#[cfg_attr(test, expect(clippy::arithmetic_side_effects, reason = "sums"))]
pub fn f(a: u8) -> u8 { a }
