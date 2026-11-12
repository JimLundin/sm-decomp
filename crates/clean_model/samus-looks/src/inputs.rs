//! §1.3 What a frame reads, each from the domain that owns it.

use clean_types::{Fixed, Item, Set, Sprite};
use enum_map::Enum;

use crate::clip::{Clip, Paces, Slowdown};

/// §1.2 Her pose: samus-motion's pose with the sprite she is drawn in. Each pose and sprite has
/// its own clip, and she takes another pose when either part differs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pose {
    /// Samus-motion's pose: body, action, facing, aim, gun, legs.
    pub motion: clean_types::Pose,
    /// The sprite `HowSheIsDrawn` chooses for it.
    pub sprite: Sprite,
}

/// The content (§1.4): her clips, by pose, and the run's paces.
pub trait Content {
    /// The clip of a pose.
    fn clip(&self, pose: &Pose) -> &Clip;
    /// The run's paces.
    fn paces(&self) -> &Paces;
}

/// What samus-motion's step did this frame, that this domain reads. Listed in no order.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Enum)]
pub enum Move {
    /// A dash began this frame, before the clip moves on (`BoostStages`).
    DashBegan,
    /// "The standing animation restarts" (`StandingRestart`).
    StandingRestart,
    /// Its near-wall test readied a wall jump (`WallJumpReady`).
    WallJumpReadied,
    /// It took the pose her clip's end handed over (`HandOverSlowdown`).
    HandedOver,
    /// Her dash was lost this frame (`BoostStages`).
    DashLost,
}

/// What the liquid she is in does to her animation (the liquids).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Liquid {
    /// Added to the holds her clip moves on to.
    pub slowdown: Slowdown,
    /// The start's slowdown: added to the first hold of the pose she takes.
    pub start_slowdown: Slowdown,
    /// Under a liquid's surface that slows her, without the gravity suit (`WallJumpBranch`).
    pub submerged: bool,
}

/// The frame's inputs (§1.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Inputs {
    /// Her pose at the frame's start.
    pub pose: Pose,
    /// The pose she ends the frame in, samus-motion's choice.
    pub pose_after: Pose,
    /// What her moves did this frame.
    pub moves: Set<Move>,
    /// She has a dash's momentum as the clip moves on (`RunPace`, `BoostStages`).
    pub dashing: bool,
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
}

/// How many sounds wait in the queue the speed booster's sound joins, before it (the sound
/// domain): 0 to `SoundsWaiting::MOST`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SoundsWaiting(u8);

impl SoundsWaiting {
    /// Spec §1.3: at most 15 sounds wait.
    pub const MOST: u8 = 15;

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
