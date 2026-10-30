//! Multiviola, fireballs that bounce off walls (bank $A2:B2BC-$A2:B519).
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey};
use crate::actors::api::{self, MovementResult::Blocked, SlopePolicy};
use crate::actors::combat::current;
use crate::actors::host::{BossEffects, LeaseActors};

// The velocity from the angle and speed parameters, and the lease the frame continues on.
fn set_velocity<H: LeaseActors + BossEffects + ?Sized>(game: &mut H) -> (ActorKey, H::Lease<'_>) {
    let runtime = game.lease();
    let key = current(&runtime);
    let [angle, speed] = runtime.resolve_actor(key, Native).parameters;
    drop(runtime);
    let velocity = game.actor_direction_vector(angle, u16::from(speed.to_ne_bytes()[0]));
    let mut runtime = game.lease();
    runtime.edit_actor(key, Native, |actor| {
        let [_, _, x_velocity, x_subvelocity, y_velocity] = &mut actor.variables;
        (*x_velocity, *x_subvelocity) = ((velocity.x >> 16) as u16, velocity.x as u16);
        (*y_velocity, actor.pre_instruction) = ((velocity.y >> 16) as u16, velocity.y as u16);
    });
    (key, runtime)
}

/// Initialisation AI - enemy $D1BF (multiviola) ($A2:B3E0).
pub fn initialize<H: LeaseActors + BossEffects + ?Sized>(game: &mut H) {
    let (key, mut runtime) = set_velocity(game);
    runtime.edit_actor(key, Native, |actor| actor.instruction = 0xb2dc);
}

/// Main AI ($A2:B40F): move along X then Y, negating each velocity whose angle + offset has
/// bit 7 clear, and reflect the angle off the first block.
pub fn main_ai<H: LeaseActors + BossEffects + ?Sized>(game: &mut H) {
    let (key, mut runtime) = set_velocity(game);
    for (horizontal, offset, reflection) in [(true, 0x40, 0x40), (false, 0x80, 0xc0)] {
        let amount = runtime.edit_actor(key, Native, |actor| {
            let negate = (actor.parameters[0].wrapping_add(offset) & 0x80) == 0;
            let [_, _, x_velocity, x_subvelocity, y_velocity] = &mut actor.variables;
            let (high, low) = match horizontal {
                true => (x_velocity, x_subvelocity),
                false => (y_velocity, &mut actor.pre_instruction),
            };
            if negate {
                // Each word negates on its own, so a zero low word still flips the high word.
                (*high, *low) = (!*high, low.wrapping_neg());
            }
            ((u32::from(*high) << 16) | u32::from(*low)) as i32
        });
        let moving = current(&runtime);
        let result = match horizontal {
            true => api::move_horizontal(&mut runtime, moving, amount, SlopePolicy::Ignore),
            false => api::move_vertical(&mut runtime, moving, amount),
        };
        if result == Blocked {
            runtime.edit_actor(key, Native, |actor| actor.parameters[0] ^= reflection);
            break;
        }
    }
}
