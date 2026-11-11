//! §6 A frame of her animation, in the order that matters.

use crate::frame::Frame;
use crate::inputs::{Content, Inputs, Report};
use crate::rules::Rules;
use crate::state::State;

/// One frame of her animation: the State after it, and what it reports.
///
/// First, from the frame's start, the end her clip reaches (`ClipEnds`, which a caller reads
/// ahead of samus-motion's step by `Playhead::end_ahead`). Then, with the pose she takes: her
/// moves' changes, the rising hold, her clip playing from her pose at the frame's start, the
/// start of the clip of the pose she takes, a dash's loss, and whether she has turned far, from
/// the step and clip the frame ends on.
#[must_use]
pub fn update(
    state: State,
    inputs: &Inputs,
    content: &impl Content,
    rules: &Rules,
) -> (State, Report) {
    let frame = Frame::new(inputs, content, rules);
    let end = state.playhead.end_ahead(frame.clip);
    let moved = State {
        playhead: frame.her_moves(state.playhead),
        boost: frame.dash_began(state.boost),
    };
    let (played, sound) = frame.play_clip(moved);
    let after = State {
        playhead: frame.start(played.playhead),
        boost: frame.dash_lost(played.boost),
    };
    let report = Report {
        end,
        turned_far: frame.turned_far(after.playhead),
        sound,
    };
    (after, report)
}
