//! Area and room selection, with the original menu-state and RAM compatibility.
use crate::game::constants::{bank_81::*, bank_82::*};
use crate::game::music::QueueSfx1_Max6;
use crate::game::prelude::*;
use crate::game::projectile_presentation_ram as sprite_output;
use crate::game::room_entry::{self, ArrivalRequest, StartSite};
use crate::game::save_slots::load_area_map;
use crate::game::session_runtime::Runtime;
use crate::game::sm_82::{
    DisplayMapElevatorDestinations, DrawFileSelectMapIcons, DrawMapScrollArrowAndCheckToScroll,
    DrawRoomSelectMap, MapScrolling, SetupMapScrollingForFileSelectMap,
};
use crate::game::visual_runtime::{clear_hdma, set_hdma_enabled};
use crate::game::{menu_tilemap, menu_window};
const AREAS: [u16; 6] = [0, 3, 5, 1, 4, 2];

static STATES: [Func_V; 23] = callback_table! { nonnull Func_V;
    prepare, fade_palette, load_foreground,
    begin_background,
    begin_area_window, open_area_window,
    choose_area, begin_room_window,
    open_room_window, initialize_room,
    choose_room, show_room, show_room, fade_room,
    start_game, clear_room, restore_area,
    load_foreground, begin_background,
    begin_area_window, begin_return_window,
    close_room_window, fade_to_options,
};
/// Requires live exclusive hardware and native ROM covering referenced map records.
pub(crate) unsafe fn advance(game: &mut Game) {
    unsafe {
        STATES[word!(game.menu_index) as usize](game);
    }
}
unsafe fn fade_to_options(game: &mut Game) {
    unsafe {
        draw_area_labels(game);
        crate::game::display_refresh::fade(false, game);
        if (game.g_ram[0x51] & 0xf) == 0 {
            word! {game; game_state = kGameState_2_GameOptionsMenu as u16; menu_index = 0; }
        }
    }
}
unsafe fn prepare(game: &mut Game) {
    menu_tilemap::clear(game);
    menu_tilemap::upload(2048, 0x7e3600, 1, true, game);
    word! {game; palette_change_num = 0; }
    menu_tilemap::load_palette(game);
    word_transfers!(game, ascending, 32; le(target_palettes, 0) <- le(palette_buffer, 0));
    word! { game, 0; target_palettes[14], target_palettes[30] }
    word!(game.menu_index += 1);
}
unsafe fn fade_palette(game: &mut Game) {
    if crate::game::palette_effects::fading::file_select(game) != 0 {
        word! { game, 0; reg_BG1VOFS, reg_BG1HOFS, reg_BG2VOFS, reg_BG2HOFS }
        word!(game.file_select_map_area_index = area_selection(game).unwrap_or(0) as u16);
        for area in 0..6 {
            menu_tilemap::area_palette(area, area == word!(game.area_index), game);
        }
        word!(game.menu_index += 1);
        word!(game.g_word_7E0787 = 0);
        game.g_ram[0x69] = 2;
    }
}
fn load_foreground(game: &mut Game) {
    menu_tilemap::upload(
        2048,
        0x810000 | u32::from(addr_kAreaSelectForegroundTilemap),
        0,
        true,
        game,
    );
    set_native_word(0x51, ((game.g_ram[0x52] as i32) << 8 | 0xf) as u16, game);
    word!(game.menu_index += 1);
}
fn begin_background(game: &mut Game) {
    word!(game.menu_index += 1);
    load_background(word!(game.area_index), game);
}
fn load_background(j: u16, game: &mut Game) {
    menu_tilemap::upload(
        2048,
        0x810000
            | u32::from((2048 * i32::from(j) + addr_kAreaSelectBackgroundTilemaps as i32) as u16),
        2,
        false,
        game,
    );
}
unsafe fn begin_area_window(game: &mut Game) {
    unsafe {
        game.g_ram[0x6b] = 4;
        crate::rtl::register_writes! { game;
            mirror(TS as u16, 4, 0x6d, false); byte(TSW as u16, 4);
            mirror(CGWSEL as u16, 2, 0x6f, true); mirror(CGADSUB as u16, 0x25, 0x72, true);
        }
        game.g_ram[0x74 + 2] = 0x80;
        game.g_ram[0x74 + 1] = 64;
        game.g_ram[0x74 + 0] = 32;
        menu_window::initialize(game);
        crate::rtl::write_register_and_mirror(HDMAEN as u16, 0xc, 0x85, true, game);
        configure_area_window(game);
        word!(game.hdma_objects_enable_flag = 0);
        game.g_ram[0x727] = ((word!(game.menu_index)) as u8).wrapping_add(1);
    }
}
unsafe fn configure_area_window(game: &mut Game) {
    unsafe {
        game.g_ram[0x69] = 19;
        crate::rtl::register_writes! { game;
            mirror(TM as u16, 0x13, 0x6c, false); byte(TMW as u16, 0x13);
            mirror(W12SEL as u16, 0x23, 0x60, true); mirror(W34SEL as u16, 3, 0x61, true);
            mirror(WOBJSEL as u16, 0x23, 0x62, true);
        }
    }
}
unsafe fn open_area_window(game: &mut Game) {
    unsafe {
        draw_area_labels(game);
        if menu_window::open_from_center(game) {
            game.g_ram[0x727] = ((word!(game.menu_index)) as u8).wrapping_add(1);
            game.g_ram[0x69] &= !2;
            game.g_ram[0x6c] = 0;
            game.g_ram[0x6d] = 0;
            word! {game; reg_BG2VOFS = 24; pausemenu_palette_animation_timer = 1; }
            let mut v1: u16 = 0;
            word_transfers!(game, ascending, 800; xray(0) <- rom(0xb6e000, 0));
            v1 = 800;
            word_transfers!(game, ascending, 224; xray(800) <- value(10241));
            v1 = 1024;
            menu_tilemap::area_label(game);
            word_transfers!(game, descending, 160; xray(959) <- rom(0x81b14b, 160));
            menu_tilemap::upload(2048, 0x7e4000, 1, false, game);
        }
    }
}
unsafe fn choose_area(game: &mut Game) {
    unsafe {
        let mut v0: i16 = word!(game.joypad1_newkeys) as i16;
        'handle_action: {
            if word!(game.joypad1_newkeys) as i32 & (kButton_Up as i32 | kButton_Left as i32) != 0 {
                v0 = word!(game.enable_debug) as i16;
                if word!(game.enable_debug) != 0 {
                    let mut selection = word!(game.file_select_map_area_index);
                    for _ in 0..3 {
                        selection = previous_area(selection);
                        if area_available(selection, game) != 0 {
                            QueueSfx1_Max6(0x37, game);
                            switch_area(selection, game);
                            draw_area_labels(game);
                            return;
                        }
                    }
                    break 'handle_action;
                }
            } else if word!(game.joypad1_newkeys) as i32
                & (kButton_Select as i32 | kButton_Down as i32 | kButton_Right as i32)
                != 0
            {
                v0 = word!(game.enable_debug) as i16;
                if word!(game.enable_debug) != 0 {
                    let mut n: i32 = 6;
                    let mut R28_0: u16 = word!(game.file_select_map_area_index);
                    loop {
                        R28_0 = next_area(R28_0);
                        if area_available(R28_0, game) != 0 {
                            break;
                        }
                        n -= 1;
                        if n == 0 {
                            break 'handle_action;
                        }
                    }
                    switch_area(R28_0, game);
                    QueueSfx1_Max6(0x37, game);
                    draw_area_labels(game);
                    return;
                }
            }
            if v0 < 0 {
                word!(game.menu_index = 22);
                draw_area_labels(game);
                return;
            }
            if v0 as i32 & (kButton_Start as i32 | kButton_A as i32) != 0 {
                QueueSfx1_Max6(0x38, game);
                select_area(game);
                return;
            }
        }
        draw_area_labels(game);
    }
}
fn previous_area(a: u16) -> u16 {
    let result: u16 = (a).wrapping_sub(1);
    if (result & 0x8000) != 0 {
        return 5;
    }
    result
}
fn next_area(a: u16) -> u16 {
    let result: u16 = (a).wrapping_add(1);
    if result as i32 - 6i32 & 0x8000 == 0 {
        return 0;
    }
    result
}
unsafe fn select_area(game: &mut Game) {
    unsafe {
        word!(game.menu_index += 1);
        if word!(game.enable_debug) == 0 {
            word!(game.area_index = word!(game.sram_area_index));
            word!(game.load_station_index = word!(game.sram_save_station_index));
            draw_area_labels(game);
            return;
        }
        word!(game.area_index = AREAS[word!(game.file_select_map_area_index) as usize]);
        let mut q: u16 = native_word(0xd8f8 + 2 * usize::from(word!(game.area_index)), game);
        let mut v0: u16 = 0;
        let r0 = station_coordinates(word!(game.area_index), game);
        let mut r20: u16 = 16;
        's_43: loop {
            let mut v2: i8 = ((q) as i8) & (1i8);
            q = q >> 1;
            if v2 != 0 {
                let mut v3: i16 = r0.table_word((2 * v0 as i32) as isize) as i16;
                if v3 != -2i16 {
                    if v3 != -1i16 {
                        break;
                    }
                    v0 = -1i32 as u16;
                }
            }
            v0 = v0.wrapping_add(1);
            r20 = r20.wrapping_sub(1);
            if r20 != 0 {
                continue;
            }
            loop {
                let mut v4: i16 = r0.table_word((2 * v0 as i32) as isize) as i16;
                if v4 != -2i16 {
                    if v4 != -1i16 {
                        break 's_43;
                    }
                    v0 = -1i32 as u16;
                }
                v0 = v0.wrapping_add(1);
                r20 = r20.wrapping_sub(1);
                if r20 == 0 {
                    Unreachable(game);
                }
            }
        }
        word!(game.load_station_index = v0);
        draw_area_labels(game);
    }
}
fn area_available(a: u16, game: &mut Game) -> u16 {
    let v1: i32 = 2 * AREAS[a as usize] as i32;
    if native_word(0xd8f8 + v1 as usize, game) != 0 {
        return 1;
    }
    let t = game
        .capture_native_rom()
        .table_word(
            crate::rtl::lorom_offset(0x82c80b) as usize,
            (v1 / 2) as isize,
        )
        .wrapping_add(64);
    (t != 0xffff) as u16
}
fn switch_area(R28: u16, game: &mut Game) {
    menu_tilemap::area_palette(
        AREAS[word!(game.file_select_map_area_index) as usize],
        false,
        game,
    );
    word!(game.file_select_map_area_index = R28);
    menu_tilemap::area_palette(AREAS[R28 as usize], true, game);
    load_background(AREAS[word!(game.file_select_map_area_index) as usize], game);
}
static LABEL_POSITIONS: [u16; 12] = [
    0x5b, 0x32, 0x2a, 0x7f, 0x5e, 0xb5, 0xce, 0x50, 0xce, 0x9f, 0x87, 0x8b,
];
unsafe fn draw_area_labels(game: &mut Game) {
    unsafe {
        let mut r3: u16 = 0;
        sprite_output::menu(
            crate::rtl::rom_table_word!(game, 0x82c749, 0),
            0x80,
            0x10,
            r3,
            game,
        );
        for i in 0i32..6 {
            r3 = (if i == word!(game.file_select_map_area_index) as i32 {
                0
            } else {
                512
            }) as u16;
            let v1: u16 = (2u16).wrapping_mul(AREAS[i as usize]);
            let mut r36: u16 = native_word(0xd8f8 + v1 as usize, game);
            let mut v2 = station_coordinates(AREAS[i as usize], game);
            let mut R30: i32 = 16;
            while v2.word(0) != 0xffff {
                let v4: i32 = r36 as i32 & 1;
                r36 = r36 >> 1;
                if !(v4 != 0 && v2.word(0) != 0xfffe) {
                    v2.base += 4;
                    R30 -= 1;
                    if R30 != 0 {
                        continue;
                    }
                    if !(word!(game.enable_debug) != 0 && v2.word(0) != 0xffff) {
                        break;
                    }
                }
                let j: i32 = 4 * AREAS[i as usize] as i32 >> 1;
                sprite_output::menu(
                    (crate::rtl::rom_table_word!(game, 0x82c749, 0) as i32
                        + AREAS[i as usize] as i32
                        + 1) as u16,
                    LABEL_POSITIONS[j as usize],
                    LABEL_POSITIONS[(j + 1) as usize],
                    r3,
                    game,
                );
                break;
            }
        }
    }
}
static WINDOW_TIMERS: [u16; 6] = [0x33, 0x35, 0x2d, 0x33, 0x33, 0x22];
unsafe fn begin_room_window(game: &mut Game) {
    unsafe {
        draw_area_labels(game);
        menu_window::initialize(game);
        game.g_ram[0x69] = 19;
        game.g_ram[0x6c] = 19;
        crate::rtl::register_writes! { game;
            mirror(W12SEL as u16, 0x32, 0x60, true); mirror(W34SEL as u16, 2, 0x61, true);
            mirror(CGADSUB as u16, 5, 0x72, true); mirror(WOBJSEL as u16, 0x22, 0x62, true);
        }
        game.g_ram[0x5d] = 48;
        game.g_ram[0x9e09] = 0;
        game.g_ram[0x9e19] = 0;
        menu_tilemap::area_label(game);
        menu_tilemap::upload(512, 0x7e4000, 1, false, game);
        word! {game; expand_sq_timer = WINDOW_TIMERS[word!(game.area_index) as usize]; }
        let v2: i32 = ((4u16).wrapping_mul(word!(game.area_index))) as i32 >> 1;
        word!(game.expand_sq_left_pos = LABEL_POSITIONS[v2 as usize]);
        word!(game.expand_sq_right_pos = word!(game.expand_sq_left_pos));
        word!(game.expand_sq_top_pos = LABEL_POSITIONS[(v2 + 1) as usize]);
        word!(game.expand_sq_bottom_pos = word!(game.expand_sq_top_pos));
        word! { game, 0;
            expand_sq_left_subpos, expand_sq_right_subpos, expand_sq_top_subpos,
            expand_sq_bottom_subpos,
        }
        menu_window::load_velocities(word!(game.area_index), game);
        menu_window::project(game);
        crate::rtl::write_register_and_mirror(HDMAEN as u16, 0xc, 0x85, true, game);
        word!(game.menu_index += 1);
        QueueSfx1_Max6(0x3b, game);
    }
}
unsafe fn open_room_window(game: &mut Game) {
    unsafe {
        menu_window::advance(menu_window::Motion::Expand, game);
        if (word!(game.expand_sq_timer -= 1, reread) & 0x8000) != 0 {
            word!(game.menu_index += 1);
            game.g_ram[0x69] = 2;
            game.g_ram[0x6c] = 0;
            game.g_ram[0x6d] = 0;
            game.g_ram[0x6b] = 0;
        }
        draw_area_labels(game);
    }
}
unsafe fn initialize_room(game: &mut Game) {
    unsafe {
        load_area_map(game);
        DrawRoomSelectMap(game);
        room_entry::begin(
            &mut Runtime::new(game),
            ArrivalRequest::Start(StartSite::CurrentLoadStation),
        );
        set_hdma_enabled(false, game);
        clear_hdma(game);
        let room = menu_tilemap::native_record(0x8f0000 | u32::from(word!(game.room_ptr)), game);
        game.g_ram[0x79f] = room.byte(1);
        game.g_ram[0x7a1] = room.byte(2);
        game.g_ram[0x7a3] = room.byte(3);
        SetupMapScrollingForFileSelectMap(game);
        word!(game.map_min_y_scroll += 24);
        word!(game.reg_BG2VOFS = 24);
        crate::ram_map::checked_word!(game, 0x69; = crate::ram_map::checked_word!(game, 0x69;) & !4);
        word!(game.menu_index += 1);
        word! { game, 0;
            map_scrolling_direction, map_scrolling_speed_index,
            samus_position_indicator_animation_frame, samus_position_indicator_animation_timer,
            samus_position_indicator_animation_loop_counter,
        }
    }
}
unsafe fn choose_room(game: &mut Game) {
    unsafe {
        DrawFileSelectMapIcons(game);
        scroll_arrows(game);
        MapScrolling(game);
        DisplayMapElevatorDestinations(game);
        if word!(game.enable_debug) != 0
            && (word!(game.joypad2_new_keys) as i32 & kButton_Select as i32) != 0
        {
            QueueSfx1_Max6(0x38, game);
            let mut r24: u16 = native_word(0xd8f8 + 2 * usize::from(word!(game.area_index)), game);
            let mut v1: i16 = word!(game.load_station_index) as i16;
            do_until! { v1 < 0 ;
                r24 = r24 >> 1;
                v1 -= 1;
            }
            let r0 = station_coordinates(word!(game.area_index), game);
            let mut v3: i16 = (4i16).wrapping_mul((word!(game.load_station_index)) as i16);
            let r18: u16 = r0.table_word((v3 >> 1) as isize);
            let r20: u16 = r0
                .image
                .table_word(r0.base, ((v3 as i32 >> 1) + 1) as isize);
            let mut check_unlocked = word!(game.load_station_index) as i32 - 16i32 & 0x8000 != 0;
            loop {
                let inspect_terminal = if check_unlocked {
                    word!(game.load_station_index += 1, reread) as i32 - 16i32 & 0x8000 == 0
                } else {
                    word!(game.load_station_index += 1);
                    true
                };
                if inspect_terminal {
                    let mut v5: i16 = r0.image.table_word(
                        r0.base,
                        (2 * word!(game.load_station_index) as i32) as isize,
                    ) as i16;
                    if v5 == -1i16 {
                        word!(game.load_station_index = 0);
                        r24 = native_word(0xd8f8 + 2 * usize::from(word!(game.area_index)), game);
                    } else if v5 != -2i16 {
                        break;
                    } else {
                        check_unlocked = false;
                        continue;
                    }
                }
                let mut v4: i8 = ((r24) as i8) & (1i8);
                r24 = r24 >> 1;
                if v4 == 0
                    || r0.image.table_word(
                        r0.base,
                        (2 * word!(game.load_station_index) as i32) as isize,
                    ) >= 0xfffe
                {
                    check_unlocked = true;
                } else {
                    break;
                }
            }
            let v6: u16 = (4u16).wrapping_mul(word!(game.load_station_index));
            let w7: u16 = r0.image.table_word(
                r0.base,
                (2 * word!(game.load_station_index) as i32) as isize,
            );
            if w7 as i32 - word!(game.reg_BG1HOFS) as i32 & 0x8000 != 0
                || w7 as i32 - 256i32 - word!(game.reg_BG1HOFS) as i32 & 0x8000 == 0
            {
                let mut v8: i16 = (word!(game.reg_BG1HOFS) as i32
                    + r0.table_word((v6 >> 1) as isize) as i32
                    - r18 as i32) as i16;
                if v8 >= 0 {
                    if (v8).wrapping_sub((word!(game.map_min_x_scroll)) as i16) >= 0 {
                        v8 = word!(game.map_min_x_scroll) as i16;
                    }
                } else {
                    v8 = 0;
                }
                word!(game.reg_BG1HOFS = v8 as u16);
            }
            let v9: u16 = (v6).wrapping_add(2);
            let v10: u16 = r0.table_word((v9 >> 1) as isize);
            if v10 as i32 - word!(game.reg_BG1VOFS) as i32 & 0x8000 != 0
                || v10 as i32 - 161i32 - word!(game.reg_BG1VOFS) as i32 & 0x8000 == 0
            {
                let mut v11: u16 = (word!(game.reg_BG1VOFS) as i32
                    + r0.table_word((v9 >> 1) as isize) as i32
                    - r20 as i32) as u16;
                if ((v11) as i16).wrapping_sub((word!(game.map_min_y_scroll)) as i16) >= 0 {
                    v11 = word!(game.map_min_y_scroll);
                }
                word!(game.reg_BG1VOFS = v11);
            }
        } else if (word!(game.joypad1_newkeys) & 0x8000) != 0 {
            word!(game.menu_index += 5);
            let Some(selection) = area_selection(game) else {
                word!(game.file_select_map_area_index = 0);
                return;
            };
            word!(game.file_select_map_area_index = selection as u16);
            QueueSfx1_Max6(0x3c, game);
        } else if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32)
            != 0
        {
            word!(game.menu_index += 1);
            QueueSfx1_Max6(0x38, game);
        }
    }
}
unsafe fn scroll_arrows(game: &mut Game) {
    unsafe {
        if word!(game.map_min_x_scroll) as i32 - 24i32 - word!(game.reg_BG1HOFS) as i32 & 0x8000
            != 0
        {
            DrawMapScrollArrowAndCheckToScroll(0x81, addr_kLeftMapScrollArrowData as u16, game);
        }
        if word!(game.map_max_x_scroll) as i32 - 232i32 - word!(game.reg_BG1HOFS) as i32 & 0x8000
            == 0
        {
            DrawMapScrollArrowAndCheckToScroll(0x81, addr_kRightMapScrollArrowData as u16, game);
        }
        if word!(game.map_min_y_scroll) as i32 - 64i32 - word!(game.reg_BG1VOFS) as i32 & 0x8000
            != 0
        {
            DrawMapScrollArrowAndCheckToScroll(0x81, addr_kUpMapScrollArrowData as u16, game);
        }
        if word!(game.map_max_y_scroll) as i32 - 145i32 - word!(game.reg_BG1VOFS) as i32 & 0x8000
            != 0
        {
            if word!(game.map_scrolling_direction)
                == game
                    .capture_native_rom()
                    .table_word(crate::rtl::lorom_offset(0x81af50) as usize, 4)
            {
                word! { game, 0;
                    map_scrolling_gear_switch_timer, map_scrolling_direction, map_scrolling_speed_index,
                }
            }
        } else {
            DrawMapScrollArrowAndCheckToScroll(0x81, addr_kDownMapScrollArrowData as u16, game);
        };
    }
}
unsafe fn show_room(game: &mut Game) {
    unsafe {
        DrawFileSelectMapIcons(game);
        DisplayMapElevatorDestinations(game);
        word!(game.menu_index += 1);
    }
}
unsafe fn fade_room(game: &mut Game) {
    unsafe {
        DrawFileSelectMapIcons(game);
        DisplayMapElevatorDestinations(game);
        crate::game::display_refresh::fade(false, game);
        if (game.g_ram[0x51] & 0xf) == 0 {
            word!(game.menu_index += 1);
            set_native_word(0xf78 + core::mem::offset_of!(EnemyData, x_pos), 32, game);
        }
    }
}
unsafe fn start_game(game: &mut Game) {
    let timer = 0xf78 + core::mem::offset_of!(EnemyData, x_pos);
    set_native_word(timer, native_word(timer, game).wrapping_sub(1), game);
    if native_word(timer, game) == 0 {
        crate::game::display_refresh::set_blank(true, game);
        word!(game.game_state += 1);
        word!(game.menu_index = 0);
    }
}
fn clear_room(game: &mut Game) {
    game.g_ram[0x69] = 18;
    word_transfers!(game, descending, 1024; native(0x37fe) <- value(15));
    menu_tilemap::upload(2048, 0x7e3000, 0, true, game);
    word!(game.menu_index += 1);
}
fn restore_area(game: &mut Game) {
    menu_tilemap::load_palette(game);
    for area in 0..6 {
        menu_tilemap::area_palette(area, false, game);
    }
    menu_tilemap::area_palette(word!(game.area_index), true, game);
    load_background(word!(game.area_index), game);
    word!(game.menu_index += 1);
}
unsafe fn begin_return_window(game: &mut Game) {
    unsafe {
        game.g_ram[0x85] = 0;
        QueueSfx1_Max6(0x3c, game);
        word! { game;
            expand_sq_timer =
                (WINDOW_TIMERS[word!(game.area_index) as usize]).wrapping_sub(12);
        }
        menu_window::load_velocities(word!(game.area_index), game);
        word! { game;
            expand_sq_left_pos = 8;
            expand_sq_right_pos = 248;
            expand_sq_top_pos = 8;
            expand_sq_bottom_pos = 216;
        }
        word! { game, 0;
            expand_sq_left_subpos, expand_sq_right_subpos, expand_sq_top_subpos,
            expand_sq_bottom_subpos,
        }
        menu_window::project(game);
        word!(game.menu_index += 1);
        crate::rtl::register_writes! { game;
            mirror(HDMAEN as u16, 0xc, 0x85, true); mirror(W12SEL as u16, 0x32, 0x60, true);
            mirror(W34SEL as u16, 2, 0x61, true);
        }
        game.g_ram[0x5d] = 48;
        game.g_ram[0x62] = 34;
        game.g_ram[0x9e09] = 0;
        game.g_ram[0x9e19] = 0;
        word! { game, 0; reg_BG1VOFS, reg_BG1HOFS }
    }
}
unsafe fn close_room_window(game: &mut Game) {
    unsafe {
        menu_window::advance(menu_window::Motion::Contract, game);
        draw_area_labels(game);
        if (word!(game.expand_sq_timer -= 1, reread) & 0x8000) != 0 {
            word!(game.menu_index -= 15);
            game.g_ram[0x69] = 17;
            game.g_ram[0x6c] = 0;
            game.g_ram[0x6d] = 0;
        }
    }
}

unsafe fn station_coordinates(area: u16, game: &mut Game) -> crate::rom::RomRecord {
    unsafe {
        let pointer = menu_tilemap::native_record(
            0x820000
                | u32::from((addr_kMapIconDataPointers as i32 + 64 + 2 * i32::from(area)) as u16),
            game,
        )
        .word(0);
        menu_tilemap::native_record(0x820000 | u32::from(pointer), game)
    }
}

fn native_word(offset: usize, game: &Game) -> u16 {
    u16::from_ne_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
}

fn set_native_word(offset: usize, value: u16, game: &mut Game) {
    game.g_ram[offset..offset + 2].copy_from_slice(&value.to_ne_bytes());
}

fn area_selection(game: &Game) -> Option<usize> {
    AREAS
        .iter()
        .position(|&area| area == word!(game.area_index))
}
