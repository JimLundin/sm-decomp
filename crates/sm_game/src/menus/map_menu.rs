//! Area and room selection, with the original menu-state and RAM compatibility.
use super::menu_tilemap;
use super::prelude::*;
use super::save_slots::load_area_map;
use crate::constants::{bank_81::*, bank_82::*};
use crate::display::control::{Layers, MainScreen, MainWindowMask, SubScreen, SubWindowMask};
use crate::display::sequence::scroll::BgScroll;
use crate::imported::{Imported, grid_word, word as table_word};
use crate::menu_state::set_menu_byte;
use crate::menu_state::{MENU, MenuState, WORDS};
use crate::menu_state::{MapScreenState, MenuFrame, MenuScreens};
use crate::session_cohort;
use sm_data::pause_menu::{
    ANIMATION_PALETTE_ADDRESS, ANIMATION_SPRITEMAP_ADDRESS, ANIMATIONS, AREA_MAP_ADDRESS,
    AREA_MAPS, LABEL_SPRITEMAP_ADDRESS, LABEL_SPRITEMAPS, MAP_ICON_ADDRESS, MAP_ICON_AREAS,
    PALETTE_ANIMATION_ADDRESS, PALETTE_ANIMATION_BYTES, PaletteAnimation, SELECTOR_CATEGORIES,
    SELECTOR_POSITION_ADDRESS, import_palette_animation, import_words,
};
use sm_data::table::Rows;

#[derive(Default)]
pub struct ImportedTables {
    label_spritemaps: Option<Imported<Rows<u16, LABEL_SPRITEMAPS>>>,
    map_icons: Option<Imported<Rows<u16, MAP_ICON_AREAS>>>,
    area_maps: Option<Imported<Rows<u16, AREA_MAPS>>>,
    animation_palettes: Option<Imported<Rows<u16, ANIMATIONS>>>,
    animation_spritemaps: Option<Imported<Rows<u16, ANIMATIONS>>>,
    selector_positions: Option<Imported<Rows<u16, SELECTOR_CATEGORIES>>>,
    palette_animation: Option<Imported<PaletteAnimation>>,
}

pub fn import(image: &sm_data::cartridge::RomImage) -> ImportedTables {
    fn words<const N: usize>(
        image: &sm_data::cartridge::RomImage,
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
pub fn animation_palette<H: Host + ?Sized>(game: &H, index: isize) -> u16 {
    table_word(
        &game.pause_menu_tables().animation_palettes,
        game,
        ANIMATION_PALETTE_ADDRESS,
        index,
    )
}
const AREAS: [u16; 6] = [0, 3, 5, 1, 4, 2];

/// Requires live exclusive hardware and native ROM covering referenced map records.
pub fn advance<H: Host + ?Sized>(game: &mut H) -> CoroutineRet {
    // The original state table, indexed and bounds-checked as before.
    let states: [fn(&mut MenuState, &mut H, &mut MenuScreens); 23] = [
        prepare::<H>,
        fade_palette::<H>,
        load_foreground::<H>,
        begin_background::<H>,
        begin_area_window::<H>,
        open_area_window::<H>,
        choose_area::<H>,
        begin_room_window::<H>,
        open_room_window::<H>,
        initialize_room::<H>,
        choose_room::<H>,
        show_room::<H>,
        show_room::<H>,
        fade_room::<H>,
        start_game::<H>,
        clear_room::<H>,
        restore_area::<H>,
        load_foreground::<H>,
        begin_background::<H>,
        begin_area_window::<H>,
        begin_return_window::<H>,
        close_room_window::<H>,
        fade_to_options::<H>,
    ];
    session_cohort::with(game, |game, (menu, screens): &mut MenuFrame| {
        states[menu.menu_index as usize](menu, game, screens)
    });
    super::pending_wait(game)
}
fn fade_to_options<H: Host + ?Sized>(
    menu: &mut MenuState,
    game: &mut H,
    screens: &mut MenuScreens,
) {
    draw_area_labels(screens, game);
    game.fade(false);
    if (game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf) == 0 {
        word!(game.GAME_STATE = K_GAME_STATE_2_GAME_OPTIONS_MENU as u16);
        menu.menu_index = 0;
    }
}
fn prepare<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, _: &mut MenuScreens) {
    menu_tilemap::clear(game);
    menu_tilemap::upload(
        menu_tilemap::MENU_TILEMAP_BYTES,
        menu_tilemap::MENU_TILEMAP_SOURCE,
        1,
        true,
        game,
    );
    word! {game; PALETTE_CHANGE_NUM = 0; }
    menu_tilemap::load_palette(game);
    crate::word_transfers!(game, ascending, 32; le(TARGET_PALETTES, 0) <- le(PALETTE_BUFFER, 0));
    word! { game, 0; TARGET_PALETTES[14], TARGET_PALETTES[30] }
    menu.menu_index = menu.menu_index.wrapping_add(1);
}
fn fade_palette<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, screens: &mut MenuScreens) {
    if crate::palette_effects::fading::file_select(game) != 0 {
        session_cohort::with(game, |_, scroll: &mut BgScroll| {
            scroll.bg1 = [0, 0];
            scroll.bg2 = [0, 0];
        });
        screens.save_selection.map_area_index = area_selection(game).unwrap_or(0) as u16;
        for area in 0..6 {
            menu_tilemap::area_palette(
                area,
                area == crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area,
                game,
            );
        }
        menu.menu_index = menu.menu_index.wrapping_add(1);
        screens.save_selection.map_unread = 0;
        game.ram().write_byte(
            crate::ram::mirror::TM,
            crate::display::control::Layers::BG2.byte(),
        );
    }
}
fn load_foreground<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, _: &mut MenuScreens) {
    menu_tilemap::upload(
        2048,
        0x810000 | ADDR_K_AREA_SELECT_FOREGROUND_TILEMAP,
        0,
        true,
        game,
    );
    set_native_word(
        0x51,
        ((game.ram().read_byte(crate::ram::mirror::OBSEL) as i32) << 8 | 0xf) as u16,
        game,
    );
    menu.menu_index = menu.menu_index.wrapping_add(1);
}
fn begin_background<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, _: &mut MenuScreens) {
    menu.menu_index = menu.menu_index.wrapping_add(1);
    load_background(
        crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area,
        game,
    );
}
fn load_background<H: Host + ?Sized>(j: u16, game: &mut H) {
    menu_tilemap::upload(
        2048,
        0x810000
            | u32::from(
                (2048 * i32::from(j) + ADDR_K_AREA_SELECT_BACKGROUND_TILEMAPS as i32) as u16,
            ),
        2,
        false,
        game,
    );
}
fn begin_area_window<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, _: &mut MenuScreens) {
    game.ram().write_byte(
        crate::ram::mirror::TS,
        crate::display::control::Layers::BG3.byte(),
    );
    crate::display::sequence::register_writes! { game;
        show_mirror(SubScreen(Layers::of(false, false, true, false, false)), 0x6d, false); show(SubWindowMask(Layers::of(false, false, true, false, false)));
        mirror(CGWSEL as u16, 2, 0x6f, true); mirror(CGADSUB as u16, 0x25, 0x72, true);
    }
    game.ram().write_byte(crate::ram::mirror::COLDATA + 2, 0x80);
    game.ram().write_byte(crate::ram::mirror::COLDATA + 1, 64);
    game.ram().write_byte(crate::ram::mirror::COLDATA, 32);
    super::menu_window::initialize(game);
    crate::display::sequence::write_register_and_mirror(game, HDMAEN as u16, 0xc, 0x85, true);
    configure_area_window(game);
    crate::hdma_objects::clear_hdma_flags(game);
    let index = (menu.menu_index as u8).wrapping_add(1);
    set_menu_byte(&mut menu.menu_index, 0, index);
}
fn configure_area_window<H: Host + ?Sized>(game: &mut H) {
    game.ram().write_byte(
        crate::ram::mirror::TM,
        crate::display::control::Layers::BG1_BG2_SPRITES.byte(),
    );
    crate::display::sequence::register_writes! { game;
        show_mirror(MainScreen(Layers::of(true, true, false, false, true)), 0x6c, false); show(MainWindowMask(Layers::of(true, true, false, false, true)));
        mirror(W12SEL as u16, 0x23, 0x60, true); mirror(W34SEL as u16, 3, 0x61, true);
        mirror(WOBJSEL as u16, 0x23, 0x62, true);
    }
}
fn open_area_window<H: Host + ?Sized>(
    menu: &mut MenuState,
    game: &mut H,
    screens: &mut MenuScreens,
) {
    draw_area_labels(screens, game);
    if super::menu_window::open_from_center(game) {
        let index = (menu.menu_index as u8).wrapping_add(1);
        set_menu_byte(&mut menu.menu_index, 0, index);
        game.ram().write_byte(
            crate::ram::mirror::TM,
            game.ram().read_byte(crate::ram::mirror::TM)
                & !crate::display::control::Layers::BG2.byte(),
        );
        game.ram().write_byte(
            crate::ram::mirror::TMW,
            crate::display::control::Layers::NONE.byte(),
        );
        game.ram().write_byte(
            crate::ram::mirror::TSW,
            crate::display::control::Layers::NONE.byte(),
        );
        session_cohort::with(game, |_, scroll: &mut BgScroll| scroll.bg2[1] = 24);
        menu.pausemenu_palette_animation_timer = 1;
        let mut _v1: u16 = 0;
        crate::word_transfers!(game, ascending, 800; xray(0) <- rom(0xb6e000, 0));
        _v1 = 800;
        crate::word_transfers!(game, ascending, 224; xray(800) <- value(10241));
        _v1 = 1024;
        menu_tilemap::area_label(game);
        crate::word_transfers!(game, descending, 160; xray(959) <- rom(0x81b14b, 160));
        menu_tilemap::upload(2048, 0x7e4000, 1, false, game);
    }
}
fn choose_area<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, screens: &mut MenuScreens) {
    let mut keys = game.observed_inputs().pressed as i16;
    'handle_action: {
        let previous = game.observed_inputs().pressed & (K_BUTTON_UP | K_BUTTON_LEFT) as u16 != 0;
        let attempts = if previous {
            3
        } else if game.observed_inputs().pressed
            & (K_BUTTON_SELECT | K_BUTTON_DOWN | K_BUTTON_RIGHT) as u16
            != 0
        {
            6
        } else {
            0
        };
        if attempts != 0 {
            keys = word!(game.ENABLE_DEBUG) as i16;
            if word!(game.ENABLE_DEBUG) != 0 {
                let mut selection = screens.save_selection.map_area_index;
                for _ in 0..attempts {
                    selection = if previous {
                        previous_area(selection)
                    } else {
                        next_area(selection)
                    };
                    if area_available(selection, game) != 0 {
                        if previous {
                            game.queue_sound(0, 0x37_u8, 6);
                        }
                        switch_area(selection, screens, game);
                        if !previous {
                            game.queue_sound(0, 0x37_u8, 6);
                        }
                        break 'handle_action;
                    }
                }
                break 'handle_action;
            }
        }
        if keys < 0 {
            menu.menu_index = 22;
            break 'handle_action;
        }
        if keys as i32 & (K_BUTTON_START as i32 | K_BUTTON_A as i32) != 0 {
            game.queue_sound(0, 0x38_u8, 6);
            select_area(screens, menu, game);
            return;
        }
    }
    draw_area_labels(screens, game);
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
    if (result as i32 - 6i32) & 0x8000 == 0 {
        return 0;
    }
    result
}
fn select_area<H: Host + ?Sized>(screens: &mut MenuScreens, menu: &mut MenuState, game: &mut H) {
    menu.menu_index = menu.menu_index.wrapping_add(1);
    if word!(game.ENABLE_DEBUG) == 0 {
        let selection = &mut screens.save_selection;
        crate::session_cohort::with(game, |_, room: &mut crate::room_view::RoomGeometry| {
            room.area = selection.save_area
        });
        selection.load_station = selection.save_station;
        draw_area_labels(screens, game);
        return;
    }
    let area = AREAS[usize::from(screens.save_selection.map_area_index)];
    crate::session_cohort::with(game, |_, room: &mut crate::room_view::RoomGeometry| {
        room.area = area
    });
    let mut bits = native_word(
        USED_STATIONS
            + 2 * usize::from(
                crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area,
            ),
        game,
    );
    let mut station = 0u16;
    let coordinates = station_coordinates(
        crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area,
        game,
    );
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
                game.unreachable_native();
            }
            unlocked_only = false;
        }
    }
    screens.save_selection.load_station = station;
    draw_area_labels(screens, game);
}
fn area_available<H: Host + ?Sized>(area: u16, game: &mut H) -> u16 {
    let station_word: i32 = 2 * AREAS[area as usize] as i32;
    if native_word(USED_STATIONS + station_word as usize, game) != 0 {
        return 1;
    }
    let first_station = game
        .native_image()
        .table_word(
            sm_data::cartridge::lorom_offset(0x82c80b) as usize,
            (station_word / 2) as isize,
        )
        .wrapping_add(64);
    (first_station != 0xffff) as u16
}
fn switch_area<H: Host + ?Sized>(area: u16, screens: &mut MenuScreens, game: &mut H) {
    let selection = &mut screens.save_selection;
    menu_tilemap::area_palette(AREAS[usize::from(selection.map_area_index)], false, game);
    selection.map_area_index = area;
    menu_tilemap::area_palette(AREAS[usize::from(area)], true, game);
    load_background(AREAS[usize::from(area)], game);
}
/// The current area's explored map tiles.
const EXPLORED: usize = crate::ram::bytes::MAP_TILES_EXPLORED;
/// Each area's used save station and elevator bits.
const USED_STATIONS: usize = crate::ram::bytes::USED_SAVE_STATIONS_AND_ELEVATORS;
/// The area select label (X, Y) of each area.
static LABEL_POSITIONS: [u16; 12] = [
    0x5b, 0x32, 0x2a, 0x7f, 0x5e, 0xb5, 0xce, 0x50, 0xce, 0x9f, 0x87, 0x8b,
];
fn draw_area_labels<H: Host + ?Sized>(screens: &MenuScreens, game: &mut H) {
    let mut palette: u16 = 0;
    crate::sprite_output::menu(
        table_word(
            &game.pause_menu_tables().label_spritemaps,
            game,
            LABEL_SPRITEMAP_ADDRESS,
            0,
        ),
        0x80,
        0x10,
        palette,
        game,
    );
    for i in 0i32..6 {
        palette = (if i == i32::from(screens.save_selection.map_area_index) {
            0
        } else {
            512
        }) as u16;
        let station_word: u16 = (2u16).wrapping_mul(AREAS[i as usize]);
        let mut unlocked: u16 = native_word(USED_STATIONS + station_word as usize, game);
        let mut station = station_coordinates(AREAS[i as usize], game);
        let mut remaining: i32 = 16;
        while station.word(0) != 0xffff {
            let is_unlocked: i32 = unlocked as i32 & 1;
            unlocked >>= 1;
            if !(is_unlocked != 0 && station.word(0) != 0xfffe) {
                station.base += 4;
                remaining -= 1;
                if remaining != 0 {
                    continue;
                }
                if !(word!(game.ENABLE_DEBUG) != 0 && station.word(0) != 0xffff) {
                    break;
                }
            }
            let position: i32 = (4 * AREAS[i as usize] as i32) >> 1;
            crate::sprite_output::menu(
                (table_word(
                    &game.pause_menu_tables().label_spritemaps,
                    game,
                    LABEL_SPRITEMAP_ADDRESS,
                    0,
                ) as i32
                    + AREAS[i as usize] as i32
                    + 1) as u16,
                LABEL_POSITIONS[position as usize],
                LABEL_POSITIONS[(position + 1) as usize],
                palette,
                game,
            );
            break;
        }
    }
}
static WINDOW_TIMERS: [u16; 6] = [0x33, 0x35, 0x2d, 0x33, 0x33, 0x22];
fn begin_room_window<H: Host + ?Sized>(
    menu: &mut MenuState,
    game: &mut H,
    screens: &mut MenuScreens,
) {
    draw_area_labels(screens, game);
    super::menu_window::initialize(game);
    game.ram().write_byte(
        crate::ram::mirror::TM,
        crate::display::control::Layers::BG1_BG2_SPRITES.byte(),
    );
    game.ram().write_byte(
        crate::ram::mirror::TMW,
        crate::display::control::Layers::BG1_BG2_SPRITES.byte(),
    );
    crate::display::sequence::register_writes! { game;
        mirror(W12SEL as u16, 0x32, 0x60, true); mirror(W34SEL as u16, 2, 0x61, true);
        mirror(CGADSUB as u16, 5, 0x72, true); mirror(WOBJSEL as u16, 0x22, 0x62, true);
    }
    game.ram().write_byte(crate::ram::mirror::BG12NBA, 48);
    // Close both windows' middle line counts.
    game.ram()
        .write_byte(super::menu_window::ram::WINDOW_1_TABLE + 9, 0);
    game.ram()
        .write_byte(super::menu_window::ram::WINDOW_2_TABLE + 9, 0);
    menu_tilemap::area_label(game);
    menu_tilemap::upload(512, 0x7e4000, 1, false, game);
    word! {game; EXPAND_SQ_TIMER = WINDOW_TIMERS[crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area as usize]; }
    let label: i32 = ((4u16)
        .wrapping_mul(crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area))
        as i32
        >> 1;
    word!(game.EXPAND_SQ_LEFT_POS = LABEL_POSITIONS[label as usize]);
    word!(game.EXPAND_SQ_RIGHT_POS = word!(game.EXPAND_SQ_LEFT_POS));
    word!(game.EXPAND_SQ_TOP_POS = LABEL_POSITIONS[(label + 1) as usize]);
    word!(game.EXPAND_SQ_BOTTOM_POS = word!(game.EXPAND_SQ_TOP_POS));
    word! { game, 0;
        EXPAND_SQ_LEFT_SUBPOS, EXPAND_SQ_RIGHT_SUBPOS, EXPAND_SQ_TOP_SUBPOS,
        EXPAND_SQ_BOTTOM_SUBPOS,
    }
    super::menu_window::load_velocities(
        crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area,
        game,
    );
    super::menu_window::project(game);
    crate::display::sequence::write_register_and_mirror(game, HDMAEN as u16, 0xc, 0x85, true);
    menu.menu_index = menu.menu_index.wrapping_add(1);
    game.queue_sound(0, 0x3b_u8, 6);
}
fn open_room_window<H: Host + ?Sized>(
    menu: &mut MenuState,
    game: &mut H,
    screens: &mut MenuScreens,
) {
    super::menu_window::advance(super::menu_window::Motion::Expand, game);
    if (word!(game.EXPAND_SQ_TIMER -= 1, reread) & 0x8000) != 0 {
        menu.menu_index = menu.menu_index.wrapping_add(1);
        game.ram().write_byte(
            crate::ram::mirror::TM,
            crate::display::control::Layers::BG2.byte(),
        );
        game.ram().write_byte(
            crate::ram::mirror::TMW,
            crate::display::control::Layers::NONE.byte(),
        );
        game.ram().write_byte(
            crate::ram::mirror::TSW,
            crate::display::control::Layers::NONE.byte(),
        );
        game.ram().write_byte(
            crate::ram::mirror::TS,
            crate::display::control::Layers::NONE.byte(),
        );
    }
    draw_area_labels(screens, game);
}
fn initialize_room<H: Host + ?Sized>(
    menu: &mut MenuState,
    game: &mut H,
    screens: &mut MenuScreens,
) {
    load_area_map(&mut screens.save_selection, game);
    draw_room_map(game);
    crate::room_entry::selection::begin(
        &mut game.runtime(),
        crate::room_entry::api::ArrivalRequest::Start(
            crate::room_entry::api::StartSite::CurrentLoadStation,
        ),
    );
    crate::hdma_objects::set_hdma_enabled(false, game);
    crate::hdma_objects::clear_hdma(game);
    let header = crate::projection::word(game, crate::ram::words::ROOM_PTR);
    let room = menu_tilemap::native_record(0x8f0000 | u32::from(header), game);
    // The area and map position take only their low bytes from the header.
    let [area, x, y] = [1, 2, 3].map(|at| u16::from(room.byte(at)));
    crate::session_cohort::with(
        game,
        |_,
         (geometry, effects): &mut (
            crate::room_view::RoomGeometry,
            crate::room_view::RoomEffects,
        )| {
            geometry.area = (geometry.area & 0xff00) | area;
            effects.map_x = (effects.map_x & 0xff00) | x;
            effects.map_y = (effects.map_y & 0xff00) | y;
        },
    );
    // The room selection above stores BG1's scroll, so the copy starts here.
    session_cohort::with(game, |game, scroll: &mut BgScroll| {
        reset_animations(&mut screens.map, menu, scroll, game);
        scroll_limits(screens, game);
        center_map(112, &screens.map, &mut scroll.bg1, game);
        scroll.bg2[1] = 24;
    });
    let map = &mut screens.map;
    map.min_y_scroll = map.min_y_scroll.wrapping_add(24);
    crate::ram::checked_word!(game, 0x69; = crate::ram::checked_word!(game, 0x69;) & !4);
    menu.menu_index = menu.menu_index.wrapping_add(1);
    map.scrolling_direction = 0;
    map.scrolling_speed_index = 0;
    map.indicator_frame = 0;
    map.indicator_timer = 0;
    map.indicator_loop_counter = 0;
}
fn choose_room<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, screens: &mut MenuScreens) {
    session_cohort::with(game, |game, scroll: &mut BgScroll| {
        choose_room_on(menu, game, screens, &mut scroll.bg1)
    });
}
/// [`choose_room`] on BG1's scroll.
fn choose_room_on<H: Host + ?Sized>(
    menu: &mut MenuState,
    game: &mut H,
    screens: &mut MenuScreens,
    bg1: &mut [u16; 2],
) {
    draw_icons(IconMode::FileSelect, screens, menu, *bg1, game);
    scroll_arrows(MapScreen::FileSelect, &mut screens.map, menu, *bg1, game);
    scroll(&mut screens.map, bg1, game);
    display_elevator_destinations(*bg1, game);
    if word!(game.ENABLE_DEBUG) != 0
        && (word!(game.JOYPAD2_NEW_KEYS) as i32 & K_BUTTON_SELECT as i32) != 0
    {
        game.queue_sound(0, 0x38_u8, 6);
        let mut unlocked: u16 = native_word(
            USED_STATIONS
                + 2 * usize::from(
                    crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area,
                ),
            game,
        );
        let selection = &mut screens.save_selection;
        let mut remaining: i16 = selection.load_station as i16;
        do_until! { remaining < 0 ;
            unlocked >>= 1;
            remaining -= 1;
        }
        let stations = station_coordinates(
            crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area,
            game,
        );
        let station_offset: i16 = (4i16).wrapping_mul(selection.load_station as i16);
        let station_x: u16 = stations.table_word((station_offset >> 1) as isize);
        let station_y: u16 = stations
            .image
            .table_word(stations.base, ((station_offset as i32 >> 1) + 1) as isize);
        let mut check_unlocked = selection.load_station.wrapping_sub(16) & 0x8000 != 0;
        loop {
            selection.load_station = selection.load_station.wrapping_add(1);
            let inspect_terminal =
                !check_unlocked || selection.load_station.wrapping_sub(16) & 0x8000 == 0;
            if inspect_terminal {
                let terminator: i16 = stations.image.table_word(
                    stations.base,
                    (2 * i32::from(selection.load_station)) as isize,
                ) as i16;
                if terminator == -1i16 {
                    selection.load_station = 0;
                    unlocked = native_word(
                        USED_STATIONS
                            + 2 * usize::from(
                                crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(
                                    game,
                                )
                                .area,
                            ),
                        game,
                    );
                } else if terminator != -2i16 {
                    break;
                } else {
                    check_unlocked = false;
                    continue;
                }
            }
            let is_unlocked: i8 = ((unlocked) as i8) & (1i8);
            unlocked >>= 1;
            if is_unlocked == 0
                || stations.image.table_word(
                    stations.base,
                    (2 * i32::from(selection.load_station)) as isize,
                ) >= 0xfffe
            {
                check_unlocked = true;
            } else {
                break;
            }
        }
        let next_offset: u16 = (4u16).wrapping_mul(selection.load_station);
        let next_x: u16 = stations.image.table_word(
            stations.base,
            (2 * i32::from(selection.load_station)) as isize,
        );
        if (next_x as i32 - bg1[0] as i32) & 0x8000 != 0
            || (next_x as i32 - 256i32 - bg1[0] as i32) & 0x8000 == 0
        {
            let mut scroll_x: i16 = (bg1[0] as i32
                + stations.table_word((next_offset >> 1) as isize) as i32
                - station_x as i32) as i16;
            if scroll_x >= 0 {
                if (scroll_x).wrapping_sub(screens.map.min_x_scroll as i16) >= 0 {
                    scroll_x = screens.map.min_x_scroll as i16;
                }
            } else {
                scroll_x = 0;
            }
            bg1[0] = scroll_x as u16;
        }
        let next_y_offset: u16 = (next_offset).wrapping_add(2);
        let next_y: u16 = stations.table_word((next_y_offset >> 1) as isize);
        if (next_y as i32 - bg1[1] as i32) & 0x8000 != 0
            || (next_y as i32 - 161i32 - bg1[1] as i32) & 0x8000 == 0
        {
            let mut scroll_y: u16 = (bg1[1] as i32
                + stations.table_word((next_y_offset >> 1) as isize) as i32
                - station_y as i32) as u16;
            if ((scroll_y) as i16).wrapping_sub(screens.map.min_y_scroll as i16) >= 0 {
                scroll_y = screens.map.min_y_scroll;
            }
            bg1[1] = scroll_y;
        }
    } else if (game.observed_inputs().pressed & 0x8000) != 0 {
        menu.menu_index = menu.menu_index.wrapping_add(5);
        let Some(selection) = area_selection(game) else {
            screens.save_selection.map_area_index = 0;
            return;
        };
        screens.save_selection.map_area_index = selection as u16;
        game.queue_sound(0, 0x3c_u8, 6);
    } else if game.observed_inputs().pressed as i32 & (K_BUTTON_START as i32 | K_BUTTON_A as i32)
        != 0
    {
        menu.menu_index = menu.menu_index.wrapping_add(1);
        game.queue_sound(0, 0x38_u8, 6);
    }
}
fn show_room<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, screens: &mut MenuScreens) {
    let bg1 = session_cohort::load::<BgScroll, _>(game).bg1;
    draw_icons(IconMode::FileSelect, screens, menu, bg1, game);
    display_elevator_destinations(bg1, game);
    menu.menu_index = menu.menu_index.wrapping_add(1);
}
fn fade_room<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, screens: &mut MenuScreens) {
    let bg1 = session_cohort::load::<BgScroll, _>(game).bg1;
    draw_icons(IconMode::FileSelect, screens, menu, bg1, game);
    display_elevator_destinations(bg1, game);
    game.fade(false);
    if (game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf) == 0 {
        menu.menu_index = menu.menu_index.wrapping_add(1);
        set_native_word(enemy_data::FIRST + enemy_data::X_POS, 32, game);
    }
}
fn start_game<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, _: &mut MenuScreens) {
    if !super::resuming(game) {
        let timer = enemy_data::FIRST + enemy_data::X_POS;
        set_native_word(timer, native_word(timer, game).wrapping_sub(1), game);
        if native_word(timer, game) != 0 {
            return;
        }
    }
    if !super::blank_and_wait(true, game) {
        word!(game.GAME_STATE += 1);
        menu.menu_index = 0;
    }
}
fn clear_room<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, _: &mut MenuScreens) {
    game.ram().write_byte(
        crate::ram::mirror::TM,
        crate::display::control::Layers::BG2_SPRITES.byte(),
    );
    crate::word_transfers!(game, descending, 1024; native(0x37fe) <- value(15));
    menu_tilemap::upload(2048, 0x7e3000, 0, true, game);
    menu.menu_index = menu.menu_index.wrapping_add(1);
}
fn restore_area<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, _: &mut MenuScreens) {
    menu_tilemap::load_palette(game);
    for area in 0..6 {
        menu_tilemap::area_palette(area, false, game);
    }
    menu_tilemap::area_palette(
        crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area,
        true,
        game,
    );
    load_background(
        crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area,
        game,
    );
    menu.menu_index = menu.menu_index.wrapping_add(1);
}
fn begin_return_window<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, _: &mut MenuScreens) {
    game.ram().write_byte(crate::ram::mirror::HDMAEN, 0);
    game.queue_sound(0, 0x3c_u8, 6);
    word! { game;
        EXPAND_SQ_TIMER =
            (WINDOW_TIMERS[crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area as usize]).wrapping_sub(12);
    }
    super::menu_window::load_velocities(
        crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area,
        game,
    );
    word! { game;
        EXPAND_SQ_LEFT_POS = 8;
        EXPAND_SQ_RIGHT_POS = 248;
        EXPAND_SQ_TOP_POS = 8;
        EXPAND_SQ_BOTTOM_POS = 216;
    }
    word! { game, 0;
        EXPAND_SQ_LEFT_SUBPOS, EXPAND_SQ_RIGHT_SUBPOS, EXPAND_SQ_TOP_SUBPOS,
        EXPAND_SQ_BOTTOM_SUBPOS,
    }
    super::menu_window::project(game);
    menu.menu_index = menu.menu_index.wrapping_add(1);
    crate::display::sequence::register_writes! { game;
        mirror(HDMAEN as u16, 0xc, 0x85, true); mirror(W12SEL as u16, 0x32, 0x60, true);
        mirror(W34SEL as u16, 2, 0x61, true);
    }
    game.ram().write_byte(crate::ram::mirror::BG12NBA, 48);
    game.ram().write_byte(crate::ram::mirror::WOBJSEL, 34);
    // Close both windows' middle line counts.
    game.ram()
        .write_byte(super::menu_window::ram::WINDOW_1_TABLE + 9, 0);
    game.ram()
        .write_byte(super::menu_window::ram::WINDOW_2_TABLE + 9, 0);
    session_cohort::with(game, |_, scroll: &mut BgScroll| scroll.bg1 = [0, 0]);
}
fn close_room_window<H: Host + ?Sized>(
    menu: &mut MenuState,
    game: &mut H,
    screens: &mut MenuScreens,
) {
    super::menu_window::advance(super::menu_window::Motion::Contract, game);
    draw_area_labels(screens, game);
    if (word!(game.EXPAND_SQ_TIMER -= 1, reread) & 0x8000) != 0 {
        menu.menu_index = menu.menu_index.wrapping_sub(15);
        game.ram().write_byte(
            crate::ram::mirror::TM,
            crate::display::control::Layers::BG1_SPRITES.byte(),
        );
        game.ram().write_byte(
            crate::ram::mirror::TMW,
            crate::display::control::Layers::NONE.byte(),
        );
        game.ram().write_byte(
            crate::ram::mirror::TSW,
            crate::display::control::Layers::NONE.byte(),
        );
    }
}

fn station_coordinates<H: Host + ?Sized>(area: u16, game: &mut H) -> sm_data::cartridge::RomRecord {
    let pointer = menu_tilemap::native_record(
        0x820000
            | u32::from((ADDR_K_MAP_ICON_DATA_POINTERS as i32 + 64 + 2 * i32::from(area)) as u16),
        game,
    )
    .word(0);
    menu_tilemap::native_record(0x820000 | u32::from(pointer), game)
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

fn area_selection<H: Host + ?Sized>(game: &H) -> Option<usize> {
    AREAS.iter().position(|&area| {
        area == crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area
    })
}

const SCROLL_SPEEDS: [u16; 32] = [
    0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Advance the shared file-select/pause-map scroll of BG1.
pub fn scroll<H: Host + ?Sized>(map: &mut MapScreenState, bg1: &mut [u16; 2], game: &mut H) {
    // Preserve the dispatch table's bounds failure before touching scroll state.
    let direction = [0, 1, 2, 3, 4][usize::from(map.scrolling_direction)];
    if direction == 0 {
        map.gear_switch_timer = 4;
        return;
    }
    let mut index = map.scrolling_speed_index;
    if map.gear_switch_timer == 0 {
        index = index.wrapping_add(32);
    }
    let speed = SCROLL_SPEEDS[usize::from(index >> 1)];
    match direction {
        1 => bg1[0] = bg1[0].wrapping_sub(speed),
        2 => bg1[0] = bg1[0].wrapping_add(speed),
        3 => bg1[1] = bg1[1].wrapping_sub(speed),
        _ => bg1[1] = bg1[1].wrapping_add(speed),
    };
    map.scrolling_speed_index = map.scrolling_speed_index.wrapping_add(2);
    if map.scrolling_speed_index & 0xf == 0 {
        game.queue_sound(0, 0x36_u8, 6);
        map.scrolling_direction = 0;
        map.scrolling_speed_index = 0;
        if map.gear_switch_timer != 0 {
            map.gear_switch_timer -= 1;
        }
    }
}

fn map_record<H: Host + ?Sized>(cursor: u16, game: &mut H) -> sm_data::cartridge::RomRecord {
    menu_tilemap::native_record(0x820000 | u32::from(cursor), game)
}

/// Draw `record`'s sprite at its map position less BG1's scroll `bg1`.
fn draw_record<H: Host + ?Sized>(
    sprite: u16,
    record: &sm_data::cartridge::RomRecord,
    palette: u16,
    bg1: [u16; 2],
    game: &mut H,
) {
    crate::sprite_output::menu(
        sprite,
        record.word(0).wrapping_sub(bg1[0]),
        record.word(2).wrapping_sub(bg1[1]),
        palette,
        game,
    );
}

#[derive(Clone, Copy)]
pub enum IconMode {
    Pause,
    FileSelect,
}

pub fn draw_icons<H: Host + ?Sized>(
    mode: IconMode,
    screens: &mut MenuScreens,
    menu: &mut MenuState,
    bg1: [u16; 2],
    game: &mut H,
) {
    if matches!(mode, IconMode::FileSelect) {
        animate_palette(menu, game);
    }
    boss_icons(9, 0xc7cb, screens.save_selection.has_area_map, bg1, game);
    for (sprite, table) in [(0xb, 0xc7db), (0xa, 0xc7eb), (0x4e, 0xc7fb)] {
        icon_group(sprite, table, u16::MAX, 3584, bg1, game);
    }
    match mode {
        IconMode::Pause => {
            if crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area
                == screens.save_selection.save_area
            {
                let bits = station_bits(0, game);
                let mut bytes = bits.to_ne_bytes();
                bytes[0] = bits as u8
                    & [1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80]
                        [usize::from(screens.save_selection.load_station)];
                icon_group(8, 0xc80b, u16::from_ne_bytes(bytes), 1024, bg1, game);
            }
            if word!(game.ENABLE_DEBUG) != 0 {
                icon_group(8, 0xc82b, u16::MAX, 1024, bg1, game);
            }
        }
        IconMode::FileSelect => {
            let sprite = advance_marker(&mut screens.map);
            // This fixed table is native and linear; only its result is bank-wrapped.
            let address = game.native_image().table_word(
                sm_data::cartridge::lorom_offset(0x82c80b) as usize,
                crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area
                    as isize,
            );
            let record = map_record(address, game);
            let offset = 4 * usize::from(screens.save_selection.load_station);
            let x = record.word(offset).wrapping_sub(bg1[0]);
            let y = record.word(offset + 2).wrapping_sub(bg1[1]);
            if screens.map.indicator_loop_counter & 1 == 0 {
                crate::sprite_output::menu(0x12, x, y, 3584, game);
            }
            crate::sprite_output::menu(sprite, x, y, 3584, game);
            if word!(game.ENABLE_DEBUG) != 0 {
                let saves = station_bits(0, game);
                icon_group(0xc, 0xc80b, saves, 1536, bg1, game);
                let elevators = station_bits(1, game);
                icon_group(0x17, 0xc81b, elevators, 1536, bg1, game);
                icon_group(0xc, 0xc82b, u16::MAX, 1536, bg1, game);
            }
        }
    }
    if crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area == 0 {
        crate::sprite_output::menu(
            0x63,
            0xd8u16.wrapping_sub(bg1[0]),
            0x28u16.wrapping_sub(bg1[1]),
            3584,
            game,
        );
    }
}

fn station_bits<H: Host + ?Sized>(offset: usize, game: &mut H) -> u16 {
    let address = USED_STATIONS
        + 2 * usize::from(
            crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area,
        )
        + offset;
    u16::from_ne_bytes([
        game.ram().read_byte(address),
        game.ram().read_byte(address + 1),
    ])
}

fn icon_group<H: Host + ?Sized>(
    sprite: u16,
    table: u16,
    mut bits: u16,
    palette: u16,
    bg1: [u16; 2],
    game: &mut H,
) {
    let mut cursor = map_record(
        (i32::from(table)
            + 2 * i32::from(
                crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area,
            )) as u16,
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
            draw_record(sprite, &record, palette, bg1, game);
        }
        cursor = cursor.wrapping_add(4);
    }
}

fn explored<H: Host + ?Sized>(x: u16, y: u16, game: &mut H) -> bool {
    let offset =
        usize::from((x & 0xff00) >> 1) + usize::from(x as u8 >> 6) + usize::from((y & 0xfff8) >> 1);
    game.ram()
        .read_byte(crate::ram::bytes::MAP_TILES_EXPLORED + offset)
        & (0x80 >> ((x >> 3) & 7))
        != 0
}

fn boss_icons<H: Host + ?Sized>(
    sprite: u16,
    table: u16,
    has_area_map: u16,
    bg1: [u16; 2],
    game: &mut H,
) {
    let mut bits = crate::progress::leased::boss_byte_at(
        crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area,
        game,
    ) as i32;
    let cursor = map_record(
        (i32::from(table)
            + 2 * i32::from(
                crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area,
            )) as u16,
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
                    draw_record(0x62, &record, 3584, bg1, game);
                    draw_record(sprite, &record, 3072, bg1, game);
                } else if has_area_map != 0 {
                    draw_record(sprite, &record, 3584, bg1, game);
                } else {
                    // An undisplayed undefeated boss consumes a second bit.
                    bits >>= 1;
                }
            }
        }
        record.base += 4;
    }
}

pub fn animate_palette<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    if menu.pausemenu_palette_animation_timer as u8 == 0 {
        return;
    }
    let timer = (menu.pausemenu_palette_animation_timer as u8).wrapping_sub(1);
    set_menu_byte(&mut menu.pausemenu_palette_animation_timer, 0, timer);
    if menu.pausemenu_palette_animation_timer as u8 != 0 {
        return;
    }
    let mut frame = i32::from(menu.pausemenu_palette_animation_frame) + 1;
    loop {
        set_menu_byte(&mut menu.pausemenu_palette_animation_frame, 0, frame as u8);
        let timer = game
            .native_image()
            .read(sm_data::cartridge::lorom_offset(0x82c10c) as usize + 3 * frame as usize);
        if timer != 0xff {
            set_menu_byte(&mut menu.pausemenu_palette_animation_timer, 0, timer);
            break;
        }
        game.queue_sound(2, 0x2a_u8, 6);
        frame = 0;
    }
    let frame_word = u16::from_ne_bytes([0, menu.pausemenu_palette_animation_frame as u8]);
    let mut source = (frame_word >> 3) + 30;
    for index in (176..192).rev() {
        let color = grid_word(
            &game.pause_menu_tables().palette_animation,
            game,
            PALETTE_ANIMATION_ADDRESS,
            (source >> 1) as isize,
        );
        crate::word_transfers!(game, ascending, 1; native(crate::ram::word_arrays::PALETTE_BUFFER + 2 * index) <- value(color));
        source = source.wrapping_sub(2);
    }
}

fn advance_marker(map: &mut MapScreenState) -> u16 {
    const DELAYS: [u16; 4] = [8, 4, 8, 4];
    const SPRITES: [u16; 4] = [0x5f, 0x60, 0x61, 0x60];
    if map.indicator_timer == 0 {
        let mut frame = map.indicator_frame.wrapping_add(2);
        if (frame as i16).wrapping_sub(8) >= 0 {
            map.indicator_loop_counter = map.indicator_loop_counter.wrapping_add(1);
            frame = 0;
        }
        map.indicator_frame = frame;
        map.indicator_timer = DELAYS[usize::from(frame >> 1)];
    }
    map.indicator_timer = map.indicator_timer.wrapping_sub(1);
    SPRITES[usize::from(map.indicator_frame >> 1)]
}

pub fn draw_player_marker<H: Host + ?Sized>(map: &mut MapScreenState, bg1: [u16; 2], game: &mut H) {
    let sprite = advance_marker(map);
    crate::sprite_output::menu(
        sprite,
        (8 * (i32::from(
            crate::session_cohort::load::<crate::room_view::RoomEffects, _>(game).map_x,
        ) + i32::from(game.ram().read_byte(crate::ram::words::SAMUS_X_POS + 1)))
            - i32::from(bg1[0])) as u16,
        (8 * (i32::from(
            crate::session_cohort::load::<crate::room_view::RoomEffects, _>(game).map_y,
        ) + i32::from(game.ram().read_byte(crate::ram::words::SAMUS_Y_POS + 1))
            + 1)
            - i32::from(bg1[1])) as u16,
        3584,
        game,
    );
}

pub fn display_elevator_destinations<H: Host + ?Sized>(bg1: [u16; 2], game: &mut H) {
    if game.ram().read_byte(
        crate::ram::bytes::MAP_STATION_BYTE_ARRAY
            + usize::from(
                crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area,
            ),
    ) == 0
    {
        return;
    }
    let mut cursor = i32::from(table_word(
        &game.pause_menu_tables().map_icons,
        game,
        MAP_ICON_ADDRESS,
        crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area as isize,
    ));
    loop {
        let record = map_record(cursor as u16, game);
        if record.word(0) == 0xffff {
            break;
        }
        draw_record(record.word(4), &record, 0, bg1, game);
        cursor += 6;
    }
}

pub fn reset_animations<H: Host + ?Sized>(
    map: &mut MapScreenState,
    menu: &mut MenuState,
    scroll: &mut BgScroll,
    game: &mut H,
) {
    menu.pausemenu_reserve_tank_delay_ctr = 0;
    scroll.bg1[0] = 0;
    scroll.bg2 = [0, 0];
    scroll.bg3 = [0, 0];
    menu.pausemenu_button_label_mode = 0;
    menu.pausemenu_lr_animation_frame = 0;
    menu.unused_word_7e0745 = 0;
    map.indicator_frame = 0;
    map.indicator_timer = 0;
    map.indicator_loop_counter = 0;
    menu.pausemenu_lr_animation_timer = game
        .native_image()
        .table_word(sm_data::cartridge::lorom_offset(0x82c10c) as usize, 0);
    menu.pausemenu_palette_animation_timer = 1;
    menu.pausemenu_palette_animation_frame = 0;
}

/// Center BG1's scroll `bg1` on Samus's map position.
pub fn center_map<H: Host + ?Sized>(
    vertical_center: u16,
    map: &MapScreenState,
    bg1: &mut [u16; 2],
    game: &mut H,
) {
    bg1[0] = (i32::from(map.min_x_scroll)
        + (i32::from(map.max_x_scroll.wrapping_sub(map.min_x_scroll)) >> 1)
        - 128) as u16;
    let x = (8
        * (i32::from(crate::session_cohort::load::<crate::room_view::RoomEffects, _>(game).map_x)
            + i32::from((game.player_x() & 0xff00) >> 8))
        - i32::from(bg1[0])) as u16;
    let right = 224i16.wrapping_sub(x as i16);
    if right >= 0 {
        let left = 32u16.wrapping_sub(x);
        if left & 0x8000 == 0 {
            bg1[0] = bg1[0].wrapping_sub(left);
        }
    } else {
        bg1[0] = bg1[0].wrapping_sub(right as u16);
    }
    let middle = (i32::from(map.min_y_scroll)
        + (i32::from(map.max_y_scroll.wrapping_sub(map.min_y_scroll)) >> 1)
        + 16) as u16;
    let aligned = (i32::from(vertical_center) - i32::from(middle)) & 0xfff8;
    bg1[1] = (-aligned) as u16;
    let y = (8
        * (i32::from(crate::session_cohort::load::<crate::room_view::RoomEffects, _>(game).map_y)
            + i32::from(game.ram().read_byte(crate::ram::words::SAMUS_Y_POS + 1))
            + 1)
        + aligned) as u16;
    let top = 64i16.wrapping_sub(y as i16);
    if top >= 0 {
        bg1[1] = bg1[1].wrapping_sub(top as u16);
        if (i32::from(bg1[1]) + 40) & 0x8000 != 0 {
            bg1[1] = -40i32 as u16;
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

fn find_map_edge<H: Host + ?Sized>(
    source: Option<&sm_data::cartridge::RomRecord>,
    edge: MapEdge,
    game: &mut H,
) -> u16 {
    let read = |offset: usize| match source {
        Some(record) => record.byte(offset),
        None => game
            .ram()
            .read_byte(crate::ram::bytes::MAP_TILES_EXPLORED + offset),
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

pub fn scroll_limits<H: Host + ?Sized>(screens: &mut MenuScreens, game: &mut H) {
    let source = if screens.save_selection.has_area_map != 0 {
        let pointer = map_record(
            (i32::from(ADDR_K_PAUSE_MENU_MAP_DATA as u16)
                + 2 * i32::from(
                    crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area,
                )) as u16,
            game,
        )
        .word(0);
        Some(map_record(pointer, game))
    } else {
        None
    };
    let map = &mut screens.map;
    map.min_x_scroll = find_map_edge(source.as_ref(), MapEdge::Left, game).wrapping_mul(8);
    if crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area == 4 {
        map.min_x_scroll = map.min_x_scroll.wrapping_sub(24);
    }
    map.max_x_scroll = find_map_edge(source.as_ref(), MapEdge::Right, game).wrapping_mul(8);
    map.min_y_scroll = find_map_edge(source.as_ref(), MapEdge::Top, game).wrapping_mul(8);
    map.max_y_scroll = find_map_edge(source.as_ref(), MapEdge::Bottom, game).wrapping_mul(8);
}

pub enum MapScreen {
    Pause,
    FileSelect,
}

pub fn render_map<H: Host + ?Sized>(screen: MapScreen, game: &mut H) {
    let mut area = crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area;
    if (i32::from(crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area) - 7)
        & 0x8000
        == 0
    {
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
            &game.pause_menu_tables().area_maps,
            game,
            AREA_MAP_ADDRESS,
            area as isize,
        ),
        game,
    );
    if game.ram().read_byte(
        crate::ram::bytes::MAP_STATION_BYTE_ARRAY
            + usize::from(
                crate::session_cohort::load::<crate::room_view::RoomGeometry, _>(game).area,
            ),
    ) != 0
    {
        let mut mapped = bitmap.word(0).swap_bytes();
        let mut explored = native_word(EXPLORED, game).swap_bytes();
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
                explored = native_word(EXPLORED + offset, game).swap_bytes();
            }
        }
    } else {
        for index in 0..2048 {
            let byte = index / 8;
            let explored = 0x7f7 + byte;
            let previous = game.ram().read_byte(explored);
            game.ram()
                .write_byte(explored, game.ram().read_byte(explored).wrapping_mul(2));
            game.observe_map_write(byte, index, 0);
            let value = if previous & 0x80 == 0 {
                empty
            } else {
                game.ram()
                    .write_byte(explored, game.ram().read_byte(explored).wrapping_add(1));
                game.observe_map_write(byte, index, 1);
                tiles.word(2 * index) & 0xfbff
            };
            set_native_word(destination + 2 * index, value, game);
            game.observe_map_write(byte, index, 2);
        }
    }
}

fn draw_room_map<H: Host + ?Sized>(game: &mut H) {
    game.ram().write_byte(crate::ram::mirror::BG12NBA, 51);
    game.ram().write_byte(
        crate::ram::mirror::TM,
        crate::display::control::Layers::BG1_BG2_SPRITES.byte(),
    );
    session_cohort::with(game, |_, scroll: &mut BgScroll| {
        scroll.bg1[1] = -40i16 as u16
    });
    render_map(MapScreen::FileSelect, game);
    menu_tilemap::upload(4096, 0x7e3000, 0, false, game);
}

// The nine-entry union tables capture their native image before checking the index.
fn animation_entry<H: Host + ?Sized>(table: u32, index: usize, game: &mut H) -> u16 {
    let record = menu_tilemap::native_record(table, game);
    let offset = [0, 2, 4, 6, 8, 10, 12, 14, 16][index];
    record.word(offset)
}

fn animation_ram_address<H: Host + ?Sized>(table: u32, index: usize, game: &mut H) -> usize {
    let address = animation_entry(table, index, game);
    if address >= 0x2000 {
        game.legacy_assertion(true);
    }
    usize::from(address)
}

/// The word at RAM address `at` the pause sprite animations name: the menu
/// state's while one starts there, RAM otherwise.
fn animation_word<H: Host + ?Sized>(at: usize, menu: &mut MenuState, game: &H) -> u16 {
    match menu.word_at(at) {
        Some(word) => *word,
        None => native_word(at, game),
    }
}

fn set_animation_word<H: Host + ?Sized>(at: usize, value: u16, menu: &mut MenuState, game: &mut H) {
    match menu.word_at(at) {
        Some(word) => *word = value,
        None => set_native_word(at, value, game),
    }
}

pub fn animate_pause_sprite<H: Host + ?Sized>(
    animation: u16,
    x: u16,
    y: u16,
    menu: &mut MenuState,
    game: &mut H,
) {
    let index = (i32::from(animation) - 1) as usize;
    let frame = animation_ram_address(0x82c0c4, index, game);
    let timer = animation_ram_address(0x82c0b2, index, game);
    let script = map_record(animation_entry(0x82c0e8, index, game), game);
    let ticked = animation_word(timer, menu, game).wrapping_sub(1);
    set_animation_word(timer, ticked, menu, game);
    if ticked as i16 <= 0 {
        let next = animation_word(frame, menu, game).wrapping_add(1);
        set_animation_word(frame, next, menu, game);
        let mut duration = u16::from(script.byte(3 * usize::from(next)));
        if duration == 255 {
            set_animation_word(frame, 0, menu, game);
            duration = u16::from(script.byte(0));
        }
        set_animation_word(timer, duration, menu, game);
    }
    let palette = animation_palette(game, 3);
    let current = animation_word(frame, menu, game);
    let delta = u16::from(script.byte(3 * usize::from(current) + 2));
    let variant = animation_ram_address(0x82c0d6, index, game);
    // The variant is one byte: the low or high byte of the word it falls in.
    let variant = if (MENU..MENU + 2 * WORDS).contains(&variant) {
        let lane = (variant - MENU) % 2;
        menu.word_at(variant - lane)
            .map_or(0, |word| word.to_le_bytes()[lane])
    } else {
        game.ram().read_byte(variant)
    };
    let variant = 2 * u16::from(variant);
    let sprite = map_record(
        table_word(
            &game.pause_menu_tables().animation_spritemaps,
            game,
            ANIMATION_SPRITEMAP_ADDRESS,
            index as isize,
        )
        .wrapping_add(variant),
        game,
    )
    .word(0);
    crate::sprite_output::menu(
        delta.wrapping_add(sprite),
        x,
        y.wrapping_sub(1),
        palette,
        game,
    );
}

pub fn draw_equipment_selector<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    if crate::session_cohort::load::<crate::inventory::Inventory, _>(game).reserve_capacity
        | crate::session_cohort::load::<crate::inventory::Inventory, _>(game)
            .equipment
            .owned
        | crate::session_cohort::load::<crate::inventory::Inventory, _>(game)
            .beams
            .owned
        == 0
    {
        return;
    }
    let address = table_word(
        &game.pause_menu_tables().selector_positions,
        game,
        SELECTOR_POSITION_ADDRESS,
        menu.pausemenu_equipment_category_item as u8 as isize,
    )
    .wrapping_add(4 * u16::from(menu.pausemenu_equipment_category_item.to_le_bytes()[1]));
    let position = map_record(address, game);
    animate_pause_sprite(
        3,
        position.word(0).wrapping_sub(1),
        position.word(2),
        menu,
        game,
    );
}

fn scroll_arrow<H: Host + ?Sized>(
    address: u32,
    map: &mut MapScreenState,
    menu: &mut MenuState,
    game: &mut H,
) {
    let record = menu_tilemap::native_record(address, game);
    animate_pause_sprite(record.word(4), record.word(0), record.word(2), menu, game);
    // Drawing can replace the image; input and direction use a fresh capture.
    let record = menu_tilemap::native_record(address, game);
    if game.observed_inputs().held & record.word(6) != 0 && map.scrolling_direction == 0 {
        map.scrolling_direction = record.word(8);
    }
}

pub fn scroll_arrows<H: Host + ?Sized>(
    screen: MapScreen,
    map: &mut MapScreenState,
    menu: &mut MenuState,
    bg1: [u16; 2],
    game: &mut H,
) {
    let (base, top, bottom) = match screen {
        MapScreen::Pause => (0x82b9a0, 56, 177),
        MapScreen::FileSelect => (0x81af32, 64, 145),
    };
    if (i32::from(map.min_x_scroll) - 24 - i32::from(bg1[0])) & 0x8000 != 0 {
        scroll_arrow(base, map, menu, game);
    }
    if (i32::from(map.max_x_scroll) - 232 - i32::from(bg1[0])) & 0x8000 == 0 {
        scroll_arrow(base + 10, map, menu, game);
    }
    if (i32::from(map.min_y_scroll) - top - i32::from(bg1[1])) & 0x8000 != 0 {
        scroll_arrow(base + 20, map, menu, game);
    }
    if (i32::from(map.max_y_scroll) - bottom - i32::from(bg1[1])) & 0x8000 != 0 {
        if map.scrolling_direction == menu_tilemap::native_record(base + 30, game).word(8) {
            map.gear_switch_timer = 0;
            map.scrolling_direction = 0;
            map.scrolling_speed_index = 0;
        }
    } else {
        scroll_arrow(base + 30, map, menu, game);
    }
}
