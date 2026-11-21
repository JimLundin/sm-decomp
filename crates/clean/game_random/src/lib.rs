//! The numeric rules for the game's shared random sequence. Callers decide when to draw and
//! what to do with the resulting number.

mod catalogue;
pub use catalogue::{Behaviour, BehaviourId, CATALOGUE, Check, Kind, Switch};
pub use types::{RandomNumber, RandomSequence};

// Spec: AdvanceRandom. The digit base, multiplier, and two biases of a draw.
const DIGIT_BASE: u32 = 256;
const MULTIPLIER: u32 = 5;
const HIGH_DIGIT_BIAS: u32 = 1;
const FINAL_BIAS: u32 = 17;
const SEQUENCE_MODULUS: u32 = 65_536;

/// Spec: `AdvanceRandom`. Complete one ordinary draw, updating the sequence and returning its
/// new current number.
///
/// The high digit's product wraps before biasing; the low digit's product carries. The biased
/// sum's carry is added back before the final wrap.
pub fn advance_random(sequence: &mut RandomSequence) -> RandomNumber {
    let [low, high] = sequence.number.0.to_le_bytes();
    let low_product = u32::from(low).wrapping_mul(MULTIPLIER);
    let high_product = u32::from(high).wrapping_mul(MULTIPLIER) % DIGIT_BASE;
    // The intermediate products and sums fit in u32, so only the spec's reductions wrap.
    let biased = low_product.wrapping_add(
        high_product
            .wrapping_add(HIGH_DIGIT_BIAS)
            .wrapping_mul(DIGIT_BASE),
    );
    let carry = biased / SEQUENCE_MODULUS;
    let drawn = biased.wrapping_add(FINAL_BIAS).wrapping_add(carry);
    let [low, high, ..] = drawn.to_le_bytes(); // Keep the low two digits: mod 65536.
    sequence.number = RandomNumber(u16::from_le_bytes([low, high]));
    sequence.number
}
