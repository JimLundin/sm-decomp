//! The baby's discovery (screens-discovery): the egg, the baby, the shell's fragments and the
//! droplets, and the sprite order, carried from one finished frame to the next, saved and
//! restored.

use serde::{Deserialize, Serialize};

use crate::Pair;

/// A position's fraction: 65,536ths of a pixel, the low 16 bits.
const FRACTION_BITS: u32 = 16;
/// 65,536ths of a pixel in one eighth of a pixel, the unit of the baby's speeds.
const SUBPIXELS_PER_EIGHTH: i32 = 8_192;

/// One axis of a room position: signed pixels with a fraction in 65,536ths of a pixel. X grows
/// rightward and y downward, in the same room coordinates as Samus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coord(pub i32);

impl Coord {
    /// The whole pixel it lies in, rounded toward the lower pixel: every whole-pixel comparison
    /// and every drawing reads this.
    #[must_use]
    pub const fn whole(self) -> i32 {
        self.0.wrapping_shr(FRACTION_BITS)
    }

    /// Its whole pixel with no fraction: where a picture is anchored.
    #[must_use]
    pub const fn snapped(self) -> Self {
        Self(self.whole().wrapping_shl(FRACTION_BITS))
    }

    /// Moved by `by`.
    #[must_use]
    pub const fn moved(self, by: Offset) -> Self {
        Self(self.0.wrapping_add(by.0))
    }
}

/// A signed distance along one axis in 65,536ths of a pixel: a drift or a vertical displacement
/// of a fragment or a droplet, or one frame's move of the baby.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offset(pub i32);

/// A position in the room: an x and a y `Coord`.
pub type RoomPoint = Pair<Coord>;

impl Pair<Coord> {
    /// The whole pixels it lies in, rounded toward the lower pixel on each axis.
    #[must_use]
    pub const fn whole(self) -> Pair<i32> {
        Pair {
            x: self.x.whole(),
            y: self.y.whole(),
        }
    }

    /// Its whole-pixel anchor: the same pixels with no fraction.
    #[must_use]
    pub const fn snapped(self) -> Self {
        Self {
            x: self.x.snapped(),
            y: self.y.snapped(),
        }
    }
}

/// The baby's speed along one axis: signed eighths of a pixel per finished frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Speed(pub i8);

impl Speed {
    /// No speed.
    pub const STILL: Self = Self(0);

    /// The change in position over one complete frame.
    #[must_use]
    pub fn per_frame(self) -> Offset {
        Offset(i32::from(self.0).wrapping_mul(SUBPIXELS_PER_EIGHTH))
    }

    /// Whether it points leftward or upward.
    #[must_use]
    pub const fn negative(self) -> bool {
        self.0 < 0
    }
}

/// Spec: `EggHatches`. The egg: absent, waiting for Samus, hatching, or spent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Egg {
    /// No egg: the dormant encounter, or one removed at the return dissolve's end.
    Absent,
    /// Closed, waiting for Samus to approach.
    Waiting,
    /// Rocking and opening.
    Hatching {
        /// Finished frames since the hatch animation's first picture, which is age zero.
        age: u16,
    },
    /// Open, after the hatch animation's last hold.
    Spent,
}

/// Spec: `SlimeFalls`. Whether the rising baby has shed its droplets yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Slime {
    /// The droplets are still to shed.
    Clinging,
    /// The droplets have been shed; a rise never sheds them twice.
    Shed,
}

/// Spec: `BabyEmerges`, `BabyFollows`. How the present baby moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BabyMotion {
    /// Still inside the egg.
    Inside,
    /// Rising out of the shell toward its place above Samus.
    Rising {
        /// Its vertical speed.
        vertical: Speed,
        /// Whether it has shed its droplets.
        slime: Slime,
    },
    /// Holding still above Samus.
    Hovering {
        /// Finished frames of hold left.
        left: u8,
    },
    /// Chasing Samus.
    Following {
        /// Its speed on each axis.
        velocity: Pair<Speed>,
        /// Finished frames of following counted toward its two cries; it stops at the second.
        cry: u8,
    },
}

/// Spec: `BabyPulses`. Where the baby's picture is in its repeating cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pulse {
    /// Newly placed: neither the baby nor the egg has a picture until the first finished frame.
    Unseen,
    /// Finished frames since the start of the current drawing cycle.
    Age(u16),
}

/// The baby, while it is present: its place, its motion and its pulse.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hatchling {
    /// Its place in the room.
    pub place: RoomPoint,
    /// How it moves.
    pub motion: BabyMotion,
    /// Where its picture is in the cycle.
    pub pulse: Pulse,
}

/// A fragment's or a droplet's identity: its place in the scene's list of that cohort.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PieceId(pub u8);

/// Spec: `ShellScatters`. A shell fragment in flight.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlyingFragment {
    /// Which of the scene's fragments it is.
    pub identity: PieceId,
    /// Its place in the room.
    pub place: RoomPoint,
    /// The finished frames of flight already applied.
    pub steps: u16,
}

/// Spec: `SlimeFalls`. Where a droplet is in its fall.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DropletStage {
    /// Falling, with the finished frames of flight already applied.
    Falling {
        /// The steps applied.
        steps: u16,
    },
    /// Stopped on the floor and splashing, with the finished frames since impact.
    Splashing {
        /// Frames since impact.
        age: u16,
    },
}

/// Spec: `SlimeFalls`. A droplet shed by the rising baby: falling, then splashing where it
/// stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlimeDroplet {
    /// Which of the scene's droplets it is.
    pub identity: PieceId,
    /// Its place in the room: where it stopped, once it splashes.
    pub place: RoomPoint,
    /// Falling or splashing.
    pub stage: DropletStage,
}

/// Up to `MOST` pieces of one cohort, in the scene's order: the first has foreground precedence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    try_from = "Vec<T>",
    into = "Vec<T>",
    bound(
        serialize = "T: Serialize + Clone",
        deserialize = "T: Deserialize<'de>"
    )
)]
pub struct Cohort<T, const MOST: usize>(Vec<T>);

impl<T, const MOST: usize> Cohort<T, MOST> {
    /// No pieces.
    #[must_use]
    pub const fn empty() -> Self {
        Self(Vec::new())
    }

    /// These pieces, if there are at most `MOST`.
    #[must_use]
    pub fn new(pieces: Vec<T>) -> Option<Self> {
        (pieces.len() <= MOST).then_some(Self(pieces))
    }

    /// The first `MOST` of these pieces.
    #[must_use]
    pub fn gathered(pieces: impl IntoIterator<Item = T>) -> Self {
        Self(pieces.into_iter().take(MOST).collect())
    }

    /// The pieces, in order.
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    /// Each piece replaced by what `next` makes of it, or removed where it makes none.
    pub fn update(&mut self, next: impl FnMut(T) -> Option<T>) {
        self.0 = self.0.drain(..).filter_map(next).collect();
    }
}

impl<T, const MOST: usize> TryFrom<Vec<T>> for Cohort<T, MOST> {
    type Error = &'static str;

    fn try_from(pieces: Vec<T>) -> Result<Self, Self::Error> {
        Self::new(pieces).ok_or("a cohort holds too many pieces")
    }
}

impl<T, const MOST: usize> From<Cohort<T, MOST>> for Vec<T> {
    fn from(cohort: Cohort<T, MOST>) -> Self {
        cohort.0
    }
}

/// The most shell fragments in flight.
pub const MOST_FRAGMENTS: usize = 6;
/// The most droplets, falling or splashing.
pub const MOST_DROPLETS: usize = 4;

/// The shell fragments in flight: up to six.
pub type Fragments = Cohort<FlyingFragment, MOST_FRAGMENTS>;
/// The droplets, falling or splashing: up to four.
pub type Droplets = Cohort<SlimeDroplet, MOST_DROPLETS>;

/// The encounter's place in sprite priority. Production schedules the room's turn from it at
/// frame start and publishes the draw order at frame end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryOrder {
    /// The encounter ahead of Samus and her projectiles.
    SceneFirst,
    /// Samus and her projectiles ahead of the encounter.
    SamusFirst,
    /// The encounter, Samus and her projectiles all omitted.
    Hidden,
}

/// The discovery's complete State. A caller may set any part to any value its types allow, read
/// it back, carry it forward and restore it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryState {
    /// The egg.
    pub egg: Egg,
    /// The baby, while it is present.
    pub baby: Option<Hatchling>,
    /// The shell fragments in flight.
    pub fragments: Fragments,
    /// The droplets, falling or splashing.
    pub droplets: Droplets,
    /// The encounter's place in sprite priority.
    pub order: DiscoveryOrder,
}

impl DiscoveryState {
    /// The dormant encounter: no egg, no baby, no pieces, hidden.
    #[must_use]
    pub const fn dormant() -> Self {
        Self {
            egg: Egg::Absent,
            baby: None,
            fragments: Cohort::empty(),
            droplets: Cohort::empty(),
            order: DiscoveryOrder::Hidden,
        }
    }
}
