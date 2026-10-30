//! The Wrecked Ship ghost fades, tracks Samus, and reappears around her.
//! Tracking and bobbing words remain in authoritative extended actor RAM.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::{word_arrays as A, words as W};
use sm_game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use sm_game::enemy_projectiles::host::Effects;

const OFFSETS: [u16; 18] = [
    0xffc0, 0xffc0, 0, 0xffc0, 0x40, 0, 0xffc0, 0, 0, 0, 0x40, 0, 0xffc0, 0x40, 0, 0x40, 0x40, 0x40,
];
const FLICKER: [u16; 17] = [1, 8, 1, 8, 1, 7, 1, 7, 2, 6, 2, 6, 3, 5, 3, 5, 0xffff];
const PALETTE: [u16; 16] = [
    0x3800, 0x57ff, 0x42f7, 0x929, 0xa5, 0x4f5a, 0x36b5, 0x2610, 0x1dce, 0x1df, 0x1f, 0x18, 0xa,
    0x6b9, 0xea, 0x45,
];
fn selected(runtime: &Runtime<'_>) -> Key {
    Key::from_legacy(runtime.word(W::cur_enemy_index))
}
fn extra(key: Key) -> usize {
    0x7800 + usize::from(key.byte_index())
}
fn palette_start(runtime: &Runtime<'_>, key: Key) -> u16 {
    runtime
        .resolve_actor(key, Native)
        .palette
        .swap_bytes()
        .wrapping_mul(16)
        .wrapping_add(256)
}
fn set_target(runtime: &mut Runtime<'_>, key: Key, colors: impl IntoIterator<Item = u16>) {
    let mut at = palette_start(runtime, key);
    for color in colors {
        runtime.write_slice_le(A::target_palettes + usize::from(at), color);
        at = at.wrapping_add(2);
    }
}
pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.properties |= 0x2500;
        actor.instruction_delay = 1;
        actor.timer = 0;
        actor.instruction = 0x9a8c;
        actor.variables[0] = 0x9d13;
        actor.variables[1] = 280;
    });
    // Initialization and disappearance intentionally include the seventeenth color.
    set_target(&mut runtime, key, [0; 17]);
}
fn flicker(runtime: &mut Runtime<'_>, key: Key) {
    runtime.edit_actor(key, Native, |actor| {
        let mut visible = actor.variables[1] == 0;
        if !visible {
            actor.variables[1] = actor.variables[1].wrapping_sub(1);
            if actor.variables[1] != 0 {
                return;
            }
            let phase = actor.variables[2];
            let delay = FLICKER[usize::from(phase >> 1)];
            if delay & 0x8000 != 0 {
                actor.variables[1] = 0;
                actor.variables[2] = 0;
                return;
            }
            actor.variables[1] = delay;
            actor.variables[2] = phase.wrapping_add(2);
            visible = phase & 2 == 0;
        }
        if visible {
            actor.properties &= !0x100;
        }
    });
}
fn fade(runtime: &mut Runtime<'_>) -> u16 {
    if runtime.word(W::door_transition_flag_enemies) != 0 {
        return 0;
    }
    let mut at = palette_start(runtime, selected(runtime));
    let mut changed = 0u16;
    for _ in 0..16 {
        let source = A::palette_buffer + usize::from(at);
        let target = A::target_palettes + usize::from(at);
        if runtime.word(target) != runtime.word(source) {
            for shift in [0, 5, 10] {
                let mask = 31 << shift;
                let goal = runtime.word(target) & mask;
                let value = runtime.word(source) & mask;
                if value != goal {
                    let value = if (value as i16).wrapping_sub(goal as i16) >= 0 {
                        value.wrapping_sub(1 << shift)
                    } else {
                        value.wrapping_add(1 << shift)
                    };
                    runtime.write_slice_le(source, value | (runtime.word(source) & !mask));
                    changed = changed.wrapping_add(1);
                }
            }
        }
        at = at.wrapping_add(2);
    }
    changed
}
fn brighten(runtime: &mut Runtime<'_>, key: Key) {
    flicker(runtime, key);
    let key = selected(runtime);
    let mut at = palette_start(runtime, key);
    let mut complete = true;
    for _ in 0..16 {
        let source = A::palette_buffer + usize::from(at);
        if runtime.word(source) & 31 != 31 {
            runtime.write_slice_le(source, runtime.word(source).wrapping_add(1057));
            complete = false;
        }
        at = at.wrapping_add(2);
    }
    if complete {
        runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0x9bad);
        set_target(runtime, key, PALETTE);
    }
}
fn appear(runtime: &mut Runtime<'_>) {
    let changed = fade(runtime);
    let key = selected(runtime);
    flicker(runtime, key);
    let key = selected(runtime);
    if runtime.resolve_actor(key, Native).variables[1] | changed != 0 {
        return;
    }
    let at = extra(key);
    let y = runtime.edit_actor(key, Native, |actor| {
        actor.properties &= 0xfaff;
        actor.variables[0] = 0x9c8a;
        actor.y.whole
    });
    runtime.write_slice_native(at, y);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[1] = 1;
        actor.variables[2] = 2;
    });
    runtime.write_slice_native(at + 2, 0);
    runtime.write_slice_native(at + 4, 1);
    runtime.edit_actor(key, Native, |actor| actor.variables[1] = 120);
    runtime.write_slice_native(at + 10, 4);
    let x = runtime.word(W::samus_x_pos);
    for offset in [12, 14, 16] {
        runtime.write_slice_native(at + offset, x);
    }
    runtime.write_slice_native(at + 18, 12);
    let y = runtime.native_word(0xafa + usize::from(runtime.word(W::cur_enemy_index)));
    for (offset, value) in [(20, y), (22, y), (24, y), (26, 64), (28, 16)] {
        runtime.write_slice_native(at + offset, value);
    }
}
fn bob(runtime: &mut Runtime<'_>, key: Key) {
    let at = extra(key);
    let speed =
        (u32::from(runtime.native_word(at + 4)) << 16) | u32::from(runtime.native_word(at + 2));
    runtime.edit_actor(key, Native, |actor| actor.y.advance(speed as i32));
    let above = (runtime.resolve_actor(key, Native).y.whole as i16)
        .wrapping_sub(runtime.native_word(at) as i16)
        < 0;
    let low = runtime.native_word(at + 2);
    let high = runtime.native_word(at + 4);
    let speed = ((u32::from(high) << 16) | u32::from(low)).wrapping_add(if above {
        0x1800
    } else {
        0xffffe800
    });
    runtime.write_slice_native(at + 2, speed as u16);
    runtime.write_slice_native(at + 4, (speed >> 16) as u16);
    let timer = runtime.edit_actor(key, Native, |actor| {
        actor.variables[1] = actor.variables[1].wrapping_sub(1);
        actor.variables[1]
    });
    if timer == 0 {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[0] = 0x9c69;
            actor.properties |= 0x400;
        });
        set_target(runtime, key, [0x7fff; 17]);
    }
}
fn ready_to_reposition(runtime: &mut Runtime<'_>, key: Key) -> bool {
    let at = extra(key);
    let outside = (runtime.word(W::samus_x_pos) as i16)
        .wrapping_sub(runtime.native_word(at + 14) as i16)
        < 0
        || (runtime.word(W::samus_x_pos) as i16).wrapping_sub(runtime.native_word(at + 16) as i16)
            >= 0
        || {
            let y = runtime.native_word(0xafa + usize::from(key.byte_index())) as i16;
            y.wrapping_sub(runtime.native_word(at + 22) as i16) < 0
                || y.wrapping_sub(runtime.native_word(at + 24) as i16) >= 0
        };
    if !outside {
        let timer = runtime.native_word(at + 26).wrapping_sub(1);
        runtime.write_slice_native(at + 26, timer);
        if timer == 0 {
            runtime.write_slice_native(at + 10, 4);
            runtime.write_slice_native(at + 18, 12);
        }
        return timer == 0;
    }
    runtime.write_slice_native(at + 26, 64);
    // Only inspect vertical movement if the horizontal direction still matches.
    for (position, previous, direction, step) in
        [(W::samus_x_pos, 12, 10, 4), (W::samus_y_pos, 20, 18, 12)]
    {
        let delta =
            (runtime.word(position) as i16).wrapping_sub(runtime.native_word(at + previous) as i16);
        let value = if delta < 0 {
            0
        } else if delta == 0 {
            step
        } else {
            2 * step
        };
        if value != runtime.native_word(at + direction) {
            runtime.write_slice_native(at + direction, value);
            runtime.write_slice_native(at + 28, 16);
            return false;
        }
    }
    let timer = runtime.native_word(at + 28).wrapping_sub(1);
    runtime.write_slice_native(at + 28, timer);
    timer == 0
}
fn track(runtime: &mut Runtime<'_>, key: Key) {
    let at = extra(key);
    if ready_to_reposition(runtime, key) {
        runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0x9b42);
        runtime.write_slice_native(at + 26, 64);
        runtime.write_slice_native(at + 28, 16);
        let index = usize::from(
            runtime
                .native_word(at + 18)
                .wrapping_add(runtime.native_word(at + 10))
                >> 1,
        );
        let x =
            OFFSETS[index].wrapping_add(runtime.native_word(0xaf6 + usize::from(key.byte_index())));
        runtime.edit_actor(key, Native, |actor| actor.x.whole = x);
        let y = OFFSETS[index + 1]
            .wrapping_add(runtime.native_word(0xafa + usize::from(key.byte_index())));
        runtime.edit_actor(key, Native, |actor| actor.y.whole = y);
    } else {
        for (position, offset) in [(W::samus_x_pos, 12), (W::samus_y_pos, 20)] {
            let value = runtime.word(position);
            runtime.write_slice_native(at + offset, value);
            runtime.write_slice_native(at + offset + 2, value.wrapping_sub(1));
            runtime.write_slice_native(at + offset + 4, runtime.word(position).wrapping_add(1));
        }
    }
}
pub(crate) fn advance(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    let state = 0xa80000 | u32::from(runtime.resolve_actor(key, Native).variables[0]);
    match state {
        0xa89b42 => brighten(&mut runtime, key),
        0xa89bad => appear(&mut runtime),
        0xa89c69 => {
            if fade(&mut runtime) == 0 {
                let key = selected(&runtime);
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[0] = 0x9d13;
                    actor.properties |= 0x100;
                    actor.variables[1] = 120;
                });
            }
        }
        0xa89c8a => bob(&mut runtime, key),
        0xa89d13 => {
            runtime.edit_actor(key, Native, |actor| {
                if actor.variables[1] != 0 {
                    actor.variables[1] = actor.variables[1].wrapping_sub(1);
                }
                if actor.variables[1] == 0 {
                    actor.variables[1] = 1;
                    actor.variables[2] = 2;
                    actor.variables[0] = 0x9d36;
                }
            });
        }
        0xa89d36 => track(&mut runtime, key),
        _ => runtime.unreachable(),
    }
}
