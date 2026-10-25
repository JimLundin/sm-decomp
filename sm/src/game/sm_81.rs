use crate::game::music::{HasQueuedMusic, QueueMusic_Delayed8};
pub use crate::game::prelude::*;
use crate::game::projectile_presentation_ram as sprite_output;
use crate::game::save_slots;
use crate::game::visual_runtime::{clear_hdma, set_hdma_enabled, spawn_hdma};
use crate::game::{menu_tilemap, menu_window};

pub use crate::game::constants::bank_82::*;

use crate::game::sm_80::{
    HandleFadeIn, HandleFadeOut, LoadFromLoadStation, LoadMirrorOfExploredMapTiles, QueueSfx1_Max6,
    QueueSfx3_Max6, ScreenOff, ScreenOn,
};
use crate::game::sm_82::{
    AdvanceGradualColorChangeOfPalette, DisplayMapElevatorDestinations, DrawFileSelectMapIcons,
    DrawMapScrollArrowAndCheckToScroll, DrawRoomSelectMap, DrawRoomSelectMapAreaLabel,
    HandleGameOverBabyMetroid, MapScrolling, SetupMapScrollingForFileSelectMap,
};

pub use crate::types::Consts_81;
crate::game::constants::typed_constants! { Consts_81;
    addr_kAreaSelectBackgroundTilemaps = 48922, addr_kAreaSelectForegroundTilemap = 46874,
    addr_kMenuTilemap_DataCleared = 46810, addr_kMenuTilemap_ClearSamusA = 46746,
    addr_kMenuTilemap_ClearWhichData = 46682, addr_kMenuTilemap_CopyCompleted = 46650,
    addr_kMenuTilemap_No = 46634, addr_kMenuTilemap_Yes = 46618,
    addr_kMenuTilemap_IsThisOk = 46594, addr_kMenuTilemap_CopySamusToSamus = 46536,
    addr_kMenuTilemap_CopySamusToWhere = 46486, addr_kMenuTilemap_CopyWhichData = 46452,
    addr_kMenuTilemap_DataClearMode = 46388, addr_kMenuTilemap_DataCopyMode = 46328,
    addr_kMenuTilemap_Exit = 46318, addr_kMenuTilemap_DataClear = 46296,
    addr_kMenuTilemap_DataCopy = 46276,
    addr_kMenuTilemap_TIME = 46240,
    addr_kMenuTilemap_SamusC = 46198,
    addr_kMenuTilemap_SamusB = 46166, addr_kMenuTilemap_SamusA = 46134,
    addr_kMenuTilemap_SamusData = 46090, addr_kDownMapScrollArrowData = 44880,
    addr_kUpMapScrollArrowData = 44870, addr_kRightMapScrollArrowData = 44860,
    addr_kLeftMapScrollArrowData = 44850, addr_kMenuTilemap_NoGoToTitle = 37792,
    addr_kMenuTilemap_YesReturnToGame = 37708, addr_kMenuTilemap_TryAgain = 37684,
    addr_kMenuTilemap_FindMetroidLarva = 37636, addr_kMenuTilemap_GameOver = 37596,
}

crate::game::constants::typed_constants! { i32;
    true_0 = 1,
}
const RomPtr_82: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x82>;
#[inline]
unsafe fn get_RoomDefHeader(a: u16, game: &mut Game) -> *mut RoomDefHeader {
    unsafe { RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomDefHeader }
}
static kFileSelectMap_AreaIndexes: [u16; 6] = [0, 3, 5, 1, 4, 2];
pub fn SoftReset(game: &mut Game) {
    {
        word!(game.game_state = 0xffff);
    }
}
pub unsafe fn GameOverMenu_0_FadeOutConfigGfx(game: &mut Game) {
    unsafe {
        HandleFadeOut(game);
        if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
            ScreenOff(game);
            QueueSfx3_Max6(1, game);
            set_hdma_enabled(false, game);
            clear_hdma(game);
            word_transfers!(game, ascending, 256; pause_map(384) <- le(palette_buffer, 0));
            word_transfers!(game, ascending, 27; pause_map(640) <- native(0x51));
            menu_tilemap::prepare_graphics(game);
            word! { game, 0;
                reg_BG1HOFS, reg_BG2HOFS, reg_BG3HOFS, reg_BG1VOFS, reg_BG2VOFS, reg_BG3VOFS,
            }
            menu_tilemap::load_palette(game);
            word!(game.menu_index += 1);
        }
    }
}
static kGameOverMenuFuncs: [Func_V; 8] = callback_table! { nonnull Func_V;
    GameOverMenu_0_FadeOutConfigGfx, GameOverMenu_1_Init, GameOverMenu_2_PlayMusic,
    GameOverMenu_3_FadeIn, GameOverMenu_4_Main, GameOverMenu_5_FadeOutToGameMap,
    GameOverMenu_6_LoadGameMapView, GameOverMenu_7_FadeOutToSoftReset,
};
pub unsafe fn GameOverMenu(game: &mut Game) {
    unsafe {
        kGameOverMenuFuncs[word!(game.menu_index) as usize](game);
    }
}
pub unsafe fn GameOverMenu_3_FadeIn(game: &mut Game) {
    unsafe {
        HandleGameOverBabyMetroid(game);
        sprite_output::menu_selection(game);
        HandleFadeIn(game);
        if (*ram_ptr!(game; 0x51) & 0xf) == 15 {
            word!(game.menu_index += 1);
        }
    }
}
pub unsafe fn GameOverMenu_5_FadeOutToGameMap(game: &mut Game) {
    unsafe {
        HandleGameOverBabyMetroid(game);
        sprite_output::menu_selection(game);
        HandleFadeOut(game);
        if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
            word!(game.menu_index += 1);
        }
    }
}
pub unsafe fn GameOverMenu_7_FadeOutToSoftReset(game: &mut Game) {
    unsafe {
        sprite_output::menu_selection(game);
        HandleGameOverBabyMetroid(game);
        HandleFadeOut(game);
        if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
            SoftReset(game);
        }
    }
}
pub unsafe fn GameOverMenu_6_LoadGameMapView(game: &mut Game) {
    unsafe {
        set_hdma_enabled(false, game);
        clear_hdma(game);
        word! {game; game_state = kGameState_5_FileSelectMap as u16; menu_index = 0; }
    }
}
pub unsafe fn GameOverMenu_4_Main(game: &mut Game) {
    unsafe {
        HandleGameOverBabyMetroid(game);
        sprite_output::menu_selection(game);
        if (word!(game.joypad1_newkeys) as i32 & kButton_Select as i32) != 0
            || (word!(game.joypad1_newkeys) as i32 & kButton_Up as i32) != 0
            || (word!(game.joypad1_newkeys) as i32 & kButton_Down as i32) != 0
        {
            QueueSfx1_Max6(0x37, game);
            word!(game.file_select_map_area_index ^= (1u16));
        } else if (word!(game.joypad1_newkeys) as i32 & kButton_A as i32) != 0 {
            (*(ram_ptr!(game, EnemyData; 0xf78))).instruction_timer = 180;
            if word!(game.file_select_map_area_index) != 0 {
                word!(game.menu_index = 7);
            } else {
                if word!(game.loading_game_state) as i32 == kGameState_31_SetUpNewGame as i32 {
                    word!(game.game_state = word!(game.loading_game_state));
                } else {
                    word!(game.menu_index += 1);
                }
                let _load_result = save_slots::load(word!(game.selected_save_slot), game);
                #[cfg(sm_verify_dll)]
                crate::flags_save_observer::selected_continue(_load_result);
            }
            return;
        }
        let mut v0: u16 = 160;
        if word!(game.file_select_map_area_index) != 0 {
            v0 = 192;
        }
        word! {game; eproj_id[5] = 40; eproj_id[10] = v0; }
    }
}
pub unsafe fn GameOverMenu_1_Init(game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x69) = 17;
        *ram_ptr!(game; 0x74, 0) = 32;
        *ram_ptr!(game; 0x74, 1) = 64;
        *ram_ptr!(game; 0x74, 2) = 0x80;
        QueueMusic_Delayed8(0, game);
        QueueMusic_Delayed8(0xff03, game);
        word! {game; cinematic_var5 = 1; eproj_id[0] = 0; }
        word_transfers!(game, ascending, 1024; native(0x3600) <- value(15));
        menu_tilemap::upload(2048, 0x7e3600, 0, true, game);
        (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
        menu_tilemap::load(0x156, addr_kMenuTilemap_GameOver as u16, game);
        menu_tilemap::load(0x38a, addr_kMenuTilemap_FindMetroidLarva as u16, game);
        menu_tilemap::load(0x414, addr_kMenuTilemap_TryAgain as u16, game);
        menu_tilemap::load(0x4ce, addr_kMenuTilemap_YesReturnToGame as u16, game);
        menu_tilemap::load(0x5ce, addr_kMenuTilemap_NoGoToTitle as u16, game);
        enemy_instruction! {[(*(ram_ptr!(game, EnemyData; 0xf78)))]; instruction = 0, timer = 0}
        HandleGameOverBabyMetroid(game);
        *(ram_ptr!(game, u16; 0x6f)) = ((*ram_ptr!(game; 0x70) as i32) << 8) as u16;
        set_hdma_enabled(false, game);
        clear_hdma(game);
        set_hdma_enabled(true, game);
        static unk_819254: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0, 0x32, 0x927d as VoidP);
        static unk_81925C: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0, 0x31, 0x928d as VoidP);
        spawn_hdma(0x81, &unk_819254, game);
        spawn_hdma(0x81, &unk_81925C, game);
        ScreenOn(game);
        word!(game.menu_index += 1);
        word! { game, 0;
            screen_fade_delay, screen_fade_counter, file_select_map_area_index,
        }
        word! {game; eproj_id[5] = 40; eproj_id[10] = 160; }
    }
}
pub unsafe fn GameOverMenu_2_PlayMusic(game: &mut Game) {
    {
        if HasQueuedMusic(game) == 0 {
            word!(game.menu_index += 1);
            QueueMusic_Delayed8(4, game);
        }
    }
}
static kFileSelectMapFuncs: [Func_V; 23] = callback_table! { nonnull Func_V;
    FileSelectMap_0, FileSelectMap_1, FileSelectMap_2_LoadAreaSelectForegroundTilemap,
    FileSelectMap_3_LoadAreaSelectBackgroundTilemap,
    FileSelectMap_4_SetupExpandingSquareTransition, FileSelectMap_5_ExpandingSquare,
    FileSelectMap_6_AreaSelectMap, FileSelectMap_7_PrepExpandSquareTransToRoomMap,
    FileSelectMap_8_ExpandSquareTransToRoomSelectMap, FileSelectMap_9_InitRoomSelectMap,
    FileSelectMap_10_RoomSelectMap, FileSelectMap_11, FileSelectMap_11, FileSelectMap_13,
    FileSelectMap_14, FileSelectMap_15_ClearTileMap, FileSelectMap_16_LoadPalettes,
    FileSelectMap_2_LoadAreaSelectForegroundTilemap, FileSelectMap_18,
    FileSelectMap_4_SetupExpandingSquareTransition, FileSelectMap_20_SetupExpandingSquare,
    FileSelectMap_21_MoveExpandingSquare, FileSelectMap_22,
};
pub unsafe fn FileSelectMap(game: &mut Game) {
    unsafe {
        kFileSelectMapFuncs[word!(game.menu_index) as usize](game);
    }
}
pub unsafe fn FileSelectMap_22(game: &mut Game) {
    unsafe {
        DrawAreaSelectMapLabels(game);
        HandleFadeOut(game);
        if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
            word! {game; game_state = kGameState_2_GameOptionsMenu as u16; menu_index = 0; }
        }
    }
}
pub unsafe fn FileSelectMap_0(game: &mut Game) {
    menu_tilemap::clear(game);
    menu_tilemap::upload(2048, 0x7e3600, 1, true, game);
    word! {game; palette_change_num = 0; }
    menu_tilemap::load_palette(game);
    word_transfers!(game, ascending, 32; le(target_palettes, 0) <- le(palette_buffer, 0));
    word! { game, 0; target_palettes[14], target_palettes[30] }
    word!(game.menu_index += 1);
}
pub unsafe fn FileSelectMap_1(game: &mut Game) {
    unsafe {
        if AdvanceGradualColorChangeOfPalette(0, 0x40, game) != 0 {
            word! { game, 0; reg_BG1VOFS, reg_BG1HOFS, reg_BG2VOFS, reg_BG2HOFS }
            let mut v0: u16 = 0;
            while word!(game.area_index) != kFileSelectMap_AreaIndexes[(v0 >> 1) as usize] {
                v0 = (v0).wrapping_add(2);
                if ((v0) as i16).wrapping_sub(12) < 0 {
                    continue;
                }
                v0 = 0;
                break;
            }
            word!(game.file_select_map_area_index = v0 >> 1);
            for area in 0..6 {
                menu_tilemap::area_palette(area, area == word!(game.area_index), game);
            }
            word!(game.menu_index += 1);
            word!(game.g_word_7E0787 = 0);
            *ram_ptr!(game; 0x69) = 2;
        }
    }
}
pub unsafe fn FileSelectMap_2_LoadAreaSelectForegroundTilemap(game: &mut Game) {
    unsafe {
        menu_tilemap::upload(
            2048,
            0x810000 | u32::from(addr_kAreaSelectForegroundTilemap),
            0,
            true,
            game,
        );
        *(ram_ptr!(game, u16; 0x51)) = ((*ram_ptr!(game; 0x52) as i32) << 8 | 0xf) as u16;
        word!(game.menu_index += 1);
    }
}
pub fn FileSelectMap_18(game: &mut Game) {
    word!(game.menu_index += 1);
    LoadAreaSelectBackgroundTilemap(word!(game.area_index), game);
}
pub use self::FileSelectMap_18 as FileSelectMap_3_LoadAreaSelectBackgroundTilemap;
pub fn LoadAreaSelectBackgroundTilemap(j: u16, game: &mut Game) {
    menu_tilemap::upload(
        2048,
        0x810000
            | u32::from((2048 * i32::from(j) + addr_kAreaSelectBackgroundTilemaps as i32) as u16),
        2,
        false,
        game,
    );
}
pub unsafe fn FileSelectMap_4_SetupExpandingSquareTransition(game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x6b) = 4;
        crate::rtl::register_writes! { game;
            mirror(TS as u16, 4, 0x6d, false); byte(TSW as u16, 4);
            mirror(CGWSEL as u16, 2, 0x6f, true); mirror(CGADSUB as u16, 0x25, 0x72, true);
        }
        *ram_ptr!(game; 0x74, 2) = 0x80;
        *ram_ptr!(game; 0x74, 1) = 64;
        *ram_ptr!(game; 0x74, 0) = 32;
        menu_window::initialize(game);
        crate::rtl::write_register_and_mirror(HDMAEN as u16, 0xc, 0x85, true, game);
        ConfigureWindow1ForExpandingSquare(game);
        word!(game.hdma_objects_enable_flag = 0);
        byte!(@pointer ram_ptr!(game, u16; 0x727); 0) =
            ((word!(game.menu_index)) as u8).wrapping_add(1);
    }
}
pub unsafe fn ConfigureWindow1ForExpandingSquare(game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x69) = 19;
        crate::rtl::register_writes! { game;
            mirror(TM as u16, 0x13, 0x6c, false); byte(TMW as u16, 0x13);
            mirror(W12SEL as u16, 0x23, 0x60, true); mirror(W34SEL as u16, 3, 0x61, true);
            mirror(WOBJSEL as u16, 0x23, 0x62, true);
        }
    }
}
pub unsafe fn FileSelectMap_5_ExpandingSquare(game: &mut Game) {
    unsafe {
        DrawAreaSelectMapLabels(game);
        if menu_window::open_from_center(game) {
            byte!(@pointer ram_ptr!(game, u16; 0x727); 0) =
                ((word!(game.menu_index)) as u8).wrapping_add(1);
            crate::update_place! { reference fresh36 = *ram_ptr!(game; 0x69) => *fresh36 & !(2); }
            *ram_ptr!(game; 0x6c) = 0;
            *ram_ptr!(game; 0x6d) = 0;
            word! {game; reg_BG2VOFS = 24; pausemenu_palette_animation_timer = 1; }
            let mut v1: u16 = 0;
            word_transfers!(game, ascending, 800; xray(0) <- rom(0xb6e000, 0));
            v1 = 800;
            word_transfers!(game, ascending, 224; xray(800) <- value(10241));
            v1 = 1024;
            DrawRoomSelectMapAreaLabel(
                (&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).bg2_room_select_map_tilemap
                    as *mut u16)
                    .offset(170),
                game,
            );
            word_transfers!(game, descending, 160; xray(959) <- rom(0x81b14b, 160));
            menu_tilemap::upload(2048, 0x7e4000, 1, false, game);
        }
    }
}
pub unsafe fn FileSelectMap_6_AreaSelectMap(game: &mut Game) {
    unsafe {
        let mut v0: i16 = word!(game.joypad1_newkeys) as i16;
        'handle_action: {
            if word!(game.joypad1_newkeys) as i32 & (kButton_Up as i32 | kButton_Left as i32) != 0 {
                v0 = word!(game.enable_debug) as i16;
                if word!(game.enable_debug) != 0 {
                    let mut R28: u16 = DecAndWraparoundTo5(word!(game.file_select_map_area_index));
                    if CheckIfFileSelectMapAreaCanBeSelected(R28, game) != 0
                        || {
                            R28 = DecAndWraparoundTo5(R28);
                            CheckIfFileSelectMapAreaCanBeSelected(R28, game) != 0
                        }
                        || {
                            R28 = DecAndWraparoundTo5(R28);
                            CheckIfFileSelectMapAreaCanBeSelected(R28, game) != 0
                        }
                    {
                        QueueSfx1_Max6(0x37, game);
                        SwitchActiveFileSelectMapArea(R28, game);
                        DrawAreaSelectMapLabels(game);
                        return;
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
                        R28_0 = WraparoundFrom6to0(R28_0);
                        if CheckIfFileSelectMapAreaCanBeSelected(R28_0, game) != 0 {
                            break;
                        }
                        n -= 1;
                        if n == 0 {
                            break 'handle_action;
                        }
                    }
                    SwitchActiveFileSelectMapArea(R28_0, game);
                    QueueSfx1_Max6(0x37, game);
                    DrawAreaSelectMapLabels(game);
                    return;
                }
            }
            if v0 < 0 {
                word!(game.menu_index = 22);
                DrawAreaSelectMapLabels(game);
                return;
            }
            if v0 as i32 & (kButton_Start as i32 | kButton_A as i32) != 0 {
                QueueSfx1_Max6(0x38, game);
                SelectFileSelectMapArea(game);
                return;
            }
        }
        DrawAreaSelectMapLabels(game);
    }
}
pub fn DecAndWraparoundTo5(a: u16) -> u16 {
    let result: u16 = (a).wrapping_sub(1);
    if (result & 0x8000) != 0 {
        return 5;
    }
    result
}
pub fn WraparoundFrom6to0(a: u16) -> u16 {
    let result: u16 = (a).wrapping_add(1);
    if result as i32 - 6i32 & 0x8000 == 0 {
        return 0;
    }
    result
}
pub unsafe fn SelectFileSelectMapArea(game: &mut Game) {
    unsafe {
        word!(game.menu_index += 1);
        if word!(game.enable_debug) == 0 {
            word!(game.area_index = word!(game.sram_area_index));
            word!(game.load_station_index = word!(game.sram_save_station_index));
            DrawAreaSelectMapLabels(game);
            return;
        }
        word!(
            game.area_index =
                kFileSelectMap_AreaIndexes[word!(game.file_select_map_area_index) as usize]
        );
        let mut q: u16 =
            *(ram_ptr!(game, u16; 0xd8f8, (2 * word!(game.area_index) as i32) as isize));
        let mut v0: u16 = 0;
        let r0 = RomPtr_82(
            *(RomPtr_82(
                (addr_kMapIconDataPointers as i32 + 64 + 2 * word!(game.area_index) as i32) as u16,
                game,
            ) as *mut u16),
            game,
        ) as *const u16;
        let mut r20: u16 = 16;
        's_43: loop {
            let mut v2: i8 = ((q) as i8) & (1i8);
            q = q >> 1;
            if v2 != 0 {
                let mut v3: i16 = *r0.offset((2 * v0 as i32) as isize) as i16;
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
                let mut v4: i16 = *r0.offset((2 * v0 as i32) as isize) as i16;
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
        DrawAreaSelectMapLabels(game);
    }
}
pub unsafe fn CheckIfFileSelectMapAreaCanBeSelected(a: u16, game: &mut Game) -> u16 {
    unsafe {
        let v1: i32 = 2 * kFileSelectMap_AreaIndexes[a as usize] as i32;
        if *(ram_ptr!(game, u16; 0xd8f8, v1 as isize)) != 0 {
            return true_0 as u16;
        }
        let t: u16 = (*((&raw mut (*(rom_ptr!(game, MapIconDataPointers; 0x82c7cb)).offset(4))
            .crateria as *mut u8)
            .offset(v1 as isize) as *mut u16))
            .wrapping_add(64);
        (t != 0xffff) as u16
    }
}
pub fn SwitchActiveFileSelectMapArea(R28: u16, game: &mut Game) {
    menu_tilemap::area_palette(
        kFileSelectMap_AreaIndexes[word!(game.file_select_map_area_index) as usize],
        false,
        game,
    );
    word!(game.file_select_map_area_index = R28);
    menu_tilemap::area_palette(kFileSelectMap_AreaIndexes[R28 as usize], true, game);
    LoadAreaSelectBackgroundTilemap(
        kFileSelectMap_AreaIndexes[word!(game.file_select_map_area_index) as usize],
        game,
    );
}
static kAreaSelectMapLabelPositions: [u16; 12] = [
    0x5b, 0x32, 0x2a, 0x7f, 0x5e, 0xb5, 0xce, 0x50, 0xce, 0x9f, 0x87, 0x8b,
];
pub unsafe fn DrawAreaSelectMapLabels(game: &mut Game) {
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
            let v1: u16 = (2u16).wrapping_mul(kFileSelectMap_AreaIndexes[i as usize]);
            let mut r36: u16 = *(ram_ptr!(game, u16; 0xd8f8, v1 as isize));
            let mut v2 = RomPtr_82(
                *((&raw mut (*(rom_ptr!(game, MapIconDataPointers; 0x82c7cb)).offset(4)).crateria
                    as *mut u8)
                    .offset(v1 as i32 as isize) as *mut VoidP),
                game,
            ) as *const u16;
            let mut R30: i32 = 16;
            while *v2 != 0xffff {
                let v4: i32 = r36 as i32 & 1;
                r36 = r36 >> 1;
                if !(v4 != 0 && *v2 != 0xfffe) {
                    v2 = v2.offset(2);
                    R30 -= 1;
                    if R30 != 0 {
                        continue;
                    }
                    if !(word!(game.enable_debug) != 0 && *v2 != 0xffff) {
                        break;
                    }
                }
                let j: i32 = 4 * kFileSelectMap_AreaIndexes[i as usize] as i32 >> 1;
                sprite_output::menu(
                    (crate::rtl::rom_table_word!(game, 0x82c749, 0) as i32
                        + kFileSelectMap_AreaIndexes[i as usize] as i32
                        + 1) as u16,
                    kAreaSelectMapLabelPositions[j as usize],
                    kAreaSelectMapLabelPositions[(j + 1) as usize],
                    r3,
                    game,
                );
                break;
            }
        }
    }
}
static kRoomSelectMapExpandingSquareTimers: [u16; 6] = [0x33, 0x35, 0x2d, 0x33, 0x33, 0x22];
pub unsafe fn FileSelectMap_7_PrepExpandSquareTransToRoomMap(game: &mut Game) {
    unsafe {
        DrawAreaSelectMapLabels(game);
        menu_window::initialize(game);
        *ram_ptr!(game; 0x69) = 19;
        *ram_ptr!(game; 0x6c) = 19;
        crate::rtl::register_writes! { game;
            mirror(W12SEL as u16, 0x32, 0x60, true); mirror(W34SEL as u16, 2, 0x61, true);
            mirror(CGADSUB as u16, 5, 0x72, true); mirror(WOBJSEL as u16, 0x22, 0x62, true);
        }
        *ram_ptr!(game; 0x5d) = 48;
        (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(3)).field_0 = 0;
        (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(3)).field_0 = 0;
        DrawRoomSelectMapAreaLabel(
            (&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).bg2_room_select_map_tilemap as *mut u16)
                .offset(170),
            game,
        );
        menu_tilemap::upload(512, 0x7e4000, 1, false, game);
        word! {game; expand_sq_timer = kRoomSelectMapExpandingSquareTimers[word!(game.area_index) as usize]; }
        let v2: i32 = ((4u16).wrapping_mul(word!(game.area_index))) as i32 >> 1;
        word!(game.expand_sq_left_pos = kAreaSelectMapLabelPositions[v2 as usize]);
        word!(game.expand_sq_right_pos = word!(game.expand_sq_left_pos));
        word!(game.expand_sq_top_pos = kAreaSelectMapLabelPositions[(v2 + 1) as usize]);
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
pub unsafe fn FileSelectMap_8_ExpandSquareTransToRoomSelectMap(game: &mut Game) {
    unsafe {
        menu_window::advance(menu_window::Motion::Expand, game);
        if (word!(game.expand_sq_timer -= 1, reread) & 0x8000) != 0 {
            word!(game.menu_index += 1);
            *ram_ptr!(game; 0x69) = 2;
            *ram_ptr!(game; 0x6c) = 0;
            *ram_ptr!(game; 0x6d) = 0;
            *ram_ptr!(game; 0x6b) = 0;
        }
        DrawAreaSelectMapLabels(game);
    }
}
pub unsafe fn FileSelectMap_9_InitRoomSelectMap(game: &mut Game) {
    unsafe {
        LoadMirrorOfExploredMapTiles(game);
        DrawRoomSelectMap(game);
        LoadFromLoadStation(game);
        set_hdma_enabled(false, game);
        clear_hdma(game);
        let RoomDefHeader = get_RoomDefHeader(word!(game.room_ptr), game);
        byte!(@pointer ram_ptr!(game, u16; 0x79f); 0) = (*RoomDefHeader).area_index_;
        byte!(@pointer ram_ptr!(game, u16; 0x7a1); 0) = (*RoomDefHeader).x_coordinate_on_map;
        byte!(@pointer ram_ptr!(game, u16; 0x7a3); 0) = (*RoomDefHeader).y_coordinate_on_map;
        SetupMapScrollingForFileSelectMap(game);
        word!(game.map_min_y_scroll += 24);
        word!(game.reg_BG2VOFS = 24);
        crate::update_place! { unaligned_word fresh31 = &raw mut (*(ram_ptr!(game, u16; 0x69))) => u16::from_le(fresh31.read_unaligned()) & !(4); }
        word!(game.menu_index += 1);
        word! { game, 0;
            map_scrolling_direction, map_scrolling_speed_index,
            samus_position_indicator_animation_frame, samus_position_indicator_animation_timer,
            samus_position_indicator_animation_loop_counter,
        }
    }
}
pub unsafe fn FileSelectMap_10_RoomSelectMap(game: &mut Game) {
    unsafe {
        DrawFileSelectMapIcons(game);
        HandleFileSelectMapScrollArrows(game);
        MapScrolling(game);
        DisplayMapElevatorDestinations(game);
        if word!(game.enable_debug) != 0
            && (word!(game.joypad2_new_keys) as i32 & kButton_Select as i32) != 0
        {
            QueueSfx1_Max6(0x38, game);
            let mut r24: u16 =
                *(ram_ptr!(game, u16; 0xd8f8, (2 * word!(game.area_index) as i32) as isize));
            let mut v1: i16 = word!(game.load_station_index) as i16;
            do_until! { v1 < 0 ;
                r24 = r24 >> 1;
                v1 -= 1;
            }
            let r0 = RomPtr_82(
                *(RomPtr_82(
                    (addr_kMapIconDataPointers as i32 + 64 + 2 * word!(game.area_index) as i32)
                        as u16,
                    game,
                ) as *mut u16),
                game,
            ) as *const u16;
            let mut v3: i16 = (4i16).wrapping_mul((word!(game.load_station_index)) as i16);
            let r18: u16 = *r0.offset((v3 >> 1) as isize);
            let r20: u16 = *r0.offset(((v3 as i32 >> 1) + 1) as isize);
            let mut check_unlocked = word!(game.load_station_index) as i32 - 16i32 & 0x8000 != 0;
            loop {
                let inspect_terminal = if check_unlocked {
                    word!(game.load_station_index += 1, reread) as i32 - 16i32 & 0x8000 == 0
                } else {
                    word!(game.load_station_index += 1);
                    true
                };
                if inspect_terminal {
                    let mut v5: i16 =
                        *r0.offset((2 * word!(game.load_station_index) as i32) as isize) as i16;
                    if v5 == -1i16 {
                        word!(game.load_station_index = 0);
                        r24 = *(ram_ptr!(game, u16; 0xd8f8, (2 * word!(game.area_index) as i32) as isize));
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
                    || *r0.offset((2 * word!(game.load_station_index) as i32) as isize) >= 0xfffe
                {
                    check_unlocked = true;
                } else {
                    break;
                }
            }
            let v6: u16 = (4u16).wrapping_mul(word!(game.load_station_index));
            let w7: u16 = *r0.offset((2 * word!(game.load_station_index) as i32) as isize);
            if w7 as i32 - word!(game.reg_BG1HOFS) as i32 & 0x8000 != 0
                || w7 as i32 - 256i32 - word!(game.reg_BG1HOFS) as i32 & 0x8000 == 0
            {
                let mut v8: i16 = (word!(game.reg_BG1HOFS) as i32
                    + *r0.offset((v6 >> 1) as isize) as i32
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
            let v10: u16 = *r0.offset((v9 >> 1) as isize);
            if v10 as i32 - word!(game.reg_BG1VOFS) as i32 & 0x8000 != 0
                || v10 as i32 - 161i32 - word!(game.reg_BG1VOFS) as i32 & 0x8000 == 0
            {
                let mut v11: u16 = (word!(game.reg_BG1VOFS) as i32
                    + *r0.offset((v9 >> 1) as isize) as i32
                    - r20 as i32) as u16;
                if ((v11) as i16).wrapping_sub((word!(game.map_min_y_scroll)) as i16) >= 0 {
                    v11 = word!(game.map_min_y_scroll);
                }
                word!(game.reg_BG1VOFS = v11);
            }
        } else if (word!(game.joypad1_newkeys) & 0x8000) != 0 {
            word!(game.menu_index += 5);
            let mut v0: u16 = 0;
            while word!(game.area_index) != kFileSelectMap_AreaIndexes[(v0 >> 1) as usize] {
                v0 = (v0).wrapping_add(2);
                if ((v0) as i16).wrapping_sub(12) >= 0 {
                    word!(game.file_select_map_area_index = 0);
                    return;
                }
            }
            word!(game.file_select_map_area_index = v0 >> 1);
            QueueSfx1_Max6(0x3c, game);
        } else if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32)
            != 0
        {
            word!(game.menu_index += 1);
            QueueSfx1_Max6(0x38, game);
        }
    }
}
pub unsafe fn HandleFileSelectMapScrollArrows(game: &mut Game) {
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
                == (*(rom_ptr!(game, MapScrollArrowData; 0x81af50))).map_scroll_dir
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
pub unsafe fn FileSelectMap_11(game: &mut Game) {
    unsafe {
        DrawFileSelectMapIcons(game);
        DisplayMapElevatorDestinations(game);
        word!(game.menu_index += 1);
    }
}
pub unsafe fn FileSelectMap_13(game: &mut Game) {
    unsafe {
        DrawFileSelectMapIcons(game);
        DisplayMapElevatorDestinations(game);
        HandleFadeOut(game);
        if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
            word!(game.menu_index += 1);
            (*(ram_ptr!(game, EnemyData; 0xf78))).x_pos = 32;
        }
    }
}
pub unsafe fn FileSelectMap_14(game: &mut Game) {
    unsafe {
        let fresh22 = &raw mut (*(ram_ptr!(game, EnemyData; 0xf78))).x_pos;
        fresh22.write_unaligned(fresh22.read_unaligned().wrapping_sub(1));
        if fresh22.read_unaligned() == 0 {
            ScreenOff(game);
            word!(game.game_state += 1);
            word!(game.menu_index = 0);
        }
    }
}
pub unsafe fn FileSelectMap_15_ClearTileMap(game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x69) = 18;
        word_transfers!(game, descending, 1024; native(0x37fe) <- value(15));
        menu_tilemap::upload(2048, 0x7e3000, 0, true, game);
        word!(game.menu_index += 1);
    }
}
pub fn FileSelectMap_16_LoadPalettes(game: &mut Game) {
    menu_tilemap::load_palette(game);
    for area in 0..6 {
        menu_tilemap::area_palette(area, false, game);
    }
    menu_tilemap::area_palette(word!(game.area_index), true, game);
    LoadAreaSelectBackgroundTilemap(word!(game.area_index), game);
    word!(game.menu_index += 1);
}
pub unsafe fn FileSelectMap_20_SetupExpandingSquare(game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x85) = 0;
        QueueSfx1_Max6(0x3c, game);
        word! { game;
            expand_sq_timer =
                (kRoomSelectMapExpandingSquareTimers[word!(game.area_index) as usize]).wrapping_sub(12);
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
        *ram_ptr!(game; 0x5d) = 48;
        *ram_ptr!(game; 0x62) = 34;
        (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(3)).field_0 = 0;
        (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(3)).field_0 = 0;
        word! { game, 0; reg_BG1VOFS, reg_BG1HOFS }
    }
}
pub unsafe fn FileSelectMap_21_MoveExpandingSquare(game: &mut Game) {
    unsafe {
        menu_window::advance(menu_window::Motion::Contract, game);
        DrawAreaSelectMapLabels(game);
        if (word!(game.expand_sq_timer -= 1, reread) & 0x8000) != 0 {
            word!(game.menu_index -= 15);
            *ram_ptr!(game; 0x69) = 17;
            *ram_ptr!(game; 0x6c) = 0;
            *ram_ptr!(game; 0x6d) = 0;
        }
    }
}
pub unsafe fn NewSaveFile(game: &mut Game) {
    {
        word! { game, 99; samus_max_health, samus_health }
        word! { game, 0;
            samus_max_missiles, samus_missiles, samus_max_super_missiles, samus_super_missiles,
            samus_max_power_bombs, samus_power_bombs, hud_item_index, collected_beams, equipped_beams,
            collected_items, equipped_items, reserve_health_mode, samus_max_reserve_health,
            samus_reserve_health, samus_reserve_missiles,
        }
        word! {game; button_config_up = kButton_Up as u16; button_config_down = kButton_Down as u16;
        button_config_left = kButton_Left as u16; button_config_right = kButton_Right as u16;
        button_config_jump_a = kButton_A as u16; button_config_run_b = kButton_B as u16;
        button_config_shoot_x = kButton_X as u16; button_config_itemcancel_y = kButton_Y as u16;
        button_config_itemswitch = kButton_Select as u16;
        button_config_aim_up_R = kButton_R as u16; button_config_aim_down_L = kButton_L as u16; }
        word! { game, 0;
            game_time_frames, game_time_seconds, game_time_minutes, game_time_hours, japanese_text_flag,
            moonwalk_flag, hud_auto_cancel_flag,
        }
        word! { game, 1; debug_flag, UNUSED_word_7E09E8 }
        let mut v0: u16 = 0;
        word_transfers!(game, ascending, 4;
            native(0xd830) <- value(0),
            native(0xd870) <- value(0),
            native(0xd870) <- value(0),
            native(0xd8b0) <- value(0),
            le(UNUSED_word_7ED8F0, 0) <- value(0),
            native(0xd908) <- value(0),
            native(0xd8f8) <- value(0),
            native(0xd900) <- value(0),
        );
        v0 = 8;
        word_transfers!(game, ascending, 28;
            native(0xd838) <- value(0),
            native(0xd878) <- value(0),
            native(0xd878) <- value(0),
            native(0xd8b8) <- value(0),
        );
        v0 = 64;
        word_transfers!(game, ascending, 896; le(explored_map_tiles_saved, 0) <- value(0));
    }
}
