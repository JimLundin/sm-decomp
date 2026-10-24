//! Verification-only legacy entry adapter; fixture owns qualification of all inputs.
pub(crate) fn write(game: &mut crate::game_state::Game) {
    unsafe {
        crate::game::sm_a7::Kraid_Death_SinkThroughFloor(game);
    }
}
