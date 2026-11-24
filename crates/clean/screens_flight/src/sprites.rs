//! The flight's sprites: the stars rushing past, the sprites drifting sideways, and the places
//! the sprite drawing is told.

use core::num::Wrapping;

use types::flight::{FlightSprite, FlightSprites, Plane, SpriteWrap, WholePlace, whole_pixels};
use types::{Fixed, Parts, fixed};

/// Spec: `StarsRushPast`. The rushing stars' speed rises by this each frame, in 256ths of a px
/// per frame: half a pixel.
const STAR_SPEED_RISE: i16 = 128;
/// A speed in 256ths of a px is this many 65,536ths.
const SPEED_TO_FRACTION: i32 = 256;
/// Spec: `SpritesDrift`. A wrapping sprite's whole pixels are taken modulo this.
const WRAP_WIDTH: u16 = 512;

/// A sprite as the sprite drawing is told it: its kind and its whole-pixel place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpriteView {
    /// Which sprite.
    pub kind: FlightSprite,
    /// Where it is, in whole pixels.
    pub place: WholePlace,
}

/// Spec: `StarsRushPast`. The rushing stars' speed rises by half a pixel per frame; then the
/// stars and the plane each move by it, down and to the right alike. With no rushing stars,
/// nothing rushes.
pub fn rush(sprites: &mut FlightSprites, plane: &mut Plane) {
    let Some(stars) = sprites.rushing_stars.as_mut() else {
        return;
    };
    stars.speed.0 = stars.speed.0.wrapping_add(STAR_SPEED_RISE);
    let step: Fixed = Wrapping(
        i32::from(stars.speed.0)
            .wrapping_mul(SPEED_TO_FRACTION)
            .cast_unsigned(),
    );
    for place in [&mut stars.place, &mut plane.position] {
        place.x += step;
        place.y += step;
    }
}

/// Spec: `SpritesDrift`. Each drifting sprite moves sideways by its drift, whatever the stage;
/// one that wraps comes back in on the other side of 512 px.
pub fn drift(sprites: &mut FlightSprites) {
    for sprite in sprites.drifting.values_mut().flatten() {
        let x = sprite.place.x + Wrapping(sprite.drift.0.cast_unsigned());
        sprite.place.x = match sprite.wrap {
            SpriteWrap::Wraps => fixed(Wrapping(x.pixel().0 % WRAP_WIDTH), x.sub()),
            SpriteWrap::RunsOn => x,
        };
    }
}

/// The sprites placed, each with its kind and its whole-pixel place: the rushing stars first,
/// then the drifting sprites.
#[must_use]
pub fn views(sprites: &FlightSprites) -> Vec<SpriteView> {
    let stars = sprites.rushing_stars.map(|stars| SpriteView {
        kind: FlightSprite::RushingStars,
        place: whole_pixels(stars.place),
    });
    let drifting = sprites.drifting.iter().filter_map(|(kind, sprite)| {
        sprite.map(|sprite| SpriteView {
            kind: FlightSprite::Drifting(kind),
            place: whole_pixels(sprite.place),
        })
    });
    stars.into_iter().chain(drifting).collect()
}
