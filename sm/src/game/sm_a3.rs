use crate::cpu_infra::Call;
use crate::game::sm_80::{
    Multiply16x16, NextRandom, QueueSfx1_Max6, QueueSfx2_Max3, QueueSfx2_Max6, QueueSfx3_Max6,
};
use crate::game::sm_86::SpawnEprojWithGfx;
use crate::game::sm_90::{CallSomeSamusCode, ProjectileReflection, ResetProjectileData};
use crate::game::sm_91::Samus_DealDamage;
use crate::game::sm_a0::{
    Abs16, CalculateAngleOfSamusFromEnemy, CalculateBlockContainingPixelPos, CallEnemyPreInstr,
    CheckIfEnemyIsOnScreen, CheckIfEnemyTouchesSamus, ConvertAngleToXy, CosineMult8bit,
    CosineMult8bitFull, DetermineDirectionOfSamusFromEnemy, EnemyDeathAnimation, EnemyFunc_B691,
    EnemyFunc_BBBF, EnemyFunc_BC76, EnemyFunc_BF8A, EnemyFunc_C8AD, EnemyGrappleDeath,
    Enemy_ItemDrop_Metroid, Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes,
    Enemy_MoveRight_ProcessSlopes, Enemy_SwitchToFrozenAi, GetSamusEnemyDelta_X,
    GetSamusEnemyDelta_Y, IsSamusWithinEnemy_X, IsSamusWithinEnemy_Y, NormalEnemyFrozenAI,
    NormalEnemyPowerBombAi, NormalEnemyShotAi, NormalEnemyShotAiSkipDeathAnim_CurEnemy,
    NormalEnemyTouchAi, SignExtend8, SineMult8bit, SineMult8bitFull, SwitchEnemyAiToMainAi,
};
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::rtl::{g_ram, g_rom, RomPtr, SnesDivide, Unreachable};
extern "C" {
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, EnemyBase,
    EnemyData, EnemyProps, EnemySpawnData, Func_V, Func_Y_V, LongPtr, VoidP,
};
pub const kEnemyProps_RespawnIfKilled: EnemyProps = 16384;
pub const kEnemyProps_DisableSamusColl: EnemyProps = 8192;
pub const kEnemyProps_BlockPlasmaBeam: EnemyProps = 4096;
pub const kEnemyProps_ProcessedOffscreen: EnemyProps = 2048;
pub const kEnemyProps_Tangible: EnemyProps = 1024;
pub const kEnemyProps_Deleted: EnemyProps = 512;
pub const kEnemyProps_Invisible: EnemyProps = 256;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct MaridiaSnailData2 {
    pub field_0: VoidP,
    pub field_2: uint16,
    pub field_4: VoidP,
    pub field_6: uint16,
}
pub use crate::types::{Consts_86, EnemyDef};
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
pub use crate::types::Consts_A3;
pub const addr_kMetroid_Ilist_EA25: Consts_A3 = 59941;
pub const addr_kMetroid_Ilist_E9CF: Consts_A3 = 59855;
pub const addr_locret_A3E08A: Consts_A3 = 57482;
pub const addr_kWreckedShipOrangeZoomer_Ilist_E01F: Consts_A3 = 57375;
pub const addr_kWreckedShipOrangeZoomer_Ilist_E003: Consts_A3 = 57347;
pub const addr_kWreckedShipOrangeZoomer_Ilist_DFE7: Consts_A3 = 57319;
pub const addr_kWreckedShipOrangeZoomer_Ilist_DFCB: Consts_A3 = 57291;
pub const addr_stru_A3CD3A: Consts_A3 = 52538;
pub const addr_stru_A3CD32: Consts_A3 = 52530;
pub const addr_stru_A3CD2A: Consts_A3 = 52522;
pub const addr_stru_A3CD22: Consts_A3 = 52514;
pub const addr_stru_A3CD1A: Consts_A3 = 52506;
pub const addr_stru_A3CD12: Consts_A3 = 52498;
pub const addr_stru_A3CD0A: Consts_A3 = 52490;
pub const addr_stru_A3CD02: Consts_A3 = 52482;
pub const addr_stru_A3CCFA: Consts_A3 = 52474;
pub const addr_stru_A3CCF2: Consts_A3 = 52466;
pub const addr_stru_A3CCEA: Consts_A3 = 52458;
pub const addr_stru_A3CCE2: Consts_A3 = 52450;
pub const addr_kSkree_Ilist_C65E: Consts_A3 = 50782;
pub const addr_kBang_Ilist_B75E: Consts_A3 = 46942;
pub const addr_kMaridiaRefillCandy_Ilist_B3C1: Consts_A3 = 46017;
pub const addr_kMochtroid_Ilist_A759: Consts_A3 = 42841;
pub const addr_kMochtroid_Ilist_A745: Consts_A3 = 42821;
pub const addr_kPlatformThatFallsWithSamus_Sprmap_A015: Consts_A3 = 40981;
pub const addr_kPlatformThatFallsWithSamus_Sprmap_A009: Consts_A3 = 40969;
pub const addr_kPlatformThatFallsWithSamus_Ilist_9C55: Consts_A3 = 40021;
pub const addr_kPlatformThatFallsWithSamus_Ilist_9C3F: Consts_A3 = 39999;
pub const addr_kPlatformThatFallsWithSamus_Ilist_9C29: Consts_A3 = 39977;
pub const addr_kPlatformThatFallsWithSamus_Ilist_9C13: Consts_A3 = 39955;
pub const addr_kPlatformThatFallsWithSamus_Ilist_9BFD: Consts_A3 = 39933;
pub const addr_kPlatformThatFallsWithSamus_Ilist_9BE7: Consts_A3 = 39911;
pub const addr_kPlatformThatFallsWithSamus_Ilist_9BD1: Consts_A3 = 39889;
pub const addr_kPlatformThatFallsWithSamus_Ilist_9BBB: Consts_A3 = 39867;
pub const addr_kElevator_Ilist_94D6: Consts_A3 = 38102;
pub const addr_kMaridiaFish_Ilist_9072: Consts_A3 = 36978;
pub const addr_kMaridiaFish_Ilist_9060: Consts_A3 = 36960;
pub const addr_kMaridiaFish_Ilist_903C: Consts_A3 = 36924;
pub const addr_kMaridiaFish_Ilist_902A: Consts_A3 = 36906;
pub const addr_kFireflea_Ilist_8C2F: Consts_A3 = 35887;
pub const addr_kMetalee_Ilist_8910: Consts_A3 = 35088;
pub const addr_kWaver_Ilist_86A7: Consts_A3 = 34471;
pub const addr_kSpritemap_Nothing_A3: Consts_A3 = 32845;
pub use crate::types::Consts_B4;
pub const addr_stru_B4EC1C: Consts_B4 = 60444;
pub const addr_asc_B4DD89: Consts_B4 = 56713;
pub const addr_kSpriteObject_Ilist_C4B6: Consts_B4 = 50358;
pub const addr_kSpriteObject_Ilist_C3BA: Consts_B4 = 50106;
pub use crate::types::Point32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Bang {
    pub base: EnemyBase,
    pub bang_var_A: uint16,
    pub bang_var_B: uint16,
    pub bang_var_C: uint16,
    pub bang_var_D: uint16,
    pub bang_var_E: uint16,
    pub bang_var_F: uint16,
    pub bang_parameter_1: uint16,
    pub bang_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub bang_var_00: uint16,
    pub bang_var_01: uint16,
    pub bang_var_02: uint16,
    pub bang_var_03: uint16,
    pub bang_var_04: uint16,
    pub bang_var_05: uint16,
    pub bang_var_06: uint16,
    pub bang_var_07: uint16,
    pub bang_var_08: uint16,
    pub bang_var_09: uint16,
    pub bang_var_0A: uint16,
    pub bang_var_0B: uint16,
    pub bang_var_0C: uint16,
    pub bang_var_0D: uint16,
    pub bang_var_0E: uint16,
    pub bang_var_0F: uint16,
    pub bang_var_10: uint16,
    pub bang_var_11: uint16,
    pub bang_var_12: uint16,
    pub bang_var_13: uint16,
    pub bang_var_14: uint16,
    pub bang_var_15: uint16,
    pub bang_var_16: uint16,
    pub bang_var_17: uint16,
    pub bang_var_18: uint16,
    pub bang_var_19: uint16,
    pub bang_var_1A: uint16,
    pub bang_var_1B: uint16,
    pub bang_var_1C: uint16,
    pub bang_var_1D: uint16,
    pub bang_var_1E: uint16,
    pub bang_var_1F: uint16,
    pub pad2: [uint8; 1984],
    pub bang_var_20: uint16,
    pub bang_var_21: uint16,
    pub bang_var_22: uint16,
    pub bang_var_23: uint16,
    pub bang_var_24: uint16,
    pub bang_var_25: uint16,
    pub bang_var_26: uint16,
    pub bang_var_27: uint16,
    pub bang_var_28: uint16,
    pub bang_var_29: uint16,
    pub bang_var_2A: uint16,
    pub bang_var_2B: uint16,
    pub bang_var_2C: uint16,
    pub bang_var_2D: uint16,
    pub bang_var_2E: uint16,
    pub bang_var_2F: uint16,
    pub bang_var_30: uint16,
    pub bang_var_31: uint16,
    pub bang_var_32: uint16,
    pub bang_var_33: uint16,
    pub bang_var_34: uint16,
    pub bang_var_35: uint16,
    pub bang_var_36: uint16,
    pub bang_var_37: uint16,
    pub bang_var_38: uint16,
    pub bang_var_39: uint16,
    pub bang_var_3A: uint16,
    pub bang_var_3B: uint16,
    pub bang_var_3C: uint16,
    pub bang_var_3D: uint16,
    pub bang_var_3E: uint16,
    pub bang_var_3F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MaridiaFish {
    pub base: EnemyBase,
    pub mfh_var_A: uint16,
    pub mfh_var_B: uint16,
    pub mfh_var_C: uint16,
    pub mfh_var_D: uint16,
    pub mfh_var_E: uint16,
    pub mfh_var_F: uint16,
    pub mfh_parameter_1: uint16,
    pub mfh_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub mfh_var_00: uint16,
    pub mfh_var_01: uint16,
    pub mfh_var_02: uint16,
    pub mfh_var_03: uint16,
    pub mfh_var_04: uint16,
    pub mfh_var_05: uint16,
    pub mfh_var_06: uint16,
    pub mfh_var_07: uint16,
    pub mfh_var_08: uint16,
    pub mfh_var_09: uint16,
    pub mfh_var_0A: uint16,
    pub mfh_var_0B: uint16,
    pub mfh_var_0C: uint16,
    pub mfh_var_0D: uint16,
    pub mfh_var_0E: uint16,
    pub mfh_var_0F: uint16,
    pub mfh_var_10: uint16,
    pub mfh_var_11: uint16,
    pub mfh_var_12: uint16,
    pub mfh_var_13: uint16,
    pub mfh_var_14: uint16,
    pub mfh_var_15: uint16,
    pub mfh_var_16: uint16,
    pub mfh_var_17: uint16,
    pub mfh_var_18: uint16,
    pub mfh_var_19: uint16,
    pub mfh_var_1A: uint16,
    pub mfh_var_1B: uint16,
    pub mfh_var_1C: uint16,
    pub mfh_var_1D: uint16,
    pub mfh_var_1E: uint16,
    pub mfh_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MaridiaRefillCandy {
    pub base: EnemyBase,
    pub mrcy_var_A: uint16,
    pub mrcy_var_B: uint16,
    pub mrcy_var_C: uint16,
    pub mrcy_var_D: uint16,
    pub mrcy_var_E: uint16,
    pub mrcy_var_F: uint16,
    pub mrcy_parameter_1: uint16,
    pub mrcy_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub mrcy_var_00: uint16,
    pub mrcy_var_01: uint16,
    pub mrcy_var_02: uint16,
    pub mrcy_var_03: uint16,
    pub mrcy_var_04: uint16,
    pub mrcy_var_05: uint16,
    pub mrcy_var_06: uint16,
    pub mrcy_var_07: uint16,
    pub mrcy_var_08: uint16,
    pub mrcy_var_09: uint16,
    pub mrcy_var_0A: uint16,
    pub mrcy_var_0B: uint16,
    pub mrcy_var_0C: uint16,
    pub mrcy_var_0D: uint16,
    pub mrcy_var_0E: uint16,
    pub mrcy_var_0F: uint16,
    pub mrcy_var_10: uint16,
    pub mrcy_var_11: uint16,
    pub mrcy_var_12: uint16,
    pub mrcy_var_13: uint16,
    pub mrcy_var_14: uint16,
    pub mrcy_var_15: uint16,
    pub mrcy_var_16: uint16,
    pub mrcy_var_17: uint16,
    pub mrcy_var_18: uint16,
    pub mrcy_var_19: uint16,
    pub mrcy_var_1A: uint16,
    pub mrcy_var_1B: uint16,
    pub mrcy_var_1C: uint16,
    pub mrcy_var_1D: uint16,
    pub mrcy_var_1E: uint16,
    pub mrcy_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MaridiaSnail {
    pub base: EnemyBase,
    pub msl_var_A: uint16,
    pub msl_var_B: uint16,
    pub msl_var_C: uint16,
    pub msl_var_D: uint16,
    pub msl_var_E: uint16,
    pub msl_var_F: uint16,
    pub msl_parameter_1: uint16,
    pub msl_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub msl_var_00: uint16,
    pub msl_var_01: uint16,
    pub msl_var_02: uint16,
    pub msl_var_03: uint16,
    pub msl_var_04: uint16,
    pub msl_var_05: uint16,
    pub msl_var_06: uint16,
    pub msl_var_07: uint16,
    pub msl_var_08: uint16,
    pub msl_var_09: uint16,
    pub msl_var_0A: uint16,
    pub msl_var_0B: uint16,
    pub msl_var_0C: uint16,
    pub msl_var_0D: uint16,
    pub msl_var_0E: uint16,
    pub msl_var_0F: uint16,
    pub msl_var_10: uint16,
    pub msl_var_11: uint16,
    pub msl_var_12: uint16,
    pub msl_var_13: uint16,
    pub msl_var_14: uint16,
    pub msl_var_15: uint16,
    pub msl_var_16: uint16,
    pub msl_var_17: uint16,
    pub msl_var_18: uint16,
    pub msl_var_19: uint16,
    pub msl_var_1A: uint16,
    pub msl_var_1B: uint16,
    pub msl_var_1C: uint16,
    pub msl_var_1D: uint16,
    pub msl_var_1E: uint16,
    pub msl_var_1F: uint16,
    pub pad2: [uint8; 1984],
    pub msl_var_20: uint16,
    pub msl_var_21: uint16,
    pub msl_var_22: uint16,
    pub msl_var_23: uint16,
    pub msl_var_24: uint16,
    pub msl_var_25: uint16,
    pub msl_var_26: uint16,
    pub msl_var_27: uint16,
    pub msl_var_28: uint16,
    pub msl_var_29: uint16,
    pub msl_var_2A: uint16,
    pub msl_var_2B: uint16,
    pub msl_var_2C: uint16,
    pub msl_var_2D: uint16,
    pub msl_var_2E: uint16,
    pub msl_var_2F: uint16,
    pub msl_var_30: uint16,
    pub msl_var_31: uint16,
    pub msl_var_32: uint16,
    pub msl_var_33: uint16,
    pub msl_var_34: uint16,
    pub msl_var_35: uint16,
    pub msl_var_36: uint16,
    pub msl_var_37: uint16,
    pub msl_var_38: uint16,
    pub msl_var_39: uint16,
    pub msl_var_3A: uint16,
    pub msl_var_3B: uint16,
    pub msl_var_3C: uint16,
    pub msl_var_3D: uint16,
    pub msl_var_3E: uint16,
    pub msl_var_3F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Metalee {
    pub base: EnemyBase,
    pub metalee_var_A: uint16,
    pub metalee_var_B: uint16,
    pub metalee_var_C: uint16,
    pub metalee_var_D: uint16,
    pub metalee_var_E: uint16,
    pub metalee_var_F: uint16,
    pub metalee_parameter_1: uint16,
    pub metalee_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_PlatformThatFallsWithSamus {
    pub base: EnemyBase,
    pub ptfwss_var_A: uint16,
    pub ptfwss_var_B: uint16,
    pub ptfwss_var_C: uint16,
    pub ptfwss_var_D: uint16,
    pub ptfwss_var_E: uint16,
    pub ptfwss_var_F: uint16,
    pub ptfwss_parameter_1: uint16,
    pub ptfwss_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub ptfwss_var_00: uint16,
    pub ptfwss_var_01: uint16,
    pub ptfwss_var_02: uint16,
    pub ptfwss_var_03: uint16,
    pub ptfwss_var_04: uint16,
    pub ptfwss_var_05: uint16,
    pub ptfwss_var_06: uint16,
    pub ptfwss_var_07: uint16,
    pub ptfwss_var_08: uint16,
    pub ptfwss_var_09: uint16,
    pub ptfwss_var_0A: uint16,
    pub ptfwss_var_0B: uint16,
    pub ptfwss_var_0C: uint16,
    pub ptfwss_var_0D: uint16,
    pub ptfwss_var_0E: uint16,
    pub ptfwss_var_0F: uint16,
    pub ptfwss_var_10: uint16,
    pub ptfwss_var_11: uint16,
    pub ptfwss_var_12: uint16,
    pub ptfwss_var_13: uint16,
    pub ptfwss_var_14: uint16,
    pub ptfwss_var_15: uint16,
    pub ptfwss_var_16: uint16,
    pub ptfwss_var_17: uint16,
    pub ptfwss_var_18: uint16,
    pub ptfwss_var_19: uint16,
    pub ptfwss_var_1A: uint16,
    pub ptfwss_var_1B: uint16,
    pub ptfwss_var_1C: uint16,
    pub ptfwss_var_1D: uint16,
    pub ptfwss_var_1E: uint16,
    pub ptfwss_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Reflec {
    pub base: EnemyBase,
    pub reflec_var_A: uint16,
    pub reflec_var_B: uint16,
    pub reflec_var_C: uint16,
    pub reflec_var_D: uint16,
    pub reflec_var_E: uint16,
    pub reflec_var_F: uint16,
    pub reflec_parameter_1: uint16,
    pub reflec_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Sidehopper {
    pub base: EnemyBase,
    pub sideh_var_A: uint16,
    pub sideh_var_B: uint16,
    pub sideh_var_C: uint16,
    pub sideh_var_D: uint16,
    pub sideh_var_E: uint16,
    pub sideh_var_F: uint16,
    pub sideh_parameter_1: uint16,
    pub sideh_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub sideh_var_00: uint16,
    pub sideh_var_01: uint16,
    pub sideh_var_02: uint16,
    pub sideh_var_03: uint16,
    pub sideh_var_04: uint16,
    pub sideh_var_05: uint16,
    pub sideh_var_06: uint16,
    pub sideh_var_07: uint16,
    pub sideh_var_08: uint16,
    pub sideh_var_09: uint16,
    pub sideh_var_0A: uint16,
    pub sideh_var_0B: uint16,
    pub sideh_var_0C: uint16,
    pub sideh_var_0D: uint16,
    pub sideh_var_0E: uint16,
    pub sideh_var_0F: uint16,
    pub sideh_var_10: uint16,
    pub sideh_var_11: uint16,
    pub sideh_var_12: uint16,
    pub sideh_var_13: uint16,
    pub sideh_var_14: uint16,
    pub sideh_var_15: uint16,
    pub sideh_var_16: uint16,
    pub sideh_var_17: uint16,
    pub sideh_var_18: uint16,
    pub sideh_var_19: uint16,
    pub sideh_var_1A: uint16,
    pub sideh_var_1B: uint16,
    pub sideh_var_1C: uint16,
    pub sideh_var_1D: uint16,
    pub sideh_var_1E: uint16,
    pub sideh_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Skree {
    pub base: EnemyBase,
    pub skree_var_A: uint16,
    pub skree_var_B: uint16,
    pub skree_var_C: uint16,
    pub skree_var_D: uint16,
    pub skree_var_E: uint16,
    pub skree_var_F: uint16,
    pub skree_parameter_1: uint16,
    pub skree_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub skree_var_00: uint16,
    pub skree_var_01: uint16,
    pub skree_var_02: uint16,
    pub skree_var_03: uint16,
    pub skree_var_04: uint16,
    pub skree_var_05: uint16,
    pub skree_var_06: uint16,
    pub skree_var_07: uint16,
    pub skree_var_08: uint16,
    pub skree_var_09: uint16,
    pub skree_var_0A: uint16,
    pub skree_var_0B: uint16,
    pub skree_var_0C: uint16,
    pub skree_var_0D: uint16,
    pub skree_var_0E: uint16,
    pub skree_var_0F: uint16,
    pub skree_var_10: uint16,
    pub skree_var_11: uint16,
    pub skree_var_12: uint16,
    pub skree_var_13: uint16,
    pub skree_var_14: uint16,
    pub skree_var_15: uint16,
    pub skree_var_16: uint16,
    pub skree_var_17: uint16,
    pub skree_var_18: uint16,
    pub skree_var_19: uint16,
    pub skree_var_1A: uint16,
    pub skree_var_1B: uint16,
    pub skree_var_1C: uint16,
    pub skree_var_1D: uint16,
    pub skree_var_1E: uint16,
    pub skree_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Waver {
    pub base: EnemyBase,
    pub waver_var_A: uint16,
    pub waver_var_B: uint16,
    pub waver_var_C: uint16,
    pub waver_var_D: uint16,
    pub waver_var_E: uint16,
    pub waver_var_F: uint16,
    pub waver_parameter_1: uint16,
    pub waver_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_WreckedShipOrangeZoomer {
    pub base: EnemyBase,
    pub wsozr_var_A: uint16,
    pub wsozr_var_B: uint16,
    pub wsozr_var_C: uint16,
    pub wsozr_var_D: uint16,
    pub wsozr_var_E: uint16,
    pub wsozr_var_F: uint16,
    pub wsozr_parameter_1: uint16,
    pub wsozr_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub wsozr_var_00: uint16,
    pub wsozr_var_01: uint16,
    pub wsozr_var_02: uint16,
    pub wsozr_var_03: uint16,
    pub wsozr_var_04: uint16,
    pub wsozr_var_05: uint16,
    pub wsozr_var_06: uint16,
    pub wsozr_var_07: uint16,
    pub wsozr_var_08: uint16,
    pub wsozr_var_09: uint16,
    pub wsozr_var_0A: uint16,
    pub wsozr_var_0B: uint16,
    pub wsozr_var_0C: uint16,
    pub wsozr_var_0D: uint16,
    pub wsozr_var_0E: uint16,
    pub wsozr_var_0F: uint16,
    pub wsozr_var_10: uint16,
    pub wsozr_var_11: uint16,
    pub wsozr_var_12: uint16,
    pub wsozr_var_13: uint16,
    pub wsozr_var_14: uint16,
    pub wsozr_var_15: uint16,
    pub wsozr_var_16: uint16,
    pub wsozr_var_17: uint16,
    pub wsozr_var_18: uint16,
    pub wsozr_var_19: uint16,
    pub wsozr_var_1A: uint16,
    pub wsozr_var_1B: uint16,
    pub wsozr_var_1C: uint16,
    pub wsozr_var_1D: uint16,
    pub wsozr_var_1E: uint16,
    pub wsozr_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_StoneZoomer {
    pub base: EnemyBase,
    pub szr_var_A: uint16,
    pub szr_var_B: uint16,
    pub szr_var_C: uint16,
    pub szr_var_D: uint16,
    pub szr_var_E: uint16,
    pub szr_var_F: uint16,
    pub szr_parameter_1: uint16,
    pub szr_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Elevator {
    pub base: EnemyBase,
    pub elevat_var_A: uint16,
    pub elevat_var_B: uint16,
    pub elevat_var_C: uint16,
    pub elevat_var_D: uint16,
    pub elevat_var_E: uint16,
    pub elevat_var_F: uint16,
    pub elevat_parameter_1: uint16,
    pub elevat_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_FireZoomer {
    pub base: EnemyBase,
    pub fzr_var_A: uint16,
    pub fzr_var_B: uint16,
    pub fzr_var_C: uint16,
    pub fzr_var_D: uint16,
    pub fzr_var_E: uint16,
    pub fzr_var_F: uint16,
    pub fzr_parameter_1: uint16,
    pub fzr_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub fzr_var_00: uint16,
    pub fzr_var_01: uint16,
    pub fzr_var_02: uint16,
    pub fzr_var_03: uint16,
    pub fzr_var_04: uint16,
    pub fzr_var_05: uint16,
    pub fzr_var_06: uint16,
    pub fzr_var_07: uint16,
    pub fzr_var_08: uint16,
    pub fzr_var_09: uint16,
    pub fzr_var_0A: uint16,
    pub fzr_var_0B: uint16,
    pub fzr_var_0C: uint16,
    pub fzr_var_0D: uint16,
    pub fzr_var_0E: uint16,
    pub fzr_var_0F: uint16,
    pub fzr_var_10: uint16,
    pub fzr_var_11: uint16,
    pub fzr_var_12: uint16,
    pub fzr_var_13: uint16,
    pub fzr_var_14: uint16,
    pub fzr_var_15: uint16,
    pub fzr_var_16: uint16,
    pub fzr_var_17: uint16,
    pub fzr_var_18: uint16,
    pub fzr_var_19: uint16,
    pub fzr_var_1A: uint16,
    pub fzr_var_1B: uint16,
    pub fzr_var_1C: uint16,
    pub fzr_var_1D: uint16,
    pub fzr_var_1E: uint16,
    pub fzr_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_FireFlea {
    pub base: EnemyBase,
    pub ffa_var_A: uint16,
    pub ffa_var_B: uint16,
    pub ffa_var_C: uint16,
    pub ffa_var_D: uint16,
    pub ffa_var_E: uint16,
    pub ffa_var_F: uint16,
    pub ffa_parameter_1: uint16,
    pub ffa_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub ffa_var_00: uint16,
    pub ffa_var_01: uint16,
    pub ffa_var_02: uint16,
    pub ffa_var_03: uint16,
    pub ffa_var_04: uint16,
    pub ffa_var_05: uint16,
    pub ffa_var_06: uint16,
    pub ffa_var_07: uint16,
    pub ffa_var_08: uint16,
    pub ffa_var_09: uint16,
    pub ffa_var_0A: uint16,
    pub ffa_var_0B: uint16,
    pub ffa_var_0C: uint16,
    pub ffa_var_0D: uint16,
    pub ffa_var_0E: uint16,
    pub ffa_var_0F: uint16,
    pub ffa_var_10: uint16,
    pub ffa_var_11: uint16,
    pub ffa_var_12: uint16,
    pub ffa_var_13: uint16,
    pub ffa_var_14: uint16,
    pub ffa_var_15: uint16,
    pub ffa_var_16: uint16,
    pub ffa_var_17: uint16,
    pub ffa_var_18: uint16,
    pub ffa_var_19: uint16,
    pub ffa_var_1A: uint16,
    pub ffa_var_1B: uint16,
    pub ffa_var_1C: uint16,
    pub ffa_var_1D: uint16,
    pub ffa_var_1E: uint16,
    pub ffa_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Metroid {
    pub base: EnemyBase,
    pub metroid_var_A: uint16,
    pub metroid_var_B: uint16,
    pub metroid_var_C: uint16,
    pub metroid_var_D: uint16,
    pub metroid_var_E: uint16,
    pub metroid_var_F: uint16,
    pub metroid_parameter_1: uint16,
    pub metroid_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub metroid_var_00: uint16,
    pub metroid_var_01: uint16,
    pub metroid_var_02: uint16,
    pub metroid_var_03: uint16,
    pub metroid_var_04: uint16,
    pub metroid_var_05: uint16,
    pub metroid_var_06: uint16,
    pub metroid_var_07: uint16,
    pub metroid_var_08: uint16,
    pub metroid_var_09: uint16,
    pub metroid_var_0A: uint16,
    pub metroid_var_0B: uint16,
    pub metroid_var_0C: uint16,
    pub metroid_var_0D: uint16,
    pub metroid_var_0E: uint16,
    pub metroid_var_0F: uint16,
    pub metroid_var_10: uint16,
    pub metroid_var_11: uint16,
    pub metroid_var_12: uint16,
    pub metroid_var_13: uint16,
    pub metroid_var_14: uint16,
    pub metroid_var_15: uint16,
    pub metroid_var_16: uint16,
    pub metroid_var_17: uint16,
    pub metroid_var_18: uint16,
    pub metroid_var_19: uint16,
    pub metroid_var_1A: uint16,
    pub metroid_var_1B: uint16,
    pub metroid_var_1C: uint16,
    pub metroid_var_1D: uint16,
    pub metroid_var_1E: uint16,
    pub metroid_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Mochtroid {
    pub base: EnemyBase,
    pub mochtr_var_A: uint16,
    pub mochtr_var_B: uint16,
    pub mochtr_var_C: uint16,
    pub mochtr_var_D: uint16,
    pub mochtr_var_E: uint16,
    pub mochtr_var_F: uint16,
    pub mochtr_parameter_1: uint16,
    pub mochtr_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub mochtr_var_00: uint16,
    pub mochtr_var_01: uint16,
    pub mochtr_var_02: uint16,
    pub mochtr_var_03: uint16,
    pub mochtr_var_04: uint16,
    pub mochtr_var_05: uint16,
    pub mochtr_var_06: uint16,
    pub mochtr_var_07: uint16,
    pub mochtr_var_08: uint16,
    pub mochtr_var_09: uint16,
    pub mochtr_var_0A: uint16,
    pub mochtr_var_0B: uint16,
    pub mochtr_var_0C: uint16,
    pub mochtr_var_0D: uint16,
    pub mochtr_var_0E: uint16,
    pub mochtr_var_0F: uint16,
    pub mochtr_var_10: uint16,
    pub mochtr_var_11: uint16,
    pub mochtr_var_12: uint16,
    pub mochtr_var_13: uint16,
    pub mochtr_var_14: uint16,
    pub mochtr_var_15: uint16,
    pub mochtr_var_16: uint16,
    pub mochtr_var_17: uint16,
    pub mochtr_var_18: uint16,
    pub mochtr_var_19: uint16,
    pub mochtr_var_1A: uint16,
    pub mochtr_var_1B: uint16,
    pub mochtr_var_1C: uint16,
    pub mochtr_var_1D: uint16,
    pub mochtr_var_1E: uint16,
    pub mochtr_var_1F: uint16,
    pub pad2: [uint8; 1984],
    pub mochtr_var_20: uint16,
    pub mochtr_var_21: uint16,
    pub mochtr_var_22: uint16,
    pub mochtr_var_23: uint16,
    pub mochtr_var_24: uint16,
    pub mochtr_var_25: uint16,
    pub mochtr_var_26: uint16,
    pub mochtr_var_27: uint16,
    pub mochtr_var_28: uint16,
    pub mochtr_var_29: uint16,
    pub mochtr_var_2A: uint16,
    pub mochtr_var_2B: uint16,
    pub mochtr_var_2C: uint16,
    pub mochtr_var_2D: uint16,
    pub mochtr_var_2E: uint16,
    pub mochtr_var_2F: uint16,
    pub mochtr_var_30: uint16,
    pub mochtr_var_31: uint16,
    pub mochtr_var_32: uint16,
    pub mochtr_var_33: uint16,
    pub mochtr_var_34: uint16,
    pub mochtr_var_35: uint16,
    pub mochtr_var_36: uint16,
    pub mochtr_var_37: uint16,
    pub mochtr_var_38: uint16,
    pub mochtr_var_39: uint16,
    pub mochtr_var_3A: uint16,
    pub mochtr_var_3B: uint16,
    pub mochtr_var_3C: uint16,
    pub mochtr_var_3D: uint16,
    pub mochtr_var_3E: uint16,
    pub mochtr_var_3F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Roach {
    pub base: EnemyBase,
    pub roach_var_A: uint16,
    pub roach_var_B: uint16,
    pub roach_var_C: uint16,
    pub roach_var_D: uint16,
    pub roach_var_E: uint16,
    pub roach_var_F: uint16,
    pub roach_parameter_1: uint16,
    pub roach_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub roach_var_00: uint16,
    pub roach_var_01: uint16,
    pub roach_var_02: uint16,
    pub roach_var_03: uint16,
    pub roach_var_04: uint16,
    pub roach_var_05: uint16,
    pub roach_var_06: uint16,
    pub roach_var_07: uint16,
    pub roach_var_08: uint16,
    pub roach_var_09: uint16,
    pub roach_var_0A: uint16,
    pub roach_var_0B: uint16,
    pub roach_var_0C: uint16,
    pub roach_var_0D: uint16,
    pub roach_var_0E: uint16,
    pub roach_var_0F: uint16,
    pub roach_var_10: uint16,
    pub roach_var_11: uint16,
    pub roach_var_12: uint16,
    pub roach_var_13: uint16,
    pub roach_var_14: uint16,
    pub roach_var_15: uint16,
    pub roach_var_16: uint16,
    pub roach_var_17: uint16,
    pub roach_var_18: uint16,
    pub roach_var_19: uint16,
    pub roach_var_1A: uint16,
    pub roach_var_1B: uint16,
    pub roach_var_1C: uint16,
    pub roach_var_1D: uint16,
    pub roach_var_1E: uint16,
    pub roach_var_1F: uint16,
    pub pad2: [uint8; 1984],
    pub roach_var_20: uint16,
    pub roach_var_21: uint16,
    pub roach_var_22: uint16,
    pub roach_var_23: uint16,
    pub roach_var_24: uint16,
    pub roach_var_25: uint16,
    pub roach_var_26: uint16,
    pub roach_var_27: uint16,
    pub roach_var_28: uint16,
    pub roach_var_29: uint16,
    pub roach_var_2A: uint16,
    pub roach_var_2B: uint16,
    pub roach_var_2C: uint16,
    pub roach_var_2D: uint16,
    pub roach_var_2E: uint16,
    pub roach_var_2F: uint16,
    pub roach_var_30: uint16,
    pub roach_var_31: uint16,
    pub roach_var_32: uint16,
    pub roach_var_33: uint16,
    pub roach_var_34: uint16,
    pub roach_var_35: uint16,
    pub roach_var_36: uint16,
    pub roach_var_37: uint16,
    pub roach_var_38: uint16,
    pub roach_var_39: uint16,
    pub roach_var_3A: uint16,
    pub roach_var_3B: uint16,
    pub roach_var_3C: uint16,
    pub roach_var_3D: uint16,
    pub roach_var_3E: uint16,
    pub roach_var_3F: uint16,
}
#[inline]
unsafe extern "C" fn abs16(mut t: uint16) -> uint16 {
    return (if t as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(t as ::core::ffi::c_int)
    } else {
        t as ::core::ffi::c_int
    }) as uint16;
}
#[inline]
unsafe extern "C" fn RomPtr_A3(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0xa30000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
#[inline]
unsafe extern "C" fn get_EnemyDef_A2(mut a: uint16) -> *mut EnemyDef {
    return RomPtr((0xa00000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t)
        as *mut EnemyDef;
}
#[inline]
unsafe extern "C" fn AddToHiLo(mut hi: *mut uint16, mut lo: *mut uint16, mut v: uint32) {
    let mut t: uint32 = ((*lo as ::core::ffi::c_int
        + ((*hi as ::core::ffi::c_int) << 16 as ::core::ffi::c_int))
        as uint32)
        .wrapping_add(v);
    *lo = t as uint16;
    *hi = (t >> 16 as ::core::ffi::c_int) as uint16;
}
#[inline]
unsafe extern "C" fn SetHiLo(mut hi: *mut uint16, mut lo: *mut uint16, mut t: uint32) {
    *lo = t as uint16;
    *hi = (t >> 16 as ::core::ffi::c_int) as uint16;
}
pub const fnMetalee_Func_1: ::core::ffi::c_int = 0xa38987 as ::core::ffi::c_int;
pub const fnMetalee_Func_3: ::core::ffi::c_int = 0xa389d4 as ::core::ffi::c_int;
pub const fnMetalee_Func_4: ::core::ffi::c_int = 0xa389f3 as ::core::ffi::c_int;
pub const fnMetalee_Func_5: ::core::ffi::c_int = 0xa38a5c as ::core::ffi::c_int;
pub const fnMaridiaFish_Func_1: ::core::ffi::c_int = 0xa39132 as ::core::ffi::c_int;
pub const fnMaridiaFish_Func_2: ::core::ffi::c_int = 0xa391ab as ::core::ffi::c_int;
pub const fnMaridiaFish_Func_3: ::core::ffi::c_int = 0xa39224 as ::core::ffi::c_int;
pub const fnMaridiaFish_Func_4: ::core::ffi::c_int = 0xa39256 as ::core::ffi::c_int;
pub const fnRoach_Func_9: ::core::ffi::c_int = 0xa3a2d7 as ::core::ffi::c_int;
pub const fnRoach_Func_10: uint32 = 10724097 as uint32;
pub const fnRoach_Func_11: uint32 = 10724107 as uint32;
pub const fnRoach_Func_12: uint32 = 10724117 as uint32;
pub const fnRoach_Func_13: uint32 = 10724133 as uint32;
pub const fnRoach_Func_14: uint32 = 10724155 as uint32;
pub const fnRoach_Func_15: uint32 = 10724171 as uint32;
pub const fnRoach_Func_16: uint32 = 10724224 as uint32;
pub const fnRoach_Func_19: ::core::ffi::c_int = 0xa3a407 as ::core::ffi::c_int;
pub const fnRoach_Func_20: ::core::ffi::c_int = 0xa3a40e as ::core::ffi::c_int;
pub const fnRoach_Func_21: ::core::ffi::c_int = 0xa3a440 as ::core::ffi::c_int;
pub const fnRoach_Func_22: ::core::ffi::c_int = 0xa3a447 as ::core::ffi::c_int;
pub const fnRoach_Func_23: ::core::ffi::c_int = 0xa3a44e as ::core::ffi::c_int;
pub const fnRoach_Func_24: ::core::ffi::c_int = 0xa3a462 as ::core::ffi::c_int;
pub const fnRoach_Func_25: ::core::ffi::c_int = 0xa3a476 as ::core::ffi::c_int;
pub const fnRoach_Func_26: ::core::ffi::c_int = 0xa3a4b6 as ::core::ffi::c_int;
pub const fnRoach_Func_27: ::core::ffi::c_int = 0xa3a4f0 as ::core::ffi::c_int;
pub const fnSidehopper_Func_4: ::core::ffi::c_int = 0xa3abd6 as ::core::ffi::c_int;
pub const fnSidehopper_Func_5: uint32 = 10726374 as uint32;
pub const fnSidehopper_Func_6: uint32 = 10726419 as uint32;
pub const fnSidehopper_Func_7: ::core::ffi::c_int = 0xa3ac40 as ::core::ffi::c_int;
pub const fnSidehopper_Func_8: ::core::ffi::c_int = 0xa3ac56 as ::core::ffi::c_int;
pub const fnSidehopper_Func_9: ::core::ffi::c_int = 0xa3ac6c as ::core::ffi::c_int;
pub const fnSidehopper_Func_10: ::core::ffi::c_int = 0xa3ac8f as ::core::ffi::c_int;
pub const fnSidehopper_Func_11: ::core::ffi::c_int = 0xa3aca8 as ::core::ffi::c_int;
pub const fnSidehopper_Func_12: ::core::ffi::c_int = 0xa3accb as ::core::ffi::c_int;
pub const fnSidehopper_Func_13: ::core::ffi::c_int = 0xa3ace4 as ::core::ffi::c_int;
pub const fnSidehopper_Func_14: ::core::ffi::c_int = 0xa3ad0e as ::core::ffi::c_int;
pub const fnSidehopper_Func_15: ::core::ffi::c_int = 0xa3ad20 as ::core::ffi::c_int;
pub const fnSidehopper_Func_16: ::core::ffi::c_int = 0xa3ad32 as ::core::ffi::c_int;
pub const fnSidehopper_Func_17: ::core::ffi::c_int = 0xa3ad44 as ::core::ffi::c_int;
pub const fnSidehopper_Func_18: ::core::ffi::c_int = 0xa3ad56 as ::core::ffi::c_int;
pub const fnMaridiaRefillCandy_Func_1: ::core::ffi::c_int = 0xa3b482 as ::core::ffi::c_int;
pub const fnMaridiaRefillCandy_Func_2: ::core::ffi::c_int = 0xa3b4a8 as ::core::ffi::c_int;
pub const fnMaridiaRefillCandy_Func_3: ::core::ffi::c_int = 0xa3b4d6 as ::core::ffi::c_int;
pub const fnBang_Func_6: ::core::ffi::c_int = 0xa3bca5 as ::core::ffi::c_int;
pub const fnBang_Func_7: ::core::ffi::c_int = 0xa3bcc1 as ::core::ffi::c_int;
pub const fnBang_Func_8: ::core::ffi::c_int = 0xa3bcc5 as ::core::ffi::c_int;
pub const fnBang_Func_10: ::core::ffi::c_int = 0xa3bd1c as ::core::ffi::c_int;
pub const fnBang_Func_11: ::core::ffi::c_int = 0xa3bd2c as ::core::ffi::c_int;
pub const fnSkree_Func_1: ::core::ffi::c_int = 0xa3c6d5 as ::core::ffi::c_int;
pub const fnSkree_Func_2: ::core::ffi::c_int = 0xa3c6f7 as ::core::ffi::c_int;
pub const fnSkree_Func_3: ::core::ffi::c_int = 0xa3c716 as ::core::ffi::c_int;
pub const fnSkree_Func_4: ::core::ffi::c_int = 0xa3c77f as ::core::ffi::c_int;
pub const fnnullsub_215: ::core::ffi::c_int = 0xa3cf5f as ::core::ffi::c_int;
pub const fnMaridiaSnail_Func_15: ::core::ffi::c_int = 0xa3d1b3 as ::core::ffi::c_int;
pub const fnReflec_Func_1: ::core::ffi::c_int = 0xa3db0c as ::core::ffi::c_int;
pub const fnnullsub_304: ::core::ffi::c_int = 0xa3e6c1 as ::core::ffi::c_int;
pub const fnFireZoomer_Func_1: ::core::ffi::c_int = 0xa3e6c8 as ::core::ffi::c_int;
pub const fnFireZoomer_Func_2: ::core::ffi::c_int = 0xa3e785 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn gEnemyData(mut o: ::core::ffi::c_int) -> *mut EnemyData {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut EnemyData;
}
#[inline]
unsafe extern "C" fn gEnemySpawnData(mut o: ::core::ffi::c_int) -> *mut EnemySpawnData {
    return (&raw mut g_ram as *mut uint8)
        .offset(0x7000 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut EnemySpawnData;
}
#[inline]
unsafe extern "C" fn Get_Bang(mut o: ::core::ffi::c_int) -> *mut Enemy_Bang {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_Bang;
}
#[inline]
unsafe extern "C" fn Get_Elevator(mut o: ::core::ffi::c_int) -> *mut Enemy_Elevator {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_Elevator;
}
#[inline]
unsafe extern "C" fn Get_FireFlea(mut o: ::core::ffi::c_int) -> *mut Enemy_FireFlea {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_FireFlea;
}
#[inline]
unsafe extern "C" fn Get_FireZoomer(mut o: ::core::ffi::c_int) -> *mut Enemy_FireZoomer {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_FireZoomer;
}
#[inline]
unsafe extern "C" fn Get_MaridiaFish(mut o: ::core::ffi::c_int) -> *mut Enemy_MaridiaFish {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_MaridiaFish;
}
#[inline]
unsafe extern "C" fn Get_MaridiaRefillCandy(
    mut o: ::core::ffi::c_int,
) -> *mut Enemy_MaridiaRefillCandy {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_MaridiaRefillCandy;
}
#[inline]
unsafe extern "C" fn Get_MaridiaSnail(mut o: ::core::ffi::c_int) -> *mut Enemy_MaridiaSnail {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_MaridiaSnail;
}
#[inline]
unsafe extern "C" fn Get_Metalee(mut o: ::core::ffi::c_int) -> *mut Enemy_Metalee {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_Metalee;
}
#[inline]
unsafe extern "C" fn Get_Metroid(mut o: ::core::ffi::c_int) -> *mut Enemy_Metroid {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_Metroid;
}
#[inline]
unsafe extern "C" fn Get_Mochtroid(mut o: ::core::ffi::c_int) -> *mut Enemy_Mochtroid {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_Mochtroid;
}
#[inline]
unsafe extern "C" fn Get_PlatformThatFallsWithSamus(
    mut o: ::core::ffi::c_int,
) -> *mut Enemy_PlatformThatFallsWithSamus {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_PlatformThatFallsWithSamus;
}
#[inline]
unsafe extern "C" fn Get_Reflec(mut o: ::core::ffi::c_int) -> *mut Enemy_Reflec {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_Reflec;
}
#[inline]
unsafe extern "C" fn Get_Roach(mut o: ::core::ffi::c_int) -> *mut Enemy_Roach {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_Roach;
}
#[inline]
unsafe extern "C" fn Get_Sidehopper(mut o: ::core::ffi::c_int) -> *mut Enemy_Sidehopper {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_Sidehopper;
}
#[inline]
unsafe extern "C" fn Get_Skree(mut o: ::core::ffi::c_int) -> *mut Enemy_Skree {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_Skree;
}
#[inline]
unsafe extern "C" fn Get_StoneZoomer(mut o: ::core::ffi::c_int) -> *mut Enemy_StoneZoomer {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_StoneZoomer;
}
#[inline]
unsafe extern "C" fn Get_Waver(mut o: ::core::ffi::c_int) -> *mut Enemy_Waver {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_Waver;
}
#[inline]
unsafe extern "C" fn Get_WreckedShipOrangeZoomer(
    mut o: ::core::ffi::c_int,
) -> *mut Enemy_WreckedShipOrangeZoomer {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_WreckedShipOrangeZoomer;
}
static mut g_word_A3A76D: [int16; 4] = [
    2 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(2 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
];
static mut g_word_A3A775: [int16; 4] = [
    0 as ::core::ffi::c_int as int16,
    -(2 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    2 as ::core::ffi::c_int as int16,
];
#[no_mangle]
pub unsafe extern "C" fn Enemy_GrappleReact_NoInteract_A3() {
    SwitchEnemyAiToMainAi();
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_GrappleReact_KillEnemy_A3() {
    EnemyGrappleDeath();
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_GrappleReact_CancelBeam_A3() {
    Enemy_SwitchToFrozenAi();
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_NormalTouchAI_A3() {
    NormalEnemyTouchAi();
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_NormalShotAI_A3() {
    NormalEnemyShotAi();
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_NormalShotAI_SkipSomeParts_A3() {
    NormalEnemyShotAiSkipDeathAnim_CurEnemy();
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_NormalPowerBombAI_A3() {
    NormalEnemyPowerBombAi();
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_NormalFrozenAI_A3() {
    NormalEnemyFrozenAI();
}
#[no_mangle]
pub unsafe extern "C" fn Waver_Instr_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Waver(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .waver_var_E = 1 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Waver_Init() {
    let mut E: *mut Enemy_Waver = Get_Waver(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).waver_var_B = 1 as uint16;
    (*E).waver_var_A = 0x8000 as uint16;
    if (*E).waver_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).waver_var_B = SignExtend8(0xfe as uint16);
        (*E).waver_var_A = SignExtend8(0x8000 as uint16);
    }
    (*E).waver_var_F = 0 as uint16;
    (*E).waver_var_C = 0 as uint16;
    (*E).waver_var_E = 0 as uint16;
    (*E).base.current_instruction = addr_kWaver_Ilist_86A7 as ::core::ffi::c_int as uint16;
    (*E).waver_var_F =
        ((*E).waver_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint16;
    Waver_Func_1();
}
#[no_mangle]
pub unsafe extern "C" fn Waver_Main() {
    let mut E: *mut Enemy_Waver = Get_Waver(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (((*E).waver_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).waver_var_A as uint32)
            as int32,
    ) != 0
    {
        let mut r18: uint16 = *((&raw mut (*E).waver_var_A as *mut uint8)
            .offset(1 as ::core::ffi::c_int as isize)
            as *mut uint16);
        (*E).waver_var_B = SignExtend8(
            ((-(r18 as ::core::ffi::c_int) & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint16,
        );
        (*E).waver_var_A =
            (-(r18 as int8 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16;
        (*E).waver_var_F = (((*E).waver_var_F as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int)
            & 1 as ::core::ffi::c_int) as uint16;
        Waver_Func_1();
    } else if Enemy_MoveDown(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (SineMult8bit(
            *(&raw mut (*E).waver_var_D as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as uint16,
            4 as uint16,
        ) as int16 as int32)
            << 16 as ::core::ffi::c_int,
    ) != 0
    {
        (*E).waver_var_D = ((*E).waver_var_D as ::core::ffi::c_int + 0x80 as ::core::ffi::c_int)
            as uint8 as uint16;
    } else {
        (*E).waver_var_D =
            ((*E).waver_var_D as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
    if (*E).waver_var_D as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int
        == 56 as ::core::ffi::c_int
    {
        (*E).waver_var_F =
            ((*E).waver_var_F as ::core::ffi::c_int | 2 as ::core::ffi::c_int) as uint16;
        Waver_Func_1();
    }
    if (*E).waver_var_E != 0 {
        (*E).waver_var_E = 0 as uint16;
        (*E).waver_var_F =
            ((*E).waver_var_F as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint16;
        Waver_Func_1();
    }
}
#[no_mangle]
pub unsafe extern "C" fn Waver_Func_1() {
    let mut E: *mut Enemy_Waver = Get_Waver(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut waver_var_F: uint16 = (*E).waver_var_F;
    if waver_var_F as ::core::ffi::c_int != (*E).waver_var_C as ::core::ffi::c_int {
        (*E).waver_var_C = waver_var_F;
        (*E).base.current_instruction = *(g_rom.offset(
            (((0xa386db as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa386db as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(waver_var_F as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Metalee_Instr_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Metalee(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .metalee_var_E = 1 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Metalee_Init() {
    let mut E: *mut Enemy_Metalee = Get_Metalee(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).metalee_var_C = 0 as uint16;
    (*E).metalee_var_D = 0 as uint16;
    (*E).metalee_var_E = 0 as uint16;
    (*E).base.current_instruction = addr_kMetalee_Ilist_8910 as ::core::ffi::c_int as uint16;
    (*E).metalee_var_B = (0xa38987 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn CallMetaleeFunc(mut ea: uint32) {
    match ea {
        10717575 => {
            Metalee_Func_1();
            return;
        }
        10717652 => {
            Metalee_Func_3(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
            return;
        }
        10717683 => {
            Metalee_Func_4();
            return;
        }
        10717788 => {
            Metalee_Func_5();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Metalee_Main() {
    let mut E: *mut Enemy_Metalee = Get_Metalee(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallMetaleeFunc(
        ((*E).metalee_var_B as ::core::ffi::c_int | 0xa30000 as ::core::ffi::c_int) as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Metalee_Func_1() {
    let mut E: *mut Enemy_Metalee = Get_Metalee(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (abs16(
        ((*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
    ) as ::core::ffi::c_int)
        < 0x48 as ::core::ffi::c_int
    {
        Metalee_Func_2(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
        (*E).metalee_var_C = (*E).metalee_var_C.wrapping_add(1);
        Metalee_Func_6();
        (*E).metalee_var_B =
            (0xa389d4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Metalee_Func_2(mut k: uint16) {
    let mut E: *mut Enemy_Metalee = Get_Metalee(k as ::core::ffi::c_int);
    let mut div: uint16 = SnesDivide(
        (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - (*E).base.y_pos as ::core::ffi::c_int) as uint16,
        24 as uint8,
    ) as uint8 as uint16;
    (*E).metalee_var_F = (div as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Metalee_Func_3(mut k: uint16) {
    let mut E: *mut Enemy_Metalee = Get_Metalee(k as ::core::ffi::c_int);
    if (*E).metalee_var_E != 0 {
        (*E).metalee_var_E = 0 as uint16;
        (*E).metalee_var_C = (*E).metalee_var_C.wrapping_add(1);
        Metalee_Func_6();
        (*E).metalee_var_B =
            (0xa389f3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        QueueSfx2_Max6(0x5b as uint16);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Metalee_Func_4() {
    let mut E: *mut Enemy_Metalee = Get_Metalee(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).metalee_var_A = 21 as uint16;
    let mut v1: uint16 =
        ((*E).base.properties as ::core::ffi::c_int | 3 as ::core::ffi::c_int) as uint16;
    (*E).base.properties = v1;
    if EnemyFunc_BF8A(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        v1,
        ((*E).metalee_var_F as int16 as int32) << 16 as ::core::ffi::c_int,
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).metalee_var_B =
            (0xa38a5c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        QueueSfx2_Max6(0x5c as uint16);
    } else {
        (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
            + (*E).metalee_var_F as ::core::ffi::c_int) as uint16;
        (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
            + if ((*E).base.x_pos as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                -(2 as ::core::ffi::c_int)
            } else {
                2 as ::core::ffi::c_int
            }) as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Metalee_Func_5() {
    let mut E: *mut Enemy_Metalee = Get_Metalee(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let fresh8 = (*E).metalee_var_A;
    (*E).metalee_var_A = (*E).metalee_var_A.wrapping_sub(1);
    if fresh8 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*gEnemySpawnData(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        ))
        .vram_tiles_index = ((*E).base.vram_tiles_index as ::core::ffi::c_int
            | (*E).base.palette_index as ::core::ffi::c_int) as uint16;
        (*E).base.palette_index = 2560 as uint16;
        (*E).base.vram_tiles_index = 0 as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
    } else {
        if (*E).metalee_var_A as ::core::ffi::c_int == 8 as ::core::ffi::c_int {
            let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
                .offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16);
            SpawnEprojWithGfx(
                0 as uint16,
                v2,
                addr_stru_868BFA as ::core::ffi::c_int as uint16,
            );
            SpawnEprojWithGfx(
                0 as uint16,
                v2,
                addr_stru_868C08 as ::core::ffi::c_int as uint16,
            );
            SpawnEprojWithGfx(
                0 as uint16,
                v2,
                addr_stru_868C16 as ::core::ffi::c_int as uint16,
            );
            SpawnEprojWithGfx(
                0 as uint16,
                v2,
                addr_stru_868C24 as ::core::ffi::c_int as uint16,
            );
        }
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    };
}
#[no_mangle]
pub unsafe extern "C" fn Metalee_Func_6() {
    let mut E: *mut Enemy_Metalee = Get_Metalee(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut metalee_var_C: uint16 = (*E).metalee_var_C;
    if metalee_var_C as ::core::ffi::c_int != (*E).metalee_var_D as ::core::ffi::c_int {
        (*E).metalee_var_D = metalee_var_C;
        (*E).base.current_instruction = *(g_rom.offset(
            (((0xa3894e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3894e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(metalee_var_C as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Metalee_Shot() {
    let mut E: *mut Enemy_Metalee = Get_Metalee(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut varE2A: uint16 = (*E).base.vram_tiles_index;
    let mut varE2C: uint16 = (*E).base.palette_index;
    Enemy_NormalShotAI_A3();
    if (*Get_Metalee(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .health
        == 0
    {
        (*E).base.vram_tiles_index = varE2A;
        (*E).base.palette_index = varE2C;
        let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16);
        SpawnEprojWithGfx(
            (*E).metalee_var_A,
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            addr_stru_868BFA as ::core::ffi::c_int as uint16,
        );
        SpawnEprojWithGfx(
            0 as uint16,
            v2,
            addr_stru_868C08 as ::core::ffi::c_int as uint16,
        );
        SpawnEprojWithGfx(
            0 as uint16,
            v2,
            addr_stru_868C16 as ::core::ffi::c_int as uint16,
        );
        SpawnEprojWithGfx(
            0 as uint16,
            v2,
            addr_stru_868C24 as ::core::ffi::c_int as uint16,
        );
        (*E).base.vram_tiles_index = 0 as uint16;
        (*E).base.palette_index = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Fireflea_Init() {
    let mut E: *mut Enemy_FireFlea = Get_FireFlea(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.current_instruction = addr_kFireflea_Ilist_8C2F as ::core::ffi::c_int as uint16;
    if (*E).ffa_parameter_1 as ::core::ffi::c_int & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        Fireflea_Func_1(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
        Fireflea_Func_2(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
        Fireflea_Func_3(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
        Fireflea_Func_4(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
        Fireflea_Func_5(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
    } else {
        Fireflea_Func_3(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
        Fireflea_Func_4(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
        Fireflea_Func_6(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn Fireflea_Func_1(mut k: uint16) {
    let mut E: *mut Enemy_FireFlea = Get_FireFlea(k as ::core::ffi::c_int);
    (*E).ffa_var_E = (*E).base.x_pos;
    (*E).ffa_var_F = (*E).base.y_pos;
}
#[no_mangle]
pub unsafe extern "C" fn Fireflea_Func_2(mut k: uint16) {
    let mut E: *mut Enemy_FireFlea = Get_FireFlea(k as ::core::ffi::c_int);
    (*E).ffa_var_D = ((*(&raw mut (*E).ffa_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Fireflea_Func_3(mut k: uint16) {
    let mut E: *mut Enemy_FireFlea = Get_FireFlea(k as ::core::ffi::c_int);
    let mut v2: uint16 = (8 as ::core::ffi::c_int
        * *(&raw mut (*E).ffa_parameter_2 as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int) as uint16;
    if (*E).ffa_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v2 = (v2 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    }
    (*E).ffa_var_02 = v2;
    let mut v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).ffa_var_B = *(g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v3 as isize);
    (*E).ffa_var_A = *(g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v3 + 1 as ::core::ffi::c_int) as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Fireflea_Func_4(mut k: uint16) {
    let mut E: *mut Enemy_FireFlea = Get_FireFlea(k as ::core::ffi::c_int);
    (*E).ffa_var_C = *((g_rom.offset(
        (((0xa38d1d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa38d1d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *(&raw mut (*E).ffa_parameter_2 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as isize,
        ) as *mut uint16 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Fireflea_Func_5(mut k: uint16) {
    let mut E: *mut Enemy_FireFlea = Get_FireFlea(k as ::core::ffi::c_int);
    (*E).base.x_pos = ((*E).ffa_var_E as ::core::ffi::c_int
        + CosineMult8bit((*E).ffa_var_D, (*E).ffa_var_C) as ::core::ffi::c_int)
        as uint16;
    (*E).base.y_pos = ((*E).ffa_var_F as ::core::ffi::c_int
        + SineMult8bit((*E).ffa_var_D, (*E).ffa_var_C) as ::core::ffi::c_int)
        as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Fireflea_Func_6(mut k: uint16) {
    let mut E: *mut Enemy_FireFlea = Get_FireFlea(k as ::core::ffi::c_int);
    (*E).ffa_var_00 =
        ((*E).base.y_pos as ::core::ffi::c_int - (*E).ffa_var_C as ::core::ffi::c_int) as uint16;
    (*E).ffa_var_01 =
        ((*E).ffa_var_C as ::core::ffi::c_int + (*E).base.y_pos as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Fireflea_Main() {
    let mut E: *mut Enemy_FireFlea = Get_FireFlea(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).ffa_parameter_1 as ::core::ffi::c_int & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.x_pos = ((*E).ffa_var_E as ::core::ffi::c_int
            + CosineMult8bit(
                *(&raw mut (*E).ffa_var_D as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as uint16,
                (*E).ffa_var_C,
            ) as ::core::ffi::c_int) as uint16;
        (*E).base.y_pos = ((*E).ffa_var_F as ::core::ffi::c_int
            + SineMult8bit(
                *(&raw mut (*E).ffa_var_D as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as uint16,
                (*E).ffa_var_C,
            ) as ::core::ffi::c_int) as uint16;
        (*E).ffa_var_D = ((*E).ffa_var_D as ::core::ffi::c_int
            + *((&raw mut (*E).ffa_var_A as *mut uint8).offset(1 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
    } else {
        let mut v2: ::core::ffi::c_int =
            (*E).ffa_var_02 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
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
        if (((*E).base.y_pos as ::core::ffi::c_int - (*E).ffa_var_00 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
            || ((*E).base.y_pos as ::core::ffi::c_int - (*E).ffa_var_01 as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
        {
            (*E).ffa_var_02 =
                ((*E).ffa_var_02 as ::core::ffi::c_int ^ 4 as ::core::ffi::c_int) as uint16;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Fireflea_Touch(mut k: uint16) {
    let mut v1: uint16 = 0 as uint16;
    Enemy_NormalTouchAI_A3();
    EnemyDeathAnimation(k, v1);
    if *((&raw mut g_ram as *mut uint8).offset(0x177e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 12 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh7 = *((&raw mut g_ram as *mut uint8)
            .offset(0x177e as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh7 = (*fresh7 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Fireflea_Powerbomb() {
    Enemy_NormalPowerBombAI_A3();
    Fireflea_Common();
}
#[no_mangle]
pub unsafe extern "C" fn Fireflea_Shot() {
    Enemy_NormalShotAI_A3();
    Fireflea_Common();
}
#[no_mangle]
pub unsafe extern "C" fn Fireflea_Common() {
    if (*Get_FireFlea(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .health
        == 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x177e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 12 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let ref mut fresh6 = *((&raw mut g_ram as *mut uint8)
                .offset(0x177e as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh6 = (*fresh6 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaFish_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    (*Get_MaridiaFish(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .layer = 6 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaFish_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    (*Get_MaridiaFish(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .layer = 2 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaFish_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    (*Get_MaridiaFish(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .mfh_var_01 = 1 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaFish_Init() {
    let mut E: *mut Enemy_MaridiaFish = Get_MaridiaFish(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.current_instruction = addr_kMaridiaFish_Ilist_902A as ::core::ffi::c_int as uint16;
    (*E).mfh_var_A = (0xa39132 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if *(&raw mut (*E).mfh_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) == 0
    {
        (*E).base.current_instruction =
            addr_kMaridiaFish_Ilist_9060 as ::core::ffi::c_int as uint16;
        (*E).mfh_var_A = (0xa391ab as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    let mut v1: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * *(&raw mut (*E).mfh_parameter_1 as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).mfh_var_C = *(g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize);
    (*E).mfh_var_B = *(g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v1 + 1 as ::core::ffi::c_int) as isize);
    (*E).mfh_var_E = *(g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v1 + 2 as ::core::ffi::c_int) as isize);
    (*E).mfh_var_D = *(g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v1 + 3 as ::core::ffi::c_int) as isize);
    (*E).mfh_var_00 = *(&raw mut (*E).mfh_parameter_2 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as uint16;
    (*E).mfh_var_02 = *(&raw mut (*E).mfh_parameter_2 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as uint16;
    (*E).mfh_var_F = 0 as uint16;
    (*E).mfh_var_01 = 0 as uint16;
    (*E).mfh_var_03 = SineMult8bit((*E).mfh_var_F, (*E).mfh_var_00);
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaFish_Func_1() {
    let mut E: *mut Enemy_MaridiaFish = Get_MaridiaFish(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (((*E).mfh_var_E as uint32) << 16 as ::core::ffi::c_int | (*E).mfh_var_D as uint32)
            as int32,
    ) != 0
    {
        (*E).mfh_var_A = (0xa39224 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction =
            addr_kMaridiaFish_Ilist_903C as ::core::ffi::c_int as uint16;
    } else {
        let mut v2: uint16 = SineMult8bit((*E).mfh_var_F, (*E).mfh_var_00);
        (*E).mfh_var_04 = v2;
        if Enemy_MoveDown(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            ((v2 as ::core::ffi::c_int - (*E).mfh_var_03 as ::core::ffi::c_int) as int16 as int32)
                << 16 as ::core::ffi::c_int,
        ) != 0
        {
            (*E).mfh_var_A =
                (0xa39224 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).base.timer = 0 as uint16;
            (*E).base.current_instruction =
                addr_kMaridiaFish_Ilist_903C as ::core::ffi::c_int as uint16;
        } else {
            (*E).mfh_var_F =
                (*(&raw mut (*E).mfh_var_02 as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    + (*E).mfh_var_F as ::core::ffi::c_int) as uint8 as uint16;
        }
    }
    (*E).mfh_var_03 = (*E).mfh_var_04;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaFish_Func_2() {
    let mut E: *mut Enemy_MaridiaFish = Get_MaridiaFish(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (((*E).mfh_var_C as uint32) << 16 as ::core::ffi::c_int | (*E).mfh_var_B as uint32)
            as int32,
    ) != 0
    {
        (*E).mfh_var_A = (0xa39256 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction =
            addr_kMaridiaFish_Ilist_9072 as ::core::ffi::c_int as uint16;
    } else {
        let mut v2: uint16 = SineMult8bit((*E).mfh_var_F, (*E).mfh_var_00);
        (*E).mfh_var_04 = v2;
        if Enemy_MoveDown(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            ((v2 as ::core::ffi::c_int - (*E).mfh_var_03 as ::core::ffi::c_int) as int16 as int32)
                << 16 as ::core::ffi::c_int,
        ) != 0
        {
            (*E).mfh_var_A =
                (0xa39256 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).base.timer = 0 as uint16;
            (*E).base.current_instruction =
                addr_kMaridiaFish_Ilist_9072 as ::core::ffi::c_int as uint16;
        } else {
            (*E).mfh_var_F =
                (*(&raw mut (*E).mfh_var_02 as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    + (*E).mfh_var_F as ::core::ffi::c_int) as uint8 as uint16;
        }
    }
    (*E).mfh_var_03 = (*E).mfh_var_04;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaFish_Func_3() {
    let mut E: *mut Enemy_MaridiaFish = Get_MaridiaFish(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).mfh_var_01 != 0 {
        (*E).mfh_var_01 = 0 as uint16;
        (*E).mfh_var_A = (0xa391ab as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mfh_var_02 = -((*E).mfh_var_02 as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction =
            addr_kMaridiaFish_Ilist_9060 as ::core::ffi::c_int as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaFish_Func_4() {
    let mut E: *mut Enemy_MaridiaFish = Get_MaridiaFish(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).mfh_var_01 != 0 {
        (*E).mfh_var_01 = 0 as uint16;
        (*E).mfh_var_A = (0xa39132 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mfh_var_02 = -((*E).mfh_var_02 as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction =
            addr_kMaridiaFish_Ilist_902A as ::core::ffi::c_int as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CallMaridiaFishFunc(mut ea: uint32) {
    match ea {
        10719538 => {
            MaridiaFish_Func_1();
            return;
        }
        10719659 => {
            MaridiaFish_Func_2();
            return;
        }
        10719780 => {
            MaridiaFish_Func_3();
            return;
        }
        10719830 => {
            MaridiaFish_Func_4();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaFish_Main() {
    let mut E: *mut Enemy_MaridiaFish = Get_MaridiaFish(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallMaridiaFishFunc(
        ((*E).mfh_var_A as ::core::ffi::c_int | 0xa30000 as ::core::ffi::c_int) as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Elevator_Init() {
    let mut E: *mut Enemy_Elevator = Get_Elevator(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A3 as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction = addr_kElevator_Ilist_94D6 as ::core::ffi::c_int as uint16;
    (*E).elevat_parameter_1 =
        ((*E).elevat_parameter_1 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
    (*E).elevat_var_A = (*E).base.y_pos;
    if *((&raw mut g_ram as *mut uint8).offset(0xe18 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 2 as ::core::ffi::c_int
    {
        let ref mut fresh5 = *((&raw mut g_ram as *mut uint8)
            .offset(0xe18 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh5 = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xe16 as ::core::ffi::c_int as isize)
            as *mut uint16) = *fresh5;
    }
    if (*((&raw mut g_ram as *mut uint8).offset(0xe18 as ::core::ffi::c_int as isize)
        as *mut uint16) as uint32)
        << 16 as ::core::ffi::c_int
        | *((&raw mut g_ram as *mut uint8).offset(0xe16 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32
        != 0
    {
        (*E).base.y_pos = (*E).elevat_parameter_2;
        Elevator_Func_4();
    }
}
static mut off_A39540: [Option<Func_V>; 4] = unsafe {
    [
        Some(Elevator_Func_1 as unsafe extern "C" fn() -> ()),
        Some(Elevator_Func_2 as unsafe extern "C" fn() -> ()),
        Some(Elevator_Func_3 as unsafe extern "C" fn() -> ()),
        Some(Elevator_Func3b as unsafe extern "C" fn() -> ()),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn Elevator_Frozen() {
    if *((&raw mut g_ram as *mut uint8).offset(0x795 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        if (*((&raw mut g_ram as *mut uint8).offset(0xe18 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0xe16 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32
            != 0
        {
            off_A39540[*((&raw mut g_ram as *mut uint8)
                    .offset(0xe18 as ::core::ffi::c_int as isize) as *mut uint16)
                    as usize]
                .expect("non-null function pointer")();
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Elevator_Func_1() {
    if *(g_rom.offset(
        (((0xa394e2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa394e2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((*Get_Elevator(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int,
            ))
            .elevat_parameter_1 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) as ::core::ffi::c_int
        & *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        QueueSfx3_Max6(0xb as uint16);
        QueueSfx1_Max6(0x32 as uint16);
        CallSomeSamusCode(7 as uint16);
        ResetProjectileData();
        Elevator_Func_4();
        let ref mut fresh4 = *((&raw mut g_ram as *mut uint8)
            .offset(0xe18 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh4 = (*fresh4).wrapping_add(1);
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xe16 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Elevator_Func_2() {
    let mut E: *mut Enemy_Elevator = Get_Elevator(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).elevat_parameter_1 != 0 {
        *((&raw mut g_ram as *mut uint8).offset(0x799 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0x8000 as uint16;
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            -(0x18000 as ::core::ffi::c_int) as uint32,
        );
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0x799 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            0x18000 as uint32,
        );
    }
    Elevator_Func_4();
}
#[no_mangle]
pub unsafe extern "C" fn Elevator_Func_3() {
    let ref mut fresh3 = *((&raw mut g_ram as *mut uint8)
        .offset(0xe18 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh3 = (*fresh3).wrapping_add(1);
    Elevator_Func3b();
}
#[no_mangle]
pub unsafe extern "C" fn Elevator_Func3b() {
    let mut E: *mut Enemy_Elevator = Get_Elevator(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).elevat_parameter_1 != 0 {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            0x18000 as uint32,
        );
        if ((*E).base.y_pos as ::core::ffi::c_int) < (*E).elevat_var_A as ::core::ffi::c_int {
            Elevator_Func_4();
            return;
        }
    } else {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            -(0x18000 as ::core::ffi::c_int) as uint32,
        );
        if (*E).base.y_pos as ::core::ffi::c_int >= (*E).elevat_var_A as ::core::ffi::c_int {
            Elevator_Func_4();
            return;
        }
    }
    *((&raw mut g_ram as *mut uint8).offset(0xe18 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xe16 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    QueueSfx3_Max6(0x25 as uint16);
    (*E).base.y_pos = (*E).elevat_var_A;
    CallSomeSamusCode(0xb as uint16);
    Elevator_Func_4();
}
#[no_mangle]
pub unsafe extern "C" fn Elevator_Func_4() {
    let mut E: *mut Enemy_Elevator = Get_Elevator(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16) =
        ((*E).base.y_pos as ::core::ffi::c_int - 26 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xafc as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16) =
        (*E).base.x_pos;
    *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Crab_Init() {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v1).parameter_2 = 8 as uint16;
    (*v1).current_instruction = *(g_rom.offset(
        (((0xa396db as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa396db as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((*v1).current_instruction as ::core::ffi::c_int & 3 as ::core::ffi::c_int) as isize,
        );
    StoneZoomer_E67A(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Crab_Func_1() {}
#[no_mangle]
pub unsafe extern "C" fn Slug_Init() {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v1).parameter_2 = 10 as uint16;
    (*v1).current_instruction = *(g_rom.offset(
        (((0xa3992b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3992b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((*v1).current_instruction as ::core::ffi::c_int & 3 as ::core::ffi::c_int) as isize,
        );
    StoneZoomer_E67A(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Slug_Func_1() {}
#[no_mangle]
pub unsafe extern "C" fn PlatformThatFallsWithSamus_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    (*Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .ptfwss_var_02 = 0 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn PlatformThatFallsWithSamus_Instr_4(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    (*Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .ptfwss_var_02 = 1 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn PlatformThatFallsWithSamus_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    (*Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .ptfwss_var_02 = 0 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn PlatformThatFallsWithSamus_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    (*Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .ptfwss_var_02 = 1 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn PlatformThatFallsWithSamus_Init() {
    let mut E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).ptfwss_var_1F = -(1 as ::core::ffi::c_int) as uint16;
    let mut ptfwss_parameter_1: uint16 = (*E).ptfwss_parameter_1;
    (*E).ptfwss_var_02 = ptfwss_parameter_1;
    if ptfwss_parameter_1 != 0 {
        PlatformThatFalls_Init(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            addr_kPlatformThatFallsWithSamus_Ilist_9BFD as ::core::ffi::c_int as uint16,
        );
    } else {
        PlatformThatFalls_Init(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            addr_kPlatformThatFallsWithSamus_Ilist_9BE7 as ::core::ffi::c_int as uint16,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn FastMovingSlowSinkingPlatform_Init() {
    let mut E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut ptfwss_parameter_1: uint16 = (*E).ptfwss_parameter_1;
    (*E).ptfwss_var_02 = ptfwss_parameter_1;
    if ptfwss_parameter_1 != 0 {
        PlatformThatFalls_Init(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            addr_kPlatformThatFallsWithSamus_Ilist_9C55 as ::core::ffi::c_int as uint16,
        );
    } else {
        PlatformThatFalls_Init(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            addr_kPlatformThatFallsWithSamus_Ilist_9C3F as ::core::ffi::c_int as uint16,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn PlatformThatFalls_Init(mut k: uint16, mut j: uint16) {
    let mut E: *mut Enemy_PlatformThatFallsWithSamus =
        Get_PlatformThatFallsWithSamus(k as ::core::ffi::c_int);
    (*E).base.current_instruction = j;
    let mut v3: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * *(&raw mut (*E).ptfwss_parameter_2 as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).ptfwss_var_C = *(g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v3 as isize);
    (*E).ptfwss_var_B = *(g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v3 + 1 as ::core::ffi::c_int) as isize);
    (*E).ptfwss_var_E = *(g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v3 + 2 as ::core::ffi::c_int) as isize);
    (*E).ptfwss_var_D = *(g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v3 + 3 as ::core::ffi::c_int) as isize);
    (*E).ptfwss_var_00 = 0 as uint16;
    (*E).ptfwss_var_03 = 0 as uint16;
    (*E).ptfwss_var_04 = 0 as uint16;
    (*E).ptfwss_var_A = ((*E).base.y_pos as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    (*E).ptfwss_var_F = 0 as uint16;
    (*E).ptfwss_var_05 = *(&raw mut (*E).ptfwss_parameter_2 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as uint16;
}
static mut off_A39C97: [Option<Func_V>; 2] = unsafe {
    [
        Some(PlatformThatFallsWithSamus_Func_1 as unsafe extern "C" fn() -> ()),
        Some(PlatformThatFallsWithSamus_Func_2 as unsafe extern "C" fn() -> ()),
    ]
};
static mut off_A39C9B: [Option<Func_V>; 2] = unsafe {
    [
        Some(PlatformThatFallsWithSamus_Func_3 as unsafe extern "C" fn() -> ()),
        Some(PlatformThatFallsWithSamus_Func_4 as unsafe extern "C" fn() -> ()),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn PlatformThatFallsWithSamus_Main() {
    let mut E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).ptfwss_var_00 = 0 as uint16;
    if CheckIfEnemyTouchesSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    ) != 0
    {
        (*E).ptfwss_var_00 = 1 as uint16;
    }
    off_A39C97[(*E).ptfwss_var_02 as usize].expect("non-null function pointer")();
    let mut v3: ::core::ffi::c_int = (*Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .ptfwss_var_00 as ::core::ffi::c_int;
    off_A39C9B[v3 as usize].expect("non-null function pointer")();
    if (*E).ptfwss_var_00 as ::core::ffi::c_int != (*E).ptfwss_var_06 as ::core::ffi::c_int {
        (*E).ptfwss_var_F = 0 as uint16;
    }
    (*E).ptfwss_var_06 = (*E).ptfwss_var_00;
}
#[no_mangle]
pub unsafe extern "C" fn PlatformThatFallsWithSamus_Func_1() {
    let mut E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).ptfwss_var_01 = (*E).base.x_pos;
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (((*E).ptfwss_var_E as uint32) << 16 as ::core::ffi::c_int | (*E).ptfwss_var_D as uint32)
            as int32,
    ) != 0
    {
        (*E).ptfwss_var_02 = 1 as uint16;
        PlatformThatFallsWithSamus_Func_8();
    }
}
#[no_mangle]
pub unsafe extern "C" fn PlatformThatFallsWithSamus_Func_2() {
    let mut E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).ptfwss_var_01 = (*E).base.x_pos;
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (((*E).ptfwss_var_C as uint32) << 16 as ::core::ffi::c_int | (*E).ptfwss_var_B as uint32)
            as int32,
    ) != 0
    {
        (*E).ptfwss_var_02 = 0 as uint16;
        PlatformThatFallsWithSamus_Func_7();
    }
}
#[no_mangle]
pub unsafe extern "C" fn PlatformThatFallsWithSamus_Func_3() {
    let mut ptfwss_var_05: uint16 = 0;
    let mut v3: ::core::ffi::c_int = 0;
    let mut current_block: u64;
    let mut E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (((*E).base.y_pos as ::core::ffi::c_int - (*E).ptfwss_var_A as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        current_block = 2432778910466551290;
    } else {
        PlatformThatFallsWithSamus_Func_9();
        ptfwss_var_05 = 0;
        (*E).ptfwss_var_F = (*E).ptfwss_var_F.wrapping_add(1);
        ptfwss_var_05 = (*E).ptfwss_var_F;
        if (ptfwss_var_05 as ::core::ffi::c_int - (*E).ptfwss_var_05 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            ptfwss_var_05 = (*E).ptfwss_var_05;
            (*E).ptfwss_var_F = ptfwss_var_05;
        }
        v3 = 0;
        v3 = 8 as ::core::ffi::c_int * ptfwss_var_05 as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        if Enemy_MoveDown(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            ((*(g_rom.offset(
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
                    .offset((v3 + 2 as ::core::ffi::c_int) as isize) as uint32)
                as int32,
        ) != 0
        {
            current_block = 2432778910466551290;
        } else {
            current_block = 3640593987805443782;
        }
    }
    match current_block {
        2432778910466551290 => {
            (*E).ptfwss_var_F = 0 as uint16;
            PlatformThatFallsWithSamus_Func_10();
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn PlatformThatFallsWithSamus_Func_4() {
    let mut E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).ptfwss_var_F = (*E).ptfwss_var_F.wrapping_add(1);
    if ((*E).ptfwss_var_F as ::core::ffi::c_int - (*E).ptfwss_var_05 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).ptfwss_var_F = (*E).ptfwss_var_05;
    }
    let ref mut fresh13 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb58 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh13 = (*fresh13 as ::core::ffi::c_int
        + ((*E).base.x_pos as ::core::ffi::c_int - (*E).ptfwss_var_01 as ::core::ffi::c_int))
        as uint16;
    (*E).ptfwss_var_01 = (*E).base.y_pos;
    let mut v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * (*E).ptfwss_var_F as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        ((*(g_rom.offset(
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
                .offset(v2 as isize) as uint32) as int32,
    ) != 0
    {
        (*E).ptfwss_var_F = 0 as uint16;
        PlatformThatFallsWithSamus_Func_10();
    }
    let ref mut fresh14 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb5c as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh14 = (*fresh14 as ::core::ffi::c_int
        + ((*E).base.y_pos as ::core::ffi::c_int - (*E).ptfwss_var_01 as ::core::ffi::c_int))
        as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn PlatformThatFallsWithSamus_Func_5() {
    let mut E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v0: uint16 =
        addr_kPlatformThatFallsWithSamus_Ilist_9C13 as ::core::ffi::c_int as uint16;
    if (*E).ptfwss_var_1F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v0 = addr_kPlatformThatFallsWithSamus_Ilist_9BBB as ::core::ffi::c_int as uint16;
    }
    (*E).base.current_instruction = v0;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn PlatformThatFallsWithSamus_Func_6() {
    let mut E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v0: uint16 =
        addr_kPlatformThatFallsWithSamus_Ilist_9C29 as ::core::ffi::c_int as uint16;
    if (*E).ptfwss_var_1F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v0 = addr_kPlatformThatFallsWithSamus_Ilist_9BD1 as ::core::ffi::c_int as uint16;
    }
    (*E).base.current_instruction = v0;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn PlatformThatFallsWithSamus_Func_7() {
    let mut E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v0: uint16 =
        addr_kPlatformThatFallsWithSamus_Ilist_9C3F as ::core::ffi::c_int as uint16;
    if (*E).ptfwss_var_1F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v0 = addr_kPlatformThatFallsWithSamus_Ilist_9BE7 as ::core::ffi::c_int as uint16;
    }
    (*E).base.current_instruction = v0;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn PlatformThatFallsWithSamus_Func_8() {
    let mut E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v0: uint16 =
        addr_kPlatformThatFallsWithSamus_Ilist_9C55 as ::core::ffi::c_int as uint16;
    if (*E).ptfwss_var_1F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v0 = addr_kPlatformThatFallsWithSamus_Ilist_9BFD as ::core::ffi::c_int as uint16;
    }
    (*E).base.current_instruction = v0;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn PlatformThatFallsWithSamus_Func_9() {
    let mut E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).ptfwss_var_03 == 0 {
        (*E).ptfwss_var_03 = 1 as uint16;
        if (*E).ptfwss_var_02 != 0 {
            PlatformThatFallsWithSamus_Func_6();
        } else {
            PlatformThatFallsWithSamus_Func_5();
        }
    }
    (*E).ptfwss_var_04 = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn PlatformThatFallsWithSamus_Func_10() {
    let mut E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).ptfwss_var_04 == 0 {
        (*E).ptfwss_var_04 = 1 as uint16;
        if (*E).ptfwss_var_02 != 0 {
            PlatformThatFallsWithSamus_Func_8();
        } else {
            PlatformThatFallsWithSamus_Func_7();
        }
    }
    (*E).ptfwss_var_03 = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn FastMovingSlowSinkingPlatform_Shot() {
    Enemy_NormalShotAI_A3();
    let mut E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).base.frozen_timer != 0 {
        if (*E).ptfwss_var_02 != 0 {
            (*E).base.spritemap_pointer =
                addr_kPlatformThatFallsWithSamus_Sprmap_A015 as ::core::ffi::c_int as uint16;
        } else {
            (*E).base.spritemap_pointer =
                addr_kPlatformThatFallsWithSamus_Sprmap_A009 as ::core::ffi::c_int as uint16;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_1() {
    let mut E: *mut Enemy_Roach = Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut roach_var_24: uint16 = (*E).roach_var_24;
    if roach_var_24 as ::core::ffi::c_int != (*E).roach_var_25 as ::core::ffi::c_int {
        (*E).roach_var_25 = roach_var_24;
        (*E).base.current_instruction = roach_var_24;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Init() {
    let mut E: *mut Enemy_Roach = Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).roach_var_25 = 0 as uint16;
    (*E).roach_var_24 = 0 as uint16;
    Roach_Func_2(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    Roach_Func_4(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    Roach_Func_5(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    Roach_Func_6(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    Roach_Func_7(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    Roach_Func_8(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    (*E).roach_var_24 = *(g_rom.offset(
        (((0xa3a111 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3a111 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(((*E).roach_var_20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Roach_Func_1();
    (*E).roach_var_B = (0xa3a2d7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_2(mut k: uint16) {
    let mut E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int);
    let mut pt: Point32 = ConvertAngleToXy(
        *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as uint16,
        *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as uint16,
    );
    SetHiLo(&raw mut (*E).roach_var_01, &raw mut (*E).roach_var_00, pt.x);
    SetHiLo(&raw mut (*E).roach_var_03, &raw mut (*E).roach_var_02, pt.y);
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_3(mut k: uint16) {
    let mut E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int);
    SetHiLo(
        &raw mut (*E).roach_var_01,
        &raw mut (*E).roach_var_00,
        CosineMult8bitFull(
            *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as uint16,
            *(&raw mut (*E).roach_parameter_1 as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize) as uint16,
        ),
    );
    SetHiLo(
        &raw mut (*E).roach_var_03,
        &raw mut (*E).roach_var_02,
        SineMult8bitFull(
            *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as uint16,
            *(&raw mut (*E).roach_parameter_1 as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize) as uint16,
        ),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_4(mut k: uint16) {
    let mut E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int);
    SetHiLo(
        &raw mut (*E).roach_var_05,
        &raw mut (*E).roach_var_04,
        CosineMult8bitFull(
            (*(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
                - 32 as ::core::ffi::c_int) as uint8 as uint16,
            *(&raw mut (*E).roach_parameter_1 as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize) as uint16,
        ),
    );
    SetHiLo(
        &raw mut (*E).roach_var_07,
        &raw mut (*E).roach_var_06,
        SineMult8bitFull(
            (*(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
                - 32 as ::core::ffi::c_int) as uint8 as uint16,
            *(&raw mut (*E).roach_parameter_1 as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize) as uint16,
        ),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_5(mut k: uint16) {
    let mut E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int);
    SetHiLo(
        &raw mut (*E).roach_var_09,
        &raw mut (*E).roach_var_08,
        CosineMult8bitFull(
            (*(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
                + 32 as ::core::ffi::c_int) as uint8 as uint16,
            *(&raw mut (*E).roach_parameter_1 as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize) as uint16,
        ),
    );
    SetHiLo(
        &raw mut (*E).roach_var_0B,
        &raw mut (*E).roach_var_0A,
        SineMult8bitFull(
            (*(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
                + 32 as ::core::ffi::c_int) as uint8 as uint16,
            *(&raw mut (*E).roach_parameter_1 as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize) as uint16,
        ),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_6(mut k: uint16) {
    let mut E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int);
    (*E).roach_var_20 = (2 as ::core::ffi::c_int
        * ((*(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            - 48 as ::core::ffi::c_int) as uint8 as ::core::ffi::c_int
            >> 5 as ::core::ffi::c_int)) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_7(mut k: uint16) {
    let mut E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int);
    (*E).roach_var_21 = (2 as ::core::ffi::c_int
        * ((*(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            - 80 as ::core::ffi::c_int) as uint8 as ::core::ffi::c_int
            >> 5 as ::core::ffi::c_int)) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_8(mut k: uint16) {
    let mut E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int);
    (*E).roach_var_22 = (2 as ::core::ffi::c_int
        * ((*(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            - 48 as ::core::ffi::c_int
            + 32 as ::core::ffi::c_int) as uint8 as ::core::ffi::c_int
            >> 5 as ::core::ffi::c_int)) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn CallRoachFunc(mut ea: uint32) {
    match ea {
        10724055 => {
            Roach_Func_9();
            return;
        }
        10724097 => {
            Roach_Func_10();
            return;
        }
        10724107 => {
            Roach_Func_11();
            return;
        }
        10724117 => {
            Roach_Func_12();
            return;
        }
        10724133 => {
            Roach_Func_13();
            return;
        }
        10724155 => {
            Roach_Func_14();
            return;
        }
        10724171 => {
            Roach_Func_15();
            return;
        }
        10724224 => {
            Roach_Func_16();
            return;
        }
        10724359 => {
            Roach_Func_19();
            return;
        }
        10724366 => {
            Roach_Func_20();
            return;
        }
        10724416 => {
            Roach_Func_21();
            return;
        }
        10724423 => {
            Roach_Func_22();
            return;
        }
        10724430 => {
            Roach_Func_23();
            return;
        }
        10724450 => {
            Roach_Func_24();
            return;
        }
        10724470 => {
            Roach_Func_25();
            return;
        }
        10724534 => {
            Roach_Func_26();
            return;
        }
        10724592 => {
            Roach_Func_27();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Main() {
    let mut E: *mut Enemy_Roach = Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallRoachFunc(
        ((*E).roach_var_B as ::core::ffi::c_int | 0xa30000 as ::core::ffi::c_int) as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_9() {
    let mut E: *mut Enemy_Roach = Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if IsSamusWithinEnemy_X(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        *(&raw mut (*E).roach_parameter_2 as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as uint16,
    ) != 0
    {
        if IsSamusWithinEnemy_Y(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            *(&raw mut (*E).roach_parameter_2 as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize) as uint16,
        ) != 0
        {
            (*E).roach_var_B = *(g_rom.offset(
                (((0xa3a121 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa3a121 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    *(&raw mut (*E).roach_parameter_2 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ) as isize,
                );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_10() {
    (*Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .roach_var_B = (0xa3a407 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_11() {
    (*Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .roach_var_B = (0xa3a40e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_12() {
    *((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize) as *mut uint16) =
        11 as uint16;
    (*Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .roach_var_B = (0xa3a4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_13() {
    let mut E: *mut Enemy_Roach = Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).roach_var_A = 512 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize) as *mut uint16) =
        11 as uint16;
    (*E).roach_var_B = (0xa3a4b6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_14() {
    let mut E: *mut Enemy_Roach = Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).roach_var_F = 32 as uint16;
    (*E).roach_var_B = (0xa3a44e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_15() {
    let mut v1: uint8 = (64 as ::core::ffi::c_int
        - CalculateAngleOfSamusFromEnemy(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        ) as ::core::ffi::c_int) as uint8;
    let mut E: *mut Enemy_Roach = Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).roach_var_C = v1 as uint16;
    Roach_Func_18(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    Roach_Func_17(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    (*E).roach_var_24 = *(g_rom.offset(
        (((0xa3a111 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3a111 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(((*E).roach_var_23 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Roach_Func_1();
    (*E).roach_var_B = (0xa3a440 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_16() {
    let mut v1: uint16 = (64 as ::core::ffi::c_int
        - CalculateAngleOfSamusFromEnemy(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        ) as ::core::ffi::c_int
        + 0x80 as ::core::ffi::c_int) as uint8 as uint16;
    let mut E: *mut Enemy_Roach = Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).roach_var_C = v1;
    Roach_Func_18(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    Roach_Func_17(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    (*E).roach_var_24 = *(g_rom.offset(
        (((0xa3a111 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3a111 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(((*E).roach_var_23 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Roach_Func_1();
    (*E).roach_var_B = (0xa3a447 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_17(mut k: uint16) {
    let mut E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int);
    (*E).roach_var_23 = (2 as ::core::ffi::c_int
        * (((*E).roach_var_C as ::core::ffi::c_int - 48 as ::core::ffi::c_int) as uint8
            as ::core::ffi::c_int
            >> 5 as ::core::ffi::c_int)) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_18(mut k: uint16) {
    let mut E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int);
    SetHiLo(
        &raw mut (*E).roach_var_0D,
        &raw mut (*E).roach_var_0C,
        CosineMult8bitFull(
            (*E).roach_var_C,
            *(&raw mut (*E).roach_parameter_1 as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize) as uint16,
        ),
    );
    SetHiLo(
        &raw mut (*E).roach_var_0F,
        &raw mut (*E).roach_var_0E,
        SineMult8bitFull(
            (*E).roach_var_C,
            *(&raw mut (*E).roach_parameter_1 as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize) as uint16,
        ),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_19() {
    Roach_Func_29(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_20() {
    let mut E: *mut Enemy_Roach = Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).base.frame_counter as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).roach_var_24 = *(g_rom.offset(
            (((0xa3a111 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3a111 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(((*E).roach_var_22 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        Roach_Func_1();
        Roach_Func_31(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
    } else {
        (*E).roach_var_24 = *(g_rom.offset(
            (((0xa3a111 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3a111 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(((*E).roach_var_21 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        Roach_Func_1();
        Roach_Func_30(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_21() {
    Roach_Func_32(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_22() {
    Roach_Func_32(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_23() {
    let mut E: *mut Enemy_Roach = Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).roach_var_F = (*E).roach_var_F.wrapping_sub(1);
    if (*E).roach_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).roach_var_B =
            (0xa3a2d7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        Roach_Func_29(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_24() {
    let mut E: *mut Enemy_Roach = Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).roach_var_F = (*E).roach_var_F.wrapping_sub(1);
    if (*E).roach_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).roach_var_B =
            (0xa3a4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        Roach_Func_33(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_25() {
    let mut E: *mut Enemy_Roach = Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).roach_var_A = (*E).roach_var_A.wrapping_sub(1);
    let mut v2: bool = (*E).roach_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if !v2 {
        (*E).roach_var_F = (*E).roach_var_F.wrapping_sub(1);
        v2 = (*E).roach_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
        if !v2 {
            let mut v3: uint16 = Abs16(
                ((*E).base.x_pos as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16,
            );
            if v3 as ::core::ffi::c_int - 96 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                let mut v4: uint16 = Abs16(
                    ((*E).base.y_pos as ::core::ffi::c_int
                        - *((&raw mut g_ram as *mut uint8)
                            .offset(0xafa as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int) as uint16,
                );
                if v4 as ::core::ffi::c_int - 96 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    Roach_Func_28();
                }
            }
        } else {
            (*E).roach_var_B =
                (0xa3a4b6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            return;
        }
    }
    Roach_Func_32(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_26() {
    let mut v1: uint8 = (NextRandom() as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as uint8;
    let mut E: *mut Enemy_Roach = Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).roach_var_C =
        ((*E).roach_var_C as ::core::ffi::c_int + v1 as ::core::ffi::c_int) as uint16;
    Roach_Func_18(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    Roach_Func_17(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    (*E).roach_var_24 = *(g_rom.offset(
        (((0xa3a111 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3a111 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(((*E).roach_var_23 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Roach_Func_1();
    (*E).roach_var_F = 32 as uint16;
    (*E).roach_var_B = (0xa3a476 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_27() {
    let mut v1: uint8 = (NextRandom() as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as uint8;
    let mut E: *mut Enemy_Roach = Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).roach_var_C =
        ((*E).roach_var_C as ::core::ffi::c_int + v1 as ::core::ffi::c_int) as uint16;
    Roach_Func_18(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    Roach_Func_17(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    (*E).roach_var_24 = *(g_rom.offset(
        (((0xa3a111 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3a111 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(((*E).roach_var_23 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Roach_Func_1();
    (*E).roach_var_F = 32 as uint16;
    (*E).roach_var_B = (0xa3a462 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_28() {
    let mut E: *mut Enemy_Roach = Get_Roach(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).roach_var_0D = -((*E).roach_var_0D as ::core::ffi::c_int) as uint16;
    (*E).roach_var_0C = -((*E).roach_var_0C as ::core::ffi::c_int) as uint16;
    (*E).roach_var_0F = -((*E).roach_var_0F as ::core::ffi::c_int) as uint16;
    (*E).roach_var_0E = -((*E).roach_var_0E as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 = ((*E).roach_var_23 as ::core::ffi::c_int + 4 as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int) as uint16;
    (*E).roach_var_23 = v1;
    (*E).roach_var_24 = *(g_rom.offset(
        (((0xa3a111 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3a111 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Roach_Func_1();
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_29(mut k: uint16) {
    let mut E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int);
    let mut varE20: uint16 = *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as uint16;
    EnemyFunc_B691(
        varE20,
        Point32 {
            x: ((*E).roach_var_01 as uint32) << 16 as ::core::ffi::c_int
                | (*E).roach_var_00 as uint32,
            y: ((*E).roach_var_03 as uint32) << 16 as ::core::ffi::c_int
                | (*E).roach_var_02 as uint32,
        },
    );
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_30(mut k: uint16) {
    let mut E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).roach_var_05 as uint32) << 16 as ::core::ffi::c_int | (*E).roach_var_04 as uint32,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).roach_var_07 as uint32) << 16 as ::core::ffi::c_int | (*E).roach_var_06 as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_31(mut k: uint16) {
    let mut E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).roach_var_09 as uint32) << 16 as ::core::ffi::c_int | (*E).roach_var_08 as uint32,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).roach_var_0B as uint32) << 16 as ::core::ffi::c_int | (*E).roach_var_0A as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_32(mut k: uint16) {
    let mut E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).roach_var_0D as uint32) << 16 as ::core::ffi::c_int | (*E).roach_var_0C as uint32,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).roach_var_0F as uint32) << 16 as ::core::ffi::c_int | (*E).roach_var_0E as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Roach_Func_33(mut k: uint16) {
    let mut E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).roach_var_0D as uint32) << 16 as ::core::ffi::c_int | (*E).roach_var_0C as uint32)
            as int32,
    ) != 0
    {
        (*E).roach_var_B =
            (0xa3a2d7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else if Enemy_MoveDown(
        k,
        (((*E).roach_var_0F as uint32) << 16 as ::core::ffi::c_int | (*E).roach_var_0E as uint32)
            as int32,
    ) != 0
    {
        (*E).roach_var_B =
            (0xa3a2d7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Mochtroid_Init() {
    let mut E: *mut Enemy_Mochtroid = Get_Mochtroid(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.layer = 2 as uint16;
    Mochtroid_Func_4(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        addr_kMochtroid_Ilist_A745 as ::core::ffi::c_int as uint16,
    );
    (*E).mochtr_var_F = 0 as uint16;
}
static mut off_A3A7A4: [Option<Func_V>; 3] = unsafe {
    [
        Some(Mochtroid_Func_1 as unsafe extern "C" fn() -> ()),
        Some(Mochtroid_Func_3 as unsafe extern "C" fn() -> ()),
        Some(Mochtroid_Func_2 as unsafe extern "C" fn() -> ()),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn Mochtroid_Main() {
    let mut v0: uint16 = (2 as ::core::ffi::c_int
        * (*Get_Mochtroid(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        ))
        .mochtr_var_F as ::core::ffi::c_int) as uint16;
    (*Get_Mochtroid(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .mochtr_var_F = 0 as uint16;
    off_A3A7A4[(v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")();
}
#[no_mangle]
pub unsafe extern "C" fn Mochtroid_Func_1() {
    let mut v5: int16 = 0;
    let mut v10: int16 = 0;
    let mut E: *mut Enemy_Mochtroid = Get_Mochtroid(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut t: int32 = ((((*E).base.y_pos as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >> 2 as ::core::ffi::c_int) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).mochtr_var_D,
        &raw mut (*E).mochtr_var_C,
        -t as uint32,
    );
    let mut current_block_4: u64;
    if ((*E).mochtr_var_D as int16 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        if ((*E).mochtr_var_D as ::core::ffi::c_int) < 0xfffd as ::core::ffi::c_int {
            v5 = -(3 as ::core::ffi::c_int) as int16;
            current_block_4 = 13796420526896809451;
        } else {
            current_block_4 = 8515828400728868193;
        }
    } else if (*E).mochtr_var_D as ::core::ffi::c_int >= 3 as ::core::ffi::c_int {
        v5 = 3 as int16;
        current_block_4 = 13796420526896809451;
    } else {
        current_block_4 = 8515828400728868193;
    }
    match current_block_4 {
        13796420526896809451 => {
            (*E).mochtr_var_D = v5 as uint16;
            (*E).mochtr_var_C = 0 as uint16;
        }
        _ => {}
    }
    if Enemy_MoveDown(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (((*E).mochtr_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).mochtr_var_C as uint32)
            as int32,
    ) != 0
    {
        (*E).mochtr_var_C = 0 as uint16;
        (*E).mochtr_var_D = 0 as uint16;
    }
    t = (((((*E).base.x_pos as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >> 2 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as int32;
    AddToHiLo(
        &raw mut (*E).mochtr_var_B,
        &raw mut (*E).mochtr_var_A,
        -t as uint32,
    );
    let mut current_block_15: u64;
    if ((*E).mochtr_var_B as int16 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        if ((*E).mochtr_var_B as ::core::ffi::c_int) < 0xfffd as ::core::ffi::c_int {
            v10 = -(3 as ::core::ffi::c_int) as int16;
            current_block_15 = 10272573711837802396;
        } else {
            current_block_15 = 4808432441040389987;
        }
    } else if (*E).mochtr_var_B as ::core::ffi::c_int >= 3 as ::core::ffi::c_int {
        v10 = 3 as int16;
        current_block_15 = 10272573711837802396;
    } else {
        current_block_15 = 4808432441040389987;
    }
    match current_block_15 {
        10272573711837802396 => {
            (*E).mochtr_var_B = v10 as uint16;
            (*E).mochtr_var_A = 0 as uint16;
        }
        _ => {}
    }
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (((*E).mochtr_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).mochtr_var_A as uint32)
            as int32,
    ) != 0
    {
        (*E).mochtr_var_A = 0 as uint16;
        (*E).mochtr_var_B = 0 as uint16;
    }
    Mochtroid_Func_4(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        addr_kMochtroid_Ilist_A745 as ::core::ffi::c_int as uint16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Mochtroid_Func_2() {
    let mut E: *mut Enemy_Mochtroid = Get_Mochtroid(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v2: ::core::ffi::c_int = ((*E).mochtr_var_E as ::core::ffi::c_int
        & 6 as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int;
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
        + g_word_A3A76D[v2 as usize] as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        + g_word_A3A775[v2 as usize] as ::core::ffi::c_int) as uint16;
    (*E).mochtr_var_A = 0 as uint16;
    (*E).mochtr_var_B = 0 as uint16;
    (*E).mochtr_var_C = 0 as uint16;
    (*E).mochtr_var_D = 0 as uint16;
    let fresh12 = (*E).mochtr_var_E;
    (*E).mochtr_var_E = (*E).mochtr_var_E.wrapping_sub(1);
    if fresh12 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).mochtr_var_F = 0 as uint16;
    }
    Mochtroid_Func_4(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        addr_kMochtroid_Ilist_A745 as ::core::ffi::c_int as uint16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Mochtroid_Func_3() {
    let mut E: *mut Enemy_Mochtroid = Get_Mochtroid(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut x_pos: uint16 = (*E).base.x_pos;
    if x_pos as ::core::ffi::c_int
        == *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    {
        (*E).mochtr_var_A = 0 as uint16;
        (*E).mochtr_var_B = 0 as uint16;
    } else if (x_pos as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).mochtr_var_A = 0 as uint16;
        (*E).mochtr_var_B = -(1 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).mochtr_var_A = 0 as uint16;
        (*E).mochtr_var_B = 1 as uint16;
    }
    Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (((*E).mochtr_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).mochtr_var_A as uint32)
            as int32,
    );
    let mut y_pos: uint16 = (*E).base.y_pos;
    if y_pos as ::core::ffi::c_int
        == *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    {
        (*E).mochtr_var_C = 0 as uint16;
        (*E).mochtr_var_D = 0 as uint16;
    } else if (y_pos as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).mochtr_var_C = 0 as uint16;
        (*E).mochtr_var_D = -(1 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).mochtr_var_C = 0 as uint16;
        (*E).mochtr_var_D = 1 as uint16;
    }
    Enemy_MoveDown(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (((*E).mochtr_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).mochtr_var_C as uint32)
            as int32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Mochtroid_Func_4(mut k: uint16, mut a: uint16) {
    let mut E: *mut Enemy_Mochtroid = Get_Mochtroid(k as ::core::ffi::c_int);
    if a as ::core::ffi::c_int != (*E).mochtr_var_01 as ::core::ffi::c_int {
        (*E).mochtr_var_01 = a;
        (*E).base.current_instruction = a;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Mochtroid_Touch() {
    let mut current_block: u64;
    let mut E: *mut Enemy_Mochtroid = Get_Mochtroid(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).mochtr_var_F = 1 as uint16;
    Mochtroid_Func_4(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        addr_kMochtroid_Ilist_A759 as ::core::ffi::c_int as uint16,
    );
    (*E).mochtr_var_20 = (*E).mochtr_var_20.wrapping_add(1);
    if *((&raw mut g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        current_block = 17285009320546006530;
    } else {
        if *((&raw mut g_ram as *mut uint8).offset(0xe44 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int
            == 7 as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 30 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            QueueSfx3_Max6(0x2d as uint16);
        }
        if (*E).mochtr_var_20 as ::core::ffi::c_int - 80 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).mochtr_var_20 = 0 as uint16;
            current_block = 17285009320546006530;
        } else {
            current_block = 13513818773234778473;
        }
    }
    match current_block {
        17285009320546006530 => {
            Enemy_NormalTouchAI_A3();
            *((&raw mut g_ram as *mut uint8).offset(0x18a8 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x18aa as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn Mochtroid_Shot() {
    Enemy_NormalShotAI_A3();
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max3(*jp);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Instr_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .sideh_var_04 = 1 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Init() {
    *((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize) as *mut uint16) =
        37 as uint16;
    NextRandom();
    let mut E: *mut Enemy_Sidehopper = Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).sideh_var_03 = 0 as uint16;
    (*E).sideh_var_04 = 0 as uint16;
    (*E).sideh_var_05 = 0 as uint16;
    let mut v2: uint16 = (2 as ::core::ffi::c_int
        * (*get_EnemyDef_A2((*E).base.enemy_ptr)).field_2A as ::core::ffi::c_int)
        as uint16;
    (*E).sideh_var_06 = v2;
    let mut v4: uint16 = 0;
    if (*E).sideh_parameter_1 != 0 {
        v4 = *(g_rom.offset(
            (((0xa3aaca as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3aaca as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(((*E).sideh_var_06 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    } else {
        v4 = *(g_rom.offset(
            (((0xa3aac2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3aac2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(((*E).sideh_var_06 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    }
    (*E).sideh_var_00 = v4;
    Sidehopper_Func_3();
    if (*get_EnemyDef_A2((*E).base.enemy_ptr)).field_2A != 0 {
        (*E).sideh_var_05 = 2 as uint16;
    }
    let mut v6: ::core::ffi::c_int =
        (*E).sideh_var_05 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).sideh_var_01 = Sidehopper_Func_2(
        *(g_rom.offset(
            (((0xa3aaee as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3aaee as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v6 as isize),
        *(g_rom.offset(
            (((0xa3aae6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3aae6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v6 as isize),
    );
    let mut v8: ::core::ffi::c_int =
        (*E).sideh_var_05 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).sideh_var_02 = Sidehopper_Func_2(
        *(g_rom.offset(
            (((0xa3aafa as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3aafa as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v8 as isize),
        *(g_rom.offset(
            (((0xa3aaf2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3aaf2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v8 as isize),
    );
    (*E).sideh_var_B = (0xa3abd6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_2(mut r22: uint16, mut r24: uint16) -> uint16 {
    let mut r18: uint16 = 0 as uint16;
    let mut r20: uint16 = 0 as uint16;
    loop {
        r18 = (r18 as ::core::ffi::c_int + r24 as ::core::ffi::c_int) as uint16;
        r20 = (r20 as ::core::ffi::c_int
            + *((g_rom.offset(
                (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16 as *mut uint8)
                .offset((8 as ::core::ffi::c_int * r18 as ::core::ffi::c_int) as isize)
                .offset(1 as ::core::ffi::c_int as isize) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
        if !(r20 as ::core::ffi::c_int - r22 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0)
        {
            break;
        }
    }
    return r18;
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_3() {
    let mut E: *mut Enemy_Sidehopper = Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.current_instruction = (*E).sideh_var_00;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Main() {
    let mut E: *mut Enemy_Sidehopper = Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallSidehopperFunc(
        ((*E).sideh_var_B as ::core::ffi::c_int | 0xa30000 as ::core::ffi::c_int) as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_4(mut k: uint16) {
    let mut v1: uint16 = *(g_rom.offset(
        (((0xa3aae2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3aae2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((NextRandom() as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as isize);
    (*Get_Sidehopper(k as ::core::ffi::c_int)).sideh_var_B = v1;
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_5() {
    let mut E: *mut Enemy_Sidehopper = Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v1: ::core::ffi::c_int =
        (*E).sideh_var_05 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).sideh_var_E = *(g_rom.offset(
        (((0xa3aae6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3aae6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize);
    (*E).sideh_var_D = *(g_rom.offset(
        (((0xa3aaea as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3aaea as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize);
    (*E).sideh_var_C = (*E).sideh_var_01;
    (*E).sideh_var_B = (0xa3ac40 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if (*E).sideh_parameter_1 != 0 {
        (*E).sideh_var_B =
            (0xa3ac56 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_6() {
    let mut E: *mut Enemy_Sidehopper = Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v1: ::core::ffi::c_int =
        (*E).sideh_var_05 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).sideh_var_E = *(g_rom.offset(
        (((0xa3aaf2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3aaf2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize);
    (*E).sideh_var_D = *(g_rom.offset(
        (((0xa3aaf6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3aaf6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize);
    (*E).sideh_var_C = (*E).sideh_var_02;
    (*E).sideh_var_B = (0xa3ac40 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if (*E).sideh_parameter_1 != 0 {
        (*E).sideh_var_B =
            (0xa3ac56 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_7() {
    let mut E: *mut Enemy_Sidehopper = Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).sideh_var_B = (0xa3ac6c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if GetSamusEnemyDelta_X(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    ) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).sideh_var_B =
            (0xa3ac8f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_8() {
    let mut E: *mut Enemy_Sidehopper = Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).sideh_var_B = (0xa3aca8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if GetSamusEnemyDelta_X(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    ) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).sideh_var_B =
            (0xa3accb as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_9() {
    let mut E: *mut Enemy_Sidehopper = Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).sideh_var_D = -((*E).sideh_var_D as ::core::ffi::c_int) as uint16;
    (*E).sideh_var_00 = *(g_rom.offset(
        (((0xa3aad2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3aad2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(((*E).sideh_var_06 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Sidehopper_Func_3();
    (*E).sideh_var_B = (0xa3ad0e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_10() {
    let mut E: *mut Enemy_Sidehopper = Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).sideh_var_00 = *(g_rom.offset(
        (((0xa3aad2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3aad2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(((*E).sideh_var_06 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Sidehopper_Func_3();
    (*E).sideh_var_B = (0xa3ad20 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_11() {
    let mut E: *mut Enemy_Sidehopper = Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).sideh_var_D = -((*E).sideh_var_D as ::core::ffi::c_int) as uint16;
    (*E).sideh_var_00 = *(g_rom.offset(
        (((0xa3aada as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3aada as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(((*E).sideh_var_06 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Sidehopper_Func_3();
    (*E).sideh_var_B = (0xa3ad32 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_12() {
    let mut E: *mut Enemy_Sidehopper = Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).sideh_var_00 = *(g_rom.offset(
        (((0xa3aada as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3aada as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(((*E).sideh_var_06 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Sidehopper_Func_3();
    (*E).sideh_var_B = (0xa3ad44 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_13() {
    let mut E: *mut Enemy_Sidehopper = Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).sideh_var_00 = *(g_rom.offset(
        (((0xa3aac2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3aac2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(((*E).sideh_var_06 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    if (*E).sideh_parameter_1 != 0 {
        (*E).sideh_var_00 = *(g_rom.offset(
            (((0xa3aaca as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3aaca as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(((*E).sideh_var_06 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    }
    Sidehopper_Func_3();
    (*E).sideh_var_B = (0xa3ad56 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_14() {
    if (*Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .sideh_var_03
        != 0
    {
        Sidehopper_Func_20();
    } else {
        Sidehopper_Func_19();
    };
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_15() {
    if (*Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .sideh_var_03
        != 0
    {
        Sidehopper_Func_20();
    } else {
        Sidehopper_Func_19();
    };
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_16() {
    if (*Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .sideh_var_03
        != 0
    {
        Sidehopper_Func_22();
    } else {
        Sidehopper_Func_21();
    };
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_17() {
    if (*Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .sideh_var_03
        != 0
    {
        Sidehopper_Func_22();
    } else {
        Sidehopper_Func_21();
    };
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_18() {
    let mut E: *mut Enemy_Sidehopper = Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).sideh_var_04 != 0 {
        (*E).sideh_var_04 = 0 as uint16;
        (*E).sideh_var_B =
            (0xa3abd6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
unsafe extern "C" fn CallSidehopperFunc(mut ea: uint32) {
    match ea {
        10726358 => {
            Sidehopper_Func_4(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
            return;
        }
        10726374 => {
            Sidehopper_Func_5();
            return;
        }
        10726419 => {
            Sidehopper_Func_6();
            return;
        }
        10726464 => {
            Sidehopper_Func_7();
            return;
        }
        10726486 => {
            Sidehopper_Func_8();
            return;
        }
        10726508 => {
            Sidehopper_Func_9();
            return;
        }
        10726543 => {
            Sidehopper_Func_10();
            return;
        }
        10726568 => {
            Sidehopper_Func_11();
            return;
        }
        10726603 => {
            Sidehopper_Func_12();
            return;
        }
        10726628 => {
            Sidehopper_Func_13();
            return;
        }
        10726670 => {
            Sidehopper_Func_14();
            return;
        }
        10726688 => {
            Sidehopper_Func_15();
            return;
        }
        10726706 => {
            Sidehopper_Func_16();
            return;
        }
        10726724 => {
            Sidehopper_Func_17();
            return;
        }
        10726742 => {
            Sidehopper_Func_18();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_19() {
    let mut E: *mut Enemy_Sidehopper = Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v2: ::core::ffi::c_int =
        8 as ::core::ffi::c_int * (*E).sideh_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        ((*(g_rom.offset(
            (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v2 + 3 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(g_rom.offset(
                (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 + 2 as ::core::ffi::c_int) as isize) as uint32) as int32,
    ) != 0
    {
        (*E).sideh_var_D = -((*E).sideh_var_D as ::core::ffi::c_int) as uint16;
        (*E).sideh_var_03 = 1 as uint16;
    } else if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        ((*E).sideh_var_D as int16 as int32) << 16 as ::core::ffi::c_int,
    ) != 0
    {
        (*E).sideh_var_D = -((*E).sideh_var_D as ::core::ffi::c_int) as uint16;
        (*E).sideh_var_03 = 1 as uint16;
    } else {
        let mut v3: int16 = ((*E).sideh_var_C as ::core::ffi::c_int
            - (*E).sideh_var_E as ::core::ffi::c_int) as int16;
        (*E).sideh_var_C = v3 as uint16;
        if (v3 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            (*E).sideh_var_03 = 1 as uint16;
            (*E).sideh_var_C = 0 as uint16;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_20() {
    let mut E: *mut Enemy_Sidehopper = Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v2: ::core::ffi::c_int =
        8 as ::core::ffi::c_int * (*E).sideh_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        ((*(g_rom.offset(
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
                .offset(v2 as isize) as uint32) as int32,
    ) != 0
    {
        (*E).sideh_var_03 = 0 as uint16;
        (*E).sideh_var_B =
            (0xa3ace4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        if Enemy_MoveRight_IgnoreSlopes(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            ((*E).sideh_var_D as int16 as int32) << 16 as ::core::ffi::c_int,
        ) != 0
        {
            (*E).sideh_var_D = -((*E).sideh_var_D as ::core::ffi::c_int) as uint16;
        }
        let mut v3: int16 = ((*E).sideh_var_E as ::core::ffi::c_int
            + (*E).sideh_var_C as ::core::ffi::c_int) as int16;
        if v3 as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v3 = 64 as int16;
        }
        (*E).sideh_var_C = v3 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_21() {
    let mut E: *mut Enemy_Sidehopper = Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v2: ::core::ffi::c_int =
        8 as ::core::ffi::c_int * (*E).sideh_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        ((*(g_rom.offset(
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
                .offset(v2 as isize) as uint32) as int32,
    ) != 0
    {
        (*E).sideh_var_D = -((*E).sideh_var_D as ::core::ffi::c_int) as uint16;
        (*E).sideh_var_03 = 1 as uint16;
    } else if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        ((*E).sideh_var_D as int16 as int32) << 16 as ::core::ffi::c_int,
    ) != 0
    {
        (*E).sideh_var_D = -((*E).sideh_var_D as ::core::ffi::c_int) as uint16;
        (*E).sideh_var_03 = 1 as uint16;
    } else {
        let mut v3: int16 = ((*E).sideh_var_C as ::core::ffi::c_int
            - (*E).sideh_var_E as ::core::ffi::c_int) as int16;
        (*E).sideh_var_C = v3 as uint16;
        if (v3 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            (*E).sideh_var_03 = 1 as uint16;
            (*E).sideh_var_C = 0 as uint16;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Sidehopper_Func_22() {
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_Sidehopper = Get_Sidehopper(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v2: ::core::ffi::c_int =
        8 as ::core::ffi::c_int * (*E).sideh_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        ((*(g_rom.offset(
            (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v2 + 3 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(g_rom.offset(
                (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 + 2 as ::core::ffi::c_int) as isize) as uint32) as int32,
    ) != 0
    {
        (*E).sideh_var_03 = 0 as uint16;
        (*E).sideh_var_B =
            (0xa3ace4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        if Enemy_MoveRight_IgnoreSlopes(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            ((*E).sideh_var_D as int16 as int32) << 16 as ::core::ffi::c_int,
        ) != 0
        {
            (*E).sideh_var_D = -((*E).sideh_var_D as ::core::ffi::c_int) as uint16;
        }
        v3 = ((*E).sideh_var_E as ::core::ffi::c_int + (*E).sideh_var_C as ::core::ffi::c_int)
            as int16;
        if v3 as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v3 = 64 as int16;
        }
        (*E).sideh_var_C = v3 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaRefillCandy_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    (*Get_MaridiaRefillCandy(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .mrcy_var_00 = 4 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaRefillCandy_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    (*Get_MaridiaRefillCandy(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .mrcy_var_00 = 8 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaRefillCandy_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    (*Get_MaridiaRefillCandy(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .mrcy_var_00 = 12 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaRefillCandy_Init() {
    let mut E: *mut Enemy_MaridiaRefillCandy = Get_MaridiaRefillCandy(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).mrcy_parameter_1 =
        (0xa3b482 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mrcy_var_D = 0 as uint16;
    (*E).mrcy_var_E = 0 as uint16;
    (*E).mrcy_var_00 = 0 as uint16;
    (*E).base.current_instruction =
        addr_kMaridiaRefillCandy_Ilist_B3C1 as ::core::ffi::c_int as uint16;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
    (*E).mrcy_var_B = (*E).base.x_pos;
    (*E).mrcy_var_C = (*E).base.y_pos;
}
#[no_mangle]
pub unsafe extern "C" fn CallMaridiaRefillCandyFunc(mut ea: uint32, mut k: uint16) {
    match ea {
        10728578 => {
            MaridiaRefillCandy_Func_1();
            return;
        }
        10728616 => {
            MaridiaRefillCandy_Func_2(k);
            return;
        }
        10728662 => {
            MaridiaRefillCandy_Func_3(k);
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaRefillCandy_Main() {
    let mut E: *mut Enemy_MaridiaRefillCandy = Get_MaridiaRefillCandy(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallMaridiaRefillCandyFunc(
        ((*E).mrcy_parameter_1 as ::core::ffi::c_int | 0xa30000 as ::core::ffi::c_int) as uint32,
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaRefillCandy_Func_1() {
    let mut v1: int16 = 0;
    if IsSamusWithinEnemy_X(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        0x80 as uint16,
    ) != 0
    {
        v1 = 1 as int16;
        if GetSamusEnemyDelta_X(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        ) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            v1 = 3 as int16;
        }
        let mut E: *mut Enemy_MaridiaRefillCandy = Get_MaridiaRefillCandy(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        );
        (*E).mrcy_var_D = v1 as uint16;
        MaridiaRefillCandy_Func_4();
        (*E).mrcy_parameter_1 =
            (0xa3b4a8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaRefillCandy_Func_2(mut k: uint16) {
    let mut E: *mut Enemy_MaridiaRefillCandy = Get_MaridiaRefillCandy(k as ::core::ffi::c_int);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    if GetSamusEnemyDelta_Y(k) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            -(0x8000 as ::core::ffi::c_int) as uint32,
        );
    } else {
        (*E).mrcy_var_D = (*E).mrcy_var_D.wrapping_sub(1);
        MaridiaRefillCandy_Func_4();
        (*E).mrcy_var_F = 0 as uint16;
        (*E).mrcy_parameter_1 =
            (0xa3b4d6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaRefillCandy_Func_3(mut k: uint16) {
    let mut E: *mut Enemy_MaridiaRefillCandy = Get_MaridiaRefillCandy(k as ::core::ffi::c_int);
    let mut v2: ::core::ffi::c_int =
        (*E).mrcy_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v: int32 = ((*(g_rom.offset(
        (((0xa3b415 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3b415 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v2 as isize) as uint32)
        << 16 as ::core::ffi::c_int
        | *(g_rom.offset(
            (((0xa3b415 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3b415 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize) as uint32)
        as int32;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (if (*E).mrcy_var_D as ::core::ffi::c_int != 0 {
            v
        } else {
            -v
        }) as uint32,
    );
    if CheckIfEnemyIsOnScreen() == 0 {
        MaridiaRefillCandy_Func_4();
        return;
    }
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
    (*E).base.x_pos = (*E).mrcy_var_B;
    (*E).base.y_pos = (*E).mrcy_var_C;
    (*E).mrcy_var_D = 0 as uint16;
    MaridiaRefillCandy_Func_4();
    (*E).mrcy_parameter_1 =
        (0xa3b482 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaRefillCandy_Func_4() {
    let mut E: *mut Enemy_MaridiaRefillCandy = Get_MaridiaRefillCandy(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut mrcy_var_D: uint16 = (*E).mrcy_var_D;
    if mrcy_var_D as ::core::ffi::c_int != (*E).mrcy_var_E as ::core::ffi::c_int {
        (*E).mrcy_var_E = mrcy_var_D;
        (*E).base.current_instruction = *(g_rom.offset(
            (((0xa3b40d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3b40d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(mrcy_var_D as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaRefillCandy_Func_5() {}
#[no_mangle]
pub unsafe extern "C" fn MaridiaRefillCandy_Func_6() {}
#[no_mangle]
pub unsafe extern "C" fn NorfairSlowFireball_Init() {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v1).parameter_2 = 6 as uint16;
    (*v1).current_instruction = *(g_rom.offset(
        (((0xa3b667 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3b667 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(((*v1).properties as ::core::ffi::c_int & 3 as ::core::ffi::c_int) as isize);
    StoneZoomer_E67A(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn NorfairSlowFireball_Func_1() {
    let mut current_instruction: uint16 = (*gEnemyData(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .current_instruction;
    if current_instruction != 0 {
        Unreachable();
    }
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Instr_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max6(0x56 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Instr_2(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .bang_var_22 = 1 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Init() {
    let mut E: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).bang_var_B = (*E).base.palette_index;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_BlockPlasmaBeam as ::core::ffi::c_int) as uint16;
    (*E).bang_var_F = (0xa3bca5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).bang_var_00 = 16 as uint16;
    (*E).bang_var_01 = 0 as uint16;
    (*E).bang_var_02 = 0 as uint16;
    (*E).bang_var_20 = 0 as uint16;
    (*E).bang_var_21 = 0 as uint16;
    (*E).bang_var_22 = 0 as uint16;
    (*E).bang_var_0B = *(g_rom.offset(
        (((0xa3ba84 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3ba84 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *(&raw mut (*E).bang_parameter_2 as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as isize,
        );
    let mut v1: ::core::ffi::c_int = 2 as ::core::ffi::c_int
        * *(&raw mut (*E).bang_parameter_2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int;
    let mut v2: uint16 = *(g_rom.offset(
        (((0xa3bc6a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3bc6a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize);
    (*E).bang_var_0C = v2;
    (*E).bang_var_0D = v2;
    (*E).bang_var_0E = *(g_rom.offset(
        (((0xa3bc6a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3bc6a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v1 + 1 as ::core::ffi::c_int) as isize);
    if (*E).base.current_instruction == 0 {
        (*E).bang_var_F = (0xa3bcc1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    (*E).base.current_instruction = addr_kBang_Ilist_B75E as ::core::ffi::c_int as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Main() {
    let mut E: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    Call(((*E).bang_parameter_1 as ::core::ffi::c_int | 0xa30000 as ::core::ffi::c_int) as uint32);
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Func_1() {
    let mut v0: uint16 = ((*Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
    ))
    .bang_var_20 as ::core::ffi::c_int
        + 10 as ::core::ffi::c_int) as uint16;
    (*Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .bang_var_20 = v0;
    Bang_Func_18();
    let mut E: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Func_2() {
    let mut v0: uint16 = ((*Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 1984 as ::core::ffi::c_int,
    ))
    .bang_var_00 as ::core::ffi::c_int
        + 20 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).bang_var_20 = v0;
    Bang_Func_18();
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Func_3() {
    let mut bang_var_B: int16 = 0;
    Bang_Func_5();
    let mut E0: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut x_pos: uint16 = (*E0).base.x_pos;
    let mut v3: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0xe54 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xe84 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((v3 + 91 as ::core::ffi::c_int) as isize) = x_pos;
    let mut E1: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
    );
    (*E1).base.x_pos = x_pos;
    let mut y_pos: uint16 = (*E0).base.y_pos;
    *((&raw mut g_ram as *mut uint8).offset(0xe84 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((v3 + 93 as ::core::ffi::c_int) as isize) = y_pos;
    (*E1).base.y_pos = y_pos;
    bang_var_B = (*E0).bang_var_B as int16;
    if (*E0).bang_var_20 as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        bang_var_B = 3072 as int16;
    }
    (*E0).base.palette_index = bang_var_B as uint16;
    Bang_Func_18();
    if (*E0).bang_var_22 != 0 {
        (*E0).bang_var_22 = 0 as uint16;
        if (*E0).bang_var_20 as ::core::ffi::c_int == 9 as ::core::ffi::c_int {
            (*E0).base.invincibility_timer = 16 as uint16;
            (*E0).base.properties = ((*E0).base.properties as ::core::ffi::c_int
                | kEnemyProps_Tangible as ::core::ffi::c_int)
                as uint16;
            let mut v7: uint16 = DetermineDirectionOfSamusFromEnemy();
            let mut v8: uint16 = Bang_Func_4(v7);
            EnemyDeathAnimation(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                v8,
            );
            let mut v9: uint16 = *((&raw mut g_ram as *mut uint8)
                .offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16);
            let mut E: *mut Enemy_Bang = Get_Bang(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + 64 as ::core::ffi::c_int,
            );
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                | 0x200 as ::core::ffi::c_int) as uint16;
            let ref mut fresh1 = *((&raw mut g_ram as *mut uint8)
                .offset(0xe84 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(
                    ((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 97 as ::core::ffi::c_int) as isize,
                );
            *fresh1 = (*fresh1 as ::core::ffi::c_int | 0x200 as ::core::ffi::c_int) as uint16;
        } else {
            (*E0).bang_var_20 = (*E0).bang_var_20.wrapping_add(1);
            Bang_Func_18();
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Func_4(mut a: uint16) -> uint16 {
    let mut i: ::core::ffi::c_int = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0xcce as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 5 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        return 1 as uint16;
    }
    i = 0 as ::core::ffi::c_int;
    while *((&raw mut g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((i >> 1 as ::core::ffi::c_int) as isize)
        != 0
    {
        i += 2 as ::core::ffi::c_int;
    }
    let mut E: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v3: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v3 as isize) = (*E).base.x_pos;
    *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v3 as isize) = (*E).base.y_pos;
    *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v3 as isize) = a;
    *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v3 as isize) = (*((&raw mut g_ram as *mut uint8)
        .offset(0x9a6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        | 0x10 as ::core::ffi::c_int) as uint16;
    let ref mut fresh0 = *((&raw mut g_ram as *mut uint8)
        .offset(0xcce as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh0 = (*fresh0).wrapping_add(1);
    ProjectileReflection(i as uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v3 as isize) = (*E).bang_var_E;
    QueueSfx1_Max6(
        *(g_rom.offset(
            (((0xa3bc4a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3bc4a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as isize,
            ),
    );
    return 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Func_5() {
    let mut E: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallEnemyPreInstr(
        ((*E).bang_var_F as ::core::ffi::c_int | 0xa30000 as ::core::ffi::c_int) as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Func_6(mut k: uint16) {
    let mut E: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v1: uint16 =
        ((*E).bang_var_00 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E).bang_var_00 = v1;
    if v1 == 0 {
        (*E).bang_var_00 = 16 as uint16;
        (*E).bang_var_F = (0xa3bcc5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Func_7(mut k: uint16) {}
#[no_mangle]
pub unsafe extern "C" fn Bang_Func_8(mut k: uint16) {
    let mut v1: uint8 = (CalculateAngleOfSamusFromEnemy(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    ) as ::core::ffi::c_int
        - 64 as ::core::ffi::c_int) as uint8;
    let mut E: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).bang_var_01 = v1 as uint16;
    (*E).bang_var_F = (0xa3bd1c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).bang_var_07 = 0 as uint16;
    (*E).bang_var_08 = 0 as uint16;
    (*E).bang_var_09 = 0 as uint16;
    (*E).bang_var_0A = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Func_9() {
    let mut v1: uint8 = (CalculateAngleOfSamusFromEnemy(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    ) as ::core::ffi::c_int
        - 64 as ::core::ffi::c_int) as uint8;
    let mut E: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).bang_var_02 = v1 as uint16;
    let mut v3: uint16 =
        SignExtend8((v1 as ::core::ffi::c_int - (*E).bang_var_01 as ::core::ffi::c_int) as uint16);
    let mut v4: uint16 = Abs16(v3);
    if v4 as ::core::ffi::c_int - 48 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        (*E).bang_var_F = (0xa3bd2c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Func_10(mut k: uint16) {
    Bang_Func_14();
    Bang_Func_15();
    Bang_Func_12();
    Bang_Func_9();
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Func_11(mut k: uint16) {
    Bang_Func_14();
    Bang_Func_15();
    Bang_Func_13();
    let mut E: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).bang_var_0A as ::core::ffi::c_int != 0
        || (*E).bang_var_09 as int16 as ::core::ffi::c_int <= 0 as ::core::ffi::c_int
    {
        (*E).bang_var_F = (0xa3bca5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Func_12() {
    let mut E: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v1: int16 = ((*E).bang_var_0C as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
    (*E).bang_var_0C = v1 as uint16;
    if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).bang_var_0C = (*E).bang_var_0D;
        if (((*E).bang_var_0A as ::core::ffi::c_int - (*E).bang_var_0B as ::core::ffi::c_int)
            as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            (*E).bang_var_09 =
                ((*E).bang_var_09 as ::core::ffi::c_int + 22 as ::core::ffi::c_int) as uint16;
            (*E).bang_var_0A = ((*E).bang_var_0A as ::core::ffi::c_int
                + (*E).bang_var_09 as ::core::ffi::c_int) as uint16;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Func_13() {
    let mut E: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v1: int16 = ((*E).bang_var_0C as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
    (*E).bang_var_0C = v1 as uint16;
    if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).bang_var_0C = (*E).bang_var_0E;
        (*E).bang_var_09 =
            ((*E).bang_var_09 as ::core::ffi::c_int - 22 as ::core::ffi::c_int) as uint16;
        (*E).bang_var_0A = ((*E).bang_var_0A as ::core::ffi::c_int
            - (*E).bang_var_09 as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Func_14() {
    let mut E: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut r18: uint16 = *(g_rom.offset(
        (((0xa0b1c3 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa0b1c3 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((*E).bang_var_01 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint8 as isize,
        );
    if r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        (*E).bang_var_07 = 1 as uint16;
    }
    let mut t: uint32 = ((Abs16(r18) as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
        * (*E).bang_var_0A as ::core::ffi::c_int) as uint32;
    if (*E).bang_var_07 != 0 {
        t = -(t as int32) as uint32;
    }
    t = t.wrapping_add((*E).base.x_subpos as uint32);
    (*E).base.x_subpos = t as uint16;
    (*E).base.x_pos = (t >> 16 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Func_15() {
    let mut E: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut r18: uint16 = *(g_rom.offset(
        (((0xa0b1c3 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa0b1c3 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((*E).bang_var_01 as uint8 as isize);
    if r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        (*E).bang_var_08 = 1 as uint16;
    }
    let mut t: uint32 = ((Abs16(r18) as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
        * (*E).bang_var_0A as ::core::ffi::c_int) as uint32;
    if (*E).bang_var_08 != 0 {
        t = -(t as int32) as uint32;
    }
    t = t.wrapping_add((*E).base.y_subpos as uint32);
    (*E).base.y_subpos = t as uint16;
    (*E).base.y_pos = (t >> 16 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Func_18() {
    let mut E: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut bang_var_20: uint16 = (*E).bang_var_20;
    if bang_var_20 as ::core::ffi::c_int != (*E).bang_var_21 as ::core::ffi::c_int {
        (*E).bang_var_21 = bang_var_20;
        (*E).base.current_instruction = *(g_rom.offset(
            (((0xa3b722 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3b722 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(bang_var_20 as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Bang_Shot() {
    let mut E: *mut Enemy_Bang = Get_Bang(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).bang_var_F as ::core::ffi::c_int != 0xbcc1 as ::core::ffi::c_int {
        (*E).bang_var_01 = *(g_rom.offset(
            (((0xa3ba94 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3ba94 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0x18a6 as ::core::ffi::c_int as isize)
                            as *mut uint16) as isize,
                    ) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as isize,
            );
        (*E).bang_var_F = (0xa3bd2c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).bang_var_07 = 0 as uint16;
        (*E).bang_var_08 = 0 as uint16;
        (*E).bang_var_09 = 256 as uint16;
        (*E).bang_var_0A = 1536 as uint16;
    }
    if (*E).bang_var_20 as ::core::ffi::c_int != 9 as ::core::ffi::c_int {
        (*E).bang_var_20 = (*E).bang_var_20.wrapping_add(1);
        Bang_Func_18();
        let mut v1: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x18a6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int;
        (*E).bang_var_E = ((*E).bang_var_E as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
        let ref mut fresh2 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc04 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh2 = (*fresh2 as ::core::ffi::c_int | 0x10 as ::core::ffi::c_int) as uint16;
        if (*E).bang_var_20 as ::core::ffi::c_int == 9 as ::core::ffi::c_int {
            (*E).bang_var_D = 1 as uint16;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Skree_Instr_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Skree(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .skree_var_E = 1 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Skree_Init() {
    let mut E: *mut Enemy_Skree = Get_Skree(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).skree_var_C = 0 as uint16;
    (*E).skree_var_D = 0 as uint16;
    (*E).skree_var_E = 0 as uint16;
    (*E).base.current_instruction = addr_kSkree_Ilist_C65E as ::core::ffi::c_int as uint16;
    (*E).skree_var_B = (0xa3c6d5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn CallSkreeFunc(mut ea: uint32) {
    match ea {
        10733269 => {
            Skree_Func_1();
            return;
        }
        10733303 => {
            Skree_Func_2(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
            return;
        }
        10733334 => {
            Skree_Func_3();
            return;
        }
        10733439 => {
            Skree_Func_4();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Skree_Main() {
    let mut E: *mut Enemy_Skree = Get_Skree(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallSkreeFunc(
        ((*E).skree_var_B as ::core::ffi::c_int | 0xa30000 as ::core::ffi::c_int) as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Skree_Func_1() {
    let mut E: *mut Enemy_Skree = Get_Skree(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (abs16(
        ((*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
    ) as ::core::ffi::c_int)
        < 0x30 as ::core::ffi::c_int
    {
        (*E).skree_var_C = (*E).skree_var_C.wrapping_add(1);
        Skree_Func_5();
        (*E).skree_var_B =
            (0xa3c6f7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Skree_Func_2(mut k: uint16) {
    let mut E: *mut Enemy_Skree = Get_Skree(k as ::core::ffi::c_int);
    if (*E).skree_var_E != 0 {
        (*E).skree_var_E = 0 as uint16;
        (*E).skree_var_C = (*E).skree_var_C.wrapping_add(1);
        Skree_Func_5();
        (*E).skree_var_B =
            (0xa3c716 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        QueueSfx2_Max6(0x5b as uint16);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Skree_Func_3() {
    let mut E: *mut Enemy_Skree = Get_Skree(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).skree_var_A = 21 as uint16;
    let mut v1: uint16 =
        ((*E).base.properties as ::core::ffi::c_int | 3 as ::core::ffi::c_int) as uint16;
    (*E).base.properties = v1;
    if EnemyFunc_BF8A(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        v1,
        (6 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).skree_var_B =
            (0xa3c77f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        QueueSfx2_Max6(0x5c as uint16);
    } else {
        (*E).base.y_pos =
            ((*E).base.y_pos as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
        (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
            + if ((*E).base.x_pos as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                -(1 as ::core::ffi::c_int)
            } else {
                1 as ::core::ffi::c_int
            }) as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Skree_Func_4() {
    let mut E: *mut Enemy_Skree = Get_Skree(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let fresh18 = (*E).skree_var_A;
    (*E).skree_var_A = (*E).skree_var_A.wrapping_sub(1);
    if fresh18 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*gEnemySpawnData(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        ))
        .vram_tiles_index = ((*E).base.vram_tiles_index as ::core::ffi::c_int
            | (*E).base.palette_index as ::core::ffi::c_int) as uint16;
        (*E).base.palette_index = 2560 as uint16;
        (*E).base.vram_tiles_index = 0 as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
    } else {
        if (*E).skree_var_A as ::core::ffi::c_int == 8 as ::core::ffi::c_int {
            let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
                .offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16);
            SpawnEprojWithGfx(
                8 as uint16,
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                addr_stru_868BC2 as ::core::ffi::c_int as uint16,
            );
            SpawnEprojWithGfx(
                0 as uint16,
                v2,
                addr_stru_868BD0 as ::core::ffi::c_int as uint16,
            );
            SpawnEprojWithGfx(
                0 as uint16,
                v2,
                addr_stru_868BDE as ::core::ffi::c_int as uint16,
            );
            SpawnEprojWithGfx(
                0 as uint16,
                v2,
                addr_stru_868BEC as ::core::ffi::c_int as uint16,
            );
        }
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    };
}
#[no_mangle]
pub unsafe extern "C" fn Skree_Func_5() {
    let mut E: *mut Enemy_Skree = Get_Skree(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut skree_var_C: uint16 = (*E).skree_var_C;
    if skree_var_C as ::core::ffi::c_int != (*E).skree_var_D as ::core::ffi::c_int {
        (*E).skree_var_D = skree_var_C;
        (*E).base.current_instruction = *(g_rom.offset(
            (((0xa3c69c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3c69c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(skree_var_C as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Skree_Shot() {
    Enemy_NormalShotAI_SkipSomeParts_A3();
    let mut E: *mut Enemy_Skree = Get_Skree(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).base.health == 0 {
        let mut v1: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16);
        SpawnEprojWithGfx(
            (*E).skree_var_A,
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            addr_stru_868BC2 as ::core::ffi::c_int as uint16,
        );
        SpawnEprojWithGfx(
            0 as uint16,
            v1,
            addr_stru_868BD0 as ::core::ffi::c_int as uint16,
        );
        SpawnEprojWithGfx(
            0 as uint16,
            v1,
            addr_stru_868BDE as ::core::ffi::c_int as uint16,
        );
        SpawnEprojWithGfx(
            0 as uint16,
            v1,
            addr_stru_868BEC as ::core::ffi::c_int as uint16,
        );
        let mut v5: uint16 = 2 as uint16;
        if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0x18a6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int
            & 0xf00 as ::core::ffi::c_int
            != 512 as ::core::ffi::c_int
        {
            v5 = 0 as uint16;
        }
        EnemyDeathAnimation(
            (2 as ::core::ffi::c_int
                * *((&raw mut g_ram as *mut uint8).offset(0x18a6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16,
            v5,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    (*Get_MaridiaSnail(k as ::core::ffi::c_int)).msl_var_F = *jp;
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    (*Get_MaridiaSnail(k as ::core::ffi::c_int)).msl_var_D = *jp;
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Instr_4(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    (*E).msl_var_07 = *jp.offset(0 as ::core::ffi::c_int as isize);
    (*E).msl_var_C = *((&raw mut (*(g_rom.offset(
        (((0xa3cd42 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3cd42 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut MaridiaSnailData2)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_6 as *mut uint8)
        .offset((8 as ::core::ffi::c_int * (*E).msl_var_07 as ::core::ffi::c_int) as isize)
        as *mut uint16);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
        + *jp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        as uint16;
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        + *jp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        as uint16;
    return jp.offset(2 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Instr_5(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    if (*Get_MaridiaSnail(k as ::core::ffi::c_int)).msl_var_08 as ::core::ffi::c_int
        == 2 as ::core::ffi::c_int
        || NextRandom() as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        jp = jp.offset(-(3 as ::core::ffi::c_int as isize));
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_1(mut k: uint16) {
    MaridiaSnail_Func_17(k);
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Init() {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).msl_var_F = (0xa3cf5f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A3 as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    let mut current_instruction: uint16 = (*E).base.current_instruction;
    (*E).base.current_instruction = (*(g_rom.offset(
        (((0xa3cd42 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3cd42 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut MaridiaSnailData2)
        .offset(current_instruction as isize))
    .field_0 as uint16;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (*(g_rom.offset(
            (((0xa3cd42 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3cd42 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut MaridiaSnailData2)
            .offset(current_instruction as isize))
        .field_2 as ::core::ffi::c_int) as uint16;
    (*E).msl_var_D = (*(g_rom.offset(
        (((0xa3cd42 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3cd42 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut MaridiaSnailData2)
        .offset(current_instruction as isize))
    .field_4 as uint16;
    (*E).msl_var_C = (*(g_rom.offset(
        (((0xa3cd42 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3cd42 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut MaridiaSnailData2)
        .offset(current_instruction as isize))
    .field_6;
    (*E).msl_var_08 = 0 as uint16;
    (*E).msl_var_06 = (*E).msl_parameter_1;
    MaridiaSnail_Func_2(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (current_instruction as ::core::ffi::c_int * 8 as ::core::ffi::c_int) as uint16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_2(mut k: uint16, mut j: uint16) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut r18: uint16 = *(g_rom.offset(
        (((0xa3cd82 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3cd82 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v2 as isize);
    let mut r20: uint16 = *(g_rom.offset(
        (((0xa3cd82 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3cd82 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v2 + 1 as ::core::ffi::c_int) as isize);
    let mut r22: uint16 = *(g_rom.offset(
        (((0xa3cd82 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3cd82 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v2 + 2 as ::core::ffi::c_int) as isize);
    let mut r24: uint16 = *(g_rom.offset(
        (((0xa3cd82 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3cd82 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v2 + 3 as ::core::ffi::c_int) as isize);
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    (*E).msl_var_A = (r20 as ::core::ffi::c_int
        + (r18 as ::core::ffi::c_int
            ^ *(g_rom.offset(
                (((0xa3cca2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa3cca2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((*E).msl_parameter_1 as isize) as ::core::ffi::c_int))
        as uint16;
    (*E).msl_var_B = (r24 as ::core::ffi::c_int
        + (r22 as ::core::ffi::c_int
            ^ *(g_rom.offset(
                (((0xa3cca2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa3cca2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((*E).msl_parameter_1 as isize) as ::core::ffi::c_int))
        as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_3(mut k: uint16) {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    (*E).msl_var_F = *(g_rom.offset(
        (((0xa3cdd2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3cdd2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((*E).msl_var_07 as isize);
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Main() {
    MaridiaSnail_Func_4(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    MaridiaSnail_Func_5(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    MaridiaSnail_Func_6(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallEnemyPreInstr(
        ((*E).msl_var_F as ::core::ffi::c_int | 0xa30000 as ::core::ffi::c_int) as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_4(mut k: uint16) {
    let mut msl_var_08: uint16 = (*Get_MaridiaSnail(k as ::core::ffi::c_int)).msl_var_08;
    if msl_var_08 as ::core::ffi::c_int != 3 as ::core::ffi::c_int
        && msl_var_08 as ::core::ffi::c_int != 4 as ::core::ffi::c_int
        && msl_var_08 as ::core::ffi::c_int != 5 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 30 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 20 as ::core::ffi::c_int
    {
        MaridiaSnail_Func_14(k);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_5(mut k: uint16) {
    let mut current_block: u64;
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    let mut msl_var_08: uint16 = (*E).msl_var_08;
    if msl_var_08 as ::core::ffi::c_int != 1 as ::core::ffi::c_int
        && msl_var_08 as ::core::ffi::c_int != 3 as ::core::ffi::c_int
        && msl_var_08 as ::core::ffi::c_int != 4 as ::core::ffi::c_int
        && msl_var_08 as ::core::ffi::c_int != 5 as ::core::ffi::c_int
    {
        if (*E).base.y_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            + 96 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            if (((*E).base.x_pos as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    != 4 as ::core::ffi::c_int
                {
                    current_block = 3898722140564953443;
                } else {
                    current_block = 6873731126896040597;
                }
            } else if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                != 8 as ::core::ffi::c_int
            {
                current_block = 3898722140564953443;
            } else {
                current_block = 6873731126896040597;
            }
            match current_block {
                3898722140564953443 => {}
                _ => {
                    if (*E).msl_var_08 as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
                        return;
                    }
                    if (*E).msl_var_D as ::core::ffi::c_int
                        != 0xa3cf5f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
                    {
                        (*E).base.current_instruction = (*E).msl_var_D;
                        (*E).base.spritemap_pointer =
                            addr_kSpritemap_Nothing_A3 as ::core::ffi::c_int as uint16;
                        (*E).base.instruction_timer = 1 as uint16;
                        (*E).base.timer = 0 as uint16;
                        (*E).msl_var_08 = 2 as uint16;
                        return;
                    }
                }
            }
        }
        (*E).msl_var_08 = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_6(mut k: uint16) {
    let mut current_block: u64;
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    if !((*E).msl_var_F as ::core::ffi::c_int
        == 0xa3d1b3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
    {
        if (*E).msl_var_08 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            if *((&raw mut g_ram as *mut uint8).offset(0xb3c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 0
                || (*E).msl_var_F as ::core::ffi::c_int
                    != 0xa3cf5f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            {
                current_block = 2304351028894610342;
            } else {
                current_block = 12573672619892793823;
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0xb3c as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            let mut msl_var_08: uint16 = (*E).msl_var_08;
            if msl_var_08 as ::core::ffi::c_int == 2 as ::core::ffi::c_int
                || msl_var_08 as ::core::ffi::c_int != 3 as ::core::ffi::c_int
                    && msl_var_08 as ::core::ffi::c_int != 5 as ::core::ffi::c_int
            {
                current_block = 12573672619892793823;
            } else {
                current_block = 2304351028894610342;
            }
        } else {
            current_block = 2304351028894610342;
        }
        match current_block {
            2304351028894610342 => {}
            _ => {
                (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                    | 0x8000 as ::core::ffi::c_int)
                    as uint16;
                return;
            }
        }
    }
    (*E).base.properties =
        ((*E).base.properties as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int) as uint16;
}
unsafe extern "C" fn Shift8AddMagn(mut a: int16, mut s: ::core::ffi::c_int) -> uint32 {
    let mut r: int32 = (a as int32) << 8 as ::core::ffi::c_int;
    r = (r as ::core::ffi::c_int
        + if r < 0 as int32 {
            -(s << 16 as ::core::ffi::c_int)
        } else {
            s << 16 as ::core::ffi::c_int
        }) as int32;
    return r as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_7(mut k: uint16) {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    if (*E).msl_var_07 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
        if EnemyFunc_BC76(
            k,
            Shift8AddMagn((*E).msl_var_B as int16, 7 as ::core::ffi::c_int) as int32,
        ) != 0
        {
            return;
        }
    } else if EnemyFunc_BBBF(
        k,
        Shift8AddMagn((*E).msl_var_A as int16, 7 as ::core::ffi::c_int) as int32,
    ) != 0
    {
        return;
    }
    MaridiaSnail_Func_14(k);
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_9(mut k: uint16) {
    let mut v1: uint16 =
        (if (*Get_MaridiaSnail(k as ::core::ffi::c_int)).msl_var_05 as ::core::ffi::c_int != 0 {
            addr_stru_A3CD22 as ::core::ffi::c_int
        } else {
            addr_stru_A3CCE2 as ::core::ffi::c_int
        }) as uint16;
    MaridiaSnail_D07E(k, RomPtr_A3(v1 as uint16_t));
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_CFB7(mut k: uint16) {
    MaridiaSnail_D002(
        k,
        RomPtr_A3(addr_stru_A3CCEA as ::core::ffi::c_int as uint16_t),
    );
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_CFBD(mut k: uint16) {
    let mut v1: uint16 =
        (if (*Get_MaridiaSnail(k as ::core::ffi::c_int)).msl_var_05 as ::core::ffi::c_int != 0 {
            addr_stru_A3CD2A as ::core::ffi::c_int
        } else {
            addr_stru_A3CCF2 as ::core::ffi::c_int
        }) as uint16;
    MaridiaSnail_D07E(k, RomPtr_A3(v1 as uint16_t));
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_CFCE(mut k: uint16) {
    MaridiaSnail_D002(
        k,
        RomPtr_A3(addr_stru_A3CCFA as ::core::ffi::c_int as uint16_t),
    );
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_CFD4(mut k: uint16) {
    let mut v1: uint16 =
        (if (*Get_MaridiaSnail(k as ::core::ffi::c_int)).msl_var_05 as ::core::ffi::c_int != 0 {
            addr_stru_A3CD32 as ::core::ffi::c_int
        } else {
            addr_stru_A3CD02 as ::core::ffi::c_int
        }) as uint16;
    MaridiaSnail_D07E(k, RomPtr_A3(v1 as uint16_t));
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_CFE5(mut k: uint16) {
    MaridiaSnail_D002(
        k,
        RomPtr_A3(addr_stru_A3CD0A as ::core::ffi::c_int as uint16_t),
    );
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_CFEB(mut k: uint16) {
    let mut v1: uint16 = 0;
    if (*Get_MaridiaSnail(k as ::core::ffi::c_int)).msl_var_05 != 0 {
        v1 = addr_stru_A3CD3A as ::core::ffi::c_int as uint16;
    } else {
        v1 = addr_stru_A3CD12 as ::core::ffi::c_int as uint16;
    }
    MaridiaSnail_D07E(k, RomPtr_A3(v1 as uint16_t));
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_CFFC(mut k: uint16) {
    MaridiaSnail_D002(
        k,
        RomPtr_A3(addr_stru_A3CD1A as ::core::ffi::c_int as uint16_t),
    );
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_D002(mut k: uint16, mut j: *const uint8) {
    MaridiaSnail_Func_10(k, j);
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    let mut rv: uint8 = Enemy_MoveRight_ProcessSlopes(
        k,
        Shift8AddMagn((*E).msl_var_A as int16, 1 as ::core::ffi::c_int) as int32,
    );
    MaridiaSnail_Func_11(k, j);
    if rv != 0 {
        (*E).msl_var_E = 0 as uint16;
        let mut carry: uint8 = EnemyFunc_C8AD(k);
        MaridiaSnail_Func_12(k, carry as uint16);
        if Enemy_MoveDown(
            k,
            ((*E).msl_var_B as int16 as int32) << 8 as ::core::ffi::c_int,
        ) != 0
        {
            (*E).msl_var_A = -((*E).msl_var_A as ::core::ffi::c_int) as uint16;
            MaridiaSnail_Func_13(
                k,
                *(j.offset(6 as ::core::ffi::c_int as isize) as *mut uint16),
            );
        }
    } else {
        let mut v7: uint16 =
            ((*E).msl_var_E as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E).msl_var_E = v7;
        if v7 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).msl_var_B = -((*E).msl_var_B as ::core::ffi::c_int) as uint16;
            MaridiaSnail_Func_13(
                k,
                *(j.offset(4 as ::core::ffi::c_int as isize) as *mut uint16),
            );
        } else {
            MaridiaSnail_Func_14(k);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_D07E(mut k: uint16, mut j: *const uint8) {
    MaridiaSnail_Func_10(k, j);
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    let mut rv: uint8 = Enemy_MoveDown(
        k,
        Shift8AddMagn((*E).msl_var_B as int16, 1 as ::core::ffi::c_int) as int32,
    );
    MaridiaSnail_Func_11(k, j);
    if rv != 0 {
        (*E).msl_var_E = 0 as uint16;
        if Enemy_MoveRight_ProcessSlopes(
            k,
            ((*E).msl_var_A as int16 as int32) << 8 as ::core::ffi::c_int,
        ) != 0
        {
            (*E).msl_var_B = -((*E).msl_var_B as ::core::ffi::c_int) as uint16;
            MaridiaSnail_Func_13(
                k,
                *(j.offset(6 as ::core::ffi::c_int as isize) as *mut uint16),
            );
        } else {
            let mut carry: uint8 = EnemyFunc_C8AD(k);
            MaridiaSnail_Func_12(k, carry as uint16);
        }
    } else {
        let mut v7: uint16 =
            ((*E).msl_var_E as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E).msl_var_E = v7;
        if v7 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).msl_var_A = -((*E).msl_var_A as ::core::ffi::c_int) as uint16;
            MaridiaSnail_Func_13(
                k,
                *(j.offset(4 as ::core::ffi::c_int as isize) as *mut uint16),
            );
        } else {
            MaridiaSnail_Func_14(k);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_10(mut k: uint16, mut j: *const uint8) {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
        + *(j as *mut uint16) as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        + *(j.offset(2 as ::core::ffi::c_int as isize) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_11(mut k: uint16, mut j: *const uint8) {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
        - *(j as *mut uint16) as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        - *(j.offset(2 as ::core::ffi::c_int as isize) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_12(mut k: uint16, mut carry: uint16) {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    if carry == 0 {
        if ((*E).msl_var_04 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
            >= 0x10 as ::core::ffi::c_int
        {
            (*E).msl_var_05 = 0 as uint16;
        } else {
            (*E).msl_var_04 = (*E).msl_var_04.wrapping_add(1);
        }
    } else {
        (*E).msl_var_05 = 1 as uint16;
        (*E).msl_var_04 = 0 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_13(mut k: uint16, mut a: uint16) {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    (*E).base.current_instruction = a;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).msl_var_05 = 1 as uint16;
    (*E).msl_var_04 = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_14(mut k: uint16) {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    if (*E).msl_var_08 as ::core::ffi::c_int != 3 as ::core::ffi::c_int {
        (*E).msl_var_08 = 3 as uint16;
        (*E).msl_var_F = (0xa3d1b3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut v2: ::core::ffi::c_int =
            2 as ::core::ffi::c_int * (*E).msl_var_C as ::core::ffi::c_int;
        (*E).base.current_instruction = *(g_rom.offset(
            (((0xa3d1ab as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3d1ab as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize);
        (*E).msl_var_D = *(g_rom.offset(
            (((0xa3d1ab as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3d1ab as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).msl_var_02 = 0 as uint16;
        (*E).msl_var_03 = 0 as uint16;
        (*E).msl_var_00 = 0 as uint16;
        (*E).msl_var_01 = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_15(mut k: uint16) {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    if (*E).msl_var_08 as ::core::ffi::c_int != 3 as ::core::ffi::c_int {
        if Enemy_MoveRight_IgnoreSlopes(
            k,
            (((*E).msl_var_03 as uint32) << 16 as ::core::ffi::c_int | (*E).msl_var_02 as uint32)
                as int32,
        ) != 0
        {
            (*E).msl_var_02 = -((*E).msl_var_02 as ::core::ffi::c_int) as uint16;
            (*E).msl_var_03 = -((*E).msl_var_03 as ::core::ffi::c_int) as uint16;
            (*E).msl_var_20 = 1 as uint16;
            QueueSfx2_Max3(0x70 as uint16);
        } else {
            let mut delta: uint32 = 0;
            if (*E).msl_var_03 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                delta = 4096 as uint32;
            } else {
                delta = -(4096 as ::core::ffi::c_int) as uint32;
            }
            let mut t: uint32 = (((*E).msl_var_03 as uint32) << 16 as ::core::ffi::c_int
                | (*E).msl_var_02 as uint32)
                .wrapping_add(delta);
            (*E).msl_var_02 = t as uint16;
            if t >> 16 as ::core::ffi::c_int != 0 {
                (*E).msl_var_03 = (t >> 16 as ::core::ffi::c_int) as uint16;
            }
        }
    }
    if Enemy_MoveDown(
        k,
        (((*E).msl_var_01 as uint32) << 16 as ::core::ffi::c_int | (*E).msl_var_00 as uint32)
            as int32,
    ) != 0
    {
        let mut msl_var_01: int16 = (*E).msl_var_01 as int16;
        if msl_var_01 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int
            && msl_var_01 as ::core::ffi::c_int - 3 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
        {
            (*E).msl_var_03 = 0 as uint16;
            (*E).msl_var_02 = 0 as uint16;
            (*E).msl_var_01 = 0 as uint16;
            (*E).msl_var_00 = 0 as uint16;
            (*E).msl_var_E = 0 as uint16;
            (*E).msl_var_04 = 0 as uint16;
            (*E).msl_var_05 = 1 as uint16;
            if (*E).msl_var_08 as ::core::ffi::c_int == 3 as ::core::ffi::c_int {
                (*E).msl_var_08 = 0 as uint16;
            } else {
                (*E).msl_var_08 = 1 as uint16;
                (*E).msl_parameter_1 = 8 as uint16;
                MaridiaSnail_Func_18(k);
                MaridiaSnail_Func_16(k);
            }
            (*E).base.properties = ((*E).msl_var_C as ::core::ffi::c_int
                | (*E).base.properties as ::core::ffi::c_int & !(3 as ::core::ffi::c_int))
                as uint16;
            StoneZoomer_E67A(k);
            (*E).msl_var_F =
                (0xa3cf5f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).base.current_instruction = (*E).msl_var_D;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).base.timer = 0 as uint16;
        } else {
            (*E).msl_var_00 = -((*E).msl_var_00 as ::core::ffi::c_int) as uint16;
            (*E).msl_var_01 = -((*E).msl_var_01 as ::core::ffi::c_int) as uint16;
            (*E).msl_var_20 = 0 as uint16;
        }
    } else {
        let mut t_0: uint32 = (((*E).msl_var_01 as uint32) << 16 as ::core::ffi::c_int
            | (*E).msl_var_00 as uint32)
            .wrapping_add(0x2000 as uint32);
        (*E).msl_var_00 = t_0 as uint16;
        if (t_0 >> 16 as ::core::ffi::c_int).wrapping_sub(4 as uint32) & 0x8000 as uint32 != 0 {
            (*E).msl_var_01 = (t_0 >> 16 as ::core::ffi::c_int) as uint16;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_16(mut k: uint16) {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    let mut v2: ::core::ffi::c_int = (4 as ::core::ffi::c_int
        * (*E).msl_var_C as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).base.current_instruction = *(g_rom.offset(
        (((0xa3d30d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3d30d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v2 as isize);
    (*E).msl_var_D = *(g_rom.offset(
        (((0xa3d30d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3d30d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v2 + 1 as ::core::ffi::c_int) as isize);
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_17(mut k: uint16) -> uint8 {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    if (*E).msl_var_05 != 0 {
        return 0 as uint8;
    }
    if (*E).msl_var_07 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
        return (MaridiaSnail_Func_18(k) as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint8;
    }
    if (*E).msl_var_07 as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        return (if (*E).base.y_pos as ::core::ffi::c_int
            >= *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        {
            MaridiaSnail_Func_19(k) as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint8;
    } else {
        return (if ((*E).base.y_pos as ::core::ffi::c_int)
            < *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        {
            MaridiaSnail_Func_19(k) as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint8;
    };
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_18(mut k: uint16) -> uint8 {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    if (*E).msl_var_C != 0 {
        return (if (*E).base.x_pos as ::core::ffi::c_int
            >= *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        {
            MaridiaSnail_Func_19(k) as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint8;
    } else {
        return (if ((*E).base.x_pos as ::core::ffi::c_int)
            < *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        {
            MaridiaSnail_Func_19(k) as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint8;
    };
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_19(mut k: uint16) -> uint8 {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    if (*E).msl_var_08 as ::core::ffi::c_int == 2 as ::core::ffi::c_int
        || (*E).msl_var_F as ::core::ffi::c_int == 0xcf5f as ::core::ffi::c_int
    {
        return 0 as uint8;
    }
    let mut v2: uint16 = *(g_rom.offset(
        (((0xa3cdc2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3cdc2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((*E).msl_var_07 as isize);
    (*E).msl_var_07 = v2;
    (*E).base.current_instruction = *((&raw mut (*(g_rom.offset(
        (((0xa3cd42 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3cd42 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut MaridiaSnailData2)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_0 as *mut uint8)
        .offset((8 as ::core::ffi::c_int * v2 as ::core::ffi::c_int) as isize)
        as *mut VoidP) as uint16;
    (*E).base.properties = (*((&raw mut (*(g_rom.offset(
        (((0xa3cd42 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3cd42 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut MaridiaSnailData2)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_2 as *mut uint8)
        .offset((8 as ::core::ffi::c_int * v2 as ::core::ffi::c_int) as isize)
        as *mut uint16) as ::core::ffi::c_int
        | (*E).base.properties as ::core::ffi::c_int & !(3 as ::core::ffi::c_int))
        as uint16;
    (*E).msl_var_D = *((&raw mut (*(g_rom.offset(
        (((0xa3cd42 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3cd42 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut MaridiaSnailData2)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_4 as *mut uint8)
        .offset((8 as ::core::ffi::c_int * v2 as ::core::ffi::c_int) as isize)
        as *mut VoidP) as uint16;
    (*E).msl_var_C = *((&raw mut (*(g_rom.offset(
        (((0xa3cd42 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3cd42 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut MaridiaSnailData2)
        .offset(0 as ::core::ffi::c_int as isize))
    .field_6 as *mut uint8)
        .offset((8 as ::core::ffi::c_int * v2 as ::core::ffi::c_int) as isize)
        as *mut uint16);
    MaridiaSnail_Func_2(
        k,
        (8 as ::core::ffi::c_int * v2 as ::core::ffi::c_int) as uint16,
    );
    MaridiaSnail_Func_3(k);
    (*E).msl_var_05 = 1 as uint16;
    (*E).msl_var_04 = 0 as uint16;
    return 1 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Touch() {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if ((*E).msl_var_08 as ::core::ffi::c_int != 1 as ::core::ffi::c_int
        || (*E).msl_var_F as ::core::ffi::c_int == 0xcf5f as ::core::ffi::c_int
        || MaridiaSnail_Func_20(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0)
        && ((*E).msl_var_F as ::core::ffi::c_int == 0xd1b3 as ::core::ffi::c_int
            || *((&raw mut g_ram as *mut uint8).offset(0xb3c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 0)
    {
        MaridiaSnail_Func_22(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
        if (*E).msl_var_F as ::core::ffi::c_int == 0xd1b3 as ::core::ffi::c_int {
            QueueSfx2_Max3(0x70 as uint16);
        }
    } else if (*E).msl_var_F as ::core::ffi::c_int != 0xcf5f as ::core::ffi::c_int
        && (*E).msl_var_08 as ::core::ffi::c_int != 4 as ::core::ffi::c_int
        && (*E).msl_var_08 as ::core::ffi::c_int != 3 as ::core::ffi::c_int
    {
        Enemy_NormalTouchAI_A3();
        (*E).msl_parameter_1 = (*E).msl_var_06;
        if (*E).msl_var_08 != 0 {
            MaridiaSnail_Func_19(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
        }
        (*E).msl_var_08 = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_20(mut k: uint16) -> uint8 {
    let mut r20: uint16 = ((*Get_MaridiaSnail(k as ::core::ffi::c_int)).msl_var_C
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int) as uint16;
    if (*((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x300 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
    {
        if r20 != 0 {
            return 0 as uint8;
        }
    } else if r20 == 0 {
        return 0 as uint8;
    }
    return 1 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_21(mut k: uint16) -> uint8 {
    if (*Get_MaridiaSnail(k as ::core::ffi::c_int)).msl_var_A as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return (*(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 8 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
    }
    return (*(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Shot() {
    let mut v0: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0xc18 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0x18a6 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int) as uint16;
    if v0 as ::core::ffi::c_int == 768 as ::core::ffi::c_int
        || v0 as ::core::ffi::c_int == 1280 as ::core::ffi::c_int
    {
        Enemy_NormalShotAI_A3();
    } else {
        let mut msl_var_08: uint16 = (*Get_MaridiaSnail(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        ))
        .msl_var_08;
        if msl_var_08 as ::core::ffi::c_int != 3 as ::core::ffi::c_int
            && msl_var_08 as ::core::ffi::c_int != 4 as ::core::ffi::c_int
        {
            MaridiaSnail_Func_23(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
        }
        QueueSfx2_Max3(0x70 as uint16);
    };
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_22(mut k: uint16) {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    (*E).msl_var_08 = 4 as uint16;
    (*E).msl_var_F = (0xa3d1b3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut v2: ::core::ffi::c_int = (4 as ::core::ffi::c_int
        * (*E).msl_var_C as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).base.current_instruction = *(g_rom.offset(
        (((0xa3d50f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3d50f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v2 as isize);
    (*E).msl_var_D = *(g_rom.offset(
        (((0xa3d50f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3d50f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v2 + 1 as ::core::ffi::c_int) as isize);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).msl_var_02 = *((&raw mut g_ram as *mut uint8).offset(0xda4 as ::core::ffi::c_int as isize)
        as *mut uint16);
    let mut v3: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xda2 as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).msl_var_03 = *((&raw mut g_ram as *mut uint8).offset(0xda2 as ::core::ffi::c_int as isize)
        as *mut uint16);
    if v3 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
        v3 = 15 as uint16;
    }
    let mut v4: ::core::ffi::c_int = (4 as ::core::ffi::c_int * v3 as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).msl_var_00 = *(g_rom.offset(
        (((0xa3d517 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3d517 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v4 as isize);
    (*E).msl_var_01 = *(g_rom.offset(
        (((0xa3d517 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3d517 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v4 + 1 as ::core::ffi::c_int) as isize);
    if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 4 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).msl_var_02 = -((*E).msl_var_02 as ::core::ffi::c_int) as uint16;
        (*E).msl_var_03 = -((*E).msl_var_03 as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn MaridiaSnail_Func_23(mut k: uint16) {
    let mut E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int);
    (*E).msl_var_08 = 5 as uint16;
    (*E).msl_var_F = (0xa3d1b3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut v2: ::core::ffi::c_int = (4 as ::core::ffi::c_int
        * (*E).msl_var_C as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).base.current_instruction = *(g_rom.offset(
        (((0xa3d5a4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3d5a4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v2 as isize);
    (*E).msl_var_D = *(g_rom.offset(
        (((0xa3d5a4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3d5a4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v2 + 1 as ::core::ffi::c_int) as isize);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).msl_var_01 = -(1 as ::core::ffi::c_int) as uint16;
    (*E).msl_var_03 = (if *(&raw mut g_ram as *mut uint8)
        .offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        -(1 as ::core::ffi::c_int)
    } else {
        1 as ::core::ffi::c_int
    }) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Reflec_Func_1() {
    if *((&raw mut g_ram as *mut uint8).offset(0x797 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
        && {
            let ref mut fresh15 = *((&raw mut g_ram as *mut uint8)
                .offset(0x1794 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(2 as ::core::ffi::c_int as isize);
            *fresh15 = (*fresh15).wrapping_sub(1);
            *fresh15 == 0
        }
    {
        *((&raw mut g_ram as *mut uint8).offset(0x1794 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize) = 16 as uint16;
        let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1794 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize);
        let mut v1: uint16 = (8 as ::core::ffi::c_int
            * *((&raw mut g_ram as *mut uint8).offset(0x1794 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as uint16;
        let mut n: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
        loop {
            *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(
                    ((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 137 as ::core::ffi::c_int) as isize,
                ) = *(g_rom.offset(
                (((0xa3dabc as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa3dabc as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            n -= 1;
            if !(n != 0) {
                break;
            }
        }
        *((&raw mut g_ram as *mut uint8).offset(0x1794 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) =
            (*(((&raw mut g_ram as *mut uint8).offset(0x1794 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(1 as ::core::ffi::c_int as isize) as *mut uint16
                as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                & 7 as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Reflec_Instr_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Reflec(k as ::core::ffi::c_int)).reflec_parameter_2 =
        *jp.offset(0 as ::core::ffi::c_int as isize);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Reflec_Init() {
    let mut E: *mut Enemy_Reflec = Get_Reflec(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_BlockPlasmaBeam as ::core::ffi::c_int) as uint16;
    (*E).base.current_instruction = *(g_rom.offset(
        (((0xa3dc0b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3dc0b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((*E).reflec_parameter_1 as isize);
    (*((&raw mut g_ram as *mut uint8).offset(0x178c as ::core::ffi::c_int as isize)
        as *mut LongPtr))
        .addr = (0xa3db0c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP;
    *(&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x178c as ::core::ffi::c_int as isize)
        as *mut LongPtr))
        .bank as *mut uint16) = 163 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1794 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = ((16 as ::core::ffi::c_int
        * (*E).base.palette_index as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1794 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) = 16 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Reflec_Shot() {
    let mut v0: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0x18a6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut EK: *mut Enemy_Reflec = Get_Reflec(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*EK).base.invincibility_timer = 10 as uint16;
    let mut v2: uint16 = (32 as ::core::ffi::c_int * (*EK).reflec_parameter_2 as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int
            * (*((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)) as uint16;
    let mut varE32: uint16 = v2;
    let mut v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *(g_rom.offset(
        (((0xa3dcae as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3dcae as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v3 as isize) as ::core::ffi::c_int
        == 0x8000 as ::core::ffi::c_int
    {
        let ref mut fresh16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc04 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh16 = (*fresh16 as ::core::ffi::c_int | 0x10 as ::core::ffi::c_int) as uint16;
        printf(b"Possible bug. What is X?\n\0" as *const u8 as *const ::core::ffi::c_char);
        let mut ET: *mut Enemy_Reflec = Get_Reflec(v2 as ::core::ffi::c_int);
        if (*ET).base.health != 0 {
            Enemy_NormalShotAI_SkipSomeParts_A3();
            if (*ET).base.health == 0 {
                (*ET).base.current_instruction = *(g_rom.offset(
                    (((0xa3dca6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa3dca6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8
                    as *mut uint16)
                    .offset((*ET).reflec_parameter_2 as isize);
                (*ET).base.instruction_timer = 1 as uint16;
                (*ET).base.timer = 0 as uint16;
            }
        }
    } else {
        let mut v4: int16 = *(g_rom.offset(
            (((0xa3dcae as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3dcae as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize) as int16;
        if (v4 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            v4 = -(*(g_rom.offset(
                (((0xa3dcae as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa3dcae as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((varE32 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int) as int16;
        }
        let mut v5: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) = v4 as uint16;
        let ref mut fresh17 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize);
        *fresh17 = (*fresh17 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
        ProjectileReflection(v0);
        QueueSfx2_Max6(0x57 as uint16);
    };
}
#[no_mangle]
pub unsafe extern "C" fn WreckedShipOrangeZoomer_Func_1(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    (*Get_WreckedShipOrangeZoomer(k as ::core::ffi::c_int)).wsozr_var_F =
        *jp.offset(0 as ::core::ffi::c_int as isize);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn WreckedShipOrangeZoomer_Init() {
    let mut E: *mut Enemy_WreckedShipOrangeZoomer = Get_WreckedShipOrangeZoomer(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.current_instruction = *(g_rom.offset(
        (((0xa3e03b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3e03b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((*E).base.current_instruction as ::core::ffi::c_int & 3 as ::core::ffi::c_int)
                as isize,
        );
    (*E).wsozr_var_F = addr_locret_A3E08A as ::core::ffi::c_int as uint16;
    let mut v1: uint16 = *(g_rom.offset(
        (((0xa3e5f0 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3e5f0 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((*E).wsozr_parameter_1 as isize);
    (*E).wsozr_var_A = v1;
    (*E).wsozr_var_B = v1;
    if (*E).base.properties as ::core::ffi::c_int & 3 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if (*E).base.properties as ::core::ffi::c_int & 3 as ::core::ffi::c_int
            == 2 as ::core::ffi::c_int
        {
            (*E).wsozr_var_B = -((*E).wsozr_var_B as ::core::ffi::c_int) as uint16;
        }
    } else {
        (*E).wsozr_var_A = -((*E).wsozr_var_A as ::core::ffi::c_int) as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn WreckedShipOrangeZoomer_Main() {
    let mut E: *mut Enemy_WreckedShipOrangeZoomer = Get_WreckedShipOrangeZoomer(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallEnemyPreInstr(
        ((*E).wsozr_var_F as ::core::ffi::c_int | 0xa30000 as ::core::ffi::c_int) as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn WreckedShipOrangeZoomer_Func_2(mut k: uint16) {
    let mut v7: int16 = 0;
    let mut v9: uint16 = 0;
    let mut v12: uint16 = 0;
    let mut E: *mut Enemy_WreckedShipOrangeZoomer =
        Get_WreckedShipOrangeZoomer(k as ::core::ffi::c_int);
    if *((&raw mut g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 30 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 20 as ::core::ffi::c_int
    {
        (*E).wsozr_var_03 = (*E).wsozr_var_F;
        (*E).wsozr_var_F =
            (0xa3e785 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    if Enemy_MoveRight_ProcessSlopes(
        k,
        Shift8AddMagn((*E).wsozr_var_A as int16, 1 as ::core::ffi::c_int) as int32,
    ) != 0
    {
        (*E).wsozr_var_04 = 0 as uint16;
        EnemyFunc_C8AD(k);
        if Enemy_MoveDown(
            k,
            ((*E).wsozr_var_B as int16 as int32) << 8 as ::core::ffi::c_int,
        ) == 0
        {
            let mut current_block_8: u64;
            if (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - (*E).base.y_pos as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                v7 = (*E).wsozr_var_B as int16;
                if (v7 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                    current_block_8 = 9790114338316996362;
                } else {
                    current_block_8 = 11050875288958768710;
                }
            } else {
                v7 = (*E).wsozr_var_B as int16;
                if v7 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                    current_block_8 = 9790114338316996362;
                } else {
                    current_block_8 = 11050875288958768710;
                }
            }
            match current_block_8 {
                9790114338316996362 => {
                    v7 = -(v7 as ::core::ffi::c_int) as int16;
                }
                _ => {}
            }
            (*E).wsozr_var_B = v7 as uint16;
            return;
        }
        (*E).wsozr_var_A = -((*E).wsozr_var_A as ::core::ffi::c_int) as uint16;
        v9 = addr_kWreckedShipOrangeZoomer_Ilist_E01F as ::core::ffi::c_int as uint16;
        if (*E).wsozr_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v9 = addr_kWreckedShipOrangeZoomer_Ilist_E003 as ::core::ffi::c_int as uint16;
        }
        (*E).base.current_instruction = v9;
        (*E).base.instruction_timer = 1 as uint16;
    } else {
        let mut v11: uint16 =
            ((*E).wsozr_var_04 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E).wsozr_var_04 = v11;
        if v11 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).wsozr_var_B = -((*E).wsozr_var_B as ::core::ffi::c_int) as uint16;
            v12 = addr_kWreckedShipOrangeZoomer_Ilist_E01F as ::core::ffi::c_int as uint16;
            if (*E).wsozr_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                v12 = addr_kWreckedShipOrangeZoomer_Ilist_E003 as ::core::ffi::c_int as uint16;
            }
            (*E).base.current_instruction = v12;
            (*E).base.instruction_timer = 1 as uint16;
        } else {
            (*E).wsozr_var_03 = (*E).wsozr_var_F;
            (*E).wsozr_var_F =
                (0xa3e785 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn sub_A3E168(mut k: uint16) {
    let mut v7: int16 = 0;
    let mut v9: uint16 = 0;
    let mut v12: uint16 = 0;
    let mut E: *mut Enemy_WreckedShipOrangeZoomer =
        Get_WreckedShipOrangeZoomer(k as ::core::ffi::c_int);
    if *((&raw mut g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 30 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 20 as ::core::ffi::c_int
    {
        (*E).wsozr_var_03 = (*E).wsozr_var_F;
        (*E).wsozr_var_F =
            (0xa3e785 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    if Enemy_MoveDown(
        k,
        Shift8AddMagn((*E).wsozr_var_B as int16, 1 as ::core::ffi::c_int) as int32,
    ) != 0
    {
        (*E).wsozr_var_04 = 0 as uint16;
        if Enemy_MoveRight_ProcessSlopes(
            k,
            ((*E).wsozr_var_A as int16 as int32) << 8 as ::core::ffi::c_int,
        ) == 0
        {
            EnemyFunc_C8AD(k);
            if (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - (*E).base.x_pos as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                v7 = (*E).wsozr_var_A as int16;
                if (v7 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                    v7 = -(v7 as ::core::ffi::c_int) as int16;
                }
            } else {
                v7 = (*E).wsozr_var_A as int16;
                if v7 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                    v7 = -(v7 as ::core::ffi::c_int) as int16;
                }
            }
            (*E).wsozr_var_A = v7 as uint16;
            return;
        }
        (*E).wsozr_var_B = -((*E).wsozr_var_B as ::core::ffi::c_int) as uint16;
        v9 = addr_kWreckedShipOrangeZoomer_Ilist_DFE7 as ::core::ffi::c_int as uint16;
        if (*E).wsozr_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v9 = addr_kWreckedShipOrangeZoomer_Ilist_DFCB as ::core::ffi::c_int as uint16;
        }
        (*E).base.current_instruction = v9;
        (*E).base.instruction_timer = 1 as uint16;
    } else {
        let mut v11: uint16 =
            ((*E).wsozr_var_04 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E).wsozr_var_04 = v11;
        if v11 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).wsozr_var_A = -((*E).wsozr_var_A as ::core::ffi::c_int) as uint16;
            v12 = addr_kWreckedShipOrangeZoomer_Ilist_DFE7 as ::core::ffi::c_int as uint16;
            if (*E).wsozr_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                v12 = addr_kWreckedShipOrangeZoomer_Ilist_DFCB as ::core::ffi::c_int as uint16;
            }
            (*E).base.current_instruction = v12;
            (*E).base.instruction_timer = 1 as uint16;
        } else {
            (*E).wsozr_var_03 = (*E).wsozr_var_F;
            (*E).wsozr_var_F =
                (0xa3e785 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn BigEyeBugs_Init() {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v1).current_instruction = *(g_rom.offset(
        (((0xa3e2cc as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3e2cc as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((*v1).current_instruction as ::core::ffi::c_int & 3 as ::core::ffi::c_int) as isize,
        );
    StoneZoomer_E67A(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn FireZoomer_Init() {
    let mut E: *mut Enemy_FireZoomer = Get_FireZoomer(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.current_instruction = *(g_rom.offset(
        (((0xa3e2cc as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3e2cc as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((*E).base.current_instruction as ::core::ffi::c_int & 3 as ::core::ffi::c_int)
                as isize,
        );
    StoneZoomer_E67A(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Zoomer_Instr_SetPreinstr(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    (*gEnemyData(k as ::core::ffi::c_int)).ai_preinstr =
        *jp.offset(0 as ::core::ffi::c_int as isize);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn StoneZoomer_Init() {
    let mut E: *mut Enemy_StoneZoomer = Get_StoneZoomer(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.current_instruction = *(g_rom.offset(
        (((0xa3e2cc as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa3e2cc as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((*E).base.current_instruction as ::core::ffi::c_int & 3 as ::core::ffi::c_int)
                as isize,
        );
    StoneZoomer_E67A(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn StoneZoomer_E67A(mut k: uint16) {
    let mut E: *mut Enemy_StoneZoomer = Get_StoneZoomer(k as ::core::ffi::c_int);
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A3 as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).szr_var_F = (0xa3e6c1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if (*E).szr_parameter_1 as ::core::ffi::c_int != 255 as ::core::ffi::c_int {
        let mut v2: uint16 = *(g_rom.offset(
            (((0xa3e5f0 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3e5f0 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((*E).szr_parameter_1 as isize);
        (*E).szr_var_A = v2;
        (*E).szr_var_B = v2;
    }
    if (*E).base.properties as ::core::ffi::c_int & 3 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if (*E).base.properties as ::core::ffi::c_int & 3 as ::core::ffi::c_int
            == 2 as ::core::ffi::c_int
        {
            (*E).szr_var_B = -((*E).szr_var_B as ::core::ffi::c_int) as uint16;
        }
    } else {
        (*E).szr_var_A = -((*E).szr_var_A as ::core::ffi::c_int) as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn StoneZoomer_Main() {
    let mut E: *mut Enemy_StoneZoomer = Get_StoneZoomer(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallEnemyPreInstr(
        ((*E).szr_var_F as ::core::ffi::c_int | 0xa30000 as ::core::ffi::c_int) as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn FireZoomer_Func_1(mut k: uint16) {
    let mut E: *mut Enemy_FireZoomer = Get_FireZoomer(k as ::core::ffi::c_int);
    if *((&raw mut g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 30 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 20 as ::core::ffi::c_int
    {
        (*E).fzr_var_03 = (*E).fzr_var_F;
        (*E).fzr_var_F = (0xa3e785 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        Shift8AddMagn((*E).fzr_var_A as int16, 1 as ::core::ffi::c_int) as int32,
    ) != 0
    {
        (*E).fzr_var_04 = 0 as uint16;
        EnemyFunc_C8AD(k);
        if Enemy_MoveDown(
            k,
            ((*E).fzr_var_B as int16 as int32) << 8 as ::core::ffi::c_int,
        ) != 0
        {
            (*E).fzr_var_A = -((*E).fzr_var_A as ::core::ffi::c_int) as uint16;
            let mut fzr_parameter_2: uint16 = (*E).fzr_parameter_2;
            let mut v7: uint16 = 0;
            if (*E).fzr_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                v7 = *(g_rom.offset(
                    (((0xa3e63c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa3e63c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(
                        (fzr_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                    );
            } else {
                v7 = *(g_rom.offset(
                    (((0xa3e630 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa3e630 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(
                        (fzr_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                    );
            }
            (*E).base.current_instruction = v7;
            (*E).base.instruction_timer = 1 as uint16;
        }
    } else {
        let mut v8: uint16 =
            ((*E).fzr_var_04 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E).fzr_var_04 = v8;
        if v8 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).fzr_var_B = -((*E).fzr_var_B as ::core::ffi::c_int) as uint16;
            let mut v9: uint16 = (*E).fzr_parameter_2;
            let mut v10: uint16 = 0;
            if (*E).fzr_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                v10 = *(g_rom.offset(
                    (((0xa3e63c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa3e63c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            } else {
                v10 = *(g_rom.offset(
                    (((0xa3e630 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa3e630 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            }
            (*E).base.current_instruction = v10;
            (*E).base.instruction_timer = 1 as uint16;
        } else {
            (*E).fzr_var_03 = (*E).fzr_var_F;
            (*E).fzr_var_F =
                (0xa3e785 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn FireZoomer_Func_2(mut k: uint16) {
    let mut E: *mut Enemy_FireZoomer = Get_FireZoomer(k as ::core::ffi::c_int);
    if Enemy_MoveDown(
        k,
        (((*E).fzr_var_02 as uint32) << 16 as ::core::ffi::c_int | (*E).fzr_var_01 as uint32)
            as int32,
    ) != 0
    {
        if (*E).fzr_parameter_1 as ::core::ffi::c_int == 255 as ::core::ffi::c_int {
            (*E).fzr_var_A = 128 as uint16;
            (*E).fzr_var_B = 128 as uint16;
        }
        (*E).fzr_var_01 = 0 as uint16;
        (*E).fzr_var_02 = 0 as uint16;
        (*E).fzr_var_04 = 0 as uint16;
        (*E).fzr_var_F = (*E).fzr_var_03;
    } else {
        if (*E).fzr_var_02 as ::core::ffi::c_int - 4 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            AddToHiLo(
                &raw mut (*E).fzr_var_02,
                &raw mut (*E).fzr_var_01,
                0x8000 as uint32,
            );
        }
        if (*E).fzr_var_01 == 0 && (*E).fzr_var_02 == 0 {
            (*E).fzr_var_F =
                (0xa3e6c8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn FireZoomer_Func_3(mut k: uint16) {
    let mut E: *mut Enemy_FireZoomer = Get_FireZoomer(k as ::core::ffi::c_int);
    if *((&raw mut g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 30 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 20 as ::core::ffi::c_int
    {
        (*E).fzr_var_03 = (*E).fzr_var_F;
        (*E).fzr_var_F = (0xa3e785 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    if Enemy_MoveDown(
        k,
        Shift8AddMagn((*E).fzr_var_B as int16, 1 as ::core::ffi::c_int) as int32,
    ) != 0
    {
        (*E).fzr_var_04 = 0 as uint16;
        let mut amt: int32 = FireZoomer_E8A5(k);
        if Enemy_MoveRight_IgnoreSlopes(k, amt) != 0 {
            (*E).fzr_var_B = -((*E).fzr_var_B as ::core::ffi::c_int) as uint16;
            let mut fzr_parameter_2: uint16 = (*E).fzr_parameter_2;
            let mut v6: uint16 = 0;
            if (*E).fzr_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                v6 = *(g_rom.offset(
                    (((0xa3e654 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa3e654 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(
                        (fzr_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                    );
            } else {
                v6 = *(g_rom.offset(
                    (((0xa3e648 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa3e648 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(
                        (fzr_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                    );
            }
            (*E).base.current_instruction = v6;
            (*E).base.instruction_timer = 1 as uint16;
        } else {
            EnemyFunc_C8AD(k);
        }
    } else {
        let mut v7: uint16 =
            ((*E).fzr_var_04 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E).fzr_var_04 = v7;
        if v7 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).fzr_var_A = -((*E).fzr_var_A as ::core::ffi::c_int) as uint16;
            let mut v8: uint16 = (*E).fzr_parameter_2;
            let mut v9: uint16 = 0;
            if (*E).fzr_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                v9 = *(g_rom.offset(
                    (((0xa3e654 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa3e654 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            } else {
                v9 = *(g_rom.offset(
                    (((0xa3e648 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa3e648 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            }
            (*E).base.current_instruction = v9;
            (*E).base.instruction_timer = 1 as uint16;
        } else {
            (*E).fzr_var_03 = (*E).fzr_var_F;
            (*E).fzr_var_F =
                (0xa3e785 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn FireZoomer_E8A5(mut k: uint16) -> int32 {
    let mut E: *mut Enemy_FireZoomer = Get_FireZoomer(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut xpos: uint16 = (*E).base.x_pos;
    let mut ypos: uint16 =
        (if (*E).fzr_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).base.y_pos as ::core::ffi::c_int - (*E).base.y_height as ::core::ffi::c_int
        } else {
            (*E).base.y_pos as ::core::ffi::c_int + (*E).base.y_height as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int
        }) as uint16;
    CalculateBlockContainingPixelPos(xpos, ypos);
    if *((&raw mut g_ram as *mut uint8).offset(0x10002 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as ::core::ffi::c_int
        & 0xf000 as ::core::ffi::c_int
        == 4096 as ::core::ffi::c_int
        && *(&raw mut g_ram as *mut uint8)
            .offset(0x16402 as ::core::ffi::c_int as isize)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int
            >= 5 as ::core::ffi::c_int
    {
        let mut v1: uint16 = *(g_rom.offset(
            (((0xa3e931 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3e931 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                ((4 as ::core::ffi::c_int
                    * (*(&raw mut g_ram as *mut uint8)
                        .offset(0x16402 as ::core::ffi::c_int as isize)
                        .offset(
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0xdc4 as ::core::ffi::c_int as isize)
                                as *mut uint16) as isize,
                        ) as ::core::ffi::c_int
                        & 0x1f as ::core::ffi::c_int)) as uint16
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
        if (*E).fzr_var_A as int16 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            return Multiply16x16((*E).fzr_var_A, v1) as int32;
        } else {
            return -(Multiply16x16(-((*E).fzr_var_A as ::core::ffi::c_int) as uint16, v1)
                as int32);
        }
    } else {
        return ((*E).fzr_var_A as int16 as int32) << 8 as ::core::ffi::c_int;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Metroid_Init() {
    let mut E: *mut Enemy_Metroid = Get_Metroid(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.current_instruction = addr_kMetroid_Ilist_E9CF as ::core::ffi::c_int as uint16;
    (*E).metroid_var_00 = CreateSpriteAtPos(
        (*E).base.x_pos,
        (*E).base.y_pos,
        50 as uint16,
        ((*E).base.vram_tiles_index as ::core::ffi::c_int
            | (*E).base.palette_index as ::core::ffi::c_int) as uint16,
    );
    (*E).metroid_var_01 = CreateSpriteAtPos(
        (*E).base.x_pos,
        (*E).base.y_pos,
        52 as uint16,
        ((*E).base.vram_tiles_index as ::core::ffi::c_int
            | (*E).base.palette_index as ::core::ffi::c_int) as uint16,
    );
    (*E).metroid_var_02 = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Metroid_Instr_2(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max6(0x50 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Metroid_Instr_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut v2: ::core::ffi::c_int = NextRandom() as ::core::ffi::c_int & 7 as ::core::ffi::c_int;
    QueueSfx2_Max6(
        *(g_rom.offset(
            (((0xa3ead6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3ead6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize),
    );
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Metroid_Frozen() {
    Enemy_NormalFrozenAI_A3();
    let mut E: *mut Enemy_Metroid = Get_Metroid(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).metroid_var_E != 0 {
        (*E).metroid_var_E = (*E).metroid_var_E.wrapping_sub(1);
        (*E).base.flash_timer = 2 as uint16;
    }
    let mut v1: ::core::ffi::c_int =
        (*E).metroid_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xf078 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 3072 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xf2f8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xef78 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = addr_kSpriteObject_Ilist_C3BA as ::core::ffi::c_int as uint16;
    let mut v2: ::core::ffi::c_int =
        (*E).metroid_var_01 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xf078 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 3072 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xf2f8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xef78 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = addr_kSpriteObject_Ilist_C4B6 as ::core::ffi::c_int as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Metroid_Hurt() {
    let mut E: *mut Enemy_Metroid = Get_Metroid(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).base.flash_timer as ::core::ffi::c_int & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut r18: uint16 = (*E).base.palette_index;
        let mut v1: ::core::ffi::c_int =
            (*E).metroid_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xf078 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (r18 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0xf078 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0xf1ff as ::core::ffi::c_int) as uint16;
        let mut v2: ::core::ffi::c_int =
            (*E).metroid_var_01 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xf078 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = (r18 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0xf078 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int
                & 0xf1ff as ::core::ffi::c_int) as uint16;
    } else {
        let mut v3: ::core::ffi::c_int =
            (*E).metroid_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh10 = *((&raw mut g_ram as *mut uint8)
            .offset(0xf078 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize);
        *fresh10 = (*fresh10 as ::core::ffi::c_int & 0xf1ff as ::core::ffi::c_int) as uint16;
        let mut metroid_var_01: uint16 = (*E).metroid_var_01;
        let ref mut fresh11 = *((&raw mut g_ram as *mut uint8)
            .offset(0xf078 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((metroid_var_01 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh11 = (*fresh11 as ::core::ffi::c_int & 0xf1ff as ::core::ffi::c_int) as uint16;
    };
}
static mut off_A3EC09: [Option<Func_Y_V>; 4] = unsafe {
    [
        Some(Metroid_Func_1 as unsafe extern "C" fn(uint16) -> ()),
        Some(Metroid_Func_2 as unsafe extern "C" fn(uint16) -> ()),
        Some(Metroid_Func_3 as unsafe extern "C" fn(uint16) -> ()),
        Some(Metroid_Func_4 as unsafe extern "C" fn(uint16) -> ()),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn Metroid_Main() {
    let mut E: *mut Enemy_Metroid = Get_Metroid(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    off_A3EC09[(*E).metroid_var_F as usize].expect("non-null function pointer")(
        (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 8 as ::core::ffi::c_int) as uint16,
    );
    let mut r24: uint16 = ((*E).base.vram_tiles_index as ::core::ffi::c_int
        | (*E).base.palette_index as ::core::ffi::c_int) as uint16;
    let mut v2: ::core::ffi::c_int =
        (*E).metroid_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xf0f8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*E).base.x_pos;
    *((&raw mut g_ram as *mut uint8).offset(0xf1f8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*E).base.y_pos;
    *((&raw mut g_ram as *mut uint8).offset(0xf078 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = r24;
    *((&raw mut g_ram as *mut uint8).offset(0xf2f8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    let mut v4: ::core::ffi::c_int =
        (*E).metroid_var_01 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xf0f8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = (*E).base.x_pos;
    *((&raw mut g_ram as *mut uint8).offset(0xf1f8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = (*E).base.y_pos;
    *((&raw mut g_ram as *mut uint8).offset(0xf078 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = r24;
    *((&raw mut g_ram as *mut uint8).offset(0xf2f8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Metroid_Func_1(mut varE32: uint16) {
    let mut current_block: u64;
    let mut v3: int16 = 0;
    let mut v4: int16 = 0;
    let mut v8: int16 = 0;
    let mut v9: int16 = 0;
    let mut E: *mut Enemy_Metroid = Get_Metroid(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut t: int32 = ((((*E).base.y_pos as ::core::ffi::c_int - varE32 as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int
        >> 2 as ::core::ffi::c_int) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let mut r18: uint16 = t as uint16;
    let mut r20: uint16 = (t >> 16 as ::core::ffi::c_int) as uint16;
    let mut metroid_var_C: uint16 = (*E).metroid_var_C;
    let mut v2: bool = (metroid_var_C as ::core::ffi::c_int) < r18 as ::core::ffi::c_int;
    (*E).metroid_var_C =
        (metroid_var_C as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
    v3 = ((*E).metroid_var_D as ::core::ffi::c_int
        - (v2 as ::core::ffi::c_int + r20 as ::core::ffi::c_int)) as int16;
    (*E).metroid_var_D = v3 as uint16;
    if (v3 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        if v3 as uint16 as ::core::ffi::c_int >= 0xfffd as ::core::ffi::c_int {
            current_block = 3810685344454692134;
        } else {
            v4 = -(3 as ::core::ffi::c_int) as int16;
            current_block = 2868539653012386629;
        }
    } else if (v3 as uint16 as ::core::ffi::c_int) < 3 as ::core::ffi::c_int {
        current_block = 3810685344454692134;
    } else {
        v4 = 3 as int16;
        current_block = 2868539653012386629;
    }
    match current_block {
        2868539653012386629 => {
            (*E).metroid_var_D = v4 as uint16;
            (*E).metroid_var_C = 0 as uint16;
        }
        _ => {}
    }
    if Enemy_MoveDown(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (((*E).metroid_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).metroid_var_C as uint32)
            as int32,
    ) != 0
    {
        (*E).metroid_var_C = 0 as uint16;
        (*E).metroid_var_D = 0 as uint16;
    }
    t = (((((*E).base.x_pos as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >> 2 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as int32;
    r18 = t as uint16;
    r20 = (t >> 16 as ::core::ffi::c_int) as uint16;
    let mut metroid_var_A: uint16 = (*E).metroid_var_A;
    v2 = (metroid_var_A as ::core::ffi::c_int) < r18 as ::core::ffi::c_int;
    (*E).metroid_var_A =
        (metroid_var_A as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
    v8 = ((*E).metroid_var_B as ::core::ffi::c_int
        - (v2 as ::core::ffi::c_int + r20 as ::core::ffi::c_int)) as int16;
    (*E).metroid_var_B = v8 as uint16;
    if (v8 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        if v8 as uint16 as ::core::ffi::c_int >= 0xfffd as ::core::ffi::c_int {
            current_block = 8759308886461481507;
        } else {
            v9 = -(3 as ::core::ffi::c_int) as int16;
            current_block = 17478428563724192186;
        }
    } else if (v8 as uint16 as ::core::ffi::c_int) < 3 as ::core::ffi::c_int {
        current_block = 8759308886461481507;
    } else {
        v9 = 3 as int16;
        current_block = 17478428563724192186;
    }
    match current_block {
        17478428563724192186 => {
            (*E).metroid_var_B = v9 as uint16;
            (*E).metroid_var_A = 0 as uint16;
        }
        _ => {}
    }
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (((*E).metroid_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).metroid_var_A as uint32)
            as int32,
    ) != 0
    {
        (*E).metroid_var_A = 0 as uint16;
        (*E).metroid_var_B = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Metroid_Func_2(mut varE32: uint16) {
    let mut current_block: u64;
    let mut v1: int16 = 0;
    let mut v5: int16 = 0;
    let mut v2: uint16 = 0;
    let mut v6: uint16 = 0;
    let mut r18: uint16 = 0 as uint16;
    let mut r20: uint16 = 0 as uint16;
    let mut E: *mut Enemy_Metroid = Get_Metroid(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    *(&raw mut v1 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
        ((varE32 as ::core::ffi::c_int - (*E).base.y_pos as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
    *(&raw mut v1 as *mut uint8).offset(
        (::core::mem::size_of::<int16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) = (varE32 as ::core::ffi::c_int
        - *(&raw mut (*E).base.y_pos as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int) as uint8;
    r20 = ((v1 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 3 as ::core::ffi::c_int) as uint16;
    if r20 as ::core::ffi::c_int & 0x1000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        r20 = (r20 as ::core::ffi::c_int | 0xe000 as ::core::ffi::c_int) as uint16;
    }
    if r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        if r20 as ::core::ffi::c_int >= 0xfffd as ::core::ffi::c_int {
            current_block = 12681381661298955615;
        } else {
            v2 = -(3 as ::core::ffi::c_int) as uint16;
            current_block = 2868539653012386629;
        }
    } else if (r20 as ::core::ffi::c_int) < 3 as ::core::ffi::c_int {
        current_block = 12681381661298955615;
    } else {
        v2 = 3 as uint16;
        current_block = 2868539653012386629;
    }
    match current_block {
        2868539653012386629 => {
            r20 = v2;
            r18 = 0 as uint16;
        }
        _ => {}
    }
    (*E).metroid_var_C = r18;
    (*E).metroid_var_D = r20;
    if Enemy_MoveDown(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        ((r20 as uint32) << 16 as ::core::ffi::c_int | r18 as uint32) as int32,
    ) != 0
    {
        (*E).metroid_var_C = 0 as uint16;
        (*E).metroid_var_D = 0 as uint16;
    }
    r18 = 0 as uint16;
    r20 = 0 as uint16;
    *(&raw mut v5 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
        ((*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
    *(&raw mut v5 as *mut uint8).offset(
        (::core::mem::size_of::<int16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) = (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *(&raw mut (*E).base.x_pos as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int) as uint8;
    r20 = ((v5 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 3 as ::core::ffi::c_int) as uint16;
    if r20 as ::core::ffi::c_int & 0x1000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        r20 = (r20 as ::core::ffi::c_int | 0xe000 as ::core::ffi::c_int) as uint16;
    }
    if r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        if r20 as ::core::ffi::c_int >= 0xfffd as ::core::ffi::c_int {
            current_block = 8114601772576486457;
        } else {
            v6 = -(3 as ::core::ffi::c_int) as uint16;
            current_block = 18386322304582297246;
        }
    } else if (r20 as ::core::ffi::c_int) < 3 as ::core::ffi::c_int {
        current_block = 8114601772576486457;
    } else {
        v6 = 3 as uint16;
        current_block = 18386322304582297246;
    }
    match current_block {
        18386322304582297246 => {
            r20 = v6;
            r18 = 0 as uint16;
        }
        _ => {}
    }
    (*E).metroid_var_A = r18;
    (*E).metroid_var_B = r20;
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        ((r20 as uint32) << 16 as ::core::ffi::c_int | r18 as uint32) as int32,
    ) != 0
    {
        (*E).metroid_var_A = 0 as uint16;
        (*E).metroid_var_B = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Metroid_Func_3(mut varE32: uint16) {
    let mut E: *mut Enemy_Metroid = Get_Metroid(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.x_pos = *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16);
    (*E).base.y_pos = varE32;
    (*E).metroid_var_A = 0 as uint16;
    (*E).metroid_var_B = 0 as uint16;
    (*E).metroid_var_C = 0 as uint16;
    (*E).metroid_var_D = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Metroid_Func_4(mut varE32: uint16) {
    let mut E: *mut Enemy_Metroid = Get_Metroid(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v1: ::core::ffi::c_int =
        (*E).metroid_var_E as ::core::ffi::c_int & 3 as ::core::ffi::c_int;
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
        + *(g_rom.offset(
            (((0xa3ea3f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3ea3f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        + *(g_rom.offset(
            (((0xa3ea3f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa3ea3f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v1 + 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int)
        as uint16;
    (*E).metroid_var_A = 0 as uint16;
    (*E).metroid_var_B = 0 as uint16;
    (*E).metroid_var_C = 0 as uint16;
    (*E).metroid_var_D = 0 as uint16;
    let fresh9 = (*E).metroid_var_E;
    (*E).metroid_var_E = (*E).metroid_var_E.wrapping_sub(1);
    if fresh9 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).metroid_var_F = 0 as uint16;
        (*E).base.current_instruction = addr_kMetroid_Ilist_E9CF as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Metroid_Touch() {
    let mut E: *mut Enemy_Metroid = Get_Metroid(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut varE32: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 8 as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        if (*E).metroid_var_F as ::core::ffi::c_int != 2 as ::core::ffi::c_int {
            (*E).metroid_var_A = 0 as uint16;
            (*E).metroid_var_C = 0 as uint16;
            let mut v2: int16 = ((*E).base.x_pos as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int16;
            (*E).metroid_var_B = (if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                -(256 as ::core::ffi::c_int)
            } else {
                0 as ::core::ffi::c_int
            }) as uint16;
            *((&raw mut (*E).metroid_var_A as *mut uint8).offset(1 as ::core::ffi::c_int as isize)
                as *mut uint16) = ((v2 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int) as uint16;
            let mut v4: int16 =
                ((*E).base.y_pos as ::core::ffi::c_int - varE32 as ::core::ffi::c_int) as int16;
            (*E).metroid_var_D = (if (v4 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                -(256 as ::core::ffi::c_int)
            } else {
                0 as ::core::ffi::c_int
            }) as uint16;
            *((&raw mut (*E).metroid_var_C as *mut uint8).offset(1 as ::core::ffi::c_int as isize)
                as *mut uint16) = ((v4 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int) as uint16;
            (*E).metroid_var_F = 0 as uint16;
            (*E).base.current_instruction =
                addr_kMetroid_Ilist_E9CF as ::core::ffi::c_int as uint16;
            (*E).base.instruction_timer = 1 as uint16;
        }
    } else {
        let mut v5: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16);
        if (*E).metroid_var_F as ::core::ffi::c_int != 3 as ::core::ffi::c_int {
            if *((&raw mut g_ram as *mut uint8).offset(0xe44 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 7 as ::core::ffi::c_int
                == 7 as ::core::ffi::c_int
                && *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 30 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
            {
                QueueSfx3_Max6(0x2d as uint16);
            }
            Metroid_Func_5(v5);
        }
        if ((*E).metroid_var_F as ::core::ffi::c_int) < 2 as ::core::ffi::c_int {
            let mut v7: uint16 = 1 as uint16;
            if (abs16(
                ((*E).base.x_pos as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16,
            ) as ::core::ffi::c_int)
                < 8 as ::core::ffi::c_int
                && (abs16(
                    ((*E).base.y_pos as ::core::ffi::c_int - varE32 as ::core::ffi::c_int)
                        as uint16,
                ) as ::core::ffi::c_int)
                    < 8 as ::core::ffi::c_int
            {
                v7 = 2 as uint16;
                CallSomeSamusCode(0x12 as uint16);
            }
            (*E).metroid_var_F = v7;
            if v7 as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
                (*E).base.current_instruction =
                    addr_kMetroid_Ilist_EA25 as ::core::ffi::c_int as uint16;
                (*E).base.instruction_timer = 1 as uint16;
            }
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Metroid_Func_5(mut k: uint16) {
    let mut v1: uint16 = 0;
    let mut varE32: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 8 as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v1 = 12288 as uint16;
    } else if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        v1 = 24576 as uint16;
    } else {
        v1 = -(16384 as ::core::ffi::c_int) as uint16;
    }
    let mut r18: uint16 = v1;
    let mut E: *mut Enemy_Metroid = Get_Metroid(k as ::core::ffi::c_int);
    let mut metroid_var_02: uint16 = (*E).metroid_var_02;
    let mut v4: bool = (metroid_var_02 as ::core::ffi::c_int) < r18 as ::core::ffi::c_int;
    (*E).metroid_var_02 =
        (metroid_var_02 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
    if v4 {
        Samus_DealDamage(1 as uint16);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Metroid_Shot() {
    let mut v0: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0x18a6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_Metroid = Get_Metroid(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).base.frozen_timer != 0 {
        let mut v3: uint16 = (*((&raw mut g_ram as *mut uint8)
            .offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            & 0xf00 as ::core::ffi::c_int) as uint16;
        if v3 as ::core::ffi::c_int == 256 as ::core::ffi::c_int
            || v3 as ::core::ffi::c_int == 512 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xf434 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*E).base.x_pos;
            *((&raw mut g_ram as *mut uint8).offset(0xf436 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*E).base.y_pos;
            Enemy_NormalShotAI_SkipSomeParts_A3();
            if (*E).base.health == 0 {
                (*E).metroid_var_B = 0 as uint16;
                EnemyDeathAnimation(
                    *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                        as *mut uint16),
                    4 as uint16,
                );
                CallSomeSamusCode(0x13 as uint16);
                *((&raw mut g_ram as *mut uint8).offset(0xef78 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(
                        ((*E).metroid_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                            as isize,
                    ) = 0 as uint16;
                let mut metroid_var_01: uint16 = (*E).metroid_var_01;
                *((&raw mut g_ram as *mut uint8).offset(0xef78 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(
                        (metroid_var_01 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                    ) = 0 as uint16;
                Enemy_ItemDrop_Metroid(metroid_var_01);
            }
        }
    } else if (*E).metroid_var_F as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
        if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            & 0xf00 as ::core::ffi::c_int
            == 1280 as ::core::ffi::c_int
        {
            (*E).metroid_var_E = 4 as uint16;
            (*E).metroid_var_F = 3 as uint16;
            (*E).base.current_instruction =
                addr_kMetroid_Ilist_E9CF as ::core::ffi::c_int as uint16;
            (*E).base.instruction_timer = 1 as uint16;
            CallSomeSamusCode(0x13 as uint16);
        }
    } else {
        (*E).metroid_var_A = 0 as uint16;
        (*E).metroid_var_C = 0 as uint16;
        let mut v7: int16 = ((*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as int16;
        (*E).metroid_var_B = (if (v7 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            -(256 as ::core::ffi::c_int)
        } else {
            0 as ::core::ffi::c_int
        }) as uint16;
        *((&raw mut (*E).metroid_var_A as *mut uint8).offset(1 as ::core::ffi::c_int as isize)
            as *mut uint16) = (32 as ::core::ffi::c_int * v7 as ::core::ffi::c_int) as uint16;
        let mut v9: int16 = ((*E).base.y_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as int16;
        (*E).metroid_var_D = (if (v9 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            -(256 as ::core::ffi::c_int)
        } else {
            0 as ::core::ffi::c_int
        }) as uint16;
        *((&raw mut (*E).metroid_var_C as *mut uint8).offset(1 as ::core::ffi::c_int as isize)
            as *mut uint16) = (32 as ::core::ffi::c_int * v9 as ::core::ffi::c_int) as uint16;
        (*E).metroid_var_F = 0 as uint16;
        (*E).base.current_instruction = addr_kMetroid_Ilist_E9CF as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        let mut v10: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x18a6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int;
        if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v10 as isize) as ::core::ffi::c_int
            & 2 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            QueueSfx3_Max6(0xa as uint16);
            let mut r18: uint16 = *((&raw mut g_ram as *mut uint8)
                .offset(0xc2c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v10 as isize);
            (*E).base.flash_timer = 4 as uint16;
            let mut metroid_parameter_2: uint16 = (*E).metroid_parameter_2;
            let mut v13: bool =
                (metroid_parameter_2 as ::core::ffi::c_int) < r18 as ::core::ffi::c_int;
            let mut v12: uint16 =
                (metroid_parameter_2 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
            v13 = !v13;
            if v12 == 0 || !v13 {
                (*E).metroid_parameter_2 = 0 as uint16;
                (*E).base.frozen_timer = 400 as uint16;
                (*E).base.ai_handler_bits = ((*E).base.ai_handler_bits as ::core::ffi::c_int
                    | 4 as ::core::ffi::c_int)
                    as uint16;
                return;
            }
            (*E).metroid_parameter_2 = v12;
        }
        QueueSfx2_Max6(0x5a as uint16);
    };
}
#[no_mangle]
pub unsafe extern "C" fn Metroid_Powerbomb(mut k: uint16) {
    NormalEnemyPowerBombAi();
    if (*Get_Metroid(k as ::core::ffi::c_int)).base.health == 0 {
        CallSomeSamusCode(0x13 as uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xef78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                ((*Get_Metroid(
                    *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int,
                ))
                .metroid_var_00 as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            ) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xef78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                ((*Get_Metroid(
                    *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int,
                ))
                .metroid_var_01 as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            ) = 0 as uint16;
    }
}
