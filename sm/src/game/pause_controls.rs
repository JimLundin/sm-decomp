//! Pause-screen input, saved graphics and transient highlights; RAM remains authoritative.
#![forbid(unsafe_code)]
use crate::game::{
    constants::buttons::{kButton_L, kButton_R, kButton_Start},
    map_menu, menu_tilemap,
    music::QueueSfx1_Max6,
    projectile_presentation_ram as sprite_output,
};
use crate::game_state::Game;
use crate::ram_map::word;

const GRAPHICS_MIRRORS: [usize; 17] = [
    0x58, 0x59, 0x5a, 0x5d, 0x5e, 0x52, 0xb1, 0xb5, 0xb9, 0xb3, 0xb7, 0xbb, 0x55, 0x91b, 0x91c,
    0x57, 0x71,
];

pub(crate) fn save_graphics(game: &mut Game) {
    for (index, source) in GRAPHICS_MIRRORS.into_iter().enumerate() {
        // The six scroll mirrors are read as LE words, then truncated to a byte.
        let value = if (6..12).contains(&index) {
            u16::from_le_bytes([game.g_ram[source], game.g_ram[source + 1]]) as u8
        } else {
            game.g_ram[source]
        };
        game.g_ram[0x765 + index] = value;
    }
}

pub(crate) fn restore_graphics(game: &mut Game) {
    for index in [13, 14, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 16] {
        game.g_ram[GRAPHICS_MIRRORS[index]] = game.g_ram[0x765 + index];
    }
}

pub(crate) fn advance(game: &mut Game) {
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

pub(crate) fn draw_start(game: &mut Game) {
    if tick_highlight(game) {
        sprite_output::menu(0x2b, 0x90, 0xd0, 0, game);
    }
}
