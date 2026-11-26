//! The shared E4 plane rule: from the ship plane's pose to the matrix and scroll E4 draws
//! with, about the imported picture origin.
//!
//! The pose's angle says how far the ship picture is turned, in 256ths of a full turn. Its
//! zoom says how large the picture is drawn, where 256 is actual size. The matrix combines the
//! two: it turns the picture by the angle and scales it by the zoom. Each of its values is
//! counted in 256ths and rounded down to the whole 256th below. The scroll is the pose's
//! position rounded down to the whole pixel below.

use types::zebes::{PixelPoint, PlaneAngle, PlanePose};

/// A quarter turn, in 256ths of a turn.
const QUARTER_TURN: u8 = 64;
/// A half turn, in 256ths of a turn.
const HALF_TURN: u8 = 128;
/// The plane's unit: zoom and matrix entries are in 256ths.
const UNIT: i32 = 256;

/// The sine of each angle of the first quarter turn, 0 through 64 256ths of a turn, in 256ths
/// and rounded toward zero. The other quarters mirror it.
const QUARTER_SINE: [i32; 65] = [
    0, 6, 12, 18, 25, 31, 37, 43, 49, 56, 62, 68, 74, 80, 86, 92, 97, 103, 109, 115, 120, 126, 131,
    136, 142, 147, 152, 157, 162, 167, 171, 176, 181, 185, 189, 193, 197, 201, 205, 209, 212, 216,
    219, 222, 225, 228, 231, 234, 236, 238, 241, 243, 244, 246, 248, 249, 251, 252, 253, 254, 254,
    255, 255, 255, 256,
];

/// The square matrix E4 draws the plane with, each value in 256ths.
///
/// It turns the picture by the pose's angle and scales it by the pose's zoom. Both values on
/// its diagonal are the zoom times the cosine of the angle. The value on the upper right is
/// the zoom times the sine of the angle, and the value on the lower left is that same product
/// negated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaneMatrix {
    /// The upper left value: the zoom times the cosine of the angle.
    pub a: i32,
    /// The upper right value: the zoom times the sine of the angle.
    pub b: i32,
    /// The lower left value: the zoom times the sine of the angle, negated.
    pub c: i32,
    /// The lower right value: the zoom times the cosine of the angle, the same as the upper
    /// left.
    pub d: i32,
}

/// What E4 is given each frame: the matrix, the whole-pixel scroll and the imported origin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaneTransform {
    /// How the picture is turned and scaled.
    pub matrix: PlaneMatrix,
    /// The scroll, rounded toward the lower whole pixel.
    pub scroll: PixelPoint,
    /// The ship picture's origin, from `DrawingPages`.
    pub origin: PixelPoint,
}

/// The sine of an angle in 256ths, from the quarter table.
fn sine(angle: PlaneAngle) -> i32 {
    let within_half = angle.0.rem_euclid(HALF_TURN);
    let index = if within_half <= QUARTER_TURN {
        within_half
    } else {
        HALF_TURN.wrapping_sub(within_half)
    };
    let size = QUARTER_SINE.get(usize::from(index)).copied().unwrap_or(0);
    if angle.0 < HALF_TURN {
        size
    } else {
        size.wrapping_neg()
    }
}

/// The cosine of an angle in 256ths: the sine a quarter turn on.
fn cosine(angle: PlaneAngle) -> i32 {
    sine(PlaneAngle(angle.0.wrapping_add(QUARTER_TURN)))
}

/// Multiply the zoom by the direction component and round down.
fn scaled(pose: PlanePose, ratio: i32) -> i32 {
    i32::from(pose.zoom.0)
        .wrapping_mul(ratio)
        .wrapping_div_euclid(UNIT)
}

/// The E4 plane rule for a pose about the imported origin.
#[must_use]
pub fn transform(pose: PlanePose, origin: PixelPoint) -> PlaneTransform {
    let along = scaled(pose, cosine(pose.angle));
    let across = scaled(pose, sine(pose.angle));
    PlaneTransform {
        matrix: PlaneMatrix {
            a: along,
            b: across,
            c: across.wrapping_neg(),
            d: along,
        },
        scroll: PixelPoint {
            x: pose.scroll.x.pixel,
            y: pose.scroll.y.pixel,
        },
        origin,
    }
}

#[cfg(test)]
mod tests {
    use super::{QUARTER_SINE, UNIT};

    /// Each entry is 256 sin(2π k / 256) rounded toward zero.
    #[test]
    fn the_quarter_table_is_the_sine_rounded_toward_zero() {
        for (step, entry) in (0_u32..).zip(QUARTER_SINE) {
            let exact = f64::from(UNIT) * (std::f64::consts::TAU * f64::from(step) / 256.0).sin();
            assert!(
                f64::from(entry) <= exact + 1e-9 && exact < f64::from(entry) + 1.0,
                "step {step}: {entry} for {exact}"
            );
        }
    }
}
