//! Room quake state carried between the scheduler, the quake model, the parts of the game that
//! start and stop shakes, and Drawing.

use core::num;

use serde::{Deserialize, Serialize};

/// Spec: `QuietQuakePhase`. The room's updates, shaking or not, counted around a cycle of
/// 65,536; it moves nothing by itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuakePhase(pub u16);

impl QuakePhase {
    /// One update further round the cycle: 65,535 is followed by 0.
    #[must_use]
    pub const fn advanced(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

/// Frames of shake: those left of the shake under way (0 when the room is steady), or those a
/// start names (0 making the start a stop).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuakeDuration(pub u16);

/// Spec: `ShakenLayers`, `ShakenActors`. What a shake moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuakeReach {
    /// Both layers and the actors.
    Whole,
    /// The back layer (the room's background) and the actors.
    Back,
    /// The front layer (the room's blocks) alone.
    Front,
    /// Both layers, not the actors.
    Scenery,
}

/// The smallest shake, 1 px.
const SMALLEST_SIZE: u8 = 1;
/// The largest shake, 3 px.
const LARGEST_SIZE: u8 = 3;

/// Spec: `ShakenLayers`. How far a shake moves what it reaches: 1, 2 or 3 px.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct QuakeSize(u8);

impl QuakeSize {
    /// The size of this many pixels, if it is one (1 to 3).
    #[must_use]
    pub const fn new(pixels: u8) -> Option<Self> {
        if pixels >= SMALLEST_SIZE && pixels <= LARGEST_SIZE {
            Some(Self(pixels))
        } else {
            None
        }
    }

    /// Its pixels.
    #[must_use]
    pub const fn pixels(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for QuakeSize {
    type Error = &'static str;

    fn try_from(pixels: u8) -> Result<Self, Self::Error> {
        Self::new(pixels).ok_or("a quake's size must be 1, 2 or 3 px")
    }
}

impl From<QuakeSize> for u8 {
    fn from(size: QuakeSize) -> Self {
        size.pixels()
    }
}

/// Spec: `ShakenLayers`. Which way a shake moves what it reaches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuakeDirection {
    /// Along x and y at once.
    Diagonal,
    /// Along x alone.
    Sideways,
    /// Along y alone.
    UpAndDown,
}

/// Spec: `ShakeStarts`. A kind of shake: what it moves, how far and which way.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuakeKind {
    /// What it moves.
    pub reach: QuakeReach,
    /// How far.
    pub size: QuakeSize,
    /// Which way.
    pub direction: QuakeDirection,
}

/// Spec: `ShakeStarts`, `ShakeCountsDown`. A shake under way: its kind and the frames it has
/// left, at least 1. A kind is present exactly while frames are left.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shake {
    /// Its kind.
    pub kind: QuakeKind,
    /// The frames it has left.
    pub remaining: num::NonZeroU16,
}

/// The room quake's complete State, to carry or restore: the phase, and the shake under way, if
/// one is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomQuakeState {
    /// The room's updates counted round the cycle.
    pub phase: QuakePhase,
    /// The shake under way; none while the room stays steady.
    pub shake: Option<Shake>,
}

impl RoomQuakeState {
    /// The frames of shake left: 0 while the room stays steady. Boss-ceres-ridley's
    /// `EscapeShakes` and its drawing of the door of Ridley's room read them.
    #[must_use]
    pub const fn remaining(&self) -> QuakeDuration {
        match self.shake {
            Some(shake) => QuakeDuration(shake.remaining.get()),
            None => QuakeDuration(0),
        }
    }
}
