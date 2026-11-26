//! The numbers the encounter's rules read. The reference profile holds the spec's values.

use types::Pair;
use types::discovery::{Coord, Speed};

/// How the baby chases a target along one axis: each finished frame its speed changes by `rate`
/// toward the target, no faster than `limit` either way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chase {
    /// The change of speed each finished frame.
    pub rate: Speed,
    /// The fastest speed either way.
    pub limit: Speed,
}

/// The encounter's numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Spec: `EggHatches`. The egg hatches once Samus's whole-pixel x is below this.
    pub approach_x: i32,
    /// Spec: `ShellScatters`, `SlimeFalls`. The floor's whole-pixel y: a fragment reaching it
    /// goes, a droplet reaching it splashes.
    pub floor_y: i32,
    /// Spec: `BabyEmerges`. How far above Samus's centre the rise aims, pixels.
    pub rise_height: i32,
    /// Spec: `BabyEmerges`. The rise's vertical chase.
    pub rise: Chase,
    /// Spec: `BabyEmerges`. How many finished frames the baby hovers.
    pub hover_frames: u8,
    /// Spec: `SlimeFalls`. The whole-pixel y at which the rising baby sheds its droplets.
    pub shed_y: i32,
    /// Spec: `BabyFollows`. How far above Samus's centre the baby aims while it follows, pixels.
    pub follow_height: i32,
    /// Spec: `BabyFollows`. The follow's chase on each axis.
    pub follow: Pair<Chase>,
    /// Spec: `BabyFollows`. The cry age of the first cry.
    pub first_cry: u8,
    /// Spec: `BabyFollows`. The cry age of the second cry, where the age then stays.
    pub last_cry: u8,
}

impl Rules {
    /// The reference game's numbers.
    pub const REFERENCE: Self = Self {
        approach_x: 169,
        floor_y: 168,
        rise_height: 32,
        rise: Chase {
            rate: Speed(1),
            limit: Speed(17),
        },
        hover_frames: 128,
        shed_y: 145,
        follow_height: 8,
        follow: Pair {
            x: Chase {
                rate: Speed(1),
                limit: Speed(20),
            },
            y: Chase {
                rate: Speed(1),
                limit: Speed(17),
            },
        },
        first_cry: 64,
        last_cry: 128,
    };
}

impl Chase {
    /// The new speed and place after one finished frame chasing `target`: faster toward the
    /// positive limit when the target is at or beyond the whole-pixel place, otherwise faster
    /// toward the negative limit, then moved by the new speed.
    #[must_use]
    pub fn toward(self, target: i32, place: Coord, speed: Speed) -> (Coord, Speed) {
        let speed = if target >= place.whole() {
            Speed(speed.0.saturating_add(self.rate.0).min(self.limit.0))
        } else {
            Speed(
                speed
                    .0
                    .saturating_sub(self.rate.0)
                    .max(self.limit.0.saturating_neg()),
            )
        };
        (place.moved(speed.per_frame()), speed)
    }
}
