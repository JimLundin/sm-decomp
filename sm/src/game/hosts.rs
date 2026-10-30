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

impl sm_game::runtime_access::RuntimeAccess for Game {
    type Runtime<'a> = crate::game::session_runtime::Runtime<'a>;
    fn runtime(&mut self) -> Self::Runtime<'_> {
        crate::game::session_runtime::Runtime::new(self)
    }
}

impl sm_game::grapple_swing::Host for Game {
    fn multiply_with_scratch(&mut self, a: u16, b: u16) -> u32 {
        Game::multiply_with_scratch(self, a, b)
    }
    fn player_bottom(&self) -> u16 {
        crate::game::session_runtime::player_bottom(self)
    }
    fn settle_grapple(&mut self) {
        crate::game::session_runtime::settle_grapple(self)
    }
}

impl sm_game::runtime_access::SessionRam for crate::game::session_runtime::Runtime<'_> {
    fn byte(&self, at: usize) -> u8 {
        crate::game::session_runtime::Runtime::byte(self, at)
    }
    fn set_byte(&mut self, at: usize, value: u8) {
        crate::game::session_runtime::Runtime::set_byte(self, at, value)
    }
    fn write_slice(&mut self, at: usize, bytes: &[u8]) {
        crate::game::session_runtime::Runtime::write_slice(self, at, bytes)
    }
}
