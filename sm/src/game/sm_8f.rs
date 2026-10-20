use crate::c_library::strcpy;
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
use crate::ram_map::ram_ptr;
use crate::ram_map::set_ram_words;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::{mov24, Mult8x8, RomPtr, Unreachable, WriteReg};
pub use crate::snes::registers::*;
pub use crate::types::EnemyData;
pub use crate::types::{CoroutineRet, LongPtr, SnesRegs, VoidP};
use ::core::ffi::c_char;
use ::core::ptr::null_mut;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct RoomDefStateSelect_E6E5_Finish {
    pub code_ptr: VoidP,
}
pub use crate::types::Consts_87;
pub use crate::types::SpawnHardcodedPlmArgs;
pub use crate::types::{Consts_86, DoorDef, RoomDefRoomstate};
crate::game::constants::typed_constants! { i32;
    kCoroutineNone = 0,
}
const RomPtr_8F: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8f>;
#[inline]
unsafe fn get_DoorDef(a: u16, game: &mut Game) -> *mut DoorDef {
    RomPtr((0x830000u32) | ((a) as u32), game) as *mut DoorDef
}
#[inline]
unsafe fn get_RoomDefRoomstate(a: u16, game: &mut Game) -> *mut RoomDefRoomstate {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomDefRoomstate
}
#[inline]
unsafe fn get_RoomDefStateSelect_E6E5_Finish(
    a: u16,
    game: &mut Game,
) -> *mut RoomDefStateSelect_E6E5_Finish {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomDefStateSelect_E6E5_Finish
}
crate::game::constants::typed_constants! { u32;
    fnRoomSetup_AfterSavingAnimals = 9408916, fnRoomSetup_AutoDestroyWallAfterEscape = 9408937,
    fnRoomSetup_TurnWallIntoShotblocks = 9408946, fnnullsub_132 = 9408955,
    fnnullsub_133 = 9408956, fnRoomSetup_ShakeDuringEscape = 9408957,
    fnRoomSetup_ScrollingSkyLand = 9408969, fnRoomSetup_ScrollingSkyOcean = 9408974,
    fnnullsub_134 = 9408979, fnnullsub_135 = 9408980,
    fnnullsub_136 = 9408981, fnnullsub_137 = 9408982,
    fnRoomSetup_RunStatueUnlockAnims = 9408983, fnnullsub_138 = 9409012,
    fnnullsub_139 = 9409013, fnnullsub_140 = 9409014,
    fnnullsub_141 = 9409015, fnDoorCode_StartWreckedShipTreadmillWest = 9419121,
    fnDoorCode_Scroll6_Green = 9419137, fnDoorCode_Scroll0_Blue = 9419148,
    fnDoorCode_Scroll13_Blue = 9419159, fnDoorCode_Scroll_4Blue_8Green = 9419170,
    fnDoorCode_Scroll_8toB_Red = 9419187, fnDoorCode_Scroll_LotsRed = 9419210,
    fnDoorCode_Scroll_1_4_Green = 9419249, fnDoorCode_Scroll_2_Blue = 9419264,
    fnDoorCode_Scroll_17_Blue = 9419275, fnDoorCode_Scroll_4_Blue = 9419286,
    fnDoorCode_Scroll_6_Green = 9419297, fnDoorCode_Scroll_3_Green = 9419308,
    fnDoorCode_SetScroll_0 = 9420039, fnDoorCode_SetScroll_1 = 9420054,
    fnDoorCode_SetScroll_2 = 9420069, fnDoorCode_SetScroll_3 = 9420080,
    fnDoorCode_SetScroll_4 = 9420095, fnDoorCode_SetScroll_5 = 9420112,
    fnDoorCode_SetScroll_6 = 9420123, fnDoorCode_SetScroll_7 = 9420140,
    fnDoorCode_SetScroll_8 = 9420151, fnDoorCode_SetScroll_9 = 9420170,
    fnDoorCode_SetScroll_10 = 9420181, fnDoorCode_SetScroll_11 = 9420192,
    fnDoorCode_SetScroll_12 = 9420207, fnDoorCode_SetScroll_13 = 9420224,
    fnDoorCode_SetScroll_14 = 9420241, fnDoorCode_SetScroll_15 = 9420258,
    fnDoorCode_SetScroll_16 = 9420273, fnDoorCode_SetScroll_17 = 9420288,
    fnDoorCode_SetScroll_18 = 9420299, fnDoorCode_SetScroll_19 = 9420314,
    fnDoorCode_SetScroll_20 = 9420325, fnDoorCode_SetScroll_21 = 9420342,
    fnDoorCode_SetScroll_22 = 9420702, fnDoorCode_SetScroll_23 = 9420731,
    fnDoorCode_SetScroll_24 = 9420762, fnDoorCode_SetScroll_25 = 9420777,
    fnDoorCode_SetScroll_26 = 9420794, fnDoorCode_SetScroll_28 = 9420822,
    fnDoorCode_SetScroll_29 = 9420837, fnDoorCode_SetScroll_30 = 9420858,
    fnDoorCode_SetScroll_31 = 9420875, fnDoorCode_SetScroll_32 = 9420886,
    fnDoorCode_SetScroll_33 = 9420903, fnDoorCode_SetScroll_34 = 9420920,
    fnDoorCode_SetScroll_35 = 9420937, fnDoorCode_SetScroll_36 = 9420962,
    fnDoorCode_SetScroll_37 = 9420973, fnDoorCode_SetScroll_38 = 9420994,
    fnDoorCode_SetScroll_39 = 9421011, fnDoorCode_SetScroll_40 = 9421022,
    fnDoorCode_SetScroll_41 = 9421039, fnDoorCode_SetScroll_42 = 9421050,
    fnDoorCode_SetScroll_43 = 9421067, fnRoomCode_ScrollingSkyLand_ = 9421078,
    fnRoomCode_ScrollingSkyOcean_ = 9421083, fnRoomCode_ScrollingSkyLand_Shakes = 9421088,
    fnRoomCode_ExplodeShakes = 9421092, fnRoomCode_ScrollRightDachora = 9421286,
    fnnullsub_142 = 9423047, fnRoomCode_8FC8C8 = 9423048,
    fnnullsub_143 = 9423056, fnnullsub_144 = 9423057,
    fnnullsub_145 = 9423058, fnRoomCode_SetupShaktoolRoomPlm = 9423059,
    fnnullsub_146 = 9423068, fnRoomCode_SetPauseCodeForDraygon = 9423069,
}
crate::game::constants::typed_constants! { i32;
    fnPauseHook_DraygonRoom = 0x8fc8f6, fnUnpauseHook_DraygonRoom = 0x8fc8fc,
}
crate::game::constants::typed_constants! { u32;
    fnRoomCode_SetCollectedMap = 9423114, fnnullsub_147 = 9423134,
    fnRoomCode_SetZebesTimebombEvent = 9423135, fnRoomCode_SetLightHorizRoomShake = 9423155,
    fnRoomCode_SetMediumHorizRoomShake = 9423174, fnRoomCode_Escape4_SetMediumHorizRoomShake = 9423187,
    fnRoomCode_SetCeresDoorSolid = 9423214, fnRoomCode_CeresColorMathHdma = 9423222,
    fnRoomCode_CeresColorMathHdma_BgBase = 9423227, fnDoorCode_StartWreckedSkipTreadmill_East = 9429464,
    fnDoorCode_SetScroll_44 = 9429480, fnDoorCode_SetScroll_45 = 9429491,
    fnDoorCode_SetScroll_46 = 9429502, fnDoorCode_SetScroll_47 = 9429519,
    fnDoorCode_SetScroll_48 = 9429530, fnDoorCode_SetScroll_49 = 9429545,
    fnDoorCode_SetupElevatubeFromSouth = 9429612, fnDoorCode_SetupElevatubeFromNorth = 9429649,
    fnRoomCode_Elevatube = 9429686, fnDoorCode_ResetElevatubeNorthExit = 9429761,
    fnDoorCode_ResetElevatubeSouthExit = 9429769, fnDoorCode_SetScroll_50 = 9429784,
    fnDoorCode_SetScroll_53 = 9429829, fnDoorCode_SetScroll_54 = 9429846,
    fnDoorCode_SetScroll_55 = 9429863, fnDoorCode_SetScroll_56 = 9429880,
    fnDoorCode_SetScroll_57 = 9429901, fnDoorCode_SetScroll_58 = 9429912,
    fnDoorCode_SetScroll_59 = 9429923, fnDoorCode_SetScroll_61 = 9429945,
    fnDoorCode_SetScroll_62 = 9429960, fnDoorCode_SetScroll_63 = 9429977,
    fnDoorCode_SetScroll_64 = 9430208, fnDoorCode_SetScroll_65 = 9430223,
    fnDoorCode_CeresElevatorShaft = 9430240, fnDoorCode_CeresElevatorShaft_2 = 9430291,
    fnRoomCode_CeresElevatorShaft_ = 9430303, fnnullsub_148 = 9430308,
    fnRoomCode_SpawnCeresFallingDebris = 9430309, fnRoomCode_HandleCeresRidleyGetaway = 9430385,
    fnRoomCode_ShakeScreenHorizDiag = 9430396, fnRoomCode_GenRandomExplodes_4th_ = 9430432,
    fnRoomCode_ShakeScreenHorizDiagStrong = 9430436, fnRoomDefStateSelect_Finish = 9430502,
    fnRoomDefStateSelect_TourianBoss01 = 9430527, fnRoomDefStateSelect_IsEventSet = 9430546,
    fnRoomDefStateSelect_IsBossDead = 9430569, fnRoomDefStateSelect_MorphBallMissiles = 9430610,
    fnRoomDefStateSelect_PowerBombs = 9430633, fnRoomCode_CrocomireShaking = 9431245,
    fnRoomCode_RidleyRoomShaking = 9431376,
}
pub unsafe fn RoomSetup_AfterSavingAnimals(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x3du8, 0xbu8, 0xbb30 as VoidP),
        game,
    );
    set_ram_word!(game.earthquake_type = 24u16);
    set_ram_word!(game.earthquake_timer = -1i32 as u16);
}
pub unsafe fn RoomSetup_AutoDestroyWallAfterEscape(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x10u8, 0x87u8, 0xb964 as VoidP),
        game,
    );
}
pub unsafe fn RoomSetup_TurnWallIntoShotblocks(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0xfu8, 0xau8, 0xb9ed as VoidP),
        game,
    );
}
pub unsafe fn RoomSetup_ShakeDuringEscape(game: &mut Game) {
    set_ram_word!(game.earthquake_type = 6u16);
    set_ram_word!(game.earthquake_timer = -1i32 as u16);
    FxTypeFunc_20(game);
}
pub unsafe fn RoomSetup_ScrollingSkyLand(game: &mut Game) {
    FxTypeFunc_20(game);
}
pub unsafe fn RoomSetup_ScrollingSkyOcean(game: &mut Game) {
    RoomSetupAsm_ScrollingSkyOcean(game);
}
pub unsafe fn RoomSetup_RunStatueUnlockAnims(game: &mut Game) {
    SpawnAnimtiles(addr_kAnimtiles_TourianStatue_Kraid as u16, game);
    SpawnAnimtiles(addr_kAnimtiles_TourianStatue_Phantoon as u16, game);
    SpawnAnimtiles(addr_kAnimtiles_TourianStatue_Draygon as u16, game);
    SpawnAnimtiles(addr_kAnimtiles_TourianStatue_Ridley as u16, game);
}
pub unsafe fn DoorCode_StartWreckedShipTreadmillWest(game: &mut Game) {
    SpawnAnimtiles(addr_kAnimtiles_WreckedShipTradmillRight as u16, game);
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x4u8, 0x9u8, 0xb64b as VoidP),
        game,
    );
}
pub unsafe fn DoorCode_Scroll6_Green(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 6) = 2u8;
}
pub unsafe fn DoorCode_Scroll0_Blue(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 1u8;
}
pub unsafe fn DoorCode_Scroll13_Blue(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 19) = 1u8;
}
pub unsafe fn DoorCode_Scroll_4Blue_8Green(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 4) = 0u8;
    *ram_ptr!(game; 0xcd20, 8) = 2u8;
}
pub unsafe fn DoorCode_Scroll_8toB_Red(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 8) = 0u8;
    *ram_ptr!(game; 0xcd20, 9) = 0u8;
    *ram_ptr!(game; 0xcd20, 10) = 0u8;
    *ram_ptr!(game; 0xcd20, 11) = 0u8;
}
pub unsafe fn DoorCode_Scroll_LotsRed(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 2) = 0u8;
    *ram_ptr!(game; 0xcd20, 3) = 0u8;
    *ram_ptr!(game; 0xcd20, 4) = 0u8;
    *ram_ptr!(game; 0xcd20, 5) = 0u8;
    *ram_ptr!(game; 0xcd20, 11) = 0u8;
    *ram_ptr!(game; 0xcd20, 12) = 0u8;
    *ram_ptr!(game; 0xcd20, 13) = 0u8;
    *ram_ptr!(game; 0xcd20, 17) = 0u8;
}
pub unsafe fn DoorCode_Scroll_1_4_Green(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 1) = 2u8;
    *ram_ptr!(game; 0xcd20, 4) = 2u8;
}
pub unsafe fn DoorCode_Scroll_2_Blue(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 2) = 1u8;
}
pub unsafe fn DoorCode_Scroll_17_Blue(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 23) = 1u8;
}
pub unsafe fn DoorCode_Scroll_4_Blue(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 4) = 1u8;
}
pub unsafe fn DoorCode_Scroll_6_Green(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 6) = 2u8;
}
pub unsafe fn DoorCode_Scroll_3_Green(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 3) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_0(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 24) = 2u8;
    *ram_ptr!(game; 0xcd20, 28) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_1(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 5) = 1u8;
    *ram_ptr!(game; 0xcd20, 6) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_2(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 29) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_3(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 2) = 2u8;
    *ram_ptr!(game; 0xcd20, 3) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_4(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 0u8;
    *ram_ptr!(game; 0xcd20, 1) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_5(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 11) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_6(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 28) = 0u8;
    *ram_ptr!(game; 0xcd20, 29) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_7(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 4) = 0u8;
}
pub unsafe fn DoorCode_SetScroll_8(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 32) = 2u8;
    *ram_ptr!(game; 0xcd20, 36) = 2u8;
    *ram_ptr!(game; 0xcd20, 37) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_9(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 2) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_10(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_11(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 6) = 2u8;
    *ram_ptr!(game; 0xcd20, 7) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_12(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 1) = 1u8;
    *ram_ptr!(game; 0xcd20, 2) = 0u8;
}
pub unsafe fn DoorCode_SetScroll_13(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 1) = 1u8;
    *ram_ptr!(game; 0xcd20, 3) = 0u8;
}
pub unsafe fn DoorCode_SetScroll_14(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 0u8;
    *ram_ptr!(game; 0xcd20, 4) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_15(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 2) = 1u8;
    *ram_ptr!(game; 0xcd20, 3) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_16(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 2u8;
    *ram_ptr!(game; 0xcd20, 1) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_17(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 1) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_18(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 15) = 2u8;
    *ram_ptr!(game; 0xcd20, 18) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_19(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 6) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_20(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 2u8;
    *ram_ptr!(game; 0xcd20, 1) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_21(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 2) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_22(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 3) = 0u8;
    *ram_ptr!(game; 0xcd20, 4) = 0u8;
    *ram_ptr!(game; 0xcd20, 6) = 1u8;
    *ram_ptr!(game; 0xcd20, 7) = 1u8;
    *ram_ptr!(game; 0xcd20, 8) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_23(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 1) = 1u8;
    *ram_ptr!(game; 0xcd20, 2) = 1u8;
    *ram_ptr!(game; 0xcd20, 3) = 1u8;
    *ram_ptr!(game; 0xcd20, 4) = 2u8;
    *ram_ptr!(game; 0xcd20, 6) = 0u8;
}
pub unsafe fn DoorCode_SetScroll_24(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 1u8;
    *ram_ptr!(game; 0xcd20, 1) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_25(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 1) = 0u8;
    *ram_ptr!(game; 0xcd20, 0) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_26(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 10) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_27(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 1u8;
    *ram_ptr!(game; 0xcd20, 2) = 0u8;
}
pub unsafe fn DoorCode_SetScroll_28(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 2u8;
    *ram_ptr!(game; 0xcd20, 2) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_29(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 6) = 1u8;
    *ram_ptr!(game; 0xcd20, 7) = 1u8;
    *ram_ptr!(game; 0xcd20, 8) = 0u8;
}
pub unsafe fn DoorCode_SetScroll_30(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 2) = 0u8;
    *ram_ptr!(game; 0xcd20, 3) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_31(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 7) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_32(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 1) = 0u8;
    *ram_ptr!(game; 0xcd20, 2) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_33(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 3) = 0u8;
    *ram_ptr!(game; 0xcd20, 0) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_34(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 1) = 1u8;
    *ram_ptr!(game; 0xcd20, 4) = 0u8;
}
pub unsafe fn DoorCode_SetScroll_35(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 1u8;
    *ram_ptr!(game; 0xcd20, 1) = 0u8;
    *ram_ptr!(game; 0xcd20, 2) = 0u8;
    *ram_ptr!(game; 0xcd20, 3) = 0u8;
}
pub unsafe fn DoorCode_SetScroll_36(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_37(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 1u8;
    *ram_ptr!(game; 0xcd20, 1) = 1u8;
    *ram_ptr!(game; 0xcd20, 4) = 0u8;
}
pub unsafe fn DoorCode_SetScroll_38(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 1u8;
    *ram_ptr!(game; 0xcd20, 3) = 0u8;
}
pub unsafe fn DoorCode_SetScroll_39(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_40(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 1u8;
    *ram_ptr!(game; 0xcd20, 1) = 0u8;
}
pub unsafe fn DoorCode_SetScroll_41(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 24) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_42(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 3) = 0u8;
    *ram_ptr!(game; 0xcd20, 2) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_43(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 14) = 0u8;
}
pub unsafe fn RoomCode_ScrollingSkyLand_(game: &mut Game) {
    RoomCode_ScrollingSkyLand(game);
}
pub unsafe fn RoomCode_ScrollingSkyOcean_(game: &mut Game) {
    RoomMainAsm_ScrollingSkyOcean(game);
}
pub unsafe fn RoomCode_GenRandomExplodes_Nonblank(game: &mut Game) {
    if ram_word!(game, time_is_frozen_flag) == 0
        && (ram_word!(game, nmi_frame_counter_word) as i32 & 1i32) == 0i32
    {
        let mut Random: u16 = NextRandom(game);
        let x: u16 = (ram_word!(game, layer1_x_pos)).wrapping_add((Random as u8) as u16);
        let y: u16 = (ram_word!(game, layer1_y_pos))
            .wrapping_add((*(&raw mut Random as *mut u8).offset(1isize)) as u16);
        let prod: u16 = Mult8x8(
            (y as i32 >> 4i32) as u8,
            ram_word!(game, room_width_in_blocks) as u8,
        );
        if (*(ram_ptr!(game, u16; 0x10002)).offset((prod as i32 + (x as i32 >> 4i32)) as isize)
            as i32
            & 0x3ffi32)
            != 255i32
        {
            RoomCode_GenRandomExplodes(x, y, game);
        }
    }
}
pub unsafe fn RoomCode_ScrollingSkyLand_Shakes(game: &mut Game) {
    RoomCode_ScrollingSkyLand(game);
    RoomCode_GenRandomExplodes_Nonblank(game);
    update_ram_word!(game.earthquake_timer, |fresh17| (fresh17) | (0x8000u16));
}
pub unsafe fn RoomCode_ExplodeShakes(game: &mut Game) {
    RoomCode_GenRandomExplodes_Nonblank(game);
    update_ram_word!(game.earthquake_timer, |fresh16| (fresh16) | (0x8000u16));
}
pub unsafe fn RoomCode_GenRandomExplodes_4th(game: &mut Game) {
    if ram_word!(game, time_is_frozen_flag) == 0
        && (ram_word!(game, nmi_frame_counter_word) as i32 & 3i32) == 0i32
    {
        let mut Random: u16 = NextRandom(game);
        RoomCode_GenRandomExplodes(
            (ram_word!(game, layer1_x_pos)).wrapping_add((Random as u8) as u16),
            (ram_word!(game, layer1_y_pos))
                .wrapping_add((*(&raw mut Random as *mut u8).offset(1isize)) as u16),
            game,
        );
    }
}
pub unsafe fn RoomCode_GenRandomExplodes(x_r18: u16, y_r20: u16, game: &mut Game) {
    static kRoomCode_GenRandomExplodes_Sprite: [u8; 8] = [3, 3, 9, 12, 12, 18, 18, 21];
    static kRoomCode_GenRandomExplodes_Sfx: [u8; 8] = [0x24, 0, 0, 0x25, 0, 0, 0, 0];
    let v1: u16 = (NextRandom(game)) & (0xfu16);
    if v1 < 8u16 {
        let v2: u16 = kRoomCode_GenRandomExplodes_Sfx[v1 as usize] as u16;
        if v2 != 0 {
            QueueSfx2_Max6(v2, game);
        }
    }
    CreateSpriteAtPos(
        x_r18,
        y_r20,
        kRoomCode_GenRandomExplodes_Sprite[(v1 as i32 & 7i32) as usize] as u16,
        0u16,
        game,
    );
}
pub unsafe fn RoomCode_ScrollRightDachora(game: &mut Game) {
    if *ram_ptr!(game; 0xcd20, 11) == 2u8
        && ram_word!(game, layer1_y_pos) < 0x500u16
        && ram_word!(game, layer1_x_pos) < 0x380u16
    {
        update_ram_word!(game.layer1_x_pos, |fresh15| (fresh15 as i32
            + ((ram_word!(game, layer1_x_pos) >= 0x380u16) as i32 + 3i32))
            as u16);
    }
}
pub unsafe fn RoomCode_8FC8C8(game: &mut Game) {
    SpawnEprojWithRoomGfx(addr_kEproj_PrePhantomRoom as u16, 0u16, game);
}
pub unsafe fn RoomCode_SetupShaktoolRoomPlm(game: &mut Game) {
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0u8, 0u8, 0xb8eb as VoidP), game);
}
pub unsafe fn RoomCode_SetPauseCodeForDraygon(game: &mut Game) {
    mov24(
        ram_ptr!(game, LongPtr; 0x601),
        fnPauseHook_DraygonRoom as u32,
    );
    mov24(
        ram_ptr!(game, LongPtr; 0x604),
        fnUnpauseHook_DraygonRoom as u32,
    );
}
pub unsafe fn PauseHook_DraygonRoom(game: &mut Game) {
    set_ram_word!(game.irqhandler_next_handler = 4u16);
}
pub unsafe fn UnpauseHook_DraygonRoom(game: &mut Game) -> CoroutineRet {
    if ram_word!(game, hdma_object_channels_bitmask[1]) == 8u16 {
        set_ram_word!(game.irqhandler_next_handler = 12u16);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn RoomCode_SetCollectedMap(game: &mut Game) {
    let v0: u16 = (*(ram_ptr!(game, u16; 0xd908, ram_word!(game, area_index) as isize))) | (1u16);
    *(ram_ptr!(game, u16; 0xd908, ram_word!(game, area_index) as isize)) = v0;
    set_ram_word!(game.has_area_map = v0);
}
pub unsafe fn RoomCode_SetZebesTimebombEvent(game: &mut Game) {
    SetEventHappened(0xeu16, game);
    set_ram_word!(game.earthquake_type = 18u16);
    set_ram_word!(game.earthquake_timer = -1i32 as u16);
}
pub unsafe fn RoomCode_SetLightHorizRoomShake(game: &mut Game) {
    set_ram_word!(game.earthquake_type = 18u16);
    *(ram_ptr!(game, u16; 0x7e1, 2)) = 18u16;
    *(ram_ptr!(game, u16; 0x7e1)) = 0u16;
    set_ram_word!(game.earthquake_timer = -1i32 as u16);
}
pub unsafe fn RoomCode_SetMediumHorizRoomShake(game: &mut Game) {
    set_ram_word!(game.earthquake_type = 21u16);
    set_ram_word!(game.earthquake_timer = -1i32 as u16);
}
pub unsafe fn RoomCode_Escape4_SetMediumHorizRoomShake(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x10u8, 0x10u8, 0xb968 as VoidP),
        game,
    );
    set_ram_word!(game.earthquake_type = 21u16);
    *(ram_ptr!(game, u16; 0x7e1, 2)) = 21u16;
    *(ram_ptr!(game, u16; 0x7e1)) = 0u16;
    set_ram_word!(game.earthquake_timer = -1i32 as u16);
}
pub unsafe fn RoomCode_SetCeresDoorSolid(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0xfu8, 0x26u8, 0xba48 as VoidP),
        game,
    );
    FxTypeFunc_2C_Haze(game);
}
pub unsafe fn RoomCode_CeresColorMathHdma(game: &mut Game) {
    FxTypeFunc_2C_Haze(game);
}
pub unsafe fn RoomCode_CeresColorMathHdma_BgBase(game: &mut Game) {
    *ram_ptr!(game; 0x5d) = 102u8;
    FxTypeFunc_2C_Haze(game);
    set_ram_word!(game.hdma_data_table_in_ceres = 9u16);
}
pub unsafe fn DoorCode_StartWreckedSkipTreadmill_East(game: &mut Game) {
    SpawnAnimtiles(addr_kAnimtiles_WreckedShipTradmillLeft as u16, game);
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x4u8, 0x9u8, 0xb64f as VoidP),
        game,
    );
}
pub unsafe fn DoorCode_SetScroll_44(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 1) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_45(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_46(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 3) = 0u8;
    *ram_ptr!(game; 0xcd20, 4) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_47(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 41) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_48(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 40) = 2u8;
    *ram_ptr!(game; 0xcd20, 46) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_49(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 6) = 0u8;
    *ram_ptr!(game; 0xcd20, 7) = 0u8;
    *ram_ptr!(game; 0xcd20, 8) = 0u8;
    *ram_ptr!(game; 0xcd20, 9) = 0u8;
    *ram_ptr!(game; 0xcd20, 10) = 0u8;
    *ram_ptr!(game; 0xcd20, 11) = 0u8;
}
pub unsafe fn DoorCode_SetupElevatubeFromSouth(game: &mut Game) {
    *(ram_ptr!(game, u16; 0x7e1, 4)) = -256i32 as u16;
    *(ram_ptr!(game, u16; 0x7e1, 2)) = 2496u16;
    *(ram_ptr!(game, u16; 0x7e1, 6)) = -32i32 as u16;
    CallSomeSamusCode(0u16, game);
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x1u8, 0u8, 0xb8f9 as VoidP),
        game,
    );
}
pub unsafe fn DoorCode_SetupElevatubeFromNorth(game: &mut Game) {
    *(ram_ptr!(game, u16; 0x7e1, 4)) = 256u16;
    strcpy(
        ram_ptr!(game, c_char; 0x7e1, 2),
        b"@\0" as *const u8 as *const c_char,
    );
    strcpy(
        ram_ptr!(game, c_char; 0x7e1, 6),
        b" \0" as *const u8 as *const c_char,
    );
    CallSomeSamusCode(0u16, game);
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x1u8, 0u8, 0xb8f9 as VoidP),
        game,
    );
}
pub unsafe fn RoomCode_Elevatube(game: &mut Game) {
    let mut v0: u16 = 0;
    set_ram_word!(game.samus_x_pos = 128u16);
    set_ram_word!(game.samus_x_subpos = 0u16);
    let amt: i32 = (*(ram_ptr!(game, u16; 0x7e1, 4)) as i16 as i32) << 8i32;
    v0 = ((amt as u32).wrapping_add(*(ram_ptr!(game, u32; 0x7e1))) >> 16i32) as u16;
    let ref mut fresh13 = *(ram_ptr!(game, u16; 0x7e1));
    *fresh13 = (*fresh13 as i32 + amt) as u16;
    *(ram_ptr!(game, u16; 0x7e1, 2)) = v0;
    Samus_MoveDown_NoSolidColl(amt, game);
    if ((*(ram_ptr!(game, u16; 0x7e1, 6)) as i32
        + *(ram_ptr!(game, u16; 0x7e1, 4)) as i32
        + 3616i32) as u16)
        < 0x1c41u16
    {
        let ref mut fresh14 = *(ram_ptr!(game, u16; 0x7e1, 4));
        *fresh14 = (*fresh14).wrapping_add(*(ram_ptr!(game, u16; 0x7e1, 6)));
    }
}
pub unsafe fn DoorCode_ResetElevatubeNorthExit(game: &mut Game) {
    CallSomeSamusCode(1u16, game);
}
pub unsafe fn DoorCode_ResetElevatubeSouthExit(game: &mut Game) {
    *(ram_ptr!(game, u16; 0xcd20)) = 514u16;
    CallSomeSamusCode(1u16, game);
}
pub unsafe fn DoorCode_SetScroll_50(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 10) = 0u8;
    *ram_ptr!(game; 0xcd20, 11) = 1u8;
}
pub unsafe fn UNUSED_DoorCode_SetScroll_51(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 17) = 0u8;
    *ram_ptr!(game; 0xcd20, 16) = 1u8;
}
pub unsafe fn UNUSED_DoorCode_SetScroll_52(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 4) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_53(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 0u8;
    *ram_ptr!(game; 0xcd20, 4) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_54(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 0u8;
    *ram_ptr!(game; 0xcd20, 1) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_55(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 9) = 0u8;
    *ram_ptr!(game; 0xcd20, 10) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_56(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 0u8;
    *ram_ptr!(game; 0xcd20, 2) = 0u8;
    *ram_ptr!(game; 0xcd20, 1) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_57(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 1) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_58(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 6) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_59(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 4) = 0u8;
}
pub unsafe fn DoorCode_SetScroll_60(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 1) = 0u8;
}
pub unsafe fn DoorCode_SetScroll_61(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 4) = 0u8;
    *ram_ptr!(game; 0xcd20, 7) = 0u8;
}
pub unsafe fn DoorCode_SetScroll_62(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 2) = 0u8;
    *ram_ptr!(game; 0xcd20, 1) = 1u8;
}
pub unsafe fn DoorCode_SetScroll_63(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 2u8;
    *ram_ptr!(game; 0xcd20, 2) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_64(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 0) = 2u8;
    *ram_ptr!(game; 0xcd20, 1) = 2u8;
}
pub unsafe fn DoorCode_SetScroll_65(game: &mut Game) {
    *ram_ptr!(game; 0xcd20, 24) = 1u8;
    *ram_ptr!(game; 0xcd20, 25) = 0u8;
}
pub unsafe fn DoorCode_CeresElevatorShaft(game: &mut Game) {
    crate::rtl::write_register_and_mirror(BGMODE as u16, 7, 0x56, true, game);
    set_ram_words! { game, 256u16; reg_M7A, reg_M7D }
    set_ram_words! { game, 0u16; reg_M7B, reg_M7C }
    set_ram_word!(game.reg_M7X = 128u16);
    set_ram_word!(game.reg_M7Y = 1008u16);
    set_ram_word!(game.irq_enable_mode7 = 1u16);
    strcpy(
        ram_ptr!(game, c_char; 0x7e1),
        b"\"\0" as *const u8 as *const c_char,
    );
    strcpy(
        ram_ptr!(game, c_char; 0x7e1, 2),
        b"<\0" as *const u8 as *const c_char,
    );
}
pub unsafe fn DoorCode_CeresElevatorShaft_2(game: &mut Game) {
    *ram_ptr!(game; 0x56) = 9u8;
    set_ram_word!(game.irq_enable_mode7 = 0u16);
}
pub unsafe fn RoomCode_CeresElevatorShaft_(game: &mut Game) {
    RoomCode_CeresElevatorShaft(game);
}
static kRoomCode_SpawnCeresFallingDebris_Tab: [u16; 16] = [
    0x50, 0x60, 0x70, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0x110, 0x130, 0x150, 0x170,
    0x190,
];
pub unsafe fn RoomCode_SpawnCeresFallingDebris(game: &mut Game) {
    if ram_word!(game, ceres_status) != 0u16 && {
        let ref mut fresh12 = *(ram_ptr!(game, u16; 0x7e1));
        *fresh12 = (*fresh12).wrapping_sub(1);
        *(ram_ptr!(game, i16; 0x7e1)) < 0i16
    } {
        *ram_ptr!(game; 0x7e1, 0) = 8u8;
        *ram_ptr!(game; 0x7e1, 1) = 0u8;
        let v0: u16 = (if ram_word!(game, random_number) as i32 & 0x8000i32 != 0 {
            addr_stru_869742 as i32
        } else {
            addr_stru_869734 as i32
        }) as u16;
        SpawnEprojWithRoomGfx(
            v0,
            kRoomCode_SpawnCeresFallingDebris_Tab
                [(ram_word!(game, random_number) as i32 & 0xfi32) as usize],
            game,
        );
    }
}
pub unsafe fn RoomCode_HandleCeresRidleyGetaway(game: &mut Game) {
    if ram_word!(game, ceres_status) as i32 & 1i32 != 0 {
        HandleCeresRidleyGetawayCutscene(game);
    }
}
pub unsafe fn RoomCode_ShakeScreenHorizDiag(game: &mut Game) {
    let mut v0: u16 = 0;
    let current_block_3: u64;
    if *(ram_ptr!(game, u16; 0x7e1)) != 0 {
        let ref mut fresh11 = *(ram_ptr!(game, u16; 0x7e1));
        *fresh11 = (*fresh11).wrapping_sub(1);
        if *fresh11 == 0 {
            v0 = 18u16;
            current_block_3 = 10175730460759437497;
        } else {
            current_block_3 = 11006700562992250127;
        }
    } else if NextRandom(game) < 0x200u16 {
        strcpy(
            ram_ptr!(game, c_char; 0x7e1),
            b"*\0" as *const u8 as *const c_char,
        );
        v0 = 23u16;
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
    let mut v0: i16 = 0;
    let current_block_3: u64;
    if *(ram_ptr!(game, u16; 0x7e1)) != 0 {
        let ref mut fresh10 = *(ram_ptr!(game, u16; 0x7e1));
        *fresh10 = (*fresh10).wrapping_sub(1);
        if *fresh10 == 0 {
            v0 = 21i16;
            current_block_3 = 1544504899801662111;
        } else {
            current_block_3 = 11006700562992250127;
        }
    } else if NextRandom(game) < 0x180u16 {
        strcpy(
            ram_ptr!(game, c_char; 0x7e1),
            b"*\0" as *const u8 as *const c_char,
        );
        v0 = 26i16;
        current_block_3 = 1544504899801662111;
    } else {
        current_block_3 = 11006700562992250127;
    }
    if current_block_3 == 1544504899801662111 {
        *(ram_ptr!(game, u16; 0x7e1, 2)) = v0 as u16;
    }
    set_ram_word!(game.earthquake_type = *(ram_ptr!(game, u16; 0x7e1, 2)));
    RoomCode_GenRandomExplodes_4th(game);
}
pub unsafe fn CallRoomDefStateSelect(ea: u32, k: u16, game: &mut Game) -> u16 {
    match ea {
        9430502 => RoomDefStateSelect_Finish(k, game),
        9430527 => RoomDefStateSelect_TourianBoss01(k, game),
        9430546 => RoomDefStateSelect_IsEventSet(k, game),
        9430569 => RoomDefStateSelect_IsBossDead(k, game),
        9430610 => RoomDefStateSelect_MorphBallMissiles(k, game),
        9430633 => RoomDefStateSelect_PowerBombs(k, game),
        _ => Unreachable(game) as u16,
    }
}
pub unsafe fn HandleRoomDefStateSelect(k: u16, game: &mut Game) {
    let mut v1: u16 = (k).wrapping_add(11u16);
    loop {
        let event_pointer: u16 = (*get_RoomDefStateSelect_E6E5_Finish(v1, game)).code_ptr;
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
pub unsafe fn RoomDefStateSelect_Finish(k: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.roomdefroomstate_ptr = k);
    0u16
}
pub unsafe fn RoomDefStateSelect_TourianBoss01(k: u16, game: &mut Game) -> u16 {
    if CheckBossBitForCurArea(1u16, game) as i32 & 1i32 == 0 {
        return (k).wrapping_add(2u16);
    }
    let v1: *const u16 = RomPtr_8F(k, game) as *const u16;
    RoomDefStateSelect_Finish(*v1, game)
}
pub unsafe fn RoomDefStateSelect_IsEventSet(k: u16, game: &mut Game) -> u16 {
    let v1: *const u8 = RomPtr_8F(k, game);
    if CheckEventHappened(*v1 as u16, game) != 0 {
        RoomDefStateSelect_Finish(*(v1.offset(1) as *mut u16), game)
    } else {
        (k).wrapping_add(3u16)
    }
}
pub unsafe fn RoomDefStateSelect_IsBossDead(k: u16, game: &mut Game) -> u16 {
    let v1: *const u8 = RomPtr_8F(k, game);
    if CheckBossBitForCurArea(*v1 as u16, game) as i32 & 1i32 != 0 {
        RoomDefStateSelect_Finish(*(v1.offset(1) as *mut u16), game)
    } else {
        (k).wrapping_add(3u16)
    }
}
pub unsafe fn RoomDefStateSelect_MorphBallMissiles(k: u16, game: &mut Game) -> u16 {
    if (ram_word!(game, collected_items) as i32 & 4i32) == 0i32
        || ram_word!(game, samus_max_missiles) == 0
    {
        return (k).wrapping_add(2u16);
    }
    let v1: *const u16 = RomPtr_8F(k, game) as *const u16;
    RoomDefStateSelect_Finish(*v1, game)
}
pub unsafe fn RoomDefStateSelect_PowerBombs(k: u16, game: &mut Game) -> u16 {
    if ram_word!(game, samus_max_power_bombs) == 0 {
        return (k).wrapping_add(2u16);
    }
    let v1: *const u16 = RomPtr_8F(k, game) as *const u16;
    RoomDefStateSelect_Finish(*v1, game)
}
crate::game::address_dispatch! {
    pub unsafe fn CallRoomSetupCode(ea: u32, game: &mut Game) => ea, game {
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
    }
}
pub unsafe fn RunRoomSetupCode(game: &mut Game) {
    let mut RoomDefRoomstate: *mut RoomDefRoomstate = null_mut::<RoomDefRoomstate>();
    RoomDefRoomstate = get_RoomDefRoomstate(ram_word!(game, roomdefroomstate_ptr), game);
    if (*RoomDefRoomstate).room_setup_code != 0 {
        CallRoomSetupCode(
            (((*RoomDefRoomstate).room_setup_code) as u32) | (0x8f0000u32),
            game,
        );
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallDoorDefSetupCode(ea: u32, game: &mut Game) => ea, game {
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
    }
}
pub unsafe fn RunDoorSetupCode(game: &mut Game) {
    let DD: *mut DoorDef = get_DoorDef(ram_word!(game, door_def_ptr), game);
    if (*DD).door_setup_code != 0 {
        CallDoorDefSetupCode((((*DD).door_setup_code) as u32) | (0x8f0000u32), game);
    }
}
pub unsafe fn RoomCode_CrocomireShaking(game: &mut Game) {
    let mut ai_var_D: u16 = 0;
    if ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).properties as i32 & 0x200i32) == 0i32 {
        if (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).ai_var_A == 64u16 {
            set_ram_word!(
                game.reg_BG1VOFS = ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).ai_var_D as i32
                    + ram_word!(game, bg1_y_offset) as i32
                    + ram_word!(game, layer1_y_pos) as i32)
                    as u16
            );
        } else if ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).ai_var_B as i32 & 0x400i32)
            != 0i32
        {
            let fresh5 = &raw mut (*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).ai_var_D;
            fresh5.write_unaligned(fresh5.read_unaligned().wrapping_sub(1));
            if fresh5.read_unaligned() as i32 + 7i32 & 0x8000i32 != 0 {
                ai_var_D = ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).ai_var_D as i32
                    - 2i32
                        * ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).ai_var_D as i32 + 7i32))
                    as u16;
            } else {
                ai_var_D = (*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).ai_var_D;
            }
            update_ram_word!(game.reg_BG1VOFS, wrapping_add(ai_var_D));
            set_ram_word!(game.reg_BG2VOFS = (ai_var_D).wrapping_sub(48u16));
        } else if (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).ai_var_C == 34u16
            && (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).ai_var_D != 0u16
        {
            let fresh7 = &raw mut (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).ai_var_D;
            fresh7.write_unaligned(fresh7.read_unaligned().wrapping_sub(1));
            if ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).ai_var_D as i32 & 1i32) != 0i32 {
                update_ram_word!(game.layer1_x_pos, wrapping_sub(4u16));
            } else {
                update_ram_word!(game.layer1_x_pos, wrapping_add(4u16));
            }
        }
    }
}
static kRoomCode_RidleyRoomShaking_X: [i16; 8] = [0, 2, 2, 2, 0, -2, -2, -2];
static kRoomCode_RidleyRoomShaking_Y: [i16; 8] = [-2, -2, 0, 2, 2, 2, 0, -2];
pub unsafe fn RoomCode_RidleyRoomShaking(game: &mut Game) {
    let ai_var_A: u16 = (*(ram_ptr!(game, EnemyData; 0xf78)).offset(4)).ai_var_A;
    if (*(ram_ptr!(game, EnemyData; 0xf78)).offset(4)).ai_var_A != 0 {
        let fresh0 = &raw mut (*(ram_ptr!(game, EnemyData; 0xf78)).offset(4)).ai_var_A;
        fresh0.write_unaligned(fresh0.read_unaligned().wrapping_sub(1));
        let v1: i32 = ai_var_A as i32 - 1i32;
        update_ram_word!(
            game.reg_BG1HOFS,
            wrapping_add((kRoomCode_RidleyRoomShaking_X[v1 as usize]) as u16)
        );
        update_ram_word!(
            game.reg_BG2HOFS,
            wrapping_add((kRoomCode_RidleyRoomShaking_X[v1 as usize]) as u16)
        );
        update_ram_word!(
            game.reg_BG1VOFS,
            wrapping_add((kRoomCode_RidleyRoomShaking_Y[v1 as usize]) as u16)
        );
        update_ram_word!(
            game.reg_BG2VOFS,
            wrapping_add((kRoomCode_RidleyRoomShaking_Y[v1 as usize]) as u16)
        );
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallRoomCode(ea: u32, game: &mut Game) => ea, game {
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
