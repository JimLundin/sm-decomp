//! Read the object strips for Ridley's ordinary ribs animation.

use data::assets::ridley_ribs::{Art, STRIP_WORDS, Strip, Strips};
use data::dma::BusAddress;
use data::rom::{AddressError, Lorom, RomAddress};

const TABLE: RomAddress = RomAddress::new(0xa6, 0xda71);
const TILE_BANK: u8 = 0xb0;
const ENTRY_BYTES: usize = 6;
const TABLE_BYTES: usize = 4 * ENTRY_BYTES + 2;
const STRIP_BYTES: usize = 2 * STRIP_WORDS;

/// An invalid ribs animation asset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The table or a strip leaves the ROM.
    Address(AddressError),
    /// The reference holds or repeated look do not match the cycle.
    Cycle,
}

impl From<AddressError> for Fault {
    fn from(value: AddressError) -> Self {
        Self::Address(value)
    }
}

/// Read all three ribs looks and validate the reference cycle.
///
/// # Errors
/// A missing or malformed cycle, or unreadable strip, returns its fault.
pub fn import(rom: Lorom<'_>) -> Result<Art, Fault> {
    let (entries, terminator) = rom.read(TABLE, TABLE_BYTES)?.as_chunks::<ENTRY_BYTES>();
    let [closed, opening, wide, closing] = entries else {
        return Err(Fault::Cycle);
    };
    if terminator != TABLE.offset.to_le_bytes() || opening != closing {
        return Err(Fault::Cycle);
    }
    Ok(Art {
        closed: strips(rom, *closed)?,
        partly_open: strips(rom, *opening)?,
        wide: strips(rom, *wide)?,
    })
}

fn strips(
    rom: Lorom<'_>,
    [wait_lo, wait_hi, top_lo, top_hi, bottom_lo, bottom_hi]: [u8; ENTRY_BYTES],
) -> Result<Strips, Fault> {
    if u16::from_le_bytes([wait_lo, wait_hi]) != 20 {
        return Err(Fault::Cycle);
    }
    Ok([
        strip(rom, u16::from_le_bytes([top_lo, top_hi]))?,
        strip(rom, u16::from_le_bytes([bottom_lo, bottom_hi]))?,
    ])
}

fn strip(rom: Lorom<'_>, offset: u16) -> Result<Strip, Fault> {
    let at = RomAddress::new(TILE_BANK, offset);
    let words: &[[u8; 2]; STRIP_WORDS] = rom
        .read(at, STRIP_BYTES)?
        .as_chunks::<2>()
        .0
        .try_into()
        .map_err(|_| AddressError::PastEnd(at))?;
    Ok(Strip {
        source: BusAddress {
            bank: TILE_BANK,
            offset,
        },
        words: words.map(u16::from_le_bytes),
    })
}
