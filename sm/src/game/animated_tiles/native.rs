//! Qualified legacy diagnostics and synchronous DMA submission.
use crate::game_state::Game;
pub(crate) fn unreachable(game: &mut Game) -> u16 {
    crate::rtl::Unreachable(game) as u16
}
pub(crate) fn upload(request: sm_game::visual_state::Upload, game: &mut Game) {
    use crate::game::prelude::*;
    unsafe {
        crate::rtl::register_writes! { game;
            word(A1T0L as u16, request.source); byte(A1B0 as u16, 0x87);
            word(DMAP0 as u16, 0x1801); word(DAS0L as u16, request.size);
            word(VMADDL as u16, request.vram); byte(VMAIN as u16, 0x80);
            byte(MDMAEN as u16, 1);
        }
    }
}
