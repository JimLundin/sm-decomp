//! Her clip plays: the time runs down, and when it is over the clip moves on as its steps say.

use types::{BoostStage, Item};

use crate::clip::{Branch, Clip, Control, Hold, Instruction, Pace, Paces, Step};
use crate::inputs::Inputs;
use crate::rules::Rules;
use crate::run::{boosting_lap_stage, hold_on, next_stage, stride};
use crate::sounds::Sound;
use crate::state::{Animation, Laps, LapsLeft, Playhead, RunDown};

/// Her clip as it plays this frame: the clip of the figure she starts the frame in, the run's
/// paces, the frame's inputs and the tuning.
pub struct ClipPlay<'a> {
    /// The frame's inputs.
    pub inputs: &'a Inputs,
    /// The clip of her figure as the frame starts.
    pub clip: &'a Clip,
    /// The run's paces.
    pub paces: &'a Paces,
    /// The tuning.
    pub rules: &'a Rules,
}

impl ClipPlay<'_> {
    /// Spec: `ClipPlays`. The time runs down by a frame, and when it is over the clip moves on
    /// to the next step. A shown step shows for its hold, or her stride's (`RunPace`), plus the
    /// step slowdown. Running, dashing as the clip moves on and with run held, any control the
    /// clip moves on to ends a lap of her run (`RunPace`, `BoostStages`); any other control does
    /// as the clip says. Gives her animation and the sound asked for.
    pub fn play(&self, animation: Animation) -> (Animation, Option<Sound>) {
        let (step, instruction) = match animation.playhead.run_down(self.clip) {
            RunDown::Stays(playhead) => return (animation.with_playhead(playhead), None),
            RunDown::MovesOn(step, instruction) => (step, instruction),
        };
        let pace = stride(self.inputs, self.paces, animation.stage);
        match instruction {
            Instruction::Show(_) => {
                let playhead = self.slowed(step, hold_on(step, pace, self.clip));
                (animation.with_playhead(playhead), None)
            }
            Instruction::Control(control) if pace.is_some() && self.inputs.run_held => {
                self.lap_end(animation, step, control)
            }
            Instruction::Control(control) => self.follow(animation, step, control),
        }
    }

    /// The clip on `step` with `hold` plus the step slowdown as the time left.
    fn slowed(&self, step: Step, hold: Option<Hold>) -> Playhead {
        Playhead::on(step, hold, &[self.inputs.liquid.step_slowdown])
    }

    /// Spec: `RunPace`, `BoostStages`. The clip moving on to `control`, at step `at`, ends a lap
    /// of her run. Without the speed booster the lap starts over on the plain pace. With it, a
    /// lap that leaves laps on the stage only counts (`CountedLapHold`); once the stage has
    /// lasted its laps, the stage goes up (boosting stays boosting), and the next lap lasts its
    /// laps and starts over on its pace. The stage going up to 4 asks for `Boosting`, and that
    /// lap takes its first hold and its laps from the stage the sounds waiting name
    /// (`BoostingLap`).
    fn lap_end(
        &self,
        animation: Animation,
        at: Step,
        control: Control,
    ) -> (Animation, Option<Sound>) {
        if !self.inputs.items[Item::SpeedBooster] {
            let playhead = self.lap_start(&self.paces.plain);
            return (animation.with_playhead(playhead), None);
        }
        if let Some(laps) = animation.laps_left.after_lap() {
            return self.counted_lap(animation, laps, at, control);
        }
        let new_stage = next_stage(animation.stage);
        let starts_boosting =
            new_stage == BoostStage::Boosting && animation.stage != BoostStage::Boosting;
        let lap_stage = if starts_boosting {
            boosting_lap_stage(self.inputs.sounds_waiting)
        } else {
            new_stage
        };
        let next_lap = Animation {
            playhead: self.lap_start(&self.paces.boosted[lap_stage]),
            stage: new_stage,
            laps_left: LapsLeft::WithBooster(self.rules.laps_per_stage[lap_stage]),
        };
        (next_lap, starts_boosting.then_some(Sound::Boosting))
    }

    /// Spec: `CountedLapHold` (a quirk). A boosted lap's end that only counts down her laps,
    /// leaving `laps` on the stage: the control does as the clip says, and the step it leads to
    /// takes the clip's own hold plus the step slowdown, in place of the boosted pace's. The
    /// pace's hold returns with the next shown step the clip moves on to.
    fn counted_lap(
        &self,
        animation: Animation,
        laps: Laps,
        at: Step,
        control: Control,
    ) -> (Animation, Option<Sound>) {
        let counted = Animation {
            laps_left: LapsLeft::WithBooster(laps),
            ..animation
        };
        self.follow(counted, at, control)
    }

    /// Spec: `RunPace`. A lap starts over in the control's place: step 0, on `pace`'s first hold
    /// plus the step slowdown.
    fn lap_start(&self, pace: &Pace) -> Playhead {
        self.slowed(Step::FIRST, hold_on(Step::FIRST, Some(pace), self.clip))
    }

    /// Spec: `ClipControls`, `Breathing`, `ClipEnds`, `WallJumpBranch`. The clip does as
    /// `control`, at step `at`, says: the step it goes on from shows the clip's own hold plus the
    /// step slowdown. A `rest` keeps her on it with no time left, so the clip moves on the next
    /// frame; so does an end, whose pose samus-movement hands over this frame.
    fn follow(
        &self,
        animation: Animation,
        at: Step,
        control: Control,
    ) -> (Animation, Option<Sound>) {
        let (to, sound) = match control {
            Control::Rest | Control::End | Control::EndUnlessJump => (at, None),
            Control::Again(to) => (to, None),
            Control::Breathe => (self.breath(at), None),
            Control::Branch(branch) => {
                let (to, sound) = self.wall_jump(branch);
                (to, Some(sound))
            }
        };
        let playhead = self.slowed(to, self.clip.hold(to));
        (animation.with_playhead(playhead), sound)
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
    fn wall_jump(&self, branch: Branch) -> (Step, Sound) {
        let items = &self.inputs.items;
        if self.inputs.liquid.submerged {
            (branch.spin, Sound::PlainSpin)
        } else if items[Item::ScrewAttack] {
            (branch.screw_attack, Sound::ScrewAttack)
        } else if items[Item::SpaceJump] {
            (branch.space_jump, Sound::SpaceJump)
        } else {
            (branch.spin, Sound::PlainSpin)
        }
    }
}
