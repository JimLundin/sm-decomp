//! Zoa movement on owned actor state, with native extended-word aliases.
#![forbid(unsafe_code)]
use super::{
    INVISIBLE, LEFT_RISING, LEFT_SHOOTING, NEAR_COLUMNS, RIGHT_RISING, RISE, SIGN, function, lane,
};
use crate::actors::access::{
    AccessMode::Native, ActorAccess, ActorCollections, CollectionStore, CollectionWindow,
};
use crate::actors::api::{self, Axis};
use crate::actors::combat;
use crate::enemy_script::EnemyScript;

#[derive(Clone, Copy, Debug)]
pub enum Table {
    InstructionLists,
    XSpeeds,
}

pub trait Host: ActorAccess + ActorCollections {
    fn zoa_table(&self, table: Table, index: isize) -> u16;
    fn unknown_zoa_function(&mut self);
}

fn x_speed_index(runtime: &impl Host) -> usize {
    usize::from(combat::current(runtime).byte_index())
}
/// Start the instruction list for Zoa's instruction list index, if it changed.
fn set_instruction_list(runtime: &mut impl Host) {
    let key = combat::current(runtime);
    let index = runtime.resolve_actor(key, Native).variables[lane::INSTRUCTION_LIST_INDEX];
    if index != runtime.resolve_actor(key, Native).variables[lane::PREVIOUS_INSTRUCTION_LIST_INDEX]
    {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[lane::PREVIOUS_INSTRUCTION_LIST_INDEX] = index
        });
        let instruction = runtime.zoa_table(Table::InstructionLists, index as isize);
        runtime.edit_actor(key, Native, |actor| actor.restart_animation(instruction));
    }
}
/// Initialisation AI - Zoa ($A3:B44A).
pub fn initialize(runtime: &mut impl Host) {
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.parameters[0] = function::WAIT_FOR_SAMUS;
        actor.variables[lane::INSTRUCTION_LIST_INDEX] = 0;
        actor.variables[lane::PREVIOUS_INSTRUCTION_LIST_INDEX] = 0;
    });
    let at = x_speed_index(runtime);
    runtime.write_collection_word(
        CollectionWindow::SpeciesExtended,
        at,
        0,
        CollectionStore::SliceNative,
    );
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = LEFT_SHOOTING;
        actor.properties |= INVISIBLE;
        actor.variables[lane::SPAWN_X] = actor.x.whole;
        actor.variables[lane::SPAWN_Y] = actor.y.whole;
    });
}
/// Main AI - Zoa ($A3:B47C): run its function. Was `advance`.
pub fn main_ai(runtime: &mut impl Host) {
    let key = combat::current(runtime);
    match runtime.resolve_actor(key, Native).parameters[0] {
        // Zoa function - wait for Samus to get near ($A3:B482).
        function::WAIT_FOR_SAMUS => {
            let current = combat::current(runtime);
            if api::player_within(runtime, current, Axis::X, NEAR_COLUMNS) {
                let current = combat::current(runtime);
                let index = if api::player_delta(runtime, current, Axis::X) & SIGN == 0 {
                    RIGHT_RISING
                } else {
                    LEFT_RISING
                };
                let key = combat::current(runtime);
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[lane::INSTRUCTION_LIST_INDEX] = index
                });
                set_instruction_list(runtime);
                runtime.edit_actor(key, Native, |actor| actor.parameters[0] = function::RISING);
            }
        }
        // Zoa function - rising ($A3:B4A8): rise to Samus's height, then shoot.
        function::RISING => {
            runtime.edit_actor(key, Native, |actor| actor.properties &= !INVISIBLE);
            if api::player_delta(runtime, key, Axis::Y) & SIGN != 0 {
                runtime.edit_actor(key, Native, |actor| actor.y.advance(RISE));
            } else {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[lane::INSTRUCTION_LIST_INDEX] =
                        actor.variables[lane::INSTRUCTION_LIST_INDEX].wrapping_sub(1)
                });
                set_instruction_list(runtime);
                runtime.edit_actor(key, Native, |actor| {
                    actor.pre_instruction = 0;
                    actor.parameters[0] = function::SHOOTING;
                });
            }
        }
        // Zoa function - shooting ($A3:B4D6): shoot sideways until off
        // screen, then return to the spawn point and wait.
        function::SHOOTING => {
            let index = (runtime.collection_word(
                CollectionWindow::SpeciesExtended,
                x_speed_index(runtime),
                Native,
            ) >> 1) as isize;
            let high = runtime.zoa_table(Table::XSpeeds, index);
            let low = runtime.zoa_table(Table::XSpeeds, index + 1);
            let speed = ((u32::from(high) << 16) | u32::from(low)) as i32;
            // Preserve ordinary signed negation, including its checked-build overflow.
            let speed = if runtime.resolve_actor(key, Native).variables
                [lane::INSTRUCTION_LIST_INDEX]
                != 0
            {
                speed
            } else {
                -speed
            };
            runtime.edit_actor(key, Native, |actor| actor.x.advance(speed));
            let outside = api::current_outside_screen(runtime);
            if outside {
                runtime.edit_actor(key, Native, |actor| {
                    actor.properties |= INVISIBLE;
                    actor.x.whole = actor.variables[lane::SPAWN_X];
                    actor.y.whole = actor.variables[lane::SPAWN_Y];
                    actor.variables[lane::INSTRUCTION_LIST_INDEX] = 0;
                });
            }
            set_instruction_list(runtime);
            if outside {
                runtime.edit_actor(key, Native, |actor| {
                    actor.parameters[0] = function::WAIT_FOR_SAMUS
                });
            }
        }
        _ => runtime.unknown_zoa_function(),
    }
}
/// Instruction - enemy X speed table index = `value` ($A3:B429, $A3:B434, $A3:B43F).
pub fn set_x_speed_index(value: u16, script: EnemyScript, runtime: &mut impl Host) -> EnemyScript {
    let at = x_speed_index(runtime);
    runtime.write_collection_word(
        CollectionWindow::SpeciesExtended,
        at,
        value,
        CollectionStore::SliceNative,
    );
    script
}
