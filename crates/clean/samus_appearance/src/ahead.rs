//! The read-ahead: what a caller asks of her State as the frame starts, before samus-movement's
//! step: the end her clip reaches and whether her spin has turned far.

use types::ClipEnd;

use crate::clip::{Clip, Instruction};
use crate::state::{AppearanceState, RunDown};

/// What is known of a frame from its start alone: the read-ahead.
///
/// It is one query of her State before the frame, not a part of the frame: it reads the State
/// as the frame starts and the clip of her pose then, and nothing else, and it changes nothing.
/// A frame with her movement gives the same answer, so the frame doesn't tell it again. A
/// caller asks it before the frame, as samus-movement does before its step, since the frame
/// needs her pose at its end, which that step picks.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Ahead {
    /// The end her clip reaches this frame, if it reaches one (`ClipEnds`), which samus-movement's
    /// `AnimationEnds` reads before it picks her pose.
    pub end: Option<ClipEnd>,
    /// Her spin has turned far enough to kick off a wall (`SpinTurnedFar`), which samus-movement's
    /// `WallJump` reads that frame.
    pub turned_far: bool,
}

impl Ahead {
    /// What is known of the frame from `state`, her appearance State as the caller kept it, and
    /// `clip`, the clip of her figure at the frame's start.
    ///
    /// Spec: `ClipEnds`. The end is the one the time running down moves her clip on to, if it
    /// moves on to one. No move of the frame can change it: the spec's clips that end have no
    /// kick, fast fall or rising step, and belong neither to her run, nor to her standing
    /// unaimed, nor to a kept row of `StartingClip`.
    ///
    /// Spec: `SpinTurnedFar`. Her spin has turned far once her clip has a kick step and her step
    /// is on it or past it by step number, so the steps of a branch numbered after the kick step
    /// are past it. What the frame's moves and the pose she takes do shows from the next frame.
    #[must_use]
    pub fn of(state: &AppearanceState, clip: &Clip) -> Self {
        let playhead = state.animation.playhead;
        let end = match playhead.run_down(clip) {
            RunDown::MovesOn(_, Instruction::Control(control)) => control.end(),
            RunDown::MovesOn(_, Instruction::Show(_)) | RunDown::Stays(_) => None,
        };
        Self {
            end,
            turned_far: clip.kick.is_some_and(|kick| playhead.step >= kick),
        }
    }
}
