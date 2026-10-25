//! Erratic fireballs recompute their velocity and reflect on blocked movement.
#![forbid(unsafe_code)]
use crate::game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::game::actors::api::{self, MovementResult, SlopePolicy};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;

pub(crate) fn update(initialize: bool, game: &mut Game) {
    let (key, parameters) = {
        let runtime = Runtime::new(game);
        let key = Key::from_legacy(runtime.word(0xe54));
        (key, runtime.resolve_actor(key, Native).parameters)
    };
    let velocity = crate::game::session_runtime::actor_direction_vector(
        parameters[0],
        u16::from(parameters[1].to_ne_bytes()[0]),
        game,
    );
    let mut runtime = Runtime::new(game);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[2] = (velocity.x >> 16) as u16;
        actor.variables[3] = velocity.x as u16;
        actor.variables[4] = (velocity.y >> 16) as u16;
        actor.pre_instruction = velocity.y as u16;
    });
    if initialize {
        runtime.edit_actor(key, Native, |actor| actor.instruction = 0xb2dc);
        return;
    }
    for axis in 0..2 {
        let amount = runtime.edit_actor(key, Native, |actor| {
            let (high, low) = if axis == 0 {
                let [_, _, high, low, _] = &mut actor.variables;
                (high, low)
            } else {
                (&mut actor.variables[4], &mut actor.pre_instruction)
            };
            if actor.parameters[0].wrapping_add(if axis == 0 { 64 } else { 128 }) & 0x80 == 0 {
                // Preserve the separate word operations, including the zero-low-word quirk.
                *high = !*high;
                *low = low.wrapping_neg();
            }
            ((u32::from(*high) << 16) | u32::from(*low)) as i32
        });
        let live_key = Key::from_legacy(runtime.word(0xe54));
        let result = if axis == 0 {
            api::move_horizontal(&mut runtime, live_key, amount, SlopePolicy::Ignore)
        } else {
            api::move_vertical(&mut runtime, live_key, amount)
        };
        if result == MovementResult::Blocked {
            runtime.edit_actor(key, Native, |actor| {
                actor.parameters[0] ^= if axis == 0 { 0x40 } else { 0xc0 };
            });
            break;
        }
    }
}
