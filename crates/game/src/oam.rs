//! The sprite buffer's frame: drawing starts from an empty buffer, and once it ends every
//! entry nothing drew is moved off screen.

use data::spritemap::{SpriteAttributes, SpriteSize, Spritemap};

use crate::state::video::{NmiBatch, OAM_HIGH_BYTES, OAM_SPRITES, OamEntry, SpriteCount};

/// The Y of a hidden sprite: line 240, below the picture. Only Y moves, so a hidden entry
/// keeps the tile and attributes the buffer last held.
pub const HIDDEN_Y: u8 = 240;

impl NmiBatch {
    /// Bank `$80` log, clear high OAM `$80:8B1A`: every sprite's X ninth bit and size bit.
    pub const fn clear_high_oam(&mut self) {
        self.oam.high = [0; OAM_HIGH_BYTES];
    }

    /// Bank `$82` log, main game loop `$82:8957`: drawing starts again from the first entry.
    pub const fn restart_sprites(&mut self) {
        self.sprites = SpriteCount(0);
    }

    /// Hide every entry past those drawn this frame and restart the count.
    ///
    /// Bank `$80` log, finalise OAM `$80:896E`. A count at or past the table's end hides
    /// nothing.
    pub fn finalise_oam(&mut self) {
        let drawn = usize::from(self.sprites.0);
        if let Some(unused) = self.oam.low.get_mut(drawn..) {
            for entry in unused {
                entry.y = HIDDEN_Y;
            }
        }
        self.restart_sprites();
    }

    /// Write a sprite at the next entry, its X ninth bit and size bit into the high table,
    /// and move on, wrapping from the last entry to the first.
    ///
    /// Bank `$81` log, add Samus spritemap to OAM `$81:89AE`: the screen X is a nine-bit
    /// sum, the high table's bits are set (never cleared: clear high OAM `$80:8B1A` cleared
    /// them for the frame), and the OAM index wraps at `$200` bytes.
    pub fn add_sprite(&mut self, x: u16, y: u8, size: SpriteSize, attributes: SpriteAttributes) {
        let index = usize::from(self.sprites.0) % OAM_SPRITES;
        let [x_low, _] = x.to_le_bytes();
        let [tile, high] = attributes.word().to_le_bytes();
        if let Some(entry) = self.oam.low.get_mut(index) {
            *entry = OamEntry {
                x: x_low,
                y,
                tile,
                attributes: high,
            };
        }
        let bits = u8::from(x & 0x100 != 0) | (u8::from(size == SpriteSize::Large) << 1);
        if let Some(byte) = self.oam.high.get_mut(index / 4) {
            *byte |= bits << ((index % 4).wrapping_mul(2));
        }
        let next = index.wrapping_add(1) % OAM_SPRITES;
        self.sprites = SpriteCount(u16::try_from(next).unwrap_or_default());
    }

    /// Write a sprite at the next entry without touching the high table, and move on.
    ///
    /// Bank `$90` log, `$90:868D`'s chest sprite: the low table's four bytes only, so a
    /// ninth X bit is dropped, and the index moves on without the `$200` wrap.
    // open: past the table's end the original writes into the high table (`$0570`); here
    // the sprite is dropped.
    pub fn add_low_sprite(&mut self, x: u16, y: u8, attributes: SpriteAttributes) {
        let [x_low, _] = x.to_le_bytes();
        let [tile, high] = attributes.word().to_le_bytes();
        if let Some(entry) = self.oam.low.get_mut(usize::from(self.sprites.0)) {
            *entry = OamEntry {
                x: x_low,
                y,
                tile,
                attributes: high,
            };
        }
        self.sprites = SpriteCount(self.sprites.0.saturating_add(1));
    }

    /// Write `spritemap`'s sprites in order, placed from `origin` on screen.
    ///
    /// Bank `$81` log, `$81:89AE`: each entry's X offset adds to the origin's X and its Y
    /// offset to the origin's Y, of which only the low byte reaches OAM; there is no
    /// off-screen test.
    pub fn add_spritemap(&mut self, spritemap: &Spritemap, origin: (u16, u16)) {
        let (x, y) = origin;
        let [y_low, _] = y.to_le_bytes();
        for entry in &spritemap.entries {
            let offset = u16::from_le_bytes(entry.x.to_le_bytes());
            self.add_sprite(
                x.wrapping_add(offset),
                y_low.wrapping_add(entry.y),
                entry.size,
                entry.attributes,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HIDDEN_Y;
    use crate::state::video::{NmiBatch, OAM_SPRITES, OamEntry, SpriteCount};
    use data::spritemap::Spritemap;

    fn batch(sprites: u16) -> NmiBatch {
        let mut batch = NmiBatch::default();
        for (index, entry) in batch.oam.low.iter_mut().enumerate() {
            let tile = u8::try_from(index).unwrap();
            *entry = OamEntry {
                x: 1,
                y: 2,
                tile,
                attributes: 3,
            };
        }
        batch.oam.high = [0xFF; _];
        batch.sprites = SpriteCount(sprites);
        batch
    }

    #[test]
    fn finalising_hides_only_the_undrawn_entries() {
        let mut drawn = batch(3);
        drawn.finalise_oam();
        let ys: Vec<u8> = drawn.oam.low.iter().map(|entry| entry.y).collect();
        assert_eq!(ys[..3], [2, 2, 2]);
        assert!(ys[3..].iter().all(|&y| y == HIDDEN_Y));
        assert_eq!(drawn.oam.low[100].tile, 100);
        assert_eq!(drawn.oam.low[100].x, 1);
        assert_eq!(drawn.oam.high, [0xFF; _]);
        assert_eq!(drawn.sprites, SpriteCount(0));
    }

    #[test]
    fn a_spritemap_writes_its_sprites_from_the_count_with_their_high_bits() {
        let mut drawn = NmiBatch {
            sprites: SpriteCount(3),
            ..NmiBatch::default()
        };
        let map = Spritemap::decode(&[
            0xF9, 0xC3, 0xF8, 0x00, 0x28, // large, (-7, -8), tile 0, palette 4, priority 2
            0x0A, 0x00, 0x05, 0x21, 0x78, // small, (10, 5), X flip
        ]);
        drawn.add_spritemap(&map, (0x0003, 0x0180));
        let [first, second] = [drawn.oam.low[3], drawn.oam.low[4]];
        // 3 - 7 is $1FC: low byte $FC with the ninth bit set; $80 - 8 is $78.
        assert_eq!(
            first,
            OamEntry {
                x: 0xFC,
                y: 0x78,
                tile: 0,
                attributes: 0x28
            }
        );
        assert_eq!(
            second,
            OamEntry {
                x: 13,
                y: 0x85,
                tile: 0x21,
                attributes: 0x78
            }
        );
        // Entry 3: X high and large (0b11) at bits 6-7; entry 4: nothing.
        assert_eq!(drawn.oam.high[0], 0b1100_0000);
        assert_eq!(drawn.oam.high[1], 0);
        assert_eq!(drawn.sprites, SpriteCount(5));
    }

    #[test]
    fn the_sprite_index_wraps_past_the_last_entry() {
        let mut drawn = NmiBatch {
            sprites: SpriteCount(127),
            ..NmiBatch::default()
        };
        let map = Spritemap::decode(&[0; 10]);
        drawn.add_spritemap(&map, (0x100, 0));
        assert_eq!(drawn.sprites, SpriteCount(1));
        assert_eq!(drawn.oam.high[31], 0b0100_0000);
        assert_eq!(drawn.oam.high[0], 0b0000_0001);
    }

    #[test]
    fn a_full_table_hides_nothing() {
        let full = u16::try_from(OAM_SPRITES).unwrap();
        for count in [full, full + 1] {
            let mut drawn = batch(count);
            drawn.finalise_oam();
            assert!(drawn.oam.low.iter().all(|entry| entry.y == 2));
            assert_eq!(drawn.sprites, SpriteCount(0));
        }
    }

    #[test]
    fn an_empty_frame_hides_every_entry_and_clears_the_high_table() {
        let mut empty = batch(9);
        empty.clear_high_oam();
        empty.restart_sprites();
        empty.finalise_oam();
        assert!(empty.oam.low.iter().all(|entry| entry.y == HIDDEN_Y));
        assert_eq!(empty.oam.high, [0; _]);
    }
}
