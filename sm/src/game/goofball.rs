//! Bouncing goofballs share one rising/falling step and the common actor codec.
//! Extended bounce words remain RAM-authoritative until their state cohort migrates.
#![forbid(unsafe_code)]
use crate::game::actors::access::{AccessMode::LittleEndian as Le, ActorAccess, ActorKey as Key};
use crate::game::actors::api::{self, Axis};
use crate::game::enemy_projectiles::host::Effects;
use crate::game::enemy_script::EnemyScript;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;

const LIMIT: usize = 0;
const DIRECTION: usize = 2;
const RESTING: usize = 4;
const SAVED_SCALE: usize = 6;
const REST_ANIMATION: usize = 8;
const INITIALIZED: usize = 10;

fn restart(runtime: &mut Runtime<'_>, instruction: u16) {
    let key = Key::from_legacy(runtime.word(0xe54));
    runtime.edit_actor(key, Le, |actor| actor.restart_animation(instruction));
}

pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let index = runtime.word(0xe54);
    let key = Key::from_legacy(index);
    runtime.edit_actor(key, Le, |actor| actor.spritemap = 0x804d);
    restart(&mut runtime, 0x86a7);
    runtime.write_slice_le(0x7800 + usize::from(index) + DIRECTION, 1);
    let parameter = runtime.resolve_actor(key, Le).parameters[0] as u8;
    let scale = runtime.foreign(|game| game.read_rom_word(0xa286df, usize::from(parameter) * 2));
    runtime.edit_actor(key, Le, |actor| actor.variables[0] = scale);
    let parameter = (runtime.resolve_actor(key, Le).parameters[0] >> 8) as u8;
    let distance = runtime.foreign(|game| game.read_rom_word(0xa286ef, usize::from(parameter) * 2));
    runtime.edit_actor(key, Le, |actor| {
        actor.variables[2] = distance;
        actor.variables[3] = 0;
        actor.variables[4] = 0;
        actor.pre_instruction = 0;
    });
}

fn movement_product(runtime: &mut Runtime<'_>, key: Key, phase: u16) -> u16 {
    // The sample read precedes the signed gate and the live scale read, even above 23.
    let sample = runtime.foreign(|game| game.read_rom_byte(0xa28701, usize::from(phase)));
    let sample = if (phase.wrapping_sub(23) as i16) >= 0 {
        255
    } else {
        sample
    };
    u16::from(sample) * u16::from(runtime.resolve_actor(key, Le).variables[0] as u8)
}

fn initialize_arc(runtime: &mut Runtime<'_>) {
    let index = runtime.word(0xe54);
    let key = Key::from_legacy(index);
    loop {
        let phase = runtime.resolve_actor(key, Le).variables[4];
        let product = movement_product(runtime, key, phase);
        let complete = runtime.edit_actor(key, Le, |actor| {
            actor.variables[1] = product;
            actor.variables[3] = actor.variables[3].wrapping_add(product);
            actor.variables[4] = actor.variables[4].wrapping_add(1);
            (actor.variables[3].wrapping_sub(actor.variables[2]) as i16) >= 0
        });
        if complete {
            break;
        }
    }
    let phase = runtime.resolve_actor(key, Le).variables[4];
    let base = 0x7800 + usize::from(index);
    runtime.write_slice_le(base + LIMIT, phase);
    runtime.write_slice_le(base + RESTING, 1);
    runtime.write_slice_le(base + REST_ANIMATION, 1);
}

pub(crate) fn advance(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let index = runtime.word(0xe54);
    let key = Key::from_legacy(index);
    let base = 0x7800 + usize::from(index);
    if runtime.word(base + INITIALIZED) == 0 {
        initialize_arc(&mut runtime);
        runtime.write_slice_le(base + INITIALIZED, 1);
        return;
    }
    let scale = runtime.resolve_actor(key, Le).variables[0];
    runtime.write_slice_le(base + SAVED_SCALE, scale);
    let current = Key::from_legacy(runtime.word(0xe54));
    let radius = runtime.resolve_actor(current, Le).parameters[1];
    if api::player_within(&mut runtime, current, Axis::X, radius) {
        runtime.write_slice_le(base + RESTING, 0);
        runtime.write_slice_le(base + REST_ANIMATION, 0);
        if runtime.resolve_actor(key, Le).pre_instruction == 0 {
            runtime.edit_actor(key, Le, |actor| actor.pre_instruction = 1);
            restart(&mut runtime, 0x86bf);
        }
    } else if runtime.word(base + RESTING) != 0 {
        if runtime.word(base + REST_ANIMATION) == 0 {
            runtime.write_slice_le(base + REST_ANIMATION, 1);
            restart(&mut runtime, 0x86a7);
        }
        return;
    }
    let scale = runtime.word(base + SAVED_SCALE);
    runtime.edit_actor(key, Le, |actor| actor.variables[0] = scale);
    match runtime.word(base + DIRECTION) {
        0 => step(&mut runtime, true),
        1 => step(&mut runtime, false),
        _ => panic!("invalid Goofball state"),
    }
}

fn step(runtime: &mut Runtime<'_>, falling: bool) {
    let index = runtime.word(0xe54);
    let key = Key::from_legacy(index);
    let base = 0x7800 + usize::from(index);
    if falling {
        runtime.edit_actor(key, Le, |actor| {
            actor.variables[4] = actor.variables[4].wrapping_add(1)
        });
    }
    let phase = runtime.resolve_actor(key, Le).variables[4];
    let product = movement_product(runtime, key, phase);
    let phase = runtime.edit_actor(key, Le, |actor| {
        actor.variables[1] = product;
        let distance = product >> 8;
        actor.y.whole = if falling {
            actor.y.whole.wrapping_add(distance)
        } else {
            actor.y.whole.wrapping_sub(distance)
        };
        if !falling {
            actor.variables[4] = actor.variables[4].wrapping_sub(1);
        }
        actor.variables[4]
    });
    if falling {
        if (phase.wrapping_sub(runtime.word(base + LIMIT)) as i16) >= 0 {
            runtime.write_slice_le(base + DIRECTION, 1);
            runtime.write_slice_le(base + RESTING, 1);
            runtime.edit_actor(key, Le, |actor| actor.pre_instruction = 0);
        }
    } else if phase & 0x8000 != 0 {
        runtime.write_slice_le(base + DIRECTION, 0);
    }
}

pub(crate) fn landed(script: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    let index = runtime.word(0xe54);
    runtime.write_slice_le(0x7800 + usize::from(index) + RESTING, 0);
    runtime.queue_sound(2, 6, 0xe);
    script
}
