use crate::c_library::{memcpy, memset};
use crate::game::callback_tables::callback_table;
pub use crate::game::constants::bank_82::*;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::game_states::*;
use crate::game::enemy_instruction;
use crate::game::sm_80::{
    HandleFadeIn, HandleFadeOut, HasQueuedMusic, LoadFromLoadStation, LoadMirrorOfExploredMapTiles,
    QueueMusic_Delayed8, QueueSfx1_Max6, QueueSfx3_Max6, ScreenOff, ScreenOn, SetupDmaTransfer,
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
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::set_ram_words;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::AddToHiLo;
use crate::rtl::{RomPtr, RtlWriteSram, Unreachable, WriteReg};
pub use crate::snes::registers::*;
pub use crate::types::Buttons;
pub use crate::types::{size_t, Func_V, LongPtr, OamEnt, SnesRegs, VoidP, VramWriteEntry};
pub use crate::types::{EnemyData, GameState, RoomDefHeader, SpawnHdmaObject_Args};
pub use crate::types::{ExpandingSquareTransitionHdma, MapIconDataPointers, StartDmaCopy};
use ::core::ffi::{c_long, c_void};
use ::core::ptr::null_mut;
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct ExpandingSquareVels {
        pub left_subvel, left_vel, right_subvel, right_vel, top_subvel, top_vel, bottom_subvel,
            bottom_vel: u16;
    }
}
pub use crate::types::{
    Consts_81, MapScrollArrowData, Ram3000, Ram3000_Menu, Ram3000_Misc, Ram3000_MsgBox, Ram4000,
    Ram4000_Backups,
};
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
pub use crate::types::Consts_82;
crate::game::constants::typed_constants! { i32;
    true_0 = 1,
}
const RomPtr_81: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x81>;
const RomPtr_82: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x82>;
const RomPtr_8C: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8c>;
const RomPtr_92: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x92>;
const RomPtr_93: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x93>;
use crate::rtl::rom_ptr_with_bank as RomPtrWithBank;
crate::ram_map::ram_pointer_functions! {
    gOamEnt(v: i32) -> OamEnt = [(0x370i32 + v) as isize];
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0i32 + a) as isize];
}
#[inline]
unsafe fn get_RoomDefHeader(a: u16, game: &mut Game) -> *mut RoomDefHeader {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomDefHeader
}
static kFileSelectMap_AreaIndexes: [u16; 6] = [0, 3, 5, 1, 4, 2];
pub unsafe fn SoftReset(game: &mut Game) {
    set_ram_word!(game.game_state = 0xffffu16);
}
pub unsafe fn SaveToSram(a: u16, game: &mut Game) {
    let mut r20: u16 = 0u16;
    let r18: u16 = (2i32 * (a as i32 & 3i32)) as u16;
    let mut i: i32 = 94i32;
    while i >= 0i32 {
        set_ram_word!(
            game.player_data_saved[(i >> 1i32)] =
                *((ram_ptr!(game, u16; 0x9a2) as *mut u8).offset(i as isize) as *mut u16)
        );
        i -= 2i32;
    }
    let mut v3: u16 = (ram_word!(game, area_index)).wrapping_mul(256u16);
    let mut v4: u16 = 0u16;
    loop {
        set_ram_word!(
            game.explored_map_tiles_saved[(v3 as i32 >> 1i32)] =
                *(ram_ptr!(game, u16; 0x7f7, v4 as isize))
        );
        v4 = (v4).wrapping_add(2u16);
        v3 = (v3).wrapping_add(2u16);
        if (((v4) as i16).wrapping_sub(256i16) >= 0i16) {
            break;
        }
    }
    PackMapToSave(game);
    set_ram_word!(game.sram_save_station_index = ram_word!(game, load_station_index));
    set_ram_word!(game.sram_area_index = ram_word!(game, area_index));
    let mut v5: u16 = crate::rtl::rom_table_word!(game, 0x81812b, (r18 as i32 >> 1i32) as isize);
    let mut v6: *mut u16 = ram_ptr!(game, u16; 0xd7c0);
    loop {
        let fresh94 = v6;
        v6 = v6.offset(1);
        let mut v7: u16 = *fresh94;
        *(game.g_sram.offset((2i32 * (v5 as i32 >> 1i32)) as isize) as *mut u16) = v7;
        r20 = (r20).wrapping_add(v7);
        v5 = (v5).wrapping_add(2u16);
        if (v6 == ram_ptr!(game, u16; 0xde1c)) {
            break;
        }
    }
    let v8: u16 = r18;
    let v9: u16 = r20;
    let v10: i32 = r18 as i32 >> 1i32;
    *(game.g_sram.offset((2i32 * v10) as isize) as *mut u16) = r20;
    *(game.g_sram.offset((2i32 * v10 + 0x1ff0i32) as isize) as *mut u16) = v9;
    let mut v11: u16 = !(v9 as i32) as u16;
    *(game.g_sram.offset((2i32 * v10 + 8i32) as isize) as *mut u16) = v11;
    *(game.g_sram.offset((2i32 * v10 + 0x1ff8i32) as isize) as *mut u16) = v11;
    RtlWriteSram(game);
}
pub unsafe fn LoadFromSram(a: u16, game: &mut Game) -> u8 {
    let mut r20: u16 = 0u16;
    let r18: u16 = (2i32 * (a as i32 & 3i32)) as u16;
    let mut v1: u16 = crate::rtl::rom_table_word!(game, 0x81812b, (a as i32 & 3i32) as isize);
    let mut v2: *mut u16 = ram_ptr!(game, u16; 0xd7c0);
    loop {
        let v3: u16 = *(game.g_sram.offset(0) as *mut u16).offset((v1 as i32 >> 1i32) as isize);
        let fresh1 = v2;
        v2 = v2.offset(1);
        *fresh1 = v3;
        r20 = (r20).wrapping_add(v3);
        v1 = (v1).wrapping_add(2u16);
        if (v2 == ram_ptr!(game, u16; 0xde1c)) {
            break;
        }
    }
    let v4: i32 = r18 as i32 >> 1i32;
    if r20 == *(game.g_sram.offset(0) as *mut u16).offset(v4 as isize)
        && r20 ^ 0xffff == *(game.g_sram.offset(0x8) as *mut u16).offset(v4 as isize)
        || r20 == *(game.g_sram.offset(0x1ff0) as *mut u16).offset(v4 as isize)
            && r20 ^ 0xffff == *(game.g_sram.offset(0x1ff8) as *mut u16).offset(v4 as isize)
    {
        let mut i: i32 = 94i32;
        while i >= 0i32 {
            *((ram_ptr!(game, u16; 0x9a2) as *mut u8).offset(i as isize) as *mut u16) =
                ram_word!(game, player_data_saved[(i >> 1i32)]);
            i -= 2i32;
        }
        UnpackMapFromSave(game);
        set_ram_word!(game.load_station_index = ram_word!(game, sram_save_station_index));
        set_ram_word!(game.area_index = ram_word!(game, sram_area_index));
        0u8
    } else {
        r20 = 0u16;
        let mut v7: u16 =
            crate::rtl::rom_table_word!(game, 0x81812b, (r18 as i32 >> 1i32) as isize);
        let mut v8: u16 = (ram_ptr!(game, u16; 0xd7c0) as *mut u8)
            .offset_from(&raw mut game.g_ram as *mut u8) as c_long as u16;
        let mut v9: u16 = 0u16;
        loop {
            *(game.g_sram.offset(0) as *mut u16).offset((v7 as i32 >> 1i32) as isize) = v9;
            v9 = (v9).wrapping_add(r20);
            r20 = v9;
            v7 = (v7).wrapping_add(2u16);
            v8 = (v8).wrapping_add(2u16);
            if (v8 as c_long
                == (ram_ptr!(game, u16; 0xde1c) as *mut u8)
                    .offset_from(&raw mut game.g_ram as *mut u8) as c_long)
            {
                break;
            }
        }
        set_ram_words! { game, 0u16; load_station_index, area_index }
        1u8
    }
}
pub unsafe fn UnpackMapFromSave(game: &mut Game) {
    let mut i: i32 = 1792i32;
    while i >= 0i32 {
        set_ram_word!(game.explored_map_tiles_saved[(i >> 1i32)] = 0u16);
        i -= 2i32;
    }
    let mut i_0: i32 = 0i32;
    while i_0 < 6i32 {
        let mut n: i32 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x818131)) as *mut u8)
            .offset(i_0 as isize) as i32;
        let mut r0 = RomPtr_81(
            crate::rtl::rom_table_word!(game, 0x8182d6, i_0 as isize),
            game,
        );
        let mut v3: i32 = crate::rtl::rom_table_word!(game, 0x818138, i_0 as isize) as i32;
        let unpacked = ram_ptr!(game, u16; 0xcd52) as *mut u8;
        loop {
            let fresh2 = r0;
            r0 = r0.offset(1);
            let v4: u16 = (i_0 * 256i32 + *fresh2 as i32) as u16;
            let fresh3 = v3;
            v3 = v3 + 1;
            *unpacked.offset(v4 as isize) = *ram_ptr!(game; 0xd91c, fresh3 as isize);
            n -= 1;
            if (n == 0) {
                break;
            }
        }
        i_0 += 1;
    }
}
pub unsafe fn PackMapToSave(game: &mut Game) {
    let mut i: i32 = 0i32;
    while i < 6i32 {
        let mut n: i32 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x818131)) as *mut u8)
            .offset(i as isize) as i32;
        let mut r0 = RomPtr_81(
            crate::rtl::rom_table_word!(game, 0x8182d6, i as isize),
            game,
        );
        let mut v1: i32 = crate::rtl::rom_table_word!(game, 0x818138, i as isize) as i32;
        let unpacked = ram_ptr!(game, u16; 0xcd52) as *mut u8;
        loop {
            let fresh95 = r0;
            r0 = r0.offset(1);
            let v3: i32 = i * 256i32 + *fresh95 as i32;
            let fresh96 = v1;
            v1 = v1 + 1;
            *ram_ptr!(game; 0xd91c, fresh96 as isize) = *unpacked.offset(v3 as isize);
            n -= 1;
            if (n == 0) {
                break;
            }
        }
        i += 1;
    }
}
pub unsafe fn DrawSpritemap(db: u8, j: u16, x_r20: u16, y_r18: u16, chr_r22: u16, game: &mut Game) {
    let mut pp = RomPtrWithBank(db, j, game);
    let mut n: i32 = *(pp as *mut u16) as i32;
    pp = pp.offset(2);
    let mut idx: i32 = ram_word!(game, oam_next_ptr) as i32;
    while n != 0i32 && idx < 0x200i32 {
        let oam = gOamEnt(idx, game);
        let mut x: i32 = x_r20 as i32 + *(pp as *mut u16) as i32;
        let mut y: i32 = y_r18 as u8 as i32 + *pp.offset(2) as i32;
        if if *pp.offset(2) & 0x80 == 0 {
            (y >= 0xe0i32) as i32
        } else if y & 0x100i32 != 0 {
            ((y as u8) >= 0xe0u8) as i32
        } else {
            ((y as u8) < 0xe0u8) as i32
        } != 0
        {
            x = 0x180i32;
            y = 0xe0i32;
        }
        (*oam).xcoord = x as u8;
        (*oam).ycoord = y as u8;
        *(&raw mut (*oam).charnum as *mut u16) = (chr_r22 | *(pp.offset(3) as *mut u16) & 0xf1ff);
        update_ram_word!(game.oam_ext[(idx >> 5i32)], |fresh9| (fresh9 as i32
            | ((x & 0x100i32) >> 8i32 | (*(pp as *mut i16) < 0i16) as i32 * 2i32)
                << 2i32 * (idx >> 2i32 & 7i32))
            as u16);
        idx += 4i32;
        pp = pp.offset(5);
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as u16);
}
pub unsafe fn DrawSpritemapOffScreen(
    j: u16,
    x_r20: u16,
    y_r18: u16,
    chr_r22: u16,
    game: &mut Game,
) {
    let mut pp = RomPtr_8C(j, game);
    let mut n: i32 = *(pp as *mut u16) as i32;
    pp = pp.offset(2);
    let mut idx: i32 = ram_word!(game, oam_next_ptr) as i32;
    while n != 0i32 && idx < 0x200i32 {
        let oam = gOamEnt(idx, game);
        let mut x: i32 = x_r20 as i32 + *(pp as *mut u16) as i32;
        let mut y: i32 = y_r18 as u8 as i32 + *pp.offset(2) as i32;
        if if *pp.offset(2) & 0x80 == 0 {
            (y < 0xe0i32) as i32
        } else if y & 0x100i32 != 0 {
            ((y as u8) < 0xe0u8) as i32
        } else {
            ((y as u8) >= 0xe0u8) as i32
        } != 0
        {
            x = 0x180i32;
            y = 0xe0i32;
        }
        (*oam).xcoord = x as u8;
        (*oam).ycoord = y as u8;
        *(&raw mut (*oam).charnum as *mut u16) = (chr_r22 | *(pp.offset(3) as *mut u16) & 0xf1ff);
        update_ram_word!(game.oam_ext[(idx >> 5i32)], |fresh10| (fresh10 as i32
            | ((x & 0x100i32) >> 8i32 | (*(pp as *mut i16) < 0i16) as i32 * 2i32)
                << 2i32 * (idx >> 2i32 & 7i32))
            as u16);
        idx += 4i32;
        pp = pp.offset(5);
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as u16);
}
pub unsafe fn DrawMenuSpritemap(a: u16, k: u16, j: u16, chr_r3: u16, game: &mut Game) {
    let mut pp = RomPtr_82(
        crate::rtl::rom_table_word!(game, 0x82c569, a as isize),
        game,
    );
    let mut n: i32 = *(pp as *mut u16) as i32;
    pp = pp.offset(2);
    let mut idx: i32 = ram_word!(game, oam_next_ptr) as i32;
    while n != 0i32 {
        let oam = gOamEnt(idx, game);
        let x: u16 = (k).wrapping_add(*(pp as *mut u16));
        (*oam).xcoord = x as u8;
        update_ram_word!(game.oam_ext[(idx >> 5i32)], |fresh4| (fresh4 as i32
            | ((x as i32 & 0x100i32) >> 8i32 | (*(pp as *mut i16) < 0i16) as i32 * 2i32)
                << 2i32 * (idx >> 2i32 & 7i32))
            as u16);
        (*oam).ycoord = ((j) as u8).wrapping_add(*pp.offset(2));
        *(&raw mut (*oam).charnum as *mut u16) = (chr_r3 | *(pp.offset(3) as *mut u16) & 0xf1ff);
        pp = pp.offset(5);
        idx = idx + 4i32 & 0x1ffi32;
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as u16);
}
pub unsafe fn DrawSamusSpritemap(a: u16, x_pos: u16, y_pos: u16, game: &mut Game) {
    if crate::rtl::rom_table_word!(game, 0x92808d, a as isize) == 0u16 {
        return;
    }
    let mut pp = RomPtr_92(
        crate::rtl::rom_table_word!(game, 0x92808d, a as isize),
        game,
    );
    let mut idx: i32 = ram_word!(game, oam_next_ptr) as i32;
    let mut n: i32 = *(pp as *mut u16) as i32;
    pp = pp.offset(2);
    while n != 0i32 {
        let x: u16 = (x_pos).wrapping_add(*(pp as *mut u16));
        let v9 = gOamEnt(idx, game);
        (*v9).xcoord = x as u8;
        update_ram_word!(game.oam_ext[(idx >> 5i32)], |fresh8| (fresh8 as i32
            | ((x as i32 & 0x100i32) >> 8i32 | (*(pp as *mut i16) < 0i16) as i32 * 2i32)
                << 2i32 * (idx >> 2i32 & 7i32))
            as u16);
        (*v9).ycoord = ((y_pos) as u8).wrapping_add(*pp.offset(2));
        *(&raw mut (*v9).charnum as *mut u16) = *(pp.offset(3) as *mut u16);
        pp = pp.offset(5);
        idx = idx + 4i32 & 0x1ffi32;
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as u16);
}
pub unsafe fn DrawBeamGrappleSpritemap(a: u16, x_r20: u16, y_r18: u16, game: &mut Game) {
    DrawGrappleOrProjectileSpritemap(
        RomPtr_93(
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
        RomPtr_93(
            ram_word!(game, projectile_spritemap_pointers[(k as i32 >> 1i32)]),
            game,
        ),
        x_r20,
        y_r18,
        game,
    );
}
pub unsafe fn DrawGrappleOrProjectileSpritemap(
    mut pp: *const u8,
    x_r20: u16,
    y_r18: u16,
    game: &mut Game,
) {
    let mut idx: i32 = ram_word!(game, oam_next_ptr) as i32;
    let mut n: i32 = *(pp as *mut u16) as i32;
    pp = pp.offset(2);
    while n != 0i32 {
        let x: u16 = (x_r20).wrapping_add(*(pp as *mut u16));
        let v4 = gOamEnt(idx, game);
        (*v4).xcoord = x as u8;
        update_ram_word!(game.oam_ext[(idx >> 5i32)], |fresh5| (fresh5 as i32
            | ((x as i32 & 0x100i32) >> 8i32 | (*(pp as *mut i16) < 0i16) as i32 * 2i32)
                << 2i32 * (idx >> 2i32 & 7i32))
            as u16);
        (*v4).ycoord = ((y_r18) as u8).wrapping_add(*pp.offset(2));
        *(&raw mut (*v4).charnum as *mut u16) = *(pp.offset(3) as *mut u16);
        pp = pp.offset(5);
        idx = idx + 4i32 & 0x1ffi32;
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as u16);
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
    if j == 0u16 {
        return;
    }
    let mut pp = RomPtrWithBank(db, j, game) as *mut u8;
    let mut idx: i32 = ram_word!(game, oam_next_ptr) as i32;
    let mut n: i32 = *(pp as *mut u16) as i32;
    pp = pp.offset(2);
    while n != 0i32 {
        let oam = gOamEnt(idx, game);
        let x: u16 = (r20_x).wrapping_add(*(pp.offset(0) as *mut u16));
        (*oam).xcoord = x as u8;
        update_ram_word!(game.oam_ext[(idx >> 5i32)], |fresh11| (fresh11 as i32
            | ((x as i32 & 0x100i32) >> 8i32 | (*(pp as *mut i16) < 0i16) as i32 * 2i32)
                << 2i32 * (idx >> 2i32 & 7i32))
            as u16);
        (*oam).ycoord = ((r18_y) as u8).wrapping_add(*pp.offset(2));
        *(&raw mut (*oam).charnum as *mut u16) =
            (r3 as i32 | r0 as i32 + *(pp.offset(3) as *mut u16) as i32) as u16;
        pp = pp.offset(5);
        idx = idx + 4i32 & 0x1ffi32;
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as u16);
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
    let mut pp = RomPtrWithBank(db, j, game);
    let mut idx: i32 = ram_word!(game, oam_next_ptr) as i32;
    let mut n: i32 = *(pp as *mut u16) as i32;
    pp = pp.offset(2);
    while n != 0i32 {
        let oam = gOamEnt(idx, game);
        let x: u16 = (r20_x).wrapping_add(*(pp as *mut u16));
        (*oam).xcoord = x as u8;
        update_ram_word!(game.oam_ext[(idx >> 5i32)], |fresh12| (fresh12 as i32
            | ((x as i32 & 0x100i32) >> 8i32 | (*(pp as *mut i16) < 0i16) as i32 * 2i32)
                << 2i32 * (idx >> 2i32 & 7i32))
            as u16);
        let y: i32 = *pp.offset(2) as i32 + r18_y as u8 as i32;
        (*oam).ycoord = (if (y & 0x100i32 == 0) as i32 == (*pp.offset(2) & 0x80 == 0) as i32 {
            y
        } else {
            0xf0i32
        }) as u8;
        *(&raw mut (*oam).charnum as *mut u16) =
            (r3 as i32 | r0 as i32 + *(pp.offset(3) as *mut u16) as i32) as u16;
        pp = pp.offset(5);
        idx = idx + 4i32 & 0x1ffi32;
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as u16);
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
    let mut pp = RomPtrWithBank(db, j, game);
    let mut idx: i32 = ram_word!(game, oam_next_ptr) as i32;
    let mut n: i32 = *(pp as *mut u16) as i32;
    pp = pp.offset(2);
    while n != 0i32 {
        let oam = gOamEnt(idx, game);
        let x: i32 = r20_x as i32 + *(pp as *mut u16) as i32;
        (*oam).xcoord = x as u8;
        update_ram_word!(game.oam_ext[(idx >> 5i32)], |fresh13| (fresh13 as i32
            | ((x & 0x100i32) >> 8i32 | (*(pp as *mut i16) < 0i16) as i32 * 2i32)
                << 2i32 * (idx >> 2i32 & 7i32))
            as u16);
        let y: i32 = *pp.offset(2) as i32 + r18_y as u8 as i32;
        (*oam).ycoord = (if (y & 0x100i32 == 0) as i32 != (*pp.offset(2) & 0x80 == 0) as i32 {
            y
        } else {
            0xf0i32
        }) as u8;
        *(&raw mut (*oam).charnum as *mut u16) =
            (r3 as i32 | r0 as i32 + *(pp.offset(3) as *mut u16) as i32) as u16;
        pp = pp.offset(5);
        idx = idx + 4i32 & 0x1ffi32;
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as u16);
}
pub unsafe fn DrawEprojSpritemapWithBaseTile(
    db: u8,
    j: u16,
    x_r20: u16,
    y_r18: u16,
    chr_r26: u16,
    chr_r28: u16,
    game: &mut Game,
) {
    let mut pp = RomPtrWithBank(db, j, game);
    let mut idx: i32 = ram_word!(game, oam_next_ptr) as i32;
    let mut n: i32 = *(pp as *mut u16) as i32;
    pp = pp.offset(2);
    while n != 0i32 {
        let oam = gOamEnt(idx, game);
        let x: u16 = (x_r20).wrapping_add(*(pp as *mut u16));
        (*oam).xcoord = x as u8;
        update_ram_word!(game.oam_ext[(idx >> 5i32)], |fresh6| (fresh6 as i32
            | ((x as i32 & 0x100i32) >> 8i32 | (*(pp as *mut i16) < 0i16) as i32 * 2i32)
                << 2i32 * (idx >> 2i32 & 7i32))
            as u16);
        let y: i32 = *pp.offset(2) as i32 + y_r18 as u8 as i32;
        (*oam).ycoord = (if (y & 0x100i32 == 0) as i32 == (*pp.offset(2) & 0x80 == 0) as i32 {
            y
        } else {
            0xf0i32
        }) as u8;
        *(&raw mut (*oam).charnum as *mut u16) =
            (chr_r28 as i32 | chr_r26 as i32 + *(pp.offset(3) as *mut u16) as i32) as u16;
        idx = idx + 4i32 & 0x1ffi32;
        pp = pp.offset(5);
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as u16);
}
pub unsafe fn DrawEprojSpritemapWithBaseTileOffscreen(
    db: u8,
    j: u16,
    x_r20: u16,
    y_r18: u16,
    chr_r26: u16,
    chr_r28: u16,
    game: &mut Game,
) {
    let mut pp = RomPtrWithBank(db, j, game);
    let mut idx: i32 = ram_word!(game, oam_next_ptr) as i32;
    let mut n: i32 = *(pp as *mut u16) as i32;
    pp = pp.offset(2);
    while n != 0i32 {
        let oam = gOamEnt(idx, game);
        let x: u16 = (x_r20).wrapping_add(*(pp as *mut u16));
        (*oam).xcoord = x as u8;
        update_ram_word!(game.oam_ext[(idx >> 5i32)], |fresh7| (fresh7 as i32
            | ((x as i32 & 0x100i32) >> 8i32 | (*(pp as *mut i16) < 0i16) as i32 * 2i32)
                << 2i32 * (idx >> 2i32 & 7i32))
            as u16);
        let y: i32 = *pp.offset(2) as i32 + y_r18 as u8 as i32;
        (*oam).ycoord = (if (y & 0x100i32 == 0) as i32 != (*pp.offset(2) & 0x80 == 0) as i32 {
            y
        } else {
            0xf0i32
        }) as u8;
        *(&raw mut (*oam).charnum as *mut u16) =
            (chr_r28 as i32 | chr_r26 as i32 + *(pp.offset(3) as *mut u16) as i32) as u16;
        idx = idx + 4i32 & 0x1ffi32;
        pp = pp.offset(5);
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as u16);
}
pub unsafe fn GameOverMenu_0_FadeOutConfigGfx(game: &mut Game) {
    HandleFadeOut(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
        ScreenOff(game);
        QueueSfx3_Max6(1u16, game);
        DisableHdmaObjects(game);
        WaitUntilEndOfVblankAndClearHdma(game);
        let mut v0: u16 = 0u16;
        loop {
            (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap
                [(v0 as i32 + 384i32) as usize] = ram_word!(game, palette_buffer[v0]);
            v0 = v0.wrapping_add(1);
            if (((v0 as i32 * 2i32 - 512i32) as i16) >= 0i16) {
                break;
            }
        }
        let mut v1: u16 = 0u16;
        loop {
            (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap
                [(v1 as i32 + 640i32) as usize] =
                *(ram_ptr!(game, u16; 0x51, (v1 as i32 * 2i32) as isize));
            v1 = v1.wrapping_add(1);
            if (((v1 as i32 * 2i32 - 54i32) as i16) >= 0i16) {
                break;
            }
        }
        MapVramForMenu(game);
        LoadInitialMenuTiles(game);
        set_ram_words! { game, 0u16;
            reg_BG1HOFS, reg_BG2HOFS, reg_BG3HOFS, reg_BG1VOFS, reg_BG2VOFS, reg_BG3VOFS,
        }
        LoadMenuPalettes(game);
        update_ram_word!(game.menu_index, wrapping_add(1));
    }
}
pub unsafe fn GameOverMenu_24_FadeIn(game: &mut Game) {
    HandleFadeIn(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 15 {
        update_ram_word!(game.menu_index, wrapping_add(1));
    }
}
pub unsafe fn MapVramForMenu(game: &mut Game) {
    *ram_ptr!(game; 0x6b) = 0u8;
    *ram_ptr!(game; 0x52) = 3u8;
    *ram_ptr!(game; 0x58) = 81u8;
    *ram_ptr!(game; 0x59) = 88u8;
    *ram_ptr!(game; 0x5a) = 92u8;
    *ram_ptr!(game; 0x5d) = 0u8;
    *ram_ptr!(game; 0x5e) = 4u8;
    *ram_ptr!(game; 0x69) = 19u8;
}
pub unsafe fn LoadInitialMenuTiles(game: &mut Game) {
    crate::rtl::register_writes! { game;
        byte(VMADDL as u16, 0u8); byte(VMADDH as u16, 0u8);
        byte(VMAIN as u16, 0x80u8);
    }
    static unk_818DF1: StartDmaCopy = StartDmaCopy::vram_words(1, 0x8e8000, 0x5600);
    SetupDmaTransfer(&raw const unk_818DF1 as *const c_void, game);
    crate::rtl::register_writes! { game;
        byte(MDMAEN as u16, 2u8); byte(VMADDL as u16, 0u8);
        byte(VMADDH as u16, 0x30u8); byte(VMAIN as u16, 0x80u8);
    }
    static unk_818E11: StartDmaCopy = StartDmaCopy::vram_words(1, 0xb68000, 0x2000);
    SetupDmaTransfer(&raw const unk_818E11 as *const c_void, game);
    crate::rtl::register_writes! { game;
        byte(MDMAEN as u16, 2u8); byte(VMADDL as u16, 0u8);
        byte(VMADDH as u16, 0x60u8); byte(VMAIN as u16, 0x80u8);
    }
    static unk_818E31: StartDmaCopy = StartDmaCopy::vram_words(1, 0xb6c000, 0x2000);
    SetupDmaTransfer(&raw const unk_818E31 as *const c_void, game);
    crate::rtl::register_writes! { game;
        byte(MDMAEN as u16, 2u8); byte(VMADDL as u16, 0u8);
        byte(VMADDH as u16, 0x40u8); byte(VMAIN as u16, 0x80u8);
    }
    static unk_818E51: StartDmaCopy = StartDmaCopy::vram_words(1, 0x8ed600, 0x600);
    SetupDmaTransfer(&raw const unk_818E51 as *const c_void, game);
    WriteReg(MDMAEN as u16, 2u8, game);
}
pub unsafe fn LoadMenuPalettes(game: &mut Game) {
    let mut v0: u16 = 0u16;
    loop {
        let v1: i32 = v0 as i32 >> 1i32;
        set_ram_word!(
            game.palette_buffer[v1] = crate::rtl::rom_table_word!(game, 0x8ee400, v1 as isize)
        );
        set_ram_word!(
            game.palette_buffer[(v1 + 1i32)] =
                crate::rtl::rom_table_word!(game, 0x8ee400, (v1 + 1i32) as isize)
        );
        v0 = (v0).wrapping_add(4u16);
        if (((v0) as i16).wrapping_sub(512i16) >= 0i16) {
            break;
        }
    }
}
pub unsafe fn GameOverMenu_3_Main(game: &mut Game) {
    if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Select as i32) != 0i32
        || (ram_word!(game, joypad1_newkeys) as i32 & kButton_Up as i32) != 0i32
        || (ram_word!(game, joypad1_newkeys) as i32 & kButton_Down as i32) != 0i32
    {
        update_ram_word!(game.file_select_map_area_index, |fresh91| (fresh91)
            ^ (1u16));
    } else if (ram_word!(game, joypad1_newkeys)
        & (kButton_B as i32 | kButton_Start as i32 | kButton_A as i32) as u16)
        != 0
    {
        if ram_word!(game, file_select_map_area_index) != 0 {
            update_ram_word!(game.menu_index, wrapping_add(1));
        } else {
            SaveToSram(ram_word!(game, selected_save_slot), game);
            SoftReset(game);
        }
        return;
    }
    let mut v0: i16 = 30720i16;
    if ram_word!(game, file_select_map_area_index) != 0 {
        v0 = -30720i16;
    }
    let mut v1: i16 = (v0) | (0x28i16);
    let v3 = gOamEnt(ram_word!(game, oam_next_ptr) as i32, game);
    *(&raw mut (*v3).xcoord as *mut u16) = v1 as u16;
    *(&raw mut (*v3).charnum as *mut u16) = 182u16;
    update_ram_word!(game.oam_next_ptr, wrapping_add(4u16));
}
pub unsafe fn RestorePalettesAndIoAfterDebugGameover(game: &mut Game) {
    let mut v0: i32 = 0i32;
    loop {
        set_ram_word!(
            game.palette_buffer[v0] =
                (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[(v0 + 384i32) as usize]
        );
        v0 += 1;
        if (((v0 * 2i32 - 512i32) as i16) >= 0i16) {
            break;
        }
    }
    let mut v1: i32 = 0i32;
    loop {
        *(ram_ptr!(game, u16; 0x51, (v1 * 2i32) as isize)) =
            (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[(v1 + 640i32) as usize];
        v1 += 1;
        if (((v1 * 2i32 - 54i32) as i16) >= 0i16) {
            break;
        }
    }
}
pub unsafe fn GameOverMenu_5_Continue(game: &mut Game) {
    crate::rtl::register_writes! { game;
        byte(VMADDL as u16, 0u8); byte(VMADDH as u16, 0x40u8);
        byte(VMAIN as u16, 0x80u8);
    }
    static unk_819093: StartDmaCopy = StartDmaCopy::vram_words(1, 0x9ab200, 0x2000);
    SetupDmaTransfer(&raw const unk_819093 as *const c_void, game);
    WriteReg(MDMAEN as u16, 2u8, game);
    set_ram_word!(game.game_state = 16u16);
    set_ram_word!(game.file_select_map_area_index = 0u16);
    RestorePalettesAndIoAfterDebugGameover(game);
}
static kGameOverMenuFuncs: [Option<Func_V>; 8] = callback_table! { unsafe Func_V;
    GameOverMenu_0_FadeOutConfigGfx, GameOverMenu_1_Init, GameOverMenu_2_PlayMusic,
    GameOverMenu_3_FadeIn, GameOverMenu_4_Main, GameOverMenu_5_FadeOutToGameMap,
    GameOverMenu_6_LoadGameMapView, GameOverMenu_7_FadeOutToSoftReset,
};
pub unsafe fn GameOverMenu(game: &mut Game) {
    kGameOverMenuFuncs[ram_word!(game, menu_index) as usize].expect("non-null function pointer")(
        game,
    );
}
pub unsafe fn GameOverMenu_3_FadeIn(game: &mut Game) {
    HandleGameOverBabyMetroid(game);
    DrawMenuSelectionMissile(game);
    HandleFadeIn(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 15 {
        update_ram_word!(game.menu_index, wrapping_add(1));
    }
}
pub unsafe fn GameOverMenu_5_FadeOutToGameMap(game: &mut Game) {
    HandleGameOverBabyMetroid(game);
    DrawMenuSelectionMissile(game);
    HandleFadeOut(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
        update_ram_word!(game.menu_index, wrapping_add(1));
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
    set_ram_word!(game.game_state = kGameState_5_FileSelectMap as u16);
    set_ram_word!(game.menu_index = 0u16);
}
pub unsafe fn GameOverMenu_4_Main(game: &mut Game) {
    HandleGameOverBabyMetroid(game);
    DrawMenuSelectionMissile(game);
    if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Select as i32) != 0i32
        || (ram_word!(game, joypad1_newkeys) as i32 & kButton_Up as i32) != 0i32
        || (ram_word!(game, joypad1_newkeys) as i32 & kButton_Down as i32) != 0i32
    {
        QueueSfx1_Max6(0x37u16, game);
        update_ram_word!(game.file_select_map_area_index, |fresh84| (fresh84)
            ^ (1u16));
    } else if (ram_word!(game, joypad1_newkeys) as i32 & kButton_A as i32) != 0i32 {
        (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).instruction_timer = 180u16;
        if ram_word!(game, file_select_map_area_index) != 0 {
            set_ram_word!(game.menu_index = 7u16);
        } else {
            if ram_word!(game, loading_game_state) as i32 == kGameState_31_SetUpNewGame as i32 {
                set_ram_word!(game.game_state = ram_word!(game, loading_game_state));
            } else {
                update_ram_word!(game.menu_index, wrapping_add(1));
            }
            LoadFromSram(ram_word!(game, selected_save_slot), game);
        }
        return;
    }
    let mut v0: u16 = 160u16;
    if ram_word!(game, file_select_map_area_index) != 0 {
        v0 = 192u16;
    }
    set_ram_word!(game.eproj_id[5] = 40u16);
    set_ram_word!(game.eproj_id[10] = v0);
}
pub unsafe fn GameOverMenu_1_Init(game: &mut Game) {
    *ram_ptr!(game; 0x69) = 17u8;
    *ram_ptr!(game; 0x74, 0) = 32u8;
    *ram_ptr!(game; 0x74, 1) = 64u8;
    *ram_ptr!(game; 0x74, 2) = 0x80u8;
    QueueMusic_Delayed8(0u16, game);
    QueueMusic_Delayed8(0xff03u16, game);
    set_ram_word!(game.cinematic_var5 = 1u16);
    set_ram_word!(game.eproj_id[0] = 0u16);
    let mut v0: i32 = 0i32;
    loop {
        *(&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).menu.menu_tilemap as *mut u8 as *mut u16)
            .offset(v0 as isize) = 15u16;
        v0 += 1;
        if (((v0 * 2i32 - 2048i32) as i16) >= 0i16) {
            break;
        }
    }
    let v1: i32 = ram_word!(game, vram_write_queue_tail) as i32;
    let v2 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
    (*v2).size = 2048u16;
    (*v2).src.addr = (&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).menu.menu_tilemap as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut u16) = 126u16;
    (*v2).vram_dst = ((*ram_ptr!(game; 0x58) as i32 & 0xfci32) << 8i32) as u16;
    set_ram_word!(game.vram_write_queue_tail = (v1 + 7i32) as u16);
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    LoadMenuTilemap(0x156u16, addr_kMenuTilemap_GameOver as u16, game);
    LoadMenuTilemap(0x38au16, addr_kMenuTilemap_FindMetroidLarva as u16, game);
    LoadMenuTilemap(0x414u16, addr_kMenuTilemap_TryAgain as u16, game);
    LoadMenuTilemap(0x4ceu16, addr_kMenuTilemap_YesReturnToGame as u16, game);
    LoadMenuTilemap(0x5ceu16, addr_kMenuTilemap_NoGoToTitle as u16, game);
    enemy_instruction! {[(*(ram_ptr!(game, EnemyData; 0xf78)).offset(0))]; instruction = 0u16, timer = 0u16}
    HandleGameOverBabyMetroid(game);
    *(ram_ptr!(game, u16; 0x6f)) = ((*ram_ptr!(game; 0x70) as i32) << 8i32) as u16;
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    EnableHdmaObjects(game);
    static unk_819254: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0u8, 0x32u8, 0x927d as VoidP);
    static unk_81925C: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0u8, 0x31u8, 0x928d as VoidP);
    SpawnHdmaObject(0x81u8, &raw const unk_819254 as *const c_void, game);
    SpawnHdmaObject(0x81u8, &raw const unk_81925C as *const c_void, game);
    ScreenOn(game);
    update_ram_word!(game.menu_index, wrapping_add(1));
    set_ram_words! { game, 0u16;
        screen_fade_delay, screen_fade_counter, file_select_map_area_index,
    }
    set_ram_word!(game.eproj_id[5] = 40u16);
    set_ram_word!(game.eproj_id[10] = 160u16);
}
pub unsafe fn GameOverMenu_2_PlayMusic(game: &mut Game) {
    if HasQueuedMusic(game) == 0 {
        update_ram_word!(game.menu_index, wrapping_add(1));
        QueueMusic_Delayed8(4u16, game);
    }
}
static kFileSelectMenuFuncs: [Option<Func_V>; 34] = callback_table! { unsafe Func_V;
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
    kFileSelectMenuFuncs[ram_word!(game, menu_index) as usize].expect("non-null function pointer")(
        game,
    );
}
pub unsafe fn FileSelectMenu_0_FadeOutConfigGfx(game: &mut Game) {
    HandleFadeOut(game);
    if (*ram_ptr!(game; 0x51) & 0xf) != 0 {
        return;
    }
    ScreenOff(game);
    QueueSfx3_Max6(1u16, game);
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    update_ram_word!(game.menu_index, wrapping_add(1));
    MapVramForMenu(game);
    LoadInitialMenuTiles(game);
    set_ram_words! { game, 0u16;
        reg_BG1HOFS, reg_BG2HOFS, reg_BG3HOFS, reg_BG1VOFS, reg_BG2VOFS, reg_BG3VOFS,
    }
    LoadFileSelectPalettes(game);
}
pub unsafe fn LoadFileSelectPalettes(game: &mut Game) {
    let mut v0: i32 = 0i32;
    loop {
        let v1: i32 = v0 >> 1i32;
        set_ram_word!(
            game.palette_buffer[v1] = crate::rtl::rom_table_word!(game, 0x8ee400, v1 as isize)
        );
        set_ram_word!(
            game.palette_buffer[(v1 + 1i32)] =
                crate::rtl::rom_table_word!(game, 0x8ee400, (v1 + 1i32) as isize)
        );
        v0 += 4i32;
        if (((v0 - 512i32) as i16) >= 0i16) {
            break;
        }
    }
}
pub unsafe fn FileSelectMenu_32_FadeOutToOptions(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    DrawBorderAroundSamusData(game);
    DrawFileSelectSamusHelmets(game);
    HandleFadeOut(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
        set_ram_word!(game.game_state = kGameState_2_GameOptionsMenu as u16);
        set_ram_word!(game.menu_index = 0u16);
        let mut v0: i32 = 0i32;
        loop {
            *((ram_ptr!(game, u16; 0x198d) as *mut u8).offset(v0 as isize) as *mut u16) = 0u16;
            v0 += 2i32;
            if (((v0 - 48i32) as i16) >= 0i16) {
                break;
            }
        }
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
    *ram_ptr!(game; 0x57) = (*ram_ptr!(game; 0x57) as i32 & 0xfi32
        | 16i32 * (*ram_ptr!(game; 0x51) as i32 & 0xfi32) ^ 0xf0i32)
        as u8;
    if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
        ScreenOff(game);
        update_ram_word!(game.menu_index, wrapping_add(1));
    }
}
static kMenuSelectionMissileXY: [u16; 12] = [
    0x30, 0xe, 0x58, 0xe, 0x80, 0xe, 0xa3, 0xe, 0xbb, 0xe, 0xd3, 0xe,
];
pub unsafe fn FileSelectMenu_17_FadeInToMain(game: &mut Game) {
    let v0: i32 = ((4u16).wrapping_mul(ram_word!(game, selected_save_slot))) as i32 >> 1i32;
    set_ram_word!(game.eproj_id[10] = kMenuSelectionMissileXY[v0 as usize]);
    set_ram_word!(game.eproj_id[5] = kMenuSelectionMissileXY[(v0 + 1i32) as usize]);
    FileSelectMenu_7_FadeInFromMain(game);
}
pub unsafe fn FileSelectMenu_7_FadeInFromMain(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    HandleFadeIn(game);
    *ram_ptr!(game; 0x57) = (*ram_ptr!(game; 0x57) as i32 & 0xfi32
        | 16i32 * (*ram_ptr!(game; 0x51) as i32 & 0xfi32) ^ 0xf0i32)
        as u8;
    if (*ram_ptr!(game; 0x51) & 0xf) == 15 {
        update_ram_word!(game.menu_index, wrapping_add(1));
    }
}
pub unsafe fn FileSelectMenu_6_FileCopyInit(game: &mut Game) {
    update_ram_word!(game.menu_index, wrapping_add(1));
    FileSelectMenu_Func1(game);
}
pub unsafe fn FileSelectMenu_Func1(game: &mut Game) {
    ClearMenuTilemap(game);
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    LoadMenuTilemap(0x52u16, addr_kMenuTilemap_DataCopyMode as u16, game);
    LoadMenuTilemap(0x150u16, addr_kMenuTilemap_CopyWhichData as u16, game);
    LoadMenuExitTilemap(game);
    DrawFileCopySaveFileInfo(game);
    SetInitialFileCopyMenuSelection(game);
    SetFileCopyMenuSelectionMissilePosition(game);
    set_ram_words! { game, 0u16; eproj_id[16], eproj_id[17] }
}
pub unsafe fn sub_819591(game: &mut Game) {
    SetInitialFileCopyMenuSelection(game);
}
pub unsafe fn SetInitialFileCopyMenuSelection(game: &mut Game) {
    let mut v0: i32 = ram_word!(game, nonempty_save_slots) as i32;
    let mut v1: i32 = 0i32;
    while (v0 & 1i32 == 0) {
        v0 >>= 1i32;
        v1 += 1;
        if (((v1 - 3i32) as i16) >= 0i16) {
            break;
        }
    }
    set_ram_word!(game.eproj_id[15] = v1 as u16);
}
pub unsafe fn ClearMenuTilemap(game: &mut Game) {
    let mut i: i32 = 2046i32;
    while i >= 0i32 {
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(768) as *mut u8)
            .offset(i as u16 as i32 as isize) as *mut u16) = 15u16;
        i -= 2i32;
    }
}
pub unsafe fn LoadMenuExitTilemap(game: &mut Game) {
    LoadMenuTilemap(0x688u16, addr_kMenuTilemap_Exit as u16, game);
}
pub unsafe fn DrawFileCopySaveFileInfo(game: &mut Game) {
    LoadFromSram_(0u16, game);
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    if (ram_word!(game, nonempty_save_slots) & 1) == 0 {
        (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 1024u16;
    }
    DrawFileCopySaveSlotAInfo(game);
    LoadFromSram_(1u16, game);
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    if (ram_word!(game, nonempty_save_slots) & 2) == 0 {
        (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 1024u16;
    }
    DrawFileCopySaveSlotBInfo(game);
    LoadFromSram_(2u16, game);
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    if (ram_word!(game, nonempty_save_slots) & 4) == 0 {
        (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 1024u16;
    }
    DrawFileCopySaveSlotCInfo(game);
    QueueTransferOfMenuTilemapToVramBG1(game);
}
pub unsafe fn DrawFileCopySaveSlotAInfo(game: &mut Game) {
    DrawFileSelectionHealth(
        ((ram_word!(game, nonempty_save_slots) & 1) == 0) as u16,
        0x218u16,
        game,
    );
    DrawFileSelectionTime(
        ((ram_word!(game, nonempty_save_slots) & 1) == 0) as u16,
        0x272u16,
        game,
    );
    LoadMenuTilemap(0x234u16, addr_kMenuTilemap_TIME as u16, game);
    LoadMenuTilemap(0x208u16, addr_kMenuTilemap_SamusA as u16, game);
}
pub unsafe fn DrawFileCopySaveSlotBInfo(game: &mut Game) {
    DrawFileSelectionHealth(
        (!(ram_word!(game, nonempty_save_slots)) & 2),
        0x318u16,
        game,
    );
    DrawFileSelectionTime(
        (!(ram_word!(game, nonempty_save_slots)) & 2),
        0x372u16,
        game,
    );
    LoadMenuTilemap(0x334u16, addr_kMenuTilemap_TIME as u16, game);
    LoadMenuTilemap(0x308u16, addr_kMenuTilemap_SamusB as u16, game);
}
pub unsafe fn DrawFileCopySaveSlotCInfo(game: &mut Game) {
    DrawFileSelectionHealth(
        (!(ram_word!(game, nonempty_save_slots)) & 4),
        0x418u16,
        game,
    );
    DrawFileSelectionTime(
        (!(ram_word!(game, nonempty_save_slots)) & 4),
        0x472u16,
        game,
    );
    LoadMenuTilemap(0x434u16, addr_kMenuTilemap_TIME as u16, game);
    LoadMenuTilemap(0x408u16, addr_kMenuTilemap_SamusC as u16, game);
}
pub unsafe fn QueueTransferOfMenuTilemapToVramBG1(game: &mut Game) {
    let v0: u16 = ram_word!(game, vram_write_queue_tail);
    let v1 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
    (*v1).size = 2048u16;
    (*v1).src.addr = (&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).menu.menu_tilemap as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as VoidP;
    *(&raw mut (*v1).src.bank as *mut u16) = 126u16;
    (*v1).vram_dst = ((*ram_ptr!(game; 0x58) as i32 & 0xfci32) << 8i32) as u16;
    set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(7u16));
}
static kBitShl: [u8; 3] = [1, 2, 4];
pub unsafe fn FileSelectMenu_8(game: &mut Game) {
    let mut v0: u8 = 0;
    DrawBorderAroundDataCopyMode(game);
    DrawMenuSelectionMissile(game);
    if ram_word!(game, joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0i32 {
        QueueSfx1_Max6(0x37u16, game);
        if ram_word!(game, eproj_id[15]) == 3u16 {
            update_ram_word!(game.menu_index, wrapping_add(7u16));
        } else {
            set_ram_word!(game.eproj_id[16] = ram_word!(game, eproj_id[15]));
            update_ram_word!(game.menu_index, wrapping_add(1));
        }
    } else if (ram_word!(game, joypad1_newkeys) & 0x8000) != 0 {
        update_ram_word!(game.menu_index, wrapping_add(7u16));
        QueueSfx1_Max6(0x37u16, game);
    } else {
        let selection_unchanged: bool;
        if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Up as i32) != 0i32 {
            v0 = ram_word!(game, eproj_id[15]) as u8;
            loop {
                v0 = v0.wrapping_sub(1);
                if ((v0 & 0x80) != 0) {
                    selection_unchanged = true;
                    break;
                }
                if (kBitShl[v0 as usize] as u16 & ram_word!(game, nonempty_save_slots)) != 0 {
                    selection_unchanged = false;
                    break;
                }
            }
        } else if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Down as i32) != 0i32 {
            v0 = ram_word!(game, eproj_id[15]) as u8;
            loop {
                v0 = v0.wrapping_add(1);
                if (((v0) as i8).wrapping_sub(4i8) >= 0i8) {
                    selection_unchanged = true;
                    break;
                }
                if v0 == 3u8
                    || (kBitShl[v0 as usize] as u16 & ram_word!(game, nonempty_save_slots)) != 0
                {
                    selection_unchanged = false;
                    break;
                }
            }
        } else {
            selection_unchanged = true;
        }
        if selection_unchanged {
        } else {
            *((ram_ptr!(game, u16; 0x1997)).offset(15) as *mut u8).offset(0) = v0;
            QueueSfx1_Max6(0x37u16, game);
            SetFileCopyMenuSelectionMissilePosition(game);
            return;
        }
        SetFileCopyMenuSelectionMissilePosition(game);
    };
}
pub unsafe fn SetFileCopyMenuSelectionMissilePosition(game: &mut Game) {
    static kFileCopyMissileY: [u16; 4] = [72u16, 104u16, 136u16, 211u16];
    set_ram_word!(game.eproj_id[10] = kFileCopyMissileY[ram_word!(game, eproj_id[15]) as usize]);
    set_ram_word!(game.eproj_id[5] = 22u16);
}
pub unsafe fn FileSelectMenu_9_InitializeSelectDest(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    DrawFileCopySelectDestinationSaveFileInfo(game);
    update_ram_word!(game.menu_index, wrapping_add(1));
    let mut v0: u16 = 0u16;
    while (v0 == ram_word!(game, eproj_id[16])) {
        v0 = v0.wrapping_add(1);
        if (v0 as i32 - 3i32 & 0x8000i32 == 0) {
            break;
        }
    }
    set_ram_word!(game.eproj_id[15] = v0);
    SetFileCopyMenuSelectionMissilePosition(game);
}
pub unsafe fn DrawFileCopySelectDestinationSaveFileInfo(game: &mut Game) {
    ClearMenuTilemap(game);
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    LoadMenuTilemap(0x52u16, addr_kMenuTilemap_DataCopyMode as u16, game);
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    LoadMenuTilemap(0x148u16, addr_kMenuTilemap_CopySamusToWhere as u16, game);
    (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[944i32 as usize] =
        (ram_word!(game, eproj_id[16])).wrapping_add(8298u16);
    LoadMenuExitTilemap(game);
    LoadFromSram_(0u16, game);
    let mut v0: u16 = 1024u16;
    if ram_word!(game, eproj_id[16]) != 0 {
        v0 = 0u16;
    }
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = v0;
    DrawFileCopySaveSlotAInfo(game);
    LoadFromSram_(1u16, game);
    let mut v1: u16 = 1024u16;
    if ram_word!(game, eproj_id[16]) != 1u16 {
        v1 = 0u16;
    }
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = v1;
    DrawFileCopySaveSlotBInfo(game);
    LoadFromSram_(2u16, game);
    let mut v2: u16 = 1024u16;
    if ram_word!(game, eproj_id[16]) != 2u16 {
        v2 = 0u16;
    }
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = v2;
    DrawFileCopySaveSlotCInfo(game);
    QueueTransferOfMenuTilemapToVramBG1(game);
}
pub unsafe fn FileSelectMenu_10_FileCopySelectDest(game: &mut Game) {
    let mut current_block: u64;
    static kFileCopySelectDest_MissileY: [u16; 4] = [72u16, 104u16, 136u16, 212u16];
    let mut v0: u16 = 0;
    DrawBorderAroundDataCopyMode(game);
    DrawMenuSelectionMissile(game);
    if ram_word!(game, joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0i32 {
        QueueSfx1_Max6(0x37u16, game);
        if ram_word!(game, eproj_id[15]) != 3u16 {
            set_ram_word!(game.eproj_id[17] = ram_word!(game, eproj_id[15]));
            update_ram_word!(game.menu_index, wrapping_add(1));
            current_block = 16545170798630791882;
        } else {
            update_ram_word!(game.menu_index, wrapping_add(5u16));
            current_block = 16203760046146113240;
        }
    } else if (ram_word!(game, joypad1_newkeys) & 0x8000) == 0 {
        if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Up as i32) != 0i32 {
            QueueSfx1_Max6(0x37u16, game);
            v0 = ram_word!(game, eproj_id[15]);
            loop {
                v0 = v0.wrapping_sub(1);
                if ((v0 & 0x8000) != 0) {
                    current_block = 16545170798630791882;
                    break;
                }
                if v0 != ram_word!(game, eproj_id[16]) {
                    current_block = 1572352429258197161;
                    break;
                }
            }
        } else if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Down as i32) != 0i32 {
            QueueSfx1_Max6(0x37u16, game);
            v0 = ram_word!(game, eproj_id[15]);
            loop {
                v0 = v0.wrapping_add(1);
                if (v0 == 4u16) {
                    current_block = 16545170798630791882;
                    break;
                }
                if v0 != ram_word!(game, eproj_id[16]) {
                    current_block = 1572352429258197161;
                    break;
                }
            }
        } else {
            current_block = 16545170798630791882;
        }
        if current_block == 16545170798630791882 {
        } else {
            set_ram_word!(game.eproj_id[15] = v0);
            current_block = 16545170798630791882;
        }
    } else {
        update_ram_word!(game.menu_index, wrapping_sub(2u16));
        set_ram_word!(game.eproj_id[15] = ram_word!(game, eproj_id[16]));
        QueueSfx1_Max6(0x37u16, game);
        FileSelectMenu_Func1(game);
        current_block = 16203760046146113240;
    }
    if current_block == 16203760046146113240 {
    } else {
        set_ram_word!(
            game.eproj_id[10] =
                kFileCopySelectDest_MissileY[ram_word!(game, eproj_id[15]) as usize]
        );
        set_ram_word!(game.eproj_id[5] = 22u16);
    }
}
pub unsafe fn FileSelectMenu_11_InitializeConfirm(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    LoadMenuTilemap(0x144u16, addr_kMenuTilemap_CopySamusToSamus as u16, game);
    (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[942i32 as usize] =
        (ram_word!(game, eproj_id[16])).wrapping_add(8298u16);
    (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[955i32 as usize] =
        (ram_word!(game, eproj_id[17])).wrapping_add(8298u16);
    DrawFileCopyClearConfirmation(game);
    update_ram_word!(game.menu_index, wrapping_add(1));
    set_ram_word!(game.eproj_id[15] = 0u16);
    set_ram_word!(game.cinematic_var6 = 8u16);
}
pub unsafe fn DrawFileCopyClearConfirmation(game: &mut Game) {
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    LoadMenuTilemap(0x514u16, addr_kMenuTilemap_IsThisOk as u16, game);
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    LoadMenuTilemap(0x59cu16, addr_kMenuTilemap_Yes as u16, game);
    let mut v0: i32 = 832i32;
    loop {
        // Menu rows live in the full $3600..$3dff RAM tilemap.
        *((&raw mut game.g_ram as *mut u8)
            .add(0x3600)
            .cast::<u16>()
            .add(v0 as usize)) = 15u16;
        v0 += 1;
        if (((v0 * 2i32 - 1728i32) as i16) >= 0i16) {
            break;
        }
    }
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    LoadMenuTilemap(0x65cu16, addr_kMenuTilemap_No as u16, game);
    DrawFileCopyConfirmationSaveFileInfo(game);
}
pub unsafe fn DrawFileCopyConfirmationSaveFileInfo(game: &mut Game) {
    LoadFromSram_(0u16, game);
    let mut v0: i32 = 0i32;
    if ram_word!(game, eproj_id[16]) != 0u16 && ram_word!(game, eproj_id[17]) != 0u16 {
        v0 = 1024i32;
    }
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = v0 as u16;
    DrawFileCopySaveSlotAInfo(game);
    LoadFromSram_(1u16, game);
    let mut v1: i32 = 0i32;
    if ram_word!(game, eproj_id[16]) != 1u16 && ram_word!(game, eproj_id[17]) != 1u16 {
        v1 = 1024i32;
    }
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = v1 as u16;
    DrawFileCopySaveSlotBInfo(game);
    LoadFromSram_(2u16, game);
    let mut v2: i32 = 0i32;
    if ram_word!(game, eproj_id[16]) != 2u16 && ram_word!(game, eproj_id[17]) != 2u16 {
        v2 = 1024i32;
    }
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = v2 as u16;
    DrawFileCopySaveSlotCInfo(game);
    QueueTransferOfMenuTilemapToVramBG1(game);
}
pub unsafe fn FileSelectMenu_12_FileCopyConfirm(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    DrawMenuSelectionMissile(game);
    HandleFileCopyArrowPalette(game);
    DrawFileCopyArrow(game);
    if ram_word!(game, joypad1_newkeys) as i32 & (kButton_Up as i32 | kButton_Down as i32) != 0i32 {
        update_ram_word!(game.eproj_id[15], |fresh64| (fresh64) ^ (1u16));
        QueueSfx1_Max6(0x37u16, game);
    } else {
        if (ram_word!(game, joypad1_newkeys) & 0x8000) != 0 {
            update_ram_word!(game.menu_index, wrapping_sub(3u16));
            set_ram_word!(game.eproj_id[15] = ram_word!(game, eproj_id[17]));
            QueueSfx1_Max6(0x37u16, game);
            return;
        }
        if ram_word!(game, joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32)
            != 0i32
        {
            QueueSfx1_Max6(0x38u16, game);
            if ram_word!(game, eproj_id[15]) != 0 {
                update_ram_word!(game.menu_index, wrapping_sub(4u16));
                FileSelectMenu_Func1(game);
            } else {
                update_ram_word!(game.menu_index, wrapping_add(1));
            }
            return;
        }
    }
    let mut v0: u16 = 184u16;
    if ram_word!(game, eproj_id[15]) != 0 {
        v0 = 208u16;
    }
    set_ram_word!(game.eproj_id[10] = v0);
    set_ram_word!(game.eproj_id[5] = 94u16);
}
pub unsafe fn HandleFileCopyArrowPalette(game: &mut Game) {
    if ram_word!(game, cinematic_var6) != 0 {
        if update_ram_word!(game.cinematic_var6, wrapping_sub(1), reread) == 0 {
            set_ram_word!(game.cinematic_var6 = 4u16);
            let v0: u16 = ram_word!(game, palette_buffer[145]);
            let mut v1: u16 = 0u16;
            loop {
                *(ram_ptr!(game, u16; 0xc000)).offset(((v1 as i32 >> 1i32) + 145i32) as isize) =
                    *(ram_ptr!(game, u16; 0xc000)).offset(((v1 as i32 >> 1i32) + 146i32) as isize);
                v1 = (v1).wrapping_add(2u16);
                if (((v1) as i16).wrapping_sub(12i16) >= 0i16) {
                    break;
                }
            }
            set_ram_word!(game.palette_buffer[151] = v0);
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
        crate::rtl::rom_table_word!(game, 0x81812b, ram_word!(game, eproj_id[16]) as isize);
    let dst_addr: u16 =
        crate::rtl::rom_table_word!(game, 0x81812b, ram_word!(game, eproj_id[17]) as isize);
    memcpy(
        game.g_sram.offset(dst_addr as isize) as *mut c_void,
        game.g_sram.offset(src_addr as isize) as *const c_void,
        1628 as size_t,
    );
    let v2: i32 = ram_word!(game, eproj_id[16]) as i32;
    let v10: i32 = *(game.g_sram.offset((2i32 * v2 + 0x1ff0i32) as isize) as *mut u16) as i32;
    let v9: i32 = *(game.g_sram.offset((2i32 * v2 + 0x1ff8i32) as isize) as *mut u16) as i32;
    let v8: i32 = *(game.g_sram.offset((2i32 * v2 + 0i32) as isize) as *mut u16) as i32;
    let v4: i32 = ram_word!(game, eproj_id[17]) as i32;
    *(game.g_sram.offset((2i32 * v4 + 8i32) as isize) as *mut u16) =
        *(game.g_sram.offset((2i32 * v2 + 8i32) as isize) as *mut u16);
    *(game.g_sram.offset((2i32 * v4) as isize) as *mut u16) = v8 as u16;
    *(game.g_sram.offset((2i32 * v4 + 0x1ff8i32) as isize) as *mut u16) = v9 as u16;
    *(game.g_sram.offset((2i32 * v4 + 0x1ff0i32) as isize) as *mut u16) = v10 as u16;
    update_ram_word!(game.menu_index, wrapping_add(1));
    let mut v5: i32 = 640i32;
    loop {
        // Menu rows live in the full $3600..$3dff RAM tilemap.
        *((&raw mut game.g_ram as *mut u8)
            .add(0x3600)
            .cast::<u16>()
            .add(v5 as usize)) = 15u16;
        v5 += 1;
        if (((v5 * 2i32 - 1856i32) as i16) >= 0i16) {
            break;
        }
    }
    update_ram_word!(game.nonempty_save_slots, |fresh62| (fresh62)
        | (kBitShl_16bit[ram_word!(game, eproj_id[17]) as usize]));
    let mut v6: i32 = ((4i32 * ram_word!(game, eproj_id[17]) as i32 + 9i32) << 6i32) + 24i32;
    let mut v7: i32 = 0i32;
    loop {
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(768) as *mut u8)
            .offset(v6 as isize) as *mut u16) = 15u16;
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(736) as *mut u8)
            .offset(v6 as isize) as *mut u16) = 15u16;
        v6 += 2i32;
        v7 += 2i32;
        if (((v7 - 22i32) as i16) >= 0i16) {
            break;
        }
    }
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    LoadMenuTilemap(0x510u16, addr_kMenuTilemap_CopyCompleted as u16, game);
    DrawFileCopyConfirmationSaveFileInfo(game);
    RtlWriteSram(game);
}
pub unsafe fn FileSelectMenu_14_CopyCompleted(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    if ram_word!(game, joypad1_newkeys) != 0 {
        QueueSfx1_Max6(0x37u16, game);
        update_ram_word!(game.menu_index, wrapping_add(1));
        let mut slot: i32 = *(game.g_sram.offset(0x1fec) as *mut u16) as i32;
        if slot & 0x8000i32 != 0
            || *(game.g_sram.offset(0x1fec) as *mut u16) as i32 - 3i32 & 0x8000i32 == 0
            || slot & *(game.g_sram.offset(0x1fee) as *mut u16) as i32 != 0
        {
            slot = 0i32;
        }
        set_ram_word!(game.selected_save_slot = slot as u16);
    }
}
pub unsafe fn FileSelectMenu_18(game: &mut Game) {
    update_ram_word!(game.menu_index, wrapping_sub(14u16));
}
pub unsafe fn FileSelectMenu_20_FileClearInit(game: &mut Game) {
    DrawBorderAroundDataClearMode(game);
    update_ram_word!(game.menu_index, wrapping_add(1));
    InitFileSelectMenuFileClear(game);
}
pub unsafe fn InitFileSelectMenuFileClear(game: &mut Game) {
    ClearMenuTilemap(game);
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    LoadMenuTilemap(0x50u16, addr_kMenuTilemap_DataClearMode as u16, game);
    LoadMenuTilemap(0x140u16, addr_kMenuTilemap_ClearWhichData as u16, game);
    LoadMenuExitTilemap(game);
    DrawFileCopySaveFileInfo(game);
    set_ram_word!(game.eproj_id[16] = 0u16);
    SetInitialFileCopyMenuSelection(game);
    SetFileClearMenuMissilePos(game);
}
pub unsafe fn FileSelectMenu_22_FileClearSelectSlot(game: &mut Game) {
    static kBitShl_: [u8; 3] = [1, 2, 4];
    let mut v0: u16 = 0;
    let mut v1: u8 = 0;
    DrawBorderAroundDataClearMode(game);
    DrawMenuSelectionMissile(game);
    *(&raw mut v0 as *mut u8).offset(1isize) =
        *(ram_ptr!(game, u16; 0x8f) as *mut u8).offset(1isize);
    if ram_word!(game, joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0i32 {
        QueueSfx1_Max6(0x37u16, game);
        if ram_word!(game, eproj_id[15]) != 3u16 {
            set_ram_word!(game.eproj_id[16] = ram_word!(game, eproj_id[15]));
            update_ram_word!(game.menu_index, wrapping_add(1));
            return;
        }
    } else if ((ram_word!(game, joypad1_newkeys) & 0x8000) == 0) {
        let selection_unchanged: bool;
        if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Up as i32) != 0i32 {
            v1 = ram_word!(game, eproj_id[15]) as u8;
            loop {
                v1 = v1.wrapping_sub(1);
                if ((v1 & 0x80) != 0) {
                    selection_unchanged = true;
                    break;
                }
                if (kBitShl_[v1 as usize] as u16 & ram_word!(game, nonempty_save_slots)) != 0 {
                    selection_unchanged = false;
                    break;
                }
            }
        } else if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Down as i32) != 0i32 {
            v1 = ram_word!(game, eproj_id[15]) as u8;
            loop {
                v1 = v1.wrapping_add(1);
                if (((v1) as i8).wrapping_sub(4i8) >= 0i8) {
                    selection_unchanged = true;
                    break;
                }
                if v1 == 3u8
                    || (kBitShl_[v1 as usize] as u16 & ram_word!(game, nonempty_save_slots)) != 0
                {
                    selection_unchanged = false;
                    break;
                }
            }
        } else {
            selection_unchanged = true;
        }
        if selection_unchanged {
        } else {
            *((ram_ptr!(game, u16; 0x1997)).offset(15) as *mut u8).offset(0) = v1;
            *(&raw mut v0 as *mut u8).offset(0) = 55u8;
            QueueSfx1_Max6(v0, game);
            SetFileClearMenuMissilePos(game);
            return;
        }
        SetFileClearMenuMissilePos(game);
        return;
    }
    QueueSfx1_Max6(0x37u16, game);
    update_ram_word!(game.menu_index, wrapping_add(5u16));
}
pub unsafe fn SetFileClearMenuMissilePos(game: &mut Game) {
    static kFileClear_MissileY: [u16; 4] = [72u16, 104u16, 136u16, 211u16];
    set_ram_word!(game.eproj_id[10] = kFileClear_MissileY[ram_word!(game, eproj_id[15]) as usize]);
    set_ram_word!(game.eproj_id[5] = 22u16);
}
pub unsafe fn FileSelectMenu_23_FileClearInitConfirm(game: &mut Game) {
    DrawBorderAroundDataClearMode(game);
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    LoadMenuTilemap(0x140u16, addr_kMenuTilemap_ClearSamusA as u16, game);
    (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap[949i32 as usize] =
        (ram_word!(game, eproj_id[16])).wrapping_add(8298u16);
    set_ram_word!(game.eproj_id[17] = 3u16);
    DrawFileCopyClearConfirmation(game);
    update_ram_word!(game.menu_index, wrapping_add(1));
    set_ram_word!(game.eproj_id[15] = 0u16);
}
pub unsafe fn FileSelectMenu_24_FileClearConfirm(game: &mut Game) {
    let keep_confirming: bool;
    DrawBorderAroundDataClearMode(game);
    DrawMenuSelectionMissile(game);
    if ram_word!(game, joypad1_newkeys) as i32 & (kButton_Up as i32 | kButton_Down as i32) != 0i32 {
        update_ram_word!(game.eproj_id[15], |fresh52| (fresh52) ^ (1u16));
        QueueSfx1_Max6(0x37u16, game);
    } else {
        if (ram_word!(game, joypad1_newkeys) & 0x8000) != 0 {
            keep_confirming = false;
        } else if ram_word!(game, joypad1_newkeys) as i32
            & (kButton_Start as i32 | kButton_A as i32)
            == 0i32
        {
            keep_confirming = true;
        } else {
            QueueSfx1_Max6(0x38u16, game);
            if ram_word!(game, eproj_id[15]) != 0 {
                keep_confirming = false;
            } else {
                update_ram_word!(game.menu_index, wrapping_add(1));
                return;
            }
        }
        if keep_confirming {
        } else {
            update_ram_word!(game.menu_index, wrapping_sub(2u16));
            set_ram_word!(game.eproj_id[15] = ram_word!(game, eproj_id[16]));
            QueueSfx1_Max6(0x37u16, game);
            InitFileSelectMenuFileClear(game);
            return;
        }
    }
    let mut v0: u16 = 184u16;
    if ram_word!(game, eproj_id[15]) != 0 {
        v0 = 208u16;
    }
    set_ram_word!(game.eproj_id[10] = v0);
    set_ram_word!(game.eproj_id[5] = 94u16);
}
pub unsafe fn FileSelectMenu_25_FileClearDoClear(game: &mut Game) {
    static kBitShl_Not: [u16; 3] = [0xfffe, 0xfffd, 0xfffb];
    DrawBorderAroundDataClearMode(game);
    let sram_addr: i32 =
        crate::rtl::rom_table_word!(game, 0x81812b, ram_word!(game, eproj_id[16]) as isize) as i32;
    memset(
        game.g_sram.offset(sram_addr as isize) as *mut c_void,
        0i32,
        1628 as size_t,
    );
    let v1: u16 = (2u16).wrapping_mul(ram_word!(game, eproj_id[16]));
    let v2: i32 = ram_word!(game, eproj_id[16]) as i32;
    *(game.g_sram.offset((2i32 * v2) as isize) as *mut u16) = 0u16;
    *(game.g_sram.offset((2i32 * v2 + 8i32) as isize) as *mut u16) = 0u16;
    *(game.g_sram.offset((2i32 * v2 + 0x1ff0i32) as isize) as *mut u16) = 0u16;
    *(game.g_sram.offset((2i32 * v2 + 0x1ff8i32) as isize) as *mut u16) = 0u16;
    update_ram_word!(game.menu_index, wrapping_add(1));
    NewSaveFile(game);
    LoadFromSram(ram_word!(game, eproj_id[16]), game);
    set_ram_word!(game.area_index = ram_word!(game, eproj_id[16]));
    LoadMirrorOfExploredMapTiles(game);
    let mut v3: u16 = 640u16;
    loop {
        // File-clear rows share the full menu tilemap at $3600.
        *((&raw mut game.g_ram as *mut u8)
            .add(0x3600)
            .cast::<u16>()
            .add(v3 as usize)) = 15u16;
        v3 = v3.wrapping_add(1);
        if (((v3 as i32 * 2i32 - 1856i32) as i16) >= 0i16) {
            break;
        }
    }
    update_ram_word!(game.nonempty_save_slots, |fresh51| (fresh51)
        & (kBitShl_Not[ram_word!(game, eproj_id[16]) as usize]));
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    LoadMenuTilemap(0x500u16, addr_kMenuTilemap_DataCleared as u16, game);
    DrawFileCopyConfirmationSaveFileInfo(game);
    RtlWriteSram(game);
}
pub unsafe fn FileSelectMenu_26_ClearCompleted(game: &mut Game) {
    DrawBorderAroundDataClearMode(game);
    if ram_word!(game, joypad1_newkeys) != 0 {
        let use_first_slot: bool;
        QueueSfx1_Max6(0x37u16, game);
        update_ram_word!(game.menu_index, wrapping_add(1));
        if (LoadFromSram(0u16, game) != 0) {
            use_first_slot = if LoadFromSram(1u16, game) != 0 {
                (LoadFromSram(2u16, game) != 0)
                    || ({
                        set_ram_word!(game.selected_save_slot = 2u16);
                        false
                    })
            } else {
                set_ram_word!(game.selected_save_slot = 1u16);
                false
            };
            if !(use_first_slot) {
                return;
            }
        }
        set_ram_word!(game.selected_save_slot = 0u16);
    }
}
pub unsafe fn FileSelectMenu_30(game: &mut Game) {
    DrawBorderAroundSamusData(game);
    update_ram_word!(game.menu_index, wrapping_sub(26u16));
}
pub unsafe fn FileSelectMenu_31_TurnSamusHelmet(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    DrawBorderAroundSamusData(game);
    DrawFileSelectSlotSamusHelmet(4u16, game);
    DrawFileSelectSlotSamusHelmet(6u16, game);
    DrawFileSelectSlotSamusHelmet(8u16, game);
    if ram_word!(game, joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) != 0i32
        || ram_word!(game, eproj_id[2]) == 7u16 && ram_word!(game, cinematic_var7) == 0
        || ram_word!(game, eproj_id[3]) == 7u16 && ram_word!(game, cinematic_var8) == 0
        || ram_word!(game, eproj_id[4]) == 7u16 && ram_word!(game, cinematic_var9) == 0
    {
        update_ram_word!(game.menu_index, wrapping_add(1));
    }
}
pub unsafe fn DrawFileSelectSamusHelmets(game: &mut Game) {
    set_ram_word!(game.cinematic_var7 = 0u16);
    DrawFileSelectSlotSamusHelmet(4u16, game);
    set_ram_word!(game.cinematic_var8 = 0u16);
    DrawFileSelectSlotSamusHelmet(6u16, game);
    set_ram_word!(game.cinematic_var9 = 0u16);
    DrawFileSelectSlotSamusHelmet(8u16, game);
}
pub unsafe fn DrawFileSelectSlotSamusHelmet(k: u16, game: &mut Game) {
    let v0: u16 = k;
    let mut v1: i16 =
        *((ram_ptr!(game, u16; 0x198d) as *mut u8).offset(v0 as i32 as isize) as *mut u16) as i16;
    if v1 != 0 {
        let v2: u16 = ((v1) as u16).wrapping_sub(1u16);
        *((ram_ptr!(game, u16; 0x198d) as *mut u8).offset(v0 as i32 as isize) as *mut u16) = v2;
        if v2 == 0 {
            *((ram_ptr!(game, u16; 0x198d) as *mut u8).offset(v0 as i32 as isize) as *mut u16) =
                8u16;
            let v3: i32 = v0 as i32 >> 1i32;
            let mut v4: u16 = (ram_word!(game, eproj_id[v3])).wrapping_add(1u16);
            if ram_word!(game, eproj_id[v3]) as i32 - 7i32 & 0x8000i32 == 0 {
                *((ram_ptr!(game, u16; 0x198d) as *mut u8).offset(v0 as i32 as isize)
                    as *mut u16) = 0u16;
                v4 = 7u16;
            }
            set_ram_word!(game.eproj_id[v3] = v4);
        }
    }
    let v5: i32 = v0 as i32 >> 1i32;
    let v6: u16 = (2u16).wrapping_mul(ram_word!(game, eproj_id[v5]));
    static kDrawFileSlotHelmet_Spritemaps: [u16; 9] =
        [0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x33];
    DrawMenuSpritemap(
        kDrawFileSlotHelmet_Spritemaps[(v6 as i32 >> 1i32) as usize],
        ram_word!(game, eproj_id[(v5 + 5i32)]),
        ram_word!(game, eproj_id[(v5 + 10i32)]),
        3584u16,
        game,
    );
}
static kFileSelectMapFuncs: [Option<Func_V>; 23] = callback_table! { unsafe Func_V;
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
    kFileSelectMapFuncs[ram_word!(game, menu_index) as usize].expect("non-null function pointer")(
        game,
    );
}
pub unsafe fn FileSelectMap_22(game: &mut Game) {
    DrawAreaSelectMapLabels(game);
    HandleFadeOut(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
        set_ram_word!(game.game_state = kGameState_2_GameOptionsMenu as u16);
        set_ram_word!(game.menu_index = 0u16);
    }
}
pub unsafe fn FileSelectMenu_1_LoadFileSelectMenuBG2(game: &mut Game) {
    // This menu spans $3600..$3dff, beyond the pause-map union field.
    let menu_tilemap = (&raw mut game.g_ram as *mut u8).add(0x3600).cast::<u16>();
    let mut i: i32 = 1023i32;
    while i >= 0i32 {
        *menu_tilemap.add(i as usize) = crate::rtl::rom_table_word!(game, 0x8edc00, i as isize);
        i -= 1;
    }
    let v1: u16 = ram_word!(game, vram_write_queue_tail);
    let mut v2 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
    (*v2).size = 2048u16;
    (*v2).src.addr = (&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).menu.menu_tilemap as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut u16) = 126u16;
    (*v2).vram_dst = ((*ram_ptr!(game; 0x59) as i32 & 0xfci32) << 8i32) as u16;
    set_ram_word!(game.vram_write_queue_tail = (v1).wrapping_add(7u16));
    update_ram_word!(game.menu_index, wrapping_add(1));
    set_ram_word!(game.cinematic_var5 = 1u16);
    set_ram_word!(game.eproj_id[0] = 0u16);
}
pub unsafe fn FileSelectMenu_2_InitMain(game: &mut Game) {
    let mut v0: u16 = 0;
    if *(game.g_sram.offset(0x1fec) as *mut u16) & 0x8000 != 0
        || *(game.g_sram.offset(0x1fec) as *mut u16) as i32 - 3i32 & 0x8000i32 == 0
        || {
            v0 = *(game.g_sram.offset(0x1fec) as *mut u16);
            (*(game.g_sram.offset(0x1fee) as *mut u16) & *(game.g_sram.offset(0x1fec) as *mut u16))
                != 0
        }
    {
        v0 = 0u16;
    }
    set_ram_word!(game.selected_save_slot = v0);
    FileSelectMenu_16(game);
}
pub unsafe fn FileSelectMenu_16(game: &mut Game) {
    let mut i: i32 = 2046i32;
    while i >= 0i32 {
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(768) as *mut u8)
            .offset(i as u16 as i32 as isize) as *mut u16) = 15u16;
        i -= 2i32;
    }
    set_ram_word!(game.nonempty_save_slots = -1i32 as u16);
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    LoadMenuTilemap(0x56u16, addr_kMenuTilemap_SamusData as u16, game);
    LoadMenuTilemap(0x146u16, addr_kMenuTilemap_SamusA as u16, game);
    let mut c: u8 = LoadFromSram_(0u16, game);
    set_ram_word!(
        game.nonempty_save_slots =
            (ram_word!(game, nonempty_save_slots) as i32 >> 1i32 | (c as i32) << 15i32) as u16
    );
    DrawFileSelectionHealth(
        (ram_word!(game, nonempty_save_slots)) & (0x8000u16),
        0x15cu16,
        game,
    );
    DrawFileSelectionTime(
        (ram_word!(game, nonempty_save_slots)) & (0x8000u16),
        0x1b4u16,
        game,
    );
    LoadMenuTilemap(0x176u16, addr_kMenuTilemap_TIME as u16, game);
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    LoadMenuTilemap(0x286u16, addr_kMenuTilemap_SamusB as u16, game);
    c = LoadFromSram_(1u16, game);
    set_ram_word!(
        game.nonempty_save_slots =
            (ram_word!(game, nonempty_save_slots) as i32 >> 1i32 | (c as i32) << 15i32) as u16
    );
    DrawFileSelectionHealth(
        (ram_word!(game, nonempty_save_slots)) & (0x8000u16),
        0x29cu16,
        game,
    );
    DrawFileSelectionTime(
        (ram_word!(game, nonempty_save_slots)) & (0x8000u16),
        0x2f4u16,
        game,
    );
    LoadMenuTilemap(0x2b6u16, addr_kMenuTilemap_TIME as u16, game);
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    LoadMenuTilemap(0x3c6u16, addr_kMenuTilemap_SamusC as u16, game);
    c = LoadFromSram_(2u16, game);
    set_ram_word!(
        game.nonempty_save_slots =
            (ram_word!(game, nonempty_save_slots) as i32 >> 1i32 | (c as i32) << 15i32) as u16
    );
    DrawFileSelectionHealth(
        (ram_word!(game, nonempty_save_slots)) & (0x8000u16),
        0x3dcu16,
        game,
    );
    DrawFileSelectionTime(
        (ram_word!(game, nonempty_save_slots)) & (0x8000u16),
        0x434u16,
        game,
    );
    LoadMenuTilemap(0x3f6u16, addr_kMenuTilemap_TIME as u16, game);
    set_ram_word!(
        game.nonempty_save_slots =
            (u16::swap_bytes(!(ram_word!(game, nonempty_save_slots) as i32) as u16) as i32 >> 5i32)
                as u16
    );
    if ram_word!(game, nonempty_save_slots) != 0 {
        (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
        LoadMenuTilemap(0x508u16, addr_kMenuTilemap_DataCopy as u16, game);
        (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
        LoadMenuTilemap(0x5c8u16, addr_kMenuTilemap_DataClear as u16, game);
    }
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index = 0u16;
    LoadMenuTilemap(0x688u16, addr_kMenuTilemap_Exit as u16, game);
    QueueTransferOfMenuTilemapToVramBG1(game);
    set_ram_word!(game.cinematic_var5 = 1u16);
    set_ram_words! { game, 0u16;
        cinematic_var6, cinematic_var7, cinematic_var8, cinematic_var9, eproj_id[0], eproj_id[1],
        eproj_id[2], eproj_id[3], eproj_id[4], eproj_id[5], eproj_id[10], eproj_id[6], eproj_id[11],
    }
    set_ram_words! { game, 100u16; eproj_id[7], eproj_id[8], eproj_id[9] }
    set_ram_word!(game.eproj_id[12] = 47u16);
    set_ram_word!(game.eproj_id[13] = 87u16);
    set_ram_word!(game.eproj_id[14] = 127u16);
    set_ram_words! { game, 1u16; screen_fade_delay, screen_fade_counter }
    ScreenOn(game);
    update_ram_word!(game.menu_index, wrapping_add(1));
    set_ram_words! { game, 0u16; eproj_id[15], eproj_id[16], eproj_id[17] }
}
pub unsafe fn LoadFromSram_(a: u16, game: &mut Game) -> u8 {
    LoadFromSram(a, game)
}
pub unsafe fn FileSelectMenu_3_FadeInToMain(game: &mut Game) {
    DrawFileSelectSamusHelmets(game);
    let v0: i32 = ((4u16).wrapping_mul(ram_word!(game, selected_save_slot))) as i32 >> 1i32;
    set_ram_word!(game.eproj_id[10] = kMenuSelectionMissileXY[v0 as usize]);
    set_ram_word!(game.eproj_id[5] = kMenuSelectionMissileXY[(v0 + 1i32) as usize]);
    DrawMenuSelectionMissile(game);
    DrawBorderAroundSamusData(game);
    HandleFadeIn(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 15 {
        update_ram_word!(game.menu_index, wrapping_add(1));
    }
}
pub unsafe fn DrawFileSelectionHealth(a: u16, k: u16, game: &mut Game) {
    if a != 0 {
        FileSelectClearRestOfMenuTilemapRow(k, game);
        LoadMenuTilemap(
            (k).wrapping_add(64u16),
            addr_kMenuTilemap_NoData as u16,
            game,
        );
    } else {
        LoadMenuTilemap(k, addr_kMenuTilemap_Energy as u16, game);
        let mut v2: i16 = ((k) as i16).wrapping_add(8i16);
        let mut n: i32 = ram_word!(game, samus_health) as i32 / 100i32;
        let mut q: i32 = ram_word!(game, samus_health) as i32 % 100i32;
        let mut mh: i32 = ram_word!(game, samus_max_health) as i32 / 100i32;
        let mut m: i32 = 7i32;
        let mut v3: u16 = ((v2) as u16).wrapping_add(64u16);
        loop {
            mh -= 1;
            if (mh < 0i32) {
                break;
            }
            let mut v4: i16 = 153i16;
            if n != 0 {
                n -= 1;
                v4 = 152i16;
            }
            *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
                .offset(768) as *mut u8)
                .offset(v3 as i32 as isize) as *mut u16) =
                ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index) | ((v4) as u16);
            v3 = (v3).wrapping_add(2u16);
            m -= 1;
            if m == 0 {
                v3 = (v3).wrapping_sub(78u16);
                m = 8i32;
            }
        }
        n = q / 10i32;
        q = q % 10i32;
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(802) as *mut u8)
            .offset(k as i32 as isize) as *mut u16) =
            ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index as i32 | q + 8288i32)
                as u16;
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(801) as *mut u8)
            .offset(k as i32 as isize) as *mut u16) =
            ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index as i32 | n + 8288i32)
                as u16;
    };
}
pub unsafe fn DrawFileSelectionTime(a: u16, k: u16, game: &mut Game) {
    if a == 0 {
        let r26: u16 = k;
        let div_val: i32 = ram_word!(game, game_time_hours) as i32 / 10i32;
        let mod_val: i32 = ram_word!(game, game_time_hours) as i32 % 10i32;
        let v2: u16 = r26;
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(769) as *mut u8)
            .offset(v2 as i32 as isize) as *mut u16) =
            ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index as i32
                | mod_val + 8288i32) as u16;
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(768) as *mut u8)
            .offset(v2 as i32 as isize) as *mut u16) =
            ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index as i32
                | div_val + 8288i32) as u16;
        LoadMenuTilemap((r26).wrapping_add(4u16), addr_word_81B4A8 as u16, game);
        let div_min: i32 = ram_word!(game, game_time_minutes) as i32 / 10i32;
        let mod_min: i32 = ram_word!(game, game_time_minutes) as i32 % 10i32;
        let v3: u16 = r26;
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(772) as *mut u8)
            .offset(v3 as i32 as isize) as *mut u16) =
            ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index as i32
                | mod_min + 8288i32) as u16;
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(771) as *mut u8)
            .offset(v3 as i32 as isize) as *mut u16) =
            ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index as i32
                | div_min + 8288i32) as u16;
    }
}
pub unsafe fn FileSelectMenu_4_Main(game: &mut Game) {
    let selection_unchanged: bool;
    let mut v1: u16 = 0;
    let mut v2: u16 = 0;
    DrawBorderAroundSamusData(game);
    DrawMenuSelectionMissile(game);
    DrawFileSelectSlotSamusHelmet(4u16, game);
    DrawFileSelectSlotSamusHelmet(6u16, game);
    DrawFileSelectSlotSamusHelmet(8u16, game);
    if ram_word!(game, joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32) == 0i32 {
        selection_unchanged =
            if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Up as i32) != 0i32 {
                if ram_word!(game, nonempty_save_slots) != 0 {
                    v1 = (ram_word!(game, selected_save_slot)).wrapping_sub(1u16);
                    if ((ram_word!(game, selected_save_slot)) as i16).wrapping_sub(1i16) < 0i16 {
                        v1 = 5u16;
                    }
                } else {
                    v1 = (ram_word!(game, selected_save_slot)).wrapping_sub(1u16);
                    if ((ram_word!(game, selected_save_slot)) as i16).wrapping_sub(1i16) >= 0i16 {
                        if ram_word!(game, selected_save_slot) as i32 - 5i32 & 0x8000i32 == 0 {
                            v1 = 2u16;
                        }
                    } else {
                        v1 = 5u16;
                    }
                }
                set_ram_word!(game.selected_save_slot = v1);
                false
            } else if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Down as i32) == 0i32 {
                if (ram_word!(game, joypad1_newkeys) as i32 & kButton_B as i32) != 0i32 {
                    QueueSfx1_Max6(0x37u16, game);
                    set_ram_word!(game.menu_index = 33u16);
                    QueueSfx1_Max6(0x37u16, game);
                }
                true
            } else {
                if ram_word!(game, nonempty_save_slots) != 0 {
                    v2 = (ram_word!(game, selected_save_slot)).wrapping_add(1u16);
                    if ram_word!(game, selected_save_slot) as i32 - 5i32 & 0x8000i32 == 0 {
                        v2 = 0u16;
                    }
                } else {
                    v2 = (ram_word!(game, selected_save_slot)).wrapping_add(1u16);
                    if ram_word!(game, selected_save_slot) as i32 - 2i32 & 0x8000i32 == 0 {
                        if ram_word!(game, selected_save_slot) as i32 - 5i32 & 0x8000i32 != 0 {
                            v2 = 5u16;
                        } else {
                            v2 = 0u16;
                        }
                    }
                }
                set_ram_word!(game.selected_save_slot = v2);
                false
            };
        if !(selection_unchanged) {
            QueueSfx1_Max6(0x37u16, game);
        }
    } else {
        let mut v0: i16 = ram_word!(game, selected_save_slot) as i16;
        if ram_word!(game, selected_save_slot) as i32 - 3i32 & 0x8000i32 != 0 {
            QueueSfx1_Max6(0x2au16, game);
            update_ram_word!(game.menu_index, wrapping_add(27u16));
            *((ram_ptr!(game, u16; 0x198d) as *mut u8).offset(
                (2i32 * (ram_word!(game, selected_save_slot) as i32 + 2i32)) as u16 as i32 as isize,
            ) as *mut u16) = 1u16;
            *(game.g_sram.offset(0x1fec) as *mut u16) = ram_word!(game, selected_save_slot);
            *(game.g_sram.offset(0x1fee) as *mut u16) =
                !(ram_word!(game, selected_save_slot) as i32) as u16;
            RtlWriteSram(game);
            if LoadFromSram(ram_word!(game, selected_save_slot), game) != 0 {
                NewSaveFile(game);
                set_ram_word!(game.has_area_map = 0u16);
            } else {
                LoadMirrorOfExploredMapTiles(game);
            }
        } else {
            if ram_word!(game, selected_save_slot) == 3u16 {
                QueueSfx1_Max6(0x37u16, game);
                update_ram_word!(game.menu_index, wrapping_add(1));
                *(ram_ptr!(game, u16; 0x57)) = (*(ram_ptr!(game, u16; 0x57)) & 0xff0c | 3);
            } else {
                if ram_word!(game, selected_save_slot) == 4u16 {
                    QueueSfx1_Max6(0x37u16, game);
                    update_ram_word!(game.menu_index, wrapping_add(15u16));
                    v0 = (*(ram_ptr!(game, u16; 0x57)) as i32 & 0xff0ci32 | 3i32) as i16;
                    *(ram_ptr!(game, u16; 0x57)) = v0 as u16;
                }
                if v0 == 5i16 {
                    set_ram_word!(game.menu_index = 33u16);
                }
            }
            return;
        }
    }
    let mut v3: i32 = ((4u16).wrapping_mul(ram_word!(game, selected_save_slot))) as i32 >> 1i32;
    set_ram_word!(game.eproj_id[10] = kMenuSelectionMissileXY[v3 as usize]);
    set_ram_word!(game.eproj_id[5] = kMenuSelectionMissileXY[(v3 + 1i32) as usize]);
}
pub unsafe fn FileSelectMap_0(game: &mut Game) {
    ClearMenuTilemap(game);
    let v0: u16 = ram_word!(game, vram_write_queue_tail);
    let mut v1 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
    (*v1).size = 2048u16;
    (*v1).src.addr = (&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).menu.menu_tilemap as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as VoidP;
    *(&raw mut (*v1).src.bank as *mut u16) = 126u16;
    (*v1).vram_dst = ((*ram_ptr!(game; 0x59) as i32 & 0xfci32) << 8i32) as u16;
    set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(7u16));
    set_ram_word!(game.palette_change_num = 0u16);
    LoadFileSelectPalettes(game);
    let mut v2: u16 = 0u16;
    loop {
        set_ram_word!(
            game.target_palettes[(v2 as i32 >> 1i32)] =
                ram_word!(game, palette_buffer[(v2 as i32 >> 1i32)])
        );
        v2 = (v2).wrapping_add(2u16);
        if (((v2) as i16).wrapping_sub(64i16) >= 0i16) {
            break;
        }
    }
    set_ram_words! { game, 0u16; target_palettes[14], target_palettes[30] }
    update_ram_word!(game.menu_index, wrapping_add(1));
}
pub unsafe fn FileSelectMap_1(game: &mut Game) {
    if AdvanceGradualColorChangeOfPalette(0u16, 0x40u16, game) != 0 {
        set_ram_words! { game, 0u16; reg_BG1VOFS, reg_BG1HOFS, reg_BG2VOFS, reg_BG2HOFS }
        let mut v0: u16 = 0u16;
        while ram_word!(game, area_index)
            != kFileSelectMap_AreaIndexes[(v0 as i32 >> 1i32) as usize]
        {
            v0 = (v0).wrapping_add(2u16);
            if (((v0) as i16).wrapping_sub(12i16) < 0i16) {
                continue;
            }
            v0 = 0u16;
            break;
        }
        set_ram_word!(game.file_select_map_area_index = (v0 as i32 >> 1i32) as u16);
        let mut v1: i16 = 0i16;
        loop {
            let mut v2: i16 = v1;
            if v1 as i32 == ram_word!(game, area_index) as i32 {
                LoadActiveAreaMapForegroundColors(v1 as u16, game);
            } else {
                LoadInactiveAreaMapForegroundColors(v1 as u16, game);
            }
            v1 += 1;
            if ((v2).wrapping_sub(5i16) >= 0i16) {
                break;
            }
        }
        update_ram_word!(game.menu_index, wrapping_add(1));
        set_ram_word!(game.g_word_7E0787 = 0u16);
        *ram_ptr!(game; 0x69) = 2u8;
    }
}
pub unsafe fn sub_81A3D1(k: u16, game: &mut Game) {
    LoadActiveAreaMapForegroundColors(k, game);
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
        let v1: i32 = j as i32 >> 1i32;
        if crate::rtl::rom_table_word!(game, 0x81a4e6, v1 as isize) == 0xffffu16 {
            break;
        }
        let v4: u16 = j;
        let mut v2: u16 = crate::rtl::rom_table_word!(game, 0x81a4e6, (v1 + 1i32) as isize);
        let mut v3: u16 = crate::rtl::rom_table_word!(game, 0x81a4e6, v1 as isize);
        for _ in 0i32..5i32 {
            set_ram_word!(
                game.palette_buffer[(v2 as i32 >> 1i32)] =
                    crate::rtl::rom_table_word!(game, 0x81a40e, (v3 as i32 >> 1i32) as isize)
            );
            v2 = (v2).wrapping_add(2u16);
            v3 = (v3).wrapping_add(2u16);
        }
        j = (v4).wrapping_add(4u16);
    }
}
pub unsafe fn FileSelectMap_2_LoadAreaSelectForegroundTilemap(game: &mut Game) {
    let v0: u16 = ram_word!(game, vram_write_queue_tail);
    let mut v1 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
    (*v1).size = 2048u16;
    (*v1).src.addr = addr_kAreaSelectForegroundTilemap as i32 as VoidP;
    *(&raw mut (*v1).src.bank as *mut u16) = 129u16;
    (*v1).vram_dst = ((*ram_ptr!(game; 0x58) as i32 & 0xfci32) << 8i32) as u16;
    set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(7u16));
    *(ram_ptr!(game, u16; 0x51)) = ((*ram_ptr!(game; 0x52) as i32) << 8i32 | 0xfi32) as u16;
    update_ram_word!(game.menu_index, wrapping_add(1));
}
pub unsafe fn FileSelectMap_18(game: &mut Game) {
    update_ram_word!(game.menu_index, wrapping_add(1));
    LoadAreaSelectBackgroundTilemap(ram_word!(game, area_index), game);
}
pub unsafe fn FileSelectMap_3_LoadAreaSelectBackgroundTilemap(game: &mut Game) {
    update_ram_word!(game.menu_index, wrapping_add(1));
    LoadAreaSelectBackgroundTilemap(ram_word!(game, area_index), game);
}
pub unsafe fn LoadAreaSelectBackgroundTilemap(j: u16, game: &mut Game) {
    let v1: u16 = ram_word!(game, vram_write_queue_tail);
    let v2 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
    (*v2).size = 2048u16;
    (*v2).src.addr =
        (8i32 * 256i32 * j as i32 + addr_kAreaSelectBackgroundTilemaps as i32) as VoidP;
    (*v2).src.bank = 0x81u8;
    (*v2).vram_dst = ((*ram_ptr!(game; 0x5a) as i32 & 0xfci32) << 8i32) as u16;
    set_ram_word!(game.vram_write_queue_tail = (v1).wrapping_add(7u16));
}
pub unsafe fn FileSelectMap_4_SetupExpandingSquareTransition(game: &mut Game) {
    *ram_ptr!(game; 0x6b) = 4u8;
    crate::rtl::register_writes! { game;
        mirror(TS as u16, 4, 0x6d, false); byte(TSW as u16, 4u8);
        mirror(CGWSEL as u16, 2, 0x6f, true); mirror(CGADSUB as u16, 0x25, 0x72, true);
    }
    *ram_ptr!(game; 0x74, 2) = 0x80u8;
    *ram_ptr!(game; 0x74, 1) = 64u8;
    *ram_ptr!(game; 0x74, 0) = 32u8;
    SetupInitialExpandingSquareHDMA(game);
    crate::rtl::write_register_and_mirror(HDMAEN as u16, 0xc, 0x85, true, game);
    ConfigureWindow1ForExpandingSquare(game);
    set_ram_word!(game.hdma_objects_enable_flag = 0u16);
    *(ram_ptr!(game, u16; 0x727) as *mut u8).offset(0) =
        ((ram_word!(game, menu_index)) as u8).wrapping_add(1u8);
}
pub unsafe fn ConfigureWindow1ForExpandingSquare(game: &mut Game) {
    *ram_ptr!(game; 0x69) = 19u8;
    crate::rtl::register_writes! { game;
        mirror(TM as u16, 0x13, 0x6c, false); byte(TMW as u16, 0x13u8);
        mirror(W12SEL as u16, 0x23, 0x60, true); mirror(W34SEL as u16, 3, 0x61, true);
        mirror(WOBJSEL as u16, 0x23, 0x62, true);
    }
}
pub unsafe fn SetupInitialExpandingSquareHDMA(game: &mut Game) {
    set_ram_word!(game.expand_sq_topbottom_margin_right_pos = 0u16);
    set_ram_word!(game.expand_sq_topbottom_margin_left_pos = -1i32 as u16);
    *(ram_ptr!(game, u16; 0x9e32) as *mut u8).offset(0) = 127u8;
    *(ram_ptr!(game, u16; 0x9e36) as *mut u8).offset(0) = -127i32 as u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(0)).field_0 = 111u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(0)).field_1 = 34u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(0)).field_2 = -98i32 as u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_0 = 1u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_1 = 50u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_2 = -98i32 as u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(2)).field_0 = 1u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(2)).field_1 = 50u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(2)).field_2 = -98i32 as u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(3)).field_0 = 111u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(3)).field_1 = 32u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(3)).field_2 = -98i32 as u8;
    crate::rtl::register_writes! { game;
        byte(DMAP2 as u16, 0x40u8); byte(BBAD2 as u16, 0x26u8);
        byte(A1T2L as u16, 0u8); byte(A1T2H as u16, 0x9eu8);
        byte(A1B2 as u16, 0x7eu8); byte(DAS20 as u16, 0x7eu8);
        byte(DAS2L as u16, 0u8); byte(DAS2H as u16, 0u8);
        byte(A2A2L as u16, 0u8); byte(A2A2H as u16, 0u8);
        byte(NTRL2 as u16, 0u8);
    }
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(0)).field_0 = 111u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(0)).field_1 = 32u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(0)).field_2 = -98i32 as u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(1)).field_0 = 1u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(1)).field_1 = 54u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(1)).field_2 = -98i32 as u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(2)).field_0 = 1u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(2)).field_1 = 54u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(2)).field_2 = -98i32 as u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(3)).field_0 = 111u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(3)).field_1 = 32u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(3)).field_2 = -98i32 as u8;
    crate::rtl::register_writes! { game;
        byte(DMAP3 as u16, 0x40u8); byte(BBAD3 as u16, 0x27u8);
        byte(A1T3L as u16, 0x10u8); byte(A1T3H as u16, 0x9eu8);
        byte(A1B3 as u16, 0x7eu8); byte(DAS30 as u16, 0x7eu8);
        byte(DAS3L as u16, 0u8); byte(DAS3H as u16, 0u8);
        byte(A2A3L as u16, 0u8); byte(A2A3H as u16, 0u8);
        byte(NTRL3 as u16, 0u8);
    }
}
static kExpandingSquareTransitionSpeed: u8 = 4u8;
pub unsafe fn FileSelectMap_5_ExpandingSquare(game: &mut Game) {
    DrawAreaSelectMapLabels(game);
    let v0: u8 = ((*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(0)).field_0)
        .wrapping_sub(kExpandingSquareTransitionSpeed);
    if (((*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(0)).field_0) as i8)
        .wrapping_sub((kExpandingSquareTransitionSpeed) as i8)
        < 0i8
    {
        *(ram_ptr!(game, u16; 0x727) as *mut u8).offset(0) =
            ((ram_word!(game, menu_index)) as u8).wrapping_add(1u8);
        let ref mut fresh36 = *ram_ptr!(game; 0x69);
        *fresh36 = (*fresh36 & !(2));
        *ram_ptr!(game; 0x6c) = 0u8;
        *ram_ptr!(game; 0x6d) = 0u8;
        set_ram_word!(game.reg_BG2VOFS = 24u16);
        set_ram_word!(game.pausemenu_palette_animation_timer = 1u16);
        let mut v1: u16 = 0u16;
        loop {
            (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps[v1 as usize] =
                crate::rtl::rom_table_word!(game, 0xb6e000, v1 as isize);
            v1 = v1.wrapping_add(1);
            if (((v1 as i32 * 2i32 - 1600i32) as i16) >= 0i16) {
                break;
            }
        }
        loop {
            let fresh37 = v1;
            v1 = v1.wrapping_add(1);
            (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps[fresh37 as usize] = 10241u16;
            if (((v1 as i32 * 2i32 - 2048i32) as i16) >= 0i16) {
                break;
            }
        }
        DrawRoomSelectMapAreaLabel(
            (&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).bg2_room_select_map_tilemap as *mut u16)
                .offset(170),
            game,
        );
        let mut v2: u16 = 320u16;
        let mut v3: u16 = 959u16;
        loop {
            let fresh38 = v3;
            v3 = v3.wrapping_sub(1);
            (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps[fresh38 as usize] =
                crate::rtl::rom_table_word!(game, 0x81b14b, (v2 as i32 >> 1i32) as isize);
            v2 = (v2).wrapping_sub(2u16);
            if (v2 == 0) {
                break;
            }
        }
        let v4: u16 = ram_word!(game, vram_write_queue_tail);
        let mut v5 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
        (*v5).size = 2048u16;
        (*v5).src.addr = (ram_ptr!(game, Ram4000; 0x4000) as *mut u8)
            .offset_from(&raw mut game.g_ram as *mut u8) as c_long
            as VoidP;
        (*v5).src.bank = 126u8;
        (*v5).vram_dst = ((*ram_ptr!(game; 0x59) as i32 & 0xfci32) << 8i32) as u16;
        set_ram_word!(game.vram_write_queue_tail = (v4).wrapping_add(7u16));
    } else {
        let ref mut fresh39 =
            (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(0)).field_0;
        *fresh39 = (*fresh39).wrapping_sub(kExpandingSquareTransitionSpeed);
        (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(3)).field_0 = v0;
        (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(0)).field_0 = v0;
        (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(3)).field_0 = v0;
        let ref mut fresh40 =
            (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_0;
        *fresh40 = (*fresh40).wrapping_add(kExpandingSquareTransitionSpeed);
        (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(2)).field_0 =
            (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_0;
        (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(1)).field_0 =
            (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_0;
        (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(2)).field_0 =
            (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_0;
        *(ram_ptr!(game, u16; 0x9e32) as *mut u8).offset(0) =
            ((ram_word!(game, expand_sq_left_pos)) as u8)
                .wrapping_sub(kExpandingSquareTransitionSpeed);
        *(ram_ptr!(game, u16; 0x9e36) as *mut u8).offset(0) = (kExpandingSquareTransitionSpeed)
            .wrapping_add((ram_word!(game, expand_sq_right_pos)) as u8);
    };
}
pub unsafe fn FileSelectMap_6_AreaSelectMap(game: &mut Game) {
    let current_block: u64;
    let mut v0: i16 = ram_word!(game, joypad1_newkeys) as i16;
    if ram_word!(game, joypad1_newkeys) as i32 & (kButton_Up as i32 | kButton_Left as i32) != 0i32 {
        v0 = ram_word!(game, enable_debug) as i16;
        if ram_word!(game, enable_debug) != 0 {
            let mut R28: u16 = DecAndWraparoundTo5(ram_word!(game, file_select_map_area_index));
            if CheckIfFileSelectMapAreaCanBeSelected(R28, game) != 0u16
                || {
                    R28 = DecAndWraparoundTo5(R28);
                    CheckIfFileSelectMapAreaCanBeSelected(R28, game) != 0u16
                }
                || {
                    R28 = DecAndWraparoundTo5(R28);
                    CheckIfFileSelectMapAreaCanBeSelected(R28, game) != 0u16
                }
            {
                QueueSfx1_Max6(0x37u16, game);
                SwitchActiveFileSelectMapArea(R28, game);
                DrawAreaSelectMapLabels(game);
                return;
            }
            current_block = 4115184416633590978;
        } else {
            current_block = 9480232856434185415;
        }
    } else if ram_word!(game, joypad1_newkeys) as i32
        & (kButton_Select as i32 | kButton_Down as i32 | kButton_Right as i32)
        == 0i32
    {
        current_block = 9480232856434185415;
    } else {
        v0 = ram_word!(game, enable_debug) as i16;
        if ram_word!(game, enable_debug) == 0 {
            current_block = 9480232856434185415;
        } else {
            let mut n: i32 = 6i32;
            let mut R28_0: u16 = ram_word!(game, file_select_map_area_index);
            loop {
                R28_0 = WraparoundFrom6to0(R28_0);
                if CheckIfFileSelectMapAreaCanBeSelected(R28_0, game) != 0 {
                    current_block = 11194104282611034094;
                    break;
                }
                n -= 1;
                if n == 0 {
                    current_block = 4115184416633590978;
                    break;
                }
            }
            if current_block == 4115184416633590978 {
            } else {
                SwitchActiveFileSelectMapArea(R28_0, game);
                QueueSfx1_Max6(0x37u16, game);
                DrawAreaSelectMapLabels(game);
                return;
            }
        }
    }
    if current_block == 9480232856434185415 {
        if v0 < 0i16 {
            set_ram_word!(game.menu_index = 22u16);
            DrawAreaSelectMapLabels(game);
            return;
        }
        if v0 as i32 & (kButton_Start as i32 | kButton_A as i32) != 0i32 {
            QueueSfx1_Max6(0x38u16, game);
            SelectFileSelectMapArea(game);
            return;
        }
    }
    DrawAreaSelectMapLabels(game);
}
pub unsafe fn DecAndWraparoundTo5(a: u16) -> u16 {
    let result: u16 = (a).wrapping_sub(1u16);
    if (result & 0x8000) != 0 {
        return 5u16;
    }
    result
}
pub unsafe fn WraparoundFrom6to0(a: u16) -> u16 {
    let result: u16 = (a).wrapping_add(1u16);
    if result as i32 - 6i32 & 0x8000i32 == 0 {
        return 0u16;
    }
    result
}
pub unsafe fn SelectFileSelectMapArea(game: &mut Game) {
    update_ram_word!(game.menu_index, wrapping_add(1));
    if ram_word!(game, enable_debug) == 0 {
        set_ram_word!(game.area_index = ram_word!(game, sram_area_index));
        set_ram_word!(game.load_station_index = ram_word!(game, sram_save_station_index));
        DrawAreaSelectMapLabels(game);
        return;
    }
    set_ram_word!(
        game.area_index =
            kFileSelectMap_AreaIndexes[ram_word!(game, file_select_map_area_index) as usize]
    );
    let mut q: u16 =
        *(ram_ptr!(game, u16; 0xd8f8, (2i32 * ram_word!(game, area_index) as i32) as isize));
    let mut v0: u16 = 0u16;
    let r0 = RomPtr_82(
        *(RomPtr_82(
            (addr_kMapIconDataPointers as i32 + 64i32 + 2i32 * ram_word!(game, area_index) as i32)
                as u16,
            game,
        ) as *mut u16),
        game,
    ) as *const u16;
    let mut r20: u16 = 16u16;
    's_43: loop {
        let mut v2: i8 = ((q) as i8) & (1i8);
        q = (q as i32 >> 1i32) as u16;
        if (v2 != 0) {
            let mut v3: i16 = *r0.offset((2i32 * v0 as i32) as isize) as i16;
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
            let mut v4: i16 = *r0.offset((2i32 * v0 as i32) as isize) as i16;
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
    set_ram_word!(game.load_station_index = v0);
    DrawAreaSelectMapLabels(game);
}
pub unsafe fn CheckIfFileSelectMapAreaCanBeSelected(a: u16, game: &mut Game) -> u16 {
    let v1: i32 = 2i32 * kFileSelectMap_AreaIndexes[a as usize] as i32;
    if *(ram_ptr!(game, u16; 0xd8f8, v1 as isize)) != 0 {
        return true_0 as u16;
    }
    let t: u16 = (*((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c7cb))
        as *mut MapIconDataPointers)
        .offset(4))
    .crateria as *mut u8)
        .offset(v1 as isize) as *mut u16))
        .wrapping_add(64u16);
    (t != 0xffffu16) as u16
}
pub unsafe fn SwitchActiveFileSelectMapArea(R28: u16, game: &mut Game) {
    LoadInactiveAreaMapForegroundColors(
        kFileSelectMap_AreaIndexes[ram_word!(game, file_select_map_area_index) as usize],
        game,
    );
    set_ram_word!(game.file_select_map_area_index = R28);
    LoadActiveAreaMapForegroundColors(kFileSelectMap_AreaIndexes[R28 as usize], game);
    LoadAreaSelectBackgroundTilemap(
        kFileSelectMap_AreaIndexes[ram_word!(game, file_select_map_area_index) as usize],
        game,
    );
}
static kAreaSelectMapLabelPositions: [u16; 12] = [
    0x5b, 0x32, 0x2a, 0x7f, 0x5e, 0xb5, 0xce, 0x50, 0xce, 0x9f, 0x87, 0x8b,
];
pub unsafe fn DrawAreaSelectMapLabels(game: &mut Game) {
    let mut r3: u16 = 0u16;
    DrawMenuSpritemap(
        crate::rtl::rom_table_word!(game, 0x82c749, 0),
        0x80u16,
        0x10u16,
        r3,
        game,
    );
    let mut i: i32 = 0i32;
    while i < 6i32 {
        r3 = (if i == ram_word!(game, file_select_map_area_index) as i32 {
            0i32
        } else {
            512i32
        }) as u16;
        let v1: u16 = (2u16).wrapping_mul(kFileSelectMap_AreaIndexes[i as usize]);
        let mut r36: u16 = *(ram_ptr!(game, u16; 0xd8f8, v1 as isize));
        let mut v2 = RomPtr_82(
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c7cb))
                as *mut MapIconDataPointers)
                .offset(4))
            .crateria as *mut u8)
                .offset(v1 as i32 as isize) as *mut VoidP),
            game,
        ) as *const u16;
        let mut R30: i32 = 16i32;
        while *v2 != 0xffffu16 {
            let v4: i32 = r36 as i32 & 1i32;
            r36 = (r36 as i32 >> 1i32) as u16;
            if !(v4 != 0 && *v2 != 0xfffeu16) {
                v2 = v2.offset(2);
                R30 -= 1;
                if (R30 != 0) {
                    continue;
                }
                if !(ram_word!(game, enable_debug) != 0u16 && *v2 != 0xffffu16) {
                    break;
                }
            }
            let j: i32 = 4i32 * kFileSelectMap_AreaIndexes[i as usize] as i32 >> 1i32;
            DrawMenuSpritemap(
                (crate::rtl::rom_table_word!(game, 0x82c749, 0) as i32
                    + kFileSelectMap_AreaIndexes[i as usize] as i32
                    + 1i32) as u16,
                kAreaSelectMapLabelPositions[j as usize],
                kAreaSelectMapLabelPositions[(j + 1i32) as usize],
                r3,
                game,
            );
            break;
        }
        i += 1;
    }
}
static kRoomSelectMapExpandingSquareTimers: [u16; 6] = [0x33, 0x35, 0x2d, 0x33, 0x33, 0x22];
pub unsafe fn FileSelectMap_7_PrepExpandSquareTransToRoomMap(game: &mut Game) {
    DrawAreaSelectMapLabels(game);
    SetupInitialExpandingSquareHDMA(game);
    *ram_ptr!(game; 0x69) = 19u8;
    *ram_ptr!(game; 0x6c) = 19u8;
    crate::rtl::register_writes! { game;
        mirror(W12SEL as u16, 0x32, 0x60, true); mirror(W34SEL as u16, 2, 0x61, true);
        mirror(CGADSUB as u16, 5, 0x72, true); mirror(WOBJSEL as u16, 0x22, 0x62, true);
    }
    *ram_ptr!(game; 0x5d) = 48u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(3)).field_0 = 0u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(3)).field_0 = 0u8;
    DrawRoomSelectMapAreaLabel(
        (&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).bg2_room_select_map_tilemap as *mut u16)
            .offset(170),
        game,
    );
    let v0: u16 = ram_word!(game, vram_write_queue_tail);
    let v1 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
    (*v1).size = 512u16;
    (*v1).src.addr = (ram_ptr!(game, Ram4000; 0x4000) as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as VoidP;
    (*v1).src.bank = 126u8;
    (*v1).vram_dst = ((*ram_ptr!(game; 0x59) as i32 & 0xfci32) << 8i32) as u16;
    set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(7u16));
    set_ram_word!(
        game.expand_sq_timer =
            kRoomSelectMapExpandingSquareTimers[ram_word!(game, area_index) as usize]
    );
    let v2: i32 = ((4u16).wrapping_mul(ram_word!(game, area_index))) as i32 >> 1i32;
    set_ram_word!(game.expand_sq_left_pos = kAreaSelectMapLabelPositions[v2 as usize]);
    set_ram_word!(game.expand_sq_right_pos = ram_word!(game, expand_sq_left_pos));
    set_ram_word!(game.expand_sq_top_pos = kAreaSelectMapLabelPositions[(v2 + 1i32) as usize]);
    set_ram_word!(game.expand_sq_bottom_pos = ram_word!(game, expand_sq_top_pos));
    set_ram_words! { game, 0u16;
        expand_sq_left_subpos, expand_sq_right_subpos, expand_sq_top_subpos,
        expand_sq_bottom_subpos,
    }
    let vels: *const ExpandingSquareVels = (game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34))
        as *mut ExpandingSquareVels)
        .offset(ram_word!(game, area_index) as isize);
    set_ram_word!(game.expand_sq_left_subvel = (*vels).left_subvel);
    set_ram_word!(game.expand_sq_left_vel = (*vels).left_vel);
    set_ram_word!(game.expand_sq_right_subvel = (*vels).right_subvel);
    set_ram_word!(game.expand_sq_right_vel = (*vels).right_vel);
    set_ram_word!(game.expand_sq_top_subvel = (*vels).top_subvel);
    set_ram_word!(game.expand_sq_top_vel = (*vels).top_vel);
    set_ram_word!(game.expand_sq_bottom_subvel = (*vels).bottom_subvel);
    set_ram_word!(game.expand_sq_bottom_vel = (*vels).bottom_vel);
    SetupRoomSelectMapExpandingSquareTransHDMA(game);
    crate::rtl::write_register_and_mirror(HDMAEN as u16, 0xc, 0x85, true, game);
    update_ram_word!(game.menu_index, wrapping_add(1));
    QueueSfx1_Max6(0x3bu16, game);
}
pub unsafe fn SetupRoomSelectMapExpandingSquareTransHDMA(game: &mut Game) {
    let mut k: u16 = 0u16;
    let v0: u16 = ram_word!(game, expand_sq_top_pos);
    AddExpandingSqTransLeftIndirHDMA(v0, k, 0x9e22u16, game);
    k = AddExpandingSqTransRightIndirHDMA(v0, k, 0x9e20u16, game);
    let mut v2: u16 =
        (ram_word!(game, expand_sq_bottom_pos)).wrapping_sub(ram_word!(game, expand_sq_top_pos));
    if (ram_word!(game, expand_sq_bottom_pos) as u8) == (ram_word!(game, expand_sq_top_pos) as u8) {
        v2 = 1u16;
    }
    AddExpandingSqTransLeftIndirHDMA(v2, k, 0x9e32u16, game);
    k = AddExpandingSqTransRightIndirHDMA(v2, k, 0x9e36u16, game);
    let mut v4: u16 = (-32i32 - ram_word!(game, expand_sq_bottom_pos) as i32) as u16;
    if (ram_word!(game, expand_sq_bottom_pos) as u8) == 0xe0u8 {
        v4 = 1u16;
    }
    AddExpandingSqTransLeftIndirHDMA(v4, k, 0x9e22u16, game);
    k = AddExpandingSqTransRightIndirHDMA(v4, k, 0x9e20u16, game);
    *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(0)).field_0)
        .offset(k as i32 as isize) = 0u8;
    *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(0)).field_0)
        .offset(k as i32 as isize) = 0u8;
}
pub unsafe fn AddExpandingSqTransLeftIndirHDMA(a: u16, k: u16, j: u16, game: &mut Game) {
    if (a & 0x80) != 0 {
        *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(0)).field_0)
            .offset(k as i32 as isize) = ((a) as u8).wrapping_sub(127u8);
        *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_0)
            .offset(k as i32 as isize) = 127u8;
        *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(0)).field_1)
            .offset(k as i32 as isize) as *mut u16) = j;
        *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_1)
            .offset(k as i32 as isize) as *mut u16) = j;
    } else {
        *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(0)).field_0)
            .offset(k as i32 as isize) = a as u8;
        *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(0)).field_1)
            .offset(k as i32 as isize) as *mut u16) = j;
    };
}
pub unsafe fn AddExpandingSqTransRightIndirHDMA(a: u16, k: u16, j: u16, game: &mut Game) -> u16 {
    if (a & 0x80) != 0 {
        *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(0)).field_0)
            .offset(k as i32 as isize) = ((a) as u8).wrapping_sub(127u8);
        *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(1)).field_0)
            .offset(k as i32 as isize) = 127u8;
        *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(0)).field_1)
            .offset(k as i32 as isize) as *mut u16) = j;
        *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(1)).field_1)
            .offset(k as i32 as isize) as *mut u16) = j;
        (k).wrapping_add(6u16)
    } else {
        *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(0)).field_0)
            .offset(k as i32 as isize) = a as u8;
        *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(0)).field_1)
            .offset(k as i32 as isize) as *mut u16) = j;
        (k).wrapping_add(3u16)
    }
}
pub unsafe fn FileSelectMap_8_ExpandSquareTransToRoomSelectMap(game: &mut Game) {
    if (HandleRoomSelectMapExpandingSquareTrans(game) & 0x8000) != 0 {
        update_ram_word!(game.menu_index, wrapping_add(1));
        *ram_ptr!(game; 0x69) = 2u8;
        *ram_ptr!(game; 0x6c) = 0u8;
        *ram_ptr!(game; 0x6d) = 0u8;
        *ram_ptr!(game; 0x6b) = 0u8;
    }
    DrawAreaSelectMapLabels(game);
}
pub unsafe fn HandleRoomSelectMapExpandingSquareTrans(game: &mut Game) -> u16 {
    AddToHiLo(
        ram_ptr!(game, u16; 0x9e32),
        ram_ptr!(game, u16; 0x9e30),
        (ram_word!(game, expand_sq_left_vel) as u32) << 16i32
            | ram_word!(game, expand_sq_left_subvel) as u32,
    );
    if ram_word!(game, expand_sq_left_pos) as i32 - 1i32 & 0x8000i32 != 0 {
        set_ram_word!(game.expand_sq_left_pos = 1u16);
    }
    AddToHiLo(
        ram_ptr!(game, u16; 0x9e36),
        ram_ptr!(game, u16; 0x9e34),
        (ram_word!(game, expand_sq_right_vel) as u32) << 16i32
            | ram_word!(game, expand_sq_right_subvel) as u32,
    );
    if ram_word!(game, expand_sq_right_pos) as i32 - 256i32 & 0x8000i32 == 0 {
        set_ram_word!(game.expand_sq_right_pos = 255u16);
    }
    AddToHiLo(
        ram_ptr!(game, u16; 0x9e3a),
        ram_ptr!(game, u16; 0x9e38),
        (ram_word!(game, expand_sq_top_vel) as u32) << 16i32
            | ram_word!(game, expand_sq_top_subvel) as u32,
    );
    if ram_word!(game, expand_sq_top_pos) as i32 - 1i32 & 0x8000i32 != 0 {
        set_ram_word!(game.expand_sq_top_pos = 1u16);
    }
    AddToHiLo(
        ram_ptr!(game, u16; 0x9e3e),
        ram_ptr!(game, u16; 0x9e3c),
        (ram_word!(game, expand_sq_bottom_vel) as u32) << 16i32
            | ram_word!(game, expand_sq_bottom_subvel) as u32,
    );
    if ram_word!(game, expand_sq_bottom_pos) as i32 - 224i32 & 0x8000i32 == 0 {
        set_ram_word!(game.expand_sq_bottom_pos = 224u16);
    }
    SetupRoomSelectMapExpandingSquareTransHDMA(game);
    update_ram_word!(game.expand_sq_timer, wrapping_sub(1), reread)
}
pub unsafe fn FileSelectMap_9_InitRoomSelectMap(game: &mut Game) {
    LoadMirrorOfExploredMapTiles(game);
    DrawRoomSelectMap(game);
    LoadFromLoadStation(game);
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    let RoomDefHeader = get_RoomDefHeader(ram_word!(game, room_ptr), game);
    *(ram_ptr!(game, u16; 0x79f) as *mut u8).offset(0) = (*RoomDefHeader).area_index_;
    *(ram_ptr!(game, u16; 0x7a1) as *mut u8).offset(0) = (*RoomDefHeader).x_coordinate_on_map;
    *(ram_ptr!(game, u16; 0x7a3) as *mut u8).offset(0) = (*RoomDefHeader).y_coordinate_on_map;
    SetupMapScrollingForFileSelectMap(game);
    update_ram_word!(game.map_min_y_scroll, wrapping_add(24u16));
    set_ram_word!(game.reg_BG2VOFS = 24u16);
    let ref mut fresh31 = *(ram_ptr!(game, u16; 0x69));
    *fresh31 = (*fresh31 & !(4));
    update_ram_word!(game.menu_index, wrapping_add(1));
    set_ram_words! { game, 0u16;
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
    if ram_word!(game, enable_debug) != 0u16
        && (ram_word!(game, joypad2_new_keys) as i32 & kButton_Select as i32) != 0i32
    {
        let mut current_block_32: u64;
        QueueSfx1_Max6(0x38u16, game);
        let mut r24: u16 =
            *(ram_ptr!(game, u16; 0xd8f8, (2i32 * ram_word!(game, area_index) as i32) as isize));
        let mut v1: i16 = ram_word!(game, load_station_index) as i16;
        loop {
            r24 = (r24 as i32 >> 1i32) as u16;
            v1 -= 1;
            if (v1 < 0i16) {
                break;
            }
        }
        let r0 = RomPtr_82(
            *(RomPtr_82(
                (addr_kMapIconDataPointers as i32
                    + 64i32
                    + 2i32 * ram_word!(game, area_index) as i32) as u16,
                game,
            ) as *mut u16),
            game,
        ) as *const u16;
        let mut v3: i16 = (4i16).wrapping_mul((ram_word!(game, load_station_index)) as i16);
        let r18: u16 = *r0.offset((v3 as i32 >> 1i32) as isize);
        let r20: u16 = *r0.offset(((v3 as i32 >> 1i32) + 1i32) as isize);
        if ram_word!(game, load_station_index) as i32 - 16i32 & 0x8000i32 == 0 {
            current_block_32 = 4150833010744877889;
        } else {
            current_block_32 = 12800627514080957624;
        }
        loop {
            if current_block_32 == 12800627514080957624 {
                if update_ram_word!(game.load_station_index, wrapping_add(1), reread) as i32 - 16i32
                    & 0x8000i32
                    == 0
                {
                    current_block_32 = 8236137900636309791;
                } else {
                    current_block_32 = 5601891728916014340;
                }
            } else {
                update_ram_word!(game.load_station_index, wrapping_add(1));
                current_block_32 = 8236137900636309791;
            }
            if current_block_32 == 8236137900636309791 {
                let mut v5: i16 =
                    *r0.offset((2i32 * ram_word!(game, load_station_index) as i32) as isize) as i16;
                if v5 == -1i16 {
                    set_ram_word!(game.load_station_index = 0u16);
                    r24 = *(ram_ptr!(game, u16; 0xd8f8, (2i32 * ram_word!(game, area_index) as i32) as isize));
                } else if v5 != -2i16 {
                    break;
                } else {
                    current_block_32 = 4150833010744877889;
                    continue;
                }
            }
            let mut v4: i8 = ((r24) as i8) & (1i8);
            r24 = (r24 as i32 >> 1i32) as u16;
            if v4 == 0
                || *r0.offset((2i32 * ram_word!(game, load_station_index) as i32) as isize)
                    >= 0xfffeu16
            {
                current_block_32 = 12800627514080957624;
            } else {
                break;
            }
        }
        let v6: u16 = (4u16).wrapping_mul(ram_word!(game, load_station_index));
        let w7: u16 = *r0.offset((2i32 * ram_word!(game, load_station_index) as i32) as isize);
        if w7 as i32 - ram_word!(game, reg_BG1HOFS) as i32 & 0x8000i32 != 0
            || w7 as i32 - 256i32 - ram_word!(game, reg_BG1HOFS) as i32 & 0x8000i32 == 0
        {
            let mut v8: i16 = (ram_word!(game, reg_BG1HOFS) as i32
                + *r0.offset((v6 as i32 >> 1i32) as isize) as i32
                - r18 as i32) as i16;
            if v8 >= 0i16 {
                if (v8).wrapping_sub((ram_word!(game, map_min_x_scroll)) as i16) >= 0i16 {
                    v8 = ram_word!(game, map_min_x_scroll) as i16;
                }
            } else {
                v8 = 0i16;
            }
            set_ram_word!(game.reg_BG1HOFS = v8 as u16);
        }
        let v9: u16 = (v6).wrapping_add(2u16);
        let v10: u16 = *r0.offset((v9 as i32 >> 1i32) as isize);
        if v10 as i32 - ram_word!(game, reg_BG1VOFS) as i32 & 0x8000i32 != 0
            || v10 as i32 - 161i32 - ram_word!(game, reg_BG1VOFS) as i32 & 0x8000i32 == 0
        {
            let mut v11: u16 = (ram_word!(game, reg_BG1VOFS) as i32
                + *r0.offset((v9 as i32 >> 1i32) as isize) as i32
                - r20 as i32) as u16;
            if ((v11) as i16).wrapping_sub((ram_word!(game, map_min_y_scroll)) as i16) >= 0i16 {
                v11 = ram_word!(game, map_min_y_scroll);
            }
            set_ram_word!(game.reg_BG1VOFS = v11);
        }
    } else if (ram_word!(game, joypad1_newkeys) & 0x8000) != 0 {
        update_ram_word!(game.menu_index, wrapping_add(5u16));
        let mut v0: u16 = 0u16;
        while ram_word!(game, area_index)
            != kFileSelectMap_AreaIndexes[(v0 as i32 >> 1i32) as usize]
        {
            v0 = (v0).wrapping_add(2u16);
            if ((v0) as i16).wrapping_sub(12i16) >= 0i16 {
                set_ram_word!(game.file_select_map_area_index = 0u16);
                return;
            }
        }
        set_ram_word!(game.file_select_map_area_index = (v0 as i32 >> 1i32) as u16);
        QueueSfx1_Max6(0x3cu16, game);
    } else if ram_word!(game, joypad1_newkeys) as i32 & (kButton_Start as i32 | kButton_A as i32)
        != 0i32
    {
        update_ram_word!(game.menu_index, wrapping_add(1));
        QueueSfx1_Max6(0x38u16, game);
    }
}
pub unsafe fn sub_81AEC8(game: &mut Game) {
    HandleFileSelectMapScrollArrows(game);
}
pub unsafe fn HandleFileSelectMapScrollArrows(game: &mut Game) {
    if ram_word!(game, map_min_x_scroll) as i32 - 24i32 - ram_word!(game, reg_BG1HOFS) as i32
        & 0x8000i32
        != 0
    {
        DrawMapScrollArrowAndCheckToScroll(0x81u8, addr_kLeftMapScrollArrowData as u16, game);
    }
    if ram_word!(game, map_max_x_scroll) as i32 - 232i32 - ram_word!(game, reg_BG1HOFS) as i32
        & 0x8000i32
        == 0
    {
        DrawMapScrollArrowAndCheckToScroll(0x81u8, addr_kRightMapScrollArrowData as u16, game);
    }
    if ram_word!(game, map_min_y_scroll) as i32 - 64i32 - ram_word!(game, reg_BG1VOFS) as i32
        & 0x8000i32
        != 0
    {
        DrawMapScrollArrowAndCheckToScroll(0x81u8, addr_kUpMapScrollArrowData as u16, game);
    }
    if ram_word!(game, map_max_y_scroll) as i32 - 145i32 - ram_word!(game, reg_BG1VOFS) as i32
        & 0x8000i32
        != 0
    {
        if ram_word!(game, map_scrolling_direction)
            == (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81af50)) as *mut MapScrollArrowData))
                .map_scroll_dir
        {
            set_ram_words! { game, 0u16;
                map_scrolling_gear_switch_timer, map_scrolling_direction, map_scrolling_speed_index,
            }
        }
    } else {
        DrawMapScrollArrowAndCheckToScroll(0x81u8, addr_kDownMapScrollArrowData as u16, game);
    };
}
pub unsafe fn FileSelectMap_11(game: &mut Game) {
    DrawFileSelectMapIcons(game);
    DisplayMapElevatorDestinations(game);
    update_ram_word!(game.menu_index, wrapping_add(1));
}
pub unsafe fn FileSelectMap_13(game: &mut Game) {
    DrawFileSelectMapIcons(game);
    DisplayMapElevatorDestinations(game);
    HandleFadeOut(game);
    if (*ram_ptr!(game; 0x51) & 0xf) == 0 {
        update_ram_word!(game.menu_index, wrapping_add(1));
        (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).x_pos = 32u16;
    }
}
pub unsafe fn FileSelectMap_14(game: &mut Game) {
    let fresh22 = &raw mut (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).x_pos;
    fresh22.write_unaligned(fresh22.read_unaligned().wrapping_sub(1));
    if fresh22.read_unaligned() == 0 {
        ScreenOff(game);
        update_ram_word!(game.game_state, wrapping_add(1));
        set_ram_word!(game.menu_index = 0u16);
    }
}
pub unsafe fn FileSelectMap_15_ClearTileMap(game: &mut Game) {
    *ram_ptr!(game; 0x69) = 18u8;
    let mut i: i32 = 2046i32;
    while i >= 0i32 {
        *((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16
            as *mut u8)
            .offset(i as u16 as i32 as isize) as *mut u16) = 15u16;
        i -= 2i32;
    }
    let v1: u16 = ram_word!(game, vram_write_queue_tail);
    let v2 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
    (*v2).size = 2048u16;
    (*v2).src.addr = (&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap
        as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut u16) = 126u16;
    (*v2).vram_dst = ((*ram_ptr!(game; 0x58) as i32 & 0xfci32) << 8i32) as u16;
    set_ram_word!(game.vram_write_queue_tail = (v1).wrapping_add(7u16));
    update_ram_word!(game.menu_index, wrapping_add(1));
}
pub unsafe fn FileSelectMap_16_LoadPalettes(game: &mut Game) {
    LoadMenuPalettes(game);
    let mut v0: i16 = 0i16;
    loop {
        let mut v1: i16 = v0;
        let fresh19 = v0;
        v0 = v0 + 1;
        LoadInactiveAreaMapForegroundColors(fresh19 as u16, game);
        if ((v1).wrapping_sub(5i16) >= 0i16) {
            break;
        }
    }
    LoadActiveAreaMapForegroundColors(ram_word!(game, area_index), game);
    LoadAreaSelectBackgroundTilemap(ram_word!(game, area_index), game);
    update_ram_word!(game.menu_index, wrapping_add(1));
}
pub unsafe fn FileSelectMap_20_SetupExpandingSquare(game: &mut Game) {
    *ram_ptr!(game; 0x85) = 0u8;
    QueueSfx1_Max6(0x3cu16, game);
    set_ram_word!(
        game.expand_sq_timer = (kRoomSelectMapExpandingSquareTimers
            [ram_word!(game, area_index) as usize])
            .wrapping_sub(12u16)
    );
    set_ram_word!(
        game.expand_sq_left_subvel =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34))
                as *mut ExpandingSquareVels)
                .offset(0))
            .left_subvel as *mut u8)
                .offset(((16u16).wrapping_mul(ram_word!(game, area_index))) as i32 as isize,)
                as *mut u16)
    );
    set_ram_word!(
        game.expand_sq_left_vel =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34))
                as *mut ExpandingSquareVels)
                .offset(0))
            .left_vel as *mut u8)
                .offset(((16u16).wrapping_mul(ram_word!(game, area_index))) as i32 as isize,)
                as *mut u16)
    );
    set_ram_word!(
        game.expand_sq_right_subvel =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34))
                as *mut ExpandingSquareVels)
                .offset(0))
            .right_subvel as *mut u8)
                .offset(((16u16).wrapping_mul(ram_word!(game, area_index))) as i32 as isize,)
                as *mut u16)
    );
    set_ram_word!(
        game.expand_sq_right_vel =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34))
                as *mut ExpandingSquareVels)
                .offset(0))
            .right_vel as *mut u8)
                .offset(((16u16).wrapping_mul(ram_word!(game, area_index))) as i32 as isize,)
                as *mut u16)
    );
    set_ram_word!(
        game.expand_sq_top_subvel =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34))
                as *mut ExpandingSquareVels)
                .offset(0))
            .top_subvel as *mut u8)
                .offset(((16u16).wrapping_mul(ram_word!(game, area_index))) as i32 as isize,)
                as *mut u16)
    );
    set_ram_word!(
        game.expand_sq_top_vel =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34))
                as *mut ExpandingSquareVels)
                .offset(0))
            .top_vel as *mut u8)
                .offset(((16u16).wrapping_mul(ram_word!(game, area_index))) as i32 as isize,)
                as *mut u16)
    );
    set_ram_word!(
        game.expand_sq_bottom_subvel =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34))
                as *mut ExpandingSquareVels)
                .offset(0))
            .bottom_subvel as *mut u8)
                .offset(((16u16).wrapping_mul(ram_word!(game, area_index))) as i32 as isize,)
                as *mut u16)
    );
    set_ram_word!(
        game.expand_sq_bottom_vel =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34))
                as *mut ExpandingSquareVels)
                .offset(0))
            .bottom_vel as *mut u8)
                .offset(((16u16).wrapping_mul(ram_word!(game, area_index))) as i32 as isize,)
                as *mut u16)
    );
    set_ram_word!(game.expand_sq_left_pos = 8u16);
    set_ram_word!(game.expand_sq_right_pos = 248u16);
    set_ram_word!(game.expand_sq_top_pos = 8u16);
    set_ram_word!(game.expand_sq_bottom_pos = 216u16);
    set_ram_words! { game, 0u16;
        expand_sq_left_subpos, expand_sq_right_subpos, expand_sq_top_subpos,
        expand_sq_bottom_subpos,
    }
    SetupRoomSelectMapExpandingSquareTransHDMA(game);
    update_ram_word!(game.menu_index, wrapping_add(1));
    crate::rtl::register_writes! { game;
        mirror(HDMAEN as u16, 0xc, 0x85, true); mirror(W12SEL as u16, 0x32, 0x60, true);
        mirror(W34SEL as u16, 2, 0x61, true);
    }
    *ram_ptr!(game; 0x5d) = 48u8;
    *ram_ptr!(game; 0x62) = 34u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(3)).field_0 = 0u8;
    (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e10)).offset(3)).field_0 = 0u8;
    set_ram_words! { game, 0u16; reg_BG1VOFS, reg_BG1HOFS }
}
pub unsafe fn FileSelectMap_21_MoveExpandingSquare(game: &mut Game) {
    AddToHiLo(
        ram_ptr!(game, u16; 0x9e32),
        ram_ptr!(game, u16; 0x9e30),
        -((ram_word!(game, expand_sq_left_vel) as i32) << 16i32
            | ram_word!(game, expand_sq_left_subvel) as i32) as u32,
    );
    AddToHiLo(
        ram_ptr!(game, u16; 0x9e36),
        ram_ptr!(game, u16; 0x9e34),
        -((ram_word!(game, expand_sq_right_vel) as i32) << 16i32
            | ram_word!(game, expand_sq_right_subvel) as i32) as u32,
    );
    AddToHiLo(
        ram_ptr!(game, u16; 0x9e3a),
        ram_ptr!(game, u16; 0x9e38),
        -((ram_word!(game, expand_sq_top_vel) as i32) << 16i32
            | ram_word!(game, expand_sq_top_subvel) as i32) as u32,
    );
    AddToHiLo(
        ram_ptr!(game, u16; 0x9e3e),
        ram_ptr!(game, u16; 0x9e3c),
        -((ram_word!(game, expand_sq_bottom_vel) as i32) << 16i32
            | ram_word!(game, expand_sq_bottom_subvel) as i32) as u32,
    );
    SetupRoomSelectMapExpandingSquareTransHDMA(game);
    DrawAreaSelectMapLabels(game);
    if (update_ram_word!(game.expand_sq_timer, wrapping_sub(1), reread) & 0x8000) != 0 {
        update_ram_word!(game.menu_index, wrapping_sub(15u16));
        *ram_ptr!(game; 0x69) = 17u8;
        *ram_ptr!(game; 0x6c) = 0u8;
        *ram_ptr!(game; 0x6d) = 0u8;
    }
}
pub unsafe fn NewSaveFile(game: &mut Game) {
    set_ram_words! { game, 99u16; samus_max_health, samus_health }
    set_ram_words! { game, 0u16;
        samus_max_missiles, samus_missiles, samus_max_super_missiles, samus_super_missiles,
        samus_max_power_bombs, samus_power_bombs, hud_item_index, collected_beams, equipped_beams,
        collected_items, equipped_items, reserve_health_mode, samus_max_reserve_health,
        samus_reserve_health, samus_reserve_missiles,
    }
    set_ram_word!(game.button_config_up = kButton_Up as u16);
    set_ram_word!(game.button_config_down = kButton_Down as u16);
    set_ram_word!(game.button_config_left = kButton_Left as u16);
    set_ram_word!(game.button_config_right = kButton_Right as u16);
    set_ram_word!(game.button_config_jump_a = kButton_A as u16);
    set_ram_word!(game.button_config_run_b = kButton_B as u16);
    set_ram_word!(game.button_config_shoot_x = kButton_X as u16);
    set_ram_word!(game.button_config_itemcancel_y = kButton_Y as u16);
    set_ram_word!(game.button_config_itemswitch = kButton_Select as u16);
    set_ram_word!(game.button_config_aim_up_R = kButton_R as u16);
    set_ram_word!(game.button_config_aim_down_L = kButton_L as u16);
    set_ram_words! { game, 0u16;
        game_time_frames, game_time_seconds, game_time_minutes, game_time_hours, japanese_text_flag,
        moonwalk_flag, hud_auto_cancel_flag,
    }
    set_ram_words! { game, 1u16; debug_flag, UNUSED_word_7E09E8 }
    let mut v0: u16 = 0u16;
    loop {
        let v1: i32 = v0 as i32 >> 1i32;
        *(ram_ptr!(game, u16; 0xd830, v0 as isize)) = 0u16;
        *(ram_ptr!(game, u16; 0xd870, v0 as isize)) = 0u16;
        *(ram_ptr!(game, u16; 0xd870, v0 as isize)) = 0u16;
        *(ram_ptr!(game, u16; 0xd8b0, v0 as isize)) = 0u16;
        set_ram_word!(game.UNUSED_word_7ED8F0[v1] = 0u16);
        *(ram_ptr!(game, u16; 0xd908, v0 as isize)) = 0u16;
        *(ram_ptr!(game, u16; 0xd8f8, v0 as isize)) = 0u16;
        *(ram_ptr!(game, u16; 0xd8f8, (v0 as i32 + 8i32) as isize)) = 0u16;
        v0 = (v0).wrapping_add(2u16);
        if (((v0) as i16).wrapping_sub(8i16) >= 0i16) {
            break;
        }
    }
    loop {
        let v2: i32 = v0 as i32 >> 1i32;
        *(ram_ptr!(game, u16; 0xd830, v0 as isize)) = 0u16;
        *(ram_ptr!(game, u16; 0xd870, v0 as isize)) = 0u16;
        *(ram_ptr!(game, u16; 0xd870, v0 as isize)) = 0u16;
        *(ram_ptr!(game, u16; 0xd8b0, v0 as isize)) = 0u16;
        v0 = (v0).wrapping_add(2u16);
        if (((v0) as i16).wrapping_sub(64i16) >= 0i16) {
            break;
        }
    }
    let mut v3: u16 = 0u16;
    loop {
        set_ram_word!(game.explored_map_tiles_saved[(v3 as i32 >> 1i32)] = 0u16);
        v3 = (v3).wrapping_add(2u16);
        if (((v3) as i16).wrapping_sub(1792i16) >= 0i16) {
            break;
        }
    }
}
pub unsafe fn FileSelectClearRestOfMenuTilemapRow(mut v0: u16, game: &mut Game) {
    let mut n: i32 = 32i32 - ((v0 as i32 & 0x3fi32) >> 1i32);
    loop {
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(768) as *mut u8)
            .offset(v0 as i32 as isize) as *mut u16) = 15u16;
        v0 = (v0).wrapping_add(2u16);
        n -= 1;
        if (n == 0) {
            break;
        }
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
            j = (j).wrapping_add(2u16);
            k_bak += 64i32;
            k = k_bak as u16;
        }
        if v2 == -1i16 {
            break;
        }
        *(((&raw mut (*(ram_ptr!(game, Ram3000; 0x3000))).pause_menu_map_tilemap as *mut u16)
            .offset(768) as *mut u8)
            .offset(k as i32 as isize) as *mut u16) =
            ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).palette_index) | ((v2) as u16);
        k = (k).wrapping_add(2u16);
        j = (j).wrapping_add(2u16);
    }
}
