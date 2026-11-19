//! What her moves do to her clip, before the time runs down.

use types::Parts;

use crate::clip::{Clip, Frames, Step};
use crate::inputs::{Inputs, MoveEvent};
use crate::rules::Rules;
use crate::state::Playhead;

/// Spec: `StandingRestart`. Holding shoot while she stands unaimed starts her standing clip
/// over: step 0 with the restart's hold, which no liquid slows.
pub fn standing_restart(inputs: &Inputs, rules: &Rules) -> Option<Playhead> {
    inputs.moves[MoveEvent::StandingRestart].then_some(Playhead {
        step: Step::FIRST,
        time_left: rules.restart_hold.frames(),
    })
}

/// Spec: `WallJumpReady`. A readied wall jump turns her spin straight to its kick step: the step
/// before it, on its last frame, so the clip moves on to the kick step this frame.
pub fn wall_jump_ready(inputs: &Inputs, clip: &Clip) -> Option<Playhead> {
    let step = clip.kick?.before()?;
    inputs.moves[MoveEvent::WallJumpReadied].then_some(Playhead {
        step,
        time_left: Frames::LAST,
    })
}

/// Spec: `FastFall`. Moving fast either way, rising or falling, on a step before her clip's fast
/// fall step, she jumps to it with the fast fall's hold, which no liquid slows.
pub fn fast_fall(from: Step, inputs: &Inputs, clip: &Clip, rules: &Rules) -> Option<Playhead> {
    let step = clip.fast_fall.filter(|&step| from < step)?;
    let fast = inputs.vertical_speed.size() >= rules.fast_fall_speed;
    fast.then_some(Playhead {
        step,
        time_left: rules.fast_fall_hold.frames(),
    })
}

/// Spec: `RisingHold`. On the rising step with 1 frame left, while not heading down,
/// install the configured hold, which no liquid slows, before ordinary countdown.
/// With hold one the following instruction can be reached immediately; vanilla hold four
/// instead renews a three-frame hold on the rising picture.
pub fn rising_hold(
    playhead: Playhead,
    inputs: &Inputs,
    clip: &Clip,
    rules: &Rules,
) -> Option<Playhead> {
    let holds = !inputs.heading_down
        && clip.rising == Some(playhead.step)
        && playhead.time_left == Frames::LAST;
    holds.then_some(Playhead {
        time_left: rules.rising_hold.frames(),
        ..playhead
    })
}
