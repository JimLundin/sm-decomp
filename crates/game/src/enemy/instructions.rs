//! Processing an enemy's instruction list (D13), bank $A0 log `$A0:C26A`: the instruction
//! timer counts down, and when it runs out the steps run until one stops the list for the
//! frame.

use std::marker::PhantomData;

use data::enemy::program::{EnemyPrograms, Instruction, Target};

use crate::state::room::{
    AiHandler, Enemy, EnemyBody, EnemyCommon, EnemyExtraProperty, EnemyProperty,
};
use crate::state::slots::{Frames, Label, Occupancy, Slot, StepIndex};

/// The label for a program step.
#[must_use]
pub const fn label(target: Target) -> Label<Enemy> {
    Label {
        program: target.program,
        step: StepIndex(target.step),
        family: PhantomData,
    }
}

/// The program step a label names: [`label`]'s inverse.
#[must_use]
pub const fn target(label: Label<Enemy>) -> Target {
    Target {
        program: label.program,
        step: label.step.0,
    }
}

/// The step after `at` in its program.
const fn next(at: Target) -> Target {
    Target {
        step: at.step.wrapping_add(1),
        ..at
    }
}

/// Process the slot's instructions (`$A0:C26A`).
///
/// A frozen enemy's list waits. Otherwise the instruction timer counts down; until it runs
/// out, the graphic isn't updated (`$A0:C2B1`). When it does, the steps run from the cursor:
/// a frame sets the timer and spritemap, moves the cursor past it and marks the graphic
/// updated (`$A0:C2A6`); a wait, a sleep or a delete stops the list and leaves the graphic
/// flag as it was.
// open: a step the content doesn't hold stops the list where it is; the loader only makes
// labels that hold one, and verify decodes only those.
pub fn process(slot: &mut Slot<EnemyCommon, EnemyBody>, programs: &EnemyPrograms) {
    let common = &mut slot.common;
    if common.ai_handlers.contains(AiHandler::Frozen) {
        return;
    }
    let ran_out = common.cursor.delay == Frames(1);
    common.cursor.delay = Frames(common.cursor.delay.0.wrapping_sub(1));
    if !ran_out {
        common
            .extra_properties
            .remove(EnemyExtraProperty::GraphicUpdated);
        return;
    }
    let mut at = target(common.cursor.at);
    // The loader rejects steps that can run forever without stopping, so this ends.
    while let Some(instruction) = programs.step(at) {
        match instruction {
            Instruction::Show { delay, spritemap } => {
                common.cursor.delay = Frames(delay);
                common.spritemap = spritemap;
                common.cursor.at = label(next(at));
                common
                    .extra_properties
                    .insert(EnemyExtraProperty::GraphicUpdated);
                return;
            }
            Instruction::Delete => {
                common.properties.insert(EnemyProperty::Deleted);
                if let Occupancy::Live(body) = slot.occupancy {
                    slot.occupancy = Occupancy::Deleting(body);
                }
                return;
            }
            Instruction::Call(function) => match function {},
            Instruction::Goto(to) => at = to,
            Instruction::DecrementTimerGoto(to) => {
                common.timer = common.timer.wrapping_sub(1);
                at = if common.timer == 0 { next(at) } else { to };
            }
            Instruction::SetTimer(value) => {
                common.timer = value;
                at = next(at);
            }
            Instruction::Sleep => {
                common.cursor.at = label(at);
                return;
            }
            Instruction::Wait(frames) => {
                common.cursor.delay = Frames(frames);
                common.cursor.at = label(next(at));
                return;
            }
            Instruction::EnableOffScreen => {
                common.properties.insert(EnemyProperty::ProcessedOffScreen);
                at = next(at);
            }
            Instruction::DisableOffScreen => {
                common.properties.remove(EnemyProperty::ProcessedOffScreen);
                at = next(at);
            }
        }
    }
}

#[cfg(test)]
mod tests;
