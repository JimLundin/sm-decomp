//! A frame of her animation, in the order that matters.

use types::AnimationSound;

use crate::inputs::{Content, Inputs};
use crate::moves::her_moves;
use crate::play::play;
use crate::rules::Rules;
use crate::run::{dash_began, dash_lost};
use crate::starting::start;
use crate::state::State;

/// One frame of her animation, after samus-movement's step: the State to keep, and the sound asked.
///
/// From the State the caller kept, it gives the State to keep for the next frame and the sound
/// asked for this frame, at most one: only a control asks for one (`WallJumpBranch`'s branch or
/// `BoostStages`' lap end), and a frame passes one control at most.
///
/// What the frame's start alone tells, the end and whether her spin has turned far, the caller
/// reads before samus-movement's step (`Ahead::of`). Then, with the pose she takes, in this order:
/// her moves' changes (a dash's beginning among them), the rising hold, her clip playing from
/// her figure at the frame's start, the start of the clip of the pose she takes, and a dash's
/// loss.
#[must_use]
pub fn update(
    state: State,
    inputs: &Inputs,
    content: &impl Content,
    rules: &Rules,
) -> (State, Option<AnimationSound>) {
    let moved = State {
        playhead: her_moves(state.playhead, inputs, content.clip(&inputs.figure), rules),
        ..dash_began(state, inputs, rules)
    };
    let (played, sound) = play(moved, inputs, content, rules);
    let started = State {
        playhead: start(inputs, content.clip(&inputs.figure_after)).unwrap_or(played.playhead),
        ..played
    };
    (dash_lost(started, inputs), sound)
}
