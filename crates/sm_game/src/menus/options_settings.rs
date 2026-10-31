//! Options selection, screen transitions and their tilemap projection.
use super::menu_tilemap;
use super::options_objects::definition;
use super::prelude::*;
use crate::ram::word;

/// The options screen's BG tilemap buffer ($7E:3000, `ram3000`).
const TILEMAP: usize = 0x3000;
/// The language highlight's last tile in that buffer.
const LANGUAGE_HIGHLIGHT_TILE: usize = TILEMAP + 0x3b8;

pub fn initialize<H: Host + ?Sized>(game: &mut H, upload: fn(&mut H)) {
    game.ram().write_byte(crate::ram::mirror::BG12NBA, 0);
    game.ram().write_byte(
        crate::ram::mirror::TM,
        crate::display::control::Layers::BG1_BG2_SPRITES.byte(),
    );
    for mirror in [0x6b, 0x6c, 0x6d, 0x6e, 0x71, 0x6f] {
        game.ram().write_byte(mirror, 0);
    }
    upload(game);
    scroll! { game, 0; Bg1H, Bg1V, Bg2H, Bg2V }
    options_menu!(game.DebugInvincibility = 0);
    crate::word_transfers!(game, descending, 256; le(PALETTE_BUFFER, 255) <- rom(0x8ee400, 255));
    for (index, source) in [0x978df4, 0x978fcd, 0x9791c4, 0x97938d, 0x97953a]
        .into_iter()
        .enumerate()
    {
        crate::decompression::decompress_to_ram(source, 0x1c000 + index * 0x800, game);
    }
    copy_screen(0x1c7fe, false, game);
    // The ROM keeps the final copied tile in A through the object clear.
    let initial_tile = u16::from_le_bytes([
        game.ram().read_byte(TILEMAP),
        game.ram().read_byte(TILEMAP + 1),
    ]);
    options_menu!(game.MenuOptionIndex = 0);
    super::options_objects::clear(game);
    super::options_objects::spawn(initial_tile, definition::MENU_SELECTION_MISSILE, game);
    // The first initializer leaves its palette $0E00 in A for this call.
    super::options_objects::spawn(0x0e00, definition::OPTION_MODE_BORDER, game);
    options_menu!(game.ScreenIndex += 1);
    highlight_language(game);
}

fn navigate<H: Host + ?Sized>(last: u16, game: &mut H) {
    if game.observed_inputs().pressed & K_BUTTON_UP as u16 != 0 {
        game.queue_sound(0, 0x37_u8, 6);
        if options_menu!(game.MenuOptionIndex -= 1, reread) & 0x8000 != 0 {
            options_menu!(game.MenuOptionIndex = last);
        }
    } else if game.observed_inputs().pressed & K_BUTTON_DOWN as u16 != 0 {
        game.queue_sound(0, 0x37_u8, 6);
        if options_menu!(game.MenuOptionIndex += 1, reread) == last + 1 {
            options_menu!(game.MenuOptionIndex = 0);
        }
    }
}

pub fn main_menu<H: Host + ?Sized>(game: &mut H) {
    navigate(4, game);
    if game.observed_inputs().pressed & K_BUTTON_B as u16 != 0 {
        options_menu!(game.ScreenIndex = 11);
    } else if game.observed_inputs().pressed & (K_BUTTON_A | K_BUTTON_START) as u16 != 0 {
        game.queue_sound(0, 0x38_u8, 6);
        let actions: [fn(&mut H); 5] =
            [start, toggle_language, toggle_language, dissolve, dissolve];
        actions[usize::from(options_menu!(game.MenuOptionIndex))](game);
    }
}

fn start<H: Host + ?Sized>(game: &mut H) {
    if word!(game.ENABLE_DEBUG) != 0 && game.observed_inputs().held & K_BUTTON_L as u16 == 0
        || word!(game.LOADING_GAME_STATE) == 5
    {
        options_menu!(game.ScreenIndex = 4);
    } else {
        crate::screen_fade::set_both(game, 0, 0);
        options_menu!(game.ScreenIndex = 12);
    }
}

pub fn finish_start<H: Host + ?Sized>(game: &mut H) {
    options_menu!(game.ScreenIndex = 0);
    if word!(game.ENABLE_DEBUG) != 0 && (game.observed_inputs().held & 0x20) == 0 {
        word!(game.GAME_STATE = 5);
        if word!(game.LOADING_GAME_STATE) != 5 {
            word!(game.LOADING_GAME_STATE = 5);
            super::save_slots::save(save_selection!(game.SelectedSlot), game);
        }
    } else if word!(game.LOADING_GAME_STATE) != 0 {
        word!(game.GAME_STATE = word!(game.LOADING_GAME_STATE));
        if word!(game.LOADING_GAME_STATE) == 34 {
            crate::flow::set(game, crate::flow::Word::CinematicFunction, 0xc11b);
        }
        options_menu! { game, 0; MenuOptionIndex, ScreenIndex }
    } else {
        word!(game.GAME_STATE = 30);
        crate::flow::set(game, crate::flow::Word::CinematicFunction, 0xa395);
        options_menu! { game, 0; MenuOptionIndex }
        crate::screen_fade::set_both(game, 0, 0);
    }
}

fn dissolve<H: Host + ?Sized>(game: &mut H) {
    game.ram().write_byte(crate::ram::mirror::MOSAIC, 3);
    crate::screen_fade::set_both(game, 0, 0);
    options_menu!(game.ScreenIndex = 5);
}

fn back<H: Host + ?Sized>(game: &mut H) {
    options_menu!(game.MenuOptionIndex = 0);
    dissolve(game);
}

fn toggle_language<H: Host + ?Sized>(game: &mut H) {
    crate::probe::hit("options_settings.toggle_language");
    options_menu!(game.MenuOptionIndex = 0);
    word!(game.JAPANESE_TEXT_FLAG = u16::from(word!(game.JAPANESE_TEXT_FLAG) == 0));
    highlight_language(game);
}

pub fn highlight_language<H: Host + ?Sized>(game: &mut H) {
    let japanese = word!(game.JAPANESE_TEXT_FLAG) != 0;
    for (offset, words, dimmed) in [
        (0x288, 12, japanese),
        (0x2c8, 12, japanese),
        (0x348, 25, !japanese),
        (0x388, 25, !japanese),
    ] {
        menu_tilemap::recolor_native(0x3000 + offset, if dimmed { 0x400 } else { 0 }, words, game);
    }
}

pub fn special<H: Host + ?Sized>(game: &mut H) {
    navigate(2, game);
    if game.observed_inputs().pressed & K_BUTTON_B as u16 != 0 {
        game.queue_sound(0, 0x38_u8, 6);
        back(game);
    } else if game.observed_inputs().pressed
        & (K_BUTTON_START | K_BUTTON_LEFT | K_BUTTON_RIGHT | K_BUTTON_A) as u16
        != 0
    {
        game.queue_sound(0, 0x38_u8, 6);
        let actions: [fn(&mut H); 3] = [toggle_special, toggle_special, back];
        actions[usize::from(options_menu!(game.MenuOptionIndex))](game);
    }
}

fn setting_address<H: Host + ?Sized>(game: &mut H) -> usize {
    let address = crate::menu_lists::option_setting_address(
        game,
        options_menu!(game.MenuOptionIndex) as isize,
    );
    if address >= 0x2000 {
        game.legacy_assertion(true);
    }
    usize::from(address)
}

fn toggle_special<H: Host + ?Sized>(game: &mut H) {
    crate::probe::hit("options_settings.toggle_special");
    let address = setting_address(game);
    let value = u16::from_ne_bytes([
        game.ram().read_byte(address),
        game.ram().read_byte(address + 1),
    ]);
    game.ram()
        .copy_in(address, &u16::from(value == 0).to_ne_bytes());
    highlight_special(game);
}

pub fn highlight_special<H: Host + ?Sized>(game: &mut H) {
    const ON: [u16; 4] = [0x1e0, 0x220, 0x360, 0x3a0];
    const OFF: [u16; 4] = [0x1ee, 0x22e, 0x36e, 0x3ae];
    // Capture the pair before the pointer lookup; only the first row uses a live index.
    let pair = usize::from(options_menu!(game.MenuOptionIndex).wrapping_mul(4) >> 1);
    let address = setting_address(game);
    let enabled = u16::from_ne_bytes([
        game.ram().read_byte(address),
        game.ram().read_byte(address + 1),
    ]) != 0;
    let palette = if enabled { 0 } else { 0x400 };
    let first = usize::from(options_menu!(game.MenuOptionIndex).wrapping_mul(4) >> 1);
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

pub fn controller<H: Host + ?Sized>(game: &mut H) {
    if game.observed_inputs().pressed & K_BUTTON_UP as u16 != 0 {
        game.queue_sound(0, 0x37_u8, 6);
        let selected = options_menu!(game.MenuOptionIndex -= 1, reread);
        if options_menu!(game.MenuOptionIndex) & 0x8000 != 0 {
            options_menu! { game; MenuOptionIndex = 8; ScreenIndex = 9; }
        } else if selected == 6 {
            options_menu!(game.ScreenIndex = 10);
        }
        return;
    }
    if game.observed_inputs().pressed & K_BUTTON_DOWN as u16 != 0 {
        game.queue_sound(0, 0x37_u8, 6);
        let selected = options_menu!(game.MenuOptionIndex).wrapping_add(1);
        options_menu!(game.MenuOptionIndex = selected);
        if selected == 7 {
            options_menu!(game.ScreenIndex = 9);
            return;
        }
        if selected == 9 {
            options_menu! { game; MenuOptionIndex = 0; ScreenIndex = 10; }
        }
    } else if game.observed_inputs().pressed != 0 {
        game.queue_sound(0, 0x38_u8, 6);
        let remap: fn(&mut H) = |game| super::controller_input::remap_binding(game);
        let actions: [fn(&mut H); 9] = [
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
        actions[usize::from(options_menu!(game.MenuOptionIndex))](game);
    } else if word!(game.JOYPAD2_NEW_KEYS) != 0
        && options_menu!(game.MenuOptionIndex) == 8
        && (i32::from(options_menu!(game.DebugInvincibility)) - 16) & 0x8000 != 0
    {
        const CODE: [u16; 16] = [
            0x20, 0x20, 0x20, 0x20, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x4000, 0x40, 0x40, 0x80,
            0x80, 0x80,
        ];
        if CODE[usize::from(options_menu!(game.DebugInvincibility))] & word!(game.JOYPAD2_NEW_KEYS)
            == CODE[usize::from(options_menu!(game.DebugInvincibility))]
        {
            options_menu!(game.DebugInvincibility += 1);
        } else {
            options_menu!(game.DebugInvincibility = 0);
        }
    }
}

fn accept_bindings<H: Host + ?Sized>(game: &mut H) {
    if game.observed_inputs().pressed & (K_BUTTON_START | K_BUTTON_A) as u16 != 0
        && super::controller_input::store_binding_choices(game) == 0
    {
        back(game);
    }
}

fn reset_bindings<H: Host + ?Sized>(game: &mut H) {
    if game.observed_inputs().pressed & (K_BUTTON_START | K_BUTTON_A) as u16 != 0 {
        super::controller_input::reset_binding_choices(game);
        menu_tilemap::controller_bindings(game);
    }
}

pub fn scroll<const DOWN: bool, H: Host + ?Sized>(game: &mut H) {
    if DOWN {
        scroll!(game.Bg1V += 2);
    } else {
        scroll!(game.Bg1V -= 2);
    }
    if scroll!(game.Bg1V) == if DOWN { 32 } else { 0 } {
        options_menu!(game.ScreenIndex = 7);
    }
}

pub fn fade_out<H: Host + ?Sized>(game: &mut H, screen: u16, enable: fn(&mut H)) {
    game.fade(false);
    if screen == 5 && game.ram().read_byte(crate::ram::mirror::MOSAIC) != 0xf3 {
        game.ram().write_byte(
            crate::ram::mirror::MOSAIC,
            game.ram()
                .read_byte(crate::ram::mirror::MOSAIC)
                .wrapping_add(16),
        );
    }
    if game.ram().read_byte(crate::ram::mirror::INIDISP) != 0x80 {
        if screen == 0 && game.ram().read_byte(crate::ram::mirror::INIDISP) == 14 {
            let subscreen = u16::from(game.ram().read_byte(crate::ram::mirror::TS));
            if game.ram().read_byte(crate::ram::mirror::TS) & 4 == 0 {
                super::options_objects::spawn(subscreen, definition::SAMUS_DATA_BORDER, game);
            }
        }
        return;
    }
    enable(game);
    crate::screen_fade::set_both(game, 0, 0);
    match screen {
        0 => options_menu!(game.ScreenIndex += 1),
        5 => {
            scroll!(game.Bg1V = 0);
            options_menu!(game.ScreenIndex += 1);
            replace_screen(game);
        }
        11 => {
            word!(game.GAME_STATE = 4);
            crate::menu!(game.MenuIndex = 0);
            options_menu!(game.ScreenIndex = 0);
        }
        _ => options_menu!(game.ScreenIndex = 4),
    }
}

pub fn fade_in<const DISSOLVE: bool, H: Host + ?Sized>(game: &mut H) {
    game.fade(true);
    if DISSOLVE && game.ram().read_byte(crate::ram::mirror::MOSAIC) != 3 {
        game.ram().write_byte(
            crate::ram::mirror::MOSAIC,
            game.ram()
                .read_byte(crate::ram::mirror::MOSAIC)
                .wrapping_sub(16),
        );
    }
    if game.ram().read_byte(crate::ram::mirror::INIDISP) != 15 {
        return;
    }
    if DISSOLVE {
        game.ram().write_byte(crate::ram::mirror::MOSAIC, 0);
    }
    crate::screen_fade::set_both(game, 0, 0);
    if !DISSOLVE {
        options_menu!(game.ScreenIndex += 1);
    } else if options_menu!(game.MenuOptionIndex) == 0 {
        options_menu!(game.ScreenIndex = 3);
    } else {
        options_menu!(
            game.ScreenIndex = 7 + u16::from(options_menu!(game.MenuOptionIndex) & 4 != 0)
        );
        options_menu!(game.MenuOptionIndex = 0);
    }
}

pub fn replace_screen<H: Host + ?Sized>(game: &mut H) {
    use super::options_objects::spawn;
    if options_menu!(game.MenuOptionIndex) != 0 {
        if options_menu!(game.MenuOptionIndex) & 4 != 0 {
            let japanese = word!(game.JAPANESE_TEXT_FLAG) != 0;
            copy_screen(0x1dffe + usize::from(japanese) * 0x800, !japanese, game);
            options_menu!(game.MenuOptionIndex = 0);
            highlight_special(game);
            options_menu!(game.MenuOptionIndex = 1);
            highlight_special(game);
            options_menu!(game.MenuOptionIndex = 4);
            spawn(4, definition::SPECIAL_SETTING_MODE_BORDER, game);
        } else {
            let source = 0x1cffe + usize::from(word!(game.JAPANESE_TEXT_FLAG) != 0) * 0x800;
            copy_screen(source, true, game);
            // The controller initializer receives the final native word copied to tile zero.
            let tile = u16::from_ne_bytes([
                game.ram().read_byte(TILEMAP),
                game.ram().read_byte(TILEMAP + 1),
            ]);
            spawn(tile, definition::CONTROLLER_SETTING_MODE_BORDER, game);
            super::controller_input::load_binding_choices(game);
            menu_tilemap::controller_bindings(game);
        }
    } else {
        copy_screen(0x1c7fe, false, game);
        highlight_language(game);
        // The ROM passes the final language-highlight tile left in A (82:EFD7).
        let tile = u16::from_le_bytes([
            game.ram().read_byte(LANGUAGE_HIGHLIGHT_TILE),
            game.ram().read_byte(LANGUAGE_HIGHLIGHT_TILE + 1),
        ]);
        spawn(tile, definition::OPTION_MODE_BORDER, game);
    }
}

// Descending native destination words; Japanese/special and main sources retain LE reads.
pub fn copy_screen<H: Host + ?Sized>(last_word: usize, native: bool, game: &mut H) {
    let source = crate::word_transfers::WordEndpoint::Ram(last_word as isize, native);
    crate::word_transfers!(game, descending, 1024; pause_map(1023) <- captured(source, 0));
}

pub fn advance<H: Host + ?Sized>(game: &mut H, upload: fn(&mut H), enable: fn(&mut H)) -> u8 {
    // Keep the original thirteen-entry bounds failure ahead of all menu effects.
    let screen =
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12][usize::from(options_menu!(game.ScreenIndex))];
    match screen {
        1 => initialize(game, upload),
        2 => fade_in::<false, H>(game),
        3 => main_menu(game),
        4 => finish_start(game),
        6 => fade_in::<true, H>(game),
        7 => controller(game),
        8 => special(game),
        9 => scroll::<true, H>(game),
        10 => scroll::<false, H>(game),
        _ => fade_out(game, screen, enable),
    }
    super::options_objects::advance_and_draw(game);
    if (options_menu!(game.ScreenIndex) as i32 - 2i32) & 0x8000 == 0 {
        super::menu_tilemap::queue_options(game);
    }
    0
}
