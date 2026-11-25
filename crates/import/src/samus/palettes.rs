//! Her palettes from the ROM: her suit's by suit, her hurt pulse's two, and her shots' for each
//! combination of the four beams that colour them.
//!
//! Her shots' palettes are a table of pointers indexed by the beams switched on as bits, wave
//! first, then ice, spazer and plasma; no combination with both the spazer and the plasma
//! beam has an entry.

use data::assets::samus::palettes::{Beam, BeamPalette, SamusPalettes, Set, Suits};
use data::rom::{AddressError, Lorom, RomAddress};

use crate::actors::palette;

/// Her suits' palettes.
const POWER: RomAddress = RomAddress::new(0x9b, 0x9400);
const VARIA: RomAddress = RomAddress::new(0x9b, 0x9520);
const GRAVITY: RomAddress = RomAddress::new(0x9b, 0x9800);
/// Her hurt pulse's colours, and the grey standing for her suit's in a cinematic.
const HURT: RomAddress = RomAddress::new(0x91, 0xa380);
const CINEMATIC_GREY: RomAddress = RomAddress::new(0x91, 0xa3a0);
/// Her shots' palette pointers, a word per combination, into their own bank.
const BEAM_POINTERS: RomAddress = RomAddress::new(0x90, 0xc3c9);
/// The beams a combination's bits stand for, lowest first.
const BEAM_BITS: [Beam; 4] = [Beam::Wave, Beam::Ice, Beam::Spazer, Beam::Plasma];
/// The combinations the table has entries for.
const COMBINATIONS: u8 = 12;

/// Her palettes from `rom`.
///
/// # Errors
///
/// [`AddressError`] when a palette or the pointer table is outside the image.
pub fn import(rom: Lorom<'_>) -> Result<SamusPalettes, AddressError> {
    let beams = (0..COMBINATIONS)
        .map(|bits| {
            let entry = BEAM_POINTERS
                .advance(usize::from(bits) * 2)
                .ok_or(AddressError::PastEnd(BEAM_POINTERS))?;
            let pointer: &[u8; 2] = rom
                .read(entry, 2)?
                .try_into()
                .map_err(|_| AddressError::PastEnd(entry))?;
            let at = RomAddress::new(BEAM_POINTERS.bank, u16::from_le_bytes(*pointer));
            Ok(BeamPalette {
                beams: beams(bits),
                palette: palette(rom, at)?,
            })
        })
        .collect::<Result<_, AddressError>>()?;
    Ok(SamusPalettes {
        suits: Suits {
            power: palette(rom, POWER)?,
            varia: palette(rom, VARIA)?,
            gravity: palette(rom, GRAVITY)?,
        },
        hurt: palette(rom, HURT)?,
        cinematic_grey: palette(rom, CINEMATIC_GREY)?,
        beams,
    })
}

/// The beams a combination's bits switch on.
fn beams(bits: u8) -> Set<Beam> {
    let mut on = Set::default();
    for (place, beam) in BEAM_BITS.into_iter().enumerate() {
        on[beam] = bits >> place & 1 != 0;
    }
    on
}

#[cfg(test)]
mod tests {
    use super::{Beam, COMBINATIONS, beams};

    #[test]
    fn a_combinations_bits_are_wave_ice_spazer_and_plasma_from_the_lowest() {
        let on = beams(0b1001);
        assert!(on[Beam::Wave] && on[Beam::Plasma]);
        assert!(!on[Beam::Ice] && !on[Beam::Spazer] && !on[Beam::Charge]);
        for bits in 0..COMBINATIONS {
            assert!((0..bits).all(|earlier| beams(earlier) != beams(bits)));
        }
    }
}

// ROM sources for the data above, kept out of the public docs (the original's bank and RAM
// addresses): bank $91 log, load Samus suit target palette `$91:DEE6`: gravity `$9B:9800`,
// else varia `$9B:9520`, else power `$9B:9400`, to target sprite palette 4 (`$7E:C380`).
// Bank $90 log, update beam tiles and palette `$90:AC8D`, load beam palette `$90:ACCD`: the
// pointers `$90:C3C9` indexed by `([equipped beams] & FFFh) * 2`, 20h bytes to sprite
// palette 6 (`$7E:C1C0`), which `$82:E139` copies to target sprite palette 6.
// Bank $91 log, Samus palette handling `$91:D8A5`: the hurt flash's colours `$91:A380` and, in a
// cinematic, the intro's grey `$91:A3A0`, 20h bytes each to sprite palette 4 (`$7E:C180`).
