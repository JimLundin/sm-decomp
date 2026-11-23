//! What her moves do to her clip, before the time runs down.

use types::Parts;

use crate::clip::{Clip, Frames, Step};
use crate::inputs::{Inputs, MoveEvent};
use crate::rules::Rules;
use crate::state::Playhead;

/// Spec: `StandingRestart`. Holding shoot while she stands unaimed starts her standing clip
/// over: step 0 with the restart's hold, which no liquid slows.
#[must_use]
pub fn standing_restart(playhead: Playhead, inputs: &Inputs, rules: &Rules) -> Playhead {
    if !inputs.moves[MoveEvent::StandingRestart] {
        return playhead;
    }
    Playhead {
        step: Step::FIRST,
        time_left: rules.restart_hold.frames(),
    }
}

/// Spec: `WallJumpReady`. A readied wall jump turns her spin straight to its kick step: the step
/// before it, on its last frame, so the clip moves on to the kick step this frame.
#[must_use]
pub fn wall_jump_ready(playhead: Playhead, inputs: &Inputs, clip: &Clip) -> Playhead {
    let before_kick = clip.kick.and_then(Step::before);
    match before_kick {
        Some(step) if inputs.moves[MoveEvent::WallJumpReadied] => Playhead {
            step,
            time_left: Frames::LAST,
        },
        Some(_) | None => playhead,
    }
}

/// Spec: `FastFall`. Moving fast either way, rising or falling, on a step before her clip's fast
/// fall step, she jumps to it with the fast fall's hold, which no liquid slows.
#[must_use]
pub fn fast_fall(playhead: Playhead, inputs: &Inputs, clip: &Clip, rules: &Rules) -> Playhead {
    let ahead = clip.fast_fall.filter(|&step| playhead.step < step);
    match ahead {
        Some(step) if inputs.vertical_speed.size() >= rules.fast_fall_speed => Playhead {
            step,
            time_left: rules.fast_fall_hold.frames(),
        },
        Some(_) | None => playhead,
    }
}

/// Spec: `RisingHold`. On the rising step with 1 frame left, while not heading down, the rising
/// hold, which no liquid slows, becomes the time left before it runs down. So at the game's hold
/// of 4 she stays on the rising picture 3 frames at a time for as long as she rises.
#[must_use]
pub fn rising_hold(playhead: Playhead, inputs: &Inputs, clip: &Clip, rules: &Rules) -> Playhead {
    let holds = !inputs.heading_down
        && clip.rising == Some(playhead.step)
        && playhead.time_left == Frames::LAST;
    if !holds {
        return playhead;
    }
    Playhead {
        time_left: rules.rising_hold.frames(),
        ..playhead
    }
}
