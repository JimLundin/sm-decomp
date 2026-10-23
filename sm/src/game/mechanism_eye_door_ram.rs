//! Checked eye-door fields and persistent progress lookup.
#![forbid(unsafe_code)]
use crate::game::mechanism_eye_door::{self, Context, Part, Side, Tile};
use crate::game_state::Game;
pub struct EyeDoorFields<'a>(pub &'a mut Game);
impl EyeDoorFields<'_> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
    fn put(&mut self, at: usize, value: u16) {
        self.0.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}
impl Context for EyeDoorFields<'_> {
    type Block = u16;
    type Row = u16;
    type Door = u16;
    type Program = u16;
    fn door(&self, slot: usize) -> u16 {
        self.word(0x1dc7 + 2 * slot)
    }
    fn opened(&mut self, door: u16) -> bool {
        let index = crate::game::progress_flags::PrepareBitAccess(door, self.0);
        self.word(0x5e7) & u16::from(self.0.g_ram[0xd8b0 + usize::from(index)]) != 0
    }
    fn block(&self, slot: usize) -> u16 {
        self.word(0x1c87 + 2 * slot)
    }
    fn set_block(&mut self, slot: usize, block: u16) {
        self.put(0x1c87 + 2 * slot, block);
    }
    fn row(&self) -> u16 {
        self.word(0x7a5).wrapping_mul(2)
    }
    fn above(&self, block: u16, row: u16) -> u16 {
        block.wrapping_sub(row)
    }
    fn below(&self, block: u16, row: u16) -> u16 {
        block.wrapping_add(row)
    }
    fn install(&mut self, block: u16, tile: Tile) {
        let value = match tile {
            Tile::Eye => 0xc044,
            Tile::Body => 0xa000,
            Tile::Blue(Side::Right) => 0xc041,
            Tile::Blue(Side::Left) => 0xc040,
            Tile::Extension(distance) => 0xd000 | u16::from(0u8.wrapping_sub(distance)),
        };
        crate::game::sm_84::WriteLevelDataBlockTypeAndBts(block, value, self.0);
    }
    fn stop_watching(&mut self, slot: usize) {
        self.put(0x1cd7 + 2 * slot, 0xd779);
    }
    fn link(&self, slot: usize) -> u16 {
        self.word(0xdebc + 2 * slot)
    }
    fn set_program(&mut self, slot: usize, program: u16) {
        self.put(0x1d27 + 2 * slot, program);
    }
    fn set_timer(&mut self, slot: usize, timer: u16) {
        self.put(0xde1c + 2 * slot, timer);
    }
}
pub fn begin(j: u16, part: Part, game: &mut Game) -> u8 {
    mechanism_eye_door::begin(&mut EyeDoorFields(game), usize::from(j >> 1), part)
}
pub fn wake(j: u16, game: &mut Game) {
    mechanism_eye_door::wake(&mut EyeDoorFields(game), usize::from(j >> 1));
}
pub fn convert(j: u16, side: Side, game: &mut Game) {
    mechanism_eye_door::convert(&mut EyeDoorFields(game), usize::from(j >> 1), side);
}
