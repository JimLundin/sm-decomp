//! Controller-selected horizontal transitions share the complete motion operation.
#![forbid(unsafe_code)]
use super::{
    effects::Host,
    operations::{
        self, ControlResult, HorizontalDirection, HorizontalRequest, HorizontalTransition,
    },
};

pub(crate) fn horizontal_control<H: Host>(
    h: &mut H,
    transition: HorizontalTransition,
) -> ControlResult {
    if let HorizontalTransition::Move(request) = transition {
        return ControlResult::Moved(operations::horizontal(h, request));
    }
    h.session_mut().player.contact.base_run.whole = 0;
    h.session_mut().player.contact.base_run.fraction = 0;
    h.session_mut().player.collision.flags = 0;
    if matches!(transition, HorizontalTransition::ClearAndSkip) {
        return ControlResult::Skipped;
    }
    if matches!(
        transition,
        HorizontalTransition::ClearAccelerationAndMoveZero
    ) {
        h.session_mut().player.contact.acceleration_mode = 0;
    }
    ControlResult::Moved(operations::horizontal(
        h,
        HorizontalRequest {
            base: 0,
            direction: HorizontalDirection::FromFacingAndAcceleration,
        },
    ))
}
