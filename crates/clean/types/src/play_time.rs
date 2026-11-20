//! Complete play-time state shared by the clock and its callers.

/// Spec: `PlayTimeAdvance`. The last whole hour on the clock.
const LAST_HOUR: u8 = 99;
/// Spec: `PlayTimeAdvance`. The last minute, second or frame before a carry.
const LAST_SUBDIVISION: u8 = 59;

macro_rules! play_time_part {
    ($part:ident, $max:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        #[serde(try_from = "u8")]
        pub struct $part(u8);

        impl $part {
            /// The zero component after a carry.
            pub const ZERO: Self = Self(0);
            /// The largest legal value of this component.
            pub const MAX: Self = Self($max);

            /// Builds this component if the value is within its legal range.
            #[must_use]
            pub const fn new(value: u8) -> Option<Self> {
                if value <= $max {
                    Some(Self(value))
                } else {
                    None
                }
            }

            /// Reads this component in its own units.
            #[must_use]
            pub const fn get(self) -> u8 {
                self.0
            }
        }

        impl TryFrom<u8> for $part {
            type Error = &'static str;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                Self::new(value).ok_or(concat!(stringify!($part), " is out of range"))
            }
        }
    };
}

play_time_part!(
    PlayHours,
    LAST_HOUR,
    "Spec: `PlayTimeAdvance`. Whole hours, 0 through 99."
);
play_time_part!(
    PlayMinutes,
    LAST_SUBDIVISION,
    "Spec: `PlayTimeAdvance`. Minutes within the hour, 0 through 59."
);
play_time_part!(
    PlaySeconds,
    LAST_SUBDIVISION,
    "Spec: `PlayTimeAdvance`. Seconds within the minute, 0 through 59."
);
play_time_part!(
    PlayFrames,
    LAST_SUBDIVISION,
    "Spec: `PlayTimeAdvance`. Clock frames within the second, 0 through 59."
);

/// Spec: `PlayTimeAdvance`. Complete clock state; all four components are retained on restore.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayTime {
    /// Whole hours.
    pub hours: PlayHours,
    /// Minutes within the hour.
    pub minutes: PlayMinutes,
    /// Seconds within the minute.
    pub seconds: PlaySeconds,
    /// Clock frames within the second.
    pub frames: PlayFrames,
}
