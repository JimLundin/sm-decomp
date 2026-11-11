//! §2 Her clip plays: the time runs down, and the clip moves on through its instructions.

use clean_types::{AnimationSound, Item};

use crate::clip::{Branch, Frames, Instruction, Step};
use crate::frame::Frame;
use crate::state::{Playhead, RunDown, State};

impl Frame<'_> {
    /// Spec: `ClipPlays`. Her clip, the one of her pose at the frame's start, plays: the time runs
    /// down by 1, and when it runs out the clip moves on. A shown step shows for its hold, or her
    /// stride's (`RunPace`), plus the slowdown; a control does what it says, unless it ends a lap
    /// of her run (`RunPace`, `BoostStages`). Gives the State and the sound asked for.
    #[must_use]
    pub fn play_clip(&self, state: State) -> (State, Option<AnimationSound>) {
        let (to, instruction) = match state.playhead.run_down(self.clip) {
            RunDown::Stays(playhead) => return (State { playhead, ..state }, None),
            RunDown::MovesOn(to, instruction) => (to, instruction),
        };
        if let Instruction::Show(_) = instruction {
            let playhead = self.goes_on(to, self.stride(state.boost));
            return (State { playhead, ..state }, None);
        }
        let lap = self.lap_end(state.boost);
        let (playhead, sound) = lap.and_then(|lap| lap.starts_over).map_or_else(
            || self.control(to, instruction),
            |start| (start.playhead, start.sound),
        );
        let boost = lap.map_or(state.boost, |lap| lap.boost);
        (State { playhead, boost }, sound)
    }

    /// Spec: `ClipControls`, `Breathing`, `ClipEnds`, `WallJumpBranch`. The clip moving on to
    /// `instruction`, at step `at`, does what it says; the step it goes on from shows the clip's
    /// own hold.
    fn control(&self, at: Step, instruction: Instruction) -> (Playhead, Option<AnimationSound>) {
        match instruction {
            Instruction::Show(_) => (self.goes_on(at, None), None),
            Instruction::Again(to) => (self.goes_on(to, None), None),
            Instruction::Breathe => (self.goes_on(self.breath(at), None), None),
            Instruction::Branch(branch) => {
                let (to, sound) = self.wall_jump(branch);
                (self.goes_on(to, None), Some(sound))
            }
            // A rest keeps her on it with no time left, so the clip moves on from it the next
            // frame. An end hands her to another pose this frame (§1.5), whose clip starts
            // afresh; were she to keep her pose, the end would keep her the way a rest does.
            Instruction::Rest | Instruction::End | Instruction::EndUnlessJump => (
                Playhead {
                    step: at,
                    frames_left: Frames::NONE,
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
}
