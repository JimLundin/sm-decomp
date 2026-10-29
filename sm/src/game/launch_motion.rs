//! Compatibility verification observes before the shared owned launch writer.
#![forbid(unsafe_code)]
pub(crate) use sm_game::launch_motion::*;

pub(crate) fn record_movement(
    motion: &mut LaunchMotion,
    direction: MovementDirection,
    displacement: i32,
) {
    #[cfg(any(test, sm_verify_dll))]
    crate::game::player_collision_owned_checks::record_launch(direction, displacement);
    motion.record_movement(direction, displacement);
}
