//! Waver (bank $A3:8687-$A3:88EF). Lanes: X subvelocity and velocity, current list index, sine
//! phase, spin finished flag; the pre-instruction word is the wanted list index (bit 0 facing
//! right, bit 1 spinning). Each step takes its own lease, as the original's did.
#![forbid(unsafe_code)]
use crate::actors::access::AccessMode::LittleEndian as Le;
use crate::actors::api::{self, MovementResult::Blocked, SlopePolicy};
use crate::actors::combat::current;
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content::table;

/// Instruction - set spin finished flag ($A3:86E3).
pub fn set_spin_finished(
    script: EnemyScript,
    runtime: &mut impl crate::actors::species::TableLease,
) -> EnemyScript {
    runtime.edit_actor(current(&*runtime), Le, |actor| actor.variables[4] = 1);
    script
}

/// Initialisation AI - enemy $D63F (waver) ($A3:86ED): parameter 1 bit 0 faces it right.
pub fn initialize(runtime: &mut impl crate::actors::species::TableLease) {
    runtime.edit_actor(current(&*runtime), Le, |actor| {
        let right = actor.parameters[0] & 1;
        // sign_extend8(0x8000) keeps 0x8000 as the subvelocity either way.
        (actor.variables[1], actor.variables[0]) = (if right == 0 { 0xfffe } else { 1 }, 0x8000);
        (actor.variables[2], actor.variables[4], actor.instruction) = (0, 0, 0x86a7);
        actor.pre_instruction = right;
    });
    set_instruction_list(runtime);
}

/// Main AI - enemy $D63F (waver) ($A3:874C): fly horizontally and turn at walls, bob on a sine,
/// and spin when the phase reaches 38h.
pub fn main_ai(runtime: &mut impl crate::actors::species::TableLease) {
    let (key, velocity) = {
        let key = current(&*runtime);
        let [low, high, ..] = runtime.resolve_actor(key, Le).variables;
        (key, ((u32::from(high) << 16) | u32::from(low)) as i32)
    };
    if api::move_horizontal(runtime, key, velocity, SlopePolicy::Ignore) == Blocked {
        runtime.edit_actor(key, Le, |actor| {
            // The ROM reverses the middle two bytes of the 16.16 velocity.
            let middle = (actor.variables[0] >> 8) | (actor.variables[1] << 8);
            actor.variables[1] = (middle.wrapping_neg() >> 8) as u8 as i8 as i16 as u16;
            actor.variables[0] = ((middle as u8 as i8 as i16).wrapping_neg() as u16) << 8;
            actor.pre_instruction = (actor.pre_instruction ^ 1) & 1;
        });
        set_instruction_list(runtime);
    } else {
        // The live selector is read before the sine; the state keeps its original key.
        let (vertical, phase) = {
            (
                current(&*runtime),
                runtime.resolve_actor(key, Le).variables[3] as u8 as u16,
            )
        };
        let velocity =
            i32::from(crate::actors::boss_lease::BossState::scaled_sine(runtime, phase, 4) as i16)
                << 16;
        let collided = api::move_vertical(runtime, vertical, velocity) == Blocked;
        runtime.edit_actor(key, Le, |actor| {
            let phase = actor.variables[3];
            actor.variables[3] = if collided {
                phase.wrapping_add(0x80) as u8 as u16
            } else {
                phase.wrapping_add(2)
            };
        });
    }
    if (runtime.resolve_actor(key, Le).variables[3] & 0x7f) == 56 {
        runtime.edit_actor(key, Le, |actor| actor.pre_instruction |= 2);
        set_instruction_list(runtime);
    }
    if runtime.resolve_actor(key, Le).variables[4] != 0 {
        runtime.edit_actor(key, Le, |actor| {
            (actor.variables[4], actor.pre_instruction) = (0, actor.pre_instruction & 1)
        });
        set_instruction_list(runtime);
    }
}

/// Set waver instruction list ($A3:87FE) from the pointers at $A3:86DB, when it changes. The
/// unmasked index reads linearly past the table, as the original did.
fn set_instruction_list(runtime: &mut impl crate::actors::species::TableLease) {
    let (key, list) = {
        let key = current(&*runtime);
        (key, runtime.resolve_actor(key, Le).pre_instruction)
    };
    if list != runtime.resolve_actor(key, Le).variables[2] {
        runtime.edit_actor(key, Le, |actor| actor.variables[2] = list);
        let instruction = table(&*runtime, 0xa3_86db, list as isize);
        runtime.edit_actor(key, Le, |actor| actor.restart_animation(instruction));
    }
}
