//! §1.4 Her clips: the content the importer supplies, a list of steps per pose, written in the
//! spec's own cells.

use core::str::FromStr;

/// A step of a clip, numbered from 0 (§1.4); her step is the one her clip is on.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct Step(pub u8);

impl Step {
    /// The first step of every clip.
    pub const FIRST: Self = Self(0);
    /// The step after the first.
    pub const SECOND: Self = Self(1);

    /// The step after this one, where the clip moves on to.
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

    /// This count lengthened by `more`, kept in the time left's range (0 to 255) by wrapping.
    #[must_use]
    pub const fn plus(self, more: Self) -> Self {
        Self(self.0.wrapping_add(more.0))
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

/// One step of a clip (§1.4): a shown step with its hold, or a control.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClipStep {
    /// `n`: shows its picture for its hold, 1 frame or more (`ClipPlays`).
    Show(Frames),
    /// `rest`: she rests on it for one frame with no time left (`ClipControls`).
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

/// A cell that is not one of §1.4's written steps.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UnwrittenStep(pub String);

impl FromStr for ClipStep {
    type Err = UnwrittenStep;

    /// §1.4's "Written" column: `n` (1 or more), `rest`, `again s`, `breathe`, `end`,
    /// `end-unless-jump`, `branch x y z`.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let words: Vec<&str> = text.split_whitespace().collect();
        written(&words).ok_or_else(|| UnwrittenStep(text.to_owned()))
    }
}

/// §1.4: the step a cell's words write, if they write one.
fn written(words: &[&str]) -> Option<ClipStep> {
    match words {
        ["rest"] => Some(ClipStep::Rest),
        ["breathe"] => Some(ClipStep::Breathe),
        ["end"] => Some(ClipStep::End),
        ["end-unless-jump"] => Some(ClipStep::EndUnlessJump),
        ["again", to] => written_step(to).map(ClipStep::Again),
        ["branch", spin, space_jump, screw_attack] => Some(ClipStep::Branch(Branch {
            spin: written_step(spin)?,
            space_jump: written_step(space_jump)?,
            screw_attack: written_step(screw_attack)?,
        })),
        [hold] => hold
            .parse()
            .ok()
            .filter(|&frames| frames > 0)
            .map(|frames| ClipStep::Show(Frames(frames))),
        _ => None,
    }
}

/// §1.4: a step's number, written in a cell.
fn written_step(text: &str) -> Option<Step> {
    text.parse().ok().map(Step)
}

impl ClipStep {
    /// `ClipEnds`: the end this step is, if it is one.
    #[must_use]
    pub const fn end(self) -> Option<clean_types::ClipEnd> {
        match self {
            Self::End => Some(clean_types::ClipEnd::End),
            Self::EndUnlessJump => Some(clean_types::ClipEnd::EndUnlessJump),
            Self::Show(_) | Self::Rest | Self::Again(_) | Self::Breathe | Self::Branch(_) => None,
        }
    }
}

/// A clip (§1.4): its steps from step 0, and the steps it marks.
///
/// §1.5 gives the clips the spec covers: an `again` or a `branch` goes to a shown step, a `rest`
/// is followed by a shown step, and the last step is not a shown one. Outside them the model
/// stays total: a step past the clip's end leaves her where she is, and a step with no hold
/// gives no time left.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Clip {
    /// The steps, in order from step 0.
    pub steps: Vec<ClipStep>,
    /// A spin's kick step (`WallJumpReady`, `SpinTurnedFar`).
    pub kick: Option<Step>,
    /// A fall's fast fall step (`FastFall`).
    pub fast_fall: Option<Step>,
    /// A straight jump's rising step (`RisingHold`).
    pub rising: Option<Step>,
}

impl Clip {
    /// The step of this clip at `step`, if the clip has one there.
    #[must_use]
    pub fn at(&self, step: Step) -> Option<ClipStep> {
        self.steps.get(step.index()).copied()
    }

    /// The step the clip moves on to from `step`, and what is there: none past the clip's end.
    #[must_use]
    pub fn after(&self, step: Step) -> Option<(Step, ClipStep)> {
        let next = step.next()?;
        Some((next, self.at(next)?))
    }

    /// The hold of the shown step at `step`; none for a control.
    #[must_use]
    pub fn hold(&self, step: Step) -> Frames {
        if let Some(ClipStep::Show(hold)) = self.at(step) {
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
    pub boosted: enum_map::EnumMap<clean_types::BoostStage, Pace>,
}
