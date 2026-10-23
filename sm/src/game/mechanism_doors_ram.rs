//! Checked views for door setup while encoded runtime fields are authoritative.
#![forbid(unsafe_code)]
use crate::game::mechanism_doors::{self, Context, Setup};
use crate::game_state::Game;

pub struct DoorFields<'a>(pub &'a mut Game);

impl DoorFields<'_> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
    fn put(&mut self, at: usize, value: u16) {
        self.0.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}
impl Context for DoorFields<'_> {
    type Block = u16;
    fn block(&self, slot: usize) -> u16 {
        self.word(0x1c87 + 2 * slot)
    }
    fn below(&self, block: u16) -> u16 {
        block.wrapping_add(self.word(0x7a5).wrapping_mul(2))
    }
    fn clear_block(&mut self, slot: usize) {
        self.put(0x1c87 + 2 * slot, 0);
    }
    fn release(&mut self, slot: usize) {
        self.put(0x1c37 + 2 * slot, 0);
    }
    fn argument_high(&self, slot: usize) -> u8 {
        self.0.g_ram[0x1dc8 + 2 * slot]
    }
    fn argument(&self, slot: usize) -> u16 {
        self.word(0x1dc7 + 2 * slot)
    }
    fn set_argument(&mut self, slot: usize, value: u16) {
        self.put(0x1dc7 + 2 * slot, value);
    }
    fn set_grey_selector(&mut self, slot: usize, value: u16) {
        self.put(0x1e17 + 2 * slot, value);
    }
    fn set_signal(&mut self, slot: usize, value: u16) {
        self.put(0x1d77 + 2 * slot, value);
    }
    fn projectile_selection(&self) -> u16 {
        self.word(0xdde)
    }
    fn projectile_word(&self, index: isize) -> u16 {
        let at = 0xc18usize
            .checked_add_signed(2 * index)
            .expect("projectile word offset");
        // The former explicit pointer read used native words, unlike RAM word fields.
        u16::from_ne_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
    fn level_word(&self, block: u16) -> u16 {
        self.word(0x10002 + 2 * usize::from(block >> 1))
    }
    fn set_level_word(&mut self, block: u16, value: u16) {
        self.put(0x10002 + 2 * usize::from(block >> 1), value);
    }
    fn level_high(&self, block: u16) -> u8 {
        self.0.g_ram[0x10003 + usize::from(block)]
    }
    fn set_level_high(&mut self, block: u16, value: u8) {
        self.0.g_ram[0x10003 + usize::from(block)] = value;
    }
    fn set_behavior(&mut self, block: u16, value: u8) {
        self.0.g_ram[0x16402 + usize::from(block >> 1)] = value;
    }
}

pub fn setup(j: u16, kind: Setup, game: &mut Game) -> u8 {
    mechanism_doors::setup(&mut DoorFields(game), usize::from(j >> 1), kind)
}
