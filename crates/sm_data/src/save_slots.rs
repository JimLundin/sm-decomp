//! Where each of the three save files starts in SRAM, at $81:812B.

/// LoROM address of slot 0's offset.
pub const ADDRESS: u32 = 0x81_812b;
pub const SLOTS: usize = 3;
pub const BYTES: usize = SLOTS * 2;

pub use crate::table::LengthError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveSlots {
    offsets: [u16; SLOTS],
}

impl SaveSlots {
    pub fn import(bytes: &[u8]) -> Result<Self, LengthError> {
        LengthError::check(bytes, BYTES)?;
        Ok(Self {
            offsets: core::array::from_fn(|slot| {
                u16::from_le_bytes([bytes[2 * slot], bytes[2 * slot + 1]])
            }),
        })
    }

    /// Slot `slot`'s SRAM byte offset, or `None` past the third slot.
    pub fn offset(&self, slot: usize) -> Option<u16> {
        self.offsets.get(slot).copied()
    }
}
