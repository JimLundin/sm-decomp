//! §4 What her moves do to her clip, before the time runs down, and whether her spin has turned
//! far enough to kick off a wall.

use crate::clip::{Frames, Step};
use crate::frame::Frame;
use crate::inputs::{Move, Vertical};
use crate::state::Playhead;

/// The time left on a step that the time running down this frame moves on from.
const LAST_FRAME: Frames = Frames(1);

impl Frame<'_> {
    /// §6, 1 and 2: her moves' changes to her clip, a standing restart, a readied wall jump and a
    /// fast fall, then the rising hold, in that order, before the time runs down.
    #[must_use]
    pub fn her_moves(&self, playhead: Playhead) -> Playhead {
        let playhead = self.standing_restart(playhead);
        let playhead = self.wall_jump_ready(playhead);
        let playhead = self.fast_fall(playhead);
        self.rising_hold(playhead)
    }

    /// Spec: `StandingRestart`. Holding shoot while she stands unaimed starts her standing clip
    /// over: step 0 with the restart's hold, which no liquid slows.
    fn standing_restart(&self, playhead: Playhead) -> Playhead {
        if self.inputs.moves[Move::StandingRestart] {
            Playhead {
                step: Step::FIRST,
                frames_left: self.rules.restart_hold,
            }
        } else {
            playhead
        }
    }

    /// Spec: `WallJumpReady`. A readied wall jump turns her spin straight to its kick step: the
    /// step before it, on its last frame, so the clip moves on to the kick step this frame.
    fn wall_jump_ready(&self, playhead: Playhead) -> Playhead {
        let before_kick = self.clip.kick.and_then(Step::before);
        match before_kick {
            Some(step) if self.inputs.moves[Move::WallJumpReadied] => Playhead {
                step,
                frames_left: LAST_FRAME,
            },
            Some(_) | None => playhead,
        }
    }

    /// Spec: `FastFall`. Falling fast, before her clip's fast fall step, she jumps to it with the
    /// fast fall's hold, which no liquid slows.
    fn fast_fall(&self, playhead: Playhead) -> Playhead {
        let rules = self.rules;
        let fast =
            matches!(self.inputs.vertical, Vertical::Down(speed) if speed >= rules.fast_fall_speed);
        match self.clip.fast_fall {
            Some(step) if fast && playhead.step < step => Playhead {
                step,
                frames_left: rules.fast_fall_hold,
            },
            Some(_) | None => playhead,
        }
    }

    /// Spec: `RisingHold`. A straight jump keeps its rising picture while she is not heading
    /// down: on its rising step's last frame, the time left is set to the rising hold, which no
    /// liquid slows, so the clip stays on that step instead of moving on.
    fn rising_hold(&self, playhead: Playhead) -> Playhead {
        let keeps = self.inputs.vertical == Vertical::NotDown
            && self.clip.rising == Some(playhead.step)
            && playhead.frames_left == LAST_FRAME;
        if keeps {
            Playhead {
                frames_left: self.rules.rising_hold,
                ..playhead
            }
        } else {
            playhead
        }
    }

    /// Spec: `SpinTurnedFar`. Her spin has turned far enough to kick off a wall once the clip she
    /// ends the frame on has a kick step and her step is at it or past it.
    #[must_use]
    pub fn turned_far(&self, playhead: Playhead) -> bool {
        self.clip_after
            .kick
            .is_some_and(|kick| playhead.step >= kick)
    }
}
