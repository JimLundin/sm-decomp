//! Checked rescue compatibility fields; installers and queue preserve live effects.
#![forbid(unsafe_code)]
use crate::game::mechanism_animal_rescue::{self, Context, Setup, Terrain};
use crate::game::progress_flags::{CheckEventHappened, SetEventHappened};
use crate::game_state::Game;
pub struct RescueFields<'a>(pub &'a mut Game);
impl RescueFields<'_> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
    fn put(&mut self, at: usize, value: u16) {
        self.0.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}
impl Context for RescueFields<'_> {
    type Block = u16;
    fn block(&self, s: usize) -> u16 {
        self.word(0x1c87 + 2 * s)
    }
    fn below(&self, block: u16) -> u16 {
        block.wrapping_add(self.word(0x7a5).wrapping_mul(2))
    }
    fn four_right(&self, block: u16) -> u16 {
        block.wrapping_add(8)
    }
    fn install(&mut self, block: u16, terrain: Terrain) {
        crate::game::sm_84::WriteLevelDataBlockTypeAndBts(
            block,
            match terrain {
                Terrain::RescueShot => 0xc04f,
                Terrain::UpExtension => 0xd0ff,
            },
            self.0,
        )
    }
    fn set_block(&mut self, s: usize, b: u16) {
        self.put(0x1c87 + 2 * s, b)
    }
    fn rescued(&mut self) -> bool {
        CheckEventHappened(0xf, self.0) != 0
    }
    fn mark_rescued(&mut self) {
        SetEventHappened(0xf, self.0)
    }
    fn release(&mut self, s: usize) {
        self.put(0x1c37 + 2 * s, 0)
    }
    fn timer(&self, s: usize) -> u16 {
        self.word(0x1d77 + 2 * s)
    }
    fn set_timer(&mut self, s: usize, v: u16) {
        self.put(0x1d77 + 2 * s, v)
    }
    fn link(&self, s: usize) -> u16 {
        self.word(0xdebc + 2 * s)
    }
    fn set_program(&mut self, s: usize, v: u16) {
        self.put(0x1d27 + 2 * s, v)
    }
    fn set_instruction_timer(&mut self, s: usize, v: u16) {
        self.put(0xde1c + 2 * s, v)
    }
    fn dud(&mut self) {
        crate::game::sm_80::QueueSfx2_Max6(0x57, self.0)
    }
}
pub fn setup(j: u16, kind: Setup, g: &mut Game) -> u8 {
    mechanism_animal_rescue::setup(&mut RescueFields(g), usize::from(j >> 1), kind)
}
pub fn mark_rescued(g: &mut Game) {
    mechanism_animal_rescue::mark_rescued(&mut RescueFields(g))
}
pub fn move_passage(j: u16, g: &mut Game) {
    mechanism_animal_rescue::move_passage(&mut RescueFields(g), usize::from(j >> 1))
}
pub fn poll_rescue_gate(j: u16, g: &mut Game) {
    mechanism_animal_rescue::poll_rescue_gate(&mut RescueFields(g), usize::from(j >> 1))
}
