//! Game's X-ray background host; the capture, overlay and restore run in
//! sm_game::xray_background.
use crate::game::player_transitions::xray::BlendPhase;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
pub(crate) use sm_game::xray_background::{
    finish_restore, restore_left, restore_right, reveal_room_block,
};

/// The X-ray background runs on the lease: the blend mask, blending and the
/// scope's finish are the lease's own player view.
impl sm_game::xray_background::Host for Runtime<'_> {
    fn xray_blend_mask(&mut self) -> u16 {
        sm_game::player_transitions::xray::blend_mask(self)
    }
    fn apply_xray_blending(&mut self, phase: BlendPhase) {
        sm_game::player_transitions::xray::apply_blending(self, phase);
    }
    fn finish_xray(&mut self) {
        crate::game::player_transitions::xray::finish(self);
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

/// The VRAM readback queue's records ($7E:0340) and tail ($7E:0360).
const READBACK_RECORDS: usize = 0x340;
const READBACK_TAIL: usize = 0x360;

/// The readback queue on the lease: its tail and records are the Session's
/// projection of $7E:0340..$0361, stored a byte at a time as on the game.
impl sm_game::display_refresh::ReadbackQueue for Runtime<'_> {
    fn readback_queue_tail(&self) -> u16 {
        self.word(READBACK_TAIL)
    }
    fn set_readback_queue_tail(&mut self, tail: u16) {
        let [low, high] = tail.to_le_bytes();
        self.set_byte(READBACK_TAIL, low);
        self.set_byte(READBACK_TAIL + 1, high);
    }
    fn store_readback_byte(&mut self, offset: usize, value: u8) {
        self.set_byte(READBACK_RECORDS + offset, value);
    }
}
