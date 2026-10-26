//! Botwoon motion, path programs, attack animation and combat over native fields.
//! Numeric dispatch and unsafe room creation remain explicit bank boundaries.
#![forbid(unsafe_code)]
use super::{access::ActorKey as Key, api, combat};
use crate::game::constants::{bank_86::*, enemy_properties::*};
use crate::game::enemy_layouts::EnemyLayout104 as Boss;
use crate::game::enemy_projectiles::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::gameplay_flag_runtime::SetBossBitForCurArea;
use crate::game::music::{QueueMusic_Delayed8, QueueSfx2_Max6};
use crate::game::session_runtime::{Runtime, actor_direction_vector, native_rom_diagnostic};
use crate::game::sprite_effect_runtime::create_sprite;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rom::{RomRecord, lorom_offset};
use crate::rtl::rom_table_word;
use crate::types::Point16U;

pub(crate) fn initialize(e: i32, game: &mut Game) {
    slot!(game, Boss[e].var_A = 24);
    loop {
        SpawnEprojWithGfx(
            24,
            word!(game.cur_enemy_index),
            addr_kEproj_BotwoonsBody as u16,
            game,
        );
        slot!(game, Boss[e].var_A, |old| old.wrapping_sub(1));
        let finished = (slot!(game, Boss[e].var_A) as i16).wrapping_sub(1) < 0;
        slot!(game, Boss[e].var_A, |old| old.wrapping_sub(1));
        if finished {
            break;
        }
    }
    slot!(game, Boss[e].base.current_instruction = 0x9389);
    slot!(game, Boss[e]; var_3B = 0x9389, var_D = 0x9878, var_E = 0x9bb7, var_F = 0x9dc0, var_20 = 256,
        var_38 = rom_table_word!(game, 0xb394bb, 0), var_C = rom_table_word!(game, 0xb394bb, 1),
        var_33 = 1, var_34 = 1, var_59 = 1, var_36 = 0xffff, var_37 = 0);
    slot!(game, Boss[e].base.properties, |old| old
        | kEnemyProps_Tangible as u16);
    let health = slot!(game, Boss[e].base.health);
    slot!(game, Boss[e]; var_5B = health, var_5C = health >> 1, var_5D = (health >> 1) >> 1);
    let x = slot!(game, Boss[e].base.x_pos);
    slot!(game, Boss[e]; var_50 = x, var_52 = x, var_54 = x, var_56 = x);
    let y = slot!(game, Boss[e].base.y_pos);
    slot!(game, Boss[e]; var_51 = y, var_53 = y, var_55 = y, var_57 = y,
        var_4E = ((i32::from(slot!(game, Boss[e].base.palette_index)) >> 4) + 256) as u16);
}
pub(crate) fn arm_death(game: &mut Game) {
    let e = i32::from(word!(game.cur_enemy_index));
    if slot!(game, Boss[e].var_2F) != 0 && slot!(game, Boss[e].var_3E) != 0 {
        slot!(game, Boss[e]; var_30 = 1, var_D = 0x9a46, var_22 = 240, var_2F = 0, var_3E = 0);
    }
}
pub(crate) fn health_palette(game: &mut Game) {
    let e = i32::from(word!(game.cur_enemy_index));
    if slot!(game, Boss[e].var_4F) != 16
        && (slot!(game, Boss[e].base.health) as i16).wrapping_sub(rom_table_word!(
            game,
            0xb3981b,
            (slot!(game, Boss[e].var_4F) >> 1) as isize
        ) as i16)
            < 0
    {
        let mut source = 16u16.wrapping_mul(slot!(game, Boss[e].var_4F));
        let mut target = slot!(game, Boss[e].var_4E);
        loop {
            word!(
                game.palette_buffer[target >> 1] =
                    rom_table_word!(game, 0xb3971b, (source >> 1) as isize)
            );
            source = source.wrapping_add(2);
            target = target.wrapping_add(2);
            if target == 512 {
                break;
            }
        }
        slot!(game, Boss[e].var_4F, |old| old.wrapping_add(2));
    }
}
pub(crate) fn wait_entry(game: &mut Game) {
    let e = i32::from(word!(game.cur_enemy_index));
    let delay = slot!(game, Boss[e].var_20).wrapping_sub(1);
    slot!(game, Boss[e].var_20 = delay);
    if delay == 0 {
        slot!(game, Boss[e].var_D = 0x989d);
    }
}
pub(crate) fn select_attack(e: i32, game: &mut Game) -> bool {
    if slot!(game, Boss[e].var_3C) == 0 {
        return false;
    }
    slot!(game, Boss[e].var_3C = 0);
    let choice = if slot!(game, Boss[e].var_33) == 0
        && slot!(game, Boss[e].var_59) == 0
        && slot!(game, Boss[e].var_3F) == 0
    {
        game.next_random() as u8 & 0xe
    } else {
        0
    };
    slot!(game, Boss[e].var_59 = 0);
    // Masking above admits exactly 0,2,4,6,8,10,12,14.
    let current = i32::from(word!(game.cur_enemy_index));
    if choice <= 4 {
        slot!(game, Boss[current]; var_D = 0x99a4, var_E = 0xe250, var_21 = 0, var_F = 0x9dc0, var_4B = 0);
        select_path(word!(game.cur_enemy_index), game);
    } else {
        slot!(game, Boss[current]; var_D = 0x99e4, var_F = 0x9e7d, var_21 = 48);
        slot!(game, Boss[current].base.properties, |old| old
            & !(kEnemyProps_Tangible as u16));
    }
    true
}
pub(crate) fn resume_chase(e: i32, game: &mut Game) -> bool {
    if slot!(game, Boss[e].var_3C) == 0 {
        return false;
    }
    slot!(game, Boss[e]; var_3C = 0, var_D = 0x989d, var_E = 0x9bb7);
    if slot!(game, Boss[e].var_33) != 0 {
        slot!(game, Boss[e].var_35 = 0);
    }
    true
}
pub(crate) fn await_spit(e: i32, game: &mut Game) -> bool {
    if slot!(game, Boss[e].var_21) != 0 {
        slot!(game, Boss[e].var_2E = 0);
        return true;
    }
    slot!(game, Boss[e]; var_3C = 0, var_D = 0x99a4, var_E = 0xe250, var_F = 0x9dc0);
    if slot!(game, Boss[e].var_2F) != 0 || {
        let choice = game.next_random() & 1;
        slot!(game, Boss[e]; var_33 = choice, var_34 = choice);
        slot!(game, Boss[e].var_34) == 0
    } {
        slot!(game, Boss[e].var_4B = 0);
    } else {
        slot!(game, Boss[e]; var_35 = 0, var_36 = 0xffff);
    }
    select_path(word!(game.cur_enemy_index), game);
    false
}
pub(crate) fn wait_collapse(game: &mut Game) {
    let e = i32::from(word!(game.cur_enemy_index));
    let delay = slot!(game, Boss[e].var_22).wrapping_add(1);
    slot!(game, Boss[e].var_22 = delay);
    if (i32::from(delay) - 256) & 0x8000 == 0 {
        slot!(game, Boss[e].var_D = 0x9a5e);
    }
}
pub(crate) fn fall(game: &mut Game) {
    let e = i32::from(word!(game.cur_enemy_index));
    let index = (8 * i32::from(slot!(byte game, Boss[e].var_28[1])) >> 1) as isize;
    let high = rom_table_word!(game, 0xa2838f, index + 1);
    let low = rom_table_word!(game, 0xa2838f, index);
    slot!(
        game,
        Boss[e].base(y_pos, y_subpos) += (u32::from(high) << 16) | u32::from(low)
    );
    if (i32::from(slot!(game, Boss[e].base.y_pos)) - 200) & 0x8000 != 0 {
        slot!(game, Boss[e].var_28, |old| old.wrapping_add(192));
    } else {
        slot!(game, Boss[e].base.y_pos = 200);
        slot!(game, Boss[e].var_D = 0x9aca);
        game.eproj_spawn_pt = Point16U {
            x: slot!(game, Boss[e].base.x_pos),
            y: slot!(game, Boss[e].base.y_pos),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x1d, game);
        QueueSfx2_Max6(0x24, game);
        slot!(game, Boss[e].base.properties, |old| old | 0x500);
    }
}
pub(crate) fn rubble(k: u16, game: &mut Game) {
    let e = i32::from(k);
    if (i32::from(slot!(game, Boss[e].var_23)) - 192) & 0x8000 != 0 {
        if (i32::from(slot!(game, Boss[e].var_23)) - 64) & 0x8000 == 0 {
            let explosion = (slot!(game, Boss[e].var_24) as i16).wrapping_sub(1);
            slot!(game, Boss[e].var_24 = explosion as u16);
            if explosion < 0 {
                slot!(game, Boss[e].var_24 = 12);
                let x = ((game.next_random() & 0x1f) + 232) as u16;
                let y = (i32::from(slot!(game, Boss[e].var_23))
                    + i32::from(game.next_random() & 0x1f)
                    - 8) as u16;
                create_sprite(x, y, 29, 2560, game);
                QueueSfx2_Max6(0x24, game);
            }
            let dust = (slot!(game, Boss[e].var_25) as i16).wrapping_sub(1);
            slot!(game, Boss[e].var_25 = dust as u16);
            if dust < 0 {
                slot!(game, Boss[e].var_25 = 4);
                for _ in 0..2 {
                    let x = ((game.next_random() & 0x3f) + 224) as u16;
                    let y = (i32::from(slot!(game, Boss[e].var_23))
                        + (i32::from(game.next_random() & 0x1f) - 8))
                        as u16;
                    create_sprite(x, y, 9, 2560, game);
                }
            }
        }
        slot!(game, Boss[e].var_23, |old| old.wrapping_add(1));
    } else {
        slot!(game, Boss[e].base.properties, |old| old
            | kEnemyProps_Deleted as u16);
        SetBossBitForCurArea(2, game);
        QueueMusic_Delayed8(3, game);
    }
}
fn mark_hit_death(e: i32, game: &mut Game) {
    if slot!(game, Boss[e].base.health) == 0 {
        slot!(game, Boss[e].var_2F = 1);
        slot!(game, Boss[0].base.properties, |old| old
            | kEnemyProps_Tangible as u16);
    }
}
pub(crate) fn shot(game: &mut Game) {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Boss[e].var_4C = slot!(game, Boss[e].base.health));
    combat::shot_reaction(&mut Runtime::new(game), api::ShotHandling::ReactionOnly);
    mark_hit_death(e, game);
}
pub(crate) fn contact(power_bomb: bool, game: &mut Game) {
    if power_bomb {
        combat::power_bomb_reaction(&mut Runtime::new(game), api::DeathHandling::LeavePending);
    } else {
        combat::contact_reaction(&mut Runtime::new(game), api::DeathHandling::LeavePending);
    }
    mark_hit_death(i32::from(word!(game.cur_enemy_index)), game);
}
fn select_path(k: u16, game: &mut Game) {
    let e = i32::from(k);
    if slot!(game, Boss[e].var_33) == 0 {
        slot!(game, Boss[e].var_3F = 0);
        let health = slot!(game, Boss[e].base.health) as i16;
        if health > 0 && health.wrapping_sub(slot!(game, Boss[e].var_5C) as i16) < 0 {
            let speed = if health.wrapping_sub(slot!(game, Boss[e].var_5D) as i16) < 0 {
                2
            } else {
                1
            };
            slot!(game, Boss[e].var_3F = speed);
            let index = (4u16.wrapping_mul(slot!(game, Boss[e].var_3F)) >> 1) as isize;
            slot!(game, Boss[e]; var_38 = rom_table_word!(game, 0xb394bb, index), var_C = rom_table_word!(game, 0xb394bb, index + 1));
        }
    }
    let bias = if slot!(game, Boss[e].var_33) != 0 {
        128
    } else {
        0
    };
    slot!(
        game,
        Boss[e].var_40 = (i32::from(game.next_random() & 0x18)
            + bias
            + 4 * i32::from(slot!(game, Boss[e].var_37))) as u16
    );
}
fn clamp_delta(value: u16) -> u16 {
    if (i32::from(value) - 256) & 0x8000 == 0 {
        255
    } else if (i32::from(value) + 256) & 0x8000 != 0 {
        (-255i32) as u16
    } else {
        value
    }
}
pub(crate) fn seek_hole(game: &mut Game) {
    let e = i32::from(word!(game.cur_enemy_index));
    let index = (slot!(game, Boss[e].var_37) >> 1) as isize;
    let x = clamp_delta(
        (i32::from(rom_table_word!(game, 0xb3949b, index)) + 4
            - i32::from(slot!(game, Boss[e].base.x_pos))) as u16,
    );
    let y = clamp_delta(
        (i32::from(rom_table_word!(game, 0xb3949b, index + 2)) + 4
            - i32::from(slot!(game, Boss[e].base.y_pos))) as u16,
    );
    let angle = api::angle_from_delta(x, y);
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Boss[e]; var_39 = angle, var_3A = u16::from(64u8.wrapping_sub(angle as u8)));
    if slot!(game, Boss[e].var_33) == slot!(game, Boss[e].var_34) {
        apply_direction(game);
    } else {
        slot!(game, Boss[e]; var_34 = slot!(game, Boss[e].var_33), var_3C = 1);
    }
}
fn apply_direction(game: &mut Game) {
    let e = i32::from(word!(game.cur_enemy_index));
    let vector = actor_direction_vector(
        slot!(game, Boss[e].var_3A),
        slot!(game, Boss[e].var_38),
        game,
    );
    Runtime::new(game).session_mut().compatibility_enemy_scratch = slot!(game, Boss[e].var_3A);
    let phase = slot!(game, Boss[e].var_3A);
    api::apply_vector(&mut Runtime::new(game), phase, vector);
}
fn native_path(cursor: u16, game: &mut Game) -> RomRecord {
    let address = 0xb30000 | u32::from(cursor);
    if cursor & 0x8000 == 0 {
        native_rom_diagnostic(address, game);
    }
    RomRecord {
        image: game.capture_native_rom(),
        base: lorom_offset(address) as usize,
    }
}
pub(crate) fn load_path(game: &mut Game) {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Boss[e]; var_E = 0xe28c, var_3C = 0);
    let record = native_path(slot!(game, Boss[e].var_40).wrapping_add(0xe150), game);
    slot!(game, Boss[e]; var_42 = record.word(0), var_37 = record.word(4));
    let direction = record.word(2) as i16;
    slot!(game, Boss[e].var_44 = direction as u16);
    if direction < 0 {
        slot!(game, Boss[e].var_42, |old| old.wrapping_sub(4));
    }
    advance_path(game);
}
pub(crate) fn advance_path(game: &mut Game) {
    let e = i32::from(word!(game.cur_enemy_index));
    let mut delta = [0u16; 2];
    let mut remaining = i32::from(slot!(game, Boss[e].var_38));
    let step = if slot!(game, Boss[e].var_44) & 0x8000 == 0 {
        2
    } else {
        -2
    };
    loop {
        let record = native_path(slot!(game, Boss[e].var_42), game);
        for (axis, total) in delta.iter_mut().enumerate() {
            let amount = (record.byte(axis) as i8) as i16 as u16;
            if amount == 0xff80 {
                slot!(game, Boss[e]; var_41 = 0, var_3C = 1);
                return;
            }
            *total = total.wrapping_add(amount);
        }
        slot!(
            game,
            Boss[e].var_42 = (i32::from(slot!(game, Boss[e].var_42)) + step) as u16
        );
        remaining -= 1;
        if remaining == 0 {
            break;
        }
    }
    if slot!(game, Boss[e].var_44) & 0x8000 != 0 {
        delta = delta.map(u16::wrapping_neg);
    }
    slot!(game, Boss[e].base.x_pos, |old| old.wrapping_add(delta[0]));
    slot!(game, Boss[e].base.y_pos, |old| old.wrapping_add(delta[1]));
}
fn animation(e: i32, instruction: u16, game: &mut Game) {
    slot!(game, Boss[e].base.current_instruction = instruction);
    slot!(game, Boss[e].var_3B = instruction);
    slot!(game, Boss[e].base; instruction_timer = 1, timer = 0);
}
pub(crate) fn animate_motion(game: &mut Game) {
    let e = i32::from(word!(game.cur_enemy_index));
    let x = slot!(game, Boss[e].base.x_pos).wrapping_sub(slot!(game, Boss[e].var_56));
    let y = slot!(game, Boss[e].base.y_pos).wrapping_sub(slot!(game, Boss[e].var_57));
    let angle = api::angle_from_delta(x, y);
    if x != 0 || y != 0 {
        let instruction = if slot!(game, Boss[e].var_33) != 0 {
            slot!(game, Boss[e].base.layer = 7);
            slot!(game, Boss[e].base.properties, |old| old
                | kEnemyProps_Tangible as u16);
            0x9389
        } else {
            slot!(game, Boss[e].base.layer = 2);
            slot!(game, Boss[e].base.properties, |old| old
                & !(kEnemyProps_Tangible as u16));
            slot!(
                game,
                Boss[e].var_45 = (i32::from(slot!(game, Boss[e].var_45))
                    + i32::from((angle as i8).wrapping_sub(slot!(game, Boss[e].var_45) as i8)) * 3
                        / 4) as u8 as u16
            );
            rom_table_word!(game, 0xb3946b, (slot!(game, Boss[e].var_45) >> 5) as isize)
        };
        if instruction != slot!(game, Boss[e].var_3B) {
            animation(e, instruction, game);
        }
    }
    slot!(game, Boss[e]; var_56 = slot!(game, Boss[e].var_54), var_57 = slot!(game, Boss[e].var_55),
        var_54 = slot!(game, Boss[e].var_52), var_55 = slot!(game, Boss[e].var_53),
        var_52 = slot!(game, Boss[e].var_50), var_53 = slot!(game, Boss[e].var_51),
        var_50 = slot!(game, Boss[e].base.x_pos), var_51 = slot!(game, Boss[e].base.y_pos));
}
pub(crate) fn aim_spit(game: &mut Game) -> u32 {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Boss[e].base.layer = 2);
    let actor = word!(game.cur_enemy_index);
    let angle = api::angle_to_player(&Runtime::new(game), Key::from_legacy(actor));
    slot!(game, Boss[e].var_3D = angle);
    let instruction = rom_table_word!(
        game,
        0xb3948b,
        ((angle as u8).wrapping_add(16) >> 5) as isize
    );
    animation(e, instruction, game);
    slot!(game, Boss[e]; var_3D = u16::from(64u8.wrapping_sub(slot!(game, Boss[e].var_3D) as u8)), var_F = 0x9ee0);
    if slot!(game, Boss[e].var_D) != 0x99e4 {
        slot!(game, Boss[e].var_F = 0x9f34);
    }
    0xb30000 | u32::from(slot!(game, Boss[e].var_F))
}
pub(crate) fn spit(wide: bool, game: &mut Game) {
    let e = i32::from(word!(game.cur_enemy_index));
    if wide && slot!(game, Boss[e].var_5A) == 0 {
        return;
    }
    game.g_ram[0x60b] =
        (slot!(game, Boss[e].var_3D) as u8).wrapping_sub(if wide { 32 } else { 16 });
    let speed = [2u16, 3, 4][slot!(game, Boss[e].var_3F) as usize];
    for _ in 0..if wide { 5 } else { 3 } {
        SpawnEprojWithGfx(
            speed,
            word!(game.cur_enemy_index),
            addr_kEproj_BotwoonsSpit as u16,
            game,
        );
        game.g_ram[0x60b] = game.g_ram[0x60b].wrapping_add(16);
    }
    let target = if wide {
        slot!(game, Boss[e].var_5A = 0);
        e
    } else {
        i32::from(word!(game.cur_enemy_index))
    };
    slot!(game, Boss[target].var_F = 0x9f7a);
}
pub(crate) fn wait_spit(k: u16, game: &mut Game) {
    let e = i32::from(k);
    let delay = (slot!(game, Boss[e].var_21) as i16).wrapping_sub(1);
    slot!(game, Boss[e].var_21 = delay as u16);
    if delay < 0 {
        slot!(game, Boss[e]; var_21 = 0, var_F = 0x9dc0);
    }
}
pub(crate) fn detect_portal(k: u16, game: &mut Game) {
    let e = i32::from(word!(game.cur_enemy_index));
    if slot!(game, Boss[i32::from(k)].var_35) != 0 {
        return;
    }
    for index in [12, 8, 4, 0] {
        if (0..4).all(|offset| {
            let coordinate = if offset < 2 {
                slot!(game, Boss[e].base.x_pos)
            } else {
                slot!(game, Boss[e].base.y_pos)
            };
            ((coordinate as i16)
                .wrapping_sub(rom_table_word!(game, 0xb3949b, index + offset) as i16)
                >= 0)
                == (offset % 2 == 0)
        }) {
            slot!(game, Boss[e].var_35 = 1);
            slot!(game, Boss[e].var_33, |old| old ^ 1);
            slot!(game, Boss[e].var_36 = slot!(game, Boss[e].var_B));
            slot!(game, Boss[e].var_26, |old| old.wrapping_sub(1));
            return;
        }
        slot!(game, Boss[e].var_35 = 0);
    }
}
pub(crate) fn set_hitbox(x: u16, y: u16, game: &mut Game) {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Boss[e].base; x_width = x, y_height = y);
}
pub(crate) fn set_spitting(game: &mut Game) {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Boss[e].var_5A = 1);
}
