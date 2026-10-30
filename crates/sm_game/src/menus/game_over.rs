//! Game-over choices and transitions over the shared menu presentation.
use super::prelude::*;
use crate::actors::layouts::EnemyBase;
use crate::constants::bank_81::{
    addr_kMenuTilemap_FindMetroidLarva, addr_kMenuTilemap_GameOver, addr_kMenuTilemap_NoGoToTitle,
    addr_kMenuTilemap_TryAgain, addr_kMenuTilemap_YesReturnToGame,
};
use crate::menus::menu_tilemap;
use crate::menus::save_slots;
use crate::ram::RawRam as _;
use crate::ram::slot;

/// Requires a live, exclusively accessed graphics backend and native ROM allocation
/// covering the referenced menu text, baby animation and save-map records.
pub fn advance<H: crate::menus::host::Host + ?Sized>(game: &mut H) -> CoroutineRet {
    // The original state table, indexed and bounds-checked as before.
    let states: [fn(&mut H); 8] = [
        prepare::<H>,
        initialize::<H>,
        wait_for_music::<H>,
        fade_in::<H>,
        choose::<H>,
        fade_to_map::<H>,
        load_map::<H>,
        fade_to_title::<H>,
    ];
    states[crate::menu!(game.menu_index) as usize](game);
    0
}

fn prepare<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    {
        game.fade(false);
        if game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf != 0 {
            return;
        }
        game.set_blank(true);
        game.queue_sound(2, 1_u8, 6);
        game.set_hdma_enabled(false);
        game.clear_hdma();
        crate::word_transfers!(game, ascending, 256; pause_map(384) <- le(palette_buffer, 0));
        crate::word_transfers!(game, ascending, 27; pause_map(640) <- native(0x51));
        menu_tilemap::prepare_graphics(game);
        word! { game, 0;
            reg_BG1HOFS, reg_BG2HOFS, reg_BG3HOFS, reg_BG1VOFS, reg_BG2VOFS, reg_BG3VOFS,
        }
        menu_tilemap::load_palette(game);
        crate::menu!(game.menu_index += 1);
    }
}

fn initialize<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    {
        for (at, value) in [(0x69, 17), (0x74, 32), (0x75, 64), (0x76, 0x80)] {
            game.ram().write_byte(at, value);
        }
        game.queue_music_delayed8(0);
        game.queue_music_delayed8(0xff03);
        word! {game; menu_selection_missile_animation_timer = 1; menu_selection_missile_animation_frame = 0; }
        crate::word_transfers!(game, ascending, 1024; native(0x3600) <- value(15));
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
            game.rtl_unreachable();
        });
        let color = u16::from(game.ram().read_byte(crate::ram::bytes::gameplay_CGWSEL)) << 8;
        game.ram()
            .copy_in(crate::ram::mirror::CGWSEL, &color.to_ne_bytes());
        game.set_hdma_enabled(false);
        game.clear_hdma();
        game.set_hdma_enabled(true);
        for (register, script) in [(0x32, 0x927d), (0x31, 0x928d)] {
            game.spawn_hdma(0x81, &SpawnHdmaObject_Args::new(0, register, script));
        }
        game.set_blank(false);
        crate::menu!(game.menu_index += 1);
        word! { game, 0; screen_fade_delay, screen_fade_counter, file_select_map_area_index }
        word! {game; menu_selection_missile_x_pos = 40; menu_selection_missile_y_pos = 160; }
    }
}

fn wait_for_music<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if game.has_queued_music() == 0 {
        crate::menu!(game.menu_index += 1);
        game.queue_music_delayed8(4);
    }
}

fn fade_in<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    fade(true, 15, game);
}
fn fade_to_map<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    fade(false, 0, game);
}
fn fade<H: crate::menus::host::Host + ?Sized>(fade_in: bool, brightness: u8, game: &mut H) {
    animate_baby(game, |game| {
        game.rtl_unreachable();
    });
    crate::sprite_output::menu_selection(game);
    game.fade(fade_in);
    if game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf == brightness {
        crate::menu!(game.menu_index += 1);
    }
}

fn choose<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    animate_baby(game, |game| {
        game.rtl_unreachable();
    });
    crate::sprite_output::menu_selection(game);
    if word!(game.joypad1_newkeys) & (kButton_Select | kButton_Up | kButton_Down) as u16 != 0 {
        game.queue_sound(0, 0x37_u8, 6);
        word!(game.file_select_map_area_index ^= 1);
    } else if word!(game.joypad1_newkeys) & kButton_A as u16 != 0 {
        slot!(game, EnemyBase[0].instruction_timer = 180);
        if word!(game.file_select_map_area_index) != 0 {
            crate::menu!(game.menu_index = 7);
        } else {
            if word!(game.loading_game_state) == kGameState_31_SetUpNewGame as u16 {
                word!(game.game_state = word!(game.loading_game_state));
            } else {
                crate::menu!(game.menu_index += 1);
            }
            let _load_result = save_slots::load(word!(game.selected_save_slot), game);
            game.observe_selected_continue(_load_result);
        }
        return;
    }
    let y = if word!(game.file_select_map_area_index) != 0 {
        192
    } else {
        160
    };
    word! {game; menu_selection_missile_x_pos = 40; menu_selection_missile_y_pos = y; }
}

fn load_map<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    game.set_hdma_enabled(false);
    {
        game.clear_hdma()
    };
    {
        word!(game.game_state = kGameState_5_FileSelectMap as u16);
        crate::menu!(game.menu_index = 0);
    }
}

fn fade_to_title<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    crate::sprite_output::menu_selection(game);
    animate_baby(game, |game| {
        game.rtl_unreachable();
    });
    game.fade(false);
    if game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf == 0 {
        game.request_reset();
    }
}

/// The menu reuses the first enemy record; it remains the only animation state.
#[forbid(unsafe_code)]
fn animate_baby<H: crate::menus::host::Host + ?Sized>(game: &mut H, invalid: impl FnOnce(&mut H)) {
    let mut timer = slot!(game, EnemyBase[0].instruction_timer);
    if timer == 0 {
        timer = start_baby(0xbc27, 10, game);
    }
    loop {
        let cursor = slot!(game, EnemyBase[0].current_instruction);
        slot!(game, EnemyBase[0].instruction_timer = timer.wrapping_sub(1));
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
                game.queue_sound(2, (sound) as u8, 6);
                let at = slot!(game, EnemyBase[0].current_instruction);
                (baby_record(at, game).word(8), 8)
            };
            slot!(game, EnemyBase[0].instruction_timer = duration);
            slot!(game, EnemyBase[0].current_instruction, |at| at
                .wrapping_add(bytes));
            if duration != 0xffff {
                let at = if ordinary {
                    cursor.wrapping_add(6)
                } else {
                    slot!(game, EnemyBase[0].current_instruction)
                };
                draw_baby(at, game);
                return;
            }
        }
        timer = start_baby(0xbc27, 10, game);
    }
}

fn start_baby<H: crate::menus::host::Host + ?Sized>(cursor: u16, timer: u16, game: &mut H) -> u16 {
    slot!(game, EnemyBase[0].current_instruction = cursor);
    slot!(game, EnemyBase[0].instruction_timer = timer);
    timer
}

fn baby_record<H: crate::menus::host::Host + ?Sized>(
    cursor: u16,
    game: &mut H,
) -> sm_data::cartridge::RomRecord {
    let address = 0x820000 | u32::from(cursor);
    if cursor & 0x8000 == 0 {
        game.native_rom_diagnostic(address);
    }
    sm_data::cartridge::RomRecord {
        image: game.native_image(),
        base: sm_data::cartridge::lorom_offset(address) as usize,
    }
}

fn draw_baby<H: crate::menus::host::Host + ?Sized>(cursor: u16, game: &mut H) {
    let palette = baby_record(cursor, game).word(4);
    for index in 0..16 {
        let color = baby_record(palette.wrapping_add(2 * index), game).word(0);
        word!(game.palette_buffer[192 + usize::from(index)] = color);
    }
    let sprite = baby_record(cursor, game).word(2);
    crate::sprite_output::menu(sprite, 0x7c, 0x50, 2048, game);
    crate::sprite_output::menu(0x64, 0x7c, 0x50, 2560, game);
}
