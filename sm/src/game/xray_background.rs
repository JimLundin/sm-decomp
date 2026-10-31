//! Game's X-ray background host; the capture, overlay and restore run in
//! sm_game::xray_background.
use crate::game::player_transitions::xray::BlendPhase;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
pub(crate) use sm_game::xray_background::{
    finish_restore, restore_left, restore_right, reveal_room_block,
};

impl sm_game::xray_background::Host for Game {
    fn xray_blend_mask(&mut self) -> u16 {
        sm_game::player_transitions::xray::blend_mask(&Runtime::new(self))
    }
    fn apply_xray_blending(&mut self, phase: BlendPhase) {
        sm_game::player_transitions::xray::apply_blending(&mut Runtime::new(self), phase);
    }
    fn finish_xray(&mut self) {
        crate::game::session_runtime::finish_xray(self);
    }
    fn overlay_origin_assertion(&mut self) {
        // SAFETY: the original's assertion failure reports and does not return.
        unsafe {
            crate::native_assertion! {
                '_c2rust_label;
                b"(uint16)(layer1_y_pos >> 12) == 0\0", b"src/sm_91.c\0", 921,
                pointer c"uint8 *RomPtr_RAM(uint16_t)".as_ptr()
            };
        }
    }
}

pub unsafe fn reveal_room_items(game: &mut Game) {
    sm_game::xray_background::reveal_room_items(&mut Runtime::new(game));
}
