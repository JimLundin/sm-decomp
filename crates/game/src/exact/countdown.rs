//! Place the countdown's fixed frame and six imported digits.

use crate::intent::SpriteDraw;
use crate::state::video::OAM_SPRITES;
use data::assets::countdown::Figures;
use interface_hud::TimeLeft;
use types::Pair;

/// Compose the countdown's sprite pieces at its whole-pixel position.
///
/// Returns `None` if the content cannot supply a digit or the drawing
/// exceeds the sprite buffer.
#[must_use]
pub fn sprites(figures: &Figures, place: Pair<u16>, time: TimeLeft) -> Option<SpriteDraw> {
    let origin = Pair {
        x: i32::from(place.x),
        y: i32::from(place.y),
    };
    let mut draw = SpriteDraw::default();
    draw.place(&figures.frame, origin);
    for (pair, offset) in time.shown().into_iter().zip([-28_i32, -4, 20]) {
        for (digit, x) in [pair / 10, pair % 10]
            .into_iter()
            .zip([offset, offset.saturating_add(8)])
        {
            let map = figures.digits.get(usize::try_from(digit).ok()?)?;
            draw.place(
                map,
                Pair {
                    x: origin.x.saturating_add(x),
                    y: origin.y,
                },
            );
        }
    }
    if draw.pieces.len() > OAM_SPRITES {
        return None;
    }
    for piece in &mut draw.pieces {
        piece.look.palette |= 5;
    }
    Some(draw)
}
