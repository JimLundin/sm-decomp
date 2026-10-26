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
