//! Menu tilemaps: native word stores, live palette reads and ROM text controls.
use super::prelude::*;
use super::save_slots;
use crate::constants::bank_81::{
    addr_kMenuTilemap_DataClear, addr_kMenuTilemap_DataCopy, addr_kMenuTilemap_Exit,
    addr_kMenuTilemap_SamusA, addr_kMenuTilemap_SamusB, addr_kMenuTilemap_SamusC,
    addr_kMenuTilemap_SamusData, addr_kMenuTilemap_TIME,
};
use crate::imported::Imported;
use crate::ram::{checked_word as at, word, words as fields};
use sm_data::area_select_palettes::{
    ADDRESS as AREA_PALETTES, AreaSelectPalettes, BYTES as AREA_PALETTE_BYTES, INACTIVE_OFFSET,
    RUN_OFFSETS,
};
use sm_data::cartridge::RomImage;

pub type ImportedAreaPalettes = Imported<AreaSelectPalettes>;

pub fn import_area_palettes(image: &RomImage) -> Option<ImportedAreaPalettes> {
    Imported::import(
        image,
        AREA_PALETTES,
        AREA_PALETTE_BYTES,
        AreaSelectPalettes::import,
    )
}

const MENU_PALETTE: usize = 0xf78 + enemy_data::PALETTE_INDEX;
const FILE_LABELS: [u16; 3] = [
    addr_kMenuTilemap_SamusA as u16,
    addr_kMenuTilemap_SamusB as u16,
    addr_kMenuTilemap_SamusC as u16,
];

pub fn set_palette<H: Host + ?Sized>(value: u16, game: &mut H) {
    game.ram().copy_in(MENU_PALETTE, &value.to_ne_bytes());
}

pub enum FileRowEmphasis {
    Occupied,
    Destination,
    Confirmation,
}

pub fn file_rows<H: Host + ?Sized>(emphasis: FileRowEmphasis, game: &mut H) {
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
pub fn main_menu<H: Host + ?Sized>(game: &mut H) {
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
    game.set_blank(false);
    word!(game.menu_index += 1);
    word! { game, 0; eproj_id[15], eproj_id[16], eproj_id[17] }
}

/// Requires a live, exclusively accessed SNES backend.
pub fn prepare_graphics<H: Host + ?Sized>(game: &mut H) {
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
        game.ram().write_byte(register, value);
    }
    for (source, destination, bytes) in [
        (0x8e8000, 0, 0x5600),
        (0xb68000, 0x3000, 0x2000),
        (0xb6c000, 0x6000, 0x2000),
        (0x8ed600, 0x4000, 0x600),
    ] {
        game.load_vram_words(source, destination, bytes);
    }
}

pub fn load_palette<H: Host + ?Sized>(game: &mut H) {
    crate::word_transfers!(game, (4), 128;
        le(palette_buffer, 0) <- rom(0x8ee400, 0),
        le(palette_buffer, 1) <- rom(0x8ee400, 1),
    );
}

pub fn upload<H: Host + ?Sized>(
    size: u16,
    source: u32,
    screen: usize,
    bank_word: bool,
    game: &mut H,
) {
    let tail = word!(game.vram_write_queue_tail);
    let entry = 0xd0 + usize::from(tail);
    game.ram().copy_in(entry, &size.to_ne_bytes());
    game.ram()
        .copy_in(entry + 2, &(source as u16).to_ne_bytes());
    let bank = (source >> 16) as u8;
    if bank_word {
        game.ram()
            .copy_in(entry + 4, &u16::from(bank).to_ne_bytes());
    } else {
        game.ram().write_byte(entry + 4, bank);
    }
    let destination = u16::from(game.ram().read_byte(0x58 + screen) & 0xfc) << 8;
    game.ram().copy_in(entry + 5, &destination.to_ne_bytes());
    word!(game.vram_write_queue_tail = tail.wrapping_add(7));
}

fn paint<H: Host + ?Sized>(offset: usize, tile: u16, game: &mut H) {
    let palette = u16::from_ne_bytes([
        game.ram().read_byte(MENU_PALETTE),
        game.ram().read_byte(MENU_PALETTE + 1),
    ]);
    let at = 0x3600 + offset;
    game.ram().copy_in(at, &(palette | tile).to_ne_bytes());
}

/// Capture checked native reads after the original low-window diagnostic.
pub fn native_record<H: Host + ?Sized>(
    address: u32,
    game: &mut H,
) -> sm_data::cartridge::RomRecord {
    if address & 0x8000 == 0 {
        game.native_rom_diagnostic(address);
    }
    sm_data::cartridge::RomRecord {
        image: game.native_image(),
        base: sm_data::cartridge::lorom_offset(address) as usize,
    }
}

fn text_word<H: Host + ?Sized>(address: u16, game: &mut H) -> u16 {
    // The warning can replace the native image; capture after it on every word.
    native_record(0x810000 | u32::from(address), game).word(0)
}

pub fn area_label<H: Host + ?Sized>(game: &mut H) {
    let address =
        crate::cartridge::rom_table_word!(game, 0x82965f, word!(game.area_index) as isize);
    let label = native_record(0x820000 | u32::from(address), game);
    for index in 0..12 {
        let tile = label.word(2 * index) & 0xefff;
        let destination = 0x4154 + 2 * index;
        game.ram().copy_in(destination, &tile.to_ne_bytes());
    }
}

pub fn load<H: Host + ?Sized>(mut destination: u16, mut source: u16, game: &mut H) {
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

fn clear_row<H: Host + ?Sized>(offset: u16, game: &mut H) {
    let count = 32 - usize::from((offset & 0x3f) >> 1);
    crate::word_transfers!(game, ascending, count; native(0x3600 + usize::from(offset)) <- value(15));
}

fn number<H: Host + ?Sized>(offset: usize, value: u16, game: &mut H) {
    paint(offset + 2, value % 10 + 8288, game);
    paint(offset, value / 10 + 8288, game);
}

pub fn health<H: Host + ?Sized>(empty: u16, offset: u16, game: &mut H) {
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

pub fn time<H: Host + ?Sized>(empty: u16, offset: u16, game: &mut H) {
    if empty != 0 {
        return;
    }
    number(usize::from(offset), word!(game.game_time_hours), game);
    load(offset.wrapping_add(4), 0xb4a8, game);
    number(usize::from(offset) + 6, word!(game.game_time_minutes), game);
}

#[forbid(unsafe_code)]
pub fn area_palette<H: Host + ?Sized>(area: u16, active: bool, game: &mut H) {
    let mut entry =
        RUN_OFFSETS[usize::from(area)] as u16 + if active { 0 } else { INACTIVE_OFFSET as u16 };
    // The table serves the area's run while the loaded image is the imported
    // allocation; otherwise the original indexed loads walk it.
    let rom = game.loaded_image();
    let copies = Imported::from(game.area_select_palettes(), rom)
        .and_then(|table| table.copies(usize::from(area), active))
        .map(<[_]>::to_vec);
    if let Some(copies) = copies {
        for copy in copies {
            copy_area_colors(copy.source, copy.destination, game);
        }
        return;
    }
    loop {
        let index = (entry >> 1) as isize;
        if crate::cartridge::rom_table_word!(game, 0x81a4e6, index) == 0xffff {
            break;
        }
        let destination = crate::cartridge::rom_table_word!(game, 0x81a4e6, index + 1);
        let source = crate::cartridge::rom_table_word!(game, 0x81a4e6, index);
        copy_area_colors(source, destination, game);
        entry = entry.wrapping_add(4);
    }
}

/// Five colours from the foreground palette data at $81:A40E into the buffer.
fn copy_area_colors<H: Host + ?Sized>(mut source: u16, mut destination: u16, game: &mut H) {
    for _ in 0..5 {
        word!(
            game.palette_buffer[destination >> 1] =
                crate::cartridge::rom_table_word!(game, 0x81a40e, (source >> 1) as isize)
        );
        destination = destination.wrapping_add(2);
        source = source.wrapping_add(2);
    }
}

pub fn clear<H: Host + ?Sized>(game: &mut H) {
    crate::word_transfers!(game, descending, 1024; native(0x3dfe) <- value(15));
}

pub fn controller_bindings<H: Host + ?Sized>(game: &mut H) {
    for index in 0..7 {
        let destination = crate::cartridge::rom_table_word!(game, 0x82f639, index);
        let address = 0x820000
            | u32::from(crate::cartridge::rom_table_word!(
                game,
                0x82f647,
                word!(game.eproj_F[index + 13]) as isize
            ));
        if address & 0x8000 == 0 {
            game.invalid_window(address);
        }
        let source = crate::word_transfers::WordEndpoint::Rom(
            game.loaded_image().clone(),
            sm_data::cartridge::lorom_offset(address),
        );
        crate::word_transfers!(game, ascending, 3;
            native(0x3000 + destination as isize) <- captured(source, 0));
        crate::word_transfers!(game, ascending, 3;
            native(0x3040 + destination as isize) <- captured(source, 3));
    }
    let cleared_hint = [0, 0xe, 0xe, 0x10, 0x1f, 0x1f];
    for (index, destination) in [0x352e, 0x35ee].into_iter().enumerate() {
        if word!(game.eproj_instr_list_ptr[index]) != 5
            && word!(game.eproj_instr_list_ptr[index]) != 6
        {
            crate::word_transfers!(game, ascending, 3;
                native(destination) <- table(cleared_hint, 0));
            crate::word_transfers!(game, ascending, 3;
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
pub enum ReserveArrow {
    Animated,
    On,
    Off,
}

pub fn update_reserve_arrow<H: Host + ?Sized>(game: &mut H) {
    let mode = if word!(game.pausemenu_equipment_category_item) as u8 != 0 {
        ReserveArrow::Off
    } else {
        [ReserveArrow::Animated, ReserveArrow::On][usize::from(game.ram().read_byte(0x756))]
    };
    reserve_arrow(mode, game);
}

pub fn reserve_arrow<H: Host + ?Sized>(mode: ReserveArrow, game: &mut H) {
    let lit = match mode {
        ReserveArrow::Animated => word!(game.reserve_health_mode) == 1,
        ReserveArrow::On => true,
        ReserveArrow::Off => false,
    };
    if matches!(mode, ReserveArrow::Animated) && lit {
        let index = usize::from(game.ram().read_byte(0x5b5) & 0x1f);
        word!(game.palette_buffer[102] = RESERVE_ARROW_COLORS[index]);
        word!(game.palette_buffer[107] = RESERVE_ARROW_SHADOWS[index]);
    } else {
        word! {game; palette_buffer[107] = 926; palette_buffer[102] = 342; }
    }
    reserve_arrow_tiles(lit, game);
}

pub fn reserve_arrow_tiles<H: Host + ?Sized>(lit: bool, game: &mut H) {
    let palette = if lit { 0x1800 } else { 0x1c00 };
    for index in (0..8).map(|row| 129 + row * 32).chain(385..387) {
        recolor_native(0x3800 + 2 * index, palette, 1, game);
    }
}

pub fn copy_native_tiles<H: Host + ?Sized>(
    destination: usize,
    cursor: u16,
    bytes: usize,
    game: &mut H,
) {
    let source = native_record(0x820000 | u32::from(cursor), game);
    for index in 0..bytes {
        let byte = source.byte(index);
        game.ram().write_byte(destination + index, byte);
    }
}

pub fn recolor_native<H: Host + ?Sized>(
    destination: usize,
    palette: u16,
    words: usize,
    game: &mut H,
) {
    for index in 0..words {
        let address = destination + 2 * index;
        let tile = u16::from_ne_bytes([
            game.ram().read_byte(address),
            game.ram().read_byte(address + 1),
        ]);
        crate::word_transfers!(game, ascending, 1;
            native(address) <- value((tile & !0x1c00) | palette));
    }
}

pub fn equipment_icons<H: Host + ?Sized>(game: &mut H) {
    if word!(game.samus_max_reserve_health) != 0 {
        for index in 0..2 {
            let destination = crate::cartridge::rom_table_word!(game, 0x82c068, index) as usize;
            let source = crate::cartridge::rom_table_word!(game, 0x82c088, index);
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
                let source = crate::cartridge::rom_table_word!(game, 0x82c0a8, index as isize);
                copy_native_tiles(destination, source, 10, game);
            } else if at!(game, collected;)
                & crate::cartridge::rom_table_word!(game, masks, index as isize)
                != 0
            {
                let source = crate::cartridge::rom_table_word!(game, tiles, index as isize);
                copy_native_tiles(destination, source, bytes, game);
                if at!(game, equipped;)
                    & crate::cartridge::rom_table_word!(game, masks, index as isize)
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

fn wireframe_rows<H: Host + ?Sized>(
    destination: usize,
    row_stride: usize,
    mut source: impl FnMut(&mut H) -> u16,
    game: &mut H,
) {
    for row in 0..17 {
        for column in 0..8 {
            let tile = source(game);
            crate::word_transfers!(game, ascending, 1;
                native(destination + row_stride * row + 2 * column) <- value(tile));
        }
    }
}

pub fn prepare_equipment_frame<H: Host + ?Sized>(game: &mut H) {
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

pub fn equipment_wireframe<H: Host + ?Sized>(game: &mut H) {
    let mut cursor = 0u16;
    while word!(game.equipped_items) & 0x101
        != crate::cartridge::rom_table_word!(game, 0x82b257, (cursor >> 1) as isize)
    {
        cursor = cursor.wrapping_add(2);
    }
    let address = crate::cartridge::rom_table_word!(game, 0x82b25f, (cursor >> 1) as isize);
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

pub fn reserve_label<H: Host + ?Sized>(tile: impl Fn(usize) -> u16, game: &mut H) {
    for index in 0..4 {
        let address = 0x3800 + 2 * (327 + index);
        let previous = u16::from_ne_bytes([
            game.ram().read_byte(address),
            game.ram().read_byte(address + 1),
        ]);
        crate::word_transfers!(game, ascending, 1;
            native(address) <- value((previous & 0xfc00) | tile(index)));
    }
}

fn reserve_digits<H: Host + ?Sized>(hundreds: u16, remainder: u16, game: &mut H) {
    for (index, digit) in [
        (394, remainder % 10),
        (393, remainder / 10),
        (392, hundreds),
    ] {
        crate::word_transfers!(game, ascending, 1;
                native(0x3800 + 2 * index) <- value(digit + 2052));
    }
}

pub fn queue_equipment_wireframe<H: Host + ?Sized>(game: &mut H) {
    equipment_wireframe(game);
    game.queue_native_transfer(
        word!(game.vram_write_queue_tail),
        1280,
        &|_| 0x3900,
        126,
        &|_| 0x3080,
    );
}

pub fn queue_pause_labels<H: Host + ?Sized>(game: &mut H) {
    game.queue_native_transfer(
        word!(game.vram_write_queue_tail),
        128,
        &|_| 0x3640,
        126,
        &|game| (u16::from(game.ram().read_byte(0x59) & 0xfc) << 8) + 800,
    );
}

pub fn queue_options<H: Host + ?Sized>(game: &mut H) {
    game.queue_native_transfer(
        word!(game.vram_write_queue_tail),
        2048,
        &|_| 0x3000,
        126,
        &|_| 0x5000,
    );
}

fn reserve_palette<H: Host + ?Sized>(game: &mut H) -> u16 {
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
    super::map_menu::animation_palette(game, 3)
}

fn reserve_sprite<H: Host + ?Sized>(sprite: u16, index: usize, palette: u16, game: &mut H) {
    const X: [u16; 6] = [0x18, 0x20, 0x28, 0x30, 0x38, 0x40];
    game.menu_sprite(sprite, X[index], 0x5f, palette);
}

pub fn reserve_tanks<H: Host + ?Sized>(game: &mut H) {
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
        if (i32::from(frame) - 7) & 0x8000 != 0
            && remainder % 14 != 0
            && game.ram().read_byte(0x5b5) & 4 == 0
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

pub fn pause_labels<H: Host + ?Sized>(mode: u16, game: &mut H) {
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
                let word = u16::from_le_bytes([
                    game.ram().read_byte(address),
                    game.ram().read_byte(address + 1),
                ]);
                let word = (word & !0x1c00) | palette;
                game.ram().copy_in(address, &word.to_le_bytes());
            }
        }
    }
}
