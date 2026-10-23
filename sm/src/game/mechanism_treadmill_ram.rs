//! Preserve the clear cursor's wrap, widened installation and ordered field writes.
#![forbid(unsafe_code)]
use crate::game::mechanism_treadmill::{self as treadmill, Direction, Motion, Surface, Terrain};
use crate::game_state::Game;
pub struct TreadmillFields<'a>(pub &'a mut Game);
impl TreadmillFields<'_> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
    fn put(&mut self, at: usize, value: u16) {
        self.0.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}
impl Terrain for TreadmillFields<'_> {
    type Block = usize;
    fn block(&self, slot: usize) -> usize {
        usize::from(self.word(0x1c87 + 2 * slot))
    }
    fn next_clear(&self, block: usize) -> usize {
        usize::from((block as u16).wrapping_add(2))
    }
    fn row_cell(&self, origin: usize, index: usize) -> usize {
        origin + 2 * index
    }
    fn set_level(&mut self, block: usize, surface: Surface) {
        let at = 0x10002 + (block & !1);
        let bytes = match surface {
            Surface::Air => 0xffu16.to_le_bytes(),
            Surface::Conveyor => 0x30ffu16.to_ne_bytes(),
        };
        self.0.g_ram[at..at + 2].copy_from_slice(&bytes);
    }
    fn set_direction(&mut self, block: usize, direction: Direction) {
        self.0.g_ram[0x16402 + (block >> 1)] = match direction {
            Direction::Right => 8,
            Direction::Left => 9,
        };
    }
}
impl Motion for TreadmillFields<'_> {
    fn area(&self) -> u16 {
        self.word(0x79f)
    }
    fn boss_defeated(&mut self) -> bool {
        crate::game::progress_flags::CheckBossBitForCurArea(1, self.0) & 1 != 0
    }
    fn extra_run_speed(&self) -> u16 {
        self.word(0xb2e)
    }
    fn clear_fraction(&mut self) {
        self.put(0xb56, 0);
    }
    fn set_displacement(&mut self, pixels: i16) {
        self.put(0xb58, pixels as u16);
    }
    fn normal_speed(&mut self) {
        let normal = crate::game::constants::bank_90::addr_kSamusSpeedTable_Normal_X as u16;
        self.put(0xa6c, normal.wrapping_add(12));
    }
}
pub fn setup(j: u16, game: &mut Game) -> u8 {
    treadmill::setup(&mut TreadmillFields(game), usize::from(j >> 1))
}
pub fn activate(j: u16, direction: Direction, game: &mut Game) {
    treadmill::activate(&mut TreadmillFields(game), usize::from(j >> 1), direction);
}
pub fn carry(direction: Direction, game: &mut Game) {
    treadmill::carry(&mut TreadmillFields(game), direction);
}
