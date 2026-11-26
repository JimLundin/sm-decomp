//! The flyby's out-of-the-way shove, separate from damage and its knockback.

use core::num;
use types::{Action, Fixed, Parts, Pose, Px, Side, fixed};

const DOWNWARD_SPEED: Fixed = num::Wrapping(0x0005_0000);

/// `GetawayArms`, `GetawayStarts`, `GetawayTravels`: the flyby request and the stored
/// side of travel. The side does not change the way the recoil pose faces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Getaway {
    /// Ordinary movement.
    Idle,
    /// The flyby has requested the shove for the next cut.
    Pending,
    /// Moving aside until a horizontal stop.
    Travelling(Side),
}

impl Getaway {
    /// The forced downward speed, separate from the ordinary vertical motion whose
    /// heading and falling phase are preserved until release.
    #[must_use]
    pub const fn downward_speed(self) -> Option<Fixed> {
        match self {
            Self::Travelling(_) => Some(DOWNWARD_SPEED),
            Self::Idle | Self::Pending => None,
        }
    }
}

/// `GetawayTravels`: caller-supplied horizontal slowdown, in halvings (0 through 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slowdown(u8);

impl Slowdown {
    /// No slowdown, as in dry Ceres.
    pub const NONE: Self = Self(0);
    const MAX_INPUT: u8 = 5;
    const MAX_APPLIED: u8 = 4;

    /// Construct a slowdown count, rejecting values outside the caller input range.
    #[must_use]
    pub fn new(halvings: u8) -> Option<Self> {
        (halvings <= Self::MAX_INPUT).then_some(Self(halvings))
    }

    /// `GetawayTravels`: halve the wrapped sum of recoil speed and dash as an unsigned magnitude.
    pub(crate) fn reduce(self, magnitude: Fixed) -> Fixed {
        num::Wrapping(magnitude.0 >> self.0.min(Self::MAX_APPLIED))
    }
}

/// `GetawayTravels`: sign the reduced speed, add room carry, then clamp the signed whole
/// word without changing its fractional word.
pub fn displacement(speed: Fixed, side: Side, carry: Fixed, limit: Px) -> Fixed {
    let signed = if side == Side::Left { -speed } else { speed } + carry;
    let limit = limit.0.min(i16::MAX.cast_unsigned()).cast_signed();
    let whole = signed
        .pixel()
        .0
        .cast_signed()
        .clamp(limit.wrapping_neg(), limit);
    fixed(num::Wrapping(whole.cast_unsigned()), signed.sub())
}

/// Which deferred inputs the caller must discard. These are not the recorded pose choices
/// or draw-based jump history, both of which still go through their usual interfaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DeferredMotion {
    /// Leave queued inputs alone.
    Keep,
    /// `GetawayStarts`: cancel any pending lift from a blast and clear all deferred pose choices
    /// and commands.
    Clear,
    /// `GetawayTravels`: discard only a deferred backward leap and its associated command.
    DiscardBackwardLeap,
}

impl DeferredMotion {
    /// Cancel the caller-owned request for a lift from a blast when the shove starts.
    pub fn discard_blast_lift<B>(self, pending: &mut Option<B>) {
        if self == Self::Clear {
            *pending = None;
        }
    }

    /// Apply the pose part of the directive to a caller-owned deferred pose and command.
    pub fn discard_pose<C>(self, deferred: &mut Option<(Pose, C)>) {
        if self == Self::Clear
            || self == Self::DiscardBackwardLeap
                && deferred
                    .as_ref()
                    .is_some_and(|(pose, _)| pose.action == Action::Leaping)
        {
            *deferred = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_clamp_keeps_the_fractional_word() {
        let minus_sixteen_and_a_quarter = num::Wrapping(0xFFEF_C000);
        let by = displacement(
            types::ZERO,
            Side::Left,
            minus_sixteen_and_a_quarter,
            num::Wrapping(15),
        );
        // Whole word −15, fraction C000, i.e. −14.25 px (decimal).
        assert_eq!(by, num::Wrapping(0xFFF1_C000));
    }

    #[test]
    fn slowdown_uses_the_wrapped_sum_as_unsigned() {
        let halves = Slowdown::new(1).unwrap();
        assert_eq!(
            displacement(
                halves.reduce(num::Wrapping(0xFFFF_0001_u32) + num::Wrapping(0x0002_0000)),
                Side::Right,
                types::ZERO,
                num::Wrapping(15),
            ),
            num::Wrapping(0x0000_8000)
        );
        assert_eq!(
            displacement(
                halves.reduce(num::Wrapping(0x8001_0003)),
                Side::Right,
                types::ZERO,
                num::Wrapping(15),
            ),
            num::Wrapping(0x000F_8001)
        );
    }
}
