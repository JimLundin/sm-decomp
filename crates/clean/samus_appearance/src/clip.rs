//! Her clips and the run's paces: the content the importer supplies, as typed values.

use enum_map::EnumMap;
use types::{BoostStage, ClipEnd};

/// A place in a clip, numbered from 0: her step is the one her clip is on, and each step holds
/// one instruction.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, serde::Serialize, serde::Deserialize,
)]
pub struct Step(pub u8);

impl Step {
    /// The first step of every clip.
    pub const FIRST: Self = Self(0);
    /// The step after the first.
    pub const SECOND: Self = Self(1);

    /// The step after this one, which the clip moves on to.
    #[must_use]
    pub fn next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }

    /// The step before this one.
    #[must_use]
    pub fn before(self) -> Option<Self> {
        self.0.checked_sub(1).map(Self)
    }

    fn index(self) -> usize {
        usize::from(self.0)
    }
}

/// The time her step has left, in frames, 0 to 255: every value of its type.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, serde::Serialize, serde::Deserialize,
)]
pub struct Frames(pub u8);

impl Frames {
    /// No time left.
    pub const NONE: Self = Self(0);
    /// One frame left: the last frame of a step, after which the clip moves on.
    pub const LAST: Self = Self(1);

    /// These frames lengthened by `slowdown`. A hold is at most `Hold::LONGEST`, 127 frames, and
    /// a slowdown at most `Slowdown::MOST`, 3, so a hold slowed twice (`HandOverSlowdown`) is at
    /// most 133 frames, within the time left's 0 to 255. The add saturates only beyond any
    /// value those bounds reach.
    #[must_use]
    pub const fn slowed(self, slowdown: Slowdown) -> Self {
        Self(self.0.saturating_add(slowdown.0))
    }
}

/// The frames a step shows when one of her moves sets it, the frame that sets it the first.
///
/// They are the restart's (`StandingRestart`), the fast fall's (`FastFall`) and the rising
/// frames (`RisingHold`): 1 to `ShownFrames::MOST`, and no liquid slows them.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "u8")]
pub struct ShownFrames(pub(crate) u8);

impl TryFrom<u8> for ShownFrames {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value).ok_or("frames shown must be from 1 to 126")
    }
}

impl ShownFrames {
    /// At most 126 frames, so the time left they set stays within a hold's bound.
    pub const MOST: u8 = 126;

    /// The frames of `frames`, if they are 1 to `MOST`.
    #[must_use]
    pub fn new(frames: u8) -> Option<Self> {
        (1..=Self::MOST).contains(&frames).then_some(Self(frames))
    }

    /// How many frames she shows.
    #[must_use]
    pub const fn count(self) -> u8 {
        self.0
    }

    /// The time left the move sets, before the frame's time runs down: the frames plus 1, since
    /// the frame that sets them runs one of them down and shows it.
    #[must_use]
    pub const fn before_the_run_down(self) -> Frames {
        Frames(self.0.saturating_add(1))
    }
}

/// A shown step's hold, as the content gives it: 1 frame or more and at most `Hold::LONGEST`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "u8")]
pub struct Hold(pub(crate) u8);

impl TryFrom<u8> for Hold {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value).ok_or("hold must be from 1 to 127 frames")
    }
}

impl Hold {
    /// The spec bounds a hold at 127 frames (the game's are at most 80), so a hold with both
    /// slowdowns stays within the time left's 0 to 255.
    pub const LONGEST: u8 = 127;

    /// The hold of `frames`, if it is 1 to `LONGEST`.
    #[must_use]
    pub fn new(frames: u8) -> Option<Self> {
        (1..=Self::LONGEST)
            .contains(&frames)
            .then_some(Self(frames))
    }

    /// Its frames.
    #[must_use]
    pub const fn frames(self) -> Frames {
        Frames(self.0)
    }
}

/// A slowdown: frames a liquid adds to a hold, 0 to `Slowdown::MOST`; the step slowdown and the
/// start slowdown each are one (`Liquid`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "u8")]
pub struct Slowdown(u8);

impl TryFrom<u8> for Slowdown {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value).ok_or("slowdown must be from 0 to 3 frames")
    }
}

impl Slowdown {
    /// A slowdown is at most 3 frames, in water.
    pub const MOST: u8 = 3;
    /// No slowdown: out of a liquid, or in one that doesn't slow her.
    pub const NONE: Self = Self(0);

    /// The slowdown of `frames`, if it is 0 to `MOST`.
    #[must_use]
    pub fn new(frames: u8) -> Option<Self> {
        (frames <= Self::MOST).then_some(Self(frames))
    }
}

/// The steps a wall jump's `branch` goes on from (`WallJumpBranch`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Branch {
    /// The plain spin's step, `x`.
    pub spin: Step,
    /// The space jump's step, `y`.
    pub space_jump: Step,
    /// The screw attack's step, `z`.
    pub screw_attack: Step,
}

/// What a step of a clip does when the clip reaches it: show its picture for a hold, or a
/// control.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Instruction {
    /// `n`: shows its picture for its hold (`ClipPlays`).
    Show(Hold),
    /// A control, which the clip passes through in the frame it reaches it unless it rests there.
    Control(Control),
}

/// A step that shows no picture of its own: it says where the clip goes, or that it ends.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Control {
    /// `rest`: she rests on it with no time left, and the clip moves on from it the next frame
    /// (`ClipControls`).
    Rest,
    /// `again s`: the clip goes on from step `s` (`ClipControls`).
    Again(Step),
    /// `breathe`: back to step 0 while her energy is high, else on (`Breathing`).
    Breathe,
    /// `end`: the clip ends (`ClipEnds`).
    End,
    /// `end-unless-jump`: the clip ends unless the pad asks for a jump (`ClipEnds`).
    EndUnlessJump,
    /// `branch x y z`: a wall jump's spin, by her items (`WallJumpBranch`).
    Branch(Branch),
}

impl Control {
    /// `ClipEnds`: the end this control is, if it is one.
    #[must_use]
    pub const fn end(self) -> Option<ClipEnd> {
        match self {
            Self::End => Some(ClipEnd::End),
            Self::EndUnlessJump => Some(ClipEnd::EndUnlessJump),
            Self::Rest | Self::Again(_) | Self::Breathe | Self::Branch(_) => None,
        }
    }
}

/// A clip: its instructions from step 0, and the steps it marks.
///
/// In the content the importer supplies, an `again` or a `branch` goes to a shown step, a
/// `rest` is followed by a shown step, and the last step is a control. On any other clip the
/// model still answers: a clip that would move on past its end leaves her where she is, and a
/// step with no hold gives no time left.
#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Clip {
    /// The instructions, one per step, in order from step 0.
    pub instructions: Vec<Instruction>,
    /// A spin's kick step (`WallJumpReady`, `SpinTurnedFar`).
    pub kick: Option<Step>,
    /// A fall's fast fall step (`FastFall`).
    pub fast_fall: Option<Step>,
    /// A straight jump's rising step (`RisingHold`).
    pub rising: Option<Step>,
    /// Spec: `FootContact`. Shown contact steps, independent of holds and other marks.
    /// The importer owns their relationship to the instructions.
    pub contacts: std::collections::BTreeSet<Step>,
}

impl Clip {
    /// The instruction at `step`, if the clip reaches that far.
    #[must_use]
    pub fn at(&self, step: Step) -> Option<Instruction> {
        self.instructions.get(step.index()).copied()
    }

    /// The hold of the shown step at `step`; none for a control.
    #[must_use]
    pub fn hold(&self, step: Step) -> Option<Hold> {
        match self.at(step)? {
            Instruction::Show(hold) => Some(hold),
            Instruction::Control(_) => None,
        }
    }
}

/// One of the run's paces: a hold for each shown step of her running clips, by step number.
/// Where a pace gives no hold for a step, the clip's own hold shows.
#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Pace(pub Vec<Hold>);

impl Pace {
    /// The pace's hold for `step`, if it gives one.
    #[must_use]
    pub fn hold(&self, step: Step) -> Option<Hold> {
        self.0.get(step.index()).copied()
    }
}

/// The run's paces: the plain pace, and a boosted pace for each boost stage.
#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Paces {
    /// Her stride without the speed booster.
    pub plain: Pace,
    /// Her stride with it, at each stage.
    pub boosted: EnumMap<BoostStage, Pace>,
}
