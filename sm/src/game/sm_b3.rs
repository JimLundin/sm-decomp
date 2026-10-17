use crate::game::sm_80::{
    CheckEventHappened, NextRandom, QueueMusic_Delayed8, QueueSfx2_Max6, SetBossBitForCurArea,
};
use crate::game::sm_84::SpawnHardcodedPlm;
use crate::game::sm_86::{eproj_spawn_pt, SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::sm_a0::{
    CalculateAngleFromXY, CalculateAngleOfSamusFromEnemy, CallEnemyPreInstr,
    CheckIfEnemyIsOnScreen, ConvertAngleToXy, EnemyFunc_B691, EnemyGrappleDeath,
    Enemy_ItemDrop_Botwoon, Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes, Enemy_SwitchToFrozenAi,
    GetSamusEnemyDelta_X, GetSamusEnemyDelta_Y, IsEnemyLeavingScreen, IsSamusWithinEnemy_X,
    IsSamusWithinEnemy_Y, NormalEnemyFrozenAI, NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy,
    NormalEnemyShotAi, NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAi,
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy, SignExtend8, SwitchEnemyAiToMainAi,
};
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::rtl::{g_ram, g_rom, RomPtr, Unreachable};

pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t,
    EnemyBase, EnemyData, EnemyProps, Point16U, VoidP,
};
pub const kEnemyProps_RespawnIfKilled: EnemyProps = 16384;
pub const kEnemyProps_DisableSamusColl: EnemyProps = 8192;
pub const kEnemyProps_BlockPlasmaBeam: EnemyProps = 4096;
pub const kEnemyProps_ProcessedOffscreen: EnemyProps = 2048;
pub const kEnemyProps_Tangible: EnemyProps = 1024;
pub const kEnemyProps_Deleted: EnemyProps = 512;
pub const kEnemyProps_Invisible: EnemyProps = 256;
pub use crate::types::Consts_86;
pub const addr_kEproj_Sparks: Consts_86 = 62616;
pub const addr_kEproj_EnemyDeathExplosion: Consts_86 = 62277;
pub const addr_kEproj_Pickup: Consts_86 = 62263;
pub const addr_word_86ECA3: Consts_86 = 60579;
pub const addr_kEproj_YappingMawsBody: Consts_86 = 60565;
pub const addr_word_86EC5C: Consts_86 = 60508;
pub const addr_word_86EC56: Consts_86 = 60502;
pub const addr_kEproj_BotwoonsSpit: Consts_86 = 60488;
pub const addr_word_86EBAE: Consts_86 = 60334;
pub const addr_kEproj_BotwoonsBody: Consts_86 = 60320;
pub const addr_kEproj_SaveStationElectricity: Consts_86 = 59090;
pub const addr_kEproj_EyeDoorSmoke: Consts_86 = 58647;
pub const addr_kEproj_DustCloudExplosion: Consts_86 = 58633;
pub const addr_word_86E208: Consts_86 = 57864;
pub const addr_kEproj_LavaThrownByLavaman: Consts_86 = 57568;
pub const addr_word_86DFDE: Consts_86 = 57310;
pub const addr_word_86DFD8: Consts_86 = 57304;
pub const addr_stru_86DFCA: Consts_86 = 57290;
pub const addr_stru_86DFBC: Consts_86 = 57276;
pub const addr_word_86DEA6: Consts_86 = 56998;
pub const addr_word_86DE96: Consts_86 = 56982;
pub const addr_kEproj_SporeSpawners: Consts_86 = 56968;
pub const addr_kEproj_Spores: Consts_86 = 56954;
pub const addr_kEproj_SporeSpawnsStalk: Consts_86 = 56940;
pub const addr_word_86DC06: Consts_86 = 56326;
pub const addr_Eproj_DBF2: Consts_86 = 56306;
pub const addr_word_86DB0C: Consts_86 = 56076;
pub const addr_kEproj_SpikeShootingPlantSpikes: Consts_86 = 56062;
pub const addr_kEproj_N00bTubeReleasedAirBubbles: Consts_86 = 55584;
pub const addr_kEproj_N00bTubeShards: Consts_86 = 55570;
pub const addr_kEproj_WreckedShipRobotLaserDownRight: Consts_86 = 53982;
pub const addr_kEproj_WreckedShipRobotLaserUpRight: Consts_86 = 53968;
pub const addr_kEproj_WreckedShipRobotLaserDownLeft: Consts_86 = 53954;
pub const addr_kEproj_WreckedShipRobotLaserHorizontal: Consts_86 = 53940;
pub const addr_kEproj_WreckedShipRobotLaserUpLeft: Consts_86 = 53926;
pub const addr_kEproj_MaridiaFloatersSpikes: Consts_86 = 53912;
pub const addr_off_86D218: Consts_86 = 53784;
pub const addr_word_86D052: Consts_86 = 53330;
pub const addr_word_86D04A: Consts_86 = 53322;
pub const addr_word_86D03C: Consts_86 = 53308;
pub const addr_off_86CF56: Consts_86 = 53078;
pub const addr_kEproj_MotherBrainGlassShatteringSparkle: Consts_86 = 53002;
pub const addr_kEproj_MotherBrainTubeFalling_TopMiddleRight: Consts_86 = 52357;
pub const addr_kEproj_MotherBrainTubeFalling_TopMiddleLeft: Consts_86 = 52343;
pub const addr_kEproj_MotherBrainTubeFalling_TopLeft: Consts_86 = 52329;
pub const addr_kEproj_TimeBombSetJapaneseText: Consts_86 = 52155;
pub const addr_kEproj_MotherBrainRainbowBeamExplosion: Consts_86 = 52141;
pub const addr_kEproj_MotherBrainDyingDrool: Consts_86 = 52127;
pub const addr_kEproj_MotherBrainDrool: Consts_86 = 52113;
pub const addr_kEproj_MotherBrainRainbowBeamCharging: Consts_86 = 52099;
pub const addr_kEproj_MotherBrainDeathBeamFired: Consts_86 = 52085;
pub const addr_kEproj_MotherBrainDeathBeamCharging: Consts_86 = 52071;
pub const addr_kEproj_MotherBrainBomb: Consts_86 = 52057;
pub const addr_kEproj_MotherBrainBlueRingLasers: Consts_86 = 52043;
pub const addr_kEproj_MotherBrainPurpleBreathSmall: Consts_86 = 52029;
pub const addr_kEproj_MotherBrainPurpleBreathBig: Consts_86 = 52015;
pub const addr_kEproj_MotherBrainExplodedEscapeDoor: Consts_86 = 52001;
pub const addr_kEproj_MotherBrainDeathExplosion: Consts_86 = 51987;
pub const addr_off_86C8E1: Consts_86 = 51425;
pub const addr_kEproj_MotherBrainRoomTurretBullets: Consts_86 = 49548;
pub const addr_kEproj_MotherBrainRoomTurrets: Consts_86 = 49534;
pub const addr_kEproj_ShaktoolAttackBackCircle: Consts_86 = 48705;
pub const addr_kEproj_ShaktoolAttackMiddleCircle: Consts_86 = 48691;
pub const addr_kEproj_ShaktoolAttackFrontCircle: Consts_86 = 48677;
pub const addr_stru_86BD5A: Consts_86 = 48474;
pub const addr_word_86BBD5: Consts_86 = 48085;
pub const addr_kEproj_NuclearWaffleBody: Consts_86 = 48071;
pub const addr_kEproj_TourianStatueBaseDecoration: Consts_86 = 47806;
pub const addr_kEproj_TourianStatuePhantoon: Consts_86 = 47792;
pub const addr_kEproj_TourianStatueRidley: Consts_86 = 47778;
pub const addr_kEproj_TourianStatueSoul: Consts_86 = 47764;
pub const addr_kEproj_TourianStatueUnlockingParticleTail: Consts_86 = 47750;
pub const addr_kEproj_TourianStatueUnlockingParticle: Consts_86 = 47736;
pub const addr_kEproj_TourianStatueEyeGlow: Consts_86 = 47722;
pub const addr_stru_86BA5C: Consts_86 = 47708;
pub const addr_off_86B79F: Consts_86 = 47007;
pub const addr_kEproj_EyeDoorSweat: Consts_86 = 46929;
pub const addr_kEproj_EyeDoorProjectile: Consts_86 = 46915;
pub const addr_off_86B61D: Consts_86 = 46621;
pub const addr_off_86B5F3: Consts_86 = 46579;
pub const addr_kEproj_LavaSeahorseFireball: Consts_86 = 46539;
pub const addr_word_86B4E3: Consts_86 = 46307;
pub const addr_word_86B4D7: Consts_86 = 46295;
pub const addr_word_86B4CB: Consts_86 = 46283;
pub const addr_word_86B4BF: Consts_86 = 46271;
pub const addr_kEproj_GoldenTorizoEyeBeam: Consts_86 = 46120;
pub const addr_off_86B3E5: Consts_86 = 46053;
pub const addr_off_86B3CD: Consts_86 = 46029;
pub const addr_stru_86B380: Consts_86 = 45952;
pub const addr_stru_86B376: Consts_86 = 45942;
pub const addr_kEproj_GoldenTorizoSuperMissile: Consts_86 = 45850;
pub const addr_off_86B2EF: Consts_86 = 45807;
pub const addr_stru_86B1C0: Consts_86 = 45504;
pub const addr_off_86B1A8: Consts_86 = 45480;
pub const addr_off_86B190: Consts_86 = 45456;
pub const addr_off_86B166: Consts_86 = 45414;
pub const addr_off_86B14B: Consts_86 = 45387;
pub const addr_kEprojInit_GoldenTorizoEgg1: Consts_86 = 45113;
pub const addr_kEprojInit_GoldenTorizoEgg0: Consts_86 = 45103;
pub const addr_stru_86AFF3: Consts_86 = 45043;
pub const addr_kEproj_TourianLandingDustCloudsRightFoot: Consts_86 = 45029;
pub const addr_kEproj_TourianStatueDustClouds: Consts_86 = 44932;
pub const addr_kEproj_WreckedShipChozoSpikeFootsteps_1: Consts_86 = 44904;
pub const addr_kEproj_GoldenTorizoSonicBoom: Consts_86 = 44726;
pub const addr_kEproj_BombTorizoSonicBoom: Consts_86 = 44712;
pub const addr_off_86ADE5: Consts_86 = 44517;
pub const addr_off_86ADD2: Consts_86 = 44498;
pub const addr_off_86ADBF: Consts_86 = 44479;
pub const addr_kEproj_GoldenTorizosChozoOrbs: Consts_86 = 44410;
pub const addr_kEproj_BombTorizosChozoOrbs: Consts_86 = 44382;
pub const addr_kEprojInit_GoldenTorizosChozoOrbs_init1: Consts_86 = 44195;
pub const addr_kEprojInit_GoldenTorizosChozoOrbs_init0: Consts_86 = 44185;
pub const addr_kEprojInit_BombTorizosChozoOrbs_init1: Consts_86 = 44050;
pub const addr_kEprojInit_BombTorizosChozoOrbs_init0: Consts_86 = 44040;
pub const addr_off_86AB41: Consts_86 = 43841;
pub const addr_off_86AB25: Consts_86 = 43813;
pub const addr_kEprojInit_AB07_Tile1: Consts_86 = 43517;
pub const addr_kEprojInit_AB07_Tile0: Consts_86 = 43453;
pub const addr_kEproj_BombTorizoDeathExplosion: Consts_86 = 43439;
pub const addr_kEproj_BombTorizoLowHealthExplode: Consts_86 = 43425;
pub const addr_kEproj_BombTorizoExplosiveSwipe: Consts_86 = 43397;
pub const addr_kEproj_BombTorizoLowHealthInitialDrool: Consts_86 = 43369;
pub const addr_stru_86A95B: Consts_86 = 43355;
pub const addr_kEproj_PrePhantomRoom: Consts_86 = 41904;
pub const addr_kEproj_CeresElevatorPlatform: Consts_86 = 41877;
pub const addr_kEproj_CeresElevatorPad: Consts_86 = 41863;
pub const addr_stru_86A379: Consts_86 = 41849;
pub const addr_off_86A28B: Consts_86 = 41611;
pub const addr_kEproj_PirateClaw: Consts_86 = 41353;
pub const addr_stru_86A17B: Consts_86 = 41339;
pub const addr_off_869FE1: Consts_86 = 40929;
pub const addr_off_869FB9: Consts_86 = 40889;
pub const addr_word_869F7D: Consts_86 = 40829;
pub const addr_word_869F41: Consts_86 = 40769;
pub const addr_kEproj_MiniKraidSpikesRight: Consts_86 = 40396;
pub const addr_kEproj_MiniKraidSpikesLeft: Consts_86 = 40382;
pub const addr_kEproj_MiniKraidSpit: Consts_86 = 40368;
pub const addr_kEproj_RocksWhenKraidRisesRight: Consts_86 = 40047;
pub const addr_kEproj_RocksWhenKraidRisesLeft: Consts_86 = 40033;
pub const addr_kEproj_RocksFallingKraidCeiling: Consts_86 = 40019;
pub const addr_kEproj_RocksKraidSpits: Consts_86 = 40005;
pub const addr_kEproj_StartingFireballs: Consts_86 = 39991;
pub const addr_kEproj_DestroyableFireballs: Consts_86 = 39977;
pub const addr_off_8697F8: Consts_86 = 38904;
pub const addr_off_8697B4: Consts_86 = 38836;
pub const addr_word_8697AC: Consts_86 = 38828;
pub const addr_word_869782: Consts_86 = 38786;
pub const addr_word_869772: Consts_86 = 38770;
pub const addr_word_86976C: Consts_86 = 38764;
pub const addr_stru_869742: Consts_86 = 38722;
pub const addr_stru_869734: Consts_86 = 38708;
pub const addr_stru_869696: Consts_86 = 38550;
pub const addr_stru_869688: Consts_86 = 38536;
pub const addr_stru_86967A: Consts_86 = 38522;
pub const addr_stru_86966C: Consts_86 = 38508;
pub const addr_stru_869650: Consts_86 = 38480;
pub const addr_stru_869634: Consts_86 = 38452;
pub const addr_off_869574: Consts_86 = 38260;
pub const addr_kRidleysFireball_Tab0: Consts_86 = 37896;
pub const addr_kEproj_CrocomireSpikeWallPieces: Consts_86 = 37057;
pub const addr_stru_868F9D: Consts_86 = 36765;
pub const addr_stru_868F8F: Consts_86 = 36751;
pub const addr_word_868EDF: Consts_86 = 36575;
pub const addr_stru_868E5E: Consts_86 = 36446;
pub const addr_stru_868E50: Consts_86 = 36432;
pub const addr_stru_868C24: Consts_86 = 35876;
pub const addr_stru_868C16: Consts_86 = 35862;
pub const addr_stru_868C08: Consts_86 = 35848;
pub const addr_stru_868BFA: Consts_86 = 35834;
pub const addr_stru_868BEC: Consts_86 = 35820;
pub const addr_stru_868BDE: Consts_86 = 35806;
pub const addr_stru_868BD0: Consts_86 = 35792;
pub const addr_stru_868BC2: Consts_86 = 35778;
pub use crate::types::Consts_B3;
pub const addr_kEscapeDachora_Ilist_E964: Consts_B3 = 59748;
pub const addr_kEscapeEtecoon_Ilist_E5DA: Consts_B3 = 58842;
pub const addr_kEscapeEtecoon_Ilist_E5AE: Consts_B3 = 58798;
pub const addr_kEscapeEtecoon_Ilist_E582: Consts_B3 = 58754;
pub const addr_kEscapeEtecoon_Ilist_E556: Consts_B3 = 58710;
pub const addr_loc_B3E28C: Consts_B3 = 57996;
pub const addr_stru_B3E150: Consts_B3 = 57680;
pub const addr_kBotwoon_Ilist_9389: Consts_B3 = 37769;
pub const addr_kBrinstarYellowPipeBug_Ilist_8F38: Consts_B3 = 36664;
pub const addr_kBrinstarYellowPipeBug_Ilist_8F24: Consts_B3 = 36644;
pub const addr_kBrinstarYellowPipeBug_Ilist_8F10: Consts_B3 = 36624;
pub const addr_kBrinstarYellowPipeBug_Ilist_8EFC: Consts_B3 = 36604;
pub const addr_kNorfairPipeBug_Ilist_8B45: Consts_B3 = 35653;
pub const addr_kNorfairPipeBug_Ilist_8B21: Consts_B3 = 35617;
pub const addr_kNorfairPipeBug_Ilist_8B05: Consts_B3 = 35589;
pub const addr_kNorfairPipeBug_Ilist_8AE1: Consts_B3 = 35553;
pub const addr_kBrinstarPipeBug_Ilist_8A1D: Consts_B3 = 35357;
pub const addr_kBrinstarPipeBug_Ilist_87AB: Consts_B3 = 34731;
pub const addr_kUnusedSpinningTurtleEye_Ilist_86A7: Consts_B3 = 34471;
pub const addr_kSpritemap_Nothing_B3: Consts_B3 = 32845;
pub use crate::types::{Enemy_Botwoon, Point32, SpawnHardcodedPlmArgs};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_EscapeDachora {
    pub base: EnemyBase,
    pub eda_var_A: uint16,
    pub eda_var_B: uint16,
    pub eda_var_C: uint16,
    pub eda_var_D: uint16,
    pub eda_var_E: uint16,
    pub eda_var_F: uint16,
    pub eda_parameter_1: uint16,
    pub eda_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_EscapeEtecoon {
    pub base: EnemyBase,
    pub een_var_A: uint16,
    pub een_var_B: uint16,
    pub een_var_C: uint16,
    pub een_var_D: uint16,
    pub een_var_E: uint16,
    pub een_var_F: uint16,
    pub een_parameter_1: uint16,
    pub een_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_PipeBug {
    pub base: EnemyBase,
    pub pbg_var_A: uint16,
    pub pbg_var_B: uint16,
    pub pbg_var_C: uint16,
    pub pbg_var_D: uint16,
    pub pbg_var_E: uint16,
    pub pbg_var_F: uint16,
    pub pbg_parameter_1: uint16,
    pub pbg_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub pbg_var_00: uint16,
    pub pbg_var_01: uint16,
    pub pbg_var_02: uint16,
    pub pbg_var_03: uint16,
    pub pbg_var_04: uint16,
    pub pbg_var_05: uint16,
    pub pbg_var_06: uint16,
    pub pbg_var_07: uint16,
    pub pbg_var_08: uint16,
    pub pbg_var_09: uint16,
    pub pbg_var_0A: uint16,
    pub pbg_var_0B: uint16,
    pub pbg_var_0C: uint16,
    pub pbg_var_0D: uint16,
    pub pbg_var_0E: uint16,
    pub pbg_var_0F: uint16,
    pub pbg_var_10: uint16,
    pub pbg_var_11: uint16,
    pub pbg_var_12: uint16,
    pub pbg_var_13: uint16,
    pub pbg_var_14: uint16,
    pub pbg_var_15: uint16,
    pub pbg_var_16: uint16,
    pub pbg_var_17: uint16,
    pub pbg_var_18: uint16,
    pub pbg_var_19: uint16,
    pub pbg_var_1A: uint16,
    pub pbg_var_1B: uint16,
    pub pbg_var_1C: uint16,
    pub pbg_var_1D: uint16,
    pub pbg_var_1E: uint16,
    pub pbg_var_1F: uint16,
    pub pad2: [uint8; 1984],
    pub pbg_var_20: uint16,
    pub pbg_var_21: uint16,
    pub pbg_var_22: uint16,
    pub pbg_var_23: uint16,
    pub pbg_var_24: uint16,
    pub pbg_var_25: uint16,
    pub pbg_var_26: uint16,
    pub pbg_var_27: uint16,
    pub pbg_var_28: uint16,
    pub pbg_var_29: uint16,
    pub pbg_var_2A: uint16,
    pub pbg_var_2B: uint16,
    pub pbg_var_2C: uint16,
    pub pbg_var_2D: uint16,
    pub pbg_var_2E: uint16,
    pub pbg_var_2F: uint16,
    pub pbg_var_30: uint16,
    pub pbg_var_31: uint16,
    pub pbg_var_32: uint16,
    pub pbg_var_33: uint16,
    pub pbg_var_34: uint16,
    pub pbg_var_35: uint16,
    pub pbg_var_36: uint16,
    pub pbg_var_37: uint16,
    pub pbg_var_38: uint16,
    pub pbg_var_39: uint16,
    pub pbg_var_3A: uint16,
    pub pbg_var_3B: uint16,
    pub pbg_var_3C: uint16,
    pub pbg_var_3D: uint16,
    pub pbg_var_3E: uint16,
    pub pbg_var_3F: uint16,
}
#[inline]
unsafe fn abs16(mut t: uint16) -> uint16 {
    return (if t as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(t as ::core::ffi::c_int)
    } else {
        t as ::core::ffi::c_int
    }) as uint16;
}
#[inline]
unsafe fn RomPtr_B3(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0xb30000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
#[inline]
unsafe fn AddToHiLo(mut hi: *mut uint16, mut lo: *mut uint16, mut v: uint32) {
    let mut t: uint32 = ((*lo as ::core::ffi::c_int
        + ((*hi as ::core::ffi::c_int) << 16 as ::core::ffi::c_int))
        as uint32)
        .wrapping_add(v);
    *lo = t as uint16;
    *hi = (t >> 16 as ::core::ffi::c_int) as uint16;
}
pub const fnnullsub_171_B3: ::core::ffi::c_int = 0xb3807b as ::core::ffi::c_int;
pub const fnBrinstarPipeBug_PreInstr_1: ::core::ffi::c_int = 0xb38880 as ::core::ffi::c_int;
pub const fnBrinstarPipeBug_PreInstr_2: ::core::ffi::c_int = 0xb38890 as ::core::ffi::c_int;
pub const fnBrinstarPipeBug_PreInstr_3: ::core::ffi::c_int = 0xb388e3 as ::core::ffi::c_int;
pub const fnBrinstarPipeBug_PreInstr_4: ::core::ffi::c_int = 0xb3891c as ::core::ffi::c_int;
pub const fnBrinstarPipeBug_PreInstr_5: ::core::ffi::c_int = 0xb3897e as ::core::ffi::c_int;
pub const fnNorfairPipeBug_Func_1: ::core::ffi::c_int = 0xb38bcd as ::core::ffi::c_int;
pub const fnNorfairPipeBug_Func_2: ::core::ffi::c_int = 0xb38bff as ::core::ffi::c_int;
pub const fnNorfairPipeBug_Func_4: ::core::ffi::c_int = 0xb38ca6 as ::core::ffi::c_int;
pub const fnNorfairPipeBug_Func_5: ::core::ffi::c_int = 0xb38cff as ::core::ffi::c_int;
pub const fnNorfairPipeBug_Func_6: ::core::ffi::c_int = 0xb38d0c as ::core::ffi::c_int;
pub const fnNorfairPipeBug_Func_7: ::core::ffi::c_int = 0xb38d4e as ::core::ffi::c_int;
pub const fnNorfairPipeBug_Func_8: ::core::ffi::c_int = 0xb38d90 as ::core::ffi::c_int;
pub const fnNorfairPipeBug_Func_9: ::core::ffi::c_int = 0xb38dd2 as ::core::ffi::c_int;
pub const fnNorfairPipeBug_Func_10: ::core::ffi::c_int = 0xb38e14 as ::core::ffi::c_int;
pub const fnNorfairPipeBug_Func_11: ::core::ffi::c_int = 0xb38e35 as ::core::ffi::c_int;
pub const fnNorfairPipeBug_Func_12: ::core::ffi::c_int = 0xb38e5a as ::core::ffi::c_int;
pub const fnBrinstarYellowPipeBug_Func_1: ::core::ffi::c_int = 0xb38fb5 as ::core::ffi::c_int;
pub const fnBrinstarYellowPipeBug_Func_2: ::core::ffi::c_int = 0xb38ff5 as ::core::ffi::c_int;
pub const fnBrinstarYellowPipeBug_Func_3: ::core::ffi::c_int = 0xb39028 as ::core::ffi::c_int;
pub const fnBrinstarYellowPipeBug_Func_5: ::core::ffi::c_int = 0xb390bd as ::core::ffi::c_int;
pub const fnBrinstarYellowPipeBug_Func_7: ::core::ffi::c_int = 0xb3915a as ::core::ffi::c_int;
pub const fnBrinstarYellowPipeBug_Func_8: ::core::ffi::c_int = 0xb391d8 as ::core::ffi::c_int;
pub const fnBotwoon_Func_6: ::core::ffi::c_int = 0xb39878 as ::core::ffi::c_int;
pub const fnBotwoon_Func_7: ::core::ffi::c_int = 0xb3989d as ::core::ffi::c_int;
pub const fnBotwoon_Func_12: ::core::ffi::c_int = 0xb399a4 as ::core::ffi::c_int;
pub const fnBotwoon_Func_13: ::core::ffi::c_int = 0xb399e4 as ::core::ffi::c_int;
pub const fnBotwoon_Func_14: ::core::ffi::c_int = 0xb39a46 as ::core::ffi::c_int;
pub const fnBotwoon_Func_15: ::core::ffi::c_int = 0xb39a5e as ::core::ffi::c_int;
pub const fnBotwoon_Func_16: ::core::ffi::c_int = 0xb39aca as ::core::ffi::c_int;
pub const fnBotwoon_Func_18: ::core::ffi::c_int = 0xb39af9 as ::core::ffi::c_int;
pub const fnBotwoon_Func_19: ::core::ffi::c_int = 0xb39bb7 as ::core::ffi::c_int;
pub const fnBotwoon_Func_26: ::core::ffi::c_int = 0xb39dc0 as ::core::ffi::c_int;
pub const fnBotwoon_Func_27: ::core::ffi::c_int = 0xb39e7d as ::core::ffi::c_int;
pub const fnBotwoon_Func_28: ::core::ffi::c_int = 0xb39ee0 as ::core::ffi::c_int;
pub const fnBotwoon_Func_29: ::core::ffi::c_int = 0xb39f34 as ::core::ffi::c_int;
pub const fnBotwoon_Func_30: ::core::ffi::c_int = 0xb39f7a as ::core::ffi::c_int;
pub const fnBotwoon_Func_32: ::core::ffi::c_int = 0xb3e250 as ::core::ffi::c_int;
pub const fnBotwoon_Func_33: uint32 = 11788940 as uint32;
#[inline]
unsafe fn gEnemyData(mut o: ::core::ffi::c_int) -> *mut EnemyData {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut EnemyData;
}
#[inline]
unsafe fn Get_Botwoon(mut o: ::core::ffi::c_int) -> *mut Enemy_Botwoon {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_Botwoon;
}
#[inline]
unsafe fn Get_EscapeDachora(mut o: ::core::ffi::c_int) -> *mut Enemy_EscapeDachora {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_EscapeDachora;
}
#[inline]
unsafe fn Get_EscapeEtecoon(mut o: ::core::ffi::c_int) -> *mut Enemy_EscapeEtecoon {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_EscapeEtecoon;
}
#[inline]
unsafe fn Get_PipeBug(mut o: ::core::ffi::c_int) -> *mut Enemy_PipeBug {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_PipeBug;
}
static mut g_word_B39E77: [int16; 3] = [
    2 as ::core::ffi::c_int as int16,
    3 as ::core::ffi::c_int as int16,
    4 as ::core::ffi::c_int as int16,
];
pub unsafe fn Enemy_GrappleReact_NoInteract_B3() {
    SwitchEnemyAiToMainAi();
}
pub unsafe fn Enemy_GrappleReact_KillEnemy_B3() {
    EnemyGrappleDeath();
}
pub unsafe fn Enemy_GrappleReact_CancelBeam_B3() {
    Enemy_SwitchToFrozenAi();
}
pub unsafe fn Enemy_NormalTouchAI_B3() {
    NormalEnemyTouchAi();
}
pub unsafe fn Enemy_NormalTouchAI_SkipDeathAnim_B3() {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy();
}
pub unsafe fn Enemy_NormalShotAI_B3() {
    NormalEnemyShotAi();
}
pub unsafe fn Enemy_NormalShotAI_SkipSomeParts_B3() {
    NormalEnemyShotAiSkipDeathAnim_CurEnemy();
}
pub unsafe fn Enemy_NormalPowerBombAI_SkipDeathAnim_B3() {
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy();
}
pub unsafe fn Enemy_NormalFrozenAI_B3() {
    NormalEnemyFrozenAI();
}
pub unsafe fn Enemy_SetAiPreInstr_B3(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*gEnemyData(k as ::core::ffi::c_int)).ai_preinstr =
        *jp.offset(0 as ::core::ffi::c_int as isize);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn Enemy_ClearAiPreInstr_B3(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*gEnemyData(k as ::core::ffi::c_int)).ai_preinstr =
        (0xb3807b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn UnusedSpinningTurtleEye_Init() {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v0).properties = ((*v0).properties as ::core::ffi::c_int
        | kEnemyProps_DisableSamusColl as ::core::ffi::c_int) as uint16;
    (*v0).current_instruction =
        addr_kUnusedSpinningTurtleEye_Ilist_86A7 as ::core::ffi::c_int as uint16;
}
pub unsafe fn UnusedSpinningTurtleEye_Main() {}
pub unsafe fn BrinstarPipeBug_Init() {
    let mut PipeBug: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*PipeBug).pbg_var_B = (*PipeBug).base.x_pos;
    let mut y_pos: uint16 = (*PipeBug).base.y_pos;
    (*PipeBug).pbg_var_C = y_pos;
    (*PipeBug).pbg_var_00 = (y_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
    (*PipeBug).pbg_var_F =
        (0xb38880 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*PipeBug).pbg_var_D = 48 as uint16;
    (*PipeBug).pbg_var_E = 0 as uint16;
    (*PipeBug).pbg_var_01 = 0 as uint16;
    (*PipeBug).base.current_instruction =
        addr_kBrinstarPipeBug_Ilist_87AB as ::core::ffi::c_int as uint16;
    if (*PipeBug).pbg_parameter_1 != 0 {
        (*PipeBug).base.current_instruction =
            addr_kBrinstarPipeBug_Ilist_8A1D as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn BrinstarPipeBug_Main() {
    let mut PipeBug: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallEnemyPreInstr(
        ((*PipeBug).pbg_var_F as ::core::ffi::c_int | 0xb30000 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn BrinstarPipeBug_PreInstr_1(mut k: uint16) {
    if CheckIfEnemyIsOnScreen() == 0 {
        (*Get_PipeBug(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        ))
        .pbg_var_F = (0xb38890 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn BrinstarPipeBug_PreInstr_2(mut k: uint16) {
    let mut SamusEnemyDelta_Y: int16 = 0;
    SamusEnemyDelta_Y = GetSamusEnemyDelta_Y(k) as int16;
    if (SamusEnemyDelta_Y as ::core::ffi::c_int) < 0 as ::core::ffi::c_int
        && SamusEnemyDelta_Y as ::core::ffi::c_int + 96 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        let mut PipeBug: *mut Enemy_PipeBug = Get_PipeBug(k as ::core::ffi::c_int);
        let mut a: uint16 = GetSamusEnemyDelta_X(k);
        (*PipeBug).pbg_var_A =
            ((*PipeBug).pbg_var_A as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int
                | a as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) as uint16;
        if (abs16(
            ((*PipeBug).base.x_pos as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int)
            < 0x40 as ::core::ffi::c_int
        {
            (*PipeBug).pbg_var_F =
                (0xb388e3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*PipeBug).base.properties = ((*PipeBug).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Invisible as ::core::ffi::c_int))
                as uint16;
            (*PipeBug).base.timer = 0 as uint16;
            let mut v4: uint16 = 0;
            if (*PipeBug).pbg_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                v4 = 0 as uint16;
            } else {
                v4 = 2 as uint16;
            }
            (*PipeBug).pbg_var_E = v4;
            BrinstarPipeBug_Func_1();
        }
    }
}
pub unsafe fn BrinstarPipeBug_PreInstr_3(mut k: uint16) {
    let mut PipeBug: *mut Enemy_PipeBug = Get_PipeBug(k as ::core::ffi::c_int);
    let fresh7 = (*PipeBug).base.y_subpos;
    (*PipeBug).base.y_subpos = (*PipeBug).base.y_subpos.wrapping_sub(1);
    let mut v2: bool = fresh7 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    (*PipeBug).base.y_pos = ((*PipeBug).base.y_pos as ::core::ffi::c_int
        + (v2 as ::core::ffi::c_int - 2 as ::core::ffi::c_int))
        as uint16;
    if ((*PipeBug).pbg_var_00 as ::core::ffi::c_int - (*PipeBug).base.y_pos as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
        && ((*PipeBug).base.y_pos as ::core::ffi::c_int)
            < *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
    {
        (*PipeBug).pbg_var_E =
            ((*PipeBug).pbg_var_E as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint16;
        BrinstarPipeBug_Func_1();
        (*PipeBug).pbg_var_F =
            (0xb3891c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn BrinstarPipeBug_PreInstr_4(mut k: uint16) {
    let mut PipeBug: *mut Enemy_PipeBug = Get_PipeBug(k as ::core::ffi::c_int);
    if (*PipeBug).pbg_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*PipeBug).base.x_subpos = (*PipeBug).base.x_subpos;
        (*PipeBug).base.x_pos =
            ((*PipeBug).base.x_pos as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    } else {
        (*PipeBug).base.x_subpos = (*PipeBug).base.x_subpos;
        (*PipeBug).base.x_pos =
            ((*PipeBug).base.x_pos as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    }
    if IsEnemyLeavingScreen(k) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        (*PipeBug).base.x_pos = (*PipeBug).pbg_var_B;
        (*PipeBug).base.x_subpos = 0 as uint16;
        let mut pbg_var_C: uint16 = (*PipeBug).pbg_var_C;
        (*PipeBug).base.y_pos = pbg_var_C;
        (*PipeBug).base.y_subpos = pbg_var_C;
        (*PipeBug).pbg_var_E = 0 as uint16;
        BrinstarPipeBug_Func_1();
        (*PipeBug).base.properties = ((*PipeBug).base.properties as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int)
            as uint16;
        (*PipeBug).pbg_var_D = 48 as uint16;
        (*PipeBug).pbg_var_F =
            (0xb3897e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn BrinstarPipeBug_PreInstr_5(mut k: uint16) {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(k as ::core::ffi::c_int);
    let fresh8 = (*E).pbg_var_D;
    (*E).pbg_var_D = (*E).pbg_var_D.wrapping_sub(1);
    if fresh8 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).pbg_var_F = (0xb38890 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn BrinstarPipeBug_Func_1() {
    let mut PipeBug: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut pbg_var_E: uint16 = (*PipeBug).pbg_var_E;
    if pbg_var_E as ::core::ffi::c_int != (*PipeBug).pbg_var_01 as ::core::ffi::c_int {
        (*PipeBug).pbg_var_01 = pbg_var_E;
        (*PipeBug).base.current_instruction =
            (if (*PipeBug).pbg_parameter_1 as ::core::ffi::c_int != 0 {
                *(g_rom.offset(
                    (((0xb38833 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xb38833 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(pbg_var_E as isize) as ::core::ffi::c_int
            } else {
                *(g_rom.offset(
                    (((0xb3882b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xb3882b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(pbg_var_E as isize) as ::core::ffi::c_int
            }) as uint16;
        (*PipeBug).base.instruction_timer = 1 as uint16;
        (*PipeBug).base.timer = 0 as uint16;
    }
}
pub unsafe fn NorfairPipeBug_Init() {
    let mut PipeBug: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*PipeBug).pbg_var_D = (*PipeBug).base.x_pos;
    (*PipeBug).pbg_var_E = (*PipeBug).base.y_pos;
    (*PipeBug).base.properties = ((*PipeBug).base.properties as ::core::ffi::c_int
        | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
    (*PipeBug).pbg_var_B = (8 as ::core::ffi::c_int
        * *(&raw mut (*PipeBug).pbg_parameter_2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int) as uint16;
    (*PipeBug).pbg_var_C = 0 as uint16;
    (*PipeBug).base.instruction_timer = 1 as uint16;
    (*PipeBug).base.timer = 0 as uint16;
    (*PipeBug).base.current_instruction =
        addr_kNorfairPipeBug_Ilist_8AE1 as ::core::ffi::c_int as uint16;
    (*PipeBug).pbg_var_A =
        (0xb38bcd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CallBugInstr(mut ea: uint32, mut k: uint16) {
    match ea {
        11766733 => {
            NorfairPipeBug_Func_1();
            return;
        }
        11766783 => {
            NorfairPipeBug_Func_2();
            return;
        }
        11766950 => {
            NorfairPipeBug_Func_4();
            return;
        }
        11767039 => {
            NorfairPipeBug_Func_5(k);
            return;
        }
        11767052 => {
            NorfairPipeBug_Func_6();
            return;
        }
        11767118 => {
            NorfairPipeBug_Func_7();
            return;
        }
        11767184 => {
            NorfairPipeBug_Func_8();
            return;
        }
        11767250 => {
            NorfairPipeBug_Func_9();
            return;
        }
        11767316 => {
            NorfairPipeBug_Func_10();
            return;
        }
        11767349 => {
            NorfairPipeBug_Func_11();
            return;
        }
        11767386 => {
            NorfairPipeBug_Func_12();
            return;
        }
        11767733 => {
            BrinstarYellowPipeBug_Func_1();
            return;
        }
        11767797 => {
            BrinstarYellowPipeBug_Func_2(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
            return;
        }
        11767848 => {
            BrinstarYellowPipeBug_Func_3();
            return;
        }
        11767997 => {
            BrinstarYellowPipeBug_Func_5();
            return;
        }
        11768154 => {
            BrinstarYellowPipeBug_Func_7();
            return;
        }
        11768280 => {
            BrinstarYellowPipeBug_Func_8();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
pub unsafe fn NorfairPipeBug_Main() {
    let mut PipeBug: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallBugInstr(
        ((*PipeBug).pbg_var_A as ::core::ffi::c_int | 0xb30000 as ::core::ffi::c_int) as uint32,
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    NorfairPipeBug_8BA8();
}
pub unsafe fn NorfairPipeBug_8BA8() {
    if CheckIfEnemyIsOnScreen() != 0 {
        let mut PipeBug: *mut Enemy_PipeBug = Get_PipeBug(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        );
        (*PipeBug).base.properties = ((*PipeBug).base.properties as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int)
            as uint16;
        (*PipeBug).pbg_var_A =
            (0xb38bcd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*PipeBug).base.x_pos = (*PipeBug).pbg_var_D;
        (*PipeBug).base.y_pos = (*PipeBug).pbg_var_E;
    }
}
pub unsafe fn NorfairPipeBug_Func_1() {
    if *(&raw mut (*(Get_PipeBug as unsafe fn(::core::ffi::c_int) -> *mut Enemy_PipeBug)(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .pbg_parameter_2 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != 0
        && (*Get_PipeBug(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 64 as ::core::ffi::c_int,
        ))
        .pbg_var_A as ::core::ffi::c_int
            == 0xb38bcd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        && (*Get_PipeBug(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 128 as ::core::ffi::c_int,
        ))
        .pbg_var_A as ::core::ffi::c_int
            == 0xb38bcd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        && (*Get_PipeBug(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 192 as ::core::ffi::c_int,
        ))
        .pbg_var_A as ::core::ffi::c_int
            == 0xb38bcd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        && (*Get_PipeBug(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 256 as ::core::ffi::c_int,
        ))
        .pbg_var_A as ::core::ffi::c_int
            == 0xb38bcd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        (*Get_PipeBug(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        ))
        .pbg_var_A = (0xb38bff as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn NorfairPipeBug_Func_2() {
    let mut k: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0xe54 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
    let mut PipeBug: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if IsSamusWithinEnemy_X(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        *(&raw mut (*PipeBug).pbg_parameter_2 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as uint16,
    ) as ::core::ffi::c_int
        != 0
        && ((*PipeBug).base.y_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
    {
        (*PipeBug).pbg_var_C = (*PipeBug).pbg_var_C.wrapping_add(1);
        (*PipeBug).base.instruction_timer = 1 as uint16;
        (*PipeBug).base.timer = 0 as uint16;
        if GetSamusEnemyDelta_X(k as uint16) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*Get_PipeBug(k)).base.current_instruction =
                addr_kNorfairPipeBug_Ilist_8B21 as ::core::ffi::c_int as uint16;
            (*Get_PipeBug(k + 64 as ::core::ffi::c_int))
                .base
                .current_instruction =
                addr_kNorfairPipeBug_Ilist_8B21 as ::core::ffi::c_int as uint16;
            (*Get_PipeBug(k + 128 as ::core::ffi::c_int))
                .base
                .current_instruction =
                addr_kNorfairPipeBug_Ilist_8B21 as ::core::ffi::c_int as uint16;
            (*Get_PipeBug(k + 192 as ::core::ffi::c_int))
                .base
                .current_instruction =
                addr_kNorfairPipeBug_Ilist_8B21 as ::core::ffi::c_int as uint16;
            (*Get_PipeBug(k + 256 as ::core::ffi::c_int))
                .base
                .current_instruction =
                addr_kNorfairPipeBug_Ilist_8B21 as ::core::ffi::c_int as uint16;
        } else {
            (*PipeBug).base.current_instruction =
                addr_kNorfairPipeBug_Ilist_8AE1 as ::core::ffi::c_int as uint16;
            (*Get_PipeBug(k + 64 as ::core::ffi::c_int))
                .base
                .current_instruction =
                addr_kNorfairPipeBug_Ilist_8AE1 as ::core::ffi::c_int as uint16;
            (*Get_PipeBug(k + 128 as ::core::ffi::c_int))
                .base
                .current_instruction =
                addr_kNorfairPipeBug_Ilist_8AE1 as ::core::ffi::c_int as uint16;
            (*Get_PipeBug(k + 192 as ::core::ffi::c_int))
                .base
                .current_instruction =
                addr_kNorfairPipeBug_Ilist_8AE1 as ::core::ffi::c_int as uint16;
            (*Get_PipeBug(k + 256 as ::core::ffi::c_int))
                .base
                .current_instruction =
                addr_kNorfairPipeBug_Ilist_8AE1 as ::core::ffi::c_int as uint16;
        }
        NorfairPipeBug_Func_3(k as uint16);
    }
}
pub unsafe fn NorfairPipeBug_Func_3(mut k: uint16) {
    (*Get_PipeBug(k as ::core::ffi::c_int)).pbg_var_A =
        (0xb38ca6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*Get_PipeBug(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int)).pbg_var_A =
        (0xb38ca6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*Get_PipeBug(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int)).pbg_var_A =
        (0xb38ca6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*Get_PipeBug(k as ::core::ffi::c_int + 192 as ::core::ffi::c_int)).pbg_var_A =
        (0xb38ca6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*Get_PipeBug(k as ::core::ffi::c_int + 256 as ::core::ffi::c_int)).pbg_var_A =
        (0xb38ca6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*Get_PipeBug(k as ::core::ffi::c_int)).pbg_var_02 = 104 as uint16;
    (*Get_PipeBug(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int)).pbg_var_02 = 96 as uint16;
    (*Get_PipeBug(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int)).pbg_var_02 = 88 as uint16;
    (*Get_PipeBug(k as ::core::ffi::c_int + 192 as ::core::ffi::c_int)).pbg_var_02 = 112 as uint16;
    (*Get_PipeBug(k as ::core::ffi::c_int + 256 as ::core::ffi::c_int)).pbg_var_02 = 120 as uint16;
    (*Get_PipeBug(k as ::core::ffi::c_int)).pbg_var_F =
        (0xb38cff as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*Get_PipeBug(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int)).pbg_var_F =
        (0xb38d0c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*Get_PipeBug(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int)).pbg_var_F =
        (0xb38d4e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*Get_PipeBug(k as ::core::ffi::c_int + 192 as ::core::ffi::c_int)).pbg_var_F =
        (0xb38d90 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*Get_PipeBug(k as ::core::ffi::c_int + 256 as ::core::ffi::c_int)).pbg_var_F =
        (0xb38dd2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn NorfairPipeBug_Func_4() {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(66 as ::core::ffi::c_int as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(67 as ::core::ffi::c_int as isize) as uint32,
    );
    if (((*E).base.y_pos as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        (*E).pbg_var_A = (*E).pbg_var_F;
        (*E).pbg_var_01 = (*E).base.y_pos;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        if GetSamusEnemyDelta_X(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        ) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*E).base.current_instruction =
                addr_kNorfairPipeBug_Ilist_8B21 as ::core::ffi::c_int as uint16;
        } else {
            (*E).base.current_instruction =
                addr_kNorfairPipeBug_Ilist_8AE1 as ::core::ffi::c_int as uint16;
        }
    }
}
pub unsafe fn NorfairPipeBug_Func_5(mut k: uint16) {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    (*E).pbg_var_A = (0xb38e5a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn NorfairPipeBug_Func_6() {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(66 as ::core::ffi::c_int as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(67 as ::core::ffi::c_int as isize) as uint32,
    );
    if (((*E).base.y_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int
        - (*E).pbg_var_01 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos =
            ((*E).pbg_var_01 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
        (*E).base.y_subpos = 0 as uint16;
        (*E).pbg_var_A = (0xb38e5a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn NorfairPipeBug_Func_7() {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(66 as ::core::ffi::c_int as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(67 as ::core::ffi::c_int as isize) as uint32,
    );
    if (((*E).base.y_pos as ::core::ffi::c_int + 32 as ::core::ffi::c_int
        - (*E).pbg_var_01 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos =
            ((*E).pbg_var_01 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
        (*E).base.y_subpos = 0 as uint16;
        (*E).pbg_var_A = (0xb38e5a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn NorfairPipeBug_Func_8() {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(64 as ::core::ffi::c_int as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(65 as ::core::ffi::c_int as isize) as uint32,
    );
    if ((*E).base.y_pos as ::core::ffi::c_int
        - 16 as ::core::ffi::c_int
        - (*E).pbg_var_01 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos =
            ((*E).pbg_var_01 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
        (*E).base.y_subpos = 0 as uint16;
        (*E).pbg_var_A = (0xb38e5a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn NorfairPipeBug_Func_9() {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(64 as ::core::ffi::c_int as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(65 as ::core::ffi::c_int as isize) as uint32,
    );
    if ((*E).base.y_pos as ::core::ffi::c_int
        - 32 as ::core::ffi::c_int
        - (*E).pbg_var_01 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos =
            ((*E).pbg_var_01 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
        (*E).base.y_subpos = 0 as uint16;
        (*E).pbg_var_A = (0xb38e5a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn NorfairPipeBug_Func_10() {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v2: ::core::ffi::c_int =
        (*E).pbg_var_B as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (*(g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v2 + 2 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 + 3 as ::core::ffi::c_int) as isize) as uint32,
    );
}
pub unsafe fn NorfairPipeBug_Func_11() {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v2: ::core::ffi::c_int =
        (*E).pbg_var_B as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (*(g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 + 1 as ::core::ffi::c_int) as isize) as uint32,
    );
}
pub unsafe fn sub_B38E56() {}
pub unsafe fn NorfairPipeBug_Func_12() {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    if ((*E).pbg_var_C as ::core::ffi::c_int - (*E).pbg_var_02 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).pbg_var_C = 0 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).pbg_var_A = (0xb38e14 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction =
            addr_kNorfairPipeBug_Ilist_8B05 as ::core::ffi::c_int as uint16;
        if GetSamusEnemyDelta_X(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        ) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*E).pbg_var_A =
                (0xb38e35 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).base.current_instruction =
                addr_kNorfairPipeBug_Ilist_8B45 as ::core::ffi::c_int as uint16;
        }
    }
}
pub unsafe fn BrinstarYellowPipeBug_Init() {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).pbg_var_07 = (*E).base.x_pos;
    (*E).pbg_var_08 = (*E).base.y_pos;
    (*E).base.x_subpos = 0 as uint16;
    (*E).base.y_subpos = 0 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction =
        addr_kBrinstarYellowPipeBug_Ilist_8EFC as ::core::ffi::c_int as uint16;
    if (*E).pbg_parameter_1 == 0 {
        (*E).base.current_instruction =
            addr_kBrinstarYellowPipeBug_Ilist_8F24 as ::core::ffi::c_int as uint16;
    }
    let mut v1: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * (*E).pbg_parameter_2 as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).pbg_var_01 = *(g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize);
    (*E).pbg_var_00 = *(g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v1 + 1 as ::core::ffi::c_int) as isize);
    (*E).pbg_var_03 = *(g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v1 + 2 as ::core::ffi::c_int) as isize);
    (*E).pbg_var_02 = *(g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v1 + 3 as ::core::ffi::c_int) as isize);
    (*E).pbg_var_A = (0xb38fb5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).pbg_var_06 = 0 as uint16;
}
pub unsafe fn BrinstarYellowPipeBug_Main() {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallBugInstr(
        ((*E).pbg_var_A as ::core::ffi::c_int | 0xb30000 as ::core::ffi::c_int) as uint32,
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
pub unsafe fn BrinstarYellowPipeBug_Func_1() {
    let mut v2: int16 = 0;
    let mut SamusEnemyDelta_X: int16 = 0;
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut current_block_6: u64;
    if (*E).pbg_parameter_1 != 0 {
        SamusEnemyDelta_X = GetSamusEnemyDelta_X(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        ) as int16;
        if (SamusEnemyDelta_X as ::core::ffi::c_int) < 0 as ::core::ffi::c_int
            && SamusEnemyDelta_X as ::core::ffi::c_int + 192 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            current_block_6 = 1628911311613519252;
        } else {
            current_block_6 = 2868539653012386629;
        }
    } else {
        v2 = GetSamusEnemyDelta_X(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        ) as int16;
        if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int
            && v2 as ::core::ffi::c_int - 192 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
        {
            current_block_6 = 1628911311613519252;
        } else {
            current_block_6 = 2868539653012386629;
        }
    }
    match current_block_6 {
        1628911311613519252 => {
            if IsSamusWithinEnemy_Y(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                0x30 as uint16,
            ) != 0
            {
                (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                    & !(kEnemyProps_Invisible as ::core::ffi::c_int))
                    as uint16;
                (*E).pbg_var_20 = 24 as uint16;
                (*E).pbg_var_A =
                    (0xb38ff5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            }
        }
        _ => {}
    };
}
pub unsafe fn BrinstarYellowPipeBug_Func_2(mut k: uint16) {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(k as ::core::ffi::c_int);
    let mut v2: uint16 =
        ((*E).pbg_var_20 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E).pbg_var_20 = v2;
    if v2 == 0 {
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction =
            addr_kBrinstarYellowPipeBug_Ilist_8EFC as ::core::ffi::c_int as uint16;
        (*E).pbg_var_A = (0xb39028 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        if (*E).pbg_parameter_1 == 0 {
            (*E).base.current_instruction =
                addr_kBrinstarYellowPipeBug_Ilist_8F24 as ::core::ffi::c_int as uint16;
            (*E).pbg_var_A =
                (0xb390bd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn BrinstarYellowPipeBug_Func_3() {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    BrinstarYellowPipeBug_Func_4(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    if CheckIfEnemyIsOnScreen() != 0 {
        (*E).base.x_pos = (*E).pbg_var_07;
        (*E).base.y_pos = (*E).pbg_var_08;
        (*E).base.x_subpos = 0 as uint16;
        (*E).base.y_subpos = 0 as uint16;
        (*E).pbg_var_A = (0xb38fb5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).pbg_var_06 = 0 as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
    } else if (*E).pbg_var_06 == 0 {
        if (*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - 48 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            (*E).pbg_var_A =
                (0xb3915a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).pbg_var_C = 0 as uint16;
            (*E).pbg_var_D = 0 as uint16;
            (*E).pbg_var_E = 40 as uint16;
            (*E).pbg_var_F = 1 as uint16;
            (*E).pbg_var_04 = (*E).base.x_pos;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).base.timer = 0 as uint16;
            (*E).base.current_instruction =
                addr_kBrinstarYellowPipeBug_Ilist_8F10 as ::core::ffi::c_int as uint16;
        }
    }
}
pub unsafe fn BrinstarYellowPipeBug_Func_4(mut k: uint16) {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(k as ::core::ffi::c_int);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).pbg_var_03 as uint32) << 16 as ::core::ffi::c_int | (*E).pbg_var_02 as uint32,
    );
}
pub unsafe fn BrinstarYellowPipeBug_Func_5() {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    BrinstarYellowPipeBug_Func_6(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    if CheckIfEnemyIsOnScreen() != 0 {
        (*E).base.x_pos = (*E).pbg_var_07;
        (*E).base.y_pos = (*E).pbg_var_08;
        (*E).base.x_subpos = 0 as uint16;
        (*E).base.y_subpos = 0 as uint16;
        (*E).pbg_var_A = (0xb38fb5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).pbg_var_06 = 0 as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
    } else if (*E).pbg_var_06 == 0 {
        if *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int
            - 48 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            (*E).pbg_var_A =
                (0xb391d8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).pbg_var_C = 0 as uint16;
            (*E).pbg_var_D = 0 as uint16;
            (*E).pbg_var_E = 40 as uint16;
            (*E).pbg_var_F = 1 as uint16;
            (*E).pbg_var_04 = (*E).base.x_pos;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).base.timer = 0 as uint16;
            (*E).base.current_instruction =
                addr_kBrinstarYellowPipeBug_Ilist_8F38 as ::core::ffi::c_int as uint16;
        }
    }
}
pub unsafe fn BrinstarYellowPipeBug_Func_6(mut k: uint16) {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(k as ::core::ffi::c_int);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).pbg_var_01 as uint32) << 16 as ::core::ffi::c_int | (*E).pbg_var_00 as uint32,
    );
}
pub unsafe fn BrinstarYellowPipeBug_Func_7() {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if CheckIfEnemyIsOnScreen() != 0 {
        (*E).base.x_pos = (*E).pbg_var_07;
        (*E).base.y_pos = (*E).pbg_var_08;
        (*E).base.x_subpos = 0 as uint16;
        (*E).base.y_subpos = 0 as uint16;
        (*E).pbg_var_A = (0xb38fb5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).pbg_var_06 = 0 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction =
            addr_kBrinstarYellowPipeBug_Ilist_8EFC as ::core::ffi::c_int as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
    } else {
        BrinstarYellowPipeBug_Func_4(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
        if (*E).pbg_var_F != 0 {
            BrinstarYellowPipeBug_Func_10(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
        } else {
            BrinstarYellowPipeBug_Func_9(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
            if (((*E).base.y_pos as ::core::ffi::c_int - (*E).pbg_var_08 as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                (*E).pbg_var_06 = 1 as uint16;
                (*E).pbg_var_F = 1 as uint16;
                (*E).pbg_var_A =
                    (0xb39028 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                (*E).base.instruction_timer = 1 as uint16;
                (*E).base.timer = 0 as uint16;
                (*E).base.current_instruction =
                    addr_kBrinstarYellowPipeBug_Ilist_8EFC as ::core::ffi::c_int as uint16;
            }
        }
    };
}
pub unsafe fn BrinstarYellowPipeBug_Func_8() {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if CheckIfEnemyIsOnScreen() != 0 {
        (*E).base.x_pos = (*E).pbg_var_07;
        (*E).base.y_pos = (*E).pbg_var_08;
        (*E).base.x_subpos = 0 as uint16;
        (*E).base.y_subpos = 0 as uint16;
        (*E).pbg_var_A = (0xb38fb5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).pbg_var_06 = 0 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction =
            addr_kBrinstarYellowPipeBug_Ilist_8F24 as ::core::ffi::c_int as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
    } else {
        BrinstarYellowPipeBug_Func_6(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
        if (*E).pbg_var_F != 0 {
            BrinstarYellowPipeBug_Func_10(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
        } else {
            BrinstarYellowPipeBug_Func_9(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
            if ((*E).pbg_var_08 as ::core::ffi::c_int - (*E).base.y_pos as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                (*E).pbg_var_06 = 1 as uint16;
                (*E).pbg_var_F = 1 as uint16;
                (*E).base.instruction_timer = 1 as uint16;
                (*E).base.timer = 0 as uint16;
                (*E).base.current_instruction =
                    addr_kBrinstarYellowPipeBug_Ilist_8F24 as ::core::ffi::c_int as uint16;
                (*E).pbg_var_A =
                    (0xb390bd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            }
        }
    };
}
pub unsafe fn BrinstarYellowPipeBug_Func_9(mut k: uint16) {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(k as ::core::ffi::c_int);
    (*E).pbg_var_E = (*E).pbg_var_E.wrapping_add(1);
    let mut v3: ::core::ffi::c_int =
        8 as ::core::ffi::c_int * (*E).pbg_var_E as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(g_rom.offset(
            (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v3 + 3 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(g_rom.offset(
                (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v3 + 2 as ::core::ffi::c_int) as isize) as uint32,
    );
}
pub unsafe fn BrinstarYellowPipeBug_Func_10(mut k: uint16) {
    let mut E: *mut Enemy_PipeBug = Get_PipeBug(k as ::core::ffi::c_int);
    (*E).pbg_var_E = (*E).pbg_var_E.wrapping_sub(1);
    if (*E).pbg_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).pbg_var_E = 0 as uint16;
        (*E).pbg_var_F = 0 as uint16;
    } else {
        let mut v4: ::core::ffi::c_int = 8 as ::core::ffi::c_int
            * (*E).pbg_var_E as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            (*(g_rom.offset(
                (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v4 + 1 as ::core::ffi::c_int) as isize) as uint32)
                << 16 as ::core::ffi::c_int
                | *(g_rom.offset(
                    (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v4 as isize) as uint32,
        );
    };
}
pub unsafe fn Botwoon_Instr_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.x_width = 8 as uint16;
    (*E).base.y_height = 16 as uint16;
    return jp;
}
pub unsafe fn Botwoon_Instr_2(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.x_width = 12 as uint16;
    (*E).base.y_height = 12 as uint16;
    return jp;
}
pub unsafe fn Botwoon_Instr_3(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.x_width = 16 as uint16;
    (*E).base.y_height = 8 as uint16;
    return jp;
}
pub unsafe fn Botwoon_Instr_4(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.x_width = 12 as uint16;
    (*E).base.y_height = 12 as uint16;
    return jp;
}
pub unsafe fn Botwoon_Instr_5(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.x_width = 8 as uint16;
    (*E).base.y_height = 16 as uint16;
    return jp;
}
pub unsafe fn Botwoon_Instr_6(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.x_width = 8 as uint16;
    (*E).base.y_height = 16 as uint16;
    return jp;
}
pub unsafe fn Botwoon_Instr_7(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.x_width = 12 as uint16;
    (*E).base.y_height = 12 as uint16;
    return jp;
}
pub unsafe fn Botwoon_Instr_8(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.x_width = 16 as uint16;
    (*E).base.y_height = 8 as uint16;
    return jp;
}
pub unsafe fn Botwoon_Instr_9(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.x_width = 12 as uint16;
    (*E).base.y_height = 12 as uint16;
    return jp;
}
pub unsafe fn Botwoon_Instr_10(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.x_width = 8 as uint16;
    (*E).base.y_height = 16 as uint16;
    return jp;
}
pub unsafe fn Botwoon_Instr_SetSpitting(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .botwoon_var_5A = 1 as uint16;
    return jp;
}
pub unsafe fn Botwoon_Instr_QueueSpitSfx(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max6(0x7c as uint16);
    return jp;
}
pub unsafe fn Botwoon_QueueExplosionSfx() {
    QueueSfx2_Max6(0x24 as uint16);
}
pub unsafe fn Botwoon_Init() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if *(&raw mut g_ram as *mut uint8)
        .offset(0xd828 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
            field_0: 0xf as uint8,
            field_1: 0x4 as uint8,
            field_2: 0xb797 as VoidP,
        });
        *((&raw mut g_ram as *mut uint8).offset(0xcd20 as ::core::ffi::c_int as isize)
            as *mut uint16) = 257 as uint16;
        (*E).base.current_instruction = addr_kBotwoon_Ilist_9389 as ::core::ffi::c_int as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
    } else {
        let mut v3: uint16 = 24 as uint16;
        (*E).botwoon_var_A = 24 as uint16;
        let mut v5: bool = false;
        loop {
            SpawnEprojWithGfx(
                v3,
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                addr_kEproj_BotwoonsBody as ::core::ffi::c_int as uint16,
            );
            (*E).botwoon_var_A = (*E).botwoon_var_A.wrapping_sub(1);
            v5 = (((*E).botwoon_var_A as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int;
            (*E).botwoon_var_A = (*E).botwoon_var_A.wrapping_sub(1);
            if v5 {
                break;
            }
        }
        (*E).base.current_instruction = addr_kBotwoon_Ilist_9389 as ::core::ffi::c_int as uint16;
        (*E).botwoon_var_3B = addr_kBotwoon_Ilist_9389 as ::core::ffi::c_int as uint16;
        (*E).botwoon_var_D =
            (0xb39878 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).botwoon_var_E =
            (0xb39bb7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).botwoon_var_F =
            (0xb39dc0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).botwoon_var_20 = 256 as uint16;
        (*E).botwoon_var_38 = *(g_rom.offset(
            (((0xb394bb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xb394bb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize);
        (*E).botwoon_var_C = *(g_rom.offset(
            (((0xb394bb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xb394bb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize);
        (*E).botwoon_var_33 = 1 as uint16;
        (*E).botwoon_var_34 = 1 as uint16;
        (*E).botwoon_var_59 = 1 as uint16;
        (*E).botwoon_var_36 = -(1 as ::core::ffi::c_int) as uint16;
        (*E).botwoon_var_37 = 0 as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
        let mut health: uint16 = (*E).base.health;
        (*E).botwoon_var_5B = health;
        health = (health as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        (*E).botwoon_var_5C = health;
        (*E).botwoon_var_5D = (health as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        let mut x_pos: uint16 = (*E).base.x_pos;
        (*E).botwoon_var_50 = x_pos;
        (*E).botwoon_var_52 = x_pos;
        (*E).botwoon_var_54 = x_pos;
        (*E).botwoon_var_56 = x_pos;
        let mut y_pos: uint16 = (*E).base.y_pos;
        (*E).botwoon_var_51 = y_pos;
        (*E).botwoon_var_53 = y_pos;
        (*E).botwoon_var_55 = y_pos;
        (*E).botwoon_var_57 = y_pos;
        (*E).botwoon_var_4E = (((*E).base.palette_index as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int)
            + 256 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn CallBotwoonFuncVarD(mut ea: uint32, mut k: uint16) {
    match ea {
        11769976 => {
            Botwoon_Func_6();
            return;
        }
        11770013 => {
            Botwoon_Func_7();
            return;
        }
        11770276 => {
            Botwoon_Func_12();
            return;
        }
        11770340 => {
            Botwoon_Func_13();
            return;
        }
        11770438 => {
            Botwoon_Func_14();
            return;
        }
        11770462 => {
            Botwoon_Func_15();
            return;
        }
        11770570 => {
            Botwoon_Func_16();
            return;
        }
        11770617 => {
            Botwoon_Func_18(k);
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
pub unsafe fn CallBotwoonFuncVarE(mut ea: uint32) {
    match ea {
        11770807 => {
            Botwoon_Func_19();
            return;
        }
        11788880 => {
            Botwoon_Func_32();
            return;
        }
        11788940 => {
            Botwoon_Func_33();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
pub unsafe fn Botwoon_Main() {
    Botwoon_Func_3();
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallBotwoonFuncVarD(
        ((*E).botwoon_var_D as ::core::ffi::c_int | 0xb30000 as ::core::ffi::c_int) as uint32,
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    Botwoon_HealthBasedPalHandling();
}
pub unsafe fn Botwoon_Func_1(mut k: uint16) {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(k as ::core::ffi::c_int);
    if *(g_rom.offset(
        (((0xb39675 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb39675 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((*E).botwoon_var_3F as isize) as ::core::ffi::c_int
        & NextRandom() as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).botwoon_var_2E = 1 as uint16;
    }
}
pub unsafe fn Botwoon_Func_2() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).botwoon_var_2F == 0
        && (*E).botwoon_var_2E as ::core::ffi::c_int != 0
        && (*E).botwoon_var_33 == 0
        && (*E).botwoon_var_21 == 0
    {
        (*E).botwoon_var_F =
            (0xb39e7d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).botwoon_var_21 = 24 as uint16;
    }
    (*E).botwoon_var_2E = 0 as uint16;
}
pub unsafe fn Botwoon_Func_3() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).botwoon_var_2F != 0 {
        if (*E).botwoon_var_3E != 0 {
            (*E).botwoon_var_30 = 1 as uint16;
            (*E).botwoon_var_D =
                (0xb39a46 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).botwoon_var_22 = 240 as uint16;
            (*E).botwoon_var_2F = 0 as uint16;
            (*E).botwoon_var_3E = 0 as uint16;
        }
    }
}
pub unsafe fn Botwoon_Func_4() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(0 as ::core::ffi::c_int);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Botwoon_Func_5() {
    let mut v0: uint16 = 34 as uint16;
    loop {
        let ref mut fresh4 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1bd7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh4 = (*fresh4 as ::core::ffi::c_int | 0x2000 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xf380 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 2 as uint16;
        v0 = (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if !((v0 as ::core::ffi::c_int - 10 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
pub unsafe fn Botwoon_HealthBasedPalHandling() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).botwoon_var_4F as ::core::ffi::c_int != 16 as ::core::ffi::c_int
        && (((*E).base.health as ::core::ffi::c_int
            - *(g_rom.offset(
                (((0xb3981b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xb3981b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    ((*E).botwoon_var_4F as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
    {
        let mut r18: uint16 =
            (16 as ::core::ffi::c_int * (*E).botwoon_var_4F as ::core::ffi::c_int) as uint16;
        let mut r20: uint16 = (*E).botwoon_var_4E;
        loop {
            let fresh5 = r18;
            r18 = r18.wrapping_add(1);
            *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                *(g_rom.offset(
                    (((0xb3971b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xb3971b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((fresh5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            r18 = r18.wrapping_add(1);
            r20 = (r20 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(r20 as ::core::ffi::c_int != 512 as ::core::ffi::c_int) {
                break;
            }
        }
        (*E).botwoon_var_4F =
            ((*E).botwoon_var_4F as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Botwoon_Func_6() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v1: uint16 =
        ((*E).botwoon_var_20 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E).botwoon_var_20 = v1;
    if v1 == 0 {
        (*E).botwoon_var_D =
            (0xb3989d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Botwoon_Func_7() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).botwoon_var_3C != 0 {
        (*E).botwoon_var_3C = 0 as uint16;
        let mut r18: uint8 = 0 as uint8;
        if (*E).botwoon_var_33 == 0 && (*E).botwoon_var_59 == 0 && (*E).botwoon_var_3F == 0 {
            r18 = (NextRandom() as ::core::ffi::c_int & 0xe as ::core::ffi::c_int) as uint8;
        }
        (*E).botwoon_var_59 = 0 as uint16;
        if r18 == 0
            || r18 as ::core::ffi::c_int == 2 as ::core::ffi::c_int
            || r18 as ::core::ffi::c_int == 4 as ::core::ffi::c_int
        {
            Botwoon_Func_8();
        } else {
            if r18 as ::core::ffi::c_int != 6 as ::core::ffi::c_int
                && r18 as ::core::ffi::c_int != 8 as ::core::ffi::c_int
                && r18 as ::core::ffi::c_int != 10 as ::core::ffi::c_int
                && r18 as ::core::ffi::c_int != 12 as ::core::ffi::c_int
                && r18 as ::core::ffi::c_int != 14 as ::core::ffi::c_int
            {
                Unreachable();
            }
            Botwoon_Func_9();
        }
    } else {
        CallBotwoonFuncVarE(
            ((*E).botwoon_var_E as ::core::ffi::c_int | 0xb30000 as ::core::ffi::c_int) as uint32,
        );
        Botwoon_Func_22();
        Botwoon_Func_23();
        CallEnemyPreInstr(
            ((*E).botwoon_var_F as ::core::ffi::c_int | 0xb30000 as ::core::ffi::c_int) as uint32,
        );
        Botwoon_Func_25();
        Botwoon_Func_24();
        Botwoon_Func_31(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
    };
}
pub unsafe fn Botwoon_Func_8() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).botwoon_var_D = (0xb399a4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).botwoon_var_E = (0xb3e250 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).botwoon_var_21 = 0 as uint16;
    (*E).botwoon_var_F = (0xb39dc0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).botwoon_var_4B = 0 as uint16;
    Botwoon_Func_10(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
pub unsafe fn Botwoon_Func_9() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).botwoon_var_D = (0xb399e4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).botwoon_var_F = (0xb39e7d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).botwoon_var_21 = 48 as uint16;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn Botwoon_Func_10(mut k: uint16) {
    Botwoon_Func_11(k);
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(k as ::core::ffi::c_int);
    let mut r18: uint16 = (if (*E).botwoon_var_33 as ::core::ffi::c_int != 0 {
        128 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as uint16;
    (*E).botwoon_var_40 = ((NextRandom() as ::core::ffi::c_int & 0x18 as ::core::ffi::c_int)
        + r18 as ::core::ffi::c_int
        + 4 as ::core::ffi::c_int * (*E).botwoon_var_37 as ::core::ffi::c_int)
        as uint16;
}
pub unsafe fn Botwoon_Func_11(mut k: uint16) {
    let mut health: int16 = 0;
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(k as ::core::ffi::c_int);
    if (*E).botwoon_var_33 == 0 {
        (*E).botwoon_var_3F = 0 as uint16;
        health = (*E).base.health as int16;
        if health as ::core::ffi::c_int > 0 as ::core::ffi::c_int
            && ((health as ::core::ffi::c_int - (*E).botwoon_var_5C as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
        {
            if ((health as ::core::ffi::c_int - (*E).botwoon_var_5D as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                (*E).botwoon_var_3F = 2 as uint16;
            } else {
                (*E).botwoon_var_3F = 1 as uint16;
            }
            let mut v3: ::core::ffi::c_int = (4 as ::core::ffi::c_int
                * (*E).botwoon_var_3F as ::core::ffi::c_int)
                as uint16 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            (*E).botwoon_var_38 = *(g_rom.offset(
                (((0xb394bb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xb394bb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v3 as isize);
            (*E).botwoon_var_C = *(g_rom.offset(
                (((0xb394bb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xb394bb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v3 + 1 as ::core::ffi::c_int) as isize);
        }
    }
}
pub unsafe fn Botwoon_Func_12() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).botwoon_var_3C != 0 {
        (*E).botwoon_var_3C = 0 as uint16;
        (*E).botwoon_var_D =
            (0xb3989d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).botwoon_var_E =
            (0xb39bb7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        if (*E).botwoon_var_33 != 0 {
            (*E).botwoon_var_35 = 0 as uint16;
        }
    } else {
        CallBotwoonFuncVarE(
            ((*E).botwoon_var_E as ::core::ffi::c_int | 0xb30000 as ::core::ffi::c_int) as uint32,
        );
        Botwoon_Func_22();
        Botwoon_Func_23();
        CallEnemyPreInstr(
            ((*E).botwoon_var_F as ::core::ffi::c_int | 0xb30000 as ::core::ffi::c_int) as uint32,
        );
        Botwoon_Func_25();
        Botwoon_Func_24();
    };
}
pub unsafe fn Botwoon_Func_13() {
    let mut v3: uint16 = 0;
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).botwoon_var_21 != 0 {
        (*E).botwoon_var_2E = 0 as uint16;
        CallEnemyPreInstr(
            ((*E).botwoon_var_F as ::core::ffi::c_int | 0xb30000 as ::core::ffi::c_int) as uint32,
        );
    } else {
        (*E).botwoon_var_3C = 0 as uint16;
        (*E).botwoon_var_D =
            (0xb399a4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).botwoon_var_E =
            (0xb3e250 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).botwoon_var_F =
            (0xb39dc0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        if (*E).botwoon_var_2F as ::core::ffi::c_int != 0 || {
            v3 = (NextRandom() as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint16;
            (*E).botwoon_var_33 = v3;
            (*E).botwoon_var_34 = v3;
            (*E).botwoon_var_34 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        } {
            (*E).botwoon_var_4B = 0 as uint16;
        } else {
            (*E).botwoon_var_35 = 0 as uint16;
            (*E).botwoon_var_36 = -(1 as ::core::ffi::c_int) as uint16;
        }
        Botwoon_Func_10(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
    };
}
pub unsafe fn Botwoon_Func_14() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v1: uint16 =
        ((*E).botwoon_var_22 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    (*E).botwoon_var_22 = v1;
    if v1 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        (*E).botwoon_var_D =
            (0xb39a5e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Botwoon_Func_15() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * *(&raw mut (*E).botwoon_var_28 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(g_rom.offset(
            (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(g_rom.offset(
                (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v2 as isize) as uint32,
    );
    if (*E).base.y_pos as ::core::ffi::c_int - 200 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).botwoon_var_28 =
            ((*E).botwoon_var_28 as ::core::ffi::c_int + 192 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).base.y_pos = 200 as uint16;
        (*E).botwoon_var_D =
            (0xb39aca as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        eproj_spawn_pt = Point16U {
            x: (*E).base.x_pos,
            y: (*E).base.y_pos,
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
            0x1d as uint16,
        );
        Botwoon_QueueExplosionSfx();
        (*E).base.properties =
            ((*E).base.properties as ::core::ffi::c_int | 0x500 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Botwoon_Func_16() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).botwoon_var_5F != 0 {
        (*E).botwoon_var_D =
            (0xb39af9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        Botwoon_Func_17(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
    }
}
pub unsafe fn Botwoon_Func_17(mut k: uint16) {
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
        field_0: 0xf as uint8,
        field_1: 0x4 as uint8,
        field_2: 0xb79b as VoidP,
    });
    Enemy_ItemDrop_Botwoon(k);
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(k as ::core::ffi::c_int);
    (*E).botwoon_var_23 = 0 as uint16;
    (*E).botwoon_var_24 = 0 as uint16;
    (*E).botwoon_var_25 = 0 as uint16;
}
pub unsafe fn Botwoon_Func_18(mut k: uint16) {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(k as ::core::ffi::c_int);
    if (*E).botwoon_var_23 as ::core::ffi::c_int - 192 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if (*E).botwoon_var_23 as ::core::ffi::c_int - 64 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            let mut v2: int16 =
                ((*E).botwoon_var_24 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
            (*E).botwoon_var_24 = v2 as uint16;
            if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                (*E).botwoon_var_24 = 12 as uint16;
                let mut x: uint16 = ((NextRandom() as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int)
                    + 232 as ::core::ffi::c_int) as uint16;
                let mut y: uint16 = ((*E).botwoon_var_23 as ::core::ffi::c_int
                    + (NextRandom() as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
                    - 8 as ::core::ffi::c_int) as uint16;
                CreateSpriteAtPos(x, y, 29 as uint16, 2560 as uint16);
                Botwoon_QueueExplosionSfx();
            }
            let mut v3: int16 =
                ((*E).botwoon_var_25 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
            (*E).botwoon_var_25 = v3 as uint16;
            if (v3 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                (*E).botwoon_var_25 = 4 as uint16;
                let mut n: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
                loop {
                    let mut x_0: uint16 =
                        ((NextRandom() as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int)
                            + 224 as ::core::ffi::c_int) as uint16;
                    let mut y_0: uint16 = ((*Get_Botwoon(k as ::core::ffi::c_int)).botwoon_var_23
                        as ::core::ffi::c_int
                        + ((NextRandom() as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
                            - 8 as ::core::ffi::c_int))
                        as uint16;
                    CreateSpriteAtPos(x_0, y_0, 9 as uint16, 2560 as uint16);
                    n -= 1;
                    if !(n != 0) {
                        break;
                    }
                }
            }
        }
        (*E).botwoon_var_23 = (*E).botwoon_var_23.wrapping_add(1);
    } else {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
        SetBossBitForCurArea(2 as uint16);
        QueueMusic_Delayed8(3 as uint16);
    };
}
pub unsafe fn Botwoon_Func_19() {
    let mut pt: Point16U = Botwoon_Func_20(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    let mut v1: uint16 = CalculateAngleFromXY(pt.x, pt.y);
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).botwoon_var_39 = v1;
    (*E).botwoon_var_3A = (64 as ::core::ffi::c_int - v1 as ::core::ffi::c_int) as uint8 as uint16;
    if (*E).botwoon_var_33 as ::core::ffi::c_int == (*E).botwoon_var_34 as ::core::ffi::c_int {
        Botwoon_Func_21();
    } else {
        (*E).botwoon_var_34 = (*E).botwoon_var_33;
        (*E).botwoon_var_3C = 1 as uint16;
    };
}
pub unsafe fn Botwoon_Func_20(mut k: uint16) -> Point16U {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(k as ::core::ffi::c_int);
    let mut v2: ::core::ffi::c_int =
        (*E).botwoon_var_37 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v3: uint16 = (*(g_rom.offset(
        (((0xb3949b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb3949b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v2 as isize) as ::core::ffi::c_int
        + 4 as ::core::ffi::c_int
        - (*E).base.x_pos as ::core::ffi::c_int) as uint16;
    let mut r18: uint16 = v3;
    if v3 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        if v3 as ::core::ffi::c_int + 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        {
            r18 = -(255 as ::core::ffi::c_int) as uint16;
        }
    } else {
        r18 = 255 as uint16;
    }
    let mut v4: uint16 = (*(g_rom.offset(
        (((0xb3949b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb3949b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v2 + 2 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        + 4 as ::core::ffi::c_int
        - (*E).base.y_pos as ::core::ffi::c_int) as uint16;
    let mut r20: uint16 = v4;
    if v4 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        if v4 as ::core::ffi::c_int + 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        {
            r20 = -(255 as ::core::ffi::c_int) as uint16;
        }
    } else {
        r20 = 255 as uint16;
    }
    return Point16U { x: r18, y: r20 };
}
pub unsafe fn Botwoon_Func_21() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut pt: Point32 = ConvertAngleToXy((*E).botwoon_var_3A, (*E).botwoon_var_38);
    EnemyFunc_B691((*E).botwoon_var_3A, pt);
}
pub unsafe fn Botwoon_Func_22() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut botwoon_var_B: uint16 = (*E).botwoon_var_B;
    *(((&raw mut g_ram as *mut uint8).offset(0x9000 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(botwoon_var_B as ::core::ffi::c_int as isize) as *mut uint16) = (*E).base.x_pos;
    *(((&raw mut g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(botwoon_var_B as ::core::ffi::c_int as isize) as *mut uint16) = (*E).base.y_pos;
}
pub unsafe fn Botwoon_Func_23() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut r18: uint16 = ((*E).botwoon_var_B as ::core::ffi::c_int
        - (*E).botwoon_var_C as ::core::ffi::c_int
        & 0x3ff as ::core::ffi::c_int) as uint16;
    let mut n: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
    loop {
        let mut ET: *mut Enemy_Botwoon = Get_Botwoon(n);
        let mut botwoo_var_00: uint16 = (*ET).botwoon_var_00;
        if (*E).botwoon_var_36 as ::core::ffi::c_int == r18 as ::core::ffi::c_int {
            let mut v2: uint16 = n as uint16;
            let mut v4: bool =
                (*ET).botwoon_var_10 as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
            (*ET).botwoon_var_10 =
                ((*ET).botwoon_var_10 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
            if v4 {
                let ref mut fresh0 = *((&raw mut g_ram as *mut uint8)
                    .offset(0x1bd7 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(
                        (botwoo_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                    );
                *fresh0 =
                    (*fresh0 as ::core::ffi::c_int & !(0x2000 as ::core::ffi::c_int)) as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xf380 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(
                        (botwoo_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                    ) = 1 as uint16;
            } else {
                let ref mut fresh1 = *((&raw mut g_ram as *mut uint8)
                    .offset(0x1bd7 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(
                        (botwoo_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                    );
                *fresh1 = (*fresh1 as ::core::ffi::c_int | 0x2000 as ::core::ffi::c_int) as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xf380 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(
                        (botwoo_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                    ) = 2 as uint16;
            }
            if v2 == 0 {
                (*E).botwoon_var_35 = 0 as uint16;
                (*E).botwoon_var_3E = 0 as uint16;
                (*E).botwoon_var_36 = -(1 as ::core::ffi::c_int) as uint16;
                if (*E).botwoon_var_33 == 0 {
                    (*E).botwoon_var_3E = 1 as uint16;
                }
            }
        }
        let mut v7: ::core::ffi::c_int =
            botwoo_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v7 as isize) =
            *(((&raw mut g_ram as *mut uint8).offset(0x9000 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(r18 as ::core::ffi::c_int as isize) as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v7 as isize) =
            *(((&raw mut g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(r18 as ::core::ffi::c_int as isize) as *mut uint16);
        r18 = (r18 as ::core::ffi::c_int - (*E).botwoon_var_C as ::core::ffi::c_int
            & 0x3ff as ::core::ffi::c_int) as uint16;
        n -= 2 as ::core::ffi::c_int;
        if !(n & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
pub unsafe fn Botwoon_Func_24() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).botwoon_var_B = ((*E).botwoon_var_B as ::core::ffi::c_int + 4 as ::core::ffi::c_int
        & 0x3ff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Botwoon_Func_25() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut n: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
    loop {
        let mut r24: uint16 = 0 as uint16;
        let mut ERR: *mut Enemy_Botwoon = Get_Botwoon(n);
        let mut botwoo_var_00: uint16 = (*ERR).botwoon_var_00;
        if (*ERR).botwoon_var_10 != 0 {
            r24 = 256 as uint16;
        }
        let mut x: uint16 = 0;
        let mut y: uint16 = 0;
        if n == 24 as ::core::ffi::c_int {
            let mut v4: ::core::ffi::c_int =
                botwoo_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            x = ((*E).base.x_pos as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v4 as isize) as ::core::ffi::c_int) as uint16;
            y = ((*E).base.y_pos as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v4 as isize) as ::core::ffi::c_int) as uint16;
        } else {
            if n == 0 {
                r24 = (r24 as ::core::ffi::c_int + 512 as ::core::ffi::c_int) as uint16;
            }
            let mut v2: ::core::ffi::c_int =
                botwoo_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            x = (*((&raw mut g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v2 + 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
            y = (*((&raw mut g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v2 + 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((botwoo_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (2 as ::core::ffi::c_int
                * ((r24 as ::core::ffi::c_int + CalculateAngleFromXY(x, y) as ::core::ffi::c_int)
                    as uint16 as ::core::ffi::c_int
                    >> 5 as ::core::ffi::c_int)) as uint16;
        n -= 2 as ::core::ffi::c_int;
        if !(n >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
pub unsafe fn Botwoon_Func_26(mut k: uint16) {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut x: uint16 = ((*E).base.x_pos as ::core::ffi::c_int
        - (*E).botwoon_var_56 as ::core::ffi::c_int) as uint16;
    let mut y: uint16 = ((*E).base.y_pos as ::core::ffi::c_int
        - (*E).botwoon_var_57 as ::core::ffi::c_int) as uint16;
    let mut r22: uint16 = CalculateAngleFromXY(x, y);
    if x as ::core::ffi::c_int != 0 || y as ::core::ffi::c_int != 0 {
        let mut v1: uint16 = 0;
        if (*E).botwoon_var_33 != 0 {
            (*E).base.layer = 7 as uint16;
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                | kEnemyProps_Tangible as ::core::ffi::c_int)
                as uint16;
            v1 = addr_kBotwoon_Ilist_9389 as ::core::ffi::c_int as uint16;
        } else {
            (*E).base.layer = 2 as uint16;
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Tangible as ::core::ffi::c_int))
                as uint16;
            (*E).botwoon_var_45 = ((*E).botwoon_var_45 as ::core::ffi::c_int
                + (r22 as ::core::ffi::c_int - (*E).botwoon_var_45 as ::core::ffi::c_int) as int8
                    as ::core::ffi::c_int
                    * 3 as ::core::ffi::c_int
                    / 4 as ::core::ffi::c_int) as uint8 as uint16;
            v1 = *(g_rom.offset(
                (((0xb3946b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xb3946b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    ((*E).botwoon_var_45 as ::core::ffi::c_int >> 5 as ::core::ffi::c_int) as isize,
                );
        }
        if v1 as ::core::ffi::c_int != (*E).botwoon_var_3B as ::core::ffi::c_int {
            (*E).base.current_instruction = v1;
            (*E).botwoon_var_3B = v1;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).base.timer = 0 as uint16;
        }
    }
    (*E).botwoon_var_56 = (*E).botwoon_var_54;
    (*E).botwoon_var_57 = (*E).botwoon_var_55;
    (*E).botwoon_var_54 = (*E).botwoon_var_52;
    (*E).botwoon_var_55 = (*E).botwoon_var_53;
    (*E).botwoon_var_52 = (*E).botwoon_var_50;
    (*E).botwoon_var_53 = (*E).botwoon_var_51;
    (*E).botwoon_var_50 = (*E).base.x_pos;
    (*E).botwoon_var_51 = (*E).base.y_pos;
}
pub unsafe fn Botwoon_Func_27(mut k: uint16) {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.layer = 2 as uint16;
    let mut v2: uint16 = CalculateAngleOfSamusFromEnemy(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    (*E).botwoon_var_3D = v2;
    let mut v3: uint16 = *(g_rom.offset(
        (((0xb3948b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb3948b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((v2 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint8 as ::core::ffi::c_int
                >> 5 as ::core::ffi::c_int) as isize,
        );
    (*E).base.current_instruction = v3;
    (*E).botwoon_var_3B = v3;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).botwoon_var_3D =
        (64 as ::core::ffi::c_int - (*E).botwoon_var_3D as ::core::ffi::c_int) as uint8 as uint16;
    (*E).botwoon_var_F = (0xb39ee0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if (*E).botwoon_var_D as ::core::ffi::c_int != 0x99e4 as ::core::ffi::c_int {
        (*E).botwoon_var_F =
            (0xb39f34 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    CallEnemyPreInstr(
        ((*E).botwoon_var_F as ::core::ffi::c_int | 0xb30000 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn Botwoon_Func_28(mut k: uint16) {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).botwoon_var_5A != 0 {
        *(&raw mut g_ram as *mut uint8).offset(0x60b as ::core::ffi::c_int as isize) =
            ((*E).botwoon_var_3D as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint8;
        let mut n: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
        let mut varE32: uint16 = g_word_B39E77[(*E).botwoon_var_3F as usize] as uint16;
        loop {
            SpawnEprojWithGfx(
                varE32,
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                addr_kEproj_BotwoonsSpit as ::core::ffi::c_int as uint16,
            );
            let ref mut fresh2 =
                *(&raw mut g_ram as *mut uint8).offset(0x60b as ::core::ffi::c_int as isize);
            *fresh2 = (*fresh2 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint8;
            n -= 1;
            if !(n != 0) {
                break;
            }
        }
        (*E).botwoon_var_5A = 0 as uint16;
        (*E).botwoon_var_F =
            (0xb39f7a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Botwoon_Func_29(mut k: uint16) {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    *(&raw mut g_ram as *mut uint8).offset(0x60b as ::core::ffi::c_int as isize) =
        ((*E).botwoon_var_3D as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint8;
    let mut n: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
    let mut varE32: uint16 = g_word_B39E77[(*E).botwoon_var_3F as usize] as uint16;
    loop {
        SpawnEprojWithGfx(
            varE32,
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            addr_kEproj_BotwoonsSpit as ::core::ffi::c_int as uint16,
        );
        let ref mut fresh3 =
            *(&raw mut g_ram as *mut uint8).offset(0x60b as ::core::ffi::c_int as isize);
        *fresh3 = (*fresh3 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint8;
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
    (*Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .botwoon_var_F = (0xb39f7a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Botwoon_Func_30(mut k: uint16) {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(k as ::core::ffi::c_int);
    let mut v2: int16 =
        ((*E).botwoon_var_21 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
    (*E).botwoon_var_21 = v2 as uint16;
    if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).botwoon_var_21 = 0 as uint16;
        (*E).botwoon_var_F =
            (0xb39dc0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Botwoon_Func_31(mut k: uint16) {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*Get_Botwoon(k as ::core::ffi::c_int)).botwoon_var_35 == 0 {
        let mut n: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
        loop {
            let mut v2: ::core::ffi::c_int = n >> 1 as ::core::ffi::c_int;
            if ((*E).base.x_pos as ::core::ffi::c_int
                - *(g_rom.offset(
                    (((0xb3949b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xb3949b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
                && (((*E).base.x_pos as ::core::ffi::c_int
                    - *(g_rom.offset(
                        (((0xb3949b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xb3949b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset((v2 + 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
                && ((*E).base.y_pos as ::core::ffi::c_int
                    - *(g_rom.offset(
                        (((0xb3949b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xb3949b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset((v2 + 2 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int
                && (((*E).base.y_pos as ::core::ffi::c_int
                    - *(g_rom.offset(
                        (((0xb3949b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xb3949b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset((v2 + 3 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
            {
                (*E).botwoon_var_35 = 1 as uint16;
                (*E).botwoon_var_33 =
                    ((*E).botwoon_var_33 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
                (*E).botwoon_var_36 = (*E).botwoon_var_B;
                (*E).botwoon_var_26 = (*E).botwoon_var_26.wrapping_sub(1);
                break;
            } else {
                (*E).botwoon_var_35 = 0 as uint16;
                n -= 8 as ::core::ffi::c_int;
                if !(n >= 0 as ::core::ffi::c_int) {
                    break;
                }
            }
        }
    }
}
pub unsafe fn Botwoon_Touch() {
    Enemy_NormalTouchAI_SkipDeathAnim_B3();
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).base.health == 0 {
        (*E).botwoon_var_2F = 1 as uint16;
        Botwoon_Func_4();
    }
}
pub unsafe fn Botwoon_Shot() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).botwoon_var_4C = (*E).base.health;
    Enemy_NormalShotAI_SkipSomeParts_B3();
    if (*E).base.health == 0 {
        (*E).botwoon_var_2F = 1 as uint16;
        Botwoon_Func_4();
    }
}
pub unsafe fn Botwoon_Powerbomb() {
    Enemy_NormalPowerBombAI_SkipDeathAnim_B3();
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).base.health == 0 {
        (*E).botwoon_var_2F = 1 as uint16;
        Botwoon_Func_4();
    }
}
pub unsafe fn Botwoon_Func_32() {
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).botwoon_var_E = addr_loc_B3E28C as ::core::ffi::c_int as uint16;
    (*E).botwoon_var_3C = 0 as uint16;
    let mut v1: *const uint8 = RomPtr_B3(
        ((*E).botwoon_var_40 as ::core::ffi::c_int + addr_stru_B3E150 as ::core::ffi::c_int)
            as uint16_t,
    );
    (*E).botwoon_var_42 = *(v1 as *mut uint16);
    (*E).botwoon_var_37 = *(v1.offset(4 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: int16 = *(v1.offset(2 as ::core::ffi::c_int as isize) as *mut uint16) as int16;
    (*E).botwoon_var_44 = v2 as uint16;
    if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).botwoon_var_42 =
            ((*E).botwoon_var_42 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    }
    Botwoon_Func_33();
}
pub unsafe fn Botwoon_Func_33() {
    let mut x: uint16 = 0 as uint16;
    let mut y: uint16 = 0 as uint16;
    let mut E: *mut Enemy_Botwoon = Get_Botwoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut n: ::core::ffi::c_int = (*E).botwoon_var_38 as ::core::ffi::c_int;
    let mut step: ::core::ffi::c_int = if (*E).botwoon_var_44 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        2 as ::core::ffi::c_int
    } else {
        -(2 as ::core::ffi::c_int)
    };
    loop {
        let mut v6: *const uint8 = RomPtr_B3((*E).botwoon_var_42 as uint16_t);
        let mut v7: uint16 = SignExtend8(*v6 as uint16);
        let mut v8: uint16 = 0;
        if v7 as ::core::ffi::c_int == 0xff80 as ::core::ffi::c_int || {
            x = (x as ::core::ffi::c_int + v7 as ::core::ffi::c_int) as uint16;
            v8 = SignExtend8(*v6.offset(1 as ::core::ffi::c_int as isize) as uint16);
            v8 as ::core::ffi::c_int == 0xff80 as ::core::ffi::c_int
        } {
            (*E).botwoon_var_41 = 0 as uint16;
            (*E).botwoon_var_3C = 1 as uint16;
            return;
        }
        y = (y as ::core::ffi::c_int + v8 as ::core::ffi::c_int) as uint16;
        (*E).botwoon_var_42 = ((*E).botwoon_var_42 as ::core::ffi::c_int + step) as uint16;
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
    if (*E).botwoon_var_44 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        x = -(x as ::core::ffi::c_int) as uint16;
        y = -(y as ::core::ffi::c_int) as uint16;
    }
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int + x as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int + y as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EscapeEtecoon_Instr_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    if *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >= 0xce as ::core::ffi::c_int
    {
        return jp.offset(1 as ::core::ffi::c_int as isize);
    } else {
        return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
    };
}
pub unsafe fn EscapeEtecoon_Instr_2(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_EscapeEtecoon = Get_EscapeEtecoon(k as ::core::ffi::c_int);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
        + *jp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        as uint16;
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn EscapeEtecoon_Main() {
    let mut E: *mut Enemy_EscapeEtecoon = Get_EscapeEtecoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallEnemyPreInstr(
        ((*E).een_var_F as ::core::ffi::c_int | 0xb30000 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn EscapeEtecoon_E65C(mut k: uint16) {
    let mut E: *mut Enemy_EscapeEtecoon = Get_EscapeEtecoon(k as ::core::ffi::c_int);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        0x38000 as uint32,
    );
}
pub unsafe fn EscapeEtecoon_E670(mut k: uint16) {
    if CheckEventHappened(0xf as uint16) != 0 {
        (*Get_EscapeEtecoon(k as ::core::ffi::c_int))
            .base
            .current_instruction = addr_kEscapeEtecoon_Ilist_E5DA as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn EscapeEtecoon_E680(mut k: uint16) {
    let mut E: *mut Enemy_EscapeEtecoon = Get_EscapeEtecoon(k as ::core::ffi::c_int);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).een_var_A as int16 as int32) << 8 as ::core::ffi::c_int,
    ) != 0
    {
        (*E).base.instruction_timer = 1 as uint16;
        let mut v3: bool = -((*E).een_var_A as ::core::ffi::c_int) & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
        (*E).een_var_A = -((*E).een_var_A as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction = (if v3 as ::core::ffi::c_int != 0 {
            addr_kEscapeEtecoon_Ilist_E556 as ::core::ffi::c_int
        } else {
            addr_kEscapeEtecoon_Ilist_E582 as ::core::ffi::c_int
        }) as uint16;
        if CheckEventHappened(0xf as uint16) != 0 {
            (*E).base.current_instruction =
                addr_kEscapeEtecoon_Ilist_E5AE as ::core::ffi::c_int as uint16;
        }
    }
    Enemy_MoveDown(
        k,
        (1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
    );
}
pub unsafe fn EscapeEtecoon_Init() {
    let mut E: *mut Enemy_EscapeEtecoon = Get_EscapeEtecoon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if CheckEventHappened(0xf as uint16) != 0 {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
    } else {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | (kEnemyProps_DisableSamusColl as ::core::ffi::c_int
                | kEnemyProps_Tangible as ::core::ffi::c_int
                | 0x8000 as ::core::ffi::c_int)) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.palette_index = 0 as uint16;
        let mut v3: ::core::ffi::c_int =
            (*E).een_parameter_1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        (*E).base.x_pos = *(g_rom.offset(
            (((0xb3e718 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xb3e718 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
        (*E).base.y_pos = *(g_rom.offset(
            (((0xb3e71e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xb3e71e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
        (*E).een_var_F = *(g_rom.offset(
            (((0xb3e724 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xb3e724 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
        (*E).base.current_instruction = *(g_rom.offset(
            (((0xb3e72a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xb3e72a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
        (*E).een_var_A = *(g_rom.offset(
            (((0xb3e730 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xb3e730 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
    };
}
pub unsafe fn EscapeDachora_Instr_2(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    if *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >= 0xce as ::core::ffi::c_int
    {
        return jp.offset(1 as ::core::ffi::c_int as isize);
    } else {
        return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
    };
}
pub unsafe fn EscapeDachora_Instr_3(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    if CheckEventHappened(0xf as uint16) != 0 {
        return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
    } else {
        return jp.offset(1 as ::core::ffi::c_int as isize);
    };
}
pub unsafe fn EscapeDachora_Instr_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_EscapeDachora = Get_EscapeDachora(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int - 6 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn EscapeDachora_Instr_4(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_EscapeDachora = Get_EscapeDachora(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn EscapeDachora_Init() {
    let mut E: *mut Enemy_EscapeDachora = Get_EscapeDachora(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if CheckEventHappened(0xf as uint16) != 0 {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
    } else {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_DisableSamusColl as ::core::ffi::c_int)
            as uint16;
        (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_B3 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction =
            addr_kEscapeDachora_Ilist_E964 as ::core::ffi::c_int as uint16;
    };
}
