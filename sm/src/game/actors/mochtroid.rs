//! Mochtroid pursuit and contact tracking share ordered per-axis movement.
#![forbid(unsafe_code)]
use crate::game::actors::{api, combat};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use sm_game::enemy_projectiles::host::Effects;

fn animate(runtime: &mut Runtime<'_>, instruction: u16) {
    let key = combat::current(runtime);
    let cache = 0x7802 + usize::from(key.byte_index());
    if runtime.native_word(cache) != instruction {
        runtime.write_slice_native(cache, instruction);
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction = instruction;
            actor.instruction_delay = 1;
            actor.timer = 0;
        });
    }
}
fn track_axis(runtime: &mut Runtime<'_>, key: Key, axis: api::Axis, pursuit: bool) {
    let (target, low) = match axis {
        api::Axis::X => (runtime.session().player.x, 0),
        api::Axis::Y => (runtime.session().player.y, 2),
    };
    runtime.edit_actor(key, Native, |actor| {
        let position = match axis {
            api::Axis::X => actor.x.whole,
            api::Axis::Y => actor.y.whole,
        };
        let distance = (position as i16).wrapping_sub(target as i16);
        let speed = if pursuit {
            let previous = ((u32::from(actor.variables[low + 1]) << 16)
                | u32::from(actor.variables[low])) as i32;
            previous
                .wrapping_sub(i32::from(distance >> 2) << 8)
                .clamp(-0x30000, 0x30000)
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
        api::Axis::X => api::move_horizontal(runtime, current, amount, api::SlopePolicy::Ignore),
        api::Axis::Y => api::move_vertical(runtime, current, amount),
    } == api::MovementResult::Blocked;
    if pursuit && blocked {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[low] = 0;
            actor.variables[low + 1] = 0;
        });
    }
}
pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    runtime.edit_actor(key, Native, |actor| actor.layer = 2);
    animate(&mut runtime, 0xa745);
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0);
}
pub(crate) fn advance(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    let phase = runtime
        .resolve_actor(key, Native)
        .pre_instruction
        .wrapping_mul(2)
        >> 1;
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0);
    // The doubled u16 mode discards bit15 before the original three-entry check.
    let phase = [0, 1, 2][usize::from(phase)];
    if phase == 2 {
        runtime.edit_actor(key, Native, |actor| {
            let (x, y) = [(2i16, 0i16), (0, -2), (-2, 0), (0, 2)]
                [usize::from((actor.variables[4] & 6) >> 1)];
            actor.x.whole = actor.x.whole.wrapping_add(x as u16);
            actor.y.whole = actor.y.whole.wrapping_add(y as u16);
            actor.variables[..4].fill(0);
            let previous = actor.variables[4];
            actor.variables[4] = previous.wrapping_sub(1);
            if previous == 1 {
                actor.pre_instruction = 0;
            }
        });
    } else {
        let axes = if phase == 0 {
            [api::Axis::Y, api::Axis::X]
        } else {
            [api::Axis::X, api::Axis::Y]
        };
        for axis in axes {
            track_axis(&mut runtime, key, axis, phase == 0);
        }
    }
    if phase != 1 {
        animate(&mut runtime, 0xa745);
    }
}
pub(crate) fn touch(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 1);
    animate(&mut runtime, 0xa759);
    let counter = 0x8000 + usize::from(key.byte_index());
    runtime.write_slice_native(counter, runtime.native_word(counter).wrapping_add(1));
    let mut damage = runtime.session().player.contact.contact_attack != 0;
    if !damage {
        if runtime.inputs().random_enemy_counter & 7 == 7
            && runtime.session().inventory.health.current.wrapping_sub(30) & 0x8000 == 0
        {
            runtime.queue_sound(3, 6, 0x2d);
        }
        damage = runtime.native_word(counter).wrapping_sub(80) & 0x8000 == 0;
        if damage {
            runtime.write_slice_native(counter, 0);
        }
    }
    if damage {
        combat::contact_reaction(&mut runtime, api::DeathHandling::Finalize);
        runtime.session_mut().player.invincibility = 0;
        runtime.session_mut().player.knockback_timer = 0;
    }
}
