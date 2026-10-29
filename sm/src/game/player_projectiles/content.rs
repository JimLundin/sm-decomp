//! Captured cartridge records and named projectile table selection.
#![forbid(unsafe_code)]
use super::host::Host;
use crate::rom::{RomImage, RomRecord};
pub fn offset(address: u32) -> usize {
    (((address >> 16) << 15 | address & 0x7fff) & 0x3fffff) as usize
}
pub fn image(h: &impl Host) -> RomImage {
    h.loaded_image().expect("ROM must be loaded")
}
pub fn table(h: &impl Host, address: u32, index: isize) -> u16 {
    image(h).table_word(offset(address), index)
}
pub fn captured(image: RomImage, address: u32) -> RomRecord {
    RomRecord {
        image,
        base: offset(address),
    }
}
