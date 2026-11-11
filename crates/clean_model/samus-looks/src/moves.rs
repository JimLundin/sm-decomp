//! §4 What her moves do to her clip, before the time runs down, and whether her spin has turned
//! far enough to kick off a wall.

use clean_types::Parts;

use crate::clip::{Frames, Step};
use crate::inputs::Move;
use crate::state::Playhead;
use crate::update::Update;

/// The time left on a step that the time running down this frame moves on from.
const LAST_FRAME: Frames = Frames(1);

impl Update<'_> {
    /// §6, 1 and 2: her moves' changes to her clip, a standing restart, a readied wall jump and a
    /// fast fall, then the rising hold, in that order, before the time runs down. Each that acts
    /// sets the playhead the next one reads.
    #[must_use]
    pub fn her_moves(&self, playhead: Playhead) -> Playhead {
        let playhead = self.standing_restart().unwrap_or(playhead);
        let playhead = self.wall_jump_ready().unwrap_or(playhead);
        let playhead = self.fast_fall(playhead.step).unwrap_or(playhead);
        self.rising_hold(playhead).unwrap_or(playhead)
    }

    /// Spec: `StandingRestart`. Holding shoot while she stands unaimed starts her standing clip
    /// over: step 0 with the restart's hold, which no liquid slows.
    fn standing_restart(&self) -> Option<Playhead> {
        self.inputs.moves[Move::StandingRestart].then_some(Playhead {
            step: Step::FIRST,
            frames_left: self.rules.restart_hold,
        })
    }

    /// Spec: `WallJumpReady`. A readied wall jump turns her spin straight to its kick step: the
    /// step before it, on its last frame, so the clip moves on to the kick step this frame.
    fn wall_jump_ready(&self) -> Option<Playhead> {
        let step = self.clip.kick?.before()?;
        self.inputs.moves[Move::WallJumpReadied].then_some(Playhead {
            step,
            frames_left: LAST_FRAME,
        })
    }

    /// Spec: `FastFall`. Moving fast either way, rising or falling, on a step before her clip's
    /// fast fall step, she jumps to it with the fast fall's hold, which no liquid slows.
    fn fast_fall(&self, from: Step) -> Option<Playhead> {
        let step = self.clip.fast_fall.filter(|&step| from < step)?;
        let fast = self.inputs.vertical_speed.size() >= self.rules.fast_fall_speed;
        fast.then_some(Playhead {
            step,
            frames_left: self.rules.fast_fall_hold,
        })
    }

    /// Spec: `RisingHold`. A straight jump keeps its rising picture while she is not heading
    /// down: on its rising step's last frame, the time left is set to the rising hold, which no
    /// liquid slows, so the clip stays on that step instead of moving on.
    fn rising_hold(&self, playhead: Playhead) -> Option<Playhead> {
        let holds = !self.inputs.heading_down
            && self.clip.rising == Some(playhead.step)
            && playhead.frames_left == LAST_FRAME;
        holds.then_some(Playhead {
            frames_left: self.rules.rising_hold,
            ..playhead
        })
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
