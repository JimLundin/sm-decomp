//! §1.3 What a frame reads, each from the domain that owns it, and §1.1 what it reports.

use clean_types::{AnimationSound, ClipEnd, Fixed, Item, Set, Sprite};
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
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
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
    /// The sounds waiting in the queue the speed booster's sound joins, 0 to 15 (`BoostingLap`).
    pub sounds_waiting: u8,
}

/// What a frame reports (§1.1), in §6's order: the end, known from the frame's start, then what
/// the pose she takes shows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Report {
    /// The end her clip reaches this frame (`ClipEnds`); also given ahead by
    /// `Playhead::end_ahead`.
    pub end: Option<ClipEnd>,
    /// Her spin has turned far enough to kick off a wall (`SpinTurnedFar`).
    pub turned_far: bool,
    /// The sound asked for this frame, at most one: only a control asks for one
    /// (`WallJumpBranch`'s branch, `BoostStages`' lap end), and a frame meets one control at
    /// most.
    pub sound: Option<AnimationSound>,
}
