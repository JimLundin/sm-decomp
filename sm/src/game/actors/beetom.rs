//! Beetom walking, jumps and player attachment share the actor movement owner.
//! Extended animation and jump words remain native, RAM-authoritative storage.
#![forbid(unsafe_code)]
use super::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use super::{api, combat};
use crate::game::enemy_projectiles::host::Effects;
use crate::game::hopping_blobs::initial_hop_phase;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::words;

const ANIMATION: usize = 0;
const SMALL_JUMP: usize = 4;
const LARGE_JUMP: usize = 6;
const CHASE_JUMP: usize = 8;
const FALLING: usize = 10;
const CONTACT_X: usize = 12;
const CONTACT_Y: usize = 14;
const ATTACHED: usize = 16;
const DIRECTION: usize = 18;

fn selected(runtime: &Runtime<'_>) -> (Key, usize) {
    let index = runtime.word(words::cur_enemy_index);
    (Key::from_legacy(index), 0x7800 + usize::from(index))
}
fn set_state(runtime: &mut Runtime<'_>, key: Key, state: u16) {
    runtime.edit_actor(key, Native, |actor| actor.variables[2] = state);
}
fn animate(runtime: &mut Runtime<'_>) {
    let (key, extra) = selected(runtime);
    let instruction = runtime.native_word(extra + ANIMATION);
    runtime.edit_actor(key, Native, |actor| actor.restart_animation(instruction));
}
fn reverse(runtime: &mut Runtime<'_>, key: Key, extra: usize) {
    runtime.write_slice_native(
        extra + DIRECTION,
        runtime.native_word(extra + DIRECTION) ^ 1,
    );
    set_state(runtime, key, 0xb97a);
}
fn horizontal(runtime: &mut Runtime<'_>, key: Key, amount: i32) -> bool {
    api::move_horizontal(runtime, key, amount, api::SlopePolicy::Ignore)
        == api::MovementResult::Blocked
}

pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (key, extra) = selected(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[3] = 0;
        actor.variables[2] = 0;
    });
    runtime.write_slice_native(extra + FALLING, 0);
    runtime.write_slice_native(extra + ATTACHED, 0);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[0] = 0;
        actor.variables[4] = 64;
    });
    let keys = runtime.word(words::joypad1_lastkeys);
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = keys);
    runtime.write_slice_le(words::random_number, 23);
    for (offset, height, step) in [
        (SMALL_JUMP, 12288, 4),
        (LARGE_JUMP, 0x4000, 5),
        (CHASE_JUMP, 12288, 3),
    ] {
        let phase = initial_hop_phase(height, step, runtime.native_image());
        runtime.write_slice_native(extra + offset, phase);
    }
    runtime.write_slice_native(extra + ANIMATION, 0xb6f2);
    let current = selected(&runtime).0;
    if api::player_delta(&runtime, current, api::Axis::X) & 0x8000 != 0 {
        runtime.write_slice_native(extra + ANIMATION, 0xb696);
    }
    animate(&mut runtime);
    set_state(&mut runtime, key, 0xb814);
}

fn begin_hop(runtime: &mut Runtime<'_>, high: bool, left: bool) {
    let (key, extra) = selected(runtime);
    let phase = runtime.native_word(extra + if high { LARGE_JUMP } else { SMALL_JUMP });
    runtime.edit_actor(key, Native, |actor| actor.variables[1] = phase);
    set_state(
        runtime,
        key,
        match (high, left) {
            (false, true) => 0xba84,
            (false, false) => 0xbab7,
            (true, true) => 0xbb55,
            (true, false) => 0xbb88,
        },
    );
    runtime.write_slice_native(extra + FALLING, 0);
    runtime.write_slice_native(extra + ANIMATION, if left { 0xb6ac } else { 0xb708 });
    animate(runtime);
}
fn walk(runtime: &mut Runtime<'_>, left: bool) {
    let (key, _) = selected(runtime);
    let timer = runtime.edit_actor(key, Native, |actor| {
        actor.variables[3] = actor.variables[3].wrapping_sub(1);
        actor.variables[3]
    });
    if timer & 0x8000 != 0 {
        runtime.edit_actor(key, Native, |actor| actor.variables[3] = 64);
        set_state(runtime, key, 0xb814);
        return;
    }
    let offset = if left { 0xfff8 } else { 8 };
    runtime.edit_actor(key, Native, |actor| {
        actor.x.whole = actor.x.whole.wrapping_add(offset)
    });
    let current = selected(runtime).0;
    if api::move_vertical(runtime, current, 1 << 16) == api::MovementResult::Blocked {
        runtime.edit_actor(key, Native, |actor| {
            actor.x.whole = actor.x.whole.wrapping_sub(offset)
        });
        let current = selected(runtime).0;
        if horizontal(runtime, current, if left { -16384 } else { 16384 }) {
            set_state(runtime, key, if left { 0xb873 } else { 0xb85f });
        }
    } else {
        set_state(runtime, key, if left { 0xb873 } else { 0xb85f });
        runtime.edit_actor(key, Native, |actor| {
            actor.y.whole = actor.y.whole.wrapping_sub(1);
            actor.x.whole = actor.x.whole.wrapping_sub(offset);
        });
    }
}
fn step_jump(runtime: &mut Runtime<'_>, key: Key, falling: bool, step: u16, captured: bool) {
    let phase = runtime.resolve_actor(key, Native).variables[1];
    let amount =
        runtime.native_speed32(8 * usize::from(phase) + if falling { 0 } else { 4 }) as i32;
    if api::move_vertical(runtime, key, amount) == api::MovementResult::Blocked {
        set_state(runtime, key, if falling { 0xb814 } else { 0xb97a });
        return;
    }
    let next = runtime.edit_actor(key, Native, |actor| {
        actor.variables[1] = if falling {
            actor.variables[1].wrapping_add(step)
        } else {
            actor.variables[1].wrapping_sub(step)
        };
        actor.variables[1]
    });
    // Retain the original captured-value and post-store-read variants.
    let tested = if captured {
        next
    } else {
        runtime.resolve_actor(key, Native).variables[1]
    };
    if falling {
        if tested.wrapping_sub(64) & 0x8000 == 0 {
            runtime.edit_actor(key, Native, |actor| actor.variables[1] = 64);
        }
    } else if tested & 0x8000 != 0 {
        runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0);
        runtime.write_slice_native(0x7800 + usize::from(key.byte_index()) + FALLING, 1);
    }
}
fn hop(runtime: &mut Runtime<'_>, high: bool, chase: bool, left: bool) {
    let (key, extra) = selected(runtime);
    let falling = runtime.native_word(extra + FALLING) != 0;
    let current = selected(runtime).0;
    let step = if chase && falling {
        3
    } else if high || chase {
        5
    } else {
        4
    };
    step_jump(runtime, current, falling, step, chase);
    // Horizontal movement rereads the selector; collision updates retain the caller's actor.
    let current = selected(runtime).0;
    let speed = if chase { 196608 } else { 16384 };
    if horizontal(runtime, current, if left { -speed } else { speed }) {
        reverse(runtime, key, extra);
    }
}
fn cling(runtime: &mut Runtime<'_>, left: bool) {
    let (key, extra) = selected(runtime);
    if runtime.resolve_actor(key, Native).variables[4] != 0 {
        let x = runtime.word(words::samus_x_pos);
        runtime.edit_actor(key, Native, |actor| actor.x.whole = x);
        let y = runtime.word(words::samus_y_pos).wrapping_sub(4);
        runtime.edit_actor(key, Native, |actor| actor.y.whole = y);
        let current = selected(runtime).0;
        if runtime.word(words::joypad1_lastkeys)
            != runtime.resolve_actor(current, Native).pre_instruction
        {
            let keys = runtime.word(words::joypad1_lastkeys);
            runtime.edit_actor(current, Native, |actor| {
                actor.pre_instruction = keys;
                actor.variables[4] = actor.variables[4].wrapping_sub(1);
            });
        }
    } else {
        let current = selected(runtime).0;
        if horizontal(runtime, current, if left { -16 << 16 } else { 16 << 16 }) {
            runtime.write_slice_native(extra + DIRECTION, u16::from(!left));
            let current = selected(runtime).0;
            horizontal(runtime, current, if left { 32 << 16 } else { -32 << 16 });
        }
        runtime.write_slice_native(extra + ATTACHED, 0);
        set_state(runtime, key, 0xb9a2);
    }
}

pub(crate) fn advance(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (key, extra) = selected(&runtime);
    let state = runtime.resolve_actor(key, Native).variables[2];
    match state {
        0xb814 => {
            let near = api::player_within(&mut runtime, key, api::Axis::X, 0x60);
            let current = selected(&runtime).0;
            set_state(&mut runtime, current, if near { 0xb90f } else { 0xb82f });
        }
        0xb82f => {
            runtime.next_random();
            let (key, extra) = selected(&runtime);
            let image = runtime.loaded_image().expect("ROM must be loaded");
            let state = image.table_word(
                crate::rom::lorom_offset(0xa8b74e) as usize,
                (runtime.word(words::random_number) & 7) as isize,
            );
            set_state(&mut runtime, key, state);
            let direction = runtime.word(words::random_number) & 1;
            runtime.write_slice_native(extra + DIRECTION, direction);
        }
        0xb84f => {
            runtime.edit_actor(key, Native, |actor| actor.variables[3] = 32);
            set_state(&mut runtime, key, 0xb9b2);
        }
        0xb85f | 0xb873 => {
            let left = state == 0xb85f;
            set_state(&mut runtime, key, if left { 0xb9c1 } else { 0xba24 });
            runtime.write_slice_native(extra + ANIMATION, if left { 0xb696 } else { 0xb6f2 });
            animate(&mut runtime);
        }
        0xb887 | 0xb8a9 | 0xb8cb | 0xb8ed => begin_hop(
            &mut runtime,
            matches!(state, 0xb8cb | 0xb8ed),
            matches!(state, 0xb887 | 0xb8cb),
        ),
        0xb90f => {
            let phase = runtime.native_word(extra + CHASE_JUMP);
            runtime.edit_actor(key, Native, |actor| actor.variables[1] = phase);
            runtime.write_slice_native(extra + ANIMATION, 0xb708);
            set_state(&mut runtime, key, 0xbc5a);
            runtime.write_slice_native(extra + DIRECTION, 1);
            let current = selected(&runtime).0;
            if api::player_delta(&runtime, current, api::Axis::X) & 0x8000 != 0 {
                runtime.write_slice_native(extra + ANIMATION, 0xb6ac);
                set_state(&mut runtime, key, 0xbc26);
                runtime.write_slice_native(extra + DIRECTION, 0);
            }
            animate(&mut runtime);
            runtime.write_slice_native(extra + FALLING, 0);
        }
        0xb952 | 0xb966 => {
            let left = state == 0xb952;
            runtime.write_slice_native(extra + ANIMATION, if left { 0xb6cc } else { 0xb728 });
            animate(&mut runtime);
            set_state(&mut runtime, key, if left { 0xbcf8 } else { 0xbd42 });
        }
        0xb97a => {
            runtime.write_slice_native(extra + ANIMATION, 0xb696);
            if runtime.native_word(extra + DIRECTION) != 0 {
                runtime.write_slice_native(extra + ANIMATION, 0xb6f2);
            }
            animate(&mut runtime);
            runtime.write_slice_native(extra + FALLING, 0);
            set_state(&mut runtime, key, 0xbd9d);
        }
        0xb9a2 => {
            runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0);
            set_state(&mut runtime, key, 0xbdc5);
        }
        0xb9b2 => {
            let timer = runtime.edit_actor(key, Native, |actor| {
                actor.variables[3] = actor.variables[3].wrapping_sub(1);
                actor.variables[3]
            });
            if timer & 0x8000 != 0 {
                set_state(&mut runtime, key, 0xb814);
            }
        }
        0xb9c1 | 0xba24 => walk(&mut runtime, state == 0xb9c1),
        0xba84 | 0xbab7 => hop(&mut runtime, false, false, state == 0xba84),
        0xbb55 | 0xbb88 => hop(&mut runtime, true, false, state == 0xbb55),
        0xbc26 | 0xbc5a => hop(&mut runtime, false, true, state == 0xbc26),
        0xbcf8 | 0xbd42 => cling(&mut runtime, state == 0xbd42),
        0xbd9d => {
            if api::move_vertical(&mut runtime, key, 3 << 16) == api::MovementResult::Blocked {
                let (key, extra) = selected(&runtime);
                let state = if runtime.native_word(extra + DIRECTION) != 0 {
                    0xb873
                } else {
                    0xb85f
                };
                set_state(&mut runtime, key, state);
            }
        }
        0xbdc5 => {
            step_jump(&mut runtime, key, true, 1, true);
            let speed = if runtime.native_word(extra + DIRECTION) != 0 {
                -2 << 16
            } else {
                2 << 16
            };
            if horizontal(&mut runtime, key, speed) {
                reverse(&mut runtime, key, extra);
            }
        }
        _ => runtime.unreachable(),
    }
}

pub(crate) fn touch(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (key, extra) = selected(&runtime);
    if runtime.native_word(extra + ATTACHED) == 0 {
        let state = if runtime.native_word(extra + DIRECTION) != 0 {
            0xb966
        } else {
            0xb952
        };
        set_state(&mut runtime, key, state);
        runtime.write_slice_native(extra + ATTACHED, 1);
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[4] = 64;
            actor.layer = 2;
        });
        let x = runtime
            .word(words::samus_x_pos)
            .wrapping_sub(runtime.resolve_actor(key, Native).x.whole);
        runtime.write_slice_native(extra + CONTACT_X, x);
        let y = runtime
            .word(words::samus_y_pos)
            .wrapping_sub(runtime.resolve_actor(key, Native).y.whole);
        runtime.write_slice_native(extra + CONTACT_Y, y);
    }
    let hurt = runtime.word(words::samus_contact_damage_index) != 0 || {
        if runtime.word(words::random_enemy_counter) & 7 == 7
            && runtime.word(words::samus_health).wrapping_sub(30) & 0x8000 == 0
        {
            runtime.queue_sound(3, 6, 0x2d);
        }
        runtime.resolve_actor(key, Native).frame & 0x3f == 63
    };
    if hurt {
        combat::contact_reaction(&mut runtime, api::DeathHandling::Finalize);
        runtime.write_slice_le(words::samus_invincibility_timer, 0);
        runtime.write_slice_le(words::samus_knockback_timer, 0);
    }
}
pub(crate) fn shot(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    combat::shot_reaction(&mut runtime, api::ShotHandling::Finalize);
    let (key, extra) = selected(&runtime);
    if runtime.resolve_actor(key, Native).frozen != 0
        && matches!(
            runtime.resolve_actor(key, Native).variables[2],
            0xbd42 | 0xbcf8
        )
    {
        set_state(&mut runtime, key, 0xb97a);
    }
    runtime.write_slice_native(extra + ATTACHED, 0);
}
