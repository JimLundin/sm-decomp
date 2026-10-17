use crate::game::sm_80::{NextRandom, QueueSfx2_Max6};
use crate::game::sm_86::{
    eproj_spawn_pt, eproj_spawn_r22, eproj_spawn_rect, SpawnEprojWithGfx, SpawnEprojWithRoomGfx,
};
use crate::game::sm_90::ProjectileReflection;
use crate::game::sm_a0::{
    CosineMult8bit, EnemyDeathAnimation, EnemyFunc_BBBF, Enemy_ItemDrop_LowerNorfairSpacePirate,
    Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes, Enemy_SwitchToFrozenAi, IsSamusWithinEnemy_Y,
    NormalEnemyFrozenAI, NormalEnemyPowerBombAi, NormalEnemyShotAi,
    NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAi, SineMult8bit,
};
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::rtl::{g_ram, g_rom, RomPtr, Unreachable};

pub use crate::types::{
    __int16_t, __int32_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32, int32_t,
    uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t, EnemyBase, EnemyData, LongPtr,
    Point16U, ProjectileType, Rect16U, VoidP,
};
pub const kProjectileType_DontInteractWithSamus: ProjectileType = 32768;
pub const kProjectileType_TypeMask: ProjectileType = 3840;
pub const kProjectileType_MissileExplosion: ProjectileType = 2048;
pub const kProjectileType_BeamExplosion: ProjectileType = 1792;
pub const kProjectileType_Bomb: ProjectileType = 1280;
pub const kProjectileType_PowerBomb: ProjectileType = 768;
pub const kProjectileType_SuperMissile: ProjectileType = 512;
pub const kProjectileType_Missile: ProjectileType = 256;
pub use crate::types::{Consts_86, EnemyDef, Vulnerability};
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
pub use crate::types::Consts_A0;
pub const addr_kEnemyDef_F593: Consts_A0 = 62867;
pub const addr_kEnemyDef_F353: Consts_A0 = 62291;
pub const addr_kEnemyDef_F293: Consts_A0 = 62099;
pub const addr_kEnemyDef_EEFF: Consts_A0 = 61183;
pub const addr_kEnemyDef_EC3F: Consts_A0 = 60479;
pub const addr_kEnemyDef_E83F: Consts_A0 = 59455;
pub const addr_kEnemyDef_E7FF: Consts_A0 = 59391;
pub const addr_kEnemyDef_E4FF: Consts_A0 = 58623;
pub const addr_kEnemyDef_E4BF: Consts_A0 = 58559;
pub const addr_kEnemyDef_E2BF: Consts_A0 = 58047;
pub const addr_kEnemyDef_E17F: Consts_A0 = 57727;
pub const addr_kEnemyDef_E0FF: Consts_A0 = 57599;
pub const addr_kEnemyDef_DF7F: Consts_A0 = 57215;
pub const addr_kEnemyDef_DF3F: Consts_A0 = 57151;
pub const addr_kEnemyDef_DE7F: Consts_A0 = 56959;
pub const addr_kEnemyDef_DE3F: Consts_A0 = 56895;
pub const addr_kEnemyDef_DDBF: Consts_A0 = 56767;
pub const addr_kEnemyDef_DD7F: Consts_A0 = 56703;
pub const addr_kEnemyDef_DAFF: Consts_A0 = 56063;
pub const addr_kExtendedSpritemap_Nothing_A0: Consts_A0 = 32847;
pub const addr_kSpritemap_Nothing_A0: Consts_A0 = 32845;
pub use crate::types::Consts_B2;
pub const addr_kSpacePirates_Ilist_FC48: Consts_B2 = 64584;
pub const addr_kSpacePirates_Ilist_FC0E: Consts_B2 = 64526;
pub const addr_kSpacePirates_Ilist_FBE6: Consts_B2 = 64486;
pub const addr_kSpacePirates_Ilist_FBC6: Consts_B2 = 64454;
pub const addr_kSpacePirates_Ilist_FB8C: Consts_B2 = 64396;
pub const addr_kSpacePirates_Ilist_FB64: Consts_B2 = 64356;
pub const addr_kSpacePirates_Ilist_FB58: Consts_B2 = 64344;
pub const addr_kSpacePirates_Ilist_FB4C: Consts_B2 = 64332;
pub const addr_kSpacePirates_Ilist_F51A: Consts_B2 = 62746;
pub const addr_kSpacePirates_Ilist_F4EA: Consts_B2 = 62698;
pub const addr_kSpacePirates_Ilist_F4CC: Consts_B2 = 62668;
pub const addr_kSpacePirates_Ilist_F4A4: Consts_B2 = 62628;
pub const addr_kSpacePirates_Ilist_F492: Consts_B2 = 62610;
pub const addr_kSpacePirates_Ilist_F462: Consts_B2 = 62562;
pub const addr_kSpacePirates_Ilist_F420: Consts_B2 = 62496;
pub const addr_kSpacePirates_Ilist_F3B2: Consts_B2 = 62386;
pub const addr_kSpacePirates_Ilist_F34A: Consts_B2 = 62282;
pub const addr_kSpacePirates_Ilist_F32E: Consts_B2 = 62254;
pub const addr_kSpacePirates_Ilist_F2F8: Consts_B2 = 62200;
pub const addr_kSpacePirates_Ilist_F2DA: Consts_B2 = 62170;
pub const addr_kSpacePirates_Ilist_F2B2: Consts_B2 = 62130;
pub const addr_kSpacePirates_Ilist_F2A0: Consts_B2 = 62112;
pub const addr_kSpacePirates_Ilist_F270: Consts_B2 = 62064;
pub const addr_kSpacePirates_Ilist_F22E: Consts_B2 = 61998;
pub const addr_kSpacePirates_Ilist_F1C4: Consts_B2 = 61892;
pub const addr_kSpacePirates_Ilist_F15C: Consts_B2 = 61788;
pub const addr_kSpacePirates_Ilist_EDF6: Consts_B2 = 60918;
pub const addr_off_B2EDAC: Consts_B2 = 60844;
pub const addr_kSpacePirates_Ilist_EDA4: Consts_B2 = 60836;
pub const addr_kSpacePirates_Ilist_ED80: Consts_B2 = 60800;
pub const addr_kSpacePirates_Ilist_ED36: Consts_B2 = 60726;
pub const addr_off_B2ECEC: Consts_B2 = 60652;
pub const addr_kSpacePirates_Ilist_ECE4: Consts_B2 = 60644;
pub const addr_kSpacePirates_Ilist_ECC0: Consts_B2 = 60608;
pub use crate::types::Consts_B4;
pub const addr_stru_B4EC1C: Consts_B4 = 60444;
pub const addr_asc_B4DD89: Consts_B4 = 56713;
pub const addr_kSpriteObject_Ilist_C4B6: Consts_B4 = 50358;
pub const addr_kSpriteObject_Ilist_C3BA: Consts_B4 = 50106;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_SpacePirates {
    pub base: EnemyBase,
    pub sps_var_A: uint16,
    pub sps_var_B: uint16,
    pub sps_var_C: uint16,
    pub sps_var_D: uint16,
    pub sps_var_E: uint16,
    pub sps_var_F: uint16,
    pub sps_parameter_1: uint16,
    pub sps_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub sps_var_00: uint16,
    pub sps_var_01: uint16,
    pub sps_var_02: uint16,
    pub sps_var_03: uint16,
    pub sps_var_04: uint16,
    pub sps_var_05: uint16,
    pub sps_var_06: uint16,
    pub sps_var_07: uint16,
    pub sps_var_08: uint16,
    pub sps_var_09: uint16,
    pub sps_var_0A: uint16,
    pub sps_var_0B: uint16,
    pub sps_var_0C: uint16,
    pub sps_var_0D: uint16,
    pub sps_var_0E: uint16,
    pub sps_var_0F: uint16,
    pub sps_var_10: uint16,
    pub sps_var_11: uint16,
    pub sps_var_12: uint16,
    pub sps_var_13: uint16,
    pub sps_var_14: uint16,
    pub sps_var_15: uint16,
    pub sps_var_16: uint16,
    pub sps_var_17: uint16,
    pub sps_var_18: uint16,
    pub sps_var_19: uint16,
    pub sps_var_1A: uint16,
    pub sps_var_1B: uint16,
    pub sps_var_1C: uint16,
    pub sps_var_1D: uint16,
    pub sps_var_1E: uint16,
    pub sps_var_1F: uint16,
    pub pad2: [uint8; 1984],
    pub sps_var_20: uint16,
    pub sps_var_21: uint16,
    pub sps_var_22: uint16,
    pub sps_var_23: uint16,
    pub sps_var_24: uint16,
    pub sps_var_25: uint16,
    pub sps_var_26: uint16,
    pub sps_var_27: uint16,
    pub sps_var_28: uint16,
    pub sps_var_29: uint16,
    pub sps_var_2A: uint16,
    pub sps_var_2B: uint16,
    pub sps_var_2C: uint16,
    pub sps_var_2D: uint16,
    pub sps_var_2E: uint16,
    pub sps_var_2F: uint16,
    pub sps_var_30: uint16,
    pub sps_var_31: uint16,
    pub sps_var_32: uint16,
    pub sps_var_33: uint16,
    pub sps_var_34: uint16,
    pub sps_var_35: uint16,
    pub sps_var_36: uint16,
    pub sps_var_37: uint16,
    pub sps_var_38: uint16,
    pub sps_var_39: uint16,
    pub sps_var_3A: uint16,
    pub sps_var_3B: uint16,
    pub sps_var_3C: uint16,
    pub sps_var_3D: uint16,
    pub sps_var_3E: uint16,
    pub sps_var_3F: uint16,
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
unsafe fn RomPtr_A0(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0xa00000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
#[inline]
unsafe fn get_EnemyDef_A2(mut a: uint16) -> *mut EnemyDef {
    return RomPtr((0xa00000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t)
        as *mut EnemyDef;
}
#[inline]
unsafe fn get_Vulnerability(mut a: uint16) -> *mut Vulnerability {
    return RomPtr((0xb40000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t)
        as *mut Vulnerability;
}
pub const fnnullsub_169_B2: ::core::ffi::c_int = 0xb2804b as ::core::ffi::c_int;
pub const fnSpacePirates_Func_1: ::core::ffi::c_int = 0xb2f034 as ::core::ffi::c_int;
pub const fnnullsub_279: uint32 = 11726927 as uint32;
pub const fnSpacePirates_Func_2: uint32 = 11726928 as uint32;
pub const fnSpacePirates_Func_3: ::core::ffi::c_int = 0xb2f0c8 as ::core::ffi::c_int;
pub const fnnullsub_281: uint32 = 11727075 as uint32;
pub const fnSpacePirates_Func_4: uint32 = 11727076 as uint32;
pub const fnSpacePirates_Func_5: uint32 = 11728553 as uint32;
pub const fnSpacePirates_Func_6: uint32 = 11728612 as uint32;
pub const fnSpacePirates_Func_11: uint32 = 11728919 as uint32;
pub const fnSpacePirates_Func_12: ::core::ffi::c_int = 0xb2f84c as ::core::ffi::c_int;
pub const fnSpacePirates_Func_13: uint32 = 11729040 as uint32;
pub const fnSpacePirates_Func_14: ::core::ffi::c_int = 0xb2f8c5 as ::core::ffi::c_int;
pub const fnSpacePirates_Func_15: uint32 = 11729161 as uint32;
pub const fnSpacePirates_F985: uint32 = 11729285 as uint32;
pub const fnSpacePirates_F9C1: ::core::ffi::c_int = 0xb2f9c1 as ::core::ffi::c_int;
pub const fnSpacePirates_FA15: ::core::ffi::c_int = 0xb2fa15 as ::core::ffi::c_int;
pub const fnSpacePirates_FA59: uint32 = 11729497 as uint32;
pub const fnSpacePirates_FA95: ::core::ffi::c_int = 0xb2fa95 as ::core::ffi::c_int;
pub const fnSpacePirates_FAE9: ::core::ffi::c_int = 0xb2fae9 as ::core::ffi::c_int;
pub const fnWalkingSpacePirates_FD44: uint32 = 11730244 as uint32;
pub const fnWalkingSpacePirates_FDCE: uint32 = 11730382 as uint32;
pub const fnnullsub_282: uint32 = 11730506 as uint32;
#[inline]
unsafe fn gEnemyData(mut o: ::core::ffi::c_int) -> *mut EnemyData {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut EnemyData;
}
#[inline]
unsafe fn Get_SpacePirates(mut o: ::core::ffi::c_int) -> *mut Enemy_SpacePirates {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_SpacePirates;
}
pub unsafe fn Enemy_GrappleReact_CancelBeam_B2() {
    Enemy_SwitchToFrozenAi();
}
pub unsafe fn Enemy_NormalFrozenAI_B2() {
    NormalEnemyFrozenAI();
}
pub unsafe fn WalkingSpacePirates_Powerbomb() {
    NormalEnemyPowerBombAi();
}
pub unsafe fn WalkingSpacePirates_Touch() {
    if (*Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .frozen_timer
        == 0
    {
        NormalEnemyTouchAi();
    }
}
pub unsafe fn WalkingSpacePirates_Shot() {
    if (*Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .enemy_ptr as ::core::ffi::c_int
        == addr_kEnemyDef_F593 as ::core::ffi::c_int
    {
        WalkingSpacePirates_8789();
    } else {
        NormalEnemyShotAi();
    };
}
pub unsafe fn WalkingSpacePirates_8789() {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    *((&raw mut g_ram as *mut uint8).offset(0xf434 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*E).base.x_pos;
    *((&raw mut g_ram as *mut uint8).offset(0xf436 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*E).base.y_pos;
    NormalEnemyShotAiSkipDeathAnim_CurEnemy();
    if (*E).base.health == 0 {
        let mut v1: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16);
        if (*E).base.enemy_ptr as ::core::ffi::c_int == addr_kEnemyDef_F593 as ::core::ffi::c_int {
            (*E).sps_var_B = 0 as uint16;
            EnemyDeathAnimation(v1, 4 as uint16);
            Enemy_ItemDrop_LowerNorfairSpacePirate(v1);
        } else {
            (*E).sps_var_B = 0 as uint16;
            EnemyDeathAnimation(v1, 4 as uint16);
        }
    }
}
pub unsafe fn WalkingSpacePirates_87C8() {
    let mut r18: uint16 = 0;
    let mut enemy_ptr: uint16 = 0;
    let mut vulnerability_ptr: uint16 = 0;
    let mut r20: uint16 = 0;
    let mut current_block: u64;
    let mut v2: *mut Vulnerability = ::core::ptr::null_mut::<Vulnerability>();
    let mut v3: int16 = 0;
    let mut v: *mut Vulnerability = ::core::ptr::null_mut::<Vulnerability>();
    if !((*Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .enemy_ptr as ::core::ffi::c_int
        != addr_kEnemyDef_F593 as ::core::ffi::c_int)
    {
        r18 = *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0x18a6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            );
        if (r18 as ::core::ffi::c_int & kProjectileType_TypeMask as ::core::ffi::c_int)
            - kProjectileType_PowerBomb as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            enemy_ptr = 0;
            vulnerability_ptr = 0;
            enemy_ptr = (*Get_SpacePirates(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int,
            ))
            .base
            .enemy_ptr;
            vulnerability_ptr = (*get_EnemyDef_A2(enemy_ptr)).vulnerability_ptr as uint16;
            if vulnerability_ptr == 0 {
                vulnerability_ptr = addr_stru_B4EC1C as ::core::ffi::c_int as uint16;
            }
            r20 = vulnerability_ptr;
            if r18 as ::core::ffi::c_int & kProjectileType_TypeMask as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                if r18 as ::core::ffi::c_int & kProjectileType_TypeMask as ::core::ffi::c_int
                    != kProjectileType_Missile as ::core::ffi::c_int
                    && r18 as ::core::ffi::c_int & kProjectileType_TypeMask as ::core::ffi::c_int
                        != kProjectileType_SuperMissile as ::core::ffi::c_int
                {
                    current_block = 1528265894032410192;
                } else {
                    v3 = ((r18 as ::core::ffi::c_int
                        & kProjectileType_TypeMask as ::core::ffi::c_int)
                        as uint16 as ::core::ffi::c_int
                        >> 8 as ::core::ffi::c_int) as int16;
                    v = get_Vulnerability(
                        (r20 as ::core::ffi::c_int + v3 as ::core::ffi::c_int) as uint16,
                    );
                    if (*v).plasma_ice_wave as ::core::ffi::c_int & 0xf as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                        && (*v).plasma_ice_wave as ::core::ffi::c_int & 0xf as ::core::ffi::c_int
                            != 15 as ::core::ffi::c_int
                    {
                        current_block = 1528265894032410192;
                    } else {
                        current_block = 17407779659766490442;
                    }
                }
            } else {
                v2 = get_Vulnerability(
                    (r20 as ::core::ffi::c_int
                        + (r18 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int))
                        as uint16,
                );
                if (*v2).power as ::core::ffi::c_int & 0xf as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                    && (*v2).power as ::core::ffi::c_int & 0xf as ::core::ffi::c_int
                        != 15 as ::core::ffi::c_int
                {
                    current_block = 1528265894032410192;
                } else {
                    current_block = 17407779659766490442;
                }
            }
            match current_block {
                1528265894032410192 => {}
                _ => {
                    WalkingSpacePirates_883E();
                    current_block = 15976848397966268834;
                }
            }
        } else {
            current_block = 15976848397966268834;
        }
        match current_block {
            1528265894032410192 => {}
            _ => {
                return;
            }
        }
    }
    WalkingSpacePirates_8789();
}
pub unsafe fn WalkingSpacePirates_883E() {
    if (*Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .enemy_ptr as ::core::ffi::c_int
        != addr_kEnemyDef_F593 as ::core::ffi::c_int
    {
        WalkingSpacePirates_8789();
        return;
    }
    let mut v0: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0x18a6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut v1: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x18a6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
    let mut r18: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xc18 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    if r18 as ::core::ffi::c_int & kProjectileType_TypeMask as ::core::ffi::c_int
        == kProjectileType_SuperMissile as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize)
            == 0
        {
            return;
        }
    } else if (r18 as ::core::ffi::c_int & kProjectileType_TypeMask as ::core::ffi::c_int)
        - kProjectileType_PowerBomb as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        return;
    }
    (*Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .invincibility_timer = 10 as uint16;
    let mut v2: uint16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 7 as ::core::ffi::c_int
    {
        v2 = 1 as uint16;
    } else if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 2 as ::core::ffi::c_int
    {
        v2 = 8 as uint16;
    } else {
        v2 = 5 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = v2;
    ProjectileReflection(v0);
    QueueSfx2_Max6(0x66 as uint16);
}
pub unsafe fn SpacePirates_Instr_MovePixelsDownAndChangeDirFaceRight(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    if Enemy_MoveDown(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (*jp.offset(0 as ::core::ffi::c_int as isize) as int16 as int32)
            << 16 as ::core::ffi::c_int,
    ) == 0
    {
        return jp.offset(1 as ::core::ffi::c_int as isize);
    }
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).sps_var_C = ((*E).sps_var_C as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
    if (*E).sps_var_C != 0 {
        return addr_off_B2ECEC as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return addr_kSpacePirates_Ilist_ED36 as ::core::ffi::c_int as uintptr_t as *const uint16;
}
pub unsafe fn SpacePirates_Instr_MovePixelsDownAndChangeDirFaceLeft(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    if Enemy_MoveDown(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (*jp.offset(0 as ::core::ffi::c_int as isize) as int16 as int32)
            << 16 as ::core::ffi::c_int,
    ) == 0
    {
        return jp.offset(1 as ::core::ffi::c_int as isize);
    }
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).sps_var_C = ((*E).sps_var_C as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
    if (*E).sps_var_C != 0 {
        return addr_kSpacePirates_Ilist_EDF6 as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return addr_off_B2EDAC as ::core::ffi::c_int as uintptr_t as *const uint16;
}
pub unsafe fn SpacePirates_Instr_RandomNewDirFaceR(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    let mut result: uint16 = addr_kSpacePirates_Ilist_ED36 as ::core::ffi::c_int as uint16;
    let mut v4: uint16 = (NextRandom() as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint16;
    (*Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .sps_var_C = v4;
    if v4 != 0 {
        return addr_off_B2ECEC as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return result as uintptr_t as *const uint16;
}
pub unsafe fn SpacePirates_Instr_RandomNewDirFaceL(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    let mut result: uint16 = addr_off_B2EDAC as ::core::ffi::c_int as uint16;
    let mut v4: uint16 = (NextRandom() as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint16;
    (*Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .sps_var_C = v4;
    if v4 != 0 {
        return addr_kSpacePirates_Ilist_EDF6 as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return result as uintptr_t as *const uint16;
}
pub unsafe fn SpacePirates_Instr_PrepareWallJumpR(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).sps_var_B = ((*E).base.x_pos as ::core::ffi::c_int
        + (*E).sps_parameter_2 as ::core::ffi::c_int) as uint16;
    (*E).sps_var_D = ((*E).base.x_pos as ::core::ffi::c_int
        + ((*E).sps_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int))
        as uint16;
    (*E).sps_var_E = (*E).base.y_pos;
    (*E).sps_var_F = 64 as uint16;
    return jp;
}
pub unsafe fn SpacePirates_Instr_PrepareWallJumpL(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).sps_var_B = ((*E).base.x_pos as ::core::ffi::c_int
        - (*E).sps_parameter_2 as ::core::ffi::c_int) as uint16;
    (*E).sps_var_D = ((*E).base.x_pos as ::core::ffi::c_int
        - ((*E).sps_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int))
        as uint16;
    (*E).sps_var_E = (*E).base.y_pos;
    (*E).sps_var_F = 192 as uint16;
    return jp;
}
pub unsafe fn SpacePirates_Instr_FireLaserL(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    *((&raw mut g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) = *(RomPtr_A0((*E).base.enemy_ptr as uint16_t) as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize);
    eproj_spawn_pt = Point16U {
        x: ((*E).base.x_pos as ::core::ffi::c_int - 24 as ::core::ffi::c_int) as uint16,
        y: ((*E).base.y_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16,
    };
    eproj_spawn_r22 = 0 as uint16;
    SpawnEprojWithRoomGfx(
        addr_stru_86A17B as ::core::ffi::c_int as uint16,
        0 as uint16,
    );
    return jp;
}
pub unsafe fn SpacePirates_Instr_FireLaserR(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    eproj_spawn_pt = Point16U {
        x: ((*E).base.x_pos as ::core::ffi::c_int + 24 as ::core::ffi::c_int) as uint16,
        y: ((*E).base.y_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16,
    };
    eproj_spawn_r22 = 1 as uint16;
    SpawnEprojWithRoomGfx(
        addr_stru_86A17B as ::core::ffi::c_int as uint16,
        1 as uint16,
    );
    return jp;
}
pub unsafe fn SpacePirates_Instr_SetEnemyFunc(
    mut k: uint16,
    mut jp: *const uint16,
) -> *const uint16 {
    (*Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .sps_var_A = *jp.offset(0 as ::core::ffi::c_int as isize);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn SpacePirates_Instr_PlaySfx(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max6(0x66 as uint16);
    return jp;
}
pub unsafe fn WallSpacePirates_Init() {
    let mut v0: uint16 = addr_kSpacePirates_Ilist_ED36 as ::core::ffi::c_int as uint16;
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).sps_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v0 = addr_off_B2EDAC as ::core::ffi::c_int as uint16;
    }
    (*E).base.current_instruction = v0;
    (*E).sps_var_20 = 190 as uint16;
    (*E).sps_var_21 = 66 as uint16;
    (*E).sps_var_22 = 2 as uint16;
    if (*E).sps_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).sps_var_20 =
            ((*E).sps_var_20 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        (*E).sps_var_21 =
            ((*E).sps_var_21 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        (*E).sps_var_22 =
            ((*E).sps_var_22 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
    let mut v2: uint16 = (0xb2f034 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if (*E).sps_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v2 = (0xb2f0c8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    (*E).sps_var_A = v2;
    let mut v3: uint16 = 0;
    if ((*E).base.x_pos as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
        - 11 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        v3 = ((*E).base.x_pos as ::core::ffi::c_int & 0xfff8 as ::core::ffi::c_int) as uint16;
    } else {
        v3 = (((*E).base.x_pos as ::core::ffi::c_int & 0xfff0 as ::core::ffi::c_int)
            + 16 as ::core::ffi::c_int) as uint16;
    }
    (*E).base.x_pos = v3;
}
pub unsafe fn CallSpacePiratesEnemyFunc(mut ea: uint32, mut k: uint16) {
    match ea {
        11698251 => return,
        11726900 => {
            SpacePirates_Func_1(k);
            return;
        }
        11726927 => return,
        11726928 => {
            SpacePirates_Func_2(k);
            return;
        }
        11727048 => {
            SpacePirates_Func_3(k);
            return;
        }
        11727075 => return,
        11730506 => return,
        11727076 => {
            SpacePirates_Func_4(k);
            return;
        }
        11728553 => {
            SpacePirates_Func_5(k);
            return;
        }
        11728612 => {
            SpacePirates_Func_6(k);
            return;
        }
        11728919 => {
            SpacePirates_Func_11(k);
            return;
        }
        11728972 => {
            SpacePirates_Func_12(k);
            return;
        }
        11729040 => {
            SpacePirates_Func_13(k);
            return;
        }
        11729093 => {
            SpacePirates_Func_14(k);
            return;
        }
        11729161 => {
            SpacePirates_Func_15(k);
            return;
        }
        11729285 => {
            SpacePirates_F985(k);
            return;
        }
        11729345 => {
            SpacePirates_F9C1(k);
            return;
        }
        11729429 => {
            SpacePirates_FA15(k);
            return;
        }
        11729497 => {
            SpacePirates_FA59(k);
            return;
        }
        11729557 => {
            SpacePirates_FA95(k);
            return;
        }
        11729641 => {
            SpacePirates_FAE9(k);
            return;
        }
        11730244 => {
            WalkingSpacePirates_FD44();
            return;
        }
        11730382 => {
            WalkingSpacePirates_FDCE();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
pub unsafe fn WallSpacePirates_Main() {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallSpacePiratesEnemyFunc(
        ((*v0).ai_var_A as ::core::ffi::c_int | 0xb20000 as ::core::ffi::c_int) as uint32,
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
pub unsafe fn SpacePirates_Func_1(mut k: uint16) {
    if IsSamusWithinEnemy_Y(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        0x20 as uint16,
    ) != 0
    {
        let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        );
        (*E).base.current_instruction =
            addr_kSpacePirates_Ilist_ED80 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
    }
}
pub unsafe fn SpacePirates_Func_2(mut k: uint16) {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.x_pos = ((*E).sps_var_D as ::core::ffi::c_int
        + SineMult8bit(
            (*E).sps_var_F,
            ((*E).sps_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E).sps_var_E as ::core::ffi::c_int
        - CosineMult8bit(
            (*E).sps_var_F,
            ((*E).sps_parameter_2 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 = (*(&raw mut (*E).sps_var_F as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        - *(&raw mut (*E).sps_var_22 as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int) as uint8 as uint16;
    (*E).sps_var_F = v1;
    if v1 as ::core::ffi::c_int == (*E).sps_var_20 as ::core::ffi::c_int {
        (*E).base.current_instruction =
            addr_kSpacePirates_Ilist_EDA4 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        let mut v2: uint16 = 0;
        if ((*E).base.x_pos as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
            - 11 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v2 = ((*E).base.x_pos as ::core::ffi::c_int & 0xfff8 as ::core::ffi::c_int) as uint16;
        } else {
            v2 = (((*E).base.x_pos as ::core::ffi::c_int & 0xfff0 as ::core::ffi::c_int)
                + 16 as ::core::ffi::c_int) as uint16;
        }
        (*E).base.x_pos = v2;
    }
}
pub unsafe fn SpacePirates_Func_3(mut k: uint16) {
    if IsSamusWithinEnemy_Y(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        0x20 as uint16,
    ) != 0
    {
        let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        );
        (*E).base.current_instruction =
            addr_kSpacePirates_Ilist_ECC0 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
    }
}
pub unsafe fn SpacePirates_Func_4(mut k: uint16) {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.x_pos = ((*E).sps_var_D as ::core::ffi::c_int
        + SineMult8bit(
            (*E).sps_var_F,
            ((*E).sps_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E).sps_var_E as ::core::ffi::c_int
        - CosineMult8bit(
            (*E).sps_var_F,
            ((*E).sps_parameter_2 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 = (*(&raw mut (*E).sps_var_22 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        + *(&raw mut (*E).sps_var_F as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int) as uint8 as uint16;
    (*E).sps_var_F = v1;
    if v1 as ::core::ffi::c_int == (*E).sps_var_21 as ::core::ffi::c_int {
        (*E).base.current_instruction =
            addr_kSpacePirates_Ilist_ECE4 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        let mut v2: uint16 = 0;
        if ((*E).base.x_pos as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
            - 11 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v2 = ((*E).base.x_pos as ::core::ffi::c_int & 0xfff8 as ::core::ffi::c_int) as uint16;
        } else {
            v2 = (((*E).base.x_pos as ::core::ffi::c_int & 0xfff0 as ::core::ffi::c_int)
                + 16 as ::core::ffi::c_int) as uint16;
        }
        (*E).base.x_pos = v2;
    }
}
pub unsafe fn SpacePirates_Instr_20(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .base
    .palette_index = *jp.offset(0 as ::core::ffi::c_int as isize);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn SpacePirates_Instr_16(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max6(*jp.offset(0 as ::core::ffi::c_int as isize));
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn sub_B2F554() {
    (*gEnemyData(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .instruction_timer = 1 as uint16;
}
pub unsafe fn SpacePirates_Instr_15(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    eproj_spawn_rect = Rect16U {
        x: (*E).base.x_pos,
        y: (*E).base.y_pos,
        w: *jp.offset(1 as ::core::ffi::c_int as isize),
        h: *jp.offset(2 as ::core::ffi::c_int as isize),
    };
    SpawnEprojWithGfx(
        *jp.offset(0 as ::core::ffi::c_int as isize),
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        addr_kEproj_PirateClaw as ::core::ffi::c_int as uint16,
    );
    return jp.offset(3 as ::core::ffi::c_int as isize);
}
pub unsafe fn SpacePirates_Instr_18(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.instruction_timer = 1 as uint16;
    (*E).sps_var_C = (if (*E).base.x_pos as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        addr_kSpacePirates_Ilist_F420 as ::core::ffi::c_int
    } else {
        addr_kSpacePirates_Ilist_F22E as ::core::ffi::c_int
    }) as uint16;
    return (*E).sps_var_C as uintptr_t as *const uint16;
}
pub unsafe fn sub_B2F5B3() {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v0).instruction_timer = 1 as uint16;
    (*v0).ai_var_C = (if (*v0).x_pos as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        addr_kSpacePirates_Ilist_F51A as ::core::ffi::c_int
    } else {
        addr_kSpacePirates_Ilist_F32E as ::core::ffi::c_int
    }) as uint16;
}
pub unsafe fn SpacePirates_Instr_17(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_SpacePirates(k as ::core::ffi::c_int)).sps_var_00 = 0 as uint16;
    return jp;
}
pub unsafe fn NinjaSpacePirates_Init() {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).base.current_instruction = (if (*E).sps_parameter_1 as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        addr_kSpacePirates_Ilist_F4CC as ::core::ffi::c_int
    } else {
        addr_kSpacePirates_Ilist_F2DA as ::core::ffi::c_int
    }) as uint16;
    (*E).sps_var_C = (*E).base.current_instruction;
    let mut x_pos: uint16 = (*E).base.x_pos;
    if (*E).sps_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).sps_var_E = x_pos;
        (*E).sps_var_F =
            ((*E).sps_parameter_2 as ::core::ffi::c_int + x_pos as ::core::ffi::c_int) as uint16;
    } else {
        (*E).sps_var_F = x_pos;
        (*E).sps_var_E =
            (x_pos as ::core::ffi::c_int - (*E).sps_parameter_2 as ::core::ffi::c_int) as uint16;
    }
    let mut r20: uint16 = (((*E).sps_var_F as ::core::ffi::c_int
        - (*E).sps_var_E as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int) as uint16;
    (*E).sps_var_D = ((*E).sps_var_E as ::core::ffi::c_int + r20 as ::core::ffi::c_int) as uint16;
    let mut r18: uint16 = 0 as uint16;
    let mut R22: uint16 = 0 as uint16;
    r20 = ((r20 as uint8 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16;
    loop {
        r18 = (r18 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
        R22 = (R22 as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
        if !(R22 as ::core::ffi::c_int - r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0)
        {
            break;
        }
    }
    (*E).sps_var_B = r18;
    let mut R24: uint16 = ((R22 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int) as uint16;
    (*E).sps_var_F = ((*E).sps_var_D as ::core::ffi::c_int + R24 as ::core::ffi::c_int) as uint16;
    (*E).sps_var_E = ((*E).sps_var_D as ::core::ffi::c_int - R24 as ::core::ffi::c_int) as uint16;
    let mut sps_var_E: uint16 = (*E).sps_var_E;
    if (*E).sps_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        sps_var_E = (*E).sps_var_F;
    }
    (*E).base.x_pos = sps_var_E;
    (*E).sps_var_A = (0xb2804b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).sps_var_08 = (*E).base.y_pos;
    let mut v6: uint16 = 0 as uint16;
    let mut v7: uint16 = 0 as uint16;
    let mut n: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                ((v7 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 240 as ::core::ffi::c_int)
                    as isize,
            ) = *(g_rom.offset(
            (((0xb28727 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xb28727 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v6 = (v6 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v7 = (v7 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        n -= 1;
        if !(n >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
pub unsafe fn NinjaSpacePirates_Main() {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallSpacePiratesEnemyFunc(
        ((*E).sps_var_A as ::core::ffi::c_int | 0xb20000 as ::core::ffi::c_int) as uint32,
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
pub unsafe fn SpacePirates_Func_5(mut k: uint16) {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (abs16(
        ((*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
    ) as ::core::ffi::c_int
        - 128 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        SpacePirates_Func_8();
    } else {
        (*E).base.current_instruction = (if (*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            addr_kSpacePirates_Ilist_F420 as ::core::ffi::c_int
        } else {
            addr_kSpacePirates_Ilist_F22E as ::core::ffi::c_int
        }) as uint16;
        (*E).sps_var_C = (*E).base.current_instruction;
        (*E).base.instruction_timer = 1 as uint16;
    };
}
pub unsafe fn SpacePirates_Func_6(mut k: uint16) {
    if SpacePirates_Func_8() == 0 && SpacePirates_Func_10() == 0 && SpacePirates_Func_9() == 0 {
        SpacePirates_Func_7(k);
    }
}
pub unsafe fn SpacePirates_Func_7(mut k: uint16) {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int);
    if (*E).base.frame_counter as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if (*E).base.x_pos as ::core::ffi::c_int == (*E).sps_var_E as ::core::ffi::c_int {
            if (((*E).base.x_pos as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                return;
            }
            (*E).base.current_instruction =
                addr_kSpacePirates_Ilist_F15C as ::core::ffi::c_int as uint16;
        } else {
            if ((*E).base.x_pos as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                return;
            }
            (*E).base.current_instruction =
                addr_kSpacePirates_Ilist_F34A as ::core::ffi::c_int as uint16;
        }
        (*E).base.instruction_timer = 1 as uint16;
    }
}
pub unsafe fn SpacePirates_Func_8() -> uint16 {
    let mut v0: uint16 = 8 as uint16;
    while *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        == 0
    {
        v0 = (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return 0 as uint16;
        }
    }
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (abs16(
        (*((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
    ) as ::core::ffi::c_int
        - 32 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
        || (abs16(
            (*((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - (*E).base.y_pos as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int
            - 32 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
    {
        return 0 as uint16;
    }
    (*E).base.current_instruction = (if (*E).base.x_pos as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        addr_kSpacePirates_Ilist_F462 as ::core::ffi::c_int
    } else {
        addr_kSpacePirates_Ilist_F270 as ::core::ffi::c_int
    }) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    return 1 as uint16;
}
pub unsafe fn SpacePirates_Func_9() -> uint16 {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (abs16(
        ((*E).sps_var_D as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
    ) as ::core::ffi::c_int
        - 32 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        return 0 as uint16;
    }
    let mut v1: uint16 = addr_kSpacePirates_Ilist_F1C4 as ::core::ffi::c_int as uint16;
    if (*E).base.x_pos as ::core::ffi::c_int == (*E).sps_var_E as ::core::ffi::c_int {
        v1 = addr_kSpacePirates_Ilist_F3B2 as ::core::ffi::c_int as uint16;
    }
    (*E).base.current_instruction = v1;
    (*E).base.instruction_timer = 1 as uint16;
    return 1 as uint16;
}
pub unsafe fn SpacePirates_Func_10() -> uint16 {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (abs16(
        (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
    ) as ::core::ffi::c_int
        - 40 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
        || (abs16(
            (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - (*E).base.y_pos as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int
            - 40 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
    {
        return 0 as uint16;
    }
    (*E).base.current_instruction = (if (*E).base.x_pos as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        addr_kSpacePirates_Ilist_F51A as ::core::ffi::c_int
    } else {
        addr_kSpacePirates_Ilist_F32E as ::core::ffi::c_int
    }) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    return 1 as uint16;
}
pub unsafe fn SpacePirates_Func_11(mut k: uint16) {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
        - *(&raw mut (*E).sps_var_00 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    (*E).sps_var_00 = ((*E).sps_var_00 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    if (((*E).base.x_pos as ::core::ffi::c_int - (*E).sps_var_D as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        (*E).sps_var_A = (0xb2f84c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn SpacePirates_Func_12(mut k: uint16) {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
        - *(&raw mut (*E).sps_var_00 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    let mut v2: uint16 =
        ((*E).sps_var_00 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
    (*E).sps_var_00 = v2;
    if v2 == 0 {
        (*E).sps_var_A = (0xb2804b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction =
            addr_kSpacePirates_Ilist_F2F8 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.x_pos = (*E).sps_var_E;
        SpacePirates_FB11(k);
    }
}
pub unsafe fn SpacePirates_Func_13(mut k: uint16) {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
        + *(&raw mut (*E).sps_var_00 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    (*E).sps_var_00 = ((*E).sps_var_00 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    if ((*E).base.x_pos as ::core::ffi::c_int - (*E).sps_var_D as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).sps_var_A = (0xb2f8c5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn SpacePirates_Func_14(mut k: uint16) {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
        + *(&raw mut (*E).sps_var_00 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    let mut v2: uint16 =
        ((*E).sps_var_00 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
    (*E).sps_var_00 = v2;
    if v2 == 0 {
        (*E).sps_var_A = (0xb2804b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction =
            addr_kSpacePirates_Ilist_F4EA as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.x_pos = (*E).sps_var_F;
        SpacePirates_FB11(k);
    }
}
pub unsafe fn SpacePirates_Func_15(mut k: uint16) {
    if SpacePirates_Func_8() == 0 && SpacePirates_Func_10() == 0 {
        SpacePirates_Func_16();
    }
}
pub unsafe fn SpacePirates_Func_16() {
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if ((abs16(
        ((*E).sps_var_D as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
    ) as ::core::ffi::c_int
        - 32 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        let mut v2: uint16 = 0;
        loop {
            v2 = (NextRandom() as ::core::ffi::c_int & 3 as ::core::ffi::c_int) as uint16;
            if !(v2 == 0) {
                break;
            }
        }
        v3 = 0 as int16;
        if (*E).base.x_pos as ::core::ffi::c_int == (*E).sps_var_E as ::core::ffi::c_int {
            v3 = 4 as int16;
        }
        (*E).base.current_instruction = *(g_rom.offset(
            (((0xb2f959 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xb2f959 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v2 as ::core::ffi::c_int + v3 as ::core::ffi::c_int) as isize);
        (*E).base.instruction_timer = 1 as uint16;
    }
}
pub unsafe fn SpacePirates_Instr_19(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int);
    (*E).sps_var_00 = 1536 as uint16;
    (*E).sps_var_03 = ((*E).sps_var_D as ::core::ffi::c_int
        + (((*E).sps_var_F as ::core::ffi::c_int - (*E).sps_var_D as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int)) as uint16;
    return jp;
}
pub unsafe fn SpacePirates_F985(mut k: uint16) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int);
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        - *(&raw mut (*E).sps_var_00 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int) as uint16;
    v2 = ((*E).sps_var_00 as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as int16;
    (*E).sps_var_00 = v2 as uint16;
    if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).sps_var_A = (0xb2f9c1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction =
            addr_kSpacePirates_Ilist_F2A0 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).sps_var_00 = 1536 as uint16;
    }
}
pub unsafe fn SpacePirates_F9C1(mut k: uint16) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as uint16;
    if Enemy_MoveDown(
        k,
        ((*(&raw mut (*E).sps_var_00 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as uint32)
            << 16 as ::core::ffi::c_int
            | *(&raw mut (*E).sps_var_00 as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as uint16 as uint32) as int32,
    ) as ::core::ffi::c_int
        != 0
        || {
            v2 = ((*E).sps_var_00 as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as int16;
            (*E).sps_var_00 = v2 as uint16;
            (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int
        }
        || v2 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*E).sps_var_A = (0xb2fa15 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction =
            addr_kSpacePirates_Ilist_F2B2 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.y_pos = (*E).sps_var_08;
        SpacePirates_FB11(k);
    }
}
pub unsafe fn SpacePirates_FA15(mut k: uint16) {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int);
    let mut v2: uint16 =
        ((*E).base.x_pos as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    (*E).base.x_pos = v2;
    if ((v2 as ::core::ffi::c_int - (*E).sps_var_E as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        (*E).base.x_pos = (*E).sps_var_E;
        (*E).base.current_instruction =
            addr_kSpacePirates_Ilist_F2F8 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).sps_var_A = (0xb2804b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn SpacePirates_Instr_21(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int);
    (*E).sps_var_00 = 1536 as uint16;
    (*E).sps_var_03 = ((*E).sps_var_E as ::core::ffi::c_int
        + (((*E).sps_var_D as ::core::ffi::c_int - (*E).sps_var_E as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int)) as uint16;
    return jp;
}
pub unsafe fn SpacePirates_FA59(mut k: uint16) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int);
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        - *(&raw mut (*E).sps_var_00 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int) as uint16;
    v2 = ((*E).sps_var_00 as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as int16;
    (*E).sps_var_00 = v2 as uint16;
    if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).sps_var_A = (0xb2fa95 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction =
            addr_kSpacePirates_Ilist_F492 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).sps_var_00 = 1536 as uint16;
    }
}
pub unsafe fn SpacePirates_FA95(mut k: uint16) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16;
    if Enemy_MoveDown(
        k,
        ((*(&raw mut (*E).sps_var_00 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as uint32)
            << 16 as ::core::ffi::c_int
            | *(&raw mut (*E).sps_var_00 as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as uint16 as uint32) as int32,
    ) as ::core::ffi::c_int
        != 0
        || {
            v2 = ((*E).sps_var_00 as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as int16;
            (*E).sps_var_00 = v2 as uint16;
            (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int
        }
        || v2 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*E).sps_var_A = (0xb2fae9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction =
            addr_kSpacePirates_Ilist_F4A4 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.y_pos = (*E).sps_var_08;
        SpacePirates_FB11(k);
    }
}
pub unsafe fn SpacePirates_FAE9(mut k: uint16) {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int);
    let mut v2: uint16 =
        ((*E).base.x_pos as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    (*E).base.x_pos = v2;
    if (v2 as ::core::ffi::c_int - (*E).sps_var_F as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).base.x_pos = (*E).sps_var_F;
        (*E).base.current_instruction =
            addr_kSpacePirates_Ilist_F4EA as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).sps_var_A = (0xb2804b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn SpacePirates_FB11(mut k: uint16) {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int);
    CreateSpriteAtPos(
        ((*E).base.x_pos as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16,
        ((*E).base.y_pos as ::core::ffi::c_int + 28 as ::core::ffi::c_int) as uint16,
        10 as uint16,
        0 as uint16,
    );
    CreateSpriteAtPos(
        ((*E).base.x_pos as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16,
        ((*E).base.y_pos as ::core::ffi::c_int + 28 as ::core::ffi::c_int) as uint16,
        10 as uint16,
        0 as uint16,
    );
}
pub unsafe fn SpacePirates_Instr_12(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    eproj_spawn_pt = Point16U {
        x: ((*E).base.x_pos as ::core::ffi::c_int - 24 as ::core::ffi::c_int) as uint16,
        y: ((*E).base.y_pos as ::core::ffi::c_int
            - *jp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as uint16,
    };
    eproj_spawn_r22 = 0 as uint16;
    SpawnEprojWithRoomGfx(
        addr_stru_86A17B as ::core::ffi::c_int as uint16,
        0 as uint16,
    );
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn SpacePirates_Instr_14(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    eproj_spawn_pt = Point16U {
        x: ((*E).base.x_pos as ::core::ffi::c_int + 24 as ::core::ffi::c_int) as uint16,
        y: ((*E).base.y_pos as ::core::ffi::c_int
            - *jp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as uint16,
    };
    eproj_spawn_r22 = 1 as uint16;
    SpawnEprojWithRoomGfx(
        addr_stru_86A17B as ::core::ffi::c_int as uint16,
        1 as uint16,
    );
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn SpacePirates_Instr_11(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .sps_var_A = *jp.offset(0 as ::core::ffi::c_int as isize);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn SpacePirates_Instr_13(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    if IsSamusWithinEnemy_Y(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        0x10 as uint16,
    ) != 0
    {
        if (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - (*Get_SpacePirates(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int,
            ))
            .base
            .x_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            return addr_kSpacePirates_Ilist_FC0E as ::core::ffi::c_int as uintptr_t
                as *const uint16;
        }
        return addr_kSpacePirates_Ilist_FB8C as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else {
        if (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - (*Get_SpacePirates(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int,
            ))
            .base
            .x_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            return addr_kSpacePirates_Ilist_FB64 as ::core::ffi::c_int as uintptr_t
                as *const uint16;
        }
        return addr_kSpacePirates_Ilist_FBE6 as ::core::ffi::c_int as uintptr_t as *const uint16;
    };
}
pub unsafe fn WalkingSpacePirates_Init() {
    let mut v0: uint16 = addr_kSpacePirates_Ilist_FB64 as ::core::ffi::c_int as uint16;
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).sps_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v0 = addr_kSpacePirates_Ilist_FBE6 as ::core::ffi::c_int as uint16;
    }
    (*E).base.current_instruction = v0;
    (*E).sps_var_A = (0xb2804b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).sps_var_F = ((*E).sps_parameter_2 as ::core::ffi::c_int
        + (*E).base.x_pos as ::core::ffi::c_int) as uint16;
    (*E).sps_var_E = ((*E).base.x_pos as ::core::ffi::c_int
        - (*E).sps_parameter_2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn WalkingSpacePirates_Main() {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    CallSpacePiratesEnemyFunc(
        ((*E).sps_var_A as ::core::ffi::c_int | 0xb20000 as ::core::ffi::c_int) as uint32,
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    if (*E).sps_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        WalkingSpacePirates_FE4B();
    }
}
pub unsafe fn WalkingSpacePirates_FD44() {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if IsSamusWithinEnemy_Y(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        0x10 as uint16,
    ) != 0
    {
        let mut v1: uint16 = addr_kSpacePirates_Ilist_FB8C as ::core::ffi::c_int as uint16;
        if (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            v1 = addr_kSpacePirates_Ilist_FC0E as ::core::ffi::c_int as uint16;
        }
        (*E).base.current_instruction = v1;
        (*E).base.instruction_timer = 1 as uint16;
    } else if Enemy_MoveDown(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
    ) != 0
    {
        let mut x_pos: uint16 = (*E).base.x_pos;
        (*E).sps_var_00 = x_pos;
        (*E).base.x_pos = (x_pos as ::core::ffi::c_int - 17 as ::core::ffi::c_int) as uint16;
        let mut result: uint8 = Enemy_MoveDown(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            (1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
        );
        (*E).base.x_pos = (*E).sps_var_00;
        if result != 0 {
            EnemyFunc_BBBF(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                (-(9 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int,
            );
            if Enemy_MoveRight_IgnoreSlopes(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                -(14337 as int32),
            ) == 0
            {
                if ((*E).base.x_pos as ::core::ffi::c_int - (*E).sps_var_E as ::core::ffi::c_int)
                    as int16 as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int
                {
                    return;
                }
            }
        }
        (*E).base.current_instruction =
            addr_kSpacePirates_Ilist_FBC6 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
    }
}
pub unsafe fn WalkingSpacePirates_FDCE() {
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if IsSamusWithinEnemy_Y(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        0x10 as uint16,
    ) != 0
    {
        let mut v1: uint16 = addr_kSpacePirates_Ilist_FB8C as ::core::ffi::c_int as uint16;
        if (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            v1 = addr_kSpacePirates_Ilist_FC0E as ::core::ffi::c_int as uint16;
        }
        (*E).base.current_instruction = v1;
        (*E).base.instruction_timer = 1 as uint16;
    } else if Enemy_MoveDown(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
    ) != 0
    {
        let mut x_pos: uint16 = (*E).base.x_pos;
        (*E).sps_var_00 = x_pos;
        (*E).base.x_pos = (x_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
        let mut result: uint8 = Enemy_MoveDown(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            (1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
        );
        (*E).base.x_pos = (*E).sps_var_00;
        if result == 0
            || Enemy_MoveRight_IgnoreSlopes(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                14336 as int32,
            ) as ::core::ffi::c_int
                != 0
            || ((*E).base.x_pos as ::core::ffi::c_int - (*E).sps_var_F as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
        {
            (*E).base.current_instruction =
                addr_kSpacePirates_Ilist_FC48 as ::core::ffi::c_int as uint16;
            (*E).base.instruction_timer = 1 as uint16;
        }
    }
}
pub unsafe fn WalkingSpacePirates_FE4B() {
    let mut v0: uint16 = 8 as uint16;
    while *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        == 0
    {
        v0 = (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return;
        }
    }
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if ((abs16(
        (*((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
    ) as ::core::ffi::c_int
        - 32 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
        && ((abs16(
            (*((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - (*E).base.y_pos as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int
            - 32 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
    {
        (*E).base.current_instruction = (if (*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            addr_kSpacePirates_Ilist_FB58 as ::core::ffi::c_int
        } else {
            addr_kSpacePirates_Ilist_FB4C as ::core::ffi::c_int
        }) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
    }
}
