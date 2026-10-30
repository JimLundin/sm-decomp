//! File-menu states and transitions over the original save and scratch-RAM formats.
use super::menu_tilemap::{self, main_menu as restore_main};
use super::prelude::*;
use super::save_slots;
use super::save_slots::load_area_map;
use crate::constants::bank_81::{
    ADDR_K_MENU_TILEMAP_CLEAR_SAMUS_A, ADDR_K_MENU_TILEMAP_CLEAR_WHICH_DATA,
    ADDR_K_MENU_TILEMAP_COPY_COMPLETED, ADDR_K_MENU_TILEMAP_COPY_SAMUS_TO_SAMUS,
    ADDR_K_MENU_TILEMAP_COPY_SAMUS_TO_WHERE, ADDR_K_MENU_TILEMAP_COPY_WHICH_DATA,
    ADDR_K_MENU_TILEMAP_DATA_CLEAR_MODE, ADDR_K_MENU_TILEMAP_DATA_CLEARED,
    ADDR_K_MENU_TILEMAP_DATA_COPY_MODE, ADDR_K_MENU_TILEMAP_EXIT, ADDR_K_MENU_TILEMAP_IS_THIS_OK,
    ADDR_K_MENU_TILEMAP_NO, ADDR_K_MENU_TILEMAP_YES,
};
use crate::save_file::header;

use crate::enemy_projectiles::table::with_file_select as file_select;

/// The selection missile's x beside the confirmation's yes and no.
const CONFIRMATION_MISSILE_X: u16 = 94;

fn copy_selection<H: Host + ?Sized>(game: &mut H) -> u16 {
    file_select(game, |menu| *menu.file_copy_selection())
}
fn set_copy_selection<H: Host + ?Sized>(value: u16, game: &mut H) {
    file_select(game, |menu| *menu.file_copy_selection() = value);
}
fn copy_source<H: Host + ?Sized>(game: &mut H) -> u16 {
    file_select(game, |menu| *menu.file_copy_source_slot())
}
fn set_copy_source<H: Host + ?Sized>(value: u16, game: &mut H) {
    file_select(game, |menu| *menu.file_copy_source_slot() = value);
}
fn copy_destination<H: Host + ?Sized>(game: &mut H) -> u16 {
    file_select(game, |menu| *menu.file_copy_destination_slot())
}
fn set_copy_destination<H: Host + ?Sized>(value: u16, game: &mut H) {
    file_select(game, |menu| *menu.file_copy_destination_slot() = value);
}
/// Restart the selection missile's animation at its first frame.
fn restart_missile<H: Host + ?Sized>(game: &mut H) {
    file_select(game, |menu| {
        *menu.missile_animation_timer() = 1;
        *menu.missile_animation_frame() = 0;
    });
}

const PALETTE_BUFFER: usize = crate::ram::word_arrays::PALETTE_BUFFER;
/// The MOSAIC mirror and the BG1SC mirror after it, read as one word.
const MOSAIC: usize = crate::ram::mirror::MOSAIC;

static BIT_SHL: [u8; 3] = [1, 2, 4];

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
    states[crate::menu!(game.MenuIndex) as usize](game);
    0
}
fn border<H: Host + ?Sized>(sprite: u16, game: &mut H) {
    let x = if sprite == 0x4a { 0x7c } else { 0x80 };
    crate::sprite_output::menu(sprite, x, 0x10, 3584, game);
}
fn prepare<H: Host + ?Sized>(game: &mut H) {
    game.fade(false);
    if (game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf) != 0 {
        return;
    }
    game.set_blank(true);
    game.queue_sound(2, 1_u8, 6);
    game.set_hdma_enabled(false);
    game.clear_hdma();
    crate::menu!(game.MenuIndex += 1);
    menu_tilemap::prepare_graphics(game);
    scroll! { game, 0; Bg1H, Bg2H, Bg3H, Bg1V, Bg2V, Bg3V }
    menu_tilemap::load_palette(game);
}
fn fade_to_options<H: Host + ?Sized>(game: &mut H) {
    crate::sprite_output::menu_selection(game);
    border(0x48, game);
    crate::sprite_output::file_helmets(false, game);
    game.fade(false);
    if (game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf) == 0 {
        word!(game.GAME_STATE = K_GAME_STATE_2_GAME_OPTIONS_MENU as u16);
        crate::menu!(game.MenuIndex = 0);
        // Clear the menu's reused RAM, $198D..BC.
        crate::word_transfers!(game, ascending, 24;
            native(crate::ram::words::MENU_SELECTION_MISSILE_ANIMATION_TIMER) <- value(0));
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
            | (16 * (game.ram().read_byte(crate::ram::mirror::INIDISP) as i32 & 0xf)) ^ 0xf0)
            as u8,
    );
    if (game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf) == 0 {
        game.set_blank(true);
        crate::menu!(game.MenuIndex += 1);
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
            | (16 * (game.ram().read_byte(crate::ram::mirror::INIDISP) as i32 & 0xf)) ^ 0xf0)
            as u8,
    );
    if (game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf) == 15 {
        crate::menu!(game.MenuIndex += 1);
    }
}
fn begin_operation<const COPY: bool, H: Host + ?Sized>(game: &mut H) {
    if !COPY {
        border(0x4a, game);
    }
    crate::menu!(game.MenuIndex += 1);
    if COPY {
        reset_copy(game);
    } else {
        reset_clear(game);
    }
}
fn reset_copy<H: Host + ?Sized>(game: &mut H) {
    menu_tilemap::clear(game);
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x52, ADDR_K_MENU_TILEMAP_DATA_COPY_MODE as u16, game);
    menu_tilemap::load(0x150, ADDR_K_MENU_TILEMAP_COPY_WHICH_DATA as u16, game);
    menu_tilemap::load(0x688, ADDR_K_MENU_TILEMAP_EXIT as u16, game);
    menu_tilemap::file_rows(menu_tilemap::FileRowEmphasis::Occupied, game);
    select_first_slot(game);
    crate::sprite_output::file_slot_cursor(211, game);
    file_select(game, |menu| {
        *menu.file_copy_source_slot() = 0;
        *menu.file_copy_destination_slot() = 0;
    });
}
fn select_first_slot<H: Host + ?Sized>(game: &mut H) {
    set_copy_selection(
        (word!(game.NONEMPTY_SAVE_SLOTS) | 8).trailing_zeros() as u16,
        game,
    );
}
fn next_occupied_slot<H: Host + ?Sized>(game: &mut H) -> Option<u8> {
    let up = if game.observed_inputs().pressed as i32 & K_BUTTON_UP as i32 != 0 {
        true
    } else if game.observed_inputs().pressed as i32 & K_BUTTON_DOWN as i32 != 0 {
        false
    } else {
        return None;
    };
    let mut slot = copy_selection(game) as u8;
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
        if BIT_SHL[slot as usize] as u16 & word!(game.NONEMPTY_SAVE_SLOTS) != 0 {
            return Some(slot);
        }
    }
}
fn choose_source<const COPY: bool, H: Host + ?Sized>(game: &mut H) {
    border(if COPY { 0x49 } else { 0x4a }, game);
    crate::sprite_output::menu_selection(game);
    let movement_high = if COPY {
        0
    } else {
        game.ram().read_byte(crate::ram::words::JOYPAD1_NEWKEYS + 1)
    };
    if game.observed_inputs().pressed as i32 & (K_BUTTON_START as i32 | K_BUTTON_A as i32) != 0 {
        game.queue_sound(0, 0x37_u8, 6);
        if copy_selection(game) != 3 {
            set_copy_source(copy_selection(game), game);
            crate::menu!(game.MenuIndex += 1);
            return;
        }
        if COPY {
            crate::menu!(game.MenuIndex += 7);
            return;
        }
    } else if (game.observed_inputs().pressed & 0x8000) != 0 {
        if COPY {
            crate::menu!(game.MenuIndex += 7);
            game.queue_sound(0, 0x37_u8, 6);
            return;
        }
    } else {
        if let Some(slot) = next_occupied_slot(game) {
            game.ram()
                .write_byte(crate::ram::word_arrays::CINEMATICBG_ARR1, slot);
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
    game.queue_sound(0, 0x37_u8, 6);
    crate::menu!(game.MenuIndex += 5);
}
fn begin_destination<H: Host + ?Sized>(game: &mut H) {
    border(0x49, game);
    draw_destination(game);
    crate::menu!(game.MenuIndex += 1);
    let mut selection: u16 = 0;
    let source = copy_source(game);
    while selection == source {
        selection = selection.wrapping_add(1);
        if (selection as i32 - 3i32) & 0x8000 == 0 {
            break;
        }
    }
    set_copy_selection(selection, game);
    crate::sprite_output::file_slot_cursor(211, game);
}
fn draw_destination<H: Host + ?Sized>(game: &mut H) {
    menu_tilemap::clear(game);
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x52, ADDR_K_MENU_TILEMAP_DATA_COPY_MODE as u16, game);
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x148, ADDR_K_MENU_TILEMAP_COPY_SAMUS_TO_WHERE as u16, game);
    set_native_word(
        0x3000 + 2 * 944,
        (copy_source(game)).wrapping_add(8298),
        game,
    );
    menu_tilemap::load(0x688, ADDR_K_MENU_TILEMAP_EXIT as u16, game);
    menu_tilemap::file_rows(menu_tilemap::FileRowEmphasis::Destination, game);
}
fn choose_destination<H: Host + ?Sized>(game: &mut H) {
    border(0x49, game);
    crate::sprite_output::menu_selection(game);
    if game.observed_inputs().pressed as i32 & (K_BUTTON_START as i32 | K_BUTTON_A as i32) != 0 {
        game.queue_sound(0, 0x37_u8, 6);
        if copy_selection(game) != 3 {
            set_copy_destination(copy_selection(game), game);
            crate::menu!(game.MenuIndex += 1);
        } else {
            crate::menu!(game.MenuIndex += 5);
            return;
        }
    } else if (game.observed_inputs().pressed & 0x8000) == 0 {
        'move_selection: {
            let step = if game.observed_inputs().pressed as i32 & K_BUTTON_UP as i32 != 0 {
                game.queue_sound(0, 0x37_u8, 6);
                u16::MAX
            } else if game.observed_inputs().pressed as i32 & K_BUTTON_DOWN as i32 != 0 {
                game.queue_sound(0, 0x37_u8, 6);
                1
            } else {
                break 'move_selection;
            };
            let mut slot = copy_selection(game);
            let source = copy_source(game);
            do_until! { slot != source;
                slot = slot.wrapping_add(step);
                if step == u16::MAX && slot & 0x8000 != 0 || step == 1 && slot == 4 {
                    break 'move_selection;
                }
            }
            set_copy_selection(slot, game);
        }
    } else {
        crate::menu!(game.MenuIndex -= 2);
        set_copy_selection(copy_source(game), game);
        game.queue_sound(0, 0x37_u8, 6);
        reset_copy(game);
        return;
    }
    crate::sprite_output::file_slot_cursor(212, game);
}
fn begin_confirmation<const COPY: bool, H: Host + ?Sized>(game: &mut H) {
    border(if COPY { 0x49 } else { 0x4a }, game);
    menu_tilemap::set_palette(0, game);
    if COPY {
        menu_tilemap::load(0x144, ADDR_K_MENU_TILEMAP_COPY_SAMUS_TO_SAMUS as u16, game);
        set_native_word(0x3000 + 2 * 942, copy_source(game).wrapping_add(8298), game);
        set_native_word(
            0x3000 + 2 * 955,
            copy_destination(game).wrapping_add(8298),
            game,
        );
    } else {
        menu_tilemap::load(0x140, ADDR_K_MENU_TILEMAP_CLEAR_SAMUS_A as u16, game);
        set_native_word(0x3000 + 2 * 949, copy_source(game).wrapping_add(8298), game);
        set_copy_destination(3, game);
    }
    draw_confirmation(game);
    crate::menu!(game.MenuIndex += 1);
    set_copy_selection(0, game);
    if COPY {
        word!(game.FILE_COPY_ARROW_PALETTE_TIMER = 8);
    }
}
fn draw_confirmation<H: Host + ?Sized>(game: &mut H) {
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x514, ADDR_K_MENU_TILEMAP_IS_THIS_OK as u16, game);
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x59c, ADDR_K_MENU_TILEMAP_YES as u16, game);
    crate::word_transfers!(game, ascending, 32; native(0x3c80) <- value(15));
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x65c, ADDR_K_MENU_TILEMAP_NO as u16, game);
    menu_tilemap::file_rows(menu_tilemap::FileRowEmphasis::Confirmation, game);
}
fn confirm_choice<const COPY: bool, H: Host + ?Sized>(game: &mut H) {
    border(if COPY { 0x49 } else { 0x4a }, game);
    crate::sprite_output::menu_selection(game);
    if COPY {
        animate_copy_arrow(game);
        crate::sprite_output::file_copy_arrow(game);
    }
    if game.observed_inputs().pressed as i32 & (K_BUTTON_UP as i32 | K_BUTTON_DOWN as i32) != 0 {
        set_copy_selection(copy_selection(game) ^ 1, game);
        game.queue_sound(0, 0x37_u8, 6);
    } else {
        let cancelled = if (game.observed_inputs().pressed & 0x8000) != 0 {
            true
        } else if game.observed_inputs().pressed as i32
            & (K_BUTTON_START as i32 | K_BUTTON_A as i32)
            == 0
        {
            false
        } else {
            game.queue_sound(0, 0x38_u8, 6);
            if copy_selection(game) == 0 {
                crate::menu!(game.MenuIndex += 1);
                return;
            }
            if COPY {
                crate::menu!(game.MenuIndex -= 4);
                reset_copy(game);
                return;
            }
            true
        };
        if cancelled {
            if COPY {
                crate::menu!(game.MenuIndex -= 3);
                set_copy_selection(copy_destination(game), game);
            } else {
                crate::menu!(game.MenuIndex -= 2);
                set_copy_selection(copy_source(game), game);
            }
            game.queue_sound(0, 0x37_u8, 6);
            if !COPY {
                reset_clear(game);
            }
            return;
        }
    }
    let y = if copy_selection(game) != 0 { 208 } else { 184 };
    file_select(game, |menu| {
        *menu.missile_y() = y;
        *menu.missile_x() = CONFIRMATION_MISSILE_X;
    });
}
fn animate_copy_arrow<H: Host + ?Sized>(game: &mut H) {
    if word!(game.FILE_COPY_ARROW_PALETTE_TIMER) != 0
        && word!(game.FILE_COPY_ARROW_PALETTE_TIMER -= 1, reread) == 0
    {
        word!(game.FILE_COPY_ARROW_PALETTE_TIMER = 4);
        // Rotate the arrow's six colours, 145..151, by one entry.
        let first: u16 = word!(game.PALETTE_BUFFER[145]);
        crate::word_transfers!(game, ascending, 6;
                native(PALETTE_BUFFER + 2 * 145) <- native(PALETTE_BUFFER + 2 * 146));
        word!(game.PALETTE_BUFFER[151] = first);
    }
}
fn copy_slot<H: Host + ?Sized>(game: &mut H) {
    static BIT_SHL_16BIT: [u16; 3] = [1, 2, 4];
    border(0x49, game);
    crate::sprite_output::menu_selection(game);
    animate_copy_arrow(game);
    crate::sprite_output::file_copy_arrow(game);
    let (source, destination) = (copy_source(game), copy_destination(game));
    let src_addr: u16 = save_slots::sram_offset(game, source as isize);
    let dst_addr: u16 = save_slots::sram_offset(game, destination as isize);
    game.sram().copy_within(
        src_addr as usize..src_addr as usize + 1628,
        dst_addr as usize,
    );
    // Copy the source's header words; the reads precede every header write.
    let source = usize::from(copy_source(game));
    let mirrored_checksum = game
        .sram()
        .word(header::slot(header::MIRRORED_CHECKSUMS, source));
    let mirrored_complement = game
        .sram()
        .word(header::slot(header::MIRRORED_COMPLEMENTS, source));
    let checksum = game.sram().word(header::slot(header::CHECKSUMS, source));
    let destination = usize::from(copy_destination(game));
    game.sram().set_word(
        header::slot(header::COMPLEMENTS, destination),
        game.sram().word(header::slot(header::COMPLEMENTS, source)),
    );
    game.sram()
        .set_word(header::slot(header::CHECKSUMS, destination), checksum);
    game.sram().set_word(
        header::slot(header::MIRRORED_COMPLEMENTS, destination),
        mirrored_complement,
    );
    game.sram().set_word(
        header::slot(header::MIRRORED_CHECKSUMS, destination),
        mirrored_checksum,
    );
    crate::menu!(game.MenuIndex += 1);
    crate::word_transfers!(game, ascending, 288; native(0x3b00) <- value(15));
    word!(game.NONEMPTY_SAVE_SLOTS |= (BIT_SHL_16BIT[copy_destination(game) as usize]));
    let row = ((4 * i32::from(copy_destination(game)) + 9) << 6) + 24;
    crate::word_transfers!(game, (2), 11;
        native(menu_tilemap::MENU_TILEMAP + row as usize) <- value(menu_tilemap::BLANK_TILE),
        native(0x35c0 + row as usize) <- value(15),
    );
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x510, ADDR_K_MENU_TILEMAP_COPY_COMPLETED as u16, game);
    menu_tilemap::file_rows(menu_tilemap::FileRowEmphasis::Confirmation, game);
    game.flush_sram();
}
fn complete_operation<const COPY: bool, H: Host + ?Sized>(game: &mut H) {
    border(if COPY { 0x49 } else { 0x4a }, game);
    if game.observed_inputs().pressed != 0 {
        game.queue_sound(0, 0x37_u8, 6);
        crate::menu!(game.MenuIndex += 1);
        if COPY {
            word!(game.SELECTED_SAVE_SLOT = save_slots::last_selected(game));
        } else {
            for slot in 0..3 {
                if save_slots::load(slot, game) == 0 {
                    word!(game.SELECTED_SAVE_SLOT = slot);
                    return;
                }
            }
            word!(game.SELECTED_SAVE_SLOT = 0);
        }
    }
}
fn resume_main<H: Host + ?Sized>(game: &mut H) {
    crate::menu!(game.MenuIndex -= 14);
}
fn reset_clear<H: Host + ?Sized>(game: &mut H) {
    menu_tilemap::clear(game);
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x50, ADDR_K_MENU_TILEMAP_DATA_CLEAR_MODE as u16, game);
    menu_tilemap::load(0x140, ADDR_K_MENU_TILEMAP_CLEAR_WHICH_DATA as u16, game);
    menu_tilemap::load(0x688, ADDR_K_MENU_TILEMAP_EXIT as u16, game);
    menu_tilemap::file_rows(menu_tilemap::FileRowEmphasis::Occupied, game);
    set_copy_source(0, game);
    select_first_slot(game);
    crate::sprite_output::file_slot_cursor(211, game);
}
fn clear_slot<H: Host + ?Sized>(game: &mut H) {
    static BIT_SHL_NOT: [u16; 3] = [0xfffe, 0xfffd, 0xfffb];
    border(0x4a, game);
    let source = copy_source(game);
    let sram_addr: i32 = save_slots::sram_offset(game, source as isize) as i32;
    game.sram()
        .fill(sram_addr as usize..sram_addr as usize + 1628, 0);
    let _v1: u16 = (2u16).wrapping_mul(copy_source(game));
    let slot = usize::from(copy_source(game));
    for table in [
        header::CHECKSUMS,
        header::COMPLEMENTS,
        header::MIRRORED_CHECKSUMS,
        header::MIRRORED_COMPLEMENTS,
    ] {
        game.sram().set_word(header::slot(table, slot), 0);
    }
    crate::menu!(game.MenuIndex += 1);
    save_slots::initialize(game);
    save_slots::load(copy_source(game), game);
    let source = copy_source(game);
    crate::room_view::set(game, crate::room_view::Word::Area, source);
    load_area_map(game);
    crate::word_transfers!(game, ascending, 288; native(0x3b00) <- value(15));
    word!(game.NONEMPTY_SAVE_SLOTS &= (BIT_SHL_NOT[copy_source(game) as usize]));
    menu_tilemap::set_palette(0, game);
    menu_tilemap::load(0x500, ADDR_K_MENU_TILEMAP_DATA_CLEARED as u16, game);
    menu_tilemap::file_rows(menu_tilemap::FileRowEmphasis::Confirmation, game);
    game.flush_sram();
}
fn return_main<H: Host + ?Sized>(game: &mut H) {
    border(0x48, game);
    crate::menu!(game.MenuIndex -= 26);
}
fn turn_helmet<H: Host + ?Sized>(game: &mut H) {
    crate::sprite_output::menu_selection(game);
    border(0x48, game);
    crate::sprite_output::file_helmets(true, game);
    if game.observed_inputs().pressed as i32 & (K_BUTTON_START as i32 | K_BUTTON_A as i32) != 0
        || file_select(game, |menu| {
            (0..3).any(|helmet| {
                *menu.helmet_animation_frame(helmet) == 7
                    && *menu.helmet_animation_timer(helmet) == 0
            })
        })
    {
        crate::menu!(game.MenuIndex += 1);
    }
}
fn load_background<H: Host + ?Sized>(game: &mut H) {
    // This menu spans $3600..$3dff, beyond the pause-map union field.
    crate::word_transfers!(game, descending, 1024; native(0x3dfe) <- rom(0x8edc00, 1023));
    menu_tilemap::upload(
        menu_tilemap::MENU_TILEMAP_BYTES,
        menu_tilemap::MENU_TILEMAP_SOURCE,
        1,
        true,
        game,
    );
    crate::menu!(game.MenuIndex += 1);
    restart_missile(game);
}
fn initialize<H: Host + ?Sized>(game: &mut H) {
    word!(game.SELECTED_SAVE_SLOT = save_slots::last_selected(game));
    menu_tilemap::main_menu(game);
}
fn fade_in_main<H: Host + ?Sized>(game: &mut H) {
    crate::sprite_output::file_helmets(false, game);
    crate::sprite_output::file_menu_cursor(game);
    crate::sprite_output::menu_selection(game);
    border(0x48, game);
    game.fade(true);
    if (game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf) == 15 {
        crate::menu!(game.MenuIndex += 1);
    }
}
fn choose_main<H: Host + ?Sized>(game: &mut H) {
    border(0x48, game);
    crate::sprite_output::menu_selection(game);
    crate::sprite_output::file_helmets(true, game);
    if game.observed_inputs().pressed as i32 & (K_BUTTON_START as i32 | K_BUTTON_A as i32) == 0 {
        let selection_unchanged: bool = if (game.observed_inputs().pressed as i32
            & K_BUTTON_UP as i32)
            != 0
        {
            let occupied = word!(game.NONEMPTY_SAVE_SLOTS) != 0;
            let selected = word!(game.SELECTED_SAVE_SLOT);
            let previous = if (selected as i16).wrapping_sub(1) < 0 {
                5
            } else if !occupied && (i32::from(selected) - 5) & 0x8000 == 0 {
                2
            } else {
                selected.wrapping_sub(1)
            };
            word!(game.SELECTED_SAVE_SLOT = previous);
            false
        } else if (game.observed_inputs().pressed as i32 & K_BUTTON_DOWN as i32) == 0 {
            if (game.observed_inputs().pressed as i32 & K_BUTTON_B as i32) != 0 {
                game.queue_sound(0, 0x37_u8, 6);
                crate::menu!(game.MenuIndex = 33);
                game.queue_sound(0, 0x37_u8, 6);
            }
            true
        } else {
            let occupied = word!(game.NONEMPTY_SAVE_SLOTS) != 0;
            let mut next = word!(game.SELECTED_SAVE_SLOT).wrapping_add(1);
            if (i32::from(word!(game.SELECTED_SAVE_SLOT)) - (if occupied { 5 } else { 2 })) & 0x8000
                == 0
            {
                next = if !occupied && (i32::from(word!(game.SELECTED_SAVE_SLOT)) - 5) & 0x8000 != 0
                {
                    5
                } else {
                    0
                };
            }
            word!(game.SELECTED_SAVE_SLOT = next);
            false
        };
        if !(selection_unchanged) {
            game.queue_sound(0, 0x37_u8, 6);
        }
    } else {
        let mut selection: i16 = word!(game.SELECTED_SAVE_SLOT) as i16;
        if (word!(game.SELECTED_SAVE_SLOT) as i32 - 3i32) & 0x8000 != 0 {
            game.queue_sound(0, 0x2a_u8, 6);
            crate::menu!(game.MenuIndex += 27);
            // Start the selected slot's helmet animation.
            let helmet = usize::from(word!(game.SELECTED_SAVE_SLOT));
            if helmet < 3 {
                file_select(game, |menu| *menu.helmet_animation_timer(helmet) = 1);
            } else {
                // A slot past the third names a word past the helmets.
                set_native_word(
                    crate::ram::words::SLOT_A_HELMET_ANIMATION_TIMER + 2 * helmet,
                    1,
                    game,
                );
            }
            game.sram()
                .set_word(header::LAST_SLOT, word!(game.SELECTED_SAVE_SLOT));
            game.sram().set_word(
                header::LAST_SLOT_COMPLEMENT,
                !(word!(game.SELECTED_SAVE_SLOT) as i32) as u16,
            );
            game.flush_sram();
            if save_slots::load(word!(game.SELECTED_SAVE_SLOT), game) != 0 {
                save_slots::initialize(game);
                word!(game.HAS_AREA_MAP = 0);
            } else {
                load_area_map(game);
            }
        } else {
            if word!(game.SELECTED_SAVE_SLOT) == 3 {
                game.queue_sound(0, 0x37_u8, 6);
                crate::menu!(game.MenuIndex += 1);
                set_native_word(MOSAIC, native_word(MOSAIC, game) & 0xff0c | 3, game);
            } else {
                if word!(game.SELECTED_SAVE_SLOT) == 4 {
                    game.queue_sound(0, 0x37_u8, 6);
                    crate::menu!(game.MenuIndex += 15);
                    selection = (i32::from(native_word(MOSAIC, game)) & 0xff0c | 3) as i16;
                    set_native_word(MOSAIC, selection as u16, game);
                }
                if selection == 5 {
                    crate::menu!(game.MenuIndex = 33);
                }
            }
            return;
        }
    }
    crate::sprite_output::file_menu_cursor(game);
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
