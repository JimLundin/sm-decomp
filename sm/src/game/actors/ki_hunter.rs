//! KiHunter body, attached wings and detached wing motion.
//! Native actor and extended words remain authoritative during the migration.
#![forbid(unsafe_code)]
use super::access::ActorKey as Key;
use super::api::{Axis, MovementResult::Blocked, SlopePolicy};
use super::{api as actor_api, combat};
use crate::game::enemy_layouts::EnemyLayout40 as Hunter;
use crate::game::music::QueueSfx2_Max6;
use crate::game::session_runtime::{Runtime, actor_cosine, actor_sine};
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rtl::{abs16, words};
use sm_game::enemy_script::EnemyScript;
use {
    crate::game::enemy_projectiles::SpawnEprojWithGfx, sm_game::enemy_projectiles::host::Effects,
};

fn begin_wing(forward: bool, k: u16, game: &mut Game) {
    let E = k as i32;
    slot!(game, Hunter[E]; var_00 = if forward { 0xf7db } else { 0xf8ad },
        var_B = slot!(game, Hunter[E].var_0A), var_F = if forward { 0xe000 } else { 0xa000 },
        var_05 = slot!(game, Hunter[E].base.x_pos), var_06 = slot!(game, Hunter[E].base.y_pos));
}

fn wing_arc(forward: bool, game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    let record = Runtime::new(game).legacy_native_record(0xa2, 0x838f);
    let speed = record
        .word(8 * usize::from(slot!(byte game, Hunter[E].var_B[1])) + if forward { 5 } else { 1 });
    slot!(game, Hunter[E].var_F, |old| old.wrapping_add(speed));
    let y = wing_coordinate(E, false, forward, game);
    if forward {
        slot!(game, Hunter[E].base.y_pos = y);
    } else {
        let amount = (y.wrapping_sub(slot!(game, Hunter[E].base.y_pos)) as i16 as i32) << 16;
        let key = Key::from_legacy(word!(game.cur_enemy_index));
        if actor_api::move_vertical(&mut Runtime::new(game), key, amount)
            == actor_api::MovementResult::Blocked
        {
            slot!(game, Hunter[E].base.properties, |old| old | 0x200);
            slot!(game, Hunter[E].base.x_pos = slot!(game, Hunter[E].var_08));
            let y = slot!(game, Hunter[E].var_07);
            slot!(game, Hunter[E].base.y_pos = y);
            slot!(game, Hunter[word!(game.cur_enemy_index) as i32].var_B = y);
            return;
        }
    }
    slot!(
        game,
        Hunter[E].base.x_pos = wing_coordinate(E, true, forward, game)
    );
    if ((slot!(game, Hunter[E].var_F).wrapping_add(0x4000) & 0x8000) != 0) == forward {
        begin_wing(!forward, word!(game.cur_enemy_index), game);
    } else {
        let speed = (slot!(game, Hunter[E].var_B) as i16).wrapping_sub(384);
        let target = if forward {
            E
        } else {
            word!(game.cur_enemy_index) as i32
        };
        slot!(
            game,
            Hunter[target].var_B = if speed < 0 { 256 } else { speed as u16 }
        );
    }
}

pub(crate) fn initialize(game: &mut Game) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Hunter[E].base.properties, |old| (old)
        | ((0x2000) as u16));
    slot!(game, Hunter[E].var_14 = 0);
    slot!(game, Hunter[E].base; instruction_timer = 1, timer = 0, current_instruction = 0xe9fa as u16);
    slot!(game, Hunter[E]; var_F = 0, var_A = 0xf268, var_08 = 0, var_09 = 1, var_06 = 0, var_07 = -1i32 as u16);
    let v1: u16 = (slot!(game, Hunter[E].base.y_pos)).wrapping_sub(16);
    slot!(game, Hunter[E].var_0A = v1);
    slot!(game, Hunter[E].var_0B = (v1).wrapping_add(32));
    slot!(game, Hunter[E].var_0C = slot!(game, Hunter[E].base.x_pos));
    slot!(game, Hunter[E].var_0D = slot!(game, Hunter[E].base.y_pos));
    if (slot!(game, Hunter[E].parameter_1) & 0x8000) != 0 {
        slot!(game, Hunter[E].var_14 = 1);
        slot!(game, Hunter[E].var_A = 0xf55a);
        slot!(game, Hunter[E].var_08 = 0);
        slot!(game, Hunter[E].var_09 = 1);
    }
}

pub(crate) fn initialize_wings(game: &mut Game) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Hunter[E].base.properties, |old| (old)
        | ((0x2000) as u16));
    slot!(game, Hunter[E].base.instruction_timer = 1);
    slot!(game, Hunter[E].base.timer = 0);
    slot!(game, Hunter[E].base.current_instruction = 0xea4e as u16);
    let v2: i32 = word!(game.cur_enemy_index) as i32 >> 1;
    copy_drawing(E as u16, &[(6, 93), (2, 91)], game);
    slot!(game, Hunter[E].var_A = 0xf6f3);
    copy_drawing(E as u16, &[(0x1e, 105), (0x20, 106)], game);
    if (word!(game.enemy_drawing_queue_sizes[(v2 + 6)]) & 0x8000) != 0 {
        slot!(game, Hunter[E].base.properties, |old| (old)
            | ((0x200) as u16));
    }
}

fn prepare_wing_acceleration(game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    slot!(game, Hunter[E]; var_0A = 0, var_B = 0);
    loop {
        let speed = slot!(game, Hunter[E].var_0A).wrapping_add(384);
        slot!(game, Hunter[E].var_0A = speed);
        let record = Runtime::new(game).legacy_native_record(0xa2, 0x838f);
        let phase = record
            .word(8 * usize::from(speed >> 8) + 1)
            .wrapping_add(slot!(game, Hunter[E].var_D));
        slot!(game, Hunter[E].var_D = phase);
        if phase.wrapping_sub(0x2000) & 0x8000 == 0 {
            break;
        }
    }
}

pub(crate) fn shot(game: &mut Game) {
    combat::shot_reaction(&mut Runtime::new(game), actor_api::ShotHandling::Finalize);
    let old_cur_enemy_index: u16 = word!(game.cur_enemy_index);
    let EK: i32 = word!(game.cur_enemy_index) as i32;
    let E1: i32 = word!(game.cur_enemy_index) as i32 + 64;
    let health: u16 = slot!(game, Hunter[EK].base.health);
    if health != 0 {
        if health == slot!(game, Hunter[E1].parameter_1)
            || ((health) as i16).wrapping_sub((slot!(game, Hunter[E1].parameter_1)) as i16) < 0
        {
            if slot!(game, Hunter[EK].var_14) == 0 {
                slot!(game, Hunter[EK]; var_14 = 1, var_A = 0xf55a, var_08 = 0, var_09 = 1);
                word!(game.cur_enemy_index = (old_cur_enemy_index).wrapping_add(64));
                if slot!(game, Hunter[E1].var_A) as i32 != 0xa8f7cf & 0xffff {
                    slot!(game, Hunter[E1].var_07 = slot!(game, Hunter[E1].base.y_pos));
                    slot!(game, Hunter[E1].var_08 = slot!(game, Hunter[E1].base.x_pos));
                    prepare_wing_acceleration(game);
                    wing_origins(game);
                    slot!(game, Hunter[E1]; var_F = -8192i32 as u16, var_A = 0xf7cf, var_00 = 0xf7db, var_06 = (slot!(game, Hunter[E1].var_07)).wrapping_sub(slot!(game, Hunter[E1].var_0B)), var_05 = slot!(game, Hunter[E1].base.x_pos), var_B = slot!(game, Hunter[E1].var_0A));
                    slot!(game, Hunter[E1].base.current_instruction = 0xea7e as u16);
                    slot!(game, Hunter[E1].base.spritemap_pointer = 0x804d as u16);
                    slot!(game, Hunter[E1].base.instruction_timer = 1);
                    slot!(game, Hunter[E1].base.timer = 0);
                    slot!(game, Hunter[E1].base.properties, |old| (old)
                        | ((0x800) as u16));
                }
                word!(game.cur_enemy_index = old_cur_enemy_index);
            }
        } else {
            for field in [0x12, 0x26, 0x28, 0x24] {
                let source = 0xf78 + EK as usize + field;
                let value = Runtime::new(game).native_word(source);
                Runtime::new(game).write_slice_native(source + 64, value);
            }
        }
    } else {
        slot!(game, Hunter[E1].base.properties = 512);
    };
}

fn spit(j: u16, game: &mut Game) {
    QueueSfx2_Max6(0x4c, game);
    SpawnEprojWithGfx(0, word!(game.cur_enemy_index), j, game);
    slot!(game, Hunter[word!(game.cur_enemy_index) as i32].var_0F = 24);
}

fn wing_origins(game: &mut Game) {
    for (phase, x_field, y_field) in [(0xe0, 3, 4), (0xa0, 1, 2)] {
        let x = actor_cosine(phase, 0x30, game);
        let at = 0x7800 + usize::from(word!(game.cur_enemy_index));
        Runtime::new(game).write_slice_native(at + x_field * 2, x);
        let y = actor_sine(phase, 0x30, game);
        Runtime::new(game).write_slice_native(at + y_field * 2, y);
    }
}

fn paired_animation(k: u16, index: u16, game: &mut Game) {
    for (offset, entry) in [(0, index), (64, index + 1)] {
        let value = crate::rtl::rom_table_word!(game, 0xa8f3b0, entry as isize);
        Runtime::new(game).write_slice_native(0xf92 + usize::from(k) + offset, value);
    }
    for offset in [0, 64] {
        Runtime::new(game).write_slice_native(0xf94 + usize::from(k) + offset, 1);
    }
}
pub(crate) fn advance(game: &mut Game) {
    let k = word!(game.cur_enemy_index);
    let E = k as i32;
    let state = slot!(game, Hunter[E].var_A);
    match state {
        0xf268 => {
            let vertical = if move_velocity(k, Axis::Y, game) {
                Some(slot!(game, Hunter[E].var_09).wrapping_neg())
            } else {
                let y = slot!(game, Hunter[E].base.y_pos);
                if (y as i16).wrapping_sub(slot!(game, Hunter[E].var_0A) as i16) < 0 {
                    Some(1)
                } else if (y as i16).wrapping_sub(slot!(game, Hunter[E].var_0B) as i16) >= 0 {
                    Some(u16::MAX)
                } else {
                    None
                }
            };
            if let Some(speed) = vertical {
                slot!(game, Hunter[E].var_09 = speed);
            }
            if move_velocity(k, Axis::X, game) {
                let speed = slot!(game, Hunter[E].var_07).wrapping_neg();
                slot!(game, Hunter[E].var_07 = speed);
                paired_animation(k, if speed & 0x8000 != 0 { 0 } else { 2 }, game);
            }
            snap(k, game);
            let r18: u16 =
                (word!(game.samus_x_pos)).wrapping_sub(slot!(game, Hunter[E].base.x_pos));
            let r20: u16 = abs16(r18);
            if ((r20) as i16).wrapping_sub((0x60) as i16) < 0
                && word!(game.samus_y_pos) as i32 - slot!(game, Hunter[E].base.y_pos) as i32 - 32i32
                    & 0x8000
                    == 0
            {
                let r24: u16 =
                    (word!(game.samus_y_pos)).wrapping_sub(slot!(game, Hunter[E].base.y_pos));
                let mut v9: u16 = if (r18 & 0x8000) != 0 {
                    slot!(game, Hunter[E].var_00 = -2i32 as u16);
                    slot!(game, Hunter[E]; var_02 = 0, var_03 = 0);
                    slot!(game, Hunter[E].var_04 = -1i32 as u16);
                    slot!(game, Hunter[E].var_07 = -1i32 as u16);
                    slot!(game, Hunter[E].var_05 = -8192i32 as u16);
                    slot!(game, Hunter[E].var_F = 255);
                    slot!(game, Hunter[E].var_0E = 240);
                    0
                } else {
                    slot!(game, Hunter[E].var_00 = 2);
                    slot!(game, Hunter[E]; var_02 = 0, var_03 = 0, var_04 = 0);
                    slot!(game, Hunter[E].var_05 = 0x2000);
                    slot!(game, Hunter[E].var_F = 128);
                    slot!(game, Hunter[E].var_07 = 1);
                    slot!(game, Hunter[E].var_0E = 144);
                    4
                };
                paired_animation(k, v9 >> 1, game);
                slot!(game, Hunter[E]; var_B = (r18).wrapping_add(slot!(game, Hunter[E].base.x_pos)), var_C = slot!(game, Hunter[E].base.y_pos), var_A = 0xf3b8, var_12 = r24, var_11 = r20, var_10 = 0);
            }
        }
        0xf3b8 => {
            let moving_forward = (slot!(game, Hunter[E].var_04) & 0x8000) == 0;
            let past_turn = ((slot!(game, Hunter[E].var_F)) as i16)
                .wrapping_sub((slot!(game, Hunter[E].var_0E)) as i16)
                >= 0;
            if moving_forward == past_turn && slot!(game, Hunter[E].var_10) == 0 {
                slot!(game, Hunter[E].var_10 = 1);
                let mut v2: u16 = 0xea32 as u16;
                if (slot!(game, Hunter[E].var_04) & 0x8000) != 0 {
                    v2 = 0xea08 as u16;
                }
                slot!(game, Hunter[E].base; current_instruction = v2, instruction_timer = 1);
            }
            let moving_forward = (slot!(game, Hunter[E].var_04) & 0x8000) == 0;
            slot!(
                game,
                Hunter[E](var_02, var_03) += crate::rtl::words!((
                    slot!(game, Hunter[E].var_04),
                    slot!(game, Hunter[E].var_05)
                ))
            );
            let delta = slot!(game, Hunter[E].var_02).wrapping_sub(slot!(game, Hunter[E].var_00));
            let reached_speed_limit = moving_forward == (delta & 0x8000 == 0);
            if reached_speed_limit {
                slot!(game, Hunter[E].var_02 = slot!(game, Hunter[E].var_00));
            }
            slot!(
                game,
                Hunter[E].var_F =
                    (slot!(game, Hunter[E].var_F)).wrapping_add(slot!(game, Hunter[E].var_02))
            );
            let finished_arc = if moving_forward {
                slot!(game, Hunter[E].var_F) as i32 - 256i32 & 0x8000 == 0
            } else {
                slot!(game, Hunter[E].var_F) as i32 - 128i32 & 0x8000 != 0
            };
            if finished_arc {
                slot!(game, Hunter[E].var_A = 0xf268);
                return;
            }
            let mut r20: u16 = (slot!(game, Hunter[E].var_B) as i32
                + actor_cosine(
                    slot!(game, Hunter[E].var_F),
                    slot!(game, Hunter[E].var_11),
                    game,
                ) as i32
                - slot!(game, Hunter[E].base.x_pos) as i32) as u16;
            let collided = if move_by(k, Axis::X, (r20 as i16 as i32) << 16, game) {
                let forward = slot!(game, Hunter[E].var_04) & 0x8000 == 0;
                slot!(game, Hunter[E].var_06 = 0);
                slot!(game, Hunter[E].var_07 = if forward { u16::MAX } else { 1 });
                true
            } else {
                snap(k, game);
                r20 = (slot!(game, Hunter[E].var_C) as i32
                    + actor_sine(
                        slot!(game, Hunter[E].var_F),
                        slot!(game, Hunter[E].var_12),
                        game,
                    ) as i32
                    - slot!(game, Hunter[E].base.y_pos) as i32) as u16;
                move_by(k, Axis::Y, (r20 as i16 as i32) << 16, game)
            };
            if collided {
                slot!(game, Hunter[E].var_A = 0xf4ed);
                slot!(game, Hunter[E].var_08 = 0);
                slot!(game, Hunter[E].var_09 = -1i32 as u16);
            };
        }
        0xf4ed => {
            if move_velocity(k, Axis::X, game)
                || {
                    snap(k, game);
                    move_velocity(k, Axis::Y, game)
                }
                || ((slot!(game, Hunter[E].base.y_pos)) as i16)
                    .wrapping_sub((slot!(game, Hunter[E].var_0D)) as i16)
                    < 0
            {
                slot!(game, Hunter[E].var_A = 0xf268);
            }
        }
        0xf55a => {
            if move_velocity(k, Axis::Y, game) {
                slot!(game, Hunter[E].var_A = 0xf58b);
            } else {
                slot!(game, Hunter[E](var_09, var_08) += words!((0x0, 0xe000)));
            };
        }
        0xf58b => {
            slot!(game, Hunter[E].var_A = 0xf5e3 as u16);
            slot!(game, Hunter[E].var_08 = 0);
            slot!(
                game,
                Hunter[E].var_09 = ((word!(game.random_number) as i32 & 1) - 8i32) as u16
            );
            let left = (slot!(game, Hunter[E].base.x_pos) as i16)
                .wrapping_sub(word!(game.samus_x_pos) as i16)
                >= 0;
            slot!(game, Hunter[E].var_06 = 0);
            slot!(
                game,
                Hunter[E].var_07 = if left { (-2i16) as u16 } else { 2 }
            );
            slot!(
                game,
                Hunter[E].base.current_instruction = if left { 0xea8a } else { 0xeaa6 }
            );
            slot!(game, Hunter[E].base.instruction_timer = 1);
        }
        0xf5f0 => {
            if move_velocity(k, Axis::Y, game) {
                if (slot!(game, Hunter[E].var_09) & 0x8000) != 0 {
                    slot!(game, Hunter[E].var_09 = 1);
                } else {
                    slot!(game, Hunter[E].var_08 = 0);
                    slot!(game, Hunter[E].var_09 = -4i32 as u16);
                    slot!(game, Hunter[E].var_A = 0xf5e3 as u16);
                    slot!(game, Hunter[E].var_0F = 12);
                    let mut v4: u16 = 0xeac2 as u16;
                    if slot!(game, Hunter[E].base.current_instruction) as i32 + 0x155a & 0x8000 == 0
                    {
                        v4 = 0xeada as u16;
                    }
                    slot!(game, Hunter[E].base; current_instruction = v4, instruction_timer = 1);
                }
            } else if move_velocity(k, Axis::X, game) {
                slot!(
                    game,
                    Hunter[E].var_07 = -(slot!(game, Hunter[E].var_07) as i32) as u16
                );
            } else {
                snap(k, game);
                slot!(game, Hunter[E](var_09, var_08) += words!((0x0, 0xe000)));
            };
        }
        0xf68b => {
            let v2: u16 = (slot!(game, Hunter[E].var_0F)).wrapping_sub(1);
            slot!(game, Hunter[E].var_0F = v2);
            if v2 == 0 {
                let mut v3: u16 = 0xf58b;
                let v4: u16 = abs16(
                    (slot!(game, Hunter[E].base.x_pos)).wrapping_sub(word!(game.samus_x_pos)),
                );
                if v4 as i32 - 96i32 & 0x8000 != 0 {
                    v3 = 0xf6b3;
                }
                slot!(game, Hunter[E].var_A = v3);
            }
        }
        0xf6b3 => {
            let mut v1: u16 = 0xeaf2 as u16;
            if ((slot!(game, Hunter[E].base.x_pos)) as i16)
                .wrapping_sub((word!(game.samus_x_pos)) as i16)
                < 0
            {
                v1 = 0xeb10 as u16;
            }
            slot!(game, Hunter[E].base; current_instruction = v1, instruction_timer = 1);
            slot!(game, Hunter[E].var_A = 0xf5e3 as u16);
        }
        0xf6f3 => copy_drawing(k, &[(2, 91), (6, 93)], game),
        0xf7cf => {
            let wing_state = slot!(game, Hunter[word!(game.cur_enemy_index) as i32].var_00);
            match wing_state {
                0xf7db => wing_arc(true, game),
                0xf8ad => wing_arc(false, game),
                _ => Runtime::new(game).unreachable(),
            }
        }
        0xf5e3 => (),
        _ => Runtime::new(game).unreachable(),
    }
}
pub(crate) fn instruction(
    address: u32,
    k: u16,
    script: EnemyScript,
    game: &mut Game,
) -> EnemyScript {
    match address {
        11072806 => {
            let E = k as i32;
            let left = slot!(game, Hunter[E].var_07) & 0x8000 != 0;
            let (next, attached) = if left {
                (0xe9fa, 0xea4e)
            } else {
                (0xea24, 0xea5e)
            };
            slot!(game, Hunter[E].base; current_instruction = attached, instruction_timer = 1);
            let peer = E + 64;
            if slot!(game, Hunter[peer].var_A) == 0xf6f3 {
                slot!(game, Hunter[peer].base; current_instruction = attached, instruction_timer = 1);
            }
            return EnemyScript::jump(next);
        }
        11072996 | 11073151 => {
            slot!(
                game,
                Hunter[word!(game.cur_enemy_index) as i32].var_A =
                    if address == 11072996 { 0xf5f0 } else { 0xf68b }
            );
        }
        11073234 | 11073240 => spit(if address == 11073234 { 0xcf18 } else { 0xcf26 }, game),
        _ => {
            Runtime::new(game).unreachable();
            return EnemyScript::Stop;
        }
    }
    script
}

fn move_by(k: u16, axis: Axis, amount: i32, game: &mut Game) -> bool {
    let mut runtime = Runtime::new(game);
    let key = Key::from_legacy(k);
    let result = match axis {
        Axis::X => actor_api::move_horizontal(&mut runtime, key, amount, SlopePolicy::Ignore),
        Axis::Y => actor_api::move_vertical(&mut runtime, key, amount),
    };
    result == Blocked
}
fn move_velocity(k: u16, axis: Axis, game: &mut Game) -> bool {
    let E = k as i32;
    let amount = match axis {
        Axis::X => words!((slot!(game, Hunter[E].var_07), slot!(game, Hunter[E].var_06))),
        Axis::Y => words!((slot!(game, Hunter[E].var_09), slot!(game, Hunter[E].var_08))),
    };
    move_by(k, axis, amount as i32, game)
}
fn snap(k: u16, game: &mut Game) {
    actor_api::snap_to_slopes(&mut Runtime::new(game), Key::from_legacy(k));
}

fn copy_drawing(k: u16, fields: &[(usize, i32)], game: &mut Game) {
    for &(field, queue) in fields {
        let value = word!(game.enemy_drawing_queue[(i32::from(k >> 1) + queue)]);
        Runtime::new(game).write_slice_native(0xf78 + usize::from(k) + field, value);
    }
}

fn wing_coordinate(E: i32, x: bool, forward: bool, game: &mut Game) -> u16 {
    let at = 0x7800 + E as usize;
    let center = Runtime::new(game).native_word(at + if x { 10 } else { 12 });
    let phase = slot!(byte game, Hunter[E].var_F[1]) as u16;
    let wave = if x { actor_cosine } else { actor_sine };
    let displacement = wave(phase, 0x30, game);
    let field = if forward { 6 } else { 2 };
    let correction = Runtime::new(game).native_word(at + field + usize::from(!x) * 2);
    center.wrapping_add(displacement).wrapping_sub(correction)
}
