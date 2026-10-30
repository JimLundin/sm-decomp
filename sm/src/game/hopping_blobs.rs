//! Hopping blobs share flight, directional animation, and landing transitions.
#![forbid(unsafe_code)]
use crate::game::actors::api::{self, Axis, MovementResult, SlopePolicy};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use sm_game::enemy_projectiles::host::Effects;

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
// Shared by Beetom and Sidehopper. The table uses odd, native word reads
// at a widened phase offset; it is not a wrapping bank-address calculation.
pub(crate) fn initial_hop_phase(height: u16, step: u16, image: crate::rom::RomImage) -> u16 {
    let (mut phase, mut distance) = (0u16, 0u16);
    loop {
        phase = phase.wrapping_add(step);
        let at = crate::rom::lorom_offset(0xa2838f) as usize + 8 * usize::from(phase) + 1;
        distance = distance.wrapping_add(u16::from_ne_bytes([image.read(at), image.read(at + 1)]));
        if distance.wrapping_sub(height) & 0x8000 == 0 {
            return phase;
        }
    }
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
fn random_kind(runtime: &mut Runtime<'_>) -> u16 {
    runtime.next_random();
    let (key, _) = selected(runtime);
    runtime
        .resolve_actor(key, Native)
        .frame
        .wrapping_add(runtime.word(0x5e5))
        & 7
}

fn begin_jump(runtime: &mut Runtime<'_>) {
    let raw = runtime.word(0xe54);
    let key = Key::from_legacy(raw);
    let waiting = runtime.edit_actor(key, Native, |actor| {
        actor.variables[2] = actor.variables[2].wrapping_sub(1);
        actor.variables[2] & 0x8000 == 0
    });
    if waiting {
        return;
    }
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[3] = 0x9b81;
        actor.variables[2] = actor.parameters[0];
    });
    runtime.write_slice_native(0x7802 + usize::from(raw), 1);
    if runtime
        .native_word(0x7800 + usize::from(raw))
        .wrapping_sub(3)
        & 0x8000
        != 0
    {
        let (live, at) = selected(runtime);
        let distance = runtime.resolve_actor(live, Native).parameters[1];
        let close = api::player_within(runtime, live, Axis::X, distance);
        runtime.write_slice_native(at, u16::from(close));
    }
    // Direction and jump kind retain this identity across the random callback.
    let raw = runtime.word(0xe54);
    choose_direction(raw, runtime);
    let random = random_kind(runtime);
    choose_jump(raw, random, runtime);
    prepare_velocity(runtime);
}

/// False preserves the bank caller's existing invalid-dispatch diagnostic.
pub(crate) fn advance(game: &mut Game) -> bool {
    let mut runtime = Runtime::new(game);
    let (key, _) = selected(&runtime);
    match runtime.resolve_actor(key, Native).variables[3] {
        0x9b65 => begin_jump(&mut runtime),
        0x9b81 => {
            let (key, at) = selected(&runtime);
            match runtime.resolve_actor(key, Native).variables[4] {
                0x9d0b => land(Landing::Animate(0x99d5), &mut runtime),
                0x9d2b => land(Landing::Animate(0x99c1), &mut runtime),
                0x9d4b => land(Landing::Animate(0x99ad), &mut runtime),
                0x9d6b => land(Landing::Restart, &mut runtime),
                0x9dcd => land(Landing::Repeat, &mut runtime),
                0x9d98 => {
                    if fall(&mut runtime) {
                        let random = random_kind(&mut runtime);
                        runtime.write_slice_native(at, (random & 1) + 5);
                        runtime.edit_actor(key, Native, |actor| actor.variables[3] = 0x9b65);
                    }
                }
                _ => return false,
            }
        }
        _ => return false,
    }
    true
}

fn choose_direction(k: u16, runtime: &mut Runtime<'_>) {
    let at = 0x7800 + usize::from(k);
    runtime.write_slice_native(at + 4, 1);
    let (live, _) = selected(runtime);
    if api::player_delta(runtime, live, Axis::X) & 0x8000 == 0 {
        runtime.write_slice_native(at + 4, 0);
    }
    if runtime.native_word(at + 8) != 0 {
        runtime.write_slice_native(at + 4, runtime.native_word(at + 10));
    }
    runtime.write_slice_native(at + 8, 0);
}
fn choose_jump(k: u16, mut random: u16, runtime: &mut Runtime<'_>) {
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
    let phase = rom_word(runtime, 0xa29a0d, offset);
    runtime.edit_actor(key, Native, |actor| actor.variables[4] = phase);
}
fn prepare_velocity(runtime: &mut Runtime<'_>) {
    let (key, at) = selected(runtime);
    let mut offset = runtime.resolve_actor(key, Native).pre_instruction;
    let (mut speed, mut height) = (0u16, 0u16);
    loop {
        speed = speed.wrapping_add(rom_word(runtime, 0xa29a09, offset));
        height = height.wrapping_add(rom_word(runtime, 0xa2838f, 8 * (speed >> 8) + 1));
        offset = runtime.resolve_actor(key, Native).pre_instruction;
        let target = rom_word(runtime, 0xa29a07, offset).swap_bytes();
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
fn flight(runtime: &mut Runtime<'_>) {
    let (key, at) = selected(runtime);
    let mut speed = runtime.resolve_actor(key, Native).variables[1];
    if speed.wrapping_sub(0x4000) & 0x8000 == 0 {
        speed = 0x4000;
    }
    let mut offset = 8 * (speed >> 8);
    if runtime.native_word(at + 6) == 0 {
        offset = offset.wrapping_add(4);
    }
    let (live, _) = selected(runtime);
    let amount = runtime.native_speed32(usize::from(offset >> 2) * 4) as i32;
    if api::move_vertical(runtime, live, amount) == MovementResult::Blocked {
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
    flight_animation(runtime, falling);
    let acceleration = rom_word(runtime, 0xa29a0b, offset);
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
    let mut horizontal = rom_word(runtime, 0xa29a09, offset) >> 8;
    if runtime.native_word(at + 4) != 0 {
        horizontal = horizontal.wrapping_neg();
    }
    let (live, _) = selected(runtime);
    if api::move_horizontal(
        runtime,
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
enum Landing {
    Animate(u16),
    Restart,
    Repeat,
}
fn land(kind: Landing, runtime: &mut Runtime<'_>) {
    flight(runtime);
    let (key, at) = selected(runtime);
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
        runtime,
        match kind {
            Landing::Animate(instruction) => instruction,
            _ => 0x99d5,
        },
    );
}
fn fall(runtime: &mut Runtime<'_>) -> bool {
    let (key, _) = selected(runtime);
    let offset = runtime.resolve_actor(key, Native).pre_instruction;
    let high = rom_word(runtime, 0xa29a0b, offset) >> 8;
    let low = u16::from(rom_bytes::<1>(runtime, 0xa29a0b, usize::from(offset))[0]) << 8;
    let (live, _) = selected(runtime);
    api::move_vertical(
        runtime,
        live,
        ((u32::from(high) << 16) | u32::from(low)) as i32,
    ) == MovementResult::Blocked
}
