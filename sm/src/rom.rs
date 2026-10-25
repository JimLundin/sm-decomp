//! Shared ownership of the one mutable cartridge image.
#![forbid(unsafe_code)]
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct RomImage(Rc<[Cell<u8>]>);

impl RomImage {
    pub fn zeroed(length: usize) -> Self {
        Self((0..length).map(|_| Cell::new(0)).collect())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn read(&self, offset: usize) -> u8 {
        self.0[offset].get()
    }

    // Table offsets are signed words in the allocation, not wrapping bank addresses.
    // Retain the former unaligned load's native byte order.
    pub(crate) fn table_word(&self, base: usize, index: isize) -> u16 {
        let offset = index
            .checked_mul(2)
            .and_then(|bytes| base.checked_add_signed(bytes))
            .expect("ROM table offset overflow");
        u16::from_ne_bytes([self.read(offset), self.read(offset + 1)])
    }

    pub fn write(&self, offset: usize, value: u8) {
        self.0[offset].set(value);
    }

    // Legacy cartridge and patch consumers borrow this allocation. Cell keeps
    // its bytes mutable while shared owners exist; access is serialized by Game.
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.0.as_ptr().cast::<u8>().cast_mut()
    }
}

/// A captured cartridge allocation with linear, native-endian record reads.
pub(crate) struct RomRecord {
    pub(crate) image: RomImage,
    pub(crate) base: usize,
}

impl RomRecord {
    pub(crate) fn byte(&self, offset: usize) -> u8 {
        self.image.read(
            self.base
                .checked_add(offset)
                .expect("ROM record offset overflow"),
        )
    }

    pub(crate) fn table_word(&self, index: isize) -> u16 {
        self.image.table_word(self.base, index)
    }

    pub(crate) fn word(&self, offset: usize) -> u16 {
        u16::from_ne_bytes([self.byte(offset), self.byte(offset + 1)])
    }

    pub(crate) fn set_word(&self, offset: usize, value: u16) {
        let start = self
            .base
            .checked_add(offset)
            .expect("ROM record offset overflow");
        for (index, byte) in value.to_ne_bytes().into_iter().enumerate() {
            self.image.write(start + index, byte);
        }
    }

    // Load24 includes the adjacent fourth byte in a native read before masking.
    pub(crate) fn load24(&self, offset: usize) -> u32 {
        u32::from_ne_bytes(core::array::from_fn(|i| self.byte(offset + i))) & 0xffffff
    }

    pub(crate) fn long(&self, offset: usize) -> crate::types::LongPtr {
        crate::types::LongPtr {
            addr: self.word(offset),
            bank: self.byte(offset + 2),
        }
    }
}

/// Map a banked LoROM address to an offset in the ROM allocation.
/// Callers keep their window/bounds checks and add linear table offsets afterward.
#[inline]
pub const fn lorom_offset(address: u32) -> isize {
    (((address >> 16) << 15 | address & 0x7fff) & 0x3fffff) as isize
}

impl RomImage {
    /// Allocation provenance only; mutable ROM contents are not copied or compared.
    pub(crate) fn same_allocation(&self, other: &Self) -> bool {
        std::rc::Rc::ptr_eq(&self.0, &other.0)
    }
}
