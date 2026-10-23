//! Captured animation records with no emulated backend dependency.
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
    assert!(address & 0x8000 != 0);
    RomRecord {
        image,
        base: 0x38000 + usize::from(address & 0x7fff),
    }
}
