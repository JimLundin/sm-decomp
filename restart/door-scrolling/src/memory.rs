//! Canonical little-endian WRAM fields from the pinned recovered C variables.
use crate::Result;

pub const WRAM_SIZE: usize = 0x20000;
pub(crate) const BG1_H: usize = 0x00b1;
pub(crate) const BG1_V: usize = 0x00b3;
pub(crate) const BG2_H: usize = 0x00b5;
pub(crate) const BG2_V: usize = 0x00b7;
pub(crate) const DIRECTION: usize = 0x0791;
pub(crate) const L1_X_BLOCK: usize = 0x08f7;
pub(crate) const L1_Y_BLOCK: usize = 0x08f9;
pub(crate) const L2_X_BLOCK: usize = 0x08fb;
pub(crate) const L2_Y_BLOCK: usize = 0x08fd;
pub(crate) const PREV_L1_X_BLOCK: usize = 0x08ff;
pub(crate) const PREV_L1_Y_BLOCK: usize = 0x0901;
pub(crate) const PREV_L2_X_BLOCK: usize = 0x0903;
pub(crate) const PREV_L2_Y_BLOCK: usize = 0x0905;
pub(crate) const L1_X: usize = 0x0911;
pub(crate) const L1_Y: usize = 0x0915;
pub(crate) const L2_X: usize = 0x0917;
pub(crate) const L2_Y: usize = 0x0919;
pub(crate) const BG1_X_OFFSET: usize = 0x091d;
pub(crate) const BG1_Y_OFFSET: usize = 0x091f;
pub(crate) const BG2_X_OFFSET: usize = 0x0921;
pub(crate) const BG2_Y_OFFSET: usize = 0x0923;
pub(crate) const COUNTER: usize = 0x0925;
pub(crate) const DEST_X: usize = 0x0927;
pub(crate) const DEST_Y: usize = 0x0929;
pub(crate) const SUBSPEED: usize = 0x092b;
pub(crate) const SPEED: usize = 0x092d;
pub(crate) const COMPLETION: usize = 0x0931;
pub(crate) const SAMUS_X: usize = 0x0af6;
pub(crate) const SAMUS_SUB_X: usize = 0x0af8;
pub(crate) const SAMUS_Y: usize = 0x0afa;
pub(crate) const SAMUS_SUB_Y: usize = 0x0afc;
pub(crate) const SAMUS_PREV_X: usize = 0x0b10;
pub(crate) const SAMUS_PREV_Y: usize = 0x0b14;

pub(crate) struct Wram<'a>(pub &'a mut [u8]);

impl<'a> Wram<'a> {
    pub fn new(bytes: &'a mut [u8]) -> Result<Self> {
        if bytes.len() != WRAM_SIZE {
            return Err("door scrolling requires complete canonical WRAM".into());
        }
        Ok(Self(bytes))
    }

    pub fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0[at], self.0[at + 1]])
    }

    pub fn set(&mut self, at: usize, value: u16) {
        self.0[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }

    pub fn add(&mut self, at: usize, value: u16) {
        self.set(at, self.word(at).wrapping_add(value));
    }

    pub fn sub(&mut self, at: usize, value: u16) {
        self.set(at, self.word(at).wrapping_sub(value));
    }
}
