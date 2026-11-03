//! Pause-screen lifecycle, input and saved graphics; RAM remains authoritative.
use super::prelude::*;
use super::{equipment_menu, map_menu, menu_tilemap};
use crate::menu_state::MenuScreens;
use crate::menu_state::MenuState;
use crate::projection::Field;

pub fn hook_address<H: crate::menus::host::Host + ?Sized>(pausing: bool, game: &H) -> u32 {
    // The hooks' six bytes: the pause hook's long address, then the unpause
    // hook's. Load24 also read the byte after the hook, which its mask drops.
    let hooks: Vec<u8> = (0..3)
        .flat_map(|index| game.view_word(Field::PauseHook(index)).to_le_bytes())
        .collect();
    let hook = if pausing { &hooks[..3] } else { &hooks[3..] };
    u32::from_le_bytes([hook[0], hook[1], hook[2], 0])
}

pub fn initialize<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    crate::menus::state::with(game, |game, screens: &mut MenuScreens| {
        map_menu::reset_animations(&mut screens.map, menu, game);
        menu_tilemap::equipment_icons(game);
        menu_tilemap::pause_labels(0, game);
        menu_tilemap::queue_pause_labels(game);
        map_menu::scroll_limits(screens, game);
        map_menu::center_map(0x80, &screens.map, game);
        crate::screen_fade::store(game, 1, 1);
        menu.pausemenu_leftarrow_animation_frame = 0;
        menu.pausemenu_palette_animation_timer = 1;
        screens.map.scrolling_direction = 0;
        screens.map.scrolling_speed_index = 0;
    });
    crate::display_refresh::queue_fx_clear(game);
    crate::gameplay_word!(game.GAME_STATE += 1);
}

pub fn fade_in<H: crate::menus::host::Host + ?Sized>(game: &mut H) -> u8 {
    crate::menus::state::with(game, |game, menu: &mut MenuState| {
        if menu.menu_index != 0 {
            draw_equipment(menu, game);
        } else {
            crate::menus::state::with(game, |game, screens| draw_map(menu, game, screens));
        }
        game.fade(true);
        if game.display_control_byte(crate::display::preparation::Field::Brightness) == 15 {
            crate::screen_fade::store(game, 0, 0);
            crate::gameplay_word!(game.GAME_STATE += 1);
        }
        0
    })
}

pub fn update_map<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    advance(menu, game);
    crate::menus::state::with(game, |game, screens: &mut MenuScreens| {
        map_menu::scroll_arrows(map_menu::MapScreen::Pause, &mut screens.map, menu, game);
        map_menu::scroll(&mut screens.map, game);
        draw_map(menu, game, screens);
    });
    menu.pause_screen_mode = 0;
}

pub fn update_equipment<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    display_scroll! { game, 0; Bg1H, Bg1V }
    equipment_menu::update(menu, game);
    advance(menu, game);
    menu.pause_screen_mode = 1;
}

pub fn fade_tab<const EQUIPMENT: bool, H: crate::menus::host::Host + ?Sized>(
    menu: &mut MenuState,
    game: &mut H,
) {
    if EQUIPMENT {
        draw_equipment(menu, game);
    } else {
        crate::menus::state::with(game, |game, screens| draw_map(menu, game, screens));
    }
    menu.pause_screen_mode = u16::from(EQUIPMENT);
    game.fade(true);
    if game.display_control_byte(crate::display::preparation::Field::Brightness) == 15 {
        crate::screen_fade::store(game, 0, 0);
        let mut selected = menu.pausemenu_button_label_mode;
        if menu.pausemenu_button_label_mode != 0 {
            selected = 1;
        }
        menu.menu_index = selected;
    }
}

pub fn fade_out_tab<H: crate::menus::host::Host + ?Sized>(
    menu: &mut MenuState,
    game: &mut H,
    equipment: bool,
    enable: fn(&mut H),
) {
    if equipment {
        draw_equipment(menu, game);
    } else {
        crate::menus::state::with(game, |game, screens: &mut MenuScreens| {
            map_menu::display_elevator_destinations(game);
            map_menu::draw_player_marker(&mut screens.map, game);
            map_menu::draw_icons(map_menu::IconMode::Pause, screens, menu, game);
        });
    }
    draw_shoulder(menu, game);
    // Equipment fade preserves this mode; map fade resets it after drawing.
    if !equipment {
        menu.pause_screen_mode = 0;
    }
    game.fade(false);
    if game.display_control_byte(crate::display::preparation::Field::Brightness) == 0x80 {
        enable(game);
        crate::screen_fade::store(game, 0, 0);
        menu.menu_index = menu.menu_index.wrapping_add(1);
    }
}

pub fn load_tab<H: crate::menus::host::Host + ?Sized>(
    menu: &mut MenuState,
    game: &mut H,
    equipment: bool,
    upload: impl FnOnce(&mut H),
) {
    map_menu::display_elevator_destinations(game);
    if equipment {
        crate::menus::reserve_tanks::prepare_menu(menu, game);
    }
    upload(game);
    if equipment {
        display_scroll!(game.Bg1V = 0);
        menu.pause_screen_mode = 1;
    }
    menu_tilemap::pause_labels(menu.pausemenu_button_label_mode, game);
    menu.pausemenu_lr_animation_frame = 0;
    menu.pausemenu_lr_animation_timer = menu_tilemap::native_record(0x82c10c, game).word(0);
    crate::screen_fade::store(game, 1, 1);
    if !equipment {
        menu.pause_screen_mode = 0;
    }
    menu.menu_index = menu.menu_index.wrapping_add(1);
}

fn draw_map<H: crate::menus::host::Host + ?Sized>(
    menu: &mut MenuState,
    game: &mut H,
    screens: &mut MenuScreens,
) {
    map_menu::draw_player_marker(&mut screens.map, game);
    map_menu::draw_icons(map_menu::IconMode::Pause, screens, menu, game);
    map_menu::display_elevator_destinations(game);
}

pub fn draw_equipment<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    map_menu::draw_equipment_selector(menu, game);
    menu_tilemap::reserve_tanks(menu, game);
}

pub fn draw_unpause<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    draw_start(menu, game);
    if menu.pause_screen_mode == 1 {
        draw_equipment(menu, game);
        draw_shoulder(menu, game);
    } else {
        // Unpause keeps the opposite icon/marker order from the active map.
        crate::menus::state::with(game, |game, screens: &mut MenuScreens| {
            map_menu::display_elevator_destinations(game);
            map_menu::draw_icons(map_menu::IconMode::Pause, screens, menu, game);
            map_menu::draw_player_marker(&mut screens.map, game);
        });
    }
}

pub fn resume<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let camera = |game: &H, offset: fn(usize) -> Field, layer: fn(usize) -> Field, axis| {
        game.view_word(offset(axis))
            .wrapping_add(game.view_word(layer(axis)))
    };
    let bg1 = [0, 1].map(|axis| camera(game, Field::Bg1Offset, Field::Layer1, axis));
    let bg2 = [0, 1].map(|axis| camera(game, Field::Bg2Scroll, Field::Layer2, axis));
    display_scroll!(game.Bg1H = bg1[0]);
    display_scroll!(game.Bg1V = bg1[1]);
    display_scroll!(game.Bg2H = bg2[0]);
    display_scroll!(game.Bg2V = bg2[1]);
    game.update_beam_tiles_and_palette();
    // Palette timer/frame and pause_screen_mode survive this ordered reset.
    game.reset_pause_menu();
    game.player_command(0xc);
}

/// The saved graphics bytes, in order: the BG1..BG3 tilemap and BG1..BG4
/// tile base mirrors, the object layout, the six scroll mirrors' low bytes,
/// the BG mode, the layer-2 scroll bytes, the mosaic and the next colour math.
const GRAPHICS_BYTES: usize = 17;

pub fn save_graphics<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    for index in 0..GRAPHICS_BYTES {
        // The six scroll mirrors are read as LE words, then truncated to a byte.
        let value = if (6..12).contains(&index) {
            let word = [
                crate::display_scroll::Word::Bg1H,
                crate::display_scroll::Word::Bg2H,
                crate::display_scroll::Word::Bg3H,
                crate::display_scroll::Word::Bg1V,
                crate::display_scroll::Word::Bg2V,
                crate::display_scroll::Word::Bg3V,
            ][index - 6];
            let low = game.physical_scroll_byte(word, false);
            let _high = game.physical_scroll_byte(word, true);
            low
        } else {
            let field = graphics_field(index);
            match field {
                Some(field) => game.display_control_byte(field),
                // The layer-2 scroll bytes, each its field's low byte.
                None => game.view_word(layer2_scroll(index)) as u8,
            }
        };
        super::state::with(game, |_, graphics: &mut crate::menu_state::MenuGraphics| {
            graphics.bytes[index] = value;
        });
    }
}

pub fn restore_graphics<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    for index in [13, 14, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 16] {
        let value = super::state::load::<crate::menu_state::MenuGraphics, _>(game).bytes[index];
        if (6..12).contains(&index) {
            let word = [
                crate::display_scroll::Word::Bg1H,
                crate::display_scroll::Word::Bg2H,
                crate::display_scroll::Word::Bg3H,
                crate::display_scroll::Word::Bg1V,
                crate::display_scroll::Word::Bg2V,
                crate::display_scroll::Word::Bg3V,
            ][index - 6];
            game.write_physical_scroll_byte(word, false, value);
        } else {
            match graphics_field(index) {
                Some(field) => game.store_display_control_byte(
                    field,
                    value,
                    crate::display::preparation::Store::Owned,
                ),
                None => game.set_view_low_byte(layer2_scroll(index), value),
            }
        }
    }
}

fn advance<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    shoulder_input(menu, game);
    map_menu::animate_pause_sprite(2, 0x18, 0xd0, menu, game);
    map_menu::animate_pause_sprite(2, 0xe8, 0xd0, menu, game);
    if crate::gameplay_word!(game.NEWLY_HELD_DOWN_TIMED_HELD_INPUT) & K_BUTTON_START as u16 != 0 {
        game.queue_sound(0, 0x38_u8, 6);
        crate::screen_fade::store(game, 1, 1);
        let previous = menu.pausemenu_button_label_mode;
        menu.pausemenu_button_label_mode = 1;
        menu_tilemap::pause_labels(menu.pausemenu_button_label_mode, game);
        menu.pausemenu_button_label_mode = previous;
        menu.pausemenu_start_lr_pressed_highlight_timer = 11;
        crate::gameplay_word!(game.GAME_STATE += 1);
    }
    menu_tilemap::queue_pause_labels(game);
}

fn shoulder_input<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    let (index, highlight, mode) =
        if crate::gameplay_word!(game.NEWLY_HELD_DOWN_TIMED_HELD_INPUT) & K_BUTTON_L as u16 != 0 {
            if menu.pausemenu_button_label_mode == 0 {
                return;
            }
            (5, 1, 0)
        } else if crate::gameplay_word!(game.NEWLY_HELD_DOWN_TIMED_HELD_INPUT) & K_BUTTON_R as u16
            != 0
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

/// The layer-2 scroll byte saved graphics byte `index` (13 or 14) holds.
fn layer2_scroll(index: usize) -> Field {
    if index == 13 {
        Field::Layer2ScrollX
    } else {
        Field::Layer2ScrollY
    }
}

fn graphics_field(index: usize) -> Option<crate::display::preparation::Field> {
    use crate::display::preparation::Field;
    match index {
        0 => Some(Field::Screens0),
        1 => Some(Field::Screens1),
        2 => Some(Field::Screens2),
        3 => Some(Field::Characters0),
        4 => Some(Field::Characters1),
        5 => Some(Field::ObjectLayout),
        12 => Some(Field::Mode),
        15 => Some(Field::Mosaic),
        16 => Some(Field::NextColorMath),
        _ => None,
    }
}
