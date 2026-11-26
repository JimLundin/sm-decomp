//! Spec: `SpaceDrifts`. The asteroid fields drift right in front of the purple background,
//! which alternates its two drawings every frame.

use enum_map::EnumMap;
use types::explosion::{BackdropFrame, ExplosionState, PictureAge, SkyContent, Subpixels};

/// Spec: `SpaceDrifts`. An asteroid field's x wraps at 512 px.
const SKY_WIDTH: Subpixels = Subpixels::whole(512);
/// Spec: `SpaceDrifts`. The purple background's two drawings take turns.
const BACKDROP_DRAWINGS: u32 = 2;

/// Spec: `SpaceDrifts`. Places the sky's pictures at their starts, with no bursts, and gives
/// them the placement frame's advance. With the frame's own advance, the first setup picture
/// shows two sky updates (`StationPictureArrives`).
pub fn place(state: &mut ExplosionState, sky: &SkyContent) {
    state.age = PictureAge(0);
    state.asteroids =
        EnumMap::from_fn(|field| Subpixels::whole(i32::from(sky.asteroids[field].start.x)));
    state.bursts.clear();
    advance(state, sky);
}

/// Spec: `SpaceDrifts`. One finished frame: the pictures age by one and each asteroid field
/// moves right by its imported speed, its x wrapping at 512 px.
pub fn advance(state: &mut ExplosionState, sky: &SkyContent) {
    state.age = PictureAge(state.age.0.saturating_add(1));
    for (field, x) in &mut state.asteroids {
        *x = x.plus(sky.asteroids[field].speed).wrapped(SKY_WIDTH);
    }
}

/// Spec: `SpaceDrifts`. The purple background's drawing at this age: the first at even ages,
/// the second at odd ones.
#[must_use]
pub const fn backdrop_frame(age: PictureAge) -> BackdropFrame {
    if age.0.is_multiple_of(BACKDROP_DRAWINGS) {
        BackdropFrame::First
    } else {
        BackdropFrame::Second
    }
}
