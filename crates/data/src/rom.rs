//! The ROM image: its identity, the `LoROM` address mapping and typed reads.
//!
//! The base content is decoded from exactly one image, the unheadered NTSC release whose
//! SHA-256 is [`Rom::SHA256`]. [`Lorom`] maps SNES addresses to file offsets for any byte
//! image, so the content decoders can also run over hand-made images in tests.

mod sha256;

pub use sha256::sha256;

/// Bytes in a `LoROM` bank's ROM half, `$8000-$FFFF`.
const BANK_BYTES: usize = 0x8000;

/// A SNES address in the cartridge's ROM: a bank and the 16-bit offset inside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RomAddress {
    /// The bank, 0x80-0xFF as the original's pointers name them (or the 0x00-0x7D mirror).
    pub bank: u8,
    /// The offset inside the bank; ROM is at 0x8000-0xFFFF.
    pub offset: u16,
}

impl RomAddress {
    /// The address `bank:offset`.
    #[must_use]
    pub const fn new(bank: u8, offset: u16) -> Self {
        Self { bank, offset }
    }

    /// The address of a 24-bit pointer as the original stores it: offset low, high, bank.
    #[must_use]
    pub const fn from_long(bytes: [u8; 3]) -> Self {
        let [low, high, bank] = bytes;
        Self {
            bank,
            offset: u16::from_le_bytes([low, high]),
        }
    }

    /// The 24-bit value the original stores for this address, `$BBOOOO`.
    #[must_use]
    pub fn long(self) -> u32 {
        (u32::from(self.bank) << 16) | u32::from(self.offset)
    }

    /// The address `count` bytes on in the same bank, or `None` if that leaves the bank.
    #[must_use]
    pub fn advance(self, count: usize) -> Option<Self> {
        let offset = u16::try_from(count)
            .ok()
            .and_then(|count| self.offset.checked_add(count))?;
        Some(Self { offset, ..self })
    }
}

/// An address that names no byte of the image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddressError {
    /// The offset is below 0x8000, outside a `LoROM` bank's ROM half.
    NotRom(RomAddress),
    /// The address maps past the image's end.
    PastEnd(RomAddress),
}

/// Why an image isn't the ROM the base content is decoded from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RomError {
    /// The image has the wrong size, perhaps a copier header or another release.
    Size(usize),
    /// The image has the right size and the wrong SHA-256.
    Hash([u8; 32]),
}

/// The validated ROM image.
#[derive(Clone, PartialEq, Eq)]
pub struct Rom {
    image: Box<[u8]>,
}

impl core::fmt::Debug for Rom {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Rom")
            .field("bytes", &self.image.len())
            .finish()
    }
}

impl Rom {
    /// The image's size: 3 MiB, no copier header.
    pub const SIZE: usize = 0x30_0000;
    /// The image's SHA-256, `12b77c4b...`.
    pub const SHA256: [u8; 32] = [
        0x12, 0xb7, 0x7c, 0x4b, 0xc9, 0xc1, 0x83, 0x2c, 0xee, 0x88, 0x81, 0x24, 0x46, 0x59, 0x06,
        0x5e, 0xe1, 0xd8, 0x4c, 0x70, 0xc3, 0xd2, 0x9e, 0x6e, 0xaf, 0x92, 0xe6, 0x79, 0x8c, 0xc2,
        0xca, 0x72,
    ];

    /// The ROM from its file's bytes.
    ///
    /// # Errors
    ///
    /// [`RomError`] if the image isn't the expected release, by size or by hash.
    pub fn new(image: Vec<u8>) -> Result<Self, RomError> {
        if image.len() != Self::SIZE {
            return Err(RomError::Size(image.len()));
        }
        let hash = sha256(&image);
        if hash != Self::SHA256 {
            return Err(RomError::Hash(hash));
        }
        Ok(Self {
            image: image.into_boxed_slice(),
        })
    }

    /// The image's `LoROM` view.
    #[must_use]
    pub fn lorom(&self) -> Lorom<'_> {
        Lorom::new(&self.image)
    }
}

/// A byte image read through the `LoROM` mapping: bank `b` offset `0x8000 + n` is file offset
/// `(b & 0x7F) * 0x8000 + n`, so the banks follow one another in the file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lorom<'a> {
    image: &'a [u8],
}

impl<'a> Lorom<'a> {
    /// The view of `image`, which need not be the release (for tests).
    #[must_use]
    pub const fn new(image: &'a [u8]) -> Self {
        Self { image }
    }

    /// The file offset of `address`.
    ///
    /// # Errors
    ///
    /// [`AddressError`] if the address isn't in ROM or maps past the image.
    pub fn file_offset(self, address: RomAddress) -> Result<usize, AddressError> {
        if address.offset < 0x8000 {
            return Err(AddressError::NotRom(address));
        }
        let offset = usize::from(address.bank & 0x7f)
            .wrapping_mul(BANK_BYTES)
            .wrapping_add(usize::from(address.offset & 0x7fff));
        if offset < self.image.len() {
            Ok(offset)
        } else {
            Err(AddressError::PastEnd(address))
        }
    }

    /// The image from `address` to its end. Reading on past a bank's end continues into
    /// the next bank's ROM half, as the original's decompressor does.
    ///
    /// # Errors
    ///
    /// [`AddressError`] if the address isn't in the image.
    pub fn from(self, address: RomAddress) -> Result<&'a [u8], AddressError> {
        let offset = self.file_offset(address)?;
        self.image
            .get(offset..)
            .ok_or(AddressError::PastEnd(address))
    }

    /// The `count` bytes at `address`.
    ///
    /// # Errors
    ///
    /// [`AddressError`] if any of them is outside the image.
    pub fn read(self, address: RomAddress, count: usize) -> Result<&'a [u8], AddressError> {
        self.from(address)?
            .get(..count)
            .ok_or(AddressError::PastEnd(address))
    }
}

#[cfg(test)]
mod tests;

// ROM sources for the data above, kept out of the public docs (the original's bank and
// RAM addresses, as the decoders use them):
// The offset inside the bank; ROM is at `$8000-$FFFF`.
// The offset is below `$8000`, outside a `LoROM` bank's ROM half.
// A byte image read through the `LoROM` mapping: bank `b` offset `$8000+n` is file offset
// `(b & $7F) * $8000 + n`, so the banks follow one another in the file.
