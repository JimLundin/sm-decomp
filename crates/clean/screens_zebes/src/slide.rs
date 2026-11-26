//! Spec: `PlanetAndStarsSlide`. The planet and star fields accelerate upward and leave; the
//! flight leaves once the star field marked `last_field` has gone.

use types::zebes::{PictureMotion, ScenePicture, UpwardSpeed, ZebesScene, ZebesStage, ZebesState};

/// Spec: `PlanetAndStarsSlide`. The planet's upward acceleration, 64/256 px per frame².
const PLANET_ACCELERATION: u16 = 64;
/// Spec: `PlanetAndStarsSlide`. Each star field's upward acceleration, 32/256 px per frame².
const STAR_ACCELERATION: u16 = 32;
/// A speed of one 256th of a pixel per frame moves a position this many 65,536ths.
const POSITION_PER_SPEED: i32 = 256;
/// Spec: `PlanetAndStarsSlide`. A picture whose whole y is below −128 px is removed.
const EXIT_Y: i16 = -128;

/// The upward acceleration of a picture that slides; the captions don't.
const fn acceleration(which: ScenePicture) -> Option<u16> {
    match which {
        ScenePicture::Planet => Some(PLANET_ACCELERATION),
        ScenePicture::Stars(_) => Some(STAR_ACCELERATION),
        ScenePicture::English | ScenePicture::Japanese => None,
    }
}

/// Spec: `PlanetAndStarsSlide` [R-slide-first, C-slide-floor]. A picture's speed gains its
/// acceleration, then the new speed rises it, the fraction carrying into the pixel.
fn rise(motion: PictureMotion, acceleration: u16) -> PictureMotion {
    let speed = motion.speed.0.saturating_add(acceleration);
    let rise = i32::from(speed).wrapping_mul(POSITION_PER_SPEED);
    let mut position = motion.position;
    position.y = position.y.moved(rise.wrapping_neg());
    PictureMotion {
        position,
        speed: UpwardSpeed(speed),
    }
}

/// Spec: `PlanetAndStarsSlide` [R-slide-removes, R-last-star, C-last-star, C-unmarked-star].
/// Every sliding picture moves; one whose whole y falls below −128 px is removed on this cut.
/// If the marked star field was removed, the flight leaves once all have moved.
pub fn slide(state: &mut ZebesState, scene: &ZebesScene) {
    let last_field = ScenePicture::Stars(scene.planet.last_field);
    let mut last_field_left = false;
    for (which, placed) in &mut state.pictures {
        let (Some(motion), Some(acceleration)) = (*placed, acceleration(which)) else {
            continue;
        };
        let moved = rise(motion, acceleration);
        if moved.position.y.pixel < EXIT_Y {
            *placed = None;
            last_field_left |= which == last_field;
        } else {
            *placed = Some(moved);
        }
    }
    if last_field_left {
        state.stage = ZebesStage::Leaving;
    }
}
