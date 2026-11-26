//! The ordinary fight's saved wing clock, independent of the getaway flyby.

use core::num;

use serde::{Deserialize, Serialize};
use types::Pair;
use types::ceres_fight::WingBeat;

const FIRST_BEAT: WingBeat = WingBeat::new(0).expect("the wing cycle starts at zero");
const PLACED_BEAT_NUMBER: u8 = 5;
const PLACED_BEAT: WingBeat = WingBeat::new(PLACED_BEAT_NUMBER).expect("beat five is in the cycle");
const TAKEOFF_COUNT: u8 = 8;
const RENEWED_COUNT: u8 = 32;
const SPEED_BAND_WIDTH: u16 = 64;
const SPEED_BAND_PERIOD: u16 = 16;
const CAPPED_PACE: u8 = 48;
const PACES: [u8; 7] = [12, 14, 16, 18, 28, 32, 40];
const HALF_PACE_DIVISOR: u8 = 2;

/// Spec: `WingsBeat`. Beat units left, from 0 through 32.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct WingCount(u8);

impl WingCount {
    /// Restore a value in its allowed range.
    #[must_use]
    pub const fn new(value: u8) -> Option<Self> {
        if value <= RENEWED_COUNT {
            Some(Self(value))
        } else {
            None
        }
    }

    /// The value as a number.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for WingCount {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value).ok_or("WingCount is out of range")
    }
}

impl From<WingCount> for u8 {
    fn from(value: WingCount) -> Self {
        value.get()
    }
}

/// Spec: `WingsBeat`. Carry the current look and remaining count between ordinary fight runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FightWings {
    /// The current wing look.
    pub beat: WingBeat,
    /// Beat units left before the next look.
    pub remaining: WingCount,
}

impl FightWings {
    /// Placement starts on beat five, with no beat units remaining.
    pub const PLACED: Self = Self {
        beat: PLACED_BEAT,
        remaining: WingCount(0),
    };

    /// First take-off seeds the count without changing the beat.
    #[must_use]
    pub const fn starting_takeoff(self) -> Self {
        Self {
            remaining: WingCount(TAKEOFF_COUNT),
            ..self
        }
    }

    /// One ordinary animation opportunity, at the speed after bounds and before the figure steps.
    /// Subtracting to exactly zero holds the beat. Only a deficit advances it.
    #[must_use]
    pub fn ran(self, speed: Pair<num::Wrapping<i16>>) -> Self {
        if let Some(left) = self.remaining.get().checked_sub(Self::pace(speed)) {
            return Self {
                remaining: WingCount(left),
                ..self
            };
        }
        Self {
            beat: WingBeat::new(self.beat.get().saturating_add(1)).unwrap_or(FIRST_BEAT),
            remaining: WingCount(RENEWED_COUNT),
        }
    }

    /// Pace in beat units per run. The bands repeat and cap; downward or level motion
    /// halves the pace. The two simultaneous extreme negative speeds count as rest.
    #[must_use]
    pub fn pace(speed: Pair<num::Wrapping<i16>>) -> u8 {
        let (x, y) = (speed.x.0, speed.y.0);
        if (x, y) == (0, 0) || (x, y) == (i16::MIN, i16::MIN) {
            return 0;
        }
        let magnitude = x.unsigned_abs().max(y.unsigned_abs());
        let band = usize::from(magnitude / SPEED_BAND_WIDTH % SPEED_BAND_PERIOD);
        let pace = PACES.get(band).copied().unwrap_or(CAPPED_PACE);
        if y < 0 {
            pace
        } else {
            pace / HALF_PACE_DIVISOR
        }
    }
}
