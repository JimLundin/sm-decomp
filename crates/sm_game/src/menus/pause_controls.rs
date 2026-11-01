//! Pause-screen lifecycle, input and saved graphics; RAM remains authoritative.
use super::prelude::*;
use super::{equipment_menu, map_menu, menu_tilemap};
use crate::display::sequence::scroll::BgScroll;
use crate::menu_state::MenuState;
use crate::menu_state::{MenuFrame, MenuScreens};
use crate::ram::RawRam as _;
use crate::ram::{word, words as fields};
use crate::session_cohort;

pub fn hook_address<H: crate::menus::host::Host + ?Sized>(pausing: bool, game: &H) -> u32 {
    let offset = if pausing { 0x601 } else { 0x604 };
    // Load24 reads four native bytes, including the following field, before masking.
    let mut bytes = [0; 4];
    game.ram().copy_out(offset, &mut bytes);
    u32::from_ne_bytes(bytes) & 0xffffff
}

pub fn initialize<H: crate::menus::host::Host + ?Sized>(
    menu: &mut MenuState,
    screens: &mut MenuScreens,
    game: &mut H,
) {
    session_cohort::with(game, |game, scroll: &mut BgScroll| {
        map_menu::reset_animations(&mut screens.map, menu, scroll, game);
        menu_tilemap::equipment_icons(game);
        menu_tilemap::pause_labels(0, game);
        menu_tilemap::queue_pause_labels(game);
        map_menu::scroll_limits(screens, game);
        map_menu::center_map(0x80, &screens.map, &mut scroll.bg1, game);
    });
    crate::screen_fade::set_both(game, 1, 1);
    menu.pausemenu_leftarrow_animation_frame = 0;
    menu.pausemenu_palette_animation_timer = 1;
    screens.map.scrolling_direction = 0;
    screens.map.scrolling_speed_index = 0;
    crate::display_refresh::queue_fx_clear(game);
    word!(game.GAME_STATE += 1);
}

pub fn fade_in<H: crate::menus::host::Host + ?Sized>(game: &mut H) -> u8 {
    session_cohort::with(game, |game, (menu, screens): &mut MenuFrame| {
        if menu.menu_index != 0 {
            draw_equipment(menu, game);
        } else {
            let bg1 = session_cohort::load::<BgScroll, _>(game).bg1;
            draw_map(menu, game, screens, bg1);
        }
        game.fade(true);
        if game.ram().read_byte(crate::ram::mirror::INIDISP) == 15 {
            crate::screen_fade::set_both(game, 0, 0);
            word!(game.GAME_STATE += 1);
        }
        0
    })
}

pub fn update_map<H: crate::menus::host::Host + ?Sized>(
    menu: &mut MenuState,
    screens: &mut MenuScreens,
    game: &mut H,
) {
    advance(menu, game);
    session_cohort::with(game, |game, scroll: &mut BgScroll| {
        let bg1 = &mut scroll.bg1;
        map_menu::scroll_arrows(
            map_menu::MapScreen::Pause,
            &mut screens.map,
            menu,
            *bg1,
            game,
        );
        map_menu::scroll(&mut screens.map, bg1, game);
        draw_map(menu, game, screens, *bg1);
    });
    menu.pause_screen_mode = 0;
}

pub fn update_equipment<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    session_cohort::with(game, |_, scroll: &mut BgScroll| scroll.bg1 = [0, 0]);
    equipment_menu::update(menu, game);
    advance(menu, game);
    menu.pause_screen_mode = 1;
}

pub fn fade_tab<const EQUIPMENT: bool, H: crate::menus::host::Host + ?Sized>(
    menu: &mut MenuState,
    screens: &mut MenuScreens,
    game: &mut H,
) {
    if EQUIPMENT {
        draw_equipment(menu, game);
    } else {
        let bg1 = session_cohort::load::<BgScroll, _>(game).bg1;
        draw_map(menu, game, screens, bg1);
    }
    menu.pause_screen_mode = u16::from(EQUIPMENT);
    game.fade(true);
    if game.ram().read_byte(crate::ram::mirror::INIDISP) == 15 {
        crate::screen_fade::set_both(game, 0, 0);
        let mut selected = menu.pausemenu_button_label_mode;
        if menu.pausemenu_button_label_mode != 0 {
            selected = 1;
        }
        menu.menu_index = selected;
    }
}

pub fn fade_out_tab<H: crate::menus::host::Host + ?Sized>(
    menu: &mut MenuState,
    screens: &mut MenuScreens,
    game: &mut H,
    equipment: bool,
    enable: fn(&mut H),
) {
    if equipment {
        draw_equipment(menu, game);
    } else {
        let bg1 = session_cohort::load::<BgScroll, _>(game).bg1;
        map_menu::display_elevator_destinations(bg1, game);
        map_menu::draw_player_marker(&mut screens.map, bg1, game);
        map_menu::draw_icons(map_menu::IconMode::Pause, screens, menu, bg1, game);
    }
    draw_shoulder(menu, game);
    // Equipment fade preserves this mode; map fade resets it after drawing.
    if !equipment {
        menu.pause_screen_mode = 0;
    }
    game.fade(false);
    if game.ram().read_byte(crate::ram::mirror::INIDISP) == 0x80 {
        enable(game);
        crate::screen_fade::set_both(game, 0, 0);
        menu.menu_index = menu.menu_index.wrapping_add(1);
    }
}

pub fn load_tab<H: crate::menus::host::Host + ?Sized>(
    menu: &mut MenuState,
    game: &mut H,
    equipment: bool,
    upload: impl FnOnce(&mut H),
) {
    map_menu::display_elevator_destinations(session_cohort::load::<BgScroll, _>(game).bg1, game);
    if equipment {
        session_cohort::with(game, |game, scroll| {
            crate::menus::reserve_tanks::prepare_menu(menu, scroll, game)
        });
    }
    upload(game);
    if equipment {
        session_cohort::with(game, |_, scroll: &mut BgScroll| scroll.bg1[1] = 0);
        menu.pause_screen_mode = 1;
    }
    menu_tilemap::pause_labels(menu.pausemenu_button_label_mode, game);
    menu.pausemenu_lr_animation_frame = 0;
    menu.pausemenu_lr_animation_timer = menu_tilemap::native_record(0x82c10c, game).word(0);
    crate::screen_fade::set_both(game, 1, 1);
    if !equipment {
        menu.pause_screen_mode = 0;
    }
    menu.menu_index = menu.menu_index.wrapping_add(1);
}

/// Draw the map tab's sprites over BG1's scroll `bg1`.
fn draw_map<H: crate::menus::host::Host + ?Sized>(
    menu: &mut MenuState,
    game: &mut H,
    screens: &mut MenuScreens,
    bg1: [u16; 2],
) {
    map_menu::draw_player_marker(&mut screens.map, bg1, game);
    map_menu::draw_icons(map_menu::IconMode::Pause, screens, menu, bg1, game);
    map_menu::display_elevator_destinations(bg1, game);
}

pub fn draw_equipment<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    map_menu::draw_equipment_selector(menu, game);
    menu_tilemap::reserve_tanks(menu, game);
}

pub fn draw_unpause<H: crate::menus::host::Host + ?Sized>(
    menu: &mut MenuState,
    screens: &mut MenuScreens,
    game: &mut H,
) {
    draw_start(menu, game);
    if menu.pause_screen_mode == 1 {
        draw_equipment(menu, game);
        draw_shoulder(menu, game);
    } else {
        // Unpause keeps the opposite icon/marker order from the active map.
        let bg1 = session_cohort::load::<BgScroll, _>(game).bg1;
        map_menu::display_elevator_destinations(bg1, game);
        map_menu::draw_icons(map_menu::IconMode::Pause, screens, menu, bg1, game);
        map_menu::draw_player_marker(&mut screens.map, bg1, game);
    }
}

pub fn resume<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let camera = crate::camera::current(game);
    session_cohort::with(game, |_, scroll: &mut BgScroll| {
        for axis in 0..2 {
            scroll.bg1[axis] = camera.bg1_offset[axis].wrapping_add(camera.layer1[axis]);
            scroll.bg2[axis] = camera.bg2_scroll[axis].wrapping_add(camera.layer2[axis]);
        }
    });
    game.update_beam_tiles_and_palette();
    // Palette timer/frame and pause_screen_mode survive this ordered reset.
    crate::word_transfers!(game, ascending, 10; little(fields::MENU_INDEX) <- value(0));
    crate::word_transfers!(game, ascending, 9; little(fields::UNUSED_WORD_7E073D) <- value(0));
    crate::word_transfers!(game, ascending, 9; little(fields::PAUSEMENU_SHOULDER_BUTTON_HIGHLIGHT) <- value(0));
    game.player_command(0xc);
}

const GRAPHICS_MIRRORS: [usize; 17] = [
    0x58, 0x59, 0x5a, 0x5d, 0x5e, 0x52, 0xb1, 0xb5, 0xb9, 0xb3, 0xb7, 0xbb, 0x55, 0x91b, 0x91c,
    0x57, 0x71,
];

pub fn save_graphics<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    for (index, source) in GRAPHICS_MIRRORS.into_iter().enumerate() {
        // The six scroll mirrors are read as LE words, then truncated to a byte.
        let value = if (6..12).contains(&index) {
            u16::from_le_bytes([
                game.ram().read_byte(source),
                game.ram().read_byte(source + 1),
            ]) as u8
        } else {
            game.ram().read_byte(source)
        };
        game.ram()
            .write_byte(crate::ram::bytes::PAUSEMENU_BAK_BG1SC + index, value);
    }
}

pub fn restore_graphics<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    for index in [13, 14, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 16] {
        let value = game
            .ram()
            .read_byte(crate::ram::bytes::PAUSEMENU_BAK_BG1SC + index);
        game.ram().write_byte(GRAPHICS_MIRRORS[index], value);
    }
}

fn advance<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    shoulder_input(menu, game);
    map_menu::animate_pause_sprite(2, 0x18, 0xd0, menu, game);
    map_menu::animate_pause_sprite(2, 0xe8, 0xd0, menu, game);
    if word!(game.NEWLY_HELD_DOWN_TIMED_HELD_INPUT) & K_BUTTON_START as u16 != 0 {
        game.queue_sound(0, 0x38_u8, 6);
        crate::screen_fade::set_both(game, 1, 1);
        let previous = menu.pausemenu_button_label_mode;
        menu.pausemenu_button_label_mode = 1;
        menu_tilemap::pause_labels(menu.pausemenu_button_label_mode, game);
        menu.pausemenu_button_label_mode = previous;
        menu.pausemenu_start_lr_pressed_highlight_timer = 11;
        word!(game.GAME_STATE += 1);
    }
    menu_tilemap::queue_pause_labels(game);
}

fn shoulder_input<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    let (index, highlight, mode) =
        if word!(game.NEWLY_HELD_DOWN_TIMED_HELD_INPUT) & K_BUTTON_L as u16 != 0 {
            if menu.pausemenu_button_label_mode == 0 {
                return;
            }
            (5, 1, 0)
        } else if word!(game.NEWLY_HELD_DOWN_TIMED_HELD_INPUT) & K_BUTTON_R as u16 != 0
            && menu.pausemenu_button_label_mode != 2
        {
            (2, 2, 2)
        } else {
            return;
        };
    crate::probe::hit("pause_controls.shoulder_input");
    menu.pausemenu_start_lr_pressed_highlight_timer = 5;
    menu.menu_index = index;
    menu.pausemenu_shoulder_button_highlight = highlight;
    menu.pausemenu_button_label_mode = mode;
    menu_tilemap::pause_labels(menu.pausemenu_button_label_mode, game);
    game.queue_sound(0, 0x38_u8, 6);
}

fn tick_highlight(menu: &mut MenuState) -> bool {
    if menu.pausemenu_start_lr_pressed_highlight_timer == 0 {
        return false;
    }
    menu.pausemenu_start_lr_pressed_highlight_timer = menu
        .pausemenu_start_lr_pressed_highlight_timer
        .wrapping_sub(1);
    true
}

pub fn draw_shoulder<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    if tick_highlight(menu) && menu.pausemenu_shoulder_button_highlight != 0 {
        let index = (i32::from(menu.pausemenu_shoulder_button_highlight) - 1) as usize;
        let (sprite, x) = [(0x28, 0x18), (0x29, 0xe8)][index];
        crate::sprite_output::menu(sprite, x, 0xcf, 0, game);
    }
}

fn draw_start<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    if tick_highlight(menu) {
        crate::sprite_output::menu(0x2b, 0x90, 0xd0, 0, game);
    }
}
