//! `TailBends`, `GetawayPicture`: the sine of a whole 256th of a turn, in 256ths.

pub const WHOLE_TURN: u16 = 256;
pub const QUARTER_TURN: u16 = 64;
const HALF_TURN: u16 = 128;

/// The first quarter wave of trunc(256 × sin(2πk / 256)), endpoints included.
const QUARTER_WAVE: [i16; 65] = [
    0, 6, 12, 18, 25, 31, 37, 43, 49, 56, 62, 68, 74, 80, 86, 92, 97, 103, 109, 115, 120, 126, 131,
    136, 142, 147, 152, 157, 162, 167, 171, 176, 181, 185, 189, 193, 197, 201, 205, 209, 212, 216,
    219, 222, 225, 228, 231, 234, 236, 238, 241, 243, 244, 246, 248, 249, 251, 252, 253, 254, 254,
    255, 255, 255, 256,
];

/// Reflect and negate the quarter wave for the other quadrants, with exact quarter turns.
pub fn sine(angle: u16) -> i32 {
    let half = angle % HALF_TURN;
    let index = half.min(HALF_TURN.saturating_sub(half));
    let magnitude = i32::from(QUARTER_WAVE.get(usize::from(index)).copied().unwrap_or(0));
    if angle % WHOLE_TURN < HALF_TURN {
        magnitude
    } else {
        magnitude.saturating_neg()
    }
}
