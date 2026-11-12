//! §1.1, §6: what a caller reads as the frame starts, before samus-motion's step: the end her
//! clip reaches and whether her spin has turned far. Each follows from her playhead and the clip
//! of her pose at the frame's start alone.

use clean_types::ClipEnd;

use crate::clip::{Clip, Instruction};
use crate::playhead::{Playhead, RunDown};

/// What is known of a frame from its start alone.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ahead {
    /// The end her clip reaches this frame, if it reaches one (`ClipEnds`), which samus-motion's
    /// `AnimationEnds` reads before it picks her pose.
    pub end: Option<ClipEnd>,
    /// Her spin has turned far enough to kick off a wall (`SpinTurnedFar`), which samus-motion's
    /// `WallJump` reads that frame.
    pub turned_far: bool,
}

impl Ahead {
    /// What is known of the frame from `playhead`, as the caller kept it, and `clip`, the clip of
    /// her pose at the frame's start.
    #[must_use]
    pub fn of(playhead: Playhead, clip: &Clip) -> Self {
        Self {
            end: end(playhead, clip),
            turned_far: turned_far(playhead, clip),
        }
    }
}

/// Spec: `ClipEnds`. The end her clip reaches this frame: the end the time running down moves it
/// on to, if it moves on to one.
///
/// Nothing the frame does to her clip before the time runs down can change it, because no clip
/// with an end is one a move acts on (§1.5): it has no kick step (a readied wall jump), no fast
/// fall step and no rising step, and it is not her run's (a lap's end), her unaimed standing's
/// (a standing restart) or a kept row's of `StartingClip`. So the clip runs down from the frame's
/// start as it would have.
fn end(playhead: Playhead, clip: &Clip) -> Option<ClipEnd> {
    match playhead.run_down(clip) {
        RunDown::MovesOn(_, Instruction::Control(control)) => control.end(),
        RunDown::MovesOn(_, Instruction::Show(_)) | RunDown::Stays(_) => None,
    }
}

/// Spec: `SpinTurnedFar`. Her spin has turned far once her clip has a kick step and her step is
/// on it or past it, by step number, so the steps of a branch numbered after the kick step are
/// past it. The frame's moves and the pose she takes show from the next frame on.
fn turned_far(playhead: Playhead, clip: &Clip) -> bool {
    clip.kick.is_some_and(|kick| playhead.step >= kick)
}
