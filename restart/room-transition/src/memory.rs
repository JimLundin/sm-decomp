//! Canonical WRAM and bank-8F command/header reads; no process pointers.
use crate::Result;
pub const WRAM_SIZE: usize = 0x20000;

pub(crate) struct Wram<'a>(pub &'a mut [u8]);
impl<'a> Wram<'a> {
    pub fn new(bytes: &'a mut [u8]) -> Result<Self> {
        if bytes.len() != WRAM_SIZE {
            return Err("room transition requires complete canonical WRAM".into());
        }
        Ok(Self(bytes))
    }
    pub fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0[at], self.0[at + 1]])
    }
    pub fn long(&self, at: usize) -> u32 {
        u32::from(self.word(at)) | u32::from(self.0[at + 2]) << 16
    }
    pub fn set(&mut self, at: usize, value: u16) {
        self.0[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    pub fn add(&mut self, at: usize, value: u16) {
        self.set(at, self.word(at).wrapping_add(value));
    }
}

pub(crate) struct Rom<'a>(pub &'a [u8]);
impl<'a> Rom<'a> {
    pub fn bytes(&self, at: u16, count: usize) -> Result<&'a [u8]> {
        if at < 0x8000 || usize::from(at) + count > 0x10000 {
            return Err(format!("room-transition bank-8F ROM extent at {at:04x}+{count}"));
        }
        let start = 0x78000 + usize::from(at & 0x7fff);
        self.0.get(start..start + count).ok_or_else(|| "incomplete room-transition ROM view".into())
    }
    pub fn word(&self, at: u16) -> Result<u16> {
        let bytes = self.bytes(at, 2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }
}
