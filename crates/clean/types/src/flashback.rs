//! The Zebes flashback encounter's State (screens-flashback).
//!
//! Mother Brain, the ring pairs, the rings, the bursts and the room background, carried from
//! one scene work frame to the next, saved and restored whole.
//!
//! A beat is one encounter step in one scene work frame; every hold here counts beats.

use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};

use crate::{Fixed, Pair, Parity, Px};

/// A hold of beats, at least one: how long a picture still shows, a burst still waits or a ring
/// pair is still due in, counting the coming beat. A live hold is positive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct HoldBeats(u16);

impl HoldBeats {
    /// The hold of this many beats, if it is at least one.
    #[must_use]
    pub const fn new(beats: u16) -> Option<Self> {
        if beats >= 1 { Some(Self(beats)) } else { None }
    }

    /// The hold of `BEATS` beats, a spec's number known when the model is built: a hold of no
    /// beats doesn't compile.
    #[must_use]
    pub const fn of<const BEATS: u16>() -> Self {
        const { assert!(BEATS >= 1, "a hold lasts at least one beat") };
        Self(BEATS)
    }

    /// Its beats.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    /// One beat of the hold spent: the hold left, or none when it expires on this beat.
    #[must_use]
    pub const fn counted_down(self) -> Option<Self> {
        Self::new(self.0.saturating_sub(1))
    }
}

impl TryFrom<u16> for HoldBeats {
    type Error = &'static str;

    fn try_from(beats: u16) -> Result<Self, Self::Error> {
        Self::new(beats).ok_or("a live hold lasts at least one beat")
    }
}

impl From<HoldBeats> for u16 {
    fn from(hold: HoldBeats) -> Self {
        hold.get()
    }
}

/// Where a clip has got to: its step and the beats its picture still holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Animation<S> {
    /// The clip's step.
    pub step: S,
    /// The beats it still holds, counting the coming one.
    pub hold: HoldBeats,
}

/// Spec: `BrainBreathes`. The steps of Mother Brain's breathing clip, in order; it repeats.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum BreathStep {
    /// The clip's first step.
    First,
    /// The second step.
    Second,
    /// The third step.
    Third,
    /// The fourth step, after which the clip begins again.
    Fourth,
}

/// Spec: `RingsAppear`, `RingsSweep`. The steps of a ring's clip, in order: three as it forms,
/// then four that repeat while it moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum RingStep {
    /// Forming: its first picture, on its birth beat.
    FormingFirst,
    /// Forming: the second picture.
    FormingSecond,
    /// Forming: the third picture.
    FormingThird,
    /// Moving: the first step of the repeating clip, ready to move from the next beat.
    MovingFirst,
    /// Moving: the second step.
    MovingSecond,
    /// Moving: the third step.
    MovingThird,
    /// Moving: the fourth step, after which the moving clip begins again.
    MovingFourth,
}

/// Spec: `BurstsPulse`. The steps of a burst's clip, in order: waiting for its first picture,
/// six pictures, then a blank interval, after which the pictures repeat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum BurstStep {
    /// Waiting out its delay, with no picture yet.
    Waiting,
    /// The first picture.
    First,
    /// The second picture.
    Second,
    /// The third picture.
    Third,
    /// The fourth picture.
    Fourth,
    /// The fifth picture.
    Fifth,
    /// The sixth picture.
    Sixth,
    /// The blank interval between rounds.
    Blank,
}

/// The longest flash, 8 beats.
const LONGEST_FLASH: u8 = 8;

/// Spec: `BrainFlashes`. The flash beats remaining after a missile hit: 0 to 8.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct FlashBeats(u8);

impl FlashBeats {
    /// No flash.
    pub const NONE: Self = Self(0);
    /// A whole flash, 8 beats: the flash a hit starts.
    pub const FULL: Self = Self(LONGEST_FLASH);

    /// The flash of this many beats, if it is 0 to 8.
    #[must_use]
    pub const fn new(beats: u8) -> Option<Self> {
        if beats <= LONGEST_FLASH {
            Some(Self(beats))
        } else {
            None
        }
    }

    /// Its beats.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Whether its count is even or odd.
    #[must_use]
    pub const fn parity(self) -> Parity {
        if self.0 & 1 == 0 {
            Parity::Even
        } else {
            Parity::Odd
        }
    }

    /// One beat of flash spent, staying at none.
    #[must_use]
    pub const fn spent(self) -> Self {
        Self(self.0.saturating_sub(1))
    }
}

impl TryFrom<u8> for FlashBeats {
    type Error = &'static str;

    fn try_from(beats: u8) -> Result<Self, Self::Error> {
        Self::new(beats).ok_or("a flash lasts 0 to 8 beats")
    }
}

impl From<FlashBeats> for u8 {
    fn from(flash: FlashBeats) -> Self {
        flash.get()
    }
}

/// Spec: `BrainEndures`. The missile hits Mother Brain has taken so far.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriorHits(pub u8);

impl PriorHits {
    /// No hit yet: how the encounter starts.
    pub const NONE: Self = Self(0);
}

/// Spec: `ReturnToPage`. The shaking beats Mother Brain has completed since the decisive impact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShakeBeats(pub u16);

impl ShakeBeats {
    /// No shaking beat yet: the decisive impact's beat.
    pub const NONE: Self = Self(0);
}

/// Where Mother Brain stands in the encounter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrainPhase {
    /// Spec: `BrainEndures`. She takes missile hits, fewer than the hit threshold so far.
    Endures {
        /// The hits taken.
        hits: PriorHits,
    },
    /// Spec: `ReturnToPage`. Defeated: she shakes while the bursts run, fewer beats than the
    /// shake hold so far.
    Shaking {
        /// The shaking beats completed.
        elapsed: ShakeBeats,
    },
    /// Spec: `ReturnToPage`. Page 2 is cued; she stays while the picture dissolves back.
    Returning,
}

/// Mother Brain while she is present: her phase, her flash and her breathing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brain {
    /// Where she stands in the encounter.
    pub phase: BrainPhase,
    /// The flash beats remaining.
    pub flash: FlashBeats,
    /// Her breathing clip.
    pub breath: Animation<BreathStep>,
}

/// Spec: `RingsAppear`. The two ring pairs, in the order they appear.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum RingPair {
    /// The first pair.
    First,
    /// The second, last pair.
    Second,
}

/// Spec: `RingsAppear`. The pair due next and the beats until it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairSchedule {
    /// The pair due next.
    pub pair: RingPair,
    /// The beats until it appears, counting the coming one.
    pub hold: HoldBeats,
}

/// Spec: `RingsAppear`. The four ring paths, each with its own start and travel; each has at
/// most one ring.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum RingPath {
    /// The first pair's ring toward Samus, the one that can hit her.
    FirstToward,
    /// The first pair's ring that appears on the far side of the room's actors.
    FirstAcross,
    /// The second pair's ring toward Samus.
    SecondToward,
    /// The second pair's ring that appears on the far side of the room's actors.
    SecondAcross,
}

/// Spec: `FightPictures`. Rank zero appears in front of every higher rank.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DrawRank(pub u8);

impl DrawRank {
    /// The foremost rank, 0.
    pub const FOREMOST: Self = Self(0);

    /// The rank just behind this one.
    #[must_use]
    pub const fn behind(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

/// A ring: where it is, its clip and its draw rank.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ring {
    /// Its room position, pixels with 16 fractional bits per axis.
    pub position: Pair<Fixed>,
    /// Its clip.
    pub clip: Animation<RingStep>,
    /// Its draw rank.
    pub rank: DrawRank,
}

/// Spec: `BurstsPulse`. The eight burst places around Mother Brain, in the order of the burst
/// table; each has at most one burst.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum BurstPlace {
    /// Small, just right of her.
    Near,
    /// Small, low on her left.
    Low,
    /// Small, high on her left.
    High,
    /// Large, at her centre.
    Centre,
    /// Large, upper right.
    UpperRight,
    /// Large, lower left.
    LowerLeft,
    /// Large, upper left.
    UpperLeft,
    /// Large, lower right.
    LowerRight,
}

/// A burst: its room position in whole pixels, its clip (its delay while it waits) and its draw
/// rank.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Burst {
    /// Its room position, whole pixels.
    pub at: Pair<Px>,
    /// Its clip; while it waits, the hold is the delay left.
    pub clip: Animation<BurstStep>,
    /// Its draw rank.
    pub rank: DrawRank,
}

/// Spec: `EncounterStarts`. Whether a start is admitted and waits for the encounter's first
/// beat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PendingStart {
    /// A start waits: the next beat is the encounter's first.
    Pending,
    /// No start waits.
    Clear,
}

/// Spec: `BackgroundBobs`. The room background's Y, signed 16-bit whole pixels; it wraps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomBackgroundY(pub i16);

/// The encounter's complete State. A caller may set each part at any value the spec allows,
/// read it back, carry it between scene work frames and restore it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterState {
    /// Whether a start waits for its first beat.
    pub start: PendingStart,
    /// The room background's Y, inherited from the intro at the start.
    pub background_y: RoomBackgroundY,
    /// Mother Brain, while she is present.
    pub brain: Option<Brain>,
    /// The ring pair due next, while one is.
    pub next_pair: Option<PairSchedule>,
    /// Each ring path's ring, where it has one.
    pub rings: EnumMap<RingPath, Option<Ring>>,
    /// Each burst place's burst, where it has one.
    pub bursts: EnumMap<BurstPlace, Option<Burst>>,
}

impl EncounterState {
    /// The State before the encounter: no start waiting, no boss, no pair due, no ring and no
    /// burst, with the room background at `background_y`.
    #[must_use]
    pub fn before_encounter(background_y: RoomBackgroundY) -> Self {
        Self {
            start: PendingStart::Clear,
            background_y,
            brain: None,
            next_pair: None,
            rings: EnumMap::default(),
            bursts: EnumMap::default(),
        }
    }
}
