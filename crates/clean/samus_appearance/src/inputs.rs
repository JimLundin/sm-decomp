//! What a frame brings, each fact once, from the domain that owns it.

use enum_map::Enum;
use serde::{Deserialize, Serialize};
use types::{Fixed, Item, Pose, Set, Sprite};

use crate::clip::{Clip, Paces, Slowdown};

/// Her figure: samus-movement's pose with the sprite she is drawn in. Each pose and sprite has its
/// own clip, and she takes another pose when either part differs.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Figure {
    /// Samus-movement's pose: body, action, facing, aim, gun, legs.
    pub pose: Pose,
    /// The sprite `HowSheIsDrawn` chooses for it.
    pub sprite: Sprite,
}

/// The content: her clips, one for each pose and sprite, and the run's paces.
pub trait Content {
    /// The clip of a pose drawn in its sprite.
    fn clip(&self, figure: &Figure) -> &Clip;
    /// The run's paces.
    fn paces(&self) -> &Paces;
}

/// What samus-movement's step did this frame, that this domain reads. Listed in no order.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Enum, Serialize, Deserialize)]
pub enum MoveEvent {
    /// "The standing animation restarts" (`StandingRestart`).
    StandingRestart,
    /// Its near-wall test readied a wall jump (`WallJumpReady`).
    WallJumpReadied,
    /// It took the pose her clip's end handed over (`HandOverSlowdown`).
    HandedOver,
    /// Her dash was lost this frame (`BoostStages`).
    DashLost,
}

/// Whether she is dashing as the clip moves on (`RunPace`, `BoostStages`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Dash {
    /// She is not dashing.
    Off,
    /// She is dashing.
    On,
    /// A dash began this frame, before the clip moves on: she is dashing.
    Began,
}

/// What the liquid she is in does to her animation (the liquids).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Liquid {
    /// Added to the holds her clip moves on to.
    pub slowdown: Slowdown,
    /// The start's slowdown: added to the first hold of the pose she takes.
    pub start_slowdown: Slowdown,
    /// Under a liquid's surface that slows her, without the gravity suit (`WallJumpBranch`,
    /// `SpinStarts`).
    pub submerged: bool,
}

impl Liquid {
    /// No liquid: out of one, nothing slows her and she is not submerged.
    pub const NONE: Self = Self {
        slowdown: Slowdown::NONE,
        start_slowdown: Slowdown::NONE,
        submerged: false,
    };
}

/// An owner's activity that the frame's sound rules read: a cinematic, a boss, special colour
/// time (`FootContact`; the cinematic also `SpinStarts`). Listed in no order.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Enum, Serialize, Deserialize)]
pub enum Activity {
    /// A cinematic is active (the cinematic owner).
    Cinematic,
    /// A boss is active (the boss owner).
    Boss,
    /// Special colour time remains (appearance's colour owner, part 2); not the palette
    /// opportunity's special colouring.
    SpecialColourTime,
}

/// How long her shot has charged, in frames, 0 to `Charge::MOST` (weapons; `AtTheDoor`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u16")]
pub struct Charge(u16);

impl TryFrom<u16> for Charge {
    type Error = &'static str;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value).ok_or("charge must be from 0 to 32767 frames")
    }
}

impl Charge {
    /// The most a shot charges: 32767 frames.
    pub const MOST: u16 = 32767;
    /// Spec: `AtTheDoor`. Below 16 frames of charge, a passage quiets her.
    pub const QUIET_BELOW: u16 = 16;

    /// The charge of `frames`, if the spec allows it: 0 to `MOST`.
    #[must_use]
    pub const fn new(frames: u16) -> Option<Self> {
        if frames <= Self::MOST {
            Some(Self(frames))
        } else {
            None
        }
    }

    /// Its frames.
    #[must_use]
    pub const fn frames(self) -> u16 {
        self.0
    }
}

/// What a frame brings: each fact once, from the domain that owns it, read by every rule that
/// needs it.
///
/// Her pose comes twice, as the frame starts and as it ends, because samus-movement picks the
/// pose she takes part-way through the frame: her stride and the preparation of her work come
/// before the pick, the clip of the first pose plays out before the clip of the second starts,
/// and the spin start compares the two. On a frame she doesn't move, the two are the same.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Inputs {
    /// Her figure as the frame starts: the one whose clip her State is on.
    pub figure: Figure,
    /// The figure she ends the frame in: samus-movement's choice of pose, in its sprite.
    pub figure_after: Figure,
    /// What her moves did this frame.
    pub moves: Set<MoveEvent>,
    /// Whether she is dashing as the clip moves on.
    pub dash: Dash,
    /// The pad holds run this frame.
    pub run_held: bool,
    /// Heading down, as her moves leave her: she falls, a fall at 0 included, and not in the
    /// moonfall; a rise a ceiling stops this frame is not (`RisingHold`).
    pub heading_down: bool,
    /// Her vertical speed as her moves leave her, downward positive, read apart from heading
    /// down: she can rise fast in a fall's pose (`FastFall`).
    pub vertical_speed: Fixed,
    /// Her energy (`Breathing`).
    pub energy: u16,
    /// The items switched on; this domain reads the speed booster, the space jump and the screw
    /// attack.
    pub items: Set<Item>,
    /// The liquid's slowdowns, and whether she is submerged.
    pub liquid: Liquid,
    /// The sounds waiting in the queue the speed booster's sound joins (`BoostingLap`).
    pub sounds_waiting: SoundsWaiting,
    /// The owners active this frame: a cinematic, a boss, special colour time.
    pub activities: Set<Activity>,
    /// The pad holds shoot (`AtTheDoor`).
    pub shoot_held: bool,
    /// How long her shot has charged (`AtTheDoor`).
    pub charge: Charge,
}

/// How many sounds wait in the queue the speed booster's sound joins, before it (the sound
/// domain): 0 to `SoundsWaiting::MOST`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(try_from = "u8")]
pub struct SoundsWaiting(u8);

impl TryFrom<u8> for SoundsWaiting {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value).ok_or("sounds waiting must be from 0 to 15")
    }
}

impl SoundsWaiting {
    /// At most 15 sounds wait.
    pub const MOST: u8 = 15;
    /// No sounds waiting, as on every record of her starting to boost.
    pub const NONE: Self = Self(0);

    /// The count of `waiting`, if the spec allows it: 0 to `MOST`.
    #[must_use]
    pub const fn new(waiting: u8) -> Option<Self> {
        if waiting <= Self::MOST {
            Some(Self(waiting))
        } else {
            None
        }
    }

    /// How many wait.
    #[must_use]
    pub const fn count(self) -> u8 {
        self.0
    }
}
