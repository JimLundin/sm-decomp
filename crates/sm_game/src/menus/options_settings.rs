//! Options selection, screen transitions and their tilemap projection.
use super::menu_tilemap;
use super::options_objects::definition;
use super::prelude::*;
use crate::menu_state::{OptionsMenu, SaveSelection};
use crate::ram::word;

/// The options screen's BG tilemap buffer ($7E:3000, `ram3000`).
const TILEMAP: usize = 0x3000;
/// The language highlight's last tile in that buffer.
const LANGUAGE_HIGHLIGHT_TILE: usize = TILEMAP + 0x3b8;

pub fn initialize<H: Host + ?Sized>(game: &mut H, options: &mut OptionsMenu, upload: fn(&mut H)) {
    game.ram().write_byte(crate::ram::mirror::BG12NBA, 0);
    game.ram().write_byte(
        crate::ram::mirror::TM,
        crate::display::control::Layers::BG1_BG2_SPRITES.byte(),
    );
    for mirror in [0x6b, 0x6c, 0x6d, 0x6e, 0x71, 0x6f] {
        game.ram().write_byte(mirror, 0);
    }
    upload(game);
    display_scroll! { game, 0; Bg1H, Bg1V, Bg2H, Bg2V }
    options.debug_invincibility = 0;
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
    options.menu_option_index = 0;
    super::options_objects::clear(game);
    super::options_objects::spawn(initial_tile, definition::MENU_SELECTION_MISSILE, game);
    // The first initializer leaves its palette $0E00 in A for this call.
    super::options_objects::spawn(0x0e00, definition::OPTION_MODE_BORDER, game);
    options.screen_index = options.screen_index.wrapping_add(1);
    highlight_language(game);
}

fn navigate<H: Host + ?Sized>(last: u16, options: &mut OptionsMenu, game: &mut H) {
    let option = &mut options.menu_option_index;
    if game.observed_inputs().pressed & K_BUTTON_UP as u16 != 0 {
        game.queue_sound(0, 0x37_u8, 6);
        *option = option.wrapping_sub(1);
        if *option & 0x8000 != 0 {
            *option = last;
        }
    } else if game.observed_inputs().pressed & K_BUTTON_DOWN as u16 != 0 {
        game.queue_sound(0, 0x37_u8, 6);
        *option = option.wrapping_add(1);
        if *option == last + 1 {
            *option = 0;
        }
    }
}

pub fn main_menu<H: Host + ?Sized>(game: &mut H, options: &mut OptionsMenu) {
    navigate(4, options, game);
    if game.observed_inputs().pressed & K_BUTTON_B as u16 != 0 {
        options.screen_index = 11;
    } else if game.observed_inputs().pressed & (K_BUTTON_A | K_BUTTON_START) as u16 != 0 {
        game.queue_sound(0, 0x38_u8, 6);
        let actions: [fn(&mut H, &mut OptionsMenu); 5] =
            [start, toggle_language, toggle_language, dissolve, dissolve];
        actions[usize::from(options.menu_option_index)](game, options);
    }
}

fn start<H: Host + ?Sized>(game: &mut H, options: &mut OptionsMenu) {
    if word!(game.ENABLE_DEBUG) != 0 && game.observed_inputs().held & K_BUTTON_L as u16 == 0
        || crate::menus::state::load::<SaveSelection, _>(game).loading_game_state == 5
    {
        options.screen_index = 4;
    } else {
        crate::screen_fade::store(game, 0, 0);
        options.screen_index = 12;
    }
}

pub fn finish_start<H: Host + ?Sized>(game: &mut H) {
    crate::menus::state::with(game, |_, options: &mut OptionsMenu| {
        options.screen_index = 0
    });
    if word!(game.ENABLE_DEBUG) != 0 && (game.observed_inputs().held & 0x20) == 0 {
        word!(game.GAME_STATE = 5);
        let slot = crate::menus::state::with(game, |_, selection: &mut SaveSelection| {
            let unsaved = selection.loading_game_state != 5;
            selection.loading_game_state = 5;
            unsaved.then_some(selection.selected_slot)
        });
        if let Some(slot) = slot {
            super::save_slots::save(slot, game);
        }
    } else if let loading @ 1.. =
        crate::menus::state::load::<SaveSelection, _>(game).loading_game_state
    {
        word!(game.GAME_STATE = loading);
        if loading == 34 {
            crate::sequence_state::with_flow(game, |_, flow: &mut crate::flow::Flow| {
                flow.cinematic_function = 0xc11b
            });
        }
        crate::menus::state::with(game, |_, options: &mut OptionsMenu| {
            options.menu_option_index = 0;
            options.screen_index = 0;
        });
    } else {
        word!(game.GAME_STATE = 30);
        crate::sequence_state::with_flow(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = 0xa395
        });
        crate::menus::state::with(game, |_, options: &mut OptionsMenu| {
            options.menu_option_index = 0;
        });
        crate::screen_fade::store(game, 0, 0);
    }
}

fn dissolve<H: Host + ?Sized>(game: &mut H, options: &mut OptionsMenu) {
    game.ram().write_byte(crate::ram::mirror::MOSAIC, 3);
    crate::screen_fade::store(game, 0, 0);
    options.screen_index = 5;
}

fn back<H: Host + ?Sized>(game: &mut H, options: &mut OptionsMenu) {
    options.menu_option_index = 0;
    dissolve(game, options);
}

fn toggle_language<H: Host + ?Sized>(game: &mut H, options: &mut OptionsMenu) {
    crate::probe::hit("options_settings.toggle_language");
    options.menu_option_index = 0;
    crate::sequence_state::with_words(game, |_, words: &mut crate::session_words::SessionWords| {
        words.japanese_text_flag = u16::from(words.japanese_text_flag == 0);
    });
    highlight_language(game);
}

pub fn highlight_language<H: Host + ?Sized>(game: &mut H) {
    let japanese = crate::session_words::current(game).japanese_text_flag != 0;
    for (offset, words, dimmed) in [
        (0x288, 12, japanese),
        (0x2c8, 12, japanese),
        (0x348, 25, !japanese),
        (0x388, 25, !japanese),
    ] {
        menu_tilemap::recolor_native(0x3000 + offset, if dimmed { 0x400 } else { 0 }, words, game);
    }
}

pub fn special<H: Host + ?Sized>(game: &mut H, options: &mut OptionsMenu) {
    navigate(2, options, game);
    if game.observed_inputs().pressed & K_BUTTON_B as u16 != 0 {
        game.queue_sound(0, 0x38_u8, 6);
        back(game, options);
    } else if game.observed_inputs().pressed
        & (K_BUTTON_START | K_BUTTON_LEFT | K_BUTTON_RIGHT | K_BUTTON_A) as u16
        != 0
    {
        game.queue_sound(0, 0x38_u8, 6);
        let actions: [fn(&mut H, &mut OptionsMenu); 3] = [toggle_special, toggle_special, back];
        actions[usize::from(options.menu_option_index)](game, options);
    }
}

fn setting_address<H: Host + ?Sized>(option: u16, game: &mut H) -> usize {
    let address = crate::menu_lists::option_setting_address(game, option as isize);
    if address >= 0x2000 {
        game.legacy_assertion(true);
    }
    usize::from(address)
}

fn toggle_special<H: Host + ?Sized>(game: &mut H, options: &mut OptionsMenu) {
    crate::probe::hit("options_settings.toggle_special");
    let address = setting_address(options.menu_option_index, game);
    let value = u16::from_ne_bytes([
        game.ram().read_byte(address),
        game.ram().read_byte(address + 1),
    ]);
    game.ram()
        .copy_in(address, &u16::from(value == 0).to_ne_bytes());
    highlight_special(options.menu_option_index, game);
}

pub fn highlight_special<H: Host + ?Sized>(option: u16, game: &mut H) {
    const ON: [u16; 4] = [0x1e0, 0x220, 0x360, 0x3a0];
    const OFF: [u16; 4] = [0x1ee, 0x22e, 0x36e, 0x3ae];
    let pair = usize::from(option.wrapping_mul(4) >> 1);
    let address = setting_address(option, game);
    let enabled = u16::from_ne_bytes([
        game.ram().read_byte(address),
        game.ram().read_byte(address + 1),
    ]) != 0;
    let palette = if enabled { 0 } else { 0x400 };
    let first = pair;
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

pub fn controller<H: Host + ?Sized>(game: &mut H, options: &mut OptionsMenu) {
    if game.observed_inputs().pressed & K_BUTTON_UP as u16 != 0 {
        game.queue_sound(0, 0x37_u8, 6);
        let selected = options.menu_option_index.wrapping_sub(1);
        options.menu_option_index = selected;
        if selected & 0x8000 != 0 {
            options.menu_option_index = 8;
            options.screen_index = 9;
        } else if selected == 6 {
            options.screen_index = 10;
        }
        return;
    }
    if game.observed_inputs().pressed & K_BUTTON_DOWN as u16 != 0 {
        game.queue_sound(0, 0x37_u8, 6);
        let selected = options.menu_option_index.wrapping_add(1);
        options.menu_option_index = selected;
        if selected == 7 {
            options.screen_index = 9;
            return;
        }
        if selected == 9 {
            options.menu_option_index = 0;
            options.screen_index = 10;
        }
    } else if game.observed_inputs().pressed != 0 {
        game.queue_sound(0, 0x38_u8, 6);
        let remap: fn(&mut H, &mut OptionsMenu) =
            |game, options| super::controller_input::remap_binding(options.menu_option_index, game);
        let actions: [fn(&mut H, &mut OptionsMenu); 9] = [
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
        actions[usize::from(options.menu_option_index)](game, options);
    } else if word!(game.JOYPAD2_NEW_KEYS) != 0
        && options.menu_option_index == 8
        && (i32::from(options.debug_invincibility) - 16) & 0x8000 != 0
    {
        const CODE: [u16; 16] = [
            0x20, 0x20, 0x20, 0x20, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x4000, 0x40, 0x40, 0x80,
            0x80, 0x80,
        ];
        let code = CODE[usize::from(options.debug_invincibility)];
        if code & word!(game.JOYPAD2_NEW_KEYS) == code {
            options.debug_invincibility += 1;
        } else {
            options.debug_invincibility = 0;
        }
    }
}

fn accept_bindings<H: Host + ?Sized>(game: &mut H, options: &mut OptionsMenu) {
    if game.observed_inputs().pressed & (K_BUTTON_START | K_BUTTON_A) as u16 != 0
        && super::controller_input::store_binding_choices(game) == 0
    {
        back(game, options);
    }
}

fn reset_bindings<H: Host + ?Sized>(game: &mut H, _: &mut OptionsMenu) {
    if game.observed_inputs().pressed & (K_BUTTON_START | K_BUTTON_A) as u16 != 0 {
        super::controller_input::reset_binding_choices(game);
        menu_tilemap::controller_bindings(game);
    }
}

pub fn scroll<const DOWN: bool, H: Host + ?Sized>(game: &mut H, options: &mut OptionsMenu) {
    if DOWN {
        display_scroll!(game.Bg1V += 2);
    } else {
        display_scroll!(game.Bg1V -= 2);
    }
    if display_scroll!(game.Bg1V) == if DOWN { 32 } else { 0 } {
        options.screen_index = 7;
    }
}

pub fn fade_out<H: Host + ?Sized>(
    game: &mut H,
    options: &mut OptionsMenu,
    screen: u16,
    enable: fn(&mut H),
) {
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
    crate::screen_fade::store(game, 0, 0);
    match screen {
        0 => options.screen_index = options.screen_index.wrapping_add(1),
        5 => {
            display_scroll!(game.Bg1V = 0);
            options.screen_index = options.screen_index.wrapping_add(1);
            replace_screen(game, options);
        }
        11 => {
            word!(game.GAME_STATE = 4);
            crate::menus::state::with(game, |_, menu: &mut crate::menu_state::MenuState| {
                menu.menu_index = 0
            });
            options.screen_index = 0;
        }
        _ => options.screen_index = 4,
    }
}

pub fn fade_in<const DISSOLVE: bool, H: Host + ?Sized>(game: &mut H, options: &mut OptionsMenu) {
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
    crate::screen_fade::store(game, 0, 0);
    if !DISSOLVE {
        options.screen_index = options.screen_index.wrapping_add(1);
    } else if options.menu_option_index == 0 {
        options.screen_index = 3;
    } else {
        options.screen_index = 7 + u16::from(options.menu_option_index & 4 != 0);
        options.menu_option_index = 0;
    }
}

pub fn replace_screen<H: Host + ?Sized>(game: &mut H, options: &mut OptionsMenu) {
    use super::options_objects::spawn;
    if options.menu_option_index != 0 {
        if options.menu_option_index & 4 != 0 {
            let japanese = crate::session_words::current(game).japanese_text_flag != 0;
            copy_screen(0x1dffe + usize::from(japanese) * 0x800, !japanese, game);
            highlight_special(0, game);
            highlight_special(1, game);
            options.menu_option_index = 4;
            spawn(4, definition::SPECIAL_SETTING_MODE_BORDER, game);
        } else {
            let source = 0x1cffe
                + usize::from(crate::session_words::current(game).japanese_text_flag != 0) * 0x800;
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
    let index = crate::menus::state::load::<OptionsMenu, _>(game).screen_index;
    let screen = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12][usize::from(index)];
    // Starting a game saves, which reads RAM; it stores its own options words.
    if screen == 4 {
        finish_start(game);
    }
    crate::menus::state::with(game, |game, options: &mut OptionsMenu| {
        match screen {
            1 => initialize(game, options, upload),
            2 => fade_in::<false, H>(game, options),
            3 => main_menu(game, options),
            4 => {}
            6 => fade_in::<true, H>(game, options),
            7 => controller(game, options),
            8 => special(game, options),
            9 => scroll::<true, H>(game, options),
            10 => scroll::<false, H>(game, options),
            _ => fade_out(game, options, screen, enable),
        }
        super::options_objects::advance_and_draw(options, game);
        if options.screen_index.wrapping_sub(2) & 0x8000 == 0 {
            super::menu_tilemap::queue_options(game);
        }
    });
    0
}
