//! Whole-number counts, and the divisors the item percentage divides them by.

use core::num::NonZeroU16;

/// A count or a capacity: a whole number of 16 bits that wraps past 65535 back to 0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Count(u16);

impl Count {
    /// None at all.
    pub const ZERO: Self = Self(0);
    /// One more.
    pub const ONE: Self = Self(1);

    /// A count of `n`.
    #[must_use]
    pub const fn new(n: u16) -> Self {
        Self(n)
    }

    /// The count as a number.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    /// Whether there is none.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// This count and `more` together, wrapping past 65535 back to 0.
    #[must_use]
    pub const fn plus(self, more: Self) -> Self {
        Self(self.0.wrapping_add(more.0))
    }

    /// How many whole times `divisor` goes into this count: the division rounds down.
    #[must_use]
    pub(crate) const fn divided_by(self, divisor: Divisor) -> Self {
        Self(self.0.div_euclid(divisor.0.get()))
    }
}

/// A number a count is divided by, never zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Divisor(NonZeroU16);

impl Divisor {
    /// A divisor of `n`, or nothing when `n` is zero.
    #[must_use]
    pub const fn new(n: u16) -> Option<Self> {
        match NonZeroU16::new(n) {
            Some(n) => Some(Self(n)),
            None => None,
        }
    }

    /// The divisor as a number.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0.get()
    }
}
