//! §1.4 Her clips and the run's paces: the content the importer supplies, as typed values.

use clean_types::{BoostStage, ClipEnd};
use enum_map::EnumMap;

/// A place in a clip, numbered from 0 (§1.4): her step is the one her clip is on, and each step
/// holds one instruction.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
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

/// A count of frames: a hold, a slowdown, or the time her step has left (0 to 255, §1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct Frames(pub u8);

impl Frames {
    /// No frames.
    pub const NONE: Self = Self(0);

    /// This hold lengthened by a slowdown. A hold is at most 127 frames and a slowdown at most 3,
    /// and a hold takes two slowdowns at most (`HandOverSlowdown`), so the sum is at most 133
    /// and always fits the time left's 0 to 255 (§1.5): nothing wraps. The add saturates only
    /// because the crate refuses a bare `+` on a byte; within the spec it never reaches 255.
    #[must_use]
    pub const fn slowed(self, slowdown: Self) -> Self {
        Self(self.0.saturating_add(slowdown.0))
    }
}

/// The steps a wall jump's `branch` goes on from (`WallJumpBranch`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Branch {
    /// The plain spin's step, `x`.
    pub spin: Step,
    /// The space jump's step, `y`.
    pub space_jump: Step,
    /// The screw attack's step, `z`.
    pub screw_attack: Step,
}

/// What a step of a clip does when the clip reaches it (§1.4): show its picture for a hold, or a
/// control, which the clip passes through in the frame it reaches it unless it rests there.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Instruction {
    /// `n`: shows its picture for its hold, 1 frame or more (`ClipPlays`).
    Show(Frames),
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

impl Instruction {
    /// `ClipEnds`: the end this instruction is, if it is one.
    #[must_use]
    pub const fn end(self) -> Option<ClipEnd> {
        match self {
            Self::End => Some(ClipEnd::End),
            Self::EndUnlessJump => Some(ClipEnd::EndUnlessJump),
            Self::Show(_) | Self::Rest | Self::Again(_) | Self::Breathe | Self::Branch(_) => None,
        }
    }
}

/// A clip (§1.4): its instructions from step 0, and the steps it marks.
///
/// §1.5 gives the clips the spec covers: an `again` or a `branch` goes to a shown step, a `rest`
/// is followed by a shown step, the last step is not a shown one, and a hold is at most 127
/// frames. Outside them the model stays total: a clip that would move on past its end leaves her
/// where she is, and a step with no hold gives no time left.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Clip {
    /// The instructions, one per step, in order from step 0.
    pub instructions: Vec<Instruction>,
    /// A spin's kick step (`WallJumpReady`, `SpinTurnedFar`).
    pub kick: Option<Step>,
    /// A fall's fast fall step (`FastFall`).
    pub fast_fall: Option<Step>,
    /// A straight jump's rising step (`RisingHold`).
    pub rising: Option<Step>,
}

impl Clip {
    /// The instruction at `step`, if the clip reaches that far.
    #[must_use]
    pub fn at(&self, step: Step) -> Option<Instruction> {
        self.instructions.get(step.index()).copied()
    }

    /// The hold of the shown step at `step`; none for a control.
    #[must_use]
    pub fn hold(&self, step: Step) -> Frames {
        if let Some(Instruction::Show(hold)) = self.at(step) {
            hold
        } else {
            Frames::NONE
        }
    }
}

/// One of the run's paces (§1.4): a hold for each shown step of her running clips, by step.
/// Where a pace gives no hold for a step (outside §1.4's content), the clip's own hold shows.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Pace(pub Vec<Frames>);

impl Pace {
    /// The pace's hold for `step`, if it gives one.
    #[must_use]
    pub fn hold(&self, step: Step) -> Option<Frames> {
        self.0.get(step.index()).copied()
    }
}

/// The run's paces (§1.4): the plain pace, and a boosted pace for each boost stage.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Paces {
    /// Her stride without the speed booster.
    pub plain: Pace,
    /// Her stride with it, at each stage.
    pub boosted: EnumMap<BoostStage, Pace>,
}
