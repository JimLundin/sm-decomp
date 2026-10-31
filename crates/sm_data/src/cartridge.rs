//! Shared ownership of the engine's cartridge image, the source that content
//! importers read from. The image is immutable, as the cartridge is to the
//! game; the oracle's patched cartridge is a separate copy.
use std::rc::Rc;

#[derive(Clone)]
pub struct RomImage(Rc<[u8]>);

/// An empty image is still its own allocation.
impl Default for RomImage {
    fn default() -> Self {
        Self::zeroed(0)
    }
}

impl RomImage {
    pub fn zeroed(length: usize) -> Self {
        Self::from(vec![0; length])
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn read(&self, offset: usize) -> u8 {
        self.0[offset]
    }

    // Table offsets are signed words in the allocation, not wrapping bank addresses.
    // Retain the former unaligned load's native byte order.
    pub fn table_word(&self, base: usize, index: isize) -> u16 {
        let offset = index
            .checked_mul(2)
            .and_then(|bytes| base.checked_add_signed(bytes))
            .expect("ROM table offset overflow");
        u16::from_ne_bytes([self.read(offset), self.read(offset + 1)])
    }

    /// The image's bytes; `native_rom::NativeRomPointer` derives the legacy pointer.
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<&[u8]> for RomImage {
    fn from(bytes: &[u8]) -> Self {
        Self(Rc::from(bytes))
    }
}

impl From<Vec<u8>> for RomImage {
    fn from(bytes: Vec<u8>) -> Self {
        Self(Rc::from(bytes))
    }
}

/// A captured cartridge allocation with linear, native-endian record reads.
pub struct RomRecord {
    pub image: RomImage,
    pub base: usize,
}

impl RomRecord {
    pub fn byte(&self, offset: usize) -> u8 {
        self.image.read(
            self.base
                .checked_add(offset)
                .expect("ROM record offset overflow"),
        )
    }

    pub fn table_word(&self, index: isize) -> u16 {
        self.image.table_word(self.base, index)
    }

    pub fn word(&self, offset: usize) -> u16 {
        u16::from_ne_bytes([self.byte(offset), self.byte(offset + 1)])
    }

    // Load24 includes the adjacent fourth byte in a native read before masking.
    pub fn load24(&self, offset: usize) -> u32 {
        u32::from_ne_bytes(core::array::from_fn(|i| self.byte(offset + i))) & 0xffffff
    }
}

/// Map a banked LoROM address to an offset in the ROM allocation.
/// Callers keep their window/bounds checks and add linear table offsets afterward.
#[inline]
pub const fn lorom_offset(address: u32) -> isize {
    (((address >> 16) << 15 | address & 0x7fff) & 0x3fffff) as isize
}

impl RomImage {
    /// Allocation provenance only; contents are not compared.
    pub fn same_allocation(&self, other: &Self) -> bool {
        std::rc::Rc::ptr_eq(&self.0, &other.0)
    }
}
