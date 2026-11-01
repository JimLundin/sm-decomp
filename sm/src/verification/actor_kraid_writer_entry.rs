//! Verification-only legacy entry adapter; fixture owns qualification of all inputs.
pub(crate) fn write(game: &mut crate::game_state::Game) {
    let mut runtime = crate::game::session_runtime::Runtime::new(game);
    sm_game::actors::kraid::sink_through_floor(&mut runtime);
}
