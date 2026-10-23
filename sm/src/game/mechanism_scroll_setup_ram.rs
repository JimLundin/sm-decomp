//! Checked encoding for extension and scroll setup effects.
#![forbid(unsafe_code)]
use crate::game::mechanism_scroll_setup::{self, Context, Direction, Setup, Solidity, Tile};
use crate::game_state::Game;
pub struct ScrollFields<'a>(pub &'a mut Game);
impl ScrollFields<'_> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
    fn put(&mut self, at: usize, value: u16) {
        self.0.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}
impl Context for ScrollFields<'_> {
    type Block = u16;
    fn block(&self, slot: usize) -> u16 {
        self.word(0x1c87 + 2 * slot)
    }
    fn clear_block(&mut self, slot: usize) {
        self.put(0x1c87 + 2 * slot, 0);
    }
    fn release(&mut self, slot: usize) {
        self.put(0x1c37 + 2 * slot, 0);
    }
    fn install(&mut self, block: u16, tile: Tile) {
        let value = match tile {
            Tile::Extension(Direction::Right) => 0x50ff,
            Tile::Extension(Direction::Left) => 0x5001,
            Tile::Extension(Direction::Down) => 0xd0ff,
            Tile::Extension(Direction::Up) => 0xd001,
            Tile::Scroll(Solidity::Air) => 0x3046,
            Tile::Scroll(Solidity::Solid) => 0xb046,
        };
        crate::game::sm_84::WriteLevelDataBlockTypeAndBts(block, value, self.0);
    }
    fn active(&self, slot: usize) -> bool {
        self.word(0x1e17 + 2 * slot) & 0x8000 != 0
    }
    fn set_active(&mut self, slot: usize, active: bool) {
        self.put(0x1e17 + 2 * slot, if active { 0x8000 } else { 0 });
    }
    fn advance(&mut self, slot: usize, bytes: u16) {
        let at = 0x1d27 + 2 * slot;
        self.put(at, self.word(at).wrapping_add(bytes));
    }
    fn set_timer(&mut self, slot: usize) {
        self.put(0xde1c + 2 * slot, 1);
    }
}
pub fn setup(j: u16, kind: Setup, game: &mut Game) -> u8 {
    mechanism_scroll_setup::setup(&mut ScrollFields(game), usize::from(j >> 1), kind)
}
