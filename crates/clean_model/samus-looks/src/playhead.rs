//! Where her clip is, and how the time running down moves it on (`ClipPlays`).

use crate::clip::{Clip, Frames, Hold, Instruction, Pace, Slowdown, Step};

/// Where her clip is: her step, and the time it has left.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Playhead {
    /// Her step: the picture she shows (part 2).
    pub step: Step,
    /// The time left: how long her step still shows, 0 to 255 frames.
    pub frames_left: Frames,
}

/// What the time running down does to her clip this frame (`ClipPlays`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RunDown {
    /// She stays on her step, with this playhead: time is left on it, or her clip has no step
    /// after it (outside the spec, §1.5).
    Stays(Playhead),
    /// The clip moves on to this step, which holds this instruction.
    MovesOn(Step, Instruction),
}

impl Playhead {
    /// Spec: `ClipPlays`, `RunPace`. Her clip going on from `step`, its hold as the time left:
    /// `pace`'s hold for the step where it gives one, else the clip's own. A control has no
    /// hold, so no time is left on it. Every hold the clip moves on to is read here.
    #[must_use]
    pub fn on(step: Step, clip: &Clip, pace: Option<&Pace>) -> Self {
        let hold = pace
            .and_then(|pace| pace.hold(step))
            .or_else(|| clip.hold(step));
        Self {
            step,
            frames_left: hold.map_or(Frames::NONE, Hold::frames),
        }
    }

    /// The same step, its time left longer by `slowdown`.
    #[must_use]
    pub const fn slowed(self, slowdown: Slowdown) -> Self {
        Self {
            step: self.step,
            frames_left: self.frames_left.slowed(slowdown),
        }
    }

    /// Spec: `ClipPlays`. The time left runs down by a frame; while some is left she stays on
    /// her step, and once it has run out the clip moves on to the step after it. The end a
    /// caller reads ahead and the clip's playback read this one answer.
    #[must_use]
    pub fn run_down(self, clip: &Clip) -> RunDown {
        if let Some(frames_left) = self.frames_left.counted_down() {
            return RunDown::Stays(Self {
                frames_left,
                ..self
            });
        }
        let next = self.step.next().and_then(|to| Some((to, clip.at(to)?)));
        next.map_or(RunDown::Stays(self), |(to, instruction)| {
            RunDown::MovesOn(to, instruction)
        })
    }
}
