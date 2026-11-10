//! §2 Her clip plays: the time runs down, and the clip moves on through its steps.

use clean_types::{AnimationSound, Item};

use crate::clip::{Branch, Clip, ClipStep, Frames, Pace, Paces, Step};
use crate::inputs::Inputs;
use crate::rules::Rules;
use crate::run::{self, LapEnd};
use crate::state::{Boost, Playhead, State};

/// Her clip as it plays this frame: the clip of her pose at the frame's start, with what the
/// frame gives it.
pub struct Player<'a> {
    /// The clip of her pose at the frame's start.
    pub clip: &'a Clip,
    /// The frame's inputs.
    pub inputs: &'a Inputs,
    /// The run's paces.
    pub paces: &'a Paces,
    /// The tuning.
    pub rules: &'a Rules,
}

impl Player<'_> {
    /// Spec: `ClipPlays`. The time runs down by 1; when it runs out the clip moves on to the
    /// next step: a shown step shows for its hold, or her stride's (`RunPace`), plus the
    /// slowdown; a control does what it says, unless it ends a lap of her run (`RunPace`,
    /// `BoostStages`). Returns the State and the sound asked for.
    #[must_use]
    pub fn run_down(&self, state: State) -> (State, Option<AnimationSound>) {
        let with = |playhead| State { playhead, ..state };
        if let Some(counted) = state.playhead.counted_down() {
            return (with(counted), None);
        }
        let Some((next, step)) = self.clip.after(state.playhead.step) else {
            return (state, None);
        };
        let stride = run::stride(self.inputs, state.boost, self.paces);
        if let ClipStep::Show(_) = step {
            return (with(self.slowed(next, self.hold(next, stride))), None);
        }
        if stride.is_some() && self.inputs.run_held {
            return self.lap_ends(next, step, state.boost);
        }
        let (playhead, sound) = self.control(next, step);
        (with(playhead), sound)
    }

    /// Spec: `RunPace`, `BoostStages`. A lap of her run ends at the control `step`, at `at`.
    fn lap_ends(&self, at: Step, step: ClipStep, boost: Boost) -> (State, Option<AnimationSound>) {
        match run::lap_ends(boost, self.inputs, self.paces, self.rules) {
            LapEnd::AsClipSays(boost) => {
                let (playhead, sound) = self.control(at, step);
                (State { playhead, boost }, sound)
            }
            LapEnd::StartsOver { boost, pace, sound } => {
                let playhead = self.slowed(Step::FIRST, self.hold(Step::FIRST, Some(pace)));
                (State { playhead, boost }, sound)
            }
        }
    }

    /// Spec: `RunPace`. The hold of `step`: the pace's, where her stride takes one that gives
    /// it, else the clip's own.
    fn hold(&self, step: Step, pace: Option<&Pace>) -> Frames {
        pace.and_then(|pace| pace.hold(step))
            .unwrap_or_else(|| self.clip.hold(step))
    }

    /// Spec: `ClipControls`, `Breathing`, `ClipEnds`, `WallJumpBranch`. The clip moving on to
    /// `step`, at `at`, does what it says.
    fn control(&self, at: Step, step: ClipStep) -> (Playhead, Option<AnimationSound>) {
        match step {
            ClipStep::Show(_) => (self.goes_on(at), None),
            ClipStep::Again(to) => (self.goes_on(to), None),
            ClipStep::Breathe => (self.goes_on(self.breath(at)), None),
            ClipStep::Branch(branch) => {
                let (to, sound) = self.wall_jump(branch);
                (self.goes_on(to), Some(sound))
            }
            // A rest holds her one frame; an end hands her to the pose she takes this frame
            // (§1.5), and rests the same way should none follow.
            ClipStep::Rest | ClipStep::End | ClipStep::EndUnlessJump => (
                Playhead {
                    step: at,
                    left: Frames::NONE,
                },
                None,
            ),
        }
    }

    /// Spec: `Breathing`. From a `breathe` at `at`: back to step 0 while her energy is at the
    /// threshold or more, else on to the next step.
    fn breath(&self, at: Step) -> Step {
        if self.inputs.energy >= self.rules.breathing_threshold {
            Step::FIRST
        } else {
            at.next().unwrap_or(at)
        }
    }

    /// Spec: `WallJumpBranch`. The spin she wall jumps in, and its sound: plainly when
    /// submerged, else as the screw attack with it, else as the space jump with it, else
    /// plainly.
    fn wall_jump(&self, branch: Branch) -> (Step, AnimationSound) {
        let items = &self.inputs.items;
        if self.inputs.liquid.submerged {
            (branch.spin, AnimationSound::PlainSpin)
        } else if items[Item::ScrewAttack] {
            (branch.screw_attack, AnimationSound::ScrewAttack)
        } else if items[Item::SpaceJump] {
            (branch.space_jump, AnimationSound::SpaceJump)
        } else {
            (branch.spin, AnimationSound::PlainSpin)
        }
    }

    /// Spec: `ClipPlays`, `ClipControls`. The clip goes on from `step` with its hold plus the
    /// slowdown.
    fn goes_on(&self, step: Step) -> Playhead {
        self.slowed(step, self.clip.hold(step))
    }

    /// Spec: `ClipPlays`. On `step` with `hold` plus the slowdown as the time left.
    const fn slowed(&self, step: Step, hold: Frames) -> Playhead {
        Playhead {
            step,
            left: hold.plus(self.inputs.liquid.slowdown),
        }
    }
}
