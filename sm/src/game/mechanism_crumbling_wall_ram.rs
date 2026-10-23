//! Checked row fields and the original native-endian scroll pair.
#![forbid(unsafe_code)]
use crate::game::mechanism_crumbling_wall::{self as wall, Context};
use crate::game_state::Game;
pub struct WallFields<'a>(pub &'a mut Game);
impl WallFields<'_> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
    fn put(&mut self, at: usize, value: u16) {
        self.0.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}
impl Context for WallFields<'_> {
    type Block = u16;
    fn set_delay(&mut self, slot: usize, frames: u16) {
        self.put(0xde1c + 2 * slot, frames);
    }
    fn open_passage(&mut self) {
        self.0.g_ram[0xcd20..0xcd22].copy_from_slice(&0x101u16.to_ne_bytes());
    }
    fn block(&self, slot: usize) -> u16 {
        self.word(0x1c87 + 2 * slot)
    }
    fn below(&self, block: u16) -> u16 {
        block.wrapping_add(self.word(0x7a5).wrapping_mul(2))
    }
    fn set_block(&mut self, slot: usize, block: u16) {
        self.put(0x1c87 + 2 * slot, block);
    }
}
pub fn setup(j: u16, game: &mut Game) -> u8 {
    wall::setup(&mut WallFields(game), usize::from(j >> 1))
}
pub fn open_passage(game: &mut Game) {
    wall::open_passage(&mut WallFields(game));
}
pub fn move_down(j: u16, game: &mut Game) {
    wall::move_down(&mut WallFields(game), usize::from(j >> 1));
}
