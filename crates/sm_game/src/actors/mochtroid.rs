//! Mochtroid, the fake Metroid of Maridia, which drifts after Samus and
//! drains her a little at a time (bank $A3:A725-$A3:A97B).
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::api::{self, Axis, DeathHandling, MovementResult, SlopePolicy};
use crate::actors::combat;
use crate::actors::host::{ActorLease, LeaseActors};
use crate::enemy_projectiles::host::Effects as _;
use crate::session_access::SessionAccess as _;

/// Mochtroid's actor words, by `variables` lane; its function index is the
/// pre-instruction word.
mod lane {
    pub const X_SUBVELOCITY: usize = 0;
    pub const Y_SUBVELOCITY: usize = 2;
    pub const SHAKE_TIMER: usize = 4;
}

/// Mochtroid function indices ($A3:A7A4).
pub mod function {
    pub const NOT_TOUCHING_SAMUS: u16 = 0;
    pub const TOUCHING_SAMUS: u16 = 1;
    /// Unused.
    pub const SHAKING: u16 = 2;
}

/// Bank $A3 Mochtroid instruction lists.
pub mod instruction_list {
    pub const NOT_TOUCHING_SAMUS: u16 = 0xa745;
    pub const TOUCHING_SAMUS: u16 = 0xa759;
}

/// $7E:7802: the instruction list last set, and $7E:8000: the damage timer.
const INSTRUCTION_LIST: usize = 0x7802;
const DAMAGE_TIMER: usize = 0x8000;
/// $A3:A76D: the unused shake's X and Y velocities, by shake timer.
const SHAKE_VELOCITIES: [(i16, i16); 4] = [(2, 0), (0, -2), (-2, 0), (0, 2)];
const SHAKE_BITS: u16 = 6;
/// Velocities are capped at 3.0 px a frame.
const MAX_SPEED: i32 = 0x30000;
/// Touching, Samus loses health every 80 frames; every 8 frames above 30
/// health the drain sound plays (2Dh, library 3).
const DAMAGE_FRAMES: u16 = 80;
const DRAIN_SOUND_FRAMES: u16 = 7;
const DRAIN_SOUND_HEALTH: u16 = 30;
const DRAIN_SOUND: u16 = 0x2d;
const SIGN: u16 = 0x8000;

/// Set the instruction list, if not already set.
fn set_instruction_list(runtime: &mut impl ActorLease, instruction: u16) {
    let key = combat::current(runtime);
    let cache = INSTRUCTION_LIST + usize::from(key.byte_index());
    if runtime.lease_native_word(cache) != instruction {
        runtime.write_slice_native(cache, instruction);
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction = instruction;
            actor.instruction_delay = 1;
            actor.timer = 0;
        });
    }
}
/// Accelerate towards Samus on `axis`, while chasing, or step 1 px towards
/// her, then move, stopping at blocks while chasing.
fn track_axis(runtime: &mut impl ActorLease, key: Key, axis: Axis, chasing: bool) {
    let (target, low) = match axis {
        Axis::X => (runtime.session().player.x, lane::X_SUBVELOCITY),
        Axis::Y => (runtime.session().player.y, lane::Y_SUBVELOCITY),
    };
    runtime.edit_actor(key, Native, |actor| {
        let position = match axis {
            Axis::X => actor.x.whole,
            Axis::Y => actor.y.whole,
        };
        let distance = (position as i16).wrapping_sub(target as i16);
        let speed = if chasing {
            let previous = ((u32::from(actor.variables[low + 1]) << 16)
                | u32::from(actor.variables[low])) as i32;
            previous
                .wrapping_sub(i32::from(distance >> 2) << 8)
                .clamp(-MAX_SPEED, MAX_SPEED)
        } else {
            -i32::from(distance.signum()) << 16
        };
        actor.variables[low] = speed as u16;
        actor.variables[low + 1] = (speed >> 16) as u16;
    });
    let current = combat::current(runtime);
    let amount = {
        let actor = runtime.resolve_actor(key, Native);
        ((u32::from(actor.variables[low + 1]) << 16) | u32::from(actor.variables[low])) as i32
    };
    let blocked = match axis {
        Axis::X => api::move_horizontal(runtime, current, amount, SlopePolicy::Ignore),
        Axis::Y => api::move_vertical(runtime, current, amount),
    } == MovementResult::Blocked;
    if chasing && blocked {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[low] = 0;
            actor.variables[low + 1] = 0;
        });
    }
}
/// Initialisation AI - Mochtroid ($A3:A77D).
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut runtime = game.lease();
    let key = combat::current(&runtime);
    runtime.edit_actor(key, Native, |actor| actor.layer = 2);
    set_instruction_list(&mut runtime, instruction_list::NOT_TOUCHING_SAMUS);
    runtime.edit_actor(key, Native, |actor| {
        actor.pre_instruction = function::NOT_TOUCHING_SAMUS
    });
}
/// Main AI - Mochtroid ($A3:A790): run its function, then reset it to not
/// touching Samus. Was `advance`.
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut runtime = game.lease();
    let key = combat::current(&runtime);
    let function = runtime
        .resolve_actor(key, Native)
        .pre_instruction
        .wrapping_mul(2)
        >> 1;
    runtime.edit_actor(key, Native, |actor| {
        actor.pre_instruction = function::NOT_TOUCHING_SAMUS
    });
    // The doubled u16 index discards bit 15 before the original three-entry check.
    let function = [
        function::NOT_TOUCHING_SAMUS,
        function::TOUCHING_SAMUS,
        function::SHAKING,
    ][usize::from(function)];
    if function == function::SHAKING {
        // Mochtroid function index 2 - shaking ($A3:A88F), unused.
        runtime.edit_actor(key, Native, |actor| {
            let (x, y) = SHAKE_VELOCITIES
                [usize::from((actor.variables[lane::SHAKE_TIMER] & SHAKE_BITS) >> 1)];
            actor.x.whole = actor.x.whole.wrapping_add(x as u16);
            actor.y.whole = actor.y.whole.wrapping_add(y as u16);
            actor.variables[..lane::SHAKE_TIMER].fill(0);
            let previous = actor.variables[lane::SHAKE_TIMER];
            actor.variables[lane::SHAKE_TIMER] = previous.wrapping_sub(1);
            if previous == 1 {
                actor.pre_instruction = function::NOT_TOUCHING_SAMUS;
            }
        });
    } else {
        // Mochtroid function index 0 - not touching Samus ($A3:A7AA): chase,
        // vertically first; index 1 - touching Samus ($A3:A8C8): follow.
        let axes = if function == function::NOT_TOUCHING_SAMUS {
            [Axis::Y, Axis::X]
        } else {
            [Axis::X, Axis::Y]
        };
        for axis in axes {
            track_axis(
                &mut runtime,
                key,
                axis,
                function == function::NOT_TOUCHING_SAMUS,
            );
        }
    }
    if function != function::TOUCHING_SAMUS {
        set_instruction_list(&mut runtime, instruction_list::NOT_TOUCHING_SAMUS);
    }
}
/// Enemy touch - Mochtroid ($A3:A953): latch on and drain Samus.
pub fn touch<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut runtime = game.lease();
    let key = combat::current(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.pre_instruction = function::TOUCHING_SAMUS
    });
    set_instruction_list(&mut runtime, instruction_list::TOUCHING_SAMUS);
    let timer = DAMAGE_TIMER + usize::from(key.byte_index());
    let next = runtime.lease_native_word(timer).wrapping_add(1);
    runtime.write_slice_native(timer, next);
    let mut damage = runtime.session().player.contact.contact_attack != 0;
    if !damage {
        if runtime.inputs().random_enemy_counter & DRAIN_SOUND_FRAMES == DRAIN_SOUND_FRAMES
            && runtime
                .session()
                .inventory
                .health
                .current
                .wrapping_sub(DRAIN_SOUND_HEALTH)
                & SIGN
                == 0
        {
            runtime.queue_sound(3, 6, DRAIN_SOUND);
        }
        damage = runtime.lease_native_word(timer).wrapping_sub(DAMAGE_FRAMES) & SIGN == 0;
        if damage {
            runtime.write_slice_native(timer, 0);
        }
    }
    if damage {
        combat::contact_reaction(&mut runtime, DeathHandling::Finalize);
        runtime.session_mut().player.invincibility = 0;
        runtime.session_mut().player.knockback_timer = 0;
    }
}
