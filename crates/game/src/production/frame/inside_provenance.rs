//! Current whole Inside provenance retained for later ordinary movement.

use super::{Caller, Content, Dependency, State, movement_stage};
use crate::production::{ProductionFault, Unsupported};

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(
        Dependency::PlayerContinuationPreparation,
    ))
}

impl State {
    pub(super) fn validate_room_owner(
        &self,
        content: &Content,
    ) -> Result<data::id::RoomId, ProductionFault> {
        let room = content.movement_room.as_ref().ok_or_else(missing)?;
        let current = &self.inside_world.as_ref().ok_or_else(missing)?.0;
        if !current.matches_room(room) {
            return Err(missing());
        }
        current.room_identity().ok_or_else(missing)
    }

    pub(super) fn validate_inside_provenance(
        &self,
        content: &Content,
    ) -> Result<&movement_stage::Completion, ProductionFault> {
        self.validate_room_owner(content)?;
        let Caller::PlayerContinuation { completed, .. } = &self.caller else {
            return Err(missing());
        };
        let inside = self.inside_completed.as_ref().ok_or_else(missing)?;
        if inside.work != crate::production::timing::WorkId(15) {
            return Err(missing());
        }
        content
            .inside_physical
            .as_ref()
            .ok_or_else(missing)?
            .physical(
                &inside.receipt,
                &inside.retained,
                &content.movement,
                &self.inside_world.as_ref().ok_or_else(missing)?.0,
                completed.0.pose_input.as_ref().ok_or_else(missing)?,
            )
            .map_err(|_| missing())?;
        Ok(inside)
    }
}
