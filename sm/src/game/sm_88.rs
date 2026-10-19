pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_87::*;
pub use crate::game::constants::bank_8f::*;
pub use crate::game::constants::samus_poses::*;
use crate::game::sm_80::{
    CheckBossBitForCurArea, CheckEventHappened, HandleMusicQueue, QueueMusic_Delayed8,
    QueueSfx1_Max15, QueueSfx1_Max6, QueueSfx2_Max6, SetEventHappened,
};
use crate::game::sm_84::SpawnHardcodedPlm;
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_87::SpawnAnimtiles;
use crate::game::sm_8b::ConfigureTitleSequenceGradientHDMA;
use crate::game::sm_90::{CallSomeSamusCode, Hdmaobj_CrystalFlash};
use crate::game::sm_91::{
    CalculateXrayHdmaTableInner, CanXrayShowBlocks, HdmaobjPreInstr_XraySetup,
    InitializeSuitPickupHdma, ResponsibleForXrayStandupGlitch, SamusFunc_F433,
    Samus_LoadSuitPalette, Samus_SetAnimationFrameIfPoseChanged, Xray_SetupStage1_FreezeTimeBackup,
    Xray_SetupStage2_ReadBg1_2ndScreen, Xray_SetupStage3_ReadBg1_1stScreen, Xray_SetupStage4,
    Xray_SetupStage5, Xray_SetupStage6, Xray_SetupStage7, Xray_SetupStage8_SetBackdropColor,
};
use crate::game::sm_94::kSinCosTable8bit_Sext;
use crate::game::sm_a9::MotherBrain_CalculateRainbowBeamHdma;
use crate::game_state::Game;
use crate::rtl::{mov24, Load24, Mult8x8, RomPtr, Unreachable, WriteReg, WriteRegWord};
use crate::rtl::{AddToHiLo, SetHiLo};
pub use crate::snes::registers::*;

pub use crate::types::EnemyData;
pub use crate::types::{
    __int16_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int8, int8_t, uint16,
    uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t, Func_U8, Func_V, Func_Y_V, Func_Y_Y,
    LongPtr, SnesRegs, VoidP, VramWriteEntry,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct HdmaScrollEntry {
    pub top_pos: uint16,
    pub scroll_subspeed: uint16,
    pub scroll_speed: uint16,
    pub hdma_data_table_entry: VoidP,
}
pub use crate::types::Consts_87;
pub use crate::types::Consts_88;
pub use crate::types::{Consts_86, ExpandingSquareTransitionHdma, Ram4000, Ram4000_Backups};
pub use crate::types::{SamusPose, SpawnHdmaObject_Args};
pub const addr_word_88DF71: Consts_88 = 57201;
pub const addr_word_88DF6B: Consts_88 = 57195;
pub const addr_word_88DF65: Consts_88 = 57189;
pub const addr_word_88DF5F: Consts_88 = 57183;
pub const addr_off_88ADA6: Consts_88 = 44454;
pub const addr_off_88AD9C: Consts_88 = 44444;
pub const addr_word_88A8E8: Consts_88 = 43240;
pub const addr_byte_889F06: Consts_88 = 40710;
pub const addr_kPowerBombExplosionShapeDef0: Consts_88 = 37446;
pub const addr_loc_889180: Consts_88 = 37248;
pub const addr_locret_88858A: Consts_88 = 34186;
pub use crate::types::Consts_89;
pub const addr_kIndirectHdmaTable_PowerBombExplodeRight: Consts_89 = 41217;
pub const addr_kIndirectHdmaTable_PowerBombExplodeLeft: Consts_89 = 38912;
pub use crate::types::Consts_8F;
pub use crate::types::SpawnHardcodedPlmArgs;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: ::core::ffi::c_int = 1;
pub const false_0: ::core::ffi::c_int = 0;
#[inline]
unsafe fn RomPtr_88(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0x880000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomBankBase(bank: uint8, game: &mut Game) -> *const uint8 {
    RomPtr(
        (((bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) + 0x8000 as ::core::ffi::c_int)
            as uint32_t,
        game,
    )
    .offset(-(0x8000 as ::core::ffi::c_int as isize))
}
#[inline]
unsafe fn gVramWriteEntry(a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry
}
pub const fnnullsub_56: uint32 = 8491739;
pub const fnDoorTransitionFunction_FadeOutScreen: ::core::ffi::c_int = 0x82e2db;
pub const fnDoorTransition_FadeInScreenAndFinish: ::core::ffi::c_int = 0x82e737;
pub const fnLayerBlendingHandler: ::core::ffi::c_int = 0x888000;
pub const fnPauseHook_Empty: ::core::ffi::c_int = 0x8883e1;
pub const fnnullsub_293: ::core::ffi::c_int = 0x8884b8;
pub const fnHdmaobjInstr_Delete: uint32 = 8947049;
pub const fnHdmaobjInstr_SetPreInstr: uint32 = 8947056;
pub const fnHdmaobjInstr_ClearPreInstr: uint32 = 8947076;
pub const fnHdmaobjInstr_CallFarFunc: uint32 = 8947124;
pub const fnHdmaobjInstr_Goto: uint32 = 8947180;
pub const fnHdmaobjInstr_GotoRel: uint32 = 8947185;
pub const fnHdmaobjInstr_DecrementAndGoto: uint32 = 8947207;
pub const fnHdmaobjInstr_DecrementAndGotoRel: uint32 = 8947215;
pub const fnHdmaobjInstr_SetTimer: uint32 = 8947222;
pub const fnHdmaobjInstr_SetHdmaControl: uint32 = 8947234;
pub const fnHdmaobjInstr_SetHdmaTarget: uint32 = 8947255;
pub const fnHdmaobjInstr_SetHdmaTablePtr: uint32 = 8947276;
pub const fnHdmaobjInstr_SetHdmaTableBank: uint32 = 8947285;
pub const fnHdmaobjInstr_SetIndirectHdmaDataBank: uint32 = 8947306;
pub const fnHdmaobjInstr_Sleep: uint32 = 8947330;
pub const fnHdmaobjPreInstr_Xray: uint32 = 8947439;
pub const fnHdmaobjPreInstr_XrayFunc0_NoBeam: uint32 = 8947506;
pub const fnHdmaobjPreInstr_XrayFunc1_BeamWidening: uint32 = 8947540;
pub const fnHdmaobjPreInstr_XrayFunc2_FullBeam: uint32 = 8947627;
pub const fnHdmaobjPreInstr_XrayFunc3_DeactivateBeam: uint32 = 8948020;
pub const fnHdmaobjPreInstr_XrayFunc4_DeactivateBeam: uint32 = 8948154;
pub const fnHdmaobjPreInstr_XrayFunc5_DeactivateBeam: uint32 = 8948232;
pub const fnHdmaobj_PreExplodeWhite: uint32 = 8948500;
pub const fnHdmaobj_PreExplodeYellow: uint32 = 8948530;
pub const fnHdmaobj_ExplodeYellow: uint32 = 8948537;
pub const fnHdmaobj_ExplodeWhite: uint32 = 8948551;
pub const fnHdmaobj_CleanUpTryCrystalFlash: uint32 = 8948558;
pub const fnHdmaobjPreInstr_PowerBombExplode_SetWindowConf: uint32 = 8948623;
pub const fnHdmaobjPreInstr_PowerBombExplode_Stage5_Afterglow: uint32 = 8948632;
pub const fnHdmaobjPreInstr_PowerBombExplode_ExplosionYellow: uint32 = 8949225;
pub const fnHdmaobjPreInstr_PowerBombExplode_ExplosionWhite: uint32 = 8949426;
pub const fnHdmaobjPreInstr_PowerBombExplode_PreExplosionWhite: uint32 = 8949983;
pub const fnHdmaobjPreInstr_PowerBombExplode_PreExplosionYellow: uint32 = 8950184;
pub const fnCrystalFlashSetupPart1: uint32 = 8954596;
pub const fnCrystalFlashSetupPart2: uint32 = 8954633;
pub const fnCrystalFlashCleanup: uint32 = 8954647;
pub const fnHdmaobjPreInstr_CrystalFlash_CustomLayerBlend: uint32 = 8954681;
pub const fnHdmaobjPreInstr_CrystalFlash_Stage2_AfterGlow: uint32 = 8954717;
pub const fnHdmaobjPreInstr_CrystalFlash_Stage1_Explosion: uint32 = 8955218;
pub const fnHdmaobjPreInstr_FxType22_BG3Yscroll: uint32 = 8955459;
pub const fnHdmaobjInstr_SetFlagB: uint32 = 8955500;
pub const fnHdmaobjPreInstr_BG3Xscroll: uint32 = 8955507;
pub const fnHdmaobjPreInstr_SkyLandBG2Xscroll: uint32 = 8957362;
pub const fnHdmaobjPreInstr_SkyLandBG2Xscroll2: uint32 = 8957370;
pub const fnHdmaobjPreInstr_SkyLandBG2XscrollInner: uint32 = 8957378;
pub const fnHdmaobjPreInstr_FirefleaBG3XScroll: uint32 = 8958140;
pub const fnFxRisingFunction_LavaAcid: ::core::ffi::c_int = 0x88b343;
pub const fnFxRisingFunction_LavaAcid_WaitToRise: ::core::ffi::c_int = 0x88b367;
pub const fnFxRisingFunction_LavaAcid_Raising: ::core::ffi::c_int = 0x88b382;
pub const fnHdmaobjInstr_B3A9: uint32 = 8958889;
pub const fnHdmaobjPreInstr_LavaAcidBG3YScroll: uint32 = 8958896;
pub const fnHdmaobjInstr_SetFlagB_Copy: uint32 = 8959182;
pub const fnHdmaobjPreInstr_LavaAcidBG2YScroll: uint32 = 8959189;
pub const fnFxRisingFunction_C428_WaterNormal: ::core::ffi::c_int = 0x88c428;
pub const fnFxRisingFunction_WaterWaitToRise: ::core::ffi::c_int = 0x88c44c;
pub const fnFxRisingFunction_WaterRising: ::core::ffi::c_int = 0x88c458;
pub const fnHdmaobjInstr_SetFlagB_Copy2: uint32 = 8963175;
pub const fnHdmaobjPreInstr_WaterBG3XScroll: uint32 = 8963214;
pub const fnHdmaobjInstr_SetFlagB_Copy3: uint32 = 8963458;
pub const fnHdmaobjPreInstr_WaterBG2XScroll: uint32 = 8963465;
pub const fnHdmaobjPreInstr_WaterBG2XScroll_Func2: uint32 = 8963556;
pub const fnHdmaobjPreInstr_WaterBG2XScroll_Func1: uint32 = 8963638;
pub const fnsub_88D916: uint32 = 8968470;
pub const fnHdmaobjInstr_SetVideoMode1: uint32 = 8968521;
pub const fnHdmaobjInstr_1938_RandomNumber: uint32 = 8968577;
pub const fnHdmaobjPreInstr_RainBg3Scroll: uint32 = 8968609;
pub const fnHdmaobjPreInstr_SporesBG3Xscroll: uint32 = 8968775;
pub const fnnullsub_112: uint32 = 8969006;
pub const fnHdmaobjPreInstr_FogBG3Scroll: uint32 = 8969014;
pub const fnHdmaobjPreInstr_CheckLotsOfEventsHappened: uint32 = 8969175;
pub const fnHdmaobjPreInstr_DC23: uint32 = 8969251;
pub const fnHdmaobjPreInstr_DC69: uint32 = 8969321;
pub const fnHdmaobjPreInstr_DCBA: uint32 = 8969402;
pub const fnHdmaobjInstr_GotoIfEventHappened: uint32 = 8969419;
pub const fnHdmaobjPreInstr_BombTorizoHazeColorMathBgColor: uint32 = 8969539;
pub const fnHdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyAlive: uint32 = 8969744;
pub const fnHdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyDead: uint32 = 8969749;
pub const fnHdmaobjPreInstr_HazeColorMathSubscreen_FadingIn: ::core::ffi::c_int = 0x88de2d;
pub const fnHdmaobjPreInstr_HazeColorMathSubscreen_FadedIn: ::core::ffi::c_int = 0x88de74;
pub const fnHdmaobjPreInstr_HazeColorMathSubscreen_FadingOut: ::core::ffi::c_int = 0x88de96;
pub const fnnullsub_113: uint32 = 8970129;
pub const fnnullsub_114: uint32 = 8970130;
pub const fnHdmaobjPreInstr_DF94: uint32 = 8970132;
pub const fnHdmaobjPreInstr_VariaSuitPickup: uint32 = 8970278;
pub const fnHdmaobjPreInstr_GravitySuitPickup: uint32 = 8970332;
pub const fnHdmaobjPreInstr_E449: uint32 = 8971337;
pub const fnHdmaobjInstr_E4BD: uint32 = 8971453;
pub const fnHdmaobjPreInstr_E567: uint32 = 8971623;
pub const fnnullsub_357: uint32 = 8972070;
pub const fnInitializeRainbowBeam: uint32 = 8972135;
pub const fnHdmaobjPreInstr_E7BC: uint32 = 8972220;
pub const fnHdmaobjInstr_InitMorphBallEyeBeamHdma: u32 = 0x88e917;
pub const fnHdmaobjPreInstr_MorphBallEyeBeamWidening: u32 = 0x88e9e6;
pub const fnHdmaobjPreInstr_MorphBallEyeBeamFull: u32 = 0x88ea3c;
pub const fnHdmaobjPreInstr_MorphBallEyeBeamFading: u32 = 0x88eacb;
pub const fnHdmaobjInsr_ConfigTitleSequenceGradientHDMA: uint32 = 8973215;
pub const fnHdmaobjPreInstr_Backdrop_TitleSequenceGradient: uint32 = 8973232;
pub const fnHdmaobjPreInstr_ColorMathControlB_TitleGradient: uint32 = 8973266;
pub const fnHdmaobjPreInstr_IntroCutsceneCrossfade: uint32 = 8973341;
pub const fnHdmaobjInstr_EC9F_ClearVars: uint32 = 8973471;
pub const fnHdmaobjPreInstr_ECB6: uint32 = 8973494;
pub const fnCinematicFunctionOpening: ::core::ffi::c_int = 0x8b9b68;
pub const fnXray_SetupStage1_FreezeTimeBackup: uint32 = 9554681;
pub const fnXray_SetupStage2_ReadBg1_2ndScreen: uint32 = 9554716;
pub const fnXray_SetupStage3_ReadBg1_1stScreen: uint32 = 9554775;
pub const fnXray_SetupStage4: uint32 = 9554830;
pub const fnXray_SetupStage5: uint32 = 9556179;
pub const fnXray_SetupStage6: uint32 = 9556339;
pub const fnXray_SetupStage7: uint32 = 9556384;
pub const fnHdmaobjPreInstr_XraySetup: uint32 = 9556607;
pub const fnXray_SetupStage8_SetBackdropColor: uint32 = 9556668;
pub const fnInitializeSuitPickupHdma: uint32 = 9557650;
pub const fnnullsub_309: uint32 = 10785852;
pub fn nullsub_9(j: uint16, game: &mut Game) -> uint16 {
    j
}
pub unsafe fn InitializeLayerBlending(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 51 as uint8;
}
pub unsafe fn LayerBlendFunc_4_PhantoonIntro(j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 4 as uint8;
    j
}
pub unsafe fn LayerBlendFunc_6(j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 20 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 39 as uint8;
    j
}
pub unsafe fn LayerBlendFunc_8(j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 20 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 34 as uint8;
    j
}
pub unsafe fn LayerBlendFunc_A(j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 50 as uint8;
    j
}
pub unsafe fn LayerBlendFunc_C(j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = -(94 as ::core::ffi::c_int) as uint8;
    j
}
pub fn nullsub_10(j: uint16, game: &mut Game) -> uint16 {
    j
}
pub unsafe fn LayerBlendFunc_10(j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 4 as uint8;
    j
}
pub unsafe fn LayerBlendFunc_14(j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = -(77 as ::core::ffi::c_int) as uint8;
    j
}
pub unsafe fn LayerBlendFunc_16(mut j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut j as *mut uint8).offset(0) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 6 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = -(79 as ::core::ffi::c_int) as uint8;
    j
}
pub unsafe fn LayerBlendFunc_1A(mut j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut j as *mut uint8).offset(0) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 21 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 53 as uint8;
    j
}
pub unsafe fn LayerBlendFunc_1C(j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 21 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 85 as uint8;
    j
}
pub unsafe fn LayerBlendFunc_18(mut j: uint16, game: &mut Game) -> uint16 {
    j = 2 as uint16;
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x84) as ::core::ffi::c_int
        & 0x30 as ::core::ffi::c_int)
        == 0x30i32
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 36 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 19 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 4 as uint8;
    }
    j
}
pub fn nullsub_11(j: uint16, game: &mut Game) -> uint16 {
    j
}
pub unsafe fn LayerBlendFunc_26(j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 119 as uint8;
    j
}
pub unsafe fn LayerBlendFunc_28(j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = -(77 as ::core::ffi::c_int) as uint8;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 37 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    }
    j
}
pub unsafe fn LayerBlendFunc_2A(j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = -(77 as ::core::ffi::c_int) as uint8;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 38 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 66 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    }
    j
}
pub unsafe fn LayerBlendFunc_2C(j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 0 as uint8;
    j
}
pub unsafe fn LayerBlendFunc_2E(j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = -(77 as ::core::ffi::c_int) as uint8;
    j
}
pub unsafe fn LayerBlendFunc_32(j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 68 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = -(78 as ::core::ffi::c_int) as uint8;
    j
}
pub unsafe fn LayerBlendFunc_34(mut j: uint16, game: &mut Game) -> uint16 {
    j = 6 as uint16;
    j
}
pub unsafe fn LayerBlendFunc_24(j: uint16, game: &mut Game) -> uint16 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 16 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 51 as uint8;
    j
}
pub unsafe fn HandleLayerBlendingXrayCanShowBlocks(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = -(56 as ::core::ffi::c_int) as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 8 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 3 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 34 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) =
        (*(&raw mut game.g_ram as *mut uint8).offset(0x71) as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            | 0x73 as ::core::ffi::c_int) as uint8;
}
pub unsafe fn HandleLayerBlendingXrayCantShowBlocks(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 8 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 3 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 34 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) =
        (*(&raw mut game.g_ram as *mut uint8).offset(0x71) as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            | 0x61 as ::core::ffi::c_int) as uint8;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79b) as *mut uint16) as ::core::ffi::c_int
        == addr_kRoom_cefb as ::core::ffi::c_int
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    }
}
pub unsafe fn HandleLayerBlendingXrayFirefleaRoom(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 8 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 3 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = -(77 as ::core::ffi::c_int) as uint8;
}
pub unsafe fn LayerBlendPowerBombFunc_0(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 8 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 55 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 4 as uint8;
}
pub unsafe fn LayerBlendPowerBombFunc_4(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 8 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 55 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 6 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 17 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 6 as uint8;
}
pub unsafe fn LayerBlendPowerBombFunc_6(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 8 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 49 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6b) = 4 as uint8;
}
static kLayerBlendFuncTable: [Option<Func_Y_Y>; 27] = unsafe {
    [
        Some(nullsub_9 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(nullsub_9 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_4_PhantoonIntro as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_6 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_8 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_A as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_C as unsafe fn(uint16, &mut Game) -> uint16),
        Some(nullsub_10 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_10 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_10 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_14 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_16 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_18 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_1A as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_1C as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_18 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(nullsub_11 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_14 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_24 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_26 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_28 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_2A as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_2C as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_2E as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_18 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_32 as unsafe fn(uint16, &mut Game) -> uint16),
        Some(LayerBlendFunc_34 as unsafe fn(uint16, &mut Game) -> uint16),
    ]
};
pub unsafe fn LayerBlendingHandler(game: &mut Game) {
    let mut v0: uint16 = 0 as uint16;
    let v1: uint8 = *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) as uint8;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) as uint8 != 0 {
        InitializeLayerBlending(game);
        v0 = kLayerBlendFuncTable[(v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(v0, game);
    }
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16)
            as ::core::ffi::c_int
            & 0x4000 as ::core::ffi::c_int)
            != 0i32
        {
            HandleLayerBlendingXrayCanShowBlocks(game);
        } else if (*((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16)
            as ::core::ffi::c_int
            & 0x2000 as ::core::ffi::c_int)
            != 0i32
        {
            HandleLayerBlendingXrayCantShowBlocks(game);
        } else if (*((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16)
            as ::core::ffi::c_int
            & 0x1000 as ::core::ffi::c_int)
            != 0i32
        {
            HandleLayerBlendingXrayFirefleaRoom(game);
        }
    } else {
        HandleLayerBlendingPowerBomb(v0, game);
    };
}
static kLayerBlendPowerBombFuncs: [Option<Func_V>; 4] = unsafe {
    [
        Some(LayerBlendPowerBombFunc_0 as unsafe fn(&mut Game) -> ()),
        Some(LayerBlendPowerBombFunc_0 as unsafe fn(&mut Game) -> ()),
        Some(LayerBlendPowerBombFunc_4 as unsafe fn(&mut Game) -> ()),
        Some(LayerBlendPowerBombFunc_6 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn HandleLayerBlendingPowerBomb(mut j: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79b) as *mut uint16) as ::core::ffi::c_int
        == addr_kRoom_a66a as ::core::ffi::c_int
    {
        j = 6 as uint16;
    }
    kLayerBlendPowerBombFuncs[(j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn EnableHdmaObjects(game: &mut Game) {
    let ref mut fresh31 = *((&raw mut game.g_ram as *mut uint8).offset(0x18b0) as *mut uint16);
    *fresh31 = (*fresh31) | (0x8000u16);
}
pub unsafe fn DisableHdmaObjects(game: &mut Game) {
    let ref mut fresh30 = *((&raw mut game.g_ram as *mut uint8).offset(0x18b0) as *mut uint16);
    *fresh30 = (*fresh30 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn WaitUntilEndOfVblankAndClearHdma(game: &mut Game) {
    WriteReg(MDMAEN as uint16, 0 as uint8, game);
    WriteReg(HDMAEN as uint16, 0 as uint8, game);
    sub_8882AC(game);
}
pub unsafe fn sub_8882AC(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x85) = 0 as uint8;
    let mut i: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
    while i >= 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x18b4) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn InitializeSpecialEffectsForNewRoom(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x607) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x609) as *mut uint16) = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79b) as *mut uint16) as ::core::ffi::c_int
        == addr_kRoom_9804 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x79b) as *mut uint16) as ::core::ffi::c_int
            == addr_kRoom_96ba as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x79b) as *mut uint16) as ::core::ffi::c_int
            == addr_kRoom_b32e as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x79b) as *mut uint16) as ::core::ffi::c_int
            == addr_kRoom_b457 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x79b) as *mut uint16) as ::core::ffi::c_int
            == addr_kRoom_dd58 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x79b) as *mut uint16) as ::core::ffi::c_int
            == addr_kRoom_dede as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x609) as *mut uint16) =
            -(1 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x5f7) as *mut uint16) = 0 as uint16;
    let mut i: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
    while i != 0x80i32 {
        WriteReg(
            (i + 17152 as ::core::ffi::c_int) as SnesRegs as uint16,
            0 as uint8,
            game,
        );
        WriteReg(
            (i + 17153 as ::core::ffi::c_int) as SnesRegs as uint16,
            0x13 as uint8,
            game,
        );
        WriteReg(
            (i + 17154 as ::core::ffi::c_int) as SnesRegs as uint16,
            0 as uint8,
            game,
        );
        WriteReg(
            (i + 17155 as ::core::ffi::c_int) as SnesRegs as uint16,
            0 as uint8,
            game,
        );
        WriteReg(
            (i + 17156 as ::core::ffi::c_int) as SnesRegs as uint16,
            0 as uint8,
            game,
        );
        WriteReg(
            (i + 17157 as ::core::ffi::c_int) as SnesRegs as uint16,
            0 as uint8,
            game,
        );
        WriteReg(
            (i + 17158 as ::core::ffi::c_int) as SnesRegs as uint16,
            0 as uint8,
            game,
        );
        WriteReg(
            (i + 17160 as ::core::ffi::c_int) as SnesRegs as uint16,
            0 as uint8,
            game,
        );
        WriteReg(
            (i + 17161 as ::core::ffi::c_int) as SnesRegs as uint16,
            0 as uint8,
            game,
        );
        i += 16 as ::core::ffi::c_int;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x195c) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x195e) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1960) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1962) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcad8) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcada) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcadc) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcade) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x783) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x941) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e6f) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e6d) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1988) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcee) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1964) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x196e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1976) as *mut uint16) = 0x8000 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x197e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1980) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1974) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1970) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1972) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1982) as *mut uint16) = 2 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb9) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xbb) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x917) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x919) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa9) as *mut uint16) = 0 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x601) as *mut LongPtr)).addr
        as *mut uint8)
        .offset(1) as *mut VoidP) = -(30720 as ::core::ffi::c_int) as VoidP;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x604) as *mut LongPtr)).addr
        as *mut uint8)
        .offset(1) as *mut VoidP) = -(30720 as ::core::ffi::c_int) as VoidP;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x601) as *mut LongPtr)).addr =
        ((0x8883e1i32) as u16);
    (*((&raw mut game.g_ram as *mut uint8).offset(0x604) as *mut LongPtr)).addr =
        ((0x8883e1i32) as u16);
    WriteReg(WMADDL as uint16, 0xf0 as uint8, game);
    WriteReg(WMADDM as uint16, 0xff as uint8, game);
    WriteReg(WMADDH as uint16, 1 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x85) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x57) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x69) = 19 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x72) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x5e) = 4 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 73 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x5a) = 90 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x5b) = 90 as uint8;
}
unsafe fn SpawnHdmaObjectInner(
    k: uint16,
    p: *mut uint16,
    r18: uint16,
    r20: uint16,
    r24: uint16,
    game: &mut Game,
) -> uint16 {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x18f0) as *mut uint16).offset(v2 as isize) =
        ((0x8884b8i32) as u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x18fc) as *mut uint16).offset(v2 as isize) =
        136 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16).offset(v2 as isize) =
        *p.offset(1);
    *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16).offset(v2 as isize) =
        1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1908) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x192c) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1938) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x18b4) as *mut uint16).offset(v2 as isize) =
        u16::swap_bytes(r18);
    *((&raw mut game.g_ram as *mut uint8).offset(0x18c0) as *mut uint16).offset(v2 as isize) =
        (r24) | (r20);
    WriteRegWord(
        (r20 as ::core::ffi::c_int + DMAP0 as ::core::ffi::c_int) as SnesRegs as uint16,
        *p,
        game,
    );
    k
}
pub unsafe fn SpawnHdmaObject(db: uint8, p: *const ::core::ffi::c_void, game: &mut Game) -> uint16 {
    let mut r18: uint16 = 1024 as uint16;
    let mut r20: uint16 = 32 as uint16;
    let mut v3: uint16 = 0 as uint16;
    loop {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x18b4) as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            == 0
        {
            return SpawnHdmaObjectInner(
                v3,
                p as *mut uint16,
                r18,
                r20,
                ((db as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16,
                game,
            );
        }
        let v4: bool = r18 as ::core::ffi::c_int >> 15 as ::core::ffi::c_int != 0;
        r18 = (r18).wrapping_mul(2u16);
        if v4 {
            return -(1 as ::core::ffi::c_int) as uint16;
        }
        r20 = (r20).wrapping_add(16u16);
        v3 = (v3).wrapping_add(2u16);
        if v3 == 12u16 {
            return -(1 as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn SpawnHdmaObjectToSlot0xA(db: uint8, p: *const ::core::ffi::c_void, game: &mut Game) {
    SpawnHdmaObjectInner(
        0xa as uint16,
        p as *mut uint16,
        0x8000 as uint16,
        0x70 as uint16,
        ((db as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn HdmaObjectHandler(game: &mut Game) {
    let mut v1: int8 = 0;
    HandleMusicQueue(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) == 0
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16)
            as ::core::ffi::c_int
            & 0x4000 as ::core::ffi::c_int)
            != 0i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16) = 0x8000 as uint16;
        static unk_8884D5: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
            field_0: 0x40 as uint8,
            field_1: 0x28 as uint8,
            addr: 0x8ace as VoidP,
        };
        static unk_8884DD: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
            field_0: 0x40 as uint8,
            field_1: 0x29 as uint8,
            addr: 0x8b80 as VoidP,
        };
        SpawnHdmaObject(
            0x88 as uint8,
            &raw const unk_8884D5 as *const ::core::ffi::c_void,
            game,
        );
        SpawnHdmaObject(
            0x88 as uint8,
            &raw const unk_8884DD as *const ::core::ffi::c_void,
            game,
        );
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1982) as *mut uint16);
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x18b0) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x85) = 0 as uint8;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0x18b2) as *mut uint16) = i as uint16;
            v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x18b4) as *mut uint16 as *mut uint8)
                .offset(i as isize) as int8;
            if v1 != 0 {
                let ref mut fresh58 = *(&raw mut game.g_ram as *mut uint8).offset(0x85);
                *fresh58 = (*fresh58) | ((v1) as u8);
                HdmaobjInstructionHandler(i as uint8, game);
                i = *((&raw mut game.g_ram as *mut uint8).offset(0x18b2) as *mut uint16)
                    as ::core::ffi::c_int;
            }
            i += 2 as ::core::ffi::c_int;
            if !(i != 12i32) {
                break;
            }
        }
        LayerBlendingHandler(game);
    }
}
pub unsafe fn HdmaobjInstructionHandler(k: uint8, game: &mut Game) {
    let kh: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    CallHdmaobjPreInstr(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x18fc) as *mut uint16).offset(kh as isize)
            as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x18f0) as *mut uint16)
                .offset(kh as isize) as ::core::ffi::c_int) as uint32,
        k as uint16,
        game,
    );
    let ref mut fresh59 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16).offset(kh as isize);
    let fresh60 = *fresh59;
    *fresh59 = (*fresh59).wrapping_sub(1);
    if fresh60 == 1u16 {
        let base: *const uint8 = RomBankBase(
            *((&raw mut game.g_ram as *mut uint8).offset(0x18c0) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize)
                .offset(1),
            game,
        );
        let mut p: *const uint8 = base.offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16).offset(kh as isize)
                as ::core::ffi::c_int as isize,
        );
        while *(p as *mut uint16) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            p = CallHdmaobjInstr(
                ((*(p as *mut uint16)) as u32) | (0x880000u32),
                k as uint16,
                p.offset(2),
                game,
            );
            if (p as uintptr_t) < 0x10000 as ::core::ffi::c_int as uintptr_t {
                if p.is_null() {
                    return;
                }
                p = base.offset(p as uintptr_t as isize);
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16).offset(kh as isize) =
            *(p as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x18d8) as *mut uint16).offset(kh as isize) =
            *(p.offset(2) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16).offset(kh as isize) =
            p.offset(4).offset_from(base) as ::core::ffi::c_long as uint16;
    }
}
pub unsafe fn HdmaobjInstr_Delete(k: uint16, hdp: *const uint8, game: &mut Game) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x18b4) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    ::core::ptr::null::<uint8>()
}
pub unsafe fn HdmaobjInstr_SetPreInstr(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x18f0) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(hdp as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x18fc) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) = *hdp.offset(2);
    hdp.offset(3)
}
pub unsafe fn HdmaobjInstr_ClearPreInstr(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x18f0) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        addr_locret_88858A as uint16;
    hdp
}
pub unsafe fn CallHdmaobjInstrFunc(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        8948500 => {
            Hdmaobj_PreExplodeWhite(game);
        }
        8948530 => {
            Hdmaobj_PreExplodeYellow(game);
        }
        8948537 => {
            Hdmaobj_ExplodeYellow(game);
        }
        8948551 => {
            Hdmaobj_ExplodeWhite(game);
        }
        8948558 => {
            Hdmaobj_CleanUpTryCrystalFlash(k, game);
        }
        8954596 => {
            CrystalFlashSetupPart1(game);
        }
        8954633 => {
            CrystalFlashSetupPart2(game);
        }
        8954647 => {
            CrystalFlashCleanup(k, game);
        }
        8970129 => (),
        8970130 => (),
        8972070 => (),
        8972135 => {
            InitializeRainbowBeam(game);
        }
        9554681 => {
            Xray_SetupStage1_FreezeTimeBackup(k, game);
        }
        9554716 => {
            Xray_SetupStage2_ReadBg1_2ndScreen(game);
        }
        9554775 => {
            Xray_SetupStage3_ReadBg1_1stScreen(game);
        }
        9554830 => {
            Xray_SetupStage4(game);
        }
        9556179 => {
            Xray_SetupStage5(game);
        }
        9556339 => {
            Xray_SetupStage6(game);
        }
        9556384 => {
            Xray_SetupStage7(game);
        }
        9556668 => {
            Xray_SetupStage8_SetBackdropColor(game);
        }
        9557650 => {
            InitializeSuitPickupHdma(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn HdmaobjInstr_CallFarFunc(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    CallHdmaobjInstrFunc(Load24(hdp as *mut LongPtr), k, game);
    hdp.offset(3)
}
pub unsafe fn HdmaobjInstr_Goto(k: uint16, hdp: *const uint8) -> *const uint8 {
    *(hdp as *mut uint16) as uintptr_t as *const uint8
}
pub unsafe fn HdmaobjInstr_GotoRel(k: uint16, hdp: *const uint8) -> *const uint8 {
    hdp.offset(*hdp as int8 as ::core::ffi::c_int as isize)
}
pub unsafe fn HdmaobjInstr_DecrementAndGoto(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh63 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1908) as *mut uint16).offset(v2 as isize);
    let fresh64 = *fresh63;
    *fresh63 = (*fresh63).wrapping_sub(1);
    if fresh64 == 1u16 {
        hdp.offset(2)
    } else {
        *(hdp as *mut uint16) as uintptr_t as *const uint8
    }
}
pub unsafe fn HdmaobjInstr_DecrementAndGotoRel(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh61 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1908) as *mut uint16).offset(v2 as isize);
    let fresh62 = *fresh61;
    *fresh61 = (*fresh61).wrapping_sub(1);
    if fresh62 == 1u16 {
        hdp.offset(1)
    } else {
        hdp.offset(*hdp as int8 as ::core::ffi::c_int as isize)
    }
}
pub unsafe fn HdmaobjInstr_SetTimer(k: uint16, hdp: *const uint8, game: &mut Game) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1908) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) = *hdp;
    hdp.offset(1)
}
pub unsafe fn HdmaobjInstr_SetHdmaControl(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    WriteReg(
        (*(((&raw mut game.g_ram as *mut uint8).offset(0x18c0) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int
            + DMAP0 as ::core::ffi::c_int) as SnesRegs as uint16,
        *hdp,
        game,
    );
    hdp.offset(1)
}
pub unsafe fn HdmaobjInstr_SetHdmaTarget(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    WriteReg(
        (*(((&raw mut game.g_ram as *mut uint8).offset(0x18c0) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int
            + BBAD0 as ::core::ffi::c_int) as SnesRegs as uint16,
        *hdp,
        game,
    );
    hdp.offset(1)
}
pub unsafe fn HdmaobjInstr_SetHdmaTablePtr(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x18d8) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(hdp as *mut uint16);
    hdp.offset(2)
}
pub unsafe fn HdmaobjInstr_SetHdmaTableBank(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    WriteReg(
        (*(((&raw mut game.g_ram as *mut uint8).offset(0x18c0) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int
            + A1B0 as ::core::ffi::c_int) as SnesRegs as uint16,
        *hdp,
        game,
    );
    hdp.offset(1)
}
pub unsafe fn HdmaobjInstr_SetIndirectHdmaDataBank(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    WriteReg(
        (*(((&raw mut game.g_ram as *mut uint8).offset(0x18c0) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int
            + DAS00 as ::core::ffi::c_int) as SnesRegs as uint16,
        *hdp,
        game,
    );
    hdp.offset(1)
}
pub unsafe fn HdmaobjInstr_Sleep(k: uint16, hdp: *const uint8, game: &mut Game) -> *const uint8 {
    let base: *const uint8 = RomBankBase(
        *((&raw mut game.g_ram as *mut uint8).offset(0x18c0) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize)
            .offset(1),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (hdp.offset_from(base) as ::core::ffi::c_long - 2 as ::core::ffi::c_long) as uint16;
    ::core::ptr::null::<uint8>()
}
pub unsafe fn RaiseOrLowerFx(game: &mut Game) -> uint8 {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        return 1 as uint8;
    }
    let r24_r22: uint32 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16)
        as int16 as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint32;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        let mut v2: uint16 = (r24_r22.wrapping_add(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0x1976) as *mut uint16) as uint32,
        ) >> 16 as ::core::ffi::c_int) as uint16;
        let ref mut fresh13 = *((&raw mut game.g_ram as *mut uint8).offset(0x1976) as *mut uint16);
        *fresh13 = (*fresh13 as uint32).wrapping_add(r24_r22) as uint16;
        if (v2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            v2 = -(1 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16) = v2;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16) >= v2 {
            0 as uint8
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x1976) as *mut uint16) = 0 as uint16;
            1 as uint8
        }
    } else {
        let mut v0: uint16 = (r24_r22.wrapping_add(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0x1976) as *mut uint16) as uint32,
        ) >> 16 as ::core::ffi::c_int) as uint16;
        let ref mut fresh14 = *((&raw mut game.g_ram as *mut uint8).offset(0x1976) as *mut uint16);
        *fresh14 = (*fresh14 as uint32).wrapping_add(r24_r22) as uint16;
        if (v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            v0 = 0 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16) = v0;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16) >= v0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x1976) as *mut uint16) = 0 as uint16;
        }
        (*((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16) >= v0) as uint8
    }
}
pub unsafe fn HdmaobjPreInstr_XrayFunc0_NoBeam(k: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x9b6) as *mut uint16) as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int)
        != 0i32
    {
        CalculateXrayHdmaTable(game);
        let ref mut fresh137 = *((&raw mut game.g_ram as *mut uint8).offset(0xa7a) as *mut uint16);
        *fresh137 = (*fresh137).wrapping_add(1);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7a) as *mut uint16) = 3 as uint16;
    };
}
pub unsafe fn HdmaobjPreInstr_XrayFunc1_BeamWidening(k: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x9b6) as *mut uint16) as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int)
        != 0i32
    {
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xa7c) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xa7e) as *mut uint16,
            2048 as uint32,
        );
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xa86) as *mut uint16,
            (*((&raw mut game.g_ram as *mut uint8).offset(0xa7c) as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0xa7e) as *mut uint16) as uint32,
        );
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16) as ::core::ffi::c_int
            - 11 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa86) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16) = 10 as uint16;
            let ref mut fresh136 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xa7a) as *mut uint16);
            *fresh136 = (*fresh136).wrapping_add(1);
        }
        CalculateXrayHdmaTable(game);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7a) as *mut uint16) = 3 as uint16;
    };
}
pub unsafe fn HdmaobjPreInstr_XrayFunc2_FullBeam(k: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x9b6) as *mut uint16) as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int)
        != 0i32
    {
        HandleMovingXrayUpDown(game);
        CalculateXrayHdmaTable(game);
    } else {
        let ref mut fresh135 = *((&raw mut game.g_ram as *mut uint8).offset(0xa7a) as *mut uint16);
        *fresh135 = (*fresh135).wrapping_add(1);
    };
}
pub unsafe fn HandleMovingXrayUpDown(game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x9aa) as *mut uint16) as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int)
        != 0i32
    {
        MoveXrayUp(game);
    } else if (*((&raw mut game.g_ram as *mut uint8).offset(0x9ac) as *mut uint16)
        as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int)
        != 0i32
    {
        MoveXrayDown(game);
    }
}
pub unsafe fn MoveXrayUp(game: &mut Game) {
    let mut v0: bool = false;
    let mut v1: int16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16) as ::core::ffi::c_int
        - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16)
            != *((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16)
        {
            if ((*((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16)) as i16)
                .wrapping_sub(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16)) as i16,
                )
                < 0i16
                || {
                    v0 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16) != 0u16;
                    let ref mut fresh52 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16);
                    *fresh52 = (*fresh52).wrapping_sub(1);
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16)
                        as ::core::ffi::c_int
                        - (!v0 as ::core::ffi::c_int
                            + *((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16)
                                as ::core::ffi::c_int)) as int16)
                        < 0i16
                }
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16) =
                    *((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16);
            }
        }
    } else if (*((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16)
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16) as ::core::ffi::c_int)
        != 256i32
    {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16)
                as ::core::ffi::c_int
            - 256 as ::core::ffi::c_int) as int16)
            >= 0i16
            || {
                v1 = (((*((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16)
                    as uint32)
                    << 16 as ::core::ffi::c_int
                    | *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16)
                        as uint32)
                    .wrapping_add(
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16)
                            as uint32)
                            << 16 as ::core::ffi::c_int
                            | 1 as uint16 as uint32,
                    )
                    >> 16 as ::core::ffi::c_int) as int16;
                let ref mut fresh53 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16);
                *fresh53 = (*fresh53).wrapping_add(1);
                v1 != 256i16 && (v1).wrapping_sub(256i16) >= 0i16
            }
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16) = (256u16)
                .wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16));
        }
    }
}
pub unsafe fn MoveXrayDown(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v1: bool = false;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16) as ::core::ffi::c_int
        - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16)
                as ::core::ffi::c_int)
            != 128i32
        {
            if ((*((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16)
                as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16)
                    as ::core::ffi::c_int
                - 128 as ::core::ffi::c_int) as int16)
                >= 0i16
                || {
                    v0 = (((*((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16)
                        as uint32)
                        << 16 as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16)
                            as uint32)
                        .wrapping_add(
                            (*((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16)
                                as uint32)
                                << 16 as ::core::ffi::c_int
                                | 1 as uint16 as uint32,
                        )
                        >> 16 as ::core::ffi::c_int) as int16;
                    let ref mut fresh50 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16);
                    *fresh50 = (*fresh50).wrapping_add(1);
                    v0 != 128i16 && (v0).wrapping_sub(128i16) >= 0i16
                }
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16) = (128u16)
                    .wrapping_sub(
                        *((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16),
                    );
            }
        }
    } else if (*((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16) as ::core::ffi::c_int)
        != 128i32
    {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16)
                as ::core::ffi::c_int
            - 128 as ::core::ffi::c_int) as int16)
            < 0i16
            || {
                v1 = *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16) != 0u16;
                let ref mut fresh51 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16);
                *fresh51 = (*fresh51).wrapping_sub(1);
                *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16)
                    as ::core::ffi::c_int
                    - (!v1 as ::core::ffi::c_int
                        + *((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16)
                            as ::core::ffi::c_int)
                    != 128i32
                    && ((*((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16)
                        as ::core::ffi::c_int
                        - (!v1 as ::core::ffi::c_int
                            + *((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16)
                                as ::core::ffi::c_int)
                        - 128 as ::core::ffi::c_int) as int16)
                        < 0i16
            }
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16))
                    .wrapping_add(128u16);
        }
    }
}
pub unsafe fn CalculateXrayHdmaTable(game: &mut Game) {
    let mut v0: int16 = 0;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        v0 = (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                as ::core::ffi::c_int
            - 3 as ::core::ffi::c_int) as int16;
    } else {
        v0 = (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                as ::core::ffi::c_int
            + 3 as ::core::ffi::c_int) as int16;
    }
    let mut v1: uint16 = 0;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) == 5u8 {
        v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int
            - 12 as ::core::ffi::c_int) as uint16;
    } else {
        v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int
            - 16 as ::core::ffi::c_int) as uint16;
    }
    let current_block_11: u64;
    if v0 < 0i16 {
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) != 4u8 {
            current_block_11 = 13131151160068275183;
        } else {
            current_block_11 = 10599921512955367680;
        }
    } else {
        if (v0).wrapping_sub(256i16) < 0i16 {
            CalculateXrayHdmaTableInner(
                v0 as uint16,
                v1,
                *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16),
                *((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16),
                false_0 != 0,
                (&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16,
                game,
            );
            return;
        }
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) != 8u8 {
            current_block_11 = 13131151160068275183;
        } else {
            current_block_11 = 10599921512955367680;
        }
    }
    match current_block_11 {
        10599921512955367680 => {}
        _ => {
            CalculateXrayHdmaTableInner(
                v0 as uint16,
                v1,
                *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16),
                *((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16),
                true_0 != 0,
                (&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16,
                game,
            );
            return;
        }
    }
    let mut i: ::core::ffi::c_int = 510 as ::core::ffi::c_int;
    while i >= 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 255 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn HdmaobjPreInstr_XrayFunc3_DeactivateBeam(k: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v4: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa88) as *mut LongPtr,
        0x980001 as uint32,
    );
    *(((&raw mut game.g_ram as *mut uint8).offset(0xa8a) as *mut uint16 as *mut uint8).offset(1)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa8c) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa8e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa90) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16).offset(0) = 255 as uint16;
    v1 = 4096 as int16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x196e) as *mut uint16) != 36u16 {
        v1 = 0x2000 as int16;
        if CanXrayShowBlocks(game) {
            v1 = 0x4000 as int16;
        }
    }
    let ref mut fresh133 = *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16);
    *fresh133 = (*fresh133) | ((v1) as u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(0) = 0 as uint16;
    let v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x18b2)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(v2 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16).offset(v2 as isize);
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x192c) as *mut uint16 as *mut uint8).offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x18b2) as *mut uint16)
                as ::core::ffi::c_int as isize,
        );
    let v3: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)) as i16)
        .wrapping_sub(240i16)
        < 0i16
    {
        v4 = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*v4).size = 2048 as uint16;
        (*v4).src.addr =
            (((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000 as *mut uint8)
                .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
                + 4096 as ::core::ffi::c_long) as VoidP;
        *(&raw mut (*v4).src.bank as *mut uint16) = 126 as uint16;
        (*v4).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            (v3).wrapping_add(7u16);
        let ref mut fresh134 = *((&raw mut game.g_ram as *mut uint8).offset(0xa7a) as *mut uint16);
        *fresh134 = (*fresh134).wrapping_add(1);
    }
}
pub unsafe fn HdmaobjPreInstr_XrayFunc4_DeactivateBeam(k: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v3: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    v1 = 4096 as int16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x196e) as *mut uint16) != 36u16 {
        v1 = 0x2000 as int16;
        if CanXrayShowBlocks(game) {
            v1 = 0x4000 as int16;
        }
    }
    let ref mut fresh131 = *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16);
    *fresh131 = (*fresh131) | ((v1) as u16);
    let v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)) as i16)
        .wrapping_sub(240i16)
        < 0i16
    {
        v3 = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*v3).size = 2048 as uint16;
        (*v3).src.addr =
            (((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000 as *mut uint8)
                .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
                + 6144 as ::core::ffi::c_long) as VoidP;
        *(&raw mut (*v3).src.bank as *mut uint16) = 126 as uint16;
        (*v3).vram_dst = (((*(&raw mut game.g_ram as *mut uint8).offset(0x59)
            as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int)
            + 1024 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            (v2).wrapping_add(7u16);
        let ref mut fresh132 = *((&raw mut game.g_ram as *mut uint8).offset(0xa7a) as *mut uint16);
        *fresh132 = (*fresh132).wrapping_add(1);
    }
}
pub unsafe fn HdmaobjPreInstr_XrayFunc5_DeactivateBeam(k: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    v1 = 4096 as int16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x196e) as *mut uint16) != 36u16 {
        v1 = 0x2000 as int16;
        if CanXrayShowBlocks(game) {
            v1 = 0x4000 as int16;
        }
    }
    let ref mut fresh130 = *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16);
    *fresh130 = (*fresh130) | ((v1) as u16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7a) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7c) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7e) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa82) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa84) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa86) as *mut uint16) = 0 as uint16;
        mov24(
            (&raw mut game.g_ram as *mut uint8).offset(0xa88) as *mut LongPtr,
            0x980001 as uint32,
        );
        *(((&raw mut game.g_ram as *mut uint8).offset(0xa8a) as *mut uint16 as *mut uint8).offset(1)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa8c) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa8e) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa90) as *mut uint16) = 0 as uint16;
        ResponsibleForXrayStandupGlitch(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x18b4) as *mut uint16).offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x18b2) as *mut uint16)
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) = 0 as uint16;
        QueueSfx1_Max6(0xa as uint16, game);
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x196e) as *mut uint16) as uint8) != 36u8 {
            *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
            *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
        }
        let mut i: ::core::ffi::c_int = 510 as ::core::ffi::c_int / 2 as ::core::ffi::c_int;
        while i >= 0i32 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16)
                .offset(i as isize) = 0xff as uint16;
            i -= 1;
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa04) as *mut uint16) != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x9d2) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa04) as *mut uint16) = 0 as uint16;
        }
    }
}
static kHdmaobjPreInstr_XrayFuncs: [Option<Func_Y_V>; 6] = unsafe {
    [
        Some(HdmaobjPreInstr_XrayFunc0_NoBeam as unsafe fn(uint16, &mut Game) -> ()),
        Some(HdmaobjPreInstr_XrayFunc1_BeamWidening as unsafe fn(uint16, &mut Game) -> ()),
        Some(HdmaobjPreInstr_XrayFunc2_FullBeam as unsafe fn(uint16, &mut Game) -> ()),
        Some(HdmaobjPreInstr_XrayFunc3_DeactivateBeam as unsafe fn(uint16, &mut Game) -> ()),
        Some(HdmaobjPreInstr_XrayFunc4_DeactivateBeam as unsafe fn(uint16, &mut Game) -> ()),
        Some(HdmaobjPreInstr_XrayFunc5_DeactivateBeam as unsafe fn(uint16, &mut Game) -> ()),
    ]
};
pub unsafe fn HdmaobjPreInstr_Xray(k: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    v1 = 4096 as int16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x196e) as *mut uint16) != 36u16 {
        v1 = 0x2000 as int16;
        if CanXrayShowBlocks(game) {
            v1 = 0x4000 as int16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) as *mut uint8
                as *mut uint16) = 0x27 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) as *mut uint8
                as *mut uint16) = 0x47 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) as *mut uint8
                as *mut uint16) = 0x87 as uint16;
        }
    }
    let ref mut fresh138 = *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16);
    *fresh138 = (*fresh138) | ((v1) as u16);
    kHdmaobjPreInstr_XrayFuncs
        [*((&raw mut game.g_ram as *mut uint8).offset(0xa7a) as *mut uint16) as usize]
        .expect("non-null function pointer")(
        (2u16).wrapping_mul(*((&raw mut game.g_ram as *mut uint8).offset(0xa7a) as *mut uint16)),
        game,
    );
}
pub unsafe fn SpawnPowerBombExplosion(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16) = 0x4000 as uint16;
    } else {
        QueueSfx1_Max15(1 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16) = 0x8000 as uint16;
        static unk_888ABA: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
            field_0: 0x40 as uint8,
            field_1: 0x28 as uint8,
            addr: 0x8ace as VoidP,
        };
        static unk_888AC2: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
            field_0: 0x40 as uint8,
            field_1: 0x29 as uint8,
            addr: 0x8b80 as VoidP,
        };
        SpawnHdmaObject(
            0x88 as uint8,
            &raw const unk_888ABA as *const ::core::ffi::c_void,
            game,
        );
        SpawnHdmaObject(
            0x88 as uint8,
            &raw const unk_888AC2 as *const ::core::ffi::c_void,
            game,
        );
    };
}
static kPowerBombPreExplosionRadiusSpeed: uint16 = 0x3000 as uint16;
static kPowerBombPreExplosionRadiusAccel: uint16 = 0x80 as uint16;
static g_word_888B96: uint16 = 3 as uint16;
static kPowerBombExplosionRadiusSpeed: uint16 = 0 as uint16;
static kPowerBombExplosionRadiusAccel: uint16 = 0x30 as uint16;
pub unsafe fn Hdmaobj_PreExplodeWhite(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xc606) = -(1 as ::core::ffi::c_int) as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xc607) = 0 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcec) as *mut uint16) = 1024 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf0) as *mut uint16) =
        kPowerBombPreExplosionRadiusSpeed;
}
pub unsafe fn Hdmaobj_PreExplodeYellow(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf2) as *mut uint16) =
        addr_byte_889F06 as uint16;
}
pub unsafe fn Hdmaobj_ExplodeYellow(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16) = 1024 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf0) as *mut uint16) =
        kPowerBombExplosionRadiusSpeed;
}
pub unsafe fn Hdmaobj_ExplodeWhite(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf2) as *mut uint16) =
        addr_kPowerBombExplosionShapeDef0 as uint16;
}
pub unsafe fn Hdmaobj_CleanUpTryCrystalFlash(v0: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
        != *((&raw mut game.g_ram as *mut uint8).offset(0xce2) as *mut uint16)
        || *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
            != *((&raw mut game.g_ram as *mut uint8).offset(0xce4) as *mut uint16)
        || Hdmaobj_CrystalFlash(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcee) as *mut uint16) = 0 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16) = 0 as uint16;
    let v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x18b4) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x18b4) as *mut uint16)
        .offset((v1 + 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcec) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16) = 0 as uint16;
    CallSomeSamusCode(0x1e as uint16, game);
}
pub unsafe fn HdmaobjPreInstr_PowerBombExplode_SetWindowConf(k: uint16, game: &mut Game) {
    let ref mut fresh129 = *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16);
    *fresh129 = (*fresh129) | (0x8000u16);
}
pub unsafe fn HdmaobjPreInstr_PowerBombExplode_Stage5_Afterglow(k: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        let v1: ::core::ffi::c_int = k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh125 = *((&raw mut game.g_ram as *mut uint8).offset(0x1908) as *mut uint16)
            .offset(v1 as isize);
        *fresh125 = (*fresh125).wrapping_sub(1);
        if (*fresh125 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            let ref mut fresh126 = *((&raw mut game.g_ram as *mut uint8).offset(0x1938)
                as *mut uint16 as *mut uint8)
                .offset(k as uint8 as ::core::ffi::c_int as isize);
            let fresh127 = *fresh126;
            *fresh126 = (*fresh126).wrapping_sub(1);
            if fresh127 == 1u8 {
                *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16)
                    .offset(v1 as isize) = 1 as uint16;
                let ref mut fresh128 = *((&raw mut game.g_ram as *mut uint8).offset(0x18cc)
                    as *mut uint16)
                    .offset(v1 as isize);
                *fresh128 = (*fresh128).wrapping_add(2u16);
            } else {
                if (*(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0)
                    as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int)
                    != 0i32
                {
                    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) =
                        ((*(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0)
                            as ::core::ffi::c_int
                            & 0x1f as ::core::ffi::c_int)
                            - 1 as ::core::ffi::c_int
                            | 0x20 as ::core::ffi::c_int) as uint8;
                }
                if (*(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1)
                    as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int)
                    != 0i32
                {
                    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) =
                        ((*(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1)
                            as ::core::ffi::c_int
                            & 0x1f as ::core::ffi::c_int)
                            - 1 as ::core::ffi::c_int
                            | 0x40 as ::core::ffi::c_int) as uint8;
                }
                if (*(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2)
                    as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int)
                    != 0i32
                {
                    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) =
                        ((*(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2)
                            as ::core::ffi::c_int
                            & 0x1f as ::core::ffi::c_int)
                            - 1 as ::core::ffi::c_int
                            | 0x80 as ::core::ffi::c_int) as uint8;
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0x1908) as *mut uint16
                    as *mut uint8)
                    .offset(k as uint8 as ::core::ffi::c_int as isize) = g_word_888B96 as uint8;
            }
        }
    }
}
pub unsafe fn CalculatePowerBombHdma_LeftOfScreen(
    mut k: uint16,
    mut j: *const uint8,
    game: &mut Game,
) {
    loop {
        let w: uint8 = *j;
        let c: uint8 = *((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16) as uint8;
        if u8::overflowing_add(w, c).1 {
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc506)
                .offset(k as isize) = (w).wrapping_add(c);
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc406)
                .offset(k as isize) = 0 as uint8;
        } else {
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc506)
                .offset(k as isize) = 0 as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc406)
                .offset(k as isize) = 1 as uint8;
        }
        j = j.offset(1);
        k = k.wrapping_add(1);
        if !(k != 192u16) {
            break;
        }
    }
}
pub unsafe fn CalculatePowerBombHdma_OnScreen(mut k: uint16, mut j: *const uint8, game: &mut Game) {
    loop {
        let w: uint8 = *j;
        if w == 0 {
            break;
        }
        let c: uint8 = *((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16) as uint8;
        let right: uint8 = (if u8::overflowing_add(c, w).1 as ::core::ffi::c_int != 0 {
            255 as ::core::ffi::c_int
        } else {
            c as ::core::ffi::c_int + w as ::core::ffi::c_int
        }) as uint8;
        let left: uint8 = (if c >= w {
            c as ::core::ffi::c_int - w as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint8;
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0xc406)
            .offset(k as isize) = left;
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0xc506)
            .offset(k as isize) = right;
        j = j.offset(1);
        k = k.wrapping_add(1);
        if !(k != 192u16) {
            break;
        }
    }
}
pub unsafe fn CalculatePowerBombHdma_RightOfScreen(
    mut k: uint16,
    mut j: *const uint8,
    game: &mut Game,
) {
    loop {
        let w: uint8 = *j;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16) as uint8) < w {
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc406)
                .offset(k as isize) = ((*((&raw mut game.g_ram as *mut uint8).offset(0xce6)
                as *mut uint16)) as u8)
                .wrapping_sub(w);
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc506)
                .offset(k as isize) = -(1 as ::core::ffi::c_int) as uint8;
        } else {
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc406)
                .offset(k as isize) = -(1 as ::core::ffi::c_int) as uint8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc506)
                .offset(k as isize) = -(2 as ::core::ffi::c_int) as uint8;
        }
        j = j.offset(1);
        k = k.wrapping_add(1);
        if !(k != 192u16) {
            break;
        }
    }
}
pub unsafe fn CalculatePowerBombHdmaObjectTablePtrs(k: uint16, game: &mut Game) {
    let mut v1: uint16 = 0;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xce2) as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) as ::core::ffi::c_int
        + 256 as ::core::ffi::c_int) as uint16)
        >= 0x300u16
        || {
            *((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xce2) as *mut uint16)
                    as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                        as ::core::ffi::c_int
                    + 256 as ::core::ffi::c_int) as uint16;
            v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0xce4) as *mut uint16)
                as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                    as ::core::ffi::c_int
                + 256 as ::core::ffi::c_int) as uint16;
            v1 >= 0x300u16
        }
    {
        v1 = 0 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xce8) as *mut uint16) =
        ((v1 as ::core::ffi::c_int ^ 0x3ff as ::core::ffi::c_int) - 256 as ::core::ffi::c_int)
            as uint16;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        == 0i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xce8) as *mut uint16) = 0 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x18d8) as *mut uint16).offset(
        ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 0 as ::core::ffi::c_int) as isize,
    ) = (3 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0xce8) as *mut uint16) as ::core::ffi::c_int
        + addr_kIndirectHdmaTable_PowerBombExplodeLeft as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x18d8) as *mut uint16).offset(
        ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int) as isize,
    ) = (3 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0xce8) as *mut uint16) as ::core::ffi::c_int
        + addr_kIndirectHdmaTable_PowerBombExplodeRight as ::core::ffi::c_int) as uint16;
}

pub unsafe fn CalculatePowerBombHdmaScaled_LeftOfScreen(
    mut k: uint16,
    mut j: uint16,
    multval: uint8,
    game: &mut Game,
) -> uint16 {
    let mut v2: int8 = 0;
    let mut v6: uint8 = 0;
    let mut v7: uint8 = 0;
    loop {
        let r20: uint8 = (Mult8x8(
            multval,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const uint8 as *mut uint8)
                .offset((j as uint8 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        v2 = *((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16) as int8;
        let Reg: uint8 = (Mult8x8(
            multval,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const uint8 as *mut uint8)
                .offset(j as uint8 as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let v4: bool = u8::overflowing_add(Reg, v2 as uint8).1;
        let v5: uint8 = (Reg).wrapping_add((v2) as u8);
        if v4 {
            v6 = v5;
            v7 = 0 as uint8;
        } else {
            v6 = 0 as uint8;
            v7 = -(1 as ::core::ffi::c_int) as uint8;
        }
        loop {
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc406)
                .offset(k as uint8 as isize) = v7;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc506)
                .offset(k as uint8 as isize) = v6;
            if (k as uint8) == r20 {
                break;
            }
            *(&raw mut k as *mut uint8).offset(0) = ((k) as u8).wrapping_sub(1u8);
        }
        *(&raw mut j as *mut uint8).offset(0) = ((j) as u8).wrapping_add(1u8);
        if !((j as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) == 0i32) {
            break;
        }
    }
    game.k_out = *(&raw mut k as *mut uint8).offset(0) as uint16;
    ((v6 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | v7 as ::core::ffi::c_int) as uint16
}
pub unsafe fn CalculatePowerBombHdmaScaled_OnScreen(
    mut k: uint16,
    mut j: uint16,
    multval: uint8,
    game: &mut Game,
) -> uint16 {
    let mut v2: int8 = 0;
    let mut v5: int8 = 0;
    let mut v6: uint8 = 0;
    let mut v9: uint8 = 0;
    loop {
        let r20: uint8 = (Mult8x8(
            multval,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const uint8 as *mut uint8)
                .offset((j as uint8 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        v2 = *((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16) as int8;
        let Reg: uint8 = (Mult8x8(
            multval,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const uint8 as *mut uint8)
                .offset(j as uint8 as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let mut v4: bool = u8::overflowing_add(Reg, v2 as uint8).1;
        v5 = ((Reg) as i8).wrapping_add(v2);
        if v4 {
            v5 = -(1 as ::core::ffi::c_int) as int8;
        }
        v6 = v5 as uint8;
        let v7: uint8 =
            *((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16) as uint8;
        let v8: uint8 = Reg;
        v4 = v7 < v8;
        v9 = (v7).wrapping_sub(v8);
        if v4 {
            v9 = 0 as uint8;
        }
        loop {
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc406)
                .offset(k as uint8 as isize) = v9;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc506)
                .offset(k as uint8 as isize) = v6;
            if (k as uint8) == r20 {
                break;
            }
            *(&raw mut k as *mut uint8).offset(0) = ((k) as u8).wrapping_sub(1u8);
        }
        *(&raw mut j as *mut uint8).offset(0) = ((j) as u8).wrapping_add(1u8);
        if !((j as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) == 0i32) {
            break;
        }
    }
    game.k_out = *(&raw mut k as *mut uint8).offset(0) as uint16;
    ((v6 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | v9 as ::core::ffi::c_int) as uint16
}
pub unsafe fn CalculatePowerBombHdmaScaled_RightOfScreen(
    mut k: uint16,
    mut j: uint16,
    multval: uint8,
    game: &mut Game,
) -> uint16 {
    let mut v5: int8 = 0;
    let mut v6: int8 = 0;
    let mut v7: int8 = 0;
    let mut v10: int8 = 0;
    let mut v8: uint8 = 0;
    let mut v9: uint8 = 0;
    loop {
        let r20: uint8 = (Mult8x8(
            multval,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const uint8 as *mut uint8)
                .offset((j as uint8 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let v2: uint8 =
            *((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16) as uint8;
        let Reg: uint8 = (Mult8x8(
            multval,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const uint8 as *mut uint8)
                .offset(j as uint8 as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let v4: bool = v2 < Reg;
        v5 = ((v2) as i8).wrapping_sub((Reg) as i8);
        if v4 {
            v6 = v5;
            v7 = -(1 as ::core::ffi::c_int) as int8;
        } else {
            v6 = -(1 as ::core::ffi::c_int) as int8;
            v7 = 0 as int8;
        }
        v10 = v7;
        v8 = v6 as uint8;
        v9 = v10 as uint8;
        loop {
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc406)
                .offset(k as uint8 as isize) = v8;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc506)
                .offset(k as uint8 as isize) = v9;
            if (k as uint8) == r20 {
                break;
            }
            *(&raw mut k as *mut uint8).offset(0) = ((k) as u8).wrapping_sub(1u8);
        }
        *(&raw mut j as *mut uint8).offset(0) = ((j) as u8).wrapping_add(1u8);
        if !((j as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) == 0i32) {
            break;
        }
    }
    game.k_out = *(&raw mut k as *mut uint8).offset(0) as uint16;
    ((v9 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | v8 as ::core::ffi::c_int) as uint16
}
pub unsafe fn HdmaobjPreInstr_PowerBombExplode_ExplosionYellow(k: uint16, game: &mut Game) {
    let mut v2: uint16 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        return;
    }
    CalculatePowerBombHdmaObjectTablePtrs(k, game);
    let kk: ::core::ffi::c_int = Mult8x8(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16)
            as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint8,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a286)) as *const uint8 as *mut uint8)
            .offset(0),
    ) as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int
        != 0
    {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16)
            as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int)
            == 1i32
        {
            v2 = CalculatePowerBombHdmaScaled_OnScreen(
                kk as uint16,
                96 as uint16,
                ((*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16)
                    as ::core::ffi::c_int
                    & 0xff00 as ::core::ffi::c_int)
                    >> 8 as ::core::ffi::c_int) as uint8,
                game,
            );
        } else {
            v2 = CalculatePowerBombHdmaScaled_RightOfScreen(
                kk as uint16,
                96 as uint16,
                ((*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16)
                    as ::core::ffi::c_int
                    & 0xff00 as ::core::ffi::c_int)
                    >> 8 as ::core::ffi::c_int) as uint8,
                game,
            );
        }
    } else {
        v2 = CalculatePowerBombHdmaScaled_LeftOfScreen(
            kk as uint16,
            96 as uint16,
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16)
                as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint8,
            game,
        );
    }
    let mut i: ::core::ffi::c_int = game.k_out as ::core::ffi::c_int;
    loop {
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0xc406)
            .offset(i as isize) = v2 as uint8;
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0xc506)
            .offset(i as isize) = ((v2 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint8;
        i -= 1;
        if !((i & 0x80 as ::core::ffi::c_int) == 0i32) {
            break;
        }
    }
    let mut v3: ::core::ffi::c_int = kk + 1 as ::core::ffi::c_int;
    while v3 != 0xc0i32 {
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0xc406)
            .offset(v3 as isize) = 255 as uint8;
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0xc506)
            .offset(v3 as isize) = 0 as uint8;
        v3 += 1;
    }
    let t: ::core::ffi::c_int = 3 as ::core::ffi::c_int
        * ((*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16)
            as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int
            >> 3 as ::core::ffi::c_int);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const uint8 as *mut uint8)
            .offset((t + 0 as ::core::ffi::c_int) as isize))
            | (0x20u8);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const uint8 as *mut uint8)
            .offset((t + 1 as ::core::ffi::c_int) as isize))
            | (0x40u8);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const uint8 as *mut uint8)
            .offset((t + 2 as ::core::ffi::c_int) as isize))
            | (0x80u8);
    let ref mut fresh122 = *((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16);
    *fresh122 = (*fresh122)
        .wrapping_add(*((&raw mut game.g_ram as *mut uint8).offset(0xcf0) as *mut uint16));
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16) < 0x8600u16 {
        let ref mut fresh123 = *((&raw mut game.g_ram as *mut uint8).offset(0xcf0) as *mut uint16);
        *fresh123 = (*fresh123).wrapping_add(kPowerBombExplosionRadiusAccel);
    } else {
        let v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16).offset(v7 as isize) =
            1 as uint16;
        let ref mut fresh124 = *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16)
            .offset(v7 as isize);
        *fresh124 = (*fresh124).wrapping_add(2u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1908) as *mut uint16).offset(v7 as isize) =
            0 as uint16;
    };
}
pub unsafe fn HdmaobjPreInstr_PowerBombExplode_ExplosionWhite(k: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        return;
    }
    CalculatePowerBombHdmaObjectTablePtrs(k, game);
    let v1: *const uint8 = RomPtr_88(
        *((&raw mut game.g_ram as *mut uint8).offset(0xcf2) as *mut uint16),
        game,
    );
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int
        != 0
    {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16)
            as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int)
            == 1i32
        {
            CalculatePowerBombHdma_OnScreen(0 as uint16, v1, game);
        } else {
            CalculatePowerBombHdma_RightOfScreen(0 as uint16, v1, game);
        }
    } else {
        CalculatePowerBombHdma_LeftOfScreen(0 as uint16, v1, game);
    }
    let t: ::core::ffi::c_int = 3 as ::core::ffi::c_int
        * ((*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16)
            as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int
            >> 3 as ::core::ffi::c_int);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const uint8 as *mut uint8)
            .offset(t as isize))
            | (0x20u8);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const uint8 as *mut uint8)
            .offset((t + 1 as ::core::ffi::c_int) as isize))
            | (0x40u8);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const uint8 as *mut uint8)
            .offset((t + 2 as ::core::ffi::c_int) as isize))
            | (0x80u8);
    let ref mut fresh118 = *((&raw mut game.g_ram as *mut uint8).offset(0xcf2) as *mut uint16);
    *fresh118 = (*fresh118).wrapping_add(192u16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcf2) as *mut uint16) as ::core::ffi::c_int
        == addr_byte_889F06 as ::core::ffi::c_int
    {
        let v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16).offset(v3 as isize) =
            1 as uint16;
        let ref mut fresh119 = *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16)
            .offset(v3 as isize);
        *fresh119 = (*fresh119).wrapping_add(2u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1908) as *mut uint16).offset(v3 as isize) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1938) as *mut uint16).offset(v3 as isize) =
            32 as uint16;
    }
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xcf0) as *mut uint16) as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16) as ::core::ffi::c_int)
        < 0x10000i32
    {
        let ref mut fresh120 = *((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16);
        *fresh120 = (*fresh120)
            .wrapping_add(*((&raw mut game.g_ram as *mut uint8).offset(0xcf0) as *mut uint16));
        let ref mut fresh121 = *((&raw mut game.g_ram as *mut uint8).offset(0xcf0) as *mut uint16);
        *fresh121 = (*fresh121).wrapping_add(kPowerBombExplosionRadiusAccel);
    }
}
pub unsafe fn CalculatePowerBombHdmaTablePointers(v0: uint16, game: &mut Game) {
    let mut v1: uint16 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xce2) as *mut uint16)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int) as uint16)
            >= 0x300u16
            || {
                *((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16) =
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xce2) as *mut uint16)
                        as ::core::ffi::c_int
                        - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                            as ::core::ffi::c_int
                        + 256 as ::core::ffi::c_int) as uint16;
                v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0xce4) as *mut uint16)
                    as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                        as ::core::ffi::c_int
                    + 256 as ::core::ffi::c_int) as uint16;
                v1 >= 0x300u16
            }
        {
            v1 = 0 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xce8) as *mut uint16) =
            ((v1 as ::core::ffi::c_int ^ 0x3ff as ::core::ffi::c_int) - 256 as ::core::ffi::c_int)
                as uint16;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xcec) as *mut uint16)
            as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            == 0i32
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xce8) as *mut uint16) = 0 as uint16;
        }
        let v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18d8) as *mut uint16).offset(v2 as isize) =
            (3 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0xce8) as *mut uint16)
                    as ::core::ffi::c_int
                + addr_kIndirectHdmaTable_PowerBombExplodeLeft as ::core::ffi::c_int)
                as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18d8) as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize) = (3 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0xce8) as *mut uint16)
                as ::core::ffi::c_int
            + addr_kIndirectHdmaTable_PowerBombExplodeRight as ::core::ffi::c_int)
            as uint16;
    }
}
pub unsafe fn HdmaobjPreInstr_PowerBombExplode_PreExplosionWhite(k: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        return;
    }
    CalculatePowerBombHdmaTablePointers(k, game);
    let v1: uint16 = 96 as uint16;
    let mut v2: uint16 = 0;
    let kk: ::core::ffi::c_int = Mult8x8(
        *((&raw mut game.g_ram as *mut uint8).offset(0xcec) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ),
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a286)) as *const uint8 as *mut uint8)
            .offset(0),
    ) as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) == 1u8
        {
            v2 = CalculatePowerBombHdmaScaled_OnScreen(
                kk as uint16,
                v1,
                *((&raw mut game.g_ram as *mut uint8).offset(0xcec) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                game,
            );
        } else {
            v2 = CalculatePowerBombHdmaScaled_RightOfScreen(
                kk as uint16,
                v1,
                *((&raw mut game.g_ram as *mut uint8).offset(0xcec) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                game,
            );
        }
    } else {
        v2 = CalculatePowerBombHdmaScaled_LeftOfScreen(
            kk as uint16,
            v1,
            *((&raw mut game.g_ram as *mut uint8).offset(0xcec) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ),
            game,
        );
    }
    let mut i: uint8 = game.k_out as uint8;
    loop {
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0xc406)
            .offset(i as isize) = v2 as uint8;
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0xc506)
            .offset(i as isize) = *(&raw mut v2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        );
        i = i.wrapping_sub(1);
        if !((i as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) == 0i32) {
            break;
        }
    }
    let mut v3: uint8 = (kk + 1 as ::core::ffi::c_int) as uint8;
    while v3 != 0xc0u8 {
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0xc406)
            .offset(v3 as isize) = 255 as uint8;
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0xc506)
            .offset(v3 as isize) = 0 as uint8;
        v3 = v3.wrapping_add(1);
    }
    let t: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0xcec) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) as ::core::ffi::c_int
            >> 3 as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x889079)) as *const uint8 as *mut uint8)
            .offset((t * 3 as ::core::ffi::c_int + 0 as ::core::ffi::c_int) as isize))
            | (0x20u8);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x889079)) as *const uint8 as *mut uint8)
            .offset((t * 3 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize))
            | (0x40u8);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x889079)) as *const uint8 as *mut uint8)
            .offset((t * 3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize))
            | (0x80u8);
    let ref mut fresh115 = *((&raw mut game.g_ram as *mut uint8).offset(0xcec) as *mut uint16);
    *fresh115 = (*fresh115)
        .wrapping_add(*((&raw mut game.g_ram as *mut uint8).offset(0xcf0) as *mut uint16));
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcec) as *mut uint16) < 0x9200u16 {
        let ref mut fresh116 = *((&raw mut game.g_ram as *mut uint8).offset(0xcf0) as *mut uint16);
        *fresh116 = (*fresh116).wrapping_sub(kPowerBombPreExplosionRadiusAccel);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
        let ref mut fresh117 = *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh117 = (*fresh117).wrapping_add(2u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1908) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    };
}
pub unsafe fn HdmaobjPreInstr_PowerBombExplode_PreExplosionYellow(k: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        return;
    }
    CalculatePowerBombHdmaTablePointers(k, game);
    let v1: *const uint8 = RomPtr_88(
        *((&raw mut game.g_ram as *mut uint8).offset(0xcf2) as *mut uint16),
        game,
    );
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int
        != 0
    {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16)
            as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int)
            == 1i32
        {
            CalculatePowerBombHdma_OnScreen(0 as uint16, v1, game);
        } else {
            CalculatePowerBombHdma_RightOfScreen(0 as uint16, v1, game);
        }
    } else {
        CalculatePowerBombHdma_LeftOfScreen(0 as uint16, v1, game);
    }
    let t: ::core::ffi::c_int = (*((&raw mut game.g_ram as *mut uint8).offset(0xcec) as *mut uint16)
        as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int
        >> 3 as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x889079)) as *const uint8 as *mut uint8)
            .offset((3 as ::core::ffi::c_int * t) as isize))
            | (0x20u8);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x889079)) as *const uint8 as *mut uint8)
            .offset((3 as ::core::ffi::c_int * t + 1 as ::core::ffi::c_int) as isize))
            | (0x40u8);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x889079)) as *const uint8 as *mut uint8)
            .offset((3 as ::core::ffi::c_int * t + 2 as ::core::ffi::c_int) as isize))
            | (0x80u8);
    let ref mut fresh111 = *((&raw mut game.g_ram as *mut uint8).offset(0xcf2) as *mut uint16);
    *fresh111 = (*fresh111).wrapping_add(192u16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcf2) as *mut uint16) == 0xa206u16 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
        let ref mut fresh112 = *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh112 = (*fresh112).wrapping_add(2u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1908) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xcf0) as *mut uint16) as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xcec) as *mut uint16) as ::core::ffi::c_int)
        < 0x10000i32
    {
        let ref mut fresh113 = *((&raw mut game.g_ram as *mut uint8).offset(0xcec) as *mut uint16);
        *fresh113 = (*fresh113)
            .wrapping_add(*((&raw mut game.g_ram as *mut uint8).offset(0xcf0) as *mut uint16));
        let ref mut fresh114 = *((&raw mut game.g_ram as *mut uint8).offset(0xcf0) as *mut uint16);
        *fresh114 = (*fresh114).wrapping_sub(kPowerBombPreExplosionRadiusAccel);
    }
}
pub unsafe fn SpawnCrystalFlashHdmaObjs(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16) = ((0x888000i32) as u16);
    static unk_88A2B0: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x40 as uint8,
        field_1: 0x28 as uint8,
        addr: 0xa2bd as VoidP,
    };
    static unk_88A2B8: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x40 as uint8,
        field_1: 0x29 as uint8,
        addr: 0xa32a as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88A2B0 as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88A2B8 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn CrystalFlashSetupPart1(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xc606) = -(1 as ::core::ffi::c_int) as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xc607) = 0 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcec) as *mut uint16) = 1024 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf0) as *mut uint16) =
        kPowerBombPreExplosionRadiusSpeed;
    QueueSfx1_Max6(1 as uint16, game);
}
pub unsafe fn CrystalFlashSetupPart2(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16) = 1024 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf0) as *mut uint16) =
        kPowerBombExplosionRadiusSpeed;
}
pub unsafe fn CrystalFlashCleanup(k: uint16, game: &mut Game) {
    let v0: uint16 = k;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcee) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16) = 0 as uint16;
    let v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x18b4) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x18b4) as *mut uint16)
        .offset((v1 + 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcec) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16) = 0 as uint16;
}
pub unsafe fn HdmaobjPreInstr_CrystalFlash_CustomLayerBlend(k: uint16, game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x60) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x61) = 8 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x62) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6e) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x71) = 51 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6c) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6d) = 4 as uint8;
}
pub unsafe fn HdmaobjPreInstr_CrystalFlash_Stage2_AfterGlow(k: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh109 = *((&raw mut game.g_ram as *mut uint8).offset(0x1908) as *mut uint16)
            .offset(v1 as isize);
        *fresh109 = (*fresh109).wrapping_sub(1);
        if (*fresh109 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            if ((*(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) as ::core::ffi::c_int
                | *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1)
                    as ::core::ffi::c_int
                | *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0)
                    as ::core::ffi::c_int)
                & 0x1f as ::core::ffi::c_int)
                != 0i32
            {
                if (*(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0)
                    as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int)
                    != 0i32
                {
                    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) =
                        ((*(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0)
                            as ::core::ffi::c_int
                            & 0x1f as ::core::ffi::c_int)
                            - 1 as ::core::ffi::c_int
                            | 0x20 as ::core::ffi::c_int) as uint8;
                }
                if (*(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1)
                    as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int)
                    != 0i32
                {
                    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) =
                        ((*(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1)
                            as ::core::ffi::c_int
                            & 0x1f as ::core::ffi::c_int)
                            - 1 as ::core::ffi::c_int
                            | 0x40 as ::core::ffi::c_int) as uint8;
                }
                if (*(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2)
                    as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int)
                    != 0i32
                {
                    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) =
                        ((*(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2)
                            as ::core::ffi::c_int
                            & 0x1f as ::core::ffi::c_int)
                            - 1 as ::core::ffi::c_int
                            | 0x80 as ::core::ffi::c_int) as uint8;
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0x1908) as *mut uint16
                    as *mut uint8)
                    .offset(k as ::core::ffi::c_int as isize) = g_word_888B96 as uint8;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16)
                    .offset(v1 as isize) = 1 as uint16;
                let ref mut fresh110 = *((&raw mut game.g_ram as *mut uint8).offset(0x18cc)
                    as *mut uint16)
                    .offset(v1 as isize);
                *fresh110 = (*fresh110).wrapping_add(2u16);
            }
        }
    }
}
pub unsafe fn CalculateCrystalFlashHdmaObjectTablePtrs(k: uint16, game: &mut Game) {
    let mut v1: uint16 = 0;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xce2) as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) as ::core::ffi::c_int
        + 256 as ::core::ffi::c_int) as uint16)
        >= 0x300u16
        || {
            *((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xce2) as *mut uint16)
                    as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                        as ::core::ffi::c_int
                    + 256 as ::core::ffi::c_int) as uint16;
            v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0xce4) as *mut uint16)
                as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                    as ::core::ffi::c_int
                + 256 as ::core::ffi::c_int) as uint16;
            v1 >= 0x300u16
        }
    {
        v1 = 0 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xce8) as *mut uint16) =
        ((v1 as ::core::ffi::c_int ^ 0x3ff as ::core::ffi::c_int) - 256 as ::core::ffi::c_int)
            as uint16;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        == 0i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xce8) as *mut uint16) = 0 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x18d8) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = (3
        as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0xce8) as *mut uint16) as ::core::ffi::c_int
        + addr_kIndirectHdmaTable_PowerBombExplodeLeft as ::core::ffi::c_int)
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x18d8) as *mut uint16).offset(
        ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int) as isize,
    ) = (3 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0xce8) as *mut uint16) as ::core::ffi::c_int
        + addr_kIndirectHdmaTable_PowerBombExplodeRight as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CalculateCrystalFlashHdmaDataTablesScaled_LeftOfScreen(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let mut right: uint8 = 0;
    let mut left: uint8 = 0;
    loop {
        let r20: uint8 = (Mult8x8(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16)
                as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint8,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const uint8 as *mut uint8)
                .offset((j as uint8 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let w: uint8 = (Mult8x8(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16)
                as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint8,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const uint8 as *mut uint8)
                .offset(j as uint8 as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let c: uint8 = *((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16) as uint8;
        left = (if (w as ::core::ffi::c_int + c as ::core::ffi::c_int) >= 256i32 {
            0 as ::core::ffi::c_int
        } else {
            255 as ::core::ffi::c_int
        }) as uint8;
        right = (if (w as ::core::ffi::c_int + c as ::core::ffi::c_int) >= 256i32 {
            w as ::core::ffi::c_int + c as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint8;
        loop {
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc406)
                .offset(k as uint8 as isize) = left;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc506)
                .offset(k as uint8 as isize) = right;
            if (k as uint8) == r20 {
                break;
            }
            k = k.wrapping_sub(1);
        }
        j = j.wrapping_add(1);
        if !((j as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) == 0i32) {
            break;
        }
    }
    game.k_out = *(&raw mut k as *mut uint8).offset(0) as uint16;
    ((right as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | left as ::core::ffi::c_int)
        as uint16
}
pub unsafe fn CalculateCrystalFlashHdmaDataTablesScaled_OnScreen(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let mut right: uint8 = 0;
    let mut left: uint8 = 0;
    loop {
        let r20: uint8 = (Mult8x8(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16)
                as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint8,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const uint8 as *mut uint8)
                .offset((j as uint8 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let w: uint8 = (Mult8x8(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16)
                as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint8,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const uint8 as *mut uint8)
                .offset(j as uint8 as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let c: uint8 = *((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16) as uint8;
        right = (if (c as ::core::ffi::c_int + w as ::core::ffi::c_int) < 256i32 {
            c as ::core::ffi::c_int + w as ::core::ffi::c_int
        } else {
            255 as ::core::ffi::c_int
        }) as uint8;
        left = (if (c as ::core::ffi::c_int - w as ::core::ffi::c_int) >= 0i32 {
            c as ::core::ffi::c_int - w as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint8;
        loop {
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc406)
                .offset(k as uint8 as isize) = left;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc506)
                .offset(k as uint8 as isize) = right;
            if (k as uint8) == r20 {
                break;
            }
            k = k.wrapping_sub(1);
        }
        j = j.wrapping_add(1);
        if !((j as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) == 0i32) {
            break;
        }
    }
    game.k_out = *(&raw mut k as *mut uint8).offset(0) as uint16;
    ((right as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | left as ::core::ffi::c_int)
        as uint16
}
pub unsafe fn CalculateCrystalFlashHdmaDataTablesScaled_RightOfScreen(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let mut left: uint8 = 0;
    let mut right: uint8 = 0;
    loop {
        let r20: uint8 = (Mult8x8(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16)
                as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint8,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const uint8 as *mut uint8)
                .offset((j as uint8 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let w: uint8 = (Mult8x8(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16)
                as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint8,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const uint8 as *mut uint8)
                .offset(j as uint8 as isize),
        ) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        let c: uint8 = *((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16) as uint8;
        right = (if c < w {
            255 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint8;
        left = (if c < w {
            c as ::core::ffi::c_int - w as ::core::ffi::c_int
        } else {
            255 as ::core::ffi::c_int
        }) as uint8;
        loop {
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc406)
                .offset(k as uint8 as isize) = left;
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xc506)
                .offset(k as uint8 as isize) = right;
            if (k as uint8) == r20 {
                break;
            }
            k = k.wrapping_sub(1);
        }
        j = j.wrapping_add(1);
        if !((j as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) == 0i32) {
            break;
        }
    }
    game.k_out = *(&raw mut k as *mut uint8).offset(0) as uint16;
    ((right as ::core::ffi::c_int) << 8 as ::core::ffi::c_int | left as ::core::ffi::c_int)
        as uint16
}
pub unsafe fn HdmaobjPreInstr_CrystalFlash_Stage1_Explosion(k: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x592) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        return;
    }
    CalculateCrystalFlashHdmaObjectTablePtrs(k, game);
    let mut v2: uint16 = 0;
    let kk: ::core::ffi::c_int = Mult8x8(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16)
            as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint8,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a286)) as *const uint8 as *mut uint8)
            .offset(0),
    ) as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int
        != 0
    {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xce6) as *mut uint16)
            as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int)
            == 1i32
        {
            v2 = CalculateCrystalFlashHdmaDataTablesScaled_OnScreen(
                kk as uint16,
                96 as uint16,
                game,
            );
        } else {
            v2 = CalculateCrystalFlashHdmaDataTablesScaled_RightOfScreen(
                kk as uint16,
                96 as uint16,
                game,
            );
        }
    } else {
        v2 = CalculateCrystalFlashHdmaDataTablesScaled_LeftOfScreen(
            kk as uint16,
            96 as uint16,
            game,
        );
    }
    let mut i: ::core::ffi::c_int = game.k_out as ::core::ffi::c_int;
    loop {
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0xc406)
            .offset(i as uint8 as isize) = v2 as uint8;
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0xc506)
            .offset(i as uint8 as isize) = ((v2 as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint8;
        i -= 1;
        if !((i & 0x80 as ::core::ffi::c_int) == 0i32) {
            break;
        }
    }
    let mut v3: uint8 = (kk + 1 as ::core::ffi::c_int) as uint8;
    while v3 != 0xc0u8 {
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0xc406)
            .offset(v3 as isize) = 255 as uint8;
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0xc506)
            .offset(v3 as isize) = 0 as uint8;
        v3 = v3.wrapping_add(1);
    }
    let pos: ::core::ffi::c_int =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) as ::core::ffi::c_int
            >> 3 as ::core::ffi::c_int)
            * 3 as ::core::ffi::c_int;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const uint8 as *mut uint8)
            .offset((pos + 0 as ::core::ffi::c_int) as isize))
            | (0x20u8);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const uint8 as *mut uint8)
            .offset((pos + 1 as ::core::ffi::c_int) as isize))
            | (0x40u8);
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const uint8 as *mut uint8)
            .offset((pos + 2 as ::core::ffi::c_int) as isize))
            | (0x80u8);
    let ref mut fresh106 = *((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16);
    *fresh106 = (*fresh106)
        .wrapping_add(*((&raw mut game.g_ram as *mut uint8).offset(0xcf0) as *mut uint16));
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16) < 0x2000u16 {
        let ref mut fresh107 = *((&raw mut game.g_ram as *mut uint8).offset(0xcf0) as *mut uint16);
        *fresh107 = (*fresh107).wrapping_add(kPowerBombExplosionRadiusAccel);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
        let ref mut fresh108 = *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh108 = (*fresh108).wrapping_add(2u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1908) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    };
}
pub unsafe fn FxTypeFunc_22_ScrollingSky(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x195e) as *mut uint16) = 1248 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x196e) as *mut uint16) = 6 as uint16;
    static unk_88A62E: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x2 as uint8,
        field_1: 0x12 as uint8,
        addr: 0xad63 as VoidP,
    };
    static unk_88A636: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x42 as uint8,
        field_1: 0x11 as uint8,
        addr: 0xad4e as VoidP,
    };
    static unk_88A63E: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x42 as uint8,
        field_1: 0xd as uint8,
        addr: 0xad39 as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88A62E as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88A636 as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88A63E as *const ::core::ffi::c_void,
        game,
    );
    HdmaobjPreInstr_FxType22_BG3Yscroll(0 as uint16, game);
}
unsafe fn SetupSomeHdmaTablesBG3(
    r24: uint16,
    r0: uint16,
    r3: uint16,
    r6: uint16,
    r9: uint16,
    game: &mut Game,
) -> uint16 {
    let mut v1: int16 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0x598) as *mut uint16) = 32 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00) as *mut uint16) = 31 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(1) as *mut uint8 as *mut uint16) =
        0 as uint16;
    let mut v0: uint16 = 3 as uint16;
    v1 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)) as i16)
        .wrapping_add(32i16);
    let mut r18: uint16 =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)).wrapping_add(32u16);
    let mut v2: uint16 = 0 as uint16;
    while (v1).wrapping_sub(
        (*((RomPtr_88 as unsafe fn(uint16_t, &mut Game) -> *const uint8)(r0, game)
            .offset(v2 as isize) as *const uint8 as *mut uint16)) as i16,
    ) < 0i16
        || (v1).wrapping_sub(
            (*((RomPtr_88 as unsafe fn(uint16_t, &mut Game) -> *const uint8)(r9, game)
                .offset(v2 as isize) as *const uint8 as *mut uint16)) as i16,
        ) >= 0i16
    {
        v2 = (v2).wrapping_add(6u16);
        if ((v2) as i16).wrapping_sub((r24) as i16) >= 0i16 {
            return v0;
        }
    }
    let mut v3: uint16 = 0;
    if r18 as ::core::ffi::c_int - 1248 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v3 = (r18) & (0xfu16);
    } else {
        v3 = (r18) & (0x1fu16);
    }
    let R22_: uint16 = v3;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(3) as *mut uint8 as *mut uint16) =
        (*((RomPtr_88 as unsafe fn(uint16_t, &mut Game) -> *const uint8)(r3, game)
            .offset(v2 as isize) as *const uint8 as *mut uint16))
            .wrapping_sub(v3);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(4) as *mut uint8 as *mut uint16) =
        (R22_ as ::core::ffi::c_int
            + *((RomPtr_88 as unsafe fn(uint16_t, &mut Game) -> *const uint8)(r6, game)
                .offset(v2 as isize) as *const uint8 as *mut uint16)
                as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x598) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
    loop {
        let r20: uint16 = *(&raw mut game.g_ram as *mut uint8)
            .offset(0x9c00)
            .offset(v0 as isize) as uint16;
        r18 = (r18).wrapping_add(r20);
        v0 = (v0).wrapping_add(3u16);
        let ref mut fresh49 = *((&raw mut game.g_ram as *mut uint8).offset(0x598) as *mut uint16);
        *fresh49 = (*fresh49).wrapping_add(r20);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x598) as *mut uint16) as ::core::ffi::c_int
            - 224 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            break;
        }
        while ((r18) as i16).wrapping_sub(
            (*((RomPtr_88 as unsafe fn(uint16_t, &mut Game) -> *const uint8)(r0, game)
                .offset(v2 as isize) as *const uint8 as *mut uint16)) as i16,
        ) < 0i16
            || ((r18) as i16).wrapping_sub(
                (*((RomPtr_88 as unsafe fn(uint16_t, &mut Game) -> *const uint8)(r9, game)
                    .offset(v2 as isize) as *const uint8 as *mut uint16)) as i16,
            ) >= 0i16
        {
            v2 = (v2).wrapping_add(6u16);
            if ((v2) as i16).wrapping_sub((r24) as i16) >= 0i16 {
                return v0;
            }
        }
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9c00)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) =
            *((RomPtr_88 as unsafe fn(uint16_t, &mut Game) -> *const uint8)(r3, game)
                .offset(v2 as isize) as *const uint8 as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9c00)
            .offset((v0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) =
            (*((RomPtr_88 as unsafe fn(uint16_t, &mut Game) -> *const uint8)(r6, game)
                .offset(v2 as isize) as *const uint8 as *mut uint16))
                .wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0x598) as *mut uint16));
    }
    v0
}
pub unsafe fn HdmaobjPreInstr_FxType22_BG3Yscroll(mut k: uint16, game: &mut Game) {
    DamageSamusInTopRow(game);
    k = SetupSomeHdmaTablesBG3(
        78 as uint16,
        addr_word_88A8E8 as uint16,
        (addr_word_88A8E8 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
        (addr_word_88A8E8 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16,
        (addr_word_88A8E8 as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9c00)
        .offset(k as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
}
pub unsafe fn HdmaobjInstr_SetFlagB(k: uint16, hdp: *const uint8, game: &mut Game) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
    hdp
}
static g_word_88C46E: [int16; 16] = [0, 1, 1, 0, 0, -(1), -(1), 0, 0, 1, 1, 0, 0, -(1), -(1), 0];
pub unsafe fn HdmaobjPreInstr_BG3Xscroll(k: uint16, game: &mut Game) {
    let mut v8: int16 = 0;
    let mut v9: int16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) as ::core::ffi::c_int
        - 1024 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if (*(&raw mut game.g_ram as *mut uint8).offset(0x5b5) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int)
            == 0i32
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x596) as *mut uint16) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0x596) as *mut uint16)
                    as ::core::ffi::c_int
                    + 2 as ::core::ffi::c_int
                    & 0x1e as ::core::ffi::c_int) as uint16;
        }
        let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x596) as *mut uint16);
        let mut v2: uint16 = 0 as uint16;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0x9e80) as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a938)) as *const uint8
                    as *mut uint16)
                    .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v2 = (v2).wrapping_add(2u16);
            v1 = (v1).wrapping_add(2u16);
            if !(((v1) as i16).wrapping_sub(32i16) < 0i16) {
                break;
            }
        }
    } else {
        let v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh104 = *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16)
            .offset(v3 as isize);
        let fresh105 = *fresh104;
        *fresh104 = (*fresh104).wrapping_sub(1);
        if fresh105 == 1u16 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16)
                .offset(v3 as isize) = 6 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
                .offset(v3 as isize) = (*((&raw mut game.g_ram as *mut uint8).offset(0x1914)
                as *mut uint16)
                .offset(v3 as isize) as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int
                & 0x1f as ::core::ffi::c_int) as uint16;
        }
        let mut v5: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
            .offset(v3 as isize);
        let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
        while i >= 0i32 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x9e80) as *mut uint16)
                .offset((v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                ((g_word_88C46E[(i >> 1 as ::core::ffi::c_int) as usize]) as u16).wrapping_add(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16),
                );
            v5 = (v5 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
                as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
    }
    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0 as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9f00) as *mut uint16) = 0 as uint16;
    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_2 as *mut uint16) = 31 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9f00).offset(2) as *mut uint8 as *mut uint16) =
        31 as uint16;
    let mut r18: uint16 = 31 as uint16;
    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(1))
    .field_0 as *mut uint16) = -(25088 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9f00).offset(3) as *mut uint8 as *mut uint16) =
        -(25088 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) as ::core::ffi::c_int
        - 1024 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9f00).offset(2) as *mut uint8
            as *mut uint16) = 0 as uint16;
    } else {
        r18 = HdmaFunc_A786(0x105 as uint16, r18, 177 as uint16, 0 as uint16, game);
    }
    let mut v7: uint16 = 5 as uint16;
    r18 = HdmaFunc_A786(
        5 as uint16,
        r18,
        -(24960 as ::core::ffi::c_int) as uint16,
        128 as uint16,
        game,
    );
    v8 = (224i16).wrapping_sub((r18) as i16);
    while (v8).wrapping_sub(16i16) >= 0i16 {
        v8 = (v8).wrapping_sub(16i16);
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0)
            .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16) = 144 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9f00)
            .offset(v7 as isize) as *mut uint8 as *mut uint16) = 144 as uint16;
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_1)
            .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16) =
            -(24960 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9f00)
            .offset((v7 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) = -(24960 as ::core::ffi::c_int) as uint16;
        v7 = (v7).wrapping_add(3u16);
    }
    v9 = v8;
    if v8 != 0 {
        v9 = (v8).wrapping_add(128i16);
    }
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0)
        .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16) = v9 as uint16;
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9f00)
        .offset(v7 as isize) as *mut uint8 as *mut uint16) = v9 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_1)
        .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16) =
        -(24960 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9f00)
        .offset((v7 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as *mut uint8
        as *mut uint16) = -(24960 as ::core::ffi::c_int) as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(1))
    .field_0)
        .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9f00)
        .offset((v7 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as *mut uint8
        as *mut uint16) = 0 as uint16;
}
pub unsafe fn HdmaFunc_A786(
    k: uint16,
    mut r18: uint16,
    r20: uint16,
    r22: uint16,
    game: &mut Game,
) -> uint16 {
    let mut v0: uint16 = k;
    let mut v2: int16 = 0;
    let mut v1: uint16 =
        (1216u16).wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16));
    let mut r24: uint16 =
        (1216u16).wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16));
    if (1216i16)
        .wrapping_sub((*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)) as i16)
        >= 0i16
        && *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) != 1216u16
    {
        if v1 as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        {
            r18 = (1216u16)
                .wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16));
        } else {
            r18 = (1216u16)
                .wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16));
            if v1 as ::core::ffi::c_int - 193 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v1 = 193 as uint16;
            }
            r18 = v1;
            r24 = v1;
            loop {
                v2 = ((v1) as i16).wrapping_sub(16i16);
                if v2 < 0i16 {
                    break;
                }
                r24 = v2 as uint16;
                *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
                    as *mut ExpandingSquareTransitionHdma)
                    .offset(0))
                .field_0)
                    .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = (r22) | (0x10u16);
                *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
                    as *mut ExpandingSquareTransitionHdma)
                    .offset(0))
                .field_1)
                    .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = r20;
                v1 = r24;
                v0 = (v0).wrapping_add(3u16);
            }
        }
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0)
            .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = (r22) | (r24);
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_1)
            .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = r20;
    }
    r18
}
static unk_88A7EF: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0xf as uint8,
    addr: 0xad76 as VoidP,
};
static unk_88A80B: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0xf as uint8,
    addr: 0xad89 as VoidP,
};
pub unsafe fn FxTypeFunc_20(game: &mut Game) {
    let ref mut fresh47 = *(&raw mut game.g_ram as *mut uint8).offset(0x91b);
    *fresh47 = (*fresh47) | (1u8);
    let ref mut fresh48 = *(&raw mut game.g_ram as *mut uint8).offset(0x91c);
    *fresh48 = (*fresh48) | (1u8);
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88A7EF as *const ::core::ffi::c_void,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x59a) as *mut uint16) = 224 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x59c) as *mut uint16) = 0 as uint16;
}
pub unsafe fn RoomSetupAsm_ScrollingSkyOcean(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 74 as uint8;
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88A80B as *const ::core::ffi::c_void,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x59a) as *mut uint16) = 224 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x59c) as *mut uint16) = 0 as uint16;
}
pub unsafe fn DamageSamusInTopRow(game: &mut Game) {
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)) as i16)
        .wrapping_sub((*((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16)) as i16)
        < 0i16
        || *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16)
                as ::core::ffi::c_int
            - 17 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa50) as *mut uint16) = 8 as uint16;
    }
}
pub unsafe fn HdmaobjPreInstr_SkyLandBG2XscrollInner(k: uint16, game: &mut Game) {
    let mut i: uint16 = 0;
    let mut r24: uint16 = 0 as uint16;
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 74 as uint8;
    let mut v1: uint16 = 0 as uint16;
    loop {
        let v3: *mut uint16 = (&raw mut game.g_ram as *mut uint8).offset(
            (*(game.g_rom.offset(crate::rtl::lorom_offset(0x88aec1)) as *const uint8
                as *mut HdmaScrollEntry)
                .offset(v1 as isize))
            .hdma_data_table_entry as isize,
        ) as *mut uint8 as *mut uint16;
        AddToHiLo(
            v3.offset(1) as *mut uint16,
            v3.offset(0) as *mut uint16,
            ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x88aec1)) as *const uint8
                as *mut HdmaScrollEntry)
                .offset(v1 as isize))
            .scroll_speed as uint32)
                << 16 as ::core::ffi::c_int
                | (*(game.g_rom.offset(crate::rtl::lorom_offset(0x88aec1)) as *const uint8
                    as *mut HdmaScrollEntry)
                    .offset(v1 as isize))
                .scroll_subspeed as uint32,
        );
        v1 = v1.wrapping_add(1);
        if !(v1 as ::core::ffi::c_int * 8 as ::core::ffi::c_int - 184 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0)
        {
            break;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x9f80) as *mut uint16).offset(44) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9f80) as *mut uint16).offset(45) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9f00) as *mut uint16) = 31 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9f00).offset(1) as *mut uint8 as *mut uint16) =
        1438 as uint16;
    let mut r18: uint16 =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)).wrapping_add(32u16);
    let r20: uint16 =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)).wrapping_add(224u16);
    let mut v5: uint16 = 0 as uint16;
    let mut v6: uint16 = 3 as uint16;
    loop {
        while ((r18) as i16).wrapping_sub(
            ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x88aec1)) as *const uint8
                as *mut HdmaScrollEntry)
                .offset(v5 as isize))
            .top_pos) as i16,
        ) >= 0i16
            && ((r18) as i16).wrapping_sub(
                ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x88aec1)) as *const uint8
                    as *mut HdmaScrollEntry)
                    .offset((v5 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize))
                .top_pos) as i16,
            ) < 0i16
        {
            let mut v8: uint16 = ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x88aec1))
                as *const uint8 as *mut HdmaScrollEntry)
                .offset((v5 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize))
            .top_pos)
                .wrapping_sub(r18);
            r24 = v8;
            if v8 as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x9f00)
                    .offset(v6 as isize) as *mut uint8 as *mut uint16) = 127 as uint16;
                *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x9f00)
                    .offset((v6 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
                    as *mut uint8 as *mut uint16) =
                    ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x88aec1)) as *const uint8
                        as *mut HdmaScrollEntry)
                        .offset(v5 as isize))
                    .hdma_data_table_entry)
                        .wrapping_add(2u16);
                v6 = (v6).wrapping_add(3u16);
                v8 = (r24).wrapping_sub(127u16);
            }
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x9f00)
                .offset(v6 as isize) as *mut uint8 as *mut uint16) = v8;
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x9f00)
                .offset((v6 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
                as *mut uint8 as *mut uint16) =
                ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x88aec1)) as *const uint8
                    as *mut HdmaScrollEntry)
                    .offset(v5 as isize))
                .hdma_data_table_entry)
                    .wrapping_add(2u16);
            r18 = (r18).wrapping_add(r24);
            v6 = (v6).wrapping_add(3u16);
            if r18 as ::core::ffi::c_int - r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x9f00)
                    .offset((v6 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize)
                    as *mut uint8 as *mut uint16) = 0 as uint16;
                return;
            }
        }
        v5 = v5.wrapping_add(1);
        if !(v5 as ::core::ffi::c_int * 8 as ::core::ffi::c_int - 184 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0)
        {
            break;
        }
    }
    i = (1535u16).wrapping_sub(r18);
    loop {
        r24 = i;
        if i as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            break;
        }
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9f00)
            .offset(v6 as isize) as *mut uint8 as *mut uint16) = 127 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9f00)
            .offset((v6 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) = 181 as uint16;
        v6 = (v6).wrapping_add(3u16);
        i = (r24).wrapping_sub(127u16);
    }
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9f00)
        .offset(v6 as isize) as *mut uint8 as *mut uint16) = i;
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9f00)
        .offset((v6 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as *mut uint8
        as *mut uint16) = 181 as uint16;
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9f00)
        .offset((v6 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as *mut uint8
        as *mut uint16) = 0 as uint16;
}
pub unsafe fn HdmaobjPreInstr_SkyLandBG2Xscroll(k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) == 0 {
        HdmaobjPreInstr_SkyLandBG2XscrollInner(k, game);
    }
}
pub unsafe fn HdmaobjPreInstr_SkyLandBG2Xscroll2(k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) == 0 {
        HdmaobjPreInstr_SkyLandBG2XscrollInner(k, game);
    }
}
pub unsafe fn RoomMainAsm_ScrollingSky(src: *const uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9f00).offset(0) as *mut uint8
            as *mut uint16) = 0 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16);
        let v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
        let v1: *mut VramWriteEntry = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*v1.offset(0)).size = 64 as uint16;
        (*v1.offset(1)).size = 64 as uint16;
        (*v1.offset(2)).size = 64 as uint16;
        (*v1.offset(3)).size = 64 as uint16;
        let mut tt: ::core::ffi::c_int = (*((&raw mut game.g_ram as *mut uint8).offset(0x915)
            as *mut uint16) as ::core::ffi::c_int
            & 0x7f8 as ::core::ffi::c_int)
            - 16 as ::core::ffi::c_int;
        if tt & 0x8000 as ::core::ffi::c_int != 0 {
            tt = 0 as ::core::ffi::c_int;
        }
        let v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int * tt as uint8 as ::core::ffi::c_int;
        let y1: ::core::ffi::c_int = (tt >> 8 as ::core::ffi::c_int) as uint8 as ::core::ffi::c_int;
        let y2: ::core::ffi::c_int = ((tt >> 8 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
            as uint8 as ::core::ffi::c_int;
        let v3: VoidP = (*src.offset(y1 as isize) as ::core::ffi::c_int + v2) as VoidP;
        (*v1.offset(0)).src.addr = v3;
        (*v1.offset(1)).src.addr = (v3).wrapping_add(64u16);
        let v5: VoidP = (*src.offset(y2 as isize) as ::core::ffi::c_int + v2) as VoidP;
        (*v1.offset(2)).src.addr = v5;
        (*v1.offset(3)).src.addr = (v5).wrapping_add(64u16);
        (*v1.offset(0)).src.bank = 0x8a as uint8;
        (*v1.offset(1)).src.bank = 0x8a as uint8;
        (*v1.offset(2)).src.bank = 0x8a as uint8;
        (*v1.offset(3)).src.bank = 0x8a as uint8;
        let t: ::core::ffi::c_int = (*(&raw mut game.g_ram as *mut uint8).offset(0x59)
            as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int;
        let v6: uint16 = (t + 4 as ::core::ffi::c_int
            * (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int
                - 16 as ::core::ffi::c_int
                & 0x1f8 as ::core::ffi::c_int)) as uint16;
        (*v1.offset(0)).vram_dst = v6;
        (*v1.offset(1)).vram_dst = (v6).wrapping_add(32u16);
        let v7: uint16 = (t + 4 as ::core::ffi::c_int
            * (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int
                + 240 as ::core::ffi::c_int
                & 0x1f8 as ::core::ffi::c_int)) as uint16;
        (*v1.offset(2)).vram_dst = v7;
        (*v1.offset(3)).vram_dst = (v7).wrapping_add(32u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            (v0).wrapping_add(28u16);
    };
}
pub unsafe fn RoomCode_ScrollingSkyLand(game: &mut Game) {
    RoomMainAsm_ScrollingSky(
        RomPtr_88(addr_off_88AD9C as uint16_t, game) as *const uint16,
        game,
    );
}
pub unsafe fn RoomMainAsm_ScrollingSkyOcean(game: &mut Game) {
    RoomMainAsm_ScrollingSky(
        RomPtr_88(addr_off_88ADA6 as uint16_t, game) as *const uint16,
        game,
    );
}
static unk_88B08C: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0x11 as uint8,
    addr: 0xb0ac as VoidP,
};
static kFirefleaFlashingShades: [uint16; 12] = [
    0, 0x100, 0x200, 0x300, 0x400, 0x500, 0x600, 0x500, 0x400, 0x300, 0x200, 0x100,
];
static kFirefleaDarknessShades: [uint16; 6] = [0, 0x600, 0xc00, 0x1200, 0x1800, 0x1900];
pub unsafe fn FxTypeFunc_24(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1778) as *mut uint16) = 6 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x177a) as *mut uint16) = 0 as uint16;
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88B08C as *const ::core::ffi::c_void,
        game,
    );
    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0 as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x177c) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x177e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1780) as *mut uint16) = 24 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1782) as *mut uint16) =
        kFirefleaFlashingShades[0 as ::core::ffi::c_int as usize];
}
pub unsafe fn HdmaobjPreInstr_FirefleaBG3XScroll(k: uint16, game: &mut Game) {
    let mut v2: int8 = 0;
    let mut v1: uint16 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int
            | 0xc as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) == 0 {
        let ref mut fresh103 = *((&raw mut game.g_ram as *mut uint8).offset(0x1778) as *mut uint16);
        *fresh103 = (*fresh103).wrapping_sub(1);
        if *fresh103 == 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1778) as *mut uint16) = 6 as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x177e) as *mut uint16)
                as ::core::ffi::c_int
                - 10 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0x177a) as *mut uint16))
                    .wrapping_add(1u16);
                if (*((&raw mut game.g_ram as *mut uint8).offset(0x177a) as *mut uint16))
                    .wrapping_add(1u16)
                    >= 0xcu16
                {
                    v1 = 0 as uint16;
                }
            } else {
                v1 = 6 as uint16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x177a) as *mut uint16) = v1;
        }
        v2 = (((kFirefleaDarknessShades[(*((&raw mut game.g_ram as *mut uint8).offset(0x177e)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as usize])
            .wrapping_add(
                kFirefleaFlashingShades
                    [*((&raw mut game.g_ram as *mut uint8).offset(0x177a) as *mut uint16) as usize],
            )) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as int8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = ((v2) as u8) | (0x80u8);
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = ((v2) as u8) | (0x40u8);
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = ((v2) as u8) | (0x20u8);
    }
}
pub unsafe fn ExpandingContractingHdmaEffect(game: &mut Game) {
    let ref mut fresh32 = *((&raw mut game.g_ram as *mut uint8).offset(0x59a) as *mut uint16);
    *fresh32 = (*fresh32).wrapping_add(1);
    if *fresh32 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x59a) as *mut uint16) = 0 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5a0) as *mut uint16) != 0 {
            let ref mut fresh33 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x5a2) as *mut uint16);
            *fresh33 = (*fresh33).wrapping_sub(1024u16);
            if *((&raw mut game.g_ram as *mut uint8).offset(0x5a2) as *mut uint16)
                as ::core::ffi::c_int
                - 0x2000 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x5a2) as *mut uint16) =
                    0x2000 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x5a0) as *mut uint16) = 0 as uint16;
            }
        } else {
            let ref mut fresh34 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x5a2) as *mut uint16);
            *fresh34 = (*fresh34).wrapping_add(1024u16);
            if *((&raw mut game.g_ram as *mut uint8).offset(0x5a2) as *mut uint16)
                as ::core::ffi::c_int
                + 0x8000 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                let ref mut fresh35 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x5a0) as *mut uint16);
                *fresh35 = (*fresh35).wrapping_add(1);
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x5a4) as *mut uint16) = 128 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x5a6) as *mut uint16) = 128 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x5a8) as *mut uint16) = 127 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x5aa) as *mut uint16) = 127 as uint16;
    }
    let mut v0: uint16 =
        (2u16).wrapping_mul(*((&raw mut game.g_ram as *mut uint8).offset(0x5a4) as *mut uint16));
    let mut v1: uint16 =
        (2u16).wrapping_mul(*((&raw mut game.g_ram as *mut uint8).offset(0x5a8) as *mut uint16));
    let mut r18: uint16 = 0 as uint16;
    let mut r20: uint16 = 32 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9c00)
            .offset(v0 as isize) as *mut uint8 as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x5a6) as *mut uint16))
                .wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0x5a4) as *mut uint16));
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9c00)
            .offset(v1 as isize) as *mut uint8 as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x5aa) as *mut uint16))
                .wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0x5a8) as *mut uint16));
        r18 =
            (r18).wrapping_add(*((&raw mut game.g_ram as *mut uint8).offset(0x5a2) as *mut uint16));
        if Unreachable(game) {
            let ref mut fresh36 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x5a6) as *mut uint16);
            *fresh36 = (*fresh36).wrapping_add(1);
            let ref mut fresh37 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x5aa) as *mut uint16);
            *fresh37 = (*fresh37).wrapping_sub(1);
        }
        let ref mut fresh38 = *((&raw mut game.g_ram as *mut uint8).offset(0x5a4) as *mut uint16);
        *fresh38 = (*fresh38).wrapping_add(1);
        let ref mut fresh39 = *((&raw mut game.g_ram as *mut uint8).offset(0x5a8) as *mut uint16);
        *fresh39 = (*fresh39).wrapping_sub(1);
        v1 = (v1).wrapping_sub(2u16);
        v0 = (v0).wrapping_add(2u16);
        r20 = r20.wrapping_sub(1);
        if !(r20 != 0) {
            break;
        }
    }
}
static g_word_88B256: [uint16; 17] = [
    0x46, 1, 0x46, 3, 0x46, 2, 0x46, 1, 0x46, 1, 0x46, 2, 0x46, 2, 0x46, 1, 0x8000,
];
pub unsafe fn HandleEarthquakeSoundEffect(game: &mut Game) {
    let mut v1: int16 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x609) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
        && {
            let ref mut fresh43 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x609) as *mut uint16);
            *fresh43 = (*fresh43).wrapping_sub(1);
            (*fresh43 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32
        }
    {
        let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x607) as *mut uint16);
        v1 = g_word_88B256[(*((&raw mut game.g_ram as *mut uint8).offset(0x607) as *mut uint16)
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as usize] as int16;
        if v1 < 0i16 {
            v0 = 0 as uint16;
            v1 = g_word_88B256[0 as ::core::ffi::c_int as usize] as int16;
        }
        QueueSfx2_Max6(v1 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x609) as *mut uint16) =
            (g_word_88B256[((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
                + (v0 as ::core::ffi::c_int & 1 as ::core::ffi::c_int)
                + (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                    as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x607) as *mut uint16) =
            (v0).wrapping_add(4u16);
    }
}
static unk_88B289: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0x12 as uint8,
    addr: 0xc3e1 as VoidP,
};
static unk_88B291: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0x10 as uint8,
    addr: 0xc3f0 as VoidP,
};
static unk_88B2B1: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0x12 as uint8,
    addr: 0xc3e1 as VoidP,
};
static unk_88B2B9: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0x10 as uint8,
    addr: 0xc3f0 as VoidP,
};
pub unsafe fn FxTypeFunc_2_Lava(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x196c) as *mut uint16) = ((0x88b343i32) as u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1962) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16);
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88B289 as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88B291 as *const ::core::ffi::c_void,
        game,
    );
    SpawnBG3ScrollHdmaObject(game);
    SpawnAnimtiles(addr_kAnimtiles_Lava as uint16, game);
}
pub unsafe fn FxTypeFunc_4_Acid(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x196c) as *mut uint16) = ((0x88b343i32) as u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1962) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16);
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88B2B1 as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88B2B9 as *const ::core::ffi::c_void,
        game,
    );
    SpawnBG3ScrollHdmaObject(game);
    SpawnAnimtiles(addr_kAnimtiles_Acid as uint16, game);
}
pub unsafe fn FxHandleTide(game: &mut Game) {
    let mut v2: uint16 = 0;
    let mut v5: uint16 = 0;
    if *(((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16 as *mut uint8).offset(1)
        as *mut int16)
        < 0i16
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1970) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1972) as *mut uint16) = 0 as uint16;
        let v0: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1974)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as ::core::ffi::c_int;
        let v1: uint16 = (8u16).wrapping_mul((kSinCosTable8bit_Sext[v0 as usize]) as u16);
        if (kSinCosTable8bit_Sext[v0 as usize] as ::core::ffi::c_int & 0x1000 as ::core::ffi::c_int)
            != 0i32
        {
            let ref mut fresh40 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1972) as *mut uint16);
            *fresh40 = (*fresh40).wrapping_sub(1);
        }
        *(((&raw mut game.g_ram as *mut uint8).offset(0x1970) as *mut uint16 as *mut uint8)
            .offset(1) as *mut uint16) = v1;
        if (kSinCosTable8bit_Sext[v0 as usize] as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
            == 0i32
        {
            v2 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1974) as *mut uint16))
                .wrapping_add(288u16);
        } else {
            v2 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1974) as *mut uint16))
                .wrapping_add(192u16);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1974) as *mut uint16) = v2;
    } else if (*(((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16 as *mut uint8)
        .offset(1) as *mut uint16) as ::core::ffi::c_int
        & 0x4000 as ::core::ffi::c_int)
        != 0i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1970) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1972) as *mut uint16) = 0 as uint16;
        let v3: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1974)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as ::core::ffi::c_int;
        let v4: uint16 = (32u16).wrapping_mul((kSinCosTable8bit_Sext[v3 as usize]) as u16);
        if (kSinCosTable8bit_Sext[v3 as usize] as ::core::ffi::c_int & 0x400 as ::core::ffi::c_int)
            != 0i32
        {
            let ref mut fresh41 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1972) as *mut uint16);
            *fresh41 = (*fresh41).wrapping_sub(1);
        }
        *(((&raw mut game.g_ram as *mut uint8).offset(0x1970) as *mut uint16 as *mut uint8)
            .offset(1) as *mut uint16) = v4;
        if (kSinCosTable8bit_Sext[v3 as usize] as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
            == 0i32
        {
            v5 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1974) as *mut uint16))
                .wrapping_add(224u16);
        } else {
            v5 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1974) as *mut uint16))
                .wrapping_add(128u16);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1974) as *mut uint16) = v5;
    }
}
pub unsafe fn FxRisingFunction_LavaAcid(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) != 0 {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16)
            as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            == 0i32
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16)
                < *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16)
                || *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16)
                    == *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16)
            {
                return;
            }
        } else if !(*((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16)
            != *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16)
            && *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16)
                < *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16))
        {
            return;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x196c) as *mut uint16) =
            ((0x88b367i32) as u16);
    }
}
pub unsafe fn FxRisingFunction_LavaAcid_WaitToRise(game: &mut Game) {
    HandleEarthquakeSoundEffect(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 21 as uint16;
    let ref mut fresh44 = *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16);
    *fresh44 = (*fresh44) | (0x20u16);
    let ref mut fresh45 = *((&raw mut game.g_ram as *mut uint8).offset(0x1980) as *mut uint16);
    *fresh45 = (*fresh45).wrapping_sub(1);
    if *fresh45 == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x196c) as *mut uint16) =
            ((0x88b382i32) as u16);
    }
}
pub unsafe fn FxRisingFunction_LavaAcid_Raising(game: &mut Game) {
    HandleEarthquakeSoundEffect(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 21 as uint16;
    let ref mut fresh42 = *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16);
    *fresh42 = (*fresh42) | (0x20u16);
    if RaiseOrLowerFx(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x196c) as *mut uint16) =
            ((0x88b343i32) as u16);
    }
}
pub unsafe fn HdmaobjInstr_B3A9(k: uint16, hdp: *const uint8, game: &mut Game) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x192c) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 112 as uint16;
    hdp
}
pub unsafe fn CallFxRisingFunc(ea: uint32, game: &mut Game) {
    match ea {
        8958787 => {
            FxRisingFunction_LavaAcid(game);
        }
        8958823 => {
            FxRisingFunction_LavaAcid_WaitToRise(game);
        }
        8958850 => {
            FxRisingFunction_LavaAcid_Raising(game);
        }
        8963112 => {
            FxRisingFunction_C428_WaterNormal(game);
        }
        8963148 => {
            FxRisingFunction_WaterWaitToRise(game);
        }
        8963160 => {
            FxRisingFunction_WaterRising(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
static kLavaSoundEffects: [uint8; 8] = [0x12, 0x13, 0x14, 0x12, 0x13, 0x14, 0x12, 0x13];
pub unsafe fn HdmaobjPreInstr_LavaAcidBG3YScroll(k: uint16, game: &mut Game) {
    let mut current_block: u64;
    let mut v2: int16 = 0;
    let mut v3: int16 = 0;
    let mut v4: int16 = 0;
    let mut v8: int16 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1984) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) == 0 {
        CallFxRisingFunc(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x196c) as *mut uint16)) as u32)
                | (0x880000u32),
            game,
        );
        FxHandleTide(game);
        SetHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0x1962) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0x1960) as *mut uint16,
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x1972) as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0x1970) as *mut uint16) as uint32)
                .wrapping_add(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16)
                        as uint32)
                        << 16 as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8).offset(0x1976) as *mut uint16)
                            as uint32,
                ),
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0xcadc) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c00) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcade) as *mut uint16) = 0 as uint16;
        v2 = (((*((&raw mut game.g_ram as *mut uint8).offset(0x1972) as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x1970) as *mut uint16) as uint32)
            .wrapping_add(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16) as uint32)
                    << 16 as ::core::ffi::c_int
                    | *((&raw mut game.g_ram as *mut uint8).offset(0x1976) as *mut uint16)
                        as uint32,
            )
            >> 16 as ::core::ffi::c_int) as int16;
        if v2 >= 0i16 {
            v3 = (v2).wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)) as i16,
            );
            if v3 <= 0i16 {
                v4 = ((v3 as ::core::ffi::c_int ^ 0x1f as ::core::ffi::c_int)
                    & 0x1f as ::core::ffi::c_int
                    | 0x100 as ::core::ffi::c_int) as int16;
                current_block = 135057599068437321;
            } else if (v3 as uint16) < 0x100u16 {
                v4 = !(v3 as ::core::ffi::c_int) as uint8 as int16;
                current_block = 135057599068437321;
            } else {
                current_block = 4068382217303356765;
            }
        } else {
            current_block = 4068382217303356765;
        }
        match current_block {
            4068382217303356765 => {
                v4 = 0 as int16;
            }
            _ => {}
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(2) as *mut uint8
            as *mut uint16) = v4 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x196e) as *mut uint16) == 2u16
            && (*((&raw mut game.g_ram as *mut uint8).offset(0x1962) as *mut uint16)
                as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int)
                == 0i32
        {
            let v5: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x18b2)
                as *mut uint16) as uint8
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            let ref mut fresh101 = *((&raw mut game.g_ram as *mut uint8).offset(0x192c)
                as *mut uint16)
                .offset(v5 as isize);
            let fresh102 = *fresh101;
            *fresh101 = (*fresh101).wrapping_sub(1);
            if fresh102 == 1u16 {
                *((&raw mut game.g_ram as *mut uint8).offset(0x192c) as *mut uint16)
                    .offset(v5 as isize) = 112 as uint16;
                QueueSfx2_Max6(
                    kLavaSoundEffects[(*((&raw mut game.g_ram as *mut uint8).offset(0x5e5)
                        as *mut uint16)
                        as ::core::ffi::c_int
                        & 7 as ::core::ffi::c_int) as usize] as uint16,
                    game,
                );
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) =
            u16::swap_bytes(*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16));
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x1962) as *mut uint16)
            as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            == 0i32
        {
            v8 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1962) as *mut uint16)
                as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                    as ::core::ffi::c_int
                + 256 as ::core::ffi::c_int) as int16;
            if v8 < 0i16 {
                v8 = 255 as int16;
                current_block = 7317190685143662286;
            } else if (v8 as uint16) < 0x200u16 {
                current_block = 7317190685143662286;
            } else {
                current_block = 11042950489265723346;
            }
        } else {
            current_block = 11042950489265723346;
        }
        match current_block {
            11042950489265723346 => {
                v8 = 511 as int16;
            }
            _ => {}
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x18d8) as *mut uint16).offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x18b2) as *mut uint16) as uint8
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) = (3 as ::core::ffi::c_int
            * ((v8 as ::core::ffi::c_int ^ 0x1ff as ::core::ffi::c_int) + 1 as ::core::ffi::c_int
                & 0x3ff as ::core::ffi::c_int)
            + 0xb62a as ::core::ffi::c_int) as uint16;
    } else {
    }
}
pub unsafe fn HdmaobjInstr_SetFlagB_Copy(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
    hdp
}
pub unsafe fn HdmaobjPreInstr_LavaAcidBG2YScroll(k: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c44) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) == 0
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x197e) as *mut uint16)
            as ::core::ffi::c_int
            & 6 as ::core::ffi::c_int)
            != 0i32
    {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x197e) as *mut uint16)
            as ::core::ffi::c_int
            & 2 as ::core::ffi::c_int)
            != 0i32
        {
            Handle_LavaAcidBG2YScroll_Func3(k, game);
        } else {
            Handle_LavaAcidBG2YScroll_Func2(k, game);
        }
    } else {
        Handle_LavaAcidBG2YScroll_Func1(k, game);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x18d8) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x18b2) as *mut uint16) as uint8
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) = (3 as ::core::ffi::c_int
        * (*((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16)
            as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)
        + 0xc0b1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Handle_LavaAcidBG2YScroll_Func1(v0: uint16, game: &mut Game) {
    WriteReg(
        (((*((&raw mut game.g_ram as *mut uint8).offset(0x18c0) as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize)) as u32)
            .wrapping_add(17153u32)) as uint16,
        0x10 as uint8,
        game,
    );
    let mut v1: uint8 = 30 as uint8;
    let v2: uint16 =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16)) & (0x1ffu16);
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c46) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v2;
        v1 = (v1).wrapping_sub(2u8);
        if !((v1 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) == 0i32) {
            break;
        }
    }
}
pub unsafe fn Handle_LavaAcidBG2YScroll_Func2(v0: uint16, game: &mut Game) {
    WriteReg(
        (((*((&raw mut game.g_ram as *mut uint8).offset(0x18c0) as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize)) as u32)
            .wrapping_add(17153u32)) as uint16,
        0xf as uint8,
        game,
    );
    let v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh54 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16).offset(v1 as isize);
    let fresh55 = *fresh54;
    *fresh54 = (*fresh54).wrapping_sub(1);
    if fresh55 == 1u16 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16).offset(v1 as isize) =
            6 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(v1 as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - 2 as ::core::ffi::c_int
                & 0x1e as ::core::ffi::c_int) as uint16;
    }
    let mut v3: uint8 = *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16
        as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize);
    let mut v4: uint8 = 30 as uint8;
    let mut n: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c46) as *mut uint16)
            .offset((v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = (*(game
            .g_rom
            .offset(crate::rtl::lorom_offset(0x88b589))
            as *const uint8
            as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16)
                as ::core::ffi::c_int
            & 0x1ff as ::core::ffi::c_int)
            as uint16;
        v3 = (v3 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
            as uint8;
        v4 = (v4 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
            as uint8;
        n -= 1;
        if !(n >= 0i32) {
            break;
        }
    }
}
pub unsafe fn Handle_LavaAcidBG2YScroll_Func3(v0: uint16, game: &mut Game) {
    WriteReg(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x18c0) as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + BBAD0 as ::core::ffi::c_int) as SnesRegs as uint16,
        0x10 as uint8,
        game,
    );
    let v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh56 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16).offset(v1 as isize);
    let fresh57 = *fresh56;
    *fresh56 = (*fresh56).wrapping_sub(1);
    if fresh57 == 1u16 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16).offset(v1 as isize) =
            4 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(v1 as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - 2 as ::core::ffi::c_int
                & 0x1e as ::core::ffi::c_int) as uint16;
    }
    let mut v3: uint8 = (*(((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(0) as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int
            * (*((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16)
                as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)
        & 0x1e as ::core::ffi::c_int) as uint8;
    let mut v4: uint8 = (2 as ::core::ffi::c_int
        * (*((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16)
            as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)
        + 30 as ::core::ffi::c_int
        & 0x1e as ::core::ffi::c_int) as uint8;
    let mut n: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c46) as *mut uint16)
            .offset((v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = (*(game
            .g_rom
            .offset(crate::rtl::lorom_offset(0x88b60a))
            as *const uint8
            as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16)
                as ::core::ffi::c_int
            & 0x1ff as ::core::ffi::c_int)
            as uint16;
        v4 = (v4 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
            as uint8;
        v3 = (v3 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
            as uint8;
        n -= 1;
        if !(n >= 0i32) {
            break;
        }
    }
}
static unk_88C40F: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0x11 as uint8,
    addr: 0xd856 as VoidP,
};
static unk_88C41F: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0xf as uint8,
    addr: 0xd847 as VoidP,
};
pub unsafe fn FxTypeFunc_6_Water(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x196c) as *mut uint16) = ((0x88c428i32) as u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x195e) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16);
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88C40F as *const ::core::ffi::c_void,
        game,
    );
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x197e) as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int)
        != 0i32
    {
        SpawnHdmaObject(
            0x88 as uint8,
            &raw const unk_88C41F as *const ::core::ffi::c_void,
            game,
        );
    }
    SpawnBG3ScrollHdmaObject(game);
}
pub unsafe fn FxRisingFunction_C428_WaterNormal(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) != 0 {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16)
            as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            == 0i32
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16)
                < *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16)
                || *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16)
                    == *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16)
            {
                return;
            }
        } else if !(*((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16)
            != *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16)
            && *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16)
                < *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16))
        {
            return;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x196c) as *mut uint16) =
            ((0x88c44ci32) as u16);
    }
}
pub unsafe fn FxRisingFunction_WaterWaitToRise(game: &mut Game) {
    let ref mut fresh46 = *((&raw mut game.g_ram as *mut uint8).offset(0x1980) as *mut uint16);
    *fresh46 = (*fresh46).wrapping_sub(1);
    if *fresh46 == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x196c) as *mut uint16) =
            ((0x88c458i32) as u16);
    }
}
pub unsafe fn FxRisingFunction_WaterRising(game: &mut Game) {
    if RaiseOrLowerFx(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x196c) as *mut uint16) =
            ((0x88c428i32) as u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn HdmaobjInstr_SetFlagB_Copy2(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
    hdp
}
pub unsafe fn CallFxRisingFunction(ea: uint32, game: &mut Game) {
    match ea {
        8958787 => {
            FxRisingFunction_LavaAcid(game);
        }
        8958823 => {
            FxRisingFunction_LavaAcid_WaitToRise(game);
        }
        8958850 => {
            FxRisingFunction_LavaAcid_Raising(game);
        }
        8963112 => {
            FxRisingFunction_C428_WaterNormal(game);
        }
        8963148 => {
            FxRisingFunction_WaterWaitToRise(game);
        }
        8963160 => {
            FxRisingFunction_WaterRising(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn HdmaobjPreInstr_WaterBG3XScroll(k: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1984) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) != 0 {
        return;
    }
    CallFxRisingFunction(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x196c) as *mut uint16)) as u32)
            | (0x880000u32),
        game,
    );
    FxHandleTide(game);
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0x195e) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0x195c) as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1972) as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x1970) as *mut uint16) as uint32)
            .wrapping_add(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16) as uint32)
                    << 16 as ::core::ffi::c_int
                    | *((&raw mut game.g_ram as *mut uint8).offset(0x1976) as *mut uint16)
                        as uint32,
            ),
    );
    let v3: int16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x195e) as *mut uint16)) as i16)
        .wrapping_sub((*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)) as i16);
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x195e) as *mut uint16) as int16) < 0i16 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcade) as *mut uint16) = 0 as uint16;
    } else if v3 <= 0i16 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcade) as *mut uint16) =
            ((v3 as ::core::ffi::c_int ^ 0x1f as ::core::ffi::c_int) & 0x1f as ::core::ffi::c_int
                | 0x100 as ::core::ffi::c_int) as uint16;
    } else if (v3 as uint16) < 0x100u16 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcade) as *mut uint16) =
            !(v3 as ::core::ffi::c_int) as uint8 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcade) as *mut uint16) = 0 as uint16;
    }
    let v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let r20: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16))
        .wrapping_add(
            (*(((&raw mut game.g_ram as *mut uint8).offset(0x192c) as *mut uint16)
                .offset(v5 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ) as int8) as u16,
        );
    let ref mut fresh98 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16).offset(v5 as isize);
    let fresh99 = *fresh98;
    *fresh98 = (*fresh98).wrapping_sub(1);
    if fresh99 == 1u16 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16).offset(v5 as isize) =
            10 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(v5 as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
                .offset(v5 as isize) as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int
                & 0x1e as ::core::ffi::c_int) as uint16;
    }
    let mut v7: uint8 = *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
        .offset(v5 as isize) as uint8;
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while (i & 0x80 as ::core::ffi::c_int) == 0i32 {
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9c00)
            .offset((v7 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut uint16) =
            ((g_word_88C46E[(i >> 1 as ::core::ffi::c_int) as usize]) as u16).wrapping_add(r20);
        v7 = (v7 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
            as uint8;
        i -= 2 as ::core::ffi::c_int;
    }
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x197e) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int)
        != 0i32
    {
        let ref mut fresh100 = *((&raw mut game.g_ram as *mut uint8).offset(0x192c) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh100 = (*fresh100).wrapping_add(64u16);
    }
    let mut v9: int16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x195e) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) as ::core::ffi::c_int
        + 256 as ::core::ffi::c_int) as int16;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x195e) as *mut uint16) as int16) < 0i16 {
        v9 = 511 as int16;
    } else if v9 >= 0i16 {
        if (v9 as uint16) >= 0x200u16 {
            v9 = 511 as int16;
        }
    } else {
        v9 = (v9 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int | 0x100 as ::core::ffi::c_int)
            as int16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x18d8) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (3 as ::core::ffi::c_int
            * ((v9 as ::core::ffi::c_int ^ 0x1ff as ::core::ffi::c_int) + 1 as ::core::ffi::c_int
                & 0x3ff as ::core::ffi::c_int)
            + 0xc645 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn HdmaobjInstr_SetFlagB_Copy3(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
    hdp
}
pub unsafe fn HdmaobjPreInstr_WaterBG2XScroll(k: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c44) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) != 0 {
        HdmaobjPreInstr_WaterBG2XScroll_Func1(k, game);
    } else if (*((&raw mut game.g_ram as *mut uint8).offset(0x197e) as *mut uint16)
        as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int)
        == 0i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x18b4) as *mut uint16)
            .offset((k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            0 as uint16;
        HdmaobjPreInstr_WaterBG2XScroll_Func1(k, game);
    } else {
        HdmaobjPreInstr_WaterBG2XScroll_Func2(k, game);
    }
    let mut v1: int16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x195e) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) as ::core::ffi::c_int
        + 256 as ::core::ffi::c_int) as int16;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x195e) as *mut uint16) as int16) < 0i16 {
        v1 = 511 as int16;
    } else if v1 >= 0i16 {
        if (v1 as uint16) >= 0x200u16 {
            v1 = 511 as int16;
        }
    } else {
        v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0x195e) as *mut uint16)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            | 0x100 as ::core::ffi::c_int) as int16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x18d8) as *mut uint16)
        .offset((k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = (3
        as ::core::ffi::c_int
        * ((v1 as ::core::ffi::c_int ^ 0x1ff as ::core::ffi::c_int) & 0x3ff as ::core::ffi::c_int)
        + 0xcf46 as ::core::ffi::c_int)
        as uint16;
}
pub unsafe fn HdmaobjPreInstr_WaterBG2XScroll_Func2(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh96 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16).offset(v1 as isize);
    let fresh97 = *fresh96;
    *fresh96 = (*fresh96).wrapping_sub(1);
    if fresh97 == 1u16 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16).offset(v1 as isize) =
            6 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(v1 as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int
                & 0x1e as ::core::ffi::c_int) as uint16;
    }
    let mut v3: uint8 = (*(((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(0) as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int
            * (*((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16)
                as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)
        & 0x1e as ::core::ffi::c_int) as uint8;
    let mut v4: uint8 = (2 as ::core::ffi::c_int
        * (*((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16)
            as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)
        + 30 as ::core::ffi::c_int
        & 0x1e as ::core::ffi::c_int) as uint8;
    let mut n: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c46) as *mut uint16).offset(
            ((v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                as isize,
        ) = ((g_word_88C46E[(v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize])
            as u16)
            .wrapping_add(*((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16));
        v4 = (v4 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
            as uint8;
        v3 = (v3 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
            as uint8;
        n -= 1;
        if !(n >= 0i32) {
            break;
        }
    }
}
pub unsafe fn HdmaobjPreInstr_WaterBG2XScroll_Func1(k: uint16, game: &mut Game) {
    let mut v1: uint8 = 30 as uint8;
    let v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16);
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c46) as *mut uint16).offset(
            ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                as isize,
        ) = v2;
        v1 = (v1).wrapping_sub(2u8);
        if !((v1 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) == 0i32) {
            break;
        }
    }
}
pub unsafe fn SpawnBG3ScrollHdmaObject(game: &mut Game) {
    static unk_88D869: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x43 as uint8,
        field_1: 0x11 as uint8,
        addr: 0xd8d0 as VoidP,
    };
    SpawnHdmaObjectToSlot0xA(
        0x88 as uint8,
        &raw const unk_88D869 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn FxTypeFunc_28_CeresRidley(game: &mut Game) {
    static unk_88D8E2: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x40 as uint8,
        field_1: 0x5 as uint8,
        addr: 0xd906 as VoidP,
    };
    static unk_88D8EA: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0 as uint8,
        field_1: 0x2c as uint8,
        addr: 0xd91d as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88D8E2 as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88D8EA as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn sub_88D916(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x7eb) as *mut uint16) = 9 as uint16;
}
pub unsafe fn FxTypeFunc_CeresElevator(game: &mut Game) {
    static unk_88D92C: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x40 as uint8,
        field_1: 0x5 as uint8,
        addr: 0xd939 as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88D92C as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn HdmaobjInstr_SetVideoMode1(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x7eb) as *mut uint16) = 9 as uint16;
    hdp
}
pub unsafe fn FxTypeFunc_A_Rain(game: &mut Game) {
    static unk_88D95C: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x43 as uint8,
        field_1: 0x11 as uint8,
        addr: 0xd96c as VoidP,
    };
    *(&raw mut game.g_ram as *mut uint8).offset(0x5b) = 92 as uint8;
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88D95C as *const ::core::ffi::c_void,
        game,
    );
    SpawnBG3ScrollHdmaObject(game);
    SpawnAnimtiles(addr_kAnimtiles_Rain as uint16, game);
}
pub unsafe fn HdmaobjInstr_1938_RandomNumber(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1938) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x88d992)) as *const uint8 as *mut uint16)
            .offset(
                ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int
                    & 6 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
    hdp
}
pub unsafe fn HdmaobjPreInstr_RainBg3Scroll(k: uint16, game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x5b) = 92 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1984) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) == 0 {
        let v1: ::core::ffi::c_int = k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcade) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1944) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                    as ::core::ffi::c_int
                + *(((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
                    .offset(v1 as isize) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ) as int8 as ::core::ffi::c_int) as uint16;
        let ref mut fresh94 = *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
            .offset(v1 as isize);
        *fresh94 = (*fresh94).wrapping_sub(1536u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1944) as *mut uint16).offset(v1 as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xcadc) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1950) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                    as ::core::ffi::c_int
                + *(((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16)
                    .offset(v1 as isize) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ) as int8 as ::core::ffi::c_int) as uint16;
        let ref mut fresh95 = *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16)
            .offset(v1 as isize);
        *fresh95 = (*fresh95).wrapping_add(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1938) as *mut uint16)
                .offset(v1 as isize),
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x1950) as *mut uint16).offset(v1 as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16);
    }
}
pub unsafe fn FxTypeFunc_8_Spores(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x5b) = 92 as uint8;
    static unk_88DA1D: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x42 as uint8,
        field_1: 0x11 as uint8,
        addr: 0xda2d as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DA1D as *const ::core::ffi::c_void,
        game,
    );
    SpawnBG3ScrollHdmaObject(game);
    SpawnAnimtiles(addr_kAnimtiles_Spores as uint16, game);
}
pub unsafe fn HdmaobjPreInstr_SporesBG3Xscroll(k: uint16, game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x5b) = 92 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1984) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) == 0 {
        let v1: ::core::ffi::c_int = k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcade) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)).wrapping_add(
                (*(((&raw mut game.g_ram as *mut uint8).offset(0x192c) as *mut uint16)
                    .offset(v1 as isize) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ) as int8) as u16,
            );
        let v3: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x192c) as *mut uint16)
            .offset(v1 as isize))
        .wrapping_sub(64u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x192c) as *mut uint16).offset(v1 as isize) =
            v3;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcadc) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)).wrapping_add(
                (*(((&raw mut game.g_ram as *mut uint8).offset(0x1938) as *mut uint16)
                    .offset(v1 as isize) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ) as int8) as u16,
            );
        *((&raw mut game.g_ram as *mut uint8).offset(0x1938) as *mut uint16).offset(v1 as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1938) as *mut uint16)
                .offset(v1 as isize);
    }
}
pub unsafe fn FxTypeFunc_C(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x5b) = 92 as uint8;
    static unk_88DB14: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x43 as uint8,
        field_1: 0x11 as uint8,
        addr: 0xdb19 as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DB14 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn HdmaobjPreInstr_FogBG3Scroll(k: uint16, game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x5b) = 92 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1984) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) == 0 {
        let v1: ::core::ffi::c_int = k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcade) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)).wrapping_add(
                (*(((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
                    .offset(v1 as isize) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ) as int8) as u16,
            );
        let ref mut fresh92 = *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
            .offset(v1 as isize);
        *fresh92 = (*fresh92).wrapping_sub(64u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xcadc) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)).wrapping_add(
                (*(((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16)
                    .offset(v1 as isize) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ) as int8) as u16,
            );
        let ref mut fresh93 = *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16)
            .offset(v1 as isize);
        *fresh93 = (*fresh93).wrapping_add(80u16);
    }
}
pub unsafe fn FxTypeFunc_26_TourianEntranceStatue(game: &mut Game) {
    static unk_88DBBA: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x42 as uint8,
        field_1: 0x11 as uint8,
        addr: 0xd856 as VoidP,
    };
    static unk_88DBC2: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x42 as uint8,
        field_1: 0x10 as uint8,
        addr: 0xdcfa as VoidP,
    };
    if CheckEventHappened(0xa as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x6 as uint8,
                field_1: 0xc as uint8,
                field_2: 0xb777 as VoidP,
            },
            game,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20) as *mut uint16) = 514 as uint16;
    }
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 74 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x196c) as *mut uint16) = ((0x88c428i32) as u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x195e) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16);
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DBBA as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DBC2 as *const ::core::ffi::c_void,
        game,
    );
    SpawnBG3ScrollHdmaObject(game);
}
pub unsafe fn sub_88DBCB(k: uint16, game: &mut Game) {
    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0 as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)).wrapping_add(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
        );
}
pub unsafe fn HdmaobjPreInstr_CheckLotsOfEventsHappened(mut v0: uint16, game: &mut Game) {
    if CheckEventHappened(6 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        if CheckEventHappened(7 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
        {
            if CheckEventHappened(8 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
                != 0
            {
                if CheckEventHappened(9 as uint16, game) as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                    != 0
                {
                    let ref mut fresh90 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0x1e6f) as *mut uint16);
                    *fresh90 = (*fresh90) | (0x10u16);
                    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1e6f) as *mut uint16)
                        as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int)
                        == 0i32
                    {
                        *((&raw mut game.g_ram as *mut uint8).offset(0x192c) as *mut uint16)
                            .offset(
                                (v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                            ) = 300 as uint16;
                        v0 = *((&raw mut game.g_ram as *mut uint8).offset(0x18b2) as *mut uint16);
                        let v1: ::core::ffi::c_int =
                            *((&raw mut game.g_ram as *mut uint8).offset(0x18b2) as *mut uint16)
                                as ::core::ffi::c_int
                                >> 1 as ::core::ffi::c_int;
                        *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16)
                            .offset(v1 as isize) = 1 as uint16;
                        let ref mut fresh91 = *((&raw mut game.g_ram as *mut uint8).offset(0x18cc)
                            as *mut uint16)
                            .offset(v1 as isize);
                        *fresh91 = (*fresh91).wrapping_add(2u16);
                    }
                }
            }
        }
    }
    sub_88DBCB(v0, game);
}
pub unsafe fn HdmaobjPreInstr_DC23(k: uint16, game: &mut Game) {
    let v0: uint16 = k;
    HandleEarthquakeSoundEffect(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 13 as uint16;
    let ref mut fresh87 = *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16);
    *fresh87 = (*fresh87) | (0x20u16);
    let v1: ::core::ffi::c_int = v0 as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh88 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x192c) as *mut uint16).offset(v1 as isize);
    *fresh88 = (*fresh88).wrapping_sub(1);
    if (*fresh88 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        SpawnEprojWithRoomGfx(
            addr_kEproj_TourianStatueDustClouds as uint16,
            0 as uint16,
            game,
        );
        SpawnEprojWithRoomGfx(
            addr_kEproj_TourianStatueDustClouds as uint16,
            0 as uint16,
            game,
        );
        SpawnEprojWithRoomGfx(
            addr_kEproj_TourianStatueDustClouds as uint16,
            0 as uint16,
            game,
        );
        SpawnEprojWithRoomGfx(
            addr_kEproj_TourianStatueDustClouds as uint16,
            0 as uint16,
            game,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        let ref mut fresh89 = *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16)
            .offset(v1 as isize);
        *fresh89 = (*fresh89).wrapping_add(2u16);
    }
    sub_88DBCB(v0, game);
}
pub unsafe fn HdmaobjPreInstr_DC69(k: uint16, game: &mut Game) {
    HandleEarthquakeSoundEffect(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 13 as uint16;
    let ref mut fresh85 = *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16);
    *fresh85 = (*fresh85) | (0x20u16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) == 0 {
        let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        AddToHiLo(
            ((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            ((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            -(0x4000 as ::core::ffi::c_int) as uint32,
        );
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16).offset(v1 as isize)
            == 0xff10u16
        {
            SpawnHardcodedPlm(
                SpawnHardcodedPlmArgs {
                    field_0: 0x6 as uint8,
                    field_1: 0xc as uint8,
                    field_2: 0xb773 as VoidP,
                },
                game,
            );
            SetEventHappened(0xa as uint16, game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16)
                .offset(v1 as isize) = 1 as uint16;
            let ref mut fresh86 = *((&raw mut game.g_ram as *mut uint8).offset(0x18cc)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh86 = (*fresh86).wrapping_add(2u16);
        }
        sub_88DBCB(k, game);
    }
}
pub unsafe fn HdmaobjPreInstr_DCBA(v0: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e6d) as *mut uint16) = 0x8000 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd20) as *mut uint16) = 514 as uint16;
    sub_88DBCB(v0, game);
}
pub unsafe fn HdmaobjInstr_GotoIfEventHappened(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x192c) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    if CheckEventHappened(0xa as uint16, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16).offset(v2 as isize) =
            -(240 as ::core::ffi::c_int) as uint16;
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0 as *mut uint16) = -(240 as ::core::ffi::c_int) as uint16;
        *(hdp as *mut uint16) as uintptr_t as *const uint8
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16).offset(v2 as isize) =
            0 as uint16;
        *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0 as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20) as *mut uint16) = 1 as uint16;
        hdp.offset(2)
    }
}
pub unsafe fn SpawnBombTorizoHaze(game: &mut Game) {
    static unk_88DD36: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x2 as uint8,
        field_1: 0x32 as uint8,
        addr: 0xdd4a as VoidP,
    };
    static unk_88DD3E: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0 as uint8,
        field_1: 0x32 as uint8,
        addr: 0xdd62 as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DD36 as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DD3E as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn HdmaobjPreInstr_BombTorizoHazeColorMathBgColor(k: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) = 44 as uint16;
}
pub unsafe fn FxTypeFunc_2C_Haze(game: &mut Game) {
    static unk_88DDD4: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x40 as uint8,
        field_1: 0x32 as uint8,
        addr: 0xded3 as VoidP,
    };
    static unk_88DDDD: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x40 as uint8,
        field_1: 0x32 as uint8,
        addr: 0xdeeb as VoidP,
    };
    if CheckBossBitForCurArea(1 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0
    {
        SpawnHdmaObject(
            0x88 as uint8,
            &raw const unk_88DDDD as *const ::core::ffi::c_void,
            game,
        );
    } else {
        SpawnHdmaObject(
            0x88 as uint8,
            &raw const unk_88DDD4 as *const ::core::ffi::c_void,
            game,
        );
    };
}
pub unsafe fn HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyAlive(k: uint16, game: &mut Game) {
    sub_88DE18(k, 0x80 as uint16, game);
}
pub unsafe fn HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyDead(k: uint16, game: &mut Game) {
    sub_88DE18(k, 0x20 as uint16, game);
}
pub unsafe fn sub_88DE18(k: uint16, a: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16).offset(v2 as isize) = a;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x99c) as *mut uint16) as ::core::ffi::c_int
        == 0x82e737 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x18f0) as *mut uint16).offset(v2 as isize) =
            ((0x88de2di32) as u16);
        HdmaobjPreInstr_HazeColorMathSubscreen_FadingIn(k, game);
    }
}
pub unsafe fn HdmaobjPreInstr_HazeColorMathSubscreen_FadingIn(k: uint16, game: &mut Game) {
    let mut v1: int8 = 0;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) = 44 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
        .offset((k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        == 16u16
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x18f0) as *mut uint16)
            .offset((k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            ((0x88de74i32) as u16);
    } else {
        let v3: uint8 = k as uint8;
        let r20: uint8 = *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16
            as *mut uint8)
            .offset(k as uint8 as ::core::ffi::c_int as isize);
        v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16 as *mut uint8)
            .offset(k as uint8 as ::core::ffi::c_int as isize) as int8;
        let mut i: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
        while i >= 0i32 {
            let r18: uint8 = v1 as uint8;
            *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16 as *mut uint8)
                .offset(i as uint8 as ::core::ffi::c_int as isize) = (r20) | ((v1) as u8);
            v1 = ((r18) as i8).wrapping_sub(1i8);
            if ((r18) as i8).wrapping_sub(1i8) < 0i8 {
                v1 = 0 as int8;
            }
            i -= 1;
        }
        let ref mut fresh84 = *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh84 = (*fresh84).wrapping_add(1);
    };
}
pub unsafe fn HdmaobjPreInstr_HazeColorMathSubscreen_FadedIn(k: uint16, game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) = 44 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x99c) as *mut uint16) as ::core::ffi::c_int
        == 0x82e2db as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x18f0) as *mut uint16)
            .offset((k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            ((0x88de96i32) as u16);
    }
}
pub unsafe fn HdmaobjPreInstr_HazeColorMathSubscreen_FadingOut(k: uint16, game: &mut Game) {
    let mut v1: int8 = 0;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) = 44 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
        .offset((k as uint8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        != 0
    {
        let v3: uint8 = k as uint8;
        let r20: uint8 = *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16
            as *mut uint8)
            .offset(k as uint8 as ::core::ffi::c_int as isize);
        v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16 as *mut uint8)
            .offset(k as uint8 as ::core::ffi::c_int as isize) as int8;
        let mut i: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
        while i >= 0i32 {
            let r18: uint8 = v1 as uint8;
            *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16 as *mut uint8)
                .offset(i as uint8 as ::core::ffi::c_int as isize) = (r20) | ((v1) as u8);
            v1 = ((r18) as i8).wrapping_sub(1i8);
            if ((r18) as i8).wrapping_sub(1i8) < 0i8 {
                v1 = 0 as int8;
            }
            i -= 1;
        }
        let ref mut fresh83 = *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh83 = (*fresh83).wrapping_sub(1);
    }
}
static unk_88DF38: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0 as uint8,
    field_1: 0x2c as uint8,
    addr: 0xdf4f as VoidP,
};
static unk_88DF41: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0 as uint8,
    field_1: 0x2c as uint8,
    addr: 0xdf6b as VoidP,
};
static unk_88DF4A: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x2 as uint8,
    field_1: 0x12 as uint8,
    addr: 0xdf77 as VoidP,
};
pub unsafe fn sub_88DF34(game: &mut Game) {
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DF38 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn sub_88DF3D(game: &mut Game) {
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DF41 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn sub_88DF46(game: &mut Game) {
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88DF4A as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn HdmaobjPreInstr_DF94(v0: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .properties as ::core::ffi::c_int
        & 0x200 as ::core::ffi::c_int)
        == 0i32
        && (((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .x_pos as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int) as int16)
            >= 0i16
        && (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                as ::core::ffi::c_int
            - 320 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        && (((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .y_pos as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as int16)
            >= 0i16
        && {
            v1 = (((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
                .offset(0))
            .y_pos) as i16)
                .wrapping_sub(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)) as i16,
                );
            (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).y_pos
                as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                    as ::core::ffi::c_int
                - 304 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
        }
    {
        if v1 as ::core::ffi::c_int - 40 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            let v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16)
                .offset(v2 as isize) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16)
                .offset(v2 as isize) = addr_word_88DF6B as uint16;
        } else {
            let v4: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16)
                .offset(v4 as isize) = 1 as uint16;
            if v1 as ::core::ffi::c_int - 192 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16)
                    .offset(v4 as isize) = addr_word_88DF5F as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16)
                    .offset(v4 as isize) = addr_word_88DF65 as uint16;
            }
        }
    } else {
        let v3: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16).offset(v3 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16).offset(v3 as isize) =
            addr_word_88DF71 as uint16;
    };
}
static off_88E04E: [Option<Func_U8>; 7] = unsafe {
    [
        Some(VariaSuitPickup_0_LightBeamAppears as unsafe fn(&mut Game) -> uint8),
        Some(VariaSuitPickup_1 as unsafe fn(&mut Game) -> uint8),
        Some(VariaSuitPickup_2_LightBeamWidens as unsafe fn(&mut Game) -> uint8),
        Some(VariaSuitPickup_3 as unsafe fn(&mut Game) -> uint8),
        Some(VariaSuitPickup_4_LightBeamShrinks as unsafe fn(&mut Game) -> uint8),
        Some(VariaSuitPickup_5_LightBeamDissipates as unsafe fn(&mut Game) -> uint8),
        Some(VariaSuitPickup_6 as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn HdmaobjPreInstr_VariaSuitPickup(k: uint16, game: &mut Game) {
    if off_88E04E[*((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) as usize]
        .expect("non-null function pointer")(game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) =
            *(&raw mut game.g_ram as *mut uint8).offset(0xdf0);
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) =
            *(&raw mut game.g_ram as *mut uint8).offset(0xdf1);
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) =
            *(&raw mut game.g_ram as *mut uint8).offset(0xdf2);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) = 18 as uint16;
    }
}
static off_88E084: [Option<Func_U8>; 7] = unsafe {
    [
        Some(VariaSuitPickup_0_LightBeamAppears as unsafe fn(&mut Game) -> uint8),
        Some(VariaSuitPickup_1 as unsafe fn(&mut Game) -> uint8),
        Some(VariaSuitPickup_2_LightBeamWidens as unsafe fn(&mut Game) -> uint8),
        Some(GravitySuitPickup_3 as unsafe fn(&mut Game) -> uint8),
        Some(VariaSuitPickup_4_LightBeamShrinks as unsafe fn(&mut Game) -> uint8),
        Some(VariaSuitPickup_5_LightBeamDissipates as unsafe fn(&mut Game) -> uint8),
        Some(GravitySuitPickup_6 as unsafe fn(&mut Game) -> uint8),
    ]
};
pub unsafe fn HdmaobjPreInstr_GravitySuitPickup(k: uint16, game: &mut Game) {
    if off_88E084[*((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) as usize]
        .expect("non-null function pointer")(game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) =
            *(&raw mut game.g_ram as *mut uint8).offset(0xdf0);
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) =
            *(&raw mut game.g_ram as *mut uint8).offset(0xdf1);
        *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) =
            *(&raw mut game.g_ram as *mut uint8).offset(0xdf2);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) = 18 as uint16;
    }
}
static g_word_88E0D5: uint16 = 0x7878 as uint16;
pub unsafe fn VariaSuitPickup_0_LightBeamAppears(game: &mut Game) -> uint8 {
    let mut v0: int16 = 0;
    let mut v3: int16 = 0;
    let ref mut fresh15 = *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16);
    *fresh15 = (*fresh15).wrapping_add(8u16);
    v0 = *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) as int16;
    let mut v1: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = g_word_88E0D5;
        v1 = (v1).wrapping_add(2u16);
        v0 -= 1;
        if !(v0 > 0i16) {
            break;
        }
    }
    let mut v2: uint16 = 510 as uint16;
    v3 = *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) as int16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16)
            .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = g_word_88E0D5;
        v2 = (v2).wrapping_sub(2u16);
        v3 -= 1;
        if !(v3 > 0i16) {
            break;
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) as ::core::ffi::c_int
        - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        let ref mut fresh16 = *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16);
        *fresh16 = (*fresh16).wrapping_add(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) = 30840 as uint16;
    }
    1 as uint8
}
pub unsafe fn VariaSuitPickup_1(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16 as *mut uint8).offset(0) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16)) as u8).wrapping_sub(
            *((&raw mut game.g_ram as *mut uint8).offset(0xddc) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ),
        );
    let ref mut fresh17 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        );
    *fresh17 = (*fresh17).wrapping_add(
        *((&raw mut game.g_ram as *mut uint8).offset(0xddc) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ),
    );
    let mut i: ::core::ffi::c_int = 510 as ::core::ffi::c_int;
    while i >= 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16);
        i -= 2 as ::core::ffi::c_int;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) as uint8
        as ::core::ffi::c_int
        - 97 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh18 = *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16);
        *fresh18 = (*fresh18).wrapping_add(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) = 0x846c as uint16;
    }
    1 as uint8
}
pub unsafe fn VariaSuitPickup_2_LightBeamWidens(game: &mut Game) -> uint8 {
    let mut v1: int8 = 0;
    let mut v2: int16 = 0;
    let mut v4: int8 = 0;
    let mut v6: int8 = 0;
    let mut v8: int8 = 0;
    let mut v10: int8 = 0;
    AdvanceSuitPickupColorMathSubscreenToWhite(game);
    let v0: bool =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16)) as i8).wrapping_sub(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xddc) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                )) as i8,
        ) < 0i8;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16 as *mut uint8).offset(0) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16)) as u8).wrapping_sub(
            *((&raw mut game.g_ram as *mut uint8).offset(0xddc) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ),
        );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) as uint8 == 0 || v0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) =
            -(256 as ::core::ffi::c_int) as uint16;
    } else {
        v1 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xddc) as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            )) as i8)
            .wrapping_add(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    )) as i8,
            );
        if u8::overflowing_add(
            *((&raw mut game.g_ram as *mut uint8).offset(0xddc) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ),
            *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ),
        )
        .1
        {
            v1 = -(1 as ::core::ffi::c_int) as int8;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) = v1 as uint8;
    }
    v2 = 0 as int16;
    let mut v3: uint16 = 0 as uint16;
    loop {
        v4 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16)) as i8)
            .wrapping_sub(
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0x88e3c9)) as *const uint8
                    as *mut uint8)
                    .offset(v3 as isize)) as i8,
            );
        if v4 < 0i8 {
            v4 = 0 as int8;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16 as *mut uint8)
            .offset(v2 as uint16 as ::core::ffi::c_int as isize) = v4 as uint8;
        let v5: uint16 = ((v2) as u16).wrapping_add(1u16);
        v6 = ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x88e3c9)) as *const uint8
            as *mut uint8)
            .offset(v3 as isize)) as i8)
            .wrapping_add(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    )) as i8,
            );
        if u8::overflowing_add(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88e3c9)) as *const uint8 as *mut uint8)
                .offset(v3 as isize),
            *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ),
        )
        .1
        {
            v6 = -(1 as ::core::ffi::c_int) as int8;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16 as *mut uint8)
            .offset(v5 as ::core::ffi::c_int as isize) = v6 as uint8;
        v2 = ((v5) as i16).wrapping_add(1i16);
        v3 = v3.wrapping_add(1);
        if !(v2 < 0x100i16) {
            break;
        }
    }
    let mut v7: uint16 = (v3).wrapping_sub(1u16);
    loop {
        v8 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16)) as i8)
            .wrapping_sub(
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0x88e3c9)) as *const uint8
                    as *mut uint8)
                    .offset(v7 as isize)) as i8,
            );
        if v8 < 0i8 {
            v8 = 0 as int8;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16 as *mut uint8)
            .offset(v2 as uint16 as ::core::ffi::c_int as isize) = v8 as uint8;
        let v9: uint16 = ((v2) as u16).wrapping_add(1u16);
        v10 = ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x88e3c9)) as *const uint8
            as *mut uint8)
            .offset(v7 as isize)) as i8)
            .wrapping_add(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    )) as i8,
            );
        if u8::overflowing_add(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88e3c9)) as *const uint8 as *mut uint8)
                .offset(v7 as isize),
            *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ),
        )
        .1
        {
            v10 = -(1 as ::core::ffi::c_int) as int8;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16 as *mut uint8)
            .offset(v9 as ::core::ffi::c_int as isize) = v10 as uint8;
        v2 = ((v9) as i16).wrapping_add(1i16);
        v7 = v7.wrapping_sub(1);
        if !(v2 < 0x200i16) {
            break;
        }
    }
    let ref mut fresh19 = *((&raw mut game.g_ram as *mut uint8).offset(0xddc) as *mut uint16);
    *fresh19 = (*fresh19).wrapping_add(96u16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) == 0xff00u16 {
        let ref mut fresh20 = *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16);
        *fresh20 = (*fresh20).wrapping_add(1);
        let ref mut fresh21 = *((&raw mut game.g_ram as *mut uint8).offset(0xddc) as *mut uint16);
        *fresh21 = (*fresh21 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) = 0 as uint16;
    }
    1 as uint8
}
pub unsafe fn VariaSuitPickup_4_LightBeamShrinks(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xdf3) != 0 {
        AdvanceSuitPickupColorMathToBlue(game);
    } else {
        AdvanceSuitPickupColorMathToOrange(game);
    }
    let ref mut fresh25 = *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16);
    *fresh25 = (*fresh25 as ::core::ffi::c_int
        + ((*((&raw mut game.g_ram as *mut uint8).offset(0xddc) as *mut uint16)
            as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int)) as uint16;
    let mut v0: int16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) as int16;
    let mut v1: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 255 as uint16;
        v1 = (v1).wrapping_add(2u16);
        v0 -= 1;
        if !(v0 > 0i16) {
            break;
        }
    }
    let mut v2: uint16 = 510 as uint16;
    let mut v3: int16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16))
        as i16)
        .wrapping_sub(1i16);
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16)
            .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 255 as uint16;
        v2 = (v2).wrapping_sub(2u16);
        v3 -= 1;
        if !(v3 > 0i16) {
            break;
        }
    }
    let ref mut fresh26 = *((&raw mut game.g_ram as *mut uint8).offset(0xddc) as *mut uint16);
    *fresh26 = (*fresh26).wrapping_sub(32u16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xddc) as *mut uint16) as ::core::ffi::c_int
        - 256 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xddc) as *mut uint16) = 256 as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) as ::core::ffi::c_int
        - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        let ref mut fresh27 = *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16);
        *fresh27 = (*fresh27).wrapping_add(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) =
            -(1793 as ::core::ffi::c_int) as uint16;
    }
    1 as uint8
}
pub unsafe fn VariaSuitPickup_5_LightBeamDissipates(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16 as *mut uint8).offset(0) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16)) as u8)
            .wrapping_add(8u8);
    let ref mut fresh28 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        );
    *fresh28 = (*fresh28).wrapping_sub(8u8);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16).offset(128) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) as uint8
        as ::core::ffi::c_int
        - 112 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        let ref mut fresh29 = *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16);
        *fresh29 = (*fresh29).wrapping_add(1);
    }
    1 as uint8
}
pub unsafe fn VariaSuitPickup_6(game: &mut Game) -> uint8 {
    QueueMusic_Delayed8(3 as uint16, game);
    GravitySuitPickup_6(game)
}
pub unsafe fn GravitySuitPickup_6(game: &mut Game) -> uint8 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x80 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 64 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0xa88) as *mut LongPtr,
        0x980001 as uint32,
    );
    *(((&raw mut game.g_ram as *mut uint8).offset(0xa8a) as *mut uint16 as *mut uint8).offset(1)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa8c) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa8e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa90) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16).offset(0) = 255 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdf0) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdf2) as *mut uint16) = 0 as uint16;
    let v0: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x18b2)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let ref mut fresh12 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16).offset(v0 as isize);
    *fresh12 = (*fresh12).wrapping_add(2u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16).offset(v0 as isize) =
        1 as uint16;
    CallSomeSamusCode(0xb as uint16, game);
    0 as uint8
}
pub unsafe fn AdvanceSuitPickupColorMathSubscreenToWhite(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xdf0) != 63u8 {
        let ref mut fresh0 = *(&raw mut game.g_ram as *mut uint8).offset(0xdf0);
        *fresh0 = (*fresh0).wrapping_add(2u8);
        if *(&raw mut game.g_ram as *mut uint8).offset(0xdf0) as ::core::ffi::c_int
            - 64 as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            == 0
        {
            *(&raw mut game.g_ram as *mut uint8).offset(0xdf0) = 63 as uint8;
        }
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xdf1) != 95u8 {
        let ref mut fresh1 = *(&raw mut game.g_ram as *mut uint8).offset(0xdf1);
        *fresh1 = (*fresh1).wrapping_add(2u8);
        if *(&raw mut game.g_ram as *mut uint8).offset(0xdf1) as ::core::ffi::c_int
            - 96 as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            == 0
        {
            *(&raw mut game.g_ram as *mut uint8).offset(0xdf1) = 95 as uint8;
        }
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xdf2) != 0x9fu8 {
        let ref mut fresh2 = *(&raw mut game.g_ram as *mut uint8).offset(0xdf2);
        *fresh2 = (*fresh2).wrapping_add(2u8);
        if *(&raw mut game.g_ram as *mut uint8).offset(0xdf2) as ::core::ffi::c_int
            + 96 as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            == 0
        {
            *(&raw mut game.g_ram as *mut uint8).offset(0xdf2) =
                -(97 as ::core::ffi::c_int) as uint8;
        }
    }
    1 as uint8
}
pub unsafe fn AdvanceSuitPickupColorMathToOrange(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xdf0) != 63u8 {
        let ref mut fresh6 = *(&raw mut game.g_ram as *mut uint8).offset(0xdf0);
        *fresh6 = (*fresh6).wrapping_sub(1);
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xdf1) != 77u8 {
        let ref mut fresh7 = *(&raw mut game.g_ram as *mut uint8).offset(0xdf1);
        *fresh7 = (*fresh7).wrapping_sub(1);
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xdf2) != 0x83u8 {
        let ref mut fresh8 = *(&raw mut game.g_ram as *mut uint8).offset(0xdf2);
        *fresh8 = (*fresh8).wrapping_sub(1);
    }
    1 as uint8
}
pub unsafe fn VariaSuitPickup_3(game: &mut Game) -> uint8 {
    let ref mut fresh22 = *((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16);
    *fresh22 = (*fresh22) | (1u16);
    let ref mut fresh23 = *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16);
    *fresh23 = (*fresh23) | (1u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) =
        kPose_9B_FaceF_VariaGravitySuit as uint16;
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa24) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa20) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa26) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa20) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16);
    Samus_LoadSuitPalette(game);
    let ref mut fresh24 = *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16);
    *fresh24 = (*fresh24).wrapping_add(1);
    1 as uint8
}
pub unsafe fn GravitySuitPickup_3(game: &mut Game) -> uint8 {
    let ref mut fresh9 = *((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16);
    *fresh9 = (*fresh9) | (0x20u16);
    let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16);
    *fresh10 = (*fresh10) | (0x20u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) =
        kPose_9B_FaceF_VariaGravitySuit as uint16;
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa24) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa20) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa26) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa20) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16);
    Samus_LoadSuitPalette(game);
    let ref mut fresh11 = *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16);
    *fresh11 = (*fresh11).wrapping_add(1);
    1 as uint8
}
pub unsafe fn AdvanceSuitPickupColorMathToBlue(game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xdf0) != 48u8 {
        let ref mut fresh3 = *(&raw mut game.g_ram as *mut uint8).offset(0xdf0);
        *fresh3 = (*fresh3).wrapping_sub(1);
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xdf1) != 73u8 {
        let ref mut fresh4 = *(&raw mut game.g_ram as *mut uint8).offset(0xdf1);
        *fresh4 = (*fresh4).wrapping_sub(1);
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xdf2) != 0x90u8 {
        let ref mut fresh5 = *(&raw mut game.g_ram as *mut uint8).offset(0xdf2);
        *fresh5 = (*fresh5).wrapping_sub(1);
    }
    1 as uint8
}
pub unsafe fn HdmaobjPreInstr_E449(k: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1988) as *mut uint16) as ::core::ffi::c_int
        & 0x4000 as ::core::ffi::c_int)
        != 0i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) = 26 as uint16;
    } else if *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
        .offset(3))
    .parameter_1 as *mut uint8)
        .offset(0)
        != 0
    {
        if *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
            .offset(3))
        .parameter_1 as *mut uint8)
            .offset(0)
            == 0xffu8
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) = 4 as uint16;
            let v0: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x18b2)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            let ref mut fresh82 = *((&raw mut game.g_ram as *mut uint8).offset(0x18cc)
                as *mut uint16)
                .offset(v0 as isize);
            *fresh82 = (*fresh82).wrapping_add(2u16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16)
                .offset(v0 as isize) = 1 as uint16;
        }
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) = 4 as uint16;
    };
}
static unk_88E4A0: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x42 as uint8,
    field_1: 0xf as uint8,
    addr: 0xe4a8 as VoidP,
};
pub unsafe fn sub_88E487(v0: uint16, r22: uint16, game: &mut Game) {
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(2))
        .parameter_1 = v0;
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(3)).ai_var_D =
        0 as uint16;
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(3)).ai_var_E =
        0 as uint16;
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(3))
        .ai_preinstr = r22;
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88E4A0 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn HdmaobjInstr_E4BD(k: uint16, hdp: *const uint8, game: &mut Game) -> *const uint8 {
    let mut v2: ::core::ffi::c_uint = 0;
    let mut v3: ::core::ffi::c_uint = 0;
    let mut v4: ::core::ffi::c_uint = 0;
    let mut v5: ::core::ffi::c_uint = 0;
    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1))
        .parameter_1 = (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
        .offset(2))
    .parameter_1;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(2))
        .parameter_1 as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int)
        != 0i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9000) as *mut uint16) = 192 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9006) as *mut uint16) = 192 as uint16;
        v4 = 12620032 as ::core::ffi::c_int as ::core::ffi::c_uint;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9002) as *mut uint16 as *mut uint8)
            .offset(1) as *mut uint16) = *(&raw mut v4 as *mut uint16).offset(
            (::core::mem::size_of::<::core::ffi::c_uint>())
                .wrapping_div(::core::mem::size_of::<uint16>())
                .wrapping_sub(1_usize) as isize,
        );
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9000) as *mut uint16 as *mut uint8)
            .offset(1) as *mut uint16) = v4 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9004) as *mut uint16) =
            -(28288 as ::core::ffi::c_int) as uint16;
        v5 = 12620032 as ::core::ffi::c_int as ::core::ffi::c_uint;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9008) as *mut uint16 as *mut uint8)
            .offset(1) as *mut uint16) = *(&raw mut v5 as *mut uint16).offset(
            (::core::mem::size_of::<::core::ffi::c_uint>())
                .wrapping_div(::core::mem::size_of::<uint16>())
                .wrapping_sub(1_usize) as isize,
        );
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9006) as *mut uint16 as *mut uint8)
            .offset(1) as *mut uint16) = v5 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x900a) as *mut uint16) =
            addr_loc_889180 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x900c) as *mut uint16) = 0 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9000) as *mut uint16) = 160 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9006) as *mut uint16) = 160 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x900c) as *mut uint16) = 160 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x900f) as *mut uint16) = 160 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9012) as *mut uint16) = 160 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9015) as *mut uint16) = 160 as uint16;
        v2 = 10522880 as ::core::ffi::c_int as ::core::ffi::c_uint;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9002) as *mut uint16 as *mut uint8)
            .offset(1) as *mut uint16) = *(&raw mut v2 as *mut uint16).offset(
            (::core::mem::size_of::<::core::ffi::c_uint>())
                .wrapping_div(::core::mem::size_of::<uint16>())
                .wrapping_sub(1_usize) as isize,
        );
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9000) as *mut uint16 as *mut uint8)
            .offset(1) as *mut uint16) = v2 as uint16;
        v3 = 10522880 as ::core::ffi::c_int as ::core::ffi::c_uint;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9008) as *mut uint16 as *mut uint8)
            .offset(1) as *mut uint16) = *(&raw mut v3 as *mut uint16).offset(
            (::core::mem::size_of::<::core::ffi::c_uint>())
                .wrapping_div(::core::mem::size_of::<uint16>())
                .wrapping_sub(1_usize) as isize,
        );
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9006) as *mut uint16 as *mut uint8)
            .offset(1) as *mut uint16) = v3 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x900c) as *mut uint16 as *mut uint8)
            .offset(1) as *mut uint16) = -(28416 as ::core::ffi::c_int) as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9012) as *mut uint16 as *mut uint8)
            .offset(1) as *mut uint16) = -(28416 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9004) as *mut uint16) =
            -(28352 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x900a) as *mut uint16) =
            -(28352 as ::core::ffi::c_int) as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x900f) as *mut uint16 as *mut uint8)
            .offset(1) as *mut uint16) = -(28352 as ::core::ffi::c_int) as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9015) as *mut uint16 as *mut uint8)
            .offset(1) as *mut uint16) = -(28352 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9018) as *mut uint16) = 0 as uint16;
    }
    let v6: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(v6 as isize) =
        -(2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16).offset(v6 as isize) =
        1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x192c) as *mut uint16).offset(v6 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1938) as *mut uint16).offset(v6 as isize) =
        0 as uint16;
    hdp
}
pub unsafe fn HdmaobjPreInstr_E567(v0: uint16, game: &mut Game) {
    let mut v7: int16 = 0;
    let mut j: uint16 = 0;
    let mut v8: uint16 = 0;
    let mut r28: uint16 = 0;
    let mut r30: uint16 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1))
        .parameter_1
        != 0
    {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1))
            .parameter_1 as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int)
            != 0i32
        {
            r28 = 4 as uint16;
            r30 = 128 as uint16;
        } else {
            r28 = 8 as uint16;
            r30 = 64 as uint16;
        }
        let v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh79 = *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16)
            .offset(v2 as isize);
        let fresh80 = *fresh79;
        *fresh79 = (*fresh79).wrapping_sub(1);
        if fresh80 == 1u16 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16)
                .offset(v2 as isize) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
                .offset(v2 as isize) = (2 as ::core::ffi::c_int
                * (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(3))
                    .ai_preinstr as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int
                & 0x1ff as ::core::ffi::c_int) as uint16;
        }
        let mut r20: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
            .offset(v2 as isize);
        let mut v4: uint16 = 0 as uint16;
        loop {
            let v11: uint16 = v4;
            let v5: uint16 = r20;
            let v6: ::core::ffi::c_int = r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            v7 = kSinCosTable8bit_Sext[(v6 + 64 as ::core::ffi::c_int) as usize];
            if v7 < 0i16 {
                let mut r18: uint16 = -(v7 as ::core::ffi::c_int) as uint16;
                let mut r22: uint16 = (Mult8x8(
                    -(v7 as int8 as ::core::ffi::c_int) as uint8,
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
                        .offset(3))
                    .ai_var_D as uint8,
                ) as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int) as uint16;
                r22 = (r22).wrapping_add(Mult8x8(
                    *(&raw mut r18 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
                        .offset(3))
                    .ai_var_D as uint8,
                ));
                let r24: uint16 = Mult8x8(
                    r18 as uint8,
                    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0xf78)
                        as *mut EnemyData)
                        .offset(3))
                    .ai_var_D as *mut uint8)
                        .offset(
                            (::core::mem::size_of::<uint16>())
                                .wrapping_div(::core::mem::size_of::<uint8>())
                                .wrapping_sub(1_usize) as isize,
                        ),
                );
                let mult: uint8 = Mult8x8(
                    *(&raw mut r18 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0xf78)
                        as *mut EnemyData)
                        .offset(3))
                    .ai_var_D as *mut uint8)
                        .offset(
                            (::core::mem::size_of::<uint16>())
                                .wrapping_div(::core::mem::size_of::<uint8>())
                                .wrapping_sub(1_usize) as isize,
                        ),
                ) as uint8;
                r22 = (r22).wrapping_add(r24);
                r18 = ((r22 as ::core::ffi::c_int
                    + ((mult as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                    & 0xff00 as ::core::ffi::c_int)
                    >> 8 as ::core::ffi::c_int) as uint16;
                r20 = (r28 as ::core::ffi::c_int + v5 as ::core::ffi::c_int
                    & 0x1ff as ::core::ffi::c_int) as uint16;
                v8 = v11;
                *(((&raw mut game.g_ram as *mut uint8).offset(0x9100) as *mut uint16 as *mut uint8)
                    .offset(v11 as ::core::ffi::c_int as isize) as *mut uint16) =
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16))
                        .wrapping_sub(r18);
            } else {
                let mut r18_0: uint16 =
                    kSinCosTable8bit_Sext[(v6 + 64 as ::core::ffi::c_int) as usize] as uint16;
                let mut r22_0: uint16 = (Mult8x8(
                    v7 as uint8,
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
                        .offset(3))
                    .ai_var_D as uint8,
                ) as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int) as uint16;
                r22_0 = (r22_0).wrapping_add(Mult8x8(
                    *(&raw mut r18_0 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
                        .offset(3))
                    .ai_var_D as uint8,
                ));
                let r24_0: uint16 = Mult8x8(
                    r18_0 as uint8,
                    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0xf78)
                        as *mut EnemyData)
                        .offset(3))
                    .ai_var_D as *mut uint8)
                        .offset(
                            (::core::mem::size_of::<uint16>())
                                .wrapping_div(::core::mem::size_of::<uint8>())
                                .wrapping_sub(1_usize) as isize,
                        ),
                );
                let mult_0: uint8 = Mult8x8(
                    *(&raw mut r18_0 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0xf78)
                        as *mut EnemyData)
                        .offset(3))
                    .ai_var_D as *mut uint8)
                        .offset(
                            (::core::mem::size_of::<uint16>())
                                .wrapping_div(::core::mem::size_of::<uint8>())
                                .wrapping_sub(1_usize) as isize,
                        ),
                ) as uint8;
                r22_0 = (r22_0).wrapping_add(r24_0);
                r18_0 = ((r22_0 as ::core::ffi::c_int
                    + ((mult_0 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                    & 0xff00 as ::core::ffi::c_int)
                    >> 8 as ::core::ffi::c_int) as uint16;
                r20 = (r28 as ::core::ffi::c_int + v5 as ::core::ffi::c_int
                    & 0x1ff as ::core::ffi::c_int) as uint16;
                v8 = v11;
                *(((&raw mut game.g_ram as *mut uint8).offset(0x9100) as *mut uint16 as *mut uint8)
                    .offset(v11 as ::core::ffi::c_int as isize) as *mut uint16) = (r18_0)
                    .wrapping_add(
                        *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16),
                    );
            }
            v4 = (v8).wrapping_add(2u16);
            if !(((v4) as i16).wrapping_sub((r30) as i16) < 0i16) {
                break;
            }
        }
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1))
            .parameter_1 as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int)
            != 0i32
        {
            let mut i: ::core::ffi::c_int = 126 as ::core::ffi::c_int;
            while i >= 0i32 {
                *(((&raw mut game.g_ram as *mut uint8).offset(0x9180) as *mut uint16 as *mut uint8)
                    .offset(i as isize) as *mut uint16) =
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16)
                        as ::core::ffi::c_int
                        + *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16)
                            as ::core::ffi::c_int
                        - *(((&raw mut game.g_ram as *mut uint8).offset(0x9100) as *mut uint16
                            as *mut uint8)
                            .offset(i as isize) as *mut uint16)
                            as ::core::ffi::c_int) as uint16;
                i -= 2 as ::core::ffi::c_int;
            }
        } else {
            j = 62 as uint16;
            while (j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
                *(((&raw mut game.g_ram as *mut uint8).offset(0x9140) as *mut uint16 as *mut uint8)
                    .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16)
                        as ::core::ffi::c_int
                        + *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16)
                            as ::core::ffi::c_int
                        - *(((&raw mut game.g_ram as *mut uint8).offset(0x9100) as *mut uint16
                            as *mut uint8)
                            .offset(j as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int) as uint16;
                j = (j).wrapping_sub(2u16);
            }
        }
    } else {
        let v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x18b2)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh81 = *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16)
            .offset(v1 as isize);
        *fresh81 = (*fresh81).wrapping_add(2u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    };
}
static unk_88E722: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0 as uint8,
    field_1: 0x2c as uint8,
    addr: 0xe727 as VoidP,
};
static unk_88E74C: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
    field_0: 0x41 as uint8,
    field_1: 0x26 as uint8,
    addr: 0xe751 as VoidP,
};
pub unsafe fn MotherBrainRisingHdmaObject(game: &mut Game) -> uint16 {
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88E722 as *const ::core::ffi::c_void,
        game,
    )
}
pub unsafe fn SpawnMotherBrainRainbowBeamHdma(game: &mut Game) -> uint16 {
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88E74C as *const ::core::ffi::c_void,
        game,
    )
}
pub unsafe fn InitializeRainbowBeam(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) = 32 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) = 71 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) = 0x8f as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) = 36 as uint16;
    *(&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(0) = 100 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(1) as *mut uint8 as *mut uint16) =
        -(25344 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(3) as *mut uint8 as *mut uint16) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(0) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(1) =
        -(32736 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(2) = 0 as uint16;
    MotherBrain_CalculateRainbowBeamHdma(game);
}
pub unsafe fn HdmaobjPreInstr_E7BC(k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) == 19u16 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x18b4) as *mut uint16).offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x18b2) as *mut uint16)
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) = 0 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1986) as *mut uint16) = 36 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) != 27u16 {
            MotherBrain_CalculateRainbowBeamHdma(game);
            sub_88E7ED(game);
        }
    };
}
pub unsafe fn sub_88E7ED(game: &mut Game) {
    let mut v0: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x88e833)) as *const uint8
        as *mut uint16)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(0)
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    if (v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        let ref mut fresh75 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(0);
        *fresh75 = (*fresh75).wrapping_add(1);
        let ref mut fresh76 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(0);
        *fresh76 = (*fresh76).wrapping_add(1);
        let ref mut fresh77 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(0);
        *fresh77 = (*fresh77).wrapping_add(1);
        let ref mut fresh78 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(0);
        *fresh78 = (*fresh78).wrapping_add(1);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(0) =
            0 as uint16;
        v0 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x88e833)) as *const uint8
            as *mut uint16)
            .offset(0);
    }
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(0) =
        (v0 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int)
            as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(1) =
        (v0 as ::core::ffi::c_int >> 5 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int
            | 0x40 as ::core::ffi::c_int) as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x74).offset(2) =
        ((v0 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int
            | 0x80 as ::core::ffi::c_int) as uint8;
}
pub unsafe fn SpawnMorphBallEyeBeamHdma(game: &mut Game) {
    static unk_88E8E4: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x41 as uint8,
        field_1: 0x26 as uint8,
        addr: 0xe8ec as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88E8E4 as *const ::core::ffi::c_void,
        game,
    );
}
pub fn HdmaobjInstr_InitMorphBallEyeBeamHdma(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    let color_slot = usize::from(k);
    game.g_ram[0x1914 + color_slot] = 0x30;
    game.g_ram[0x74] = 0x30;
    game.g_ram[0x1915 + color_slot] = 0x50;
    game.g_ram[0x75] = 0x50;
    game.g_ram[0x1920 + color_slot] = 0x80;
    game.g_ram[0x76] = 0x80;
    game.g_ram[0x1986..0x1988].copy_from_slice(&16u16.to_le_bytes());
    // The original stores also clear the byte after the HDMA terminator.
    game.g_ram[0x9000..0x900b].copy_from_slice(&[
        0xe4, 0x00, 0x91, 0xe4, 0xc8, 0x91, 0x98, 0x90, 0x92, 0x00, 0x00,
    ]);
    let active_offset = 0xf78 + 0x40 + core::mem::offset_of!(EnemyData, ai_var_C);
    game.g_ram[active_offset..active_offset + 2].copy_from_slice(&1u16.to_le_bytes());
    let slot = usize::from(k & !1);
    let width_offset = 0x192c + slot;
    game.g_ram[width_offset..width_offset + 2].copy_from_slice(&0u16.to_le_bytes());
    let width_fraction_offset = 0x1938 + slot;
    game.g_ram[width_fraction_offset..width_fraction_offset + 2]
        .copy_from_slice(&0u16.to_le_bytes());
    game.g_ram[0x9080..0x9084].fill(0);
    game.g_ram[0x9090..0x9092].fill(0);
    hdp
}
pub unsafe fn UpdateMorphBallEyeBeamHdma(k: u16, game: &mut Game) {
    // The beam's origin and angle belong to enemy slot 1.
    const BEAM_ENEMY: u16 = 0x40;
    let angle_offset = 0xf78 + usize::from(BEAM_ENEMY) + core::mem::offset_of!(EnemyData, ai_var_D);
    let angle = u16::from_le_bytes([game.g_ram[angle_offset], game.g_ram[angle_offset + 1]]);
    // HDMA object C uses a word index; the colors below keep the byte index.
    let width_offset = 0x192c + usize::from(k & !1);
    let width = u16::from_le_bytes([game.g_ram[width_offset], game.g_ram[width_offset + 1]]);
    let relative_y = game
        .enemy_coordinates(BEAM_ENEMY)
        .y_pos()
        .wrapping_sub(game.layer1_y_pos());
    let relative_x = game
        .enemy_coordinates(BEAM_ENEMY)
        .x_pos()
        .wrapping_sub(game.layer1_x_pos());
    CalculateXrayHdmaTableInner(
        relative_x,
        relative_y,
        angle,
        width,
        relative_x >= 256,
        game.g_ram.as_mut_ptr().add(0x9100).cast(),
        game,
    );
    // Color sources can overlap the table, so read them after it is updated.
    let color_offset = usize::from(k);
    game.g_ram[0x74] = game.g_ram[0x1914 + color_offset];
    game.g_ram[0x75] = game.g_ram[0x1915 + color_offset];
    game.g_ram[0x76] = game.g_ram[0x1920 + color_offset];
}
pub unsafe fn HdmaobjPreInstr_MorphBallEyeBeamWidening(k: u16, game: &mut Game) {
    fn add_fixed_point(ram: &mut [u8], whole: usize, fraction: usize, increment: u32) -> u32 {
        let value =
            u32::from_le_bytes([ram[fraction], ram[fraction + 1], ram[whole], ram[whole + 1]])
                .wrapping_add(increment);
        let bytes = value.to_le_bytes();
        ram[fraction..fraction + 2].copy_from_slice(&bytes[..2]);
        ram[whole..whole + 2].copy_from_slice(&bytes[2..]);
        value
    }

    // Select the eye-beam layer blending configuration.
    game.g_ram[0x1986..0x1988].copy_from_slice(&16u16.to_le_bytes());
    let widening_speed = add_fixed_point(&mut game.g_ram, 0x9080, 0x9082, 0x4000);
    let slot = usize::from(k & !1);
    let width_offset = 0x192c + slot;
    let fraction_offset = 0x1938 + slot;
    let width = add_fixed_point(
        &mut game.g_ram,
        width_offset,
        fraction_offset,
        widening_speed,
    );
    let whole_width = (width >> 16) as u16;
    if (whole_width.wrapping_sub(4) as i16) >= 0 {
        // Keep the fractional width when the integer part reaches the limit.
        game.g_ram[width_offset..width_offset + 2].copy_from_slice(&4u16.to_le_bytes());
        let current_slot = u16::from_le_bytes([game.g_ram[0x18b2], game.g_ram[0x18b3]]);
        let current_offset = usize::from(current_slot & !1);
        let instruction_offset = 0x18cc + current_offset;
        let next_instruction = u16::from_le_bytes([
            game.g_ram[instruction_offset],
            game.g_ram[instruction_offset + 1],
        ])
        .wrapping_add(2);
        game.g_ram[instruction_offset..instruction_offset + 2]
            .copy_from_slice(&next_instruction.to_le_bytes());
        let timer_offset = 0x18e4 + current_offset;
        game.g_ram[timer_offset..timer_offset + 2].copy_from_slice(&1u16.to_le_bytes());
    }
    UpdateMorphBallEyeBeamHdma(k, game);
}
pub unsafe fn HdmaobjPreInstr_MorphBallEyeBeamFull(k: u16, game: &mut Game) {
    game.g_ram[0x1986..0x1988].copy_from_slice(&16u16.to_le_bytes());
    // Enemy slot 1 signals whether the eye is still active.
    let active_offset = 0xf78 + 0x40 + core::mem::offset_of!(EnemyData, ai_var_C);
    let active = u16::from_le_bytes([game.g_ram[active_offset], game.g_ram[active_offset + 1]]);
    if active != 0 {
        UpdateMorphBallEyeBeamHdma(k, game);
        let color_index = u16::from_le_bytes([game.g_ram[0x9090], game.g_ram[0x9091]]);
        let color_offset = usize::from(color_index.wrapping_mul(4));
        let slot = usize::from(k);
        game.g_ram[0x1914 + slot] = game.read_rom_byte(0x88ea8b, color_offset);
        game.g_ram[0x1915 + slot] = game.read_rom_byte(0x88ea8b, color_offset + 1);
        game.g_ram[0x1920 + slot] = game.read_rom_byte(0x88ea8b, color_offset + 2);
        // Color destinations can overlap the index, so advance its live value.
        let next_color_index =
            u16::from_le_bytes([game.g_ram[0x9090], game.g_ram[0x9091]]).wrapping_add(1) & 0xf;
        game.g_ram[0x9090..0x9092].copy_from_slice(&next_color_index.to_le_bytes());
    } else {
        let current_slot = u16::from_le_bytes([game.g_ram[0x18b2], game.g_ram[0x18b3]]);
        let current_offset = usize::from(current_slot & !1);
        let instruction_offset = 0x18cc + current_offset;
        let next_instruction = u16::from_le_bytes([
            game.g_ram[instruction_offset],
            game.g_ram[instruction_offset + 1],
        ])
        .wrapping_add(2);
        game.g_ram[instruction_offset..instruction_offset + 2]
            .copy_from_slice(&next_instruction.to_le_bytes());
        let timer_offset = 0x18e4 + current_offset;
        game.g_ram[timer_offset..timer_offset + 2].copy_from_slice(&1u16.to_le_bytes());
    }
}
pub unsafe fn HdmaobjPreInstr_MorphBallEyeBeamFading(k: u16, game: &mut Game) {
    game.g_ram[0x1986..0x1988].copy_from_slice(&16u16.to_le_bytes());
    if game.g_ram[0x1915 + usize::from(k)] == 0x40 {
        game.g_ram[0x74..0x77].copy_from_slice(&[0x20, 0x40, 0x80]);
        let slot = usize::from(k & !1);
        let width_fraction_offset = 0x1938 + slot;
        game.g_ram[width_fraction_offset..width_fraction_offset + 2]
            .copy_from_slice(&0u16.to_le_bytes());
        let width_offset = 0x192c + slot;
        game.g_ram[width_offset..width_offset + 2].copy_from_slice(&0u16.to_le_bytes());
        for entry in game.g_ram[0x9100..0x9300].chunks_exact_mut(2).rev() {
            entry.copy_from_slice(&0xffu16.to_le_bytes());
        }
        game.g_ram[0x9000..0x900a].fill(0);

        let current_slot = u16::from_le_bytes([game.g_ram[0x18b2], game.g_ram[0x18b3]]);
        let current_offset = usize::from(current_slot & !1);
        let instruction_offset = 0x18cc + current_offset;
        let next_instruction = u16::from_le_bytes([
            game.g_ram[instruction_offset],
            game.g_ram[instruction_offset + 1],
        ])
        .wrapping_add(2);
        game.g_ram[instruction_offset..instruction_offset + 2]
            .copy_from_slice(&next_instruction.to_le_bytes());
        let timer_offset = 0x18e4 + current_offset;
        game.g_ram[timer_offset..timer_offset + 2].copy_from_slice(&1u16.to_le_bytes());
    } else {
        UpdateMorphBallEyeBeamHdma(k, game);
        let current_slot = u16::from_le_bytes([game.g_ram[0x18b2], game.g_ram[0x18b3]]);
        // Colors use byte slots, including odd current object indices.
        for (color_offset, black) in [(0x1914, 0x20), (0x1915, 0x40), (0x1920, 0x80)] {
            let offset = color_offset + usize::from(current_slot);
            if game.g_ram[offset] != black {
                game.g_ram[offset] = game.g_ram[offset].wrapping_sub(1);
            }
        }
    }
}
pub unsafe fn SpawnTitleScreenGradientObjs(game: &mut Game) {
    static unk_88EB6B: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0 as uint8,
        field_1: 0x31 as uint8,
        addr: 0xeb85 as VoidP,
    };
    static unk_88EB63: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0 as uint8,
        field_1: 0x32 as uint8,
        addr: 0xeb73 as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88EB63 as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88EB6B as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn HdmaobjInsr_ConfigTitleSequenceGradientHDMA(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *(&raw mut game.g_ram as *mut uint8).offset(0x6f) = 0 as uint8;
    ConfigureTitleSequenceGradientHDMA(game);
    hdp
}
pub unsafe fn HdmaobjPreInstr_Backdrop_TitleSequenceGradient(k: uint16, game: &mut Game) {
    ConfigureTitleSequenceGradientHDMA(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
        == 0x8b9b68 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        let v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x18b2)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh68 = *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16)
            .offset(v1 as isize);
        *fresh68 = (*fresh68).wrapping_add(2u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    }
}
pub unsafe fn HdmaobjPreInstr_ColorMathControlB_TitleGradient(k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
        == 0x8b9b68 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        let v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x18b2)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh67 = *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16)
            .offset(v1 as isize);
        *fresh67 = (*fresh67).wrapping_add(2u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    }
}
pub unsafe fn SpawnIntroCutsceneCrossfadeHdma(game: &mut Game) {
    static unk_88EBFB: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0 as uint8,
        field_1: 0x31 as uint8,
        addr: 0xec03 as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88EBFB as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn HdmaobjPreInstr_IntroCutsceneCrossfade(k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(0) == 1u16 {
        let v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x18b2)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh66 = *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16)
            .offset(v1 as isize);
        *fresh66 = (*fresh66).wrapping_add(2u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    }
}
pub unsafe fn CinematicFunction_Intro_Func133(game: &mut Game) {
    let mut v0: ::core::ffi::c_uint = 0;
    let mut v1: ::core::ffi::c_uint = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd9c) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd9e) as *mut uint16) = 0x4000 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xda0) as *mut uint16) = 8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd82) as *mut uint16) = 192 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd88) as *mut uint16) = 192 as uint16;
    v0 = 12621824 as ::core::ffi::c_int as ::core::ffi::c_uint;
    *(((&raw mut game.g_ram as *mut uint8).offset(0xd84) as *mut uint16 as *mut uint8).offset(1)
        as *mut uint16) = *(&raw mut v0 as *mut uint16).offset(
        (::core::mem::size_of::<::core::ffi::c_uint>())
            .wrapping_div(::core::mem::size_of::<uint16>())
            .wrapping_sub(1_usize) as isize,
    );
    *(((&raw mut game.g_ram as *mut uint8).offset(0xd82) as *mut uint16 as *mut uint8).offset(1)
        as *mut uint16) = v0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd86) as *mut uint16) =
        -(26496 as ::core::ffi::c_int) as uint16;
    v1 = 12621824 as ::core::ffi::c_int as ::core::ffi::c_uint;
    *(((&raw mut game.g_ram as *mut uint8).offset(0xd8a) as *mut uint16 as *mut uint8).offset(1)
        as *mut uint16) = *(&raw mut v1 as *mut uint16).offset(
        (::core::mem::size_of::<::core::ffi::c_uint>())
            .wrapping_div(::core::mem::size_of::<uint16>())
            .wrapping_sub(1_usize) as isize,
    );
    *(((&raw mut game.g_ram as *mut uint8).offset(0xd88) as *mut uint16 as *mut uint8).offset(1)
        as *mut uint16) = v1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd8c) as *mut uint16) =
        -(26496 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd8e) as *mut uint16) = 0 as uint16;
    static unk_88EC82: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x42 as uint8,
        field_1: 0x11 as uint8,
        addr: 0xec8a as VoidP,
    };
    SpawnHdmaObject(
        0x88 as uint8,
        &raw const unk_88EC82 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn HdmaobjInstr_EC9F_ClearVars(
    k: uint16,
    hdp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(v2 as isize) =
        -(2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1920) as *mut uint16).offset(v2 as isize) =
        1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x192c) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1938) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    hdp
}
pub unsafe fn HdmaobjPreInstr_ECB6(k: uint16, game: &mut Game) {
    let mut v7: int16 = 0;
    let mut v8: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xd9c) as *mut uint16) != 0 {
        let r28: uint16 = 4 as uint16;
        let n: ::core::ffi::c_int = 128 as ::core::ffi::c_int;
        let v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16).offset(v3 as isize) =
            (2 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0xda0) as *mut uint16)
                    as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                & 0x1ff as ::core::ffi::c_int) as uint16;
        let mut r20: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1914) as *mut uint16)
            .offset(v3 as isize);
        let mut v4: uint16 = 0 as uint16;
        loop {
            let v10: uint16 = v4;
            let v5: uint16 = r20;
            let v6: ::core::ffi::c_int = r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            v7 = kSinCosTable8bit_Sext[(v6 + 64 as ::core::ffi::c_int) as usize];
            if v7 < 0i16 {
                let mut r18: uint16 = -(v7 as ::core::ffi::c_int) as uint16;
                let mut r22: uint16 = (Mult8x8(
                    -(v7 as int8 as ::core::ffi::c_int) as uint8,
                    *(&raw mut game.g_ram as *mut uint8).offset(0xd9e),
                ) as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int) as uint16;
                r22 = (r22).wrapping_add(Mult8x8(
                    *(&raw mut r18 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                    *(&raw mut game.g_ram as *mut uint8).offset(0xd9e),
                ));
                let r24: uint16 = Mult8x8(
                    r18 as uint8,
                    *(&raw mut game.g_ram as *mut uint8).offset(0xd9f),
                );
                let mult: uint8 = Mult8x8(
                    *(&raw mut r18 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                    *(&raw mut game.g_ram as *mut uint8).offset(0xd9f),
                ) as uint8;
                r22 = (r22).wrapping_add(r24);
                r18 = ((r22 as ::core::ffi::c_int
                    + ((mult as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                    & 0xff00 as ::core::ffi::c_int)
                    >> 8 as ::core::ffi::c_int) as uint16;
                r20 = (r28 as ::core::ffi::c_int + v5 as ::core::ffi::c_int
                    & 0x1ff as ::core::ffi::c_int) as uint16;
                v8 = v10;
                *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16)
                    .offset((v10 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xb9) as *mut uint16))
                        .wrapping_sub(r18);
            } else {
                let mut r18_0: uint16 =
                    kSinCosTable8bit_Sext[(v6 + 64 as ::core::ffi::c_int) as usize] as uint16;
                let mut r22_0: uint16 = (Mult8x8(
                    v7 as uint8,
                    *(&raw mut game.g_ram as *mut uint8).offset(0xd9e),
                ) as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int) as uint16;
                r22_0 = (r22_0).wrapping_add(Mult8x8(
                    *(&raw mut r18_0 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                    *(&raw mut game.g_ram as *mut uint8).offset(0xd9e),
                ));
                let r24_0: uint16 = Mult8x8(
                    r18_0 as uint8,
                    *(&raw mut game.g_ram as *mut uint8).offset(0xd9f),
                );
                let mult_0: uint8 = Mult8x8(
                    *(&raw mut r18_0 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                    *(&raw mut game.g_ram as *mut uint8).offset(0xd9f),
                ) as uint8;
                r22_0 = (r22_0).wrapping_add(r24_0);
                r18_0 = ((r22_0 as ::core::ffi::c_int
                    + ((mult_0 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                    & 0xff00 as ::core::ffi::c_int)
                    >> 8 as ::core::ffi::c_int) as uint16;
                r20 = (r28 as ::core::ffi::c_int + v5 as ::core::ffi::c_int
                    & 0x1ff as ::core::ffi::c_int) as uint16;
                v8 = v10;
                *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16)
                    .offset((v10 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                    (r18_0).wrapping_add(
                        *((&raw mut game.g_ram as *mut uint8).offset(0xb9) as *mut uint16),
                    );
            }
            v4 = (v8).wrapping_add(2u16);
            if !(((v4 as ::core::ffi::c_int - n) as int16) < 0i16) {
                break;
            }
        }
        let mut i: ::core::ffi::c_int = 126 as ::core::ffi::c_int;
        while i >= 0i32 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 64 as ::core::ffi::c_int) as isize) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xb9) as *mut uint16)
                    as ::core::ffi::c_int
                    + *((&raw mut game.g_ram as *mut uint8).offset(0xb9) as *mut uint16)
                        as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x9800) as *mut uint16)
                        .offset((i >> 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int) as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
    } else {
        let v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x18b2)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh65 = *((&raw mut game.g_ram as *mut uint8).offset(0x18cc) as *mut uint16)
            .offset(v2 as isize);
        *fresh65 = (*fresh65).wrapping_add(2u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x18e4) as *mut uint16).offset(v2 as isize) =
            1 as uint16;
    };
}
pub unsafe fn CallHdmaobjPreInstr(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        8491739 => (),
        8946872 => (),
        10785852 => (),
        9556607 => {
            HdmaobjPreInstr_XraySetup(k, game);
        }
        8947439 => {
            HdmaobjPreInstr_Xray(k, game);
        }
        8947506 => {
            HdmaobjPreInstr_XrayFunc0_NoBeam(k, game);
        }
        8947540 => {
            HdmaobjPreInstr_XrayFunc1_BeamWidening(k, game);
        }
        8947627 => {
            HdmaobjPreInstr_XrayFunc2_FullBeam(k, game);
        }
        8948020 => {
            HdmaobjPreInstr_XrayFunc3_DeactivateBeam(k, game);
        }
        8948154 => {
            HdmaobjPreInstr_XrayFunc4_DeactivateBeam(k, game);
        }
        8948232 => {
            HdmaobjPreInstr_XrayFunc5_DeactivateBeam(k, game);
        }
        8948623 => {
            HdmaobjPreInstr_PowerBombExplode_SetWindowConf(k, game);
        }
        8948632 => {
            HdmaobjPreInstr_PowerBombExplode_Stage5_Afterglow(k, game);
        }
        8949225 => {
            HdmaobjPreInstr_PowerBombExplode_ExplosionYellow(k, game);
        }
        8949426 => {
            HdmaobjPreInstr_PowerBombExplode_ExplosionWhite(k, game);
        }
        8949983 => {
            HdmaobjPreInstr_PowerBombExplode_PreExplosionWhite(k, game);
        }
        8950184 => {
            HdmaobjPreInstr_PowerBombExplode_PreExplosionYellow(k, game);
        }
        8954681 => {
            HdmaobjPreInstr_CrystalFlash_CustomLayerBlend(k, game);
        }
        8954717 => {
            HdmaobjPreInstr_CrystalFlash_Stage2_AfterGlow(k, game);
        }
        8955218 => {
            HdmaobjPreInstr_CrystalFlash_Stage1_Explosion(k, game);
        }
        8955459 => {
            HdmaobjPreInstr_FxType22_BG3Yscroll(k, game);
        }
        8955507 => {
            HdmaobjPreInstr_BG3Xscroll(k, game);
        }
        8957362 => {
            HdmaobjPreInstr_SkyLandBG2Xscroll(k, game);
        }
        8957370 => {
            HdmaobjPreInstr_SkyLandBG2Xscroll2(k, game);
        }
        8957378 => {
            HdmaobjPreInstr_SkyLandBG2XscrollInner(k, game);
        }
        8958140 => {
            HdmaobjPreInstr_FirefleaBG3XScroll(k, game);
        }
        8958896 => {
            HdmaobjPreInstr_LavaAcidBG3YScroll(k, game);
        }
        8959189 => {
            HdmaobjPreInstr_LavaAcidBG2YScroll(k, game);
        }
        8963214 => {
            HdmaobjPreInstr_WaterBG3XScroll(k, game);
        }
        8963465 => {
            HdmaobjPreInstr_WaterBG2XScroll(k, game);
        }
        8963556 => {
            HdmaobjPreInstr_WaterBG2XScroll_Func2(k, game);
        }
        8963638 => {
            HdmaobjPreInstr_WaterBG2XScroll_Func1(k, game);
        }
        8968609 => {
            HdmaobjPreInstr_RainBg3Scroll(k, game);
        }
        8968775 => {
            HdmaobjPreInstr_SporesBG3Xscroll(k, game);
        }
        8969014 => {
            HdmaobjPreInstr_FogBG3Scroll(k, game);
        }
        8969175 => {
            HdmaobjPreInstr_CheckLotsOfEventsHappened(k, game);
        }
        8969251 => {
            HdmaobjPreInstr_DC23(k, game);
        }
        8969321 => {
            HdmaobjPreInstr_DC69(k, game);
        }
        8969402 => {
            HdmaobjPreInstr_DCBA(k, game);
        }
        8969539 => {
            HdmaobjPreInstr_BombTorizoHazeColorMathBgColor(k, game);
        }
        8969744 => {
            HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyAlive(k, game);
        }
        8969749 => {
            HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyDead(k, game);
        }
        8969773 => {
            HdmaobjPreInstr_HazeColorMathSubscreen_FadingIn(k, game);
        }
        8969844 => {
            HdmaobjPreInstr_HazeColorMathSubscreen_FadedIn(k, game);
        }
        8969878 => {
            HdmaobjPreInstr_HazeColorMathSubscreen_FadingOut(k, game);
        }
        8970132 => {
            HdmaobjPreInstr_DF94(k, game);
        }
        8970278 => {
            HdmaobjPreInstr_VariaSuitPickup(k, game);
        }
        8970332 => {
            HdmaobjPreInstr_GravitySuitPickup(k, game);
        }
        8971337 => {
            HdmaobjPreInstr_E449(k, game);
        }
        8971623 => {
            HdmaobjPreInstr_E567(k, game);
        }
        8972220 => {
            HdmaobjPreInstr_E7BC(k, game);
        }
        fnHdmaobjPreInstr_MorphBallEyeBeamWidening => {
            HdmaobjPreInstr_MorphBallEyeBeamWidening(k, game);
        }
        fnHdmaobjPreInstr_MorphBallEyeBeamFull => {
            HdmaobjPreInstr_MorphBallEyeBeamFull(k, game);
        }
        fnHdmaobjPreInstr_MorphBallEyeBeamFading => {
            HdmaobjPreInstr_MorphBallEyeBeamFading(k, game);
        }
        8973232 => {
            HdmaobjPreInstr_Backdrop_TitleSequenceGradient(k, game);
        }
        8973266 => {
            HdmaobjPreInstr_ColorMathControlB_TitleGradient(k, game);
        }
        8973341 => {
            HdmaobjPreInstr_IntroCutsceneCrossfade(k, game);
        }
        8972070 => (),
        8973494 => {
            HdmaobjPreInstr_ECB6(k, game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CallHdmaobjInstr(
    ea: uint32,
    k: uint16,
    j: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    match ea {
        8969006 => j,
        8947049 => HdmaobjInstr_Delete(k, j, game),
        8947056 => HdmaobjInstr_SetPreInstr(k, j, game),
        8947076 => HdmaobjInstr_ClearPreInstr(k, j, game),
        8947124 => HdmaobjInstr_CallFarFunc(k, j, game),
        8947180 => HdmaobjInstr_Goto(k, j),
        8947185 => HdmaobjInstr_GotoRel(k, j),
        8947207 => HdmaobjInstr_DecrementAndGoto(k, j, game),
        8947215 => HdmaobjInstr_DecrementAndGotoRel(k, j, game),
        8947222 => HdmaobjInstr_SetTimer(k, j, game),
        8947234 => HdmaobjInstr_SetHdmaControl(k, j, game),
        8947255 => HdmaobjInstr_SetHdmaTarget(k, j, game),
        8947276 => HdmaobjInstr_SetHdmaTablePtr(k, j, game),
        8947285 => HdmaobjInstr_SetHdmaTableBank(k, j, game),
        8947306 => HdmaobjInstr_SetIndirectHdmaDataBank(k, j, game),
        8947330 => HdmaobjInstr_Sleep(k, j, game),
        8955500 => HdmaobjInstr_SetFlagB(k, j, game),
        8959182 => HdmaobjInstr_SetFlagB_Copy(k, j, game),
        8963175 => HdmaobjInstr_SetFlagB_Copy2(k, j, game),
        8963458 => HdmaobjInstr_SetFlagB_Copy3(k, j, game),
        8968521 => HdmaobjInstr_SetVideoMode1(k, j, game),
        8968577 => HdmaobjInstr_1938_RandomNumber(k, j, game),
        8969419 => HdmaobjInstr_GotoIfEventHappened(k, j, game),
        8971453 => HdmaobjInstr_E4BD(k, j, game),
        fnHdmaobjInstr_InitMorphBallEyeBeamHdma => {
            HdmaobjInstr_InitMorphBallEyeBeamHdma(k, j, game)
        }
        8973471 => HdmaobjInstr_EC9F_ClearVars(k, j, game),
        8958889 => HdmaobjInstr_B3A9(k, j, game),
        8973215 => HdmaobjInsr_ConfigTitleSequenceGradientHDMA(k, j, game),
        8968470 => {
            sub_88D916(game);
            j
        }
        _ => {
            Unreachable(game);
            ::core::ptr::null::<uint8>()
        }
    }
}
