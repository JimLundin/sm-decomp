//! The game's random number generator.

/// Generate random number ($80:8111): the next value after `seed`, as the
/// original's byte-wise multiply by 5 plus 17 (with the low byte's carry).
pub fn next(seed: u16) -> u16 {
    let [low, high] = seed.to_le_bytes();
    let [product_low, product_high] = (u16::from(low) * 5).to_le_bytes();
    let carry = u16::from(product_high) + u16::from(high.wrapping_mul(5)) + 1;
    let [carry_low, _] = carry.to_le_bytes();
    u16::from_le_bytes([product_low, carry_low]).wrapping_add(17 + (carry >> 8))
}
