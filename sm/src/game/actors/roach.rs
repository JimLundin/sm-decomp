//! Roach flight shares velocity preparation, animation, and retargeting paths.
#![forbid(unsafe_code)]
use super::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use super::{api, combat};
use crate::game::enemy_projectiles::host::Effects;
use crate::game::player_projectiles::content::table;
use crate::game::session_runtime::{Runtime, actor_cosine_full, actor_sine_full};
use crate::game_state::Game;
use sm_game::geometry::Vector32;

fn selected(runtime: &Runtime<'_>) -> (Key, usize) {
    let key = combat::current(runtime);
    (key, 0x7800 + usize::from(key.byte_index()))
}
fn pair(runtime: &Runtime<'_>, at: usize) -> u32 {
    (u32::from(runtime.native_word(at + 2)) << 16) | u32::from(runtime.native_word(at))
}
fn store_pair(runtime: &mut Runtime<'_>, at: usize, value: u32) {
    runtime.write_slice_native(at, value as u16);
    runtime.write_slice_native(at + 2, (value >> 16) as u16);
}
fn animate(runtime: &mut Runtime<'_>) {
    let (key, at) = selected(runtime);
    let instruction = runtime.native_word(at + 0x808);
    if instruction != runtime.native_word(at + 0x80a) {
        runtime.write_slice_native(at + 0x80a, instruction);
        runtime.edit_actor(key, Native, |actor| actor.restart_animation(instruction));
    }
}
fn select_animation(runtime: &mut Runtime<'_>, at: usize, direction: u16) {
    let instruction = table(runtime, 0xa3a111, (direction >> 1) as isize);
    runtime.write_slice_native(at + 0x808, instruction);
    animate(runtime);
}
fn trig_velocity(runtime: &mut Runtime<'_>, offset: usize, adjustment: Option<u8>) {
    let (key, at) = selected(runtime);
    for (axis_offset, cosine) in [(0, true), (4, false)] {
        let (angle, speed) = {
            let actor = runtime.resolve_actor(key, Native);
            let angle = match adjustment {
                Some(delta) => u16::from(actor.parameters[0].to_ne_bytes()[1].wrapping_add(delta)),
                None => actor.variables[2],
            };
            (angle, u16::from(actor.parameters[0].to_ne_bytes()[0]))
        };
        let value = runtime.foreign(|game| {
            if cosine {
                actor_cosine_full(angle, speed, game)
            } else {
                actor_sine_full(angle, speed, game)
            }
        });
        store_pair(runtime, at + offset + axis_offset, value);
    }
}
fn retarget(runtime: &mut Runtime<'_>, at: usize) {
    trig_velocity(runtime, 24, None);
    let (key, current_at) = selected(runtime);
    let angle = runtime.resolve_actor(key, Native).variables[2] as u8;
    runtime.write_slice_native(
        current_at + 0x806,
        2 * u16::from(angle.wrapping_sub(48) >> 5),
    );
    select_animation(runtime, at, runtime.native_word(at + 0x806));
}
fn translate(runtime: &mut Runtime<'_>, offset: usize) {
    let (key, at) = selected(runtime);
    let x = pair(runtime, at + offset);
    runtime.edit_actor(key, Native, |actor| actor.x.advance(x as i32));
    let y = pair(runtime, at + offset + 4);
    runtime.edit_actor(key, Native, |actor| actor.y.advance(y as i32));
}
fn initial_flight(runtime: &mut Runtime<'_>) {
    let (key, at) = selected(runtime);
    let angle = u16::from(runtime.resolve_actor(key, Native).parameters[0].to_ne_bytes()[1]);
    let point = Vector32 {
        x: pair(runtime, at),
        y: pair(runtime, at + 4),
    };
    runtime.session_mut().compatibility_enemy_scratch = angle;
    api::apply_vector(runtime, angle, point);
}
fn reflect(runtime: &mut Runtime<'_>) {
    let (_, at) = selected(runtime);
    // Each native word is negated independently; this is not 32-bit negation.
    for offset in [26, 24, 30, 28] {
        runtime.write_slice_native(at + offset, runtime.native_word(at + offset).wrapping_neg());
    }
    let direction = runtime.native_word(at + 0x806).wrapping_add(4) & 7;
    runtime.write_slice_native(at + 0x806, direction);
    select_animation(runtime, at, direction);
}
pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (key, at) = selected(&runtime);
    runtime.write_slice_native(at + 0x80a, 0);
    runtime.write_slice_native(at + 0x808, 0);
    let (current, current_at) = selected(&runtime);
    let parameter = runtime.resolve_actor(current, Native).parameters[0].to_ne_bytes();
    let point = runtime.convert_angle_to_xy(u16::from(parameter[1]), u16::from(parameter[0]));
    store_pair(&mut runtime, current_at, point.x);
    store_pair(&mut runtime, current_at + 4, point.y);
    trig_velocity(&mut runtime, 8, Some(0u8.wrapping_sub(32)));
    trig_velocity(&mut runtime, 16, Some(32));
    for (offset, adjustment) in [(0, 48), (2, 80), (4, 16)] {
        let (current, current_at) = selected(&runtime);
        let angle = runtime.resolve_actor(current, Native).parameters[0].to_ne_bytes()[1];
        runtime.write_slice_native(
            current_at + 0x800 + offset,
            2 * u16::from(angle.wrapping_sub(adjustment) >> 5),
        );
    }
    let direction = runtime.native_word(at + 0x800);
    select_animation(&mut runtime, at, direction);
    runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0xa2d7);
}
pub(crate) fn advance(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (key, at) = selected(&runtime);
    let phase = runtime.resolve_actor(key, Native).variables[1];
    match phase {
        0xa2d7 => {
            let mut near = true;
            for axis in [api::Axis::X, api::Axis::Y] {
                let current = combat::current(&runtime);
                let distance =
                    u16::from(runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes()[0]);
                if !api::player_within(&mut runtime, current, axis, distance) {
                    near = false;
                    break;
                }
            }
            if near {
                let index =
                    runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes()[1] as isize;
                let next = table(&runtime, 0xa3a121, index);
                runtime.edit_actor(key, Native, |actor| actor.variables[1] = next);
            }
        }
        0xa301 | 0xa30b => runtime.edit_actor(key, Native, |actor| {
            actor.variables[1] = if phase == 0xa301 { 0xa407 } else { 0xa40e }
        }),
        0xa315 | 0xa325 => {
            if phase == 0xa325 {
                runtime.edit_actor(key, Native, |actor| actor.variables[0] = 512);
            }
            runtime.write_slice_le(crate::ram_map::words::random_number, 11);
            let current = combat::current(&runtime);
            runtime.edit_actor(current, Native, |actor| {
                actor.variables[1] = if phase == 0xa315 { 0xa4f0 } else { 0xa4b6 }
            });
        }
        0xa33b => runtime.edit_actor(key, Native, |actor| {
            actor.pre_instruction = 32;
            actor.variables[1] = 0xa44e;
        }),
        0xa34b | 0xa380 | 0xa4b6 | 0xa4f0 => {
            let random = matches!(phase, 0xa4b6 | 0xa4f0);
            let angle = if random {
                runtime.next_random() as u8
            } else {
                let current = combat::current(&runtime);
                api::angle_to_player(&runtime, current) as u8
            };
            let (current, current_at) = selected(&runtime);
            runtime.edit_actor(current, Native, |actor| {
                actor.variables[2] = if random {
                    actor.variables[2].wrapping_add(u16::from(angle.wrapping_sub(64)))
                } else {
                    u16::from((if phase == 0xa34b { 64u8 } else { 192 }).wrapping_sub(angle))
                };
            });
            retarget(&mut runtime, current_at);
            runtime.edit_actor(current, Native, |actor| {
                if random {
                    actor.pre_instruction = 32;
                }
                actor.variables[1] = match phase {
                    0xa34b => 0xa440,
                    0xa380 => 0xa447,
                    0xa4b6 => 0xa476,
                    _ => 0xa462,
                };
            });
        }
        0xa407 => initial_flight(&mut runtime),
        0xa40e => {
            let second = runtime.resolve_actor(key, Native).frame & 0x10 != 0;
            let direction = runtime.native_word(at + if second { 0x804 } else { 0x802 });
            select_animation(&mut runtime, at, direction);
            translate(&mut runtime, if second { 16 } else { 8 });
        }
        0xa440 | 0xa447 => translate(&mut runtime, 24),
        0xa44e | 0xa462 => {
            let expired = runtime.edit_actor(key, Native, |actor| {
                actor.pre_instruction = actor.pre_instruction.wrapping_sub(1);
                actor.pre_instruction & 0x8000 != 0
            });
            if expired {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[1] = if phase == 0xa44e { 0xa2d7 } else { 0xa4f0 }
                });
            } else if phase == 0xa44e {
                initial_flight(&mut runtime);
            } else {
                let (current, current_at) = selected(&runtime);
                let x = pair(&runtime, current_at + 24) as i32;
                let mut blocked =
                    api::move_horizontal(&mut runtime, current, x, api::SlopePolicy::Ignore)
                        == api::MovementResult::Blocked;
                if !blocked {
                    let y = pair(&runtime, current_at + 28) as i32;
                    blocked = api::move_vertical(&mut runtime, current, y)
                        == api::MovementResult::Blocked;
                }
                if blocked {
                    runtime.edit_actor(current, Native, |actor| actor.variables[1] = 0xa2d7);
                }
            }
        }
        0xa476 => {
            let expired = runtime.edit_actor(key, Native, |actor| {
                actor.variables[0] = actor.variables[0].wrapping_sub(1);
                actor.variables[0] & 0x8000 != 0
            });
            if !expired {
                let expired = runtime.edit_actor(key, Native, |actor| {
                    actor.pre_instruction = actor.pre_instruction.wrapping_sub(1);
                    actor.pre_instruction & 0x8000 != 0
                });
                if expired {
                    runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0xa4b6);
                    return;
                }
                let far = [api::Axis::X, api::Axis::Y].into_iter().all(|axis| {
                    let actor = runtime.resolve_actor(key, Native);
                    let (position, player) = match axis {
                        api::Axis::X => (actor.x.whole, runtime.session().player.x),
                        api::Axis::Y => (actor.y.whole, runtime.session().player.y),
                    };
                    let distance = (position.wrapping_sub(player) as i16).wrapping_abs() as u16;
                    distance.wrapping_sub(96) & 0x8000 == 0
                });
                if far {
                    reflect(&mut runtime);
                }
            }
            translate(&mut runtime, 24);
        }
        _ => runtime.unreachable(),
    }
}
