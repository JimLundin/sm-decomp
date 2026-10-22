//! Enemy selection and ordered behavior passes; slots remain encoded in SNES RAM.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game::enemy_combat::{
    enemy_death,
};
use crate::game::enemy_animation::advance_animation;
use crate::game::enemy_ram::{native_write, EnemyField as F, EnemySlot};
use crate::game::scene_drawing::queue_enemy;
use crate::game::sm_a0::{
    CallEnemyAi, EnemyWithNormalSpritesIsOffScreen,
};
use crate::game::sprite_objects::update_sprites;
use crate::game_state::Game;
use crate::ram_map::{checked_word as word, word_arrays as arrays};
use crate::rtl::rom_record;
use crate::types::{EnemyDef, EnemySpawnData};
use core::mem::offset_of;

#[forbid(unsafe_code)]
fn frozen(game: &Game) -> bool {
    word!(game.debug_time_frozen_for_enemies) | word!(game.time_is_frozen_flag) != 0
}

#[forbid(unsafe_code)]
pub(crate) fn select_enemies(game: &mut Game) {
    word!(game.UNUSED_word_7E0E46 += 1);
    word!(game.cur_enemy_index = 0);
    word!(game.active_enemy_indexes_write_ptr = 0);
    word!(game.interactive_enemy_indexes_write_ptr = 0);
    let all = word!(game.flag_process_all_enemies) != 0;
    loop {
        let index = word!(game.cur_enemy_index);
        let enemy = EnemySlot::new(index);
        let definition = enemy.word(F::Definition, game);
        if definition != 0 && definition != 0xdaff {
            let properties = enemy.word(F::Properties, game);
            if properties & 0x200 != 0 {
                enemy.set(F::Definition, 0, game);
            } else if all
                || properties & 0x800 != 0
                || enemy.word(F::AiHandlers, game) & 4 != 0
                || (enemy
                    .word(F::Width, game)
                    .wrapping_add(enemy.word(F::X, game))
                    .wrapping_sub(word!(game.layer1_x_pos)) as i16)
                    >= 0
                    && (enemy
                        .word(F::Width, game)
                        .wrapping_add(word!(game.layer1_x_pos))
                        .wrapping_add(256)
                        .wrapping_sub(enemy.word(F::X, game)) as i16)
                        >= 0
                    && (enemy
                        .word(F::Y, game)
                        .wrapping_add(8)
                        .wrapping_sub(word!(game.layer1_y_pos)) as i16)
                        >= 0
                    && (word!(game.layer1_y_pos)
                        .wrapping_add(248)
                        .wrapping_sub(enemy.word(F::Y, game)) as i16)
                        >= 0
            {
                let active = word!(game.active_enemy_indexes_write_ptr);
                if all {
                    word!(game.active_enemy_indexes[active >> 1] = word!(game.cur_enemy_index));
                    // The all-enemy path also leaves these extra interactive-list writes.
                    word!(game.interactive_enemy_indexes[active >> 1] = index);
                } else {
                    native_write(
                        game,
                        arrays::active_enemy_indexes
                            + usize::from(word!(game.active_enemy_indexes_write_ptr) >> 1) * 2,
                        word!(game.cur_enemy_index),
                    );
                }
                word!(game.active_enemy_indexes_write_ptr = active.wrapping_add(2));
                if enemy.word(F::Properties, game) & 0x400 == 0 {
                    let interactive = word!(game.interactive_enemy_indexes_write_ptr);
                    native_write(
                        game,
                        arrays::interactive_enemy_indexes
                            + usize::from(word!(game.interactive_enemy_indexes_write_ptr) >> 1) * 2,
                        index,
                    );
                    word!(game.interactive_enemy_indexes_write_ptr = interactive.wrapping_add(2));
                }
            }
        }
        word!(game.cur_enemy_index += 64);
        if word!(game.cur_enemy_index).wrapping_sub(2048) & 0x8000 == 0 {
            break;
        }
    }
    // A full active list terminates over the first interactive entry.
    native_write(
        game,
        arrays::active_enemy_indexes
            + usize::from(word!(game.active_enemy_indexes_write_ptr) >> 1) * 2,
        0xffff,
    );
    native_write(
        game,
        arrays::interactive_enemy_indexes
            + usize::from(word!(game.interactive_enemy_indexes_write_ptr) >> 1) * 2,
        0xffff,
    );
}

#[forbid(unsafe_code)]
pub(crate) fn shake_active_enemies(game: &mut Game) {
    let mut cursor = 0;
    loop {
        let index = word!(game.active_enemy_indexes[cursor]);
        if index == 0xffff {
            break;
        }
        EnemySlot::new(index).set(F::Shake, 2, game);
        cursor += 1;
    }
}

#[forbid(unsafe_code)]
pub(crate) fn update_frozen_enemy(game: &mut Game) {
    let enemy = EnemySlot::new(word!(game.cur_enemy_index));
    enemy.set(F::Flash, 0, game);
    let timer = enemy.word(F::Frozen, game);
    if timer == 0 || {
        enemy.set(F::Frozen, timer.wrapping_sub(1), game);
        word!(game.equipped_beams) & 2 == 0
    } {
        let handlers = enemy.word(F::AiHandlers, game) & !4;
        enemy.set(F::AiHandlers, handlers, game);
        enemy.set(F::Frozen, handlers, game);
    }
}

pub(crate) unsafe fn run_enemies(game: &mut Game) {
    if word!(game.first_free_enemy_index) != 0 {
        let shake = word!(game.enemy_index_to_shake);
        if shake != 0xffff {
            EnemySlot::new(shake).set(F::Shake, 64, game);
            word!(game.enemy_index_to_shake = 0xffff);
        }
        let mut cursor = 0;
        loop {
            let index = word!(game.active_enemy_indexes[cursor]);
            if index == 0xffff {
                break;
            }
            word!(game.cur_enemy_index = index);
            let enemy = EnemySlot::new(index);
            unsafe {
                process_enemy(enemy, game);
            }
            let flash = enemy.word(F::Flash, game);
            if flash != 0 && !frozen(game) {
                let flash = flash.wrapping_sub(1);
                enemy.set(F::Flash, flash, game);
                if flash.wrapping_sub(8) & 0x8000 != 0 {
                    enemy.set(F::AiHandlers, enemy.word(F::AiHandlers, game) & !2, game);
                }
            }
            cursor += 1;
        }
    }
    unsafe {
        update_sprites(game);
    }
    word!(game.random_enemy_counter += 1);
    for direction in 0..4 {
        word!(game.enemy_index_colliding_dirs[direction] = 0xffff);
    }
}

unsafe fn process_enemy(enemy: EnemySlot, game: &mut Game) {
    // Callbacks can change the global slot; field reads keep this captured handle.
    let run_ai = if enemy.word(F::Properties, game) & 0x400 == 0 {
        let invincibility = enemy.word(F::Invincibility, game);
        if invincibility != 0 {
            enemy.set(F::Invincibility, invincibility.wrapping_sub(1), game);
            true
        } else if word!(game.debug_disable_sprite_interact) == 0 {
            if !frozen(game) {
                unsafe {
                    crate::game::enemy_contact::process_contacts(game);
                }
                if enemy.word(F::Definition, game) == 0 {
                    return;
                }
            }
            enemy.word(F::ExtraProperties, game) & 1 == 0
        } else {
            true
        }
    } else {
        true
    };
    if run_ai {
        word!(game.UNUSED_word_7E17A2 = 0);
        let ran_ai = if !frozen(game) {
            let handlers = enemy.word(F::AiHandlers, game);
            let handler = if handlers == 0 {
                0
            } else {
                handlers.trailing_zeros() as u16 + 1
            };
            // Normal dispatch captures the bank before the definition diagnostic.
            let bank = enemy.bank(game);
            let definition = enemy.word(F::Definition, game).wrapping_add(2 * handler);
            let routine =
                unsafe { rom_record(0xa0, definition, game) }.word(offset_of!(EnemyDef, main_ai));
            unsafe {
                CallEnemyAi(u32::from(bank) << 16 | u32::from(routine), game);
            }
            true
        } else {
            let definition = unsafe { rom_record(0xa0, enemy.word(F::Definition, game), game) };
            let routine = definition.word(offset_of!(EnemyDef, time_is_frozen_ai));
            if routine != 0 {
                unsafe {
                    CallEnemyAi(u32::from(enemy.bank(game)) << 16 | u32::from(routine), game);
                }
                true
            } else {
                false
            }
        };
        if ran_ai && !frozen(game) {
            enemy.set(F::Frame, enemy.word(F::Frame, game).wrapping_add(1), game);
            if enemy.word(F::Properties, game) & 0x2000 != 0 {
                word!(game.enemy_processing_stage = 2);
                unsafe {
                    advance_animation(game);
                }
            }
        }
    }
    if enemy.word(F::ExtraProperties, game) & 1 != 0
        && (enemy.word(F::Flash, game) == 1 || enemy.word(F::Frozen, game) == 1)
    {
        native_write(
            game,
            0x7000
                + usize::from(word!(game.cur_enemy_index))
                + offset_of!(EnemySpawnData, cause_of_death),
            0,
        );
        unsafe {
            enemy_death(word!(game.cur_enemy_index), 0, game);
        }
    }
    if (enemy.word(F::ExtraProperties, game) & 4 != 0
        || EnemyWithNormalSpritesIsOffScreen(game) == 0)
        && enemy.word(F::Properties, game) & 0x300 == 0
        && word!(game.UNUSED_word_7E17A2) & 1 == 0
    {
        unsafe {
            queue_enemy(game);
        }
    }
}
