pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_84::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::samus_poses::*;
use crate::game::sm_80::{
    AddGrappleToHudTilemap, AddMissilesToHudTilemap, AddPowerBombsToHudTilemap,
    AddSuperMissilesToHudTilemap, AddXrayToHudTilemap, CheckBossBitForCurArea, CheckEventHappened,
    PrepareBitAccess, QueueMusic_Delayed8, QueueSfx1_Max1, QueueSfx1_Max15, QueueSfx1_Max3,
    QueueSfx1_Max6, QueueSfx1_Max9, QueueSfx2_Max1, QueueSfx2_Max15, QueueSfx2_Max3,
    QueueSfx2_Max6, QueueSfx2_Max9, QueueSfx3_Max1, QueueSfx3_Max15, QueueSfx3_Max3,
    QueueSfx3_Max6, QueueSfx3_Max9, SetEventHappened,
};
use crate::game::sm_81::SaveToSram;
use crate::game::sm_82::PlayRoomMusicTrackAfterAFrames;
use crate::game::sm_85::{DisplayMessageBox, DisplayMessageBox_Async, DisplayMessageBox_Poll};
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_90::{CallSomeSamusCode, UpdateBeamTilesAndPalette};
use crate::game::sm_91::{
    GravitySuitPickup, LoadBlockToXrayTilemap, MakeSamusFaceForward, VariaSuitPickup,
};
use crate::game_state::Game;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::abs16;
use crate::types::TileDrawOrder;
use ::core::ffi::{c_char, c_long, c_void};
use ::core::ptr::{null, null_mut};

use crate::rtl::{Load24, Mult8x8, RomPtr, SnesDivide, SnesModulus, Unreachable};
extern "C" {
    fn __assert_fail(
        __assertion: *const c_char,
        __file: *const c_char,
        __line: u32,
        __function: *const c_char,
    ) -> !;
    fn printf(__format: *const c_char, ...) -> i32;

}
pub use crate::types::{CoroutineRet, Func_Y_V, LongPtr, VoidP, VramWriteEntry};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct PlmHeader_Size4 {
    pub func_ptr: VoidP,
    pub instr_list_ptr: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct XraySpecialCasing {
    pub x_block: u8,
    pub y_block: u8,
    pub level_data_block: u16,
}
pub use crate::types::Buttons;
pub use crate::types::Consts_84;
pub use crate::types::Consts_86;
pub use crate::types::SpawnHardcodedPlmArgs;
pub use crate::types::{Consts_60, RoomDefRoomstate, TileTable, TileTables};
pub use crate::types::{EnemyData, RoomPlmEntry, SamusPose};
pub const NULL: *mut c_void = null_mut::<c_void>();
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
pub const kCoroutineNone: i32 = 0;
#[inline]
unsafe fn RomPtr_RAM(addr: u16, game: &mut Game) -> *mut u8 {
    '_c2rust_label: {
        if addr < 0x2000u16 {
        } else {
            __assert_fail(
                b"addr < 0x2000\0" as *const u8 as *const c_char,
                b"src/sm_rtl.h\0" as *const u8 as *const c_char,
                51u32,
                b"uint8 *RomPtr_RAM(uint16_t)\0" as *const u8 as *const c_char,
            );
        }
    };
    (&raw mut game.g_ram as *mut u8).offset(addr as i32 as isize)
}
const RomPtr_84: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x84>;
const RomPtr_8F: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8f>;
#[inline]
unsafe fn RomBankBase(bank: u8, game: &mut Game) -> *const u8 {
    RomPtr((((bank as i32) << 16i32) + 0x8000i32) as u32, game).offset(-(0x8000i32 as isize))
}
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0i32 + a) as isize];
}
#[inline]
unsafe fn get_RoomDefRoomstate(a: u16, game: &mut Game) -> *mut RoomDefRoomstate {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomDefRoomstate
}
#[inline]
unsafe fn get_RoomPlmEntry(a: u16, game: &mut Game) -> *mut RoomPlmEntry {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomPlmEntry
}
#[inline]
unsafe fn get_PlmHeader_Size4(a: u16, game: &mut Game) -> *mut PlmHeader_Size4 {
    RomPtr((0x840000u32) | ((a) as u32), game) as *mut PlmHeader_Size4
}
pub const fnPlmPreInstr_nullsub_60: u32 = 8684649;
pub const fnPlmPreInstr_Empty2: i32 = 0x8484e6;
pub const fnPlmPreInstr_Empty3: i32 = 0x84853d;
pub const fnPlmInstr_Sleep: u32 = 8685236;
pub const fnPlmInstr_Delete: u32 = 8685244;
pub const fnPlmInstr_PreInstr: u32 = 8685249;
pub const fnPlmInstr_ClearPreInstr: u32 = 8685258;
pub const fnPlmPreInstr_Empty: i32 = 0x8486d0;
pub const fnPlmInstr_CallFunction: u32 = 8685323;
pub const fnPlmInstr_Goto: u32 = 8685348;
pub const fnPlmInstr_DecrementAndBranchNonzero: u32 = 8685375;
pub const fnPlmInstr_SetTimer: u32 = 8685390;
pub const fnPlmInstr_LoadItemPlmGfx: u32 = 8685412;
pub const fnPlmInstr_CopyFromRamToVram: u32 = 8685541;
pub const fnPlmInstr_GotoIfBossBitSet: u32 = 8685582;
pub const fnPlmInstr_GotoIfEventSet: u32 = 8685613;
pub const fnPlmInstr_SetEvent: u32 = 8685630;
pub const fnPlmInstr_GotoIfChozoSet: u32 = 8685640;
pub const fnPlmInstr_SetRoomChozoBit: u32 = 8685669;
pub const fnPlmInstr_GotoIfItemBitSet: u32 = 8685692;
pub const fnPlmInstr_SetItemBit: u32 = 8685721;
pub const fnPlmInstr_PickupBeamAndShowMessage: u32 = 8685744;
pub const fnPlmInstr_PickupEquipmentAndShowMessage: u32 = 8685811;
pub const fnPlmInstr_PickupEquipmentAddGrappleShowMessage: u32 = 8685850;
pub const fnPlmInstr_PickupEquipmentAddXrayShowMessage: u32 = 8685889;
pub const fnPlmInstr_CollectHealthEnergyTank: u32 = 8685928;
pub const fnPlmInstr_CollectHealthReserveTank: u32 = 8685958;
pub const fnPlmInstr_CollectAmmoMissileTank: u32 = 8685993;
pub const fnPlmInstr_CollectAmmoSuperMissileTank: u32 = 8686034;
pub const fnPlmInstr_CollectAmmoPowerBombTank: u32 = 8686075;
pub const fnPlmInstr_SetLinkReg: u32 = 8686116;
pub const fnPlmInstr_Call: u32 = 8686126;
pub const fnPlmInstr_Return: u32 = 8686138;
pub const fnPlmInstr_GotoIfDoorBitSet: u32 = 8686194;
pub const fnPlmInstr_IncrementDoorHitCounterAndJGE: u32 = 8686225;
pub const fnPlmInstr_IncrementArgumentAndJGE: u32 = 8686285;
pub const fnlocret_848AE0: u32 = 8686304;
pub const fnPlmInstr_SetBTS: u32 = 8686321;
pub const fnPlmInstr_DrawPlmBlock: u32 = 8686341;
pub const fnPlmInstr_DrawPlmBlock_: u32 = 8686359;
pub const fnPlmInstr_ProcessAirScrollUpdate: u32 = 8686421;
pub const fnPlmInstr_ProcessSolidScrollUpdate: u32 = 8686483;
pub const fnPlmInstr_QueueMusic: u32 = 8686545;
pub const fnPlmInstr_ClearMusicQueueAndQueueTrack: u32 = 8686557;
pub const fnPlmInstr_QueueSfx1_Max6: u32 = 8686599;
pub const fnPlmInstr_QueueSfx2_Max6: u32 = 8686608;
pub const fnPlmInstr_QueueSfx3_Max6: u32 = 8686617;
pub const fnPlmInstr_QueueSfx1_Max15: u32 = 8686626;
pub const fnPlmInstr_QueueSfx2_Max15: u32 = 8686635;
pub const fnPlmInstr_QueueSfx3_Max15: u32 = 8686644;
pub const fnPlmInstr_QueueSfx1_Max3: u32 = 8686653;
pub const fnPlmInstr_QueueSfx2_Max3: u32 = 8686662;
pub const fnPlmInstr_QueueSfx_Max3: u32 = 8686671;
pub const fnPlmInstr_QueueSfx1_Max9: u32 = 8686680;
pub const fnPlmInstr_QueueSfx2_Max9: u32 = 8686689;
pub const fnPlmInstr_QueueSfx3_Max9: u32 = 8686698;
pub const fnPlmInstr_QueueSfx1_Max1: u32 = 8686707;
pub const fnPlmInstr_QueueSfx2_Max1: u32 = 8686716;
pub const fnPlmInstr_QueueSfx3_Max1: u32 = 8686725;
pub const fnPlmInstr_ActivateMapStation: u32 = 8686735;
pub const fnPlmInstr_ActivateEnergyStation: u32 = 8686767;
pub const fnPlmInstr_ActivateMissileStation: u32 = 8686800;
pub const fnPlmInstr_ActivateSaveStationAndGotoIfNo: u32 = 8686833;
pub const fnPlmInstr_GotoIfSamusNear: u32 = 8686913;
pub const fnPlmInstr_MovePlmDownOneBlock: u32 = 8694528;
pub const fnnullsub_67: u32 = 8694567;
pub const fnPlmSetup_CrumbleBotwoonWall: u32 = 8694568;
pub const fnPlmInstr_Scroll_0_1_Blue: u32 = 8694609;
pub const fnPlmInstr_MovePlmDownOneBlock_0: u32 = 8694617;
pub const fnPlmInstr_ABD6: u32 = 8694742;
pub const fnPlmPreInstr_PositionSamusAndInvincible: u32 = 8694921;
pub const fnPlmInstr_DealDamage_2: u32 = 8694941;
pub const fnPlmInstr_GiveInvincibility: u32 = 8694961;
pub const fnPlmInstr_Draw0x38FramesOfRightTreadmill: u32 = 8695107;
pub const fnPlmInstr_Draw0x38FramesOfLeftTreadmill: u32 = 8695128;
pub const fnPlmInstr_GotoIfSamusHealthFull: u32 = 8695349;
pub const fnPlmInstr_GotoIfMissilesFull: u32 = 8695487;
pub const fnPlmInstr_PlaceSamusOnSaveStation: u32 = 8695822;
pub const fnPlmInstr_DisplayGameSavedMessageBox: u32 = 8695844;
pub const fnPlmInstr_EnableMovementAndSetSaveStationUsed: u32 = 8695856;
pub const fnPlmSetup_SetrupWreckedShipEntrance: i32 = 0x84b04a;
pub const fnPlmSetup_BTS_Brinstar_0x80_Floorplant: u32 = 8696028;
pub const fnPlmSetup_BTS_Brinstar_0x81_Ceilingplant: u32 = 8696083;
pub const fnPlmSetup_B6D3_MapStation: u32 = 8696203;
pub const fnPlmSetup_Bts47_MapStationRightAccess: u32 = 8696264;
pub const fnPlmSetup_Bts4_MapStationLeftAccess: u32 = 8696304;
pub const fnPlmSetup_PlmB6DF_EnergyStation: u32 = 8696349;
pub const fnPlmSetup_PlmB6EB_EnergyStation: u32 = 8696389;
pub const fnPlmSetup_B6E3_EnergyStationRightAccess: u32 = 8696429;
pub const fnPlmSetup_B6E7_EnergyStationLeftAccess: u32 = 8696477;
pub const fnPlmSetup_B6EF_MissileStationRightAccess: u32 = 8696528;
pub const fnPlmSetup_B6F3_MissileStationLeftAccess: u32 = 8696576;
pub const fnPlmSetup_B638_Rightwards_Extension: u32 = 8696634;
pub const fnPlmSetup_B63F_Leftwards_Extension: u32 = 8696645;
pub const fnPlmSetup_B643_Downwards_Extension: u32 = 8696656;
pub const fnPlmSetup_B647_Upwards_Extension: u32 = 8696667;
pub const fnPlmSetup_B703_ScrollPLM: u32 = 8696689;
pub const fnPlmSetup_B707_SolidScrollPLM: u32 = 8696706;
pub const fnPlmSetup_B6FF_ScrollBlockTouch: u32 = 8696723;
pub const fnPlmSetup_DeactivatePlm: u32 = 8696769;
pub const fnnullsub_290: u32 = 8696783;
pub const fnPlmSetup_ReturnCarryClear: u32 = 8696784;
pub const fnPlmSetup_ReturnCarrySet: u32 = 8696786;
pub const fnPlmSetup_D094_EnemyBreakableBlock: u32 = 8696788;
pub const fnUNUSED_sub_84B3E3: u32 = 8696803;
pub const fnPlmSetup_B70F_IcePhysics: u32 = 8696811;
pub const fnPlmSetup_QuicksandSurface: u32 = 8696840;
pub const fnPlmSetup_B71F_SubmergingQuicksand: u32 = 8696983;
pub const fnPlmSetup_B723_SandfallsSlow: u32 = 8697000;
pub const fnPlmSetup_B727_SandFallsFast: u32 = 8697014;
pub const fnPlmSetup_QuicksandSurfaceB: u32 = 8697028;
pub const fnPlmSetup_B737_SubmergingQuicksand: u32 = 8697153;
pub const fnPlmSetup_B73B_B73F_SandFalls: u32 = 8697167;
pub const fnPlmSetup_ClearShitroidInvisibleWall: u32 = 8697169;
pub const fnPlmSetup_B767_ClearShitroidInvisibleWall: u32 = 8697199;
pub const fnPlmSetup_B76B_SaveStationTrigger: u32 = 8697232;
pub const fnPlmSetup_B76F_SaveStation: u32 = 8697326;
pub const fnPlmSetup_MotherBrainRoomEscapeDoor: u32 = 8697336;
pub const fnPlmSetup_B7EB_EnableSoundsIn32Frames: u32 = 8697795;
pub const fnPlmPreInstr_B7EB_DecTimerEnableSoundsDeletePlm: i32 = 0x84b7dd;
pub const fnPlmPreInstr_WakeAndLavaIfBoosterCollected: u32 = 8697839;
pub const fnPlmPreInstr_WakePLMAndStartFxMotionSamusFarLeft: u32 = 8697898;
pub const fnPlmPreInstr_AdvanceLavaSamusMovesLeft: u32 = 8697926;
pub const fnPlmSetup_SpeedBoosterEscape: u32 = 8698012;
pub const fnPlmPreInstr_ShaktoolsRoom: u32 = 8698032;
pub const fnPlmSetup_ShaktoolsRoom: u32 = 8698076;
pub const fnnullsub_68: u32 = 8698095;
pub const fnPlmPreInstr_OldTourianEscapeShaftEscape: u32 = 8698151;
pub const fnPlmPreInstr_EscapeRoomBeforeOldTourianEscapeShaft: u32 = 8698184;
pub const fnPlmSetup_B974: u32 = 8698220;
pub const fnPlmSetup_B9C1_CrittersEscapeBlock: u32 = 8698232;
pub const fnPlmInstr_SetCrittersEscapedEvent: u32 = 8698297;
pub const fnPlmSetup_B9ED_CrittersEscapeBlock: u32 = 8698309;
pub const fnsub_84B9F1: u32 = 8698353;
pub const fnPlmInstr_JumpIfSamusHasNoBombs: u32 = 8698479;
pub const fnnullsub_84BAFA: u32 = 8698618;
pub const fnPlmSetup_BB30_CrateriaMainstreetEscape: u32 = 8698633;
pub const fnPlmInstr_MovePlmRight4Blocks: u32 = 8698661;
pub const fnPlmPreInstr_WakePlmIfTriggered: u32 = 8698706;
pub const fnnullsub_359: u32 = 8698730;
pub const fnPlmPreInstr_WakePlmIfTriggeredOrSamusBelowPlm: u32 = 8698731;
pub const fnPlmPreInstr_Empty5: i32 = 0x84bba3;
pub const fnPlmPreInstr_WakePlmIfTriggeredOrSamusAbovePlm: u32 = 8698788;
pub const fnPlmInstr_ClearTrigger: u32 = 8698845;
pub const fnPlmInstr_SpawnEproj: u32 = 8698849;
pub const fnPlmInstr_WakeEprojAtPlmPos: u32 = 8698864;
pub const fnPlmPreInstr_GoToLinkInstrIfShot: u32 = 8699151;
pub const fnPlmPreInstr_GoToLinkInstrIfShotWithPowerBomb: u32 = 8699174;
pub const fnPlmPreInstr_GoToLinkInstrIfShotWithAnyMissile: u32 = 8699216;
pub const fnPlmPreInstr_GoToLinkInstrIfShotWithSuperMissile: u32 = 8699272;
pub const fnPlmPreInstr_GoToLinkInstruction: u32 = 8699314;
pub const fnPlmPreInstr_GotoLinkIfBoss1Dead: u32 = 8699348;
pub const fnPlmPreInstr_GotoLinkIfMiniBossDead: u32 = 8699363;
pub const fnPlmPreInstr_GotoLinkIfTorizoDead: u32 = 8699378;
pub const fnPlmPreInstr_GotoLinkIfEnemyDeathQuotaOk: u32 = 8699393;
pub const fnPlmPreInstr_PlayDudSound: u32 = 8699420;
pub const fnPlmPreInstr_GotoLinkIfTourianStatueFinishedProcessing: u32 = 8699423;
pub const fnPlmPreInstr_GotoLinkIfCrittersEscaped: u32 = 8699440;
pub const fnPlmInstr_SetGreyDoorPreInstr: u32 = 8699455;
pub const fnPlmSetup_C806_LeftGreenGateTrigger: u32 = 8701261;
pub const fnPlmSetup_C80A_RightGreenGateTrigger: u32 = 8701292;
pub const fnPlmSetup_C80E_LeftRedGateTrigger: u32 = 8701323;
pub const fnPlmSetup_C812_RightRedGateTrigger: u32 = 8701359;
pub const fnPlmSetup_C81E_LeftYellowGateTrigger: u32 = 8701395;
pub const fnPlmSetup_C822_RightYellowGateTrigger: u32 = 8701425;
pub const fnPlmSetup_C816_LeftBlueGateTrigger: u32 = 8701456;
pub const fnPlmSetup_C81A_RightBlueGateTrigger: u32 = 8701479;
pub const fnPlmSetup_C82A_DownwardsClosedGate: u32 = 8701630;
pub const fnPlmSetup_C832_UpwardsClosedGate: u32 = 8701643;
pub const fnPlmSetup_C826_DownwardsOpenGate: u32 = 8701656;
pub const fnPlmSetup_C82E_UpwardsOpenGate: u32 = 8701660;
pub const fnPlmSetup_C836_DownwardsGateShootblock: u32 = 8701664;
pub const fnPlmSetup_C73A_UpwardsGateShootblock: u32 = 8701754;
pub const fnPlmSetup_C794_GreyDoor: u32 = 8701844;
pub const fnPlmSetup_Door_Colored: u32 = 8701873;
pub const fnPlmSetup_Door_Blue: u32 = 8701883;
pub const fnPlmSetup_Door_Strange: u32 = 8701922;
pub const fnPlmInstr_SetBtsTo1: u32 = 8703379;
pub const fnPlmSetup_D028_D02C_Unused: u32 = 8703426;
pub const fnPlmSetup_RespawningSpeedBoostBlock: u32 = 8703466;
pub const fnPlmSetup_RespawningCrumbleBlock: u32 = 8703543;
pub const fnPlmSetup_RespawningShotBlock: u32 = 8703595;
pub const fnPlmSetup_RespawningBombBlock: u32 = 8703619;
pub const fnPlmSetup_RespawningBombBlock2: u32 = 8703706;
pub const fnPlmSetup_RespawningPowerBombBlock: u32 = 8703790;
pub const fnPlmSetup_D08C_SuperMissileBlockRespawning: u32 = 8703847;
pub const fnPlmSetup_D08C_CrumbleBlock: u32 = 8703904;
pub const fnPlmSetup_D0DC_BreakableGrappleBlock: u32 = 8703925;
pub const fnPlmSetup_D0D8_SetVFlag: u32 = 8703949;
pub const fnPlmSetup_D0D8_ClearVflag: u32 = 8703953;
pub const fnPlmSetup_D0E8_GiveSamusDamage: u32 = 8703957;
pub const fnPlmSetup_D113_LowerNorfairChozoRoomPlug: u32 = 8704264;
pub const fnPlmSetup_D127: u32 = 8704279;
pub const fnPlmSetup_D138: u32 = 8704299;
pub const fnPlmInstr_FxBaseYPos_0x2D2: u32 = 8704341;
pub const fnPlmPreInstr_DeletePlmAndSpawnTriggerIfBlockDestroyed: u32 = 8704348;
pub const fnnullsub_69: u32 = 8704398;
pub const fnPlmSetup_D6DA_LowerNorfairChozoHandTrigger: u32 = 8704399;
pub const fnPlmPreInstr_IncrementRoomArgIfShotBySuperMissile: u32 = 8704486;
pub const fnPlmInstr_GotoIfRoomArgLess: u32 = 8704761;
pub const fnPlmInstr_SpawnFourMotherBrainGlass: u32 = 8704779;
pub const fnPlmPreInstr_WakePlmIfSamusHasBombs: u32 = 8704827;
pub const fnnullsub_351: i32 = 0x84d356;
pub const fnPlmInstr_SpawnTorizoStatueBreaking: u32 = 8704855;
pub const fnPlmInstr_QueueSong1MusicTrack: u32 = 8704967;
pub const fnPlmInstr_TransferWreckedShipChozoSpikesToSlopes: u32 = 8704983;
pub const fnPlmInstr_TransferWreckedShipSlopesToChozoSpikes: u32 = 8705012;
pub const fnPlmPreInstr_WakeOnKeyPress: u32 = 8705215;
pub const fnPlmInstr_EnableWaterPhysics: u32 = 8705317;
pub const fnPlmInstr_SpawnN00bTubeCrackEproj: u32 = 8705324;
pub const fnPlmInstr_DiagonalEarthquake: u32 = 8705334;
pub const fnPlmInstr_Spawn10shardsAnd6n00bs: u32 = 8705347;
pub const fnPlmInstr_DisableSamusControls: u32 = 8705510;
pub const fnPlmInstr_EnableSamusControls: u32 = 8705518;
pub const fnPlmSetup_MotherBrainGlass: u32 = 8705526;
pub const fnPlmSetup_DeletePlmIfAreaTorizoDead: u32 = 8705542;
pub const fnPlmSetup_MakeBllockChozoHandTrigger: u32 = 8705558;
pub const fnPlmSetup_D6F2_WreckedShipChozoHandTrigger: u32 = 8705568;
pub const fnPlmSetup_D700_MakePlmAirBlock_Unused: u32 = 8705663;
pub const fnPlmSetup_D704_AlteranateLowerNorfairChozoHand_Unused: u32 = 8705673;
pub const fnPlmSetup_D708_LowerNorfairChozoBlockUnused: u32 = 8705683;
pub const fnPlmSetup_D70C_NoobTube: u32 = 8705740;
pub const fnnullsub_71: u32 = 8705782;
pub const fnnullsub_72: u32 = 8705783;
pub const fnPlmPreInstr_WakePlmIfRoomArgumentDoorIsSet: u32 = 8705875;
pub const fnPlmInstr_ShootEyeDoorProjectileWithProjectileArg: u32 = 8705914;
pub const fnPlmInstr_SpawnEyeDoorSweatEproj: u32 = 8705936;
pub const fnPlmInstr_SpawnTwoEyeDoorSmoke: u32 = 8705951;
pub const fnPlmInstr_SpawnEyeDoorSmokeProjectile: u32 = 8705974;
pub const fnPlmInstr_MoveUpAndMakeBlueDoorFacingRight: u32 = 8705987;
pub const fnPlmInstr_MoveUpAndMakeBlueDoorFacingLeft: u32 = 8706010;
pub const fnPlmSetup_EyeDoorEye: u32 = 8706700;
pub const fnPlmSetup_EyeDoor: u32 = 8706745;
pub const fnnullsub_73: u32 = 8706773;
pub const fnnullsub_74: u32 = 8706774;
pub const fnnullsub_75: u32 = 8706775;
pub const fnnullsub_76: u32 = 8706776;
pub const fnnullsub_77: u32 = 8706777;
pub const fnnullsub_78: u32 = 8706778;
pub const fnnullsub_79: u32 = 8706779;
pub const fnnullsub_80: u32 = 8706780;
pub const fnnullsub_81: u32 = 8706781;
pub const fnPlmPreInstr_SetMetroidsClearState_Ev0x10: u32 = 8706782;
pub const fnPlmPreInstr_SetMetroidsClearState_Ev0x11: u32 = 8706798;
pub const fnPlmPreInstr_SetMetroidsClearState_Ev0x12: u32 = 8706814;
pub const fnPlmPreInstr_SetMetroidsClearState_Ev0x13: u32 = 8706830;
pub const fnPlmSetup_SetMetroidRequiredClearState: u32 = 8706846;
pub const fnPlmPreInstr_GotoLinkIfShotWithSuperMissile: u32 = 8706916;
pub const fnPlmInstr_DamageDraygonTurret: u32 = 8706958;
pub const fnPlmInstr_DamageDraygonTurretFacingDownRight: u32 = 8707000;
pub const fnPlmInstr_DamageDraygonTurretFacingUpRight: u32 = 8707063;
pub const fnPlmInstr_DamageDraygonTurret2: u32 = 8707126;
pub const fnPlmInstr_DamageDraygonTurretFacingDownLeft: u32 = 8707168;
pub const fnPlmInstr_DamageDraygonTurretFacingUpLeft: u32 = 8707231;
pub const fnPlmSetup_DraygonCannonFacingRight: u32 = 8707732;
pub const fnPlmSetup_DraygonCannonFacingDiagonalRight: u32 = 8707769;
pub const fnPlmSetup_DraygonCannonFacingLeft: u32 = 8707824;
pub const fnPlmSetup_DraygonCannonFacingDiagonalLeft: u32 = 8707861;
pub const fnPlmSetup_DraygonCannon: u32 = 8707916;
pub const fnPlmPreInstr_GotoLinkIfTriggered: i32 = 0x84df89;
pub const fnPlmPreInstr_nullsub_301: i32 = 0x84dfa8;
pub const fnPlmPreInstr_WakeIfTriggered: u32 = 8708070;
pub const fnPlmInstr_DrawItemFrame0: u32 = 8708175;
pub const fnPlmInstr_DrawItemFrame1: u32 = 8708199;
pub const fnPlmInstr_DrawItemFrame_Common: u32 = 8708223;
pub const fnPlmInstr_ClearChargeBeamCounter: u32 = 8708765;
pub const fnPlmInstr_E63B: u32 = 8709691;
pub const fnsub_84EE4D: u32 = 8711757;
pub const fnsub_84EE52: u32 = 8711762;
pub const fnsub_84EE57: u32 = 8711767;
pub const fnsub_84EE5C: u32 = 8711772;
pub const fnsub_84EE64: u32 = 8711780;
pub const fnsub_84EE77: u32 = 8711799;
pub const fnsub_84EE7C: u32 = 8711804;
pub const fnsub_84EE81: u32 = 8711809;
pub const fnsub_84EE86: u32 = 8711814;
pub const fnsub_84EE8E: u32 = 8711822;
pub const fnsub_84EEAB: u32 = 8711851;
pub const fnVariaSuitPickup: u32 = 9557220;
pub const fnGravitySuitPickup: u32 = 9557434;
pub const fnPlmPreInstr_Empty4: u32 = 8686246;
pub unsafe fn SetGoldenTorizoPalette(mut a: u16, game: &mut Game) {
    let mut v1: i16 = 0;
    v1 = ((*(&raw mut a as *mut u8).offset(1isize)) as i16) & (0x78i16);
    if (v1 as i32 & 0x40i32) != 0i32 {
        v1 = 56i16;
    }
    let mut v2: u16 = (4i32 * v1 as i32 | 0x1ei32) as u16;
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        let v4: i32 = v2 as i32 >> 1i32;
        let v5: i32 = i >> 1i32;
        set_ram_word!(
            game.palette_buffer[(v5 + 160i32)] =
                crate::rtl::rom_table_word!(game, 0x848132, v4 as isize)
        );
        set_ram_word!(
            game.palette_buffer[(v5 + 144i32)] =
                crate::rtl::rom_table_word!(game, 0x848032, v4 as isize)
        );
        v2 = (v2).wrapping_sub(2u16);
        i -= 2i32;
    }
}
pub unsafe fn LoadRoomPlmGfx(game: &mut Game) {
    set_ram_word!(game.plm_item_gfx_index = 0u16);
    let mut v0: i32 = 0i32;
    loop {
        if ram_word!(game, plm_item_gfx_ptrs[(v0 >> 1i32)]) != 0 {
            PlmInstr_LoadItemPlmGfx(
                RomPtr_84(ram_word!(game, plm_item_gfx_ptrs[(v0 >> 1i32)]), game),
                v0 as u16,
                game,
            );
        }
        v0 += 2i32;
        if (v0 == 8i32) {
            break;
        }
    }
}
pub unsafe fn ClearSoundsWhenGoingThroughDoor(game: &mut Game) {
    CallSomeSamusCode(0x1du16, game);
}
pub unsafe fn UNUSED_sub_848258(game: &mut Game) {
    if ((*(&raw mut game.g_ram as *mut u8).offset(0xa1f) as i32) << 8i32) == 768i32
        || ((*(&raw mut game.g_ram as *mut u8).offset(0xa1f) as i32) << 8i32) == 5120i32
    {
        QueueSfx1_Max15(0x32u16, game);
    }
}
pub unsafe fn PlaySpinJumpSoundIfSpinJumping(game: &mut Game) {
    CallSomeSamusCode(0x1cu16, game);
}
pub unsafe fn UNUSED_sub_848278(game: &mut Game) {
    if ((*(&raw mut game.g_ram as *mut u8).offset(0xa1f) as i32) << 8i32) == 768i32
        || ((*(&raw mut game.g_ram as *mut u8).offset(0xa1f) as i32) << 8i32) == 5120i32
    {
        QueueSfx1_Max15(0x30u16, game);
    }
}
pub unsafe fn CalculatePlmBlockCoords(k: u16, game: &mut Game) {
    let block_index_addr = 0x1c87 + usize::from(k >> 1) * 2; // plm_block_indices
    let block_index = u16::from_le_bytes([
        game.g_ram[block_index_addr],
        game.g_ram[block_index_addr + 1],
    ]) >> 1;
    let room_width = u16::from_le_bytes([game.g_ram[0x7a5], game.g_ram[0x7a6]]) as u8;
    let y = SnesDivide(block_index, room_width);
    game.g_ram[0x1c2b..0x1c2d].copy_from_slice(&y.to_le_bytes()); // plm_y_block
    let room_width = u16::from_le_bytes([game.g_ram[0x7a5], game.g_ram[0x7a6]]) as u8;
    let x = SnesModulus(block_index, room_width);
    game.g_ram[0x1c29..0x1c2b].copy_from_slice(&x.to_le_bytes()); // plm_x_block
}
pub fn WriteLevelDataBlockTypeAndBts(k: u16, a: u16, game: &mut Game) {
    // The level-data byte uses the exact offset, while BTS rounds down to a word.
    let block_high = 0x10003 + usize::from(k);
    game.g_ram[block_high] = (a >> 8) as u8 | (game.g_ram[block_high] & 0x0f);
    game.g_ram[0x16402 + usize::from(k >> 1)] = a as u8;
}
pub unsafe fn WriteRowOfLevelDataBlockAndBTS(
    k: u16,
    arg0: u16,
    arg1: u16,
    arg2: u16,
    game: &mut Game,
) {
    let v2: u16 = ram_word!(game, plm_block_indices[(k as i32 >> 1i32)]);
    let v8: u16 = (v2 as i32 >> 1i32) as u16;
    let mut i: i32 = 0i32;
    while i < arg2 as i32 {
        *((&raw mut game.g_ram as *mut u8).offset(0x10002) as *mut u16)
            .offset((v2 as i32 + i * 2i32 >> 1i32) as isize) = arg0;
        *(&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset((v8 as i32 + i) as isize) = arg1 as u8;
        i += 1;
    }
}
pub unsafe fn LoadXrayBlocks(game: &mut Game) {
    let mut i: i32 = 78i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        if ram_word!(game, plm_header_ptr[v1]) as i32 >= 0x84df89i32 & 0xffffi32 {
            let k: u16 = i as u16;
            let v2: u16 = *(&raw mut game.g_ram as *mut u8)
                .offset(0xd870)
                .offset(PrepareBitAccess(ram_word!(game, plm_room_arguments[v1]), game) as isize)
                as u16;
            i = k as i32;
            if (ram_word!(game, bitmask) as i32 & v2 as i32) == 0i32 {
                CalculatePlmBlockCoords(k, game);
                let v3: *const u8 = RomPtr_84(
                    crate::rtl::rom_table_word!(
                        game,
                        0x84839d,
                        (*((&raw mut game.g_ram as *mut u8).offset(0xdf0c) as *mut u16)
                            .offset((k as i32 >> 1i32) as isize) as i32
                            >> 1i32) as isize,
                    ),
                    game,
                );
                LoadBlockToXrayTilemap(
                    (*(v3.offset(2) as *mut u16)) & (0xfffu16),
                    ram_word!(game, plm_x_block),
                    ram_word!(game, plm_y_block),
                    game,
                );
                i = k as i32;
            }
        }
        i -= 2i32;
    }
    let RS: *mut RoomDefRoomstate =
        get_RoomDefRoomstate(ram_word!(game, roomdefroomstate_ptr), game);
    if (*RS).xray_special_casing_ptr != 0 {
        let mut p: *const XraySpecialCasing =
            RomPtr_8F((*RS).xray_special_casing_ptr, game) as *const XraySpecialCasing;
        while (*p).x_block != 0u8 || (*p).y_block != 0u8 {
            LoadBlockToXrayTilemap(
                (*p).level_data_block,
                (*p).x_block as u16,
                (*p).y_block as u16,
                game,
            );
            p = p.offset(1);
        }
    }
}
pub fn EnablePLMs(game: &mut Game) {
    // plm_flag: enable bit 15 while preserving the other flag bits.
    let flags = u16::from_le_bytes([game.g_ram[0x1c23], game.g_ram[0x1c24]]);
    game.g_ram[0x1c23..0x1c25].copy_from_slice(&(flags | 0x8000).to_le_bytes());
}
pub unsafe fn DisablePLMs(game: &mut Game) {
    update_ram_word!(game.plm_flag, |fresh22| (fresh22 as i32 & !(0x8000i32))
        as u16);
}
pub fn ClearPLMs(game: &mut Game) {
    for slot in (0..40).rev() {
        let header_addr = 0x1c37 + slot * 2; // plm_header_ptr
        game.g_ram[header_addr..header_addr + 2].copy_from_slice(&0u16.to_le_bytes());
    }
    game.g_ram[0x1c2d..0x1c2f].copy_from_slice(&0u16.to_le_bytes()); // plm_item_gfx_index
}
pub unsafe fn SpawnHardcodedPlm(p: SpawnHardcodedPlmArgs, game: &mut Game) {
    spawn_hardcoded_plm(p, core::ptr::null_mut(), game);
}
// Nested setup spawns retain the calling collision's displacement.
unsafe fn spawn_hardcoded_plm(
    mut p: SpawnHardcodedPlmArgs,
    displacement: *mut i32,
    game: &mut Game,
) {
    let pp: *mut SpawnHardcodedPlmArgs = &raw mut p;
    let mut v1: u16 = 78u16;
    while ram_word!(game, plm_header_ptr[(v1 as i32 >> 1i32)]) != 0 {
        v1 = (v1).wrapping_sub(2u16);
        if (v1 as i32 & 0x8000i32) != 0i32 {
            return;
        }
    }
    let prod: u16 = Mult8x8((*pp).block_y, ram_word!(game, room_width_in_blocks) as u8);
    let index = usize::from(v1 >> 1) * 2;
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    let block_index = (2 * (u32::from(prod) + u32::from((*pp).block_x))) as u16;
    write_word(game, 0x1c87 + index, block_index); // plm_block_indices
    let v3 = (*pp).header;
    write_word(game, 0x1c37 + index, v3); // plm_header_ptr
    write_word(game, 0x1dc7 + index, 0); // plm_room_arguments
    write_word(game, 0xdf0c + index, 0); // plm_variables
    write_word(game, 0x1cd7 + index, 0x8469); // plm_pre_instrs
    let PH = get_PlmHeader_Size4(v3, game);
    let instruction = (*PH).instr_list_ptr;
    write_word(game, 0x1d27 + index, instruction); // plm_instr_list_ptrs
    write_word(game, 0xde1c + index, 1); // plm_instruction_timer
    write_word(game, 0xde6c + index, addr_kDefaultPlmDrawInstruction as u16);
    write_word(game, 0x1d77 + index, 0); // plm_timers
    write_word(game, 0x1c27, v1); // plm_id
    CallPlmHeaderFunc(
        (((*PH).func_ptr) as u32) | (0x840000u32),
        v1,
        displacement,
        addr_kDefaultPlmDrawInstruction as u16,
        game,
    );
}
pub unsafe fn SpawnRoomPLM(k: u16, game: &mut Game) {
    let v1 = match (0..40).rev().find(|&slot| {
        let header_addr = 0x1c37 + slot * 2; // plm_header_ptr
        u16::from_le_bytes([game.g_ram[header_addr], game.g_ram[header_addr + 1]]) == 0
    }) {
        Some(slot) => (slot * 2) as u16,
        None => return,
    };
    let entry = if k == 0x12 {
        game.g_ram.as_ptr().add(0x12)
    } else {
        get_RoomPlmEntry(k, game).cast::<u8>()
    };
    let read_entry_word =
        |offset: usize| u16::from_le_bytes([*entry.add(offset), *entry.add(offset + 1)]);
    let y_block = u16::from(*entry.add(3));
    let room_width = u16::from_le_bytes([game.g_ram[0x7a5], game.g_ram[0x7a6]]) as u8;
    let row_start = y_block * u16::from(room_width);
    let x_block = u16::from(*entry.add(2));
    let index = usize::from(v1 >> 1) * 2;
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    let block_index = row_start.wrapping_add(x_block).wrapping_mul(2);
    write_word(game, 0x1c87 + index, block_index); // plm_block_indices
    let argument = read_entry_word(4);
    write_word(game, 0x1dc7 + index, argument); // plm_room_arguments
    let pp = read_entry_word(0);
    let header = read_entry_word(0);
    write_word(game, 0x1c37 + index, header); // plm_header_ptr
    write_word(game, 0xdf0c + index, 0); // plm_variables
    write_word(game, 0x1cd7 + index, 0x84e6); // plm_pre_instrs
    let header_bytes = get_PlmHeader_Size4(pp, game).cast::<u8>();
    let read_header_word = |offset: usize| {
        u16::from_le_bytes([*header_bytes.add(offset), *header_bytes.add(offset + 1)])
    };
    let instruction = read_header_word(2);
    write_word(game, 0x1d27 + index, instruction); // plm_instr_list_ptrs
    write_word(game, 0xde1c + index, 1); // plm_instruction_timer
    write_word(game, 0xde6c + index, addr_kDefaultPlmDrawInstruction as u16);
    write_word(game, 0x1d77 + index, 0); // plm_timers
    write_word(game, 0x1c27, v1); // plm_id
    CallPlmHeaderFunc(
        0x840000 | u32::from(read_header_word(0)),
        v1,
        core::ptr::null_mut(),
        addr_kDefaultPlmDrawInstruction as u16,
        game,
    );
}
pub fn PlmPreInstr_Empty2() {}
/// Prepare a PLM slot at the current collision block and invoke its setup.
///
/// # Safety
/// The selected setup callback requires the legacy game runtime to be initialized.
pub unsafe fn SpawnPLM(header: u16, game: &mut Game) -> u8 {
    SpawnPLMWithCollision(header, core::ptr::null_mut(), game)
}
/// Prepare a PLM while retaining the caller's active collision displacement.
///
/// # Safety
/// Setup requires initialized runtime state. A non-null `displacement` must
/// remain live and writable through setup and any nested PLM spawns.
pub unsafe fn SpawnPLMWithCollision(header: u16, displacement: *mut i32, game: &mut Game) -> u8 {
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    // Allocate from the last slot down, retaining the original full-table result.
    let Some(slot) = (0..40)
        .rev()
        .find(|&slot| read_word(game, 0x1c37 + slot * 2) == 0)
    else {
        return 0;
    };
    let index = slot * 2;
    write_word(game, 0x1c87 + index, game.cur_block_index().wrapping_mul(2));
    write_word(game, 0x1c37 + index, header);
    write_word(game, 0x1cd7 + index, 0x853d);
    let instruction = game.read_rom_word(0x840000 | u32::from(header), 2);
    write_word(game, 0x1d27 + index, instruction);
    write_word(game, 0xde1c + index, 1);
    write_word(game, 0xde6c + index, addr_kDefaultPlmDrawInstruction as u16);
    write_word(game, 0x1d77 + index, 0);
    write_word(game, 0x1dc7 + index, 0);
    write_word(game, 0xdf0c + index, 0);
    write_word(game, 0x1c27, index as u16);
    // Read setup after preparing the slot, at the original callback boundary.
    let setup = game.read_rom_word(0x840000 | u32::from(header), 0);
    CallPlmHeaderFunc(
        0x840000 | u32::from(setup),
        index as u16,
        displacement,
        0,
        game,
    )
}
pub unsafe fn PlmHandler_Async(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut u8 = (&raw mut game.g_ram as *mut u8).offset(0x613);
    's_158: {
        let mut base: *const u8 = null::<u8>();
        let mut v5: *const u8 = null::<u8>();
        let mut current_block_28: u64;
        match *_state_variable_ as i32 {
            0 => {
                if (ram_word!(game, plm_flag) as i32 & 0x8000i32) == 0i32 {
                    return kCoroutineNone as CoroutineRet;
                }
                set_ram_word!(game.plm_draw_tilemap_index = 0u16);
                set_ram_word!(game.plm_id = 78u16);
                current_block_28 = 17179679302217393232;
            }
            1 => {
                current_block_28 = 12780425637862113792;
            }
            _ => {
                current_block_28 = 2569451025026770673;
            }
        }
        'c_23315: loop {
            match current_block_28 {
                2569451025026770673 => {
                    break 's_158;
                }
                17179679302217393232 => {
                    if (ram_word!(game, plm_id) as i32 & 0x8000i32) == 0i32 {
                        if *((&raw mut game.g_ram as *mut u8).offset(0x1c37) as *mut u16)
                            .offset((ram_word!(game, plm_id) as i32 >> 1i32) as isize)
                            == 0
                        {
                            current_block_28 = 735147466149431745;
                        } else {
                            CallPlmPreInstr(
                                ((*((&raw mut game.g_ram as *mut u8).offset(0x1cd7) as *mut u16)
                                    .offset((ram_word!(game, plm_id) as i32 >> 1i32) as isize))
                                    as u32)
                                    | (0x840000u32),
                                ram_word!(game, plm_id),
                                game,
                            );
                            if update_ram_word!(
                                game.plm_instruction_timer
                                    [(ram_word!(game, plm_id) as i32 >> 1i32)],
                                wrapping_sub(1),
                                reread
                            ) != 0
                            {
                                current_block_28 = 735147466149431745;
                            } else {
                                base = RomBankBase(0x84u8, game);
                                v5 = null::<u8>();
                                v5 = base.offset(
                                    *((&raw mut game.g_ram as *mut u8).offset(0x1d27) as *mut u16)
                                        .offset((ram_word!(game, plm_id) as i32 >> 1i32) as isize)
                                        as i32 as isize,
                                );
                                current_block_28 = 11812396948646013369;
                            }
                        }
                    } else {
                        set_ram_word!(game.plm_id = 0u16);
                        current_block_28 = 2569451025026770673;
                        continue;
                    }
                }
                _ => {
                    let mut _coret_: u8 =
                        DisplayMessageBox_Async(ram_word!(game, queued_message_box_index), game);
                    if _coret_ != 0 {
                        *_state_variable_ = 1u8;
                        return _coret_;
                    }
                    set_ram_word!(game.queued_message_box_index = 0u16);
                    base = RomBankBase(0x84u8, game);
                    v5 = base.offset(
                        *((&raw mut game.g_ram as *mut u8).offset(0x1d27) as *mut u16)
                            .offset((ram_word!(game, plm_id) as i32 >> 1i32) as isize)
                            as i32 as isize,
                    );
                    current_block_28 = 11812396948646013369;
                }
            }
            loop {
                match current_block_28 {
                    735147466149431745 => {
                        update_ram_word!(game.plm_id, wrapping_sub(2u16));
                        current_block_28 = 17179679302217393232;
                        continue 'c_23315;
                    }
                    _ => {
                        if (*(v5 as *mut u16) as i32 & 0x8000i32) == 0i32 {
                            let mut v7: i32 = 0;
                            v7 = ram_word!(game, plm_id) as i32 >> 1i32;
                            set_ram_word!(game.plm_instruction_timer[v7] = *(v5 as *mut u16));
                            set_ram_word!(
                                game.plm_instruction_draw_ptr[v7] = *(v5.offset(2) as *mut u16)
                            );
                            set_ram_word!(
                                game.plm_instr_list_ptrs[v7] =
                                    v5.offset(4).offset_from(base) as c_long as u16
                            );
                            ProcessPlmDrawInstruction(ram_word!(game, plm_id), game);
                            CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
                            DrawPlm(ram_word!(game, plm_id), game);
                            current_block_28 = 735147466149431745;
                        } else {
                            v5 = CallPlmInstr(
                                ((*(v5 as *mut u16)) as u32) | (0x840000u32),
                                v5.offset(2),
                                ram_word!(game, plm_id),
                                game,
                            );
                            if v5.is_null() {
                                current_block_28 = 735147466149431745;
                                continue;
                            }
                            if (v5 as usize) < 0x10000i32 as usize {
                                v5 = base.offset(v5 as usize as isize);
                            }
                            if ram_word!(game, queued_message_box_index) != 0u16 {
                                break;
                            } else {
                                current_block_28 = 11812396948646013369;
                            }
                        }
                    }
                }
            }
            *((&raw mut game.g_ram as *mut u8).offset(0x1d27) as *mut u16)
                .offset((ram_word!(game, plm_id) as i32 >> 1i32) as isize) =
                v5.offset_from(base) as c_long as u16;
            current_block_28 = 12780425637862113792;
        }
    }
    *_state_variable_ = 0u8;
    0 as CoroutineRet
}
#[inline]
unsafe fn RomPtr_84orRAM(addr: u16, game: &mut Game) -> *mut u8 {
    if addr as i32 & 0x8000i32 != 0 {
        RomPtr((0x840000u32) | ((addr) as u32), game) as *mut u8
    } else {
        '_c2rust_label: {
            if addr < 0x2000u16 {
            } else {
                __assert_fail(
                    b"addr < 0x2000\0" as *const u8 as *const c_char,
                    b"src/sm_84.c\0" as *const u8 as *const c_char,
                    265u32,
                    b"uint8 *RomPtr_84orRAM(uint16_t)\0" as *const u8 as *const c_char,
                );
            }
        };
        RomPtr_RAM(addr, game)
    }
}
pub unsafe fn ProcessPlmDrawInstruction(k: u16, game: &mut Game) {
    let slot = usize::from(k >> 1) * 2;
    let draw_instruction =
        u16::from_le_bytes([game.g_ram[0xde6c + slot], game.g_ram[0xde6d + slot]]);
    let mut words = RomPtr_84orRAM(draw_instruction, game) as *const u16;
    let mut destination =
        u16::from_le_bytes([game.g_ram[0x1c87 + slot], game.g_ram[0x1c88 + slot]]);
    let origin = destination;
    loop {
        let command = words.read_unaligned();
        words = words.add(1);
        let stride = if command & 0x8000 != 0 {
            usize::from(u16::from_le_bytes([game.g_ram[0x7a5], game.g_ram[0x7a6]])) * 2
        } else {
            2
        };
        for block in 0..usize::from(command as u8) {
            let word = words.read_unaligned();
            words = words.add(1);
            // The original word indexing rounds odd byte destinations down.
            let offset = 0x10002 + ((usize::from(destination) + block * stride) >> 1) * 2;
            game.g_ram[offset..offset + 2].copy_from_slice(&word.to_le_bytes());
        }
        let displacement = words.read_unaligned();
        if displacement == 0 {
            break;
        }
        let x = i32::from(displacement as u8 as i8);
        let y = i32::from((displacement >> 8) as u8 as i8);
        let width = i32::from(u16::from_le_bytes([game.g_ram[0x7a5], game.g_ram[0x7a6]]));
        destination = (i32::from(origin) + 2 * (x + y * width)) as u16;
        words = words.add(1);
    }
}
pub unsafe fn PlmInstr_Sleep(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    let instruction = (plmp.offset_from(RomBankBase(0x84, game)) as u16).wrapping_sub(2);
    let offset = 0x1d27 + usize::from(k >> 1) * 2; // plm_instruction_list_ptrs
    game.g_ram[offset..offset + 2].copy_from_slice(&instruction.to_le_bytes());
    null()
}
pub fn PlmInstr_Delete(_plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    let offset = 0x1c37 + usize::from(k >> 1) * 2; // plm_header_ptr
    game.g_ram[offset..offset + 2].copy_from_slice(&0u16.to_le_bytes());
    null()
}
pub unsafe fn PlmInstr_PreInstr(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    let pre_instruction = (plmp as *const u16).read_unaligned();
    let offset = 0x1cd7 + usize::from(k >> 1) * 2; // plm_pre_instrs
    game.g_ram[offset..offset + 2].copy_from_slice(&pre_instruction.to_le_bytes());
    plmp.add(2)
}
pub fn PlmInstr_ClearPreInstr(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    let offset = 0x1cd7 + usize::from(k >> 1) * 2; // plm_pre_instrs
    game.g_ram[offset..offset + 2].copy_from_slice(&0x86d0u16.to_le_bytes());
    plmp
}
pub unsafe fn CallPlmInstrFunc(ea: u32, game: &mut Game) {
    match ea {
        9557220 => VariaSuitPickup(game),
        9557434 => GravitySuitPickup(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn PlmInstr_CallFunction(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    CallPlmInstrFunc(Load24(plmp as *mut LongPtr), game);
    plmp.offset(3)
}
pub unsafe fn PlmInstr_Goto(plmp: *const u8, _k: u16) -> *const u8 {
    let target = (plmp as *const u16).read_unaligned();
    usize::from(target) as *const u8
}
pub unsafe fn PlmInstr_DecrementAndBranchNonzero(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    let ref mut fresh65 = *((&raw mut game.g_ram as *mut u8).offset(0x1d77) as *mut u16)
        .offset((k as i32 >> 1i32) as isize);
    let fresh66 = *fresh65;
    *fresh65 = (*fresh65).wrapping_sub(1);
    if fresh66 == 1u16 {
        plmp.offset(2)
    } else {
        PlmInstr_Goto(plmp, k)
    }
}
pub unsafe fn PlmInstr_SetTimer(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    game.g_ram[0x1d77 + usize::from(k)] = *plmp; // Only one byte of plm_timers.
    plmp.add(1)
}
pub unsafe fn PlmInstr_LoadItemPlmGfx(mut plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    let gfx_index = u16::from_le_bytes([game.g_ram[0x1c2d], game.g_ram[0x1c2e]]);
    let slot = 0xdf0c + usize::from(k >> 1) * 2; // plm_variables
    let current_index = u16::from_le_bytes([game.g_ram[0x1c2d], game.g_ram[0x1c2e]]);
    game.g_ram[slot..slot + 2].copy_from_slice(&current_index.to_le_bytes());
    let next_index = gfx_index.wrapping_add(2) & 6;
    game.g_ram[0x1c2d..0x1c2f].copy_from_slice(&next_index.to_le_bytes());
    let table_offset = usize::from(gfx_index >> 1) * 2;
    let vram_destination = game.read_rom_word(0x8487cd, table_offset);
    let tile_offset = usize::from(game.read_rom_word(0x8487d5, table_offset));
    let mut tile_number = game.read_rom_word(0x8487dd, table_offset);
    let instruction = plmp.offset_from(RomBankBase(0x84, game)) as u16;
    let pointer_offset = 0x1c2f + usize::from(gfx_index >> 1) * 2; // plm_item_gfx_ptrs
    game.g_ram[pointer_offset..pointer_offset + 2].copy_from_slice(&instruction.to_le_bytes());
    let queue_tail = u16::from_le_bytes([game.g_ram[0x330], game.g_ram[0x331]]);
    let entry_tail = u16::from_le_bytes([game.g_ram[0x330], game.g_ram[0x331]]);
    let entry = 0xd0 + usize::from(entry_tail); // vram_write_queue
    game.g_ram[entry..entry + 2].copy_from_slice(&256u16.to_le_bytes());
    let source = (plmp as *const u16).read_unaligned();
    game.g_ram[entry + 2..entry + 4].copy_from_slice(&source.to_le_bytes());
    game.g_ram[entry + 4] = 0x89;
    game.g_ram[entry + 5..entry + 7].copy_from_slice(&vram_destination.to_le_bytes());
    game.g_ram[0x330..0x332].copy_from_slice(&queue_tail.wrapping_add(7).to_le_bytes());
    plmp = plmp.add(2);
    for tile in 0..8 {
        let attributes = u16::from(*plmp) << 10;
        let word = tile_number.wrapping_add(attributes);
        let destination = 0xa000 + tile_offset + tile * 2; // tile_table.tables[0].top_left
        game.g_ram[destination..destination + 2].copy_from_slice(&word.to_le_bytes());
        tile_number = tile_number.wrapping_add(1);
        plmp = plmp.add(1);
    }
    plmp
}
pub unsafe fn PlmInstr_CopyFromRamToVram(plmp: *const u8, _k: u16, game: &mut Game) -> *const u8 {
    let queue_tail = u16::from_le_bytes([game.g_ram[0x330], game.g_ram[0x331]]);
    let entry_tail = u16::from_le_bytes([game.g_ram[0x330], game.g_ram[0x331]]);
    let entry = 0xd0 + usize::from(entry_tail); // vram_write_queue
    let size = (plmp as *const u16).read_unaligned();
    game.g_ram[entry..entry + 2].copy_from_slice(&size.to_le_bytes());
    let source = (plmp.add(2) as *const u16).read_unaligned();
    game.g_ram[entry + 2..entry + 4].copy_from_slice(&source.to_le_bytes());
    game.g_ram[entry + 4] = *plmp.add(4);
    let destination = (plmp.add(5) as *const u16).read_unaligned();
    game.g_ram[entry + 5..entry + 7].copy_from_slice(&destination.to_le_bytes());
    game.g_ram[0x330..0x332].copy_from_slice(&queue_tail.wrapping_add(7).to_le_bytes());
    plmp.add(7)
}
pub unsafe fn PlmInstr_GotoIfBossBitSet(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    if CheckBossBitForCurArea(*plmp.offset(0) as u16, game) != 0 {
        PlmInstr_Goto(plmp.offset(1), k)
    } else {
        plmp.offset(3)
    }
}
pub unsafe fn PlmInstr_GotoIfEventSet(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    if CheckEventHappened(*(plmp as *mut u16), game) != 0 {
        PlmInstr_Goto(plmp.offset(2), k)
    } else {
        plmp.offset(4)
    }
}
pub unsafe fn PlmInstr_SetEvent(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    SetEventHappened(*(plmp as *mut u16), game);
    plmp.offset(2)
}
pub unsafe fn PlmInstr_GotoIfChozoSet(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    let v3: u16 = ram_word!(game, plm_room_arguments[(k as i32 >> 1i32)]);
    let idx: i32 = PrepareBitAccess(v3, game) as i32;
    if v3 as i32 & 0x8000i32 == 0
        && (ram_word!(game, bitmask) as i32
            & *(&raw mut game.g_ram as *mut u8)
                .offset(0xd830)
                .offset(idx as isize) as i32)
            != 0i32
    {
        PlmInstr_Goto(plmp, k)
    } else {
        plmp.offset(2)
    }
}
pub unsafe fn PlmInstr_SetRoomChozoBit(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    let v3: u16 = ram_word!(game, plm_room_arguments[(k as i32 >> 1i32)]);
    if v3 as i32 & 0x8000i32 == 0 {
        let idx: i32 = PrepareBitAccess(v3, game) as i32;
        let ref mut fresh64 = *(&raw mut game.g_ram as *mut u8)
            .offset(0xd830)
            .offset(idx as isize);
        *fresh64 = (*fresh64) | ((ram_word!(game, bitmask)) as u8);
    }
    plmp
}
pub unsafe fn PlmInstr_GotoIfItemBitSet(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    let index = usize::from(k >> 1) * 2;
    let v2 = u16::from_le_bytes([game.g_ram[0x1dc7 + index], game.g_ram[0x1dc8 + index]]);
    let byte_index = usize::from(PrepareBitAccess(v2, game));
    if v2 & 0x8000 == 0 {
        let bitmask = u16::from_le_bytes([game.g_ram[0x5e7], game.g_ram[0x5e8]]);
        if bitmask & u16::from(game.g_ram[0xd870 + byte_index]) != 0 {
            return PlmInstr_Goto(plmp, k);
        }
    }
    plmp.offset(2)
}
pub unsafe fn PlmInstr_SetItemBit(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    let index = usize::from(k >> 1) * 2;
    let v2 = u16::from_le_bytes([game.g_ram[0x1dc7 + index], game.g_ram[0x1dc8 + index]]);
    if v2 & 0x8000 == 0 {
        let byte_index = usize::from(PrepareBitAccess(v2, game));
        let bitmask = u16::from_le_bytes([game.g_ram[0x5e7], game.g_ram[0x5e8]]);
        game.g_ram[0xd870 + byte_index] |= bitmask as u8;
    }
    plmp
}
pub unsafe fn PlmInstr_PickupBeamAndShowMessage(
    plmp: *const u8,
    _k: u16,
    game: &mut Game,
) -> *const u8 {
    let beam = u16::from_le_bytes([*plmp, *plmp.add(1)]);
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    game.set_collected_beams(read_word(game, 0x9a8) | beam);
    game.set_equipped_beams(read_word(game, 0x9a6) | beam);
    // Spazer and Plasma are mutually exclusive; retain both ordered masks.
    game.set_equipped_beams(read_word(game, 0x9a6) & !((beam << 1) & 8));
    game.set_equipped_beams(read_word(game, 0x9a6) & !((beam >> 1) & 4));
    UpdateBeamTilesAndPalette(game);
    PlayRoomMusicTrackAfterAFrames(0x168, game);
    let message = u16::from(*plmp.add(2));
    DisplayMessageBox(message, game);
    plmp.add(3)
}
pub unsafe fn PlmInstr_PickupEquipmentAndShowMessage(
    plmp: *const u8,
    _k: u16,
    game: &mut Game,
) -> *const u8 {
    let equipment = (plmp as *const u16).read_unaligned();
    let equipped = u16::from_le_bytes([game.g_ram[0x9a2], game.g_ram[0x9a3]]);
    game.set_equipped_items(equipped | equipment);
    game.set_collected_items(game.collected_items() | equipment);
    PlayRoomMusicTrackAfterAFrames(0x168, game);
    let message = u16::from(*plmp.add(2));
    DisplayMessageBox(message, game);
    plmp.add(3)
}
pub unsafe fn PlmInstr_PickupEquipmentAddGrappleShowMessage(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    let t: u16 = *(plmp as *mut u16);
    update_ram_word!(game.equipped_items, |fresh55| (fresh55) | (t));
    update_ram_word!(game.collected_items, |fresh56| (fresh56) | (t));
    AddGrappleToHudTilemap(game);
    PlayRoomMusicTrackAfterAFrames(0x168u16, game);
    DisplayMessageBox(5u16, game);
    plmp.offset(2)
}
pub unsafe fn PlmInstr_PickupEquipmentAddXrayShowMessage(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    let t: u16 = *(plmp as *mut u16);
    update_ram_word!(game.equipped_items, |fresh53| (fresh53) | (t));
    update_ram_word!(game.collected_items, |fresh54| (fresh54) | (t));
    AddXrayToHudTilemap(game);
    PlayRoomMusicTrackAfterAFrames(0x168u16, game);
    DisplayMessageBox(6u16, game);
    plmp.offset(2)
}
pub unsafe fn PlmInstr_CollectHealthEnergyTank(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    let amount = (plmp as *const u16).read_unaligned();
    game.set_samus_max_health(game.samus_max_health().wrapping_add(amount));
    game.set_samus_health(game.samus_max_health());
    PlayRoomMusicTrackAfterAFrames(0x168, game);
    DisplayMessageBox(1, game);
    plmp.add(2)
}
pub unsafe fn PlmInstr_CollectHealthReserveTank(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    update_ram_word!(
        game.samus_max_reserve_health,
        wrapping_add(*(plmp as *mut u16))
    );
    if ram_word!(game, reserve_health_mode) == 0 {
        update_ram_word!(game.reserve_health_mode, wrapping_add(1));
    }
    PlayRoomMusicTrackAfterAFrames(0x168u16, game);
    DisplayMessageBox(0x19u16, game);
    plmp.offset(2)
}
pub unsafe fn PlmInstr_CollectAmmoMissileTank(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    let t = *(plmp as *const u16);
    game.set_samus_max_missiles(game.samus_max_missiles().wrapping_add(t));
    game.set_samus_missiles(game.samus_missiles().wrapping_add(t));
    AddMissilesToHudTilemap(game);
    PlayRoomMusicTrackAfterAFrames(0x168u16, game);
    DisplayMessageBox(2u16, game);
    plmp.offset(2)
}
pub unsafe fn PlmInstr_CollectAmmoSuperMissileTank(
    plmp: *const u8,
    _k: u16,
    game: &mut Game,
) -> *const u8 {
    let amount = u16::from_le_bytes([*plmp, *plmp.add(1)]);
    game.set_samus_max_super_missiles(game.samus_max_super_missiles().wrapping_add(amount));
    game.set_samus_super_missiles(game.samus_super_missiles().wrapping_add(amount));
    AddSuperMissilesToHudTilemap(game);
    PlayRoomMusicTrackAfterAFrames(0x168, game);
    DisplayMessageBox(3, game);
    plmp.add(2)
}
pub unsafe fn PlmInstr_CollectAmmoPowerBombTank(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    let t: u16 = *(plmp as *mut u16);
    update_ram_word!(game.samus_max_power_bombs, wrapping_add(t));
    update_ram_word!(game.samus_power_bombs, wrapping_add(t));
    AddPowerBombsToHudTilemap(game);
    PlayRoomMusicTrackAfterAFrames(0x168u16, game);
    DisplayMessageBox(4u16, game);
    plmp.offset(2)
}
pub unsafe fn PlmInstr_SetLinkReg(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    let link = (plmp as *const u16).read_unaligned();
    let offset = 0xdebc + usize::from(k >> 1) * 2; // plm_instruction_list_link_reg
    game.g_ram[offset..offset + 2].copy_from_slice(&link.to_le_bytes());
    plmp.add(2)
}
pub unsafe fn PlmInstr_Call(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    let return_address = (plmp.offset_from(RomBankBase(0x84, game)) as u16).wrapping_add(2);
    let offset = 0xdebc + usize::from(k >> 1) * 2; // plm_instruction_list_link_reg
    game.g_ram[offset..offset + 2].copy_from_slice(&return_address.to_le_bytes());
    let target = (plmp as *const u16).read_unaligned();
    usize::from(target) as *const u8
}
pub fn PlmInstr_Return(_plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    let offset = 0xdebc + usize::from(k >> 1) * 2; // plm_instruction_list_link_reg
    let link = u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]]);
    usize::from(link) as *const u8
}
pub unsafe fn PlmInstr_GotoIfDoorBitSet(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    let index = usize::from(k >> 1) * 2;
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    let argument = read_word(game, 0x1dc7 + index); // plm_room_arguments
    if argument & 0x8000 != 0 {
        return plmp.add(2);
    }
    let bit_index = usize::from(PrepareBitAccess(argument, game));
    let mask = read_word(game, 0x5e7); // bitmask
    if mask & u16::from(game.g_ram[0xd8b0 + bit_index]) != 0 {
        return usize::from((plmp as *const u16).read_unaligned()) as *const u8;
    }
    plmp.add(2)
}
pub unsafe fn PlmInstr_IncrementDoorHitCounterAndJGE(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    let index = usize::from(k >> 1) * 2;
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    let hits = read_word(game, 0xdf0c + index).wrapping_add(1); // plm_variables
    write_word(game, 0xdf0c + index, hits);
    if (hits as u8) < *plmp {
        return plmp.add(3);
    }
    let argument = read_word(game, 0x1dc7 + index); // plm_room_arguments
    if argument & 0x8000 == 0 {
        let bit_index = usize::from(PrepareBitAccess(argument, game));
        let mask = read_word(game, 0x5e7); // bitmask
        let door = 0xd8b0 + bit_index; // opened_door_bit_array
        game.g_ram[door] |= mask as u8;
        // Preserve the original word read, including the following bit-array byte.
        let marked_argument = read_word(game, door) | 0x8000;
        write_word(game, 0x1dc7 + index, marked_argument);
    }
    write_word(game, 0x1cd7 + index, addr_PlmPreInstr_Empty4 as u16);
    PlmInstr_Goto(plmp.add(1), k)
}
pub unsafe fn PlmInstr_IncrementArgumentAndJGE(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    let v2: u8 = (*((&raw mut game.g_ram as *mut u8).offset(0x1dc7) as *mut u16 as *mut u8)
        .offset(k as i32 as isize))
    .wrapping_add(1u8);
    if v2 >= *plmp.offset(0) {
        let v4: i32 = k as i32 >> 1i32;
        set_ram_word!(game.plm_room_arguments[v4] = -1i32 as u16);
        set_ram_word!(game.plm_pre_instrs[v4] = addr_locret_848AE0 as u16);
        PlmInstr_Goto(plmp.offset(1), k)
    } else {
        set_ram_word!(game.plm_room_arguments[(k as i32 >> 1i32)] = v2 as u16);
        plmp.offset(3)
    }
}
pub unsafe fn PlmInstr_SetBTS(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    let bts = *plmp;
    let index_addr = 0x1c87 + usize::from(k >> 1) * 2; // plm_block_indices
    let block_index = u16::from_le_bytes([game.g_ram[index_addr], game.g_ram[index_addr + 1]]) >> 1;
    game.g_ram[0x16402 + usize::from(block_index)] = bts;
    plmp.add(1)
}
pub unsafe fn PlmInstr_DrawPlmBlock(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    let index = usize::from(k >> 1) * 2;
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    let block = read_word(game, 0x1e17 + index); // plm_variable
    let block_index = read_word(game, 0x1c87 + index);
    write_word(game, 0x10002 + usize::from(block_index >> 1) * 2, block);
    write_word(game, 0x1e69, block);
    write_word(game, 0x1e67, 1);
    write_word(game, 0x1e6b, 0);
    write_word(game, 0xde1c + index, 1); // plm_instruction_timer
    write_word(game, 0xde6c + index, 0x1e67); // plm_instruction_draw_ptr
    let instruction = plmp.offset_from(RomBankBase(0x84, game)) as u16;
    write_word(game, 0x1d27 + index, instruction); // plm_instr_list_ptrs
    ProcessPlmDrawInstruction(k, game);
    let v4: u16 = ram_word!(game, plm_id);
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    DrawPlm(v4, game);
    null::<u8>()
}
pub unsafe fn PlmInstr_ProcessAirScrollUpdate(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    let index = usize::from(k >> 1) * 2;
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    write_word(game, 0x1e17 + index, 0); // plm_variable
    let argument = read_word(game, 0x1dc7 + index); // plm_room_arguments
    let mut entries = RomPtr_8F(argument, game);
    while *entries & 0x80 == 0 {
        let value = *entries.add(1);
        game.g_ram[0xcd20 + usize::from(*entries)] = value; // scrolls
        entries = entries.add(2);
    }
    let block_index = read_word(game, 0x1c87 + index); // plm_block_indices
    update_level_block_bits(usize::from(block_index >> 1), 0x0fff, 0x3000, game);
    plmp
}
pub unsafe fn PlmInstr_ProcessSolidScrollUpdate(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    set_ram_word!(game.plm_variable[(k as i32 >> 1i32)] = 0u16);
    let mut v5: *const u8 = RomPtr_8F(
        ram_word!(game, plm_room_arguments[(k as i32 >> 1i32)]),
        game,
    );
    while (*v5.offset(0) as i32 & 0x80i32 == 0) {
        *(&raw mut game.g_ram as *mut u8)
            .offset(0xcd20)
            .offset(*v5.offset(0) as isize) = *v5.offset(1);
        v5 = v5.offset(2);
    }
    let v7: i32 = ram_word!(game, plm_block_indices[(k as i32 >> 1i32)]) as i32 >> 1i32;
    set_ram_word!(
        game.level_data[v7] =
            (ram_word!(game, level_data[v7]) as i32 & 0xfffi32 | 0xb000i32) as u16
    );
    plmp
}
pub unsafe fn PlmInstr_QueueMusic(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    QueueMusic_Delayed8(*plmp.offset(0) as u16, game);
    plmp.offset(1)
}
pub unsafe fn PlmInstr_ClearMusicQueueAndQueueTrack(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    let mut i: i32 = 14i32;
    while i >= 0i32 {
        let v3: i32 = i >> 1i32;
        set_ram_word!(game.music_queue_track[v3] = 0u16);
        set_ram_word!(game.music_queue_delay[v3] = 0u16);
        i -= 2i32;
    }
    set_ram_word!(game.music_queue_read_pos = ram_word!(game, music_queue_write_pos));
    set_ram_word!(game.music_timer = 0u16);
    set_ram_word!(game.music_entry = 0u16);
    QueueMusic_Delayed8(*plmp.offset(0) as u16, game);
    plmp.offset(1)
}
pub unsafe fn PlmInstr_QueueSfx1_Max6(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    QueueSfx1_Max6(*plmp.offset(0) as u16, game);
    plmp.offset(1)
}
pub unsafe fn PlmInstr_QueueSfx2_Max6(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    QueueSfx2_Max6(*plmp.offset(0) as u16, game);
    plmp.offset(1)
}
pub unsafe fn PlmInstr_QueueSfx3_Max6(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    QueueSfx3_Max6(*plmp.offset(0) as u16, game);
    plmp.offset(1)
}
pub unsafe fn PlmInstr_QueueSfx1_Max15(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    QueueSfx1_Max15(*plmp.offset(0) as u16, game);
    plmp.offset(1)
}
pub unsafe fn PlmInstr_QueueSfx2_Max15(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    QueueSfx2_Max15(*plmp.offset(0) as u16, game);
    plmp.offset(1)
}
pub unsafe fn PlmInstr_QueueSfx3_Max15(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    QueueSfx3_Max15(*plmp.offset(0) as u16, game);
    plmp.offset(1)
}
pub unsafe fn PlmInstr_QueueSfx1_Max3(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    QueueSfx1_Max3(*plmp.offset(0) as u16, game);
    plmp.offset(1)
}
pub unsafe fn PlmInstr_QueueSfx2_Max3(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    QueueSfx2_Max3(*plmp.offset(0) as u16, game);
    plmp.offset(1)
}
pub unsafe fn PlmInstr_QueueSfx_Max3(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    QueueSfx3_Max3(*plmp.offset(0) as u16, game);
    plmp.offset(1)
}
pub unsafe fn PlmInstr_QueueSfx1_Max9(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    QueueSfx1_Max9(*plmp.offset(0) as u16, game);
    plmp.offset(1)
}
pub unsafe fn PlmInstr_QueueSfx2_Max9(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    QueueSfx2_Max9(*plmp.offset(0) as u16, game);
    plmp.offset(1)
}
pub unsafe fn PlmInstr_QueueSfx3_Max9(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    QueueSfx3_Max9(*plmp.offset(0) as u16, game);
    plmp.offset(1)
}
pub unsafe fn PlmInstr_QueueSfx1_Max1(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    QueueSfx1_Max1(*plmp.offset(0) as u16, game);
    plmp.offset(1)
}
pub unsafe fn PlmInstr_QueueSfx2_Max1(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    QueueSfx2_Max1(*plmp.offset(0) as u16, game);
    plmp.offset(1)
}
pub unsafe fn PlmInstr_QueueSfx3_Max1(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    QueueSfx3_Max1(*plmp.offset(0) as u16, game);
    plmp.offset(1)
}
pub unsafe fn PlmInstr_ActivateMapStation(plmp: *const u8, _k: u16, game: &mut Game) -> *const u8 {
    // map_station_byte_array; word OR with 00FF preserves the neighboring byte.
    let station = 0xd908 + usize::from(game.area_index());
    game.g_ram[station] = 0xff;
    DisplayMessageBox(0x14, game);
    game.set_has_area_map(1);
    plmp
}
pub unsafe fn PlmInstr_ActivateEnergyStation(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    if ram_word!(game, samus_max_health) != ram_word!(game, samus_health) {
        DisplayMessageBox(0x15u16, game);
        set_ram_word!(game.samus_health = ram_word!(game, samus_max_health));
    }
    CallSomeSamusCode(1u16, game);
    plmp
}
pub unsafe fn PlmInstr_ActivateMissileStation(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    if ram_word!(game, samus_max_missiles) != ram_word!(game, samus_missiles) {
        DisplayMessageBox(0x16u16, game);
        set_ram_word!(game.samus_missiles = ram_word!(game, samus_max_missiles));
    }
    CallSomeSamusCode(1u16, game);
    plmp
}
pub unsafe fn PlmInstr_ActivateSaveStationAndGotoIfNo(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    let r: i32 = DisplayMessageBox_Poll(23u16, game);
    if r < 0i32 {
        return plmp.offset(-(2i32 as isize));
    }
    if r == 2i32 {
        return *(plmp as *mut u16) as usize as *const u8;
    }
    SpawnEprojWithRoomGfx(addr_kEproj_SaveStationElectricity as u16, 0u16, game);
    set_ram_word!(
        game.load_station_index = (*((&raw mut game.g_ram as *mut u8).offset(0x1dc7) as *mut u16)
            .offset((ram_word!(game, plm_id) as i32 >> 1i32) as isize,))
            & (7u16)
    );
    PrepareBitAccess(ram_word!(game, load_station_index), game);
    let ref mut fresh40 = *(&raw mut game.g_ram as *mut u8)
        .offset(0xd8f8)
        .offset((2i32 * ram_word!(game, area_index) as i32) as isize);
    *fresh40 = (*fresh40) | ((ram_word!(game, bitmask)) as u8);
    SaveToSram(ram_word!(game, selected_save_slot), game);
    plmp.offset(2)
}
pub unsafe fn PlmInstr_GotoIfSamusNear(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    CalculatePlmBlockCoords(k, game);
    let v2: u8 = abs16(
        ((ram_word!(game, samus_x_pos) as i32 >> 4i32) - ram_word!(game, plm_x_block) as i32)
            as u16,
    ) as u8;
    let v4: u8 = abs16(
        ((ram_word!(game, samus_y_pos) as i32 >> 4i32) - ram_word!(game, plm_y_block) as i32)
            as u16,
    ) as u8;
    if v2 <= *plmp.offset(0) && v4 <= *plmp.offset(1) {
        *(plmp.offset(2) as *mut u16) as usize as *const u8
    } else {
        plmp.offset(4)
    }
}
unsafe fn PartiallyQueueVramForSingleScreenPlm(
    a: u16,
    k: u16,
    r20: u16,
    r28: u16,
    game: &mut Game,
) {
    let ent: *mut VramWriteEntry = gVramWriteEntry(k as i32, game);
    (*ent).vram_dst = (r28) | (a);
    let size: u16 = (4u16).wrapping_mul(r20);
    let base: VoidP = (ram_word!(game, plm_draw_tilemap_index)).wrapping_add(0xc6c8u16);
    (*ent.offset(0)).size = size;
    (*ent.offset(1)).size = size;
    (*ent.offset(0)).src = LongPtr {
        addr: base,
        bank: 126u8,
    };
    (*ent.offset(1)).src = LongPtr {
        addr: ((base).wrapping_add(size)),
        bank: 126u8,
    };
}
unsafe fn CalculatePlmDrawTilemapVramDst(
    k: u16,
    r9: u16,
    r12: u16,
    r20: u16,
    r24: u16,
    r26: u16,
    r28: u16,
    game: &mut Game,
) {
    let mut a: u16 = 0;
    let prod: u16 = ((r26 as i32 & 0xfi32) * 0x40i32) as u16;
    let v1: u16 = (r24) & (0x1fu16);
    if v1 >= 0x10u16 {
        a = (prod as i32 + r12 as i32 + 2i32 * v1 as i32) as u16;
        if (ram_word!(game, bg1_x_offset) as i32 & 0x100i32) != 0i32 {
            a = (a).wrapping_sub(1024u16);
        }
    } else {
        a = (prod as i32 + r9 as i32 + 2i32 * v1 as i32) as u16;
        if (ram_word!(game, bg1_x_offset) as i32 & 0x100i32) != 0i32 {
            a = (a).wrapping_add(1024u16);
        }
    }
    PartiallyQueueVramForSingleScreenPlm(a, k, r20, r28, game);
}
pub unsafe fn DrawPlm(mut k: u16, game: &mut Game) {
    let mut v24: u16 = 0;
    let mut v26: u16 = 0;
    let mut v29: u16 = 0;
    let mut v2: i16 = 0;
    let mut v5: i16 = 0;
    let mut v8: i16 = 0;
    let mut v10: *mut VramWriteEntry = null_mut::<VramWriteEntry>();
    let mut v12: VoidP = 0;
    let mut v13: VoidP = 0;
    let mut v15: *mut VramWriteEntry = null_mut::<VramWriteEntry>();
    let mut v20: i16 = 0;
    let mut v23: bool = false;
    let mut v25: *mut VramWriteEntry = null_mut::<VramWriteEntry>();
    let mut v31: i16 = 0;
    let mut v35: u16 = 0;
    let mut r26: u16 = 0;
    let mut r22: u16 = 0;
    let mut a: u16 = 0;
    let mut r24: u16 = 0;
    let mut r20: u16 = 0;
    let mut r18: u16 = 0;
    let r9: u16 = addr_unk_605000 as u16;
    let r12: u16 = addr_unk_6053E0 as u16;
    let mut v1: u16 = ram_word!(game, plm_instruction_draw_ptr[(k as i32 >> 1i32)]);
    let mut x: u16 = ram_word!(game, plm_x_block);
    let mut y: u16 = ram_word!(game, plm_y_block);
    let mut r3: u16 = 0;
    let mut r0: *mut u16 = null_mut::<u16>();
    let mut r6: *mut u16 = null_mut::<u16>();
    '_LABEL_2: loop {
        r26 = (ram_word!(game, layer1_y_pos) as i32 >> 4i32) as u16;
        if (ram_word!(game, layer1_y_pos) as i32 >> 4i32) + 15i32 - y as i32 & 0x8000i32 != 0 {
            return;
        }
        v2 = *(RomPtr_84orRAM(v1, game) as *mut u16) as i16;
        if v2 < 0i16 {
            r20 = ((v2) as u16) & (0x7fffu16);
            if !(ram_word!(game, layer1_x_pos) as i32 >> 4i32 == x as i32
                || (((ram_word!(game, layer1_x_pos) as i32 >> 4i32) - x as i32) as i16) < 0i16)
            {
                break;
            }
            v20 = ((ram_word!(game, layer1_x_pos) as i32 >> 4i32) + 17i32) as i16;
            if !(v20 as i32 != x as i32 && (v20).wrapping_sub((x) as i16) >= 0i16) {
                break;
            }
            r24 = x;
            r18 = 0u16;
            r22 = (r26).wrapping_add(16u16);
            if ((r26) as i16).wrapping_sub((y) as i16) < 0i16 {
                r26 = y;
            } else {
                r18 = (r26).wrapping_sub(y);
                if y as i32 + r20 as i32 - r26 as i32 & 0x8000i32 != 0 {
                    return;
                }
                let v21: bool = r20 == r18;
                let v22: bool = ((r20) as i16).wrapping_sub((r18) as i16) < 0i16;
                r20 = (r20).wrapping_sub(r18);
                if v22 {
                    Unreachable(game);
                }
                if v21 {
                    return;
                }
            }
            r22 = (r20 as i32 + r26 as i32 - r22 as i32) as u16;
            if !((r22 as i32 & 0x8000i32) != 0i32 || {
                v23 = ((r20) as i16).wrapping_sub((r22) as i16) < 0i16;
                r20 = (r20).wrapping_sub(r22);
                r20 != 0u16 && !v23
            }) {
                break;
            }
            v35 = k;
            v24 = 0;
            v24 = ram_word!(game, vram_write_queue_tail);
            if !(((ram_word!(game, vram_write_queue_tail)) as i16).wrapping_sub(240i16) < 0i16
                && (((512u16).wrapping_sub(ram_word!(game, plm_draw_tilemap_index))) as i32
                    >> 3i32)
                    - r20 as i32
                    & 0x8000i32
                    == 0)
            {
                break;
            }
            CalculatePlmDrawTilemapVramDst(
                ram_word!(game, vram_write_queue_tail),
                r9,
                r12,
                r20,
                r24,
                r26,
                0x8000u16,
                game,
            );
            v25 = gVramWriteEntry(v24 as i32, game);
            (*v25.offset(1)).vram_dst = ((*v25.offset(0)).vram_dst).wrapping_add(1u16);
            r0 = (&raw mut game.g_ram as *mut u8).offset((*v25.offset(0)).src.addr as i32 as isize)
                as *mut u16;
            r6 = (&raw mut game.g_ram as *mut u8).offset((*v25.offset(1)).src.addr as i32 as isize)
                as *mut u16;
            set_ram_word!(game.vram_write_queue_tail = (v24).wrapping_add(14u16));
            r18 = (r18).wrapping_mul(2u16);
            r3 = (r18 as i32 + v1 as i32 + 2i32) as u16;
            v26 = 0;
            v26 = 0u16;
            loop {
                x = *(RomPtr_84orRAM(r3, game) as *mut u16);
                v29 = 0;
                v29 = (v26).wrapping_add(2u16);
                TileTables::draw_block(
                    (&raw const game.g_ram).cast::<u8>().add(0xa000).cast(),
                    x,
                    [
                        r0.add((v26 >> 1) as usize),
                        r6.add((v26 >> 1) as usize),
                        r0.add((v29 >> 1) as usize),
                        r6.add((v29 >> 1) as usize),
                    ],
                    TileDrawOrder::Rows,
                );
                v26 = (v29).wrapping_add(2u16);
                r3 = r3.wrapping_add(1);
                r3 = r3.wrapping_add(1);
                update_ram_word!(game.plm_draw_tilemap_index, wrapping_add(8u16));
                if ram_word!(game, plm_draw_tilemap_index) as i32 - 512i32 & 0x8000i32 == 0 {
                    break '_LABEL_2;
                }
                r20 = r20.wrapping_sub(1);
                if r20 == 0 {
                    break;
                }
            }
        } else {
            r20 = ((v2) as u16) & (0x7fffu16);
            if y as i32 - r26 as i32 & 0x8000i32 != 0 {
                return;
            }
            r26 = y;
            r18 = 0u16;
            r24 = x;
            r22 = ((((ram_word!(game, layer1_x_pos)).wrapping_add(15u16)) as i32 >> 4i32) - 1i32)
                as u16;
            if ((r22) as i16).wrapping_sub((x) as i16) >= 0i16 && r22 != x {
                r18 = (r22).wrapping_sub(x);
                if x as i32 + r20 as i32 == r22 as i32
                    || ((x as i32 + r20 as i32 - r22 as i32) as i16) < 0i16
                {
                    return;
                }
                r20 = (r20).wrapping_sub(r18);
                r24 = r22;
            }
            r22 = (r22).wrapping_add(17u16);
            if (r22 as i32 - x as i32 & 0x8000i32 != 0) {
                break;
            }
            r22 = (r20 as i32 + r24 as i32 - 1i32 - r22 as i32) as u16;
            if !((r22 as i32 & 0x8000i32) != 0i32 || {
                r20 = (r20).wrapping_sub(r22);
                r20 != 0u16
            }) {
                break;
            }
            v35 = k;
            let v3: u16 = ram_word!(game, vram_write_queue_tail);
            if !(((ram_word!(game, vram_write_queue_tail)) as i16).wrapping_sub(480i16) < 0i16
                && (((512u16).wrapping_sub(ram_word!(game, plm_draw_tilemap_index))) as i32
                    >> 3i32)
                    - r20 as i32
                    & 0x8000i32
                    == 0)
            {
                break;
            }
            let prod: u16 = ((r26 as i32 & 0xfi32) * 0x40i32) as u16;
            let v4: u16 = (r24) & (0x1fu16);
            if v4 >= 0x10u16 {
                v8 = (r12 as i32 + 2i32 * v4 as i32) as i16;
                a = (prod).wrapping_add((v8) as u16);
                if (ram_word!(game, bg1_x_offset) as i32 & 0x100i32) != 0i32 {
                    a = (a).wrapping_sub(1024u16);
                }
            } else {
                v5 = (r9 as i32 + 2i32 * v4 as i32) as i16;
                let RegWord: u16 = prod;
                a = (RegWord).wrapping_add((v5) as u16);
                if (ram_word!(game, bg1_x_offset) as i32 & 0x100i32) != 0i32 {
                    a = (a).wrapping_add(1024u16);
                }
            }
            x = (2u16).wrapping_mul(r20);
            let R34: u16 = (a) & (0x1fu16);
            if (2i32 * r20 as i32 + (a as i32 & 0x1fi32) - 1i32 & 0xffe0i32) != 0i32 {
                if ((v3) as i16).wrapping_sub(228i16) >= 0i16
                    || (32i16).wrapping_sub((R34) as i16) < 0i16
                {
                    return;
                }
                let v9: u16 = (2i32 * (32i32 - R34 as i32)) as u16;
                v10 = gVramWriteEntry(v3 as i32, game);
                (*v10).size = v9;
                (*v10.offset(2)).size = v9;
                (*v10.offset(0)).vram_dst = a;
                (*v10.offset(1)).vram_dst = (a as i32 & 0xffe0i32 ^ 0x400i32) as u16;
                (*v10.offset(3)).vram_dst = ((*v10.offset(1)).vram_dst).wrapping_add(32u16);
                (*v10.offset(2)).vram_dst = ((*v10).vram_dst).wrapping_add(32u16);
                x = (4u16).wrapping_mul(r20);
                let v11: u16 = (4i32 * r20 as i32 - (*v10).size as i32) as u16;
                (*v10.offset(1)).size = v11;
                (*v10.offset(3)).size = v11;
                v12 = (ram_word!(game, plm_draw_tilemap_index)).wrapping_sub(14648u16);
                (*v10).src.addr = (ram_word!(game, plm_draw_tilemap_index)).wrapping_sub(14648u16);
                r0 = (&raw mut game.g_ram as *mut u8).offset(v12 as i32 as isize) as *mut u16;
                v13 = ((*v10).size).wrapping_add(v12);
                (*v10.offset(1)).src.addr = v13;
                let v14: u16 = ((*v10.offset(1)).size).wrapping_add(v13);
                (*v10.offset(2)).src.addr = v14;
                r6 = (&raw mut game.g_ram as *mut u8).offset(v14 as i32 as isize) as *mut u16;
                (*v10.offset(3)).src.addr = ((*v10.offset(2)).size).wrapping_add(v14);
                (*v10).src.bank = 126u8;
                (*v10.offset(1)).src.bank = 126u8;
                (*v10.offset(2)).src.bank = 126u8;
                (*v10.offset(3)).src.bank = 126u8;
                set_ram_word!(game.vram_write_queue_tail = (v3).wrapping_add(28u16));
            } else {
                PartiallyQueueVramForSingleScreenPlm(a, v3, r20, 0u16, game);
                v15 = gVramWriteEntry(v3 as i32, game);
                (*v15.offset(1)).vram_dst = ((*v15.offset(0)).vram_dst).wrapping_add(32u16);
                r0 = (&raw mut game.g_ram as *mut u8)
                    .offset((*v15.offset(0)).src.addr as i32 as isize)
                    as *mut u16;
                r6 = (&raw mut game.g_ram as *mut u8)
                    .offset((*v15.offset(1)).src.addr as i32 as isize)
                    as *mut u16;
                set_ram_word!(game.vram_write_queue_tail = (v3).wrapping_add(14u16));
            }
            r18 = (r18).wrapping_mul(2u16);
            r3 = (r18 as i32 + v1 as i32 + 2i32) as u16;
            let mut v16: u16 = 0u16;
            loop {
                x = *(RomPtr_84orRAM(r3, game) as *mut u16);
                let mut v19: u16 = 0;
                v19 = (v16).wrapping_add(2u16);
                TileTables::draw_block(
                    (&raw const game.g_ram).cast::<u8>().add(0xa000).cast(),
                    x,
                    [
                        r0.add((v16 >> 1) as usize),
                        r6.add((v16 >> 1) as usize),
                        r0.add((v19 >> 1) as usize),
                        r6.add((v19 >> 1) as usize),
                    ],
                    TileDrawOrder::Columns,
                );
                v16 = (v19).wrapping_add(2u16);
                r3 = (r3).wrapping_add(2u16);
                update_ram_word!(game.plm_draw_tilemap_index, wrapping_add(8u16));
                if ram_word!(game, plm_draw_tilemap_index) as i32 - 512i32 & 0x8000i32 == 0 {
                    break '_LABEL_2;
                }
                r20 = r20.wrapping_sub(1);
                if r20 == 0 {
                    break;
                }
            }
        }
        k = v35;
        let mut addr: u16 = r3;
        if (r22 as i32 & 0x8000i32) == 0i32 {
            addr = (r3 as i32 + 2i32 * r22 as i32) as u16;
        }
        v31 = *(RomPtr_84orRAM(addr, game) as *mut u16) as i16;
        if v31 != 0 {
            x = (ram_word!(game, plm_x_block)).wrapping_add((v31 as i8) as u16);
            let v32: u16 = (addr).wrapping_add(1u16);
            y = (ram_word!(game, plm_y_block))
                .wrapping_add((*RomPtr_84orRAM(v32, game) as i8) as u16);
            v1 = (v32).wrapping_add(1u16);
        } else {
            return;
        }
    }
}
pub unsafe fn PlmInstr_MovePlmDownOneBlock(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    update_ram_word!(
        game.plm_block_indices[(k as i32 >> 1i32)],
        |fresh39| (fresh39 as i32 + 2i32 * ram_word!(game, room_width_in_blocks) as i32) as u16
    );
    plmp
}
pub unsafe fn PlmSetup_CrumbleBotwoonWall(j: u16, game: &mut Game) -> u8 {
    set_ram_word!(game.plm_instruction_timer[(j as i32 >> 1i32)] = 64u16);
    0u8
}
pub unsafe fn PlmInstr_Scroll_0_1_Blue(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    *((&raw mut game.g_ram as *mut u8).offset(0xcd20) as *mut u16) = 257u16;
    plmp
}
pub unsafe fn PlmInstr_MovePlmDownOneBlock_0(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    update_ram_word!(
        game.plm_block_indices[(k as i32 >> 1i32)],
        |fresh38| (fresh38 as i32 + 2i32 * ram_word!(game, room_width_in_blocks) as i32) as u16
    );
    plmp
}
pub unsafe fn PlmInstr_ABD6(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    update_ram_word!(
        game.plm_block_indices[(k as i32 >> 1i32)],
        wrapping_add(2u16)
    );
    plmp
}
pub unsafe fn PlmPreInstr_PositionSamusAndInvincible(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    set_ram_word!(game.samus_x_pos = ram_word!(game, plm_variable[v1]));
    set_ram_word!(game.samus_y_pos = ram_word!(game, plm_variables[v1]));
    update_ram_word!(game.samus_invincibility_timer, |fresh69| (fresh69)
        | (0x10u16));
}
pub unsafe fn PlmInstr_DealDamage_2(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    update_ram_word!(game.samus_periodic_damage, wrapping_add(2u16));
    plmp
}
pub unsafe fn PlmInstr_GiveInvincibility(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    set_ram_word!(game.samus_invincibility_timer = 48u16);
    plmp
}
pub unsafe fn PlmInstr_Draw0x38FramesOfRightTreadmill(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    WriteRowOfLevelDataBlockAndBTS(k, 0x30ffu16, 0x8u16, 0x38u16, game);
    plmp
}
pub unsafe fn PlmInstr_Draw0x38FramesOfLeftTreadmill(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    WriteRowOfLevelDataBlockAndBTS(k, 0x30ffu16, 0x9u16, 0x38u16, game);
    plmp
}
pub unsafe fn PlmInstr_GotoIfSamusHealthFull(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    if ram_word!(game, samus_max_health) != ram_word!(game, samus_health) {
        return plmp.offset(2);
    }
    CallSomeSamusCode(1u16, game);
    *(plmp as *mut u16) as usize as *const u8
}
pub unsafe fn PlmInstr_GotoIfMissilesFull(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    if ram_word!(game, samus_max_missiles) != ram_word!(game, samus_missiles) {
        return plmp.offset(2);
    }
    CallSomeSamusCode(1u16, game);
    *(plmp as *mut u16) as usize as *const u8
}
pub unsafe fn PlmInstr_PlaceSamusOnSaveStation(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    set_ram_word!(
        game.samus_x_pos = (ram_word!(game, samus_x_pos) as i32 + 8i32 & 0xfff0i32) as u16
    );
    MakeSamusFaceForward(game);
    plmp
}
pub unsafe fn PlmInstr_DisplayGameSavedMessageBox(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    DisplayMessageBox(0x18u16, game);
    plmp
}
pub unsafe fn PlmInstr_EnableMovementAndSetSaveStationUsed(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    CallSomeSamusCode(1u16, game);
    set_ram_word!(game.save_station_lockout_flag = 1u16);
    plmp
}
pub unsafe fn PlmSetup_SetrupWreckedShipEntrance(j: u16, game: &mut Game) -> u8 {
    let mut v2: i16 = 0;
    let mut v1: u16 = ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]);
    v2 = 56i16;
    loop {
        set_ram_word!(game.level_data[(v1 as i32 >> 1i32)] = 255u16);
        v1 = (v1).wrapping_add(2u16);
        v2 -= 1;
        if (v2 == 0) {
            break;
        }
    }
    0u8
}
pub unsafe fn PlmSetup_BTS_Brinstar_0x80_Floorplant(j: u16, game: &mut Game) -> u8 {
    if (ram_word!(game, samus_y_radius) as i32 + ram_word!(game, samus_y_pos) as i32 - 1i32
        & 0xfi32)
        == 15i32
    {
        let v1: i32 = ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]) as i32 >> 1i32;
        update_ram_word!(game.level_data[v1], |fresh20| (fresh20) & (0x8fffu16));
        let v2: i32 = j as i32 >> 1i32;
        set_ram_word!(game.plm_variable[v2] = ram_word!(game, samus_x_pos));
        set_ram_word!(game.plm_variables[v2] = (ram_word!(game, samus_y_pos)).wrapping_sub(1u16));
    } else {
        set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    }
    0u8
}
pub unsafe fn PlmSetup_BTS_Brinstar_0x81_Ceilingplant(j: u16, game: &mut Game) -> u8 {
    if (ram_word!(game, samus_y_pos) as i32 - ram_word!(game, samus_y_radius) as i32 & 0xfi32)
        != 0i32
    {
        set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    } else {
        let v1: i32 = ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]) as i32 >> 1i32;
        update_ram_word!(game.level_data[v1], |fresh19| (fresh19) & (0x8fffu16));
        let v2: i32 = j as i32 >> 1i32;
        set_ram_word!(game.plm_variable[v2] = ram_word!(game, samus_x_pos));
        set_ram_word!(game.plm_variables[v2] = (ram_word!(game, samus_y_pos)).wrapping_add(1u16));
    }
    0u8
}
pub unsafe fn ActivateStationIfSamusCannonLinedUp(a: u16, j: u16, game: &mut Game) -> u8 {
    let current_block: u64;
    let mut v2: u16 = 78u16;
    loop {
        if (a == ram_word!(game, plm_block_indices[(v2 as i32 >> 1i32)])) {
            current_block = 16658872821858055392;
            break;
        }
        v2 = (v2).wrapping_sub(2u16);
        if (v2 as i32 & 0x8000i32) != 0i32 {
            current_block = 12991533354927412482;
            break;
        }
    }
    if current_block == 16658872821858055392 {
        CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
        if ((16u16).wrapping_mul(ram_word!(game, plm_y_block))) as i32 | 0xbi32
            == ram_word!(game, samus_y_pos) as i32
        {
            let v3: i32 = v2 as i32 >> 1i32;
            set_ram_word!(
                game.plm_instr_list_ptrs[v3] = ram_word!(game, plm_instruction_list_link_reg[v3])
            );
            set_ram_word!(game.plm_instruction_timer[v3] = 1u16);
            CallSomeSamusCode(6u16, game);
            return 1u8;
        }
    }
    set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    1u8
}
pub unsafe fn PlmSetup_B6D3_MapStation(j: u16, game: &mut Game) -> u8 {
    let v1: i32 = j as i32 >> 1i32;
    let v2: i32 = ram_word!(game, plm_block_indices[v1]) as i32 >> 1i32;
    set_ram_word!(
        game.level_data[v2] =
            (ram_word!(game, level_data[v2]) as i32 & 0xfffi32 | 0x8000i32) as u16
    );
    if *(&raw mut game.g_ram as *mut u8)
        .offset(0xd908)
        .offset(ram_word!(game, area_index) as isize)
        != 0
    {
        set_ram_word!(game.plm_instr_list_ptrs[v1] = addr_word_84AD76 as u16);
    } else {
        WriteLevelDataBlockTypeAndBts(
            (ram_word!(game, plm_block_indices[v1])).wrapping_add(2u16),
            0xb047u16,
            game,
        );
        WriteLevelDataBlockTypeAndBts(
            (ram_word!(game, plm_block_indices[v1])).wrapping_sub(4u16),
            0xb048u16,
            game,
        );
    }
    0u8
}
pub unsafe fn PlmSetup_Bts47_MapStationRightAccess(j: u16, game: &mut Game) -> u8 {
    if (ram_word!(game, samus_collision_direction) as i32 & 0xfi32) == 0i32
        && ram_word!(game, samus_pose) as i32 == kPose_8A_FaceL_Ranintowall as i32
        && (*(&raw mut game.g_ram as *mut u8).offset(0xa1e) as i32 & 4i32) != 0i32
    {
        return ActivateStationIfSamusCannonLinedUp(
            (ram_word!(game, plm_block_indices[(j as i32 >> 1i32)])).wrapping_sub(2u16),
            j,
            game,
        );
    }
    set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    1u8
}
pub unsafe fn PlmSetup_Bts4_MapStationLeftAccess(j: u16, game: &mut Game) -> u8 {
    if (ram_word!(game, samus_collision_direction) as i32 & 0xfi32) == 1i32
        && ram_word!(game, samus_pose) as i32 == kPose_89_FaceR_Ranintowall as i32
        && (*(&raw mut game.g_ram as *mut u8).offset(0xa1e) as i32 & 8i32) != 0i32
    {
        return ActivateStationIfSamusCannonLinedUp(
            (ram_word!(game, plm_block_indices[(j as i32 >> 1i32)])).wrapping_add(4u16),
            j,
            game,
        );
    }
    set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    1u8
}
pub unsafe fn PlmSetup_PlmB6DF_EnergyStation(j: u16, game: &mut Game) -> u8 {
    let v1: i32 = j as i32 >> 1i32;
    let v2: i32 = ram_word!(game, plm_block_indices[v1]) as i32 >> 1i32;
    set_ram_word!(
        game.level_data[v2] =
            (ram_word!(game, level_data[v2]) as i32 & 0xfffi32 | 0x8000i32) as u16
    );
    WriteLevelDataBlockTypeAndBts(
        (ram_word!(game, plm_block_indices[v1])).wrapping_add(2u16),
        0xb049u16,
        game,
    );
    WriteLevelDataBlockTypeAndBts(
        (ram_word!(game, plm_block_indices[v1])).wrapping_sub(2u16),
        ((0x84b04ai32) as u16),
        game,
    );
    0u8
}
pub unsafe fn PlmSetup_PlmB6EB_EnergyStation(j: u16, game: &mut Game) -> u8 {
    let v1: i32 = j as i32 >> 1i32;
    let v2: i32 = ram_word!(game, plm_block_indices[v1]) as i32 >> 1i32;
    set_ram_word!(
        game.level_data[v2] =
            (ram_word!(game, level_data[v2]) as i32 & 0xfffi32 | 0x8000i32) as u16
    );
    WriteLevelDataBlockTypeAndBts(
        (ram_word!(game, plm_block_indices[v1])).wrapping_add(2u16),
        0xb04bu16,
        game,
    );
    WriteLevelDataBlockTypeAndBts(
        (ram_word!(game, plm_block_indices[v1])).wrapping_sub(2u16),
        0xb04cu16,
        game,
    );
    0u8
}
pub unsafe fn PlmSetup_B6E3_EnergyStationRightAccess(j: u16, game: &mut Game) -> u8 {
    if (ram_word!(game, samus_collision_direction) as i32 & 0xfi32) != 0i32
        || ram_word!(game, samus_pose) as i32 != kPose_8A_FaceL_Ranintowall as i32
        || (*(&raw mut game.g_ram as *mut u8).offset(0xa1e) as i32 & 4i32) == 0i32
        || ram_word!(game, samus_health) == ram_word!(game, samus_max_health)
    {
        set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
        1u8
    } else {
        ActivateStationIfSamusCannonLinedUp(
            (ram_word!(game, plm_block_indices[(j as i32 >> 1i32)])).wrapping_sub(2u16),
            j,
            game,
        )
    }
}
pub unsafe fn PlmSetup_B6E7_EnergyStationLeftAccess(j: u16, game: &mut Game) -> u8 {
    if (ram_word!(game, samus_collision_direction) as i32 & 0xfi32) == 1i32
        && ram_word!(game, samus_pose) as i32 == kPose_89_FaceR_Ranintowall as i32
        && (*(&raw mut game.g_ram as *mut u8).offset(0xa1e) as i32 & 8i32) != 0i32
        && ram_word!(game, samus_health) != ram_word!(game, samus_max_health)
    {
        return ActivateStationIfSamusCannonLinedUp(
            (ram_word!(game, plm_block_indices[(j as i32 >> 1i32)])).wrapping_add(2u16),
            j,
            game,
        );
    }
    set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    1u8
}
pub unsafe fn PlmSetup_B6EF_MissileStationRightAccess(j: u16, game: &mut Game) -> u8 {
    if (ram_word!(game, samus_collision_direction) as i32 & 0xfi32) != 0i32
        || ram_word!(game, samus_pose) as i32 != kPose_8A_FaceL_Ranintowall as i32
        || (*(&raw mut game.g_ram as *mut u8).offset(0xa1e) as i32 & 4i32) == 0i32
        || ram_word!(game, samus_missiles) == ram_word!(game, samus_max_missiles)
    {
        set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
        return 1u8;
    }
    ActivateStationIfSamusCannonLinedUp(
        (ram_word!(game, plm_block_indices[(j as i32 >> 1i32)])).wrapping_sub(2u16),
        j,
        game,
    )
}
pub unsafe fn PlmSetup_B6F3_MissileStationLeftAccess(j: u16, game: &mut Game) -> u8 {
    if (ram_word!(game, samus_collision_direction) as i32 & 0xfi32) == 1i32
        && ram_word!(game, samus_pose) as i32 == kPose_89_FaceR_Ranintowall as i32
        && (*(&raw mut game.g_ram as *mut u8).offset(0xa1e) as i32 & 8i32) != 0i32
        && ram_word!(game, samus_missiles) != ram_word!(game, samus_max_missiles)
    {
        ActivateStationIfSamusCannonLinedUp(
            (ram_word!(game, plm_block_indices[(j as i32 >> 1i32)])).wrapping_add(2u16),
            j,
            game,
        )
    } else {
        set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
        1u8
    }
}
pub fn DeletePlm(j: u16, game: &mut Game) {
    let header_offset = 0x1c37 + usize::from(j & !1);
    game.g_ram[header_offset..header_offset + 2].fill(0);
}
pub fn PlmSetup_B638_Rightwards_Extension(j: u16, game: &mut Game) -> u8 {
    setup_extension_block(j, 0x50ff, game)
}
pub fn PlmSetup_B63F_Leftwards_Extension(j: u16, game: &mut Game) -> u8 {
    setup_extension_block(j, 0x5001, game)
}
pub fn PlmSetup_B643_Downwards_Extension(j: u16, game: &mut Game) -> u8 {
    setup_extension_block(j, 0xd0ff, game)
}
pub fn PlmSetup_B647_Upwards_Extension(j: u16, game: &mut Game) -> u8 {
    setup_extension_block(j, 0xd001, game)
}
fn setup_extension_block(j: u16, block_type_and_bts: u16, game: &mut Game) -> u8 {
    let slot_offset = 0x1c87 + usize::from(j & !1);
    let block_index = u16::from_le_bytes([game.g_ram[slot_offset], game.g_ram[slot_offset + 1]]);
    WriteLevelDataBlockTypeAndBts(block_index, block_type_and_bts, game);
    DeletePlm(j, game);
    0
}
fn advance_plm_instruction(word_index: usize, bytes: u16, game: &mut Game) {
    let offset = 0x1d27 + word_index * 2;
    let instruction = u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]]);
    game.g_ram[offset..offset + 2].copy_from_slice(&instruction.wrapping_add(bytes).to_le_bytes());
}
pub fn SkipDebugDrawInstructionsForScrollPlms(j: u16, game: &mut Game) {
    advance_plm_instruction(usize::from(j >> 1), 4, game);
}
pub fn PlmSetup_B703_ScrollPLM(j: u16, game: &mut Game) -> u8 {
    setup_scroll_block(j, 0x3046, game)
}
pub fn PlmSetup_B707_SolidScrollPLM(j: u16, game: &mut Game) -> u8 {
    setup_scroll_block(j, 0xb046, game)
}
fn setup_scroll_block(j: u16, block_type_and_bts: u16, game: &mut Game) -> u8 {
    let slot_index = usize::from(j & !1);
    let block_offset = 0x1c87 + slot_index;
    let block_index = u16::from_le_bytes([game.g_ram[block_offset], game.g_ram[block_offset + 1]]);
    WriteLevelDataBlockTypeAndBts(block_index, block_type_and_bts, game);
    let trigger_offset = 0x1e17 + slot_index;
    game.g_ram[trigger_offset..trigger_offset + 2].fill(0);
    // Read the live instruction after the level-data/BTS and trigger writes.
    SkipDebugDrawInstructionsForScrollPlms(j, game);
    0
}
pub fn PlmSetup_B6FF_ScrollBlockTouch(j: u16, game: &mut Game) -> u8 {
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    let block_offset = 0x1c87 + usize::from(j & !1);
    let block_index = read_word(game, block_offset);
    write_word(game, block_offset, 0);
    // Search after clearing our block index, including our own slot as before.
    let Some(slot) = (0..40)
        .rev()
        .find(|&slot| read_word(game, 0x1c87 + slot * 2) == block_index)
    else {
        // The original handler hangs when no matching scroll PLM exists.
        loop {}
    };
    let index = slot * 2;
    if read_word(game, 0x1e17 + index) & 0x8000 == 0 {
        write_word(game, 0x1e17 + index, 0x8000);
        // Retain both separate instruction writes before setting its timer.
        advance_plm_instruction(slot, 1, game);
        advance_plm_instruction(slot, 1, game);
        write_word(game, 0xde1c + index, 1);
    }
    0
}
pub fn PlmSetup_DeactivatePlm(j: u16, game: &mut Game) -> u8 {
    retain_plm_block_bits(j, 0x8fff, game);
    0
}
pub fn PlmSetup_ReturnCarryClear(_j: u16) -> u8 {
    0
}
pub fn PlmSetup_ReturnCarrySet(_j: u16) -> u8 {
    1
}
pub fn PlmSetup_D094_EnemyBreakableBlock(j: u16, game: &mut Game) -> u8 {
    // Turn the spike into air, retaining its tile and flip bits.
    retain_plm_block_bits(j, 0x0fff, game);
    0
}
fn update_level_block_bits(word_index: usize, mask: u16, bits: u16, game: &mut Game) {
    let offset = 0x10002 + word_index * 2;
    let block = u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]]);
    game.g_ram[offset..offset + 2].copy_from_slice(&((block & mask) | bits).to_le_bytes());
}
fn retain_plm_block_bits(j: u16, mask: u16, game: &mut Game) {
    // Both indices are byte offsets; the original word indexing rounds them down.
    let slot_offset = 0x1c87 + usize::from(j & !1);
    let block_index = u16::from_le_bytes([game.g_ram[slot_offset], game.g_ram[slot_offset + 1]]);
    update_level_block_bits(usize::from(block_index >> 1), mask, 0, game);
}
pub unsafe fn UNUSED_sub_84B3E3(j: u16) -> u8 {
    '_c2rust_label: {
        __assert_fail(
            b"0\0" as *const u8 as *const c_char,
            b"src/sm_84.c\0" as *const u8 as *const c_char,
            1370u32,
            b"uint8 UNUSED_sub_84B3E3(uint16)\0" as *const u8 as *const c_char,
        );
    };
    0u8
}
pub fn PlmSetup_B70F_IcePhysics(_j: u16, game: &mut Game) -> u8 {
    let y_radius = u16::from_le_bytes([game.g_ram[0xb00], game.g_ram[0xb01]]);
    let y_pos = u16::from_le_bytes([game.g_ram[0xafa], game.g_ram[0xafb]]);
    let bottom = y_pos.wrapping_add(y_radius).wrapping_sub(1);
    if bottom & 0x0f == 7 || bottom & 0x0f == 15 {
        game.g_ram[0xb4c..0xb4e].copy_from_slice(&16u16.to_le_bytes());
    }
    0
}
static g_word_84B48B: [u16; 2] = [0x200, 0x200];
static g_word_84B48F: [u16; 2] = [0x120, 0x100];
static g_word_84B493: [u16; 2] = [0x280, 0x380];
pub fn PlmSetup_QuicksandSurface(_j: u16, game: &mut Game) -> u8 {
    // Clear momentum, boost and extra running speed in the original order.
    for offset in [0xb3c, 0xb3e, 0xb40, 0xb44, 0xb42] {
        game.g_ram[offset..offset + 2].fill(0);
    }
    game.g_ram[0xb49] &= 0x7f;
    game.g_ram[0xb46..0xb48].fill(0);
    let equipped_items = u16::from_le_bytes([game.g_ram[0x9a2], game.g_ram[0x9a3]]);
    let physics_index = if equipped_items & 0x20 != 0 { 2 } else { 0 };
    let reaction_point = u16::from_le_bytes([game.g_ram[0x1e73], game.g_ram[0x1e74]]);
    if reaction_point == 0 {
        let y_direction = u16::from_le_bytes([game.g_ram[0xb36], game.g_ram[0xb37]]);
        match y_direction & 3 {
            0 | 3 => PlmSetup_QuicksandSurface_0(physics_index, game),
            1 => PlmSetup_QuicksandSurface_1(physics_index, game),
            2 => PlmSetup_QuicksandSurface_2(physics_index, game),
            _ => unreachable!(),
        }
    }
    0
}
pub fn PlmSetup_QuicksandSurface_0(j: u16, game: &mut Game) {
    game.g_ram[0xb2c..0xb2e].fill(0);
    game.g_ram[0xb2e..0xb30].fill(0);
    game.g_ram[0xb5a..0xb5c].fill(0);
    game.g_ram[0xb5c..0xb5e].fill(0);
    // Write the middle word of the extra 16.16 Y displacement.
    game.g_ram[0xb5b..0xb5d].copy_from_slice(&g_word_84B48F[usize::from(j >> 1)].to_le_bytes());
}
pub fn PlmSetup_QuicksandSurface_1(j: u16, game: &mut Game) {
    let physics_index = usize::from(j >> 1);
    let y_speed_middle = u16::from_le_bytes([game.g_ram[0xb2d], game.g_ram[0xb2e]]);
    if g_word_84B493[physics_index] < y_speed_middle {
        game.g_ram[0xb2c..0xb2e].fill(0);
        game.g_ram[0xb2e..0xb30].fill(0);
        game.g_ram[0xb2d..0xb2f].copy_from_slice(&g_word_84B493[physics_index].to_le_bytes());
    }
    game.g_ram[0xb5a..0xb5c].fill(0);
    game.g_ram[0xb5c..0xb5e].fill(0);
    game.g_ram[0xb5b..0xb5d].copy_from_slice(&g_word_84B48B[physics_index].to_le_bytes());
}
pub fn PlmSetup_QuicksandSurface_2(j: u16, game: &mut Game) {
    game.g_ram[0xb5a..0xb5c].fill(0);
    game.g_ram[0xb5c..0xb5e].fill(0);
    game.g_ram[0xb5b..0xb5d].copy_from_slice(&g_word_84B48B[usize::from(j >> 1)].to_le_bytes());
    game.g_ram[0xaf4..0xaf6].fill(0);
}
pub fn PlmSetup_B71F_SubmergingQuicksand(_j: u16, game: &mut Game) -> u8 {
    game.g_ram[0xaf4..0xaf6].fill(0);
    game.g_ram[0xb5a..0xb5c].copy_from_slice(&0x2000u16.to_le_bytes());
    game.g_ram[0xb5c..0xb5e].copy_from_slice(&1u16.to_le_bytes());
    0
}
pub fn PlmSetup_B723_SandfallsSlow(_j: u16, game: &mut Game) -> u8 {
    game.g_ram[0xb5a..0xb5c].copy_from_slice(&0x4000u16.to_le_bytes());
    game.g_ram[0xb5c..0xb5e].copy_from_slice(&1u16.to_le_bytes());
    0
}
pub fn PlmSetup_B727_SandFallsFast(_j: u16, game: &mut Game) -> u8 {
    game.g_ram[0xb5a..0xb5c].copy_from_slice(&0xc000u16.to_le_bytes());
    game.g_ram[0xb5c..0xb5e].copy_from_slice(&1u16.to_le_bytes());
    0
}
/// Apply the collision-side quicksand reaction to the active displacement.
///
/// # Safety
/// A non-null `displacement` must point to a live, writable i32 for the
/// duration of this call, as passed through collision PLM setup.
pub unsafe fn PlmSetup_QuicksandSurfaceB(_j: u16, displacement: *mut i32, game: &mut Game) -> u8 {
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    if read_word(game, 0xb02) & 2 == 0 {
        return 0;
    }
    if displacement.is_null() {
        printf(b"ERROR: PlmSetup_QuicksandSurfaceB is broken!!\n\0" as *const u8 as *const c_char);
        Unreachable(game);
        return 0u8;
    }
    match read_word(game, 0xb36) & 3 {
        0 | 3 => {
            if read_word(game, 0xb02) & 0x0f == 3 {
                if read_word(game, 0xa6e) == 1 {
                    *displacement = 0;
                    return 1;
                }
                // Compare the unsigned middle word; retain both outside bytes.
                if (*displacement >> 8) as u16 > 0x30 {
                    *displacement = ((*displacement as u32 & 0xff0000ff) | 0x3000) as i32;
                }
                let in_quicksand = read_word(game, 0x1e71).wrapping_add(1);
                game.g_ram[0x1e71..0x1e73].copy_from_slice(&in_quicksand.to_le_bytes());
            }
        }
        2 => {
            if read_word(game, 0xa6e) == 1 {
                *displacement = 0;
                return 1;
            }
            let in_quicksand = read_word(game, 0x1e71).wrapping_add(1);
            game.g_ram[0x1e71..0x1e73].copy_from_slice(&in_quicksand.to_le_bytes());
        }
        _ => {}
    }
    0
}
pub fn PlmSetup_B737_SubmergingQuicksand(_j: u16, game: &mut Game) -> u8 {
    game.g_ram[0xb2c..0xb2e].fill(0);
    game.g_ram[0xb2e..0xb30].fill(0);
    game.g_ram[0xb32..0xb34].fill(0);
    game.g_ram[0xb34..0xb36].fill(0);
    0
}
pub fn PlmSetup_B73B_B73F_SandFalls(_j: u16) -> u8 {
    0
}
pub unsafe fn PlmSetup_ClearShitroidInvisibleWall(j: u16, game: &mut Game) -> u8 {
    let mut v2: i16 = 0;
    let mut v1: u16 = ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]);
    v2 = 10i16;
    loop {
        update_ram_word!(game.level_data[(v1 as i32 >> 1i32)], |fresh10| (fresh10)
            & (0xfffu16));
        v1 = (v1 as i32 + ram_word!(game, room_width_in_blocks) as i32 * 2i32) as u16;
        v2 -= 1;
        if (v2 == 0) {
            break;
        }
    }
    0u8
}
pub unsafe fn PlmSetup_B767_ClearShitroidInvisibleWall(j: u16, game: &mut Game) -> u8 {
    let mut v2: i16 = 0;
    let mut v1: u16 = ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]);
    v2 = 10i16;
    loop {
        set_ram_word!(
            game.level_data[(v1 as i32 >> 1i32)] =
                (ram_word!(game, level_data[(v1 as i32 >> 1i32)]) as i32 & 0xfffi32 | 0x8000i32)
                    as u16
        );
        v1 = (v1 as i32 + ram_word!(game, room_width_in_blocks) as i32 * 2i32) as u16;
        v2 -= 1;
        if (v2 == 0) {
            break;
        }
    }
    0u8
}
pub unsafe fn PlmSetup_B76B_SaveStationTrigger(j: u16, game: &mut Game) -> u8 {
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    if read_word(game, 0x592) == 0 // power_bomb_explosion_status
        && (game.samus_pose() == kPose_01_FaceR_Normal as u16
            || game.samus_pose() == kPose_02_FaceL_Normal as u16)
        && read_word(game, 0x1e75) == 0 // save_station_lockout_flag
        && game.samus_collision_direction() & 0xf == 3
    {
        CalculatePlmBlockCoords(j, game);
        let samus_block = game.samus_x_pos().wrapping_sub(8) >> 4;
        let station_block = read_word(game, 0x1c29); // plm_x_block
        if samus_block == station_block {
            let trigger_offset = usize::from(j >> 1) * 2;
            let block_index = read_word(game, 0x1c87 + trigger_offset);
            // Remove the temporary trigger before finding the station's PLM.
            game.g_ram[0x1c87 + trigger_offset..0x1c89 + trigger_offset]
                .copy_from_slice(&0u16.to_le_bytes());
            game.g_ram[0x1c37 + trigger_offset..0x1c39 + trigger_offset]
                .copy_from_slice(&0u16.to_le_bytes());
            for slot in (0..40).rev() {
                let slot_offset = slot * 2;
                if read_word(game, 0x1c87 + slot_offset) == block_index {
                    advance_plm_instruction(slot, 1, game);
                    advance_plm_instruction(slot, 1, game);
                    game.g_ram[0xde1c + slot_offset..0xde1e + slot_offset]
                        .copy_from_slice(&1u16.to_le_bytes());
                    break;
                }
            }
        }
    }
    1
}
pub unsafe fn PlmSetup_B76F_SaveStation(j: u16, game: &mut Game) -> u8 {
    WriteLevelDataBlockTypeAndBts(
        ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]),
        0xb04du16,
        game,
    );
    0u8
}
pub unsafe fn PlmSetup_MotherBrainRoomEscapeDoor(j: u16, game: &mut Game) -> u8 {
    let v1: u16 = ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]);
    WriteLevelDataBlockTypeAndBts(v1, 0x9001u16, game);
    let v2: u16 = (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + v1 as i32) as u16;
    WriteLevelDataBlockTypeAndBts(v2, 0xd0ffu16, game);
    let v3: u16 = (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + v2 as i32) as u16;
    WriteLevelDataBlockTypeAndBts(v3, 0xd0ffu16, game);
    WriteLevelDataBlockTypeAndBts(
        (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + v3 as i32) as u16,
        0xd0ffu16,
        game,
    );
    0u8
}
pub unsafe fn PlmSetup_B7EB_EnableSoundsIn32Frames(j: u16, game: &mut Game) -> u8 {
    let mut v1: u16 = 0;
    if ram_word!(game, area_index) == 6u16 {
        v1 = 32u16;
    } else {
        v1 = 240u16;
    }
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.plm_timers[v2] = v1);
    set_ram_word!(game.plm_pre_instrs[v2] = ((0x84b7ddi32) as u16));
    0u8
}
pub unsafe fn PlmPreInstr_B7EB_DecTimerEnableSoundsDeletePlm(k: u16, game: &mut Game) {
    let v2: i32 = k as i32 >> 1i32;
    let ref mut fresh84 =
        *((&raw mut game.g_ram as *mut u8).offset(0x1d77) as *mut u16).offset(v2 as isize);
    let fresh85 = *fresh84;
    *fresh84 = (*fresh84).wrapping_sub(1);
    if fresh85 == 1u16 {
        set_ram_word!(game.debug_disable_sounds = 0u16);
        set_ram_word!(game.plm_header_ptr[v2] = 0u16);
    }
}
pub unsafe fn PlmPreInstr_WakeAndLavaIfBoosterCollected(k: u16, game: &mut Game) {
    if (ram_word!(game, collected_items) as i32 & 0x2000i32) != 0i32 {
        if (ram_word!(game, fx_target_y_pos) as i32 & 0x8000i32) != 0i32 {
            set_ram_word!(game.plm_header_ptr[(k as i32 >> 1i32)] = 0u16);
        } else {
            set_ram_word!(game.fx_y_vel = -128i32 as u16);
            let v1: i32 = k as i32 >> 1i32;
            set_ram_word!(game.plm_instruction_timer[v1] = 1u16);
            update_ram_word!(game.plm_instr_list_ptrs[v1], wrapping_add(1));
            update_ram_word!(game.plm_instr_list_ptrs[v1], wrapping_add(1));
            set_ram_word!(game.plm_timers[v1] = 0u16);
        }
    } else {
        set_ram_word!(game.fx_target_y_pos = -1i32 as u16);
        set_ram_word!(game.fx_y_vel = 0u16);
        set_ram_word!(game.fx_timer = 0u16);
        set_ram_word!(game.earthquake_timer = 0u16);
        set_ram_word!(game.plm_header_ptr[(k as i32 >> 1i32)] = 0u16);
    };
}
pub unsafe fn PlmPreInstr_WakePLMAndStartFxMotionSamusFarLeft(k: u16, game: &mut Game) {
    if ram_word!(game, samus_x_pos) <= 0xae0u16 {
        set_ram_word!(game.fx_timer = 1u16);
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.plm_instruction_timer[v1] = 1u16);
        update_ram_word!(game.plm_instr_list_ptrs[v1], wrapping_add(1));
        update_ram_word!(game.plm_instr_list_ptrs[v1], wrapping_add(1));
        set_ram_word!(game.plm_timers[v1] = 0u16);
    }
}
pub unsafe fn PlmPreInstr_AdvanceLavaSamusMovesLeft(k: u16, game: &mut Game) {
    static g_word_84B876: [u16; 10] = [
        0x72b, 0x1bf, 0xff50, 0x50a, 0x167, 0xff20, 0x244, 0x100, 0xff20, 0x8000,
    ];
    let v1: i32 = k as i32 >> 1i32;
    let v2: u16 = ram_word!(game, plm_timers[v1]);
    let v3: i32 = v2 as i32 >> 1i32;
    let v4: u16 = g_word_84B876[v3 as usize];
    if (v4 as i32 & 0x8000i32) != 0i32 {
        SetEventHappened(0x15u16, game);
    } else if v4 >= ram_word!(game, samus_x_pos) {
        if g_word_84B876[(v3 + 1i32) as usize] < ram_word!(game, fx_base_y_pos) {
            set_ram_word!(game.fx_base_y_pos = g_word_84B876[(v3 + 1i32) as usize]);
        }
        set_ram_word!(game.fx_y_vel = g_word_84B876[(v3 + 2i32) as usize]);
        set_ram_word!(game.plm_timers[v1] = (v2).wrapping_add(6u16));
    }
}
pub unsafe fn PlmSetup_SpeedBoosterEscape(j: u16, game: &mut Game) -> u8 {
    if CheckEventHappened(0x15u16, game) != 0 {
        set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    }
    0u8
}
pub unsafe fn PlmPreInstr_ShaktoolsRoom(k: u16, game: &mut Game) {
    if ram_word!(game, power_bomb_explosion_status) != 0 {
        *((&raw mut game.g_ram as *mut u8).offset(0xcd20) as *mut u16) = 257u16;
        *((&raw mut game.g_ram as *mut u8).offset(0xcd20).offset(2) as *mut u8 as *mut u16) =
            257u16;
    }
    if ram_word!(game, samus_x_pos) > 0x348u16 {
        SetEventHappened(0xdu16, game);
        set_ram_word!(game.plm_header_ptr[(k as i32 >> 1i32)] = 0u16);
    }
}
pub unsafe fn PlmSetup_ShaktoolsRoom(j: u16, game: &mut Game) -> u8 {
    *((&raw mut game.g_ram as *mut u8).offset(0xcd20) as *mut u16) = 1u16;
    *((&raw mut game.g_ram as *mut u8).offset(0xcd20).offset(2) as *mut u8 as *mut u16) = 0u16;
    0u8
}
pub unsafe fn WakePlmIfSamusIsBelowAndRightOfTarget(
    k: u16,
    x_r18: u16,
    y_r20: u16,
    game: &mut Game,
) -> u8 {
    if x_r18 < ram_word!(game, samus_x_pos) && y_r20 < ram_word!(game, samus_y_pos) {
        let v2: i32 = k as i32 >> 1i32;
        update_ram_word!(game.plm_instr_list_ptrs[v2], wrapping_add(2u16));
        set_ram_word!(game.plm_instruction_timer[v2] = 1u16);
        return false_0 as u8;
    }
    true_0 as u8
}
pub unsafe fn PlmPreInstr_OldTourianEscapeShaftEscape(k: u16, game: &mut Game) {
    if WakePlmIfSamusIsBelowAndRightOfTarget(k, 240u16, 2080u16, game) == 0 {
        SpawnEprojWithRoomGfx(0xb4b1u16, 0u16, game);
    }
}
pub unsafe fn PlmPreInstr_EscapeRoomBeforeOldTourianEscapeShaft(k: u16, game: &mut Game) {
    if WakePlmIfSamusIsBelowAndRightOfTarget(k, 240u16, 1344u16, game) == 0 {
        set_ram_word!(game.fx_y_vel = -104i32 as u16);
        set_ram_word!(game.fx_timer = 16u16);
    }
}
pub fn PlmSetup_B974(j: u16) -> u8 {
    0u8
}
pub unsafe fn PlmSetup_B9C1_CrittersEscapeBlock(j: u16, game: &mut Game) -> u8 {
    if *((&raw mut game.g_ram as *mut u8).offset(0xc18) as *mut u16)
        .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize)
        != 0
    {
        let v1: i32 = j as i32 >> 1i32;
        let v2: i32 = ram_word!(game, plm_block_indices[v1]) as i32 >> 1i32;
        let v3: u16 = (ram_word!(game, level_data[v2]) as i32 & 0xf000i32 | 0x9fi32) as u16;
        set_ram_word!(game.plm_variable[v1] = v3);
        set_ram_word!(game.level_data[v2] = (v3) & (0x8fffu16));
    } else {
        set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    }
    0u8
}
pub unsafe fn PlmInstr_SetCrittersEscapedEvent(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    SetEventHappened(0xfu16, game);
    plmp
}
pub unsafe fn PlmSetup_B9ED_CrittersEscapeBlock(j: u16, game: &mut Game) -> u8 {
    let v1: u16 = ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]);
    WriteLevelDataBlockTypeAndBts(v1, 0xc04fu16, game);
    let v2: u16 = (v1 as i32 + ram_word!(game, room_width_in_blocks) as i32 * 2i32) as u16;
    WriteLevelDataBlockTypeAndBts(v2, 0xd0ffu16, game);
    WriteLevelDataBlockTypeAndBts(
        (v2 as i32 + ram_word!(game, room_width_in_blocks) as i32 * 2i32) as u16,
        0xd0ffu16,
        game,
    );
    0u8
}
pub unsafe fn sub_84B9F1(j: u16, game: &mut Game) -> u8 {
    let v1: u16 = ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]);
    update_level_block_bits((v1 >> 1) as usize, 0xfff, 0x8000, game);
    let v2: u16 = (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + v1 as i32) as u16;
    update_level_block_bits((v2 >> 1) as usize, 0xfff, 0x8000, game);
    let v3: u16 = (ram_word!(game, room_width_in_blocks) as i32
        + ram_word!(game, room_width_in_blocks) as i32
        + v2 as i32) as u16;
    update_level_block_bits((v3 >> 1) as usize, 0xfff, 0x8000, game);
    let v4: i32 =
        (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + v3 as i32) as u16 as i32 >> 1i32;
    update_level_block_bits(v4 as usize, 0xfff, 0x8000, game);
    0u8
}
pub unsafe fn PlmInstr_JumpIfSamusHasNoBombs(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    if (ram_word!(game, collected_items) as i32 & 0x1000i32) != 0i32 {
        plmp.offset(2)
    } else {
        *(plmp as *mut u16) as usize as *const u8
    }
}
pub unsafe fn PlmSetup_BB30_CrateriaMainstreetEscape(j: u16, game: &mut Game) -> u8 {
    if CheckEventHappened(0xfu16, game) == 0 {
        set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    }
    0u8
}
pub unsafe fn PlmInstr_MovePlmRight4Blocks(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    update_ram_word!(
        game.plm_block_indices[(k as i32 >> 1i32)],
        wrapping_add(8u16)
    );
    plmp
}
pub unsafe fn PlmPreInstr_WakePlmIfTriggered(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, plm_timers[v1]) != 0 {
        update_ram_word!(game.plm_instr_list_ptrs[v1], wrapping_add(1));
        update_ram_word!(game.plm_instr_list_ptrs[v1], wrapping_add(1));
        set_ram_word!(game.plm_instruction_timer[v1] = 1u16);
        set_ram_word!(game.plm_pre_instrs[v1] = addr_locret_84BB6A as u16);
    }
}
pub unsafe fn PlmPreInstr_WakePlmIfTriggeredOrSamusBelowPlm(k: u16, game: &mut Game) {
    CalculatePlmBlockCoords(k, game);
    if ram_word!(game, samus_x_pos) as i32 >> 4i32 == ram_word!(game, plm_x_block) as i32
        && (((ram_word!(game, samus_y_pos) as i32 >> 4i32) - ram_word!(game, plm_y_block) as i32)
            as u16)
            < 5u16
        || ram_word!(game, plm_timers[(k as i32 >> 1i32)]) != 0u16
    {
        let v1: i32 = k as i32 >> 1i32;
        advance_plm_instruction(v1 as usize, 1, game);
        advance_plm_instruction(v1 as usize, 1, game);
        set_ram_word!(game.plm_instruction_timer[v1] = 1u16);
        set_ram_word!(game.plm_pre_instrs[v1] = ((0x84bba3i32) as u16));
    }
}
pub unsafe fn PlmPreInstr_WakePlmIfTriggeredOrSamusAbovePlm(k: u16, game: &mut Game) {
    CalculatePlmBlockCoords(k, game);
    if ram_word!(game, samus_x_pos) as i32 >> 4i32 == ram_word!(game, plm_x_block) as i32
        && (((ram_word!(game, samus_y_pos) as i32 >> 4i32) - ram_word!(game, plm_y_block) as i32)
            as u16)
            >= 0xfffcu16
        || ram_word!(game, plm_timers[(k as i32 >> 1i32)]) != 0u16
    {
        let v1: i32 = k as i32 >> 1i32;
        update_ram_word!(game.plm_instr_list_ptrs[v1], wrapping_add(1));
        update_ram_word!(game.plm_instr_list_ptrs[v1], wrapping_add(1));
        set_ram_word!(game.plm_instruction_timer[v1] = 1u16);
        set_ram_word!(game.plm_pre_instrs[v1] = addr_locret_84BBDC as u16);
    }
}
pub unsafe fn PlmInstr_ClearTrigger(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    set_ram_word!(game.plm_timers[(k as i32 >> 1i32)] = 0u16);
    plmp
}
pub unsafe fn PlmInstr_SpawnEproj(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    SpawnEprojWithRoomGfx(*(plmp as *mut u16), 0u16, game);
    plmp.offset(2)
}
pub unsafe fn PlmInstr_WakeEprojAtPlmPos(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    let mut i: i32 = 0;
    let v2: u16 = ram_word!(game, plm_block_indices[(k as i32 >> 1i32)]);
    i = 34i32;
    while i >= 0i32 {
        if v2 == ram_word!(game, eproj_E[(i >> 1i32)]) {
            break;
        }
        i -= 2i32;
    }
    let v4: i32 = i >> 1i32;
    set_ram_word!(game.eproj_instr_timers[v4] = 1u16);
    update_ram_word!(game.eproj_instr_list_ptr[v4], wrapping_add(2u16));
    plmp.offset(2)
}
pub unsafe fn PlmPreInstr_GoToLinkInstrIfShot(k: u16, game: &mut Game) {
    let v2: i32 = k as i32 >> 1i32;
    if ram_word!(game, plm_timers[v2]) != 0 {
        set_ram_word!(game.plm_timers[v2] = 0u16);
        set_ram_word!(
            game.plm_instr_list_ptrs[v2] = ram_word!(game, plm_instruction_list_link_reg[v2])
        );
        set_ram_word!(game.plm_instruction_timer[v2] = 1u16);
    }
}
pub unsafe fn PlmPreInstr_GoToLinkInstrIfShotWithPowerBomb(k: u16, game: &mut Game) {
    let v2: i32 = k as i32 >> 1i32;
    let v3: u16 = ram_word!(game, plm_timers[v2]);
    if v3 != 0 {
        if (v3 as i32 & 0xf00i32) == 768i32 {
            set_ram_word!(game.plm_timers[v2] = 0u16);
            set_ram_word!(
                game.plm_instr_list_ptrs[v2] = ram_word!(game, plm_instruction_list_link_reg[v2])
            );
            set_ram_word!(game.plm_instruction_timer[v2] = 1u16);
            return;
        }
        QueueSfx2_Max6(0x57u16, game);
    }
    set_ram_word!(game.plm_timers[v2] = 0u16);
}
pub unsafe fn PlmPreInstr_GoToLinkInstrIfShotWithAnyMissile(k: u16, game: &mut Game) {
    let index = usize::from(k >> 1) * 2;
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    let shot = read_word(game, 0x1d77 + index); // plm_timers
    if shot != 0 {
        match shot & 0x0f00 {
            0x0200 => write_word(game, 0xdf0c + index, 119), // plm_variables
            0x0100 => {}
            _ => {
                QueueSfx2_Max6(0x57, game);
                write_word(game, 0x1d77 + index, 0);
                return;
            }
        }
        write_word(game, 0x1d77 + index, 0);
        let link = read_word(game, 0xdebc + index); // plm_instruction_list_link_reg
        write_word(game, 0x1d27 + index, link); // plm_instr_list_ptrs
        write_word(game, 0xde1c + index, 1); // plm_instruction_timer
        return;
    }
    write_word(game, 0x1d77 + index, 0);
}
pub unsafe fn PlmPreInstr_GoToLinkInstrIfShotWithSuperMissile(k: u16, game: &mut Game) {
    let v2: i32 = k as i32 >> 1i32;
    let v3: u16 = ram_word!(game, plm_timers[v2]);
    if v3 != 0 {
        if (v3 as i32 & 0xf00i32) == 512i32 {
            set_ram_word!(game.plm_timers[v2] = 0u16);
            set_ram_word!(
                game.plm_instr_list_ptrs[v2] = ram_word!(game, plm_instruction_list_link_reg[v2])
            );
            set_ram_word!(game.plm_instruction_timer[v2] = 1u16);
            return;
        }
        QueueSfx2_Max6(0x57u16, game);
    }
    set_ram_word!(game.plm_timers[v2] = 0u16);
}
pub unsafe fn PlmPreInstr_GoToLinkInstruction(k: u16, game: &mut Game) {
    let v2: i32 = k as i32 >> 1i32;
    set_ram_word!(game.plm_timers[v2] = 0u16);
    set_ram_word!(
        game.plm_instr_list_ptrs[v2] = ram_word!(game, plm_instruction_list_link_reg[v2])
    );
    set_ram_word!(game.plm_instruction_timer[v2] = 1u16);
}
pub unsafe fn PlmPreInstr_PlayDudSound(k: u16, game: &mut Game) {
    let v2: i32 = k as i32 >> 1i32;
    if ram_word!(game, plm_timers[v2]) != 0 {
        QueueSfx2_Max6(0x57u16, game);
    }
    set_ram_word!(game.plm_timers[v2] = 0u16);
}
pub unsafe fn PlmPreInstr_GotoLinkIfBoss1Dead(k: u16, game: &mut Game) {
    if CheckBossBitForCurArea(1u16, game) as i32 & 1i32 != 0 {
        PlmPreInstr_GoToLinkInstruction(k, game);
    } else {
        PlmPreInstr_PlayDudSound(k, game);
    };
}
pub unsafe fn PlmPreInstr_GotoLinkIfMiniBossDead(k: u16, game: &mut Game) {
    if CheckBossBitForCurArea(2u16, game) as i32 & 1i32 != 0 {
        PlmPreInstr_GoToLinkInstruction(k, game);
    } else {
        PlmPreInstr_PlayDudSound(k, game);
    };
}
pub unsafe fn PlmPreInstr_GotoLinkIfTorizoDead(k: u16, game: &mut Game) {
    if CheckBossBitForCurArea(4u16, game) as i32 & 1i32 != 0 {
        PlmPreInstr_GoToLinkInstruction(k, game);
    } else {
        PlmPreInstr_PlayDudSound(k, game);
    };
}
pub unsafe fn PlmPreInstr_GotoLinkIfEnemyDeathQuotaOk(k: u16, game: &mut Game) {
    if ram_word!(game, num_enemies_killed_in_room) < ram_word!(game, num_enemy_deaths_left_to_clear)
    {
        PlmPreInstr_PlayDudSound(k, game);
    } else {
        SetEventHappened(0u16, game);
        PlmPreInstr_GoToLinkInstruction(k, game);
    };
}
pub unsafe fn PlmPreInstr_GotoLinkIfTourianStatueFinishedProcessing(k: u16, game: &mut Game) {
    if (ram_word!(game, tourian_entrance_statue_finished) as i32 & 0x8000i32) == 0i32 {
        PlmPreInstr_PlayDudSound(k, game);
    } else {
        PlmPreInstr_GoToLinkInstruction(k, game);
    };
}
pub unsafe fn PlmPreInstr_GotoLinkIfCrittersEscaped(k: u16, game: &mut Game) {
    if CheckEventHappened(0xfu16, game) as i32 & 1i32 != 0 {
        PlmPreInstr_GoToLinkInstruction(k, game);
    } else {
        PlmPreInstr_PlayDudSound(k, game);
    };
}
pub unsafe fn PlmInstr_SetGreyDoorPreInstr(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    set_ram_word!(
        game.plm_pre_instrs[(k as i32 >> 1i32)] = *(game
            .g_rom
            .offset(crate::rtl::lorom_offset(0x84be4b))
            as *const u8 as *mut u16)
            .offset((ram_word!(game, plm_variable[(k as i32 >> 1i32)]) as i32 >> 1i32) as isize,)
    );
    plmp
}
pub unsafe fn PlmSetup_C806_LeftGreenGateTrigger(j: u16, game: &mut Game) -> u8 {
    if (*((&raw mut game.g_ram as *mut u8).offset(0xc18) as *mut u16)
        .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize) as i32
        & 0xfffi32)
        == 512i32
    {
        return sub_84C63F(j, game);
    }
    QueueSfx2_Max6(0x57u16, game);
    set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    0u8
}
pub unsafe fn PlmSetup_C80A_RightGreenGateTrigger(j: u16, game: &mut Game) -> u8 {
    if (*((&raw mut game.g_ram as *mut u8).offset(0xc18) as *mut u16)
        .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize) as i32
        & 0xfffi32)
        == 512i32
    {
        return sub_84C647(j, game);
    }
    QueueSfx2_Max6(0x57u16, game);
    set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    0u8
}
pub unsafe fn PlmSetup_C80E_LeftRedGateTrigger(j: u16, game: &mut Game) -> u8 {
    let mut v1: i16 = 0;
    v1 = ((*((&raw mut game.g_ram as *mut u8).offset(0xc18) as *mut u16)
        .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize)) as i16)
        & (0xfffi16);
    if v1 == 256i16 || v1 == 512i16 {
        return sub_84C63F(j, game);
    }
    QueueSfx2_Max6(0x57u16, game);
    set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    0u8
}
pub unsafe fn PlmSetup_C812_RightRedGateTrigger(j: u16, game: &mut Game) -> u8 {
    let mut v1: i16 = 0;
    v1 = ((*((&raw mut game.g_ram as *mut u8).offset(0xc18) as *mut u16)
        .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize)) as i16)
        & (0xfffi16);
    if v1 == 256i16 || v1 == 512i16 {
        return sub_84C647(j, game);
    }
    QueueSfx2_Max6(0x57u16, game);
    set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    0u8
}
pub unsafe fn PlmSetup_C822_RightYellowGateTrigger(j: u16, game: &mut Game) -> u8 {
    if (*((&raw mut game.g_ram as *mut u8).offset(0xc18) as *mut u16)
        .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize) as i32
        & 0xfffi32)
        != 768i32
    {
        return sub_84C647(j, game);
    }
    QueueSfx2_Max6(0x57u16, game);
    set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    0u8
}
pub unsafe fn PlmSetup_C81E_LeftYellowGateTrigger(j: u16, game: &mut Game) -> u8 {
    if (*((&raw mut game.g_ram as *mut u8).offset(0xc18) as *mut u16)
        .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize) as i32
        & 0xfffi32)
        == 768i32
    {
        return sub_84C63F(j, game);
    }
    QueueSfx2_Max6(0x57u16, game);
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.plm_header_ptr[v1] = 0u16);
    PlmSetup_C822_RightYellowGateTrigger(j, game)
}
pub unsafe fn PlmSetup_C81A_RightBlueGateTrigger(j: u16, game: &mut Game) -> u8 {
    if (*((&raw mut game.g_ram as *mut u8).offset(0xc18) as *mut u16)
        .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize) as i32
        & 0xfffi32)
        != 768i32
    {
        return sub_84C647(j, game);
    }
    set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    0u8
}
pub unsafe fn PlmSetup_C816_LeftBlueGateTrigger(j: u16, game: &mut Game) -> u8 {
    if (*((&raw mut game.g_ram as *mut u8).offset(0xc18) as *mut u16)
        .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize) as i32
        & 0xfffi32)
        != 768i32
    {
        return sub_84C63F(j, game);
    }
    set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    PlmSetup_C81A_RightBlueGateTrigger(j, game)
}
pub unsafe fn sub_84C63F(j: u16, game: &mut Game) -> u8 {
    sub_84C64C(
        j,
        (ram_word!(game, plm_block_indices[(j as i32 >> 1i32)])).wrapping_add(2u16),
        game,
    )
}
pub unsafe fn sub_84C647(j: u16, game: &mut Game) -> u8 {
    sub_84C64C(
        j,
        (ram_word!(game, plm_block_indices[(j as i32 >> 1i32)])).wrapping_sub(2u16),
        game,
    )
}
pub unsafe fn sub_84C64C(j: u16, a: u16, game: &mut Game) -> u8 {
    let mut v3: i32 = 0;
    let current_block: u64;
    let mut v2: u16 = 78u16;
    loop {
        if (a == ram_word!(game, plm_block_indices[(v2 as i32 >> 1i32)])) {
            current_block = 16658872821858055392;
            break;
        }
        v2 = (v2).wrapping_sub(2u16);
        if (v2 as i32 & 0x8000i32) != 0i32 {
            current_block = 5208286747870620093;
            break;
        }
    }
    if current_block == 16658872821858055392 {
        v3 = 0;
        v3 = v2 as i32 >> 1i32;
        if ram_word!(game, plm_timers[v3]) == 0 {
            update_ram_word!(game.plm_timers[v3], wrapping_add(1));
        }
    }
    set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    1u8
}
pub unsafe fn SetBts0x10FiveStepsDown(j: u16, game: &mut Game) {
    let v1: u16 = SetBtsTo0x10AdvanceRow(
        (ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]) as i32 >> 1i32) as u16,
        game,
    );
    let v2: u16 = SetBtsTo0x10AdvanceRow(v1, game);
    let v3: u16 = SetBtsTo0x10AdvanceRow(v2, game);
    let v4: u16 = SetBtsTo0x10AdvanceRow(v3, game);
    SetBtsTo0x10AdvanceRow(v4, game);
}
pub unsafe fn SetBtsTo0x10AdvanceRow(k: u16, game: &mut Game) -> u16 {
    *((&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(k as isize) as *mut u8 as *mut u16) = ((*(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset((k as i32 + 1i32) as isize)
        as i32)
        << 8i32
        | 0x10i32) as u16;
    (ram_word!(game, room_width_in_blocks)).wrapping_add(k)
}
pub unsafe fn SetBts0x10FiveStepsUp(j: u16, game: &mut Game) {
    let v1: u16 = SetBtsTo0x10AdvanceRowUp(
        (ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]) as i32 >> 1i32) as u16,
        game,
    );
    let v2: u16 = SetBtsTo0x10AdvanceRowUp(v1, game);
    let v3: u16 = SetBtsTo0x10AdvanceRowUp(v2, game);
    let v4: u16 = SetBtsTo0x10AdvanceRowUp(v3, game);
    SetBtsTo0x10AdvanceRowUp(v4, game);
}
pub unsafe fn SetBtsTo0x10AdvanceRowUp(k: u16, game: &mut Game) -> u16 {
    *((&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(k as isize) as *mut u8 as *mut u16) = ((*(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset((k as i32 + 1i32) as isize)
        as i32)
        << 8i32
        | 0x10i32) as u16;
    (k).wrapping_sub(ram_word!(game, room_width_in_blocks))
}
pub unsafe fn PlmSetup_C82A_DownwardsClosedGate(j: u16, setup_a: u16, game: &mut Game) -> u8 {
    SpawnEprojWithRoomGfx(0xe659u16, setup_a, game);
    SetBts0x10FiveStepsDown(j, game);
    0u8
}
pub unsafe fn PlmSetup_C832_UpwardsClosedGate(j: u16, game: &mut Game) -> u8 {
    SpawnEprojWithRoomGfx(0xe675u16, 0u16, game);
    SetBts0x10FiveStepsUp(j, game);
    0u8
}
pub unsafe fn PlmSetup_C826_DownwardsOpenGate(j: u16, game: &mut Game) -> u8 {
    SetBts0x10FiveStepsDown(j, game);
    0u8
}
pub unsafe fn PlmSetup_C82E_UpwardsOpenGate(j: u16, game: &mut Game) -> u8 {
    SetBts0x10FiveStepsUp(j, game);
    0u8
}
pub unsafe fn PlmSetup_C836_DownwardsGateShootblock(j: u16, game: &mut Game) -> u8 {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.plm_instr_list_ptrs[v1] = crate::rtl::rom_table_word!(
            game,
            0x84c70a,
            (ram_word!(game, plm_room_arguments[v1]) as i32 >> 1i32) as isize,
        )
    );
    let v2: u16 = crate::rtl::rom_table_word!(
        game,
        0x84c71a,
        (ram_word!(game, plm_room_arguments[v1]) as i32 >> 1i32) as isize
    );
    if v2 != 0 {
        WriteLevelDataBlockTypeAndBts(
            (ram_word!(game, plm_block_indices[v1])).wrapping_sub(2u16),
            v2,
            game,
        );
    }
    let v3: u16 = crate::rtl::rom_table_word!(
        game,
        0x84c72a,
        (ram_word!(game, plm_room_arguments[v1]) as i32 >> 1i32) as isize
    );
    if v3 != 0 {
        WriteLevelDataBlockTypeAndBts(
            (ram_word!(game, plm_block_indices[v1])).wrapping_add(2u16),
            v3,
            game,
        );
    }
    0u8
}
pub unsafe fn PlmSetup_C73A_UpwardsGateShootblock(j: u16, game: &mut Game) -> u8 {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.plm_instr_list_ptrs[v1] = crate::rtl::rom_table_word!(
            game,
            0x84c764,
            (ram_word!(game, plm_room_arguments[v1]) as i32 >> 1i32) as isize,
        )
    );
    let v2: u16 = crate::rtl::rom_table_word!(
        game,
        0x84c774,
        (ram_word!(game, plm_room_arguments[v1]) as i32 >> 1i32) as isize
    );
    if v2 != 0 {
        WriteLevelDataBlockTypeAndBts(
            (ram_word!(game, plm_block_indices[v1])).wrapping_sub(2u16),
            v2,
            game,
        );
    }
    let v3: u16 = crate::rtl::rom_table_word!(
        game,
        0x84c784,
        (ram_word!(game, plm_room_arguments[v1]) as i32 >> 1i32) as isize
    );
    if v3 != 0 {
        WriteLevelDataBlockTypeAndBts(
            (ram_word!(game, plm_block_indices[v1])).wrapping_add(2u16),
            v3,
            game,
        );
    }
    0u8
}
pub unsafe fn PlmSetup_C794_GreyDoor(j: u16, game: &mut Game) -> u8 {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.plm_variable[v1] = ((((*(((&raw mut game.g_ram as *mut u8).offset(0x1dc7) as *mut u16)
            .offset(v1 as isize) as *mut u16 as *mut u8)
            .offset(1isize,)) as u16)
            & (0x7cu16)) as i32
            >> 1i32) as u16
    );
    update_ram_word!(
        game.plm_room_arguments[v1],
        |fresh6| (fresh6 as i32 & !(0x7c00i32)) as u16
    );
    WriteLevelDataBlockTypeAndBts(ram_word!(game, plm_block_indices[v1]), 0xc044u16, game);
    0u8
}
pub fn PlmSetup_Door_Colored(j: u16, game: &mut Game) -> u8 {
    let index = usize::from(j >> 1) * 2;
    let block_index = u16::from_le_bytes([game.g_ram[0x1c87 + index], game.g_ram[0x1c88 + index]]);
    WriteLevelDataBlockTypeAndBts(block_index, 0xc044, game);
    0
}
pub unsafe fn PlmSetup_Door_Blue(j: u16, game: &mut Game) -> u8 {
    if ram_word!(game, projectile_index) as i32 & 0x8000i32 == 0
        && (*((&raw mut game.g_ram as *mut u8).offset(0xc18) as *mut u16)
            .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize) as i32
            & 0xf00i32)
            == 768i32
    {
        set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    } else {
        let index = usize::from(j >> 1) * 2;
        let v1 = u16::from_le_bytes([game.g_ram[0x1c87 + index], game.g_ram[0x1c88 + index]]);
        set_ram_word!(
            game.level_data[(v1 as i32 >> 1i32)] =
                (ram_word!(game, level_data[(v1 as i32 >> 1i32)]) as i32 & 0xfffi32 | 0x8000i32)
                    as u16
        );
    }
    0u8
}
pub unsafe fn PlmSetup_Door_Strange(j: u16, game: &mut Game) -> u8 {
    let v1: i32 = j as i32 >> 1i32;
    let v2: u16 = ram_word!(game, plm_block_indices[v1]);
    set_ram_word!(game.plm_block_indices[v1] = 0u16);
    let mut v3: u16 = 78u16;
    while v2 != ram_word!(game, plm_block_indices[(v3 as i32 >> 1i32)]) {
        v3 = (v3).wrapping_sub(2u16);
        if (v3 as i32 & 0x8000i32) != 0i32 {
            return 0u8;
        }
    }
    set_ram_word!(
        game.plm_timers[(v3 as i32 >> 1i32)] = (*((&raw mut game.g_ram as *mut u8).offset(0xc18)
            as *mut u16)
            .offset((ram_word!(game, projectile_index) as i16 as i32 >> 1i32) as isize,)
            as i32
            & 0x1fffi32
            | 0x8000i32) as u16
    );
    0u8
}
pub unsafe fn PlmInstr_SetBtsTo1(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    *(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset((ram_word!(game, plm_block_indices[(k as i32 >> 1i32)]) as i32 >> 1i32) as isize) =
        1u8;
    plmp
}
pub unsafe fn PlmSetup_D028_D02C_Unused(j: u16, game: &mut Game) -> u8 {
    if ram_word!(game, samus_pose) as i32 == kPose_81_FaceR_Screwattack as i32
        || ram_word!(game, samus_pose) as i32 == kPose_82_FaceL_Screwattack as i32
    {
        let v1: i32 = j as i32 >> 1i32;
        let v2: i32 = ram_word!(game, plm_block_indices[v1]) as i32 >> 1i32;
        let v3: u16 = ram_word!(game, level_data[v2]);
        set_ram_word!(game.plm_variable[v1] = v3);
        set_ram_word!(game.level_data[v2] = (v3) & (0xfffu16));
        0u8
    } else {
        set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
        1u8
    }
}
pub unsafe fn PlmSetup_RespawningSpeedBoostBlock(j: u16, game: &mut Game) -> u8 {
    if (ram_word!(game, speed_boost_counter) as i32 & 0xf00i32) == 1024i32
        || ram_word!(game, samus_pose) as i32 == kPose_C9_FaceR_Shinespark_Horiz as i32
        || ram_word!(game, samus_pose) as i32 == kPose_CA_FaceL_Shinespark_Horiz as i32
        || ram_word!(game, samus_pose) as i32 == kPose_CB_FaceR_Shinespark_Vert as i32
        || ram_word!(game, samus_pose) as i32 == kPose_CC_FaceL_Shinespark_Vert as i32
        || ram_word!(game, samus_pose) as i32 == kPose_CD_FaceR_Shinespark_Diag as i32
        || ram_word!(game, samus_pose) as i32 == kPose_CE_FaceL_Shinespark_Diag as i32
    {
        let v1: i32 = j as i32 >> 1i32;
        let v2: i32 = ram_word!(game, plm_block_indices[v1]) as i32 >> 1i32;
        let v3: u16 = (ram_word!(game, level_data[v2]) as i32 & 0xf000i32 | 0xb6i32) as u16;
        set_ram_word!(game.plm_variable[v1] = v3);
        set_ram_word!(game.level_data[v2] = (v3) & (0xfffu16));
        0u8
    } else {
        set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
        1u8
    }
}
pub unsafe fn PlmSetup_RespawningCrumbleBlock(j: u16, game: &mut Game) -> u8 {
    if (ram_word!(game, samus_collision_direction) as i32 & 0xfi32) == 3i32 {
        let v1: i32 = j as i32 >> 1i32;
        let v2: i32 = ram_word!(game, plm_block_indices[v1]) as i32 >> 1i32;
        let v3: u16 = (ram_word!(game, level_data[v2]) as i32 & 0xf000i32 | 0xbci32) as u16;
        set_ram_word!(game.plm_variable[v1] = v3);
        set_ram_word!(game.level_data[v2] = (v3) & (0x8fffu16));
        set_ram_word!(game.plm_instruction_timer[v1] = 4u16);
    } else {
        set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    }
    1u8
}
pub unsafe fn PlmSetup_RespawningShotBlock(j: u16, game: &mut Game) -> u8 {
    let v1: i32 = j as i32 >> 1i32;
    let v2: i32 = ram_word!(game, plm_block_indices[v1]) as i32 >> 1i32;
    let v3: u16 = (ram_word!(game, level_data[v2]) as i32 & 0xf000i32 | 0x52i32) as u16;
    set_ram_word!(game.plm_variable[v1] = v3);
    set_ram_word!(game.level_data[v2] = (v3) & (0x8fffu16));
    0u8
}
pub unsafe fn PlmSetup_RespawningBombBlock(j: u16, game: &mut Game) -> u8 {
    if (ram_word!(game, speed_boost_counter) as i32 & 0xf00i32) == 1024i32
        || ram_word!(game, samus_pose) as i32 == kPose_81_FaceR_Screwattack as i32
        || ram_word!(game, samus_pose) as i32 == kPose_82_FaceL_Screwattack as i32
        || ram_word!(game, samus_pose) as i32 == kPose_C9_FaceR_Shinespark_Horiz as i32
        || ram_word!(game, samus_pose) as i32 == kPose_CA_FaceL_Shinespark_Horiz as i32
        || ram_word!(game, samus_pose) as i32 == kPose_CB_FaceR_Shinespark_Vert as i32
        || ram_word!(game, samus_pose) as i32 == kPose_CC_FaceL_Shinespark_Vert as i32
        || ram_word!(game, samus_pose) as i32 == kPose_CD_FaceR_Shinespark_Diag as i32
        || ram_word!(game, samus_pose) as i32 == kPose_CE_FaceL_Shinespark_Diag as i32
    {
        let v2: i32 = j as i32 >> 1i32;
        let v3: i32 = ram_word!(game, plm_block_indices[v2]) as i32 >> 1i32;
        let v4: u16 = (ram_word!(game, level_data[v3]) as i32 & 0xf000i32 | 0x58i32) as u16;
        set_ram_word!(game.plm_variable[v2] = v4);
        set_ram_word!(game.level_data[v3] = (v4) & (0xfffu16));
        0u8
    } else {
        set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
        1u8
    }
}
pub unsafe fn PlmSetup_RespawningBombBlock2(j: u16, game: &mut Game) -> u8 {
    let mut v1: i16 = 0;
    v1 = ((*((&raw mut game.g_ram as *mut u8).offset(0xc18) as *mut u16)
        .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize)) as i16)
        & (0xf00i16);
    if v1 == 1280i16 {
        let v5: i32 = j as i32 >> 1i32;
        advance_plm_instruction(v5 as usize, 3, game);
        let v6: i32 = ram_word!(game, plm_block_indices[v5]) as i32 >> 1i32;
        let v7: u16 = (ram_word!(game, level_data[v6]) as i32 & 0xf000i32 | 0x58i32) as u16;
        set_ram_word!(game.plm_variable[v5] = v7);
        set_ram_word!(game.level_data[v6] = (v7) & (0x8fffu16));
    } else if v1 == 768i16 {
        let v2: i32 = j as i32 >> 1i32;
        let v3: i32 = ram_word!(game, plm_block_indices[v2]) as i32 >> 1i32;
        let v4: u16 = (ram_word!(game, level_data[v3]) as i32 & 0xf000i32 | 0x58i32) as u16;
        set_ram_word!(game.plm_variable[v2] = v4);
        set_ram_word!(game.level_data[v3] = (v4) & (0x8fffu16));
    } else {
        set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    }
    0u8
}
pub fn PlmSetup_RespawningPowerBombBlock(j: u16, game: &mut Game) -> u8 {
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    let projectile_index = usize::from(read_word(game, 0xdde) >> 1) * 2;
    let projectile_type = read_word(game, 0xc18 + projectile_index) & 0x0f00;
    let index = usize::from(j >> 1) * 2;
    match projectile_type {
        0x0500 => write_word(game, 0x1d27 + index, addr_kPlmInstrList_C91C as u16),
        0x0300 => {
            let block_index = usize::from(read_word(game, 0x1c87 + index) >> 1) * 2;
            let block = read_word(game, 0x10002 + block_index);
            let revealed = (block & 0xf000) | 0x0057;
            write_word(game, 0x1e17 + index, revealed);
            // Keep the original reveal mask and variable-before-level write order.
            write_word(game, 0x10002 + block_index, revealed & 0x8fff);
        }
        _ => write_word(game, 0x1c37 + index, 0),
    }
    0
}
pub unsafe fn PlmSetup_D08C_SuperMissileBlockRespawning(j: u16, game: &mut Game) -> u8 {
    let mut v1: i16 = 0;
    v1 = ((*((&raw mut game.g_ram as *mut u8).offset(0xc18) as *mut u16)
        .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize)) as i16)
        & (0xf00i16);
    if v1 == 1280i16 {
        set_ram_word!(
            game.plm_instr_list_ptrs[(j as i32 >> 1i32)] = addr_kPlmInstrList_C922 as u16
        );
    } else if v1 == 512i16 {
        let v2: i32 = j as i32 >> 1i32;
        let v3: i32 = ram_word!(game, plm_block_indices[v2]) as i32 >> 1i32;
        let v4: u16 = (ram_word!(game, level_data[v3]) as i32 & 0xf000i32 | 0x9fi32) as u16;
        set_ram_word!(game.plm_variable[v2] = v4);
        set_ram_word!(game.level_data[v3] = (v4) & (0x8fffu16));
    } else {
        set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    }
    0u8
}
pub unsafe fn PlmSetup_D08C_CrumbleBlock(j: u16, game: &mut Game) -> u8 {
    if (*((&raw mut game.g_ram as *mut u8).offset(0xc18) as *mut u16)
        .offset((ram_word!(game, projectile_index) as i32 >> 1i32) as isize) as i32
        & 0xf00i32)
        != 1280i32
    {
        set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    }
    0u8
}
pub unsafe fn PlmSetup_D0DC_BreakableGrappleBlock(j: u16, game: &mut Game) -> u8 {
    let v1: i32 = j as i32 >> 1i32;
    let v2: u16 = ram_word!(game, plm_block_indices[v1]);
    set_ram_word!(game.plm_variable[v1] = ram_word!(game, level_data[(v2 as i32 >> 1i32)]));
    *((&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset((v2 as i32 >> 1i32) as isize) as *mut u8 as *mut u16) =
        ((*(&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset(((v2 as i32 >> 1i32) + 1i32) as isize) as i32)
            << 8i32) as u16;
    0x41u8
}
pub fn PlmSetup_D0D8_SetVFlag(j: u16) -> u8 {
    0x41u8
}
pub fn PlmSetup_D0D8_ClearVflag(j: u16) -> u8 {
    1u8
}
pub unsafe fn PlmSetup_D0E8_GiveSamusDamage(j: u16, game: &mut Game) -> u8 {
    update_ram_word!(game.samus_periodic_damage, wrapping_add(1));
    0x41u8
}
pub unsafe fn PlmSetup_D113_LowerNorfairChozoRoomPlug(j: u16, game: &mut Game) -> u8 {
    let a: u16 = 0u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x10002) as *mut u16)
        .offset((ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]) as i32 >> 1i32) as isize) =
        (a) & (0xfffu16);
    0u8
}
pub unsafe fn PlmSetup_D127(j: u16, game: &mut Game) -> u8 {
    WriteLevelDataBlockTypeAndBts(
        ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]),
        0xc000u16,
        game,
    );
    0u8
}
pub unsafe fn PlmSetup_D138(j: u16, game: &mut Game) -> u8 {
    WriteLevelDataBlockTypeAndBts(
        ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]),
        0xe000u16,
        game,
    );
    0u8
}
pub unsafe fn PlmInstr_FxBaseYPos_0x2D2(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    set_ram_word!(game.fx_base_y_pos = 722u16);
    plmp
}
pub unsafe fn PlmPreInstr_DeletePlmAndSpawnTriggerIfBlockDestroyed(k: u16, game: &mut Game) {
    let prod: u16 = (8u16).wrapping_mul((ram_word!(game, room_width_in_blocks) as u8) as u16);
    let v1: u16 = (2i32 * (prod as i32 + 4i32)) as u16;
    if ram_word!(game, level_data[(v1 as i32 >> 1i32)]) == 255u16 {
        WriteLevelDataBlockTypeAndBts(v1, 0xb083u16, game);
        *((&raw mut game.g_ram as *mut u8).offset(0x1c37) as *mut u16)
            .offset((ram_word!(game, plm_id) as i32 >> 1i32) as isize) = 0u16;
    }
}
pub unsafe fn PlmSetup_D6DA_LowerNorfairChozoHandTrigger(
    j: u16,
    displacement: *mut i32,
    game: &mut Game,
) -> u8 {
    if (ram_word!(game, collected_items) as i32 & 0x200i32) != 0i32
        && (ram_word!(game, samus_collision_direction) as i32 & 0xfi32) == 3i32
        && (ram_word!(game, samus_pose) as i32 == kPose_1D_FaceR_Morphball_Ground as i32
            || ram_word!(game, samus_pose) as i32 == kPose_79_FaceR_Springball_Ground as i32
            || ram_word!(game, samus_pose) as i32 == kPose_7A_FaceL_Springball_Ground as i32)
    {
        SetEventHappened(0xcu16, game);
        (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0))
            .parameter_1 = 1u16;
        let v2: i32 = ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]) as i32 >> 1i32;
        update_ram_word!(game.level_data[v2], |fresh3| (fresh3) & (0xfffu16));
        CallSomeSamusCode(0u16, game);
        spawn_hardcoded_plm(
            SpawnHardcodedPlmArgs::new(0xcu8, 0x1du8, 0xd113 as VoidP),
            displacement,
            game,
        );
    }
    set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    1u8
}
pub unsafe fn PlmPreInstr_IncrementRoomArgIfShotBySuperMissile(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let v2: u16 = ram_word!(game, plm_timers[v1]);
    if (v2 as i32 & 0xf00i32) == 512i32 || (v2 as i32 & 0xf00i32) == 256i32 {
        update_ram_word!(game.plm_room_arguments[v1], wrapping_add(1));
    }
    set_ram_word!(game.plm_timers[v1] = 0u16);
}
pub unsafe fn PlmInstr_GotoIfRoomArgLess(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    if ram_word!(game, plm_room_arguments[(k as i32 >> 1i32)]) >= *(plmp as *mut u16) {
        plmp.offset(4)
    } else {
        *(plmp.offset(2) as *mut u16) as usize as *const u8
    }
}
pub unsafe fn PlmInstr_SpawnFourMotherBrainGlass(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    QueueSfx3_Max15(0x2eu16, game);
    SpawnMotherBrainGlassShatteringShard(*(plmp.offset(0) as *mut u16), game);
    SpawnMotherBrainGlassShatteringShard(*(plmp.offset(2) as *mut u16), game);
    SpawnMotherBrainGlassShatteringShard(*(plmp.offset(4) as *mut u16), game);
    SpawnMotherBrainGlassShatteringShard(*(plmp.offset(6) as *mut u16), game);
    plmp.offset(8)
}
pub unsafe fn SpawnMotherBrainGlassShatteringShard(a: u16, game: &mut Game) {
    SpawnEprojWithRoomGfx(0xcefcu16, a, game);
}
pub unsafe fn PlmPreInstr_WakePlmIfSamusHasBombs(k: u16, game: &mut Game) {
    if (ram_word!(game, collected_items) as i32 & 0x1000i32) != 0i32 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.plm_instruction_timer[v1] = 1u16);
        advance_plm_instruction(v1 as usize, 2, game);
        set_ram_word!(game.plm_pre_instrs[v1] = ((0x84d356i32) as u16));
    }
}
pub unsafe fn PlmInstr_SpawnTorizoStatueBreaking(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    SpawnEprojWithRoomGfx(0xa993u16, *(plmp as *mut u16), game);
    plmp.offset(2)
}
pub unsafe fn PlmInstr_QueueSong1MusicTrack(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    QueueMusic_Delayed8(6u16, game);
    plmp
}
pub unsafe fn PlmInstr_TransferWreckedShipChozoSpikesToSlopes(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    WriteLevelDataBlockTypeAndBts(0x1608u16, 0x1012u16, game);
    WriteLevelDataBlockTypeAndBts(0x160au16, 0x1013u16, game);
    plmp
}
pub unsafe fn PlmInstr_TransferWreckedShipSlopesToChozoSpikes(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    WriteLevelDataBlockTypeAndBts(0x1608u16, 0xa000u16, game);
    WriteLevelDataBlockTypeAndBts(0x160au16, 0xa000u16, game);
    plmp
}
pub unsafe fn PlmPreInstr_WakeOnKeyPress(k: u16, game: &mut Game) {
    if (ram_word!(game, joypad1_newkeys) as i32
        & (kButton_B as i32
            | kButton_Y as i32
            | kButton_Left as i32
            | kButton_Right as i32
            | kButton_A as i32
            | kButton_X as i32) as u16 as i32)
        != 0i32
    {
        let v2: i32 = k as i32 >> 1i32;
        set_ram_word!(game.plm_instruction_timer[v2] = 1u16);
        update_ram_word!(game.plm_instr_list_ptrs[v2], wrapping_add(1));
        update_ram_word!(game.plm_instr_list_ptrs[v2], wrapping_add(1));
    }
}
pub unsafe fn PlmInstr_EnableWaterPhysics(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    update_ram_word!(game.fx_liquid_options, |fresh34| (fresh34 as i32 & !(4i32))
        as u16);
    plmp
}
pub unsafe fn PlmInstr_SpawnN00bTubeCrackEproj(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    SpawnEprojWithRoomGfx(0xd904u16, 0u16, game);
    plmp
}
pub unsafe fn PlmInstr_DiagonalEarthquake(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    set_ram_word!(game.earthquake_type = 11u16);
    set_ram_word!(game.earthquake_timer = 64u16);
    plmp
}
pub unsafe fn PlmInstr_Spawn10shardsAnd6n00bs(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 0u16, game);
    SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 2u16, game);
    SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 4u16, game);
    SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 6u16, game);
    SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 8u16, game);
    SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 0xau16, game);
    SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 0xcu16, game);
    SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 0xeu16, game);
    SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 0x10u16, game);
    SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 0x12u16, game);
    SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeReleasedAirBubbles as u16, 0u16, game);
    SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeReleasedAirBubbles as u16, 2u16, game);
    SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeReleasedAirBubbles as u16, 4u16, game);
    SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeReleasedAirBubbles as u16, 6u16, game);
    SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeReleasedAirBubbles as u16, 8u16, game);
    SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeReleasedAirBubbles as u16, 0xau16, game);
    plmp
}
pub unsafe fn PlmInstr_DisableSamusControls(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    CallSomeSamusCode(0u16, game);
    plmp
}
pub unsafe fn PlmInstr_EnableSamusControls(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    CallSomeSamusCode(1u16, game);
    plmp
}
pub unsafe fn PlmSetup_MotherBrainGlass(j: u16, game: &mut Game) -> u8 {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.plm_room_arguments[v1] = 0u16);
    WriteLevelDataBlockTypeAndBts(ram_word!(game, plm_block_indices[v1]), 0x8044u16, game);
    0u8
}
pub unsafe fn PlmSetup_DeletePlmIfAreaTorizoDead(j: u16, game: &mut Game) -> u8 {
    if CheckBossBitForCurArea(4u16, game) as i32 & 1i32 != 0 {
        set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    }
    0u8
}
pub unsafe fn PlmSetup_MakeBllockChozoHandTrigger(j: u16, game: &mut Game) -> u8 {
    WriteLevelDataBlockTypeAndBts(
        ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]),
        0xb080u16,
        game,
    );
    0u8
}
pub unsafe fn PlmSetup_D6F2_WreckedShipChozoHandTrigger(
    j: u16,
    displacement: *mut i32,
    game: &mut Game,
) -> u8 {
    if CheckBossBitForCurArea(1u16, game) as i32 & 1i32 != 0
        && (ram_word!(game, samus_collision_direction) as i32 & 0xfi32) == 3i32
        && (ram_word!(game, samus_pose) as i32 == kPose_1D_FaceR_Morphball_Ground as i32
            || ram_word!(game, samus_pose) as i32 == kPose_79_FaceR_Springball_Ground as i32
            || ram_word!(game, samus_pose) as i32 == kPose_7A_FaceL_Springball_Ground as i32)
    {
        (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0))
            .parameter_1 = 1u16;
        *((&raw mut game.g_ram as *mut u8).offset(0xcd20).offset(7) as *mut u8 as *mut u16) =
            514u16;
        *((&raw mut game.g_ram as *mut u8).offset(0xcd20).offset(13) as *mut u8 as *mut u16) =
            257u16;
        let v1: i32 = ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]) as i32 >> 1i32;
        update_ram_word!(game.level_data[v1], |fresh2| (fresh2) & (0xfffu16));
        CallSomeSamusCode(0u16, game);
        spawn_hardcoded_plm(
            SpawnHardcodedPlmArgs::new(0x17u8, 0x1du8, 0xd6f8 as VoidP),
            displacement,
            game,
        );
    }
    set_ram_word!(game.plm_header_ptr[(j as i32 >> 1i32)] = 0u16);
    1u8
}
pub unsafe fn PlmSetup_D700_MakePlmAirBlock_Unused(j: u16, game: &mut Game) -> u8 {
    WriteLevelDataBlockTypeAndBts(
        ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]),
        0x44u16,
        game,
    );
    0u8
}
pub unsafe fn PlmSetup_D704_AlteranateLowerNorfairChozoHand_Unused(j: u16, game: &mut Game) -> u8 {
    WriteLevelDataBlockTypeAndBts(
        ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]),
        0x8044u16,
        game,
    );
    0u8
}
pub unsafe fn PlmSetup_D708_LowerNorfairChozoBlockUnused(j: u16, game: &mut Game) -> u8 {
    let v1: i32 = j as i32 >> 1i32;
    WriteLevelDataBlockTypeAndBts(ram_word!(game, plm_block_indices[v1]), 0x8044u16, game);
    WriteLevelDataBlockTypeAndBts(
        (ram_word!(game, plm_block_indices[v1])).wrapping_add(2u16),
        0x50ffu16,
        game,
    );
    WriteLevelDataBlockTypeAndBts(
        (ram_word!(game, room_width_in_blocks) as i32
            + ram_word!(game, room_width_in_blocks) as i32
            + ram_word!(game, plm_block_indices[v1]) as i32) as u16,
        0xd0ffu16,
        game,
    );
    WriteLevelDataBlockTypeAndBts(
        (ram_word!(game, room_width_in_blocks) as i32
            + ram_word!(game, room_width_in_blocks) as i32
            + ram_word!(game, plm_block_indices[v1]) as i32
            + 2i32) as u16,
        0xd0ffu16,
        game,
    );
    0u8
}
pub unsafe fn PlmSetup_D70C_NoobTube(j: u16, game: &mut Game) -> u8 {
    WriteLevelDataBlockTypeAndBts(
        ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]),
        0x8044u16,
        game,
    );
    0u8
}
pub unsafe fn PlmPreInstr_WakePlmIfRoomArgumentDoorIsSet(k: u16, game: &mut Game) {
    let v2: i32 = k as i32 >> 1i32;
    let idx: i32 = PrepareBitAccess(ram_word!(game, plm_room_arguments[v2]), game) as i32;
    if (ram_word!(game, bitmask) as i32
        & *(&raw mut game.g_ram as *mut u8)
            .offset(0xd8b0)
            .offset(idx as isize) as i32)
        != 0i32
    {
        let v3: i32 = k as i32 >> 1i32;
        set_ram_word!(game.plm_pre_instrs[v3] = addr_locret_84D779 as u16);
        set_ram_word!(
            game.plm_instr_list_ptrs[v3] = ram_word!(game, plm_instruction_list_link_reg[v3])
        );
        set_ram_word!(game.plm_instruction_timer[v3] = 1u16);
    }
}
pub unsafe fn PlmInstr_ShootEyeDoorProjectileWithProjectileArg(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_EyeDoorProjectile as u16,
        *(plmp as *mut u16),
        game,
    );
    QueueSfx2_Max6(0x4cu16, game);
    plmp.offset(2)
}
pub unsafe fn PlmInstr_SpawnEyeDoorSweatEproj(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    SpawnEprojWithRoomGfx(addr_kEproj_EyeDoorSweat as u16, *(plmp as *mut u16), game);
    plmp.offset(2)
}
pub unsafe fn PlmInstr_SpawnTwoEyeDoorSmoke(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    SpawnEprojWithRoomGfx(0xe517u16, 0x30au16, game);
    SpawnEprojWithRoomGfx(0xe517u16, 0x30au16, game);
    plmp
}
pub unsafe fn PlmInstr_SpawnEyeDoorSmokeProjectile(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    SpawnEprojWithRoomGfx(addr_kEproj_EyeDoorSmoke as u16, 0xbu16, game);
    plmp
}
pub unsafe fn PlmInstr_MoveUpAndMakeBlueDoorFacingRight(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    let v2: i32 = k as i32 >> 1i32;
    let blk: u16 = update_ram_word!(
        game.plm_block_indices[v2],
        |fresh33| (fresh33 as i32 - ram_word!(game, room_width_in_blocks) as i32 * 2i32) as u16,
        reread
    );
    WriteLevelDataBlockTypeAndBts(blk, 0xc041u16, game);
    sub_84D7EF(blk, game);
    plmp
}
pub unsafe fn PlmInstr_MoveUpAndMakeBlueDoorFacingLeft(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    let v2: i32 = k as i32 >> 1i32;
    let blk: u16 = update_ram_word!(
        game.plm_block_indices[v2],
        |fresh32| (fresh32 as i32 - ram_word!(game, room_width_in_blocks) as i32 * 2i32) as u16,
        reread
    );
    WriteLevelDataBlockTypeAndBts(blk, 0xc040u16, game);
    sub_84D7EF(blk, game);
    plmp
}
pub unsafe fn sub_84D7EF(k: u16, game: &mut Game) {
    let v1: u16 = (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + k as i32) as u16;
    WriteLevelDataBlockTypeAndBts(v1, 0xd0ffu16, game);
    let v2: u16 = (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + v1 as i32) as u16;
    WriteLevelDataBlockTypeAndBts(v2, 0xd0feu16, game);
    WriteLevelDataBlockTypeAndBts(
        (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + v2 as i32) as u16,
        0xd0fdu16,
        game,
    );
}
pub unsafe fn PlmSetup_EyeDoorEye(j: u16, game: &mut Game) -> u8 {
    let idx: i32 = PrepareBitAccess(
        ram_word!(game, plm_room_arguments[(j as i32 >> 1i32)]),
        game,
    ) as i32;
    if (ram_word!(game, bitmask) as i32
        & *(&raw mut game.g_ram as *mut u8)
            .offset(0xd8b0)
            .offset(idx as isize) as i32)
        == 0i32
    {
        let v1: i32 = j as i32 >> 1i32;
        WriteLevelDataBlockTypeAndBts(ram_word!(game, plm_block_indices[v1]), 0xc044u16, game);
        WriteLevelDataBlockTypeAndBts(
            (2i32 * ram_word!(game, room_width_in_blocks) as i32
                + ram_word!(game, plm_block_indices[v1]) as i32) as u16,
            0xd0ffu16,
            game,
        );
    }
    0u8
}
pub unsafe fn PlmSetup_EyeDoor(j: u16, game: &mut Game) -> u8 {
    let idx: i32 = PrepareBitAccess(
        ram_word!(game, plm_room_arguments[(j as i32 >> 1i32)]),
        game,
    ) as i32;
    if (ram_word!(game, bitmask) as i32
        & *(&raw mut game.g_ram as *mut u8)
            .offset(0xd8b0)
            .offset(idx as isize) as i32)
        == 0i32
    {
        WriteLevelDataBlockTypeAndBts(
            ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]),
            0xa000u16,
            game,
        );
    }
    0u8
}
pub unsafe fn PlmPreInstr_SetMetroidsClearState_Ev0x10(k: u16, game: &mut Game) {
    if ram_word!(game, num_enemies_killed_in_room)
        >= ram_word!(game, num_enemy_deaths_left_to_clear)
    {
        SetEventHappened(0x10u16, game);
    }
}
pub unsafe fn PlmPreInstr_SetMetroidsClearState_Ev0x11(k: u16, game: &mut Game) {
    if ram_word!(game, num_enemies_killed_in_room)
        >= ram_word!(game, num_enemy_deaths_left_to_clear)
    {
        SetEventHappened(0x11u16, game);
    }
}
pub unsafe fn PlmPreInstr_SetMetroidsClearState_Ev0x12(k: u16, game: &mut Game) {
    if ram_word!(game, num_enemies_killed_in_room)
        >= ram_word!(game, num_enemy_deaths_left_to_clear)
    {
        SetEventHappened(0x12u16, game);
    }
}
pub unsafe fn PlmPreInstr_SetMetroidsClearState_Ev0x13(k: u16, game: &mut Game) {
    if ram_word!(game, num_enemies_killed_in_room)
        >= ram_word!(game, num_enemy_deaths_left_to_clear)
    {
        SetEventHappened(0x13u16, game);
    }
}
pub unsafe fn PlmSetup_SetMetroidRequiredClearState(j: u16, game: &mut Game) -> u8 {
    set_ram_word!(
        game.plm_pre_instrs[(j as i32 >> 1i32)] = crate::rtl::rom_table_word!(
            game,
            0x84db28,
            (ram_word!(game, plm_room_arguments[(j as i32 >> 1i32)]) as i32 >> 1i32) as isize,
        )
    );
    0u8
}
pub unsafe fn PlmPreInstr_GotoLinkIfShotWithSuperMissile(k: u16, game: &mut Game) {
    let mut v1: i16 = 0;
    v1 = ((ram_word!(game, plm_timers[(k as i32 >> 1i32)])) as i16) & (0xf00i16);
    if v1 == 512i16 {
        set_ram_word!(game.plm_room_arguments[(k as i32 >> 1i32)] = 119u16);
    } else if v1 != 256i16 {
        return;
    }
    let v2: i32 = k as i32 >> 1i32;
    set_ram_word!(game.plm_timers[v2] = 0u16);
    set_ram_word!(
        game.plm_instr_list_ptrs[v2] = ram_word!(game, plm_instruction_list_link_reg[v2])
    );
    set_ram_word!(game.plm_instruction_timer[v2] = 1u16);
}
pub unsafe fn SetPlmVarPtr(k: u16, a: u16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut u8)
        .offset(ram_word!(game, plm_variable[(k as i32 >> 1i32)]) as isize) as *mut u8
        as *mut u16) = a;
}
pub unsafe fn PlmInstr_DamageDraygonTurret(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    SetPlmVarPtr(k, 1u16, game);
    let v2: u16 = ram_word!(game, plm_block_indices[(k as i32 >> 1i32)]);
    WriteLevelDataBlockTypeAndBts(v2, 0xa003u16, game);
    WriteLevelDataBlockTypeAndBts(
        (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + v2 as i32) as u16,
        0xa003u16,
        game,
    );
    plmp
}
pub unsafe fn PlmInstr_DamageDraygonTurretFacingDownRight(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    SetPlmVarPtr(k, 1u16, game);
    let v2: u16 = ram_word!(game, plm_block_indices[(k as i32 >> 1i32)]);
    WriteLevelDataBlockTypeAndBts(v2, 0xa003u16, game);
    WriteLevelDataBlockTypeAndBts((v2).wrapping_add(2u16), 0xa003u16, game);
    let v3: u16 = *((&raw mut game.g_ram as *mut u8).offset(0x1c87) as *mut u16)
        .offset((ram_word!(game, plm_id) as i32 >> 1i32) as isize);
    let v4: u16 = (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + v3 as i32) as u16;
    WriteLevelDataBlockTypeAndBts(v4, 0xa003u16, game);
    WriteLevelDataBlockTypeAndBts((v4).wrapping_add(2u16), 0u16, game);
    plmp
}
pub unsafe fn PlmInstr_DamageDraygonTurretFacingUpRight(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    SetPlmVarPtr(k, 1u16, game);
    let v2: u16 = ram_word!(game, plm_block_indices[(k as i32 >> 1i32)]);
    WriteLevelDataBlockTypeAndBts(v2, 0xa003u16, game);
    WriteLevelDataBlockTypeAndBts((v2).wrapping_add(2u16), 0u16, game);
    let v4: u16 = (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + v2 as i32) as u16;
    WriteLevelDataBlockTypeAndBts(v4, 0xa003u16, game);
    WriteLevelDataBlockTypeAndBts((v4).wrapping_add(2u16), 0xa003u16, game);
    plmp
}
pub unsafe fn PlmInstr_DamageDraygonTurret2(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    SetPlmVarPtr(k, 1u16, game);
    let v2: u16 = ram_word!(game, plm_block_indices[(k as i32 >> 1i32)]);
    WriteLevelDataBlockTypeAndBts(v2, 0xa003u16, game);
    WriteLevelDataBlockTypeAndBts(
        (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + v2 as i32) as u16,
        0xa003u16,
        game,
    );
    plmp
}
pub unsafe fn PlmInstr_DamageDraygonTurretFacingDownLeft(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    SetPlmVarPtr(k, 1u16, game);
    let v2: u16 = ram_word!(game, plm_block_indices[(k as i32 >> 1i32)]);
    WriteLevelDataBlockTypeAndBts(v2, 0xa003u16, game);
    WriteLevelDataBlockTypeAndBts((v2).wrapping_sub(2u16), 0xa003u16, game);
    let v4: u16 = (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + v2 as i32) as u16;
    WriteLevelDataBlockTypeAndBts(v4, 0xa003u16, game);
    WriteLevelDataBlockTypeAndBts((v4).wrapping_sub(2u16), 0u16, game);
    plmp
}
pub unsafe fn PlmInstr_DamageDraygonTurretFacingUpLeft(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    SetPlmVarPtr(k, 1u16, game);
    let v2: u16 = ram_word!(game, plm_block_indices[(k as i32 >> 1i32)]);
    WriteLevelDataBlockTypeAndBts(v2, 0xa003u16, game);
    WriteLevelDataBlockTypeAndBts((v2).wrapping_sub(2u16), 0u16, game);
    let v4: u16 = (v2 as i32 + ram_word!(game, room_width_in_blocks) as i32 * 2i32) as u16;
    WriteLevelDataBlockTypeAndBts(v4, 0xa003u16, game);
    WriteLevelDataBlockTypeAndBts((v4).wrapping_sub(2u16), 0xa003u16, game);
    plmp
}
pub unsafe fn PlmSetup_DraygonCannonFacingRight(j: u16, game: &mut Game) -> u8 {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.plm_variable[v1] = ram_word!(game, plm_room_arguments[v1]));
    set_ram_word!(game.plm_room_arguments[v1] = 0u16);
    let v2: u16 = ram_word!(game, plm_block_indices[v1]);
    WriteLevelDataBlockTypeAndBts(v2, 0xc044u16, game);
    WriteLevelDataBlockTypeAndBts(
        (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + v2 as i32) as u16,
        0xd0ffu16,
        game,
    );
    0u8
}
pub unsafe fn PlmSetup_DraygonCannonFacingDiagonalRight(j: u16, game: &mut Game) -> u8 {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.plm_variable[v1] = ram_word!(game, plm_room_arguments[v1]));
    set_ram_word!(game.plm_room_arguments[v1] = 0u16);
    let v2: u16 = ram_word!(game, plm_block_indices[v1]);
    WriteLevelDataBlockTypeAndBts(v2, 0xc044u16, game);
    WriteLevelDataBlockTypeAndBts((v2).wrapping_add(2u16), 0x50ffu16, game);
    let v3: u16 = (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + v2 as i32) as u16;
    WriteLevelDataBlockTypeAndBts(v3, 0xd0ffu16, game);
    WriteLevelDataBlockTypeAndBts((v3).wrapping_add(2u16), 0xd0ffu16, game);
    0u8
}
pub unsafe fn PlmSetup_DraygonCannonFacingLeft(j: u16, game: &mut Game) -> u8 {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.plm_variable[v1] = ram_word!(game, plm_room_arguments[v1]));
    set_ram_word!(game.plm_room_arguments[v1] = 0u16);
    let v2: u16 = ram_word!(game, plm_block_indices[v1]);
    WriteLevelDataBlockTypeAndBts(v2, 0xc044u16, game);
    WriteLevelDataBlockTypeAndBts(
        (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + v2 as i32) as u16,
        0xd0ffu16,
        game,
    );
    0u8
}
pub unsafe fn PlmSetup_DraygonCannonFacingDiagonalLeft(j: u16, game: &mut Game) -> u8 {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.plm_variable[v1] = ram_word!(game, plm_room_arguments[v1]));
    set_ram_word!(game.plm_room_arguments[v1] = 0u16);
    let v2: u16 = ram_word!(game, plm_block_indices[v1]);
    WriteLevelDataBlockTypeAndBts(v2, 0xc044u16, game);
    WriteLevelDataBlockTypeAndBts((v2).wrapping_sub(2u16), 0x5001u16, game);
    let v3: u16 = (ram_word!(game, room_width_in_blocks) as i32 * 2i32 + v2 as i32) as u16;
    WriteLevelDataBlockTypeAndBts(v3, 0xd0ffu16, game);
    WriteLevelDataBlockTypeAndBts((v3).wrapping_sub(2u16), 0xd0ffu16, game);
    0u8
}
pub unsafe fn PlmSetup_DraygonCannon(j: u16, game: &mut Game) -> u8 {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.plm_variable[v1] = ram_word!(game, plm_room_arguments[v1]));
    set_ram_word!(game.plm_room_arguments[v1] = 3u16);
    0u8
}
pub unsafe fn PlmPreInstr_GotoLinkIfTriggered(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if *(((&raw mut game.g_ram as *mut u8).offset(0x1d77) as *mut u16).offset(v1 as isize)
        as *mut u16 as *mut u8)
        .offset(0)
        == 255u8
    {
        set_ram_word!(game.plm_pre_instrs[v1] = ((0x84dfa8i32) as u16));
        set_ram_word!(
            game.plm_instr_list_ptrs[v1] = ram_word!(game, plm_instruction_list_link_reg[v1])
        );
        set_ram_word!(game.plm_instruction_timer[v1] = 1u16);
    }
}
pub unsafe fn PlmPreInstr_WakeIfTriggered(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let v2: u16 = ram_word!(game, plm_timers[v1]);
    if v2 != 768u16 && (v2 as u8) == 255u8 {
        set_ram_word!(game.plm_timers[v1] = 0u16);
        advance_plm_instruction(v1 as usize, 1, game);
        advance_plm_instruction(v1 as usize, 1, game);
        set_ram_word!(game.plm_instruction_timer[v1] = 1u16);
    }
}
pub unsafe fn PlmInstr_DrawItemFrame_Common(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    let v2: i32 = k as i32 >> 1i32;
    set_ram_word!(game.plm_instruction_timer[v2] = 4u16);
    set_ram_word!(
        game.plm_instr_list_ptrs[v2] = plmp.offset_from(RomBankBase(0x84u8, game)) as c_long as u16
    );
    ProcessPlmDrawInstruction(k, game);
    let v3: u16 = ram_word!(game, plm_id);
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    DrawPlm(v3, game);
    null::<u8>()
}
pub unsafe fn PlmInstr_DrawItemFrame0(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    set_ram_word!(
        game.plm_instruction_draw_ptr[(k as i32 >> 1i32)] = *(game
            .g_rom
            .offset(crate::rtl::lorom_offset(0x84e05f))
            as *const u8 as *mut u16)
            .offset((ram_word!(game, plm_variables[(k as i32 >> 1i32)]) as i32 >> 1i32) as isize,)
    );
    PlmInstr_DrawItemFrame_Common(plmp, k, game)
}
pub unsafe fn PlmInstr_DrawItemFrame1(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    set_ram_word!(
        game.plm_instruction_draw_ptr[(k as i32 >> 1i32)] = *(game
            .g_rom
            .offset(crate::rtl::lorom_offset(0x84e077))
            as *const u8 as *mut u16)
            .offset((ram_word!(game, plm_variables[(k as i32 >> 1i32)]) as i32 >> 1i32) as isize,)
    );
    PlmInstr_DrawItemFrame_Common(plmp, k, game)
}
pub unsafe fn PlmInstr_ClearChargeBeamCounter(
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8 {
    set_ram_word!(game.flare_counter = 0u16);
    plmp
}
pub unsafe fn PlmInstr_E63B(plmp: *const u8, k: u16, game: &mut Game) -> *const u8 {
    set_ram_word!(game.fx_y_vel = -32i32 as u16);
    plmp
}
pub unsafe fn sub_84EE4D(j: u16, game: &mut Game) -> u8 {
    sub_84EE5F(j, 8u16, game)
}
pub unsafe fn sub_84EE52(j: u16, game: &mut Game) -> u8 {
    sub_84EE5F(j, 0xau16, game)
}
pub unsafe fn sub_84EE57(j: u16, game: &mut Game) -> u8 {
    sub_84EE5F(j, 0xcu16, game)
}
pub unsafe fn sub_84EE5C(j: u16, game: &mut Game) -> u8 {
    sub_84EE5F(j, 0xeu16, game)
}
pub unsafe fn sub_84EE5F(j: u16, a: u16, game: &mut Game) -> u8 {
    set_ram_word!(game.plm_variables[(j as i32 >> 1i32)] = a);
    sub_84EE64(j, game)
}
pub unsafe fn sub_84EE64(j: u16, game: &mut Game) -> u8 {
    WriteLevelDataBlockTypeAndBts(
        ram_word!(game, plm_block_indices[(j as i32 >> 1i32)]),
        0x45u16,
        game,
    );
    update_ram_word!(game.global_number_of_items_loaded_ctr, wrapping_add(1));
    0u8
}
pub unsafe fn sub_84EE77(j: u16, game: &mut Game) -> u8 {
    sub_84EE89(j, 8u16, game)
}
pub unsafe fn sub_84EE7C(j: u16, game: &mut Game) -> u8 {
    sub_84EE89(j, 0xau16, game)
}
pub unsafe fn sub_84EE81(j: u16, game: &mut Game) -> u8 {
    sub_84EE89(j, 0xcu16, game)
}
pub unsafe fn sub_84EE86(j: u16, game: &mut Game) -> u8 {
    sub_84EE89(j, 0xeu16, game)
}
pub unsafe fn sub_84EE89(j: u16, a: u16, game: &mut Game) -> u8 {
    set_ram_word!(game.plm_variables[(j as i32 >> 1i32)] = a);
    sub_84EE8E(j, game)
}
pub unsafe fn sub_84EE8E(j: u16, game: &mut Game) -> u8 {
    let v1: i32 = j as i32 >> 1i32;
    WriteLevelDataBlockTypeAndBts(ram_word!(game, plm_block_indices[v1]), 0xc045u16, game);
    set_ram_word!(
        game.plm_variable[v1] = *((&raw mut game.g_ram as *mut u8).offset(0x10002) as *mut u16)
            .offset((ram_word!(game, plm_block_indices[v1]) as i32 >> 1i32) as isize,)
    );
    update_ram_word!(game.global_number_of_items_loaded_ctr, wrapping_add(1));
    0u8
}
pub unsafe fn sub_84EEAB(v0: u16, game: &mut Game) -> u8 {
    let mut i: i32 = 0;
    if ram_word!(game, time_is_frozen_flag) != 0 {
        set_ram_word!(game.plm_header_ptr[(v0 as i32 >> 1i32)] = 0u16);
    } else {
        let v1: i32 = v0 as i32 >> 1i32;
        let v2: u16 = ram_word!(game, plm_block_indices[v1]);
        set_ram_word!(game.plm_block_indices[v1] = 0u16);
        i = 78i32;
        while i >= 0i32 {
            if v2 == ram_word!(game, plm_block_indices[(i >> 1i32)]) {
                break;
            }
            i -= 2i32;
        }
        set_ram_word!(game.plm_timers[(i >> 1i32)] = 255u16);
    }
    0u8
}
/// Dispatch setup with the active collision displacement, or null outside collision.
/// `setup_a` preserves the spawner accumulator for setup routines that consume it.
///
/// # Safety
/// Legacy setup callbacks require initialized runtime state. A non-null
/// `displacement` must remain live and writable throughout the setup call.
pub unsafe fn CallPlmHeaderFunc(
    ea: u32,
    j: u16,
    displacement: *mut i32,
    setup_a: u16,
    game: &mut Game,
) -> u8 {
    match ea {
        8694567 | 8696783 | 8698095 | 8698618 | 8705782 | 8705783 | 8704398 => 0u8,
        8694568 => PlmSetup_CrumbleBotwoonWall(j, game),
        8695882 => PlmSetup_SetrupWreckedShipEntrance(j, game),
        8696028 => PlmSetup_BTS_Brinstar_0x80_Floorplant(j, game),
        8696083 => PlmSetup_BTS_Brinstar_0x81_Ceilingplant(j, game),
        8696203 => PlmSetup_B6D3_MapStation(j, game),
        8696264 => PlmSetup_Bts47_MapStationRightAccess(j, game),
        8696304 => PlmSetup_Bts4_MapStationLeftAccess(j, game),
        8696349 => PlmSetup_PlmB6DF_EnergyStation(j, game),
        8696389 => PlmSetup_PlmB6EB_EnergyStation(j, game),
        8696429 => PlmSetup_B6E3_EnergyStationRightAccess(j, game),
        8696477 => PlmSetup_B6E7_EnergyStationLeftAccess(j, game),
        8696528 => PlmSetup_B6EF_MissileStationRightAccess(j, game),
        8696576 => PlmSetup_B6F3_MissileStationLeftAccess(j, game),
        8696634 => PlmSetup_B638_Rightwards_Extension(j, game),
        8696645 => PlmSetup_B63F_Leftwards_Extension(j, game),
        8696656 => PlmSetup_B643_Downwards_Extension(j, game),
        8696667 => PlmSetup_B647_Upwards_Extension(j, game),
        8696689 => PlmSetup_B703_ScrollPLM(j, game),
        8696706 => PlmSetup_B707_SolidScrollPLM(j, game),
        8696723 => PlmSetup_B6FF_ScrollBlockTouch(j, game),
        8696769 => PlmSetup_DeactivatePlm(j, game),
        8696784 => PlmSetup_ReturnCarryClear(j),
        8696786 => PlmSetup_ReturnCarrySet(j),
        8696788 => PlmSetup_D094_EnemyBreakableBlock(j, game),
        8696803 => UNUSED_sub_84B3E3(j),
        8696811 => PlmSetup_B70F_IcePhysics(j, game),
        8696840 => PlmSetup_QuicksandSurface(j, game),
        8696983 => PlmSetup_B71F_SubmergingQuicksand(j, game),
        8697000 => PlmSetup_B723_SandfallsSlow(j, game),
        8697014 => PlmSetup_B727_SandFallsFast(j, game),
        8697028 => PlmSetup_QuicksandSurfaceB(j, displacement, game),
        8697153 => PlmSetup_B737_SubmergingQuicksand(j, game),
        8697167 => PlmSetup_B73B_B73F_SandFalls(j),
        8697169 => PlmSetup_ClearShitroidInvisibleWall(j, game),
        8697199 => PlmSetup_B767_ClearShitroidInvisibleWall(j, game),
        8697232 => PlmSetup_B76B_SaveStationTrigger(j, game),
        8697326 => PlmSetup_B76F_SaveStation(j, game),
        8697336 => PlmSetup_MotherBrainRoomEscapeDoor(j, game),
        8697795 => PlmSetup_B7EB_EnableSoundsIn32Frames(j, game),
        8698012 => PlmSetup_SpeedBoosterEscape(j, game),
        8698076 => PlmSetup_ShaktoolsRoom(j, game),
        8698220 => PlmSetup_B974(j),
        8698232 => PlmSetup_B9C1_CrittersEscapeBlock(j, game),
        8698309 => PlmSetup_B9ED_CrittersEscapeBlock(j, game),
        8698353 => sub_84B9F1(j, game),
        8698633 => PlmSetup_BB30_CrateriaMainstreetEscape(j, game),
        8701261 => PlmSetup_C806_LeftGreenGateTrigger(j, game),
        8701292 => PlmSetup_C80A_RightGreenGateTrigger(j, game),
        8701323 => PlmSetup_C80E_LeftRedGateTrigger(j, game),
        8701359 => PlmSetup_C812_RightRedGateTrigger(j, game),
        8701395 => PlmSetup_C81E_LeftYellowGateTrigger(j, game),
        8701425 => PlmSetup_C822_RightYellowGateTrigger(j, game),
        8701456 => PlmSetup_C816_LeftBlueGateTrigger(j, game),
        8701479 => PlmSetup_C81A_RightBlueGateTrigger(j, game),
        8701630 => PlmSetup_C82A_DownwardsClosedGate(j, setup_a, game),
        8701643 => PlmSetup_C832_UpwardsClosedGate(j, game),
        8701656 => PlmSetup_C826_DownwardsOpenGate(j, game),
        8701660 => PlmSetup_C82E_UpwardsOpenGate(j, game),
        8701664 => PlmSetup_C836_DownwardsGateShootblock(j, game),
        8701754 => PlmSetup_C73A_UpwardsGateShootblock(j, game),
        8701844 => PlmSetup_C794_GreyDoor(j, game),
        8701873 => PlmSetup_Door_Colored(j, game),
        8701883 => PlmSetup_Door_Blue(j, game),
        8701922 => PlmSetup_Door_Strange(j, game),
        8703426 => PlmSetup_D028_D02C_Unused(j, game),
        8703466 => PlmSetup_RespawningSpeedBoostBlock(j, game),
        8703543 => PlmSetup_RespawningCrumbleBlock(j, game),
        8703595 => PlmSetup_RespawningShotBlock(j, game),
        8703619 => PlmSetup_RespawningBombBlock(j, game),
        8703706 => PlmSetup_RespawningBombBlock2(j, game),
        8703790 => PlmSetup_RespawningPowerBombBlock(j, game),
        8703847 => PlmSetup_D08C_SuperMissileBlockRespawning(j, game),
        8703904 => PlmSetup_D08C_CrumbleBlock(j, game),
        8703925 => PlmSetup_D0DC_BreakableGrappleBlock(j, game),
        8703949 => PlmSetup_D0D8_SetVFlag(j),
        8703953 => PlmSetup_D0D8_ClearVflag(j),
        8703957 => PlmSetup_D0E8_GiveSamusDamage(j, game),
        8704264 => PlmSetup_D113_LowerNorfairChozoRoomPlug(j, game),
        8704279 => PlmSetup_D127(j, game),
        8704299 => PlmSetup_D138(j, game),
        8704399 => PlmSetup_D6DA_LowerNorfairChozoHandTrigger(j, displacement, game),
        8705526 => PlmSetup_MotherBrainGlass(j, game),
        8705542 => PlmSetup_DeletePlmIfAreaTorizoDead(j, game),
        8705558 => PlmSetup_MakeBllockChozoHandTrigger(j, game),
        8705568 => PlmSetup_D6F2_WreckedShipChozoHandTrigger(j, displacement, game),
        8705663 => PlmSetup_D700_MakePlmAirBlock_Unused(j, game),
        8705673 => PlmSetup_D704_AlteranateLowerNorfairChozoHand_Unused(j, game),
        8705683 => PlmSetup_D708_LowerNorfairChozoBlockUnused(j, game),
        8705740 => PlmSetup_D70C_NoobTube(j, game),
        8706700 => PlmSetup_EyeDoorEye(j, game),
        8706745 => PlmSetup_EyeDoor(j, game),
        8706846 => PlmSetup_SetMetroidRequiredClearState(j, game),
        8707732 => PlmSetup_DraygonCannonFacingRight(j, game),
        8707769 => PlmSetup_DraygonCannonFacingDiagonalRight(j, game),
        8707824 => PlmSetup_DraygonCannonFacingLeft(j, game),
        8707861 => PlmSetup_DraygonCannonFacingDiagonalLeft(j, game),
        8707916 => PlmSetup_DraygonCannon(j, game),
        8711757 => sub_84EE4D(j, game),
        8711762 => sub_84EE52(j, game),
        8711767 => sub_84EE57(j, game),
        8711772 => sub_84EE5C(j, game),
        8711780 => sub_84EE64(j, game),
        8711799 => sub_84EE77(j, game),
        8711804 => sub_84EE7C(j, game),
        8711809 => sub_84EE81(j, game),
        8711814 => sub_84EE86(j, game),
        8711822 => sub_84EE8E(j, game),
        8711851 => sub_84EEAB(j, game),
        _ => Unreachable(game) as u8,
    }
}
pub unsafe fn CallPlmPreInstr(ea: u32, k: u16, game: &mut Game) {
    match ea {
        8684649 => (),
        8708008 => (),
        8685264 => (),
        8684774 => (),
        8684861 => (),
        8686246 => (),
        8698787 => (),
        8704854 => (),
        8698618 => (),
        8696028 => {
            PlmSetup_BTS_Brinstar_0x80_Floorplant(k, game);
        }
        8696083 => {
            PlmSetup_BTS_Brinstar_0x81_Ceilingplant(k, game);
        }
        8697821 => PlmPreInstr_B7EB_DecTimerEnableSoundsDeletePlm(k, game),
        8697839 => PlmPreInstr_WakeAndLavaIfBoosterCollected(k, game),
        8697898 => PlmPreInstr_WakePLMAndStartFxMotionSamusFarLeft(k, game),
        8697926 => PlmPreInstr_AdvanceLavaSamusMovesLeft(k, game),
        8698032 => PlmPreInstr_ShaktoolsRoom(k, game),
        8698151 => PlmPreInstr_OldTourianEscapeShaftEscape(k, game),
        8698184 => PlmPreInstr_EscapeRoomBeforeOldTourianEscapeShaft(k, game),
        8698706 => PlmPreInstr_WakePlmIfTriggered(k, game),
        8698731 => PlmPreInstr_WakePlmIfTriggeredOrSamusBelowPlm(k, game),
        8698788 => PlmPreInstr_WakePlmIfTriggeredOrSamusAbovePlm(k, game),
        8704348 => PlmPreInstr_DeletePlmAndSpawnTriggerIfBlockDestroyed(k, game),
        8704486 => PlmPreInstr_IncrementRoomArgIfShotBySuperMissile(k, game),
        8704827 => PlmPreInstr_WakePlmIfSamusHasBombs(k, game),
        8705875 => PlmPreInstr_WakePlmIfRoomArgumentDoorIsSet(k, game),
        8706916 => PlmPreInstr_GotoLinkIfShotWithSuperMissile(k, game),
        8707977 => PlmPreInstr_GotoLinkIfTriggered(k, game),
        8708070 => PlmPreInstr_WakeIfTriggered(k, game),
        8699151 => PlmPreInstr_GoToLinkInstrIfShot(k, game),
        8699174 => PlmPreInstr_GoToLinkInstrIfShotWithPowerBomb(k, game),
        8699216 => PlmPreInstr_GoToLinkInstrIfShotWithAnyMissile(k, game),
        8699272 => PlmPreInstr_GoToLinkInstrIfShotWithSuperMissile(k, game),
        8699314 => PlmPreInstr_GoToLinkInstruction(k, game),
        8699420 => PlmPreInstr_PlayDudSound(k, game),
        8699348 => PlmPreInstr_GotoLinkIfBoss1Dead(k, game),
        8699363 => PlmPreInstr_GotoLinkIfMiniBossDead(k, game),
        8699378 => PlmPreInstr_GotoLinkIfTorizoDead(k, game),
        8699393 => PlmPreInstr_GotoLinkIfEnemyDeathQuotaOk(k, game),
        8699423 => PlmPreInstr_GotoLinkIfTourianStatueFinishedProcessing(k, game),
        8699440 => PlmPreInstr_GotoLinkIfCrittersEscaped(k, game),
        8694921 => PlmPreInstr_PositionSamusAndInvincible(k, game),
        8705215 => PlmPreInstr_WakeOnKeyPress(k, game),
        8698730 => (),
        8706773 | 8706774 | 8706775 | 8706776 | 8706777 | 8706778 | 8706779 | 8706780 | 8706781
        | 8686304 => (),
        8706782 => PlmPreInstr_SetMetroidsClearState_Ev0x10(k, game),
        8706798 => PlmPreInstr_SetMetroidsClearState_Ev0x11(k, game),
        8706814 => PlmPreInstr_SetMetroidsClearState_Ev0x12(k, game),
        8706830 => PlmPreInstr_SetMetroidsClearState_Ev0x13(k, game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CallPlmInstr(ea: u32, j: *const u8, k: u16, game: &mut Game) -> *const u8 {
    match ea {
        8685236 => PlmInstr_Sleep(j, k, game),
        8685244 => PlmInstr_Delete(j, k, game),
        8685249 => PlmInstr_PreInstr(j, k, game),
        8685258 => PlmInstr_ClearPreInstr(j, k, game),
        8685323 => PlmInstr_CallFunction(j, k, game),
        8685348 => PlmInstr_Goto(j, k),
        8685375 => PlmInstr_DecrementAndBranchNonzero(j, k, game),
        8685390 => PlmInstr_SetTimer(j, k, game),
        8685412 => PlmInstr_LoadItemPlmGfx(j, k, game),
        8685541 => PlmInstr_CopyFromRamToVram(j, k, game),
        8685582 => PlmInstr_GotoIfBossBitSet(j, k, game),
        8685613 => PlmInstr_GotoIfEventSet(j, k, game),
        8685630 => PlmInstr_SetEvent(j, k, game),
        8685640 => PlmInstr_GotoIfChozoSet(j, k, game),
        8685669 => PlmInstr_SetRoomChozoBit(j, k, game),
        8685692 => PlmInstr_GotoIfItemBitSet(j, k, game),
        8685721 => PlmInstr_SetItemBit(j, k, game),
        8685744 => PlmInstr_PickupBeamAndShowMessage(j, k, game),
        8685811 => PlmInstr_PickupEquipmentAndShowMessage(j, k, game),
        8685850 => PlmInstr_PickupEquipmentAddGrappleShowMessage(j, k, game),
        8685889 => PlmInstr_PickupEquipmentAddXrayShowMessage(j, k, game),
        8685928 => PlmInstr_CollectHealthEnergyTank(j, k, game),
        8685958 => PlmInstr_CollectHealthReserveTank(j, k, game),
        8685993 => PlmInstr_CollectAmmoMissileTank(j, k, game),
        8686034 => PlmInstr_CollectAmmoSuperMissileTank(j, k, game),
        8686075 => PlmInstr_CollectAmmoPowerBombTank(j, k, game),
        8686116 => PlmInstr_SetLinkReg(j, k, game),
        8686126 => PlmInstr_Call(j, k, game),
        8686138 => PlmInstr_Return(j, k, game),
        8686194 => PlmInstr_GotoIfDoorBitSet(j, k, game),
        8686225 => PlmInstr_IncrementDoorHitCounterAndJGE(j, k, game),
        8686285 => PlmInstr_IncrementArgumentAndJGE(j, k, game),
        8686321 => PlmInstr_SetBTS(j, k, game),
        8686341 => PlmInstr_DrawPlmBlock(j, k, game),
        8686359 => PlmInstr_DrawPlmBlock(j, k, game),
        8686421 => PlmInstr_ProcessAirScrollUpdate(j, k, game),
        8686483 => PlmInstr_ProcessSolidScrollUpdate(j, k, game),
        8686545 => PlmInstr_QueueMusic(j, k, game),
        8686557 => PlmInstr_ClearMusicQueueAndQueueTrack(j, k, game),
        8686599 => PlmInstr_QueueSfx1_Max6(j, k, game),
        8686608 => PlmInstr_QueueSfx2_Max6(j, k, game),
        8686617 => PlmInstr_QueueSfx3_Max6(j, k, game),
        8686626 => PlmInstr_QueueSfx1_Max15(j, k, game),
        8686635 => PlmInstr_QueueSfx2_Max15(j, k, game),
        8686644 => PlmInstr_QueueSfx3_Max15(j, k, game),
        8686653 => PlmInstr_QueueSfx1_Max3(j, k, game),
        8686662 => PlmInstr_QueueSfx2_Max3(j, k, game),
        8686671 => PlmInstr_QueueSfx_Max3(j, k, game),
        8686680 => PlmInstr_QueueSfx1_Max9(j, k, game),
        8686689 => PlmInstr_QueueSfx2_Max9(j, k, game),
        8686698 => PlmInstr_QueueSfx3_Max9(j, k, game),
        8686707 => PlmInstr_QueueSfx1_Max1(j, k, game),
        8686716 => PlmInstr_QueueSfx2_Max1(j, k, game),
        8686725 => PlmInstr_QueueSfx3_Max1(j, k, game),
        8686735 => PlmInstr_ActivateMapStation(j, k, game),
        8686767 => PlmInstr_ActivateEnergyStation(j, k, game),
        8686800 => PlmInstr_ActivateMissileStation(j, k, game),
        8686833 => PlmInstr_ActivateSaveStationAndGotoIfNo(j, k, game),
        8686913 => PlmInstr_GotoIfSamusNear(j, k, game),
        8694528 => PlmInstr_MovePlmDownOneBlock(j, k, game),
        8694609 => PlmInstr_Scroll_0_1_Blue(j, k, game),
        8694617 => PlmInstr_MovePlmDownOneBlock_0(j, k, game),
        8694941 => PlmInstr_DealDamage_2(j, k, game),
        8694961 => PlmInstr_GiveInvincibility(j, k, game),
        8695107 => PlmInstr_Draw0x38FramesOfRightTreadmill(j, k, game),
        8695128 => PlmInstr_Draw0x38FramesOfLeftTreadmill(j, k, game),
        8695349 => PlmInstr_GotoIfSamusHealthFull(j, k, game),
        8695487 => PlmInstr_GotoIfMissilesFull(j, k, game),
        8695822 => PlmInstr_PlaceSamusOnSaveStation(j, k, game),
        8695844 => PlmInstr_DisplayGameSavedMessageBox(j, k, game),
        8695856 => PlmInstr_EnableMovementAndSetSaveStationUsed(j, k, game),
        8698297 => PlmInstr_SetCrittersEscapedEvent(j, k, game),
        8698479 => PlmInstr_JumpIfSamusHasNoBombs(j, k, game),
        8698661 => PlmInstr_MovePlmRight4Blocks(j, k, game),
        8698845 => PlmInstr_ClearTrigger(j, k, game),
        8698849 => PlmInstr_SpawnEproj(j, k, game),
        8698864 => PlmInstr_WakeEprojAtPlmPos(j, k, game),
        8699455 => PlmInstr_SetGreyDoorPreInstr(j, k, game),
        8704341 => PlmInstr_FxBaseYPos_0x2D2(j, k, game),
        8704761 => PlmInstr_GotoIfRoomArgLess(j, k, game),
        8704779 => PlmInstr_SpawnFourMotherBrainGlass(j, k, game),
        8704855 => PlmInstr_SpawnTorizoStatueBreaking(j, k, game),
        8704967 => PlmInstr_QueueSong1MusicTrack(j, k, game),
        8704983 => PlmInstr_TransferWreckedShipChozoSpikesToSlopes(j, k, game),
        8705012 => PlmInstr_TransferWreckedShipSlopesToChozoSpikes(j, k, game),
        8705317 => PlmInstr_EnableWaterPhysics(j, k, game),
        8705324 => PlmInstr_SpawnN00bTubeCrackEproj(j, k, game),
        8705334 => PlmInstr_DiagonalEarthquake(j, k, game),
        8705347 => PlmInstr_Spawn10shardsAnd6n00bs(j, k, game),
        8705914 => PlmInstr_ShootEyeDoorProjectileWithProjectileArg(j, k, game),
        8705936 => PlmInstr_SpawnEyeDoorSweatEproj(j, k, game),
        8705951 => PlmInstr_SpawnTwoEyeDoorSmoke(j, k, game),
        8705974 => PlmInstr_SpawnEyeDoorSmokeProjectile(j, k, game),
        8705987 => PlmInstr_MoveUpAndMakeBlueDoorFacingRight(j, k, game),
        8706010 => PlmInstr_MoveUpAndMakeBlueDoorFacingLeft(j, k, game),
        8706958 => PlmInstr_DamageDraygonTurret(j, k, game),
        8707000 => PlmInstr_DamageDraygonTurretFacingDownRight(j, k, game),
        8707063 => PlmInstr_DamageDraygonTurretFacingUpRight(j, k, game),
        8707126 => PlmInstr_DamageDraygonTurret2(j, k, game),
        8707168 => PlmInstr_DamageDraygonTurretFacingDownLeft(j, k, game),
        8707231 => PlmInstr_DamageDraygonTurretFacingUpLeft(j, k, game),
        8708175 => PlmInstr_DrawItemFrame0(j, k, game),
        8708199 => PlmInstr_DrawItemFrame1(j, k, game),
        8708223 => PlmInstr_DrawItemFrame_Common(j, k, game),
        8708765 => PlmInstr_ClearChargeBeamCounter(j, k, game),
        8694742 => PlmInstr_ABD6(j, k, game),
        8709691 => PlmInstr_E63B(j, k, game),
        8703379 => PlmInstr_SetBtsTo1(j, k, game),
        8705510 => PlmInstr_DisableSamusControls(j, k, game),
        8705518 => PlmInstr_EnableSamusControls(j, k, game),
        _ => {
            Unreachable(game);
            null::<u8>()
        }
    }
}
