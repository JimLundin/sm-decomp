//! Checked compatibility access to persistent event and area-boss flags.
//! Queries retain their scratch-mask writes; RAM remains the state owner.
#![forbid(unsafe_code)]
use crate::{
    game_state::Game,
    ram_map::{checked_word, words as ram},
};

pub fn PrepareBitAccess(bit: u16, game: &mut Game) -> u16 {
    checked_word!(game.bitmask = 1u16 << (bit & 7));
    bit >> 3
}

pub fn SetBossBitForCurArea(mask: u16, game: &mut Game) {
    game.g_ram[ram::bitmask] = mask as u8;
    let address = 0xd828 + usize::from(checked_word!(game.area_index));
    game.g_ram[address] |= mask as u8;
}
pub fn CheckBossBitForCurArea(mask: u16, game: &mut Game) -> u8 {
    // Boss operations leave the high scratch byte untouched.
    game.g_ram[ram::bitmask] = mask as u8;
    let address = 0xd828 + usize::from(checked_word!(game.area_index));
    u8::from(game.g_ram[address] & mask as u8 != 0)
}

pub fn SetEventHappened(event: u16, game: &mut Game) {
    let index = usize::from(PrepareBitAccess(event, game));
    game.g_ram[0xd820 + index] |= checked_word!(game.bitmask) as u8;
}
pub fn ClearEventHappened(event: u16, game: &mut Game) {
    let index = usize::from(PrepareBitAccess(event, game));
    game.g_ram[0xd820 + index] &= !(checked_word!(game.bitmask) as u8);
}
pub fn CheckEventHappened(event: u16, game: &mut Game) -> u16 {
    let index = usize::from(PrepareBitAccess(event, game));
    u16::from(checked_word!(game.bitmask) & u16::from(game.g_ram[0xd820 + index]) != 0)
}
