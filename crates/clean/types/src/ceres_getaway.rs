//! The saved Ceres flyby and its request to Samus's movement.

use core::num;

use serde::{Deserialize, Serialize};

use crate::Pair;

/// Spec: `GetawayPass`. Ask movement to move Samus aside after her moves. This is not a hit:
/// movement chooses the direction, pose, travel and release, and health takes no damage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutOfTheWay;

/// Spec: `GetawayPicture`. The baby's place in the four-beat flyby cycle, not her held routine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlybyBabyBeat {
    /// Beat zero, first look.
    First,
    /// Beat one, middle look on the way to the third.
    ToThird,
    /// Beat two, third look.
    Third,
    /// Beat three, middle look on the way back to the first.
    ToFirst,
}

/// Spec: `GetawayPicture`. The wings' place in their two-beat flyby cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, enum_map::Enum, Serialize, Deserialize)]
pub enum FlybyWingBeat {
    /// Beat zero, first look.
    First,
    /// Beat one, second look.
    Second,
}

/// Spec: `GetawayPicture`. The two animation cycles advance independently on the game clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlybyBeats {
    /// The baby's beat.
    pub baby: FlybyBabyBeat,
    /// The wings' beat.
    pub wings: FlybyWingBeat,
}

impl FlybyBeats {
    /// Both cycles start at beat zero when the room starts the getaway.
    pub const START: Self = Self {
        baby: FlybyBabyBeat::First,
        wings: FlybyWingBeat::First,
    };
}

/// Spec: `GetawayPass`. Turn in 65,536ths of a revolution, wrapping at 16 bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlybyTurn(pub u16);

/// Spec: `GetawayPass`. A saved flyby after at least one picture, including the last.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlybyProgress {
    /// How many pictures were shown. The saved count belongs to the room path it was made with.
    pub shown: num::NonZeroUsize,
    /// Picture scroll, in wrapping signed whole pixels.
    pub scroll: Pair<i16>,
    /// The turn as of the last picture.
    pub turn: FlybyTurn,
    /// The last picture's animation beats.
    pub beats: FlybyBeats,
}

/// Spec: `GetawayPass`. State between runs, including either handoff.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GetawayState {
    /// The backdrop is prepared; the next run will start the path.
    GetawayReady(FlybyBeats),
    /// A picture has been shown; there may be another left.
    FlyingPast(FlybyProgress),
    /// The room view has been restored; the warning owns the next run.
    WarningReady,
}
