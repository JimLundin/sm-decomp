use crate::cpu_infra::DebugGameOverMenu;
pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_82::*;
pub use crate::game::constants::bank_8f::*;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::game_states::*;
pub use crate::game::constants::loading_game_states::*;
pub use crate::game::constants::samus_poses::*;
use crate::game::sm_80::{
    memset7E, CalculateLayer2PosAndScrollsWhenScrolling, CheckEventHappened, ClearBG2Tilemap,
    ClearFXTilemap, ClearOamExt, ClearTimerRam, ClearUnusedOam, CopyToVramNow,
    DebugScrollPosSaveLoad, DecompressToMem, DecompressToVRAM, DisableIrqInterrupts,
    DoorTransitionScrollingSetup, DrawTimer, EnableIrqInterruptsNow, EnableNMI, FixDoorsMovingUp,
    HandleFadeIn, HandleFadeOut, HandleHudTilemap, HasQueuedMusic, InitializeHud,
    LoadFromLoadStation, LoadMirrorOfExploredMapTiles, NextRandom, PrepareBitAccess,
    QueueClearingOfFxTilemap, QueueMusic_Delayed8, QueueMusic_DelayedY, QueueSfx1_Max15,
    QueueSfx1_Max6, QueueSfx2_Max15, QueueSfx2_Max6, QueueSfx3_Max15, QueueSfx3_Max3,
    QueueSfx3_Max6, ReadJoypadInputs, ReleaseButtonsFilter, ResumeGameplay,
    SaveExploredMapTilesToSaved, SetElevatorsAsUsed, SetupDmaTransfer, StartGameplay_Async,
    Vector_NMI, Vector_RESET_Async, WaitForNMI_Async,
};
use crate::game::sm_81::{
    DrawMenuSpritemap, DrawSpritemap, FileSelectMap, FileSelectMenu, GameOverMenu, SaveToSram,
};
use crate::game::sm_84::{
    ClearPLMs, ClearSoundsWhenGoingThroughDoor, EnablePLMs, PlaySpinJumpSoundIfSpinJumping,
    PlmHandler_Async, SpawnRoomPLM,
};
use crate::game::sm_85::DisplayMessageBox_Async;
use crate::game::sm_86::{ClearEprojs, EnableEprojs, EprojRunAll};
use crate::game::sm_87::{AnimtilesHandler, ClearAnimtiles, DisableAnimtiles, EnableAnimtiles};
use crate::game::sm_88::{
    sub_8882AC, DisableHdmaObjects, EnableHdmaObjects, HdmaObjectHandler,
    InitializeSpecialEffectsForNewRoom, SpawnBG3ScrollHdmaObject, WaitUntilEndOfVblankAndClearHdma,
};
use crate::game::sm_89::LoadFXHeader;
use crate::game::sm_8b::{
    GameState_1_OpeningCinematic_, GameState_37_CeresGoesBoomWithSamus_,
    GameState_39_EndingAndCredits_, LoadTitleSequenceGraphics,
};
use crate::game::sm_8d::{
    ClearPaletteFXObjects, DisablePaletteFx, EnablePaletteFx, PaletteFxHandler,
};
use crate::game::sm_8f::{
    HandleRoomDefStateSelect, PauseHook_DraygonRoom, RunDoorSetupCode, RunRoomMainCode,
    RunRoomSetupCode, UnpauseHook_DraygonRoom,
};
use crate::game::sm_90::{
    CallSomeSamusCode, HandleControllerInputForGamePhysics, HandleSamusMovementAndPause,
    MainScrollingRoutine, ResetProjectileData, Samus_DrawDuringDeathAnim,
    Samus_DrawWhenNotAnimatingOrDying, Samus_LowHealthCheck_0, SetLiquidPhysicsType,
    UpdateBeamTilesAndPalette,
};
use crate::game::sm_91::{
    GameState_28_Unused_, SamusFunc_F433, Samus_Initialize, Samus_LoadSuitTargetPalette,
    Samus_SetAnimationFrameIfPoseChanged,
};
use crate::game::sm_9b::{
    DrawSamusStartingDeathAnim_, GameState_24_SamusNoHealth_Explosion_Helper,
    HandleSamusDeathSequence, StartSamusDeathAnimation,
};
use crate::game::sm_a0::{
    DecrementSamusTimers, DetermineWhichEnemiesToProcess, DrawSamusEnemiesAndProjectiles,
    EnemyMain, EprojProjCollDet, EprojSamusCollDetect, HandleRoomShaking, InitializeEnemies,
    LoadEnemies, ProcessEnemyPowerBombInteraction, QueueEnemyBG2TilemapTransfers,
    SamusProjectileInteractionHandler, TransferEnemyTilesToVramAndInit,
};
use crate::game::sm_a7::{
    Kraid_UnpauseHook_IsSinking, PauseHook_Kraid, UnpauseHook_Kraid_IsAlive,
    UnpauseHook_Kraid_IsDead,
};
use crate::game::sm_a9::MotherBrainsBody_UnpauseHook;
use crate::game_state::Game;
use crate::host::{Die, Warning};
use crate::rtl::abs16;
use crate::rtl::{
    Load24, Mult8x8, RomPtr, RtlApuWrite, SnesDivide, Unreachable, WriteReg, WriteRegWord,
};
pub use crate::snes::registers::*;
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;

}
pub use crate::types::RoomPlmEntry;
pub use crate::types::{
    __int16_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int8, int8_t, size_t,
    uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, CoroutineRet, Func_V, Func_V_Coroutine,
    Func_Y_V, Func_Y_Y, LongPtr, SnesRegs, VoidP, VramWriteEntry,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct PlmHeader_Size6 {
    pub func_ptr: VoidP,
    pub instr_list_1_ptr: VoidP,
    pub instr_list_2_ptr: VoidP,
}
pub use crate::types::EnemyData;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct LoadBg_E {
    pub field_0: uint16,
    pub field_2: uint16,
}
pub use crate::types::Buttons;
pub use crate::types::LoadingGameState;
pub use crate::types::{DoorDef, SamusPose};
pub use crate::types::{EnemyTileset, GameState, RoomDefHeader};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct TileSet {
    pub tile_table_ptr: LongPtr,
    pub tiles_ptr: LongPtr,
    pub palette_ptr: LongPtr,
}
pub use crate::types::{
    ExpandingSquareTransitionHdma, MapIconDataPointers, MapScrollArrowData, RoomDefRoomstate,
    StartDmaCopy,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct DemoRoomData {
    pub room_ptr_: VoidP,
    pub door_ptr: uint16,
    pub door_slot: uint16,
    pub screen_x_pos: uint16,
    pub screen_y_pos: uint16,
    pub samus_x_offs: uint16,
    pub samus_y_offs: uint16,
    pub demo_length: uint16,
    pub demo_code_ptr: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union PauseScreenSpriteAnimationData {
    pub c2rust_unnamed: C2RustUnnamed,
    pub arr: [VoidP; 9],
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct C2RustUnnamed {
    pub unused: VoidP,
    pub lr_highlight: VoidP,
    pub item_selector: VoidP,
    pub unused2: VoidP,
    pub unused3: VoidP,
    pub map_scroll_arrow_up: VoidP,
    pub map_scroll_arrow_down: VoidP,
    pub map_scroll_arrow_right: VoidP,
    pub map_scroll_arrow_left: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct FileCopyArrowStuff {
    pub spritemap: uint16,
    pub xpos: uint16,
    pub ypos: uint16,
}
pub use crate::types::Consts_82;
pub use crate::types::Consts_83;
pub use crate::types::{
    Consts_60, EnemyDef, Ram3000, Ram3000_Menu, Ram3000_Misc, Ram3000_MsgBox, Ram3800, Ram4000,
    Ram4000_Backups,
};
pub const addr_kDoorDef_947a: Consts_83 = 38010;
pub use crate::types::Consts_8F;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"uint8 *RomPtr_RAM(uint16_t)\0")
};
pub const kCoroutineNone: ::core::ffi::c_int = 0;
#[inline]
unsafe fn RomPtr_RAM(mut addr: uint16_t, game: &mut Game) -> *mut uint8 {
    '_c2rust_label: {
        if (addr as ::core::ffi::c_int) < 0x2000 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"addr < 0x2000\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_rtl.h\0" as *const u8 as *const ::core::ffi::c_char,
                51 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    return (&raw mut game.g_ram as *mut uint8).offset(addr as ::core::ffi::c_int as isize);
}
#[inline]
unsafe fn RomPtr_82(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(((0x820000u32) | ((addr) as u32)), game);
}
#[inline]
unsafe fn RomPtr_8F(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(((0x8f0000u32) | ((addr) as u32)), game);
}
#[inline]
unsafe fn RomPtrWithBank(mut bank: uint8, mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        ((bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int | addr as ::core::ffi::c_int)
            as uint32_t,
        game,
    );
}
#[inline]
unsafe fn gVramWriteEntry(mut a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    return (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry;
}
#[inline]
unsafe fn get_DoorDef(mut a: uint16, game: &mut Game) -> *mut DoorDef {
    return RomPtr(((0x830000u32) | ((a) as u32)), game) as *mut DoorDef;
}
#[inline]
unsafe fn get_RoomDefHeader(mut a: uint16, game: &mut Game) -> *mut RoomDefHeader {
    return RomPtr(((0x8f0000u32) | ((a) as u32)), game) as *mut RoomDefHeader;
}
#[inline]
unsafe fn get_RoomDefRoomstate(mut a: uint16, game: &mut Game) -> *mut RoomDefRoomstate {
    return RomPtr(((0x8f0000u32) | ((a) as u32)), game) as *mut RoomDefRoomstate;
}
#[inline]
unsafe fn get_TileSet(mut a: uint16, game: &mut Game) -> *mut TileSet {
    return RomPtr(((0x8f0000u32) | ((a) as u32)), game) as *mut TileSet;
}
#[inline]
unsafe fn get_EnemyDef_A2(mut a: uint16, game: &mut Game) -> *mut EnemyDef {
    return RomPtr(((0xa00000u32) | ((a) as u32)), game) as *mut EnemyDef;
}
#[inline]
unsafe fn get_EnemyTileset(mut a: uint16, game: &mut Game) -> *mut EnemyTileset {
    return RomPtr(((0xb40000u32) | ((a) as u32)), game) as *mut EnemyTileset;
}
#[inline]
unsafe fn get_RoomPlmEntry(mut a: uint16, game: &mut Game) -> *mut RoomPlmEntry {
    return RomPtr(((0x8f0000u32) | ((a) as u32)), game) as *mut RoomPlmEntry;
}
#[inline]
unsafe fn get_DemoRoomData(mut a: uint16, game: &mut Game) -> *mut DemoRoomData {
    return RomPtr(((0x820000u32) | ((a) as u32)), game) as *mut DemoRoomData;
}
#[inline]
unsafe fn get_LoadBg_E(mut a: uint16, game: &mut Game) -> *mut LoadBg_E {
    return RomPtr(((0x8f0000u32) | ((a) as u32)), game) as *mut LoadBg_E;
}
#[inline]
unsafe fn get_PlmHeader_Size6(mut a: uint16, game: &mut Game) -> *mut PlmHeader_Size6 {
    return RomPtr(((0x840000u32) | ((a) as u32)), game) as *mut PlmHeader_Size6;
}
pub const fnDemoRoom_ChargeBeamRoomScroll21: uint32 = 8554778;
pub const fnnullsub_291: uint32 = 8554788;
pub const fnDemoRoom_SetBG2TilemapBase: uint32 = 8554789;
pub const fnDemoRoom_SetKraidFunctionTimer: uint32 = 8554795;
pub const fnDemoRoom_SetBrinstarBossBits: uint32 = 8554802;
pub const fnOptionsPreInstr_nullsub_57: uint32 = 8555536;
pub const fnOptionsInstr_Destroy: uint32 = 8555610;
pub const fnOptionsInstr_8C64: uint32 = 8555620;
pub const fnOptionsInstr_SetPreInstr: uint32 = 8555630;
pub const fnOptionsInstr_8C79: uint32 = 8555641;
pub const fnOptionsInstr_Goto: uint32 = 8555650;
pub const fnOptionsInstr_8C89: uint32 = 8555657;
pub const fnOptionsInstr_8C93: uint32 = 8555667;
pub const fnBabyMetroidPlaySfx0x23: uint32 = 8567820;
pub const fnBabyMetroidPlaySfx0x26: uint32 = 8567829;
pub const fnBabyMetroidPlaySfx0x27: uint32 = 8567838;
pub const fnDoorTransitionFunction_HandleElevator: uint32 = 8577405;
pub const fnDoorTransitionFunction_Wait48frames: ::core::ffi::c_int = 0x82e19f;
pub const fnDoorTransitionFunction_WaitForSoundsToFinish: ::core::ffi::c_int = 0x82e29e;
pub const fnDoorTransitionFunction_FadeOutScreen: ::core::ffi::c_int = 0x82e2db;
pub const fnDoorTransitionFunction_LoadDoorHeaderEtc: ::core::ffi::c_int = 0x82e2f7;
pub const fnDoorTransitionFunction_ScrollScreenToAlignment: ::core::ffi::c_int = 0x82e310;
pub const fnDoorTransitionFunction_FixDoorsMovingUp: ::core::ffi::c_int = 0x82e353;
pub const fnDoorTransitionFunction_SetupNewRoom: ::core::ffi::c_int = 0x82e36e;
pub const fnDoorTransitionFunction_SetupScrolling: ::core::ffi::c_int = 0x82e38e;
pub const fnDoorTransitionFunction_PlaceSamusLoadTiles: ::core::ffi::c_int = 0x82e3c0;
pub const fnDoorTransitionFunction_LoadMoreThings_Async: ::core::ffi::c_int = 0x82e4a9;
pub const fnDoorTransition_C_HandleAnimTiles: ::core::ffi::c_int = 0x82e659;
pub const fnDoorTransition_WaitForMusicToClear: ::core::ffi::c_int = 0x82e664;
pub const fnDoorTransition_HandleTransition: ::core::ffi::c_int = 0x82e6a2;
pub const fnDoorTransition_FadeInScreenAndFinish: ::core::ffi::c_int = 0x82e737;
pub const fnsub_82F296: uint32 = 8581782;
pub const fnOptionsPreInstr_F2A9: uint32 = 8581801;
pub const fnsub_82F34B: uint32 = 8581963;
pub const fnsub_82F353: uint32 = 8581971;
pub const fnsub_82F35B: uint32 = 8581979;
pub const fnsub_82F363: uint32 = 8581987;
pub const fnOptionsPreInstr_F376: uint32 = 8582006;
pub const fnOptionsPreInstr_F3A0: uint32 = 8582048;
pub const fnOptionsPreInstr_F3E2: uint32 = 8582114;
pub const fnsub_82F419: uint32 = 8582169;
pub const fnOptionsPreInstr_F42C: uint32 = 8582188;
pub const fnPauseHook_Empty: uint32 = 8946657;
pub const fnCinematicFunctionNone: ::core::ffi::c_int = 0x8b9a47;
pub const fnCinematicFunctionOpening: ::core::ffi::c_int = 0x8b9b68;
pub const fnCinematicFunction_Intro_Initial: ::core::ffi::c_int = 0x8ba395;
pub const fnCinematicFunctionBlackoutFromCeres: ::core::ffi::c_int = 0x8bc11b;
pub const fnCinematicFunctionEscapeFromCebes: ::core::ffi::c_int = 0x8bd480;
pub const fnPauseHook_DraygonRoom: uint32 = 9423094;
pub const fnUnpauseHook_DraygonRoom: uint32 = 9423100;
pub const fnUnpauseHook_Kraid_IsDead: uint32 = 10994171;
pub const fnUnpauseHook_Kraid_IsAlive: uint32 = 10994254;
pub const fnKraid_UnpauseHook_IsSinking: uint32 = 10994336;
pub const fnPauseHook_Kraid: uint32 = 10994469;
pub const fnMotherBrainsBody_UnpauseHook: uint32 = 11110243;
pub const fnlocret_828C81: ::core::ffi::c_int = 0x828c81;
pub unsafe fn CallDemoRoomDataFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        8554778 => {
            DemoRoom_ChargeBeamRoomScroll21(game);
            return;
        }
        8554788 => return,
        8554789 => {
            DemoRoom_SetBG2TilemapBase(game);
            return;
        }
        8554795 => {
            DemoRoom_SetKraidFunctionTimer(game);
            return;
        }
        8554802 => {
            DemoRoom_SetBrinstarBossBits(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn InitAndLoadGameData_Async(game: &mut Game) -> CoroutineRet {
    let mut v0: int16 = 0;
    let mut v2: int16 = 0;
    let mut v4: int16 = 0;
    let mut v7: int16 = 0;
    let mut _state_variable_: *mut uint8 = (&raw mut game.g_ram as *mut uint8).offset(0x612);
    's_445: {
        let mut current_block_103: u64;
        match *_state_variable_ as ::core::ffi::c_int {
            0 => {
                if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16)
                    as ::core::ffi::c_int
                    == kGameState_40_TransitionToDemo as ::core::ffi::c_int
                {
                    InitIoForGameplay(game);
                    LoadStdBG3andSpriteTilesClearTilemaps(game);
                    LoadInitialPalette(game);
                    Samus_Initialize(game);
                    LoadDemoRoomData(game);
                } else {
                    if *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16)
                        as ::core::ffi::c_int
                        != kLoadingGameState_5_Main as ::core::ffi::c_int
                    {
                        if *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16)
                            as ::core::ffi::c_int
                            == kLoadingGameState_1F_StartingAtCeres as ::core::ffi::c_int
                        {
                            *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) =
                                6 as uint16;
                            *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16) =
                                0 as uint16;
                            ClearTimerRam(game);
                        } else if *((&raw mut game.g_ram as *mut uint8).offset(0xd914)
                            as *mut uint16) as ::core::ffi::c_int
                            == kLoadingGameState_22_EscapingCeres as ::core::ffi::c_int
                        {
                            *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) =
                                0 as uint16;
                            *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16) =
                                18 as uint16;
                            LoadMirrorOfExploredMapTiles(game);
                        }
                    }
                    InitIoForGameplay(game);
                    LoadStdBG3andSpriteTilesClearTilemaps(game);
                    LoadInitialPalette(game);
                    Samus_Initialize(game);
                    LoadFromLoadStation(game);
                }
                current_block_103 = 9959648584402098530;
            }
            1 => {
                current_block_103 = 9959648584402098530;
            }
            2 => {
                current_block_103 = 6249748917338837630;
            }
            3 => {
                current_block_103 = 4453751417072702604;
            }
            4 => {
                current_block_103 = 12557724779959852293;
            }
            _ => {
                current_block_103 = 12705158477165241210;
            }
        }
        match current_block_103 {
            9959648584402098530 => {
                let mut _coret_: uint8 = StartGameplay_Async(game);
                if _coret_ != 0 {
                    *_state_variable_ = 1 as uint8;
                    return _coret_;
                }
                InitializeHud(game);
                v0 = 32 as int16;
                let mut v1: uint16 = 0 as uint16;
                loop {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(
                        ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                            + 192 as ::core::ffi::c_int) as isize,
                    ) = *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset(
                            ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                                + 192 as ::core::ffi::c_int) as isize,
                        );
                    v1 = ((v1).wrapping_add((2u16)));
                    v0 = ((v0).wrapping_sub((2i16)));
                    if !(v0 != 0) {
                        break;
                    }
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 1 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 1 as uint16;
                EnableNMI(game);
                EnableEprojs(game);
                EnablePLMs(game);
                EnablePaletteFx(game);
                EnableHdmaObjects(game);
                EnableAnimtiles(game);
                SetLiquidPhysicsType(game);
                if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16)
                    as ::core::ffi::c_int
                    == kGameState_40_TransitionToDemo as ::core::ffi::c_int
                {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xda0) as *mut uint16) =
                        6 as uint16;
                    current_block_103 = 980989089337379490;
                } else if *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16)
                    as ::core::ffi::c_int
                    == kLoadingGameState_22_EscapingCeres as ::core::ffi::c_int
                {
                    QueueMusic_Delayed8(5 as uint16, game);
                    *((&raw mut game.g_ram as *mut uint8).offset(0xda0) as *mut uint16) =
                        15 as uint16;
                    current_block_103 = 10753070352654377903;
                } else {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xda0) as *mut uint16) =
                        6 as uint16;
                    current_block_103 = 129780949503461575;
                }
            }
            _ => {}
        }
        loop {
            match current_block_103 {
                129780949503461575 => {
                    TransferEnemyTilesToVramAndInit(game);
                    current_block_103 = 12557724779959852293;
                }
                4453751417072702604 => {
                    let mut _coret__1: uint8 = WaitForNMI_Async(game);
                    if _coret__1 != 0 {
                        *_state_variable_ = 3 as uint8;
                        return _coret__1;
                    }
                    let ref mut fresh141 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0xda0) as *mut uint16);
                    *fresh141 = (*fresh141).wrapping_sub(1);
                    if *((&raw mut game.g_ram as *mut uint8).offset(0xda0) as *mut uint16)
                        as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        current_block_103 = 10753070352654377903;
                        continue;
                    }
                    let ref mut fresh142 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
                    *fresh142 = (*fresh142).wrapping_add(1);
                    v2 = 512 as int16;
                    let mut v3: uint16 = 0 as uint16;
                    loop {
                        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                            .offset(
                                (v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                            ) = *((&raw mut game.g_ram as *mut uint8).offset(0xc200)
                            as *mut uint16)
                            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
                        v3 = ((v3).wrapping_add((2u16)));
                        v2 = ((v2).wrapping_sub((2i16)));
                        if !(v2 != 0) {
                            break;
                        }
                    }
                    current_block_103 = 12705158477165241210;
                }
                6249748917338837630 => {
                    let mut _coret__0: uint8 = WaitForNMI_Async(game);
                    if _coret__0 != 0 {
                        *_state_variable_ = 2 as uint8;
                        return _coret__0;
                    }
                    let ref mut fresh139 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0xda0) as *mut uint16);
                    *fresh139 = (*fresh139).wrapping_sub(1);
                    if *((&raw mut game.g_ram as *mut uint8).offset(0xda0) as *mut uint16)
                        as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        current_block_103 = 980989089337379490;
                        continue;
                    }
                    let mut r18: uint16 = (*get_DemoRoomData(
                        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82876c)) as *const uint8
                            as *mut uint16)
                            .offset(
                                *((&raw mut game.g_ram as *mut uint8).offset(0x1f55) as *mut uint16)
                                    as isize,
                            ) as ::core::ffi::c_int
                            + 18 as ::core::ffi::c_int
                                * (*((&raw mut game.g_ram as *mut uint8).offset(0x1f57)
                                    as *mut uint16)
                                    as ::core::ffi::c_int
                                    - 1 as ::core::ffi::c_int)) as uint16,
                        game,
                    ))
                    .demo_code_ptr;
                    CallDemoRoomDataFunc((((r18) as u32) | (0x820000u32)), game);
                    let ref mut fresh140 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
                    *fresh140 = (*fresh140).wrapping_add(1);
                    v7 = 512 as int16;
                    let mut v8: uint16 = 0 as uint16;
                    loop {
                        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                            .offset(
                                (v8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                            ) = *((&raw mut game.g_ram as *mut uint8).offset(0xc200)
                            as *mut uint16)
                            .offset((v8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
                        v8 = ((v8).wrapping_add((2u16)));
                        v7 = ((v7).wrapping_sub((2i16)));
                        if !(v7 != 0) {
                            break;
                        }
                    }
                    current_block_103 = 12705158477165241210;
                }
                12705158477165241210 => {
                    break 's_445;
                }
                980989089337379490 => {
                    TransferEnemyTilesToVramAndInit(game);
                    current_block_103 = 6249748917338837630;
                }
                10753070352654377903 => {
                    TransferEnemyTilesToVramAndInit(game);
                    current_block_103 = 4453751417072702604;
                }
                _ => {
                    let mut _coret__2: uint8 = WaitForNMI_Async(game);
                    if _coret__2 != 0 {
                        *_state_variable_ = 4 as uint8;
                        return _coret__2;
                    }
                    let ref mut fresh143 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0xda0) as *mut uint16);
                    *fresh143 = (*fresh143).wrapping_sub(1);
                    if *((&raw mut game.g_ram as *mut uint8).offset(0xda0) as *mut uint16)
                        as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        current_block_103 = 129780949503461575;
                        continue;
                    }
                    *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
                        kGameState_7_MainGameplayFadeIn as uint16;
                    v4 = 512 as int16;
                    let mut v5: uint16 = 0 as uint16;
                    loop {
                        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                            .offset(
                                (v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                            ) = *((&raw mut game.g_ram as *mut uint8).offset(0xc200)
                            as *mut uint16)
                            .offset((v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
                        v5 = ((v5).wrapping_add((2u16)));
                        v4 = ((v4).wrapping_sub((2i16)));
                        if !(v4 != 0) {
                            break;
                        }
                    }
                    if *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16)
                        as ::core::ffi::c_int
                        == kLoadingGameState_1F_StartingAtCeres as ::core::ffi::c_int
                    {
                        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                            .offset(223) = 0 as uint16;
                        CallSomeSamusCode(8 as uint16, game);
                    } else {
                        CallSomeSamusCode(9 as uint16, game);
                    }
                    current_block_103 = 12705158477165241210;
                }
            }
        }
    }
    *_state_variable_ = 0 as uint8;
    return 0 as CoroutineRet;
}
pub unsafe fn InitIoForGameplay(game: &mut Game) {
    InitCpuForGameplay(game);
    InitPpuForGameplay(game);
}
pub unsafe fn InitCpuForGameplay(game: &mut Game) {
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
pub unsafe fn InitPpuForGameplay(game: &mut Game) {
    WriteReg(INIDISP as uint16, 0x80 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x80 as uint8;
    WriteReg(OBSEL as uint16, 3 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x52) = 3 as uint8;
    WriteReg(OAMADDL as uint16, 0 as uint8, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x53) as *mut uint16 as *mut uint8).offset(0) =
        0 as uint8;
    WriteReg(OAMADDH as uint16, 0x80 as uint8, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x53) as *mut uint16 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) = 0x80 as uint8;
    WriteReg(OAMDATA as uint16, 0 as uint8, game);
    WriteReg(OAMDATA as uint16, 0 as uint8, game);
    WriteReg(BGMODE as uint16, 9 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 9 as uint8;
    WriteReg(MOSAIC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x57) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 0 as uint8;
    WriteReg(BG12NBA as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 4 as uint8;
    WriteReg(BG34NBA as uint16, 4 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 81 as uint8;
    WriteReg(BG1SC as uint16, 0x51 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 73 as uint8;
    WriteReg(BG2SC as uint16, 0x49 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5a) = 90 as uint8;
    WriteReg(BG3SC as uint16, 0x5a as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5c) = 0 as uint8;
    WriteReg(BG4SC as uint16, 0 as uint8, game);
    WriteReg(VMAIN as uint16, 0 as uint8, game);
    WriteReg(W12SEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 0 as uint8;
    WriteReg(W12SEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 0 as uint8;
    WriteReg(W34SEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 0 as uint8;
    WriteReg(WOBJSEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 0 as uint8;
    WriteReg(WH0 as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x63) = 0 as uint8;
    WriteReg(WH1 as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x64) = 0 as uint8;
    WriteReg(WH2 as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x65) = 0 as uint8;
    WriteReg(WH3 as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x66) = 0 as uint8;
    WriteReg(WBGLOG as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x67) = 0 as uint8;
    WriteReg(WOBJLOG as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x68) = 0 as uint8;
    WriteReg(TM as uint16, 0x17 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 23 as uint8;
    WriteReg(TMW as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
    WriteReg(TS as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
    WriteReg(TSW as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
    WriteReg(CGWSEL as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
    WriteReg(CGADSUB as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 0 as uint8;
    WriteReg(COLDATA as uint16, 0xe0 as uint8, game);
    WriteReg(SETINI as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x77) = 0 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = 0 as uint16;
    memset7E(
        (&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000 as *mut uint16,
        0 as uint16,
        0x7fe as uint16,
    );
    memset7E(
        (&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000 as *mut uint16,
        0x6f as uint16,
        0x7fe as uint16,
    );
    memset7E(
        &raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).bg2_tilemap
            as *mut uint16,
        0x2c0f as uint16,
        0xfe as uint16,
    );
}
pub unsafe fn LoadInitialPalette(game: &mut Game) {
    let mut v0: int16 = 0;
    v0 = 512 as int16;
    let mut v1: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x9a8000)) as *const uint8 as *mut uint16)
                .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v1 = ((v1).wrapping_add((2u16)));
        v0 = ((v0).wrapping_sub((2i16)));
        if !(v0 != 0) {
            break;
        }
    }
}
pub unsafe fn LoadStdBG3andSpriteTilesClearTilemaps(game: &mut Game) {
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x40 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_8282F8: StartDmaCopy = StartDmaCopy::vram_words(1, 0x9ab200, 0x2000);
    SetupDmaTransfer(&raw const unk_8282F8 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x60 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_828318: StartDmaCopy = StartDmaCopy::vram_words(1, 0x9ad200, 0x2e00);
    SetupDmaTransfer(&raw const unk_828318 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x50 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_828338: StartDmaCopy = StartDmaCopy::vram_words(1, 0x7e4000, 0x1000);
    SetupDmaTransfer(&raw const unk_828338 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x58 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_828358: StartDmaCopy = StartDmaCopy::vram_words(1, 0x7e4000, 0x800);
    SetupDmaTransfer(&raw const unk_828358 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
}
pub unsafe fn GameState_32_MadeItToCeresElevator(game: &mut Game) -> CoroutineRet {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x943) as *mut uint16) != 0 {
        DrawTimer(game);
    }
    let mut _coret_: uint8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    let ref mut fresh124 = *((&raw mut game.g_ram as *mut uint8).offset(0xaa0) as *mut uint16);
    *fresh124 = (*fresh124).wrapping_sub(1);
    let mut v0: bool =
        *fresh124 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xaa0) as *mut uint16) == 0 || v0 {
        let ref mut fresh125 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
        *fresh125 = (*fresh125).wrapping_add(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_33_BlackoutFromCeres(game: &mut Game) -> CoroutineRet {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x943) as *mut uint16) != 0 {
        DrawTimer(game);
    }
    let mut _coret_: uint8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 0x80 as ::core::ffi::c_int
    {
        EnableNMI(game);
        WaitUntilEndOfVblankAndClearHdma(game);
        DisableIrqInterrupts(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1982) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xab) as *mut uint16) = 0 as uint16;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 16 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 9 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16) =
            kLoadingGameState_22_EscapingCeres as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) = 34 as uint16;
        SaveToSram(
            *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16),
            game,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (((0x8bc11bi32) as u16) & (0xffffu16));
        *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x943) as *mut uint16) = 0 as uint16;
        QueueMusic_Delayed8(0 as uint16, game);
        QueueSfx1_Max15(2 as uint16, game);
        QueueSfx2_Max15(0x71 as uint16, game);
        QueueSfx3_Max15(1 as uint16, game);
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_35_TimeUp(game: &mut Game) -> CoroutineRet {
    let mut _coret_: uint8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xc402) as *mut uint16) = 8 as uint16;
    if AdvancePaletteFadeForAllPalettes(game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
            kGameState_36_WhitingOutFromTimeUp as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_36_WhitingOutFromTimeUp(game: &mut Game) -> CoroutineRet {
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 0x80 as ::core::ffi::c_int
    {
        EnableNMI(game);
        WaitUntilEndOfVblankAndClearHdma(game);
        DisableIrqInterrupts(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1982) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xab) as *mut uint16) = 0 as uint16;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 16 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 9 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x943) as *mut uint16) = 0 as uint16;
        QueueSfx1_Max15(2 as uint16, game);
        QueueSfx2_Max15(0x71 as uint16, game);
        QueueSfx3_Max15(1 as uint16, game);
        if CheckEventHappened(0xe as uint16, game) != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) = 0 as uint16;
            let mut i: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
            while i >= 0 as ::core::ffi::c_int {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16)
                    .offset(((i >> 1 as ::core::ffi::c_int) + 15 as ::core::ffi::c_int) as isize) =
                    0 as uint16;
                i -= 2 as ::core::ffi::c_int;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
                kGameState_25_SamusNoHealth as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
                kGameState_37_CeresGoesBoomWithSamus as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
                (((0x8bc11bi32) as u16) & (0xffffu16));
        }
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_38_SamusEscapesFromZebes(game: &mut Game) -> CoroutineRet {
    let mut _coret_: uint8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 0x80 as ::core::ffi::c_int
    {
        EnableNMI(game);
        WaitUntilEndOfVblankAndClearHdma(game);
        DisableIrqInterrupts(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1982) as *mut uint16) = 0 as uint16;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 16 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 9 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
            kGameState_39_EndingAndCredits as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (((0x8bd480i32) as u16) & (0xffffu16));
        *((&raw mut game.g_ram as *mut uint8).offset(0x943) as *mut uint16) = 0 as uint16;
        QueueMusic_Delayed8(0 as uint16, game);
        QueueSfx1_Max15(2 as uint16, game);
        QueueSfx2_Max15(0x71 as uint16, game);
        QueueSfx3_Max15(1 as uint16, game);
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_41_TransitionToDemo(game: &mut Game) -> CoroutineRet {
    let mut _coret_: uint8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    HdmaObjectHandler(game);
    let ref mut fresh126 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
    *fresh126 = (*fresh126).wrapping_add(1);
    *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 15 as uint8;
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_42_PlayingDemo_Async(game: &mut Game) -> CoroutineRet {
    let mut current_block: u64;
    let mut _state_variable_: *mut uint8 = (&raw mut game.g_ram as *mut uint8).offset(0x614);
    match *_state_variable_ as ::core::ffi::c_int {
        0 | 2 => {
            let mut _coret_: uint8 = GameState_8_MainGameplay(game);
            if _coret_ != 0 {
                *_state_variable_ = 2 as uint8;
                return _coret_;
            }
            if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) != 0 {
                *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) = 1 as uint16;
                current_block = 4040822987312515878;
            } else {
                let ref mut fresh127 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1f53) as *mut uint16);
                *fresh127 = (*fresh127).wrapping_sub(1);
                if *((&raw mut game.g_ram as *mut uint8).offset(0x1f53) as *mut uint16) == 0
                    || *((&raw mut game.g_ram as *mut uint8).offset(0x1f53) as *mut uint16)
                        as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) =
                        0 as uint16;
                    *((&raw mut game.g_ram as *mut uint8).offset(0x77e) as *mut uint16) =
                        90 as uint16;
                    current_block = 12296459429166835708;
                } else {
                    current_block = 13472856163611868459;
                }
            }
        }
        1 => {
            current_block = 12296459429166835708;
        }
        _ => {
            current_block = 13472856163611868459;
        }
    }
    loop {
        match current_block {
            13472856163611868459 => {
                *_state_variable_ = 0 as uint8;
                break;
            }
            12296459429166835708 => {
                let mut _coret__0: uint8 = WaitForNMI_Async(game);
                if _coret__0 != 0 {
                    *_state_variable_ = 1 as uint8;
                    return _coret__0;
                }
                if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) != 0 {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) =
                        1 as uint16;
                    current_block = 4040822987312515878;
                } else {
                    let ref mut fresh128 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0x77e) as *mut uint16);
                    *fresh128 = (*fresh128).wrapping_sub(1);
                    if *fresh128 == 0 {
                        current_block = 4040822987312515878;
                    } else {
                        current_block = 12296459429166835708;
                    }
                }
            }
            _ => {
                let ref mut fresh129 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
                *fresh129 = (*fresh129).wrapping_add(1);
                *((&raw mut game.g_ram as *mut uint8).offset(0x5f5) as *mut uint16) = 0 as uint16;
                *(&raw mut game.g_ram as *mut uint8).offset(0x51) = 0x80 as uint8;
                *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 1 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 1 as uint16;
                current_block = 13472856163611868459;
            }
        }
    }
    return 0 as CoroutineRet;
}
pub unsafe fn GameState_43_TransitionFromDemo(game: &mut Game) -> CoroutineRet {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) as ::core::ffi::c_int
        != 1 as ::core::ffi::c_int
    {
        CheckForNextDemo(game);
    }
    EnableNMI(game);
    let ref mut fresh130 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
    *fresh130 = (*fresh130).wrapping_add(1);
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
    WaitUntilEndOfVblankAndClearHdma(game);
    DisableIrqInterrupts(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1982) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xab) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7) as *mut uint16) = 0 as uint16;
    DisablePaletteFx(game);
    ClearPaletteFXObjects(game);
    let mut i: ::core::ffi::c_int = 656 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16 as *mut uint8)
            .offset(i as isize) as *mut uint16) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let mut j: ::core::ffi::c_int = 538 as ::core::ffi::c_int;
    while j >= 0 as ::core::ffi::c_int {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x77c) as *mut uint16 as *mut uint8)
            .offset(j as isize) as *mut uint16) = 0 as uint16;
        j -= 2 as ::core::ffi::c_int;
    }
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 16 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_44_TransitionFromDemo(game: &mut Game) -> CoroutineRet {
    *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
        kGameState_1_OpeningCinematic as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
            kGameState_40_TransitionToDemo as uint16;
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) != 0 {
        LoadTitleSequenceGraphics(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(4) =
            2 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (((0x8b9a47i32) as u16) & (0xffffu16));
    } else {
        QueueMusic_Delayed8(0 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x5f5) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (((0x8b9b68i32) as u16) & (0xffffu16));
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn CheckForNextDemo(game: &mut Game) {
    if *(RomPtr_82(
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82876c)) as *const uint8 as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x1f55) as *mut uint16) as isize)
            as ::core::ffi::c_int
            + 18 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0x1f57) as *mut uint16)
                    as ::core::ffi::c_int) as uint16_t,
        game,
    ) as *mut uint16) as ::core::ffi::c_int
        == 0xffff as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) = 0 as uint16;
        let mut v0: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x1f55)
            as *mut uint16))
            .wrapping_add((1u16)));
        if v0 as ::core::ffi::c_int
            >= *((&raw mut game.g_ram as *mut uint8).offset(0x1f59) as *mut uint16)
                as ::core::ffi::c_int
        {
            v0 = 0 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f55) as *mut uint16) = v0;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f57) as *mut uint16) = 0 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) = 0x8000 as uint16;
    };
}
pub unsafe fn LoadDemoRoomData(game: &mut Game) {
    let mut drd: *mut DemoRoomData = ::core::ptr::null_mut::<DemoRoomData>();
    *((&raw mut game.g_ram as *mut uint8).offset(0x78d) as *mut uint16) = 0 as uint16;
    drd = get_DemoRoomData(
        (18 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x1f57) as *mut uint16)
                as ::core::ffi::c_int
            + *(game.g_rom.offset(crate::rtl::lorom_offset(0x82876c)) as *const uint8
                as *mut uint16)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1f55) as *mut uint16) as isize,
                ) as ::core::ffi::c_int) as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x79b) as *mut uint16) = (*drd).room_ptr_;
    *((&raw mut game.g_ram as *mut uint8).offset(0x78d) as *mut uint16) = (*drd).door_ptr;
    *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) = (*drd).screen_x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x91d) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) = (*drd).screen_y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x91f) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16))
            .wrapping_add(((*drd).samus_x_offs)));
    *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) = ((*drd).samus_y_offs
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) as ::core::ffi::c_int
        + 128 as ::core::ffi::c_int)
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f53) as *mut uint16) = (*drd).demo_length;
    *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16 as *mut uint8).offset(0) =
        (*get_RoomDefHeader(
            *((&raw mut game.g_ram as *mut uint8).offset(0x79b) as *mut uint16),
            game,
        ))
        .area_index_;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = 0 as uint16;
    let ref mut fresh144 = *((&raw mut game.g_ram as *mut uint8).offset(0x1f57) as *mut uint16);
    *fresh144 = (*fresh144).wrapping_add(1);
    let mut v1: uint16 = 0 as uint16;
    loop {
        let mut v2: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd830)
            .offset(v1 as isize) as *mut uint8 as *mut uint16) =
            -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd870)
            .offset(v1 as isize) as *mut uint8 as *mut uint16) =
            -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd8f0) as *mut uint16).offset(v2 as isize) =
            -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd908)
            .offset(v1 as isize) as *mut uint8 as *mut uint16) =
            -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd8f8)
            .offset(v1 as isize) as *mut uint8 as *mut uint16) =
            -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd8f8)
            .offset((v1 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd8b0)
            .offset(v1 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd820)
            .offset(v1 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd828)
            .offset(v1 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        v1 = ((v1).wrapping_add((2u16)));
        if !(((((v1) as i16).wrapping_sub((8i16))) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    loop {
        let mut v3: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd830)
            .offset(v1 as isize) as *mut uint8 as *mut uint16) =
            -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd870)
            .offset(v1 as isize) as *mut uint8 as *mut uint16) =
            -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd8b0)
            .offset(v1 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        v1 = ((v1).wrapping_add((2u16)));
        if !(((((v1) as i16).wrapping_sub((64i16))) as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    let mut v4: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd52) as *mut uint16)
            .offset((v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        v4 = ((v4).wrapping_add((2u16)));
        if !(((((v4) as i16).wrapping_sub((1536i16))) as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d4) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c0) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16) =
        kLoadingGameState_0_Intro as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x5f7) as *mut uint16) = 0 as uint16;
}
pub unsafe fn DemoRoom_ChargeBeamRoomScroll21(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(33) = 0 as uint8;
}
pub unsafe fn DemoRoom_SetBG2TilemapBase(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x59) as *mut uint16) = 74 as uint16;
}
pub unsafe fn DemoRoom_SetKraidFunctionTimer(game: &mut Game) {
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .ai_preinstr = 60 as uint16;
}
pub unsafe fn DemoRoom_SetBrinstarBossBits(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xd828).offset(1) = 1 as uint8;
}
static kGameStateFuncs: [Option<Func_V_Coroutine>; 45] = unsafe {
    [
        Some(GameState_0_Reset_Async as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_1_OpeningCinematic as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_2_GameOptionsMenu as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_3_Null as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_4_FileSelectMenus as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_5_FileSelectMap as unsafe fn(&mut Game) -> CoroutineRet),
        Some(InitAndLoadGameData_Async as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_7_MainGameplayFadeIn as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_8_MainGameplay as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_9_HitDoorBlock as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_10_LoadingNextRoom_Async as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_11_LoadingNextRoom_Async as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_12_Pausing_Darkening_Async as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_13_Pausing_Async as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_14_Paused_Async as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_15_Paused_Async as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_16_Unpausing_Async as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_17_Unpausing_Async as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_18_Unpausing as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_19_SamusNoHealth as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_20_SamusNoHealth_BlackOut as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_21_SamusNoHealth as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_22_SamusNoHealth_Dying as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_23_SamusNoHealth_Flashing as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_24_SamusNoHealth_Explosion as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_25_SamusNoHealth_BlackOut as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_26_GameOverMenu as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_27_ReserveTanksAuto as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_28_Unused as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_29_DebugGameOverMenu as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_37_CeresGoesBoomWithSamus as unsafe fn(&mut Game) -> CoroutineRet),
        Some(InitAndLoadGameData_Async as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_32_MadeItToCeresElevator as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_33_BlackoutFromCeres as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_37_CeresGoesBoomWithSamus as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_35_TimeUp as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_36_WhitingOutFromTimeUp as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_37_CeresGoesBoomWithSamus as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_38_SamusEscapesFromZebes as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_39_EndingAndCredits as unsafe fn(&mut Game) -> CoroutineRet),
        Some(InitAndLoadGameData_Async as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_41_TransitionToDemo as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_42_PlayingDemo_Async as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_43_TransitionFromDemo as unsafe fn(&mut Game) -> CoroutineRet),
        Some(GameState_44_TransitionFromDemo as unsafe fn(&mut Game) -> CoroutineRet),
    ]
};
pub unsafe fn RunOneFrameOfGameInner(game: &mut Game) -> CoroutineRet {
    let mut current_block: u64;
    let mut st: ::core::ffi::c_int =
        *(&raw mut game.g_ram as *mut uint8).offset(0x611) as ::core::ffi::c_int;
    if st >= 1 as ::core::ffi::c_int && st <= 3 as ::core::ffi::c_int {
        let mut _coret_: uint8 = Vector_RESET_Async(game);
        if _coret_ != 0 {
            return _coret_;
        }
        current_block = 8515828400728868193;
    } else if st >= 10 as ::core::ffi::c_int {
        current_block = 2185597630302572016;
    } else {
        if st != 0 as ::core::ffi::c_int {
            Die(b"Incorrect coroutine_state_0\0" as *const u8 as *const ::core::ffi::c_char);
        }
        current_block = 8515828400728868193;
    }
    match current_block {
        8515828400728868193 => {
            let ref mut fresh149 = *(&raw mut game.g_ram as *mut uint8).offset(0x615);
            *fresh149 = 0 as uint8;
            let ref mut fresh150 = *(&raw mut game.g_ram as *mut uint8).offset(0x614);
            *fresh150 = *fresh149;
            let ref mut fresh151 = *(&raw mut game.g_ram as *mut uint8).offset(0x613);
            *fresh151 = *fresh150;
            *(&raw mut game.g_ram as *mut uint8).offset(0x612) = *fresh151;
            ReadJoypadInputs(game);
            HdmaObjectHandler(game);
            NextRandom(game);
            ClearOamExt(game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x71d) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x71f) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x721) as *mut uint16) = 0 as uint16;
            st = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16)
                as ::core::ffi::c_int
                + 10 as ::core::ffi::c_int;
            *(&raw mut game.g_ram as *mut uint8).offset(0x611) = st as uint8;
        }
        _ => {}
    }
    let mut _coret__0: uint8 = kGameStateFuncs[(st - 10 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(game);
    if _coret__0 != 0 {
        return _coret__0;
    }
    HandleSoundEffects(game);
    ClearUnusedOam(game);
    ShowSpareCpu(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x612) as ::core::ffi::c_int
        | *(&raw mut game.g_ram as *mut uint8).offset(0x613) as ::core::ffi::c_int
        | *(&raw mut game.g_ram as *mut uint8).offset(0x614) as ::core::ffi::c_int
        | *(&raw mut game.g_ram as *mut uint8).offset(0x615) as ::core::ffi::c_int
        != 0
    {
        printf(
            b"Coroutine state: %d, %d, %d, %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            *(&raw mut game.g_ram as *mut uint8).offset(0x612) as ::core::ffi::c_int,
            *(&raw mut game.g_ram as *mut uint8).offset(0x613) as ::core::ffi::c_int,
            *(&raw mut game.g_ram as *mut uint8).offset(0x614) as ::core::ffi::c_int,
            *(&raw mut game.g_ram as *mut uint8).offset(0x615) as ::core::ffi::c_int,
        );
        Warning(b"Coroutine State is broken!\0" as *const u8 as *const ::core::ffi::c_char);
    }
    *(&raw mut game.g_ram as *mut uint8).offset(0x611) = 0 as uint8;
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn RunOneFrameOfGame(game: &mut Game) {
    let mut ret: CoroutineRet = RunOneFrameOfGameInner(game);
    if ret as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        *(&raw mut game.g_ram as *mut uint8).offset(0x5b4) = 1 as uint8;
    }
    Vector_NMI(game);
}
pub unsafe fn GameState_29_DebugGameOverMenu(game: &mut Game) -> CoroutineRet {
    DebugGameOverMenu();
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_26_GameOverMenu(game: &mut Game) -> CoroutineRet {
    GameOverMenu(game);
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_4_FileSelectMenus(game: &mut Game) -> CoroutineRet {
    FileSelectMenu(game);
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_5_FileSelectMap(game: &mut Game) -> CoroutineRet {
    FileSelectMap(game);
    return kCoroutineNone as CoroutineRet;
}
static kSfxHandlers: [Option<Func_Y_V>; 5] = unsafe {
    [
        Some(SfxHandlers_0_SendToApu as unsafe fn(uint16, &mut Game) -> ()),
        Some(SfxHandlers_1_WaitForAck as unsafe fn(uint16, &mut Game) -> ()),
        Some(SfxHandlers_2_ClearRequest as unsafe fn(uint16, &mut Game) -> ()),
        Some(SfxHandlers_3_WaitForAck as unsafe fn(uint16, &mut Game) -> ()),
        Some(SfxHandlers_4_Reset as unsafe fn(uint16, &mut Game) -> ()),
    ]
};
pub unsafe fn HandleSoundEffects(game: &mut Game) {
    if (((*((&raw mut game.g_ram as *mut uint8).offset(0x686) as *mut uint16)) as i8)
        .wrapping_sub((1i8))) as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x686) as *mut uint16 as *mut uint8)
            .offset(0) = (((*((&raw mut game.g_ram as *mut uint8).offset(0x686) as *mut uint16))
            as u8)
            .wrapping_sub((1u8)));
        RtlApuWrite(APUI01, 0 as uint8, game);
        *(&raw mut game.g_ram as *mut uint8).offset(0x64d).offset(0) = 0 as uint8;
        RtlApuWrite(APUI02, 0 as uint8, game);
        *(&raw mut game.g_ram as *mut uint8).offset(0x64d).offset(1) = 0 as uint8;
        RtlApuWrite(APUI03, 0 as uint8, game);
        *(&raw mut game.g_ram as *mut uint8).offset(0x64d).offset(2) = 0 as uint8;
    } else {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 3 as ::core::ffi::c_int {
            kSfxHandlers[*(&raw mut game.g_ram as *mut uint8)
                .offset(0x649)
                .offset(i as isize) as usize]
                .expect("non-null function pointer")(i as uint16, game);
            i += 1;
        }
    };
}
pub unsafe fn SfxHandlers_0_SendToApu(mut j: uint16, game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0x643)
        .offset(j as isize) as ::core::ffi::c_int
        != *(&raw mut game.g_ram as *mut uint8)
            .offset(0x646)
            .offset(j as isize) as ::core::ffi::c_int
    {
        let mut v2: uint8 = (*(&raw mut game.g_ram as *mut uint8)
            .offset(0x643)
            .offset(j as isize) as ::core::ffi::c_int
            + j as ::core::ffi::c_int * 16 as ::core::ffi::c_int)
            as uint8;
        let mut v3: uint8 = *(&raw mut game.g_ram as *mut uint8)
            .offset(0x656)
            .offset(v2 as isize);
        RtlApuWrite(
            (j as ::core::ffi::c_int + APUI01 as ::core::ffi::c_int) as SnesRegs,
            v3,
            game,
        );
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0x64d)
            .offset(j as isize) = v3;
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0x643)
            .offset(j as isize) = (v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int) as uint8;
        let ref mut fresh134 = *(&raw mut game.g_ram as *mut uint8)
            .offset(0x649)
            .offset(j as isize);
        *fresh134 = (*fresh134).wrapping_add(1);
    }
}
pub unsafe fn SfxHandlers_1_WaitForAck(mut j: uint16, game: &mut Game) {
    let mut v1: uint8 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x64d)
        .offset(j as isize);
    let ref mut fresh138 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x649)
        .offset(j as isize);
    *fresh138 = (*fresh138).wrapping_add(1);
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0x650)
        .offset(j as isize) = 6 as uint8;
}
pub unsafe fn SfxHandlers_2_ClearRequest(mut j: uint16, game: &mut Game) {
    let ref mut fresh135 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x650)
        .offset(j as isize);
    let fresh136 = *fresh135;
    *fresh135 = (*fresh135).wrapping_sub(1);
    if fresh136 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0x64d)
            .offset(j as isize) = 0 as uint8;
        let ref mut fresh137 = *(&raw mut game.g_ram as *mut uint8)
            .offset(0x649)
            .offset(j as isize);
        *fresh137 = (*fresh137).wrapping_add(1);
    }
}
pub unsafe fn SfxHandlers_3_WaitForAck(mut j: uint16, game: &mut Game) {
    let mut v1: uint8 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x64d)
        .offset(j as isize);
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0x649)
        .offset(j as isize) = 0 as uint8;
    SfxHandlers_0_SendToApu(j, game);
}
pub unsafe fn SfxHandlers_4_Reset(mut j: uint16, game: &mut Game) {
    let ref mut fresh132 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x650)
        .offset(j as isize);
    let fresh133 = *fresh132;
    *fresh132 = (*fresh132).wrapping_sub(1);
    if fresh133 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0x649)
            .offset(j as isize) = 0 as uint8;
    }
}
pub unsafe fn ResetSoundQueues(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x643) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x643).offset(2) as *mut uint8 as *mut uint16) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x646).offset(1) as *mut uint8 as *mut uint16) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x649) as *mut uint16) = 0 as uint16;
    *(&raw mut game.g_ram as *mut uint8).offset(0x649).offset(2) = 0 as uint8;
}
pub unsafe fn ShowSpareCpu(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xdfe) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16);
}
pub unsafe fn GameState_0_Reset_Async(game: &mut Game) -> CoroutineRet {
    *((&raw mut game.g_ram as *mut uint8).offset(0xdf8) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdfa) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdfc) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
        (((0x8b9b68i32) as u16) & (0xffffu16));
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f55) as *mut uint16) = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f59) as *mut uint16) as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f55) as *mut uint16) = 3 as uint16;
    }
    let ref mut fresh79 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
    *fresh79 = (*fresh79).wrapping_add(1);
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_1_OpeningCinematic(game: &mut Game) -> CoroutineRet {
    return GameState_1_OpeningCinematic_(game);
}
pub unsafe fn GameState_3_Null(game: &mut Game) -> CoroutineRet {
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_37_CeresGoesBoomWithSamus(game: &mut Game) -> CoroutineRet {
    return GameState_37_CeresGoesBoomWithSamus_(game);
}
pub unsafe fn GameState_39_EndingAndCredits(game: &mut Game) -> CoroutineRet {
    return GameState_39_EndingAndCredits_(game);
}
pub unsafe fn GameState_7_MainGameplayFadeIn(game: &mut Game) -> CoroutineRet {
    let mut _coret_: uint8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    HandleFadeIn(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        let ref mut fresh131 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
        *fresh131 = (*fresh131).wrapping_add(1);
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_28_Unused(game: &mut Game) -> CoroutineRet {
    GameState_28_Unused_(game);
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_8_MainGameplay(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut uint8 = (&raw mut game.g_ram as *mut uint8).offset(0x612);
    let mut current_block_38: u64;
    match *_state_variable_ as ::core::ffi::c_int {
        0 => {
            DetermineWhichEnemiesToProcess(game);
            PaletteFxHandler(game);
            HandleControllerInputForGamePhysics(game);
            if *((&raw mut game.g_ram as *mut uint8).offset(0xe12) as *mut uint16) == 0 {
                SamusProjectileInteractionHandler(game);
            }
            EnemyMain(game);
            if *((&raw mut game.g_ram as *mut uint8).offset(0xdc8) as *mut uint16) != 0 {
                current_block_38 = 6718109325120641364;
            } else {
                current_block_38 = 13586036798005543211;
            }
        }
        1 => {
            current_block_38 = 6718109325120641364;
        }
        2 => {
            current_block_38 = 8001970166466707590;
        }
        _ => {
            current_block_38 = 652864300344834934;
        }
    }
    match current_block_38 {
        6718109325120641364 => {
            let mut _coret_: uint8 = DisplayMessageBox_Async(
                *((&raw mut game.g_ram as *mut uint8).offset(0xdc8) as *mut uint16),
                game,
            );
            if _coret_ != 0 {
                *_state_variable_ = 1 as uint8;
                return _coret_;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc8) as *mut uint16) = 0 as uint16;
            current_block_38 = 13586036798005543211;
        }
        _ => {}
    }
    match current_block_38 {
        13586036798005543211 => {
            HandleSamusMovementAndPause(game);
            EprojRunAll(game);
            current_block_38 = 8001970166466707590;
        }
        _ => {}
    }
    match current_block_38 {
        8001970166466707590 => {
            let mut _coret__0: uint8 = PlmHandler_Async(game);
            if _coret__0 != 0 {
                *_state_variable_ = 2 as uint8;
                return _coret__0;
            }
            AnimtilesHandler(game);
            if *((&raw mut game.g_ram as *mut uint8).offset(0xe12) as *mut uint16) == 0 {
                EprojSamusCollDetect(game);
                EprojProjCollDet(game);
                ProcessEnemyPowerBombInteraction(game);
            }
            MainScrollingRoutine(game);
            let mut debug_scrolling_enabled: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if debug_scrolling_enabled != 0 {
                DebugScrollPosSaveLoad(game);
            }
            DrawSamusEnemiesAndProjectiles(game);
            QueueEnemyBG2TilemapTransfers(game);
            HandleHudTilemap(game);
            CalculateLayer2PosAndScrollsWhenScrolling(game);
            RunRoomMainCode(game);
            HandleSamusOutOfHealthAndGameTile(game);
            HandleRoomShaking(game);
            DecrementSamusTimers(game);
        }
        _ => {}
    }
    *_state_variable_ = 0 as uint8;
    return 0 as CoroutineRet;
}
pub unsafe fn DeleteAllOptionsMenuObjects_(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
            .offset((v1 + 5 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset((v1 + 17 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn CallOptionsEntryFunc(mut ea: uint32, mut j: uint16, game: &mut Game) {
    match ea {
        8581782 => {
            sub_82F296(j, game);
            return;
        }
        8581963 => {
            sub_82F34B(j, game);
            return;
        }
        8581971 => {
            sub_82F353(j, game);
            return;
        }
        8581979 => {
            sub_82F35B(j, game);
            return;
        }
        8581987 => {
            sub_82F363(j, game);
            return;
        }
        8582169 => {
            sub_82F419(j, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CreateOptionsMenuObject_(mut a: uint16, mut j: uint16, game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16) = a;
    let mut v3: uint16 = 14 as uint16;
    while *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16)
        .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        != 0
    {
        v3 = ((v3).wrapping_sub((2u16)));
        if v3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return 1 as uint8;
        }
    }
    let mut v5: *const uint8 = RomPtr_82(j, game);
    let mut v6: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b0d) as *mut uint16).offset(v6 as isize) =
        *(v5.offset(2) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v6 as isize) =
        *(v5.offset(4) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v6 as isize) =
        1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v6 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b2d) as *mut uint16).offset(v6 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1add) as *mut uint16).offset(v6 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aed) as *mut uint16).offset(v6 as isize) =
        0 as uint16;
    CallOptionsEntryFunc((((*(v5 as *mut uint16)) as u32) | (0x820000u32)), v3, game);
    return 0 as uint8;
}
pub fn OptionsPreInstr_nullsub_57(mut k: uint16) {}
pub unsafe fn OptionsMenuFunc1(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a8f) as *mut uint16) = i as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize)
            != 0
        {
            OptionsMenuFunc2(i as uint16, game);
            i = *((&raw mut game.g_ram as *mut uint8).offset(0x1a8f) as *mut uint16)
                as ::core::ffi::c_int;
        }
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn CallOptionsPreInstr(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        8555536 => {
            OptionsPreInstr_nullsub_57(k);
            return;
        }
        8581801 => {
            OptionsPreInstr_F2A9(k, game);
            return;
        }
        8582006 => {
            OptionsPreInstr_F376(k, game);
            return;
        }
        8582048 => {
            OptionsPreInstr_F3A0(k, game);
            return;
        }
        8582114 => {
            OptionsPreInstr_F3E2(k, game);
            return;
        }
        8582188 => {
            OptionsPreInstr_F42C(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CallOptionsInstr(
    mut ea: uint32,
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    match ea {
        8555610 => return OptionsInstr_Destroy(k, j, game),
        8555620 => return OptionsInstr_8C64(k, j, game),
        8555630 => return OptionsInstr_SetPreInstr(k, j, game),
        8555641 => return OptionsInstr_8C79(k, j, game),
        8555650 => return OptionsInstr_Goto(k, j, game),
        8555657 => return OptionsInstr_8C89(k, j, game),
        8555667 => return OptionsInstr_8C93(k, j, game),
        _ => return Unreachable(game) as uint16,
    };
}
pub unsafe fn OptionsMenuFunc2(mut k: uint16, game: &mut Game) {
    CallOptionsPreInstr(
        (((*((&raw mut game.g_ram as *mut uint8).offset(0x1b0d) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize))
            as u32)
            | (0x820000u32)),
        k,
        game,
    );
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a8f) as *mut uint16);
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1a8f)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let ref mut fresh119 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v2 as isize);
    let fresh120 = *fresh119;
    *fresh119 = (*fresh119).wrapping_sub(1);
    if fresh120 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        let mut v4: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16)
            .offset(v2 as isize);
        let mut v5: *mut uint16 = ::core::ptr::null_mut::<uint16>();
        loop {
            v5 = RomPtr_82(v4, game) as *mut uint16;
            if *v5.offset(0) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                break;
            }
            v4 = CallOptionsInstr(
                (((*v5.offset(0)) as u32) | (0x820000u32)),
                v1,
                ((v4).wrapping_add((2u16))),
                game,
            );
            if v4 == 0 {
                return;
            }
        }
        let mut v7: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b1d) as *mut uint16).offset(v7 as isize) =
            *v5.offset(0);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v7 as isize) =
            *v5.offset(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v7 as isize) =
            ((v4).wrapping_add((4u16)));
    }
}
pub unsafe fn OptionsInstr_Destroy(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    return 0 as uint16;
}
pub unsafe fn OptionsInstr_8C64(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1afd) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        ((j).wrapping_sub((2u16)));
    return 0 as uint16;
}
pub unsafe fn OptionsInstr_SetPreInstr(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b0d) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(RomPtr_82(j, game) as *mut uint16);
    return ((j).wrapping_add((2u16)));
}
pub unsafe fn OptionsInstr_8C79(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b0d) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (((0x828c81i32) as u16) & (0xffffu16));
    return j;
}
pub unsafe fn OptionsInstr_Goto(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    return *(RomPtr_82(j, game) as *mut uint16);
}
pub unsafe fn OptionsInstr_8C89(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh0 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b2d) as *mut uint16).offset(v2 as isize);
    let fresh1 = *fresh0;
    *fresh0 = (*fresh0).wrapping_sub(1);
    if fresh1 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        return ((j).wrapping_add((2u16)));
    } else {
        return OptionsInstr_Goto(k, j, game);
    };
}
pub unsafe fn OptionsInstr_8C93(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b2d) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(RomPtr_82(j, game) as *mut uint16);
    return ((j).wrapping_add((2u16)));
}
pub unsafe fn DrawOptionsMenuSpritemaps(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d) as *mut uint16).offset(v1 as isize)
            != 0
        {
            let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a9d)
                as *mut uint16)
                .offset(v1 as isize);
            DrawSpritemap(
                0x82 as uint8,
                v2,
                *((&raw mut game.g_ram as *mut uint8).offset(0x1aad) as *mut uint16)
                    .offset(v1 as isize),
                *((&raw mut game.g_ram as *mut uint8).offset(0x1abd) as *mut uint16)
                    .offset(v1 as isize),
                *((&raw mut game.g_ram as *mut uint8).offset(0x1acd) as *mut uint16)
                    .offset(v1 as isize),
                game,
            );
        }
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn GameState_12_Pausing_Darkening_Async(game: &mut Game) -> CoroutineRet {
    let mut _coret_: uint8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        EnableNMI(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        let ref mut fresh80 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
        *fresh80 = (*fresh80).wrapping_add(1);
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn CallPauseHook(mut ea: uint32, game: &mut Game) {
    match ea {
        8946657 => return,
        9423094 => {
            PauseHook_DraygonRoom(game);
            return;
        }
        10994469 => {
            PauseHook_Kraid(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CallUnpauseHook_Async(mut ea: uint32, game: &mut Game) -> CoroutineRet {
    match ea {
        8946657 => return kCoroutineNone as CoroutineRet,
        9423100 => return UnpauseHook_DraygonRoom(game),
        10994171 => return UnpauseHook_Kraid_IsDead(game),
        10994254 => return UnpauseHook_Kraid_IsAlive(game),
        10994336 => return Kraid_UnpauseHook_IsSinking(game),
        11110243 => return MotherBrainsBody_UnpauseHook(game),
        _ => {
            Unreachable(game);
            return kCoroutineNone as CoroutineRet;
        }
    };
}
pub unsafe fn GameState_13_Pausing_Async(game: &mut Game) -> CoroutineRet {
    DisableHdmaObjects(game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x85) = 0 as uint8;
    WriteReg(HDMAEN as uint16, 0 as uint8, game);
    DisableAnimtiles(game);
    BackupBG2TilemapForPauseMenu(game);
    CallPauseHook(
        Load24((&raw mut game.g_ram as *mut uint8).offset(0x601) as *mut LongPtr),
        game,
    );
    CancelSoundEffects(game);
    BackupSomeGfxStateForPause(game);
    LoadPauseMenuTilesAndClearBG2(game);
    LoadPauseScreenBaseTilemaps(game);
    LoadPauseMenuMapTilemapAndAreaLabel(game);
    BackupGameplayPalettesAndLoadForPause(game);
    ContinueInitPauseMenu(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x74d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x73b) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x5fd) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x5ff) as *mut uint16) = 0 as uint16;
    QueueClearingOfFxTilemap(game);
    let ref mut fresh85 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
    *fresh85 = (*fresh85).wrapping_add(1);
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn BackupBG2TilemapForPauseMenu(game: &mut Game) {
    WriteRegWord(
        VMADDL as uint16,
        ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as uint16,
        game,
    );
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    WriteReg(DMAP1 as uint16, 0x81 as uint8, game);
    WriteReg(BBAD1 as uint16, 0x39 as uint8, game);
    WriteRegWord(A1T1L as uint16, 0xdf5c as uint16, game);
    WriteReg(A1B1 as uint16, 0x7e as uint8, game);
    WriteRegWord(DAS1L as uint16, 0x1000 as uint16, game);
    WriteReg(DAS10 as uint16, 0 as uint8, game);
    WriteRegWord(A2A1L as uint16, 0 as uint16, game);
    WriteReg(NTRL1 as uint16, 0 as uint8, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
}
pub unsafe fn RestoreBG2TilemapFromPauseScreen(game: &mut Game) {
    WriteRegWord(
        VMADDL as uint16,
        ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16,
        game,
    );
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_828DAE: StartDmaCopy = StartDmaCopy::vram_words(1, 0x7edf5c, 0x1000);
    SetupDmaTransfer(&raw const unk_828DAE as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
}
pub unsafe fn BackupSomeGfxStateForPause(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x765) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x58);
    *(&raw mut game.g_ram as *mut uint8).offset(0x766) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x59);
    *(&raw mut game.g_ram as *mut uint8).offset(0x767) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x5a);
    *(&raw mut game.g_ram as *mut uint8).offset(0x768) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x5d);
    *(&raw mut game.g_ram as *mut uint8).offset(0x769) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x5e);
    *(&raw mut game.g_ram as *mut uint8).offset(0x76a) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x52);
    *(&raw mut game.g_ram as *mut uint8).offset(0x76b) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x76c) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x76d) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb9) as *mut uint16) as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x76e) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x76f) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x770) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xbb) as *mut uint16) as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x771) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x55);
    *(&raw mut game.g_ram as *mut uint8).offset(0x772) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x91b);
    *(&raw mut game.g_ram as *mut uint8).offset(0x773) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x91c);
    *(&raw mut game.g_ram as *mut uint8).offset(0x774) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x57);
    *(&raw mut game.g_ram as *mut uint8).offset(0x775) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x71);
}
pub unsafe fn RestoreSomeGfxStateForPause(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x91b) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x772);
    *(&raw mut game.g_ram as *mut uint8).offset(0x91c) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x773);
    *(&raw mut game.g_ram as *mut uint8).offset(0x55) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x771);
    *((&raw mut game.g_ram as *mut uint8).offset(0xbb) as *mut uint16 as *mut uint8).offset(0) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x770);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16 as *mut uint8).offset(0) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x76f);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16 as *mut uint8).offset(0) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x76e);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb9) as *mut uint16 as *mut uint8).offset(0) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x76d);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16 as *mut uint8).offset(0) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x76c);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16 as *mut uint8).offset(0) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x76b);
    *(&raw mut game.g_ram as *mut uint8).offset(0x52) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x76a);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x769);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x768);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5a) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x767);
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x766);
    *(&raw mut game.g_ram as *mut uint8).offset(0x58) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x765);
    *(&raw mut game.g_ram as *mut uint8).offset(0x57) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x774);
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x775);
}
pub unsafe fn LoadPauseMenuTilesAndClearBG2(game: &mut Game) {
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_828E8B: StartDmaCopy = StartDmaCopy::vram_words(1, 0xb68000, 0x4000);
    SetupDmaTransfer(&raw const unk_828E8B as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x20 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_828EAB: StartDmaCopy = StartDmaCopy::vram_words(1, 0xb6c000, 0x2000);
    SetupDmaTransfer(&raw const unk_828EAB as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x40 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_828ECB: StartDmaCopy = StartDmaCopy::vram_words(1, 0x9ab200, 0x2000);
    SetupDmaTransfer(&raw const unk_828ECB as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
}
pub unsafe fn LoadPauseScreenBaseTilemaps(game: &mut Game) {
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x38 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_828EF0: StartDmaCopy = StartDmaCopy::vram_words(1, 0xb6e000, 0x800);
    SetupDmaTransfer(&raw const unk_828EF0 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(WMADDL as uint16, 0 as uint8, game);
    WriteReg(WMADDM as uint16, 0x34 as uint8, game);
    WriteReg(WMADDH as uint16, 0x7e as uint8, game);
    static unk_828F10: StartDmaCopy = StartDmaCopy::wram_bytes(1, 0xb6e400, 0x400);
    SetupDmaTransfer(&raw const unk_828F10 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(WMADDL as uint16, 0 as uint8, game);
    WriteReg(WMADDM as uint16, 0x38 as uint8, game);
    WriteReg(WMADDH as uint16, 0x7e as uint8, game);
    static unk_828F30: StartDmaCopy = StartDmaCopy::wram_bytes(1, 0xb6e800, 0x800);
    SetupDmaTransfer(&raw const unk_828F30 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    let mut v0: uint16 = addr_kDummySamusWireframeTilemap as uint16;
    let mut v1: uint16 = 236 as uint16;
    let mut m: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
    loop {
        let mut n: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
        loop {
            let fresh92 = v1;
            v1 = v1.wrapping_add(1);
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap[fresh92 as usize] = *(RomPtr_82(v0, game) as *mut uint16);
            v0 = ((v0).wrapping_add((2u16)));
            n -= 1;
            if !(n != 0) {
                break;
            }
        }
        v1 = ((v1).wrapping_add((8u16)));
        m -= 1;
        if !(m != 0) {
            break;
        }
    }
    WriteSamusWireframeTilemap(game);
    LoadEqupmentScreenReserveHealthTilemap(game);
}
pub unsafe fn LoadEqupmentScreenReserveHealthTilemap(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9d4) as *mut uint16) != 0 {
        let mut r42: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x9d6)
            as *mut uint16) as ::core::ffi::c_int
            / 100 as ::core::ffi::c_int;
        let mut rest: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x9d6)
            as *mut uint16) as ::core::ffi::c_int
            % 100 as ::core::ffi::c_int;
        let mut div10: ::core::ffi::c_int = rest / 10 as ::core::ffi::c_int;
        let mut mod10: ::core::ffi::c_int = rest % 10 as ::core::ffi::c_int;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
            .cinematic_bg_tilemap[394 as ::core::ffi::c_int as usize] =
            (mod10 + 2052 as ::core::ffi::c_int) as uint16;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
            .cinematic_bg_tilemap[393 as ::core::ffi::c_int as usize] =
            (div10 + 2052 as ::core::ffi::c_int) as uint16;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
            .cinematic_bg_tilemap[392 as ::core::ffi::c_int as usize] =
            (r42 + 2052 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn BackupGameplayPalettesAndLoadForPause(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v2: int16 = 0;
    v0 = 512 as int16;
    let mut v1: uint16 = 0 as uint16;
    loop {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap
            [(v1 as ::core::ffi::c_int + 384 as ::core::ffi::c_int) as usize] =
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset(v1 as isize);
        v1 = v1.wrapping_add(1);
        v0 = ((v0).wrapping_sub((2i16)));
        if !(v0 != 0) {
            break;
        }
    }
    v2 = 512 as int16;
    let mut v3: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xb6f000)) as *const uint8 as *mut uint16)
                .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v3 = ((v3).wrapping_add((2u16)));
        v2 = ((v2).wrapping_sub((2i16)));
        if !(v2 != 0) {
            break;
        }
    }
}
pub unsafe fn ContinueInitPauseMenu(game: &mut Game) {
    SetupPpuForPauseMenu(game);
    ResetPauseMenuAnimations(game);
    LoadEquipmentScreenEquipmentTilemaps(game);
    set_pause_label_palettes(0, game);
    UpdatePauseMenuLRStartVramTilemap(game);
    DetermineMapScrollLimits(game);
    SetupMapScrollingForPauseMenu(0x80 as uint16, game);
}
pub unsafe fn SetupMapScrollingForFileSelectMap(game: &mut Game) {
    ResetPauseMenuAnimations(game);
    DetermineMapScrollLimits(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x5ac) as *mut uint16) as ::core::ffi::c_int
            + (((*((&raw mut game.g_ram as *mut uint8).offset(0x5ae) as *mut uint16)).wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x5ac) as *mut uint16)),
            )) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int)
            - 128 as ::core::ffi::c_int) as uint16;
    let mut r18: uint16 = (8 as ::core::ffi::c_int
        * (*((&raw mut game.g_ram as *mut uint8).offset(0x7a1) as *mut uint16)
            as ::core::ffi::c_int
            + ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int))
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    let mut v0: int16 = ((224i16).wrapping_sub(((r18) as i16)));
    if ((224i16).wrapping_sub(((r18) as i16))) as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        r18 = ((32u16).wrapping_sub((r18)));
        if r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            let ref mut fresh146 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16);
            *fresh146 = ((*fresh146).wrapping_sub((r18)));
        }
    } else {
        r18 = ((224u16).wrapping_sub((r18)));
        let ref mut fresh147 = *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16);
        *fresh147 = ((*fresh147).wrapping_sub(((v0) as u16)));
    }
    r18 = (*((&raw mut game.g_ram as *mut uint8).offset(0x5b0) as *mut uint16)
        as ::core::ffi::c_int
        + (((*((&raw mut game.g_ram as *mut uint8).offset(0x5b2) as *mut uint16))
            .wrapping_sub((*((&raw mut game.g_ram as *mut uint8).offset(0x5b0) as *mut uint16))))
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int)
        + 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) =
        -(112 as ::core::ffi::c_int - r18 as ::core::ffi::c_int & 0xfff8 as ::core::ffi::c_int)
            as uint16;
    r18 = (8 as ::core::ffi::c_int
        * (*((&raw mut game.g_ram as *mut uint8).offset(0x7a3) as *mut uint16)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int)
        + (112 as ::core::ffi::c_int - r18 as ::core::ffi::c_int & 0xfff8 as ::core::ffi::c_int))
        as uint16;
    let mut v1: int16 = ((64i16).wrapping_sub(((r18) as i16)));
    if ((64i16).wrapping_sub(((r18) as i16))) as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        r18 = ((64u16).wrapping_sub((r18)));
        let ref mut fresh148 = *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16);
        *fresh148 = ((*fresh148).wrapping_sub(((v1) as u16)));
        if *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) as ::core::ffi::c_int
            + 40 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) =
                -(40 as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn GameState_14_Paused_Async(game: &mut Game) -> CoroutineRet {
    DrawPauseMenuDuringFadeIn(game);
    HandleFadeIn(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        let ref mut fresh93 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
        *fresh93 = (*fresh93).wrapping_add(1);
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_15_Paused_Async(game: &mut Game) -> CoroutineRet {
    ReleaseButtonsFilter(3 as uint16, game);
    MainPauseRoutine(game);
    HandleHudTilemap(game);
    HandlePauseScreenPaletteAnimation(game);
    return kCoroutineNone as CoroutineRet;
}
static kPauseMenuFuncs: [Option<Func_V>; 8] = unsafe {
    [
        Some(PauseMenu_0_MapScreen as unsafe fn(&mut Game) -> ()),
        Some(PauseMenu_1_EquipmentScreen as unsafe fn(&mut Game) -> ()),
        Some(PauseMenu_2 as unsafe fn(&mut Game) -> ()),
        Some(PauseMenu_3_MapToEquipment_Load as unsafe fn(&mut Game) -> ()),
        Some(PauseMenu_4_MapToEquipment_FadeIn as unsafe fn(&mut Game) -> ()),
        Some(PauseMenu_5 as unsafe fn(&mut Game) -> ()),
        Some(PauseMenu_6_EquipmentToMap_Load as unsafe fn(&mut Game) -> ()),
        Some(PauseMenu_7_EquipmentToMap_FadeIn as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn MainPauseRoutine(game: &mut Game) {
    kPauseMenuFuncs[*((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn PauseMenu_0_MapScreen(game: &mut Game) {
    HandlePauseScreenLR(game);
    HandlePauseScreenStart(game);
    HandleMapScrollArrows(game);
    MapScrolling(game);
    MapScreenDrawSamusPositionIndicator(game);
    DrawMapIcons(game);
    DisplayMapElevatorDestinations(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x763) as *mut uint16) = 0 as uint16;
}
pub unsafe fn PauseMenu_1_EquipmentScreen(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = 0 as uint16;
    EquipmentScreenMain(game);
    HandlePauseScreenLR(game);
    HandlePauseScreenStart(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x763) as *mut uint16) = 1 as uint16;
}
pub unsafe fn PauseMenu_2(game: &mut Game) {
    DisplayMapElevatorDestinations(game);
    MapScreenDrawSamusPositionIndicator(game);
    DrawMapIcons(game);
    HandlePauseMenuLRPressHighlight(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x763) as *mut uint16) = 0 as uint16;
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 0x80 as ::core::ffi::c_int
    {
        EnableNMI(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        let ref mut fresh97 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh97 = (*fresh97).wrapping_add(1);
    }
}
pub unsafe fn PauseMenu_5(game: &mut Game) {
    EquipmentScreenDrawItemSelector(game);
    EquipmentScreenDisplayReserveTankAmount(game);
    HandlePauseMenuLRPressHighlight(game);
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 0x80 as ::core::ffi::c_int
    {
        EnableNMI(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        let ref mut fresh95 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
        *fresh95 = (*fresh95).wrapping_add(1);
    }
}
pub unsafe fn PauseMenu_3_MapToEquipment_Load(game: &mut Game) {
    DisplayMapElevatorDestinations(game);
    EquipmentScreenSetupReserveMode(game);
    EquipmentScreenTransferBG1Tilemap(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x763) as *mut uint16) = 1 as uint16;
    SetPauseScreenButtonLabelPalettes(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x73f) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x72b) as *mut uint16) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c10c)) as *const uint8 as *mut uint8
            as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 1 as uint16;
    let ref mut fresh96 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh96 = (*fresh96).wrapping_add(1);
}
pub unsafe fn PauseMenu_6_EquipmentToMap_Load(game: &mut Game) {
    DisplayMapElevatorDestinations(game);
    LoadPauseMenuMapTilemapAndAreaLabel(game);
    SetPauseScreenButtonLabelPalettes(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x73f) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x72b) as *mut uint16) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c10c)) as *const uint8 as *mut uint8
            as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x763) as *mut uint16) = 0 as uint16;
    let ref mut fresh94 = *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16);
    *fresh94 = (*fresh94).wrapping_add(1);
}
pub unsafe fn PauseMenu_7_EquipmentToMap_FadeIn(game: &mut Game) {
    MapScreenDrawSamusPositionIndicator(game);
    DrawMapIcons(game);
    DisplayMapElevatorDestinations(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x763) as *mut uint16) = 0 as uint16;
    HandleFadeIn(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x753) as *mut uint16);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x753) as *mut uint16) != 0 {
            v0 = 1 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) = v0;
    }
}
pub unsafe fn PauseMenu_4_MapToEquipment_FadeIn(game: &mut Game) {
    EquipmentScreenDrawItemSelector(game);
    EquipmentScreenDisplayReserveTankAmount(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x763) as *mut uint16) = 1 as uint16;
    HandleFadeIn(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x753) as *mut uint16);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x753) as *mut uint16) != 0 {
            v0 = 1 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) = v0;
    }
}
static kMapScrollingFuncs: [Option<Func_V>; 5] = unsafe {
    [
        Some(MapScrolling_0_None as unsafe fn(&mut Game) -> ()),
        Some(MapScrolling_1_Left as unsafe fn(&mut Game) -> ()),
        Some(MapScrolling_2_Right as unsafe fn(&mut Game) -> ()),
        Some(MapScrolling_3_Up as unsafe fn(&mut Game) -> ()),
        Some(MapScrolling_4_Down as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn MapScrolling(game: &mut Game) {
    kMapScrollingFuncs
        [*((&raw mut game.g_ram as *mut uint8).offset(0x5fd) as *mut uint16) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn MapScrolling_0_None(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x5fb) as *mut uint16) = 4 as uint16;
}
pub unsafe fn MapScrolling_GetSpeedIndex(game: &mut Game) -> uint16 {
    let mut result: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x5ff) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5fb) as *mut uint16) == 0 {
        return ((*((&raw mut game.g_ram as *mut uint8).offset(0x5ff) as *mut uint16))
            .wrapping_add((32u16)));
    }
    return result;
}
static kMapScrollingSpeedTable: [uint16; 32] = [
    0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub unsafe fn MapScrolling_1_Left(game: &mut Game) {
    let ref mut fresh105 = *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16);
    *fresh105 = ((*fresh105).wrapping_sub(
        (kMapScrollingSpeedTable[(MapScrolling_GetSpeedIndex(game) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as usize]),
    ));
    MapScrolling_Common(game);
}
pub unsafe fn MapScrolling_Common(game: &mut Game) {
    let ref mut fresh100 = *((&raw mut game.g_ram as *mut uint8).offset(0x5ff) as *mut uint16);
    *fresh100 = (*fresh100).wrapping_add(1);
    let ref mut fresh101 = *((&raw mut game.g_ram as *mut uint8).offset(0x5ff) as *mut uint16);
    *fresh101 = (*fresh101).wrapping_add(1);
    if *fresh101 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        QueueSfx1_Max6(0x36 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x5fd) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x5ff) as *mut uint16) = 0 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5fb) as *mut uint16) != 0 {
            let ref mut fresh102 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x5fb) as *mut uint16);
            *fresh102 = (*fresh102).wrapping_sub(1);
        }
    }
}
pub unsafe fn MapScrolling_2_Right(game: &mut Game) {
    let ref mut fresh104 = *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16);
    *fresh104 = ((*fresh104).wrapping_add(
        (kMapScrollingSpeedTable[(MapScrolling_GetSpeedIndex(game) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as usize]),
    ));
    MapScrolling_Common(game);
}
pub unsafe fn MapScrolling_3_Up(game: &mut Game) {
    let ref mut fresh103 = *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16);
    *fresh103 = ((*fresh103).wrapping_sub(
        (kMapScrollingSpeedTable[(MapScrolling_GetSpeedIndex(game) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as usize]),
    ));
    MapScrolling_Common(game);
}
pub unsafe fn MapScrolling_4_Down(game: &mut Game) {
    let ref mut fresh99 = *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16);
    *fresh99 = ((*fresh99).wrapping_add(
        (kMapScrollingSpeedTable[(MapScrolling_GetSpeedIndex(game) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as usize]),
    ));
    MapScrolling_Common(game);
}
pub unsafe fn GameState_16_Unpausing_Async(game: &mut Game) -> CoroutineRet {
    HighlightPauseScreenButton(game);
    DrawPauseMenuDuringFadeout(game);
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 0x80 as ::core::ffi::c_int
    {
        EnableNMI(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        let ref mut fresh106 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
        *fresh106 = (*fresh106).wrapping_add(1);
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn DrawPauseMenuDuringFadeout(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x763) as *mut uint16) as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
    {
        EquipmentScreenDrawItemSelector(game);
        EquipmentScreenDisplayReserveTankAmount(game);
        HandlePauseMenuLRPressHighlight(game);
    } else {
        DisplayMapElevatorDestinations(game);
        DrawMapIcons(game);
        MapScreenDrawSamusPositionIndicator(game);
    };
}
pub unsafe fn GameState_17_Unpausing_Async(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut uint8 = (&raw mut game.g_ram as *mut uint8).offset(0x612);
    let mut current_block_17: u64;
    match *_state_variable_ as ::core::ffi::c_int {
        0 => {
            ClearSamusBeamTiles(game);
            ContinueInitGameplayResume(game);
            ResumeGameplay(game);
            RestoreSomeGfxStateForPause(game);
            RestoreBG2TilemapFromPauseScreen(game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 1 as uint16;
            current_block_17 = 16264310372000621783;
        }
        1 => {
            current_block_17 = 16264310372000621783;
        }
        _ => {
            current_block_17 = 4166486009154926805;
        }
    }
    match current_block_17 {
        16264310372000621783 => {
            let mut _coret_: uint8 = CallUnpauseHook_Async(
                Load24((&raw mut game.g_ram as *mut uint8).offset(0x604) as *mut LongPtr),
                game,
            );
            if _coret_ != 0 {
                *_state_variable_ = 1 as uint8;
                return _coret_;
            }
            EnableHdmaObjects(game);
            EnableAnimtiles(game);
            QueueSamusMovementSfx(game);
            let ref mut fresh108 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
            *fresh108 = (*fresh108).wrapping_add(1);
        }
        _ => {}
    }
    *_state_variable_ = 0 as uint8;
    return 0 as CoroutineRet;
}
pub unsafe fn GameState_18_Unpausing(game: &mut Game) -> CoroutineRet {
    let mut _coret_: uint8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    HandleFadeIn(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
            kGameState_8_MainGameplay as uint16;
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn LoadPauseMenuMapTilemapAndAreaLabel(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xbd) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xbf) as *mut uint16);
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x30 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    LoadPauseMenuMapTilemap(game);
    static unk_8293E9: StartDmaCopy = StartDmaCopy::vram_words(1, 0x7e4000, 0x1000);
    SetupDmaTransfer(&raw const unk_8293E9 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    WriteReg(VMADDL as uint16, 0xaa as uint8, game);
    WriteReg(VMADDH as uint16, 0x38 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    WriteReg(DMAP1 as uint16, 1 as uint8, game);
    WriteReg(BBAD1 as uint16, 0x18 as uint8, game);
    WriteReg(DAS1L as uint16, 0x18 as uint8, game);
    WriteReg(DAS1H as uint16, 0 as uint8, game);
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as ::core::ffi::c_int
        - 7 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        v0 = 0 as uint16;
    }
    WriteRegWord(
        A1T1L as uint16,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x82965f)) as *const uint8 as *mut uint16)
            .offset(v0 as isize),
        game,
    );
    WriteReg(A1B1 as uint16, 0x82 as uint8, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
}
pub unsafe fn LoadPauseMenuMapTilemap(game: &mut Game) {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as ::core::ffi::c_int
        - 7 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        v0 = 0 as uint16;
    }
    let mut r0: *const uint16 = RomPtr(
        Load24(
            (game.g_rom.offset(crate::rtl::lorom_offset(0x82964a)) as *const uint8 as *mut LongPtr)
                .offset(v0 as isize) as *mut LongPtr,
        ),
        game,
    ) as *const uint16;
    let mut r3: *mut uint16 =
        (&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000 as *mut uint16;
    let mut r6: *const uint16 = RomPtr_82(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x829717)) as *const uint8 as *mut uint16)
            .offset(v0 as isize),
        game,
    ) as *const uint16;
    let mut r9: *const uint16 = (&raw mut game.g_ram as *mut uint8).offset(0x7f7) as *const uint16;
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd908)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        != 0
    {
        let fresh86 = r6;
        r6 = r6.offset(1);
        let mut r38: uint16 = u16::swap_bytes(*fresh86);
        let fresh87 = r9;
        r9 = r9.offset(1);
        let mut r40: uint16 = u16::swap_bytes(*fresh87);
        let mut v8: uint16 = 0 as uint16;
        let mut v9: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
        loop {
            let mut v10: uint16 =
                *r0.offset((v8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            let mut v11: bool = r40 as ::core::ffi::c_int >> 15 as ::core::ffi::c_int != 0;
            r40 = ((r40).wrapping_mul((2u16)));
            if v11 {
                v10 = (v10 as ::core::ffi::c_int & !(0x400 as ::core::ffi::c_int)) as uint16;
                r38 = ((r38).wrapping_mul((2u16)));
            } else {
                v11 = r38 as ::core::ffi::c_int >> 15 as ::core::ffi::c_int != 0;
                r38 = ((r38).wrapping_mul((2u16)));
                if !v11 {
                    v10 = 31 as uint16;
                }
            }
            *r3.offset((v8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v10;
            v9 -= 1;
            if v9 == 0 {
                v9 = 16 as ::core::ffi::c_int;
                let fresh88 = r6;
                r6 = r6.offset(1);
                r38 = u16::swap_bytes(*fresh88);
                let fresh89 = r9;
                r9 = r9.offset(1);
                r40 = u16::swap_bytes(*fresh89);
            }
            v8 = ((v8).wrapping_add((2u16)));
            if !(((((v8) as i16).wrapping_sub((4096i16))) as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    } else {
        let mut v1: uint16 = 0 as uint16;
        let mut v2: uint16 = 0 as uint16;
        let mut r18: uint8 = 0 as uint8;
        loop {
            let mut t: uint8 = *(&raw mut game.g_ram as *mut uint8)
                .offset(0x7f7)
                .offset(v2 as isize);
            let ref mut fresh90 = *(&raw mut game.g_ram as *mut uint8)
                .offset(0x7f7)
                .offset(v2 as isize);
            *fresh90 = ((*fresh90 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int) as uint8;
            if t as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 {
                *r3.offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                    0x1f as uint16;
            } else {
                let ref mut fresh91 = *(&raw mut game.g_ram as *mut uint8)
                    .offset(0x7f7)
                    .offset(v2 as isize);
                *fresh91 = (*fresh91).wrapping_add(1);
                *r3.offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = ((*r0
                    .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize))
                    & (0xfbffu16));
            }
            v1 = ((v1).wrapping_add((2u16)));
            r18 = r18.wrapping_add(1);
            if !(r18 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int) {
                continue;
            }
            r18 = 0 as uint8;
            v2 = v2.wrapping_add(1);
            if (((v2) as i16).wrapping_sub((256i16))) as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                break;
            }
        }
    };
}
pub unsafe fn DrawRoomSelectMap(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 51 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 19 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) =
        -(40 as ::core::ffi::c_int) as uint16;
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as ::core::ffi::c_int
        - 7 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        v0 = 0 as uint16;
    }
    let mut r0: *const uint16 = RomPtr(
        Load24(
            (game.g_rom.offset(crate::rtl::lorom_offset(0x82964a)) as *const uint8 as *mut LongPtr)
                .offset(v0 as isize) as *mut LongPtr,
        ),
        game,
    ) as *const uint16;
    let mut r3: *mut uint16 =
        (&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000 as *mut uint16;
    let mut r6: *const uint16 = RomPtr_82(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x829717)) as *const uint8 as *mut uint16)
            .offset(v0 as isize),
        game,
    ) as *const uint16;
    let mut r9: *const uint16 = (&raw mut game.g_ram as *mut uint8).offset(0x7f7) as *const uint16;
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd908)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        != 0
    {
        let fresh43 = r6;
        r6 = r6.offset(1);
        let mut r38: uint16 = u16::swap_bytes(*fresh43);
        let fresh44 = r9;
        r9 = r9.offset(1);
        let mut r40: uint16 = u16::swap_bytes(*fresh44);
        let mut v9: uint16 = 0 as uint16;
        let mut v10: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
        loop {
            let mut v11: uint16 =
                *r0.offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            let mut v12: bool = r40 as ::core::ffi::c_int >> 15 as ::core::ffi::c_int != 0;
            r40 = ((r40).wrapping_mul((2u16)));
            if v12 {
                v11 = (v11 as ::core::ffi::c_int & !(0x400 as ::core::ffi::c_int)) as uint16;
                r38 = ((r38).wrapping_mul((2u16)));
            } else {
                v12 = r38 as ::core::ffi::c_int >> 15 as ::core::ffi::c_int != 0;
                r38 = ((r38).wrapping_mul((2u16)));
                if !v12 {
                    v11 = 31 as uint16;
                }
            }
            *r3.offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v11;
            v10 -= 1;
            if v10 == 0 {
                v10 = 16 as ::core::ffi::c_int;
                let fresh45 = r6;
                r6 = r6.offset(1);
                r38 = u16::swap_bytes(*fresh45);
                let fresh46 = r9;
                r9 = r9.offset(1);
                r40 = u16::swap_bytes(*fresh46);
            }
            v9 = ((v9).wrapping_add((2u16)));
            if !(((((v9) as i16).wrapping_sub((4096i16))) as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    } else {
        let mut v1: uint16 = 0 as uint16;
        let mut v2: uint16 = 0 as uint16;
        let mut r18: uint8 = 0 as uint8;
        loop {
            let mut what: uint8 = *(&raw mut game.g_ram as *mut uint8)
                .offset(0x7f7)
                .offset(v2 as isize);
            let ref mut fresh47 = *(&raw mut game.g_ram as *mut uint8)
                .offset(0x7f7)
                .offset(v2 as isize);
            *fresh47 = ((*fresh47 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int) as uint8;
            if what as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 {
                *r3.offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                    0xf as uint16;
            } else {
                let ref mut fresh48 = *(&raw mut game.g_ram as *mut uint8)
                    .offset(0x7f7)
                    .offset(v2 as isize);
                *fresh48 = (*fresh48).wrapping_add(1);
                *r3.offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                    (*r0.offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        & !(0x400 as ::core::ffi::c_int)) as uint16;
            }
            v1 = ((v1).wrapping_add((2u16)));
            r18 = r18.wrapping_add(1);
            if !(r18 as ::core::ffi::c_int - 8 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
                == 0)
            {
                continue;
            }
            r18 = 0 as uint8;
            v2 = v2.wrapping_add(1);
            if (((v2) as i16).wrapping_sub((256i16))) as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                break;
            }
        }
    }
    let mut v16: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    let mut v17: *mut VramWriteEntry = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v17).size = 4096 as uint16;
    (*v17).src.addr =
        ((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long as VoidP;
    (*v17).src.bank = 126 as uint8;
    (*v17).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x58) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        ((v16).wrapping_add((7u16)));
}
pub unsafe fn DrawRoomSelectMapAreaLabel(mut dst: *mut uint16, game: &mut Game) {
    let mut v2: *const uint16 = RomPtr_82(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x82965f)) as *const uint8 as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize),
        game,
    ) as *const uint16;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 12 as ::core::ffi::c_int {
        *dst.offset(i as isize) = ((*v2.offset(i as isize)) & (0xefffu16));
        i += 1;
    }
}
pub unsafe fn SetupMapScrollingForPauseMenu(mut a: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v2: int16 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x5ac) as *mut uint16) as ::core::ffi::c_int
            + (((*((&raw mut game.g_ram as *mut uint8).offset(0x5ae) as *mut uint16)).wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x5ac) as *mut uint16)),
            )) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int)
            - 128 as ::core::ffi::c_int) as uint16;
    let mut r18: uint16 = (8 as ::core::ffi::c_int
        * (*((&raw mut game.g_ram as *mut uint8).offset(0x7a1) as *mut uint16)
            as ::core::ffi::c_int
            + ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int))
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    v1 = ((224i16).wrapping_sub(((r18) as i16)));
    if ((224i16).wrapping_sub(((r18) as i16))) as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        r18 = ((32u16).wrapping_sub((r18)));
        if r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            let ref mut fresh19 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16);
            *fresh19 = ((*fresh19).wrapping_sub((r18)));
        }
    } else {
        r18 = ((224u16).wrapping_sub((r18)));
        let ref mut fresh20 = *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16);
        *fresh20 = ((*fresh20).wrapping_sub(((v1) as u16)));
    }
    r18 = (*((&raw mut game.g_ram as *mut uint8).offset(0x5b0) as *mut uint16)
        as ::core::ffi::c_int
        + (((*((&raw mut game.g_ram as *mut uint8).offset(0x5b2) as *mut uint16))
            .wrapping_sub((*((&raw mut game.g_ram as *mut uint8).offset(0x5b0) as *mut uint16))))
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int)
        + 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) =
        -(a as ::core::ffi::c_int - r18 as ::core::ffi::c_int & 0xfff8 as ::core::ffi::c_int)
            as uint16;
    r18 = (8 as ::core::ffi::c_int
        * (*((&raw mut game.g_ram as *mut uint8).offset(0x7a3) as *mut uint16)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int)
        + (a as ::core::ffi::c_int - r18 as ::core::ffi::c_int & 0xfff8 as ::core::ffi::c_int))
        as uint16;
    v2 = ((64i16).wrapping_sub(((r18) as i16)));
    if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        let ref mut fresh21 = *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16);
        *fresh21 = ((*fresh21).wrapping_sub(((v2) as u16)));
        if *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) as ::core::ffi::c_int
            + 40 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) =
                -(40 as ::core::ffi::c_int) as uint16;
        }
    }
}
static k0x80Shr: [uint8; 8] = [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1];
static k0x80Shr_0: [uint8; 8] = [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1];
unsafe fn DetermineLeftmostMapColumn(mut r0: *const uint8) -> uint16 {
    let mut r18: uint8 = 0;
    let mut v2: uint16 = 0;
    let mut v0: uint16 = 0;
    let mut result: uint16 = 0 as uint16;
    '_LABEL_2: loop {
        v0 = ((result) & (7u16));
        r18 = k0x80Shr[v0 as usize];
        v2 = 0 as uint16;
        loop {
            if !(r18 as ::core::ffi::c_int & *r0.offset(v2 as isize) as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int)
            {
                break '_LABEL_2;
            }
            v2 = ((v2).wrapping_add((4u16)));
            if !((((v2) as i16).wrapping_sub((128i16))) as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int)
            {
                continue;
            }
            result = result.wrapping_add(1);
            if (((result) as i16).wrapping_sub((64i16))) as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                return 26 as uint16;
            }
            if result as ::core::ffi::c_int & 7 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                r0 = r0.offset(1);
            }
            if result as ::core::ffi::c_int == 32 as ::core::ffi::c_int {
                r0 = r0.offset(123);
            }
            break;
        }
    }
    return result;
}
unsafe fn DetermineRightmostMapColumn(mut r0: *const uint8) -> uint16 {
    let mut r18: uint8 = 0;
    let mut v1: uint16 = 0;
    let mut result: uint16 = 63 as uint16;
    's_7: loop {
        r18 = k0x80Shr_0[(result as ::core::ffi::c_int & 7 as ::core::ffi::c_int) as usize];
        v1 = 0 as uint16;
        loop {
            if !(r18 as ::core::ffi::c_int & *r0.offset(v1 as isize) as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int)
            {
                break 's_7;
            }
            v1 = ((v1).wrapping_add((4u16)));
            if !((((v1) as i16).wrapping_sub((128i16))) as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int)
            {
                continue;
            }
            result = result.wrapping_sub(1);
            if result as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                return 28 as uint16;
            }
            if result as ::core::ffi::c_int & 7 as ::core::ffi::c_int == 7 as ::core::ffi::c_int {
                r0 = r0.offset(-1);
            }
            if result as ::core::ffi::c_int == 31 as ::core::ffi::c_int {
                r0 = r0.offset(-(124 as ::core::ffi::c_int as isize));
            }
            break;
        }
    }
    return result;
}
unsafe fn DetermineTopmostMapColumn(mut r0: *const uint8) -> uint16 {
    let mut r3: *const uint8 = r0.offset(128);
    let mut result: uint16 = 0 as uint16;
    let mut v1: uint16 = 0 as uint16;
    while *r0.offset(v1 as isize) == 0 && *r3.offset(v1 as isize) == 0 {
        v1 = v1.wrapping_add(1);
        if (((v1) as i16).wrapping_sub((4i16))) as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            v1 = 0 as uint16;
            r0 = r0.offset(4);
            r3 = r3.offset(4);
            result = result.wrapping_add(1);
            if (((result) as i16).wrapping_sub((31i16))) as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                return 1 as uint16;
            }
        }
    }
    return result;
}
unsafe fn DetermineBottommostMapColumn(mut r0: *const uint8) -> uint16 {
    let mut r3: *const uint8 = r0.offset(128);
    let mut result: uint16 = 31 as uint16;
    let mut v1: uint16 = 0 as uint16;
    while *r0.offset(v1 as isize) == 0 && *r3.offset(v1 as isize) == 0 {
        v1 = v1.wrapping_add(1);
        if (((v1) as i16).wrapping_sub((4i16))) as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            v1 = 0 as uint16;
            r0 = r0.offset(-(4 as ::core::ffi::c_int as isize));
            r3 = r3.offset(-(4 as ::core::ffi::c_int as isize));
            result = result.wrapping_sub(1);
            if result == 0 {
                return 11 as uint16;
            }
        }
    }
    return result;
}
pub unsafe fn DetermineMapScrollLimits(game: &mut Game) {
    let mut r6: *const uint8 = ::core::ptr::null::<uint8>();
    if *((&raw mut game.g_ram as *mut uint8).offset(0x789) as *mut uint16) != 0 {
        r6 = RomPtr_82(
            *(RomPtr_82(
                (addr_kPauseMenuMapData as ::core::ffi::c_int
                    + 2 as ::core::ffi::c_int
                        * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                            as ::core::ffi::c_int) as uint16_t,
                game,
            ) as *mut uint16),
            game,
        );
    } else {
        r6 = (&raw mut game.g_ram as *mut uint8).offset(0x7f7);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x5ac) as *mut uint16) =
        ((DetermineLeftmostMapColumn(r6)).wrapping_mul((8u16)));
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        let ref mut fresh22 = *((&raw mut game.g_ram as *mut uint8).offset(0x5ac) as *mut uint16);
        *fresh22 = ((*fresh22).wrapping_sub((24u16)));
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x5ae) as *mut uint16) =
        ((DetermineRightmostMapColumn(r6.offset(131))).wrapping_mul((8u16)));
    *((&raw mut game.g_ram as *mut uint8).offset(0x5b0) as *mut uint16) =
        ((DetermineTopmostMapColumn(r6)).wrapping_mul((8u16)));
    *((&raw mut game.g_ram as *mut uint8).offset(0x5b2) as *mut uint16) =
        ((DetermineBottommostMapColumn(r6.offset(124))).wrapping_mul((8u16)));
}
pub unsafe fn SetupPpuForPauseMenu(game: &mut Game) {
    WriteReg(OBSEL as uint16, 1 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x52) = 1 as uint8;
    WriteReg(BGMODE as uint16, 9 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 9 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 0 as uint8;
    WriteReg(BG12NBA as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 4 as uint8;
    WriteReg(BG34NBA as uint16, 4 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 49 as uint8;
    WriteReg(BG1SC as uint16, 0x31 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 56 as uint8;
    WriteReg(BG2SC as uint16, 0x38 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5a) = 88 as uint8;
    WriteReg(BG3SC as uint16, 0x58 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5c) = 0 as uint8;
    WriteReg(BG4SC as uint16, 0 as uint8, game);
    WriteReg(TM as uint16, 0x17 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 23 as uint8;
    WriteReg(MOSAIC as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x57) = 0 as uint8;
    let ref mut fresh24 = *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0);
    *fresh24 = ((*fresh24) & (0xe0u8));
    let ref mut fresh25 = *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1);
    *fresh25 = ((*fresh25) & (0xe0u8));
    let ref mut fresh26 = *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2);
    *fresh26 = ((*fresh26) & (0xe0u8));
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 0 as uint8;
}
pub unsafe fn ResetPauseMenuAnimations(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x757) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb9) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xbb) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x753) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x73f) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x745) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x776) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x778) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x77a) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x72b) as *mut uint16) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c10c)) as *const uint8 as *mut uint8
            as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x73b) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x74f) as *mut uint16) = 0 as uint16;
}
unsafe fn ChangePaletteValues(mut target: *mut uint16, mut new_color: uint16, mut n: size_t) {
    let mut i: size_t = 0 as size_t;
    while i != n {
        *target.offset(i as isize) = (*target.offset(i as isize) as ::core::ffi::c_int
            & !(0x1c00 as ::core::ffi::c_int)
            | new_color as ::core::ffi::c_int) as uint16;
        i = i.wrapping_add(1);
    }
}
pub unsafe fn LoadEquipmentScreenEquipmentTilemaps(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9d4) as *mut uint16) != 0 {
        memcpy(
            (&raw mut game.g_ram as *mut uint8).offset(
                *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c068)) as *const uint8
                    as *mut uint16)
                    .offset(0) as ::core::ffi::c_int as isize,
            ) as *mut ::core::ffi::c_void,
            RomPtr_82(
                *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c088)) as *const uint8
                    as *mut uint16)
                    .offset(0),
                game,
            ) as *const ::core::ffi::c_void,
            14 as size_t,
        );
        memcpy(
            (&raw mut game.g_ram as *mut uint8).offset(
                *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c068)) as *const uint8
                    as *mut uint16)
                    .offset(1) as ::core::ffi::c_int as isize,
            ) as *mut ::core::ffi::c_void,
            RomPtr_82(
                *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c088)) as *const uint8
                    as *mut uint16)
                    .offset(1),
                game,
            ) as *const ::core::ffi::c_void,
            14 as size_t,
        );
    }
    let mut p: *const uint16 =
        RomPtr_82(addr_kEquipmentTilemapOffs_Weapons as uint16_t, game) as *mut uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa76) as *mut uint16) != 0 {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 5 as ::core::ffi::c_int {
            memcpy(
                (&raw mut game.g_ram as *mut uint8)
                    .offset(*p.offset(i as isize) as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                RomPtr_82(
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c0a8)) as *const uint8
                        as *mut uint16)
                        .offset(i as isize),
                    game,
                ) as *const ::core::ffi::c_void,
                10 as size_t,
            );
            i += 1;
        }
    } else {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < 6 as ::core::ffi::c_int {
            let mut target: *mut uint16 = (&raw mut game.g_ram as *mut uint8)
                .offset(*p.offset(i_0 as isize) as ::core::ffi::c_int as isize)
                as *mut uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9a8) as *mut uint16)
                as ::core::ffi::c_int
                & *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c04c)) as *const uint8
                    as *mut uint16)
                    .offset(i_0 as isize) as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                memcpy(
                    target as *mut ::core::ffi::c_void,
                    RomPtr_82(
                        *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c08c)) as *const uint8
                            as *mut uint16)
                            .offset(i_0 as isize),
                        game,
                    ) as *const ::core::ffi::c_void,
                    10 as size_t,
                );
                if *((&raw mut game.g_ram as *mut uint8).offset(0x9a6) as *mut uint16)
                    as ::core::ffi::c_int
                    & *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c04c)) as *const uint8
                        as *mut uint16)
                        .offset(i_0 as isize) as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    ChangePaletteValues(target, 3072 as uint16, 5 as size_t);
                }
            } else {
                memcpy(
                    target as *mut ::core::ffi::c_void,
                    RomPtr_82(addr_kEquipmentScreenTilemap_Blank as uint16_t, game)
                        as *const ::core::ffi::c_void,
                    10 as size_t,
                );
            }
            i_0 += 1;
        }
    }
    p = RomPtr_82(addr_kEquipmentTilemapOffs_Suits as uint16_t, game) as *mut uint16;
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 < 6 as ::core::ffi::c_int {
        let mut target_0: *mut uint16 = (&raw mut game.g_ram as *mut uint8)
            .offset(*p.offset(i_1 as isize) as ::core::ffi::c_int as isize)
            as *mut uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16) as ::core::ffi::c_int
            & *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c056)) as *const uint8
                as *mut uint16)
                .offset(i_1 as isize) as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            memcpy(
                target_0 as *mut ::core::ffi::c_void,
                RomPtr_82(
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c096)) as *const uint8
                        as *mut uint16)
                        .offset(i_1 as isize),
                    game,
                ) as *const ::core::ffi::c_void,
                18 as size_t,
            );
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16)
                as ::core::ffi::c_int
                & *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c056)) as *const uint8
                    as *mut uint16)
                    .offset(i_1 as isize) as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                ChangePaletteValues(target_0, 3072 as uint16, 9 as size_t);
            }
        } else {
            memcpy(
                target_0 as *mut ::core::ffi::c_void,
                RomPtr_82(addr_kEquipmentScreenTilemap_Blank as uint16_t, game)
                    as *const ::core::ffi::c_void,
                18 as size_t,
            );
        }
        i_1 += 1;
    }
    p = RomPtr_82(addr_kEquipmentTilemapOffs_Boots as uint16_t, game) as *mut uint16;
    let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_2 < 3 as ::core::ffi::c_int {
        let mut target_1: *mut uint16 = (&raw mut game.g_ram as *mut uint8)
            .offset(*p.offset(i_2 as isize) as ::core::ffi::c_int as isize)
            as *mut uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16) as ::core::ffi::c_int
            & *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c062)) as *const uint8
                as *mut uint16)
                .offset(i_2 as isize) as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            memcpy(
                target_1 as *mut ::core::ffi::c_void,
                RomPtr_82(
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c0a2)) as *const uint8
                        as *mut uint16)
                        .offset(i_2 as isize),
                    game,
                ) as *const ::core::ffi::c_void,
                18 as size_t,
            );
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16)
                as ::core::ffi::c_int
                & *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c062)) as *const uint8
                    as *mut uint16)
                    .offset(i_2 as isize) as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                ChangePaletteValues(target_1, 3072 as uint16, 9 as size_t);
            }
        } else {
            memcpy(
                target_1 as *mut ::core::ffi::c_void,
                RomPtr_82(addr_kEquipmentScreenTilemap_Blank as uint16_t, game)
                    as *const ::core::ffi::c_void,
                18 as size_t,
            );
        }
        i_2 += 1;
    }
}
pub unsafe fn ClearSamusBeamTiles(game: &mut Game) {
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x60 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_82A2D4: StartDmaCopy = StartDmaCopy::vram_words(1, 0x9ad200, 0x1000);
    SetupDmaTransfer(&raw const unk_82A2D4 as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
}
pub unsafe fn ContinueInitGameplayResume(game: &mut Game) {
    let mut v0: int16 = 0;
    v0 = 512 as int16;
    let mut v1: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(v1 as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap
                [(v1 as ::core::ffi::c_int + 384 as ::core::ffi::c_int) as usize];
        v1 = v1.wrapping_add(1);
        v0 = ((v0).wrapping_sub((2i16)));
        if !(v0 != 0) {
            break;
        }
    }
    SetupPpuForGameplayResume(game);
    CalculateBgScrolls_Unpause(game);
    UpdateBeamTilesAndPalette_Unpause(game);
    ClearPauseMenuData(game);
    CallSomeSamusCode(0xc as uint16, game);
}
pub unsafe fn SetupPpuForGameplayResume(game: &mut Game) {
    WriteReg(OBSEL as uint16, 3 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x52) = 3 as uint8;
    WriteReg(BGMODE as uint16, 9 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 9 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 0 as uint8;
    WriteReg(BG12NBA as uint16, 0 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 4 as uint8;
    WriteReg(BG34NBA as uint16, 4 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x58) = 81 as uint8;
    WriteReg(BG1SC as uint16, 0x51 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 73 as uint8;
    WriteReg(BG2SC as uint16, 0x49 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5a) = 90 as uint8;
    WriteReg(BG3SC as uint16, 0x5a as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5c) = 0 as uint8;
    WriteReg(BG4SC as uint16, 0 as uint8, game);
}
pub unsafe fn CalculateBgScrolls_Unpause(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x91d) as *mut uint16))
            .wrapping_add((*((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16))));
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x91f) as *mut uint16))
            .wrapping_add((*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16))));
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x921) as *mut uint16))
            .wrapping_add((*((&raw mut game.g_ram as *mut uint8).offset(0x917) as *mut uint16))));
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x923) as *mut uint16))
            .wrapping_add((*((&raw mut game.g_ram as *mut uint8).offset(0x919) as *mut uint16))));
}
pub unsafe fn UpdateBeamTilesAndPalette_Unpause(game: &mut Game) {
    UpdateBeamTilesAndPalette(game);
}
pub unsafe fn ClearPauseMenuData(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x729) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x72b) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x72d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x72f) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x731) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x733) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x735) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x737) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x739) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x73d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x73f) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x741) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x743) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x745) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x747) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x749) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x74b) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x74d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x751) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x753) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x757) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x759) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x75b) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x75d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x75f) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x761) as *mut uint16) = 0 as uint16;
}
pub fn sub_82A425() {}
pub unsafe fn sub_82A42A(game: &mut Game) {
    let mut current_block: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
        == kPose_81_FaceR_Screwattack as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_82_FaceL_Screwattack as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16) as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            return;
        }
        current_block = 2878476956402529644;
    } else if (*((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16)
        as ::core::ffi::c_int
        == kPose_1B_FaceR_SpaceJump as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16)
            as ::core::ffi::c_int
            == kPose_1C_FaceL_SpaceJump as ::core::ffi::c_int)
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16) as ::core::ffi::c_int
            & 0x20 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        current_block = 2878476956402529644;
    } else {
        current_block = 10886091980245723256;
    }
    match current_block {
        2878476956402529644 => {
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
                == 4 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) =
                    kPose_1A_FaceL_SpinJump as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) =
                    kPose_19_FaceR_SpinJump as uint16;
            }
            SamusFunc_F433(game);
            Samus_SetAnimationFrameIfPoseChanged(game);
        }
        _ => {}
    };
}
pub unsafe fn sub_82A47B(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16) as ::core::ffi::c_int
        & 4 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) =
                kPose_41_FaceL_Morphball_Ground as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) =
                kPose_1D_FaceR_Morphball_Ground as uint16;
        }
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
    }
}
pub unsafe fn sub_82A4A9(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) =
                kPose_7A_FaceL_Springball_Ground as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) =
                kPose_79_FaceR_Springball_Ground as uint16;
        }
        SamusFunc_F433(game);
        Samus_SetAnimationFrameIfPoseChanged(game);
    }
}
pub unsafe fn HandlePauseScreenLR(game: &mut Game) {
    HandlePauseScreenLrInput(game);
    DrawLrHighlight(game);
}
pub unsafe fn HandlePauseScreenLrInput(game: &mut Game) {
    let mut current_block: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e1) as *mut uint16) as ::core::ffi::c_int
        & kButton_L as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x753) as *mut uint16) == 0 {
            return;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x729) as *mut uint16) = 5 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) = 5 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x751) as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x753) as *mut uint16) = 0 as uint16;
        SetPauseScreenButtonLabelPalettes(game);
        current_block = 2636532137229557926;
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x5e1) as *mut uint16)
        as ::core::ffi::c_int
        & kButton_R as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x753) as *mut uint16) as ::core::ffi::c_int
            != 2 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x729) as *mut uint16) = 5 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) = 2 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x751) as *mut uint16) = 2 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x753) as *mut uint16) = 2 as uint16;
        SetPauseScreenButtonLabelPalettes(game);
        current_block = 2636532137229557926;
    } else {
        current_block = 13536709405535804910;
    }
    match current_block {
        2636532137229557926 => {
            QueueSfx1_Max6(0x38 as uint16, game);
        }
        _ => {}
    };
}
static kPauseLrButtonPressedHighlight_Spritemap: [uint16; 2] = [0x28, 0x29];
static kPauseLrButtonPressedHighlight_Y: [uint16; 2] = [0x18, 0xe8];
static kPauseLrButtonPressedHighlight_X: [uint16; 2] = [0xd0, 0xd0];
pub unsafe fn HandlePauseMenuLRPressHighlight(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x729) as *mut uint16) != 0 {
        let ref mut fresh42 = *((&raw mut game.g_ram as *mut uint8).offset(0x729) as *mut uint16);
        *fresh42 = (*fresh42).wrapping_sub(1);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x751) as *mut uint16) != 0 {
            let mut v0: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x751)
                as *mut uint16) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int;
            DrawMenuSpritemap(
                kPauseLrButtonPressedHighlight_Spritemap[v0 as usize],
                kPauseLrButtonPressedHighlight_Y[v0 as usize],
                ((kPauseLrButtonPressedHighlight_X[v0 as usize]).wrapping_sub((1u16))),
                0 as uint16,
                game,
            );
        }
    }
}
pub unsafe fn DrawLrHighlight(game: &mut Game) {
    DrawPauseScreenSpriteAnim(2 as uint16, 0x18 as uint16, 0xd0 as uint16, game);
    DrawPauseScreenSpriteAnim(2 as uint16, 0xe8 as uint16, 0xd0 as uint16, game);
}
pub unsafe fn HandlePauseScreenStart(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e1) as *mut uint16) as ::core::ffi::c_int
        & kButton_Start as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x38 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 1 as uint16;
        let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x753) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x753) as *mut uint16) = 1 as uint16;
        SetPauseScreenButtonLabelPalettes(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x753) as *mut uint16) = v0;
        *((&raw mut game.g_ram as *mut uint8).offset(0x729) as *mut uint16) = 11 as uint16;
        let ref mut fresh98 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
        *fresh98 = (*fresh98).wrapping_add(1);
    }
    UpdatePauseMenuLRStartVramTilemap(game);
}
pub unsafe fn HighlightPauseScreenButton(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x729) as *mut uint16) != 0 {
        let ref mut fresh107 = *((&raw mut game.g_ram as *mut uint8).offset(0x729) as *mut uint16);
        *fresh107 = (*fresh107).wrapping_sub(1);
        DrawMenuSpritemap(
            0x2b as uint16,
            0x90 as uint16,
            0xd0 as uint16,
            0 as uint16,
            game,
        );
    }
}
pub fn SetPauseScreenButtonLabelPalettes(game: &mut Game) {
    set_pause_label_palettes(game.pausemenu_button_label_mode(), game);
}

fn set_pause_label_palettes(mode: u16, game: &mut Game) {
    // First tile and width of each two-row label in the pause tilemap.
    const LEFT: (usize, usize) = (805, 5);
    const START: (usize, usize) = (812, 4);
    const RIGHT: (usize, usize) = (822, 5);
    const TILEMAP: usize = 0x3000;

    // Modes retain the original label order, including the unpause mode (1).
    let labels = [
        [(RIGHT, 0x0800), (START, 0x0800), (LEFT, 0x1400)],
        [(START, 0x0800), (LEFT, 0x1400), (RIGHT, 0x1400)],
        [(LEFT, 0x0800), (START, 0x0800), (RIGHT, 0x1400)],
    ][usize::from(mode)];
    for ((first_tile, width), palette) in labels {
        for row in [first_tile, first_tile + 32] {
            for tile in row..row + width {
                let address = TILEMAP + 2 * tile;
                let word = u16::from_le_bytes([game.g_ram[address], game.g_ram[address + 1]]);
                let word = (word & !0x1c00) | palette;
                game.g_ram[address..address + 2].copy_from_slice(&word.to_le_bytes());
            }
        }
    }
}

pub unsafe fn UpdatePauseMenuLRStartVramTilemap(game: &mut Game) {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    (*gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .size = 128 as uint16;
    v0 = ((v0).wrapping_add((2u16)));
    (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size = 0x3640 as uint16;
    v0 = ((v0).wrapping_add((2u16)));
    let fresh23 = v0;
    v0 = v0.wrapping_add(1);
    *(&raw mut (*(gVramWriteEntry
        as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
        fresh23 as ::core::ffi::c_int,
        game,
    ))
    .size as *mut uint8)
        .offset(0) = 126 as uint8;
    (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size =
        (((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int)
            + 800 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        ((v0).wrapping_add((2u16)));
}
pub unsafe fn DrawPauseScreenSpriteAnim(
    mut a: uint16,
    mut input_k: uint16,
    mut input_j: uint16,
    game: &mut Game,
) {
    let mut t: ::core::ffi::c_int = a as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    let mut v8: *mut uint16 = RomPtr_RAM(
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c0c4)) as *const uint8
            as *mut PauseScreenSpriteAnimationData))
            .arr[t as usize],
        game,
    ) as *mut uint16;
    let mut v5: *mut uint16 = RomPtr_RAM(
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c0b2)) as *const uint8
            as *mut PauseScreenSpriteAnimationData))
            .arr[t as usize],
        game,
    ) as *mut uint16;
    let mut v3: *const uint8 = RomPtr_82(
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c0e8)) as *const uint8
            as *mut PauseScreenSpriteAnimationData))
            .arr[t as usize],
        game,
    );
    *v5 = (*v5).wrapping_sub(1);
    if *v5 as int16 as ::core::ffi::c_int <= 0 as ::core::ffi::c_int {
        *v8 = (*v8).wrapping_add(1);
        let mut v10: uint16 =
            *v3.offset((3 as ::core::ffi::c_int * *v8 as ::core::ffi::c_int) as isize) as uint16;
        if v10 as ::core::ffi::c_int == 255 as ::core::ffi::c_int {
            *v8 = 0 as uint16;
            v10 = *v3.offset(0) as uint16;
        }
        *v5 = v10;
    }
    let mut r3: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c0fa)) as *const uint8
        as *mut uint16)
        .offset(3);
    let mut r24: ::core::ffi::c_int = *v3.offset(
        (3 as ::core::ffi::c_int * *v8 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize,
    ) as ::core::ffi::c_int;
    let mut r26: ::core::ffi::c_int = 2 as ::core::ffi::c_int
        * *RomPtr_RAM(
            (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c0d6)) as *const uint8
                as *mut PauseScreenSpriteAnimationData))
                .arr[t as usize],
            game,
        ) as ::core::ffi::c_int;
    let mut v11: *const uint8 = RomPtr_82(
        (r26 + *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c1e4)) as *const uint8
            as *mut uint16)
            .offset(t as isize) as ::core::ffi::c_int) as uint16_t,
        game,
    );
    DrawMenuSpritemap(
        (r24 + *(v11 as *mut uint16) as ::core::ffi::c_int) as uint16,
        input_k,
        ((input_j).wrapping_sub((1u16))),
        r3,
        game,
    );
}
pub unsafe fn HandlePauseScreenPaletteAnimation(game: &mut Game) {
    let mut v2: uint16 = 0;
    let mut j: uint16 = 0;
    let mut v1: uint8 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x73b) as *mut uint16) as uint8 != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x73b) as *mut uint16 as *mut uint8)
            .offset(0) = (((*((&raw mut game.g_ram as *mut uint8).offset(0x73b) as *mut uint16))
            as u8)
            .wrapping_sub((1u8)));
        if *((&raw mut game.g_ram as *mut uint8).offset(0x73b) as *mut uint16) as uint8 == 0 {
            let mut i: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x74f)
                as *mut uint16) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int;
            loop {
                *((&raw mut game.g_ram as *mut uint8).offset(0x74f) as *mut uint16 as *mut uint8)
                    .offset(0) = i as uint8;
                v1 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c10c)) as *const uint8
                    as *mut uint8)
                    .offset((3 as ::core::ffi::c_int * i) as isize);
                if v1 as ::core::ffi::c_int != 0xff as ::core::ffi::c_int {
                    break;
                }
                QueueSfx3_Max6(0x2a as uint16, game);
                i = 0 as ::core::ffi::c_int;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x73b) as *mut uint16 as *mut uint8)
                .offset(0) = v1;
            *(&raw mut v2 as *mut uint8).offset(0) = 0 as uint8;
            *(&raw mut v2 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1 as usize) as isize,
            ) = *((&raw mut game.g_ram as *mut uint8).offset(0x74f) as *mut uint16) as uint8;
            let mut v3: uint16 = ((v2 as ::core::ffi::c_int >> 3 as ::core::ffi::c_int)
                + 30 as ::core::ffi::c_int) as uint16;
            j = 30 as uint16;
            while j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                    ((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 176 as ::core::ffi::c_int) as isize,
                ) = *(game.g_rom.offset(crate::rtl::lorom_offset(0x82a987)) as *const uint8
                    as *mut uint16)
                    .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
                v3 = ((v3).wrapping_sub((2u16)));
                j = ((j).wrapping_sub((2u16)));
            }
        }
    }
}
pub unsafe fn EquipmentScreenSetupReserveMode(game: &mut Game) {
    let mut current_block: u64;
    let mut v0: VoidP = 0;
    let mut v5: int16 = 0;
    let mut v7: int16 = 0;
    let mut v8: int16 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xbd) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xbf) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9d4) as *mut uint16) as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9c0) as *mut uint16) as ::core::ffi::c_int
            != 0
    {
        v0 = addr_kEquipmentScreenTilemap_AUTO as ::core::ffi::c_int as VoidP;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9c0) as *mut uint16) as ::core::ffi::c_int
            != 1 as ::core::ffi::c_int
        {
            v0 = addr_kEquipmentScreenTilemap_MANUAL as ::core::ffi::c_int as VoidP;
        }
        let mut table: *mut uint16 = RomPtr_82(v0, game) as *mut uint16;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 4 as ::core::ffi::c_int {
            (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
                .cinematic_bg_tilemap[(i + 327 as ::core::ffi::c_int) as usize] =
                ((*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
                    .cinematic_bg_tilemap[(i + 327 as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int
                    & 0xfc00 as ::core::ffi::c_int
                    | *table.offset(i as isize) as ::core::ffi::c_int) as uint16;
            i += 1;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x741) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x72d) as *mut uint16) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c10c)) as *const uint8 as *mut uint8)
            .offset(0) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x743) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x72f) as *mut uint16) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c165)) as *const uint8 as *mut uint8)
            .offset(0) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9d4) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) = 0 as uint16;
    } else {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa76) as *mut uint16) != 0 {
            current_block = 15976848397966268834;
        } else {
            let mut v4: uint16 = 0 as uint16;
            loop {
                if !(*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c04c)) as *const uint8
                    as *mut uint16)
                    .offset((v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    & *((&raw mut game.g_ram as *mut uint8).offset(0x9a8) as *mut uint16)
                        as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int)
                {
                    current_block = 4761528863920922185;
                    break;
                }
                v4 = ((v4).wrapping_add((2u16)));
                if (((v4) as i16).wrapping_sub((10i16))) as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int
                {
                    current_block = 15976848397966268834;
                    break;
                }
            }
            match current_block {
                15976848397966268834 => {}
                _ => {
                    *(&raw mut v5 as *mut uint8).offset(0) =
                        ((v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16
                            as ::core::ffi::c_int
                            >> 8 as ::core::ffi::c_int) as uint8;
                    *(&raw mut v5 as *mut uint8).offset(
                        (::core::mem::size_of::<int16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1 as usize) as isize,
                    ) = (v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint8;
                    *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) =
                        (((v5) as u16) | (1u16));
                    current_block = 9932683223035187777;
                }
            }
        }
        match current_block {
            9932683223035187777 => {}
            _ => {
                let mut v6: uint16 = 0 as uint16;
                loop {
                    if *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c056)) as *const uint8
                        as *mut uint16)
                        .offset((v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        & *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16)
                            as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                    {
                        *(&raw mut v7 as *mut uint8).offset(0) =
                            ((v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16
                                as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int) as uint8;
                        *(&raw mut v7 as *mut uint8).offset(
                            (::core::mem::size_of::<int16>())
                                .wrapping_div(::core::mem::size_of::<uint8>())
                                .wrapping_sub(1 as usize) as isize,
                        ) = (v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint8;
                        *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) =
                            (((v7) as u16) | (2u16));
                        current_block = 9932683223035187777;
                        break;
                    } else {
                        v6 = ((v6).wrapping_add((2u16)));
                        if !(((((v6) as i16).wrapping_sub((12i16))) as ::core::ffi::c_int)
                            < 0 as ::core::ffi::c_int)
                        {
                            current_block = 1109700713171191020;
                            break;
                        }
                    }
                }
                match current_block {
                    9932683223035187777 => {}
                    _ => {
                        v8 = 0 as int16;
                        loop {
                            v8 = ((v8).wrapping_add((2i16)));
                            if !((((v8).wrapping_sub((6i16))) as ::core::ffi::c_int)
                                < 0 as ::core::ffi::c_int)
                            {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16) != 0 {
        EquipmentScreenGlowingArrowSolidOn(game);
        WriteSamusWireframeTilemapAndQueue(game);
    }
}
pub unsafe fn EquipmentScreenTransferBG1Tilemap(game: &mut Game) {
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x30 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_82AC3B: StartDmaCopy = StartDmaCopy::vram_words(1, 0x7e3800, 0x800);
    SetupDmaTransfer(&raw const unk_82AC3B as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = 0 as uint16;
}
static kEquipmentScreenCategories: [Option<Func_V>; 4] = unsafe {
    [
        Some(EquipmentScreenCategory_Tanks as unsafe fn(&mut Game) -> ()),
        Some(EquipmentScreenCategory_Weapons as unsafe fn(&mut Game) -> ()),
        Some(EquipmentScreenCategory_Suit as unsafe fn(&mut Game) -> ()),
        Some(EquipmentScreenCategory_Boots as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn EquipmentScreenMain(game: &mut Game) {
    kEquipmentScreenCategories
        [*((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) as uint8 as usize]
        .expect("non-null function pointer")(game);
    EquipmentScreenDrawItemSelector(game);
    EquipmentScreenDisplayReserveTankAmount(game);
    WriteSamusWireframeTilemapAndQueue(game);
}
static kEquipmentScreenCategory_TanksFuncs: [Option<Func_V>; 2] = unsafe {
    [
        Some(EquipmentScreenCategory_Tanks_0 as unsafe fn(&mut Game) -> ()),
        Some(EquipmentScreenCategory_Tanks_1 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn EquipmentScreenCategory_Tanks(game: &mut Game) {
    kEquipmentScreenCategory_TanksFuncs[*((&raw mut game.g_ram as *mut uint8).offset(0x755)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as usize]
        .expect("non-null function pointer")(game);
    EquipmentScreenHandleDpad(game);
    EquipmentScreenGlowingArrow(game);
}
pub unsafe fn EquipmentScreenHandleDpad(game: &mut Game) {
    let mut r18: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & kButton_Right as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            || EquipmentScreenMoveLowerOnSuitsMisc(0 as uint16, game) as ::core::ffi::c_int
                == 0xffff as ::core::ffi::c_int
        {
            EquipmentScreenMoveToHighJumpOrLowerInBoots(0 as uint16, r18, game);
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & kButton_Up as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            QueueSfx1_Max6(0x37 as uint16, game);
            let ref mut fresh53 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16);
            *fresh53 = ((*fresh53).wrapping_sub((256u16)));
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & kButton_Down as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int
            == 256 as ::core::ffi::c_int
            || *((&raw mut game.g_ram as *mut uint8).offset(0x9c0) as *mut uint16)
                as ::core::ffi::c_int
                == 1 as ::core::ffi::c_int
            || {
                let ref mut fresh54 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16);
                *fresh54 = ((*fresh54).wrapping_add((256u16)));
                *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16) == 0
            }
        {
            EquipmentScreenMoveToBeams(0 as uint16, r18, game);
        } else {
            QueueSfx1_Max6(0x37 as uint16, game);
        }
    }
}
static kEquipmentScreenGlowingArrowFuncs: [Option<Func_V>; 2] = unsafe {
    [
        Some(EquipmentScreenGlowingArrowAnimated as unsafe fn(&mut Game) -> ()),
        Some(EquipmentScreenGlowingArrowSolidOn as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn EquipmentScreenGlowingArrow(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) as uint8 != 0 {
        EquipmentScreenGlowingArrowSolidOff(game);
    } else {
        kEquipmentScreenGlowingArrowFuncs[*((&raw mut game.g_ram as *mut uint8).offset(0x755)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1 as usize) as isize,
            ) as usize]
            .expect("non-null function pointer")(game);
    };
}
static kEquipmentScreenGlowingArrowPalettes0: [uint16; 32] = [
    0x39e, 0x77d, 0xb5c, 0xf5b, 0x133a, 0x171a, 0x1f19, 0x22f8, 0x26d7, 0x2ad6, 0x2eb6, 0x3695,
    0x3a94, 0x3e73, 0x4253, 0x4a52, 0x4a52, 0x4253, 0x3e73, 0x3a94, 0x3695, 0x2eb6, 0x2ad6, 0x26d7,
    0x22f8, 0x1f19, 0x171a, 0x133a, 0xf5b, 0xb5c, 0x77d, 0x39e,
];
static kEquipmentScreenGlowingArrowPalettes1: [uint16; 32] = [
    0x156, 0x155, 0x554, 0x954, 0xd53, 0xd52, 0x1152, 0x1551, 0x1970, 0x1d70, 0x1d6f, 0x216e,
    0x256e, 0x296d, 0x296c, 0x318c, 0x318c, 0x296c, 0x296d, 0x256e, 0x216e, 0x1d6f, 0x1d70, 0x1970,
    0x1551, 0x1152, 0xd52, 0xd53, 0x954, 0x554, 0x155, 0x156,
];
pub unsafe fn EquipmentScreenGlowingArrowAnimated(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9c0) as *mut uint16) as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
    {
        let mut v0: ::core::ffi::c_int = *(&raw mut game.g_ram as *mut uint8).offset(0x5b5)
            as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(102) =
            kEquipmentScreenGlowingArrowPalettes0[v0 as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(107) =
            kEquipmentScreenGlowingArrowPalettes1[v0 as usize];
        EquipmentScreenEnergyArrowGlow_On(game);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(107) =
            926 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(102) =
            342 as uint16;
        EquipmentScreenEnergyArrowGlow_Off(game);
    };
}
pub unsafe fn EquipmentScreenGlowingArrowSolidOn(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(107) =
        926 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(102) =
        342 as uint16;
    EquipmentScreenEnergyArrowGlow_On(game);
}
pub unsafe fn EquipmentScreenGlowingArrowSolidOff(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(107) =
        926 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(102) =
        342 as uint16;
    EquipmentScreenEnergyArrowGlow_Off(game);
}
pub unsafe fn EquipmentScreenEnergyArrowGlow_On(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
            .equipment_screen_bg1_tilemap
            [(129 as ::core::ffi::c_int + i * 32 as ::core::ffi::c_int) as usize] =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
                .equipment_screen_bg1_tilemap
                [(129 as ::core::ffi::c_int + i * 32 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int
                & !(0x1c00 as ::core::ffi::c_int)
                | 0x1800 as ::core::ffi::c_int) as uint16;
        i += 1;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < 2 as ::core::ffi::c_int {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
            .equipment_screen_bg1_tilemap[(385 as ::core::ffi::c_int + i_0) as usize] =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
                .equipment_screen_bg1_tilemap[(385 as ::core::ffi::c_int + i_0) as usize]
                as ::core::ffi::c_int
                & !(0x1c00 as ::core::ffi::c_int)
                | 0x1800 as ::core::ffi::c_int) as uint16;
        i_0 += 1;
    }
}
pub unsafe fn EquipmentScreenEnergyArrowGlow_Off(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
            .equipment_screen_bg1_tilemap
            [(129 as ::core::ffi::c_int + i * 32 as ::core::ffi::c_int) as usize] =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
                .equipment_screen_bg1_tilemap
                [(129 as ::core::ffi::c_int + i * 32 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int
                & !(0x1c00 as ::core::ffi::c_int)
                | 0x1c00 as ::core::ffi::c_int) as uint16;
        i += 1;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < 2 as ::core::ffi::c_int {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
            .equipment_screen_bg1_tilemap[(385 as ::core::ffi::c_int + i_0) as usize] =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
                .equipment_screen_bg1_tilemap[(385 as ::core::ffi::c_int + i_0) as usize]
                as ::core::ffi::c_int
                & !(0x1c00 as ::core::ffi::c_int)
                | 0x1c00 as ::core::ffi::c_int) as uint16;
        i_0 += 1;
    }
}
static kEquipmentScreenTilemap_MANUAL: [uint16; 4] = [0x3d46, 0x3d47, 0x3d48, 0x3d49];
static kEquipmentScreenTilemap_AUTO: [uint16; 4] = [0x3d56, 0x3d57, 0x3d58, 0x3d59];
pub unsafe fn EquipmentScreenCategory_Tanks_0(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v2: int16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & kButton_A as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9d4) as *mut uint16) as ::core::ffi::c_int
            != 0
    {
        QueueSfx1_Max6(0x37 as uint16, game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9c0) as *mut uint16) as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x9c0) as *mut uint16) = 2 as uint16;
            EquipmentScreenHudReserveAutoTilemap_Off(game);
            v0 = 8 as int16;
            let mut v1: uint16 = 0 as uint16;
            loop {
                (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
                    .cinematic_bg_tilemap
                    [(v1 as ::core::ffi::c_int + 327 as ::core::ffi::c_int) as usize] =
                    (kEquipmentScreenTilemap_MANUAL[v1 as usize] as ::core::ffi::c_int
                        | (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
                            .cinematic_bg_tilemap
                            [(v1 as ::core::ffi::c_int + 327 as ::core::ffi::c_int) as usize]
                            as ::core::ffi::c_int
                            & 0xfc00 as ::core::ffi::c_int) as uint16;
                v1 = v1.wrapping_add(1);
                v0 = ((v0).wrapping_sub((2i16)));
                if !(v0 != 0) {
                    break;
                }
            }
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x9c0) as *mut uint16) = 1 as uint16;
            EquipmentScreenHudReserveAutoTilemap_On_BUGGY(game);
            v2 = 8 as int16;
            let mut v3: uint16 = 0 as uint16;
            loop {
                (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
                    .cinematic_bg_tilemap
                    [(v3 as ::core::ffi::c_int + 327 as ::core::ffi::c_int) as usize] =
                    (kEquipmentScreenTilemap_AUTO[v3 as usize] as ::core::ffi::c_int
                        | (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
                            .cinematic_bg_tilemap
                            [(v3 as ::core::ffi::c_int + 327 as ::core::ffi::c_int) as usize]
                            as ::core::ffi::c_int
                            & 0xfc00 as ::core::ffi::c_int) as uint16;
                v3 = v3.wrapping_add(1);
                v2 = ((v2).wrapping_sub((2i16)));
                if !(v2 != 0) {
                    break;
                }
            }
        }
    }
}
pub unsafe fn EquipmentScreenHudReserveAutoTilemap_On_BUGGY(game: &mut Game) {
    let mut v0: uint16 = -(26229 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16) == 0 {
        v0 = -(26217 as ::core::ffi::c_int) as uint16;
    }
    let mut v1: *const uint16 = RomPtr_82(v0, game) as *const uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(8) = *v1;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(9) = *v1.offset(1);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(40) = *v1.offset(2);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(41) = *v1.offset(3);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(72) = *v1.offset(4);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(73) = *v1.offset(5);
}
pub unsafe fn EquipmentScreenHudReserveAutoTilemap_Off(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(8) =
        11279 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(9) =
        11279 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(40) =
        11279 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(41) =
        11279 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(72) =
        11279 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc608) as *mut uint16).offset(73) =
        11279 as uint16;
}
static kReserveTankEnergyTransferPerFrame: uint16 = 1 as uint16;
pub unsafe fn EquipmentScreenCategory_Tanks_1(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x757) as *mut uint16) == 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & kButton_A as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            return;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x757) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16)
                as ::core::ffi::c_int
                + 7 as ::core::ffi::c_int
                & 0xfff8 as ::core::ffi::c_int) as uint16;
    }
    let ref mut fresh55 = *((&raw mut game.g_ram as *mut uint8).offset(0x757) as *mut uint16);
    *fresh55 = (*fresh55).wrapping_sub(1);
    if *fresh55 as ::core::ffi::c_int & 7 as ::core::ffi::c_int == 7 as ::core::ffi::c_int {
        QueueSfx3_Max6(0x2d as uint16, game);
    }
    let ref mut fresh56 = *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16);
    *fresh56 = ((*fresh56).wrapping_add((kReserveTankEnergyTransferPerFrame)));
    if ((((*((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16)) as i16)
        .wrapping_sub(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x9c4) as *mut uint16)) as i16),
        )) as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        let ref mut fresh57 = *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16);
        *fresh57 = ((*fresh57).wrapping_sub((kReserveTankEnergyTransferPerFrame)));
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16) as int16
            as ::core::ffi::c_int
            > 0 as ::core::ffi::c_int
        {
            return;
        }
        let ref mut fresh58 = *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16);
        *fresh58 = ((*fresh58)
            .wrapping_add((*((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16))));
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x9c4) as *mut uint16);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x757) as *mut uint16) = 0 as uint16;
    EquipmentScreenEnergyArrowGlow_Off(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) = 0 as uint16;
}
pub unsafe fn EquipmentScreenCategory_Weapons(game: &mut Game) {
    EquipmentScreenCategory_Weapons_MoveButtons(game);
    let mut R36: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x9a6) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a8) as *mut uint16) != 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) as uint8
            as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
        {
            EquipmentScreenCategory_ButtonResponse(10 as uint16, game);
            EquipmentScreenCategory_Weapons_PlazmaSpazerCheck(R36, game);
        }
    }
}
pub unsafe fn EquipmentScreenCategory_Weapons_MoveButtons(game: &mut Game) {
    let mut r18: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & 0x100 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & 0x800 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            EquipmentScreenMoveLowerOnSuitsMisc(0 as uint16, game);
        } else if EquipmentScreenMoveLowerOnSuitsMisc(4 as uint16, game) != 0 {
            EquipmentScreenMoveToHighJumpOrLowerInBoots(0 as uint16, r18, game);
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & 0x400 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa76) as *mut uint16) == 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16)
                as ::core::ffi::c_int
                != 1025 as ::core::ffi::c_int
        {
            let ref mut fresh61 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16);
            *fresh61 = ((*fresh61).wrapping_add((256u16)));
            EquipmentScreenMoveToBeams(
                ((2u16).wrapping_mul(
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16
                        as *mut uint8)
                        .offset(
                            (::core::mem::size_of::<uint16>())
                                .wrapping_div(::core::mem::size_of::<uint8>())
                                .wrapping_sub(1 as usize) as isize,
                        )) as u16),
                )),
                r18,
                game,
            );
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & 0x800 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
            || {
                let ref mut fresh62 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16);
                *fresh62 = ((*fresh62).wrapping_sub((256u16)));
                EquipmentScreenMoveToBottomOfBeams(
                    ((2u16).wrapping_mul(
                        ((*((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16
                            as *mut uint8)
                            .offset(
                                (::core::mem::size_of::<uint16>())
                                    .wrapping_div(::core::mem::size_of::<uint8>())
                                    .wrapping_sub(1 as usize)
                                    as isize,
                            )) as u16),
                    )),
                    game,
                ) as ::core::ffi::c_int
                    == 0xffff as ::core::ffi::c_int
            }
        {
            if EquipmentScreenMoveToReserveTanks(game) == 0 {
                *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) = r18;
            }
        }
    }
}
pub unsafe fn EquipmentScreenCategory_Weapons_PlazmaSpazerCheck(mut R36: uint16, game: &mut Game) {
    let mut t: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x9a6)
        as *mut uint16) as ::core::ffi::c_int
        & !(R36 as ::core::ffi::c_int);
    if t & 4 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        if R36 as ::core::ffi::c_int & 4 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0x9a6) as *mut uint16)
                as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
        {
            let ref mut fresh59 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x9a6) as *mut uint16);
            *fresh59 = (*fresh59 as ::core::ffi::c_int & !(8 as ::core::ffi::c_int)) as uint16;
            ChangePaletteValues(
                (&raw mut game.g_ram as *mut uint8).offset(
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c06c)) as *const uint8
                        as *mut uint16)
                        .offset(4) as ::core::ffi::c_int as isize,
                ) as *mut uint16,
                3072 as uint16,
                5 as size_t,
            );
        }
    } else if t & 8 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        && R36 as ::core::ffi::c_int & 8 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9a6) as *mut uint16) as ::core::ffi::c_int
            & 4 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        let ref mut fresh60 = *((&raw mut game.g_ram as *mut uint8).offset(0x9a6) as *mut uint16);
        *fresh60 = (*fresh60 as ::core::ffi::c_int & !(4 as ::core::ffi::c_int)) as uint16;
        ChangePaletteValues(
            (&raw mut game.g_ram as *mut uint8).offset(
                *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c06c)) as *const uint8
                    as *mut uint16)
                    .offset(3) as ::core::ffi::c_int as isize,
            ) as *mut uint16,
            3072 as uint16,
            5 as size_t,
        );
    }
}
pub unsafe fn EquipmentScreenCategory_Suit(game: &mut Game) {
    EquipmentScreenCategory_Suit_MoveResponse(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) as uint8
        as ::core::ffi::c_int
        == 2 as ::core::ffi::c_int
    {
        EquipmentScreenCategory_ButtonResponse(18 as uint16, game);
    }
}
pub unsafe fn EquipmentScreenCategory_Suit_MoveResponse(game: &mut Game) {
    let mut r18: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & kButton_Left as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if EquipmentScreenMoveToReserveTanks(game) != 0 {
                return;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) = r18;
        }
        EquipmentScreenMoveToBeams(0 as uint16, r18, game);
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & kButton_Up as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let ref mut fresh51 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16);
            *fresh51 = ((*fresh51).wrapping_sub((256u16)));
            EquipmentScreenMoveToScrewOrHigherOnSuits(
                ((2u16).wrapping_mul(
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16
                        as *mut uint8)
                        .offset(
                            (::core::mem::size_of::<uint16>())
                                .wrapping_div(::core::mem::size_of::<uint8>())
                                .wrapping_sub(1 as usize) as isize,
                        )) as u16),
                )),
                r18,
                game,
            );
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & kButton_Down as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int
            == 1280 as ::core::ffi::c_int
            || {
                let ref mut fresh52 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16);
                *fresh52 = ((*fresh52).wrapping_add((256u16)));
                EquipmentScreenMoveLowerOnSuitsMisc(
                    ((2u16).wrapping_mul(
                        ((*((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16
                            as *mut uint8)
                            .offset(
                                (::core::mem::size_of::<uint16>())
                                    .wrapping_div(::core::mem::size_of::<uint8>())
                                    .wrapping_sub(1 as usize)
                                    as isize,
                            )) as u16),
                    )),
                    game,
                ) as ::core::ffi::c_int
                    == 0xffff as ::core::ffi::c_int
            }
        {
            EquipmentScreenMoveToHighJumpOrLowerInBoots(0 as uint16, r18, game);
        }
    }
}
pub unsafe fn EquipmentScreenCategory_Boots(game: &mut Game) {
    EquipmentScreenCategory_Boots_MoveResponse(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) as uint8
        as ::core::ffi::c_int
        == 3 as ::core::ffi::c_int
    {
        EquipmentScreenCategory_ButtonResponse(18 as uint16, game);
    }
}
pub unsafe fn EquipmentScreenCategory_Boots_MoveResponse(game: &mut Game) {
    let mut r18: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & kButton_Left as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
            as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            || EquipmentScreenMoveToBottomOfBeams(8 as uint16, game) as ::core::ffi::c_int
                == 0xffff as ::core::ffi::c_int)
            && EquipmentScreenMoveToReserveTanks(game) == 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) = r18;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & kButton_Down as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) as ::core::ffi::c_int
            != 515 as ::core::ffi::c_int
        {
            let ref mut fresh49 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16);
            *fresh49 = ((*fresh49).wrapping_add((256u16)));
            EquipmentScreenMoveToHighJumpOrLowerInBoots(
                ((2u16).wrapping_mul(
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16
                        as *mut uint8)
                        .offset(
                            (::core::mem::size_of::<uint16>())
                                .wrapping_div(::core::mem::size_of::<uint8>())
                                .wrapping_sub(1 as usize) as isize,
                        )) as u16),
                )),
                r18,
                game,
            );
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & kButton_Up as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
            || {
                let ref mut fresh50 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16);
                *fresh50 = ((*fresh50).wrapping_sub((256u16)));
                EquipmentScreenCategory_Boots_MoveUpInBoots(
                    ((2u16).wrapping_mul(
                        ((*((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16
                            as *mut uint8)
                            .offset(
                                (::core::mem::size_of::<uint16>())
                                    .wrapping_div(::core::mem::size_of::<uint8>())
                                    .wrapping_sub(1 as usize)
                                    as isize,
                            )) as u16),
                    )),
                    game,
                ) as ::core::ffi::c_int
                    == 0xffff as ::core::ffi::c_int
            }
        {
            EquipmentScreenMoveToScrewOrHigherOnSuits(0xa as uint16, r18, game);
        }
    }
}
pub unsafe fn WriteSamusWireframeTilemapAndQueue(game: &mut Game) {
    WriteSamusWireframeTilemap(game);
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    (*gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .size = 1280 as uint16;
    v0 = ((v0).wrapping_add((2u16)));
    (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size =
        (((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            + 256 as ::core::ffi::c_long) as uint16;
    v0 = ((v0).wrapping_add((2u16)));
    let fresh63 = v0;
    v0 = v0.wrapping_add(1);
    *(&raw mut (*(gVramWriteEntry
        as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
        fresh63 as ::core::ffi::c_int,
        game,
    ))
    .size as *mut uint8)
        .offset(0) = 126 as uint8;
    (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size = 0x3080 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        ((v0).wrapping_add((2u16)));
}
pub unsafe fn WriteSamusWireframeTilemap(game: &mut Game) {
    let mut i: uint16 = 0;
    i = 0 as uint16;
    while *((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16) as ::core::ffi::c_int
        & 0x101 as ::core::ffi::c_int
        != *(game.g_rom.offset(crate::rtl::lorom_offset(0x82b257)) as *const uint8 as *mut uint16)
            .offset((i as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
    {
        i = ((i).wrapping_add((2u16)));
    }
    let mut src: *const uint16 = RomPtr_82(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x82b25f)) as *const uint8 as *mut uint16)
            .offset((i as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
        game,
    ) as *const uint16;
    let mut m: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
    let mut v1: uint16 = 0 as uint16;
    let mut v2: uint16 = 472 as uint16;
    loop {
        let mut v3: uint16 = v2;
        let mut n: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
        loop {
            let fresh64 = src;
            src = src.offset(1);
            *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
                .cinematic_bg_tilemap as *mut uint16 as *mut uint8)
                .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) = *fresh64;
            v2 = ((v2).wrapping_add((2u16)));
            n -= 1;
            if !(n != 0) {
                break;
            }
        }
        v2 = ((v3).wrapping_add((64u16)));
        m -= 1;
        if !(m != 0) {
            break;
        }
    }
}
pub unsafe fn EquipmentScreenDrawItemSelector(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9d4) as *mut uint16) as ::core::ffi::c_int
        | ((*((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16))
            | (*((&raw mut game.g_ram as *mut uint8).offset(0x9a8) as *mut uint16)))
            as ::core::ffi::c_int
        != 0
    {
        let mut t: *mut uint16 = RomPtr_82(
            (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c18e)) as *const uint8 as *mut uint16)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) as uint8
                        as isize,
                ) as ::core::ffi::c_int
                + 4 as ::core::ffi::c_int
                    * *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16
                        as *mut uint8)
                        .offset(
                            (::core::mem::size_of::<uint16>())
                                .wrapping_div(::core::mem::size_of::<uint8>())
                                .wrapping_sub(1 as usize) as isize,
                        ) as ::core::ffi::c_int) as uint16_t,
            game,
        ) as *mut uint16;
        DrawPauseScreenSpriteAnim(
            3 as uint16,
            ((*t.offset(0)).wrapping_sub((1u16))),
            *t.offset(1),
            game,
        );
    }
}
pub unsafe fn EquipmentScreenDisplayReserveTankAmount(game: &mut Game) {
    EquipmentScreenDisplayReserveTankAmount_(game);
}
static kEquipmentScreenReserveTank_X: [uint16; 6] = [0x18, 0x20, 0x28, 0x30, 0x38, 0x40];
static kEquipmentScreenReserveTank_Y: uint16 = 0x60 as uint16;
static kPartialReserveTankSpritemapIds: [uint16; 16] = [
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27,
];
pub unsafe fn EquipmentScreenDisplayReserveTankAmount_(game: &mut Game) {
    let mut r3: uint16 = EquipmentScreenDisplayReserves_PaletteSetup(game);
    let mut r52: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9d4) as *mut uint16) == 0 {
        return;
    }
    let mut R44: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x9d4) as *mut uint16)
        as ::core::ffi::c_int
        / 100 as ::core::ffi::c_int) as uint16;
    let mut mod_value: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x9d6)
        as *mut uint16) as ::core::ffi::c_int
        % 100 as ::core::ffi::c_int;
    let mut R50: uint16 = mod_value as uint16;
    let mut r42: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x9d6)
        as *mut uint16) as ::core::ffi::c_int
        / 100 as ::core::ffi::c_int;
    let mut R48: uint16 = r42 as uint16;
    let mut r46: ::core::ffi::c_int = r42;
    if r42 != 0 {
        let mut v0: uint16 = 0 as uint16;
        loop {
            DrawMenuSpritemap(
                0x1b as uint16,
                kEquipmentScreenReserveTank_X
                    [(v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize],
                ((kEquipmentScreenReserveTank_Y).wrapping_sub((1u16))),
                r3,
                game,
            );
            v0 = ((v0).wrapping_add((2u16)));
            r46 -= 1;
            if !(r46 != 0) {
                break;
            }
        }
        r52 = v0 as ::core::ffi::c_int;
    }
    let mut RegWord: uint16 = mod_value as uint16;
    if RegWord != 0 {
        let mut v2: uint16 = (2 as ::core::ffi::c_int
            * (RegWord as ::core::ffi::c_int / 14 as ::core::ffi::c_int))
            as uint16;
        if v2 as ::core::ffi::c_int - 7 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
            && RegWord as ::core::ffi::c_int % 14 as ::core::ffi::c_int != 0
            && *(&raw mut game.g_ram as *mut uint8).offset(0x5b5) as ::core::ffi::c_int
                & 4 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            v2 = ((v2).wrapping_add((2u16)));
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16) as ::core::ffi::c_int
            - 100 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v2 = ((v2).wrapping_add((16u16)));
        }
        DrawMenuSpritemap(
            kPartialReserveTankSpritemapIds
                [(v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize],
            kEquipmentScreenReserveTank_X[(r52 >> 1 as ::core::ffi::c_int) as usize],
            ((kEquipmentScreenReserveTank_Y).wrapping_sub((1u16))),
            r3,
            game,
        );
        R48 = R48.wrapping_add(1);
        r52 += 2 as ::core::ffi::c_int;
    }
    while R48 as ::core::ffi::c_int - R44 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    {
        DrawMenuSpritemap(
            0x20 as uint16,
            kEquipmentScreenReserveTank_X[(r52 >> 1 as ::core::ffi::c_int) as usize],
            ((kEquipmentScreenReserveTank_Y).wrapping_sub((1u16))),
            r3,
            game,
        );
        r52 += 2 as ::core::ffi::c_int;
        R48 = R48.wrapping_add(1);
    }
    DrawMenuSpritemap(
        0x1f as uint16,
        kEquipmentScreenReserveTank_X[(r52 >> 1 as ::core::ffi::c_int) as usize],
        ((kEquipmentScreenReserveTank_Y).wrapping_sub((1u16))),
        r3,
        game,
    );
    let mut div_val: ::core::ffi::c_int = R50 as ::core::ffi::c_int / 10 as ::core::ffi::c_int;
    let mut mod_val: ::core::ffi::c_int = R50 as ::core::ffi::c_int % 10 as ::core::ffi::c_int;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800)).cinematic_bg_tilemap
        [394 as ::core::ffi::c_int as usize] = (mod_val + 2052 as ::core::ffi::c_int) as uint16;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800)).cinematic_bg_tilemap
        [393 as ::core::ffi::c_int as usize] = (div_val + 2052 as ::core::ffi::c_int) as uint16;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800)).cinematic_bg_tilemap
        [392 as ::core::ffi::c_int as usize] = (r42 + 2052 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EquipmentScreenDisplayReserves_PaletteSetup(game: &mut Game) -> uint16 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16) != 0 {
        let ref mut fresh40 = *((&raw mut game.g_ram as *mut uint8).offset(0x72f) as *mut uint16);
        *fresh40 = (*fresh40).wrapping_sub(1);
        let mut v0: bool = *fresh40 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x72f) as *mut uint16) == 0 || v0 {
            let ref mut fresh41 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x743) as *mut uint16);
            *fresh41 = (*fresh41).wrapping_add(1);
            let mut v1: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c165))
                as *const uint8 as *mut uint8)
                .offset(
                    (2 as ::core::ffi::c_int
                        * *((&raw mut game.g_ram as *mut uint8).offset(0x743) as *mut uint16)
                            as ::core::ffi::c_int) as isize,
                ) as uint16;
            if v1 as ::core::ffi::c_int == 255 as ::core::ffi::c_int {
                *((&raw mut game.g_ram as *mut uint8).offset(0x743) as *mut uint16) = 0 as uint16;
                v1 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c165)) as *const uint8
                    as *mut uint8)
                    .offset(0) as uint16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x72f) as *mut uint16) = v1;
        }
        return *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c0fa)) as *const uint8
            as *mut uint16)
            .offset(3);
    } else {
        return 1536 as uint16;
    };
}
pub unsafe fn EquipmentScreenMoveToReserveTanks(game: &mut Game) -> uint16 {
    let mut result: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x9d4) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9d4) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) = 0 as uint16;
        QueueSfx1_Max6(0x37 as uint16, game);
        return 1 as uint16;
    }
    return result;
}
pub unsafe fn EquipmentScreenMoveToBeams(mut v0: uint16, mut r18: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    let mut current_block_5: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa76) as *mut uint16) != 0 {
        current_block_5 = 16475780731721606008;
    } else {
        loop {
            if !(*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c04c)) as *const uint8
                as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x9a8) as *mut uint16)
                    as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int)
            {
                current_block_5 = 6873731126896040597;
                break;
            }
            v0 = ((v0).wrapping_add((2u16)));
            if (((v0) as i16).wrapping_sub((10i16))) as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                current_block_5 = 16475780731721606008;
                break;
            }
        }
        match current_block_5 {
            16475780731721606008 => {}
            _ => {
                *(&raw mut v1 as *mut uint8).offset(0) =
                    ((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16
                        as ::core::ffi::c_int
                        >> 8 as ::core::ffi::c_int) as uint8;
                *(&raw mut v1 as *mut uint8).offset(
                    (::core::mem::size_of::<int16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1 as usize) as isize,
                ) = (v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint8;
                *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) =
                    (v1 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                        | 1 as ::core::ffi::c_int) as uint16;
                QueueSfx1_Max6(0x37 as uint16, game);
                current_block_5 = 13513818773234778473;
            }
        }
    }
    match current_block_5 {
        16475780731721606008 => {
            *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) = r18;
        }
        _ => {}
    };
}
pub unsafe fn EquipmentScreenMoveToBottomOfBeams(mut k: uint16, game: &mut Game) -> uint16 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa76) as *mut uint16) != 0 {
        return -(1 as ::core::ffi::c_int) as uint16;
    }
    while *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c04c)) as *const uint8 as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x9a8) as *mut uint16) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        k = ((k).wrapping_sub((2u16)));
        if k as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int) as uint16;
        }
    }
    QueueSfx1_Max6(0x37 as uint16, game);
    let mut result: uint16 = ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int
        | 1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) = result;
    return result;
}
pub unsafe fn EquipmentScreenMoveLowerOnSuitsMisc(mut v0: uint16, game: &mut Game) -> uint16 {
    let mut v1: int16 = 0;
    while *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c056)) as *const uint8 as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v0 = ((v0).wrapping_add((2u16)));
        if (((v0) as i16).wrapping_sub((10i16))) as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int) as uint16;
        }
    }
    QueueSfx1_Max6(0x37 as uint16, game);
    *(&raw mut v1 as *mut uint8).offset(0) = ((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
        as uint16 as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as uint8;
    *(&raw mut v1 as *mut uint8).offset(
        (::core::mem::size_of::<int16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) = (v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) =
        (v1 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int | 2 as ::core::ffi::c_int)
            as uint16;
    return 0 as uint16;
}
pub unsafe fn EquipmentScreenMoveToScrewOrHigherOnSuits(
    mut v0: uint16,
    mut r18: uint16,
    game: &mut Game,
) {
    while *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c056)) as *const uint8 as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v0 = ((v0).wrapping_sub((2u16)));
        if v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) = r18;
            return;
        }
    }
    QueueSfx1_Max6(0x37 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) =
        ((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EquipmentScreenMoveToHighJumpOrLowerInBoots(
    mut v0: uint16,
    mut r18: uint16,
    game: &mut Game,
) {
    while *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c062)) as *const uint8 as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v0 = ((v0).wrapping_add((2u16)));
        if (((v0) as i16).wrapping_sub((6i16))) as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) = r18;
            return;
        }
    }
    QueueSfx1_Max6(0x37 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) =
        ((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EquipmentScreenCategory_Boots_MoveUpInBoots(
    mut k: uint16,
    game: &mut Game,
) -> uint16 {
    while *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c062)) as *const uint8 as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        k = ((k).wrapping_sub((2u16)));
        if k as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int) as uint16;
        }
    }
    QueueSfx1_Max6(0x37 as uint16, game);
    let mut result: uint16 = ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int
        | 3 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x755) as *mut uint16) = result;
    return result;
}
pub unsafe fn EquipmentScreenCategory_ButtonResponse(mut r24: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & kButton_A as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x38 as uint16, game);
        let mut item: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x755)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int;
        let mut category: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x755)
            as *mut uint16) as uint8
            as ::core::ffi::c_int;
        let mut target: *mut uint8 = (&raw mut game.g_ram as *mut uint8).offset(
            *(RomPtr_82(
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c02c)) as *const uint8
                    as *mut uint16)
                    .offset(category as isize) as ::core::ffi::c_int
                    + item * 2 as ::core::ffi::c_int) as uint16_t,
                game,
            ) as *mut uint16) as ::core::ffi::c_int as isize,
        );
        let mut var: *mut uint16 = RomPtr_RAM(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c03c)) as *const uint8 as *mut uint16)
                .offset(category as isize),
            game,
        ) as *mut uint16;
        let mut mask: uint16 = *(RomPtr_82(
            (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c034)) as *const uint8 as *mut uint16)
                .offset(category as isize) as ::core::ffi::c_int
                + item * 2 as ::core::ffi::c_int) as uint16_t,
            game,
        ) as *mut uint16);
        if *var as ::core::ffi::c_int & mask as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            *var = (*var as ::core::ffi::c_int & !(mask as ::core::ffi::c_int)) as uint16;
            ChangePaletteValues(
                target as *mut uint16,
                0xc00 as uint16,
                (r24 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as size_t,
            );
        } else {
            *var = ((*var) | (mask));
            let mut src: uint16 = *(RomPtr_82(
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c044)) as *const uint8
                    as *mut uint16)
                    .offset(category as isize) as ::core::ffi::c_int
                    + item * 2 as ::core::ffi::c_int) as uint16_t,
                game,
            ) as *mut uint16);
            memcpy(
                target as *mut ::core::ffi::c_void,
                RomPtr_82(src, game) as *const ::core::ffi::c_void,
                r24 as size_t,
            );
        }
    }
}
pub unsafe fn DrawPauseMenuDuringFadeIn(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) != 0 {
        EquipmentScreenDrawItemSelector(game);
        EquipmentScreenDisplayReserveTankAmount(game);
    } else {
        MapScreenDrawSamusPositionIndicator(game);
        DrawMapIcons(game);
        DisplayMapElevatorDestinations(game);
    };
}
pub unsafe fn UNKNOWN_sub_82B650(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x753) as *mut uint16) != 0 {
        DisplayMapElevatorDestinations(game);
        MapScreenDrawSamusPositionIndicator(game);
        DrawMapIcons(game);
    } else {
        EquipmentScreenDrawItemSelector(game);
        EquipmentScreenDisplayReserveTankAmount(game);
    };
}
static kMap_Criteria_SavePoints: [uint16; 16] = [
    0xd8, 0x28, 0x90, 0x38, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe, 0xfffe,
    0xfffe, 0xfffe, 0xfffe,
];
pub unsafe fn DrawMapIcons(game: &mut Game) {
    DrawBossMapIcons(9 as uint16, addr_kMapIconDataPointers as uint16, game);
    DrawSimpleMapIcons(
        0xb as uint16,
        (addr_kMapIconDataPointers as ::core::ffi::c_int + 0x10 as ::core::ffi::c_int) as uint16,
        3584 as uint16,
        game,
    );
    DrawSimpleMapIcons(
        0xa as uint16,
        (addr_kMapIconDataPointers as ::core::ffi::c_int + 0x20 as ::core::ffi::c_int) as uint16,
        3584 as uint16,
        game,
    );
    DrawSimpleMapIcons(
        0x4e as uint16,
        (addr_kMapIconDataPointers as ::core::ffi::c_int + 0x30 as ::core::ffi::c_int) as uint16,
        3584 as uint16,
        game,
    );
    DrawSaveStationMapIcon(8 as uint16, 0xc80b as uint16, 1024 as uint16, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5d1) as *mut uint16) != 0 {
        DrawSimpleMapIcons(8 as uint16, 0xc82b as uint16, 1024 as uint16, game);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) == 0 {
        DrawMenuSpritemap(
            0x63 as uint16,
            ((kMap_Criteria_SavePoints[0 as ::core::ffi::c_int as usize]).wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16)),
            )),
            ((kMap_Criteria_SavePoints[1 as ::core::ffi::c_int as usize]).wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16)),
            )),
            3584 as uint16,
            game,
        );
    }
}
pub unsafe fn DrawFileSelectMapIcons(game: &mut Game) {
    HandlePauseScreenPaletteAnimation(game);
    DrawBossMapIcons(9 as uint16, 0xc7cb as uint16, game);
    DrawSimpleMapIcons(0xb as uint16, 0xc7db as uint16, 3584 as uint16, game);
    DrawSimpleMapIcons(0xa as uint16, 0xc7eb as uint16, 3584 as uint16, game);
    DrawSimpleMapIcons(0x4e as uint16, 0xc7fb as uint16, 3584 as uint16, game);
    let mut a: uint16 = UpdateSamusPositionIndicatorAnimation(game);
    let mut data: *const uint16 = (RomPtr_82(
        *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82c7cb)) as *const uint8
            as *mut MapIconDataPointers)
            .offset(4))
        .crateria as *mut uint8)
            .offset(
                (2 as ::core::ffi::c_int
                    * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                        as ::core::ffi::c_int) as isize,
            ) as *mut VoidP),
        game,
    ) as *mut uint16)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16)
                as ::core::ffi::c_int
                * 2 as ::core::ffi::c_int) as isize,
        );
    let mut v1: uint16 = ((*data.offset(0))
        .wrapping_sub((*((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16))));
    let mut v2: uint16 = ((*data.offset(1))
        .wrapping_sub((*((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16))));
    if *((&raw mut game.g_ram as *mut uint8).offset(0x77a) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        DrawMenuSpritemap(0x12 as uint16, v1, v2, 3584 as uint16, game);
    }
    DrawMenuSpritemap(a, v1, v2, 3584 as uint16, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5d1) as *mut uint16) != 0 {
        DrawDebugSaveMapIcons(0xc as uint16, 0xc80b as uint16, 1536 as uint16, game);
        DrawDebugElevatorMapIcons(0x17 as uint16, 0xc81b as uint16, 1536 as uint16, game);
        DrawSimpleMapIcons(0xc as uint16, 0xc82b as uint16, 1536 as uint16, game);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) == 0 {
        DrawMenuSpritemap(
            0x63 as uint16,
            ((kMap_Criteria_SavePoints[0 as ::core::ffi::c_int as usize]).wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16)),
            )),
            ((kMap_Criteria_SavePoints[1 as ::core::ffi::c_int as usize]).wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16)),
            )),
            3584 as uint16,
            game,
        );
    }
}
pub unsafe fn DrawSaveStationMapIcon(
    mut a: uint16,
    mut k: uint16,
    mut r3: uint16,
    game: &mut Game,
) {
    static kShlBit: [uint8; 8] = [1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80];
    let mut R34: uint16 = a;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as ::core::ffi::c_int
        == *((&raw mut game.g_ram as *mut uint8).offset(0xd918) as *mut uint16)
            as ::core::ffi::c_int
    {
        let mut R36: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xd8f8).offset(
            (2 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                    as ::core::ffi::c_int) as isize,
        ) as *mut uint8 as *mut uint16);
        *(&raw mut R36 as *mut uint8).offset(0) = (((R36) as u8)
            & (kShlBit
                [*((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16) as usize]));
        let mut v2: uint16 = *(RomPtr_82(
            (k as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int
                    * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                        as ::core::ffi::c_int) as uint16_t,
            game,
        ) as *mut uint16);
        if v2 != 0 {
            DrawMapIconsOfType(v2, R34, R36, r3, game);
        }
    }
}
pub unsafe fn DrawDebugSaveMapIcons(mut a: uint16, mut k: uint16, mut r3: uint16, game: &mut Game) {
    let mut R34: uint16 = a;
    let mut R36: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xd8f8).offset(
        (2 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                as ::core::ffi::c_int) as isize,
    ) as *mut uint8 as *mut uint16);
    let mut v2: uint16 = *(RomPtr_82(
        (k as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                    as ::core::ffi::c_int) as uint16_t,
        game,
    ) as *mut uint16);
    if v2 != 0 {
        DrawMapIconsOfType(v2, R34, R36, r3, game);
    }
}
pub unsafe fn DrawDebugElevatorMapIcons(
    mut a: uint16,
    mut k: uint16,
    mut r3: uint16,
    game: &mut Game,
) {
    let mut R34: uint16 = a;
    let mut R36: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xd8f8).offset(
        (2 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as isize,
    ) as *mut uint8 as *mut uint16);
    let mut v2: uint16 = *(RomPtr_82(
        (k as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                    as ::core::ffi::c_int) as uint16_t,
        game,
    ) as *mut uint16);
    if v2 != 0 {
        DrawMapIconsOfType(v2, R34, R36, r3, game);
    }
}
pub unsafe fn DrawSimpleMapIcons(mut a: uint16, mut k: uint16, mut r3: uint16, game: &mut Game) {
    let mut R34: uint16 = a;
    let mut v2: uint16 = *(RomPtr_82(
        (k as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                    as ::core::ffi::c_int) as uint16_t,
        game,
    ) as *mut uint16);
    if v2 != 0 {
        DrawMapIconsOfType(v2, R34, -(1 as ::core::ffi::c_int) as uint16, r3, game);
    }
}
pub unsafe fn DrawMapIconsOfType(
    mut a: uint16,
    mut r34: uint16,
    mut r36: uint16,
    mut r3: uint16,
    game: &mut Game,
) {
    let mut v3: int8 = 0;
    let mut v4: int16 = 0;
    loop {
        let mut v2: *const uint16 = RomPtr_82(a, game) as *const uint16;
        if *v2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            break;
        }
        v3 = (((r36) as i8) & (1i8));
        r36 = (r36 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        if v3 != 0 {
            v4 = CheckIfMapPositionIsExplored(*v2.offset(0), *v2.offset(1), game) as int16;
            if v4 != 0 {
                DrawMenuSpritemap(
                    r34,
                    ((*v2.offset(0)).wrapping_sub(
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16)),
                    )),
                    ((*v2.offset(1)).wrapping_sub(
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16)),
                    )),
                    r3,
                    game,
                );
            }
        }
        a = ((a).wrapping_add((4u16)));
    }
}
pub unsafe fn CheckIfMapPositionIsExplored(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    static kBits0x80Shr: [uint8; 8] = [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1];
    let mut r18: ::core::ffi::c_int =
        ((k) & (0xff00u16)) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    r18 += k as uint8 as ::core::ffi::c_int >> 6 as ::core::ffi::c_int;
    r18 += ((j) & (0xfff8u16)) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v3: uint16 =
        (k as ::core::ffi::c_int >> 3 as ::core::ffi::c_int & 7 as ::core::ffi::c_int) as uint16;
    return (kBits0x80Shr[v3 as usize] as ::core::ffi::c_int
        & *(&raw mut game.g_ram as *mut uint8)
            .offset(0x7f7)
            .offset(r18 as isize) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn DrawBossMapIcons(mut a: uint16, mut k: uint16, game: &mut Game) {
    let mut bits: ::core::ffi::c_int = *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        as ::core::ffi::c_int;
    let mut t: ::core::ffi::c_int = *(RomPtr_82(
        (k as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                    as ::core::ffi::c_int) as uint16_t,
        game,
    ) as *mut uint16) as ::core::ffi::c_int;
    if t == 0 as ::core::ffi::c_int {
        return;
    }
    let mut v4: *const uint16 = RomPtr_82(t as uint16_t, game) as *const uint16;
    let mut current_block_8: u64;
    loop {
        if *v4.offset(0) as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
            return;
        }
        if *v4.offset(0) as ::core::ffi::c_int != 0xfffe as ::core::ffi::c_int {
            let mut v5: ::core::ffi::c_int = bits & 1 as ::core::ffi::c_int;
            bits >>= 1 as ::core::ffi::c_int;
            if v5 != 0 {
                DrawMenuSpritemap(
                    0x62 as uint16,
                    ((*v4.offset(0)).wrapping_sub(
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16)),
                    )),
                    ((*v4.offset(1)).wrapping_sub(
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16)),
                    )),
                    3584 as uint16,
                    game,
                );
                DrawMenuSpritemap(
                    a,
                    ((*v4.offset(0)).wrapping_sub(
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16)),
                    )),
                    ((*v4.offset(1)).wrapping_sub(
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16)),
                    )),
                    3072 as uint16,
                    game,
                );
                current_block_8 = 12675440807659640239;
            } else if *((&raw mut game.g_ram as *mut uint8).offset(0x789) as *mut uint16) != 0 {
                DrawMenuSpritemap(
                    a,
                    ((*v4.offset(0)).wrapping_sub(
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16)),
                    )),
                    ((*v4.offset(1)).wrapping_sub(
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16)),
                    )),
                    3584 as uint16,
                    game,
                );
                current_block_8 = 12675440807659640239;
            } else {
                current_block_8 = 7651349459974463963;
            }
        } else {
            current_block_8 = 7651349459974463963;
        }
        match current_block_8 {
            7651349459974463963 => {
                bits >>= 1 as ::core::ffi::c_int;
            }
            _ => {}
        }
        v4 = v4.offset(2);
    }
}
pub unsafe fn DrawMapScrollArrowAndCheckToScroll(mut db: uint8, mut k: uint16, game: &mut Game) {
    let mut v1: *const uint16 = RomPtrWithBank(db, k, game) as *const uint16;
    DrawPauseScreenSpriteAnim(*v1.offset(2), *v1, *v1.offset(1), game);
    let mut v2: *const uint8 = RomPtrWithBank(db, k, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int
        & *(v2.offset(6) as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x5fd) as *mut uint16) == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x5fd) as *mut uint16) =
            *(v2.offset(8) as *mut uint16);
    }
}
pub unsafe fn sub_82B932(game: &mut Game) {
    HandleMapScrollArrows(game);
}
pub unsafe fn HandleMapScrollArrows(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5ac) as *mut uint16) as ::core::ffi::c_int
        - 24 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        DrawMapScrollArrowAndCheckToScroll(0x82 as uint8, addr_stru_82B9A0 as uint16, game);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5ae) as *mut uint16) as ::core::ffi::c_int
        - 232 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        DrawMapScrollArrowAndCheckToScroll(0x82 as uint8, addr_stru_82B9AA as uint16, game);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5b0) as *mut uint16) as ::core::ffi::c_int
        - 56 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        DrawMapScrollArrowAndCheckToScroll(0x82 as uint8, addr_stru_82B9B4 as uint16, game);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5b2) as *mut uint16) as ::core::ffi::c_int
        - 177 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5fd) as *mut uint16) as ::core::ffi::c_int
            == (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82b9be)) as *const uint8
                as *mut MapScrollArrowData))
                .map_scroll_dir as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x5fb) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x5fd) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x5ff) as *mut uint16) = 0 as uint16;
        }
    } else {
        DrawMapScrollArrowAndCheckToScroll(0x82 as uint8, addr_stru_82B9BE as uint16, game);
    };
}
pub unsafe fn MapScreenDrawSamusPositionIndicator(game: &mut Game) {
    let mut a: uint16 = UpdateSamusPositionIndicatorAnimation(game);
    DrawMenuSpritemap(
        a,
        (8 as ::core::ffi::c_int
            * (*((&raw mut game.g_ram as *mut uint8).offset(0x7a1) as *mut uint16)
                as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1 as usize) as isize,
                    ) as ::core::ffi::c_int)
            - *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16)
                as ::core::ffi::c_int) as uint16,
        (8 as ::core::ffi::c_int
            * (*((&raw mut game.g_ram as *mut uint8).offset(0x7a3) as *mut uint16)
                as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1 as usize) as isize,
                    ) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int)
            - *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16)
                as ::core::ffi::c_int) as uint16,
        3584 as uint16,
        game,
    );
}
static kMap_SamusPositionIndicator_Delays: [uint16; 4] =
    [8 as uint16, 4 as uint16, 8 as uint16, 4 as uint16];
static kMap_SamusPositionIndicator_SpritemapIds: [uint16; 4] = [0x5f, 0x60, 0x61, 0x60];
pub unsafe fn UpdateSamusPositionIndicatorAnimation(game: &mut Game) -> uint16 {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x778) as *mut uint16);
    if v0 == 0 {
        let mut v1: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x776)
            as *mut uint16))
            .wrapping_add((2u16)));
        if (((v1) as i16).wrapping_sub((8i16))) as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            let ref mut fresh2 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x77a) as *mut uint16);
            *fresh2 = (*fresh2).wrapping_add(1);
            v1 = 0 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x776) as *mut uint16) = v1;
        v0 = kMap_SamusPositionIndicator_Delays
            [(v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0x778) as *mut uint16) = v0;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x778) as *mut uint16) =
        ((v0).wrapping_sub((1u16)));
    return kMap_SamusPositionIndicator_SpritemapIds[(*((&raw mut game.g_ram as *mut uint8)
        .offset(0x776) as *mut uint16)
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int) as usize];
}
pub unsafe fn DrawBorderAroundSamusData(game: &mut Game) {
    DrawMenuSpritemap(
        0x48 as uint16,
        0x80 as uint16,
        0x10 as uint16,
        3584 as uint16,
        game,
    );
}
pub unsafe fn DrawBorderAroundDataCopyMode(game: &mut Game) {
    DrawMenuSpritemap(
        0x49 as uint16,
        0x80 as uint16,
        0x10 as uint16,
        3584 as uint16,
        game,
    );
}
pub unsafe fn DrawBorderAroundDataClearMode(game: &mut Game) {
    DrawMenuSpritemap(
        0x4a as uint16,
        0x7c as uint16,
        0x10 as uint16,
        3584 as uint16,
        game,
    );
}
static kDrawMenuSelectionMissile_Enable: [uint16; 4] =
    [8 as uint16, 8 as uint16, 8 as uint16, 8 as uint16];
static kDrawMenuSelectionMissile_SpriteMap: [uint16; 4] = [0x37, 0x36, 0x35, 0x34];
pub unsafe fn DrawMenuSelectionMissile(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) != 0 {
        let ref mut fresh38 = *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16);
        *fresh38 = (*fresh38).wrapping_sub(1);
        if *fresh38 == 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(0) =
                (*(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(0)
                    as *mut uint16 as *mut uint8)
                    .offset(0) as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16) =
                kDrawMenuSelectionMissile_Enable[*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1997)
                    as *mut uint16)
                    .offset(0) as usize];
        }
    }
    DrawMenuSpritemap(
        kDrawMenuSelectionMissile_SpriteMap[*((&raw mut game.g_ram as *mut uint8).offset(0x1997)
            as *mut uint16)
            .offset(0) as usize],
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(5),
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(10),
        3584 as uint16,
        game,
    );
}
pub unsafe fn DrawFileCopyArrow(game: &mut Game) {
    let mut v0: uint16 = 0;
    if (((*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)) as i16)
        .wrapping_sub(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17))
                as i16),
        )) as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        v0 = 3 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17)
                as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v0 = 4 as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
                as ::core::ffi::c_int
                != 1 as ::core::ffi::c_int
            {
                v0 = 5 as uint16;
            }
        }
    } else {
        v0 = 0 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(17)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16)
                as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v0 = 1 as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(16) != 0
            {
                v0 = 2 as uint16;
            }
        }
    }
    DrawMenuSpritemap(
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82bb0c)) as *const uint8
            as *mut FileCopyArrowStuff)
            .offset(v0 as isize))
        .spritemap,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82bb0c)) as *const uint8
            as *mut FileCopyArrowStuff)
            .offset(v0 as isize))
        .xpos,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x82bb0c)) as *const uint8
            as *mut FileCopyArrowStuff)
            .offset(v0 as isize))
        .ypos,
        512 as uint16,
        game,
    );
}
pub unsafe fn DisplayMapElevatorDestinations(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd908)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        != 0
    {
        let mut i: ::core::ffi::c_int = *(game.g_rom.offset(crate::rtl::lorom_offset(0x82c74d))
            as *const uint8 as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
            as ::core::ffi::c_int;
        loop {
            let mut v1: *const uint8 = RomPtr_82(i as uint16_t, game);
            if *(v1 as *mut uint16) as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
                break;
            }
            DrawMenuSpritemap(
                *(v1.offset(4) as *mut uint16),
                ((*(v1 as *mut uint16)).wrapping_sub(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16)),
                )),
                ((*(v1.offset(2) as *mut uint16)).wrapping_sub(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16)),
                )),
                0 as uint16,
                game,
            );
            i += 6 as ::core::ffi::c_int;
        }
    }
}
pub unsafe fn HandleGameOverBabyMetroid(game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .instruction_timer
        != 0
    {
        sub_82BB7F(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .instruction_timer,
            game,
        );
    } else {
        sub_82BBDD(game);
    };
}
pub unsafe fn CallBabyMetroidPlaySfx(mut ea: uint32, game: &mut Game) {
    match ea {
        8567820 => {
            BabyMetroidPlaySfx0x23(game);
            return;
        }
        8567829 => {
            BabyMetroidPlaySfx0x26(game);
            return;
        }
        8567838 => {
            BabyMetroidPlaySfx0x27(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn sub_82BB7F(mut a: uint16, game: &mut Game) {
    let mut current_instruction: uint16 =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .current_instruction;
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .instruction_timer = ((a).wrapping_sub((1u16)));
    if a as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        let mut v2: *const uint8 = RomPtr_82(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .current_instruction,
            game,
        );
        if *(v2.offset(6) as *mut uint16) as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
            sub_82BBDD(game);
        } else if (((*(v2.offset(6) as *mut uint16)) as i16).wrapping_add((1i16)))
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .instruction_timer = *(v2.offset(6) as *mut uint16);
            let fresh18 = &raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0xf78)
                as *mut EnemyData)
                .offset(0))
            .current_instruction;
            fresh18.write_unaligned(((fresh18.read_unaligned()).wrapping_add((6u16))));
            DrawBabyMetroid(((current_instruction).wrapping_add((6u16))), game);
        } else {
            CallBabyMetroidPlaySfx(
                (((*(v2.offset(6) as *mut uint16)) as u32) | (0x820000u32)),
                game,
            );
        }
    } else {
        DrawBabyMetroid(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .current_instruction,
            game,
        );
    };
}
pub unsafe fn sub_82BBDD(game: &mut Game) {
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .current_instruction = addr_kgameOverBabyMetridInstructionList as uint16;
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .instruction_timer = 10 as uint16;
    sub_82BB7F(0xa as uint16, game);
}
pub unsafe fn DrawBabyMetroid(mut k: uint16, game: &mut Game) {
    let mut v1: uint16 = *(RomPtr_82(k, game) as *mut uint16).offset(2);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 32 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 192 as ::core::ffi::c_int) as isize) =
            *(RomPtr_82((v1 as ::core::ffi::c_int + i) as uint16_t, game) as *mut uint16);
        i += 2 as ::core::ffi::c_int;
    }
    let mut v3: *const uint8 = RomPtr_82(k, game);
    DrawMenuSpritemap(
        *(v3.offset(2) as *mut uint16),
        0x7c as uint16,
        0x50 as uint16,
        2048 as uint16,
        game,
    );
    DrawMenuSpritemap(
        0x64 as uint16,
        0x7c as uint16,
        0x50 as uint16,
        2560 as uint16,
        game,
    );
}
pub unsafe fn FinishProcessingGameOverBabyMetroidAsm(game: &mut Game) {
    let mut t: uint16 = *(RomPtr_82(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .current_instruction,
        game,
    ) as *mut uint16)
        .offset(4);
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .instruction_timer = t;
    let fresh17 = &raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
        .offset(0))
    .current_instruction;
    fresh17.write_unaligned(((fresh17.read_unaligned()).wrapping_add((8u16))));
    if t as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
        sub_82BBDD(game);
    } else {
        DrawBabyMetroid(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .current_instruction,
            game,
        );
    };
}
pub unsafe fn BabyMetroidPlaySfx0x23(game: &mut Game) {
    QueueSfx3_Max6(0x23 as uint16, game);
    FinishProcessingGameOverBabyMetroidAsm(game);
}
pub unsafe fn BabyMetroidPlaySfx0x26(game: &mut Game) {
    QueueSfx3_Max6(0x26 as uint16, game);
    FinishProcessingGameOverBabyMetroidAsm(game);
}
pub unsafe fn BabyMetroidPlaySfx0x27(game: &mut Game) {
    QueueSfx3_Max6(0x27 as uint16, game);
    FinishProcessingGameOverBabyMetroidAsm(game);
}
pub unsafe fn CancelSoundEffects(game: &mut Game) {
    QueueSfx1_Max6(2 as uint16, game);
    QueueSfx2_Max6(0x71 as uint16, game);
    QueueSfx3_Max6(1 as uint16, game);
}
pub unsafe fn QueueSamusMovementSfx(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb3e) as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int
        == 1024 as ::core::ffi::c_int
    {
        QueueSfx3_Max6(0x2b as uint16, game);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcd0) as *mut uint16) as ::core::ffi::c_int
        - 16 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        QueueSfx1_Max6(0x41 as uint16, game);
    }
    CallSomeSamusCode(0x14 as uint16, game);
}
pub unsafe fn AdvancePaletteFadeForAllPalettes_0xc(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xc402) as *mut uint16) = 12 as uint16;
    return AdvancePaletteFadeForAllPalettes(game);
}
pub unsafe fn AdvancePaletteFade_BgPalette6(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xc402) as *mut uint16) = 12 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16) as ::core::ffi::c_int
        <= 0xd as ::core::ffi::c_int
    {
        let mut i: ::core::ffi::c_int = 192 as ::core::ffi::c_int;
        while i < 0xe0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize) =
                CalculateNthTransitionColorFromXtoY(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16),
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset((i >> 1 as ::core::ffi::c_int) as isize),
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                        .offset((i >> 1 as ::core::ffi::c_int) as isize),
                    game,
                );
            i += 2 as ::core::ffi::c_int;
        }
        let ref mut fresh5 = *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16);
        *fresh5 = (*fresh5).wrapping_add(1);
        return 0 as uint8;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16) = 0 as uint16;
        return 1 as uint8;
    };
}
pub unsafe fn AdvanceGradualColorChangeOfPalette(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xc402) as *mut uint16) = 15 as uint16;
    if ((15i16).wrapping_sub(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16)) as i16),
    )) as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        let mut R34: uint16 = j;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                CalculateNthTransitionColorFromXtoY(
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16))
                        .wrapping_add((1u16))),
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
                    game,
                );
            k = ((k).wrapping_add((2u16)));
            if !((k as ::core::ffi::c_int) < R34 as ::core::ffi::c_int) {
                break;
            }
        }
        let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16);
        *fresh3 = (*fresh3).wrapping_add(1);
        return 0 as uint8;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16) = 0 as uint16;
        return 1 as uint8;
    };
}
pub unsafe fn AdvancePaletteFadeForAllPalettes(game: &mut Game) -> uint8 {
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xc402) as *mut uint16)).wrapping_add((1u16)))
        as ::core::ffi::c_int
        >= *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16)
            as ::core::ffi::c_int
    {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 0x200 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc404) as *mut uint16) = i as uint16;
            let mut v2: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
            if *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int
                != *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int
            {
                let mut v3: uint16 = CalculateNthTransitionColorFromXtoY(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16),
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset(v2 as isize),
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                        .offset(v2 as isize),
                    game,
                );
                i = *((&raw mut game.g_ram as *mut uint8).offset(0xc404) as *mut uint16)
                    as ::core::ffi::c_int;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xc404) as *mut uint16)
                        as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                ) = v3;
            }
            i += 2 as ::core::ffi::c_int;
        }
        let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16);
        *fresh4 = (*fresh4).wrapping_add(1);
        return 0 as uint8;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16) = 0 as uint16;
        return 1 as uint8;
    };
}
pub unsafe fn CalculateNthTransitionColorFromXtoY(
    mut a: uint16,
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    return (CalculateNthTransitionColorComponentFromXtoY(
        a,
        ((k) & (0x1fu16)),
        ((j) & (0x1fu16)),
        game,
    ) as ::core::ffi::c_int
        | (CalculateNthTransitionColorComponentFromXtoY(
            a,
            (k as ::core::ffi::c_int >> 5 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
                as uint16,
            (j as ::core::ffi::c_int >> 5 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
                as uint16,
            game,
        ) as ::core::ffi::c_int)
            << 5 as ::core::ffi::c_int
        | (CalculateNthTransitionColorComponentFromXtoY(
            a,
            (k as ::core::ffi::c_int >> 10 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
                as uint16,
            (j as ::core::ffi::c_int >> 10 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
                as uint16,
            game,
        ) as ::core::ffi::c_int)
            << 10 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CalculateNthTransitionColorComponentFromXtoY(
    mut a: uint16,
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    if a == 0 {
        return k;
    }
    let mut v4: uint16 = ((a).wrapping_sub((1u16)));
    if v4 as ::core::ffi::c_int
        == *((&raw mut game.g_ram as *mut uint8).offset(0xc402) as *mut uint16)
            as ::core::ffi::c_int
    {
        return j;
    }
    let mut RegWord: uint16 = SnesDivide(
        ((abs16(((j).wrapping_sub((k)))) as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
            as uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xc402) as *mut uint16) as ::core::ffi::c_int
            - (v4 as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + 1 as ::core::ffi::c_int) as uint8,
    );
    let mut r18: uint16 =
        (if j as ::core::ffi::c_int - k as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            -(RegWord as ::core::ffi::c_int)
        } else {
            RegWord as ::core::ffi::c_int
        }) as uint16;
    return ((r18 as ::core::ffi::c_int + ((k as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        as uint16 as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn sub_82DAF7(mut a: uint16, game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xc402) as *mut uint16) = 12 as uint16;
    return sub_82DB0C(a, game);
}
pub unsafe fn sub_82DB0C(mut a: uint16, game: &mut Game) -> uint8 {
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xc402) as *mut uint16)).wrapping_add((1u16)))
        as ::core::ffi::c_int
        >= *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16)
            as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc404) as *mut uint16) = 0 as uint16;
        while a != 0 {
            if a as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
                let ref mut fresh15 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc404) as *mut uint16);
                *fresh15 = ((*fresh15).wrapping_add((32u16)));
            } else {
                sub_82DB41(game);
            }
            a = (a as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        }
        let ref mut fresh16 = *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16);
        *fresh16 = (*fresh16).wrapping_add(1);
        return 0 as uint8;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16) = 0 as uint16;
        return 1 as uint8;
    };
}
pub unsafe fn sub_82DB41(game: &mut Game) {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xc404) as *mut uint16);
    loop {
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            != *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
        {
            let mut v2: uint16 = CalculateNthTransitionColorFromXtoY(
                *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16),
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                    .offset(v1 as isize),
                *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                    .offset(v1 as isize),
                game,
            );
            v0 = *((&raw mut game.g_ram as *mut uint8).offset(0xc404) as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xc404) as *mut uint16)
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            ) = v2;
        }
        v0 = ((v0).wrapping_add((2u16)));
        *((&raw mut game.g_ram as *mut uint8).offset(0xc404) as *mut uint16) = v0;
        if !(v0 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int != 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
pub unsafe fn HandleSamusOutOfHealthAndGameTile(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) as int16
        as ::core::ffi::c_int
        <= 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9c0) as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16)
                as ::core::ffi::c_int
                != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) = 0x8000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
                kGameState_27_ReserveTanksAuto as uint16;
            CallSomeSamusCode(0x1b as uint16, game);
        } else {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16)
                as ::core::ffi::c_int
                != kGameState_8_MainGameplay as ::core::ffi::c_int
            {
                return;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) = 0x8000 as uint16;
            CallSomeSamusCode(0x11 as uint16, game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
                kGameState_19_SamusNoHealth as uint16;
        }
    }
    let ref mut fresh81 = *((&raw mut game.g_ram as *mut uint8).offset(0x9da) as *mut uint16);
    *fresh81 = (*fresh81).wrapping_add(1);
    if *fresh81 as ::core::ffi::c_int - 60 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9da) as *mut uint16) = 0 as uint16;
        let ref mut fresh82 = *((&raw mut game.g_ram as *mut uint8).offset(0x9dc) as *mut uint16);
        *fresh82 = (*fresh82).wrapping_add(1);
        if *fresh82 as ::core::ffi::c_int - 60 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x9dc) as *mut uint16) = 0 as uint16;
            let ref mut fresh83 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x9de) as *mut uint16);
            *fresh83 = (*fresh83).wrapping_add(1);
            if *fresh83 as ::core::ffi::c_int - 60 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x9de) as *mut uint16) = 0 as uint16;
                let ref mut fresh84 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x9e0) as *mut uint16);
                *fresh84 = (*fresh84).wrapping_add(1);
            }
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e0) as *mut uint16) as ::core::ffi::c_int
        - 100 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9da) as *mut uint16) = 59 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9dc) as *mut uint16) = 59 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9de) as *mut uint16) = 59 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9e0) as *mut uint16) = 99 as uint16;
    }
}
pub unsafe fn GameState_27_ReserveTanksAuto(game: &mut Game) -> CoroutineRet {
    if *(&raw mut game.g_ram as *mut uint8).offset(0x612) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && RefillHealthFromReserveTanks(game) as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
            kGameState_8_MainGameplay as uint16;
        CallSomeSamusCode(0x10 as uint16, game);
    }
    let mut _coret_: uint8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    Samus_LowHealthCheck_0(game);
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn RefillHealthFromReserveTanks(game: &mut Game) -> uint8 {
    let mut current_block: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16) != 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16) as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            QueueSfx3_Max3(0x2d as uint16, game);
        }
        let ref mut fresh11 = *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16);
        *fresh11 = (*fresh11).wrapping_add(1);
        if (((*fresh11) as i16).wrapping_sub(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x9c4) as *mut uint16)) as i16),
        )) as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x9c4) as *mut uint16);
            current_block = 15379867480756242343;
        } else {
            let ref mut fresh13 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16);
            *fresh13 = (*fresh13).wrapping_sub(1);
            let mut v0: bool = *fresh13 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16) == 0 {
                current_block = 15379867480756242343;
            } else if v0 {
                let ref mut fresh14 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16);
                *fresh14 = ((*fresh14).wrapping_add(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16)),
                ));
                current_block = 15379867480756242343;
            } else {
                current_block = 13183875560443969876;
            }
        }
        match current_block {
            13183875560443969876 => {}
            _ => {
                *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16) = 0 as uint16;
                return (*((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16)
                    as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int) as uint8;
            }
        }
    }
    return (*((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16)
        as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int) as uint8;
}
pub unsafe fn GameState_19_SamusNoHealth(game: &mut Game) -> CoroutineRet {
    let mut _coret_: uint8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    let mut i: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[(i + 384 as ::core::ffi::c_int) as usize] =
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(i as isize);
        i -= 1;
    }
    let mut j: ::core::ffi::c_int = 382 as ::core::ffi::c_int;
    while j & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((j >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        j -= 2 as ::core::ffi::c_int;
    }
    let mut k: ::core::ffi::c_int = 94 as ::core::ffi::c_int;
    while k & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset(((k >> 1 as ::core::ffi::c_int) + 208 as ::core::ffi::c_int) as isize) =
            0 as uint16;
        k -= 2 as ::core::ffi::c_int;
    }
    let mut m: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while m & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset(((m >> 1 as ::core::ffi::c_int) + 192 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset(((m >> 1 as ::core::ffi::c_int) + 192 as ::core::ffi::c_int) as isize);
        m -= 2 as ::core::ffi::c_int;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 3 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xde4) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xde6) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xde8) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d2) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa04) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x18aa) as *mut uint16) = 0 as uint16;
    let ref mut fresh109 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
    *fresh109 = (*fresh109).wrapping_add(1);
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_20_SamusNoHealth_BlackOut(game: &mut Game) -> CoroutineRet {
    let mut _coret_: uint8 = GameState_8_MainGameplay(game);
    if _coret_ != 0 {
        return _coret_;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xc402) as *mut uint16) = 6 as uint16;
    if AdvancePaletteFadeForAllPalettes(game) != 0 {
        WaitUntilEndOfVblankAndClearHdma(game);
        DisableIrqInterrupts(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1982) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xab) as *mut uint16) = 0 as uint16;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 16 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x55) = 9 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        let mut i: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 15 as ::core::ffi::c_int) as isize) =
                0 as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xde8) as *mut uint16) = 16 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 3 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xde4) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xde6) as *mut uint16) = 0 as uint16;
        let ref mut fresh110 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
        *fresh110 = (*fresh110).wrapping_add(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16) = 0 as uint16;
        QueueSfx1_Max15(2 as uint16, game);
        QueueSfx2_Max15(0x71 as uint16, game);
        QueueSfx3_Max15(1 as uint16, game);
        QueueMusic_Delayed8(0 as uint16, game);
        QueueMusic_Delayed8(0xff39 as uint16, game);
        QueueMusic_DelayedY(5 as uint16, 0xe as uint16, game);
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_21_SamusNoHealth(game: &mut Game) -> CoroutineRet {
    Samus_DrawWhenNotAnimatingOrDying(game);
    if HasQueuedMusic(game) == 0 {
        StartSamusDeathAnimation(game);
        let ref mut fresh111 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
        *fresh111 = (*fresh111).wrapping_add(1);
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_22_SamusNoHealth_Dying(game: &mut Game) -> CoroutineRet {
    DrawSamusStartingDeathAnim_(game);
    let ref mut fresh113 = *((&raw mut game.g_ram as *mut uint8).offset(0xde8) as *mut uint16);
    *fresh113 = (*fresh113).wrapping_sub(1);
    let mut v0: bool =
        *fresh113 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xde8) as *mut uint16) == 0 || v0 {
        let ref mut fresh114 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
        *fresh114 = (*fresh114).wrapping_add(1);
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_23_SamusNoHealth_Flashing(game: &mut Game) -> CoroutineRet {
    if HandleSamusDeathSequence(game) != 0 {
        let ref mut fresh115 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
        *fresh115 = (*fresh115).wrapping_add(1);
    } else {
        Samus_DrawDuringDeathAnim(game);
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_24_SamusNoHealth_Explosion(game: &mut Game) -> CoroutineRet {
    if GameState_24_SamusNoHealth_Explosion_Helper(game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 1 as uint16;
        let ref mut fresh116 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
        *fresh116 = (*fresh116).wrapping_add(1);
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_25_SamusNoHealth_BlackOut(game: &mut Game) -> CoroutineRet {
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 0x80 as ::core::ffi::c_int
    {
        EnableNMI(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        let ref mut fresh117 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
        *fresh117 = (*fresh117).wrapping_add(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x5f5) as *mut uint16) = 0 as uint16;
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn LoaadDesinationRoomCreBitset(game: &mut Game) {
    let destination_room = (*get_DoorDef(game.door_def_ptr(), game)).room_definition_ptr;
    game.set_previous_cre_bitset(game.cre_bitset());
    let room = get_RoomDefHeader(destination_room, game);
    game.set_cre_bitset(u16::from((*room).cre_bitset_));
}
pub unsafe fn LoadDoorHeader(game: &mut Game) {
    let door = get_DoorDef(game.door_def_ptr(), game);
    game.set_room_ptr((*door).room_definition_ptr);
    // The properties word includes the adjacent orientation byte.
    let properties = u16::from_le_bytes([(*door).door_bitflags, (*door).door_orientation]);
    game.set_elevator_door_properties_orientation(properties);
    game.set_elevator_flags(game.elevator_door_properties_orientation() & 0x0080);
    game.set_door_direction(u16::from((*door).door_orientation));
    game.set_door_destination_x_pos(u16::from((*door).x_pos_in_room) << 8);
    game.set_door_destination_y_pos(u16::from((*door).y_pos_in_room) << 8);
    let mut distance = (*door).samus_distance_from_door as i16;
    if distance < 0 {
        distance = if game.door_direction() & 2 != 0 {
            384
        } else {
            200
        };
    }
    let speed = (i32::from(distance) << 8) as u32;
    game.set_samus_door_transition_subspeed(speed as u16);
    game.set_samus_door_transition_speed((speed >> 16) as u16);
}
pub unsafe fn LoadRoomHeader(game: &mut Game) {
    let room = get_RoomDefHeader(game.room_ptr(), game);
    game.set_room_index(u16::from((*room).semiunique_room_number));
    game.set_area_index(u16::from((*room).area_index_));
    game.set_room_x_coordinate_on_map(u16::from((*room).x_coordinate_on_map));
    game.set_room_y_coordinate_on_map(u16::from((*room).y_coordinate_on_map));
    game.set_room_width_in_scrolls(u16::from((*room).width));
    game.set_room_width_in_blocks(game.room_width_in_scrolls().wrapping_mul(16));
    game.set_room_height_in_scrolls(u16::from((*room).height));
    game.set_room_height_in_blocks(game.room_height_in_scrolls().wrapping_mul(16));
    game.set_up_scroller(u16::from((*room).up_scroller_));
    game.set_down_scroller(u16::from((*room).down_scroller_));
    game.set_door_list_pointer((*room).ptr_to_doorout);
    HandleRoomDefStateSelect(game.room_ptr(), game);
    let product = Mult8x8(
        game.room_width_in_blocks() as u8,
        game.room_height_in_blocks() as u8,
    );
    game.set_room_size_in_blocks(product.wrapping_mul(2));
}
pub unsafe fn LoadStateHeader(game: &mut Game) {
    let room_state = get_RoomDefRoomstate(game.roomdefroomstate_ptr(), game);
    let graphics_set = usize::from((*room_state).graphics_set);
    let tileset_ptr = game.read_rom_word(0x8fe7a7, 2 * graphics_set);
    let tileset = get_TileSet(tileset_ptr, game);
    game.set_tileset_tile_table_pointer((*tileset).tile_table_ptr);
    game.set_tileset_tiles_pointer((*tileset).tiles_ptr);
    game.set_tileset_compr_palette_ptr((*tileset).palette_ptr);
    game.set_room_compr_level_data_ptr((*room_state).compressed_room_map_ptr);
    game.set_room_music_data_index(u16::from((*room_state).music_track));
    game.set_room_music_track_index(u16::from((*room_state).music_control));
    game.set_room_layer3_asm_ptr((*room_state).room_layer3_fx_ptr);
    game.set_room_enemy_population_ptr((*room_state).enemy_population_ptr_);
    game.set_room_enemy_tilesets_ptr((*room_state).enemy_tilesets_ptr);
    let [scroll_x, scroll_y] = (*room_state).vertical_screen_nudge_limit.to_le_bytes();
    game.set_layer2_scroll_x_and_y(scroll_x, scroll_y);
    game.set_room_main_code_ptr((*room_state).main_code_ptr);
}
pub unsafe fn WaitUntilEndOfVblankAndEnableIrq(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0x84) as ::core::ffi::c_int
        & 0x30 as ::core::ffi::c_int
        != 0x30 as ::core::ffi::c_int
    {
        EnableIrqInterruptsNow(game);
    }
}
pub fn PointlessFunctionStupidToo() {}
pub unsafe fn SaveMapExploredifElevator(game: &mut Game) {
    if game.elevator_door_properties_orientation() & 0x000f != 0 {
        SetElevatorsAsUsed(game);
    }
    let door = get_DoorDef(game.door_def_ptr(), game);
    if (*door).door_bitflags & 0x40 != 0 {
        SaveExploredMapTilesToSaved(game);
    }
}
pub unsafe fn LoadMapExploredIfElevator(game: &mut Game) {
    let door = get_DoorDef(game.door_def_ptr(), game);
    if (*door).door_bitflags & 0x40 != 0 {
        LoadMirrorOfExploredMapTiles(game);
    }
}
pub unsafe fn EnsureSamusDrawnEachFrame(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe16) as *mut uint16) == 0 {
        Samus_DrawWhenNotAnimatingOrDying(game);
    }
}
pub unsafe fn LoadEnemyGfxToVram(game: &mut Game) {
    let mut ED: *mut EnemyDef = ::core::ptr::null_mut::<EnemyDef>();
    let mut dst: uint16 = 0x7000 as uint16;
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x7d1) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x7d1) as *mut uint16) != 0 {
        let mut i: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x7d1)
            as *mut uint16) as ::core::ffi::c_int;
        loop {
            let mut enemy_def: uint16 = (*get_EnemyTileset(v0, game)).enemy_def;
            if enemy_def as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
                break;
            }
            ED = get_EnemyDef_A2(enemy_def, game);
            let mut vram_update_size: uint16 = 0;
            let mut vram_update_dst: uint16 = 0;
            if (*ED).tile_data_size as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                vram_update_size = (((*ED).tile_data_size) & (0x7fffu16));
                vram_update_dst = ((((*get_EnemyTileset(i as uint16, game)).vram_dst) & (0xf000u16))
                    as ::core::ffi::c_int
                    >> 4 as ::core::ffi::c_int
                    | 0x6000 as ::core::ffi::c_int) as uint16;
            } else {
                vram_update_size = (*ED).tile_data_size;
                vram_update_dst = dst;
                dst = (dst as ::core::ffi::c_int
                    + ((*ED).tile_data_size as ::core::ffi::c_int >> 1 as ::core::ffi::c_int))
                    as uint16;
            }
            CopyToVramNow(
                vram_update_dst,
                Load24(&raw mut (*ED).tile_data),
                vram_update_size,
                game,
            );
            i += 4 as ::core::ffi::c_int;
            v0 = i as uint16;
        }
    }
}
pub unsafe fn LoadRoomMusic(game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) as ::core::ffi::c_int)
        < kGameState_40_TransitionToDemo as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x7cb) as *mut uint16) as ::core::ffi::c_int
            != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x7cb) as *mut uint16) as ::core::ffi::c_int
            != *((&raw mut game.g_ram as *mut uint8).offset(0x7f3) as *mut uint16)
                as ::core::ffi::c_int
    {
        QueueMusic_Delayed8(0 as uint16, game);
        QueueMusic_Delayed8(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x7cb) as *mut uint16)) | (0xff00u16)),
            game,
        );
    }
}
pub unsafe fn UpdateMusicTrackIndex(game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) as ::core::ffi::c_int)
        < kGameState_40_TransitionToDemo as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x7c9) as *mut uint16) as ::core::ffi::c_int
            != 0
    {
        let mut r18: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x7cb) as *mut uint16)
            as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16;
        r18 = ((r18) | (*((&raw mut game.g_ram as *mut uint8).offset(0x7c9) as *mut uint16)));
        let mut r20: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x7f3) as *mut uint16)
            as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16;
        r20 = ((r20) | (*((&raw mut game.g_ram as *mut uint8).offset(0x7f5) as *mut uint16)));
        if r18 as ::core::ffi::c_int != r20 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x7f5) as *mut uint16) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x7c9) as *mut uint16);
        }
    }
}
pub unsafe fn LoadNewMusicTrackIfChanged(game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) as ::core::ffi::c_int)
        < 0x28 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x7c9) as *mut uint16) as ::core::ffi::c_int
            != 0
    {
        let mut r18: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x7cb) as *mut uint16)
            as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16;
        r18 = ((r18) | (*((&raw mut game.g_ram as *mut uint8).offset(0x7c9) as *mut uint16)));
        let mut r20: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x7f3) as *mut uint16)
            as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16;
        r20 = ((r20) | (*((&raw mut game.g_ram as *mut uint8).offset(0x7f5) as *mut uint16)));
        if r18 as ::core::ffi::c_int != r20 as ::core::ffi::c_int {
            QueueMusic_DelayedY(
                *((&raw mut game.g_ram as *mut uint8).offset(0x7c9) as *mut uint16),
                6 as uint16,
                game,
            );
        }
    }
}
pub fn NullFunc() {}
pub unsafe fn PlayRoomMusicTrackAfterAFrames(mut a: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) as ::core::ffi::c_int)
        < kGameState_40_TransitionToDemo as ::core::ffi::c_int
    {
        QueueMusic_DelayedY(0 as uint16, a, game);
        QueueMusic_Delayed8(
            *((&raw mut game.g_ram as *mut uint8).offset(0x7f5) as *mut uint16),
            game,
        );
    }
}
pub unsafe fn LoadColorsForSpritesBeamsAndEnemies(game: &mut Game) {
    let mut j: uint16 = 0;
    let mut k: uint16 = 0;
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 208 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x9a8000)) as *const uint8 as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 208 as ::core::ffi::c_int) as isize);
        i -= 2 as ::core::ffi::c_int;
    }
    j = 30 as uint16;
    while j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(
            ((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 224 as ::core::ffi::c_int)
                as isize,
        ) = *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
            ((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 224 as ::core::ffi::c_int)
                as isize,
        );
        j = ((j).wrapping_sub((2u16)));
    }
    k = 30 as uint16;
    while k as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(
            ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 128 as ::core::ffi::c_int)
                as isize,
        ) = *(game.g_rom.offset(crate::rtl::lorom_offset(0x9afc00)) as *const uint8 as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        k = ((k).wrapping_sub((2u16)));
    }
}
pub unsafe fn CallDoorTransitionFunction_Async(mut ea: uint32, game: &mut Game) -> CoroutineRet {
    match ea {
        8577405 => return DoorTransitionFunction_HandleElevator(game),
        8577439 => return DoorTransitionFunction_Wait48frames(game),
        8577694 => return DoorTransitionFunction_WaitForSoundsToFinish(game),
        8577755 => return DoorTransitionFunction_FadeOutScreen(game),
        8577783 => return DoorTransitionFunction_LoadDoorHeaderEtc(game),
        8577808 => return DoorTransitionFunction_ScrollScreenToAlignment(game),
        8577875 => return DoorTransitionFunction_FixDoorsMovingUp(game),
        8577902 => return DoorTransitionFunction_SetupNewRoom(game),
        8577934 => return DoorTransitionFunction_SetupScrolling(game),
        8577984 => return DoorTransitionFunction_PlaceSamusLoadTiles(game),
        8578217 => return DoorTransitionFunction_LoadMoreThings_Async(game),
        8578649 => return DoorTransition_C_HandleAnimTiles(game),
        8578660 => return DoorTransition_WaitForMusicToClear(game),
        8578722 => return DoorTransition_HandleTransition(game),
        8578871 => return DoorTransition_FadeInScreenAndFinish(game),
        _ => return Unreachable(game) as CoroutineRet,
    };
}
pub unsafe fn GameState_9_HitDoorBlock(game: &mut Game) -> CoroutineRet {
    CallDoorTransitionFunction_Async(u32::from(game.door_transition_function()) | 0x820000, game)
}
pub unsafe fn DoorTransitionFunction_HandleElevator(game: &mut Game) -> CoroutineRet {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe16) as *mut uint16) == 0 {
        let ref mut fresh6 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
        *fresh6 = (*fresh6).wrapping_add(1);
        return GameState_10_LoadingNextRoom_Async(game);
    }
    CallSomeSamusCode(0 as uint16, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x799) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh7 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
        *fresh7 = (*fresh7).wrapping_add(1);
        return GameState_10_LoadingNextRoom_Async(game);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x92f) as *mut uint16) = 48 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x99c) as *mut uint16) =
        (((0x82e19fi32) as u16) & (0xffffu16));
    return DoorTransitionFunction_Wait48frames(game);
}
pub unsafe fn DoorTransitionFunction_Wait48frames(game: &mut Game) -> CoroutineRet {
    let ref mut fresh8 = *((&raw mut game.g_ram as *mut uint8).offset(0x92f) as *mut uint16);
    *fresh8 = (*fresh8).wrapping_sub(1);
    if *fresh8 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        let ref mut fresh9 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
        *fresh9 = (*fresh9).wrapping_add(1);
        return GameState_10_LoadingNextRoom_Async(game);
    } else {
        DetermineWhichEnemiesToProcess(game);
        EnemyMain(game);
        DrawSamusEnemiesAndProjectiles(game);
        EnsureSamusDrawnEachFrame(game);
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_10_LoadingNextRoom_Async(game: &mut Game) -> CoroutineRet {
    *((&raw mut game.g_ram as *mut uint8).offset(0x797) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x795) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x5f7) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e75) as *mut uint16) = 0 as uint16;
    DetermineWhichEnemiesToProcess(game);
    EnemyMain(game);
    DrawSamusEnemiesAndProjectiles(game);
    EnsureSamusDrawnEachFrame(game);
    LoaadDesinationRoomCreBitset(game);
    let mut i: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v1 + 128 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(9) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(9);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(10) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(10);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(13) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(13);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(14) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(14);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(17) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(17);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(18) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(18);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(19) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(19);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(29) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(29);
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x7b1) as *mut uint16) as ::core::ffi::c_int
        | *((&raw mut game.g_ram as *mut uint8).offset(0x7b3) as *mut uint16) as ::core::ffi::c_int)
        & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(20) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(20);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(21) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(21);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(22) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(22);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(23) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(23);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(28) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(28);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x943) as *mut uint16) != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(209) =
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(209);
            *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(210) =
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(210);
            *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(212) =
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(212);
            *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(221) =
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(221);
            DrawTimer(game);
        }
    }
    ClearSoundsWhenGoingThroughDoor(game);
    QueueSfx2_Max15(0x71 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x5f5) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x99c) as *mut uint16) =
        (((0x82e29ei32) as u16) & (0xffffu16));
    let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16);
    *fresh10 = (*fresh10).wrapping_add(1);
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameState_11_LoadingNextRoom_Async(game: &mut Game) -> CoroutineRet {
    let dispatch_result = CallDoorTransitionFunction_Async(
        u32::from(game.door_transition_function()) | 0x820000,
        game,
    );
    if dispatch_result != 0 {
        return dispatch_result;
    }
    if game.timer_status() != 0 {
        DrawTimer(game);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransitionFunction_WaitForSoundsToFinish(game: &mut Game) -> CoroutineRet {
    DetermineWhichEnemiesToProcess(game);
    EnemyMain(game);
    DrawSamusEnemiesAndProjectiles(game);
    EnsureSamusDrawnEachFrame(game);
    if game.sfx_queue_is_empty(0) && game.sfx_queue_is_empty(1) && game.sfx_queue_is_empty(2) {
        game.set_door_transition_function(0xe2db);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransitionFunction_FadeOutScreen(game: &mut Game) -> CoroutineRet {
    if AdvancePaletteFadeForAllPalettes_0xc(game) != 0 {
        game.set_door_transition_function(0xe2f7);
    } else {
        DetermineWhichEnemiesToProcess(game);
        EnemyMain(game);
        DrawSamusEnemiesAndProjectiles(game);
        EnsureSamusDrawnEachFrame(game);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransitionFunction_LoadDoorHeaderEtc(game: &mut Game) -> CoroutineRet {
    LoadDoorHeader(game);
    sub_8882AC(game);
    game.set_hdma_objects_enable_flag(game.hdma_objects_enable_flag() & 0x7fff);
    game.set_irqhandler_next_handler(8);
    game.set_door_transition_function(0xe310);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransitionFunction_ScrollScreenToAlignment(game: &mut Game) -> CoroutineRet {
    let align_x = game.door_direction() & 2 != 0;
    let position = if align_x {
        game.layer1_x_pos()
    } else {
        game.layer1_y_pos()
    };
    let aligned = position as u8 == 0;
    if !aligned {
        let position = if position & 0x80 != 0 {
            position.wrapping_add(1)
        } else {
            position.wrapping_sub(1)
        };
        if align_x {
            game.set_layer1_x_pos(position);
        } else {
            game.set_layer1_y_pos(position);
        }
    }
    CalculateLayer2PosAndScrollsWhenScrolling(game);
    if aligned {
        game.set_door_transition_function(0xe353);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransitionFunction_FixDoorsMovingUp(game: &mut Game) -> CoroutineRet {
    if game.door_direction() & 3 == 3 {
        FixDoorsMovingUp(game);
    }
    game.set_door_transition_function(0xe36e);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransitionFunction_SetupNewRoom(game: &mut Game) -> CoroutineRet {
    SaveMapExploredifElevator(game);
    LoadRoomHeader(game);
    LoadStateHeader(game);
    LoadMapExploredIfElevator(game);
    InitializeSpecialEffectsForNewRoom(game);
    LoadLevelScrollAndCre(game);
    game.set_door_transition_function(0xe38e);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransitionFunction_SetupScrolling(game: &mut Game) -> CoroutineRet {
    game.set_reg_bg2hofs(0);
    game.set_reg_bg2vofs(0);
    game.set_scrolling_finished_hook(0);
    if game.door_direction() & 3 == 2 {
        game.set_reg_bg1vofs(game.reg_bg1vofs().wrapping_add(1));
    }
    if game.door_direction() & 3 != 3 {
        game.set_door_transition_frame_counter(0);
    }
    DoorTransitionScrollingSetup(game);
    game.set_door_transition_function(0xe3c0);
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransitionFunction_PlaceSamusLoadTiles(game: &mut Game) -> CoroutineRet {
    game.set_samus_x_pos(
        game.layer1_x_pos()
            .wrapping_add(u16::from(game.samus_x_pos() as u8)),
    );
    game.set_samus_prev_x_pos(game.samus_x_pos());
    game.set_samus_y_pos(
        game.layer1_y_pos()
            .wrapping_add(u16::from(game.samus_y_pos() as u8)),
    );
    game.set_samus_prev_y_pos(game.samus_y_pos());
    game.set_door_transition_flag(0);
    let irq_handler = if game.door_direction() & 3 == 2 {
        16
    } else {
        22
    };
    game.set_irqhandler_next_handler(irq_handler);
    WaitUntilEndOfVblankAndEnableIrq(game);
    if game.cre_bitset() & 2 != 0 && game.door_def_ptr() != addr_kDoorDef_947a as u16 {
        DecompressToMem(
            0xb98000 as uint32,
            (&raw mut game.g_ram as *mut uint8).offset(0x7000),
            game,
        );
    }
    DecompressToMem(
        game.tileset_tiles_pointer(),
        (&raw mut game.g_ram as *mut uint8).offset(0x2000),
        game,
    );
    DecompressToMem(
        game.tileset_compr_palette_ptr(),
        (&raw mut game.g_ram as *mut uint8).offset(0xc200),
        game,
    );
    CopyToVramNow(0 as uint16, 0x7e2000 as uint32, 0x2000 as uint16, game);
    CopyToVramNow(0x1000 as uint16, 0x7e4000 as uint32, 0x2000 as uint16, game);
    CopyToVramNow(0x2000 as uint16, 0x7e6000 as uint32, 0x1000 as uint16, game);
    if game.cre_bitset() & 6 != 0 && game.door_def_ptr() != addr_kDoorDef_947a as u16 {
        CopyToVramNow(0x2800 as uint16, 0x7e7000 as uint32, 0x1000 as uint16, game);
        CopyToVramNow(0x3000 as uint16, 0x7e8000 as uint32, 0x2000 as uint16, game);
        CopyToVramNow(0x4000 as uint16, 0x9ab200 as uint32, 0x1000 as uint16, game);
    }
    if game.door_direction() & 3 == 3 {
        game.set_irqhandler_next_handler(16);
    }
    game.set_door_transition_function(0xe4a9);
    kCoroutineNone as CoroutineRet
}
static kUpdateBackgroundCommands: [Func_Y_Y; 8] = [
    UpdateBackgroundCommand_0_Finish,
    UpdateBackgroundCommand_2_TransferToVram,
    UpdateBackgroundCommand_4_Decompression,
    UpdateBackgroundCommand_6_ClearFxTilemap,
    UpdateBackgroundCommand_8_TransferToVramAndSetBg3,
    UpdateBackgroundCommand_A_ClearBg2Tilemap,
    UpdateBackgroundCommand_C_ClearKraidBg2Tilemap,
    UpdateBackgroundCommand_E_DoorDependentTransferToVRAM,
];
pub unsafe fn DoorTransitionFunction_LoadMoreThings_Async(game: &mut Game) -> CoroutineRet {
    let mut bg_data_ptr: uint16 = 0;
    's_188: {
        let mut current_block_45: u64;
        match game.coroutine_state_1() {
            0 => {
                LoadEnemyGfxToVram(game);
                LoadRoomMusic(game);
                ClearEprojs(game);
                ClearAnimtiles(game);
                ClearPaletteFXObjects(game);
                ClearPLMs(game);
                CreatePlmsExecuteDoorAsmRoomSetup(game);
                LoadFXHeader(game);
                SpawnDoorClosingPLM(game);
                UpdateBeamTilesAndPalette(game);
                LoadColorsForSpritesBeamsAndEnemies(game);
                LoadEnemies(game);
                InitializeEnemies(game);
                ResetProjectileData(game);
                Samus_LoadSuitTargetPalette(game);
                ClearFxTilemap(game);
                let fx_tilemap_ptr = u16::from_le_bytes([game.g_ram[0x1964], game.g_ram[0x1965]]);
                if fx_tilemap_ptr != 0 {
                    CopyToVramNow(
                        addr_unk_605BE0 as u16,
                        0x8a0000 | u32::from(fx_tilemap_ptr),
                        2112,
                        game,
                    );
                }
                bg_data_ptr = game.read_rom_word(
                    0x8f0000 | u32::from(game.roomdefroomstate_ptr()),
                    core::mem::offset_of!(RoomDefRoomstate, bg_data_ptr),
                );
                if bg_data_ptr & 0x8000 != 0 {
                    loop {
                        let command = game.read_rom_word(0x8f0000 | u32::from(bg_data_ptr), 0);
                        bg_data_ptr = kUpdateBackgroundCommands[usize::from(command >> 1)](
                            bg_data_ptr.wrapping_add(2),
                            game,
                        );
                        if bg_data_ptr == 0 {
                            break;
                        }
                    }
                }
                current_block_45 = 2370887241019905314;
            }
            1 => {
                current_block_45 = 16864618877125483552;
            }
            2 => {
                current_block_45 = 17993885260445251066;
            }
            _ => {
                current_block_45 = 6417057564578538666;
            }
        }
        loop {
            match current_block_45 {
                6417057564578538666 => {
                    break 's_188;
                }
                2370887241019905314 => {
                    let door_transition_flag =
                        u16::from_le_bytes([game.g_ram[0x931], game.g_ram[0x932]]);
                    if door_transition_flag & 0x8000 == 0 {
                        current_block_45 = 16864618877125483552;
                        continue;
                    }
                    game.g_ram[0xc188..0xc18a].copy_from_slice(&15328u16.to_le_bytes());
                    SpawnBG3ScrollHdmaObject(game);
                    game.set_hdma_objects_enable_flag(game.hdma_objects_enable_flag() | 0x8000);
                    current_block_45 = 17993885260445251066;
                }
                16864618877125483552 => {
                    let mut _coret_: uint8 = WaitForNMI_Async(game);
                    if _coret_ != 0 {
                        game.set_coroutine_state_1(1);
                        return _coret_;
                    }
                    current_block_45 = 2370887241019905314;
                }
                _ => {
                    let mut _coret__0: uint8 = PlmHandler_Async(game);
                    if _coret__0 != 0 {
                        game.set_coroutine_state_1(2);
                        return _coret__0;
                    }
                    if game.door_direction() & 2 == 0 {
                        if game.door_direction() & 3 != 0 {
                            game.set_samus_x_pos(game.samus_x_pos() & 0xfff8);
                        } else {
                            game.set_samus_x_pos(game.samus_x_pos() | 7);
                        }
                    }
                    game.set_door_transition_function(0xe659);
                    current_block_45 = 6417057564578538666;
                }
            }
        }
    }
    game.set_coroutine_state_1(0);
    return 0 as CoroutineRet;
}
pub unsafe fn ClearFxTilemap(game: &mut Game) {
    for i in (0..960).rev() {
        let offset = i * 2;
        game.g_ram[0x4000 + offset..0x4002 + offset].copy_from_slice(&6222u16.to_le_bytes());
        game.g_ram[0x4780 + offset..0x4782 + offset].copy_from_slice(&6222u16.to_le_bytes());
    }
    CopyToVramNow(0x5880, 0x7e4000, 0xf00, game);
}
pub unsafe fn ClearBg2Tilemap(game: &mut Game) {
    for i in (0..1024).rev() {
        let offset = i * 2;
        game.g_ram[0x4000 + offset..0x4002 + offset].copy_from_slice(&824u16.to_le_bytes());
        game.g_ram[0x4800 + offset..0x4802 + offset].copy_from_slice(&824u16.to_le_bytes());
    }
    CopyToVramNow(0x4800, 0x7e4000, 0x1000, game);
}
pub unsafe fn ClearKraidBg2Tilemap(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 1023 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [i as usize] = 824 as uint16;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
            [(i + 1024 as ::core::ffi::c_int) as usize] = 824 as uint16;
        i -= 1;
    }
    CopyToVramNow(0x4000 as uint16, 0x7e4000 as uint32, 0x1000 as uint16, game);
    CopyToVramNow(0x4800 as uint16, 0x7e4000 as uint32, 0x1000 as uint16, game);
}
pub fn UpdateBackgroundCommand_0_Finish(mut y: uint16, game: &mut Game) -> uint16 {
    return 0 as uint16;
}
pub unsafe fn UpdateBackgroundCommand_E_DoorDependentTransferToVRAM(
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x78d) as *mut uint16) as ::core::ffi::c_int
        == *(RomPtr_8F(j, game) as *mut uint16) as ::core::ffi::c_int
    {
        return UpdateBackgroundCommand_2_TransferToVram(((j).wrapping_add((2u16))), game);
    } else {
        return ((j).wrapping_add((9u16)));
    };
}
pub unsafe fn UpdateBackgroundCommand_2_TransferToVram(j: uint16, game: &mut Game) -> uint16 {
    let address = 0x8f0000 | u32::from(j);
    let destination = game.read_rom_word(address, 3);
    let source = u32::from(game.read_rom_word(address, 0))
        | (u32::from(game.read_rom_byte(address, 2)) << 16);
    let size = game.read_rom_word(address, 5);
    CopyToVramNow(destination, source, size, game);
    j.wrapping_add(7)
}
pub unsafe fn UpdateBackgroundCommand_4_Decompression(j: uint16, game: &mut Game) -> uint16 {
    let address = 0x8f0000 | u32::from(j);
    let source = u32::from(game.read_rom_word(address, 0))
        | (u32::from(game.read_rom_byte(address, 2)) << 16);
    let destination = usize::from(game.read_rom_word(address, 3));
    DecompressToMem(source, game.g_ram.as_mut_ptr().add(destination), game);
    j.wrapping_add(5)
}
pub unsafe fn UpdateBackgroundCommand_6_ClearFxTilemap(mut j: uint16, game: &mut Game) -> uint16 {
    ClearFxTilemap(game);
    return j;
}
pub unsafe fn UpdateBackgroundCommand_8_TransferToVramAndSetBg3(
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    j = UpdateBackgroundCommand_2_TransferToVram(j, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 2 as uint8;
    return j;
}
pub unsafe fn UpdateBackgroundCommand_A_ClearBg2Tilemap(j: uint16, game: &mut Game) -> uint16 {
    ClearBg2Tilemap(game);
    j
}
pub unsafe fn UpdateBackgroundCommand_C_ClearKraidBg2Tilemap(
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    ClearKraidBg2Tilemap(game);
    return j;
}
pub unsafe fn DoorTransition_C_HandleAnimTiles(game: &mut Game) -> CoroutineRet {
    AnimtilesHandler(game);
    game.set_door_transition_function(fnDoorTransition_WaitForMusicToClear as u16);
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn DoorTransition_WaitForMusicToClear(game: &mut Game) -> CoroutineRet {
    if HasQueuedMusic(game) == 0 {
        game.set_door_transition_function(fnDoorTransition_HandleTransition as u16);
        LoadNewMusicTrackIfChanged(game);
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn DoorTransition_HandleTransition(game: &mut Game) -> CoroutineRet {
    let x = game.samus_x_pos();
    match x & 0x00f0 {
        0x10 => game.set_samus_x_pos((x | 0x000f).wrapping_add(8)),
        0xe0 => game.set_samus_x_pos((x & 0xfff0).wrapping_sub(8)),
        _ => {}
    }
    let y = game.samus_y_pos();
    if y & 0x00f0 == 0x10 {
        game.set_samus_y_pos((y | 0x000f).wrapping_add(8));
    }
    for offset in (0..512).step_by(2).rev() {
        game.g_ram[0x9c00 + offset..0x9c02 + offset].copy_from_slice(&[0, 0]);
        game.g_ram[0x9e00 + offset..0x9e02 + offset].copy_from_slice(&[0, 0]);
    }
    let room_irq = game.room_loading_irq_handler();
    game.set_irqhandler_next_handler(if room_irq == 0 { 4 } else { room_irq });
    PointlessFunctionStupidToo();
    if game.elevator_flags() != 0 {
        let samus_command = if game.elevator_direction() & 0x8000 == 0 {
            7
        } else {
            0
        };
        CallSomeSamusCode(samus_command, game);
    }
    SetLiquidPhysicsType(game);
    game.set_door_transition_function(fnDoorTransition_FadeInScreenAndFinish as u16);
    let inidisp_obsel = u16::from_le_bytes([game.g_ram[0x51], game.g_ram[0x52]]);
    game.g_ram[0x51..0x53].copy_from_slice(&(inidisp_obsel | 0x001f).to_le_bytes());
    kCoroutineNone as CoroutineRet
}
pub unsafe fn DoorTransition_FadeInScreenAndFinish(game: &mut Game) -> CoroutineRet {
    AnimtilesHandler(game);
    DetermineWhichEnemiesToProcess(game);
    EnemyMain(game);
    EprojRunAll(game);
    DrawSamusEnemiesAndProjectiles(game);
    EnsureSamusDrawnEachFrame(game);
    QueueEnemyBG2TilemapTransfers(game);
    if AdvancePaletteFadeForAllPalettes_0xc(game) != 0 {
        game.set_debug_disable_sounds(0);
        PlaySpinJumpSoundIfSpinJumping(game);
        game.set_door_transition_flag_elevator_zebetites(0);
        game.set_door_transition_flag_enemies(0);
        game.set_game_state(kGameState_8_MainGameplay as u16);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn LoadDestinationRoomThings(game: &mut Game) {
    LoaadDesinationRoomCreBitset(game);
    LoadDoorHeader(game);
    LoadRoomHeader(game);
    LoadStateHeader(game);
    LoadCRETilesTilesetTilesAndPalette(game);
}
pub unsafe fn LoadCRETilesTilesetTilesAndPalette(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xe16) as *mut uint16) = 0 as uint16;
    WriteRegWord(VMAIN as uint16, 0x80 as uint16, game);
    WriteRegWord(
        VMADDL as uint16,
        (addr_unk_605000 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16,
        game,
    );
    DecompressToVRAM(0xb98000 as uint32, addr_unk_605000 as uint16, game);
    WriteRegWord(VMADDL as uint16, 0 as uint16, game);
    DecompressToVRAM(
        Load24((&raw mut game.g_ram as *mut uint8).offset(0x7c3) as *mut LongPtr),
        0 as uint16,
        game,
    );
    DecompressToMem(
        Load24((&raw mut game.g_ram as *mut uint8).offset(0x7c6) as *mut LongPtr),
        (&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint8,
        game,
    );
}
pub unsafe fn LoadLevelDataAndOtherThings(game: &mut Game) {
    let mut rdf_scroll_ptr: int16 = 0;
    let mut m: uint16 = 0;
    let mut v10: int8 = 0;
    let mut v11: int8 = 0;
    let mut n: uint16 = 0;
    let mut i: ::core::ffi::c_int = 25598 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0x8000 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    DecompressToMem(
        Load24((&raw mut game.g_ram as *mut uint8).offset(0x7bd) as *mut LongPtr),
        (&raw mut game.g_ram as *mut uint8).offset(0x10000) as *mut uint16 as *mut uint8,
        game,
    );
    let mut size: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x10000) as *mut uint16);
    memcpy(
        (&raw mut game.g_ram as *mut uint8).offset(0x19602) as *mut uint16
            as *mut ::core::ffi::c_void,
        ((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16 as *mut uint8)
            .offset(size as ::core::ffi::c_int as isize)
            .offset((size as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as *const ::core::ffi::c_void,
        size as size_t,
    );
    memcpy(
        (&raw mut game.g_ram as *mut uint8).offset(0x16402) as *mut ::core::ffi::c_void,
        ((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16 as *mut uint8)
            .offset(size as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        (size as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as size_t,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as ::core::ffi::c_int
        == 6 as ::core::ffi::c_int
    {
        DecompressToMem(
            Load24((&raw mut game.g_ram as *mut uint8).offset(0x7c0) as *mut LongPtr),
            (&raw mut game.g_ram as *mut uint8).offset(0xa000),
            game,
        );
    } else {
        DecompressToMem(
            0xb9a09d as uint32,
            (&raw mut game.g_ram as *mut uint8).offset(0xa000),
            game,
        );
        DecompressToMem(
            Load24((&raw mut game.g_ram as *mut uint8).offset(0x7c0) as *mut LongPtr),
            (&raw mut game.g_ram as *mut uint8).offset(0xa800),
            game,
        );
    }
    let mut RD: *mut RoomDefRoomstate = get_RoomDefRoomstate(
        *((&raw mut game.g_ram as *mut uint8).offset(0x7bb) as *mut uint16),
        game,
    );
    rdf_scroll_ptr = (*RD).rdf_scroll_ptr as int16;
    if rdf_scroll_ptr as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        let mut scrollval: uint16 = (*RD).rdf_scroll_ptr;
        let mut r20: uint8 =
            (((*((&raw mut game.g_ram as *mut uint8).offset(0x7ab) as *mut uint16)) as u8)
                .wrapping_sub((1u8)));
        let mut v8: uint8 = 2 as uint8;
        let mut v9: uint8 = 0 as uint8;
        v10 = 0 as int8;
        loop {
            if v10 as ::core::ffi::c_int == r20 as ::core::ffi::c_int {
                v8 = (((scrollval) as u8).wrapping_add((1u8)));
            }
            v11 = 0 as int8;
            loop {
                let fresh145 = v9;
                v9 = v9.wrapping_add(1);
                *(&raw mut game.g_ram as *mut uint8)
                    .offset(0xcd20)
                    .offset(fresh145 as isize) = v8;
                v11 += 1;
                if !(v11 as ::core::ffi::c_int
                    != *((&raw mut game.g_ram as *mut uint8).offset(0x7a9) as *mut uint16) as uint8
                        as ::core::ffi::c_int)
                {
                    break;
                }
            }
            v10 += 1;
            if !(v10 as ::core::ffi::c_int
                != *((&raw mut game.g_ram as *mut uint8).offset(0x7ab) as *mut uint16) as uint8
                    as ::core::ffi::c_int)
            {
                break;
            }
        }
    } else {
        m = 0 as uint16;
        while m as ::core::ffi::c_int != 50 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0xcd20)
                .offset(m as isize) as *mut uint8 as *mut uint16) =
                *(RomPtr_8F(rdf_scroll_ptr as uint16_t, game) as *mut uint16);
            rdf_scroll_ptr = ((rdf_scroll_ptr).wrapping_add((2i16)));
            m = ((m).wrapping_add((2u16)));
        }
    }
    if (*RD).room_plm_header_ptr != 0 {
        n = (*RD).room_plm_header_ptr;
        while (*get_RoomPlmEntry(n, game)).plm_header_ptr_ != 0 {
            SpawnRoomPLM(n, game);
            n = ((n).wrapping_add((6u16)));
        }
    }
    RunDoorSetupCode(game);
    RunRoomSetupCode(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe16) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xe18) as *mut uint16) = 2 as uint16;
    }
}
pub unsafe fn SpawnDoorClosingPLM(game: &mut Game) {
    if CheckIfColoredDoorCapSpawned(game) == 0 {
        let mut v0: *const uint16 = RomPtr_8F(
            (2 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0x791) as *mut uint16)
                    as ::core::ffi::c_int
                + addr_kDoorClosingPlmIds as ::core::ffi::c_int) as uint16_t,
            game,
        ) as *const uint16;
        if *v0 != 0 {
            let mut rp: *mut RoomPlmEntry =
                (&raw mut game.g_ram as *mut uint8).offset(0x12) as *mut RoomPlmEntry;
            (*rp).plm_header_ptr_ = *v0;
            *(&raw mut (*rp).x_block as *mut uint16) = *(&raw mut (*(get_DoorDef
                as unsafe fn(uint16, &mut Game) -> *mut DoorDef)(
                *((&raw mut game.g_ram as *mut uint8).offset(0x78d) as *mut uint16),
                game,
            ))
            .x_pos_plm as *mut uint16);
            (*rp).plm_room_argument = 0 as uint16;
            SpawnRoomPLM(0x12 as uint16, game);
        }
    }
}
pub unsafe fn CheckIfColoredDoorCapSpawned(game: &mut Game) -> uint8 {
    let mut DD: *mut DoorDef = get_DoorDef(
        *((&raw mut game.g_ram as *mut uint8).offset(0x78d) as *mut uint16),
        game,
    );
    let mut v2: uint16 = (2 as ::core::ffi::c_int
        * (Mult8x8(
            (*DD).y_pos_plm,
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8,
        ) as ::core::ffi::c_int
            + (*DD).x_pos_plm as ::core::ffi::c_int)) as uint16;
    let mut v3: uint16 = 78 as uint16;
    while v2 as ::core::ffi::c_int
        != *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
    {
        v3 = ((v3).wrapping_sub((2u16)));
        if v3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return 0 as uint8;
        }
    }
    let mut v5: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16).offset(v5 as isize) == 0
    {
        return 0 as uint8;
    }
    let mut v6: int16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16)
        .offset(v5 as isize) as int16;
    if v6 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        let mut idx: ::core::ffi::c_int =
            PrepareBitAccess(v6 as uint16, game) as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5e7) as *mut uint16) as ::core::ffi::c_int
            & *(&raw mut game.g_ram as *mut uint8)
                .offset(0xd8b0)
                .offset(idx as isize) as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            return 0 as uint8;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xde1c) as *mut uint16).offset(v5 as isize) =
        1 as uint16;
    let mut v8: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16).offset(v5 as isize);
    if v8 == 0 {
        return 0 as uint8;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1d27) as *mut uint16).offset(v5 as isize) =
        (*get_PlmHeader_Size6(v8, game)).instr_list_2_ptr;
    return 1 as uint8;
}
static kLoadLibraryBackgroundFuncs: [Option<Func_Y_Y>; 8] = unsafe {
    [
        Some(LoadLibraryBackgroundFunc_0_DONE as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LoadLibraryBackgroundFunc_2_TransferToVram as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LoadLibraryBackgroundFunc_4_Decompress as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LoadLibraryBackgroundFunc_6_ClearFxTilemap as unsafe fn(uint16, &mut Game) -> uint16),
        Some(
            LoadLibraryBackgroundFunc_8_TransferToVramAndSetBg3
                as unsafe fn(uint16, &mut Game) -> uint16,
        ),
        Some(LoadLibraryBackgroundFunc_A_ClearBG2Tilemap as unsafe fn(uint16, &mut Game) -> uint16),
        Some(
            LoadLibraryBackgroundFunc_C_ClearKraidLayer2 as unsafe fn(uint16, &mut Game) -> uint16,
        ),
        Some(
            LoadLibraryBackgroundFunc_E_DoorDependentTransferToVram
                as unsafe fn(uint16, &mut Game) -> uint16,
        ),
    ]
};
pub unsafe fn LoadLibraryBackground(game: &mut Game) {
    let mut bg_data_ptr: uint16 = 0;
    ClearFXTilemap(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1964) as *mut uint16) != 0 {
        WriteRegWord(
            A1T1L as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x1964) as *mut uint16),
            game,
        );
        WriteRegWord(VMADDL as uint16, addr_unk_605BE0 as uint16, game);
        WriteRegWord(DMAP1 as uint16, 0x1801 as uint16, game);
        WriteRegWord(A1B1 as uint16, 0x8a as uint16, game);
        WriteRegWord(DAS1L as uint16, 0x840 as uint16, game);
        WriteReg(VMAIN as uint16, 0x80 as uint8, game);
        WriteReg(MDMAEN as uint16, 2 as uint8, game);
    }
    bg_data_ptr = (*get_RoomDefRoomstate(
        *((&raw mut game.g_ram as *mut uint8).offset(0x7bb) as *mut uint16),
        game,
    ))
    .bg_data_ptr;
    if bg_data_ptr as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        loop {
            let mut v1: uint16 = *(RomPtr_8F(bg_data_ptr, game) as *mut uint16);
            bg_data_ptr = kLoadLibraryBackgroundFuncs
                [(v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
                .expect("non-null function pointer")(
                ((bg_data_ptr).wrapping_add((2u16))), game
            );
            if !(bg_data_ptr != 0) {
                break;
            }
        }
    }
}
pub fn LoadLibraryBackgroundFunc_0_DONE(mut j: uint16, game: &mut Game) -> uint16 {
    return 0 as uint16;
}
pub unsafe fn LoadLibraryBackgroundFunc_E_DoorDependentTransferToVram(
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x78d) as *mut uint16) as ::core::ffi::c_int
        == (*get_LoadBg_E(j, game)).field_0 as ::core::ffi::c_int
    {
        return LoadLibraryBackgroundFunc_2_TransferToVram(((j).wrapping_add((2u16))), game);
    } else {
        return ((j).wrapping_add((9u16)));
    };
}
pub unsafe fn LoadLibraryBackgroundFunc_2_TransferToVram(mut j: uint16, game: &mut Game) -> uint16 {
    let mut LoadBg_28: *mut uint16 = RomPtr_8F(j, game) as *mut uint16;
    WriteRegWord(
        VMADDL as uint16,
        *((LoadBg_28 as *mut uint8).offset(3) as *mut uint16),
        game,
    );
    WriteRegWord(DMAP1 as uint16, 0x1801 as uint16, game);
    WriteRegWord(A1T1L as uint16, *LoadBg_28, game);
    WriteRegWord(A1B1 as uint16, *LoadBg_28.offset(1), game);
    WriteRegWord(
        DAS1L as uint16,
        *((LoadBg_28 as *mut uint8).offset(5) as *mut uint16),
        game,
    );
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    return ((j).wrapping_add((7u16)));
}
pub unsafe fn LoadLibraryBackgroundFunc_4_Decompress(mut j: uint16, game: &mut Game) -> uint16 {
    let mut p: *const uint8 = RomPtr_8F(j, game);
    DecompressToMem(
        Load24(p as *mut LongPtr),
        (&raw mut game.g_ram as *mut uint8)
            .offset(*(p.offset(3) as *mut uint16) as ::core::ffi::c_int as isize),
        game,
    );
    return ((j).wrapping_add((5u16)));
}
pub unsafe fn LoadLibraryBackgroundFunc_6_ClearFxTilemap(mut j: uint16, game: &mut Game) -> uint16 {
    ClearFXTilemap(game);
    return j;
}
pub unsafe fn LoadLibraryBackgroundFunc_A_ClearBG2Tilemap(
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    ClearBG2Tilemap(game);
    return j;
}
pub unsafe fn LoadLibraryBackgroundFunc_C_ClearKraidLayer2(
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    ClearBG2Tilemap(game);
    return j;
}
pub unsafe fn LoadLibraryBackgroundFunc_8_TransferToVramAndSetBg3(
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    j = LoadLibraryBackgroundFunc_2_TransferToVram(j, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 2 as uint8;
    return j;
}
pub unsafe fn LoadLevelScrollAndCre(game: &mut Game) {
    let mut rdf_scroll_ptr: int16 = 0;
    let mut m: uint16 = 0;
    let mut v10: int8 = 0;
    let mut v11: int8 = 0;
    let mut v12: int8 = 0;
    let mut i: ::core::ffi::c_int = 6398 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0x8000 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(
            ((i >> 1 as ::core::ffi::c_int) + 3200 as ::core::ffi::c_int * 1 as ::core::ffi::c_int)
                as isize,
        ) = 0x8000 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(
            ((i >> 1 as ::core::ffi::c_int) + 3200 as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
                as isize,
        ) = 0x8000 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(
            ((i >> 1 as ::core::ffi::c_int) + 3200 as ::core::ffi::c_int * 3 as ::core::ffi::c_int)
                as isize,
        ) = 0x8000 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    DecompressToMem(
        Load24((&raw mut game.g_ram as *mut uint8).offset(0x7bd) as *mut LongPtr),
        (&raw mut game.g_ram as *mut uint8).offset(0x10000) as *mut uint16 as *mut uint8,
        game,
    );
    let mut size: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x10000) as *mut uint16);
    memcpy(
        (&raw mut game.g_ram as *mut uint8).offset(0x19602) as *mut uint16
            as *mut ::core::ffi::c_void,
        ((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16 as *mut uint8)
            .offset(size as ::core::ffi::c_int as isize)
            .offset((size as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as *const ::core::ffi::c_void,
        size as size_t,
    );
    memcpy(
        (&raw mut game.g_ram as *mut uint8).offset(0x16402) as *mut ::core::ffi::c_void,
        ((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16 as *mut uint8)
            .offset(size as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        (size as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as size_t,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as ::core::ffi::c_int
        == 6 as ::core::ffi::c_int
    {
        DecompressToMem(
            Load24((&raw mut game.g_ram as *mut uint8).offset(0x7c0) as *mut LongPtr),
            (&raw mut game.g_ram as *mut uint8).offset(0xa000),
            game,
        );
    } else {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x7b3) as *mut uint16) as ::core::ffi::c_int
            & 2 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            DecompressToMem(
                0xb9a09d as uint32,
                (&raw mut game.g_ram as *mut uint8).offset(0xa000),
                game,
            );
        }
        DecompressToMem(
            Load24((&raw mut game.g_ram as *mut uint8).offset(0x7c0) as *mut LongPtr),
            (&raw mut game.g_ram as *mut uint8).offset(0xa800),
            game,
        );
    }
    let mut RD: *mut RoomDefRoomstate = get_RoomDefRoomstate(
        *((&raw mut game.g_ram as *mut uint8).offset(0x7bb) as *mut uint16),
        game,
    );
    rdf_scroll_ptr = (*RD).rdf_scroll_ptr as int16;
    if rdf_scroll_ptr as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        let mut r18: uint16 = (*RD).rdf_scroll_ptr;
        let mut r20: uint8 =
            (((*((&raw mut game.g_ram as *mut uint8).offset(0x7ab) as *mut uint16)) as u8)
                .wrapping_sub((1u8)));
        let mut v8: uint8 = 2 as uint8;
        let mut v9: uint8 = 0 as uint8;
        v10 = 0 as int8;
        loop {
            if v10 as ::core::ffi::c_int == r20 as ::core::ffi::c_int {
                v8 = (((r18) as u8).wrapping_add((1u8)));
            }
            v12 = v10;
            v11 = 0 as int8;
            loop {
                let fresh35 = v9;
                v9 = v9.wrapping_add(1);
                *(&raw mut game.g_ram as *mut uint8)
                    .offset(0xcd20)
                    .offset(fresh35 as isize) = v8;
                v11 += 1;
                if !(v11 as ::core::ffi::c_int
                    != *((&raw mut game.g_ram as *mut uint8).offset(0x7a9) as *mut uint16) as uint8
                        as ::core::ffi::c_int)
                {
                    break;
                }
            }
            v10 = ((v12).wrapping_add((1i8)));
            if !(v12 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                != *((&raw mut game.g_ram as *mut uint8).offset(0x7ab) as *mut uint16) as uint8
                    as ::core::ffi::c_int)
            {
                break;
            }
        }
    } else {
        m = 0 as uint16;
        while m as ::core::ffi::c_int != 50 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0xcd20)
                .offset(m as isize) as *mut uint8 as *mut uint16) =
                *(RomPtr_8F(rdf_scroll_ptr as uint16_t, game) as *mut uint16);
            rdf_scroll_ptr = ((rdf_scroll_ptr).wrapping_add((2i16)));
            m = ((m).wrapping_add((2u16)));
        }
    };
}
pub unsafe fn CreatePlmsExecuteDoorAsmRoomSetup(game: &mut Game) {
    let mut RoomDefRoomstate: *mut RoomDefRoomstate = ::core::ptr::null_mut::<RoomDefRoomstate>();
    RoomDefRoomstate = get_RoomDefRoomstate(
        *((&raw mut game.g_ram as *mut uint8).offset(0x7bb) as *mut uint16),
        game,
    );
    if (*RoomDefRoomstate).room_plm_header_ptr != 0 {
        let mut i: ::core::ffi::c_int =
            (*RoomDefRoomstate).room_plm_header_ptr as ::core::ffi::c_int;
        while (*get_RoomPlmEntry(i as uint16, game)).plm_header_ptr_ != 0 {
            SpawnRoomPLM(i as uint16, game);
            i += 6 as ::core::ffi::c_int;
        }
    }
    RunDoorSetupCode(game);
    RunRoomSetupCode(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe16) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xe18) as *mut uint16) = 2 as uint16;
    }
}
static kGameOptionsMenuFuncs: [Option<Func_V>; 13] = unsafe {
    [
        Some(GameOptionsMenuFunc_0 as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenu_1_LoadingOptionsScreen as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenu_2_FadeInOptionsScreen as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenu_3_OptionsScreen as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenu_4_StartGame as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenu_5_DissolveOutScreen as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenu_6_DissolveInScreen as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenu_7_ControllerSettings as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenu_8_SpecialSettings as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenu_9_ScrollControllerSettingsDown as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenu_A_ScrollControllerSettingsUp as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenu_B_TransitionBackToFileSelect as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenu_C_FadeOutOptionsScreenToStart as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn GameState_2_GameOptionsMenu(game: &mut Game) -> CoroutineRet {
    kGameOptionsMenuFuncs
        [*((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) as usize]
        .expect("non-null function pointer")(game);
    OptionsMenuFunc1(game);
    DrawOptionsMenuSpritemaps(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) as ::core::ffi::c_int
        - 2 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        OptionsMenu_AddToVramQueue(game);
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn GameOptionsMenuFunc_0(game: &mut Game) {
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 0x80 as ::core::ffi::c_int
    {
        EnableNMI(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        let ref mut fresh65 = *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16);
        *fresh65 = (*fresh65).wrapping_add(1);
    } else if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 14 as ::core::ffi::c_int
    {
        let mut v0: uint16 = *(&raw mut game.g_ram as *mut uint8).offset(0x6b) as uint16;
        if *(&raw mut game.g_ram as *mut uint8).offset(0x6b) as ::core::ffi::c_int
            & 4 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            CreateOptionsMenuObject_(v0, addr_stru_82F4D6 as uint16, game);
        }
    }
}
pub unsafe fn GameOptionsMenu_1_LoadingOptionsScreen(game: &mut Game) {
    let mut j: uint16 = 0;
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
    WriteReg(VMADDL as uint16, 0 as uint8, game);
    WriteReg(VMADDH as uint16, 0x58 as uint8, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    static unk_82EC3D: StartDmaCopy = StartDmaCopy::vram_words(1, 0x8edc00, 0x800);
    SetupDmaTransfer(&raw const unk_82EC3D as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xde0) as *mut uint16) = 0 as uint16;
    let mut i: ::core::ffi::c_int = 510 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x8ee400)) as *const uint8 as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize);
        i -= 2 as ::core::ffi::c_int;
    }
    DecompressToMem(
        0x978df4 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x1c000),
        game,
    );
    DecompressToMem(
        0x978fcd as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x1c800),
        game,
    );
    DecompressToMem(
        0x9791c4 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x1d000),
        game,
    );
    DecompressToMem(
        0x97938d as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x1d800),
        game,
    );
    DecompressToMem(
        0x97953a as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x1e000),
        game,
    );
    j = 1023 as uint16;
    while j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[j as usize] = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x19602) as *mut uint16)
            .offset((j as ::core::ffi::c_int + 5375 as ::core::ffi::c_int) as isize);
        j = j.wrapping_sub(1);
    }
    // The ROM keeps the final copied tile in A through the object clear.
    let initial_tile = u16::from_le_bytes([game.g_ram[0x3000], game.g_ram[0x3001]]);
    *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 0 as uint16;
    DeleteAllOptionsMenuObjects_(game);
    CreateOptionsMenuObject_(initial_tile, addr_stru_82F4B8 as uint16, game);
    // The first initializer leaves its palette $0E00 in A for this call.
    CreateOptionsMenuObject_(0x0e00, addr_stru_82F4C4 as uint16, game);
    let ref mut fresh66 = *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16);
    *fresh66 = (*fresh66).wrapping_add(1);
    OptionsMenuFunc4(game);
}
pub unsafe fn GameOptionsMenu_2_FadeInOptionsScreen(game: &mut Game) {
    HandleFadeIn(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        let ref mut fresh67 = *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16);
        *fresh67 = (*fresh67).wrapping_add(1);
    }
}
pub unsafe fn OptionsMenu_AddToVramQueue(game: &mut Game) {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    (*gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .size = 2048 as uint16;
    v0 = ((v0).wrapping_add((2u16)));
    (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size = 12288 as uint16;
    v0 = ((v0).wrapping_add((2u16)));
    let fresh118 = v0;
    v0 = v0.wrapping_add(1);
    *(&raw mut (*(gVramWriteEntry
        as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
        fresh118 as ::core::ffi::c_int,
        game,
    ))
    .size as *mut uint8)
        .offset(0) = 126 as uint8;
    (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size = 20480 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        ((v0).wrapping_add((2u16)));
}
pub unsafe fn OptionsMenuFunc5(mut a: uint16, mut k: uint16, mut j: uint16, game: &mut Game) {
    loop {
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = (a as ::core::ffi::c_int
            | *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                .pause_menu_map_tilemap as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16)
                as ::core::ffi::c_int
                & 0xe3ff as ::core::ffi::c_int)
            as uint16;
        k = ((k).wrapping_add((2u16)));
        j = ((j).wrapping_sub((2u16)));
        if !(j != 0) {
            break;
        }
    }
}
static kGameOptionsMenuItemFuncs: [Option<Func_V>; 5] = unsafe {
    [
        Some(GameOptionsMenuItemFunc_0 as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenuItemFunc_2_ToggleJapanese as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenuItemFunc_2_ToggleJapanese as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenuItemFunc_4 as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenuItemFunc_4 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn GameOptionsMenu_3_OptionsScreen(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & kButton_Up as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x37 as uint16, game);
        let ref mut fresh68 = *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16);
        *fresh68 = (*fresh68).wrapping_sub(1);
        if *fresh68 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 4 as uint16;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & kButton_Down as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x37 as uint16, game);
        let ref mut fresh69 = *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16);
        *fresh69 = (*fresh69).wrapping_add(1);
        if *fresh69 as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 0 as uint16;
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 11 as uint16;
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & kButton_A as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & kButton_Start as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x38 as uint16, game);
        kGameOptionsMenuItemFuncs
            [*((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) as usize]
            .expect("non-null function pointer")(game);
    }
}
pub unsafe fn GameOptionsMenuItemFunc_0(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5d1) as *mut uint16) as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int
            & kButton_L as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16)
            as ::core::ffi::c_int
            == kGameState_5_FileSelectMap as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 4 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 12 as uint16;
    };
}
pub unsafe fn GameOptionsMenuItemFunc_2_ToggleJapanese(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int) as uint16;
    OptionsMenuFunc4(game);
}
pub unsafe fn OptionsMenuFunc4(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
        OptionsMenuFunc5(0x400 as uint16, 0x288 as uint16, 0x18 as uint16, game);
        OptionsMenuFunc5(0x400 as uint16, 0x2c8 as uint16, 0x18 as uint16, game);
        OptionsMenuFunc5(0 as uint16, 0x348 as uint16, 0x32 as uint16, game);
        OptionsMenuFunc5(0 as uint16, 0x388 as uint16, 0x32 as uint16, game);
    } else {
        OptionsMenuFunc5(0 as uint16, 0x288 as uint16, 0x18 as uint16, game);
        OptionsMenuFunc5(0 as uint16, 0x2c8 as uint16, 0x18 as uint16, game);
        OptionsMenuFunc5(0x400 as uint16, 0x348 as uint16, 0x32 as uint16, game);
        OptionsMenuFunc5(0x400 as uint16, 0x388 as uint16, 0x32 as uint16, game);
    };
}
pub unsafe fn GameOptionsMenuItemFunc_4(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x57) = 3 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 5 as uint16;
}
pub unsafe fn GameOptionsMenu_B_TransitionBackToFileSelect(game: &mut Game) {
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 0x80 as ::core::ffi::c_int
    {
        EnableNMI(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
            kGameState_4_FileSelectMenus as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x727) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn GameOptionsMenu_C_FadeOutOptionsScreenToStart(game: &mut Game) {
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 0x80 as ::core::ffi::c_int
    {
        EnableNMI(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 4 as uint16;
    }
}
pub unsafe fn GameOptionsMenu_4_StartGame(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5d1) as *mut uint16) as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int
            & 0x20 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
            kGameState_5_FileSelectMap as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16)
            as ::core::ffi::c_int
            != kLoadingGameState_5_Main as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16) =
                kGameState_5_FileSelectMap as uint16;
            SaveToSram(
                *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16),
                game,
            );
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16);
        if *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16)
            as ::core::ffi::c_int
            == kGameState_34_CeresGoesBoom as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
                (((0x8bc11bi32) as u16) & (0xffffu16));
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 0 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) =
            kGameState_30_IntroCinematic as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) =
            (((0x8ba395i32) as u16) & (0xffffu16));
        *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
    };
}
pub unsafe fn GameOptionsMenu_5_DissolveOutScreen(game: &mut Game) {
    let mut m: uint16 = 0;
    let mut k: uint16 = 0;
    let mut j: uint16 = 0;
    let mut n: uint16 = 0;
    let mut v1: uint16 = 0;
    HandleFadeOut(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x57) as ::core::ffi::c_int
        != 0xf3 as ::core::ffi::c_int
    {
        let ref mut fresh70 = *(&raw mut game.g_ram as *mut uint8).offset(0x57);
        *fresh70 = ((*fresh70).wrapping_add((16u8)));
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 0x80 as ::core::ffi::c_int
    {
        EnableNMI(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = 0 as uint16;
        let ref mut fresh71 = *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16);
        *fresh71 = (*fresh71).wrapping_add(1);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) != 0 {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16)
                as ::core::ffi::c_int
                & 4 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
                    let mut i: ::core::ffi::c_int = 1023 as ::core::ffi::c_int;
                    while i >= 0 as ::core::ffi::c_int {
                        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                            .pause_menu_map_tilemap[i as usize] =
                            *((&raw mut game.g_ram as *mut uint8).offset(0x19602) as *mut uint16)
                                .offset((i + 9471 as ::core::ffi::c_int) as isize);
                        i -= 1;
                    }
                } else {
                    j = 1023 as uint16;
                    while j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                            .pause_menu_map_tilemap[j as usize] =
                            *((&raw mut game.g_ram as *mut uint8).offset(0x19602) as *mut uint16)
                                .offset(
                                    (j as ::core::ffi::c_int + 8447 as ::core::ffi::c_int) as isize,
                                );
                        j = j.wrapping_sub(1);
                    }
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 0 as uint16;
                OptionsMenuFunc7(game);
                *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 1 as uint16;
                OptionsMenuFunc7(game);
                *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 4 as uint16;
                CreateOptionsMenuObject_(4 as uint16, addr_stru_82F4D0 as uint16, game);
            } else {
                if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) != 0 {
                    k = 1023 as uint16;
                    while k as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x19602) as *mut uint16)
                            .offset(
                                (k as ::core::ffi::c_int + 7423 as ::core::ffi::c_int) as isize,
                            );
                        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                            .pause_menu_map_tilemap[k as usize] = v1;
                        k = k.wrapping_sub(1);
                    }
                } else {
                    m = 1023 as uint16;
                    while m as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x19602) as *mut uint16)
                            .offset(
                                (m as ::core::ffi::c_int + 6399 as ::core::ffi::c_int) as isize,
                            );
                        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                            .pause_menu_map_tilemap[m as usize] = v1;
                        m = m.wrapping_sub(1);
                    }
                }
                CreateOptionsMenuObject_(v1, addr_stru_82F4CA as uint16, game);
                LoadControllerOptionsFromControllerBindings(game);
                OptionsMenuFunc6(game);
            }
        } else {
            n = 1023 as uint16;
            while n as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            {
                (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
                    .pause_menu_map_tilemap[n as usize] =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x19602) as *mut uint16)
                        .offset((n as ::core::ffi::c_int + 5375 as ::core::ffi::c_int) as isize);
                n = n.wrapping_sub(1);
            }
            OptionsMenuFunc4(game);
            let mut v6: uint16 = 0 as uint16;
            CreateOptionsMenuObject_(v6, addr_stru_82F4C4 as uint16, game);
        }
    }
}
pub unsafe fn GameOptionsMenu_6_DissolveInScreen(game: &mut Game) {
    HandleFadeIn(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x57) as ::core::ffi::c_int
        != 3 as ::core::ffi::c_int
    {
        let ref mut fresh72 = *(&raw mut game.g_ram as *mut uint8).offset(0x57);
        *fresh72 = ((*fresh72).wrapping_sub((16u8)));
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
        == 15 as ::core::ffi::c_int
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x57) = 0 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) != 0 {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16)
                as ::core::ffi::c_int
                & 4 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 8 as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 7 as uint16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 0 as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 3 as uint16;
        }
    }
}
static kGameOptionsMenuSpecialSettings: [Option<Func_V>; 3] = unsafe {
    [
        Some(GameOptionsMenuSpecialSettings_0 as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenuSpecialSettings_0 as unsafe fn(&mut Game) -> ()),
        Some(GameOptionsMenuSpecialSettings_2 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn GameOptionsMenu_8_SpecialSettings(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & kButton_Up as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x37 as uint16, game);
        let ref mut fresh75 = *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16);
        *fresh75 = (*fresh75).wrapping_sub(1);
        if *fresh75 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 2 as uint16;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & kButton_Down as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x37 as uint16, game);
        let ref mut fresh76 = *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16);
        *fresh76 = (*fresh76).wrapping_add(1);
        if *fresh76 as ::core::ffi::c_int == 3 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 0 as uint16;
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & (kButton_Start as ::core::ffi::c_int
                | kButton_Left as ::core::ffi::c_int
                | kButton_Right as ::core::ffi::c_int
                | kButton_A as ::core::ffi::c_int)
            != 0 as ::core::ffi::c_int
        {
            QueueSfx1_Max6(0x38 as uint16, game);
            kGameOptionsMenuSpecialSettings
                [*((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) as usize]
                .expect("non-null function pointer")(game);
        }
    } else {
        QueueSfx1_Max6(0x38 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 0 as uint16;
        GameOptionsMenuItemFunc_4(game);
    };
}
pub unsafe fn GameOptionsMenuSpecialSettings_0(game: &mut Game) {
    let mut v0: *mut uint8 = RomPtr_RAM(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x82f0ae)) as *const uint8 as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) as isize),
        game,
    );
    if *(v0 as *mut uint16) != 0 {
        *(v0 as *mut uint16) = 0 as uint16;
    } else {
        *(v0 as *mut uint16) = 1 as uint16;
    }
    OptionsMenuFunc7(game);
}
pub unsafe fn GameOptionsMenuSpecialSettings_2(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 0 as uint16;
    GameOptionsMenuItemFunc_4(game);
}
static g_word_82F149: [uint16; 4] = [0x1e0, 0x220, 0x360, 0x3a0];
static g_word_82F151: [uint16; 4] = [0x1ee, 0x22e, 0x36e, 0x3ae];
pub unsafe fn OptionsMenuFunc7(game: &mut Game) {
    let mut v0: uint16 = ((4u16)
        .wrapping_mul((*((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16))));
    if *(RomPtr_RAM(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x82f0ae)) as *const uint8 as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) as isize),
        game,
    ) as *mut uint16)
        != 0
    {
        OptionsMenuFunc5(
            0 as uint16,
            g_word_82F149[(((4u16).wrapping_mul(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16)),
            )) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as usize],
            0xc as uint16,
            game,
        );
        OptionsMenuFunc5(
            0 as uint16,
            g_word_82F149[((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int) as usize],
            0xc as uint16,
            game,
        );
        OptionsMenuFunc5(
            0x400 as uint16,
            g_word_82F151[(v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize],
            0xc as uint16,
            game,
        );
        OptionsMenuFunc5(
            0x400 as uint16,
            g_word_82F151[((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int) as usize],
            0xc as uint16,
            game,
        );
    } else {
        OptionsMenuFunc5(
            0x400 as uint16,
            g_word_82F149[(((4u16).wrapping_mul(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16)),
            )) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as usize],
            0xc as uint16,
            game,
        );
        OptionsMenuFunc5(
            0x400 as uint16,
            g_word_82F149[((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int) as usize],
            0xc as uint16,
            game,
        );
        OptionsMenuFunc5(
            0 as uint16,
            g_word_82F151[(v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize],
            0xc as uint16,
            game,
        );
        OptionsMenuFunc5(
            0 as uint16,
            g_word_82F151[((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int) as usize],
            0xc as uint16,
            game,
        );
    };
}
static word_82F204: [uint16; 16] = [
    0x20, 0x20, 0x20, 0x20, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x4000, 0x40, 0x40, 0x80, 0x80,
    0x80,
];
pub unsafe fn GameOptionsMenu_7_ControllerSettings(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & kButton_Up as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x37 as uint16, game);
        let ref mut fresh73 = *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16);
        *fresh73 = (*fresh73).wrapping_sub(1);
        let mut v0: uint16 = *fresh73;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if v0 as ::core::ffi::c_int != 6 as ::core::ffi::c_int {
                return;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 10 as uint16;
            return;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 8 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 9 as uint16;
        return;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & kButton_Down as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x37 as uint16, game);
        let mut v1: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x99e)
            as *mut uint16))
            .wrapping_add((1u16)));
        *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = v1;
        if v1 as ::core::ffi::c_int == 7 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 9 as uint16;
            return;
        }
        if v1 as ::core::ffi::c_int == 9 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 10 as uint16;
            return;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) != 0 {
        QueueSfx1_Max6(0x38 as uint16, game);
        static kOptionsMenuControllerFuncs: [Option<Func_V>; 9] = unsafe {
            [
                Some(OptionsMenuControllerFunc_0 as unsafe fn(&mut Game) -> ()),
                Some(OptionsMenuControllerFunc_0 as unsafe fn(&mut Game) -> ()),
                Some(OptionsMenuControllerFunc_0 as unsafe fn(&mut Game) -> ()),
                Some(OptionsMenuControllerFunc_0 as unsafe fn(&mut Game) -> ()),
                Some(OptionsMenuControllerFunc_0 as unsafe fn(&mut Game) -> ()),
                Some(OptionsMenuControllerFunc_0 as unsafe fn(&mut Game) -> ()),
                Some(OptionsMenuControllerFunc_0 as unsafe fn(&mut Game) -> ()),
                Some(OptionsMenuControllerFunc_7 as unsafe fn(&mut Game) -> ()),
                Some(OptionsMenuControllerFunc_8 as unsafe fn(&mut Game) -> ()),
            ]
        };
        kOptionsMenuControllerFuncs
            [*((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) as usize]
            .expect("non-null function pointer")(game);
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x91) as *mut uint16)
        as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) as ::core::ffi::c_int
            == 8 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xde0) as *mut uint16) as ::core::ffi::c_int
            - 16 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        if word_82F204[*((&raw mut game.g_ram as *mut uint8).offset(0xde0) as *mut uint16) as usize]
            as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x91) as *mut uint16)
                as ::core::ffi::c_int
            == word_82F204
                [*((&raw mut game.g_ram as *mut uint8).offset(0xde0) as *mut uint16) as usize]
                as ::core::ffi::c_int
        {
            let ref mut fresh74 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xde0) as *mut uint16);
            *fresh74 = (*fresh74).wrapping_add(1);
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xde0) as *mut uint16) = 0 as uint16;
        }
    }
}
pub unsafe fn OptionsMenuControllerFunc_8(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9b2) as *mut uint16) = 64 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9b4) as *mut uint16) = 128 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9b6) as *mut uint16) = 0x8000 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9b8) as *mut uint16) = 0x4000 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9ba) as *mut uint16) = 0x2000 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9be) as *mut uint16) = 16 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9bc) as *mut uint16) = 32 as uint16;
        LoadControllerOptionsFromControllerBindings(game);
        OptionsMenuFunc6(game);
    }
}
pub unsafe fn OptionsMenuControllerFunc_7(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & (kButton_Start as ::core::ffi::c_int | kButton_A as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
        && OptionsMenuFunc8(game) == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) = 0 as uint16;
        GameOptionsMenuItemFunc_4(game);
    }
}
pub unsafe fn GameOptionsMenu_9_ScrollControllerSettingsDown(game: &mut Game) {
    let ref mut fresh77 = *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16);
    *fresh77 = ((*fresh77).wrapping_add((2u16)));
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) as ::core::ffi::c_int
        == 32 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 7 as uint16;
    }
}
pub unsafe fn GameOptionsMenu_A_ScrollControllerSettingsUp(game: &mut Game) {
    let ref mut fresh78 = *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16);
    *fresh78 = ((*fresh78).wrapping_sub((2u16)));
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 7 as uint16;
    }
}
pub unsafe fn sub_82F296(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
        .offset((v1 + 13 as ::core::ffi::c_int) as isize) = 24 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
        .offset((v1 + 3 as ::core::ffi::c_int) as isize) = 56 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
        .offset((v1 + 11 as ::core::ffi::c_int) as isize) = 3584 as uint16;
}
pub unsafe fn OptionsPreInstr_F2A9(mut v0: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) as ::core::ffi::c_int
        == kGameState_2_GameOptionsMenu as ::core::ffi::c_int
    {
        let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xde2)
            as *mut uint16) as ::core::ffi::c_int;
        let mut v3: uint16 =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x82f2ed)) as *const uint8 as *mut uint16)
                .offset(v2 as isize);
        if v3 != 0 {
            let mut v4: *const uint16 = RomPtr_82(
                (v3 as ::core::ffi::c_int
                    + 4 as ::core::ffi::c_int
                        * *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16)
                            as ::core::ffi::c_int) as uint16_t,
                game,
            ) as *const uint16;
            let mut v5: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset((v5 + 13 as ::core::ffi::c_int) as isize) = *v4;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset((v5 + 3 as ::core::ffi::c_int) as isize) = *v4.offset(1);
        } else {
            let mut v6: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset((v6 + 13 as ::core::ffi::c_int) as isize) = 384 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset((v6 + 3 as ::core::ffi::c_int) as isize) = 16 as uint16;
        }
    } else {
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset((v1 + 15 as ::core::ffi::c_int) as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset((v1 + 17 as ::core::ffi::c_int) as isize) = addr_off_82F4B6 as uint16;
    };
}
pub unsafe fn sub_82F34B(mut j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(
        ((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 13 as ::core::ffi::c_int) as isize,
    ) = 124 as uint16;
    sub_82F369(j, game);
}
pub unsafe fn sub_82F353(mut j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(
        ((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 13 as ::core::ffi::c_int) as isize,
    ) = 132 as uint16;
    sub_82F369(j, game);
}
pub unsafe fn sub_82F35B(mut j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(
        ((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 13 as ::core::ffi::c_int) as isize,
    ) = 128 as uint16;
    sub_82F369(j, game);
}
pub unsafe fn sub_82F363(mut j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(
        ((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 13 as ::core::ffi::c_int) as isize,
    ) = 128 as uint16;
    sub_82F369(j, game);
}
pub unsafe fn sub_82F369(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
        .offset((v1 + 3 as ::core::ffi::c_int) as isize) = 16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
        .offset((v1 + 11 as ::core::ffi::c_int) as isize) = 3584 as uint16;
}
pub unsafe fn OptionsPreInstr_F376(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) as ::core::ffi::c_int
        != kGameState_2_GameOptionsMenu as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) as ::core::ffi::c_int
            == 6 as ::core::ffi::c_int
            && *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
                == 0x80 as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset((v1 + 15 as ::core::ffi::c_int) as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset((v1 + 17 as ::core::ffi::c_int) as isize) = addr_off_82F4B6 as uint16;
    }
}
pub unsafe fn OptionsPreInstr_F3A0(mut k: uint16, game: &mut Game) {
    match *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) as ::core::ffi::c_int
    {
        6 => {
            if *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
                == 0x80 as ::core::ffi::c_int
            {
                let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                    .offset((v1 + 15 as ::core::ffi::c_int) as isize) = 1 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                    .offset((v1 + 17 as ::core::ffi::c_int) as isize) =
                    -(2890 as ::core::ffi::c_int) as uint16;
            }
        }
        9 => {
            let ref mut fresh121 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(
                    ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 3 as ::core::ffi::c_int)
                        as isize,
                );
            *fresh121 = ((*fresh121).wrapping_sub((2u16)));
        }
        10 => {
            let ref mut fresh122 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(
                    ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 3 as ::core::ffi::c_int)
                        as isize,
                );
            *fresh122 = ((*fresh122).wrapping_add((2u16)));
        }
        _ => {}
    };
}
pub unsafe fn OptionsPreInstr_F3E2(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) as ::core::ffi::c_int
        == 6 as ::core::ffi::c_int
        && *(&raw mut game.g_ram as *mut uint8).offset(0x51) as ::core::ffi::c_int
            == 0x80 as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset((v1 + 15 as ::core::ffi::c_int) as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset((v1 + 17 as ::core::ffi::c_int) as isize) =
            -(2890 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn sub_82F404(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset((v1 + 15 as ::core::ffi::c_int) as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset((v1 + 17 as ::core::ffi::c_int) as isize) =
            -(2890 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn sub_82F419(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
        .offset((v1 + 13 as ::core::ffi::c_int) as isize) = 216 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
        .offset((v1 + 3 as ::core::ffi::c_int) as isize) = 16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
        .offset((v1 + 11 as ::core::ffi::c_int) as isize) = 3584 as uint16;
}
pub unsafe fn OptionsPreInstr_F42C(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) as ::core::ffi::c_int
        != 2 as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset((v1 + 15 as ::core::ffi::c_int) as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset((v1 + 17 as ::core::ffi::c_int) as isize) = addr_off_82F4B6 as uint16;
    }
}
pub unsafe fn LoadControllerOptionsFromControllerBindings(game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v0: uint16 = 0 as uint16;
    loop {
        let mut current_block_13: u64;
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        v2 = *((&raw mut game.g_ram as *mut uint8).offset(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x82f54a)) as *const uint8 as *mut uint16)
                .offset(v1 as isize) as isize,
        ) as *mut uint8 as *mut uint16) as int16;
        if v2 as ::core::ffi::c_int & kButton_X as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            current_block_13 = 4528428927647462225;
        } else if v2 as ::core::ffi::c_int & kButton_A as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                .offset((v1 + 13 as ::core::ffi::c_int) as isize) = 1 as uint16;
            current_block_13 = 2720989448508244618;
        } else if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                .offset((v1 + 13 as ::core::ffi::c_int) as isize) = 2 as uint16;
            current_block_13 = 2720989448508244618;
        } else if v2 as ::core::ffi::c_int & kButton_Select as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                .offset((v1 + 13 as ::core::ffi::c_int) as isize) = 3 as uint16;
            current_block_13 = 2720989448508244618;
        } else if v2 as ::core::ffi::c_int & kButton_Y as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                .offset((v1 + 13 as ::core::ffi::c_int) as isize) = 4 as uint16;
            current_block_13 = 2720989448508244618;
        } else if v2 as ::core::ffi::c_int & kButton_L as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                .offset((v1 + 13 as ::core::ffi::c_int) as isize) = 5 as uint16;
            current_block_13 = 2720989448508244618;
        } else if v2 as ::core::ffi::c_int & kButton_R as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            current_block_13 = 4528428927647462225;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                .offset((v1 + 13 as ::core::ffi::c_int) as isize) = 6 as uint16;
            current_block_13 = 2720989448508244618;
        }
        match current_block_13 {
            4528428927647462225 => {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                    .offset((v1 + 13 as ::core::ffi::c_int) as isize) = 0 as uint16;
            }
            _ => {}
        }
        v0 = ((v0).wrapping_add((2u16)));
        if !(((((v0) as i16).wrapping_sub((14i16))) as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
static word_82F575: [Buttons; 9] = [
    kButton_X,
    kButton_A,
    kButton_B,
    kButton_Select,
    kButton_Y,
    kButton_L,
    kButton_R,
    kButton_Left,
    kButton_Right,
];
pub unsafe fn OptionsMenuFunc8(game: &mut Game) -> uint8 {
    let mut v0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut v2: ::core::ffi::c_int = 0;
    loop {
        v2 = v0;
        *((&raw mut game.g_ram as *mut uint8).offset(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x82f54a)) as *const uint8 as *mut uint16)
                .offset((v0 >> 1 as ::core::ffi::c_int) as isize) as isize,
        ) as *mut uint8 as *mut uint16) = word_82F575[*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1b23) as *mut uint16)
            .offset(((v0 >> 1 as ::core::ffi::c_int) + 13 as ::core::ffi::c_int) as isize)
            as usize] as uint16;
        v0 += 2 as ::core::ffi::c_int;
        if !(v2 < 12 as ::core::ffi::c_int) {
            break;
        }
    }
    return 0 as uint8;
}
static g_word_82F6AD: [uint16; 6] = [0, 0xe, 0xe, 0x10, 0x1f, 0x1f];
pub unsafe fn OptionsMenuFunc6(game: &mut Game) {
    let mut v0: uint16 = 0 as uint16;
    let mut v4: uint16 = 0;
    loop {
        v4 = v0;
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v2: uint16 =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x82f639)) as *const uint8 as *mut uint16)
                .offset(v1 as isize);
        let mut v3: *const uint16 = RomPtr_82(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x82f647)) as *const uint8 as *mut uint16)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                        .offset((v1 + 13 as ::core::ffi::c_int) as isize)
                        as isize,
                ),
            game,
        ) as *const uint16;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16 as *mut uint8)
            .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) = *v3;
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(1) as *mut uint16 as *mut uint8)
            .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) = *v3.offset(1);
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(2) as *mut uint16 as *mut uint8)
            .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) = *v3.offset(2);
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(32) as *mut uint16 as *mut uint8)
            .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) = *v3.offset(3);
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(33) as *mut uint16 as *mut uint8)
            .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) = *v3.offset(4);
        *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16)
            .offset(34) as *mut uint16 as *mut uint8)
            .offset(v2 as ::core::ffi::c_int as isize) as *mut uint16) = *v3.offset(5);
        v0 = ((v4).wrapping_add((2u16)));
        if !(((((v4) as i16).wrapping_sub((12i16))) as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(0)
        as ::core::ffi::c_int
        != 5 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(0)
            as ::core::ffi::c_int
            != 6 as ::core::ffi::c_int
    {
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .menu
            .backup_of_io_registers_in_gameover as *mut uint8)
            .offset(46) as *mut uint8 as *mut uint32) =
            *(&raw const g_word_82F6AD as *const uint16 as *mut uint32);
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[665 as ::core::ffi::c_int as usize] =
            g_word_82F6AD[2 as ::core::ffi::c_int as usize];
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .menu
            .field_536 as *mut uint8)
            .offset(56) as *mut uint8 as *mut uint32) =
            *((&raw const g_word_82F6AD as *const uint16).offset(3) as *const uint16
                as *mut uint32);
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[697 as ::core::ffi::c_int as usize] =
            g_word_82F6AD[5 as ::core::ffi::c_int as usize];
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(1)
        as ::core::ffi::c_int
        != 5 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(1)
            as ::core::ffi::c_int
            != 6 as ::core::ffi::c_int
    {
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .menu
            .field_536 as *mut uint8)
            .offset(184) as *mut uint8 as *mut uint32) =
            *(&raw const g_word_82F6AD as *const uint16 as *mut uint32);
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[761 as ::core::ffi::c_int as usize] =
            g_word_82F6AD[2 as ::core::ffi::c_int as usize];
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .menu
            .menu_tilemap as *mut uint8)
            .offset(46) as *mut uint8 as *mut uint32) =
            *((&raw const g_word_82F6AD as *const uint16).offset(3) as *const uint16
                as *mut uint32);
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[793 as ::core::ffi::c_int as usize] =
            g_word_82F6AD[5 as ::core::ffi::c_int as usize];
    }
}
pub unsafe fn OptionsMenuControllerFunc_0(game: &mut Game) {
    let mut v0: uint16 = 12 as uint16;
    while word_82F575[(v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        as ::core::ffi::c_uint
        & *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_uint
        == 0 as ::core::ffi::c_uint
    {
        v0 = ((v0).wrapping_sub((2u16)));
        if v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return;
        }
    }
    let mut r18: uint16 = (v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int) as uint16;
    if 2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) as ::core::ffi::c_int
        - 12 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        v1 = 0 as uint16;
    }
    let mut v2: uint16 = v1;
    let mut i: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(
            ((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 13 as ::core::ffi::c_int)
                as isize,
        ) as ::core::ffi::c_int
            == r18 as ::core::ffi::c_int
        {
            break;
        }
        v2 = ((v2).wrapping_add((2u16)));
        if (((v2) as i16).wrapping_sub((14i16))) as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            v2 = 0 as uint16;
        }
        i -= 1;
    }
    let mut v4: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x99e) as *mut uint16) as ::core::ffi::c_int;
    let mut r20: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
        .offset((v4 + 13 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
        .offset((v4 + 13 as ::core::ffi::c_int) as isize) = r18;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(
        ((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 13 as ::core::ffi::c_int) as isize,
    ) = r20;
    OptionsMenuFunc6(game);
}
