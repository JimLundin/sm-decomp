//! Checked scroll pairs retain the original native-endian stores.
#![forbid(unsafe_code)]
use crate::game::mechanism_shaktool::{self, Access, Context, ScrollPair};
use crate::game_state::Game;
pub struct ShaktoolFields<'a>(pub &'a mut Game);
impl ShaktoolFields<'_> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
}
impl Context for ShaktoolFields<'_> {
    fn set_scroll_pair(&mut self, pair: ScrollPair, access: Access) {
        let at = match pair {
            ScrollPair::Near => 0xcd20,
            ScrollPair::Far => 0xcd22,
        };
        let value: u16 = match access {
            Access::EntranceOnly => 1,
            Access::Closed => 0,
            Access::Open => 257,
        };
        self.0.g_ram[at..at + 2].copy_from_slice(&value.to_ne_bytes());
    }
    fn power_bomb_active(&self) -> bool {
        self.word(0x592) != 0
    }
    fn player_x(&self) -> u16 {
        self.word(0xaf6)
    }
    fn mark_path_open(&mut self) {
        crate::game::progress_flags::SetEventHappened(0xd, self.0)
    }
    fn release(&mut self, slot: usize) {
        let at = 0x1c37 + 2 * slot;
        self.0.g_ram[at..at + 2].copy_from_slice(&0u16.to_le_bytes());
    }
}
pub fn begin(g: &mut Game) -> u8 {
    mechanism_shaktool::begin(&mut ShaktoolFields(g))
}
pub fn poll(j: u16, g: &mut Game) {
    mechanism_shaktool::poll(&mut ShaktoolFields(g), usize::from(j >> 1));
}
