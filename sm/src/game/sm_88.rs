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
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::{mov24, Load24, Mult8x8, RomPtr, Unreachable, WriteReg, WriteRegWord};
use crate::rtl::{AddToHiLo, SetHiLo};
pub use crate::snes::registers::*;

pub use crate::types::EnemyData;
pub use crate::types::{
    Func_U8, Func_V, Func_Y_V, Func_Y_Y, LongPtr, SnesRegs, VoidP, VramWriteEntry,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct HdmaScrollEntry {
    pub top_pos: u16,
    pub scroll_subspeed: u16,
    pub scroll_speed: u16,
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
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
#[inline]
unsafe fn RomPtr_88(addr: u16, game: &mut Game) -> *const u8 {
    RomPtr((0x880000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomBankBase(bank: u8, game: &mut Game) -> *const u8 {
    RomPtr((((bank as i32) << 16 as i32) + 0x8000 as i32) as u32, game)
        .offset(-(0x8000 as i32 as isize))
}
#[inline]
unsafe fn gVramWriteEntry(a: i32, game: &mut Game) -> *mut VramWriteEntry {
    (&raw mut game.g_ram as *mut u8).offset((0xd0 as i32 + a) as isize) as *mut u8
        as *mut VramWriteEntry
}
pub const fnnullsub_56: u32 = 8491739;
pub const fnDoorTransitionFunction_FadeOutScreen: i32 = 0x82e2db;
pub const fnDoorTransition_FadeInScreenAndFinish: i32 = 0x82e737;
pub const fnLayerBlendingHandler: i32 = 0x888000;
pub const fnPauseHook_Empty: i32 = 0x8883e1;
pub const fnnullsub_293: i32 = 0x8884b8;
pub const fnHdmaobjInstr_Delete: u32 = 8947049;
pub const fnHdmaobjInstr_SetPreInstr: u32 = 8947056;
pub const fnHdmaobjInstr_ClearPreInstr: u32 = 8947076;
pub const fnHdmaobjInstr_CallFarFunc: u32 = 8947124;
pub const fnHdmaobjInstr_Goto: u32 = 8947180;
pub const fnHdmaobjInstr_GotoRel: u32 = 8947185;
pub const fnHdmaobjInstr_DecrementAndGoto: u32 = 8947207;
pub const fnHdmaobjInstr_DecrementAndGotoRel: u32 = 8947215;
pub const fnHdmaobjInstr_SetTimer: u32 = 8947222;
pub const fnHdmaobjInstr_SetHdmaControl: u32 = 8947234;
pub const fnHdmaobjInstr_SetHdmaTarget: u32 = 8947255;
pub const fnHdmaobjInstr_SetHdmaTablePtr: u32 = 8947276;
pub const fnHdmaobjInstr_SetHdmaTableBank: u32 = 8947285;
pub const fnHdmaobjInstr_SetIndirectHdmaDataBank: u32 = 8947306;
pub const fnHdmaobjInstr_Sleep: u32 = 8947330;
pub const fnHdmaobjPreInstr_Xray: u32 = 8947439;
pub const fnHdmaobjPreInstr_XrayFunc0_NoBeam: u32 = 8947506;
pub const fnHdmaobjPreInstr_XrayFunc1_BeamWidening: u32 = 8947540;
pub const fnHdmaobjPreInstr_XrayFunc2_FullBeam: u32 = 8947627;
pub const fnHdmaobjPreInstr_XrayFunc3_DeactivateBeam: u32 = 8948020;
pub const fnHdmaobjPreInstr_XrayFunc4_DeactivateBeam: u32 = 8948154;
pub const fnHdmaobjPreInstr_XrayFunc5_DeactivateBeam: u32 = 8948232;
pub const fnHdmaobj_PreExplodeWhite: u32 = 8948500;
pub const fnHdmaobj_PreExplodeYellow: u32 = 8948530;
pub const fnHdmaobj_ExplodeYellow: u32 = 8948537;
pub const fnHdmaobj_ExplodeWhite: u32 = 8948551;
pub const fnHdmaobj_CleanUpTryCrystalFlash: u32 = 8948558;
pub const fnHdmaobjPreInstr_PowerBombExplode_SetWindowConf: u32 = 8948623;
pub const fnHdmaobjPreInstr_PowerBombExplode_Stage5_Afterglow: u32 = 8948632;
pub const fnHdmaobjPreInstr_PowerBombExplode_ExplosionYellow: u32 = 8949225;
pub const fnHdmaobjPreInstr_PowerBombExplode_ExplosionWhite: u32 = 8949426;
pub const fnHdmaobjPreInstr_PowerBombExplode_PreExplosionWhite: u32 = 8949983;
pub const fnHdmaobjPreInstr_PowerBombExplode_PreExplosionYellow: u32 = 8950184;
pub const fnCrystalFlashSetupPart1: u32 = 8954596;
pub const fnCrystalFlashSetupPart2: u32 = 8954633;
pub const fnCrystalFlashCleanup: u32 = 8954647;
pub const fnHdmaobjPreInstr_CrystalFlash_CustomLayerBlend: u32 = 8954681;
pub const fnHdmaobjPreInstr_CrystalFlash_Stage2_AfterGlow: u32 = 8954717;
pub const fnHdmaobjPreInstr_CrystalFlash_Stage1_Explosion: u32 = 8955218;
pub const fnHdmaobjPreInstr_FxType22_BG3Yscroll: u32 = 8955459;
pub const fnHdmaobjInstr_SetFlagB: u32 = 8955500;
pub const fnHdmaobjPreInstr_BG3Xscroll: u32 = 8955507;
pub const fnHdmaobjPreInstr_SkyLandBG2Xscroll: u32 = 8957362;
pub const fnHdmaobjPreInstr_SkyLandBG2Xscroll2: u32 = 8957370;
pub const fnHdmaobjPreInstr_SkyLandBG2XscrollInner: u32 = 8957378;
pub const fnHdmaobjPreInstr_FirefleaBG3XScroll: u32 = 8958140;
pub const fnFxRisingFunction_LavaAcid: i32 = 0x88b343;
pub const fnFxRisingFunction_LavaAcid_WaitToRise: i32 = 0x88b367;
pub const fnFxRisingFunction_LavaAcid_Raising: i32 = 0x88b382;
pub const fnHdmaobjInstr_B3A9: u32 = 8958889;
pub const fnHdmaobjPreInstr_LavaAcidBG3YScroll: u32 = 8958896;
pub const fnHdmaobjInstr_SetFlagB_Copy: u32 = 8959182;
pub const fnHdmaobjPreInstr_LavaAcidBG2YScroll: u32 = 8959189;
pub const fnFxRisingFunction_C428_WaterNormal: i32 = 0x88c428;
pub const fnFxRisingFunction_WaterWaitToRise: i32 = 0x88c44c;
pub const fnFxRisingFunction_WaterRising: i32 = 0x88c458;
pub const fnHdmaobjInstr_SetFlagB_Copy2: u32 = 8963175;
pub const fnHdmaobjPreInstr_WaterBG3XScroll: u32 = 8963214;
pub const fnHdmaobjInstr_SetFlagB_Copy3: u32 = 8963458;
pub const fnHdmaobjPreInstr_WaterBG2XScroll: u32 = 8963465;
pub const fnHdmaobjPreInstr_WaterBG2XScroll_Func2: u32 = 8963556;
pub const fnHdmaobjPreInstr_WaterBG2XScroll_Func1: u32 = 8963638;
pub const fnsub_88D916: u32 = 8968470;
pub const fnHdmaobjInstr_SetVideoMode1: u32 = 8968521;
pub const fnHdmaobjInstr_1938_RandomNumber: u32 = 8968577;
pub const fnHdmaobjPreInstr_RainBg3Scroll: u32 = 8968609;
pub const fnHdmaobjPreInstr_SporesBG3Xscroll: u32 = 8968775;
pub const fnnullsub_112: u32 = 8969006;
pub const fnHdmaobjPreInstr_FogBG3Scroll: u32 = 8969014;
pub const fnHdmaobjPreInstr_CheckLotsOfEventsHappened: u32 = 8969175;
pub const fnHdmaobjPreInstr_DC23: u32 = 8969251;
pub const fnHdmaobjPreInstr_DC69: u32 = 8969321;
pub const fnHdmaobjPreInstr_DCBA: u32 = 8969402;
pub const fnHdmaobjInstr_GotoIfEventHappened: u32 = 8969419;
pub const fnHdmaobjPreInstr_BombTorizoHazeColorMathBgColor: u32 = 8969539;
pub const fnHdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyAlive: u32 = 8969744;
pub const fnHdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyDead: u32 = 8969749;
pub const fnHdmaobjPreInstr_HazeColorMathSubscreen_FadingIn: i32 = 0x88de2d;
pub const fnHdmaobjPreInstr_HazeColorMathSubscreen_FadedIn: i32 = 0x88de74;
pub const fnHdmaobjPreInstr_HazeColorMathSubscreen_FadingOut: i32 = 0x88de96;
pub const fnnullsub_113: u32 = 8970129;
pub const fnnullsub_114: u32 = 8970130;
pub const fnHdmaobjPreInstr_DF94: u32 = 8970132;
pub const fnHdmaobjPreInstr_VariaSuitPickup: u32 = 8970278;
pub const fnHdmaobjPreInstr_GravitySuitPickup: u32 = 8970332;
pub const fnHdmaobjPreInstr_E449: u32 = 8971337;
pub const fnHdmaobjInstr_E4BD: u32 = 8971453;
pub const fnHdmaobjPreInstr_E567: u32 = 8971623;
pub const fnnullsub_357: u32 = 8972070;
pub const fnInitializeRainbowBeam: u32 = 8972135;
pub const fnHdmaobjPreInstr_E7BC: u32 = 8972220;
pub const fnHdmaobjInstr_InitMorphBallEyeBeamHdma: u32 = 0x88e917;
pub const fnHdmaobjPreInstr_MorphBallEyeBeamWidening: u32 = 0x88e9e6;
pub const fnHdmaobjPreInstr_MorphBallEyeBeamFull: u32 = 0x88ea3c;
pub const fnHdmaobjPreInstr_MorphBallEyeBeamFading: u32 = 0x88eacb;
pub const fnHdmaobjInsr_ConfigTitleSequenceGradientHDMA: u32 = 8973215;
pub const fnHdmaobjPreInstr_Backdrop_TitleSequenceGradient: u32 = 8973232;
pub const fnHdmaobjPreInstr_ColorMathControlB_TitleGradient: u32 = 8973266;
pub const fnHdmaobjPreInstr_IntroCutsceneCrossfade: u32 = 8973341;
pub const fnHdmaobjInstr_EC9F_ClearVars: u32 = 8973471;
pub const fnHdmaobjPreInstr_ECB6: u32 = 8973494;
pub const fnCinematicFunctionOpening: i32 = 0x8b9b68;
pub const fnXray_SetupStage1_FreezeTimeBackup: u32 = 9554681;
pub const fnXray_SetupStage2_ReadBg1_2ndScreen: u32 = 9554716;
pub const fnXray_SetupStage3_ReadBg1_1stScreen: u32 = 9554775;
pub const fnXray_SetupStage4: u32 = 9554830;
pub const fnXray_SetupStage5: u32 = 9556179;
pub const fnXray_SetupStage6: u32 = 9556339;
pub const fnXray_SetupStage7: u32 = 9556384;
pub const fnHdmaobjPreInstr_XraySetup: u32 = 9556607;
pub const fnXray_SetupStage8_SetBackdropColor: u32 = 9556668;
pub const fnInitializeSuitPickupHdma: u32 = 9557650;
pub const fnnullsub_309: u32 = 10785852;
pub fn nullsub_9(j: u16, game: &mut Game) -> u16 {
    j
}
pub unsafe fn InitializeLayerBlending(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x60) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x61) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x62) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 19 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 4 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6c) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6d) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6e) = 2 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = 51 as u8;
}
pub unsafe fn LayerBlendFunc_4_PhantoonIntro(j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 17 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 4 as u8;
    j
}
pub unsafe fn LayerBlendFunc_6(j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 20 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = 39 as u8;
    j
}
pub unsafe fn LayerBlendFunc_8(j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 20 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = 34 as u8;
    j
}
pub unsafe fn LayerBlendFunc_A(j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = 50 as u8;
    j
}
pub unsafe fn LayerBlendFunc_C(j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x6e) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = -(94 as i32) as u8;
    j
}
pub fn nullsub_10(j: u16, game: &mut Game) -> u16 {
    j
}
pub unsafe fn LayerBlendFunc_10(j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x61) = 2 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x62) = 32 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6d) = 4 as u8;
    j
}
pub unsafe fn LayerBlendFunc_14(j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = -(77 as i32) as u8;
    j
}
pub unsafe fn LayerBlendFunc_16(mut j: u16, game: &mut Game) -> u16 {
    *(&raw mut j as *mut u8).offset(0) = 4 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 17 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 6 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = -(79 as i32) as u8;
    j
}
pub unsafe fn LayerBlendFunc_1A(mut j: u16, game: &mut Game) -> u16 {
    *(&raw mut j as *mut u8).offset(0) = 4 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 21 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 2 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = 53 as u8;
    j
}
pub unsafe fn LayerBlendFunc_1C(j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 21 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 2 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = 85 as u8;
    j
}
pub unsafe fn LayerBlendFunc_18(mut j: u16, game: &mut Game) -> u16 {
    j = 2 as u16;
    if (*(&raw mut game.g_ram as *mut u8).offset(0x84) as i32 & 0x30 as i32) == 0x30i32 {
        *(&raw mut game.g_ram as *mut u8).offset(0x71) = 36 as u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 19 as u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 4 as u8;
    }
    j
}
pub fn nullsub_11(j: u16, game: &mut Game) -> u16 {
    j
}
pub unsafe fn LayerBlendFunc_26(j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = 119 as u8;
    j
}
pub unsafe fn LayerBlendFunc_28(j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x6e) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = -(77 as i32) as u8;
    if (ram_word!(game, fx_layer_blending_config_c) as i32 & 0x8000 as i32) == 0i32 {
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 37 as u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64 as u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80 as u8;
    }
    j
}
pub unsafe fn LayerBlendFunc_2A(j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x6e) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = -(77 as i32) as u8;
    if (ram_word!(game, fx_layer_blending_config_c) as i32 & 0x8000 as i32) == 0i32 {
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 38 as u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 66 as u8;
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80 as u8;
    }
    j
}
pub unsafe fn LayerBlendFunc_2C(j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x6e) = 0 as u8;
    j
}
pub unsafe fn LayerBlendFunc_2E(j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = -(77 as i32) as u8;
    j
}
pub unsafe fn LayerBlendFunc_32(j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 68 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = -(78 as i32) as u8;
    j
}
pub unsafe fn LayerBlendFunc_34(mut j: u16, game: &mut Game) -> u16 {
    j = 6 as u16;
    j
}
pub unsafe fn LayerBlendFunc_24(j: u16, game: &mut Game) -> u16 {
    *(&raw mut game.g_ram as *mut u8).offset(0x60) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x61) = 2 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x62) = 32 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 19 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 4 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6c) = 19 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6d) = 4 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6e) = 16 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = 51 as u8;
    j
}
pub unsafe fn HandleLayerBlendingXrayCanShowBlocks(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x60) = -(56 as i32) as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x61) = 8 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x62) = 0x80 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 19 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 4 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6c) = 3 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6d) = 4 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6e) = 34 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) =
        (*(&raw mut game.g_ram as *mut u8).offset(0x71) as i32 & 0x80 as i32 | 0x73 as i32) as u8;
}
pub unsafe fn HandleLayerBlendingXrayCantShowBlocks(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x60) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x61) = 8 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x62) = 0x80 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 19 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 4 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6c) = 3 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6d) = 4 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6e) = 34 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) =
        (*(&raw mut game.g_ram as *mut u8).offset(0x71) as i32 & 0x80 as i32 | 0x61 as i32) as u8;
    if ram_word!(game, room_ptr) as i32 == addr_kRoom_cefb as i32 {
        *(&raw mut game.g_ram as *mut u8).offset(0x69) = 17 as u8;
    }
}
pub unsafe fn HandleLayerBlendingXrayFirefleaRoom(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x60) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x61) = 8 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x62) = 0x80 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 19 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6c) = 3 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6d) = 4 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6e) = 32 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = -(77 as i32) as u8;
}
pub unsafe fn LayerBlendPowerBombFunc_0(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x60) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x61) = 8 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x62) = 0x80 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6e) = 2 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = 55 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6c) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6d) = 4 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 19 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 4 as u8;
}
pub unsafe fn LayerBlendPowerBombFunc_4(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x60) = 0x80 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x61) = 8 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x62) = 0x80 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6e) = 2 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = 55 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6c) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6d) = 6 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 17 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 6 as u8;
}
pub unsafe fn LayerBlendPowerBombFunc_6(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x60) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x61) = 8 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x62) = 0x80 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6e) = 2 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = 49 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6c) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6d) = 4 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 19 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6b) = 4 as u8;
}
static kLayerBlendFuncTable: [Option<Func_Y_Y>; 27] = unsafe {
    [
        Some(nullsub_9 as unsafe fn(u16, &mut Game) -> u16),
        Some(nullsub_9 as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_4_PhantoonIntro as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_6 as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_8 as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_A as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_C as unsafe fn(u16, &mut Game) -> u16),
        Some(nullsub_10 as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_10 as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_10 as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_14 as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_16 as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_18 as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_1A as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_1C as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_18 as unsafe fn(u16, &mut Game) -> u16),
        Some(nullsub_11 as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_14 as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_24 as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_26 as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_28 as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_2A as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_2C as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_2E as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_18 as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_32 as unsafe fn(u16, &mut Game) -> u16),
        Some(LayerBlendFunc_34 as unsafe fn(u16, &mut Game) -> u16),
    ]
};
pub unsafe fn LayerBlendingHandler(game: &mut Game) {
    let mut v0: u16 = 0 as u16;
    let v1: u8 = ram_word!(game, fx_layer_blending_config_c) as u8;
    if ram_word!(game, fx_layer_blending_config_c) as u8 != 0 {
        InitializeLayerBlending(game);
        v0 = kLayerBlendFuncTable[(v1 as i32 >> 1 as i32) as usize]
            .expect("non-null function pointer")(v0, game);
    }
    if (ram_word!(game, fx_layer_blending_config_c) as i32 & 0x8000 as i32) == 0i32 {
        if (ram_word!(game, fx_layer_blending_config_c) as i32 & 0x4000 as i32) != 0i32 {
            HandleLayerBlendingXrayCanShowBlocks(game);
        } else if (ram_word!(game, fx_layer_blending_config_c) as i32 & 0x2000 as i32) != 0i32 {
            HandleLayerBlendingXrayCantShowBlocks(game);
        } else if (ram_word!(game, fx_layer_blending_config_c) as i32 & 0x1000 as i32) != 0i32 {
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
pub unsafe fn HandleLayerBlendingPowerBomb(mut j: u16, game: &mut Game) {
    if ram_word!(game, room_ptr) as i32 == addr_kRoom_a66a as i32 {
        j = 6 as u16;
    }
    kLayerBlendPowerBombFuncs[(j as i32 >> 1 as i32) as usize].expect("non-null function pointer")(
        game,
    );
}
pub unsafe fn EnableHdmaObjects(game: &mut Game) {
    update_ram_word!(game.hdma_objects_enable_flag, |fresh31| (fresh31)
        | (0x8000u16));
}
pub unsafe fn DisableHdmaObjects(game: &mut Game) {
    update_ram_word!(
        game.hdma_objects_enable_flag,
        |fresh30| (fresh30 as i32 & !(0x8000 as i32)) as u16
    );
}
pub unsafe fn WaitUntilEndOfVblankAndClearHdma(game: &mut Game) {
    WriteReg(MDMAEN as u16, 0 as u8, game);
    WriteReg(HDMAEN as u16, 0 as u8, game);
    sub_8882AC(game);
}
pub unsafe fn sub_8882AC(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x85) = 0 as u8;
    let mut i: i32 = 10 as i32;
    while i >= 0i32 {
        set_ram_word!(game.hdma_object_channels_bitmask[(i >> 1i32)] = 0 as u16);
        i -= 2 as i32;
    }
}
pub unsafe fn InitializeSpecialEffectsForNewRoom(game: &mut Game) {
    set_ram_word!(game.earthquake_sfx_index = 0 as u16);
    set_ram_word!(game.earthquake_sfx_timer = 0 as u16);
    if ram_word!(game, room_ptr) as i32 == addr_kRoom_9804 as i32
        || ram_word!(game, room_ptr) as i32 == addr_kRoom_96ba as i32
        || ram_word!(game, room_ptr) as i32 == addr_kRoom_b32e as i32
        || ram_word!(game, room_ptr) as i32 == addr_kRoom_b457 as i32
        || ram_word!(game, room_ptr) as i32 == addr_kRoom_dd58 as i32
        || ram_word!(game, room_ptr) as i32 == addr_kRoom_dede as i32
    {
        set_ram_word!(game.earthquake_sfx_timer = -(1 as i32) as u16);
    }
    set_ram_word!(game.debug_disable_minimap = 0 as u16);
    let mut i: i32 = 32 as i32;
    while i != 0x80i32 {
        WriteReg((i + 17152 as i32) as SnesRegs as u16, 0 as u8, game);
        WriteReg((i + 17153 as i32) as SnesRegs as u16, 0x13 as u8, game);
        WriteReg((i + 17154 as i32) as SnesRegs as u16, 0 as u8, game);
        WriteReg((i + 17155 as i32) as SnesRegs as u16, 0 as u8, game);
        WriteReg((i + 17156 as i32) as SnesRegs as u16, 0 as u8, game);
        WriteReg((i + 17157 as i32) as SnesRegs as u16, 0 as u8, game);
        WriteReg((i + 17158 as i32) as SnesRegs as u16, 0 as u8, game);
        WriteReg((i + 17160 as i32) as SnesRegs as u16, 0 as u8, game);
        WriteReg((i + 17161 as i32) as SnesRegs as u16, 0 as u8, game);
        i += 16 as i32;
    }
    set_ram_word!(game.fx_y_subpos = 0 as u16);
    set_ram_word!(game.fx_y_pos = -(1 as i32) as u16);
    set_ram_word!(game.lava_acid_y_subpos = 0 as u16);
    set_ram_word!(game.lava_acid_y_pos = -(1 as i32) as u16);
    set_ram_word!(game.hud_bg3_xpos = 0 as u16);
    set_ram_word!(game.hud_bg3_ypos = 0 as u16);
    set_ram_word!(game.bg3_xpos = 0 as u16);
    set_ram_word!(game.bg3_ypos = 0 as u16);
    set_ram_word!(game.irq_enable_mode7 = 0 as u16);
    set_ram_word!(game.camera_distance_index = 0 as u16);
    set_ram_word!(game.tourian_entrance_statue_animstate = 0 as u16);
    set_ram_word!(game.tourian_entrance_statue_finished = 0 as u16);
    set_ram_word!(game.earthquake_timer = 0 as u16);
    set_ram_word!(game.phantom_related_layer_flag = 0 as u16);
    set_ram_word!(game.power_bomb_explosion_status = 0 as u16);
    set_ram_word!(game.power_bomb_flag = 0 as u16);
    set_ram_word!(game.power_bomb_explosion_radius = 0 as u16);
    set_ram_word!(game.fx_tilemap_ptr = 0 as u16);
    set_ram_word!(game.fx_type = 0 as u16);
    set_ram_word!(game.fx_base_y_subpos = 0x8000 as u16);
    set_ram_word!(game.fx_base_y_pos = 0 as u16);
    set_ram_word!(game.fx_target_y_pos = 0 as u16);
    set_ram_word!(game.fx_y_vel = 0 as u16);
    set_ram_word!(game.fx_liquid_options = 0 as u16);
    set_ram_word!(game.fx_timer = 0 as u16);
    set_ram_word!(game.tide_phase = 0 as u16);
    set_ram_word!(game.fx_y_suboffset = 0 as u16);
    set_ram_word!(game.fx_y_offset = 0 as u16);
    set_ram_word!(game.fx_layer_blending_config_a = 2 as u16);
    set_ram_word!(game.reg_BG3HOFS = 0 as u16);
    set_ram_word!(game.reg_BG3VOFS = 0 as u16);
    set_ram_word!(game.layer2_x_pos = 0 as u16);
    set_ram_word!(game.layer2_y_pos = 0 as u16);
    set_ram_word!(game.room_loading_irq_handler = 0 as u16);
    *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x601) as *mut LongPtr)).addr as *mut u8)
        .offset(1) as *mut VoidP) = -(30720 as i32) as VoidP;
    *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x604) as *mut LongPtr)).addr as *mut u8)
        .offset(1) as *mut VoidP) = -(30720 as i32) as VoidP;
    (*((&raw mut game.g_ram as *mut u8).offset(0x601) as *mut LongPtr)).addr =
        ((0x8883e1i32) as u16);
    (*((&raw mut game.g_ram as *mut u8).offset(0x604) as *mut LongPtr)).addr =
        ((0x8883e1i32) as u16);
    WriteReg(WMADDL as u16, 0xf0 as u8, game);
    WriteReg(WMADDM as u16, 0xff as u8, game);
    WriteReg(WMADDH as u16, 1 as u8, game);
    *(&raw mut game.g_ram as *mut u8).offset(0x85) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x57) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x69) = 19 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x72) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x5d) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x5e) = 4 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x59) = 73 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x5a) = 90 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x5b) = 90 as u8;
}
unsafe fn SpawnHdmaObjectInner(
    k: u16,
    p: *mut u16,
    r18: u16,
    r20: u16,
    r24: u16,
    game: &mut Game,
) -> u16 {
    let v2: i32 = k as i32 >> 1 as i32;
    set_ram_word!(game.hdma_object_pre_instructions[v2] = ((0x8884b8i32) as u16));
    set_ram_word!(game.hdma_object_pre_instruction_bank[v2] = 136 as u16);
    set_ram_word!(game.hdma_object_instruction_list_pointers[v2] = *p.offset(1));
    set_ram_word!(game.hdma_object_instruction_timers[v2] = 1 as u16);
    set_ram_word!(game.hdma_object_timers[v2] = 0 as u16);
    set_ram_word!(game.hdma_object_A[v2] = 0 as u16);
    set_ram_word!(game.hdma_object_B[v2] = 0 as u16);
    set_ram_word!(game.hdma_object_C[v2] = 0 as u16);
    set_ram_word!(game.hdma_object_D[v2] = 0 as u16);
    set_ram_word!(game.hdma_object_channels_bitmask[v2] = u16::swap_bytes(r18));
    set_ram_word!(game.hdma_object_bank_slot[v2] = (r24) | (r20));
    WriteRegWord((r20 as i32 + DMAP0 as i32) as SnesRegs as u16, *p, game);
    k
}
pub unsafe fn SpawnHdmaObject(db: u8, p: *const ::core::ffi::c_void, game: &mut Game) -> u16 {
    let mut r18: u16 = 1024 as u16;
    let mut r20: u16 = 32 as u16;
    let mut v3: u16 = 0 as u16;
    loop {
        if ram_word!(game, hdma_object_channels_bitmask[(v3 as i32 >> 1i32)]) == 0 {
            return SpawnHdmaObjectInner(
                v3,
                p as *mut u16,
                r18,
                r20,
                ((db as i32) << 8 as i32) as u16,
                game,
            );
        }
        let v4: bool = r18 as i32 >> 15 as i32 != 0;
        r18 = (r18).wrapping_mul(2u16);
        if v4 {
            return -(1 as i32) as u16;
        }
        r20 = (r20).wrapping_add(16u16);
        v3 = (v3).wrapping_add(2u16);
        if v3 == 12u16 {
            return -(1 as i32) as u16;
        }
    }
}
pub unsafe fn SpawnHdmaObjectToSlot0xA(db: u8, p: *const ::core::ffi::c_void, game: &mut Game) {
    SpawnHdmaObjectInner(
        0xa as u16,
        p as *mut u16,
        0x8000 as u16,
        0x70 as u16,
        ((db as i32) << 8 as i32) as u16,
        game,
    );
}
pub unsafe fn HdmaObjectHandler(game: &mut Game) {
    let mut v1: i8 = 0;
    HandleMusicQueue(game);
    if ram_word!(game, time_is_frozen_flag) == 0
        && (ram_word!(game, power_bomb_explosion_status) as i32 & 0x4000 as i32) != 0i32
    {
        set_ram_word!(game.power_bomb_explosion_status = 0x8000 as u16);
        static unk_8884D5: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x40 as u8, 0x28 as u8, 0x8ace as VoidP);
        static unk_8884DD: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x40 as u8, 0x29 as u8, 0x8b80 as VoidP);
        SpawnHdmaObject(
            0x88 as u8,
            &raw const unk_8884D5 as *const ::core::ffi::c_void,
            game,
        );
        SpawnHdmaObject(
            0x88 as u8,
            &raw const unk_8884DD as *const ::core::ffi::c_void,
            game,
        );
    }
    set_ram_word!(game.fx_layer_blending_config_c = ram_word!(game, fx_layer_blending_config_a));
    if (ram_word!(game, hdma_objects_enable_flag) as i32 & 0x8000 as i32) != 0i32 {
        *(&raw mut game.g_ram as *mut u8).offset(0x85) = 0 as u8;
        let mut i: i32 = 0 as i32;
        loop {
            set_ram_word!(game.hdma_object_index = i as u16);
            v1 = *((&raw mut game.g_ram as *mut u8).offset(0x18b4) as *mut u16 as *mut u8)
                .offset(i as isize) as i8;
            if v1 != 0 {
                let ref mut fresh58 = *(&raw mut game.g_ram as *mut u8).offset(0x85);
                *fresh58 = (*fresh58) | ((v1) as u8);
                HdmaobjInstructionHandler(i as u8, game);
                i = ram_word!(game, hdma_object_index) as i32;
            }
            i += 2 as i32;
            if (i == 12i32) {
                break;
            }
        }
        LayerBlendingHandler(game);
    }
}
pub unsafe fn HdmaobjInstructionHandler(k: u8, game: &mut Game) {
    let kh: i32 = k as i32 >> 1 as i32;
    CallHdmaobjPreInstr(
        ((ram_word!(game, hdma_object_pre_instruction_bank[kh]) as i32) << 16 as i32
            | ram_word!(game, hdma_object_pre_instructions[kh]) as i32) as u32,
        k as u16,
        game,
    );
    let ref mut fresh59 =
        *((&raw mut game.g_ram as *mut u8).offset(0x18e4) as *mut u16).offset(kh as isize);
    let fresh60 = *fresh59;
    *fresh59 = (*fresh59).wrapping_sub(1);
    if fresh60 == 1u16 {
        let base: *const u8 = RomBankBase(
            *((&raw mut game.g_ram as *mut u8).offset(0x18c0) as *mut u16 as *mut u8)
                .offset(k as i32 as isize)
                .offset(1),
            game,
        );
        let mut p: *const u8 =
            base.offset(ram_word!(game, hdma_object_instruction_list_pointers[kh]) as i32 as isize);
        while *(p as *mut u16) as i32 & 0x8000 as i32 != 0 {
            p = CallHdmaobjInstr(
                ((*(p as *mut u16)) as u32) | (0x880000u32),
                k as u16,
                p.offset(2),
                game,
            );
            if (p as usize) < 0x10000 as i32 as usize {
                if p.is_null() {
                    return;
                }
                p = base.offset(p as usize as isize);
            }
        }
        set_ram_word!(game.hdma_object_instruction_timers[kh] = *(p as *mut u16));
        set_ram_word!(game.hdma_object_table_pointers[kh] = *(p.offset(2) as *mut u16));
        set_ram_word!(
            game.hdma_object_instruction_list_pointers[kh] =
                p.offset(4).offset_from(base) as ::core::ffi::c_long as u16
        );
    }
}
pub unsafe fn HdmaobjInstr_Delete(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.hdma_object_channels_bitmask[(k as i32 >> 1i32)] = 0 as u16);
    ::core::ptr::null::<u8>()
}
pub unsafe fn HdmaobjInstr_SetPreInstr(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.hdma_object_pre_instructions[(k as i32 >> 1i32)] = *(hdp as *mut u16));
    *((&raw mut game.g_ram as *mut u8).offset(0x18fc) as *mut u16 as *mut u8)
        .offset(k as i32 as isize) = *hdp.offset(2);
    hdp.offset(3)
}
pub unsafe fn HdmaobjInstr_ClearPreInstr(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(
        game.hdma_object_pre_instructions[(k as i32 >> 1i32)] = addr_locret_88858A as u16
    );
    hdp
}
pub unsafe fn CallHdmaobjInstrFunc(ea: u32, k: u16, game: &mut Game) {
    match ea {
        8948500 => Hdmaobj_PreExplodeWhite(game),
        8948530 => Hdmaobj_PreExplodeYellow(game),
        8948537 => Hdmaobj_ExplodeYellow(game),
        8948551 => Hdmaobj_ExplodeWhite(game),
        8948558 => Hdmaobj_CleanUpTryCrystalFlash(k, game),
        8954596 => CrystalFlashSetupPart1(game),
        8954633 => CrystalFlashSetupPart2(game),
        8954647 => CrystalFlashCleanup(k, game),
        8970129 => (),
        8970130 => (),
        8972070 => (),
        8972135 => InitializeRainbowBeam(game),
        9554681 => Xray_SetupStage1_FreezeTimeBackup(k, game),
        9554716 => Xray_SetupStage2_ReadBg1_2ndScreen(game),
        9554775 => Xray_SetupStage3_ReadBg1_1stScreen(game),
        9554830 => Xray_SetupStage4(game),
        9556179 => Xray_SetupStage5(game),
        9556339 => Xray_SetupStage6(game),
        9556384 => Xray_SetupStage7(game),
        9556668 => Xray_SetupStage8_SetBackdropColor(game),
        9557650 => InitializeSuitPickupHdma(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn HdmaobjInstr_CallFarFunc(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    CallHdmaobjInstrFunc(Load24(hdp as *mut LongPtr), k, game);
    hdp.offset(3)
}
pub unsafe fn HdmaobjInstr_Goto(k: u16, hdp: *const u8) -> *const u8 {
    *(hdp as *mut u16) as usize as *const u8
}
pub unsafe fn HdmaobjInstr_GotoRel(k: u16, hdp: *const u8) -> *const u8 {
    hdp.offset(*hdp as i8 as i32 as isize)
}
pub unsafe fn HdmaobjInstr_DecrementAndGoto(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    let v2: i32 = k as i32 >> 1 as i32;
    let ref mut fresh63 =
        *((&raw mut game.g_ram as *mut u8).offset(0x1908) as *mut u16).offset(v2 as isize);
    let fresh64 = *fresh63;
    *fresh63 = (*fresh63).wrapping_sub(1);
    if fresh64 == 1u16 {
        hdp.offset(2)
    } else {
        *(hdp as *mut u16) as usize as *const u8
    }
}
pub unsafe fn HdmaobjInstr_DecrementAndGotoRel(
    k: u16,
    hdp: *const u8,
    game: &mut Game,
) -> *const u8 {
    let v2: i32 = k as i32 >> 1 as i32;
    let ref mut fresh61 =
        *((&raw mut game.g_ram as *mut u8).offset(0x1908) as *mut u16).offset(v2 as isize);
    let fresh62 = *fresh61;
    *fresh61 = (*fresh61).wrapping_sub(1);
    if fresh62 == 1u16 {
        hdp.offset(1)
    } else {
        hdp.offset(*hdp as i8 as i32 as isize)
    }
}
pub unsafe fn HdmaobjInstr_SetTimer(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    *((&raw mut game.g_ram as *mut u8).offset(0x1908) as *mut u16 as *mut u8)
        .offset(k as i32 as isize) = *hdp;
    hdp.offset(1)
}
pub unsafe fn HdmaobjInstr_SetHdmaControl(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    WriteReg(
        (*(((&raw mut game.g_ram as *mut u8).offset(0x18c0) as *mut u16)
            .offset((k as i32 >> 1 as i32) as isize) as *mut u16 as *mut u8)
            .offset(0) as i32
            + DMAP0 as i32) as SnesRegs as u16,
        *hdp,
        game,
    );
    hdp.offset(1)
}
pub unsafe fn HdmaobjInstr_SetHdmaTarget(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    WriteReg(
        (*(((&raw mut game.g_ram as *mut u8).offset(0x18c0) as *mut u16)
            .offset((k as i32 >> 1 as i32) as isize) as *mut u16 as *mut u8)
            .offset(0) as i32
            + BBAD0 as i32) as SnesRegs as u16,
        *hdp,
        game,
    );
    hdp.offset(1)
}
pub unsafe fn HdmaobjInstr_SetHdmaTablePtr(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.hdma_object_table_pointers[(k as i32 >> 1i32)] = *(hdp as *mut u16));
    hdp.offset(2)
}
pub unsafe fn HdmaobjInstr_SetHdmaTableBank(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    WriteReg(
        (*(((&raw mut game.g_ram as *mut u8).offset(0x18c0) as *mut u16)
            .offset((k as i32 >> 1 as i32) as isize) as *mut u16 as *mut u8)
            .offset(0) as i32
            + A1B0 as i32) as SnesRegs as u16,
        *hdp,
        game,
    );
    hdp.offset(1)
}
pub unsafe fn HdmaobjInstr_SetIndirectHdmaDataBank(
    k: u16,
    hdp: *const u8,
    game: &mut Game,
) -> *const u8 {
    WriteReg(
        (*(((&raw mut game.g_ram as *mut u8).offset(0x18c0) as *mut u16)
            .offset((k as i32 >> 1 as i32) as isize) as *mut u16 as *mut u8)
            .offset(0) as i32
            + DAS00 as i32) as SnesRegs as u16,
        *hdp,
        game,
    );
    hdp.offset(1)
}
pub unsafe fn HdmaobjInstr_Sleep(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    let base: *const u8 = RomBankBase(
        *((&raw mut game.g_ram as *mut u8).offset(0x18c0) as *mut u16 as *mut u8)
            .offset(k as i32 as isize)
            .offset(1),
        game,
    );
    set_ram_word!(
        game.hdma_object_instruction_list_pointers[(k as i32 >> 1i32)] =
            (hdp.offset_from(base) as ::core::ffi::c_long - 2 as ::core::ffi::c_long) as u16
    );
    ::core::ptr::null::<u8>()
}
pub unsafe fn RaiseOrLowerFx(game: &mut Game) -> u8 {
    if (ram_word!(game, fx_target_y_pos) as i32 & 0x8000 as i32) != 0i32 {
        return 1 as u8;
    }
    let r24_r22: u32 = ((ram_word!(game, fx_y_vel) as i16 as i32) << 8 as i32) as u32;
    if (ram_word!(game, fx_y_vel) as i32 & 0x8000 as i32) == 0i32 {
        let mut v2: u16 = (r24_r22.wrapping_add(
            (ram_word!(game, fx_base_y_pos) as u32) << 16 as i32
                | ram_word!(game, fx_base_y_subpos) as u32,
        ) >> 16 as i32) as u16;
        update_ram_word!(
            game.fx_base_y_subpos,
            |fresh13| (fresh13 as u32).wrapping_add(r24_r22) as u16
        );
        if (v2 as i32 & 0x8000 as i32) != 0i32 {
            v2 = -(1 as i32) as u16;
        }
        set_ram_word!(game.fx_base_y_pos = v2);
        if ram_word!(game, fx_target_y_pos) >= v2 {
            0 as u8
        } else {
            set_ram_word!(game.fx_base_y_pos = ram_word!(game, fx_target_y_pos));
            set_ram_word!(game.fx_base_y_subpos = 0 as u16);
            1 as u8
        }
    } else {
        let mut v0: u16 = (r24_r22.wrapping_add(
            (ram_word!(game, fx_base_y_pos) as u32) << 16 as i32
                | ram_word!(game, fx_base_y_subpos) as u32,
        ) >> 16 as i32) as u16;
        update_ram_word!(
            game.fx_base_y_subpos,
            |fresh14| (fresh14 as u32).wrapping_add(r24_r22) as u16
        );
        if (v0 as i32 & 0x8000 as i32) != 0i32 {
            v0 = 0 as u16;
        }
        set_ram_word!(game.fx_base_y_pos = v0);
        if ram_word!(game, fx_target_y_pos) >= v0 {
            set_ram_word!(game.fx_base_y_pos = ram_word!(game, fx_target_y_pos));
            set_ram_word!(game.fx_base_y_subpos = 0 as u16);
        }
        (ram_word!(game, fx_target_y_pos) >= v0) as u8
    }
}
pub unsafe fn HdmaobjPreInstr_XrayFunc0_NoBeam(k: u16, game: &mut Game) {
    if (ram_word!(game, button_config_run_b) as i32 & ram_word!(game, joypad1_lastkeys) as i32)
        != 0i32
    {
        CalculateXrayHdmaTable(game);
        update_ram_word!(game.demo_input_pre_instr, wrapping_add(1));
    } else {
        set_ram_word!(game.demo_input_pre_instr = 3 as u16);
    };
}
pub unsafe fn HdmaobjPreInstr_XrayFunc1_BeamWidening(k: u16, game: &mut Game) {
    if (ram_word!(game, button_config_run_b) as i32 & ram_word!(game, joypad1_lastkeys) as i32)
        != 0i32
    {
        AddToHiLo(
            (&raw mut game.g_ram as *mut u8).offset(0xa7c) as *mut u16,
            (&raw mut game.g_ram as *mut u8).offset(0xa7e) as *mut u16,
            2048 as u32,
        );
        AddToHiLo(
            (&raw mut game.g_ram as *mut u8).offset(0xa84) as *mut u16,
            (&raw mut game.g_ram as *mut u8).offset(0xa86) as *mut u16,
            (ram_word!(game, demo_input_instr_timer) as u32) << 16 as i32
                | ram_word!(game, demo_input_instr_ptr) as u32,
        );
        if ram_word!(game, demo_input) as i32 - 11 as i32 & 0x8000 as i32 == 0 {
            set_ram_word!(game.demo_input_new = 0 as u16);
            set_ram_word!(game.demo_input = 10 as u16);
            update_ram_word!(game.demo_input_pre_instr, wrapping_add(1));
        }
        CalculateXrayHdmaTable(game);
    } else {
        set_ram_word!(game.demo_input_pre_instr = 3 as u16);
    };
}
pub unsafe fn HdmaobjPreInstr_XrayFunc2_FullBeam(k: u16, game: &mut Game) {
    if (ram_word!(game, button_config_run_b) as i32 & ram_word!(game, joypad1_lastkeys) as i32)
        != 0i32
    {
        HandleMovingXrayUpDown(game);
        CalculateXrayHdmaTable(game);
    } else {
        update_ram_word!(game.demo_input_pre_instr, wrapping_add(1));
    };
}
pub unsafe fn HandleMovingXrayUpDown(game: &mut Game) {
    if (ram_word!(game, button_config_up) as i32 & ram_word!(game, joypad1_lastkeys) as i32) != 0i32
    {
        MoveXrayUp(game);
    } else if (ram_word!(game, button_config_down) as i32
        & ram_word!(game, joypad1_lastkeys) as i32)
        != 0i32
    {
        MoveXrayDown(game);
    }
}
pub unsafe fn MoveXrayUp(game: &mut Game) {
    let mut v0: bool = false;
    let mut v1: i16 = 0;
    if ram_word!(game, xray_angle) as i32 - 128 as i32 & 0x8000 as i32 != 0 {
        if ram_word!(game, xray_angle) != ram_word!(game, demo_input)
            && (((ram_word!(game, xray_angle)) as i16)
                .wrapping_sub((ram_word!(game, demo_input)) as i16)
                < 0i16
                || {
                    v0 = ram_word!(game, xray_angle) != 0u16;
                    update_ram_word!(game.xray_angle, wrapping_sub(1));
                    ((ram_word!(game, xray_angle) as i32
                        - (!v0 as i32 + ram_word!(game, demo_input) as i32))
                        as i16)
                        < 0i16
                })
        {
            set_ram_word!(game.xray_angle = ram_word!(game, demo_input));
        }
    } else if (ram_word!(game, demo_input) as i32 + ram_word!(game, xray_angle) as i32) != 256i32
        && (((ram_word!(game, demo_input) as i32 + ram_word!(game, xray_angle) as i32 - 256 as i32)
            as i16)
            >= 0i16
            || {
                v1 = (((ram_word!(game, demo_input) as u32) << 16 as i32
                    | ram_word!(game, xray_angle) as u32)
                    .wrapping_add(
                        (ram_word!(game, xray_angle) as u32) << 16 as i32 | 1 as u16 as u32,
                    )
                    >> 16 as i32) as i16;
                update_ram_word!(game.xray_angle, wrapping_add(1));
                v1 != 256i16 && (v1).wrapping_sub(256i16) >= 0i16
            })
    {
        set_ram_word!(game.xray_angle = (256u16).wrapping_sub(ram_word!(game, demo_input)));
    }
}
pub unsafe fn MoveXrayDown(game: &mut Game) {
    let mut v0: i16 = 0;
    let mut v1: bool = false;
    if ram_word!(game, xray_angle) as i32 - 128 as i32 & 0x8000 as i32 != 0 {
        if (ram_word!(game, demo_input) as i32 + ram_word!(game, xray_angle) as i32) != 128i32
            && (((ram_word!(game, demo_input) as i32 + ram_word!(game, xray_angle) as i32
                - 128 as i32) as i16)
                >= 0i16
                || {
                    v0 = (((ram_word!(game, demo_input) as u32) << 16 as i32
                        | ram_word!(game, xray_angle) as u32)
                        .wrapping_add(
                            (ram_word!(game, xray_angle) as u32) << 16 as i32 | 1 as u16 as u32,
                        )
                        >> 16 as i32) as i16;
                    update_ram_word!(game.xray_angle, wrapping_add(1));
                    v0 != 128i16 && (v0).wrapping_sub(128i16) >= 0i16
                })
        {
            set_ram_word!(game.xray_angle = (128u16).wrapping_sub(ram_word!(game, demo_input)));
        }
    } else if (ram_word!(game, xray_angle) as i32 - ram_word!(game, demo_input) as i32) != 128i32
        && (((ram_word!(game, xray_angle) as i32 - ram_word!(game, demo_input) as i32 - 128 as i32)
            as i16)
            < 0i16
            || {
                v1 = ram_word!(game, xray_angle) != 0u16;
                update_ram_word!(game.xray_angle, wrapping_sub(1));
                ram_word!(game, xray_angle) as i32
                    - (!v1 as i32 + ram_word!(game, demo_input) as i32)
                    != 128i32
                    && ((ram_word!(game, xray_angle) as i32
                        - (!v1 as i32 + ram_word!(game, demo_input) as i32)
                        - 128 as i32) as i16)
                        < 0i16
            })
    {
        set_ram_word!(game.xray_angle = (ram_word!(game, demo_input)).wrapping_add(128u16));
    }
}
pub unsafe fn CalculateXrayHdmaTable(game: &mut Game) {
    let mut v0: i16 = 0;
    if *(&raw mut game.g_ram as *mut u8).offset(0xa1e) == 4u8 {
        v0 = (ram_word!(game, samus_x_pos) as i32 - ram_word!(game, layer1_x_pos) as i32 - 3 as i32)
            as i16;
    } else {
        v0 = (ram_word!(game, samus_x_pos) as i32 - ram_word!(game, layer1_x_pos) as i32 + 3 as i32)
            as i16;
    }
    let mut v1: u16 = 0;
    if *(&raw mut game.g_ram as *mut u8).offset(0xa1f) == 5u8 {
        v1 = (ram_word!(game, samus_y_pos) as i32
            - ram_word!(game, layer1_y_pos) as i32
            - 12 as i32) as u16;
    } else {
        v1 = (ram_word!(game, samus_y_pos) as i32
            - ram_word!(game, layer1_y_pos) as i32
            - 16 as i32) as u16;
    }
    let current_block_11: u64;
    if v0 < 0i16 {
        if *(&raw mut game.g_ram as *mut u8).offset(0xa1e) != 4u8 {
            current_block_11 = 13131151160068275183;
        } else {
            current_block_11 = 10599921512955367680;
        }
    } else {
        if (v0).wrapping_sub(256i16) < 0i16 {
            CalculateXrayHdmaTableInner(
                v0 as u16,
                v1,
                ram_word!(game, xray_angle),
                ram_word!(game, demo_input),
                false_0 != 0,
                (&raw mut game.g_ram as *mut u8).offset(0x9800) as *mut u16,
                game,
            );
            return;
        }
        if *(&raw mut game.g_ram as *mut u8).offset(0xa1e) != 8u8 {
            current_block_11 = 13131151160068275183;
        } else {
            current_block_11 = 10599921512955367680;
        }
    }
    if current_block_11 == 10599921512955367680 {
    } else {
        CalculateXrayHdmaTableInner(
            v0 as u16,
            v1,
            ram_word!(game, xray_angle),
            ram_word!(game, demo_input),
            true_0 != 0,
            (&raw mut game.g_ram as *mut u8).offset(0x9800) as *mut u16,
            game,
        );
        return;
    }
    let mut i: i32 = 510 as i32;
    while i >= 0i32 {
        set_ram_word!(game.hdma_table_1[(i >> 1i32)] = 255 as u16);
        i -= 2 as i32;
    }
}
pub unsafe fn HdmaobjPreInstr_XrayFunc3_DeactivateBeam(k: u16, game: &mut Game) {
    let mut v1: i16 = 0;
    let mut v4: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    mov24(
        (&raw mut game.g_ram as *mut u8).offset(0xa88) as *mut LongPtr,
        0x980001 as u32,
    );
    *(((&raw mut game.g_ram as *mut u8).offset(0xa8a) as *mut u16 as *mut u8).offset(1)
        as *mut u16) = 0 as u16;
    set_ram_word!(game.demo_input_prev = 0 as u16);
    set_ram_word!(game.demo_input_prev_new = 0 as u16);
    set_ram_word!(game.demo_backup_prev_controller_input = 0 as u16);
    set_ram_word!(game.hdma_table_1[0] = 255 as u16);
    v1 = 4096 as i16;
    if ram_word!(game, fx_type) != 36u16 {
        v1 = 0x2000 as i16;
        if CanXrayShowBlocks(game) {
            v1 = 0x4000 as i16;
        }
    }
    update_ram_word!(game.fx_layer_blending_config_c, |fresh133| (fresh133)
        | ((v1) as u16));
    set_ram_word!(game.palette_buffer[0] = 0 as u16);
    let v2: i32 = ram_word!(game, hdma_object_index) as i32 >> 1 as i32;
    set_ram_word!(game.reg_BG2HOFS = ram_word!(game, hdma_object_A[v2]));
    set_ram_word!(game.reg_BG2VOFS = ram_word!(game, hdma_object_B[v2]));
    *(&raw mut game.g_ram as *mut u8).offset(0x59) =
        *((&raw mut game.g_ram as *mut u8).offset(0x192c) as *mut u16 as *mut u8)
            .offset(ram_word!(game, hdma_object_index) as i32 as isize);
    let v3: u16 = ram_word!(game, vram_write_queue_tail);
    if ((ram_word!(game, vram_write_queue_tail)) as i16).wrapping_sub(240i16) < 0i16 {
        v4 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
        (*v4).size = 2048 as u16;
        (*v4).src.addr =
            (((&raw mut game.g_ram as *mut u8).offset(0x4000) as *mut Ram4000 as *mut u8)
                .offset_from(&raw mut game.g_ram as *mut u8) as ::core::ffi::c_long
                + 4096 as ::core::ffi::c_long) as VoidP;
        *(&raw mut (*v4).src.bank as *mut u16) = 126 as u16;
        (*v4).vram_dst = ((*(&raw mut game.g_ram as *mut u8).offset(0x59) as i32 & 0xfc as i32)
            << 8 as i32) as u16;
        set_ram_word!(game.vram_write_queue_tail = (v3).wrapping_add(7u16));
        update_ram_word!(game.demo_input_pre_instr, wrapping_add(1));
    }
}
pub unsafe fn HdmaobjPreInstr_XrayFunc4_DeactivateBeam(k: u16, game: &mut Game) {
    let mut v1: i16 = 0;
    let mut v3: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    v1 = 4096 as i16;
    if ram_word!(game, fx_type) != 36u16 {
        v1 = 0x2000 as i16;
        if CanXrayShowBlocks(game) {
            v1 = 0x4000 as i16;
        }
    }
    update_ram_word!(game.fx_layer_blending_config_c, |fresh131| (fresh131)
        | ((v1) as u16));
    let v2: u16 = ram_word!(game, vram_write_queue_tail);
    if ((ram_word!(game, vram_write_queue_tail)) as i16).wrapping_sub(240i16) < 0i16 {
        v3 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
        (*v3).size = 2048 as u16;
        (*v3).src.addr =
            (((&raw mut game.g_ram as *mut u8).offset(0x4000) as *mut Ram4000 as *mut u8)
                .offset_from(&raw mut game.g_ram as *mut u8) as ::core::ffi::c_long
                + 6144 as ::core::ffi::c_long) as VoidP;
        *(&raw mut (*v3).src.bank as *mut u16) = 126 as u16;
        (*v3).vram_dst = (((*(&raw mut game.g_ram as *mut u8).offset(0x59) as i32 & 0xfc as i32)
            << 8 as i32)
            + 1024 as i32) as u16;
        set_ram_word!(game.vram_write_queue_tail = (v2).wrapping_add(7u16));
        update_ram_word!(game.demo_input_pre_instr, wrapping_add(1));
    }
}
pub unsafe fn HdmaobjPreInstr_XrayFunc5_DeactivateBeam(k: u16, game: &mut Game) {
    let mut v1: i16 = 0;
    v1 = 4096 as i16;
    if ram_word!(game, fx_type) != 36u16 {
        v1 = 0x2000 as i16;
        if CanXrayShowBlocks(game) {
            v1 = 0x4000 as i16;
        }
    }
    update_ram_word!(game.fx_layer_blending_config_c, |fresh130| (fresh130)
        | ((v1) as u16));
    if ram_word!(game, time_is_frozen_flag) != 0 {
        set_ram_word!(game.time_is_frozen_flag = 0 as u16);
        set_ram_word!(game.demo_input_pre_instr = 0 as u16);
        set_ram_word!(game.demo_input_instr_timer = 0 as u16);
        set_ram_word!(game.demo_input_instr_ptr = 0 as u16);
        set_ram_word!(game.xray_angle = 0 as u16);
        set_ram_word!(game.demo_input = 0 as u16);
        set_ram_word!(game.demo_input_new = 0 as u16);
        mov24(
            (&raw mut game.g_ram as *mut u8).offset(0xa88) as *mut LongPtr,
            0x980001 as u32,
        );
        *(((&raw mut game.g_ram as *mut u8).offset(0xa8a) as *mut u16 as *mut u8).offset(1)
            as *mut u16) = 0 as u16;
        set_ram_word!(game.demo_input_prev = 0 as u16);
        set_ram_word!(game.demo_input_prev_new = 0 as u16);
        set_ram_word!(game.demo_backup_prev_controller_input = 0 as u16);
        ResponsibleForXrayStandupGlitch(game);
        *((&raw mut game.g_ram as *mut u8).offset(0x18b4) as *mut u16)
            .offset((ram_word!(game, hdma_object_index) as i32 >> 1 as i32) as isize) = 0 as u16;
        QueueSfx1_Max6(0xa as u16, game);
        if (ram_word!(game, fx_type) as u8) != 36u8 {
            *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80 as u8;
            *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64 as u8;
            *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32 as u8;
        }
        let mut i: i32 = 510 as i32 / 2 as i32;
        while i >= 0i32 {
            set_ram_word!(game.hdma_table_1[i] = 0xff as u16);
            i -= 1;
        }
        if ram_word!(game, samus_auto_cancel_hud_item_index) != 0 {
            set_ram_word!(game.hud_item_index = 0 as u16);
            set_ram_word!(game.samus_auto_cancel_hud_item_index = 0 as u16);
        }
    }
}
static kHdmaobjPreInstr_XrayFuncs: [Option<Func_Y_V>; 6] = unsafe {
    [
        Some(HdmaobjPreInstr_XrayFunc0_NoBeam as unsafe fn(u16, &mut Game) -> ()),
        Some(HdmaobjPreInstr_XrayFunc1_BeamWidening as unsafe fn(u16, &mut Game) -> ()),
        Some(HdmaobjPreInstr_XrayFunc2_FullBeam as unsafe fn(u16, &mut Game) -> ()),
        Some(HdmaobjPreInstr_XrayFunc3_DeactivateBeam as unsafe fn(u16, &mut Game) -> ()),
        Some(HdmaobjPreInstr_XrayFunc4_DeactivateBeam as unsafe fn(u16, &mut Game) -> ()),
        Some(HdmaobjPreInstr_XrayFunc5_DeactivateBeam as unsafe fn(u16, &mut Game) -> ()),
    ]
};
pub unsafe fn HdmaobjPreInstr_Xray(k: u16, game: &mut Game) {
    let mut v1: i16 = 0;
    v1 = 4096 as i16;
    if ram_word!(game, fx_type) != 36u16 {
        v1 = 0x2000 as i16;
        if CanXrayShowBlocks(game) {
            v1 = 0x4000 as i16;
            *((&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) as *mut u8 as *mut u16) =
                0x27 as u16;
            *((&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) as *mut u8 as *mut u16) =
                0x47 as u16;
            *((&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) as *mut u8 as *mut u16) =
                0x87 as u16;
        }
    }
    update_ram_word!(game.fx_layer_blending_config_c, |fresh138| (fresh138)
        | ((v1) as u16));
    kHdmaobjPreInstr_XrayFuncs[ram_word!(game, demo_input_pre_instr) as usize]
        .expect("non-null function pointer")(
        (2u16).wrapping_mul(ram_word!(game, demo_input_pre_instr)),
        game,
    );
}
pub unsafe fn SpawnPowerBombExplosion(game: &mut Game) {
    if ram_word!(game, time_is_frozen_flag) != 0 {
        set_ram_word!(game.power_bomb_explosion_status = 0x4000 as u16);
    } else {
        QueueSfx1_Max15(1 as u16, game);
        set_ram_word!(game.power_bomb_explosion_status = 0x8000 as u16);
        static unk_888ABA: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x40 as u8, 0x28 as u8, 0x8ace as VoidP);
        static unk_888AC2: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x40 as u8, 0x29 as u8, 0x8b80 as VoidP);
        SpawnHdmaObject(
            0x88 as u8,
            &raw const unk_888ABA as *const ::core::ffi::c_void,
            game,
        );
        SpawnHdmaObject(
            0x88 as u8,
            &raw const unk_888AC2 as *const ::core::ffi::c_void,
            game,
        );
    };
}
static kPowerBombPreExplosionRadiusSpeed: u16 = 0x3000 as u16;
static kPowerBombPreExplosionRadiusAccel: u16 = 0x80 as u16;
static g_word_888B96: u16 = 3 as u16;
static kPowerBombExplosionRadiusSpeed: u16 = 0 as u16;
static kPowerBombExplosionRadiusAccel: u16 = 0x30 as u16;
pub unsafe fn Hdmaobj_PreExplodeWhite(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0xc606) = -(1 as i32) as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0xc607) = 0 as u8;
    set_ram_word!(game.power_bomb_pre_explosion_flash_radius = 1024 as u16);
    set_ram_word!(game.power_bomb_pre_explosion_radius_speed = kPowerBombPreExplosionRadiusSpeed);
}
pub unsafe fn Hdmaobj_PreExplodeYellow(game: &mut Game) {
    set_ram_word!(game.pre_scaled_power_bomb_explosion_shape_def_ptr = addr_byte_889F06 as u16);
}
pub unsafe fn Hdmaobj_ExplodeYellow(game: &mut Game) {
    set_ram_word!(game.power_bomb_explosion_radius = 1024 as u16);
    set_ram_word!(game.power_bomb_pre_explosion_radius_speed = kPowerBombExplosionRadiusSpeed);
}
pub unsafe fn Hdmaobj_ExplodeWhite(game: &mut Game) {
    set_ram_word!(
        game.pre_scaled_power_bomb_explosion_shape_def_ptr =
            addr_kPowerBombExplosionShapeDef0 as u16
    );
}
pub unsafe fn Hdmaobj_CleanUpTryCrystalFlash(v0: u16, game: &mut Game) {
    if ram_word!(game, samus_x_pos) != ram_word!(game, power_bomb_explosion_x_pos)
        || ram_word!(game, samus_y_pos) != ram_word!(game, power_bomb_explosion_y_pos)
        || Hdmaobj_CrystalFlash(game) as i32 & 1 as i32 != 0
    {
        set_ram_word!(game.power_bomb_flag = 0 as u16);
    }
    set_ram_word!(game.power_bomb_explosion_status = 0 as u16);
    let v1: i32 = v0 as i32 >> 1 as i32;
    set_ram_word!(game.hdma_object_channels_bitmask[v1] = 0 as u16);
    set_ram_word!(game.hdma_object_channels_bitmask[(v1 + 1i32)] = 0 as u16);
    set_ram_word!(game.power_bomb_pre_explosion_flash_radius = 0 as u16);
    set_ram_word!(game.power_bomb_explosion_radius = 0 as u16);
    CallSomeSamusCode(0x1e as u16, game);
}
pub unsafe fn HdmaobjPreInstr_PowerBombExplode_SetWindowConf(k: u16, game: &mut Game) {
    update_ram_word!(game.fx_layer_blending_config_c, |fresh129| (fresh129)
        | (0x8000u16));
}
pub unsafe fn HdmaobjPreInstr_PowerBombExplode_Stage5_Afterglow(k: u16, game: &mut Game) {
    if (ram_word!(game, power_bomb_explosion_status) as i32 & 0x8000 as i32) != 0i32 {
        let v1: i32 = k as u8 as i32 >> 1 as i32;
        if (update_ram_word!(game.hdma_object_timers[v1], wrapping_sub(1), reread) as i32
            & 0x8000 as i32)
            != 0i32
        {
            let ref mut fresh126 = *((&raw mut game.g_ram as *mut u8).offset(0x1938) as *mut u16
                as *mut u8)
                .offset(k as u8 as i32 as isize);
            let fresh127 = *fresh126;
            *fresh126 = (*fresh126).wrapping_sub(1);
            if fresh127 == 1u8 {
                set_ram_word!(game.hdma_object_instruction_timers[v1] = 1 as u16);
                update_ram_word!(
                    game.hdma_object_instruction_list_pointers[v1],
                    wrapping_add(2u16)
                );
            } else {
                if (*(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) as i32 & 0x1f as i32)
                    != 0i32
                {
                    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) =
                        ((*(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) as i32
                            & 0x1f as i32)
                            - 1 as i32
                            | 0x20 as i32) as u8;
                }
                if (*(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) as i32 & 0x1f as i32)
                    != 0i32
                {
                    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) =
                        ((*(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) as i32
                            & 0x1f as i32)
                            - 1 as i32
                            | 0x40 as i32) as u8;
                }
                if (*(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) as i32 & 0x1f as i32)
                    != 0i32
                {
                    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) =
                        ((*(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) as i32
                            & 0x1f as i32)
                            - 1 as i32
                            | 0x80 as i32) as u8;
                }
                *((&raw mut game.g_ram as *mut u8).offset(0x1908) as *mut u16 as *mut u8)
                    .offset(k as u8 as i32 as isize) = g_word_888B96 as u8;
            }
        }
    }
}
pub unsafe fn CalculatePowerBombHdma_LeftOfScreen(mut k: u16, mut j: *const u8, game: &mut Game) {
    loop {
        let w: u8 = *j;
        let c: u8 = ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as u8;
        if u8::overflowing_add(w, c).1 {
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc506)
                .offset(k as isize) = (w).wrapping_add(c);
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc406)
                .offset(k as isize) = 0 as u8;
        } else {
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc506)
                .offset(k as isize) = 0 as u8;
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc406)
                .offset(k as isize) = 1 as u8;
        }
        j = j.offset(1);
        k = k.wrapping_add(1);
        if (k == 192u16) {
            break;
        }
    }
}
pub unsafe fn CalculatePowerBombHdma_OnScreen(mut k: u16, mut j: *const u8, game: &mut Game) {
    loop {
        let w: u8 = *j;
        if w == 0 {
            break;
        }
        let c: u8 = ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as u8;
        let right: u8 = (if u8::overflowing_add(c, w).1 as i32 != 0 {
            255 as i32
        } else {
            c as i32 + w as i32
        }) as u8;
        let left: u8 = (if c >= w {
            c as i32 - w as i32
        } else {
            0 as i32
        }) as u8;
        *(&raw mut game.g_ram as *mut u8)
            .offset(0xc406)
            .offset(k as isize) = left;
        *(&raw mut game.g_ram as *mut u8)
            .offset(0xc506)
            .offset(k as isize) = right;
        j = j.offset(1);
        k = k.wrapping_add(1);
        if (k == 192u16) {
            break;
        }
    }
}
pub unsafe fn CalculatePowerBombHdma_RightOfScreen(mut k: u16, mut j: *const u8, game: &mut Game) {
    loop {
        let w: u8 = *j;
        if (ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as u8) < w {
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc406)
                .offset(k as isize) =
                ((ram_word!(game, power_bomb_explosion_x_pos_plus_0x100)) as u8).wrapping_sub(w);
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc506)
                .offset(k as isize) = -(1 as i32) as u8;
        } else {
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc406)
                .offset(k as isize) = -(1 as i32) as u8;
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc506)
                .offset(k as isize) = -(2 as i32) as u8;
        }
        j = j.offset(1);
        k = k.wrapping_add(1);
        if (k == 192u16) {
            break;
        }
    }
}
pub unsafe fn CalculatePowerBombHdmaObjectTablePtrs(k: u16, game: &mut Game) {
    let mut v1: u16 = 0;
    if ((ram_word!(game, power_bomb_explosion_x_pos) as i32 - ram_word!(game, layer1_x_pos) as i32
        + 256 as i32) as u16)
        >= 0x300u16
        || {
            set_ram_word!(
                game.power_bomb_explosion_x_pos_plus_0x100 =
                    (ram_word!(game, power_bomb_explosion_x_pos) as i32
                        - ram_word!(game, layer1_x_pos) as i32
                        + 256 as i32) as u16
            );
            v1 = (ram_word!(game, power_bomb_explosion_y_pos) as i32
                - ram_word!(game, layer1_y_pos) as i32
                + 256 as i32) as u16;
            v1 >= 0x300u16
        }
    {
        v1 = 0 as u16;
    }
    set_ram_word!(
        game.power_bomb_explosion_y_pos_rsub_0x1ff =
            ((v1 as i32 ^ 0x3ff as i32) - 256 as i32) as u16
    );
    if (ram_word!(game, power_bomb_explosion_radius) as i32 & 0xff00 as i32) == 0i32 {
        set_ram_word!(game.power_bomb_explosion_y_pos_rsub_0x1ff = 0 as u16);
    }
    *((&raw mut game.g_ram as *mut u8).offset(0x18d8) as *mut u16)
        .offset(((k as i32 >> 1 as i32) + 0 as i32) as isize) =
        (3 as i32 * ram_word!(game, power_bomb_explosion_y_pos_rsub_0x1ff) as i32
            + addr_kIndirectHdmaTable_PowerBombExplodeLeft as i32) as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x18d8) as *mut u16)
        .offset(((k as i32 >> 1 as i32) + 1 as i32) as isize) =
        (3 as i32 * ram_word!(game, power_bomb_explosion_y_pos_rsub_0x1ff) as i32
            + addr_kIndirectHdmaTable_PowerBombExplodeRight as i32) as u16;
}

pub unsafe fn CalculatePowerBombHdmaScaled_LeftOfScreen(
    mut k: u16,
    mut j: u16,
    multval: u8,
    game: &mut Game,
) -> u16 {
    let mut v2: i8 = 0;
    let mut v6: u8 = 0;
    let mut v7: u8 = 0;
    loop {
        let r20: u8 = (Mult8x8(
            multval,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const u8 as *mut u8)
                .offset((j as u8 as i32 + 32 as i32) as isize),
        ) as i32
            >> 8 as i32) as u8;
        v2 = ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as i8;
        let Reg: u8 = (Mult8x8(
            multval,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const u8 as *mut u8)
                .offset(j as u8 as isize),
        ) as i32
            >> 8 as i32) as u8;
        let v4: bool = u8::overflowing_add(Reg, v2 as u8).1;
        let v5: u8 = (Reg).wrapping_add((v2) as u8);
        if v4 {
            v6 = v5;
            v7 = 0 as u8;
        } else {
            v6 = 0 as u8;
            v7 = -(1 as i32) as u8;
        }
        loop {
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc406)
                .offset(k as u8 as isize) = v7;
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc506)
                .offset(k as u8 as isize) = v6;
            if (k as u8) == r20 {
                break;
            }
            *(&raw mut k as *mut u8).offset(0) = ((k) as u8).wrapping_sub(1u8);
        }
        *(&raw mut j as *mut u8).offset(0) = ((j) as u8).wrapping_add(1u8);
        if ((j as i32 & 0x80 as i32) != 0i32) {
            break;
        }
    }
    game.k_out = *(&raw mut k as *mut u8).offset(0) as u16;
    ((v6 as i32) << 8 as i32 | v7 as i32) as u16
}
pub unsafe fn CalculatePowerBombHdmaScaled_OnScreen(
    mut k: u16,
    mut j: u16,
    multval: u8,
    game: &mut Game,
) -> u16 {
    let mut v2: i8 = 0;
    let mut v5: i8 = 0;
    let mut v6: u8 = 0;
    let mut v9: u8 = 0;
    loop {
        let r20: u8 = (Mult8x8(
            multval,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const u8 as *mut u8)
                .offset((j as u8 as i32 + 32 as i32) as isize),
        ) as i32
            >> 8 as i32) as u8;
        v2 = ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as i8;
        let Reg: u8 = (Mult8x8(
            multval,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const u8 as *mut u8)
                .offset(j as u8 as isize),
        ) as i32
            >> 8 as i32) as u8;
        let mut v4: bool = u8::overflowing_add(Reg, v2 as u8).1;
        v5 = ((Reg) as i8).wrapping_add(v2);
        if v4 {
            v5 = -(1 as i32) as i8;
        }
        v6 = v5 as u8;
        let v7: u8 = ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as u8;
        let v8: u8 = Reg;
        v4 = v7 < v8;
        v9 = (v7).wrapping_sub(v8);
        if v4 {
            v9 = 0 as u8;
        }
        loop {
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc406)
                .offset(k as u8 as isize) = v9;
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc506)
                .offset(k as u8 as isize) = v6;
            if (k as u8) == r20 {
                break;
            }
            *(&raw mut k as *mut u8).offset(0) = ((k) as u8).wrapping_sub(1u8);
        }
        *(&raw mut j as *mut u8).offset(0) = ((j) as u8).wrapping_add(1u8);
        if ((j as i32 & 0x80 as i32) != 0i32) {
            break;
        }
    }
    game.k_out = *(&raw mut k as *mut u8).offset(0) as u16;
    ((v6 as i32) << 8 as i32 | v9 as i32) as u16
}
pub unsafe fn CalculatePowerBombHdmaScaled_RightOfScreen(
    mut k: u16,
    mut j: u16,
    multval: u8,
    game: &mut Game,
) -> u16 {
    let mut v5: i8 = 0;
    let mut v6: i8 = 0;
    let mut v7: i8 = 0;
    let mut v10: i8 = 0;
    let mut v8: u8 = 0;
    let mut v9: u8 = 0;
    loop {
        let r20: u8 = (Mult8x8(
            multval,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const u8 as *mut u8)
                .offset((j as u8 as i32 + 32 as i32) as isize),
        ) as i32
            >> 8 as i32) as u8;
        let v2: u8 = ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as u8;
        let Reg: u8 = (Mult8x8(
            multval,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const u8 as *mut u8)
                .offset(j as u8 as isize),
        ) as i32
            >> 8 as i32) as u8;
        let v4: bool = v2 < Reg;
        v5 = ((v2) as i8).wrapping_sub((Reg) as i8);
        if v4 {
            v6 = v5;
            v7 = -(1 as i32) as i8;
        } else {
            v6 = -(1 as i32) as i8;
            v7 = 0 as i8;
        }
        v10 = v7;
        v8 = v6 as u8;
        v9 = v10 as u8;
        loop {
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc406)
                .offset(k as u8 as isize) = v8;
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc506)
                .offset(k as u8 as isize) = v9;
            if (k as u8) == r20 {
                break;
            }
            *(&raw mut k as *mut u8).offset(0) = ((k) as u8).wrapping_sub(1u8);
        }
        *(&raw mut j as *mut u8).offset(0) = ((j) as u8).wrapping_add(1u8);
        if ((j as i32 & 0x80 as i32) != 0i32) {
            break;
        }
    }
    game.k_out = *(&raw mut k as *mut u8).offset(0) as u16;
    ((v9 as i32) << 8 as i32 | v8 as i32) as u16
}
pub unsafe fn HdmaobjPreInstr_PowerBombExplode_ExplosionYellow(k: u16, game: &mut Game) {
    let mut v2: u16 = 0;
    if (ram_word!(game, power_bomb_explosion_status) as i32 & 0x8000 as i32) == 0i32 {
        return;
    }
    CalculatePowerBombHdmaObjectTablePtrs(k, game);
    let kk: i32 = Mult8x8(
        ((ram_word!(game, power_bomb_explosion_radius) as i32 & 0xff00 as i32) >> 8 as i32) as u8,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a286)) as *const u8 as *mut u8).offset(0),
    ) as i32
        >> 8 as i32;
    if (ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as i32 & 0xff00 as i32) >> 8 as i32
        != 0
    {
        if ((ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as i32 & 0xff00 as i32)
            >> 8 as i32)
            == 1i32
        {
            v2 = CalculatePowerBombHdmaScaled_OnScreen(
                kk as u16,
                96 as u16,
                ((ram_word!(game, power_bomb_explosion_radius) as i32 & 0xff00 as i32) >> 8 as i32)
                    as u8,
                game,
            );
        } else {
            v2 = CalculatePowerBombHdmaScaled_RightOfScreen(
                kk as u16,
                96 as u16,
                ((ram_word!(game, power_bomb_explosion_radius) as i32 & 0xff00 as i32) >> 8 as i32)
                    as u8,
                game,
            );
        }
    } else {
        v2 = CalculatePowerBombHdmaScaled_LeftOfScreen(
            kk as u16,
            96 as u16,
            ((ram_word!(game, power_bomb_explosion_radius) as i32 & 0xff00 as i32) >> 8 as i32)
                as u8,
            game,
        );
    }
    let mut i: i32 = game.k_out as i32;
    loop {
        *(&raw mut game.g_ram as *mut u8)
            .offset(0xc406)
            .offset(i as isize) = v2 as u8;
        *(&raw mut game.g_ram as *mut u8)
            .offset(0xc506)
            .offset(i as isize) = ((v2 as i32 & 0xff00 as i32) >> 8 as i32) as u8;
        i -= 1;
        if ((i & 0x80 as i32) != 0i32) {
            break;
        }
    }
    let mut v3: i32 = kk + 1 as i32;
    while v3 != 0xc0i32 {
        *(&raw mut game.g_ram as *mut u8)
            .offset(0xc406)
            .offset(v3 as isize) = 255 as u8;
        *(&raw mut game.g_ram as *mut u8)
            .offset(0xc506)
            .offset(v3 as isize) = 0 as u8;
        v3 += 1;
    }
    let t: i32 = 3 as i32
        * ((ram_word!(game, power_bomb_explosion_radius) as i32 & 0xff00 as i32)
            >> 8 as i32
            >> 3 as i32);
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const u8 as *mut u8)
            .offset((t + 0 as i32) as isize))
            | (0x20u8);
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const u8 as *mut u8)
            .offset((t + 1 as i32) as isize))
            | (0x40u8);
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const u8 as *mut u8)
            .offset((t + 2 as i32) as isize))
            | (0x80u8);
    update_ram_word!(
        game.power_bomb_explosion_radius,
        wrapping_add(ram_word!(game, power_bomb_pre_explosion_radius_speed))
    );
    if ram_word!(game, power_bomb_explosion_radius) < 0x8600u16 {
        update_ram_word!(
            game.power_bomb_pre_explosion_radius_speed,
            wrapping_add(kPowerBombExplosionRadiusAccel)
        );
    } else {
        let v7: i32 = k as i32 >> 1 as i32;
        set_ram_word!(game.hdma_object_instruction_timers[v7] = 1 as u16);
        update_ram_word!(
            game.hdma_object_instruction_list_pointers[v7],
            wrapping_add(2u16)
        );
        set_ram_word!(game.hdma_object_timers[v7] = 0 as u16);
    };
}
pub unsafe fn HdmaobjPreInstr_PowerBombExplode_ExplosionWhite(k: u16, game: &mut Game) {
    if (ram_word!(game, power_bomb_explosion_status) as i32 & 0x8000 as i32) == 0i32 {
        return;
    }
    CalculatePowerBombHdmaObjectTablePtrs(k, game);
    let v1: *const u8 = RomPtr_88(
        ram_word!(game, pre_scaled_power_bomb_explosion_shape_def_ptr),
        game,
    );
    if (ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as i32 & 0xff00 as i32) >> 8 as i32
        != 0
    {
        if ((ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as i32 & 0xff00 as i32)
            >> 8 as i32)
            == 1i32
        {
            CalculatePowerBombHdma_OnScreen(0 as u16, v1, game);
        } else {
            CalculatePowerBombHdma_RightOfScreen(0 as u16, v1, game);
        }
    } else {
        CalculatePowerBombHdma_LeftOfScreen(0 as u16, v1, game);
    }
    let t: i32 = 3 as i32
        * ((ram_word!(game, power_bomb_explosion_radius) as i32 & 0xff00 as i32)
            >> 8 as i32
            >> 3 as i32);
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const u8 as *mut u8)
            .offset(t as isize))
            | (0x20u8);
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const u8 as *mut u8)
            .offset((t + 1 as i32) as isize))
            | (0x40u8);
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const u8 as *mut u8)
            .offset((t + 2 as i32) as isize))
            | (0x80u8);
    update_ram_word!(
        game.pre_scaled_power_bomb_explosion_shape_def_ptr,
        wrapping_add(192u16)
    );
    if ram_word!(game, pre_scaled_power_bomb_explosion_shape_def_ptr) as i32
        == addr_byte_889F06 as i32
    {
        let v3: i32 = k as i32 >> 1 as i32;
        set_ram_word!(game.hdma_object_instruction_timers[v3] = 1 as u16);
        update_ram_word!(
            game.hdma_object_instruction_list_pointers[v3],
            wrapping_add(2u16)
        );
        set_ram_word!(game.hdma_object_timers[v3] = 0 as u16);
        set_ram_word!(game.hdma_object_D[v3] = 32 as u16);
    }
    if (ram_word!(game, power_bomb_pre_explosion_radius_speed) as i32
        + ram_word!(game, power_bomb_explosion_radius) as i32)
        < 0x10000i32
    {
        update_ram_word!(
            game.power_bomb_explosion_radius,
            wrapping_add(ram_word!(game, power_bomb_pre_explosion_radius_speed))
        );
        update_ram_word!(
            game.power_bomb_pre_explosion_radius_speed,
            wrapping_add(kPowerBombExplosionRadiusAccel)
        );
    }
}
pub unsafe fn CalculatePowerBombHdmaTablePointers(v0: u16, game: &mut Game) {
    let mut v1: u16 = 0;
    if (ram_word!(game, power_bomb_explosion_status) as i32 & 0x8000 as i32) != 0i32 {
        if ((ram_word!(game, power_bomb_explosion_x_pos) as i32
            - ram_word!(game, layer1_x_pos) as i32
            + 256 as i32) as u16)
            >= 0x300u16
            || {
                set_ram_word!(
                    game.power_bomb_explosion_x_pos_plus_0x100 =
                        (ram_word!(game, power_bomb_explosion_x_pos) as i32
                            - ram_word!(game, layer1_x_pos) as i32
                            + 256 as i32) as u16
                );
                v1 = (ram_word!(game, power_bomb_explosion_y_pos) as i32
                    - ram_word!(game, layer1_y_pos) as i32
                    + 256 as i32) as u16;
                v1 >= 0x300u16
            }
        {
            v1 = 0 as u16;
        }
        set_ram_word!(
            game.power_bomb_explosion_y_pos_rsub_0x1ff =
                ((v1 as i32 ^ 0x3ff as i32) - 256 as i32) as u16
        );
        if (ram_word!(game, power_bomb_pre_explosion_flash_radius) as i32 & 0xff00 as i32) == 0i32 {
            set_ram_word!(game.power_bomb_explosion_y_pos_rsub_0x1ff = 0 as u16);
        }
        let v2: i32 = v0 as i32 >> 1 as i32;
        set_ram_word!(
            game.hdma_object_table_pointers[v2] =
                (3 as i32 * ram_word!(game, power_bomb_explosion_y_pos_rsub_0x1ff) as i32
                    + addr_kIndirectHdmaTable_PowerBombExplodeLeft as i32) as u16
        );
        set_ram_word!(
            game.hdma_object_table_pointers[(v2 + 1i32)] =
                (3 as i32 * ram_word!(game, power_bomb_explosion_y_pos_rsub_0x1ff) as i32
                    + addr_kIndirectHdmaTable_PowerBombExplodeRight as i32) as u16
        );
    }
}
pub unsafe fn HdmaobjPreInstr_PowerBombExplode_PreExplosionWhite(k: u16, game: &mut Game) {
    if (ram_word!(game, power_bomb_explosion_status) as i32 & 0x8000 as i32) == 0i32 {
        return;
    }
    CalculatePowerBombHdmaTablePointers(k, game);
    let v1: u16 = 96 as u16;
    let mut v2: u16 = 0;
    let kk: i32 = Mult8x8(
        *((&raw mut game.g_ram as *mut u8).offset(0xcec) as *mut u16 as *mut u8).offset(
            (::core::mem::size_of::<u16>())
                .wrapping_div(::core::mem::size_of::<u8>())
                .wrapping_sub(1_usize) as isize,
        ),
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a286)) as *const u8 as *mut u8).offset(0),
    ) as i32
        >> 8 as i32;
    if *((&raw mut game.g_ram as *mut u8).offset(0xce6) as *mut u16 as *mut u8).offset(
        (::core::mem::size_of::<u16>())
            .wrapping_div(::core::mem::size_of::<u8>())
            .wrapping_sub(1_usize) as isize,
    ) != 0
    {
        if *((&raw mut game.g_ram as *mut u8).offset(0xce6) as *mut u16 as *mut u8).offset(
            (::core::mem::size_of::<u16>())
                .wrapping_div(::core::mem::size_of::<u8>())
                .wrapping_sub(1_usize) as isize,
        ) == 1u8
        {
            v2 = CalculatePowerBombHdmaScaled_OnScreen(
                kk as u16,
                v1,
                *((&raw mut game.g_ram as *mut u8).offset(0xcec) as *mut u16 as *mut u8).offset(
                    (::core::mem::size_of::<u16>())
                        .wrapping_div(::core::mem::size_of::<u8>())
                        .wrapping_sub(1_usize) as isize,
                ),
                game,
            );
        } else {
            v2 = CalculatePowerBombHdmaScaled_RightOfScreen(
                kk as u16,
                v1,
                *((&raw mut game.g_ram as *mut u8).offset(0xcec) as *mut u16 as *mut u8).offset(
                    (::core::mem::size_of::<u16>())
                        .wrapping_div(::core::mem::size_of::<u8>())
                        .wrapping_sub(1_usize) as isize,
                ),
                game,
            );
        }
    } else {
        v2 = CalculatePowerBombHdmaScaled_LeftOfScreen(
            kk as u16,
            v1,
            *((&raw mut game.g_ram as *mut u8).offset(0xcec) as *mut u16 as *mut u8).offset(
                (::core::mem::size_of::<u16>())
                    .wrapping_div(::core::mem::size_of::<u8>())
                    .wrapping_sub(1_usize) as isize,
            ),
            game,
        );
    }
    let mut i: u8 = game.k_out as u8;
    loop {
        *(&raw mut game.g_ram as *mut u8)
            .offset(0xc406)
            .offset(i as isize) = v2 as u8;
        *(&raw mut game.g_ram as *mut u8)
            .offset(0xc506)
            .offset(i as isize) = *(&raw mut v2 as *mut u8).offset(
            (::core::mem::size_of::<u16>())
                .wrapping_div(::core::mem::size_of::<u8>())
                .wrapping_sub(1_usize) as isize,
        );
        i = i.wrapping_sub(1);
        if ((i as i32 & 0x80 as i32) != 0i32) {
            break;
        }
    }
    let mut v3: u8 = (kk + 1 as i32) as u8;
    while v3 != 0xc0u8 {
        *(&raw mut game.g_ram as *mut u8)
            .offset(0xc406)
            .offset(v3 as isize) = 255 as u8;
        *(&raw mut game.g_ram as *mut u8)
            .offset(0xc506)
            .offset(v3 as isize) = 0 as u8;
        v3 = v3.wrapping_add(1);
    }
    let t: i32 = *((&raw mut game.g_ram as *mut u8).offset(0xcec) as *mut u16 as *mut u8).offset(
        (::core::mem::size_of::<u16>())
            .wrapping_div(::core::mem::size_of::<u8>())
            .wrapping_sub(1_usize) as isize,
    ) as i32
        >> 3 as i32
        & 0xf as i32;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x889079)) as *const u8 as *mut u8)
            .offset((t * 3 as i32 + 0 as i32) as isize))
            | (0x20u8);
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x889079)) as *const u8 as *mut u8)
            .offset((t * 3 as i32 + 1 as i32) as isize))
            | (0x40u8);
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x889079)) as *const u8 as *mut u8)
            .offset((t * 3 as i32 + 2 as i32) as isize))
            | (0x80u8);
    update_ram_word!(
        game.power_bomb_pre_explosion_flash_radius,
        wrapping_add(ram_word!(game, power_bomb_pre_explosion_radius_speed))
    );
    if ram_word!(game, power_bomb_pre_explosion_flash_radius) < 0x9200u16 {
        update_ram_word!(
            game.power_bomb_pre_explosion_radius_speed,
            wrapping_sub(kPowerBombPreExplosionRadiusAccel)
        );
    } else {
        set_ram_word!(game.hdma_object_instruction_timers[(k as i32 >> 1i32)] = 1 as u16);
        update_ram_word!(
            game.hdma_object_instruction_list_pointers[(k as i32 >> 1 as i32)],
            wrapping_add(2u16)
        );
        set_ram_word!(game.hdma_object_timers[(k as i32 >> 1i32)] = 0 as u16);
    };
}
pub unsafe fn HdmaobjPreInstr_PowerBombExplode_PreExplosionYellow(k: u16, game: &mut Game) {
    if (ram_word!(game, power_bomb_explosion_status) as i32 & 0x8000 as i32) == 0i32 {
        return;
    }
    CalculatePowerBombHdmaTablePointers(k, game);
    let v1: *const u8 = RomPtr_88(
        ram_word!(game, pre_scaled_power_bomb_explosion_shape_def_ptr),
        game,
    );
    if (ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as i32 & 0xff00 as i32) >> 8 as i32
        != 0
    {
        if ((ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as i32 & 0xff00 as i32)
            >> 8 as i32)
            == 1i32
        {
            CalculatePowerBombHdma_OnScreen(0 as u16, v1, game);
        } else {
            CalculatePowerBombHdma_RightOfScreen(0 as u16, v1, game);
        }
    } else {
        CalculatePowerBombHdma_LeftOfScreen(0 as u16, v1, game);
    }
    let t: i32 = (ram_word!(game, power_bomb_pre_explosion_flash_radius) as i32 & 0xff00 as i32)
        >> 8 as i32
        >> 3 as i32
        & 0xf as i32;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x889079)) as *const u8 as *mut u8)
            .offset((3 as i32 * t) as isize))
            | (0x20u8);
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x889079)) as *const u8 as *mut u8)
            .offset((3 as i32 * t + 1 as i32) as isize))
            | (0x40u8);
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x889079)) as *const u8 as *mut u8)
            .offset((3 as i32 * t + 2 as i32) as isize))
            | (0x80u8);
    update_ram_word!(
        game.pre_scaled_power_bomb_explosion_shape_def_ptr,
        wrapping_add(192u16)
    );
    if ram_word!(game, pre_scaled_power_bomb_explosion_shape_def_ptr) == 0xa206u16 {
        set_ram_word!(game.hdma_object_instruction_timers[(k as i32 >> 1i32)] = 1 as u16);
        update_ram_word!(
            game.hdma_object_instruction_list_pointers[(k as i32 >> 1 as i32)],
            wrapping_add(2u16)
        );
        set_ram_word!(game.hdma_object_timers[(k as i32 >> 1i32)] = 0 as u16);
    }
    if (ram_word!(game, power_bomb_pre_explosion_radius_speed) as i32
        + ram_word!(game, power_bomb_pre_explosion_flash_radius) as i32)
        < 0x10000i32
    {
        update_ram_word!(
            game.power_bomb_pre_explosion_flash_radius,
            wrapping_add(ram_word!(game, power_bomb_pre_explosion_radius_speed))
        );
        update_ram_word!(
            game.power_bomb_pre_explosion_radius_speed,
            wrapping_sub(kPowerBombPreExplosionRadiusAccel)
        );
    }
}
pub unsafe fn SpawnCrystalFlashHdmaObjs(game: &mut Game) {
    set_ram_word!(game.power_bomb_explosion_status = ((0x888000i32) as u16));
    static unk_88A2B0: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x40 as u8, 0x28 as u8, 0xa2bd as VoidP);
    static unk_88A2B8: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x40 as u8, 0x29 as u8, 0xa32a as VoidP);
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88A2B0 as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88A2B8 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn CrystalFlashSetupPart1(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0xc606) = -(1 as i32) as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0xc607) = 0 as u8;
    set_ram_word!(game.power_bomb_pre_explosion_flash_radius = 1024 as u16);
    set_ram_word!(game.power_bomb_pre_explosion_radius_speed = kPowerBombPreExplosionRadiusSpeed);
    QueueSfx1_Max6(1 as u16, game);
}
pub unsafe fn CrystalFlashSetupPart2(game: &mut Game) {
    set_ram_word!(game.power_bomb_explosion_radius = 1024 as u16);
    set_ram_word!(game.power_bomb_pre_explosion_radius_speed = kPowerBombExplosionRadiusSpeed);
}
pub unsafe fn CrystalFlashCleanup(k: u16, game: &mut Game) {
    let v0: u16 = k;
    set_ram_word!(game.power_bomb_flag = 0 as u16);
    set_ram_word!(game.power_bomb_explosion_status = 0 as u16);
    let v1: i32 = v0 as i32 >> 1 as i32;
    set_ram_word!(game.hdma_object_channels_bitmask[v1] = 0 as u16);
    set_ram_word!(game.hdma_object_channels_bitmask[(v1 + 1i32)] = 0 as u16);
    set_ram_word!(game.power_bomb_pre_explosion_flash_radius = 0 as u16);
    set_ram_word!(game.power_bomb_explosion_radius = 0 as u16);
}
pub unsafe fn HdmaobjPreInstr_CrystalFlash_CustomLayerBlend(k: u16, game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x60) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x61) = 8 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x62) = 0x80 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6e) = 2 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x71) = 51 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6c) = 0 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x6d) = 4 as u8;
}
pub unsafe fn HdmaobjPreInstr_CrystalFlash_Stage2_AfterGlow(k: u16, game: &mut Game) {
    if (ram_word!(game, power_bomb_explosion_status) as i32 & 0x8000 as i32) != 0i32 {
        let v1: i32 = k as i32 >> 1 as i32;
        if (update_ram_word!(game.hdma_object_timers[v1], wrapping_sub(1), reread) as i32
            & 0x8000 as i32)
            != 0i32
        {
            if ((*(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) as i32
                | *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) as i32
                | *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) as i32)
                & 0x1f as i32)
                != 0i32
            {
                if (*(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) as i32 & 0x1f as i32)
                    != 0i32
                {
                    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) =
                        ((*(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) as i32
                            & 0x1f as i32)
                            - 1 as i32
                            | 0x20 as i32) as u8;
                }
                if (*(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) as i32 & 0x1f as i32)
                    != 0i32
                {
                    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) =
                        ((*(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) as i32
                            & 0x1f as i32)
                            - 1 as i32
                            | 0x40 as i32) as u8;
                }
                if (*(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) as i32 & 0x1f as i32)
                    != 0i32
                {
                    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) =
                        ((*(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) as i32
                            & 0x1f as i32)
                            - 1 as i32
                            | 0x80 as i32) as u8;
                }
                *((&raw mut game.g_ram as *mut u8).offset(0x1908) as *mut u16 as *mut u8)
                    .offset(k as i32 as isize) = g_word_888B96 as u8;
            } else {
                set_ram_word!(game.hdma_object_instruction_timers[v1] = 1 as u16);
                update_ram_word!(
                    game.hdma_object_instruction_list_pointers[v1],
                    wrapping_add(2u16)
                );
            }
        }
    }
}
pub unsafe fn CalculateCrystalFlashHdmaObjectTablePtrs(k: u16, game: &mut Game) {
    let mut v1: u16 = 0;
    if ((ram_word!(game, power_bomb_explosion_x_pos) as i32 - ram_word!(game, layer1_x_pos) as i32
        + 256 as i32) as u16)
        >= 0x300u16
        || {
            set_ram_word!(
                game.power_bomb_explosion_x_pos_plus_0x100 =
                    (ram_word!(game, power_bomb_explosion_x_pos) as i32
                        - ram_word!(game, layer1_x_pos) as i32
                        + 256 as i32) as u16
            );
            v1 = (ram_word!(game, power_bomb_explosion_y_pos) as i32
                - ram_word!(game, layer1_y_pos) as i32
                + 256 as i32) as u16;
            v1 >= 0x300u16
        }
    {
        v1 = 0 as u16;
    }
    set_ram_word!(
        game.power_bomb_explosion_y_pos_rsub_0x1ff =
            ((v1 as i32 ^ 0x3ff as i32) - 256 as i32) as u16
    );
    if (ram_word!(game, power_bomb_explosion_radius) as i32 & 0xff00 as i32) == 0i32 {
        set_ram_word!(game.power_bomb_explosion_y_pos_rsub_0x1ff = 0 as u16);
    }
    set_ram_word!(
        game.hdma_object_table_pointers[(k as i32 >> 1i32)] =
            (3 as i32 * ram_word!(game, power_bomb_explosion_y_pos_rsub_0x1ff) as i32
                + addr_kIndirectHdmaTable_PowerBombExplodeLeft as i32) as u16
    );
    *((&raw mut game.g_ram as *mut u8).offset(0x18d8) as *mut u16)
        .offset(((k as i32 >> 1 as i32) + 1 as i32) as isize) =
        (3 as i32 * ram_word!(game, power_bomb_explosion_y_pos_rsub_0x1ff) as i32
            + addr_kIndirectHdmaTable_PowerBombExplodeRight as i32) as u16;
}
pub unsafe fn CalculateCrystalFlashHdmaDataTablesScaled_LeftOfScreen(
    mut k: u16,
    mut j: u16,
    game: &mut Game,
) -> u16 {
    let mut right: u8 = 0;
    let mut left: u8 = 0;
    loop {
        let r20: u8 = (Mult8x8(
            ((ram_word!(game, power_bomb_explosion_radius) as i32 & 0xff00 as i32) >> 8 as i32)
                as u8,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const u8 as *mut u8)
                .offset((j as u8 as i32 + 32 as i32) as isize),
        ) as i32
            >> 8 as i32) as u8;
        let w: u8 = (Mult8x8(
            ((ram_word!(game, power_bomb_explosion_radius) as i32 & 0xff00 as i32) >> 8 as i32)
                as u8,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const u8 as *mut u8)
                .offset(j as u8 as isize),
        ) as i32
            >> 8 as i32) as u8;
        let c: u8 = ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as u8;
        left = (if (w as i32 + c as i32) >= 256i32 {
            0 as i32
        } else {
            255 as i32
        }) as u8;
        right = (if (w as i32 + c as i32) >= 256i32 {
            w as i32 + c as i32
        } else {
            0 as i32
        }) as u8;
        loop {
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc406)
                .offset(k as u8 as isize) = left;
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc506)
                .offset(k as u8 as isize) = right;
            if (k as u8) == r20 {
                break;
            }
            k = k.wrapping_sub(1);
        }
        j = j.wrapping_add(1);
        if ((j as i32 & 0x80 as i32) != 0i32) {
            break;
        }
    }
    game.k_out = *(&raw mut k as *mut u8).offset(0) as u16;
    ((right as i32) << 8 as i32 | left as i32) as u16
}
pub unsafe fn CalculateCrystalFlashHdmaDataTablesScaled_OnScreen(
    mut k: u16,
    mut j: u16,
    game: &mut Game,
) -> u16 {
    let mut right: u8 = 0;
    let mut left: u8 = 0;
    loop {
        let r20: u8 = (Mult8x8(
            ((ram_word!(game, power_bomb_explosion_radius) as i32 & 0xff00 as i32) >> 8 as i32)
                as u8,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const u8 as *mut u8)
                .offset((j as u8 as i32 + 32 as i32) as isize),
        ) as i32
            >> 8 as i32) as u8;
        let w: u8 = (Mult8x8(
            ((ram_word!(game, power_bomb_explosion_radius) as i32 & 0xff00 as i32) >> 8 as i32)
                as u8,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const u8 as *mut u8)
                .offset(j as u8 as isize),
        ) as i32
            >> 8 as i32) as u8;
        let c: u8 = ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as u8;
        right = (if (c as i32 + w as i32) < 256i32 {
            c as i32 + w as i32
        } else {
            255 as i32
        }) as u8;
        left = (if (c as i32 - w as i32) >= 0i32 {
            c as i32 - w as i32
        } else {
            0 as i32
        }) as u8;
        loop {
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc406)
                .offset(k as u8 as isize) = left;
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc506)
                .offset(k as u8 as isize) = right;
            if (k as u8) == r20 {
                break;
            }
            k = k.wrapping_sub(1);
        }
        j = j.wrapping_add(1);
        if ((j as i32 & 0x80 as i32) != 0i32) {
            break;
        }
    }
    game.k_out = *(&raw mut k as *mut u8).offset(0) as u16;
    ((right as i32) << 8 as i32 | left as i32) as u16
}
pub unsafe fn CalculateCrystalFlashHdmaDataTablesScaled_RightOfScreen(
    mut k: u16,
    mut j: u16,
    game: &mut Game,
) -> u16 {
    let mut left: u8 = 0;
    let mut right: u8 = 0;
    loop {
        let r20: u8 = (Mult8x8(
            ((ram_word!(game, power_bomb_explosion_radius) as i32 & 0xff00 as i32) >> 8 as i32)
                as u8,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const u8 as *mut u8)
                .offset((j as u8 as i32 + 32 as i32) as isize),
        ) as i32
            >> 8 as i32) as u8;
        let w: u8 = (Mult8x8(
            ((ram_word!(game, power_bomb_explosion_radius) as i32 & 0xff00 as i32) >> 8 as i32)
                as u8,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a206)) as *const u8 as *mut u8)
                .offset(j as u8 as isize),
        ) as i32
            >> 8 as i32) as u8;
        let c: u8 = ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as u8;
        right = (if c < w { 255 as i32 } else { 0 as i32 }) as u8;
        left = (if c < w {
            c as i32 - w as i32
        } else {
            255 as i32
        }) as u8;
        loop {
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc406)
                .offset(k as u8 as isize) = left;
            *(&raw mut game.g_ram as *mut u8)
                .offset(0xc506)
                .offset(k as u8 as isize) = right;
            if (k as u8) == r20 {
                break;
            }
            k = k.wrapping_sub(1);
        }
        j = j.wrapping_add(1);
        if ((j as i32 & 0x80 as i32) != 0i32) {
            break;
        }
    }
    game.k_out = *(&raw mut k as *mut u8).offset(0) as u16;
    ((right as i32) << 8 as i32 | left as i32) as u16
}
pub unsafe fn HdmaobjPreInstr_CrystalFlash_Stage1_Explosion(k: u16, game: &mut Game) {
    if (ram_word!(game, power_bomb_explosion_status) as i32 & 0x8000 as i32) == 0i32 {
        return;
    }
    CalculateCrystalFlashHdmaObjectTablePtrs(k, game);
    let mut v2: u16 = 0;
    let kk: i32 = Mult8x8(
        ((ram_word!(game, power_bomb_explosion_radius) as i32 & 0xff00 as i32) >> 8 as i32) as u8,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a286)) as *const u8 as *mut u8).offset(0),
    ) as i32
        >> 8 as i32;
    if (ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as i32 & 0xff00 as i32) >> 8 as i32
        != 0
    {
        if ((ram_word!(game, power_bomb_explosion_x_pos_plus_0x100) as i32 & 0xff00 as i32)
            >> 8 as i32)
            == 1i32
        {
            v2 = CalculateCrystalFlashHdmaDataTablesScaled_OnScreen(kk as u16, 96 as u16, game);
        } else {
            v2 =
                CalculateCrystalFlashHdmaDataTablesScaled_RightOfScreen(kk as u16, 96 as u16, game);
        }
    } else {
        v2 = CalculateCrystalFlashHdmaDataTablesScaled_LeftOfScreen(kk as u16, 96 as u16, game);
    }
    let mut i: i32 = game.k_out as i32;
    loop {
        *(&raw mut game.g_ram as *mut u8)
            .offset(0xc406)
            .offset(i as u8 as isize) = v2 as u8;
        *(&raw mut game.g_ram as *mut u8)
            .offset(0xc506)
            .offset(i as u8 as isize) = ((v2 as i32 & 0xff00 as i32) >> 8 as i32) as u8;
        i -= 1;
        if ((i & 0x80 as i32) != 0i32) {
            break;
        }
    }
    let mut v3: u8 = (kk + 1 as i32) as u8;
    while v3 != 0xc0u8 {
        *(&raw mut game.g_ram as *mut u8)
            .offset(0xc406)
            .offset(v3 as isize) = 255 as u8;
        *(&raw mut game.g_ram as *mut u8)
            .offset(0xc506)
            .offset(v3 as isize) = 0 as u8;
        v3 = v3.wrapping_add(1);
    }
    let pos: i32 = (*((&raw mut game.g_ram as *mut u8).offset(0xcea) as *mut u16 as *mut u8).offset(
        (::core::mem::size_of::<u16>())
            .wrapping_div(::core::mem::size_of::<u8>())
            .wrapping_sub(1_usize) as isize,
    ) as i32
        >> 3 as i32)
        * 3 as i32;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const u8 as *mut u8)
            .offset((pos + 0 as i32) as isize))
            | (0x20u8);
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const u8 as *mut u8)
            .offset((pos + 1 as i32) as isize))
            | (0x40u8);
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x888d85)) as *const u8 as *mut u8)
            .offset((pos + 2 as i32) as isize))
            | (0x80u8);
    update_ram_word!(
        game.power_bomb_explosion_radius,
        wrapping_add(ram_word!(game, power_bomb_pre_explosion_radius_speed))
    );
    if ram_word!(game, power_bomb_explosion_radius) < 0x2000u16 {
        update_ram_word!(
            game.power_bomb_pre_explosion_radius_speed,
            wrapping_add(kPowerBombExplosionRadiusAccel)
        );
    } else {
        set_ram_word!(game.hdma_object_instruction_timers[(k as i32 >> 1i32)] = 1 as u16);
        update_ram_word!(
            game.hdma_object_instruction_list_pointers[(k as i32 >> 1 as i32)],
            wrapping_add(2u16)
        );
        set_ram_word!(game.hdma_object_timers[(k as i32 >> 1i32)] = 0 as u16);
    };
}
pub unsafe fn FxTypeFunc_22_ScrollingSky(game: &mut Game) {
    set_ram_word!(game.fx_y_pos = 1248 as u16);
    set_ram_word!(game.fx_type = 6 as u16);
    static unk_88A62E: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x2 as u8, 0x12 as u8, 0xad63 as VoidP);
    static unk_88A636: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x42 as u8, 0x11 as u8, 0xad4e as VoidP);
    static unk_88A63E: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x42 as u8, 0xd as u8, 0xad39 as VoidP);
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88A62E as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88A636 as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88A63E as *const ::core::ffi::c_void,
        game,
    );
    HdmaobjPreInstr_FxType22_BG3Yscroll(0 as u16, game);
}
unsafe fn SetupSomeHdmaTablesBG3(
    r24: u16,
    r0: u16,
    r3: u16,
    r6: u16,
    r9: u16,
    game: &mut Game,
) -> u16 {
    let mut v1: i16 = 0;
    set_ram_word!(game.g_word_7E0598 = 32 as u16);
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00) as *mut u16) = 31 as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(1) as *mut u8 as *mut u16) = 0 as u16;
    let mut v0: u16 = 3 as u16;
    v1 = ((ram_word!(game, layer1_y_pos)) as i16).wrapping_add(32i16);
    let mut r18: u16 = (ram_word!(game, layer1_y_pos)).wrapping_add(32u16);
    let mut v2: u16 = 0 as u16;
    while (v1).wrapping_sub(
        (*((RomPtr_88 as unsafe fn(u16, &mut Game) -> *const u8)(r0, game).offset(v2 as isize)
            as *const u8 as *mut u16)) as i16,
    ) < 0i16
        || (v1).wrapping_sub(
            (*((RomPtr_88 as unsafe fn(u16, &mut Game) -> *const u8)(r9, game).offset(v2 as isize)
                as *const u8 as *mut u16)) as i16,
        ) >= 0i16
    {
        v2 = (v2).wrapping_add(6u16);
        if ((v2) as i16).wrapping_sub((r24) as i16) >= 0i16 {
            return v0;
        }
    }
    let mut v3: u16 = 0;
    if r18 as i32 - 1248 as i32 & 0x8000 as i32 != 0 {
        v3 = (r18) & (0xfu16);
    } else {
        v3 = (r18) & (0x1fu16);
    }
    let R22_: u16 = v3;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(3) as *mut u8 as *mut u16) =
        (*((RomPtr_88 as unsafe fn(u16, &mut Game) -> *const u8)(r3, game).offset(v2 as isize)
            as *const u8 as *mut u16))
            .wrapping_sub(v3);
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(4) as *mut u8 as *mut u16) =
        (R22_ as i32
            + *((RomPtr_88 as unsafe fn(u16, &mut Game) -> *const u8)(r6, game).offset(v2 as isize)
                as *const u8 as *mut u16) as i32
            - ram_word!(game, g_word_7E0598) as i32) as u16;
    loop {
        let r20: u16 = *(&raw mut game.g_ram as *mut u8)
            .offset(0x9c00)
            .offset(v0 as isize) as u16;
        r18 = (r18).wrapping_add(r20);
        v0 = (v0).wrapping_add(3u16);
        update_ram_word!(game.g_word_7E0598, wrapping_add(r20));
        if ram_word!(game, g_word_7E0598) as i32 - 224 as i32 & 0x8000 as i32 == 0 {
            break;
        }
        while ((r18) as i16).wrapping_sub(
            (*((RomPtr_88 as unsafe fn(u16, &mut Game) -> *const u8)(r0, game).offset(v2 as isize)
                as *const u8 as *mut u16)) as i16,
        ) < 0i16
            || ((r18) as i16).wrapping_sub(
                (*((RomPtr_88 as unsafe fn(u16, &mut Game) -> *const u8)(r9, game)
                    .offset(v2 as isize) as *const u8 as *mut u16)) as i16,
            ) >= 0i16
        {
            v2 = (v2).wrapping_add(6u16);
            if ((v2) as i16).wrapping_sub((r24) as i16) >= 0i16 {
                return v0;
            }
        }
        *((&raw mut game.g_ram as *mut u8)
            .offset(0x9c00)
            .offset(v0 as isize) as *mut u8 as *mut u16) =
            *((RomPtr_88 as unsafe fn(u16, &mut Game) -> *const u8)(r3, game).offset(v2 as isize)
                as *const u8 as *mut u16);
        *((&raw mut game.g_ram as *mut u8)
            .offset(0x9c00)
            .offset((v0 as i32 + 1 as i32) as isize) as *mut u8 as *mut u16) =
            (*((RomPtr_88 as unsafe fn(u16, &mut Game) -> *const u8)(r6, game).offset(v2 as isize)
                as *const u8 as *mut u16))
                .wrapping_sub(ram_word!(game, g_word_7E0598));
    }
    v0
}
pub unsafe fn HdmaobjPreInstr_FxType22_BG3Yscroll(mut k: u16, game: &mut Game) {
    DamageSamusInTopRow(game);
    k = SetupSomeHdmaTablesBG3(
        78 as u16,
        addr_word_88A8E8 as u16,
        (addr_word_88A8E8 as i32 + 2 as i32) as u16,
        (addr_word_88A8E8 as i32 + 4 as i32) as u16,
        (addr_word_88A8E8 as i32 + 6 as i32) as u16,
        game,
    );
    *((&raw mut game.g_ram as *mut u8)
        .offset(0x9c00)
        .offset(k as isize) as *mut u8 as *mut u16) = 0 as u16;
}
pub unsafe fn HdmaobjInstr_SetFlagB(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.hdma_object_B[(k as i32 >> 1i32)] = 1 as u16);
    hdp
}
static g_word_88C46E: [i16; 16] = [0, 1, 1, 0, 0, -(1), -(1), 0, 0, 1, 1, 0, 0, -(1), -(1), 0];
pub unsafe fn HdmaobjPreInstr_BG3Xscroll(k: u16, game: &mut Game) {
    let mut v8: i16 = 0;
    let mut v9: i16 = 0;
    if ram_word!(game, layer1_y_pos) as i32 - 1024 as i32 & 0x8000 as i32 != 0 {
        if (*(&raw mut game.g_ram as *mut u8).offset(0x5b5) as i32 & 1 as i32) == 0i32 {
            set_ram_word!(
                game.g_word_7E0596 =
                    (ram_word!(game, g_word_7E0596) as i32 + 2 as i32 & 0x1e as i32) as u16
            );
        }
        let mut v1: u16 = ram_word!(game, g_word_7E0596);
        let mut v2: u16 = 0 as u16;
        loop {
            set_ram_word!(
                game.g_word_7E9E80[(v2 as i32 >> 1i32)] =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0x88a938)) as *const u8
                        as *mut u16)
                        .offset((v1 as i32 >> 1 as i32) as isize)
            );
            v2 = (v2).wrapping_add(2u16);
            v1 = (v1).wrapping_add(2u16);
            if (((v1) as i16).wrapping_sub(32i16) >= 0i16) {
                break;
            }
        }
    } else {
        let v3: i32 = k as i32 >> 1 as i32;
        let ref mut fresh104 =
            *((&raw mut game.g_ram as *mut u8).offset(0x1920) as *mut u16).offset(v3 as isize);
        let fresh105 = *fresh104;
        *fresh104 = (*fresh104).wrapping_sub(1);
        if fresh105 == 1u16 {
            set_ram_word!(game.hdma_object_B[v3] = 6 as u16);
            set_ram_word!(
                game.hdma_object_A[v3] =
                    (ram_word!(game, hdma_object_A[v3]) as i32 + 2 as i32 & 0x1f as i32) as u16
            );
        }
        let mut v5: u16 = ram_word!(game, hdma_object_A[v3]);
        let mut i: i32 = 30 as i32;
        while i >= 0i32 {
            set_ram_word!(
                game.g_word_7E9E80[(v5 as i32 >> 1i32)] =
                    ((g_word_88C46E[(i >> 1 as i32) as usize]) as u16)
                        .wrapping_add(ram_word!(game, reg_BG1HOFS))
            );
            v5 = (v5 as i32 - 2 as i32 & 0x1f as i32) as u16;
            i -= 2 as i32;
        }
    }
    *(&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0 as *mut u16) = 0 as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9f00) as *mut u16) = 0 as u16;
    *(&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_2 as *mut u16) = 31 as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9f00).offset(2) as *mut u8 as *mut u16) = 31 as u16;
    let mut r18: u16 = 31 as u16;
    *(&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(1))
    .field_0 as *mut u16) = -(25088 as i32) as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9f00).offset(3) as *mut u8 as *mut u16) =
        -(25088 as i32) as u16;
    if ram_word!(game, layer1_y_pos) as i32 - 1024 as i32 & 0x8000 as i32 != 0 {
        *((&raw mut game.g_ram as *mut u8).offset(0x9f00).offset(2) as *mut u8 as *mut u16) =
            0 as u16;
    } else {
        r18 = HdmaFunc_A786(0x105 as u16, r18, 177 as u16, 0 as u16, game);
    }
    let mut v7: u16 = 5 as u16;
    r18 = HdmaFunc_A786(5 as u16, r18, -(24960 as i32) as u16, 128 as u16, game);
    v8 = (224i16).wrapping_sub((r18) as i16);
    while (v8).wrapping_sub(16i16) >= 0i16 {
        v8 = (v8).wrapping_sub(16i16);
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0)
            .offset(v7 as i32 as isize) as *mut u16) = 144 as u16;
        *((&raw mut game.g_ram as *mut u8)
            .offset(0x9f00)
            .offset(v7 as isize) as *mut u8 as *mut u16) = 144 as u16;
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_1)
            .offset(v7 as i32 as isize) as *mut u16) = -(24960 as i32) as u16;
        *((&raw mut game.g_ram as *mut u8)
            .offset(0x9f00)
            .offset((v7 as i32 + 1 as i32) as isize) as *mut u8 as *mut u16) =
            -(24960 as i32) as u16;
        v7 = (v7).wrapping_add(3u16);
    }
    v9 = v8;
    if v8 != 0 {
        v9 = (v8).wrapping_add(128i16);
    }
    *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0)
        .offset(v7 as i32 as isize) as *mut u16) = v9 as u16;
    *((&raw mut game.g_ram as *mut u8)
        .offset(0x9f00)
        .offset(v7 as isize) as *mut u8 as *mut u16) = v9 as u16;
    *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_1)
        .offset(v7 as i32 as isize) as *mut u16) = -(24960 as i32) as u16;
    *((&raw mut game.g_ram as *mut u8)
        .offset(0x9f00)
        .offset((v7 as i32 + 1 as i32) as isize) as *mut u8 as *mut u16) = -(24960 as i32) as u16;
    *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(1))
    .field_0)
        .offset(v7 as i32 as isize) as *mut u16) = 0 as u16;
    *((&raw mut game.g_ram as *mut u8)
        .offset(0x9f00)
        .offset((v7 as i32 + 3 as i32) as isize) as *mut u8 as *mut u16) = 0 as u16;
}
pub unsafe fn HdmaFunc_A786(k: u16, mut r18: u16, r20: u16, r22: u16, game: &mut Game) -> u16 {
    let mut v0: u16 = k;
    let mut v2: i16 = 0;
    let mut v1: u16 = (1216u16).wrapping_sub(ram_word!(game, layer1_y_pos));
    let mut r24: u16 = (1216u16).wrapping_sub(ram_word!(game, layer1_y_pos));
    if (1216i16).wrapping_sub((ram_word!(game, layer1_y_pos)) as i16) >= 0i16
        && ram_word!(game, layer1_y_pos) != 1216u16
    {
        if v1 as i32 - 128 as i32 & 0x8000 as i32 != 0 {
            r18 = (1216u16).wrapping_sub(ram_word!(game, layer1_y_pos));
        } else {
            r18 = (1216u16).wrapping_sub(ram_word!(game, layer1_y_pos));
            if v1 as i32 - 193 as i32 & 0x8000 as i32 == 0 {
                v1 = 193 as u16;
            }
            r18 = v1;
            r24 = v1;
            loop {
                v2 = ((v1) as i16).wrapping_sub(16i16);
                if v2 < 0i16 {
                    break;
                }
                r24 = v2 as u16;
                *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x9e00)
                    as *mut ExpandingSquareTransitionHdma)
                    .offset(0))
                .field_0)
                    .offset(v0 as i32 as isize) as *mut u16) = (r22) | (0x10u16);
                *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x9e00)
                    as *mut ExpandingSquareTransitionHdma)
                    .offset(0))
                .field_1)
                    .offset(v0 as i32 as isize) as *mut u16) = r20;
                v1 = r24;
                v0 = (v0).wrapping_add(3u16);
            }
        }
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0)
            .offset(v0 as i32 as isize) as *mut u16) = (r22) | (r24);
        *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_1)
            .offset(v0 as i32 as isize) as *mut u16) = r20;
    }
    r18
}
static unk_88A7EF: SpawnHdmaObject_Args =
    SpawnHdmaObject_Args::new(0x42 as u8, 0xf as u8, 0xad76 as VoidP);
static unk_88A80B: SpawnHdmaObject_Args =
    SpawnHdmaObject_Args::new(0x42 as u8, 0xf as u8, 0xad89 as VoidP);
pub unsafe fn FxTypeFunc_20(game: &mut Game) {
    let ref mut fresh47 = *(&raw mut game.g_ram as *mut u8).offset(0x91b);
    *fresh47 = (*fresh47) | (1u8);
    let ref mut fresh48 = *(&raw mut game.g_ram as *mut u8).offset(0x91c);
    *fresh48 = (*fresh48) | (1u8);
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88A7EF as *const ::core::ffi::c_void,
        game,
    );
    set_ram_word!(game.set_to_e0_by_scrolling_sky = 224 as u16);
    set_ram_word!(game.UNUSED_word_7E059C = 0 as u16);
}
pub unsafe fn RoomSetupAsm_ScrollingSkyOcean(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x59) = 74 as u8;
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88A80B as *const ::core::ffi::c_void,
        game,
    );
    set_ram_word!(game.set_to_e0_by_scrolling_sky = 224 as u16);
    set_ram_word!(game.UNUSED_word_7E059C = 0 as u16);
}
pub unsafe fn DamageSamusInTopRow(game: &mut Game) {
    if ((ram_word!(game, samus_y_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_y_radius)) as i16)
        < 0i16
        || ram_word!(game, samus_y_pos) as i32 - ram_word!(game, samus_y_radius) as i32 - 17 as i32
            & 0x8000 as i32
            != 0
    {
        set_ram_word!(game.samus_periodic_damage = 8 as u16);
    }
}
pub unsafe fn HdmaobjPreInstr_SkyLandBG2XscrollInner(k: u16, game: &mut Game) {
    let mut i: u16 = 0;
    let mut r24: u16 = 0 as u16;
    *(&raw mut game.g_ram as *mut u8).offset(0x59) = 74 as u8;
    let mut v1: u16 = 0 as u16;
    loop {
        let v3: *mut u16 = (&raw mut game.g_ram as *mut u8).offset(
            (*(game.g_rom.offset(crate::rtl::lorom_offset(0x88aec1)) as *const u8
                as *mut HdmaScrollEntry)
                .offset(v1 as isize))
            .hdma_data_table_entry as isize,
        ) as *mut u8 as *mut u16;
        AddToHiLo(
            v3.offset(1) as *mut u16,
            v3.offset(0) as *mut u16,
            ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x88aec1)) as *const u8
                as *mut HdmaScrollEntry)
                .offset(v1 as isize))
            .scroll_speed as u32)
                << 16 as i32
                | (*(game.g_rom.offset(crate::rtl::lorom_offset(0x88aec1)) as *const u8
                    as *mut HdmaScrollEntry)
                    .offset(v1 as isize))
                .scroll_subspeed as u32,
        );
        v1 = v1.wrapping_add(1);
        if (v1 as i32 * 8 as i32 - 184 as i32 & 0x8000 as i32 == 0) {
            break;
        }
    }
    set_ram_word!(game.scrolling_sky_bg2_hdma_data[44] = 0 as u16);
    set_ram_word!(game.scrolling_sky_bg2_hdma_data[45] = 0 as u16);
    *((&raw mut game.g_ram as *mut u8).offset(0x9f00) as *mut u16) = 31 as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9f00).offset(1) as *mut u8 as *mut u16) =
        1438 as u16;
    let mut r18: u16 = (ram_word!(game, layer1_y_pos)).wrapping_add(32u16);
    let r20: u16 = (ram_word!(game, layer1_y_pos)).wrapping_add(224u16);
    let mut v5: u16 = 0 as u16;
    let mut v6: u16 = 3 as u16;
    loop {
        while ((r18) as i16).wrapping_sub(
            ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x88aec1)) as *const u8
                as *mut HdmaScrollEntry)
                .offset(v5 as isize))
            .top_pos) as i16,
        ) >= 0i16
            && ((r18) as i16).wrapping_sub(
                ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x88aec1)) as *const u8
                    as *mut HdmaScrollEntry)
                    .offset((v5 as i32 + 1 as i32) as isize))
                .top_pos) as i16,
            ) < 0i16
        {
            let mut v8: u16 =
                ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x88aec1)) as *const u8
                    as *mut HdmaScrollEntry)
                    .offset((v5 as i32 + 1 as i32) as isize))
                .top_pos)
                    .wrapping_sub(r18);
            r24 = v8;
            if v8 as i32 - 128 as i32 & 0x8000 as i32 == 0 {
                *((&raw mut game.g_ram as *mut u8)
                    .offset(0x9f00)
                    .offset(v6 as isize) as *mut u8 as *mut u16) = 127 as u16;
                *((&raw mut game.g_ram as *mut u8)
                    .offset(0x9f00)
                    .offset((v6 as i32 + 1 as i32) as isize) as *mut u8
                    as *mut u16) = ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x88aec1))
                    as *const u8 as *mut HdmaScrollEntry)
                    .offset(v5 as isize))
                .hdma_data_table_entry)
                    .wrapping_add(2u16);
                v6 = (v6).wrapping_add(3u16);
                v8 = (r24).wrapping_sub(127u16);
            }
            *((&raw mut game.g_ram as *mut u8)
                .offset(0x9f00)
                .offset(v6 as isize) as *mut u8 as *mut u16) = v8;
            *((&raw mut game.g_ram as *mut u8)
                .offset(0x9f00)
                .offset((v6 as i32 + 1 as i32) as isize) as *mut u8 as *mut u16) =
                ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x88aec1)) as *const u8
                    as *mut HdmaScrollEntry)
                    .offset(v5 as isize))
                .hdma_data_table_entry)
                    .wrapping_add(2u16);
            r18 = (r18).wrapping_add(r24);
            v6 = (v6).wrapping_add(3u16);
            if r18 as i32 - r20 as i32 & 0x8000 as i32 == 0 {
                *((&raw mut game.g_ram as *mut u8)
                    .offset(0x9f00)
                    .offset((v6 as i32 + 3 as i32) as isize) as *mut u8
                    as *mut u16) = 0 as u16;
                return;
            }
        }
        v5 = v5.wrapping_add(1);
        if (v5 as i32 * 8 as i32 - 184 as i32 & 0x8000 as i32 == 0) {
            break;
        }
    }
    i = (1535u16).wrapping_sub(r18);
    loop {
        r24 = i;
        if i as i32 - 128 as i32 & 0x8000 as i32 != 0 {
            break;
        }
        *((&raw mut game.g_ram as *mut u8)
            .offset(0x9f00)
            .offset(v6 as isize) as *mut u8 as *mut u16) = 127 as u16;
        *((&raw mut game.g_ram as *mut u8)
            .offset(0x9f00)
            .offset((v6 as i32 + 1 as i32) as isize) as *mut u8 as *mut u16) = 181 as u16;
        v6 = (v6).wrapping_add(3u16);
        i = (r24).wrapping_sub(127u16);
    }
    *((&raw mut game.g_ram as *mut u8)
        .offset(0x9f00)
        .offset(v6 as isize) as *mut u8 as *mut u16) = i;
    *((&raw mut game.g_ram as *mut u8)
        .offset(0x9f00)
        .offset((v6 as i32 + 1 as i32) as isize) as *mut u8 as *mut u16) = 181 as u16;
    *((&raw mut game.g_ram as *mut u8)
        .offset(0x9f00)
        .offset((v6 as i32 + 3 as i32) as isize) as *mut u8 as *mut u16) = 0 as u16;
}
pub unsafe fn HdmaobjPreInstr_SkyLandBG2Xscroll(k: u16, game: &mut Game) {
    if ram_word!(game, time_is_frozen_flag) == 0 {
        HdmaobjPreInstr_SkyLandBG2XscrollInner(k, game);
    }
}
pub unsafe fn HdmaobjPreInstr_SkyLandBG2Xscroll2(k: u16, game: &mut Game) {
    if ram_word!(game, time_is_frozen_flag) == 0 {
        HdmaobjPreInstr_SkyLandBG2XscrollInner(k, game);
    }
}
pub unsafe fn RoomMainAsm_ScrollingSky(src: *const u16, game: &mut Game) {
    if ram_word!(game, time_is_frozen_flag) != 0 {
        *((&raw mut game.g_ram as *mut u8).offset(0x9f00).offset(0) as *mut u8 as *mut u16) =
            0 as u16;
    } else {
        set_ram_word!(game.reg_BG2VOFS = ram_word!(game, layer1_y_pos));
        let v0: u16 = ram_word!(game, vram_write_queue_tail);
        let v1: *mut VramWriteEntry =
            gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
        (*v1.offset(0)).size = 64 as u16;
        (*v1.offset(1)).size = 64 as u16;
        (*v1.offset(2)).size = 64 as u16;
        (*v1.offset(3)).size = 64 as u16;
        let mut tt: i32 = (ram_word!(game, layer1_y_pos) as i32 & 0x7f8 as i32) - 16 as i32;
        if tt & 0x8000 as i32 != 0 {
            tt = 0 as i32;
        }
        let v2: i32 = 8 as i32 * tt as u8 as i32;
        let y1: i32 = (tt >> 8 as i32) as u8 as i32;
        let y2: i32 = ((tt >> 8 as i32) + 1 as i32) as u8 as i32;
        let v3: VoidP = (*src.offset(y1 as isize) as i32 + v2) as VoidP;
        (*v1.offset(0)).src.addr = v3;
        (*v1.offset(1)).src.addr = (v3).wrapping_add(64u16);
        let v5: VoidP = (*src.offset(y2 as isize) as i32 + v2) as VoidP;
        (*v1.offset(2)).src.addr = v5;
        (*v1.offset(3)).src.addr = (v5).wrapping_add(64u16);
        (*v1.offset(0)).src.bank = 0x8a as u8;
        (*v1.offset(1)).src.bank = 0x8a as u8;
        (*v1.offset(2)).src.bank = 0x8a as u8;
        (*v1.offset(3)).src.bank = 0x8a as u8;
        let t: i32 =
            (*(&raw mut game.g_ram as *mut u8).offset(0x59) as i32 & 0xfc as i32) << 8 as i32;
        let v6: u16 = (t + 4 as i32
            * (ram_word!(game, layer1_y_pos) as i32 - 16 as i32 & 0x1f8 as i32))
            as u16;
        (*v1.offset(0)).vram_dst = v6;
        (*v1.offset(1)).vram_dst = (v6).wrapping_add(32u16);
        let v7: u16 = (t + 4 as i32
            * (ram_word!(game, layer1_y_pos) as i32 + 240 as i32 & 0x1f8 as i32))
            as u16;
        (*v1.offset(2)).vram_dst = v7;
        (*v1.offset(3)).vram_dst = (v7).wrapping_add(32u16);
        set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(28u16));
    };
}
pub unsafe fn RoomCode_ScrollingSkyLand(game: &mut Game) {
    RoomMainAsm_ScrollingSky(RomPtr_88(addr_off_88AD9C as u16, game) as *const u16, game);
}
pub unsafe fn RoomMainAsm_ScrollingSkyOcean(game: &mut Game) {
    RoomMainAsm_ScrollingSky(RomPtr_88(addr_off_88ADA6 as u16, game) as *const u16, game);
}
static unk_88B08C: SpawnHdmaObject_Args =
    SpawnHdmaObject_Args::new(0x42 as u8, 0x11 as u8, 0xb0ac as VoidP);
static kFirefleaFlashingShades: [u16; 12] = [
    0, 0x100, 0x200, 0x300, 0x400, 0x500, 0x600, 0x500, 0x400, 0x300, 0x200, 0x100,
];
static kFirefleaDarknessShades: [u16; 6] = [0, 0x600, 0xc00, 0x1200, 0x1800, 0x1900];
pub unsafe fn FxTypeFunc_24(game: &mut Game) {
    set_ram_word!(game.fireflea_flashing_timer = 6 as u16);
    set_ram_word!(game.fireflea_flashing_index = 0 as u16);
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88B08C as *const ::core::ffi::c_void,
        game,
    );
    *(&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0 as *mut u16) = 0 as u16;
    set_ram_word!(game.UNUSED_word_7E177C = 0 as u16);
    set_ram_word!(game.fireflea_darkness_level = 0 as u16);
    set_ram_word!(game.UNUSED_word_7E1780 = 24 as u16);
    set_ram_word!(game.UNUSED_word_7E1782 = kFirefleaFlashingShades[0 as i32 as usize]);
}
pub unsafe fn HdmaobjPreInstr_FirefleaBG3XScroll(k: u16, game: &mut Game) {
    let mut v2: i8 = 0;
    let mut v1: u16 = 0;
    set_ram_word!(
        game.fx_layer_blending_config_c = (ram_word!(game, fx_layer_blending_config_c) as i32
            & 0xff00 as i32
            | 0xc as i32) as u16
    );
    if ram_word!(game, time_is_frozen_flag) == 0 {
        if update_ram_word!(game.fireflea_flashing_timer, wrapping_sub(1), reread) == 0 {
            set_ram_word!(game.fireflea_flashing_timer = 6 as u16);
            if ram_word!(game, fireflea_darkness_level) as i32 - 10 as i32 & 0x8000 as i32 != 0 {
                v1 = (ram_word!(game, fireflea_flashing_index)).wrapping_add(1u16);
                if (ram_word!(game, fireflea_flashing_index)).wrapping_add(1u16) >= 0xcu16 {
                    v1 = 0 as u16;
                }
            } else {
                v1 = 6 as u16;
            }
            set_ram_word!(game.fireflea_flashing_index = v1);
        }
        v2 = (((kFirefleaDarknessShades
            [(ram_word!(game, fireflea_darkness_level) as i32 >> 1 as i32) as usize])
            .wrapping_add(
                kFirefleaFlashingShades[ram_word!(game, fireflea_flashing_index) as usize],
            )) as i32
            >> 8 as i32) as i8;
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = ((v2) as u8) | (0x80u8);
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = ((v2) as u8) | (0x40u8);
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = ((v2) as u8) | (0x20u8);
    }
}
pub unsafe fn ExpandingContractingHdmaEffect(game: &mut Game) {
    if update_ram_word!(game.set_to_e0_by_scrolling_sky, wrapping_add(1), reread) as i32 - 4 as i32
        & 0x8000 as i32
        == 0
    {
        set_ram_word!(game.set_to_e0_by_scrolling_sky = 0 as u16);
        if ram_word!(game, UNUSED_hdma_contracting_flag) != 0 {
            update_ram_word!(game.message_box_animation_y_radius, wrapping_sub(1024u16));
            if ram_word!(game, message_box_animation_y_radius) as i32 - 0x2000 as i32
                & 0x8000 as i32
                != 0
            {
                set_ram_word!(game.message_box_animation_y_radius = 0x2000 as u16);
                set_ram_word!(game.UNUSED_hdma_contracting_flag = 0 as u16);
            }
        } else {
            update_ram_word!(game.message_box_animation_y_radius, wrapping_add(1024u16));
            if ram_word!(game, message_box_animation_y_radius) as i32 + 0x8000 as i32
                & 0x8000 as i32
                == 0
            {
                update_ram_word!(game.UNUSED_hdma_contracting_flag, wrapping_add(1));
            }
        }
        set_ram_word!(game.message_box_animation_y0 = 128 as u16);
        set_ram_word!(game.message_box_animation_y1 = 128 as u16);
        set_ram_word!(game.message_box_animation_y2 = 127 as u16);
        set_ram_word!(game.message_box_animation_y3 = 127 as u16);
    }
    let mut v0: u16 = (2u16).wrapping_mul(ram_word!(game, message_box_animation_y0));
    let mut v1: u16 = (2u16).wrapping_mul(ram_word!(game, message_box_animation_y2));
    let mut r18: u16 = 0 as u16;
    let mut r20: u16 = 32 as u16;
    loop {
        *((&raw mut game.g_ram as *mut u8)
            .offset(0x9c00)
            .offset(v0 as isize) as *mut u8 as *mut u16) =
            (ram_word!(game, message_box_animation_y1))
                .wrapping_sub(ram_word!(game, message_box_animation_y0));
        *((&raw mut game.g_ram as *mut u8)
            .offset(0x9c00)
            .offset(v1 as isize) as *mut u8 as *mut u16) =
            (ram_word!(game, message_box_animation_y3))
                .wrapping_sub(ram_word!(game, message_box_animation_y2));
        r18 = (r18).wrapping_add(ram_word!(game, message_box_animation_y_radius));
        if Unreachable(game) {
            update_ram_word!(game.message_box_animation_y1, wrapping_add(1));
            update_ram_word!(game.message_box_animation_y3, wrapping_sub(1));
        }
        update_ram_word!(game.message_box_animation_y0, wrapping_add(1));
        update_ram_word!(game.message_box_animation_y2, wrapping_sub(1));
        v1 = (v1).wrapping_sub(2u16);
        v0 = (v0).wrapping_add(2u16);
        r20 = r20.wrapping_sub(1);
        if (r20 == 0) {
            break;
        }
    }
}
static g_word_88B256: [u16; 17] = [
    0x46, 1, 0x46, 3, 0x46, 2, 0x46, 1, 0x46, 1, 0x46, 2, 0x46, 2, 0x46, 1, 0x8000,
];
pub unsafe fn HandleEarthquakeSoundEffect(game: &mut Game) {
    let mut v1: i16 = 0;
    if (ram_word!(game, earthquake_sfx_timer) as i32 & 0x8000 as i32) == 0i32 && {
        (update_ram_word!(game.earthquake_sfx_timer, wrapping_sub(1), reread) as i32
            & 0x8000 as i32)
            != 0i32
    } {
        let mut v0: u16 = ram_word!(game, earthquake_sfx_index);
        v1 = g_word_88B256[(ram_word!(game, earthquake_sfx_index) as i32 >> 1 as i32) as usize]
            as i16;
        if v1 < 0i16 {
            v0 = 0 as u16;
            v1 = g_word_88B256[0 as i32 as usize] as i16;
        }
        QueueSfx2_Max6(v1 as u16, game);
        set_ram_word!(
            game.earthquake_sfx_timer =
                (g_word_88B256[((v0 as i32 >> 1 as i32) + 1 as i32) as usize] as i32
                    + (v0 as i32 & 1 as i32)
                    + (ram_word!(game, random_number) as i32 & 3 as i32)) as u16
        );
        set_ram_word!(game.earthquake_sfx_index = (v0).wrapping_add(4u16));
    }
}
static unk_88B289: SpawnHdmaObject_Args =
    SpawnHdmaObject_Args::new(0x42 as u8, 0x12 as u8, 0xc3e1 as VoidP);
static unk_88B291: SpawnHdmaObject_Args =
    SpawnHdmaObject_Args::new(0x42 as u8, 0x10 as u8, 0xc3f0 as VoidP);
static unk_88B2B1: SpawnHdmaObject_Args =
    SpawnHdmaObject_Args::new(0x42 as u8, 0x12 as u8, 0xc3e1 as VoidP);
static unk_88B2B9: SpawnHdmaObject_Args =
    SpawnHdmaObject_Args::new(0x42 as u8, 0x10 as u8, 0xc3f0 as VoidP);
pub unsafe fn FxTypeFunc_2_Lava(game: &mut Game) {
    set_ram_word!(game.fx_rising_function_bank_88 = ((0x88b343i32) as u16));
    set_ram_word!(game.lava_acid_y_pos = ram_word!(game, fx_base_y_pos));
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88B289 as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88B291 as *const ::core::ffi::c_void,
        game,
    );
    SpawnBG3ScrollHdmaObject(game);
    SpawnAnimtiles(addr_kAnimtiles_Lava as u16, game);
}
pub unsafe fn FxTypeFunc_4_Acid(game: &mut Game) {
    set_ram_word!(game.fx_rising_function_bank_88 = ((0x88b343i32) as u16));
    set_ram_word!(game.lava_acid_y_pos = ram_word!(game, fx_base_y_pos));
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88B2B1 as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88B2B9 as *const ::core::ffi::c_void,
        game,
    );
    SpawnBG3ScrollHdmaObject(game);
    SpawnAnimtiles(addr_kAnimtiles_Acid as u16, game);
}
pub unsafe fn FxHandleTide(game: &mut Game) {
    let mut v2: u16 = 0;
    let mut v5: u16 = 0;
    if *(((&raw mut game.g_ram as *mut u8).offset(0x197c) as *mut u16 as *mut u8).offset(1)
        as *mut i16)
        < 0i16
    {
        set_ram_word!(game.fx_y_suboffset = 0 as u16);
        set_ram_word!(game.fx_y_offset = 0 as u16);
        let v0: i32 = *((&raw mut game.g_ram as *mut u8).offset(0x1974) as *mut u16 as *mut u8)
            .offset(
                (::core::mem::size_of::<u16>())
                    .wrapping_div(::core::mem::size_of::<u8>())
                    .wrapping_sub(1_usize) as isize,
            ) as i32;
        let v1: u16 = (8u16).wrapping_mul((kSinCosTable8bit_Sext[v0 as usize]) as u16);
        if (kSinCosTable8bit_Sext[v0 as usize] as i32 & 0x1000 as i32) != 0i32 {
            update_ram_word!(game.fx_y_offset, wrapping_sub(1));
        }
        *(((&raw mut game.g_ram as *mut u8).offset(0x1970) as *mut u16 as *mut u8).offset(1)
            as *mut u16) = v1;
        if (kSinCosTable8bit_Sext[v0 as usize] as i32 & 0x8000 as i32) == 0i32 {
            v2 = (ram_word!(game, tide_phase)).wrapping_add(288u16);
        } else {
            v2 = (ram_word!(game, tide_phase)).wrapping_add(192u16);
        }
        set_ram_word!(game.tide_phase = v2);
    } else if (*(((&raw mut game.g_ram as *mut u8).offset(0x197c) as *mut u16 as *mut u8).offset(1)
        as *mut u16) as i32
        & 0x4000 as i32)
        != 0i32
    {
        set_ram_word!(game.fx_y_suboffset = 0 as u16);
        set_ram_word!(game.fx_y_offset = 0 as u16);
        let v3: i32 = *((&raw mut game.g_ram as *mut u8).offset(0x1974) as *mut u16 as *mut u8)
            .offset(
                (::core::mem::size_of::<u16>())
                    .wrapping_div(::core::mem::size_of::<u8>())
                    .wrapping_sub(1_usize) as isize,
            ) as i32;
        let v4: u16 = (32u16).wrapping_mul((kSinCosTable8bit_Sext[v3 as usize]) as u16);
        if (kSinCosTable8bit_Sext[v3 as usize] as i32 & 0x400 as i32) != 0i32 {
            update_ram_word!(game.fx_y_offset, wrapping_sub(1));
        }
        *(((&raw mut game.g_ram as *mut u8).offset(0x1970) as *mut u16 as *mut u8).offset(1)
            as *mut u16) = v4;
        if (kSinCosTable8bit_Sext[v3 as usize] as i32 & 0x8000 as i32) == 0i32 {
            v5 = (ram_word!(game, tide_phase)).wrapping_add(224u16);
        } else {
            v5 = (ram_word!(game, tide_phase)).wrapping_add(128u16);
        }
        set_ram_word!(game.tide_phase = v5);
    }
}
pub unsafe fn FxRisingFunction_LavaAcid(game: &mut Game) {
    if ram_word!(game, fx_y_vel) != 0 {
        if (ram_word!(game, fx_y_vel) as i32 & 0x8000 as i32) == 0i32 {
            if ram_word!(game, fx_target_y_pos) < ram_word!(game, fx_base_y_pos)
                || ram_word!(game, fx_target_y_pos) == ram_word!(game, fx_base_y_pos)
            {
                return;
            }
        } else if !(ram_word!(game, fx_target_y_pos) != ram_word!(game, fx_base_y_pos)
            && ram_word!(game, fx_target_y_pos) < ram_word!(game, fx_base_y_pos))
        {
            return;
        }
        set_ram_word!(game.fx_rising_function_bank_88 = ((0x88b367i32) as u16));
    }
}
pub unsafe fn FxRisingFunction_LavaAcid_WaitToRise(game: &mut Game) {
    HandleEarthquakeSoundEffect(game);
    set_ram_word!(game.earthquake_type = 21 as u16);
    update_ram_word!(game.earthquake_timer, |fresh44| (fresh44) | (0x20u16));
    if update_ram_word!(game.fx_timer, wrapping_sub(1), reread) == 0 {
        set_ram_word!(game.fx_rising_function_bank_88 = ((0x88b382i32) as u16));
    }
}
pub unsafe fn FxRisingFunction_LavaAcid_Raising(game: &mut Game) {
    HandleEarthquakeSoundEffect(game);
    set_ram_word!(game.earthquake_type = 21 as u16);
    update_ram_word!(game.earthquake_timer, |fresh42| (fresh42) | (0x20u16));
    if RaiseOrLowerFx(game) as i32 & 1 as i32 != 0 {
        set_ram_word!(game.fx_y_vel = 0 as u16);
        set_ram_word!(game.fx_rising_function_bank_88 = ((0x88b343i32) as u16));
    }
}
pub unsafe fn HdmaobjInstr_B3A9(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.hdma_object_C[(k as i32 >> 1i32)] = 112 as u16);
    hdp
}
pub unsafe fn CallFxRisingFunc(ea: u32, game: &mut Game) {
    match ea {
        8958787 => FxRisingFunction_LavaAcid(game),
        8958823 => FxRisingFunction_LavaAcid_WaitToRise(game),
        8958850 => FxRisingFunction_LavaAcid_Raising(game),
        8963112 => FxRisingFunction_C428_WaterNormal(game),
        8963148 => FxRisingFunction_WaterWaitToRise(game),
        8963160 => FxRisingFunction_WaterRising(game),
        _ => {
            Unreachable(game);
        }
    }
}
static kLavaSoundEffects: [u8; 8] = [0x12, 0x13, 0x14, 0x12, 0x13, 0x14, 0x12, 0x13];
pub unsafe fn HdmaobjPreInstr_LavaAcidBG3YScroll(k: u16, game: &mut Game) {
    let mut current_block: u64;
    let mut v2: i16 = 0;
    let mut v3: i16 = 0;
    let mut v4: i16 = 0;
    let mut v8: i16 = 0;
    set_ram_word!(game.fx_layer_blending_config_c = ram_word!(game, fx_layer_blending_config_b));
    if ram_word!(game, time_is_frozen_flag) == 0 {
        CallFxRisingFunc(
            ((ram_word!(game, fx_rising_function_bank_88)) as u32) | (0x880000u32),
            game,
        );
        FxHandleTide(game);
        SetHiLo(
            (&raw mut game.g_ram as *mut u8).offset(0x1962) as *mut u16,
            (&raw mut game.g_ram as *mut u8).offset(0x1960) as *mut u16,
            ((ram_word!(game, fx_y_offset) as u32) << 16 as i32
                | ram_word!(game, fx_y_suboffset) as u32)
                .wrapping_add(
                    (ram_word!(game, fx_base_y_pos) as u32) << 16 as i32
                        | ram_word!(game, fx_base_y_subpos) as u32,
                ),
        );
        set_ram_word!(game.bg3_xpos = ram_word!(game, reg_BG1HOFS));
        *((&raw mut game.g_ram as *mut u8).offset(0x9c00) as *mut u16) = 0 as u16;
        set_ram_word!(game.bg3_ypos = 0 as u16);
        v2 = (((ram_word!(game, fx_y_offset) as u32) << 16 as i32
            | ram_word!(game, fx_y_suboffset) as u32)
            .wrapping_add(
                (ram_word!(game, fx_base_y_pos) as u32) << 16 as i32
                    | ram_word!(game, fx_base_y_subpos) as u32,
            )
            >> 16 as i32) as i16;
        if v2 >= 0i16 {
            v3 = (v2).wrapping_sub((ram_word!(game, layer1_y_pos)) as i16);
            if v3 <= 0i16 {
                v4 = ((v3 as i32 ^ 0x1f as i32) & 0x1f as i32 | 0x100 as i32) as i16;
                current_block = 135057599068437321;
            } else if (v3 as u16) < 0x100u16 {
                v4 = !(v3 as i32) as u8 as i16;
                current_block = 135057599068437321;
            } else {
                current_block = 4068382217303356765;
            }
        } else {
            current_block = 4068382217303356765;
        }
        if current_block == 4068382217303356765 {
            v4 = 0 as i16;
        }
        *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(2) as *mut u8 as *mut u16) =
            v4 as u16;
        if ram_word!(game, fx_type) == 2u16
            && (ram_word!(game, lava_acid_y_pos) as i32 & 0x8000 as i32) == 0i32
        {
            let v5: i32 = ram_word!(game, hdma_object_index) as u8 as i32 >> 1 as i32;
            let ref mut fresh101 =
                *((&raw mut game.g_ram as *mut u8).offset(0x192c) as *mut u16).offset(v5 as isize);
            let fresh102 = *fresh101;
            *fresh101 = (*fresh101).wrapping_sub(1);
            if fresh102 == 1u16 {
                set_ram_word!(game.hdma_object_C[v5] = 112 as u16);
                QueueSfx2_Max6(
                    kLavaSoundEffects[(ram_word!(game, random_number) as i32 & 7 as i32) as usize]
                        as u16,
                    game,
                );
            }
        }
        set_ram_word!(game.random_number = u16::swap_bytes(ram_word!(game, random_number)));
        if (ram_word!(game, lava_acid_y_pos) as i32 & 0x8000 as i32) == 0i32 {
            v8 = (ram_word!(game, lava_acid_y_pos) as i32 - ram_word!(game, layer1_y_pos) as i32
                + 256 as i32) as i16;
            if v8 < 0i16 {
                v8 = 255 as i16;
                current_block = 7317190685143662286;
            } else if (v8 as u16) < 0x200u16 {
                current_block = 7317190685143662286;
            } else {
                current_block = 11042950489265723346;
            }
        } else {
            current_block = 11042950489265723346;
        }
        if current_block == 11042950489265723346 {
            v8 = 511 as i16;
        }
        *((&raw mut game.g_ram as *mut u8).offset(0x18d8) as *mut u16)
            .offset((ram_word!(game, hdma_object_index) as u8 as i32 >> 1 as i32) as isize) =
            (3 as i32 * ((v8 as i32 ^ 0x1ff as i32) + 1 as i32 & 0x3ff as i32) + 0xb62a as i32)
                as u16;
    } else {
    }
}
pub unsafe fn HdmaobjInstr_SetFlagB_Copy(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.hdma_object_B[(k as i32 >> 1i32)] = 1 as u16);
    hdp
}
pub unsafe fn HdmaobjPreInstr_LavaAcidBG2YScroll(k: u16, game: &mut Game) {
    set_ram_word!(game.g_word_7E9C44 = ram_word!(game, reg_BG2VOFS));
    if ram_word!(game, time_is_frozen_flag) == 0
        && (ram_word!(game, fx_liquid_options) as i32 & 6 as i32) != 0i32
    {
        if (ram_word!(game, fx_liquid_options) as i32 & 2 as i32) != 0i32 {
            Handle_LavaAcidBG2YScroll_Func3(k, game);
        } else {
            Handle_LavaAcidBG2YScroll_Func2(k, game);
        }
    } else {
        Handle_LavaAcidBG2YScroll_Func1(k, game);
    }
    *((&raw mut game.g_ram as *mut u8).offset(0x18d8) as *mut u16)
        .offset((ram_word!(game, hdma_object_index) as u8 as i32 >> 1 as i32) as isize) =
        (3 as i32 * (ram_word!(game, reg_BG2VOFS) as i32 & 0xf as i32) + 0xc0b1 as i32) as u16;
}
pub unsafe fn Handle_LavaAcidBG2YScroll_Func1(v0: u16, game: &mut Game) {
    WriteReg(
        (((*((&raw mut game.g_ram as *mut u8).offset(0x18c0) as *mut u16 as *mut u8)
            .offset(v0 as i32 as isize)) as u32)
            .wrapping_add(17153u32)) as u16,
        0x10 as u8,
        game,
    );
    let mut v1: u8 = 30 as u8;
    let v2: u16 = (ram_word!(game, reg_BG2VOFS)) & (0x1ffu16);
    loop {
        set_ram_word!(game.g_word_7E9C46[(v1 as i32 >> 1i32)] = v2);
        v1 = (v1).wrapping_sub(2u8);
        if ((v1 as i32 & 0x80 as i32) != 0i32) {
            break;
        }
    }
}
pub unsafe fn Handle_LavaAcidBG2YScroll_Func2(v0: u16, game: &mut Game) {
    WriteReg(
        (((*((&raw mut game.g_ram as *mut u8).offset(0x18c0) as *mut u16 as *mut u8)
            .offset(v0 as i32 as isize)) as u32)
            .wrapping_add(17153u32)) as u16,
        0xf as u8,
        game,
    );
    let v1: i32 = v0 as i32 >> 1 as i32;
    let ref mut fresh54 =
        *((&raw mut game.g_ram as *mut u8).offset(0x1920) as *mut u16).offset(v1 as isize);
    let fresh55 = *fresh54;
    *fresh54 = (*fresh54).wrapping_sub(1);
    if fresh55 == 1u16 {
        set_ram_word!(game.hdma_object_B[v1] = 6 as u16);
        set_ram_word!(
            game.hdma_object_A[v1] =
                (ram_word!(game, hdma_object_A[v1]) as i32 - 2 as i32 & 0x1e as i32) as u16
        );
    }
    let mut v3: u8 = *((&raw mut game.g_ram as *mut u8).offset(0x1914) as *mut u16 as *mut u8)
        .offset(v0 as i32 as isize);
    let mut v4: u8 = 30 as u8;
    let mut n: i32 = 15 as i32;
    loop {
        set_ram_word!(
            game.g_word_7E9C46[(v4 as i32 >> 1i32)] =
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0x88b589)) as *const u8 as *mut u16)
                    .offset((v3 as i32 >> 1 as i32) as isize) as i32
                    + ram_word!(game, reg_BG2HOFS) as i32
                    & 0x1ff as i32) as u16
        );
        v3 = (v3 as i32 - 2 as i32 & 0x1e as i32) as u8;
        v4 = (v4 as i32 - 2 as i32 & 0x1e as i32) as u8;
        n -= 1;
        if (n < 0i32) {
            break;
        }
    }
}
pub unsafe fn Handle_LavaAcidBG2YScroll_Func3(v0: u16, game: &mut Game) {
    WriteReg(
        (*((&raw mut game.g_ram as *mut u8).offset(0x18c0) as *mut u16 as *mut u8)
            .offset(v0 as i32 as isize) as i32
            + BBAD0 as i32) as SnesRegs as u16,
        0x10 as u8,
        game,
    );
    let v1: i32 = v0 as i32 >> 1 as i32;
    let ref mut fresh56 =
        *((&raw mut game.g_ram as *mut u8).offset(0x1920) as *mut u16).offset(v1 as isize);
    let fresh57 = *fresh56;
    *fresh56 = (*fresh56).wrapping_sub(1);
    if fresh57 == 1u16 {
        set_ram_word!(game.hdma_object_B[v1] = 4 as u16);
        set_ram_word!(
            game.hdma_object_A[v1] =
                (ram_word!(game, hdma_object_A[v1]) as i32 - 2 as i32 & 0x1e as i32) as u16
        );
    }
    let mut v3: u8 = (*(((&raw mut game.g_ram as *mut u8).offset(0x1914) as *mut u16)
        .offset(v1 as isize) as *mut u16 as *mut u8)
        .offset(0) as i32
        + 2 as i32 * (ram_word!(game, reg_BG2VOFS) as i32 & 0xf as i32)
        & 0x1e as i32) as u8;
    let mut v4: u8 = (2 as i32 * (ram_word!(game, reg_BG2VOFS) as i32 & 0xf as i32) + 30 as i32
        & 0x1e as i32) as u8;
    let mut n: i32 = 15 as i32;
    loop {
        set_ram_word!(
            game.g_word_7E9C46[(v4 as i32 >> 1i32)] =
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0x88b60a)) as *const u8 as *mut u16)
                    .offset((v3 as i32 >> 1 as i32) as isize) as i32
                    + ram_word!(game, reg_BG2VOFS) as i32
                    & 0x1ff as i32) as u16
        );
        v4 = (v4 as i32 - 2 as i32 & 0x1e as i32) as u8;
        v3 = (v3 as i32 - 2 as i32 & 0x1e as i32) as u8;
        n -= 1;
        if (n < 0i32) {
            break;
        }
    }
}
static unk_88C40F: SpawnHdmaObject_Args =
    SpawnHdmaObject_Args::new(0x42 as u8, 0x11 as u8, 0xd856 as VoidP);
static unk_88C41F: SpawnHdmaObject_Args =
    SpawnHdmaObject_Args::new(0x42 as u8, 0xf as u8, 0xd847 as VoidP);
pub unsafe fn FxTypeFunc_6_Water(game: &mut Game) {
    set_ram_word!(game.fx_rising_function_bank_88 = ((0x88c428i32) as u16));
    set_ram_word!(game.fx_y_pos = ram_word!(game, fx_base_y_pos));
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88C40F as *const ::core::ffi::c_void,
        game,
    );
    if (ram_word!(game, fx_liquid_options) as i32 & 2 as i32) != 0i32 {
        SpawnHdmaObject(
            0x88 as u8,
            &raw const unk_88C41F as *const ::core::ffi::c_void,
            game,
        );
    }
    SpawnBG3ScrollHdmaObject(game);
}
pub unsafe fn FxRisingFunction_C428_WaterNormal(game: &mut Game) {
    if ram_word!(game, fx_y_vel) != 0 {
        if (ram_word!(game, fx_y_vel) as i32 & 0x8000 as i32) == 0i32 {
            if ram_word!(game, fx_target_y_pos) < ram_word!(game, fx_base_y_pos)
                || ram_word!(game, fx_target_y_pos) == ram_word!(game, fx_base_y_pos)
            {
                return;
            }
        } else if !(ram_word!(game, fx_target_y_pos) != ram_word!(game, fx_base_y_pos)
            && ram_word!(game, fx_target_y_pos) < ram_word!(game, fx_base_y_pos))
        {
            return;
        }
        set_ram_word!(game.fx_rising_function_bank_88 = ((0x88c44ci32) as u16));
    }
}
pub unsafe fn FxRisingFunction_WaterWaitToRise(game: &mut Game) {
    if update_ram_word!(game.fx_timer, wrapping_sub(1), reread) == 0 {
        set_ram_word!(game.fx_rising_function_bank_88 = ((0x88c458i32) as u16));
    }
}
pub unsafe fn FxRisingFunction_WaterRising(game: &mut Game) {
    if RaiseOrLowerFx(game) as i32 & 1 as i32 != 0 {
        set_ram_word!(game.fx_rising_function_bank_88 = ((0x88c428i32) as u16));
        set_ram_word!(game.fx_y_vel = 0 as u16);
    }
}
pub unsafe fn HdmaobjInstr_SetFlagB_Copy2(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.hdma_object_B[(k as i32 >> 1i32)] = 1 as u16);
    hdp
}
pub unsafe fn CallFxRisingFunction(ea: u32, game: &mut Game) {
    match ea {
        8958787 => FxRisingFunction_LavaAcid(game),
        8958823 => FxRisingFunction_LavaAcid_WaitToRise(game),
        8958850 => FxRisingFunction_LavaAcid_Raising(game),
        8963112 => FxRisingFunction_C428_WaterNormal(game),
        8963148 => FxRisingFunction_WaterWaitToRise(game),
        8963160 => FxRisingFunction_WaterRising(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn HdmaobjPreInstr_WaterBG3XScroll(k: u16, game: &mut Game) {
    set_ram_word!(game.fx_layer_blending_config_c = ram_word!(game, fx_layer_blending_config_b));
    if ram_word!(game, time_is_frozen_flag) != 0 {
        return;
    }
    CallFxRisingFunction(
        ((ram_word!(game, fx_rising_function_bank_88)) as u32) | (0x880000u32),
        game,
    );
    FxHandleTide(game);
    SetHiLo(
        (&raw mut game.g_ram as *mut u8).offset(0x195e) as *mut u16,
        (&raw mut game.g_ram as *mut u8).offset(0x195c) as *mut u16,
        ((ram_word!(game, fx_y_offset) as u32) << 16 as i32
            | ram_word!(game, fx_y_suboffset) as u32)
            .wrapping_add(
                (ram_word!(game, fx_base_y_pos) as u32) << 16 as i32
                    | ram_word!(game, fx_base_y_subpos) as u32,
            ),
    );
    let v3: i16 =
        ((ram_word!(game, fx_y_pos)) as i16).wrapping_sub((ram_word!(game, layer1_y_pos)) as i16);
    if (ram_word!(game, fx_y_pos) as i16) < 0i16 {
        set_ram_word!(game.bg3_ypos = 0 as u16);
    } else if v3 <= 0i16 {
        set_ram_word!(
            game.bg3_ypos = ((v3 as i32 ^ 0x1f as i32) & 0x1f as i32 | 0x100 as i32) as u16
        );
    } else if (v3 as u16) < 0x100u16 {
        set_ram_word!(game.bg3_ypos = !(v3 as i32) as u8 as u16);
    } else {
        set_ram_word!(game.bg3_ypos = 0 as u16);
    }
    let v5: i32 = k as i32 >> 1 as i32;
    let r20: u16 = (ram_word!(game, layer1_x_pos)).wrapping_add(
        (*(((&raw mut game.g_ram as *mut u8).offset(0x192c) as *mut u16).offset(v5 as isize)
            as *mut u16 as *mut u8)
            .offset(
                (::core::mem::size_of::<u16>())
                    .wrapping_div(::core::mem::size_of::<u8>())
                    .wrapping_sub(1_usize) as isize,
            ) as i8) as u16,
    );
    let ref mut fresh98 =
        *((&raw mut game.g_ram as *mut u8).offset(0x1920) as *mut u16).offset(v5 as isize);
    let fresh99 = *fresh98;
    *fresh98 = (*fresh98).wrapping_sub(1);
    if fresh99 == 1u16 {
        set_ram_word!(game.hdma_object_B[v5] = 10 as u16);
        set_ram_word!(
            game.hdma_object_A[v5] =
                (ram_word!(game, hdma_object_A[v5]) as i32 + 2 as i32 & 0x1e as i32) as u16
        );
    }
    let mut v7: u8 = ram_word!(game, hdma_object_A[v5]) as u8;
    let mut i: i32 = 30 as i32;
    while (i & 0x80 as i32) == 0i32 {
        *((&raw mut game.g_ram as *mut u8)
            .offset(0x9c00)
            .offset((v7 as i32 + 4 as i32) as isize) as *mut u8 as *mut u16) =
            ((g_word_88C46E[(i >> 1 as i32) as usize]) as u16).wrapping_add(r20);
        v7 = (v7 as i32 - 2 as i32 & 0x1e as i32) as u8;
        i -= 2 as i32;
    }
    if (ram_word!(game, fx_liquid_options) as i32 & 1 as i32) != 0i32 {
        update_ram_word!(
            game.hdma_object_C[(k as i32 >> 1 as i32)],
            wrapping_add(64u16)
        );
    }
    let mut v9: i16 = (ram_word!(game, fx_y_pos) as i32 - ram_word!(game, layer1_y_pos) as i32
        + 256 as i32) as i16;
    if (ram_word!(game, fx_y_pos) as i16) < 0i16 {
        v9 = 511 as i16;
    } else if v9 >= 0i16 {
        if (v9 as u16) >= 0x200u16 {
            v9 = 511 as i16;
        }
    } else {
        v9 = (v9 as i32 & 0xf as i32 | 0x100 as i32) as i16;
    }
    set_ram_word!(
        game.hdma_object_table_pointers[(k as i32 >> 1i32)] = (3 as i32
            * ((v9 as i32 ^ 0x1ff as i32) + 1 as i32 & 0x3ff as i32)
            + 0xc645 as i32) as u16
    );
}
pub unsafe fn HdmaobjInstr_SetFlagB_Copy3(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.hdma_object_B[(k as i32 >> 1i32)] = 1 as u16);
    hdp
}
pub unsafe fn HdmaobjPreInstr_WaterBG2XScroll(k: u16, game: &mut Game) {
    set_ram_word!(game.g_word_7E9C44 = ram_word!(game, reg_BG2HOFS));
    if ram_word!(game, time_is_frozen_flag) != 0 {
        HdmaobjPreInstr_WaterBG2XScroll_Func1(k, game);
    } else if (ram_word!(game, fx_liquid_options) as i32 & 2 as i32) == 0i32 {
        set_ram_word!(game.hdma_object_channels_bitmask[(k as u8 as i32 >> 1i32)] = 0 as u16);
        HdmaobjPreInstr_WaterBG2XScroll_Func1(k, game);
    } else {
        HdmaobjPreInstr_WaterBG2XScroll_Func2(k, game);
    }
    let mut v1: i16 = (ram_word!(game, fx_y_pos) as i32 - ram_word!(game, layer1_y_pos) as i32
        + 256 as i32) as i16;
    if (ram_word!(game, fx_y_pos) as i16) < 0i16 {
        v1 = 511 as i16;
    } else if v1 >= 0i16 {
        if (v1 as u16) >= 0x200u16 {
            v1 = 511 as i16;
        }
    } else {
        v1 = (ram_word!(game, fx_y_pos) as i32 - ram_word!(game, layer1_y_pos) as i32 & 0xf as i32
            | 0x100 as i32) as i16;
    }
    set_ram_word!(
        game.hdma_object_table_pointers[(k as u8 as i32 >> 1i32)] =
            (3 as i32 * ((v1 as i32 ^ 0x1ff as i32) & 0x3ff as i32) + 0xcf46 as i32) as u16
    );
}
pub unsafe fn HdmaobjPreInstr_WaterBG2XScroll_Func2(k: u16, game: &mut Game) {
    let v1: i32 = k as u8 as i32 >> 1 as i32;
    let ref mut fresh96 =
        *((&raw mut game.g_ram as *mut u8).offset(0x1920) as *mut u16).offset(v1 as isize);
    let fresh97 = *fresh96;
    *fresh96 = (*fresh96).wrapping_sub(1);
    if fresh97 == 1u16 {
        set_ram_word!(game.hdma_object_B[v1] = 6 as u16);
        set_ram_word!(
            game.hdma_object_A[v1] =
                (ram_word!(game, hdma_object_A[v1]) as i32 + 2 as i32 & 0x1e as i32) as u16
        );
    }
    let mut v3: u8 = (*(((&raw mut game.g_ram as *mut u8).offset(0x1914) as *mut u16)
        .offset(v1 as isize) as *mut u16 as *mut u8)
        .offset(0) as i32
        + 2 as i32 * (ram_word!(game, reg_BG2VOFS) as i32 & 0xf as i32)
        & 0x1e as i32) as u8;
    let mut v4: u8 = (2 as i32 * (ram_word!(game, reg_BG2VOFS) as i32 & 0xf as i32) + 30 as i32
        & 0x1e as i32) as u8;
    let mut n: i32 = 15 as i32;
    loop {
        *((&raw mut game.g_ram as *mut u8).offset(0x9c46) as *mut u16)
            .offset(((v4 as i32 >> 1 as i32) + 1 as i32) as isize) =
            ((g_word_88C46E[(v3 as i32 >> 1 as i32) as usize]) as u16)
                .wrapping_add(ram_word!(game, reg_BG2HOFS));
        v4 = (v4 as i32 - 2 as i32 & 0x1e as i32) as u8;
        v3 = (v3 as i32 - 2 as i32 & 0x1e as i32) as u8;
        n -= 1;
        if (n < 0i32) {
            break;
        }
    }
}
pub unsafe fn HdmaobjPreInstr_WaterBG2XScroll_Func1(k: u16, game: &mut Game) {
    let mut v1: u8 = 30 as u8;
    let v2: u16 = ram_word!(game, reg_BG2HOFS);
    loop {
        *((&raw mut game.g_ram as *mut u8).offset(0x9c46) as *mut u16)
            .offset(((v1 as i32 >> 1 as i32) + 1 as i32) as isize) = v2;
        v1 = (v1).wrapping_sub(2u8);
        if ((v1 as i32 & 0x80 as i32) != 0i32) {
            break;
        }
    }
}
pub unsafe fn SpawnBG3ScrollHdmaObject(game: &mut Game) {
    static unk_88D869: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x43 as u8, 0x11 as u8, 0xd8d0 as VoidP);
    SpawnHdmaObjectToSlot0xA(
        0x88 as u8,
        &raw const unk_88D869 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn FxTypeFunc_28_CeresRidley(game: &mut Game) {
    static unk_88D8E2: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x40 as u8, 0x5 as u8, 0xd906 as VoidP);
    static unk_88D8EA: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0 as u8, 0x2c as u8, 0xd91d as VoidP);
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88D8E2 as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88D8EA as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn sub_88D916(game: &mut Game) {
    set_ram_word!(game.hdma_data_table_in_ceres = 9 as u16);
}
pub unsafe fn FxTypeFunc_CeresElevator(game: &mut Game) {
    static unk_88D92C: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x40 as u8, 0x5 as u8, 0xd939 as VoidP);
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88D92C as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn HdmaobjInstr_SetVideoMode1(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.hdma_data_table_in_ceres = 9 as u16);
    hdp
}
pub unsafe fn FxTypeFunc_A_Rain(game: &mut Game) {
    static unk_88D95C: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x43 as u8, 0x11 as u8, 0xd96c as VoidP);
    *(&raw mut game.g_ram as *mut u8).offset(0x5b) = 92 as u8;
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88D95C as *const ::core::ffi::c_void,
        game,
    );
    SpawnBG3ScrollHdmaObject(game);
    SpawnAnimtiles(addr_kAnimtiles_Rain as u16, game);
}
pub unsafe fn HdmaobjInstr_1938_RandomNumber(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(
        game.hdma_object_D[(k as i32 >> 1i32)] =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88d992)) as *const u8 as *mut u16)
                .offset(
                    ((ram_word!(game, random_number) as i32 >> 1 as i32 & 6 as i32) as u16 as i32
                        >> 1 as i32) as isize,
                )
    );
    hdp
}
pub unsafe fn HdmaobjPreInstr_RainBg3Scroll(k: u16, game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x5b) = 92 as u8;
    set_ram_word!(game.fx_layer_blending_config_c = ram_word!(game, fx_layer_blending_config_b));
    if ram_word!(game, time_is_frozen_flag) == 0 {
        let v1: i32 = k as u8 as i32 >> 1 as i32;
        set_ram_word!(
            game.bg3_ypos = (ram_word!(game, hdma_object_E[v1]) as i32
                - ram_word!(game, layer1_y_pos) as i32
                + *(((&raw mut game.g_ram as *mut u8).offset(0x1914) as *mut u16)
                    .offset(v1 as isize) as *mut u16 as *mut u8)
                    .offset(
                        (::core::mem::size_of::<u16>())
                            .wrapping_div(::core::mem::size_of::<u8>())
                            .wrapping_sub(1_usize) as isize,
                    ) as i8 as i32) as u16
        );
        update_ram_word!(game.hdma_object_A[v1], wrapping_sub(1536u16));
        set_ram_word!(game.hdma_object_E[v1] = ram_word!(game, layer1_y_pos));
        set_ram_word!(
            game.bg3_xpos = (ram_word!(game, hdma_object_F[v1]) as i32
                - ram_word!(game, layer1_x_pos) as i32
                + *(((&raw mut game.g_ram as *mut u8).offset(0x1920) as *mut u16)
                    .offset(v1 as isize) as *mut u16 as *mut u8)
                    .offset(
                        (::core::mem::size_of::<u16>())
                            .wrapping_div(::core::mem::size_of::<u8>())
                            .wrapping_sub(1_usize) as isize,
                    ) as i8 as i32) as u16
        );
        update_ram_word!(
            game.hdma_object_B[v1],
            wrapping_add(ram_word!(game, hdma_object_D[v1]))
        );
        set_ram_word!(game.hdma_object_F[v1] = ram_word!(game, layer1_x_pos));
    }
}
pub unsafe fn FxTypeFunc_8_Spores(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x5b) = 92 as u8;
    static unk_88DA1D: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x42 as u8, 0x11 as u8, 0xda2d as VoidP);
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88DA1D as *const ::core::ffi::c_void,
        game,
    );
    SpawnBG3ScrollHdmaObject(game);
    SpawnAnimtiles(addr_kAnimtiles_Spores as u16, game);
}
pub unsafe fn HdmaobjPreInstr_SporesBG3Xscroll(k: u16, game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x5b) = 92 as u8;
    set_ram_word!(game.fx_layer_blending_config_c = ram_word!(game, fx_layer_blending_config_b));
    if ram_word!(game, time_is_frozen_flag) == 0 {
        let v1: i32 = k as u8 as i32 >> 1 as i32;
        set_ram_word!(
            game.bg3_ypos = (ram_word!(game, layer1_y_pos)).wrapping_add(
                (*(((&raw mut game.g_ram as *mut u8).offset(0x192c) as *mut u16).offset(v1 as isize)
                    as *mut u16 as *mut u8)
                    .offset(
                        (::core::mem::size_of::<u16>())
                            .wrapping_div(::core::mem::size_of::<u8>())
                            .wrapping_sub(1_usize) as isize,
                    ) as i8) as u16,
            )
        );
        let v3: u16 = (ram_word!(game, hdma_object_C[v1])).wrapping_sub(64u16);
        set_ram_word!(game.hdma_object_C[v1] = v3);
        set_ram_word!(
            game.bg3_xpos = (ram_word!(game, layer1_x_pos)).wrapping_add(
                (*(((&raw mut game.g_ram as *mut u8).offset(0x1938) as *mut u16).offset(v1 as isize)
                    as *mut u16 as *mut u8)
                    .offset(
                        (::core::mem::size_of::<u16>())
                            .wrapping_div(::core::mem::size_of::<u8>())
                            .wrapping_sub(1_usize) as isize,
                    ) as i8) as u16,
            )
        );
        set_ram_word!(game.hdma_object_D[v1] = ram_word!(game, hdma_object_D[v1]));
    }
}
pub unsafe fn FxTypeFunc_C(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x5b) = 92 as u8;
    static unk_88DB14: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x43 as u8, 0x11 as u8, 0xdb19 as VoidP);
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88DB14 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn HdmaobjPreInstr_FogBG3Scroll(k: u16, game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x5b) = 92 as u8;
    set_ram_word!(game.fx_layer_blending_config_c = ram_word!(game, fx_layer_blending_config_b));
    if ram_word!(game, time_is_frozen_flag) == 0 {
        let v1: i32 = k as u8 as i32 >> 1 as i32;
        set_ram_word!(
            game.bg3_ypos = (ram_word!(game, layer1_y_pos)).wrapping_add(
                (*(((&raw mut game.g_ram as *mut u8).offset(0x1914) as *mut u16).offset(v1 as isize)
                    as *mut u16 as *mut u8)
                    .offset(
                        (::core::mem::size_of::<u16>())
                            .wrapping_div(::core::mem::size_of::<u8>())
                            .wrapping_sub(1_usize) as isize,
                    ) as i8) as u16,
            )
        );
        update_ram_word!(game.hdma_object_A[v1], wrapping_sub(64u16));
        set_ram_word!(
            game.bg3_xpos = (ram_word!(game, layer1_x_pos)).wrapping_add(
                (*(((&raw mut game.g_ram as *mut u8).offset(0x1920) as *mut u16).offset(v1 as isize)
                    as *mut u16 as *mut u8)
                    .offset(
                        (::core::mem::size_of::<u16>())
                            .wrapping_div(::core::mem::size_of::<u8>())
                            .wrapping_sub(1_usize) as isize,
                    ) as i8) as u16,
            )
        );
        update_ram_word!(game.hdma_object_B[v1], wrapping_add(80u16));
    }
}
pub unsafe fn FxTypeFunc_26_TourianEntranceStatue(game: &mut Game) {
    static unk_88DBBA: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x42 as u8, 0x11 as u8, 0xd856 as VoidP);
    static unk_88DBC2: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x42 as u8, 0x10 as u8, 0xdcfa as VoidP);
    if CheckEventHappened(0xa as u16, game) as i32 & 1 as i32 != 0 {
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x6 as u8, 0xc as u8, 0xb777 as VoidP),
            game,
        );
        *((&raw mut game.g_ram as *mut u8).offset(0xcd20) as *mut u16) = 514 as u16;
    }
    *(&raw mut game.g_ram as *mut u8).offset(0x59) = 74 as u8;
    set_ram_word!(game.fx_rising_function_bank_88 = ((0x88c428i32) as u16));
    set_ram_word!(game.fx_y_pos = ram_word!(game, fx_base_y_pos));
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88DBBA as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88DBC2 as *const ::core::ffi::c_void,
        game,
    );
    SpawnBG3ScrollHdmaObject(game);
}
pub unsafe fn sub_88DBCB(k: u16, game: &mut Game) {
    *(&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x9e00)
        as *mut ExpandingSquareTransitionHdma)
        .offset(0))
    .field_0 as *mut u16) = (ram_word!(game, layer1_y_pos))
        .wrapping_add(ram_word!(game, hdma_object_B[(k as i32 >> 1i32)]));
}
pub unsafe fn HdmaobjPreInstr_CheckLotsOfEventsHappened(mut v0: u16, game: &mut Game) {
    if CheckEventHappened(6 as u16, game) as i32 & 1 as i32 != 0
        && CheckEventHappened(7 as u16, game) as i32 & 1 as i32 != 0
        && CheckEventHappened(8 as u16, game) as i32 & 1 as i32 != 0
        && CheckEventHappened(9 as u16, game) as i32 & 1 as i32 != 0
    {
        update_ram_word!(game.tourian_entrance_statue_animstate, |fresh90| (fresh90)
            | (0x10u16));
        if (ram_word!(game, tourian_entrance_statue_animstate) as i32 & 0x8000 as i32) == 0i32 {
            set_ram_word!(game.hdma_object_C[(v0 as i32 >> 1i32)] = 300 as u16);
            v0 = ram_word!(game, hdma_object_index);
            let v1: i32 = ram_word!(game, hdma_object_index) as i32 >> 1 as i32;
            set_ram_word!(game.hdma_object_instruction_timers[v1] = 1 as u16);
            update_ram_word!(
                game.hdma_object_instruction_list_pointers[v1],
                wrapping_add(2u16)
            );
        }
    }
    sub_88DBCB(v0, game);
}
pub unsafe fn HdmaobjPreInstr_DC23(k: u16, game: &mut Game) {
    let v0: u16 = k;
    HandleEarthquakeSoundEffect(game);
    set_ram_word!(game.earthquake_type = 13 as u16);
    update_ram_word!(game.earthquake_timer, |fresh87| (fresh87) | (0x20u16));
    let v1: i32 = v0 as u8 as i32 >> 1 as i32;
    if (update_ram_word!(game.hdma_object_C[v1], wrapping_sub(1), reread) as i32 & 0x8000 as i32)
        != 0i32
    {
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueDustClouds as u16, 0 as u16, game);
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueDustClouds as u16, 0 as u16, game);
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueDustClouds as u16, 0 as u16, game);
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueDustClouds as u16, 0 as u16, game);
        set_ram_word!(game.hdma_object_instruction_timers[v1] = 1 as u16);
        update_ram_word!(
            game.hdma_object_instruction_list_pointers[v1],
            wrapping_add(2u16)
        );
    }
    sub_88DBCB(v0, game);
}
pub unsafe fn HdmaobjPreInstr_DC69(k: u16, game: &mut Game) {
    HandleEarthquakeSoundEffect(game);
    set_ram_word!(game.earthquake_type = 13 as u16);
    update_ram_word!(game.earthquake_timer, |fresh85| (fresh85) | (0x20u16));
    if ram_word!(game, time_is_frozen_flag) == 0 {
        let v1: i32 = k as i32 >> 1 as i32;
        AddToHiLo(
            ((&raw mut game.g_ram as *mut u8).offset(0x1920) as *mut u16).offset(v1 as isize)
                as *mut u16,
            ((&raw mut game.g_ram as *mut u8).offset(0x1914) as *mut u16).offset(v1 as isize)
                as *mut u16,
            -(0x4000 as i32) as u32,
        );
        if ram_word!(game, hdma_object_B[v1]) == 0xff10u16 {
            SpawnHardcodedPlm(
                SpawnHardcodedPlmArgs::new(0x6 as u8, 0xc as u8, 0xb773 as VoidP),
                game,
            );
            SetEventHappened(0xa as u16, game);
            set_ram_word!(game.hdma_object_instruction_timers[v1] = 1 as u16);
            update_ram_word!(
                game.hdma_object_instruction_list_pointers[v1],
                wrapping_add(2u16)
            );
        }
        sub_88DBCB(k, game);
    }
}
pub unsafe fn HdmaobjPreInstr_DCBA(v0: u16, game: &mut Game) {
    set_ram_word!(game.tourian_entrance_statue_finished = 0x8000 as u16);
    *((&raw mut game.g_ram as *mut u8).offset(0xcd20) as *mut u16) = 514 as u16;
    sub_88DBCB(v0, game);
}
pub unsafe fn HdmaobjInstr_GotoIfEventHappened(
    k: u16,
    hdp: *const u8,
    game: &mut Game,
) -> *const u8 {
    let v2: i32 = k as i32 >> 1 as i32;
    set_ram_word!(game.hdma_object_C[v2] = 0 as u16);
    set_ram_word!(game.hdma_object_A[v2] = 0 as u16);
    if CheckEventHappened(0xa as u16, game) != 0 {
        set_ram_word!(game.hdma_object_B[v2] = -(240 as i32) as u16);
        *(&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0 as *mut u16) = -(240 as i32) as u16;
        *(hdp as *mut u16) as usize as *const u8
    } else {
        set_ram_word!(game.hdma_object_B[v2] = 0 as u16);
        *(&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x9e00)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0))
        .field_0 as *mut u16) = 0 as u16;
        *((&raw mut game.g_ram as *mut u8).offset(0xcd20) as *mut u16) = 1 as u16;
        hdp.offset(2)
    }
}
pub unsafe fn SpawnBombTorizoHaze(game: &mut Game) {
    static unk_88DD36: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x2 as u8, 0x32 as u8, 0xdd4a as VoidP);
    static unk_88DD3E: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0 as u8, 0x32 as u8, 0xdd62 as VoidP);
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88DD36 as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88DD3E as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn HdmaobjPreInstr_BombTorizoHazeColorMathBgColor(k: u16, game: &mut Game) {
    set_ram_word!(game.fx_layer_blending_config_c = 44 as u16);
}
pub unsafe fn FxTypeFunc_2C_Haze(game: &mut Game) {
    static unk_88DDD4: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x40 as u8, 0x32 as u8, 0xded3 as VoidP);
    static unk_88DDDD: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x40 as u8, 0x32 as u8, 0xdeeb as VoidP);
    if CheckBossBitForCurArea(1 as u16, game) as i32 & 1 as i32 != 0 {
        SpawnHdmaObject(
            0x88 as u8,
            &raw const unk_88DDDD as *const ::core::ffi::c_void,
            game,
        );
    } else {
        SpawnHdmaObject(
            0x88 as u8,
            &raw const unk_88DDD4 as *const ::core::ffi::c_void,
            game,
        );
    };
}
pub unsafe fn HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyAlive(k: u16, game: &mut Game) {
    sub_88DE18(k, 0x80 as u16, game);
}
pub unsafe fn HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyDead(k: u16, game: &mut Game) {
    sub_88DE18(k, 0x20 as u16, game);
}
pub unsafe fn sub_88DE18(k: u16, a: u16, game: &mut Game) {
    let v2: i32 = k as u8 as i32 >> 1 as i32;
    set_ram_word!(game.hdma_object_B[v2] = a);
    set_ram_word!(game.hdma_object_A[v2] = 0 as u16);
    if ram_word!(game, door_transition_function) as i32 == 0x82e737 as i32 & 0xffff as i32 {
        set_ram_word!(game.hdma_object_pre_instructions[v2] = ((0x88de2di32) as u16));
        HdmaobjPreInstr_HazeColorMathSubscreen_FadingIn(k, game);
    }
}
pub unsafe fn HdmaobjPreInstr_HazeColorMathSubscreen_FadingIn(k: u16, game: &mut Game) {
    let mut v1: i8 = 0;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80 as u8;
    set_ram_word!(game.fx_layer_blending_config_c = 44 as u16);
    if ram_word!(game, hdma_object_A[(k as u8 as i32 >> 1i32)]) == 16u16 {
        set_ram_word!(
            game.hdma_object_pre_instructions[(k as u8 as i32 >> 1i32)] = ((0x88de74i32) as u16)
        );
    } else {
        let v3: u8 = k as u8;
        let r20: u8 = *((&raw mut game.g_ram as *mut u8).offset(0x1920) as *mut u16 as *mut u8)
            .offset(k as u8 as i32 as isize);
        v1 = *((&raw mut game.g_ram as *mut u8).offset(0x1914) as *mut u16 as *mut u8)
            .offset(k as u8 as i32 as isize) as i8;
        let mut i: i32 = 15 as i32;
        while i >= 0i32 {
            let r18: u8 = v1 as u8;
            *((&raw mut game.g_ram as *mut u8).offset(0x9d00) as *mut u16 as *mut u8)
                .offset(i as u8 as i32 as isize) = (r20) | ((v1) as u8);
            v1 = ((r18) as i8).wrapping_sub(1i8);
            if ((r18) as i8).wrapping_sub(1i8) < 0i8 {
                v1 = 0 as i8;
            }
            i -= 1;
        }
        update_ram_word!(game.hdma_object_A[(v3 as i32 >> 1 as i32)], wrapping_add(1));
    };
}
pub unsafe fn HdmaobjPreInstr_HazeColorMathSubscreen_FadedIn(k: u16, game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80 as u8;
    set_ram_word!(game.fx_layer_blending_config_c = 44 as u16);
    if ram_word!(game, door_transition_function) as i32 == 0x82e2db as i32 & 0xffff as i32 {
        set_ram_word!(
            game.hdma_object_pre_instructions[(k as u8 as i32 >> 1i32)] = ((0x88de96i32) as u16)
        );
    }
}
pub unsafe fn HdmaobjPreInstr_HazeColorMathSubscreen_FadingOut(k: u16, game: &mut Game) {
    let mut v1: i8 = 0;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80 as u8;
    set_ram_word!(game.fx_layer_blending_config_c = 44 as u16);
    if ram_word!(game, hdma_object_A[(k as u8 as i32 >> 1i32)]) != 0 {
        let v3: u8 = k as u8;
        let r20: u8 = *((&raw mut game.g_ram as *mut u8).offset(0x1920) as *mut u16 as *mut u8)
            .offset(k as u8 as i32 as isize);
        v1 = *((&raw mut game.g_ram as *mut u8).offset(0x1914) as *mut u16 as *mut u8)
            .offset(k as u8 as i32 as isize) as i8;
        let mut i: i32 = 15 as i32;
        while i >= 0i32 {
            let r18: u8 = v1 as u8;
            *((&raw mut game.g_ram as *mut u8).offset(0x9d00) as *mut u16 as *mut u8)
                .offset(i as u8 as i32 as isize) = (r20) | ((v1) as u8);
            v1 = ((r18) as i8).wrapping_sub(1i8);
            if ((r18) as i8).wrapping_sub(1i8) < 0i8 {
                v1 = 0 as i8;
            }
            i -= 1;
        }
        update_ram_word!(game.hdma_object_A[(v3 as i32 >> 1 as i32)], wrapping_sub(1));
    }
}
static unk_88DF38: SpawnHdmaObject_Args =
    SpawnHdmaObject_Args::new(0 as u8, 0x2c as u8, 0xdf4f as VoidP);
static unk_88DF41: SpawnHdmaObject_Args =
    SpawnHdmaObject_Args::new(0 as u8, 0x2c as u8, 0xdf6b as VoidP);
static unk_88DF4A: SpawnHdmaObject_Args =
    SpawnHdmaObject_Args::new(0x2 as u8, 0x12 as u8, 0xdf77 as VoidP);
pub unsafe fn sub_88DF34(game: &mut Game) {
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88DF38 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn sub_88DF3D(game: &mut Game) {
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88DF41 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn sub_88DF46(game: &mut Game) {
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88DF4A as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn HdmaobjPreInstr_DF94(v0: u16, game: &mut Game) {
    let mut v1: i16 = 0;
    if ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).properties
        as i32
        & 0x200 as i32)
        == 0i32
        && (((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
            as i32
            - ram_word!(game, layer1_x_pos) as i32
            + 64 as i32) as i16)
            >= 0i16
        && (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
            as i32
            - ram_word!(game, layer1_x_pos) as i32
            - 320 as i32
            & 0x8000 as i32
            != 0
        && (((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).y_pos
            as i32
            - ram_word!(game, layer1_y_pos) as i32
            + 16 as i32) as i16)
            >= 0i16
        && {
            v1 = (((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0))
                .y_pos) as i16)
                .wrapping_sub((ram_word!(game, layer1_y_pos)) as i16);
            (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).y_pos
                as i32
                - ram_word!(game, layer1_y_pos) as i32
                - 304 as i32
                & 0x8000 as i32
                != 0
        }
    {
        if v1 as i32 - 40 as i32 & 0x8000 as i32 != 0 {
            let v2: i32 = v0 as i32 >> 1 as i32;
            set_ram_word!(game.hdma_object_instruction_timers[v2] = 1 as u16);
            set_ram_word!(game.hdma_object_instruction_list_pointers[v2] = addr_word_88DF6B as u16);
        } else {
            let v4: i32 = v0 as i32 >> 1 as i32;
            set_ram_word!(game.hdma_object_instruction_timers[v4] = 1 as u16);
            if v1 as i32 - 192 as i32 & 0x8000 as i32 != 0 {
                set_ram_word!(
                    game.hdma_object_instruction_list_pointers[v4] = addr_word_88DF5F as u16
                );
            } else {
                set_ram_word!(
                    game.hdma_object_instruction_list_pointers[v4] = addr_word_88DF65 as u16
                );
            }
        }
    } else {
        let v3: i32 = v0 as i32 >> 1 as i32;
        set_ram_word!(game.hdma_object_instruction_timers[v3] = 1 as u16);
        set_ram_word!(game.hdma_object_instruction_list_pointers[v3] = addr_word_88DF71 as u16);
    };
}
static off_88E04E: [Option<Func_U8>; 7] = unsafe {
    [
        Some(VariaSuitPickup_0_LightBeamAppears as unsafe fn(&mut Game) -> u8),
        Some(VariaSuitPickup_1 as unsafe fn(&mut Game) -> u8),
        Some(VariaSuitPickup_2_LightBeamWidens as unsafe fn(&mut Game) -> u8),
        Some(VariaSuitPickup_3 as unsafe fn(&mut Game) -> u8),
        Some(VariaSuitPickup_4_LightBeamShrinks as unsafe fn(&mut Game) -> u8),
        Some(VariaSuitPickup_5_LightBeamDissipates as unsafe fn(&mut Game) -> u8),
        Some(VariaSuitPickup_6 as unsafe fn(&mut Game) -> u8),
    ]
};
pub unsafe fn HdmaobjPreInstr_VariaSuitPickup(k: u16, game: &mut Game) {
    if off_88E04E[ram_word!(game, substate) as usize].expect("non-null function pointer")(game)
        as i32
        & 1 as i32
        != 0
    {
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) =
            *(&raw mut game.g_ram as *mut u8).offset(0xdf0);
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) =
            *(&raw mut game.g_ram as *mut u8).offset(0xdf1);
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) =
            *(&raw mut game.g_ram as *mut u8).offset(0xdf2);
        set_ram_word!(game.fx_layer_blending_config_c = 18 as u16);
    }
}
static off_88E084: [Option<Func_U8>; 7] = unsafe {
    [
        Some(VariaSuitPickup_0_LightBeamAppears as unsafe fn(&mut Game) -> u8),
        Some(VariaSuitPickup_1 as unsafe fn(&mut Game) -> u8),
        Some(VariaSuitPickup_2_LightBeamWidens as unsafe fn(&mut Game) -> u8),
        Some(GravitySuitPickup_3 as unsafe fn(&mut Game) -> u8),
        Some(VariaSuitPickup_4_LightBeamShrinks as unsafe fn(&mut Game) -> u8),
        Some(VariaSuitPickup_5_LightBeamDissipates as unsafe fn(&mut Game) -> u8),
        Some(GravitySuitPickup_6 as unsafe fn(&mut Game) -> u8),
    ]
};
pub unsafe fn HdmaobjPreInstr_GravitySuitPickup(k: u16, game: &mut Game) {
    if off_88E084[ram_word!(game, substate) as usize].expect("non-null function pointer")(game)
        as i32
        & 1 as i32
        != 0
    {
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) =
            *(&raw mut game.g_ram as *mut u8).offset(0xdf0);
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) =
            *(&raw mut game.g_ram as *mut u8).offset(0xdf1);
        *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) =
            *(&raw mut game.g_ram as *mut u8).offset(0xdf2);
        set_ram_word!(game.fx_layer_blending_config_c = 18 as u16);
    }
}
static g_word_88E0D5: u16 = 0x7878 as u16;
pub unsafe fn VariaSuitPickup_0_LightBeamAppears(game: &mut Game) -> u8 {
    let mut v0: i16 = 0;
    let mut v3: i16 = 0;
    update_ram_word!(game.suit_pickup_light_beam_pos, wrapping_add(8u16));
    v0 = ram_word!(game, suit_pickup_light_beam_pos) as i16;
    let mut v1: u16 = 0 as u16;
    loop {
        set_ram_word!(game.hdma_table_1[(v1 as i32 >> 1i32)] = g_word_88E0D5);
        v1 = (v1).wrapping_add(2u16);
        v0 -= 1;
        if (v0 <= 0i16) {
            break;
        }
    }
    let mut v2: u16 = 510 as u16;
    v3 = ram_word!(game, suit_pickup_light_beam_pos) as i16;
    loop {
        set_ram_word!(game.hdma_table_1[(v2 as i32 >> 1i32)] = g_word_88E0D5);
        v2 = (v2).wrapping_sub(2u16);
        v3 -= 1;
        if (v3 <= 0i16) {
            break;
        }
    }
    if ram_word!(game, suit_pickup_light_beam_pos) as i32 - 128 as i32 & 0x8000 as i32 == 0 {
        update_ram_word!(game.substate, wrapping_add(1));
        set_ram_word!(game.suit_pickup_light_beam_pos = 30840 as u16);
    }
    1 as u8
}
pub unsafe fn VariaSuitPickup_1(game: &mut Game) -> u8 {
    *((&raw mut game.g_ram as *mut u8).offset(0xdee) as *mut u16 as *mut u8).offset(0) =
        ((ram_word!(game, suit_pickup_light_beam_pos)) as u8).wrapping_sub(
            *((&raw mut game.g_ram as *mut u8).offset(0xddc) as *mut u16 as *mut u8).offset(
                (::core::mem::size_of::<u16>())
                    .wrapping_div(::core::mem::size_of::<u8>())
                    .wrapping_sub(1_usize) as isize,
            ),
        );
    let ref mut fresh17 = *((&raw mut game.g_ram as *mut u8).offset(0xdee) as *mut u16 as *mut u8)
        .offset(
            (::core::mem::size_of::<u16>())
                .wrapping_div(::core::mem::size_of::<u8>())
                .wrapping_sub(1_usize) as isize,
        );
    *fresh17 = (*fresh17).wrapping_add(
        *((&raw mut game.g_ram as *mut u8).offset(0xddc) as *mut u16 as *mut u8).offset(
            (::core::mem::size_of::<u16>())
                .wrapping_div(::core::mem::size_of::<u8>())
                .wrapping_sub(1_usize) as isize,
        ),
    );
    let mut i: i32 = 510 as i32;
    while i >= 0i32 {
        set_ram_word!(game.hdma_table_1[(i >> 1i32)] = ram_word!(game, suit_pickup_light_beam_pos));
        i -= 2 as i32;
    }
    if ram_word!(game, suit_pickup_light_beam_pos) as u8 as i32 - 97 as i32 & 0x8000 as i32 != 0 {
        update_ram_word!(game.substate, wrapping_add(1));
        set_ram_word!(game.suit_pickup_light_beam_pos = 0x846c as u16);
    }
    1 as u8
}
pub unsafe fn VariaSuitPickup_2_LightBeamWidens(game: &mut Game) -> u8 {
    let mut v1: i8 = 0;
    let mut v2: i16 = 0;
    let mut v4: i8 = 0;
    let mut v6: i8 = 0;
    let mut v8: i8 = 0;
    let mut v10: i8 = 0;
    AdvanceSuitPickupColorMathSubscreenToWhite(game);
    let v0: bool = ((ram_word!(game, suit_pickup_light_beam_pos)) as i8).wrapping_sub(
        (*((&raw mut game.g_ram as *mut u8).offset(0xddc) as *mut u16 as *mut u8).offset(
            (::core::mem::size_of::<u16>())
                .wrapping_div(::core::mem::size_of::<u8>())
                .wrapping_sub(1_usize) as isize,
        )) as i8,
    ) < 0i8;
    *((&raw mut game.g_ram as *mut u8).offset(0xdee) as *mut u16 as *mut u8).offset(0) =
        ((ram_word!(game, suit_pickup_light_beam_pos)) as u8).wrapping_sub(
            *((&raw mut game.g_ram as *mut u8).offset(0xddc) as *mut u16 as *mut u8).offset(
                (::core::mem::size_of::<u16>())
                    .wrapping_div(::core::mem::size_of::<u8>())
                    .wrapping_sub(1_usize) as isize,
            ),
        );
    if ram_word!(game, suit_pickup_light_beam_pos) as u8 == 0 || v0 {
        set_ram_word!(game.suit_pickup_light_beam_pos = -(256 as i32) as u16);
    } else {
        v1 = ((*((&raw mut game.g_ram as *mut u8).offset(0xddc) as *mut u16 as *mut u8).offset(
            (::core::mem::size_of::<u16>())
                .wrapping_div(::core::mem::size_of::<u8>())
                .wrapping_sub(1_usize) as isize,
        )) as i8)
            .wrapping_add(
                (*((&raw mut game.g_ram as *mut u8).offset(0xdee) as *mut u16 as *mut u8).offset(
                    (::core::mem::size_of::<u16>())
                        .wrapping_div(::core::mem::size_of::<u8>())
                        .wrapping_sub(1_usize) as isize,
                )) as i8,
            );
        if u8::overflowing_add(
            *((&raw mut game.g_ram as *mut u8).offset(0xddc) as *mut u16 as *mut u8).offset(
                (::core::mem::size_of::<u16>())
                    .wrapping_div(::core::mem::size_of::<u8>())
                    .wrapping_sub(1_usize) as isize,
            ),
            *((&raw mut game.g_ram as *mut u8).offset(0xdee) as *mut u16 as *mut u8).offset(
                (::core::mem::size_of::<u16>())
                    .wrapping_div(::core::mem::size_of::<u8>())
                    .wrapping_sub(1_usize) as isize,
            ),
        )
        .1
        {
            v1 = -(1 as i32) as i8;
        }
        *((&raw mut game.g_ram as *mut u8).offset(0xdee) as *mut u16 as *mut u8).offset(
            (::core::mem::size_of::<u16>())
                .wrapping_div(::core::mem::size_of::<u8>())
                .wrapping_sub(1_usize) as isize,
        ) = v1 as u8;
    }
    v2 = 0 as i16;
    let mut v3: u16 = 0 as u16;
    loop {
        v4 = ((ram_word!(game, suit_pickup_light_beam_pos)) as i8).wrapping_sub(
            (*(game.g_rom.offset(crate::rtl::lorom_offset(0x88e3c9)) as *const u8 as *mut u8)
                .offset(v3 as isize)) as i8,
        );
        if v4 < 0i8 {
            v4 = 0 as i8;
        }
        *((&raw mut game.g_ram as *mut u8).offset(0x9800) as *mut u16 as *mut u8)
            .offset(v2 as u16 as i32 as isize) = v4 as u8;
        let v5: u16 = ((v2) as u16).wrapping_add(1u16);
        v6 = ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x88e3c9)) as *const u8 as *mut u8)
            .offset(v3 as isize)) as i8)
            .wrapping_add(
                (*((&raw mut game.g_ram as *mut u8).offset(0xdee) as *mut u16 as *mut u8).offset(
                    (::core::mem::size_of::<u16>())
                        .wrapping_div(::core::mem::size_of::<u8>())
                        .wrapping_sub(1_usize) as isize,
                )) as i8,
            );
        if u8::overflowing_add(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88e3c9)) as *const u8 as *mut u8)
                .offset(v3 as isize),
            *((&raw mut game.g_ram as *mut u8).offset(0xdee) as *mut u16 as *mut u8).offset(
                (::core::mem::size_of::<u16>())
                    .wrapping_div(::core::mem::size_of::<u8>())
                    .wrapping_sub(1_usize) as isize,
            ),
        )
        .1
        {
            v6 = -(1 as i32) as i8;
        }
        *((&raw mut game.g_ram as *mut u8).offset(0x9800) as *mut u16 as *mut u8)
            .offset(v5 as i32 as isize) = v6 as u8;
        v2 = ((v5) as i16).wrapping_add(1i16);
        v3 = v3.wrapping_add(1);
        if (v2 >= 0x100i16) {
            break;
        }
    }
    let mut v7: u16 = (v3).wrapping_sub(1u16);
    loop {
        v8 = ((ram_word!(game, suit_pickup_light_beam_pos)) as i8).wrapping_sub(
            (*(game.g_rom.offset(crate::rtl::lorom_offset(0x88e3c9)) as *const u8 as *mut u8)
                .offset(v7 as isize)) as i8,
        );
        if v8 < 0i8 {
            v8 = 0 as i8;
        }
        *((&raw mut game.g_ram as *mut u8).offset(0x9800) as *mut u16 as *mut u8)
            .offset(v2 as u16 as i32 as isize) = v8 as u8;
        let v9: u16 = ((v2) as u16).wrapping_add(1u16);
        v10 = ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x88e3c9)) as *const u8 as *mut u8)
            .offset(v7 as isize)) as i8)
            .wrapping_add(
                (*((&raw mut game.g_ram as *mut u8).offset(0xdee) as *mut u16 as *mut u8).offset(
                    (::core::mem::size_of::<u16>())
                        .wrapping_div(::core::mem::size_of::<u8>())
                        .wrapping_sub(1_usize) as isize,
                )) as i8,
            );
        if u8::overflowing_add(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x88e3c9)) as *const u8 as *mut u8)
                .offset(v7 as isize),
            *((&raw mut game.g_ram as *mut u8).offset(0xdee) as *mut u16 as *mut u8).offset(
                (::core::mem::size_of::<u16>())
                    .wrapping_div(::core::mem::size_of::<u8>())
                    .wrapping_sub(1_usize) as isize,
            ),
        )
        .1
        {
            v10 = -(1 as i32) as i8;
        }
        *((&raw mut game.g_ram as *mut u8).offset(0x9800) as *mut u16 as *mut u8)
            .offset(v9 as i32 as isize) = v10 as u8;
        v2 = ((v9) as i16).wrapping_add(1i16);
        v7 = v7.wrapping_sub(1);
        if (v2 >= 0x200i16) {
            break;
        }
    }
    update_ram_word!(
        game.suit_pickup_light_beam_widening_speed,
        wrapping_add(96u16)
    );
    if ram_word!(game, suit_pickup_light_beam_pos) == 0xff00u16 {
        update_ram_word!(game.substate, wrapping_add(1));
        update_ram_word!(
            game.suit_pickup_light_beam_widening_speed,
            |fresh21| (fresh21 as i32 >> 1 as i32) as u16
        );
        set_ram_word!(game.suit_pickup_light_beam_pos = 0 as u16);
    }
    1 as u8
}
pub unsafe fn VariaSuitPickup_4_LightBeamShrinks(game: &mut Game) -> u8 {
    if *(&raw mut game.g_ram as *mut u8).offset(0xdf3) != 0 {
        AdvanceSuitPickupColorMathToBlue(game);
    } else {
        AdvanceSuitPickupColorMathToOrange(game);
    }
    update_ram_word!(game.suit_pickup_light_beam_pos, |fresh25| (fresh25 as i32
        + ((ram_word!(game, suit_pickup_light_beam_widening_speed) as i32 & 0xff00 as i32)
            >> 8 as i32))
        as u16);
    let mut v0: i16 = ram_word!(game, suit_pickup_light_beam_pos) as i16;
    let mut v1: u16 = 0 as u16;
    loop {
        set_ram_word!(game.hdma_table_1[(v1 as i32 >> 1i32)] = 255 as u16);
        v1 = (v1).wrapping_add(2u16);
        v0 -= 1;
        if (v0 <= 0i16) {
            break;
        }
    }
    let mut v2: u16 = 510 as u16;
    let mut v3: i16 = ((ram_word!(game, suit_pickup_light_beam_pos)) as i16).wrapping_sub(1i16);
    loop {
        set_ram_word!(game.hdma_table_1[(v2 as i32 >> 1i32)] = 255 as u16);
        v2 = (v2).wrapping_sub(2u16);
        v3 -= 1;
        if (v3 <= 0i16) {
            break;
        }
    }
    update_ram_word!(
        game.suit_pickup_light_beam_widening_speed,
        wrapping_sub(32u16)
    );
    if ram_word!(game, suit_pickup_light_beam_widening_speed) as i32 - 256 as i32 & 0x8000 as i32
        != 0
    {
        set_ram_word!(game.suit_pickup_light_beam_widening_speed = 256 as u16);
    }
    if ram_word!(game, suit_pickup_light_beam_pos) as i32 - 128 as i32 & 0x8000 as i32 == 0 {
        update_ram_word!(game.substate, wrapping_add(1));
        set_ram_word!(game.suit_pickup_light_beam_pos = -(1793 as i32) as u16);
    }
    1 as u8
}
pub unsafe fn VariaSuitPickup_5_LightBeamDissipates(game: &mut Game) -> u8 {
    *((&raw mut game.g_ram as *mut u8).offset(0xdee) as *mut u16 as *mut u8).offset(0) =
        ((ram_word!(game, suit_pickup_light_beam_pos)) as u8).wrapping_add(8u8);
    let ref mut fresh28 = *((&raw mut game.g_ram as *mut u8).offset(0xdee) as *mut u16 as *mut u8)
        .offset(
            (::core::mem::size_of::<u16>())
                .wrapping_div(::core::mem::size_of::<u8>())
                .wrapping_sub(1_usize) as isize,
        );
    *fresh28 = (*fresh28).wrapping_sub(8u8);
    set_ram_word!(game.hdma_table_1[128] = ram_word!(game, suit_pickup_light_beam_pos));
    if ram_word!(game, suit_pickup_light_beam_pos) as u8 as i32 - 112 as i32 & 0x8000 as i32 == 0 {
        update_ram_word!(game.substate, wrapping_add(1));
    }
    1 as u8
}
pub unsafe fn VariaSuitPickup_6(game: &mut Game) -> u8 {
    QueueMusic_Delayed8(3 as u16, game);
    GravitySuitPickup_6(game)
}
pub unsafe fn GravitySuitPickup_6(game: &mut Game) -> u8 {
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x80 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 64 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32 as u8;
    mov24(
        (&raw mut game.g_ram as *mut u8).offset(0xa88) as *mut LongPtr,
        0x980001 as u32,
    );
    *(((&raw mut game.g_ram as *mut u8).offset(0xa8a) as *mut u16 as *mut u8).offset(1)
        as *mut u16) = 0 as u16;
    set_ram_word!(game.demo_input_prev = 0 as u16);
    set_ram_word!(game.demo_input_prev_new = 0 as u16);
    set_ram_word!(game.demo_backup_prev_controller_input = 0 as u16);
    set_ram_word!(game.hdma_table_1[0] = 255 as u16);
    set_ram_word!(game.substate = 0 as u16);
    set_ram_word!(game.suit_pickup_light_beam_pos = 0 as u16);
    *((&raw mut game.g_ram as *mut u8).offset(0xdf0) as *mut u16) = 0 as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0xdf2) as *mut u16) = 0 as u16;
    let v0: i32 = ram_word!(game, hdma_object_index) as i32 >> 1 as i32;
    update_ram_word!(
        game.hdma_object_instruction_list_pointers[v0],
        wrapping_add(2u16)
    );
    set_ram_word!(game.hdma_object_instruction_timers[v0] = 1 as u16);
    CallSomeSamusCode(0xb as u16, game);
    0 as u8
}
pub unsafe fn AdvanceSuitPickupColorMathSubscreenToWhite(game: &mut Game) -> u8 {
    if *(&raw mut game.g_ram as *mut u8).offset(0xdf0) != 63u8 {
        let ref mut fresh0 = *(&raw mut game.g_ram as *mut u8).offset(0xdf0);
        *fresh0 = (*fresh0).wrapping_add(2u8);
        if *(&raw mut game.g_ram as *mut u8).offset(0xdf0) as i32 - 64 as i32 & 0x80 as i32 == 0 {
            *(&raw mut game.g_ram as *mut u8).offset(0xdf0) = 63 as u8;
        }
    }
    if *(&raw mut game.g_ram as *mut u8).offset(0xdf1) != 95u8 {
        let ref mut fresh1 = *(&raw mut game.g_ram as *mut u8).offset(0xdf1);
        *fresh1 = (*fresh1).wrapping_add(2u8);
        if *(&raw mut game.g_ram as *mut u8).offset(0xdf1) as i32 - 96 as i32 & 0x80 as i32 == 0 {
            *(&raw mut game.g_ram as *mut u8).offset(0xdf1) = 95 as u8;
        }
    }
    if *(&raw mut game.g_ram as *mut u8).offset(0xdf2) != 0x9fu8 {
        let ref mut fresh2 = *(&raw mut game.g_ram as *mut u8).offset(0xdf2);
        *fresh2 = (*fresh2).wrapping_add(2u8);
        if *(&raw mut game.g_ram as *mut u8).offset(0xdf2) as i32 + 96 as i32 & 0x80 as i32 == 0 {
            *(&raw mut game.g_ram as *mut u8).offset(0xdf2) = -(97 as i32) as u8;
        }
    }
    1 as u8
}
pub unsafe fn AdvanceSuitPickupColorMathToOrange(game: &mut Game) -> u8 {
    if *(&raw mut game.g_ram as *mut u8).offset(0xdf0) != 63u8 {
        let ref mut fresh6 = *(&raw mut game.g_ram as *mut u8).offset(0xdf0);
        *fresh6 = (*fresh6).wrapping_sub(1);
    }
    if *(&raw mut game.g_ram as *mut u8).offset(0xdf1) != 77u8 {
        let ref mut fresh7 = *(&raw mut game.g_ram as *mut u8).offset(0xdf1);
        *fresh7 = (*fresh7).wrapping_sub(1);
    }
    if *(&raw mut game.g_ram as *mut u8).offset(0xdf2) != 0x83u8 {
        let ref mut fresh8 = *(&raw mut game.g_ram as *mut u8).offset(0xdf2);
        *fresh8 = (*fresh8).wrapping_sub(1);
    }
    1 as u8
}
pub unsafe fn VariaSuitPickup_3(game: &mut Game) -> u8 {
    update_ram_word!(game.equipped_items, |fresh22| (fresh22) | (1u16));
    update_ram_word!(game.collected_items, |fresh23| (fresh23) | (1u16));
    set_ram_word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as u16);
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
    *((&raw mut game.g_ram as *mut u8).offset(0xa26) as *mut u16) =
        *((&raw mut game.g_ram as *mut u8).offset(0xa22) as *mut u16);
    set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
    *((&raw mut game.g_ram as *mut u8).offset(0xa22) as *mut u16) =
        *((&raw mut game.g_ram as *mut u8).offset(0xa1e) as *mut u16);
    Samus_LoadSuitPalette(game);
    update_ram_word!(game.substate, wrapping_add(1));
    1 as u8
}
pub unsafe fn GravitySuitPickup_3(game: &mut Game) -> u8 {
    update_ram_word!(game.equipped_items, |fresh9| (fresh9) | (0x20u16));
    update_ram_word!(game.collected_items, |fresh10| (fresh10) | (0x20u16));
    set_ram_word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as u16);
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
    *((&raw mut game.g_ram as *mut u8).offset(0xa26) as *mut u16) =
        *((&raw mut game.g_ram as *mut u8).offset(0xa22) as *mut u16);
    set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
    *((&raw mut game.g_ram as *mut u8).offset(0xa22) as *mut u16) =
        *((&raw mut game.g_ram as *mut u8).offset(0xa1e) as *mut u16);
    Samus_LoadSuitPalette(game);
    update_ram_word!(game.substate, wrapping_add(1));
    1 as u8
}
pub unsafe fn AdvanceSuitPickupColorMathToBlue(game: &mut Game) -> u8 {
    if *(&raw mut game.g_ram as *mut u8).offset(0xdf0) != 48u8 {
        let ref mut fresh3 = *(&raw mut game.g_ram as *mut u8).offset(0xdf0);
        *fresh3 = (*fresh3).wrapping_sub(1);
    }
    if *(&raw mut game.g_ram as *mut u8).offset(0xdf1) != 73u8 {
        let ref mut fresh4 = *(&raw mut game.g_ram as *mut u8).offset(0xdf1);
        *fresh4 = (*fresh4).wrapping_sub(1);
    }
    if *(&raw mut game.g_ram as *mut u8).offset(0xdf2) != 0x90u8 {
        let ref mut fresh5 = *(&raw mut game.g_ram as *mut u8).offset(0xdf2);
        *fresh5 = (*fresh5).wrapping_sub(1);
    }
    1 as u8
}
pub unsafe fn HdmaobjPreInstr_E449(k: u16, game: &mut Game) {
    if (ram_word!(game, phantom_related_layer_flag) as i32 & 0x4000 as i32) != 0i32 {
        set_ram_word!(game.fx_layer_blending_config_c = 26 as u16);
    } else if *(&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData)
        .offset(3))
    .parameter_1 as *mut u8)
        .offset(0)
        != 0
    {
        if *(&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData)
            .offset(3))
        .parameter_1 as *mut u8)
            .offset(0)
            == 0xffu8
        {
            set_ram_word!(game.fx_layer_blending_config_c = 4 as u16);
            let v0: i32 = ram_word!(game, hdma_object_index) as i32 >> 1 as i32;
            update_ram_word!(
                game.hdma_object_instruction_list_pointers[v0],
                wrapping_add(2u16)
            );
            set_ram_word!(game.hdma_object_instruction_timers[v0] = 1 as u16);
        }
    } else {
        set_ram_word!(game.fx_layer_blending_config_c = 4 as u16);
    };
}
static unk_88E4A0: SpawnHdmaObject_Args =
    SpawnHdmaObject_Args::new(0x42 as u8, 0xf as u8, 0xe4a8 as VoidP);
pub unsafe fn sub_88E487(v0: u16, r22: u16, game: &mut Game) {
    (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(2)).parameter_1 =
        v0;
    (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(3)).ai_var_D =
        0 as u16;
    (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(3)).ai_var_E =
        0 as u16;
    (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(3)).ai_preinstr =
        r22;
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88E4A0 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn HdmaobjInstr_E4BD(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    let mut v2: u32 = 0;
    let mut v3: u32 = 0;
    let mut v4: u32 = 0;
    let mut v5: u32 = 0;
    (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1)).parameter_1 =
        (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(2)).parameter_1;
    if ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(2)).parameter_1
        as i32
        & 1 as i32)
        != 0i32
    {
        set_ram_word!(game.kraid_unk9000 = 192 as u16);
        set_ram_word!(game.g_word_7E9006 = 192 as u16);
        v4 = 12620032 as i32 as u32;
        *(((&raw mut game.g_ram as *mut u8).offset(0x9002) as *mut u16 as *mut u8).offset(1)
            as *mut u16) = *(&raw mut v4 as *mut u16).offset(
            (::core::mem::size_of::<u32>())
                .wrapping_div(::core::mem::size_of::<u16>())
                .wrapping_sub(1_usize) as isize,
        );
        *(((&raw mut game.g_ram as *mut u8).offset(0x9000) as *mut u16 as *mut u8).offset(1)
            as *mut u16) = v4 as u16;
        set_ram_word!(game.g_word_7E9004 = -(28288 as i32) as u16);
        v5 = 12620032 as i32 as u32;
        *(((&raw mut game.g_ram as *mut u8).offset(0x9008) as *mut u16 as *mut u8).offset(1)
            as *mut u16) = *(&raw mut v5 as *mut u16).offset(
            (::core::mem::size_of::<u32>())
                .wrapping_div(::core::mem::size_of::<u16>())
                .wrapping_sub(1_usize) as isize,
        );
        *(((&raw mut game.g_ram as *mut u8).offset(0x9006) as *mut u16 as *mut u8).offset(1)
            as *mut u16) = v5 as u16;
        set_ram_word!(game.g_word_7E900A = addr_loc_889180 as u16);
        set_ram_word!(game.g_word_7E900C = 0 as u16);
    } else {
        set_ram_word!(game.kraid_unk9000 = 160 as u16);
        set_ram_word!(game.g_word_7E9006 = 160 as u16);
        set_ram_word!(game.g_word_7E900C = 160 as u16);
        set_ram_word!(game.g_word_7E900F = 160 as u16);
        set_ram_word!(game.g_word_7E9012 = 160 as u16);
        set_ram_word!(game.g_word_7E9015 = 160 as u16);
        v2 = 10522880 as i32 as u32;
        *(((&raw mut game.g_ram as *mut u8).offset(0x9002) as *mut u16 as *mut u8).offset(1)
            as *mut u16) = *(&raw mut v2 as *mut u16).offset(
            (::core::mem::size_of::<u32>())
                .wrapping_div(::core::mem::size_of::<u16>())
                .wrapping_sub(1_usize) as isize,
        );
        *(((&raw mut game.g_ram as *mut u8).offset(0x9000) as *mut u16 as *mut u8).offset(1)
            as *mut u16) = v2 as u16;
        v3 = 10522880 as i32 as u32;
        *(((&raw mut game.g_ram as *mut u8).offset(0x9008) as *mut u16 as *mut u8).offset(1)
            as *mut u16) = *(&raw mut v3 as *mut u16).offset(
            (::core::mem::size_of::<u32>())
                .wrapping_div(::core::mem::size_of::<u16>())
                .wrapping_sub(1_usize) as isize,
        );
        *(((&raw mut game.g_ram as *mut u8).offset(0x9006) as *mut u16 as *mut u8).offset(1)
            as *mut u16) = v3 as u16;
        *(((&raw mut game.g_ram as *mut u8).offset(0x900c) as *mut u16 as *mut u8).offset(1)
            as *mut u16) = -(28416 as i32) as u16;
        *(((&raw mut game.g_ram as *mut u8).offset(0x9012) as *mut u16 as *mut u8).offset(1)
            as *mut u16) = -(28416 as i32) as u16;
        set_ram_word!(game.g_word_7E9004 = -(28352 as i32) as u16);
        set_ram_word!(game.g_word_7E900A = -(28352 as i32) as u16);
        *(((&raw mut game.g_ram as *mut u8).offset(0x900f) as *mut u16 as *mut u8).offset(1)
            as *mut u16) = -(28352 as i32) as u16;
        *(((&raw mut game.g_ram as *mut u8).offset(0x9015) as *mut u16 as *mut u8).offset(1)
            as *mut u16) = -(28352 as i32) as u16;
        set_ram_word!(game.g_word_7E9018 = 0 as u16);
    }
    let v6: i32 = k as i32 >> 1 as i32;
    set_ram_word!(game.hdma_object_A[v6] = -(2 as i32) as u16);
    set_ram_word!(game.hdma_object_B[v6] = 1 as u16);
    set_ram_word!(game.hdma_object_C[v6] = 0 as u16);
    set_ram_word!(game.hdma_object_D[v6] = 0 as u16);
    hdp
}
pub unsafe fn HdmaobjPreInstr_E567(v0: u16, game: &mut Game) {
    let mut v7: i16 = 0;
    let mut j: u16 = 0;
    let mut v8: u16 = 0;
    let mut r28: u16 = 0;
    let mut r30: u16 = 0;
    if (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1)).parameter_1
        != 0
    {
        if ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1))
            .parameter_1 as i32
            & 1 as i32)
            != 0i32
        {
            r28 = 4 as u16;
            r30 = 128 as u16;
        } else {
            r28 = 8 as u16;
            r30 = 64 as u16;
        }
        let v2: i32 = v0 as i32 >> 1 as i32;
        let ref mut fresh79 =
            *((&raw mut game.g_ram as *mut u8).offset(0x1920) as *mut u16).offset(v2 as isize);
        let fresh80 = *fresh79;
        *fresh79 = (*fresh79).wrapping_sub(1);
        if fresh80 == 1u16 {
            set_ram_word!(game.hdma_object_B[v2] = 1 as u16);
            set_ram_word!(
                game.hdma_object_A[v2] = (2 as i32
                    * (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData)
                        .offset(3))
                    .ai_preinstr as i32
                    + ram_word!(game, hdma_object_A[v2]) as i32
                    & 0x1ff as i32) as u16
            );
        }
        let mut r20: u16 = ram_word!(game, hdma_object_A[v2]);
        let mut v4: u16 = 0 as u16;
        loop {
            let v11: u16 = v4;
            let v5: u16 = r20;
            let v6: i32 = r20 as i32 >> 1 as i32;
            v7 = kSinCosTable8bit_Sext[(v6 + 64 as i32) as usize];
            if v7 < 0i16 {
                let mut r18: u16 = -(v7 as i32) as u16;
                let mut r22: u16 = (Mult8x8(
                    -(v7 as i8 as i32) as u8,
                    (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(3))
                        .ai_var_D as u8,
                ) as i32
                    >> 8 as i32) as u16;
                r22 = (r22).wrapping_add(Mult8x8(
                    *(&raw mut r18 as *mut u8).offset(
                        (::core::mem::size_of::<u16>())
                            .wrapping_div(::core::mem::size_of::<u8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                    (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(3))
                        .ai_var_D as u8,
                ));
                let r24: u16 = Mult8x8(
                    r18 as u8,
                    *(&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData)
                        .offset(3))
                    .ai_var_D as *mut u8)
                        .offset(
                            (::core::mem::size_of::<u16>())
                                .wrapping_div(::core::mem::size_of::<u8>())
                                .wrapping_sub(1_usize) as isize,
                        ),
                );
                let mult: u8 = Mult8x8(
                    *(&raw mut r18 as *mut u8).offset(
                        (::core::mem::size_of::<u16>())
                            .wrapping_div(::core::mem::size_of::<u8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                    *(&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData)
                        .offset(3))
                    .ai_var_D as *mut u8)
                        .offset(
                            (::core::mem::size_of::<u16>())
                                .wrapping_div(::core::mem::size_of::<u8>())
                                .wrapping_sub(1_usize) as isize,
                        ),
                ) as u8;
                r22 = (r22).wrapping_add(r24);
                r18 =
                    ((r22 as i32 + ((mult as i32) << 8 as i32) & 0xff00 as i32) >> 8 as i32) as u16;
                r20 = (r28 as i32 + v5 as i32 & 0x1ff as i32) as u16;
                v8 = v11;
                *(((&raw mut game.g_ram as *mut u8).offset(0x9100) as *mut u16 as *mut u8)
                    .offset(v11 as i32 as isize) as *mut u16) =
                    (ram_word!(game, reg_BG2HOFS)).wrapping_sub(r18);
            } else {
                let mut r18_0: u16 = kSinCosTable8bit_Sext[(v6 + 64 as i32) as usize] as u16;
                let mut r22_0: u16 = (Mult8x8(
                    v7 as u8,
                    (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(3))
                        .ai_var_D as u8,
                ) as i32
                    >> 8 as i32) as u16;
                r22_0 = (r22_0).wrapping_add(Mult8x8(
                    *(&raw mut r18_0 as *mut u8).offset(
                        (::core::mem::size_of::<u16>())
                            .wrapping_div(::core::mem::size_of::<u8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                    (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(3))
                        .ai_var_D as u8,
                ));
                let r24_0: u16 = Mult8x8(
                    r18_0 as u8,
                    *(&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData)
                        .offset(3))
                    .ai_var_D as *mut u8)
                        .offset(
                            (::core::mem::size_of::<u16>())
                                .wrapping_div(::core::mem::size_of::<u8>())
                                .wrapping_sub(1_usize) as isize,
                        ),
                );
                let mult_0: u8 = Mult8x8(
                    *(&raw mut r18_0 as *mut u8).offset(
                        (::core::mem::size_of::<u16>())
                            .wrapping_div(::core::mem::size_of::<u8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                    *(&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData)
                        .offset(3))
                    .ai_var_D as *mut u8)
                        .offset(
                            (::core::mem::size_of::<u16>())
                                .wrapping_div(::core::mem::size_of::<u8>())
                                .wrapping_sub(1_usize) as isize,
                        ),
                ) as u8;
                r22_0 = (r22_0).wrapping_add(r24_0);
                r18_0 = ((r22_0 as i32 + ((mult_0 as i32) << 8 as i32) & 0xff00 as i32) >> 8 as i32)
                    as u16;
                r20 = (r28 as i32 + v5 as i32 & 0x1ff as i32) as u16;
                v8 = v11;
                *(((&raw mut game.g_ram as *mut u8).offset(0x9100) as *mut u16 as *mut u8)
                    .offset(v11 as i32 as isize) as *mut u16) =
                    (r18_0).wrapping_add(ram_word!(game, reg_BG2HOFS));
            }
            v4 = (v8).wrapping_add(2u16);
            if (((v4) as i16).wrapping_sub((r30) as i16) >= 0i16) {
                break;
            }
        }
        if ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1))
            .parameter_1 as i32
            & 1 as i32)
            != 0i32
        {
            let mut i: i32 = 126 as i32;
            while i >= 0i32 {
                *(((&raw mut game.g_ram as *mut u8).offset(0x9180) as *mut u16 as *mut u8)
                    .offset(i as isize) as *mut u16) =
                    (ram_word!(game, reg_BG2HOFS) as i32 + ram_word!(game, reg_BG2HOFS) as i32
                        - *(((&raw mut game.g_ram as *mut u8).offset(0x9100) as *mut u16 as *mut u8)
                            .offset(i as isize) as *mut u16) as i32) as u16;
                i -= 2 as i32;
            }
        } else {
            j = 62 as u16;
            while (j as i32 & 0x8000 as i32) == 0i32 {
                *(((&raw mut game.g_ram as *mut u8).offset(0x9140) as *mut u16 as *mut u8)
                    .offset(j as i32 as isize) as *mut u16) = (ram_word!(game, reg_BG2HOFS) as i32
                    + ram_word!(game, reg_BG2HOFS) as i32
                    - *(((&raw mut game.g_ram as *mut u8).offset(0x9100) as *mut u16 as *mut u8)
                        .offset(j as i32 as isize) as *mut u16) as i32)
                    as u16;
                j = (j).wrapping_sub(2u16);
            }
        }
    } else {
        let v1: i32 = ram_word!(game, hdma_object_index) as i32 >> 1 as i32;
        update_ram_word!(
            game.hdma_object_instruction_list_pointers[v1],
            wrapping_add(2u16)
        );
        set_ram_word!(game.hdma_object_instruction_timers[v1] = 1 as u16);
    };
}
static unk_88E722: SpawnHdmaObject_Args =
    SpawnHdmaObject_Args::new(0 as u8, 0x2c as u8, 0xe727 as VoidP);
static unk_88E74C: SpawnHdmaObject_Args =
    SpawnHdmaObject_Args::new(0x41 as u8, 0x26 as u8, 0xe751 as VoidP);
pub unsafe fn MotherBrainRisingHdmaObject(game: &mut Game) -> u16 {
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88E722 as *const ::core::ffi::c_void,
        game,
    )
}
pub unsafe fn SpawnMotherBrainRainbowBeamHdma(game: &mut Game) -> u16 {
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88E74C as *const ::core::ffi::c_void,
        game,
    )
}
pub unsafe fn InitializeRainbowBeam(game: &mut Game) {
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) = 32 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) = 71 as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) = 0x8f as u8;
    set_ram_word!(game.fx_layer_blending_config_c = 36 as u16);
    *(&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(0) = 100 as u8;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(1) as *mut u8 as *mut u16) =
        -(25344 as i32) as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(3) as *mut u8 as *mut u16) = 0 as u16;
    set_ram_word!(game.hdma_table_2[0] = 0 as u16);
    set_ram_word!(game.hdma_table_2[1] = -(32736 as i32) as u16);
    set_ram_word!(game.hdma_table_2[2] = 0 as u16);
    MotherBrain_CalculateRainbowBeamHdma(game);
}
pub unsafe fn HdmaobjPreInstr_E7BC(k: u16, game: &mut Game) {
    if ram_word!(game, game_state) == 19u16 {
        *((&raw mut game.g_ram as *mut u8).offset(0x18b4) as *mut u16)
            .offset((ram_word!(game, hdma_object_index) as i32 >> 1 as i32) as isize) = 0 as u16;
    } else {
        set_ram_word!(game.fx_layer_blending_config_c = 36 as u16);
        if ram_word!(game, game_state) != 27u16 {
            MotherBrain_CalculateRainbowBeamHdma(game);
            sub_88E7ED(game);
        }
    };
}
pub unsafe fn sub_88E7ED(game: &mut Game) {
    let mut v0: u16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x88e833)) as *const u8
        as *mut u16)
        .offset((ram_word!(game, hdma_object_A[0]) as i32 >> 1 as i32) as isize);
    if (v0 as i32 & 0x8000 as i32) == 0i32 {
        update_ram_word!(game.hdma_object_A[0], wrapping_add(1));
        update_ram_word!(game.hdma_object_A[0], wrapping_add(1));
        update_ram_word!(game.hdma_object_A[0], wrapping_add(1));
        update_ram_word!(game.hdma_object_A[0], wrapping_add(1));
    } else {
        set_ram_word!(game.hdma_object_A[0] = 0 as u16);
        v0 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x88e833)) as *const u8 as *mut u16)
            .offset(0);
    }
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(0) =
        (v0 as i32 & 0x1f as i32 | 0x20 as i32) as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(1) =
        (v0 as i32 >> 5 as i32 & 0x1f as i32 | 0x40 as i32) as u8;
    *(&raw mut game.g_ram as *mut u8).offset(0x74).offset(2) =
        ((v0 as i32 >> 2 as i32) as u16 as i32 >> 8 as i32 & 0x1f as i32 | 0x80 as i32) as u8;
}
pub unsafe fn SpawnMorphBallEyeBeamHdma(game: &mut Game) {
    static unk_88E8E4: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x41 as u8, 0x26 as u8, 0xe8ec as VoidP);
    SpawnHdmaObject(
        0x88 as u8,
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
    static unk_88EB6B: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0 as u8, 0x31 as u8, 0xeb85 as VoidP);
    static unk_88EB63: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0 as u8, 0x32 as u8, 0xeb73 as VoidP);
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88EB63 as *const ::core::ffi::c_void,
        game,
    );
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88EB6B as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn HdmaobjInsr_ConfigTitleSequenceGradientHDMA(
    k: u16,
    hdp: *const u8,
    game: &mut Game,
) -> *const u8 {
    *(&raw mut game.g_ram as *mut u8).offset(0x6f) = 0 as u8;
    ConfigureTitleSequenceGradientHDMA(game);
    hdp
}
pub unsafe fn HdmaobjPreInstr_Backdrop_TitleSequenceGradient(k: u16, game: &mut Game) {
    ConfigureTitleSequenceGradientHDMA(game);
    if ram_word!(game, cinematic_function) as i32 == 0x8b9b68 as i32 & 0xffff as i32 {
        let v1: i32 = ram_word!(game, hdma_object_index) as i32 >> 1 as i32;
        update_ram_word!(
            game.hdma_object_instruction_list_pointers[v1],
            wrapping_add(2u16)
        );
        set_ram_word!(game.hdma_object_instruction_timers[v1] = 1 as u16);
    }
}
pub unsafe fn HdmaobjPreInstr_ColorMathControlB_TitleGradient(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_function) as i32 == 0x8b9b68 as i32 & 0xffff as i32 {
        let v1: i32 = ram_word!(game, hdma_object_index) as i32 >> 1 as i32;
        update_ram_word!(
            game.hdma_object_instruction_list_pointers[v1],
            wrapping_add(2u16)
        );
        set_ram_word!(game.hdma_object_instruction_timers[v1] = 1 as u16);
    }
}
pub unsafe fn SpawnIntroCutsceneCrossfadeHdma(game: &mut Game) {
    static unk_88EBFB: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0 as u8, 0x31 as u8, 0xec03 as VoidP);
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88EBFB as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn HdmaobjPreInstr_IntroCutsceneCrossfade(k: u16, game: &mut Game) {
    if ram_word!(game, eproj_x_pos[0]) == 1u16 {
        let v1: i32 = ram_word!(game, hdma_object_index) as i32 >> 1 as i32;
        update_ram_word!(
            game.hdma_object_instruction_list_pointers[v1],
            wrapping_add(2u16)
        );
        set_ram_word!(game.hdma_object_instruction_timers[v1] = 1 as u16);
    }
}
pub unsafe fn CinematicFunction_Intro_Func133(game: &mut Game) {
    let mut v0: u32 = 0;
    let mut v1: u32 = 0;
    set_ram_word!(game.g_word_7E0D9C = 1 as u16);
    *((&raw mut game.g_ram as *mut u8).offset(0xd9e) as *mut u16) = 0x4000 as u16;
    set_ram_word!(game.loop_counter_transfer_enemies_to_vram = 8 as u16);
    set_ram_word!(game.button_config_shoot_x_saved = 192 as u16);
    set_ram_word!(game.button_config_itemcancel_y_saved = 192 as u16);
    v0 = 12621824 as i32 as u32;
    *(((&raw mut game.g_ram as *mut u8).offset(0xd84) as *mut u16 as *mut u8).offset(1)
        as *mut u16) = *(&raw mut v0 as *mut u16).offset(
        (::core::mem::size_of::<u32>())
            .wrapping_div(::core::mem::size_of::<u16>())
            .wrapping_sub(1_usize) as isize,
    );
    *(((&raw mut game.g_ram as *mut u8).offset(0xd82) as *mut u16 as *mut u8).offset(1)
        as *mut u16) = v0 as u16;
    set_ram_word!(game.button_config_run_b_saved = -(26496 as i32) as u16);
    v1 = 12621824 as i32 as u32;
    *(((&raw mut game.g_ram as *mut u8).offset(0xd8a) as *mut u16 as *mut u8).offset(1)
        as *mut u16) = *(&raw mut v1 as *mut u16).offset(
        (::core::mem::size_of::<u32>())
            .wrapping_div(::core::mem::size_of::<u16>())
            .wrapping_sub(1_usize) as isize,
    );
    *(((&raw mut game.g_ram as *mut u8).offset(0xd88) as *mut u16 as *mut u8).offset(1)
        as *mut u16) = v1 as u16;
    set_ram_word!(game.button_config_aim_down_L_saved = -(26496 as i32) as u16);
    set_ram_word!(game.button_config_aim_up_R_saved = 0 as u16);
    static unk_88EC82: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x42 as u8, 0x11 as u8, 0xec8a as VoidP);
    SpawnHdmaObject(
        0x88 as u8,
        &raw const unk_88EC82 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn HdmaobjInstr_EC9F_ClearVars(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    let v2: i32 = k as i32 >> 1 as i32;
    set_ram_word!(game.hdma_object_A[v2] = -(2 as i32) as u16);
    set_ram_word!(game.hdma_object_B[v2] = 1 as u16);
    set_ram_word!(game.hdma_object_C[v2] = 0 as u16);
    set_ram_word!(game.hdma_object_D[v2] = 0 as u16);
    hdp
}
pub unsafe fn HdmaobjPreInstr_ECB6(k: u16, game: &mut Game) {
    let mut v7: i16 = 0;
    let mut v8: u16 = 0;
    if ram_word!(game, g_word_7E0D9C) != 0 {
        let r28: u16 = 4 as u16;
        let n: i32 = 128 as i32;
        let v3: i32 = k as i32 >> 1 as i32;
        set_ram_word!(
            game.hdma_object_A[v3] = (2 as i32
                * ram_word!(game, loop_counter_transfer_enemies_to_vram) as i32
                + ram_word!(game, hdma_object_A[v3]) as i32
                & 0x1ff as i32) as u16
        );
        let mut r20: u16 = ram_word!(game, hdma_object_A[v3]);
        let mut v4: u16 = 0 as u16;
        loop {
            let v10: u16 = v4;
            let v5: u16 = r20;
            let v6: i32 = r20 as i32 >> 1 as i32;
            v7 = kSinCosTable8bit_Sext[(v6 + 64 as i32) as usize];
            if v7 < 0i16 {
                let mut r18: u16 = -(v7 as i32) as u16;
                let mut r22: u16 = (Mult8x8(
                    -(v7 as i8 as i32) as u8,
                    *(&raw mut game.g_ram as *mut u8).offset(0xd9e),
                ) as i32
                    >> 8 as i32) as u16;
                r22 = (r22).wrapping_add(Mult8x8(
                    *(&raw mut r18 as *mut u8).offset(
                        (::core::mem::size_of::<u16>())
                            .wrapping_div(::core::mem::size_of::<u8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                    *(&raw mut game.g_ram as *mut u8).offset(0xd9e),
                ));
                let r24: u16 = Mult8x8(r18 as u8, *(&raw mut game.g_ram as *mut u8).offset(0xd9f));
                let mult: u8 = Mult8x8(
                    *(&raw mut r18 as *mut u8).offset(
                        (::core::mem::size_of::<u16>())
                            .wrapping_div(::core::mem::size_of::<u8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                    *(&raw mut game.g_ram as *mut u8).offset(0xd9f),
                ) as u8;
                r22 = (r22).wrapping_add(r24);
                r18 =
                    ((r22 as i32 + ((mult as i32) << 8 as i32) & 0xff00 as i32) >> 8 as i32) as u16;
                r20 = (r28 as i32 + v5 as i32 & 0x1ff as i32) as u16;
                v8 = v10;
                set_ram_word!(
                    game.hdma_table_1[(v10 as i32 >> 1i32)] =
                        (ram_word!(game, reg_BG3HOFS)).wrapping_sub(r18)
                );
            } else {
                let mut r18_0: u16 = kSinCosTable8bit_Sext[(v6 + 64 as i32) as usize] as u16;
                let mut r22_0: u16 =
                    (Mult8x8(v7 as u8, *(&raw mut game.g_ram as *mut u8).offset(0xd9e)) as i32
                        >> 8 as i32) as u16;
                r22_0 = (r22_0).wrapping_add(Mult8x8(
                    *(&raw mut r18_0 as *mut u8).offset(
                        (::core::mem::size_of::<u16>())
                            .wrapping_div(::core::mem::size_of::<u8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                    *(&raw mut game.g_ram as *mut u8).offset(0xd9e),
                ));
                let r24_0: u16 =
                    Mult8x8(r18_0 as u8, *(&raw mut game.g_ram as *mut u8).offset(0xd9f));
                let mult_0: u8 = Mult8x8(
                    *(&raw mut r18_0 as *mut u8).offset(
                        (::core::mem::size_of::<u16>())
                            .wrapping_div(::core::mem::size_of::<u8>())
                            .wrapping_sub(1_usize) as isize,
                    ),
                    *(&raw mut game.g_ram as *mut u8).offset(0xd9f),
                ) as u8;
                r22_0 = (r22_0).wrapping_add(r24_0);
                r18_0 = ((r22_0 as i32 + ((mult_0 as i32) << 8 as i32) & 0xff00 as i32) >> 8 as i32)
                    as u16;
                r20 = (r28 as i32 + v5 as i32 & 0x1ff as i32) as u16;
                v8 = v10;
                set_ram_word!(
                    game.hdma_table_1[(v10 as i32 >> 1i32)] =
                        (r18_0).wrapping_add(ram_word!(game, reg_BG3HOFS))
                );
            }
            v4 = (v8).wrapping_add(2u16);
            if (((v4 as i32 - n) as i16) >= 0i16) {
                break;
            }
        }
        let mut i: i32 = 126 as i32;
        while i >= 0i32 {
            set_ram_word!(
                game.hdma_table_1[((i >> 1i32) + 64i32)] = (ram_word!(game, reg_BG3HOFS) as i32
                    + ram_word!(game, reg_BG3HOFS) as i32
                    - ram_word!(game, hdma_table_1[(i >> 1i32)]) as i32)
                    as u16
            );
            i -= 2 as i32;
        }
    } else {
        let v2: i32 = ram_word!(game, hdma_object_index) as i32 >> 1 as i32;
        update_ram_word!(
            game.hdma_object_instruction_list_pointers[v2],
            wrapping_add(2u16)
        );
        set_ram_word!(game.hdma_object_instruction_timers[v2] = 1 as u16);
    };
}
pub unsafe fn CallHdmaobjPreInstr(ea: u32, k: u16, game: &mut Game) {
    match ea {
        8491739 => (),
        8946872 => (),
        10785852 => (),
        9556607 => HdmaobjPreInstr_XraySetup(k, game),
        8947439 => HdmaobjPreInstr_Xray(k, game),
        8947506 => HdmaobjPreInstr_XrayFunc0_NoBeam(k, game),
        8947540 => HdmaobjPreInstr_XrayFunc1_BeamWidening(k, game),
        8947627 => HdmaobjPreInstr_XrayFunc2_FullBeam(k, game),
        8948020 => HdmaobjPreInstr_XrayFunc3_DeactivateBeam(k, game),
        8948154 => HdmaobjPreInstr_XrayFunc4_DeactivateBeam(k, game),
        8948232 => HdmaobjPreInstr_XrayFunc5_DeactivateBeam(k, game),
        8948623 => HdmaobjPreInstr_PowerBombExplode_SetWindowConf(k, game),
        8948632 => HdmaobjPreInstr_PowerBombExplode_Stage5_Afterglow(k, game),
        8949225 => HdmaobjPreInstr_PowerBombExplode_ExplosionYellow(k, game),
        8949426 => HdmaobjPreInstr_PowerBombExplode_ExplosionWhite(k, game),
        8949983 => HdmaobjPreInstr_PowerBombExplode_PreExplosionWhite(k, game),
        8950184 => HdmaobjPreInstr_PowerBombExplode_PreExplosionYellow(k, game),
        8954681 => HdmaobjPreInstr_CrystalFlash_CustomLayerBlend(k, game),
        8954717 => HdmaobjPreInstr_CrystalFlash_Stage2_AfterGlow(k, game),
        8955218 => HdmaobjPreInstr_CrystalFlash_Stage1_Explosion(k, game),
        8955459 => HdmaobjPreInstr_FxType22_BG3Yscroll(k, game),
        8955507 => HdmaobjPreInstr_BG3Xscroll(k, game),
        8957362 => HdmaobjPreInstr_SkyLandBG2Xscroll(k, game),
        8957370 => HdmaobjPreInstr_SkyLandBG2Xscroll2(k, game),
        8957378 => HdmaobjPreInstr_SkyLandBG2XscrollInner(k, game),
        8958140 => HdmaobjPreInstr_FirefleaBG3XScroll(k, game),
        8958896 => HdmaobjPreInstr_LavaAcidBG3YScroll(k, game),
        8959189 => HdmaobjPreInstr_LavaAcidBG2YScroll(k, game),
        8963214 => HdmaobjPreInstr_WaterBG3XScroll(k, game),
        8963465 => HdmaobjPreInstr_WaterBG2XScroll(k, game),
        8963556 => HdmaobjPreInstr_WaterBG2XScroll_Func2(k, game),
        8963638 => HdmaobjPreInstr_WaterBG2XScroll_Func1(k, game),
        8968609 => HdmaobjPreInstr_RainBg3Scroll(k, game),
        8968775 => HdmaobjPreInstr_SporesBG3Xscroll(k, game),
        8969014 => HdmaobjPreInstr_FogBG3Scroll(k, game),
        8969175 => HdmaobjPreInstr_CheckLotsOfEventsHappened(k, game),
        8969251 => HdmaobjPreInstr_DC23(k, game),
        8969321 => HdmaobjPreInstr_DC69(k, game),
        8969402 => HdmaobjPreInstr_DCBA(k, game),
        8969539 => HdmaobjPreInstr_BombTorizoHazeColorMathBgColor(k, game),
        8969744 => HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyAlive(k, game),
        8969749 => HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyDead(k, game),
        8969773 => HdmaobjPreInstr_HazeColorMathSubscreen_FadingIn(k, game),
        8969844 => HdmaobjPreInstr_HazeColorMathSubscreen_FadedIn(k, game),
        8969878 => HdmaobjPreInstr_HazeColorMathSubscreen_FadingOut(k, game),
        8970132 => HdmaobjPreInstr_DF94(k, game),
        8970278 => HdmaobjPreInstr_VariaSuitPickup(k, game),
        8970332 => HdmaobjPreInstr_GravitySuitPickup(k, game),
        8971337 => HdmaobjPreInstr_E449(k, game),
        8971623 => HdmaobjPreInstr_E567(k, game),
        8972220 => HdmaobjPreInstr_E7BC(k, game),
        fnHdmaobjPreInstr_MorphBallEyeBeamWidening => {
            HdmaobjPreInstr_MorphBallEyeBeamWidening(k, game);
        }
        fnHdmaobjPreInstr_MorphBallEyeBeamFull => {
            HdmaobjPreInstr_MorphBallEyeBeamFull(k, game);
        }
        fnHdmaobjPreInstr_MorphBallEyeBeamFading => {
            HdmaobjPreInstr_MorphBallEyeBeamFading(k, game);
        }
        8973232 => HdmaobjPreInstr_Backdrop_TitleSequenceGradient(k, game),
        8973266 => HdmaobjPreInstr_ColorMathControlB_TitleGradient(k, game),
        8973341 => HdmaobjPreInstr_IntroCutsceneCrossfade(k, game),
        8972070 => (),
        8973494 => HdmaobjPreInstr_ECB6(k, game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CallHdmaobjInstr(ea: u32, k: u16, j: *const u8, game: &mut Game) -> *const u8 {
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
            ::core::ptr::null::<u8>()
        }
    }
}
