//! Mini Crocomire walking and firing share the common actor state and movement.
#![forbid(unsafe_code)]
use crate::game::actors::api::{self, MovementResult, SlopePolicy};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::words as fields;
use sm_game::actors::access::{AccessMode::LittleEndian as Le, ActorAccess, ActorKey as Key};
use sm_game::enemy_projectiles::host::Effects;
use sm_game::enemy_script::EnemyScript;

fn selected(runtime: &Runtime<'_>) -> Key {
    Key::from_legacy(runtime.word(fields::cur_enemy_index))
}

fn animate(runtime: &mut Runtime<'_>, instruction: u16) {
    let key = selected(runtime);
    runtime.edit_actor(key, Le, |actor| actor.restart_animation(instruction));
}

pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    runtime.edit_actor(key, Le, |actor| actor.spritemap = 0x804d);
    // The full parameter scales a linear ROM offset; it does not wrap at a bank.
    let offset = usize::from(runtime.resolve_actor(key, Le).parameters[1]) * 8;
    for (word, variable) in [1, 0, 3, 2].into_iter().enumerate() {
        let value = runtime.foreign(|game| game.read_rom_word(0xa28187, offset + 2 * word));
        runtime.edit_actor(key, Le, |actor| actor.variables[variable] = value);
    }
    animate(&mut runtime, 0x8932);
    runtime.edit_actor(key, Le, |actor| actor.pre_instruction = 0x8a43);
    let direction = runtime.resolve_actor(key, Le).parameters[0];
    runtime.edit_actor(key, Le, |actor| actor.variables[4] = direction);
    if direction != 0 {
        animate(&mut runtime, 0x8958);
        runtime.edit_actor(key, Le, |actor| actor.pre_instruction = 0x8a5c);
    }
}

pub(crate) fn patrol(right: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    let low = if right { 0 } else { 2 };
    let high_word = runtime.resolve_actor(key, Le).variables[low + 1];
    let low_word = runtime.resolve_actor(key, Le).variables[low];
    let speed = ((u32::from(high_word) << 16) | u32::from(low_word)) as i32;
    let blocked = api::move_horizontal(&mut runtime, key, speed, SlopePolicy::Ignore)
        == MovementResult::Blocked;
    let on_floor = if blocked {
        false
    } else {
        let key = selected(&runtime);
        api::probe_vertical(&mut runtime, key, 2 << 16) == MovementResult::Blocked
    };
    if !on_floor {
        animate(&mut runtime, 0x8932);
        let key = selected(&runtime);
        if runtime.resolve_actor(key, Le).variables[4] != 1 {
            animate(&mut runtime, 0x8958);
        }
    }
    runtime.next_random();
    let key = selected(&runtime);
    let frame = runtime.resolve_actor(key, Le).frame as u8;
    if frame.wrapping_add(runtime.word(fields::random_number) as u8) < 2 {
        runtime.edit_actor(key, Le, |actor| actor.pre_instruction = 0x8a75);
        animate(&mut runtime, if right { 0x896e } else { 0x8948 });
    }
}

pub(crate) fn resume(right: bool, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    runtime.edit_actor(key, Le, |actor| {
        actor.pre_instruction = if right { 0x8a5c } else { 0x8a43 };
    });
    runtime.edit_actor(key, Le, |actor| actor.variables[4] = u16::from(right));
    script
}
