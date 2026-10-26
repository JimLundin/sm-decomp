//! File-menu states and transitions over the original save and scratch-RAM formats.
use crate::game::constants::bank_81::{
    addr_kMenuTilemap_ClearSamusA, addr_kMenuTilemap_ClearWhichData,
    addr_kMenuTilemap_CopyCompleted, addr_kMenuTilemap_CopySamusToSamus,
    addr_kMenuTilemap_CopySamusToWhere, addr_kMenuTilemap_CopyWhichData,
    addr_kMenuTilemap_DataClearMode, addr_kMenuTilemap_DataCleared, addr_kMenuTilemap_DataCopyMode,
    addr_kMenuTilemap_Exit, addr_kMenuTilemap_IsThisOk, addr_kMenuTilemap_No,
    addr_kMenuTilemap_Yes,
};
use crate::game::menu_tilemap::{self, main_menu as restore_main};
use crate::game::music::{QueueSfx1_Max6, QueueSfx3_Max6};
use crate::game::prelude::*;
use crate::game::projectile_presentation_ram as sprite_output;
use crate::game::save_slots;
use crate::game::sm_80::LoadMirrorOfExploredMapTiles;
use crate::game::sm_82::{
    DrawBorderAroundDataClearMode, DrawBorderAroundDataCopyMode, DrawBorderAroundSamusData,
};
use crate::game::visual_runtime::{clear_hdma, set_hdma_enabled};

static STATES: [Func_V; 34] = callback_table! { nonnull Func_V;
    prepare, load_background,
    initialize, fade_in_main, choose_main,
    fade_from_main, begin_copy,
    fade_in, choose_copy, begin_destination,
    choose_destination, begin_copy_confirmation,
    confirm_copy, copy_slot,
    copy_complete, fade_to_main, restore_main,
    restore_cursor, resume_main, fade_from_main,
    begin_clear, fade_in,
    choose_clear, begin_clear_confirmation,
    confirm_clear, clear_slot,
    clear_complete, fade_to_main, restore_main,
    restore_cursor, return_main, turn_helmet,
    fade_to_options, fade_to_title,
};
static kBitShl: [u8; 3] = [1, 2, 4];

/// Requires a live, exclusively accessed backend for screen and graphics transitions.
/// The native ROM allocation must cover every referenced menu text and save-map record.
pub(crate) unsafe fn advance(game: &mut Game) {
    unsafe {
        STATES[word!(game.menu_index) as usize](game);
    }
}
unsafe fn prepare(game: &mut Game) {
    unsafe {
        crate::game::display_refresh::fade(false, game);
        if (game.g_ram[0x51] & 0xf) != 0 {
            return;
        }
        crate::game::display_refresh::set_blank(true, game);
        QueueSfx3_Max6(1, game);
        set_hdma_enabled(false, game);
        clear_hdma(game);
        word!(game.menu_index += 1);
        menu_tilemap::prepare_graphics(game);
        word! { game, 0;
            reg_BG1HOFS, reg_BG2HOFS, reg_BG3HOFS, reg_BG1VOFS, reg_BG2VOFS, reg_BG3VOFS,
        }
        menu_tilemap::load_palette(game);
    }
}
fn fade_to_options(game: &mut Game) {
    sprite_output::menu_selection(game);
    DrawBorderAroundSamusData(game);
    sprite_output::file_helmets(false, game);
    crate::game::display_refresh::fade(false, game);
    if (game.g_ram[0x51] & 0xf) == 0 {
        word! {game; game_state = kGameState_2_GameOptionsMenu as u16; menu_index = 0; }
        word_transfers!(game, ascending, 24; native(0x198d) <- value(0));
    }
}
fn fade_to_title(game: &mut Game) {
    DrawBorderAroundSamusData(game);
    sprite_output::file_helmets(false, game);
    crate::game::display_refresh::fade(false, game);
    if (game.g_ram[0x51] & 0xf) == 0 {
        game.request_reset();
    }
}
unsafe fn fade_from_main(game: &mut Game) {
    unsafe {
        sprite_output::menu_selection(game);
        fade_to_main(game);
    }
}
unsafe fn fade_to_main(game: &mut Game) {
    crate::game::display_refresh::fade(false, game);
    game.g_ram[0x57] =
        (game.g_ram[0x57] as i32 & 0xf | 16 * (game.g_ram[0x51] as i32 & 0xf) ^ 0xf0) as u8;
    if (game.g_ram[0x51] & 0xf) == 0 {
        crate::game::display_refresh::set_blank(true, game);
        word!(game.menu_index += 1);
    }
}
fn restore_cursor(game: &mut Game) {
    sprite_output::file_menu_cursor(game);
    fade_in(game);
}
fn fade_in(game: &mut Game) {
    sprite_output::menu_selection(game);
    crate::game::display_refresh::fade(true, game);
    game.g_ram[0x57] =
        (game.g_ram[0x57] as i32 & 0xf | 16 * (game.g_ram[0x51] as i32 & 0xf) ^ 0xf0) as u8;
    if (game.g_ram[0x51] & 0xf) == 15 {
        word!(game.menu_index += 1);
    }
}
unsafe fn begin_copy(game: &mut Game) {
    unsafe {
        word!(game.menu_index += 1);
        reset_copy(game);
    }
}
unsafe fn reset_copy(game: &mut Game) {
    menu_tilemap::clear(game);
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x52, addr_kMenuTilemap_DataCopyMode as u16, game);
    menu_tilemap::load(0x150, addr_kMenuTilemap_CopyWhichData as u16, game);
    menu_tilemap::load(0x688, addr_kMenuTilemap_Exit as u16, game);
    menu_tilemap::file_rows(menu_tilemap::FileRowEmphasis::Occupied, game);
    select_first_slot(game);
    sprite_output::file_slot_cursor(211, game);
    word! { game, 0; eproj_id[16], eproj_id[17] }
}
fn select_first_slot(game: &mut Game) {
    word!(game.eproj_id[15] = (word!(game.nonempty_save_slots) | 8).trailing_zeros() as u16);
}
fn next_occupied_slot(game: &mut Game) -> Option<u8> {
    let up = if word!(game.joypad1_newkeys) as i32 & kButton_Up as i32 != 0 {
        true
    } else if word!(game.joypad1_newkeys) as i32 & kButton_Down as i32 != 0 {
        false
    } else {
        return None;
    };
    let mut slot = word!(game.eproj_id[15]) as u8;
    loop {
        if up {
            slot = slot.wrapping_sub(1);
            if slot & 0x80 != 0 {
                return None;
            }
        } else {
            slot = slot.wrapping_add(1);
            if (slot as i8).wrapping_sub(4) >= 0 {
                return None;
            }
            if slot == 3 {
                return Some(slot);
            }
        }
        if kBitShl[slot as usize] as u16 & word!(game.nonempty_save_slots) != 0 {
            return Some(slot);
        }
    }
}
fn choose_copy(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    sprite_output::menu_selection(game);
    if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0 {
        QueueSfx1_Max6(0x37, game);
        if word!(game.eproj_id[15]) == 3 {
            word!(game.menu_index += 7);
        } else {
            word!(game.eproj_id[16] = word!(game.eproj_id[15]));
            word!(game.menu_index += 1);
        }
    } else if (word!(game.joypad1_newkeys) & 0x8000) != 0 {
        word!(game.menu_index += 7);
        QueueSfx1_Max6(0x37, game);
    } else {
        let selection = next_occupied_slot(game);
        if let Some(v0) = selection {
            game.g_ram[0x19b5] = v0;
            QueueSfx1_Max6(0x37, game);
            sprite_output::file_slot_cursor(211, game);
            return;
        }
        sprite_output::file_slot_cursor(211, game);
    };
}
unsafe fn begin_destination(game: &mut Game) {
    unsafe {
        DrawBorderAroundDataCopyMode(game);
        draw_destination(game);
        word!(game.menu_index += 1);
        let mut v0: u16 = 0;
        while v0 == word!(game.eproj_id[16]) {
            v0 = v0.wrapping_add(1);
            if v0 as i32 - 3i32 & 0x8000 == 0 {
                break;
            }
        }
        word!(game.eproj_id[15] = v0);
        sprite_output::file_slot_cursor(211, game);
    }
}
unsafe fn draw_destination(game: &mut Game) {
    menu_tilemap::clear(game);
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x52, addr_kMenuTilemap_DataCopyMode as u16, game);
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x148, addr_kMenuTilemap_CopySamusToWhere as u16, game);
    set_native_word(
        0x3000 + 2 * 944,
        (word!(game.eproj_id[16])).wrapping_add(8298),
        game,
    );
    menu_tilemap::load(0x688, addr_kMenuTilemap_Exit as u16, game);
    menu_tilemap::file_rows(menu_tilemap::FileRowEmphasis::Destination, game);
}
unsafe fn choose_destination(game: &mut Game) {
    unsafe {
        let mut v0: u16 = 0;
        DrawBorderAroundDataCopyMode(game);
        sprite_output::menu_selection(game);
        if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0 {
            QueueSfx1_Max6(0x37, game);
            if word!(game.eproj_id[15]) != 3 {
                word!(game.eproj_id[17] = word!(game.eproj_id[15]));
                word!(game.menu_index += 1);
            } else {
                word!(game.menu_index += 5);
                return;
            }
        } else if (word!(game.joypad1_newkeys) & 0x8000) == 0 {
            'move_selection: {
                if (word!(game.joypad1_newkeys) as i32 & kButton_Up as i32) != 0 {
                    QueueSfx1_Max6(0x37, game);
                    v0 = word!(game.eproj_id[15]);
                    do_until! { v0 != word!(game.eproj_id[16]);
                        v0 = v0.wrapping_sub(1);
                        if (v0 & 0x8000) != 0 {
                            break 'move_selection;
                        }
                    }
                } else if (word!(game.joypad1_newkeys) as i32 & kButton_Down as i32) != 0 {
                    QueueSfx1_Max6(0x37, game);
                    v0 = word!(game.eproj_id[15]);
                    do_until! { v0 != word!(game.eproj_id[16]);
                        v0 = v0.wrapping_add(1);
                        if v0 == 4 {
                            break 'move_selection;
                        }
                    }
                } else {
                    break 'move_selection;
                }
                word!(game.eproj_id[15] = v0);
            }
        } else {
            word!(game.menu_index -= 2);
            word!(game.eproj_id[15] = word!(game.eproj_id[16]));
            QueueSfx1_Max6(0x37, game);
            reset_copy(game);
            return;
        }
        sprite_output::file_slot_cursor(212, game);
    }
}
unsafe fn begin_copy_confirmation(game: &mut Game) {
    unsafe {
        DrawBorderAroundDataCopyMode(game);
        menu_tilemap::set_palette(0, game);
        menu_tilemap::load(0x144, addr_kMenuTilemap_CopySamusToSamus as u16, game);
        set_native_word(
            0x3000 + 2 * 942,
            (word!(game.eproj_id[16])).wrapping_add(8298),
            game,
        );
        set_native_word(
            0x3000 + 2 * 955,
            (word!(game.eproj_id[17])).wrapping_add(8298),
            game,
        );
        draw_confirmation(game);
        word!(game.menu_index += 1);
        word! {game; eproj_id[15] = 0; cinematic_var6 = 8; }
    }
}
unsafe fn draw_confirmation(game: &mut Game) {
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x514, addr_kMenuTilemap_IsThisOk as u16, game);
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x59c, addr_kMenuTilemap_Yes as u16, game);
    word_transfers!(game, ascending, 32; native(0x3c80) <- value(15));
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x65c, addr_kMenuTilemap_No as u16, game);
    menu_tilemap::file_rows(menu_tilemap::FileRowEmphasis::Confirmation, game);
}
unsafe fn confirm_copy(game: &mut Game) {
    unsafe {
        DrawBorderAroundDataCopyMode(game);
        sprite_output::menu_selection(game);
        animate_copy_arrow(game);
        sprite_output::file_copy_arrow(game);
        if word!(game.joypad1_newkeys) as i32 & (kButton_Up as i32 | kButton_Down as i32) != 0 {
            word!(game.eproj_id[15] ^= (1u16));
            QueueSfx1_Max6(0x37, game);
        } else {
            if (word!(game.joypad1_newkeys) & 0x8000) != 0 {
                word!(game.menu_index -= 3);
                word!(game.eproj_id[15] = word!(game.eproj_id[17]));
                QueueSfx1_Max6(0x37, game);
                return;
            }
            if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0 {
                QueueSfx1_Max6(0x38, game);
                if word!(game.eproj_id[15]) != 0 {
                    word!(game.menu_index -= 4);
                    reset_copy(game);
                } else {
                    word!(game.menu_index += 1);
                }
                return;
            }
        }
        let mut v0: u16 = 184;
        if word!(game.eproj_id[15]) != 0 {
            v0 = 208;
        }
        word! {game; eproj_id[10] = v0; eproj_id[5] = 94; }
    }
}
fn animate_copy_arrow(game: &mut Game) {
    if word!(game.cinematic_var6) != 0 {
        if word!(game.cinematic_var6 -= 1, reread) == 0 {
            word!(game.cinematic_var6 = 4);
            let v0: u16 = word!(game.palette_buffer[145]);
            word_transfers!(game, ascending, 6; native(0xc122) <- native(0xc124));
            word!(game.palette_buffer[151] = v0);
        }
    }
}
unsafe fn copy_slot(game: &mut Game) {
    unsafe {
        static kBitShl_16bit: [u16; 3] = [1, 2, 4];
        DrawBorderAroundDataCopyMode(game);
        sprite_output::menu_selection(game);
        animate_copy_arrow(game);
        sprite_output::file_copy_arrow(game);
        let src_addr: u16 =
            crate::rtl::rom_table_word!(game, 0x81812b, word!(game.eproj_id[16]) as isize);
        let dst_addr: u16 =
            crate::rtl::rom_table_word!(game, 0x81812b, word!(game.eproj_id[17]) as isize);
        game.g_sram.copy_within(
            src_addr as usize..src_addr as usize + 1628,
            dst_addr as usize,
        );
        let v2: i32 = word!(game.eproj_id[16]) as i32;
        let v10: i32 = game.g_sram.word((2 * v2 + 0x1ff0) as usize) as i32;
        let v9: i32 = game.g_sram.word((2 * v2 + 0x1ff8) as usize) as i32;
        let v8: i32 = game.g_sram.word((2 * v2) as usize) as i32;
        let v4: i32 = word!(game.eproj_id[17]) as i32;
        game.g_sram.set_word(
            (2 * v4 + 8) as usize,
            game.g_sram.word((2 * v2 + 8) as usize),
        );
        game.g_sram.set_word((2 * v4) as usize, v8 as u16);
        game.g_sram.set_word((2 * v4 + 0x1ff8) as usize, v9 as u16);
        game.g_sram.set_word((2 * v4 + 0x1ff0) as usize, v10 as u16);
        word!(game.menu_index += 1);
        word_transfers!(game, ascending, 288; native(0x3b00) <- value(15));
        word!(game.nonempty_save_slots |= (kBitShl_16bit[word!(game.eproj_id[17]) as usize]));
        let row = ((4 * i32::from(word!(game.eproj_id[17])) + 9) << 6) + 24;
        word_transfers!(game, (2), 11;
            native(0x3600 + row as usize) <- value(15),
            native(0x35c0 + row as usize) <- value(15),
        );
        menu_tilemap::set_palette(0, game);
        menu_tilemap::load(0x510, addr_kMenuTilemap_CopyCompleted as u16, game);
        menu_tilemap::file_rows(menu_tilemap::FileRowEmphasis::Confirmation, game);
        RtlWriteSram(game);
    }
}
fn copy_complete(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    if word!(game.joypad1_newkeys) != 0 {
        QueueSfx1_Max6(0x37, game);
        word!(game.menu_index += 1);
        word!(game.selected_save_slot = save_slots::last_selected(game));
    }
}
fn resume_main(game: &mut Game) {
    word!(game.menu_index -= 14);
}
unsafe fn begin_clear(game: &mut Game) {
    unsafe {
        DrawBorderAroundDataClearMode(game);
        word!(game.menu_index += 1);
        reset_clear(game);
    }
}
unsafe fn reset_clear(game: &mut Game) {
    menu_tilemap::clear(game);
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x50, addr_kMenuTilemap_DataClearMode as u16, game);
    menu_tilemap::load(0x140, addr_kMenuTilemap_ClearWhichData as u16, game);
    menu_tilemap::load(0x688, addr_kMenuTilemap_Exit as u16, game);
    menu_tilemap::file_rows(menu_tilemap::FileRowEmphasis::Occupied, game);
    word!(game.eproj_id[16] = 0);
    select_first_slot(game);
    sprite_output::file_slot_cursor(211, game);
}
fn choose_clear(game: &mut Game) {
    DrawBorderAroundDataClearMode(game);
    sprite_output::menu_selection(game);
    let movement_high = game.g_ram[0x90];
    if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0 {
        QueueSfx1_Max6(0x37, game);
        if word!(game.eproj_id[15]) != 3 {
            word!(game.eproj_id[16] = word!(game.eproj_id[15]));
            word!(game.menu_index += 1);
            return;
        }
    } else if (word!(game.joypad1_newkeys) & 0x8000) == 0 {
        let selection = next_occupied_slot(game);
        if let Some(v1) = selection {
            game.g_ram[0x19b5] = v1;
            QueueSfx1_Max6(u16::from_ne_bytes([55, movement_high]), game);
            sprite_output::file_slot_cursor(211, game);
            return;
        }
        sprite_output::file_slot_cursor(211, game);
        return;
    }
    QueueSfx1_Max6(0x37, game);
    word!(game.menu_index += 5);
}
unsafe fn begin_clear_confirmation(game: &mut Game) {
    unsafe {
        DrawBorderAroundDataClearMode(game);
        menu_tilemap::set_palette(0, game);
        menu_tilemap::load(0x140, addr_kMenuTilemap_ClearSamusA as u16, game);
        set_native_word(
            0x3000 + 2 * 949,
            (word!(game.eproj_id[16])).wrapping_add(8298),
            game,
        );
        word!(game.eproj_id[17] = 3);
        draw_confirmation(game);
        word!(game.menu_index += 1);
        word!(game.eproj_id[15] = 0);
    }
}
unsafe fn confirm_clear(game: &mut Game) {
    unsafe {
        let keep_confirming: bool;
        DrawBorderAroundDataClearMode(game);
        sprite_output::menu_selection(game);
        if word!(game.joypad1_newkeys) as i32 & (kButton_Up as i32 | kButton_Down as i32) != 0 {
            word!(game.eproj_id[15] ^= (1u16));
            QueueSfx1_Max6(0x37, game);
        } else {
            if (word!(game.joypad1_newkeys) & 0x8000) != 0 {
                keep_confirming = false;
            } else if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32)
                == 0
            {
                keep_confirming = true;
            } else {
                QueueSfx1_Max6(0x38, game);
                if word!(game.eproj_id[15]) != 0 {
                    keep_confirming = false;
                } else {
                    word!(game.menu_index += 1);
                    return;
                }
            }
            if keep_confirming {
            } else {
                word!(game.menu_index -= 2);
                word!(game.eproj_id[15] = word!(game.eproj_id[16]));
                QueueSfx1_Max6(0x37, game);
                reset_clear(game);
                return;
            }
        }
        let mut v0: u16 = 184;
        if word!(game.eproj_id[15]) != 0 {
            v0 = 208;
        }
        word! {game; eproj_id[10] = v0; eproj_id[5] = 94; }
    }
}
unsafe fn clear_slot(game: &mut Game) {
    unsafe {
        static kBitShl_Not: [u16; 3] = [0xfffe, 0xfffd, 0xfffb];
        DrawBorderAroundDataClearMode(game);
        let sram_addr: i32 =
            crate::rtl::rom_table_word!(game, 0x81812b, word!(game.eproj_id[16]) as isize) as i32;
        game.g_sram
            .fill(sram_addr as usize..sram_addr as usize + 1628, 0);
        let v1: u16 = (2u16).wrapping_mul(word!(game.eproj_id[16]));
        let v2: i32 = word!(game.eproj_id[16]) as i32;
        game.g_sram.set_word((2 * v2) as usize, 0);
        game.g_sram.set_word((2 * v2 + 8) as usize, 0);
        game.g_sram.set_word((2 * v2 + 0x1ff0) as usize, 0);
        game.g_sram.set_word((2 * v2 + 0x1ff8) as usize, 0);
        word!(game.menu_index += 1);
        save_slots::initialize(game);
        save_slots::load(word!(game.eproj_id[16]), game);
        word!(game.area_index = word!(game.eproj_id[16]));
        LoadMirrorOfExploredMapTiles(game);
        word_transfers!(game, ascending, 288; native(0x3b00) <- value(15));
        word!(game.nonempty_save_slots &= (kBitShl_Not[word!(game.eproj_id[16]) as usize]));
        menu_tilemap::set_palette(0, game);
        menu_tilemap::load(0x500, addr_kMenuTilemap_DataCleared as u16, game);
        menu_tilemap::file_rows(menu_tilemap::FileRowEmphasis::Confirmation, game);
        RtlWriteSram(game);
    }
}
unsafe fn clear_complete(game: &mut Game) {
    DrawBorderAroundDataClearMode(game);
    if word!(game.joypad1_newkeys) != 0 {
        QueueSfx1_Max6(0x37, game);
        word!(game.menu_index += 1);
        for slot in 0..3 {
            if save_slots::load(slot, game) == 0 {
                word!(game.selected_save_slot = slot);
                return;
            }
        }
        word!(game.selected_save_slot = 0);
    }
}
fn return_main(game: &mut Game) {
    DrawBorderAroundSamusData(game);
    word!(game.menu_index -= 26);
}
fn turn_helmet(game: &mut Game) {
    sprite_output::menu_selection(game);
    DrawBorderAroundSamusData(game);
    sprite_output::file_helmets(true, game);
    if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0
        || word!(game.eproj_id[2]) == 7 && word!(game.cinematic_var7) == 0
        || word!(game.eproj_id[3]) == 7 && word!(game.cinematic_var8) == 0
        || word!(game.eproj_id[4]) == 7 && word!(game.cinematic_var9) == 0
    {
        word!(game.menu_index += 1);
    }
}
fn load_background(game: &mut Game) {
    // This menu spans $3600..$3dff, beyond the pause-map union field.
    word_transfers!(game, descending, 1024; native(0x3dfe) <- rom(0x8edc00, 1023));
    menu_tilemap::upload(2048, 0x7e3600, 1, true, game);
    word!(game.menu_index += 1);
    word! {game; cinematic_var5 = 1; eproj_id[0] = 0; }
}
unsafe fn initialize(game: &mut Game) {
    unsafe {
        word!(game.selected_save_slot = save_slots::last_selected(game));
        menu_tilemap::main_menu(game);
    }
}
fn fade_in_main(game: &mut Game) {
    sprite_output::file_helmets(false, game);
    sprite_output::file_menu_cursor(game);
    sprite_output::menu_selection(game);
    DrawBorderAroundSamusData(game);
    crate::game::display_refresh::fade(true, game);
    if (game.g_ram[0x51] & 0xf) == 15 {
        word!(game.menu_index += 1);
    }
}
unsafe fn choose_main(game: &mut Game) {
    unsafe {
        let mut v2: u16 = 0;
        DrawBorderAroundSamusData(game);
        sprite_output::menu_selection(game);
        sprite_output::file_helmets(true, game);
        if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) == 0 {
            let selection_unchanged: bool =
                if (word!(game.joypad1_newkeys) as i32 & kButton_Up as i32) != 0 {
                    let occupied = word!(game.nonempty_save_slots) != 0;
                    let selected = word!(game.selected_save_slot);
                    let previous = if (selected as i16).wrapping_sub(1) < 0 {
                        5
                    } else if !occupied && i32::from(selected) - 5 & 0x8000 == 0 {
                        2
                    } else {
                        selected.wrapping_sub(1)
                    };
                    word!(game.selected_save_slot = previous);
                    false
                } else if (word!(game.joypad1_newkeys) as i32 & kButton_Down as i32) == 0 {
                    if (word!(game.joypad1_newkeys) as i32 & kButton_B as i32) != 0 {
                        QueueSfx1_Max6(0x37, game);
                        word!(game.menu_index = 33);
                        QueueSfx1_Max6(0x37, game);
                    }
                    true
                } else {
                    if word!(game.nonempty_save_slots) != 0 {
                        v2 = (word!(game.selected_save_slot)).wrapping_add(1);
                        if word!(game.selected_save_slot) as i32 - 5i32 & 0x8000 == 0 {
                            v2 = 0;
                        }
                    } else {
                        v2 = (word!(game.selected_save_slot)).wrapping_add(1);
                        if word!(game.selected_save_slot) as i32 - 2i32 & 0x8000 == 0 {
                            if word!(game.selected_save_slot) as i32 - 5i32 & 0x8000 != 0 {
                                v2 = 5;
                            } else {
                                v2 = 0;
                            }
                        }
                    }
                    word!(game.selected_save_slot = v2);
                    false
                };
            if !(selection_unchanged) {
                QueueSfx1_Max6(0x37, game);
            }
        } else {
            let mut v0: i16 = word!(game.selected_save_slot) as i16;
            if word!(game.selected_save_slot) as i32 - 3i32 & 0x8000 != 0 {
                QueueSfx1_Max6(0x2a, game);
                word!(game.menu_index += 27);
                set_native_word(
                    0x198d
                        + usize::from((2 * (i32::from(word!(game.selected_save_slot)) + 2)) as u16),
                    1,
                    game,
                );
                game.g_sram.set_word(0x1fec, word!(game.selected_save_slot));
                game.g_sram
                    .set_word(0x1fee, !(word!(game.selected_save_slot) as i32) as u16);
                RtlWriteSram(game);
                if save_slots::load(word!(game.selected_save_slot), game) != 0 {
                    save_slots::initialize(game);
                    word!(game.has_area_map = 0);
                } else {
                    LoadMirrorOfExploredMapTiles(game);
                }
            } else {
                if word!(game.selected_save_slot) == 3 {
                    QueueSfx1_Max6(0x37, game);
                    word!(game.menu_index += 1);
                    set_native_word(0x57, native_word(0x57, game) & 0xff0c | 3, game);
                } else {
                    if word!(game.selected_save_slot) == 4 {
                        QueueSfx1_Max6(0x37, game);
                        word!(game.menu_index += 15);
                        v0 = (i32::from(native_word(0x57, game)) & 0xff0c | 3) as i16;
                        set_native_word(0x57, v0 as u16, game);
                    }
                    if v0 == 5 {
                        word!(game.menu_index = 33);
                    }
                }
                return;
            }
        }
        sprite_output::file_menu_cursor(game);
    }
}

fn native_word(offset: usize, game: &Game) -> u16 {
    u16::from_ne_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
}

fn set_native_word(offset: usize, value: u16, game: &mut Game) {
    game.g_ram[offset..offset + 2].copy_from_slice(&value.to_ne_bytes());
}
