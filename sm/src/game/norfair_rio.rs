//! Norfair Rio variants share their dive, bounce and animation transitions.
#![forbid(unsafe_code)]
use crate::game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::game::actors::api::Axis;
use crate::game::rio::{bounce_horizontal, move_axis};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;

pub fn animate(instruction: u16, game: &mut Game) {
    animate_current(&mut Runtime::new(game), instruction);
}

fn animate_current(runtime: &mut Runtime<'_>, instruction: u16) {
    let raw = runtime.word(0xe54);
    let last_animation = 0x7800 + usize::from(raw);
    if runtime.native_word(last_animation) != instruction {
        runtime.write_slice_native(last_animation, instruction);
        runtime.edit_actor(Key::from_legacy(raw), Native, |actor| {
            actor.instruction = instruction;
            actor.instruction_delay = 1;
            actor.timer = 0;
        });
    }
}

pub(crate) fn dive(k: u16, lower: bool, turning: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(k);
    let vertical = if lower { 2 } else { 0 };
    bounce_horizontal(&mut runtime, key, vertical + 1);
    let blocked = move_axis(&mut runtime, key, Axis::Y, vertical);
    let mut sound = false;
    if turning {
        if blocked
            || runtime.edit_actor(key, Native, |actor| {
                actor.variables[vertical] = actor.variables[vertical].wrapping_sub(32);
                (actor.variables[vertical] as i16) < 0
            })
        {
            runtime.edit_actor(key, Native, |actor| actor.variables[vertical] = u16::MAX);
            animate_current(&mut runtime, if lower { 0xc662 } else { 0xc145 });
            runtime.edit_actor(key, Native, |actor| {
                actor.pre_instruction = if lower { 0xc82d } else { 0xc3b1 };
            });
            sound = lower;
        }
    } else if blocked {
        if lower {
            animate_current(&mut runtime, 0xc686);
        }
        runtime.edit_actor(key, Native, |actor| {
            actor.pre_instruction = if lower { 0xc888 } else { 0xc406 };
        });
    } else {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[vertical] = actor.variables[vertical].wrapping_sub(32);
        });
        let finished = 0x7802 + usize::from(k);
        if runtime.native_word(finished) != 0 {
            runtime.write_slice_native(finished, 0);
            animate_current(&mut runtime, if lower { 0xc674 } else { 0xc179 });
        }
    }
    drop(runtime);
    if sound {
        crate::game::sm_80::QueueSfx2_Max6(0x64, game);
    }
}
