pub use crate::game::prelude::*;
use crate::game::sprite_drawing::{Clip, SpriteMap};

pub use crate::game::constants::bank_82::*;

use crate::game::sm_80::{
    GraphicsAsset, HandleFadeIn, HandleFadeOut, HasQueuedMusic, LoadFromLoadStation,
    LoadMirrorOfExploredMapTiles, QueueMusic_Delayed8, QueueSfx1_Max6, QueueSfx3_Max6, ScreenOff,
    ScreenOn,
};
use crate::game::sm_82::{
    AdvanceGradualColorChangeOfPalette, DisplayMapElevatorDestinations,
    DrawBorderAroundDataClearMode, DrawBorderAroundDataCopyMode, DrawBorderAroundSamusData,
    DrawFileCopyArrow, DrawFileSelectMapIcons, DrawMapScrollArrowAndCheckToScroll,
    DrawMenuSelectionMissile, DrawRoomSelectMap, DrawRoomSelectMapAreaLabel,
    HandleGameOverBabyMetroid, MapScrolling, SetupMapScrollingForFileSelectMap,
};
use crate::game::sm_88::{
    DisableHdmaObjects, EnableHdmaObjects, SpawnHdmaObject, WaitUntilEndOfVblankAndClearHdma,
};

crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct ExpandingSquareVels {
        pub left_subvel, left_vel, right_subvel, right_vel, top_subvel, top_vel, bottom_subvel,
            bottom_vel: u16;
    }
}
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
    addr_kMenuTilemap_DataCopy = 46276, addr_kMenuTilemap_NoData = 46252,
    addr_word_81B4A8 = 46248, addr_kMenuTilemap_TIME = 46240,
    addr_kMenuTilemap_Energy = 46230, addr_kMenuTilemap_SamusC = 46198,
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
const RomPtr_81: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x81>;
const RomPtr_82: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x82>;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
#[inline]
unsafe fn get_RoomDefHeader(a: u16, game: &mut Game) -> *mut RoomDefHeader {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomDefHeader
}
static kFileSelectMap_AreaIndexes: [u16; 6] = [0, 3, 5, 1, 4, 2];
pub unsafe fn SoftReset(game: &mut Game) {
    word!(game.game_state = 0xffff);
}
pub unsafe fn SaveToSram(a: u16, game: &mut Game) {
    let mut r20: u16 = 0;
    let r18: u16 = (2 * (a as i32 & 3)) as u16;
    word_transfers!(game, descending, 48; le(player_data_saved, 47) <- native(0xa00));
    let mut v3: u16 = (word!(game.area_index)).wrapping_mul(256);
    let mut v4: u16 = 0;
    do_until! { (((v4) as i16).wrapping_sub(256) >= 0);
        word!(
            game.explored_map_tiles_saved[(v3 >> 1)] = *(ram_ptr!(game, u16; 0x7f7, v4 as isize))
        );
        v4 = (v4).wrapping_add(2);
        v3 = (v3).wrapping_add(2);
    }
    PackMapToSave(game);
    word!(game.sram_save_station_index = word!(game.load_station_index));
    word!(game.sram_area_index = word!(game.area_index));
    let mut v5: u16 = crate::rtl::rom_table_word!(game, 0x81812b, (r18 >> 1) as isize);
    for offset in (0xd7c0..0xde1c).step_by(2) {
        let v7 = u16::from_ne_bytes(game.g_ram[offset..offset + 2].try_into().unwrap());
        game.g_sram.set_word((2 * (v5 as i32 >> 1)) as usize, v7);
        r20 = (r20).wrapping_add(v7);
        v5 = (v5).wrapping_add(2);
    }
    let v8: u16 = r18;
    let v9: u16 = r20;
    let v10: i32 = r18 as i32 >> 1;
    game.g_sram.set_word((2 * v10) as usize, r20);
    game.g_sram.set_word((2 * v10 + 0x1ff0) as usize, v9);
    let mut v11: u16 = !(v9 as i32) as u16;
    game.g_sram.set_word((2 * v10 + 8) as usize, v11);
    game.g_sram.set_word((2 * v10 + 0x1ff8) as usize, v11);
    RtlWriteSram(game);
}
pub unsafe fn LoadFromSram(a: u16, game: &mut Game) -> u8 {
    let mut r20: u16 = 0;
    let r18: u16 = (2 * (a as i32 & 3)) as u16;
    let mut v1: u16 = crate::rtl::rom_table_word!(game, 0x81812b, (a as i32 & 3) as isize);
    for offset in (0xd7c0..0xde1c).step_by(2) {
        let v3: u16 = game.g_sram.word((2 * (v1 >> 1)) as usize);
        game.g_ram[offset..offset + 2].copy_from_slice(&v3.to_ne_bytes());
        r20 = (r20).wrapping_add(v3);
        v1 = (v1).wrapping_add(2);
    }
    let v4: i32 = r18 as i32 >> 1;
    if r20 == game.g_sram.word((2 * v4) as usize)
        && r20 ^ 0xffff == game.g_sram.word((0x8 + 2 * v4) as usize)
        || r20 == game.g_sram.word((0x1ff0 + 2 * v4) as usize)
            && r20 ^ 0xffff == game.g_sram.word((0x1ff8 + 2 * v4) as usize)
    {
        word_transfers!(game, descending, 48; native(0xa00) <- le(player_data_saved, 47));
        UnpackMapFromSave(game);
        word!(game.load_station_index = word!(game.sram_save_station_index));
        word!(game.area_index = word!(game.sram_area_index));
        0
    } else {
        r20 = 0;
        let mut v7: u16 = crate::rtl::rom_table_word!(game, 0x81812b, (r18 >> 1) as isize);
        let mut v9: u16 = 0;
        for _ in (0xd7c0..0xde1c).step_by(2) {
            game.g_sram.set_word((2 * (v7 >> 1)) as usize, v9);
            v9 = (v9).wrapping_add(r20);
            r20 = v9;
            v7 = (v7).wrapping_add(2);
        }
        word! { game, 0; load_station_index, area_index }
        1
    }
}
pub unsafe fn UnpackMapFromSave(game: &mut Game) {
    word_transfers!(game, descending, 897; le(explored_map_tiles_saved, 896) <- value(0));
    for i_0 in 0i32..6 {
        let mut n: i32 = byte!(@pointer rom_ptr!(game; 0x818131); i_0 as isize) as i32;
        let mut r0 = RomPtr_81(
            crate::rtl::rom_table_word!(game, 0x8182d6, i_0 as isize),
            game,
        );
        let mut v3: i32 = crate::rtl::rom_table_word!(game, 0x818138, i_0 as isize) as i32;
        let unpacked = ram_ptr!(game, u16; 0xcd52) as *mut u8;
        do_until! { (n == 0);
            crate::update_place! { previous fresh2 = r0 => r0.offset(1); }
            let v4: u16 = (i_0 * 256 + *fresh2 as i32) as u16;
            crate::update_place! { previous fresh3 = v3 => v3 + 1; }
            *unpacked.offset(v4 as isize) = *ram_ptr!(game; 0xd91c, fresh3 as isize);
            n -= 1;
        }
    }
}
pub unsafe fn PackMapToSave(game: &mut Game) {
    for i in 0i32..6 {
        let mut n: i32 = byte!(@pointer rom_ptr!(game; 0x818131); i as isize) as i32;
        let mut r0 = RomPtr_81(
            crate::rtl::rom_table_word!(game, 0x8182d6, i as isize),
            game,
        );
        let mut v1: i32 = crate::rtl::rom_table_word!(game, 0x818138, i as isize) as i32;
        let unpacked = ram_ptr!(game, u16; 0xcd52) as *mut u8;
        do_until! { (n == 0);
            crate::update_place! { previous fresh95 = r0 => r0.offset(1); }
            let v3: i32 = i * 256 + *fresh95 as i32;
            crate::update_place! { previous fresh96 = v1 => v1 + 1; }
            *ram_ptr!(game; 0xd91c, fresh96 as isize) = *unpacked.offset(v3 as isize);
            n -= 1;
        }
    }
}
// The native warning is an explicit remaining unsafe boundary. Sprite decoding
// and OAM writes use the owned image and checked bytes in sprite_drawing.
unsafe fn sprite_map(bank: u8, address: u16, game: &mut Game) -> SpriteMap {
    let address = u32::from(bank) << 16 | u32::from(address);
    if address & 0x8000 == 0 {
        printf(c"RomPtr - Invalid access 0x%x!\n".as_ptr(), address);
        game.g_fail = true;
    }
    SpriteMap {
        image: game
            .loaded_rom
            .as_ref()
            .expect("ROM must be loaded")
            .clone(),
        offset: crate::rtl::lorom_offset(address) as usize,
    }
}
pub unsafe fn DrawSpritemap(db: u8, j: u16, x: u16, y: u16, palette: u16, game: &mut Game) {
    sprite_map(db, j, game).draw((x, y), |tile| palette | tile & 0xf1ff, Clip::Bounded, game);
}
pub unsafe fn DrawSpritemapOffScreen(j: u16, x: u16, y: u16, palette: u16, game: &mut Game) {
    sprite_map(0x8c, j, game).draw(
        (x, y),
        |tile| palette | tile & 0xf1ff,
        Clip::BoundedOutside,
        game,
    );
}
pub unsafe fn DrawMenuSpritemap(a: u16, k: u16, j: u16, chr_r3: u16, game: &mut Game) {
    let pp = sprite_map(
        0x82,
        crate::rtl::rom_table_word!(game, 0x82c569, a as isize),
        game,
    );
    pp.draw((k, j), |tile| chr_r3 | tile & 0xf1ff, Clip::None, game);
}
pub unsafe fn DrawSamusSpritemap(a: u16, x_pos: u16, y_pos: u16, game: &mut Game) {
    if crate::rtl::rom_table_word!(game, 0x92808d, a as isize) == 0 {
        return;
    }
    let pp = sprite_map(
        0x92,
        crate::rtl::rom_table_word!(game, 0x92808d, a as isize),
        game,
    );
    pp.draw((x_pos, y_pos), |tile| tile, Clip::None, game);
}
pub unsafe fn DrawBeamGrappleSpritemap(a: u16, x_r20: u16, y_r18: u16, game: &mut Game) {
    DrawGrappleOrProjectileSpritemap(
        sprite_map(
            0x93,
            crate::rtl::rom_table_word!(game, 0x93a1a1, a as isize),
            game,
        ),
        x_r20,
        y_r18,
        game,
    );
}
pub unsafe fn DrawProjectileSpritemap(k: u16, x_r20: u16, y_r18: u16, game: &mut Game) {
    DrawGrappleOrProjectileSpritemap(
        sprite_map(
            0x93,
            u16::from_ne_bytes([
                game.g_ram[0xcb8 + usize::from(k >> 1) * 2],
                game.g_ram[0xcb9 + usize::from(k >> 1) * 2],
            ]),
            game,
        ),
        x_r20,
        y_r18,
        game,
    );
}
fn DrawGrappleOrProjectileSpritemap(pp: SpriteMap, x_r20: u16, y_r18: u16, game: &mut Game) {
    pp.draw((x_r20, y_r18), |tile| tile, Clip::None, game);
}
pub unsafe fn DrawSpritemapWithBaseTile(
    db: u8,
    j: u16,
    r20_x: u16,
    r18_y: u16,
    r3: u16,
    r0: u16,
    game: &mut Game,
) {
    if j == 0 {
        return;
    }
    let pp = sprite_map(db, j, game);
    pp.draw(
        (r20_x, r18_y),
        |tile| (r3 as i32 | r0 as i32 + tile as i32) as u16,
        Clip::None,
        game,
    );
}
pub unsafe fn DrawSpritemapWithBaseTile2(
    db: u8,
    j: u16,
    r20_x: u16,
    r18_y: u16,
    r3: u16,
    r0: u16,
    game: &mut Game,
) {
    let pp = sprite_map(db, j, game);
    pp.draw(
        (r20_x, r18_y),
        |tile| (r3 as i32 | r0 as i32 + tile as i32) as u16,
        Clip::Inside,
        game,
    );
}
pub unsafe fn DrawSpritemapWithBaseTileOffscreen(
    db: u8,
    j: u16,
    r20_x: u16,
    r18_y: u16,
    r3: u16,
    r0: u16,
    game: &mut Game,
) {
    let pp = sprite_map(db, j, game);
    pp.draw(
        (r20_x, r18_y),
        |tile| (r3 as i32 | r0 as i32 + tile as i32) as u16,
        Clip::Outside,
        game,
    );
}
crate::forward_calls! { pub unsafe fn (
    db: u8, j: u16, x: u16, y: u16, tile: u16, flags: u16, game: &mut Game
);
    DrawEprojSpritemapWithBaseTile =>
        DrawSpritemapWithBaseTile2(db, j, x, y, flags, tile, game);
    DrawEprojSpritemapWithBaseTileOffscreen =>
        DrawSpritemapWithBaseTileOffscreen(db, j, x, y, flags, tile, game);
}
pub unsafe fn GameOverMenu_0_FadeOutConfigGfx(game: &mut Game) {
    HandleFadeOut(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
        ScreenOff(game);
        QueueSfx3_Max6(1, game);
        DisableHdmaObjects(game);
        WaitUntilEndOfVblankAndClearHdma(game);
        word_transfers!(game, ascending, 256; pause_map(384) <- le(palette_buffer, 0));
        word_transfers!(game, ascending, 27; pause_map(640) <- native(0x51));
        MapVramForMenu(game);
        LoadInitialMenuTiles(game);
        word! { game, 0;
            reg_BG1HOFS, reg_BG2HOFS, reg_BG3HOFS, reg_BG1VOFS, reg_BG2VOFS, reg_BG3VOFS,
        }
        LoadMenuPalettes(game);
        word!(game.menu_index += 1);
    }
}
pub unsafe fn MapVramForMenu(game: &mut Game) {
    *ram_ptr!(game; 0x6b) = 0;
    *ram_ptr!(game; 0x52) = 3;
    *ram_ptr!(game; 0x58) = 81;
    *ram_ptr!(game; 0x59) = 88;
    *ram_ptr!(game; 0x5a) = 92;
    *ram_ptr!(game; 0x5d) = 0;
    *ram_ptr!(game; 0x5e) = 4;
    *ram_ptr!(game; 0x69) = 19;
}
pub unsafe fn LoadInitialMenuTiles(game: &mut Game) {
    GraphicsAsset::VramWords(0x8e8000, 0, 0x5600).load(game);
    GraphicsAsset::VramWords(0xb68000, 0x3000, 0x2000).load(game);
    GraphicsAsset::VramWords(0xb6c000, 0x6000, 0x2000).load(game);
    GraphicsAsset::VramWords(0x8ed600, 0x4000, 0x600).load(game);
}
pub unsafe fn LoadMenuPalettes(game: &mut Game) {
    word_transfers!(game, (4), 128;
        le(palette_buffer, 0) <- rom(0x8ee400, 0),
        le(palette_buffer, 1) <- rom(0x8ee400, 1),
    );
}
static kGameOverMenuFuncs: [Func_V; 8] = callback_table! { nonnull Func_V;
    GameOverMenu_0_FadeOutConfigGfx, GameOverMenu_1_Init, GameOverMenu_2_PlayMusic,
    GameOverMenu_3_FadeIn, GameOverMenu_4_Main, GameOverMenu_5_FadeOutToGameMap,
    GameOverMenu_6_LoadGameMapView, GameOverMenu_7_FadeOutToSoftReset,
};
pub unsafe fn GameOverMenu(game: &mut Game) {
    kGameOverMenuFuncs[word!(game.menu_index) as usize](game);
}
pub unsafe fn GameOverMenu_3_FadeIn(game: &mut Game) {
    HandleGameOverBabyMetroid(game);
    DrawMenuSelectionMissile(game);
    HandleFadeIn(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 15 {
        word!(game.menu_index += 1);
    }
}
pub unsafe fn GameOverMenu_5_FadeOutToGameMap(game: &mut Game) {
    HandleGameOverBabyMetroid(game);
    DrawMenuSelectionMissile(game);
    HandleFadeOut(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
        word!(game.menu_index += 1);
    }
}
pub unsafe fn GameOverMenu_7_FadeOutToSoftReset(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    HandleGameOverBabyMetroid(game);
    HandleFadeOut(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
        SoftReset(game);
    }
}
pub unsafe fn GameOverMenu_6_LoadGameMapView(game: &mut Game) {
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    word! {game; game_state = kGameState_5_FileSelectMap as u16; menu_index = 0; }
}
pub unsafe fn GameOverMenu_4_Main(game: &mut Game) {
    HandleGameOverBabyMetroid(game);
    DrawMenuSelectionMissile(game);
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
            LoadFromSram(word!(game.selected_save_slot), game);
        }
        return;
    }
    let mut v0: u16 = 160;
    if word!(game.file_select_map_area_index) != 0 {
        v0 = 192;
    }
    word! {game; eproj_id[5] = 40; eproj_id[10] = v0; }
}
pub unsafe fn GameOverMenu_1_Init(game: &mut Game) {
    *ram_ptr!(game; 0x69) = 17;
    *ram_ptr!(game; 0x74, 0) = 32;
    *ram_ptr!(game; 0x74, 1) = 64;
    *ram_ptr!(game; 0x74, 2) = 0x80;
    QueueMusic_Delayed8(0, game);
    QueueMusic_Delayed8(0xff03, game);
    word! {game; cinematic_var5 = 1; eproj_id[0] = 0; }
    word_transfers!(game, ascending, 1024; native(0x3600) <- value(15));
    let v1: i32 = word!(game.vram_write_queue_tail) as i32;
    let v2 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
    (*v2).size = 2048;
    (*v2).src.addr = (&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).menu.menu_tilemap as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut u16) = 126;
    (*v2).vram_dst = ((*ram_ptr!(game; 0x58) as i32 & 0xfc) << 8) as u16;
    word!(game.vram_write_queue_tail = (v1 + 7) as u16);
    (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
    LoadMenuTilemap(0x156, addr_kMenuTilemap_GameOver as u16, game);
    LoadMenuTilemap(0x38a, addr_kMenuTilemap_FindMetroidLarva as u16, game);
    LoadMenuTilemap(0x414, addr_kMenuTilemap_TryAgain as u16, game);
    LoadMenuTilemap(0x4ce, addr_kMenuTilemap_YesReturnToGame as u16, game);
    LoadMenuTilemap(0x5ce, addr_kMenuTilemap_NoGoToTitle as u16, game);
    enemy_instruction! {[(*(ram_ptr!(game, EnemyData; 0xf78)))]; instruction = 0, timer = 0}
    HandleGameOverBabyMetroid(game);
    *(ram_ptr!(game, u16; 0x6f)) = ((*ram_ptr!(game; 0x70) as i32) << 8) as u16;
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    EnableHdmaObjects(game);
    static unk_819254: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0, 0x32, 0x927d as VoidP);
    static unk_81925C: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0, 0x31, 0x928d as VoidP);
    SpawnHdmaObject(0x81, &raw const unk_819254 as *const c_void, game);
    SpawnHdmaObject(0x81, &raw const unk_81925C as *const c_void, game);
    ScreenOn(game);
    word!(game.menu_index += 1);
    word! { game, 0;
        screen_fade_delay, screen_fade_counter, file_select_map_area_index,
    }
    word! {game; eproj_id[5] = 40; eproj_id[10] = 160; }
}
pub unsafe fn GameOverMenu_2_PlayMusic(game: &mut Game) {
    if HasQueuedMusic(game) == 0 {
        word!(game.menu_index += 1);
        QueueMusic_Delayed8(4, game);
    }
}
static kFileSelectMenuFuncs: [Func_V; 34] = callback_table! { nonnull Func_V;
    FileSelectMenu_0_FadeOutConfigGfx, FileSelectMenu_1_LoadFileSelectMenuBG2,
    FileSelectMenu_2_InitMain, FileSelectMenu_3_FadeInToMain, FileSelectMenu_4_Main,
    FileSelectMenu_5_FadeOutFromMain, FileSelectMenu_6_FileCopyInit,
    FileSelectMenu_7_FadeInFromMain, FileSelectMenu_8, FileSelectMenu_9_InitializeSelectDest,
    FileSelectMenu_10_FileCopySelectDest, FileSelectMenu_11_InitializeConfirm,
    FileSelectMenu_12_FileCopyConfirm, FileSelectMenu_13_FileCopyDoIt,
    FileSelectMenu_14_CopyCompleted, FileSelectMenu_15_FadeOutToMain, FileSelectMenu_16,
    FileSelectMenu_17_FadeInToMain, FileSelectMenu_18, FileSelectMenu_5_FadeOutFromMain,
    FileSelectMenu_20_FileClearInit, FileSelectMenu_7_FadeInFromMain,
    FileSelectMenu_22_FileClearSelectSlot, FileSelectMenu_23_FileClearInitConfirm,
    FileSelectMenu_24_FileClearConfirm, FileSelectMenu_25_FileClearDoClear,
    FileSelectMenu_26_ClearCompleted, FileSelectMenu_15_FadeOutToMain, FileSelectMenu_16,
    FileSelectMenu_17_FadeInToMain, FileSelectMenu_30, FileSelectMenu_31_TurnSamusHelmet,
    FileSelectMenu_32_FadeOutToOptions, FileSelectMenu_33_FadeOutToTitle,
};
pub unsafe fn FileSelectMenu(game: &mut Game) {
    kFileSelectMenuFuncs[word!(game.menu_index) as usize](game);
}
pub unsafe fn FileSelectMenu_0_FadeOutConfigGfx(game: &mut Game) {
    HandleFadeOut(game);
    if (*ram_ptr!(game; 0x51) & 0xf) != 0 {
        return;
    }
    ScreenOff(game);
    QueueSfx3_Max6(1, game);
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    word!(game.menu_index += 1);
    MapVramForMenu(game);
    LoadInitialMenuTiles(game);
    word! { game, 0;
        reg_BG1HOFS, reg_BG2HOFS, reg_BG3HOFS, reg_BG1VOFS, reg_BG2VOFS, reg_BG3VOFS,
    }
    LoadFileSelectPalettes(game);
}
pub unsafe fn LoadFileSelectPalettes(game: &mut Game) {
    word_transfers!(game, (4), 128;
        le(palette_buffer, 0) <- rom(0x8ee400, 0),
        le(palette_buffer, 1) <- rom(0x8ee400, 1),
    );
}
pub unsafe fn FileSelectMenu_32_FadeOutToOptions(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    DrawBorderAroundSamusData(game);
    DrawFileSelectSamusHelmets(game);
    HandleFadeOut(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
        word! {game; game_state = kGameState_2_GameOptionsMenu as u16; menu_index = 0; }
        word_transfers!(game, ascending, 24; native(0x198d) <- value(0));
    }
}
pub unsafe fn FileSelectMenu_33_FadeOutToTitle(game: &mut Game) {
    DrawBorderAroundSamusData(game);
    DrawFileSelectSamusHelmets(game);
    HandleFadeOut(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
        SoftReset(game);
    }
}
pub unsafe fn FileSelectMenu_5_FadeOutFromMain(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    FileSelectMenu_15_FadeOutToMain(game);
}
pub unsafe fn FileSelectMenu_15_FadeOutToMain(game: &mut Game) {
    HandleFadeOut(game);
    *ram_ptr!(game; 0x57) = (*ram_ptr!(game; 0x57) as i32 & 0xf
        | 16 * (*ram_ptr!(game; 0x51) as i32 & 0xf) ^ 0xf0) as u8;
    if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
        ScreenOff(game);
        word!(game.menu_index += 1);
    }
}
static kMenuSelectionMissileXY: [u16; 12] = [
    0x30, 0xe, 0x58, 0xe, 0x80, 0xe, 0xa3, 0xe, 0xbb, 0xe, 0xd3, 0xe,
];
pub unsafe fn FileSelectMenu_17_FadeInToMain(game: &mut Game) {
    let v0: i32 = ((4u16).wrapping_mul(word!(game.selected_save_slot))) as i32 >> 1;
    word!(game.eproj_id[10] = kMenuSelectionMissileXY[v0 as usize]);
    word!(game.eproj_id[5] = kMenuSelectionMissileXY[(v0 + 1) as usize]);
    FileSelectMenu_7_FadeInFromMain(game);
}
pub unsafe fn FileSelectMenu_7_FadeInFromMain(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    HandleFadeIn(game);
    *ram_ptr!(game; 0x57) = (*ram_ptr!(game; 0x57) as i32 & 0xf
        | 16 * (*ram_ptr!(game; 0x51) as i32 & 0xf) ^ 0xf0) as u8;
    if (*ram_ptr!(game; 0x51) & 0xf) == 15 {
        word!(game.menu_index += 1);
    }
}
pub unsafe fn FileSelectMenu_6_FileCopyInit(game: &mut Game) {
    word!(game.menu_index += 1);
    FileSelectMenu_Func1(game);
}
pub unsafe fn FileSelectMenu_Func1(game: &mut Game) {
    ClearMenuTilemap(game);
    (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
    LoadMenuTilemap(0x52, addr_kMenuTilemap_DataCopyMode as u16, game);
    LoadMenuTilemap(0x150, addr_kMenuTilemap_CopyWhichData as u16, game);
    LoadMenuExitTilemap(game);
    DrawFileCopySaveFileInfo(game);
    SetInitialFileCopyMenuSelection(game);
    SetFileCopyMenuSelectionMissilePosition(game);
    word! { game, 0; eproj_id[16], eproj_id[17] }
}
pub unsafe fn SetInitialFileCopyMenuSelection(game: &mut Game) {
    let mut v0: i32 = word!(game.nonempty_save_slots) as i32;
    let mut v1: i32 = 0;
    while (v0 & 1 == 0) {
        v0 >>= 1;
        v1 += 1;
        if (((v1 - 3i32) as i16) >= 0) {
            break;
        }
    }
    word!(game.eproj_id[15] = v1 as u16);
}
pub unsafe fn ClearMenuTilemap(game: &mut Game) {
    word_transfers!(game, descending, 1024; native(0x3dfe) <- value(15));
}
pub unsafe fn LoadMenuExitTilemap(game: &mut Game) {
    LoadMenuTilemap(0x688, addr_kMenuTilemap_Exit as u16, game);
}
#[derive(Clone, Copy)]
enum FileRowEmphasis {
    Occupied,
    Destination,
    Confirmation,
}
unsafe fn draw_file_copy_rows(emphasis: FileRowEmphasis, game: &mut Game) {
    for slot in 0..3u16 {
        LoadFromSram_(slot, game);
        let dim = match emphasis {
            FileRowEmphasis::Occupied => word!(game.nonempty_save_slots) & (1 << slot) == 0,
            FileRowEmphasis::Destination => word!(game.eproj_id[16]) == slot,
            FileRowEmphasis::Confirmation => {
                word!(game.eproj_id[16]) != slot && word!(game.eproj_id[17]) != slot
            }
        };
        (*ram_ptr!(game, EnemyData; 0xf78)).palette_index = if dim { 1024 } else { 0 };
        draw_file_copy_row(slot, game);
    }
    QueueTransferOfMenuTilemapToVramBG1(game);
}
unsafe fn draw_file_copy_row(slot: u16, game: &mut Game) {
    let mask = 1 << slot;
    let base = (slot + 2) << 8;
    DrawFileSelectionHealth(!word!(game.nonempty_save_slots) & mask, base + 0x18, game);
    DrawFileSelectionTime(!word!(game.nonempty_save_slots) & mask, base + 0x72, game);
    LoadMenuTilemap(base + 0x34, addr_kMenuTilemap_TIME as u16, game);
    let label = [
        addr_kMenuTilemap_SamusA,
        addr_kMenuTilemap_SamusB,
        addr_kMenuTilemap_SamusC,
    ][slot as usize];
    LoadMenuTilemap(base + 8, label as u16, game);
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    DrawFileCopySaveSlotAInfo => draw_file_copy_row(0, game);
    DrawFileCopySaveSlotBInfo => draw_file_copy_row(1, game);
    DrawFileCopySaveSlotCInfo => draw_file_copy_row(2, game);
}
pub unsafe fn DrawFileCopySaveFileInfo(game: &mut Game) {
    draw_file_copy_rows(FileRowEmphasis::Occupied, game);
}
pub unsafe fn QueueTransferOfMenuTilemapToVramBG1(game: &mut Game) {
    let v0: u16 = word!(game.vram_write_queue_tail);
    let v1 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
    (*v1).size = 2048;
    (*v1).src.addr = (&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).menu.menu_tilemap as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as VoidP;
    *(&raw mut (*v1).src.bank as *mut u16) = 126;
    (*v1).vram_dst = ((*ram_ptr!(game; 0x58) as i32 & 0xfc) << 8) as u16;
    word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
}
static kBitShl: [u8; 3] = [1, 2, 4];
// Search toward the pressed direction without wrapping through the exit row.
unsafe fn next_occupied_file_slot(game: &mut Game) -> Option<u8> {
    let up = if word!(game.joypad1_newkeys) as i32 & kButton_Up as i32 != 0 {
        true
    } else if word!(game.joypad1_newkeys) as i32 & kButton_Down as i32 != 0 {
        false
    } else {
        return None;
    };
    let mut slot = word!(game.eproj_id[15]) as u8;
    loop {
        if up {
            slot = slot.wrapping_sub(1);
            if slot & 0x80 != 0 {
                return None;
            }
        } else {
            slot = slot.wrapping_add(1);
            if (slot as i8).wrapping_sub(4) >= 0 {
                return None;
            }
            if slot == 3 {
                return Some(slot);
            }
        }
        if kBitShl[slot as usize] as u16 & word!(game.nonempty_save_slots) != 0 {
            return Some(slot);
        }
    }
}
pub unsafe fn FileSelectMenu_8(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    DrawMenuSelectionMissile(game);
    if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0 {
        QueueSfx1_Max6(0x37, game);
        if word!(game.eproj_id[15]) == 3 {
            word!(game.menu_index += 7);
        } else {
            word!(game.eproj_id[16] = word!(game.eproj_id[15]));
            word!(game.menu_index += 1);
        }
    } else if (word!(game.joypad1_newkeys) & 0x8000) != 0 {
        word!(game.menu_index += 7);
        QueueSfx1_Max6(0x37, game);
    } else {
        let selection = next_occupied_file_slot(game);
        if let Some(v0) = selection {
            byte!(@pointer (ram_ptr!(game, u16; 0x1997)).offset(15); 0) = v0;
            QueueSfx1_Max6(0x37, game);
            SetFileCopyMenuSelectionMissilePosition(game);
            return;
        }
        SetFileCopyMenuSelectionMissilePosition(game);
    };
}
pub unsafe fn SetFileCopyMenuSelectionMissilePosition(game: &mut Game) {
    static kFileCopyMissileY: [u16; 4] = [72, 104, 136, 211];
    word!(game.eproj_id[10] = kFileCopyMissileY[word!(game.eproj_id[15]) as usize]);
    word!(game.eproj_id[5] = 22);
}
pub unsafe fn FileSelectMenu_9_InitializeSelectDest(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    DrawFileCopySelectDestinationSaveFileInfo(game);
    word!(game.menu_index += 1);
    let mut v0: u16 = 0;
    while (v0 == word!(game.eproj_id[16])) {
        v0 = v0.wrapping_add(1);
        if (v0 as i32 - 3i32 & 0x8000 == 0) {
            break;
        }
    }
    word!(game.eproj_id[15] = v0);
    SetFileCopyMenuSelectionMissilePosition(game);
}
pub unsafe fn DrawFileCopySelectDestinationSaveFileInfo(game: &mut Game) {
    ClearMenuTilemap(game);
    (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
    LoadMenuTilemap(0x52, addr_kMenuTilemap_DataCopyMode as u16, game);
    (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
    LoadMenuTilemap(0x148, addr_kMenuTilemap_CopySamusToWhere as u16, game);
    (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[944i32 as usize] =
        (word!(game.eproj_id[16])).wrapping_add(8298);
    LoadMenuExitTilemap(game);
    draw_file_copy_rows(FileRowEmphasis::Destination, game);
}
pub unsafe fn FileSelectMenu_10_FileCopySelectDest(game: &mut Game) {
    static kFileCopySelectDest_MissileY: [u16; 4] = [72, 104, 136, 212];
    let mut v0: u16 = 0;
    DrawBorderAroundDataCopyMode(game);
    DrawMenuSelectionMissile(game);
    if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0 {
        QueueSfx1_Max6(0x37, game);
        if word!(game.eproj_id[15]) != 3 {
            word!(game.eproj_id[17] = word!(game.eproj_id[15]));
            word!(game.menu_index += 1);
        } else {
            word!(game.menu_index += 5);
            return;
        }
    } else if (word!(game.joypad1_newkeys) & 0x8000) == 0 {
        'move_selection: {
            if (word!(game.joypad1_newkeys) as i32 & kButton_Up as i32) != 0 {
                QueueSfx1_Max6(0x37, game);
                v0 = word!(game.eproj_id[15]);
                do_until! { v0 != word!(game.eproj_id[16]);
                    v0 = v0.wrapping_sub(1);
                    if ((v0 & 0x8000) != 0) {
                        break 'move_selection;
                    }
                }
            } else if (word!(game.joypad1_newkeys) as i32 & kButton_Down as i32) != 0 {
                QueueSfx1_Max6(0x37, game);
                v0 = word!(game.eproj_id[15]);
                do_until! { v0 != word!(game.eproj_id[16]);
                    v0 = v0.wrapping_add(1);
                    if (v0 == 4) {
                        break 'move_selection;
                    }
                }
            } else {
                break 'move_selection;
            }
            word!(game.eproj_id[15] = v0);
        }
    } else {
        word!(game.menu_index -= 2);
        word!(game.eproj_id[15] = word!(game.eproj_id[16]));
        QueueSfx1_Max6(0x37, game);
        FileSelectMenu_Func1(game);
        return;
    }
    word!(game.eproj_id[10] = kFileCopySelectDest_MissileY[word!(game.eproj_id[15]) as usize]);
    word!(game.eproj_id[5] = 22);
}
pub unsafe fn FileSelectMenu_11_InitializeConfirm(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
    LoadMenuTilemap(0x144, addr_kMenuTilemap_CopySamusToSamus as u16, game);
    (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[942i32 as usize] =
        (word!(game.eproj_id[16])).wrapping_add(8298);
    (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[955i32 as usize] =
        (word!(game.eproj_id[17])).wrapping_add(8298);
    DrawFileCopyClearConfirmation(game);
    word!(game.menu_index += 1);
    word! {game; eproj_id[15] = 0; cinematic_var6 = 8; }
}
pub unsafe fn DrawFileCopyClearConfirmation(game: &mut Game) {
    (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
    LoadMenuTilemap(0x514, addr_kMenuTilemap_IsThisOk as u16, game);
    (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
    LoadMenuTilemap(0x59c, addr_kMenuTilemap_Yes as u16, game);
    word_transfers!(game, ascending, 32; native(0x3c80) <- value(15));
    (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
    LoadMenuTilemap(0x65c, addr_kMenuTilemap_No as u16, game);
    DrawFileCopyConfirmationSaveFileInfo(game);
}
pub unsafe fn DrawFileCopyConfirmationSaveFileInfo(game: &mut Game) {
    draw_file_copy_rows(FileRowEmphasis::Confirmation, game);
}
pub unsafe fn FileSelectMenu_12_FileCopyConfirm(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    DrawMenuSelectionMissile(game);
    HandleFileCopyArrowPalette(game);
    DrawFileCopyArrow(game);
    if word!(game.joypad1_newkeys) as i32 & (kButton_Up as i32 | kButton_Down as i32) != 0 {
        word!(game.eproj_id[15] ^= (1u16));
        QueueSfx1_Max6(0x37, game);
    } else {
        if (word!(game.joypad1_newkeys) & 0x8000) != 0 {
            word!(game.menu_index -= 3);
            word!(game.eproj_id[15] = word!(game.eproj_id[17]));
            QueueSfx1_Max6(0x37, game);
            return;
        }
        if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0 {
            QueueSfx1_Max6(0x38, game);
            if word!(game.eproj_id[15]) != 0 {
                word!(game.menu_index -= 4);
                FileSelectMenu_Func1(game);
            } else {
                word!(game.menu_index += 1);
            }
            return;
        }
    }
    let mut v0: u16 = 184;
    if word!(game.eproj_id[15]) != 0 {
        v0 = 208;
    }
    word! {game; eproj_id[10] = v0; eproj_id[5] = 94; }
}
pub unsafe fn HandleFileCopyArrowPalette(game: &mut Game) {
    if word!(game.cinematic_var6) != 0 {
        if word!(game.cinematic_var6 -= 1, reread) == 0 {
            word!(game.cinematic_var6 = 4);
            let v0: u16 = word!(game.palette_buffer[145]);
            word_transfers!(game, ascending, 6; native(0xc122) <- native(0xc124));
            word!(game.palette_buffer[151] = v0);
        }
    }
}
pub unsafe fn FileSelectMenu_13_FileCopyDoIt(game: &mut Game) {
    static kBitShl_16bit: [u16; 3] = [1, 2, 4];
    DrawBorderAroundDataCopyMode(game);
    DrawMenuSelectionMissile(game);
    HandleFileCopyArrowPalette(game);
    DrawFileCopyArrow(game);
    let src_addr: u16 =
        crate::rtl::rom_table_word!(game, 0x81812b, word!(game.eproj_id[16]) as isize);
    let dst_addr: u16 =
        crate::rtl::rom_table_word!(game, 0x81812b, word!(game.eproj_id[17]) as isize);
    game.g_sram.copy_within(
        src_addr as usize..src_addr as usize + 1628,
        dst_addr as usize,
    );
    let v2: i32 = word!(game.eproj_id[16]) as i32;
    let v10: i32 = game.g_sram.word((2 * v2 + 0x1ff0) as usize) as i32;
    let v9: i32 = game.g_sram.word((2 * v2 + 0x1ff8) as usize) as i32;
    let v8: i32 = game.g_sram.word((2 * v2) as usize) as i32;
    let v4: i32 = word!(game.eproj_id[17]) as i32;
    game.g_sram.set_word(
        (2 * v4 + 8) as usize,
        game.g_sram.word((2 * v2 + 8) as usize),
    );
    game.g_sram.set_word((2 * v4) as usize, v8 as u16);
    game.g_sram.set_word((2 * v4 + 0x1ff8) as usize, v9 as u16);
    game.g_sram.set_word((2 * v4 + 0x1ff0) as usize, v10 as u16);
    word!(game.menu_index += 1);
    word_transfers!(game, ascending, 288; native(0x3b00) <- value(15));
    word!(game.nonempty_save_slots |= (kBitShl_16bit[word!(game.eproj_id[17]) as usize]));
    let mut v6: i32 = ((4 * word!(game.eproj_id[17]) as i32 + 9) << 6) + 24;
    let mut v7: i32 = 0;
    do_until! { (((v7 - 22i32) as i16) >= 0);
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(768) as *mut u8)
            .offset(v6 as isize) as *mut u16) = 15;
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(736) as *mut u8)
            .offset(v6 as isize) as *mut u16) = 15;
        v6 += 2;
        v7 += 2;
    }
    (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
    LoadMenuTilemap(0x510, addr_kMenuTilemap_CopyCompleted as u16, game);
    DrawFileCopyConfirmationSaveFileInfo(game);
    RtlWriteSram(game);
}
pub unsafe fn FileSelectMenu_14_CopyCompleted(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    if word!(game.joypad1_newkeys) != 0 {
        QueueSfx1_Max6(0x37, game);
        word!(game.menu_index += 1);
        let mut slot: i32 = game.g_sram.word(0x1fec) as i32;
        if slot & 0x8000 != 0
            || game.g_sram.word(0x1fec) as i32 - 3i32 & 0x8000 == 0
            || slot & game.g_sram.word(0x1fee) as i32 != 0
        {
            slot = 0;
        }
        word!(game.selected_save_slot = slot as u16);
    }
}
pub unsafe fn FileSelectMenu_18(game: &mut Game) {
    word!(game.menu_index -= 14);
}
pub unsafe fn FileSelectMenu_20_FileClearInit(game: &mut Game) {
    DrawBorderAroundDataClearMode(game);
    word!(game.menu_index += 1);
    InitFileSelectMenuFileClear(game);
}
pub unsafe fn InitFileSelectMenuFileClear(game: &mut Game) {
    ClearMenuTilemap(game);
    (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
    LoadMenuTilemap(0x50, addr_kMenuTilemap_DataClearMode as u16, game);
    LoadMenuTilemap(0x140, addr_kMenuTilemap_ClearWhichData as u16, game);
    LoadMenuExitTilemap(game);
    DrawFileCopySaveFileInfo(game);
    word!(game.eproj_id[16] = 0);
    SetInitialFileCopyMenuSelection(game);
    SetFileClearMenuMissilePos(game);
}
pub unsafe fn FileSelectMenu_22_FileClearSelectSlot(game: &mut Game) {
    let mut v0: u16 = 0;
    DrawBorderAroundDataClearMode(game);
    DrawMenuSelectionMissile(game);
    byte!(v0; 1) = byte!(@pointer ram_ptr!(game, u16; 0x8f); 1);
    if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0 {
        QueueSfx1_Max6(0x37, game);
        if word!(game.eproj_id[15]) != 3 {
            word!(game.eproj_id[16] = word!(game.eproj_id[15]));
            word!(game.menu_index += 1);
            return;
        }
    } else if ((word!(game.joypad1_newkeys) & 0x8000) == 0) {
        let selection = next_occupied_file_slot(game);
        if let Some(v1) = selection {
            byte!(@pointer (ram_ptr!(game, u16; 0x1997)).offset(15); 0) = v1;
            byte!(v0; 0) = 55;
            QueueSfx1_Max6(v0, game);
            SetFileClearMenuMissilePos(game);
            return;
        }
        SetFileClearMenuMissilePos(game);
        return;
    }
    QueueSfx1_Max6(0x37, game);
    word!(game.menu_index += 5);
}
pub unsafe fn SetFileClearMenuMissilePos(game: &mut Game) {
    static kFileClear_MissileY: [u16; 4] = [72, 104, 136, 211];
    word!(game.eproj_id[10] = kFileClear_MissileY[word!(game.eproj_id[15]) as usize]);
    word!(game.eproj_id[5] = 22);
}
pub unsafe fn FileSelectMenu_23_FileClearInitConfirm(game: &mut Game) {
    DrawBorderAroundDataClearMode(game);
    (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
    LoadMenuTilemap(0x140, addr_kMenuTilemap_ClearSamusA as u16, game);
    (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[949i32 as usize] =
        (word!(game.eproj_id[16])).wrapping_add(8298);
    word!(game.eproj_id[17] = 3);
    DrawFileCopyClearConfirmation(game);
    word!(game.menu_index += 1);
    word!(game.eproj_id[15] = 0);
}
pub unsafe fn FileSelectMenu_24_FileClearConfirm(game: &mut Game) {
    let keep_confirming: bool;
    DrawBorderAroundDataClearMode(game);
    DrawMenuSelectionMissile(game);
    if word!(game.joypad1_newkeys) as i32 & (kButton_Up as i32 | kButton_Down as i32) != 0 {
        word!(game.eproj_id[15] ^= (1u16));
        QueueSfx1_Max6(0x37, game);
    } else {
        if (word!(game.joypad1_newkeys) & 0x8000) != 0 {
            keep_confirming = false;
        } else if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32)
            == 0
        {
            keep_confirming = true;
        } else {
            QueueSfx1_Max6(0x38, game);
            if word!(game.eproj_id[15]) != 0 {
                keep_confirming = false;
            } else {
                word!(game.menu_index += 1);
                return;
            }
        }
        if keep_confirming {
        } else {
            word!(game.menu_index -= 2);
            word!(game.eproj_id[15] = word!(game.eproj_id[16]));
            QueueSfx1_Max6(0x37, game);
            InitFileSelectMenuFileClear(game);
            return;
        }
    }
    let mut v0: u16 = 184;
    if word!(game.eproj_id[15]) != 0 {
        v0 = 208;
    }
    word! {game; eproj_id[10] = v0; eproj_id[5] = 94; }
}
pub unsafe fn FileSelectMenu_25_FileClearDoClear(game: &mut Game) {
    static kBitShl_Not: [u16; 3] = [0xfffe, 0xfffd, 0xfffb];
    DrawBorderAroundDataClearMode(game);
    let sram_addr: i32 =
        crate::rtl::rom_table_word!(game, 0x81812b, word!(game.eproj_id[16]) as isize) as i32;
    game.g_sram
        .fill(sram_addr as usize..sram_addr as usize + 1628, 0);
    let v1: u16 = (2u16).wrapping_mul(word!(game.eproj_id[16]));
    let v2: i32 = word!(game.eproj_id[16]) as i32;
    game.g_sram.set_word((2 * v2) as usize, 0);
    game.g_sram.set_word((2 * v2 + 8) as usize, 0);
    game.g_sram.set_word((2 * v2 + 0x1ff0) as usize, 0);
    game.g_sram.set_word((2 * v2 + 0x1ff8) as usize, 0);
    word!(game.menu_index += 1);
    NewSaveFile(game);
    LoadFromSram(word!(game.eproj_id[16]), game);
    word!(game.area_index = word!(game.eproj_id[16]));
    LoadMirrorOfExploredMapTiles(game);
    word_transfers!(game, ascending, 288; native(0x3b00) <- value(15));
    word!(game.nonempty_save_slots &= (kBitShl_Not[word!(game.eproj_id[16]) as usize]));
    (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
    LoadMenuTilemap(0x500, addr_kMenuTilemap_DataCleared as u16, game);
    DrawFileCopyConfirmationSaveFileInfo(game);
    RtlWriteSram(game);
}
pub unsafe fn FileSelectMenu_26_ClearCompleted(game: &mut Game) {
    DrawBorderAroundDataClearMode(game);
    if word!(game.joypad1_newkeys) != 0 {
        QueueSfx1_Max6(0x37, game);
        word!(game.menu_index += 1);
        if (LoadFromSram(0, game) != 0) {
            let use_first_slot: bool = if LoadFromSram(1, game) != 0 {
                (LoadFromSram(2, game) != 0)
                    || ({
                        word!(game.selected_save_slot = 2);
                        false
                    })
            } else {
                word!(game.selected_save_slot = 1);
                false
            };
            if !(use_first_slot) {
                return;
            }
        }
        word!(game.selected_save_slot = 0);
    }
}
pub unsafe fn FileSelectMenu_30(game: &mut Game) {
    DrawBorderAroundSamusData(game);
    word!(game.menu_index -= 26);
}
pub unsafe fn FileSelectMenu_31_TurnSamusHelmet(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    DrawBorderAroundSamusData(game);
    DrawFileSelectSlotSamusHelmet(4, game);
    DrawFileSelectSlotSamusHelmet(6, game);
    DrawFileSelectSlotSamusHelmet(8, game);
    if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0
        || word!(game.eproj_id[2]) == 7 && word!(game.cinematic_var7) == 0
        || word!(game.eproj_id[3]) == 7 && word!(game.cinematic_var8) == 0
        || word!(game.eproj_id[4]) == 7 && word!(game.cinematic_var9) == 0
    {
        word!(game.menu_index += 1);
    }
}
pub unsafe fn DrawFileSelectSamusHelmets(game: &mut Game) {
    word!(game.cinematic_var7 = 0);
    DrawFileSelectSlotSamusHelmet(4, game);
    word!(game.cinematic_var8 = 0);
    DrawFileSelectSlotSamusHelmet(6, game);
    word!(game.cinematic_var9 = 0);
    DrawFileSelectSlotSamusHelmet(8, game);
}
pub unsafe fn DrawFileSelectSlotSamusHelmet(k: u16, game: &mut Game) {
    let v0: u16 = k;
    let mut v1: i16 = *(ram_ptr!(game, u16; 0x198d, v0 as i32 as isize)) as i16;
    if v1 != 0 {
        let v2: u16 = ((v1) as u16).wrapping_sub(1);
        *(ram_ptr!(game, u16; 0x198d, v0 as i32 as isize)) = v2;
        if v2 == 0 {
            *(ram_ptr!(game, u16; 0x198d, v0 as i32 as isize)) = 8;
            let v3: i32 = v0 as i32 >> 1;
            let mut v4: u16 = (word!(game.eproj_id[v3])).wrapping_add(1);
            if word!(game.eproj_id[v3]) as i32 - 7i32 & 0x8000 == 0 {
                *(ram_ptr!(game, u16; 0x198d, v0 as i32 as isize)) = 0;
                v4 = 7;
            }
            word!(game.eproj_id[v3] = v4);
        }
    }
    let v5: i32 = v0 as i32 >> 1;
    let v6: u16 = (2u16).wrapping_mul(word!(game.eproj_id[v5]));
    static kDrawFileSlotHelmet_Spritemaps: [u16; 9] =
        [0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x33];
    DrawMenuSpritemap(
        kDrawFileSlotHelmet_Spritemaps[(v6 >> 1) as usize],
        word!(game.eproj_id[(v5 + 5)]),
        word!(game.eproj_id[(v5 + 10)]),
        3584,
        game,
    );
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
    kFileSelectMapFuncs[word!(game.menu_index) as usize](game);
}
pub unsafe fn FileSelectMap_22(game: &mut Game) {
    DrawAreaSelectMapLabels(game);
    HandleFadeOut(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
        word! {game; game_state = kGameState_2_GameOptionsMenu as u16; menu_index = 0; }
    }
}
pub unsafe fn FileSelectMenu_1_LoadFileSelectMenuBG2(game: &mut Game) {
    // This menu spans $3600..$3dff, beyond the pause-map union field.
    let menu_tilemap = (&raw mut game.g_ram as *mut u8).add(0x3600).cast::<u16>();
    word_transfers!(game, descending, 1024; native(0x3dfe) <- rom(0x8edc00, 1023));
    let v1: u16 = word!(game.vram_write_queue_tail);
    let mut v2 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
    (*v2).size = 2048;
    (*v2).src.addr = (&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).menu.menu_tilemap as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut u16) = 126;
    (*v2).vram_dst = ((*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8) as u16;
    word!(game.vram_write_queue_tail = (v1).wrapping_add(7));
    word!(game.menu_index += 1);
    word! {game; cinematic_var5 = 1; eproj_id[0] = 0; }
}
pub unsafe fn FileSelectMenu_2_InitMain(game: &mut Game) {
    let mut v0: u16 = 0;
    if game.g_sram.word(0x1fec) & 0x8000 != 0
        || game.g_sram.word(0x1fec) as i32 - 3i32 & 0x8000 == 0
        || {
            v0 = game.g_sram.word(0x1fec);
            (game.g_sram.word(0x1fee) & game.g_sram.word(0x1fec)) != 0
        }
    {
        v0 = 0;
    }
    word!(game.selected_save_slot = v0);
    FileSelectMenu_16(game);
}
pub unsafe fn FileSelectMenu_16(game: &mut Game) {
    word_transfers!(game, descending, 1024; native(0x3dfe) <- value(15));
    word!(game.nonempty_save_slots = -1i32 as u16);
    (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
    LoadMenuTilemap(0x56, addr_kMenuTilemap_SamusData as u16, game);
    LoadMenuTilemap(0x146, addr_kMenuTilemap_SamusA as u16, game);
    let mut c: u8 = LoadFromSram_(0, game);
    word!(
        game.nonempty_save_slots =
            (word!(game.nonempty_save_slots) as i32 >> 1 | (c as i32) << 15) as u16
    );
    DrawFileSelectionHealth((word!(game.nonempty_save_slots)) & (0x8000u16), 0x15c, game);
    DrawFileSelectionTime((word!(game.nonempty_save_slots)) & (0x8000u16), 0x1b4, game);
    LoadMenuTilemap(0x176, addr_kMenuTilemap_TIME as u16, game);
    (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
    LoadMenuTilemap(0x286, addr_kMenuTilemap_SamusB as u16, game);
    c = LoadFromSram_(1, game);
    word!(
        game.nonempty_save_slots =
            (word!(game.nonempty_save_slots) as i32 >> 1 | (c as i32) << 15) as u16
    );
    DrawFileSelectionHealth((word!(game.nonempty_save_slots)) & (0x8000u16), 0x29c, game);
    DrawFileSelectionTime((word!(game.nonempty_save_slots)) & (0x8000u16), 0x2f4, game);
    LoadMenuTilemap(0x2b6, addr_kMenuTilemap_TIME as u16, game);
    (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
    LoadMenuTilemap(0x3c6, addr_kMenuTilemap_SamusC as u16, game);
    c = LoadFromSram_(2, game);
    word!(
        game.nonempty_save_slots =
            (word!(game.nonempty_save_slots) as i32 >> 1 | (c as i32) << 15) as u16
    );
    DrawFileSelectionHealth((word!(game.nonempty_save_slots)) & (0x8000u16), 0x3dc, game);
    DrawFileSelectionTime((word!(game.nonempty_save_slots)) & (0x8000u16), 0x434, game);
    LoadMenuTilemap(0x3f6, addr_kMenuTilemap_TIME as u16, game);
    word!(
        game.nonempty_save_slots =
            (u16::swap_bytes(!(word!(game.nonempty_save_slots) as i32) as u16) >> 5)
    );
    if word!(game.nonempty_save_slots) != 0 {
        (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
        LoadMenuTilemap(0x508, addr_kMenuTilemap_DataCopy as u16, game);
        (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
        LoadMenuTilemap(0x5c8, addr_kMenuTilemap_DataClear as u16, game);
    }
    (*(ram_ptr!(game, EnemyData; 0xf78))).palette_index = 0;
    LoadMenuTilemap(0x688, addr_kMenuTilemap_Exit as u16, game);
    QueueTransferOfMenuTilemapToVramBG1(game);
    word!(game.cinematic_var5 = 1);
    word! { game, 0;
        cinematic_var6, cinematic_var7, cinematic_var8, cinematic_var9, eproj_id[0], eproj_id[1],
        eproj_id[2], eproj_id[3], eproj_id[4], eproj_id[5], eproj_id[10], eproj_id[6], eproj_id[11],
    }
    word! { game, 100; eproj_id[7], eproj_id[8], eproj_id[9] }
    word! {game; eproj_id[12] = 47; eproj_id[13] = 87; eproj_id[14] = 127; }
    word! { game, 1; screen_fade_delay, screen_fade_counter }
    ScreenOn(game);
    word!(game.menu_index += 1);
    word! { game, 0; eproj_id[15], eproj_id[16], eproj_id[17] }
}
pub unsafe fn LoadFromSram_(a: u16, game: &mut Game) -> u8 {
    LoadFromSram(a, game)
}
pub unsafe fn FileSelectMenu_3_FadeInToMain(game: &mut Game) {
    DrawFileSelectSamusHelmets(game);
    let v0: i32 = ((4u16).wrapping_mul(word!(game.selected_save_slot))) as i32 >> 1;
    word!(game.eproj_id[10] = kMenuSelectionMissileXY[v0 as usize]);
    word!(game.eproj_id[5] = kMenuSelectionMissileXY[(v0 + 1) as usize]);
    DrawMenuSelectionMissile(game);
    DrawBorderAroundSamusData(game);
    HandleFadeIn(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 15 {
        word!(game.menu_index += 1);
    }
}
pub unsafe fn DrawFileSelectionHealth(a: u16, k: u16, game: &mut Game) {
    if a != 0 {
        FileSelectClearRestOfMenuTilemapRow(k, game);
        LoadMenuTilemap((k).wrapping_add(64), addr_kMenuTilemap_NoData as u16, game);
    } else {
        LoadMenuTilemap(k, addr_kMenuTilemap_Energy as u16, game);
        let mut v2: i16 = ((k) as i16).wrapping_add(8);
        let mut n: i32 = word!(game.samus_health) as i32 / 100;
        let mut q: i32 = word!(game.samus_health) as i32 % 100;
        let mut mh: i32 = word!(game.samus_max_health) as i32 / 100;
        let mut m: i32 = 7;
        let mut v3: u16 = ((v2) as u16).wrapping_add(64);
        loop {
            mh -= 1;
            if (mh < 0) {
                break;
            }
            let mut v4: i16 = 153;
            if n != 0 {
                n -= 1;
                v4 = 152;
            }
            *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
                .offset(768) as *mut u8)
                .offset(v3 as i32 as isize) as *mut u16) =
                ((*(ram_ptr!(game, EnemyData; 0xf78))).palette_index) | ((v4) as u16);
            v3 = (v3).wrapping_add(2);
            m -= 1;
            if m == 0 {
                v3 = (v3).wrapping_sub(78);
                m = 8;
            }
        }
        n = q / 10;
        q = q % 10;
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(802) as *mut u8)
            .offset(k as i32 as isize) as *mut u16) =
            ((*(ram_ptr!(game, EnemyData; 0xf78))).palette_index as i32 | q + 8288) as u16;
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(801) as *mut u8)
            .offset(k as i32 as isize) as *mut u16) =
            ((*(ram_ptr!(game, EnemyData; 0xf78))).palette_index as i32 | n + 8288) as u16;
    };
}
pub unsafe fn DrawFileSelectionTime(a: u16, k: u16, game: &mut Game) {
    if a == 0 {
        let r26: u16 = k;
        let div_val: i32 = word!(game.game_time_hours) as i32 / 10;
        let mod_val: i32 = word!(game.game_time_hours) as i32 % 10;
        let v2: u16 = r26;
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(769) as *mut u8)
            .offset(v2 as i32 as isize) as *mut u16) =
            ((*(ram_ptr!(game, EnemyData; 0xf78))).palette_index as i32 | mod_val + 8288) as u16;
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(768) as *mut u8)
            .offset(v2 as i32 as isize) as *mut u16) =
            ((*(ram_ptr!(game, EnemyData; 0xf78))).palette_index as i32 | div_val + 8288) as u16;
        LoadMenuTilemap((r26).wrapping_add(4), addr_word_81B4A8 as u16, game);
        let div_min: i32 = word!(game.game_time_minutes) as i32 / 10;
        let mod_min: i32 = word!(game.game_time_minutes) as i32 % 10;
        let v3: u16 = r26;
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(772) as *mut u8)
            .offset(v3 as i32 as isize) as *mut u16) =
            ((*(ram_ptr!(game, EnemyData; 0xf78))).palette_index as i32 | mod_min + 8288) as u16;
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(771) as *mut u8)
            .offset(v3 as i32 as isize) as *mut u16) =
            ((*(ram_ptr!(game, EnemyData; 0xf78))).palette_index as i32 | div_min + 8288) as u16;
    }
}
pub unsafe fn FileSelectMenu_4_Main(game: &mut Game) {
    let mut v1: u16 = 0;
    let mut v2: u16 = 0;
    DrawBorderAroundSamusData(game);
    DrawMenuSelectionMissile(game);
    DrawFileSelectSlotSamusHelmet(4, game);
    DrawFileSelectSlotSamusHelmet(6, game);
    DrawFileSelectSlotSamusHelmet(8, game);
    if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) == 0 {
        let selection_unchanged: bool =
            if (word!(game.joypad1_newkeys) as i32 & kButton_Up as i32) != 0 {
                if word!(game.nonempty_save_slots) != 0 {
                    v1 = (word!(game.selected_save_slot)).wrapping_sub(1);
                    if ((word!(game.selected_save_slot)) as i16).wrapping_sub(1) < 0 {
                        v1 = 5;
                    }
                } else {
                    v1 = (word!(game.selected_save_slot)).wrapping_sub(1);
                    if ((word!(game.selected_save_slot)) as i16).wrapping_sub(1) >= 0 {
                        if word!(game.selected_save_slot) as i32 - 5i32 & 0x8000 == 0 {
                            v1 = 2;
                        }
                    } else {
                        v1 = 5;
                    }
                }
                word!(game.selected_save_slot = v1);
                false
            } else if (word!(game.joypad1_newkeys) as i32 & kButton_Down as i32) == 0 {
                if (word!(game.joypad1_newkeys) as i32 & kButton_B as i32) != 0 {
                    QueueSfx1_Max6(0x37, game);
                    word!(game.menu_index = 33);
                    QueueSfx1_Max6(0x37, game);
                }
                true
            } else {
                if word!(game.nonempty_save_slots) != 0 {
                    v2 = (word!(game.selected_save_slot)).wrapping_add(1);
                    if word!(game.selected_save_slot) as i32 - 5i32 & 0x8000 == 0 {
                        v2 = 0;
                    }
                } else {
                    v2 = (word!(game.selected_save_slot)).wrapping_add(1);
                    if word!(game.selected_save_slot) as i32 - 2i32 & 0x8000 == 0 {
                        if word!(game.selected_save_slot) as i32 - 5i32 & 0x8000 != 0 {
                            v2 = 5;
                        } else {
                            v2 = 0;
                        }
                    }
                }
                word!(game.selected_save_slot = v2);
                false
            };
        if !(selection_unchanged) {
            QueueSfx1_Max6(0x37, game);
        }
    } else {
        let mut v0: i16 = word!(game.selected_save_slot) as i16;
        if word!(game.selected_save_slot) as i32 - 3i32 & 0x8000 != 0 {
            QueueSfx1_Max6(0x2a, game);
            word!(game.menu_index += 27);
            *(ram_ptr!(game, u16; 0x198d, (2 * (word!(game.selected_save_slot) as i32 + 2)) as u16 as i32 as isize)) =
                1;
            game.g_sram.set_word(0x1fec, word!(game.selected_save_slot));
            game.g_sram
                .set_word(0x1fee, !(word!(game.selected_save_slot) as i32) as u16);
            RtlWriteSram(game);
            if LoadFromSram(word!(game.selected_save_slot), game) != 0 {
                NewSaveFile(game);
                word!(game.has_area_map = 0);
            } else {
                LoadMirrorOfExploredMapTiles(game);
            }
        } else {
            if word!(game.selected_save_slot) == 3 {
                QueueSfx1_Max6(0x37, game);
                word!(game.menu_index += 1);
                *(ram_ptr!(game, u16; 0x57)) = (*(ram_ptr!(game, u16; 0x57)) & 0xff0c | 3);
            } else {
                if word!(game.selected_save_slot) == 4 {
                    QueueSfx1_Max6(0x37, game);
                    word!(game.menu_index += 15);
                    v0 = (*(ram_ptr!(game, u16; 0x57)) as i32 & 0xff0c | 3) as i16;
                    *(ram_ptr!(game, u16; 0x57)) = v0 as u16;
                }
                if v0 == 5 {
                    word!(game.menu_index = 33);
                }
            }
            return;
        }
    }
    let mut v3: i32 = ((4u16).wrapping_mul(word!(game.selected_save_slot))) as i32 >> 1;
    word!(game.eproj_id[10] = kMenuSelectionMissileXY[v3 as usize]);
    word!(game.eproj_id[5] = kMenuSelectionMissileXY[(v3 + 1) as usize]);
}
pub unsafe fn FileSelectMap_0(game: &mut Game) {
    ClearMenuTilemap(game);
    let v0: u16 = word!(game.vram_write_queue_tail);
    let mut v1 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
    (*v1).size = 2048;
    (*v1).src.addr = (&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).menu.menu_tilemap as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as VoidP;
    *(&raw mut (*v1).src.bank as *mut u16) = 126;
    (*v1).vram_dst = ((*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8) as u16;
    word! {game; vram_write_queue_tail = (v0).wrapping_add(7); palette_change_num = 0; }
    LoadFileSelectPalettes(game);
    word_transfers!(game, ascending, 32; le(target_palettes, 0) <- le(palette_buffer, 0));
    word! { game, 0; target_palettes[14], target_palettes[30] }
    word!(game.menu_index += 1);
}
pub unsafe fn FileSelectMap_1(game: &mut Game) {
    if AdvanceGradualColorChangeOfPalette(0, 0x40, game) != 0 {
        word! { game, 0; reg_BG1VOFS, reg_BG1HOFS, reg_BG2VOFS, reg_BG2HOFS }
        let mut v0: u16 = 0;
        while word!(game.area_index) != kFileSelectMap_AreaIndexes[(v0 >> 1) as usize] {
            v0 = (v0).wrapping_add(2);
            if (((v0) as i16).wrapping_sub(12) < 0) {
                continue;
            }
            v0 = 0;
            break;
        }
        word!(game.file_select_map_area_index = (v0 >> 1));
        let mut v1: i16 = 0;
        do_until! { ((v2).wrapping_sub(5) >= 0);
            let mut v2: i16 = v1;
            if v1 as i32 == word!(game.area_index) as i32 {
                LoadActiveAreaMapForegroundColors(v1 as u16, game);
            } else {
                LoadInactiveAreaMapForegroundColors(v1 as u16, game);
            }
            v1 += 1;
        }
        word!(game.menu_index += 1);
        word!(game.g_word_7E0787 = 0);
        *ram_ptr!(game; 0x69) = 2;
    }
}
static kActiveAreaMapForegroundColors: [u16; 6] = [0, 0xa, 0x10, 0x16, 0x24, 0x2a];
static kInactiveAreaMapForegroundColors: [u16; 6] = [0x30, 0x3a, 0x40, 0x46, 0x54, 0x5a];
crate::forward_calls! { pub unsafe fn (v0: u16, game: &mut Game);
    LoadActiveAreaMapForegroundColors =>
        LoadAreaMapForegroundColors(kActiveAreaMapForegroundColors[v0 as usize], game);
    LoadInactiveAreaMapForegroundColors =>
        LoadAreaMapForegroundColors(kInactiveAreaMapForegroundColors[v0 as usize], game);
}
pub unsafe fn LoadAreaMapForegroundColors(mut j: u16, game: &mut Game) {
    loop {
        let v1: i32 = j as i32 >> 1;
        if crate::rtl::rom_table_word!(game, 0x81a4e6, v1 as isize) == 0xffff {
            break;
        }
        let v4: u16 = j;
        let mut v2: u16 = crate::rtl::rom_table_word!(game, 0x81a4e6, (v1 + 1) as isize);
        let mut v3: u16 = crate::rtl::rom_table_word!(game, 0x81a4e6, v1 as isize);
        for _ in 0..5 {
            word!(
                game.palette_buffer[(v2 >> 1)] =
                    crate::rtl::rom_table_word!(game, 0x81a40e, (v3 >> 1) as isize)
            );
            v2 = (v2).wrapping_add(2);
            v3 = (v3).wrapping_add(2);
        }
        j = (v4).wrapping_add(4);
    }
}
pub unsafe fn FileSelectMap_2_LoadAreaSelectForegroundTilemap(game: &mut Game) {
    let v0: u16 = word!(game.vram_write_queue_tail);
    let mut v1 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
    (*v1).size = 2048;
    (*v1).src.addr = addr_kAreaSelectForegroundTilemap as i32 as VoidP;
    *(&raw mut (*v1).src.bank as *mut u16) = 129;
    (*v1).vram_dst = ((*ram_ptr!(game; 0x58) as i32 & 0xfc) << 8) as u16;
    word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
    *(ram_ptr!(game, u16; 0x51)) = ((*ram_ptr!(game; 0x52) as i32) << 8 | 0xf) as u16;
    word!(game.menu_index += 1);
}
pub unsafe fn FileSelectMap_18(game: &mut Game) {
    word!(game.menu_index += 1);
    LoadAreaSelectBackgroundTilemap(word!(game.area_index), game);
}
pub use self::FileSelectMap_18 as FileSelectMap_3_LoadAreaSelectBackgroundTilemap;
pub unsafe fn LoadAreaSelectBackgroundTilemap(j: u16, game: &mut Game) {
    let v1: u16 = word!(game.vram_write_queue_tail);
    let v2 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
    (*v2).size = 2048;
    (*v2).src.addr = (8 * 256 * j as i32 + addr_kAreaSelectBackgroundTilemaps as i32) as VoidP;
    (*v2).src.bank = 0x81;
    (*v2).vram_dst = ((*ram_ptr!(game; 0x5a) as i32 & 0xfc) << 8) as u16;
    word!(game.vram_write_queue_tail = (v1).wrapping_add(7));
}
pub unsafe fn FileSelectMap_4_SetupExpandingSquareTransition(game: &mut Game) {
    *ram_ptr!(game; 0x6b) = 4;
    crate::rtl::register_writes! { game;
        mirror(TS as u16, 4, 0x6d, false); byte(TSW as u16, 4);
        mirror(CGWSEL as u16, 2, 0x6f, true); mirror(CGADSUB as u16, 0x25, 0x72, true);
    }
    *ram_ptr!(game; 0x74, 2) = 0x80;
    *ram_ptr!(game; 0x74, 1) = 64;
    *ram_ptr!(game; 0x74, 0) = 32;
    SetupInitialExpandingSquareHDMA(game);
    crate::rtl::write_register_and_mirror(HDMAEN as u16, 0xc, 0x85, true, game);
    ConfigureWindow1ForExpandingSquare(game);
    word!(game.hdma_objects_enable_flag = 0);
    byte!(@pointer ram_ptr!(game, u16; 0x727); 0) =
        ((word!(game.menu_index)) as u8).wrapping_add(1);
}
pub unsafe fn ConfigureWindow1ForExpandingSquare(game: &mut Game) {
    *ram_ptr!(game; 0x69) = 19;
    crate::rtl::register_writes! { game;
        mirror(TM as u16, 0x13, 0x6c, false); byte(TMW as u16, 0x13);
        mirror(W12SEL as u16, 0x23, 0x60, true); mirror(W34SEL as u16, 3, 0x61, true);
        mirror(WOBJSEL as u16, 0x23, 0x62, true);
    }
}
pub unsafe fn SetupInitialExpandingSquareHDMA(game: &mut Game) {
    word!(game.expand_sq_topbottom_margin_right_pos = 0);
    word!(game.expand_sq_topbottom_margin_left_pos = -1i32 as u16);
    byte!(@pointer ram_ptr!(game, u16; 0x9e32); 0) = 127;
    byte!(@pointer ram_ptr!(game, u16; 0x9e36); 0) = -127i32 as u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0 = 111;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_1 = 34;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_2 = -98i32 as u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_0 = 1;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_1 = 50;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_2 = -98i32 as u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(2)).field_0 = 1;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(2)).field_1 = 50;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(2)).field_2 = -98i32 as u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(3)).field_0 = 111;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(3)).field_1 = 32;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(3)).field_2 = -98i32 as u8;
    crate::rtl::register_writes! { game;
        byte(DMAP2 as u16, 0x40); byte(BBAD2 as u16, 0x26);
        byte(A1T2L as u16, 0); byte(A1T2H as u16, 0x9e);
        byte(A1B2 as u16, 0x7e); byte(DAS20 as u16, 0x7e);
        byte(DAS2L as u16, 0); byte(DAS2H as u16, 0);
        byte(A2A2L as u16, 0); byte(A2A2H as u16, 0);
        byte(NTRL2 as u16, 0);
    }
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10))).field_0 = 111;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10))).field_1 = 32;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10))).field_2 = -98i32 as u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(1)).field_0 = 1;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(1)).field_1 = 54;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(1)).field_2 = -98i32 as u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(2)).field_0 = 1;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(2)).field_1 = 54;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(2)).field_2 = -98i32 as u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(3)).field_0 = 111;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(3)).field_1 = 32;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(3)).field_2 = -98i32 as u8;
    crate::rtl::register_writes! { game;
        byte(DMAP3 as u16, 0x40); byte(BBAD3 as u16, 0x27);
        byte(A1T3L as u16, 0x10); byte(A1T3H as u16, 0x9e);
        byte(A1B3 as u16, 0x7e); byte(DAS30 as u16, 0x7e);
        byte(DAS3L as u16, 0); byte(DAS3H as u16, 0);
        byte(A2A3L as u16, 0); byte(A2A3H as u16, 0);
        byte(NTRL3 as u16, 0);
    }
}
static kExpandingSquareTransitionSpeed: u8 = 4;
pub unsafe fn FileSelectMap_5_ExpandingSquare(game: &mut Game) {
    DrawAreaSelectMapLabels(game);
    let v0: u8 = ((*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0)
        .wrapping_sub(kExpandingSquareTransitionSpeed);
    if (((*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0) as i8)
        .wrapping_sub((kExpandingSquareTransitionSpeed) as i8)
        < 0
    {
        byte!(@pointer ram_ptr!(game, u16; 0x727); 0) =
            ((word!(game.menu_index)) as u8).wrapping_add(1);
        crate::update_place! { reference fresh36 = *ram_ptr!(game; 0x69) => (*fresh36 & !(2)); }
        *ram_ptr!(game; 0x6c) = 0;
        *ram_ptr!(game; 0x6d) = 0;
        word! {game; reg_BG2VOFS = 24; pausemenu_palette_animation_timer = 1; }
        let mut v1: u16 = 0;
        word_transfers!(game, ascending, 800; xray(0) <- rom(0xb6e000, 0));
        v1 = 800;
        word_transfers!(game, ascending, 224; xray(800) <- value(10241));
        v1 = 1024;
        DrawRoomSelectMapAreaLabel(
            (&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).bg2_room_select_map_tilemap as *mut u16)
                .offset(170),
            game,
        );
        word_transfers!(game, descending, 160; xray(959) <- rom(0x81b14b, 160));
        let v4: u16 = word!(game.vram_write_queue_tail);
        let mut v5 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v5).size = 2048;
        (*v5).src.addr = (ram_ptr!(game, Ram4000; 0x4000) as *mut u8)
            .offset_from(&raw mut game.g_ram as *mut u8) as c_long
            as VoidP;
        (*v5).src.bank = 126;
        (*v5).vram_dst = ((*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8) as u16;
        word!(game.vram_write_queue_tail = (v4).wrapping_add(7));
    } else {
        crate::update_place! { reference fresh39 = (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0 => (*fresh39).wrapping_sub(kExpandingSquareTransitionSpeed); }
        (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(3)).field_0 = v0;
        (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10))).field_0 = v0;
        (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(3)).field_0 = v0;
        crate::update_place! { reference fresh40 = (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_0 => (*fresh40).wrapping_add(kExpandingSquareTransitionSpeed); }
        (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(2)).field_0 =
            (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_0;
        (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(1)).field_0 =
            (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_0;
        (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(2)).field_0 =
            (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_0;
        byte!(@pointer ram_ptr!(game, u16; 0x9e32); 0) =
            ((word!(game.expand_sq_left_pos)) as u8).wrapping_sub(kExpandingSquareTransitionSpeed);
        byte!(@pointer ram_ptr!(game, u16; 0x9e36); 0) =
            (kExpandingSquareTransitionSpeed).wrapping_add((word!(game.expand_sq_right_pos)) as u8);
    };
}
pub unsafe fn FileSelectMap_6_AreaSelectMap(game: &mut Game) {
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
pub unsafe fn DecAndWraparoundTo5(a: u16) -> u16 {
    let result: u16 = (a).wrapping_sub(1);
    if (result & 0x8000) != 0 {
        return 5;
    }
    result
}
pub unsafe fn WraparoundFrom6to0(a: u16) -> u16 {
    let result: u16 = (a).wrapping_add(1);
    if result as i32 - 6i32 & 0x8000 == 0 {
        return 0;
    }
    result
}
pub unsafe fn SelectFileSelectMapArea(game: &mut Game) {
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
    let mut q: u16 = *(ram_ptr!(game, u16; 0xd8f8, (2 * word!(game.area_index) as i32) as isize));
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
        q = (q >> 1);
        if (v2 != 0) {
            let mut v3: i16 = *r0.offset((2 * v0 as i32) as isize) as i16;
            if (v3 != -2i16) {
                if v3 != -1i16 {
                    break;
                }
                v0 = -1i32 as u16;
            }
        }
        v0 = v0.wrapping_add(1);
        r20 = r20.wrapping_sub(1);
        if (r20 != 0) {
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
pub unsafe fn CheckIfFileSelectMapAreaCanBeSelected(a: u16, game: &mut Game) -> u16 {
    let v1: i32 = 2 * kFileSelectMap_AreaIndexes[a as usize] as i32;
    if *(ram_ptr!(game, u16; 0xd8f8, v1 as isize)) != 0 {
        return true_0 as u16;
    }
    let t: u16 = (*((&raw mut (*(rom_ptr!(game, MapIconDataPointers; 0x82c7cb)).offset(4)).crateria
        as *mut u8)
        .offset(v1 as isize) as *mut u16))
        .wrapping_add(64);
    (t != 0xffff) as u16
}
pub unsafe fn SwitchActiveFileSelectMapArea(R28: u16, game: &mut Game) {
    LoadInactiveAreaMapForegroundColors(
        kFileSelectMap_AreaIndexes[word!(game.file_select_map_area_index) as usize],
        game,
    );
    word!(game.file_select_map_area_index = R28);
    LoadActiveAreaMapForegroundColors(kFileSelectMap_AreaIndexes[R28 as usize], game);
    LoadAreaSelectBackgroundTilemap(
        kFileSelectMap_AreaIndexes[word!(game.file_select_map_area_index) as usize],
        game,
    );
}
static kAreaSelectMapLabelPositions: [u16; 12] = [
    0x5b, 0x32, 0x2a, 0x7f, 0x5e, 0xb5, 0xce, 0x50, 0xce, 0x9f, 0x87, 0x8b,
];
pub unsafe fn DrawAreaSelectMapLabels(game: &mut Game) {
    let mut r3: u16 = 0;
    DrawMenuSpritemap(
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
            r36 = (r36 >> 1);
            if !(v4 != 0 && *v2 != 0xfffe) {
                v2 = v2.offset(2);
                R30 -= 1;
                if (R30 != 0) {
                    continue;
                }
                if !(word!(game.enable_debug) != 0 && *v2 != 0xffff) {
                    break;
                }
            }
            let j: i32 = 4 * kFileSelectMap_AreaIndexes[i as usize] as i32 >> 1;
            DrawMenuSpritemap(
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
static kRoomSelectMapExpandingSquareTimers: [u16; 6] = [0x33, 0x35, 0x2d, 0x33, 0x33, 0x22];
pub unsafe fn FileSelectMap_7_PrepExpandSquareTransToRoomMap(game: &mut Game) {
    DrawAreaSelectMapLabels(game);
    SetupInitialExpandingSquareHDMA(game);
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
    let v0: u16 = word!(game.vram_write_queue_tail);
    let v1 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
    (*v1).size = 512;
    (*v1).src.addr = (ram_ptr!(game, Ram4000; 0x4000) as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as VoidP;
    (*v1).src.bank = 126;
    (*v1).vram_dst = ((*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8) as u16;
    word! {game; vram_write_queue_tail = (v0).wrapping_add(7);
    expand_sq_timer = kRoomSelectMapExpandingSquareTimers[word!(game.area_index) as usize]; }
    let v2: i32 = ((4u16).wrapping_mul(word!(game.area_index))) as i32 >> 1;
    word!(game.expand_sq_left_pos = kAreaSelectMapLabelPositions[v2 as usize]);
    word!(game.expand_sq_right_pos = word!(game.expand_sq_left_pos));
    word!(game.expand_sq_top_pos = kAreaSelectMapLabelPositions[(v2 + 1) as usize]);
    word!(game.expand_sq_bottom_pos = word!(game.expand_sq_top_pos));
    word! { game, 0;
        expand_sq_left_subpos, expand_sq_right_subpos, expand_sq_top_subpos,
        expand_sq_bottom_subpos,
    }
    let vels: *const ExpandingSquareVels =
        (rom_ptr!(game, ExpandingSquareVels; 0x81aa34)).offset(word!(game.area_index) as isize);
    word! {game; expand_sq_left_subvel = (*vels).left_subvel;
    expand_sq_left_vel = (*vels).left_vel; expand_sq_right_subvel = (*vels).right_subvel;
    expand_sq_right_vel = (*vels).right_vel; expand_sq_top_subvel = (*vels).top_subvel;
    expand_sq_top_vel = (*vels).top_vel; expand_sq_bottom_subvel = (*vels).bottom_subvel;
    expand_sq_bottom_vel = (*vels).bottom_vel; }
    SetupRoomSelectMapExpandingSquareTransHDMA(game);
    crate::rtl::write_register_and_mirror(HDMAEN as u16, 0xc, 0x85, true, game);
    word!(game.menu_index += 1);
    QueueSfx1_Max6(0x3b, game);
}
pub unsafe fn SetupRoomSelectMapExpandingSquareTransHDMA(game: &mut Game) {
    let mut k: u16 = 0;
    let v0: u16 = word!(game.expand_sq_top_pos);
    AddExpandingSqTransLeftIndirHDMA(v0, k, 0x9e22, game);
    k = AddExpandingSqTransRightIndirHDMA(v0, k, 0x9e20, game);
    let mut v2: u16 =
        (word!(game.expand_sq_bottom_pos)).wrapping_sub(word!(game.expand_sq_top_pos));
    if (word!(game.expand_sq_bottom_pos) as u8) == (word!(game.expand_sq_top_pos) as u8) {
        v2 = 1;
    }
    AddExpandingSqTransLeftIndirHDMA(v2, k, 0x9e32, game);
    k = AddExpandingSqTransRightIndirHDMA(v2, k, 0x9e36, game);
    let mut v4: u16 = (-32i32 - word!(game.expand_sq_bottom_pos) as i32) as u16;
    if (word!(game.expand_sq_bottom_pos) as u8) == 0xe0 {
        v4 = 1;
    }
    AddExpandingSqTransLeftIndirHDMA(v4, k, 0x9e22, game);
    k = AddExpandingSqTransRightIndirHDMA(v4, k, 0x9e20, game);
    *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0)
        .offset(k as i32 as isize) = 0;
    *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10))).field_0)
        .offset(k as i32 as isize) = 0;
}
pub unsafe fn AddExpandingSqTransLeftIndirHDMA(a: u16, k: u16, j: u16, game: &mut Game) {
    if (a & 0x80) != 0 {
        *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0)
            .offset(k as i32 as isize) = ((a) as u8).wrapping_sub(127);
        *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_0)
            .offset(k as i32 as isize) = 127;
        *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_1)
            .offset(k as i32 as isize) as *mut u16) = j;
        *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_1)
            .offset(k as i32 as isize) as *mut u16) = j;
    } else {
        *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0)
            .offset(k as i32 as isize) = a as u8;
        *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_1)
            .offset(k as i32 as isize) as *mut u16) = j;
    };
}
pub unsafe fn AddExpandingSqTransRightIndirHDMA(a: u16, k: u16, j: u16, game: &mut Game) -> u16 {
    if (a & 0x80) != 0 {
        *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10))).field_0)
            .offset(k as i32 as isize) = ((a) as u8).wrapping_sub(127);
        *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(1)).field_0)
            .offset(k as i32 as isize) = 127;
        *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10))).field_1)
            .offset(k as i32 as isize) as *mut u16) = j;
        *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(1)).field_1)
            .offset(k as i32 as isize) as *mut u16) = j;
        (k).wrapping_add(6)
    } else {
        *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10))).field_0)
            .offset(k as i32 as isize) = a as u8;
        *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10))).field_1)
            .offset(k as i32 as isize) as *mut u16) = j;
        (k).wrapping_add(3)
    }
}
pub unsafe fn FileSelectMap_8_ExpandSquareTransToRoomSelectMap(game: &mut Game) {
    if (HandleRoomSelectMapExpandingSquareTrans(game) & 0x8000) != 0 {
        word!(game.menu_index += 1);
        *ram_ptr!(game; 0x69) = 2;
        *ram_ptr!(game; 0x6c) = 0;
        *ram_ptr!(game; 0x6d) = 0;
        *ram_ptr!(game; 0x6b) = 0;
    }
    DrawAreaSelectMapLabels(game);
}
pub unsafe fn HandleRoomSelectMapExpandingSquareTrans(game: &mut Game) -> u16 {
    words!(
        game.expand_sq_left_pos,
        expand_sq_left_subpos += words!(game.expand_sq_left_vel, expand_sq_left_subvel)
    );
    if word!(game.expand_sq_left_pos) as i32 - 1i32 & 0x8000 != 0 {
        word!(game.expand_sq_left_pos = 1);
    }
    words!(
        game.expand_sq_right_pos,
        expand_sq_right_subpos += words!(game.expand_sq_right_vel, expand_sq_right_subvel)
    );
    if word!(game.expand_sq_right_pos) as i32 - 256i32 & 0x8000 == 0 {
        word!(game.expand_sq_right_pos = 255);
    }
    words!(
        game.expand_sq_top_pos,
        expand_sq_top_subpos += words!(game.expand_sq_top_vel, expand_sq_top_subvel)
    );
    if word!(game.expand_sq_top_pos) as i32 - 1i32 & 0x8000 != 0 {
        word!(game.expand_sq_top_pos = 1);
    }
    words!(
        game.expand_sq_bottom_pos,
        expand_sq_bottom_subpos += words!(game.expand_sq_bottom_vel, expand_sq_bottom_subvel)
    );
    if word!(game.expand_sq_bottom_pos) as i32 - 224i32 & 0x8000 == 0 {
        word!(game.expand_sq_bottom_pos = 224);
    }
    SetupRoomSelectMapExpandingSquareTransHDMA(game);
    word!(game.expand_sq_timer -= 1, reread)
}
pub unsafe fn FileSelectMap_9_InitRoomSelectMap(game: &mut Game) {
    LoadMirrorOfExploredMapTiles(game);
    DrawRoomSelectMap(game);
    LoadFromLoadStation(game);
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    let RoomDefHeader = get_RoomDefHeader(word!(game.room_ptr), game);
    byte!(@pointer ram_ptr!(game, u16; 0x79f); 0) = (*RoomDefHeader).area_index_;
    byte!(@pointer ram_ptr!(game, u16; 0x7a1); 0) = (*RoomDefHeader).x_coordinate_on_map;
    byte!(@pointer ram_ptr!(game, u16; 0x7a3); 0) = (*RoomDefHeader).y_coordinate_on_map;
    SetupMapScrollingForFileSelectMap(game);
    word!(game.map_min_y_scroll += 24);
    word!(game.reg_BG2VOFS = 24);
    crate::update_place! { unaligned_word fresh31 = &raw mut (*(ram_ptr!(game, u16; 0x69))) => (u16::from_le(fresh31.read_unaligned()) & !(4)); }
    word!(game.menu_index += 1);
    word! { game, 0;
        map_scrolling_direction, map_scrolling_speed_index,
        samus_position_indicator_animation_frame, samus_position_indicator_animation_timer,
        samus_position_indicator_animation_loop_counter,
    }
}
pub unsafe fn FileSelectMap_10_RoomSelectMap(game: &mut Game) {
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
        do_until! { (v1 < 0);
            r24 = (r24 >> 1);
            v1 -= 1;
        }
        let r0 = RomPtr_82(
            *(RomPtr_82(
                (addr_kMapIconDataPointers as i32 + 64 + 2 * word!(game.area_index) as i32) as u16,
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
            r24 = (r24 >> 1);
            if v4 == 0 || *r0.offset((2 * word!(game.load_station_index) as i32) as isize) >= 0xfffe
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
        word!(game.file_select_map_area_index = (v0 >> 1));
        QueueSfx1_Max6(0x3c, game);
    } else if word!(game.joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0 {
        word!(game.menu_index += 1);
        QueueSfx1_Max6(0x38, game);
    }
}
pub unsafe fn HandleFileSelectMapScrollArrows(game: &mut Game) {
    if word!(game.map_min_x_scroll) as i32 - 24i32 - word!(game.reg_BG1HOFS) as i32 & 0x8000 != 0 {
        DrawMapScrollArrowAndCheckToScroll(0x81, addr_kLeftMapScrollArrowData as u16, game);
    }
    if word!(game.map_max_x_scroll) as i32 - 232i32 - word!(game.reg_BG1HOFS) as i32 & 0x8000 == 0 {
        DrawMapScrollArrowAndCheckToScroll(0x81, addr_kRightMapScrollArrowData as u16, game);
    }
    if word!(game.map_min_y_scroll) as i32 - 64i32 - word!(game.reg_BG1VOFS) as i32 & 0x8000 != 0 {
        DrawMapScrollArrowAndCheckToScroll(0x81, addr_kUpMapScrollArrowData as u16, game);
    }
    if word!(game.map_max_y_scroll) as i32 - 145i32 - word!(game.reg_BG1VOFS) as i32 & 0x8000 != 0 {
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
pub unsafe fn FileSelectMap_11(game: &mut Game) {
    DrawFileSelectMapIcons(game);
    DisplayMapElevatorDestinations(game);
    word!(game.menu_index += 1);
}
pub unsafe fn FileSelectMap_13(game: &mut Game) {
    DrawFileSelectMapIcons(game);
    DisplayMapElevatorDestinations(game);
    HandleFadeOut(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
        word!(game.menu_index += 1);
        (*(ram_ptr!(game, EnemyData; 0xf78))).x_pos = 32;
    }
}
pub unsafe fn FileSelectMap_14(game: &mut Game) {
    let fresh22 = &raw mut (*(ram_ptr!(game, EnemyData; 0xf78))).x_pos;
    fresh22.write_unaligned(fresh22.read_unaligned().wrapping_sub(1));
    if fresh22.read_unaligned() == 0 {
        ScreenOff(game);
        word!(game.game_state += 1);
        word!(game.menu_index = 0);
    }
}
pub unsafe fn FileSelectMap_15_ClearTileMap(game: &mut Game) {
    *ram_ptr!(game; 0x69) = 18;
    word_transfers!(game, descending, 1024; native(0x37fe) <- value(15));
    let v1: u16 = word!(game.vram_write_queue_tail);
    let v2 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
    (*v2).size = 2048;
    (*v2).src.addr = (&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap
        as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut u16) = 126;
    (*v2).vram_dst = ((*ram_ptr!(game; 0x58) as i32 & 0xfc) << 8) as u16;
    word!(game.vram_write_queue_tail = (v1).wrapping_add(7));
    word!(game.menu_index += 1);
}
pub unsafe fn FileSelectMap_16_LoadPalettes(game: &mut Game) {
    LoadMenuPalettes(game);
    let mut v0: i16 = 0;
    do_until! { ((v1).wrapping_sub(5) >= 0);
        let mut v1: i16 = v0;
        crate::update_place! { previous fresh19 = v0 => v0 + 1; }
        LoadInactiveAreaMapForegroundColors(fresh19 as u16, game);
    }
    LoadActiveAreaMapForegroundColors(word!(game.area_index), game);
    LoadAreaSelectBackgroundTilemap(word!(game.area_index), game);
    word!(game.menu_index += 1);
}
pub unsafe fn FileSelectMap_20_SetupExpandingSquare(game: &mut Game) {
    *ram_ptr!(game; 0x85) = 0;
    QueueSfx1_Max6(0x3c, game);
    word! { game;
        expand_sq_timer =
            (kRoomSelectMapExpandingSquareTimers[word!(game.area_index) as usize]).wrapping_sub(12);
        expand_sq_left_subvel = *((&raw mut (*(rom_ptr!(game, ExpandingSquareVels; 0x81aa34)))
            .left_subvel as *mut u8)
            .offset(((16u16).wrapping_mul(word!(game.area_index))) as i32 as isize)
            as *mut u16);
        expand_sq_left_vel = *((&raw mut (*(rom_ptr!(game, ExpandingSquareVels; 0x81aa34))).left_vel
            as *mut u8)
            .offset(((16u16).wrapping_mul(word!(game.area_index))) as i32 as isize)
            as *mut u16);
        expand_sq_right_subvel = *((&raw mut (*(rom_ptr!(game, ExpandingSquareVels; 0x81aa34)))
            .right_subvel as *mut u8)
            .offset(((16u16).wrapping_mul(word!(game.area_index))) as i32 as isize)
            as *mut u16);
        expand_sq_right_vel = *((&raw mut (*(rom_ptr!(game, ExpandingSquareVels; 0x81aa34)))
            .right_vel as *mut u8)
            .offset(((16u16).wrapping_mul(word!(game.area_index))) as i32 as isize)
            as *mut u16);
        expand_sq_top_subvel = *((&raw mut (*(rom_ptr!(game, ExpandingSquareVels; 0x81aa34)))
            .top_subvel as *mut u8)
            .offset(((16u16).wrapping_mul(word!(game.area_index))) as i32 as isize)
            as *mut u16);
        expand_sq_top_vel = *((&raw mut (*(rom_ptr!(game, ExpandingSquareVels; 0x81aa34))).top_vel
            as *mut u8)
            .offset(((16u16).wrapping_mul(word!(game.area_index))) as i32 as isize)
            as *mut u16);
        expand_sq_bottom_subvel = *((&raw mut (*(rom_ptr!(game, ExpandingSquareVels; 0x81aa34)))
            .bottom_subvel as *mut u8)
            .offset(((16u16).wrapping_mul(word!(game.area_index))) as i32 as isize)
            as *mut u16);
        expand_sq_bottom_vel = *((&raw mut (*(rom_ptr!(game, ExpandingSquareVels; 0x81aa34)))
            .bottom_vel as *mut u8)
            .offset(((16u16).wrapping_mul(word!(game.area_index))) as i32 as isize)
            as *mut u16);
        expand_sq_left_pos = 8;
        expand_sq_right_pos = 248;
        expand_sq_top_pos = 8;
        expand_sq_bottom_pos = 216;
    }
    word! { game, 0;
        expand_sq_left_subpos, expand_sq_right_subpos, expand_sq_top_subpos,
        expand_sq_bottom_subpos,
    }
    SetupRoomSelectMapExpandingSquareTransHDMA(game);
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
pub unsafe fn FileSelectMap_21_MoveExpandingSquare(game: &mut Game) {
    words!(
        game.expand_sq_left_pos,
        expand_sq_left_subpos +=
            -(words!(i32, game.expand_sq_left_vel, expand_sq_left_subvel)) as u32
    );
    words!(
        game.expand_sq_right_pos,
        expand_sq_right_subpos +=
            -(words!(i32, game.expand_sq_right_vel, expand_sq_right_subvel)) as u32
    );
    words!(
        game.expand_sq_top_pos,
        expand_sq_top_subpos += -(words!(i32, game.expand_sq_top_vel, expand_sq_top_subvel)) as u32
    );
    words!(
        game.expand_sq_bottom_pos,
        expand_sq_bottom_subpos +=
            -(words!(i32, game.expand_sq_bottom_vel, expand_sq_bottom_subvel)) as u32
    );
    SetupRoomSelectMapExpandingSquareTransHDMA(game);
    DrawAreaSelectMapLabels(game);
    if (word!(game.expand_sq_timer -= 1, reread) & 0x8000) != 0 {
        word!(game.menu_index -= 15);
        *ram_ptr!(game; 0x69) = 17;
        *ram_ptr!(game; 0x6c) = 0;
        *ram_ptr!(game; 0x6d) = 0;
    }
}
pub unsafe fn NewSaveFile(game: &mut Game) {
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
pub unsafe fn FileSelectClearRestOfMenuTilemapRow(mut v0: u16, game: &mut Game) {
    let mut n: i32 = 32 - ((v0 as i32 & 0x3f) >> 1);
    do_until! { (n == 0);
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(768) as *mut u8)
            .offset(v0 as i32 as isize) as *mut u16) = 15;
        v0 = (v0).wrapping_add(2);
        n -= 1;
    }
}
pub unsafe fn LoadMenuTilemap(mut k: u16, mut j: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    let mut k_bak: i32 = k as i32;
    loop {
        loop {
            v2 = *(RomPtr_81(j, game) as *mut u16) as i16;
            if v2 != -2i16 {
                break;
            }
            j = (j).wrapping_add(2);
            k_bak += 64;
            k = k_bak as u16;
        }
        if v2 == -1i16 {
            break;
        }
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(768) as *mut u8)
            .offset(k as i32 as isize) as *mut u16) =
            ((*(ram_ptr!(game, EnemyData; 0xf78))).palette_index) | ((v2) as u16);
        k = (k).wrapping_add(2);
        j = (j).wrapping_add(2);
    }
}
