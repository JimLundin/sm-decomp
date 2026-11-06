//! The sprite buffer's frame: drawing starts from an empty buffer, and once it ends every
//! entry nothing drew is moved off screen.

use crate::state::video::{NmiBatch, OAM_HIGH_BYTES, SpriteCount};

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
}

#[cfg(test)]
mod tests {
    use super::HIDDEN_Y;
    use crate::state::video::{NmiBatch, OAM_SPRITES, OamEntry, SpriteCount};

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
