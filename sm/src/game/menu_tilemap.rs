//! Menu tilemaps: native word stores, live palette reads and ROM text controls.
use crate::game::constants::bank_81::{
    addr_kMenuTilemap_DataClear, addr_kMenuTilemap_DataCopy, addr_kMenuTilemap_Exit,
    addr_kMenuTilemap_SamusA, addr_kMenuTilemap_SamusB, addr_kMenuTilemap_SamusC,
    addr_kMenuTilemap_SamusData, addr_kMenuTilemap_TIME,
};
use crate::game::projectile_presentation_ram as sprite_output;
use crate::game::save_slots;
use crate::game_state::Game;
use crate::ram_map::{checked_word as at, word, words as fields};

const MENU_PALETTE: usize = 0xf78 + core::mem::offset_of!(crate::types::EnemyData, palette_index);
const FILE_LABELS: [u16; 3] = [
    addr_kMenuTilemap_SamusA as u16,
    addr_kMenuTilemap_SamusB as u16,
    addr_kMenuTilemap_SamusC as u16,
];

pub(crate) fn set_palette(value: u16, game: &mut Game) {
    game.g_ram[MENU_PALETTE..MENU_PALETTE + 2].copy_from_slice(&value.to_ne_bytes());
}

pub(crate) enum FileRowEmphasis {
    Occupied,
    Destination,
    Confirmation,
}

pub(crate) fn file_rows(emphasis: FileRowEmphasis, game: &mut Game) {
    for slot in 0..3u16 {
        save_slots::load(slot, game);
        let dim = match emphasis {
            FileRowEmphasis::Occupied => word!(game.nonempty_save_slots) & (1 << slot) == 0,
            FileRowEmphasis::Destination => word!(game.eproj_id[16]) == slot,
            FileRowEmphasis::Confirmation => {
                word!(game.eproj_id[16]) != slot && word!(game.eproj_id[17]) != slot
            }
        };
        set_palette(if dim { 1024 } else { 0 }, game);
        let mask = 1 << slot;
        let base = (slot + 2) << 8;
        health(!word!(game.nonempty_save_slots) & mask, base + 0x18, game);
        time(!word!(game.nonempty_save_slots) & mask, base + 0x72, game);
        load(base + 0x34, addr_kMenuTilemap_TIME as u16, game);
        load(base + 8, FILE_LABELS[usize::from(slot)], game);
    }
    upload(2048, 0x7e3600, 0, true, game);
}

/// Requires the native file-menu definitions for its text and tile reads.
pub(crate) unsafe fn main_menu(game: &mut Game) {
    clear(game);
    word!(game.nonempty_save_slots = u16::MAX);
    set_palette(0, game);
    load(0x56, addr_kMenuTilemap_SamusData as u16, game);
    for (slot, label) in FILE_LABELS.into_iter().enumerate() {
        // The first label follows the heading without resetting its live palette.
        if slot != 0 {
            set_palette(0, game);
        }
        let top = 0x146 + 0x140 * slot as u16;
        load(top, label, game);
        let empty = save_slots::load(slot as u16, game);
        word!(
            game.nonempty_save_slots =
                (word!(game.nonempty_save_slots) >> 1) | (u16::from(empty) << 15)
        );
        health(word!(game.nonempty_save_slots) & 0x8000, top + 0x16, game);
        time(word!(game.nonempty_save_slots) & 0x8000, top + 0x6e, game);
        load(top + 0x30, addr_kMenuTilemap_TIME as u16, game);
    }
    word!(game.nonempty_save_slots = (!word!(game.nonempty_save_slots)).swap_bytes() >> 5);
    if word!(game.nonempty_save_slots) != 0 {
        set_palette(0, game);
        load(0x508, addr_kMenuTilemap_DataCopy as u16, game);
        set_palette(0, game);
        load(0x5c8, addr_kMenuTilemap_DataClear as u16, game);
    }
    set_palette(0, game);
    load(0x688, addr_kMenuTilemap_Exit as u16, game);
    upload(2048, 0x7e3600, 0, true, game);
    word!(game.cinematic_var5 = 1);
    word! { game, 0;
        cinematic_var6, cinematic_var7, cinematic_var8, cinematic_var9, eproj_id[0], eproj_id[1],
        eproj_id[2], eproj_id[3], eproj_id[4], eproj_id[5], eproj_id[10], eproj_id[6], eproj_id[11],
    }
    word! { game, 100; eproj_id[7], eproj_id[8], eproj_id[9] }
    word! {game; eproj_id[12] = 47; eproj_id[13] = 87; eproj_id[14] = 127; }
    word! { game, 1; screen_fade_delay, screen_fade_counter }
    crate::game::display_refresh::set_blank(false, game);
    word!(game.menu_index += 1);
    word! { game, 0; eproj_id[15], eproj_id[16], eproj_id[17] }
}

/// Requires a live, exclusively accessed SNES backend.
pub(crate) unsafe fn prepare_graphics(game: &mut Game) {
    for (register, value) in [
        (0x6b, 0),
        (0x52, 3),
        (0x58, 81),
        (0x59, 88),
        (0x5a, 92),
        (0x5d, 0),
        (0x5e, 4),
        (0x69, 19),
    ] {
        game.g_ram[register] = value;
    }
    for (source, destination, bytes) in [
        (0x8e8000, 0, 0x5600),
        (0xb68000, 0x3000, 0x2000),
        (0xb6c000, 0x6000, 0x2000),
        (0x8ed600, 0x4000, 0x600),
    ] {
        unsafe {
            crate::game::display_refresh::GraphicsAsset::VramWords(source, destination, bytes)
                .load(game)
        };
    }
}

pub(crate) fn load_palette(game: &mut Game) {
    word_transfers!(game, (4), 128;
        le(palette_buffer, 0) <- rom(0x8ee400, 0),
        le(palette_buffer, 1) <- rom(0x8ee400, 1),
    );
}

pub(crate) fn upload(size: u16, source: u32, screen: usize, bank_word: bool, game: &mut Game) {
    let tail = word!(game.vram_write_queue_tail);
    let entry = 0xd0 + usize::from(tail);
    game.g_ram[entry..entry + 2].copy_from_slice(&size.to_ne_bytes());
    game.g_ram[entry + 2..entry + 4].copy_from_slice(&(source as u16).to_ne_bytes());
    let bank = (source >> 16) as u8;
    if bank_word {
        game.g_ram[entry + 4..entry + 6].copy_from_slice(&u16::from(bank).to_ne_bytes());
    } else {
        game.g_ram[entry + 4] = bank;
    }
    let destination = u16::from(game.g_ram[0x58 + screen] & 0xfc) << 8;
    game.g_ram[entry + 5..entry + 7].copy_from_slice(&destination.to_ne_bytes());
    word!(game.vram_write_queue_tail = tail.wrapping_add(7));
}

fn paint(offset: usize, tile: u16, game: &mut Game) {
    let palette = u16::from_ne_bytes([game.g_ram[MENU_PALETTE], game.g_ram[MENU_PALETTE + 1]]);
    let at = 0x3600 + offset;
    game.g_ram[at..at + 2].copy_from_slice(&(palette | tile).to_ne_bytes());
}

/// Capture checked native reads after the original low-window diagnostic.
pub(crate) fn native_record(address: u32, game: &mut Game) -> crate::rom::RomRecord {
    if address & 0x8000 == 0 {
        crate::game::session_runtime::native_rom_diagnostic(address, game);
    }
    crate::rom::RomRecord {
        image: game.capture_native_rom(),
        base: crate::rtl::lorom_offset(address) as usize,
    }
}

fn text_word(address: u16, game: &mut Game) -> u16 {
    // The warning can replace the native image; capture after it on every word.
    native_record(0x810000 | u32::from(address), game).word(0)
}

pub(crate) fn area_label(game: &mut Game) {
    let address = crate::rtl::rom_table_word!(game, 0x82965f, word!(game.area_index) as isize);
    let label = native_record(0x820000 | u32::from(address), game);
    for index in 0..12 {
        let tile = label.word(2 * index) & 0xefff;
        let destination = 0x4154 + 2 * index;
        game.g_ram[destination..destination + 2].copy_from_slice(&tile.to_ne_bytes());
    }
}

pub(crate) fn load(mut destination: u16, mut source: u16, game: &mut Game) {
    let mut row = i32::from(destination);
    loop {
        match text_word(source, game) {
            0xfffe => {
                row += 64;
                destination = row as u16;
            }
            0xffff => break,
            tile => {
                paint(usize::from(destination), tile, game);
                destination = destination.wrapping_add(2);
            }
        }
        source = source.wrapping_add(2);
    }
}

fn clear_row(offset: u16, game: &mut Game) {
    let count = 32 - usize::from((offset & 0x3f) >> 1);
    word_transfers!(game, ascending, count; native(0x3600 + usize::from(offset)) <- value(15));
}

fn number(offset: usize, value: u16, game: &mut Game) {
    paint(offset + 2, value % 10 + 8288, game);
    paint(offset, value / 10 + 8288, game);
}

pub(crate) fn health(empty: u16, offset: u16, game: &mut Game) {
    if empty != 0 {
        clear_row(offset, game);
        load(offset.wrapping_add(64), 0xb4ac, game);
        return;
    }
    load(offset, 0xb496, game);
    let mut full = word!(game.samus_health) / 100;
    let remainder = word!(game.samus_health) % 100;
    let maximum = word!(game.samus_max_health) / 100;
    let mut position = offset.wrapping_add(72);
    let mut columns = 7;
    for _ in 0..maximum {
        let tile = if full != 0 {
            full -= 1;
            152
        } else {
            153
        };
        paint(usize::from(position), tile, game);
        position = position.wrapping_add(2);
        columns -= 1;
        if columns == 0 {
            position = position.wrapping_sub(78);
            columns = 8;
        }
    }
    // Digit offsets stay linear; only the tank cursor wraps to sixteen bits.
    number(usize::from(offset) + 66, remainder, game);
}

pub(crate) fn time(empty: u16, offset: u16, game: &mut Game) {
    if empty != 0 {
        return;
    }
    number(usize::from(offset), word!(game.game_time_hours), game);
    load(offset.wrapping_add(4), 0xb4a8, game);
    number(usize::from(offset) + 6, word!(game.game_time_minutes), game);
}

#[forbid(unsafe_code)]
pub(crate) fn area_palette(area: u16, active: bool, game: &mut Game) {
    const OFFSETS: [u16; 6] = [0, 0xa, 0x10, 0x16, 0x24, 0x2a];
    let mut entry = OFFSETS[usize::from(area)] + if active { 0 } else { 0x30 };
    loop {
        let index = (entry >> 1) as isize;
        if crate::rtl::rom_table_word!(game, 0x81a4e6, index) == 0xffff {
            break;
        }
        let mut destination = crate::rtl::rom_table_word!(game, 0x81a4e6, index + 1);
        let mut source = crate::rtl::rom_table_word!(game, 0x81a4e6, index);
        for _ in 0..5 {
            word!(
                game.palette_buffer[destination >> 1] =
                    crate::rtl::rom_table_word!(game, 0x81a40e, (source >> 1) as isize)
            );
            destination = destination.wrapping_add(2);
            source = source.wrapping_add(2);
        }
        entry = entry.wrapping_add(4);
    }
}

pub(crate) fn clear(game: &mut Game) {
    word_transfers!(game, descending, 1024; native(0x3dfe) <- value(15));
}

pub(crate) fn controller_bindings(game: &mut Game) {
    for index in 0..7 {
        let destination = crate::rtl::rom_table_word!(game, 0x82f639, index);
        let address = 0x820000
            | u32::from(crate::rtl::rom_table_word!(
                game,
                0x82f647,
                word!(game.eproj_F[index + 13]) as isize
            ));
        if address & 0x8000 == 0 {
            crate::game::session_runtime::invalid_window(game, address);
        }
        let source = crate::word_transfers::rom_source(address, game);
        word_transfers!(game, ascending, 3;
            native(0x3000 + destination as isize) <- captured(source, 0));
        word_transfers!(game, ascending, 3;
            native(0x3040 + destination as isize) <- captured(source, 3));
    }
    let cleared_hint = [0, 0xe, 0xe, 0x10, 0x1f, 0x1f];
    for (index, destination) in [0x352e, 0x35ee].into_iter().enumerate() {
        if word!(game.eproj_instr_list_ptr[index]) != 5
            && word!(game.eproj_instr_list_ptr[index]) != 6
        {
            word_transfers!(game, ascending, 3;
                native(destination) <- table(cleared_hint, 0));
            word_transfers!(game, ascending, 3;
                native(destination + 0x40) <- table(cleared_hint, 3));
        }
    }
}

const RESERVE_ARROW_COLORS: [u16; 32] = [
    0x39e, 0x77d, 0xb5c, 0xf5b, 0x133a, 0x171a, 0x1f19, 0x22f8, 0x26d7, 0x2ad6, 0x2eb6, 0x3695,
    0x3a94, 0x3e73, 0x4253, 0x4a52, 0x4a52, 0x4253, 0x3e73, 0x3a94, 0x3695, 0x2eb6, 0x2ad6, 0x26d7,
    0x22f8, 0x1f19, 0x171a, 0x133a, 0xf5b, 0xb5c, 0x77d, 0x39e,
];
const RESERVE_ARROW_SHADOWS: [u16; 32] = [
    0x156, 0x155, 0x554, 0x954, 0xd53, 0xd52, 0x1152, 0x1551, 0x1970, 0x1d70, 0x1d6f, 0x216e,
    0x256e, 0x296d, 0x296c, 0x318c, 0x318c, 0x296c, 0x296d, 0x256e, 0x216e, 0x1d6f, 0x1d70, 0x1970,
    0x1551, 0x1152, 0xd52, 0xd53, 0x954, 0x554, 0x155, 0x156,
];

#[derive(Clone, Copy)]
pub(crate) enum ReserveArrow {
    Animated,
    On,
    Off,
}

pub(crate) fn update_reserve_arrow(game: &mut Game) {
    let mode = if word!(game.pausemenu_equipment_category_item) as u8 != 0 {
        ReserveArrow::Off
    } else {
        [ReserveArrow::Animated, ReserveArrow::On][usize::from(game.g_ram[0x756])]
    };
    reserve_arrow(mode, game);
}

pub(crate) fn reserve_arrow(mode: ReserveArrow, game: &mut Game) {
    let lit = match mode {
        ReserveArrow::Animated => word!(game.reserve_health_mode) == 1,
        ReserveArrow::On => true,
        ReserveArrow::Off => false,
    };
    if matches!(mode, ReserveArrow::Animated) && lit {
        let index = usize::from(game.g_ram[0x5b5] & 0x1f);
        word!(game.palette_buffer[102] = RESERVE_ARROW_COLORS[index]);
        word!(game.palette_buffer[107] = RESERVE_ARROW_SHADOWS[index]);
    } else {
        word! {game; palette_buffer[107] = 926; palette_buffer[102] = 342; }
    }
    reserve_arrow_tiles(lit, game);
}

pub(crate) fn reserve_arrow_tiles(lit: bool, game: &mut Game) {
    let palette = if lit { 0x1800 } else { 0x1c00 };
    for index in (0..8).map(|row| 129 + row * 32).chain(385..387) {
        recolor_native(0x3800 + 2 * index, palette, 1, game);
    }
}

pub(crate) fn copy_native_tiles(destination: usize, cursor: u16, bytes: usize, game: &mut Game) {
    let source = native_record(0x820000 | u32::from(cursor), game);
    for index in 0..bytes {
        game.g_ram[destination + index] = source.byte(index);
    }
}

pub(crate) fn recolor_native(destination: usize, palette: u16, words: usize, game: &mut Game) {
    for index in 0..words {
        let address = destination + 2 * index;
        let tile = u16::from_ne_bytes([game.g_ram[address], game.g_ram[address + 1]]);
        word_transfers!(game, ascending, 1;
            native(address) <- value((tile & !0x1c00) | palette));
    }
}

pub(crate) fn equipment_icons(game: &mut Game) {
    if word!(game.samus_max_reserve_health) != 0 {
        for index in 0..2 {
            let destination = crate::rtl::rom_table_word!(game, 0x82c068, index) as usize;
            let source = crate::rtl::rom_table_word!(game, 0x82c088, index);
            copy_native_tiles(destination, source, 14, game);
        }
    }
    for (offsets, count, masks, tiles) in [
        (0xc06c, 6, 0x82c04c, 0x82c08c),
        (0xc076, 6, 0x82c056, 0x82c096),
        (0xc082, 3, 0x82c062, 0x82c0a2),
    ] {
        let destinations = native_record(0x820000 | offsets, game);
        let weapons = offsets == 0xc06c;
        let hyper = weapons && word!(game.hyper_beam_flag) != 0;
        let (collected, equipped, bytes) = if weapons {
            (fields::collected_beams, fields::equipped_beams, 10)
        } else {
            (fields::collected_items, fields::equipped_items, 18)
        };
        for index in 0..if hyper { 5 } else { count } {
            let destination = destinations.word(2 * index) as usize;
            if hyper {
                let source = crate::rtl::rom_table_word!(game, 0x82c0a8, index as isize);
                copy_native_tiles(destination, source, 10, game);
            } else if at!(game, collected;)
                & crate::rtl::rom_table_word!(game, masks, index as isize)
                != 0
            {
                let source = crate::rtl::rom_table_word!(game, tiles, index as isize);
                copy_native_tiles(destination, source, bytes, game);
                if at!(game, equipped;) & crate::rtl::rom_table_word!(game, masks, index as isize)
                    == 0
                {
                    recolor_native(destination, 3072, bytes / 2, game);
                }
            } else {
                copy_native_tiles(destination, 0xc01a, bytes, game);
            }
        }
    }
}

fn wireframe_rows(
    destination: usize,
    row_stride: usize,
    mut source: impl FnMut(&mut Game) -> u16,
    game: &mut Game,
) {
    for row in 0..17 {
        for column in 0..8 {
            let tile = source(game);
            word_transfers!(game, ascending, 1;
                native(destination + row_stride * row + 2 * column) <- value(tile));
        }
    }
}

pub(crate) fn prepare_equipment_frame(game: &mut Game) {
    let mut cursor = 0xc639u16;
    wireframe_rows(
        0x31d8,
        32,
        |game| {
            let tile = native_record(0x820000 | u32::from(cursor), game).word(0);
            cursor = cursor.wrapping_add(2);
            tile
        },
        game,
    );
    equipment_wireframe(game);
    if word!(game.samus_max_reserve_health) != 0 {
        let hundreds = word!(game.samus_reserve_health) / 100;
        let remainder = word!(game.samus_reserve_health) % 100;
        reserve_digits(hundreds, remainder, game);
    }
}

pub(crate) fn equipment_wireframe(game: &mut Game) {
    let mut cursor = 0u16;
    while word!(game.equipped_items) & 0x101
        != crate::rtl::rom_table_word!(game, 0x82b257, (cursor >> 1) as isize)
    {
        cursor = cursor.wrapping_add(2);
    }
    let address = crate::rtl::rom_table_word!(game, 0x82b25f, (cursor >> 1) as isize);
    let record = native_record(0x820000 | u32::from(address), game);
    let mut offset = 0;
    wireframe_rows(
        0x39d8,
        64,
        |_| {
            let tile = record.word(offset);
            offset += 2;
            tile
        },
        game,
    );
}

pub(crate) fn reserve_label(tile: impl Fn(usize) -> u16, game: &mut Game) {
    for index in 0..4 {
        let address = 0x3800 + 2 * (327 + index);
        let previous = u16::from_ne_bytes([game.g_ram[address], game.g_ram[address + 1]]);
        word_transfers!(game, ascending, 1;
            native(address) <- value((previous & 0xfc00) | tile(index)));
    }
}

fn reserve_digits(hundreds: u16, remainder: u16, game: &mut Game) {
    for (index, digit) in [
        (394, remainder % 10),
        (393, remainder / 10),
        (392, hundreds),
    ] {
        word_transfers!(game, ascending, 1;
                native(0x3800 + 2 * index) <- value(digit + 2052));
    }
}

pub(crate) fn queue_equipment_wireframe(game: &mut Game) {
    equipment_wireframe(game);
    queue_ram_tilemap(1280, 0x3900, |_| 0x3080, game);
}

pub(crate) fn queue_pause_labels(game: &mut Game) {
    queue_ram_tilemap(
        128,
        0x3640,
        |game| (u16::from(game.g_ram[0x59] & 0xfc) << 8) + 800,
        game,
    );
}

fn queue_ram_tilemap(
    size: u16,
    source: u16,
    destination: impl FnOnce(&Game) -> u16,
    game: &mut Game,
) {
    let mut cursor = word!(game.vram_write_queue_tail);
    for value in [size, source] {
        word_transfers!(game, ascending, 1;
            native(0xd0 + usize::from(cursor)) <- value(value));
        cursor = cursor.wrapping_add(2);
    }
    game.g_ram[0xd0 + usize::from(cursor)] = 126;
    cursor = cursor.wrapping_add(1);
    let destination = destination(game);
    word_transfers!(game, ascending, 1;
        native(0xd0 + usize::from(cursor)) <- value(destination));
    word!(game.vram_write_queue_tail = cursor.wrapping_add(2));
}

fn reserve_palette(game: &mut Game) -> u16 {
    if word!(game.samus_reserve_health) == 0 {
        return 1536;
    }
    let expired = word!(game.pausemenu_reserve_tank_animation_timer -= 1, reread) & 0x8000 != 0;
    if word!(game.pausemenu_reserve_tank_animation_timer) == 0 || expired {
        word!(game.pausemenu_reserve_tank_animation_frame += 1);
        let mut timer = u16::from(
            native_record(0x82c165, game)
                .byte(2 * usize::from(word!(game.pausemenu_reserve_tank_animation_frame))),
        );
        if timer == 255 {
            word!(game.pausemenu_reserve_tank_animation_frame = 0);
            timer = u16::from(native_record(0x82c165, game).byte(0));
        }
        word!(game.pausemenu_reserve_tank_animation_timer = timer);
    }
    crate::rtl::rom_table_word!(game, 0x82c0fa, 3)
}

fn reserve_sprite(sprite: u16, index: usize, palette: u16, game: &mut Game) {
    const X: [u16; 6] = [0x18, 0x20, 0x28, 0x30, 0x38, 0x40];
    sprite_output::menu(sprite, X[index], 0x5f, palette, game);
}

pub(crate) fn reserve_tanks(game: &mut Game) {
    let palette = reserve_palette(game);
    if word!(game.samus_max_reserve_health) == 0 {
        return;
    }
    let maximum = word!(game.samus_max_reserve_health) / 100;
    let remainder = word!(game.samus_reserve_health) % 100;
    let full = word!(game.samus_reserve_health) / 100;
    for index in 0..usize::from(full) {
        reserve_sprite(0x1b, index, palette, game);
    }
    let mut occupied = full;
    let mut index = usize::from(full);
    if remainder != 0 {
        let mut frame = 2 * (remainder / 14);
        if (i32::from(frame) - 7) & 0x8000 != 0 && remainder % 14 != 0 && game.g_ram[0x5b5] & 4 == 0
        {
            frame = frame.wrapping_add(2);
        }
        if (i32::from(word!(game.samus_reserve_health)) - 100) & 0x8000 == 0 {
            frame = frame.wrapping_add(16);
        }
        const PARTIAL: [u16; 16] = [
            0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25,
            0x26, 0x27,
        ];
        reserve_sprite(PARTIAL[usize::from(frame >> 1)], index, palette, game);
        occupied = occupied.wrapping_add(1);
        index += 1;
    }
    while (i32::from(occupied) - i32::from(maximum)) & 0x8000 != 0 {
        reserve_sprite(0x20, index, palette, game);
        index += 1;
        occupied = occupied.wrapping_add(1);
    }
    reserve_sprite(0x1f, index, palette, game);
    reserve_digits(full, remainder, game);
}

pub(crate) fn pause_labels(mode: u16, game: &mut Game) {
    // First tile and width of each two-row label in the pause tilemap.
    const LEFT: (usize, usize) = (805, 5);
    const START: (usize, usize) = (812, 4);
    const RIGHT: (usize, usize) = (822, 5);
    const TILEMAP: usize = 0x3000;

    // Modes retain the original label order, including the unpause mode (1).
    let labels = [
        [(RIGHT, 0x0800), (START, 0x0800), (LEFT, 0x1400)],
        [(START, 0x0800), (LEFT, 0x1400), (RIGHT, 0x1400)],
        [(LEFT, 0x0800), (START, 0x0800), (RIGHT, 0x1400)],
    ][usize::from(mode)];
    for ((first_tile, width), palette) in labels {
        for row in [first_tile, first_tile + 32] {
            for tile in row..row + width {
                let address = TILEMAP + 2 * tile;
                let word = u16::from_le_bytes([game.g_ram[address], game.g_ram[address + 1]]);
                let word = (word & !0x1c00) | palette;
                game.g_ram[address..address + 2].copy_from_slice(&word.to_le_bytes());
            }
        }
    }
}
