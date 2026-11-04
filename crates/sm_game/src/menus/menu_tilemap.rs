//! Menu tilemaps: native word stores, live palette reads and ROM text controls.
use super::prelude::*;
use crate::display::bg_tile;
use crate::enemy_projectiles::table::with_file_select;
use crate::menu_state::MenuState;
use crate::menu_state::SaveSelection;
use crate::menu_window_state::ArrowAction;

/// The pause menu's map and equipment BG tilemap buffers in work RAM.
const PAUSE_TILEMAP: usize = 0x3000;
const EQUIPMENT_TILEMAP: usize = 0x3800;
/// The game options menu draws into the pause map's buffer.
const OPTIONS_TILEMAP: usize = PAUSE_TILEMAP;
/// The menu tilemap, $7E:3600..3DFF, that the file select, game over and
/// area select screens draw and upload to BG1.
pub const MENU_TILEMAP: usize = 0x3600;
pub const MENU_TILEMAP_BYTES: u16 = 0x800;
/// Its long address, the upload's source.
pub const MENU_TILEMAP_SOURCE: u32 = 0x7e_0000 | MENU_TILEMAP as u32;
/// The tile a cleared menu tilemap holds.
pub const BLANK_TILE: u16 = 15;
/// Bytes per 32-tile tilemap row.
const ROW: usize = 64;
/// The byte offset of tile (`row`, `column`) in a 32-tile-wide tilemap.
const fn cell(row: usize, column: usize) -> usize {
    ROW * row + 2 * column
}
/// $7E:4154: where the area select map draws the area's label.
const AREA_LABEL: usize = 0x4154;
use super::save_slots;
use crate::constants::bank_81::{
    ADDR_K_MENU_TILEMAP_DATA_CLEAR, ADDR_K_MENU_TILEMAP_DATA_COPY, ADDR_K_MENU_TILEMAP_EXIT,
    ADDR_K_MENU_TILEMAP_SAMUS_A, ADDR_K_MENU_TILEMAP_SAMUS_B, ADDR_K_MENU_TILEMAP_SAMUS_C,
    ADDR_K_MENU_TILEMAP_SAMUS_DATA, ADDR_K_MENU_TILEMAP_TIME,
};
use crate::imported::Imported;

/// Bank $82 equipment screen tilemap offsets for each category, and the
/// blank icon tilemap.
const WEAPON_OFFSETS: u32 = crate::constants::bank_82::ADDR_K_EQUIPMENT_TILEMAP_OFFS_WEAPONS;
const SUIT_OFFSETS: u32 = crate::constants::bank_82::ADDR_K_EQUIPMENT_TILEMAP_OFFS_SUITS;
const BOOT_OFFSETS: u32 = crate::constants::bank_82::ADDR_K_EQUIPMENT_TILEMAP_OFFS_BOOTS;
const BLANK_ICON_TILEMAP: u16 =
    crate::constants::bank_82::ADDR_K_EQUIPMENT_SCREEN_TILEMAP_BLANK as u16;
const _: () = assert!(WEAPON_OFFSETS == 0xc06c && SUIT_OFFSETS == 0xc076 && BOOT_OFFSETS == 0xc082);
const _: () = assert!(BLANK_ICON_TILEMAP == 0xc01a);
/// Bank $82 equipment bitmasks and pointers to equipment tilemaps, by category.
const WEAPON_MASKS: u32 = 0x82_c04c;
const SUIT_MASKS: u32 = 0x82_c056;
const BOOT_MASKS: u32 = 0x82_c062;
const WEAPON_TILEMAPS: u32 = 0x82_c08c;
const SUIT_TILEMAPS: u32 = 0x82_c096;
const BOOT_TILEMAPS: u32 = 0x82_c0a2;
/// $82:C165: the reserve tank animation, (timer, unused) byte pairs until FFh.
const RESERVE_TANK_ANIMATION: u32 = 0x82_c165;
/// $82:C639: the dummy Samus wireframe tilemap, and where the frame draws
/// it and the equipped wireframe.
const DUMMY_WIREFRAME: u16 = crate::constants::bank_82::ADDR_K_DUMMY_SAMUS_WIREFRAME_TILEMAP as u16;
const _: () = assert!(DUMMY_WIREFRAME == 0xc639);
const WIREFRAME: usize = cell(7, 12);
/// Bank $81 menu text tilemaps: ENERGY, the clock's colon and NO DATA.
const ENERGY_TEXT: u16 = 0xb496;
const COLON_TEXT: u16 = 0xb4a8;
const NO_DATA_TEXT: u16 = 0xb4ac;
/// $81:A4E6: the area select map foreground palette table, (palette data
/// index, palette RAM index) pairs until FFFFh.
const AREA_FOREGROUND_PALETTES: u32 = 0x81_a4e6;
/// The menu graphics: $8E:8000 menu BG1/2 tiles, $B6:8000 pause menu BG1/2
/// tiles, $B6:C000 menu and pause menu sprite tiles, and $8E:D600 area select
/// map BG3 tiles.
const MENU_BG_TILES: u32 = 0x8e_8000;
const PAUSE_BG_TILES: u32 = 0xb6_8000;
const MENU_SPRITE_TILES: u32 = 0xb6_c000;
const AREA_SELECT_BG3_TILES: u32 = 0x8e_d600;
/// $8E:E400: the menu palettes.
const MENU_PALETTES: u32 = 0x8e_e400;
use super::options_objects::objects::{self as bindings, Array};
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

const FILE_LABELS: [u16; 3] = [
    ADDR_K_MENU_TILEMAP_SAMUS_A as u16,
    ADDR_K_MENU_TILEMAP_SAMUS_B as u16,
    ADDR_K_MENU_TILEMAP_SAMUS_C as u16,
];

pub fn set_palette<H: Host + ?Sized>(value: u16, game: &mut H) {
    game.set_file_palette(value);
}

pub enum FileRowEmphasis {
    Occupied,
    Destination,
    Confirmation,
}

pub fn file_rows<H: Host + ?Sized>(emphasis: FileRowEmphasis, game: &mut H) {
    // Loading a slot leaves the nonempty-slot bits as they were.
    let nonempty = crate::menus::state::load::<SaveSelection, _>(game).nonempty_slots;
    for slot in 0..3u16 {
        save_slots::load(slot, game);
        let dim = match emphasis {
            FileRowEmphasis::Occupied => nonempty & (1 << slot) == 0,
            FileRowEmphasis::Destination => {
                with_file_select(game, |menu| *menu.file_copy_source_slot() == slot)
            }
            FileRowEmphasis::Confirmation => with_file_select(game, |menu| {
                *menu.file_copy_source_slot() != slot && *menu.file_copy_destination_slot() != slot
            }),
        };
        set_palette(if dim { 1024 } else { 0 }, game);
        let mask = 1 << slot;
        let base = (slot + 2) << 8;
        health(!nonempty & mask, base + 0x18, game);
        time(!nonempty & mask, base + 0x72, game);
        load(base + 0x34, ADDR_K_MENU_TILEMAP_TIME as u16, game);
        load(base + 8, FILE_LABELS[usize::from(slot)], game);
    }
    upload(
        MENU_TILEMAP_BYTES,
        MENU_TILEMAP_SOURCE,
        crate::display::preparation::Field::Screens0,
        true,
        game,
    );
}

/// Requires the native file-menu definitions for its text and tile reads.
pub fn main_menu<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    if !super::resuming(game) {
        draw_main_menu(game);
    }
    if super::blank_and_wait(false, game) {
        return;
    }
    menu.menu_index = menu.menu_index.wrapping_add(1);
    with_file_select(game, |menu| {
        *menu.file_copy_selection() = 0;
        *menu.file_copy_source_slot() = 0;
        *menu.file_copy_destination_slot() = 0;
    });
}

/// $81:9EF3 up to the clear-forced-blank wait: the file rows and the cursor.
fn draw_main_menu<H: Host + ?Sized>(game: &mut H) {
    clear(game);
    let mut nonempty = u16::MAX;
    set_palette(0, game);
    load(0x56, ADDR_K_MENU_TILEMAP_SAMUS_DATA as u16, game);
    for (slot, label) in FILE_LABELS.into_iter().enumerate() {
        // The first label follows the heading without resetting its live palette.
        if slot != 0 {
            set_palette(0, game);
        }
        let top = 0x146 + 0x140 * slot as u16;
        load(top, label, game);
        let empty = save_slots::load(slot as u16, game);
        nonempty = (nonempty >> 1) | (u16::from(empty) << 15);
        health(nonempty & 0x8000, top + 0x16, game);
        time(nonempty & 0x8000, top + 0x6e, game);
        load(top + 0x30, ADDR_K_MENU_TILEMAP_TIME as u16, game);
    }
    let nonempty = (!nonempty).swap_bytes() >> 5;
    crate::menus::state::with(game, |_, selection: &mut SaveSelection| {
        selection.nonempty_slots = nonempty;
    });
    if nonempty != 0 {
        set_palette(0, game);
        load(0x508, ADDR_K_MENU_TILEMAP_DATA_COPY as u16, game);
        set_palette(0, game);
        load(0x5c8, ADDR_K_MENU_TILEMAP_DATA_CLEAR as u16, game);
    }
    set_palette(0, game);
    load(0x688, ADDR_K_MENU_TILEMAP_EXIT as u16, game);
    upload(
        MENU_TILEMAP_BYTES,
        MENU_TILEMAP_SOURCE,
        crate::display::preparation::Field::Screens0,
        true,
        game,
    );
    // The copy arrow's palette timer is outside the Session-owned table.
    game.file_arrow(ArrowAction::Reset(0));
    with_file_select(game, |menu| {
        *menu.missile_animation_timer() = 1;
        *menu.missile_animation_frame() = 0;
        *menu.missile_x() = 0;
        *menu.missile_y() = 0;
        menu.clear_unused_words();
        for (helmet, y) in HELMET_ROWS.into_iter().enumerate() {
            *menu.helmet_animation_timer(helmet) = 0;
            *menu.helmet_animation_frame(helmet) = 0;
            *menu.helmet_x(helmet) = HELMET_X;
            *menu.helmet_y(helmet) = y;
        }
    });
    crate::screen_fade::store(game, 1, 1);
}

/// The slot helmets' x, and their y beside each save slot's row.
const HELMET_X: u16 = 100;
const HELMET_ROWS: [u16; 3] = [47, 87, 127];

/// Requires a live, exclusively accessed SNES backend.
pub fn prepare_graphics<H: Host + ?Sized>(game: &mut H) {
    use crate::display::preparation::{Field, Store};
    for (register, value) in [
        (Field::SubLayers, 0),
        (Field::ObjectLayout, 3),
        (Field::Screens0, 81),
        (Field::Screens1, 88),
        (Field::Screens2, 92),
        (Field::Characters0, 0),
        (Field::Characters1, 4),
        (Field::MainLayers, 19),
    ] {
        game.store_display_control_byte(register, value, Store::Owned);
    }
    for (source, destination, bytes) in [
        (MENU_BG_TILES, 0, 0x5600),
        (PAUSE_BG_TILES, 0x3000, 0x2000),
        (MENU_SPRITE_TILES, 0x6000, 0x2000),
        (AREA_SELECT_BG3_TILES, 0x4000, 0x600),
    ] {
        game.load_vram_words(source, destination, bytes);
    }
}

pub fn load_palette<H: Host + ?Sized>(game: &mut H) {
    let source = crate::word_transfers::WordEndpoint::Rom(
        game.loaded_rom_image(),
        sm_data::cartridge::lorom_offset(MENU_PALETTES),
    );
    for index in 0..128 {
        for lane in 0..2 {
            let word = 2 * index + lane;
            let color = source.read(isize::try_from(2 * word).unwrap());
            game.palette_write_word(false, 2 * word, color);
        }
    }
}

pub fn upload<H: Host + ?Sized>(
    size: u16,
    source: u32,
    screen: crate::display::preparation::Field,
    bank_word: bool,
    game: &mut H,
) {
    let tail = game.queue_tail();
    let entry = usize::from(tail);
    game.queue_native_slice_at(entry, size);
    let bytes = source.to_le_bytes();
    game.queue_native_slice_at(entry + 2, u16::from_le_bytes([bytes[0], bytes[1]]));
    if bank_word {
        game.queue_native_slice_at(entry + 4, u16::from(bytes[2]));
    } else {
        game.queue_byte_at(entry + 4, bytes[2]);
    }
    // The screen register mirror (BGnSC) names the tilemap's VRAM base.
    let screen = game.display_control_byte(screen);
    let destination = u16::from(screen & 0xfc) << 8;
    game.queue_native_slice_at(entry + 5, destination);
    game.store_queue_tail(tail.wrapping_add(7));
}

fn paint<H: Host + ?Sized>(offset: usize, tile: u16, game: &mut H) {
    let palette = game.file_palette();
    game.screen_apply(|workspace| super::canvas::paint_file(workspace, offset, tile, palette));
}

/// Capture checked native reads after the original low-window diagnostic.
pub fn native_record<H: crate::cartridge::NativeDiagnostics + ?Sized>(
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
    let address = crate::gameplay_tables::area_label(
        game,
        crate::gameplay_context::navigation(game).area as isize,
    );
    let label = native_record(0x820000 | u32::from(address), game);
    for index in 0..12 {
        let tile = label.word(2 * index) & 0xefff;
        let destination = AREA_LABEL + 2 * index;
        game.screen_write_slice(
            Point::new(Role::SavedBackground, destination - 0x4100),
            &tile.to_ne_bytes(),
        );
    }
}

pub fn load<H: Host + ?Sized>(destination: u16, source: u16, game: &mut H) {
    super::canvas::load_text(destination, source, &mut NativeText(game));
}

fn clear_row<H: Host + ?Sized>(offset: u16, game: &mut H) {
    game.screen_apply(|workspace| super::canvas::clear_file_row(workspace, offset));
}

fn number<H: Host + ?Sized>(offset: usize, value: u16, game: &mut H) {
    paint(offset + 2, value % 10 + 8288, game);
    paint(offset, value / 10 + 8288, game);
}

pub fn health<H: Host + ?Sized>(empty: u16, offset: u16, game: &mut H) {
    if empty != 0 {
        clear_row(offset, game);
        load(offset.wrapping_add(64), NO_DATA_TEXT, game);
        return;
    }
    load(offset, ENERGY_TEXT, game);
    let mut full = crate::gameplay_context::inventory(game).health.current / 100;
    let remainder = crate::gameplay_context::inventory(game).health.current % 100;
    let maximum = crate::gameplay_context::inventory(game).health.maximum / 100;
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
    number(
        usize::from(offset),
        crate::gameplay_word!(game.GAME_TIME_HOURS),
        game,
    );
    load(offset.wrapping_add(4), COLON_TEXT, game);
    number(
        usize::from(offset) + 6,
        crate::gameplay_word!(game.GAME_TIME_MINUTES),
        game,
    );
}

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
        if crate::cartridge::rom_table_word!(game, AREA_FOREGROUND_PALETTES, index) == 0xffff {
            break;
        }
        let destination =
            crate::cartridge::rom_table_word!(game, AREA_FOREGROUND_PALETTES, index + 1);
        let source = crate::cartridge::rom_table_word!(game, AREA_FOREGROUND_PALETTES, index);
        copy_area_colors(source, destination, game);
        entry = entry.wrapping_add(4);
    }
}

/// Five colours from the foreground palette data at $81:A40E into the buffer.
fn copy_area_colors<H: Host + ?Sized>(mut source: u16, mut destination: u16, game: &mut H) {
    for _ in 0..5 {
        let color = crate::menu_lists::area_select_colour(game, (source >> 1) as isize);
        game.palette_write_word(false, 2 * usize::from(destination >> 1), color);
        destination = destination.wrapping_add(2);
        source = source.wrapping_add(2);
    }
}

pub fn clear<H: Host + ?Sized>(game: &mut H) {
    game.screen_apply(|workspace| super::canvas::clear_file(workspace));
}

pub fn controller_bindings<H: Host + ?Sized>(game: &mut H) {
    for index in 0..7 {
        let destination = crate::menu_lists::binding_destination(game, index);
        let address = 0x820000
            | u32::from(crate::menu_lists::binding_button_tilemap(
                game,
                bindings::get_at(game, Array::ControllerBindings, index) as isize,
            ));
        if address & 0x8000 == 0 {
            game.invalid_window(address);
        }
        let source = crate::word_transfers::WordEndpoint::Rom(
            game.loaded_image().clone(),
            sm_data::cartridge::lorom_offset(address),
        );
        for (row, source_word) in [(0, 0), (ROW, 3)] {
            let source = source.shifted(source_word);
            for index in 0..3 {
                let tile = source.read(isize::try_from(2 * index).unwrap());
                game.screen_write_native_word(
                    Point::new(Role::Pause, usize::from(destination) + row + 2 * index),
                    tile,
                );
            }
        }
    }
    // The aim up and aim down rows' 3x2 hint, cleared unless bound to L or R.
    let cleared_hint = [0, 0xe, 0xe, 0x10, 0x1f, 0x1f];
    let hints = [cell(20, 23), cell(23, 23)];
    for (index, destination) in hints.into_iter().enumerate() {
        if bindings::get_at(game, Array::ControllerBindings, index + 5) != 5
            && bindings::get_at(game, Array::ControllerBindings, index + 5) != 6
        {
            for (row, tiles) in cleared_hint.as_chunks::<3>().0.iter().enumerate() {
                for (column, tile) in tiles.iter().enumerate() {
                    game.screen_write_native_word(
                        Point::new(Role::Pause, destination + row * ROW + 2 * column),
                        *tile,
                    );
                }
            }
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

pub fn update_reserve_arrow<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    let mode = if menu.pausemenu_equipment_category_item as u8 != 0 {
        ReserveArrow::Off
    } else {
        [ReserveArrow::Animated, ReserveArrow::On]
            [usize::from(menu.pausemenu_equipment_category_item.to_le_bytes()[1])]
    };
    reserve_arrow(mode, game);
}

pub fn reserve_arrow<H: Host + ?Sized>(mode: ReserveArrow, game: &mut H) {
    let lit = match mode {
        ReserveArrow::Animated => crate::gameplay_context::inventory(game).reserve_mode == 1,
        ReserveArrow::On => true,
        ReserveArrow::Off => false,
    };
    if matches!(mode, ReserveArrow::Animated) && lit {
        let index = usize::from(
            game.display_control_byte(crate::display::preparation::Field::NmiFrameByte) & 0x1f,
        );
        game.palette_write_word(false, 204, RESERVE_ARROW_COLORS[index]);
        game.palette_write_word(false, 214, RESERVE_ARROW_SHADOWS[index]);
    } else {
        // The idle arrow holds the cycle's first entries, in the opposite slots.
        game.palette_write_word(false, 214, RESERVE_ARROW_COLORS[0]);
        game.palette_write_word(false, 204, RESERVE_ARROW_SHADOWS[0]);
    }
    reserve_arrow_tiles(lit, game);
}

pub fn reserve_arrow_tiles<H: Host + ?Sized>(lit: bool, game: &mut H) {
    let palette = if lit {
        bg_tile::palette(6)
    } else {
        bg_tile::palette(7)
    };
    for index in (0..8).map(|row| 129 + row * 32).chain(385..387) {
        recolor_tiles(Point::new(Role::Equipment, 2 * index), palette, 1, game);
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
        game.screen_write_byte(table_point(destination + index), byte);
    }
}

/// The screen point a bank $82 table's work RAM destination names: the pause
/// map's or the equipment screen's buffer at its offset while it lies in the
/// workspace ($7E:3000..$6FFE), else the original's alias at that address.
fn table_point(destination: usize) -> Point {
    match destination {
        PAUSE_TILEMAP..EQUIPMENT_TILEMAP => Point::new(Role::Pause, destination - PAUSE_TILEMAP),
        EQUIPMENT_TILEMAP..0x6fff => Point::new(Role::Equipment, destination - EQUIPMENT_TILEMAP),
        // Any other destination keeps the original's work RAM alias.
        _ => Point::physical(destination),
    }
}

/// Recolor `words` tiles from a bank $82 table's work RAM destination.
pub fn recolor_native<H: Host + ?Sized>(
    destination: usize,
    palette: u16,
    words: usize,
    game: &mut H,
) {
    for index in 0..words {
        recolor_tile(table_point(destination + 2 * index), palette, game);
    }
}

/// Recolor `words` tiles of a workspace buffer from `start`.
pub fn recolor_tiles<H: Host + ?Sized>(start: Point, palette: u16, words: usize, game: &mut H) {
    for index in 0..words {
        recolor_tile(
            start.advance(2 * index).expect("screen offset"),
            palette,
            game,
        );
    }
}

fn recolor_tile<H: Host + ?Sized>(point: Point, palette: u16, game: &mut H) {
    // Publish each tile before selecting the next one, including failure prefixes.
    if point.physical_address().is_none() {
        game.screen_apply(|workspace| super::canvas::recolor(workspace, point, palette));
    } else {
        // Native ROM destinations can select retained physical aliases.
        let tile = game.screen_native_word(point);
        game.screen_write_native_word(point, bg_tile::recolour(tile, palette));
    }
}

pub fn equipment_icons<H: Host + ?Sized>(game: &mut H) {
    if crate::gameplay_context::inventory(game).reserve_capacity != 0 {
        for index in 0..2 {
            let destination = crate::menu_lists::reserve_tank_destination(game, index) as usize;
            let source = crate::menu_lists::reserve_tank_tilemap(game, index);
            copy_native_tiles(destination, source, 14, game);
        }
    }
    for (offsets, count, masks, tiles) in [
        (WEAPON_OFFSETS, 6, WEAPON_MASKS, WEAPON_TILEMAPS),
        (SUIT_OFFSETS, 6, SUIT_MASKS, SUIT_TILEMAPS),
        (BOOT_OFFSETS, 3, BOOT_MASKS, BOOT_TILEMAPS),
    ] {
        let destinations = native_record(0x820000 | offsets, game);
        let weapons = offsets == WEAPON_OFFSETS;
        let hyper = weapons && crate::gameplay_context::inventory(game).hyper_beam != 0;
        let (collected, equipped, bytes) = if weapons {
            (
                crate::inventory::Word::CollectedBeams,
                crate::inventory::Word::EquippedBeams,
                10,
            )
        } else {
            (
                crate::inventory::Word::CollectedItems,
                crate::inventory::Word::EquippedItems,
                18,
            )
        };
        for index in 0..if hyper { 5 } else { count } {
            let destination = destinations.word(2 * index) as usize;
            if hyper {
                let source = crate::menu_lists::hyper_beam_tilemap(game, index as isize);
                copy_native_tiles(destination, source, 10, game);
            } else if crate::gameplay_context::inventory(game).get(collected)
                & crate::menu_lists::equipment_bit(game, masks, index as isize)
                != 0
            {
                let source = crate::menu_lists::equipment_tilemap(game, tiles, index as isize);
                copy_native_tiles(destination, source, bytes, game);
                if crate::gameplay_context::inventory(game).get(equipped)
                    & crate::menu_lists::equipment_bit(game, masks, index as isize)
                    == 0
                {
                    recolor_native(destination, bg_tile::palette(3), bytes / 2, game);
                }
            } else {
                copy_native_tiles(destination, BLANK_ICON_TILEMAP, bytes, game);
            }
        }
    }
}

fn wireframe_rows<H: Host + ?Sized>(
    destination: Point,
    row_stride: usize,
    mut source: impl FnMut(&mut H) -> u16,
    game: &mut H,
) {
    for row in 0..17 {
        for column in 0..8 {
            let tile = source(game);
            game.screen_write_native_word(
                destination
                    .advance(row_stride * row + 2 * column)
                    .expect("screen offset"),
                tile,
            );
        }
    }
}

pub fn prepare_equipment_frame<H: Host + ?Sized>(game: &mut H) {
    let mut cursor = DUMMY_WIREFRAME;
    wireframe_rows(
        Point::new(Role::Pause, WIREFRAME),
        32,
        |game| {
            let tile = native_record(0x820000 | u32::from(cursor), game).word(0);
            cursor = cursor.wrapping_add(2);
            tile
        },
        game,
    );
    equipment_wireframe(game);
    if crate::gameplay_context::inventory(game).reserve_capacity != 0 {
        let hundreds = crate::gameplay_context::inventory(game).reserve_health / 100;
        let remainder = crate::gameplay_context::inventory(game).reserve_health % 100;
        reserve_digits(hundreds, remainder, game);
    }
}

pub fn equipment_wireframe<H: Host + ?Sized>(game: &mut H) {
    let mut cursor = 0u16;
    while crate::gameplay_context::inventory(game).equipment.active & 0x101
        != crate::menu_lists::wireframe_suit(game, (cursor >> 1) as isize)
    {
        cursor = cursor.wrapping_add(2);
    }
    let address = crate::menu_lists::wireframe_tilemap(game, (cursor >> 1) as isize);
    let record = native_record(0x820000 | u32::from(address), game);
    let mut offset = 0;
    wireframe_rows(
        Point::new(Role::Equipment, WIREFRAME),
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
        let point = Point::new(Role::Equipment, 2 * (327 + index));
        let previous = game.screen_native_word(point);
        let value = (previous & !0x3ff) | tile(index);
        game.screen_write_native_word(point, value);
    }
}

fn reserve_digits<H: Host + ?Sized>(hundreds: u16, remainder: u16, game: &mut H) {
    game.screen_apply(|workspace| super::canvas::reserve_digits(workspace, hundreds, remainder));
}

pub fn queue_equipment_wireframe<H: Host + ?Sized>(game: &mut H) {
    equipment_wireframe(game);
    crate::display_refresh::queue_native_transfer(
        game.queue_tail(),
        1280,
        |_| (EQUIPMENT_TILEMAP + cell(4, 0)) as u16,
        126,
        |_| 0x3080,
        game,
    );
}

pub fn queue_pause_labels<H: Host + ?Sized>(game: &mut H) {
    crate::display_refresh::queue_native_transfer(
        game.queue_tail(),
        128,
        |_| (PAUSE_TILEMAP + cell(25, 0)) as u16,
        126,
        |game| {
            (u16::from(
                game.display_control_byte(crate::display::preparation::Field::Screens1) & 0xfc,
            ) << 8)
                + 800
        },
        game,
    );
}

pub fn queue_options<H: Host + ?Sized>(game: &mut H) {
    crate::display_refresh::queue_native_transfer(
        game.queue_tail(),
        2048,
        |_| OPTIONS_TILEMAP as u16,
        126,
        |_| 0x5000,
        game,
    );
}

fn reserve_palette<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H) -> u16 {
    if crate::gameplay_context::inventory(game).reserve_health == 0 {
        return 1536;
    }
    let timer = &mut menu.pausemenu_reserve_tank_animation_timer;
    *timer = timer.wrapping_sub(1);
    if *timer == 0 || *timer & 0x8000 != 0 {
        menu.pausemenu_reserve_tank_animation_frame =
            menu.pausemenu_reserve_tank_animation_frame.wrapping_add(1);
        let mut timer = u16::from(
            native_record(RESERVE_TANK_ANIMATION, game)
                .byte(2 * usize::from(menu.pausemenu_reserve_tank_animation_frame)),
        );
        if timer == 255 {
            menu.pausemenu_reserve_tank_animation_frame = 0;
            timer = u16::from(native_record(RESERVE_TANK_ANIMATION, game).byte(0));
        }
        menu.pausemenu_reserve_tank_animation_timer = timer;
    }
    super::map_menu::animation_palette(game, 3)
}

fn reserve_sprite<H: Host + ?Sized>(sprite: u16, index: usize, palette: u16, game: &mut H) {
    const X: [u16; 6] = [0x18, 0x20, 0x28, 0x30, 0x38, 0x40];
    crate::sprite_output::menu(sprite, X[index], 0x5f, palette, game);
}

pub fn reserve_tanks<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    let palette = reserve_palette(menu, game);
    if crate::gameplay_context::inventory(game).reserve_capacity == 0 {
        return;
    }
    let maximum = crate::gameplay_context::inventory(game).reserve_capacity / 100;
    let remainder = crate::gameplay_context::inventory(game).reserve_health % 100;
    let full = crate::gameplay_context::inventory(game).reserve_health / 100;
    for index in 0..usize::from(full) {
        reserve_sprite(0x1b, index, palette, game);
    }
    let mut occupied = full;
    let mut index = usize::from(full);
    if remainder != 0 {
        let mut frame = 2 * (remainder / 14);
        if (i32::from(frame) - 7) & 0x8000 != 0
            && !remainder.is_multiple_of(14)
            && game.display_control_byte(crate::display::preparation::Field::NmiFrameByte) & 4 == 0
        {
            frame = frame.wrapping_add(2);
        }
        if (i32::from(crate::gameplay_context::inventory(game).reserve_health) - 100) & 0x8000 == 0
        {
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
    game.screen_apply(|workspace| super::canvas::pause_labels(workspace, mode));
}

/// Captured loaded allocation, queried one word at a time after each publication.
pub fn copy_rom_words<H: Host + ?Sized>(
    first: Point,
    address: u32,
    source_word: usize,
    count: usize,
    descending: bool,
    game: &mut H,
) {
    let source = crate::word_transfers::WordEndpoint::Rom(
        game.loaded_rom_image(),
        sm_data::cartridge::lorom_offset(address),
    )
    .shifted(isize::try_from(source_word).unwrap());
    for index in 0..count {
        let displacement = isize::try_from(2 * index).unwrap();
        let displacement = if descending {
            -displacement
        } else {
            displacement
        };
        let value = source.read(displacement);
        let offset = if descending {
            first.offset - 2 * index
        } else {
            first.offset + 2 * index
        };
        game.screen_write_native_word(Point::new(first.role, offset), value);
    }
}

struct NativeText<'a, H: ?Sized>(&'a mut H);
impl<H: Host + ?Sized> super::canvas::TextComposition for NativeText<'_, H> {
    fn word(&mut self, cursor: u16) -> u16 {
        text_word(cursor, self.0)
    }
    fn paint(&mut self, offset: usize, tile: u16) {
        paint(offset, tile, self.0);
    }
}
