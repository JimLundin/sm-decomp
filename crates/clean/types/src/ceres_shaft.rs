//! Restorable state shared by the shaft's actors, scene and drawing.

use serde::{Deserialize, Serialize};

use crate::{Pair, Px};

macro_rules! rumble_count {
    ($name:ident, $max:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(try_from = "u8", into = "u8")]
        pub struct $name(u8);

        impl $name {
            /// The largest count this quantity can hold.
            pub const MAX: Self = Self($max);
            /// No turns left.
            pub const ZERO: Self = Self(0);

            /// Restore a count within the allowed range.
            #[must_use]
            pub const fn new(value: u8) -> Option<Self> {
                if value <= Self::MAX.0 {
                    Some(Self(value))
                } else {
                    None
                }
            }

            /// The remaining turns.
            #[must_use]
            pub const fn get(self) -> u8 {
                self.0
            }

            /// Spend a turn, stopping at zero.
            #[must_use]
            pub const fn count_down(self) -> Self {
                Self(self.0.saturating_sub(1))
            }
        }

        impl TryFrom<u8> for $name {
            type Error = &'static str;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                Self::new(value).ok_or(concat!(stringify!($name), " is out of range"))
            }
        }

        impl From<$name> for u8 {
            fn from(value: $name) -> Self {
                value.get()
            }
        }
    };
}

const RUMBLE_LENGTH: u8 = 48;
const TURNS_BETWEEN_BURSTS: u8 = 4;

rumble_count!(
    RumbleTurns,
    RUMBLE_LENGTH,
    "Spec: `OverlayRumbles`. Overlay turns left in the rumble, 0 through 48."
);
rumble_count!(
    BurstWait,
    TURNS_BETWEEN_BURSTS,
    "Spec: `OverlayRumbles`. Turns until the next burst, 0 through 4."
);

/// Spec: `OverlayRumbles`. The four burst points, in the order they are visited.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BurstPoint {
    /// First point.
    First,
    /// Second point.
    Second,
    /// Third point.
    Third,
    /// Fourth point.
    Fourth,
}

/// Spec: `OverlayAppears`, `OverlayRumbles`. The escape overlay's phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShaftOverlayPhase {
    /// Placed, but not yet drawn.
    Opening,
    /// The figure is visible and the lights pause.
    Rumbling {
        /// Turns before the rumble ends.
        remaining: RumbleTurns,
        /// Zero means a burst is due on this turn.
        wait: BurstWait,
        /// Where the next burst appears.
        point: BurstPoint,
    },
    /// The figure is gone, but the lights still take turns.
    Hidden,
}

/// Spec: `OverlayAppears`. The caller holds the anchor separately from the running phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaftOverlay {
    /// Figure anchor in whole room pixels.
    pub anchor: Pair<Px>,
    /// The phase of the figure and rumble.
    pub phase: ShaftOverlayPhase,
}

/// Spec: `ShaftTilts`. The affine room view drawn by the display and read by mounted vents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaftView {
    /// The fixed pivot in whole room pixels.
    pub pivot: Pair<Px>,
    /// Signed 8.8 coefficients, in A, B, C, D order.
    pub matrix: [i16; 4],
}

/// Spec: `ShaftTilts`. Which way to move along the ordered samples.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TiltDirection {
    /// Toward the last sample.
    Forward,
    /// Toward the first sample.
    Backward,
}

/// Spec: `ShaftTilts`. The setup view and track cursor can be restored independently.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaftRotation {
    /// Whether the rumble has enabled turning.
    pub active: bool,
    /// Next sample's index in the room's track.
    pub next: usize,
    /// Direction of travel from the next sample.
    pub direction: TiltDirection,
    /// Turns left before using that sample.
    pub hold: u16,
    /// The current view, initially supplied by room setup.
    pub view: ShaftView,
}

/// Spec: `VentDrawFollowsShaft`. A drawing-only displacement from a vent's mount in signed
/// whole pixels, wide enough for the projection without wrapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VentDrawOffset(pub Pair<i32>);

impl VentDrawOffset {
    /// A newly placed vent draws at its mount.
    pub const ZERO: Self = Self(Pair { x: 0, y: 0 });
}
