//! The flies: Mellow, Mella and Memu (bank $A2:AFF3-$A2:B2BB). The function is the
//! pre-instruction word; lanes: cooldown, X and Y velocity, target Y, circle angle.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::api::{self, Axis};
use crate::actors::combat::current;
use crate::actors::host::{ActorLease, LeaseActors};
use crate::projectile_geometry::SINE_COSINE;

const CLOCKWISE: u16 = 0xb14e;
const ANTICLOCKWISE: u16 = 0xb17c;
const ATTACK: u16 = 0xb1aa;
const RETREAT: u16 = 0xb1d2;

/// Move fly according to angle or velocities ($A2:B090, $A2:B0DC); its new Y position.
fn move_fly(runtime: &mut impl ActorLease, key: Key, circling: bool) -> u16 {
    let angle = usize::from(runtime.resolve_actor(key, Native).variables[4] >> 1);
    for axis in 0..2 {
        let velocity = match circling {
            true => SINE_COSINE[angle + if axis == 0 { 64 } else { 0 }] as u16,
            false => runtime.resolve_actor(key, Native).variables[axis + 1],
        };
        runtime.edit_actor(key, Native, |actor| {
            let position = if axis == 0 {
                &mut actor.x
            } else {
                &mut actor.y
            };
            position.advance_8_8(i16::from_le_bytes(velocity.to_ne_bytes()));
        });
    }
    runtime.resolve_actor(key, Native).y.whole
}

/// Set fly to attack Samus ($A2:B129).
fn attack(runtime: &mut impl ActorLease, key: Key) {
    let angle = usize::from(api::angle_to_player(runtime, key));
    for (variable, index, scale) in [(1, angle + 64, 2u16), (2, angle, 4)] {
        let speed = scale.wrapping_mul(SINE_COSINE[index] as u16);
        runtime.edit_actor(key, Native, |actor| actor.variables[variable] = speed);
    }
    let target_y = runtime.session().player.y;
    runtime.edit_actor(key, Native, |actor| {
        (actor.variables[3], actor.pre_instruction) = (target_y, ATTACK)
    });
}

/// Initialisation AI - enemies $D0FF, $D13F and $D17F (flies) ($A2:B06B).
pub fn initialize<H: LeaseActors>(game: &mut H) {
    let mut runtime = game.lease();
    runtime.edit_actor(current(&runtime), Native, |actor| {
        (actor.variables[4], actor.pre_instruction, actor.instruction) = (0, CLOCKWISE, 0xb013);
        (actor.spritemap, actor.instruction_delay) = (0x804d, 1);
    });
}

/// Fly function - idle movement - clockwise and anti-clockwise circle ($A2:B14E, $A2:B17C):
/// after the cooldown, attack when Samus is within 70h pixel columns.
pub fn circle<H: LeaseActors>(k: u16, anticlockwise: bool, game: &mut H) {
    let mut runtime = game.lease();
    let key = Key::from_legacy(k);
    let cooldown = runtime.resolve_actor(key, Native).variables[0];
    if cooldown != 0 {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[0] = cooldown.wrapping_sub(1)
        });
    } else if ((api::player_delta(&runtime, key, Axis::X) as i16).wrapping_abs() as u16) < 0x70 {
        return attack(&mut runtime, key);
    }
    move_fly(&mut runtime, key, true);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[4] =
            actor.variables[4].wrapping_add(if anticlockwise { 0xffe0 } else { 32 }) & 0x1ff;
        if actor.variables[4] == 0 {
            actor.pre_instruction = if anticlockwise {
                CLOCKWISE
            } else {
                ANTICLOCKWISE
            };
        }
    });
}

/// Fly function - attack Samus and retreat ($A2:B1AA, $A2:B1D2): dive to Samus's Y, then back.
pub fn dive<H: LeaseActors>(k: u16, retreating: bool, game: &mut H) {
    let mut runtime = game.lease();
    let key = Key::from_legacy(k);
    let y = move_fly(&mut runtime, key, false);
    runtime.edit_actor(key, Native, |actor| {
        if retreating {
            actor.variables[0] = actor.variables[0].wrapping_sub(1);
            if (actor.variables[0] & 0x8000) != 0 {
                (actor.variables[0], actor.pre_instruction) = (24, CLOCKWISE);
            }
            return;
        }
        actor.variables[0] = actor.variables[0].wrapping_add(1);
        // Past the target Y in the direction of travel.
        if (y >= actor.variables[3]) == ((actor.variables[2] & 0x8000) == 0) {
            actor.variables[2] = actor.variables[2].wrapping_neg();
            actor.pre_instruction = RETREAT;
        }
    });
}
