//! Options selection, screen transitions and their tilemap projection.
#![forbid(unsafe_code)]
use crate::game::{constants::buttons::*, controller_input, menu_tilemap, music::QueueSfx1_Max6};
use crate::game_state::Game;
use crate::ram_map::word;

pub(crate) fn initialize(game: &mut Game, upload: fn(&mut Game)) {
    game.g_ram[0x5d] = 0;
    game.g_ram[0x69] = 19;
    for mirror in [0x6b, 0x6c, 0x6d, 0x6e, 0x71, 0x6f] {
        game.g_ram[mirror] = 0;
    }
    upload(game);
    word! { game, 0;
        reg_BG1HOFS, reg_BG1VOFS, reg_BG2HOFS, reg_BG2VOFS, debug_invincibility,
    }
    word_transfers!(game, descending, 256; le(palette_buffer, 255) <- rom(0x8ee400, 255));
    for (index, source) in [0x978df4, 0x978fcd, 0x9791c4, 0x97938d, 0x97953a]
        .into_iter()
        .enumerate()
    {
        crate::game::decompression::decompress_to_ram(source, 0x1c000 + index * 0x800, game);
    }
    copy_screen(0x1c7fe, false, game);
    // The ROM keeps the final copied tile in A through the object clear.
    let initial_tile = u16::from_le_bytes([game.g_ram[0x3000], game.g_ram[0x3001]]);
    word!(game.menu_option_index = 0);
    crate::game::options_objects::clear(game);
    crate::game::options_objects::spawn(initial_tile, 0xf4b8, game);
    // The first initializer leaves its palette $0E00 in A for this call.
    crate::game::options_objects::spawn(0x0e00, 0xf4c4, game);
    word!(game.game_options_screen_index += 1);
    highlight_language(game);
}

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

pub(crate) fn fade_out(game: &mut Game, screen: u16, enable: fn(&mut Game)) {
    crate::game::display_refresh::fade(false, game);
    if screen == 5 && game.g_ram[0x57] != 0xf3 {
        game.g_ram[0x57] = game.g_ram[0x57].wrapping_add(16);
    }
    if game.g_ram[0x51] != 0x80 {
        if screen == 0 && game.g_ram[0x51] == 14 {
            let subscreen = u16::from(game.g_ram[0x6b]);
            if game.g_ram[0x6b] & 4 == 0 {
                crate::game::options_objects::spawn(subscreen, 0xf4d6, game);
            }
        }
        return;
    }
    enable(game);
    word! { game, 0; screen_fade_delay, screen_fade_counter }
    match screen {
        0 => word!(game.game_options_screen_index += 1),
        5 => {
            word!(game.reg_BG1VOFS = 0);
            word!(game.game_options_screen_index += 1);
            replace_screen(game);
        }
        11 => {
            word!(game.game_state = 4);
            word! { game, 0; menu_index, game_options_screen_index }
        }
        _ => word!(game.game_options_screen_index = 4),
    }
}

pub(crate) fn fade_in<const DISSOLVE: bool>(game: &mut Game) {
    crate::game::display_refresh::fade(true, game);
    if DISSOLVE && game.g_ram[0x57] != 3 {
        game.g_ram[0x57] = game.g_ram[0x57].wrapping_sub(16);
    }
    if game.g_ram[0x51] != 15 {
        return;
    }
    if DISSOLVE {
        game.g_ram[0x57] = 0;
    }
    word! { game, 0; screen_fade_delay, screen_fade_counter }
    if !DISSOLVE {
        word!(game.game_options_screen_index += 1);
    } else if word!(game.menu_option_index) == 0 {
        word!(game.game_options_screen_index = 3);
    } else {
        word!(
            game.game_options_screen_index = 7 + u16::from(word!(game.menu_option_index) & 4 != 0)
        );
        word!(game.menu_option_index = 0);
    }
}

pub(crate) fn replace_screen(game: &mut Game) {
    use crate::game::options_objects::spawn;
    if word!(game.menu_option_index) != 0 {
        if word!(game.menu_option_index) & 4 != 0 {
            let japanese = word!(game.japanese_text_flag) != 0;
            copy_screen(0x1dffe + usize::from(japanese) * 0x800, !japanese, game);
            word!(game.menu_option_index = 0);
            highlight_special(game);
            word!(game.menu_option_index = 1);
            highlight_special(game);
            word!(game.menu_option_index = 4);
            spawn(4, 0xf4d0, game);
        } else {
            let source = 0x1cffe + usize::from(word!(game.japanese_text_flag) != 0) * 0x800;
            copy_screen(source, true, game);
            // The controller initializer receives the final native word copied to tile zero.
            let tile = u16::from_ne_bytes([game.g_ram[0x3000], game.g_ram[0x3001]]);
            spawn(tile, 0xf4ca, game);
            controller_input::load_binding_choices(game);
            menu_tilemap::controller_bindings(game);
        }
    } else {
        copy_screen(0x1c7fe, false, game);
        highlight_language(game);
        // The ROM passes the final language-highlight tile left in A (82:EFD7).
        let tile = u16::from_le_bytes([game.g_ram[0x33b8], game.g_ram[0x33b9]]);
        spawn(tile, 0xf4c4, game);
    }
}

// Descending native destination words; Japanese/special and main sources retain LE reads.
pub(crate) fn copy_screen(last_word: usize, native: bool, game: &mut Game) {
    let source = crate::word_transfers::WordEndpoint::Ram(last_word as isize, native);
    word_transfers!(game, descending, 1024; pause_map(1023) <- captured(source, 0));
}
