//! §6 A frame of her animation, in the order that matters.

use crate::inputs::{Content, Inputs, Move, Report};
use crate::moves;
use crate::player::Player;
use crate::rules::Rules;
use crate::run;
use crate::starting;
use crate::state::{Boost, State};

/// One frame: the State after it, and what it reports.
///
/// First, from the frame's start, the end her clip reaches (`ClipEnds`, which a caller may also
/// read ahead of samus-motion's step by `Playhead::end_ahead`). Then, with the pose she takes:
/// her moves' changes, the rising hold, the time running down from her pose at the frame's
/// start, the start of the clip of the pose she takes, a dash's loss, and whether she has turned
/// far, from the step and clip the frame ends on.
#[must_use]
pub fn step(
    state: State,
    inputs: &Inputs,
    content: &impl Content,
    rules: &Rules,
) -> (State, Report) {
    let clip = content.clip(&inputs.pose);
    let end = state.playhead.end_ahead(clip);

    let boost = if inputs.moves[Move::DashBegan] {
        run::dash_began(&inputs.items, rules)
    } else {
        state.boost
    };
    let playhead = moves::apply(state.playhead, clip, inputs, rules);
    let (ran_down, sound) = Player {
        clip,
        inputs,
        paces: content.paces(),
        rules,
    }
    .run_down(State { playhead, boost });
    let playhead = starting::start(ran_down.playhead, inputs, content);
    let boost = if inputs.moves[Move::DashLost] {
        // Spec: `BoostStages`. A dash lost: stage 0, with 0 laps left.
        Boost::default()
    } else {
        ran_down.boost
    };
    let turned_far = moves::turned_far(playhead, content.clip(&inputs.pose_after));

    let report = Report {
        end,
        turned_far,
        sound,
    };
    (State { playhead, boost }, report)
}
