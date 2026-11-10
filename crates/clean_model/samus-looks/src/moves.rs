//! §4 What her moves do to her clip, before the time runs down, and whether her spin has turned
//! far enough to kick off a wall.

use crate::clip::{Clip, Frames, Step};
use crate::inputs::{Inputs, Move, Vertical};
use crate::rules::Rules;
use crate::state::Playhead;

/// §6, 1 and 2: her moves' changes to her clip, then the rising hold, in that order.
#[must_use]
pub fn apply(playhead: Playhead, clip: &Clip, inputs: &Inputs, rules: &Rules) -> Playhead {
    let playhead = standing_restart(playhead, inputs, rules);
    let playhead = wall_jump_ready(playhead, clip, inputs);
    let playhead = fast_fall(playhead, clip, inputs.vertical, rules);
    rising_hold(playhead, clip, inputs.vertical, rules)
}

/// Spec: `StandingRestart`. Holding shoot while she stands unaimed starts her standing clip
/// over: step 0 with the restart's hold, which no liquid slows.
fn standing_restart(playhead: Playhead, inputs: &Inputs, rules: &Rules) -> Playhead {
    if inputs.moves[Move::StandingRestart] {
        Playhead {
            step: Step::FIRST,
            left: rules.restart_hold,
        }
    } else {
        playhead
    }
}

/// The frame left on a step that the time running down this frame moves on from.
const LAST_FRAME: Frames = Frames(1);

/// Spec: `WallJumpReady`. A readied wall jump turns her spin straight to its kick step: the
/// step before it, with 1 frame left, so the clip moves on to it this frame.
fn wall_jump_ready(playhead: Playhead, clip: &Clip, inputs: &Inputs) -> Playhead {
    let before_kick = clip.kick.and_then(Step::before);
    match before_kick {
        Some(step) if inputs.moves[Move::WallJumpReadied] => Playhead {
            step,
            left: LAST_FRAME,
        },
        Some(_) | None => playhead,
    }
}

/// Spec: `FastFall`. Falling fast, before her clip's fast fall step, she jumps to it with its
/// hold, which no liquid slows.
fn fast_fall(playhead: Playhead, clip: &Clip, vertical: Vertical, rules: &Rules) -> Playhead {
    let fast = matches!(vertical, Vertical::Down(speed) if speed >= rules.fast_fall_speed);
    match clip.fast_fall {
        Some(step) if fast && playhead.step < step => Playhead {
            step,
            left: rules.fast_fall_hold,
        },
        Some(_) | None => playhead,
    }
}

/// Spec: `RisingHold`. A straight jump on its rising step with 1 frame left, not heading down,
/// holds it again for the rising hold, which no liquid slows.
fn rising_hold(playhead: Playhead, clip: &Clip, vertical: Vertical, rules: &Rules) -> Playhead {
    let holds = vertical == Vertical::NotDown
        && clip.rising == Some(playhead.step)
        && playhead.left == LAST_FRAME;
    if holds {
        Playhead {
            left: rules.rising_hold,
            ..playhead
        }
    } else {
        playhead
    }
}

/// Spec: `SpinTurnedFar`. Her spin has turned far enough to kick off a wall once the clip she
/// ends the frame on has a kick step and her step is at it or past it.
#[must_use]
pub fn turned_far(playhead: Playhead, clip: &Clip) -> bool {
    clip.kick.is_some_and(|kick| playhead.step >= kick)
}
