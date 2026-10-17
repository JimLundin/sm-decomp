use crate::game::sm_80::{NextRandom, QueueMusic_Delayed8, QueueSfx2_Max6, QueueSfx3_Max6};
use crate::game::sm_84::SpawnHardcodedPlm;
use crate::game::sm_86::{eproj_spawn_pt, SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::sm_88::{sub_88DF34, SpawnBG3ScrollHdmaObject};
use crate::game::sm_90::{
    CallSomeSamusCode, DisableMinimapAndMarkBossRoomAsExplored, Samus_ReleaseFromDraygon_,
    Samus_SetGrabbedByDraygonPose,
};
use crate::game::sm_91::Samus_DealDamage;
use crate::game::sm_a0::{
    Abs16, CalculateAngleFromXY, ConvertAngleToXy, CosineMult8bit, EnemyFunc_A6B4_UsedBySporeSpawn,
    EnemyFunc_B691, EnemyFunc_B7A1, EnemyFunc_Divide, Enemy_ItemDrop_Draygon,
    Enemy_ItemDrop_SporeSpawn, Enemy_SwitchToFrozenAi, NormalEnemyFrozenAI,
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy, NormalEnemyShotAi,
    NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAi,
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy, SignExtend8, SineMult8bit, SuitDamageDivision,
    SwitchEnemyAiToMainAi,
};
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::rtl::{g_ram, g_rom, RomPtr, Unreachable};
extern "C" {
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, size_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, EnemyBase,
    EnemyData, EnemyProps, LongPtr, Point16U, VoidP, VramWriteEntry,
};
pub const kEnemyProps_RespawnIfKilled: EnemyProps = 16384;
pub const kEnemyProps_DisableSamusColl: EnemyProps = 8192;
pub const kEnemyProps_BlockPlasmaBeam: EnemyProps = 4096;
pub const kEnemyProps_ProcessedOffscreen: EnemyProps = 2048;
pub const kEnemyProps_Tangible: EnemyProps = 1024;
pub const kEnemyProps_Deleted: EnemyProps = 512;
pub const kEnemyProps_Invisible: EnemyProps = 256;
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
pub use crate::types::Consts_A5;
pub const addr_kDraygon_Ilist_E77D: Consts_A5 = 59261;
pub const addr_stru_A5E729: Consts_A5 = 59177;
pub const addr_kDraygon_Ilist_E6D5: Consts_A5 = 59093;
pub const addr_kDraygon_Ilist_E6C7: Consts_A5 = 59079;
pub const addr_kDraygon_Ilist_E6B9: Consts_A5 = 59065;
pub const addr_kDraygon_BigSprmap_C11B: Consts_A5 = 49435;
pub const addr_kDraygon_BigSprmap_C08F: Consts_A5 = 49295;
pub const addr_word_A5A297: Consts_A5 = 41623;
pub const addr_kDraygon_MorePalettes2: Consts_A5 = 41591;
pub const addr_kDraygon_Palette: Consts_A5 = 41463;
pub const addr_kDraygon_Ilist_9F15: Consts_A5 = 40725;
pub const addr_kDraygon_Ilist_9EA1: Consts_A5 = 40609;
pub const addr_kDraygon_Ilist_9E21: Consts_A5 = 40481;
pub const addr_kDraygon_Ilist_9D9E: Consts_A5 = 40350;
pub const addr_kDraygon_Ilist_9D62: Consts_A5 = 40290;
pub const addr_kDraygon_Ilist_9D5C: Consts_A5 = 40284;
pub const addr_kDraygon_Ilist_9D56: Consts_A5 = 40278;
pub const addr_kDraygon_Ilist_9D50: Consts_A5 = 40272;
pub const addr_kDraygon_Ilist_9D3E: Consts_A5 = 40254;
pub const addr_kDraygon_Ilist_9D1C: Consts_A5 = 40220;
pub const addr_kDraygon_Ilist_9CB4: Consts_A5 = 40116;
pub const addr_kDraygon_Ilist_9C90: Consts_A5 = 40080;
pub const addr_kDraygon_Ilist_9C5A: Consts_A5 = 40026;
pub const addr_kDraygon_Ilist_9C38: Consts_A5 = 39992;
pub const addr_kDraygon_Ilist_9C18: Consts_A5 = 39960;
pub const addr_kDraygon_Ilist_9C06: Consts_A5 = 39942;
pub const addr_kDraygon_Ilist_9BDA: Consts_A5 = 39898;
pub const addr_kDraygon_Ilist_9B5A: Consts_A5 = 39770;
pub const addr_kDraygon_Ilist_9AE8: Consts_A5 = 39656;
pub const addr_kDraygon_Ilist_9A68: Consts_A5 = 39528;
pub const addr_kDraygon_Ilist_99FC: Consts_A5 = 39420;
pub const addr_kDraygon_Ilist_99C0: Consts_A5 = 39360;
pub const addr_kDraygon_Ilist_99BA: Consts_A5 = 39354;
pub const addr_kDraygon_Ilist_99B4: Consts_A5 = 39348;
pub const addr_kDraygon_Ilist_99AE: Consts_A5 = 39342;
pub const addr_kDraygon_Ilist_999C: Consts_A5 = 39324;
pub const addr_kDraygon_Ilist_997A: Consts_A5 = 39290;
pub const addr_kDraygon_Ilist_9944: Consts_A5 = 39236;
pub const addr_kDraygon_Ilist_9922: Consts_A5 = 39202;
pub const addr_kDraygon_Ilist_98FE: Consts_A5 = 39166;
pub const addr_kDraygon_Ilist_98ED: Consts_A5 = 39149;
pub const addr_kDraygon_Ilist_9889: Consts_A5 = 39049;
pub const addr_kDraygon_Ilist_9867: Consts_A5 = 39015;
pub const addr_kDraygon_Ilist_9845: Consts_A5 = 38981;
pub const addr_kDraygon_Ilist_9825: Consts_A5 = 38949;
pub const addr_kDraygon_Ilist_9813: Consts_A5 = 38931;
pub const addr_kDraygon_Ilist_97E7: Consts_A5 = 38887;
pub const addr_kDraygon_Ilist_97D1: Consts_A5 = 38865;
pub const addr_kDraygon_Ilist_97BB: Consts_A5 = 38843;
pub const addr_kDraygon_Ilist_97B9: Consts_A5 = 38841;
pub use crate::types::{Point32, SpawnHardcodedPlmArgs};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Draygon {
    pub base: EnemyBase,
    pub draygon_var_A: uint16,
    pub draygon_var_B: uint16,
    pub draygon_var_C: uint16,
    pub draygon_var_D: uint16,
    pub draygon_var_E: uint16,
    pub draygon_var_F: uint16,
    pub draygon_parameter_1: uint16,
    pub draygon_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub draygon_var_00: uint16,
    pub draygon_var_01: uint16,
    pub draygon_var_02: uint16,
    pub draygon_var_03: uint16,
    pub draygon_var_04: uint16,
    pub draygon_var_05: uint16,
    pub draygon_var_06: uint16,
    pub draygon_var_07: uint16,
    pub draygon_var_08: uint16,
    pub draygon_var_09: uint16,
    pub draygon_var_0A: uint16,
    pub draygon_var_0B: uint16,
    pub draygon_var_0C: uint16,
    pub draygon_var_0D: uint16,
    pub draygon_var_0E: uint16,
    pub draygon_var_0F: uint16,
    pub draygon_var_10: uint16,
    pub draygon_var_11: uint16,
    pub draygon_var_12: uint16,
    pub draygon_var_13: uint16,
    pub draygon_var_14: uint16,
    pub draygon_var_15: uint16,
    pub draygon_var_16: uint16,
    pub draygon_var_17: uint16,
    pub draygon_var_18: uint16,
    pub draygon_var_19: uint16,
    pub draygon_var_1A: uint16,
    pub draygon_var_1B: uint16,
    pub draygon_var_1C: uint16,
    pub draygon_var_1D: uint16,
    pub draygon_var_1E: uint16,
    pub draygon_var_1F: uint16,
    pub pad2: [uint8; 1984],
    pub draygon_var_20: uint16,
    pub draygon_var_21: uint16,
    pub draygon_var_22: uint16,
    pub draygon_var_23: uint16,
    pub draygon_var_24: uint16,
    pub draygon_var_25: uint16,
    pub draygon_var_26: uint16,
    pub draygon_var_27: uint16,
    pub draygon_var_28: uint16,
    pub draygon_var_29: uint16,
    pub draygon_var_2A: uint16,
    pub draygon_var_2B: uint16,
    pub draygon_var_2C: uint16,
    pub draygon_var_2D: uint16,
    pub draygon_var_2E: uint16,
    pub draygon_var_2F: uint16,
    pub draygon_var_30: uint16,
    pub draygon_var_31: uint16,
    pub draygon_var_32: uint16,
    pub draygon_var_33: uint16,
    pub draygon_var_34: uint16,
    pub draygon_var_35: uint16,
    pub draygon_var_36: uint16,
    pub draygon_var_37: uint16,
    pub draygon_var_38: uint16,
    pub draygon_var_39: uint16,
    pub draygon_var_3A: uint16,
    pub draygon_var_3B: uint16,
    pub draygon_var_3C: uint16,
    pub draygon_var_3D: uint16,
    pub draygon_var_3E: uint16,
    pub draygon_var_3F: uint16,
    pub pad3: [uint8; 1984],
    pub draygon_var_40: uint16,
    pub draygon_var_41: uint16,
    pub draygon_var_42: uint16,
    pub draygon_var_43: uint16,
    pub draygon_var_44: uint16,
    pub draygon_var_45: uint16,
    pub draygon_var_46: uint16,
    pub draygon_var_47: uint16,
    pub draygon_var_48: uint16,
    pub draygon_var_49: uint16,
    pub draygon_var_4A: uint16,
    pub draygon_var_4B: uint16,
    pub draygon_var_4C: uint16,
    pub draygon_var_4D: uint16,
    pub draygon_var_4E: uint16,
    pub draygon_var_4F: uint16,
    pub draygon_var_50: uint16,
    pub draygon_var_51: uint16,
    pub draygon_var_52: uint16,
    pub draygon_var_53: uint16,
    pub draygon_var_54: uint16,
    pub draygon_var_55: uint16,
    pub draygon_var_56: uint16,
    pub draygon_var_57: uint16,
    pub draygon_var_58: uint16,
    pub draygon_var_59: uint16,
    pub draygon_var_5A: uint16,
    pub draygon_var_5B: uint16,
    pub draygon_var_5C: uint16,
    pub draygon_var_5D: uint16,
    pub draygon_var_5E: uint16,
    pub draygon_var_5F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_SporeSpawn {
    pub base: EnemyBase,
    pub ssn_var_A: uint16,
    pub ssn_var_B: uint16,
    pub ssn_var_C: uint16,
    pub ssn_var_D: uint16,
    pub ssn_var_E: uint16,
    pub ssn_var_F: uint16,
    pub ssn_parameter_1: uint16,
    pub ssn_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub ssn_var_00: uint16,
    pub ssn_var_01: uint16,
    pub ssn_var_02: uint16,
    pub ssn_var_03: uint16,
    pub ssn_var_04: uint16,
    pub ssn_var_05: uint16,
    pub ssn_var_06: uint16,
    pub ssn_var_07: uint16,
    pub ssn_var_08: uint16,
    pub ssn_var_09: uint16,
    pub ssn_var_0A: uint16,
    pub ssn_var_0B: uint16,
    pub ssn_var_0C: uint16,
    pub ssn_var_0D: uint16,
    pub ssn_var_0E: uint16,
    pub ssn_var_0F: uint16,
    pub ssn_var_10: uint16,
    pub ssn_var_11: uint16,
    pub ssn_var_12: uint16,
    pub ssn_var_13: uint16,
    pub ssn_var_14: uint16,
    pub ssn_var_15: uint16,
    pub ssn_var_16: uint16,
    pub ssn_var_17: uint16,
    pub ssn_var_18: uint16,
    pub ssn_var_19: uint16,
    pub ssn_var_1A: uint16,
    pub ssn_var_1B: uint16,
    pub ssn_var_1C: uint16,
    pub ssn_var_1D: uint16,
    pub ssn_var_1E: uint16,
    pub ssn_var_1F: uint16,
    pub pad2: [uint8; 1984],
    pub ssn_var_20: uint16,
    pub ssn_var_21: uint16,
    pub ssn_var_22: uint16,
    pub ssn_var_23: uint16,
    pub ssn_var_24: uint16,
    pub ssn_var_25: uint16,
    pub ssn_var_26: uint16,
    pub ssn_var_27: uint16,
    pub ssn_var_28: uint16,
    pub ssn_var_29: uint16,
    pub ssn_var_2A: uint16,
    pub ssn_var_2B: uint16,
    pub ssn_var_2C: uint16,
    pub ssn_var_2D: uint16,
    pub ssn_var_2E: uint16,
    pub ssn_var_2F: uint16,
    pub ssn_var_30: uint16,
    pub ssn_var_31: uint16,
    pub ssn_var_32: uint16,
    pub ssn_var_33: uint16,
    pub ssn_var_34: uint16,
    pub ssn_var_35: uint16,
    pub ssn_var_36: uint16,
    pub ssn_var_37: uint16,
    pub ssn_var_38: uint16,
    pub ssn_var_39: uint16,
    pub ssn_var_3A: uint16,
    pub ssn_var_3B: uint16,
    pub ssn_var_3C: uint16,
    pub ssn_var_3D: uint16,
    pub ssn_var_3E: uint16,
    pub ssn_var_3F: uint16,
    pub pad3: [uint8; 1984],
    pub ssn_var_40: uint16,
    pub ssn_var_41: uint16,
    pub ssn_var_42: uint16,
    pub ssn_var_43: uint16,
    pub ssn_var_44: uint16,
    pub ssn_var_45: uint16,
    pub ssn_var_46: uint16,
    pub ssn_var_47: uint16,
    pub ssn_var_48: uint16,
    pub ssn_var_49: uint16,
    pub ssn_var_4A: uint16,
    pub ssn_var_4B: uint16,
    pub ssn_var_4C: uint16,
    pub ssn_var_4D: uint16,
    pub ssn_var_4E: uint16,
    pub ssn_var_4F: uint16,
    pub ssn_var_50: uint16,
    pub ssn_var_51: uint16,
    pub ssn_var_52: uint16,
    pub ssn_var_53: uint16,
    pub ssn_var_54: uint16,
    pub ssn_var_55: uint16,
    pub ssn_var_56: uint16,
    pub ssn_var_57: uint16,
    pub ssn_var_58: uint16,
    pub ssn_var_59: uint16,
    pub ssn_var_5A: uint16,
    pub ssn_var_5B: uint16,
    pub ssn_var_5C: uint16,
    pub ssn_var_5D: uint16,
    pub ssn_var_5E: uint16,
    pub ssn_var_5F: uint16,
}
#[inline]
unsafe extern "C" fn RomPtr_A0(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0xa00000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
#[inline]
unsafe extern "C" fn RomPtr_A5(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0xa50000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
#[inline]
unsafe extern "C" fn gVramWriteEntry(mut a: ::core::ffi::c_int) -> *mut VramWriteEntry {
    return (&raw mut g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry;
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
pub const fnSamus_ScrollFinishedHook_SporeSpawnFight: ::core::ffi::c_int =
    0x909589 as ::core::ffi::c_int;
pub const fnGrappleBeam_Func2: ::core::ffi::c_int = 0x9bc8c5 as ::core::ffi::c_int;
pub const fnnullsub_169_A5: ::core::ffi::c_int = 0xa5804b as ::core::ffi::c_int;
pub const fnDraygon_Func_1: ::core::ffi::c_int = 0xa5871b as ::core::ffi::c_int;
pub const fnDraygon_Func_2: ::core::ffi::c_int = 0xa5878b as ::core::ffi::c_int;
pub const fnDraygon_Func_4: ::core::ffi::c_int = 0xa587f4 as ::core::ffi::c_int;
pub const fnDraygon_Func_6: ::core::ffi::c_int = 0xa588b1 as ::core::ffi::c_int;
pub const fnDraygon_Func_8: ::core::ffi::c_int = 0xa58922 as ::core::ffi::c_int;
pub const fnDraygon_Func_9: ::core::ffi::c_int = 0xa58951 as ::core::ffi::c_int;
pub const fnDraygon_Func_10: ::core::ffi::c_int = 0xa589b3 as ::core::ffi::c_int;
pub const fnDraygon_Func_11: ::core::ffi::c_int = 0xa58a00 as ::core::ffi::c_int;
pub const fnDraygon_Func_12: ::core::ffi::c_int = 0xa58a50 as ::core::ffi::c_int;
pub const fnDraygon_Func_13: ::core::ffi::c_int = 0xa58a90 as ::core::ffi::c_int;
pub const fnDraygon_Func_14: ::core::ffi::c_int = 0xa58b0a as ::core::ffi::c_int;
pub const fnDraygon_Func_15: ::core::ffi::c_int = 0xa58b52 as ::core::ffi::c_int;
pub const fnDraygon_Func_16: ::core::ffi::c_int = 0xa58bae as ::core::ffi::c_int;
pub const fnDraygon_Func_17: ::core::ffi::c_int = 0xa58c33 as ::core::ffi::c_int;
pub const fnDraygon_Func_18: ::core::ffi::c_int = 0xa58c8e as ::core::ffi::c_int;
pub const fnDraygon_Func_19: ::core::ffi::c_int = 0xa58cd4 as ::core::ffi::c_int;
pub const fnDraygon_Func_20: ::core::ffi::c_int = 0xa58d30 as ::core::ffi::c_int;
pub const fnDraygon_Func_21: ::core::ffi::c_int = 0xa58db2 as ::core::ffi::c_int;
pub const fnDraygon_Func_22: ::core::ffi::c_int = 0xa58e19 as ::core::ffi::c_int;
pub const fnDraygon_Func_23: ::core::ffi::c_int = 0xa58f10 as ::core::ffi::c_int;
pub const fnDraygon_Func_24: ::core::ffi::c_int = 0xa58f1e as ::core::ffi::c_int;
pub const fnDraygon_Func_25: ::core::ffi::c_int = 0xa58fd6 as ::core::ffi::c_int;
pub const fnDraygon_Func_26: ::core::ffi::c_int = 0xa590d4 as ::core::ffi::c_int;
pub const fnDraygon_Func_27: ::core::ffi::c_int = 0xa59105 as ::core::ffi::c_int;
pub const fnDraygon_Func_28: ::core::ffi::c_int = 0xa59124 as ::core::ffi::c_int;
pub const fnDraygon_Func_29: ::core::ffi::c_int = 0xa59128 as ::core::ffi::c_int;
pub const fnDraygon_Func_30: ::core::ffi::c_int = 0xa59154 as ::core::ffi::c_int;
pub const fnDraygon_Func_31: ::core::ffi::c_int = 0xa59185 as ::core::ffi::c_int;
pub const fnDraygon_Func_32: ::core::ffi::c_int = 0xa59294 as ::core::ffi::c_int;
pub const fnDraygon_Func_33: ::core::ffi::c_int = 0xa592ab as ::core::ffi::c_int;
pub const fnDraygon_Func_36: ::core::ffi::c_int = 0xa59342 as ::core::ffi::c_int;
pub const fnDraygon_Func_48: uint32 = 10863757 as uint32;
pub const fnDraygon_Func_49: uint32 = 10863891 as uint32;
pub const fnnullsub_223: ::core::ffi::c_int = 0xa5eb1a as ::core::ffi::c_int;
pub const fnSporeSpawn_Func_1: uint32 = 10873627 as uint32;
pub const fnSporeSpawn_Func_2: ::core::ffi::c_int = 0xa5eb52 as ::core::ffi::c_int;
pub const fnSporeSpawn_Func_3: uint32 = 10873755 as uint32;
pub const fnSporeSpawn_Func_4: uint32 = 10873838 as uint32;
#[inline]
unsafe extern "C" fn gEnemyData(mut o: ::core::ffi::c_int) -> *mut EnemyData {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut EnemyData;
}
#[inline]
unsafe extern "C" fn Get_Draygon(mut o: ::core::ffi::c_int) -> *mut Enemy_Draygon {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_Draygon;
}
#[inline]
unsafe extern "C" fn Get_SporeSpawn(mut o: ::core::ffi::c_int) -> *mut Enemy_SporeSpawn {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_SporeSpawn;
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_GrappleReact_NoInteract_A5() {
    SwitchEnemyAiToMainAi();
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_GrappleReact_CancelBeam_A5() {
    Enemy_SwitchToFrozenAi();
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_NormalTouchAI_A5() {
    NormalEnemyTouchAi();
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_NormalShotAI_A5() {
    NormalEnemyShotAi();
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_NormalFrozenAI_A5() {
    NormalEnemyFrozenAI();
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Init() {
    let mut j: uint16 = 0;
    let mut i: ::core::ffi::c_int = 48 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 144 as ::core::ffi::c_int) as isize) =
            *(g_rom.offset(
                (((0xa5a217 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa5a217 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize);
        i -= 2 as ::core::ffi::c_int;
    }
    j = 4094 as uint16;
    while j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
        j = (j as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    }
    (*Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .palette_index = 0xe00 as uint16;
    *(&raw mut (*((&raw mut g_ram as *mut uint8).offset(0x178c as ::core::ffi::c_int as isize)
        as *mut LongPtr))
        .bank as *mut uint16) = 165 as uint16;
    (*((&raw mut g_ram as *mut uint8).offset(0x178c as ::core::ffi::c_int as isize)
        as *mut LongPtr))
        .addr = (0xa59342 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP;
    *((&raw mut g_ram as *mut uint8).offset(0x183c as ::core::ffi::c_int as isize)
        as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
    (*Get_Draygon(0 as ::core::ffi::c_int))
        .base
        .current_instruction = addr_kDraygon_Ilist_9889 as ::core::ffi::c_int as uint16;
    (*Get_Draygon(0x40 as ::core::ffi::c_int))
        .base
        .current_instruction = addr_kDraygon_Ilist_9944 as ::core::ffi::c_int as uint16;
    (*Get_Draygon(0x80 as ::core::ffi::c_int))
        .base
        .current_instruction = addr_kDraygon_Ilist_99FC as ::core::ffi::c_int as uint16;
    (*Get_Draygon(0xc0 as ::core::ffi::c_int))
        .base
        .current_instruction = addr_kDraygon_Ilist_9813 as ::core::ffi::c_int as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x179a as ::core::ffi::c_int as isize)
        as *mut uint16) = 1024 as uint16;
    (*Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .draygon_var_A = (0xa5871b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    sub_88DF34();
    *((&raw mut g_ram as *mut uint8).offset(0xa9 as ::core::ffi::c_int as isize) as *mut uint16) =
        12 as uint16;
    SpawnBG3ScrollHdmaObject();
    (*Get_Draygon(0 as ::core::ffi::c_int)).draygon_var_45 = 1 as uint16;
    DisableMinimapAndMarkBossRoomAsExplored();
}
#[no_mangle]
pub unsafe extern "C" fn CallDraygonFunc(mut ea: uint32, mut k: uint16) {
    match ea {
        10846283 => return,
        10848027 => {
            Draygon_Func_1();
            return;
        }
        10848139 => {
            Draygon_Func_2();
            return;
        }
        10848244 => {
            Draygon_Func_4();
            return;
        }
        10848433 => {
            Draygon_Func_6();
            return;
        }
        10848546 => {
            Draygon_Func_8();
            return;
        }
        10848593 => {
            Draygon_Func_9();
            return;
        }
        10848691 => {
            Draygon_Func_10();
            return;
        }
        10848768 => {
            Draygon_Func_11();
            return;
        }
        10848848 => {
            Draygon_Func_12();
            return;
        }
        10848912 => {
            Draygon_Func_13();
            return;
        }
        10849034 => {
            Draygon_Func_14();
            return;
        }
        10849106 => {
            Draygon_Func_15();
            return;
        }
        10849198 => {
            Draygon_Func_16();
            return;
        }
        10849331 => {
            Draygon_Func_17(k);
            return;
        }
        10849422 => {
            Draygon_Func_18();
            return;
        }
        10849492 => {
            Draygon_Func_19();
            return;
        }
        10849584 => {
            Draygon_Func_20();
            return;
        }
        10849714 => {
            Draygon_Func_21(k);
            return;
        }
        10849817 => {
            Draygon_Func_22();
            return;
        }
        10850064 => {
            Draygon_Func_23();
            return;
        }
        10850078 => {
            Draygon_Func_24(k);
            return;
        }
        10850262 => {
            Draygon_Func_25(k);
            return;
        }
        10850516 => {
            Draygon_Func_26(k);
            return;
        }
        10850565 => {
            Draygon_Func_27(k);
            return;
        }
        10850596 => {
            Draygon_Func_28(k);
            return;
        }
        10850600 => {
            Draygon_Func_29();
            return;
        }
        10850644 => {
            Draygon_Func_30();
            return;
        }
        10850693 => {
            Draygon_Func_31();
            return;
        }
        10850964 => {
            Draygon_Func_32();
            return;
        }
        10850987 => {
            Draygon_Func_33();
            return;
        }
        10863757 => {
            Draygon_Func_48(k);
            return;
        }
        10863891 => {
            Draygon_Func_49(k);
            return;
        }
        10873626 => return,
        10873627 => {
            SporeSpawn_Func_1();
            return;
        }
        10873682 => {
            SporeSpawn_Func_2(k);
            return;
        }
        10873755 => {
            SporeSpawn_Func_3();
            return;
        }
        10873838 => {
            SporeSpawn_Func_4();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Main() {
    let mut EK: *mut Enemy_Draygon = Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallDraygonFunc(
        ((*EK).draygon_var_A as ::core::ffi::c_int | 0xa50000 as ::core::ffi::c_int) as uint32,
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    let mut E0: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut x_pos: uint16 = (*E0).base.x_pos;
    let mut E1: *mut Enemy_Draygon = Get_Draygon(0x40 as ::core::ffi::c_int);
    (*E1).base.x_pos = x_pos;
    let mut E2: *mut Enemy_Draygon = Get_Draygon(0x80 as ::core::ffi::c_int);
    (*E2).base.x_pos = x_pos;
    let mut E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int);
    (*E3).base.x_pos = x_pos;
    let mut y_pos: uint16 = (*E0).base.y_pos;
    (*E1).base.y_pos = y_pos;
    (*E2).base.y_pos = y_pos;
    (*E3).base.y_pos = y_pos;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_1() {
    let mut v2: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    Draygon_Func_3();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    if (*E).draygon_var_B as ::core::ffi::c_int - 256 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if (*E).draygon_var_B == 0 {
            v2 = gVramWriteEntry(
                *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int,
            );
            (*v2).size = 1536 as uint16;
            (*v2).src.addr = -(27648 as ::core::ffi::c_int) as VoidP;
            *(&raw mut (*v2).src.bank as *mut uint16) = 177 as uint16;
            (*v2).vram_dst = 27904 as uint16;
            let ref mut fresh0 = *((&raw mut g_ram as *mut uint8)
                .offset(0x330 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh0 = (*fresh0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
            Draygon_Func_45();
        }
        (*E).draygon_var_B = (*E).draygon_var_B.wrapping_add(1);
    } else {
        (*E).draygon_var_A =
            (0xa5878b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).draygon_var_B = 0 as uint16;
        let mut x_pos: uint16 = (*E).base.x_pos;
        (*E).draygon_var_00 = x_pos;
        (*E).draygon_var_02 = (x_pos as ::core::ffi::c_int + 672 as ::core::ffi::c_int) as uint16;
        (*E).draygon_var_01 = (*E).base.y_pos;
        (*E).base.x_pos = (*E).draygon_var_00;
        (*E).base.y_pos = (*E).draygon_var_01;
        (*E).draygon_var_0F = 24 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_2() {
    Draygon_Func_3();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    if (*E).draygon_var_B as ::core::ffi::c_int - 1232 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        Draygon_Func_46();
        (*E).draygon_var_B = (*E).draygon_var_B.wrapping_add(1);
    } else {
        (*E).draygon_var_A =
            (0xa587f4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).draygon_var_B = 0 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_3() {
    let mut Random: int8 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x3f as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        Random = NextRandom() as int8;
        let mut result: uint16 = (2 as ::core::ffi::c_int
            * ((Random as ::core::ffi::c_int & 3 as ::core::ffi::c_int) + 2 as ::core::ffi::c_int))
            as uint16;
        if (*Get_Draygon(result as ::core::ffi::c_int)).draygon_var_40 == 0 {
            let mut v2: ::core::ffi::c_int = (4 as ::core::ffi::c_int
                * ((Random as ::core::ffi::c_int & 3 as ::core::ffi::c_int)
                    + 2 as ::core::ffi::c_int))
                as uint16 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            eproj_spawn_pt = Point16U {
                x: *(g_rom.offset(
                    (((0xa587dc as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa587dc as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v2 as isize),
                y: *(g_rom.offset(
                    (((0xa587dc as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa587dc as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v2 + 1 as ::core::ffi::c_int) as isize),
            };
            SpawnEprojWithGfx(
                3 as uint16,
                result,
                addr_stru_868E5E as ::core::ffi::c_int as uint16,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_4() {
    Draygon_Func_3();
    Draygon_Func_5();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    (*E).draygon_var_A = (0xa588b1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.current_instruction = addr_kDraygon_Ilist_97D1 as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).draygon_var_20 = 1 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_5() {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut varE24: uint16 = 384 as uint16;
    let mut varE22: uint16 = 0 as uint16;
    let mut varE26: uint16 = 0 as uint16;
    loop {
        varE22 =
            (varE22 as ::core::ffi::c_int + (*E).draygon_var_0F as ::core::ffi::c_int) as uint16;
        let mut v1: uint16 = (varE24 as ::core::ffi::c_int
            - *(&raw mut varE22 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int) as uint16;
        if ((v1 as ::core::ffi::c_int - (*E).draygon_var_01 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            break;
        }
        varE24 = (varE24 as ::core::ffi::c_int
            - *(&raw mut varE22 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int) as uint16;
        let fresh4 = varE26;
        varE26 = varE26.wrapping_add(1);
        *(((&raw mut g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(fresh4 as ::core::ffi::c_int as isize) as *mut uint16) = v1;
        varE26 = (varE26 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        varE26 = varE26.wrapping_add(1);
        if varE26 as ::core::ffi::c_int - 2048 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0
        {
            Unreachable();
        }
    }
    let mut t: uint32 = EnemyFunc_Divide(
        (varE26 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint32,
        ((Abs16(
            ((*E).draygon_var_00 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int) as uint32,
    );
    SetHiLo(&raw mut (*E).draygon_var_D, &raw mut (*E).draygon_var_E, t);
    let mut v6: uint16 = varE26;
    *(((&raw mut g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(varE26 as ::core::ffi::c_int as isize) as *mut uint16) = (*E).base.y_pos;
    (*E).draygon_var_B = v6;
    (*E).draygon_var_C = v6;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_6() {
    Draygon_Func_3();
    Draygon_Func_35();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut draygon_var_B: uint16 = (*E).draygon_var_B;
    if draygon_var_B as ::core::ffi::c_int == 104 as ::core::ffi::c_int {
        let mut E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int);
        (*E3).base.current_instruction = addr_kDraygon_Ilist_9C06 as ::core::ffi::c_int as uint16;
        (*E3).base.instruction_timer = 1 as uint16;
    }
    (*Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .y_pos = *(((&raw mut g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(draygon_var_B as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).draygon_var_B =
        ((*E).draygon_var_B as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    if (*E).draygon_var_B != 0 {
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            ((*E).draygon_var_D as uint32) << 16 as ::core::ffi::c_int
                | (*E).draygon_var_E as uint32,
        );
    } else {
        (*E).draygon_var_A =
            (0xa58922 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_7() {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    if *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).base.current_instruction = addr_kDraygon_Ilist_98FE as ::core::ffi::c_int as uint16;
        if (*E).draygon_var_20 != 0 {
            (*E).base.current_instruction =
                addr_kDraygon_Ilist_9C90 as ::core::ffi::c_int as uint16;
        }
        (*E).base.instruction_timer = 1 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_8() {
    Draygon_Func_3();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut t: uint32 = EnemyFunc_Divide(
        ((*E).draygon_var_C as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint32,
        ((672 as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int) as uint32,
    );
    SetHiLo(&raw mut (*E).draygon_var_D, &raw mut (*E).draygon_var_E, t);
    (*E).draygon_var_A = (0xa58951 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_9() {
    Draygon_Func_3();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut draygon_var_B: uint16 = (*E).draygon_var_B;
    if draygon_var_B as ::core::ffi::c_int == 104 as ::core::ffi::c_int {
        let mut E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int);
        (*E3).base.current_instruction = addr_kDraygon_Ilist_9BDA as ::core::ffi::c_int as uint16;
        (*E3).base.instruction_timer = 1 as uint16;
    }
    (*Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .y_pos = *(((&raw mut g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(draygon_var_B as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).draygon_var_B =
        ((*E).draygon_var_B as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    if (*E).draygon_var_B as ::core::ffi::c_int == (*E).draygon_var_C as ::core::ffi::c_int {
        (*E).draygon_var_B = 0 as uint16;
        if *((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).draygon_var_A =
                (0xa589b3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            (*E).draygon_var_A =
                (0xa58c8e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    } else {
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            ((*E).draygon_var_D as uint32) << 16 as ::core::ffi::c_int
                | (*E).draygon_var_E as uint32,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_10() {
    Draygon_Func_3();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    printf(b"Wtf is varE26?\n\0" as *const u8 as *const ::core::ffi::c_char);
    let mut varE26: uint16 = 0 as uint16;
    let mut t: uint32 = EnemyFunc_Divide(
        (varE26 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint32,
        ((Abs16(
            ((*E).draygon_var_02 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int) as uint32,
    );
    SetHiLo(&raw mut (*E).draygon_var_D, &raw mut (*E).draygon_var_E, t);
    (*E).draygon_var_A = (0xa58a00 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.current_instruction = addr_kDraygon_Ilist_97BB as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).draygon_var_20 = 0 as uint16;
    (*E).draygon_var_B = (*E).draygon_var_C;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_11() {
    Draygon_Func_3();
    Draygon_Func_35();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut draygon_var_B: uint16 = (*E).draygon_var_B;
    if draygon_var_B as ::core::ffi::c_int == 104 as ::core::ffi::c_int {
        let mut E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int);
        (*E3).base.current_instruction = addr_kDraygon_Ilist_9813 as ::core::ffi::c_int as uint16;
        (*E3).base.instruction_timer = 1 as uint16;
    }
    (*Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .y_pos = *(((&raw mut g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(draygon_var_B as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).draygon_var_B =
        ((*E).draygon_var_B as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    if (*E).draygon_var_B != 0 {
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            -((((*E).draygon_var_D as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*E).draygon_var_E as ::core::ffi::c_int) as int32) as uint32,
        );
    } else {
        (*E).draygon_var_A =
            (0xa58a50 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_12() {
    Draygon_Func_3();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut t: uint32 = EnemyFunc_Divide(
        ((*E).draygon_var_C as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint32,
        (((*E).base.x_pos as ::core::ffi::c_int - (*E).draygon_var_00 as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int) as uint32,
    );
    (*E).draygon_var_D = (t >> 16 as ::core::ffi::c_int) as uint16;
    (*E).draygon_var_E = t as uint16;
    (*E).draygon_var_A = (0xa58a90 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_13() {
    Draygon_Func_3();
    Draygon_Func_35();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut draygon_var_B: uint16 = (*E).draygon_var_B;
    if draygon_var_B as ::core::ffi::c_int == 104 as ::core::ffi::c_int {
        let mut E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int);
        (*E3).base.current_instruction = addr_kDraygon_Ilist_97E7 as ::core::ffi::c_int as uint16;
        (*E3).base.instruction_timer = 1 as uint16;
    }
    (*Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .y_pos = *(((&raw mut g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(draygon_var_B as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).draygon_var_B =
        ((*E).draygon_var_B as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    if (*E).draygon_var_B as ::core::ffi::c_int == (*E).draygon_var_C as ::core::ffi::c_int {
        if *((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).draygon_var_A =
                (0xa587f4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).draygon_var_B = 0 as uint16;
            (*E).base.x_pos = (*E).draygon_var_00;
            (*E).base.y_pos = (*E).draygon_var_01;
        } else {
            (*E).draygon_var_A =
                (0xa58b0a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).base.x_pos = (*E).draygon_var_00;
        }
    } else {
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            -((((*E).draygon_var_D as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*E).draygon_var_E as ::core::ffi::c_int) as int32) as uint32,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_14() {
    Draygon_Func_3();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    (*E).base.x_pos = -(80 as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = 384 as uint16;
    (*E).draygon_var_E = 0 as uint16;
    (*E).draygon_var_D = 1 as uint16;
    (*E).draygon_var_0D = 0 as uint16;
    let mut E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int);
    (*E3).base.current_instruction = addr_kDraygon_Ilist_9C06 as ::core::ffi::c_int as uint16;
    (*E3).base.instruction_timer = 1 as uint16;
    (*E).draygon_var_A = (0xa58b52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.current_instruction = addr_kDraygon_Ilist_97D1 as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).draygon_var_20 = 1 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_15() {
    Draygon_Func_3();
    Draygon_Func_35();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut v2: uint16 = Abs16(
        ((*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
    );
    if v2 as ::core::ffi::c_int - 208 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E).draygon_var_A =
            (0xa58bae as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).draygon_var_03 = 16 as uint16;
    } else {
        (*E).base.y_pos = (CosineMult8bit((*E).draygon_var_0D, 32 as uint16) as ::core::ffi::c_int
            + 384 as ::core::ffi::c_int) as uint16;
        (*E).draygon_var_0D = ((*E).draygon_var_0D as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            as uint8 as uint16;
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            ((*E).draygon_var_D as uint32) << 16 as ::core::ffi::c_int
                | (*E).draygon_var_E as uint32,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_16() {
    let mut current_block: u64;
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    Draygon_Func_3();
    if *((&raw mut g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        current_block = 1399980599923904196;
    } else {
        if *((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            let mut v1: uint16 =
                ((*E).draygon_var_03 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
            (*E).draygon_var_03 = v1;
            if v1 == 0 {
                current_block = 11361156962102030361;
            } else {
                (*E).base.current_instruction =
                    addr_kDraygon_Ilist_9C90 as ::core::ffi::c_int as uint16;
                (*E).base.instruction_timer = 1 as uint16;
                current_block = 6483416627284290920;
            }
        } else {
            current_block = 6483416627284290920;
        }
        match current_block {
            6483416627284290920 => {
                (*E).base.y_pos = (CosineMult8bit((*E).draygon_var_0D, 32 as uint16)
                    as ::core::ffi::c_int
                    + 384 as ::core::ffi::c_int) as uint16;
                (*E).draygon_var_0D = ((*E).draygon_var_0D as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as uint8
                    as uint16;
                AddToHiLo(
                    &raw mut (*E).base.x_pos,
                    &raw mut (*E).base.x_subpos,
                    ((*E).draygon_var_D as uint32) << 16 as ::core::ffi::c_int
                        | (*E).draygon_var_E as uint32,
                );
                if (*E).base.x_pos as int16 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int
                    && (*E).base.x_pos as ::core::ffi::c_int - 672 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0
                {
                    current_block = 11361156962102030361;
                } else {
                    current_block = 7976072742316086414;
                }
            }
            _ => {}
        }
        match current_block {
            7976072742316086414 => {}
            _ => {
                if *((&raw mut g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    == 0
                {
                    let mut E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int);
                    (*E3).base.current_instruction =
                        addr_kDraygon_Ilist_9BDA as ::core::ffi::c_int as uint16;
                    (*E3).base.instruction_timer = 1 as uint16;
                    (*E).draygon_var_A =
                        (0xa58c33 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                    return;
                }
                current_block = 1399980599923904196;
            }
        }
    }
    match current_block {
        1399980599923904196 => {
            (*E).draygon_var_A =
                (0xa58e19 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_17(mut k: uint16) {
    Draygon_Func_35();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    (*E).base.y_pos = (CosineMult8bit((*E).draygon_var_0D, 32 as uint16) as ::core::ffi::c_int
        + 384 as ::core::ffi::c_int) as uint16;
    (*E).draygon_var_0D =
        ((*E).draygon_var_0D as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint8 as uint16;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).draygon_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).draygon_var_E as uint32,
    );
    if (*E).base.x_pos as int16 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int
        && (*E).base.x_pos as ::core::ffi::c_int - 672 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        (*E).draygon_var_A =
            (0xa589b3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.x_pos = 592 as uint16;
        (*E).base.y_pos = -(80 as ::core::ffi::c_int) as uint16;
        (*E).draygon_var_B = (*E).draygon_var_C;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_18() {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    (*E).base.x_pos = (*E).draygon_var_02;
    (*E).base.y_pos = 384 as uint16;
    (*E).draygon_var_E = 0 as uint16;
    (*E).draygon_var_D = 1 as uint16;
    (*E).draygon_var_0D = 0 as uint16;
    let mut E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int);
    (*E3).base.current_instruction = addr_kDraygon_Ilist_9C06 as ::core::ffi::c_int as uint16;
    (*E3).base.instruction_timer = 1 as uint16;
    (*E).draygon_var_A = (0xa58cd4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.current_instruction = addr_kDraygon_Ilist_97BB as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).draygon_var_20 = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_19() {
    Draygon_Func_3();
    Draygon_Func_35();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut v2: uint16 = Abs16(
        ((*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
    );
    if v2 as ::core::ffi::c_int - 208 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E).draygon_var_A =
            (0xa58d30 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).draygon_var_03 = 16 as uint16;
    } else {
        (*E).base.y_pos = (CosineMult8bit((*E).draygon_var_0D, 32 as uint16) as ::core::ffi::c_int
            + 384 as ::core::ffi::c_int) as uint16;
        (*E).draygon_var_0D = ((*E).draygon_var_0D as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            as uint8 as uint16;
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            -((((*E).draygon_var_D as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*E).draygon_var_E as ::core::ffi::c_int) as int32) as uint32,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_20() {
    let mut current_block: u64;
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    if *((&raw mut g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        current_block = 11384386367536015713;
    } else {
        if *((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            let mut v1: uint16 =
                ((*E).draygon_var_03 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
            (*E).draygon_var_03 = v1;
            if v1 == 0 {
                current_block = 17278387905106781498;
            } else {
                (*E).base.current_instruction =
                    addr_kDraygon_Ilist_98FE as ::core::ffi::c_int as uint16;
                (*E).base.instruction_timer = 1 as uint16;
                current_block = 6873731126896040597;
            }
        } else {
            current_block = 6873731126896040597;
        }
        match current_block {
            6873731126896040597 => {
                (*E).base.y_pos = (CosineMult8bit((*E).draygon_var_0D, 32 as uint16)
                    as ::core::ffi::c_int
                    + 384 as ::core::ffi::c_int) as uint16;
                (*E).draygon_var_0D = ((*E).draygon_var_0D as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as uint8
                    as uint16;
                AddToHiLo(
                    &raw mut (*E).base.x_pos,
                    &raw mut (*E).base.x_subpos,
                    -((((*E).draygon_var_D as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                        | (*E).draygon_var_E as ::core::ffi::c_int) as int32)
                        as uint32,
                );
                if ((*E).base.x_pos as int32) < 0 as int32
                    && (*E).base.x_pos as ::core::ffi::c_int + 80 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0
                {
                    current_block = 17278387905106781498;
                } else {
                    current_block = 10048703153582371463;
                }
            }
            _ => {}
        }
        match current_block {
            10048703153582371463 => {}
            _ => {
                if *((&raw mut g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    == 0
                {
                    let mut E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int);
                    (*E3).base.current_instruction =
                        addr_kDraygon_Ilist_97E7 as ::core::ffi::c_int as uint16;
                    (*E3).base.instruction_timer = 1 as uint16;
                    (*Get_Draygon(0 as ::core::ffi::c_int)).draygon_var_A =
                        (0xa58db2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                    return;
                }
                current_block = 11384386367536015713;
            }
        }
    }
    match current_block {
        11384386367536015713 => {
            (*Get_Draygon(0 as ::core::ffi::c_int)).draygon_var_A =
                (0xa58e19 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_21(mut k: uint16) {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    Draygon_Func_35();
    if *((&raw mut g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        (*E).draygon_var_A =
            (0xa58e19 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).base.y_pos = (CosineMult8bit((*E).draygon_var_0D, 32 as uint16) as ::core::ffi::c_int
            + 384 as ::core::ffi::c_int) as uint16;
        (*E).draygon_var_0D = ((*E).draygon_var_0D as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            as uint8 as uint16;
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            -((((*E).draygon_var_D as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*E).draygon_var_E as ::core::ffi::c_int) as int32) as uint32,
        );
        if ((*E).base.x_pos as int16 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int
            && (*E).base.x_pos as ::core::ffi::c_int + 80 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
        {
            (*E).draygon_var_A =
                (0xa587f4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).base.x_pos = -(80 as ::core::ffi::c_int) as uint16;
            (*E).base.y_pos = -(80 as ::core::ffi::c_int) as uint16;
            (*E).draygon_var_B = (*E).draygon_var_C;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_22() {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut v2: int16 = 0;
    let mut v4: uint16 = 0;
    Draygon_Func_3();
    if *((&raw mut g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
        v2 = 8 as int16;
        if (*E).draygon_var_20 == 0 {
            v2 = -(8 as ::core::ffi::c_int) as int16;
        }
        let mut v3: uint16 = Abs16(
            ((*E).base.x_pos as ::core::ffi::c_int + v2 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16,
        );
        if v3 as ::core::ffi::c_int - 8 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
            && {
                v4 = Abs16(
                    ((*E).base.y_pos as ::core::ffi::c_int
                        - *((&raw mut g_ram as *mut uint8)
                            .offset(0xafa as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int) as uint16,
                );
                v4 as ::core::ffi::c_int - 8 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0
            }
        {
            let mut v5: uint16 = addr_kDraygon_Ilist_9845 as ::core::ffi::c_int as uint16;
            if (*E).draygon_var_20 != 0 {
                v5 = addr_kDraygon_Ilist_9C38 as ::core::ffi::c_int as uint16;
            }
            let mut E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int);
            (*E3).base.current_instruction = v5;
            (*E3).base.instruction_timer = 1 as uint16;
            if CallSomeSamusCode(0xd as uint16) != 0 {
                (*E).draygon_var_A =
                    (0xa58f10 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            } else {
                Samus_SetGrabbedByDraygonPose(
                    ((*E).draygon_var_20 as ::core::ffi::c_int != 0 as ::core::ffi::c_int)
                        as ::core::ffi::c_int as uint16,
                );
                (*E).draygon_var_06 = 256 as uint16;
                (*E).draygon_var_07 = 384 as uint16;
                (*E).draygon_var_05 = 0 as uint16;
                (*E).draygon_var_09 = 0 as uint16;
                (*E).draygon_var_08 = 192 as uint16;
                (*E).draygon_var_0B = 2048 as uint16;
                (*E).draygon_var_A =
                    (0xa58f1e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            }
        } else {
            let mut varE20: uint16 = (64 as ::core::ffi::c_int
                - CalculateAngleFromXY(
                    (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
                    (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - (*E).base.y_pos as ::core::ffi::c_int) as uint16,
                ) as ::core::ffi::c_int) as uint8 as uint16;
            let mut pt: Point32 = ConvertAngleToXy(varE20, 2 as uint16);
            EnemyFunc_B691(varE20, pt);
        }
    } else {
        (*E).draygon_var_A =
            (0xa59154 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_23() {
    *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x9bc8c5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*Get_Draygon(0 as ::core::ffi::c_int)).draygon_var_A =
        (0xa59128 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_24(mut k: uint16) {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut v5: uint16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0xa64 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.flash_timer = ((*get_EnemyDef_A2((*E).base.enemy_ptr)).hurt_ai_time
            as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int) as uint16;
        (*E).base.ai_handler_bits =
            ((*E).base.ai_handler_bits as ::core::ffi::c_int | 2 as ::core::ffi::c_int) as uint16;
    } else {
        let mut v4: uint16 =
            Abs16(((*E).base.x_pos as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as uint16);
        if v4 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
            && {
                v5 = Abs16(
                    ((*E).base.y_pos as ::core::ffi::c_int - 384 as ::core::ffi::c_int) as uint16,
                );
                v5 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0
            }
        {
            (*E).draygon_var_A =
                (0xa58fd6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            let mut v6: uint16 = addr_kDraygon_Ilist_9922 as ::core::ffi::c_int as uint16;
            if (*E).draygon_var_20 != 0 {
                v6 = addr_kDraygon_Ilist_9CB4 as ::core::ffi::c_int as uint16;
            }
            let mut EK: *mut Enemy_Draygon = Get_Draygon(k as ::core::ffi::c_int);
            (*EK).base.current_instruction = v6;
            (*EK).base.instruction_timer = 1 as uint16;
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                | kEnemyProps_Tangible as ::core::ffi::c_int)
                as uint16;
        } else {
            let mut varE20: uint16 = (64 as ::core::ffi::c_int
                - CalculateAngleFromXY(
                    (256 as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
                    (384 as ::core::ffi::c_int - (*E).base.y_pos as ::core::ffi::c_int) as uint16,
                ) as ::core::ffi::c_int) as uint8 as uint16;
            let mut pt: Point32 = ConvertAngleToXy(varE20, 2 as uint16);
            EnemyFunc_B691(varE20, pt);
            Draygon_Func_40(k);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_25(mut k: uint16) {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut v6: uint16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0xa64 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.flash_timer = ((*get_EnemyDef_A2((*E).base.enemy_ptr)).hurt_ai_time
            as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int) as uint16;
        (*E).base.ai_handler_bits =
            ((*E).base.ai_handler_bits as ::core::ffi::c_int | 2 as ::core::ffi::c_int) as uint16;
    } else if *((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as uint8
        != 0
    {
        (*E).base.x_pos = ((*E).draygon_var_06 as ::core::ffi::c_int
            + CosineMult8bit((*E).draygon_var_08, (*E).draygon_var_05) as ::core::ffi::c_int)
            as uint16;
        (*E).base.y_pos = ((*E).draygon_var_07 as ::core::ffi::c_int
            + SineMult8bit(
                (*E).draygon_var_08,
                ((*E).draygon_var_05 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16,
            ) as ::core::ffi::c_int) as uint16;
        if (*Get_Draygon(k as ::core::ffi::c_int)).base.frame_counter as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            let mut x: uint16 = (if (*E).draygon_var_20 as ::core::ffi::c_int != 0 {
                (*E).base.x_pos as ::core::ffi::c_int + 32 as ::core::ffi::c_int
            } else {
                (*E).base.x_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int
            }) as uint16;
            let mut y: uint16 =
                ((*E).base.y_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
            CreateSpriteAtPos(x, y, 61 as uint16, 0 as uint16);
        }
        AddToHiLo(
            &raw mut (*E).draygon_var_05,
            &raw mut (*E).draygon_var_09,
            0x2000 as uint32,
        );
        let mut draygon_var_0A: uint16 = 0;
        let mut v8: uint16 = 0;
        if (*E).draygon_var_05 as ::core::ffi::c_int - 160 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
            || {
                v6 =
                    ((*E).draygon_var_0B as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
                (*E).draygon_var_0B = v6;
                (*E).draygon_var_08 = (*(&raw mut (*E).draygon_var_08 as *mut uint8)
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    + *(&raw mut v6 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ) as ::core::ffi::c_int) as uint8
                    as uint16;
                draygon_var_0A = (*E).draygon_var_0A;
                (*E).draygon_var_0A =
                    (draygon_var_0A as ::core::ffi::c_int - 0x4000 as ::core::ffi::c_int) as uint16;
                v8 = ((((*E).draygon_var_07 as uint32) << 16 as ::core::ffi::c_int
                    | draygon_var_0A as uint32)
                    .wrapping_sub(0x4000 as uint32)
                    >> 16 as ::core::ffi::c_int) as uint16;
                (*E).draygon_var_07 = v8;
                v8 as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0
            }
        {
            (*E).draygon_var_A =
                (0xa59105 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            Draygon_Func_40(k);
        }
    } else {
        (*E).draygon_var_0C = 64 as uint16;
        (*E).draygon_var_A =
            (0xa590d4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_26(mut k: uint16) {
    let mut v0: uint16 = k;
    let mut v3: uint16 = 0;
    Draygon_Func_40(v0);
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut v2: uint16 =
        ((*E).draygon_var_0C as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E).draygon_var_0C = v2;
    if v2 != 0 {
        if v2 as ::core::ffi::c_int == 63 as ::core::ffi::c_int {
            v3 = addr_kDraygon_Ilist_9AE8 as ::core::ffi::c_int as uint16;
            if (*E).draygon_var_20 != 0 {
                v3 = addr_kDraygon_Ilist_9EA1 as ::core::ffi::c_int as uint16;
            }
            let mut E2: *mut Enemy_Draygon = Get_Draygon(0x80 as ::core::ffi::c_int);
            (*E2).base.current_instruction = v3;
            (*E2).base.instruction_timer = 1 as uint16;
        }
    } else {
        (*E).draygon_var_A =
            (0xa58fd6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_27(mut k: uint16) {
    Draygon_Func_40(k);
    let mut v1: uint16 = addr_kDraygon_Ilist_9A68 as ::core::ffi::c_int as uint16;
    if (*Get_Draygon(k as ::core::ffi::c_int)).draygon_var_20 != 0 {
        v1 = addr_kDraygon_Ilist_9E21 as ::core::ffi::c_int as uint16;
    }
    let mut E: *mut Enemy_Draygon = Get_Draygon(0x80 as ::core::ffi::c_int);
    (*E).base.current_instruction = v1;
    (*E).base.instruction_timer = 1 as uint16;
    (*Get_Draygon(0 as ::core::ffi::c_int)).draygon_var_A =
        (0xa59124 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_28(mut k: uint16) {
    Draygon_Func_40(k);
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_29() {
    Samus_ReleaseFromDraygon_();
    *((&raw mut g_ram as *mut uint8).offset(0xa64 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    (*E).draygon_var_A = (0xa59154 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
    let mut v1: uint16 = addr_kDraygon_Ilist_9B5A as ::core::ffi::c_int as uint16;
    if (*E).draygon_var_20 != 0 {
        v1 = addr_kDraygon_Ilist_9F15 as ::core::ffi::c_int as uint16;
    }
    let mut E2: *mut Enemy_Draygon = Get_Draygon(0x80 as ::core::ffi::c_int);
    (*E2).base.current_instruction = v1;
    (*E2).base.instruction_timer = 1 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_30() {
    let mut v1: int16 = 0;
    Draygon_Func_3();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    v1 = ((*E).base.y_pos as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as int16;
    (*E).base.y_pos = v1 as uint16;
    if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_Tangible as ::core::ffi::c_int))
            as uint16;
        (*E).draygon_var_A =
            (0xa587f4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).draygon_var_B = 0 as uint16;
        (*E).base.x_pos = (*E).draygon_var_00;
        (*E).base.y_pos = (*E).draygon_var_01;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_31() {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    if (*Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .frame_counter as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut x: uint16 = (if (*E).draygon_var_20 as ::core::ffi::c_int != 0 {
            (*E).base.x_pos as ::core::ffi::c_int + 32 as ::core::ffi::c_int
        } else {
            (*E).base.x_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int
        }) as uint16;
        let mut y: uint16 =
            ((*E).base.y_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
        CreateSpriteAtPos(x, y, 61 as uint16, 0 as uint16);
    }
    (*E).draygon_var_43 = (64 as ::core::ffi::c_int
        - CalculateAngleFromXY(
            (64 as ::core::ffi::c_int
                - ((*E).base.x_pos as ::core::ffi::c_int >> 2 as ::core::ffi::c_int))
                as uint16,
            (120 as ::core::ffi::c_int
                - ((*E).base.y_pos as ::core::ffi::c_int >> 2 as ::core::ffi::c_int))
                as uint16,
        ) as ::core::ffi::c_int) as uint8 as uint16;
    let mut pt: Point32 = ConvertAngleToXy(
        *(&raw mut (*E).draygon_var_43 as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as uint16,
        1 as uint16,
    );
    EnemyFunc_B691(
        *(&raw mut (*E).draygon_var_43 as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as uint16,
        pt,
    );
    let mut v3: uint16 =
        Abs16(((*E).base.x_pos as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as uint16);
    if v3 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        let mut v4: uint16 =
            Abs16(((*E).base.y_pos as ::core::ffi::c_int - 480 as ::core::ffi::c_int) as uint16);
        if v4 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            Draygon_Func_43();
            (*E).draygon_var_A =
                (0xa59294 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            QueueMusic_Delayed8(3 as uint16);
            (*E).draygon_var_B = 416 as uint16;
            (*E).base.current_instruction =
                addr_kDraygon_Ilist_97B9 as ::core::ffi::c_int as uint16;
            (*E).base.instruction_timer = 1 as uint16;
            let mut v5: uint16 = ((*E).base.properties as ::core::ffi::c_int
                | kEnemyProps_Deleted as ::core::ffi::c_int)
                as uint16;
            (*Get_Draygon(0x80 as ::core::ffi::c_int)).base.properties = v5;
            (*Get_Draygon(0xc0 as ::core::ffi::c_int)).base.properties = v5;
            let mut v6: uint16 = addr_kDraygon_Ilist_999C as ::core::ffi::c_int as uint16;
            if (*E).draygon_var_20 != 0 {
                v6 = addr_kDraygon_Ilist_9D3E as ::core::ffi::c_int as uint16;
            }
            let mut E1: *mut Enemy_Draygon = Get_Draygon(0x40 as ::core::ffi::c_int);
            (*E1).base.current_instruction = v6;
            (*E1).base.instruction_timer = 1 as uint16;
            (*E1).draygon_var_A =
                (0xa5804b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_32() {
    Draygon_Func_34();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    (*E).draygon_var_B = (*E).draygon_var_B.wrapping_sub(1);
    if (*E).draygon_var_B != 0 {
        Draygon_Func_42(0 as uint16);
    } else {
        (*E).draygon_var_A =
            (0xa592ab as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_33() {
    Draygon_Func_34();
    Draygon_Func_42(0 as uint16);
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    if (*E).base.y_pos as ::core::ffi::c_int - 576 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E).base.current_instruction = addr_kDraygon_Ilist_98ED as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        let mut v1: uint16 = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
        (*E).base.properties = v1;
        (*Get_Draygon(0x40 as ::core::ffi::c_int)).base.properties = v1;
        let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0x79f as ::core::ffi::c_int as isize)
            as *mut uint16);
        let ref mut fresh3 = *((&raw mut g_ram as *mut uint8)
            .offset(0xd828 as ::core::ffi::c_int as isize)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as *mut uint8 as *mut uint16);
        *fresh3 = (*fresh3 as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint16;
        Enemy_ItemDrop_Draygon(v2);
        Draygon_Func_44();
    }
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_34() {
    if *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut x: uint16 = ((NextRandom() as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int)
            + 192 as ::core::ffi::c_int) as uint16;
        let mut y: uint16 =
            (((*((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x3f00 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int)
                + 400 as ::core::ffi::c_int) as uint16;
        CreateSpriteAtPos(x, y, 21 as uint16, 0 as uint16);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_35() {
    let mut Draygon: *mut Enemy_Draygon = Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*Draygon).base.frame_counter as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        Draygon = Get_Draygon(0 as ::core::ffi::c_int);
        CreateSpriteAtPos(
            ((*Draygon).base.x_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16,
            ((*Draygon).base.y_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16,
            24 as uint16,
            0 as uint16,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_36() {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    *((&raw mut g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize) as *mut uint16) =
        ((*E).draygon_var_5E as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int
            - 450 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize) as *mut uint16) =
        ((*E).draygon_var_5F as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - (*E).base.y_pos as ::core::ffi::c_int
            - 192 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_37() {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    if *((&raw mut g_ram as *mut uint8).offset(0x8d as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x40 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        Draygon_Func_39();
    } else {
        Draygon_Func_38();
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x91 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x4000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut v0: uint16 = addr_kDraygon_Ilist_98FE as ::core::ffi::c_int as uint16;
        if (*E).draygon_var_20 != 0 {
            v0 = addr_kDraygon_Ilist_9C90 as ::core::ffi::c_int as uint16;
        }
        (*E).base.current_instruction = v0;
        (*E).base.instruction_timer = 1 as uint16;
    } else if *((&raw mut g_ram as *mut uint8).offset(0x91 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x91 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let mut v4: uint16 = addr_kDraygon_Ilist_9825 as ::core::ffi::c_int as uint16;
            if (*E).draygon_var_20 != 0 {
                v4 = addr_kDraygon_Ilist_9C18 as ::core::ffi::c_int as uint16;
            }
            let mut E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int);
            (*E3).base.current_instruction = v4;
            (*E3).base.instruction_timer = 1 as uint16;
        }
    } else {
        let mut v2: uint16 = addr_kDraygon_Ilist_99FC as ::core::ffi::c_int as uint16;
        if (*E).draygon_var_20 != 0 {
            v2 = addr_kDraygon_Ilist_9D9E as ::core::ffi::c_int as uint16;
        }
        let mut E2: *mut Enemy_Draygon = Get_Draygon(0x80 as ::core::ffi::c_int);
        (*E2).base.current_instruction = v2;
        (*E2).base.instruction_timer = 1 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_38() {
    let mut current_block: u64;
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x8d as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: uint16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0x8d as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x200 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        || {
            (*E).base.x_pos = (*E).base.x_pos.wrapping_sub(1);
            v0 as ::core::ffi::c_int
                == *((&raw mut g_ram as *mut uint8).offset(0x1866 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
        }
    {
        if v0 as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            current_block = 1196177745739367891;
        } else {
            (*E).base.x_pos = (*E).base.x_pos.wrapping_add(1);
            if v0 as ::core::ffi::c_int
                == *((&raw mut g_ram as *mut uint8).offset(0x1866 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
            {
                current_block = 1196177745739367891;
            } else {
                *((&raw mut g_ram as *mut uint8).offset(0x1866 as ::core::ffi::c_int as isize)
                    as *mut uint16) = v0;
                v2 = addr_kDraygon_Ilist_97D1 as ::core::ffi::c_int as uint16;
                (*E).draygon_var_20 = 1 as uint16;
                current_block = 3276175668257526147;
            }
        }
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0x1866 as ::core::ffi::c_int as isize)
            as *mut uint16) = v0;
        v2 = addr_kDraygon_Ilist_97BB as ::core::ffi::c_int as uint16;
        (*E).draygon_var_20 = 0 as uint16;
        current_block = 3276175668257526147;
    }
    match current_block {
        3276175668257526147 => {
            (*E).base.current_instruction = v2;
            (*E).base.instruction_timer = 1 as uint16;
        }
        _ => {}
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x8d as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x800 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    } else if *((&raw mut g_ram as *mut uint8).offset(0x8d as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x400 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_39() {
    let mut current_block: u64;
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x8d as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: uint16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0x8d as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x200 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        || {
            (*E).base.x_pos =
                ((*E).base.x_pos as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
            v0 = *((&raw mut g_ram as *mut uint8).offset(0x8d as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut g_ram as *mut uint8).offset(0x8d as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == *((&raw mut g_ram as *mut uint8).offset(0x1866 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
        }
    {
        if v0 as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            current_block = 18201098939790280272;
        } else {
            (*E).base.x_pos =
                ((*E).base.x_pos as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
            if *((&raw mut g_ram as *mut uint8).offset(0x8d as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == *((&raw mut g_ram as *mut uint8).offset(0x1866 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
            {
                current_block = 18201098939790280272;
            } else {
                *((&raw mut g_ram as *mut uint8).offset(0x1866 as ::core::ffi::c_int as isize)
                    as *mut uint16) = *((&raw mut g_ram as *mut uint8)
                    .offset(0x8d as ::core::ffi::c_int as isize)
                    as *mut uint16);
                v2 = addr_kDraygon_Ilist_97D1 as ::core::ffi::c_int as uint16;
                (*E).draygon_var_20 = 1 as uint16;
                current_block = 3276175668257526147;
            }
        }
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0x1866 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut g_ram as *mut uint8)
            .offset(0x8d as ::core::ffi::c_int as isize)
            as *mut uint16);
        v2 = addr_kDraygon_Ilist_97BB as ::core::ffi::c_int as uint16;
        (*E).draygon_var_20 = 0 as uint16;
        current_block = 3276175668257526147;
    }
    match current_block {
        3276175668257526147 => {
            (*E).base.current_instruction = v2;
            (*E).base.instruction_timer = 1 as uint16;
        }
        _ => {}
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x8d as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x800 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos =
            ((*E).base.y_pos as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    } else if *((&raw mut g_ram as *mut uint8).offset(0x8d as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x400 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
        (*E).base.y_pos =
            ((*E).base.y_pos as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_40(mut k: uint16) {
    let mut v1: int16 = 0;
    v1 = 8 as int16;
    if (*Get_Draygon(k as ::core::ffi::c_int)).draygon_var_20 == 0 {
        v1 = -(8 as ::core::ffi::c_int) as int16;
    }
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16) =
        ((*E).base.x_pos as ::core::ffi::c_int + v1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16) =
        ((*E).base.y_pos as ::core::ffi::c_int + 40 as ::core::ffi::c_int) as uint16;
    EnemyFunc_B7A1();
    if *((&raw mut g_ram as *mut uint8).offset(0xa64 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa64 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        (*E).draygon_var_A =
            (0xa59154 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Draygon(0 as ::core::ffi::c_int))
        .base
        .current_instruction = *jp.offset(0 as ::core::ffi::c_int as isize);
    (*Get_Draygon(0x40 as ::core::ffi::c_int))
        .base
        .current_instruction = *jp.offset(1 as ::core::ffi::c_int as isize);
    (*Get_Draygon(0x80 as ::core::ffi::c_int))
        .base
        .current_instruction = *jp.offset(2 as ::core::ffi::c_int as isize);
    (*Get_Draygon(0xc0 as ::core::ffi::c_int))
        .base
        .current_instruction = *jp.offset(3 as ::core::ffi::c_int as isize);
    (*Get_Draygon(0 as ::core::ffi::c_int))
        .base
        .instruction_timer = 1 as uint16;
    (*Get_Draygon(0x40 as ::core::ffi::c_int))
        .base
        .instruction_timer = 1 as uint16;
    (*Get_Draygon(0x80 as ::core::ffi::c_int))
        .base
        .instruction_timer = 1 as uint16;
    (*Get_Draygon(0xc0 as ::core::ffi::c_int))
        .base
        .instruction_timer = 1 as uint16;
    return jp.offset(4 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Hurt() {
    let mut v0: uint16 = addr_kDraygon_MorePalettes2 as ::core::ffi::c_int as uint16;
    if (*Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .flash_timer as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v0 = addr_word_A5A297 as ::core::ffi::c_int as uint16;
    }
    memcpy(
        (&raw mut g_ram as *mut uint8)
            .offset(addr_kDraygon_BigSprmap_C08F as ::core::ffi::c_int as isize)
            .offset(17 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
        RomPtr_A5(v0 as uint16_t) as *const ::core::ffi::c_void,
        32 as size_t,
    );
    if (*Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .flash_timer as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut v3: uint16 = (4 as ::core::ffi::c_int
            * (*Get_Draygon(0 as ::core::ffi::c_int)).draygon_var_0E as ::core::ffi::c_int)
            as uint16;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i != 8 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 89 as ::core::ffi::c_int) as isize) =
                *(g_rom.offset(
                    (((0xa596af as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa596af as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            i += 2 as ::core::ffi::c_int;
        }
    }
    let mut v5: uint16 = addr_kDraygon_Palette as ::core::ffi::c_int as uint16;
    if (*Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .flash_timer as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v5 = addr_word_A5A297 as ::core::ffi::c_int as uint16;
    }
    memcpy(
        (&raw mut g_ram as *mut uint8)
            .offset(addr_kDraygon_BigSprmap_C11B as ::core::ffi::c_int as isize)
            .offset(197 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
        RomPtr_A5(v5 as uint16_t) as *const ::core::ffi::c_void,
        32 as size_t,
    );
    if *((&raw mut g_ram as *mut uint8).offset(0xa64 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
        if (*E).base.frame_counter as ::core::ffi::c_int & 7 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            let mut v9: int16 =
                ((*E).base.health as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as int16;
            if (v9 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                v9 = 0 as int16;
            }
            (*E).base.health = v9 as uint16;
            sub_A5960D();
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Touch() {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy();
    sub_A5960D();
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Shot() {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    if (*E).draygon_var_0F as ::core::ffi::c_int - 152 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).draygon_var_0F =
            ((*E).draygon_var_0F as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
    }
    NormalEnemyShotAiSkipDeathAnim_CurEnemy();
    sub_A5960D();
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Powerbomb() {
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy();
    sub_A5960D();
}
#[no_mangle]
pub unsafe extern "C" fn sub_A5960D() {
    if (*Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .health
        != 0
    {
        Draygon_Func_41();
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x9bc8c5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut v0: uint16 = addr_kDraygon_Ilist_9867 as ::core::ffi::c_int as uint16;
        let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
        if (*E).draygon_var_20 != 0 {
            v0 = addr_kDraygon_Ilist_9C5A as ::core::ffi::c_int as uint16;
        }
        (*E).base.current_instruction = v0;
        (*E).base.instruction_timer = 1 as uint16;
        let mut v3: uint16 = addr_kDraygon_Ilist_997A as ::core::ffi::c_int as uint16;
        if (*E).draygon_var_20 != 0 {
            v3 = addr_kDraygon_Ilist_9D1C as ::core::ffi::c_int as uint16;
        }
        let mut E1: *mut Enemy_Draygon = Get_Draygon(0x40 as ::core::ffi::c_int);
        (*E1).base.current_instruction = v3;
        (*E1).base.instruction_timer = 1 as uint16;
        (*E1).draygon_var_A =
            (0xa5804b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).draygon_var_A =
            (0xa59185 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        Samus_ReleaseFromDraygon_();
        *((&raw mut g_ram as *mut uint8).offset(0xa64 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        (*E).draygon_var_43 = (64 as ::core::ffi::c_int
            - CalculateAngleFromXY(
                (256 as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
                (480 as ::core::ffi::c_int - (*E).base.y_pos as ::core::ffi::c_int) as uint16,
            ) as ::core::ffi::c_int) as uint8 as uint16;
        let mut pt: Point32 = ConvertAngleToXy(
            *(&raw mut (*E).draygon_var_43 as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as uint16,
            1 as uint16,
        );
        (*E).draygon_var_28 = (pt.x >> 16 as ::core::ffi::c_int) as uint16;
        (*E).draygon_var_29 = pt.x as uint16;
        (*E).draygon_var_2A = (pt.y >> 16 as ::core::ffi::c_int) as uint16;
        (*E).draygon_var_2B = pt.y as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_41() {
    let mut i: ::core::ffi::c_int = 0;
    let mut j: uint16 = 0;
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    i = 0 as ::core::ffi::c_int;
    while !(((*E).base.health as ::core::ffi::c_int
        - *(g_rom.offset(
            (((0xa596ef as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa596ef as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int)
    {
        i += 2 as ::core::ffi::c_int;
    }
    if i != (*E).draygon_var_0E as ::core::ffi::c_int {
        (*E).draygon_var_0E = i as uint16;
        let mut v2: uint16 =
            (4 as ::core::ffi::c_int * (*E).draygon_var_0E as ::core::ffi::c_int) as uint16;
        j = 0 as uint16;
        while j as ::core::ffi::c_int != 8 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(
                    ((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 89 as ::core::ffi::c_int) as isize,
                ) = *(g_rom.offset(
                (((0xa596af as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa596af as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            j = (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_13(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Draygon(k as ::core::ffi::c_int)).draygon_var_A =
        *jp.offset(0 as ::core::ffi::c_int as isize);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
unsafe extern "C" fn Draygon_Func_47() -> Point16U {
    NextRandom();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut r18: uint16 = ((*((&raw mut g_ram as *mut uint8)
        .offset(0x5e5 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x7f as ::core::ffi::c_int)
        - 64 as ::core::ffi::c_int) as uint16;
    let mut r20: uint16 = (((*((&raw mut g_ram as *mut uint8)
        .offset(0x5e5 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x7f00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int)
        - 64 as ::core::ffi::c_int) as uint16;
    return Point16U {
        x: (r18 as ::core::ffi::c_int + (*E).base.x_pos as ::core::ffi::c_int) as uint16,
        y: (r20 as ::core::ffi::c_int + (*E).base.y_pos as ::core::ffi::c_int) as uint16,
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_8(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut pt: Point16U = Draygon_Func_47();
    CreateSpriteAtPos(pt.x, pt.y, 21 as uint16, 0 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_7(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut pt: Point16U = Draygon_Func_47();
    CreateSpriteAtPos(pt.x, pt.y, 3 as uint16, 0 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_6(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut pt: Point16U = Draygon_Func_47();
    CreateSpriteAtPos(pt.x, pt.y, 29 as uint16, 0 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_9(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut pt: Point16U = Draygon_Func_47();
    CreateSpriteAtPos(pt.x, pt.y, 24 as uint16, 0 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_2(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    *((&raw mut g_ram as *mut uint8).offset(0xa9 as ::core::ffi::c_int as isize) as *mut uint16) =
        12 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_11(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E2: *mut Enemy_Draygon = Get_Draygon(0x80 as ::core::ffi::c_int);
    (*E2).base.instruction_timer = 1 as uint16;
    (*E2).base.current_instruction = addr_kDraygon_Ilist_97B9 as ::core::ffi::c_int as uint16;
    let mut E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int);
    (*E3).base.instruction_timer = 1 as uint16;
    (*E3).base.current_instruction = addr_kDraygon_Ilist_97B9 as ::core::ffi::c_int as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_5(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_15(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    (*E).draygon_var_0F = 24 as uint16;
    let mut v3: *const uint8 = RomPtr_A0((*E).base.enemy_ptr as uint16_t);
    Samus_DealDamage(SuitDamageDivision(
        *(v3.offset(6 as ::core::ffi::c_int as isize) as *mut uint16),
    ));
    *((&raw mut g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) = 32 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = 7 as uint16;
    CreateSpriteAtPos(
        *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as uint16,
        21 as uint16,
        0 as uint16,
    );
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_17(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    *((&raw mut g_ram as *mut uint8).offset(0xa9 as ::core::ffi::c_int as isize) as *mut uint16) =
        12 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_14(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    (*E).draygon_var_5E = *jp.offset(0 as ::core::ffi::c_int as isize);
    (*E).draygon_var_5F = *jp.offset(1 as ::core::ffi::c_int as isize);
    return jp.offset(2 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_16(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Draygon(0 as ::core::ffi::c_int)).draygon_var_A =
        *jp.offset(0 as ::core::ffi::c_int as isize);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_10(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max6(*jp.offset(0 as ::core::ffi::c_int as isize));
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_4(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx3_Max6(*jp.offset(0 as ::core::ffi::c_int as isize));
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_12(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    eproj_spawn_pt = Point16U {
        x: ((*E).base.x_pos as ::core::ffi::c_int - 28 as ::core::ffi::c_int) as uint16,
        y: ((*E).base.y_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16,
    };
    *((&raw mut g_ram as *mut uint8).offset(0x1995 as ::core::ffi::c_int as isize)
        as *mut uint16) = ((NextRandom() as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int)
        + 128 as ::core::ffi::c_int) as uint16;
    SpawnEprojWithGfx(
        2 as uint16,
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        addr_stru_868E50 as ::core::ffi::c_int as uint16,
    );
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_18(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    eproj_spawn_pt = Point16U {
        x: ((*E).base.x_pos as ::core::ffi::c_int + 24 as ::core::ffi::c_int) as uint16,
        y: ((*E).base.y_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16,
    };
    *((&raw mut g_ram as *mut uint8).offset(0x1995 as ::core::ffi::c_int as isize)
        as *mut uint16) = ((NextRandom() as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int)
        + 192 as ::core::ffi::c_int) as uint16;
    SpawnEprojWithGfx(
        2 as uint16,
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        addr_stru_868E50 as ::core::ffi::c_int as uint16,
    );
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_42(mut varE24: uint16) {
    let mut v0: uint16 = 62 as uint16;
    let mut i: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v2: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        let mut v3: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *(g_rom.offset(
            (((0xa5a1df as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa5a1df as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            AddToHiLo(
                ((&raw mut g_ram as *mut uint8).offset(0xf0f8 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as *mut uint16,
                ((&raw mut g_ram as *mut uint8).offset(0xf178 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as *mut uint16,
                *(g_rom.offset(
                    (((0xa5a1af as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa5a1af as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v2 as isize) as uint32,
            );
        } else {
            AddToHiLo(
                ((&raw mut g_ram as *mut uint8).offset(0xf0f8 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as *mut uint16,
                ((&raw mut g_ram as *mut uint8).offset(0xf178 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as *mut uint16,
                (-(*(g_rom.offset(
                    (((0xa5a1af as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa5a1af as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int)
                    - ((varE24 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int))
                    as uint32,
            );
        }
        let mut v8: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xf278 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize);
        if *(g_rom.offset(
            (((0xa5a1df as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa5a1df as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            + 128 as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            AddToHiLo(
                ((&raw mut g_ram as *mut uint8).offset(0xf1f8 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as *mut uint16,
                ((&raw mut g_ram as *mut uint8).offset(0xf278 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as *mut uint16,
                *(g_rom.offset(
                    (((0xa5a1af as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa5a1af as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v2 + 1 as ::core::ffi::c_int) as isize) as uint32,
            );
        } else {
            AddToHiLo(
                ((&raw mut g_ram as *mut uint8).offset(0xf1f8 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as *mut uint16,
                ((&raw mut g_ram as *mut uint8).offset(0xf278 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as *mut uint16,
                -(*(g_rom.offset(
                    (((0xa5a1af as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa5a1af as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v2 + 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int) as uint32,
            );
        }
        v0 = (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        i -= 4 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_43() {
    let mut v1: int16 = 0;
    let mut j: int16 = 0;
    let mut i: ::core::ffi::c_int = 62 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xef78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    v1 = 2 as int16;
    let mut v2: uint16 = 20 as uint16;
    loop {
        let mut v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        CreateSpriteAtPos(
            *(g_rom.offset(
                (((0xa5a1c7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa5a1c7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v3 as isize),
            *(g_rom.offset(
                (((0xa5a1c7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa5a1c7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v3 + 1 as ::core::ffi::c_int) as isize),
            59 as uint16,
            0xe00 as uint16,
        );
        v2 = (v2 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
        v1 -= 1;
        if !(v1 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
    j = 2 as int16;
    while j as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        let mut v5: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        CreateSpriteAtPos(
            *(g_rom.offset(
                (((0xa5a1c7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa5a1c7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v5 as isize),
            *(g_rom.offset(
                (((0xa5a1c7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa5a1c7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v5 + 1 as ::core::ffi::c_int) as isize),
            60 as uint16,
            0xe00 as uint16,
        );
        v2 = (v2 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
        j -= 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_44() {
    let mut i: ::core::ffi::c_int = 62 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xef78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_45() {
    CreateSpriteAtPos(16 as uint16, 384 as uint16, 59 as uint16, 0xe00 as uint16);
    CreateSpriteAtPos(16 as uint16, 384 as uint16, 59 as uint16, 0xe00 as uint16);
    CreateSpriteAtPos(16 as uint16, 384 as uint16, 59 as uint16, 0xe00 as uint16);
    CreateSpriteAtPos(16 as uint16, 384 as uint16, 59 as uint16, 0xe00 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_46() {
    let mut v0: uint16 = 62 as uint16;
    loop {
        let mut v1: uint16 = ((*Get_Draygon(0 as ::core::ffi::c_int)).draygon_var_46
            as ::core::ffi::c_int
            + *(g_rom.offset(
                (((0xa5a19f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa5a19f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    ((v0 as ::core::ffi::c_int - 56 as ::core::ffi::c_int) as uint16
                        as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int) as uint16;
        if v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            let mut v2: uint16 = v1;
            if *((g_rom.offset(
                (((0xa5ce07 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa5ce07 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(v1 as isize) as *mut uint8 as *mut uint16)
                as ::core::ffi::c_int
                == 0x8080 as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0xef78 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                    0 as uint16;
            } else {
                let mut v3: ::core::ffi::c_int =
                    v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                let ref mut fresh1 = *((&raw mut g_ram as *mut uint8)
                    .offset(0xf0f8 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize);
                *fresh1 = (*fresh1 as ::core::ffi::c_int
                    + SignExtend8(
                        *(g_rom.offset(
                            (((0xa5ce07 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                << 15 as ::core::ffi::c_int
                                | 0xa5ce07 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                                & 0x3fffff as ::core::ffi::c_int)
                                as isize,
                        ) as *const uint8 as *mut uint8)
                            .offset(v1 as isize) as uint16,
                    ) as ::core::ffi::c_int) as uint16;
                let ref mut fresh2 = *((&raw mut g_ram as *mut uint8)
                    .offset(0xf1f8 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize);
                *fresh2 = (*fresh2 as ::core::ffi::c_int
                    + SignExtend8(
                        *(g_rom.offset(
                            (((0xa5ce07 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                << 15 as ::core::ffi::c_int
                                | 0xa5ce07 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                                & 0x3fffff as ::core::ffi::c_int)
                                as isize,
                        ) as *const uint8 as *mut uint8)
                            .offset((v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
                            as uint16,
                    ) as ::core::ffi::c_int) as uint16;
            }
        }
        v0 = (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if !((v0 as ::core::ffi::c_int - 56 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    (*E).draygon_var_46 =
        ((*E).draygon_var_46 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn DraygonsEye_Init() {
    let mut E: *mut Enemy_Draygon = Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.current_instruction = addr_kDraygon_Ilist_9944 as ::core::ffi::c_int as uint16;
    (*E).draygon_var_A = (0xa5804b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_3(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Draygon(0x40 as ::core::ffi::c_int)).draygon_var_A =
        *jp.offset(0 as ::core::ffi::c_int as isize);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn DraygonsEye_Main() {
    let mut E: *mut Enemy_Draygon = Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallDraygonFunc(
        ((*E).draygon_var_A as ::core::ffi::c_int | 0xa50000 as ::core::ffi::c_int) as uint32,
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_48(mut k: uint16) {
    if (*Get_Draygon(k as ::core::ffi::c_int)).base.frame_counter as ::core::ffi::c_int
        & 0x7f as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut E1: *mut Enemy_Draygon = Get_Draygon(0x40 as ::core::ffi::c_int);
        eproj_spawn_pt = Point16U {
            x: ((*E1).base.x_pos as ::core::ffi::c_int - 24 as ::core::ffi::c_int) as uint16,
            y: ((*E1).base.y_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16,
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
            0x18 as uint16,
        );
    }
    let mut E0: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut v4: uint16 = CalculateAngleFromXY(
        (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - ((*E0).base.x_pos as ::core::ffi::c_int - 24 as ::core::ffi::c_int))
            as uint16,
        (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - ((*E0).base.y_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int))
            as uint16,
    );
    let mut E: *mut Enemy_Draygon = Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if v4 as ::core::ffi::c_int != (*E).draygon_var_F as ::core::ffi::c_int {
        let mut v6: uint16 = addr_kDraygon_Ilist_99BA as ::core::ffi::c_int as uint16;
        if v4 as ::core::ffi::c_int - 32 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v6 = addr_kDraygon_Ilist_99B4 as ::core::ffi::c_int as uint16;
            if v4 as ::core::ffi::c_int - 96 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v6 = addr_kDraygon_Ilist_99C0 as ::core::ffi::c_int as uint16;
                if v4 as ::core::ffi::c_int - 160 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    v6 = addr_kDraygon_Ilist_99AE as ::core::ffi::c_int as uint16;
                    if v4 as ::core::ffi::c_int - 224 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0
                    {
                        v6 = addr_kDraygon_Ilist_99BA as ::core::ffi::c_int as uint16;
                    }
                }
            }
        }
        (*E).draygon_var_F = v4;
        (*E).base.current_instruction = v6;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_49(mut k: uint16) {
    let mut E0: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    if (*Get_Draygon(k as ::core::ffi::c_int)).base.frame_counter as ::core::ffi::c_int
        & 0x7f as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        eproj_spawn_pt = Point16U {
            x: ((*E0).base.x_pos as ::core::ffi::c_int + 24 as ::core::ffi::c_int) as uint16,
            y: ((*E0).base.y_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16,
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
            0x18 as uint16,
        );
    }
    let mut v4: uint16 = CalculateAngleFromXY(
        (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - ((*E0).base.x_pos as ::core::ffi::c_int + 24 as ::core::ffi::c_int))
            as uint16,
        (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - ((*E0).base.y_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int))
            as uint16,
    );
    let mut E: *mut Enemy_Draygon = Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if v4 as ::core::ffi::c_int != (*E).draygon_var_F as ::core::ffi::c_int {
        let mut v6: uint16 = addr_kDraygon_Ilist_9D5C as ::core::ffi::c_int as uint16;
        if v4 as ::core::ffi::c_int - 32 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v6 = addr_kDraygon_Ilist_9D50 as ::core::ffi::c_int as uint16;
            if v4 as ::core::ffi::c_int - 96 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v6 = addr_kDraygon_Ilist_9D62 as ::core::ffi::c_int as uint16;
                if v4 as ::core::ffi::c_int - 160 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    v6 = addr_kDraygon_Ilist_9D56 as ::core::ffi::c_int as uint16;
                    if v4 as ::core::ffi::c_int - 224 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0
                    {
                        v6 = addr_kDraygon_Ilist_9D5C as ::core::ffi::c_int as uint16;
                    }
                }
            }
        }
        (*E).draygon_var_F = v4;
        (*E).base.current_instruction = v6;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn DraygonsTail_Init() {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v0).current_instruction = addr_kDraygon_Ilist_99FC as ::core::ffi::c_int as uint16;
    (*v0).palette_index = 0xe00 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn DraygonsArms_Init() {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v0).current_instruction = addr_kDraygon_Ilist_97E7 as ::core::ffi::c_int as uint16;
    (*v0).palette_index = 0xe00 as uint16;
    (*v0).layer = 2 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_25(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    if (*E).draygon_var_0B as ::core::ffi::c_int - 40 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).draygon_var_0B =
            ((*E).draygon_var_0B as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_24(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Draygon(0 as ::core::ffi::c_int)).draygon_var_2F = 0 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_50(mut j: uint16) {
    let mut v1: *const uint8 = RomPtr_A5(j as uint16_t);
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    (*E).draygon_var_0B = *(v1 as *mut uint16);
    (*E).draygon_var_0C = *(v1.offset(2 as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).draygon_var_0A = *(v1.offset(4 as ::core::ffi::c_int as isize) as *mut uint16);
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_21(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    (*E).draygon_var_0B = *jp.offset(0 as ::core::ffi::c_int as isize);
    (*E).draygon_var_0C = *jp.offset(1 as ::core::ffi::c_int as isize);
    return jp.offset(2 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_51(mut j: uint16) {
    let mut v1: uint16 = *(RomPtr_A5(j as uint16_t) as *mut uint16);
    (*Get_Draygon(0 as ::core::ffi::c_int)).draygon_var_0B = v1;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Func_52(mut j: uint16) {
    let mut v1: uint16 = *(RomPtr_A5(j as uint16_t) as *mut uint16);
    (*Get_Draygon(0 as ::core::ffi::c_int)).draygon_var_0C = v1;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_53(mut k: uint16, mut j: uint16) -> uint16 {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    (*E).draygon_var_0B = ((*E).draygon_var_0B as ::core::ffi::c_int
        + *(RomPtr_A5(j as uint16_t) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_22(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    *((&raw mut g_ram as *mut uint8).offset(0x9000 as ::core::ffi::c_int as isize)
        as *mut uint16) = *jp.offset(0 as ::core::ffi::c_int as isize);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_27(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    (*E).base.x_pos = 128 as uint16;
    (*E).base.y_pos = 624 as uint16;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & 0x5bff as ::core::ffi::c_int
        | 0xa000 as ::core::ffi::c_int) as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_23(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max6(*jp.offset(0 as ::core::ffi::c_int as isize));
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_30(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    Enemy_ItemDrop_SporeSpawn(k);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_20(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Draygon(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .draygon_var_A = *jp.offset(0 as ::core::ffi::c_int as isize);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_29(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut v2: uint16 = *jp.offset(0 as ::core::ffi::c_int as isize);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 32 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 144 as ::core::ffi::c_int) as isize) =
            *(g_rom.offset(
                (((0xa5e3f9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa5e3f9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int + i >> 1 as ::core::ffi::c_int) as isize);
        i += 2 as ::core::ffi::c_int;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 != 32 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(((i_0 >> 1 as ::core::ffi::c_int) + 64 as ::core::ffi::c_int) as isize) =
            *(g_rom.offset(
                (((0xa5e4f9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa5e4f9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int + i_0 >> 1 as ::core::ffi::c_int) as isize);
        i_0 += 2 as ::core::ffi::c_int;
    }
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 != 32 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(((i_1 >> 1 as ::core::ffi::c_int) + 112 as ::core::ffi::c_int) as isize) =
            *(g_rom.offset(
                (((0xa5e5d9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa5e5d9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int + i_1 >> 1 as ::core::ffi::c_int) as isize);
        i_1 += 2 as ::core::ffi::c_int;
    }
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_19(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut v2: uint16 = *jp.offset(0 as ::core::ffi::c_int as isize);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 32 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 144 as ::core::ffi::c_int) as isize) =
            *(g_rom.offset(
                (((0xa5e3f9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa5e3f9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int + i >> 1 as ::core::ffi::c_int) as isize);
        i += 2 as ::core::ffi::c_int;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 != 32 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(((i_0 >> 1 as ::core::ffi::c_int) + 64 as ::core::ffi::c_int) as isize) =
            *(g_rom.offset(
                (((0xa5e4f9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa5e4f9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int + i_0 >> 1 as ::core::ffi::c_int) as isize);
        i_0 += 2 as ::core::ffi::c_int;
    }
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 != 32 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(((i_1 >> 1 as ::core::ffi::c_int) + 112 as ::core::ffi::c_int) as isize) =
            *(g_rom.offset(
                (((0xa5e5d9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa5e5d9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int + i_1 >> 1 as ::core::ffi::c_int) as isize);
        i_1 += 2 as ::core::ffi::c_int;
    }
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_28(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    NextRandom();
    let mut r18: uint16 = ((*((&raw mut g_ram as *mut uint8)
        .offset(0x5e5 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x7f as ::core::ffi::c_int)
        - 64 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    r18 = (r18 as ::core::ffi::c_int + (*E).base.x_pos as ::core::ffi::c_int) as uint16;
    let mut r20: uint16 =
        (((*((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x7f00 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int)
            - 64 as ::core::ffi::c_int) as uint16;
    r20 = (r20 as ::core::ffi::c_int + (*E).base.y_pos as ::core::ffi::c_int) as uint16;
    eproj_spawn_pt = Point16U { x: r18, y: r20 };
    SpawnEprojWithRoomGfx(0xe509 as uint16, 0x15 as uint16);
    QueueSfx2_Max6(0x29 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Draygon_Instr_26(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    NextRandom();
    let mut E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int);
    let mut x: uint16 = ((*E).base.x_pos as ::core::ffi::c_int
        + (*((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x7f as ::core::ffi::c_int)
        - 64 as ::core::ffi::c_int) as uint16;
    let mut y: uint16 = ((*E).base.y_pos as ::core::ffi::c_int
        + ((*((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x3f00 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int)
        - 32 as ::core::ffi::c_int) as uint16;
    CreateSpriteAtPos(x, y, 3 as uint16, 0 as uint16);
    QueueSfx2_Max6(0x25 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn sub_A5E9F5() {
    if *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        NextRandom();
        let mut x: uint16 = ((*((&raw mut g_ram as *mut uint8)
            .offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x3f as ::core::ffi::c_int)
            + 96 as ::core::ffi::c_int) as uint16;
        let mut y: uint16 =
            (((*((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xf00 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int)
                + 480 as ::core::ffi::c_int) as uint16;
        CreateSpriteAtPos(x, y, 21 as uint16, 0 as uint16);
    }
}
#[no_mangle]
pub unsafe extern "C" fn SporeSpawn_Init() {
    let mut v0: uint16 = 0 as uint16;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 32 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 240 as ::core::ffi::c_int) as isize) =
            *(g_rom.offset(
                (((0xa5e359 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa5e359 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        i += 2 as ::core::ffi::c_int;
    }
    SpawnEprojWithGfx(
        0 as uint16,
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        addr_kEproj_SporeSpawnsStalk as ::core::ffi::c_int as uint16,
    );
    SpawnEprojWithGfx(
        1 as uint16,
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        addr_kEproj_SporeSpawnsStalk as ::core::ffi::c_int as uint16,
    );
    SpawnEprojWithGfx(
        2 as uint16,
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        addr_kEproj_SporeSpawnsStalk as ::core::ffi::c_int as uint16,
    );
    SpawnEprojWithGfx(
        3 as uint16,
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        addr_kEproj_SporeSpawnsStalk as ::core::ffi::c_int as uint16,
    );
    let mut E: *mut Enemy_SporeSpawn = Get_SporeSpawn(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0 as ::core::ffi::c_int);
    (*E).ssn_var_04 = (*E).base.x_pos;
    (*E).ssn_var_05 = ((*E).base.y_pos as ::core::ffi::c_int - 72 as ::core::ffi::c_int) as uint16;
    (*E).ssn_var_C = (*E).base.x_pos;
    (*E).ssn_var_D = (*E).base.y_pos;
    (*E).ssn_var_F = 0 as uint16;
    if *(&raw mut g_ram as *mut uint8)
        .offset(0xd828 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E0).base.current_instruction = addr_kDraygon_Ilist_E6B9 as ::core::ffi::c_int as uint16;
        (*E0).ssn_var_A = (0xa5eb1a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E0).base.properties =
            ((*E0).base.properties as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
        SporeSpawn_Func_5();
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
            field_0: 0x7 as uint8,
            field_1: 0x1e as uint8,
            field_2: 0xb793 as VoidP,
        });
        *((&raw mut g_ram as *mut uint8).offset(0x7e9 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    } else {
        (*E0).base.current_instruction = addr_kDraygon_Ilist_E6C7 as ::core::ffi::c_int as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x183c as ::core::ffi::c_int as isize)
            as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
        let mut v5: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16);
        (*E).ssn_var_A = (0xa5eb1a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x7e9 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x909589 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.y_pos =
            ((*E).base.y_pos as ::core::ffi::c_int - 128 as ::core::ffi::c_int) as uint16;
        SpawnEprojWithGfx(
            0 as uint16,
            v5,
            addr_kEproj_SporeSpawners as ::core::ffi::c_int as uint16,
        );
        SpawnEprojWithGfx(
            1 as uint16,
            v5,
            addr_kEproj_SporeSpawners as ::core::ffi::c_int as uint16,
        );
        SpawnEprojWithGfx(
            2 as uint16,
            v5,
            addr_kEproj_SporeSpawners as ::core::ffi::c_int as uint16,
        );
        SpawnEprojWithGfx(
            3 as uint16,
            v5,
            addr_kEproj_SporeSpawners as ::core::ffi::c_int as uint16,
        );
        SporeSpawn_Func_5();
    };
}
#[no_mangle]
pub unsafe extern "C" fn CallSporeSpawnFunc(mut ea: uint32) {
    match ea {
        10873626 => return,
        10873627 => {
            SporeSpawn_Func_1();
            return;
        }
        10873682 => {
            SporeSpawn_Func_2(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
            return;
        }
        10873755 => {
            SporeSpawn_Func_3();
            return;
        }
        10873838 => {
            SporeSpawn_Func_4();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn SporeSpawn_Main() {
    let mut E: *mut Enemy_SporeSpawn = Get_SporeSpawn(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallSporeSpawnFunc(
        ((*E).ssn_var_A as ::core::ffi::c_int | 0xa50000 as ::core::ffi::c_int) as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn SporeSpawn_Func_1() {
    SporeSpawn_Func_5();
    let mut E: *mut Enemy_SporeSpawn = Get_SporeSpawn(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut v1: uint16 =
        ((*E).base.y_pos as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = v1;
    if v1 as ::core::ffi::c_int - 624 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        (*E).base.current_instruction = addr_kDraygon_Ilist_E6D5 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
    }
    let mut E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0 as ::core::ffi::c_int);
    (*E0).ssn_var_0B = 48 as uint16;
    (*E0).ssn_var_0C = 1 as uint16;
    (*E0).ssn_var_0A = 192 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn SporeSpawn_Func_2(mut k: uint16) {
    SporeSpawn_Func_5();
    let mut E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0 as ::core::ffi::c_int);
    let mut E: *mut Enemy_SporeSpawn = Get_SporeSpawn(k as ::core::ffi::c_int);
    (*E).base.x_pos = ((*E).ssn_var_C as ::core::ffi::c_int
        + CosineMult8bit((*E0).ssn_var_0A, (*E0).ssn_var_0B) as ::core::ffi::c_int)
        as uint16;
    (*E).base.y_pos = ((*E).ssn_var_D as ::core::ffi::c_int
        + SineMult8bit(
            (2 as ::core::ffi::c_int
                * ((*E0).ssn_var_0A as ::core::ffi::c_int - 64 as ::core::ffi::c_int))
                as uint16,
            ((*E0).ssn_var_0B as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int) as uint16;
    (*E0).ssn_var_0A = (*(&raw mut (*E0).ssn_var_0C as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        + (*E0).ssn_var_0A as ::core::ffi::c_int) as uint8 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn SporeSpawn_Func_3() {
    let mut E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0 as ::core::ffi::c_int);
    (*E0).ssn_var_43 = (64 as ::core::ffi::c_int
        - CalculateAngleFromXY(
            (128 as ::core::ffi::c_int - (*E0).base.x_pos as ::core::ffi::c_int) as uint16,
            (624 as ::core::ffi::c_int - (*E0).base.y_pos as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int) as uint8 as uint16;
    let mut E: *mut Enemy_SporeSpawn = Get_SporeSpawn(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut pt: Point32 = ConvertAngleToXy(
        *(&raw mut (*E0).ssn_var_43 as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as uint16,
        1 as uint16,
    );
    (*E).ssn_var_28 = (pt.x >> 16 as ::core::ffi::c_int) as uint16;
    (*E).ssn_var_29 = pt.x as uint16;
    (*E).ssn_var_2A = (pt.y >> 16 as ::core::ffi::c_int) as uint16;
    (*E).ssn_var_2B = pt.y as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn SporeSpawn_Func_4() {
    let mut E: *mut Enemy_SporeSpawn = Get_SporeSpawn(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0 as ::core::ffi::c_int);
    let mut varE20: uint16 = *(&raw mut (*E0).ssn_var_43 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as uint16;
    EnemyFunc_B691(
        varE20,
        Point32 {
            x: ((*E).ssn_var_28 as uint32) << 16 as ::core::ffi::c_int | (*E).ssn_var_29 as uint32,
            y: ((*E).ssn_var_2A as uint32) << 16 as ::core::ffi::c_int | (*E).ssn_var_2B as uint32,
        },
    );
    let mut v2: uint16 =
        Abs16(((*E0).base.x_pos as ::core::ffi::c_int - 128 as ::core::ffi::c_int) as uint16);
    if v2 as ::core::ffi::c_int - 8 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        let mut v3: uint16 =
            Abs16(((*E0).base.y_pos as ::core::ffi::c_int - 624 as ::core::ffi::c_int) as uint16);
        if v3 as ::core::ffi::c_int - 8 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E0).ssn_var_A =
                (0xa5eb1a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
    SporeSpawn_Func_5();
    sub_A5E9F5();
}
#[no_mangle]
pub unsafe extern "C" fn SporeSpawn_Func_5() {
    let mut E: *mut Enemy_SporeSpawn = Get_SporeSpawn(0 as ::core::ffi::c_int);
    let mut v1: uint16 =
        ((*E).base.x_pos as ::core::ffi::c_int - (*E).ssn_var_04 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = 0;
    let mut r18: uint16 = 0;
    let mut r20: uint16 = 0;
    let mut r22: uint16 = 0;
    if v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        r18 = (v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        r20 = (v1 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16;
        r22 = ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
            + (v1 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int)) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(14 as ::core::ffi::c_int as isize) = 128 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(15 as ::core::ffi::c_int as isize) = ((*E).ssn_var_04 as ::core::ffi::c_int
            + (v1 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int))
            as uint16;
        (*E).ssn_var_20 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(15 as ::core::ffi::c_int as isize);
        *((&raw mut g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize) =
            ((*E).ssn_var_04 as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
        (*E).ssn_var_21 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize);
        v2 = ((*E).ssn_var_04 as ::core::ffi::c_int + r22 as ::core::ffi::c_int) as uint16;
    } else {
        r18 = (((*E).ssn_var_04 as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int)
            as uint16 as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16;
        r20 = (-(v1 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 2 as ::core::ffi::c_int) as uint16;
        r22 = (r18 as ::core::ffi::c_int + r20 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(14 as ::core::ffi::c_int as isize) = 128 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(15 as ::core::ffi::c_int as isize) =
            ((*E).ssn_var_04 as ::core::ffi::c_int - r20 as ::core::ffi::c_int) as uint16;
        (*E).ssn_var_20 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(15 as ::core::ffi::c_int as isize);
        *((&raw mut g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize) =
            ((*E).ssn_var_04 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
        (*E).ssn_var_21 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize);
        v2 = ((*E).ssn_var_04 as ::core::ffi::c_int - r22 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(17 as ::core::ffi::c_int as isize) = v2;
    (*E).ssn_var_22 = v2;
    let mut v3: uint16 = ((*E).base.y_pos as ::core::ffi::c_int
        - 40 as ::core::ffi::c_int
        - (*E).ssn_var_05 as ::core::ffi::c_int) as uint16;
    let mut v4: uint16 = 0;
    if v3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        r18 = (v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        r20 = (v3 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16;
        r22 = ((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
            + (v3 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int)) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(14 as ::core::ffi::c_int as isize) = 560 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(15 as ::core::ffi::c_int as isize) = ((*E).ssn_var_05 as ::core::ffi::c_int
            + (v3 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int))
            as uint16;
        (*E).ssn_var_23 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(15 as ::core::ffi::c_int as isize);
        *((&raw mut g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize) =
            ((*E).ssn_var_05 as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
        (*E).ssn_var_24 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize);
        v4 = ((*E).ssn_var_05 as ::core::ffi::c_int + r22 as ::core::ffi::c_int) as uint16;
    } else {
        r18 = (((*E).ssn_var_05 as ::core::ffi::c_int
            - ((*E).base.y_pos as ::core::ffi::c_int - 40 as ::core::ffi::c_int))
            as uint16 as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16;
        r20 = (-(v3 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 2 as ::core::ffi::c_int) as uint16;
        r22 = (r18 as ::core::ffi::c_int + r20 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(14 as ::core::ffi::c_int as isize) = 560 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(15 as ::core::ffi::c_int as isize) =
            ((*E).ssn_var_05 as ::core::ffi::c_int - r20 as ::core::ffi::c_int) as uint16;
        (*E).ssn_var_23 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(15 as ::core::ffi::c_int as isize);
        *((&raw mut g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize) =
            ((*E).ssn_var_05 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
        (*E).ssn_var_24 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(16 as ::core::ffi::c_int as isize);
        v4 = ((*E).ssn_var_05 as ::core::ffi::c_int - r22 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(17 as ::core::ffi::c_int as isize) = v4;
    (*E).ssn_var_25 = v4;
}
#[no_mangle]
pub unsafe extern "C" fn SporeSpawn_Shot() {
    let mut v4: int16 = 0;
    let mut v0: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0x18a6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xc18 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0x18a6 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        );
    if v1 as ::core::ffi::c_int & 0x700 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        || v1 as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        EnemyFunc_A6B4_UsedBySporeSpawn();
        v0 = *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut EK: *mut Enemy_SporeSpawn = Get_SporeSpawn(v0 as ::core::ffi::c_int);
        if (*EK).base.flash_timer != 0 {
            let mut E: *mut Enemy_SporeSpawn = Get_SporeSpawn(0 as ::core::ffi::c_int);
            (*EK).ssn_var_A =
                (0xa5eb52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            v4 = 2 as int16;
            if (*EK).base.health as ::core::ffi::c_int - 400 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                if (*E).ssn_var_0C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    v4 = -(2 as ::core::ffi::c_int) as int16;
                }
                (*E).ssn_var_0C = v4 as uint16;
            }
            if (*E).ssn_var_2F == 0 {
                (*E).ssn_var_0C = -((*E).ssn_var_0C as ::core::ffi::c_int) as uint16;
                (*E).ssn_var_2F = 1 as uint16;
                (*EK).base.current_instruction = addr_stru_A5E729 as ::core::ffi::c_int as uint16;
                (*EK).base.instruction_timer = 1 as uint16;
                let mut v7: uint16 = 96 as uint16;
                let mut health: uint16 = (*EK).base.health;
                if health as ::core::ffi::c_int - 70 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    v7 = 64 as uint16;
                    if health as ::core::ffi::c_int - 410 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0
                    {
                        v7 = 32 as uint16;
                        if health as ::core::ffi::c_int - 770 as ::core::ffi::c_int
                            & 0x8000 as ::core::ffi::c_int
                            == 0
                        {
                            v7 = 0 as uint16;
                        }
                    }
                }
                if health as ::core::ffi::c_int != (*E).ssn_var_40 as ::core::ffi::c_int {
                    (*E).ssn_var_40 = health;
                    SporeSpawn_Func_7(v7);
                }
            }
        }
        SporeSpawn_Func_6();
    }
}
#[no_mangle]
pub unsafe extern "C" fn SporeSpawn_Touch() {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy();
    SporeSpawn_Func_6();
}
#[no_mangle]
pub unsafe extern "C" fn SporeSpawn_Func_6() {
    if (*Get_SporeSpawn(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .health
        == 0
    {
        let mut E: *mut Enemy_SporeSpawn = Get_SporeSpawn(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        );
        let mut E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0 as ::core::ffi::c_int);
        (*E0).ssn_var_0E = 0 as uint16;
        (*E).base.invincibility_timer = 0 as uint16;
        (*E).base.flash_timer = 0 as uint16;
        (*E).base.ai_handler_bits = 0 as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
        let mut i: ::core::ffi::c_int = 26 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *(((&raw mut g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
        (*E0).base.current_instruction = addr_kDraygon_Ilist_E77D as ::core::ffi::c_int as uint16;
        (*E0).base.instruction_timer = 1 as uint16;
        let ref mut fresh5 = *((&raw mut g_ram as *mut uint8)
            .offset(0xd828 as ::core::ffi::c_int as isize)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as *mut uint8 as *mut uint16);
        *fresh5 = (*fresh5 as ::core::ffi::c_int | 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x7e9 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
            field_0: 0x7 as uint8,
            field_1: 0x1e as uint8,
            field_2: 0xb78f as VoidP,
        });
    }
}
#[no_mangle]
pub unsafe extern "C" fn SporeSpawn_Func_7(mut a: uint16) {
    let mut v1: uint16 = a;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 32 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 144 as ::core::ffi::c_int) as isize) =
            *(g_rom.offset(
                (((0xa5e379 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa5e379 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        i += 2 as ::core::ffi::c_int;
    }
}
