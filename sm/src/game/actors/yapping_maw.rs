//! Yapping Maw reach, articulated body, player grip and attachment lifecycle.
//! RAM words remain authoritative; operand writes run immediately through the caller.
#![forbid(unsafe_code)]
use super::{api, combat, scheduling};
use crate::game::enemy_layouts::EnemyLayout104 as Maw;
use crate::game::enemy_projectiles::{self, SpawnEprojWithGfx};
use crate::game::enemy_script::EnemyScript;
use crate::game::music::QueueSfx2_Max6;
use crate::game::session::Point;
use crate::game::session_runtime::{Runtime, correct_previous_position, player_command};
use crate::game::sprite_effect_runtime as sprites;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rtl::{abs16, rom_table_word, words};

pub(crate) fn initialize(game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    let sprite_owner = word!(game.cur_enemy_index) as i32;
    slot!(game, Maw[E].var_0C = slot!(game, Maw[E].base.x_pos));
    slot!(game, Maw[E].var_0D = slot!(game, Maw[E].base.y_pos));
    for field in 0..8 {
        slot!(game, Maw[E + 2 * field].var_00 = 0);
    }
    slot!(game, Maw[E]; var_30 = 0, var_F = slot!(game, Maw[E].parameter_1), var_E = 64);
    slot!(game, Maw[E].base.current_instruction = 0x9f6f);
    let (mut sprite_kind, mut displacement) = (57, 8u16);
    if slot!(game, Maw[E].parameter_2) == 0 {
        slot!(game, Maw[E].base.current_instruction = 0x9fc7);
        sprite_kind = 56;
        displacement = (-8i16) as u16;
    }
    slot!(game, Maw[E].base; instruction_timer = 1, timer = 0);
    slot!(game, Maw[E]; var_A = 0xa235, var_45 = slot!(game, Maw[E].base.palette_index) & 0xe00);
    let mut part = 3i16;
    slot!(game, Maw[E].var_44 = 3);
    loop {
        SpawnEprojWithGfx(part as u16, word!(game.cur_enemy_index), 0xec95, game);
        part = (slot!(game, Maw[E].var_44) as i16).wrapping_sub(1);
        slot!(game, Maw[E].var_44 = part as u16);
        if part < 0 {
            break;
        }
    }
    let appearance =
        slot!(game, Maw[E].base.vram_tiles_index) | slot!(game, Maw[E].base.palette_index);
    slot!(game, Maw[E].var_47 = appearance);
    let sprite = sprites::create_sprite(
        slot!(game, Maw[E].base.x_pos),
        displacement.wrapping_add(slot!(game, Maw[E].base.y_pos)),
        sprite_kind,
        appearance,
        game,
    );
    slot!(game, Maw[sprite_owner].var_46 = sprite);
}
fn detect_player(game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    let origin = Point {
        x: slot!(game, Maw[E].base.x_pos),
        y: slot!(game, Maw[E].base.y_pos),
    };
    let target = Point {
        x: word!(game.samus_x_pos),
        y: word!(game.samus_y_pos),
    };
    let pair = api::distance_and_direction(&mut Runtime::new(game), origin, target);
    let distance = abs16(pair.distance);
    slot!(game, Maw[E].var_08 = distance);
    if distance.wrapping_sub(32) & 0x8000 != 0 {
        slot!(game, Maw[E].var_35 = 48);
    } else if (distance as i16).wrapping_sub(slot!(game, Maw[E].var_F) as i16) < 0 {
        if slot!(game, Maw[E].var_08).wrapping_sub(64) & 0x8000 == 0 {
            slot!(game, Maw[E].var_08 = 64);
        }
        slot!(game, Maw[E]; var_0A = pair.direction, var_A = 0xa28c);
    }
}
fn project(
    angle: u16,
    distance: u16,
    write_operand: &mut impl FnMut(u8, &mut Game),
    game: &mut Game,
) -> u16 {
    let sine = rom_table_word!(game, 0xa0b1c3, -(angle as i32) as u8 as isize) as i16;
    let magnitude = if sine < 0 {
        -(sine as i32) as i16
    } else {
        sine
    };
    // The operand is physical byte one of the native signed word.
    let operand = magnitude.to_ne_bytes()[1];
    write_operand(operand, game);
    if distance as u8 == 0 {
        return 0;
    }
    let product = u16::from(operand) * u16::from(distance as u8);
    if sine < 0 {
        (2 * ((-(product as i32) as u16 as i32) >> 8) | 0xff00) as u16
    } else {
        2 * (product >> 8)
    }
}
fn start_reach(write_operand: &mut impl FnMut(u8, &mut Game), game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    slot!(game, Maw[E]; var_B = 0, var_C = 0, var_D = 0);
    slot!(game, Maw[E].var_09 = slot!(game, Maw[E].var_08) >> 1);
    let direction = 64u8.wrapping_sub(slot!(game, Maw[E].var_0A) as u8) as u16;
    slot!(game, Maw[E]; var_0B = direction, var_2F = u16::from(direction >= 128));
    let origin = word!(game.cur_enemy_index) as i32;
    let x = project(64, slot!(game, Maw[origin].var_09), write_operand, game);
    slot!(game, Maw[origin].var_0E = x);
    let y = project(
        128,
        slot!(game, Maw[origin].var_09) >> 1,
        write_operand,
        game,
    );
    slot!(game, Maw[origin].var_0F = y);
    let direction = 2 * ((slot!(game, Maw[E].var_0A) as u8).wrapping_add(16) as u16 >> 5);
    slot!(game, Maw[E].var_34 = direction);
    slot!(game, Maw[E].base; current_instruction = rom_table_word!(game, 0xa8a097, (direction >> 1) as isize), instruction_timer = 1, timer = 0);
    // These offsets use the native binding; animation above uses the loaded image.
    let x = game.capture_native_rom().table_word(
        crate::rom::lorom_offset(0xa8a0a7) as usize,
        direction as isize,
    );
    slot!(game, Maw[E].var_32 = x);
    let y = game.capture_native_rom().table_word(
        crate::rom::lorom_offset(0xa8a0a9) as usize,
        direction as isize,
    );
    slot!(game, Maw[E].var_33 = y);
    slot!(game, Maw[E].var_A = 0xa445);
}
fn reorient_segment(part: i32, write_operand: &mut impl FnMut(u8, &mut Game), game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    let distance = slot!(game, Maw[E + 4 * part].var_00);
    let x = project(0u16.wrapping_sub(64), distance, write_operand, game);
    slot!(game, Maw[E].var_10 = x);
    let y = project(0, distance, write_operand, game);
    slot!(game, Maw[E].var_11 = y);
    let angle = slot!(game, Maw[E].var_0B).wrapping_sub(64);
    let x = project(angle, distance, write_operand, game);
    slot!(
        game,
        Maw[E + 4 * part].var_27 = x.wrapping_sub(slot!(game, Maw[E].var_10))
    );
    let angle = slot!(game, Maw[E].var_0B);
    let y = project(angle, distance, write_operand, game);
    slot!(
        game,
        Maw[E + 4 * part].var_28 = y.wrapping_sub(slot!(game, Maw[E].var_11))
    );
}
fn carry_player(game: &mut Game) {
    player_command(3, game);
    let E = word!(game.cur_enemy_index) as i32;
    word!(
        game.samus_x_pos = slot!(game, Maw[E].var_32).wrapping_add(slot!(game, Maw[E].base.x_pos))
    );
    word!(
        game.samus_y_pos = slot!(game, Maw[E].var_33).wrapping_add(slot!(game, Maw[E].base.y_pos))
    );
    correct_previous_position(game);
}
fn stretch(write_operand: &mut impl FnMut(u8, &mut Game), game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    let step = slot!(game, Maw[E].var_B) >> 2;
    let positive = slot!(game, Maw[E].var_2F) != 0;
    for part in 0..4 {
        let angle = if positive {
            128u16.wrapping_add(step.wrapping_mul(part as u16 + 1))
        } else {
            128u16.wrapping_sub(step.wrapping_mul(part as u16 + 1))
        };
        slot!(game, Maw[E + 2 * part].var_21 = angle);
    }
    let radius = slot!(game, Maw[E].var_09);
    for part in 0..4 {
        let angle = slot!(game, Maw[E + 2 * part].var_21).wrapping_sub(64);
        let x = project(angle, radius, write_operand, game);
        slot!(
            game,
            Maw[E + 4 * part].var_00 = x.wrapping_sub(slot!(game, Maw[E].var_0E))
        );
    }
    let radius = slot!(game, Maw[E].var_09) >> 1;
    for part in 0..4 {
        let angle = slot!(game, Maw[E + 2 * part].var_21);
        let y = project(angle, radius, write_operand, game);
        slot!(
            game,
            Maw[E + 4 * part].var_01 = y.wrapping_sub(slot!(game, Maw[E].var_0F))
        );
    }
    for part in 0..4 {
        reorient_segment(part, write_operand, game);
    }
    for field in 0..8 {
        slot!(game, Maw[E + 2 * field].var_00, |old| old
            .wrapping_add(slot!(game, Maw[E + 2 * field].var_27)));
    }
    slot!(
        game,
        Maw[E].base.x_pos = slot!(game, Maw[E].var_06).wrapping_add(slot!(game, Maw[E].var_0C))
    );
    slot!(
        game,
        Maw[E].base.y_pos = slot!(game, Maw[E].var_07).wrapping_add(slot!(game, Maw[E].var_0D))
    );
    let velocity_owner = word!(game.cur_enemy_index) as i32;
    let speed = slot!(game, Maw[velocity_owner].var_D) as isize >> 1;
    slot!(
        game,
        Maw[velocity_owner](var_B, var_C) += words!((
            rom_table_word!(game, 0xa2838f, speed + 1),
            rom_table_word!(game, 0xa2838f, speed)
        ))
    );
    slot!(game, Maw[velocity_owner].var_D, |old| old.wrapping_add(8));
    if slot!(game, Maw[E].var_B) & 0x8000 == 0 {
        if slot!(game, Maw[E].var_B).wrapping_sub(128) & 0x8000 == 0 {
            slot!(game, Maw[E]; var_B = 128, var_C = 0);
            slot!(game, Maw[E].var_D, |old| old.wrapping_add(4));
        }
    } else {
        slot!(game, Maw[E]; var_A = 0xa68a, var_35 = 48);
        let direction = slot!(game, Maw[E].var_34);
        slot!(
            game,
            Maw[E].base.current_instruction = if direction == 4 {
                0xa01f
            } else if direction == 12 {
                0xa03d
            } else {
                0xa025
            }
        );
        if slot!(game, Maw[E].parameter_2) != 0 {
            return;
        }
        if slot!(game, Maw[E].var_34) == 4 {
            slot!(game, Maw[E].base.current_instruction = 0xa05b);
        } else if slot!(game, Maw[E].var_34) == 12 {
            slot!(game, Maw[E].base.current_instruction = 0xa079);
        } else {
            slot!(game, Maw[E].base.current_instruction = 0xa061);
            return;
        }
    }
    if slot!(game, Maw[E].var_30) != 0 {
        carry_player(game);
    }
}
fn release_player(game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    if slot!(game, Maw[E].var_30) != 0 {
        carry_player(game);
    }
    slot!(game, Maw[E].var_E, |old| old.wrapping_sub(1));
    if slot!(game, Maw[E].var_E) & 0x8000 != 0 && word!(game.samus_input_handler) != 0xe91d {
        word!(game.samus_input_handler = 0xe913);
        slot!(game, Maw[E]; var_30 = 0, var_35 = 48, var_E = 64, var_A = 0xa235);
    }
}
/// False selects the existing runtime's unknown-address failure before attachments update.
pub(crate) fn advance(game: &mut Game, mut write_operand: impl FnMut(u8, &mut Game)) -> bool {
    let E = word!(game.cur_enemy_index) as i32;
    slot!(game, Maw[E].var_35, |old| old.wrapping_sub(1));
    Runtime::new(game).update_maw_visibility(E);
    match 0xa80000 | u32::from(slot!(game, Maw[E].var_A)) {
        0xa8a235 => detect_player(game),
        0xa8a28c => start_reach(&mut write_operand, game),
        0xa8a445 => stretch(&mut write_operand, game),
        0xa8a68a => release_player(game),
        _ => return false,
    }
    true
}
pub(crate) fn update_segments(game: &mut Game) {
    for part in (0..4).rev() {
        enemy_projectiles::update_maw_segment(part, game);
    }
}
pub(crate) fn touch(game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    if slot!(game, Maw[E].var_35) & 0x8000 != 0 && slot!(game, Maw[E].var_30) == 0 {
        slot!(game, Maw[E]; var_35 = 0, var_30 = 1);
        word!(game.samus_input_handler = 0xe90e);
    }
}
pub(crate) fn appearance(game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    let enemy = word!(game.cur_enemy_index);
    let appearance = slot!(game, Maw[E].var_45);
    let handle = slot!(game, Maw[E].var_46);
    sprites::maw_appearance(handle, appearance, enemy, game);
}
pub(crate) fn shot(game: &mut Game) {
    combat::shot_reaction(&mut Runtime::new(game), api::ShotHandling::Finalize);
    let E = word!(game.cur_enemy_index) as i32;
    if slot!(game, Maw[E].base.health) != 0 {
        if slot!(game, Maw[E].base.frozen_timer) != 0 {
            if word!(game.samus_input_handler) != 0xe91d {
                word!(game.samus_input_handler = 0xe913);
            }
            slot!(game, Maw[E].var_30 = 0);
        }
    } else {
        for part in 0..4 {
            let at = 0x1997 + usize::from(slot!(game, Maw[E + 2 * part].var_40));
            game.g_ram[at..at + 2].copy_from_slice(&0u16.to_ne_bytes());
        }
        let handle = slot!(game, Maw[E].var_46);
        sprites::retire(handle, game);
        if word!(game.samus_input_handler) != 0xe91d {
            word!(game.samus_input_handler = 0xe913);
        }
        slot!(game, Maw[word!(game.cur_enemy_index) as i32].var_30 = 0);
    }
}
pub(crate) fn frozen(game: &mut Game) {
    scheduling::update_frozen_enemy(&mut Runtime::new(game));
    let E = word!(game.cur_enemy_index) as i32;
    for part in 0..4 {
        enemy_projectiles::update_maw_palette(slot!(game, Maw[E + 2 * part].var_40), game);
    }
    appearance(game);
}
pub(crate) fn instruction(address: u32, script: EnemyScript, game: &mut Game) -> EnemyScript {
    if address == 11051315 {
        if slot!(game, Maw[word!(game.cur_enemy_index) as i32].var_36) == 0 {
            QueueSfx2_Max6(0x2f, game);
        }
    } else {
        let E = word!(game.cur_enemy_index) as i32;
        let table = match address {
            11051207 => 0xa8a0ab,
            11051225 => 0xa8a0c3,
            11051243 => 0xa8a0b3,
            11051261 => 0xa8a0bb,
            11051279 => 0xa8a0a7,
            11051297 => 0xa8a0b7,
            _ => unreachable!("Yapping Maw instruction {address:06x}"),
        };
        slot!(game, Maw[E].var_32 = rom_table_word!(game, table, 0));
        slot!(game, Maw[E].var_33 = rom_table_word!(game, table + 2, 0));
    }
    script
}
