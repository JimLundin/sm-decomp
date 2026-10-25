//! Hopping blobs share flight, directional animation, and landing transitions.
#![forbid(unsafe_code)]
use crate::game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::game::actors::api::{self, Axis, MovementResult, SlopePolicy};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;

fn selected(runtime: &Runtime<'_>) -> (Key, usize) {
    let raw = runtime.word(0xe54);
    (Key::from_legacy(raw), 0x7800 + usize::from(raw))
}
fn rom_bytes<const N: usize>(runtime: &Runtime<'_>, address: u32, offset: usize) -> [u8; N] {
    let image = runtime.native_image();
    let start = crate::rom::lorom_offset(address) as usize + offset;
    core::array::from_fn(|i| image.read(start + i))
}
fn rom_word(runtime: &Runtime<'_>, address: u32, offset: u16) -> u16 {
    u16::from_ne_bytes(rom_bytes(runtime, address, usize::from(offset)))
}
fn animate(runtime: &mut Runtime<'_>, instruction: u16) {
    let (key, _) = selected(runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.restart_animation(instruction);
    });
}
pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (key, at) = selected(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.spritemap = 0x804d;
        actor.variables[0] = 0;
    });
    animate(&mut runtime, 0x99ad);
    runtime.write_slice_native(at, 0);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[3] = 0x9b65;
        actor.variables[2] = actor.parameters[0];
    });
    runtime.write_slice_native(at + 8, 0);
}
pub(crate) fn choose_direction(k: u16, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let at = 0x7800 + usize::from(k);
    runtime.write_slice_native(at + 4, 1);
    let (live, _) = selected(&runtime);
    if api::player_delta(&runtime, live, Axis::X) & 0x8000 == 0 {
        runtime.write_slice_native(at + 4, 0);
    }
    if runtime.native_word(at + 8) != 0 {
        runtime.write_slice_native(at + 4, runtime.native_word(at + 10));
    }
    runtime.write_slice_native(at + 8, 0);
}
pub(crate) fn choose_jump(k: u16, mut random: u16, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(k);
    let mut kind = runtime.native_word(0x7800 + usize::from(k));
    if kind.wrapping_sub(3) & 0x8000 != 0 {
        if kind == 0 {
            random &= 1;
        }
        kind = if random.wrapping_sub(2) & 0x8000 == 0 {
            2
        } else {
            random
        };
    }
    let offset = kind.wrapping_mul(8);
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = offset);
    let phase = rom_word(&runtime, 0xa29a0d, offset);
    runtime.edit_actor(key, Native, |actor| actor.variables[4] = phase);
}
pub(crate) fn prepare_velocity(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (key, at) = selected(&runtime);
    let mut offset = runtime.resolve_actor(key, Native).pre_instruction;
    let (mut speed, mut height) = (0u16, 0u16);
    loop {
        speed = speed.wrapping_add(rom_word(&runtime, 0xa29a09, offset));
        height = height.wrapping_add(rom_word(&runtime, 0xa2838f, 8 * (speed >> 8) + 1));
        offset = runtime.resolve_actor(key, Native).pre_instruction;
        let target = rom_word(&runtime, 0xa29a07, offset).swap_bytes();
        if target.wrapping_sub(height) & 0x8000 != 0 {
            break;
        }
    }
    runtime.edit_actor(key, Native, |actor| actor.variables[1] = speed);
    runtime.write_slice_native(at + 6, 0);
    runtime.write_slice_native(at + 14, speed >> 1);
    runtime.write_slice_native(at + 12, (speed >> 1).wrapping_add((speed >> 1) >> 1));
}
fn flight_animation(runtime: &mut Runtime<'_>, falling: bool) {
    let (key, at) = selected(runtime);
    let speed = runtime.resolve_actor(key, Native).variables[1];
    let low = runtime.native_word(at + 14);
    let high = runtime.native_word(at + 12);
    let level = if falling {
        if (speed.wrapping_sub(low) as i16) < 0 {
            0
        } else if (speed.wrapping_sub(high) as i16) < 0 {
            1
        } else {
            2
        }
    } else if (speed.wrapping_sub(high) as i16) >= 0 {
        2
    } else if (speed.wrapping_sub(low) as i16) >= 0 {
        1
    } else {
        0
    };
    let left = runtime.native_word(at + 4) != 0;
    animate(
        runtime,
        [[0x99f5, 0x99ef, 0x99e9], [0x99f5, 0x99fb, 0x9a01]][usize::from(left ^ falling)][level],
    );
}
pub(crate) fn flight(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (key, at) = selected(&runtime);
    let mut speed = runtime.resolve_actor(key, Native).variables[1];
    if speed.wrapping_sub(0x4000) & 0x8000 == 0 {
        speed = 0x4000;
    }
    let mut offset = 8 * (speed >> 8);
    if runtime.native_word(at + 6) == 0 {
        offset = offset.wrapping_add(4);
    }
    let (live, _) = selected(&runtime);
    let amount =
        u32::from_ne_bytes(rom_bytes(&runtime, 0xa0cbc7, usize::from(offset >> 2) * 4)) as i32;
    if api::move_vertical(&mut runtime, live, amount) == MovementResult::Blocked {
        if runtime.native_word(at + 6) != 0 {
            runtime.edit_actor(key, Native, |actor| actor.variables[3] = 0x9b65);
        } else {
            runtime.write_slice_native(at + 8, 1);
            runtime.write_slice_native(at + 10, runtime.native_word(at + 4) ^ 1);
            runtime.write_slice_native(at, 4);
            runtime.edit_actor(key, Native, |actor| actor.variables[4] = 0x9d98);
        }
        runtime.write_slice_native(at + 2, 0);
        return;
    }
    let offset = runtime.resolve_actor(key, Native).pre_instruction;
    let falling = runtime.native_word(at + 6) != 0;
    flight_animation(&mut runtime, falling);
    let acceleration = rom_word(&runtime, 0xa29a0b, offset);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[1] = if falling {
            actor.variables[1].wrapping_add(acceleration)
        } else {
            actor.variables[1].wrapping_sub(acceleration)
        };
    });
    if runtime.resolve_actor(key, Native).variables[1] & 0x8000 != 0 {
        runtime.write_slice_native(at + 6, 1);
        runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0);
    }
    let offset = runtime.resolve_actor(key, Native).pre_instruction;
    let mut horizontal = rom_word(&runtime, 0xa29a09, offset) >> 8;
    if runtime.native_word(at + 4) != 0 {
        horizontal = horizontal.wrapping_neg();
    }
    let (live, _) = selected(&runtime);
    if api::move_horizontal(
        &mut runtime,
        live,
        (horizontal as i16 as i32) << 16,
        SlopePolicy::Ignore,
    ) == MovementResult::Blocked
    {
        runtime.write_slice_native(at + 8, 1);
        runtime.write_slice_native(at + 10, runtime.native_word(at + 4) ^ 1);
        runtime.write_slice_native(at + 2, 0);
        runtime.write_slice_native(at, 4);
        runtime.edit_actor(key, Native, |actor| actor.variables[4] = 0x9d98);
    }
}
#[derive(Clone, Copy)]
pub(crate) enum Landing {
    Animate(u16),
    Restart,
    Repeat,
}
pub(crate) fn land(kind: Landing, game: &mut Game) {
    flight(game);
    let mut runtime = Runtime::new(game);
    let (key, at) = selected(&runtime);
    let interrupted = !matches!(kind, Landing::Repeat) && runtime.native_word(at + 8) != 0;
    if !interrupted && runtime.native_word(at + 2) != 0 {
        return;
    }
    if matches!(kind, Landing::Restart) && !interrupted {
        runtime.write_slice_native(at, 0);
        runtime.edit_actor(key, Native, |actor| actor.variables[3] = 0x9b65);
    }
    runtime.write_slice_native(at + 2, 0);
    if matches!(kind, Landing::Repeat) {
        runtime.write_slice_native(at, 3);
        runtime.edit_actor(key, Native, |actor| actor.variables[3] = 0x9b65);
    }
    animate(
        &mut runtime,
        match kind {
            Landing::Animate(instruction) => instruction,
            _ => 0x99d5,
        },
    );
}
pub(crate) fn fall(game: &mut Game) -> bool {
    let mut runtime = Runtime::new(game);
    let (key, _) = selected(&runtime);
    let offset = runtime.resolve_actor(key, Native).pre_instruction;
    let high = rom_word(&runtime, 0xa29a0b, offset) >> 8;
    let low = u16::from(rom_bytes::<1>(&runtime, 0xa29a0b, usize::from(offset))[0]) << 8;
    let (live, _) = selected(&runtime);
    api::move_vertical(
        &mut runtime,
        live,
        ((u32::from(high) << 16) | u32::from(low)) as i32,
    ) == MovementResult::Blocked
}
