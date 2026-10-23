//! Checked plant fields during the RAM-authoritative player interaction.
#![forbid(unsafe_code)]
use crate::game::mechanism_plants::{self, Context, Surface};
use crate::game_state::Game;
pub struct PlantFields<'a>(pub &'a mut Game);
impl PlantFields<'_> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
    fn put(&mut self, at: usize, value: u16) {
        self.0.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}
impl Context for PlantFields<'_> {
    type Block = u16;
    fn radius(&self) -> u16 {
        self.word(0xb00)
    }
    fn player_x(&self) -> u16 {
        self.word(0xaf6)
    }
    fn player_y(&self) -> u16 {
        self.word(0xafa)
    }
    fn block(&self, slot: usize) -> u16 {
        self.word(0x1c87 + 2 * slot)
    }
    fn deactivate(&mut self, block: u16) {
        let at = 0x10002 + usize::from(block & !1);
        self.put(at, self.word(at) & 0x8fff);
    }
    fn save_x(&mut self, slot: usize, x: u16) {
        self.put(0x1e17 + 2 * slot, x);
    }
    fn save_y(&mut self, slot: usize, y: u16) {
        self.put(0xdf0c + 2 * slot, y);
    }
    fn saved_x(&self, slot: usize) -> u16 {
        self.word(0x1e17 + 2 * slot)
    }
    fn saved_y(&self, slot: usize) -> u16 {
        self.word(0xdf0c + 2 * slot)
    }
    fn set_player_x(&mut self, x: u16) {
        self.put(0xaf6, x);
    }
    fn set_player_y(&mut self, y: u16) {
        self.put(0xafa, y);
    }
    fn protection(&self) -> u16 {
        self.word(0x18a8)
    }
    fn set_protection(&mut self, frames: u16) {
        self.put(0x18a8, frames);
    }
    fn damage(&self) -> u16 {
        self.word(0xa50)
    }
    fn set_damage(&mut self, damage: u16) {
        self.put(0xa50, damage);
    }
    fn release(&mut self, slot: usize) {
        self.put(0x1c37 + 2 * slot, 0);
    }
}
pub fn begin(j: u16, surface: Surface, game: &mut Game) -> u8 {
    mechanism_plants::begin(&mut PlantFields(game), usize::from(j >> 1), surface)
}
pub fn hold(j: u16, game: &mut Game) {
    mechanism_plants::hold(&mut PlantFields(game), usize::from(j >> 1));
}
pub fn hurt(game: &mut Game) {
    mechanism_plants::hurt(&mut PlantFields(game));
}
pub fn protect(game: &mut Game) {
    mechanism_plants::protect(&mut PlantFields(game));
}
