//! Pause-screen lifecycle, input and saved graphics; RAM remains authoritative.
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
    crate::screen_fade::set_both(game, 1, 1);
    crate::menu!(game.PausemenuLeftarrowAnimationFrame = 0);
    crate::menu!(game.PausemenuPaletteAnimationTimer = 1);
    word! { game, 0; MAP_SCROLLING_DIRECTION, MAP_SCROLLING_SPEED_INDEX }
    game.queue_fx_clear();
    word!(game.GAME_STATE += 1);
}

pub fn fade_in<H: crate::menus::host::Host + ?Sized>(game: &mut H) -> u8 {
    if crate::menu!(game.MenuIndex) != 0 {
        draw_equipment(game);
    } else {
        draw_map(game);
    }
    game.fade(true);
    if game.ram().read_byte(crate::ram::mirror::INIDISP) == 15 {
        crate::screen_fade::set_both(game, 0, 0);
        word!(game.GAME_STATE += 1);
    }
    0
}

pub fn update_map<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    advance(game);
    map_menu::scroll_arrows(map_menu::MapScreen::Pause, game);
    map_menu::scroll(game);
    draw_map(game);
    crate::menu!(game.PauseScreenMode = 0);
}

pub fn update_equipment<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    scroll! { game, 0; Bg1H, Bg1V }
    equipment_menu::update(game);
    advance(game);
    crate::menu!(game.PauseScreenMode = 1);
}

pub fn fade_tab<const EQUIPMENT: bool, H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if EQUIPMENT {
        draw_equipment(game);
    } else {
        draw_map(game);
    }
    crate::menu!(game.PauseScreenMode = u16::from(EQUIPMENT));
    game.fade(true);
    if game.ram().read_byte(crate::ram::mirror::INIDISP) == 15 {
        crate::screen_fade::set_both(game, 0, 0);
        let mut selected = crate::menu!(game.PausemenuButtonLabelMode);
        if crate::menu!(game.PausemenuButtonLabelMode) != 0 {
            selected = 1;
        }
        crate::menu!(game.MenuIndex = selected);
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
        crate::menu!(game.PauseScreenMode = 0);
    }
    game.fade(false);
    if game.ram().read_byte(crate::ram::mirror::INIDISP) == 0x80 {
        enable(game);
        crate::screen_fade::set_both(game, 0, 0);
        crate::menu!(game.MenuIndex += 1);
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
        scroll!(game.Bg1V = 0);
        crate::menu!(game.PauseScreenMode = 1);
    }
    menu_tilemap::pause_labels(crate::menu!(game.PausemenuButtonLabelMode), game);
    crate::menu!(game.PausemenuLrAnimationFrame = 0);
    crate::menu!(
        game.PausemenuLrAnimationTimer = menu_tilemap::native_record(0x82c10c, game).word(0)
    );
    crate::screen_fade::set_both(game, 1, 1);
    if !equipment {
        crate::menu!(game.PauseScreenMode = 0);
    }
    crate::menu!(game.MenuIndex += 1);
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
    if crate::menu!(game.PauseScreenMode) == 1 {
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
    scroll!(game.Bg1H = camera.bg1_offset[0].wrapping_add(camera.layer1[0]));
    scroll!(game.Bg1V = camera.bg1_offset[1].wrapping_add(camera.layer1[1]));
    scroll!(game.Bg2H = camera.bg2_scroll[0].wrapping_add(camera.layer2[0]));
    scroll!(game.Bg2V = camera.bg2_scroll[1].wrapping_add(camera.layer2[1]));
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

fn advance<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    shoulder_input(game);
    map_menu::animate_pause_sprite(2, 0x18, 0xd0, game);
    map_menu::animate_pause_sprite(2, 0xe8, 0xd0, game);
    if word!(game.NEWLY_HELD_DOWN_TIMED_HELD_INPUT) & K_BUTTON_START as u16 != 0 {
        game.queue_sound(0, 0x38_u8, 6);
        crate::screen_fade::set_both(game, 1, 1);
        let previous = crate::menu!(game.PausemenuButtonLabelMode);
        crate::menu!(game.PausemenuButtonLabelMode = 1);
        menu_tilemap::pause_labels(crate::menu!(game.PausemenuButtonLabelMode), game);
        crate::menu!(game.PausemenuButtonLabelMode = previous);
        crate::menu!(game.PausemenuStartLrPressedHighlightTimer = 11);
        word!(game.GAME_STATE += 1);
    }
    menu_tilemap::queue_pause_labels(game);
}

fn shoulder_input<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let (menu, highlight, mode) =
        if word!(game.NEWLY_HELD_DOWN_TIMED_HELD_INPUT) & K_BUTTON_L as u16 != 0 {
            if crate::menu!(game.PausemenuButtonLabelMode) == 0 {
                return;
            }
            (5, 1, 0)
        } else if word!(game.NEWLY_HELD_DOWN_TIMED_HELD_INPUT) & K_BUTTON_R as u16 != 0
            && crate::menu!(game.PausemenuButtonLabelMode) != 2
        {
            (2, 2, 2)
        } else {
            return;
        };
    crate::menu!(game.PausemenuStartLrPressedHighlightTimer = 5);
    crate::menu!(game.MenuIndex = menu);
    crate::menu!(game.PausemenuShoulderButtonHighlight = highlight);
    crate::menu!(game.PausemenuButtonLabelMode = mode);
    menu_tilemap::pause_labels(crate::menu!(game.PausemenuButtonLabelMode), game);
    game.queue_sound(0, 0x38_u8, 6);
}

fn tick_highlight<H: crate::menus::host::Host + ?Sized>(game: &mut H) -> bool {
    if crate::menu!(game.PausemenuStartLrPressedHighlightTimer) == 0 {
        return false;
    }
    crate::menu!(game.PausemenuStartLrPressedHighlightTimer -= 1);
    true
}

pub fn draw_shoulder<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if tick_highlight(game) && crate::menu!(game.PausemenuShoulderButtonHighlight) != 0 {
        let index = (i32::from(crate::menu!(game.PausemenuShoulderButtonHighlight)) - 1) as usize;
        let (sprite, x) = [(0x28, 0x18), (0x29, 0xe8)][index];
        crate::sprite_output::menu(sprite, x, 0xcf, 0, game);
    }
}

fn draw_start<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if tick_highlight(game) {
        crate::sprite_output::menu(0x2b, 0x90, 0xd0, 0, game);
    }
}
