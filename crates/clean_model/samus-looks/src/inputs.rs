//! §1.3 What a frame reads, each from the domain that owns it, and §1.1 what it reports.

use clean_types::{AnimationSound, ClipEnd, Item, Set, Sprite};
use enum_map::Enum;

use crate::clip::{Clip, Frames, Paces};

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

/// Her vertical motion this frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Vertical {
    /// Rising, or not moving up or down.
    NotDown,
    /// Heading down, at this many whole pixels a frame (a fall at 0 included).
    Down(u16),
}

/// What the liquid she is in does to her animation (the liquids).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Liquid {
    /// Added to the holds her clip moves on to, 0 to 3.
    pub slowdown: Frames,
    /// Added to the first hold of the pose she takes, 0 to 3.
    pub start_slowdown: Frames,
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
    /// Her vertical motion.
    pub vertical: Vertical,
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
    /// The sound asked for: a frame moves her clip on once, so it asks for one at most
    /// (`WallJumpBranch`, `BoostStages`).
    pub sound: Option<AnimationSound>,
}
