//! The run's counts and capacities.

use core::num::Wrapping;

/// A count or a capacity. The spec's notation: counts and capacities are whole numbers of 16 bits
/// and wrap at 65536, so 65535 and 2 more is 1: its `+` is `Wrapping`'s.
pub type Count = Wrapping<u16>;

/// `n` as a count: like every count, it keeps its low 16 bits, wrapping at 65536.
#[must_use]
pub const fn count(n: u32) -> Count {
    let [low, high, ..] = n.to_le_bytes();
    Wrapping(u16::from_le_bytes([low, high]))
}
