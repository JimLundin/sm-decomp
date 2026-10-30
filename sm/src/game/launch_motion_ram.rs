//! Transitional serialization for the launch-motion model. No decoded state or
//! RAM borrow survives a call back into gameplay. This adapter is removable when
//! Samus owns LaunchMotion directly; it is not the final engine state storage.
#![forbid(unsafe_code)]
use sm_game::launch_motion::{LaunchMotion, MovementDirection};

/// Record one launch-motion step, after the verifier observes it.
pub(crate) fn record_observed(
    motion: &mut LaunchMotion,
    direction: MovementDirection,
    amount: i32,
) {
    #[cfg(any(test, sm_verify_dll))]
    crate::game::player_collision_owned_checks::record_launch(direction, amount);
    motion.record_movement(direction, amount);
}
