//! The ordinary fight's strictly clipped sprite maps and extended body.

use boss_ceres_ridley::{BabyLook, Drawn, RidleyLook};
use data::assets::ridley_picture::{Art, BodyLooks};
use data::spritemap::Spritemap;
use types::{Pair, Parts, Position, Px, Side};

use crate::intent::{ScreenPoint, SpriteDraw};
use crate::state::video::NmiBatch;

use super::extended_sprite;

/// A map origin relative to the ordinary room camera.
#[must_use]
pub fn relative(position: Pair<Px>, camera: Position) -> ScreenPoint {
    let axis =
        |position: Px, camera: Px| i32::from(position.0.wrapping_sub(camera.0).cast_signed());
    Pair {
        x: axis(position.x, camera.x.pixel()),
        y: axis(position.y, camera.y.pixel()),
    }
}

/// Append wings, a tail link or baby art, clipping each piece by its top edge.
pub fn append_visible(batch: &mut NmiBatch, art: &Spritemap, origin: ScreenPoint, palette: u8) {
    let mut draw = SpriteDraw::default();
    draw.place(art, origin);
    draw.pieces.retain_mut(|piece| {
        piece.look.palette |= palette;
        (0..224).contains(&piece.at.y)
    });
    batch.draw_sprites(&draw);
}

/// Append the tail's visible links, starting at its tip.
///
/// Returns `None` when `orientation` has no imported tip map.
#[must_use]
pub fn tail(
    batch: &mut NmiBatch,
    art: &Art,
    orientation: usize,
    tip_first: [Pair<Px>; 7],
    camera: Position,
    palette: u8,
) -> Option<()> {
    let tip = art.tip.get(orientation)?;
    let [base, middle, outer] = &art.links;
    for (position, map) in tip_first
        .into_iter()
        .zip([tip, outer, outer, middle, middle, base, base])
    {
        append_visible(batch, map, relative(position, camera), palette);
    }
    Some(())
}

/// Append the wings at their pre-hop draw anchor.
///
/// Returns `None` when `beat` has no imported wing map.
#[must_use]
pub fn wings(
    batch: &mut NmiBatch,
    art: &Art,
    side: Side,
    beat: usize,
    origin: ScreenPoint,
    palette: u8,
) -> Option<()> {
    let [left, right] = &art.wings;
    let maps = match side {
        Side::Left => left,
        Side::Right => right,
    };
    append_visible(batch, maps.get(beat)?, origin, palette);
    Some(())
}

/// Append the baby's current ordinary sprite map.
pub fn baby(batch: &mut NmiBatch, art: &Art, drawn: Drawn<BabyLook>, camera: Position) {
    let [first, second, third] = &art.baby;
    let art = match drawn.look {
        BabyLook::First => first,
        BabyLook::Second => second,
        BabyLook::Third => third,
    };
    append_visible(batch, art, relative(drawn.position, camera), 0);
}

/// Append the four ordered maps of the current ordinary body.
pub fn body(
    batch: &mut NmiBatch,
    looks: &BodyLooks,
    look: RidleyLook,
    origin: ScreenPoint,
    palette: u8,
) {
    let maps = match look {
        RidleyLook::Rest => &looks.rest,
        RidleyLook::Open => &looks.open,
        RidleyLook::Wide => &looks.wide,
        RidleyLook::TakeOff1 => &looks.takeoff1,
        RidleyLook::TakeOff2 => &looks.takeoff2,
    };
    extended_sprite::append(batch, maps, origin, palette, 0);
}
