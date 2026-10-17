//! Canonical bytes only; no native pointers or state outside the supplied WRAM.
use crate::Result;

pub const WRAM_SIZE: usize = 0x20000;
pub(crate) const ENEMY: usize = 0xf78;
pub(crate) const PROPERTIES: usize = 0x0e;
pub(crate) const PALETTE: usize = 0x1e;
pub(crate) const VAR_A: usize = 0x30;
pub(crate) const VAR_B: usize = 0x32;
pub(crate) const VAR_D: usize = 0x36;
pub(crate) const VAR_E: usize = 0x38;
pub(crate) const VAR_F: usize = 0x3a;
pub(crate) const PARAMETER: usize = 0x3c;

pub(crate) struct Wram<'a>(pub &'a mut [u8]);
impl<'a> Wram<'a> {
    pub fn new(bytes: &'a mut [u8]) -> Result<Self> {
        if bytes.len() != WRAM_SIZE {
            return Err("Ceres door requires complete canonical WRAM".into());
        }
        Ok(Self(bytes))
    }
    pub fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0[at], self.0[at + 1]])
    }
    pub fn set(&mut self, at: usize, value: u16) {
        self.0[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    pub fn mask(&mut self, at: usize, clear: u16, set: u16) {
        self.set(at, (self.word(at) & !clear) | set);
    }
}

pub(crate) struct Rom<'a>(pub &'a [u8]);
impl<'a> Rom<'a> {
    pub fn bytes(&self, address: u16, length: usize) -> Result<&'a [u8]> {
        if address < 0x8000 || usize::from(address) + length > 0x10000 {
            return Err(format!("Ceres door ROM access outside bank A6: {address:04x}+{length}"));
        }
        let at = 0x130000 + usize::from(address & 0x7fff);
        self.0.get(at..at + length).ok_or_else(|| "incomplete Ceres door ROM view".into())
    }
    pub fn word(&self, address: u16) -> Result<u16> {
        let bytes = self.bytes(address, 2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }
}
