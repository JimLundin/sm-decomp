pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::game_states::*;
use crate::game_state::Game;
use crate::ram_map::{ram_word, set_ram_word};
use crate::rtl::AddToHiLo;
pub use crate::snes::registers::*;

use crate::game::sm_81::{DrawSpritemap, LoadFromSram, SoftReset};
use crate::game::sm_82::{
    LoadCRETilesTilesetTilesAndPalette, LoadColorsForSpritesBeamsAndEnemies,
    LoadDestinationRoomThings, LoadLevelDataAndOtherThings, LoadLibraryBackground, LoadRoomMusic,
    NullFunc, ResetSoundQueues, UpdateMusicTrackIndex,
};
use crate::game::sm_84::{ClearPLMs, LoadRoomPlmGfx, SpawnHardcodedPlm};
use crate::game::sm_86::ClearEprojs;
use crate::game::sm_87::ClearAnimtiles;
use crate::game::sm_88::{InitializeSpecialEffectsForNewRoom, WaitUntilEndOfVblankAndClearHdma};
use crate::game::sm_89::LoadFXHeader;
use crate::game::sm_8b::InitializeIoDisplayLogo_Async;
use crate::game::sm_8d::ClearPaletteFXObjects;
use crate::game::sm_90::{InitializeMiniMapBroken, UpdateBeamTilesAndPalette};
use crate::game::sm_94::kSinCosTable8bit_Sext;
use crate::game::sm_a0::LoadEnemies;
use crate::rtl::{
    Load24, Mult8x8, ReadRegWord, RomPtr, RtlApuUpload, RtlApuWrite, RtlWriteSram, SnesDivide,
    SnesModulus, Unreachable, WriteReg, WriteRegWord,
};
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
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, size_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t,
    CoroutineRet, Func_U8, Func_V, LongPtr, OamEnt, SnesRegs, VoidP, VramWriteEntry,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct LoadStationList {
    pub room_ptr_: VoidP,
    pub door_ptr: VoidP,
    pub door_bts: uint16,
    pub screen_x_pos: uint16,
    pub screen_y_pos: uint16,
    pub samus_y_offset: uint16,
    pub samus_x_offset: uint16,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct SamusTileAnimationTileDefs {
    pub src: LongPtr,
    pub part1_size: uint16,
    pub part2_size: uint16,
}
pub use crate::types::Buttons;
pub use crate::types::{GameState, Mode7VramWriteQueue, RoomDefHeader};
pub use crate::types::{
    Ram4000, Ram4000_Backups, StartDmaCopy, TileTable, TileTables, VramReadQueueEnt,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Mode7CgvmWriteQueue {
    pub tag: uint8,
    pub src_addr: LongPtr,
    pub count: uint16,
    pub vram_addr: uint16,
    pub vmain: uint8,
}
pub use crate::types::Consts_60;
pub use crate::types::Consts_80;
pub const addr_word_80A060: Consts_80 = 41056;
pub const addrl_kDigitTilesetsWeapon: Consts_80 = 8429011;
pub const addrl_kDigitTilesetsHealth: Consts_80 = 8428991;
pub const addr_kHudTilemaps_Missiles: Consts_80 = 39331;
pub const addr_kHudTilemaps_AutoReserve: Consts_80 = 39307;
pub use crate::types::SpawnHardcodedPlmArgs;
#[inline]
unsafe fn RomPtr_80(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0x800000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtr_92(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0x920000u32) | ((addr) as u32), game)
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
unsafe fn gVramWriteEntry(a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry
}
#[inline]
unsafe fn get_RoomDefHeader(a: uint16, game: &mut Game) -> *mut RoomDefHeader {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomDefHeader
}
pub const fnDoorTransition_FadeInScreenAndFinish: ::core::ffi::c_int = 0x82e737;
pub unsafe fn APU_UploadBank(addr: uint32, game: &mut Game) {
    if !game.g_use_my_apu_code {
        crate::native_audio::upload(addr, game);
        return;
    }
    RtlApuUpload(RomPtr(addr, game), game);
}
pub unsafe fn NextRandom(game: &mut Game) -> uint16 {
    let mut RegWord: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16
        as *mut uint8)
        .offset(0)) as u16)
        .wrapping_mul(5u16);
    let Reg: uint8 =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ))
        .wrapping_mul(5u8);
    let carry: ::core::ffi::c_int = *(&raw mut RegWord as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) as ::core::ffi::c_int
        + Reg as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int;
    *(&raw mut RegWord as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) = carry as uint8;
    let result: uint16 = ((carry >> 8 as ::core::ffi::c_int)
        + RegWord as ::core::ffi::c_int
        + 17 as ::core::ffi::c_int) as uint16;
    set_ram_word!(game.random_number = result);
    result
}
pub unsafe fn ReleaseButtonsFilter(v0: uint16, game: &mut Game) {
    set_ram_word!(game.timed_held_input_timer_reset = v0);
    let v1: bool = !(ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        & ram_word!(game, joypad1_lastkeys) as ::core::ffi::c_int
        == ram_word!(game, joypad_released_keys) as ::core::ffi::c_int;
    set_ram_word!(
        game.joypad_released_keys = (!(ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int)
            & ram_word!(game, joypad1_lastkeys) as ::core::ffi::c_int)
            as uint16
    );
    if !v1 {
        set_ram_word!(game.timed_held_input_timer = ram_word!(game, timed_held_input_timer_reset));
        set_ram_word!(game.timed_held_input = 0 as uint16);
    } else {
        let ref mut fresh87 = *((&raw mut game.g_ram as *mut uint8).offset(0x5db) as *mut uint16);
        *fresh87 = (*fresh87).wrapping_sub(1);
        if (*fresh87 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
            set_ram_word!(game.timed_held_input = 0 as uint16);
        } else {
            set_ram_word!(game.timed_held_input_timer = 0 as uint16);
            set_ram_word!(game.previous_timed_held_input = ram_word!(game, timed_held_input));
            set_ram_word!(
                game.timed_held_input = (!(ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int)
                    & ram_word!(game, joypad1_lastkeys) as ::core::ffi::c_int)
                    as uint16
            );
        }
    }
    set_ram_word!(
        game.newly_held_down_timed_held_input = (ram_word!(game, timed_held_input)
            as ::core::ffi::c_int
            & (ram_word!(game, previous_timed_held_input) as ::core::ffi::c_int
                ^ ram_word!(game, timed_held_input) as ::core::ffi::c_int))
            as uint16
    );
}
pub fn PrepareBitAccess(a: uint16, game: &mut Game) -> uint16 {
    let bitmask = 1u16 << (a & 7);
    game.g_ram[0x5e7..0x5e9].copy_from_slice(&bitmask.to_le_bytes());
    a >> 3
}
pub unsafe fn SetBossBitForCurArea(a: uint16, game: &mut Game) {
    game.g_ram[0x5e7] = a as u8; // The ROM stores the mask with an 8-bit accumulator.
    let ref mut fresh91 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(ram_word!(game, area_index) as isize);
    *fresh91 = (*fresh91) | ((a) as u8);
}
pub unsafe fn ClearBossBitForCurArea(a: uint16, game: &mut Game) {
    let ref mut fresh19 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(ram_word!(game, area_index) as isize);
    *fresh19 = (*fresh19 as ::core::ffi::c_int & !(a as ::core::ffi::c_int)) as uint8;
}
pub unsafe fn CheckBossBitForCurArea(a: uint16, game: &mut Game) -> uint8 {
    game.g_ram[0x5e7] = a as u8; // The ROM stores the mask with an 8-bit accumulator.
    ((a as ::core::ffi::c_int
        & *(&raw mut game.g_ram as *mut uint8)
            .offset(0xd828)
            .offset(ram_word!(game, area_index) as isize) as ::core::ffi::c_int)
        != 0i32) as uint8
}
pub unsafe fn SetEventHappened(a: uint16, game: &mut Game) {
    let v1: uint16 = PrepareBitAccess(a, game);
    let ref mut fresh94 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd820)
        .offset(v1 as isize);
    *fresh94 = (*fresh94) | ((ram_word!(game, bitmask)) as u8);
}
pub unsafe fn ClearEventHappened(v0: uint16, game: &mut Game) {
    let v1: uint16 = PrepareBitAccess(v0, game);
    let ref mut fresh20 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd820)
        .offset(v1 as isize);
    *fresh20 = (*fresh20 as ::core::ffi::c_int & !(ram_word!(game, bitmask) as ::core::ffi::c_int))
        as uint8;
}
pub unsafe fn CheckEventHappened(a: uint16, game: &mut Game) -> uint16 {
    let idx: uint16 = PrepareBitAccess(a, game);
    ((ram_word!(game, bitmask) as ::core::ffi::c_int
        & *(&raw mut game.g_ram as *mut uint8)
            .offset(0xd820)
            .offset(idx as isize) as ::core::ffi::c_int)
        != 0i32) as uint16
}
pub unsafe fn CopySuperMetroidString(game: &mut Game) {
    memcpy(
        game.g_sram.offset(0x1fe0) as *mut uint8 as *mut ::core::ffi::c_void,
        b"supermetroid\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        12 as size_t,
    );
    RtlWriteSram(game);
}
pub unsafe fn VerifySRAM(game: &mut Game) {
    set_ram_word!(game.num_demo_sets = 3 as uint16);
    if LoadFromSram(0 as uint16, game) != 0u8
        && LoadFromSram(1 as uint16, game) != 0u8
        && LoadFromSram(2 as uint16, game) != 0u8
    {
        memcpy(
            game.g_sram.offset(0x1fe0) as *mut uint8 as *mut ::core::ffi::c_void,
            b"madadameyohn\0" as *const u8 as *const ::core::ffi::c_char
                as *const ::core::ffi::c_void,
            12 as size_t,
        );
        RtlWriteSram(game);
    } else if memcmp(
        game.g_sram.offset(0x1fe0) as *mut uint8 as *const ::core::ffi::c_void,
        b"supermetroid\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        12 as size_t,
    ) == 0
    {
        set_ram_word!(game.num_demo_sets = 4 as uint16);
    }
}
pub unsafe fn Multiply16x16(a: uint16, j: uint16, game: &mut Game) -> uint32 {
    game.g_ram[0x5e9..0x5eb].copy_from_slice(&a.to_le_bytes());
    game.g_ram[0x5eb..0x5ed].copy_from_slice(&j.to_le_bytes());
    let result: uint32 = (a as uint32).wrapping_mul(j as uint32);
    result
}
pub unsafe fn WaitForNMI_Async(game: &mut Game) -> CoroutineRet {
    if *(&raw mut game.g_ram as *mut uint8).offset(0x782) != 0 {
        *(&raw mut game.g_ram as *mut uint8).offset(0x782) = 0 as uint8;
        return 0 as CoroutineRet;
    }
    *(&raw mut game.g_ram as *mut uint8).offset(0x5b4) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x782) = 1 as uint8;
    1 as CoroutineRet
}
pub unsafe fn WaitForNMI_NoUpdate_Async(game: &mut Game) -> CoroutineRet {
    if *(&raw mut game.g_ram as *mut uint8).offset(0x782) != 0 {
        *(&raw mut game.g_ram as *mut uint8).offset(0x782) = 0 as uint8;
        return 0 as CoroutineRet;
    }
    *(&raw mut game.g_ram as *mut uint8).offset(0x782) = 1 as uint8;
    1 as CoroutineRet
}
pub unsafe fn EnableNMI(game: &mut Game) {
    let v0: uint8 = (*(&raw mut game.g_ram as *mut uint8).offset(0x84)) | (0x80u8);
    WriteReg(
        NMITIMEN as uint16,
        (*(&raw mut game.g_ram as *mut uint8).offset(0x84)) | (0x80u8),
        game,
    );
    *(&raw mut game.g_ram as *mut uint8).offset(0x84) = v0;
}
pub unsafe fn DisableNMI(game: &mut Game) {
    let v0: uint8 = (*(&raw mut game.g_ram as *mut uint8).offset(0x84)) & (0x7fu8);
    WriteReg(
        NMITIMEN as uint16,
        (*(&raw mut game.g_ram as *mut uint8).offset(0x84)) & (0x7fu8),
        game,
    );
    *(&raw mut game.g_ram as *mut uint8).offset(0x84) = v0;
}
pub unsafe fn ScreenOff(game: &mut Game) {
    let ref mut fresh89 = *(&raw mut game.g_ram as *mut uint8).offset(0x51);
    *fresh89 = (*fresh89) | (0x80u8);
}
pub unsafe fn ScreenOn(game: &mut Game) {
    let ref mut fresh90 = *(&raw mut game.g_ram as *mut uint8).offset(0x51);
    *fresh90 = (*fresh90 as ::core::ffi::c_int & !(0x80 as ::core::ffi::c_int)) as uint8;
}
pub unsafe fn memset7E(mut k: *mut uint16, a: uint16, mut j: uint16) {
    loop {
        let fresh99 = k;
        k = k.offset(1);
        *fresh99 = a;
        j = (j).wrapping_sub(2u16);
        if (j == 0) {
            break;
        }
    }
}
pub unsafe fn Vector_RESET_Async(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut uint8 = (&raw mut game.g_ram as *mut uint8).offset(0x611);
    let mut bak: uint16 = 0;
    let mut current_block_63: u64;
    match *_state_variable_ as ::core::ffi::c_int {
        1 => {
            WriteReg(MEMSEL as uint16, 1 as uint8, game);
            *(&raw mut game.g_ram as *mut uint8).offset(0x86) = 1 as uint8;
            bak = ram_word!(game, bug_fix_counter);
            memset(
                &raw mut game.g_ram as *mut uint8 as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                8192 as size_t,
            );
            set_ram_word!(game.bug_fix_counter = bak);
            current_block_63 = 17803482508355472374;
        }
        2 => {
            current_block_63 = 17803482508355472374;
        }
        3 => {
            current_block_63 = 3640593987805443782;
        }
        _ => {
            current_block_63 = 5330834795799507926;
        }
    }
    if current_block_63 == 17803482508355472374 {
        let mut _coret_: uint8 = InitializeIoDisplayLogo_Async(game);
        if _coret_ != 0 {
            *_state_variable_ = 2 as uint8;
            return _coret_;
        }
        current_block_63 = 3640593987805443782;
    }
    if current_block_63 == 3640593987805443782 {
        APU_UploadBank(0xcf8000 as uint32, game);
        WriteReg(INIDISP as uint16, 0x8f as uint8, game);
        bak = ram_word!(game, bug_fix_counter);
        memset(
            &raw mut game.g_ram as *mut uint8 as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            0x10000 as ::core::ffi::c_int as size_t,
        );
        set_ram_word!(game.bug_fix_counter = bak);
        WriteReg(NMITIMEN as uint16, 0 as uint8, game);
        *(&raw mut game.g_ram as *mut uint8).offset(0x84) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x8f as uint8;
        InitializeCpuIoRegs(game);
        InitializePpuIoRegs(game);
        WriteLotsOf0x1c2f(game);
        *(&raw mut game.g_ram as *mut uint8).offset(0x643).offset(0) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x643).offset(1) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x643).offset(2) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x646).offset(0) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x646).offset(1) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x646).offset(2) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x649).offset(0) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x649).offset(1) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x649).offset(2) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x64d).offset(0) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x64d).offset(1) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x64d).offset(2) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x650).offset(0) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x650).offset(1) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x650).offset(2) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x656).offset(0) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x666).offset(0) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x676).offset(0) = 0 as uint8;
        set_ram_word!(game.oam_next_ptr = 0 as uint16);
        set_ram_word!(game.reg_OAMaddr_UNUSED = 0 as uint16);
        ClearOamExt(game);
        ClearUnusedOam(game);
        set_ram_word!(game.nmi_copy_samus_halves = 0 as uint16);
        set_ram_word!(game.nmi_copy_samus_top_half_src = 0 as uint16);
        set_ram_word!(game.nmi_copy_samus_bottom_half_src = 0 as uint16);
        EnableNMI(game);
        RtlApuWrite(APUI00, 0 as uint8, game);
        RtlApuWrite(APUI01, 0 as uint8, game);
        RtlApuWrite(APUI02, 0 as uint8, game);
        RtlApuWrite(APUI03, 0 as uint8, game);
        set_ram_word!(game.random_number = 97 as uint16);
        set_ram_word!(game.music_timer = 0 as uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x629) as *mut uint16).offset(0) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x629) as *mut uint16).offset(1) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x629) as *mut uint16).offset(2) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x629) as *mut uint16).offset(3) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x629) as *mut uint16).offset(4) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x629) as *mut uint16).offset(5) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x629) as *mut uint16).offset(6) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x629) as *mut uint16).offset(7) = 0 as uint16;
        set_ram_word!(game.enable_debug = 0 as uint16);
        VerifySRAM(game);
        set_ram_word!(game.debug_disable_sounds = 0 as uint16);
        set_ram_word!(game.sound_handler_downtime = 0 as uint16);
    }
    *_state_variable_ = 2 as uint8;
    0 as CoroutineRet
}
pub unsafe fn InvalidInterrupt_Crash(game: &mut Game) {
    printf(b"InvalidInterrupt_Crash\n\0" as *const u8 as *const ::core::ffi::c_char);
    Unreachable(game);
    loop {}
}
pub unsafe fn LoadMirrorOfExploredMapTiles(game: &mut Game) {
    let mut v1: uint16 = u16::swap_bytes(ram_word!(game, area_index));
    let mut v2: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x7f7)
            .offset(v2 as isize) as *mut uint8 as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xcd52) as *mut uint16)
                .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v1 = (v1).wrapping_add(2u16);
        v2 = (v2).wrapping_add(2u16);
        if (((v2) as i16).wrapping_sub(256i16) >= 0i16) {
            break;
        }
    }
    set_ram_word!(
        game.has_area_map = *(&raw mut game.g_ram as *mut uint8)
            .offset(0xd908)
            .offset(ram_word!(game, area_index) as isize) as uint16
    );
}
pub unsafe fn SaveExploredMapTilesToSaved(game: &mut Game) {
    let mut v1: uint16 = u16::swap_bytes(ram_word!(game, area_index));
    let mut v2: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd52) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x7f7)
                .offset(v2 as isize) as *mut uint8 as *mut uint16);
        v1 = (v1).wrapping_add(2u16);
        v2 = (v2).wrapping_add(2u16);
        if (((v2) as i16).wrapping_sub(256i16) >= 0i16) {
            break;
        }
    }
    if ram_word!(game, has_area_map) != 0 {
        let ref mut fresh88 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd908)
            .offset(ram_word!(game, area_index) as isize)
            as *mut uint8 as *mut uint16);
        *fresh88 = (*fresh88) | (0xffu16);
    }
}
pub unsafe fn InitializeCpuIoRegs(game: &mut Game) {
    WriteReg(NMITIMEN as uint16, 1 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x84) = 1 as uint8;
    WriteReg(WRIO as uint16, 0 as uint8, game);
    WriteReg(WRMPYA as uint16, 0 as uint8, game);
    WriteReg(WRMPYB as uint16, 0 as uint8, game);
    WriteReg(WRDIVL as uint16, 0 as uint8, game);
    WriteReg(WRDIVH as uint16, 0 as uint8, game);
    WriteReg(WRDIVB as uint16, 0 as uint8, game);
    WriteReg(HTIMEL as uint16, 0 as uint8, game);
    WriteReg(HTIMEH as uint16, 0 as uint8, game);
    WriteReg(VTIMEL as uint16, 0 as uint8, game);
    WriteReg(VTIMEH as uint16, 0 as uint8, game);
    WriteReg(MDMAEN as uint16, 0 as uint8, game);
    WriteReg(HDMAEN as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x85) = 0 as uint8;
    WriteReg(MEMSEL as uint16, 1 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x86) = 1 as uint8;
}
pub unsafe fn InitializePpuIoRegs(game: &mut Game) {
    WriteReg(INIDISP as uint16, 0x8f as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x8f as uint8;
    WriteReg(OBSEL as uint16, 3 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x52) = 3 as uint8;
    WriteReg(OAMADDL as uint16, 0 as uint8, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x53) as *mut uint16 as *mut uint8).offset(0) =
        0 as uint8;
    WriteReg(OAMADDH as uint16, 0x80 as uint8, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x53) as *mut uint16 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) = 0x80 as uint8;
    WriteReg(OAMDATA as uint16, 0 as uint8, game);
    WriteReg(OAMDATA as uint16, 0 as uint8, game);
    WriteReg(BGMODE as uint16, 9 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 9 as uint8;
    WriteReg(MOSAIC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x57) = 0 as uint8;
    WriteReg(BG1SC as uint16, 0x40 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 64 as uint8;
    WriteReg(BG2SC as uint16, 0x44 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 68 as uint8;
    WriteReg(BG3SC as uint16, 0x48 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5a) = 72 as uint8;
    WriteReg(BG4SC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5c) = 0 as uint8;
    WriteReg(BG12NBA as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 0 as uint8;
    WriteReg(BG34NBA as uint16, 5 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 5 as uint8;
    WriteReg(BG1HOFS as uint16, 0 as uint8, game);
    WriteReg(BG1HOFS as uint16, 0 as uint8, game);
    WriteReg(BG1VOFS as uint16, 0 as uint8, game);
    WriteReg(BG1VOFS as uint16, 0 as uint8, game);
    WriteReg(BG2HOFS as uint16, 0 as uint8, game);
    WriteReg(BG2HOFS as uint16, 0 as uint8, game);
    WriteReg(BG2VOFS as uint16, 0 as uint8, game);
    WriteReg(BG2VOFS as uint16, 0 as uint8, game);
    WriteReg(BG3HOFS as uint16, 0 as uint8, game);
    WriteReg(BG3HOFS as uint16, 0 as uint8, game);
    WriteReg(BG3VOFS as uint16, 0 as uint8, game);
    WriteReg(BG3VOFS as uint16, 0 as uint8, game);
    WriteReg(BG4HOFS as uint16, 0 as uint8, game);
    WriteReg(BG4HOFS as uint16, 0 as uint8, game);
    WriteReg(BG4VOFS as uint16, 0 as uint8, game);
    WriteReg(BG4VOFS as uint16, 0 as uint8, game);
    WriteReg(VMAIN as uint16, 0 as uint8, game);
    WriteReg(M7SEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0 as uint8;
    WriteReg(M7A as uint16, 0 as uint8, game);
    WriteReg(M7B as uint16, 0 as uint8, game);
    WriteReg(M7C as uint16, 0 as uint8, game);
    WriteReg(M7D as uint16, 0 as uint8, game);
    WriteReg(M7X as uint16, 0 as uint8, game);
    WriteReg(M7Y as uint16, 0 as uint8, game);
    WriteReg(W12SEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 0 as uint8;
    WriteReg(W34SEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 0 as uint8;
    WriteReg(WOBJSEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 0 as uint8;
    WriteReg(WH0 as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x63) = 0 as uint8;
    WriteReg(WH1 as uint16, 0xf8 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x64) = -(8 as ::core::ffi::c_int) as uint8;
    WriteReg(WH2 as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x65) = 0 as uint8;
    WriteReg(WH3 as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x66) = 0 as uint8;
    WriteReg(WBGLOG as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x67) = 0 as uint8;
    WriteReg(WOBJLOG as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x68) = 0 as uint8;
    WriteReg(TM as uint16, 0x11 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    WriteReg(TMW as uint16, 0x11 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 17 as uint8;
    WriteReg(TS as uint16, 2 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 2 as uint8;
    WriteReg(TSW as uint16, 2 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 2 as uint8;
    WriteReg(CGWSEL as uint16, 2 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 2 as uint8;
    WriteReg(CGADSUB as uint16, 0xa1 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = -(95 as ::core::ffi::c_int) as uint8;
    WriteReg(COLDATA as uint16, 0xe0 as uint8, game);
    WriteReg(COLDATA as uint16, 0xe0 as uint8, game);
    WriteReg(COLDATA as uint16, 0x80 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 0x80 as uint8;
    WriteReg(COLDATA as uint16, 0x40 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    WriteReg(COLDATA as uint16, 0x20 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 32 as uint8;
    WriteReg(SETINI as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x77) = 0 as uint8;
}
pub unsafe fn WriteLotsOf0x1c2f(game: &mut Game) {
    sub_8088EB(0x1c2f as uint16, game);
    sub_8088FE(0x1c2f as uint16, game);
    sub_808911(0x1c2f as uint16, game);
}
pub unsafe fn sub_8088EB(a: uint16, game: &mut Game) {
    memset7E(
        (&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut uint16,
        a,
        0x800 as uint16,
    );
}
pub unsafe fn sub_8088FE(a: uint16, game: &mut Game) {
    memset7E(
        (&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut uint16,
        a,
        0x800 as uint16,
    );
}
pub unsafe fn sub_808911(a: uint16, game: &mut Game) {
    memset7E(
        (&raw mut game.g_ram as *mut uint8).offset(0x6000) as *mut uint16,
        a,
        0x800 as uint16,
    );
}
fn tick_screen_fade(game: &mut Game) -> bool {
    let screen_fade_counter = u16::from_le_bytes([game.g_ram[0x725], game.g_ram[0x726]]);
    // Test the sign after the word decrement, then reload before changing brightness.
    let remaining = screen_fade_counter.wrapping_sub(1);
    let step_due = (remaining as i16) < 0;
    let next_counter = if step_due {
        let screen_fade_delay = u16::from_le_bytes([game.g_ram[0x723], game.g_ram[0x724]]);
        screen_fade_delay
    } else {
        remaining
    };
    game.g_ram[0x725..0x727].copy_from_slice(&next_counter.to_le_bytes());
    step_due
}

pub fn HandleFadeOut(game: &mut Game) {
    if !tick_screen_fade(game) {
        return;
    }
    let reg_inidisp = &mut game.g_ram[0x51];
    match *reg_inidisp & 0x0f {
        0 => {}
        1 => *reg_inidisp = 0x80,
        brightness => *reg_inidisp = brightness - 1,
    }
}

pub fn HandleFadeIn(game: &mut Game) {
    if !tick_screen_fade(game) {
        return;
    }
    let reg_inidisp = &mut game.g_ram[0x51];
    let brightness = reg_inidisp.wrapping_add(1) & 0x0f;
    if brightness != 0 {
        *reg_inidisp = brightness;
    }
}
pub fn ClearUnusedOam(game: &mut Game) {
    let oam_next_ptr = u16::from_le_bytes([game.g_ram[0x590], game.g_ram[0x591]]);
    let first_unused_sprite = usize::from(oam_next_ptr >> 2);
    for sprite in game.g_ram[0x370..0x570]
        .chunks_exact_mut(4)
        .skip(first_unused_sprite)
    {
        // Each four-byte OAM entry stores its Y coordinate in byte 1.
        sprite[1] = 0xf0;
    }
    game.g_ram[0x590..0x592].fill(0);
}

pub fn ClearOamExt(game: &mut Game) {
    game.g_ram[0x570..0x590].fill(0);
}
pub unsafe fn QueueMode7Transfers(db: uint8, k: uint16, game: &mut Game) {
    let mut p: *const uint8 = RomPtrWithBank(db, k, game);
    let mut dst: *mut uint8 = ((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
        as *mut Mode7VramWriteQueue as *mut uint8)
        .offset(ram_word!(game, mode7_vram_write_queue_tail) as ::core::ffi::c_int as isize);
    loop {
        let f: ::core::ffi::c_int = *(p as *mut uint8) as ::core::ffi::c_int;
        if f & 0x80 as ::core::ffi::c_int != 0 {
            *dst.offset(9) = 0 as uint8;
            memcpy(
                dst as *mut ::core::ffi::c_void,
                p as *const ::core::ffi::c_void,
                9 as size_t,
            );
            p = p.offset(9);
            dst = dst.offset(9);
        } else {
            if (f & 0x40 as ::core::ffi::c_int == 0) {
                break;
            }
            *dst.offset(7) = 0 as uint8;
            memcpy(
                dst as *mut ::core::ffi::c_void,
                p as *const ::core::ffi::c_void,
                7 as size_t,
            );
            p = p.offset(7);
            dst = dst.offset(7);
        }
    }
    set_ram_word!(
        game.mode7_vram_write_queue_tail = dst
            .offset_from((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
                as *mut Mode7VramWriteQueue as *mut uint8)
            as ::core::ffi::c_long as uint16
    );
}
pub unsafe fn NMI_ProcessMode7Queue(game: &mut Game) {
    if ram_word!(game, mode7_vram_write_queue_tail) != 0 {
        NMI_ProcessMode7QueueInner(
            (&raw mut game.g_ram as *mut uint8).offset(0x2d0) as *mut uint8,
            game,
        );
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x2d0)
            as *mut Mode7VramWriteQueue)
            .offset(0))
        .field_0 as *mut uint16) = 0 as uint16;
        set_ram_word!(game.mode7_vram_write_queue_tail = 0 as uint16);
    }
}
pub unsafe fn NMI_ProcessMode7QueueInner(mut p: *const uint8, game: &mut Game) {
    loop {
        let mut v2: uint8 = 0;
        loop {
            v2 = *p;
            if (v2 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) == 0i32 {
                break;
            }
            WriteReg(DMAP1 as uint16, (v2) & (0x1fu8), game);
            let v5: *mut Mode7CgvmWriteQueue = p as *mut Mode7CgvmWriteQueue;
            WriteRegWord(A1T1L as uint16, (*v5).src_addr.addr, game);
            WriteReg(A1B1 as uint16, (*v5).src_addr.bank, game);
            WriteRegWord(DAS1L as uint16, (*v5).count, game);
            if v2 as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int != 0 {
                WriteReg(BBAD1 as uint16, 0x19 as uint8, game);
            } else {
                WriteReg(BBAD1 as uint16, 0x18 as uint8, game);
            }
            WriteRegWord(VMADDL as uint16, (*v5).vram_addr, game);
            WriteReg(VMAIN as uint16, (*v5).vmain, game);
            WriteReg(MDMAEN as uint16, 2 as uint8, game);
            p = p.offset(::core::mem::size_of::<Mode7CgvmWriteQueue>() as isize);
        }
        if v2 as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int == 0 {
            break;
        }
        WriteReg(DMAP1 as uint16, (v2) & (0x1fu8), game);
        WriteRegWord(A1T1L as uint16, *(p.offset(1) as *mut uint16), game);
        WriteReg(A1B1 as uint16, *p.offset(3), game);
        WriteRegWord(DAS1L as uint16, *(p as *mut uint16).offset(2), game);
        WriteReg(BBAD1 as uint16, 0x22 as uint8, game);
        WriteReg(CGADD as uint16, *p.offset(6), game);
        WriteReg(MDMAEN as uint16, 2 as uint8, game);
        p = p.offset(7);
    }
}
pub unsafe fn NMI_ProcessVramWriteQueue(game: &mut Game) {
    let tail = game.vram_write_queue_tail();
    if tail != 0 {
        // The sentinel uses the byte tail, which need not lie on an entry boundary.
        game.write_vram_queue_word(tail, 0, 0);
        WriteRegWord(DMAP1 as u16, 0x1801, game);
        let read_word = |game: &Game, address: usize| {
            u16::from_le_bytes([game.g_ram[address], game.g_ram[address + 1]])
        };
        // Entries are seven packed bytes; the consumer advances without bank wrap.
        let mut entry = 0xd0;
        loop {
            let size = read_word(game, entry);
            if size == 0 {
                break;
            }
            WriteRegWord(DAS1L as u16, size, game);
            WriteRegWord(A1T1L as u16, read_word(game, entry + 2), game);
            WriteReg(A1B1 as u16, game.g_ram[entry + 4], game);
            let increment = if read_word(game, entry + 5) & 0x8000 != 0 {
                0x81
            } else {
                0x80
            };
            // This word write also clears VMADDL before the full destination write.
            WriteRegWord(VMAIN as u16, increment, game);
            WriteRegWord(VMADDL as u16, read_word(game, entry + 5), game);
            WriteReg(MDMAEN as u16, 2, game);
            entry += 7;
        }
    }
    game.set_vram_write_queue_tail(0);
    Nmi_ProcessHorizScrollingDma(game);
    Nmi_ProcessVertScrollingDma(game);
}
#[derive(Clone, Copy)]
enum ScrollingDmaLayout {
    Column,
    Row,
}

impl ScrollingDmaLayout {
    fn second_half_destination(self, address: u16) -> u16 {
        match self {
            Self::Column => address.wrapping_add(1),
            Self::Row => address | 0x20,
        }
    }
}

unsafe fn transfer_scrolling_tilemap(
    parameters: usize,
    initial_sources: [u16; 2],
    layout: ScrollingDmaLayout,
    game: &mut Game,
) {
    let enable = parameters + 12;
    if game.g_ram[enable] == 0 {
        return;
    }
    // Only the low enable byte is consumed, before reading transfer parameters.
    game.g_ram[enable] = 0;
    // Sizes, destinations and wrapped sources are three pairs of RAM words.
    let read_parameter = |game: &Game, offset: usize| {
        u16::from_le_bytes([
            game.g_ram[parameters + offset],
            game.g_ram[parameters + offset + 1],
        ])
    };

    let destination = read_parameter(game, 4);
    WriteRegWord(VMADDL as u16, destination, game);
    WriteRegWord(DMAP1 as u16, 0x1801, game);
    WriteRegWord(A1T1L as u16, initial_sources[0], game);
    WriteReg(A1B1 as u16, 0x7e, game);
    let size = read_parameter(game, 0);
    // A zero unwrapped size still triggers a full 65,536-byte DMA.
    WriteRegWord(DAS1L as u16, size, game);
    WriteReg(MDMAEN as u16, 2, game);
    WriteRegWord(
        VMADDL as u16,
        layout.second_half_destination(destination),
        game,
    );
    WriteRegWord(DAS1L as u16, size, game);
    WriteRegWord(A1T1L as u16, initial_sources[1], game);
    WriteReg(MDMAEN as u16, 2, game);

    // Prepare the wrapped source even when there is no wrapped transfer.
    WriteRegWord(A1T1L as u16, read_parameter(game, 8), game);
    let wrapped_size = read_parameter(game, 2);
    if wrapped_size != 0 {
        WriteRegWord(DAS1L as u16, wrapped_size, game);
        let wrapped_destination = read_parameter(game, 6);
        WriteRegWord(VMADDL as u16, wrapped_destination, game);
        WriteReg(MDMAEN as u16, 2, game);
        WriteRegWord(
            VMADDL as u16,
            layout.second_half_destination(wrapped_destination),
            game,
        );
        WriteRegWord(DAS1L as u16, wrapped_size, game);
        WriteRegWord(A1T1L as u16, read_parameter(game, 10), game);
        WriteReg(MDMAEN as u16, 2, game);
    }
}

pub unsafe fn Nmi_ProcessHorizScrollingDma(game: &mut Game) {
    WriteReg(VMAIN as u16, 0x81, game);
    let layout = ScrollingDmaLayout::Column;
    transfer_scrolling_tilemap(0x956, [0xc8c8, 0xc908], layout, game);
    transfer_scrolling_tilemap(0x972, [0xc9d0, 0xca10], layout, game);
}

pub unsafe fn Nmi_ProcessVertScrollingDma(game: &mut Game) {
    WriteReg(VMAIN as u16, 0x80, game);
    let layout = ScrollingDmaLayout::Row;
    transfer_scrolling_tilemap(0x964, [0xc948, 0xc98c], layout, game);
    transfer_scrolling_tilemap(0x980, [0xca50, 0xca94], layout, game);
}
pub unsafe fn NMI_ProcessVramReadQueue(game: &mut Game) {
    if ram_word!(game, vram_read_queue_tail) as uint8 != 0 {
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
            .offset(0))
        .vram_target as *mut uint8)
            .offset(
                ram_word!(game, vram_read_queue_tail) as uint8 as ::core::ffi::c_int as isize,
            ) = 0 as uint8;
        let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        WriteReg(VMAIN as uint16, 0x80 as uint8, game);
        while (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
            .offset(v0 as isize))
        .vram_target
            != 0
        {
            WriteRegWord(
                VMADDL as uint16,
                (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
                    .offset(v0 as isize))
                .vram_target,
                game,
            );
            ReadRegWord(RDVRAML as uint16, game);
            WriteRegWord(
                DMAP1 as uint16,
                (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
                    .offset(v0 as isize))
                .dma_parameters,
                game,
            );
            WriteRegWord(
                A1T1L as uint16,
                (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
                    .offset(v0 as isize))
                .src
                .addr,
                game,
            );
            WriteRegWord(
                A1T1H as uint16,
                *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340)
                    as *mut VramReadQueueEnt)
                    .offset(v0 as isize))
                .src
                .addr as *mut uint8)
                    .offset(1) as *mut VoidP),
                game,
            );
            WriteRegWord(
                DAS1L as uint16,
                (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
                    .offset(v0 as isize))
                .size,
                game,
            );
            WriteRegWord(DAS10 as uint16, 0 as uint16, game);
            WriteRegWord(A2A1H as uint16, 0 as uint16, game);
            WriteReg(MDMAEN as uint16, 2 as uint8, game);
            v0 += 1;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x360) as *mut uint16 as *mut uint8)
            .offset(0) = 0 as uint8;
    }
}
pub unsafe fn HasQueuedMusic(game: &mut Game) -> uint8 {
    let mut v0: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
    while *((&raw mut game.g_ram as *mut uint8).offset(0x629) as *mut uint16)
        .offset((v0 >> 1 as ::core::ffi::c_int) as isize)
        == 0
    {
        v0 -= 2 as ::core::ffi::c_int;
        if v0 < 0i32 {
            return 0 as uint8;
        }
    }
    1 as uint8
}
pub unsafe fn HandleMusicQueue(game: &mut Game) {
    let ref mut fresh74 = *((&raw mut game.g_ram as *mut uint8).offset(0x63f) as *mut uint16);
    let fresh75 = *fresh74;
    *fresh74 = (*fresh74).wrapping_sub(1);
    let v0: bool = fresh75 == 1u16;
    if (ram_word!(game, music_timer) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        if !v0 {
            return;
        }
        if (ram_word!(game, music_entry) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
            != 0i32
        {
            set_ram_word!(game.music_data_index = ram_word!(game, music_entry) as uint8 as uint16);
            *(&raw mut game.g_ram as *mut uint8).offset(0x64c) =
                -(1 as ::core::ffi::c_int) as uint8;
            APU_UploadBank(
                Load24(
                    (game.g_rom.offset(crate::rtl::lorom_offset(0x8fe7e1)) as *const uint8
                        as *mut LongPtr as *mut uint8)
                        .offset(
                            ram_word!(game, music_entry)
                                as uint8 as ::core::ffi::c_int as isize,
                        ) as *mut LongPtr,
                ),
                game,
            );
            *(&raw mut game.g_ram as *mut uint8).offset(0x64c) = 0 as uint8;
            let v5: ::core::ffi::c_int =
                ram_word!(game, music_queue_read_pos) as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x619) as *mut uint16)
                .offset((v5 >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x629) as *mut uint16)
                .offset((v5 >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
            set_ram_word!(
                game.music_queue_read_pos =
                    (v5 + 2 as ::core::ffi::c_int & 0xe as ::core::ffi::c_int) as uint16
            );
            set_ram_word!(game.sound_handler_downtime = 8 as uint16);
            return;
        }
        let v1: uint8 = ((ram_word!(game, music_entry)) as u8) & (0x7fu8);
        set_ram_word!(game.music_track_index = (ram_word!(game, music_entry)) & (0x7fu16));
        RtlApuWrite(
            APUI00,
            ((ram_word!(game, music_entry)) as u8) & (0x7fu8),
            game,
        );
        *(&raw mut game.g_ram as *mut uint8).offset(0x64c) = v1;
        set_ram_word!(game.sound_handler_downtime = 8 as uint16);
        let v2: ::core::ffi::c_int = ram_word!(game, music_queue_read_pos) as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x619) as *mut uint16)
            .offset((v2 >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x629) as *mut uint16)
            .offset((v2 >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        set_ram_word!(
            game.music_queue_read_pos =
                (v2 + 2 as ::core::ffi::c_int & 0xe as ::core::ffi::c_int) as uint16
        );
    }
    if ram_word!(game, music_queue_read_pos) == ram_word!(game, music_queue_write_pos) {
        set_ram_word!(game.music_timer = 0 as uint16);
    } else {
        let v4: ::core::ffi::c_int =
            ram_word!(game, music_queue_read_pos) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        set_ram_word!(
            game.music_entry = *((&raw mut game.g_ram as *mut uint8).offset(0x619) as *mut uint16)
                .offset(v4 as isize)
        );
        set_ram_word!(
            game.music_timer = *((&raw mut game.g_ram as *mut uint8).offset(0x629) as *mut uint16)
                .offset(v4 as isize)
        );
    };
}
pub unsafe fn QueueMusic_Delayed8(a: uint16, game: &mut Game) {
    let mut v1: int8 = 0;
    if (ram_word!(game, game_state) as ::core::ffi::c_int)
        < kGameState_40_TransitionToDemo as ::core::ffi::c_int
        && ram_word!(game, music_queue_write_pos) as ::core::ffi::c_int + 2 as ::core::ffi::c_int
            & 0xe as ::core::ffi::c_int
            != ram_word!(game, music_queue_read_pos) as ::core::ffi::c_int
    {
        v1 = ram_word!(game, music_queue_write_pos) as int8;
        let v2: ::core::ffi::c_int =
            ram_word!(game, music_queue_write_pos) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x619) as *mut uint16).offset(v2 as isize) = a;
        *((&raw mut game.g_ram as *mut uint8).offset(0x629) as *mut uint16).offset(v2 as isize) =
            8 as uint16;
        set_ram_word!(
            game.music_queue_write_pos = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int
                & 0xe as ::core::ffi::c_int) as uint16
        );
    }
}
pub unsafe fn QueueMusic_DelayedY(a: uint16, j: uint16, game: &mut Game) {
    if ram_word!(game, game_state) < 0x28u16 {
        let v2: ::core::ffi::c_int = ram_word!(game, music_queue_write_pos) as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x619) as *mut uint16)
            .offset((v2 >> 1 as ::core::ffi::c_int) as isize) = a;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int;
        if j < 8u16 {
            v4 = 8 as ::core::ffi::c_int;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x629) as *mut uint16)
            .offset((v2 >> 1 as ::core::ffi::c_int) as isize) = v4 as uint16;
        set_ram_word!(
            game.music_queue_write_pos =
                (v2 + 2 as ::core::ffi::c_int & 0xe as ::core::ffi::c_int) as uint16
        );
    }
}
// Byte-backed queue layout from ram_map: three channels, sixteen slots each.
fn queue_sfx(channel: usize, sound: u8, max_queued: u8, game: &mut Game) {
    let read_pos = 0x643 + channel;
    let write_pos = 0x646 + channel;
    let limit_pos = 0x653 + channel;
    let buffer = 0x656 + 16 * channel;

    // This write also happens when capacity or the sound gates reject the request.
    game.g_ram[limit_pos] = max_queued;
    let queued = game.g_ram[write_pos].wrapping_sub(game.g_ram[read_pos]) & 15;
    if queued >= max_queued {
        return;
    }
    if game.debug_disable_sounds() != 0
        || game.game_state() >= 40
        || game.power_bomb_explosion_status() & 0x8000 != 0
    {
        return;
    }

    let write = game.g_ram[write_pos];
    let mut next = write.wrapping_add(1);
    if next >= 16 {
        next = 0;
    }
    // Preserve adjacent-RAM access for an out-of-range current write position.
    let slot = buffer + usize::from(write);
    if next == game.g_ram[read_pos] {
        // Lower sound numbers take priority when the ring is full.
        if sound < game.g_ram[slot] {
            game.g_ram[slot] = sound;
        }
    } else {
        game.g_ram[slot] = sound;
        game.g_ram[write_pos] = next;
        game.g_ram[buffer + usize::from(next)] = 0;
    }
}

pub fn QueueSfx1_Max15(sound: uint16, game: &mut Game) {
    queue_sfx(0, sound as u8, 15, game);
}
pub fn QueueSfx1_Max9(sound: uint16, game: &mut Game) {
    queue_sfx(0, sound as u8, 9, game);
}
pub fn QueueSfx1_Max3(sound: uint16, game: &mut Game) {
    queue_sfx(0, sound as u8, 3, game);
}
pub fn QueueSfx1_Max1(sound: uint16, game: &mut Game) {
    queue_sfx(0, sound as u8, 1, game);
}
pub fn QueueSfx1_Max6(sound: uint16, game: &mut Game) {
    queue_sfx(0, sound as u8, 6, game);
}
pub fn QueueSfx1_Internal(packed: uint16, game: &mut Game) {
    let [max_queued, sound] = packed.to_le_bytes();
    queue_sfx(0, sound, max_queued, game);
}
pub fn QueueSfx2_Max15(sound: uint16, game: &mut Game) {
    queue_sfx(1, sound as u8, 15, game);
}
pub fn QueueSfx2_Max9(sound: uint16, game: &mut Game) {
    queue_sfx(1, sound as u8, 9, game);
}
pub fn QueueSfx2_Max3(sound: uint16, game: &mut Game) {
    queue_sfx(1, sound as u8, 3, game);
}
pub fn QueueSfx2_Max1(sound: uint16, game: &mut Game) {
    queue_sfx(1, sound as u8, 1, game);
}
pub fn QueueSfx2_Max6(sound: uint16, game: &mut Game) {
    queue_sfx(1, sound as u8, 6, game);
}
pub fn QueueSfx2_Internal(packed: uint16, game: &mut Game) {
    let [max_queued, sound] = packed.to_le_bytes();
    queue_sfx(1, sound, max_queued, game);
}
pub fn QueueSfx3_Max15(sound: uint16, game: &mut Game) {
    queue_sfx(2, sound as u8, 15, game);
}
pub fn QueueSfx3_Max9(sound: uint16, game: &mut Game) {
    queue_sfx(2, sound as u8, 9, game);
}
pub fn QueueSfx3_Max3(sound: uint16, game: &mut Game) {
    queue_sfx(2, sound as u8, 3, game);
}
pub fn QueueSfx3_Max1(sound: uint16, game: &mut Game) {
    queue_sfx(2, sound as u8, 1, game);
}
pub fn QueueSfx3_Max6(sound: uint16, game: &mut Game) {
    queue_sfx(2, sound as u8, 6, game);
}
pub fn QueueSfx3_Internal(packed: uint16, game: &mut Game) {
    let [max_queued, sound] = packed.to_le_bytes();
    queue_sfx(2, sound, max_queued, game);
}
pub unsafe fn SetupDmaTransfer(p: *const ::core::ffi::c_void, game: &mut Game) {
    let s: *const StartDmaCopy = p as *const StartDmaCopy;
    let v2: ::core::ffi::c_int = (*s).chan as ::core::ffi::c_int * 16 as ::core::ffi::c_int;
    WriteRegWord(
        (v2 + DMAP0 as ::core::ffi::c_int) as SnesRegs as uint16,
        *(&raw const (*s).dmap as *mut uint16),
        game,
    );
    WriteRegWord(
        (v2 + A1T0L as ::core::ffi::c_int) as SnesRegs as uint16,
        *(&raw const (*s).a1.addr as *mut uint16),
        game,
    );
    WriteReg(
        (v2 + A1B0 as ::core::ffi::c_int) as SnesRegs as uint16,
        (*s).a1.bank,
        game,
    );
    WriteRegWord(
        (v2 + DAS0L as ::core::ffi::c_int) as SnesRegs as uint16,
        (*s).das,
        game,
    );
}
pub unsafe fn NmiUpdateIoRegisters(game: &mut Game) {
    WriteReg(NMITIMEN as u16, game.g_ram[0x84], game);
    WriteReg(INIDISP as u16, game.g_ram[0x51], game);
    WriteReg(OBSEL as u16, game.g_ram[0x52], game);
    WriteReg(BGMODE as u16, game.g_ram[0x55], game);
    WriteReg(MOSAIC as u16, game.g_ram[0x57], game);
    WriteReg(BG1SC as u16, game.g_ram[0x58], game);
    WriteReg(BG2SC as u16, game.g_ram[0x59], game);
    WriteReg(BG3SC as u16, game.g_ram[0x5a], game);
    WriteReg(BG4SC as u16, game.g_ram[0x5c], game);
    WriteReg(BG12NBA as u16, game.g_ram[0x5d], game);
    WriteReg(BG34NBA as u16, game.g_ram[0x5e], game);
    WriteReg(M7SEL as u16, game.g_ram[0x5f], game);
    WriteReg(W12SEL as u16, game.g_ram[0x60], game);
    WriteReg(W34SEL as u16, game.g_ram[0x61], game);
    WriteReg(WOBJSEL as u16, game.g_ram[0x62], game);
    WriteReg(WH0 as u16, game.g_ram[0x63], game);
    WriteReg(WH1 as u16, game.g_ram[0x64], game);
    WriteReg(WH2 as u16, game.g_ram[0x65], game);
    WriteReg(WH3 as u16, game.g_ram[0x66], game);
    WriteReg(WBGLOG as u16, game.g_ram[0x67], game);
    WriteReg(WOBJLOG as u16, game.g_ram[0x68], game);
    game.g_ram[0x6a] = game.g_ram[0x69];
    WriteReg(TM as u16, game.g_ram[0x69], game);
    WriteReg(TMW as u16, game.g_ram[0x6c], game);
    WriteReg(TS as u16, game.g_ram[0x6b], game);
    WriteReg(TSW as u16, game.g_ram[0x6d], game);
    WriteReg(CGWSEL as u16, game.g_ram[0x6f], game);
    WriteReg(CGADSUB as u16, game.g_ram[0x72], game);
    game.g_ram[0x70] = game.g_ram[0x6e];
    game.g_ram[0x73] = game.g_ram[0x71];
    WriteReg(COLDATA as u16, game.g_ram[0x74], game);
    WriteReg(COLDATA as u16, game.g_ram[0x75], game);
    WriteReg(COLDATA as u16, game.g_ram[0x76], game);
    WriteReg(SETINI as u16, game.g_ram[0x77], game);

    // Scroll and Mode 7 words latch low then high bytes through the same address.
    WriteReg(BG1HOFS as u16, game.g_ram[0xb1], game);
    WriteReg(BG1HOFS as u16, game.g_ram[0xb2], game);
    WriteReg(BG1VOFS as u16, game.g_ram[0xb3], game);
    WriteReg(BG1VOFS as u16, game.g_ram[0xb4], game);
    WriteReg(BG2HOFS as u16, game.g_ram[0xb5], game);
    WriteReg(BG2HOFS as u16, game.g_ram[0xb6], game);
    WriteReg(BG2VOFS as u16, game.g_ram[0xb7], game);
    WriteReg(BG2VOFS as u16, game.g_ram[0xb8], game);
    WriteReg(BG3HOFS as u16, game.g_ram[0xb9], game);
    WriteReg(BG3HOFS as u16, game.g_ram[0xba], game);
    WriteReg(BG3VOFS as u16, game.g_ram[0xbb], game);
    WriteReg(BG3VOFS as u16, game.g_ram[0xbc], game);
    WriteReg(BG4HOFS as u16, game.g_ram[0xbd], game);
    WriteReg(BG4HOFS as u16, game.g_ram[0xbe], game);
    WriteReg(BG4VOFS as u16, game.g_ram[0xbf], game);
    WriteReg(BG4VOFS as u16, game.g_ram[0xc0], game);
    game.g_ram[0x7ec] = game.g_ram[0x56];
    if game.g_ram[0x55] & 7 == 7 || game.g_ram[0x56] & 7 == 7 {
        WriteReg(M7A as u16, game.g_ram[0x78], game);
        WriteReg(M7A as u16, game.g_ram[0x79], game);
        WriteReg(M7B as u16, game.g_ram[0x7a], game);
        WriteReg(M7B as u16, game.g_ram[0x7b], game);
        WriteReg(M7C as u16, game.g_ram[0x7c], game);
        WriteReg(M7C as u16, game.g_ram[0x7d], game);
        WriteReg(M7D as u16, game.g_ram[0x7e], game);
        WriteReg(M7D as u16, game.g_ram[0x7f], game);
        WriteReg(M7X as u16, game.g_ram[0x80], game);
        WriteReg(M7X as u16, game.g_ram[0x81], game);
        WriteReg(M7Y as u16, game.g_ram[0x82], game);
        WriteReg(M7Y as u16, game.g_ram[0x83], game);
    }
}
pub unsafe fn NmiUpdatePalettesAndOam(game: &mut Game) {
    // Channel 0 uploads the OAM entries and their high table from RAM.
    WriteRegWord(DMAP0 as u16, 0x0400, game);
    WriteRegWord(A1T0L as u16, 0x0370, game);
    WriteReg(A1B0 as u16, 0, game);
    WriteRegWord(DAS0L as u16, 0x0220, game);
    WriteRegWord(OAMADDL as u16, 0, game);

    // Channel 1 uploads palette_buffer to CGRAM.
    WriteRegWord(DMAP1 as u16, 0x2200, game);
    WriteRegWord(A1T1L as u16, 0xc000, game);
    WriteReg(A1B1 as u16, 0x7e, game);
    WriteRegWord(DAS1L as u16, 0x0200, game);
    WriteReg(CGADD as u16, 0, game);
    WriteReg(MDMAEN as u16, 3, game);
}
unsafe fn transfer_samus_tile_half(descriptor_addr: u16, vram_addr: u16, game: &mut Game) {
    // Keep the diagnostic and failure flag for low-address ROM mirrors.
    let _ = RomPtr_92(descriptor_addr, game);
    let descriptor = 0x928000 | u32::from(descriptor_addr & 0x7fff);
    // Packed fields: source address at 0, bank at 2, and byte counts at 3 and 5.
    WriteRegWord(VMADDL as u16, vram_addr, game);
    WriteRegWord(DMAP1 as u16, 0x1801, game);
    WriteRegWord(A1T1L as u16, game.read_rom_word(descriptor, 0), game);
    WriteReg(A1B1 as u16, game.read_rom_byte(descriptor, 2), game);
    // A zero first count still runs a 65,536-byte DMA.
    WriteRegWord(DAS1L as u16, game.read_rom_word(descriptor, 3), game);
    WriteReg(MDMAEN as u16, 2, game);

    // Prepare the second destination/source even when its count is zero.
    WriteRegWord(VMADDL as u16, vram_addr.wrapping_add(0x100), game);
    let second_source = game
        .read_rom_word(descriptor, 0)
        .wrapping_add(game.read_rom_word(descriptor, 3));
    WriteRegWord(A1T1L as u16, second_source, game);
    if game.read_rom_word(descriptor, 5) != 0 {
        WriteRegWord(DAS1L as u16, game.read_rom_word(descriptor, 5), game);
        WriteReg(MDMAEN as u16, 2, game);
    }
}

pub unsafe fn NmiTransferSamusToVram(game: &mut Game) {
    WriteReg(VMAIN as u16, 0x80, game);
    if game.g_ram[0x71d] != 0 {
        let top_half_src = u16::from_le_bytes([game.g_ram[0x71f], game.g_ram[0x720]]);
        transfer_samus_tile_half(top_half_src, 0x6000, game);
    }
    if game.g_ram[0x71e] != 0 {
        let bottom_half_src = u16::from_le_bytes([game.g_ram[0x721], game.g_ram[0x722]]);
        transfer_samus_tile_half(bottom_half_src, 0x6080, game);
    }
}
pub unsafe fn NmiProcessAnimtilesVramTransfers(game: &mut Game) {
    // Only bit 15 of animtiles_enable_flag enables transfers.
    if game.g_ram[0x1ef2] & 0x80 == 0 {
        return;
    }
    for slot in (0..6).rev() {
        let offset = slot * 2;
        let id = u16::from_le_bytes([game.g_ram[0x1ef5 + offset], game.g_ram[0x1ef6 + offset]]);
        if id == 0 {
            continue;
        }
        let source_index = 0x1f25 + offset;
        let source = u16::from_le_bytes([game.g_ram[source_index], game.g_ram[source_index + 1]]);
        if source == 0 {
            continue;
        }

        WriteRegWord(A1T0L as u16, source, game);
        WriteReg(A1B0 as u16, 0x87, game);
        WriteRegWord(DMAP0 as u16, 0x1801, game);
        let size = u16::from_le_bytes([game.g_ram[0x1f31 + offset], game.g_ram[0x1f32 + offset]]);
        WriteRegWord(DAS0L as u16, size, game);
        let vram_addr =
            u16::from_le_bytes([game.g_ram[0x1f3d + offset], game.g_ram[0x1f3e + offset]]);
        WriteRegWord(VMADDL as u16, vram_addr, game);
        WriteReg(VMAIN as u16, 0x80, game);
        WriteReg(MDMAEN as u16, 1, game);
        // Consume the request only after its transfer completes.
        game.g_ram[source_index..source_index + 2].fill(0);
    }
}
pub unsafe fn ReadJoypadInputs(game: &mut Game) {
    let RegWord: uint16 = ReadRegWord(JOY1L as uint16, game);
    set_ram_word!(game.joypad1_lastkeys = RegWord);
    set_ram_word!(
        game.joypad1_newkeys = (RegWord as ::core::ffi::c_int
            & (ram_word!(game, joypad1_prev) as ::core::ffi::c_int ^ RegWord as ::core::ffi::c_int))
            as uint16
    );
    set_ram_word!(
        game.joypad1_newkeys2_UNUSED = (RegWord as ::core::ffi::c_int
            & (ram_word!(game, joypad1_prev) as ::core::ffi::c_int ^ RegWord as ::core::ffi::c_int))
            as uint16
    );
    if RegWord != 0u16 && RegWord == ram_word!(game, joypad1_prev) {
        let ref mut fresh80 = *((&raw mut game.g_ram as *mut uint8).offset(0xa3) as *mut uint16);
        *fresh80 = (*fresh80).wrapping_sub(1);
        if *fresh80 == 0 {
            set_ram_word!(game.joypad1_newkeys2_UNUSED = ram_word!(game, joypad1_lastkeys));
            set_ram_word!(game.joypad1_keyrepeat_ctr = ram_word!(game, joypad_ctr_repeat_next));
        }
    } else {
        set_ram_word!(game.joypad1_keyrepeat_ctr = ram_word!(game, joypad_ctr_repeat_first));
    }
    set_ram_word!(game.joypad1_prev = ram_word!(game, joypad1_lastkeys));
    if ram_word!(game, enable_debug) != 0 {
        let v1: uint16 = ReadRegWord(JOY2L as uint16, game);
        set_ram_word!(game.joypad2_last = v1);
        set_ram_word!(
            game.joypad2_new_keys = (v1 as ::core::ffi::c_int
                & (ram_word!(game, joypad2_prev) as ::core::ffi::c_int ^ v1 as ::core::ffi::c_int))
                as uint16
        );
        set_ram_word!(
            game.joypad2_newkeys2 = (v1 as ::core::ffi::c_int
                & (ram_word!(game, joypad2_prev) as ::core::ffi::c_int ^ v1 as ::core::ffi::c_int))
                as uint16
        );
        if v1 != 0u16 && v1 == ram_word!(game, joypad2_prev) {
            let ref mut fresh81 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xa5) as *mut uint16);
            *fresh81 = (*fresh81).wrapping_sub(1);
            if *fresh81 == 0 {
                set_ram_word!(game.joypad2_newkeys2 = ram_word!(game, joypad2_last));
                set_ram_word!(game.joypad2_keyrepeat_ctr = ram_word!(game, joypad_ctr_repeat_next));
            }
        } else {
            set_ram_word!(game.joypad2_keyrepeat_ctr = ram_word!(game, joypad_ctr_repeat_first));
        }
        set_ram_word!(game.joypad2_prev = ram_word!(game, joypad2_last));
        if ram_word!(game, is_uploading_apu) != 0u16
            || ram_word!(game, joypad1_lastkeys) as ::core::ffi::c_int
                != kButton_Select as ::core::ffi::c_int
                    | kButton_Start as ::core::ffi::c_int
                    | kButton_L as ::core::ffi::c_int
                    | kButton_R as ::core::ffi::c_int
        {
            if ram_word!(game, enable_debug) != 0 {
                set_ram_word!(game.joypad_dbg_1 = 0 as uint16);
                set_ram_word!(game.joypad_dbg_2 = 0 as uint16);
                if (ram_word!(game, joypad_dbg_flags) as ::core::ffi::c_int
                    & 0x4000 as ::core::ffi::c_int)
                    == 0i32
                {
                    if ram_word!(game, joypad1_lastkeys) as ::core::ffi::c_int
                        & (kButton_Select as ::core::ffi::c_int | kButton_L as ::core::ffi::c_int)
                        == kButton_Select as ::core::ffi::c_int | kButton_L as ::core::ffi::c_int
                    {
                        set_ram_word!(game.joypad_dbg_1 = ram_word!(game, joypad1_newkeys));
                        set_ram_word!(game.joypad1_lastkeys = 0 as uint16);
                        set_ram_word!(game.joypad1_newkeys = 0 as uint16);
                    }
                    if ram_word!(game, joypad1_lastkeys) as ::core::ffi::c_int
                        & (kButton_Select as ::core::ffi::c_int | kButton_R as ::core::ffi::c_int)
                        == kButton_Select as ::core::ffi::c_int | kButton_R as ::core::ffi::c_int
                    {
                        set_ram_word!(game.joypad_dbg_2 = ram_word!(game, joypad1_newkeys));
                        set_ram_word!(game.joypad1_lastkeys = 0 as uint16);
                        set_ram_word!(game.joypad1_newkeys = 0 as uint16);
                    }
                    if (ram_word!(game, joypad_dbg_2) as ::core::ffi::c_int
                        & 0x80 as ::core::ffi::c_int)
                        != 0i32
                    {
                        let ref mut fresh82 =
                            *((&raw mut game.g_ram as *mut uint8).offset(0x84) as *mut uint16);
                        *fresh82 = (*fresh82) ^ (0x30u16);
                    }
                    if (ram_word!(game, joypad_dbg_2) as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int)
                        != 0i32
                    {
                        let v2: bool = (!(ram_word!(game, joypad_dbg_flags) as ::core::ffi::c_int)
                            & 0x8000 as ::core::ffi::c_int)
                            != 0i32;
                        let ref mut fresh83 =
                            *((&raw mut game.g_ram as *mut uint8).offset(0x5cf) as *mut uint16);
                        *fresh83 = (*fresh83) ^ (0x8000u16);
                        if v2 {
                            set_ram_word!(
                                game.joypad_dbg_missiles_swap = ram_word!(game, samus_missiles)
                            );
                            set_ram_word!(
                                game.joypad_dbg_super_missiles_swap =
                                    ram_word!(game, samus_super_missiles)
                            );
                            set_ram_word!(
                                game.joypad_dbg_power_bombs_swap =
                                    ram_word!(game, samus_power_bombs)
                            );
                            set_ram_word!(game.samus_missiles = 0 as uint16);
                            set_ram_word!(game.samus_super_missiles = 0 as uint16);
                            set_ram_word!(game.samus_power_bombs = 0 as uint16);
                        } else {
                            set_ram_word!(
                                game.samus_missiles = ram_word!(game, joypad_dbg_missiles_swap)
                            );
                            set_ram_word!(
                                game.samus_super_missiles =
                                    ram_word!(game, joypad_dbg_super_missiles_swap)
                            );
                            set_ram_word!(
                                game.samus_power_bombs =
                                    ram_word!(game, joypad_dbg_power_bombs_swap)
                            );
                        }
                    }
                    if (ram_word!(game, joypad_dbg_2) as ::core::ffi::c_int
                        & 0x40 as ::core::ffi::c_int)
                        != 0i32
                    {
                        let ref mut fresh84 =
                            *((&raw mut game.g_ram as *mut uint8).offset(0x5cf) as *mut uint16);
                        *fresh84 = (*fresh84) ^ (0x2000u16);
                    }
                }
            } else {
                set_ram_word!(game.joypad_dbg_1 = 0 as uint16);
                set_ram_word!(game.joypad_dbg_2 = 0 as uint16);
                let ref mut fresh85 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x8d) as *mut uint16);
                *fresh85 = (*fresh85) & (0x10u16);
                let ref mut fresh86 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x91) as *mut uint16);
                *fresh86 = (*fresh86) & (0x10u16);
            }
        } else {
            set_ram_word!(game.debug_disable_sounds = 0 as uint16);
            SoftReset(game);
        }
    }
}
pub unsafe fn Vector_NMI(game: &mut Game) {
    const BG_MODE: usize = 0x55;
    const BG_MODE_FAKE: usize = 0x56;
    const HDMA_ENABLE: usize = 0x85;
    const WAITING_FOR_NMI: usize = 0x5b4;
    const FRAME_COUNTER_BYTE: usize = 0x5b5;
    const FRAME_COUNTER_WORD: usize = 0x5b6;
    const FRAME_COUNTER_INCLUDING_LAG: usize = 0x5b8;
    const FRAMES_MISSED: usize = 0x5ba;
    const FRAMES_MISSED_MAX: usize = 0x5bb;

    if game.g_ram[WAITING_FOR_NMI] != 0 {
        NmiUpdatePalettesAndOam(game);
        NmiTransferSamusToVram(game);
        NmiProcessAnimtilesVramTransfers(game);
        NmiUpdateIoRegisters(game);
        for slot in 0..6 {
            let offset = slot * 2;
            let channels =
                u16::from_le_bytes([game.g_ram[0x18b4 + offset], game.g_ram[0x18b5 + offset]]);
            if channels != 0 {
                // The bank-slot word stores the DMA register offset in its low byte.
                let bank_slot = u16::from(game.g_ram[0x18c0 + offset]);
                let table_pointer =
                    u16::from_le_bytes([game.g_ram[0x18d8 + offset], game.g_ram[0x18d9 + offset]]);
                WriteRegWord(A1T0L as u16 + bank_slot, table_pointer, game);
            }
        }
        if game.g_ram[BG_MODE] == 7 || game.g_ram[BG_MODE_FAKE] == 7 {
            NMI_ProcessMode7Queue(game);
        }
        NMI_ProcessVramWriteQueue(game);
        NMI_ProcessVramReadQueue(game);
        WriteReg(HDMAEN as u16, game.g_ram[HDMA_ENABLE], game);
        game.g_ram[WAITING_FOR_NMI] = 0;
        game.g_ram[FRAMES_MISSED] = 0;
        game.g_ram[FRAME_COUNTER_BYTE] = game.g_ram[FRAME_COUNTER_BYTE].wrapping_add(1);
        let frame_counter = u16::from_le_bytes([
            game.g_ram[FRAME_COUNTER_WORD],
            game.g_ram[FRAME_COUNTER_WORD + 1],
        ])
        .wrapping_add(1);
        game.g_ram[FRAME_COUNTER_WORD..FRAME_COUNTER_WORD + 2]
            .copy_from_slice(&frame_counter.to_le_bytes());
    } else {
        game.g_ram[FRAMES_MISSED] = game.g_ram[FRAMES_MISSED].wrapping_add(1);
        if game.g_ram[FRAMES_MISSED] >= game.g_ram[FRAMES_MISSED_MAX] {
            game.g_ram[FRAMES_MISSED_MAX] = game.g_ram[FRAMES_MISSED];
        }
    }
    let frame_counter = u16::from_le_bytes([
        game.g_ram[FRAME_COUNTER_INCLUDING_LAG],
        game.g_ram[FRAME_COUNTER_INCLUDING_LAG + 1],
    ])
    .wrapping_add(1);
    game.g_ram[FRAME_COUNTER_INCLUDING_LAG..FRAME_COUNTER_INCLUDING_LAG + 2]
        .copy_from_slice(&frame_counter.to_le_bytes());
}
pub unsafe fn CopyToVramNow(vram_dst: uint16, src: uint32, size: uint16, game: &mut Game) {
    WriteReg(INIDISP as uint16, 0x80 as uint8, game);
    WriteRegWord(VMADDL as uint16, vram_dst, game);
    WriteRegWord(DMAP1 as uint16, 0x1801 as uint16, game);
    WriteRegWord(A1T1L as uint16, src as uint16, game);
    WriteReg(
        A1B1 as uint16,
        (src >> 16 as ::core::ffi::c_int) as uint8,
        game,
    );
    WriteRegWord(DAS1L as uint16, size, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(INIDISP as uint16, 0xf as uint8, game);
}
pub unsafe fn IrqHandler_SetResult(a: uint16, y: uint16, x: uint16, game: &mut Game) {
    set_ram_word!(game.cur_irq_handler = a);
    WriteRegWord(VTIMEL as uint16, y, game);
    WriteRegWord(HTIMEL as uint16, x, game);
}
pub unsafe fn IrqHandler_0_Nothing(game: &mut Game) {
    let a: uint16 = ram_word!(game, irqhandler_next_handler);
    set_ram_word!(game.irqhandler_next_handler = 0 as uint16);
    IrqHandler_SetResult(a, 0 as uint16, 0 as uint16, game);
}
pub unsafe fn IrqHandler_2_DisableIRQ(game: &mut Game) {
    let ref mut fresh76 = *((&raw mut game.g_ram as *mut uint8).offset(0x84) as *mut uint16);
    *fresh76 = (*fresh76 as ::core::ffi::c_int & !(0x30 as ::core::ffi::c_int)) as uint16;
    IrqHandler_SetResult(0 as uint16, 0 as uint16, 0 as uint16, game);
}
pub unsafe fn IrqHandler_4_Main_BeginHudDraw(game: &mut Game) {
    WriteReg(BG3SC as uint16, 0x5a as uint8, game);
    WriteReg(CGWSEL as uint16, 0 as uint8, game);
    WriteReg(CGADSUB as uint16, 0 as uint8, game);
    WriteReg(TM as uint16, 4 as uint8, game);
    IrqHandler_SetResult(6 as uint16, 31 as uint16, 152 as uint16, game);
}
pub unsafe fn IrqHandler_6_Main_EndHudDraw(game: &mut Game) {
    WriteReg(
        CGWSEL as uint16,
        *(&raw mut game.g_ram as *mut uint8).offset(0x70),
        game,
    );
    WriteReg(
        CGADSUB as uint16,
        *(&raw mut game.g_ram as *mut uint8).offset(0x73),
        game,
    );
    WriteReg(
        BG3SC as uint16,
        *(&raw mut game.g_ram as *mut uint8).offset(0x5b),
        game,
    );
    WriteReg(
        TM as uint16,
        *(&raw mut game.g_ram as *mut uint8).offset(0x6a),
        game,
    );
    let a: uint16 = ram_word!(game, irqhandler_next_handler);
    set_ram_word!(game.irqhandler_next_handler = 0 as uint16);
    IrqHandler_SetResult(
        (if a != 0u16 {
            a as ::core::ffi::c_int
        } else {
            4 as ::core::ffi::c_int
        }) as uint16,
        0 as uint16,
        152 as uint16,
        game,
    );
}
pub unsafe fn IrqHandler_8_StartOfDoor_BeginHud(game: &mut Game) {
    WriteReg(BG3SC as uint16, 0x5a as uint8, game);
    WriteReg(TM as uint16, 4 as uint8, game);
    WriteReg(CGWSEL as uint16, 0 as uint8, game);
    WriteReg(CGADSUB as uint16, 0 as uint8, game);
    IrqHandler_SetResult(10 as uint16, 31 as uint16, 152 as uint16, game);
}
pub unsafe fn IrqHandler_10_StartOfDoor_EndHud(game: &mut Game) {
    let mut v0: uint8 = 0;
    if ((ram_word!(game, previous_cre_bitset) as ::core::ffi::c_int
        | ram_word!(game, cre_bitset) as ::core::ffi::c_int)
        & 1 as ::core::ffi::c_int)
        != 0i32
    {
        v0 = 16 as uint8;
    } else {
        v0 = 17 as uint8;
    }
    WriteReg(TM as uint16, v0, game);
    let a: uint16 = ram_word!(game, irqhandler_next_handler);
    set_ram_word!(game.irqhandler_next_handler = 0 as uint16);
    IrqHandler_SetResult(
        (if a != 0u16 {
            a as ::core::ffi::c_int
        } else {
            8 as ::core::ffi::c_int
        }) as uint16,
        0 as uint16,
        152 as uint16,
        game,
    );
}
pub unsafe fn IrqHandler_12_Draygon_BeginHud(game: &mut Game) {
    WriteReg(TM as uint16, 4 as uint8, game);
    WriteReg(CGWSEL as uint16, 0 as uint8, game);
    WriteReg(CGADSUB as uint16, 0 as uint8, game);
    IrqHandler_SetResult(14 as uint16, 31 as uint16, 152 as uint16, game);
}
pub unsafe fn IrqHandler_14_Draygon_EndHud(game: &mut Game) {
    WriteReg(
        BG3SC as uint16,
        *(&raw mut game.g_ram as *mut uint8).offset(0x5b),
        game,
    );
    WriteReg(
        CGWSEL as uint16,
        *(&raw mut game.g_ram as *mut uint8).offset(0x70),
        game,
    );
    WriteReg(
        CGADSUB as uint16,
        *(&raw mut game.g_ram as *mut uint8).offset(0x73),
        game,
    );
    let a: uint16 = ram_word!(game, irqhandler_next_handler);
    set_ram_word!(game.irqhandler_next_handler = 0 as uint16);
    IrqHandler_SetResult(
        (if a != 0u16 {
            a as ::core::ffi::c_int
        } else {
            12 as ::core::ffi::c_int
        }) as uint16,
        0 as uint16,
        152 as uint16,
        game,
    );
}
pub unsafe fn IrqHandler_16_VerticalDoor_BeginHud(game: &mut Game) {
    WriteReg(TM as uint16, 4 as uint8, game);
    WriteReg(CGWSEL as uint16, 0 as uint8, game);
    WriteReg(CGADSUB as uint16, 0 as uint8, game);
    IrqHandler_SetResult(18 as uint16, 31 as uint16, 152 as uint16, game);
}
pub unsafe fn IrqHandler_18_VerticalDoor_EndHud(game: &mut Game) {
    let mut v0: uint8 = 0;
    if ((ram_word!(game, previous_cre_bitset) as ::core::ffi::c_int
        | ram_word!(game, cre_bitset) as ::core::ffi::c_int)
        & 1 as ::core::ffi::c_int)
        != 0i32
    {
        v0 = 16 as uint8;
    } else {
        v0 = 17 as uint8;
    }
    WriteReg(TM as uint16, v0, game);
    WriteReg(CGWSEL as uint16, 0 as uint8, game);
    WriteReg(CGADSUB as uint16, 0 as uint8, game);
    if (ram_word!(game, door_transition_flag) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        Irq_FollowDoorTransition(game);
    }
    IrqHandler_SetResult(20 as uint16, 216 as uint16, 152 as uint16, game);
}
pub unsafe fn IrqHandler_20_VerticalDoor_EndDraw(game: &mut Game) {
    let a: uint16 = ram_word!(game, irqhandler_next_handler);
    set_ram_word!(game.irqhandler_next_handler = 0 as uint16);
    IrqHandler_SetResult(
        (if a != 0u16 {
            a as ::core::ffi::c_int
        } else {
            16 as ::core::ffi::c_int
        }) as uint16,
        0 as uint16,
        152 as uint16,
        game,
    );
}
pub unsafe fn IrqHandler_22_HorizDoor_BeginHud(game: &mut Game) {
    WriteReg(TM as uint16, 4 as uint8, game);
    WriteReg(CGWSEL as uint16, 0 as uint8, game);
    WriteReg(CGADSUB as uint16, 0 as uint8, game);
    IrqHandler_SetResult(24 as uint16, 31 as uint16, 152 as uint16, game);
}
pub unsafe fn IrqHandler_24_HorizDoor_EndHud(game: &mut Game) {
    let mut v0: uint8 = 0;
    if ((ram_word!(game, previous_cre_bitset) as ::core::ffi::c_int
        | ram_word!(game, cre_bitset) as ::core::ffi::c_int)
        & 1 as ::core::ffi::c_int)
        != 0i32
    {
        v0 = 16 as uint8;
    } else {
        v0 = 17 as uint8;
    }
    WriteReg(TM as uint16, v0, game);
    WriteReg(CGWSEL as uint16, 0 as uint8, game);
    WriteReg(CGADSUB as uint16, 0 as uint8, game);
    if (ram_word!(game, door_transition_flag) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        Irq_FollowDoorTransition(game);
    }
    IrqHandler_SetResult(26 as uint16, 160 as uint16, 152 as uint16, game);
}
pub unsafe fn IrqHandler_26_HorizDoor_EndDraw(game: &mut Game) {
    let a: uint16 = ram_word!(game, irqhandler_next_handler);
    set_ram_word!(game.irqhandler_next_handler = 0 as uint16);
    IrqHandler_SetResult(
        (if a != 0u16 {
            a as ::core::ffi::c_int
        } else {
            22 as ::core::ffi::c_int
        }) as uint16,
        0 as uint16,
        152 as uint16,
        game,
    );
}
pub unsafe fn EnableIrqInterrupts(game: &mut Game) {
    WriteRegWord(VTIMEL as uint16, 0 as uint16, game);
    WriteRegWord(HTIMEL as uint16, 152 as uint16, game);
    let ref mut fresh64 = *((&raw mut game.g_ram as *mut uint8).offset(0x84) as *mut uint16);
    *fresh64 = (*fresh64) | (0x30u16);
}
pub unsafe fn EnableIrqInterruptsNow(game: &mut Game) {
    WriteRegWord(VTIMEL as uint16, 0 as uint16, game);
    WriteRegWord(HTIMEL as uint16, 152 as uint16, game);
    let ref mut fresh65 = *((&raw mut game.g_ram as *mut uint8).offset(0x84) as *mut uint16);
    *fresh65 = (*fresh65) | (0x30u16);
    WriteReg(
        NMITIMEN as uint16,
        *(&raw mut game.g_ram as *mut uint8).offset(0x84),
        game,
    );
}
pub unsafe fn DisableIrqInterrupts(game: &mut Game) {
    let ref mut fresh41 = *((&raw mut game.g_ram as *mut uint8).offset(0x84) as *mut uint16);
    *fresh41 = (*fresh41 as ::core::ffi::c_int & !(0x30 as ::core::ffi::c_int)) as uint16;
}
static kIrqHandlers: [Func_V; 14] = [
    IrqHandler_0_Nothing,
    IrqHandler_2_DisableIRQ,
    IrqHandler_4_Main_BeginHudDraw,
    IrqHandler_6_Main_EndHudDraw,
    IrqHandler_8_StartOfDoor_BeginHud,
    IrqHandler_10_StartOfDoor_EndHud,
    IrqHandler_12_Draygon_BeginHud,
    IrqHandler_14_Draygon_EndHud,
    IrqHandler_16_VerticalDoor_BeginHud,
    IrqHandler_18_VerticalDoor_EndHud,
    IrqHandler_20_VerticalDoor_EndDraw,
    IrqHandler_22_HorizDoor_BeginHud,
    IrqHandler_24_HorizDoor_EndHud,
    IrqHandler_26_HorizDoor_EndDraw,
];
pub unsafe fn Vector_IRQ(game: &mut Game) {
    kIrqHandlers[(ram_word!(game, cur_irq_handler) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
        as usize](game);
}
static kHudTilemaps_Row1to3: [uint16; 96] = [
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c12, 0x2c12, 0x2c23, 0x2c12, 0x2c12, 0x2c1e, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2822, 0x2822,
    0x2823, 0x2813, 0x2c14, 0x2c1e, 0x2c0f, 0x2c0b, 0x2c0c, 0x2c0d, 0x2c32, 0x2c0f, 0x2c09, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c12, 0x2c12, 0xa824, 0x2815, 0x2c16, 0x2c1e,
];
static kHudTilemaps_Missiles: [uint16; 22] = [
    0x344b, 0x3449, 0x744b, 0x344c, 0x344a, 0x744c, 0x3434, 0x7434, 0x3435, 0x7435, 0x3436, 0x7436,
    0x3437, 0x7437, 0x3438, 0x7438, 0x3439, 0x7439, 0x343a, 0x743a, 0x343b, 0x743b,
];
pub unsafe fn AddMissilesToHudTilemap(game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(10)
        as ::core::ffi::c_int
        & 0x3ff as ::core::ffi::c_int)
        == 15i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(10) =
            kHudTilemaps_Missiles[0 as ::core::ffi::c_int as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(11) =
            kHudTilemaps_Missiles[1 as ::core::ffi::c_int as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(12) =
            kHudTilemaps_Missiles[2 as ::core::ffi::c_int as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(42) =
            kHudTilemaps_Missiles[3 as ::core::ffi::c_int as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(43) =
            kHudTilemaps_Missiles[4 as ::core::ffi::c_int as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(44) =
            kHudTilemaps_Missiles[5 as ::core::ffi::c_int as usize];
    }
}
pub unsafe fn AddSuperMissilesToHudTilemap(game: &mut Game) {
    AddToTilemapInner(
        0x1c as uint16,
        RomPtr_80(
            (addr_kHudTilemaps_Missiles as ::core::ffi::c_int + 12 as ::core::ffi::c_int)
                as uint16_t,
            game,
        ) as *const uint16,
        game,
    );
}
pub unsafe fn AddPowerBombsToHudTilemap(game: &mut Game) {
    AddToTilemapInner(
        0x22 as uint16,
        RomPtr_80(
            (addr_kHudTilemaps_Missiles as ::core::ffi::c_int + 20 as ::core::ffi::c_int)
                as uint16_t,
            game,
        ) as *const uint16,
        game,
    );
}
pub unsafe fn AddGrappleToHudTilemap(game: &mut Game) {
    AddToTilemapInner(
        0x28 as uint16,
        RomPtr_80(
            (addr_kHudTilemaps_Missiles as ::core::ffi::c_int + 28 as ::core::ffi::c_int)
                as uint16_t,
            game,
        ) as *const uint16,
        game,
    );
}
pub unsafe fn AddXrayToHudTilemap(game: &mut Game) {
    AddToTilemapInner(
        0x2e as uint16,
        RomPtr_80(
            (addr_kHudTilemaps_Missiles as ::core::ffi::c_int + 36 as ::core::ffi::c_int)
                as uint16_t,
            game,
        ) as *const uint16,
        game,
    );
}
pub unsafe fn AddToTilemapInner(k: uint16, j: *const uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(v2 as isize)
        as ::core::ffi::c_int
        & 0x3ff as ::core::ffi::c_int)
        == 15i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(v2 as isize) =
            *j.offset(0);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize) = *j.offset(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
            .offset((v2 + 32 as ::core::ffi::c_int) as isize) = *j.offset(2);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
            .offset((v2 + 33 as ::core::ffi::c_int) as isize) = *j.offset(3);
    }
}
pub unsafe fn InitializeHud(game: &mut Game) {
    WriteRegWord(VMADDL as uint16, addr_unk_605800 as uint16, game);
    WriteRegWord(VMAIN as uint16, 0x80 as uint16, game);
    static unk_809A8F: StartDmaCopy = StartDmaCopy::vram_words(1, 0x80988b, 0x40);
    SetupDmaTransfer(&raw const unk_809A8F as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 192i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) =
            kHudTilemaps_Row1to3[(i >> 1 as ::core::ffi::c_int) as usize];
        i += 2 as ::core::ffi::c_int;
    }
    if (ram_word!(game, equipped_items) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        AddXrayToHudTilemap(game);
    }
    if (ram_word!(game, equipped_items) as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int)
        != 0i32
    {
        AddGrappleToHudTilemap(game);
    }
    if ram_word!(game, samus_max_missiles) != 0 {
        AddMissilesToHudTilemap(game);
    }
    if ram_word!(game, samus_max_super_missiles) != 0 {
        AddSuperMissilesToHudTilemap(game);
    }
    if ram_word!(game, samus_max_power_bombs) != 0 {
        AddPowerBombsToHudTilemap(game);
    }
    set_ram_word!(game.samus_prev_health = 0 as uint16);
    set_ram_word!(game.samus_prev_missiles = 0 as uint16);
    set_ram_word!(game.samus_prev_super_missiles = 0 as uint16);
    set_ram_word!(game.samus_prev_power_bombs = 0 as uint16);
    set_ram_word!(game.samus_prev_hud_item_index = 0 as uint16);
    InitializeMiniMapBroken();
    if ram_word!(game, samus_max_missiles) != 0 {
        DrawThreeHudDigits(
            addrl_kDigitTilesetsWeapon,
            ram_word!(game, samus_missiles),
            0x94 as uint16,
            game,
        );
    }
    if ram_word!(game, samus_max_super_missiles) != 0 {
        DrawTwoHudDigits(
            addrl_kDigitTilesetsWeapon,
            ram_word!(game, samus_super_missiles),
            0x9c as uint16,
            game,
        );
    }
    if ram_word!(game, samus_max_power_bombs) != 0 {
        DrawTwoHudDigits(
            addrl_kDigitTilesetsWeapon,
            ram_word!(game, samus_power_bombs),
            0xa2 as uint16,
            game,
        );
    }
    ToggleHudItemHighlight(ram_word!(game, hud_item_index), 0x1000 as uint16, game);
    ToggleHudItemHighlight(
        ram_word!(game, samus_prev_hud_item_index),
        0x1400 as uint16,
        game,
    );
    HandleHudTilemap(game);
}
static kEnergyTankIconTilemapOffsets: [uint16; 14] = [
    0x42, 0x44, 0x46, 0x48, 0x4a, 0x4c, 0x4e, 2, 4, 6, 8, 0xa, 0xc, 0xe,
];
pub unsafe fn HandleHudTilemap(game: &mut Game) {
    if ram_word!(game, reserve_health_mode) == 1u16 {
        let mut v1: *const uint16 =
            RomPtr_80(addr_kHudTilemaps_AutoReserve as uint16_t, game) as *const uint16;
        if ram_word!(game, samus_reserve_health) == 0 {
            v1 = v1.offset(6);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(8) =
            *v1.offset(0);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(9) =
            *v1.offset(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(40) =
            *v1.offset(2);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(41) =
            *v1.offset(3);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(72) =
            *v1.offset(4);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(73) =
            *v1.offset(5);
    }
    if ram_word!(game, samus_health) != ram_word!(game, samus_prev_health) {
        set_ram_word!(game.samus_prev_health = ram_word!(game, samus_health));
        let mut r20: uint16 = SnesDivide(ram_word!(game, samus_health), 100 as uint8);
        let r18: uint16 = SnesModulus(ram_word!(game, samus_health), 100 as uint8);
        let mut v2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut n: ::core::ffi::c_int = SnesDivide(ram_word!(game, samus_max_health), 100 as uint8)
            as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int;
        loop {
            n -= 1;
            if n == 0 {
                break;
            }
            let mut v3: uint16 = 13360 as uint16;
            if r20 != 0 {
                r20 = r20.wrapping_sub(1);
                v3 = 10289 as uint16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(
                (kEnergyTankIconTilemapOffsets[(v2 >> 1 as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            ) = v3;
            v2 += 2 as ::core::ffi::c_int;
            if (((v2 - 28 as ::core::ffi::c_int) as int16) >= 0i16) {
                break;
            }
        }
        DrawTwoHudDigits(addrl_kDigitTilesetsHealth, r18, 0x8c as uint16, game);
    }
    if ram_word!(game, samus_max_missiles) != 0u16
        && ram_word!(game, samus_missiles) != ram_word!(game, samus_prev_missiles)
    {
        set_ram_word!(game.samus_prev_missiles = ram_word!(game, samus_missiles));
        DrawThreeHudDigits(
            addrl_kDigitTilesetsWeapon,
            ram_word!(game, samus_missiles),
            0x94 as uint16,
            game,
        );
    }
    if ram_word!(game, samus_max_super_missiles) != 0u16
        && ram_word!(game, samus_super_missiles) != ram_word!(game, samus_prev_super_missiles)
    {
        set_ram_word!(game.samus_prev_super_missiles = ram_word!(game, samus_super_missiles));
        if (ram_word!(game, joypad_dbg_flags) as ::core::ffi::c_int & 0x1f40 as ::core::ffi::c_int)
            != 0i32
        {
            DrawThreeHudDigits(
                addrl_kDigitTilesetsWeapon,
                ram_word!(game, samus_prev_super_missiles),
                0x9c as uint16,
                game,
            );
        } else {
            DrawTwoHudDigits(
                addrl_kDigitTilesetsWeapon,
                ram_word!(game, samus_prev_super_missiles),
                0x9c as uint16,
                game,
            );
        }
    }
    if ram_word!(game, samus_max_power_bombs) != 0u16
        && ram_word!(game, samus_power_bombs) != ram_word!(game, samus_prev_power_bombs)
    {
        set_ram_word!(game.samus_prev_power_bombs = ram_word!(game, samus_power_bombs));
        DrawTwoHudDigits(
            addrl_kDigitTilesetsWeapon,
            ram_word!(game, samus_power_bombs),
            0xa2 as uint16,
            game,
        );
    }
    if ram_word!(game, hud_item_index) != ram_word!(game, samus_prev_hud_item_index) {
        ToggleHudItemHighlight(ram_word!(game, hud_item_index), 0x1000 as uint16, game);
        ToggleHudItemHighlight(
            ram_word!(game, samus_prev_hud_item_index),
            0x1400 as uint16,
            game,
        );
        set_ram_word!(game.samus_prev_hud_item_index = ram_word!(game, hud_item_index));
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) != 3u8
            && *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) != 20u8
            && ram_word!(game, grapple_beam_function) == 0xc4f0u16
            && ram_word!(game, time_is_frozen_flag) == 0
        {
            QueueSfx1_Max6(0x39 as uint16, game);
        }
    }
    let mut v4: uint16 = 5120 as uint16;
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x5b5) as ::core::ffi::c_int
        & 0x10 as ::core::ffi::c_int)
        != 0i32
    {
        v4 = 4096 as uint16;
    }
    ToggleHudItemHighlight(ram_word!(game, samus_auto_cancel_hud_item_index), v4, game);
    let mut v5: uint16 = ram_word!(game, vram_write_queue_tail);
    (*gVramWriteEntry(
        ram_word!(game, vram_write_queue_tail) as ::core::ffi::c_int,
        game,
    ))
    .size = 192 as uint16;
    v5 = (v5).wrapping_add(2u16);
    (*gVramWriteEntry(v5 as ::core::ffi::c_int, game)).size =
        ((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            as uint16;
    v5 = (v5).wrapping_add(2u16);
    let fresh71 = v5;
    v5 = v5.wrapping_add(1);
    (*gVramWriteEntry(fresh71 as ::core::ffi::c_int, game)).size = 126 as uint16;
    (*gVramWriteEntry(v5 as ::core::ffi::c_int, game)).size = addr_unk_605820 as uint16;
    set_ram_word!(game.vram_write_queue_tail = (v5).wrapping_add(2u16));
}
static kHudItemTilemapOffsets: [uint16; 5] = [0x14, 0x1c, 0x22, 0x28, 0x2e];
pub unsafe fn ToggleHudItemHighlight(a: uint16, k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    set_ram_word!(game.hud_item_tilemap_palette_bits = k);
    v2 = ((a) as i16).wrapping_sub(1i16);
    if v2 >= 0i16 {
        let v3: ::core::ffi::c_int =
            kHudItemTilemapOffsets[v2 as usize] as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(v3 as isize)
            != 11279u16
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
                .offset(v3 as isize) =
                (ram_word!(game, hud_item_tilemap_palette_bits) as ::core::ffi::c_int
                    | *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
                        .offset(v3 as isize) as ::core::ffi::c_int
                        & 0xe3ff as ::core::ffi::c_int) as uint16;
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
            .offset((v3 + 1 as ::core::ffi::c_int) as isize)
            != 11279u16
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
                .offset((v3 + 1 as ::core::ffi::c_int) as isize) =
                (ram_word!(game, hud_item_tilemap_palette_bits) as ::core::ffi::c_int
                    | *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
                        .offset((v3 + 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        & 0xe3ff as ::core::ffi::c_int) as uint16;
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
            .offset((v3 + 32 as ::core::ffi::c_int) as isize)
            != 11279u16
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
                .offset((v3 + 32 as ::core::ffi::c_int) as isize) =
                (ram_word!(game, hud_item_tilemap_palette_bits) as ::core::ffi::c_int
                    | *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
                        .offset((v3 + 32 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        & 0xe3ff as ::core::ffi::c_int) as uint16;
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
            .offset((v3 + 33 as ::core::ffi::c_int) as isize)
            != 11279u16
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
                .offset((v3 + 33 as ::core::ffi::c_int) as isize) =
                (ram_word!(game, hud_item_tilemap_palette_bits) as ::core::ffi::c_int
                    | *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
                        .offset((v3 + 33 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        & 0xe3ff as ::core::ffi::c_int) as uint16;
        }
        if 2 as ::core::ffi::c_int * v2 as ::core::ffi::c_int == 0 {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
                .offset((v3 + 2 as ::core::ffi::c_int) as isize)
                != 11279u16
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
                    .offset((v3 + 2 as ::core::ffi::c_int) as isize) =
                    (ram_word!(game, hud_item_tilemap_palette_bits) as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
                            .offset((v3 + 2 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            & 0xe3ff as ::core::ffi::c_int) as uint16;
            }
            if *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
                .offset((v3 + 34 as ::core::ffi::c_int) as isize)
                != 11279u16
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
                    .offset((v3 + 34 as ::core::ffi::c_int) as isize) =
                    (ram_word!(game, hud_item_tilemap_palette_bits) as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
                            .offset((v3 + 34 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            & 0xe3ff as ::core::ffi::c_int) as uint16;
            }
        }
    }
}
pub unsafe fn DrawThreeHudDigits(addr: uint32, a: uint16, k: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = *(RomPtr(
        (addr).wrapping_add(
            (2 as ::core::ffi::c_int * (a as ::core::ffi::c_int / 100 as ::core::ffi::c_int))
                as uint32_t,
        ),
        game,
    )
        as *mut uint16);
    DrawTwoHudDigits(
        addr,
        (a as ::core::ffi::c_int % 100 as ::core::ffi::c_int) as uint16,
        (k).wrapping_add(2u16),
        game,
    );
}
pub unsafe fn DrawTwoHudDigits(addr: uint32, a: uint16, k: uint16, game: &mut Game) {
    let v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(v3 as isize) =
        *(RomPtr(
            (addr).wrapping_add(
                (2 as ::core::ffi::c_int * (a as ::core::ffi::c_int / 10 as ::core::ffi::c_int))
                    as uint32_t,
            ),
            game,
        ) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16)
        .offset((v3 + 1 as ::core::ffi::c_int) as isize) = *(RomPtr(
        (addr).wrapping_add(
            (2 as ::core::ffi::c_int * (a as ::core::ffi::c_int % 10 as ::core::ffi::c_int))
                as uint32_t,
        ),
        game,
    ) as *mut uint16);
}
static kTimerProcessFuncs: [Option<Func_U8>; 7] = unsafe {
    [
        Some(ProcessTimer_Empty as unsafe fn(&mut Game) -> uint8),
        Some(ProcessTimer_CeresStart as unsafe fn(&mut Game) -> uint8),
        Some(ProcessTimer_MotherBrainStart as unsafe fn(&mut Game) -> uint8),
        Some(ProcessTimer_InitialDelay as unsafe fn(&mut Game) -> uint8),
        Some(ProcessTimer_MovementDelayed as unsafe fn(&mut Game) -> uint8),
        Some(ProcessTimer_MovingIntoPlace as unsafe fn(&mut Game) -> uint8),
        Some(ProcessTimer_Decrement as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn ProcessTimer(game: &mut Game) -> uint8 {
    kTimerProcessFuncs[ram_word!(game, timer_status) as uint8 as usize]
        .expect("non-null function pointer")(game)
}
pub unsafe fn ProcessTimer_CeresStart(game: &mut Game) -> uint8 {
    ClearTimerRam(game);
    SetTimerMinutes(0x100 as uint16, game);
    set_ram_word!(game.timer_status = -(32765 as ::core::ffi::c_int) as uint16);
    0 as uint8
}
pub fn ProcessTimer_Empty(game: &mut Game) -> uint8 {
    0 as uint8
}
pub unsafe fn ProcessTimer_MotherBrainStart(game: &mut Game) -> uint8 {
    ClearTimerRam(game);
    SetTimerMinutes(0x300 as uint16, game);
    set_ram_word!(game.timer_status = -(32765 as ::core::ffi::c_int) as uint16);
    0 as uint8
}
pub unsafe fn ProcessTimer_InitialDelay(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x948) as *mut uint16 as *mut uint8).offset(0) =
        ((ram_word!(game, timer_x_pos)) as u8).wrapping_add(1u8);
    if (ram_word!(game, timer_x_pos) as uint8) >= 0x10u8 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x943) as *mut uint16 as *mut uint8)
            .offset(0) = ((ram_word!(game, timer_status)) as u8).wrapping_add(1u8);
    }
    0 as uint8
}
pub unsafe fn ProcessTimer_MovementDelayed(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x948) as *mut uint16 as *mut uint8).offset(0) =
        ((ram_word!(game, timer_x_pos)) as u8).wrapping_add(1u8);
    if (ram_word!(game, timer_x_pos) as uint8) >= 0x60u8 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x948) as *mut uint16 as *mut uint8)
            .offset(0) = 0 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x943) as *mut uint16 as *mut uint8)
            .offset(0) = ((ram_word!(game, timer_status)) as u8).wrapping_add(1u8);
    }
    ProcessTimer_Decrement(game)
}
pub unsafe fn ProcessTimer_MovingIntoPlace(game: &mut Game) -> uint8 {
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut v1: uint16 = (ram_word!(game, timer_x_pos)).wrapping_add(224u16);
    if (ram_word!(game, timer_x_pos)).wrapping_add(224u16) >= 0xdc00u16 {
        v0 = 1 as ::core::ffi::c_int;
        v1 = -(9216 as ::core::ffi::c_int) as uint16;
    }
    set_ram_word!(game.timer_x_pos = v1);
    let mut v2: uint16 = (ram_word!(game, timer_y_pos)).wrapping_sub(193u16);
    if (ram_word!(game, timer_y_pos)).wrapping_sub(193u16) < 0x3000u16 {
        v0 += 1;
        v2 = 12288 as uint16;
    }
    set_ram_word!(game.timer_y_pos = v2);
    if v0 == 2i32 {
        let ref mut fresh18 = *((&raw mut game.g_ram as *mut uint8).offset(0x943) as *mut uint16);
        *fresh18 = (*fresh18).wrapping_add(1);
    }
    ProcessTimer_Decrement(game)
}
pub unsafe fn SetTimerMinutes(a: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x945) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x946) as *mut uint16) = a;
}
pub unsafe fn ClearTimerRam(game: &mut Game) {
    set_ram_word!(game.timer_x_pos = 0x8000 as uint16);
    set_ram_word!(game.timer_y_pos = 0x8000 as uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x945) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x946) as *mut uint16) = 0 as uint16;
    set_ram_word!(game.timer_status = 0 as uint16);
}
pub unsafe fn DecrementDecimal(number: *mut uint8, value: uint8) -> bool {
    let mut result: ::core::ffi::c_int = (*number as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        + (!(value as ::core::ffi::c_int) & 0xf as ::core::ffi::c_int)
        + 1 as ::core::ffi::c_int;
    if result < 0x10i32 {
        result = result - 0x6 as ::core::ffi::c_int
            & (if (result - 0x6 as ::core::ffi::c_int) < 0i32 {
                0xf as ::core::ffi::c_int
            } else {
                0x1f as ::core::ffi::c_int
            });
    }
    result = (*number as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int)
        + (!(value as ::core::ffi::c_int) & 0xf0 as ::core::ffi::c_int)
        + result;
    if result < 0x100i32 {
        result -= 0x60 as ::core::ffi::c_int;
    }
    *number = result as uint8;
    result < 0x100i32
}
pub unsafe fn ProcessTimer_Decrement(game: &mut Game) -> uint8 {
    static kTimerCentisecondDecrements: [uint8; 128] = [
        1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2, 2, 1, 2, 2,
        1, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2, 2, 1,
        2, 2, 1, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2,
        2, 1, 2, 2, 1, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2,
        2, 1, 2, 2, 2, 1, 2, 2,
    ];
    if DecrementDecimal(
        (&raw mut game.g_ram as *mut uint8).offset(0x945),
        kTimerCentisecondDecrements[(ram_word!(game, nmi_frame_counter_word) as ::core::ffi::c_int
            & 0x7f as ::core::ffi::c_int) as usize],
    ) && DecrementDecimal(
        (&raw mut game.g_ram as *mut uint8).offset(0x946),
        1 as uint8,
    ) {
        if DecrementDecimal(
            (&raw mut game.g_ram as *mut uint8).offset(0x947),
            1 as uint8,
        ) {
            *(&raw mut game.g_ram as *mut uint8).offset(0x945) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x946) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x947) = 0 as uint8;
        } else {
            *(&raw mut game.g_ram as *mut uint8).offset(0x946) = 0x59 as uint8;
        }
    }
    ((*(&raw mut game.g_ram as *mut uint8).offset(0x945) as ::core::ffi::c_int
        | *(&raw mut game.g_ram as *mut uint8).offset(0x946) as ::core::ffi::c_int
        | *(&raw mut game.g_ram as *mut uint8).offset(0x947) as ::core::ffi::c_int)
        == 0i32) as uint8
}
pub unsafe fn DrawTimer(game: &mut Game) {
    DrawTimerSpritemap(0 as uint16, addr_word_80A060 as uint16, game);
    DrawTwoTimerDigits(
        *((&raw mut game.g_ram as *mut uint8).offset(0x947) as *mut uint16),
        0xffe4 as uint16,
        game,
    );
    DrawTwoTimerDigits(
        *((&raw mut game.g_ram as *mut uint8).offset(0x946) as *mut uint16),
        0xfffc as uint16,
        game,
    );
    DrawTwoTimerDigits(
        *((&raw mut game.g_ram as *mut uint8).offset(0x945) as *mut uint16),
        0x14 as uint16,
        game,
    );
}
pub unsafe fn DrawTwoTimerDigits(a: uint16, k: uint16, game: &mut Game) {
    DrawTimerSpritemap(
        k,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x809fd4)) as *const uint8 as *mut uint16)
            .offset(
                ((a as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int) >> 4 as ::core::ffi::c_int)
                    as isize,
            ),
        game,
    );
    DrawTimerSpritemap(
        (k).wrapping_add(8u16),
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x809fd4)) as *const uint8 as *mut uint16)
            .offset((a as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as isize),
        game,
    );
}
pub unsafe fn DrawTimerSpritemap(a: uint16, j: uint16, game: &mut Game) {
    DrawSpritemap(
        0x80 as uint8,
        j,
        (a).wrapping_add(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x948) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                )) as u16,
        ),
        *((&raw mut game.g_ram as *mut uint8).offset(0x94a) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) as uint16,
        2560 as uint16,
        game,
    );
}
pub unsafe fn StartGameplay_Async(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut uint8 = (&raw mut game.g_ram as *mut uint8).offset(0x613);
    let mut current_block_54: u64;
    match *_state_variable_ as ::core::ffi::c_int {
        0 => {
            WriteRegWord(MDMAEN as uint16, 0 as uint16, game);
            set_ram_word!(game.scrolling_finished_hook = 0 as uint16);
            set_ram_word!(game.music_data_index = 0 as uint16);
            set_ram_word!(game.music_track_index = 0 as uint16);
            set_ram_word!(game.timer_status = 0 as uint16);
            ResetSoundQueues(game);
            set_ram_word!(game.debug_disable_sounds = -(1 as ::core::ffi::c_int) as uint16);
            DisableNMI(game);
            DisableIrqInterrupts(game);
            LoadDestinationRoomThings(game);
            current_block_54 = 9431711618385729511;
        }
        1 => {
            current_block_54 = 9431711618385729511;
        }
        2 => {
            current_block_54 = 4121473168792104488;
        }
        3 => {
            current_block_54 = 3421737773041484705;
        }
        _ => {
            current_block_54 = 7659304154607701039;
        }
    }
    if current_block_54 == 9431711618385729511 {
        let mut _coret_: uint8 = Play20FramesOfMusic_Async(game);
        if _coret_ != 0 {
            *_state_variable_ = 1 as uint8;
            return _coret_;
        }
        ClearAnimtiles(game);
        WaitUntilEndOfVblankAndClearHdma(game);
        InitializeSpecialEffectsForNewRoom(game);
        ClearPLMs(game);
        ClearEprojs(game);
        ClearPaletteFXObjects(game);
        UpdateBeamTilesAndPalette(game);
        LoadColorsForSpritesBeamsAndEnemies(game);
        LoadEnemies(game);
        LoadRoomMusic(game);
        current_block_54 = 4121473168792104488;
    }
    if current_block_54 == 4121473168792104488 {
        let mut _coret__0: uint8 = Play20FramesOfMusic_Async(game);
        if _coret__0 != 0 {
            *_state_variable_ = 2 as uint8;
            return _coret__0;
        }
        UpdateMusicTrackIndex(game);
        NullFunc();
        ClearBG2Tilemap(game);
        LoadLevelDataAndOtherThings(game);
        LoadFXHeader(game);
        LoadLibraryBackground(game);
        CalculateLayer2Xpos(game);
        CalculateLayer2Ypos(game);
        set_ram_word!(game.bg2_x_scroll = ram_word!(game, layer2_x_pos));
        set_ram_word!(game.bg2_y_scroll = ram_word!(game, layer2_y_pos));
        CalculateBgScrolls(game);
        DisplayViewablePartOfRoom(game);
        EnableNMI(game);
        set_ram_word!(
            game.irqhandler_next_handler = (if ram_word!(game, room_loading_irq_handler) == 0u16 {
                4 as ::core::ffi::c_int
            } else {
                ram_word!(game, room_loading_irq_handler) as ::core::ffi::c_int
            }) as uint16
        );
        EnableIrqInterrupts(game);
        current_block_54 = 3421737773041484705;
    }
    if current_block_54 == 3421737773041484705 {
        let mut _coret__1: uint8 = Play20FramesOfMusic_Async(game);
        if _coret__1 != 0 {
            *_state_variable_ = 3 as uint8;
            return _coret__1;
        }
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x8 as uint8,
                field_1: 0x8 as uint8,
                field_2: 0xb7eb as VoidP,
            },
            game,
        );
        set_ram_word!(game.door_transition_function = ((0x82e737i32) as u16));
    }
    *_state_variable_ = 0 as uint8;
    0 as CoroutineRet
}
pub unsafe fn Play20FramesOfMusic_Async(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut uint8 = (&raw mut game.g_ram as *mut uint8).offset(0x614);
    's_57: {
        let mut current_block_9: u64;
        match *_state_variable_ as ::core::ffi::c_int {
            0 => {
                EnableNMI(game);
                set_ram_word!(game.my_counter = 0 as uint16);
                current_block_9 = 8258075665625361029;
            }
            1 => {
                current_block_9 = 4436247046726756244;
            }
            _ => {
                current_block_9 = 13536709405535804910;
            }
        }
        loop {
            match current_block_9 {
                13536709405535804910 => {
                    break 's_57;
                }
                8258075665625361029 => {
                    if ram_word!(game, my_counter) != 20u16 {
                        HandleMusicQueue(game);
                        current_block_9 = 4436247046726756244;
                    } else {
                        DisableNMI(game);
                        current_block_9 = 13536709405535804910;
                    }
                }
                _ => {
                    let mut _coret_: uint8 = WaitForNMI_Async(game);
                    if _coret_ != 0 {
                        *_state_variable_ = 1 as uint8;
                        return _coret_;
                    }
                    let ref mut fresh79 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0x77e) as *mut uint16);
                    *fresh79 = (*fresh79).wrapping_add(1);
                    current_block_9 = 8258075665625361029;
                }
            }
        }
    }
    *_state_variable_ = 0 as uint8;
    0 as CoroutineRet
}
pub unsafe fn ResumeGameplay(game: &mut Game) {
    WriteRegWord(MDMAEN as uint16, 0 as uint16, game);
    DisableNMI(game);
    DisableIrqInterrupts(game);
    LoadCRETilesTilesetTilesAndPalette(game);
    LoadLibraryBackground(game);
    DisplayViewablePartOfRoom(game);
    LoadRoomPlmGfx(game);
    EnableNMI(game);
    EnableIrqInterrupts(game);
}
pub unsafe fn DisplayViewablePartOfRoom(game: &mut Game) {
    let mut v1: ::core::ffi::c_int = 0;
    let mut v2: ::core::ffi::c_int = 0;
    let mut v3: ::core::ffi::c_int = 0;
    v1 = (*(&raw mut game.g_ram as *mut uint8).offset(0x58) as ::core::ffi::c_int
        - *(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int;
    set_ram_word!(game.size_of_bg2 = (v1 & 0xf800 as ::core::ffi::c_int) as uint16);
    CalculateBgScrollAndLayerPositionBlocks(game);
    v2 = 0 as ::core::ffi::c_int;
    loop {
        v3 = v2;
        set_ram_word!(game.blocks_to_update_x_block = ram_word!(game, layer1_x_block));
        set_ram_word!(game.blocks_to_update_y_block = ram_word!(game, layer1_y_block));
        set_ram_word!(game.vram_blocks_to_update_x_block = ram_word!(game, bg1_x_block));
        set_ram_word!(game.vram_blocks_to_update_y_block = ram_word!(game, bg1_y_block));
        UploadLevelDataColumn(game);
        if *(&raw mut game.g_ram as *mut uint8).offset(0x91b) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0
        {
            set_ram_word!(game.blocks_to_update_x_block = ram_word!(game, layer2_x_block));
            set_ram_word!(game.blocks_to_update_y_block = ram_word!(game, layer2_y_block));
            set_ram_word!(game.vram_blocks_to_update_x_block = ram_word!(game, bg2_x_block));
            set_ram_word!(game.vram_blocks_to_update_y_block = ram_word!(game, bg2_y_block));
            UploadBackgroundDataColumn(game);
        }
        NMI_ProcessVramWriteQueue(game);
        let ref mut fresh42 = *((&raw mut game.g_ram as *mut uint8).offset(0x8f7) as *mut uint16);
        *fresh42 = (*fresh42).wrapping_add(1);
        let ref mut fresh43 = *((&raw mut game.g_ram as *mut uint8).offset(0x907) as *mut uint16);
        *fresh43 = (*fresh43).wrapping_add(1);
        let ref mut fresh44 = *((&raw mut game.g_ram as *mut uint8).offset(0x8fb) as *mut uint16);
        *fresh44 = (*fresh44).wrapping_add(1);
        let ref mut fresh45 = *((&raw mut game.g_ram as *mut uint8).offset(0x90b) as *mut uint16);
        *fresh45 = (*fresh45).wrapping_add(1);
        v2 += 1;
        if (v3 == 16i32) {
            break;
        }
    }
}
pub unsafe fn QueueClearingOfFxTilemap(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 3838 as ::core::ffi::c_int;
    while i >= 0i32 {
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
            .xray_tilemaps as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 6222 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let v1: uint16 = ram_word!(game, vram_write_queue_tail);
    let v2: *mut VramWriteEntry = gVramWriteEntry(
        ram_word!(game, vram_write_queue_tail) as ::core::ffi::c_int,
        game,
    );
    (*v2).size = 3840 as uint16;
    (*v2).src.addr =
        ((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut uint16) = 126 as uint16;
    (*v2).vram_dst = addr_unk_605880 as uint16;
    set_ram_word!(game.vram_write_queue_tail = (v1).wrapping_add(7u16));
}
pub unsafe fn ClearBG2Tilemap(game: &mut Game) {
    WriteRegWord(VMADDL as uint16, 0x4800 as uint16, game);
    WriteRegWord(DMAP1 as uint16, 0x1808 as uint16, game);
    WriteRegWord(A1T1L as uint16, 0xa29a as uint16, game);
    WriteRegWord(A1B1 as uint16, 0x80 as uint16, game);
    WriteRegWord(DAS1L as uint16, 0x800 as uint16, game);
    WriteReg(VMAIN as uint16, 0 as uint8, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteRegWord(VMADDL as uint16, 0x4800 as uint16, game);
    WriteRegWord(DMAP1 as uint16, 0x1908 as uint16, game);
    WriteRegWord(A1T1L as uint16, 0xa29a as uint16, game);
    WriteRegWord(A1B1 as uint16, 0x80 as uint16, game);
    WriteRegWord(DAS1L as uint16, 0x800 as uint16, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
}
pub unsafe fn ClearFXTilemap(game: &mut Game) {
    WriteRegWord(VMADDL as uint16, 0x5880 as uint16, game);
    WriteRegWord(DMAP1 as uint16, 0x1808 as uint16, game);
    WriteRegWord(A1T1L as uint16, 0xa2f7 as uint16, game);
    WriteRegWord(A1B1 as uint16, 0x80 as uint16, game);
    WriteRegWord(DAS1L as uint16, 0x780 as uint16, game);
    WriteReg(VMAIN as uint16, 0 as uint8, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteRegWord(VMADDL as uint16, 0x5880 as uint16, game);
    WriteRegWord(DMAP1 as uint16, 0x1908 as uint16, game);
    WriteRegWord(A1T1L as uint16, 0xa2f8 as uint16, game);
    WriteRegWord(A1B1 as uint16, 0x80 as uint16, game);
    WriteRegWord(DAS1L as uint16, 0x780 as uint16, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
}
pub unsafe fn CalculateLayer2Xpos(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0x91b) == 0 {
        set_ram_word!(game.layer2_x_pos = ram_word!(game, layer1_x_pos));
        return 0 as uint8;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0x91b) != 1u8 {
        let t: ::core::ffi::c_int = *(&raw mut game.g_ram as *mut uint8).offset(0x91b)
            as ::core::ffi::c_int
            & 0xfe as ::core::ffi::c_int;
        let var933: uint8 = (t * *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16
            as *mut uint8)
            .offset(0) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        set_ram_word!(
            game.layer2_x_pos = (t * *((&raw mut game.g_ram as *mut uint8).offset(0x911)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ) as ::core::ffi::c_int
                + var933 as ::core::ffi::c_int) as uint16
        );
        return 0 as uint8;
    }
    1 as uint8
}
pub unsafe fn CalculateLayer2Ypos(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0x91c) == 0 {
        set_ram_word!(game.layer2_y_pos = ram_word!(game, layer1_y_pos));
        return 0 as uint8;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0x91c) != 1u8 {
        let t: ::core::ffi::c_int = *(&raw mut game.g_ram as *mut uint8).offset(0x91c)
            as ::core::ffi::c_int
            & 0xfe as ::core::ffi::c_int;
        let var933: uint8 = (t * ram_word!(game, layer1_y_pos) as uint8 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        set_ram_word!(
            game.layer2_y_pos = (t * *((&raw mut game.g_ram as *mut uint8).offset(0x915)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ) as ::core::ffi::c_int
                + var933 as ::core::ffi::c_int) as uint16
        );
        return 0 as uint8;
    }
    1 as uint8
}
pub unsafe fn CalculateBgScrolls(game: &mut Game) {
    set_ram_word!(
        game.reg_BG1HOFS =
            (ram_word!(game, bg1_x_offset)).wrapping_add(ram_word!(game, layer1_x_pos))
    );
    set_ram_word!(
        game.reg_BG1VOFS =
            (ram_word!(game, bg1_y_offset)).wrapping_add(ram_word!(game, layer1_y_pos))
    );
    set_ram_word!(
        game.reg_BG2HOFS =
            (ram_word!(game, bg2_x_scroll)).wrapping_add(ram_word!(game, layer2_x_pos))
    );
    set_ram_word!(
        game.reg_BG2VOFS =
            (ram_word!(game, bg2_y_scroll)).wrapping_add(ram_word!(game, layer2_y_pos))
    );
}
pub unsafe fn UpdateScrollVarsUpdateMap(game: &mut Game) {
    CalculateBgScrolls(game);
    UpdateBgGraphicsWhenScrolling(game);
}
pub unsafe fn CalculateLayer2PosAndScrollsWhenScrolling(game: &mut Game) {
    if ram_word!(game, time_is_frozen_flag) == 0 {
        set_ram_word!(
            game.reg_BG1HOFS =
                (ram_word!(game, bg1_x_offset)).wrapping_add(ram_word!(game, layer1_x_pos))
        );
        set_ram_word!(
            game.reg_BG1VOFS =
                (ram_word!(game, bg1_y_offset)).wrapping_add(ram_word!(game, layer1_y_pos))
        );
        if CalculateLayer2Xpos(game) == 0 {
            set_ram_word!(
                game.reg_BG2HOFS =
                    (ram_word!(game, bg2_x_scroll)).wrapping_add(ram_word!(game, layer2_x_pos))
            );
        }
        if CalculateLayer2Ypos(game) == 0 {
            set_ram_word!(
                game.reg_BG2VOFS =
                    (ram_word!(game, bg2_y_scroll)).wrapping_add(ram_word!(game, layer2_y_pos))
            );
        }
        UpdateBgGraphicsWhenScrolling(game);
    }
}
pub unsafe fn UpdateBgGraphicsWhenScrolling(game: &mut Game) {
    CalculateBgScrollAndLayerPositionBlocks(game);
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let v1: bool = ((ram_word!(game, layer1_x_block)) as i16)
        .wrapping_sub((ram_word!(game, previous_layer1_x_block)) as i16)
        < 0i16;
    if ram_word!(game, layer1_x_block) != ram_word!(game, previous_layer1_x_block) {
        set_ram_word!(game.previous_layer1_x_block = ram_word!(game, layer1_x_block));
        if !v1 {
            v0 = 16 as ::core::ffi::c_int;
        }
        set_ram_word!(
            game.blocks_to_update_x_block =
                (ram_word!(game, layer1_x_block) as ::core::ffi::c_int + v0) as uint16
        );
        set_ram_word!(
            game.vram_blocks_to_update_x_block =
                (ram_word!(game, bg1_x_block) as ::core::ffi::c_int + v0) as uint16
        );
        set_ram_word!(game.blocks_to_update_y_block = ram_word!(game, layer1_y_block));
        set_ram_word!(game.vram_blocks_to_update_y_block = ram_word!(game, bg1_y_block));
        UploadLevelDataColumn(game);
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0x91b) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        let mut v2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let v3: bool = ((ram_word!(game, layer2_x_block)) as i16)
            .wrapping_sub((ram_word!(game, previous_layer2_x_block)) as i16)
            < 0i16;
        if ram_word!(game, layer2_x_block) != ram_word!(game, previous_layer2_x_block) {
            set_ram_word!(game.previous_layer2_x_block = ram_word!(game, layer2_x_block));
            if !v3 {
                v2 = 16 as ::core::ffi::c_int;
            }
            set_ram_word!(
                game.blocks_to_update_x_block =
                    (ram_word!(game, layer2_x_block) as ::core::ffi::c_int + v2) as uint16
            );
            set_ram_word!(
                game.vram_blocks_to_update_x_block =
                    (ram_word!(game, bg2_x_block) as ::core::ffi::c_int + v2) as uint16
            );
            set_ram_word!(game.blocks_to_update_y_block = ram_word!(game, layer2_y_block));
            set_ram_word!(game.vram_blocks_to_update_y_block = ram_word!(game, bg2_y_block));
            UploadBackgroundDataColumn(game);
        }
    }
    let mut v4: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let v5: bool = ((ram_word!(game, layer1_y_block)) as i16)
        .wrapping_sub((ram_word!(game, previous_layer1_y_block)) as i16)
        < 0i16;
    if ram_word!(game, layer1_y_block) != ram_word!(game, previous_layer1_y_block) {
        set_ram_word!(game.previous_layer1_y_block = ram_word!(game, layer1_y_block));
        if !v5 {
            v4 = 15 as ::core::ffi::c_int;
        }
        set_ram_word!(
            game.blocks_to_update_y_block =
                (ram_word!(game, layer1_y_block) as ::core::ffi::c_int + v4) as uint16
        );
        set_ram_word!(
            game.vram_blocks_to_update_y_block =
                (ram_word!(game, bg1_y_block) as ::core::ffi::c_int + v4) as uint16
        );
        set_ram_word!(game.blocks_to_update_x_block = ram_word!(game, layer1_x_block));
        set_ram_word!(game.vram_blocks_to_update_x_block = ram_word!(game, bg1_x_block));
        UpdateLevelDataRow(game);
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0x91c) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        let mut v6: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let v7: bool = ((ram_word!(game, layer2_y_block)) as i16)
            .wrapping_sub((ram_word!(game, previous_layer2_y_block)) as i16)
            < 0i16;
        if ram_word!(game, layer2_y_block) != ram_word!(game, previous_layer2_y_block) {
            set_ram_word!(game.previous_layer2_y_block = ram_word!(game, layer2_y_block));
            if !v7 {
                v6 = 15 as ::core::ffi::c_int;
            }
            set_ram_word!(
                game.blocks_to_update_y_block =
                    (ram_word!(game, layer2_y_block) as ::core::ffi::c_int + v6) as uint16
            );
            set_ram_word!(
                game.vram_blocks_to_update_y_block =
                    (ram_word!(game, bg2_y_block) as ::core::ffi::c_int + v6) as uint16
            );
            set_ram_word!(game.blocks_to_update_x_block = ram_word!(game, layer2_x_block));
            set_ram_word!(game.vram_blocks_to_update_x_block = ram_word!(game, bg2_x_block));
            UpdateBackgroundDataRow(game);
        }
    }
}
pub unsafe fn CalculateBgScrollAndLayerPositionBlocks(game: &mut Game) {
    set_ram_word!(
        game.bg1_x_block = (ram_word!(game, reg_BG1HOFS) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint16
    );
    set_ram_word!(
        game.bg2_x_block = (ram_word!(game, reg_BG2HOFS) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint16
    );
    let mut v0: uint16 =
        (ram_word!(game, layer1_x_pos) as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint16;
    if (ram_word!(game, layer1_x_pos) as ::core::ffi::c_int >> 4 as ::core::ffi::c_int
        & 0x800 as ::core::ffi::c_int)
        != 0i32
    {
        v0 = (v0) | (0xf000u16);
    }
    set_ram_word!(game.layer1_x_block = v0);
    let mut v1: uint16 =
        (ram_word!(game, layer2_x_pos) as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint16;
    if (ram_word!(game, layer2_x_pos) as ::core::ffi::c_int >> 4 as ::core::ffi::c_int
        & 0x800 as ::core::ffi::c_int)
        != 0i32
    {
        v1 = (v1) | (0xf000u16);
    }
    set_ram_word!(game.layer2_x_block = v1);
    set_ram_word!(
        game.bg1_y_block = (ram_word!(game, reg_BG1VOFS) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint16
    );
    set_ram_word!(
        game.bg2_y_block = (ram_word!(game, reg_BG2VOFS) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint16
    );
    let mut v2: uint16 =
        (ram_word!(game, layer1_y_pos) as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint16;
    if (ram_word!(game, layer1_y_pos) as ::core::ffi::c_int >> 4 as ::core::ffi::c_int
        & 0x800 as ::core::ffi::c_int)
        != 0i32
    {
        v2 = (v2) | (0xf000u16);
    }
    set_ram_word!(game.layer1_y_block = v2);
    let mut v3: uint16 =
        (ram_word!(game, layer2_y_pos) as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint16;
    if (ram_word!(game, layer2_y_pos) as ::core::ffi::c_int >> 4 as ::core::ffi::c_int
        & 0x800 as ::core::ffi::c_int)
        != 0i32
    {
        v3 = (v3) | (0xf000u16);
    }
    set_ram_word!(game.layer2_y_block = v3);
}
pub unsafe fn HandleAutoscrolling_X(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v3: int16 = 0;
    let mut v6: int16 = 0;
    let mut v4: uint16 = 0;
    let mut var933: uint16 = 0;
    if ram_word!(game, time_is_frozen_flag) == 0 {
        let mut var939: uint16 = ram_word!(game, layer1_x_pos);
        if (ram_word!(game, layer1_x_pos) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
            != 0i32
        {
            set_ram_word!(game.layer1_x_pos = 0 as uint16);
        }
        let v0: uint16 =
            u16::swap_bytes((ram_word!(game, room_width_in_scrolls)).wrapping_sub(1u16));
        if v0 < ram_word!(game, layer1_x_pos) {
            set_ram_word!(game.layer1_x_pos = v0);
        }
        v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as int16;
        let v2: uint16 = (Mult8x8(
            (((ram_word!(game, layer1_y_pos)).wrapping_add(128u16)) as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int) as uint8,
            ram_word!(game, room_width_in_scrolls) as uint8,
        ))
        .wrapping_add((v1) as u16);
        if *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20)
            .offset(v2 as isize)
            != 0
        {
            if *(&raw mut game.g_ram as *mut uint8)
                .offset(0xcd20)
                .offset(((v2).wrapping_add(1u16)) as isize)
                != 0
            {
                return;
            }
            var933 = (ram_word!(game, layer1_x_pos)) & (0xff00u16);
            let v5: uint16 = (var939 as ::core::ffi::c_int
                - ram_word!(game, absolute_moved_last_frame_x) as ::core::ffi::c_int
                - 2 as ::core::ffi::c_int) as uint16;
            if ((v5 as ::core::ffi::c_int
                - (ram_word!(game, layer1_x_pos) as ::core::ffi::c_int
                    & 0xff00 as ::core::ffi::c_int)) as int16)
                < 0i16
            {
                v4 = var933;
            } else {
                var939 = v5;
                v6 = *(&raw mut var939 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ) as int16;
                if *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(
                    ((Mult8x8(
                        (((ram_word!(game, layer1_y_pos)).wrapping_add(128u16))
                            as ::core::ffi::c_int
                            >> 8 as ::core::ffi::c_int) as uint8,
                        ram_word!(game, room_width_in_scrolls) as uint8,
                    ))
                    .wrapping_add((v6) as u16)) as isize,
                ) != 0
                {
                    v4 = var939;
                } else {
                    v4 = ((var939 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
                        + 256 as ::core::ffi::c_int) as uint16;
                }
            }
        } else {
            var933 = ((ram_word!(game, layer1_x_pos) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                + 256 as ::core::ffi::c_int) as uint16;
            if ((ram_word!(game, absolute_moved_last_frame_x) as ::core::ffi::c_int
                + var939 as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16)
                >= var933
            {
                v4 = var933;
            } else {
                var939 = (var939 as ::core::ffi::c_int
                    + (ram_word!(game, absolute_moved_last_frame_x) as ::core::ffi::c_int
                        + 2 as ::core::ffi::c_int)) as uint16;
                v3 = ((*(&raw mut var939 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ))
                .wrapping_add(1u8)) as int16;
                if *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(
                    ((Mult8x8(
                        (((ram_word!(game, layer1_y_pos)).wrapping_add(128u16))
                            as ::core::ffi::c_int
                            >> 8 as ::core::ffi::c_int) as uint8,
                        ram_word!(game, room_width_in_scrolls) as uint8,
                    ))
                    .wrapping_add((v3) as u16)) as isize,
                ) != 0
                {
                    v4 = var939;
                } else {
                    v4 = (var939) & (0xff00u16);
                }
            }
        }
        set_ram_word!(game.layer1_x_pos = v4);
    }
}
pub unsafe fn HandleScrollingWhenTriggeringScrollRight(game: &mut Game) {
    let mut v1: int16 = 0;
    let var939: uint16 = ram_word!(game, layer1_x_pos);
    if ((ram_word!(game, ideal_layer1_xpos)) as i16)
        .wrapping_sub((ram_word!(game, layer1_x_pos)) as i16)
        < 0i16
    {
        set_ram_word!(game.layer1_x_pos = ram_word!(game, ideal_layer1_xpos));
        set_ram_word!(game.layer1_x_subpos = 0 as uint16);
    }
    let v0: uint16 = u16::swap_bytes((ram_word!(game, room_width_in_scrolls)).wrapping_sub(1u16));
    if v0 >= ram_word!(game, layer1_x_pos) {
        v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as int16;
        let RegWord: uint16 = Mult8x8(
            (((ram_word!(game, layer1_y_pos)).wrapping_add(128u16)) as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int) as uint8,
            ram_word!(game, room_width_in_scrolls) as uint8,
        );
        if *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(
            (RegWord as ::core::ffi::c_int + 1 as ::core::ffi::c_int + v1 as ::core::ffi::c_int)
                as uint16 as isize,
        ) == 0
        {
            let var933: uint16 = (ram_word!(game, layer1_x_pos)) & (0xff00u16);
            let mut v4: uint16 = (var939 as ::core::ffi::c_int
                - ram_word!(game, absolute_moved_last_frame_x) as ::core::ffi::c_int
                - 2 as ::core::ffi::c_int) as uint16;
            if ((v4 as ::core::ffi::c_int
                - (ram_word!(game, layer1_x_pos) as ::core::ffi::c_int
                    & 0xff00 as ::core::ffi::c_int)) as int16)
                < 0i16
            {
                v4 = var933;
            }
            set_ram_word!(game.layer1_x_pos = v4);
        }
    } else {
        set_ram_word!(game.layer1_x_pos = v0);
    };
}
pub unsafe fn HandleScrollingWhenTriggeringScrollLeft(game: &mut Game) {
    let mut v0: int16 = 0;
    let var939: uint16 = ram_word!(game, layer1_x_pos);
    if ((ram_word!(game, layer1_x_pos)) as i16)
        .wrapping_sub((ram_word!(game, ideal_layer1_xpos)) as i16)
        < 0i16
    {
        set_ram_word!(game.layer1_x_pos = ram_word!(game, ideal_layer1_xpos));
        set_ram_word!(game.layer1_x_subpos = 0 as uint16);
    }
    if (ram_word!(game, layer1_x_pos) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32
    {
        v0 = *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as int16;
        let prod: uint16 = Mult8x8(
            (((ram_word!(game, layer1_y_pos)).wrapping_add(128u16)) as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int) as uint8,
            ram_word!(game, room_width_in_scrolls) as uint8,
        );
        if *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20)
            .offset(((prod).wrapping_add((v0) as u16)) as isize)
            == 0
        {
            let var933: uint16 = ((ram_word!(game, layer1_x_pos) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                + 256 as ::core::ffi::c_int) as uint16;
            let mut v1: uint16 = (ram_word!(game, absolute_moved_last_frame_x)
                as ::core::ffi::c_int
                + var939 as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16;
            if v1 >= var933 {
                v1 = var933;
            }
            set_ram_word!(game.layer1_x_pos = v1);
        }
    } else {
        set_ram_word!(game.layer1_x_pos = 0 as uint16);
    };
}
pub unsafe fn HandleAutoscrolling_Y(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v2: int16 = 0;
    let mut v4: int16 = 0;
    let mut v7: int16 = 0;
    let mut v10: int16 = 0;
    let mut v8: uint16 = 0;
    if ram_word!(game, time_is_frozen_flag) == 0 {
        let mut v0: uint16 = 0 as uint16;
        v1 = (((ram_word!(game, layer1_x_pos)).wrapping_add(128u16)) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as int16;
        let r20: uint16 = (Mult8x8(
            *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ),
            ram_word!(game, room_width_in_scrolls) as uint8,
        ))
        .wrapping_add((v1) as u16);
        if *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20)
            .offset(r20 as isize)
            != 1u8
        {
            v0 = 31 as uint16;
        }
        let var933: uint16 = v0;
        let mut var939: uint16 = ram_word!(game, layer1_y_pos);
        if (ram_word!(game, layer1_y_pos) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
            != 0i32
        {
            set_ram_word!(game.layer1_y_pos = 0 as uint16);
        }
        *(&raw mut v2 as *mut uint8).offset(0) =
            (((ram_word!(game, room_height_in_scrolls)).wrapping_sub(1u16)) as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int) as uint8;
        *(&raw mut v2 as *mut uint8).offset(
            (::core::mem::size_of::<int16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) = ((ram_word!(game, room_height_in_scrolls)) as u8).wrapping_sub(1u8);
        let v3: uint16 = (var933).wrapping_add((v2) as u16);
        if v3 < ram_word!(game, layer1_y_pos) {
            set_ram_word!(game.layer1_y_pos = v3);
        }
        v4 = (((ram_word!(game, layer1_x_pos)).wrapping_add(128u16)) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as int16;
        let v5: uint16 = (Mult8x8(
            *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ),
            ram_word!(game, room_width_in_scrolls) as uint8,
        ))
        .wrapping_add((v4) as u16);
        if *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20)
            .offset(v5 as isize)
            == 0
        {
            let var935: uint16 = ((ram_word!(game, layer1_y_pos) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                + 256 as ::core::ffi::c_int) as uint16;
            let mut v6: uint16 = (ram_word!(game, absolute_moved_last_frame_y)
                as ::core::ffi::c_int
                + var939 as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16;
            if v6 >= var935 {
                v8 = var935;
            } else {
                var939 = (var939 as ::core::ffi::c_int
                    + (ram_word!(game, absolute_moved_last_frame_y) as ::core::ffi::c_int
                        + 2 as ::core::ffi::c_int)) as uint16;
                v7 = (((ram_word!(game, layer1_x_pos)).wrapping_add(128u16)) as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int) as int16;
                let prod: uint16 = Mult8x8(
                    (*(&raw mut v6 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ))
                    .wrapping_add(1u8),
                    ram_word!(game, room_width_in_scrolls) as uint8,
                );
                if *(&raw mut game.g_ram as *mut uint8)
                    .offset(0xcd20)
                    .offset(((prod).wrapping_add((v7) as u16)) as isize)
                    != 0
                {
                    v8 = var939;
                } else {
                    v8 = (var939) & (0xff00u16);
                }
            }
            set_ram_word!(game.layer1_y_pos = v8);
            return;
        }
        if *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20)
            .offset(((ram_word!(game, room_width_in_scrolls)).wrapping_add(v5)) as isize)
            == 0
        {
            let var937: uint16 = (var933 as ::core::ffi::c_int
                + (ram_word!(game, layer1_y_pos) as ::core::ffi::c_int
                    & 0xff00 as ::core::ffi::c_int)) as uint16;
            if var937 < ram_word!(game, layer1_y_pos) {
                let mut v9: uint16 = (var939 as ::core::ffi::c_int
                    - ram_word!(game, absolute_moved_last_frame_y) as ::core::ffi::c_int
                    - 2 as ::core::ffi::c_int) as uint16;
                if ((v9) as i16).wrapping_sub((var937) as i16) < 0i16 {
                    v8 = var937;
                } else {
                    var939 = v9;
                    let prod_0: uint16 = Mult8x8(
                        *(&raw mut v9 as *mut uint8).offset(
                            (::core::mem::size_of::<uint16>())
                                .wrapping_div(::core::mem::size_of::<uint8>())
                                .wrapping_sub(1_usize) as isize,
                        ),
                        ram_word!(game, room_width_in_scrolls) as uint8,
                    );
                    v10 = (((ram_word!(game, layer1_x_pos)).wrapping_add(128u16))
                        as ::core::ffi::c_int
                        >> 8 as ::core::ffi::c_int) as int16;
                    if *(&raw mut game.g_ram as *mut uint8)
                        .offset(0xcd20)
                        .offset(((prod_0).wrapping_add((v10) as u16)) as isize)
                        != 0
                    {
                        v8 = var939;
                    } else {
                        v8 = ((var939 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
                            + 256 as ::core::ffi::c_int) as uint16;
                    }
                }
                set_ram_word!(game.layer1_y_pos = v8);
            }
        }
    }
}
pub unsafe fn HandleScrollingWhenTriggeringScrollDown(game: &mut Game) {
    let var939: uint16 = ram_word!(game, layer1_y_pos);
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let prod: uint16 = Mult8x8(
        *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ),
        ram_word!(game, room_width_in_scrolls) as uint8,
    );
    let v1: uint16 = (((ram_word!(game, layer1_x_pos)).wrapping_add(128u16)) as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as uint16;
    let r20: uint16 = (prod).wrapping_add(v1);
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(r20 as isize)
        != 1u8
    {
        v0 = 31 as ::core::ffi::c_int;
    }
    let var933: uint16 = v0 as uint16;
    if ((ram_word!(game, ideal_layer1_ypos)) as i16)
        .wrapping_sub((ram_word!(game, layer1_y_pos)) as i16)
        < 0i16
    {
        set_ram_word!(game.layer1_y_pos = ram_word!(game, ideal_layer1_ypos));
        set_ram_word!(game.layer1_y_subpos = 0 as uint16);
    }
    let v2: uint16 = u16::swap_bytes((ram_word!(game, room_height_in_scrolls)).wrapping_sub(1u16));
    let mut var937: uint16 = (var933).wrapping_add(v2);
    if (var933).wrapping_add(v2) < ram_word!(game, layer1_y_pos)
        || *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20)
            .offset(((ram_word!(game, room_width_in_scrolls)).wrapping_add(r20)) as isize)
            == 0
            && {
                var937 = (var933 as ::core::ffi::c_int
                    + (ram_word!(game, layer1_y_pos) as ::core::ffi::c_int
                        & 0xff00 as ::core::ffi::c_int)) as uint16;
                var937 < ram_word!(game, layer1_y_pos)
            }
    {
        let mut v3: uint16 = (var939 as ::core::ffi::c_int
            - ram_word!(game, absolute_moved_last_frame_y) as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int) as uint16;
        if ((v3) as i16).wrapping_sub((var937) as i16) < 0i16 {
            v3 = var937;
        }
        set_ram_word!(game.layer1_y_pos = v3);
    }
}
pub unsafe fn HandleScrollingWhenTriggeringScrollUp(game: &mut Game) {
    let mut v0: int16 = 0;
    let var939: uint16 = ram_word!(game, layer1_y_pos);
    if ((ram_word!(game, layer1_y_pos)) as i16)
        .wrapping_sub((ram_word!(game, ideal_layer1_ypos)) as i16)
        < 0i16
    {
        set_ram_word!(game.layer1_y_pos = ram_word!(game, ideal_layer1_ypos));
        set_ram_word!(game.layer1_y_subpos = 0 as uint16);
    }
    if (ram_word!(game, layer1_y_pos) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32
    {
        let prod: uint16 = Mult8x8(
            *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ),
            ram_word!(game, room_width_in_scrolls) as uint8,
        );
        v0 = (((ram_word!(game, layer1_x_pos)).wrapping_add(128u16)) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as int16;
        if *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20)
            .offset(((prod).wrapping_add((v0) as u16)) as isize)
            == 0
        {
            let var933: uint16 = ((ram_word!(game, layer1_y_pos) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                + 256 as ::core::ffi::c_int) as uint16;
            let mut v1: uint16 = (ram_word!(game, absolute_moved_last_frame_y)
                as ::core::ffi::c_int
                + var939 as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16;
            if v1 >= var933 {
                v1 = var933;
            }
            set_ram_word!(game.layer1_y_pos = v1);
        }
    } else {
        set_ram_word!(game.layer1_y_pos = 0 as uint16);
    };
}
pub unsafe fn DebugScrollPosSaveLoad(game: &mut Game) {
    if (ram_word!(game, joypad2_new_keys) as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int)
        != 0i32
    {
        let ref mut fresh22 = *((&raw mut game.g_ram as *mut uint8).offset(0x5d3) as *mut uint16);
        *fresh22 = (*fresh22).wrapping_add(1);
    }
    if ram_word!(game, debug_saveload_scrollpos_toggle) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        set_ram_word!(game.layer1_x_pos = ram_word!(game, debug_saved_xscroll));
        set_ram_word!(game.layer1_y_pos = ram_word!(game, debug_saved_yscroll));
    } else {
        set_ram_word!(game.debug_saved_xscroll = ram_word!(game, layer1_x_pos));
        set_ram_word!(game.debug_saved_yscroll = ram_word!(game, layer1_y_pos));
    };
}
pub unsafe fn UploadBackgroundDataColumn(game: &mut Game) {
    UpdateLevelOrBackgroundDataColumn(0x1c as uint16, game);
}
pub unsafe fn UploadLevelDataColumn(game: &mut Game) {
    UpdateLevelOrBackgroundDataColumn(0 as uint16, game);
}
pub unsafe fn UpdateLevelOrBackgroundDataColumn(k: uint16, game: &mut Game) {
    if ram_word!(game, irq_enable_mode7) != 0 {
        return;
    }
    let mut prod: uint16 = Mult8x8(
        ram_word!(game, blocks_to_update_y_block) as uint8,
        ram_word!(game, room_width_in_blocks) as uint8,
    );
    let v1: uint16 = ram_word!(game, blocks_to_update_x_block);
    let mut v2: uint16 = (2 as ::core::ffi::c_int
        * (prod as ::core::ffi::c_int + v1 as ::core::ffi::c_int)
        + 2 as ::core::ffi::c_int) as uint16;
    if k != 0 {
        v2 = (v2).wrapping_add(0x9600u16);
    }
    let r54: *const uint16 = (&raw mut game.g_ram as *mut uint8)
        .offset(0x10000)
        .offset(v2 as ::core::ffi::c_int as isize) as *const uint16;
    let v3: uint16 = (4 as ::core::ffi::c_int
        * ram_word!(game, vram_blocks_to_update_y_block) as ::core::ffi::c_int
        & 0x3c as ::core::ffi::c_int) as uint16;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x958) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = v3;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x956) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) =
        ((v3 as ::core::ffi::c_int ^ 0x3f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
            as uint16;
    prod = Mult8x8(
        ((ram_word!(game, vram_blocks_to_update_y_block)) as u8) & (0xfu8),
        0x40 as uint8,
    );
    let var935: uint16 = (ram_word!(game, vram_blocks_to_update_x_block)) & (0x1fu16);
    let v4: uint16 = (2u16).wrapping_mul(var935);
    let var933: uint16 = (prod).wrapping_add(v4);
    let mut v5: uint16 = addr_unk_605000 as uint16;
    if var935 >= 0x10u16 {
        v5 = addr_unk_6053E0 as uint16;
    }
    if k != 0 {
        v5 = (v5).wrapping_sub(ram_word!(game, size_of_bg2));
    }
    let mut var937: uint16 = v5;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x95a) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = (var933).wrapping_add(v5);
    *(((&raw mut game.g_ram as *mut uint8).offset(0x95c) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = (var935 as ::core::ffi::c_int
        + var935 as ::core::ffi::c_int
        + var937 as ::core::ffi::c_int)
        as uint16;
    let mut v6: uint16 = ((&raw mut game.g_ram as *mut uint8).offset(0xc8c8) as *mut uint16
        as *mut uint8)
        .offset_from(&raw mut game.g_ram as *mut uint8)
        as ::core::ffi::c_long as uint16;
    let mut v7: uint16 = 0 as uint16;
    if k != 0 {
        v6 = (&raw mut game.g_ram as *mut uint8)
            .offset(0xc9d0)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            as uint16;
        v7 = 264 as uint16;
    }
    let v8: uint16 = (*(((&raw mut game.g_ram as *mut uint8).offset(0x956) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16))
        .wrapping_add(v6);
    *(((&raw mut game.g_ram as *mut uint8).offset(0x95e) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = v8;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x960) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = (v8).wrapping_add(64u16);
    var937 = v7;
    let t2: uint16 = k;
    let mut v9: uint16 = 0 as uint16;
    let mut var939: uint16 = 16 as uint16;
    loop {
        let var93B: uint16 =
            *r54.offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        let v10: uint16 = (var93B) & (0x3ffu16);
        let v17: uint16 = v9;
        let v11: uint16 = var937;
        let v12: uint16 = (var93B) & (0xc00u16);
        if (var93B as ::core::ffi::c_int & 0xc00 as ::core::ffi::c_int) != 0i32 {
            if v12 == 1024u16 {
                let v14: uint16 =
                    (var937 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc8c8) as *mut uint16)
                    .offset(v14 as isize) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                    as *mut TileTables))
                    .tables[v10 as usize]
                    .top_right)
                    ^ (0x4000u16);
                *((&raw mut game.g_ram as *mut uint8).offset(0xc908) as *mut uint16)
                    .offset(v14 as isize) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                    as *mut TileTables))
                    .tables[v10 as usize]
                    .top_left)
                    ^ (0x4000u16);
                *((&raw mut game.g_ram as *mut uint8).offset(0xc8c8) as *mut uint16)
                    .offset((v14 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000) as *mut TileTables))
                        .tables[v10 as usize]
                        .bottom_right)
                        ^ (0x4000u16);
                *((&raw mut game.g_ram as *mut uint8).offset(0xc908) as *mut uint16)
                    .offset((v14 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000) as *mut TileTables))
                        .tables[v10 as usize]
                        .bottom_left)
                        ^ (0x4000u16);
            } else {
                let v15: uint16 =
                    (var937 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
                let mut v16: uint16 = 0;
                if v12 == 2048u16 {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc8c8) as *mut uint16)
                        .offset(v15 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa000)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .bottom_left)
                        ^ (0x8000u16);
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc908) as *mut uint16)
                        .offset(v15 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa000)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .bottom_right)
                        ^ (0x8000u16);
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc8c8) as *mut uint16)
                        .offset((v15 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                        ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v10 as usize]
                            .top_left)
                            ^ (0x8000u16);
                    v16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .top_right)
                        ^ (0x8000u16);
                } else {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc8c8) as *mut uint16)
                        .offset(v15 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa000)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .bottom_right)
                        ^ (0xc000u16);
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc908) as *mut uint16)
                        .offset(v15 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa000)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .bottom_left)
                        ^ (0xc000u16);
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc8c8) as *mut uint16)
                        .offset((v15 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                        ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v10 as usize]
                            .top_right)
                            ^ (0xc000u16);
                    v16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .top_left)
                        ^ (0xc000u16);
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0xc908) as *mut uint16)
                    .offset((v15 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) = v16;
            }
        } else {
            let v13: uint16 = (var937 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc8c8) as *mut uint16)
                .offset(v13 as isize) = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                as *mut TileTables))
                .tables[v10 as usize]
                .top_left;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc908) as *mut uint16)
                .offset(v13 as isize) = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                as *mut TileTables))
                .tables[v10 as usize]
                .top_right;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc8c8) as *mut uint16)
                .offset((v13 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xa000) as *mut TileTables)).tables
                    [v10 as usize]
                    .bottom_left;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc908) as *mut uint16)
                .offset((v13 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xa000) as *mut TileTables)).tables
                    [v10 as usize]
                    .bottom_right;
        }
        var937 = (v11).wrapping_add(4u16);
        v9 = (ram_word!(game, room_width_in_blocks) as ::core::ffi::c_int * 2 as ::core::ffi::c_int
            + v17 as ::core::ffi::c_int) as uint16;
        var939 = var939.wrapping_sub(1);
        if (var939 == 0) {
            break;
        }
    }
    let ref mut fresh7 = *(((&raw mut game.g_ram as *mut uint8).offset(0x962) as *mut uint16
        as *mut uint8)
        .offset(t2 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh7 = (*fresh7).wrapping_add(1);
}
pub unsafe fn UpdateBackgroundDataRow(game: &mut Game) {
    UpdateLevelOrBackgroundDataRow(0x1c as uint16, game);
}
pub unsafe fn UpdateLevelDataRow(game: &mut Game) {
    UpdateLevelOrBackgroundDataRow(0 as uint16, game);
}
pub unsafe fn UpdateLevelOrBackgroundDataRow(v0: uint16, game: &mut Game) {
    if ram_word!(game, irq_enable_mode7) != 0 {
        return;
    }
    let mut prod: uint16 = Mult8x8(
        ram_word!(game, blocks_to_update_y_block) as uint8,
        ram_word!(game, room_width_in_blocks) as uint8,
    );
    let v1: uint16 = ram_word!(game, blocks_to_update_x_block);
    let mut v2: uint16 = (2 as ::core::ffi::c_int
        * (prod as ::core::ffi::c_int + v1 as ::core::ffi::c_int)
        + 2 as ::core::ffi::c_int) as uint16;
    if v0 != 0 {
        v2 = (v2).wrapping_sub(27136u16);
    }
    let r54: *const uint16 = (&raw mut game.g_ram as *mut uint8)
        .offset(0x10000)
        .offset(v2 as ::core::ffi::c_int as isize) as *const uint16;
    let mut var933: uint16 = (ram_word!(game, vram_blocks_to_update_x_block)) & (0xfu16);
    *(((&raw mut game.g_ram as *mut uint8).offset(0x964) as *mut uint16 as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = (4 as ::core::ffi::c_int
        * (16 as ::core::ffi::c_int - var933 as ::core::ffi::c_int))
        as uint16;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x966) as *mut uint16 as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = (4 as ::core::ffi::c_int
        * (var933 as ::core::ffi::c_int + 1 as ::core::ffi::c_int))
        as uint16;
    prod = Mult8x8(
        ((ram_word!(game, vram_blocks_to_update_y_block)) as u8) & (0xfu8),
        0x40 as uint8,
    );
    let var935: uint16 = (ram_word!(game, vram_blocks_to_update_x_block)) & (0x1fu16);
    let v3: uint16 = (2u16).wrapping_mul(var935);
    var933 = (prod).wrapping_add(v3);
    let mut var937: uint16 = addr_unk_605400 as uint16;
    let mut v4: uint16 = addr_unk_605000 as uint16;
    if var935 >= 0x10u16 {
        var937 = addr_unk_605000 as uint16;
        v4 = addr_unk_6053E0 as uint16;
    }
    if v0 != 0 {
        v4 = (v4).wrapping_sub(ram_word!(game, size_of_bg2));
    }
    *(((&raw mut game.g_ram as *mut uint8).offset(0x968) as *mut uint16 as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = (var933).wrapping_add(v4);
    let mut v5: uint16 = var937;
    if v0 != 0 {
        v5 = (var937).wrapping_sub(ram_word!(game, size_of_bg2));
    }
    *(((&raw mut game.g_ram as *mut uint8).offset(0x96a) as *mut uint16 as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = (prod).wrapping_add(v5);
    let mut v6: uint16 = ((&raw mut game.g_ram as *mut uint8).offset(0xc948) as *mut uint16
        as *mut uint8)
        .offset_from(&raw mut game.g_ram as *mut uint8)
        as ::core::ffi::c_long as uint16;
    let mut v7: uint16 = 0 as uint16;
    if v0 != 0 {
        v6 = (&raw mut game.g_ram as *mut uint8)
            .offset(0xca50)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            as uint16;
        v7 = 264 as uint16;
    }
    let v8: uint16 = (*(((&raw mut game.g_ram as *mut uint8).offset(0x964) as *mut uint16
        as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16))
        .wrapping_add(v6);
    *(((&raw mut game.g_ram as *mut uint8).offset(0x96c) as *mut uint16 as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = v8;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x96e) as *mut uint16 as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = (v8).wrapping_add(68u16);
    var937 = v7;
    let t2: uint16 = v0;
    let mut v9: uint16 = 0 as uint16;
    let mut var939: uint16 = 17 as uint16;
    loop {
        let var93B: uint16 =
            *r54.offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        let v10: uint16 = (var93B) & (0x3ffu16);
        let v17: uint16 = v9;
        let v11: uint16 = var937;
        let v12: uint16 = (var93B) & (0xc00u16);
        if (var93B as ::core::ffi::c_int & 0xc00 as ::core::ffi::c_int) != 0i32 {
            if v12 == 1024u16 {
                let v14: uint16 =
                    (var937 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc948) as *mut uint16)
                    .offset(v14 as isize) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                    as *mut TileTables))
                    .tables[v10 as usize]
                    .top_right)
                    ^ (0x4000u16);
                *((&raw mut game.g_ram as *mut uint8).offset(0xc948) as *mut uint16)
                    .offset((v14 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000) as *mut TileTables))
                        .tables[v10 as usize]
                        .top_left)
                        ^ (0x4000u16);
                *((&raw mut game.g_ram as *mut uint8).offset(0xc98c) as *mut uint16)
                    .offset(v14 as isize) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                    as *mut TileTables))
                    .tables[v10 as usize]
                    .bottom_right)
                    ^ (0x4000u16);
                *((&raw mut game.g_ram as *mut uint8).offset(0xc98c) as *mut uint16)
                    .offset((v14 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000) as *mut TileTables))
                        .tables[v10 as usize]
                        .bottom_left)
                        ^ (0x4000u16);
            } else {
                let v15: uint16 =
                    (var937 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
                let mut v16: uint16 = 0;
                if v12 == 2048u16 {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc948) as *mut uint16)
                        .offset(v15 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa000)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .bottom_left)
                        ^ (0x8000u16);
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc948) as *mut uint16)
                        .offset((v15 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                        ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v10 as usize]
                            .bottom_right)
                            ^ (0x8000u16);
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc98c) as *mut uint16)
                        .offset(v15 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa000)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .top_left)
                        ^ (0x8000u16);
                    v16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .top_right)
                        ^ (0x8000u16);
                } else {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc948) as *mut uint16)
                        .offset(v15 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa000)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .bottom_right)
                        ^ (0xc000u16);
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc948) as *mut uint16)
                        .offset((v15 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                        ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                            as *mut TileTables))
                            .tables[v10 as usize]
                            .bottom_left)
                            ^ (0xc000u16);
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc98c) as *mut uint16)
                        .offset(v15 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xa000)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .top_right)
                        ^ (0xc000u16);
                    v16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                        as *mut TileTables))
                        .tables[v10 as usize]
                        .top_left)
                        ^ (0xc000u16);
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0xc98c) as *mut uint16)
                    .offset((v15 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) = v16;
            }
        } else {
            let v13: uint16 = (var937 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc948) as *mut uint16)
                .offset(v13 as isize) = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                as *mut TileTables))
                .tables[v10 as usize]
                .top_left;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc948) as *mut uint16)
                .offset((v13 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xa000) as *mut TileTables)).tables
                    [v10 as usize]
                    .top_right;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc98c) as *mut uint16)
                .offset(v13 as isize) = (*((&raw mut game.g_ram as *mut uint8).offset(0xa000)
                as *mut TileTables))
                .tables[v10 as usize]
                .bottom_left;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc98c) as *mut uint16)
                .offset((v13 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xa000) as *mut TileTables)).tables
                    [v10 as usize]
                    .bottom_right;
        }
        var937 = (v11).wrapping_add(4u16);
        v9 = (v17).wrapping_add(2u16);
        var939 = var939.wrapping_sub(1);
        if (var939 == 0) {
            break;
        }
    }
    let ref mut fresh6 = *(((&raw mut game.g_ram as *mut uint8).offset(0x970) as *mut uint16
        as *mut uint8)
        .offset(t2 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh6 = (*fresh6).wrapping_add(1);
}
pub unsafe fn FixDoorsMovingUp(game: &mut Game) {
    set_ram_word!(game.door_transition_frame_counter = 0 as uint16);
    CalculateBgScrollAndLayerPositionBlocks(game);
    UpdatePreviousLayerBlocks(game);
    let ref mut fresh66 = *((&raw mut game.g_ram as *mut uint8).offset(0x901) as *mut uint16);
    *fresh66 = (*fresh66).wrapping_add(1);
    let ref mut fresh67 = *((&raw mut game.g_ram as *mut uint8).offset(0x905) as *mut uint16);
    *fresh67 = (*fresh67).wrapping_add(1);
    DoorTransition_Up(game);
}
static kDoorTransitionSetupFuncs: [Option<Func_V>; 4] = unsafe {
    [
        Some(DoorTransitionScrollingSetup_Right as unsafe fn(&mut Game) -> ()),
        Some(DoorTransitionScrollingSetup_Left as unsafe fn(&mut Game) -> ()),
        Some(DoorTransitionScrollingSetup_Down as unsafe fn(&mut Game) -> ()),
        Some(DoorTransitionScrollingSetup_Up as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn DoorTransitionScrollingSetup(game: &mut Game) {
    set_ram_word!(game.layer1_x_pos = ram_word!(game, door_destination_x_pos));
    set_ram_word!(game.layer1_y_pos = ram_word!(game, door_destination_y_pos));
    kDoorTransitionSetupFuncs[(ram_word!(game, door_direction) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn DoorTransitionScrollingSetup_Right(game: &mut Game) {
    CalculateLayer2Xpos(game);
    let ref mut fresh60 = *((&raw mut game.g_ram as *mut uint8).offset(0x917) as *mut uint16);
    *fresh60 = (*fresh60).wrapping_sub(256u16);
    CalculateLayer2Ypos(game);
    let ref mut fresh61 = *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16);
    *fresh61 = (*fresh61).wrapping_sub(256u16);
    UpdateBgScrollOffsets(game);
    CalculateBgScrollAndLayerPositionBlocks(game);
    UpdatePreviousLayerBlocks(game);
    let ref mut fresh62 = *((&raw mut game.g_ram as *mut uint8).offset(0x8ff) as *mut uint16);
    *fresh62 = (*fresh62).wrapping_sub(1);
    let ref mut fresh63 = *((&raw mut game.g_ram as *mut uint8).offset(0x903) as *mut uint16);
    *fresh63 = (*fresh63).wrapping_sub(1);
    DoorTransition_Right(game);
}
pub unsafe fn DoorTransitionScrollingSetup_Left(game: &mut Game) {
    CalculateLayer2Xpos(game);
    let ref mut fresh56 = *((&raw mut game.g_ram as *mut uint8).offset(0x917) as *mut uint16);
    *fresh56 = (*fresh56).wrapping_add(256u16);
    CalculateLayer2Ypos(game);
    let ref mut fresh57 = *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16);
    *fresh57 = (*fresh57).wrapping_add(256u16);
    UpdateBgScrollOffsets(game);
    CalculateBgScrollAndLayerPositionBlocks(game);
    UpdatePreviousLayerBlocks(game);
    let ref mut fresh58 = *((&raw mut game.g_ram as *mut uint8).offset(0x8ff) as *mut uint16);
    *fresh58 = (*fresh58).wrapping_add(1);
    let ref mut fresh59 = *((&raw mut game.g_ram as *mut uint8).offset(0x903) as *mut uint16);
    *fresh59 = (*fresh59).wrapping_add(1);
    DoorTransition_Left(game);
}
pub unsafe fn DoorTransitionScrollingSetup_Down(game: &mut Game) {
    CalculateLayer2Xpos(game);
    CalculateLayer2Ypos(game);
    let ref mut fresh52 = *((&raw mut game.g_ram as *mut uint8).offset(0x919) as *mut uint16);
    *fresh52 = (*fresh52).wrapping_sub(224u16);
    let ref mut fresh53 = *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16);
    *fresh53 = (*fresh53).wrapping_sub(224u16);
    UpdateBgScrollOffsets(game);
    CalculateBgScrollAndLayerPositionBlocks(game);
    UpdatePreviousLayerBlocks(game);
    let ref mut fresh54 = *((&raw mut game.g_ram as *mut uint8).offset(0x901) as *mut uint16);
    *fresh54 = (*fresh54).wrapping_sub(1);
    let ref mut fresh55 = *((&raw mut game.g_ram as *mut uint8).offset(0x905) as *mut uint16);
    *fresh55 = (*fresh55).wrapping_sub(1);
    DoorTransition_Down(game);
}
pub unsafe fn DoorTransitionScrollingSetup_Up(game: &mut Game) {
    CalculateLayer2Xpos(game);
    let v1: uint16 = ram_word!(game, layer1_y_pos);
    let ref mut fresh46 = *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16);
    *fresh46 = (*fresh46).wrapping_add(31u16);
    CalculateLayer2Ypos(game);
    let ref mut fresh47 = *((&raw mut game.g_ram as *mut uint8).offset(0x919) as *mut uint16);
    *fresh47 = (*fresh47).wrapping_add(224u16);
    set_ram_word!(game.layer1_y_pos = (v1).wrapping_add(256u16));
    UpdateBgScrollOffsets(game);
    let ref mut fresh48 = *((&raw mut game.g_ram as *mut uint8).offset(0x929) as *mut uint16);
    *fresh48 = (*fresh48).wrapping_add(32u16);
    CalculateBgScrollAndLayerPositionBlocks(game);
    UpdatePreviousLayerBlocks(game);
    let ref mut fresh49 = *((&raw mut game.g_ram as *mut uint8).offset(0x901) as *mut uint16);
    *fresh49 = (*fresh49).wrapping_add(1);
    let ref mut fresh50 = *((&raw mut game.g_ram as *mut uint8).offset(0x905) as *mut uint16);
    *fresh50 = (*fresh50).wrapping_add(1);
    let ref mut fresh51 = *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16);
    *fresh51 = (*fresh51).wrapping_sub(1);
    DoorTransition_Up(game);
}
pub unsafe fn UpdatePreviousLayerBlocks(game: &mut Game) {
    set_ram_word!(game.previous_layer1_x_block = ram_word!(game, layer1_x_block));
    set_ram_word!(game.previous_layer2_x_block = ram_word!(game, layer2_x_block));
    set_ram_word!(game.previous_layer1_y_block = ram_word!(game, layer1_y_block));
    set_ram_word!(game.previous_layer2_y_block = ram_word!(game, layer2_y_block));
}
pub unsafe fn UpdateBgScrollOffsets(game: &mut Game) {
    set_ram_word!(
        game.bg1_x_offset =
            (ram_word!(game, reg_BG1HOFS)).wrapping_sub(ram_word!(game, layer1_x_pos))
    );
    set_ram_word!(
        game.bg1_y_offset =
            (ram_word!(game, reg_BG1VOFS)).wrapping_sub(ram_word!(game, layer1_y_pos))
    );
    set_ram_word!(
        game.bg2_x_scroll =
            (ram_word!(game, reg_BG2HOFS)).wrapping_sub(ram_word!(game, layer1_x_pos))
    );
    set_ram_word!(
        game.bg2_y_scroll =
            (ram_word!(game, reg_BG2VOFS)).wrapping_sub(ram_word!(game, layer1_y_pos))
    );
}
static kDoorTransitionFuncs: [Option<Func_U8>; 4] = unsafe {
    [
        Some(DoorTransition_Right as unsafe fn(&mut Game) -> uint8),
        Some(DoorTransition_Left as unsafe fn(&mut Game) -> uint8),
        Some(DoorTransition_Down as unsafe fn(&mut Game) -> uint8),
        Some(DoorTransition_Up as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn Irq_FollowDoorTransition(game: &mut Game) {
    if kDoorTransitionFuncs
        [(ram_word!(game, door_direction) as ::core::ffi::c_int & 3 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(game)
        != 0
    {
        set_ram_word!(game.layer1_x_pos = ram_word!(game, door_destination_x_pos));
        set_ram_word!(game.layer1_y_pos = ram_word!(game, door_destination_y_pos));
        let ref mut fresh68 = *((&raw mut game.g_ram as *mut uint8).offset(0x931) as *mut uint16);
        *fresh68 = (*fresh68) | (0x8000u16);
    }
}
pub unsafe fn DoorTransition_Right(game: &mut Game) -> uint8 {
    let v2: uint16 = ram_word!(game, door_transition_frame_counter);
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xaf8) as *mut uint16,
        (ram_word!(game, samus_door_transition_speed) as uint32) << 16 as ::core::ffi::c_int
            | ram_word!(game, samus_door_transition_subspeed) as uint32,
    );
    set_ram_word!(game.samus_prev_x_pos = ram_word!(game, samus_x_pos));
    let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16);
    *fresh10 = (*fresh10).wrapping_add(4u16);
    let ref mut fresh11 = *((&raw mut game.g_ram as *mut uint8).offset(0x917) as *mut uint16);
    *fresh11 = (*fresh11).wrapping_add(4u16);
    UpdateScrollVarsUpdateMap(game);
    set_ram_word!(game.door_transition_frame_counter = (v2).wrapping_add(1u16));
    if v2 != 63u16 {
        return 0 as uint8;
    }
    UpdateScrollVarsUpdateMap(game);
    1 as uint8
}
pub unsafe fn DoorTransition_Left(game: &mut Game) -> uint8 {
    let v2: uint16 = ram_word!(game, door_transition_frame_counter);
    AddToHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xaf8) as *mut uint16,
        -((ram_word!(game, samus_door_transition_speed) as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int
            | ram_word!(game, samus_door_transition_subspeed) as ::core::ffi::c_int)
            as uint32,
    );
    set_ram_word!(game.samus_prev_x_pos = ram_word!(game, samus_x_pos));
    let ref mut fresh8 = *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16);
    *fresh8 = (*fresh8).wrapping_sub(4u16);
    let ref mut fresh9 = *((&raw mut game.g_ram as *mut uint8).offset(0x917) as *mut uint16);
    *fresh9 = (*fresh9).wrapping_sub(4u16);
    UpdateScrollVarsUpdateMap(game);
    set_ram_word!(game.door_transition_frame_counter = (v2).wrapping_add(1u16));
    (v2 == 63u16) as uint8
}
pub unsafe fn DoorTransition_Down(game: &mut Game) -> uint8 {
    let v6: uint16 = ram_word!(game, door_transition_frame_counter);
    if ram_word!(game, door_transition_frame_counter) != 0 {
        if ram_word!(game, door_transition_frame_counter) < 0x39u16 {
            AddToHiLo(
                (&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16,
                (&raw mut game.g_ram as *mut uint8).offset(0xafc) as *mut uint16,
                (ram_word!(game, samus_door_transition_speed) as uint32)
                    << 16 as ::core::ffi::c_int
                    | ram_word!(game, samus_door_transition_subspeed) as uint32,
            );
            set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
            let ref mut fresh0 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16);
            *fresh0 = (*fresh0).wrapping_add(4u16);
            let ref mut fresh1 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x919) as *mut uint16);
            *fresh1 = (*fresh1).wrapping_add(4u16);
            UpdateScrollVarsUpdateMap(game);
        }
    } else {
        let v5: uint16 = ram_word!(game, reg_BG1VOFS);
        let v4: uint16 = ram_word!(game, reg_BG2VOFS);
        let v3: uint16 = ram_word!(game, layer1_y_pos);
        let ref mut fresh2 = *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16);
        *fresh2 = (*fresh2).wrapping_sub(15u16);
        let v2: uint16 = ram_word!(game, layer2_y_pos);
        let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8).offset(0x919) as *mut uint16);
        *fresh3 = (*fresh3).wrapping_sub(15u16);
        CalculateBgScrollAndLayerPositionBlocks(game);
        UpdatePreviousLayerBlocks(game);
        let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8).offset(0x901) as *mut uint16);
        *fresh4 = (*fresh4).wrapping_sub(1);
        let ref mut fresh5 = *((&raw mut game.g_ram as *mut uint8).offset(0x905) as *mut uint16);
        *fresh5 = (*fresh5).wrapping_sub(1);
        UpdateScrollVarsUpdateMap(game);
        set_ram_word!(game.layer2_y_pos = v2);
        set_ram_word!(game.layer1_y_pos = v3);
        set_ram_word!(game.reg_BG2VOFS = v4);
        set_ram_word!(game.reg_BG1VOFS = v5);
    }
    set_ram_word!(game.door_transition_frame_counter = (v6).wrapping_add(1u16));
    if (v6).wrapping_add(1u16) < 0x39u16 {
        return 0 as uint8;
    }
    UpdateScrollVarsUpdateMap(game);
    1 as uint8
}
pub unsafe fn DoorTransition_Up(game: &mut Game) -> uint8 {
    let v6: uint16 = ram_word!(game, door_transition_frame_counter);
    if ram_word!(game, door_transition_frame_counter) != 0 {
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xafc) as *mut uint16,
            -((ram_word!(game, samus_door_transition_speed) as ::core::ffi::c_int)
                << 16 as ::core::ffi::c_int
                | ram_word!(game, samus_door_transition_subspeed) as ::core::ffi::c_int)
                as uint32,
        );
        set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
        let ref mut fresh12 = *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16);
        *fresh12 = (*fresh12).wrapping_sub(4u16);
        let ref mut fresh13 = *((&raw mut game.g_ram as *mut uint8).offset(0x919) as *mut uint16);
        *fresh13 = (*fresh13).wrapping_sub(4u16);
        if ram_word!(game, door_transition_frame_counter) >= 5u16 {
            UpdateScrollVarsUpdateMap(game);
        } else {
            set_ram_word!(
                game.reg_BG1HOFS =
                    (ram_word!(game, bg1_x_offset)).wrapping_add(ram_word!(game, layer1_x_pos))
            );
            set_ram_word!(
                game.reg_BG1VOFS =
                    (ram_word!(game, bg1_y_offset)).wrapping_add(ram_word!(game, layer1_y_pos))
            );
            set_ram_word!(
                game.reg_BG2HOFS =
                    (ram_word!(game, bg2_x_scroll)).wrapping_add(ram_word!(game, layer2_x_pos))
            );
            set_ram_word!(
                game.reg_BG2VOFS =
                    (ram_word!(game, bg2_y_scroll)).wrapping_add(ram_word!(game, layer2_y_pos))
            );
        }
    } else {
        let v5: uint16 = ram_word!(game, reg_BG1VOFS);
        let v4: uint16 = ram_word!(game, reg_BG2VOFS);
        let v3: uint16 = ram_word!(game, layer1_y_pos);
        let ref mut fresh14 = *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16);
        *fresh14 = (*fresh14).wrapping_sub(16u16);
        let v2: uint16 = ram_word!(game, layer2_y_pos);
        let ref mut fresh15 = *((&raw mut game.g_ram as *mut uint8).offset(0x919) as *mut uint16);
        *fresh15 = (*fresh15).wrapping_sub(16u16);
        CalculateBgScrollAndLayerPositionBlocks(game);
        UpdatePreviousLayerBlocks(game);
        let ref mut fresh16 = *((&raw mut game.g_ram as *mut uint8).offset(0x901) as *mut uint16);
        *fresh16 = (*fresh16).wrapping_add(1);
        let ref mut fresh17 = *((&raw mut game.g_ram as *mut uint8).offset(0x905) as *mut uint16);
        *fresh17 = (*fresh17).wrapping_add(1);
        UpdateScrollVarsUpdateMap(game);
        set_ram_word!(game.layer2_y_pos = v2);
        set_ram_word!(game.layer1_y_pos = v3);
        set_ram_word!(game.reg_BG2VOFS = v4);
        set_ram_word!(game.reg_BG1VOFS = v5);
    }
    set_ram_word!(game.door_transition_frame_counter = (v6).wrapping_add(1u16));
    (v6 == 56u16) as uint8
}
pub unsafe fn ConfigureMode7RotationMatrix(game: &mut Game) {
    if ram_word!(game, irq_enable_mode7) != 0
        && (ram_word!(game, nmi_frame_counter_word) as ::core::ffi::c_int & 7 as ::core::ffi::c_int)
            == 0i32
    {
        set_ram_word!(
            game.reg_M7B = kSinCosTable8bit_Sext[(ram_word!(game, mode7_rotation_angle) as uint8
                as ::core::ffi::c_int
                + 64 as ::core::ffi::c_int)
                as usize] as uint16
        );
        set_ram_word!(game.reg_M7C = -(ram_word!(game, reg_M7B) as ::core::ffi::c_int) as uint16);
        set_ram_word!(
            game.reg_M7A = kSinCosTable8bit_Sext[((((ram_word!(game, mode7_rotation_angle)) as u8)
                .wrapping_add(64u8))
                as ::core::ffi::c_int
                + 64 as ::core::ffi::c_int)
                as usize] as uint16
        );
        set_ram_word!(game.reg_M7D = ram_word!(game, reg_M7A));
        let ref mut fresh21 = *((&raw mut game.g_ram as *mut uint8).offset(0x785) as *mut uint16);
        *fresh21 = (*fresh21).wrapping_add(1);
    }
}

unsafe fn DecompNextByte(game: &mut Game) -> uint8 {
    let b: uint8 = *RomPtr(game.decompress_src, game);
    let fresh31 = game.decompress_src;
    game.decompress_src = game.decompress_src.wrapping_add(1);
    if (fresh31 & 0xffff as uint32) == 0xffffu32 {
        game.decompress_src = game.decompress_src.wrapping_add(0x8000 as uint32);
    }
    b
}
pub unsafe fn DecompressToMem(src: uint32, decompress_dst: *mut uint8, game: &mut Game) {
    game.decompress_src = src;
    let mut src_pos: ::core::ffi::c_int = 0;
    let mut dst_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        let mut len: ::core::ffi::c_int = 0;
        let mut cmd: uint8 = 0;
        let mut b: uint8 = 0;
        b = DecompNextByte(game);
        if b == 0xffu8 {
            break;
        }
        if (b as ::core::ffi::c_int & 0xe0 as ::core::ffi::c_int) == 0xe0i32 {
            cmd = (8 as ::core::ffi::c_int * b as ::core::ffi::c_int & 0xe0 as ::core::ffi::c_int)
                as uint8;
            len = ((b as ::core::ffi::c_int & 3 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
                | DecompNextByte(game) as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int;
        } else {
            cmd = (b) & (0xe0u8);
            len = (b as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int;
        }
        if cmd as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
            let want_xor: uint8 = (if cmd as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int != 0 {
                0xff as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as uint8;
            if cmd >= 0xc0u8 {
                src_pos = dst_pos - DecompNextByte(game) as ::core::ffi::c_int;
            } else {
                src_pos = DecompNextByte(game) as ::core::ffi::c_int;
                src_pos += DecompNextByte(game) as ::core::ffi::c_int * 256 as ::core::ffi::c_int;
            }
            loop {
                let fresh23 = src_pos;
                src_pos = src_pos + 1;
                b = (*decompress_dst.offset(fresh23 as isize)) ^ (want_xor);
                let fresh24 = dst_pos;
                dst_pos = dst_pos + 1;
                *decompress_dst.offset(fresh24 as isize) = b;
                len -= 1;
                if (len == 0) {
                    break;
                }
            }
        } else {
            match cmd as ::core::ffi::c_int {
                32 => {
                    b = DecompNextByte(game);
                    loop {
                        let fresh25 = dst_pos;
                        dst_pos = dst_pos + 1;
                        *decompress_dst.offset(fresh25 as isize) = b;
                        len -= 1;
                        if (len == 0) {
                            break;
                        }
                    }
                }
                64 => {
                    b = DecompNextByte(game);
                    let b2: uint8 = DecompNextByte(game);
                    loop {
                        let fresh26 = dst_pos;
                        dst_pos = dst_pos + 1;
                        *decompress_dst.offset(fresh26 as isize) = b;
                        len -= 1;
                        if len == 0 {
                            break;
                        }
                        let fresh27 = dst_pos;
                        dst_pos = dst_pos + 1;
                        *decompress_dst.offset(fresh27 as isize) = b2;
                        len -= 1;
                        if (len == 0) {
                            break;
                        }
                    }
                }
                96 => {
                    b = DecompNextByte(game);
                    loop {
                        let fresh28 = b;
                        b = b.wrapping_add(1);
                        let fresh29 = dst_pos;
                        dst_pos = dst_pos + 1;
                        *decompress_dst.offset(fresh29 as isize) = fresh28;
                        len -= 1;
                        if (len == 0) {
                            break;
                        }
                    }
                }
                _ => loop {
                    b = DecompNextByte(game);
                    let fresh30 = dst_pos;
                    dst_pos = dst_pos + 1;
                    *decompress_dst.offset(fresh30 as isize) = b;
                    len -= 1;
                    if (len == 0) {
                        break;
                    }
                },
            }
        }
    }
}
unsafe fn ReadPpuByte(addr: uint16, game: &mut Game) -> uint8 {
    WriteRegWord(
        VMADDL as uint16,
        (addr as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16,
        game,
    );
    ReadRegWord(RDVRAML as uint16, game);
    let data: uint16 = ReadRegWord(RDVRAML as uint16, game);
    ((if addr as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        (data as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
    } else {
        data as ::core::ffi::c_int
    }) as uint8)
}
pub unsafe fn DecompressToVRAM(src: uint32, dst_addr: uint16, game: &mut Game) {
    game.decompress_src = src;
    let mut src_pos: ::core::ffi::c_int = 0;
    let mut dst_pos: ::core::ffi::c_int = dst_addr as ::core::ffi::c_int;
    loop {
        let mut len: ::core::ffi::c_int = 0;
        let mut b: uint8 = DecompNextByte(game);
        let mut cmd: uint8 = 0;
        if b == 0xffu8 {
            break;
        }
        if (b as ::core::ffi::c_int & 0xe0 as ::core::ffi::c_int) == 0xe0i32 {
            cmd = (8 as ::core::ffi::c_int * b as ::core::ffi::c_int & 0xe0 as ::core::ffi::c_int)
                as uint8;
            len = ((b as ::core::ffi::c_int & 3 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
                | DecompNextByte(game) as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int;
        } else {
            cmd = (b) & (0xe0u8);
            len = (b as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int;
        }
        if cmd as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
            let want_xor: uint8 = (if cmd as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int != 0 {
                0xff as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as uint8;
            if cmd >= 0xc0u8 {
                src_pos = dst_pos - DecompNextByte(game) as ::core::ffi::c_int;
            } else {
                src_pos = DecompNextByte(game) as ::core::ffi::c_int;
                src_pos += DecompNextByte(game) as ::core::ffi::c_int * 256 as ::core::ffi::c_int;
                src_pos += dst_addr as ::core::ffi::c_int;
            }
            loop {
                let fresh32 = src_pos;
                src_pos = src_pos + 1;
                b = (ReadPpuByte(fresh32 as uint16, game)) ^ (want_xor);
                WriteRegWord(
                    VMADDL as uint16,
                    (dst_pos >> 1 as ::core::ffi::c_int) as uint16,
                    game,
                );
                let fresh33 = dst_pos;
                dst_pos = dst_pos + 1;
                WriteReg(
                    (VMDATAL as ::core::ffi::c_int + (fresh33 & 1 as ::core::ffi::c_int)) as uint16,
                    b,
                    game,
                );
                len -= 1;
                if (len == 0) {
                    break;
                }
            }
        } else {
            match cmd as ::core::ffi::c_int {
                32 => {
                    b = DecompNextByte(game);
                    loop {
                        let fresh34 = dst_pos;
                        dst_pos = dst_pos + 1;
                        WriteReg(
                            (VMDATAL as ::core::ffi::c_int + (fresh34 & 1 as ::core::ffi::c_int))
                                as uint16,
                            b,
                            game,
                        );
                        len -= 1;
                        if (len == 0) {
                            break;
                        }
                    }
                }
                64 => {
                    b = DecompNextByte(game);
                    let b2: uint8 = DecompNextByte(game);
                    loop {
                        let fresh35 = dst_pos;
                        dst_pos = dst_pos + 1;
                        WriteReg(
                            (VMDATAL as ::core::ffi::c_int + (fresh35 & 1 as ::core::ffi::c_int))
                                as uint16,
                            b,
                            game,
                        );
                        len -= 1;
                        if len == 0 {
                            break;
                        }
                        let fresh36 = dst_pos;
                        dst_pos = dst_pos + 1;
                        WriteReg(
                            (VMDATAL as ::core::ffi::c_int + (fresh36 & 1 as ::core::ffi::c_int))
                                as uint16,
                            b2,
                            game,
                        );
                        len -= 1;
                        if (len == 0) {
                            break;
                        }
                    }
                }
                96 => {
                    b = DecompNextByte(game);
                    loop {
                        let fresh37 = dst_pos;
                        dst_pos = dst_pos + 1;
                        let fresh38 = b;
                        b = b.wrapping_add(1);
                        WriteReg(
                            (VMDATAL as ::core::ffi::c_int + (fresh37 & 1 as ::core::ffi::c_int))
                                as uint16,
                            fresh38,
                            game,
                        );
                        len -= 1;
                        if (len == 0) {
                            break;
                        }
                    }
                }
                _ => loop {
                    b = DecompNextByte(game);
                    let fresh39 = dst_pos;
                    dst_pos = dst_pos + 1;
                    let fresh40 = b;
                    b = b.wrapping_add(1);
                    WriteReg(
                        (VMDATAL as ::core::ffi::c_int + (fresh39 & 1 as ::core::ffi::c_int))
                            as uint16,
                        fresh40,
                        game,
                    );
                    len -= 1;
                    if (len == 0) {
                        break;
                    }
                },
            }
        }
    }
}
pub unsafe fn LoadFromLoadStation(game: &mut Game) {
    set_ram_word!(game.save_station_lockout_flag = 1 as uint16);
    let L: *const LoadStationList = RomPtr_80(
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x80c4b5)) as *const uint8 as *mut uint16)
            .offset(ram_word!(game, area_index) as isize) as ::core::ffi::c_int
            + 14 as ::core::ffi::c_int * ram_word!(game, load_station_index) as ::core::ffi::c_int)
            as uint16_t,
        game,
    ) as *mut LoadStationList;
    set_ram_word!(game.room_ptr = (*L).room_ptr_);
    set_ram_word!(game.door_def_ptr = (*L).door_ptr);
    let ref mut fresh77 = *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16);
    *fresh77 = (*L).screen_x_pos;
    set_ram_word!(game.bg1_x_offset = *fresh77);
    let ref mut fresh78 = *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16);
    *fresh78 = (*L).screen_y_pos;
    set_ram_word!(game.bg1_y_offset = *fresh78);
    set_ram_word!(
        game.samus_y_pos = (ram_word!(game, layer1_y_pos)).wrapping_add((*L).samus_y_offset)
    );
    set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
    set_ram_word!(
        game.samus_x_pos = (ram_word!(game, layer1_x_pos) as ::core::ffi::c_int
            + 128 as ::core::ffi::c_int
            + (*L).samus_x_offset as ::core::ffi::c_int) as uint16
    );
    set_ram_word!(game.samus_prev_x_pos = ram_word!(game, samus_x_pos));
    set_ram_word!(game.reg_BG1HOFS = 0 as uint16);
    set_ram_word!(game.reg_BG1VOFS = 0 as uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16 as *mut uint8).offset(0) =
        (*get_RoomDefHeader(ram_word!(game, room_ptr), game)).area_index_;
    *((&raw mut game.g_ram as *mut uint8).offset(0x5f7) as *mut uint16 as *mut uint8).offset(0) =
        0 as uint8;
}
pub unsafe fn SetElevatorsAsUsed(game: &mut Game) {
    let v0: *const uint8 = RomPtr_80(
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x80cd46)) as *const uint8 as *mut uint16)
            .offset(ram_word!(game, area_index) as isize) as ::core::ffi::c_int
            + 4 as ::core::ffi::c_int
                * ((ram_word!(game, elevator_door_properties_orientation) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int)
                    - 1 as ::core::ffi::c_int)) as uint16_t,
        game,
    );
    let ref mut fresh92 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd8f8)
        .offset(*v0.offset(0) as isize);
    *fresh92 = (*fresh92) | (*v0.offset(1));
    let ref mut fresh93 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd8f8)
        .offset(*v0.offset(2) as isize);
    *fresh93 = (*fresh93) | (*v0.offset(3));
}
