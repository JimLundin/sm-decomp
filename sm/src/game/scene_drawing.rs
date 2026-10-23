//! Layer-ordered scene drawing and live enemy graphics compatibility.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game::enemy_projectiles::{DrawEprojsWithPriority, EprojPriority};
use crate::game::enemy_ram::{
    EnemyField as F, EnemySlot, native_read as read, native_write as write,
};
use crate::game::player_projectiles::draw_explosions;
use crate::game::sm_81::{
    DrawSpritemapWithBaseTile, DrawSpritemapWithBaseTile2, DrawSpritemapWithBaseTileOffscreen,
};
use crate::game::sm_90::DrawSamusAndProjectiles;
use crate::game::sm_a0::CallEnemyGfxDrawHook;
use crate::game::sprite_objects::draw_sprites;
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
use crate::rtl::{Unreachable, rom_record};
use crate::types::EnemySpawnData;
use core::mem::offset_of;

unsafe fn queue_address(addr: u16) -> usize {
    if addr >= 0x2000 {
        unsafe {
            crate::c_library::__assert_fail(
                c"addr < 0x2000".as_ptr(),
                c"src/sm_rtl.h".as_ptr(),
                51,
                c"uint8 *RomPtr_RAM(uint16_t)".as_ptr(),
            );
        }
    }
    usize::from(addr)
}

pub(crate) unsafe fn queue_enemy(game: &mut Game) {
    let enemy = EnemySlot::new(word!(game.cur_enemy_index));
    let layer = enemy.word(F::Layer, game).wrapping_mul(2) >> 1;
    // The stored index is evaluated before the queue address and its assertion.
    let index = word!(game.cur_enemy_index);
    let destination = word!(game.enemy_drawing_queue_sizes[layer])
        .wrapping_add(crate::rtl::rom_table_word!(game, 0xa0b133, layer as isize));
    let address = unsafe { queue_address(destination) };
    write(game, address, index);
    word!(game.enemy_drawing_queue_sizes[layer] += 2);
}

pub(crate) unsafe fn draw_scene(game: &mut Game) {
    unsafe {
        draw_sprites(game);
        draw_explosions(game);
        DrawEprojsWithPriority(EprojPriority::Low, game);
    }
    for layer in 0..8 {
        match layer {
            3 => unsafe {
                DrawSamusAndProjectiles(game);
            },
            6 => unsafe {
                DrawEprojsWithPriority(EprojPriority::High, game);
            },
            _ => {}
        }
        let size = word!(game.enemy_drawing_queue_sizes[layer]);
        if size == 0 {
            continue;
        }
        let base = crate::rtl::rom_table_word!(game, 0xa0b133, layer as isize);
        let mut cursor = 0u16;
        word!(game.enemy_drawing_queue_sizes[layer] = 0);
        loop {
            let address = unsafe { queue_address(base.wrapping_add(cursor)) };
            let index = read(game, address);
            write(game, address, 0);
            word!(game.cur_enemy_index = index);
            unsafe {
                draw_enemy(game);
            }
            // Odd sizes and 0xffff never satisfy the original widened comparison.
            if u32::from(cursor) + 2 == u32::from(size) {
                break;
            }
            cursor = cursor.wrapping_add(2);
        }
    }
    let hook = u32::from_ne_bytes(core::array::from_fn(|i| game.g_ram[0x178c + i])) & 0xffffff;
    unsafe {
        CallEnemyGfxDrawHook(hook, game);
    }
}

unsafe fn draw_enemy(game: &mut Game) {
    let index = word!(game.cur_enemy_index);
    let enemy = EnemySlot::new(index);
    let spawn = 0x7000 + usize::from(index);
    // This slot stays captured across ROM diagnostics and sprite emission.
    let mut x = read(game, spawn + offset_of!(EnemySpawnData, xpos2))
        .wrapping_add(enemy.word(F::X, game))
        .wrapping_sub(word!(game.layer1_x_pos));
    let y = read(game, spawn + offset_of!(EnemySpawnData, ypos2))
        .wrapping_add(enemy.word(F::Y, game))
        .wrapping_sub(word!(game.layer1_y_pos));
    let shake = enemy.word(F::Shake, game);
    if shake != 0 {
        x = x.wrapping_add(if enemy.word(F::Frame, game) & 2 == 0 {
            1
        } else {
            0xffff
        });
        enemy.set(F::Shake, shake.wrapping_sub(1), game);
    }
    let frozen = enemy.word(F::Frozen, game);
    let palette = if enemy.word(F::Flash, game) != 0 && word!(game.random_enemy_counter) & 2 != 0 {
        0
    } else if frozen != 0 && (frozen >= 0x5a || frozen & 2 != 0) {
        0xc00
    } else {
        enemy.word(F::Palette, game)
    };
    let tile = enemy.word(F::Tile, game);
    if enemy.word(F::ExtraProperties, game) & 4 == 0 {
        word!(game.enemy_processing_stage = 1);
        unsafe {
            DrawSpritemapWithBaseTile(
                enemy.bank(game),
                enemy.word(F::Map, game),
                x,
                y,
                palette,
                tile,
                game,
            );
        }
        return;
    }
    if enemy.word(F::Map, game) & 0x8000 == 0 {
        unsafe {
            Unreachable(game);
        }
    }
    let mut remaining =
        i32::from(unsafe { rom_record(enemy.bank(game), enemy.word(F::Map, game), game) }.byte(0));
    let mut cursor = enemy.word(F::Map, game).wrapping_add(2);
    loop {
        let part = unsafe { rom_record(enemy.bank(game), cursor, game) };
        let tilemap = unsafe { rom_record(enemy.bank(game), part.word(4), game) }.word(0) == 0xfffe;
        let part_x = x.wrapping_add(part.word(0));
        let part_y = y.wrapping_add(part.word(2));
        if tilemap {
            if enemy.word(F::ExtraProperties, game) & 0x8000 != 0 {
                unsafe {
                    copy_enemy_tilemap(enemy.bank(game), part.word(4), game);
                }
            }
        } else if (u32::from(part_x) + 128) & 0xfe00 == 0 && (u32::from(part_y) + 128) & 0xfe00 == 0
        {
            let draw = if part_y.to_ne_bytes()[1] != 0 {
                DrawSpritemapWithBaseTileOffscreen
            } else {
                DrawSpritemapWithBaseTile2
            };
            unsafe {
                draw(
                    enemy.bank(game),
                    part.word(4),
                    part_x,
                    part_y,
                    palette,
                    tile,
                    game,
                );
            }
        }
        cursor = cursor.wrapping_add(8);
        remaining -= 1;
        if remaining == 0 {
            break;
        }
    }
}

unsafe fn copy_enemy_tilemap(bank: u8, address: u16, game: &mut Game) {
    let mut record = unsafe { rom_record(bank, address.wrapping_add(2), game) };
    loop {
        let destination = usize::from(record.word(0));
        if destination == 0xffff {
            break;
        }
        let bytes = usize::from(record.word(2)) * 2;
        record.base += 4;
        for (i, byte) in game.g_ram[destination..destination + bytes]
            .iter_mut()
            .enumerate()
        {
            *byte = record.byte(i);
        }
        record.base += bytes;
    }
    word!(game.nmi_flag_bg2_enemy_vram_transfer += 1);
}

#[forbid(unsafe_code)]
pub(crate) fn queue_enemy_tiles(game: &mut Game) {
    if game.nmi_flag_bg2_enemy_vram_transfer() != 0
        && game.debug_time_frozen_for_enemies() | game.time_is_frozen_flag() == 0
    {
        let cursor = game.vram_write_queue_tail();
        let size = game.enemy_bg2_tilemap_size();
        game.write_vram_queue_word(cursor, 0, size);
        game.write_vram_queue_word(cursor, 2, 0x2000);
        game.write_vram_queue_byte(cursor, 4, 126);
        game.write_vram_queue_word(cursor, 5, 0x4800);
        // Entry writes can overlap the tail; advance its live value.
        let tail = game.vram_write_queue_tail().wrapping_add(7);
        game.set_vram_write_queue_tail(tail);
    }
    game.set_nmi_flag_bg2_enemy_vram_transfer(0);
}
