//! Waver (bank $A3:8687-$A3:88EF). Lanes: X subvelocity and velocity, current list index, sine
//! phase, spin finished flag; the pre-instruction word is the wanted list index (bit 0 facing
//! right, bit 1 spinning). Each step takes its own lease, as the original's did.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::LittleEndian as Le, ActorAccess};
use crate::actors::api::{self, MovementResult::Blocked, SlopePolicy};
use crate::actors::combat::current;
use crate::actors::host::{BossEffects, LeaseActors};
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content::table;

/// Instruction - set spin finished flag ($A3:86E3).
pub fn set_spin_finished<H: LeaseActors>(script: EnemyScript, game: &mut H) -> EnemyScript {
    let mut runtime = game.lease();
    runtime.edit_actor(current(&runtime), Le, |actor| actor.variables[4] = 1);
    script
}

/// Initialisation AI - enemy $D63F (waver) ($A3:86ED): parameter 1 bit 0 faces it right.
pub fn initialize<H: LeaseActors>(game: &mut H) {
    let mut runtime = game.lease();
    runtime.edit_actor(current(&runtime), Le, |actor| {
        let right = actor.parameters[0] & 1;
        // sign_extend8(0x8000) keeps 0x8000 as the subvelocity either way.
        (actor.variables[1], actor.variables[0]) = (if right == 0 { 0xfffe } else { 1 }, 0x8000);
        (actor.variables[2], actor.variables[4], actor.instruction) = (0, 0, 0x86a7);
        actor.pre_instruction = right;
    });
    drop(runtime);
    set_instruction_list(game);
}

/// Main AI - enemy $D63F (waver) ($A3:874C): fly horizontally and turn at walls, bob on a sine,
/// and spin when the phase reaches 38h.
pub fn main_ai<H: LeaseActors + BossEffects>(game: &mut H) {
    let (key, velocity) = {
        let runtime = game.lease();
        let key = current(&runtime);
        let [low, high, ..] = runtime.resolve_actor(key, Le).variables;
        (key, ((u32::from(high) << 16) | u32::from(low)) as i32)
    };
    if api::move_horizontal(&mut game.lease(), key, velocity, SlopePolicy::Ignore) == Blocked {
        game.lease().edit_actor(key, Le, |actor| {
            // The ROM reverses the middle two bytes of the 16.16 velocity.
            let middle = (actor.variables[0] >> 8) | (actor.variables[1] << 8);
            actor.variables[1] = (middle.wrapping_neg() >> 8) as u8 as i8 as i16 as u16;
            actor.variables[0] = ((middle as u8 as i8 as i16).wrapping_neg() as u16) << 8;
            actor.pre_instruction = (actor.pre_instruction ^ 1) & 1;
        });
        set_instruction_list(game);
    } else {
        // The live selector is read before the sine; the state keeps its original key.
        let (vertical, phase) = {
            let runtime = game.lease();
            (
                current(&runtime),
                runtime.resolve_actor(key, Le).variables[3] as u8 as u16,
            )
        };
        let velocity = i32::from(game.actor_sine(phase, 4) as i16) << 16;
        let collided = api::move_vertical(&mut game.lease(), vertical, velocity) == Blocked;
        game.lease().edit_actor(key, Le, |actor| {
            let phase = actor.variables[3];
            actor.variables[3] = if collided {
                phase.wrapping_add(0x80) as u8 as u16
            } else {
                phase.wrapping_add(2)
            };
        });
    }
    if (game.lease().resolve_actor(key, Le).variables[3] & 0x7f) == 56 {
        game.lease()
            .edit_actor(key, Le, |actor| actor.pre_instruction |= 2);
        set_instruction_list(game);
    }
    if game.lease().resolve_actor(key, Le).variables[4] != 0 {
        game.lease().edit_actor(key, Le, |actor| {
            (actor.variables[4], actor.pre_instruction) = (0, actor.pre_instruction & 1)
        });
        set_instruction_list(game);
    }
}

/// Set waver instruction list ($A3:87FE) from the pointers at $A3:86DB, when it changes. The
/// unmasked index reads linearly past the table, as the original did.
fn set_instruction_list<H: LeaseActors>(game: &mut H) {
    let (key, list) = {
        let runtime = game.lease();
        let key = current(&runtime);
        (key, runtime.resolve_actor(key, Le).pre_instruction)
    };
    if list != game.lease().resolve_actor(key, Le).variables[2] {
        game.lease()
            .edit_actor(key, Le, |actor| actor.variables[2] = list);
        let mut runtime = game.lease();
        let instruction = table(&runtime, 0xa3_86db, list as isize);
        runtime.edit_actor(key, Le, |actor| actor.restart_animation(instruction));
    }
}
