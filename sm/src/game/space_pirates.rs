//! Wall, ninja and walking pirates share checked actor state and movement.
//! Extended species words remain RAM-authoritative.
#![forbid(unsafe_code)]
use crate::game::actors::api::{self, Axis, MovementResult};
use crate::game::actors::combat;
use crate::game::session_runtime::{Runtime, actor_cosine, actor_sine};
use crate::game::sprite_effect_runtime::create_sprite;
use crate::game_state::Game;
use crate::ram_map::{word, word_arrays as arrays, words as fields};
use crate::types::{EnemyDef, Point16U, Rect16U, Vulnerability};
use core::mem::offset_of;
use sm_game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use sm_game::enemy_projectiles::host::Effects;
use sm_game::enemy_script::EnemyScript;
use {
    crate::game::enemy_projectiles::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx, scatter_drop},
    sm_game::enemy_projectiles::drops::DropSource,
};

fn current(runtime: &Runtime<'_>) -> Key {
    Key::from_legacy(runtime.word(0xe54))
}

fn offset_x(x: u16, amount: u16, right: bool) -> u16 {
    x.wrapping_add(if right { amount } else { amount.wrapping_neg() })
}

fn snap_to_wall(x: u16) -> u16 {
    if x & 15 < 11 {
        x & 0xfff8
    } else {
        (x & 0xfff0).wrapping_add(16)
    }
}

fn initialize_wall(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let index = runtime.word(0xe54);
    let key = Key::from_legacy(index);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = if actor.parameters[0] & 1 == 0 {
            0xed36
        } else {
            0xedac
        };
    });
    for (at, value) in [(0x8000, 190), (0x8002, 66), (0x8004, 2)] {
        runtime.write_slice_native(at + usize::from(index), value);
    }
    if runtime.resolve_actor(key, Native).parameters[0] & 0x8000 == 0 {
        for (at, delta) in [(0x8000, 2u16), (0x8002, u16::MAX - 1), (0x8004, 2)] {
            let at = at + usize::from(index);
            runtime.write_slice_native(at, runtime.native_word(at).wrapping_add(delta));
        }
    }
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[0] = if actor.parameters[0] & 1 == 0 {
            0xf034
        } else {
            0xf0c8
        };
        actor.x.whole = snap_to_wall(actor.x.whole);
    });
}

fn wait_at_wall(right: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = current(&runtime);
    if api::player_within(&mut runtime, key, Axis::Y, 32) {
        let key = current(&runtime);
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction = if right { 0xed80 } else { 0xecc0 };
            actor.instruction_delay = 1;
        });
    }
}

fn jump_between_walls(right: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let index = runtime.word(0xe54);
    let key = Key::from_legacy(index);
    let actor = runtime.resolve_actor(key, Native);
    let (origin, phase, radius) = (
        actor.variables[3],
        actor.pre_instruction,
        actor.parameters[1] >> 1,
    );
    let dx = runtime.foreign(|game| actor_sine(phase, radius, game));
    runtime.edit_actor(key, Native, |actor| actor.x.whole = origin.wrapping_add(dx));
    let actor = runtime.resolve_actor(key, Native);
    let (origin, phase, radius) = (
        actor.variables[4],
        actor.pre_instruction,
        actor.parameters[1] >> 2,
    );
    let dy = runtime.foreign(|game| actor_cosine(phase, radius, game));
    runtime.edit_actor(key, Native, |actor| actor.y.whole = origin.wrapping_sub(dy));
    let phase = runtime
        .resolve_actor(key, Native)
        .pre_instruction
        .to_ne_bytes()[0];
    let step = runtime
        .native_word(0x8004 + usize::from(index))
        .to_ne_bytes()[0];
    let phase = u16::from(if right {
        phase.wrapping_sub(step)
    } else {
        phase.wrapping_add(step)
    });
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = phase);
    let end = runtime.native_word((if right { 0x8000 } else { 0x8002 }) + usize::from(index));
    if phase == end {
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction = if right { 0xeda4 } else { 0xece4 };
            actor.instruction_delay = 1;
            actor.x.whole = snap_to_wall(actor.x.whole);
        });
    }
}

fn wall_direction(right: bool, random: bool, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    let direction = if random {
        let value = runtime.next_random() & 1;
        let key = current(&runtime);
        runtime.edit_actor(key, Native, |actor| actor.variables[2] = value);
        value
    } else {
        let key = current(&runtime);
        let distance = (script.word(0) as i16 as i32) << 16;
        if api::move_vertical(&mut runtime, key, distance) != MovementResult::Blocked {
            return script.offset(1);
        }
        let key = current(&runtime);
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[2] ^= 1;
            actor.variables[2]
        })
    };
    EnemyScript::jump(match (right, direction != 0) {
        (true, false) => 0xed36,
        (true, true) => 0xecec,
        (false, false) => 0xedac,
        (false, true) => 0xedf6,
    })
}

fn prepare_wall_jump(right: bool, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    let key = current(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        let x = actor.x.whole;
        let distance = actor.parameters[1];
        actor.variables[1] = offset_x(x, distance, right);
        actor.variables[3] = offset_x(x, distance >> 1, right);
        actor.variables[4] = actor.y.whole;
        actor.pre_instruction = if right { 64 } else { 192 };
    });
    script
}

fn within(delta: u16, distance: i16) -> bool {
    (delta as i16).wrapping_abs().wrapping_sub(distance) < 0
}

fn react_ninja(check_center: bool, game: &mut Game) -> bool {
    let mut runtime = Runtime::new(game);
    let key = current(&runtime);
    let actor = runtime.resolve_actor(key, Native);
    let player = &runtime.session().player;
    let instruction = if within(player.x.wrapping_sub(actor.x.whole), 40)
        && within(player.y.wrapping_sub(actor.y.whole), 40)
    {
        if (actor.x.whole.wrapping_sub(player.x) as i16) < 0 {
            0xf51a
        } else {
            0xf32e
        }
    } else if check_center && within(actor.variables[3].wrapping_sub(player.x), 32) {
        if actor.x.whole == actor.variables[4] {
            0xf3b2
        } else {
            0xf1c4
        }
    } else {
        return false;
    };
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = instruction;
        actor.instruction_delay = 1;
    });
    true
}

fn dodge_projectile(walking: bool, game: &mut Game) -> bool {
    let mut runtime = Runtime::new(game);
    let Some(shot) = runtime.session().projectiles.shots[..5]
        .iter()
        .rev()
        .find(|shot| shot.kind != 0)
    else {
        return false;
    };
    let key = current(&runtime);
    let actor = runtime.resolve_actor(key, Native);
    if !within(shot.x.wrapping_sub(actor.x.whole), 32)
        || !within(shot.y.wrapping_sub(actor.y.whole), 32)
    {
        return false;
    }
    let left = (actor.x.whole.wrapping_sub(runtime.session().player.x) as i16) < 0;
    let [right_instruction, left_instruction] = if walking {
        [0xfb4c, 0xfb58]
    } else {
        [0xf270, 0xf462]
    };
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = if left {
            left_instruction
        } else {
            right_instruction
        };
        actor.instruction_delay = 1;
    });
    true
}

fn fire_laser(index: u16, right: bool, height: u16, game: &mut Game) {
    let point = {
        let runtime = Runtime::new(game);
        let actor = runtime.resolve_actor(Key::from_legacy(index), Native);
        Point16U {
            x: offset_x(actor.x.whole, 24, right),
            y: actor.y.whole.wrapping_sub(height),
        }
    };
    game.eproj_spawn_pt = point;
    game.eproj_spawn_r22 = u16::from(right);
    SpawnEprojWithRoomGfx(0xa17b, u16::from(right), game);
}

fn landing_dust(runtime: &mut Runtime<'_>, key: Key) {
    for dx in [0xfff8, 8] {
        let actor = runtime.resolve_actor(key, Native);
        let (x, y) = (
            actor.x.whole.wrapping_add(dx),
            actor.y.whole.wrapping_add(28),
        );
        runtime.foreign(|game| create_sprite(x, y, 10, 0, game));
    }
}

fn somersault(index: u16, right: bool, descending: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(index);
    let velocity_at = 0x7800 + usize::from(index);
    let speed = u16::from(runtime.native_word(velocity_at).to_ne_bytes()[1]);
    runtime.edit_actor(key, Native, |actor| {
        actor.x.whole = offset_x(actor.x.whole, speed, right);
    });
    for _ in 0..2 {
        runtime.edit_actor(key, Native, |actor| {
            actor.y.whole = actor
                .y
                .whole
                .wrapping_add(if descending { 1 } else { u16::MAX });
        });
    }
    let velocity = runtime
        .native_word(velocity_at)
        .wrapping_add(if descending { 0xffe0 } else { 32 });
    runtime.write_slice_native(velocity_at, velocity);
    if descending {
        if velocity == 0 {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[0] = 0x804b;
                actor.instruction = if right { 0xf4ea } else { 0xf2f8 };
                actor.instruction_delay = 1;
                actor.x.whole = if right {
                    actor.pre_instruction
                } else {
                    actor.variables[4]
                };
            });
            landing_dust(&mut runtime, key);
        }
    } else {
        runtime.edit_actor(key, Native, |actor| {
            if ((actor.x.whole.wrapping_sub(actor.variables[3]) as i16) >= 0) == right {
                actor.variables[0] = if right { 0xf8c5 } else { 0xf84c };
            }
        });
    }
}

enum JumpPhase {
    Rise,
    Drift,
    Recover,
}

fn vertical_jump(index: u16, right: bool, phase: JumpPhase, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(index);
    let velocity_at = 0x7800 + usize::from(index);
    match phase {
        JumpPhase::Rise => {
            let speed = u16::from(runtime.native_word(velocity_at).to_ne_bytes()[1]);
            runtime.edit_actor(key, Native, |actor| {
                actor.y.whole = actor.y.whole.wrapping_sub(speed)
            });
            let velocity = (runtime.native_word(velocity_at) as i16).wrapping_sub(64);
            runtime.write_slice_native(velocity_at, velocity as u16);
            if velocity < 0 {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[0] = if right { 0xfa95 } else { 0xf9c1 };
                    actor.instruction = if right { 0xf492 } else { 0xf2a0 };
                    actor.instruction_delay = 1;
                });
                runtime.write_slice_native(velocity_at, 1536);
            }
        }
        JumpPhase::Drift => {
            runtime.edit_actor(key, Native, |actor| {
                actor.x.whole = offset_x(actor.x.whole, 5, right);
            });
            let [fraction, whole] = runtime.native_word(velocity_at).to_ne_bytes();
            let amount = (i32::from(whole) << 16) | i32::from(fraction);
            let finished = api::move_vertical(&mut runtime, key, amount) == MovementResult::Blocked
                || {
                    let velocity = (runtime.native_word(velocity_at) as i16).wrapping_sub(64);
                    runtime.write_slice_native(velocity_at, velocity as u16);
                    velocity < 0 || velocity as u16 & 0xff00 == 0
                };
            if finished {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[0] = if right { 0xfae9 } else { 0xfa15 };
                    actor.instruction = if right { 0xf4a4 } else { 0xf2b2 };
                    actor.instruction_delay = 1;
                });
                let y = runtime.native_word(0x7810 + usize::from(index));
                runtime.edit_actor(key, Native, |actor| actor.y.whole = y);
                landing_dust(&mut runtime, key);
            }
        }
        JumpPhase::Recover => {
            runtime.edit_actor(key, Native, |actor| {
                actor.x.whole = offset_x(actor.x.whole, 2, right);
                let target = if right {
                    actor.pre_instruction
                } else {
                    actor.variables[4]
                };
                if ((actor.x.whole.wrapping_sub(target) as i16) >= 0) == right {
                    actor.x.whole = target;
                    actor.instruction = if right { 0xf4ea } else { 0xf2f8 };
                    actor.instruction_delay = 1;
                    actor.variables[0] = 0x804b;
                }
            });
        }
    }
}

fn reset_ninja_jump(index: u16, game: &mut Game) {
    Runtime::new(game).write_slice_native(0x7800 + usize::from(index), 0);
}

fn prepare_ninja_jump(index: u16, right: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    runtime.write_slice_native(0x7800 + usize::from(index), 1536);
    let actor = runtime.resolve_actor(Key::from_legacy(index), Native);
    let (start, end) = if right {
        (actor.variables[4], actor.variables[3])
    } else {
        (actor.variables[3], actor.pre_instruction)
    };
    let midpoint = start.wrapping_add(end.wrapping_sub(start) >> 1);
    runtime.write_slice_native(0x7806 + usize::from(index), midpoint);
}

fn initialize_ninja(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let index = runtime.word(0xe54);
    let y = runtime.edit_actor(Key::from_legacy(index), Native, |actor| {
        let right = actor.parameters[0] & 1 != 0;
        actor.instruction = if right { 0xf4cc } else { 0xf2da };
        actor.variables[2] = actor.instruction;
        let span = actor.parameters[1];
        let left = if right {
            actor.x.whole
        } else {
            actor.x.whole.wrapping_sub(span)
        };
        actor.variables[3] = left.wrapping_add(span >> 1);
        let threshold = u16::from((span >> 1) as u8) << 8;
        let (mut velocity, mut distance) = (0u16, 0u16);
        loop {
            velocity = velocity.wrapping_add(32);
            distance = distance.wrapping_add(velocity);
            if (distance.wrapping_sub(threshold) as i16) >= 0 {
                break;
            }
        }
        actor.variables[1] = velocity;
        actor.pre_instruction = actor.variables[3].wrapping_add(distance >> 8);
        actor.variables[4] = actor.variables[3].wrapping_sub(distance >> 8);
        actor.x.whole = if right {
            actor.variables[4]
        } else {
            actor.pre_instruction
        };
        actor.variables[0] = 0x804b;
        actor.y.whole
    });
    runtime.write_slice_native(0x7810 + usize::from(index), y);
    runtime.foreign(|game| {
        word_transfers!(game, ascending, 16; native(0xc3e0) <- rom(0xb28727, 0));
    });
}

fn wake_ninja(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = current(&runtime);
    let player_x = runtime.session().player.x;
    let x = runtime.resolve_actor(key, Native).x.whole;
    if within(x.wrapping_sub(player_x), 128) {
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction = if (x.wrapping_sub(player_x) as i16) < 0 {
                0xf420
            } else {
                0xf22e
            };
            actor.variables[2] = actor.instruction;
            actor.instruction_delay = 1;
        });
    } else {
        runtime.foreign(|game| dodge_projectile(false, game));
    }
}

fn select_somersault(index: u16, game: &mut Game) {
    if dodge_projectile(false, game) || react_ninja(true, game) {
        return;
    }
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(index);
    let actor = runtime.resolve_actor(key, Native);
    let left = actor.x.whole == actor.variables[4];
    if actor.frame & 0x3f != 0
        || ((actor.x.whole.wrapping_sub(runtime.session().player.x) as i16) >= 0) != left
    {
        return;
    }
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = if left { 0xf15c } else { 0xf34a };
        actor.instruction_delay = 1;
    });
}

fn select_leap(game: &mut Game) {
    if dodge_projectile(false, game) || react_ninja(false, game) {
        return;
    }
    let mut runtime = Runtime::new(game);
    let key = current(&runtime);
    if !within(
        runtime.resolve_actor(key, Native).variables[3].wrapping_sub(runtime.session().player.x),
        32,
    ) {
        return;
    }
    let choice = loop {
        let choice = runtime.next_random() & 3;
        if choice != 0 {
            break choice;
        }
    };
    let actor = runtime.resolve_actor(key, Native);
    let choice = choice
        + if actor.x.whole == actor.variables[4] {
            4
        } else {
            0
        };
    let instruction = runtime
        .foreign(|game| sm_game::gameplay_tables::pirate_instruction_list(game, choice as isize));
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = instruction;
        actor.instruction_delay = 1;
    });
}

fn damage(runtime: &mut Runtime<'_>) {
    let key = current(runtime);
    let x = runtime.resolve_actor(key, Native).x.whole;
    runtime.write_word_le(x, fields::special_death_item_drop_x_origin_pos);
    let y = runtime.resolve_actor(key, Native).y.whole;
    runtime.write_word_le(y, fields::special_death_item_drop_y_origin_pos);
    combat::shot_reaction(runtime, api::ShotHandling::ReactionOnly);
    if runtime.resolve_actor(key, Native).health == 0 {
        let index = runtime.word(fields::cur_enemy_index);
        let drop_items = runtime.resolve_actor(key, Native).definition == 0xf593;
        runtime.edit_actor(key, Native, |actor| actor.variables[1] = 0);
        combat::die(runtime, api::DeathKind::Normal { animation: 4 });
        if drop_items {
            runtime.foreign(|game| scatter_drop(DropSource::LowerNorfairPirate, index, game));
        }
    }
}

pub(crate) fn touch(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    if runtime.resolve_actor(current(&runtime), Native).frozen == 0 {
        combat::contact_reaction(&mut runtime, api::DeathHandling::Finalize);
    }
}

pub(crate) fn shot(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    if runtime.resolve_actor(current(&runtime), Native).definition == 0xf593 {
        damage(&mut runtime);
    } else {
        combat::shot_reaction(&mut runtime, api::ShotHandling::Finalize);
    }
}

pub(crate) fn check_shot(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    if runtime.resolve_actor(current(&runtime), Native).definition == 0xf593 {
        let index = usize::from(runtime.word(fields::collision_detection_index));
        let kind = runtime.word(arrays::projectile_type + 2 * index);
        let class = kind & 0xf00;
        if class >= 0x300 {
            return;
        }
        let definition = runtime.resolve_actor(current(&runtime), Native).definition;
        let vulnerability = runtime
            .actor_legacy_record(0xa0, definition)
            .word(offset_of!(EnemyDef, vulnerability_ptr));
        let vulnerability = if vulnerability == 0 {
            0xec1c
        } else {
            vulnerability
        };
        let (index, field) = if class == 0 {
            (kind & 15, offset_of!(Vulnerability, power))
        } else {
            (class >> 8, offset_of!(Vulnerability, plasma_ice_wave))
        };
        let coefficient = runtime
            .actor_legacy_record(0xb4, vulnerability.wrapping_add(index))
            .byte(field)
            & 15;
        if matches!(coefficient, 0 | 15) {
            reflect_shot(&mut runtime);
            return;
        }
    }
    damage(&mut runtime);
}

pub(crate) fn reflect(game: &mut Game) {
    reflect_shot(&mut Runtime::new(game));
}

fn reflect_shot(runtime: &mut Runtime<'_>) {
    if runtime.resolve_actor(current(runtime), Native).definition != 0xf593 {
        damage(runtime);
        return;
    }
    let reflection_index = runtime
        .word(fields::collision_detection_index)
        .wrapping_mul(2);
    // Array reads keep the unwrapped element index; the reflection argument wraps.
    let index = 2 * usize::from(runtime.word(fields::collision_detection_index));
    let class = runtime.word(arrays::projectile_type + index) & 0xf00;
    if class == 0x200 {
        if runtime.word(arrays::projectile_variables + index) == 0 {
            return;
        }
    } else if class >= 0x300 {
        return;
    }
    let key = current(runtime);
    runtime.edit_actor(key, Native, |actor| actor.invincibility = 10);
    let direction_at = arrays::projectile_dir + index;
    let direction = match runtime.word(direction_at) & 15 {
        7 => 1,
        2 => 8,
        _ => 5,
    };
    runtime.write_word_le(direction, direction_at);
    runtime.foreign(|game| crate::game::player_projectiles::reflect(reflection_index, game));
    runtime.queue_sound(2, 6, 0x66);
}

fn initialize_walking(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = current(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = if actor.parameters[0] & 1 != 0 {
            0xfbe6
        } else {
            0xfb64
        };
        actor.variables[0] = 0x804b;
        actor.pre_instruction = actor.parameters[1].wrapping_add(actor.x.whole);
        actor.variables[4] = actor.x.whole.wrapping_sub(actor.parameters[1]);
    });
}

fn walking_instruction(runtime: &Runtime<'_>, key: Key, attack: bool) -> u16 {
    let x = runtime.resolve_actor(key, Native).x.whole;
    let right = (runtime.session().player.x.wrapping_sub(x) as i16) >= 0;
    match (attack, right) {
        (true, true) => 0xfc0e,
        (true, false) => 0xfb8c,
        (false, true) => 0xfb64,
        (false, false) => 0xfbe6,
    }
}

fn patrol(right: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let index = runtime.word(fields::cur_enemy_index);
    let key = Key::from_legacy(index);
    if api::player_within(&mut runtime, key, Axis::Y, 16) {
        let instruction = walking_instruction(&runtime, key, true);
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction = instruction;
            actor.instruction_delay = 1;
        });
        return;
    }
    let mover = current(&runtime);
    if api::move_vertical(&mut runtime, mover, 1 << 16) != MovementResult::Blocked {
        return;
    }
    let x = runtime.resolve_actor(key, Native).x.whole;
    runtime.write_slice_native(0x7800 + usize::from(index), x);
    runtime.edit_actor(key, Native, |actor| {
        actor.x.whole = offset_x(x, if right { 16 } else { 17 }, right)
    });
    let mover = current(&runtime);
    let floor = api::move_vertical(&mut runtime, mover, 1 << 16) == MovementResult::Blocked;
    let x = runtime.native_word(0x7800 + usize::from(index));
    runtime.edit_actor(key, Native, |actor| actor.x.whole = x);
    let turn = !floor || {
        if !right {
            let mover = current(&runtime);
            api::probe_horizontal(&mut runtime, mover, -9 << 16);
        }
        let mover = current(&runtime);
        api::move_horizontal(
            &mut runtime,
            mover,
            if right { 14336 } else { -14337 },
            api::SlopePolicy::Ignore,
        ) == MovementResult::Blocked
            || {
                let actor = runtime.resolve_actor(key, Native);
                let bound = if right {
                    actor.pre_instruction
                } else {
                    actor.variables[4]
                };
                ((actor.x.whole.wrapping_sub(bound) as i16) >= 0) == right
            }
    };
    if turn {
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction = if right { 0xfc48 } else { 0xfbc6 };
            actor.instruction_delay = 1;
        });
    }
}

fn advance(walking: bool, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = current(&runtime);
    let state = u32::from(runtime.resolve_actor(key, Native).variables[0]) | 0xb20000;
    let index = runtime.word(fields::cur_enemy_index);
    runtime.foreign(|game| dispatch_state(state, index, game));
    if walking && runtime.resolve_actor(key, Native).parameters[0] & 0x8000 != 0 {
        runtime.foreign(|game| dodge_projectile(true, game));
    }
}

pub(crate) fn dispatch_ai(ea: u32, game: &mut Game) {
    match ea {
        11700076 => touch(game),
        11700089 => shot(game),
        11730178 => initialize_walking(game),
        11730226 => advance(true, game),
        11698191 => combat::select_frozen_ai(&mut Runtime::new(game)),
        11698241 => crate::game::actors::scheduling::update_frozen_enemy(&mut Runtime::new(game)),
        11700071 => {
            combat::power_bomb_reaction(&mut Runtime::new(game), api::DeathHandling::Finalize)
        }
        11726893 | 11728546 => advance(false, game),
        11726751 => initialize_wall(game),
        11728350 => initialize_ninja(game),
        _ => Runtime::new(game).unreachable(),
    }
}

pub(crate) fn dispatch_instruction(
    ea: u32,
    k: u16,
    script: EnemyScript,
    game: &mut Game,
) -> EnemyScript {
    match ea {
        11726634 => {
            let mut runtime = Runtime::new(game);
            let index = runtime.word(fields::cur_enemy_index);
            let definition = runtime
                .resolve_actor(Key::from_legacy(index), Native)
                .definition;
            let value = runtime.legacy_native_record(0xa0, definition).word(6);
            runtime.write_word_le(value, fields::cinematic_var8);
            drop(runtime);
            fire_laser(index, false, 16, game);
            script
        }
        11726685 => {
            fire_laser(word!(game.cur_enemy_index), true, 16, game);
            script
        }
        11726723 | 11730104 | 11728182 => {
            let value = script.word(0);
            let mut runtime = Runtime::new(game);
            let key = current(&runtime);
            runtime.edit_actor(key, Native, |actor| {
                if ea == 11728182 {
                    actor.palette = value;
                } else {
                    actor.variables[0] = value;
                }
            });
            script.offset(1)
        }
        11726739 => {
            Runtime::new(game).queue_sound(2, 6, 0x66);
            script
        }
        11728198 => {
            let sound = script.word(0);
            Runtime::new(game).queue_sound(2, 6, sound);
            script.offset(1)
        }
        11728228 => {
            let (x, y) = {
                let runtime = Runtime::new(game);
                let actor = runtime.resolve_actor(current(&runtime), Native);
                (actor.x.whole, actor.y.whole)
            };
            game.eproj_spawn_rect = Rect16U {
                x,
                y,
                w: script.word(2),
                h: script.word(4),
            };
            SpawnEprojWithGfx(script.word(0), word!(game.cur_enemy_index), 0xa189, game);
            script.offset(3)
        }
        11728272 => {
            let mut runtime = Runtime::new(game);
            let key = current(&runtime);
            runtime.edit_actor(key, Native, |actor| actor.instruction_delay = 1);
            let x = runtime.resolve_actor(key, Native).x.whole;
            let instruction = if (x.wrapping_sub(runtime.session().player.x) as i16) < 0 {
                0xf420
            } else {
                0xf22e
            };
            runtime.edit_actor(key, Native, |actor| actor.variables[2] = instruction);
            EnemyScript::jump(instruction)
        }
        11730024 | 11730064 => {
            fire_laser(
                word!(game.cur_enemy_index),
                ea == 11730064,
                script.word(0),
                game,
            );
            script.offset(1)
        }
        11730120 => {
            let mut runtime = Runtime::new(game);
            let key = current(&runtime);
            let attack = api::player_within(&mut runtime, key, Axis::Y, 16);
            EnemyScript::jump(walking_instruction(&runtime, current(&runtime), attack))
        }
        11728342 => {
            reset_ninja_jump(k, game);
            script
        }
        11729257 | 11729469 => {
            prepare_ninja_jump(k, ea == 11729469, game);
            script
        }
        11726400 | 11726450 => wall_direction(ea == 11726400, false, script, game),
        11726500 | 11726524 => wall_direction(ea == 11726500, true, script, game),
        11726548 | 11726589 => prepare_wall_jump(ea == 11726548, script, game),
        _ => {
            Runtime::new(game).unreachable();
            EnemyScript::Stop
        }
    }
}

fn dispatch_state(ea: u32, k: u16, game: &mut Game) {
    match ea {
        11728553 => wake_ninja(game),
        11728612 => select_somersault(k, game),
        11729161 => select_leap(game),
        11728919 => somersault(k, false, false, game),
        11728972 => somersault(k, false, true, game),
        11729040 => somersault(k, true, false, game),
        11729093 => somersault(k, true, true, game),
        11729285 => vertical_jump(k, false, JumpPhase::Rise, game),
        11729345 => vertical_jump(k, false, JumpPhase::Drift, game),
        11729429 => vertical_jump(k, false, JumpPhase::Recover, game),
        11729497 => vertical_jump(k, true, JumpPhase::Rise, game),
        11729557 => vertical_jump(k, true, JumpPhase::Drift, game),
        11729641 => vertical_jump(k, true, JumpPhase::Recover, game),
        11726900 => wait_at_wall(true, game),
        11726928 => jump_between_walls(true, game),
        11727048 => wait_at_wall(false, game),
        11727076 => jump_between_walls(false, game),
        11698251 | 11726927 | 11727075 | 11730506 => (),
        11730244 => patrol(false, game),
        11730382 => patrol(true, game),
        _ => Runtime::new(game).unreachable(),
    }
}
