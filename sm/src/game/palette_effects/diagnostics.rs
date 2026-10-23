//! Remaining native diagnostics; fixed C strings and checked ROM records.
//! These calls are retained explicitly, not counted as unsafe elimination.
use crate::{game_state::Game, rom::RomRecord};
pub(super) fn low_record(address: u16, game: &mut Game) -> RomRecord {
    unsafe { crate::rtl::rom_bank_record::<0x8d>(address, game) }
}
pub(super) fn unreachable(game: &mut Game) {
    unsafe {
        crate::rtl::Unreachable(game);
    }
}
