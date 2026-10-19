pub use crate::game::constants::bank_82::*;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::game_states::*;
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
use crate::ram_map::{ram_word, set_ram_word};
use crate::rtl::AddToHiLo;
use crate::rtl::{RomPtr, RtlWriteSram, Unreachable, WriteReg};
pub use crate::snes::registers::*;
extern "C" {
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;

}
pub use crate::types::Buttons;
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, size_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, Func_V,
    LongPtr, OamEnt, SnesRegs, VoidP, VramWriteEntry,
};
pub use crate::types::{EnemyData, GameState, RoomDefHeader, SpawnHdmaObject_Args};
pub use crate::types::{ExpandingSquareTransitionHdma, MapIconDataPointers, StartDmaCopy};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ExpandingSquareVels {
    pub left_subvel: uint16,
    pub left_vel: uint16,
    pub right_subvel: uint16,
    pub right_vel: uint16,
    pub top_subvel: uint16,
    pub top_vel: uint16,
    pub bottom_subvel: uint16,
    pub bottom_vel: uint16,
}
pub use crate::types::{
    Consts_81, MapScrollArrowData, Ram3000, Ram3000_Menu, Ram3000_Misc, Ram3000_MsgBox, Ram4000,
    Ram4000_Backups,
};
pub const addr_kAreaSelectBackgroundTilemaps: Consts_81 = 48922;
pub const addr_kAreaSelectForegroundTilemap: Consts_81 = 46874;
pub const addr_kMenuTilemap_DataCleared: Consts_81 = 46810;
pub const addr_kMenuTilemap_ClearSamusA: Consts_81 = 46746;
pub const addr_kMenuTilemap_ClearWhichData: Consts_81 = 46682;
pub const addr_kMenuTilemap_CopyCompleted: Consts_81 = 46650;
pub const addr_kMenuTilemap_No: Consts_81 = 46634;
pub const addr_kMenuTilemap_Yes: Consts_81 = 46618;
pub const addr_kMenuTilemap_IsThisOk: Consts_81 = 46594;
pub const addr_kMenuTilemap_CopySamusToSamus: Consts_81 = 46536;
pub const addr_kMenuTilemap_CopySamusToWhere: Consts_81 = 46486;
pub const addr_kMenuTilemap_CopyWhichData: Consts_81 = 46452;
pub const addr_kMenuTilemap_DataClearMode: Consts_81 = 46388;
pub const addr_kMenuTilemap_DataCopyMode: Consts_81 = 46328;
pub const addr_kMenuTilemap_Exit: Consts_81 = 46318;
pub const addr_kMenuTilemap_DataClear: Consts_81 = 46296;
pub const addr_kMenuTilemap_DataCopy: Consts_81 = 46276;
pub const addr_kMenuTilemap_NoData: Consts_81 = 46252;
pub const addr_word_81B4A8: Consts_81 = 46248;
pub const addr_kMenuTilemap_TIME: Consts_81 = 46240;
pub const addr_kMenuTilemap_Energy: Consts_81 = 46230;
pub const addr_kMenuTilemap_SamusC: Consts_81 = 46198;
pub const addr_kMenuTilemap_SamusB: Consts_81 = 46166;
pub const addr_kMenuTilemap_SamusA: Consts_81 = 46134;
pub const addr_kMenuTilemap_SamusData: Consts_81 = 46090;
pub const addr_kDownMapScrollArrowData: Consts_81 = 44880;
pub const addr_kUpMapScrollArrowData: Consts_81 = 44870;
pub const addr_kRightMapScrollArrowData: Consts_81 = 44860;
pub const addr_kLeftMapScrollArrowData: Consts_81 = 44850;
pub const addr_kMenuTilemap_NoGoToTitle: Consts_81 = 37792;
pub const addr_kMenuTilemap_YesReturnToGame: Consts_81 = 37708;
pub const addr_kMenuTilemap_TryAgain: Consts_81 = 37684;
pub const addr_kMenuTilemap_FindMetroidLarva: Consts_81 = 37636;
pub const addr_kMenuTilemap_GameOver: Consts_81 = 37596;
pub use crate::types::Consts_82;
pub const true_0: ::core::ffi::c_int = 1;
#[inline]
unsafe fn RomPtr_81(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0x810000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtr_82(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0x820000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtr_8C(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0x8c0000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtr_92(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0x920000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtr_93(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0x930000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtrWithBank(bank: uint8, addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr(
        ((bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int | addr as ::core::ffi::c_int)
            as uint32_t,
        game,
    )
}
#[inline]
unsafe fn gOamEnt(v: ::core::ffi::c_int, game: &mut Game) -> *mut OamEnt {
    (&raw mut game.g_ram as *mut uint8).offset((0x370 as ::core::ffi::c_int + v) as isize)
        as *mut uint8 as *mut OamEnt
}
#[inline]
unsafe fn gVramWriteEntry(a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry
}
#[inline]
unsafe fn get_RoomDefHeader(a: uint16, game: &mut Game) -> *mut RoomDefHeader {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomDefHeader
}
static kFileSelectMap_AreaIndexes: [uint16; 6] = [0, 3, 5, 1, 4, 2];
pub unsafe fn SoftReset(game: &mut Game) {
    set_ram_word!(game.game_state = 0xffff as uint16);
}
pub unsafe fn SaveToSram(a: uint16, game: &mut Game) {
    let mut v7: uint16 = 0;
    let mut v11: uint16 = 0;
    let mut r20: uint16 = 0 as uint16;
    let r18: uint16 =
        (2 as ::core::ffi::c_int * (a as ::core::ffi::c_int & 3 as ::core::ffi::c_int)) as uint16;
    let mut i: ::core::ffi::c_int = 94 as ::core::ffi::c_int;
    while i >= 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd7c0) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16 as *mut uint8)
                .offset(i as isize) as *mut uint16);
        i -= 2 as ::core::ffi::c_int;
    }
    let mut v3: uint16 = (ram_word!(game, area_index)).wrapping_mul(256u16);
    let mut v4: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd52) as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x7f7)
                .offset(v4 as isize) as *mut uint8 as *mut uint16);
        v4 = (v4).wrapping_add(2u16);
        v3 = (v3).wrapping_add(2u16);
        if (((v4) as i16).wrapping_sub(256i16) >= 0i16) {
            break;
        }
    }
    PackMapToSave(game);
    set_ram_word!(game.sram_save_station_index = ram_word!(game, load_station_index));
    set_ram_word!(game.sram_area_index = ram_word!(game, area_index));
    let mut v5: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x81812b)) as *const uint8
        as *mut uint16)
        .offset((r18 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    let mut v6: *mut uint16 = (&raw mut game.g_ram as *mut uint8).offset(0xd7c0) as *mut uint16;
    loop {
        let fresh94 = v6;
        v6 = v6.offset(1);
        v7 = *fresh94;
        *(game.g_sram.offset(
            (2 as ::core::ffi::c_int * (v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int))
                as isize,
        ) as *mut uint8 as *mut uint16) = v7;
        r20 = (r20).wrapping_add(v7);
        v5 = (v5).wrapping_add(2u16);
        if (v6 == (&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16) {
            break;
        }
    }
    let v8: uint16 = r18;
    let v9: uint16 = r20;
    let v10: ::core::ffi::c_int = r18 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *(game.g_sram.offset((2 as ::core::ffi::c_int * v10) as isize) as *mut uint8 as *mut uint16) =
        r20;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v10 + 0x1ff0 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = v9;
    v11 = !(v9 as ::core::ffi::c_int) as uint16;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v10 + 8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = v11;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v10 + 0x1ff8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = v11;
    RtlWriteSram(game);
}
pub unsafe fn LoadFromSram(a: uint16, game: &mut Game) -> uint8 {
    let mut r20: uint16 = 0 as uint16;
    let r18: uint16 =
        (2 as ::core::ffi::c_int * (a as ::core::ffi::c_int & 3 as ::core::ffi::c_int)) as uint16;
    let mut v1: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x81812b)) as *const uint8
        as *mut uint16)
        .offset((a as ::core::ffi::c_int & 3 as ::core::ffi::c_int) as isize);
    let mut v2: *mut uint16 = (&raw mut game.g_ram as *mut uint8).offset(0xd7c0) as *mut uint16;
    loop {
        let v3: uint16 = *(game.g_sram.offset(0) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        let fresh1 = v2;
        v2 = v2.offset(1);
        *fresh1 = v3;
        r20 = (r20).wrapping_add(v3);
        v1 = (v1).wrapping_add(2u16);
        if (v2 == (&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16) {
            break;
        }
    }
    let v4: ::core::ffi::c_int = r18 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if r20 == *(game.g_sram.offset(0) as *mut uint16).offset(v4 as isize)
        && r20 as ::core::ffi::c_int ^ 0xffff as ::core::ffi::c_int
            == *(game.g_sram.offset(0x8) as *mut uint16).offset(v4 as isize) as ::core::ffi::c_int
        || r20 == *(game.g_sram.offset(0x1ff0) as *mut uint16).offset(v4 as isize)
            && r20 as ::core::ffi::c_int ^ 0xffff as ::core::ffi::c_int
                == *(game.g_sram.offset(0x1ff8) as *mut uint16).offset(v4 as isize)
                    as ::core::ffi::c_int
    {
        let mut i: ::core::ffi::c_int = 94 as ::core::ffi::c_int;
        while i >= 0i32 {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16 as *mut uint8)
                .offset(i as isize) as *mut uint16) =
                *((&raw mut game.g_ram as *mut uint8).offset(0xd7c0) as *mut uint16)
                    .offset((i >> 1 as ::core::ffi::c_int) as isize);
            i -= 2 as ::core::ffi::c_int;
        }
        UnpackMapFromSave(game);
        set_ram_word!(game.load_station_index = ram_word!(game, sram_save_station_index));
        set_ram_word!(game.area_index = ram_word!(game, sram_area_index));
        0 as uint8
    } else {
        r20 = 0 as uint16;
        let mut v7: uint16 =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x81812b)) as *const uint8 as *mut uint16)
                .offset((r18 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        let mut v8: uint16 = ((&raw mut game.g_ram as *mut uint8).offset(0xd7c0) as *mut uint16
            as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8)
            as ::core::ffi::c_long as uint16;
        let mut v9: uint16 = 0 as uint16;
        loop {
            *(game.g_sram.offset(0) as *mut uint16)
                .offset((v7 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v9;
            v9 = (v9).wrapping_add(r20);
            r20 = v9;
            v7 = (v7).wrapping_add(2u16);
            v8 = (v8).wrapping_add(2u16);
            if (v8 as ::core::ffi::c_long
                == ((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16 as *mut uint8)
                    .offset_from(&raw mut game.g_ram as *mut uint8)
                    as ::core::ffi::c_long)
            {
                break;
            }
        }
        set_ram_word!(game.load_station_index = 0 as uint16);
        set_ram_word!(game.area_index = 0 as uint16);
        1 as uint8
    }
}
pub unsafe fn UnpackMapFromSave(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 1792 as ::core::ffi::c_int;
    while i >= 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd52) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < 6i32 {
        let mut n: ::core::ffi::c_int = *(game.g_rom.offset(crate::rtl::lorom_offset(0x818131))
            as *const uint8 as *mut uint8)
            .offset(i_0 as isize) as ::core::ffi::c_int;
        let mut r0: *const uint8 = RomPtr_81(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x8182d6)) as *const uint8 as *mut uint16)
                .offset(i_0 as isize),
            game,
        );
        let mut v3: ::core::ffi::c_int = *(game.g_rom.offset(crate::rtl::lorom_offset(0x818138))
            as *const uint8 as *mut uint16)
            .offset(i_0 as isize) as ::core::ffi::c_int;
        let unpacked: *mut uint8 =
            (&raw mut game.g_ram as *mut uint8).offset(0xcd52) as *mut uint16 as *mut uint8;
        loop {
            let fresh2 = r0;
            r0 = r0.offset(1);
            let v4: uint16 =
                (i_0 * 256 as ::core::ffi::c_int + *fresh2 as ::core::ffi::c_int) as uint16;
            let fresh3 = v3;
            v3 = v3 + 1;
            *unpacked.offset(v4 as isize) = *(&raw mut game.g_ram as *mut uint8)
                .offset(0xd91c)
                .offset(fresh3 as isize);
            n -= 1;
            if (n == 0) {
                break;
            }
        }
        i_0 += 1;
    }
}
pub unsafe fn PackMapToSave(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 6i32 {
        let mut n: ::core::ffi::c_int = *(game.g_rom.offset(crate::rtl::lorom_offset(0x818131))
            as *const uint8 as *mut uint8)
            .offset(i as isize) as ::core::ffi::c_int;
        let mut r0: *const uint8 = RomPtr_81(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x8182d6)) as *const uint8 as *mut uint16)
                .offset(i as isize),
            game,
        );
        let mut v1: ::core::ffi::c_int = *(game.g_rom.offset(crate::rtl::lorom_offset(0x818138))
            as *const uint8 as *mut uint16)
            .offset(i as isize) as ::core::ffi::c_int;
        let unpacked: *mut uint8 =
            (&raw mut game.g_ram as *mut uint8).offset(0xcd52) as *mut uint16 as *mut uint8;
        loop {
            let fresh95 = r0;
            r0 = r0.offset(1);
            let v3: ::core::ffi::c_int =
                i * 256 as ::core::ffi::c_int + *fresh95 as ::core::ffi::c_int;
            let fresh96 = v1;
            v1 = v1 + 1;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xd91c)
                .offset(fresh96 as isize) = *unpacked.offset(v3 as isize);
            n -= 1;
            if (n == 0) {
                break;
            }
        }
        i += 1;
    }
}
pub unsafe fn DrawSpritemap(
    db: uint8,
    j: uint16,
    x_r20: uint16,
    y_r18: uint16,
    chr_r22: uint16,
    game: &mut Game,
) {
    let mut pp: *const uint8 = RomPtrWithBank(db, j, game);
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    let mut idx: ::core::ffi::c_int = ram_word!(game, oam_next_ptr) as ::core::ffi::c_int;
    while n != 0i32 && idx < 0x200i32 {
        let oam: *mut OamEnt = gOamEnt(idx, game);
        let mut x: ::core::ffi::c_int =
            x_r20 as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int;
        let mut y: ::core::ffi::c_int =
            y_r18 as uint8 as ::core::ffi::c_int + *pp.offset(2) as ::core::ffi::c_int;
        if if *pp.offset(2) as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 {
            (y >= 0xe0i32) as ::core::ffi::c_int
        } else if y & 0x100 as ::core::ffi::c_int != 0 {
            ((y as uint8) >= 0xe0u8) as ::core::ffi::c_int
        } else {
            ((y as uint8) < 0xe0u8) as ::core::ffi::c_int
        } != 0
        {
            x = 0x180 as ::core::ffi::c_int;
            y = 0xe0 as ::core::ffi::c_int;
        }
        (*oam).xcoord = x as uint8;
        (*oam).ycoord = y as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (chr_r22 as ::core::ffi::c_int
            | *(pp.offset(3) as *mut uint16) as ::core::ffi::c_int & 0xf1ff as ::core::ffi::c_int)
            as uint16;
        let ref mut fresh9 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh9 = (*fresh9 as ::core::ffi::c_int
            | ((x & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | (*(pp as *mut int16) < 0i16) as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        idx += 4 as ::core::ffi::c_int;
        pp = pp.offset(5);
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as uint16);
}
pub unsafe fn DrawSpritemapOffScreen(
    j: uint16,
    x_r20: uint16,
    y_r18: uint16,
    chr_r22: uint16,
    game: &mut Game,
) {
    let mut pp: *const uint8 = RomPtr_8C(j, game);
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    let mut idx: ::core::ffi::c_int = ram_word!(game, oam_next_ptr) as ::core::ffi::c_int;
    while n != 0i32 && idx < 0x200i32 {
        let oam: *mut OamEnt = gOamEnt(idx, game);
        let mut x: ::core::ffi::c_int =
            x_r20 as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int;
        let mut y: ::core::ffi::c_int =
            y_r18 as uint8 as ::core::ffi::c_int + *pp.offset(2) as ::core::ffi::c_int;
        if if *pp.offset(2) as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 {
            (y < 0xe0i32) as ::core::ffi::c_int
        } else if y & 0x100 as ::core::ffi::c_int != 0 {
            ((y as uint8) < 0xe0u8) as ::core::ffi::c_int
        } else {
            ((y as uint8) >= 0xe0u8) as ::core::ffi::c_int
        } != 0
        {
            x = 0x180 as ::core::ffi::c_int;
            y = 0xe0 as ::core::ffi::c_int;
        }
        (*oam).xcoord = x as uint8;
        (*oam).ycoord = y as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (chr_r22 as ::core::ffi::c_int
            | *(pp.offset(3) as *mut uint16) as ::core::ffi::c_int & 0xf1ff as ::core::ffi::c_int)
            as uint16;
        let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh10 = (*fresh10 as ::core::ffi::c_int
            | ((x & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | (*(pp as *mut int16) < 0i16) as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        idx += 4 as ::core::ffi::c_int;
        pp = pp.offset(5);
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as uint16);
}
pub unsafe fn DrawMenuSpritemap(a: uint16, k: uint16, j: uint16, chr_r3: uint16, game: &mut Game) {
    let mut pp: *const uint8 = RomPtr_82(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c569)) as *const uint8 as *mut uint16)
            .offset(a as isize),
        game,
    );
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    let mut idx: ::core::ffi::c_int = ram_word!(game, oam_next_ptr) as ::core::ffi::c_int;
    while n != 0i32 {
        let oam: *mut OamEnt = gOamEnt(idx, game);
        let x: uint16 = (k).wrapping_add(*(pp as *mut uint16));
        (*oam).xcoord = x as uint8;
        let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh4 = (*fresh4 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | (*(pp as *mut int16) < 0i16) as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        (*oam).ycoord = ((j) as u8).wrapping_add(*pp.offset(2));
        *(&raw mut (*oam).charnum as *mut uint16) = (chr_r3 as ::core::ffi::c_int
            | *(pp.offset(3) as *mut uint16) as ::core::ffi::c_int & 0xf1ff as ::core::ffi::c_int)
            as uint16;
        pp = pp.offset(5);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as uint16);
}
pub unsafe fn DrawSamusSpritemap(a: uint16, x_pos: uint16, y_pos: uint16, game: &mut Game) {
    if *(game.g_rom.offset(crate::rtl::lorom_offset(0x92808d)) as *const uint8 as *mut uint16)
        .offset(a as isize)
        == 0u16
    {
        return;
    }
    let mut pp: *const uint8 = RomPtr_92(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x92808d)) as *const uint8 as *mut uint16)
            .offset(a as isize),
        game,
    );
    let mut idx: ::core::ffi::c_int = ram_word!(game, oam_next_ptr) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    while n != 0i32 {
        let x: uint16 = (x_pos).wrapping_add(*(pp as *mut uint16));
        let v9: *mut OamEnt = gOamEnt(idx, game);
        (*v9).xcoord = x as uint8;
        let ref mut fresh8 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh8 = (*fresh8 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | (*(pp as *mut int16) < 0i16) as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        (*v9).ycoord = ((y_pos) as u8).wrapping_add(*pp.offset(2));
        *(&raw mut (*v9).charnum as *mut uint16) = *(pp.offset(3) as *mut uint16);
        pp = pp.offset(5);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as uint16);
}
pub unsafe fn DrawBeamGrappleSpritemap(a: uint16, x_r20: uint16, y_r18: uint16, game: &mut Game) {
    DrawGrappleOrProjectileSpritemap(
        RomPtr_93(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x93a1a1)) as *const uint8 as *mut uint16)
                .offset(a as isize),
            game,
        ),
        x_r20,
        y_r18,
        game,
    );
}
pub unsafe fn DrawProjectileSpritemap(k: uint16, x_r20: uint16, y_r18: uint16, game: &mut Game) {
    DrawGrappleOrProjectileSpritemap(
        RomPtr_93(
            *((&raw mut game.g_ram as *mut uint8).offset(0xcb8) as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
            game,
        ),
        x_r20,
        y_r18,
        game,
    );
}
pub unsafe fn DrawGrappleOrProjectileSpritemap(
    mut pp: *const uint8,
    x_r20: uint16,
    y_r18: uint16,
    game: &mut Game,
) {
    let mut idx: ::core::ffi::c_int = ram_word!(game, oam_next_ptr) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    while n != 0i32 {
        let x: uint16 = (x_r20).wrapping_add(*(pp as *mut uint16));
        let v4: *mut OamEnt = gOamEnt(idx, game);
        (*v4).xcoord = x as uint8;
        let ref mut fresh5 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh5 = (*fresh5 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | (*(pp as *mut int16) < 0i16) as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        (*v4).ycoord = ((y_r18) as u8).wrapping_add(*pp.offset(2));
        *(&raw mut (*v4).charnum as *mut uint16) = *(pp.offset(3) as *mut uint16);
        pp = pp.offset(5);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as uint16);
}
pub unsafe fn DrawSpritemapWithBaseTile(
    db: uint8,
    j: uint16,
    r20_x: uint16,
    r18_y: uint16,
    r3: uint16,
    r0: uint16,
    game: &mut Game,
) {
    if j == 0u16 {
        return;
    }
    let mut pp: *mut uint8 = RomPtrWithBank(db, j, game) as *mut uint8;
    let mut idx: ::core::ffi::c_int = ram_word!(game, oam_next_ptr) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    while n != 0i32 {
        let oam: *mut OamEnt = gOamEnt(idx, game);
        let x: uint16 = (r20_x).wrapping_add(*(pp.offset(0) as *mut uint16));
        (*oam).xcoord = x as uint8;
        let ref mut fresh11 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh11 = (*fresh11 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | (*(pp as *mut int16) < 0i16) as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        (*oam).ycoord = ((r18_y) as u8).wrapping_add(*pp.offset(2));
        *(&raw mut (*oam).charnum as *mut uint16) = (r3 as ::core::ffi::c_int
            | r0 as ::core::ffi::c_int + *(pp.offset(3) as *mut uint16) as ::core::ffi::c_int)
            as uint16;
        pp = pp.offset(5);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as uint16);
}
pub unsafe fn DrawSpritemapWithBaseTile2(
    db: uint8,
    j: uint16,
    r20_x: uint16,
    r18_y: uint16,
    r3: uint16,
    r0: uint16,
    game: &mut Game,
) {
    let mut pp: *const uint8 = RomPtrWithBank(db, j, game);
    let mut idx: ::core::ffi::c_int = ram_word!(game, oam_next_ptr) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    while n != 0i32 {
        let oam: *mut OamEnt = gOamEnt(idx, game);
        let x: uint16 = (r20_x).wrapping_add(*(pp as *mut uint16));
        (*oam).xcoord = x as uint8;
        let ref mut fresh12 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh12 = (*fresh12 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | (*(pp as *mut int16) < 0i16) as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        let y: ::core::ffi::c_int =
            *pp.offset(2) as ::core::ffi::c_int + r18_y as uint8 as ::core::ffi::c_int;
        (*oam).ycoord = (if (y & 0x100 as ::core::ffi::c_int == 0) as ::core::ffi::c_int
            == (*pp.offset(2) as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0)
                as ::core::ffi::c_int
        {
            y
        } else {
            0xf0 as ::core::ffi::c_int
        }) as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (r3 as ::core::ffi::c_int
            | r0 as ::core::ffi::c_int + *(pp.offset(3) as *mut uint16) as ::core::ffi::c_int)
            as uint16;
        pp = pp.offset(5);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as uint16);
}
pub unsafe fn DrawSpritemapWithBaseTileOffscreen(
    db: uint8,
    j: uint16,
    r20_x: uint16,
    r18_y: uint16,
    r3: uint16,
    r0: uint16,
    game: &mut Game,
) {
    let mut pp: *const uint8 = RomPtrWithBank(db, j, game);
    let mut idx: ::core::ffi::c_int = ram_word!(game, oam_next_ptr) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    while n != 0i32 {
        let oam: *mut OamEnt = gOamEnt(idx, game);
        let x: ::core::ffi::c_int =
            r20_x as ::core::ffi::c_int + *(pp as *mut uint16) as ::core::ffi::c_int;
        (*oam).xcoord = x as uint8;
        let ref mut fresh13 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh13 = (*fresh13 as ::core::ffi::c_int
            | ((x & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | (*(pp as *mut int16) < 0i16) as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        let y: ::core::ffi::c_int =
            *pp.offset(2) as ::core::ffi::c_int + r18_y as uint8 as ::core::ffi::c_int;
        (*oam).ycoord = (if (y & 0x100 as ::core::ffi::c_int == 0) as ::core::ffi::c_int
            != (*pp.offset(2) as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0)
                as ::core::ffi::c_int
        {
            y
        } else {
            0xf0 as ::core::ffi::c_int
        }) as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (r3 as ::core::ffi::c_int
            | r0 as ::core::ffi::c_int + *(pp.offset(3) as *mut uint16) as ::core::ffi::c_int)
            as uint16;
        pp = pp.offset(5);
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as uint16);
}
pub unsafe fn DrawEprojSpritemapWithBaseTile(
    db: uint8,
    j: uint16,
    x_r20: uint16,
    y_r18: uint16,
    chr_r26: uint16,
    chr_r28: uint16,
    game: &mut Game,
) {
    let mut pp: *const uint8 = RomPtrWithBank(db, j, game);
    let mut idx: ::core::ffi::c_int = ram_word!(game, oam_next_ptr) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    while n != 0i32 {
        let oam: *mut OamEnt = gOamEnt(idx, game);
        let x: uint16 = (x_r20).wrapping_add(*(pp as *mut uint16));
        (*oam).xcoord = x as uint8;
        let ref mut fresh6 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh6 = (*fresh6 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | (*(pp as *mut int16) < 0i16) as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        let y: ::core::ffi::c_int =
            *pp.offset(2) as ::core::ffi::c_int + y_r18 as uint8 as ::core::ffi::c_int;
        (*oam).ycoord = (if (y & 0x100 as ::core::ffi::c_int == 0) as ::core::ffi::c_int
            == (*pp.offset(2) as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0)
                as ::core::ffi::c_int
        {
            y
        } else {
            0xf0 as ::core::ffi::c_int
        }) as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (chr_r28 as ::core::ffi::c_int
            | chr_r26 as ::core::ffi::c_int + *(pp.offset(3) as *mut uint16) as ::core::ffi::c_int)
            as uint16;
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        pp = pp.offset(5);
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as uint16);
}
pub unsafe fn DrawEprojSpritemapWithBaseTileOffscreen(
    db: uint8,
    j: uint16,
    x_r20: uint16,
    y_r18: uint16,
    chr_r26: uint16,
    chr_r28: uint16,
    game: &mut Game,
) {
    let mut pp: *const uint8 = RomPtrWithBank(db, j, game);
    let mut idx: ::core::ffi::c_int = ram_word!(game, oam_next_ptr) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = *(pp as *mut uint16) as ::core::ffi::c_int;
    pp = pp.offset(2);
    while n != 0i32 {
        let oam: *mut OamEnt = gOamEnt(idx, game);
        let x: uint16 = (x_r20).wrapping_add(*(pp as *mut uint16));
        (*oam).xcoord = x as uint8;
        let ref mut fresh7 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
            .offset((idx >> 5 as ::core::ffi::c_int) as isize);
        *fresh7 = (*fresh7 as ::core::ffi::c_int
            | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                | (*(pp as *mut int16) < 0i16) as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                    * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16;
        let y: ::core::ffi::c_int =
            *pp.offset(2) as ::core::ffi::c_int + y_r18 as uint8 as ::core::ffi::c_int;
        (*oam).ycoord = (if (y & 0x100 as ::core::ffi::c_int == 0) as ::core::ffi::c_int
            != (*pp.offset(2) as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0)
                as ::core::ffi::c_int
        {
            y
        } else {
            0xf0 as ::core::ffi::c_int
        }) as uint8;
        *(&raw mut (*oam).charnum as *mut uint16) = (chr_r28 as ::core::ffi::c_int
            | chr_r26 as ::core::ffi::c_int + *(pp.offset(3) as *mut uint16) as ::core::ffi::c_int)
            as uint16;
        idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        pp = pp.offset(5);
        n -= 1;
    }
    set_ram_word!(game.oam_next_ptr = idx as uint16);
}
pub unsafe fn GameOverMenu_0_FadeOutConfigGfx(game: &mut Game) {
    HandleFadeOut(game);
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 0i32
    {
        ScreenOff(game);
        QueueSfx3_Max6(1 as uint16, game);
        DisableHdmaObjects(game);
        WaitUntilEndOfVblankAndClearHdma(game);
        let mut v0: uint16 = 0 as uint16;
        loop {
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap
                [(v0 as ::core::ffi::c_int + 384 as ::core::ffi::c_int) as usize] =
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                    .offset(v0 as isize);
            v0 = v0.wrapping_add(1);
            if (((v0 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 512 as ::core::ffi::c_int)
                as int16)
                >= 0i16)
            {
                break;
            }
        }
        let mut v1: uint16 = 0 as uint16;
        loop {
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap
                [(v1 as ::core::ffi::c_int + 640 as ::core::ffi::c_int) as usize] =
                *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x51)
                    .offset((v1 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as isize)
                    as *mut uint16);
            v1 = v1.wrapping_add(1);
            if (((v1 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 54 as ::core::ffi::c_int)
                as int16)
                >= 0i16)
            {
                break;
            }
        }
        MapVramForMenu(game);
        LoadInitialMenuTiles(game);
        set_ram_word!(game.reg_BG1HOFS = 0 as uint16);
        set_ram_word!(game.reg_BG2HOFS = 0 as uint16);
        set_ram_word!(game.reg_BG3HOFS = 0 as uint16);
        set_ram_word!(game.reg_BG1VOFS = 0 as uint16);
        set_ram_word!(game.reg_BG2VOFS = 0 as uint16);
        set_ram_word!(game.reg_BG3VOFS = 0 as uint16);
        LoadMenuPalettes(game);
        let ref mut fresh89 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh89 = (*fresh89).wrapping_add(1);
    }
}
pub unsafe fn GameOverMenu_24_FadeIn(game: &mut Game) {
    HandleFadeIn(game);
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 15i32
    {
        let ref mut fresh90 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh90 = (*fresh90).wrapping_add(1);
    }
}
pub unsafe fn MapVramForMenu(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x52) = 3 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 81 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 88 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x5a) = 92 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 19 as uint8;
}
pub unsafe fn LoadInitialMenuTiles(game: &mut Game) {
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_818DF1: StartDmaCopy = StartDmaCopy::vram_words(1, 0x8e8000, 0x5600);
    SetupDmaTransfer(&raw const unk_818DF1 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x30 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_818E11: StartDmaCopy = StartDmaCopy::vram_words(1, 0xb68000, 0x2000);
    SetupDmaTransfer(&raw const unk_818E11 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x60 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_818E31: StartDmaCopy = StartDmaCopy::vram_words(1, 0xb6c000, 0x2000);
    SetupDmaTransfer(&raw const unk_818E31 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x40 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_818E51: StartDmaCopy = StartDmaCopy::vram_words(1, 0x8ed600, 0x600);
    SetupDmaTransfer(&raw const unk_818E51 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
}
pub unsafe fn LoadMenuPalettes(game: &mut Game) {
    let mut v0: uint16 = 0 as uint16;
    loop {
        let v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(v1 as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x8ee400)) as *const uint8 as *mut uint16)
                .offset(v1 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v1 + 1 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x8ee400)) as *const uint8 as *mut uint16)
                .offset((v1 + 1 as ::core::ffi::c_int) as isize);
        v0 = (v0).wrapping_add(4u16);
        if (((v0) as i16).wrapping_sub(512i16) >= 0i16) {
            break;
        }
    }
}
pub unsafe fn GameOverMenu_3_Main(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v1: int16 = 0;
    if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & kButton_Select as ::core::ffi::c_int)
        != 0i32
        || (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int)
            != 0i32
        || (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int)
            != 0i32
    {
        let ref mut fresh91 = *((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16);
        *fresh91 = (*fresh91) ^ (1u16);
    } else if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & (kButton_B as ::core::ffi::c_int
            | kButton_Start as ::core::ffi::c_int
            | kButton_A as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int)
        != 0i32
    {
        if ram_word!(game, file_select_map_area_index) != 0 {
            let ref mut fresh92 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
            *fresh92 = (*fresh92).wrapping_add(1);
        } else {
            SaveToSram(ram_word!(game, selected_save_slot), game);
            SoftReset(game);
        }
        return;
    }
    v0 = 30720 as int16;
    if ram_word!(game, file_select_map_area_index) != 0 {
        v0 = -(30720 as ::core::ffi::c_int) as int16;
    }
    v1 = (v0) | (0x28i16);
    let v3: *mut OamEnt = gOamEnt(ram_word!(game, oam_next_ptr) as ::core::ffi::c_int, game);
    *(&raw mut (*v3).xcoord as *mut uint16) = v1 as uint16;
    *(&raw mut (*v3).charnum as *mut uint16) = 182 as uint16;
    let ref mut fresh93 = *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16);
    *fresh93 = (*fresh93).wrapping_add(4u16);
}
pub unsafe fn RestorePalettesAndIoAfterDebugGameover(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(v0 as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap[(v0 + 384 as ::core::ffi::c_int) as usize];
        v0 += 1;
        if (((v0 * 2 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16) >= 0i16) {
            break;
        }
    }
    let mut v1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x51)
            .offset((v1 * 2 as ::core::ffi::c_int) as isize) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap[(v1 + 640 as ::core::ffi::c_int) as usize];
        v1 += 1;
        if (((v1 * 2 as ::core::ffi::c_int - 54 as ::core::ffi::c_int) as int16) >= 0i16) {
            break;
        }
    }
}
pub unsafe fn GameOverMenu_5_Continue(game: &mut Game) {
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x40 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_819093: StartDmaCopy = StartDmaCopy::vram_words(1, 0x9ab200, 0x2000);
    SetupDmaTransfer(&raw const unk_819093 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    set_ram_word!(game.game_state = 16 as uint16);
    set_ram_word!(game.file_select_map_area_index = 0 as uint16);
    RestorePalettesAndIoAfterDebugGameover(game);
}
static kGameOverMenuFuncs: [Option<Func_V>; 8] = unsafe {
    [
        Some(GameOverMenu_0_FadeOutConfigGfx as unsafe fn(&mut Game) -> ()),
        Some(GameOverMenu_1_Init as unsafe fn(&mut Game) -> ()),
        Some(GameOverMenu_2_PlayMusic as unsafe fn(&mut Game) -> ()),
        Some(GameOverMenu_3_FadeIn as unsafe fn(&mut Game) -> ()),
        Some(GameOverMenu_4_Main as unsafe fn(&mut Game) -> ()),
        Some(GameOverMenu_5_FadeOutToGameMap as unsafe fn(&mut Game) -> ()),
        Some(GameOverMenu_6_LoadGameMapView as unsafe fn(&mut Game) -> ()),
        Some(GameOverMenu_7_FadeOutToSoftReset as unsafe fn(&mut Game) -> ()),
    ]
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
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 15i32
    {
        let ref mut fresh86 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh86 = (*fresh86).wrapping_add(1);
    }
}
pub unsafe fn GameOverMenu_5_FadeOutToGameMap(game: &mut Game) {
    HandleGameOverBabyMetroid(game);
    DrawMenuSelectionMissile(game);
    HandleFadeOut(game);
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 0i32
    {
        let ref mut fresh83 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh83 = (*fresh83).wrapping_add(1);
    }
}
pub unsafe fn GameOverMenu_7_FadeOutToSoftReset(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    HandleGameOverBabyMetroid(game);
    HandleFadeOut(game);
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 0i32
    {
        SoftReset(game);
    }
}
pub unsafe fn GameOverMenu_6_LoadGameMapView(game: &mut Game) {
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    set_ram_word!(game.game_state = kGameState_5_FileSelectMap as uint16);
    set_ram_word!(game.menu_index = 0 as uint16);
}
pub unsafe fn GameOverMenu_4_Main(game: &mut Game) {
    HandleGameOverBabyMetroid(game);
    DrawMenuSelectionMissile(game);
    if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & kButton_Select as ::core::ffi::c_int)
        != 0i32
        || (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int)
            != 0i32
        || (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int)
            != 0i32
    {
        QueueSfx1_Max6(0x37 as uint16, game);
        let ref mut fresh84 = *((&raw mut game.g_ram as *mut uint8).offset(0x950) as *mut uint16);
        *fresh84 = (*fresh84) ^ (1u16);
    } else if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & kButton_A as ::core::ffi::c_int)
        != 0i32
    {
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .instruction_timer = 180 as uint16;
        if ram_word!(game, file_select_map_area_index) != 0 {
            set_ram_word!(game.menu_index = 7 as uint16);
        } else {
            if ram_word!(game, loading_game_state) as ::core::ffi::c_int
                == kGameState_31_SetUpNewGame as ::core::ffi::c_int
            {
                set_ram_word!(game.game_state = ram_word!(game, loading_game_state));
            } else {
                let ref mut fresh85 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
                *fresh85 = (*fresh85).wrapping_add(1);
            }
            LoadFromSram(ram_word!(game, selected_save_slot), game);
        }
        return;
    }
    let mut v0: uint16 = 160 as uint16;
    if ram_word!(game, file_select_map_area_index) != 0 {
        v0 = 192 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) = 40 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) = v0;
}
pub unsafe fn GameOverMenu_1_Init(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    QueueMusic_Delayed8(0 as uint16, game);
    QueueMusic_Delayed8(0xff03 as uint16, game);
    set_ram_word!(game.cinematic_var5 = 1 as uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(0) = 0 as uint16;
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .menu
            .menu_tilemap as *mut uint8 as *mut uint16)
            .offset(v0 as isize) = 15 as uint16;
        v0 += 1;
        if (((v0 * 2 as ::core::ffi::c_int - 2048 as ::core::ffi::c_int) as int16) >= 0i16) {
            break;
        }
    }
    let v1: ::core::ffi::c_int = ram_word!(game, vram_write_queue_tail) as ::core::ffi::c_int;
    let v2: *mut VramWriteEntry = gVramWriteEntry(
        ram_word!(game, vram_write_queue_tail) as ::core::ffi::c_int,
        game,
    );
    (*v2).size = 2048 as uint16;
    (*v2).src.addr =
        (&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .menu
            .menu_tilemap as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut uint16) = 126 as uint16;
    (*v2).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x58) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    set_ram_word!(game.vram_write_queue_tail = (v1 + 7 as ::core::ffi::c_int) as uint16);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(0x156 as uint16, addr_kMenuTilemap_GameOver as uint16, game);
    LoadMenuTilemap(
        0x38a as uint16,
        addr_kMenuTilemap_FindMetroidLarva as uint16,
        game,
    );
    LoadMenuTilemap(0x414 as uint16, addr_kMenuTilemap_TryAgain as uint16, game);
    LoadMenuTilemap(
        0x4ce as uint16,
        addr_kMenuTilemap_YesReturnToGame as uint16,
        game,
    );
    LoadMenuTilemap(
        0x5ce as uint16,
        addr_kMenuTilemap_NoGoToTitle as uint16,
        game,
    );
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .current_instruction = 0 as uint16;
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .instruction_timer = 0 as uint16;
    HandleGameOverBabyMetroid(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x6f) as *mut uint16) =
        ((*(&raw mut game.g_ram as *mut uint8).offset(0x70) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16;
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    EnableHdmaObjects(game);
    static unk_819254: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0 as uint8,
        field_1: 0x32 as uint8,
        addr: 0x927d as VoidP,
    };
    static unk_81925C: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0 as uint8,
        field_1: 0x31 as uint8,
        addr: 0x928d as VoidP,
    };
    SpawnHdmaObject(
        0x81 as uint8,
        &raw const unk_819254 as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x81 as uint8,
        &raw const unk_81925C as *const ::core::ffi::c_void,
        game,
    );
    ScreenOn(game);
    let ref mut fresh88 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh88 = (*fresh88).wrapping_add(1);
    set_ram_word!(game.screen_fade_delay = 0 as uint16);
    set_ram_word!(game.screen_fade_counter = 0 as uint16);
    set_ram_word!(game.file_select_map_area_index = 0 as uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) = 40 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) = 160 as uint16;
}
pub unsafe fn GameOverMenu_2_PlayMusic(game: &mut Game) {
    if HasQueuedMusic(game) == 0 {
        let ref mut fresh87 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh87 = (*fresh87).wrapping_add(1);
        QueueMusic_Delayed8(4 as uint16, game);
    }
}
static kFileSelectMenuFuncs: [Option<Func_V>; 34] = unsafe {
    [
        Some(FileSelectMenu_0_FadeOutConfigGfx as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_1_LoadFileSelectMenuBG2 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_2_InitMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_3_FadeInToMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_4_Main as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_5_FadeOutFromMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_6_FileCopyInit as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_7_FadeInFromMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_8 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_9_InitializeSelectDest as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_10_FileCopySelectDest as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_11_InitializeConfirm as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_12_FileCopyConfirm as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_13_FileCopyDoIt as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_14_CopyCompleted as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_15_FadeOutToMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_16 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_17_FadeInToMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_18 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_5_FadeOutFromMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_20_FileClearInit as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_7_FadeInFromMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_22_FileClearSelectSlot as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_23_FileClearInitConfirm as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_24_FileClearConfirm as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_25_FileClearDoClear as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_26_ClearCompleted as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_15_FadeOutToMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_16 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_17_FadeInToMain as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_30 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_31_TurnSamusHelmet as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_32_FadeOutToOptions as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMenu_33_FadeOutToTitle as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn FileSelectMenu(game: &mut Game) {
    kFileSelectMenuFuncs[ram_word!(game, menu_index) as usize].expect("non-null function pointer")(
        game,
    );
}
pub unsafe fn FileSelectMenu_0_FadeOutConfigGfx(game: &mut Game) {
    HandleFadeOut(game);
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        != 0i32
    {
        return;
    }
    ScreenOff(game);
    QueueSfx3_Max6(1 as uint16, game);
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    let ref mut fresh82 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh82 = (*fresh82).wrapping_add(1);
    MapVramForMenu(game);
    LoadInitialMenuTiles(game);
    set_ram_word!(game.reg_BG1HOFS = 0 as uint16);
    set_ram_word!(game.reg_BG2HOFS = 0 as uint16);
    set_ram_word!(game.reg_BG3HOFS = 0 as uint16);
    set_ram_word!(game.reg_BG1VOFS = 0 as uint16);
    set_ram_word!(game.reg_BG2VOFS = 0 as uint16);
    set_ram_word!(game.reg_BG3VOFS = 0 as uint16);
    LoadFileSelectPalettes(game);
}
pub unsafe fn LoadFileSelectPalettes(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        let v1: ::core::ffi::c_int = v0 >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(v1 as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x8ee400)) as *const uint8 as *mut uint16)
                .offset(v1 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v1 + 1 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x8ee400)) as *const uint8 as *mut uint16)
                .offset((v1 + 1 as ::core::ffi::c_int) as isize);
        v0 += 4 as ::core::ffi::c_int;
        if (((v0 - 512 as ::core::ffi::c_int) as int16) >= 0i16) {
            break;
        }
    }
}
pub unsafe fn FileSelectMenu_32_FadeOutToOptions(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    DrawBorderAroundSamusData(game);
    DrawFileSelectSamusHelmets(game);
    HandleFadeOut(game);
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 0i32
    {
        set_ram_word!(game.game_state = kGameState_2_GameOptionsMenu as uint16);
        set_ram_word!(game.menu_index = 0 as uint16);
        let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
                .offset(v0 as isize) as *mut uint16) = 0 as uint16;
            v0 += 2 as ::core::ffi::c_int;
            if (((v0 - 48 as ::core::ffi::c_int) as int16) >= 0i16) {
                break;
            }
        }
    }
}
pub unsafe fn FileSelectMenu_33_FadeOutToTitle(game: &mut Game) {
    DrawBorderAroundSamusData(game);
    DrawFileSelectSamusHelmets(game);
    HandleFadeOut(game);
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 0i32
    {
        SoftReset(game);
    }
}
pub unsafe fn FileSelectMenu_5_FadeOutFromMain(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    FileSelectMenu_15_FadeOutToMain(game);
}
pub unsafe fn FileSelectMenu_15_FadeOutToMain(game: &mut Game) {
    HandleFadeOut(game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x57) =
        (*(&raw mut game.g_ram as *mut uint8).offset(0x57) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            | 16 as ::core::ffi::c_int
                * (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int)
                ^ 0xf0 as ::core::ffi::c_int) as uint8;
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 0i32
    {
        ScreenOff(game);
        let ref mut fresh48 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh48 = (*fresh48).wrapping_add(1);
    }
}
static kMenuSelectionMissileXY: [uint16; 12] = [
    0x30, 0xe, 0x58, 0xe, 0x80, 0xe, 0xa3, 0xe, 0xbb, 0xe, 0xd3, 0xe,
];
pub unsafe fn FileSelectMenu_17_FadeInToMain(game: &mut Game) {
    let v0: ::core::ffi::c_int = ((4u16).wrapping_mul(ram_word!(game, selected_save_slot)))
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) =
        kMenuSelectionMissileXY[v0 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) =
        kMenuSelectionMissileXY[(v0 + 1 as ::core::ffi::c_int) as usize];
    FileSelectMenu_7_FadeInFromMain(game);
}
pub unsafe fn FileSelectMenu_7_FadeInFromMain(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    HandleFadeIn(game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x57) =
        (*(&raw mut game.g_ram as *mut uint8).offset(0x57) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            | 16 as ::core::ffi::c_int
                * (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int)
                ^ 0xf0 as ::core::ffi::c_int) as uint8;
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 15i32
    {
        let ref mut fresh46 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh46 = (*fresh46).wrapping_add(1);
    }
}
pub unsafe fn FileSelectMenu_6_FileCopyInit(game: &mut Game) {
    let ref mut fresh76 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh76 = (*fresh76).wrapping_add(1);
    FileSelectMenu_Func1(game);
}
pub unsafe fn FileSelectMenu_Func1(game: &mut Game) {
    ClearMenuTilemap(game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x52 as uint16,
        addr_kMenuTilemap_DataCopyMode as uint16,
        game,
    );
    LoadMenuTilemap(
        0x150 as uint16,
        addr_kMenuTilemap_CopyWhichData as uint16,
        game,
    );
    LoadMenuExitTilemap(game);
    DrawFileCopySaveFileInfo(game);
    SetInitialFileCopyMenuSelection(game);
    SetFileCopyMenuSelectionMissilePosition(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17) = 0 as uint16;
}
pub unsafe fn sub_819591(game: &mut Game) {
    SetInitialFileCopyMenuSelection(game);
}
pub unsafe fn SetInitialFileCopyMenuSelection(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int;
    let mut v1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (v0 & 1 as ::core::ffi::c_int == 0) {
        v0 >>= 1 as ::core::ffi::c_int;
        v1 += 1;
        if (((v1 - 3 as ::core::ffi::c_int) as int16) >= 0i16) {
            break;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) = v1 as uint16;
}
pub unsafe fn ClearMenuTilemap(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 2046 as ::core::ffi::c_int;
    while i >= 0i32 {
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768) as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 15 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn LoadMenuExitTilemap(game: &mut Game) {
    LoadMenuTilemap(0x688 as uint16, addr_kMenuTilemap_Exit as uint16, game);
}
pub unsafe fn DrawFileCopySaveFileInfo(game: &mut Game) {
    LoadFromSram_(0 as uint16, game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    if (ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int & 1 as ::core::ffi::c_int)
        == 0i32
    {
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .palette_index = 1024 as uint16;
    }
    DrawFileCopySaveSlotAInfo(game);
    LoadFromSram_(1 as uint16, game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    if (ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int & 2 as ::core::ffi::c_int)
        == 0i32
    {
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .palette_index = 1024 as uint16;
    }
    DrawFileCopySaveSlotBInfo(game);
    LoadFromSram_(2 as uint16, game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    if (ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int & 4 as ::core::ffi::c_int)
        == 0i32
    {
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .palette_index = 1024 as uint16;
    }
    DrawFileCopySaveSlotCInfo(game);
    QueueTransferOfMenuTilemapToVramBG1(game);
}
pub unsafe fn DrawFileCopySaveSlotAInfo(game: &mut Game) {
    DrawFileSelectionHealth(
        ((ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int & 1 as ::core::ffi::c_int)
            == 0i32) as uint16,
        0x218 as uint16,
        game,
    );
    DrawFileSelectionTime(
        ((ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int & 1 as ::core::ffi::c_int)
            == 0i32) as uint16,
        0x272 as uint16,
        game,
    );
    LoadMenuTilemap(0x234 as uint16, addr_kMenuTilemap_TIME as uint16, game);
    LoadMenuTilemap(0x208 as uint16, addr_kMenuTilemap_SamusA as uint16, game);
}
pub unsafe fn DrawFileCopySaveSlotBInfo(game: &mut Game) {
    DrawFileSelectionHealth(
        (!(ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int) & 2 as ::core::ffi::c_int)
            as uint16,
        0x318 as uint16,
        game,
    );
    DrawFileSelectionTime(
        (!(ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int) & 2 as ::core::ffi::c_int)
            as uint16,
        0x372 as uint16,
        game,
    );
    LoadMenuTilemap(0x334 as uint16, addr_kMenuTilemap_TIME as uint16, game);
    LoadMenuTilemap(0x308 as uint16, addr_kMenuTilemap_SamusB as uint16, game);
}
pub unsafe fn DrawFileCopySaveSlotCInfo(game: &mut Game) {
    DrawFileSelectionHealth(
        (!(ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int) & 4 as ::core::ffi::c_int)
            as uint16,
        0x418 as uint16,
        game,
    );
    DrawFileSelectionTime(
        (!(ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int) & 4 as ::core::ffi::c_int)
            as uint16,
        0x472 as uint16,
        game,
    );
    LoadMenuTilemap(0x434 as uint16, addr_kMenuTilemap_TIME as uint16, game);
    LoadMenuTilemap(0x408 as uint16, addr_kMenuTilemap_SamusC as uint16, game);
}
pub unsafe fn QueueTransferOfMenuTilemapToVramBG1(game: &mut Game) {
    let v0: uint16 = ram_word!(game, vram_write_queue_tail);
    let v1: *mut VramWriteEntry = gVramWriteEntry(
        ram_word!(game, vram_write_queue_tail) as ::core::ffi::c_int,
        game,
    );
    (*v1).size = 2048 as uint16;
    (*v1).src.addr =
        (&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .menu
            .menu_tilemap as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    *(&raw mut (*v1).src.bank as *mut uint16) = 126 as uint16;
    (*v1).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x58) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(7u16));
}
static kBitShl: [uint8; 3] = [1, 2, 4];
pub unsafe fn FileSelectMenu_8(game: &mut Game) {
    let mut v0: uint8 = 0;
    DrawBorderAroundDataCopyMode(game);
    DrawMenuSelectionMissile(game);
    if ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        != 0i32
    {
        QueueSfx1_Max6(0x37 as uint16, game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) == 3u16 {
            let ref mut fresh73 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
            *fresh73 = (*fresh73).wrapping_add(7u16);
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15);
            let ref mut fresh74 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
            *fresh74 = (*fresh74).wrapping_add(1);
        }
    } else if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        let ref mut fresh75 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh75 = (*fresh75).wrapping_add(7u16);
        QueueSfx1_Max6(0x37 as uint16, game);
    } else {
        let current_block_18: u64;
        if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int)
            != 0i32
        {
            v0 = *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15)
                as uint8;
            loop {
                v0 = v0.wrapping_sub(1);
                if ((v0 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) != 0i32) {
                    current_block_18 = 1109700713171191020;
                    break;
                }
                if (kBitShl[v0 as usize] as ::core::ffi::c_int
                    & ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int)
                    != 0i32
                {
                    current_block_18 = 1398313740379482408;
                    break;
                }
            }
        } else if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int)
            != 0i32
        {
            v0 = *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15)
                as uint8;
            loop {
                v0 = v0.wrapping_add(1);
                if (((v0) as i8).wrapping_sub(4i8) >= 0i8) {
                    current_block_18 = 1109700713171191020;
                    break;
                }
                if v0 == 3u8
                    || (kBitShl[v0 as usize] as ::core::ffi::c_int
                        & ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int)
                        != 0i32
                {
                    current_block_18 = 1398313740379482408;
                    break;
                }
            }
        } else {
            current_block_18 = 1109700713171191020;
        }
        match current_block_18 {
            1109700713171191020 => {}
            _ => {
                *(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15)
                    as *mut uint16 as *mut uint8)
                    .offset(0) = v0;
                QueueSfx1_Max6(0x37 as uint16, game);
                SetFileCopyMenuSelectionMissilePosition(game);
                return;
            }
        }
        SetFileCopyMenuSelectionMissilePosition(game);
    };
}
pub unsafe fn SetFileCopyMenuSelectionMissilePosition(game: &mut Game) {
    static kFileCopyMissileY: [uint16; 4] =
        [72 as uint16, 104 as uint16, 136 as uint16, 211 as uint16];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) =
        kFileCopyMissileY[*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset(15) as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) = 22 as uint16;
}
pub unsafe fn FileSelectMenu_9_InitializeSelectDest(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    DrawFileCopySelectDestinationSaveFileInfo(game);
    let ref mut fresh72 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh72 = (*fresh72).wrapping_add(1);
    let mut v0: uint16 = 0 as uint16;
    while (v0 == *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)) {
        v0 = v0.wrapping_add(1);
        if (v0 as ::core::ffi::c_int - 3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0)
        {
            break;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) = v0;
    SetFileCopyMenuSelectionMissilePosition(game);
}
pub unsafe fn DrawFileCopySelectDestinationSaveFileInfo(game: &mut Game) {
    ClearMenuTilemap(game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x52 as uint16,
        addr_kMenuTilemap_DataCopyMode as uint16,
        game,
    );
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x148 as uint16,
        addr_kMenuTilemap_CopySamusToWhere as uint16,
        game,
    );
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .pause_menu_map_tilemap[944 as ::core::ffi::c_int as usize] =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16))
            .wrapping_add(8298u16);
    LoadMenuExitTilemap(game);
    LoadFromSram_(0 as uint16, game);
    let mut v0: uint16 = 1024 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) != 0 {
        v0 = 0 as uint16;
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = v0;
    DrawFileCopySaveSlotAInfo(game);
    LoadFromSram_(1 as uint16, game);
    let mut v1: uint16 = 1024 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) != 1u16 {
        v1 = 0 as uint16;
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = v1;
    DrawFileCopySaveSlotBInfo(game);
    LoadFromSram_(2 as uint16, game);
    let mut v2: uint16 = 1024 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) != 2u16 {
        v2 = 0 as uint16;
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = v2;
    DrawFileCopySaveSlotCInfo(game);
    QueueTransferOfMenuTilemapToVramBG1(game);
}
pub unsafe fn FileSelectMenu_10_FileCopySelectDest(game: &mut Game) {
    let mut current_block: u64;
    static kFileCopySelectDest_MissileY: [uint16; 4] =
        [72 as uint16, 104 as uint16, 136 as uint16, 212 as uint16];
    let mut v0: uint16 = 0;
    DrawBorderAroundDataCopyMode(game);
    DrawMenuSelectionMissile(game);
    if ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        != 0i32
    {
        QueueSfx1_Max6(0x37 as uint16, game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) != 3u16 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15);
            let ref mut fresh69 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
            *fresh69 = (*fresh69).wrapping_add(1);
            current_block = 16545170798630791882;
        } else {
            let ref mut fresh70 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
            *fresh70 = (*fresh70).wrapping_add(5u16);
            current_block = 16203760046146113240;
        }
    } else if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int)
            != 0i32
        {
            QueueSfx1_Max6(0x37 as uint16, game);
            v0 = *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15);
            loop {
                v0 = v0.wrapping_sub(1);
                if ((v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32) {
                    current_block = 16545170798630791882;
                    break;
                }
                if v0
                    != *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                        .offset(16)
                {
                    current_block = 1572352429258197161;
                    break;
                }
            }
        } else if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int)
            != 0i32
        {
            QueueSfx1_Max6(0x37 as uint16, game);
            v0 = *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15);
            loop {
                v0 = v0.wrapping_add(1);
                if (v0 == 4u16) {
                    current_block = 16545170798630791882;
                    break;
                }
                if v0
                    != *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                        .offset(16)
                {
                    current_block = 1572352429258197161;
                    break;
                }
            }
        } else {
            current_block = 16545170798630791882;
        }
        match current_block {
            16545170798630791882 => {}
            _ => {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) =
                    v0;
                current_block = 16545170798630791882;
            }
        }
    } else {
        let ref mut fresh71 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh71 = (*fresh71).wrapping_sub(2u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16);
        QueueSfx1_Max6(0x37 as uint16, game);
        FileSelectMenu_Func1(game);
        current_block = 16203760046146113240;
    }
    match current_block {
        16203760046146113240 => {}
        _ => {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) =
                kFileCopySelectDest_MissileY[*((&raw mut game.g_ram as *mut uint8).offset(0x1997)
                    as *mut uint16)
                    .offset(15) as usize];
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) =
                22 as uint16;
        }
    }
}
pub unsafe fn FileSelectMenu_11_InitializeConfirm(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x144 as uint16,
        addr_kMenuTilemap_CopySamusToSamus as uint16,
        game,
    );
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .pause_menu_map_tilemap[942 as ::core::ffi::c_int as usize] =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16))
            .wrapping_add(8298u16);
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .pause_menu_map_tilemap[955 as ::core::ffi::c_int as usize] =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17))
            .wrapping_add(8298u16);
    DrawFileCopyClearConfirmation(game);
    let ref mut fresh68 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh68 = (*fresh68).wrapping_add(1);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) = 0 as uint16;
    set_ram_word!(game.cinematic_var6 = 8 as uint16);
}
pub unsafe fn DrawFileCopyClearConfirmation(game: &mut Game) {
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(0x514 as uint16, addr_kMenuTilemap_IsThisOk as uint16, game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(0x59c as uint16, addr_kMenuTilemap_Yes as uint16, game);
    let mut v0: ::core::ffi::c_int = 832 as ::core::ffi::c_int;
    loop {
        // Menu rows live in the full $3600..$3dff RAM tilemap.
        *((&raw mut game.g_ram as *mut uint8)
            .add(0x3600)
            .cast::<uint16>()
            .add(v0 as usize)) = 15 as uint16;
        v0 += 1;
        if (((v0 * 2 as ::core::ffi::c_int - 1728 as ::core::ffi::c_int) as int16) >= 0i16) {
            break;
        }
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(0x65c as uint16, addr_kMenuTilemap_No as uint16, game);
    DrawFileCopyConfirmationSaveFileInfo(game);
}
pub unsafe fn DrawFileCopyConfirmationSaveFileInfo(game: &mut Game) {
    LoadFromSram_(0 as uint16, game);
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) != 0u16
        && *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17) != 0u16
    {
        v0 = 1024 as ::core::ffi::c_int;
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = v0 as uint16;
    DrawFileCopySaveSlotAInfo(game);
    LoadFromSram_(1 as uint16, game);
    let mut v1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) != 1u16
        && *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17) != 1u16
    {
        v1 = 1024 as ::core::ffi::c_int;
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = v1 as uint16;
    DrawFileCopySaveSlotBInfo(game);
    LoadFromSram_(2 as uint16, game);
    let mut v2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) != 2u16
        && *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17) != 2u16
    {
        v2 = 1024 as ::core::ffi::c_int;
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = v2 as uint16;
    DrawFileCopySaveSlotCInfo(game);
    QueueTransferOfMenuTilemapToVramBG1(game);
}
pub unsafe fn FileSelectMenu_12_FileCopyConfirm(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    DrawMenuSelectionMissile(game);
    HandleFileCopyArrowPalette(game);
    DrawFileCopyArrow(game);
    if ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & (kButton_Up as ::core::ffi::c_int | kButton_Down as ::core::ffi::c_int)
        != 0i32
    {
        let ref mut fresh64 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15);
        *fresh64 = (*fresh64) ^ (1u16);
        QueueSfx1_Max6(0x37 as uint16, game);
    } else {
        if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
            != 0i32
        {
            let ref mut fresh65 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
            *fresh65 = (*fresh65).wrapping_sub(3u16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17);
            QueueSfx1_Max6(0x37 as uint16, game);
            return;
        }
        if ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
            & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
            != 0i32
        {
            QueueSfx1_Max6(0x38 as uint16, game);
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) != 0
            {
                let ref mut fresh66 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
                *fresh66 = (*fresh66).wrapping_sub(4u16);
                FileSelectMenu_Func1(game);
            } else {
                let ref mut fresh67 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
                *fresh67 = (*fresh67).wrapping_add(1);
            }
            return;
        }
    }
    let mut v0: uint16 = 184 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) != 0 {
        v0 = 208 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) = v0;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) = 94 as uint16;
}
pub unsafe fn HandleFileCopyArrowPalette(game: &mut Game) {
    if ram_word!(game, cinematic_var6) != 0 {
        let ref mut fresh63 = *((&raw mut game.g_ram as *mut uint8).offset(0x198f) as *mut uint16);
        *fresh63 = (*fresh63).wrapping_sub(1);
        if *fresh63 == 0 {
            set_ram_word!(game.cinematic_var6 = 4 as uint16);
            let v0: uint16 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(145);
            let mut v1: uint16 = 0 as uint16;
            loop {
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                    ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 145 as ::core::ffi::c_int) as isize,
                ) = *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                    ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 146 as ::core::ffi::c_int) as isize,
                );
                v1 = (v1).wrapping_add(2u16);
                if (((v1) as i16).wrapping_sub(12i16) >= 0i16) {
                    break;
                }
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(151) = v0;
        }
    }
}
pub unsafe fn FileSelectMenu_13_FileCopyDoIt(game: &mut Game) {
    static kBitShl_16bit: [uint16; 3] = [1, 2, 4];
    DrawBorderAroundDataCopyMode(game);
    DrawMenuSelectionMissile(game);
    HandleFileCopyArrowPalette(game);
    DrawFileCopyArrow(game);
    let src_addr: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x81812b)) as *const uint8
        as *mut uint16)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
                as isize,
        );
    let dst_addr: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x81812b)) as *const uint8
        as *mut uint16)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17)
                as isize,
        );
    memcpy(
        game.g_sram.offset(dst_addr as isize) as *mut uint8 as *mut ::core::ffi::c_void,
        game.g_sram.offset(src_addr as isize) as *mut uint8 as *const ::core::ffi::c_void,
        1628 as size_t,
    );
    let v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1997)
        as *mut uint16)
        .offset(16) as ::core::ffi::c_int;
    let v10: ::core::ffi::c_int = *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 0x1ff0 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int;
    let v9: ::core::ffi::c_int = *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 0x1ff8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int;
    let v8: ::core::ffi::c_int = *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 0 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int;
    let v4: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1997)
        as *mut uint16)
        .offset(17) as ::core::ffi::c_int;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v4 + 8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16);
    *(game.g_sram.offset((2 as ::core::ffi::c_int * v4) as isize) as *mut uint8 as *mut uint16) =
        v8 as uint16;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v4 + 0x1ff8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = v9 as uint16;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v4 + 0x1ff0 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = v10 as uint16;
    let ref mut fresh61 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh61 = (*fresh61).wrapping_add(1);
    let mut v5: ::core::ffi::c_int = 640 as ::core::ffi::c_int;
    loop {
        // Menu rows live in the full $3600..$3dff RAM tilemap.
        *((&raw mut game.g_ram as *mut uint8)
            .add(0x3600)
            .cast::<uint16>()
            .add(v5 as usize)) = 15 as uint16;
        v5 += 1;
        if (((v5 * 2 as ::core::ffi::c_int - 1856 as ::core::ffi::c_int) as int16) >= 0i16) {
            break;
        }
    }
    let ref mut fresh62 = *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16);
    *fresh62 = (*fresh62)
        | (kBitShl_16bit[*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset(17) as usize]);
    let mut v6: ::core::ffi::c_int = ((4 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17)
            as ::core::ffi::c_int
        + 9 as ::core::ffi::c_int)
        << 6 as ::core::ffi::c_int)
        + 24 as ::core::ffi::c_int;
    let mut v7: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768) as *mut uint16 as *mut uint8)
            .offset(v6 as isize) as *mut uint16) = 15 as uint16;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(736) as *mut uint16 as *mut uint8)
            .offset(v6 as isize) as *mut uint16) = 15 as uint16;
        v6 += 2 as ::core::ffi::c_int;
        v7 += 2 as ::core::ffi::c_int;
        if (((v7 - 22 as ::core::ffi::c_int) as int16) >= 0i16) {
            break;
        }
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x510 as uint16,
        addr_kMenuTilemap_CopyCompleted as uint16,
        game,
    );
    DrawFileCopyConfirmationSaveFileInfo(game);
    RtlWriteSram(game);
}
pub unsafe fn FileSelectMenu_14_CopyCompleted(game: &mut Game) {
    DrawBorderAroundDataCopyMode(game);
    if ram_word!(game, joypad1_newkeys) != 0 {
        QueueSfx1_Max6(0x37 as uint16, game);
        let ref mut fresh60 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh60 = (*fresh60).wrapping_add(1);
        let mut slot: ::core::ffi::c_int =
            *(game.g_sram.offset(0x1fec) as *mut uint16) as ::core::ffi::c_int;
        if slot & 0x8000 as ::core::ffi::c_int != 0
            || *(game.g_sram.offset(0x1fec) as *mut uint16) as ::core::ffi::c_int
                - 3 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            || slot & *(game.g_sram.offset(0x1fee) as *mut uint16) as ::core::ffi::c_int != 0
        {
            slot = 0 as ::core::ffi::c_int;
        }
        set_ram_word!(game.selected_save_slot = slot as uint16);
    }
}
pub unsafe fn FileSelectMenu_18(game: &mut Game) {
    let ref mut fresh59 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh59 = (*fresh59).wrapping_sub(14u16);
}
pub unsafe fn FileSelectMenu_20_FileClearInit(game: &mut Game) {
    DrawBorderAroundDataClearMode(game);
    let ref mut fresh58 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh58 = (*fresh58).wrapping_add(1);
    InitFileSelectMenuFileClear(game);
}
pub unsafe fn InitFileSelectMenuFileClear(game: &mut Game) {
    ClearMenuTilemap(game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x50 as uint16,
        addr_kMenuTilemap_DataClearMode as uint16,
        game,
    );
    LoadMenuTilemap(
        0x140 as uint16,
        addr_kMenuTilemap_ClearWhichData as uint16,
        game,
    );
    LoadMenuExitTilemap(game);
    DrawFileCopySaveFileInfo(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) = 0 as uint16;
    SetInitialFileCopyMenuSelection(game);
    SetFileClearMenuMissilePos(game);
}
pub unsafe fn FileSelectMenu_22_FileClearSelectSlot(game: &mut Game) {
    static kBitShl_: [uint8; 3] = [1, 2, 4];
    let mut v0: uint16 = 0;
    let mut v1: uint8 = 0;
    DrawBorderAroundDataClearMode(game);
    DrawMenuSelectionMissile(game);
    *(&raw mut v0 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) = *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    );
    if ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        != 0i32
    {
        QueueSfx1_Max6(0x37 as uint16, game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) != 3u16 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15);
            let ref mut fresh56 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
            *fresh56 = (*fresh56).wrapping_add(1);
            return;
        }
    } else if ((ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32)
    {
        let current_block_18: u64;
        if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int)
            != 0i32
        {
            v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15)
                as uint8;
            loop {
                v1 = v1.wrapping_sub(1);
                if ((v1 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) != 0i32) {
                    current_block_18 = 11307063007268554308;
                    break;
                }
                if (kBitShl_[v1 as usize] as ::core::ffi::c_int
                    & ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int)
                    != 0i32
                {
                    current_block_18 = 11325801268588962582;
                    break;
                }
            }
        } else if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int)
            != 0i32
        {
            v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15)
                as uint8;
            loop {
                v1 = v1.wrapping_add(1);
                if (((v1) as i8).wrapping_sub(4i8) >= 0i8) {
                    current_block_18 = 11307063007268554308;
                    break;
                }
                if v1 == 3u8
                    || (kBitShl_[v1 as usize] as ::core::ffi::c_int
                        & ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int)
                        != 0i32
                {
                    current_block_18 = 11325801268588962582;
                    break;
                }
            }
        } else {
            current_block_18 = 11307063007268554308;
        }
        match current_block_18 {
            11307063007268554308 => {}
            _ => {
                *(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15)
                    as *mut uint16 as *mut uint8)
                    .offset(0) = v1;
                *(&raw mut v0 as *mut uint8).offset(0) = 55 as uint8;
                QueueSfx1_Max6(v0, game);
                SetFileClearMenuMissilePos(game);
                return;
            }
        }
        SetFileClearMenuMissilePos(game);
        return;
    }
    QueueSfx1_Max6(0x37 as uint16, game);
    let ref mut fresh57 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh57 = (*fresh57).wrapping_add(5u16);
}
pub unsafe fn SetFileClearMenuMissilePos(game: &mut Game) {
    static kFileClear_MissileY: [uint16; 4] =
        [72 as uint16, 104 as uint16, 136 as uint16, 211 as uint16];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) =
        kFileClear_MissileY[*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset(15) as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) = 22 as uint16;
}
pub unsafe fn FileSelectMenu_23_FileClearInitConfirm(game: &mut Game) {
    DrawBorderAroundDataClearMode(game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x140 as uint16,
        addr_kMenuTilemap_ClearSamusA as uint16,
        game,
    );
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .pause_menu_map_tilemap[949 as ::core::ffi::c_int as usize] =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16))
            .wrapping_add(8298u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17) = 3 as uint16;
    DrawFileCopyClearConfirmation(game);
    let ref mut fresh55 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh55 = (*fresh55).wrapping_add(1);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) = 0 as uint16;
}
pub unsafe fn FileSelectMenu_24_FileClearConfirm(game: &mut Game) {
    let current_block: u64;
    let mut v0: uint16 = 0;
    DrawBorderAroundDataClearMode(game);
    DrawMenuSelectionMissile(game);
    if ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & (kButton_Up as ::core::ffi::c_int | kButton_Down as ::core::ffi::c_int)
        != 0i32
    {
        let ref mut fresh52 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15);
        *fresh52 = (*fresh52) ^ (1u16);
        QueueSfx1_Max6(0x37 as uint16, game);
    } else {
        if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
            != 0i32
        {
            current_block = 18062911466027671358;
        } else if ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
            & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
            == 0i32
        {
            current_block = 504450154722759016;
        } else {
            QueueSfx1_Max6(0x38 as uint16, game);
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) != 0
            {
                current_block = 18062911466027671358;
            } else {
                let ref mut fresh54 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
                *fresh54 = (*fresh54).wrapping_add(1);
                return;
            }
        }
        match current_block {
            504450154722759016 => {}
            _ => {
                let ref mut fresh53 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
                *fresh53 = (*fresh53).wrapping_sub(2u16);
                *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16);
                QueueSfx1_Max6(0x37 as uint16, game);
                InitFileSelectMenuFileClear(game);
                return;
            }
        }
    }
    v0 = 184 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) != 0 {
        v0 = 208 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) = v0;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) = 94 as uint16;
}
pub unsafe fn FileSelectMenu_25_FileClearDoClear(game: &mut Game) {
    static kBitShl_Not: [uint16; 3] = [0xfffe, 0xfffd, 0xfffb];
    DrawBorderAroundDataClearMode(game);
    let sram_addr: ::core::ffi::c_int = *(game.g_rom.offset(crate::rtl::lorom_offset(0x81812b))
        as *const uint8 as *mut uint16)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
                as isize,
        ) as ::core::ffi::c_int;
    memset(
        game.g_sram.offset(sram_addr as isize) as *mut uint8 as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        1628 as size_t,
    );
    let v1: uint16 = (2u16).wrapping_mul(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16),
    );
    let v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1997)
        as *mut uint16)
        .offset(16) as ::core::ffi::c_int;
    *(game.g_sram.offset((2 as ::core::ffi::c_int * v2) as isize) as *mut uint8 as *mut uint16) =
        0 as uint16;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = 0 as uint16;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 0x1ff0 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = 0 as uint16;
    *(game
        .g_sram
        .offset((2 as ::core::ffi::c_int * v2 + 0x1ff8 as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut uint16) = 0 as uint16;
    let ref mut fresh50 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh50 = (*fresh50).wrapping_add(1);
    NewSaveFile(game);
    LoadFromSram(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16),
        game,
    );
    set_ram_word!(
        game.area_index =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
    );
    LoadMirrorOfExploredMapTiles(game);
    let mut v3: uint16 = 640 as uint16;
    loop {
        // File-clear rows share the full menu tilemap at $3600.
        *((&raw mut game.g_ram as *mut uint8)
            .add(0x3600)
            .cast::<uint16>()
            .add(v3 as usize)) = 15 as uint16;
        v3 = v3.wrapping_add(1);
        if (((v3 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 1856 as ::core::ffi::c_int)
            as int16)
            >= 0i16)
        {
            break;
        }
    }
    let ref mut fresh51 = *((&raw mut game.g_ram as *mut uint8).offset(0x954) as *mut uint16);
    *fresh51 = (*fresh51)
        & (kBitShl_Not[*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset(16) as usize]);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(
        0x500 as uint16,
        addr_kMenuTilemap_DataCleared as uint16,
        game,
    );
    DrawFileCopyConfirmationSaveFileInfo(game);
    RtlWriteSram(game);
}
pub unsafe fn FileSelectMenu_26_ClearCompleted(game: &mut Game) {
    DrawBorderAroundDataClearMode(game);
    if ram_word!(game, joypad1_newkeys) != 0 {
        let current_block_7: u64;
        QueueSfx1_Max6(0x37 as uint16, game);
        let ref mut fresh49 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh49 = (*fresh49).wrapping_add(1);
        if (LoadFromSram(0 as uint16, game) != 0) {
            if LoadFromSram(1 as uint16, game) != 0 {
                if LoadFromSram(2 as uint16, game) != 0 {
                    current_block_7 = 17037298857681653719;
                } else {
                    set_ram_word!(game.selected_save_slot = 2 as uint16);
                    current_block_7 = 1394248824506584008;
                }
            } else {
                set_ram_word!(game.selected_save_slot = 1 as uint16);
                current_block_7 = 1394248824506584008;
            }
            match current_block_7 {
                17037298857681653719 => {}
                _ => {
                    return;
                }
            }
        }
        set_ram_word!(game.selected_save_slot = 0 as uint16);
    }
}
pub unsafe fn FileSelectMenu_30(game: &mut Game) {
    DrawBorderAroundSamusData(game);
    let ref mut fresh45 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh45 = (*fresh45).wrapping_sub(26u16);
}
pub unsafe fn FileSelectMenu_31_TurnSamusHelmet(game: &mut Game) {
    DrawMenuSelectionMissile(game);
    DrawBorderAroundSamusData(game);
    DrawFileSelectSlotSamusHelmet(4 as uint16, game);
    DrawFileSelectSlotSamusHelmet(6 as uint16, game);
    DrawFileSelectSlotSamusHelmet(8 as uint16, game);
    if ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        != 0i32
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(2) == 7u16
            && ram_word!(game, cinematic_var7) == 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(3) == 7u16
            && ram_word!(game, cinematic_var8) == 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(4) == 7u16
            && ram_word!(game, cinematic_var9) == 0
    {
        let ref mut fresh44 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh44 = (*fresh44).wrapping_add(1);
    }
}
pub unsafe fn DrawFileSelectSamusHelmets(game: &mut Game) {
    set_ram_word!(game.cinematic_var7 = 0 as uint16);
    DrawFileSelectSlotSamusHelmet(4 as uint16, game);
    set_ram_word!(game.cinematic_var8 = 0 as uint16);
    DrawFileSelectSlotSamusHelmet(6 as uint16, game);
    set_ram_word!(game.cinematic_var9 = 0 as uint16);
    DrawFileSelectSlotSamusHelmet(8 as uint16, game);
}
pub unsafe fn DrawFileSelectSlotSamusHelmet(k: uint16, game: &mut Game) {
    let v0: uint16 = k;
    let mut v1: int16 = 0;
    v1 = *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) as int16;
    if v1 != 0 {
        let v2: uint16 = ((v1) as u16).wrapping_sub(1u16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = v2;
        if v2 == 0 {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
                .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 8 as uint16;
            let v3: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1997)
                as *mut uint16)
                .offset(v3 as isize))
            .wrapping_add(1u16);
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                .offset(v3 as isize) as ::core::ffi::c_int
                - 7 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
                    .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
                v4 = 7 as uint16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                .offset(v3 as isize) = v4;
        }
    }
    let v5: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v6: uint16 = (2u16).wrapping_mul(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(v5 as isize),
    );
    static kDrawFileSlotHelmet_Spritemaps: [uint16; 9] =
        [0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x33];
    DrawMenuSpritemap(
        kDrawFileSlotHelmet_Spritemaps
            [(v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize],
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((v5 + 5 as ::core::ffi::c_int) as isize),
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((v5 + 10 as ::core::ffi::c_int) as isize),
        3584 as uint16,
        game,
    );
}
static kFileSelectMapFuncs: [Option<Func_V>; 23] = unsafe {
    [
        Some(FileSelectMap_0 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_1 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_2_LoadAreaSelectForegroundTilemap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_3_LoadAreaSelectBackgroundTilemap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_4_SetupExpandingSquareTransition as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_5_ExpandingSquare as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_6_AreaSelectMap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_7_PrepExpandSquareTransToRoomMap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_8_ExpandSquareTransToRoomSelectMap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_9_InitRoomSelectMap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_10_RoomSelectMap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_11 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_11 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_13 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_14 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_15_ClearTileMap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_16_LoadPalettes as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_2_LoadAreaSelectForegroundTilemap as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_18 as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_4_SetupExpandingSquareTransition as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_20_SetupExpandingSquare as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_21_MoveExpandingSquare as unsafe fn(&mut Game) -> ()),
        Some(FileSelectMap_22 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn FileSelectMap(game: &mut Game) {
    kFileSelectMapFuncs[ram_word!(game, menu_index) as usize].expect("non-null function pointer")(
        game,
    );
}
pub unsafe fn FileSelectMap_22(game: &mut Game) {
    DrawAreaSelectMapLabels(game);
    HandleFadeOut(game);
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 0i32
    {
        set_ram_word!(game.game_state = kGameState_2_GameOptionsMenu as uint16);
        set_ram_word!(game.menu_index = 0 as uint16);
    }
}
pub unsafe fn FileSelectMenu_1_LoadFileSelectMenuBG2(game: &mut Game) {
    let mut v2: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    // This menu spans $3600..$3dff, beyond the pause-map union field.
    let menu_tilemap = (&raw mut game.g_ram as *mut uint8)
        .add(0x3600)
        .cast::<uint16>();
    let mut i: ::core::ffi::c_int = 1023 as ::core::ffi::c_int;
    while i >= 0i32 {
        *menu_tilemap.add(i as usize) = *(game.g_rom.offset(crate::rtl::lorom_offset(0x8edc00))
            as *const uint8 as *mut uint16)
            .offset(i as isize);
        i -= 1;
    }
    let v1: uint16 = ram_word!(game, vram_write_queue_tail);
    v2 = gVramWriteEntry(
        ram_word!(game, vram_write_queue_tail) as ::core::ffi::c_int,
        game,
    );
    (*v2).size = 2048 as uint16;
    (*v2).src.addr =
        (&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .menu
            .menu_tilemap as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut uint16) = 126 as uint16;
    (*v2).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    set_ram_word!(game.vram_write_queue_tail = (v1).wrapping_add(7u16));
    let ref mut fresh81 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh81 = (*fresh81).wrapping_add(1);
    set_ram_word!(game.cinematic_var5 = 1 as uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(0) = 0 as uint16;
}
pub unsafe fn FileSelectMenu_2_InitMain(game: &mut Game) {
    let mut v0: uint16 = 0;
    if *(game.g_sram.offset(0x1fec) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        || *(game.g_sram.offset(0x1fec) as *mut uint16) as ::core::ffi::c_int
            - 3 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        || {
            v0 = *(game.g_sram.offset(0x1fec) as *mut uint16);
            (*(game.g_sram.offset(0x1fee) as *mut uint16) as ::core::ffi::c_int
                & *(game.g_sram.offset(0x1fec) as *mut uint16) as ::core::ffi::c_int)
                != 0i32
        }
    {
        v0 = 0 as uint16;
    }
    set_ram_word!(game.selected_save_slot = v0);
    FileSelectMenu_16(game);
}
pub unsafe fn FileSelectMenu_16(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 2046 as ::core::ffi::c_int;
    while i >= 0i32 {
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768) as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 15 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    set_ram_word!(game.nonempty_save_slots = -(1 as ::core::ffi::c_int) as uint16);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(0x56 as uint16, addr_kMenuTilemap_SamusData as uint16, game);
    LoadMenuTilemap(0x146 as uint16, addr_kMenuTilemap_SamusA as uint16, game);
    let mut c: uint8 = LoadFromSram_(0 as uint16, game);
    set_ram_word!(
        game.nonempty_save_slots =
            (ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int
                | (c as ::core::ffi::c_int) << 15 as ::core::ffi::c_int) as uint16
    );
    DrawFileSelectionHealth(
        (ram_word!(game, nonempty_save_slots)) & (0x8000u16),
        0x15c as uint16,
        game,
    );
    DrawFileSelectionTime(
        (ram_word!(game, nonempty_save_slots)) & (0x8000u16),
        0x1b4 as uint16,
        game,
    );
    LoadMenuTilemap(0x176 as uint16, addr_kMenuTilemap_TIME as uint16, game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(0x286 as uint16, addr_kMenuTilemap_SamusB as uint16, game);
    c = LoadFromSram_(1 as uint16, game);
    set_ram_word!(
        game.nonempty_save_slots =
            (ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int
                | (c as ::core::ffi::c_int) << 15 as ::core::ffi::c_int) as uint16
    );
    DrawFileSelectionHealth(
        (ram_word!(game, nonempty_save_slots)) & (0x8000u16),
        0x29c as uint16,
        game,
    );
    DrawFileSelectionTime(
        (ram_word!(game, nonempty_save_slots)) & (0x8000u16),
        0x2f4 as uint16,
        game,
    );
    LoadMenuTilemap(0x2b6 as uint16, addr_kMenuTilemap_TIME as uint16, game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(0x3c6 as uint16, addr_kMenuTilemap_SamusC as uint16, game);
    c = LoadFromSram_(2 as uint16, game);
    set_ram_word!(
        game.nonempty_save_slots =
            (ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int
                | (c as ::core::ffi::c_int) << 15 as ::core::ffi::c_int) as uint16
    );
    DrawFileSelectionHealth(
        (ram_word!(game, nonempty_save_slots)) & (0x8000u16),
        0x3dc as uint16,
        game,
    );
    DrawFileSelectionTime(
        (ram_word!(game, nonempty_save_slots)) & (0x8000u16),
        0x434 as uint16,
        game,
    );
    LoadMenuTilemap(0x3f6 as uint16, addr_kMenuTilemap_TIME as uint16, game);
    set_ram_word!(
        game.nonempty_save_slots = (u16::swap_bytes(
            !(ram_word!(game, nonempty_save_slots) as ::core::ffi::c_int) as uint16
        ) as ::core::ffi::c_int
            >> 5 as ::core::ffi::c_int) as uint16
    );
    if ram_word!(game, nonempty_save_slots) != 0 {
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .palette_index = 0 as uint16;
        LoadMenuTilemap(0x508 as uint16, addr_kMenuTilemap_DataCopy as uint16, game);
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .palette_index = 0 as uint16;
        LoadMenuTilemap(0x5c8 as uint16, addr_kMenuTilemap_DataClear as uint16, game);
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .palette_index = 0 as uint16;
    LoadMenuTilemap(0x688 as uint16, addr_kMenuTilemap_Exit as uint16, game);
    QueueTransferOfMenuTilemapToVramBG1(game);
    set_ram_word!(game.cinematic_var5 = 1 as uint16);
    set_ram_word!(game.cinematic_var6 = 0 as uint16);
    set_ram_word!(game.cinematic_var7 = 0 as uint16);
    set_ram_word!(game.cinematic_var8 = 0 as uint16);
    set_ram_word!(game.cinematic_var9 = 0 as uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(0) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(1) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(2) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(3) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(4) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(6) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(11) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(7) = 100 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(8) = 100 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(9) = 100 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(12) = 47 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(13) = 87 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(14) = 127 as uint16;
    set_ram_word!(game.screen_fade_delay = 1 as uint16);
    set_ram_word!(game.screen_fade_counter = 1 as uint16);
    ScreenOn(game);
    let ref mut fresh47 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh47 = (*fresh47).wrapping_add(1);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(15) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17) = 0 as uint16;
}
pub unsafe fn LoadFromSram_(a: uint16, game: &mut Game) -> uint8 {
    LoadFromSram(a, game)
}
pub unsafe fn FileSelectMenu_3_FadeInToMain(game: &mut Game) {
    DrawFileSelectSamusHelmets(game);
    let v0: ::core::ffi::c_int = ((4u16).wrapping_mul(ram_word!(game, selected_save_slot)))
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) =
        kMenuSelectionMissileXY[v0 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) =
        kMenuSelectionMissileXY[(v0 + 1 as ::core::ffi::c_int) as usize];
    DrawMenuSelectionMissile(game);
    DrawBorderAroundSamusData(game);
    HandleFadeIn(game);
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 15i32
    {
        let ref mut fresh80 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh80 = (*fresh80).wrapping_add(1);
    }
}
pub unsafe fn DrawFileSelectionHealth(a: uint16, k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v4: int16 = 0;
    if a != 0 {
        FileSelectClearRestOfMenuTilemapRow(k, game);
        LoadMenuTilemap(
            (k).wrapping_add(64u16),
            addr_kMenuTilemap_NoData as uint16,
            game,
        );
    } else {
        LoadMenuTilemap(k, addr_kMenuTilemap_Energy as uint16, game);
        v2 = ((k) as i16).wrapping_add(8i16);
        let mut n: ::core::ffi::c_int =
            ram_word!(game, samus_health) as ::core::ffi::c_int / 100 as ::core::ffi::c_int;
        let mut q: ::core::ffi::c_int =
            ram_word!(game, samus_health) as ::core::ffi::c_int % 100 as ::core::ffi::c_int;
        let mut mh: ::core::ffi::c_int =
            ram_word!(game, samus_max_health) as ::core::ffi::c_int / 100 as ::core::ffi::c_int;
        let mut m: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
        let mut v3: uint16 = ((v2) as u16).wrapping_add(64u16);
        loop {
            mh -= 1;
            if (mh < 0i32) {
                break;
            }
            v4 = 153 as int16;
            if n != 0 {
                n -= 1;
                v4 = 152 as int16;
            }
            *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap as *mut uint16)
                .offset(768) as *mut uint16 as *mut uint8)
                .offset(v3 as ::core::ffi::c_int as isize) as *mut uint16) =
                ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
                    .offset(0))
                .palette_index)
                    | ((v4) as u16);
            v3 = (v3).wrapping_add(2u16);
            m -= 1;
            if m == 0 {
                v3 = (v3).wrapping_sub(78u16);
                m = 8 as ::core::ffi::c_int;
            }
        }
        n = q / 10 as ::core::ffi::c_int;
        q = q % 10 as ::core::ffi::c_int;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(802) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .palette_index as ::core::ffi::c_int
                | q + 8288 as ::core::ffi::c_int) as uint16;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(801) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .palette_index as ::core::ffi::c_int
                | n + 8288 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn DrawFileSelectionTime(a: uint16, k: uint16, game: &mut Game) {
    if a == 0 {
        let r26: uint16 = k;
        let div_val: ::core::ffi::c_int =
            ram_word!(game, game_time_hours) as ::core::ffi::c_int / 10 as ::core::ffi::c_int;
        let mod_val: ::core::ffi::c_int =
            ram_word!(game, game_time_hours) as ::core::ffi::c_int % 10 as ::core::ffi::c_int;
        let v2: uint16 = r26;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(769) as *mut uint16 as *mut uint8)
            .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .palette_index as ::core::ffi::c_int
                | mod_val + 8288 as ::core::ffi::c_int) as uint16;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768) as *mut uint16 as *mut uint8)
            .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .palette_index as ::core::ffi::c_int
                | div_val + 8288 as ::core::ffi::c_int) as uint16;
        LoadMenuTilemap((r26).wrapping_add(4u16), addr_word_81B4A8 as uint16, game);
        let div_min: ::core::ffi::c_int =
            ram_word!(game, game_time_minutes) as ::core::ffi::c_int / 10 as ::core::ffi::c_int;
        let mod_min: ::core::ffi::c_int =
            ram_word!(game, game_time_minutes) as ::core::ffi::c_int % 10 as ::core::ffi::c_int;
        let v3: uint16 = r26;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(772) as *mut uint16 as *mut uint8)
            .offset(v3 as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .palette_index as ::core::ffi::c_int
                | mod_min + 8288 as ::core::ffi::c_int) as uint16;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(771) as *mut uint16 as *mut uint8)
            .offset(v3 as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .palette_index as ::core::ffi::c_int
                | div_min + 8288 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn FileSelectMenu_4_Main(game: &mut Game) {
    let current_block: u64;
    let mut v0: int16 = 0;
    let mut v1: uint16 = 0;
    let mut v2: uint16 = 0;
    let mut v3: ::core::ffi::c_int = 0;
    DrawBorderAroundSamusData(game);
    DrawMenuSelectionMissile(game);
    DrawFileSelectSlotSamusHelmet(4 as uint16, game);
    DrawFileSelectSlotSamusHelmet(6 as uint16, game);
    DrawFileSelectSlotSamusHelmet(8 as uint16, game);
    if ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        == 0i32
    {
        if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int)
            != 0i32
        {
            if ram_word!(game, nonempty_save_slots) != 0 {
                v1 = (ram_word!(game, selected_save_slot)).wrapping_sub(1u16);
                if ((ram_word!(game, selected_save_slot)) as i16).wrapping_sub(1i16) < 0i16 {
                    v1 = 5 as uint16;
                }
            } else {
                v1 = (ram_word!(game, selected_save_slot)).wrapping_sub(1u16);
                if ((ram_word!(game, selected_save_slot)) as i16).wrapping_sub(1i16) >= 0i16 {
                    if ram_word!(game, selected_save_slot) as ::core::ffi::c_int
                        - 5 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0
                    {
                        v1 = 2 as uint16;
                    }
                } else {
                    v1 = 5 as uint16;
                }
            }
            set_ram_word!(game.selected_save_slot = v1);
            current_block = 14136749492126903395;
        } else if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int)
            == 0i32
        {
            if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
                & kButton_B as ::core::ffi::c_int)
                != 0i32
            {
                QueueSfx1_Max6(0x37 as uint16, game);
                set_ram_word!(game.menu_index = 33 as uint16);
                QueueSfx1_Max6(0x37 as uint16, game);
            }
            current_block = 8908889263641380247;
        } else {
            if ram_word!(game, nonempty_save_slots) != 0 {
                v2 = (ram_word!(game, selected_save_slot)).wrapping_add(1u16);
                if ram_word!(game, selected_save_slot) as ::core::ffi::c_int
                    - 5 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    v2 = 0 as uint16;
                }
            } else {
                v2 = (ram_word!(game, selected_save_slot)).wrapping_add(1u16);
                if ram_word!(game, selected_save_slot) as ::core::ffi::c_int
                    - 2 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    if ram_word!(game, selected_save_slot) as ::core::ffi::c_int
                        - 5 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0
                    {
                        v2 = 5 as uint16;
                    } else {
                        v2 = 0 as uint16;
                    }
                }
            }
            set_ram_word!(game.selected_save_slot = v2);
            current_block = 14136749492126903395;
        }
        match current_block {
            8908889263641380247 => {}
            _ => {
                QueueSfx1_Max6(0x37 as uint16, game);
            }
        }
    } else {
        v0 = ram_word!(game, selected_save_slot) as int16;
        if ram_word!(game, selected_save_slot) as ::core::ffi::c_int - 3 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            QueueSfx1_Max6(0x2a as uint16, game);
            let ref mut fresh77 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
            *fresh77 = (*fresh77).wrapping_add(27u16);
            *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
                .offset(
                    (2 as ::core::ffi::c_int
                        * (ram_word!(game, selected_save_slot) as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int)) as uint16
                        as ::core::ffi::c_int as isize,
                ) as *mut uint16) = 1 as uint16;
            *(game.g_sram.offset(0x1fec) as *mut uint8 as *mut uint16) =
                ram_word!(game, selected_save_slot);
            *(game.g_sram.offset(0x1fee) as *mut uint8 as *mut uint16) =
                !(ram_word!(game, selected_save_slot) as ::core::ffi::c_int) as uint16;
            RtlWriteSram(game);
            if LoadFromSram(ram_word!(game, selected_save_slot), game) != 0 {
                NewSaveFile(game);
                set_ram_word!(game.has_area_map = 0 as uint16);
            } else {
                LoadMirrorOfExploredMapTiles(game);
            }
        } else {
            if ram_word!(game, selected_save_slot) == 3u16 {
                QueueSfx1_Max6(0x37 as uint16, game);
                let ref mut fresh78 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
                *fresh78 = (*fresh78).wrapping_add(1);
                *((&raw mut game.g_ram as *mut uint8).offset(0x57) as *mut uint16) =
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x57) as *mut uint16)
                        as ::core::ffi::c_int
                        & 0xff0c as ::core::ffi::c_int
                        | 3 as ::core::ffi::c_int) as uint16;
            } else {
                if ram_word!(game, selected_save_slot) == 4u16 {
                    QueueSfx1_Max6(0x37 as uint16, game);
                    let ref mut fresh79 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
                    *fresh79 = (*fresh79).wrapping_add(15u16);
                    v0 = (*((&raw mut game.g_ram as *mut uint8).offset(0x57) as *mut uint16)
                        as ::core::ffi::c_int
                        & 0xff0c as ::core::ffi::c_int
                        | 3 as ::core::ffi::c_int) as int16;
                    *((&raw mut game.g_ram as *mut uint8).offset(0x57) as *mut uint16) =
                        v0 as uint16;
                }
                if v0 == 5i16 {
                    set_ram_word!(game.menu_index = 33 as uint16);
                }
            }
            return;
        }
    }
    v3 = ((4u16).wrapping_mul(ram_word!(game, selected_save_slot))) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10) =
        kMenuSelectionMissileXY[v3 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5) =
        kMenuSelectionMissileXY[(v3 + 1 as ::core::ffi::c_int) as usize];
}
pub unsafe fn FileSelectMap_0(game: &mut Game) {
    let mut v1: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    ClearMenuTilemap(game);
    let v0: uint16 = ram_word!(game, vram_write_queue_tail);
    v1 = gVramWriteEntry(
        ram_word!(game, vram_write_queue_tail) as ::core::ffi::c_int,
        game,
    );
    (*v1).size = 2048 as uint16;
    (*v1).src.addr =
        (&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .menu
            .menu_tilemap as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    *(&raw mut (*v1).src.bank as *mut uint16) = 126 as uint16;
    (*v1).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(7u16));
    set_ram_word!(game.palette_change_num = 0 as uint16);
    LoadFileSelectPalettes(game);
    let mut v2: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v2 = (v2).wrapping_add(2u16);
        if (((v2) as i16).wrapping_sub(64i16) >= 0i16) {
            break;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(14) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(30) = 0 as uint16;
    let ref mut fresh43 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh43 = (*fresh43).wrapping_add(1);
}
pub unsafe fn FileSelectMap_1(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v2: int16 = 0;
    if AdvanceGradualColorChangeOfPalette(0 as uint16, 0x40 as uint16, game) != 0 {
        set_ram_word!(game.reg_BG1VOFS = 0 as uint16);
        set_ram_word!(game.reg_BG1HOFS = 0 as uint16);
        set_ram_word!(game.reg_BG2VOFS = 0 as uint16);
        set_ram_word!(game.reg_BG2HOFS = 0 as uint16);
        let mut v0: uint16 = 0 as uint16;
        while ram_word!(game, area_index)
            != kFileSelectMap_AreaIndexes
                [(v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        {
            v0 = (v0).wrapping_add(2u16);
            if (((v0) as i16).wrapping_sub(12i16) < 0i16) {
                continue;
            }
            v0 = 0 as uint16;
            break;
        }
        set_ram_word!(
            game.file_select_map_area_index =
                (v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16
        );
        v1 = 0 as int16;
        loop {
            v2 = v1;
            if v1 as ::core::ffi::c_int == ram_word!(game, area_index) as ::core::ffi::c_int {
                LoadActiveAreaMapForegroundColors(v1 as uint16, game);
            } else {
                LoadInactiveAreaMapForegroundColors(v1 as uint16, game);
            }
            v1 += 1;
            if ((v2).wrapping_sub(5i16) >= 0i16) {
                break;
            }
        }
        let ref mut fresh42 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh42 = (*fresh42).wrapping_add(1);
        set_ram_word!(game.g_word_7E0787 = 0 as uint16);
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 2 as uint8;
    }
}
pub unsafe fn sub_81A3D1(k: uint16, game: &mut Game) {
    LoadActiveAreaMapForegroundColors(k, game);
}
static kActiveAreaMapForegroundColors: [uint16; 6] = [0, 0xa, 0x10, 0x16, 0x24, 0x2a];
static kInactiveAreaMapForegroundColors: [uint16; 6] = [0x30, 0x3a, 0x40, 0x46, 0x54, 0x5a];
pub unsafe fn LoadActiveAreaMapForegroundColors(v0: uint16, game: &mut Game) {
    LoadAreaMapForegroundColors(kActiveAreaMapForegroundColors[v0 as usize], game);
}
pub unsafe fn LoadInactiveAreaMapForegroundColors(v0: uint16, game: &mut Game) {
    LoadAreaMapForegroundColors(kInactiveAreaMapForegroundColors[v0 as usize], game);
}
pub unsafe fn LoadAreaMapForegroundColors(mut j: uint16, game: &mut Game) {
    loop {
        let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *(game.g_rom.offset(crate::rtl::lorom_offset(0x81a4e6)) as *const uint8 as *mut uint16)
            .offset(v1 as isize)
            == 0xffffu16
        {
            break;
        }
        let v4: uint16 = j;
        let mut v2: uint16 =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x81a4e6)) as *const uint8 as *mut uint16)
                .offset((v1 + 1 as ::core::ffi::c_int) as isize);
        let mut v3: uint16 =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x81a4e6)) as *const uint8 as *mut uint16)
                .offset(v1 as isize);
        let mut n: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                *(game.g_rom.offset(crate::rtl::lorom_offset(0x81a40e)) as *const uint8
                    as *mut uint16)
                    .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v2 = (v2).wrapping_add(2u16);
            v3 = (v3).wrapping_add(2u16);
            n -= 1;
            if (n == 0) {
                break;
            }
        }
        j = (v4).wrapping_add(4u16);
    }
}
pub unsafe fn FileSelectMap_2_LoadAreaSelectForegroundTilemap(game: &mut Game) {
    let mut v1: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let v0: uint16 = ram_word!(game, vram_write_queue_tail);
    v1 = gVramWriteEntry(
        ram_word!(game, vram_write_queue_tail) as ::core::ffi::c_int,
        game,
    );
    (*v1).size = 2048 as uint16;
    (*v1).src.addr = addr_kAreaSelectForegroundTilemap as ::core::ffi::c_int as VoidP;
    *(&raw mut (*v1).src.bank as *mut uint16) = 129 as uint16;
    (*v1).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x58) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(7u16));
    *((&raw mut game.g_ram as *mut uint8).offset(0x51) as *mut uint16) =
        ((*(&raw mut game.g_ram as *mut uint8).offset(0x52) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int
            | 0xf as ::core::ffi::c_int) as uint16;
    let ref mut fresh18 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh18 = (*fresh18).wrapping_add(1);
}
pub unsafe fn FileSelectMap_18(game: &mut Game) {
    let ref mut fresh17 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh17 = (*fresh17).wrapping_add(1);
    LoadAreaSelectBackgroundTilemap(ram_word!(game, area_index), game);
}
pub unsafe fn FileSelectMap_3_LoadAreaSelectBackgroundTilemap(game: &mut Game) {
    let ref mut fresh41 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh41 = (*fresh41).wrapping_add(1);
    LoadAreaSelectBackgroundTilemap(ram_word!(game, area_index), game);
}
pub unsafe fn LoadAreaSelectBackgroundTilemap(j: uint16, game: &mut Game) {
    let v1: uint16 = ram_word!(game, vram_write_queue_tail);
    let v2: *mut VramWriteEntry = gVramWriteEntry(
        ram_word!(game, vram_write_queue_tail) as ::core::ffi::c_int,
        game,
    );
    (*v2).size = 2048 as uint16;
    (*v2).src.addr = (8 as ::core::ffi::c_int * 256 as ::core::ffi::c_int * j as ::core::ffi::c_int
        + addr_kAreaSelectBackgroundTilemaps as ::core::ffi::c_int) as VoidP;
    (*v2).src.bank = 0x81 as uint8;
    (*v2).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x5a) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    set_ram_word!(game.vram_write_queue_tail = (v1).wrapping_add(7u16));
}
pub unsafe fn FileSelectMap_4_SetupExpandingSquareTransition(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 4 as uint8;
    WriteReg(TS as uint16, 4 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 4 as uint8;
    WriteReg(TSW as uint16, 4 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 2 as uint8;
    WriteReg(CGWSEL as uint16, 2 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 37 as uint8;
    WriteReg(CGADSUB as uint16, 0x25 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    SetupInitialExpandingSquareHDMA(game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x85) = 12 as uint8;
    WriteReg(HDMAEN as uint16, 0xc as uint8, game);
    ConfigureWindow1ForExpandingSquare(game);
    set_ram_word!(game.hdma_objects_enable_flag = 0 as uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16 as *mut uint8).offset(0) =
        ((ram_word!(game, menu_index)) as u8).wrapping_add(1u8);
}
pub unsafe fn ConfigureWindow1ForExpandingSquare(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 19 as uint8;
    WriteReg(TM as uint16, 0x13 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 19 as uint8;
    WriteReg(TMW as uint16, 0x13 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 35 as uint8;
    WriteReg(W12SEL as uint16, 0x23 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 3 as uint8;
    WriteReg(W34SEL as uint16, 3 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 35 as uint8;
    WriteReg(WOBJSEL as uint16, 0x23 as uint8, game);
}
pub unsafe fn SetupInitialExpandingSquareHDMA(game: &mut Game) {
    set_ram_word!(game.expand_sq_topbottom_margin_right_pos = 0 as uint16);
    set_ram_word!(game.expand_sq_topbottom_margin_left_pos = -(1 as ::core::ffi::c_int) as uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e32) as *mut uint16 as *mut uint8).offset(0) =
        127 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e36) as *mut uint16 as *mut uint8).offset(0) =
        -(127 as ::core::ffi::c_int) as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0 = 111 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_1 = 34 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(1))
    .field_0 = 1 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(1))
    .field_1 = 50 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(1))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(2))
    .field_0 = 1 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(2))
    .field_1 = 50 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(2))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_0 = 111 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_1 = 32 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    WriteReg(DMAP2 as uint16, 0x40 as uint8, game);
    WriteReg(BBAD2 as uint16, 0x26 as uint8, game);
    WriteReg(A1T2L as uint16, 0 as uint8, game);
    WriteReg(A1T2H as uint16, 0x9e as uint8, game);
    WriteReg(A1B2 as uint16, 0x7e as uint8, game);
    WriteReg(DAS20 as uint16, 0x7e as uint8, game);
    WriteReg(DAS2L as uint16, 0 as uint8, game);
    WriteReg(DAS2H as uint16, 0 as uint8, game);
    WriteReg(A2A2L as uint16, 0 as uint8, game);
    WriteReg(A2A2H as uint16, 0 as uint8, game);
    WriteReg(NTRL2 as uint16, 0 as uint8, game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0 = 111 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_1 = 32 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(1))
    .field_0 = 1 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(1))
    .field_1 = 54 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(1))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(2))
    .field_0 = 1 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(2))
    .field_1 = 54 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(2))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_0 = 111 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_1 = 32 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_2 = -(98 as ::core::ffi::c_int) as uint8;
    WriteReg(DMAP3 as uint16, 0x40 as uint8, game);
    WriteReg(BBAD3 as uint16, 0x27 as uint8, game);
    WriteReg(A1T3L as uint16, 0x10 as uint8, game);
    WriteReg(A1T3H as uint16, 0x9e as uint8, game);
    WriteReg(A1B3 as uint16, 0x7e as uint8, game);
    WriteReg(DAS30 as uint16, 0x7e as uint8, game);
    WriteReg(DAS3L as uint16, 0 as uint8, game);
    WriteReg(DAS3H as uint16, 0 as uint8, game);
    WriteReg(A2A3L as uint16, 0 as uint8, game);
    WriteReg(A2A3H as uint16, 0 as uint8, game);
    WriteReg(NTRL3 as uint16, 0 as uint8, game);
}
static kExpandingSquareTransitionSpeed: uint8 = 4 as uint8;
pub unsafe fn FileSelectMap_5_ExpandingSquare(game: &mut Game) {
    let mut v5: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    DrawAreaSelectMapLabels(game);
    let v0: uint8 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0)
        .wrapping_sub(kExpandingSquareTransitionSpeed);
    if (((*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0) as i8)
        .wrapping_sub((kExpandingSquareTransitionSpeed) as i8)
        < 0i8
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16 as *mut uint8)
            .offset(0) = ((ram_word!(game, menu_index)) as u8).wrapping_add(1u8);
        let ref mut fresh36 = *(&raw mut game.g_ram as *mut uint8).offset(0x69);
        *fresh36 = (*fresh36 as ::core::ffi::c_int & !(2 as ::core::ffi::c_int)) as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
        set_ram_word!(game.reg_BG2VOFS = 24 as uint16);
        set_ram_word!(game.pausemenu_palette_animation_timer = 1 as uint16);
        let mut v1: uint16 = 0 as uint16;
        loop {
            (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
                [v1 as usize] = *(game.g_rom.offset(crate::rtl::lorom_offset(0xb6e000))
                as *const uint8 as *mut uint16)
                .offset(v1 as isize);
            v1 = v1.wrapping_add(1);
            if (((v1 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 1600 as ::core::ffi::c_int)
                as int16)
                >= 0i16)
            {
                break;
            }
        }
        loop {
            let fresh37 = v1;
            v1 = v1.wrapping_add(1);
            (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
                [fresh37 as usize] = 10241 as uint16;
            if (((v1 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 2048 as ::core::ffi::c_int)
                as int16)
                >= 0i16)
            {
                break;
            }
        }
        DrawRoomSelectMapAreaLabel(
            (&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
                .bg2_room_select_map_tilemap as *mut uint16)
                .offset(170),
            game,
        );
        let mut v2: uint16 = 320 as uint16;
        let mut v3: uint16 = 959 as uint16;
        loop {
            let fresh38 = v3;
            v3 = v3.wrapping_sub(1);
            (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
                [fresh38 as usize] = *(game.g_rom.offset(crate::rtl::lorom_offset(0x81b14b))
                as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v2 = (v2).wrapping_sub(2u16);
            if (v2 == 0) {
                break;
            }
        }
        let v4: uint16 = ram_word!(game, vram_write_queue_tail);
        v5 = gVramWriteEntry(
            ram_word!(game, vram_write_queue_tail) as ::core::ffi::c_int,
            game,
        );
        (*v5).size = 2048 as uint16;
        (*v5).src.addr = ((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000
            as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8)
            as ::core::ffi::c_long as VoidP;
        (*v5).src.bank = 126 as uint8;
        (*v5).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16;
        set_ram_word!(game.vram_write_queue_tail = (v4).wrapping_add(7u16));
    } else {
        let ref mut fresh39 = (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0;
        *fresh39 = (*fresh39).wrapping_sub(kExpandingSquareTransitionSpeed);
        (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(3))
        .field_0 = v0;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0 = v0;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(3))
        .field_0 = v0;
        let ref mut fresh40 = (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_0;
        *fresh40 = (*fresh40).wrapping_add(kExpandingSquareTransitionSpeed);
        (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(2))
        .field_0 = (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_0;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_0 = (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_0;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(2))
        .field_0 = (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_0;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9e32) as *mut uint16 as *mut uint8)
            .offset(0) = ((ram_word!(game, expand_sq_left_pos)) as u8)
            .wrapping_sub(kExpandingSquareTransitionSpeed);
        *((&raw mut game.g_ram as *mut uint8).offset(0x9e36) as *mut uint16 as *mut uint8)
            .offset(0) = (kExpandingSquareTransitionSpeed)
            .wrapping_add((ram_word!(game, expand_sq_right_pos)) as u8);
    };
}
pub unsafe fn FileSelectMap_6_AreaSelectMap(game: &mut Game) {
    let current_block: u64;
    let mut v0: int16 = 0;
    v0 = ram_word!(game, joypad1_newkeys) as int16;
    if ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & (kButton_Up as ::core::ffi::c_int | kButton_Left as ::core::ffi::c_int)
        != 0i32
    {
        v0 = ram_word!(game, enable_debug) as int16;
        if ram_word!(game, enable_debug) != 0 {
            let mut R28: uint16 = DecAndWraparoundTo5(ram_word!(game, file_select_map_area_index));
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
                QueueSfx1_Max6(0x37 as uint16, game);
                SwitchActiveFileSelectMapArea(R28, game);
                DrawAreaSelectMapLabels(game);
                return;
            }
            current_block = 4115184416633590978;
        } else {
            current_block = 9480232856434185415;
        }
    } else if ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & (kButton_Select as ::core::ffi::c_int
            | kButton_Down as ::core::ffi::c_int
            | kButton_Right as ::core::ffi::c_int)
        == 0i32
    {
        current_block = 9480232856434185415;
    } else {
        v0 = ram_word!(game, enable_debug) as int16;
        if ram_word!(game, enable_debug) == 0 {
            current_block = 9480232856434185415;
        } else {
            let mut n: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
            let mut R28_0: uint16 = ram_word!(game, file_select_map_area_index);
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
            match current_block {
                4115184416633590978 => {}
                _ => {
                    SwitchActiveFileSelectMapArea(R28_0, game);
                    QueueSfx1_Max6(0x37 as uint16, game);
                    DrawAreaSelectMapLabels(game);
                    return;
                }
            }
        }
    }
    if current_block == 9480232856434185415 {
        if v0 < 0i16 {
            set_ram_word!(game.menu_index = 22 as uint16);
            DrawAreaSelectMapLabels(game);
            return;
        }
        if v0 as ::core::ffi::c_int
            & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
            != 0i32
        {
            QueueSfx1_Max6(0x38 as uint16, game);
            SelectFileSelectMapArea(game);
            return;
        }
    }
    DrawAreaSelectMapLabels(game);
}
pub unsafe fn DecAndWraparoundTo5(a: uint16) -> uint16 {
    let result: uint16 = (a).wrapping_sub(1u16);
    if (result as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        return 5 as uint16;
    }
    result
}
pub unsafe fn WraparoundFrom6to0(a: uint16) -> uint16 {
    let result: uint16 = (a).wrapping_add(1u16);
    if result as ::core::ffi::c_int - 6 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        return 0 as uint16;
    }
    result
}
pub unsafe fn SelectFileSelectMapArea(game: &mut Game) {
    let mut v2: int8 = 0;
    let mut v3: int16 = 0;
    let mut v4: int16 = 0;
    let ref mut fresh35 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh35 = (*fresh35).wrapping_add(1);
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
    let mut q: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xd8f8).offset(
        (2 as ::core::ffi::c_int * ram_word!(game, area_index) as ::core::ffi::c_int) as isize,
    ) as *mut uint8 as *mut uint16);
    let mut v0: uint16 = 0 as uint16;
    let r0: *const uint16 = RomPtr_82(
        *(RomPtr_82(
            (addr_kMapIconDataPointers as ::core::ffi::c_int
                + 64 as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int * ram_word!(game, area_index) as ::core::ffi::c_int)
                as uint16_t,
            game,
        ) as *mut uint16),
        game,
    ) as *const uint16;
    let mut r20: uint16 = 16 as uint16;
    's_43: loop {
        v2 = ((q) as i8) & (1i8);
        q = (q as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        if (v2 != 0) {
            v3 = *r0.offset((2 as ::core::ffi::c_int * v0 as ::core::ffi::c_int) as isize) as int16;
            if (v3 != -2i16) {
                if v3 != -1i16 {
                    break;
                }
                v0 = -(1 as ::core::ffi::c_int) as uint16;
            }
        }
        v0 = v0.wrapping_add(1);
        r20 = r20.wrapping_sub(1);
        if (r20 != 0) {
            continue;
        }
        loop {
            v4 = *r0.offset((2 as ::core::ffi::c_int * v0 as ::core::ffi::c_int) as isize) as int16;
            if v4 != -2i16 {
                if v4 != -1i16 {
                    break 's_43;
                }
                v0 = -(1 as ::core::ffi::c_int) as uint16;
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
pub unsafe fn CheckIfFileSelectMapAreaCanBeSelected(a: uint16, game: &mut Game) -> uint16 {
    let v1: ::core::ffi::c_int =
        2 as ::core::ffi::c_int * kFileSelectMap_AreaIndexes[a as usize] as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8)
        .offset(0xd8f8)
        .offset(v1 as isize) as *mut uint8 as *mut uint16)
        != 0
    {
        return true_0 as uint16;
    }
    let t: uint16 = (*((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c7cb))
        as *const uint8 as *mut MapIconDataPointers)
        .offset(4))
    .crateria as *mut uint8)
        .offset(v1 as isize) as *mut uint16))
        .wrapping_add(64u16);
    (t != 0xffffu16) as uint16
}
pub unsafe fn SwitchActiveFileSelectMapArea(R28: uint16, game: &mut Game) {
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
static kAreaSelectMapLabelPositions: [uint16; 12] = [
    0x5b, 0x32, 0x2a, 0x7f, 0x5e, 0xb5, 0xce, 0x50, 0xce, 0x9f, 0x87, 0x8b,
];
pub unsafe fn DrawAreaSelectMapLabels(game: &mut Game) {
    let mut r3: uint16 = 0 as uint16;
    DrawMenuSpritemap(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c749)) as *const uint8 as *mut uint16)
            .offset(0),
        0x80 as uint16,
        0x10 as uint16,
        r3,
        game,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 6i32 {
        r3 = (if i == ram_word!(game, file_select_map_area_index) as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            512 as ::core::ffi::c_int
        }) as uint16;
        let v1: uint16 = (2u16).wrapping_mul(kFileSelectMap_AreaIndexes[i as usize]);
        let mut r36: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd8f8)
            .offset(v1 as isize) as *mut uint8 as *mut uint16);
        let mut v2: *const uint16 = RomPtr_82(
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c7cb)) as *const uint8
                as *mut MapIconDataPointers)
                .offset(4))
            .crateria as *mut uint8)
                .offset(v1 as ::core::ffi::c_int as isize) as *mut VoidP),
            game,
        ) as *const uint16;
        let mut R30: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
        while *v2 != 0xffffu16 {
            let v4: ::core::ffi::c_int = r36 as ::core::ffi::c_int & 1 as ::core::ffi::c_int;
            r36 = (r36 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
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
            let j: ::core::ffi::c_int = 4 as ::core::ffi::c_int
                * kFileSelectMap_AreaIndexes[i as usize] as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            DrawMenuSpritemap(
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c749)) as *const uint8
                    as *mut uint16)
                    .offset(0) as ::core::ffi::c_int
                    + kFileSelectMap_AreaIndexes[i as usize] as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as uint16,
                kAreaSelectMapLabelPositions[j as usize],
                kAreaSelectMapLabelPositions[(j + 1 as ::core::ffi::c_int) as usize],
                r3,
                game,
            );
            break;
        }
        i += 1;
    }
}
static kRoomSelectMapExpandingSquareTimers: [uint16; 6] = [0x33, 0x35, 0x2d, 0x33, 0x33, 0x22];
pub unsafe fn FileSelectMap_7_PrepExpandSquareTransToRoomMap(game: &mut Game) {
    DrawAreaSelectMapLabels(game);
    SetupInitialExpandingSquareHDMA(game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 50 as uint8;
    WriteReg(W12SEL as uint16, 0x32 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 2 as uint8;
    WriteReg(W34SEL as uint16, 2 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 5 as uint8;
    WriteReg(CGADSUB as uint16, 5 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 34 as uint8;
    WriteReg(WOBJSEL as uint16, 0x22 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 48 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_0 = 0 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_0 = 0 as uint8;
    DrawRoomSelectMapAreaLabel(
        (&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
            .bg2_room_select_map_tilemap as *mut uint16)
            .offset(170),
        game,
    );
    let v0: uint16 = ram_word!(game, vram_write_queue_tail);
    let v1: *mut VramWriteEntry = gVramWriteEntry(
        ram_word!(game, vram_write_queue_tail) as ::core::ffi::c_int,
        game,
    );
    (*v1).size = 512 as uint16;
    (*v1).src.addr =
        ((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    (*v1).src.bank = 126 as uint8;
    (*v1).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(7u16));
    set_ram_word!(
        game.expand_sq_timer =
            kRoomSelectMapExpandingSquareTimers[ram_word!(game, area_index) as usize]
    );
    let v2: ::core::ffi::c_int = ((4u16).wrapping_mul(ram_word!(game, area_index)))
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    set_ram_word!(game.expand_sq_left_pos = kAreaSelectMapLabelPositions[v2 as usize]);
    set_ram_word!(game.expand_sq_right_pos = ram_word!(game, expand_sq_left_pos));
    set_ram_word!(
        game.expand_sq_top_pos =
            kAreaSelectMapLabelPositions[(v2 + 1 as ::core::ffi::c_int) as usize]
    );
    set_ram_word!(game.expand_sq_bottom_pos = ram_word!(game, expand_sq_top_pos));
    set_ram_word!(game.expand_sq_left_subpos = 0 as uint16);
    set_ram_word!(game.expand_sq_right_subpos = 0 as uint16);
    set_ram_word!(game.expand_sq_top_subpos = 0 as uint16);
    set_ram_word!(game.expand_sq_bottom_subpos = 0 as uint16);
    let vels: *const ExpandingSquareVels = (game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34))
        as *const uint8 as *mut ExpandingSquareVels)
        .offset(ram_word!(game, area_index) as isize)
        as *mut ExpandingSquareVels;
    set_ram_word!(game.expand_sq_left_subvel = (*vels).left_subvel);
    set_ram_word!(game.expand_sq_left_vel = (*vels).left_vel);
    set_ram_word!(game.expand_sq_right_subvel = (*vels).right_subvel);
    set_ram_word!(game.expand_sq_right_vel = (*vels).right_vel);
    set_ram_word!(game.expand_sq_top_subvel = (*vels).top_subvel);
    set_ram_word!(game.expand_sq_top_vel = (*vels).top_vel);
    set_ram_word!(game.expand_sq_bottom_subvel = (*vels).bottom_subvel);
    set_ram_word!(game.expand_sq_bottom_vel = (*vels).bottom_vel);
    SetupRoomSelectMapExpandingSquareTransHDMA(game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x85) = 12 as uint8;
    WriteReg(HDMAEN as uint16, 0xc as uint8, game);
    let ref mut fresh34 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh34 = (*fresh34).wrapping_add(1);
    QueueSfx1_Max6(0x3b as uint16, game);
}
pub unsafe fn SetupRoomSelectMapExpandingSquareTransHDMA(game: &mut Game) {
    let mut k: uint16 = 0 as uint16;
    let v0: uint16 = ram_word!(game, expand_sq_top_pos);
    AddExpandingSqTransLeftIndirHDMA(v0, k, 0x9e22 as uint16, game);
    k = AddExpandingSqTransRightIndirHDMA(v0, k, 0x9e20 as uint16, game);
    let mut v2: uint16 =
        (ram_word!(game, expand_sq_bottom_pos)).wrapping_sub(ram_word!(game, expand_sq_top_pos));
    if (ram_word!(game, expand_sq_bottom_pos) as uint8)
        == (ram_word!(game, expand_sq_top_pos) as uint8)
    {
        v2 = 1 as uint16;
    }
    AddExpandingSqTransLeftIndirHDMA(v2, k, 0x9e32 as uint16, game);
    k = AddExpandingSqTransRightIndirHDMA(v2, k, 0x9e36 as uint16, game);
    let mut v4: uint16 = (-(32 as ::core::ffi::c_int)
        - ram_word!(game, expand_sq_bottom_pos) as ::core::ffi::c_int)
        as uint16;
    if (ram_word!(game, expand_sq_bottom_pos) as uint8) == 0xe0u8 {
        v4 = 1 as uint16;
    }
    AddExpandingSqTransLeftIndirHDMA(v4, k, 0x9e22 as uint16, game);
    k = AddExpandingSqTransRightIndirHDMA(v4, k, 0x9e20 as uint16, game);
    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0)
        .offset(k as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0)
        .offset(k as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn AddExpandingSqTransLeftIndirHDMA(a: uint16, k: uint16, j: uint16, game: &mut Game) {
    if (a as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) != 0i32 {
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0)
            .offset(k as ::core::ffi::c_int as isize) = ((a) as u8).wrapping_sub(127u8);
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_0)
            .offset(k as ::core::ffi::c_int as isize) = 127 as uint8;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
    } else {
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0)
            .offset(k as ::core::ffi::c_int as isize) = a as uint8;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
    };
}
pub unsafe fn AddExpandingSqTransRightIndirHDMA(
    a: uint16,
    k: uint16,
    j: uint16,
    game: &mut Game,
) -> uint16 {
    if (a as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) != 0i32 {
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0)
            .offset(k as ::core::ffi::c_int as isize) = ((a) as u8).wrapping_sub(127u8);
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_0)
            .offset(k as ::core::ffi::c_int as isize) = 127 as uint8;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(1))
        .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
        (k).wrapping_add(6u16)
    } else {
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0)
            .offset(k as ::core::ffi::c_int as isize) = a as uint8;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_1)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = j;
        (k).wrapping_add(3u16)
    }
}
pub unsafe fn FileSelectMap_8_ExpandSquareTransToRoomSelectMap(game: &mut Game) {
    if (HandleRoomSelectMapExpandingSquareTrans(game) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        let ref mut fresh33 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh33 = (*fresh33).wrapping_add(1);
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 2 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
    }
    DrawAreaSelectMapLabels(game);
}
pub unsafe fn HandleRoomSelectMapExpandingSquareTrans(game: &mut Game) -> uint16 {
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x9e32) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x9e30) as *mut uint16,
        (ram_word!(game, expand_sq_left_vel) as uint32) << 16 as ::core::ffi::c_int
            | ram_word!(game, expand_sq_left_subvel) as uint32,
    );
    if ram_word!(game, expand_sq_left_pos) as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        set_ram_word!(game.expand_sq_left_pos = 1 as uint16);
    }
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x9e36) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x9e34) as *mut uint16,
        (ram_word!(game, expand_sq_right_vel) as uint32) << 16 as ::core::ffi::c_int
            | ram_word!(game, expand_sq_right_subvel) as uint32,
    );
    if ram_word!(game, expand_sq_right_pos) as ::core::ffi::c_int - 256 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        set_ram_word!(game.expand_sq_right_pos = 255 as uint16);
    }
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x9e3a) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x9e38) as *mut uint16,
        (ram_word!(game, expand_sq_top_vel) as uint32) << 16 as ::core::ffi::c_int
            | ram_word!(game, expand_sq_top_subvel) as uint32,
    );
    if ram_word!(game, expand_sq_top_pos) as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        set_ram_word!(game.expand_sq_top_pos = 1 as uint16);
    }
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x9e3e) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x9e3c) as *mut uint16,
        (ram_word!(game, expand_sq_bottom_vel) as uint32) << 16 as ::core::ffi::c_int
            | ram_word!(game, expand_sq_bottom_subvel) as uint32,
    );
    if ram_word!(game, expand_sq_bottom_pos) as ::core::ffi::c_int - 224 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        set_ram_word!(game.expand_sq_bottom_pos = 224 as uint16);
    }
    SetupRoomSelectMapExpandingSquareTransHDMA(game);
    let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0x9e50) as *mut uint16);
    *fresh0 = (*fresh0).wrapping_sub(1);
    *fresh0
}
pub unsafe fn FileSelectMap_9_InitRoomSelectMap(game: &mut Game) {
    LoadMirrorOfExploredMapTiles(game);
    DrawRoomSelectMap(game);
    LoadFromLoadStation(game);
    DisableHdmaObjects(game);
    WaitUntilEndOfVblankAndClearHdma(game);
    let RoomDefHeader: *mut RoomDefHeader = get_RoomDefHeader(ram_word!(game, room_ptr), game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16 as *mut uint8).offset(0) =
        (*RoomDefHeader).area_index_;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a1) as *mut uint16 as *mut uint8).offset(0) =
        (*RoomDefHeader).x_coordinate_on_map;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a3) as *mut uint16 as *mut uint8).offset(0) =
        (*RoomDefHeader).y_coordinate_on_map;
    SetupMapScrollingForFileSelectMap(game);
    let ref mut fresh30 = *((&raw mut game.g_ram as *mut uint8).offset(0x5b0) as *mut uint16);
    *fresh30 = (*fresh30).wrapping_add(24u16);
    set_ram_word!(game.reg_BG2VOFS = 24 as uint16);
    let ref mut fresh31 = *((&raw mut game.g_ram as *mut uint8).offset(0x69) as *mut uint16);
    *fresh31 = (*fresh31 as ::core::ffi::c_int & !(4 as ::core::ffi::c_int)) as uint16;
    let ref mut fresh32 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh32 = (*fresh32).wrapping_add(1);
    set_ram_word!(game.map_scrolling_direction = 0 as uint16);
    set_ram_word!(game.map_scrolling_speed_index = 0 as uint16);
    set_ram_word!(game.samus_position_indicator_animation_frame = 0 as uint16);
    set_ram_word!(game.samus_position_indicator_animation_timer = 0 as uint16);
    set_ram_word!(game.samus_position_indicator_animation_loop_counter = 0 as uint16);
}
pub unsafe fn FileSelectMap_10_RoomSelectMap(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v3: int16 = 0;
    let mut v4: int8 = 0;
    let mut v5: int16 = 0;
    let mut v8: int16 = 0;
    DrawFileSelectMapIcons(game);
    HandleFileSelectMapScrollArrows(game);
    MapScrolling(game);
    DisplayMapElevatorDestinations(game);
    if ram_word!(game, enable_debug) != 0u16
        && (ram_word!(game, joypad2_new_keys) as ::core::ffi::c_int
            & kButton_Select as ::core::ffi::c_int)
            != 0i32
    {
        let mut current_block_32: u64;
        QueueSfx1_Max6(0x38 as uint16, game);
        let mut r24: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xd8f8).offset(
            (2 as ::core::ffi::c_int * ram_word!(game, area_index) as ::core::ffi::c_int) as isize,
        ) as *mut uint8 as *mut uint16);
        v1 = ram_word!(game, load_station_index) as int16;
        loop {
            r24 = (r24 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
            v1 -= 1;
            if (v1 < 0i16) {
                break;
            }
        }
        let r0: *const uint16 = RomPtr_82(
            *(RomPtr_82(
                (addr_kMapIconDataPointers as ::core::ffi::c_int
                    + 64 as ::core::ffi::c_int
                    + 2 as ::core::ffi::c_int * ram_word!(game, area_index) as ::core::ffi::c_int)
                    as uint16_t,
                game,
            ) as *mut uint16),
            game,
        ) as *const uint16;
        v3 = (4i16).wrapping_mul((ram_word!(game, load_station_index)) as i16);
        let r18: uint16 =
            *r0.offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        let r20: uint16 = *r0.offset(
            ((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                as isize,
        );
        if ram_word!(game, load_station_index) as ::core::ffi::c_int - 16 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            current_block_32 = 4150833010744877889;
        } else {
            current_block_32 = 12800627514080957624;
        }
        loop {
            match current_block_32 {
                12800627514080957624 => {
                    let ref mut fresh26 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16);
                    *fresh26 = (*fresh26).wrapping_add(1);
                    if *fresh26 as ::core::ffi::c_int - 16 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0
                    {
                        current_block_32 = 8236137900636309791;
                    } else {
                        current_block_32 = 5601891728916014340;
                    }
                }
                _ => {
                    let ref mut fresh27 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16);
                    *fresh27 = (*fresh27).wrapping_add(1);
                    current_block_32 = 8236137900636309791;
                }
            }
            if current_block_32 == 8236137900636309791 {
                v5 = *r0.offset(
                    (2 as ::core::ffi::c_int
                        * ram_word!(game, load_station_index) as ::core::ffi::c_int)
                        as isize,
                ) as int16;
                if v5 == -1i16 {
                    set_ram_word!(game.load_station_index = 0 as uint16);
                    r24 = *((&raw mut game.g_ram as *mut uint8).offset(0xd8f8).offset(
                        (2 as ::core::ffi::c_int
                            * ram_word!(game, area_index) as ::core::ffi::c_int)
                            as isize,
                    ) as *mut uint8 as *mut uint16);
                } else if v5 != -2i16 {
                    break;
                } else {
                    current_block_32 = 4150833010744877889;
                    continue;
                }
            }
            v4 = ((r24) as i8) & (1i8);
            r24 = (r24 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
            if v4 == 0
                || *r0.offset(
                    (2 as ::core::ffi::c_int
                        * ram_word!(game, load_station_index) as ::core::ffi::c_int)
                        as isize,
                ) >= 0xfffeu16
            {
                current_block_32 = 12800627514080957624;
            } else {
                break;
            }
        }
        let v6: uint16 = (4u16).wrapping_mul(ram_word!(game, load_station_index));
        let w7: uint16 = *r0.offset(
            (2 as ::core::ffi::c_int * ram_word!(game, load_station_index) as ::core::ffi::c_int)
                as isize,
        );
        if w7 as ::core::ffi::c_int - ram_word!(game, reg_BG1HOFS) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            || w7 as ::core::ffi::c_int
                - 256 as ::core::ffi::c_int
                - ram_word!(game, reg_BG1HOFS) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            v8 = (ram_word!(game, reg_BG1HOFS) as ::core::ffi::c_int
                + *r0.offset((v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                - r18 as ::core::ffi::c_int) as int16;
            if v8 >= 0i16 {
                if (v8).wrapping_sub((ram_word!(game, map_min_x_scroll)) as i16) >= 0i16 {
                    v8 = ram_word!(game, map_min_x_scroll) as int16;
                }
            } else {
                v8 = 0 as int16;
            }
            set_ram_word!(game.reg_BG1HOFS = v8 as uint16);
        }
        let v9: uint16 = (v6).wrapping_add(2u16);
        let v10: uint16 =
            *r0.offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        if v10 as ::core::ffi::c_int - ram_word!(game, reg_BG1VOFS) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            || v10 as ::core::ffi::c_int
                - 161 as ::core::ffi::c_int
                - ram_word!(game, reg_BG1VOFS) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            let mut v11: uint16 = (ram_word!(game, reg_BG1VOFS) as ::core::ffi::c_int
                + *r0.offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                - r20 as ::core::ffi::c_int) as uint16;
            if ((v11) as i16).wrapping_sub((ram_word!(game, map_min_y_scroll)) as i16) >= 0i16 {
                v11 = ram_word!(game, map_min_y_scroll);
            }
            set_ram_word!(game.reg_BG1VOFS = v11);
        }
    } else if (ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        let ref mut fresh28 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh28 = (*fresh28).wrapping_add(5u16);
        let mut v0: uint16 = 0 as uint16;
        while ram_word!(game, area_index)
            != kFileSelectMap_AreaIndexes
                [(v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        {
            v0 = (v0).wrapping_add(2u16);
            if ((v0) as i16).wrapping_sub(12i16) >= 0i16 {
                set_ram_word!(game.file_select_map_area_index = 0 as uint16);
                return;
            }
        }
        set_ram_word!(
            game.file_select_map_area_index =
                (v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16
        );
        QueueSfx1_Max6(0x3c as uint16, game);
    } else if ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        != 0i32
    {
        let ref mut fresh29 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh29 = (*fresh29).wrapping_add(1);
        QueueSfx1_Max6(0x38 as uint16, game);
    }
}
pub unsafe fn sub_81AEC8(game: &mut Game) {
    HandleFileSelectMapScrollArrows(game);
}
pub unsafe fn HandleFileSelectMapScrollArrows(game: &mut Game) {
    if ram_word!(game, map_min_x_scroll) as ::core::ffi::c_int
        - 24 as ::core::ffi::c_int
        - ram_word!(game, reg_BG1HOFS) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        DrawMapScrollArrowAndCheckToScroll(
            0x81 as uint8,
            addr_kLeftMapScrollArrowData as uint16,
            game,
        );
    }
    if ram_word!(game, map_max_x_scroll) as ::core::ffi::c_int
        - 232 as ::core::ffi::c_int
        - ram_word!(game, reg_BG1HOFS) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        DrawMapScrollArrowAndCheckToScroll(
            0x81 as uint8,
            addr_kRightMapScrollArrowData as uint16,
            game,
        );
    }
    if ram_word!(game, map_min_y_scroll) as ::core::ffi::c_int
        - 64 as ::core::ffi::c_int
        - ram_word!(game, reg_BG1VOFS) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        DrawMapScrollArrowAndCheckToScroll(
            0x81 as uint8,
            addr_kUpMapScrollArrowData as uint16,
            game,
        );
    }
    if ram_word!(game, map_max_y_scroll) as ::core::ffi::c_int
        - 145 as ::core::ffi::c_int
        - ram_word!(game, reg_BG1VOFS) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if ram_word!(game, map_scrolling_direction)
            == (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81af50)) as *const uint8
                as *mut MapScrollArrowData))
                .map_scroll_dir
        {
            set_ram_word!(game.map_scrolling_gear_switch_timer = 0 as uint16);
            set_ram_word!(game.map_scrolling_direction = 0 as uint16);
            set_ram_word!(game.map_scrolling_speed_index = 0 as uint16);
        }
    } else {
        DrawMapScrollArrowAndCheckToScroll(
            0x81 as uint8,
            addr_kDownMapScrollArrowData as uint16,
            game,
        );
    };
}
pub unsafe fn FileSelectMap_11(game: &mut Game) {
    DrawFileSelectMapIcons(game);
    DisplayMapElevatorDestinations(game);
    let ref mut fresh25 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh25 = (*fresh25).wrapping_add(1);
}
pub unsafe fn FileSelectMap_13(game: &mut Game) {
    DrawFileSelectMapIcons(game);
    DisplayMapElevatorDestinations(game);
    HandleFadeOut(game);
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 0i32
    {
        let ref mut fresh24 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh24 = (*fresh24).wrapping_add(1);
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos =
            32 as uint16;
    }
}
pub unsafe fn FileSelectMap_14(game: &mut Game) {
    let fresh22 = &raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
        .offset(0))
    .x_pos;
    fresh22.write_unaligned(fresh22.read_unaligned().wrapping_sub(1));
    if fresh22.read_unaligned() == 0 {
        ScreenOff(game);
        let ref mut fresh23 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
        *fresh23 = (*fresh23).wrapping_add(1);
        set_ram_word!(game.menu_index = 0 as uint16);
    }
}
pub unsafe fn FileSelectMap_15_ClearTileMap(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 18 as uint8;
    let mut i: ::core::ffi::c_int = 2046 as ::core::ffi::c_int;
    while i >= 0i32 {
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 15 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let v1: uint16 = ram_word!(game, vram_write_queue_tail);
    let v2: *mut VramWriteEntry = gVramWriteEntry(
        ram_word!(game, vram_write_queue_tail) as ::core::ffi::c_int,
        game,
    );
    (*v2).size = 2048 as uint16;
    (*v2).src.addr =
        (&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut uint16) = 126 as uint16;
    (*v2).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x58) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    set_ram_word!(game.vram_write_queue_tail = (v1).wrapping_add(7u16));
    let ref mut fresh21 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh21 = (*fresh21).wrapping_add(1);
}
pub unsafe fn FileSelectMap_16_LoadPalettes(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v1: int16 = 0;
    LoadMenuPalettes(game);
    v0 = 0 as int16;
    loop {
        v1 = v0;
        let fresh19 = v0;
        v0 = v0 + 1;
        LoadInactiveAreaMapForegroundColors(fresh19 as uint16, game);
        if ((v1).wrapping_sub(5i16) >= 0i16) {
            break;
        }
    }
    LoadActiveAreaMapForegroundColors(ram_word!(game, area_index), game);
    LoadAreaSelectBackgroundTilemap(ram_word!(game, area_index), game);
    let ref mut fresh20 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh20 = (*fresh20).wrapping_add(1);
}
pub unsafe fn FileSelectMap_20_SetupExpandingSquare(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x85) = 0 as uint8;
    QueueSfx1_Max6(0x3c as uint16, game);
    set_ram_word!(
        game.expand_sq_timer = (kRoomSelectMapExpandingSquareTimers
            [ram_word!(game, area_index) as usize])
            .wrapping_sub(12u16)
    );
    set_ram_word!(
        game.expand_sq_left_subvel =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34)) as *const uint8
                as *mut ExpandingSquareVels)
                .offset(0))
            .left_subvel as *mut uint8)
                .offset(
                    ((16u16).wrapping_mul(ram_word!(game, area_index))) as ::core::ffi::c_int
                        as isize,
                ) as *mut uint16)
    );
    set_ram_word!(
        game.expand_sq_left_vel =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34)) as *const uint8
                as *mut ExpandingSquareVels)
                .offset(0))
            .left_vel as *mut uint8)
                .offset(
                    ((16u16).wrapping_mul(ram_word!(game, area_index))) as ::core::ffi::c_int
                        as isize,
                ) as *mut uint16)
    );
    set_ram_word!(
        game.expand_sq_right_subvel =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34)) as *const uint8
                as *mut ExpandingSquareVels)
                .offset(0))
            .right_subvel as *mut uint8)
                .offset(
                    ((16u16).wrapping_mul(ram_word!(game, area_index))) as ::core::ffi::c_int
                        as isize,
                ) as *mut uint16)
    );
    set_ram_word!(
        game.expand_sq_right_vel =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34)) as *const uint8
                as *mut ExpandingSquareVels)
                .offset(0))
            .right_vel as *mut uint8)
                .offset(
                    ((16u16).wrapping_mul(ram_word!(game, area_index))) as ::core::ffi::c_int
                        as isize,
                ) as *mut uint16)
    );
    set_ram_word!(
        game.expand_sq_top_subvel =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34)) as *const uint8
                as *mut ExpandingSquareVels)
                .offset(0))
            .top_subvel as *mut uint8)
                .offset(
                    ((16u16).wrapping_mul(ram_word!(game, area_index))) as ::core::ffi::c_int
                        as isize,
                ) as *mut uint16)
    );
    set_ram_word!(
        game.expand_sq_top_vel =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34)) as *const uint8
                as *mut ExpandingSquareVels)
                .offset(0))
            .top_vel as *mut uint8)
                .offset(
                    ((16u16).wrapping_mul(ram_word!(game, area_index))) as ::core::ffi::c_int
                        as isize,
                ) as *mut uint16)
    );
    set_ram_word!(
        game.expand_sq_bottom_subvel =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34)) as *const uint8
                as *mut ExpandingSquareVels)
                .offset(0))
            .bottom_subvel as *mut uint8)
                .offset(
                    ((16u16).wrapping_mul(ram_word!(game, area_index))) as ::core::ffi::c_int
                        as isize,
                ) as *mut uint16)
    );
    set_ram_word!(
        game.expand_sq_bottom_vel =
            *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x81aa34)) as *const uint8
                as *mut ExpandingSquareVels)
                .offset(0))
            .bottom_vel as *mut uint8)
                .offset(
                    ((16u16).wrapping_mul(ram_word!(game, area_index))) as ::core::ffi::c_int
                        as isize,
                ) as *mut uint16)
    );
    set_ram_word!(game.expand_sq_left_pos = 8 as uint16);
    set_ram_word!(game.expand_sq_right_pos = 248 as uint16);
    set_ram_word!(game.expand_sq_top_pos = 8 as uint16);
    set_ram_word!(game.expand_sq_bottom_pos = 216 as uint16);
    set_ram_word!(game.expand_sq_left_subpos = 0 as uint16);
    set_ram_word!(game.expand_sq_right_subpos = 0 as uint16);
    set_ram_word!(game.expand_sq_top_subpos = 0 as uint16);
    set_ram_word!(game.expand_sq_bottom_subpos = 0 as uint16);
    SetupRoomSelectMapExpandingSquareTransHDMA(game);
    let ref mut fresh16 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh16 = (*fresh16).wrapping_add(1);
    *(&raw mut game.g_ram as *mut uint8).offset(0x85) = 12 as uint8;
    WriteReg(HDMAEN as uint16, 0xc as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 50 as uint8;
    WriteReg(W12SEL as uint16, 0x32 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 2 as uint8;
    WriteReg(W34SEL as uint16, 2 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 48 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 34 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_0 = 0 as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x9e10) as *mut ExpandingSquareTransitionHdma)
        .offset(3))
    .field_0 = 0 as uint8;
    set_ram_word!(game.reg_BG1VOFS = 0 as uint16);
    set_ram_word!(game.reg_BG1HOFS = 0 as uint16);
}
pub unsafe fn FileSelectMap_21_MoveExpandingSquare(game: &mut Game) {
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x9e32) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x9e30) as *mut uint16,
        -((ram_word!(game, expand_sq_left_vel) as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | ram_word!(game, expand_sq_left_subvel) as ::core::ffi::c_int) as uint32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x9e36) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x9e34) as *mut uint16,
        -((ram_word!(game, expand_sq_right_vel) as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | ram_word!(game, expand_sq_right_subvel) as ::core::ffi::c_int) as uint32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x9e3a) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x9e38) as *mut uint16,
        -((ram_word!(game, expand_sq_top_vel) as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | ram_word!(game, expand_sq_top_subvel) as ::core::ffi::c_int) as uint32,
    );
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x9e3e) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x9e3c) as *mut uint16,
        -((ram_word!(game, expand_sq_bottom_vel) as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | ram_word!(game, expand_sq_bottom_subvel) as ::core::ffi::c_int) as uint32,
    );
    SetupRoomSelectMapExpandingSquareTransHDMA(game);
    DrawAreaSelectMapLabels(game);
    let ref mut fresh14 = *((&raw mut game.g_ram as *mut uint8).offset(0x9e50) as *mut uint16);
    *fresh14 = (*fresh14).wrapping_sub(1);
    if (*fresh14 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        let ref mut fresh15 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh15 = (*fresh15).wrapping_sub(15u16);
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
    }
}
pub unsafe fn NewSaveFile(game: &mut Game) {
    set_ram_word!(game.samus_max_health = 99 as uint16);
    set_ram_word!(game.samus_health = 99 as uint16);
    set_ram_word!(game.samus_max_missiles = 0 as uint16);
    set_ram_word!(game.samus_missiles = 0 as uint16);
    set_ram_word!(game.samus_max_super_missiles = 0 as uint16);
    set_ram_word!(game.samus_super_missiles = 0 as uint16);
    set_ram_word!(game.samus_max_power_bombs = 0 as uint16);
    set_ram_word!(game.samus_power_bombs = 0 as uint16);
    set_ram_word!(game.hud_item_index = 0 as uint16);
    set_ram_word!(game.collected_beams = 0 as uint16);
    set_ram_word!(game.equipped_beams = 0 as uint16);
    set_ram_word!(game.collected_items = 0 as uint16);
    set_ram_word!(game.equipped_items = 0 as uint16);
    set_ram_word!(game.reserve_health_mode = 0 as uint16);
    set_ram_word!(game.samus_max_reserve_health = 0 as uint16);
    set_ram_word!(game.samus_reserve_health = 0 as uint16);
    set_ram_word!(game.samus_reserve_missiles = 0 as uint16);
    set_ram_word!(game.button_config_up = kButton_Up as uint16);
    set_ram_word!(game.button_config_down = kButton_Down as uint16);
    set_ram_word!(game.button_config_left = kButton_Left as uint16);
    set_ram_word!(game.button_config_right = kButton_Right as uint16);
    set_ram_word!(game.button_config_jump_a = kButton_A as uint16);
    set_ram_word!(game.button_config_run_b = kButton_B as uint16);
    set_ram_word!(game.button_config_shoot_x = kButton_X as uint16);
    set_ram_word!(game.button_config_itemcancel_y = kButton_Y as uint16);
    set_ram_word!(game.button_config_itemswitch = kButton_Select as uint16);
    set_ram_word!(game.button_config_aim_up_R = kButton_R as uint16);
    set_ram_word!(game.button_config_aim_down_L = kButton_L as uint16);
    set_ram_word!(game.game_time_frames = 0 as uint16);
    set_ram_word!(game.game_time_seconds = 0 as uint16);
    set_ram_word!(game.game_time_minutes = 0 as uint16);
    set_ram_word!(game.game_time_hours = 0 as uint16);
    set_ram_word!(game.japanese_text_flag = 0 as uint16);
    set_ram_word!(game.moonwalk_flag = 0 as uint16);
    set_ram_word!(game.hud_auto_cancel_flag = 0 as uint16);
    set_ram_word!(game.debug_flag = 1 as uint16);
    set_ram_word!(game.UNUSED_word_7E09E8 = 1 as uint16);
    let mut v0: uint16 = 0 as uint16;
    loop {
        let v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd830)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd870)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd870)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd8b0)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd8f0) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd908)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd8f8)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd8f8)
            .offset((v0 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) = 0 as uint16;
        v0 = (v0).wrapping_add(2u16);
        if (((v0) as i16).wrapping_sub(8i16) >= 0i16) {
            break;
        }
    }
    loop {
        let v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd830)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd870)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd870)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd8b0)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        v0 = (v0).wrapping_add(2u16);
        if (((v0) as i16).wrapping_sub(64i16) >= 0i16) {
            break;
        }
    }
    let mut v3: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd52) as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        v3 = (v3).wrapping_add(2u16);
        if (((v3) as i16).wrapping_sub(1792i16) >= 0i16) {
            break;
        }
    }
}
pub unsafe fn FileSelectClearRestOfMenuTilemapRow(mut v0: uint16, game: &mut Game) {
    let mut n: ::core::ffi::c_int = 32 as ::core::ffi::c_int
        - ((v0 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int) >> 1 as ::core::ffi::c_int);
    loop {
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768) as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 15 as uint16;
        v0 = (v0).wrapping_add(2u16);
        n -= 1;
        if (n == 0) {
            break;
        }
    }
}
pub unsafe fn LoadMenuTilemap(mut k: uint16, mut j: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut k_bak: ::core::ffi::c_int = k as ::core::ffi::c_int;
    loop {
        loop {
            v2 = *(RomPtr_81(j, game) as *mut uint16) as int16;
            if v2 != -2i16 {
                break;
            }
            j = (j).wrapping_add(2u16);
            k_bak += 64 as ::core::ffi::c_int;
            k = k_bak as uint16;
        }
        if v2 == -1i16 {
            break;
        }
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(768) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .palette_index)
                | ((v2) as u16);
        k = (k).wrapping_add(2u16);
        j = (j).wrapping_add(2u16);
    }
}
