//! Checked countdown fields and the existing global sound gate.
#![forbid(unsafe_code)]
use crate::game::mechanism_sound_delay::{self, Context};
use crate::game_state::Game;
pub struct SoundDelayFields<'a>(pub &'a mut Game);
impl SoundDelayFields<'_> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
    fn put(&mut self, at: usize, value: u16) {
        self.0.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}
impl Context for SoundDelayFields<'_> {
    fn in_ceres(&self) -> bool {
        self.word(0x79f) == 6
    }
    fn timer(&self, slot: usize) -> u16 {
        self.word(0x1d77 + 2 * slot)
    }
    fn set_timer(&mut self, slot: usize, timer: u16) {
        self.put(0x1d77 + 2 * slot, timer);
    }
    fn watch(&mut self, slot: usize) {
        self.put(0x1cd7 + 2 * slot, 0xb7dd);
    }
    fn enable_sounds(&mut self) {
        self.put(0x5f5, 0);
    }
    fn release(&mut self, slot: usize) {
        self.put(0x1c37 + 2 * slot, 0);
    }
}
pub fn begin(j: u16, game: &mut Game) -> u8 {
    mechanism_sound_delay::begin(&mut SoundDelayFields(game), usize::from(j >> 1))
}
pub fn poll(j: u16, game: &mut Game) {
    mechanism_sound_delay::poll(&mut SoundDelayFields(game), usize::from(j >> 1));
}
