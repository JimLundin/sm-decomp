//! Pause-screen lifecycle, input and saved graphics; RAM remains authoritative.
#![forbid(unsafe_code)]
use crate::game::{
    constants::buttons::{kButton_L, kButton_R, kButton_Start},
    display_refresh, equipment_menu, map_menu, menu_tilemap,
    music::QueueSfx1_Max6,
    projectile_presentation_ram as sprite_output,
};
use crate::game_state::Game;
use crate::ram_map::{word, words as fields};

pub(crate) fn hook_address(pausing: bool, game: &Game) -> u32 {
    let offset = if pausing { 0x601 } else { 0x604 };
    // Load24 reads four native bytes, including the following field, before masking.
    let mut bytes = [0; 4];
    game.ram().copy_out(offset, &mut bytes);
    u32::from_ne_bytes(bytes) & 0xffffff
}

pub(crate) fn initialize(game: &mut Game) {
    map_menu::reset_animations(game);
    menu_tilemap::equipment_icons(game);
    menu_tilemap::pause_labels(0, game);
    menu_tilemap::queue_pause_labels(game);
    map_menu::scroll_limits(game);
    map_menu::center_map(0x80, game);
    word! { game, 1; screen_fade_delay, screen_fade_counter }
    word! { game; pausemenu_leftarrow_animation_frame = 0; pausemenu_palette_animation_timer = 1; }
    word! { game, 0; map_scrolling_direction, map_scrolling_speed_index }
    display_refresh::queue_fx_clear(game);
    word!(game.game_state += 1);
}

pub(crate) fn fade_in(game: &mut Game) -> u8 {
    if word!(game.menu_index) != 0 {
        draw_equipment(game);
    } else {
        draw_map(game);
    }
    display_refresh::fade(true, game);
    if game.ram().read_byte(0x51) == 15 {
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.game_state += 1);
    }
    0
}

pub(crate) fn update_map(game: &mut Game) {
    advance(game);
    map_menu::scroll_arrows(map_menu::MapScreen::Pause, game);
    map_menu::scroll(game);
    draw_map(game);
    word!(game.pause_screen_mode = 0);
}

pub(crate) fn update_equipment(game: &mut Game) {
    word! { game, 0; reg_BG1HOFS, reg_BG1VOFS }
    equipment_menu::update(game);
    advance(game);
    word!(game.pause_screen_mode = 1);
}

pub(crate) fn fade_tab<const EQUIPMENT: bool>(game: &mut Game) {
    if EQUIPMENT {
        draw_equipment(game);
    } else {
        draw_map(game);
    }
    word!(game.pause_screen_mode = u16::from(EQUIPMENT));
    display_refresh::fade(true, game);
    if game.ram().read_byte(0x51) == 15 {
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        let mut selected = word!(game.pausemenu_button_label_mode);
        if word!(game.pausemenu_button_label_mode) != 0 {
            selected = 1;
        }
        word!(game.menu_index = selected);
    }
}

pub(crate) fn fade_out_tab(game: &mut Game, equipment: bool, enable: fn(&mut Game)) {
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
    display_refresh::fade(false, game);
    if game.ram().read_byte(0x51) == 0x80 {
        enable(game);
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.menu_index += 1);
    }
}

pub(crate) fn load_tab(game: &mut Game, equipment: bool, upload: impl FnOnce(&mut Game)) {
    map_menu::display_elevator_destinations(game);
    if equipment {
        crate::game::reserve_tanks::prepare_menu(game);
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

fn draw_map(game: &mut Game) {
    map_menu::draw_player_marker(game);
    map_menu::draw_icons(map_menu::IconMode::Pause, game);
    map_menu::display_elevator_destinations(game);
}

pub(crate) fn draw_equipment(game: &mut Game) {
    map_menu::draw_equipment_selector(game);
    menu_tilemap::reserve_tanks(game);
}

pub(crate) fn draw_unpause(game: &mut Game) {
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

pub(crate) fn resume(game: &mut Game) {
    word!(game.reg_BG1HOFS = word!(game.bg1_x_offset).wrapping_add(word!(game.layer1_x_pos)));
    word!(game.reg_BG1VOFS = word!(game.bg1_y_offset).wrapping_add(word!(game.layer1_y_pos)));
    word!(game.reg_BG2HOFS = word!(game.bg2_x_scroll).wrapping_add(word!(game.layer2_x_pos)));
    word!(game.reg_BG2VOFS = word!(game.bg2_y_scroll).wrapping_add(word!(game.layer2_y_pos)));
    crate::game::visual_runtime::update_beam_tiles_and_palette(game);
    // Palette timer/frame and pause_screen_mode survive this ordered reset.
    word_transfers!(game, ascending, 10; little(fields::menu_index) <- value(0));
    word_transfers!(game, ascending, 9; little(fields::UNUSED_word_7E073D) <- value(0));
    word_transfers!(game, ascending, 9; little(fields::pausemenu_shoulder_button_highlight) <- value(0));
    crate::game::session_runtime::player_command(0xc, game);
}

const GRAPHICS_MIRRORS: [usize; 17] = [
    0x58, 0x59, 0x5a, 0x5d, 0x5e, 0x52, 0xb1, 0xb5, 0xb9, 0xb3, 0xb7, 0xbb, 0x55, 0x91b, 0x91c,
    0x57, 0x71,
];

pub(crate) fn save_graphics(game: &mut Game) {
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
        game.ram().write_byte(0x765 + index, value);
    }
}

pub(crate) fn restore_graphics(game: &mut Game) {
    for index in [13, 14, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 16] {
        let value = game.ram().read_byte(0x765 + index);
        game.ram().write_byte(GRAPHICS_MIRRORS[index], value);
    }
}

fn advance(game: &mut Game) {
    shoulder_input(game);
    map_menu::animate_pause_sprite(2, 0x18, 0xd0, game);
    map_menu::animate_pause_sprite(2, 0xe8, 0xd0, game);
    if word!(game.newly_held_down_timed_held_input) & kButton_Start as u16 != 0 {
        QueueSfx1_Max6(0x38, game);
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

fn shoulder_input(game: &mut Game) {
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
    QueueSfx1_Max6(0x38, game);
}

fn tick_highlight(game: &mut Game) -> bool {
    if word!(game.pausemenu_start_lr_pressed_highlight_timer) == 0 {
        return false;
    }
    word!(game.pausemenu_start_lr_pressed_highlight_timer -= 1);
    true
}

pub(crate) fn draw_shoulder(game: &mut Game) {
    if tick_highlight(game) && word!(game.pausemenu_shoulder_button_highlight) != 0 {
        let index = (i32::from(word!(game.pausemenu_shoulder_button_highlight)) - 1) as usize;
        let (sprite, x) = [(0x28, 0x18), (0x29, 0xe8)][index];
        sprite_output::menu(sprite, x, 0xcf, 0, game);
    }
}

fn draw_start(game: &mut Game) {
    if tick_highlight(game) {
        sprite_output::menu(0x2b, 0x90, 0xd0, 0, game);
    }
}
