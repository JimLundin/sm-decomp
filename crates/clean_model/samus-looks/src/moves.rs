//! §4 What her moves do to her clip, before the time runs down.

use clean_types::Parts;

use crate::clip::{Clip, Frames, Step};
use crate::inputs::{Inputs, Move};
use crate::playhead::Playhead;
use crate::rules::Rules;

/// §6, 1 and 2: her moves' changes to `clip`, the clip of her pose at the frame's start, in this
/// order: a standing restart, a readied wall jump and a fast fall, then the rising hold, before
/// the time runs down. Each that acts sets the playhead the next one reads.
#[must_use]
pub fn her_moves(playhead: Playhead, inputs: &Inputs, clip: &Clip, rules: &Rules) -> Playhead {
    let playhead = standing_restart(inputs, rules).unwrap_or(playhead);
    let playhead = wall_jump_ready(inputs, clip).unwrap_or(playhead);
    let playhead = fast_fall(playhead.step, inputs, clip, rules).unwrap_or(playhead);
    rising_hold(playhead, inputs, clip, rules).unwrap_or(playhead)
}

/// Spec: `StandingRestart`. Holding shoot while she stands unaimed starts her standing clip
/// over: step 0 with the restart's hold, which no liquid slows.
fn standing_restart(inputs: &Inputs, rules: &Rules) -> Option<Playhead> {
    inputs.moves[Move::StandingRestart].then_some(Playhead {
        step: Step::FIRST,
        frames_left: rules.restart_hold.frames(),
    })
}

/// Spec: `WallJumpReady`. A readied wall jump turns her spin straight to its kick step: the step
/// before it, on its last frame, so the clip moves on to the kick step this frame.
fn wall_jump_ready(inputs: &Inputs, clip: &Clip) -> Option<Playhead> {
    let step = clip.kick?.before()?;
    inputs.moves[Move::WallJumpReadied].then_some(Playhead {
        step,
        frames_left: Frames::LAST,
    })
}

/// Spec: `FastFall`. Moving fast either way, rising or falling, on a step before her clip's fast
/// fall step, she jumps to it with the fast fall's hold, which no liquid slows.
fn fast_fall(from: Step, inputs: &Inputs, clip: &Clip, rules: &Rules) -> Option<Playhead> {
    let step = clip.fast_fall.filter(|&step| from < step)?;
    let fast = inputs.vertical_speed.size() >= rules.fast_fall_speed;
    fast.then_some(Playhead {
        step,
        frames_left: rules.fast_fall_hold.frames(),
    })
}

/// Spec: `RisingHold`. A straight jump keeps its rising picture while she is not heading down:
/// on its rising step with 1 frame left, the time left is set to the rising hold, which no
/// liquid slows, so the clip stays on that step instead of moving on.
fn rising_hold(
    playhead: Playhead,
    inputs: &Inputs,
    clip: &Clip,
    rules: &Rules,
) -> Option<Playhead> {
    let holds = !inputs.heading_down
        && clip.rising == Some(playhead.step)
        && playhead.frames_left == Frames::LAST;
    holds.then_some(Playhead {
        frames_left: rules.rising_hold.frames(),
        ..playhead
    })
}
