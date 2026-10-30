//! Mini-Crocomire (bank $A2:8912-$A2:8B5F). The function is the pre-instruction word; lanes 0-3
//! are the right and left speeds (low word first), lane 4 says it is moving right.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::LittleEndian as Le, ActorAccess};
use crate::actors::api::{self, MovementResult::Blocked, SlopePolicy};
use crate::actors::combat::current;
use crate::actors::host::{ActorLease, LeaseActors};
use crate::enemy_projectiles::host::Effects as _;
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content::table;
use crate::ram::words as W;

const MOVING_LEFT: [u16; 2] = [0x8932, 0x8a43]; // instruction list, function
const MOVING_RIGHT: [u16; 2] = [0x8958, 0x8a5c];

fn animate(runtime: &mut impl ActorLease, list: u16) {
    let key = current(runtime);
    runtime.edit_actor(key, Le, |actor| actor.restart_animation(list));
}

/// Initialisation AI - enemy $CEFF (mini-Crocomire) ($A2:89AD): speeds from the linear speed
/// table ($A2:8187) at parameter 2 * 8 bytes (linear, no bank wrap), direction parameter 1.
pub fn initialize<H: LeaseActors>(game: &mut H) {
    let mut runtime = game.lease();
    let key = current(&runtime);
    runtime.edit_actor(key, Le, |actor| actor.spritemap = 0x804d);
    let index = runtime.resolve_actor(key, Le).parameters[1] as isize * 4;
    for (word, variable) in [1, 0, 3, 2].into_iter().enumerate() {
        let value = table(&runtime, 0xa2_8187, index + word as isize);
        runtime.edit_actor(key, Le, |actor| actor.variables[variable] = value);
    }
    animate(&mut runtime, MOVING_LEFT[0]);
    runtime.edit_actor(key, Le, |actor| actor.pre_instruction = MOVING_LEFT[1]);
    let direction = runtime.resolve_actor(key, Le).parameters[0];
    runtime.edit_actor(key, Le, |actor| actor.variables[4] = direction);
    if direction != 0 {
        animate(&mut runtime, MOVING_RIGHT[0]);
        runtime.edit_actor(key, Le, |actor| actor.pre_instruction = MOVING_RIGHT[1]);
    }
}

/// Mini-Crocomire function - moving left and right ($A2:8A43, $A2:8A5C): turn at walls and
/// ledges, and attack when the frame counter plus a random byte is under 2.
pub fn patrol<H: LeaseActors>(right: bool, game: &mut H) {
    let mut runtime = game.lease();
    let key = current(&runtime);
    let low = if right { 0 } else { 2 };
    let [high_word, low_word] =
        [low + 1, low].map(|lane| runtime.resolve_actor(key, Le).variables[lane]);
    let speed = ((u32::from(high_word) << 16) | u32::from(low_word)) as i32;
    let blocked = api::move_horizontal(&mut runtime, key, speed, SlopePolicy::Ignore) == Blocked;
    let on_floor = !blocked && {
        let key = current(&runtime);
        api::probe_vertical(&mut runtime, key, 2 << 16) == Blocked
    };
    if !on_floor {
        animate(&mut runtime, MOVING_LEFT[0]);
        if runtime.resolve_actor(current(&runtime), Le).variables[4] != 1 {
            animate(&mut runtime, MOVING_RIGHT[0]);
        }
    }
    runtime.next_random();
    let key = current(&runtime);
    let frame = runtime.resolve_actor(key, Le).frame as u8;
    if frame.wrapping_add(runtime.word(W::RANDOM_NUMBER) as u8) < 2 {
        runtime.edit_actor(key, Le, |actor| actor.pre_instruction = 0x8a75); // attacking: RTS
        animate(&mut runtime, if right { 0x896e } else { 0x8948 });
    }
}

/// Instruction - set moving left and right ($A2:8990, $A2:899D).
pub fn set_moving<H: LeaseActors>(right: bool, script: EnemyScript, game: &mut H) -> EnemyScript {
    let mut runtime = game.lease();
    let key = current(&runtime);
    let function = [MOVING_LEFT, MOVING_RIGHT][usize::from(right)][1];
    runtime.edit_actor(key, Le, |actor| actor.pre_instruction = function);
    runtime.edit_actor(key, Le, |actor| actor.variables[4] = u16::from(right));
    script
}
