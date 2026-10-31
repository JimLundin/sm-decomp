//! The upper half of a bank of interpreted programs, imported once: the
//! palette effect ($8D) and animated tile ($87) programs.

pub use crate::table::LengthError;

pub const BYTES: usize = 0x8000;

/// The LoROM address of `bank`'s upper half.
pub const fn address(bank: u8) -> u32 {
    ((bank as u32) << 16) | 0x8000
}

/// A bank's $8000-$FFFF, as an interpreter reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramBank(Box<[u8; BYTES]>);

impl ProgramBank {
    pub fn import(bytes: &[u8]) -> Result<Self, LengthError> {
        LengthError::check(bytes, BYTES)?;
        let mut data = Box::new([0; BYTES]);
        data.copy_from_slice(bytes);
        Ok(Self(data))
    }

    fn offset(address: u16, bytes: usize) -> Option<usize> {
        let offset = usize::from(address.checked_sub(0x8000)?);
        (offset + bytes <= BYTES).then_some(offset)
    }

    /// The word at `address`, or `None` outside $8000-$FFFF.
    pub fn word(&self, address: u16) -> Option<u16> {
        let at = Self::offset(address, 2)?;
        Some(u16::from_le_bytes([self.0[at], self.0[at + 1]]))
    }

    /// The byte at `address`, or `None` outside $8000-$FFFF.
    pub fn byte(&self, address: u16) -> Option<u8> {
        Self::offset(address, 1).map(|at| self.0[at])
    }
}
