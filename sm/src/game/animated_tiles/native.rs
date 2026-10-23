//! Explicit remaining native effects; these are not unsafe-elimination claims.
use crate::{game_state::Game, rom::RomRecord};
pub(super) fn low_record(address: u16, game: &mut Game) -> RomRecord {
    unsafe { crate::rtl::rom_bank_record::<0x87>(address, game) }
}
pub(super) fn unreachable(game: &mut Game) -> u16 {
    unsafe { crate::rtl::Unreachable(game) as u16 }
}
pub(super) fn projectile(definition: u16, parameter: u16, game: &mut Game) {
    unsafe {
        crate::game::sm_86::SpawnEprojWithRoomGfx(definition, parameter, game);
    }
}
pub(super) fn upload(request: super::Upload, game: &mut Game) {
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
