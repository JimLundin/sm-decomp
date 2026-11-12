//! §2 Her clip plays: the time runs down, and the clip moves on through its instructions.

use clean_types::{AnimationSound, Item};

use crate::clip::{Branch, Clip, Control, Frames, Instruction, Pace, Paces, Step};
use crate::inputs::Inputs;
use crate::playhead::{Playhead, RunDown};
use crate::rules::Rules;
use crate::run::{LapEnd, lap_end, stride};
use crate::state::{Boost, State};

/// Where her clip goes as it moves on to a step.
enum Onward<'p> {
    /// She rests on the step it moved on to, with no time left: a `rest`, or an end.
    Rests,
    /// The clip goes on from `to`, on `pace`'s hold for it where a pace gives one, else the
    /// clip's own, asking for `sound`.
    GoesOn {
        /// The step it goes on from.
        to: Step,
        /// Her stride's pace, where it sets the hold.
        pace: Option<&'p Pace>,
        /// The sound asked for, if any.
        sound: Option<AnimationSound>,
    },
}

/// Spec: `ClipPlays`. Her clip, `clip`, the one of her pose at the frame's start, plays: the
/// time runs down by a frame, and when it runs out the clip moves on. A shown step shows for its
/// hold, or her stride's (`RunPace`), plus the slowdown; a control does what it says, unless it
/// ends a lap of her run (`RunPace`, `BoostStages`). Gives the State and the sound asked for.
#[must_use]
pub fn play_clip(
    state: State,
    inputs: &Inputs,
    clip: &Clip,
    paces: &Paces,
    rules: &Rules,
) -> (State, Option<AnimationSound>) {
    let (at, instruction) = match state.playhead.run_down(clip) {
        RunDown::Stays(playhead) => return (State { playhead, ..state }, None),
        RunDown::MovesOn(at, instruction) => (at, instruction),
    };
    let (boost, onward) = match instruction {
        Instruction::Show(_) => {
            let pace = stride(inputs, paces, state.boost.stage);
            let onward = Onward::GoesOn {
                to: at,
                pace,
                sound: None,
            };
            (state.boost, onward)
        }
        Instruction::Control(control) => pass(at, control, state.boost, inputs, paces, rules),
    };
    let (playhead, sound) = match onward {
        // A rest keeps her on it with no time left, whatever the slowdown, so the clip moves on
        // from it the next frame. An end hands her to another pose this frame (§1.5), whose clip
        // starts afresh; were she to keep her pose, the end would keep her the way a rest does.
        Onward::Rests => {
            let rests = Playhead {
                step: at,
                frames_left: Frames::NONE,
            };
            (rests, None)
        }
        Onward::GoesOn { to, pace, sound } => {
            let goes_on = Playhead::on(to, clip, pace).slowed(inputs.liquid.slowdown);
            (goes_on, sound)
        }
    };
    (State { playhead, boost }, sound)
}

/// The clip moving on to `control`, at step `at`, with her boost `boost`. Where that ends a lap
/// of her run (`RunPace`, `BoostStages`), a lap that starts over goes in the control's place, and
/// laps only counted down leave the control to do as the clip says; otherwise the control does
/// what it says. Gives her boost after it and where the clip goes.
fn pass<'p>(
    at: Step,
    control: Control,
    boost: Boost,
    inputs: &Inputs,
    paces: &'p Paces,
    rules: &Rules,
) -> (Boost, Onward<'p>) {
    match lap_end(boost, inputs, paces, rules) {
        Some(LapEnd::StartsOver { boost, pace, sound }) => {
            let starts_over = Onward::GoesOn {
                to: Step::FIRST,
                pace: Some(pace),
                sound,
            };
            (boost, starts_over)
        }
        Some(LapEnd::CountedDown(laps)) => {
            let counted = Boost {
                laps_left: Some(laps),
                ..boost
            };
            (counted, onward(at, control, inputs, rules))
        }
        None => (boost, onward(at, control, inputs, rules)),
    }
}

/// Spec: `ClipControls`, `Breathing`, `ClipEnds`, `WallJumpBranch`. Where `control`, at step
/// `at`, sends the clip: the step it goes on from shows the clip's own hold.
fn onward(at: Step, control: Control, inputs: &Inputs, rules: &Rules) -> Onward<'static> {
    let (to, sound) = match control {
        Control::Again(to) => (to, None),
        Control::Breathe => (breath(at, inputs, rules), None),
        Control::Branch(branch) => {
            let (to, sound) = wall_jump(branch, inputs);
            (to, Some(sound))
        }
        Control::Rest | Control::End | Control::EndUnlessJump => return Onward::Rests,
    };
    Onward::GoesOn {
        to,
        pace: None,
        sound,
    }
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
