use crate::c_library::printf;
use crate::game::actors::reflec;
use crate::game_state::Game;
use std::ffi::c_char;

pub(crate) unsafe fn Reflec_Shot(game: &mut Game) {
    unsafe {
        if let Some(target) = reflec::prepare_shot(game) {
            printf(b"Possible bug. What is X?\n\0" as *const u8 as *const c_char);
            reflec::absorb(target, game);
        }
    }
}
