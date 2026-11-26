//! The ribs' ordinary fight clock, independent of wing pace and display uploads.

use serde::{Deserialize, Serialize};
use types::ceres_fight::RibLook;

const REVEAL_PERIOD: u8 = 20;

/// Spec: `RibsBeat`. The next beat to reveal, in cycle order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RibBeat {
    /// Closed ribs.
    Closed,
    /// Partly open ribs that will widen next.
    Opening,
    /// Wide ribs.
    Wide,
    /// Partly open ribs that will close next.
    Closing,
}

impl RibBeat {
    const fn reveal(self) -> (RibLook, Self) {
        match self {
            Self::Closed => (RibLook::Closed, Self::Opening),
            Self::Opening => (RibLook::PartlyOpen, Self::Wide),
            Self::Wide => (RibLook::Wide, Self::Closing),
            Self::Closing => (RibLook::PartlyOpen, Self::Closed),
        }
    }
}

/// Spec: `RibsBeat`. Eligible runs left before the reveal, from 1 through 20.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct RibWait(u8);

impl RibWait {
    /// Restore a wait in its allowed range.
    #[must_use]
    pub const fn new(value: u8) -> Option<Self> {
        if value > 0 && value <= REVEAL_PERIOD {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Eligible runs left.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for RibWait {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value).ok_or("RibWait is out of range")
    }
}

impl From<RibWait> for u8 {
    fn from(value: RibWait) -> Self {
        value.get()
    }
}

/// Spec: `RibsBeat`. Save the next beat and its wait together; drawing owns the revealed look.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FightRibs {
    /// The next beat to reveal.
    pub beat: RibBeat,
    /// Eligible runs left until the reveal.
    pub remaining: RibWait,
}

impl FightRibs {
    /// The first ordinary animation run reveals the closed look.
    pub const PLACED: Self = Self {
        beat: RibBeat::Closed,
        remaining: RibWait(1),
    };

    /// Spend one ordinary animation opportunity, emitting a look only when the wait ends.
    pub(crate) const fn run(&mut self) -> Option<RibLook> {
        if let Some(left) = RibWait::new(self.remaining.get().saturating_sub(1)) {
            self.remaining = left;
            return None;
        }
        let (look, next) = self.beat.reveal();
        self.beat = next;
        self.remaining = RibWait(REVEAL_PERIOD);
        Some(look)
    }
}
