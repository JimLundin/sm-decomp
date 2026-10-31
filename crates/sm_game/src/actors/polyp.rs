//! Polyp, lava vents shooting rocks at Samus (bank $A2:B51A-$A2:B601). Was `lavaquake`.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess};
use crate::actors::api::{self, Axis};
use crate::actors::combat::current;
use crate::actors::host::{ActorLease, ActorRuntime as _, LeaseActors};
use crate::enemy_projectiles::host::Effects as _;
use crate::player_projectiles::content::table;
use crate::ram::words as W;

const WAIT_FOR_SAMUS_TO_GET_NEAR: u16 = 0xb596;
const SHOOT_ROCK: u16 = 0xb5b2;
const COOLDOWN: u16 = 0xb5ea;

fn random_entry(runtime: &mut impl ActorLease, address: u32, mask: u16) -> u16 {
    let index = (runtime.next_random() & mask) >> 1;
    table(runtime, address, index as isize)
}

/// Initialisation AI - enemy $D1FF (polyp) ($A2:B570): also seeds the random number with 11h.
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut runtime = game.lease();
    let key = current(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.restart_animation(0xb51a);
        actor.variables[0] = WAIT_FOR_SAMUS_TO_GET_NEAR;
    });
    runtime.write_slice_le(W::RANDOM_NUMBER, 0x11);
}

/// Main AI ($A2:B58F): Samus within 40h px, a random rock ($A2:B550/30), random cooldown ($A2:B520).
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) {
    crate::probe::hit("polyp.main_ai");
    let mut runtime = game.lease();
    let key = current(&runtime);
    match runtime.resolve_actor(key, Native).variables[0] {
        WAIT_FOR_SAMUS_TO_GET_NEAR => {
            let mut near = |axis| api::player_within(&mut runtime, key, axis, 0x40);
            if near(Axis::X) && near(Axis::Y) {
                runtime.edit_actor(key, Native, |actor| actor.variables[0] = SHOOT_ROCK);
            }
        }
        SHOOT_ROCK => {
            let x_velocity = random_entry(&mut runtime, 0xa2_b550, 0x1e);
            runtime.write_slice_le(W::EPROJ_INIT_PARAM_2, x_velocity);
            let y_speed_index = random_entry(&mut runtime, 0xa2_b530, 0x1e);
            let k = runtime.word(W::CUR_ENEMY_INDEX);
            runtime.foreign(|game| game.spawn_projectile(y_speed_index, k, 0xbd5a));
            let key = current(&runtime);
            runtime.edit_actor(key, Native, |actor| actor.variables[0] = COOLDOWN);
            let cooldown = random_entry(&mut runtime, 0xa2_b520, 0xe);
            runtime.edit_actor(key, Native, |actor| actor.variables[1] = cooldown);
        }
        COOLDOWN => runtime.edit_actor(key, Native, |actor| {
            actor.variables[1] = actor.variables[1].wrapping_sub(1);
            if (actor.variables[1] & 0x8000) != 0 {
                actor.variables[0] = WAIT_FOR_SAMUS_TO_GET_NEAR;
            }
        }),
        _ => runtime.unreachable(),
    }
}
