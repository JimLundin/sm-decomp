//! Owned native and loaded records; source selection remains an adapter decision.
#![forbid(unsafe_code)]
use super::host;
use sm_data::cartridge::RomImage;
pub struct LoadedRecord {
    image: RomImage,
    base: usize,
}
impl LoadedRecord {
    pub fn new(image: RomImage, base: usize) -> Self {
        Self { image, base }
    }
}
impl host::LoadedRecord for LoadedRecord {
    fn byte(&self, offset: isize) -> u8 {
        self.image.read(
            self.base
                .checked_add_signed(offset)
                .expect("loaded record offset overflow"),
        )
    }
    fn native_word(&self, offset: isize) -> u16 {
        u16::from_ne_bytes([self.byte(offset), self.byte(offset + 1)])
    }
    fn definition_word(&self, field: host::DefinitionField) -> u16 {
        use host::DefinitionField::*;
        self.native_word(match field {
            Initialize => 0,
            PreAction => 2,
            Program => 4,
            Radius => 6,
            Properties => 8,
            HitProgram => 10,
            ShotProgram => 12,
            EnemyDropTable => 0x3a,
        })
    }
}
pub struct NativeRecord {
    image: RomImage,
    base: usize,
}
impl NativeRecord {
    pub fn new(image: RomImage, base: usize) -> Self {
        Self { image, base }
    }
}
impl host::NativeRecord for NativeRecord {
    fn byte(&self, index: isize) -> u8 {
        let offset = self
            .base
            .checked_add_signed(index)
            .expect("native ROM offset overflow");
        let value = self.image.read(offset);
        crate::probe::hit("eproj.native_content.read");
        value
    }
    fn word_le(&self, index: isize) -> u16 {
        let offset = index
            .checked_mul(2)
            .expect("native ROM word offset overflow");
        let high = offset
            .checked_add(1)
            .expect("native ROM word extent overflow");
        u16::from_le_bytes([self.byte(offset), self.byte(high)])
    }
    fn long_le(&self, index: isize) -> u32 {
        u32::from_le_bytes(core::array::from_fn(|lane| {
            self.byte(
                index
                    .checked_add(lane as isize)
                    .expect("native ROM long extent overflow"),
            )
        }))
    }
}
