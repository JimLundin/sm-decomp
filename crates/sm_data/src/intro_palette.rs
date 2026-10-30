//! The intro palettes at $8C:E3E9: a full 256-colour palette buffer.

/// LoROM address of colour 0.
pub const ADDRESS: u32 = 0x8c_e3e9;
pub const COLORS: usize = 256;
pub const BYTES: usize = 2 * COLORS;

pub use crate::table::LengthError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntroPalette {
    /// SNES BGR555 colours, background palettes then sprite palettes.
    colors: Box<[u16; COLORS]>,
}

impl IntroPalette {
    pub fn import(bytes: &[u8]) -> Result<Self, LengthError> {
        LengthError::check(bytes, BYTES)?;
        Ok(Self {
            colors: Box::new(core::array::from_fn(|color| {
                u16::from_le_bytes([bytes[2 * color], bytes[2 * color + 1]])
            })),
        })
    }

    pub fn color(&self, index: usize) -> Option<u16> {
        self.colors.get(index).copied()
    }
}
