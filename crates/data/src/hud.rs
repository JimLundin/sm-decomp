//! The HUD's pictures, by what each one shows.
//!
//! The status bar and the mini-map draw with these. The colour sets they are drawn in are the
//! HUD's rules, not content, so a look here is a picture and its flips only. The blank and the
//! diagonal are pictures alone: the HUD tests the blank unflipped and the diagonal by its number.

use clean_types::{Flip, Look, Picture};

use crate::rom::{AddressError, Lorom, RomAddress};

// The blank's word, which the highlight compares the icon's top-left cell with: the immediate of
// `CMP #$2C0F` at $80:9CF9.
const BLANK: RomAddress = RomAddress::new(0x80, 0x9CFA);
// The digits' table, digit 0 first ($80:9DBF; $80:9DD3 is the same table again).
const DIGITS: RomAddress = RomAddress::new(0x80, 0x9DBF);
// The tanks' words: the immediates of `LDX #$2831` at $80:9BDC and `LDX #$3430` at $80:9BD3.
const FULL_TANK: RomAddress = RomAddress::new(0x80, 0x9BDD);
const EMPTY_TANK: RomAddress = RomAddress::new(0x80, 0x9BD4);
// The reserve sign, lit ($80:998B); the dim one at $80:9997 shows the same looks.
const RESERVE_SIGN: RomAddress = RomAddress::new(0x80, 0x998B);
// The icons: the missiles' three by two, then two by two for each other item.
const MISSILES_ICON: RomAddress = RomAddress::new(0x80, 0x99A3);
const SUPER_MISSILES_ICON: RomAddress = RomAddress::new(0x80, 0x99AF);
const POWER_BOMBS_ICON: RomAddress = RomAddress::new(0x80, 0x99B7);
const GRAPPLE_BEAM_ICON: RomAddress = RomAddress::new(0x80, 0x99BF);
const X_RAY_SCOPE_ICON: RomAddress = RomAddress::new(0x80, 0x99C7);
// The mini-map's blank: the immediate of `LDA #$001F` at $90:AAAE.
const MAP_BLANK: RomAddress = RomAddress::new(0x90, 0xAAAF);
// The diagonal the mini-map tests her cell for: the immediate of `CMP #$0028` at $90:AB00.
const DIAGONAL: RomAddress = RomAddress::new(0x90, 0xAB01);

/// A tilemap word's picture, and its flips across and down; the bits between are its colours
/// and its priority, which the HUD's rules set.
const PICTURE: u16 = 0x03FF;
const FLIP_ACROSS: u16 = 0x4000;
const FLIP_DOWN: u16 = 0x8000;

/// An item's icon: its rows top first, each left to right.
pub type Icon<const ACROSS: usize> = [[Look; ACROSS]; 2];

/// The five items' icons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemIcons {
    /// The missiles', three across.
    pub missiles: Icon<3>,
    /// The super missiles'.
    pub super_missiles: Icon<2>,
    /// The power bombs'.
    pub power_bombs: Icon<2>,
    /// The grapple beam's.
    pub grapple_beam: Icon<2>,
    /// The x-ray scope's.
    pub x_ray_scope: Icon<2>,
}

/// The HUD's pictures by what each shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HudPictures {
    /// The blank a cell shows where nothing is drawn.
    pub blank: Picture,
    /// The digits, 0 first.
    pub digits: [Look; 10],
    /// A full energy tank.
    pub full_tank: Look,
    /// An empty energy tank.
    pub empty_tank: Look,
    /// The reserve sign (AUTO): three rows top first, two across.
    pub reserve_sign: [[Look; 2]; 3],
    /// The items' icons.
    pub icons: ItemIcons,
    /// A mini-map cell that shows nothing of the map.
    pub map_blank: Look,
    /// The map's diagonal, which also explores the cell above her.
    pub diagonal: Picture,
}

/// The look of a tilemap word: its ten bits of picture, which always name one, and its flips.
fn look(word: u16) -> Option<Look> {
    let flip = match (word & FLIP_ACROSS != 0, word & FLIP_DOWN != 0) {
        (false, false) => Flip::None,
        (true, false) => Flip::Across,
        (false, true) => Flip::Down,
        (true, true) => Flip::Both,
    };
    Some(Look {
        picture: Picture::new(word & PICTURE)?,
        flip,
    })
}

/// The `N` looks of the words at `at`; every word read has its look.
fn looks<const N: usize>(rom: Lorom<'_>, at: RomAddress) -> Result<[Look; N], AddressError> {
    rom.read(at, N.wrapping_mul(2))?
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&word| look(u16::from_le_bytes(word)))
        .collect::<Option<Vec<Look>>>()
        .and_then(|looks| looks.try_into().ok())
        .ok_or(AddressError::PastEnd(at))
}

/// A table of `DOWN` rows of `ACROSS` looks at `at`, the top row first.
fn rows<const ACROSS: usize, const DOWN: usize>(
    rom: Lorom<'_>,
    at: RomAddress,
) -> Result<[[Look; ACROSS]; DOWN], AddressError> {
    let past_end = AddressError::PastEnd(at);
    (0..DOWN)
        .map(|row| {
            let start = at
                .advance(row.wrapping_mul(ACROSS).wrapping_mul(2))
                .ok_or(past_end)?;
            looks(rom, start)
        })
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| past_end)
}

/// The one look of the word at `at`.
fn one(rom: Lorom<'_>, at: RomAddress) -> Result<Look, AddressError> {
    let [single] = looks::<1>(rom, at)?;
    Ok(single)
}

impl HudPictures {
    /// The HUD's pictures from the ROM.
    ///
    /// # Errors
    ///
    /// [`AddressError`] when a table is outside the image.
    pub fn load(rom: Lorom<'_>) -> Result<Self, AddressError> {
        Ok(Self {
            blank: one(rom, BLANK)?.picture,
            digits: looks(rom, DIGITS)?,
            full_tank: one(rom, FULL_TANK)?,
            empty_tank: one(rom, EMPTY_TANK)?,
            reserve_sign: rows(rom, RESERVE_SIGN)?,
            icons: ItemIcons {
                missiles: rows(rom, MISSILES_ICON)?,
                super_missiles: rows(rom, SUPER_MISSILES_ICON)?,
                power_bombs: rows(rom, POWER_BOMBS_ICON)?,
                grapple_beam: rows(rom, GRAPPLE_BEAM_ICON)?,
                x_ray_scope: rows(rom, X_RAY_SCOPE_ICON)?,
            },
            map_blank: one(rom, MAP_BLANK)?,
            diagonal: one(rom, DIAGONAL)?.picture,
        })
    }
}

#[cfg(test)]
mod tests;
