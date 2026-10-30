//! Room scrolling lives in `sm_game::room_scrolls`; compat supplies its host.
#![forbid(unsafe_code)]
pub(crate) use sm_game::room_scrolls::*;

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
