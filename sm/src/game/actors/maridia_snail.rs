//! Maridia Snail crawling, falls and player reactions share one movement lifecycle.
#![forbid(unsafe_code)]
use crate::game::actors::{api, combat, wall_crawlers};
use crate::game::music::QueueSfx2_Max3;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::{word_arrays, words};
use sm_game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use sm_game::enemy_script::EnemyScript;
use sm_game::player_projectiles::content::{self, table};

fn extra(key: Key) -> usize {
    0x7800 + usize::from(key.byte_index())
}
fn definition(runtime: &Runtime<'_>, orientation: u16, field: isize) -> u16 {
    content::captured(runtime.native_image(), 0xa3cd42)
        .table_word(isize::try_from(orientation).unwrap() * 4 + field)
}
fn speed(runtime: &mut Runtime<'_>, key: Key, offset: u16) {
    let index = isize::try_from(offset >> 1).unwrap();
    let masks = [
        table(runtime, 0xa3cd82, index),
        table(runtime, 0xa3cd82, index + 1),
        table(runtime, 0xa3cd82, index + 2),
        table(runtime, 0xa3cd82, index + 3),
    ];
    for axis in 0..2 {
        let parameter = runtime.resolve_actor(key, Native).parameters[0];
        let value = masks[axis * 2 + 1]
            .wrapping_add(masks[axis * 2] ^ table(runtime, 0xa3cca2, parameter as isize));
        runtime.edit_actor(key, Native, |actor| actor.variables[axis] = value);
    }
}
fn animation_pair(runtime: &mut Runtime<'_>, key: Key, address: u32, index: isize) {
    let instruction = table(runtime, address, index);
    runtime.edit_actor(key, Native, |actor| actor.instruction = instruction);
    let resting = table(runtime, address, index + 1);
    runtime.edit_actor(key, Native, |actor| actor.variables[3] = resting);
}
fn reaction(runtime: &mut Runtime<'_>, key: Key, phase: u16, address: u32) {
    runtime.write_slice_native(extra(key) + 16, phase);
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = 0xd1b3);
    let facing = runtime.resolve_actor(key, Native).variables[2];
    let index = if phase == 3 {
        2 * facing as isize
    } else {
        (facing.wrapping_mul(4) >> 1) as isize
    };
    animation_pair(runtime, key, address, index);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction_delay = 1;
        actor.timer = 0;
    });
}
fn fall(runtime: &mut Runtime<'_>, key: Key) {
    if runtime.native_word(extra(key) + 16) != 3 {
        reaction(runtime, key, 3, 0xa3d1ab);
        for offset in [4, 6, 0, 2] {
            runtime.write_slice_native(extra(key) + offset, 0);
        }
    }
}
fn orient(runtime: &mut Runtime<'_>, key: Key, orientation: u16, replace: bool) {
    let instruction = definition(runtime, orientation, 0);
    runtime.edit_actor(key, Native, |actor| actor.instruction = instruction);
    let properties = definition(runtime, orientation, 1);
    runtime.edit_actor(key, Native, |actor| {
        actor.properties = properties
            | (if replace {
                actor.properties & !3
            } else {
                actor.properties
            })
    });
    for index in [3, 2] {
        let value = definition(runtime, orientation, if index == 3 { 2 } else { 3 });
        runtime.edit_actor(key, Native, |actor| actor.variables[index] = value);
    }
}
fn reverse(runtime: &mut Runtime<'_>, key: Key) {
    let at = extra(key);
    if runtime.native_word(at + 16) == 2
        || runtime.resolve_actor(key, Native).pre_instruction == 0xcf5f
    {
        return;
    }
    let orientation = table(runtime, 0xa3cdc2, runtime.native_word(at + 14) as isize);
    runtime.write_slice_native(at + 14, orientation);
    orient(runtime, key, orientation, true);
    speed(runtime, key, orientation.wrapping_mul(8));
    let phase = table(runtime, 0xa3cdd2, runtime.native_word(at + 14) as isize);
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = phase);
    slope_cache(runtime, key, true);
}
fn face_player(runtime: &mut Runtime<'_>, key: Key) {
    let actor = runtime.resolve_actor(key, Native);
    let reverse_facing = if actor.variables[2] != 0 {
        actor.x.whole >= runtime.session().player.x
    } else {
        actor.x.whole < runtime.session().player.x
    };
    if reverse_facing {
        reverse(runtime, key);
    }
}
fn input_matches(runtime: &Runtime<'_>, key: Key) -> bool {
    let facing = runtime.resolve_actor(key, Native).variables[2] & 1;
    if (runtime.word(words::joypad1_lastkeys) & 0x300) >> 8 == 1 {
        facing == 0
    } else {
        facing != 0
    }
}
fn slope_cache(runtime: &mut Runtime<'_>, key: Key, on_slope: bool) {
    let at = extra(key);
    if on_slope {
        runtime.write_slice_native(at + 10, 1);
        runtime.write_slice_native(at + 8, 0);
    } else if runtime.native_word(at + 8).wrapping_add(1) >= 16 {
        runtime.write_slice_native(at + 10, 0);
    } else {
        runtime.write_slice_native(at + 8, runtime.native_word(at + 8).wrapping_add(1));
    }
}
fn snap(runtime: &mut Runtime<'_>, key: Key) {
    let on_slope = api::snap_to_slopes(runtime, key);
    slope_cache(runtime, key, on_slope);
}
fn animate_turn(runtime: &mut Runtime<'_>, key: Key, instruction: u16) {
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = instruction;
        actor.instruction_delay = 1;
    });
    slope_cache(runtime, key, true);
}
fn adjust(runtime: &mut Runtime<'_>, key: Key, read: impl Fn(usize) -> u16, undo: bool) {
    for axis in 0..2 {
        let position = {
            let actor = runtime.resolve_actor(key, Native);
            if axis == 0 {
                actor.x.whole
            } else {
                actor.y.whole
            }
        };
        let offset = read(axis * 2);
        let position = if undo {
            position.wrapping_sub(offset)
        } else {
            position.wrapping_add(offset)
        };
        runtime.edit_actor(key, Native, |actor| {
            if axis == 0 {
                actor.x.whole = position;
            } else {
                actor.y.whole = position;
            }
        });
    }
}
fn velocity(runtime: &Runtime<'_>, key: Key, axis: usize, leading: i32) -> i32 {
    let speed = i32::from(runtime.resolve_actor(key, Native).variables[axis] as i16) << 8;
    speed
        + if speed < 0 {
            -(leading << 16)
        } else {
            leading << 16
        }
}
fn move_axis(runtime: &mut Runtime<'_>, key: Key, axis: usize, amount: i32) -> bool {
    (if axis == 0 {
        api::move_horizontal(runtime, key, amount, api::SlopePolicy::Follow)
    } else {
        api::move_vertical(runtime, key, amount)
    }) == api::MovementResult::Blocked
}
pub(crate) fn crawl(raw: u16, axis: api::Axis, address: u32, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(raw);
    let axis = usize::from(axis == api::Axis::Y);
    let address = if axis == 1 && runtime.native_word(extra(key) + 10) != 0 {
        0xa3cd22 + (address - 0xa3cce2) / 2
    } else {
        address
    };
    let record = content::captured(runtime.native_image(), address);
    adjust(&mut runtime, key, |offset| record.word(offset), false);
    let amount = velocity(&runtime, key, axis, 1);
    let blocked = move_axis(&mut runtime, key, axis, amount);
    adjust(&mut runtime, key, |offset| record.word(offset), true);
    if blocked {
        runtime.edit_actor(key, Native, |actor| actor.variables[4] = 0);
        if axis == 0 {
            snap(&mut runtime, key);
        }
        let amount = velocity(&runtime, key, 1 - axis, 0);
        if move_axis(&mut runtime, key, 1 - axis, amount) {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[axis] = actor.variables[axis].wrapping_neg()
            });
            animate_turn(&mut runtime, key, record.word(6));
        } else if axis == 1 {
            snap(&mut runtime, key);
        }
    } else {
        let edges = runtime.edit_actor(key, Native, |actor| {
            actor.variables[4] = actor.variables[4].wrapping_add(1);
            actor.variables[4]
        });
        if edges.wrapping_sub(4) & 0x8000 != 0 {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[1 - axis] = actor.variables[1 - axis].wrapping_neg()
            });
            animate_turn(&mut runtime, key, record.word(4));
        } else {
            fall(&mut runtime, key);
        }
    }
}
pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.pre_instruction = 0xcf5f;
        actor.spritemap = 0x804d;
        actor.instruction_delay = 1;
    });
    let orientation = runtime.resolve_actor(key, Native).instruction;
    orient(&mut runtime, key, orientation, false);
    runtime.write_slice_native(extra(key) + 16, 0);
    runtime.write_slice_native(
        extra(key) + 12,
        runtime.resolve_actor(key, Native).parameters[0],
    );
    let current = combat::current(&runtime);
    speed(&mut runtime, current, orientation.wrapping_mul(8));
}
fn react_to_player(runtime: &mut Runtime<'_>, key: Key) {
    let at = extra(key);
    if !matches!(runtime.native_word(at + 16), 1 | 3 | 4 | 5) {
        let actor = runtime.resolve_actor(key, Native);
        if (i32::from(actor.y.whole) - i32::from(runtime.session().player.y) + 96) & 0x8000 == 0 {
            let direction =
                if (actor.x.whole as i16).wrapping_sub(runtime.session().player.x as i16) < 0 {
                    4
                } else {
                    8
                };
            if runtime.byte(0xa1e) == direction {
                if runtime.native_word(at + 16) == 2 {
                    return;
                }
                if runtime.resolve_actor(key, Native).variables[3] != 0xcf5f {
                    runtime.edit_actor(key, Native, |actor| {
                        actor.instruction = actor.variables[3];
                        actor.spritemap = 0x804d;
                        actor.instruction_delay = 1;
                        actor.timer = 0;
                    });
                    runtime.write_slice_native(at + 16, 2);
                    return;
                }
            }
        }
        runtime.write_slice_native(at + 16, 0);
    }
}
fn contact_properties(runtime: &mut Runtime<'_>, key: Key) {
    let mut solid = false;
    if runtime.resolve_actor(key, Native).pre_instruction != 0xd1b3 {
        solid = if runtime.native_word(extra(key) + 16) == 1 {
            runtime.word(words::samus_has_momentum_flag) == 0
                && runtime.resolve_actor(key, Native).pre_instruction == 0xcf5f
        } else {
            runtime.word(words::samus_has_momentum_flag) == 0 && {
                let phase = runtime.native_word(extra(key) + 16);
                phase == 2 || phase != 3 && phase != 5
            }
        };
    }
    runtime.edit_actor(key, Native, |actor| {
        if solid {
            actor.properties |= 0x8000;
        } else {
            actor.properties &= 0x7fff;
        }
    });
}
pub(crate) fn prepare(game: &mut Game) -> u32 {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    if !matches!(runtime.native_word(extra(key) + 16), 3 | 4 | 5)
        && runtime.session().feedback.earthquake_timer == 30
        && runtime.session().feedback.earthquake_kind == 20
    {
        fall(&mut runtime, key);
    }
    let key = combat::current(&runtime);
    react_to_player(&mut runtime, key);
    let key = combat::current(&runtime);
    contact_properties(&mut runtime, key);
    let key = combat::current(&runtime);
    u32::from(runtime.resolve_actor(key, Native).pre_instruction) | 0xa30000
}
pub(crate) fn probe_edge(raw: u16, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(raw);
    let axis = usize::from(runtime.native_word(extra(key) + 14) >= 4);
    let amount = velocity(&runtime, key, axis, 7);
    let result = if axis == 1 {
        api::probe_vertical(&mut runtime, key, amount)
    } else {
        api::probe_horizontal(&mut runtime, key, amount)
    };
    if result != api::MovementResult::Blocked {
        fall(&mut runtime, key);
    }
}
fn pair(runtime: &Runtime<'_>, at: usize) -> u32 {
    (u32::from(runtime.native_word(at + 2)) << 16) | u32::from(runtime.native_word(at))
}
fn negate_pair(runtime: &mut Runtime<'_>, at: usize) {
    for offset in [0, 2] {
        runtime.write_slice_native(at + offset, runtime.native_word(at + offset).wrapping_neg());
    }
}
fn sound(runtime: &mut Runtime<'_>) {
    runtime.foreign(|game| QueueSfx2_Max3(0x70, game));
}
pub(crate) fn falling(raw: u16, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(raw);
    let at = extra(key);
    if runtime.native_word(at + 16) != 3 {
        let amount = pair(&runtime, at + 4) as i32;
        if api::move_horizontal(&mut runtime, key, amount, api::SlopePolicy::Ignore)
            == api::MovementResult::Blocked
        {
            negate_pair(&mut runtime, at + 4);
            runtime.write_slice_native(at + 0x800, 1);
            sound(&mut runtime);
        } else {
            let delta = if runtime.native_word(at + 6) & 0x8000 != 0 {
                0x1000
            } else {
                -0x1000i32 as u32
            };
            let next = pair(&runtime, at + 4).wrapping_add(delta);
            runtime.write_slice_native(at + 4, next as u16);
            if next >> 16 != 0 {
                runtime.write_slice_native(at + 6, (next >> 16) as u16);
            }
        }
    }
    let amount = pair(&runtime, at) as i32;
    if api::move_vertical(&mut runtime, key, amount) == api::MovementResult::Blocked {
        if (0..3).contains(&(runtime.native_word(at + 2) as i16)) {
            for offset in [6, 4, 2, 0] {
                runtime.write_slice_native(at + offset, 0);
            }
            runtime.edit_actor(key, Native, |actor| actor.variables[4] = 0);
            runtime.write_slice_native(at + 8, 0);
            runtime.write_slice_native(at + 10, 1);
            if runtime.native_word(at + 16) == 3 {
                runtime.write_slice_native(at + 16, 0);
            } else {
                runtime.write_slice_native(at + 16, 1);
                runtime.edit_actor(key, Native, |actor| actor.parameters[0] = 8);
                face_player(&mut runtime, key);
                let index =
                    (runtime.resolve_actor(key, Native).variables[2].wrapping_mul(4) >> 1) as isize;
                animation_pair(&mut runtime, key, 0xa3d30d, index);
            }
            runtime.edit_actor(key, Native, |actor| {
                actor.properties = actor.variables[2] | actor.properties & !3
            });
            runtime.foreign(|game| wall_crawlers::prepare(raw, game));
            runtime.edit_actor(key, Native, |actor| {
                actor.pre_instruction = 0xcf5f;
                actor.instruction = actor.variables[3];
                actor.instruction_delay = 1;
                actor.timer = 0;
            });
        } else {
            negate_pair(&mut runtime, at);
            runtime.write_slice_native(at + 0x800, 0);
        }
    } else {
        let next = pair(&runtime, at).wrapping_add(0x2000);
        runtime.write_slice_native(at, next as u16);
        if (next >> 16).wrapping_sub(4) & 0x8000 != 0 {
            runtime.write_slice_native(at + 2, (next >> 16) as u16);
        }
    }
}
fn pushed(runtime: &mut Runtime<'_>, key: Key) {
    reaction(runtime, key, 4, 0xa3d50f);
    let at = extra(key);
    runtime.write_slice_native(
        at + 4,
        runtime.word(words::absolute_moved_last_frame_x_fract),
    );
    let distance = runtime.word(words::absolute_moved_last_frame_x);
    runtime.write_slice_native(at + 6, runtime.word(words::absolute_moved_last_frame_x));
    let index = (distance.min(15).wrapping_mul(4) >> 1) as isize;
    runtime.write_slice_native(at, table(runtime, 0xa3d517, index));
    runtime.write_slice_native(at + 2, table(runtime, 0xa3d517, index + 1));
    if runtime.byte(0xa1e) & 4 != 0 {
        negate_pair(runtime, at + 4);
    }
}
pub(crate) fn touch(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    let at = extra(key);
    if (runtime.native_word(at + 16) != 1
        || runtime.resolve_actor(key, Native).pre_instruction == 0xcf5f
        || !input_matches(&runtime, combat::current(&runtime)))
        && (runtime.resolve_actor(key, Native).pre_instruction == 0xd1b3
            || runtime.word(words::samus_has_momentum_flag) != 0)
    {
        let current = combat::current(&runtime);
        pushed(&mut runtime, current);
        if runtime.resolve_actor(key, Native).pre_instruction == 0xd1b3 {
            sound(&mut runtime);
        }
    } else if runtime.resolve_actor(key, Native).pre_instruction != 0xcf5f
        && runtime.native_word(at + 16) != 4
        && runtime.native_word(at + 16) != 3
    {
        api::contact_reaction(&mut runtime, api::DeathHandling::Finalize);
        let parameter = runtime.native_word(at + 12);
        runtime.edit_actor(key, Native, |actor| actor.parameters[0] = parameter);
        if runtime.native_word(at + 16) != 0 {
            let current = combat::current(&runtime);
            reverse(&mut runtime, current);
        }
        runtime.write_slice_native(at + 16, 0);
    }
}
pub(crate) fn shot(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let index = usize::from(runtime.word(words::collision_detection_index));
    let kind = runtime.word(word_arrays::projectile_type + 2 * index) & 0xff00;
    if kind == 0x300 || kind == 0x500 {
        combat::shot_reaction(&mut runtime, api::ShotHandling::Finalize);
    } else {
        let key = combat::current(&runtime);
        if !matches!(runtime.native_word(extra(key) + 16), 3 | 4) {
            let key = combat::current(&runtime);
            reaction(&mut runtime, key, 5, 0xa3d5a4);
            runtime.write_slice_native(extra(key) + 2, 0xffff);
            runtime.write_slice_native(
                extra(key) + 6,
                if runtime.byte(0xa1e) == 4 { 0xffff } else { 1 },
            );
        }
        sound(&mut runtime);
    }
}
pub(crate) fn instruction(
    raw: u16,
    command: u8,
    script: EnemyScript,
    game: &mut Game,
) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(raw);
    match command {
        0 | 1 => {
            let value = script.word(0);
            runtime.edit_actor(key, Native, |actor| {
                if command == 0 {
                    actor.pre_instruction = value;
                } else {
                    actor.variables[3] = value;
                }
            });
        }
        2 => {
            runtime.write_slice_native(extra(key) + 14, script.word(0));
            let facing = definition(&runtime, runtime.native_word(extra(key) + 14), 3);
            runtime.edit_actor(key, Native, |actor| actor.variables[2] = facing);
        }
        3 => {
            adjust(
                &mut runtime,
                key,
                |offset| script.word(offset as isize),
                false,
            );
            return script.offset(2);
        }
        4 => {
            return if runtime.native_word(extra(key) + 16) == 2
                || runtime.foreign(|game| game.next_random()) & 1 != 0
            {
                script.offset(-3)
            } else {
                script
            };
        }
        _ => unreachable!("unknown Maridia Snail instruction"),
    }
    script.offset(1)
}
