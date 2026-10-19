pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_87::*;
use crate::game::sm_80::{
    CheckBossBitForCurArea, CheckEventHappened, NextRandom, QueueSfx2_Max6, SetEventHappened,
};
use crate::game::sm_84::SpawnHardcodedPlm;
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_87::SpawnAnimtiles;
use crate::game::sm_88::{
    FxTypeFunc_20, FxTypeFunc_2C_Haze, RoomCode_ScrollingSkyLand, RoomMainAsm_ScrollingSkyOcean,
    RoomSetupAsm_ScrollingSkyOcean,
};
use crate::game::sm_89::RoomCode_CeresElevatorShaft;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game::sm_94::Samus_MoveDown_NoSolidColl;
use crate::game::sm_a6::HandleCeresRidleyGetawayCutscene;
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::game_state::Game;
use crate::ram_map::{ram_word, set_ram_word};
use crate::rtl::{mov24, Mult8x8, RomPtr, Unreachable, WriteReg};
pub use crate::snes::registers::*;
extern "C" {
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;

}
pub use crate::types::EnemyData;
pub use crate::types::{
    __int16_t, __int32_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32, int32_t,
    uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, CoroutineRet, LongPtr, SnesRegs, VoidP,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct RoomDefStateSelect_E6E5_Finish {
    pub code_ptr: VoidP,
}
pub use crate::types::Consts_87;
pub use crate::types::SpawnHardcodedPlmArgs;
pub use crate::types::{Consts_86, DoorDef, RoomDefRoomstate};
pub const kCoroutineNone: i32 = 0;
#[inline]
unsafe fn RomPtr_8F(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0x8f0000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn get_DoorDef(a: uint16, game: &mut Game) -> *mut DoorDef {
    RomPtr((0x830000u32) | ((a) as u32), game) as *mut DoorDef
}
#[inline]
unsafe fn get_RoomDefRoomstate(a: uint16, game: &mut Game) -> *mut RoomDefRoomstate {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomDefRoomstate
}
#[inline]
unsafe fn get_RoomDefStateSelect_E6E5_Finish(
    a: uint16,
    game: &mut Game,
) -> *mut RoomDefStateSelect_E6E5_Finish {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomDefStateSelect_E6E5_Finish
}
pub const fnRoomSetup_AfterSavingAnimals: uint32 = 9408916;
pub const fnRoomSetup_AutoDestroyWallAfterEscape: uint32 = 9408937;
pub const fnRoomSetup_TurnWallIntoShotblocks: uint32 = 9408946;
pub const fnnullsub_132: uint32 = 9408955;
pub const fnnullsub_133: uint32 = 9408956;
pub const fnRoomSetup_ShakeDuringEscape: uint32 = 9408957;
pub const fnRoomSetup_ScrollingSkyLand: uint32 = 9408969;
pub const fnRoomSetup_ScrollingSkyOcean: uint32 = 9408974;
pub const fnnullsub_134: uint32 = 9408979;
pub const fnnullsub_135: uint32 = 9408980;
pub const fnnullsub_136: uint32 = 9408981;
pub const fnnullsub_137: uint32 = 9408982;
pub const fnRoomSetup_RunStatueUnlockAnims: uint32 = 9408983;
pub const fnnullsub_138: uint32 = 9409012;
pub const fnnullsub_139: uint32 = 9409013;
pub const fnnullsub_140: uint32 = 9409014;
pub const fnnullsub_141: uint32 = 9409015;
pub const fnDoorCode_StartWreckedShipTreadmillWest: uint32 = 9419121;
pub const fnDoorCode_Scroll6_Green: uint32 = 9419137;
pub const fnDoorCode_Scroll0_Blue: uint32 = 9419148;
pub const fnDoorCode_Scroll13_Blue: uint32 = 9419159;
pub const fnDoorCode_Scroll_4Blue_8Green: uint32 = 9419170;
pub const fnDoorCode_Scroll_8toB_Red: uint32 = 9419187;
pub const fnDoorCode_Scroll_LotsRed: uint32 = 9419210;
pub const fnDoorCode_Scroll_1_4_Green: uint32 = 9419249;
pub const fnDoorCode_Scroll_2_Blue: uint32 = 9419264;
pub const fnDoorCode_Scroll_17_Blue: uint32 = 9419275;
pub const fnDoorCode_Scroll_4_Blue: uint32 = 9419286;
pub const fnDoorCode_Scroll_6_Green: uint32 = 9419297;
pub const fnDoorCode_Scroll_3_Green: uint32 = 9419308;
pub const fnDoorCode_SetScroll_0: uint32 = 9420039;
pub const fnDoorCode_SetScroll_1: uint32 = 9420054;
pub const fnDoorCode_SetScroll_2: uint32 = 9420069;
pub const fnDoorCode_SetScroll_3: uint32 = 9420080;
pub const fnDoorCode_SetScroll_4: uint32 = 9420095;
pub const fnDoorCode_SetScroll_5: uint32 = 9420112;
pub const fnDoorCode_SetScroll_6: uint32 = 9420123;
pub const fnDoorCode_SetScroll_7: uint32 = 9420140;
pub const fnDoorCode_SetScroll_8: uint32 = 9420151;
pub const fnDoorCode_SetScroll_9: uint32 = 9420170;
pub const fnDoorCode_SetScroll_10: uint32 = 9420181;
pub const fnDoorCode_SetScroll_11: uint32 = 9420192;
pub const fnDoorCode_SetScroll_12: uint32 = 9420207;
pub const fnDoorCode_SetScroll_13: uint32 = 9420224;
pub const fnDoorCode_SetScroll_14: uint32 = 9420241;
pub const fnDoorCode_SetScroll_15: uint32 = 9420258;
pub const fnDoorCode_SetScroll_16: uint32 = 9420273;
pub const fnDoorCode_SetScroll_17: uint32 = 9420288;
pub const fnDoorCode_SetScroll_18: uint32 = 9420299;
pub const fnDoorCode_SetScroll_19: uint32 = 9420314;
pub const fnDoorCode_SetScroll_20: uint32 = 9420325;
pub const fnDoorCode_SetScroll_21: uint32 = 9420342;
pub const fnDoorCode_SetScroll_22: uint32 = 9420702;
pub const fnDoorCode_SetScroll_23: uint32 = 9420731;
pub const fnDoorCode_SetScroll_24: uint32 = 9420762;
pub const fnDoorCode_SetScroll_25: uint32 = 9420777;
pub const fnDoorCode_SetScroll_26: uint32 = 9420794;
pub const fnDoorCode_SetScroll_28: uint32 = 9420822;
pub const fnDoorCode_SetScroll_29: uint32 = 9420837;
pub const fnDoorCode_SetScroll_30: uint32 = 9420858;
pub const fnDoorCode_SetScroll_31: uint32 = 9420875;
pub const fnDoorCode_SetScroll_32: uint32 = 9420886;
pub const fnDoorCode_SetScroll_33: uint32 = 9420903;
pub const fnDoorCode_SetScroll_34: uint32 = 9420920;
pub const fnDoorCode_SetScroll_35: uint32 = 9420937;
pub const fnDoorCode_SetScroll_36: uint32 = 9420962;
pub const fnDoorCode_SetScroll_37: uint32 = 9420973;
pub const fnDoorCode_SetScroll_38: uint32 = 9420994;
pub const fnDoorCode_SetScroll_39: uint32 = 9421011;
pub const fnDoorCode_SetScroll_40: uint32 = 9421022;
pub const fnDoorCode_SetScroll_41: uint32 = 9421039;
pub const fnDoorCode_SetScroll_42: uint32 = 9421050;
pub const fnDoorCode_SetScroll_43: uint32 = 9421067;
pub const fnRoomCode_ScrollingSkyLand_: uint32 = 9421078;
pub const fnRoomCode_ScrollingSkyOcean_: uint32 = 9421083;
pub const fnRoomCode_ScrollingSkyLand_Shakes: uint32 = 9421088;
pub const fnRoomCode_ExplodeShakes: uint32 = 9421092;
pub const fnRoomCode_ScrollRightDachora: uint32 = 9421286;
pub const fnnullsub_142: uint32 = 9423047;
pub const fnRoomCode_8FC8C8: uint32 = 9423048;
pub const fnnullsub_143: uint32 = 9423056;
pub const fnnullsub_144: uint32 = 9423057;
pub const fnnullsub_145: uint32 = 9423058;
pub const fnRoomCode_SetupShaktoolRoomPlm: uint32 = 9423059;
pub const fnnullsub_146: uint32 = 9423068;
pub const fnRoomCode_SetPauseCodeForDraygon: uint32 = 9423069;
pub const fnPauseHook_DraygonRoom: i32 = 0x8fc8f6;
pub const fnUnpauseHook_DraygonRoom: i32 = 0x8fc8fc;
pub const fnRoomCode_SetCollectedMap: uint32 = 9423114;
pub const fnnullsub_147: uint32 = 9423134;
pub const fnRoomCode_SetZebesTimebombEvent: uint32 = 9423135;
pub const fnRoomCode_SetLightHorizRoomShake: uint32 = 9423155;
pub const fnRoomCode_SetMediumHorizRoomShake: uint32 = 9423174;
pub const fnRoomCode_Escape4_SetMediumHorizRoomShake: uint32 = 9423187;
pub const fnRoomCode_SetCeresDoorSolid: uint32 = 9423214;
pub const fnRoomCode_CeresColorMathHdma: uint32 = 9423222;
pub const fnRoomCode_CeresColorMathHdma_BgBase: uint32 = 9423227;
pub const fnDoorCode_StartWreckedSkipTreadmill_East: uint32 = 9429464;
pub const fnDoorCode_SetScroll_44: uint32 = 9429480;
pub const fnDoorCode_SetScroll_45: uint32 = 9429491;
pub const fnDoorCode_SetScroll_46: uint32 = 9429502;
pub const fnDoorCode_SetScroll_47: uint32 = 9429519;
pub const fnDoorCode_SetScroll_48: uint32 = 9429530;
pub const fnDoorCode_SetScroll_49: uint32 = 9429545;
pub const fnDoorCode_SetupElevatubeFromSouth: uint32 = 9429612;
pub const fnDoorCode_SetupElevatubeFromNorth: uint32 = 9429649;
pub const fnRoomCode_Elevatube: uint32 = 9429686;
pub const fnDoorCode_ResetElevatubeNorthExit: uint32 = 9429761;
pub const fnDoorCode_ResetElevatubeSouthExit: uint32 = 9429769;
pub const fnDoorCode_SetScroll_50: uint32 = 9429784;
pub const fnDoorCode_SetScroll_53: uint32 = 9429829;
pub const fnDoorCode_SetScroll_54: uint32 = 9429846;
pub const fnDoorCode_SetScroll_55: uint32 = 9429863;
pub const fnDoorCode_SetScroll_56: uint32 = 9429880;
pub const fnDoorCode_SetScroll_57: uint32 = 9429901;
pub const fnDoorCode_SetScroll_58: uint32 = 9429912;
pub const fnDoorCode_SetScroll_59: uint32 = 9429923;
pub const fnDoorCode_SetScroll_61: uint32 = 9429945;
pub const fnDoorCode_SetScroll_62: uint32 = 9429960;
pub const fnDoorCode_SetScroll_63: uint32 = 9429977;
pub const fnDoorCode_SetScroll_64: uint32 = 9430208;
pub const fnDoorCode_SetScroll_65: uint32 = 9430223;
pub const fnDoorCode_CeresElevatorShaft: uint32 = 9430240;
pub const fnDoorCode_CeresElevatorShaft_2: uint32 = 9430291;
pub const fnRoomCode_CeresElevatorShaft_: uint32 = 9430303;
pub const fnnullsub_148: uint32 = 9430308;
pub const fnRoomCode_SpawnCeresFallingDebris: uint32 = 9430309;
pub const fnRoomCode_HandleCeresRidleyGetaway: uint32 = 9430385;
pub const fnRoomCode_ShakeScreenHorizDiag: uint32 = 9430396;
pub const fnRoomCode_GenRandomExplodes_4th_: uint32 = 9430432;
pub const fnRoomCode_ShakeScreenHorizDiagStrong: uint32 = 9430436;
pub const fnRoomDefStateSelect_Finish: uint32 = 9430502;
pub const fnRoomDefStateSelect_TourianBoss01: uint32 = 9430527;
pub const fnRoomDefStateSelect_IsEventSet: uint32 = 9430546;
pub const fnRoomDefStateSelect_IsBossDead: uint32 = 9430569;
pub const fnRoomDefStateSelect_MorphBallMissiles: uint32 = 9430610;
pub const fnRoomDefStateSelect_PowerBombs: uint32 = 9430633;
pub const fnRoomCode_CrocomireShaking: uint32 = 9431245;
pub const fnRoomCode_RidleyRoomShaking: uint32 = 9431376;
pub unsafe fn RoomSetup_AfterSavingAnimals(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x3d as uint8, 0xb as uint8, 0xbb30 as VoidP),
        game,
    );
    set_ram_word!(game.earthquake_type = 24 as uint16);
    set_ram_word!(game.earthquake_timer = -(1 as i32) as uint16);
}
pub unsafe fn RoomSetup_AutoDestroyWallAfterEscape(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x10 as uint8, 0x87 as uint8, 0xb964 as VoidP),
        game,
    );
}
pub unsafe fn RoomSetup_TurnWallIntoShotblocks(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0xf as uint8, 0xa as uint8, 0xb9ed as VoidP),
        game,
    );
}
pub unsafe fn RoomSetup_ShakeDuringEscape(game: &mut Game) {
    set_ram_word!(game.earthquake_type = 6 as uint16);
    set_ram_word!(game.earthquake_timer = -(1 as i32) as uint16);
    FxTypeFunc_20(game);
}
pub unsafe fn RoomSetup_ScrollingSkyLand(game: &mut Game) {
    FxTypeFunc_20(game);
}
pub unsafe fn RoomSetup_ScrollingSkyOcean(game: &mut Game) {
    RoomSetupAsm_ScrollingSkyOcean(game);
}
pub unsafe fn RoomSetup_RunStatueUnlockAnims(game: &mut Game) {
    SpawnAnimtiles(addr_kAnimtiles_TourianStatue_Kraid as uint16, game);
    SpawnAnimtiles(addr_kAnimtiles_TourianStatue_Phantoon as uint16, game);
    SpawnAnimtiles(addr_kAnimtiles_TourianStatue_Draygon as uint16, game);
    SpawnAnimtiles(addr_kAnimtiles_TourianStatue_Ridley as uint16, game);
}
pub unsafe fn DoorCode_StartWreckedShipTreadmillWest(game: &mut Game) {
    SpawnAnimtiles(addr_kAnimtiles_WreckedShipTradmillRight as uint16, game);
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x4 as uint8, 0x9 as uint8, 0xb64b as VoidP),
        game,
    );
}
pub unsafe fn DoorCode_Scroll6_Green(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(6) = 2 as uint8;
}
pub unsafe fn DoorCode_Scroll0_Blue(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 1 as uint8;
}
pub unsafe fn DoorCode_Scroll13_Blue(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(19) = 1 as uint8;
}
pub unsafe fn DoorCode_Scroll_4Blue_8Green(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(8) = 2 as uint8;
}
pub unsafe fn DoorCode_Scroll_8toB_Red(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(8) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(9) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(10) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(11) = 0 as uint8;
}
pub unsafe fn DoorCode_Scroll_LotsRed(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(3) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(5) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(11) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(12) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(13) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(17) = 0 as uint8;
}
pub unsafe fn DoorCode_Scroll_1_4_Green(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) = 2 as uint8;
}
pub unsafe fn DoorCode_Scroll_2_Blue(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) = 1 as uint8;
}
pub unsafe fn DoorCode_Scroll_17_Blue(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(23) = 1 as uint8;
}
pub unsafe fn DoorCode_Scroll_4_Blue(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) = 1 as uint8;
}
pub unsafe fn DoorCode_Scroll_6_Green(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(6) = 2 as uint8;
}
pub unsafe fn DoorCode_Scroll_3_Green(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(3) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_0(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(24) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(28) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_1(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(5) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(6) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_2(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(29) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_3(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(3) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_4(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_5(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(11) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_6(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(28) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(29) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_7(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_8(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(32) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(36) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(37) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_9(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_10(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_11(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(6) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(7) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_12(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_13(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(3) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_14(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_15(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(3) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_16(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_17(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_18(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(15) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(18) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_19(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(6) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_20(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_21(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_22(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(3) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(6) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(7) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(8) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_23(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(3) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(6) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_24(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_25(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_26(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(10) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_27(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_28(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_29(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(6) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(7) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(8) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_30(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(3) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_31(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(7) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_32(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_33(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(3) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_34(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_35(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(3) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_36(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_37(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_38(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(3) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_39(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_40(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_41(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(24) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_42(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(3) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_43(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(14) = 0 as uint8;
}
pub unsafe fn RoomCode_ScrollingSkyLand_(game: &mut Game) {
    RoomCode_ScrollingSkyLand(game);
}
pub unsafe fn RoomCode_ScrollingSkyOcean_(game: &mut Game) {
    RoomMainAsm_ScrollingSkyOcean(game);
}
pub unsafe fn RoomCode_GenRandomExplodes_Nonblank(game: &mut Game) {
    if ram_word!(game, time_is_frozen_flag) == 0
        && (ram_word!(game, nmi_frame_counter_word) as i32 & 1 as i32) == 0i32
    {
        let mut Random: uint16 = NextRandom(game);
        let x: uint16 = (ram_word!(game, layer1_x_pos)).wrapping_add((Random as uint8) as u16);
        let y: uint16 = (ram_word!(game, layer1_y_pos)).wrapping_add(
            (*(&raw mut Random as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            )) as u16,
        );
        let prod: uint16 = Mult8x8(
            (y as i32 >> 4 as i32) as uint8,
            ram_word!(game, room_width_in_blocks) as uint8,
        );
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
            .offset((prod as i32 + (x as i32 >> 4 as i32)) as isize) as i32
            & 0x3ff as i32)
            != 255i32
        {
            RoomCode_GenRandomExplodes(x, y, game);
        }
    }
}
pub unsafe fn RoomCode_ScrollingSkyLand_Shakes(game: &mut Game) {
    RoomCode_ScrollingSkyLand(game);
    RoomCode_GenRandomExplodes_Nonblank(game);
    let ref mut fresh17 = *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16);
    *fresh17 = (*fresh17) | (0x8000u16);
}
pub unsafe fn RoomCode_ExplodeShakes(game: &mut Game) {
    RoomCode_GenRandomExplodes_Nonblank(game);
    let ref mut fresh16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16);
    *fresh16 = (*fresh16) | (0x8000u16);
}
pub unsafe fn RoomCode_GenRandomExplodes_4th(game: &mut Game) {
    if ram_word!(game, time_is_frozen_flag) == 0
        && (ram_word!(game, nmi_frame_counter_word) as i32 & 3 as i32) == 0i32
    {
        let mut Random: uint16 = NextRandom(game);
        RoomCode_GenRandomExplodes(
            (ram_word!(game, layer1_x_pos)).wrapping_add((Random as uint8) as u16),
            (ram_word!(game, layer1_y_pos)).wrapping_add(
                (*(&raw mut Random as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                )) as u16,
            ),
            game,
        );
    }
}
pub unsafe fn RoomCode_GenRandomExplodes(x_r18: uint16, y_r20: uint16, game: &mut Game) {
    static kRoomCode_GenRandomExplodes_Sprite: [uint8; 8] = [3, 3, 9, 12, 12, 18, 18, 21];
    static kRoomCode_GenRandomExplodes_Sfx: [uint8; 8] = [0x24, 0, 0, 0x25, 0, 0, 0, 0];
    let v1: uint16 = (NextRandom(game)) & (0xfu16);
    if v1 < 8u16 {
        let v2: uint16 = kRoomCode_GenRandomExplodes_Sfx[v1 as usize] as uint16;
        if v2 != 0 {
            QueueSfx2_Max6(v2, game);
        }
    }
    CreateSpriteAtPos(
        x_r18,
        y_r20,
        kRoomCode_GenRandomExplodes_Sprite[(v1 as i32 & 7 as i32) as usize] as uint16,
        0 as uint16,
        game,
    );
}
pub unsafe fn RoomCode_ScrollRightDachora(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(11)
        == 2u8
        && ram_word!(game, layer1_y_pos) < 0x500u16
        && ram_word!(game, layer1_x_pos) < 0x380u16
    {
        let ref mut fresh15 = *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16);
        *fresh15 = (*fresh15 as i32
            + ((ram_word!(game, layer1_x_pos) >= 0x380u16) as i32 + 3 as i32))
            as uint16;
    }
}
pub unsafe fn RoomCode_8FC8C8(game: &mut Game) {
    SpawnEprojWithRoomGfx(addr_kEproj_PrePhantomRoom as uint16, 0 as uint16, game);
}
pub unsafe fn RoomCode_SetupShaktoolRoomPlm(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0 as uint8, 0 as uint8, 0xb8eb as VoidP),
        game,
    );
}
pub unsafe fn RoomCode_SetPauseCodeForDraygon(game: &mut Game) {
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0x601) as *mut LongPtr,
        fnPauseHook_DraygonRoom as uint32,
    );
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0x604) as *mut LongPtr,
        fnUnpauseHook_DraygonRoom as uint32,
    );
}
pub unsafe fn PauseHook_DraygonRoom(game: &mut Game) {
    set_ram_word!(game.irqhandler_next_handler = 4 as uint16);
}
pub unsafe fn UnpauseHook_DraygonRoom(game: &mut Game) -> CoroutineRet {
    if ram_word!(game, hdma_object_channels_bitmask[1]) == 8u16 {
        set_ram_word!(game.irqhandler_next_handler = 12 as uint16);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn RoomCode_SetCollectedMap(game: &mut Game) {
    let v0: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd908)
        .offset(ram_word!(game, area_index) as isize) as *mut uint8
        as *mut uint16))
        | (1u16);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0xd908)
        .offset(ram_word!(game, area_index) as isize) as *mut uint8 as *mut uint16) = v0;
    set_ram_word!(game.has_area_map = v0);
}
pub unsafe fn RoomCode_SetZebesTimebombEvent(game: &mut Game) {
    SetEventHappened(0xe as uint16, game);
    set_ram_word!(game.earthquake_type = 18 as uint16);
    set_ram_word!(game.earthquake_timer = -(1 as i32) as uint16);
}
pub unsafe fn RoomCode_SetLightHorizRoomShake(game: &mut Game) {
    set_ram_word!(game.earthquake_type = 18 as uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(2) as *mut uint8 as *mut uint16) =
        18 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut uint16) = 0 as uint16;
    set_ram_word!(game.earthquake_timer = -(1 as i32) as uint16);
}
pub unsafe fn RoomCode_SetMediumHorizRoomShake(game: &mut Game) {
    set_ram_word!(game.earthquake_type = 21 as uint16);
    set_ram_word!(game.earthquake_timer = -(1 as i32) as uint16);
}
pub unsafe fn RoomCode_Escape4_SetMediumHorizRoomShake(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x10 as uint8, 0x10 as uint8, 0xb968 as VoidP),
        game,
    );
    set_ram_word!(game.earthquake_type = 21 as uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(2) as *mut uint8 as *mut uint16) =
        21 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut uint16) = 0 as uint16;
    set_ram_word!(game.earthquake_timer = -(1 as i32) as uint16);
}
pub unsafe fn RoomCode_SetCeresDoorSolid(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0xf as uint8, 0x26 as uint8, 0xba48 as VoidP),
        game,
    );
    FxTypeFunc_2C_Haze(game);
}
pub unsafe fn RoomCode_CeresColorMathHdma(game: &mut Game) {
    FxTypeFunc_2C_Haze(game);
}
pub unsafe fn RoomCode_CeresColorMathHdma_BgBase(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d) = 102 as uint8;
    FxTypeFunc_2C_Haze(game);
    set_ram_word!(game.hdma_data_table_in_ceres = 9 as uint16);
}
pub unsafe fn DoorCode_StartWreckedSkipTreadmill_East(game: &mut Game) {
    SpawnAnimtiles(addr_kAnimtiles_WreckedShipTradmillLeft as uint16, game);
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x4 as uint8, 0x9 as uint8, 0xb64f as VoidP),
        game,
    );
}
pub unsafe fn DoorCode_SetScroll_44(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_45(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_46(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(3) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_47(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(41) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_48(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(40) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(46) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_49(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(6) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(7) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(8) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(9) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(10) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(11) = 0 as uint8;
}
pub unsafe fn DoorCode_SetupElevatubeFromSouth(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(4) as *mut uint8 as *mut uint16) =
        -(256 as i32) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(2) as *mut uint8 as *mut uint16) =
        2496 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(6) as *mut uint8 as *mut uint16) =
        -(32 as i32) as uint16;
    CallSomeSamusCode(0 as uint16, game);
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x1 as uint8, 0 as uint8, 0xb8f9 as VoidP),
        game,
    );
}
pub unsafe fn DoorCode_SetupElevatubeFromNorth(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(4) as *mut uint8 as *mut uint16) =
        256 as uint16;
    strcpy(
        (&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(2) as *mut uint8
            as *mut ::core::ffi::c_char,
        b"@\0" as *const u8 as *const ::core::ffi::c_char,
    );
    strcpy(
        (&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(6) as *mut uint8
            as *mut ::core::ffi::c_char,
        b" \0" as *const u8 as *const ::core::ffi::c_char,
    );
    CallSomeSamusCode(0 as uint16, game);
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x1 as uint8, 0 as uint8, 0xb8f9 as VoidP),
        game,
    );
}
pub unsafe fn RoomCode_Elevatube(game: &mut Game) {
    let mut v0: uint16 = 0;
    set_ram_word!(game.samus_x_pos = 128 as uint16);
    set_ram_word!(game.samus_x_subpos = 0 as uint16);
    let amt: int32 = (*((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(4) as *mut uint8
        as *mut uint16) as int16 as int32)
        << 8 as i32;
    v0 = ((amt as uint32)
        .wrapping_add(*((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut uint32))
        >> 16 as i32) as uint16;
    let ref mut fresh13 = *((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut uint16);
    *fresh13 = (*fresh13 as i32 + amt as i32) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(2) as *mut uint8 as *mut uint16) =
        v0;
    Samus_MoveDown_NoSolidColl(amt, game);
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(6) as *mut uint8 as *mut uint16)
        as i32
        + *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(4) as *mut uint8
            as *mut uint16) as i32
        + 3616 as i32) as uint16)
        < 0x1c41u16
    {
        let ref mut fresh14 = *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(4)
            as *mut uint8 as *mut uint16);
        *fresh14 = (*fresh14).wrapping_add(
            *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(6) as *mut uint8
                as *mut uint16),
        );
    }
}
pub unsafe fn DoorCode_ResetElevatubeNorthExit(game: &mut Game) {
    CallSomeSamusCode(1 as uint16, game);
}
pub unsafe fn DoorCode_ResetElevatubeSouthExit(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd20) as *mut uint16) = 514 as uint16;
    CallSomeSamusCode(1 as uint16, game);
}
pub unsafe fn DoorCode_SetScroll_50(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(10) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(11) = 1 as uint8;
}
pub unsafe fn UNUSED_DoorCode_SetScroll_51(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(17) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(16) = 1 as uint8;
}
pub unsafe fn UNUSED_DoorCode_SetScroll_52(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_53(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_54(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_55(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(9) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(10) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_56(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_57(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_58(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(6) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_59(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_60(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_61(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(7) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_62(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_63(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_64(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(1) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_65(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(24) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(25) = 0 as uint8;
}
pub unsafe fn DoorCode_CeresElevatorShaft(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x56) = 7 as uint8;
    WriteReg(BGMODE as uint16, 7 as uint8, game);
    set_ram_word!(game.reg_M7A = 256 as uint16);
    set_ram_word!(game.reg_M7D = 256 as uint16);
    set_ram_word!(game.reg_M7B = 0 as uint16);
    set_ram_word!(game.reg_M7C = 0 as uint16);
    set_ram_word!(game.reg_M7X = 128 as uint16);
    set_ram_word!(game.reg_M7Y = 1008 as uint16);
    set_ram_word!(game.irq_enable_mode7 = 1 as uint16);
    strcpy(
        (&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut ::core::ffi::c_char,
        b"\"\0" as *const u8 as *const ::core::ffi::c_char,
    );
    strcpy(
        (&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(2) as *mut uint8
            as *mut ::core::ffi::c_char,
        b"<\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
pub unsafe fn DoorCode_CeresElevatorShaft_2(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x56) = 9 as uint8;
    set_ram_word!(game.irq_enable_mode7 = 0 as uint16);
}
pub unsafe fn RoomCode_CeresElevatorShaft_(game: &mut Game) {
    RoomCode_CeresElevatorShaft(game);
}
static kRoomCode_SpawnCeresFallingDebris_Tab: [uint16; 16] = [
    0x50, 0x60, 0x70, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0x110, 0x130, 0x150, 0x170,
    0x190,
];
pub unsafe fn RoomCode_SpawnCeresFallingDebris(game: &mut Game) {
    if ram_word!(game, ceres_status) != 0u16 && {
        let ref mut fresh12 = *((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut uint16);
        *fresh12 = (*fresh12).wrapping_sub(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut int16) < 0i16
    } {
        *(&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(0) = 8 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(1) = 0 as uint8;
        let v0: uint16 = (if ram_word!(game, random_number) as i32 & 0x8000 as i32 != 0 {
            addr_stru_869742 as i32
        } else {
            addr_stru_869734 as i32
        }) as uint16;
        SpawnEprojWithRoomGfx(
            v0,
            kRoomCode_SpawnCeresFallingDebris_Tab
                [(ram_word!(game, random_number) as i32 & 0xf as i32) as usize],
            game,
        );
    }
}
pub unsafe fn RoomCode_HandleCeresRidleyGetaway(game: &mut Game) {
    if ram_word!(game, ceres_status) as i32 & 1 as i32 != 0 {
        HandleCeresRidleyGetawayCutscene(game);
    }
}
pub unsafe fn RoomCode_ShakeScreenHorizDiag(game: &mut Game) {
    let mut v0: uint16 = 0;
    let current_block_3: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut uint16) != 0 {
        let ref mut fresh11 = *((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut uint16);
        *fresh11 = (*fresh11).wrapping_sub(1);
        if *fresh11 == 0 {
            v0 = 18 as uint16;
            current_block_3 = 10175730460759437497;
        } else {
            current_block_3 = 11006700562992250127;
        }
    } else if NextRandom(game) < 0x200u16 {
        strcpy(
            (&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut ::core::ffi::c_char,
            b"*\0" as *const u8 as *const ::core::ffi::c_char,
        );
        v0 = 23 as uint16;
        current_block_3 = 10175730460759437497;
    } else {
        current_block_3 = 11006700562992250127;
    }
    if current_block_3 == 10175730460759437497 {
        set_ram_word!(game.earthquake_type = v0);
    }
    RoomCode_GenRandomExplodes_4th(game);
}
pub unsafe fn RoomCode_ShakeScreenHorizDiagStrong(game: &mut Game) {
    let mut v0: int16 = 0;
    let current_block_3: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut uint16) != 0 {
        let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut uint16);
        *fresh10 = (*fresh10).wrapping_sub(1);
        if *fresh10 == 0 {
            v0 = 21 as int16;
            current_block_3 = 1544504899801662111;
        } else {
            current_block_3 = 11006700562992250127;
        }
    } else if NextRandom(game) < 0x180u16 {
        strcpy(
            (&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut ::core::ffi::c_char,
            b"*\0" as *const u8 as *const ::core::ffi::c_char,
        );
        v0 = 26 as int16;
        current_block_3 = 1544504899801662111;
    } else {
        current_block_3 = 11006700562992250127;
    }
    if current_block_3 == 1544504899801662111 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(2) as *mut uint8
            as *mut uint16) = v0 as uint16;
    }
    set_ram_word!(
        game.earthquake_type = *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(2)
            as *mut uint8 as *mut uint16)
    );
    RoomCode_GenRandomExplodes_4th(game);
}
pub unsafe fn CallRoomDefStateSelect(ea: uint32, k: uint16, game: &mut Game) -> uint16 {
    match ea {
        9430502 => RoomDefStateSelect_Finish(k, game),
        9430527 => RoomDefStateSelect_TourianBoss01(k, game),
        9430546 => RoomDefStateSelect_IsEventSet(k, game),
        9430569 => RoomDefStateSelect_IsBossDead(k, game),
        9430610 => RoomDefStateSelect_MorphBallMissiles(k, game),
        9430633 => RoomDefStateSelect_PowerBombs(k, game),
        _ => Unreachable(game) as uint16,
    }
}
pub unsafe fn HandleRoomDefStateSelect(k: uint16, game: &mut Game) {
    let mut v1: uint16 = (k).wrapping_add(11u16);
    loop {
        let event_pointer: uint16 = (*get_RoomDefStateSelect_E6E5_Finish(v1, game)).code_ptr;
        v1 = CallRoomDefStateSelect(
            ((event_pointer) as u32) | (0x8f0000u32),
            (v1).wrapping_add(2u16),
            game,
        );
        if (v1 == 0) {
            break;
        }
    }
}
pub unsafe fn RoomDefStateSelect_Finish(k: uint16, game: &mut Game) -> uint16 {
    set_ram_word!(game.roomdefroomstate_ptr = k);
    0 as uint16
}
pub unsafe fn RoomDefStateSelect_TourianBoss01(k: uint16, game: &mut Game) -> uint16 {
    if CheckBossBitForCurArea(1 as uint16, game) as i32 & 1 as i32 == 0 {
        return (k).wrapping_add(2u16);
    }
    let v1: *const uint16 = RomPtr_8F(k, game) as *const uint16;
    RoomDefStateSelect_Finish(*v1, game)
}
pub unsafe fn RoomDefStateSelect_IsEventSet(k: uint16, game: &mut Game) -> uint16 {
    let v1: *const uint8 = RomPtr_8F(k, game);
    if CheckEventHappened(*v1 as uint16, game) != 0 {
        RoomDefStateSelect_Finish(*(v1.offset(1) as *mut uint16), game)
    } else {
        (k).wrapping_add(3u16)
    }
}
pub unsafe fn RoomDefStateSelect_IsBossDead(k: uint16, game: &mut Game) -> uint16 {
    let v1: *const uint8 = RomPtr_8F(k, game);
    if CheckBossBitForCurArea(*v1 as uint16, game) as i32 & 1 as i32 != 0 {
        RoomDefStateSelect_Finish(*(v1.offset(1) as *mut uint16), game)
    } else {
        (k).wrapping_add(3u16)
    }
}
pub unsafe fn RoomDefStateSelect_MorphBallMissiles(k: uint16, game: &mut Game) -> uint16 {
    if (ram_word!(game, collected_items) as i32 & 4 as i32) == 0i32
        || ram_word!(game, samus_max_missiles) == 0
    {
        return (k).wrapping_add(2u16);
    }
    let v1: *const uint16 = RomPtr_8F(k, game) as *const uint16;
    RoomDefStateSelect_Finish(*v1, game)
}
pub unsafe fn RoomDefStateSelect_PowerBombs(k: uint16, game: &mut Game) -> uint16 {
    if ram_word!(game, samus_max_power_bombs) == 0 {
        return (k).wrapping_add(2u16);
    }
    let v1: *const uint16 = RomPtr_8F(k, game) as *const uint16;
    RoomDefStateSelect_Finish(*v1, game)
}
pub unsafe fn CallRoomSetupCode(ea: uint32, game: &mut Game) {
    match ea {
        9408916 => RoomSetup_AfterSavingAnimals(game),
        9408937 => RoomSetup_AutoDestroyWallAfterEscape(game),
        9408946 => RoomSetup_TurnWallIntoShotblocks(game),
        9408957 => RoomSetup_ShakeDuringEscape(game),
        9408969 => RoomSetup_ScrollingSkyLand(game),
        9408974 => RoomSetup_ScrollingSkyOcean(game),
        9408983 => RoomSetup_RunStatueUnlockAnims(game),
        9423048 => RoomCode_8FC8C8(game),
        9423059 => RoomCode_SetupShaktoolRoomPlm(game),
        9423069 => RoomCode_SetPauseCodeForDraygon(game),
        9423114 => RoomCode_SetCollectedMap(game),
        9408955 | 9408956 | 9408979 | 9408980 | 9408981 | 9408982 | 9409012 | 9409013 | 9409014
        | 9409015 | 9423047 | 9423056 | 9423057 | 9423058 | 9423068 | 9423134 => (),
        9423135 => RoomCode_SetZebesTimebombEvent(game),
        9423155 => RoomCode_SetLightHorizRoomShake(game),
        9423174 => RoomCode_SetMediumHorizRoomShake(game),
        9423187 => RoomCode_Escape4_SetMediumHorizRoomShake(game),
        9423214 => RoomCode_SetCeresDoorSolid(game),
        9423222 => RoomCode_CeresColorMathHdma(game),
        9423227 => RoomCode_CeresColorMathHdma_BgBase(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn RunRoomSetupCode(game: &mut Game) {
    let mut RoomDefRoomstate: *mut RoomDefRoomstate = ::core::ptr::null_mut::<RoomDefRoomstate>();
    RoomDefRoomstate = get_RoomDefRoomstate(ram_word!(game, roomdefroomstate_ptr), game);
    if (*RoomDefRoomstate).room_setup_code != 0 {
        CallRoomSetupCode(
            (((*RoomDefRoomstate).room_setup_code) as u32) | (0x8f0000u32),
            game,
        );
    }
}
pub unsafe fn CallDoorDefSetupCode(ea: uint32, game: &mut Game) {
    match ea {
        9419121 => DoorCode_StartWreckedShipTreadmillWest(game),
        9419137 => DoorCode_Scroll6_Green(game),
        9419148 => DoorCode_Scroll0_Blue(game),
        9419159 => DoorCode_Scroll13_Blue(game),
        9419170 => DoorCode_Scroll_4Blue_8Green(game),
        9419187 => DoorCode_Scroll_8toB_Red(game),
        9419210 => DoorCode_Scroll_LotsRed(game),
        9419249 => DoorCode_Scroll_1_4_Green(game),
        9419264 => DoorCode_Scroll_2_Blue(game),
        9419275 => DoorCode_Scroll_17_Blue(game),
        9419286 => DoorCode_Scroll_4_Blue(game),
        9419297 => DoorCode_Scroll_6_Green(game),
        9419308 => DoorCode_Scroll_3_Green(game),
        9420039 => DoorCode_SetScroll_0(game),
        9420054 => DoorCode_SetScroll_1(game),
        9420069 => DoorCode_SetScroll_2(game),
        9420080 => DoorCode_SetScroll_3(game),
        9420095 => DoorCode_SetScroll_4(game),
        9420112 => DoorCode_SetScroll_5(game),
        9420123 => DoorCode_SetScroll_6(game),
        9420140 => DoorCode_SetScroll_7(game),
        9420151 => DoorCode_SetScroll_8(game),
        9420170 => DoorCode_SetScroll_9(game),
        9420181 => DoorCode_SetScroll_10(game),
        9420192 => DoorCode_SetScroll_11(game),
        9420207 => DoorCode_SetScroll_12(game),
        9420224 => DoorCode_SetScroll_13(game),
        9420241 => DoorCode_SetScroll_14(game),
        9420258 => DoorCode_SetScroll_15(game),
        9420273 => DoorCode_SetScroll_16(game),
        9420288 => DoorCode_SetScroll_17(game),
        9420299 => DoorCode_SetScroll_18(game),
        9420314 => DoorCode_SetScroll_19(game),
        9420325 => DoorCode_SetScroll_20(game),
        9420342 => DoorCode_SetScroll_21(game),
        9420702 => DoorCode_SetScroll_22(game),
        9420731 => DoorCode_SetScroll_23(game),
        9420762 => DoorCode_SetScroll_24(game),
        9420777 => DoorCode_SetScroll_25(game),
        9420794 => DoorCode_SetScroll_26(game),
        9420822 => DoorCode_SetScroll_28(game),
        9420837 => DoorCode_SetScroll_29(game),
        9420858 => DoorCode_SetScroll_30(game),
        9420875 => DoorCode_SetScroll_31(game),
        9420886 => DoorCode_SetScroll_32(game),
        9420903 => DoorCode_SetScroll_33(game),
        9420920 => DoorCode_SetScroll_34(game),
        9420937 => DoorCode_SetScroll_35(game),
        9420962 => DoorCode_SetScroll_36(game),
        9420973 => DoorCode_SetScroll_37(game),
        9420994 => DoorCode_SetScroll_38(game),
        9421011 => DoorCode_SetScroll_39(game),
        9421022 => DoorCode_SetScroll_40(game),
        9421039 => DoorCode_SetScroll_41(game),
        9421050 => DoorCode_SetScroll_42(game),
        9421067 => DoorCode_SetScroll_43(game),
        9429464 => DoorCode_StartWreckedSkipTreadmill_East(game),
        9429480 => DoorCode_SetScroll_44(game),
        9429491 => DoorCode_SetScroll_45(game),
        9429502 => DoorCode_SetScroll_46(game),
        9429519 => DoorCode_SetScroll_47(game),
        9429530 => DoorCode_SetScroll_48(game),
        9429545 => DoorCode_SetScroll_49(game),
        9429612 => DoorCode_SetupElevatubeFromSouth(game),
        9429649 => DoorCode_SetupElevatubeFromNorth(game),
        9429761 => DoorCode_ResetElevatubeNorthExit(game),
        9429769 => DoorCode_ResetElevatubeSouthExit(game),
        9429784 => DoorCode_SetScroll_50(game),
        9429829 => DoorCode_SetScroll_53(game),
        9429846 => DoorCode_SetScroll_54(game),
        9429863 => DoorCode_SetScroll_55(game),
        9429880 => DoorCode_SetScroll_56(game),
        9429901 => DoorCode_SetScroll_57(game),
        9429912 => DoorCode_SetScroll_58(game),
        9429923 => DoorCode_SetScroll_59(game),
        9429945 => DoorCode_SetScroll_61(game),
        9429960 => DoorCode_SetScroll_62(game),
        9429977 => DoorCode_SetScroll_63(game),
        9430208 => DoorCode_SetScroll_64(game),
        9430223 => DoorCode_SetScroll_65(game),
        9430240 => DoorCode_CeresElevatorShaft(game),
        9430291 => DoorCode_CeresElevatorShaft_2(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn RunDoorSetupCode(game: &mut Game) {
    let DD: *mut DoorDef = get_DoorDef(ram_word!(game, door_def_ptr), game);
    if (*DD).door_setup_code != 0 {
        CallDoorDefSetupCode((((*DD).door_setup_code) as u32) | (0x8f0000u32), game);
    }
}
pub unsafe fn RoomCode_CrocomireShaking(game: &mut Game) {
    let mut ai_var_D: uint16 = 0;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .properties as i32
        & 0x200 as i32)
        == 0i32
    {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .ai_var_A
            == 64u16
        {
            set_ram_word!(
                game.reg_BG1VOFS =
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
                        .offset(1))
                    .ai_var_D as i32
                        + ram_word!(game, bg1_y_offset) as i32
                        + ram_word!(game, layer1_y_pos) as i32) as uint16
            );
        } else if ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
            .offset(0))
        .ai_var_B as i32
            & 0x400 as i32)
            != 0i32
        {
            let fresh5 = &raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0xf78)
                as *mut EnemyData)
                .offset(1))
            .ai_var_D;
            fresh5.write_unaligned(fresh5.read_unaligned().wrapping_sub(1));
            if fresh5.read_unaligned() as i32 + 7 as i32 & 0x8000 as i32 != 0 {
                ai_var_D = ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
                    .offset(1))
                .ai_var_D as i32
                    - 2 as i32
                        * ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
                            .offset(1))
                        .ai_var_D as i32
                            + 7 as i32)) as uint16;
            } else {
                ai_var_D = (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
                    .offset(1))
                .ai_var_D;
            }
            let ref mut fresh6 = *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16);
            *fresh6 = (*fresh6).wrapping_add(ai_var_D);
            set_ram_word!(game.reg_BG2VOFS = (ai_var_D).wrapping_sub(48u16));
        } else if (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .ai_var_C
            == 34u16
            && (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .ai_var_D
                != 0u16
        {
            let fresh7 = &raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0xf78)
                as *mut EnemyData)
                .offset(0))
            .ai_var_D;
            fresh7.write_unaligned(fresh7.read_unaligned().wrapping_sub(1));
            if ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .ai_var_D as i32
                & 1 as i32)
                != 0i32
            {
                let ref mut fresh8 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16);
                *fresh8 = (*fresh8).wrapping_sub(4u16);
            } else {
                let ref mut fresh9 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16);
                *fresh9 = (*fresh9).wrapping_add(4u16);
            }
        }
    }
}
static kRoomCode_RidleyRoomShaking_X: [int16; 8] = [0, 2, 2, 2, 0, -(2), -(2), -(2)];
static kRoomCode_RidleyRoomShaking_Y: [int16; 8] = [-(2), -(2), 0, 2, 2, 2, 0, -(2)];
pub unsafe fn RoomCode_RidleyRoomShaking(game: &mut Game) {
    let ai_var_A: uint16 =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(4)).ai_var_A;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(4)).ai_var_A
        != 0
    {
        let fresh0 = &raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0xf78)
            as *mut EnemyData)
            .offset(4))
        .ai_var_A;
        fresh0.write_unaligned(fresh0.read_unaligned().wrapping_sub(1));
        let v1: i32 = ai_var_A as i32 - 1 as i32;
        let ref mut fresh1 = *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16);
        *fresh1 = (*fresh1).wrapping_add((kRoomCode_RidleyRoomShaking_X[v1 as usize]) as u16);
        let ref mut fresh2 = *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16);
        *fresh2 = (*fresh2).wrapping_add((kRoomCode_RidleyRoomShaking_X[v1 as usize]) as u16);
        let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16);
        *fresh3 = (*fresh3).wrapping_add((kRoomCode_RidleyRoomShaking_Y[v1 as usize]) as u16);
        let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16);
        *fresh4 = (*fresh4).wrapping_add((kRoomCode_RidleyRoomShaking_Y[v1 as usize]) as u16);
    }
}
pub unsafe fn CallRoomCode(ea: uint32, game: &mut Game) {
    match ea {
        9421078 => RoomCode_ScrollingSkyLand_(game),
        9421083 => RoomCode_ScrollingSkyOcean_(game),
        9421088 => RoomCode_ScrollingSkyLand_Shakes(game),
        9421092 => RoomCode_ExplodeShakes(game),
        9421286 => RoomCode_ScrollRightDachora(game),
        9429686 => RoomCode_Elevatube(game),
        9430303 => RoomCode_CeresElevatorShaft_(game),
        9430308 => (),
        9430309 => RoomCode_SpawnCeresFallingDebris(game),
        9430385 => RoomCode_HandleCeresRidleyGetaway(game),
        9430396 => RoomCode_ShakeScreenHorizDiag(game),
        9430432 => RoomCode_GenRandomExplodes_4th(game),
        9430436 => RoomCode_ShakeScreenHorizDiagStrong(game),
        9431245 => RoomCode_CrocomireShaking(game),
        9431376 => RoomCode_RidleyRoomShaking(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn RunRoomMainCode(game: &mut Game) {
    if ram_word!(game, room_main_code_ptr) != 0 {
        CallRoomCode(
            ((ram_word!(game, room_main_code_ptr)) as u32) | (0x8f0000u32),
            game,
        );
    }
}
