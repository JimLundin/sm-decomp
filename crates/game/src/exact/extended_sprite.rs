//! Sprite encoding for actors assembled from offset submaps.

use data::spritemap::{ExtendedMap, SpriteAttributes};
use types::Pair;

use crate::intent::{Placing, ScreenPoint, SpriteDraw, SpritePiece};
use crate::state::video::NmiBatch;

use super::oam::HIDDEN_Y;

/// Append actor maps to the current cursor. Cull each complete submap by its
/// origin, then park outlying Y coordinates without dropping their slots.
pub fn append(
    batch: &mut NmiBatch,
    maps: &[ExtendedMap],
    origin: ScreenPoint,
    palette: u8,
    base_tile: u16,
) {
    let mut draw = SpriteDraw::default();
    for map in maps {
        let x = origin.x.wrapping_add(i32::from(map.offset.x));
        let y = origin.y.wrapping_add(i32::from(map.offset.y));
        if !(-128..384).contains(&x) || !(-128..384).contains(&y) {
            continue;
        }
        for piece in &map.pieces {
            let [lo, hi, ..] = x.to_le_bytes();
            let (_, carry) = u16::from_le_bytes([lo, hi]).overflowing_add(piece.x_word);
            let at_y = y
                .wrapping_add(i32::from(piece.sprite.y.cast_signed()))
                .wrapping_add(i32::from(carry));
            let y = if (0..256).contains(&at_y) {
                at_y
            } else {
                i32::from(HIDDEN_Y)
            };
            let word =
                piece.sprite.attributes.word().wrapping_add(base_tile) | (u16::from(palette) << 9);
            draw.pieces.push(SpritePiece {
                at: Pair {
                    x: x.wrapping_add(i32::from(piece.sprite.x)),
                    y,
                },
                look: SpriteAttributes::from_word(word),
                placing: Placing::Mapped(piece.sprite.size),
            });
        }
    }
    batch.draw_sprites(&draw);
}
