//! Her clip plays: the time runs down, and when it is over the clip moves on as its steps say.

use clean_types::{AnimationSound, BoostStage, Item};

use crate::clip::{Branch, Clip, Control, Instruction, Pace, Step};
use crate::inputs::{Content, Inputs};
use crate::rules::Rules;
use crate::run::{boosting_lap_stage, hold_on, next_stage, stride};
use crate::state::{LapsLeft, Playhead, RunDown, State};

/// Spec: `ClipPlays`. The clip of her figure at the frame's start plays: the time runs down by a
/// frame, and when it is over the clip moves on to the next step. A shown step shows for its
/// hold, or her stride's (`RunPace`), plus the slowdown. A control that ends a lap of her run
/// (running, dashing and run held) ends it (`RunPace`, `BoostStages`); any other control does as
/// the clip says. Gives the State and the sound asked for.
#[must_use]
pub fn play(
    state: State,
    inputs: &Inputs,
    content: &impl Content,
    rules: &Rules,
) -> (State, Option<AnimationSound>) {
    let clip = content.clip(&inputs.figure);
    let (step, instruction) = match state.playhead.run_down(clip) {
        RunDown::Stays(playhead) => return (State { playhead, ..state }, None),
        RunDown::MovesOn(step, instruction) => (step, instruction),
    };
    let pace = stride(inputs, content.paces(), state.stage);
    match instruction {
        Instruction::Show(_) => {
            let playhead = Playhead::on(step, hold_on(step, pace, clip), &[inputs.liquid.slowdown]);
            (State { playhead, ..state }, None)
        }
        Instruction::Control(control) if pace.is_some() && inputs.run_held => {
            lap_end(state, step, control, inputs, content, rules)
        }
        Instruction::Control(control) => follow(state, step, control, inputs, clip, rules),
    }
}

/// Spec: `RunPace`, `BoostStages`. The clip moving on to `control`, at step `at`, ends a lap of
/// her run. Without the speed booster the lap starts over on the plain pace. With it, the laps
/// left go down by the lap she ran, and the control does as the clip says; once the stage has
/// lasted its laps, the stage goes up (boosting stays boosting), the next lap lasts its laps and
/// starts over on its pace. The stage going up to 4 asks for `Boosting`, and that lap takes its
/// first hold and its laps from the stage the sounds waiting name (`BoostingLap`).
fn lap_end(
    state: State,
    at: Step,
    control: Control,
    inputs: &Inputs,
    content: &impl Content,
    rules: &Rules,
) -> (State, Option<AnimationSound>) {
    let (clip, paces) = (content.clip(&inputs.figure), content.paces());
    if !inputs.items[Item::SpeedBooster] {
        let playhead = lap_start(&paces.plain, clip, inputs);
        return (State { playhead, ..state }, None);
    }
    if let Some(laps) = state.laps_left.after_lap() {
        let counted = State {
            laps_left: LapsLeft::WithBooster(laps),
            ..state
        };
        return follow(counted, at, control, inputs, clip, rules);
    }
    let new_stage = next_stage(state.stage);
    let starts_boosting = new_stage == BoostStage::Boosting && state.stage != BoostStage::Boosting;
    let lap_stage = if starts_boosting {
        boosting_lap_stage(inputs.sounds_waiting)
    } else {
        new_stage
    };
    let next_lap = State {
        playhead: lap_start(&paces.boosted[lap_stage], clip, inputs),
        stage: new_stage,
        laps_left: LapsLeft::WithBooster(rules.laps_per_stage[lap_stage]),
    };
    (
        next_lap,
        starts_boosting.then_some(AnimationSound::Boosting),
    )
}

/// Spec: `RunPace`. A lap starts over in the control's place: step 0, on `pace`'s first hold
/// plus the slowdown.
fn lap_start(pace: &Pace, clip: &Clip, inputs: &Inputs) -> Playhead {
    let hold = hold_on(Step::FIRST, Some(pace), clip);
    Playhead::on(Step::FIRST, hold, &[inputs.liquid.slowdown])
}

/// Spec: `ClipControls`, `Breathing`, `ClipEnds`, `WallJumpBranch`. The clip does as `control`,
/// at step `at`, says: the step it goes on from shows the clip's own hold plus the slowdown. A
/// `rest` keeps her on it with no time left, so the clip moves on the next frame; so does an end,
/// whose pose samus-motion hands over this frame.
fn follow(
    state: State,
    at: Step,
    control: Control,
    inputs: &Inputs,
    clip: &Clip,
    rules: &Rules,
) -> (State, Option<AnimationSound>) {
    let (to, sound) = match control {
        Control::Rest | Control::End | Control::EndUnlessJump => (at, None),
        Control::Again(to) => (to, None),
        Control::Breathe => (breath(at, inputs, rules), None),
        Control::Branch(branch) => {
            let (to, sound) = wall_jump(branch, inputs);
            (to, Some(sound))
        }
    };
    let playhead = Playhead::on(to, clip.hold(to), &[inputs.liquid.slowdown]);
    (State { playhead, ..state }, sound)
}

/// Spec: `Breathing`. From a `breathe` at `at`: back to step 0 while her energy is at the
/// threshold or more, else on to the next step.
fn breath(at: Step, inputs: &Inputs, rules: &Rules) -> Step {
    if inputs.energy >= rules.breathing_threshold {
        Step::FIRST
    } else {
        at.next().unwrap_or(at)
    }
}

/// Spec: `WallJumpBranch`. The spin she wall jumps in, and its sound: plainly when submerged,
/// else as the screw attack with it, else as the space jump with it, else plainly.
fn wall_jump(branch: Branch, inputs: &Inputs) -> (Step, AnimationSound) {
    let items = &inputs.items;
    if inputs.liquid.submerged {
        (branch.spin, AnimationSound::PlainSpin)
    } else if items[Item::ScrewAttack] {
        (branch.screw_attack, AnimationSound::ScrewAttack)
    } else if items[Item::SpaceJump] {
        (branch.space_jump, AnimationSound::SpaceJump)
    } else {
        (branch.spin, AnimationSound::PlainSpin)
    }
}
