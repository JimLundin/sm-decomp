//! Spritemaps: the lists of sprites a picture is drawn from, in the format every bank's
//! spritemap routines read.
//!
//! Bank `$81` log, add spritemap to OAM `$81:8A5F` (<https://patrickjohnston.org/bank/81>):
//! a spritemap is a word count, then five bytes an entry,
//! `s000000x xxxxxxxx  yyyyyyyy  YXppPPPt tttttttt`: the size bit and a nine-bit X offset,
//! the Y offset, then the sprite's tile and attributes as OAM holds them (the Y and X flips,
//! the priority, the palette and a nine-bit tile).

use crate::rom::{AddressError, Lorom, RomAddress};

/// Bytes of one spritemap entry.
const ENTRY_BYTES: usize = 5;

/// A sprite's size: which of the two sizes the object select register names.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SpriteSize {
    /// The small size.
    #[default]
    Small,
    /// The large size.
    Large,
}

/// A sprite's tile and attributes, the second word of an OAM entry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SpriteAttributes {
    /// The tile, nine bits: the name table's bit and the tile number.
    pub tile: u16,
    /// The palette, 0 to 7.
    pub palette: u8,
    /// The priority relative to the backgrounds, 0 to 3.
    pub priority: u8,
    /// Mirrored left to right.
    pub flip_x: bool,
    /// Mirrored top to bottom.
    pub flip_y: bool,
}

impl SpriteAttributes {
    /// The attributes of an OAM word, `YXppPPPt tttttttt`.
    #[must_use]
    pub const fn from_word(word: u16) -> Self {
        let [_, high] = word.to_le_bytes();
        Self {
            tile: word & 0x01FF,
            palette: (high >> 1) & 7,
            priority: (high >> 4) & 3,
            flip_x: high & 0x40 != 0,
            flip_y: high & 0x80 != 0,
        }
    }

    /// The OAM word, as the original stores it.
    #[must_use]
    pub fn word(self) -> u16 {
        let high = ((self.palette & 7) << 1)
            | ((self.priority & 3) << 4)
            | (u8::from(self.flip_x) << 6)
            | (u8::from(self.flip_y) << 7);
        (u16::from(high) << 8) | (self.tile & 0x01FF)
    }
}

/// One sprite of a spritemap, placed relative to the spritemap's origin.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SpritemapEntry {
    /// The X offset, nine bits sign-extended. The original adds the whole first word to the
    /// origin and keeps the sum's low nine bits, so bits 9 to 14 (set in some of Samus's
    /// entries) never reach OAM and aren't kept.
    pub x: i16,
    /// The Y offset. Only the sum's low byte reaches OAM, so its sign doesn't matter.
    pub y: u8,
    /// The sprite's size.
    pub size: SpriteSize,
    /// The tile and attributes, copied to OAM as they are.
    pub attributes: SpriteAttributes,
}

impl SpritemapEntry {
    /// The entry from its five bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; ENTRY_BYTES]) -> Self {
        let [x_low, x_high, y, tile, attributes] = bytes;
        let x_word = u16::from_le_bytes([x_low, x_high]);
        // The nine-bit offset, sign-extended from bit 8.
        let x = i16::from_le_bytes((x_word << 7).to_le_bytes()) >> 7;
        Self {
            x,
            y,
            size: if x_word & 0x8000 == 0 {
                SpriteSize::Small
            } else {
                SpriteSize::Large
            },
            attributes: SpriteAttributes::from_word(u16::from_le_bytes([tile, attributes])),
        }
    }
}

/// A spritemap: its sprites in the order they are written to OAM.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Spritemap {
    /// The sprites, first written first.
    pub entries: Vec<SpritemapEntry>,
}

impl Spritemap {
    /// The spritemap at `address`.
    ///
    /// # Errors
    ///
    /// [`AddressError`] if its count or entries aren't in the image.
    pub fn load(rom: Lorom<'_>, address: RomAddress) -> Result<Self, AddressError> {
        let count = rom.read(address, 2)?;
        let count = usize::from(u16::from_le_bytes([
            count.first().copied().unwrap_or_default(),
            count.get(1).copied().unwrap_or_default(),
        ]));
        let start = address.advance(2).ok_or(AddressError::PastEnd(address))?;
        let bytes = rom.read(start, count.saturating_mul(ENTRY_BYTES))?;
        Ok(Self::decode(bytes))
    }

    /// The entries of `bytes`, five bytes each; a partial entry at the end is dropped.
    #[must_use]
    pub fn decode(bytes: &[u8]) -> Self {
        let entries = bytes
            .as_chunks::<ENTRY_BYTES>()
            .0
            .iter()
            .map(|&entry| SpritemapEntry::from_bytes(entry))
            .collect();
        Self { entries }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_entry_decodes_its_offsets_size_and_attributes() {
        // $92:A072's first entry: X $1F9 with size set and stray bits 9 and 14, Y -8,
        // tile 0, palette 4, priority 2.
        let entry = SpritemapEntry::from_bytes([0xF9, 0xC3, 0xF8, 0x00, 0x28]);
        assert_eq!(
            entry,
            SpritemapEntry {
                x: -7,
                y: 0xF8,
                size: SpriteSize::Large,
                attributes: SpriteAttributes {
                    tile: 0,
                    palette: 4,
                    priority: 2,
                    flip_x: false,
                    flip_y: false,
                },
            }
        );
        let small = SpritemapEntry::from_bytes([0x0A, 0x00, 0xFD, 0x04, 0x28]);
        assert_eq!((small.x, small.size), (10, SpriteSize::Small));
    }

    #[test]
    fn every_attribute_word_round_trips() {
        for word in 0..=u16::MAX {
            assert_eq!(SpriteAttributes::from_word(word).word(), word);
        }
        let flipped = SpriteAttributes::from_word(0xC1FF);
        assert!(flipped.flip_x && flipped.flip_y);
        assert_eq!(flipped.tile, 0x1FF);
    }

    #[test]
    fn a_spritemap_drops_a_partial_entry() {
        let map = Spritemap::decode(&[0x02, 0x00, 0x00, 0x21, 0x38, 0xFF]);
        assert_eq!(map.entries.len(), 1);
        assert_eq!(
            map.entries.first().map(|entry| entry.attributes.word()),
            Some(0x3821)
        );
    }
}
