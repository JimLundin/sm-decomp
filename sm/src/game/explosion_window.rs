//! The explosion windows live in `sm_game::explosion_window`; compat supplies its host.
#![forbid(unsafe_code)]
pub(crate) use sm_game::explosion_window::*;

impl sm_game::explosion_window::Host for crate::game_state::Game {
    fn initialize_crystal_flash(&mut self) -> bool {
        sm_game::player_movement::forced::initialize_crystal_flash(
            &mut crate::game::session_runtime::Runtime::new(self),
        )
    }
    fn window_row(&self) -> u16 {
        self.k_out
    }
    fn set_window_row(&mut self, row: u16) {
        self.k_out = row;
    }
}
