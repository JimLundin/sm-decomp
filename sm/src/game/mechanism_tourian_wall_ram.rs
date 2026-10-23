//! Checked little-endian terrain words and live row strides.
#![forbid(unsafe_code)]
use crate::game::mechanism_tourian_wall::{self, Context, Setup};
use crate::game_state::Game;
pub struct WallFields<'a>(pub &'a mut Game);
impl WallFields<'_> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
}
impl Context for WallFields<'_> {
    type Block = u16;
    fn block(&self, s: usize) -> u16 {
        self.word(0x1c87 + 2 * s)
    }
    fn level(&self, b: u16) -> u16 {
        self.word(0x10002 + usize::from(b & !1))
    }
    fn set_level(&mut self, b: u16, v: u16) {
        let at = 0x10002 + usize::from(b & !1);
        self.0.g_ram[at..at + 2].copy_from_slice(&v.to_le_bytes());
    }
    fn below(&self, b: u16) -> u16 {
        b.wrapping_add(self.word(0x7a5).wrapping_mul(2))
    }
}
pub fn setup(j: u16, kind: Setup, g: &mut Game) -> u8 {
    mechanism_tourian_wall::setup(&mut WallFields(g), usize::from(j >> 1), kind)
}
