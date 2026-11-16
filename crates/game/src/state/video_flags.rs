//! Typed flag sets used by exact video encoding.

use canonical::Canonical;
use std::marker::PhantomData;

/// A member of a [`FlagSet`]: its bit is its place in declaration order.
pub trait Flag: Copy {
    /// The member's bit in a set.
    fn bit(self) -> u32;
}

/// A set of `E`s: bit `n` holds `E`'s `n`th variant in declaration order.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlagSet<E> {
    /// The members, one bit per variant.
    pub bits: u32,
    /// The member type.
    pub members: PhantomData<E>,
}

impl<E> Default for FlagSet<E> {
    fn default() -> Self {
        Self {
            bits: 0,
            members: PhantomData,
        }
    }
}

impl<E> FlagSet<E> {
    /// Whether the set has no members.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bits == 0
    }

    /// The members of `self` that aren't in `other`.
    #[must_use]
    pub const fn difference(&self, other: &Self) -> Self {
        Self {
            bits: self.bits & !other.bits,
            members: PhantomData,
        }
    }
}

impl<E: Flag> FlagSet<E> {
    /// Whether `member` is in the set.
    #[must_use]
    pub fn contains(&self, member: E) -> bool {
        (self.bits & member.bit()) != 0
    }

    /// Add `member`.
    pub fn insert(&mut self, member: E) {
        self.bits |= member.bit();
    }

    /// Take `member` out.
    pub fn remove(&mut self, member: E) {
        self.bits &= !member.bit();
    }
}
