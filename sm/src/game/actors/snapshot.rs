//! Paired timeline/presentation snapshots; live handle epochs never rewind.
#![forbid(unsafe_code)]
use super::{
    content::GameplayRevision,
    identity::{ActorIdentityError, ActorIdentityState, ActorTimeline},
    media::MediaRevision,
};
use crate::game::room_entry::world::WorldBinding;
use std::sync::Arc;
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ActorSnapshotPayload {
    timeline: ActorTimeline,
    binding: Arc<WorldBinding>,
    gameplay: GameplayRevision,
    media: MediaRevision,
    residency: super::media::ActorPaletteResidency,
}
/// Constructed only after binding/payload/checked-next-epoch preflight, before RAM writes.
pub(crate) struct PreparedActorRestore {
    next_live_epoch: u64,
    timeline: ActorTimeline,
    residency: super::media::ActorPaletteResidency,
}
impl ActorSnapshotPayload {
    pub(crate) fn capture(state: &ActorIdentityState, world: &Arc<WorldBinding>) -> Self {
        Self {
            timeline: state.timeline().clone(),
            residency: state.palette_residency().clone(),
            binding: world.clone(),
            gameplay: world
                .actor_gameplay_revision()
                .unwrap_or(GameplayRevision([0; 32])),
            media: world
                .actor_media_revision()
                .unwrap_or(MediaRevision([0; 32])),
        }
    }

    pub(crate) fn preflight(
        &self,
        current: &ActorIdentityState,
        world: &Arc<WorldBinding>,
    ) -> Result<PreparedActorRestore, ActorIdentityError> {
        if !Arc::ptr_eq(&self.binding, world)
            || self.gameplay
                != world
                    .actor_gameplay_revision()
                    .unwrap_or(GameplayRevision([0; 32]))
            || self.media
                != world
                    .actor_media_revision()
                    .unwrap_or(MediaRevision([0; 32]))
        {
            return Err(ActorIdentityError::BindingMismatch);
        }
        Ok(PreparedActorRestore::new(
            current.checked_next_epoch()?,
            self.timeline.clone(),
            self.residency.clone(),
        ))
    }
}
impl PreparedActorRestore {
    pub(super) fn new(
        next_live_epoch: u64,
        timeline: ActorTimeline,
        residency: super::media::ActorPaletteResidency,
    ) -> Self {
        Self {
            next_live_epoch,
            timeline,
            residency,
        }
    }

    /// Noninterleaved with preflight and the paired RAM adoption; no fallible work remains.
    pub(crate) fn commit(self, current: &mut ActorIdentityState) {
        current.replace_timeline(self.next_live_epoch, self.timeline, self.residency);
    }
}

/// Game-owned companions to CPU snapshots: never embedded in raw-copied Snapshot.
#[derive(Clone, Default)]
pub(crate) struct ActorSnapshotSidecars {
    pub(crate) before: Option<ActorSnapshotPayload>,
    pub(crate) theirs: Option<ActorSnapshotPayload>,
    pub(crate) mine: Option<ActorSnapshotPayload>,
}
/// Paired semantic payload for a replay/checkpoint RAM image, with existing CPU authority.
#[derive(Clone)]
pub(crate) struct ActorCheckpointPayload {
    pub(crate) actors: ActorSnapshotPayload,
}
