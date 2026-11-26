//! The attached door's closed sprite map is appended after Ridley and the baby.

use crate::intent::SpriteDraw;
use crate::state::video::NmiBatch;
use data::assets::actors::SpriteRow;
use data::spritemap::Spritemap;
use types::{Pair, Px};

/// Append the attached door using its screen-aligned room position and current horizontal shake.
///
/// The mapped-sprite writer keeps entries above the top if they can still wrap into view.
/// It parks farther pieces offscreen and stops at the end of the OAM table.
pub fn append(batch: &mut NmiBatch, art: &Spritemap, position: Pair<Px>, shake_x: i16) {
    let [y, _] = position.y.0.to_le_bytes();
    let origin = Pair {
        x: i32::from(position.x.0).wrapping_add(i32::from(shake_x)),
        y: i32::from(y),
    };
    let mut draw = SpriteDraw::default();
    draw.place(art, origin);
    for piece in &mut draw.pieces {
        piece.look.palette = SpriteRow::RIDLEY_DOOR.get();
        if !(-32..224).contains(&piece.at.y) {
            piece.at = Pair { x: 384, y: 224 };
        }
    }
    batch.draw_capped_sprites(&draw);
}

#[cfg(test)]
mod tests {
    use super::append;
    use crate::state::video::{NmiBatch, SpriteCount};
    use core::num::Wrapping;
    use data::spritemap::{Spritemap, SpritemapEntry};
    use types::Pair;

    #[test]
    fn clipped_pieces_keep_their_slot_and_stop_at_the_end() {
        let art = Spritemap {
            entries: vec![
                SpritemapEntry {
                    y: 224,
                    ..SpritemapEntry::default()
                },
                SpritemapEntry::default(),
                SpritemapEntry::default(),
            ],
        };
        let mut batch = NmiBatch {
            sprites: SpriteCount(126),
            ..NmiBatch::default()
        };
        // A piece 32 pixels above the picture can still wrap into view.
        append(
            &mut batch,
            &art,
            Pair {
                x: Wrapping(12),
                y: Wrapping(0),
            },
            0,
        );
        let first = batch.oam.low[126];
        assert_eq!((first.x, first.y), (12, 224));
        // Changing the first piece to -33 parks it. The last piece still cannot wrap.
        let mut art = art;
        art.entries[0].y = 223;
        batch.sprites = SpriteCount(126);
        let untouched = batch.oam.low[0];
        append(
            &mut batch,
            &art,
            Pair {
                x: Wrapping(12),
                y: Wrapping(0),
            },
            0,
        );
        let parked = batch.oam.low[126];
        assert_eq!((parked.x, parked.y), (128, 224));
        assert_eq!(batch.oam.high[31], 0x10);
        assert_eq!(batch.oam.low[0], untouched);
        assert_eq!(batch.sprites, SpriteCount(128));
    }
}
