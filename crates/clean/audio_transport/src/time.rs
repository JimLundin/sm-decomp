//! Spec: VisibleCompletion. Exact unbounded rational AudioCycles arithmetic.
use crate::{AudioInstant, InstantFraction};
use num_bigint::BigUint;
use std::{borrow::Borrow, num::NonZeroU64};
/// Spec: VisibleCompletion. Normalize ordinary callers' unsigned fractions.
#[must_use]
pub fn instant(numerator: u64, denominator: NonZeroU64) -> AudioInstant {
    let value = InstantFraction {
        numerator: numerator.into(),
        denominator: denominator.get().into(),
    };
    // Both unsigned components satisfy the domain invariants by construction.
    AudioInstant::try_from(value).unwrap_or_else(|_| AudioInstant::whole(0u64.into()))
}
/// Spec: VisibleCompletion. Ordinary whole audio-cycle interval.
#[must_use]
pub fn cycles(value: u64) -> AudioInstant {
    AudioInstant::whole(value.into())
}
/// Spec: VisibleCompletion. Unbounded whole audio-cycle instant.
#[must_use]
pub fn whole(value: BigUint) -> AudioInstant {
    AudioInstant::whole(value)
}
/// Spec: StableRequests. Half-cycle offsets.
#[must_use]
pub fn halves(value: u64) -> AudioInstant {
    const DENOMINATOR: u64 = 2;
    instant(
        value,
        NonZeroU64::new(DENOMINATOR).unwrap_or(NonZeroU64::MIN),
    )
}
/// Spec: VisibleCompletion. Complete exact comparison.
#[must_use]
pub fn compare(a: impl Borrow<AudioInstant>, b: impl Borrow<AudioInstant>) -> std::cmp::Ordering {
    a.borrow().cmp(b.borrow())
}
/// Spec: VisibleCompletion. Exact normalized sum.
#[must_use]
pub fn add(a: impl Borrow<AudioInstant>, b: impl Borrow<AudioInstant>) -> AudioInstant {
    a.borrow().sum(b.borrow())
}
/// Spec: VisibleCompletion. Nonnegative difference, zero for earlier instants.
#[must_use]
pub fn subtract(a: impl Borrow<AudioInstant>, b: impl Borrow<AudioInstant>) -> AudioInstant {
    a.borrow().difference(b.borrow())
}
/// Spec: ClockObservation. Unbounded floor for timer accumulation.
#[must_use]
pub fn floor(a: impl Borrow<AudioInstant>) -> BigUint {
    a.borrow().rational().to_integer().magnitude().clone()
}
