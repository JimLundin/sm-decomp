//! §2 Her clip plays: the time runs down, and the clip moves on through its instructions.

use clean_types::{AnimationSound, Item};

use crate::clip::{Branch, Control, Frames, Instruction, Step};
use crate::state::{Boost, Playhead, RunDown, State};
use crate::update::Update;

/// Where her clip goes from a control, and the sound the control asks for: what a control
/// answers, and a lap of her run that starts over in its place (`RunPace`, `BoostStages`).
#[derive(Clone, Copy)]
pub struct Onward {
    /// The step the clip goes on from, and its time left.
    pub playhead: Playhead,
    /// The sound asked for, if any.
    pub sound: Option<AnimationSound>,
}

impl Update<'_> {
    /// Spec: `ClipPlays`. Her clip, the one of her pose at the frame's start, plays: the time runs
    /// down by 1, and when it runs out the clip moves on. A shown step shows for its hold, or her
    /// stride's (`RunPace`), plus the slowdown; a control does what it says, unless it ends a lap
    /// of her run (`RunPace`, `BoostStages`). Gives the State and the sound asked for.
    #[must_use]
    pub fn play_clip(&self, state: State) -> (State, Option<AnimationSound>) {
        match state.playhead.run_down(self.clip) {
            RunDown::Stays(playhead) => (State { playhead, ..state }, None),
            RunDown::MovesOn(to, Instruction::Show(_)) => {
                let playhead = self.goes_on(to, self.stride(state.boost));
                (State { playhead, ..state }, None)
            }
            RunDown::MovesOn(at, Instruction::Control(control)) => {
                self.pass(at, control, state.boost)
            }
        }
    }

    /// The clip moving on to `control`, at step `at`. Where that ends a lap of her run (`RunPace`,
    /// `BoostStages`), her boost is the lap's, and a lap that starts over answers in the
    /// control's place; otherwise the control does what it says.
    fn pass(&self, at: Step, control: Control, boost: Boost) -> (State, Option<AnimationSound>) {
        let lap = self.lap_end(boost);
        let onward = lap
            .and_then(|lap| lap.starts_over)
            .unwrap_or_else(|| self.control(at, control));
        let state = State {
            playhead: onward.playhead,
            boost: lap.map_or(boost, |lap| lap.boost),
        };
        (state, onward.sound)
    }

    /// Spec: `ClipControls`, `Breathing`, `ClipEnds`, `WallJumpBranch`. The clip moving on to
    /// `control`, at step `at`, does what it says; the step it goes on from shows the clip's own
    /// hold.
    fn control(&self, at: Step, control: Control) -> Onward {
        let (playhead, sound) = match control {
            Control::Again(to) => (self.goes_on(to, None), None),
            Control::Breathe => (self.goes_on(self.breath(at), None), None),
            Control::Branch(branch) => {
                let (to, sound) = self.wall_jump(branch);
                (self.goes_on(to, None), Some(sound))
            }
            // A rest keeps her on it with no time left, so the clip moves on from it the next
            // frame. An end hands her to another pose this frame (§1.5), whose clip starts
            // afresh; were she to keep her pose, the end would keep her the way a rest does.
            Control::Rest | Control::End | Control::EndUnlessJump => {
                let rests = Playhead {
                    step: at,
                    frames_left: Frames::NONE,
                };
                (rests, None)
            }
        };
        Onward { playhead, sound }
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
}
