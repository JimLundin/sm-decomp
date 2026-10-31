//! Fune and namihe, the wall-mounted fire spitters (bank $A8:959D-$A8:97B3):
//! a fune spits on a timer, a namihe when Samus comes into its vertical range.
//! The saved instruction list pointers word remains RAM-authoritative.
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::api::{Axis, player_within};
use crate::actors::host::{ActorLease, ActorRuntime as _, LeaseActors};
use crate::enemy_projectiles::host::Effects as _;
use crate::enemy_script::EnemyScript;
use crate::ram::words;

/// The actor words fune and namihe use, by `variables` lane.
mod lane {
    /// The instruction list pointers pointer, into $A8:96D3.
    pub const LISTS: usize = 0;
    pub const FUNCTION: usize = 1;
    pub const Y_PROXIMITY: usize = 2;
    /// 0 for a fune, nonzero for a namihe.
    pub const VARIANT: usize = 3;
    pub const COOLDOWN_TIMER: usize = 4;
}

/// Bank $A8 fune and namihe functions, stored in the function lane.
pub mod function {
    pub const WAIT_FOR_TIMER: u16 = 0x9737;
    pub const WAIT_FOR_SAMUS: u16 = 0x975c;
    pub const FUNE_ACTIVE: u16 = 0x978e;
    pub const NAMIHE_ACTIVE: u16 = 0x978f;
}

/// Bank $A8 instructions their scripts call.
pub mod instruction {
    pub const QUEUE_SPIT_SOUND: u32 = 0xa8_9625;
    pub const SPAWN_LEFTWARDS_NAMIHE_FIREBALL: u32 = 0xa8_9631;
    pub const SPAWN_RIGHTWARDS_NAMIHE_FIREBALL: u32 = 0xa8_964a;
    pub const SPAWN_LEFTWARDS_FUNE_FIREBALL: u32 = 0xa8_9663;
    pub const SPAWN_RIGHTWARDS_FUNE_FIREBALL: u32 = 0xa8_967c;
    pub const FINISH_ACTIVITY: u32 = 0xa8_9695;
    pub const FINISH_ACTIVITY_2: u32 = 0xa8_96b4;
}

const BANK: u8 = 0xa8;
/// $A8:96D3: the instruction list pointers, per variant: active facing left
/// and right, then idle facing left and right. Each variant starts idle.
const FUNE_IDLE_LEFT: u16 = 0x96d7;
const NAMIHE_IDLE_LEFT: u16 = 0x96df;
const FACING_RIGHT: u16 = 2;
const ACTIVE: u16 = 4;
/// Parameter 1's low nibble selects the variant, its high nibble faces right.
const VARIANT_BITS: u16 = 0xf;
const RIGHT_BITS: u16 = 0xf0;
/// $7E:7800: the namihe's saved instruction list pointers pointer, and
/// $7E:1995: enemy projectile initialisation parameter 1, never read.
const SAVED_LISTS: usize = 0x7800;
const PROJECTILE_PARAMETER: usize = 0x1995;
/// The namihe and fune fireball projectiles, and the fune spit sound (1Fh,
/// library 2, at most 9 queued).
const NAMIHE_FIREBALL: u16 = 0xdfbc;
const FUNE_FIREBALL: u16 = 0xdfca;
const SPIT_SOUND: u16 = 0x1f;

fn selected(runtime: &impl ActorLease) -> Key {
    Key::from_legacy(runtime.word(words::CUR_ENEMY_INDEX))
}
/// Set fune / namihe instruction list ($A8:979B).
fn set_instruction_list(runtime: &mut impl ActorLease) {
    let key = selected(runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction_delay = 1;
        actor.timer = 0;
    });
    let table = runtime.resolve_actor(key, Native).variables[lane::LISTS];
    let instruction = runtime.lease_native_record(BANK, table).word(0);
    runtime.edit_actor(key, Native, |actor| actor.instruction = instruction);
}
/// Initialisation AI - fune / namihe ($A8:96E3).
pub fn initialize(runtime: &mut impl ActorLease) {
    let key = selected(runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[lane::LISTS] = FUNE_IDLE_LEFT;
        actor.variables[lane::FUNCTION] = function::WAIT_FOR_TIMER;
        actor.variables[lane::VARIANT] = actor.parameters[0] & VARIANT_BITS;
        if actor.variables[lane::VARIANT] != 0 {
            actor.variables[lane::LISTS] = NAMIHE_IDLE_LEFT;
            actor.variables[lane::FUNCTION] = function::WAIT_FOR_SAMUS;
        }
        if actor.parameters[0] & RIGHT_BITS != 0 {
            actor.variables[lane::LISTS] = actor.variables[lane::LISTS].wrapping_add(FACING_RIGHT);
        }
    });
    set_instruction_list(runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[lane::Y_PROXIMITY] = u16::from(actor.parameters[1].to_ne_bytes()[1]);
        // The cooldown time.
        actor.pre_instruction = u16::from(actor.parameters[0].to_ne_bytes()[1]);
        actor.variables[lane::COOLDOWN_TIMER] = 0;
    });
}
/// Main AI - fune / namihe ($A8:9730): run its function. Was `advance`.
pub fn main_ai(runtime: &mut impl ActorLease) {
    let key = selected(runtime);
    match runtime.resolve_actor(key, Native).variables[lane::FUNCTION] {
        // Fune / namihe function - wait for timer (fune) ($A8:9737).
        function::WAIT_FOR_TIMER => {
            let ready = runtime.edit_actor(key, Native, |actor| {
                actor.variables[lane::COOLDOWN_TIMER] =
                    actor.variables[lane::COOLDOWN_TIMER].wrapping_add(1);
                (actor.variables[lane::COOLDOWN_TIMER] as i16)
                    .wrapping_sub(actor.pre_instruction as i16)
                    >= 0
            });
            if ready {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[lane::LISTS] = actor.variables[lane::LISTS].wrapping_sub(ACTIVE)
                });
                set_instruction_list(runtime);
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[lane::FUNCTION] = function::FUNE_ACTIVE;
                    actor.variables[lane::COOLDOWN_TIMER] = 0;
                });
            }
        }
        // Fune / namihe function - wait for Samus to get near (namihe) ($A8:975C).
        function::WAIT_FOR_SAMUS => {
            let saved = SAVED_LISTS + usize::from(key.byte_index());
            let lists = runtime.resolve_actor(key, Native).variables[lane::LISTS];
            runtime.write_slice_native(saved, lists);
            let live = selected(runtime);
            let range = runtime.resolve_actor(live, Native).variables[lane::Y_PROXIMITY];
            if player_within(runtime, live, Axis::Y, range) {
                let table = runtime.lease_native_word(saved);
                runtime.edit_actor(key, Native, |actor| actor.variables[lane::LISTS] = table);
                let table = table.wrapping_sub(ACTIVE);
                runtime.write_slice_native(saved, table);
                runtime.edit_actor(key, Native, |actor| actor.variables[lane::LISTS] = table);
                set_instruction_list(runtime);
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[lane::FUNCTION] = function::NAMIHE_ACTIVE
                });
            }
            let table = runtime.lease_native_word(saved);
            runtime.edit_actor(key, Native, |actor| actor.variables[lane::LISTS] = table);
        }
        function::FUNE_ACTIVE | function::NAMIHE_ACTIVE => {}
        _ => runtime.unreachable(),
    }
}
/// Run one of fune's and namihe's script instructions (one of [`instruction`]).
pub fn instruction<H: LeaseActors + ?Sized>(
    address: u32,
    script: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    use instruction::*;
    let mut runtime = game.lease();
    match address {
        QUEUE_SPIT_SOUND => runtime.queue_sound(2, 9, SPIT_SOUND),
        SPAWN_LEFTWARDS_NAMIHE_FIREBALL
        | SPAWN_RIGHTWARDS_NAMIHE_FIREBALL
        | SPAWN_LEFTWARDS_FUNE_FIREBALL
        | SPAWN_RIGHTWARDS_FUNE_FIREBALL => {
            let key = selected(&runtime);
            let parameter = runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes()[0];
            runtime.write_slice_le(PROJECTILE_PARAMETER, u16::from(parameter));
            let actor = runtime.word(words::CUR_ENEMY_INDEX);
            let right = u16::from(matches!(
                address,
                SPAWN_RIGHTWARDS_NAMIHE_FIREBALL | SPAWN_RIGHTWARDS_FUNE_FIREBALL
            ));
            let definition = if address < SPAWN_LEFTWARDS_FUNE_FIREBALL {
                NAMIHE_FIREBALL
            } else {
                FUNE_FIREBALL
            };
            runtime.foreign(|game| {
                game.spawn_projectile(right, actor, definition);
            });
        }
        FINISH_ACTIVITY | FINISH_ACTIVITY_2 => {
            let key = selected(&runtime);
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[lane::LISTS] = actor.variables[lane::LISTS].wrapping_add(ACTIVE);
                actor.variables[lane::FUNCTION] = function::WAIT_FOR_TIMER;
                if actor.variables[lane::VARIANT] != 0 {
                    actor.variables[lane::FUNCTION] = function::WAIT_FOR_SAMUS;
                }
            });
        }
        _ => {
            runtime.unreachable();
            return EnemyScript::Stop;
        }
    }
    script
}
