//! Samus's palette tables, and the Ceres doors' two colour sets.

use crate::rom::{AddressError, Lorom, RomAddress};

/// The visor's cycle: the last three of the
/// table's six colours, after the x-ray beam's widening three.
const VISOR_CYCLE: RomAddress = RomAddress::new(0x9b, 0xa3c6);

/// Colours in the visor's cycle.
pub const VISOR_SHADES: usize = 3;

/// The cycle's bytes: a word per colour.
const VISOR_BYTES: usize = 6;

/// The visor's colours in rooms whose colour math adds the backdrop, in the order the cycle
/// shows them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VisorColours(pub [u16; VISOR_SHADES]);

impl VisorColours {
    /// The cycle's colours from the ROM.
    ///
    /// # Errors
    ///
    /// [`AddressError`] when the table is outside the image.
    pub fn load(rom: Lorom<'_>) -> Result<Self, AddressError> {
        let bytes: &[u8; VISOR_BYTES] = rom
            .read(VISOR_CYCLE, VISOR_BYTES)?
            .first_chunk()
            .ok_or(AddressError::PastEnd(VISOR_CYCLE))?;
        let mut colours = [0; VISOR_SHADES];
        for (colour, &[low, high]) in colours.iter_mut().zip(bytes.as_chunks::<2>().0) {
            *colour = u16::from_le_bytes([low, high]);
        }
        Ok(Self(colours))
    }
}

/// The Ceres doors' calm colour set, and the escape set right after it.
const CERES_DOORS_CALM: RomAddress = RomAddress::new(0xa6, 0xf4ee);
const CERES_DOORS_ESCAPE: RomAddress = RomAddress::new(0xa6, 0xf50e);

/// Colours in a Ceres door colour set: a sprite palette's colours 1 to 15.
pub const CERES_DOOR_SHADES: usize = 15;

/// A Ceres door colour set's colours, 1 to 15 of the doors' sprite palette.
pub type DoorShades = [u16; CERES_DOOR_SHADES];

/// The two colour sets the Ceres doors' palette takes when a room places them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CeresDoorColours {
    /// The set before Ridley's getaway.
    pub calm: DoorShades,
    /// The set from the getaway on.
    pub escape: DoorShades,
}

impl CeresDoorColours {
    /// Both sets from the ROM.
    ///
    /// # Errors
    ///
    /// [`AddressError`] when a set is outside the image.
    pub fn load(rom: Lorom<'_>) -> Result<Self, AddressError> {
        Ok(Self {
            calm: shades(rom, CERES_DOORS_CALM)?,
            escape: shades(rom, CERES_DOORS_ESCAPE)?,
        })
    }
}

/// A colour set's little-endian words at `at`.
fn shades(rom: Lorom<'_>, at: RomAddress) -> Result<DoorShades, AddressError> {
    let bytes = rom.read(at, 2 * CERES_DOOR_SHADES)?;
    let mut colours = [0; CERES_DOOR_SHADES];
    for (colour, &[low, high]) in colours.iter_mut().zip(bytes.as_chunks::<2>().0) {
        *colour = u16::from_le_bytes([low, high]);
    }
    Ok(colours)
}

// ROM sources for the data above, kept out of the public docs (the original's bank and
// RAM addresses, as the decoders use them):
// Samus's palette tables (bank $9B log, <https://patrickjohnston.org/bank/9B>).
// The visor's cycle (bank $9B log, the visor colours `$9B:A3C0`): the last three of the
// shows them (bank $91 log, handle visor palette `$91:D83F`).
// The Ceres doors' colour sets (bank $A6 log, the door's initialisation `$A6:F6F7`, loaded
// into target sprite palette 7 by `$A9:D2F6`): `$A6:F4EE` calm, `$A6:F50E` from the getaway on.
