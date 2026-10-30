//! Turtle parent/children, riding interaction and their shared script state.
#![forbid(unsafe_code)]
use crate::game::actors::{api, combat};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::words as fields;
use crate::rtl::abs16;
use sm_game::actors::access::{
    AccessMode::{LittleEndian as Le, Native},
    ActorAccess, ActorKey as Key,
};
use sm_game::enemy_projectiles::host::Effects;
use sm_game::enemy_script::EnemyScript;

const SHELL_HEIGHT: [i16; 48] = [
    -16, -16, -16, -16, -15, -15, -15, -15, -15, -14, -13, -13, -12, -11, -10, -9, -8, -7, -6, -5,
    -4, -4, 0, 0, -16, -16, -16, -15, -15, -15, -14, -13, -12, -11, -10, -9, -8, -7, -6, -5, -4,
    -3, -3, -2, 0, 0, 0, 0,
];
fn selected(runtime: &Runtime<'_>) -> Key {
    Key::from_legacy(runtime.word(fields::cur_enemy_index))
}
fn animation(runtime: &mut Runtime<'_>, key: Key, instruction: u16) {
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = instruction;
        actor.instruction_delay = 1;
    });
}
fn walking_animation(runtime: &mut Runtime<'_>, key: Key) {
    let forward = runtime.resolve_actor(key, Native).variables[4] & 0x8000 == 0;
    animation(runtime, key, if forward { 0x8c72 } else { 0x8b80 });
    runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0x9142);
}
fn displacement(runtime: &mut Runtime<'_>, field: usize, delta: u16) {
    runtime.write_word_le(runtime.word(field).wrapping_add(delta), field);
}
fn horizontal(runtime: &mut Runtime<'_>, key: Key, speed: i32) -> bool {
    api::move_horizontal(runtime, key, speed, api::SlopePolicy::Ignore)
        == api::MovementResult::Blocked
}
fn vertical(runtime: &mut Runtime<'_>, key: Key, speed: i32) -> bool {
    api::move_vertical(runtime, key, speed) == api::MovementResult::Blocked
}
fn contact(runtime: &mut Runtime<'_>) {
    let (left, right, top, bottom) = {
        let actor = runtime.resolve_actor(Key::from_legacy(0), Native);
        (
            actor.x.whole.wrapping_sub(actor.radius.x).wrapping_sub(8),
            actor.x.whole.wrapping_add(actor.radius.x).wrapping_add(8),
            actor.y.whole.wrapping_sub(actor.radius.y).wrapping_add(4),
            actor.y.whole.wrapping_add(actor.radius.y).wrapping_sub(4),
        )
    };
    let x = i32::from(runtime.word(fields::samus_x_pos));
    let width = i32::from(runtime.word(fields::samus_x_radius));
    let y = i32::from(runtime.word(fields::samus_y_pos));
    let height = i32::from(runtime.word(fields::samus_y_radius));
    if (x - width - 1 - i32::from(right)) & 0x8000 != 0
        && (width + x - i32::from(left)) & 0x8000 == 0
        && (y - height + 1 - i32::from(bottom)) & 0x8000 != 0
        && (height + y - i32::from(top)) & 0x8000 == 0
        && runtime.word(fields::samus_invincibility_timer) == 0
    {
        combat::contact_reaction(runtime, api::DeathHandling::Finalize);
    }
}
fn detach_child(runtime: &mut Runtime<'_>) {
    let key = selected(runtime);
    let parent = Key::from_legacy(runtime.resolve_actor(key, Native).variables[1]);
    runtime.edit_actor(parent, Native, |actor| {
        if actor.pre_instruction != 0 {
            actor.pre_instruction = actor.pre_instruction.wrapping_sub(1);
        }
    });
}
pub(crate) fn initialize(child: bool, game: &mut Game) {
    let key = Key::from_legacy(game.cur_enemy_index());
    Runtime::new(game).edit_actor(key, Le, |actor| {
        if child {
            actor.variables[2] = actor.x.whole;
            actor.variables[3] = actor.y.whole.wrapping_sub(actor.radius.y);
            actor.variables[0] = 0x9142;
            actor.instruction_delay = 1;
            actor.timer = 0;
            let parameter = actor.parameters[0];
            actor.variables[4] = parameter;
            actor.instruction = if parameter as i16 >= 0 {
                0x8c72
            } else {
                0x8b80
            };
        } else {
            actor.properties |= 0x2000;
            actor.spritemap = 0x804d;
            actor.instruction_delay = 1;
            actor.timer = 0;
            actor.radius.y = 0;
            actor.instruction = 0x8c44;
            actor.variables[0] = 0x8dd8;
            actor.pre_instruction = 1;
        }
    });
}
pub(crate) fn advance(child: bool, game: &mut Game) -> bool {
    let mut runtime = Runtime::new(game);
    let key = selected(&runtime);
    if child {
        let parent = runtime.resolve_actor(key, Le).variables[1];
        runtime.write_word_le(0, 0x780c + usize::from(parent));
    }
    let phase = runtime.resolve_actor(key, Le).variables[0];
    let raw = runtime.word(fields::cur_enemy_index);
    run_phase(&mut runtime, raw, phase)
}
fn run_phase(runtime: &mut Runtime<'_>, raw: u16, phase: u16) -> bool {
    let key = Key::from_legacy(raw);
    let at = 0x7800 + usize::from(raw);
    match phase {
        0x8dd8 => {
            let base = 0xf78 + usize::from(raw);
            // These child offsets are wide, even when the parent identity is near u16::MAX.
            for field in [
                core::mem::offset_of!(crate::types::EnemyBase, palette_index),
                core::mem::offset_of!(crate::types::EnemyBase, vram_tiles_index),
            ] {
                let value = runtime.word(base + field);
                for child in [64, 128, 192, 256] {
                    runtime.write_word_le(value, base + child + field);
                }
            }
            for child in [64, 128, 192, 256] {
                runtime.write_word_le(raw, base + child + 0x32);
            }
            runtime.write_word_le(0x8e0a, base + 0x30);
        }
        0x8e0a => {
            if runtime.resolve_actor(key, Le).pre_instruction != 0 {
                runtime.edit_actor(key, Le, |actor| actor.radius.y = 0);
                let delta = runtime
                    .resolve_actor(key, Le)
                    .x
                    .whole
                    .wrapping_sub(runtime.word(fields::samus_x_pos));
                let magnitude = abs16(delta);
                if magnitude < 24 {
                    let index = magnitude + if delta & 0x8000 != 0 { 24 } else { 0 };
                    let live = selected(runtime);
                    let radius = SHELL_HEIGHT[usize::from(index)].wrapping_neg() as u16;
                    runtime.edit_actor(key, Le, |actor| {
                        actor.radius.y = radius;
                        actor.properties |= 0x8000;
                    });
                    if api::touches_player_from_above(runtime, live) {
                        let actor = runtime.resolve_actor(key, Le);
                        let top = actor.y.whole.wrapping_sub(actor.radius.y);
                        let feet = runtime
                            .word(fields::samus_y_radius)
                            .wrapping_add(runtime.word(fields::samus_y_pos));
                        if feet.wrapping_sub(top) as i16 >= 0 {
                            displacement(
                                runtime,
                                fields::extra_samus_y_displacement,
                                top.wrapping_sub(feet),
                            );
                        }
                    }
                }
            } else {
                runtime.edit_actor(key, Le, |actor| {
                    actor.variables[0] = 0x8ee0;
                    actor.properties &= !0x400;
                });
            }
        }
        0x8ee0 => {
            contact(runtime);
            if runtime.byte(0x5b5) & 1 == 0 {
                if api::touches_player_from_above(runtime, key) {
                    displacement(runtime, fields::extra_samus_x_displacement, u16::MAX);
                }
                runtime.edit_actor(key, Native, |actor| {
                    actor.y.whole = actor.y.whole.wrapping_sub(1);
                    actor.radius.y = 16;
                    actor.x.whole = actor.x.whole.wrapping_add(if actor.y.whole & 1 != 0 {
                        u16::MAX
                    } else {
                        1
                    });
                });
                if !horizontal(runtime, key, 1 << 16) {
                    animation(runtime, key, 0x8c4a);
                    runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0x8e09);
                }
            }
        }
        0x8f3f => {
            let left = (runtime
                .resolve_actor(key, Native)
                .x
                .whole
                .wrapping_sub(runtime.word(fields::samus_x_pos)) as i16)
                < 0;
            animation(runtime, key, if left { 0x8d00 } else { 0x8c1c });
            runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0x8e09);
        }
        0x8f8d => {
            contact(runtime);
            if !vertical(runtime, key, -1 << 16) {
                if api::touches_player_from_above(runtime, key) {
                    displacement(runtime, fields::extra_samus_y_displacement, u16::MAX);
                }
                let delay = runtime.native_word(at).wrapping_sub(1);
                runtime.write_slice_native(at, delay);
                if delay == 0 {
                    let left = (runtime
                        .resolve_actor(key, Native)
                        .x
                        .whole
                        .wrapping_sub(runtime.word(fields::samus_x_pos))
                        as i16)
                        < 0;
                    let index = if left { 2 } else { 0 };
                    let value = runtime
                        .native_image()
                        .table_word(crate::rom::lorom_offset(0xa28d56) as usize, index);
                    runtime.write_slice_native(at + 2, value);
                    let value = runtime
                        .native_image()
                        .table_word(crate::rom::lorom_offset(0xa28d58) as usize, index);
                    runtime.write_slice_native(at + 4, value);
                    runtime.edit_actor(key, Native, |actor| actor.variables[4] = 0);
                    runtime.write_slice_native(at + 6, 0);
                    runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0x8feb);
                }
            }
        }
        0x8feb => {
            contact(runtime);
            let speed = (u32::from(runtime.resolve_actor(key, Native).variables[4]) << 16)
                | u32::from(runtime.native_word(at + 6));
            if horizontal(runtime, key, speed as i32) {
                let speed = (u32::from(runtime.resolve_actor(key, Native).variables[4]) << 16)
                    | u32::from(runtime.native_word(at + 6));
                let speed = (-(speed as i32)) as u32;
                runtime.write_slice_native(at + 6, speed as u16);
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[4] = (speed >> 16) as u16
                });
                let acceleration = (u32::from(runtime.native_word(at + 4)) << 16)
                    | u32::from(runtime.native_word(at + 2));
                let acceleration = (-(acceleration as i32)) as u32;
                runtime.write_slice_native(at + 2, acceleration as u16);
                runtime.write_slice_native(at + 4, (acceleration >> 16) as u16);
                runtime.write_word_le(0, fields::earthquake_type);
                runtime.write_word_le(16, fields::earthquake_timer);
                runtime.queue_sound(2, 6, 0x1b);
            } else {
                carry_player(runtime, key, at);
                let speed = (u32::from(runtime.resolve_actor(key, Native).variables[4]) << 16)
                    | u32::from(runtime.native_word(at + 6));
                let acceleration = (u32::from(runtime.native_word(at + 4)) << 16)
                    | u32::from(runtime.native_word(at + 2));
                let speed = speed.wrapping_add(acceleration);
                runtime.write_slice_native(at + 6, speed as u16);
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[4] = (speed >> 16) as u16
                });
                runtime.edit_actor(key, Native, |actor| {
                    let speed = actor.variables[4];
                    if (abs16(speed) as i16).wrapping_sub(3) >= 0 {
                        actor.variables[4] = if speed & 0x8000 != 0 {
                            (-3i16) as u16
                        } else {
                            3
                        };
                    }
                });
            }
        }
        0x9083 => {
            contact(runtime);
            let phase =
                if (runtime.resolve_actor(key, Native).y.whole as i16).wrapping_sub(0x1e8) < 0 {
                    runtime.write_slice_native(at, 30);
                    0x90cc
                } else {
                    if api::touches_player_from_above(runtime, key) {
                        runtime.edit_actor(key, Native, |actor| {
                            actor.y.whole = actor.y.whole.wrapping_sub(7)
                        });
                        displacement(runtime, fields::extra_samus_y_displacement, (-7i16) as u16);
                        return true;
                    }
                    0x90e1
                };
            runtime.edit_actor(key, Native, |actor| actor.variables[0] = phase);
            runtime.write_slice_native(at + 14, 0);
            runtime.write_slice_native(at + 8, 0);
        }
        0x90cc => {
            contact(runtime);
            let delay = runtime.native_word(at).wrapping_sub(1);
            runtime.write_slice_native(at, delay);
            if delay == 0 {
                runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0x90e1);
            }
        }
        0x90e1 => {
            contact(runtime);
            if (runtime.native_word(0x7808) as i16).wrapping_sub(4) < 0 {
                let speed = (u32::from(runtime.native_word(at + 8)) << 16)
                    | u32::from(runtime.native_word(at + 14));
                let speed = speed.wrapping_add(0x2000);
                runtime.write_slice_native(at + 14, speed as u16);
                runtime.write_slice_native(at + 8, (speed >> 16) as u16);
            }
            let speed = i32::from(runtime.native_word(at + 8) as i16) << 16;
            if vertical(runtime, key, speed) {
                let forward = runtime.resolve_actor(key, Native).variables[4] & 0x8000 == 0;
                animation(runtime, key, if forward { 0x8d28 } else { 0x8c4a });
                runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0x8e09);
            }
        }
        0x9142 => {
            if api::touches_player_from_above(runtime, key) {
                runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0x916e);
                runtime.write_slice_native(at + 10, 4);
                let forward = runtime.resolve_actor(key, Native).variables[4] & 0x8000 == 0;
                animation(runtime, key, if forward { 0x8d14 } else { 0x8c30 });
            }
        }
        0x916e => {
            if api::touches_player_from_above(runtime, key) {
                runtime.write_slice_native(at + 10, 4);
            } else {
                let delay = runtime.native_word(at + 10).wrapping_sub(1);
                runtime.write_slice_native(at + 10, delay);
                if delay == 0 {
                    runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0x9198);
                    runtime.write_slice_native(at, 60);
                }
            }
        }
        0x9198 => {
            if api::touches_player_from_above(runtime, key) {
                runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0x91f8);
                animation(runtime, key, 0x8bd2);
                runtime.write_slice_native(at + 8, 1);
                let speed = if runtime.byte(0xa1e) & 15 == 8 {
                    3
                } else {
                    (-3i16) as u16
                };
                runtime.edit_actor(key, Native, |actor| actor.variables[4] = speed);
            } else {
                let delay = runtime.native_word(at).wrapping_sub(1);
                runtime.write_slice_native(at, delay);
                if delay == 0 {
                    walking_animation(runtime, key);
                }
            }
        }
        0x91f8 => move_child(runtime, key, at),
        0x9239 => {
            if api::touches_player_from_above(runtime, key) {
                walking_animation(runtime, key);
            } else {
                move_child(runtime, key, at);
            }
        }
        0x925e => {
            let key = selected(runtime);
            let parent = runtime.resolve_actor(key, Native).variables[1];
            let height = runtime.resolve_actor(key, Native).radius.y;
            runtime.write_slice_native(0x780c + usize::from(parent), height);
            let current = selected(runtime);
            if !api::touches_player_from_above(runtime, current) {
                let current = selected(runtime);
                runtime.edit_actor(current, Native, |actor| actor.variables[0] = 0x9142);
            }
        }
        0x8e09 => {}
        _ => return false,
    }
    true
}
fn move_child(runtime: &mut Runtime<'_>, key: Key, at: usize) {
    let speed = i32::from(runtime.resolve_actor(key, Native).variables[4] as i16) << 16;
    if horizontal(runtime, key, speed) {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[4] = actor.variables[4].wrapping_neg()
        });
    } else {
        let speed = i32::from(runtime.native_word(at + 8) as i16) << 16;
        vertical(runtime, key, speed);
    }
}
fn carry_player(runtime: &mut Runtime<'_>, key: Key, at: usize) {
    if api::touches_player_from_above(runtime, key) {
        runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0x9083);
        let speed = (u32::from(runtime.resolve_actor(key, Native).variables[4]) << 16)
            | u32::from(runtime.native_word(at + 6));
        let delta = (-(speed as i32)) as u32;
        let low = u32::from(runtime.native_word(fields::extra_samus_x_subdisplacement));
        let high = u32::from(runtime.native_word(fields::extra_samus_x_displacement));
        let moved = (low | (high << 16)).wrapping_add(delta);
        runtime.write_slice_native(fields::extra_samus_x_subdisplacement, moved as u16);
        runtime.write_slice_native(fields::extra_samus_x_displacement, (moved >> 16) as u16);
        let whole = i32::from(runtime.word(fields::extra_samus_x_displacement));
        if (whole + 16) & 0x8000 != 0 {
            runtime.write_word_le((-16i16) as u16, fields::extra_samus_x_displacement);
        }
    }
}
pub(crate) fn touch(child: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let raw = runtime.word(fields::cur_enemy_index);
    let key = Key::from_legacy(raw);
    if !child {
        if runtime.resolve_actor(key, Native).properties & 0x8000 == 0 {
            combat::contact_reaction(&mut runtime, api::DeathHandling::Finalize);
            runtime.edit_actor(key, Native, |actor| actor.variables[0] = 0x90e1);
            runtime.write_slice_native(0x7808 + usize::from(raw), 2);
        }
    } else if runtime.resolve_actor(key, Native).variables[0] != 0x925e {
        runtime.edit_actor(key, Native, |actor| {
            let right = actor.variables[4] & 0x8000 != 0;
            actor.instruction = if right { 0x8c72 } else { 0x8b80 };
            actor.variables[4] = if right { 1 } else { u16::MAX };
            actor.instruction_delay = 1;
        });
        let x = if runtime
            .resolve_actor(key, Native)
            .x
            .whole
            .wrapping_sub(runtime.word(fields::samus_x_pos)) as i16
            >= 0
        {
            runtime
                .resolve_actor(key, Native)
                .radius
                .x
                .wrapping_add(runtime.word(fields::samus_x_radius))
                .wrapping_add(runtime.word(fields::samus_x_pos))
        } else {
            // Preserve the original packed subtraction, including its low-word borrow.
            let player = u32::from(runtime.word(fields::samus_x_pos));
            let radius = u32::from(runtime.word(fields::samus_x_radius));
            let packed = (player.wrapping_sub(radius) << 16) | player;
            let width = u32::from(runtime.resolve_actor(key, Native).radius.x);
            (packed.wrapping_sub((width << 16) | u32::from(runtime.word(fields::samus_x_radius)))
                >> 16) as u16
        };
        runtime.edit_actor(key, Native, |actor| {
            actor.x.whole = x;
            actor.variables[0] = 0x9142;
        });
        let current = selected(&runtime);
        let speed = i32::from(runtime.resolve_actor(key, Native).variables[4] as i16) << 16;
        horizontal(&mut runtime, current, speed);
        detach_child(&mut runtime);
    }
}
pub(crate) fn shot(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    combat::shot_reaction(&mut runtime, api::ShotHandling::Finalize);
    detach_child(&mut runtime);
}
pub(crate) fn instruction(command: u16, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    match command {
        0x9381 => {
            let current = selected(&runtime);
            let riding = api::touches_player_from_above(&runtime, current);
            if riding {
                let current = selected(&runtime);
                let speed = runtime.resolve_actor(current, Native).variables[4];
                displacement(&mut runtime, fields::extra_samus_x_displacement, speed);
            }
            let key = selected(&runtime);
            let old_y = runtime.edit_actor(key, Native, |actor| {
                let old = actor.y.whole;
                actor.y.whole = actor.variables[3];
                old
            });
            let current = selected(&runtime);
            let speed = i32::from(runtime.resolve_actor(key, Native).variables[4] as i16) << 16;
            horizontal(&mut runtime, current, speed);
            let parent = Key::from_legacy(runtime.resolve_actor(key, Native).variables[1]);
            if runtime.resolve_actor(parent, Native).variables[0] == 0x8e0a {
                let parent = Key::from_legacy(runtime.resolve_actor(key, Native).variables[1]);
                let x = runtime.resolve_actor(parent, Native).x.whole;
                let delta = x.wrapping_sub(runtime.resolve_actor(key, Native).x.whole);
                let magnitude = abs16(delta);
                let step = if magnitude < 24 {
                    SHELL_HEIGHT[usize::from(magnitude + if delta & 0x8000 != 0 { 24 } else { 0 })]
                } else {
                    1
                };
                let current = selected(&runtime);
                vertical(&mut runtime, current, i32::from(step) << 16);
                if riding {
                    let current = selected(&runtime);
                    let y = runtime.resolve_actor(current, Native).y.whole;
                    displacement(
                        &mut runtime,
                        fields::extra_samus_y_displacement,
                        y.wrapping_sub(old_y),
                    );
                }
            }
        }
        0x9412 => {
            let key = selected(&runtime);
            runtime.edit_actor(key, Native, |actor| {
                let delta = actor.variables[2].wrapping_sub(actor.x.whole);
                if (abs16(delta) as i16).wrapping_sub(48) >= 0 {
                    actor.variables[4] = if delta & 0x8000 == 0 { 1 } else { u16::MAX };
                }
            });
            let key = selected(&runtime);
            return EnemyScript::jump(
                if runtime.resolve_actor(key, Native).variables[4] & 0x8000 != 0 {
                    0x8b80
                } else {
                    0x8c72
                },
            );
        }
        0x9447 | 0x94c7 => {
            let key = selected(&runtime);
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[0] = if command == 0x9447 { 0x8f3f } else { 0x9239 }
            });
        }
        0x9451 | 0x946b => {
            let raw = runtime.word(fields::cur_enemy_index);
            runtime.edit_actor(Key::from_legacy(raw), Native, |actor| {
                actor.variables[0] = 0x8f8d;
                actor.variables[4] = if command == 0x9451 { u16::MAX } else { 1 };
            });
            runtime.write_slice_native(0x7800 + usize::from(raw), 16);
            return EnemyScript::jump(0x8c02);
        }
        0x9485 => {
            let key = selected(&runtime);
            if api::touches_player_from_above(&runtime, key) {
                let key = selected(&runtime);
                return EnemyScript::jump(
                    if runtime.resolve_actor(key, Native).variables[4] & 0x8000 == 0 {
                        0x8d40
                    } else {
                        0x8c62
                    },
                );
            }
        }
        0x94a1 => {
            let key = selected(&runtime);
            let current = selected(&runtime);
            let riding = api::touches_player_from_above(&runtime, current);
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[0] = if riding { 0x925e } else { 0x9142 }
            });
            return EnemyScript::jump(
                if runtime.resolve_actor(key, Native).variables[4] & 0x8000 == 0 {
                    0x8c72
                } else {
                    0x8b80
                },
            );
        }
        _ => unreachable!("unknown turtle instruction"),
    }
    script
}
