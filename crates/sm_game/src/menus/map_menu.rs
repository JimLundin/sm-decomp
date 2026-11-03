//! Area and room selection, with the original menu-state and RAM compatibility.
use super::menu_tilemap;
use super::prelude::*;
use super::save_slots::load_area_map;
use crate::constants::{bank_81::*, bank_82::*};
use crate::display::control::Layers;
use crate::display::preparation::Field as DisplayField;
use crate::display::video::{Command, Control, apply_mirrored};
use crate::imported::{Imported, grid_word, word as table_word};
use crate::menu_state::set_menu_byte;
use crate::menu_state::{MENU, MenuState, WORDS};
use crate::menu_state::{MapScreenState, MenuScreens};
use crate::menu_window_state::Field;
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
    crate::menus::state::with(
        game,
        |game, (menu, screens): &mut (MenuState, MenuScreens)| {
            states[menu.menu_index as usize](menu, game, screens)
        },
    );
    super::pending_wait(game)
}
fn fade_to_options<H: Host + ?Sized>(
    menu: &mut MenuState,
    game: &mut H,
    screens: &mut MenuScreens,
) {
    draw_area_labels(screens, game);
    game.fade(false);
    if (game.display_control_byte(crate::display::preparation::Field::Brightness) & 0xf) == 0 {
        crate::gameplay_word!(game.GAME_STATE = K_GAME_STATE_2_GAME_OPTIONS_MENU as u16);
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
    crate::sequence_word!(game.PALETTE_CHANGE_NUM = 0);
    menu_tilemap::load_palette(game);
    for index in 0..32 {
        let color = game.palette_word(false, 2 * index);
        game.palette_write_word(true, 2 * index, color);
    }
    game.palette_write_word(true, 28, 0);
    game.palette_write_word(true, 60, 0);
    menu.menu_index = menu.menu_index.wrapping_add(1);
}
fn fade_palette<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, screens: &mut MenuScreens) {
    if crate::palette_effects::fading::file_select(game) != 0 {
        display_scroll! { game, 0; Bg1V, Bg1H, Bg2V, Bg2H }
        screens.save_selection.map_area_index = area_selection(game).unwrap_or(0) as u16;
        for area in 0..6 {
            menu_tilemap::area_palette(
                area,
                area == crate::gameplay_context::navigation(game).area,
                game,
            );
        }
        menu.menu_index = menu.menu_index.wrapping_add(1);
        screens.save_selection.map_unread = 0;
        game.store_display_control_byte(
            crate::display::preparation::Field::MainLayers,
            crate::display::control::Layers::BG2.byte(),
            crate::display::preparation::Store::Owned,
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
        ((game.display_control_byte(crate::display::preparation::Field::ObjectLayout) as i32) << 8
            | 0xf) as u16,
        game,
    );
    menu.menu_index = menu.menu_index.wrapping_add(1);
}
fn begin_background<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, _: &mut MenuScreens) {
    menu.menu_index = menu.menu_index.wrapping_add(1);
    load_background(crate::gameplay_context::navigation(game).area, game);
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
/// The producer name of the area window's display commands.
const WINDOW: &str = "map window";
/// HDMA channels 2 and 3: the window edges.
const WINDOW_CHANNELS: Control = Control::HdmaChannels(0xc);

/// Apply each control, its display mirror stored first.
fn show_mirrored<H: Host + ?Sized>(game: &mut H, controls: &[(Control, DisplayField)]) {
    for &(control, mirror) in controls {
        apply_mirrored(game, WINDOW, control, mirror, true);
    }
}
fn begin_area_window<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, _: &mut MenuScreens) {
    game.store_display_control_byte(
        crate::display::preparation::Field::SubLayers,
        crate::display::control::Layers::BG3.byte(),
        crate::display::preparation::Store::Owned,
    );
    // The subscreen's layers are mirrored into the sub window byte.
    apply_mirrored(
        game,
        WINDOW,
        Control::SubScreen(Layers::BG3),
        DisplayField::SubWindow,
        false,
    );
    game.apply(
        WINDOW,
        Command::Control(Control::SubWindowMask(Layers::BG3)),
    );
    show_mirrored(
        game,
        &[
            (Control::ColorSelection(2), DisplayField::ColorSelection),
            (Control::ColorMath(0x25), DisplayField::ColorMath),
        ],
    );
    game.store_display_control_byte(
        crate::display::preparation::Field::FixedColor2,
        0x80,
        crate::display::preparation::Store::Owned,
    );
    game.store_display_control_byte(
        crate::display::preparation::Field::FixedColor1,
        64,
        crate::display::preparation::Store::Owned,
    );
    game.store_display_control_byte(
        crate::display::preparation::Field::FixedColor0,
        32,
        crate::display::preparation::Store::Owned,
    );
    super::menu_window::initialize(game);
    show_mirrored(game, &[(WINDOW_CHANNELS, DisplayField::HdmaEnable)]);
    configure_area_window(game);
    crate::hdma_objects::clear_hdma_flags(game);
    let index = (menu.menu_index as u8).wrapping_add(1);
    set_menu_byte(&mut menu.menu_index, 0, index);
}
fn configure_area_window<H: Host + ?Sized>(game: &mut H) {
    game.store_display_control_byte(
        crate::display::preparation::Field::MainLayers,
        crate::display::control::Layers::BG1_BG2_SPRITES.byte(),
        crate::display::preparation::Store::Owned,
    );
    // The main screen's layers are mirrored into the main window byte.
    apply_mirrored(
        game,
        WINDOW,
        Control::MainScreen(Layers::BG1_BG2_SPRITES),
        DisplayField::MainWindow,
        false,
    );
    game.apply(
        WINDOW,
        Command::Control(Control::MainWindowMask(Layers::BG1_BG2_SPRITES)),
    );
    show_mirrored(
        game,
        &[
            (Control::WindowsBg1Bg2(0x23), DisplayField::WindowSelection0),
            (Control::WindowsBg3Bg4(3), DisplayField::WindowSelection1),
            (
                Control::WindowsSpritesColor(0x23),
                DisplayField::WindowSelection2,
            ),
        ],
    );
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
        game.store_display_control_byte(
            crate::display::preparation::Field::MainLayers,
            game.display_control_byte(crate::display::preparation::Field::MainLayers)
                & !crate::display::control::Layers::BG2.byte(),
            crate::display::preparation::Store::Owned,
        );
        game.store_display_control_byte(
            crate::display::preparation::Field::MainWindow,
            crate::display::control::Layers::NONE.byte(),
            crate::display::preparation::Store::Owned,
        );
        game.store_display_control_byte(
            crate::display::preparation::Field::SubWindow,
            crate::display::control::Layers::NONE.byte(),
            crate::display::preparation::Store::Owned,
        );
        display_scroll!(game.Bg2V = 24);
        menu.pausemenu_palette_animation_timer = 1;
        let mut _v1: u16 = 0;
        menu_tilemap::copy_rom_words(Point::new(Role::Xray, 0), 0xb6e000, 0, 800, false, game);
        _v1 = 800;
        game.screen_apply(|workspace| {
            super::canvas::fill_native(workspace, Point::new(Role::Xray, 1600), 224, false, 10241)
        });
        _v1 = 1024;
        menu_tilemap::area_label(game);
        menu_tilemap::copy_rom_words(
            Point::new(Role::Xray, 0x77e),
            0x81b14b,
            160,
            160,
            true,
            game,
        );
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
            keys = crate::gameplay_word!(game.ENABLE_DEBUG) as i16;
            if crate::gameplay_word!(game.ENABLE_DEBUG) != 0 {
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
    if crate::gameplay_word!(game.ENABLE_DEBUG) == 0 {
        let selection = &mut screens.save_selection;
        crate::gameplay_context::set_room(game, crate::room_view::Word::Area, selection.save_area);
        selection.load_station = selection.save_station;
        draw_area_labels(screens, game);
        return;
    }
    crate::gameplay_context::set_room(
        game,
        crate::room_view::Word::Area,
        AREAS[usize::from(screens.save_selection.map_area_index)],
    );
    let mut bits = native_word(
        USED_STATIONS + 2 * usize::from(crate::gameplay_context::navigation(game).area),
        game,
    );
    let mut station = 0u16;
    let coordinates = station_coordinates(crate::gameplay_context::navigation(game).area, game);
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
                if !(crate::gameplay_word!(game.ENABLE_DEBUG) != 0 && station.word(0) != 0xffff) {
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
    game.store_display_control_byte(
        crate::display::preparation::Field::MainLayers,
        crate::display::control::Layers::BG1_BG2_SPRITES.byte(),
        crate::display::preparation::Store::Owned,
    );
    game.store_display_control_byte(
        crate::display::preparation::Field::MainWindow,
        crate::display::control::Layers::BG1_BG2_SPRITES.byte(),
        crate::display::preparation::Store::Owned,
    );
    show_mirrored(
        game,
        &[
            (Control::WindowsBg1Bg2(0x32), DisplayField::WindowSelection0),
            (Control::WindowsBg3Bg4(2), DisplayField::WindowSelection1),
            (Control::ColorMath(5), DisplayField::ColorMath),
            (
                Control::WindowsSpritesColor(0x22),
                DisplayField::WindowSelection2,
            ),
        ],
    );
    game.store_display_control_byte(
        crate::display::preparation::Field::Characters0,
        48,
        crate::display::preparation::Store::Owned,
    );
    // Close both windows' middle line counts.
    game.ram()
        .write_byte(super::menu_window::ram::WINDOW_1_TABLE + 9, 0);
    game.ram()
        .write_byte(super::menu_window::ram::WINDOW_2_TABLE + 9, 0);
    menu_tilemap::area_label(game);
    menu_tilemap::upload(512, 0x7e4000, 1, false, game);
    let mut window = game.window_state();
    window.timer = WINDOW_TIMERS[crate::gameplay_context::navigation(game).area as usize];
    game.publish_window(window, &[Field::Timer]);
    let label: i32 =
        ((4u16).wrapping_mul(crate::gameplay_context::navigation(game).area)) as i32 >> 1;
    window.positions[0].whole = LABEL_POSITIONS[label as usize];
    game.publish_window(window, &[Field::Whole(0)]);
    window.positions[1].whole = window.positions[0].whole;
    game.publish_window(window, &[Field::Whole(1)]);
    window.positions[2].whole = LABEL_POSITIONS[(label + 1) as usize];
    game.publish_window(window, &[Field::Whole(2)]);
    window.positions[3].whole = window.positions[2].whole;
    game.publish_window(window, &[Field::Whole(3)]);
    for position in &mut window.positions {
        position.fraction = 0;
    }
    game.publish_window(
        window,
        &[
            Field::Fraction(0),
            Field::Fraction(1),
            Field::Fraction(2),
            Field::Fraction(3),
        ],
    );
    super::menu_window::load_velocities(crate::gameplay_context::navigation(game).area, game);
    super::menu_window::project(game);
    show_mirrored(game, &[(WINDOW_CHANNELS, DisplayField::HdmaEnable)]);
    menu.menu_index = menu.menu_index.wrapping_add(1);
    game.queue_sound(0, 0x3b_u8, 6);
}
fn open_room_window<H: Host + ?Sized>(
    menu: &mut MenuState,
    game: &mut H,
    screens: &mut MenuScreens,
) {
    super::menu_window::advance(super::menu_window::Motion::Expand, game);
    let mut window = game.window_state();
    let finished = window.decrement_timer();
    game.publish_window(window, &[Field::Timer]);
    if finished {
        menu.menu_index = menu.menu_index.wrapping_add(1);
        game.store_display_control_byte(
            crate::display::preparation::Field::MainLayers,
            crate::display::control::Layers::BG2.byte(),
            crate::display::preparation::Store::Owned,
        );
        game.store_display_control_byte(
            crate::display::preparation::Field::MainWindow,
            crate::display::control::Layers::NONE.byte(),
            crate::display::preparation::Store::Owned,
        );
        game.store_display_control_byte(
            crate::display::preparation::Field::SubWindow,
            crate::display::control::Layers::NONE.byte(),
            crate::display::preparation::Store::Owned,
        );
        game.store_display_control_byte(
            crate::display::preparation::Field::SubLayers,
            crate::display::control::Layers::NONE.byte(),
            crate::display::preparation::Store::Owned,
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
    let header = game.view_word(crate::projection::Field::RoomHeader);
    let room = menu_tilemap::native_record(0x8f0000 | u32::from(header), game);
    crate::gameplay_context::enter_header(game, [1, 2, 3].map(|at| room.byte(at)));
    reset_animations(&mut screens.map, menu, game);
    scroll_limits(screens, game);
    center_map(112, &screens.map, game);
    let map = &mut screens.map;
    map.min_y_scroll = map.min_y_scroll.wrapping_add(24);
    display_scroll!(game.Bg2V = 24);
    use crate::display::preparation::{Field, Store};
    let layers = u16::from_le_bytes([
        game.display_control_byte(Field::MainLayers),
        game.display_control_byte(Field::PreviousMainLayers),
    ]) & !4;
    let [low, high] = layers.to_le_bytes();
    game.store_display_control_byte(Field::MainLayers, low, Store::Owned);
    game.store_display_control_byte(Field::PreviousMainLayers, high, Store::Owned);
    menu.menu_index = menu.menu_index.wrapping_add(1);
    map.scrolling_direction = 0;
    map.scrolling_speed_index = 0;
    map.indicator_frame = 0;
    map.indicator_timer = 0;
    map.indicator_loop_counter = 0;
}
fn choose_room<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, screens: &mut MenuScreens) {
    draw_icons(IconMode::FileSelect, screens, menu, game);
    scroll_arrows(MapScreen::FileSelect, &mut screens.map, menu, game);
    scroll(&mut screens.map, game);
    display_elevator_destinations(game);
    if crate::gameplay_word!(game.ENABLE_DEBUG) != 0
        && (crate::gameplay_word!(game.JOYPAD2_NEW_KEYS) as i32 & K_BUTTON_SELECT as i32) != 0
    {
        game.queue_sound(0, 0x38_u8, 6);
        let mut unlocked: u16 = native_word(
            USED_STATIONS + 2 * usize::from(crate::gameplay_context::navigation(game).area),
            game,
        );
        let selection = &mut screens.save_selection;
        let mut remaining: i16 = selection.load_station as i16;
        do_until! { remaining < 0 ;
            unlocked >>= 1;
            remaining -= 1;
        }
        let stations = station_coordinates(crate::gameplay_context::navigation(game).area, game);
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
                            + 2 * usize::from(crate::gameplay_context::navigation(game).area),
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
        if (next_x as i32 - display_scroll!(game.Bg1H) as i32) & 0x8000 != 0
            || (next_x as i32 - 256i32 - display_scroll!(game.Bg1H) as i32) & 0x8000 == 0
        {
            let mut scroll_x: i16 = (display_scroll!(game.Bg1H) as i32
                + stations.table_word((next_offset >> 1) as isize) as i32
                - station_x as i32) as i16;
            if scroll_x >= 0 {
                if (scroll_x).wrapping_sub(screens.map.min_x_scroll as i16) >= 0 {
                    scroll_x = screens.map.min_x_scroll as i16;
                }
            } else {
                scroll_x = 0;
            }
            display_scroll!(game.Bg1H = scroll_x as u16);
        }
        let next_y_offset: u16 = (next_offset).wrapping_add(2);
        let next_y: u16 = stations.table_word((next_y_offset >> 1) as isize);
        if (next_y as i32 - display_scroll!(game.Bg1V) as i32) & 0x8000 != 0
            || (next_y as i32 - 161i32 - display_scroll!(game.Bg1V) as i32) & 0x8000 == 0
        {
            let mut scroll_y: u16 = (display_scroll!(game.Bg1V) as i32
                + stations.table_word((next_y_offset >> 1) as isize) as i32
                - station_y as i32) as u16;
            if ((scroll_y) as i16).wrapping_sub(screens.map.min_y_scroll as i16) >= 0 {
                scroll_y = screens.map.min_y_scroll;
            }
            display_scroll!(game.Bg1V = scroll_y);
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
    draw_icons(IconMode::FileSelect, screens, menu, game);
    display_elevator_destinations(game);
    menu.menu_index = menu.menu_index.wrapping_add(1);
}
fn fade_room<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, screens: &mut MenuScreens) {
    draw_icons(IconMode::FileSelect, screens, menu, game);
    display_elevator_destinations(game);
    game.fade(false);
    if (game.display_control_byte(crate::display::preparation::Field::Brightness) & 0xf) == 0 {
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
        crate::gameplay_word!(game.GAME_STATE += 1);
        menu.menu_index = 0;
    }
}
fn clear_room<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, _: &mut MenuScreens) {
    game.store_display_control_byte(
        crate::display::preparation::Field::MainLayers,
        crate::display::control::Layers::BG2_SPRITES.byte(),
        crate::display::preparation::Store::Owned,
    );
    game.screen_apply(|workspace| {
        super::canvas::fill_native(workspace, Point::new(Role::Pause, 0x7fe), 1024, true, 15)
    });
    menu_tilemap::upload(2048, 0x7e3000, 0, true, game);
    menu.menu_index = menu.menu_index.wrapping_add(1);
}
fn restore_area<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, _: &mut MenuScreens) {
    menu_tilemap::load_palette(game);
    for area in 0..6 {
        menu_tilemap::area_palette(area, false, game);
    }
    menu_tilemap::area_palette(crate::gameplay_context::navigation(game).area, true, game);
    load_background(crate::gameplay_context::navigation(game).area, game);
    menu.menu_index = menu.menu_index.wrapping_add(1);
}
fn begin_return_window<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H, _: &mut MenuScreens) {
    game.store_display_control_byte(
        crate::display::preparation::Field::HdmaEnable,
        0,
        crate::display::preparation::Store::Owned,
    );
    game.queue_sound(0, 0x3c_u8, 6);
    let mut window = game.window_state();
    window.timer =
        WINDOW_TIMERS[crate::gameplay_context::navigation(game).area as usize].wrapping_sub(12);
    game.publish_window(window, &[Field::Timer]);
    super::menu_window::load_velocities(crate::gameplay_context::navigation(game).area, game);
    let mut window = game.window_state();
    for (position, whole) in window.positions.iter_mut().zip([8, 248, 8, 216]) {
        position.whole = whole;
    }
    game.publish_window(
        window,
        &[
            Field::Whole(0),
            Field::Whole(1),
            Field::Whole(2),
            Field::Whole(3),
        ],
    );
    for position in &mut window.positions {
        position.fraction = 0;
    }
    game.publish_window(
        window,
        &[
            Field::Fraction(0),
            Field::Fraction(1),
            Field::Fraction(2),
            Field::Fraction(3),
        ],
    );
    super::menu_window::project(game);
    menu.menu_index = menu.menu_index.wrapping_add(1);
    show_mirrored(
        game,
        &[
            (WINDOW_CHANNELS, DisplayField::HdmaEnable),
            (Control::WindowsBg1Bg2(0x32), DisplayField::WindowSelection0),
            (Control::WindowsBg3Bg4(2), DisplayField::WindowSelection1),
        ],
    );
    game.store_display_control_byte(
        crate::display::preparation::Field::Characters0,
        48,
        crate::display::preparation::Store::Owned,
    );
    game.store_display_control_byte(
        crate::display::preparation::Field::WindowSelection2,
        34,
        crate::display::preparation::Store::Owned,
    );
    // Close both windows' middle line counts.
    game.ram()
        .write_byte(super::menu_window::ram::WINDOW_1_TABLE + 9, 0);
    game.ram()
        .write_byte(super::menu_window::ram::WINDOW_2_TABLE + 9, 0);
    display_scroll! { game, 0; Bg1V, Bg1H }
}
fn close_room_window<H: Host + ?Sized>(
    menu: &mut MenuState,
    game: &mut H,
    screens: &mut MenuScreens,
) {
    super::menu_window::advance(super::menu_window::Motion::Contract, game);
    draw_area_labels(screens, game);
    let mut window = game.window_state();
    let finished = window.decrement_timer();
    game.publish_window(window, &[Field::Timer]);
    if finished {
        menu.menu_index = menu.menu_index.wrapping_sub(15);
        game.store_display_control_byte(
            crate::display::preparation::Field::MainLayers,
            crate::display::control::Layers::BG1_SPRITES.byte(),
            crate::display::preparation::Store::Owned,
        );
        game.store_display_control_byte(
            crate::display::preparation::Field::MainWindow,
            crate::display::control::Layers::NONE.byte(),
            crate::display::preparation::Store::Owned,
        );
        game.store_display_control_byte(
            crate::display::preparation::Field::SubWindow,
            crate::display::control::Layers::NONE.byte(),
            crate::display::preparation::Store::Owned,
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
    if (EXPLORED..EXPLORED + 258).contains(&offset) {
        let index = isize::try_from(offset - EXPLORED).unwrap();
        return u16::from_ne_bytes([
            game.map_explored_byte(index),
            game.map_explored_byte(index + 1),
        ]);
    }
    if (0x3000..0x7000).contains(&offset) {
        return game.screen_native_word(Point::physical(offset));
    }
    // Native address tables may select scalar/control storage outside this owner.
    u16::from_ne_bytes([
        game.ram().read_byte(offset),
        game.ram().read_byte(offset + 1),
    ])
}

fn set_native_word<H: Host + ?Sized>(offset: usize, value: u16, game: &mut H) {
    if (0x3000..0x7000).contains(&offset) {
        game.screen_write_slice(Point::physical(offset), &value.to_ne_bytes());
    } else {
        game.ram().copy_in(offset, &value.to_ne_bytes());
    }
}

fn area_selection<H: Host + ?Sized>(game: &H) -> Option<usize> {
    AREAS
        .iter()
        .position(|&area| area == crate::gameplay_context::navigation(game).area)
}

const SCROLL_SPEEDS: [u16; 32] = [
    0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Advance the shared file-select/pause-map scroll, with RAM as its only state.
pub fn scroll<H: Host + ?Sized>(map: &mut MapScreenState, game: &mut H) {
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
        1 => display_scroll!(game.Bg1H -= speed),
        2 => display_scroll!(game.Bg1H += speed),
        3 => display_scroll!(game.Bg1V -= speed),
        _ => display_scroll!(game.Bg1V += speed),
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

fn draw_record<H: Host + ?Sized>(
    sprite: u16,
    record: &sm_data::cartridge::RomRecord,
    palette: u16,
    game: &mut H,
) {
    crate::sprite_output::menu(
        sprite,
        record.word(0).wrapping_sub(display_scroll!(game.Bg1H)),
        record.word(2).wrapping_sub(display_scroll!(game.Bg1V)),
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
    game: &mut H,
) {
    if matches!(mode, IconMode::FileSelect) {
        animate_palette(menu, game);
    }
    boss_icons(9, 0xc7cb, screens.save_selection.has_area_map, game);
    for (sprite, table) in [(0xb, 0xc7db), (0xa, 0xc7eb), (0x4e, 0xc7fb)] {
        icon_group(sprite, table, u16::MAX, 3584, game);
    }
    match mode {
        IconMode::Pause => {
            if crate::gameplay_context::navigation(game).area == screens.save_selection.save_area {
                let bits = station_bits(0, game);
                let mut bytes = bits.to_ne_bytes();
                bytes[0] = bits as u8
                    & [1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80]
                        [usize::from(screens.save_selection.load_station)];
                icon_group(8, 0xc80b, u16::from_ne_bytes(bytes), 1024, game);
            }
            if crate::gameplay_word!(game.ENABLE_DEBUG) != 0 {
                icon_group(8, 0xc82b, u16::MAX, 1024, game);
            }
        }
        IconMode::FileSelect => {
            let sprite = advance_marker(&mut screens.map);
            // This fixed table is native and linear; only its result is bank-wrapped.
            let address = game.native_image().table_word(
                sm_data::cartridge::lorom_offset(0x82c80b) as usize,
                crate::gameplay_context::navigation(game).area as isize,
            );
            let record = map_record(address, game);
            let offset = 4 * usize::from(screens.save_selection.load_station);
            let x = record.word(offset).wrapping_sub(display_scroll!(game.Bg1H));
            let y = record
                .word(offset + 2)
                .wrapping_sub(display_scroll!(game.Bg1V));
            if screens.map.indicator_loop_counter & 1 == 0 {
                crate::sprite_output::menu(0x12, x, y, 3584, game);
            }
            crate::sprite_output::menu(sprite, x, y, 3584, game);
            if crate::gameplay_word!(game.ENABLE_DEBUG) != 0 {
                let saves = station_bits(0, game);
                icon_group(0xc, 0xc80b, saves, 1536, game);
                let elevators = station_bits(1, game);
                icon_group(0x17, 0xc81b, elevators, 1536, game);
                icon_group(0xc, 0xc82b, u16::MAX, 1536, game);
            }
        }
    }
    if crate::gameplay_context::navigation(game).area == 0 {
        crate::sprite_output::menu(
            0x63,
            0xd8u16.wrapping_sub(display_scroll!(game.Bg1H)),
            0x28u16.wrapping_sub(display_scroll!(game.Bg1V)),
            3584,
            game,
        );
    }
}

fn station_bits<H: Host + ?Sized>(offset: usize, game: &mut H) -> u16 {
    let address =
        USED_STATIONS + 2 * usize::from(crate::gameplay_context::navigation(game).area) + offset;
    game.used_station_native_word(address - USED_STATIONS)
}

fn icon_group<H: Host + ?Sized>(
    sprite: u16,
    table: u16,
    mut bits: u16,
    palette: u16,
    game: &mut H,
) {
    let mut cursor = map_record(
        (i32::from(table) + 2 * i32::from(crate::gameplay_context::navigation(game).area)) as u16,
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

fn explored<H: Host + ?Sized>(x: u16, y: u16, game: &mut H) -> bool {
    let offset =
        usize::from((x & 0xff00) >> 1) + usize::from(x as u8 >> 6) + usize::from((y & 0xfff8) >> 1);
    game.map_explored_byte(isize::try_from(offset).unwrap()) & (0x80 >> ((x >> 3) & 7)) != 0
}

fn boss_icons<H: Host + ?Sized>(sprite: u16, table: u16, has_area_map: u16, game: &mut H) {
    let mut bits =
        crate::progress::leased::boss_byte_at(crate::gameplay_context::navigation(game).area, game)
            as i32;
    let cursor = map_record(
        (i32::from(table) + 2 * i32::from(crate::gameplay_context::navigation(game).area)) as u16,
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
                } else if has_area_map != 0 {
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
        game.palette_write_native_word(false, 2 * index, color);
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

pub fn draw_player_marker<H: Host + ?Sized>(map: &mut MapScreenState, game: &mut H) {
    let sprite = advance_marker(map);
    crate::sprite_output::menu(
        sprite,
        (8 * (i32::from(crate::gameplay_context::room_effects(game).map_x)
            + i32::from(game.player_x().to_le_bytes()[1]))
            - i32::from(display_scroll!(game.Bg1H))) as u16,
        (8 * (i32::from(crate::gameplay_context::room_effects(game).map_y)
            + i32::from(game.player_y().to_le_bytes()[1])
            + 1)
            - i32::from(display_scroll!(game.Bg1V))) as u16,
        3584,
        game,
    );
}

pub fn display_elevator_destinations<H: Host + ?Sized>(game: &mut H) {
    if game.map_station_byte(usize::from(crate::gameplay_context::navigation(game).area)) == 0 {
        return;
    }
    let mut cursor = i32::from(table_word(
        &game.pause_menu_tables().map_icons,
        game,
        MAP_ICON_ADDRESS,
        crate::gameplay_context::navigation(game).area as isize,
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

pub fn reset_animations<H: Host + ?Sized>(
    map: &mut MapScreenState,
    menu: &mut MenuState,
    game: &mut H,
) {
    menu.pausemenu_reserve_tank_delay_ctr = 0;
    display_scroll! { game, 0; Bg1H, Bg2H, Bg3H, Bg2V, Bg3V }
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

pub fn center_map<H: Host + ?Sized>(vertical_center: u16, map: &MapScreenState, game: &mut H) {
    display_scroll!(
        game.Bg1H = (i32::from(map.min_x_scroll)
            + (i32::from(map.max_x_scroll.wrapping_sub(map.min_x_scroll)) >> 1)
            - 128) as u16
    );
    let x = (8
        * (i32::from(crate::gameplay_context::room_effects(game).map_x)
            + i32::from((game.player_x() & 0xff00) >> 8))
        - i32::from(display_scroll!(game.Bg1H))) as u16;
    let right = 224i16.wrapping_sub(x as i16);
    if right >= 0 {
        let left = 32u16.wrapping_sub(x);
        if left & 0x8000 == 0 {
            display_scroll!(game.Bg1H -= left);
        }
    } else {
        display_scroll!(game.Bg1H -= right as u16);
    }
    let middle = (i32::from(map.min_y_scroll)
        + (i32::from(map.max_y_scroll.wrapping_sub(map.min_y_scroll)) >> 1)
        + 16) as u16;
    let aligned = (i32::from(vertical_center) - i32::from(middle)) & 0xfff8;
    display_scroll!(game.Bg1V = (-aligned) as u16);
    let y = (8
        * (i32::from(crate::gameplay_context::room_effects(game).map_y)
            + i32::from(game.player_y().to_le_bytes()[1])
            + 1)
        + aligned) as u16;
    let top = 64i16.wrapping_sub(y as i16);
    if top >= 0 {
        display_scroll!(game.Bg1V -= top as u16);
        if (i32::from(display_scroll!(game.Bg1V)) + 40) & 0x8000 != 0 {
            display_scroll!(game.Bg1V = -40i32 as u16);
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
        None => game.map_explored_byte(isize::try_from(offset).unwrap()),
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
                + 2 * i32::from(crate::gameplay_context::navigation(game).area)) as u16,
            game,
        )
        .word(0);
        Some(map_record(pointer, game))
    } else {
        None
    };
    let map = &mut screens.map;
    map.min_x_scroll = find_map_edge(source.as_ref(), MapEdge::Left, game).wrapping_mul(8);
    if crate::gameplay_context::navigation(game).area == 4 {
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
    let mut area = crate::gameplay_context::navigation(game).area;
    if (i32::from(crate::gameplay_context::navigation(game).area) - 7) & 0x8000 == 0 {
        area = 0;
    }
    let address = menu_tilemap::native_record(0x82964a, game).load24(3 * usize::from(area));
    let tiles = menu_tilemap::native_record(address, game);
    let (destination, empty) = match screen {
        MapScreen::Pause => (Role::Xray, 31),
        MapScreen::FileSelect => (Role::Pause, 15),
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

    let mapped_area =
        game.map_station_byte(usize::from(crate::gameplay_context::navigation(game).area)) != 0;
    let mut composition = NativeMapComposition {
        game,
        tiles,
        bitmap,
        destination,
    };
    super::canvas::compose_map(mapped_area, empty, &mut composition);
}

fn draw_room_map<H: Host + ?Sized>(game: &mut H) {
    game.store_display_control_byte(
        crate::display::preparation::Field::Characters0,
        51,
        crate::display::preparation::Store::Owned,
    );
    game.store_display_control_byte(
        crate::display::preparation::Field::MainLayers,
        crate::display::control::Layers::BG1_BG2_SPRITES.byte(),
        crate::display::preparation::Store::Owned,
    );
    display_scroll!(game.Bg1V = -40i16 as u16);
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
    if crate::gameplay_context::inventory(game).reserve_capacity
        | crate::gameplay_context::inventory(game).equipment.owned
        | crate::gameplay_context::inventory(game).beams.owned
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
    game: &mut H,
) {
    let (base, top, bottom) = match screen {
        MapScreen::Pause => (0x82b9a0, 56, 177),
        MapScreen::FileSelect => (0x81af32, 64, 145),
    };
    if (i32::from(map.min_x_scroll) - 24 - i32::from(display_scroll!(game.Bg1H))) & 0x8000 != 0 {
        scroll_arrow(base, map, menu, game);
    }
    if (i32::from(map.max_x_scroll) - 232 - i32::from(display_scroll!(game.Bg1H))) & 0x8000 == 0 {
        scroll_arrow(base + 10, map, menu, game);
    }
    if (i32::from(map.min_y_scroll) - top - i32::from(display_scroll!(game.Bg1V))) & 0x8000 != 0 {
        scroll_arrow(base + 20, map, menu, game);
    }
    if (i32::from(map.max_y_scroll) - bottom - i32::from(display_scroll!(game.Bg1V))) & 0x8000 != 0
    {
        if map.scrolling_direction == menu_tilemap::native_record(base + 30, game).word(8) {
            map.gear_switch_timer = 0;
            map.scrolling_direction = 0;
            map.scrolling_speed_index = 0;
        }
    } else {
        scroll_arrow(base + 30, map, menu, game);
    }
}

/// Native sources retain the allocation captured by the original map lookup.
struct NativeMapComposition<'a, H: ?Sized> {
    game: &'a mut H,
    tiles: sm_data::cartridge::RomRecord,
    bitmap: sm_data::cartridge::RomRecord,
    destination: Role,
}
impl<H: Host + ?Sized> super::canvas::MapComposition for NativeMapComposition<'_, H> {
    fn tile(&mut self, index: usize) -> u16 {
        self.tiles.word(2 * index)
    }
    fn mapped_word(&mut self, offset: usize) -> u16 {
        self.bitmap.word(offset)
    }
    fn explored_word(&mut self, offset: usize) -> u16 {
        let offset = isize::try_from(offset).unwrap();
        u16::from_ne_bytes([
            self.game.map_explored_byte(offset),
            self.game.map_explored_byte(offset + 1),
        ])
    }
    fn explored_byte(&mut self, offset: usize) -> u8 {
        self.game
            .map_explored_byte(isize::try_from(offset).unwrap())
    }
    fn store_explored_byte(&mut self, offset: usize, value: u8) {
        self.game
            .store_map_explored_byte(isize::try_from(offset).unwrap(), value);
    }
    fn store_tile(&mut self, index: usize, value: u16) {
        self.game
            .screen_write_native_slice(Point::new(self.destination, 2 * index), value);
    }
    fn observed_store(&mut self, byte: usize, index: usize, stage: u8) {
        self.game.observe_map_write(byte, index, stage);
    }
}
