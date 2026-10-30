//! File-menu states and transitions over the original save and scratch-RAM formats.
use super::menu_tilemap::{self, main_menu as restore_main};
use super::prelude::*;
use super::save_slots;
use super::save_slots::load_area_map;
use crate::constants::bank_81::{
    addr_kMenuTilemap_ClearSamusA, addr_kMenuTilemap_ClearWhichData,
    addr_kMenuTilemap_CopyCompleted, addr_kMenuTilemap_CopySamusToSamus,
    addr_kMenuTilemap_CopySamusToWhere, addr_kMenuTilemap_CopyWhichData,
    addr_kMenuTilemap_DataClearMode, addr_kMenuTilemap_DataCleared, addr_kMenuTilemap_DataCopyMode,
    addr_kMenuTilemap_Exit, addr_kMenuTilemap_IsThisOk, addr_kMenuTilemap_No,
    addr_kMenuTilemap_Yes,
};

static kBitShl: [u8; 3] = [1, 2, 4];

/// Requires a live, exclusively accessed backend for screen and graphics transitions.
/// The native ROM allocation must cover every referenced menu text and save-map record.
pub fn advance<H: Host + ?Sized>(game: &mut H) -> CoroutineRet {
    // The original state table, indexed and bounds-checked as before.
    let states: [fn(&mut H); 34] = [
        prepare::<H>,
        load_background::<H>,
        initialize::<H>,
        fade_in_main::<H>,
        choose_main::<H>,
        fade_from_main::<H>,
        begin_operation::<true, H>,
        fade_in::<H>,
        choose_source::<true, H>,
        begin_destination::<H>,
        choose_destination::<H>,
        begin_confirmation::<true, H>,
        confirm_choice::<true, H>,
        copy_slot::<H>,
        complete_operation::<true, H>,
        fade_to_main::<H>,
        restore_main::<H>,
        restore_cursor::<H>,
        resume_main::<H>,
        fade_from_main::<H>,
        begin_operation::<false, H>,
        fade_in::<H>,
        choose_source::<false, H>,
        begin_confirmation::<false, H>,
        confirm_choice::<false, H>,
        clear_slot::<H>,
        complete_operation::<false, H>,
        fade_to_main::<H>,
        restore_main::<H>,
        restore_cursor::<H>,
        return_main::<H>,
        turn_helmet::<H>,
        fade_to_options::<H>,
        fade_to_title::<H>,
    ];
    states[word!(game.menu_index) as usize](game);
    0
}
fn border<H: Host + ?Sized>(sprite: u16, game: &mut H) {
    let x = if sprite == 0x4a { 0x7c } else { 0x80 };
    crate::sprite_output::menu(sprite, x, 0x10, 3584, game);
}
fn prepare<H: Host + ?Sized>(game: &mut H) {
    {
        game.fade(false);
        if (game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf) != 0 {
            return;
        }
        game.set_blank(true);
        game.queue_sound(2, (1) as u8, 6);
        game.set_hdma_enabled(false);
        game.clear_hdma();
        word!(game.menu_index += 1);
        menu_tilemap::prepare_graphics(game);
        word! { game, 0;
            reg_BG1HOFS, reg_BG2HOFS, reg_BG3HOFS, reg_BG1VOFS, reg_BG2VOFS, reg_BG3VOFS,
        }
        menu_tilemap::load_palette(game);
    }
}
fn fade_to_options<H: Host + ?Sized>(game: &mut H) {
    crate::sprite_output::menu_selection(game);
    border(0x48, game);
    crate::sprite_output::file_helmets(false, game);
    game.fade(false);
    if (game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf) == 0 {
        word! {game; game_state = kGameState_2_GameOptionsMenu as u16; menu_index = 0; }
        crate::word_transfers!(game, ascending, 24; native(0x198d) <- value(0));
    }
}
fn fade_to_title<H: Host + ?Sized>(game: &mut H) {
    border(0x48, game);
    crate::sprite_output::file_helmets(false, game);
    game.fade(false);
    if (game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf) == 0 {
        game.request_reset();
    }
}
fn fade_from_main<H: Host + ?Sized>(game: &mut H) {
    crate::sprite_output::menu_selection(game);
    fade_to_main(game);
}
fn fade_to_main<H: Host + ?Sized>(game: &mut H) {
    game.fade(false);
    game.ram().write_byte(
        crate::ram::mirror::MOSAIC,
        (game.ram().read_byte(crate::ram::mirror::MOSAIC) as i32 & 0xf
            | 16 * (game.ram().read_byte(crate::ram::mirror::INIDISP) as i32 & 0xf) ^ 0xf0)
            as u8,
    );
    if (game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf) == 0 {
        game.set_blank(true);
        word!(game.menu_index += 1);
    }
}
fn restore_cursor<H: Host + ?Sized>(game: &mut H) {
    crate::sprite_output::file_menu_cursor(game);
    fade_in(game);
}
fn fade_in<H: Host + ?Sized>(game: &mut H) {
    crate::sprite_output::menu_selection(game);
    game.fade(true);
    game.ram().write_byte(
        crate::ram::mirror::MOSAIC,
        (game.ram().read_byte(crate::ram::mirror::MOSAIC) as i32 & 0xf
            | 16 * (game.ram().read_byte(crate::ram::mirror::INIDISP) as i32 & 0xf) ^ 0xf0)
            as u8,
    );
    if (game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf) == 15 {
        word!(game.menu_index += 1);
    }
}
#[forbid(unsafe_code)]
fn begin_operation<const COPY: bool, H: Host + ?Sized>(game: &mut H) {
    if !COPY {
        border(0x4a, game);
    }
    word!(game.menu_index += 1);
    if COPY {
        reset_copy(game);
    } else {
        reset_clear(game);
    }
}
fn reset_copy<H: Host + ?Sized>(game: &mut H) {
    menu_tilemap::clear(game);
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x52, addr_kMenuTilemap_DataCopyMode as u16, game);
    menu_tilemap::load(0x150, addr_kMenuTilemap_CopyWhichData as u16, game);
    menu_tilemap::load(0x688, addr_kMenuTilemap_Exit as u16, game);
    menu_tilemap::file_rows(menu_tilemap::FileRowEmphasis::Occupied, game);
    select_first_slot(game);
    crate::sprite_output::file_slot_cursor(211, game);
    word! { game, 0; eproj_id[16], eproj_id[17] }
}
fn select_first_slot<H: Host + ?Sized>(game: &mut H) {
    word!(game.eproj_id[15] = (word!(game.nonempty_save_slots) | 8).trailing_zeros() as u16);
}
fn next_occupied_slot<H: Host + ?Sized>(game: &mut H) -> Option<u8> {
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
#[forbid(unsafe_code)]
fn choose_source<const COPY: bool, H: Host + ?Sized>(game: &mut H) {
    border(if COPY { 0x49 } else { 0x4a }, game);
    crate::sprite_output::menu_selection(game);
    let movement_high = if COPY {
        0
    } else {
        game.ram().read_byte(crate::ram::words::joypad1_newkeys + 1)
    };
    if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0 {
        game.queue_sound(0, (0x37) as u8, 6);
        if word!(game.eproj_id[15]) != 3 {
            word!(game.eproj_id[16] = word!(game.eproj_id[15]));
            word!(game.menu_index += 1);
            return;
        }
        if COPY {
            word!(game.menu_index += 7);
            return;
        }
    } else if (word!(game.joypad1_newkeys) & 0x8000) != 0 {
        if COPY {
            word!(game.menu_index += 7);
            game.queue_sound(0, (0x37) as u8, 6);
            return;
        }
    } else {
        if let Some(slot) = next_occupied_slot(game) {
            game.ram()
                .write_byte(crate::ram::word_arrays::cinematicbg_arr1, slot);
            let sound = if COPY {
                0x37
            } else {
                u16::from_ne_bytes([55, movement_high])
            };
            game.queue_sound(0, (sound) as u8, 6);
            crate::sprite_output::file_slot_cursor(211, game);
            return;
        }
        crate::sprite_output::file_slot_cursor(211, game);
        return;
    }
    game.queue_sound(0, (0x37) as u8, 6);
    word!(game.menu_index += 5);
}
fn begin_destination<H: Host + ?Sized>(game: &mut H) {
    border(0x49, game);
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
    crate::sprite_output::file_slot_cursor(211, game);
}
fn draw_destination<H: Host + ?Sized>(game: &mut H) {
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
#[forbid(unsafe_code)]
fn choose_destination<H: Host + ?Sized>(game: &mut H) {
    border(0x49, game);
    crate::sprite_output::menu_selection(game);
    if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0 {
        game.queue_sound(0, (0x37) as u8, 6);
        if word!(game.eproj_id[15]) != 3 {
            word!(game.eproj_id[17] = word!(game.eproj_id[15]));
            word!(game.menu_index += 1);
        } else {
            word!(game.menu_index += 5);
            return;
        }
    } else if (word!(game.joypad1_newkeys) & 0x8000) == 0 {
        'move_selection: {
            let step = if word!(game.joypad1_newkeys) as i32 & kButton_Up as i32 != 0 {
                game.queue_sound(0, (0x37) as u8, 6);
                u16::MAX
            } else if word!(game.joypad1_newkeys) as i32 & kButton_Down as i32 != 0 {
                game.queue_sound(0, (0x37) as u8, 6);
                1
            } else {
                break 'move_selection;
            };
            let mut slot = word!(game.eproj_id[15]);
            do_until! { slot != word!(game.eproj_id[16]);
                slot = slot.wrapping_add(step);
                if step == u16::MAX && slot & 0x8000 != 0 || step == 1 && slot == 4 {
                    break 'move_selection;
                }
            }
            word!(game.eproj_id[15] = slot);
        }
    } else {
        word!(game.menu_index -= 2);
        word!(game.eproj_id[15] = word!(game.eproj_id[16]));
        game.queue_sound(0, (0x37) as u8, 6);
        reset_copy(game);
        return;
    }
    crate::sprite_output::file_slot_cursor(212, game);
}
#[forbid(unsafe_code)]
fn begin_confirmation<const COPY: bool, H: Host + ?Sized>(game: &mut H) {
    border(if COPY { 0x49 } else { 0x4a }, game);
    menu_tilemap::set_palette(0, game);
    if COPY {
        menu_tilemap::load(0x144, addr_kMenuTilemap_CopySamusToSamus as u16, game);
        set_native_word(
            0x3000 + 2 * 942,
            word!(game.eproj_id[16]).wrapping_add(8298),
            game,
        );
        set_native_word(
            0x3000 + 2 * 955,
            word!(game.eproj_id[17]).wrapping_add(8298),
            game,
        );
    } else {
        menu_tilemap::load(0x140, addr_kMenuTilemap_ClearSamusA as u16, game);
        set_native_word(
            0x3000 + 2 * 949,
            word!(game.eproj_id[16]).wrapping_add(8298),
            game,
        );
        word!(game.eproj_id[17] = 3);
    }
    draw_confirmation(game);
    word!(game.menu_index += 1);
    word!(game.eproj_id[15] = 0);
    if COPY {
        word!(game.cinematic_var6 = 8);
    }
}
fn draw_confirmation<H: Host + ?Sized>(game: &mut H) {
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x514, addr_kMenuTilemap_IsThisOk as u16, game);
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x59c, addr_kMenuTilemap_Yes as u16, game);
    crate::word_transfers!(game, ascending, 32; native(0x3c80) <- value(15));
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x65c, addr_kMenuTilemap_No as u16, game);
    menu_tilemap::file_rows(menu_tilemap::FileRowEmphasis::Confirmation, game);
}
#[forbid(unsafe_code)]
fn confirm_choice<const COPY: bool, H: Host + ?Sized>(game: &mut H) {
    border(if COPY { 0x49 } else { 0x4a }, game);
    crate::sprite_output::menu_selection(game);
    if COPY {
        animate_copy_arrow(game);
        crate::sprite_output::file_copy_arrow(game);
    }
    if word!(game.joypad1_newkeys) as i32 & (kButton_Up as i32 | kButton_Down as i32) != 0 {
        word!(game.eproj_id[15] ^= (1u16));
        game.queue_sound(0, (0x37) as u8, 6);
    } else {
        let cancelled = if (word!(game.joypad1_newkeys) & 0x8000) != 0 {
            true
        } else if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32)
            == 0
        {
            false
        } else {
            game.queue_sound(0, (0x38) as u8, 6);
            if word!(game.eproj_id[15]) == 0 {
                word!(game.menu_index += 1);
                return;
            }
            if COPY {
                word!(game.menu_index -= 4);
                reset_copy(game);
                return;
            }
            true
        };
        if cancelled {
            if COPY {
                word!(game.menu_index -= 3);
                word!(game.eproj_id[15] = word!(game.eproj_id[17]));
            } else {
                word!(game.menu_index -= 2);
                word!(game.eproj_id[15] = word!(game.eproj_id[16]));
            }
            game.queue_sound(0, (0x37) as u8, 6);
            if !COPY {
                reset_clear(game);
            }
            return;
        }
    }
    let y = if word!(game.eproj_id[15]) != 0 {
        208
    } else {
        184
    };
    word! {game; eproj_id[10] = y; eproj_id[5] = 94; }
}
fn animate_copy_arrow<H: Host + ?Sized>(game: &mut H) {
    if word!(game.cinematic_var6) != 0 {
        if word!(game.cinematic_var6 -= 1, reread) == 0 {
            word!(game.cinematic_var6 = 4);
            let v0: u16 = word!(game.palette_buffer[145]);
            crate::word_transfers!(game, ascending, 6; native(0xc122) <- native(0xc124));
            word!(game.palette_buffer[151] = v0);
        }
    }
}
fn copy_slot<H: Host + ?Sized>(game: &mut H) {
    {
        static kBitShl_16bit: [u16; 3] = [1, 2, 4];
        border(0x49, game);
        crate::sprite_output::menu_selection(game);
        animate_copy_arrow(game);
        crate::sprite_output::file_copy_arrow(game);
        let src_addr: u16 = save_slots::sram_offset(game, word!(game.eproj_id[16]) as isize);
        let dst_addr: u16 = save_slots::sram_offset(game, word!(game.eproj_id[17]) as isize);
        game.sram().copy_within(
            src_addr as usize..src_addr as usize + 1628,
            dst_addr as usize,
        );
        let v2: i32 = word!(game.eproj_id[16]) as i32;
        let v10: i32 = game.sram().word((2 * v2 + 0x1ff0) as usize) as i32;
        let v9: i32 = game.sram().word((2 * v2 + 0x1ff8) as usize) as i32;
        let v8: i32 = game.sram().word((2 * v2) as usize) as i32;
        let v4: i32 = word!(game.eproj_id[17]) as i32;
        game.sram().set_word(
            (2 * v4 + 8) as usize,
            game.sram().word((2 * v2 + 8) as usize),
        );
        game.sram().set_word((2 * v4) as usize, v8 as u16);
        game.sram().set_word((2 * v4 + 0x1ff8) as usize, v9 as u16);
        game.sram().set_word((2 * v4 + 0x1ff0) as usize, v10 as u16);
        word!(game.menu_index += 1);
        crate::word_transfers!(game, ascending, 288; native(0x3b00) <- value(15));
        word!(game.nonempty_save_slots |= (kBitShl_16bit[word!(game.eproj_id[17]) as usize]));
        let row = ((4 * i32::from(word!(game.eproj_id[17])) + 9) << 6) + 24;
        crate::word_transfers!(game, (2), 11;
            native(0x3600 + row as usize) <- value(15),
            native(0x35c0 + row as usize) <- value(15),
        );
        menu_tilemap::set_palette(0, game);
        menu_tilemap::load(0x510, addr_kMenuTilemap_CopyCompleted as u16, game);
        menu_tilemap::file_rows(menu_tilemap::FileRowEmphasis::Confirmation, game);
        game.flush_sram();
    }
}
#[forbid(unsafe_code)]
fn complete_operation<const COPY: bool, H: Host + ?Sized>(game: &mut H) {
    border(if COPY { 0x49 } else { 0x4a }, game);
    if word!(game.joypad1_newkeys) != 0 {
        game.queue_sound(0, (0x37) as u8, 6);
        word!(game.menu_index += 1);
        if COPY {
            word!(game.selected_save_slot = save_slots::last_selected(game));
        } else {
            for slot in 0..3 {
                if save_slots::load(slot, game) == 0 {
                    word!(game.selected_save_slot = slot);
                    return;
                }
            }
            word!(game.selected_save_slot = 0);
        }
    }
}
fn resume_main<H: Host + ?Sized>(game: &mut H) {
    word!(game.menu_index -= 14);
}
fn reset_clear<H: Host + ?Sized>(game: &mut H) {
    menu_tilemap::clear(game);
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x50, addr_kMenuTilemap_DataClearMode as u16, game);
    menu_tilemap::load(0x140, addr_kMenuTilemap_ClearWhichData as u16, game);
    menu_tilemap::load(0x688, addr_kMenuTilemap_Exit as u16, game);
    menu_tilemap::file_rows(menu_tilemap::FileRowEmphasis::Occupied, game);
    word!(game.eproj_id[16] = 0);
    select_first_slot(game);
    crate::sprite_output::file_slot_cursor(211, game);
}
fn clear_slot<H: Host + ?Sized>(game: &mut H) {
    {
        static kBitShl_Not: [u16; 3] = [0xfffe, 0xfffd, 0xfffb];
        border(0x4a, game);
        let sram_addr: i32 =
            save_slots::sram_offset(game, word!(game.eproj_id[16]) as isize) as i32;
        game.sram()
            .fill(sram_addr as usize..sram_addr as usize + 1628, 0);
        let _v1: u16 = (2u16).wrapping_mul(word!(game.eproj_id[16]));
        let v2: i32 = word!(game.eproj_id[16]) as i32;
        game.sram().set_word((2 * v2) as usize, 0);
        game.sram().set_word((2 * v2 + 8) as usize, 0);
        game.sram().set_word((2 * v2 + 0x1ff0) as usize, 0);
        game.sram().set_word((2 * v2 + 0x1ff8) as usize, 0);
        word!(game.menu_index += 1);
        save_slots::initialize(game);
        save_slots::load(word!(game.eproj_id[16]), game);
        word!(game.area_index = word!(game.eproj_id[16]));
        load_area_map(game);
        crate::word_transfers!(game, ascending, 288; native(0x3b00) <- value(15));
        word!(game.nonempty_save_slots &= (kBitShl_Not[word!(game.eproj_id[16]) as usize]));
        menu_tilemap::set_palette(0, game);
        menu_tilemap::load(0x500, addr_kMenuTilemap_DataCleared as u16, game);
        menu_tilemap::file_rows(menu_tilemap::FileRowEmphasis::Confirmation, game);
        game.flush_sram();
    }
}
fn return_main<H: Host + ?Sized>(game: &mut H) {
    border(0x48, game);
    word!(game.menu_index -= 26);
}
fn turn_helmet<H: Host + ?Sized>(game: &mut H) {
    crate::sprite_output::menu_selection(game);
    border(0x48, game);
    crate::sprite_output::file_helmets(true, game);
    if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0
        || word!(game.eproj_id[2]) == 7 && word!(game.cinematic_var7) == 0
        || word!(game.eproj_id[3]) == 7 && word!(game.cinematic_var8) == 0
        || word!(game.eproj_id[4]) == 7 && word!(game.cinematic_var9) == 0
    {
        word!(game.menu_index += 1);
    }
}
fn load_background<H: Host + ?Sized>(game: &mut H) {
    // This menu spans $3600..$3dff, beyond the pause-map union field.
    crate::word_transfers!(game, descending, 1024; native(0x3dfe) <- rom(0x8edc00, 1023));
    menu_tilemap::upload(2048, 0x7e3600, 1, true, game);
    word!(game.menu_index += 1);
    word! {game; cinematic_var5 = 1; eproj_id[0] = 0; }
}
fn initialize<H: Host + ?Sized>(game: &mut H) {
    {
        word!(game.selected_save_slot = save_slots::last_selected(game));
        menu_tilemap::main_menu(game);
    }
}
fn fade_in_main<H: Host + ?Sized>(game: &mut H) {
    crate::sprite_output::file_helmets(false, game);
    crate::sprite_output::file_menu_cursor(game);
    crate::sprite_output::menu_selection(game);
    border(0x48, game);
    game.fade(true);
    if (game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf) == 15 {
        word!(game.menu_index += 1);
    }
}
fn choose_main<H: Host + ?Sized>(game: &mut H) {
    {
        border(0x48, game);
        crate::sprite_output::menu_selection(game);
        crate::sprite_output::file_helmets(true, game);
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
                        game.queue_sound(0, (0x37) as u8, 6);
                        word!(game.menu_index = 33);
                        game.queue_sound(0, (0x37) as u8, 6);
                    }
                    true
                } else {
                    let occupied = word!(game.nonempty_save_slots) != 0;
                    let mut next = word!(game.selected_save_slot).wrapping_add(1);
                    if i32::from(word!(game.selected_save_slot)) - (if occupied { 5 } else { 2 })
                        & 0x8000
                        == 0
                    {
                        next = if !occupied
                            && i32::from(word!(game.selected_save_slot)) - 5 & 0x8000 != 0
                        {
                            5
                        } else {
                            0
                        };
                    }
                    word!(game.selected_save_slot = next);
                    false
                };
            if !(selection_unchanged) {
                game.queue_sound(0, (0x37) as u8, 6);
            }
        } else {
            let mut v0: i16 = word!(game.selected_save_slot) as i16;
            if word!(game.selected_save_slot) as i32 - 3i32 & 0x8000 != 0 {
                game.queue_sound(0, (0x2a) as u8, 6);
                word!(game.menu_index += 27);
                set_native_word(
                    0x198d
                        + usize::from((2 * (i32::from(word!(game.selected_save_slot)) + 2)) as u16),
                    1,
                    game,
                );
                game.sram().set_word(0x1fec, word!(game.selected_save_slot));
                game.sram()
                    .set_word(0x1fee, !(word!(game.selected_save_slot) as i32) as u16);
                game.flush_sram();
                if save_slots::load(word!(game.selected_save_slot), game) != 0 {
                    save_slots::initialize(game);
                    word!(game.has_area_map = 0);
                } else {
                    load_area_map(game);
                }
            } else {
                if word!(game.selected_save_slot) == 3 {
                    game.queue_sound(0, (0x37) as u8, 6);
                    word!(game.menu_index += 1);
                    set_native_word(0x57, native_word(0x57, game) & 0xff0c | 3, game);
                } else {
                    if word!(game.selected_save_slot) == 4 {
                        game.queue_sound(0, (0x37) as u8, 6);
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
        crate::sprite_output::file_menu_cursor(game);
    }
}

fn native_word<H: Host + ?Sized>(offset: usize, game: &H) -> u16 {
    u16::from_ne_bytes([
        game.ram().read_byte(offset),
        game.ram().read_byte(offset + 1),
    ])
}

fn set_native_word<H: Host + ?Sized>(offset: usize, value: u16, game: &mut H) {
    game.ram().copy_in(offset, &value.to_ne_bytes());
}
