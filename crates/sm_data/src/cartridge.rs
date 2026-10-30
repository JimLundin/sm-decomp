//! Shared ownership of the one mutable cartridge image, the source that
//! content importers read from. Bug-fix patches write it in place.
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct RomImage(Rc<Shared>);

#[derive(Default)]
struct Shared {
    cells: Box<[Cell<u8>]>,
    /// Counts writes so imported records can tell they are stale.
    writes: Cell<u64>,
}

impl RomImage {
    pub fn zeroed(length: usize) -> Self {
        Self(Rc::new(Shared {
            cells: (0..length).map(|_| Cell::new(0)).collect(),
            writes: Cell::new(0),
        }))
    }

    pub fn len(&self) -> usize {
        self.0.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.cells.is_empty()
    }

    pub fn read(&self, offset: usize) -> u8 {
        self.0.cells[offset].get()
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

    pub fn write(&self, offset: usize, value: u8) {
        self.0.cells[offset].set(value);
        self.0.writes.set(self.0.writes.get().wrapping_add(1));
    }

    /// The shared bytes; `native_rom::NativeRomPointer` derives the legacy pointer.
    pub fn cells(&self) -> &[Cell<u8>] {
        &self.0.cells
    }

    /// How many bytes have been written since the image was created.
    pub fn writes(&self) -> u64 {
        self.0.writes.get()
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

    pub fn set_word(&self, offset: usize, value: u16) {
        let start = self
            .base
            .checked_add(offset)
            .expect("ROM record offset overflow");
        for (index, byte) in value.to_ne_bytes().into_iter().enumerate() {
            self.image.write(start + index, byte);
        }
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
    /// Allocation provenance only; mutable ROM contents are not copied or compared.
    pub fn same_allocation(&self, other: &Self) -> bool {
        std::rc::Rc::ptr_eq(&self.0, &other.0)
    }
}
