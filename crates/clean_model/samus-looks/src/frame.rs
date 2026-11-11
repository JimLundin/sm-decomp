//! The frame her animation acts in: what it reads, gathered once, for every rule that acts in it.

use crate::clip::{Clip, Pace, Paces, Step};
use crate::inputs::{Content, Inputs};
use crate::rules::Rules;
use crate::state::Playhead;

/// The frame her animation acts in: its inputs (§1.3), the clip of her pose at its start and of
/// the pose she ends it in, the run's paces and the tuning. Her moves, her clip's playback, her
/// run and the start of the pose she takes each read it.
pub struct Frame<'a> {
    /// What the frame reads, each from the domain that owns it.
    pub inputs: &'a Inputs,
    /// The clip of her pose at the frame's start: the one that plays this frame.
    pub clip: &'a Clip,
    /// The clip of the pose she ends the frame in.
    pub clip_after: &'a Clip,
    /// The run's paces.
    pub paces: &'a Paces,
    /// The tuning.
    pub rules: &'a Rules,
}

impl<'a> Frame<'a> {
    /// The frame that reads `inputs`, with its clips and paces from `content`.
    #[must_use]
    pub fn new(inputs: &'a Inputs, content: &'a impl Content, rules: &'a Rules) -> Self {
        Self {
            inputs,
            clip: content.clip(&inputs.pose),
            clip_after: content.clip(&inputs.pose_after),
            paces: content.paces(),
            rules,
        }
    }

    /// Spec: `ClipPlays`, `RunPace`. The clip goes on from `step`: its hold, or the pace's where
    /// `pace` gives one, plus the slowdown, is the time left.
    #[must_use]
    pub fn goes_on(&self, step: Step, pace: Option<&Pace>) -> Playhead {
        let hold = pace
            .and_then(|pace| pace.hold(step))
            .unwrap_or_else(|| self.clip.hold(step));
        Playhead {
            step,
            frames_left: hold.slowed(self.inputs.liquid.slowdown),
        }
    }
}
