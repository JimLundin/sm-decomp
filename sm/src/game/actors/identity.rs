//! Sole actor identity, lifecycle and room presentation provenance.
#![forbid(unsafe_code)]
use super::{
    access::ActorKey,
    content::{ActorTemplateId, PopulationEntryId, PopulationId, SpeciesId},
    media::AppearanceId,
    state::Actors,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct InstanceHandle {
    epoch: u64,
    slot: u16,
    generation: u64,
}
/// One checked live-epoch transition. Never clone or cache across another adoption.
pub(crate) struct PreparedCanonicalTransition {
    next_live_epoch: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ActorIdentityState {
    live_epoch: u64,
    poisoned: bool,
    timeline: ActorTimeline,
    palette_residency: super::media::ActorPaletteResidency,
}
/// Snapshot-restored counters and lineage; live external handle epoch is excluded.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct ActorTimeline {
    next_birth_order: u64,
    slots: [SlotIdentity; 32],
}
#[derive(Clone, Debug, Default, PartialEq)]
struct SlotIdentity {
    generation: u64,
    occupant: Option<ActorOccupant>,
}
#[derive(Clone, Debug, PartialEq)]
struct ActorOccupant {
    birth_order: u64,
    provenance: ActorProvenance,
    appearance: Option<AppearanceId>,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct ActorProvenance {
    pub(super) authored_species: Option<SpeciesId>,
    pub(super) origin: ActorBirthOrigin,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) enum ActorBirthOrigin {
    Population {
        population: PopulationId,
        entry: PopulationEntryId,
        respawn: bool,
    },
    Dynamic {
        template: ActorTemplateId,
        part: u16,
    },
    CanonicalImport,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ActorIdentityError {
    Poisoned,
    StaleEpoch,
    StaleGeneration,
    Vacant,
    InvalidSlot,
    GenerationOverflow,
    OrderOverflow,
    EpochOverflow,
    MissingPayload,
    BindingMismatch,
    Content(super::content::ActorContentError),
}
/// Checked prospective metadata only; not published before place_entry's definition store.
pub(super) struct PendingActorBirth {
    slot: u16,
    generation: u64,
    next_birth_order: u64,
    occupant: ActorOccupant,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ActorObservation {
    pub handle: Option<InstanceHandle>,
    pub birth_order: Option<u64>,
    pub authored_species: Option<SpeciesId>,
    pub effective_species: Option<SpeciesId>,
    pub appearance: Option<AppearanceId>,
    pub live: bool,
}
impl ActorIdentityState {
    /// Fresh detached owner. Runtime entry moves the existing owner instead.
    pub(crate) fn empty() -> Self {
        Self {
            live_epoch: 0,
            poisoned: false,
            palette_residency: super::media::ActorPaletteResidency::default(),
            timeline: ActorTimeline {
                next_birth_order: 0,
                slots: core::array::from_fn(|_| SlotIdentity::default()),
            },
        }
    }

    pub(super) fn resolve(
        &self,
        handle: InstanceHandle,
        actors: &Actors,
    ) -> Result<ActorKey, ActorIdentityError> {
        if self.poisoned {
            return Err(ActorIdentityError::Poisoned);
        }
        if handle.epoch != self.live_epoch {
            return Err(ActorIdentityError::StaleEpoch);
        }
        let slot = usize::from(handle.slot);
        let identity = self
            .timeline
            .slots
            .get(slot)
            .ok_or(ActorIdentityError::InvalidSlot)?;
        if identity.generation != handle.generation {
            return Err(ActorIdentityError::StaleGeneration);
        }
        if identity.occupant.is_none() || actors.slots[slot].definition == 0 {
            return Err(ActorIdentityError::Vacant);
        }
        Ok(ActorKey::from_legacy(handle.slot * 64))
    }

    /// Exceptional windows still perform byte behavior, but have no nominal slot metadata.
    pub(super) fn prepare_legacy_birth(
        &self,
        key: ActorKey,
        provenance: ActorProvenance,
        appearance: Option<AppearanceId>,
    ) -> Result<Option<PendingActorBirth>, ActorIdentityError> {
        let index = key.byte_index();
        if index & 63 != 0 || index >= 32 * 64 {
            return Ok(None);
        }
        self.prepare_birth(index / 64, provenance, appearance)
            .map(Some)
    }

    pub(super) fn prepare_birth(
        &self,
        slot: u16,
        provenance: ActorProvenance,
        appearance: Option<AppearanceId>,
    ) -> Result<PendingActorBirth, ActorIdentityError> {
        if self.poisoned {
            return Err(ActorIdentityError::Poisoned);
        }
        let old = self
            .timeline
            .slots
            .get(usize::from(slot))
            .ok_or(ActorIdentityError::InvalidSlot)?;
        let generation = old
            .generation
            .checked_add(1)
            .ok_or(ActorIdentityError::GenerationOverflow)?;
        let next_birth_order = self
            .timeline
            .next_birth_order
            .checked_add(1)
            .ok_or(ActorIdentityError::OrderOverflow)?;
        Ok(PendingActorBirth {
            slot,
            generation,
            next_birth_order,
            occupant: ActorOccupant {
                birth_order: self.timeline.next_birth_order,
                provenance,
                appearance,
            },
        })
    }

    /// Called immediately after the first definition store; no callback may intervene.
    pub(super) fn commit_birth(&mut self, birth: PendingActorBirth) -> InstanceHandle {
        let slot = &mut self.timeline.slots[usize::from(birth.slot)];
        debug_assert_eq!(slot.generation.checked_add(1), Some(birth.generation));
        debug_assert_eq!(
            self.timeline.next_birth_order.checked_add(1),
            Some(birth.next_birth_order)
        );
        slot.generation = birth.generation;
        slot.occupant = Some(birth.occupant);
        self.timeline.next_birth_order = birth.next_birth_order;
        InstanceHandle {
            epoch: self.live_epoch,
            slot: birth.slot,
            generation: birth.generation,
        }
    }

    /// Ordinal slots only. Broad legacy windows carry no nominal metadata.
    pub(super) fn retire(&mut self, slot: u16) {
        if let Some(identity) = self.timeline.slots.get_mut(usize::from(slot)) {
            identity.occupant = None;
        }
    }

    pub(super) fn retire_all(&mut self) {
        for slot in &mut self.timeline.slots {
            slot.occupant = None;
        }
    }

    /// DAFF is an in-place conversion: preserve generation, provenance and appearance.
    pub(super) fn convert_to_placeholder(&mut self, _slot: u16) {}

    pub(in crate::game) fn observe(
        &self,
        slot: u16,
        live: bool,
        effective: Option<SpeciesId>,
    ) -> ActorObservation {
        let identity = self.timeline.slots.get(usize::from(slot));
        let occupant = identity.and_then(|value| value.occupant.as_ref());
        ActorObservation {
            handle: if live && occupant.is_some() {
                identity.map(|value| InstanceHandle {
                    epoch: self.live_epoch,
                    slot,
                    generation: value.generation,
                })
            } else {
                None
            },
            birth_order: occupant.map(|value| value.birth_order),
            authored_species: occupant.and_then(|value| value.provenance.authored_species),
            effective_species: if live { effective } else { None },
            appearance: occupant.and_then(|value| value.appearance),
            live,
        }
    }

    pub(super) fn timeline(&self) -> &ActorTimeline {
        &self.timeline
    }

    pub(super) fn checked_next_epoch(&self) -> Result<u64, ActorIdentityError> {
        self.live_epoch
            .checked_add(1)
            .ok_or(ActorIdentityError::EpochOverflow)
    }

    pub(super) fn replace_timeline(
        &mut self,
        epoch: u64,
        timeline: ActorTimeline,
        residency: super::media::ActorPaletteResidency,
    ) {
        debug_assert_eq!(self.live_epoch.checked_add(1), Some(epoch));
        self.live_epoch = epoch;
        self.poisoned = false;
        self.timeline = timeline;
        self.palette_residency = residency;
    }

    /// Validate epoch capacity before any reset/load/device mutation starts.
    pub(crate) fn prepare_canonical_transition(
        &self,
    ) -> Result<PreparedCanonicalTransition, ActorIdentityError> {
        Ok(PreparedCanonicalTransition {
            next_live_epoch: self.checked_next_epoch()?,
        })
    }

    /// Reconstruct from the final bytes after the selected transition, without fallible
    /// allocation or general birth checks. There are at most32 compatibility occupants.
    pub(crate) fn commit_canonical_transition(
        &mut self,
        transition: PreparedCanonicalTransition,
        final_ram: &[u8; 131072],
    ) {
        self.replace_timeline(
            transition.next_live_epoch,
            canonical_timeline(final_ram),
            super::media::ActorPaletteResidency::default(),
        );
    }

    /// Fixed byte-image adoption may construct its complete paired restore in advance.
    pub(crate) fn prepare_canonical_adoption(
        &self,
        ram: &[u8; 131072],
    ) -> Result<super::snapshot::PreparedActorRestore, ActorIdentityError> {
        let transition = self.prepare_canonical_transition()?;
        Ok(super::snapshot::PreparedActorRestore::new(
            transition.next_live_epoch,
            canonical_timeline(ram),
            super::media::ActorPaletteResidency::default(),
        ))
    }

    /// Validate the handle and selected room materials before publishing the choice.
    pub(crate) fn set_appearance(
        &mut self,
        handle: InstanceHandle,
        appearance: AppearanceId,
        actors: &Actors,
        world: &crate::game::room_entry::world::WorldBinding,
        image: &crate::rom::RomImage,
    ) -> Result<(), ActorIdentityError> {
        self.resolve(handle, actors)?;
        world.actor_media().validate_appearance_residency(
            appearance,
            &self.palette_residency,
            image,
        )?;
        self.timeline.slots[usize::from(handle.slot)]
            .occupant
            .as_mut()
            .expect("resolved actor occurrence")
            .appearance = Some(appearance);
        Ok(())
    }
}

impl From<super::content::ActorContentError> for ActorIdentityError {
    fn from(error: super::content::ActorContentError) -> Self {
        Self::Content(error)
    }
}

fn canonical_timeline(ram: &[u8; 131072]) -> ActorTimeline {
    let mut next_birth_order = 0;
    let slots = core::array::from_fn(|slot| {
        let at = 0xf78 + slot * 64;
        if u16::from_le_bytes([ram[at], ram[at + 1]]) == 0 {
            SlotIdentity::default()
        } else {
            let birth_order = next_birth_order;
            next_birth_order += 1;
            SlotIdentity {
                generation: 1,
                occupant: Some(ActorOccupant {
                    birth_order,
                    provenance: ActorProvenance {
                        authored_species: None,
                        origin: ActorBirthOrigin::CanonicalImport,
                    },
                    appearance: None,
                }),
            }
        }
    });
    ActorTimeline {
        next_birth_order,
        slots,
    }
}

impl ActorIdentityState {
    pub(crate) fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    pub(crate) fn poison(&mut self, transition: PreparedCanonicalTransition) {
        self.live_epoch = transition.next_live_epoch;
        self.retire_all();
        self.palette_residency.clear();
        self.poisoned = true;
    }

    pub(crate) fn retain_poison(&mut self) {
        self.poisoned = true;
        self.retire_all();
        self.palette_residency.clear();
    }

    pub(super) fn palette_residency(&self) -> &super::media::ActorPaletteResidency {
        &self.palette_residency
    }
    pub(in crate::game) fn clear_palette_residency(&mut self) {
        self.palette_residency.clear();
    }
    pub(super) fn clear_actor_palette_residency(&mut self) {
        self.palette_residency.clear_actor_rows();
    }
    pub(in crate::game) fn record_palette_residency(
        &mut self,
        row: u16,
        resource: Option<crate::game::room_entry::world::ResourceId>,
        image: crate::rom::RomImage,
    ) {
        self.palette_residency.record(row, resource, image);
    }
}

// Verification-only child can prepare overflow fixtures without production setters.
#[cfg(test)]
#[path = "../../verification/actor_identity_checks.rs"]
mod actor_identity_checks;

// Append to game/actors/identity.rs in the candidate verification snapshot only.
#[cfg(sm_verify_dll)]
impl ActorIdentityState {
    pub(super) fn initializer_fixture_origin(&self, slot: u16) -> Option<ActorBirthOrigin> {
        self.timeline
            .slots
            .get(usize::from(slot))?
            .occupant
            .as_ref()
            .map(|occupant| occupant.provenance.origin.clone())
    }
}

impl ActorTimeline {
    pub(super) fn encode(&self, bytes: &mut Vec<u8>, ram: &[u8; 131072]) {
        bytes.extend_from_slice(&self.next_birth_order.to_le_bytes());
        for (index, slot) in self.slots.iter().enumerate() {
            bytes.extend_from_slice(&slot.generation.to_le_bytes());
            let at = 0xf78 + index * 64;
            let raw_live = ram[at..at + 2] != [0, 0];
            // Unknown compatibility-live slots are possible through odd/raw aliases.
            bytes.push(if slot.occupant.is_some() {
                1
            } else if raw_live {
                2
            } else {
                0
            });
            let Some(occupant) = &slot.occupant else {
                continue;
            };
            bytes.extend_from_slice(&occupant.birth_order.to_le_bytes());
            super::snapshot::optional_id(
                bytes,
                occupant.provenance.authored_species.map(|id| id.0),
            );
            match occupant.provenance.origin {
                ActorBirthOrigin::CanonicalImport => bytes.push(0),
                ActorBirthOrigin::Population {
                    population,
                    entry,
                    respawn,
                } => {
                    bytes.push(1);
                    bytes.extend_from_slice(&population.0.to_le_bytes());
                    bytes.extend_from_slice(&entry.0.to_le_bytes());
                    bytes.push(u8::from(respawn));
                }
                ActorBirthOrigin::Dynamic { template, part } => {
                    bytes.push(2);
                    bytes.extend_from_slice(&template.0.to_le_bytes());
                    bytes.extend_from_slice(&part.to_le_bytes());
                }
            }
            super::snapshot::optional_id(bytes, occupant.appearance.map(|id| id.0));
        }
    }

    pub(super) fn decode(
        read: &mut super::snapshot::CheckpointReader<'_>,
        world: &crate::game::room_entry::world::WorldBinding,
        ram: &[u8; 131072],
    ) -> Result<Self, String> {
        let next_birth_order = read.counter()?;
        let mut slots: [SlotIdentity; 32] = core::array::from_fn(|_| SlotIdentity::default());
        let mut births = std::collections::BTreeSet::new();
        for (index, slot) in slots.iter_mut().enumerate() {
            slot.generation = read.counter()?;
            let at = 0xf78 + index * 64;
            let raw_live = ram[at..at + 2] != [0, 0];
            match read.byte()? {
                0 if !raw_live => continue,
                2 if raw_live => continue,
                // A retained occurrence may have a raw-zero definition or DAFF conversion.
                1 => {}
                _ => return Err("invalid actor checkpoint occupant tag".into()),
            }
            let birth_order = read.counter()?;
            if slot.generation == 0
                || birth_order >= next_birth_order
                || !births.insert(birth_order)
            {
                return Err("invalid actor checkpoint generation or birth order".into());
            }
            let authored_species = read.optional_id()?.map(SpeciesId);
            let origin = match read.byte()? {
                0 => ActorBirthOrigin::CanonicalImport,
                1 => {
                    let population = PopulationId(read.id()?);
                    let entry = PopulationEntryId(read.id()?);
                    let respawn = match read.byte()? {
                        0 => false,
                        1 => true,
                        _ => return Err("invalid actor checkpoint respawn tag".into()),
                    };
                    ActorBirthOrigin::Population {
                        population,
                        entry,
                        respawn,
                    }
                }
                2 => ActorBirthOrigin::Dynamic {
                    template: ActorTemplateId(read.id()?),
                    part: read.word()?,
                },
                _ => return Err("invalid actor checkpoint origin".into()),
            };
            let appearance = read.optional_id()?.map(AppearanceId);
            if let Some(definitions) = world.actors().definitions() {
                if authored_species.is_some_and(|id| !definitions.species.contains_key(&id)) {
                    return Err("actor checkpoint species is unavailable".into());
                }
                match origin {
                    ActorBirthOrigin::Population {
                        population, entry, ..
                    } => {
                        if !definitions
                            .populations
                            .get(&population)
                            .is_some_and(|value| {
                                value.entries.iter().any(|value| {
                                    value.id == entry && authored_species == Some(value.species)
                                })
                            })
                        {
                            return Err("actor checkpoint population lineage is unavailable".into());
                        }
                    }
                    ActorBirthOrigin::Dynamic { template, part } => {
                        if !definitions.templates.get(&template).is_some_and(|entry| {
                            authored_species == Some(entry.species)
                                && definitions
                                    .species
                                    .get(&entry.species)
                                    .is_some_and(|species| part < species.multipart_count)
                        }) {
                            return Err("actor checkpoint template lineage is unavailable".into());
                        }
                    }
                    ActorBirthOrigin::CanonicalImport if authored_species.is_some() => {
                        return Err(
                            "canonical actor checkpoint cannot have authored species".into()
                        );
                    }
                    ActorBirthOrigin::CanonicalImport => {}
                }
            } else if authored_species.is_some()
                || !matches!(origin, ActorBirthOrigin::CanonicalImport)
            {
                return Err("actor checkpoint has authored lineage without a catalog".into());
            }
            if let Some(definitions) = world.actor_media().definitions() {
                if appearance.is_some_and(|id| !definitions.appearances.contains_key(&id)) {
                    return Err("actor checkpoint appearance is unavailable".into());
                }
            } else if appearance.is_some() {
                return Err("actor checkpoint has an appearance without a catalog".into());
            }
            slot.occupant = Some(ActorOccupant {
                birth_order,
                provenance: ActorProvenance {
                    authored_species,
                    origin,
                },
                appearance,
            });
        }
        Ok(Self {
            next_birth_order,
            slots,
        })
    }
}
