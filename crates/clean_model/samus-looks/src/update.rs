//! §6 A frame of her animation, in the order that matters.

use clean_types::AnimationSound;

use crate::inputs::{Content, Inputs};
use crate::moves::her_moves;
use crate::playback::play_clip;
use crate::rules::Rules;
use crate::run::{dash_began, dash_lost};
use crate::starting::start;
use crate::state::State;

/// One frame of her animation, after samus-motion's step: the State to keep, and the sound asked.
///
/// From the State the caller kept, it gives the State to keep for the next frame and the sound
/// asked for this frame, at most one: only a control asks for one (`WallJumpBranch`'s branch or
/// `BoostStages`' lap end), and a frame passes one control at most.
///
/// What the frame's start alone tells, the end and whether her spin has turned far, the caller
/// reads before samus-motion's step (`Ahead::of`). Then, with the pose she takes, in §6's order:
/// her moves' changes (a dash's beginning among them), the rising hold, her clip playing from
/// her pose at the frame's start, the start of the clip of the pose she takes, and a dash's
/// loss.
#[must_use]
pub fn update(
    state: State,
    inputs: &Inputs,
    content: &impl Content,
    rules: &Rules,
) -> (State, Option<AnimationSound>) {
    let clip = content.clip(&inputs.pose);
    let moved = State {
        playhead: her_moves(state.playhead, inputs, clip, rules),
        boost: dash_began(inputs, rules).unwrap_or(state.boost),
    };
    let (played, sound) = play_clip(moved, inputs, clip, content.paces(), rules);
    let after = State {
        playhead: start(inputs, content.clip(&inputs.pose_after)).unwrap_or(played.playhead),
        boost: dash_lost(inputs).unwrap_or(played.boost),
    };
    (after, sound)
}
