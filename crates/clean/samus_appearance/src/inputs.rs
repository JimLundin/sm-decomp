//! What a frame brings, each fact once, from the domain that owns it, and the content her
//! appearance shows: her clips, the run's paces and her pulse's colours.

use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};
use types::{Fixed, Item, Pose, RowColours, Set, Sprite};

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

/// The suit selection made for this frame, from the equipment switched on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum Suit {
    /// Neither Gravity nor Varia is enabled.
    Power,
    /// Varia is enabled without Gravity.
    Varia,
    /// Gravity is enabled, with or without Varia.
    Gravity,
}

/// The grapple beam's current activity, from Grapple.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrappleActivity {
    /// Grapple is dormant.
    Dormant,
    /// Grapple is busy.
    Busy,
}

/// Spec: `HurtColours`. The colours her hurt pulse shows in her row, as the importer supplies
/// them: 16 colours each, her row's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PulseColours {
    /// Her hurt colours.
    pub hurt: RowColours,
    /// Each suit's colours.
    pub suits: EnumMap<Suit, RowColours>,
    /// The cinematic's grey colours, which stand for her suit's while a cinematic is active.
    pub cinematic_grey: RowColours,
}

/// The content her appearance shows, as the importer supplies it: a clip for each figure the
/// content holds, the run's paces, and the colours of her hurt pulse.
pub trait AppearanceContent {
    /// The clip of `figure`, or none where the content holds no clip for it.
    fn clip(&self, figure: &Figure) -> Option<&Clip>;
    /// The run's paces.
    fn paces(&self) -> &Paces;
    /// The colours her hurt pulse shows (`HurtColours`).
    fn pulse_colours(&self) -> &PulseColours;
}

/// The figure whose clip a frame or the read-ahead needs and the content lacks. The model gives
/// it back for the caller to refuse the frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MissingClip(pub Figure);

/// The clip `content` holds for `figure`, or the figure as missing.
pub fn clip_of(content: &impl AppearanceContent, figure: Figure) -> Result<&Clip, MissingClip> {
    content.clip(&figure).ok_or(MissingClip(figure))
}

/// What samus-movement's step did this frame that her appearance reads. Listed in no order.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Enum, Serialize, Deserialize)]
pub enum MoveEvent {
    /// "The standing animation restarts" (`StandingRestart`).
    StandingRestart,
    /// Its near-wall test readied a wall jump (`WallJumpReady`).
    WallJumpReadied,
    /// It took the pose her clip's end handed over (`HandOverSlowdown`).
    HandedOver,
    /// A hit's push started this frame: samus-movement's "push started" (`HurtColours`).
    PushStarted,
}

/// Spec: `RunPace`, `BoostStages`. Her dash this frame: one value for its whole lifecycle.
///
/// Samus-movement's dash after her moves, dash began and dash lost together give it. The clip
/// moves on in the middle of the frame; a dash that began did so before it, and a dash lost is
/// lost by the frame's end.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Dash {
    /// Not dashing all frame.
    Off,
    /// Dashing all frame.
    On,
    /// The dash began this frame, before the clip moves on, and goes on.
    Began,
    /// Dashing as the clip moves on, and lost by the frame's end.
    Lost,
    /// The dash began this frame before the clip moves on, and is lost by the frame's end: she
    /// slows down in the same pose.
    BeganAndLost,
    /// Her moves stopped the dash before the clip moves on (a wall halts her, say), and it is
    /// lost by the frame's end.
    Stopped,
}

impl Dash {
    /// Whether she is dashing as the clip moves on, so her stride takes the run's pace.
    #[must_use]
    pub const fn dashing_as_the_clip_moves_on(self) -> bool {
        match self {
            Self::On | Self::Began | Self::Lost | Self::BeganAndLost => true,
            Self::Off | Self::Stopped => false,
        }
    }

    /// Whether the dash began this frame, before the clip moves on.
    #[must_use]
    pub const fn began(self) -> bool {
        match self {
            Self::Began | Self::BeganAndLost => true,
            Self::Off | Self::On | Self::Lost | Self::Stopped => false,
        }
    }

    /// Whether the dash is lost by the frame's end.
    #[must_use]
    pub const fn lost(self) -> bool {
        match self {
            Self::Lost | Self::BeganAndLost | Self::Stopped => true,
            Self::Off | Self::On | Self::Began => false,
        }
    }
}

/// The liquids' three facts for the frame, each its own, none following from another. The
/// gravity suit zeroes the start slowdown in every liquid and the step slowdown only in lava.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Liquid {
    /// The step slowdown: frames added to the hold of each step her clip moves on to
    /// (`ClipPlays`, `ClipControls`, `Breathing`, `WallJumpBranch`, `RunPace`, `BoostStages`),
    /// and to the first step of a pose a clip's end handed over (`HandOverSlowdown`).
    pub step_slowdown: Slowdown,
    /// The start slowdown: frames added to the hold of the step the clip of the pose she takes
    /// starts on, by the liquid at the feet of that pose (`StartingClip`).
    pub start_slowdown: Slowdown,
    /// She is under a liquid's surface, the liquid slows her and she lacks the gravity suit
    /// (`WallJumpBranch`, `SpinStarts`).
    pub submerged: bool,
}

impl Liquid {
    /// Out of a liquid: both slowdowns 0, and she is not submerged.
    pub const NONE: Self = Self {
        step_slowdown: Slowdown::NONE,
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
    /// Special colour time remains (appearance's colour owner, part 2).
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

    /// The charge of `frames`, if it is 0 to `MOST`.
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
/// Her figure comes twice, as the frame starts and as it ends, because samus-movement picks the
/// pose she takes part-way through the frame: her stride and the preparation of her work come
/// before the pick, the clip of the first figure plays out before the clip of the second starts,
/// and the spin start compares the two. On a frame she doesn't move, the two are the same.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Inputs {
    /// Her figure as the frame starts: the one whose clip her State is on.
    pub figure: Figure,
    /// The figure she ends the frame in: samus-movement's choice of pose, in its sprite.
    pub figure_after: Figure,
    /// What her moves did this frame.
    pub moves: Set<MoveEvent>,
    /// Her dash this frame, from its beginning to its loss.
    pub dash: Dash,
    /// The pad holds run this frame.
    pub run_held: bool,
    /// Heading down, as her moves leave her: she falls, a fall at 0 included, outside the
    /// moonfall; a rise a ceiling stops this frame counts as rising (`RisingHold`).
    pub heading_down: bool,
    /// Her vertical speed as her moves leave her, downward positive, a fact apart from heading
    /// down: she can rise fast in a fall's pose (`FastFall`).
    pub vertical_speed: Fixed,
    /// Her energy, as the frame leaves Health (`Breathing`, `HurtColours`).
    pub energy: u16,
    /// Her energy as the frame before left it, which her energy falls from (`HurtColours`).
    pub previous_energy: u16,
    /// The items switched on; this domain reads the speed booster, the space jump and the screw
    /// attack.
    pub items: Set<Item>,
    /// The step slowdown, the start slowdown, and whether she is submerged.
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

impl Inputs {
    /// Spec: `HurtColours`. Whether her energy fell this frame: it is below the energy the frame
    /// before left.
    #[must_use]
    pub const fn energy_fell(&self) -> bool {
        self.energy < self.previous_energy
    }
}

/// How many sounds wait in the queue the speed booster's sound joins, before it, 0 to
/// `SoundsWaiting::MOST` (the sound domain; `BoostingLap`). The caller supplies it on every
/// frame.
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

    /// The count of `waiting`, if it is 0 to `MOST`.
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
