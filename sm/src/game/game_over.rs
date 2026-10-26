//! Game-over choices and transitions over the shared menu presentation.
use crate::game::constants::bank_81::{
    addr_kMenuTilemap_FindMetroidLarva, addr_kMenuTilemap_GameOver, addr_kMenuTilemap_NoGoToTitle,
    addr_kMenuTilemap_TryAgain, addr_kMenuTilemap_YesReturnToGame,
};
use crate::game::music::{HasQueuedMusic, QueueMusic_Delayed8};
use crate::game::music::{QueueSfx1_Max6, QueueSfx3_Max6};
use crate::game::prelude::*;
use crate::game::projectile_presentation_ram as sprite_output;
use crate::game::visual_runtime::{clear_hdma, set_hdma_enabled, spawn_hdma};
use crate::game::{menu_tilemap, save_slots};
use crate::ram_map::slot;

static STATES: [Func_V; 8] = callback_table! { nonnull Func_V;
    prepare, initialize, wait_for_music, fade_in, choose, fade_to_map, load_map, fade_to_title,
};

/// Requires a live, exclusively accessed graphics backend and native ROM allocation
/// covering the referenced menu text, baby animation and save-map records.
pub(crate) unsafe fn advance(game: &mut Game) -> CoroutineRet {
    unsafe { STATES[word!(game.menu_index) as usize](game) };
    0
}

unsafe fn prepare(game: &mut Game) {
    unsafe {
        crate::game::display_refresh::fade(false, game);
        if game.g_ram[0x51] & 0xf != 0 {
            return;
        }
        crate::game::display_refresh::set_blank(true, game);
        QueueSfx3_Max6(1, game);
        set_hdma_enabled(false, game);
        clear_hdma(game);
        word_transfers!(game, ascending, 256; pause_map(384) <- le(palette_buffer, 0));
        word_transfers!(game, ascending, 27; pause_map(640) <- native(0x51));
        menu_tilemap::prepare_graphics(game);
        word! { game, 0;
            reg_BG1HOFS, reg_BG2HOFS, reg_BG3HOFS, reg_BG1VOFS, reg_BG2VOFS, reg_BG3VOFS,
        }
        menu_tilemap::load_palette(game);
        word!(game.menu_index += 1);
    }
}

unsafe fn initialize(game: &mut Game) {
    unsafe {
        for (at, value) in [(0x69, 17), (0x74, 32), (0x75, 64), (0x76, 0x80)] {
            game.g_ram[at] = value;
        }
        QueueMusic_Delayed8(0, game);
        QueueMusic_Delayed8(0xff03, game);
        word! {game; cinematic_var5 = 1; eproj_id[0] = 0; }
        word_transfers!(game, ascending, 1024; native(0x3600) <- value(15));
        menu_tilemap::upload(2048, 0x7e3600, 0, true, game);
        menu_tilemap::set_palette(0, game);
        for (destination, text) in [
            (0x156, addr_kMenuTilemap_GameOver),
            (0x38a, addr_kMenuTilemap_FindMetroidLarva),
            (0x414, addr_kMenuTilemap_TryAgain),
            (0x4ce, addr_kMenuTilemap_YesReturnToGame),
            (0x5ce, addr_kMenuTilemap_NoGoToTitle),
        ] {
            menu_tilemap::load(destination, text as u16, game);
        }
        start_baby(0, 0, game);
        animate_baby(game, |game| {
            Unreachable(game);
        });
        let color = u16::from(game.g_ram[0x70]) << 8;
        game.g_ram[0x6f..0x71].copy_from_slice(&color.to_ne_bytes());
        set_hdma_enabled(false, game);
        clear_hdma(game);
        set_hdma_enabled(true, game);
        for (register, script) in [(0x32, 0x927d), (0x31, 0x928d)] {
            spawn_hdma(0x81, &SpawnHdmaObject_Args::new(0, register, script), game);
        }
        crate::game::display_refresh::set_blank(false, game);
        word!(game.menu_index += 1);
        word! { game, 0; screen_fade_delay, screen_fade_counter, file_select_map_area_index }
        word! {game; eproj_id[5] = 40; eproj_id[10] = 160; }
    }
}

fn wait_for_music(game: &mut Game) {
    if HasQueuedMusic(game) == 0 {
        word!(game.menu_index += 1);
        QueueMusic_Delayed8(4, game);
    }
}

crate::forward_calls! { unsafe fn (game: &mut Game);
    fade_in => fade(true, 15, game);
    fade_to_map => fade(false, 0, game);
}
unsafe fn fade(fade_in: bool, brightness: u8, game: &mut Game) {
    animate_baby(game, |game| {
        Unreachable(game);
    });
    sprite_output::menu_selection(game);
    crate::game::display_refresh::fade(fade_in, game);
    if game.g_ram[0x51] & 0xf == brightness {
        word!(game.menu_index += 1);
    }
}

unsafe fn choose(game: &mut Game) {
    animate_baby(game, |game| {
        Unreachable(game);
    });
    sprite_output::menu_selection(game);
    if word!(game.joypad1_newkeys) & (kButton_Select | kButton_Up | kButton_Down) as u16 != 0 {
        QueueSfx1_Max6(0x37, game);
        word!(game.file_select_map_area_index ^= 1);
    } else if word!(game.joypad1_newkeys) & kButton_A as u16 != 0 {
        slot!(game, EnemyData[0].instruction_timer = 180);
        if word!(game.file_select_map_area_index) != 0 {
            word!(game.menu_index = 7);
        } else {
            if word!(game.loading_game_state) == kGameState_31_SetUpNewGame as u16 {
                word!(game.game_state = word!(game.loading_game_state));
            } else {
                word!(game.menu_index += 1);
            }
            let _load_result = save_slots::load(word!(game.selected_save_slot), game);
            #[cfg(sm_verify_dll)]
            crate::flags_save_observer::selected_continue(_load_result);
        }
        return;
    }
    let y = if word!(game.file_select_map_area_index) != 0 {
        192
    } else {
        160
    };
    word! {game; eproj_id[5] = 40; eproj_id[10] = y; }
}

unsafe fn load_map(game: &mut Game) {
    set_hdma_enabled(false, game);
    unsafe { clear_hdma(game) };
    word! {game; game_state = kGameState_5_FileSelectMap as u16; menu_index = 0; }
}

unsafe fn fade_to_title(game: &mut Game) {
    sprite_output::menu_selection(game);
    animate_baby(game, |game| {
        Unreachable(game);
    });
    crate::game::display_refresh::fade(false, game);
    if game.g_ram[0x51] & 0xf == 0 {
        game.request_reset();
    }
}

/// The menu reuses the first enemy record; it remains the only animation state.
#[forbid(unsafe_code)]
fn animate_baby(game: &mut Game, invalid: impl FnOnce(&mut Game)) {
    let mut timer = slot!(game, EnemyData[0].instruction_timer);
    if timer == 0 {
        timer = start_baby(0xbc27, 10, game);
    }
    loop {
        let cursor = slot!(game, EnemyData[0].current_instruction);
        slot!(game, EnemyData[0].instruction_timer = timer.wrapping_sub(1));
        if timer != 1 {
            draw_baby(cursor, game);
            return;
        }
        let next = baby_record(cursor, game).word(6);
        if next != 0xffff {
            let ordinary = (next as i16).wrapping_add(1) >= 0;
            let (duration, bytes) = if ordinary {
                (next, 6)
            } else {
                let sound = match next {
                    0xbc0c => 0x23,
                    0xbc15 => 0x26,
                    0xbc1e => 0x27,
                    _ => {
                        invalid(game);
                        return;
                    }
                };
                QueueSfx3_Max6(sound, game);
                let at = slot!(game, EnemyData[0].current_instruction);
                (baby_record(at, game).word(8), 8)
            };
            slot!(game, EnemyData[0].instruction_timer = duration);
            slot!(game, EnemyData[0].current_instruction, |at| at
                .wrapping_add(bytes));
            if duration != 0xffff {
                let at = if ordinary {
                    cursor.wrapping_add(6)
                } else {
                    slot!(game, EnemyData[0].current_instruction)
                };
                draw_baby(at, game);
                return;
            }
        }
        timer = start_baby(0xbc27, 10, game);
    }
}

fn start_baby(cursor: u16, timer: u16, game: &mut Game) -> u16 {
    slot!(game, EnemyData[0].current_instruction = cursor);
    slot!(game, EnemyData[0].instruction_timer = timer);
    timer
}

fn baby_record(cursor: u16, game: &mut Game) -> crate::rom::RomRecord {
    let address = 0x820000 | u32::from(cursor);
    if cursor & 0x8000 == 0 {
        crate::game::session_runtime::native_rom_diagnostic(address, game);
    }
    crate::rom::RomRecord {
        image: game.capture_native_rom(),
        base: crate::rom::lorom_offset(address) as usize,
    }
}

fn draw_baby(cursor: u16, game: &mut Game) {
    let palette = baby_record(cursor, game).word(4);
    for index in 0..16 {
        let color = baby_record(palette.wrapping_add(2 * index), game).word(0);
        word!(game.palette_buffer[192 + usize::from(index)] = color);
    }
    let sprite = baby_record(cursor, game).word(2);
    sprite_output::menu(sprite, 0x7c, 0x50, 2048, game);
    sprite_output::menu(0x64, 0x7c, 0x50, 2560, game);
}
