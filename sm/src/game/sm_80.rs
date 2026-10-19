pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::game_states::*;
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::AddToHiLo;
pub use crate::snes::registers::*;
use crate::types::TileDrawOrder;
use ::core::ffi::{c_char, c_long, c_void};
use ::core::mem::size_of;

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
    fn memcpy(__dest: *mut c_void, __src: *const c_void, __n: size_t) -> *mut c_void;
    fn memset(__s: *mut c_void, __c: i32, __n: size_t) -> *mut c_void;
    fn memcmp(__s1: *const c_void, __s2: *const c_void, __n: size_t) -> i32;
    fn printf(__format: *const c_char, ...) -> i32;

}
pub use crate::types::{
    size_t, CoroutineRet, Func_U8, Func_V, LongPtr, OamEnt, SnesRegs, VoidP, VramWriteEntry,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct LoadStationList {
    pub room_ptr_: VoidP,
    pub door_ptr: VoidP,
    pub door_bts: u16,
    pub screen_x_pos: u16,
    pub screen_y_pos: u16,
    pub samus_y_offset: u16,
    pub samus_x_offset: u16,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct SamusTileAnimationTileDefs {
    pub src: LongPtr,
    pub part1_size: u16,
    pub part2_size: u16,
}
pub use crate::types::Buttons;
pub use crate::types::{GameState, Mode7VramWriteQueue, RoomDefHeader};
pub use crate::types::{
    Ram4000, Ram4000_Backups, StartDmaCopy, TileTable, TileTables, VramReadQueueEnt,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Mode7CgvmWriteQueue {
    pub tag: u8,
    pub src_addr: LongPtr,
    pub count: u16,
    pub vram_addr: u16,
    pub vmain: u8,
}
pub use crate::types::Consts_60;
pub use crate::types::Consts_80;
pub const addr_word_80A060: Consts_80 = 41056;
pub const addrl_kDigitTilesetsWeapon: Consts_80 = 8429011;
pub const addrl_kDigitTilesetsHealth: Consts_80 = 8428991;
pub const addr_kHudTilemaps_Missiles: Consts_80 = 39331;
pub const addr_kHudTilemaps_AutoReserve: Consts_80 = 39307;
pub use crate::types::SpawnHardcodedPlmArgs;
const RomPtr_80: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x80>;
const RomPtr_92: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x92>;
use crate::rtl::rom_ptr_with_bank as RomPtrWithBank;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0i32 + a) as isize];
}
#[inline]
unsafe fn get_RoomDefHeader(a: u16, game: &mut Game) -> *mut RoomDefHeader {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomDefHeader
}
pub const fnDoorTransition_FadeInScreenAndFinish: i32 = 0x82e737;
pub unsafe fn APU_UploadBank(addr: u32, game: &mut Game) {
    if !game.g_use_my_apu_code {
        crate::native_audio::upload(addr, game);
        return;
    }
    RtlApuUpload(RomPtr(addr, game), game);
}
pub unsafe fn NextRandom(game: &mut Game) -> u16 {
    let mut RegWord: u16 =
        ((*(ram_ptr!(game, u16; 0x5e5) as *mut u8).offset(0)) as u16).wrapping_mul(5u16);
    let Reg: u8 = (*(ram_ptr!(game, u16; 0x5e5) as *mut u8).offset(1isize)).wrapping_mul(5u8);
    let carry: i32 = *(&raw mut RegWord as *mut u8).offset(1isize) as i32 + Reg as i32 + 1i32;
    *(&raw mut RegWord as *mut u8).offset(1isize) = carry as u8;
    let result: u16 = ((carry >> 8i32) + RegWord as i32 + 17i32) as u16;
    set_ram_word!(game.random_number = result);
    result
}
pub unsafe fn ReleaseButtonsFilter(v0: u16, game: &mut Game) {
    set_ram_word!(game.timed_held_input_timer_reset = v0);
    let v1: bool = !(ram_word!(game, joypad1_newkeys) as i32) as u16 as i32
        & ram_word!(game, joypad1_lastkeys) as i32
        == ram_word!(game, joypad_released_keys) as i32;
    set_ram_word!(
        game.joypad_released_keys = (!(ram_word!(game, joypad1_newkeys) as i32)
            & ram_word!(game, joypad1_lastkeys) as i32) as u16
    );
    if !v1 {
        set_ram_word!(game.timed_held_input_timer = ram_word!(game, timed_held_input_timer_reset));
        set_ram_word!(game.timed_held_input = 0u16);
    } else {
        if (update_ram_word!(game.timed_held_input_timer, wrapping_sub(1), reread) as i32
            & 0x8000i32)
            == 0i32
        {
            set_ram_word!(game.timed_held_input = 0u16);
        } else {
            set_ram_word!(game.timed_held_input_timer = 0u16);
            set_ram_word!(game.previous_timed_held_input = ram_word!(game, timed_held_input));
            set_ram_word!(
                game.timed_held_input = (!(ram_word!(game, joypad1_newkeys) as i32)
                    & ram_word!(game, joypad1_lastkeys) as i32)
                    as u16
            );
        }
    }
    set_ram_word!(
        game.newly_held_down_timed_held_input = (ram_word!(game, timed_held_input) as i32
            & (ram_word!(game, previous_timed_held_input) as i32
                ^ ram_word!(game, timed_held_input) as i32))
            as u16
    );
}
pub fn PrepareBitAccess(a: u16, game: &mut Game) -> u16 {
    let bitmask = 1u16 << (a & 7);
    game.g_ram[0x5e7..0x5e9].copy_from_slice(&bitmask.to_le_bytes());
    a >> 3
}
pub unsafe fn SetBossBitForCurArea(a: u16, game: &mut Game) {
    game.g_ram[0x5e7] = a as u8; // The ROM stores the mask with an 8-bit accumulator.
    let ref mut fresh91 = *ram_ptr!(game; 0xd828, ram_word!(game, area_index) as isize);
    *fresh91 = (*fresh91) | ((a) as u8);
}
pub unsafe fn ClearBossBitForCurArea(a: u16, game: &mut Game) {
    let ref mut fresh19 = *ram_ptr!(game; 0xd828, ram_word!(game, area_index) as isize);
    *fresh19 = (*fresh19 as i32 & !(a as i32)) as u8;
}
pub unsafe fn CheckBossBitForCurArea(a: u16, game: &mut Game) -> u8 {
    game.g_ram[0x5e7] = a as u8; // The ROM stores the mask with an 8-bit accumulator.
    ((a as i32 & *ram_ptr!(game; 0xd828, ram_word!(game, area_index) as isize) as i32) != 0i32)
        as u8
}
pub unsafe fn SetEventHappened(a: u16, game: &mut Game) {
    let v1: u16 = PrepareBitAccess(a, game);
    let ref mut fresh94 = *ram_ptr!(game; 0xd820, v1 as isize);
    *fresh94 = (*fresh94) | ((ram_word!(game, bitmask)) as u8);
}
pub unsafe fn ClearEventHappened(v0: u16, game: &mut Game) {
    let v1: u16 = PrepareBitAccess(v0, game);
    let ref mut fresh20 = *ram_ptr!(game; 0xd820, v1 as isize);
    *fresh20 = (*fresh20 as i32 & !(ram_word!(game, bitmask) as i32)) as u8;
}
pub unsafe fn CheckEventHappened(a: u16, game: &mut Game) -> u16 {
    let idx: u16 = PrepareBitAccess(a, game);
    ((ram_word!(game, bitmask) as i32 & *ram_ptr!(game; 0xd820, idx as isize) as i32) != 0i32)
        as u16
}
pub unsafe fn CopySuperMetroidString(game: &mut Game) {
    memcpy(
        game.g_sram.offset(0x1fe0) as *mut c_void,
        b"supermetroid\0" as *const u8 as *const c_char as *const c_void,
        12 as size_t,
    );
    RtlWriteSram(game);
}
pub unsafe fn VerifySRAM(game: &mut Game) {
    set_ram_word!(game.num_demo_sets = 3u16);
    if LoadFromSram(0u16, game) != 0u8
        && LoadFromSram(1u16, game) != 0u8
        && LoadFromSram(2u16, game) != 0u8
    {
        memcpy(
            game.g_sram.offset(0x1fe0) as *mut c_void,
            b"madadameyohn\0" as *const u8 as *const c_char as *const c_void,
            12 as size_t,
        );
        RtlWriteSram(game);
    } else if memcmp(
        game.g_sram.offset(0x1fe0) as *const c_void,
        b"supermetroid\0" as *const u8 as *const c_char as *const c_void,
        12 as size_t,
    ) == 0
    {
        set_ram_word!(game.num_demo_sets = 4u16);
    }
}
pub unsafe fn Multiply16x16(a: u16, j: u16, game: &mut Game) -> u32 {
    game.g_ram[0x5e9..0x5eb].copy_from_slice(&a.to_le_bytes());
    game.g_ram[0x5eb..0x5ed].copy_from_slice(&j.to_le_bytes());
    let result: u32 = (a as u32).wrapping_mul(j as u32);
    result
}
pub unsafe fn WaitForNMI_Async(game: &mut Game) -> CoroutineRet {
    if *ram_ptr!(game; 0x782) != 0 {
        *ram_ptr!(game; 0x782) = 0u8;
        return 0 as CoroutineRet;
    }
    *ram_ptr!(game; 0x5b4) = 1u8;
    *ram_ptr!(game; 0x782) = 1u8;
    1 as CoroutineRet
}
pub unsafe fn WaitForNMI_NoUpdate_Async(game: &mut Game) -> CoroutineRet {
    if *ram_ptr!(game; 0x782) != 0 {
        *ram_ptr!(game; 0x782) = 0u8;
        return 0 as CoroutineRet;
    }
    *ram_ptr!(game; 0x782) = 1u8;
    1 as CoroutineRet
}
pub unsafe fn EnableNMI(game: &mut Game) {
    let v0: u8 = (*ram_ptr!(game; 0x84)) | (0x80u8);
    WriteReg(NMITIMEN as u16, (*ram_ptr!(game; 0x84)) | (0x80u8), game);
    *ram_ptr!(game; 0x84) = v0;
}
pub unsafe fn DisableNMI(game: &mut Game) {
    let v0: u8 = (*ram_ptr!(game; 0x84)) & (0x7fu8);
    WriteReg(NMITIMEN as u16, (*ram_ptr!(game; 0x84)) & (0x7fu8), game);
    *ram_ptr!(game; 0x84) = v0;
}
pub unsafe fn ScreenOff(game: &mut Game) {
    let ref mut fresh89 = *ram_ptr!(game; 0x51);
    *fresh89 = (*fresh89) | (0x80u8);
}
pub unsafe fn ScreenOn(game: &mut Game) {
    let ref mut fresh90 = *ram_ptr!(game; 0x51);
    *fresh90 = (*fresh90 as i32 & !(0x80i32)) as u8;
}
pub unsafe fn memset7E(mut k: *mut u16, a: u16, mut j: u16) {
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
    let mut _state_variable_: *mut u8 = ram_ptr!(game; 0x611);
    let mut bak: u16 = 0;
    let mut current_block_63: u64;
    match *_state_variable_ as i32 {
        1 => {
            crate::rtl::write_register_and_mirror(MEMSEL as u16, 1, 0x86, false, game);
            bak = ram_word!(game, bug_fix_counter);
            memset(
                &raw mut game.g_ram as *mut u8 as *mut c_void,
                0i32,
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
        let mut _coret_: u8 = InitializeIoDisplayLogo_Async(game);
        if _coret_ != 0 {
            *_state_variable_ = 2u8;
            return _coret_;
        }
        current_block_63 = 3640593987805443782;
    }
    if current_block_63 == 3640593987805443782 {
        APU_UploadBank(0xcf8000u32, game);
        WriteReg(INIDISP as u16, 0x8fu8, game);
        bak = ram_word!(game, bug_fix_counter);
        memset(
            &raw mut game.g_ram as *mut u8 as *mut c_void,
            0i32,
            0x10000i32 as size_t,
        );
        set_ram_word!(game.bug_fix_counter = bak);
        crate::rtl::write_register_and_mirror(NMITIMEN as u16, 0, 0x84, false, game);
        *ram_ptr!(game; 0x51) = 0x8fu8;
        InitializeCpuIoRegs(game);
        InitializePpuIoRegs(game);
        WriteLotsOf0x1c2f(game);
        *ram_ptr!(game; 0x643, 0) = 0u8;
        *ram_ptr!(game; 0x643, 1) = 0u8;
        *ram_ptr!(game; 0x643, 2) = 0u8;
        *ram_ptr!(game; 0x646, 0) = 0u8;
        *ram_ptr!(game; 0x646, 1) = 0u8;
        *ram_ptr!(game; 0x646, 2) = 0u8;
        *ram_ptr!(game; 0x649, 0) = 0u8;
        *ram_ptr!(game; 0x649, 1) = 0u8;
        *ram_ptr!(game; 0x649, 2) = 0u8;
        *ram_ptr!(game; 0x64d, 0) = 0u8;
        *ram_ptr!(game; 0x64d, 1) = 0u8;
        *ram_ptr!(game; 0x64d, 2) = 0u8;
        *ram_ptr!(game; 0x650, 0) = 0u8;
        *ram_ptr!(game; 0x650, 1) = 0u8;
        *ram_ptr!(game; 0x650, 2) = 0u8;
        *ram_ptr!(game; 0x656, 0) = 0u8;
        *ram_ptr!(game; 0x666, 0) = 0u8;
        *ram_ptr!(game; 0x676, 0) = 0u8;
        set_ram_word!(game.oam_next_ptr = 0u16);
        set_ram_word!(game.reg_OAMaddr_UNUSED = 0u16);
        ClearOamExt(game);
        ClearUnusedOam(game);
        set_ram_word!(game.nmi_copy_samus_halves = 0u16);
        set_ram_word!(game.nmi_copy_samus_top_half_src = 0u16);
        set_ram_word!(game.nmi_copy_samus_bottom_half_src = 0u16);
        EnableNMI(game);
        RtlApuWrite(APUI00, 0u8, game);
        RtlApuWrite(APUI01, 0u8, game);
        RtlApuWrite(APUI02, 0u8, game);
        RtlApuWrite(APUI03, 0u8, game);
        set_ram_word!(game.random_number = 97u16);
        set_ram_word!(game.music_timer = 0u16);
        set_ram_word!(game.music_queue_delay[0] = 0u16);
        set_ram_word!(game.music_queue_delay[1] = 0u16);
        set_ram_word!(game.music_queue_delay[2] = 0u16);
        set_ram_word!(game.music_queue_delay[3] = 0u16);
        set_ram_word!(game.music_queue_delay[4] = 0u16);
        set_ram_word!(game.music_queue_delay[5] = 0u16);
        set_ram_word!(game.music_queue_delay[6] = 0u16);
        set_ram_word!(game.music_queue_delay[7] = 0u16);
        set_ram_word!(game.enable_debug = 0u16);
        VerifySRAM(game);
        set_ram_word!(game.debug_disable_sounds = 0u16);
        set_ram_word!(game.sound_handler_downtime = 0u16);
    }
    *_state_variable_ = 2u8;
    0 as CoroutineRet
}
pub unsafe fn InvalidInterrupt_Crash(game: &mut Game) {
    printf(b"InvalidInterrupt_Crash\n\0" as *const u8 as *const c_char);
    Unreachable(game);
    loop {}
}
pub unsafe fn LoadMirrorOfExploredMapTiles(game: &mut Game) {
    let mut v1: u16 = u16::swap_bytes(ram_word!(game, area_index));
    let mut v2: u16 = 0u16;
    loop {
        *(ram_ptr!(game, u16; 0x7f7, v2 as isize)) =
            ram_word!(game, explored_map_tiles_saved[(v1 as i32 >> 1i32)]);
        v1 = (v1).wrapping_add(2u16);
        v2 = (v2).wrapping_add(2u16);
        if (((v2) as i16).wrapping_sub(256i16) >= 0i16) {
            break;
        }
    }
    set_ram_word!(
        game.has_area_map = *ram_ptr!(game; 0xd908, ram_word!(game, area_index) as isize) as u16
    );
}
pub unsafe fn SaveExploredMapTilesToSaved(game: &mut Game) {
    let mut v1: u16 = u16::swap_bytes(ram_word!(game, area_index));
    let mut v2: u16 = 0u16;
    loop {
        set_ram_word!(
            game.explored_map_tiles_saved[(v1 as i32 >> 1i32)] =
                *(ram_ptr!(game, u16; 0x7f7, v2 as isize))
        );
        v1 = (v1).wrapping_add(2u16);
        v2 = (v2).wrapping_add(2u16);
        if (((v2) as i16).wrapping_sub(256i16) >= 0i16) {
            break;
        }
    }
    if ram_word!(game, has_area_map) != 0 {
        let ref mut fresh88 = *(ram_ptr!(game, u16; 0xd908, ram_word!(game, area_index) as isize));
        *fresh88 = (*fresh88) | (0xffu16);
    }
}
pub unsafe fn InitializeCpuIoRegs(game: &mut Game) {
    crate::rtl::write_register_and_mirror(NMITIMEN as u16, 1, 0x84, false, game);
    WriteReg(WRIO as u16, 0u8, game);
    WriteReg(WRMPYA as u16, 0u8, game);
    WriteReg(WRMPYB as u16, 0u8, game);
    WriteReg(WRDIVL as u16, 0u8, game);
    WriteReg(WRDIVH as u16, 0u8, game);
    WriteReg(WRDIVB as u16, 0u8, game);
    WriteReg(HTIMEL as u16, 0u8, game);
    WriteReg(HTIMEH as u16, 0u8, game);
    WriteReg(VTIMEL as u16, 0u8, game);
    WriteReg(VTIMEH as u16, 0u8, game);
    WriteReg(MDMAEN as u16, 0u8, game);
    crate::rtl::write_register_and_mirror(HDMAEN as u16, 0, 0x85, false, game);
    crate::rtl::write_register_and_mirror(MEMSEL as u16, 1, 0x86, false, game);
}
pub unsafe fn InitializePpuIoRegs(game: &mut Game) {
    crate::rtl::write_register_and_mirror(INIDISP as u16, 0x8f, 0x51, false, game);
    crate::rtl::write_register_and_mirror(OBSEL as u16, 3, 0x52, false, game);
    WriteReg(OAMADDL as u16, 0u8, game);
    *(ram_ptr!(game, u16; 0x53) as *mut u8).offset(0) = 0u8;
    WriteReg(OAMADDH as u16, 0x80u8, game);
    *(ram_ptr!(game, u16; 0x53) as *mut u8).offset(1isize) = 0x80u8;
    WriteReg(OAMDATA as u16, 0u8, game);
    WriteReg(OAMDATA as u16, 0u8, game);
    crate::rtl::write_register_and_mirror(BGMODE as u16, 9, 0x55, false, game);
    crate::rtl::write_register_and_mirror(MOSAIC as u16, 0, 0x57, false, game);
    crate::rtl::write_register_and_mirror(BG1SC as u16, 0x40, 0x58, false, game);
    crate::rtl::write_register_and_mirror(BG2SC as u16, 0x44, 0x59, false, game);
    crate::rtl::write_register_and_mirror(BG3SC as u16, 0x48, 0x5a, false, game);
    crate::rtl::write_register_and_mirror(BG4SC as u16, 0, 0x5c, false, game);
    crate::rtl::write_register_and_mirror(BG12NBA as u16, 0, 0x5d, false, game);
    crate::rtl::write_register_and_mirror(BG34NBA as u16, 5, 0x5e, false, game);
    WriteReg(BG1HOFS as u16, 0u8, game);
    WriteReg(BG1HOFS as u16, 0u8, game);
    WriteReg(BG1VOFS as u16, 0u8, game);
    WriteReg(BG1VOFS as u16, 0u8, game);
    WriteReg(BG2HOFS as u16, 0u8, game);
    WriteReg(BG2HOFS as u16, 0u8, game);
    WriteReg(BG2VOFS as u16, 0u8, game);
    WriteReg(BG2VOFS as u16, 0u8, game);
    WriteReg(BG3HOFS as u16, 0u8, game);
    WriteReg(BG3HOFS as u16, 0u8, game);
    WriteReg(BG3VOFS as u16, 0u8, game);
    WriteReg(BG3VOFS as u16, 0u8, game);
    WriteReg(BG4HOFS as u16, 0u8, game);
    WriteReg(BG4HOFS as u16, 0u8, game);
    WriteReg(BG4VOFS as u16, 0u8, game);
    WriteReg(BG4VOFS as u16, 0u8, game);
    WriteReg(VMAIN as u16, 0u8, game);
    crate::rtl::write_register_and_mirror(M7SEL as u16, 0, 0x5f, false, game);
    WriteReg(M7A as u16, 0u8, game);
    WriteReg(M7B as u16, 0u8, game);
    WriteReg(M7C as u16, 0u8, game);
    WriteReg(M7D as u16, 0u8, game);
    WriteReg(M7X as u16, 0u8, game);
    WriteReg(M7Y as u16, 0u8, game);
    crate::rtl::write_register_and_mirror(W12SEL as u16, 0, 0x60, false, game);
    crate::rtl::write_register_and_mirror(W34SEL as u16, 0, 0x61, false, game);
    crate::rtl::write_register_and_mirror(WOBJSEL as u16, 0, 0x62, false, game);
    crate::rtl::write_register_and_mirror(WH0 as u16, 0, 0x63, false, game);
    WriteReg(WH1 as u16, 0xf8u8, game);
    *ram_ptr!(game; 0x64) = -8i32 as u8;
    crate::rtl::write_register_and_mirror(WH2 as u16, 0, 0x65, false, game);
    crate::rtl::write_register_and_mirror(WH3 as u16, 0, 0x66, false, game);
    crate::rtl::write_register_and_mirror(WBGLOG as u16, 0, 0x67, false, game);
    crate::rtl::write_register_and_mirror(WOBJLOG as u16, 0, 0x68, false, game);
    crate::rtl::write_register_and_mirror(TM as u16, 0x11, 0x69, false, game);
    crate::rtl::write_register_and_mirror(TMW as u16, 0x11, 0x6c, false, game);
    crate::rtl::write_register_and_mirror(TS as u16, 2, 0x6b, false, game);
    crate::rtl::write_register_and_mirror(TSW as u16, 2, 0x6d, false, game);
    crate::rtl::write_register_and_mirror(CGWSEL as u16, 2, 0x6e, false, game);
    WriteReg(CGADSUB as u16, 0xa1u8, game);
    *ram_ptr!(game; 0x71) = -95i32 as u8;
    WriteReg(COLDATA as u16, 0xe0u8, game);
    WriteReg(COLDATA as u16, 0xe0u8, game);
    WriteReg(COLDATA as u16, 0x80u8, game);
    *ram_ptr!(game; 0x74, 0) = 0x80u8;
    WriteReg(COLDATA as u16, 0x40u8, game);
    *ram_ptr!(game; 0x74, 1) = 64u8;
    WriteReg(COLDATA as u16, 0x20u8, game);
    *ram_ptr!(game; 0x74, 2) = 32u8;
    crate::rtl::write_register_and_mirror(SETINI as u16, 0, 0x77, false, game);
}
pub unsafe fn WriteLotsOf0x1c2f(game: &mut Game) {
    sub_8088EB(0x1c2fu16, game);
    sub_8088FE(0x1c2fu16, game);
    sub_808911(0x1c2fu16, game);
}
pub unsafe fn sub_8088EB(a: u16, game: &mut Game) {
    memset7E(ram_ptr!(game, u16; 0x3000), a, 0x800u16);
}
pub unsafe fn sub_8088FE(a: u16, game: &mut Game) {
    memset7E(ram_ptr!(game, u16; 0x4000), a, 0x800u16);
}
pub unsafe fn sub_808911(a: u16, game: &mut Game) {
    memset7E(ram_ptr!(game, u16; 0x6000), a, 0x800u16);
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
pub unsafe fn QueueMode7Transfers(db: u8, k: u16, game: &mut Game) {
    let mut p: *const u8 = RomPtrWithBank(db, k, game);
    let mut dst: *mut u8 = (ram_ptr!(game, Mode7VramWriteQueue; 0x2d0) as *mut u8)
        .offset(ram_word!(game, mode7_vram_write_queue_tail) as i32 as isize);
    loop {
        let f: i32 = *(p as *mut u8) as i32;
        if f & 0x80i32 != 0 {
            *dst.offset(9) = 0u8;
            memcpy(dst as *mut c_void, p as *const c_void, 9 as size_t);
            p = p.offset(9);
            dst = dst.offset(9);
        } else {
            if (f & 0x40i32 == 0) {
                break;
            }
            *dst.offset(7) = 0u8;
            memcpy(dst as *mut c_void, p as *const c_void, 7 as size_t);
            p = p.offset(7);
            dst = dst.offset(7);
        }
    }
    set_ram_word!(
        game.mode7_vram_write_queue_tail =
            dst.offset_from(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0) as *mut u8) as c_long as u16
    );
}
pub unsafe fn NMI_ProcessMode7Queue(game: &mut Game) {
    if ram_word!(game, mode7_vram_write_queue_tail) != 0 {
        NMI_ProcessMode7QueueInner(ram_ptr!(game; 0x2d0), game);
        *(&raw mut (*(ram_ptr!(game, Mode7VramWriteQueue; 0x2d0)).offset(0)).field_0 as *mut u16) =
            0u16;
        set_ram_word!(game.mode7_vram_write_queue_tail = 0u16);
    }
}
pub unsafe fn NMI_ProcessMode7QueueInner(mut p: *const u8, game: &mut Game) {
    loop {
        let mut v2: u8 = 0;
        loop {
            v2 = *p;
            if (v2 as i32 & 0x80i32) == 0i32 {
                break;
            }
            WriteReg(DMAP1 as u16, (v2) & (0x1fu8), game);
            let v5: *mut Mode7CgvmWriteQueue = p as *mut Mode7CgvmWriteQueue;
            WriteRegWord(A1T1L as u16, (*v5).src_addr.addr, game);
            WriteReg(A1B1 as u16, (*v5).src_addr.bank, game);
            WriteRegWord(DAS1L as u16, (*v5).count, game);
            if v2 as i32 & 0x40i32 != 0 {
                WriteReg(BBAD1 as u16, 0x19u8, game);
            } else {
                WriteReg(BBAD1 as u16, 0x18u8, game);
            }
            WriteRegWord(VMADDL as u16, (*v5).vram_addr, game);
            WriteReg(VMAIN as u16, (*v5).vmain, game);
            WriteReg(MDMAEN as u16, 2u8, game);
            p = p.offset(size_of::<Mode7CgvmWriteQueue>() as isize);
        }
        if v2 as i32 & 0x40i32 == 0 {
            break;
        }
        WriteReg(DMAP1 as u16, (v2) & (0x1fu8), game);
        WriteRegWord(A1T1L as u16, *(p.offset(1) as *mut u16), game);
        WriteReg(A1B1 as u16, *p.offset(3), game);
        WriteRegWord(DAS1L as u16, *(p as *mut u16).offset(2), game);
        WriteReg(BBAD1 as u16, 0x22u8, game);
        WriteReg(CGADD as u16, *p.offset(6), game);
        WriteReg(MDMAEN as u16, 2u8, game);
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
    if ram_word!(game, vram_read_queue_tail) as u8 != 0 {
        *(&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(0)).vram_target as *mut u8)
            .offset(ram_word!(game, vram_read_queue_tail) as u8 as i32 as isize) = 0u8;
        let mut v0: i32 = 0i32;
        WriteReg(VMAIN as u16, 0x80u8, game);
        while (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(v0 as isize)).vram_target != 0 {
            WriteRegWord(
                VMADDL as u16,
                (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(v0 as isize)).vram_target,
                game,
            );
            ReadRegWord(RDVRAML as u16, game);
            WriteRegWord(
                DMAP1 as u16,
                (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(v0 as isize)).dma_parameters,
                game,
            );
            WriteRegWord(
                A1T1L as u16,
                (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(v0 as isize))
                    .src
                    .addr,
                game,
            );
            WriteRegWord(
                A1T1H as u16,
                *((&raw mut (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(v0 as isize))
                    .src
                    .addr as *mut u8)
                    .offset(1) as *mut VoidP),
                game,
            );
            WriteRegWord(
                DAS1L as u16,
                (*(ram_ptr!(game, VramReadQueueEnt; 0x340)).offset(v0 as isize)).size,
                game,
            );
            WriteRegWord(DAS10 as u16, 0u16, game);
            WriteRegWord(A2A1H as u16, 0u16, game);
            WriteReg(MDMAEN as u16, 2u8, game);
            v0 += 1;
        }
        *(ram_ptr!(game, u16; 0x360) as *mut u8).offset(0) = 0u8;
    }
}
pub unsafe fn HasQueuedMusic(game: &mut Game) -> u8 {
    let mut v0: i32 = 14i32;
    while ram_word!(game, music_queue_delay[(v0 >> 1i32)]) == 0 {
        v0 -= 2i32;
        if v0 < 0i32 {
            return 0u8;
        }
    }
    1u8
}
pub unsafe fn HandleMusicQueue(game: &mut Game) {
    let ref mut fresh74 = *(ram_ptr!(game, u16; 0x63f));
    let fresh75 = *fresh74;
    *fresh74 = (*fresh74).wrapping_sub(1);
    let v0: bool = fresh75 == 1u16;
    if (ram_word!(game, music_timer) as i32 & 0x8000i32) == 0i32 {
        if !v0 {
            return;
        }
        if (ram_word!(game, music_entry) as i32 & 0x8000i32) != 0i32 {
            set_ram_word!(game.music_data_index = ram_word!(game, music_entry) as u8 as u16);
            *ram_ptr!(game; 0x64c) = -1i32 as u8;
            APU_UploadBank(
                Load24(
                    (game.g_rom.offset(crate::rtl::lorom_offset(0x8fe7e1)) as *mut LongPtr
                        as *mut u8)
                        .offset(ram_word!(game, music_entry) as u8 as i32 as isize)
                        as *mut LongPtr,
                ),
                game,
            );
            *ram_ptr!(game; 0x64c) = 0u8;
            let v5: i32 = ram_word!(game, music_queue_read_pos) as i32;
            set_ram_word!(game.music_queue_track[(v5 >> 1i32)] = 0u16);
            set_ram_word!(game.music_queue_delay[(v5 >> 1i32)] = 0u16);
            set_ram_word!(game.music_queue_read_pos = (v5 + 2i32 & 0xei32) as u16);
            set_ram_word!(game.sound_handler_downtime = 8u16);
            return;
        }
        let v1: u8 = ((ram_word!(game, music_entry)) as u8) & (0x7fu8);
        set_ram_word!(game.music_track_index = (ram_word!(game, music_entry)) & (0x7fu16));
        RtlApuWrite(
            APUI00,
            ((ram_word!(game, music_entry)) as u8) & (0x7fu8),
            game,
        );
        *ram_ptr!(game; 0x64c) = v1;
        set_ram_word!(game.sound_handler_downtime = 8u16);
        let v2: i32 = ram_word!(game, music_queue_read_pos) as i32;
        set_ram_word!(game.music_queue_track[(v2 >> 1i32)] = 0u16);
        set_ram_word!(game.music_queue_delay[(v2 >> 1i32)] = 0u16);
        set_ram_word!(game.music_queue_read_pos = (v2 + 2i32 & 0xei32) as u16);
    }
    if ram_word!(game, music_queue_read_pos) == ram_word!(game, music_queue_write_pos) {
        set_ram_word!(game.music_timer = 0u16);
    } else {
        let v4: i32 = ram_word!(game, music_queue_read_pos) as i32 >> 1i32;
        set_ram_word!(game.music_entry = ram_word!(game, music_queue_track[v4]));
        set_ram_word!(game.music_timer = ram_word!(game, music_queue_delay[v4]));
    };
}
pub unsafe fn QueueMusic_Delayed8(a: u16, game: &mut Game) {
    let mut v1: i8 = 0;
    if (ram_word!(game, game_state) as i32) < kGameState_40_TransitionToDemo as i32
        && ram_word!(game, music_queue_write_pos) as i32 + 2i32 & 0xei32
            != ram_word!(game, music_queue_read_pos) as i32
    {
        v1 = ram_word!(game, music_queue_write_pos) as i8;
        let v2: i32 = ram_word!(game, music_queue_write_pos) as i32 >> 1i32;
        set_ram_word!(game.music_queue_track[v2] = a);
        set_ram_word!(game.music_queue_delay[v2] = 8u16);
        set_ram_word!(game.music_queue_write_pos = (v1 as i32 + 2i32 & 0xei32) as u16);
    }
}
pub unsafe fn QueueMusic_DelayedY(a: u16, j: u16, game: &mut Game) {
    if ram_word!(game, game_state) < 0x28u16 {
        let v2: i32 = ram_word!(game, music_queue_write_pos) as i32;
        set_ram_word!(game.music_queue_track[(v2 >> 1i32)] = a);
        let mut v4: i32 = j as i32;
        if j < 8u16 {
            v4 = 8i32;
        }
        set_ram_word!(game.music_queue_delay[(v2 >> 1i32)] = v4 as u16);
        set_ram_word!(game.music_queue_write_pos = (v2 + 2i32 & 0xei32) as u16);
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

pub fn QueueSfx1_Max15(sound: u16, game: &mut Game) {
    queue_sfx(0, sound as u8, 15, game);
}
pub fn QueueSfx1_Max9(sound: u16, game: &mut Game) {
    queue_sfx(0, sound as u8, 9, game);
}
pub fn QueueSfx1_Max3(sound: u16, game: &mut Game) {
    queue_sfx(0, sound as u8, 3, game);
}
pub fn QueueSfx1_Max1(sound: u16, game: &mut Game) {
    queue_sfx(0, sound as u8, 1, game);
}
pub fn QueueSfx1_Max6(sound: u16, game: &mut Game) {
    queue_sfx(0, sound as u8, 6, game);
}
pub fn QueueSfx1_Internal(packed: u16, game: &mut Game) {
    let [max_queued, sound] = packed.to_le_bytes();
    queue_sfx(0, sound, max_queued, game);
}
pub fn QueueSfx2_Max15(sound: u16, game: &mut Game) {
    queue_sfx(1, sound as u8, 15, game);
}
pub fn QueueSfx2_Max9(sound: u16, game: &mut Game) {
    queue_sfx(1, sound as u8, 9, game);
}
pub fn QueueSfx2_Max3(sound: u16, game: &mut Game) {
    queue_sfx(1, sound as u8, 3, game);
}
pub fn QueueSfx2_Max1(sound: u16, game: &mut Game) {
    queue_sfx(1, sound as u8, 1, game);
}
pub fn QueueSfx2_Max6(sound: u16, game: &mut Game) {
    queue_sfx(1, sound as u8, 6, game);
}
pub fn QueueSfx2_Internal(packed: u16, game: &mut Game) {
    let [max_queued, sound] = packed.to_le_bytes();
    queue_sfx(1, sound, max_queued, game);
}
pub fn QueueSfx3_Max15(sound: u16, game: &mut Game) {
    queue_sfx(2, sound as u8, 15, game);
}
pub fn QueueSfx3_Max9(sound: u16, game: &mut Game) {
    queue_sfx(2, sound as u8, 9, game);
}
pub fn QueueSfx3_Max3(sound: u16, game: &mut Game) {
    queue_sfx(2, sound as u8, 3, game);
}
pub fn QueueSfx3_Max1(sound: u16, game: &mut Game) {
    queue_sfx(2, sound as u8, 1, game);
}
pub fn QueueSfx3_Max6(sound: u16, game: &mut Game) {
    queue_sfx(2, sound as u8, 6, game);
}
pub fn QueueSfx3_Internal(packed: u16, game: &mut Game) {
    let [max_queued, sound] = packed.to_le_bytes();
    queue_sfx(2, sound, max_queued, game);
}
pub unsafe fn SetupDmaTransfer(p: *const c_void, game: &mut Game) {
    let s: *const StartDmaCopy = p as *const StartDmaCopy;
    let v2: i32 = (*s).chan as i32 * 16i32;
    WriteRegWord(
        (v2 + DMAP0 as i32) as SnesRegs as u16,
        (&raw const (*s).dmap).cast::<u16>().read_unaligned(),
        game,
    );
    WriteRegWord(
        (v2 + A1T0L as i32) as SnesRegs as u16,
        (&raw const (*s).a1.addr).read_unaligned(),
        game,
    );
    WriteReg((v2 + A1B0 as i32) as SnesRegs as u16, (*s).a1.bank, game);
    WriteRegWord((v2 + DAS0L as i32) as SnesRegs as u16, (*s).das, game);
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
    let RegWord: u16 = ReadRegWord(JOY1L as u16, game);
    set_ram_word!(game.joypad1_lastkeys = RegWord);
    set_ram_word!(
        game.joypad1_newkeys =
            (RegWord as i32 & (ram_word!(game, joypad1_prev) as i32 ^ RegWord as i32)) as u16
    );
    set_ram_word!(
        game.joypad1_newkeys2_UNUSED =
            (RegWord as i32 & (ram_word!(game, joypad1_prev) as i32 ^ RegWord as i32)) as u16
    );
    if RegWord != 0u16 && RegWord == ram_word!(game, joypad1_prev) {
        if update_ram_word!(game.joypad1_keyrepeat_ctr, wrapping_sub(1), reread) == 0 {
            set_ram_word!(game.joypad1_newkeys2_UNUSED = ram_word!(game, joypad1_lastkeys));
            set_ram_word!(game.joypad1_keyrepeat_ctr = ram_word!(game, joypad_ctr_repeat_next));
        }
    } else {
        set_ram_word!(game.joypad1_keyrepeat_ctr = ram_word!(game, joypad_ctr_repeat_first));
    }
    set_ram_word!(game.joypad1_prev = ram_word!(game, joypad1_lastkeys));
    if ram_word!(game, enable_debug) != 0 {
        let v1: u16 = ReadRegWord(JOY2L as u16, game);
        set_ram_word!(game.joypad2_last = v1);
        set_ram_word!(
            game.joypad2_new_keys =
                (v1 as i32 & (ram_word!(game, joypad2_prev) as i32 ^ v1 as i32)) as u16
        );
        set_ram_word!(
            game.joypad2_newkeys2 =
                (v1 as i32 & (ram_word!(game, joypad2_prev) as i32 ^ v1 as i32)) as u16
        );
        if v1 != 0u16 && v1 == ram_word!(game, joypad2_prev) {
            if update_ram_word!(game.joypad2_keyrepeat_ctr, wrapping_sub(1), reread) == 0 {
                set_ram_word!(game.joypad2_newkeys2 = ram_word!(game, joypad2_last));
                set_ram_word!(game.joypad2_keyrepeat_ctr = ram_word!(game, joypad_ctr_repeat_next));
            }
        } else {
            set_ram_word!(game.joypad2_keyrepeat_ctr = ram_word!(game, joypad_ctr_repeat_first));
        }
        set_ram_word!(game.joypad2_prev = ram_word!(game, joypad2_last));
        if ram_word!(game, is_uploading_apu) != 0u16
            || ram_word!(game, joypad1_lastkeys) as i32
                != kButton_Select as i32
                    | kButton_Start as i32
                    | kButton_L as i32
                    | kButton_R as i32
        {
            if ram_word!(game, enable_debug) != 0 {
                set_ram_word!(game.joypad_dbg_1 = 0u16);
                set_ram_word!(game.joypad_dbg_2 = 0u16);
                if (ram_word!(game, joypad_dbg_flags) as i32 & 0x4000i32) == 0i32 {
                    if ram_word!(game, joypad1_lastkeys) as i32
                        & (kButton_Select as i32 | kButton_L as i32)
                        == kButton_Select as i32 | kButton_L as i32
                    {
                        set_ram_word!(game.joypad_dbg_1 = ram_word!(game, joypad1_newkeys));
                        set_ram_word!(game.joypad1_lastkeys = 0u16);
                        set_ram_word!(game.joypad1_newkeys = 0u16);
                    }
                    if ram_word!(game, joypad1_lastkeys) as i32
                        & (kButton_Select as i32 | kButton_R as i32)
                        == kButton_Select as i32 | kButton_R as i32
                    {
                        set_ram_word!(game.joypad_dbg_2 = ram_word!(game, joypad1_newkeys));
                        set_ram_word!(game.joypad1_lastkeys = 0u16);
                        set_ram_word!(game.joypad1_newkeys = 0u16);
                    }
                    if (ram_word!(game, joypad_dbg_2) as i32 & 0x80i32) != 0i32 {
                        let ref mut fresh82 = *(ram_ptr!(game, u16; 0x84));
                        *fresh82 = (*fresh82) ^ (0x30u16);
                    }
                    if (ram_word!(game, joypad_dbg_2) as i32 & 0x8000i32) != 0i32 {
                        let v2: bool =
                            (!(ram_word!(game, joypad_dbg_flags) as i32) & 0x8000i32) != 0i32;
                        update_ram_word!(game.joypad_dbg_flags, |fresh83| (fresh83) ^ (0x8000u16));
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
                            set_ram_word!(game.samus_missiles = 0u16);
                            set_ram_word!(game.samus_super_missiles = 0u16);
                            set_ram_word!(game.samus_power_bombs = 0u16);
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
                    if (ram_word!(game, joypad_dbg_2) as i32 & 0x40i32) != 0i32 {
                        update_ram_word!(game.joypad_dbg_flags, |fresh84| (fresh84) ^ (0x2000u16));
                    }
                }
            } else {
                set_ram_word!(game.joypad_dbg_1 = 0u16);
                set_ram_word!(game.joypad_dbg_2 = 0u16);
                update_ram_word!(game.joypad2_last, |fresh85| (fresh85) & (0x10u16));
                update_ram_word!(game.joypad2_new_keys, |fresh86| (fresh86) & (0x10u16));
            }
        } else {
            set_ram_word!(game.debug_disable_sounds = 0u16);
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
pub unsafe fn CopyToVramNow(vram_dst: u16, src: u32, size: u16, game: &mut Game) {
    WriteReg(INIDISP as u16, 0x80u8, game);
    WriteRegWord(VMADDL as u16, vram_dst, game);
    WriteRegWord(DMAP1 as u16, 0x1801u16, game);
    WriteRegWord(A1T1L as u16, src as u16, game);
    WriteReg(A1B1 as u16, (src >> 16i32) as u8, game);
    WriteRegWord(DAS1L as u16, size, game);
    WriteReg(VMAIN as u16, 0x80u8, game);
    WriteReg(MDMAEN as u16, 2u8, game);
    WriteReg(INIDISP as u16, 0xfu8, game);
}
pub unsafe fn IrqHandler_SetResult(a: u16, y: u16, x: u16, game: &mut Game) {
    set_ram_word!(game.cur_irq_handler = a);
    WriteRegWord(VTIMEL as u16, y, game);
    WriteRegWord(HTIMEL as u16, x, game);
}
pub unsafe fn IrqHandler_0_Nothing(game: &mut Game) {
    let a: u16 = ram_word!(game, irqhandler_next_handler);
    set_ram_word!(game.irqhandler_next_handler = 0u16);
    IrqHandler_SetResult(a, 0u16, 0u16, game);
}
pub unsafe fn IrqHandler_2_DisableIRQ(game: &mut Game) {
    let ref mut fresh76 = *(ram_ptr!(game, u16; 0x84));
    *fresh76 = (*fresh76 as i32 & !(0x30i32)) as u16;
    IrqHandler_SetResult(0u16, 0u16, 0u16, game);
}
pub unsafe fn IrqHandler_4_Main_BeginHudDraw(game: &mut Game) {
    WriteReg(BG3SC as u16, 0x5au8, game);
    WriteReg(CGWSEL as u16, 0u8, game);
    WriteReg(CGADSUB as u16, 0u8, game);
    WriteReg(TM as u16, 4u8, game);
    IrqHandler_SetResult(6u16, 31u16, 152u16, game);
}
pub unsafe fn IrqHandler_6_Main_EndHudDraw(game: &mut Game) {
    WriteReg(CGWSEL as u16, *ram_ptr!(game; 0x70), game);
    WriteReg(CGADSUB as u16, *ram_ptr!(game; 0x73), game);
    WriteReg(BG3SC as u16, *ram_ptr!(game; 0x5b), game);
    WriteReg(TM as u16, *ram_ptr!(game; 0x6a), game);
    let a: u16 = ram_word!(game, irqhandler_next_handler);
    set_ram_word!(game.irqhandler_next_handler = 0u16);
    IrqHandler_SetResult(
        (if a != 0u16 { a as i32 } else { 4i32 }) as u16,
        0u16,
        152u16,
        game,
    );
}
pub unsafe fn IrqHandler_8_StartOfDoor_BeginHud(game: &mut Game) {
    WriteReg(BG3SC as u16, 0x5au8, game);
    WriteReg(TM as u16, 4u8, game);
    WriteReg(CGWSEL as u16, 0u8, game);
    WriteReg(CGADSUB as u16, 0u8, game);
    IrqHandler_SetResult(10u16, 31u16, 152u16, game);
}
pub unsafe fn IrqHandler_10_StartOfDoor_EndHud(game: &mut Game) {
    let mut v0: u8 = 0;
    if ((ram_word!(game, previous_cre_bitset) as i32 | ram_word!(game, cre_bitset) as i32) & 1i32)
        != 0i32
    {
        v0 = 16u8;
    } else {
        v0 = 17u8;
    }
    WriteReg(TM as u16, v0, game);
    let a: u16 = ram_word!(game, irqhandler_next_handler);
    set_ram_word!(game.irqhandler_next_handler = 0u16);
    IrqHandler_SetResult(
        (if a != 0u16 { a as i32 } else { 8i32 }) as u16,
        0u16,
        152u16,
        game,
    );
}
pub unsafe fn IrqHandler_12_Draygon_BeginHud(game: &mut Game) {
    WriteReg(TM as u16, 4u8, game);
    WriteReg(CGWSEL as u16, 0u8, game);
    WriteReg(CGADSUB as u16, 0u8, game);
    IrqHandler_SetResult(14u16, 31u16, 152u16, game);
}
pub unsafe fn IrqHandler_14_Draygon_EndHud(game: &mut Game) {
    WriteReg(BG3SC as u16, *ram_ptr!(game; 0x5b), game);
    WriteReg(CGWSEL as u16, *ram_ptr!(game; 0x70), game);
    WriteReg(CGADSUB as u16, *ram_ptr!(game; 0x73), game);
    let a: u16 = ram_word!(game, irqhandler_next_handler);
    set_ram_word!(game.irqhandler_next_handler = 0u16);
    IrqHandler_SetResult(
        (if a != 0u16 { a as i32 } else { 12i32 }) as u16,
        0u16,
        152u16,
        game,
    );
}
pub unsafe fn IrqHandler_16_VerticalDoor_BeginHud(game: &mut Game) {
    WriteReg(TM as u16, 4u8, game);
    WriteReg(CGWSEL as u16, 0u8, game);
    WriteReg(CGADSUB as u16, 0u8, game);
    IrqHandler_SetResult(18u16, 31u16, 152u16, game);
}
pub unsafe fn IrqHandler_18_VerticalDoor_EndHud(game: &mut Game) {
    let mut v0: u8 = 0;
    if ((ram_word!(game, previous_cre_bitset) as i32 | ram_word!(game, cre_bitset) as i32) & 1i32)
        != 0i32
    {
        v0 = 16u8;
    } else {
        v0 = 17u8;
    }
    WriteReg(TM as u16, v0, game);
    WriteReg(CGWSEL as u16, 0u8, game);
    WriteReg(CGADSUB as u16, 0u8, game);
    if (ram_word!(game, door_transition_flag) as i32 & 0x8000i32) == 0i32 {
        Irq_FollowDoorTransition(game);
    }
    IrqHandler_SetResult(20u16, 216u16, 152u16, game);
}
pub unsafe fn IrqHandler_20_VerticalDoor_EndDraw(game: &mut Game) {
    let a: u16 = ram_word!(game, irqhandler_next_handler);
    set_ram_word!(game.irqhandler_next_handler = 0u16);
    IrqHandler_SetResult(
        (if a != 0u16 { a as i32 } else { 16i32 }) as u16,
        0u16,
        152u16,
        game,
    );
}
pub unsafe fn IrqHandler_22_HorizDoor_BeginHud(game: &mut Game) {
    WriteReg(TM as u16, 4u8, game);
    WriteReg(CGWSEL as u16, 0u8, game);
    WriteReg(CGADSUB as u16, 0u8, game);
    IrqHandler_SetResult(24u16, 31u16, 152u16, game);
}
pub unsafe fn IrqHandler_24_HorizDoor_EndHud(game: &mut Game) {
    let mut v0: u8 = 0;
    if ((ram_word!(game, previous_cre_bitset) as i32 | ram_word!(game, cre_bitset) as i32) & 1i32)
        != 0i32
    {
        v0 = 16u8;
    } else {
        v0 = 17u8;
    }
    WriteReg(TM as u16, v0, game);
    WriteReg(CGWSEL as u16, 0u8, game);
    WriteReg(CGADSUB as u16, 0u8, game);
    if (ram_word!(game, door_transition_flag) as i32 & 0x8000i32) == 0i32 {
        Irq_FollowDoorTransition(game);
    }
    IrqHandler_SetResult(26u16, 160u16, 152u16, game);
}
pub unsafe fn IrqHandler_26_HorizDoor_EndDraw(game: &mut Game) {
    let a: u16 = ram_word!(game, irqhandler_next_handler);
    set_ram_word!(game.irqhandler_next_handler = 0u16);
    IrqHandler_SetResult(
        (if a != 0u16 { a as i32 } else { 22i32 }) as u16,
        0u16,
        152u16,
        game,
    );
}
pub unsafe fn EnableIrqInterrupts(game: &mut Game) {
    WriteRegWord(VTIMEL as u16, 0u16, game);
    WriteRegWord(HTIMEL as u16, 152u16, game);
    let ref mut fresh64 = *(ram_ptr!(game, u16; 0x84));
    *fresh64 = (*fresh64) | (0x30u16);
}
pub unsafe fn EnableIrqInterruptsNow(game: &mut Game) {
    WriteRegWord(VTIMEL as u16, 0u16, game);
    WriteRegWord(HTIMEL as u16, 152u16, game);
    let ref mut fresh65 = *(ram_ptr!(game, u16; 0x84));
    *fresh65 = (*fresh65) | (0x30u16);
    WriteReg(NMITIMEN as u16, *ram_ptr!(game; 0x84), game);
}
pub unsafe fn DisableIrqInterrupts(game: &mut Game) {
    let ref mut fresh41 = *(ram_ptr!(game, u16; 0x84));
    *fresh41 = (*fresh41 as i32 & !(0x30i32)) as u16;
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
    kIrqHandlers[(ram_word!(game, cur_irq_handler) as i32 >> 1i32) as usize](game);
}
static kHudTilemaps_Row1to3: [u16; 96] = [
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c12, 0x2c12, 0x2c23, 0x2c12, 0x2c12, 0x2c1e, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2822, 0x2822,
    0x2823, 0x2813, 0x2c14, 0x2c1e, 0x2c0f, 0x2c0b, 0x2c0c, 0x2c0d, 0x2c32, 0x2c0f, 0x2c09, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c12, 0x2c12, 0xa824, 0x2815, 0x2c16, 0x2c1e,
];
static kHudTilemaps_Missiles: [u16; 22] = [
    0x344b, 0x3449, 0x744b, 0x344c, 0x344a, 0x744c, 0x3434, 0x7434, 0x3435, 0x7435, 0x3436, 0x7436,
    0x3437, 0x7437, 0x3438, 0x7438, 0x3439, 0x7439, 0x343a, 0x743a, 0x343b, 0x743b,
];
pub unsafe fn AddMissilesToHudTilemap(game: &mut Game) {
    if (ram_word!(game, hud_tilemap[10]) as i32 & 0x3ffi32) == 15i32 {
        set_ram_word!(game.hud_tilemap[10] = kHudTilemaps_Missiles[0i32 as usize]);
        set_ram_word!(game.hud_tilemap[11] = kHudTilemaps_Missiles[1i32 as usize]);
        set_ram_word!(game.hud_tilemap[12] = kHudTilemaps_Missiles[2i32 as usize]);
        set_ram_word!(game.hud_tilemap[42] = kHudTilemaps_Missiles[3i32 as usize]);
        set_ram_word!(game.hud_tilemap[43] = kHudTilemaps_Missiles[4i32 as usize]);
        set_ram_word!(game.hud_tilemap[44] = kHudTilemaps_Missiles[5i32 as usize]);
    }
}
pub unsafe fn AddSuperMissilesToHudTilemap(game: &mut Game) {
    AddToTilemapInner(
        0x1cu16,
        RomPtr_80((addr_kHudTilemaps_Missiles as i32 + 12i32) as u16, game) as *const u16,
        game,
    );
}
pub unsafe fn AddPowerBombsToHudTilemap(game: &mut Game) {
    AddToTilemapInner(
        0x22u16,
        RomPtr_80((addr_kHudTilemaps_Missiles as i32 + 20i32) as u16, game) as *const u16,
        game,
    );
}
pub unsafe fn AddGrappleToHudTilemap(game: &mut Game) {
    AddToTilemapInner(
        0x28u16,
        RomPtr_80((addr_kHudTilemaps_Missiles as i32 + 28i32) as u16, game) as *const u16,
        game,
    );
}
pub unsafe fn AddXrayToHudTilemap(game: &mut Game) {
    AddToTilemapInner(
        0x2eu16,
        RomPtr_80((addr_kHudTilemaps_Missiles as i32 + 36i32) as u16, game) as *const u16,
        game,
    );
}
pub unsafe fn AddToTilemapInner(k: u16, j: *const u16, game: &mut Game) {
    let v2: i32 = k as i32 >> 1i32;
    if (ram_word!(game, hud_tilemap[v2]) as i32 & 0x3ffi32) == 15i32 {
        set_ram_word!(game.hud_tilemap[v2] = *j.offset(0));
        set_ram_word!(game.hud_tilemap[(v2 + 1i32)] = *j.offset(1));
        set_ram_word!(game.hud_tilemap[(v2 + 32i32)] = *j.offset(2));
        set_ram_word!(game.hud_tilemap[(v2 + 33i32)] = *j.offset(3));
    }
}
pub unsafe fn InitializeHud(game: &mut Game) {
    WriteRegWord(VMADDL as u16, addr_unk_605800 as u16, game);
    WriteRegWord(VMAIN as u16, 0x80u16, game);
    static unk_809A8F: StartDmaCopy = StartDmaCopy::vram_words(1, 0x80988b, 0x40);
    SetupDmaTransfer(&raw const unk_809A8F as *const c_void, game);
    WriteReg(MDMAEN as u16, 2u8, game);
    let mut i: i32 = 0i32;
    while i != 192i32 {
        set_ram_word!(game.hud_tilemap[(i >> 1i32)] = kHudTilemaps_Row1to3[(i >> 1i32) as usize]);
        i += 2i32;
    }
    if (ram_word!(game, equipped_items) as i32 & 0x8000i32) != 0i32 {
        AddXrayToHudTilemap(game);
    }
    if (ram_word!(game, equipped_items) as i32 & 0x4000i32) != 0i32 {
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
    set_ram_word!(game.samus_prev_health = 0u16);
    set_ram_word!(game.samus_prev_missiles = 0u16);
    set_ram_word!(game.samus_prev_super_missiles = 0u16);
    set_ram_word!(game.samus_prev_power_bombs = 0u16);
    set_ram_word!(game.samus_prev_hud_item_index = 0u16);
    InitializeMiniMapBroken();
    if ram_word!(game, samus_max_missiles) != 0 {
        DrawThreeHudDigits(
            addrl_kDigitTilesetsWeapon,
            ram_word!(game, samus_missiles),
            0x94u16,
            game,
        );
    }
    if ram_word!(game, samus_max_super_missiles) != 0 {
        DrawTwoHudDigits(
            addrl_kDigitTilesetsWeapon,
            ram_word!(game, samus_super_missiles),
            0x9cu16,
            game,
        );
    }
    if ram_word!(game, samus_max_power_bombs) != 0 {
        DrawTwoHudDigits(
            addrl_kDigitTilesetsWeapon,
            ram_word!(game, samus_power_bombs),
            0xa2u16,
            game,
        );
    }
    ToggleHudItemHighlight(ram_word!(game, hud_item_index), 0x1000u16, game);
    ToggleHudItemHighlight(ram_word!(game, samus_prev_hud_item_index), 0x1400u16, game);
    HandleHudTilemap(game);
}
static kEnergyTankIconTilemapOffsets: [u16; 14] = [
    0x42, 0x44, 0x46, 0x48, 0x4a, 0x4c, 0x4e, 2, 4, 6, 8, 0xa, 0xc, 0xe,
];
pub unsafe fn HandleHudTilemap(game: &mut Game) {
    if ram_word!(game, reserve_health_mode) == 1u16 {
        let mut v1: *const u16 =
            RomPtr_80(addr_kHudTilemaps_AutoReserve as u16, game) as *const u16;
        if ram_word!(game, samus_reserve_health) == 0 {
            v1 = v1.offset(6);
        }
        set_ram_word!(game.hud_tilemap[8] = *v1.offset(0));
        set_ram_word!(game.hud_tilemap[9] = *v1.offset(1));
        set_ram_word!(game.hud_tilemap[40] = *v1.offset(2));
        set_ram_word!(game.hud_tilemap[41] = *v1.offset(3));
        set_ram_word!(game.hud_tilemap[72] = *v1.offset(4));
        set_ram_word!(game.hud_tilemap[73] = *v1.offset(5));
    }
    if ram_word!(game, samus_health) != ram_word!(game, samus_prev_health) {
        set_ram_word!(game.samus_prev_health = ram_word!(game, samus_health));
        let mut r20: u16 = SnesDivide(ram_word!(game, samus_health), 100u8);
        let r18: u16 = SnesModulus(ram_word!(game, samus_health), 100u8);
        let mut v2: i32 = 0i32;
        let mut n: i32 = SnesDivide(ram_word!(game, samus_max_health), 100u8) as i32 + 1i32;
        loop {
            n -= 1;
            if n == 0 {
                break;
            }
            let mut v3: u16 = 13360u16;
            if r20 != 0 {
                r20 = r20.wrapping_sub(1);
                v3 = 10289u16;
            }
            *(ram_ptr!(game, u16; 0xc608)).offset(
                (kEnergyTankIconTilemapOffsets[(v2 >> 1i32) as usize] as i32 >> 1i32) as isize,
            ) = v3;
            v2 += 2i32;
            if (((v2 - 28i32) as i16) >= 0i16) {
                break;
            }
        }
        DrawTwoHudDigits(addrl_kDigitTilesetsHealth, r18, 0x8cu16, game);
    }
    if ram_word!(game, samus_max_missiles) != 0u16
        && ram_word!(game, samus_missiles) != ram_word!(game, samus_prev_missiles)
    {
        set_ram_word!(game.samus_prev_missiles = ram_word!(game, samus_missiles));
        DrawThreeHudDigits(
            addrl_kDigitTilesetsWeapon,
            ram_word!(game, samus_missiles),
            0x94u16,
            game,
        );
    }
    if ram_word!(game, samus_max_super_missiles) != 0u16
        && ram_word!(game, samus_super_missiles) != ram_word!(game, samus_prev_super_missiles)
    {
        set_ram_word!(game.samus_prev_super_missiles = ram_word!(game, samus_super_missiles));
        if (ram_word!(game, joypad_dbg_flags) as i32 & 0x1f40i32) != 0i32 {
            DrawThreeHudDigits(
                addrl_kDigitTilesetsWeapon,
                ram_word!(game, samus_prev_super_missiles),
                0x9cu16,
                game,
            );
        } else {
            DrawTwoHudDigits(
                addrl_kDigitTilesetsWeapon,
                ram_word!(game, samus_prev_super_missiles),
                0x9cu16,
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
            0xa2u16,
            game,
        );
    }
    if ram_word!(game, hud_item_index) != ram_word!(game, samus_prev_hud_item_index) {
        ToggleHudItemHighlight(ram_word!(game, hud_item_index), 0x1000u16, game);
        ToggleHudItemHighlight(ram_word!(game, samus_prev_hud_item_index), 0x1400u16, game);
        set_ram_word!(game.samus_prev_hud_item_index = ram_word!(game, hud_item_index));
        if *ram_ptr!(game; 0xa1f) != 3u8
            && *ram_ptr!(game; 0xa1f) != 20u8
            && ram_word!(game, grapple_beam_function) == 0xc4f0u16
            && ram_word!(game, time_is_frozen_flag) == 0
        {
            QueueSfx1_Max6(0x39u16, game);
        }
    }
    let mut v4: u16 = 5120u16;
    if (*ram_ptr!(game; 0x5b5) as i32 & 0x10i32) != 0i32 {
        v4 = 4096u16;
    }
    ToggleHudItemHighlight(ram_word!(game, samus_auto_cancel_hud_item_index), v4, game);
    let mut v5: u16 = ram_word!(game, vram_write_queue_tail);
    (*gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game)).size = 192u16;
    v5 = (v5).wrapping_add(2u16);
    (*gVramWriteEntry(v5 as i32, game)).size = (ram_ptr!(game, u16; 0xc608) as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8)
        as c_long as u16;
    v5 = (v5).wrapping_add(2u16);
    let fresh71 = v5;
    v5 = v5.wrapping_add(1);
    (*gVramWriteEntry(fresh71 as i32, game)).size = 126u16;
    (*gVramWriteEntry(v5 as i32, game)).size = addr_unk_605820 as u16;
    set_ram_word!(game.vram_write_queue_tail = (v5).wrapping_add(2u16));
}
static kHudItemTilemapOffsets: [u16; 5] = [0x14, 0x1c, 0x22, 0x28, 0x2e];
pub unsafe fn ToggleHudItemHighlight(a: u16, k: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    set_ram_word!(game.hud_item_tilemap_palette_bits = k);
    v2 = ((a) as i16).wrapping_sub(1i16);
    if v2 >= 0i16 {
        let v3: i32 = kHudItemTilemapOffsets[v2 as usize] as i32 >> 1i32;
        if ram_word!(game, hud_tilemap[v3]) != 11279u16 {
            set_ram_word!(
                game.hud_tilemap[v3] = (ram_word!(game, hud_item_tilemap_palette_bits) as i32
                    | ram_word!(game, hud_tilemap[v3]) as i32 & 0xe3ffi32)
                    as u16
            );
        }
        if ram_word!(game, hud_tilemap[(v3 + 1i32)]) != 11279u16 {
            set_ram_word!(
                game.hud_tilemap[(v3 + 1i32)] = (ram_word!(game, hud_item_tilemap_palette_bits)
                    as i32
                    | ram_word!(game, hud_tilemap[(v3 + 1i32)]) as i32 & 0xe3ffi32)
                    as u16
            );
        }
        if ram_word!(game, hud_tilemap[(v3 + 32i32)]) != 11279u16 {
            set_ram_word!(
                game.hud_tilemap[(v3 + 32i32)] = (ram_word!(game, hud_item_tilemap_palette_bits)
                    as i32
                    | ram_word!(game, hud_tilemap[(v3 + 32i32)]) as i32 & 0xe3ffi32)
                    as u16
            );
        }
        if ram_word!(game, hud_tilemap[(v3 + 33i32)]) != 11279u16 {
            set_ram_word!(
                game.hud_tilemap[(v3 + 33i32)] = (ram_word!(game, hud_item_tilemap_palette_bits)
                    as i32
                    | ram_word!(game, hud_tilemap[(v3 + 33i32)]) as i32 & 0xe3ffi32)
                    as u16
            );
        }
        if 2i32 * v2 as i32 == 0 {
            if ram_word!(game, hud_tilemap[(v3 + 2i32)]) != 11279u16 {
                set_ram_word!(
                    game.hud_tilemap[(v3 + 2i32)] = (ram_word!(game, hud_item_tilemap_palette_bits)
                        as i32
                        | ram_word!(game, hud_tilemap[(v3 + 2i32)]) as i32 & 0xe3ffi32)
                        as u16
                );
            }
            if ram_word!(game, hud_tilemap[(v3 + 34i32)]) != 11279u16 {
                set_ram_word!(
                    game.hud_tilemap[(v3 + 34i32)] = (ram_word!(game, hud_item_tilemap_palette_bits)
                        as i32
                        | ram_word!(game, hud_tilemap[(v3 + 34i32)]) as i32 & 0xe3ffi32)
                        as u16
                );
            }
        }
    }
}
pub unsafe fn DrawThreeHudDigits(addr: u32, a: u16, k: u16, game: &mut Game) {
    set_ram_word!(
        game.hud_tilemap[(k as i32 >> 1i32)] = *(RomPtr(
            (addr).wrapping_add((2i32 * (a as i32 / 100i32)) as u32,),
            game,
        ) as *mut u16)
    );
    DrawTwoHudDigits(
        addr,
        (a as i32 % 100i32) as u16,
        (k).wrapping_add(2u16),
        game,
    );
}
pub unsafe fn DrawTwoHudDigits(addr: u32, a: u16, k: u16, game: &mut Game) {
    let v3: i32 = k as i32 >> 1i32;
    set_ram_word!(
        game.hud_tilemap[v3] = *(RomPtr(
            (addr).wrapping_add((2i32 * (a as i32 / 10i32)) as u32,),
            game,
        ) as *mut u16)
    );
    set_ram_word!(
        game.hud_tilemap[(v3 + 1i32)] = *(RomPtr(
            (addr).wrapping_add((2i32 * (a as i32 % 10i32)) as u32,),
            game,
        ) as *mut u16)
    );
}
static kTimerProcessFuncs: [Option<Func_U8>; 7] = unsafe {
    [
        Some(ProcessTimer_Empty as unsafe fn(&mut Game) -> u8),
        Some(ProcessTimer_CeresStart as unsafe fn(&mut Game) -> u8),
        Some(ProcessTimer_MotherBrainStart as unsafe fn(&mut Game) -> u8),
        Some(ProcessTimer_InitialDelay as unsafe fn(&mut Game) -> u8),
        Some(ProcessTimer_MovementDelayed as unsafe fn(&mut Game) -> u8),
        Some(ProcessTimer_MovingIntoPlace as unsafe fn(&mut Game) -> u8),
        Some(ProcessTimer_Decrement as unsafe fn(&mut Game) -> u8),
    ]
};
pub unsafe fn ProcessTimer(game: &mut Game) -> u8 {
    kTimerProcessFuncs[ram_word!(game, timer_status) as u8 as usize]
        .expect("non-null function pointer")(game)
}
pub unsafe fn ProcessTimer_CeresStart(game: &mut Game) -> u8 {
    ClearTimerRam(game);
    SetTimerMinutes(0x100u16, game);
    set_ram_word!(game.timer_status = -32765i32 as u16);
    0u8
}
pub fn ProcessTimer_Empty(game: &mut Game) -> u8 {
    0u8
}
pub unsafe fn ProcessTimer_MotherBrainStart(game: &mut Game) -> u8 {
    ClearTimerRam(game);
    SetTimerMinutes(0x300u16, game);
    set_ram_word!(game.timer_status = -32765i32 as u16);
    0u8
}
pub unsafe fn ProcessTimer_InitialDelay(game: &mut Game) -> u8 {
    *(ram_ptr!(game, u16; 0x948) as *mut u8).offset(0) =
        ((ram_word!(game, timer_x_pos)) as u8).wrapping_add(1u8);
    if (ram_word!(game, timer_x_pos) as u8) >= 0x10u8 {
        *(ram_ptr!(game, u16; 0x943) as *mut u8).offset(0) =
            ((ram_word!(game, timer_status)) as u8).wrapping_add(1u8);
    }
    0u8
}
pub unsafe fn ProcessTimer_MovementDelayed(game: &mut Game) -> u8 {
    *(ram_ptr!(game, u16; 0x948) as *mut u8).offset(0) =
        ((ram_word!(game, timer_x_pos)) as u8).wrapping_add(1u8);
    if (ram_word!(game, timer_x_pos) as u8) >= 0x60u8 {
        *(ram_ptr!(game, u16; 0x948) as *mut u8).offset(0) = 0u8;
        *(ram_ptr!(game, u16; 0x943) as *mut u8).offset(0) =
            ((ram_word!(game, timer_status)) as u8).wrapping_add(1u8);
    }
    ProcessTimer_Decrement(game)
}
pub unsafe fn ProcessTimer_MovingIntoPlace(game: &mut Game) -> u8 {
    let mut v0: i32 = 0i32;
    let mut v1: u16 = (ram_word!(game, timer_x_pos)).wrapping_add(224u16);
    if (ram_word!(game, timer_x_pos)).wrapping_add(224u16) >= 0xdc00u16 {
        v0 = 1i32;
        v1 = -9216i32 as u16;
    }
    set_ram_word!(game.timer_x_pos = v1);
    let mut v2: u16 = (ram_word!(game, timer_y_pos)).wrapping_sub(193u16);
    if (ram_word!(game, timer_y_pos)).wrapping_sub(193u16) < 0x3000u16 {
        v0 += 1;
        v2 = 12288u16;
    }
    set_ram_word!(game.timer_y_pos = v2);
    if v0 == 2i32 {
        update_ram_word!(game.timer_status, wrapping_add(1));
    }
    ProcessTimer_Decrement(game)
}
pub unsafe fn SetTimerMinutes(a: u16, game: &mut Game) {
    *(ram_ptr!(game, u16; 0x945)) = 0u16;
    *(ram_ptr!(game, u16; 0x946)) = a;
}
pub unsafe fn ClearTimerRam(game: &mut Game) {
    set_ram_word!(game.timer_x_pos = 0x8000u16);
    set_ram_word!(game.timer_y_pos = 0x8000u16);
    *(ram_ptr!(game, u16; 0x945)) = 0u16;
    *(ram_ptr!(game, u16; 0x946)) = 0u16;
    set_ram_word!(game.timer_status = 0u16);
}
pub unsafe fn DecrementDecimal(number: *mut u8, value: u8) -> bool {
    let mut result: i32 = (*number as i32 & 0xfi32) + (!(value as i32) & 0xfi32) + 1i32;
    if result < 0x10i32 {
        result = result - 0x6i32
            & (if (result - 0x6i32) < 0i32 {
                0xfi32
            } else {
                0x1fi32
            });
    }
    result = (*number as i32 & 0xf0i32) + (!(value as i32) & 0xf0i32) + result;
    if result < 0x100i32 {
        result -= 0x60i32;
    }
    *number = result as u8;
    result < 0x100i32
}
pub unsafe fn ProcessTimer_Decrement(game: &mut Game) -> u8 {
    static kTimerCentisecondDecrements: [u8; 128] = [
        1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2, 2, 1, 2, 2,
        1, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2, 2, 1,
        2, 2, 1, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2,
        2, 1, 2, 2, 1, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2,
        2, 1, 2, 2, 2, 1, 2, 2,
    ];
    if DecrementDecimal(
        ram_ptr!(game; 0x945),
        kTimerCentisecondDecrements
            [(ram_word!(game, nmi_frame_counter_word) as i32 & 0x7fi32) as usize],
    ) && DecrementDecimal(ram_ptr!(game; 0x946), 1u8)
    {
        if DecrementDecimal(ram_ptr!(game; 0x947), 1u8) {
            *ram_ptr!(game; 0x945) = 0u8;
            *ram_ptr!(game; 0x946) = 0u8;
            *ram_ptr!(game; 0x947) = 0u8;
        } else {
            *ram_ptr!(game; 0x946) = 0x59u8;
        }
    }
    ((*ram_ptr!(game; 0x945) as i32
        | *ram_ptr!(game; 0x946) as i32
        | *ram_ptr!(game; 0x947) as i32)
        == 0i32) as u8
}
pub unsafe fn DrawTimer(game: &mut Game) {
    DrawTimerSpritemap(0u16, addr_word_80A060 as u16, game);
    DrawTwoTimerDigits(*(ram_ptr!(game, u16; 0x947)), 0xffe4u16, game);
    DrawTwoTimerDigits(*(ram_ptr!(game, u16; 0x946)), 0xfffcu16, game);
    DrawTwoTimerDigits(*(ram_ptr!(game, u16; 0x945)), 0x14u16, game);
}
pub unsafe fn DrawTwoTimerDigits(a: u16, k: u16, game: &mut Game) {
    DrawTimerSpritemap(
        k,
        crate::rtl::rom_table_word!(game, 0x809fd4, ((a as i32 & 0xf0i32) >> 4i32) as isize),
        game,
    );
    DrawTimerSpritemap(
        (k).wrapping_add(8u16),
        crate::rtl::rom_table_word!(game, 0x809fd4, (a as i32 & 0xfi32) as isize),
        game,
    );
}
pub unsafe fn DrawTimerSpritemap(a: u16, j: u16, game: &mut Game) {
    DrawSpritemap(
        0x80u8,
        j,
        (a).wrapping_add((*(ram_ptr!(game, u16; 0x948) as *mut u8).offset(1isize)) as u16),
        *(ram_ptr!(game, u16; 0x94a) as *mut u8).offset(1isize) as u16,
        2560u16,
        game,
    );
}
pub unsafe fn StartGameplay_Async(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut u8 = ram_ptr!(game; 0x613);
    let mut current_block_54: u64;
    match *_state_variable_ as i32 {
        0 => {
            WriteRegWord(MDMAEN as u16, 0u16, game);
            set_ram_word!(game.scrolling_finished_hook = 0u16);
            set_ram_word!(game.music_data_index = 0u16);
            set_ram_word!(game.music_track_index = 0u16);
            set_ram_word!(game.timer_status = 0u16);
            ResetSoundQueues(game);
            set_ram_word!(game.debug_disable_sounds = -1i32 as u16);
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
        let mut _coret_: u8 = Play20FramesOfMusic_Async(game);
        if _coret_ != 0 {
            *_state_variable_ = 1u8;
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
        let mut _coret__0: u8 = Play20FramesOfMusic_Async(game);
        if _coret__0 != 0 {
            *_state_variable_ = 2u8;
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
                4i32
            } else {
                ram_word!(game, room_loading_irq_handler) as i32
            }) as u16
        );
        EnableIrqInterrupts(game);
        current_block_54 = 3421737773041484705;
    }
    if current_block_54 == 3421737773041484705 {
        let mut _coret__1: u8 = Play20FramesOfMusic_Async(game);
        if _coret__1 != 0 {
            *_state_variable_ = 3u8;
            return _coret__1;
        }
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x8u8, 0x8u8, 0xb7eb as VoidP),
            game,
        );
        set_ram_word!(game.door_transition_function = ((0x82e737i32) as u16));
    }
    *_state_variable_ = 0u8;
    0 as CoroutineRet
}
pub unsafe fn Play20FramesOfMusic_Async(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut u8 = ram_ptr!(game; 0x614);
    's_57: {
        let mut current_block_9: u64;
        match *_state_variable_ as i32 {
            0 => {
                EnableNMI(game);
                set_ram_word!(game.my_counter = 0u16);
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
                    let mut _coret_: u8 = WaitForNMI_Async(game);
                    if _coret_ != 0 {
                        *_state_variable_ = 1u8;
                        return _coret_;
                    }
                    update_ram_word!(game.my_counter, wrapping_add(1));
                    current_block_9 = 8258075665625361029;
                }
            }
        }
    }
    *_state_variable_ = 0u8;
    0 as CoroutineRet
}
pub unsafe fn ResumeGameplay(game: &mut Game) {
    WriteRegWord(MDMAEN as u16, 0u16, game);
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
    let mut v1: i32 = 0;
    let mut v2: i32 = 0;
    let mut v3: i32 = 0;
    v1 = (*ram_ptr!(game; 0x58) as i32 - *ram_ptr!(game; 0x59) as i32) << 8i32;
    set_ram_word!(game.size_of_bg2 = (v1 & 0xf800i32) as u16);
    CalculateBgScrollAndLayerPositionBlocks(game);
    v2 = 0i32;
    loop {
        v3 = v2;
        set_ram_word!(game.blocks_to_update_x_block = ram_word!(game, layer1_x_block));
        set_ram_word!(game.blocks_to_update_y_block = ram_word!(game, layer1_y_block));
        set_ram_word!(game.vram_blocks_to_update_x_block = ram_word!(game, bg1_x_block));
        set_ram_word!(game.vram_blocks_to_update_y_block = ram_word!(game, bg1_y_block));
        UploadLevelDataColumn(game);
        if *ram_ptr!(game; 0x91b) as i32 & 1i32 == 0 {
            set_ram_word!(game.blocks_to_update_x_block = ram_word!(game, layer2_x_block));
            set_ram_word!(game.blocks_to_update_y_block = ram_word!(game, layer2_y_block));
            set_ram_word!(game.vram_blocks_to_update_x_block = ram_word!(game, bg2_x_block));
            set_ram_word!(game.vram_blocks_to_update_y_block = ram_word!(game, bg2_y_block));
            UploadBackgroundDataColumn(game);
        }
        NMI_ProcessVramWriteQueue(game);
        update_ram_word!(game.layer1_x_block, wrapping_add(1));
        update_ram_word!(game.bg1_x_block, wrapping_add(1));
        update_ram_word!(game.layer2_x_block, wrapping_add(1));
        update_ram_word!(game.bg2_x_block, wrapping_add(1));
        v2 += 1;
        if (v3 == 16i32) {
            break;
        }
    }
}
pub unsafe fn QueueClearingOfFxTilemap(game: &mut Game) {
    let mut i: i32 = 3838i32;
    while i >= 0i32 {
        *((&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16 as *mut u8)
            .offset(i as u16 as i32 as isize) as *mut u16) = 6222u16;
        i -= 2i32;
    }
    let v1: u16 = ram_word!(game, vram_write_queue_tail);
    let v2: *mut VramWriteEntry =
        gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
    (*v2).size = 3840u16;
    (*v2).src.addr = (ram_ptr!(game, Ram4000; 0x4000) as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as VoidP;
    *(&raw mut (*v2).src.bank as *mut u16) = 126u16;
    (*v2).vram_dst = addr_unk_605880 as u16;
    set_ram_word!(game.vram_write_queue_tail = (v1).wrapping_add(7u16));
}
pub unsafe fn ClearBG2Tilemap(game: &mut Game) {
    WriteRegWord(VMADDL as u16, 0x4800u16, game);
    WriteRegWord(DMAP1 as u16, 0x1808u16, game);
    WriteRegWord(A1T1L as u16, 0xa29au16, game);
    WriteRegWord(A1B1 as u16, 0x80u16, game);
    WriteRegWord(DAS1L as u16, 0x800u16, game);
    WriteReg(VMAIN as u16, 0u8, game);
    WriteReg(MDMAEN as u16, 2u8, game);
    WriteRegWord(VMADDL as u16, 0x4800u16, game);
    WriteRegWord(DMAP1 as u16, 0x1908u16, game);
    WriteRegWord(A1T1L as u16, 0xa29au16, game);
    WriteRegWord(A1B1 as u16, 0x80u16, game);
    WriteRegWord(DAS1L as u16, 0x800u16, game);
    WriteReg(VMAIN as u16, 0x80u8, game);
    WriteReg(MDMAEN as u16, 2u8, game);
}
pub unsafe fn ClearFXTilemap(game: &mut Game) {
    WriteRegWord(VMADDL as u16, 0x5880u16, game);
    WriteRegWord(DMAP1 as u16, 0x1808u16, game);
    WriteRegWord(A1T1L as u16, 0xa2f7u16, game);
    WriteRegWord(A1B1 as u16, 0x80u16, game);
    WriteRegWord(DAS1L as u16, 0x780u16, game);
    WriteReg(VMAIN as u16, 0u8, game);
    WriteReg(MDMAEN as u16, 2u8, game);
    WriteRegWord(VMADDL as u16, 0x5880u16, game);
    WriteRegWord(DMAP1 as u16, 0x1908u16, game);
    WriteRegWord(A1T1L as u16, 0xa2f8u16, game);
    WriteRegWord(A1B1 as u16, 0x80u16, game);
    WriteRegWord(DAS1L as u16, 0x780u16, game);
    WriteReg(VMAIN as u16, 0x80u8, game);
    WriteReg(MDMAEN as u16, 2u8, game);
}
pub unsafe fn CalculateLayer2Xpos(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0x91b) == 0 {
        set_ram_word!(game.layer2_x_pos = ram_word!(game, layer1_x_pos));
        return 0u8;
    }
    if *ram_ptr!(game; 0x91b) != 1u8 {
        let t: i32 = *ram_ptr!(game; 0x91b) as i32 & 0xfei32;
        let var933: u8 =
            (t * *(ram_ptr!(game, u16; 0x911) as *mut u8).offset(0) as i32 >> 8i32) as u8;
        set_ram_word!(
            game.layer2_x_pos = (t * *(ram_ptr!(game, u16; 0x911) as *mut u8).offset(1isize,)
                as i32
                + var933 as i32) as u16
        );
        return 0u8;
    }
    1u8
}
pub unsafe fn CalculateLayer2Ypos(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0x91c) == 0 {
        set_ram_word!(game.layer2_y_pos = ram_word!(game, layer1_y_pos));
        return 0u8;
    }
    if *ram_ptr!(game; 0x91c) != 1u8 {
        let t: i32 = *ram_ptr!(game; 0x91c) as i32 & 0xfei32;
        let var933: u8 = (t * ram_word!(game, layer1_y_pos) as u8 as i32 >> 8i32) as u8;
        set_ram_word!(
            game.layer2_y_pos = (t * *(ram_ptr!(game, u16; 0x915) as *mut u8).offset(1isize,)
                as i32
                + var933 as i32) as u16
        );
        return 0u8;
    }
    1u8
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
    let mut v0: i32 = 0i32;
    let v1: bool = ((ram_word!(game, layer1_x_block)) as i16)
        .wrapping_sub((ram_word!(game, previous_layer1_x_block)) as i16)
        < 0i16;
    if ram_word!(game, layer1_x_block) != ram_word!(game, previous_layer1_x_block) {
        set_ram_word!(game.previous_layer1_x_block = ram_word!(game, layer1_x_block));
        if !v1 {
            v0 = 16i32;
        }
        set_ram_word!(
            game.blocks_to_update_x_block = (ram_word!(game, layer1_x_block) as i32 + v0) as u16
        );
        set_ram_word!(
            game.vram_blocks_to_update_x_block = (ram_word!(game, bg1_x_block) as i32 + v0) as u16
        );
        set_ram_word!(game.blocks_to_update_y_block = ram_word!(game, layer1_y_block));
        set_ram_word!(game.vram_blocks_to_update_y_block = ram_word!(game, bg1_y_block));
        UploadLevelDataColumn(game);
    }
    if *ram_ptr!(game; 0x91b) as i32 & 1i32 == 0 {
        let mut v2: i32 = 0i32;
        let v3: bool = ((ram_word!(game, layer2_x_block)) as i16)
            .wrapping_sub((ram_word!(game, previous_layer2_x_block)) as i16)
            < 0i16;
        if ram_word!(game, layer2_x_block) != ram_word!(game, previous_layer2_x_block) {
            set_ram_word!(game.previous_layer2_x_block = ram_word!(game, layer2_x_block));
            if !v3 {
                v2 = 16i32;
            }
            set_ram_word!(
                game.blocks_to_update_x_block =
                    (ram_word!(game, layer2_x_block) as i32 + v2) as u16
            );
            set_ram_word!(
                game.vram_blocks_to_update_x_block =
                    (ram_word!(game, bg2_x_block) as i32 + v2) as u16
            );
            set_ram_word!(game.blocks_to_update_y_block = ram_word!(game, layer2_y_block));
            set_ram_word!(game.vram_blocks_to_update_y_block = ram_word!(game, bg2_y_block));
            UploadBackgroundDataColumn(game);
        }
    }
    let mut v4: i32 = 1i32;
    let v5: bool = ((ram_word!(game, layer1_y_block)) as i16)
        .wrapping_sub((ram_word!(game, previous_layer1_y_block)) as i16)
        < 0i16;
    if ram_word!(game, layer1_y_block) != ram_word!(game, previous_layer1_y_block) {
        set_ram_word!(game.previous_layer1_y_block = ram_word!(game, layer1_y_block));
        if !v5 {
            v4 = 15i32;
        }
        set_ram_word!(
            game.blocks_to_update_y_block = (ram_word!(game, layer1_y_block) as i32 + v4) as u16
        );
        set_ram_word!(
            game.vram_blocks_to_update_y_block = (ram_word!(game, bg1_y_block) as i32 + v4) as u16
        );
        set_ram_word!(game.blocks_to_update_x_block = ram_word!(game, layer1_x_block));
        set_ram_word!(game.vram_blocks_to_update_x_block = ram_word!(game, bg1_x_block));
        UpdateLevelDataRow(game);
    }
    if *ram_ptr!(game; 0x91c) as i32 & 1i32 == 0 {
        let mut v6: i32 = 1i32;
        let v7: bool = ((ram_word!(game, layer2_y_block)) as i16)
            .wrapping_sub((ram_word!(game, previous_layer2_y_block)) as i16)
            < 0i16;
        if ram_word!(game, layer2_y_block) != ram_word!(game, previous_layer2_y_block) {
            set_ram_word!(game.previous_layer2_y_block = ram_word!(game, layer2_y_block));
            if !v7 {
                v6 = 15i32;
            }
            set_ram_word!(
                game.blocks_to_update_y_block =
                    (ram_word!(game, layer2_y_block) as i32 + v6) as u16
            );
            set_ram_word!(
                game.vram_blocks_to_update_y_block =
                    (ram_word!(game, bg2_y_block) as i32 + v6) as u16
            );
            set_ram_word!(game.blocks_to_update_x_block = ram_word!(game, layer2_x_block));
            set_ram_word!(game.vram_blocks_to_update_x_block = ram_word!(game, bg2_x_block));
            UpdateBackgroundDataRow(game);
        }
    }
}
pub unsafe fn CalculateBgScrollAndLayerPositionBlocks(game: &mut Game) {
    set_ram_word!(game.bg1_x_block = (ram_word!(game, reg_BG1HOFS) as i32 >> 4i32) as u16);
    set_ram_word!(game.bg2_x_block = (ram_word!(game, reg_BG2HOFS) as i32 >> 4i32) as u16);
    let mut v0: u16 = (ram_word!(game, layer1_x_pos) as i32 >> 4i32) as u16;
    if (ram_word!(game, layer1_x_pos) as i32 >> 4i32 & 0x800i32) != 0i32 {
        v0 = (v0) | (0xf000u16);
    }
    set_ram_word!(game.layer1_x_block = v0);
    let mut v1: u16 = (ram_word!(game, layer2_x_pos) as i32 >> 4i32) as u16;
    if (ram_word!(game, layer2_x_pos) as i32 >> 4i32 & 0x800i32) != 0i32 {
        v1 = (v1) | (0xf000u16);
    }
    set_ram_word!(game.layer2_x_block = v1);
    set_ram_word!(game.bg1_y_block = (ram_word!(game, reg_BG1VOFS) as i32 >> 4i32) as u16);
    set_ram_word!(game.bg2_y_block = (ram_word!(game, reg_BG2VOFS) as i32 >> 4i32) as u16);
    let mut v2: u16 = (ram_word!(game, layer1_y_pos) as i32 >> 4i32) as u16;
    if (ram_word!(game, layer1_y_pos) as i32 >> 4i32 & 0x800i32) != 0i32 {
        v2 = (v2) | (0xf000u16);
    }
    set_ram_word!(game.layer1_y_block = v2);
    let mut v3: u16 = (ram_word!(game, layer2_y_pos) as i32 >> 4i32) as u16;
    if (ram_word!(game, layer2_y_pos) as i32 >> 4i32 & 0x800i32) != 0i32 {
        v3 = (v3) | (0xf000u16);
    }
    set_ram_word!(game.layer2_y_block = v3);
}
pub unsafe fn HandleAutoscrolling_X(game: &mut Game) {
    let mut v1: i16 = 0;
    let mut v3: i16 = 0;
    let mut v6: i16 = 0;
    let mut v4: u16 = 0;
    let mut var933: u16 = 0;
    if ram_word!(game, time_is_frozen_flag) == 0 {
        let mut var939: u16 = ram_word!(game, layer1_x_pos);
        if (ram_word!(game, layer1_x_pos) as i32 & 0x8000i32) != 0i32 {
            set_ram_word!(game.layer1_x_pos = 0u16);
        }
        let v0: u16 = u16::swap_bytes((ram_word!(game, room_width_in_scrolls)).wrapping_sub(1u16));
        if v0 < ram_word!(game, layer1_x_pos) {
            set_ram_word!(game.layer1_x_pos = v0);
        }
        v1 = *(ram_ptr!(game, u16; 0x911) as *mut u8).offset(1isize) as i16;
        let v2: u16 = (Mult8x8(
            (((ram_word!(game, layer1_y_pos)).wrapping_add(128u16)) as i32 >> 8i32) as u8,
            ram_word!(game, room_width_in_scrolls) as u8,
        ))
        .wrapping_add((v1) as u16);
        if *ram_ptr!(game; 0xcd20, v2 as isize) != 0 {
            if *ram_ptr!(game; 0xcd20, ((v2).wrapping_add(1u16)) as isize) != 0 {
                return;
            }
            var933 = (ram_word!(game, layer1_x_pos)) & (0xff00u16);
            let v5: u16 =
                (var939 as i32 - ram_word!(game, absolute_moved_last_frame_x) as i32 - 2i32) as u16;
            if ((v5 as i32 - (ram_word!(game, layer1_x_pos) as i32 & 0xff00i32)) as i16) < 0i16 {
                v4 = var933;
            } else {
                var939 = v5;
                v6 = *(&raw mut var939 as *mut u8).offset(1isize) as i16;
                if *ram_ptr!(game; 0xcd20, ((Mult8x8(
                        (((ram_word!(game, layer1_y_pos)).wrapping_add(128u16)) as i32 >> 8i32)
                            as u8,
                        ram_word!(game, room_width_in_scrolls) as u8,
                    ))
                    .wrapping_add((v6) as u16)) as isize,)
                    != 0
                {
                    v4 = var939;
                } else {
                    v4 = ((var939 as i32 & 0xff00i32) + 256i32) as u16;
                }
            }
        } else {
            var933 = ((ram_word!(game, layer1_x_pos) as i32 & 0xff00i32) + 256i32) as u16;
            if ((ram_word!(game, absolute_moved_last_frame_x) as i32 + var939 as i32 + 2i32) as u16)
                >= var933
            {
                v4 = var933;
            } else {
                var939 = (var939 as i32
                    + (ram_word!(game, absolute_moved_last_frame_x) as i32 + 2i32))
                    as u16;
                v3 = ((*(&raw mut var939 as *mut u8).offset(1isize)).wrapping_add(1u8)) as i16;
                if *ram_ptr!(game; 0xcd20, ((Mult8x8(
                        (((ram_word!(game, layer1_y_pos)).wrapping_add(128u16)) as i32 >> 8i32)
                            as u8,
                        ram_word!(game, room_width_in_scrolls) as u8,
                    ))
                    .wrapping_add((v3) as u16)) as isize,)
                    != 0
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
    let mut v1: i16 = 0;
    let var939: u16 = ram_word!(game, layer1_x_pos);
    if ((ram_word!(game, ideal_layer1_xpos)) as i16)
        .wrapping_sub((ram_word!(game, layer1_x_pos)) as i16)
        < 0i16
    {
        set_ram_word!(game.layer1_x_pos = ram_word!(game, ideal_layer1_xpos));
        set_ram_word!(game.layer1_x_subpos = 0u16);
    }
    let v0: u16 = u16::swap_bytes((ram_word!(game, room_width_in_scrolls)).wrapping_sub(1u16));
    if v0 >= ram_word!(game, layer1_x_pos) {
        v1 = *(ram_ptr!(game, u16; 0x911) as *mut u8).offset(1isize) as i16;
        let RegWord: u16 = Mult8x8(
            (((ram_word!(game, layer1_y_pos)).wrapping_add(128u16)) as i32 >> 8i32) as u8,
            ram_word!(game, room_width_in_scrolls) as u8,
        );
        if *ram_ptr!(game; 0xcd20, (RegWord as i32 + 1i32 + v1 as i32) as u16 as isize) == 0 {
            let var933: u16 = (ram_word!(game, layer1_x_pos)) & (0xff00u16);
            let mut v4: u16 =
                (var939 as i32 - ram_word!(game, absolute_moved_last_frame_x) as i32 - 2i32) as u16;
            if ((v4 as i32 - (ram_word!(game, layer1_x_pos) as i32 & 0xff00i32)) as i16) < 0i16 {
                v4 = var933;
            }
            set_ram_word!(game.layer1_x_pos = v4);
        }
    } else {
        set_ram_word!(game.layer1_x_pos = v0);
    };
}
pub unsafe fn HandleScrollingWhenTriggeringScrollLeft(game: &mut Game) {
    let mut v0: i16 = 0;
    let var939: u16 = ram_word!(game, layer1_x_pos);
    if ((ram_word!(game, layer1_x_pos)) as i16)
        .wrapping_sub((ram_word!(game, ideal_layer1_xpos)) as i16)
        < 0i16
    {
        set_ram_word!(game.layer1_x_pos = ram_word!(game, ideal_layer1_xpos));
        set_ram_word!(game.layer1_x_subpos = 0u16);
    }
    if (ram_word!(game, layer1_x_pos) as i32 & 0x8000i32) == 0i32 {
        v0 = *(ram_ptr!(game, u16; 0x911) as *mut u8).offset(1isize) as i16;
        let prod: u16 = Mult8x8(
            (((ram_word!(game, layer1_y_pos)).wrapping_add(128u16)) as i32 >> 8i32) as u8,
            ram_word!(game, room_width_in_scrolls) as u8,
        );
        if *ram_ptr!(game; 0xcd20, ((prod).wrapping_add((v0) as u16)) as isize) == 0 {
            let var933: u16 = ((ram_word!(game, layer1_x_pos) as i32 & 0xff00i32) + 256i32) as u16;
            let mut v1: u16 =
                (ram_word!(game, absolute_moved_last_frame_x) as i32 + var939 as i32 + 2i32) as u16;
            if v1 >= var933 {
                v1 = var933;
            }
            set_ram_word!(game.layer1_x_pos = v1);
        }
    } else {
        set_ram_word!(game.layer1_x_pos = 0u16);
    };
}
pub unsafe fn HandleAutoscrolling_Y(game: &mut Game) {
    let mut v1: i16 = 0;
    let mut v2: i16 = 0;
    let mut v4: i16 = 0;
    let mut v7: i16 = 0;
    let mut v10: i16 = 0;
    let mut v8: u16 = 0;
    if ram_word!(game, time_is_frozen_flag) == 0 {
        let mut v0: u16 = 0u16;
        v1 = (((ram_word!(game, layer1_x_pos)).wrapping_add(128u16)) as i32 >> 8i32) as i16;
        let r20: u16 = (Mult8x8(
            *(ram_ptr!(game, u16; 0x915) as *mut u8).offset(1isize),
            ram_word!(game, room_width_in_scrolls) as u8,
        ))
        .wrapping_add((v1) as u16);
        if *ram_ptr!(game; 0xcd20, r20 as isize) != 1u8 {
            v0 = 31u16;
        }
        let var933: u16 = v0;
        let mut var939: u16 = ram_word!(game, layer1_y_pos);
        if (ram_word!(game, layer1_y_pos) as i32 & 0x8000i32) != 0i32 {
            set_ram_word!(game.layer1_y_pos = 0u16);
        }
        *(&raw mut v2 as *mut u8).offset(0) =
            (((ram_word!(game, room_height_in_scrolls)).wrapping_sub(1u16)) as i32 >> 8i32) as u8;
        *(&raw mut v2 as *mut u8).offset(1isize) =
            ((ram_word!(game, room_height_in_scrolls)) as u8).wrapping_sub(1u8);
        let v3: u16 = (var933).wrapping_add((v2) as u16);
        if v3 < ram_word!(game, layer1_y_pos) {
            set_ram_word!(game.layer1_y_pos = v3);
        }
        v4 = (((ram_word!(game, layer1_x_pos)).wrapping_add(128u16)) as i32 >> 8i32) as i16;
        let v5: u16 = (Mult8x8(
            *(ram_ptr!(game, u16; 0x915) as *mut u8).offset(1isize),
            ram_word!(game, room_width_in_scrolls) as u8,
        ))
        .wrapping_add((v4) as u16);
        if *ram_ptr!(game; 0xcd20, v5 as isize) == 0 {
            let var935: u16 = ((ram_word!(game, layer1_y_pos) as i32 & 0xff00i32) + 256i32) as u16;
            let mut v6: u16 =
                (ram_word!(game, absolute_moved_last_frame_y) as i32 + var939 as i32 + 2i32) as u16;
            if v6 >= var935 {
                v8 = var935;
            } else {
                var939 = (var939 as i32
                    + (ram_word!(game, absolute_moved_last_frame_y) as i32 + 2i32))
                    as u16;
                v7 = (((ram_word!(game, layer1_x_pos)).wrapping_add(128u16)) as i32 >> 8i32) as i16;
                let prod: u16 = Mult8x8(
                    (*(&raw mut v6 as *mut u8).offset(1isize)).wrapping_add(1u8),
                    ram_word!(game, room_width_in_scrolls) as u8,
                );
                if *ram_ptr!(game; 0xcd20, ((prod).wrapping_add((v7) as u16)) as isize) != 0 {
                    v8 = var939;
                } else {
                    v8 = (var939) & (0xff00u16);
                }
            }
            set_ram_word!(game.layer1_y_pos = v8);
            return;
        }
        if *ram_ptr!(game; 0xcd20, ((ram_word!(game, room_width_in_scrolls)).wrapping_add(v5)) as isize)
            == 0
        {
            let var937: u16 =
                (var933 as i32 + (ram_word!(game, layer1_y_pos) as i32 & 0xff00i32)) as u16;
            if var937 < ram_word!(game, layer1_y_pos) {
                let mut v9: u16 = (var939 as i32
                    - ram_word!(game, absolute_moved_last_frame_y) as i32
                    - 2i32) as u16;
                if ((v9) as i16).wrapping_sub((var937) as i16) < 0i16 {
                    v8 = var937;
                } else {
                    var939 = v9;
                    let prod_0: u16 = Mult8x8(
                        *(&raw mut v9 as *mut u8).offset(1isize),
                        ram_word!(game, room_width_in_scrolls) as u8,
                    );
                    v10 = (((ram_word!(game, layer1_x_pos)).wrapping_add(128u16)) as i32 >> 8i32)
                        as i16;
                    if *ram_ptr!(game; 0xcd20, ((prod_0).wrapping_add((v10) as u16)) as isize) != 0
                    {
                        v8 = var939;
                    } else {
                        v8 = ((var939 as i32 & 0xff00i32) + 256i32) as u16;
                    }
                }
                set_ram_word!(game.layer1_y_pos = v8);
            }
        }
    }
}
pub unsafe fn HandleScrollingWhenTriggeringScrollDown(game: &mut Game) {
    let var939: u16 = ram_word!(game, layer1_y_pos);
    let mut v0: i32 = 0i32;
    let prod: u16 = Mult8x8(
        *(ram_ptr!(game, u16; 0x915) as *mut u8).offset(1isize),
        ram_word!(game, room_width_in_scrolls) as u8,
    );
    let v1: u16 = (((ram_word!(game, layer1_x_pos)).wrapping_add(128u16)) as i32 >> 8i32) as u16;
    let r20: u16 = (prod).wrapping_add(v1);
    if *ram_ptr!(game; 0xcd20, r20 as isize) != 1u8 {
        v0 = 31i32;
    }
    let var933: u16 = v0 as u16;
    if ((ram_word!(game, ideal_layer1_ypos)) as i16)
        .wrapping_sub((ram_word!(game, layer1_y_pos)) as i16)
        < 0i16
    {
        set_ram_word!(game.layer1_y_pos = ram_word!(game, ideal_layer1_ypos));
        set_ram_word!(game.layer1_y_subpos = 0u16);
    }
    let v2: u16 = u16::swap_bytes((ram_word!(game, room_height_in_scrolls)).wrapping_sub(1u16));
    let mut var937: u16 = (var933).wrapping_add(v2);
    if (var933).wrapping_add(v2) < ram_word!(game, layer1_y_pos)
        || *ram_ptr!(game; 0xcd20, ((ram_word!(game, room_width_in_scrolls)).wrapping_add(r20)) as isize)
            == 0
            && {
                var937 =
                    (var933 as i32 + (ram_word!(game, layer1_y_pos) as i32 & 0xff00i32)) as u16;
                var937 < ram_word!(game, layer1_y_pos)
            }
    {
        let mut v3: u16 =
            (var939 as i32 - ram_word!(game, absolute_moved_last_frame_y) as i32 - 2i32) as u16;
        if ((v3) as i16).wrapping_sub((var937) as i16) < 0i16 {
            v3 = var937;
        }
        set_ram_word!(game.layer1_y_pos = v3);
    }
}
pub unsafe fn HandleScrollingWhenTriggeringScrollUp(game: &mut Game) {
    let mut v0: i16 = 0;
    let var939: u16 = ram_word!(game, layer1_y_pos);
    if ((ram_word!(game, layer1_y_pos)) as i16)
        .wrapping_sub((ram_word!(game, ideal_layer1_ypos)) as i16)
        < 0i16
    {
        set_ram_word!(game.layer1_y_pos = ram_word!(game, ideal_layer1_ypos));
        set_ram_word!(game.layer1_y_subpos = 0u16);
    }
    if (ram_word!(game, layer1_y_pos) as i32 & 0x8000i32) == 0i32 {
        let prod: u16 = Mult8x8(
            *(ram_ptr!(game, u16; 0x915) as *mut u8).offset(1isize),
            ram_word!(game, room_width_in_scrolls) as u8,
        );
        v0 = (((ram_word!(game, layer1_x_pos)).wrapping_add(128u16)) as i32 >> 8i32) as i16;
        if *ram_ptr!(game; 0xcd20, ((prod).wrapping_add((v0) as u16)) as isize) == 0 {
            let var933: u16 = ((ram_word!(game, layer1_y_pos) as i32 & 0xff00i32) + 256i32) as u16;
            let mut v1: u16 =
                (ram_word!(game, absolute_moved_last_frame_y) as i32 + var939 as i32 + 2i32) as u16;
            if v1 >= var933 {
                v1 = var933;
            }
            set_ram_word!(game.layer1_y_pos = v1);
        }
    } else {
        set_ram_word!(game.layer1_y_pos = 0u16);
    };
}
pub unsafe fn DebugScrollPosSaveLoad(game: &mut Game) {
    if (ram_word!(game, joypad2_new_keys) as i32 & 0x40i32) != 0i32 {
        update_ram_word!(game.debug_saveload_scrollpos_toggle, wrapping_add(1));
    }
    if ram_word!(game, debug_saveload_scrollpos_toggle) as i32 & 1i32 != 0 {
        set_ram_word!(game.layer1_x_pos = ram_word!(game, debug_saved_xscroll));
        set_ram_word!(game.layer1_y_pos = ram_word!(game, debug_saved_yscroll));
    } else {
        set_ram_word!(game.debug_saved_xscroll = ram_word!(game, layer1_x_pos));
        set_ram_word!(game.debug_saved_yscroll = ram_word!(game, layer1_y_pos));
    };
}
pub unsafe fn UploadBackgroundDataColumn(game: &mut Game) {
    UpdateLevelOrBackgroundDataColumn(0x1cu16, game);
}
pub unsafe fn UploadLevelDataColumn(game: &mut Game) {
    UpdateLevelOrBackgroundDataColumn(0u16, game);
}
pub unsafe fn UpdateLevelOrBackgroundDataColumn(k: u16, game: &mut Game) {
    if ram_word!(game, irq_enable_mode7) != 0 {
        return;
    }
    let mut prod: u16 = Mult8x8(
        ram_word!(game, blocks_to_update_y_block) as u8,
        ram_word!(game, room_width_in_blocks) as u8,
    );
    let v1: u16 = ram_word!(game, blocks_to_update_x_block);
    let mut v2: u16 = (2i32 * (prod as i32 + v1 as i32) + 2i32) as u16;
    if k != 0 {
        v2 = (v2).wrapping_add(0x9600u16);
    }
    let r54: *const u16 = ram_ptr!(game; 0x10000, v2 as i32 as isize) as *const u16;
    let v3: u16 = (4i32 * ram_word!(game, vram_blocks_to_update_y_block) as i32 & 0x3ci32) as u16;
    *((ram_ptr!(game, u16; 0x958) as *mut u8).offset(k as i32 as isize) as *mut u16) = v3;
    *((ram_ptr!(game, u16; 0x956) as *mut u8).offset(k as i32 as isize) as *mut u16) =
        ((v3 as i32 ^ 0x3fi32) + 1i32) as u16;
    prod = Mult8x8(
        ((ram_word!(game, vram_blocks_to_update_y_block)) as u8) & (0xfu8),
        0x40u8,
    );
    let var935: u16 = (ram_word!(game, vram_blocks_to_update_x_block)) & (0x1fu16);
    let v4: u16 = (2u16).wrapping_mul(var935);
    let var933: u16 = (prod).wrapping_add(v4);
    let mut v5: u16 = addr_unk_605000 as u16;
    if var935 >= 0x10u16 {
        v5 = addr_unk_6053E0 as u16;
    }
    if k != 0 {
        v5 = (v5).wrapping_sub(ram_word!(game, size_of_bg2));
    }
    let mut var937: u16 = v5;
    *((ram_ptr!(game, u16; 0x95a) as *mut u8).offset(k as i32 as isize) as *mut u16) =
        (var933).wrapping_add(v5);
    *((ram_ptr!(game, u16; 0x95c) as *mut u8).offset(k as i32 as isize) as *mut u16) =
        (var935 as i32 + var935 as i32 + var937 as i32) as u16;
    let mut v6: u16 = (ram_ptr!(game, u16; 0xc8c8) as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as u16;
    let mut v7: u16 = 0u16;
    if k != 0 {
        v6 = ram_ptr!(game; 0xc9d0).offset_from(&raw mut game.g_ram as *mut u8) as c_long as u16;
        v7 = 264u16;
    }
    let v8: u16 = (*((ram_ptr!(game, u16; 0x956) as *mut u8).offset(k as i32 as isize)
        as *mut u16))
        .wrapping_add(v6);
    *((ram_ptr!(game, u16; 0x95e) as *mut u8).offset(k as i32 as isize) as *mut u16) = v8;
    *((ram_ptr!(game, u16; 0x960) as *mut u8).offset(k as i32 as isize) as *mut u16) =
        (v8).wrapping_add(64u16);
    var937 = v7;
    let t2: u16 = k;
    let mut v9: u16 = 0u16;
    let mut var939: u16 = 16u16;
    loop {
        let var93B: u16 = *r54.offset((v9 as i32 >> 1i32) as isize);
        let v17: u16 = v9;
        let v11: u16 = var937;
        let tile_index = (var937 >> 1) as usize;
        let ram = (&raw mut game.g_ram).cast::<u8>();
        let first = ram
            .add(crate::ram_map::word_arrays::bg1_column_update_tilemap_left_halves)
            .cast::<u16>()
            .add(tile_index);
        let second = ram
            .add(crate::ram_map::word_arrays::bg1_column_update_tilemap_right_halves)
            .cast::<u16>()
            .add(tile_index);
        TileTables::draw_block(
            ram.add(0xa000).cast(),
            var93B,
            [first, second, first.add(1), second.add(1)],
            TileDrawOrder::Rows,
        );
        var937 = (v11).wrapping_add(4u16);
        v9 = (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + v17 as i32) as u16;
        var939 = var939.wrapping_sub(1);
        if (var939 == 0) {
            break;
        }
    }
    let ref mut fresh7 =
        *((ram_ptr!(game, u16; 0x962) as *mut u8).offset(t2 as i32 as isize) as *mut u16);
    *fresh7 = (*fresh7).wrapping_add(1);
}
pub unsafe fn UpdateBackgroundDataRow(game: &mut Game) {
    UpdateLevelOrBackgroundDataRow(0x1cu16, game);
}
pub unsafe fn UpdateLevelDataRow(game: &mut Game) {
    UpdateLevelOrBackgroundDataRow(0u16, game);
}
pub unsafe fn UpdateLevelOrBackgroundDataRow(v0: u16, game: &mut Game) {
    if ram_word!(game, irq_enable_mode7) != 0 {
        return;
    }
    let mut prod: u16 = Mult8x8(
        ram_word!(game, blocks_to_update_y_block) as u8,
        ram_word!(game, room_width_in_blocks) as u8,
    );
    let v1: u16 = ram_word!(game, blocks_to_update_x_block);
    let mut v2: u16 = (2i32 * (prod as i32 + v1 as i32) + 2i32) as u16;
    if v0 != 0 {
        v2 = (v2).wrapping_sub(27136u16);
    }
    let r54: *const u16 = ram_ptr!(game; 0x10000, v2 as i32 as isize) as *const u16;
    let mut var933: u16 = (ram_word!(game, vram_blocks_to_update_x_block)) & (0xfu16);
    *((ram_ptr!(game, u16; 0x964) as *mut u8).offset(v0 as i32 as isize) as *mut u16) =
        (4i32 * (16i32 - var933 as i32)) as u16;
    *((ram_ptr!(game, u16; 0x966) as *mut u8).offset(v0 as i32 as isize) as *mut u16) =
        (4i32 * (var933 as i32 + 1i32)) as u16;
    prod = Mult8x8(
        ((ram_word!(game, vram_blocks_to_update_y_block)) as u8) & (0xfu8),
        0x40u8,
    );
    let var935: u16 = (ram_word!(game, vram_blocks_to_update_x_block)) & (0x1fu16);
    let v3: u16 = (2u16).wrapping_mul(var935);
    var933 = (prod).wrapping_add(v3);
    let mut var937: u16 = addr_unk_605400 as u16;
    let mut v4: u16 = addr_unk_605000 as u16;
    if var935 >= 0x10u16 {
        var937 = addr_unk_605000 as u16;
        v4 = addr_unk_6053E0 as u16;
    }
    if v0 != 0 {
        v4 = (v4).wrapping_sub(ram_word!(game, size_of_bg2));
    }
    *((ram_ptr!(game, u16; 0x968) as *mut u8).offset(v0 as i32 as isize) as *mut u16) =
        (var933).wrapping_add(v4);
    let mut v5: u16 = var937;
    if v0 != 0 {
        v5 = (var937).wrapping_sub(ram_word!(game, size_of_bg2));
    }
    *((ram_ptr!(game, u16; 0x96a) as *mut u8).offset(v0 as i32 as isize) as *mut u16) =
        (prod).wrapping_add(v5);
    let mut v6: u16 = (ram_ptr!(game, u16; 0xc948) as *mut u8)
        .offset_from(&raw mut game.g_ram as *mut u8) as c_long as u16;
    let mut v7: u16 = 0u16;
    if v0 != 0 {
        v6 = ram_ptr!(game; 0xca50).offset_from(&raw mut game.g_ram as *mut u8) as c_long as u16;
        v7 = 264u16;
    }
    let v8: u16 = (*((ram_ptr!(game, u16; 0x964) as *mut u8).offset(v0 as i32 as isize)
        as *mut u16))
        .wrapping_add(v6);
    *((ram_ptr!(game, u16; 0x96c) as *mut u8).offset(v0 as i32 as isize) as *mut u16) = v8;
    *((ram_ptr!(game, u16; 0x96e) as *mut u8).offset(v0 as i32 as isize) as *mut u16) =
        (v8).wrapping_add(68u16);
    var937 = v7;
    let t2: u16 = v0;
    let mut v9: u16 = 0u16;
    let mut var939: u16 = 17u16;
    loop {
        let var93B: u16 = *r54.offset((v9 as i32 >> 1i32) as isize);
        let v17: u16 = v9;
        let v11: u16 = var937;
        let tile_index = (var937 >> 1) as usize;
        let ram = (&raw mut game.g_ram).cast::<u8>();
        let first = ram
            .add(crate::ram_map::word_arrays::bg1_column_update_tilemap_top_halves)
            .cast::<u16>()
            .add(tile_index);
        let second = ram
            .add(crate::ram_map::word_arrays::bg1_column_update_tilemap_bottom_halves)
            .cast::<u16>()
            .add(tile_index);
        TileTables::draw_block(
            ram.add(0xa000).cast(),
            var93B,
            [first, first.add(1), second, second.add(1)],
            TileDrawOrder::Rows,
        );
        var937 = (v11).wrapping_add(4u16);
        v9 = (v17).wrapping_add(2u16);
        var939 = var939.wrapping_sub(1);
        if (var939 == 0) {
            break;
        }
    }
    let ref mut fresh6 =
        *((ram_ptr!(game, u16; 0x970) as *mut u8).offset(t2 as i32 as isize) as *mut u16);
    *fresh6 = (*fresh6).wrapping_add(1);
}
pub unsafe fn FixDoorsMovingUp(game: &mut Game) {
    set_ram_word!(game.door_transition_frame_counter = 0u16);
    CalculateBgScrollAndLayerPositionBlocks(game);
    UpdatePreviousLayerBlocks(game);
    update_ram_word!(game.previous_layer1_y_block, wrapping_add(1));
    update_ram_word!(game.previous_layer2_y_block, wrapping_add(1));
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
    kDoorTransitionSetupFuncs[(ram_word!(game, door_direction) as i32 & 3i32) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn DoorTransitionScrollingSetup_Right(game: &mut Game) {
    CalculateLayer2Xpos(game);
    update_ram_word!(game.layer2_x_pos, wrapping_sub(256u16));
    CalculateLayer2Ypos(game);
    update_ram_word!(game.layer1_x_pos, wrapping_sub(256u16));
    UpdateBgScrollOffsets(game);
    CalculateBgScrollAndLayerPositionBlocks(game);
    UpdatePreviousLayerBlocks(game);
    update_ram_word!(game.previous_layer1_x_block, wrapping_sub(1));
    update_ram_word!(game.previous_layer2_x_block, wrapping_sub(1));
    DoorTransition_Right(game);
}
pub unsafe fn DoorTransitionScrollingSetup_Left(game: &mut Game) {
    CalculateLayer2Xpos(game);
    update_ram_word!(game.layer2_x_pos, wrapping_add(256u16));
    CalculateLayer2Ypos(game);
    update_ram_word!(game.layer1_x_pos, wrapping_add(256u16));
    UpdateBgScrollOffsets(game);
    CalculateBgScrollAndLayerPositionBlocks(game);
    UpdatePreviousLayerBlocks(game);
    update_ram_word!(game.previous_layer1_x_block, wrapping_add(1));
    update_ram_word!(game.previous_layer2_x_block, wrapping_add(1));
    DoorTransition_Left(game);
}
pub unsafe fn DoorTransitionScrollingSetup_Down(game: &mut Game) {
    CalculateLayer2Xpos(game);
    CalculateLayer2Ypos(game);
    update_ram_word!(game.layer2_y_pos, wrapping_sub(224u16));
    update_ram_word!(game.layer1_y_pos, wrapping_sub(224u16));
    UpdateBgScrollOffsets(game);
    CalculateBgScrollAndLayerPositionBlocks(game);
    UpdatePreviousLayerBlocks(game);
    update_ram_word!(game.previous_layer1_y_block, wrapping_sub(1));
    update_ram_word!(game.previous_layer2_y_block, wrapping_sub(1));
    DoorTransition_Down(game);
}
pub unsafe fn DoorTransitionScrollingSetup_Up(game: &mut Game) {
    CalculateLayer2Xpos(game);
    let v1: u16 = ram_word!(game, layer1_y_pos);
    update_ram_word!(game.layer1_y_pos, wrapping_add(31u16));
    CalculateLayer2Ypos(game);
    update_ram_word!(game.layer2_y_pos, wrapping_add(224u16));
    set_ram_word!(game.layer1_y_pos = (v1).wrapping_add(256u16));
    UpdateBgScrollOffsets(game);
    update_ram_word!(game.door_destination_y_pos, wrapping_add(32u16));
    CalculateBgScrollAndLayerPositionBlocks(game);
    UpdatePreviousLayerBlocks(game);
    update_ram_word!(game.previous_layer1_y_block, wrapping_add(1));
    update_ram_word!(game.previous_layer2_y_block, wrapping_add(1));
    update_ram_word!(game.layer1_y_pos, wrapping_sub(1));
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
        Some(DoorTransition_Right as unsafe fn(&mut Game) -> u8),
        Some(DoorTransition_Left as unsafe fn(&mut Game) -> u8),
        Some(DoorTransition_Down as unsafe fn(&mut Game) -> u8),
        Some(DoorTransition_Up as unsafe fn(&mut Game) -> u8),
    ]
};
pub unsafe fn Irq_FollowDoorTransition(game: &mut Game) {
    if kDoorTransitionFuncs[(ram_word!(game, door_direction) as i32 & 3i32) as usize]
        .expect("non-null function pointer")(game)
        != 0
    {
        set_ram_word!(game.layer1_x_pos = ram_word!(game, door_destination_x_pos));
        set_ram_word!(game.layer1_y_pos = ram_word!(game, door_destination_y_pos));
        update_ram_word!(game.door_transition_flag, |fresh68| (fresh68) | (0x8000u16));
    }
}
pub unsafe fn DoorTransition_Right(game: &mut Game) -> u8 {
    let v2: u16 = ram_word!(game, door_transition_frame_counter);
    AddToHiLo(
        ram_ptr!(game, u16; 0xaf6),
        ram_ptr!(game, u16; 0xaf8),
        (ram_word!(game, samus_door_transition_speed) as u32) << 16i32
            | ram_word!(game, samus_door_transition_subspeed) as u32,
    );
    set_ram_word!(game.samus_prev_x_pos = ram_word!(game, samus_x_pos));
    update_ram_word!(game.layer1_x_pos, wrapping_add(4u16));
    update_ram_word!(game.layer2_x_pos, wrapping_add(4u16));
    UpdateScrollVarsUpdateMap(game);
    set_ram_word!(game.door_transition_frame_counter = (v2).wrapping_add(1u16));
    if v2 != 63u16 {
        return 0u8;
    }
    UpdateScrollVarsUpdateMap(game);
    1u8
}
pub unsafe fn DoorTransition_Left(game: &mut Game) -> u8 {
    let v2: u16 = ram_word!(game, door_transition_frame_counter);
    AddToHiLo(
        ram_ptr!(game, u16; 0xaf6),
        ram_ptr!(game, u16; 0xaf8),
        -((ram_word!(game, samus_door_transition_speed) as i32) << 16i32
            | ram_word!(game, samus_door_transition_subspeed) as i32) as u32,
    );
    set_ram_word!(game.samus_prev_x_pos = ram_word!(game, samus_x_pos));
    update_ram_word!(game.layer1_x_pos, wrapping_sub(4u16));
    update_ram_word!(game.layer2_x_pos, wrapping_sub(4u16));
    UpdateScrollVarsUpdateMap(game);
    set_ram_word!(game.door_transition_frame_counter = (v2).wrapping_add(1u16));
    (v2 == 63u16) as u8
}
pub unsafe fn DoorTransition_Down(game: &mut Game) -> u8 {
    let v6: u16 = ram_word!(game, door_transition_frame_counter);
    if ram_word!(game, door_transition_frame_counter) != 0 {
        if ram_word!(game, door_transition_frame_counter) < 0x39u16 {
            AddToHiLo(
                ram_ptr!(game, u16; 0xafa),
                ram_ptr!(game, u16; 0xafc),
                (ram_word!(game, samus_door_transition_speed) as u32) << 16i32
                    | ram_word!(game, samus_door_transition_subspeed) as u32,
            );
            set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
            update_ram_word!(game.layer1_y_pos, wrapping_add(4u16));
            update_ram_word!(game.layer2_y_pos, wrapping_add(4u16));
            UpdateScrollVarsUpdateMap(game);
        }
    } else {
        let v5: u16 = ram_word!(game, reg_BG1VOFS);
        let v4: u16 = ram_word!(game, reg_BG2VOFS);
        let v3: u16 = ram_word!(game, layer1_y_pos);
        update_ram_word!(game.layer1_y_pos, wrapping_sub(15u16));
        let v2: u16 = ram_word!(game, layer2_y_pos);
        update_ram_word!(game.layer2_y_pos, wrapping_sub(15u16));
        CalculateBgScrollAndLayerPositionBlocks(game);
        UpdatePreviousLayerBlocks(game);
        update_ram_word!(game.previous_layer1_y_block, wrapping_sub(1));
        update_ram_word!(game.previous_layer2_y_block, wrapping_sub(1));
        UpdateScrollVarsUpdateMap(game);
        set_ram_word!(game.layer2_y_pos = v2);
        set_ram_word!(game.layer1_y_pos = v3);
        set_ram_word!(game.reg_BG2VOFS = v4);
        set_ram_word!(game.reg_BG1VOFS = v5);
    }
    set_ram_word!(game.door_transition_frame_counter = (v6).wrapping_add(1u16));
    if (v6).wrapping_add(1u16) < 0x39u16 {
        return 0u8;
    }
    UpdateScrollVarsUpdateMap(game);
    1u8
}
pub unsafe fn DoorTransition_Up(game: &mut Game) -> u8 {
    let v6: u16 = ram_word!(game, door_transition_frame_counter);
    if ram_word!(game, door_transition_frame_counter) != 0 {
        AddToHiLo(
            ram_ptr!(game, u16; 0xafa),
            ram_ptr!(game, u16; 0xafc),
            -((ram_word!(game, samus_door_transition_speed) as i32) << 16i32
                | ram_word!(game, samus_door_transition_subspeed) as i32) as u32,
        );
        set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
        update_ram_word!(game.layer1_y_pos, wrapping_sub(4u16));
        update_ram_word!(game.layer2_y_pos, wrapping_sub(4u16));
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
        let v5: u16 = ram_word!(game, reg_BG1VOFS);
        let v4: u16 = ram_word!(game, reg_BG2VOFS);
        let v3: u16 = ram_word!(game, layer1_y_pos);
        update_ram_word!(game.layer1_y_pos, wrapping_sub(16u16));
        let v2: u16 = ram_word!(game, layer2_y_pos);
        update_ram_word!(game.layer2_y_pos, wrapping_sub(16u16));
        CalculateBgScrollAndLayerPositionBlocks(game);
        UpdatePreviousLayerBlocks(game);
        update_ram_word!(game.previous_layer1_y_block, wrapping_add(1));
        update_ram_word!(game.previous_layer2_y_block, wrapping_add(1));
        UpdateScrollVarsUpdateMap(game);
        set_ram_word!(game.layer2_y_pos = v2);
        set_ram_word!(game.layer1_y_pos = v3);
        set_ram_word!(game.reg_BG2VOFS = v4);
        set_ram_word!(game.reg_BG1VOFS = v5);
    }
    set_ram_word!(game.door_transition_frame_counter = (v6).wrapping_add(1u16));
    (v6 == 56u16) as u8
}
pub unsafe fn ConfigureMode7RotationMatrix(game: &mut Game) {
    if ram_word!(game, irq_enable_mode7) != 0
        && (ram_word!(game, nmi_frame_counter_word) as i32 & 7i32) == 0i32
    {
        set_ram_word!(
            game.reg_M7B = kSinCosTable8bit_Sext
                [(ram_word!(game, mode7_rotation_angle) as u8 as i32 + 64i32) as usize]
                as u16
        );
        set_ram_word!(game.reg_M7C = -(ram_word!(game, reg_M7B) as i32) as u16);
        set_ram_word!(
            game.reg_M7A = kSinCosTable8bit_Sext[((((ram_word!(game, mode7_rotation_angle)) as u8)
                .wrapping_add(64u8)) as i32
                + 64i32) as usize] as u16
        );
        set_ram_word!(game.reg_M7D = ram_word!(game, reg_M7A));
        update_ram_word!(game.mode7_rotation_angle, wrapping_add(1));
    }
}

unsafe fn DecompNextByte(game: &mut Game) -> u8 {
    let b: u8 = *RomPtr(game.decompress_src, game);
    let fresh31 = game.decompress_src;
    game.decompress_src = game.decompress_src.wrapping_add(1);
    if (fresh31 & 0xffffu32) == 0xffffu32 {
        game.decompress_src = game.decompress_src.wrapping_add(0x8000u32);
    }
    b
}
pub unsafe fn DecompressToMem(src: u32, decompress_dst: *mut u8, game: &mut Game) {
    game.decompress_src = src;
    let mut src_pos: i32 = 0;
    let mut dst_pos: i32 = 0i32;
    loop {
        let mut len: i32 = 0;
        let mut cmd: u8 = 0;
        let mut b: u8 = 0;
        b = DecompNextByte(game);
        if b == 0xffu8 {
            break;
        }
        if (b as i32 & 0xe0i32) == 0xe0i32 {
            cmd = (8i32 * b as i32 & 0xe0i32) as u8;
            len = ((b as i32 & 3i32) << 8i32 | DecompNextByte(game) as i32) + 1i32;
        } else {
            cmd = (b) & (0xe0u8);
            len = (b as i32 & 0x1fi32) + 1i32;
        }
        if cmd as i32 & 0x80i32 != 0 {
            let want_xor: u8 = (if cmd as i32 & 0x20i32 != 0 {
                0xffi32
            } else {
                0i32
            }) as u8;
            if cmd >= 0xc0u8 {
                src_pos = dst_pos - DecompNextByte(game) as i32;
            } else {
                src_pos = DecompNextByte(game) as i32;
                src_pos += DecompNextByte(game) as i32 * 256i32;
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
            match cmd as i32 {
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
                    let b2: u8 = DecompNextByte(game);
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
unsafe fn ReadPpuByte(addr: u16, game: &mut Game) -> u8 {
    WriteRegWord(VMADDL as u16, (addr as i32 >> 1i32) as u16, game);
    ReadRegWord(RDVRAML as u16, game);
    let data: u16 = ReadRegWord(RDVRAML as u16, game);
    ((if addr as i32 & 1i32 != 0 {
        (data as i32 & 0xff00i32) >> 8i32
    } else {
        data as i32
    }) as u8)
}
pub unsafe fn DecompressToVRAM(src: u32, dst_addr: u16, game: &mut Game) {
    game.decompress_src = src;
    let mut src_pos: i32 = 0;
    let mut dst_pos: i32 = dst_addr as i32;
    loop {
        let mut len: i32 = 0;
        let mut b: u8 = DecompNextByte(game);
        let mut cmd: u8 = 0;
        if b == 0xffu8 {
            break;
        }
        if (b as i32 & 0xe0i32) == 0xe0i32 {
            cmd = (8i32 * b as i32 & 0xe0i32) as u8;
            len = ((b as i32 & 3i32) << 8i32 | DecompNextByte(game) as i32) + 1i32;
        } else {
            cmd = (b) & (0xe0u8);
            len = (b as i32 & 0x1fi32) + 1i32;
        }
        if cmd as i32 & 0x80i32 != 0 {
            let want_xor: u8 = (if cmd as i32 & 0x20i32 != 0 {
                0xffi32
            } else {
                0i32
            }) as u8;
            if cmd >= 0xc0u8 {
                src_pos = dst_pos - DecompNextByte(game) as i32;
            } else {
                src_pos = DecompNextByte(game) as i32;
                src_pos += DecompNextByte(game) as i32 * 256i32;
                src_pos += dst_addr as i32;
            }
            loop {
                let fresh32 = src_pos;
                src_pos = src_pos + 1;
                b = (ReadPpuByte(fresh32 as u16, game)) ^ (want_xor);
                WriteRegWord(VMADDL as u16, (dst_pos >> 1i32) as u16, game);
                let fresh33 = dst_pos;
                dst_pos = dst_pos + 1;
                WriteReg((VMDATAL as i32 + (fresh33 & 1i32)) as u16, b, game);
                len -= 1;
                if (len == 0) {
                    break;
                }
            }
        } else {
            match cmd as i32 {
                32 => {
                    b = DecompNextByte(game);
                    loop {
                        let fresh34 = dst_pos;
                        dst_pos = dst_pos + 1;
                        WriteReg((VMDATAL as i32 + (fresh34 & 1i32)) as u16, b, game);
                        len -= 1;
                        if (len == 0) {
                            break;
                        }
                    }
                }
                64 => {
                    b = DecompNextByte(game);
                    let b2: u8 = DecompNextByte(game);
                    loop {
                        let fresh35 = dst_pos;
                        dst_pos = dst_pos + 1;
                        WriteReg((VMDATAL as i32 + (fresh35 & 1i32)) as u16, b, game);
                        len -= 1;
                        if len == 0 {
                            break;
                        }
                        let fresh36 = dst_pos;
                        dst_pos = dst_pos + 1;
                        WriteReg((VMDATAL as i32 + (fresh36 & 1i32)) as u16, b2, game);
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
                        WriteReg((VMDATAL as i32 + (fresh37 & 1i32)) as u16, fresh38, game);
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
                    WriteReg((VMDATAL as i32 + (fresh39 & 1i32)) as u16, fresh40, game);
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
    set_ram_word!(game.save_station_lockout_flag = 1u16);
    let L: *const LoadStationList = RomPtr_80(
        (crate::rtl::rom_table_word!(game, 0x80c4b5, ram_word!(game, area_index) as isize) as i32
            + 14i32 * ram_word!(game, load_station_index) as i32) as u16,
        game,
    ) as *mut LoadStationList;
    set_ram_word!(game.room_ptr = (*L).room_ptr_);
    set_ram_word!(game.door_def_ptr = (*L).door_ptr);
    let ref mut fresh77 = *(ram_ptr!(game, u16; 0x911));
    *fresh77 = (*L).screen_x_pos;
    set_ram_word!(game.bg1_x_offset = *fresh77);
    let ref mut fresh78 = *(ram_ptr!(game, u16; 0x915));
    *fresh78 = (*L).screen_y_pos;
    set_ram_word!(game.bg1_y_offset = *fresh78);
    set_ram_word!(
        game.samus_y_pos = (ram_word!(game, layer1_y_pos)).wrapping_add((*L).samus_y_offset)
    );
    set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
    set_ram_word!(
        game.samus_x_pos =
            (ram_word!(game, layer1_x_pos) as i32 + 128i32 + (*L).samus_x_offset as i32) as u16
    );
    set_ram_word!(game.samus_prev_x_pos = ram_word!(game, samus_x_pos));
    set_ram_word!(game.reg_BG1HOFS = 0u16);
    set_ram_word!(game.reg_BG1VOFS = 0u16);
    *(ram_ptr!(game, u16; 0x79f) as *mut u8).offset(0) =
        (*get_RoomDefHeader(ram_word!(game, room_ptr), game)).area_index_;
    *(ram_ptr!(game, u16; 0x5f7) as *mut u8).offset(0) = 0u8;
}
pub unsafe fn SetElevatorsAsUsed(game: &mut Game) {
    let v0: *const u8 = RomPtr_80(
        (crate::rtl::rom_table_word!(game, 0x80cd46, ram_word!(game, area_index) as isize) as i32
            + 4i32
                * ((ram_word!(game, elevator_door_properties_orientation) as i32 & 0xfi32) - 1i32))
            as u16,
        game,
    );
    let ref mut fresh92 = *ram_ptr!(game; 0xd8f8, *v0.offset(0) as isize);
    *fresh92 = (*fresh92) | (*v0.offset(1));
    let ref mut fresh93 = *ram_ptr!(game; 0xd8f8, *v0.offset(2) as isize);
    *fresh93 = (*fresh93) | (*v0.offset(3));
}
