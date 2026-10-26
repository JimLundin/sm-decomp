//! Options selection, native setting words and their tilemap projection.
#![forbid(unsafe_code)]
use crate::game::{constants::buttons::*, controller_input, menu_tilemap, music::QueueSfx1_Max6};
use crate::game_state::Game;
use crate::ram_map::word;

fn navigate(last: u16, game: &mut Game) {
    if word!(game.joypad1_newkeys) & kButton_Up as u16 != 0 {
        QueueSfx1_Max6(0x37, game);
        if word!(game.menu_option_index -= 1, reread) & 0x8000 != 0 {
            word!(game.menu_option_index = last);
        }
    } else if word!(game.joypad1_newkeys) & kButton_Down as u16 != 0 {
        QueueSfx1_Max6(0x37, game);
        if word!(game.menu_option_index += 1, reread) == last + 1 {
            word!(game.menu_option_index = 0);
        }
    }
}

pub(crate) fn main_menu(game: &mut Game) {
    navigate(4, game);
    if word!(game.joypad1_newkeys) & kButton_B as u16 != 0 {
        word!(game.game_options_screen_index = 11);
    } else if word!(game.joypad1_newkeys) & (kButton_A | kButton_Start) as u16 != 0 {
        QueueSfx1_Max6(0x38, game);
        const ACTIONS: [fn(&mut Game); 5] =
            [start, toggle_language, toggle_language, dissolve, dissolve];
        ACTIONS[usize::from(word!(game.menu_option_index))](game);
    }
}

fn start(game: &mut Game) {
    if word!(game.enable_debug) != 0 && word!(game.joypad1_lastkeys) & kButton_L as u16 == 0
        || word!(game.loading_game_state) == 5
    {
        word!(game.game_options_screen_index = 4);
    } else {
        word! { game, 0; screen_fade_delay, screen_fade_counter }
        word!(game.game_options_screen_index = 12);
    }
}

fn dissolve(game: &mut Game) {
    game.g_ram[0x57] = 3;
    word! { game, 0; screen_fade_delay, screen_fade_counter }
    word!(game.game_options_screen_index = 5);
}

fn back(game: &mut Game) {
    word!(game.menu_option_index = 0);
    dissolve(game);
}

fn toggle_language(game: &mut Game) {
    word!(game.menu_option_index = 0);
    word!(game.japanese_text_flag = u16::from(word!(game.japanese_text_flag) == 0));
    highlight_language(game);
}

pub(crate) fn highlight_language(game: &mut Game) {
    let japanese = word!(game.japanese_text_flag) != 0;
    for (offset, words, dimmed) in [
        (0x288, 12, japanese),
        (0x2c8, 12, japanese),
        (0x348, 25, !japanese),
        (0x388, 25, !japanese),
    ] {
        menu_tilemap::recolor_native(0x3000 + offset, if dimmed { 0x400 } else { 0 }, words, game);
    }
}

pub(crate) fn special(game: &mut Game) {
    navigate(2, game);
    if word!(game.joypad1_newkeys) & kButton_B as u16 != 0 {
        QueueSfx1_Max6(0x38, game);
        back(game);
    } else if word!(game.joypad1_newkeys)
        & (kButton_Start | kButton_Left | kButton_Right | kButton_A) as u16
        != 0
    {
        QueueSfx1_Max6(0x38, game);
        const ACTIONS: [fn(&mut Game); 3] = [toggle_special, toggle_special, back];
        ACTIONS[usize::from(word!(game.menu_option_index))](game);
    }
}

fn setting_address(game: &mut Game) -> usize {
    let address =
        crate::rtl::rom_table_word!(game, 0x82f0ae, word!(game.menu_option_index) as isize);
    if address >= 0x2000 {
        crate::game::session_runtime::legacy_assertion(true);
    }
    usize::from(address)
}

fn toggle_special(game: &mut Game) {
    let address = setting_address(game);
    let value = u16::from_ne_bytes([game.g_ram[address], game.g_ram[address + 1]]);
    game.g_ram[address..address + 2].copy_from_slice(&u16::from(value == 0).to_ne_bytes());
    highlight_special(game);
}

pub(crate) fn highlight_special(game: &mut Game) {
    const ON: [u16; 4] = [0x1e0, 0x220, 0x360, 0x3a0];
    const OFF: [u16; 4] = [0x1ee, 0x22e, 0x36e, 0x3ae];
    // Capture the pair before the pointer lookup; only the first row uses a live index.
    let pair = usize::from(word!(game.menu_option_index).wrapping_mul(4) >> 1);
    let address = setting_address(game);
    let enabled = u16::from_ne_bytes([game.g_ram[address], game.g_ram[address + 1]]) != 0;
    let palette = if enabled { 0 } else { 0x400 };
    let first = usize::from(word!(game.menu_option_index).wrapping_mul(4) >> 1);
    menu_tilemap::recolor_native(0x3000 + usize::from(ON[first]), palette, 6, game);
    menu_tilemap::recolor_native(0x3000 + usize::from(ON[pair + 1]), palette, 6, game);
    menu_tilemap::recolor_native(0x3000 + usize::from(OFF[pair]), palette ^ 0x400, 6, game);
    menu_tilemap::recolor_native(
        0x3000 + usize::from(OFF[pair + 1]),
        palette ^ 0x400,
        6,
        game,
    );
}

pub(crate) fn controller(game: &mut Game) {
    if word!(game.joypad1_newkeys) & kButton_Up as u16 != 0 {
        QueueSfx1_Max6(0x37, game);
        let selected = word!(game.menu_option_index -= 1, reread);
        if word!(game.menu_option_index) & 0x8000 != 0 {
            word! { game; menu_option_index = 8; game_options_screen_index = 9; }
        } else if selected == 6 {
            word!(game.game_options_screen_index = 10);
        }
        return;
    }
    if word!(game.joypad1_newkeys) & kButton_Down as u16 != 0 {
        QueueSfx1_Max6(0x37, game);
        let selected = word!(game.menu_option_index).wrapping_add(1);
        word!(game.menu_option_index = selected);
        if selected == 7 {
            word!(game.game_options_screen_index = 9);
            return;
        }
        if selected == 9 {
            word! { game; menu_option_index = 0; game_options_screen_index = 10; }
        }
    } else if word!(game.joypad1_newkeys) != 0 {
        QueueSfx1_Max6(0x38, game);
        use controller_input::remap_binding as remap;
        const ACTIONS: [fn(&mut Game); 9] = [
            remap,
            remap,
            remap,
            remap,
            remap,
            remap,
            remap,
            accept_bindings,
            reset_bindings,
        ];
        ACTIONS[usize::from(word!(game.menu_option_index))](game);
    } else if word!(game.joypad2_new_keys) != 0
        && word!(game.menu_option_index) == 8
        && (i32::from(word!(game.debug_invincibility)) - 16) & 0x8000 != 0
    {
        const CODE: [u16; 16] = [
            0x20, 0x20, 0x20, 0x20, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x4000, 0x40, 0x40, 0x80,
            0x80, 0x80,
        ];
        if CODE[usize::from(word!(game.debug_invincibility))] & word!(game.joypad2_new_keys)
            == CODE[usize::from(word!(game.debug_invincibility))]
        {
            word!(game.debug_invincibility += 1);
        } else {
            word!(game.debug_invincibility = 0);
        }
    }
}

fn accept_bindings(game: &mut Game) {
    if word!(game.joypad1_newkeys) & (kButton_Start | kButton_A) as u16 != 0
        && controller_input::store_binding_choices(game) == 0
    {
        back(game);
    }
}

fn reset_bindings(game: &mut Game) {
    if word!(game.joypad1_newkeys) & (kButton_Start | kButton_A) as u16 != 0 {
        controller_input::reset_binding_choices(game);
        menu_tilemap::controller_bindings(game);
    }
}

pub(crate) fn scroll<const DOWN: bool>(game: &mut Game) {
    if DOWN {
        word!(game.reg_BG1VOFS += 2);
    } else {
        word!(game.reg_BG1VOFS -= 2);
    }
    if word!(game.reg_BG1VOFS) == if DOWN { 32 } else { 0 } {
        word!(game.game_options_screen_index = 7);
    }
}
