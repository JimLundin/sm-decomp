//! Boyon, the bouncing blobs (bank $A2:8687-$A2:8911). Was `goofball`. Lanes: speed scale,
//! speed, bounce height, distance risen, bounce phase; the rest is RAM at $7E:7800 + enemy.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::LittleEndian as Le, ActorAccess, ActorKey as Key};
use crate::actors::api::{self, Axis};
use crate::actors::combat::current;
use crate::actors::host::{ActorLease, LeaseActors};
use crate::enemy_projectiles::host::Effects as _;
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content::{offset, table};

const EXTRA: usize = 0x7800; // word offsets below
const LIMIT: usize = 0;
const DIRECTION: usize = 2;
const RESTING: usize = 4;
const SAVED_SCALE: usize = 6;
const REST_ANIMATION: usize = 8;
const INITIALIZED: usize = 10;
const IDLE: u16 = 0x86a7; // instruction lists
const BOUNCING: u16 = 0x86bf;

fn extra(runtime: &impl ActorLease) -> usize {
    EXTRA + usize::from(current(runtime).byte_index())
}

/// Initialisation AI - enemy $CEBF (boyon) ($A2:871C): speed scale and bounce height from the
/// Boyon data ($A2:86DF, $A2:86EF) by the bytes of parameter 1.
pub fn initialize<H: LeaseActors>(game: &mut H) {
    let mut runtime = game.lease();
    let key = current(&runtime);
    runtime.edit_actor(key, Le, |actor| actor.spritemap = 0x804d);
    runtime.edit_actor(key, Le, |actor| actor.restart_animation(IDLE));
    runtime.write_slice_le(extra(&runtime) + DIRECTION, 1);
    let [scale, height] = runtime.resolve_actor(key, Le).parameters[0].to_le_bytes();
    let scale = table(&runtime, 0xa2_86df, isize::from(scale));
    runtime.edit_actor(key, Le, |actor| actor.variables[0] = scale);
    let height = table(&runtime, 0xa2_86ef, isize::from(height));
    runtime.edit_actor(key, Le, |actor| {
        (actor.variables[2], actor.variables[3], actor.variables[4]) = (height, 0, 0);
        actor.pre_instruction = 0;
    });
}

/// The speed at `phase`: the sample at $A2:8701 (FFh from 23 on) times the speed scale.
fn speed(runtime: &mut impl ActorLease, key: Key, phase: u16) -> u16 {
    let image = runtime.leased_image().expect("ROM must be loaded");
    let sample = image.read(offset(0xa2_8701) + usize::from(phase));
    let sample = if (phase.wrapping_sub(23) as i16) >= 0 {
        255
    } else {
        sample
    };
    u16::from(sample) * u16::from(runtime.resolve_actor(key, Le).variables[0] as u8)
}

/// Calculate initial bounce speed ($A2:8755): the phase at which the bounce reaches its height.
fn initialize_bounce(runtime: &mut impl ActorLease) {
    let key = current(runtime);
    loop {
        let phase = runtime.resolve_actor(key, Le).variables[4];
        let speed = speed(runtime, key, phase);
        let complete = runtime.edit_actor(key, Le, |actor| {
            actor.variables[1] = speed;
            actor.variables[3] = actor.variables[3].wrapping_add(speed);
            actor.variables[4] = actor.variables[4].wrapping_add(1);
            (actor.variables[3].wrapping_sub(actor.variables[2]) as i16) >= 0
        });
        if complete {
            break;
        }
    }
    let phase = runtime.resolve_actor(key, Le).variables[4];
    let base = extra(runtime);
    runtime.write_slice_le(base + LIMIT, phase);
    runtime.write_slice_le(base + RESTING, 1);
    runtime.write_slice_le(base + REST_ANIMATION, 1);
}

/// Main AI - enemy $CEBF (boyon) ($A2:879C): bounce while Samus is in proximity.
pub fn main_ai<H: LeaseActors>(game: &mut H) {
    let mut runtime = game.lease();
    let key = current(&runtime);
    let base = extra(&runtime);
    if runtime.word(base + INITIALIZED) == 0 {
        initialize_bounce(&mut runtime);
        return runtime.write_slice_le(base + INITIALIZED, 1);
    }
    let scale = runtime.resolve_actor(key, Le).variables[0];
    runtime.write_slice_le(base + SAVED_SCALE, scale);
    let selected = current(&runtime);
    let radius = runtime.resolve_actor(selected, Le).parameters[1];
    if api::player_within(&mut runtime, selected, Axis::X, radius) {
        runtime.write_slice_le(base + RESTING, 0);
        runtime.write_slice_le(base + REST_ANIMATION, 0);
        if runtime.resolve_actor(key, Le).pre_instruction == 0 {
            runtime.edit_actor(key, Le, |actor| actor.pre_instruction = 1);
            runtime.edit_actor(key, Le, |actor| actor.restart_animation(BOUNCING));
        }
    } else if runtime.word(base + RESTING) != 0 {
        if runtime.word(base + REST_ANIMATION) == 0 {
            runtime.write_slice_le(base + REST_ANIMATION, 1);
            runtime.edit_actor(key, Le, |actor| actor.restart_animation(IDLE));
        }
        return;
    }
    let scale = runtime.word(base + SAVED_SCALE);
    runtime.edit_actor(key, Le, |actor| actor.variables[0] = scale);
    match runtime.word(base + DIRECTION) {
        0 => bounce(&mut runtime, true),
        1 => bounce(&mut runtime, false),
        _ => panic!("invalid Goofball state"),
    }
}

/// Boyon bounce movement - falling and rising ($A2:8801, $A2:8850).
fn bounce(runtime: &mut impl ActorLease, falling: bool) {
    let key = current(runtime);
    let base = extra(runtime);
    if falling {
        runtime.edit_actor(key, Le, |actor| {
            actor.variables[4] = actor.variables[4].wrapping_add(1)
        });
    }
    let phase = runtime.resolve_actor(key, Le).variables[4];
    let speed = speed(runtime, key, phase);
    let phase = runtime.edit_actor(key, Le, |actor| {
        actor.variables[1] = speed;
        let distance = speed >> 8;
        if falling {
            actor.y.whole = actor.y.whole.wrapping_add(distance);
        } else {
            actor.y.whole = actor.y.whole.wrapping_sub(distance);
            actor.variables[4] = actor.variables[4].wrapping_sub(1);
        }
        actor.variables[4]
    });
    if falling && (phase.wrapping_sub(runtime.word(base + LIMIT)) as i16) >= 0 {
        runtime.write_slice_le(base + DIRECTION, 1);
        runtime.write_slice_le(base + RESTING, 1);
        runtime.edit_actor(key, Le, |actor| actor.pre_instruction = 0);
    } else if !falling && (phase & 0x8000) != 0 {
        runtime.write_slice_le(base + DIRECTION, 0);
    }
}

/// Instruction - start bounce ($A2:88C6).
pub fn start_bounce<H: LeaseActors>(script: EnemyScript, game: &mut H) -> EnemyScript {
    let mut runtime = game.lease();
    let base = extra(&runtime);
    runtime.write_slice_le(base + RESTING, 0);
    runtime.queue_sound(2, 6, 0xe);
    script
}
