//! Game's implementations of the sm_game host traits whose modules moved.
#![forbid(unsafe_code)]
use crate::game_state::Game;
use sm_game::display::transfer::ScrollingLayout;
use sm_game::player_collision::operations::Axis;

impl sm_game::room_scrolls::Host for Game {
    fn unreachable(&mut self) -> bool {
        crate::rtl::Unreachable(self)
    }
    fn record_camera_distance(&mut self, axis: Axis) {
        sm_game::player_movement::speed::record_camera_distance(
            crate::game::session_runtime::Runtime::new(self).session_mut(),
            axis,
        );
    }
    fn queue_scrolling_tiles(&mut self, k: u16, layout: ScrollingLayout) {
        crate::game::display_refresh::queue_scrolling_tiles(k, layout, self);
    }
}

impl sm_game::explosion_window::Host for Game {
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
