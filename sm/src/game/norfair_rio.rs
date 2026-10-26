//! Norfair Rio variants share their dive, bounce and animation transitions.
#![forbid(unsafe_code)]
use crate::game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::game::actors::api::Axis;
use crate::game::enemy_script::EnemyScript;
use crate::game::rio::{aim_velocity, bounce_horizontal, move_axis};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;

/// Both Rio variants share resume and follower-offset commands; the cursor is retained.
pub(crate) fn instruction(address: u32, cursor: EnemyScript, game: &mut Game) -> EnemyScript {
    let (variable, value) = match address {
        0xa2c1c9 | 0xa2c6d2 => (1, 1),
        0xa2c1d4 | 0xa2c1df => (2, 8),
        0xa2c1ea | 0xa2c237 => (2, 12),
        0xa2c1f5 | 0xa2c221 => (2, (-12i16) as u16),
        0xa2c200 => (2, 4),
        0xa2c20b | 0xa2c6dd => (2, 0),
        0xa2c216 => (2, (-4i16) as u16),
        0xa2c22c => (2, (-16i16) as u16),
        0xa2c6e8 => (2, 1),
        _ => unreachable!("unknown Rio animation command"),
    };
    let mut runtime = Runtime::new(game);
    let at = 0x7800 + usize::from(runtime.word(0xe54)) + 2 * variable;
    runtime.write_slice_native(at, value);
    cursor
}

pub(crate) fn initialize(lower: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let raw = runtime.word(0xe54);
    let key = Key::from_legacy(raw);
    let scratch = 0x7800 + usize::from(raw);
    runtime.write_slice_native(scratch + 2, 0);
    if !lower {
        runtime.write_slice_native(scratch + 4, 0);
    }
    let follower = runtime.resolve_actor(key, Native).parameters[0] & 0x8000 != 0;
    let (instruction, phase) = match (lower, follower) {
        (false, false) => (0xc0f1, 0xc2e7),
        (false, true) => (0xc18f, 0xc281),
        (true, false) => (0xc61a, 0xc771),
        (true, true) => (0xc6b0, 0xc72e),
    };
    if lower {
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = phase);
    }
    runtime.write_slice_native(scratch, instruction);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = instruction;
        if !lower {
            actor.pre_instruction = phase;
        }
    });
}

pub(crate) fn follow_parent(k: u16, lower: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(k);
    let drawing = 0xe84 + (usize::from(k) >> 1) * 2;
    let spawn = 0x77c0 + usize::from(k);
    if runtime.word(drawing + 200) == 0 {
        runtime.edit_actor(key, Native, |actor| actor.properties |= 0x200);
        return;
    }
    let frozen = runtime.word(drawing + 218);
    runtime.edit_actor(key, Native, |actor| actor.frozen = frozen);
    if !lower && frozen == 0 {
        runtime.edit_actor(key, Native, |actor| actor.properties &= !0x100);
    }
    if frozen != 0
        || if lower {
            runtime.native_word(spawn + 4) == 0
        } else {
            runtime.native_word(spawn) == 0xc0f1
        }
    {
        runtime.edit_actor(key, Native, |actor| actor.properties |= 0x100);
        return;
    }
    if !lower {
        let instruction = if runtime.native_word(spawn + 4) & 0x8000 != 0 {
            0xc1a3
        } else {
            0xc18f
        };
        animate_cached(&mut runtime, instruction, 0);
    }
    runtime.edit_actor(key, Native, |actor| actor.properties &= !0x100);
    let x = runtime.word(drawing + 182);
    runtime.edit_actor(key, Native, |actor| actor.x.whole = x);
    let offset = if lower {
        12
    } else {
        runtime.native_word(spawn + 4)
    };
    let y = offset.wrapping_add(runtime.word(drawing + 186));
    runtime.edit_actor(key, Native, |actor| actor.y.whole = y);
}

pub(crate) fn wait_to_dive(k: u16, lower: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(k);
    let finished = 0x7802 + usize::from(k);
    if runtime.word(0x5e5) & 0x101 != 0
        && ((crate::game::actors::api::player_delta(&runtime, key, Axis::X) as i16).wrapping_abs()
            as u16)
            < if lower { 0x70 } else { 0xc0 }
    {
        let vertical = if lower { 2 } else { 0 };
        let index = if lower {
            0
        } else {
            ((runtime.word(0x5e5) >> 1 & 2) >> 1) as isize
        };
        let tables = if lower {
            [0xa2c6ca, 0xa2c6ce]
        } else {
            [0xa2c1c1, 0xa2c1c5]
        };
        aim_velocity(&mut runtime, key, vertical, tables, index);
        animate_cached(&mut runtime, if lower { 0xc630 } else { 0xc107 }, 0);
        runtime.edit_actor(key, Native, |actor| {
            actor.pre_instruction = if lower { 0xc7bb } else { 0xc33f }
        });
    } else if lower || runtime.native_word(finished) != 0 {
        runtime.write_slice_native(finished, 0);
        animate_cached(&mut runtime, if lower { 0xc61a } else { 0xc0f1 }, 0);
    }
}

pub(crate) fn finish_animation(k: u16, lower: bool, recovering: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(k);
    if !lower && recovering {
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0xc2e7);
        return;
    }
    let finished = 0x7802 + usize::from(k);
    if runtime.native_word(finished) == 0 {
        return;
    }
    runtime.write_slice_native(finished, 0);
    let (instruction, phase) = match (lower, recovering) {
        (false, _) => (0xc12f, 0xc361),
        (true, false) => (0xc65a, 0xc7d6),
        (true, true) => (0xc686, 0xc771),
    };
    animate_cached(&mut runtime, instruction, 0);
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = phase);
    drop(runtime);
    if !lower {
        crate::game::music::QueueSfx2_Max6(0x65, game);
    }
}

pub(super) fn animate_cached(runtime: &mut Runtime<'_>, instruction: u16, cache_offset: usize) {
    let raw = runtime.word(0xe54);
    let last_animation = 0x7800 + usize::from(raw) + cache_offset;
    if runtime.native_word(last_animation) != instruction {
        runtime.write_slice_native(last_animation, instruction);
        runtime.edit_actor(Key::from_legacy(raw), Native, |actor| {
            actor.restart_animation(instruction);
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
            animate_cached(&mut runtime, if lower { 0xc662 } else { 0xc145 }, 0);
            runtime.edit_actor(key, Native, |actor| {
                actor.pre_instruction = if lower { 0xc82d } else { 0xc3b1 };
            });
            sound = lower;
        }
    } else if blocked {
        if lower {
            animate_cached(&mut runtime, 0xc686, 0);
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
            animate_cached(&mut runtime, if lower { 0xc674 } else { 0xc179 }, 0);
        }
    }
    drop(runtime);
    if sound {
        crate::game::music::QueueSfx2_Max6(0x64, game);
    }
}
