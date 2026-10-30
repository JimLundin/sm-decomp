//! Sole actor identity, lifecycle and room presentation provenance.
use super::snapshot::checkpoint_layout;
use super::{
    access::ActorKey,
    content::{ActorTemplateId, PopulationEntryId, PopulationId, SpeciesId},
    media::AppearanceId,
};
use crate::actors::state::Actors;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceHandle {
    epoch: u64,
    slot: u16,
    generation: u64,
}
/// One checked live-epoch transition. Never clone or cache across another adoption.
pub struct PreparedCanonicalTransition {
    next_live_epoch: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActorIdentityState {
    live_epoch: u64,
    poisoned: bool,
    timeline: ActorTimeline,
    palette_residency: super::media::ActorPaletteResidency,
}
/// Snapshot-restored counters and lineage; live external handle epoch is excluded.
#[derive(Clone, Debug, PartialEq)]
pub struct ActorTimeline {
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
pub struct ActorProvenance {
    pub authored_species: Option<SpeciesId>,
    pub origin: ActorBirthOrigin,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ActorBirthOrigin {
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
pub enum ActorIdentityError {
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
pub struct PendingActorBirth {
    slot: u16,
    generation: u64,
    next_birth_order: u64,
    occupant: ActorOccupant,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ActorObservation {
    pub handle: Option<InstanceHandle>,
    pub birth_order: Option<u64>,
    pub authored_species: Option<SpeciesId>,
    pub effective_species: Option<SpeciesId>,
    pub appearance: Option<AppearanceId>,
    pub live: bool,
}
impl ActorIdentityState {
    /// Fresh detached owner. Runtime entry moves the existing owner instead.
    pub fn empty() -> Self {
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

    pub fn resolve(
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
    pub fn prepare_legacy_birth(
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

    pub fn prepare_birth(
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
    pub fn commit_birth(&mut self, birth: PendingActorBirth) -> InstanceHandle {
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
    pub fn retire(&mut self, slot: u16) {
        if let Some(identity) = self.timeline.slots.get_mut(usize::from(slot)) {
            identity.occupant = None;
        }
    }

    pub fn retire_all(&mut self) {
        for slot in &mut self.timeline.slots {
            slot.occupant = None;
        }
    }

    /// DAFF is an in-place conversion: preserve generation, provenance and appearance.
    pub fn convert_to_placeholder(&mut self, _slot: u16) {}

    pub fn observe(&self, slot: u16, live: bool, effective: Option<SpeciesId>) -> ActorObservation {
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

    pub fn timeline(&self) -> &ActorTimeline {
        &self.timeline
    }

    pub fn checked_next_epoch(&self) -> Result<u64, ActorIdentityError> {
        self.live_epoch
            .checked_add(1)
            .ok_or(ActorIdentityError::EpochOverflow)
    }

    pub fn replace_timeline(
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
    pub fn prepare_canonical_transition(
        &self,
    ) -> Result<PreparedCanonicalTransition, ActorIdentityError> {
        Ok(PreparedCanonicalTransition {
            next_live_epoch: self.checked_next_epoch()?,
        })
    }

    /// Reconstruct from the final bytes after the selected transition, without fallible
    /// allocation or general birth checks. There are at most32 compatibility occupants.
    pub fn commit_canonical_transition(
        &mut self,
        transition: PreparedCanonicalTransition,
        final_ram: &impl Fn(usize) -> u8,
    ) {
        self.replace_timeline(
            transition.next_live_epoch,
            canonical_timeline(final_ram),
            super::media::ActorPaletteResidency::default(),
        );
    }

    /// Validate the handle and selected room materials before publishing the choice.
    pub fn set_appearance(
        &mut self,
        handle: InstanceHandle,
        appearance: AppearanceId,
        actors: &Actors,
        world: &crate::room_entry::world::WorldBinding,
        image: &sm_data::cartridge::RomImage,
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

fn canonical_timeline(ram: &impl Fn(usize) -> u8) -> ActorTimeline {
    let mut next_birth_order = 0;
    let slots = core::array::from_fn(|slot| {
        let at = 0xf78 + slot * 64;
        if u16::from_le_bytes([ram(at), ram(at + 1)]) == 0 {
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
    pub fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    pub fn poison(&mut self, transition: PreparedCanonicalTransition) {
        self.live_epoch = transition.next_live_epoch;
        self.retire_all();
        self.palette_residency.clear();
        self.poisoned = true;
    }

    pub fn retain_poison(&mut self) {
        self.poisoned = true;
        self.retire_all();
        self.palette_residency.clear();
    }

    pub fn palette_residency(&self) -> &super::media::ActorPaletteResidency {
        &self.palette_residency
    }
    pub fn clear_palette_residency(&mut self) {
        self.palette_residency.clear();
    }
    pub fn clear_actor_palette_residency(&mut self) {
        self.palette_residency.clear_actor_rows();
    }
    pub fn record_prepared_palette_residency(
        &mut self,
        row: u16,
        resource: crate::room_entry::world::ResourceId,
        revision: [u8; 32],
    ) {
        self.palette_residency
            .record_prepared(row, resource, revision);
    }
    pub fn record_palette_residency(
        &mut self,
        row: u16,
        resource: Option<crate::room_entry::world::ResourceId>,
        image: sm_data::cartridge::RomImage,
    ) {
        self.palette_residency.record(row, resource, image);
    }
}

// Verification-only child can prepare overflow fixtures without production setters.
// Append to game/actors/identity.rs in the candidate verification snapshot only.
#[cfg(sm_verify_dll)]
impl ActorIdentityState {
    pub fn initializer_fixture_origin(&self, slot: u16) -> Option<ActorBirthOrigin> {
        self.timeline
            .slots
            .get(usize::from(slot))?
            .occupant
            .as_ref()
            .map(|occupant| occupant.provenance.origin.clone())
    }
}

checkpoint_layout!(actor_timeline_layout, write_actor_timeline(bytes, timeline: &ActorTimeline, live: &impl Fn(usize) -> bool) {
    bind ActorTimeline { next_birth_order, slots } = timeline;
    field next_birth_order: u64 = *next_birth_order;
    array slots: [SlotIdentity; 32] = (*slots) => ((index, slot)) {
        bind SlotIdentity { generation, occupant } = slot;
        field generation: u64 = *generation;
        variants occupant = ((live(index), occupant)) {
            0 Empty ((false, None)) => {}
            2 Untracked ((true, None)) => {}
            1 Retained ((_, Some(occupant))) => {
                bind ActorOccupant { birth_order, provenance, appearance } = occupant;
                bind ActorProvenance { authored_species, origin } = provenance;
                field birth_order: u64 = *birth_order;
                option authored_species: u32 = authored_species.map(|id| id.0);
                variants origin = (origin) {
                    0 CanonicalImport (ActorBirthOrigin::CanonicalImport) => {}
                    1 Population (ActorBirthOrigin::Population { population, entry, respawn }) => {
                        field population: u32 = population.0;
                        field entry: u32 = entry.0;
                        boolean respawn = *respawn;
                    }
                    2 Dynamic (ActorBirthOrigin::Dynamic { template, part }) => {
                        field template: u32 = template.0;
                        field part: u16 = *part;
                    }
                }
                option appearance: u32 = appearance.map(|id| id.0);
            }
        }
    }
});

impl ActorTimeline {
    #[cfg(any(feature = "frontend", sm_verify_dll))]
    pub fn serialized_layout() -> String {
        actor_timeline_layout()
    }
    pub fn encode(&self, bytes: &mut Vec<u8>, ram: &impl Fn(usize) -> u8) {
        self.encode_occupancy(bytes, &|index| {
            let at = 0xf78 + index * 64;
            [ram(at), ram(at + 1)] != [0, 0]
        });
    }

    pub fn encode_occupancy(&self, bytes: &mut Vec<u8>, live: &impl Fn(usize) -> bool) {
        write_actor_timeline(bytes, self, live);
    }

    pub fn decode(
        read: &mut super::snapshot::CheckpointReader<'_>,
        world: &crate::room_entry::world::WorldBinding,
        ram: &impl Fn(usize) -> u8,
    ) -> Result<Self, String> {
        Self::decode_occupancy(read, world, &|index| {
            let at = 0xf78 + index * 64;
            [ram(at), ram(at + 1)] != [0, 0]
        })
    }

    pub fn decode_occupancy(
        read: &mut super::snapshot::CheckpointReader<'_>,
        world: &crate::room_entry::world::WorldBinding,
        live: &impl Fn(usize) -> bool,
    ) -> Result<Self, String> {
        let next_birth_order = read.counter()?;
        let mut slots: [SlotIdentity; 32] = core::array::from_fn(|_| SlotIdentity::default());
        let mut births = std::collections::BTreeSet::new();
        for (index, slot) in slots.iter_mut().enumerate() {
            slot.generation = read.counter()?;
            let raw_live = live(index);
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

impl ActorIdentityState {
    /// A preflighted detached replacement; the supplied fresh epoch is never encoded.
    pub fn from_checkpoint(
        live_epoch: u64,
        timeline: ActorTimeline,
        palette_residency: super::media::ActorPaletteResidency,
    ) -> Self {
        Self {
            live_epoch,
            poisoned: false,
            timeline,
            palette_residency,
        }
    }
}

#[cfg(test)]
#[path = "../verification/actor_identity_checks.rs"]
mod actor_identity_checks;

#[cfg(sm_verify_dll)]
#[path = "../verification/actor_owned_checkpoint_cases.rs"]
mod owned_checkpoint_cases;
#[cfg(sm_verify_dll)]
pub use owned_checkpoint_cases::{assert_installed_epoch, assert_owned_checkpoint};
