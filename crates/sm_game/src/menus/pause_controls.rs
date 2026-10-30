//! Pause-screen lifecycle, input and saved graphics; RAM remains authoritative.
#![forbid(unsafe_code)]
use super::prelude::*;
use super::{equipment_menu, map_menu, menu_tilemap};
use crate::ram::RawRam as _;
use crate::ram::{word, words as fields};

pub fn hook_address<H: crate::menus::host::Host + ?Sized>(pausing: bool, game: &H) -> u32 {
    let offset = if pausing { 0x601 } else { 0x604 };
    // Load24 reads four native bytes, including the following field, before masking.
    let mut bytes = [0; 4];
    game.ram().copy_out(offset, &mut bytes);
    u32::from_ne_bytes(bytes) & 0xffffff
}

pub fn initialize<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    map_menu::reset_animations(game);
    menu_tilemap::equipment_icons(game);
    menu_tilemap::pause_labels(0, game);
    menu_tilemap::queue_pause_labels(game);
    map_menu::scroll_limits(game);
    map_menu::center_map(0x80, game);
    word! { game, 1; screen_fade_delay, screen_fade_counter }
    word! { game; pausemenu_leftarrow_animation_frame = 0; pausemenu_palette_animation_timer = 1; }
    word! { game, 0; map_scrolling_direction, map_scrolling_speed_index }
    game.queue_fx_clear();
    word!(game.game_state += 1);
}

pub fn fade_in<H: crate::menus::host::Host + ?Sized>(game: &mut H) -> u8 {
    if word!(game.menu_index) != 0 {
        draw_equipment(game);
    } else {
        draw_map(game);
    }
    game.fade(true);
    if game.ram().read_byte(crate::ram::mirror::INIDISP) == 15 {
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.game_state += 1);
    }
    0
}

pub fn update_map<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    advance(game);
    map_menu::scroll_arrows(map_menu::MapScreen::Pause, game);
    map_menu::scroll(game);
    draw_map(game);
    word!(game.pause_screen_mode = 0);
}

pub fn update_equipment<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    word! { game, 0; reg_BG1HOFS, reg_BG1VOFS }
    equipment_menu::update(game);
    advance(game);
    word!(game.pause_screen_mode = 1);
}

pub fn fade_tab<const EQUIPMENT: bool, H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if EQUIPMENT {
        draw_equipment(game);
    } else {
        draw_map(game);
    }
    word!(game.pause_screen_mode = u16::from(EQUIPMENT));
    game.fade(true);
    if game.ram().read_byte(crate::ram::mirror::INIDISP) == 15 {
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        let mut selected = word!(game.pausemenu_button_label_mode);
        if word!(game.pausemenu_button_label_mode) != 0 {
            selected = 1;
        }
        word!(game.menu_index = selected);
    }
}

pub fn fade_out_tab<H: crate::menus::host::Host + ?Sized>(
    game: &mut H,
    equipment: bool,
    enable: fn(&mut H),
) {
    if equipment {
        draw_equipment(game);
    } else {
        map_menu::display_elevator_destinations(game);
        map_menu::draw_player_marker(game);
        map_menu::draw_icons(map_menu::IconMode::Pause, game);
    }
    draw_shoulder(game);
    // Equipment fade preserves this mode; map fade resets it after drawing.
    if !equipment {
        word!(game.pause_screen_mode = 0);
    }
    game.fade(false);
    if game.ram().read_byte(crate::ram::mirror::INIDISP) == 0x80 {
        enable(game);
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.menu_index += 1);
    }
}

pub fn load_tab<H: crate::menus::host::Host + ?Sized>(
    game: &mut H,
    equipment: bool,
    upload: impl FnOnce(&mut H),
) {
    map_menu::display_elevator_destinations(game);
    if equipment {
        crate::menus::reserve_tanks::prepare_menu(game);
    }
    upload(game);
    if equipment {
        word!(game.reg_BG1VOFS = 0);
        word!(game.pause_screen_mode = 1);
    }
    menu_tilemap::pause_labels(word!(game.pausemenu_button_label_mode), game);
    word!(game.pausemenu_lr_animation_frame = 0);
    word!(game.pausemenu_lr_animation_timer = menu_tilemap::native_record(0x82c10c, game).word(0));
    word! { game, 1; screen_fade_delay, screen_fade_counter }
    if !equipment {
        word!(game.pause_screen_mode = 0);
    }
    word!(game.menu_index += 1);
}

fn draw_map<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    map_menu::draw_player_marker(game);
    map_menu::draw_icons(map_menu::IconMode::Pause, game);
    map_menu::display_elevator_destinations(game);
}

pub fn draw_equipment<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    map_menu::draw_equipment_selector(game);
    menu_tilemap::reserve_tanks(game);
}

pub fn draw_unpause<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    draw_start(game);
    if word!(game.pause_screen_mode) == 1 {
        draw_equipment(game);
        draw_shoulder(game);
    } else {
        // Unpause keeps the opposite icon/marker order from the active map.
        map_menu::display_elevator_destinations(game);
        map_menu::draw_icons(map_menu::IconMode::Pause, game);
        map_menu::draw_player_marker(game);
    }
}

pub fn resume<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let camera = crate::camera::current(game);
    word!(game.reg_BG1HOFS = camera.bg1_offset[0].wrapping_add(camera.layer1[0]));
    word!(game.reg_BG1VOFS = camera.bg1_offset[1].wrapping_add(camera.layer1[1]));
    word!(game.reg_BG2HOFS = camera.bg2_scroll[0].wrapping_add(camera.layer2[0]));
    word!(game.reg_BG2VOFS = camera.bg2_scroll[1].wrapping_add(camera.layer2[1]));
    game.update_beam_tiles_and_palette();
    // Palette timer/frame and pause_screen_mode survive this ordered reset.
    crate::word_transfers!(game, ascending, 10; little(fields::menu_index) <- value(0));
    crate::word_transfers!(game, ascending, 9; little(fields::UNUSED_word_7E073D) <- value(0));
    crate::word_transfers!(game, ascending, 9; little(fields::pausemenu_shoulder_button_highlight) <- value(0));
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
            .write_byte(crate::ram::bytes::pausemenu_bak_BG1SC + index, value);
    }
}

pub fn restore_graphics<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    for index in [13, 14, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 16] {
        let value = game
            .ram()
            .read_byte(crate::ram::bytes::pausemenu_bak_BG1SC + index);
        game.ram().write_byte(GRAPHICS_MIRRORS[index], value);
    }
}

fn advance<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    shoulder_input(game);
    map_menu::animate_pause_sprite(2, 0x18, 0xd0, game);
    map_menu::animate_pause_sprite(2, 0xe8, 0xd0, game);
    if word!(game.newly_held_down_timed_held_input) & kButton_Start as u16 != 0 {
        game.queue_sound(0, (0x38) as u8, 6);
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        let previous = word!(game.pausemenu_button_label_mode);
        word!(game.pausemenu_button_label_mode = 1);
        menu_tilemap::pause_labels(word!(game.pausemenu_button_label_mode), game);
        word!(game.pausemenu_button_label_mode = previous);
        word!(game.pausemenu_start_lr_pressed_highlight_timer = 11);
        word!(game.game_state += 1);
    }
    menu_tilemap::queue_pause_labels(game);
}

fn shoulder_input<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let (menu, highlight, mode) =
        if word!(game.newly_held_down_timed_held_input) & kButton_L as u16 != 0 {
            if word!(game.pausemenu_button_label_mode) == 0 {
                return;
            }
            (5, 1, 0)
        } else if word!(game.newly_held_down_timed_held_input) & kButton_R as u16 != 0
            && word!(game.pausemenu_button_label_mode) != 2
        {
            (2, 2, 2)
        } else {
            return;
        };
    word!(game.pausemenu_start_lr_pressed_highlight_timer = 5);
    word! {game; menu_index = menu; pausemenu_shoulder_button_highlight = highlight; pausemenu_button_label_mode = mode; }
    menu_tilemap::pause_labels(word!(game.pausemenu_button_label_mode), game);
    game.queue_sound(0, (0x38) as u8, 6);
}

fn tick_highlight<H: crate::menus::host::Host + ?Sized>(game: &mut H) -> bool {
    if word!(game.pausemenu_start_lr_pressed_highlight_timer) == 0 {
        return false;
    }
    word!(game.pausemenu_start_lr_pressed_highlight_timer -= 1);
    true
}

pub fn draw_shoulder<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if tick_highlight(game) && word!(game.pausemenu_shoulder_button_highlight) != 0 {
        let index = (i32::from(word!(game.pausemenu_shoulder_button_highlight)) - 1) as usize;
        let (sprite, x) = [(0x28, 0x18), (0x29, 0xe8)][index];
        crate::sprite_output::menu(sprite, x, 0xcf, 0, game);
    }
}

fn draw_start<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if tick_highlight(game) {
        crate::sprite_output::menu(0x2b, 0x90, 0xd0, 0, game);
    }
}
