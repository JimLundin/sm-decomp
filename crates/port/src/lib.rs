//! Readable gameplay translated from pinned snesrev/sm.
//! Upstream MIT notices: ../decomp/LICENSE.upstream.txt.

pub const WRAM_SIZE: usize = 0x20000;

pub mod audio;
pub mod collision;
pub mod enemy;
pub mod enemy_ai;
pub mod enemy_instructions;
pub mod enemy_jump;
pub mod enemy_rio;
pub mod plm;
pub mod projectile_flight;
pub mod projectile_special;
pub mod projectiles;
pub mod room;
pub mod samus;
pub mod samus_control;
pub mod system;

pub struct Wram<'a>(&'a mut [u8]);

impl<'a> Wram<'a> {
    pub fn new(bytes: &'a mut [u8]) -> Result<Self, &'static str> {
        if bytes.len() != WRAM_SIZE {
            return Err("full canonical WRAM required");
        }
        Ok(Self(bytes))
    }
    pub fn byte(&self, offset: usize) -> u8 {
        self.0[offset]
    }
    pub fn set_byte(&mut self, offset: usize, value: u8) {
        self.0[offset] = value;
    }
    pub fn word(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.0[offset], self.0[offset + 1]])
    }
    pub fn set_word(&mut self, offset: usize, value: u16) {
        self.0[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
}
