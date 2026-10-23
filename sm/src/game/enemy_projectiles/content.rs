//! Captured ownership of the native source, distinct from the loaded image.
#![forbid(unsafe_code)]
use crate::rom::RomImage;

pub(super) struct NativeRecord {
    image: RomImage,
    base: usize,
}
impl NativeRecord {
    pub(super) fn new(image: RomImage, base: usize) -> Self {
        Self { image, base }
    }
    pub(super) fn byte(&self, index: isize) -> u8 {
        let offset = self
            .base
            .checked_add_signed(index)
            .expect("native ROM offset overflow");
        let value = self.image.read(offset);
        crate::verify_probe::hit("eproj.native_content.read");
        value
    }
    pub(super) fn word(&self, index: isize) -> u16 {
        let offset = index
            .checked_mul(2)
            .expect("native ROM word offset overflow");
        let high = offset
            .checked_add(1)
            .expect("native ROM word extent overflow");
        u16::from_le_bytes([self.byte(offset), self.byte(high)])
    }
    pub(super) fn long(&self, index: isize) -> u32 {
        u32::from_le_bytes(core::array::from_fn(|lane| {
            let offset = index
                .checked_add(lane as isize)
                .expect("native ROM long extent overflow");
            self.byte(offset)
        }))
    }
}
