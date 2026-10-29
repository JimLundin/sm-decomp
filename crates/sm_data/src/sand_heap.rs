//! Dead Torizo's sand heap tiles at $B7:A800.
//!
//! Two rows of sixteen 4bpp tiles, 0x200 bytes per row. The corpse copies the
//! heap into its tile buffer one pixel line at a time, choosing each line's
//! word offset from a table in bank $A9.

/// LoROM address of the first tile.
pub const ADDRESS: u32 = 0xb7_a800;
/// Two rows of sixteen 32-byte tiles.
pub const BYTES: usize = 0x400;

/// The importer was given a byte count other than `BYTES`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LengthError(pub usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandHeapTiles {
    bytes: Box<[u8; BYTES]>,
}

impl SandHeapTiles {
    pub fn import(bytes: &[u8]) -> Result<Self, LengthError> {
        let bytes = bytes.try_into().map_err(|_| LengthError(bytes.len()))?;
        Ok(Self {
            bytes: Box::new(bytes),
        })
    }

    /// The little-endian word at a word index from $B7:A800, or `None` past the tiles.
    pub fn word(&self, index: usize) -> Option<u16> {
        let at = index.checked_mul(2)?;
        let pair = self.bytes.get(at..at.checked_add(2)?)?;
        Some(u16::from_le_bytes([pair[0], pair[1]]))
    }
}
