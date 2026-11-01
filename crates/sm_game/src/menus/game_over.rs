//! Game-over choices and transitions over the shared menu presentation.
use super::prelude::*;
use crate::actors::layouts::EnemyBase;
use crate::constants::bank_81::{
    ADDR_K_MENU_TILEMAP_FIND_METROID_LARVA, ADDR_K_MENU_TILEMAP_GAME_OVER,
    ADDR_K_MENU_TILEMAP_NO_GO_TO_TITLE, ADDR_K_MENU_TILEMAP_TRY_AGAIN,
    ADDR_K_MENU_TILEMAP_YES_RETURN_TO_GAME,
};
use crate::enemy_projectiles::table::with_file_select;
use crate::menu_state::MenuState;
use crate::menu_state::SaveSelection;
use crate::menus::menu_tilemap;
use crate::menus::save_slots;
use crate::ram::RawRam as _;
use crate::session_slot;

/// Requires a live, exclusively accessed graphics backend and native ROM allocation
/// covering the referenced menu text, baby animation and save-map records.
pub fn advance<H: crate::menus::host::Host + ?Sized>(game: &mut H) -> CoroutineRet {
    // The original state table, indexed and bounds-checked as before.
    let states: [fn(&mut MenuState, &mut H); 8] = [
        prepare::<H>,
        initialize::<H>,
        wait_for_music::<H>,
        fade_in::<H>,
        choose::<H>,
        fade_to_map::<H>,
        load_map::<H>,
        |_, game| fade_to_title(game),
    ];
    crate::menus::state::with(game, |game, menu: &mut MenuState| {
        states[menu.menu_index as usize](menu, game)
    });
    super::pending_wait(game)
}

fn prepare<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    if !super::resuming(game) {
        game.fade(false);
        if game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf != 0 {
            return;
        }
    }
    if super::blank_and_wait(true, game) {
        return;
    }
    game.queue_sound(2, 1_u8, 6);
    crate::hdma_objects::set_hdma_enabled(false, game);
    crate::hdma_objects::clear_hdma(game);
    crate::word_transfers!(game, ascending, 256; pause_map(384) <- le(PALETTE_BUFFER, 0));
    crate::word_transfers!(game, ascending, 27; pause_map(640) <- native(0x51));
    menu_tilemap::prepare_graphics(game);
    display_scroll! { game, 0; Bg1H, Bg2H, Bg3H, Bg1V, Bg2V, Bg3V }
    menu_tilemap::load_palette(game);
    menu.menu_index = menu.menu_index.wrapping_add(1);
}

fn initialize<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    if !super::resuming(game) {
        draw_menu(game);
    }
    if super::blank_and_wait(false, game) {
        return;
    }
    menu.menu_index = menu.menu_index.wrapping_add(1);
    crate::screen_fade::store(game, 0, 0);
    crate::menus::state::with(game, |_, selection: &mut SaveSelection| {
        selection.map_area_index = 0;
    });
    place_missile(FIRST_CHOICE_ROW, game);
}

/// Index 1 up to the clear-forced-blank wait: the menu, the baby and the gradient.
fn draw_menu<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    for (at, value) in [(0x69, 17), (0x74, 32), (0x75, 64), (0x76, 0x80)] {
        game.ram().write_byte(at, value);
    }
    crate::music_requests::queue_delayed_8(0, game);
    crate::music_requests::queue_delayed_8(0xff03, game);
    with_file_select(game, |menu| {
        *menu.missile_animation_timer() = 1;
        *menu.missile_animation_frame() = 0;
    });
    crate::word_transfers!(game, ascending, 1024; native(menu_tilemap::MENU_TILEMAP) <- value(menu_tilemap::BLANK_TILE));
    menu_tilemap::upload(
        menu_tilemap::MENU_TILEMAP_BYTES,
        menu_tilemap::MENU_TILEMAP_SOURCE,
        0,
        true,
        game,
    );
    menu_tilemap::set_palette(0, game);
    for (destination, text) in [
        (0x156, ADDR_K_MENU_TILEMAP_GAME_OVER),
        (0x38a, ADDR_K_MENU_TILEMAP_FIND_METROID_LARVA),
        (0x414, ADDR_K_MENU_TILEMAP_TRY_AGAIN),
        (0x4ce, ADDR_K_MENU_TILEMAP_YES_RETURN_TO_GAME),
        (0x5ce, ADDR_K_MENU_TILEMAP_NO_GO_TO_TITLE),
    ] {
        menu_tilemap::load(destination, text as u16, game);
    }
    start_baby(0, 0, game);
    animate_baby(game, |game| {
        game.rtl_unreachable();
    });
    let color = u16::from(game.ram().read_byte(crate::ram::bytes::GAMEPLAY_CGWSEL)) << 8;
    game.ram()
        .copy_in(crate::ram::mirror::CGWSEL, &color.to_ne_bytes());
    crate::hdma_objects::set_hdma_enabled(false, game);
    crate::hdma_objects::clear_hdma(game);
    crate::hdma_objects::set_hdma_enabled(true, game);
    for (register, script) in [(0x32, 0x927d), (0x31, 0x928d)] {
        game.spawn_hdma(0x81, &HdmaObjectDefinition::new(0, register, script));
    }
}

fn wait_for_music<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    if crate::music_requests::has_queued_music(game) == 0 {
        menu.menu_index = menu.menu_index.wrapping_add(1);
        crate::music_requests::queue_delayed_8(4, game);
    }
}

fn fade_in<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    fade(true, 15, menu, game);
}
fn fade_to_map<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    fade(false, 0, menu, game);
}
fn fade<H: crate::menus::host::Host + ?Sized>(
    fade_in: bool,
    brightness: u8,
    menu: &mut MenuState,
    game: &mut H,
) {
    animate_baby(game, |game| {
        game.rtl_unreachable();
    });
    crate::sprite_output::menu_selection(game);
    game.fade(fade_in);
    if game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf == brightness {
        menu.menu_index = menu.menu_index.wrapping_add(1);
    }
}

fn choose<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    animate_baby(game, |game| {
        game.rtl_unreachable();
    });
    crate::sprite_output::menu_selection(game);
    if game.observed_inputs().pressed & (K_BUTTON_SELECT | K_BUTTON_UP | K_BUTTON_DOWN) as u16 != 0
    {
        game.queue_sound(0, 0x37_u8, 6);
        crate::menus::state::with(game, |_, selection: &mut SaveSelection| {
            selection.map_area_index ^= 1;
        });
    } else if game.observed_inputs().pressed & K_BUTTON_A as u16 != 0 {
        session_slot!(game, EnemyBase[0].instruction_timer = 180);
        let selection = crate::menus::state::load::<SaveSelection, _>(game);
        if selection.map_area_index != 0 {
            menu.menu_index = 7;
        } else {
            if selection.loading_game_state == K_GAME_STATE_31_SET_UP_NEW_GAME as u16 {
                word!(game.GAME_STATE = selection.loading_game_state);
            } else {
                menu.menu_index = menu.menu_index.wrapping_add(1);
            }
            let load_result = save_slots::load(selection.selected_slot, game);
            game.observe_selected_continue(load_result);
        }
        return;
    }
    let row = if crate::menus::state::load::<SaveSelection, _>(game).map_area_index != 0 {
        SECOND_CHOICE_ROW
    } else {
        FIRST_CHOICE_ROW
    };
    place_missile(row, game);
}

/// The selection missile's x beside the game over menu's two choices, and their rows.
const MISSILE_X: u16 = 40;
const FIRST_CHOICE_ROW: u16 = 160;
const SECOND_CHOICE_ROW: u16 = 192;

fn place_missile<H: Host + ?Sized>(y: u16, game: &mut H) {
    with_file_select(game, |menu| {
        *menu.missile_x() = MISSILE_X;
        *menu.missile_y() = y;
    });
}

fn load_map<H: crate::menus::host::Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    crate::hdma_objects::set_hdma_enabled(false, game);
    crate::hdma_objects::clear_hdma(game);
    word!(game.GAME_STATE = K_GAME_STATE_5_FILE_SELECT_MAP as u16);
    menu.menu_index = 0;
}

fn fade_to_title<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    crate::sprite_output::menu_selection(game);
    animate_baby(game, |game| {
        game.rtl_unreachable();
    });
    game.fade(false);
    if game.ram().read_byte(crate::ram::mirror::INIDISP) & 0xf == 0 {
        game.request_reset();
    }
}

/// The menu reuses the first enemy record; it remains the only animation state.
fn animate_baby<H: crate::menus::host::Host + ?Sized>(game: &mut H, invalid: impl FnOnce(&mut H)) {
    let mut timer = session_slot!(game, EnemyBase[0].instruction_timer);
    if timer == 0 {
        timer = start_baby(0xbc27, 10, game);
    }
    loop {
        let cursor = session_slot!(game, EnemyBase[0].current_instruction);
        session_slot!(game, EnemyBase[0].instruction_timer = timer.wrapping_sub(1));
        if timer != 1 {
            draw_baby(cursor, game);
            return;
        }
        let next = baby_record(cursor, game).word(6);
        if next != 0xffff {
            let ordinary = (next as i16).wrapping_add(1) >= 0;
            let (duration, bytes) = if ordinary {
                (next, 6)
            } else {
                let sound = match next {
                    0xbc0c => 0x23,
                    0xbc15 => 0x26,
                    0xbc1e => 0x27,
                    _ => {
                        invalid(game);
                        return;
                    }
                };
                game.queue_sound(2, (sound) as u8, 6);
                let at = session_slot!(game, EnemyBase[0].current_instruction);
                (baby_record(at, game).word(8), 8)
            };
            session_slot!(game, EnemyBase[0].instruction_timer = duration);
            session_slot!(game, EnemyBase[0].current_instruction, |at| at
                .wrapping_add(bytes));
            if duration != 0xffff {
                let at = if ordinary {
                    cursor.wrapping_add(6)
                } else {
                    session_slot!(game, EnemyBase[0].current_instruction)
                };
                draw_baby(at, game);
                return;
            }
        }
        timer = start_baby(0xbc27, 10, game);
    }
}

fn start_baby<H: crate::menus::host::Host + ?Sized>(cursor: u16, timer: u16, game: &mut H) -> u16 {
    session_slot!(game, EnemyBase[0]; current_instruction = cursor, instruction_timer = timer);
    timer
}

fn baby_record<H: crate::menus::host::Host + ?Sized>(
    cursor: u16,
    game: &mut H,
) -> sm_data::cartridge::RomRecord {
    let address = 0x820000 | u32::from(cursor);
    if cursor & 0x8000 == 0 {
        game.native_rom_diagnostic(address);
    }
    sm_data::cartridge::RomRecord {
        image: game.native_image(),
        base: sm_data::cartridge::lorom_offset(address) as usize,
    }
}

fn draw_baby<H: crate::menus::host::Host + ?Sized>(cursor: u16, game: &mut H) {
    let palette = baby_record(cursor, game).word(4);
    for index in 0..16 {
        let color = baby_record(palette.wrapping_add(2 * index), game).word(0);
        word!(game.PALETTE_BUFFER[192 + usize::from(index)] = color);
    }
    let sprite = baby_record(cursor, game).word(2);
    crate::sprite_output::menu(sprite, 0x7c, 0x50, 2048, game);
    crate::sprite_output::menu(0x64, 0x7c, 0x50, 2560, game);
}
