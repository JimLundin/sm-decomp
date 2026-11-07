//! Samus's palette tables.

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

// ROM sources for the data above, kept out of the public docs (the original's bank and
// RAM addresses, as the decoders use them):
// Samus's palette tables (bank $9B log, <https://patrickjohnston.org/bank/9B>).
// The visor's cycle (bank $9B log, the visor colours `$9B:A3C0`): the last three of the
// shows them (bank $91 log, handle visor palette `$91:D83F`).
