//! §6 A frame of her animation, in the order that matters.

use crate::clip::{Clip, Frames, Hold, Pace, Paces, Step};
use crate::inputs::{Content, Inputs, Report};
use crate::rules::Rules;
use crate::state::{Playhead, State};

/// One update, and what it reads: the frame's inputs (§1.3), the clip of her pose at the frame's
/// start and of the pose she ends it in, the run's paces and the tuning. Her moves, her clip's
/// playback, her run and the start of the pose she takes are each a step of it.
pub struct Update<'a> {
    /// The frame's inputs, each from the domain that owns it.
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

impl<'a> Update<'a> {
    /// The update of `inputs`, with its clips and paces from `content`.
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
            .or_else(|| self.clip.hold(step));
        Playhead {
            step,
            frames_left: hold
                .map_or(Frames::NONE, Hold::frames)
                .slowed(self.inputs.liquid.slowdown),
        }
    }
}

/// One frame of her animation: from the State the caller kept, the State to keep for the next
/// frame, and what the frame reports.
///
/// First, from the frame's start, the end her clip reaches (`ClipEnds`, which a caller reads
/// ahead of samus-motion's step by `Playhead::end_ahead`). Then, with the pose she takes: her
/// moves' changes (a dash's beginning among them), the rising hold, her clip playing from her
/// pose at the frame's start, the start of the clip of the pose she takes, a dash's loss, and
/// whether she has turned far, from the step and clip the frame ends on.
#[must_use]
pub fn update(
    state: State,
    inputs: &Inputs,
    content: &impl Content,
    rules: &Rules,
) -> (State, Report) {
    let this = Update::new(inputs, content, rules);
    let end = state.playhead.end_ahead(this.clip);
    let moved = State {
        playhead: this.her_moves(state.playhead),
        boost: this.dash_began().unwrap_or(state.boost),
    };
    let (played, sound) = this.play_clip(moved);
    let after = State {
        playhead: this.start().unwrap_or(played.playhead),
        boost: this.dash_lost().unwrap_or(played.boost),
    };
    let report = Report {
        end,
        turned_far: this.turned_far(after.playhead),
        sound,
    };
    (after, report)
}
