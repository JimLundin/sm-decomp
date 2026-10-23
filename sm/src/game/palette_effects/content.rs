//! Captured safe cartridge records, independent of the emulated backend.
#![forbid(unsafe_code)]
use super::execution::Record;
use crate::rom::{RomImage, RomRecord};
impl Record for RomRecord {
    fn byte(&self, displacement: usize) -> u8 {
        RomRecord::byte(self, displacement)
    }
    fn word(&self, displacement: usize) -> u16 {
        RomRecord::word(self, displacement)
    }
}
pub(crate) fn record(image: RomImage, address: u16) -> RomRecord {
    // Callers handle low-window diagnostics before reaching this bank-local mapping.
    assert!(address & 0x8000 != 0);
    RomRecord {
        image,
        base: 0x68000 + usize::from(address & 0x7fff),
    }
}
