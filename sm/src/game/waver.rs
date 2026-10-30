//! Waver movement and animation use the common actor owner between legacy effects.
//! Animation words remain little-endian reads from the loaded ROM allocation.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game::actors::api as actor_api;
use crate::game::session_runtime::{Runtime, actor_sine};
use crate::game_state::Game;
use sm_game::actors::access::{AccessMode::LittleEndian as Le, ActorAccess, ActorKey as Key};
use sm_game::enemy_script::EnemyScript;

const INITIAL_INSTRUCTION: u16 = 0x86a7;
const ANIMATION_TABLE: u32 = 0xa386db;

pub fn Waver_Instr_1(_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript {
    let key = Key::from_legacy(game.cur_enemy_index());
    Runtime::new(game).edit_actor(key, Le, |actor| actor.variables[4] = 1);
    jp
}

pub fn Waver_Init(game: &mut Game) {
    let key = Key::from_legacy(game.cur_enemy_index());
    Runtime::new(game).edit_actor(key, Le, |actor| {
        actor.variables[1] = 1;
        actor.variables[0] = 0x8000;
        if actor.parameters[0] & 1 == 0 {
            actor.variables[1] = 0xfffe;
            // sign_extend8(0x8000) retains 0x8000, not zero.
            actor.variables[0] = 0x8000;
        }
        actor.pre_instruction = 0;
        actor.variables[2] = 0;
        actor.variables[4] = 0;
        actor.instruction = INITIAL_INSTRUCTION;
        actor.pre_instruction = actor.parameters[0] & 1;
    });
    Waver_Func_1(game);
}

// Spike collision can invoke retained native mechanism diagnostics.
// Keep the caller's existing backend contract while consolidating its actor state.
pub unsafe fn Waver_Main(game: &mut Game) {
    let index = game.cur_enemy_index();
    let key = Key::from_legacy(index);
    let horizontal_velocity = {
        let runtime = Runtime::new(game);
        let actor = runtime.resolve_actor(key, Le);
        ((u32::from(actor.variables[1]) << 16) | u32::from(actor.variables[0])) as i32
    };
    if actor_api::move_horizontal(
        &mut Runtime::new(game),
        key,
        horizontal_velocity,
        actor_api::SlopePolicy::Ignore,
    ) == actor_api::MovementResult::Blocked
    {
        Runtime::new(game).edit_actor(key, Le, |actor| {
            // The ROM reverses the middle two bytes of the 16.16 velocity.
            let middle = (actor.variables[0] >> 8) | (actor.variables[1] << 8);
            let reversed = middle.wrapping_neg();
            actor.variables[1] = (reversed >> 8) as u8 as i8 as i16 as u16;
            actor.variables[0] = ((middle as u8 as i8 as i16).wrapping_neg() as u16) << 8;
            actor.pre_instruction = (actor.pre_instruction ^ 1) & 1;
        });
        Waver_Func_1(game);
    } else {
        // Capture the live vertical selector before the sine effect; state keeps its original key.
        let vertical_index = game.cur_enemy_index();
        let phase = Runtime::new(game).resolve_actor(key, Le).variables[3] as u8 as u16;
        let vertical_velocity = i32::from(actor_sine(phase, 4, game) as i16) << 16;
        let collided = actor_api::move_vertical(
            &mut Runtime::new(game),
            Key::from_legacy(vertical_index),
            vertical_velocity,
        ) == actor_api::MovementResult::Blocked;
        Runtime::new(game).edit_actor(key, Le, |actor| {
            actor.variables[3] = if collided {
                actor.variables[3].wrapping_add(0x80) as u8 as u16
            } else {
                actor.variables[3].wrapping_add(2)
            };
        });
    }
    if Runtime::new(game).resolve_actor(key, Le).variables[3] & 0x7f == 56 {
        Runtime::new(game).edit_actor(key, Le, |actor| actor.pre_instruction |= 2);
        Waver_Func_1(game);
    }
    if Runtime::new(game).resolve_actor(key, Le).variables[4] != 0 {
        Runtime::new(game).edit_actor(key, Le, |actor| {
            actor.variables[4] = 0;
            actor.pre_instruction &= 1;
        });
        Waver_Func_1(game);
    }
}

pub fn Waver_Func_1(game: &mut Game) {
    let key = Key::from_legacy(game.cur_enemy_index());
    let animation = Runtime::new(game).resolve_actor(key, Le).pre_instruction;
    if animation != Runtime::new(game).resolve_actor(key, Le).variables[2] {
        Runtime::new(game).edit_actor(key, Le, |actor| actor.variables[2] = animation);
        let instruction = animation_instruction(game, animation);
        Runtime::new(game).edit_actor(key, Le, |actor| actor.restart_animation(instruction));
    }
}

/// Preserve the original unmasked animation index and linear ROM offset.
fn animation_instruction(game: &Game, animation: u16) -> u16 {
    game.read_rom_word(ANIMATION_TABLE, usize::from(animation) * 2)
}
