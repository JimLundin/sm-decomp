//! Area and room selection, with the original menu-state and RAM compatibility.
use crate::game::constants::{bank_81::*, bank_82::*};
use crate::game::imported::{Imported, grid_word, word as table_word};
use crate::game::prelude::*;
use crate::game::projectile_presentation_ram as sprite_output;
use crate::game::room_entry::{self, ArrivalRequest, StartSite};
use crate::game::save_slots::load_area_map;
use crate::game::session_runtime::Runtime;
use crate::game::visual_runtime::{clear_hdma, set_hdma_enabled};
use crate::game::{menu_tilemap, menu_window};
use sm_data::pause_menu::{
    ANIMATION_PALETTE_ADDRESS, ANIMATION_SPRITEMAP_ADDRESS, ANIMATIONS, AREA_MAP_ADDRESS,
    AREA_MAPS, LABEL_SPRITEMAP_ADDRESS, LABEL_SPRITEMAPS, MAP_ICON_ADDRESS, MAP_ICON_AREAS,
    PALETTE_ANIMATION_ADDRESS, PALETTE_ANIMATION_BYTES, PaletteAnimation, SELECTOR_CATEGORIES,
    SELECTOR_POSITION_ADDRESS, import_palette_animation, import_words,
};
use sm_data::table::Rows;
use sm_game::display::control::{Layers, MainScreen, MainWindowMask, SubScreen, SubWindowMask};
use sm_game::effects::{DisplayRefresh as _, SoundEffects as _};

#[derive(Default)]
pub(crate) struct ImportedTables {
    label_spritemaps: Option<Imported<Rows<u16, LABEL_SPRITEMAPS>>>,
    map_icons: Option<Imported<Rows<u16, MAP_ICON_AREAS>>>,
    area_maps: Option<Imported<Rows<u16, AREA_MAPS>>>,
    animation_palettes: Option<Imported<Rows<u16, ANIMATIONS>>>,
    animation_spritemaps: Option<Imported<Rows<u16, ANIMATIONS>>>,
    selector_positions: Option<Imported<Rows<u16, SELECTOR_CATEGORIES>>>,
    palette_animation: Option<Imported<PaletteAnimation>>,
}

pub(crate) fn import(image: &crate::rom::RomImage) -> ImportedTables {
    fn words<const N: usize>(
        image: &crate::rom::RomImage,
        address: u32,
    ) -> Option<Imported<Rows<u16, N>>> {
        Imported::import(image, address, 2 * N, import_words)
    }
    ImportedTables {
        label_spritemaps: words(image, LABEL_SPRITEMAP_ADDRESS),
        map_icons: words(image, MAP_ICON_ADDRESS),
        area_maps: words(image, AREA_MAP_ADDRESS),
        animation_palettes: words(image, ANIMATION_PALETTE_ADDRESS),
        animation_spritemaps: words(image, ANIMATION_SPRITEMAP_ADDRESS),
        selector_positions: words(image, SELECTOR_POSITION_ADDRESS),
        palette_animation: Imported::import(
            image,
            PALETTE_ANIMATION_ADDRESS,
            PALETTE_ANIMATION_BYTES,
            import_palette_animation,
        ),
    }
}

/// The palette bits for pause menu sprite animation `index`.
pub(crate) fn animation_palette(game: &Game, index: isize) -> u16 {
    table_word(
        &game.pause_menu.animation_palettes,
        game,
        ANIMATION_PALETTE_ADDRESS,
        index,
    )
}
const AREAS: [u16; 6] = [0, 3, 5, 1, 4, 2];

static STATES: [Func_V; 23] = callback_table! { nonnull Func_V;
    prepare, fade_palette, load_foreground,
    begin_background,
    begin_area_window, open_area_window,
    choose_area, begin_room_window,
    open_room_window, initialize_room,
    choose_room, show_room, show_room, fade_room,
    start_game, clear_room, restore_area,
    load_foreground, begin_background,
    begin_area_window, begin_return_window,
    close_room_window, fade_to_options,
};
/// Requires live exclusive hardware and native ROM covering referenced map records.
pub(crate) unsafe fn advance(game: &mut Game) -> CoroutineRet {
    unsafe {
        STATES[word!(game.menu_index) as usize](game);
    }
    0
}
fn fade_to_options(game: &mut Game) {
    draw_area_labels(game);
    game.fade(false);
    if (game.ram().read_byte(0x51) & 0xf) == 0 {
        word! {game; game_state = kGameState_2_GameOptionsMenu as u16; menu_index = 0; }
    }
}
fn prepare(game: &mut Game) {
    menu_tilemap::clear(game);
    menu_tilemap::upload(2048, 0x7e3600, 1, true, game);
    word! {game; palette_change_num = 0; }
    menu_tilemap::load_palette(game);
    word_transfers!(game, ascending, 32; le(target_palettes, 0) <- le(palette_buffer, 0));
    word! { game, 0; target_palettes[14], target_palettes[30] }
    word!(game.menu_index += 1);
}
fn fade_palette(game: &mut Game) {
    if crate::game::palette_effects::fading::file_select(game) != 0 {
        word! { game, 0; reg_BG1VOFS, reg_BG1HOFS, reg_BG2VOFS, reg_BG2HOFS }
        word!(game.file_select_map_area_index = area_selection(game).unwrap_or(0) as u16);
        for area in 0..6 {
            menu_tilemap::area_palette(area, area == word!(game.area_index), game);
        }
        word!(game.menu_index += 1);
        word!(game.g_word_7E0787 = 0);
        game.ram().write_byte(0x69, 2);
    }
}
fn load_foreground(game: &mut Game) {
    menu_tilemap::upload(
        2048,
        0x810000 | u32::from(addr_kAreaSelectForegroundTilemap),
        0,
        true,
        game,
    );
    set_native_word(
        0x51,
        ((game.ram().read_byte(0x52) as i32) << 8 | 0xf) as u16,
        game,
    );
    word!(game.menu_index += 1);
}
fn begin_background(game: &mut Game) {
    word!(game.menu_index += 1);
    load_background(word!(game.area_index), game);
}
fn load_background(j: u16, game: &mut Game) {
    menu_tilemap::upload(
        2048,
        0x810000
            | u32::from((2048 * i32::from(j) + addr_kAreaSelectBackgroundTilemaps as i32) as u16),
        2,
        false,
        game,
    );
}
unsafe fn begin_area_window(game: &mut Game) {
    unsafe {
        game.ram().write_byte(0x6b, 4);
        crate::rtl::register_writes! { game;
            show_mirror(SubScreen(Layers::of(false, false, true, false, false)), 0x6d, false); show(SubWindowMask(Layers::of(false, false, true, false, false)));
            mirror(CGWSEL as u16, 2, 0x6f, true); mirror(CGADSUB as u16, 0x25, 0x72, true);
        }
        game.ram().write_byte(0x74 + 2, 0x80);
        game.ram().write_byte(0x74 + 1, 64);
        game.ram().write_byte(0x74 + 0, 32);
        menu_window::initialize(game);
        crate::rtl::write_register_and_mirror(HDMAEN as u16, 0xc, 0x85, true, game);
        configure_area_window(game);
        word!(game.hdma_objects_enable_flag = 0);
        game.ram()
            .write_byte(0x727, ((word!(game.menu_index)) as u8).wrapping_add(1));
    }
}
unsafe fn configure_area_window(game: &mut Game) {
    unsafe {
        game.ram().write_byte(0x69, 19);
        crate::rtl::register_writes! { game;
            show_mirror(MainScreen(Layers::of(true, true, false, false, true)), 0x6c, false); show(MainWindowMask(Layers::of(true, true, false, false, true)));
            mirror(W12SEL as u16, 0x23, 0x60, true); mirror(W34SEL as u16, 3, 0x61, true);
            mirror(WOBJSEL as u16, 0x23, 0x62, true);
        }
    }
}
fn open_area_window(game: &mut Game) {
    draw_area_labels(game);
    if menu_window::open_from_center(game) {
        game.ram()
            .write_byte(0x727, ((word!(game.menu_index)) as u8).wrapping_add(1));
        game.ram().write_byte(0x69, game.ram().read_byte(0x69) & !2);
        game.ram().write_byte(0x6c, 0);
        game.ram().write_byte(0x6d, 0);
        word! {game; reg_BG2VOFS = 24; pausemenu_palette_animation_timer = 1; }
        let mut _v1: u16 = 0;
        word_transfers!(game, ascending, 800; xray(0) <- rom(0xb6e000, 0));
        _v1 = 800;
        word_transfers!(game, ascending, 224; xray(800) <- value(10241));
        _v1 = 1024;
        menu_tilemap::area_label(game);
        word_transfers!(game, descending, 160; xray(959) <- rom(0x81b14b, 160));
        menu_tilemap::upload(2048, 0x7e4000, 1, false, game);
    }
}
#[forbid(unsafe_code)]
fn choose_area(game: &mut Game) {
    let mut keys = word!(game.joypad1_newkeys) as i16;
    'handle_action: {
        let previous = word!(game.joypad1_newkeys) & (kButton_Up | kButton_Left) as u16 != 0;
        let attempts = if previous {
            3
        } else if word!(game.joypad1_newkeys)
            & (kButton_Select | kButton_Down | kButton_Right) as u16
            != 0
        {
            6
        } else {
            0
        };
        if attempts != 0 {
            keys = word!(game.enable_debug) as i16;
            if word!(game.enable_debug) != 0 {
                let mut selection = word!(game.file_select_map_area_index);
                for _ in 0..attempts {
                    selection = if previous {
                        previous_area(selection)
                    } else {
                        next_area(selection)
                    };
                    if area_available(selection, game) != 0 {
                        if previous {
                            game.queue_sound(0, (0x37) as u8, 6);
                        }
                        switch_area(selection, game);
                        if !previous {
                            game.queue_sound(0, (0x37) as u8, 6);
                        }
                        break 'handle_action;
                    }
                }
                break 'handle_action;
            }
        }
        if keys < 0 {
            word!(game.menu_index = 22);
            break 'handle_action;
        }
        if keys as i32 & (kButton_Start as i32 | kButton_A as i32) != 0 {
            game.queue_sound(0, (0x38) as u8, 6);
            select_area(game);
            return;
        }
    }
    draw_area_labels(game);
}
fn previous_area(a: u16) -> u16 {
    let result: u16 = (a).wrapping_sub(1);
    if (result & 0x8000) != 0 {
        return 5;
    }
    result
}
fn next_area(a: u16) -> u16 {
    let result: u16 = (a).wrapping_add(1);
    if result as i32 - 6i32 & 0x8000 == 0 {
        return 0;
    }
    result
}
#[forbid(unsafe_code)]
fn select_area(game: &mut Game) {
    word!(game.menu_index += 1);
    if word!(game.enable_debug) == 0 {
        word!(game.area_index = word!(game.sram_area_index));
        word!(game.load_station_index = word!(game.sram_save_station_index));
        draw_area_labels(game);
        return;
    }
    word!(game.area_index = AREAS[word!(game.file_select_map_area_index) as usize]);
    let mut bits = native_word(0xd8f8 + 2 * usize::from(word!(game.area_index)), game);
    let mut station = 0u16;
    let coordinates = station_coordinates(word!(game.area_index), game);
    let mut remaining = 16u16;
    let mut unlocked_only = true;
    loop {
        if !unlocked_only || {
            let unlocked = bits & 1 != 0;
            bits >>= 1;
            unlocked
        } {
            match coordinates.table_word((2 * i32::from(station)) as isize) {
                0xfffe => {}
                0xffff => station = u16::MAX,
                _ => break,
            }
        }
        station = station.wrapping_add(1);
        remaining = remaining.wrapping_sub(1);
        if remaining == 0 {
            if !unlocked_only {
                Unreachable(game);
            }
            unlocked_only = false;
        }
    }
    word!(game.load_station_index = station);
    draw_area_labels(game);
}
fn area_available(a: u16, game: &mut Game) -> u16 {
    let v1: i32 = 2 * AREAS[a as usize] as i32;
    if native_word(0xd8f8 + v1 as usize, game) != 0 {
        return 1;
    }
    let t = game
        .capture_native_rom()
        .table_word(
            crate::rtl::lorom_offset(0x82c80b) as usize,
            (v1 / 2) as isize,
        )
        .wrapping_add(64);
    (t != 0xffff) as u16
}
fn switch_area(R28: u16, game: &mut Game) {
    menu_tilemap::area_palette(
        AREAS[word!(game.file_select_map_area_index) as usize],
        false,
        game,
    );
    word!(game.file_select_map_area_index = R28);
    menu_tilemap::area_palette(AREAS[R28 as usize], true, game);
    load_background(AREAS[word!(game.file_select_map_area_index) as usize], game);
}
static LABEL_POSITIONS: [u16; 12] = [
    0x5b, 0x32, 0x2a, 0x7f, 0x5e, 0xb5, 0xce, 0x50, 0xce, 0x9f, 0x87, 0x8b,
];
fn draw_area_labels(game: &mut Game) {
    let mut r3: u16 = 0;
    sprite_output::menu(
        table_word(
            &game.pause_menu.label_spritemaps,
            game,
            LABEL_SPRITEMAP_ADDRESS,
            0,
        ),
        0x80,
        0x10,
        r3,
        game,
    );
    for i in 0i32..6 {
        r3 = (if i == word!(game.file_select_map_area_index) as i32 {
            0
        } else {
            512
        }) as u16;
        let v1: u16 = (2u16).wrapping_mul(AREAS[i as usize]);
        let mut r36: u16 = native_word(0xd8f8 + v1 as usize, game);
        let mut v2 = station_coordinates(AREAS[i as usize], game);
        let mut R30: i32 = 16;
        while v2.word(0) != 0xffff {
            let v4: i32 = r36 as i32 & 1;
            r36 = r36 >> 1;
            if !(v4 != 0 && v2.word(0) != 0xfffe) {
                v2.base += 4;
                R30 -= 1;
                if R30 != 0 {
                    continue;
                }
                if !(word!(game.enable_debug) != 0 && v2.word(0) != 0xffff) {
                    break;
                }
            }
            let j: i32 = 4 * AREAS[i as usize] as i32 >> 1;
            sprite_output::menu(
                (table_word(
                    &game.pause_menu.label_spritemaps,
                    game,
                    LABEL_SPRITEMAP_ADDRESS,
                    0,
                ) as i32
                    + AREAS[i as usize] as i32
                    + 1) as u16,
                LABEL_POSITIONS[j as usize],
                LABEL_POSITIONS[(j + 1) as usize],
                r3,
                game,
            );
            break;
        }
    }
}
static WINDOW_TIMERS: [u16; 6] = [0x33, 0x35, 0x2d, 0x33, 0x33, 0x22];
unsafe fn begin_room_window(game: &mut Game) {
    unsafe {
        draw_area_labels(game);
        menu_window::initialize(game);
        game.ram().write_byte(0x69, 19);
        game.ram().write_byte(0x6c, 19);
        crate::rtl::register_writes! { game;
            mirror(W12SEL as u16, 0x32, 0x60, true); mirror(W34SEL as u16, 2, 0x61, true);
            mirror(CGADSUB as u16, 5, 0x72, true); mirror(WOBJSEL as u16, 0x22, 0x62, true);
        }
        game.ram().write_byte(0x5d, 48);
        game.ram().write_byte(0x9e09, 0);
        game.ram().write_byte(0x9e19, 0);
        menu_tilemap::area_label(game);
        menu_tilemap::upload(512, 0x7e4000, 1, false, game);
        word! {game; expand_sq_timer = WINDOW_TIMERS[word!(game.area_index) as usize]; }
        let v2: i32 = ((4u16).wrapping_mul(word!(game.area_index))) as i32 >> 1;
        word!(game.expand_sq_left_pos = LABEL_POSITIONS[v2 as usize]);
        word!(game.expand_sq_right_pos = word!(game.expand_sq_left_pos));
        word!(game.expand_sq_top_pos = LABEL_POSITIONS[(v2 + 1) as usize]);
        word!(game.expand_sq_bottom_pos = word!(game.expand_sq_top_pos));
        word! { game, 0;
            expand_sq_left_subpos, expand_sq_right_subpos, expand_sq_top_subpos,
            expand_sq_bottom_subpos,
        }
        menu_window::load_velocities(word!(game.area_index), game);
        menu_window::project(game);
        crate::rtl::write_register_and_mirror(HDMAEN as u16, 0xc, 0x85, true, game);
        word!(game.menu_index += 1);
        game.queue_sound(0, (0x3b) as u8, 6);
    }
}
fn open_room_window(game: &mut Game) {
    menu_window::advance(menu_window::Motion::Expand, game);
    if (word!(game.expand_sq_timer -= 1, reread) & 0x8000) != 0 {
        word!(game.menu_index += 1);
        game.ram().write_byte(0x69, 2);
        game.ram().write_byte(0x6c, 0);
        game.ram().write_byte(0x6d, 0);
        game.ram().write_byte(0x6b, 0);
    }
    draw_area_labels(game);
}
unsafe fn initialize_room(game: &mut Game) {
    unsafe {
        load_area_map(game);
        draw_room_map(game);
        room_entry::begin(
            &mut Runtime::new(game),
            ArrivalRequest::Start(StartSite::CurrentLoadStation),
        );
        set_hdma_enabled(false, game);
        clear_hdma(game);
        let room = menu_tilemap::native_record(0x8f0000 | u32::from(word!(game.room_ptr)), game);
        game.ram().write_byte(0x79f, room.byte(1));
        game.ram().write_byte(0x7a1, room.byte(2));
        game.ram().write_byte(0x7a3, room.byte(3));
        reset_animations(game);
        scroll_limits(game);
        center_map(112, game);
        word!(game.map_min_y_scroll += 24);
        word!(game.reg_BG2VOFS = 24);
        crate::ram_map::checked_word!(game, 0x69; = crate::ram_map::checked_word!(game, 0x69;) & !4);
        word!(game.menu_index += 1);
        word! { game, 0;
            map_scrolling_direction, map_scrolling_speed_index,
            samus_position_indicator_animation_frame, samus_position_indicator_animation_timer,
            samus_position_indicator_animation_loop_counter,
        }
    }
}
fn choose_room(game: &mut Game) {
    {
        draw_icons(IconMode::FileSelect, game);
        scroll_arrows(MapScreen::FileSelect, game);
        scroll(game);
        display_elevator_destinations(game);
        if word!(game.enable_debug) != 0
            && (word!(game.joypad2_new_keys) as i32 & kButton_Select as i32) != 0
        {
            game.queue_sound(0, (0x38) as u8, 6);
            let mut r24: u16 = native_word(0xd8f8 + 2 * usize::from(word!(game.area_index)), game);
            let mut v1: i16 = word!(game.load_station_index) as i16;
            do_until! { v1 < 0 ;
                r24 = r24 >> 1;
                v1 -= 1;
            }
            let r0 = station_coordinates(word!(game.area_index), game);
            let mut v3: i16 = (4i16).wrapping_mul((word!(game.load_station_index)) as i16);
            let r18: u16 = r0.table_word((v3 >> 1) as isize);
            let r20: u16 = r0
                .image
                .table_word(r0.base, ((v3 as i32 >> 1) + 1) as isize);
            let mut check_unlocked = word!(game.load_station_index) as i32 - 16i32 & 0x8000 != 0;
            loop {
                let inspect_terminal = if check_unlocked {
                    word!(game.load_station_index += 1, reread) as i32 - 16i32 & 0x8000 == 0
                } else {
                    word!(game.load_station_index += 1);
                    true
                };
                if inspect_terminal {
                    let mut v5: i16 = r0.image.table_word(
                        r0.base,
                        (2 * word!(game.load_station_index) as i32) as isize,
                    ) as i16;
                    if v5 == -1i16 {
                        word!(game.load_station_index = 0);
                        r24 = native_word(0xd8f8 + 2 * usize::from(word!(game.area_index)), game);
                    } else if v5 != -2i16 {
                        break;
                    } else {
                        check_unlocked = false;
                        continue;
                    }
                }
                let mut v4: i8 = ((r24) as i8) & (1i8);
                r24 = r24 >> 1;
                if v4 == 0
                    || r0.image.table_word(
                        r0.base,
                        (2 * word!(game.load_station_index) as i32) as isize,
                    ) >= 0xfffe
                {
                    check_unlocked = true;
                } else {
                    break;
                }
            }
            let v6: u16 = (4u16).wrapping_mul(word!(game.load_station_index));
            let w7: u16 = r0.image.table_word(
                r0.base,
                (2 * word!(game.load_station_index) as i32) as isize,
            );
            if w7 as i32 - word!(game.reg_BG1HOFS) as i32 & 0x8000 != 0
                || w7 as i32 - 256i32 - word!(game.reg_BG1HOFS) as i32 & 0x8000 == 0
            {
                let mut v8: i16 = (word!(game.reg_BG1HOFS) as i32
                    + r0.table_word((v6 >> 1) as isize) as i32
                    - r18 as i32) as i16;
                if v8 >= 0 {
                    if (v8).wrapping_sub((word!(game.map_min_x_scroll)) as i16) >= 0 {
                        v8 = word!(game.map_min_x_scroll) as i16;
                    }
                } else {
                    v8 = 0;
                }
                word!(game.reg_BG1HOFS = v8 as u16);
            }
            let v9: u16 = (v6).wrapping_add(2);
            let v10: u16 = r0.table_word((v9 >> 1) as isize);
            if v10 as i32 - word!(game.reg_BG1VOFS) as i32 & 0x8000 != 0
                || v10 as i32 - 161i32 - word!(game.reg_BG1VOFS) as i32 & 0x8000 == 0
            {
                let mut v11: u16 = (word!(game.reg_BG1VOFS) as i32
                    + r0.table_word((v9 >> 1) as isize) as i32
                    - r20 as i32) as u16;
                if ((v11) as i16).wrapping_sub((word!(game.map_min_y_scroll)) as i16) >= 0 {
                    v11 = word!(game.map_min_y_scroll);
                }
                word!(game.reg_BG1VOFS = v11);
            }
        } else if (word!(game.joypad1_newkeys) & 0x8000) != 0 {
            word!(game.menu_index += 5);
            let Some(selection) = area_selection(game) else {
                word!(game.file_select_map_area_index = 0);
                return;
            };
            word!(game.file_select_map_area_index = selection as u16);
            game.queue_sound(0, (0x3c) as u8, 6);
        } else if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32)
            != 0
        {
            word!(game.menu_index += 1);
            game.queue_sound(0, (0x38) as u8, 6);
        }
    }
}
fn show_room(game: &mut Game) {
    draw_icons(IconMode::FileSelect, game);
    display_elevator_destinations(game);
    word!(game.menu_index += 1);
}
fn fade_room(game: &mut Game) {
    draw_icons(IconMode::FileSelect, game);
    display_elevator_destinations(game);
    game.fade(false);
    if (game.ram().read_byte(0x51) & 0xf) == 0 {
        word!(game.menu_index += 1);
        set_native_word(0xf78 + core::mem::offset_of!(EnemyData, x_pos), 32, game);
    }
}
fn start_game(game: &mut Game) {
    let timer = 0xf78 + core::mem::offset_of!(EnemyData, x_pos);
    set_native_word(timer, native_word(timer, game).wrapping_sub(1), game);
    if native_word(timer, game) == 0 {
        game.set_blank(true);
        word!(game.game_state += 1);
        word!(game.menu_index = 0);
    }
}
fn clear_room(game: &mut Game) {
    game.ram().write_byte(0x69, 18);
    word_transfers!(game, descending, 1024; native(0x37fe) <- value(15));
    menu_tilemap::upload(2048, 0x7e3000, 0, true, game);
    word!(game.menu_index += 1);
}
fn restore_area(game: &mut Game) {
    menu_tilemap::load_palette(game);
    for area in 0..6 {
        menu_tilemap::area_palette(area, false, game);
    }
    menu_tilemap::area_palette(word!(game.area_index), true, game);
    load_background(word!(game.area_index), game);
    word!(game.menu_index += 1);
}
unsafe fn begin_return_window(game: &mut Game) {
    unsafe {
        game.ram().write_byte(0x85, 0);
        game.queue_sound(0, (0x3c) as u8, 6);
        word! { game;
            expand_sq_timer =
                (WINDOW_TIMERS[word!(game.area_index) as usize]).wrapping_sub(12);
        }
        menu_window::load_velocities(word!(game.area_index), game);
        word! { game;
            expand_sq_left_pos = 8;
            expand_sq_right_pos = 248;
            expand_sq_top_pos = 8;
            expand_sq_bottom_pos = 216;
        }
        word! { game, 0;
            expand_sq_left_subpos, expand_sq_right_subpos, expand_sq_top_subpos,
            expand_sq_bottom_subpos,
        }
        menu_window::project(game);
        word!(game.menu_index += 1);
        crate::rtl::register_writes! { game;
            mirror(HDMAEN as u16, 0xc, 0x85, true); mirror(W12SEL as u16, 0x32, 0x60, true);
            mirror(W34SEL as u16, 2, 0x61, true);
        }
        game.ram().write_byte(0x5d, 48);
        game.ram().write_byte(0x62, 34);
        game.ram().write_byte(0x9e09, 0);
        game.ram().write_byte(0x9e19, 0);
        word! { game, 0; reg_BG1VOFS, reg_BG1HOFS }
    }
}
fn close_room_window(game: &mut Game) {
    menu_window::advance(menu_window::Motion::Contract, game);
    draw_area_labels(game);
    if (word!(game.expand_sq_timer -= 1, reread) & 0x8000) != 0 {
        word!(game.menu_index -= 15);
        game.ram().write_byte(0x69, 17);
        game.ram().write_byte(0x6c, 0);
        game.ram().write_byte(0x6d, 0);
    }
}

fn station_coordinates(area: u16, game: &mut Game) -> crate::rom::RomRecord {
    let pointer = menu_tilemap::native_record(
        0x820000 | u32::from((addr_kMapIconDataPointers as i32 + 64 + 2 * i32::from(area)) as u16),
        game,
    )
    .word(0);
    menu_tilemap::native_record(0x820000 | u32::from(pointer), game)
}

fn native_word(offset: usize, game: &Game) -> u16 {
    u16::from_ne_bytes([
        game.ram().read_byte(offset),
        game.ram().read_byte(offset + 1),
    ])
}

fn set_native_word(offset: usize, value: u16, game: &mut Game) {
    game.ram().copy_in(offset, &value.to_ne_bytes());
}

fn area_selection(game: &Game) -> Option<usize> {
    AREAS
        .iter()
        .position(|&area| area == word!(game.area_index))
}

const SCROLL_SPEEDS: [u16; 32] = [
    0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Advance the shared file-select/pause-map scroll, with RAM as its only state.
#[forbid(unsafe_code)]
pub(crate) fn scroll(game: &mut Game) {
    // Preserve the dispatch table's bounds failure before touching scroll state.
    let direction = [0, 1, 2, 3, 4][usize::from(word!(game.map_scrolling_direction))];
    if direction == 0 {
        word!(game.map_scrolling_gear_switch_timer = 4);
        return;
    }
    let mut index = word!(game.map_scrolling_speed_index);
    if word!(game.map_scrolling_gear_switch_timer) == 0 {
        index = index.wrapping_add(32);
    }
    let speed = SCROLL_SPEEDS[usize::from(index >> 1)];
    match direction {
        1 => word!(game.reg_BG1HOFS -= speed),
        2 => word!(game.reg_BG1HOFS += speed),
        3 => word!(game.reg_BG1VOFS -= speed),
        _ => word!(game.reg_BG1VOFS += speed),
    };
    word!(game.map_scrolling_speed_index += 1);
    if word!(game.map_scrolling_speed_index += 1, reread) & 0xf == 0 {
        game.queue_sound(0, (0x36) as u8, 6);
        word! { game, 0; map_scrolling_direction, map_scrolling_speed_index }
        if word!(game.map_scrolling_gear_switch_timer) != 0 {
            word!(game.map_scrolling_gear_switch_timer -= 1);
        }
    }
}

fn map_record(cursor: u16, game: &mut Game) -> crate::rom::RomRecord {
    menu_tilemap::native_record(0x820000 | u32::from(cursor), game)
}

fn draw_record(sprite: u16, record: &crate::rom::RomRecord, palette: u16, game: &mut Game) {
    sprite_output::menu(
        sprite,
        record.word(0).wrapping_sub(word!(game.reg_BG1HOFS)),
        record.word(2).wrapping_sub(word!(game.reg_BG1VOFS)),
        palette,
        game,
    );
}

#[derive(Clone, Copy)]
pub(crate) enum IconMode {
    Pause,
    FileSelect,
}

pub(crate) fn draw_icons(mode: IconMode, game: &mut Game) {
    if matches!(mode, IconMode::FileSelect) {
        animate_palette(game);
    }
    boss_icons(9, 0xc7cb, game);
    for (sprite, table) in [(0xb, 0xc7db), (0xa, 0xc7eb), (0x4e, 0xc7fb)] {
        icon_group(sprite, table, u16::MAX, 3584, game);
    }
    match mode {
        IconMode::Pause => {
            if word!(game.area_index) == word!(game.sram_area_index) {
                let bits = station_bits(0, game);
                let mut bytes = bits.to_ne_bytes();
                bytes[0] = bits as u8
                    & [1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80][word!(game.load_station_index) as usize];
                icon_group(8, 0xc80b, u16::from_ne_bytes(bytes), 1024, game);
            }
            if word!(game.enable_debug) != 0 {
                icon_group(8, 0xc82b, u16::MAX, 1024, game);
            }
        }
        IconMode::FileSelect => {
            let sprite = advance_marker(game);
            // This fixed table is native and linear; only its result is bank-wrapped.
            let address = game.capture_native_rom().table_word(
                crate::rtl::lorom_offset(0x82c80b) as usize,
                word!(game.area_index) as isize,
            );
            let record = map_record(address, game);
            let offset = 4 * usize::from(word!(game.load_station_index));
            let x = record.word(offset).wrapping_sub(word!(game.reg_BG1HOFS));
            let y = record
                .word(offset + 2)
                .wrapping_sub(word!(game.reg_BG1VOFS));
            if word!(game.samus_position_indicator_animation_loop_counter) & 1 == 0 {
                sprite_output::menu(0x12, x, y, 3584, game);
            }
            sprite_output::menu(sprite, x, y, 3584, game);
            if word!(game.enable_debug) != 0 {
                let saves = station_bits(0, game);
                icon_group(0xc, 0xc80b, saves, 1536, game);
                let elevators = station_bits(1, game);
                icon_group(0x17, 0xc81b, elevators, 1536, game);
                icon_group(0xc, 0xc82b, u16::MAX, 1536, game);
            }
        }
    }
    if word!(game.area_index) == 0 {
        sprite_output::menu(
            0x63,
            0xd8u16.wrapping_sub(word!(game.reg_BG1HOFS)),
            0x28u16.wrapping_sub(word!(game.reg_BG1VOFS)),
            3584,
            game,
        );
    }
}

fn station_bits(offset: usize, game: &mut Game) -> u16 {
    let address = 0xd8f8 + 2 * usize::from(word!(game.area_index)) + offset;
    u16::from_ne_bytes([
        game.ram().read_byte(address),
        game.ram().read_byte(address + 1),
    ])
}

fn icon_group(sprite: u16, table: u16, mut bits: u16, palette: u16, game: &mut Game) {
    let mut cursor = map_record(
        (i32::from(table) + 2 * i32::from(word!(game.area_index))) as u16,
        game,
    )
    .word(0);
    if cursor == 0 {
        return;
    }
    loop {
        let record = map_record(cursor, game);
        if record.word(0) & 0x8000 != 0 {
            break;
        }
        let visible = bits & 1 != 0;
        bits >>= 1;
        if visible && explored(record.word(0), record.word(2), game) {
            draw_record(sprite, &record, palette, game);
        }
        cursor = cursor.wrapping_add(4);
    }
}

fn explored(x: u16, y: u16, game: &mut Game) -> bool {
    let offset =
        usize::from((x & 0xff00) >> 1) + usize::from(x as u8 >> 6) + usize::from((y & 0xfff8) >> 1);
    game.ram().read_byte(0x7f7 + offset) & (0x80 >> ((x >> 3) & 7)) != 0
}

fn boss_icons(sprite: u16, table: u16, game: &mut Game) {
    let mut bits =
        crate::game::gameplay_flag_runtime::boss_byte_at(word!(game.area_index), game) as i32;
    let cursor = map_record(
        (i32::from(table) + 2 * i32::from(word!(game.area_index))) as u16,
        game,
    )
    .word(0);
    if cursor == 0 {
        return;
    }
    let mut record = map_record(cursor, game);
    loop {
        match record.word(0) {
            0xffff => return,
            0xfffe => bits >>= 1,
            _ => {
                let defeated = bits & 1 != 0;
                bits >>= 1;
                if defeated {
                    draw_record(0x62, &record, 3584, game);
                    draw_record(sprite, &record, 3072, game);
                } else if word!(game.has_area_map) != 0 {
                    draw_record(sprite, &record, 3584, game);
                } else {
                    // An undisplayed undefeated boss consumes a second bit.
                    bits >>= 1;
                }
            }
        }
        record.base += 4;
    }
}

pub(crate) fn animate_palette(game: &mut Game) {
    if word!(game.pausemenu_palette_animation_timer) as u8 == 0 {
        return;
    }
    game.ram().write_byte(
        0x73b,
        (word!(game.pausemenu_palette_animation_timer) as u8).wrapping_sub(1),
    );
    if word!(game.pausemenu_palette_animation_timer) as u8 != 0 {
        return;
    }
    let mut frame = i32::from(word!(game.pausemenu_palette_animation_frame)) + 1;
    loop {
        game.ram().write_byte(0x74f, frame as u8);
        let timer = game
            .capture_native_rom()
            .read(crate::rtl::lorom_offset(0x82c10c) as usize + 3 * frame as usize);
        if timer != 0xff {
            game.ram().write_byte(0x73b, timer);
            break;
        }
        game.queue_sound(2, (0x2a) as u8, 6);
        frame = 0;
    }
    let frame_word = u16::from_ne_bytes([0, word!(game.pausemenu_palette_animation_frame) as u8]);
    let mut source = (frame_word >> 3) + 30;
    for index in (176..192).rev() {
        let color = grid_word(
            &game.pause_menu.palette_animation,
            game,
            PALETTE_ANIMATION_ADDRESS,
            (source >> 1) as isize,
        );
        word_transfers!(game, ascending, 1; native(0xc000 + 2 * index) <- value(color));
        source = source.wrapping_sub(2);
    }
}

fn advance_marker(game: &mut Game) -> u16 {
    const DELAYS: [u16; 4] = [8, 4, 8, 4];
    const SPRITES: [u16; 4] = [0x5f, 0x60, 0x61, 0x60];
    let mut timer = word!(game.samus_position_indicator_animation_timer);
    if timer == 0 {
        let mut frame = word!(game.samus_position_indicator_animation_frame).wrapping_add(2);
        if (frame as i16).wrapping_sub(8) >= 0 {
            word!(game.samus_position_indicator_animation_loop_counter += 1);
            frame = 0;
        }
        word!(game.samus_position_indicator_animation_frame = frame);
        timer = DELAYS[(frame >> 1) as usize];
        word!(game.samus_position_indicator_animation_timer = timer);
    }
    word!(game.samus_position_indicator_animation_timer = timer.wrapping_sub(1));
    SPRITES[(word!(game.samus_position_indicator_animation_frame) >> 1) as usize]
}

pub(crate) fn draw_player_marker(game: &mut Game) {
    let sprite = advance_marker(game);
    sprite_output::menu(
        sprite,
        (8 * (i32::from(word!(game.room_x_coordinate_on_map))
            + i32::from(game.ram().read_byte(0xaf7)))
            - i32::from(word!(game.reg_BG1HOFS))) as u16,
        (8 * (i32::from(word!(game.room_y_coordinate_on_map))
            + i32::from(game.ram().read_byte(0xafb))
            + 1)
            - i32::from(word!(game.reg_BG1VOFS))) as u16,
        3584,
        game,
    );
}

pub(crate) fn display_elevator_destinations(game: &mut Game) {
    if game
        .ram()
        .read_byte(0xd908 + usize::from(word!(game.area_index)))
        == 0
    {
        return;
    }
    let mut cursor = i32::from(table_word(
        &game.pause_menu.map_icons,
        game,
        MAP_ICON_ADDRESS,
        word!(game.area_index) as isize,
    ));
    loop {
        let record = map_record(cursor as u16, game);
        if record.word(0) == 0xffff {
            break;
        }
        draw_record(record.word(4), &record, 0, game);
        cursor += 6;
    }
}

pub(crate) fn reset_animations(game: &mut Game) {
    word! { game, 0;
        pausemenu_reserve_tank_delay_ctr, reg_BG1HOFS, reg_BG2HOFS, reg_BG3HOFS, reg_BG2VOFS,
        reg_BG3VOFS, pausemenu_button_label_mode, pausemenu_lr_animation_frame, UNUSED_word_7E0745,
        samus_position_indicator_animation_frame, samus_position_indicator_animation_timer,
        samus_position_indicator_animation_loop_counter,
    }
    word! {game; pausemenu_lr_animation_timer = game.capture_native_rom().table_word(crate::rtl::lorom_offset(0x82c10c) as usize, 0);
    pausemenu_palette_animation_timer = 1; pausemenu_palette_animation_frame = 0; }
}

pub(crate) fn center_map(vertical_center: u16, game: &mut Game) {
    word!(
        game.reg_BG1HOFS = (i32::from(word!(game.map_min_x_scroll))
            + (i32::from(word!(game.map_max_x_scroll).wrapping_sub(word!(game.map_min_x_scroll)))
                >> 1)
            - 128) as u16
    );
    let x = (8
        * (i32::from(word!(game.room_x_coordinate_on_map))
            + i32::from((word!(game.samus_x_pos) & 0xff00) >> 8))
        - i32::from(word!(game.reg_BG1HOFS))) as u16;
    let right = 224i16.wrapping_sub(x as i16);
    if right >= 0 {
        let left = 32u16.wrapping_sub(x);
        if left & 0x8000 == 0 {
            word!(game.reg_BG1HOFS -= left);
        }
    } else {
        word!(game.reg_BG1HOFS -= right as u16);
    }
    let middle = (i32::from(word!(game.map_min_y_scroll))
        + (i32::from(word!(game.map_max_y_scroll).wrapping_sub(word!(game.map_min_y_scroll))) >> 1)
        + 16) as u16;
    let aligned = (i32::from(vertical_center) - i32::from(middle)) & 0xfff8;
    word!(game.reg_BG1VOFS = (-aligned) as u16);
    let y = (8
        * (i32::from(word!(game.room_y_coordinate_on_map))
            + i32::from(game.ram().read_byte(0xafb))
            + 1)
        + aligned) as u16;
    let top = 64i16.wrapping_sub(y as i16);
    if top >= 0 {
        word!(game.reg_BG1VOFS -= top as u16);
        if (i32::from(word!(game.reg_BG1VOFS)) + 40) & 0x8000 != 0 {
            word!(game.reg_BG1VOFS = -40i32 as u16);
        }
    }
}

#[derive(Clone, Copy)]
enum MapEdge {
    Left,
    Right,
    Top,
    Bottom,
}

fn find_map_edge(source: Option<&crate::rom::RomRecord>, edge: MapEdge, game: &mut Game) -> u16 {
    let read = |offset: usize| match source {
        Some(record) => record.byte(offset),
        None => game.ram().read_byte(0x7f7 + offset),
    };
    match edge {
        MapEdge::Left | MapEdge::Right => {
            let right = matches!(edge, MapEdge::Right);
            for step in 0..64u16 {
                let column = if right { 63 - step } else { step };
                // The original left scan starts its second half one byte earlier.
                let base = usize::from(column / 8)
                    + if column < 32 {
                        0
                    } else if right {
                        124
                    } else {
                        123
                    };
                let mask = 0x80u8 >> (column & 7);
                for offset in (0..128).step_by(4) {
                    if mask & read(base + offset) != 0 {
                        return column;
                    }
                }
            }
            if right { 28 } else { 26 }
        }
        MapEdge::Top | MapEdge::Bottom => {
            let bottom = matches!(edge, MapEdge::Bottom);
            // Both searches stop before the final row at the opposite edge.
            for step in 0..31u16 {
                let row = if bottom { 31 - step } else { step };
                let base = 4 * usize::from(row);
                for offset in 0..4 {
                    if read(base + offset) != 0 || read(base + 128 + offset) != 0 {
                        return row;
                    }
                }
            }
            if bottom { 11 } else { 1 }
        }
    }
}

pub(crate) fn scroll_limits(game: &mut Game) {
    let source = if word!(game.has_area_map) != 0 {
        let pointer = map_record(
            (i32::from(addr_kPauseMenuMapData as u16) + 2 * i32::from(word!(game.area_index)))
                as u16,
            game,
        )
        .word(0);
        Some(map_record(pointer, game))
    } else {
        None
    };
    word!(
        game.map_min_x_scroll = find_map_edge(source.as_ref(), MapEdge::Left, game).wrapping_mul(8)
    );
    if word!(game.area_index) == 4 {
        word!(game.map_min_x_scroll -= 24);
    }
    word!(
        game.map_max_x_scroll =
            find_map_edge(source.as_ref(), MapEdge::Right, game).wrapping_mul(8)
    );
    word!(
        game.map_min_y_scroll = find_map_edge(source.as_ref(), MapEdge::Top, game).wrapping_mul(8)
    );
    word!(
        game.map_max_y_scroll =
            find_map_edge(source.as_ref(), MapEdge::Bottom, game).wrapping_mul(8)
    );
}

pub(crate) enum MapScreen {
    Pause,
    FileSelect,
}

pub(crate) fn render_map(screen: MapScreen, game: &mut Game) {
    let mut area = word!(game.area_index);
    if (i32::from(word!(game.area_index)) - 7) & 0x8000 == 0 {
        area = 0;
    }
    let address = menu_tilemap::native_record(0x82964a, game).load24(3 * usize::from(area));
    let tiles = menu_tilemap::native_record(address, game);
    let (destination, empty) = match screen {
        MapScreen::Pause => (0x4000, 31),
        MapScreen::FileSelect => (0x3000, 15),
    };
    let bitmap = map_record(
        table_word(
            &game.pause_menu.area_maps,
            game,
            AREA_MAP_ADDRESS,
            area as isize,
        ),
        game,
    );
    if game
        .ram()
        .read_byte(0xd908 + usize::from(word!(game.area_index)))
        != 0
    {
        let mut mapped = bitmap.word(0).swap_bytes();
        let mut explored = native_word(0x7f7, game).swap_bytes();
        for index in 0..2048 {
            let mut value = tiles.word(2 * index);
            if explored & 0x8000 != 0 {
                value &= 0xfbff;
            } else if mapped & 0x8000 == 0 {
                value = 31;
            }
            explored = explored.wrapping_mul(2);
            mapped = mapped.wrapping_mul(2);
            set_native_word(destination + 2 * index, value, game);
            if index & 15 == 15 {
                // Retain the native/physical prefetch after the final tile too.
                let offset = 2 * ((index + 1) / 16);
                mapped = bitmap.word(offset).swap_bytes();
                explored = native_word(0x7f7 + offset, game).swap_bytes();
            }
        }
    } else {
        for index in 0..2048 {
            let byte = index / 8;
            let explored = 0x7f7 + byte;
            let previous = game.ram().read_byte(explored);
            game.ram()
                .write_byte(explored, game.ram().read_byte(explored).wrapping_mul(2));
            #[cfg(sm_verify_dll)]
            crate::game::session_runtime::player_legacy_edges::map_write(game, byte, index, 0);
            let value = if previous & 0x80 == 0 {
                empty
            } else {
                game.ram()
                    .write_byte(explored, game.ram().read_byte(explored).wrapping_add(1));
                #[cfg(sm_verify_dll)]
                crate::game::session_runtime::player_legacy_edges::map_write(game, byte, index, 1);
                tiles.word(2 * index) & 0xfbff
            };
            set_native_word(destination + 2 * index, value, game);
            #[cfg(sm_verify_dll)]
            crate::game::session_runtime::player_legacy_edges::map_write(game, byte, index, 2);
        }
    }
}

fn draw_room_map(game: &mut Game) {
    game.ram().write_byte(0x5d, 51);
    game.ram().write_byte(0x69, 19);
    word!(game.reg_BG1VOFS = -40i16 as u16);
    render_map(MapScreen::FileSelect, game);
    menu_tilemap::upload(4096, 0x7e3000, 0, false, game);
}

// The nine-entry union tables capture their native image before checking the index.
fn animation_entry(table: u32, index: usize, game: &mut Game) -> u16 {
    let record = menu_tilemap::native_record(table, game);
    let offset = [0, 2, 4, 6, 8, 10, 12, 14, 16][index];
    record.word(offset)
}

fn animation_ram_address(table: u32, index: usize, game: &mut Game) -> usize {
    let address = animation_entry(table, index, game);
    if address >= 0x2000 {
        crate::game::session_runtime::legacy_assertion(true);
    }
    usize::from(address)
}

pub(crate) fn animate_pause_sprite(animation: u16, x: u16, y: u16, game: &mut Game) {
    let index = (i32::from(animation) - 1) as usize;
    let frame = animation_ram_address(0x82c0c4, index, game);
    let timer = animation_ram_address(0x82c0b2, index, game);
    let script = map_record(animation_entry(0x82c0e8, index, game), game);
    set_native_word(timer, native_word(timer, game).wrapping_sub(1), game);
    if native_word(timer, game) as i16 <= 0 {
        set_native_word(frame, native_word(frame, game).wrapping_add(1), game);
        let mut duration = u16::from(script.byte(3 * usize::from(native_word(frame, game))));
        if duration == 255 {
            set_native_word(frame, 0, game);
            duration = u16::from(script.byte(0));
        }
        set_native_word(timer, duration, game);
    }
    let palette = animation_palette(game, 3);
    let delta = u16::from(script.byte(3 * usize::from(native_word(frame, game)) + 2));
    let variant = animation_ram_address(0x82c0d6, index, game);
    let variant = 2 * u16::from(game.ram().read_byte(variant));
    let sprite = map_record(
        table_word(
            &game.pause_menu.animation_spritemaps,
            game,
            ANIMATION_SPRITEMAP_ADDRESS,
            index as isize,
        )
        .wrapping_add(variant),
        game,
    )
    .word(0);
    sprite_output::menu(
        delta.wrapping_add(sprite),
        x,
        y.wrapping_sub(1),
        palette,
        game,
    );
}

pub(crate) fn draw_equipment_selector(game: &mut Game) {
    if word!(game.samus_max_reserve_health)
        | word!(game.collected_items)
        | word!(game.collected_beams)
        == 0
    {
        return;
    }
    let address = table_word(
        &game.pause_menu.selector_positions,
        game,
        SELECTOR_POSITION_ADDRESS,
        word!(game.pausemenu_equipment_category_item) as u8 as isize,
    )
    .wrapping_add(4 * u16::from(game.ram().read_byte(0x756)));
    let position = map_record(address, game);
    animate_pause_sprite(3, position.word(0).wrapping_sub(1), position.word(2), game);
}

fn scroll_arrow(address: u32, game: &mut Game) {
    let record = menu_tilemap::native_record(address, game);
    animate_pause_sprite(record.word(4), record.word(0), record.word(2), game);
    // Drawing can replace the image; input and direction use a fresh capture.
    let record = menu_tilemap::native_record(address, game);
    if word!(game.joypad1_lastkeys) & record.word(6) != 0
        && word!(game.map_scrolling_direction) == 0
    {
        word!(game.map_scrolling_direction = record.word(8));
    }
}

pub(crate) fn scroll_arrows(screen: MapScreen, game: &mut Game) {
    let (base, top, bottom) = match screen {
        MapScreen::Pause => (0x82b9a0, 56, 177),
        MapScreen::FileSelect => (0x81af32, 64, 145),
    };
    if (i32::from(word!(game.map_min_x_scroll)) - 24 - i32::from(word!(game.reg_BG1HOFS))) & 0x8000
        != 0
    {
        scroll_arrow(base, game);
    }
    if (i32::from(word!(game.map_max_x_scroll)) - 232 - i32::from(word!(game.reg_BG1HOFS))) & 0x8000
        == 0
    {
        scroll_arrow(base + 10, game);
    }
    if (i32::from(word!(game.map_min_y_scroll)) - top - i32::from(word!(game.reg_BG1VOFS))) & 0x8000
        != 0
    {
        scroll_arrow(base + 20, game);
    }
    if (i32::from(word!(game.map_max_y_scroll)) - bottom - i32::from(word!(game.reg_BG1VOFS)))
        & 0x8000
        != 0
    {
        if word!(game.map_scrolling_direction)
            == menu_tilemap::native_record(base + 30, game).word(8)
        {
            word! { game, 0; map_scrolling_gear_switch_timer, map_scrolling_direction, map_scrolling_speed_index }
        }
    } else {
        scroll_arrow(base + 30, game);
    }
}
