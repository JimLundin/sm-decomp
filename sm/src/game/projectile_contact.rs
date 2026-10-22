//! Player/enemy projectile contact, with ordered damage and reaction effects.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game::contact_rules::{axis, is_shot};
use crate::game::enemy_combat::reduce_suit_damage;
use crate::game::enemy_ram::{native_read, native_write};
use crate::game::sm_80::QueueSfx1_Max6;
use crate::game::sm_91::Samus_DealDamage;
use crate::game::sprite_objects::create_sprite;
use crate::game_state::Game;
use crate::ram_map::{checked_word as word, word_arrays as arrays};
use crate::rtl::rom_record;
use crate::types::EprojDef;
use core::mem::offset_of;

fn mark(index: u16, game: &mut Game) {
    word!(game.projectile_dir[index] = word!(game.projectile_dir[index]) | 0x10);
}
pub unsafe fn player_projectiles_hit_samus(game: &mut Game) {
    word!(game.enemy_processing_stage = 10);
    if word!(game.flag_disable_projectile_interaction) != 0 {
        return;
    }
    let count = if word!(game.bomb_counter) != 0 {
        10
    } else {
        if word!(game.projectile_counter) == 0 {
            return;
        }
        5
    };
    if word!(game.projectile_invincibility_timer) != 0
        || word!(game.samus_contact_damage_index) != 0
    {
        return;
    }
    for index in 0..count {
        word!(game.collision_detection_index = index);
        let kind = word!(game.projectile_type[index]);
        if word!(game.projectile_damage[index]) == 0
            || kind & 0x8000 != 0
            || kind & 0xf00 >= 0x700
            || word!(game.projectile_dir[index]) & 0x10 != 0
        {
            continue;
        }
        if !axis(
            word!(game.projectile_x_pos[index]),
            word!(game.samus_x_pos),
            word!(game.projectile_x_radius[index]),
            word!(game.samus_x_radius),
        ) || !axis(
            word!(game.projectile_y_pos[index]),
            word!(game.samus_y_pos),
            word!(game.projectile_y_radius[index]),
            word!(game.samus_y_radius),
        ) {
            continue;
        }
        if kind & 0xff00 != 0x300 && kind & 0xff00 != 0x500 {
            mark(index, game);
            unsafe {
                Samus_DealDamage(
                    reduce_suit_damage(word!(game.projectile_damage[index]), game),
                    game,
                );
            }
            word!(game.samus_invincibility_timer = 96);
            word!(game.samus_knockback_timer = 5);
            unsafe {
                crate::c_library::__assert_fail(
                    c"0".as_ptr(),
                    c"src/sm_a0.c".as_ptr(),
                    2048,
                    c"void SamusProjectileInteractionHandler(void)".as_ptr(),
                );
            }
            word!(
                game.knockback_x_dir = (word!(game.samus_x_pos)
                    .wrapping_sub(native_read(game, arrays::eproj_x_pos))
                    as i16
                    >= 0) as u16
            );
            return;
        }
        if word!(game.projectile_variables[index]) == 8 {
            let x = word!(game.samus_x_pos);
            let bomb_x = word!(game.projectile_x_pos[index]);
            word!(
                game.bomb_jump_dir = if x == bomb_x {
                    2
                } else if (x.wrapping_sub(bomb_x) as i16) < 0 {
                    1
                } else {
                    3
                }
            );
        }
    }
}
pub unsafe fn enemy_projectiles_hit_samus(game: &mut Game) {
    word!(game.enemy_processing_stage = 11);
    if word!(game.samus_invincibility_timer) != 0 || word!(game.samus_contact_damage_index) != 0 {
        return;
    }
    for index in (0..18).rev() {
        if word!(game.eproj_id[index]) == 0
            || word!(game.eproj_properties[index]) & 0x2000 != 0
            || word!(game.eproj_radius[index]) == 0
        {
            continue;
        }
        let radii = word!(game.eproj_radius[index]);
        if axis(
            word!(game.samus_x_pos),
            word!(game.eproj_x_pos[index]),
            word!(game.samus_x_radius),
            radii & 0xff,
        ) && axis(
            word!(game.samus_y_pos),
            word!(game.eproj_y_pos[index]),
            word!(game.samus_y_radius),
            radii >> 8,
        ) {
            word!(game.collision_detection_index = index * 2);
            unsafe {
                hit_samus(index * 2, game);
            }
        }
    }
}
unsafe fn hit_samus(offset: u16, game: &mut Game) {
    word!(game.samus_invincibility_timer = 96);
    word!(game.samus_knockback_timer = 5);
    // Definition access uses the byte offset, even when other fields round it down.
    let address = arrays::eproj_id + usize::from(offset);
    let definition = unsafe { rom_record(0x86, native_read(game, address), game) };
    let instruction = definition.word(offset_of!(EprojDef, hit_instruction_list));
    let index = offset >> 1;
    if instruction != 0 {
        word!(game.eproj_instr_list_ptr[index] = instruction);
        word!(game.eproj_instr_timers[index] = 1);
    }
    if word!(game.eproj_properties[index]) & 0x4000 == 0 {
        native_write(game, address, 0);
    }
    unsafe {
        Samus_DealDamage(
            reduce_suit_damage(word!(game.eproj_properties[index]) & 0xfff, game),
            game,
        );
    }
    word!(
        game.knockback_x_dir = (word!(game.samus_x_pos).wrapping_sub(word!(game.eproj_x_pos[index]))
            as i16
            >= 0) as u16
    );
}
pub unsafe fn shoot_enemy_projectiles(game: &mut Game) {
    word!(game.enemy_processing_stage = 12);
    if word!(game.projectile_counter) == 0 {
        return;
    }
    for enemy in (0..18).rev() {
        if word!(game.eproj_id[enemy]) == 0 || word!(game.eproj_properties[enemy]) & 0x8000 == 0 {
            continue;
        }
        for shot in 0..5 {
            if word!(game.eproj_flags[enemy]) == 2 {
                break;
            }
            if is_shot(word!(game.projectile_type[shot]))
                && word!(game.eproj_x_pos[enemy]) & 0xffe0
                    == word!(game.projectile_x_pos[shot]) & 0xffe0
                && word!(game.eproj_y_pos[enemy]) & 0xffe0
                    == word!(game.projectile_y_pos[shot]) & 0xffe0
            {
                unsafe {
                    hit_enemy_projectile(enemy * 2, shot * 2, game);
                }
            }
        }
    }
}
unsafe fn hit_enemy_projectile(enemy_offset: u16, shot_offset: u16, game: &mut Game) {
    let shot = shot_offset >> 1;
    let enemy = enemy_offset >> 1;
    if word!(game.projectile_type[shot]) & 8 == 0 {
        mark(shot, game);
    }
    if word!(game.eproj_flags[enemy]) == 1 {
        // The original dud uses the enemy slot to index player-projectile positions.
        unsafe {
            create_sprite(
                word!(game.projectile_x_pos[enemy]),
                word!(game.projectile_y_pos[enemy]),
                6,
                0,
                game,
            );
            QueueSfx1_Max6(0x3d, game);
        }
    } else {
        word!(game.eproj_G[enemy] = word!(game.projectile_type[shot]));
        let definition = unsafe { rom_record(0x86, word!(game.eproj_id[enemy]), game) };
        word!(
            game.eproj_instr_list_ptr[enemy] =
                definition.word(offset_of!(EprojDef, shot_instruction_list))
        );
        word!(game.eproj_instr_timers[enemy] = 1);
        word!(game.eproj_pre_instr[enemy] = 0x84fb);
        word!(game.eproj_properties[enemy] = word!(game.eproj_properties[enemy]) & 0xfff);
    }
}
