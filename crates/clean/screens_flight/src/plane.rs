//! Spec: `PlaneTransforms`. Every frame the plane is drawn turned by its angle and scaled by its
//! zoom: the one rule that gives its transformation.

use std::f64::consts::TAU;

use types::flight::{Angle, Plane, Zoom};

/// A quarter turn, in 256ths of a turn: the cosine is the sine a quarter turn on.
const QUARTER_TURN: u8 = 64;
/// Half a turn: the sine's second half is its first, negated.
const HALF_TURN: u8 = 128;
/// A whole turn, in the angle's units.
const WHOLE_TURN: u16 = 256;
/// The sine's scale and the zoom's: 256ths.
const UNIT: i16 = 256;

/// The plane's transformation, four values in 256ths, as the renderer's plane takes them:
/// `(a, b, c, d)` with `a = d` the scaled cosine, `b` the scaled sine and `c = −b`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaneTransform {
    /// The scaled cosine.
    pub a: i16,
    /// The scaled sine.
    pub b: i16,
    /// The scaled sine, negated.
    pub c: i16,
    /// The scaled cosine.
    pub d: i16,
}

/// Spec: `PlaneTransforms`. `a = d = floor(cos(angle) × zoom / 256)`,
/// `b = floor(sin(angle) × zoom / 256)`, `c = −b`.
#[must_use]
pub fn transform(plane: &Plane) -> PlaneTransform {
    let cosine = scaled(
        sine(Angle(plane.angle.0.wrapping_add(QUARTER_TURN))),
        plane.zoom,
    );
    let sine = scaled(sine(plane.angle), plane.zoom);
    PlaneTransform {
        a: cosine,
        b: sine,
        c: sine.saturating_neg(),
        d: cosine,
    }
}

/// A sine or cosine times the zoom, in 256ths, rounded down. It always fits 16 bits (the sine
/// is at most 256 and the zoom at most 32,767); it would stop at the nearest end if not.
fn scaled(trig: i16, zoom: Zoom) -> i16 {
    let product = i32::from(trig).wrapping_mul(i32::from(zoom.get()));
    let value = product.div_euclid(i32::from(UNIT));
    i16::try_from(value).unwrap_or(if value < 0 { i16::MIN } else { i16::MAX })
}

/// The game's sine, `trunc(256 × sin(2π a / 256))`, from −256 to 256: the first quarter turn's
/// values, mirrored into the other three.
fn sine(angle: Angle) -> i16 {
    let within_half = angle.0 % HALF_TURN;
    let size = quarter_sine(within_half.min(HALF_TURN.wrapping_sub(within_half)));
    if angle.0 < HALF_TURN {
        size
    } else {
        size.saturating_neg()
    }
}

/// `trunc(256 × sin(2π a / 256))` for an angle of the first quarter turn, 0 to 64: the
/// largest whole number not above it, 0 to 256.
fn quarter_sine(angle: u8) -> i16 {
    let exact = f64::from(UNIT) * (TAU * f64::from(angle) / f64::from(WHOLE_TURN)).sin();
    (0..=UNIT)
        .rev()
        .find(|&whole| f64::from(whole) <= exact)
        .unwrap_or(0)
}
