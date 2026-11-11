//! §1 What the domain remembers from one frame to the next: where her clip is, and her run's
//! boost. A caller may build each part at any value the spec allows and read it back, and keeps
//! it between frames as `update` left it: nothing else carries over.

use clean_types::{BoostStage, ClipEnd};

use crate::clip::{Clip, Frames, Instruction, Step};

/// Where her clip is: her step, and the time it has left.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Playhead {
    /// Her step: the picture she shows (part 2).
    pub step: Step,
    /// The frames left: how long her step still shows, 0 to 255.
    pub frames_left: Frames,
}

/// What the time running down does to her clip this frame (`ClipPlays`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RunDown {
    /// She stays on her step, with this playhead: time is left on it, or her clip has no step
    /// after it (outside the spec, §1.5).
    Stays(Playhead),
    /// The clip moves on to this step, which holds this instruction.
    MovesOn(Step, Instruction),
}

impl Playhead {
    /// Spec: `ClipPlays`. The time left runs down by 1; above 0 she stays on her step, and at 0
    /// or below the clip moves on to the step after it. Both the end reported at the frame's
    /// start and the clip's playback read this one answer.
    #[must_use]
    pub fn run_down(self, clip: &Clip) -> RunDown {
        if let Some(left) = self.frames_left.0.checked_sub(1).filter(|&left| left > 0) {
            return RunDown::Stays(Self {
                frames_left: Frames(left),
                ..self
            });
        }
        let next = self.step.next().and_then(|to| Some((to, clip.at(to)?)));
        next.map_or(RunDown::Stays(self), |(to, instruction)| {
            RunDown::MovesOn(to, instruction)
        })
    }

    /// Spec: `ClipEnds`. The end her clip reaches this frame, from this playhead and her clip at
    /// the frame's start, for a caller to hand samus-motion before it picks her pose.
    ///
    /// Nothing the frame does to her clip before the time runs down can change it, because no
    /// clip with an end is one a move acts on (§1.5): it has no kick step (a readied wall jump),
    /// no fast fall step and no rising step, and it is not her run's (a lap's end), her unaimed
    /// standing's (a standing restart) or a kept row's of `StartingClip`. So the clip runs down
    /// from the frame's start as it would have.
    #[must_use]
    pub fn end_ahead(self, clip: &Clip) -> Option<ClipEnd> {
        match self.run_down(clip) {
            RunDown::MovesOn(_, Instruction::Control(control)) => control.end(),
            RunDown::MovesOn(_, Instruction::Show(_)) | RunDown::Stays(_) => None,
        }
    }
}

/// A count of laps of her run: the laps a stage lasts, or the laps it still lasts.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Laps(pub u8);

impl Laps {
    /// `BoostStages`: the laps left less 1, while that stays above 0; none when the stage has
    /// lasted its laps.
    #[must_use]
    pub fn counted_down(self) -> Option<Self> {
        self.0.checked_sub(1).filter(|&laps| laps > 0).map(Self)
    }
}

/// Her run's boost: its stage and the laps the stage still lasts (`BoostStages`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Boost {
    /// The boost stage, 0 to 4; 4 is boosting.
    pub stage: BoostStage,
    /// The laps left of the stage: 1 or 2 with the speed booster, 0 without it (§1.5).
    pub laps_left: Laps,
}

/// The State: everything the domain remembers from one frame to the next (§1).
///
/// Her step, the time left, the boost stage and the laps left. The end, where the clip of the
/// pose she takes starts, the slowdowns and what her moves did are each worked out within the
/// frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct State {
    /// Where her clip is.
    pub playhead: Playhead,
    /// Her run's boost.
    pub boost: Boost,
}
