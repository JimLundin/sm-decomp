use crate::game::sm_80::{
    CheckEventHappened, ClearEventHappened, NextRandom, QueueMode7Transfers, QueueMusic_Delayed8,
    QueueSfx1_Max6, QueueSfx2_Max3, QueueSfx2_Max6, QueueSfx3_Max3, QueueSfx3_Max6,
    SetBossBitForCurArea, SetEventHappened,
};
use crate::game::sm_81::DrawSpritemap;
use crate::game::sm_86::{
    Math_MultByCos, Math_MultBySin, SpawnEprojWithGfx, SpawnEprojWithRoomGfx,
};
use crate::game::sm_8b::CalcCeresSteamPos_Mode7;
use crate::game::sm_90::{
    CallSomeSamusCode, DisableMinimapAndMarkBossRoomAsExplored, Samus_SetPushedOutOfCeresRidley,
};
use crate::game::sm_91::Samus_DealDamage;
use crate::game::sm_94::kSinCosTable8bit_Sext;
use crate::game::sm_a0::{
    CalculateAngleFromXY, CheckIfEnemyIsOnScreen, CosineMult8bit, EnemyDeathAnimation,
    Enemy_ItemDrop_MiniKraid, Enemy_ItemDrop_Ridley, Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes,
    Enemy_SwitchToFrozenAi, GetSamusEnemyDelta_X, GetSamusEnemyDelta_Y, NormalEnemyFrozenAI,
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy, NormalEnemyShotAi,
    NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAi,
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy, SineMult8bit, SpawnEnemy, SuitDamageDivision,
    SwitchEnemyAiToMainAi,
};
use crate::game::sm_a9::{
    ComputeCosMult, ComputeSinMult, MoveEnemyWithVelocity, Shitroid_Func_2, WriteColorsToPalette,
    WriteColorsToTargetPalette,
};
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::game_state::Game;
use crate::rtl::abs16;
use crate::rtl::AddToHiLo;
use crate::rtl::{mov24, Mult8x8, RomPtr, SnesDivide, Unreachable};
extern "C" {
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t,
    EnemyBase, EnemyData, EnemyProps, EnemySpawnData, Func_V, LongPtr, OamEnt, PairU16, Point16U,
    Rect16U, VoidP, VramWriteEntry,
};
pub const kEnemyProps_RespawnIfKilled: EnemyProps = 16384;
pub const kEnemyProps_DisableSamusColl: EnemyProps = 8192;
pub const kEnemyProps_BlockPlasmaBeam: EnemyProps = 4096;
pub const kEnemyProps_ProcessedOffscreen: EnemyProps = 2048;
pub const kEnemyProps_Tangible: EnemyProps = 1024;
pub const kEnemyProps_Deleted: EnemyProps = 512;
pub const kEnemyProps_Invisible: EnemyProps = 256;
pub use crate::types::{Consts_86, ExtraEnemyRam7800, ExtraEnemyRam8000, Ram7800_Kraid};
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
pub use crate::types::Consts_A6;
pub const addr_stru_A6FCF9: Consts_A6 = 64761;
pub const addr_stru_A6FCE1: Consts_A6 = 64737;
pub const addr_kCeresDoor_Sprmap_FAC7: Consts_A6 = 64199;
pub const addr_word_A6F50C: Consts_A6 = 62732;
pub const addr_kCeresDoor_Palette: Consts_A6 = 62700;
pub const addr_kRidley_Ilist_E91D: Consts_A6 = 59677;
pub const addr_kRidley_Ilist_E73A: Consts_A6 = 59194;
pub const addr_kRidley_Ilist_E706: Consts_A6 = 59142;
pub const addr_kRidley_Ilist_E6F0: Consts_A6 = 59120;
pub const addr_kRidley_Ilist_E6C8: Consts_A6 = 59080;
pub const addr_kRidley_Ilist_E690: Consts_A6 = 59024;
pub const addr_kRidley_Ilist_E658: Consts_A6 = 58968;
pub const addr_kRidley_Ilist_E548: Consts_A6 = 58696;
pub const addr_kRidley_Ilist_E538: Consts_A6 = 58680;
pub const addr_word_A6E1CF: Consts_A6 = 57807;
pub const addr_word_A6E16F: Consts_A6 = 57711;
pub const addr_off_A6DAD4: Consts_A6 = 56020;
pub const addr_off_A6DAD0: Consts_A6 = 56016;
pub const addr_word_A6DA71: Consts_A6 = 55921;
pub const addr_word_A6D3A6: Consts_A6 = 54182;
pub const addr_word_A6D398: Consts_A6 = 54168;
pub const addr_word_A6D38A: Consts_A6 = 54154;
pub const addr_word_A6D37C: Consts_A6 = 54140;
pub const addr_word_A6D36E: Consts_A6 = 54126;
pub const addr_kRidleysExplosion_Ilist_CA53: Consts_A6 = 51795;
pub const addr_kRidleysExplosion_Ilist_CA4D: Consts_A6 = 51789;
pub const addr_kRidleysExplosion_Ilist_CA47: Consts_A6 = 51783;
pub const addr_stru_A6CA37: Consts_A6 = 51767;
pub const addr_stru_A6CA27: Consts_A6 = 51751;
pub const addr_stru_A6CA17: Consts_A6 = 51735;
pub const addr_stru_A6CA07: Consts_A6 = 51719;
pub const addr_stru_A6C9F7: Consts_A6 = 51703;
pub const addr_stru_A6C9E7: Consts_A6 = 51687;
pub const addr_stru_A6C9D7: Consts_A6 = 51671;
pub const addr_stru_A6C9C7: Consts_A6 = 51655;
pub const addr_stru_A6C9B7: Consts_A6 = 51639;
pub const addr_stru_A6C9A7: Consts_A6 = 51623;
pub const addr_stru_A6C997: Consts_A6 = 51607;
pub const addr_stru_A6C987: Consts_A6 = 51591;
pub const addr_locret_A6C600: Consts_A6 = 50688;
pub const addr_stru_A6C4CB: Consts_A6 = 50379;
pub const addr_stru_A6C3B8: Consts_A6 = 50104;
pub const addr_byte_A6C15D: Consts_A6 = 49501;
pub const addr_kBabyMetroid_Ilist_BF31: Consts_A6 = 48945;
pub const addr_locret_A6BF19: Consts_A6 = 48921;
pub const addr_loc_A6B455: Consts_A6 = 46165;
pub const addr_off_A6B3DC: Consts_A6 = 46044;
pub const addr_off_A6B3CC: Consts_A6 = 46028;
pub const addr_off_A6B3BC: Consts_A6 = 46012;
pub const addr_off_A6B3AC: Consts_A6 = 45996;
pub const addr_off_A6B39C: Consts_A6 = 45980;
pub const addr_off_A6B38C: Consts_A6 = 45964;
pub const addr_stru_A6AA3F: Consts_A6 = 43583;
pub const addr_stru_A6AA2F: Consts_A6 = 43567;
pub const addr_word_A6AA01: Consts_A6 = 43521;
pub const addr_word_A6A9E3: Consts_A6 = 43491;
pub const addr_loc_A6A7F9: Consts_A6 = 43001;
pub const addr_kCeresRidley_Sprmap_A329: Consts_A6 = 41769;
pub const addr_kFakeKraid_Ilist_9A2A: Consts_A6 = 39466;
pub const addr_stru_A69A14: Consts_A6 = 39444;
pub const addr_stru_A699FC: Consts_A6 = 39420;
pub const addr_kFakeKraid_Ilist_99DC: Consts_A6 = 39388;
pub const addr_stru_A699C6: Consts_A6 = 39366;
pub const addr_stru_A699AE: Consts_A6 = 39342;
pub const addr_kNuclearWaffle_Ilist_9490: Consts_A6 = 38032;
pub const addr_kFireGeyser_Ilist_8DA9: Consts_A6 = 36265;
pub const addr_kFireGeyser_Ilist_8D1B: Consts_A6 = 36123;
pub const addr_kSpikeyPlatform_Ilist_8B29: Consts_A6 = 35625;
pub const addr_kBoulder_Ilist_86CB: Consts_A6 = 34507;
pub const addr_kBoulder_Ilist_86A7: Consts_A6 = 34471;
pub use crate::types::Consts_A9;
pub const addr_kShitroid_Ilist_F93A: Consts_A9 = 63802;
pub const addr_kShitroid_Ilist_F924: Consts_A9 = 63780;
pub const addr_kShitroid_Ilist_F90E: Consts_A9 = 63758;
pub const addr_kShitroid_Ilist_F906: Consts_A9 = 63750;
pub const addr_kShitroidInCutscene_Palette: Consts_A9 = 63718;
pub const addr_word_A9F8C6: Consts_A9 = 63686;
pub const addr_kDeadSidehopper_Palette_0: Consts_A9 = 63654;
pub const addr_word_A9F711: Consts_A9 = 63249;
pub const addr_word_A9F6D1: Consts_A9 = 63185;
pub const addr_kDeadMonsters_Ilist_ECEF: Consts_A9 = 60655;
pub const addr_kDeadMonsters_Ilist_ECE9: Consts_A9 = 60649;
pub const addr_kDeadMonsters_Ilist_ECE3: Consts_A9 = 60643;
pub const addr_kDeadMonsters_Ilist_ECAC: Consts_A9 = 60588;
pub const addr_stru_A9DE08: Consts_A9 = 56840;
pub const addr_stru_A9DD78: Consts_A9 = 56696;
pub const addr_stru_A9DD68: Consts_A9 = 56680;
pub const addr_stru_A9DD58: Consts_A9 = 56664;
pub const addr_kDeadTorizo_Hitbox_D77C: Consts_A9 = 55164;
pub const addr_kDeadTorizo_Sprmap_D761: Consts_A9 = 55137;
pub const addr_kDeadTorizo_Ilist_D6DC: Consts_A9 = 55004;
pub const addr_word_A9D082: Consts_A9 = 53378;
pub const addr_stru_A9D046: Consts_A9 = 53318;
pub const addr_kShitroid_Ilist_CFCE: Consts_A9 = 53198;
pub const addr_kShitroid_Ilist_CFB8: Consts_A9 = 53176;
pub const addr_kShitroid_Ilist_CFA2: Consts_A9 = 53154;
pub const addr_stru_A9CA24: Consts_A9 = 51748;
pub const addr_loc_A9C959: Consts_A9 = 51545;
pub const addr_loc_A9C49C: Consts_A9 = 50332;
pub const addr_locret_A9C353: Consts_A9 = 50003;
pub const addr_locret_A9C18D: Consts_A9 = 49549;
pub const addr_stru_A9BE28: Consts_A9 = 48680;
pub const addr_byte_A9B6DF: Consts_A9 = 46815;
pub const addr_byte_A9B6DC: Consts_A9 = 46812;
pub const addr_word_A9B427: Consts_A9 = 46119;
pub const addr_word_A9B10F: Consts_A9 = 45327;
pub const addr_word_A9B109: Consts_A9 = 45321;
pub const addr_kMotherBrain_Sprmap_A694: Consts_A9 = 42644;
pub const addr_kMotherBrain_Ilist_9F6C: Consts_A9 = 40812;
pub const addr_kMotherBrain_Ilist_9F34: Consts_A9 = 40756;
pub const addr_kMotherBrain_Ilist_9F00: Consts_A9 = 40704;
pub const addr_kMotherBrain_Ilist_9ECC: Consts_A9 = 40652;
pub const addr_kMotherBrain_Ilist_9DBB: Consts_A9 = 40379;
pub const addr_kMotherBrain_Ilist_9DB1: Consts_A9 = 40369;
pub const addr_kMotherBrain_Ilist_9D7F: Consts_A9 = 40319;
pub const addr_kMotherBrain_Ilist_9D3D: Consts_A9 = 40253;
pub const addr_kMotherBrain_Ilist_9D25: Consts_A9 = 40229;
pub const addr_stru_A99CD1: Consts_A9 = 40145;
pub const addr_stru_A99C9F: Consts_A9 = 40095;
pub const addr_kMotherBrain_Ilist_9C87: Consts_A9 = 40071;
pub const addr_kMotherBrain_Ilist_9C77: Consts_A9 = 40055;
pub const addr_stru_A99C5F: Consts_A9 = 40031;
pub const addr_stru_A99C47: Consts_A9 = 40007;
pub const addr_kMotherBrain_Ilist_9C39: Consts_A9 = 39993;
pub const addr_kMotherBrain_Ilist_9C29: Consts_A9 = 39977;
pub const addr_stru_A99C21: Consts_A9 = 39969;
pub const addr_kMotherBrain_Ilist_9C13: Consts_A9 = 39955;
pub const addr_kMotherBrain_Ilist_9BE7: Consts_A9 = 39911;
pub const addr_kMotherBrain_Ilist_9BB3: Consts_A9 = 39859;
pub const addr_kMotherBrain_Ilist_9B7F: Consts_A9 = 39807;
pub const addr_kMotherBrain_Ilist_9A42: Consts_A9 = 39490;
pub const addr_kMotherBrain_Ilist_9A26: Consts_A9 = 39462;
pub const addr_kMotherBrain_Ilist_9A0A: Consts_A9 = 39434;
pub const addr_kMotherBrain_Ilist_9A02: Consts_A9 = 39426;
pub const addr_kMotherBrain_Ilist_99F2: Consts_A9 = 39410;
pub const addr_kMotherBrain_Ilist_99E2: Consts_A9 = 39394;
pub const addr_kMotherBrain_Ilist_99C6: Consts_A9 = 39366;
pub const addr_kMotherBrain_Ilist_99AA: Consts_A9 = 39338;
pub const addr_kMotherBrain_Ilist_98C6: Consts_A9 = 39110;
pub const addr_kMotherBrain_Ilist_97A4: Consts_A9 = 38820;
pub const addr_kMotherBrainPalette_5: Consts_A9 = 38194;
pub const addr_kMotherBrainPalette_4: Consts_A9 = 38162;
pub const addr_kMotherBrainPalette_3: Consts_A9 = 38130;
pub const addr_kMotherBrainPalette_2: Consts_A9 = 38098;
pub const addr_kMotherBrainPalette_1: Consts_A9 = 38066;
pub const addr_kMotherBrainPalette_0: Consts_A9 = 38034;
pub const addr_kMotherBrainsBrain_Palette: Consts_A9 = 38002;
pub const addr_stru_A9902F: Consts_A9 = 36911;
pub const addr_stru_A99003: Consts_A9 = 36867;
pub const addr_stru_A98FE5: Consts_A9 = 36837;
pub const addr_stru_A98FC7: Consts_A9 = 36807;
pub const addr_stru_A98F8F: Consts_A9 = 36751;
pub const addr_stru_A98B25: Consts_A9 = 35621;
pub const addr_stru_A98B05: Consts_A9 = 35589;
pub const addr_stru_A98AF5: Consts_A9 = 35573;
pub const addr_stru_A98AE5: Consts_A9 = 35557;
pub const addr_locret_A98AE4: Consts_A9 = 35556;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Ridley {
    pub base: EnemyBase,
    pub ridley_var_A: uint16,
    pub ridley_var_B: uint16,
    pub ridley_var_C: uint16,
    pub ridley_var_D: uint16,
    pub ridley_var_E: uint16,
    pub ridley_var_F: uint16,
    pub ridley_parameter_1: uint16,
    pub ridley_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub ridley_var_00: uint16,
    pub ridley_var_01: uint16,
    pub ridley_var_02: uint16,
    pub ridley_var_03: uint16,
    pub ridley_var_04: uint16,
    pub ridley_var_05: uint16,
    pub ridley_var_06: uint16,
    pub ridley_var_07: uint16,
    pub ridley_var_08: uint16,
    pub ridley_var_09: uint16,
    pub ridley_var_0A: uint16,
    pub ridley_var_0B: uint16,
    pub ridley_var_0C: uint16,
    pub ridley_var_0D: uint16,
    pub ridley_var_0E: uint16,
    pub ridley_var_0F: uint16,
    pub ridley_var_10: uint16,
    pub ridley_var_11: uint16,
    pub ridley_var_12: uint16,
    pub ridley_var_13: uint16,
    pub ridley_var_14: uint16,
    pub ridley_var_15: uint16,
    pub ridley_var_16: uint16,
    pub ridley_var_17: uint16,
    pub ridley_var_18: uint16,
    pub ridley_var_19: uint16,
    pub ridley_var_1A: uint16,
    pub ridley_var_1B: uint16,
    pub ridley_var_1C: uint16,
    pub ridley_var_1D: uint16,
    pub ridley_var_1E: uint16,
    pub ridley_var_1F: uint16,
    pub pad2: [uint8; 1984],
    pub ridley_var_20: uint16,
    pub ridley_var_21: uint16,
    pub ridley_var_22: uint16,
    pub ridley_var_23: uint16,
    pub ridley_var_24: uint16,
    pub ridley_var_25: uint16,
    pub ridley_var_26: uint16,
    pub ridley_var_27: uint16,
    pub ridley_var_28: uint16,
    pub ridley_var_29: uint16,
    pub ridley_var_2A: uint16,
    pub ridley_var_2B: uint16,
    pub ridley_var_2C: uint16,
    pub ridley_var_2D: uint16,
    pub ridley_var_2E: uint16,
    pub ridley_var_2F: uint16,
    pub ridley_var_30: uint16,
    pub ridley_var_31: uint16,
    pub ridley_var_32: uint16,
    pub ridley_var_33: uint16,
    pub ridley_var_34: uint16,
    pub ridley_var_35: uint16,
    pub ridley_var_36: uint16,
    pub ridley_var_37: uint16,
    pub ridley_var_38: uint16,
    pub ridley_var_39: uint16,
    pub ridley_var_3A: uint16,
    pub ridley_var_3B: uint16,
    pub ridley_var_3C: uint16,
    pub ridley_var_3D: uint16,
    pub ridley_var_3E: uint16,
    pub ridley_var_3F: uint16,
    pub pad3: [uint8; 1984],
    pub ridley_var_40: uint16,
    pub ridley_var_41: uint16,
    pub ridley_var_42: uint16,
    pub ridley_var_43: uint16,
    pub ridley_var_44: uint16,
    pub ridley_var_45: uint16,
    pub ridley_var_46: uint16,
    pub ridley_var_47: uint16,
    pub ridley_var_48: uint16,
    pub ridley_var_49: uint16,
    pub ridley_var_4A: uint16,
    pub ridley_var_4B: uint16,
    pub ridley_var_4C: uint16,
    pub ridley_var_4D: uint16,
    pub ridley_var_4E: uint16,
    pub ridley_var_4F: uint16,
    pub ridley_var_50: uint16,
    pub ridley_var_51: uint16,
    pub ridley_var_52: uint16,
    pub ridley_var_53: uint16,
    pub ridley_var_54: uint16,
    pub ridley_var_55: uint16,
    pub ridley_var_56: uint16,
    pub ridley_var_57: uint16,
    pub ridley_var_58: uint16,
    pub ridley_var_59: uint16,
    pub ridley_var_5A: uint16,
    pub ridley_var_5B: uint16,
    pub ridley_var_5C: uint16,
    pub ridley_var_5D: uint16,
    pub ridley_var_5E: uint16,
    pub ridley_var_5F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_CeresDoor {
    pub base: EnemyBase,
    pub cdr_var_A: uint16,
    pub cdr_var_B: uint16,
    pub cdr_var_C: uint16,
    pub cdr_var_D: uint16,
    pub cdr_var_E: uint16,
    pub cdr_var_F: uint16,
    pub cdr_parameter_1: uint16,
    pub cdr_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_CeresSteam {
    pub base: EnemyBase,
    pub csm_var_A: uint16,
    pub csm_var_B: uint16,
    pub csm_var_C: uint16,
    pub csm_var_D: uint16,
    pub csm_var_E: uint16,
    pub csm_var_F: uint16,
    pub csm_parameter_1: uint16,
    pub csm_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_FakeKraid {
    pub base: EnemyBase,
    pub fkd_var_A: uint16,
    pub fkd_var_B: uint16,
    pub fkd_var_C: uint16,
    pub fkd_var_D: uint16,
    pub fkd_var_E: uint16,
    pub fkd_var_F: uint16,
    pub fkd_parameter_1: uint16,
    pub fkd_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub fkd_var_00: uint16,
    pub fkd_var_01: uint16,
    pub fkd_var_02: uint16,
    pub fkd_var_03: uint16,
    pub fkd_var_04: uint16,
    pub fkd_var_05: uint16,
    pub fkd_var_06: uint16,
    pub fkd_var_07: uint16,
    pub fkd_var_08: uint16,
    pub fkd_var_09: uint16,
    pub fkd_var_0A: uint16,
    pub fkd_var_0B: uint16,
    pub fkd_var_0C: uint16,
    pub fkd_var_0D: uint16,
    pub fkd_var_0E: uint16,
    pub fkd_var_0F: uint16,
    pub fkd_var_10: uint16,
    pub fkd_var_11: uint16,
    pub fkd_var_12: uint16,
    pub fkd_var_13: uint16,
    pub fkd_var_14: uint16,
    pub fkd_var_15: uint16,
    pub fkd_var_16: uint16,
    pub fkd_var_17: uint16,
    pub fkd_var_18: uint16,
    pub fkd_var_19: uint16,
    pub fkd_var_1A: uint16,
    pub fkd_var_1B: uint16,
    pub fkd_var_1C: uint16,
    pub fkd_var_1D: uint16,
    pub fkd_var_1E: uint16,
    pub fkd_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_FireGeyser {
    pub base: EnemyBase,
    pub fgr_var_A: uint16,
    pub fgr_var_B: uint16,
    pub fgr_var_C: uint16,
    pub fgr_var_D: uint16,
    pub fgr_var_E: uint16,
    pub fgr_var_F: uint16,
    pub fgr_parameter_1: uint16,
    pub fgr_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_NuclearWaffle {
    pub base: EnemyBase,
    pub nwe_var_A: uint16,
    pub nwe_var_B: uint16,
    pub nwe_var_C: uint16,
    pub nwe_var_D: uint16,
    pub nwe_var_E: uint16,
    pub nwe_var_F: uint16,
    pub nwe_parameter_1: uint16,
    pub nwe_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub nwe_var_00: uint16,
    pub nwe_var_01: uint16,
    pub nwe_var_02: uint16,
    pub nwe_var_03: uint16,
    pub nwe_var_04: uint16,
    pub nwe_var_05: uint16,
    pub nwe_var_06: uint16,
    pub nwe_var_07: uint16,
    pub nwe_var_08: uint16,
    pub nwe_var_09: uint16,
    pub nwe_var_0A: uint16,
    pub nwe_var_0B: uint16,
    pub nwe_var_0C: uint16,
    pub nwe_var_0D: uint16,
    pub nwe_var_0E: uint16,
    pub nwe_var_0F: uint16,
    pub nwe_var_10: uint16,
    pub nwe_var_11: uint16,
    pub nwe_var_12: uint16,
    pub nwe_var_13: uint16,
    pub nwe_var_14: uint16,
    pub nwe_var_15: uint16,
    pub nwe_var_16: uint16,
    pub nwe_var_17: uint16,
    pub nwe_var_18: uint16,
    pub nwe_var_19: uint16,
    pub nwe_var_1A: uint16,
    pub nwe_var_1B: uint16,
    pub nwe_var_1C: uint16,
    pub nwe_var_1D: uint16,
    pub nwe_var_1E: uint16,
    pub nwe_var_1F: uint16,
    pub pad2: [uint8; 1984],
    pub nwe_var_20: uint16,
    pub nwe_var_21: uint16,
    pub nwe_var_22: uint16,
    pub nwe_var_23: uint16,
    pub nwe_var_24: uint16,
    pub nwe_var_25: uint16,
    pub nwe_var_26: uint16,
    pub nwe_var_27: uint16,
    pub nwe_var_28: uint16,
    pub nwe_var_29: uint16,
    pub nwe_var_2A: uint16,
    pub nwe_var_2B: uint16,
    pub nwe_var_2C: uint16,
    pub nwe_var_2D: uint16,
    pub nwe_var_2E: uint16,
    pub nwe_var_2F: uint16,
    pub nwe_var_30: uint16,
    pub nwe_var_31: uint16,
    pub nwe_var_32: uint16,
    pub nwe_var_33: uint16,
    pub nwe_var_34: uint16,
    pub nwe_var_35: uint16,
    pub nwe_var_36: uint16,
    pub nwe_var_37: uint16,
    pub nwe_var_38: uint16,
    pub nwe_var_39: uint16,
    pub nwe_var_3A: uint16,
    pub nwe_var_3B: uint16,
    pub nwe_var_3C: uint16,
    pub nwe_var_3D: uint16,
    pub nwe_var_3E: uint16,
    pub nwe_var_3F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_SpikeyPlatform {
    pub base: EnemyBase,
    pub spm_var_A: uint16,
    pub spm_var_B: uint16,
    pub spm_var_C: uint16,
    pub spm_var_D: uint16,
    pub spm_var_E: uint16,
    pub spm_var_F: uint16,
    pub spm_parameter_1: uint16,
    pub spm_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub spm_var_00: uint16,
    pub spm_var_01: uint16,
    pub spm_var_02: uint16,
    pub spm_var_03: uint16,
    pub spm_var_04: uint16,
    pub spm_var_05: uint16,
    pub spm_var_06: uint16,
    pub spm_var_07: uint16,
    pub spm_var_08: uint16,
    pub spm_var_09: uint16,
    pub spm_var_0A: uint16,
    pub spm_var_0B: uint16,
    pub spm_var_0C: uint16,
    pub spm_var_0D: uint16,
    pub spm_var_0E: uint16,
    pub spm_var_0F: uint16,
    pub spm_var_10: uint16,
    pub spm_var_11: uint16,
    pub spm_var_12: uint16,
    pub spm_var_13: uint16,
    pub spm_var_14: uint16,
    pub spm_var_15: uint16,
    pub spm_var_16: uint16,
    pub spm_var_17: uint16,
    pub spm_var_18: uint16,
    pub spm_var_19: uint16,
    pub spm_var_1A: uint16,
    pub spm_var_1B: uint16,
    pub spm_var_1C: uint16,
    pub spm_var_1D: uint16,
    pub spm_var_1E: uint16,
    pub spm_var_1F: uint16,
}
pub use crate::types::Enemy_MotherBrain;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Boulder {
    pub base: EnemyBase,
    pub boulder_var_A: uint16,
    pub boulder_var_B: uint16,
    pub boulder_var_C: uint16,
    pub boulder_var_D: uint16,
    pub boulder_var_E: uint16,
    pub boulder_var_F: uint16,
    pub boulder_parameter_1: uint16,
    pub boulder_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub boulder_var_00: uint16,
    pub boulder_var_01: uint16,
    pub boulder_var_02: uint16,
    pub boulder_var_03: uint16,
    pub boulder_var_04: uint16,
    pub boulder_var_05: uint16,
    pub boulder_var_06: uint16,
    pub boulder_var_07: uint16,
    pub boulder_var_08: uint16,
    pub boulder_var_09: uint16,
    pub boulder_var_0A: uint16,
    pub boulder_var_0B: uint16,
    pub boulder_var_0C: uint16,
    pub boulder_var_0D: uint16,
    pub boulder_var_0E: uint16,
    pub boulder_var_0F: uint16,
    pub boulder_var_10: uint16,
    pub boulder_var_11: uint16,
    pub boulder_var_12: uint16,
    pub boulder_var_13: uint16,
    pub boulder_var_14: uint16,
    pub boulder_var_15: uint16,
    pub boulder_var_16: uint16,
    pub boulder_var_17: uint16,
    pub boulder_var_18: uint16,
    pub boulder_var_19: uint16,
    pub boulder_var_1A: uint16,
    pub boulder_var_1B: uint16,
    pub boulder_var_1C: uint16,
    pub boulder_var_1D: uint16,
    pub boulder_var_1E: uint16,
    pub boulder_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_CeresRidley {
    pub base: EnemyBase,
    pub cry_var_A: uint16,
    pub cry_var_B: uint16,
    pub cry_var_C: uint16,
    pub cry_var_D: uint16,
    pub cry_var_E: uint16,
    pub cry_var_F: uint16,
    pub cry_parameter_1: uint16,
    pub cry_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub cry_var_00: uint16,
    pub cry_var_01: uint16,
    pub cry_var_02: uint16,
    pub cry_var_03: uint16,
    pub cry_var_04: uint16,
    pub cry_var_05: uint16,
    pub cry_var_06: uint16,
    pub cry_var_07: uint16,
    pub cry_var_08: uint16,
    pub cry_var_09: uint16,
    pub cry_var_0A: uint16,
    pub cry_var_0B: uint16,
    pub cry_var_0C: uint16,
    pub cry_var_0D: uint16,
    pub cry_var_0E: uint16,
    pub cry_var_0F: uint16,
    pub cry_var_10: uint16,
    pub cry_var_11: uint16,
    pub cry_var_12: uint16,
    pub cry_var_13: uint16,
    pub cry_var_14: uint16,
    pub cry_var_15: uint16,
    pub cry_var_16: uint16,
    pub cry_var_17: uint16,
    pub cry_var_18: uint16,
    pub cry_var_19: uint16,
    pub cry_var_1A: uint16,
    pub cry_var_1B: uint16,
    pub cry_var_1C: uint16,
    pub cry_var_1D: uint16,
    pub cry_var_1E: uint16,
    pub cry_var_1F: uint16,
    pub pad2: [uint8; 1984],
    pub cry_var_20: uint16,
    pub cry_var_21: uint16,
    pub cry_var_22: uint16,
    pub cry_var_23: uint16,
    pub cry_var_24: uint16,
    pub cry_var_25: uint16,
    pub cry_var_26: uint16,
    pub cry_var_27: uint16,
    pub cry_var_28: uint16,
    pub cry_var_29: uint16,
    pub cry_var_2A: uint16,
    pub cry_var_2B: uint16,
    pub cry_var_2C: uint16,
    pub cry_var_2D: uint16,
    pub cry_var_2E: uint16,
    pub cry_var_2F: uint16,
    pub cry_var_30: uint16,
    pub cry_var_31: uint16,
    pub cry_var_32: uint16,
    pub cry_var_33: uint16,
    pub cry_var_34: uint16,
    pub cry_var_35: uint16,
    pub cry_var_36: uint16,
    pub cry_var_37: uint16,
    pub cry_var_38: uint16,
    pub cry_var_39: uint16,
    pub cry_var_3A: uint16,
    pub cry_var_3B: uint16,
    pub cry_var_3C: uint16,
    pub cry_var_3D: uint16,
    pub cry_var_3E: uint16,
    pub cry_var_3F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct BabyMetroidExecState {
    pub ip: uint16,
    pub timer: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ram8800_Default {
    pub var_40: uint16,
    pub var_41: uint16,
    pub var_42: uint16,
    pub var_43: uint16,
    pub var_44: uint16,
    pub var_45: uint16,
    pub var_46: uint16,
    pub var_47: uint16,
    pub var_48: uint16,
    pub var_49: uint16,
    pub var_4A: uint16,
    pub var_4B: uint16,
    pub var_4C: uint16,
    pub var_4D: uint16,
    pub var_4E: uint16,
    pub var_4F: uint16,
    pub var_50: uint16,
    pub var_51: uint16,
    pub var_52: uint16,
    pub var_53: uint16,
    pub var_54: uint16,
    pub var_55: uint16,
    pub var_56: uint16,
    pub var_57: uint16,
    pub var_58: uint16,
    pub var_59: uint16,
    pub var_5A: uint16,
    pub var_5B: uint16,
    pub var_5C: uint16,
    pub var_5D: uint16,
    pub var_5E: uint16,
    pub var_5F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_RidleysExplosion {
    pub base: EnemyBase,
    pub ren_var_A: uint16,
    pub ren_var_B: uint16,
    pub ren_var_C: uint16,
    pub ren_var_D: uint16,
    pub ren_var_E: uint16,
    pub ren_var_F: uint16,
    pub ren_parameter_1: uint16,
    pub ren_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Zebetites {
    pub base: EnemyBase,
    pub zebet_var_A: uint16,
    pub zebet_var_B: uint16,
    pub zebet_var_C: uint16,
    pub zebet_var_D: uint16,
    pub zebet_var_E: uint16,
    pub zebet_var_F: uint16,
    pub zebet_parameter_1: uint16,
    pub zebet_parameter_2: uint16,
}
pub const true_0: ::core::ffi::c_int = 1;
pub const false_0: ::core::ffi::c_int = 0;
#[inline]
unsafe fn RomPtr_A0(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0xa00000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomPtr_A6(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0xa60000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn gOamEnt(mut v: ::core::ffi::c_int, game: &mut Game) -> *mut OamEnt {
    return (&raw mut game.g_ram as *mut uint8).offset((0x370 as ::core::ffi::c_int + v) as isize)
        as *mut uint8 as *mut OamEnt;
}
#[inline]
unsafe fn gVramWriteEntry(mut a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    return (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry;
}
pub const fnBoulder_Func_1: ::core::ffi::c_int = 0xa6879a;
pub const fnBoulder_Func_2: ::core::ffi::c_int = 0xa687ed;
pub const fnBoulder_Func_3: ::core::ffi::c_int = 0xa68832;
pub const fnBoulder_Func_4: ::core::ffi::c_int = 0xa6888b;
pub const fnBoulder_Func_5: ::core::ffi::c_int = 0xa68942;
pub const fnBoulder_Func_6: ::core::ffi::c_int = 0xa689fc;
pub const fnSpikeyPlatform_Func_1: ::core::ffi::c_int = 0xa68bb4;
pub const fnSpikeyPlatform_Func_2: ::core::ffi::c_int = 0xa68bdc;
pub const fnSpikeyPlatform_Func_3: ::core::ffi::c_int = 0xa68c4a;
pub const fnSpikeyPlatform_Func_4: ::core::ffi::c_int = 0xa68c5d;
pub const fnFireGeyser_Func_1: ::core::ffi::c_int = 0xa6902f;
pub const fnFireGeyser_Func_2: ::core::ffi::c_int = 0xa69062;
pub const fnNuclearWaffle_Func_1: ::core::ffi::c_int = 0xa69615;
pub const fnNuclearWaffle_Func_2: ::core::ffi::c_int = 0xa69682;
pub const fnCeresRidley_Func_2: ::core::ffi::c_int = 0xa6a354;
pub const fnCeresRidley_Func_3: ::core::ffi::c_int = 0xa6a35b;
pub const fnCeresRidley_A377: ::core::ffi::c_int = 0xa6a377;
pub const fnCeresRidley_Func_4: ::core::ffi::c_int = 0xa6a389;
pub const fnCeresRidley_Func_5: ::core::ffi::c_int = 0xa6a3df;
pub const fnCeresRidley_Func_6: ::core::ffi::c_int = 0xa6a455;
pub const fnCeresRidley_Func_7: ::core::ffi::c_int = 0xa6a478;
pub const fnCeresRidley_Func_9: ::core::ffi::c_int = 0xa6a6af;
pub const fnCeresRidley_Func_10: ::core::ffi::c_int = 0xa6a6c8;
pub const fnCeresRidley_Func_11: ::core::ffi::c_int = 0xa6a6e8;
pub const fnCeresRidley_Func_13: uint32 = 10921858;
pub const fnCeresRidley_A7F9: uint32 = 10921977;
pub const fnCeresRidley_Func_14: uint32 = 10922044;
pub const fnCeresRidley_Func_15: ::core::ffi::c_int = 0xa6a84e;
pub const fnCeresRidley_Func_16: ::core::ffi::c_int = 0xa6a88d;
pub const fnCeresRidley_Func_17: ::core::ffi::c_int = 0xa6a8a4;
pub const fnCeresRidley_Func_18: ::core::ffi::c_int = 0xa6a8d4;
pub const fnCeresRidley_Func_19: ::core::ffi::c_int = 0xa6a8f8;
pub const fnCeresRidley_Func_20: ::core::ffi::c_int = 0xa6a923;
pub const fnCeresRidley_Func_21: ::core::ffi::c_int = 0xa6a947;
pub const fnCeresRidley_Func_22: ::core::ffi::c_int = 0xa6a971;
pub const fnCeresRidley_Func_23: ::core::ffi::c_int = 0xa6a9a0;
pub const fnCeresRidley_Func_24: ::core::ffi::c_int = 0xa6aa11;
pub const fnnullsub_233: ::core::ffi::c_int = 0xa6aa4f;
pub const fnCeresRidley_Func_26: ::core::ffi::c_int = 0xa6aa50;
pub const fnRidley_Func_3: ::core::ffi::c_int = 0xa6b2f3;
pub const fnRidley_Func_3b: ::core::ffi::c_int = 0xa6b321;
pub const fnRidley_Func_5: ::core::ffi::c_int = 0xa6b3ec;
pub const fnRidley_Func_6: ::core::ffi::c_int = 0xa6b3f8;
pub const fnRidley_Func_8: uint32 = 10925121;
pub const fnRidley_B455: uint32 = 10925141;
pub const fnRidley_Func_9: ::core::ffi::c_int = 0xa6b493;
pub const fnRidley_Func_10: ::core::ffi::c_int = 0xa6b4d1;
pub const fnRidley_Func_11: ::core::ffi::c_int = 0xa6b516;
pub const fnRidley_Func_12: ::core::ffi::c_int = 0xa6b554;
pub const fnRidley_Func_13: ::core::ffi::c_int = 0xa6b594;
pub const fnRidley_Func_14: uint32 = 10925508;
pub const fnRidley_Func_15: ::core::ffi::c_int = 0xa6b5e5;
pub const fnRidley_Func_16: ::core::ffi::c_int = 0xa6b613;
pub const fnRidley_Func_19: ::core::ffi::c_int = 0xa6b6a7;
pub const fnRidley_Func_20: ::core::ffi::c_int = 0xa6b6dd;
pub const fnRidley_Func_21: ::core::ffi::c_int = 0xa6b70e;
pub const fnRidley_Func_22: ::core::ffi::c_int = 0xa6b7b9;
pub const fnRidley_Func_33: ::core::ffi::c_int = 0xa6bab7;
pub const fnRidley_Func_33b: ::core::ffi::c_int = 0xa6bb8f;
pub const fnRidley_Func_34: ::core::ffi::c_int = 0xa6bbc4;
pub const fnRidley_Func_35: ::core::ffi::c_int = 0xa6bbf1;
pub const fnRidley_Func_36: ::core::ffi::c_int = 0xa6bc2e;
pub const fnRidley_Func_43: ::core::ffi::c_int = 0xa6bd4e;
pub const fnRidley_Func_44: ::core::ffi::c_int = 0xa6bd9a;
pub const fnRidley_Func_45: ::core::ffi::c_int = 0xa6bdbc;
pub const fnRidley_Func_46: ::core::ffi::c_int = 0xa6bdf2;
pub const fnRidley_Func_47: ::core::ffi::c_int = 0xa6be03;
pub const fnRidley_Func_50: ::core::ffi::c_int = 0xa6be9c;
pub const fnRidley_Func_51: ::core::ffi::c_int = 0xa6beb3;
pub const fnRidley_Func_52: ::core::ffi::c_int = 0xa6beca;
pub const fnRidley_Func_53: ::core::ffi::c_int = 0xa6bedc;
pub const fnnullsub_348: uint32 = 10927897;
pub const fnBabyMetroid_Instr_2: uint32 = 10928073;
pub const fnBabyMetroid_Instr_3: uint32 = 10928097;
pub const fnBabyMetroid_Instr_1: uint32 = 10928114;
pub const fnBabyMetroid_Instr_4: uint32 = 10928120;
pub const fnRidley_Func_54: ::core::ffi::c_int = 0xa6c04e;
pub const fnRidley_Func_63: ::core::ffi::c_int = 0xa6c538;
pub const fnRidley_C53E: ::core::ffi::c_int = 0xa6c53e;
pub const fnRidley_C551: ::core::ffi::c_int = 0xa6c551;
pub const fnRidley_Func_64: ::core::ffi::c_int = 0xa6c588;
pub const fnRidley_Func_65: ::core::ffi::c_int = 0xa6c5a8;
pub const fnRidley_Func_66: ::core::ffi::c_int = 0xa6c5c8;
pub const fnRidley_Func_67: ::core::ffi::c_int = 0xa6c5da;
pub const fnnullsub_349: uint32 = 10929664;
pub const fnnullsub_350: uint32 = 10940404;
pub const fnCeresSteam_Func_1: uint32 = 10940441;
pub const fnCeresDoor_Func_2: uint32 = 10942315;
pub const fnCeresDoor_Func_3: uint32 = 10942320;
pub const fnCeresDoor_Func_4: uint32 = 10942373;
pub const fnCeresDoor_Func_5: uint32 = 10942397;
pub const fnCeresDoor_Func_6: ::core::ffi::c_int = 0xa6f7dc;
pub const fnCeresDoor_Func_7: ::core::ffi::c_int = 0xa6f850;
pub const fnZebetites_Func_1: ::core::ffi::c_int = 0xa6fc41;
pub const fnZebetites_Func_2: ::core::ffi::c_int = 0xa6fc5b;
pub const fnZebetites_Func_3: ::core::ffi::c_int = 0xa6fc67;
#[inline]
unsafe fn gEnemyData(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemyData {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut EnemyData;
}
#[inline]
unsafe fn gEnemySpawnData(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemySpawnData {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0x7000)
        .offset(o as isize) as *mut EnemySpawnData;
}
#[inline]
unsafe fn gRam8800_Default(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Ram8800_Default {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0x8800)
        .offset(o as isize) as *mut Ram8800_Default;
}
#[inline]
unsafe fn Get_Boulder(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Boulder {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Boulder;
}
#[inline]
unsafe fn Get_CeresDoor(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_CeresDoor {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_CeresDoor;
}
#[inline]
unsafe fn Get_CeresRidley(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_CeresRidley {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_CeresRidley;
}
#[inline]
unsafe fn Get_CeresSteam(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_CeresSteam {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_CeresSteam;
}
#[inline]
unsafe fn Get_FakeKraid(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_FakeKraid {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_FakeKraid;
}
#[inline]
unsafe fn Get_FireGeyser(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_FireGeyser {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_FireGeyser;
}
#[inline]
unsafe fn Get_MotherBrain(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_MotherBrain {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MotherBrain;
}
#[inline]
unsafe fn Get_NuclearWaffle(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_NuclearWaffle {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_NuclearWaffle;
}
#[inline]
unsafe fn Get_Ridley(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Ridley {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Ridley;
}
#[inline]
unsafe fn Get_RidleysExplosion(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_RidleysExplosion {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_RidleysExplosion;
}
#[inline]
unsafe fn Get_SpikeyPlatform(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_SpikeyPlatform {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_SpikeyPlatform;
}
#[inline]
unsafe fn Get_Zebetites(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Zebetites {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Zebetites;
}
static g_word_A686F1: [uint16; 2] = [
    0x1000 as ::core::ffi::c_int as uint16,
    0x1800 as ::core::ffi::c_int as uint16,
];
static g_word_A68DBB: uint16 = 5 as uint16;
static g_word_A68DBD: uint16 = 0xa as uint16;
static g_word_A68DBF: uint16 = 0xf as uint16;
static g_word_A68DC1: uint16 = 0x14 as uint16;
static g_word_A68DC3: uint16 = 0x19 as uint16;
static g_word_A68DC5: uint16 = 0x1e as uint16;
static g_word_A68DC7: uint16 = 0x23 as uint16;
static g_word_A68DC9: uint16 = 0x28 as uint16;
static g_word_A68DCB: uint16 = 0x2d as uint16;
static g_word_A68DCD: uint16 = 0x32 as uint16;
static g_word_A68DCF: uint16 = 0x37 as uint16;
static g_word_A68DD1: uint16 = 0x3c as uint16;
static g_word_A68DD3: uint16 = 0x41 as uint16;
static g_word_A68DD5: uint16 = 0x46 as uint16;
static g_word_A68DD7: uint16 = 0x4b as uint16;
static g_word_A68DD9: uint16 = 0x50 as uint16;
static g_word_A68DDB: uint16 = 0x55 as uint16;
static g_word_A68DDD: uint16 = 0x5a as uint16;
static g_word_A68DDF: uint16 = 0x5f as uint16;
static g_word_A68DE1: uint16 = 0x64 as uint16;
static g_word_A68DE3: uint16 = 0x69 as uint16;
static g_word_A68DE5: uint16 = 0x6e as uint16;
static g_word_A68DE7: uint16 = 0x18 as uint16;
static g_word_A68DE9: uint16 = 0x18 as uint16;
static g_word_A68DEB: uint16 = 0x18 as uint16;
static g_word_A68DED: uint16 = 0x18 as uint16;
static g_word_A68DEF: uint16 = 0x18 as uint16;
static g_word_A68DF1: uint16 = 0x18 as uint16;
static g_word_A68DF3: uint16 = 0x18 as uint16;
static g_word_A68DF5: uint16 = 0x18 as uint16;
static g_word_A68DF7: uint16 = 0x18 as uint16;
static g_word_A68DF9: uint16 = 0x18 as uint16;
static g_word_A68DFB: uint16 = 0x18 as uint16;
static g_word_A68DFD: uint16 = 0x18 as uint16;
static g_word_A68DFF: uint16 = 0x18 as uint16;
static g_word_A68E01: uint16 = 0x18 as uint16;
static g_word_A68E03: uint16 = 0x18 as uint16;
static g_word_A68E05: uint16 = 0x18 as uint16;
static g_word_A68E07: uint16 = 0x18 as uint16;
static g_word_A68E09: uint16 = 0x18 as uint16;
static g_word_A68E0B: uint16 = 0x14 as uint16;
static g_word_A68E0D: uint16 = 0x10 as uint16;
static g_word_A68E0F: uint16 = 0xc as uint16;
static g_word_A68E11: uint16 = 8 as uint16;
static g_word_A695F6: [uint16; 4] = [
    0x190 as ::core::ffi::c_int as uint16,
    0xf0 as ::core::ffi::c_int as uint16,
    0xf0 as ::core::ffi::c_int as uint16,
    0x190 as ::core::ffi::c_int as uint16,
];
static g_word_A695FE: [uint16; 4] = [
    0xffe8 as ::core::ffi::c_int as uint16,
    0xfff4 as ::core::ffi::c_int as uint16,
    0x18 as ::core::ffi::c_int as uint16,
    0xc as ::core::ffi::c_int as uint16,
];
static g_word_A69606: [uint16; 4] = [
    0x180 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
    0x180 as ::core::ffi::c_int as uint16,
];
static g_word_A69A48: [uint16; 8] = [
    0xfe00 as ::core::ffi::c_int as uint16,
    0xfb00 as ::core::ffi::c_int as uint16,
    0xfc00 as ::core::ffi::c_int as uint16,
    0xfb00 as ::core::ffi::c_int as uint16,
    0x200 as ::core::ffi::c_int as uint16,
    0xfb00 as ::core::ffi::c_int as uint16,
    0x400 as ::core::ffi::c_int as uint16,
    0xfb00 as ::core::ffi::c_int as uint16,
];
static g_word_A6B288: uint16 = 8 as uint16;
static g_word_A6BBEB: [uint16; 3] = [
    0x40 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0xd0 as ::core::ffi::c_int as uint16,
];
static g_word_A6BC62: [uint16; 3] = [
    0xb0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x50 as ::core::ffi::c_int as uint16,
];
static g_word_A6C66E: [int16; 20] = [
    -(24 as ::core::ffi::c_int) as int16,
    -(24 as ::core::ffi::c_int) as int16,
    -(20 as ::core::ffi::c_int) as int16,
    20 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    -(30 as ::core::ffi::c_int) as int16,
    30 as ::core::ffi::c_int as int16,
    -(3 as ::core::ffi::c_int) as int16,
    14 as ::core::ffi::c_int as int16,
    -(13 as ::core::ffi::c_int) as int16,
    -(2 as ::core::ffi::c_int) as int16,
    18 as ::core::ffi::c_int as int16,
    -(2 as ::core::ffi::c_int) as int16,
    -(32 as ::core::ffi::c_int) as int16,
    -(31 as ::core::ffi::c_int) as int16,
    8 as ::core::ffi::c_int as int16,
    -(4 as ::core::ffi::c_int) as int16,
    -(10 as ::core::ffi::c_int) as int16,
    19 as ::core::ffi::c_int as int16,
    19 as ::core::ffi::c_int as int16,
];
static g_word_A6CF54: [int16; 3] = [
    0x20 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(0x20 as ::core::ffi::c_int) as int16,
];
static g_byte_A6D61F: [uint8; 16] = [
    0x10 as ::core::ffi::c_int as uint8,
    0xf as ::core::ffi::c_int as uint8,
    0xe as ::core::ffi::c_int as uint8,
    0xd as ::core::ffi::c_int as uint8,
    0xc as ::core::ffi::c_int as uint8,
    0xb as ::core::ffi::c_int as uint8,
    0xa as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    7 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
];
static g_byte_A6D712: [uint8; 16] = [
    0x10 as ::core::ffi::c_int as uint8,
    0xf as ::core::ffi::c_int as uint8,
    0xe as ::core::ffi::c_int as uint8,
    0xd as ::core::ffi::c_int as uint8,
    0xc as ::core::ffi::c_int as uint8,
    0xb as ::core::ffi::c_int as uint8,
    0xa as ::core::ffi::c_int as uint8,
    9 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    7 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
];
static g_word_A6D9ED: [uint16; 8] = [
    0xc as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0x10 as ::core::ffi::c_int as uint16,
    0x12 as ::core::ffi::c_int as uint16,
    0x1c as ::core::ffi::c_int as uint16,
    0x20 as ::core::ffi::c_int as uint16,
    0x28 as ::core::ffi::c_int as uint16,
    0x30 as ::core::ffi::c_int as uint16,
];
pub unsafe fn Enemy_GrappleReact_NoInteract_A6(game: &mut Game) {
    SwitchEnemyAiToMainAi(game);
}
pub unsafe fn Enemy_GrappleReact_CancelBeam_A6(game: &mut Game) {
    Enemy_SwitchToFrozenAi(game);
}
pub unsafe fn Enemy_NormalTouchAI_A6(game: &mut Game) {
    NormalEnemyTouchAi(game);
}
pub unsafe fn Enemy_NormalShotAI_A6(game: &mut Game) {
    NormalEnemyShotAi(game);
}
pub unsafe fn Enemy_NormalFrozenAI_A6(game: &mut Game) {
    NormalEnemyFrozenAI(game);
}
pub unsafe fn Boulder_Init(game: &mut Game) {
    let mut boulder_parameter_2_high: int16 = 0;
    let mut E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).boulder_var_C = 0 as uint16;
    (*E).boulder_var_B = 0 as uint16;
    (*E).boulder_var_D = 2 as uint16;
    (*E).boulder_var_A = (0xa6879a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).boulder_var_04 = (*E).base.x_pos;
    let mut y_pos: uint16 = (*E).base.y_pos;
    (*E).boulder_var_05 = y_pos;
    (*E).boulder_var_03 = y_pos;
    (*E).boulder_var_02 = (*E).base.y_subpos;
    boulder_parameter_2_high = *(&raw mut (*E).boulder_parameter_2 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as int16;
    if *(&raw mut (*E).boulder_parameter_2 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) == 0
    {
        boulder_parameter_2_high = 1 as int16;
        (*E).boulder_var_07 = 1 as uint16;
    }
    (*E).base.y_pos = ((*E).boulder_var_05 as ::core::ffi::c_int
        - boulder_parameter_2_high as ::core::ffi::c_int) as uint16;
    (*E).boulder_var_00 = -(*(&raw mut (*E).boulder_parameter_2 as *mut uint8).offset(0)
        as ::core::ffi::c_int) as uint16;
    (*E).boulder_var_06 =
        *(&raw mut (*E).base.current_instruction as *mut uint8).offset(0) as uint16;
    (*E).base.current_instruction = addr_kBoulder_Ilist_86A7 as ::core::ffi::c_int as uint16;
    let mut boulder_parameter_1_high: uint16 = *(&raw mut (*E).boulder_parameter_1 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as uint16;
    (*E).boulder_var_E = boulder_parameter_1_high;
    if boulder_parameter_1_high == 0 {
        (*E).boulder_var_00 = -((*E).boulder_var_00 as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction = addr_kBoulder_Ilist_86CB as ::core::ffi::c_int as uint16;
    }
    (*E).boulder_var_01 = 2 as uint16;
    if *(&raw mut (*E).boulder_parameter_1 as *mut uint8).offset(0) != 0 {
        (*E).boulder_var_01 = 5 as uint16;
    }
}
pub unsafe fn CallBoulderFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        10913690 => {
            Boulder_Func_1(game);
            return;
        }
        10913773 => {
            Boulder_Func_2(game);
            return;
        }
        10913842 => {
            Boulder_Func_3(game);
            return;
        }
        10913931 => {
            Boulder_Func_4(game);
            return;
        }
        10914114 => {
            Boulder_Func_5(game);
            return;
        }
        10914300 => {
            Boulder_Func_6();
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn Boulder_Main(game: &mut Game) {
    let mut E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallBoulderFunc(
        ((*E).boulder_var_A as ::core::ffi::c_int | 0xa60000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn Boulder_Func_1(game: &mut Game) {
    let mut SamusEnemyDelta_Y: int16 = 0;
    let mut v3: int16 = 0;
    let mut SamusEnemyDelta_X: int16 = 0;
    SamusEnemyDelta_Y = GetSamusEnemyDelta_Y(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) as int16;
    if SamusEnemyDelta_Y as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        let mut E: *mut Enemy_Boulder = Get_Boulder(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        if ((SamusEnemyDelta_Y as ::core::ffi::c_int - (*E).boulder_var_06 as ::core::ffi::c_int)
            as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            if (*E).boulder_var_E != 0 {
                SamusEnemyDelta_X = GetSamusEnemyDelta_X(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                    game,
                ) as int16;
                if (SamusEnemyDelta_X as ::core::ffi::c_int) < 0 as ::core::ffi::c_int
                    && (SamusEnemyDelta_X as ::core::ffi::c_int
                        - (*E).boulder_var_00 as ::core::ffi::c_int) as int16
                        as ::core::ffi::c_int
                        >= 0 as ::core::ffi::c_int
                {
                    (*E).boulder_var_A =
                        (0xa687ed as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                    if (*E).boulder_var_07 != 0 {
                        (*E).boulder_var_A = (0xa68942 as ::core::ffi::c_int
                            & 0xffff as ::core::ffi::c_int)
                            as uint16;
                    }
                }
            } else {
                v3 = GetSamusEnemyDelta_X(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                    game,
                ) as int16;
                if v3 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int
                    && ((v3 as ::core::ffi::c_int - (*E).boulder_var_00 as ::core::ffi::c_int)
                        as int16 as ::core::ffi::c_int)
                        < 0 as ::core::ffi::c_int
                {
                    (*E).boulder_var_A =
                        (0xa687ed as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                    if (*E).boulder_var_07 != 0 {
                        (*E).boulder_var_A = (0xa68942 as ::core::ffi::c_int
                            & 0xffff as ::core::ffi::c_int)
                            as uint16;
                    }
                }
            }
        }
    }
}
pub unsafe fn Boulder_Func_2(game: &mut Game) {
    let mut E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    Boulder_Func_9(
        (8 as ::core::ffi::c_int
            * *(&raw mut (*E).boulder_var_C as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int) as uint16,
        game,
    );
    if (((*E).base.y_pos as ::core::ffi::c_int - (*E).boulder_var_05 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        let mut v1: uint16 =
            ((*E).boulder_var_C as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as uint16;
        (*E).boulder_var_C = v1;
        if v1 as ::core::ffi::c_int - 20480 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).boulder_var_C = 20480 as uint16;
        }
    } else {
        (*E).base.y_pos = (*E).boulder_var_05;
        (*E).boulder_var_A =
            (0xa68832 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).boulder_var_C = 0x2000 as uint16;
    };
}
pub unsafe fn Boulder_Func_3(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    Boulder_Func_9(
        (8 as ::core::ffi::c_int
            * *(&raw mut (*E).boulder_var_C as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            + 4 as ::core::ffi::c_int) as uint16,
        game,
    );
    v1 = ((*E).boulder_var_C as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as int16;
    (*E).boulder_var_C = v1 as uint16;
    if v1 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        let mut v2: uint16 = (8 as ::core::ffi::c_int
            * *(&raw mut (*E).boulder_var_B as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int) as uint16;
        if (*E).boulder_var_E != 0 {
            v2 = (v2 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
        }
        Boulder_Func_7(v2, game);
        let mut v3: uint16 =
            ((*E).boulder_var_B as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
        (*E).boulder_var_B = v3;
        if v3 as ::core::ffi::c_int - 20480 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).boulder_var_B = 20480 as uint16;
        }
    } else {
        (*E).boulder_var_C = 0 as uint16;
        (*E).boulder_var_A =
            (0xa6888b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Boulder_Func_4(game: &mut Game) {
    let mut E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v1: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * *(&raw mut (*E).boulder_var_C as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        *(game.g_rom.offset(
            (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint32)
            .offset((v1 >> 1 as ::core::ffi::c_int) as isize) as int32,
        game,
    ) != 0
    {
        QueueSfx2_Max6(0x42 as uint16, game);
        if (*E).boulder_var_E as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                | kEnemyProps_Deleted as ::core::ffi::c_int)
                as uint16;
            game.eproj_spawn_pt = Point16U {
                x: (*E).base.x_pos,
                y: (*E).base.y_pos,
            };
            SpawnEprojWithRoomGfx(
                addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
                0x11 as uint16,
                game,
            );
            QueueSfx2_Max6(0x43 as uint16, game);
        } else {
            (*E).boulder_var_A =
                (0xa68832 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).boulder_var_C = g_word_A686F1
                [((*E).boulder_var_D as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize];
            (*E).boulder_var_D = (*E).boulder_var_D.wrapping_sub(1);
            if (*E).boulder_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                (*E).boulder_var_03 = (*E).base.y_pos;
                (*E).boulder_var_02 = (*E).base.y_subpos;
                (*E).boulder_var_A =
                    (0xa68942 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            }
        }
    } else {
        (*E).boulder_var_C =
            ((*E).boulder_var_C as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as uint16;
        let mut v3: uint16 = (8 as ::core::ffi::c_int
            * *(&raw mut (*E).boulder_var_B as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int) as uint16;
        if (*E).boulder_var_E != 0 {
            v3 = (v3 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
        }
        Boulder_Func_7(v3, game);
        let mut v4: uint16 =
            ((*E).boulder_var_B as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
        (*E).boulder_var_B = v4;
        if v4 as ::core::ffi::c_int - 20480 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).boulder_var_B = 20480 as uint16;
        }
    };
}
pub unsafe fn Boulder_Func_5(game: &mut Game) {
    let mut E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: uint16 = (8 as ::core::ffi::c_int
        * *(&raw mut (*E).boulder_var_B as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int) as uint16;
    let mut v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (*(game.g_rom.offset(
            (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint32)
            .offset((v3 >> 1 as ::core::ffi::c_int) as isize))
        .wrapping_add(
            (((*E).boulder_var_01 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as uint32,
        ) as int32,
        game,
    );
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        - (*E).boulder_var_01 as ::core::ffi::c_int) as uint16;
    if (*E).boulder_var_E != 0 {
        v2 = (v2 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    }
    let mut v4: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        *(game.g_rom.offset(
            (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint32)
            .offset((v4 >> 1 as ::core::ffi::c_int) as isize) as int32,
        game,
    ) != 0
    {
        (*E).base.properties =
            ((*E).base.properties as ::core::ffi::c_int | 0x300 as ::core::ffi::c_int) as uint16;
        (*E).boulder_var_A =
            (0xa689fc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        QueueSfx2_Max6(0x42 as uint16, game);
        game.eproj_spawn_pt = Point16U {
            x: (*E).base.x_pos,
            y: (*E).base.y_pos,
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
            0x11 as uint16,
            game,
        );
        QueueSfx2_Max6(0x43 as uint16, game);
    } else {
        let mut v5: uint16 =
            ((*E).boulder_var_B as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
        (*E).boulder_var_B = v5;
        if v5 as ::core::ffi::c_int - 0x4000 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).boulder_var_B = 0x4000 as uint16;
        }
        if (*E).base.y_pos as ::core::ffi::c_int == (*E).boulder_var_03 as ::core::ffi::c_int
            && (*E).base.y_subpos as ::core::ffi::c_int == (*E).boulder_var_02 as ::core::ffi::c_int
        {
            (*E).boulder_var_01 = 0 as uint16;
        }
    }
    (*E).boulder_var_03 = (*E).base.y_pos;
    (*E).boulder_var_02 = (*E).base.y_subpos;
}
pub unsafe fn Boulder_Func_6() {}
pub unsafe fn Boulder_Func_7(mut j: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (*(game.g_rom.offset(
            (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v3 + 1 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(game.g_rom.offset(
                (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v3 as isize) as uint32,
    );
}
pub unsafe fn Boulder_Func_8(mut j: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (*(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v3 + 1 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(game.g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v3 as isize) as uint32,
    );
}
pub unsafe fn Boulder_Func_9(mut j: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(game.g_rom.offset(
            (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v3 + 1 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(game.g_rom.offset(
                (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v3 as isize) as uint32,
    );
}
pub unsafe fn SpikeyPlatform_Init(game: &mut Game) {
    let mut E: *mut Enemy_SpikeyPlatform = Get_SpikeyPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction = addr_kSpikeyPlatform_Ilist_8B29 as ::core::ffi::c_int as uint16;
    (*E).spm_var_A = (0xa68bb4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 = (8 as ::core::ffi::c_int
        * *(&raw mut (*E).spm_parameter_1 as *mut uint8).offset(0) as ::core::ffi::c_int)
        as uint16;
    (*E).spm_var_02 = v1;
    let mut v2: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).spm_var_E = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v2 + 1 as ::core::ffi::c_int) as isize);
    (*E).spm_var_F = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v2 as isize);
    (*E).spm_var_D = ((*E).base.y_pos as ::core::ffi::c_int
        + *(&raw mut (*E).spm_parameter_2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int) as uint16;
    (*E).spm_var_C = (*E).base.y_pos;
    let mut spm_parameter_2_low: uint16 =
        *(&raw mut (*E).spm_parameter_2 as *mut uint8).offset(0) as uint16;
    (*E).spm_var_00 = spm_parameter_2_low;
    (*E).spm_var_B = spm_parameter_2_low;
}
pub unsafe fn SpikeyPlatform2ndEnemy_Init(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xe54)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).x_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v0 + 91 as ::core::ffi::c_int) as isize);
    (*v1).y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v0 + 93 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
        + 12 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn SpikeyPlatform2ndEnemy_Main(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xe54)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).x_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v0 + 91 as ::core::ffi::c_int) as isize);
    (*v1).y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v0 + 93 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
        + 12 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CallSpikeyPlatformFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        10914740 => {
            SpikeyPlatform_Func_1(game);
            return;
        }
        10914780 => {
            SpikeyPlatform_Func_2(game);
            return;
        }
        10914890 => {
            SpikeyPlatform_Func_3(k, game);
            return;
        }
        10914909 => {
            SpikeyPlatform_Func_4(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn SpikeyPlatform_Main(game: &mut Game) {
    let mut E: *mut Enemy_SpikeyPlatform = Get_SpikeyPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallSpikeyPlatformFunc(
        ((*E).spm_var_A as ::core::ffi::c_int | 0xa60000 as ::core::ffi::c_int) as uint32,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn SpikeyPlatform_Func_1(game: &mut Game) {
    let mut E: *mut Enemy_SpikeyPlatform = Get_SpikeyPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).spm_var_B = (*E).spm_var_B.wrapping_sub(1);
    if (*E).spm_var_B == 0 {
        (*E).spm_var_B = (*E).spm_var_00;
        (*E).spm_var_02 = (8 as ::core::ffi::c_int
            * *(&raw mut (*E).spm_parameter_1 as *mut uint8).offset(0) as ::core::ffi::c_int)
            as uint16;
        (*E).spm_var_A = (0xa68bdc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn SpikeyPlatform_Func_2(game: &mut Game) {
    let mut E: *mut Enemy_SpikeyPlatform = Get_SpikeyPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).spm_var_01 = (*E).base.y_pos;
    let mut v1: ::core::ffi::c_int =
        (*E).spm_var_02 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v1 as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(game.g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v1 + 1 as ::core::ffi::c_int) as isize) as uint32,
    );
    if ((*E).base.y_pos as ::core::ffi::c_int - (*E).spm_var_D as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).spm_var_03 = 64 as uint16;
        (*E).spm_var_A = (0xa68c4a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.y_pos = (*E).spm_var_D;
        QueueSfx2_Max6(0x1b as uint16, game);
    }
    if SpikeyPlatform_Func_5(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        let ref mut fresh17 = *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16);
        *fresh17 = (*fresh17 as ::core::ffi::c_int
            + ((*E).base.y_pos as ::core::ffi::c_int - (*E).spm_var_01 as ::core::ffi::c_int))
            as uint16;
    }
    let mut v4: int16 = ((*E).spm_var_02 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as int16;
    if (*E).spm_var_02 as ::core::ffi::c_int - 504 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        v4 = 512 as int16;
    }
    (*E).spm_var_02 = v4 as uint16;
}
pub unsafe fn SpikeyPlatform_Func_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_SpikeyPlatform = Get_SpikeyPlatform(k as ::core::ffi::c_int, game);
    let mut v2: uint16 =
        ((*E).spm_var_03 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E).spm_var_03 = v2;
    if v2 == 0 {
        (*E).spm_var_A = (0xa68c5d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn SpikeyPlatform_Func_4(game: &mut Game) {
    let mut E: *mut Enemy_SpikeyPlatform = Get_SpikeyPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).spm_var_01 = (*E).base.y_pos;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        -(0x8000 as ::core::ffi::c_int) as uint32,
    );
    if (((*E).base.y_pos as ::core::ffi::c_int - (*E).spm_var_C as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        (*E).spm_var_A = (0xa68bb4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.y_pos = (*E).spm_var_C;
    }
    if SpikeyPlatform_Func_5(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        let ref mut fresh18 = *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16);
        *fresh18 = (*fresh18 as ::core::ffi::c_int
            + ((*E).base.y_pos as ::core::ffi::c_int - (*E).spm_var_01 as ::core::ffi::c_int))
            as uint16;
    }
}
pub unsafe fn SpikeyPlatform_Func_5(mut k: uint16, game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_SpikeyPlatform = Get_SpikeyPlatform(k as ::core::ffi::c_int, game);
    let mut v2: uint16 = abs16(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
    );
    let mut v3: bool = (v2 as ::core::ffi::c_int)
        < *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16) as ::core::ffi::c_int;
    let mut v4: uint16 = (v2 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    if !v3 && v4 as ::core::ffi::c_int >= (*E).base.x_width as ::core::ffi::c_int {
        return 0 as uint16;
    }
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        + 5 as ::core::ffi::c_int
        - (*E).base.y_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        let mut v6: uint16 = ((*E).base.y_pos as ::core::ffi::c_int
            - (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                + 5 as ::core::ffi::c_int)) as uint16;
        v3 = (v6 as ::core::ffi::c_int)
            < *((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16)
                as ::core::ffi::c_int;
        let mut v7: uint16 = (v6 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
        if v3 as ::core::ffi::c_int != 0
            || v7 as ::core::ffi::c_int == (*E).base.y_height as ::core::ffi::c_int
            || (v7 as ::core::ffi::c_int) < (*E).base.y_height as ::core::ffi::c_int
        {
            return -(1 as ::core::ffi::c_int) as uint16;
        }
    }
    return 0 as uint16;
}
pub unsafe fn FireGeyser_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x61 as uint16, game);
    return jp;
}
pub unsafe fn FireGeyser_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DBB as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DE7;
    (*E).base.x_width = 8 as uint16;
    return jp;
}
pub unsafe fn FireGeyser_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DBD as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DE9;
    return jp;
}
pub unsafe fn FireGeyser_Instr_4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DBF as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DEB;
    return jp;
}
pub unsafe fn FireGeyser_Instr_5(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DC1 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DED;
    return jp;
}
pub unsafe fn FireGeyser_Instr_6(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DC3 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DEF;
    return jp;
}
pub unsafe fn FireGeyser_Instr_7(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DC5 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DF1;
    return jp;
}
pub unsafe fn FireGeyser_Instr_8(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DC7 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DF3;
    return jp;
}
pub unsafe fn FireGeyser_Instr_9(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DC9 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DF5;
    return jp;
}
pub unsafe fn FireGeyser_Instr_10(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DCB as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DF7;
    return jp;
}
pub unsafe fn FireGeyser_Instr_11(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DCD as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DF9;
    return jp;
}
pub unsafe fn FireGeyser_Instr_12(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DCF as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DFB;
    return jp;
}
pub unsafe fn FireGeyser_Instr_13(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DD1 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DFD;
    return jp;
}
pub unsafe fn FireGeyser_Instr_14(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DD3 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DFF;
    return jp;
}
pub unsafe fn FireGeyser_Instr_15(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DD5 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E01;
    return jp;
}
pub unsafe fn FireGeyser_Instr_16(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DD7 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E03;
    return jp;
}
pub unsafe fn FireGeyser_Instr_17(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DD9 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E05;
    return jp;
}
pub unsafe fn FireGeyser_Instr_18(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DDB as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E07;
    return jp;
}
pub unsafe fn FireGeyser_Instr_19(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DDD as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E09;
    return jp;
}
pub unsafe fn FireGeyser_Instr_20(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DDF as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E0B;
    return jp;
}
pub unsafe fn FireGeyser_Instr_21(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DE1 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E0D;
    return jp;
}
pub unsafe fn FireGeyser_Instr_22(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DE3 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E0F;
    return jp;
}
pub unsafe fn FireGeyser_Instr_23(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D as ::core::ffi::c_int
        - g_word_A68DE5 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E11;
    return jp;
}
pub unsafe fn FireGeyser_Instr_24(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).fgr_var_C = 1 as uint16;
    let mut E1: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E1).base.x_width = 0 as uint16;
    (*E1).base.y_height = 0 as uint16;
    (*E).base.y_pos = (*E).fgr_var_D;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
    (*E1).base.properties =
        ((*E1).base.properties as ::core::ffi::c_int | 0x400 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn FireGeyser_Init(game: &mut Game) {
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction = addr_kFireGeyser_Ilist_8DA9 as ::core::ffi::c_int as uint16;
    if (*E).fgr_parameter_2 == 0 {
        (*E).base.current_instruction = addr_kFireGeyser_Ilist_8D1B as ::core::ffi::c_int as uint16;
        (*E).fgr_var_A = (0xa6902f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).fgr_var_D = (*E).base.y_pos;
        (*E).base.x_width = 0 as uint16;
    }
}
pub unsafe fn CallFireGeyserFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        10915887 => {
            FireGeyser_Func_1(game);
            return;
        }
        10915938 => {
            FireGeyser_Func_2(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn FireGeyser_Main(game: &mut Game) {
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).fgr_parameter_2 == 0 {
        CallFireGeyserFunc(
            ((*E).fgr_var_A as ::core::ffi::c_int | 0xa60000 as ::core::ffi::c_int) as uint32,
            game,
        );
    }
}
pub unsafe fn FireGeyser_Func_1(game: &mut Game) {
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).fgr_var_B = (*E).fgr_var_B.wrapping_sub(1);
    if (*E).fgr_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).fgr_var_A = (0xa69062 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).fgr_var_C = 0 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction = addr_kFireGeyser_Ilist_8D1B as ::core::ffi::c_int as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_Invisible as ::core::ffi::c_int))
            as uint16;
        let mut E1: *mut Enemy_FireGeyser = Get_FireGeyser(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
                + 64 as ::core::ffi::c_int,
            game,
        );
        (*E1).base.properties = ((*E1).base.properties as ::core::ffi::c_int
            & !(0x400 as ::core::ffi::c_int)) as uint16;
    }
}
pub unsafe fn FireGeyser_Func_2(game: &mut Game) {
    let mut E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).fgr_var_C != 0 {
        (*E).fgr_var_B = (*E).fgr_parameter_1;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
        (*E).fgr_var_A = (0xa6902f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn NuclearWaffle_Init(game: &mut Game) {
    let mut E: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction = addr_kNuclearWaffle_Ilist_9490 as ::core::ffi::c_int as uint16;
    (*E).nwe_var_C = *(&raw mut (*E).nwe_parameter_1 as *mut uint8).offset(0) as uint16;
    (*E).nwe_var_D = *(&raw mut (*E).nwe_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as uint16;
    (*E).nwe_var_E = *(&raw mut (*E).nwe_parameter_2 as *mut uint8).offset(0) as uint16;
    let mut nwe_parameter_2_high: uint16 = *(&raw mut (*E).nwe_parameter_2 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as uint16;
    (*E).nwe_var_F = nwe_parameter_2_high;
    (*E).nwe_var_B = nwe_parameter_2_high;
    (*E).nwe_var_A = (0xa69615 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut v2: ::core::ffi::c_int = (4 as ::core::ffi::c_int
        * (*E).nwe_var_E as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v3: uint16 = g_word_A695F6[v2 as usize];
    (*E).nwe_var_23 = v3;
    (*E).nwe_var_21 = v3;
    (*E).nwe_var_2E = g_word_A695F6[(v2 + 1 as ::core::ffi::c_int) as usize];
    (*E).nwe_var_2B = g_word_A695FE[v2 as usize];
    (*E).nwe_var_2C = g_word_A695FE[(v2 + 1 as ::core::ffi::c_int) as usize];
    (*E).nwe_var_30 = g_word_A69606[v2 as usize];
    (*E).nwe_var_2F = g_word_A69606[(v2 + 1 as ::core::ffi::c_int) as usize];
    let mut v4: uint16 = (8 as ::core::ffi::c_int * (*E).nwe_var_C as ::core::ffi::c_int) as uint16;
    if (*E).nwe_var_E == 0 {
        v4 = (v4 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    }
    let mut v5: ::core::ffi::c_int = v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).nwe_var_25 = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v5 as isize);
    (*E).nwe_var_24 = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v5 + 1 as ::core::ffi::c_int) as isize);
    (*E).nwe_var_26 = (*E).base.x_pos;
    (*E).nwe_var_27 = (*E).base.y_pos;
    let mut v6: uint16 = ((*E).nwe_var_26 as ::core::ffi::c_int
        + CosineMult8bit((*E).nwe_var_23, (*E).nwe_var_D, game) as ::core::ffi::c_int)
        as uint16;
    (*E).nwe_var_28 = v6;
    (*E).base.x_pos = v6;
    let mut v7: uint16 = ((*E).nwe_var_27 as ::core::ffi::c_int
        + SineMult8bit((*E).nwe_var_23, (*E).nwe_var_D, game) as ::core::ffi::c_int)
        as uint16;
    (*E).nwe_var_29 = v7;
    (*E).base.y_pos = v7;
    let mut v8: uint16 = 8 as uint16;
    (*E).nwe_var_2A = 8 as uint16;
    loop {
        SpawnEprojWithGfx(
            v8,
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            addr_kEproj_NuclearWaffleBody as ::core::ffi::c_int as uint16,
            game,
        );
        v8 = ((*E).nwe_var_2A as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        (*E).nwe_var_2A = v8;
        if !(v8 != 0) {
            break;
        }
    }
    (*E).nwe_var_2A = 6 as uint16;
    let mut v15: uint16 = 0;
    loop {
        let mut v13: uint16 = ((*E).base.vram_tiles_index as ::core::ffi::c_int
            | (*E).base.palette_index as ::core::ffi::c_int)
            as uint16;
        (*E).nwe_var_34 = v13;
        let mut r18: uint16 =
            CreateSpriteAtPos((*E).base.x_pos, (*E).base.y_pos, 43 as uint16, v13, game);
        (*Get_NuclearWaffle(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
                + (*E).nwe_var_2A as ::core::ffi::c_int,
            game,
        ))
        .nwe_var_10 = r18;
        v15 = ((*E).nwe_var_2A as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        (*E).nwe_var_2A = v15;
        if !(v15 != 0) {
            break;
        }
    }
}
pub unsafe fn CallNuclearWaffleFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        10917397 => {
            NuclearWaffle_Func_1(game);
            return;
        }
        10917506 => {
            NuclearWaffle_Func_2(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn NuclearWaffle_Main(game: &mut Game) {
    let mut E: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallNuclearWaffleFunc(
        ((*E).nwe_var_A as ::core::ffi::c_int | 0xa60000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn NuclearWaffle_Func_1(game: &mut Game) {
    let mut E: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).nwe_var_B = (*E).nwe_var_B.wrapping_sub(1);
    if (*E).nwe_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).nwe_var_B = (*E).nwe_var_F;
        (*E).nwe_var_21 = (*E).nwe_var_23;
        (*E).nwe_var_A = (0xa69682 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).nwe_var_33 = 0 as uint16;
        (*E).nwe_var_08 = 0 as uint16;
        (*E).nwe_var_09 = 0 as uint16;
        (*E).nwe_var_0A = 0 as uint16;
        (*E).nwe_var_0B = 0 as uint16;
        (*E).nwe_var_0C = 0 as uint16;
        (*E).nwe_var_0D = 0 as uint16;
        (*E).nwe_var_0E = 0 as uint16;
        (*E).nwe_var_0F = 0 as uint16;
        (*E).nwe_var_18 = 0 as uint16;
        (*E).nwe_var_19 = 0 as uint16;
        (*E).nwe_var_1A = 0 as uint16;
        (*E).nwe_var_1B = 0 as uint16;
        (*E).nwe_var_1C = 0 as uint16;
        (*E).nwe_var_1D = 0 as uint16;
        (*E).nwe_var_1E = 0 as uint16;
        (*E).nwe_var_1F = 0 as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int)
            as uint16;
    }
}
pub unsafe fn NuclearWaffle_Func_2(game: &mut Game) {
    let mut E: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut varE32: uint16 = (*E).nwe_var_D;
    let mut pair: PairU16 = NuclearWaffle_Func_6((*E).nwe_var_21, game);
    let mut v8: uint16 = pair.j;
    if pair.j as ::core::ffi::c_int != (*E).nwe_var_33 as ::core::ffi::c_int {
        CreateSpriteAtPos(
            (*E).base.x_pos,
            (*E).base.y_pos,
            46 as uint16,
            (*E).nwe_var_34,
            game,
        );
        CreateSpriteAtPos(
            (*E).base.x_pos,
            (*E).base.y_pos,
            (pair.k as ::core::ffi::c_int + 44 as ::core::ffi::c_int) as uint16,
            (*E).nwe_var_34,
            game,
        );
        NuclearWaffle_Func_7(pair.j, game);
    }
    (*E).nwe_var_33 = v8;
    let mut tmp: uint16 = 0;
    let mut v4: uint16 = NuclearWaffle_Func_5((*E).nwe_var_21, &raw mut tmp, game);
    (*E).base.x_pos = ((*E).nwe_var_26 as ::core::ffi::c_int
        + CosineMult8bit(v4, (*E).nwe_var_D, game) as ::core::ffi::c_int)
        as uint16;
    let mut v5: uint16 = NuclearWaffle_Func_5((*E).nwe_var_21, &raw mut tmp, game);
    (*E).base.y_pos = ((*E).nwe_var_27 as ::core::ffi::c_int
        + SineMult8bit(v5, (*E).nwe_var_D, game) as ::core::ffi::c_int)
        as uint16;
    NuclearWaffle_Func_3(varE32, game);
    NuclearWaffle_Func_4(varE32, game);
    AddToHiLo(
        &raw mut (*E).nwe_var_21,
        &raw mut (*E).nwe_var_20,
        ((*E).nwe_var_25 as uint32) << 16 as ::core::ffi::c_int | (*E).nwe_var_24 as uint32,
    );
}
pub unsafe fn NuclearWaffle_Func_3(mut varE32: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v12: uint16 = 0;
    (*E).nwe_var_2A = 8 as uint16;
    let mut R36: uint16 =
        ((*E).nwe_var_2C as ::core::ffi::c_int + (*E).nwe_var_21 as ::core::ffi::c_int) as uint16;
    let mut r28: uint16 = 0;
    loop {
        R36 = (R36 as ::core::ffi::c_int - (*E).nwe_var_2B as ::core::ffi::c_int) as uint16;
        let mut ET: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
                + (*E).nwe_var_2A as ::core::ffi::c_int,
            game,
        );
        let mut nwe_var_00: uint16 = (*ET).nwe_var_00;
        let mut pair: PairU16 = NuclearWaffle_Func_6(R36, game);
        let mut v13: uint16 = pair.j;
        if pair.j as ::core::ffi::c_int != (*ET).nwe_var_08 as ::core::ffi::c_int {
            let mut v5: ::core::ffi::c_int =
                nwe_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            CreateSpriteAtPos(
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                    .offset(v5 as isize),
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                    .offset(v5 as isize),
                46 as uint16,
                (*E).nwe_var_34,
                game,
            );
            CreateSpriteAtPos(
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                    .offset(v5 as isize),
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                    .offset(v5 as isize),
                (pair.k as ::core::ffi::c_int + 44 as ::core::ffi::c_int) as uint16,
                (*E).nwe_var_34,
                game,
            );
            NuclearWaffle_Func_7(pair.j, game);
        }
        (*ET).nwe_var_08 = v13;
        let mut v7: uint16 = NuclearWaffle_Func_5(R36, &raw mut r28, game);
        let mut v10: ::core::ffi::c_int =
            nwe_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v10 as isize) =
            ((*E).nwe_var_26 as ::core::ffi::c_int
                + CosineMult8bit(v7, varE32, game) as ::core::ffi::c_int) as uint16;
        let mut v11: uint16 = NuclearWaffle_Func_5(R36, &raw mut r28, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v10 as isize) =
            ((*E).nwe_var_27 as ::core::ffi::c_int
                + SineMult8bit(v11, varE32, game) as ::core::ffi::c_int) as uint16;
        v12 = ((*E).nwe_var_2A as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        (*E).nwe_var_2A = v12;
        if !(v12 != 0) {
            break;
        }
    }
    if r28 != 0 {
        (*E).nwe_var_A = (0xa69615 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int))
            as uint16;
    }
}
pub unsafe fn NuclearWaffle_Func_4(mut varE32: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).nwe_var_2A = 6 as uint16;
    let mut R36: uint16 = (*E).nwe_var_21;
    let mut v12: uint16 = 0;
    let mut r28: uint16 = 0 as uint16;
    loop {
        R36 = (R36 as ::core::ffi::c_int - (*E).nwe_var_2B as ::core::ffi::c_int) as uint16;
        let mut ET: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
                + (*E).nwe_var_2A as ::core::ffi::c_int,
            game,
        );
        let mut nwe_var_10: uint16 = (*ET).nwe_var_10;
        let mut pair: PairU16 = NuclearWaffle_Func_6(R36, game);
        let mut v13: uint16 = pair.j;
        if pair.j as ::core::ffi::c_int != (*ET).nwe_var_18 as ::core::ffi::c_int {
            let mut r38: uint16 = (*E).nwe_var_34;
            let mut v5: ::core::ffi::c_int =
                nwe_var_10 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            CreateSpriteAtPos(
                *((&raw mut game.g_ram as *mut uint8).offset(0xf0f8) as *mut uint16)
                    .offset(v5 as isize),
                *((&raw mut game.g_ram as *mut uint8).offset(0xf1f8) as *mut uint16)
                    .offset(v5 as isize),
                46 as uint16,
                r38,
                game,
            );
            CreateSpriteAtPos(
                *((&raw mut game.g_ram as *mut uint8).offset(0xf0f8) as *mut uint16)
                    .offset(v5 as isize),
                *((&raw mut game.g_ram as *mut uint8).offset(0xf1f8) as *mut uint16)
                    .offset(v5 as isize),
                (pair.k as ::core::ffi::c_int + 44 as ::core::ffi::c_int) as uint16,
                r38,
                game,
            );
            NuclearWaffle_Func_7(pair.j, game);
        }
        (*ET).nwe_var_18 = v13;
        let mut v6: uint16 = NuclearWaffle_Func_5(R36, &raw mut r28, game);
        let mut v7: uint16 = CosineMult8bit(v6, varE32, game);
        let mut R32: uint16 =
            ((*E).nwe_var_26 as ::core::ffi::c_int + v7 as ::core::ffi::c_int) as uint16;
        let mut v9: uint16 = NuclearWaffle_Func_5(R36, &raw mut r28, game);
        let mut R34: uint16 = ((*E).nwe_var_27 as ::core::ffi::c_int
            + SineMult8bit(v9, varE32, game) as ::core::ffi::c_int)
            as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xf0f8) as *mut uint16)
            .offset((nwe_var_10 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = R32;
        *((&raw mut game.g_ram as *mut uint8).offset(0xf1f8) as *mut uint16)
            .offset((nwe_var_10 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = R34;
        v12 = ((*E).nwe_var_2A as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        (*E).nwe_var_2A = v12;
        if !(v12 != 0) {
            break;
        }
    }
}
pub unsafe fn NuclearWaffle_Func_5(
    mut a: uint16,
    mut r28_out: *mut uint16,
    game: &mut Game,
) -> uint16 {
    *r28_out = 0 as uint16;
    let mut E: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).nwe_var_E == 0 {
        if (a as ::core::ffi::c_int - (*E).nwe_var_2E as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            if ((a as ::core::ffi::c_int - (*E).nwe_var_23 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                return a;
            }
            return (*E).nwe_var_23;
        }
    } else if !((a as ::core::ffi::c_int - (*E).nwe_var_2E as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int)
    {
        if ((a as ::core::ffi::c_int - (*E).nwe_var_23 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            return (*E).nwe_var_23;
        }
        return a;
    }
    *r28_out = (*r28_out).wrapping_add(1);
    return (*E).nwe_var_2E;
}
unsafe fn NuclearWaffle_Func_6(mut a: uint16, game: &mut Game) -> PairU16 {
    let mut E: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).nwe_var_E != 0 {
        if (a as ::core::ffi::c_int - (*E).nwe_var_2F as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            return PairU16 {
                k: 1 as uint16,
                j: 2 as uint16,
            };
        } else if (a as ::core::ffi::c_int - (*E).nwe_var_30 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            return PairU16 {
                k: 0 as uint16,
                j: 1 as uint16,
            };
        } else {
            return PairU16 {
                k: 0 as uint16,
                j: 0 as uint16,
            };
        }
    } else if ((a as ::core::ffi::c_int - (*E).nwe_var_2F as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        return PairU16 {
            k: 0 as uint16,
            j: 2 as uint16,
        };
    } else if ((a as ::core::ffi::c_int - (*E).nwe_var_30 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        return PairU16 {
            k: 1 as uint16,
            j: 1 as uint16,
        };
    } else {
        return PairU16 {
            k: 0 as uint16,
            j: 0 as uint16,
        };
    };
}
pub unsafe fn NuclearWaffle_Func_7(mut r30: uint16, game: &mut Game) {
    if r30 as ::core::ffi::c_int != 2 as ::core::ffi::c_int {
        QueueSfx2_Max6(0x5e as uint16, game);
    }
}
pub unsafe fn FakeKraid_Init(game: &mut Game) {
    let mut v0: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
        as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int)
        + 2 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_FakeKraid = Get_FakeKraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).fkd_var_D = v0;
    (*E).fkd_var_E = v0;
    v0 = (v0 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
    (*E).fkd_var_03 = v0;
    v0 = (v0 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    (*E).fkd_var_04 = v0;
    (*E).fkd_var_05 = (v0 as ::core::ffi::c_int - 48 as ::core::ffi::c_int) as uint16;
    (*E).fkd_var_07 = 0 as uint16;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_DisableSamusColl as ::core::ffi::c_int) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).fkd_var_B = -(4 as ::core::ffi::c_int) as uint16;
    (*E).fkd_var_C = -(4 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = addr_stru_A699AE as ::core::ffi::c_int as uint16;
    if (((*E).base.x_pos as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        (*E).fkd_var_B = 4 as uint16;
        (*E).fkd_var_C = 4 as uint16;
        v2 = addr_stru_A699FC as ::core::ffi::c_int as uint16;
    }
    (*E).base.current_instruction = v2;
}
pub unsafe fn FakeKraid_Main(game: &mut Game) {
    let mut E: *mut Enemy_FakeKraid = Get_FakeKraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut fkd_var_07: uint16 = (*E).fkd_var_07;
    let mut v3: uint16 = (fkd_var_07 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    if fkd_var_07 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        v3 = 0 as uint16;
    }
    (*E).fkd_var_07 = v3;
    FakeKraid_Func_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        fkd_var_07,
        game,
    );
}
pub unsafe fn FakeKraid_Func_1(mut k: uint16, mut j: uint16, game: &mut Game) {
    let mut ET: *mut Enemy_FakeKraid =
        Get_FakeKraid(k as ::core::ffi::c_int + j as ::core::ffi::c_int, game);
    let mut fkd_var_03: uint16 = (*ET).fkd_var_03;
    if fkd_var_03 != 0 {
        (*ET).fkd_var_03 = (fkd_var_03 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    } else {
        (*ET).fkd_var_03 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 0x3f as ::core::ffi::c_int)
            + 16 as ::core::ffi::c_int) as uint16;
        let mut E: *mut Enemy_FakeKraid = Get_FakeKraid(k as ::core::ffi::c_int, game);
        (*E).fkd_var_06 = j;
        let mut v6: uint16 = addr_kEproj_MiniKraidSpikesLeft as ::core::ffi::c_int as uint16;
        let mut fkd_var_C: int16 = (*E).fkd_var_C as int16;
        if fkd_var_C as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            v6 = addr_kEproj_MiniKraidSpikesRight as ::core::ffi::c_int as uint16;
        }
        SpawnEprojWithGfx(fkd_var_C as uint16, k, v6, game);
        if CheckIfEnemyIsOnScreen(game) == 0 {
            QueueSfx2_Max6(0x3f as uint16, game);
        }
    };
}
pub unsafe fn FakeKraid_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut current_block: u64;
    let mut E: *mut Enemy_FakeKraid = Get_FakeKraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).fkd_var_E != 0 {
        (*E).fkd_var_E = (*E).fkd_var_E.wrapping_sub(1);
    }
    let fresh1 = (*E).fkd_var_D;
    (*E).fkd_var_D = (*E).fkd_var_D.wrapping_sub(1);
    if fresh1 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).fkd_var_D = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int)
            + 7 as ::core::ffi::c_int) as uint16;
        current_block = 6873731126896040597;
    } else if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        ((*E).fkd_var_B as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) == 0
    {
        current_block = 7288802866200656202;
    } else {
        current_block = 6873731126896040597;
    }
    match current_block {
        6873731126896040597 => {
            (*E).fkd_var_B = -((*E).fkd_var_B as ::core::ffi::c_int) as uint16;
        }
        _ => {}
    }
    (*E).fkd_var_C = -(4 as ::core::ffi::c_int) as uint16;
    if (((*E).base.x_pos as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        (*E).fkd_var_C = 4 as uint16;
    }
    return jp;
}
pub unsafe fn FakeKraid_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_FakeKraid = Get_FakeKraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).fkd_var_E != 0 {
        if (*E).fkd_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if (*Get_FakeKraid(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            ))
            .fkd_var_B as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                return addr_stru_A699C6 as ::core::ffi::c_int as uintptr_t as *const uint16;
            }
            return addr_stru_A699AE as ::core::ffi::c_int as uintptr_t as *const uint16;
        } else {
            if (*E).fkd_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                return addr_stru_A69A14 as ::core::ffi::c_int as uintptr_t as *const uint16;
            }
            return addr_stru_A699FC as ::core::ffi::c_int as uintptr_t as *const uint16;
        }
    } else {
        (*E).fkd_var_E = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int)
            + 3 as ::core::ffi::c_int) as uint16;
        if (*E).fkd_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            return addr_kFakeKraid_Ilist_99DC as ::core::ffi::c_int as uintptr_t as *const uint16;
        }
        return addr_kFakeKraid_Ilist_9A2A as ::core::ffi::c_int as uintptr_t as *const uint16;
    };
}
pub unsafe fn FakeKraid_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if CheckIfEnemyIsOnScreen(game) as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        QueueSfx2_Max6(0x16 as uint16, game);
    }
    return jp;
}
pub unsafe fn FakeKraid_Instr_4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    FakeKraid_InstrHelper_45(k, 0 as uint16, 0xfffc as uint16, game);
    return jp;
}
pub unsafe fn FakeKraid_InstrHelper_45(
    mut k: uint16,
    mut j: uint16,
    mut a: uint16,
    game: &mut Game,
) {
    let mut ka: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
    let mut E: *mut Enemy_FakeKraid = Get_FakeKraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).fkd_var_02 = a;
    let mut v5: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).fkd_var_00 = g_word_A69A48[v5 as usize];
    let mut v6: uint16 = g_word_A69A48[(v5 + 1 as ::core::ffi::c_int) as usize];
    (*E).fkd_var_01 = v6;
    SpawnEprojWithGfx(
        v6,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kEproj_MiniKraidSpit as ::core::ffi::c_int as uint16,
        game,
    );
    (*E).fkd_var_00 = g_word_A69A48[(v5 + 2 as ::core::ffi::c_int) as usize];
    let mut v8: uint16 = g_word_A69A48[(v5 + 3 as ::core::ffi::c_int) as usize];
    (*E).fkd_var_01 = v8;
    SpawnEprojWithGfx(
        v8,
        ka,
        addr_kEproj_MiniKraidSpit as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn FakeKraid_Instr_5(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    FakeKraid_InstrHelper_45(k, 8 as uint16, 4 as uint16, game);
    return jp;
}
pub unsafe fn sub_A69C0B(game: &mut Game) {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xf434) as *mut uint16) = (*v0).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf436) as *mut uint16) = (*v0).y_pos;
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
    FakeKraid_9C50(game);
}
pub unsafe fn FakeKraid_Touch(game: &mut Game) {
    let mut E: *mut Enemy_FakeKraid = Get_FakeKraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xf434) as *mut uint16) = (*E).base.x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf436) as *mut uint16) = (*E).base.y_pos;
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
    FakeKraid_9C50(game);
}
pub unsafe fn FakeKraid_Shot(game: &mut Game) {
    let mut E: *mut Enemy_FakeKraid = Get_FakeKraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xf434) as *mut uint16) = (*E).base.x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf436) as *mut uint16) = (*E).base.y_pos;
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    FakeKraid_9C50(game);
}
pub unsafe fn FakeKraid_9C50(game: &mut Game) {
    if (*Get_FakeKraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .health
        == 0
    {
        // The death animation remains in the ROM spawn parameter for all four drops.
        let death_animation = 3;
        EnemyDeathAnimation(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            death_animation,
            game,
        );
        Enemy_ItemDrop_MiniKraid(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            death_animation,
            game,
        );
    }
}
pub unsafe fn CeresRidley_Init(game: &mut Game) {
    let mut v4: int16 = 0;
    let mut v8: int16 = 0;
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | (kEnemyProps_Tangible as ::core::ffi::c_int
                | kEnemyProps_Deleted as ::core::ffi::c_int
                | kEnemyProps_Invisible as ::core::ffi::c_int))
            as uint16;
    } else {
        let mut i: ::core::ffi::c_int = 4094 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
        DisableMinimapAndMarkBossRoomAsExplored(game);
        (*E).cry_parameter_1 = 0 as uint16;
        (*E).cry_parameter_2 = 0 as uint16;
        Ridley_Func_99(
            addr_kRidley_Ilist_E538 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).base.palette_index = 3584 as uint16;
        (*E).cry_var_0C = 3584 as uint16;
        (*E).base.extra_properties =
            ((*E).base.extra_properties as ::core::ffi::c_int | 4 as ::core::ffi::c_int) as uint16;
        (*E).cry_var_01 = 0 as uint16;
        (*E).cry_var_0D = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 0 as uint16;
        Ridley_Func_92(game);
        Ridley_Func_117(game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as ::core::ffi::c_int
            == 2 as ::core::ffi::c_int
        {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                | (kEnemyProps_BlockPlasmaBeam as ::core::ffi::c_int
                    | kEnemyProps_Tangible as ::core::ffi::c_int))
                as uint16;
            (*E).base.x_pos = 96 as uint16;
            (*E).base.y_pos = 394 as uint16;
            (*E).cry_var_A =
                (0xa6a35b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).cry_var_B = 0 as uint16;
            (*E).cry_var_C = 0 as uint16;
            (*E).base.layer = 5 as uint16;
            (*E).cry_var_01 = 0 as uint16;
            (*E).cry_var_02 = 1 as uint16;
            (*E).cry_var_20 = 64 as uint16;
            (*E).cry_var_21 = 416 as uint16;
            (*E).cry_var_22 = 64 as uint16;
            (*E).cry_var_23 = 224 as uint16;
            (*E).cry_var_10 = 2 as uint16;
            (*E).cry_var_1C = 120 as uint16;
            WriteColorsToTargetPalette(
                0xa6 as uint8,
                0x140 as uint16,
                addr_word_A6E1CF as ::core::ffi::c_int as uint16,
                0x20 as uint16,
                game,
            );
            let mut v3: uint16 = 0 as uint16;
            v4 = 15 as int16;
            loop {
                let mut v5: ::core::ffi::c_int =
                    v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                    .offset((v5 + 113 as ::core::ffi::c_int) as isize) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                    .offset((v5 + 241 as ::core::ffi::c_int) as isize) = 0 as uint16;
                v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                v4 -= 1;
                if !(v4 != 0) {
                    break;
                }
            }
        } else {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                | (kEnemyProps_BlockPlasmaBeam as ::core::ffi::c_int
                    | kEnemyProps_Tangible as ::core::ffi::c_int))
                as uint16;
            (*E).base.x_pos = 186 as uint16;
            (*E).base.y_pos = 169 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) = 0 as uint16;
            (*E).cry_var_10 = 0 as uint16;
            (*E).cry_var_02 = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(1) =
                1 as uint16;
            (*E).cry_var_1C = 15 as uint16;
            (*E).cry_var_A =
                (0xa6a35b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).cry_var_B = 0 as uint16;
            (*E).cry_var_C = 0 as uint16;
            (*E).cry_var_20 = -(32 as ::core::ffi::c_int) as uint16;
            (*E).cry_var_21 = 176 as uint16;
            (*E).cry_var_22 = 40 as uint16;
            (*E).cry_var_23 = 224 as uint16;
            WriteColorsToTargetPalette(
                0xa6 as uint8,
                0x140 as uint16,
                addr_word_A6E16F as ::core::ffi::c_int as uint16,
                0x20 as uint16,
                game,
            );
            let mut v7: uint16 = 482 as uint16;
            v8 = 15 as int16;
            loop {
                *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                    .offset((v7 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                    0 as uint16;
                v7 = (v7 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                v8 -= 1;
                if !(v8 != 0) {
                    break;
                }
            }
            (*E).cry_var_03 = addr_kBabyMetroid_Ilist_BF31 as ::core::ffi::c_int as uint16;
            (*E).cry_var_04 = 1 as uint16;
            (*gRam8800_Default(0 as ::core::ffi::c_int, game)).var_40 =
                (0xa6be9c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).cry_var_07 = 5 as uint16;
            QueueMusic_Delayed8(0 as uint16, game);
        }
    };
}
pub unsafe fn CallRidleyFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        10920788 => {
            CeresRidley_Func_2(game);
            return;
        }
        10920795 => {
            CeresRidley_Func_3(game);
            return;
        }
        10920823 => {
            CeresRidley_A377(game);
            return;
        }
        10920841 => {
            CeresRidley_Func_4(game);
            return;
        }
        10920927 => {
            CeresRidley_Func_5(game);
            return;
        }
        10921045 => {
            CeresRidley_Func_6(game);
            return;
        }
        10921080 => {
            CeresRidley_Func_7(game);
            return;
        }
        10921647 => {
            CeresRidley_Func_9(game);
            return;
        }
        10921672 => {
            CeresRidley_Func_10(game);
            return;
        }
        10921704 => {
            CeresRidley_Func_11(game);
            return;
        }
        10921858 => {
            CeresRidley_Func_13(game);
            return;
        }
        10921977 => {
            CeresRidley_A7F9(game);
            return;
        }
        10922044 => {
            CeresRidley_Func_14(game);
            return;
        }
        10922062 => {
            CeresRidley_Func_15(game);
            return;
        }
        10922125 => {
            CeresRidley_Func_16(game);
            return;
        }
        10922148 => {
            CeresRidley_Func_17(game);
            return;
        }
        10922196 => {
            CeresRidley_Func_18(game);
            return;
        }
        10922232 => {
            CeresRidley_Func_19(game);
            return;
        }
        10922275 => {
            CeresRidley_Func_20(game);
            return;
        }
        10922311 => {
            CeresRidley_Func_21(game);
            return;
        }
        10922353 => {
            CeresRidley_Func_22(game);
            return;
        }
        10922400 => {
            CeresRidley_Func_23(game);
            return;
        }
        10922513 => {
            CeresRidley_Func_24(game);
            return;
        }
        10922575 => return,
        10922576 => {
            CeresRidley_Func_26(game);
            return;
        }
        10924787 => {
            Ridley_Func_3(game);
            return;
        }
        10924833 => {
            Ridley_Func_3b(game);
            return;
        }
        10925036 => {
            Ridley_Func_5(game);
            return;
        }
        10925048 => {
            Ridley_Func_6(game);
            return;
        }
        10925121 => {
            Ridley_Func_8(game);
            return;
        }
        10925141 => {
            Ridley_B455(game);
            return;
        }
        10925203 => {
            Ridley_Func_9(game);
            return;
        }
        10925265 => {
            Ridley_Func_10(game);
            return;
        }
        10925334 => {
            Ridley_Func_11(game);
            return;
        }
        10925396 => {
            Ridley_Func_12(game);
            return;
        }
        10925460 => {
            Ridley_Func_13(game);
            return;
        }
        10925508 => {
            Ridley_Func_14(game);
            return;
        }
        10925541 => {
            Ridley_Func_15(game);
            return;
        }
        10925587 => {
            Ridley_Func_16(game);
            return;
        }
        10925735 => {
            Ridley_Func_19(game);
            return;
        }
        10925789 => {
            Ridley_Func_20(game);
            return;
        }
        10925838 => {
            Ridley_Func_21(game);
            return;
        }
        10926009 => {
            Ridley_Func_22(game);
            return;
        }
        10926775 => {
            Ridley_Func_33(game);
            return;
        }
        10926991 => {
            Ridley_Func_33b(game);
            return;
        }
        10927044 => {
            Ridley_Func_34(game);
            return;
        }
        10927089 => {
            Ridley_Func_35(game);
            return;
        }
        10927150 => {
            Ridley_Func_36(game);
            return;
        }
        10927438 => {
            Ridley_Func_43(game);
            return;
        }
        10927514 => {
            Ridley_Func_44(game);
            return;
        }
        10927548 => {
            Ridley_Func_45(game);
            return;
        }
        10927602 => {
            Ridley_Func_46(game);
            return;
        }
        10927619 => {
            Ridley_Func_47(game);
            return;
        }
        10928206 => {
            Ridley_Func_54(game);
            return;
        }
        10929464 => {
            Ridley_Func_63(game);
            return;
        }
        10929470 => {
            Ridley_C53E(game);
            return;
        }
        10929489 => {
            Ridley_C551(game);
            return;
        }
        10929544 => {
            Ridley_Func_64(game);
            return;
        }
        10929576 => {
            Ridley_Func_65(game);
            return;
        }
        10929608 => {
            Ridley_Func_66(game);
            return;
        }
        10929626 => {
            Ridley_Func_67(game);
            return;
        }
        10929664 => return,
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CeresRidley_Main(game: &mut Game) {
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).base.health = 0x7fff as uint16;
    CallRidleyFunc(
        ((*E).cry_var_A as ::core::ffi::c_int | 0xa60000 as ::core::ffi::c_int) as uint32,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) == 0 {
        if (*Get_CeresRidley(0 as ::core::ffi::c_int, game)).cry_var_02 != 0 {
            Ridley_Func_102(game);
            Ridley_Func_112(game);
            Ridley_Func_115(game);
            CeresRidley_Func_1(game);
            Ridley_Func_70(game);
            Ridley_Func_101(game);
            Ridley_Func_118(game);
            sub_A6DB2A(game);
            Ridley_Func_120(game);
        }
        Ridley_A2DC(game);
    }
}
pub unsafe fn CeresRidley_Func_1(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        >= 0xff00 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(15) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)
                + 8 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CeresRidley_Hurt(game: &mut Game) {
    Ridley_Func_102(game);
    sub_A6DB2A(game);
    Ridley_Func_120(game);
    Ridley_A2DC(game);
}
pub unsafe fn Ridley_A2DC(game: &mut Game) {
    Ridley_Func_49(game);
    if (*Get_CeresRidley(0 as ::core::ffi::c_int, game)).cry_var_02 != 0 {
        mov24(
            (&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr,
            0xa6a2f2 as uint32,
        );
    } else {
        Ridley_A2F2(game);
    };
}
pub unsafe fn Ridley_A2F2(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) == 0 {
        DrawBabyMetroid_0(game);
    }
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0x40 as ::core::ffi::c_int, game);
    if (*E).cry_var_B != 0 {
        static g_word_A6A321: [int16; 4] = [
            0 as ::core::ffi::c_int as int16,
            -(1024 as ::core::ffi::c_int) as int16,
            -(4 as ::core::ffi::c_int) as int16,
            -(1 as ::core::ffi::c_int) as int16,
        ];
        DrawSpritemap(
            0xa6 as uint8,
            addr_kCeresRidley_Sprmap_A329 as ::core::ffi::c_int as uint16,
            ((*E).base.x_pos as ::core::ffi::c_int
                + g_word_A6A321[(*((&raw mut game.g_ram as *mut uint8).offset(0x1840)
                    as *mut uint16) as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int)
                as uint16,
            (*E).base.y_pos,
            1024 as uint16,
            game,
        );
    }
}
pub unsafe fn CeresRidley_Func_2(game: &mut Game) {
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_B = 0 as uint16;
    (*E).cry_var_C = 0 as uint16;
}
pub unsafe fn CeresRidley_Func_3(game: &mut Game) {
    let mut v1: int16 = 0;
    if !(*((&raw mut game.g_ram as *mut uint8).offset(0x797) as *mut uint16) != 0) {
        let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
        (*E).cry_var_A = (0xa6a377 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        v1 = 512 as int16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as ::core::ffi::c_int
            == 2 as ::core::ffi::c_int
        {
            v1 = 170 as int16;
        }
        (*E).cry_var_F = v1 as uint16;
        CeresRidley_A377(game);
    }
}
pub unsafe fn CeresRidley_A377(game: &mut Game) {
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if (*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).cry_var_A = (0xa6a389 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).cry_var_E = 0 as uint16;
        (*E).cry_var_F = 0 as uint16;
    }
}
pub unsafe fn CeresRidley_Func_4(game: &mut Game) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    if (*E).cry_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut cry_var_E: uint16 = (*E).cry_var_E;
        (*E).cry_var_F = (*E).cry_var_F.wrapping_add(1);
        if (*E).cry_var_F != 0 {
            (*E).cry_var_F = 0 as uint16;
            v2 = *(game.g_rom.offset(
                (((0xa6e269 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6e269 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(cry_var_E as isize) as int16;
            if v2 as ::core::ffi::c_int == 255 as ::core::ffi::c_int {
                (*E).cry_var_E = 0 as uint16;
                (*E).cry_var_A =
                    (0xa6a3df as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                (*E).cry_var_02 = 1 as uint16;
            } else {
                (*E).cry_var_E =
                    (cry_var_E as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
                let mut v3: ::core::ffi::c_int =
                    (6 as ::core::ffi::c_int * v2 as ::core::ffi::c_int) as uint16
                        as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(252) =
                    *(game.g_rom.offset(
                        (((0xa6e2aa as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa6e2aa as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset(v3 as isize);
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(253) =
                    *(game.g_rom.offset(
                        (((0xa6e2aa as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa6e2aa as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset((v3 + 1 as ::core::ffi::c_int) as isize);
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(254) =
                    *(game.g_rom.offset(
                        (((0xa6e2aa as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa6e2aa as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset((v3 + 2 as ::core::ffi::c_int) as isize);
            }
        }
    }
}
pub unsafe fn CeresRidley_Func_5(game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v5: int16 = 0;
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_add(1);
    if (*E).cry_var_F as ::core::ffi::c_int >= 2 as ::core::ffi::c_int {
        (*E).cry_var_F = 0 as uint16;
        let mut cry_var_E: uint16 = (*E).cry_var_E;
        let mut r18: uint16 = 290 as uint16;
        let mut r20: uint16 = 482 as uint16;
        v2 = 11 as int16;
        loop {
            v5 = v2;
            let mut v3: uint16 = *(game.g_rom.offset(
                (((0xa6e30a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6e30a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((cry_var_E as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((r18 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v3;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v3;
            cry_var_E = (cry_var_E as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            r18 = (r18 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            r20 = (r20 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            v2 = (v5 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
            if !(v5 as ::core::ffi::c_int != 1 as ::core::ffi::c_int) {
                break;
            }
        }
        if cry_var_E as ::core::ffi::c_int >= 0x160 as ::core::ffi::c_int {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                as ::core::ffi::c_int
                == 2 as ::core::ffi::c_int
            {
                (*E).base.layer = 2 as uint16;
            }
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Tangible as ::core::ffi::c_int))
                as uint16;
            (*E).cry_var_E = 0 as uint16;
            (*E).cry_var_A =
                (0xa6a455 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).cry_var_F = 4 as uint16;
            QueueMusic_Delayed8(5 as uint16, game);
        } else {
            (*E).cry_var_E = cry_var_E;
        }
    }
}
pub unsafe fn CeresRidley_Func_6(game: &mut Game) {
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if (*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        Ridley_Func_99(
            addr_kRidley_Ilist_E690 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).cry_var_F = 0 as uint16;
        (*E).cry_var_A = (0xa6a478 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as ::core::ffi::c_int
            != 2 as ::core::ffi::c_int
        {
            (*E).cry_var_F = 252 as uint16;
        }
    }
}
pub unsafe fn CeresRidley_Func_7(game: &mut Game) {
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as ::core::ffi::c_int
        == 2 as ::core::ffi::c_int
    {
        (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
        let mut v1: bool = (*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
        if !v1 {
            return;
        }
        (*E).cry_var_F = 2 as uint16;
        let mut v2: uint16 =
            ((*E).cry_var_E as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E).cry_var_E = v2;
        if CeresRidley_Func_8(
            (v2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
            game,
        ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0
        {
            return;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16) = 440 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) =
            -(96 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1980) as *mut uint16) = 32 as uint16;
    } else {
        (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
        let mut v1_0: bool = (*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
        if !v1_0 {
            return;
        }
    }
    (*E).cry_var_E = 0 as uint16;
    Ridley_Func_99(
        addr_kRidley_Ilist_E91D as ::core::ffi::c_int as uint16,
        game,
    );
    (*E).cry_var_08 = 8 as uint16;
    (*E).cry_var_09 = 8 as uint16;
    Ridley_Func_95(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) = 1 as uint16;
    (*E).cry_var_A = (0xa6a354 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CeresRidley_Func_8(mut a: uint16, game: &mut Game) -> uint8 {
    let mut v1: uint16 = *(game.g_rom.offset(
        (((0xa6a4eb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa6a4eb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(a as isize);
    if v1 == 0 {
        return 1 as uint8;
    }
    WriteColorsToPalette(0xe2 as uint16, 0xa6 as uint8, v1, 0xe as uint16, game);
    return 0 as uint8;
}
pub unsafe fn CeresRidley_Func_9(game: &mut Game) {
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_C = ((*E).cry_var_C as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
    if (*E).base.y_pos as ::core::ffi::c_int - 112 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).cry_var_A = (0xa6a6c8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        CeresRidley_Func_10(game);
    }
}
pub unsafe fn CeresRidley_Func_10(game: &mut Game) {
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_C = ((*E).cry_var_C as ::core::ffi::c_int + 20 as ::core::ffi::c_int) as uint16;
    if (*E).base.y_pos as ::core::ffi::c_int - 80 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).cry_var_A = (0xa6a6e8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).cry_var_01 = 1 as uint16;
    }
}
pub unsafe fn CeresRidley_Func_11(game: &mut Game) {
    let mut v1: uint16 = 0;
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    if (*E).cry_var_0D as ::core::ffi::c_int >= 0x64 as ::core::ffi::c_int {
        (*E).cry_var_01 = 0 as uint16;
        (*E).cry_var_A = (0xa6bd9a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        Ridley_Func_44(game);
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16)
        as ::core::ffi::c_int
        - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).cry_var_01 = 0 as uint16;
        (*E).cry_var_A = (0xa6a971 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        CeresRidley_Func_22(game);
    } else if CeresRidley_Func_12(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 || {
        v1 = ((*E).cry_var_00 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E).cry_var_00 = v1;
        v1 as ::core::ffi::c_int >= 0x7c as ::core::ffi::c_int
    } {
        (*E).cry_var_A = *(game.g_rom.offset(
            (((0xa6a743 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa6a743 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                    as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as isize,
            );
        (*E).cry_var_00 = 0 as uint16;
    }
}
pub unsafe fn CeresRidley_Func_12(game: &mut Game) -> uint8 {
    let mut rect: Rect16U = Rect16U {
        x: 192 as uint16,
        y: 100 as uint16,
        w: 8 as uint16,
        h: 8 as uint16,
    };
    Ridley_Func_106(0 as uint16, 0 as uint16, rect.x, rect.y, game);
    return Shitroid_Func_2(0 as uint16, rect, game);
}
pub unsafe fn CeresRidley_Func_13(game: &mut Game) {
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 = abs16((*E).cry_var_C);
    if (v1 as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        v1 = 128 as uint16;
    }
    (*E).cry_var_C = (if (*E).cry_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(v1 as ::core::ffi::c_int)
    } else {
        v1 as ::core::ffi::c_int
    }) as uint16;
    Ridley_Func_106(
        0 as uint16,
        0 as uint16,
        (*E).base.x_pos,
        88 as uint16,
        game,
    );
    if (*E).base.y_pos as ::core::ffi::c_int - 80 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: uint16 =
            ((*E).cry_var_00 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E).cry_var_00 = v3;
        if v3 as ::core::ffi::c_int >= 0x30 as ::core::ffi::c_int {
            (*E).cry_var_A =
                (0xa6a88d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    } else if (*E).base.y_pos as ::core::ffi::c_int - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).cry_var_17 = (*E).base.x_pos;
        (*E).cry_var_18 = (*E).base.y_pos;
        Ridley_Func_99(
            addr_kRidley_Ilist_E73A as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).cry_var_A = addr_loc_A6A7F9 as ::core::ffi::c_int as uint16;
        (*E).cry_var_00 = 224 as uint16;
        CeresRidley_A7F9(game);
    }
}
pub unsafe fn CeresRidley_A7F9(game: &mut Game) {
    let mut r22: uint16 = (if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        -(*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int)
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int
    }) as uint16;
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    let mut r18: uint16 =
        ((*E).cry_var_17 as ::core::ffi::c_int + r22 as ::core::ffi::c_int) as uint16;
    let mut r20: uint16 =
        ((*E).cry_var_18 as ::core::ffi::c_int + r22 as ::core::ffi::c_int) as uint16;
    Ridley_Func_106(0 as uint16, 0 as uint16, r18, r20, game);
    let mut v4: int16 = ((*E).cry_var_00 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
    (*E).cry_var_00 = v4 as uint16;
    if (v4 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).cry_var_00 = 0 as uint16;
        (*E).cry_var_A = (0xa6a6e8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CeresRidley_Func_14(game: &mut Game) {
    Ridley_Func_99(
        addr_kRidley_Ilist_E548 as ::core::ffi::c_int as uint16,
        game,
    );
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_A = (0xa6a84e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).cry_var_F = 64 as uint16;
    CeresRidley_Func_15(game);
}
pub unsafe fn CeresRidley_Func_15(game: &mut Game) {
    let mut v2: bool = false;
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    let mut r18: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
    let mut v0: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
        as ::core::ffi::c_int
        - 68 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        - 132 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        v0 = 64 as uint16;
    }
    let mut r20: uint16 = v0;
    Ridley_Func_106(0 as uint16, 0xd as uint16, r18, r20, game);
    let mut rect: Rect16U = Rect16U {
        x: r18,
        y: r20,
        w: 2 as uint16,
        h: 2 as uint16,
    };
    if Shitroid_Func_2(0 as uint16, rect, game) == 0 || {
        v2 = (((*E).cry_var_F as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int;
        (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
        v2 as ::core::ffi::c_int != 0
    } {
        (*E).cry_var_00 = 0 as uint16;
        (*E).cry_var_A = (0xa6a6e8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CeresRidley_Func_16(game: &mut Game) {
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_A = (0xa6a8a4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).cry_var_F = 10 as uint16;
    (*E).cry_var_0A = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(1) = 0 as uint16;
    CeresRidley_Func_17(game);
}
pub unsafe fn CeresRidley_Func_17(game: &mut Game) {
    Ridley_Func_106(0 as uint16, 1 as uint16, 192 as uint16, 80 as uint16, game);
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    if (*E).base.y_pos as ::core::ffi::c_int - 96 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).cry_var_A = (0xa6a8d4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).cry_var_F = 10 as uint16;
        (*E).cry_var_0A = 0 as uint16;
    }
}
pub unsafe fn CeresRidley_Func_18(game: &mut Game) {
    Ridley_Func_111(
        -(32 as ::core::ffi::c_int) as uint16,
        -(1024 as ::core::ffi::c_int) as uint16,
        768 as uint16,
        game,
    );
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if (*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).cry_var_A = (0xa6a8f8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).cry_var_F = 36 as uint16;
    }
}
pub unsafe fn CeresRidley_Func_19(game: &mut Game) {
    Ridley_Func_111(
        -(512 as ::core::ffi::c_int) as uint16,
        -(16384 as ::core::ffi::c_int) as uint16,
        768 as uint16,
        game,
    );
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if (*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).cry_var_A = (0xa6a923 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).cry_var_F = 28 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2) =
            1 as uint16;
    }
}
pub unsafe fn CeresRidley_Func_20(game: &mut Game) {
    Ridley_Func_111(
        -(512 as ::core::ffi::c_int) as uint16,
        -(30720 as ::core::ffi::c_int) as uint16,
        768 as uint16,
        game,
    );
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if (*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).cry_var_A = (0xa6a947 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).cry_var_F = 1 as uint16;
    }
}
pub unsafe fn CeresRidley_Func_21(game: &mut Game) {
    Ridley_Func_111(
        -(768 as ::core::ffi::c_int) as uint16,
        -(30720 as ::core::ffi::c_int) as uint16,
        768 as uint16,
        game,
    );
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if (*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).cry_var_A = (0xa6a6e8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).cry_var_00 = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(1) =
            1 as uint16;
    }
}
pub unsafe fn CeresRidley_Func_22(game: &mut Game) {
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_20 = -(192 as ::core::ffi::c_int) as uint16;
    Ridley_Func_106(
        0 as uint16,
        1 as uint16,
        192 as uint16,
        -(128 as ::core::ffi::c_int) as uint16,
        game,
    );
    if (*E).base.y_pos as ::core::ffi::c_int + 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        CeresRidley_A994(game);
    }
}
pub unsafe fn CeresRidley_A994(game: &mut Game) {
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_A = (0xa6a9a0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).cry_var_F = 64 as uint16;
    CeresRidley_Func_23(game);
}
pub unsafe fn CeresRidley_Func_23(game: &mut Game) {
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if (*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        CeresRidley_Func_25(game);
        (*E).cry_var_B = 0 as uint16;
        (*E).cry_var_C = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) =
            0 as uint16;
        (*E).cry_var_A = (0xa6aa11 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        WriteColorsToPalette(
            0xa2 as uint16,
            0xa6 as uint8,
            addr_word_A6A9E3 as ::core::ffi::c_int as uint16,
            0xf as uint16,
            game,
        );
        WriteColorsToPalette(
            0x42 as uint16,
            0xa6 as uint8,
            addr_word_A6AA01 as ::core::ffi::c_int as uint16,
            8 as uint16,
            game,
        );
        WriteColorsToPalette(
            0x1e2 as uint16,
            0xa6 as uint8,
            addr_word_A6AA01 as ::core::ffi::c_int as uint16,
            8 as uint16,
            game,
        );
    }
}
pub unsafe fn CeresRidley_Func_24(game: &mut Game) {
    (*Get_CeresRidley(0 as ::core::ffi::c_int, game)).cry_var_A =
        (0xa6aa4f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) = 1 as uint16;
    CeresRidley_Func_27(game);
}
pub unsafe fn CeresRidley_Func_25(game: &mut Game) {
    SpawnEnemy(
        0xa6 as uint8,
        addr_stru_A6AA2F as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnEnemy(
        0xa6 as uint8,
        addr_stru_A6AA3F as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn CeresRidley_Func_26(game: &mut Game) {
    Ridley_Func_61(game);
}
pub unsafe fn CeresRidley_Func_27(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x56) = 7 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x783) as *mut uint16) = 1 as uint16;
    *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0x80 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e) as *mut uint16) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) = 64 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16) = 64 as uint16;
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_32 = 0 as uint16;
    (*E).cry_var_33 = 0 as uint16;
    (*E).cry_var_0A = 1 as uint16;
    (*E).cry_var_34 = -(128 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) =
        -(128 as ::core::ffi::c_int) as uint16;
    (*E).cry_var_35 = 32 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = 32 as uint16;
    (*E).cry_var_30 = 2048 as uint16;
    (*E).cry_var_31 = 1024 as uint16;
}
pub unsafe fn HandleCeresRidleyGetawayCutscene(game: &mut Game) {
    if (*Get_CeresRidley(0 as ::core::ffi::c_int, game)).cry_var_32 == 0 {
        CeresRidley_Func_28(game);
    }
}
pub unsafe fn CeresRidley_Func_28(game: &mut Game) {
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    let mut cry_var_33: uint16 = (*E).cry_var_33;
    (*E).cry_var_33 = (cry_var_33 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    if cry_var_33 == 0 {
        QueueSfx2_Max6(0x4e as uint16, game);
    }
    if cry_var_33 as ::core::ffi::c_int == 208 as ::core::ffi::c_int {
        Samus_SetPushedOutOfCeresRidley(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 35 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 64 as uint16;
    }
    let mut v2: ::core::ffi::c_int = cry_var_33 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *(game.g_rom.offset(
        (((0xa6ae4d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa6ae4d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v2 as isize) as ::core::ffi::c_int
        == 0xffff as ::core::ffi::c_int
    {
        (*E).cry_var_32 = (*E).cry_var_32.wrapping_add(1);
        *(&raw mut game.g_ram as *mut uint8).offset(0x56) = 9 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x7e) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = 0 as uint16;
        (*E).cry_var_A = (0xa6c04e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).cry_var_F = 0 as uint16;
    } else {
        (*E).cry_var_30 = *(game.g_rom.offset(
            (((0xa6ae4d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa6ae4d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize);
        let mut v3: uint16 = (*(game.g_rom.offset(
            (((0xa6af2f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa6af2f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            + (*E).cry_var_35 as ::core::ffi::c_int) as uint16;
        (*E).cry_var_35 = v3;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = v3;
        let mut v4: uint16 = ((*E).cry_var_34 as ::core::ffi::c_int
            - *(game.g_rom.offset(
                (((0xa6b00f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6b00f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
        (*E).cry_var_34 = v4;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) = v4;
        CeresRidley_Func_35((*E).cry_var_30, game);
        (*E).cry_var_0A =
            ((*E).cry_var_0A as ::core::ffi::c_int + 48 as ::core::ffi::c_int) as uint16;
        CeresRidley_Func_29(game);
        CeresRidley_Func_33(game);
        CeresRidley_Func_34(game);
    };
}
unsafe fn CeresMult(mut a: uint16, mut b: uint16) -> uint16 {
    let mut r: uint16 = (abs16(a) as ::core::ffi::c_int * b as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as uint16;
    return (if a as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(r as ::core::ffi::c_int)
    } else {
        r as ::core::ffi::c_int
    }) as uint16;
}
pub unsafe fn CeresRidley_Func_29(game: &mut Game) {
    let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    let mut r18: uint16 = *(&raw mut (*E).cry_var_0A as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16) = CeresMult(
        CeresRidley_AC30(
            (r18 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16,
            0x100 as int16,
        ) as uint16,
        (*E).cry_var_30,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16) = CeresMult(
        CeresRidley_AC30(r18, 0x100 as int16) as uint16,
        (*E).cry_var_30,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16) = CeresMult(
        -(CeresRidley_AC30(r18, 0x100 as int16) as ::core::ffi::c_int) as uint16,
        (*E).cry_var_30,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e) as *mut uint16) = CeresMult(
        CeresRidley_AC30(
            (r18 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16,
            0x100 as int16,
        ) as uint16,
        (*E).cry_var_30,
    );
}
pub unsafe fn CeresRidley_AC30(mut a: uint16, mut r20: int16) -> int16 {
    return (kSinCosTable8bit_Sext[((a as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
        + 64 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
        * r20 as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as int16;
}
pub unsafe fn CeresRidley_Func_33(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
        let mut v1: uint16 = ((*E).cry_var_36 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int) as uint16;
        (*E).cry_var_36 = v1;
        QueueMode7Transfers(
            0xa6 as uint8,
            *(game.g_rom.offset(
                (((0xa6acda as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6acda as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize),
            game,
        );
    }
}
pub unsafe fn CeresRidley_Func_34(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16) as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
        let mut v1: uint16 = ((*E).cry_var_37 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int) as uint16;
        (*E).cry_var_37 = v1;
        QueueMode7Transfers(
            0xa6 as uint8,
            *(game.g_rom.offset(
                (((0xa6ad45 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6ad45 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize),
            game,
        );
    }
}
pub unsafe fn CeresRidley_Func_35(mut a: uint16, game: &mut Game) {
    WriteColorsToPalette(
        0xa2 as uint16,
        0xa6 as uint8,
        (32 as ::core::ffi::c_int
            * *(&raw mut a as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            - 0x4ef9 as ::core::ffi::c_int) as uint16,
        0xf as uint16,
        game,
    );
}
pub unsafe fn Ridley_Main(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    v1 = ((*E).ridley_var_24 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as int16;
    if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        v1 = 0 as int16;
    }
    (*E).ridley_var_24 = v1 as uint16;
    Ridley_Func_39(game);
    Ridley_Func_42(game);
    CallRidleyFunc(
        ((*E).ridley_var_A as ::core::ffi::c_int | 0xa60000 as ::core::ffi::c_int) as uint32,
        game,
    );
    if (*E).ridley_var_02 != 0 {
        Ridley_Func_102(game);
        Ridley_Func_112(game);
        Ridley_Func_115(game);
        Ridley_Func_70(game);
        Ridley_Func_129(game);
        sub_A6DB2A(game);
        Ridley_Func_120(game);
        Ridley_Func_118(game);
        if (*E).ridley_var_1B != 0 {
            Ridley_Func_30(game);
        }
    }
    Ridley_Func_100(game);
}
pub unsafe fn Ridley_Func_1(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_1B as ::core::ffi::c_int | (*E).ridley_var_1E as ::core::ffi::c_int == 0 {
        if Ridley_Func_26(4 as uint16, 4 as uint16, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            Ridley_Func_37(game);
        }
    }
}
pub unsafe fn Ridley_Func_2(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_24 = 0 as uint16;
    (*E).base.frame_counter = 1 as uint16;
    Ridley_Hurt(game);
}
pub unsafe fn Ridley_Hurt(game: &mut Game) {
    let mut current_block: u64;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).base.frame_counter as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        Ridley_Func_39(game);
        Ridley_Func_42(game);
        CallRidleyFunc(
            ((*E).ridley_var_A as ::core::ffi::c_int | 0xa60000 as ::core::ffi::c_int) as uint32,
            game,
        );
        if (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_02 == 0 {
            current_block = 18350095613861854843;
        } else {
            Ridley_Func_112(game);
            Ridley_Func_115(game);
            Ridley_Func_70(game);
            current_block = 13513818773234778473;
        }
    } else {
        current_block = 13513818773234778473;
    }
    match current_block {
        13513818773234778473 => {
            Ridley_Func_129(game);
            Ridley_Func_102(game);
            sub_A6DB2A(game);
            Ridley_Func_120(game);
            Ridley_Func_100(game);
            Ridley_Func_118(game);
            let mut v2: uint16 =
                ((*E).ridley_var_24 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            if (v2 as ::core::ffi::c_int - g_word_A6B288 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                v2 = g_word_A6B288;
            }
            (*E).ridley_var_24 = v2;
            if !(((*E).ridley_var_24 as ::core::ffi::c_int - g_word_A6B288 as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int)
            {
                return;
            }
        }
        _ => {}
    }
    if (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_1B != 0 {
        Ridley_Func_30(game);
    }
}
pub unsafe fn Ridley_Func_3(game: &mut Game) {
    let mut rect: Rect16U = Rect16U {
        x: 64 as uint16,
        y: 256 as uint16,
        w: 8 as uint16,
        h: 8 as uint16,
    };
    Ridley_Func_104_0(0 as uint16, 14 as uint16, rect.x, rect.y, game);
    if Shitroid_Func_2(0 as uint16, rect, game) == 0 {
        let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        (*E).ridley_var_01 = 1 as uint16;
        (*E).ridley_var_A =
            (0xa6b321 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        Ridley_Func_3b(game);
    }
}
pub unsafe fn Ridley_Func_3b(game: &mut Game) {
    let mut r18: uint16 = Ridley_Func_4(game);
    let mut v0: ::core::ffi::c_int = 2 as ::core::ffi::c_int
        * (NextRandom(game) as ::core::ffi::c_int & 7 as ::core::ffi::c_int);
    let mut v1: uint16 =
        *((RomPtr_A6 as unsafe fn(uint16_t, &mut Game) -> *const uint8)(r18 as uint16_t, game)
            .offset(v0 as isize) as *const uint8 as *mut uint16);
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_A = v1;
    CallRidleyFunc(
        ((*E).ridley_var_A as ::core::ffi::c_int | 0xa60000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn Ridley_Func_4(game: &mut Game) -> uint16 {
    let mut r18: uint16 = 0;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as ::core::ffi::c_int
        == 3 as ::core::ffi::c_int
    {
        r18 = addr_off_A6B3CC as ::core::ffi::c_int as uint16;
    } else {
        let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        let mut health: uint16 = (*E).base.health;
        if health != 0 {
            if health as ::core::ffi::c_int - 14400 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                r18 = addr_off_A6B38C as ::core::ffi::c_int as uint16;
            } else if Ridley_Func_41(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
                r18 = addr_off_A6B3BC as ::core::ffi::c_int as uint16;
            } else if Ridley_Func_40_Carry(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                r18 = addr_off_A6B3AC as ::core::ffi::c_int as uint16;
            } else {
                r18 = (if (*E).base.health as ::core::ffi::c_int - 9000 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    addr_off_A6B39C as ::core::ffi::c_int
                } else {
                    addr_off_A6B38C as ::core::ffi::c_int
                }) as uint16;
            }
        } else {
            r18 = addr_off_A6B3DC as ::core::ffi::c_int as uint16;
            (*E).ridley_var_25 = (*E).ridley_var_25.wrapping_add(1);
        }
    }
    return r18;
}
pub unsafe fn Ridley_Func_5(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_A = (0xa6b3f8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).ridley_var_F = 128 as uint16;
    Ridley_Func_6(game);
}
pub unsafe fn Ridley_Func_6(game: &mut Game) {
    let mut v2: uint16 = 0;
    let mut rect: Rect16U = Rect16U {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    };
    let mut current_block: u64;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if (*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        current_block = 10433200308384073363;
    } else {
        v2 = 0;
        v2 = 192 as uint16;
        if (*E).ridley_var_10 != 0 {
            v2 = 96 as uint16;
        }
        rect = Rect16U {
            x: v2,
            y: 256 as uint16,
            w: 8 as uint16,
            h: 8 as uint16,
        };
        Ridley_Func_104_0(0 as uint16, Ridley_Func_7(game), rect.x, rect.y, game);
        if Shitroid_Func_2(0 as uint16, rect, game) == 0 {
            current_block = 10433200308384073363;
        } else {
            current_block = 11006700562992250127;
        }
    }
    match current_block {
        10433200308384073363 => {
            (*E).ridley_var_A =
                (0xa6b321 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        _ => {}
    };
}
pub unsafe fn Ridley_Func_7(game: &mut Game) -> uint16 {
    static g_word_A6B439: [uint16; 4] = [
        4 as ::core::ffi::c_int as uint16,
        8 as ::core::ffi::c_int as uint16,
        0xa as ::core::ffi::c_int as uint16,
        0xc as ::core::ffi::c_int as uint16,
    ];
    return g_word_A6B439[(*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_12 as usize];
}
pub unsafe fn Ridley_Func_8(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_A = addr_loc_A6B455 as ::core::ffi::c_int as uint16;
    (*E).ridley_var_00 = 10 as uint16;
    (*E).ridley_var_0A = 0 as uint16;
    Ridley_B455(game);
}
pub unsafe fn Ridley_B455(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 = 192 as uint16;
    if (*E).ridley_var_10 != 0 {
        v1 = 64 as uint16;
    }
    let mut rect: Rect16U = Rect16U {
        x: v1,
        y: 128 as uint16,
        w: 8 as uint16,
        h: 8 as uint16,
    };
    Ridley_Func_104_0(0 as uint16, 1 as uint16, rect.x, rect.y, game);
    if Shitroid_Func_2(0 as uint16, rect, game) == 0 {
        (*E).ridley_var_A =
            (0xa6b493 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_00 = 32 as uint16;
        (*E).ridley_var_0A = 0 as uint16;
    }
}
pub unsafe fn Ridley_Func_9(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_10 != 0 {
        Ridley_Func_111(32 as uint16, 512 as uint16, 1152 as uint16, game);
    } else {
        Ridley_Func_111(
            -(32 as ::core::ffi::c_int) as uint16,
            -(512 as ::core::ffi::c_int) as uint16,
            1152 as uint16,
            game,
        );
    }
    let mut ridley_var_00: uint16 = (*E).ridley_var_00;
    if ridley_var_00 != 0 {
        (*E).ridley_var_00 =
            (ridley_var_00 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).ridley_var_A =
            (0xa6b4d1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_00 = 20 as uint16;
    };
}
pub unsafe fn Ridley_Func_10(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_10 != 0 {
        Ridley_Func_111(320 as uint16, 0x4000 as uint16, 1280 as uint16, game);
    } else {
        Ridley_Func_111(
            -(320 as ::core::ffi::c_int) as uint16,
            -(16384 as ::core::ffi::c_int) as uint16,
            1280 as uint16,
            game,
        );
    }
    let mut ridley_var_00: uint16 = (*E).ridley_var_00;
    if ridley_var_00 != 0 {
        (*E).ridley_var_00 =
            (ridley_var_00 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).ridley_var_A =
            (0xa6b516 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_00 = 16 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2) =
            1 as uint16;
    };
}
pub unsafe fn Ridley_Func_11(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_10 != 0 {
        Ridley_Func_111(512 as uint16, 30720 as uint16, 768 as uint16, game);
    } else {
        Ridley_Func_111(
            -(512 as ::core::ffi::c_int) as uint16,
            -(30720 as ::core::ffi::c_int) as uint16,
            768 as uint16,
            game,
        );
    }
    let mut ridley_var_00: uint16 = (*E).ridley_var_00;
    if ridley_var_00 != 0 {
        (*E).ridley_var_00 =
            (ridley_var_00 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).ridley_var_A =
            (0xa6b554 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_00 = 32 as uint16;
    };
}
pub unsafe fn Ridley_Func_12(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_10 != 0 {
        Ridley_Func_111(1024 as uint16, 30720 as uint16, 768 as uint16, game);
    } else {
        Ridley_Func_111(
            -(1024 as ::core::ffi::c_int) as uint16,
            -(30720 as ::core::ffi::c_int) as uint16,
            768 as uint16,
            game,
        );
    }
    let mut ridley_var_00: uint16 = (*E).ridley_var_00;
    if ridley_var_00 != 0 {
        (*E).ridley_var_00 =
            (ridley_var_00 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).ridley_var_A =
            (0xa6b594 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_00 = 32 as uint16;
        Ridley_Func_114(game);
    };
}
pub unsafe fn Ridley_Func_13(game: &mut Game) {
    Ridley_Func_111(0 as uint16, 0x8000 as uint16, 448 as uint16, game);
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut ridley_var_00: uint16 = (*E).ridley_var_00;
    if ridley_var_00 != 0 {
        (*E).ridley_var_00 =
            (ridley_var_00 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).ridley_var_A = (if Ridley_Func_40_Carry(game) as ::core::ffi::c_int != 0 {
            0xa6bab7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        } else {
            0xa6b321 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        }) as uint16;
    };
}
pub unsafe fn Ridley_Func_14(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(15) = 11 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9) = 384 as uint16;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_A = (0xa6b5e5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).ridley_var_F = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
        as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int)
        + 32 as ::core::ffi::c_int) as uint16;
    Ridley_Func_15(game);
}
pub unsafe fn Ridley_Func_15(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if Ridley_Func_17(
        *(game.g_rom.offset(
            (((0xa6b60d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa6b60d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((*E).ridley_var_10 as isize),
        game,
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        Ridley_Func_18(game);
    } else {
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
        if (*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).ridley_var_A =
                (0xa6b613 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).ridley_var_F = 128 as uint16;
            Ridley_Func_114(game);
        }
    };
}
pub unsafe fn Ridley_Func_16(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if Ridley_Func_17(
        *(game.g_rom.offset(
            (((0xa6b63b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa6b63b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((*E).ridley_var_10 as isize),
        game,
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        Ridley_Func_18(game);
    } else {
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
        if (*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).ridley_var_A =
                (0xa6b5e5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).ridley_var_F = 128 as uint16;
            Ridley_Func_114(game);
        }
    };
}
pub unsafe fn Ridley_Func_17(mut r18: uint16, game: &mut Game) -> uint8 {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        - 352 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        v0 = 352 as uint16;
    }
    Ridley_Func_104_0(0 as uint16, Ridley_Func_7(game), r18, v0, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2) = 1 as uint16;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as ::core::ffi::c_int
        != 3 as ::core::ffi::c_int
    {
        return 1 as uint8;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as uint8
        as ::core::ffi::c_int
        >= 0x80 as ::core::ffi::c_int
    {
        let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        if (*E).ridley_var_0F == 0
            && (*E).ridley_var_10 as ::core::ffi::c_int != 1 as ::core::ffi::c_int
        {
            Ridley_Func_99(
                addr_kRidley_Ilist_E73A as ::core::ffi::c_int as uint16,
                game,
            );
        }
    }
    return 0 as uint8;
}
pub unsafe fn Ridley_Func_18(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9) = 240 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(15) = 16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) = 1 as uint16;
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_A =
        (0xa6b6a7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Ridley_Func_19(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).base.y_pos as ::core::ffi::c_int - 288 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        Ridley_Func_114(game);
        (*E).ridley_var_A =
            (0xa6b6dd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_F = 32 as uint16;
        Ridley_Func_20(game);
    } else {
        Ridley_Func_104_0(
            0 as uint16,
            0 as uint16,
            *(game.g_rom.offset(
                (((0xa6b6c8 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6b6c8 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((*E).ridley_var_10 as isize),
            288 as uint16,
            game,
        );
    };
}
pub unsafe fn Ridley_Func_20(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    Ridley_Func_104_0(
        0 as uint16,
        0 as uint16,
        (*E).base.x_pos,
        288 as uint16,
        game,
    );
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if (*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        Ridley_Func_71(game);
        Ridley_Func_29(game);
        (*E).ridley_var_A =
            (0xa6b70e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_F = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 0x3f as ::core::ffi::c_int)
            + 128 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Ridley_Func_21(game: &mut Game) {
    let mut v1: int16 = 0;
    if Ridley_Func_25(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        Ridley_Func_24(game);
        Ridley_Func_27(game);
    } else {
        let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(6)
            as ::core::ffi::c_int
            + (*E).ridley_var_C as ::core::ffi::c_int) as int16;
        if v1 as ::core::ffi::c_int - 1536 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        {
            v1 = 1536 as int16;
        }
        (*E).ridley_var_C = v1 as uint16;
        if Ridley_Func_23(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
            game.eproj_spawn_pt = Point16U {
                x: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82),
                y: (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(83)
                    as ::core::ffi::c_int
                    + 12 as ::core::ffi::c_int) as uint16,
            };
            SpawnEprojWithRoomGfx(
                addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
                9 as uint16,
                game,
            );
            QueueSfx2_Max6(0x76 as uint16, game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 13 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 4 as uint16;
            Ridley_Func_28(game);
            Ridley_Func_29(game);
            Ridley_Func_93(
                0x2026 as uint16,
                addr_word_A6D37C as ::core::ffi::c_int as uint16,
                game,
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(20) =
                3072 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(30) =
                3072 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(40) =
                3072 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(50) =
                3072 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(60) =
                3072 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(70) =
                3072 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(80) =
                3072 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) =
                4 as uint16;
            let mut v2: uint16 =
                ((*E).ridley_var_26 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            if (*E).ridley_var_26 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                if (*E).ridley_var_10 as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
                    Ridley_Func_99(
                        addr_kRidley_Ilist_E73A as ::core::ffi::c_int as uint16,
                        game,
                    );
                }
                v2 = 0 as uint16;
            }
            (*E).ridley_var_26 = v2;
            (*E).ridley_var_A =
                (0xa6b7b9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    };
}
pub unsafe fn Ridley_Func_22(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut v1: bool = false;
    if Ridley_Func_41(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 || {
        v1 = (((*E).ridley_var_F as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int;
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
        v1 as ::core::ffi::c_int != 0
    } {
        Ridley_Func_24(game);
        (*E).ridley_var_A =
            (0xa6b321 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        let mut ridley_var_C: uint16 = (*E).ridley_var_C;
        v1 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(7)
            as ::core::ffi::c_int
            + ridley_var_C as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int;
        (*E).ridley_var_C = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset(7) as ::core::ffi::c_int
            + ridley_var_C as ::core::ffi::c_int) as uint16;
        if !v1 {
            (*E).ridley_var_C = 0 as uint16;
            (*E).ridley_var_A =
                (0xa6b70e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    };
}
pub unsafe fn Ridley_Func_23(game: &mut Game) -> uint8 {
    let mut v0: uint8 = (Ridley_Func_103(
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82),
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(83)
            as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as uint16,
        game,
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int) as uint8;
    if v0 == 0 {
        v0 = (Ridley_Func_103(
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(72),
            (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(73)
                as ::core::ffi::c_int
                + 18 as ::core::ffi::c_int) as uint16,
            game,
        ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int) as uint8;
        if v0 == 0 {
            v0 = (Ridley_Func_103(
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(62),
                (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(63)
                    as ::core::ffi::c_int
                    + 18 as ::core::ffi::c_int) as uint16,
                game,
            ) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int) as uint8;
            if v0 == 0 {
                v0 = (Ridley_Func_103(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(52),
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(53)
                        as ::core::ffi::c_int
                        + 18 as ::core::ffi::c_int) as uint16,
                    game,
                ) as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int) as uint8;
                if v0 == 0 {
                    return (Ridley_Func_103(
                        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                            .offset(42),
                        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                            .offset(43) as ::core::ffi::c_int
                            + 18 as ::core::ffi::c_int) as uint16,
                        game,
                    ) as ::core::ffi::c_int
                        & 1 as ::core::ffi::c_int) as uint8;
                }
            }
        }
    }
    return v0;
}
pub unsafe fn Ridley_Func_24(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = 1 as uint16;
}
pub unsafe fn Ridley_Func_25(game: &mut Game) -> uint8 {
    if Ridley_Func_40_Carry(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return (Ridley_Func_26(4 as uint16, 4 as uint16, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int) as uint8;
    } else {
        return 0 as uint8;
    };
}
pub unsafe fn Ridley_Func_26(mut k: uint16, mut j: uint16, game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut rect: Rect16U = Rect16U {
        x: ((*E).base.x_pos as ::core::ffi::c_int
            + *(game.g_rom.offset(
                (((0xa6b9d5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6b9d5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((*E).ridley_var_10 as isize) as ::core::ffi::c_int) as uint16,
        y: ((*E).base.y_pos as ::core::ffi::c_int
            + *(game.g_rom.offset(
                (((0xa6b9db as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6b9db as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    ((*E).ridley_var_1D as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int) as uint16,
        w: k,
        h: j,
    };
    return Ridley_Func_124(rect, game);
}
pub unsafe fn Ridley_Func_27(game: &mut Game) {
    let mut ridley_var_C: int16 = 0;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    ridley_var_C = (*E).ridley_var_C as int16;
    if ridley_var_C as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        ridley_var_C = -(ridley_var_C as ::core::ffi::c_int) as int16;
    }
    if ridley_var_C as ::core::ffi::c_int + 512 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        ridley_var_C = -(512 as ::core::ffi::c_int) as int16;
    }
    (*E).ridley_var_C = ridley_var_C as uint16;
    Ridley_Func_24(game);
    (*E).ridley_var_A = (0xa6bb8f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Ridley_Func_33b(game);
}
pub unsafe fn Ridley_Func_28(game: &mut Game) {
    let mut x_pos: uint16 = 0;
    let mut v4: uint16 = 0;
    let mut current_block: u64;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_B == 0 {
        (*E).ridley_var_B =
            (if (*E).base.x_pos as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
                -(0xc0 as ::core::ffi::c_int)
            } else {
                0xc0 as ::core::ffi::c_int
            }) as uint16;
    }
    if ((*E).ridley_var_B as ::core::ffi::c_int
        ^ (*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int)
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        x_pos = 0;
        x_pos = (*E).base.x_pos;
        if (x_pos as ::core::ffi::c_int - (*E).ridley_var_22 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            if ((x_pos as ::core::ffi::c_int - (*E).ridley_var_23 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                    as ::core::ffi::c_int)
                    < 0x555 as ::core::ffi::c_int
                {
                    return;
                }
                current_block = 7587493754414084492;
            } else {
                current_block = 12558520430340354144;
            }
        } else {
            current_block = 3419095937434320988;
        }
    } else {
        v4 = 0;
        v4 = (*E).base.x_pos;
        if ((v4 as ::core::ffi::c_int - (*E).ridley_var_22 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            current_block = 3419095937434320988;
        } else if (v4 as ::core::ffi::c_int - (*E).ridley_var_23 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            current_block = 12558520430340354144;
        } else if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int)
            < 0x555 as ::core::ffi::c_int
        {
            current_block = 7587493754414084492;
        } else {
            return;
        }
    }
    match current_block {
        3419095937434320988 => {
            if (*E).ridley_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                return;
            }
        }
        12558520430340354144 => {
            if (*E).ridley_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                return;
            }
        }
        _ => {}
    }
    (*E).ridley_var_B = -((*E).ridley_var_B as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Ridley_Func_29(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x5e5)
        as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int;
    let mut r18: uint16 = *(game.g_rom.offset(
        (((0xa6b965 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa6b965 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v0 as isize);
    let mut r20: uint16 = *(game.g_rom.offset(
        (((0xa6b96d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa6b96d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v0 as isize);
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut v2: uint16 = (2 as ::core::ffi::c_int
        * ((*E).ridley_var_12 as ::core::ffi::c_int + 2 as ::core::ffi::c_int))
        as uint16;
    let mut v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(7) =
        *(game.g_rom.offset(
            (((0xa6b94d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa6b94d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(6) =
        *(game.g_rom.offset(
            (((0xa6b959 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa6b959 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
    (*E).ridley_var_C =
        *((RomPtr_A6 as unsafe fn(uint16_t, &mut Game) -> *const uint8)(r20 as uint16_t, game)
            .offset(v2 as isize) as *const uint8 as *mut uint16);
    let mut v4: *const uint8 = RomPtr_A6(r18 as uint16_t, game);
    let mut t: uint16 = *(v4.offset(v2 as isize) as *const uint8 as *mut uint16);
    (*E).ridley_var_B =
        (if (*E).ridley_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            -(t as ::core::ffi::c_int)
        } else {
            t as ::core::ffi::c_int
        }) as uint16;
}
pub unsafe fn Ridley_Func_30(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut ridley_var_14: uint16 = (*E).ridley_var_14;
    let mut v3: uint16 = 0;
    let mut v6: uint16 = 0;
    if ridley_var_14 != 0 {
        let mut v2: int16 =
            (abs16(ridley_var_14) as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as int16;
        if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            v3 = (if ridley_var_14 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
                -(v2 as ::core::ffi::c_int)
            } else {
                v2 as ::core::ffi::c_int
            }) as uint16;
        } else {
            v3 = 0 as uint16;
        }
        (*E).ridley_var_14 = v3;
    }
    let mut ridley_var_15: uint16 = (*E).ridley_var_15;
    if ridley_var_15 != 0 {
        let mut v5: int16 =
            (abs16(ridley_var_15) as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as int16;
        if v5 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            v6 = (if ridley_var_15 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
                -(v5 as ::core::ffi::c_int)
            } else {
                v5 as ::core::ffi::c_int
            }) as uint16;
        } else {
            v6 = 0 as uint16;
        }
        (*E).ridley_var_15 = v6;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) =
        ((*E).ridley_var_14 as ::core::ffi::c_int
            + (*E).base.x_pos as ::core::ffi::c_int
            + *(game.g_rom.offset(
                (((0xa6b9d5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6b9d5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((*E).ridley_var_10 as isize) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) = ((*E).ridley_var_15
        as ::core::ffi::c_int
        + (*E).base.y_pos as ::core::ffi::c_int
        + *(game.g_rom.offset(
            (((0xa6b9db as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa6b9db as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(((*E).ridley_var_1D as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int)
        as uint16;
}
pub unsafe fn Ridley_Func_31(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_14 =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            - ((*E).base.x_pos as ::core::ffi::c_int
                + *(game.g_rom.offset(
                    (((0xa6b9d5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa6b9d5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((*E).ridley_var_10 as isize) as ::core::ffi::c_int)) as uint16;
    (*E).ridley_var_15 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
        as ::core::ffi::c_int
        - ((*E).base.y_pos as ::core::ffi::c_int
            + *(game.g_rom.offset(
                (((0xa6b9db as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6b9db as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    ((*E).ridley_var_1D as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn Ridley_Func_32(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_25 as ::core::ffi::c_int - 10 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xcee) as *mut uint16) != 0 {
            (*E).ridley_var_A =
                (0xa6bd4e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            Ridley_Func_43(game);
        } else {
            (*E).ridley_var_A =
                (0xa6b3ec as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2) =
                1 as uint16;
        }
    } else {
        Ridley_Func_126(game);
        (*E).ridley_var_A =
            (0xa6c53e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        Ridley_C53E(game);
    };
}
pub unsafe fn Ridley_Func_33(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 = 0;
    let mut v2: int16 = 0;
    let mut r18: uint16 = 0;
    let mut r20: uint16 = 0;
    if Ridley_Func_40_Carry(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && (*E).ridley_var_1F == 0
        && {
            r18 = *(game.g_rom.offset(
                (((0xa6bb48 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6bb48 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((*E).ridley_var_10 as isize);
            r20 = ((*E).base.x_pos as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
            (r18 as ::core::ffi::c_int ^ r20 as ::core::ffi::c_int) & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                || {
                    v1 = abs16(r20);
                    v1 as ::core::ffi::c_int - 32 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0
                }
        }
        && (((*E).base.y_pos as ::core::ffi::c_int + 35 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
    {
        v2 = -(16 as ::core::ffi::c_int) as int16;
        if (*E).ridley_var_10 != 0 {
            v2 = 16 as int16;
        }
        Ridley_Func_104_0(
            0 as uint16,
            *(game.g_rom.offset(
                (((0xa6bb4e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6bb4e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((*E).ridley_var_12 as isize),
            (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                + v2 as ::core::ffi::c_int) as uint16,
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - 4 as ::core::ffi::c_int) as uint16,
            game,
        );
        let mut rect: Rect16U = Rect16U {
            x: ((*E).base.x_pos as ::core::ffi::c_int
                + *(game.g_rom.offset(
                    (((0xa6b9d5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa6b9d5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((*E).ridley_var_10 as isize) as ::core::ffi::c_int)
                as uint16,
            y: ((*E).base.y_pos as ::core::ffi::c_int + 35 as ::core::ffi::c_int) as uint16,
            w: 8 as uint16,
            h: 12 as uint16,
        };
        if Ridley_Func_124(rect, game) != 0 {
            (*E).ridley_var_C = -((*E).ridley_var_C as ::core::ffi::c_int) as uint16;
            if (*E).base.health != 0 {
                if *((&raw mut game.g_ram as *mut uint8).offset(0xcee) as *mut uint16) != 0 {
                    Ridley_Func_37(game);
                    (*E).ridley_var_A =
                        (0xa6bd4e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                    Ridley_Func_43(game);
                } else {
                    Ridley_Func_33b(game);
                }
            } else {
                if (*E).ridley_var_1B == 0 {
                    Ridley_Func_37(game);
                }
                Ridley_Func_126(game);
                (*E).ridley_var_A =
                    (0xa6c538 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                Ridley_Func_63(game);
            }
        }
    } else {
        Ridley_Func_32(game);
    };
}
pub unsafe fn Ridley_Func_33b(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_17 = g_word_A6BBEB[(*E).ridley_var_10 as usize];
    v1 = ((*E).base.y_pos as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as int16;
    if (*E).base.y_pos as ::core::ffi::c_int - 320 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        v1 = 256 as int16;
    }
    (*E).ridley_var_18 = v1 as uint16;
    if (*E).ridley_var_1B == 0 {
        Ridley_Func_37(game);
    }
    (*E).ridley_var_A = (0xa6bbc4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).ridley_var_F = 32 as uint16;
    Ridley_Func_34(game);
}
pub unsafe fn Ridley_Func_34(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    Ridley_Func_104_0(
        0 as uint16,
        0 as uint16,
        (*E).ridley_var_17,
        (*E).ridley_var_18,
        game,
    );
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if (*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).ridley_var_A =
            (0xa6bbf1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_F = 32 as uint16;
    }
}
pub unsafe fn Ridley_Func_35(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if (*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(15) =
            8 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9) =
            240 as uint16;
        Ridley_Func_38(game);
        (*E).ridley_var_A =
            (0xa6bc2e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_F = 64 as uint16;
    } else {
        Ridley_Func_104_0(
            0 as uint16,
            0 as uint16,
            (*E).ridley_var_17,
            256 as uint16,
            game,
        );
    };
}
pub unsafe fn Ridley_Func_36(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if (*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(15) =
            16 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9) =
            240 as uint16;
        (*E).ridley_var_A =
            (0xa6b321 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        Ridley_Func_104_0(
            0 as uint16,
            0 as uint16,
            g_word_A6BC62[(*E).ridley_var_10 as usize],
            224 as uint16,
            game,
        );
    };
}
pub unsafe fn Ridley_Func_37(game: &mut Game) {
    Ridley_Func_31(game);
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
    (*E).ridley_var_1B = 1 as uint16;
    CallSomeSamusCode(0 as uint16, game);
    Ridley_Func_119(1 as uint8, game);
}
pub unsafe fn Ridley_Func_38(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) = 1 as uint16;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_01 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).ridley_var_1E = (if Ridley_Func_40_Sign(game) as ::core::ffi::c_int != 0 {
            6 as ::core::ffi::c_int
        } else {
            10 as ::core::ffi::c_int
        }) as uint16;
    }
    (*E).ridley_var_1B = 0 as uint16;
    CallSomeSamusCode(1 as uint16, game);
    Ridley_Func_119(0 as uint8, game);
}
pub unsafe fn Ridley_Func_39(game: &mut Game) {
    let mut ridley_var_01: int16 = 0;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    ridley_var_01 = (*E).ridley_var_01 as int16;
    if ridley_var_01 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        if ridley_var_01 != 0 {
            if Ridley_Func_121(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
                (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                    | kEnemyProps_Tangible as ::core::ffi::c_int)
                    as uint16;
                return;
            }
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Tangible as ::core::ffi::c_int))
                as uint16;
        }
        let mut ridley_var_1E: uint16 = (*E).ridley_var_1E;
        if ridley_var_1E != 0 {
            let mut v3: uint16 =
                (ridley_var_1E as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
            (*E).ridley_var_1E = v3;
            if v3 == 0
                && (*E).ridley_var_01 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                    & !(kEnemyProps_Tangible as ::core::ffi::c_int))
                    as uint16;
            }
        }
    }
}
static byte_A6BD04: [uint8; 28] = [
    0x80 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0xff as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0xff as ::core::ffi::c_int as uint8,
    0xff as ::core::ffi::c_int as uint8,
    0xff as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0xff as ::core::ffi::c_int as uint8,
    0xff as ::core::ffi::c_int as uint8,
    0xff as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
];
unsafe fn Ridley_Func_40_Carry(game: &mut Game) -> uint8 {
    return (byte_A6BD04[*(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as usize]
        as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
}
unsafe fn Ridley_Func_40_Sign(game: &mut Game) -> uint8 {
    return (byte_A6BD04[*(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as usize]
        as ::core::ffi::c_int
        & 0x40 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
}
pub unsafe fn Ridley_Func_41(game: &mut Game) -> uint8 {
    return (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
        as ::core::ffi::c_int
        - 352 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
}
pub unsafe fn Ridley_Func_42(game: &mut Game) {
    let mut ridley_var_01: int16 = 0;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    ridley_var_01 = (*E).ridley_var_01 as int16;
    if ridley_var_01 as ::core::ffi::c_int > 0 as ::core::ffi::c_int
        && ridley_var_01 as ::core::ffi::c_int != 2 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xcee) as *mut uint16) as ::core::ffi::c_int
            != 0
        && (*E).ridley_var_1B == 0
    {
        Ridley_Func_24(game);
        (*E).ridley_var_A =
            (0xa6bab7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Ridley_Func_43(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcee) as *mut uint16) != 0 {
        (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_01 = 2 as uint16;
        let mut r18: uint16 = (if *((&raw mut game.g_ram as *mut uint8).offset(0xce2)
            as *mut uint16) as ::core::ffi::c_int
            - 128 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            192 as ::core::ffi::c_int
        } else {
            80 as ::core::ffi::c_int
        }) as uint16;
        let mut r20: uint16 = (if *((&raw mut game.g_ram as *mut uint8).offset(0xce4)
            as *mut uint16) as ::core::ffi::c_int
            - 256 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            384 as ::core::ffi::c_int
        } else {
            192 as ::core::ffi::c_int
        }) as uint16;
        Ridley_Func_104_0(0 as uint16, Ridley_Func_7(game), r18, r20, game);
    } else {
        let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        (*E).ridley_var_01 = 1 as uint16;
        (*E).ridley_var_A = (if (*E).ridley_var_1B as ::core::ffi::c_int != 0 {
            0xa6bb8f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        } else {
            0xa6b321 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        }) as uint16;
    };
}
pub unsafe fn Ridley_Func_44(game: &mut Game) {
    Ridley_Func_106(0 as uint16, 1 as uint16, 192 as uint16, 128 as uint16, game);
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).base.x_pos as ::core::ffi::c_int - 192 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E).ridley_var_A =
            (0xa6bdbc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Ridley_Func_45(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_20 = -(192 as ::core::ffi::c_int) as uint16;
    Ridley_Func_106(
        0 as uint16,
        1 as uint16,
        192 as uint16,
        -(128 as ::core::ffi::c_int) as uint16,
        game,
    );
    if (*E).base.y_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).ridley_var_40 =
            (0xa6beca as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_A =
            (0xa6bdf2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_F = 21 as uint16;
    }
}
pub unsafe fn Ridley_Func_46(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if (*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        Ridley_Func_99(
            addr_kRidley_Ilist_E658 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).ridley_var_A =
            (0xa6be03 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        Ridley_Func_47(game);
    }
}
pub unsafe fn Ridley_Func_47(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    Ridley_Func_104_0(
        0 as uint16,
        12 as uint16,
        ((*E).ridley_var_42 as ::core::ffi::c_int - 10 as ::core::ffi::c_int) as uint16,
        ((*E).ridley_var_44 as ::core::ffi::c_int - 56 as ::core::ffi::c_int) as uint16,
        game,
    );
    let mut rect: Rect16U = Rect16U {
        x: ((*E).base.x_pos as ::core::ffi::c_int + 14 as ::core::ffi::c_int) as uint16,
        y: ((*E).base.y_pos as ::core::ffi::c_int + 66 as ::core::ffi::c_int) as uint16,
        w: 4 as uint16,
        h: 4 as uint16,
    };
    if Ridley_Func_48(rect, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        (*E).ridley_var_1B = 1 as uint16;
        Ridley_Func_119(1 as uint8, game);
        (*E).ridley_var_C = -(512 as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_40 =
            (0xa6beb3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_A =
            (0xa6a971 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Ridley_Func_48(mut rect: Rect16U, game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 =
        abs16(((*E).ridley_var_42 as ::core::ffi::c_int - rect.x as ::core::ffi::c_int) as uint16);
    let mut v2: bool = (v1 as ::core::ffi::c_int) < 4 as ::core::ffi::c_int;
    let mut v3: uint16 = (v1 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    let mut result: uint8 = 0 as uint8;
    if v2 as ::core::ffi::c_int != 0 || (v3 as ::core::ffi::c_int) < rect.w as ::core::ffi::c_int {
        let mut v4: uint16 = abs16(
            ((*E).ridley_var_44 as ::core::ffi::c_int - rect.y as ::core::ffi::c_int) as uint16,
        );
        v2 = (v4 as ::core::ffi::c_int) < 4 as ::core::ffi::c_int;
        let mut v5: uint16 = (v4 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
        if v2 as ::core::ffi::c_int != 0
            || (v5 as ::core::ffi::c_int) < rect.h as ::core::ffi::c_int
        {
            return 1 as uint8;
        }
    }
    return result;
}
pub unsafe fn CallRidleyFunc_var40(mut ea: uint32, game: &mut Game) {
    match ea {
        10927772 => {
            Ridley_Func_50(game);
            return;
        }
        10927795 => {
            Ridley_Func_51(game);
            return;
        }
        10927818 => {
            Ridley_Func_52(game);
            return;
        }
        10927836 => {
            Ridley_Func_53(game);
            return;
        }
        10927897 => return,
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn Ridley_Func_49(game: &mut Game) {
    CallRidleyFunc_var40(
        ((*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_40 as ::core::ffi::c_int
            | 0xa60000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn Ridley_Func_50(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_42 =
        ((*E).base.x_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
    (*E).ridley_var_44 =
        ((*E).base.y_pos as ::core::ffi::c_int + 22 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Ridley_Func_51(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_42 =
        ((*E).base.x_pos as ::core::ffi::c_int + 14 as ::core::ffi::c_int) as uint16;
    (*E).ridley_var_44 =
        ((*E).base.y_pos as ::core::ffi::c_int + 66 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Ridley_Func_52(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_45 = 0 as uint16;
    (*E).ridley_var_46 = 0 as uint16;
    (*E).ridley_var_40 = (0xa6bedc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Ridley_Func_53(game);
}
pub unsafe fn Ridley_Func_53(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 =
        ((*E).ridley_var_46 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
    (*E).ridley_var_46 = v1;
    let mut t: ::core::ffi::c_int = *(&raw mut (*E).ridley_var_43 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int
        + (v1 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int);
    *(&raw mut (*E).ridley_var_43 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) = t as uint8;
    (*E).ridley_var_44 = ((*E).ridley_var_44 as ::core::ffi::c_int
        + ((v1 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
            + (t >> 8 as ::core::ffi::c_int))) as uint16;
    if (*E).ridley_var_44 as ::core::ffi::c_int - 192 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E).ridley_var_44 = 192 as uint16;
        (*E).ridley_var_40 = addr_locret_A6BF19 as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn DrawBabyMetroid_0(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = BabyMetroid_DBCB_DoubleRetEx(
        (((&raw mut game.g_ram as *mut uint8).offset(0x7800) as *mut ExtraEnemyRam7800
            as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            + 6 as ::core::ffi::c_long) as uint16,
        game,
    );
    if v0 < 0 as ::core::ffi::c_int {
        return;
    }
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    sub_A6DC13(
        v0 as uint16,
        (*E).ridley_var_42,
        (*E).ridley_var_44,
        0 as uint16,
        game,
    );
}
pub unsafe fn BabyMetroid_Instr_2(mut k: uint16, game: &mut Game) -> uint16 {
    if (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_46 == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        return BabyMetroid_Goto(k, game);
    }
    QueueSfx3_Max6(0x24 as uint16, game);
    return (k as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn BabyMetroid_Instr_3(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v1: uint16 = *(RomPtr_A6(k as uint16_t, game) as *mut uint16);
    WriteColorsToPalette(0x162 as uint16, 0xa6 as uint8, v1, 0xf as uint16, game);
    return (k as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn BabyMetroid_Instr_1(mut k: uint16, game: &mut Game) -> uint16 {
    if (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_46 != 0 {
        return BabyMetroid_Goto(k, game);
    } else {
        return (k as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn BabyMetroid_Goto(mut k: uint16, game: &mut Game) -> uint16 {
    return *(RomPtr_A6(k as uint16_t, game) as *mut uint16);
}
pub unsafe fn Ridley_Func_54(game: &mut Game) {
    let mut ridley_var_F: uint16 = (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_F;
    if ridley_var_F != 0 {
        match ridley_var_F as ::core::ffi::c_int {
            2 => {
                Ridley_C08E(game);
            }
            4 => {
                Ridley_C09F(game);
            }
            6 => {
                Ridley_Func_56(game);
            }
            8 => {
                Ridley_Func_57(game);
            }
            10 => {
                Ridley_C104(game);
            }
            12 => {
                Ridley_Func_58(game);
            }
            _ => {
                Unreachable(game);
                loop {}
            }
        }
    } else {
        Ridley_Func_55(game);
    };
}
pub unsafe fn Ridley_Func_55(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(97) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(1);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(99) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(3);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(81) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(17);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(83) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(19);
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_E = -(15157 as ::core::ffi::c_int) as uint16;
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
    Ridley_C08E(game);
}
pub unsafe fn Ridley_C08E(game: &mut Game) {
    if ProcessEscapeTimerTileTransfers(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        (*E).ridley_var_E = -(15106 as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        Ridley_C09F(game);
    }
}
pub unsafe fn Ridley_C09F(game: &mut Game) {
    if ProcessEscapeTimerTileTransfers(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        Ridley_Func_59(game);
        (*E).ridley_var_E = 128 as uint16;
        QueueMusic_Delayed8(7 as uint16, game);
    }
}
pub unsafe fn Ridley_Func_56(game: &mut Game) {
    Ridley_Func_61(game);
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let fresh16 = (*E).ridley_var_E;
    (*E).ridley_var_E = (*E).ridley_var_E.wrapping_sub(1);
    if fresh16 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).ridley_var_3B = -(15280 as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_3C = 0 as uint16;
        (*E).ridley_var_3D = 0 as uint16;
        (*E).ridley_var_3E = 0 as uint16;
        (*E).ridley_var_3F = 0 as uint16;
        (*E).ridley_var_E = 32 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) == 0 {
            (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
            (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        }
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
    }
}
pub unsafe fn Ridley_Func_57(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let fresh15 = (*E).ridley_var_E;
    (*E).ridley_var_E = (*E).ridley_var_E.wrapping_sub(1);
    if fresh15 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        Ridley_Func_62(game);
    }
    Ridley_C104(game);
}
pub unsafe fn Ridley_C104(game: &mut Game) {
    Ridley_Func_61(game);
    if HandleTypewriterText_Ext(0x3582 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
    }
}
pub unsafe fn Ridley_Func_58(game: &mut Game) {
    Ridley_Func_61(game);
    CeresRidley_Func_2(game);
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = 0 as uint16;
    (*E).ridley_var_A = (0xa6aa50 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x943) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) = 2 as uint16;
    SetBossBitForCurArea(1 as uint16, game);
}
pub unsafe fn Ridley_Func_59(game: &mut Game) {
    let mut v2: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    let mut v1: *const uint8 = RomPtr_A6(addr_byte_A6C15D as ::core::ffi::c_int as uint16_t, game);
    v2 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v2).size = *(v1 as *mut uint16);
    *((&raw mut (*v2).src.addr as *mut uint8).offset(1) as *mut VoidP) =
        *(v1.offset(3) as *mut uint16) as VoidP;
    (*v2).src.addr = *(v1.offset(2) as *mut uint16) as VoidP;
    (*v2).vram_dst = *(v1.offset(5) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Ridley_Func_60(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    v1 = ((*E).ridley_var_39 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as int16;
    if (*E).ridley_var_39 as ::core::ffi::c_int - 31 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        v1 = -(31 as ::core::ffi::c_int) as int16;
    }
    (*E).ridley_var_39 = v1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(11) =
        (abs16((*E).ridley_var_39) as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Ridley_Func_61(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x797) as *mut uint16) as ::core::ffi::c_int
        | *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16) as ::core::ffi::c_int
        == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16) as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        let mut v1: uint16 =
            ((*E).ridley_var_38 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        if v1 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
            v1 = 0 as uint16;
        }
        (*E).ridley_var_38 = v1;
        let mut v2: ::core::ffi::c_int = (*E).ridley_var_38 as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int * v1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(97) =
            *(game.g_rom.offset(
                (((0xa6c1df as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6c1df as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v2 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(98) =
            *(game.g_rom.offset(
                (((0xa6c1df as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6c1df as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 + 1 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(99) =
            *(game.g_rom.offset(
                (((0xa6c1df as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6c1df as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 + 2 as ::core::ffi::c_int) as isize);
    }
}
pub unsafe fn SetupZebesEscapeTypewriter(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(157) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(125);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(158) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(126);
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_3B = addr_loc_A9C49C as ::core::ffi::c_int as uint16;
    (*E).mbn_var_3C = 0 as uint16;
    (*E).mbn_var_3D = 0 as uint16;
    (*E).mbn_var_3E = 0 as uint16;
    (*E).mbn_var_3F = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 0 as uint16;
}
pub unsafe fn ProcessEscapeTimerTileTransfers(game: &mut Game) -> uint8 {
    let mut v4: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v0: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
    let mut ai_var_E: uint16 = (*v0).ai_var_E;
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    let mut v3: *const uint8 = RomPtr_A6(ai_var_E as uint16_t, game);
    let mut result: uint8 = 1 as uint8;
    if *(v3 as *mut uint16) != 0 {
        v4 = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*v4).size = *(v3 as *mut uint16);
        *((&raw mut (*v4).src.addr as *mut uint8).offset(1) as *mut VoidP) =
            *(v3.offset(3) as *mut uint16) as VoidP;
        (*v4).src.addr = *(v3.offset(2) as *mut uint16) as VoidP;
        (*v4).vram_dst = *(v3.offset(5) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            (v2 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
        (*v0).ai_var_E = (ai_var_E as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
        if *(RomPtr_A6((*v0).ai_var_E as uint16_t, game) as *mut uint16) != 0 {
            return 0 as uint8;
        }
    }
    return result;
}
pub unsafe fn HandleTypewriterText_Ext(mut a: uint16, game: &mut Game) -> uint8 {
    let mut r18: uint16 = a;
    let mut v4: int16 = 0;
    let mut v12: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let mut mbn_var_3D: uint16 = (*E).mbn_var_3D;
    if mbn_var_3D != 0 {
        (*E).mbn_var_3D = (mbn_var_3D as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        return 0 as uint8;
    } else {
        (*E).mbn_var_3D = (*E).mbn_var_3E;
        let mut i: ::core::ffi::c_int = 0;
        let mut v7: ::core::ffi::c_int = 0;
        i = (*E).mbn_var_3B as ::core::ffi::c_int;
        loop {
            loop {
                v4 = *(RomPtr_A6(i as uint16_t, game) as *mut uint16) as int16;
                if v4 == 0 {
                    return 1 as uint8;
                }
                if v4 as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
                    break;
                }
                let mut v5: uint16 = (i + 2 as ::core::ffi::c_int) as uint16;
                let mut v6: uint16 = *(RomPtr_A6(v5 as uint16_t, game) as *mut uint16);
                (*E).mbn_var_3E = v6;
                i = v5 as ::core::ffi::c_int + 2 as ::core::ffi::c_int;
            }
            if v4 as ::core::ffi::c_int != 13 as ::core::ffi::c_int {
                break;
            }
            v7 = i + 2 as ::core::ffi::c_int;
            let mut v8: uint16 = *(RomPtr_A6(v7 as uint16_t, game) as *mut uint16);
            (*E).mbn_var_3C = v8;
            i = v7 + 2 as ::core::ffi::c_int;
        }
        v4 = v4 as uint8 as int16;
        if v4 as uint8 as ::core::ffi::c_int == 32 as ::core::ffi::c_int {
            (*E).mbn_var_3C = (*E).mbn_var_3C.wrapping_add(1);
            (*E).mbn_var_3B = (i + 1 as ::core::ffi::c_int) as uint16;
            return 0 as uint8;
        } else {
            if v4 as uint8 as ::core::ffi::c_int == 33 as ::core::ffi::c_int {
                v4 = 91 as int16;
            }
            (*E).mbn_var_3B = (i + 1 as ::core::ffi::c_int) as uint16;
            let mut v11: uint16 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
            v12 = gVramWriteEntry(
                *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            );
            (*v12).size = 2 as uint16;
            *((&raw mut (*v12).src.addr as *mut uint8).offset(1) as *mut VoidP) = 32256 as VoidP;
            (*E).mbn_var_3A = (r18 as ::core::ffi::c_int + v4 as ::core::ffi::c_int
                - 65 as ::core::ffi::c_int) as uint16;
            (*v12).src.addr = (((&raw mut game.g_ram as *mut uint8).offset(0x8000)
                as *mut ExtraEnemyRam8000 as *mut uint8)
                .offset_from(&raw mut game.g_ram as *mut uint8)
                as ::core::ffi::c_long
                + 52 as ::core::ffi::c_long) as VoidP;
            let mut mbn_var_3C: uint16 = (*E).mbn_var_3C;
            (*v12).vram_dst = mbn_var_3C;
            (*E).mbn_var_3C =
                (mbn_var_3C as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
                (v11 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
            let mut v14: uint16 =
                ((*E).mbn_var_3F as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            (*E).mbn_var_3F = v14;
            if v14 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                (*E).mbn_var_3F = 0 as uint16;
                if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                    as ::core::ffi::c_int
                    == 6 as ::core::ffi::c_int
                {
                    QueueSfx2_Max3(0x45 as uint16, game);
                } else {
                    QueueSfx3_Max3(0xd as uint16, game);
                }
            }
            return 0 as uint8;
        }
    };
}
pub unsafe fn Ridley_Func_62(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 0;
    let mut v3: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v0: uint16 = addr_stru_A6C3B8 as ::core::ffi::c_int as uint16;
    i = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int;
    loop {
        let mut v2: *const uint8 = RomPtr_A6(v0 as uint16_t, game);
        if *(v2 as *mut uint16) == 0 {
            break;
        }
        v3 = gVramWriteEntry(i, game);
        (*v3).size = *(v2 as *mut uint16);
        *((&raw mut (*v3).src.addr as *mut uint8).offset(1) as *mut VoidP) =
            *(v2.offset(3) as *mut uint16) as VoidP;
        (*v3).src.addr = *(v2.offset(2) as *mut uint16) as VoidP;
        (*v3).vram_dst = *(v2.offset(5) as *mut uint16);
        v0 = (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
        i += 7 as ::core::ffi::c_int;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) = i as uint16;
}
pub unsafe fn Ridley_Func_63(game: &mut Game) {
    if Ridley_Func_68(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        Ridley_C53E(game);
    }
}
pub unsafe fn Ridley_C53E(game: &mut Game) {
    Ridley_Func_99(
        addr_kRidley_Ilist_E6C8 as ::core::ffi::c_int as uint16,
        game,
    );
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_A = (0xa6c551 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).ridley_var_F = 32 as uint16;
}
pub unsafe fn Ridley_C551(game: &mut Game) {
    Ridley_Func_68(game);
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if (*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).ridley_var_B = 0 as uint16;
        (*E).ridley_var_C = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16) = 528 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) = 64 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1980) as *mut uint16) = 1 as uint16;
        (*E).ridley_var_27 = 0 as uint16;
        (*E).ridley_var_28 = 0 as uint16;
        (*E).ridley_var_A =
            (0xa6c588 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_F = 160 as uint16;
        Ridley_Func_64(game);
    }
}
pub unsafe fn Ridley_Func_64(game: &mut Game) {
    Ridley_Func_69(game);
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if (*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if (*E).ridley_var_1B != 0 {
            Ridley_Func_38(game);
        }
        (*E).ridley_var_A =
            (0xa6c5a8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_B = 0 as uint16;
        (*E).ridley_var_C = 0 as uint16;
        RidleysExplosion_Func_2(game);
    }
}
pub unsafe fn Ridley_Func_65(game: &mut Game) {
    Ridley_Func_69(game);
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_02 = 0 as uint16;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
    (*E).ridley_var_A = (0xa6c5c8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).ridley_var_F = 32 as uint16;
}
pub unsafe fn Ridley_Func_66(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if (*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).ridley_var_A =
            (0xa6c5da as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_F = 256 as uint16;
    }
}
pub unsafe fn Ridley_Func_67(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if (*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut k: uint16 = 0 as uint16;
        printf(b"Warning: X undefined\n\0" as *const u8 as *const ::core::ffi::c_char);
        SetBossBitForCurArea(1 as uint16, game);
        Enemy_ItemDrop_Ridley(k, game);
        QueueMusic_Delayed8(3 as uint16, game);
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_A = addr_locret_A6C600 as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn Ridley_Func_68(game: &mut Game) -> uint8 {
    let mut rect: Rect16U = Rect16U {
        x: 128 as uint16,
        y: 328 as uint16,
        w: 4 as uint16,
        h: 4 as uint16,
    };
    Ridley_Func_104(
        0 as uint16,
        0 as uint16,
        0x10 as uint16,
        rect.x,
        rect.y,
        game,
    );
    return Shitroid_Func_2(0 as uint16, rect, game);
}
pub unsafe fn Ridley_Func_69(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    v1 = ((*E).ridley_var_27 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
    if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).ridley_var_27 = 4 as uint16;
        let mut v2: uint16 =
            ((*E).ridley_var_28 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        if (*E).ridley_var_28 as ::core::ffi::c_int - 9 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v2 = 0 as uint16;
        }
        (*E).ridley_var_28 = v2;
        let mut v3: ::core::ffi::c_int = (4 as ::core::ffi::c_int * v2 as ::core::ffi::c_int)
            as uint16 as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        game.eproj_spawn_pt = Point16U {
            x: ((*E).base.x_pos as ::core::ffi::c_int
                + g_word_A6C66E[v3 as usize] as ::core::ffi::c_int) as uint16,
            y: ((*E).base.y_pos as ::core::ffi::c_int
                + g_word_A6C66E[(v3 + 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int)
                as uint16,
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
            3 as uint16,
            game,
        );
        QueueSfx2_Max3(0x24 as uint16, game);
    } else {
        (*E).ridley_var_27 = v1 as uint16;
    };
}
pub unsafe fn RidleysExplosion_Init(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.vram_tiles_index = 0 as uint16;
    (*E).base.palette_index = 3584 as uint16;
    let mut ridley_parameter_1: uint16 = (*E).ridley_parameter_1;
    (*E).ridley_var_F = *(game.g_rom.offset(
        (((0xa6c6ce as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa6c6ce as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((ridley_parameter_1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    let mut Random: uint16 = NextRandom(game);
    (*E).ridley_var_B = (if Random as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(Random as ::core::ffi::c_int & 0x130 as ::core::ffi::c_int)
    } else {
        Random as ::core::ffi::c_int & 0x130 as ::core::ffi::c_int
    }) as uint16;
    (*E).ridley_var_C = 0 as uint16;
    if ridley_parameter_1 != 0 {
        match ridley_parameter_1 as ::core::ffi::c_int {
            2 => {
                (*E).base.x_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32);
                (*E).base.y_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(33);
                (*E).base.current_instruction =
                    addr_kRidleysExplosion_Ilist_CA47 as ::core::ffi::c_int as uint16;
            }
            4 => {
                (*E).base.x_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(42);
                (*E).base.y_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(43);
                (*E).base.current_instruction =
                    addr_kRidleysExplosion_Ilist_CA4D as ::core::ffi::c_int as uint16;
            }
            6 => {
                (*E).base.x_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(52);
                (*E).base.y_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(53);
                (*E).base.current_instruction =
                    addr_kRidleysExplosion_Ilist_CA4D as ::core::ffi::c_int as uint16;
            }
            8 => {
                (*E).base.x_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(62);
                (*E).base.y_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(63);
                (*E).base.current_instruction =
                    addr_kRidleysExplosion_Ilist_CA53 as ::core::ffi::c_int as uint16;
            }
            10 => {
                (*E).base.x_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(72);
                (*E).base.y_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(73);
                (*E).base.current_instruction =
                    addr_kRidleysExplosion_Ilist_CA53 as ::core::ffi::c_int as uint16;
            }
            12 => {
                (*E).base.x_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82);
                (*E).base.y_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(83);
                (*E).base.current_instruction = *(game.g_rom.offset(
                    (((0xa6c7ba as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa6c7ba as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(
                        ((*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                            .offset(71) as *mut uint16 as *mut uint8)
                            .offset(0) as ::core::ffi::c_int
                            + *(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                                .offset(81) as *mut uint16
                                as *mut uint8)
                                .offset(0) as ::core::ffi::c_int
                            + 8 as ::core::ffi::c_int
                            & 0xf0 as ::core::ffi::c_int) as uint8
                            as ::core::ffi::c_int
                            >> 4 as ::core::ffi::c_int) as isize,
                    );
            }
            14 => {
                let mut v10: uint16 = 0 as uint16;
                let mut E0: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
                if (*E0).ridley_var_10 != 0 {
                    v10 = 2 as uint16;
                }
                let mut v13: ::core::ffi::c_int =
                    v10 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                let mut v14: uint16 = ((*E0).base.x_pos as ::core::ffi::c_int
                    + *(game.g_rom.offset(
                        (((0xa6c804 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa6c804 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset(v13 as isize) as ::core::ffi::c_int)
                    as uint16;
                (*E).base.x_pos = v14;
                (*E).base.y_pos = (*E0).base.y_pos;
                (*E).base.current_instruction = *(game.g_rom.offset(
                    (((0xa6c808 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa6c808 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v13 as isize);
            }
            16 => {
                let mut v16: uint16 = 0 as uint16;
                let mut E0_0: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
                if (*E0_0).ridley_var_10 != 0 {
                    v16 = 2 as uint16;
                }
                let mut v19: ::core::ffi::c_int =
                    v16 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                let mut v20: uint16 = ((*E0_0).base.x_pos as ::core::ffi::c_int
                    + *(game.g_rom.offset(
                        (((0xa6c836 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa6c836 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset(v19 as isize) as ::core::ffi::c_int)
                    as uint16;
                (*E).base.x_pos = v20;
                (*E).base.y_pos =
                    ((*E0_0).base.y_pos as ::core::ffi::c_int + 22 as ::core::ffi::c_int) as uint16;
                (*E).base.current_instruction = *(game.g_rom.offset(
                    (((0xa6c83a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa6c83a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v19 as isize);
            }
            18 => {
                let mut v22: uint16 = 0 as uint16;
                let mut E0_1: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
                if (*E0_1).ridley_var_10 != 0 {
                    v22 = 2 as uint16;
                }
                let mut v25: ::core::ffi::c_int =
                    v22 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                let mut v26: uint16 = ((*E0_1).base.x_pos as ::core::ffi::c_int
                    + *(game.g_rom.offset(
                        (((0xa6c868 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa6c868 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset(v25 as isize) as ::core::ffi::c_int)
                    as uint16;
                (*E).base.x_pos = v26;
                (*E).base.y_pos =
                    ((*E0_1).base.y_pos as ::core::ffi::c_int - 24 as ::core::ffi::c_int) as uint16;
                (*E).base.current_instruction = *(game.g_rom.offset(
                    (((0xa6c86c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa6c86c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v25 as isize);
            }
            20 => {
                let mut v28: uint16 = 0 as uint16;
                let mut E0_2: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
                if (*E0_2).ridley_var_10 != 0 {
                    v28 = 2 as uint16;
                }
                let mut v31: ::core::ffi::c_int =
                    v28 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                let mut v32: uint16 = ((*E0_2).base.x_pos as ::core::ffi::c_int
                    + *(game.g_rom.offset(
                        (((0xa6c89a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa6c89a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset(v31 as isize) as ::core::ffi::c_int)
                    as uint16;
                (*E).base.x_pos = v32;
                (*E).base.y_pos = (*E0_2).base.y_pos;
                (*E).base.current_instruction = *(game.g_rom.offset(
                    (((0xa6c89e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa6c89e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v31 as isize);
            }
            22 => {
                let mut v34: uint16 = 0 as uint16;
                let mut E0_3: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
                if (*E0_3).ridley_var_10 != 0 {
                    v34 = 2 as uint16;
                }
                let mut v37: ::core::ffi::c_int =
                    v34 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                let mut v38: uint16 = ((*E0_3).base.x_pos as ::core::ffi::c_int
                    + *(game.g_rom.offset(
                        (((0xa6c8cc as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa6c8cc as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset(v37 as isize) as ::core::ffi::c_int)
                    as uint16;
                (*E).base.x_pos = v38;
                (*E).base.y_pos =
                    ((*E0_3).base.y_pos as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
                (*E).base.current_instruction = *(game.g_rom.offset(
                    (((0xa6c8d0 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa6c8d0 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v37 as isize);
            }
            _ => {
                Unreachable(game);
                loop {}
            }
        }
    } else {
        (*E).base.x_pos =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(22);
        (*E).base.y_pos =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(23);
        (*E).base.current_instruction =
            addr_kRidleysExplosion_Ilist_CA47 as ::core::ffi::c_int as uint16;
    };
}
pub unsafe fn RidleysExplosion_Main(game: &mut Game) {
    let mut v2: int16 = 0;
    RidleysExplosion_Func_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let mut E: *mut Enemy_RidleysExplosion = Get_RidleysExplosion(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    v2 = (abs16((*E).ren_var_B) as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as int16;
    if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        v2 = 0 as int16;
    }
    (*E).ren_var_B = (if (*E).ren_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(v2 as ::core::ffi::c_int)
    } else {
        v2 as ::core::ffi::c_int
    }) as uint16;
    (*E).ren_var_C = ((*E).ren_var_C as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    MoveEnemyWithVelocity(game);
    (*E).ren_var_F = (*E).ren_var_F.wrapping_sub(1);
    if (*E).ren_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        EnemyDeathAnimation(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            0 as uint16,
            game,
        );
    }
}
pub unsafe fn RidleysExplosion_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_RidleysExplosion = Get_RidleysExplosion(k as ::core::ffi::c_int, game);
    let mut v2: uint16 =
        ((*E).base.frame_counter as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint16;
    let mut v3: uint16 = 0;
    (*E).base.frame_counter = v2;
    if v2 != 0 {
        v3 = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
    } else {
        v3 = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    }
    (*E).base.properties = v3;
}
pub unsafe fn RidleysExplosion_Func_2(game: &mut Game) {
    SpawnEnemy(
        0xa6 as uint8,
        addr_stru_A6C9E7 as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnEnemy(
        0xa6 as uint8,
        addr_stru_A6C9D7 as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnEnemy(
        0xa6 as uint8,
        addr_stru_A6C9C7 as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnEnemy(
        0xa6 as uint8,
        addr_stru_A6C9B7 as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnEnemy(
        0xa6 as uint8,
        addr_stru_A6C9A7 as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnEnemy(
        0xa6 as uint8,
        addr_stru_A6C997 as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnEnemy(
        0xa6 as uint8,
        addr_stru_A6C987 as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnEnemy(
        0xa6 as uint8,
        addr_stru_A6C9F7 as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnEnemy(
        0xa6 as uint8,
        addr_stru_A6CA07 as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnEnemy(
        0xa6 as uint8,
        addr_stru_A6CA27 as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnEnemy(
        0xa6 as uint8,
        addr_stru_A6CA17 as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnEnemy(
        0xa6 as uint8,
        addr_stru_A6CA37 as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn locret_A6CB20(game: &mut Game) {}
static off_A6CB21: [Option<Func_V>; 9] = unsafe {
    [
        Some(locret_A6CB20 as unsafe fn(&mut Game) -> ()),
        Some(Ridley_CBC0 as unsafe fn(&mut Game) -> ()),
        Some(Ridley_Func_71 as unsafe fn(&mut Game) -> ()),
        Some(Ridley_Func_72 as unsafe fn(&mut Game) -> ()),
        Some(Ridley_CBC7 as unsafe fn(&mut Game) -> ()),
        Some(Ridley_CBCE as unsafe fn(&mut Game) -> ()),
        Some(Ridley_Func_73 as unsafe fn(&mut Game) -> ()),
        Some(Ridley_Func_74 as unsafe fn(&mut Game) -> ()),
        Some(Ridley_CBD5 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn Ridley_Func_70(game: &mut Game) {
    Ridley_Func_91(game);
    off_A6CB21
        [*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) as usize]
        .expect("non-null function pointer")(game);
    Ridley_Func_86(game);
    Ridley_Func_87(game);
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_1B as ::core::ffi::c_int
        | (*((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16)
            as ::core::ffi::c_int
            | (*E).base.properties as ::core::ffi::c_int
                & kEnemyProps_Tangible as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
        == 0
    {
        Ridley_Func_127(game);
    }
}
pub unsafe fn Ridley_Func_71(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = 8 as uint16;
    Ridley_Func_75(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) = 3 as uint16;
}
pub unsafe fn Ridley_Func_72(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = 8 as uint16;
    Ridley_Func_75(game);
}
pub unsafe fn Ridley_Func_73(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = 3 as uint16;
    Ridley_Func_75(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) = 6 as uint16;
}
pub unsafe fn Ridley_Func_74(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = 2 as uint16;
    Ridley_Func_75(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) = 7 as uint16;
}
pub unsafe fn Ridley_Func_75(game: &mut Game) {
    if (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_10 != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
            0x4000 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14) =
            -(1 as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14) =
            0x4000 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
            -(1 as ::core::ffi::c_int) as uint16;
    }
    Ridley_CBC7(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76)
        as ::core::ffi::c_int
        | (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66)
            as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16)
                as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) =
            4 as uint16;
    }
}
pub unsafe fn Ridley_CBC0(game: &mut Game) {
    Ridley_Func_79(game);
    Ridley_CBDC(game);
}
pub unsafe fn Ridley_CBC7(game: &mut Game) {
    Ridley_Func_83(game);
    Ridley_CBDC(game);
}
pub unsafe fn Ridley_CBCE(game: &mut Game) {
    Ridley_Func_84(game);
    Ridley_CBDC(game);
}
pub unsafe fn Ridley_CBD5(game: &mut Game) {
    Ridley_Func_81(game);
    Ridley_CBDC(game);
}
pub unsafe fn Ridley_CBDC(game: &mut Game) {
    let mut v0: uint16 = 7 as uint16;
    let mut v1: uint16 = 0 as uint16;
    loop {
        Ridley_Func_88(v1, game);
        v1 = (v1 as ::core::ffi::c_int + 20 as ::core::ffi::c_int) as uint16;
        let fresh11 = v0;
        v0 = v0.wrapping_sub(1);
        if !(fresh11 as ::core::ffi::c_int != 1 as ::core::ffi::c_int) {
            break;
        }
    }
}
pub unsafe fn Ridley_Func_77(game: &mut Game) {
    let mut v0: ::core::ffi::c_int =
        (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_10 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(11) =
        *(game.g_rom.offset(
            (((0xa6cc12 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa6cc12 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v0 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(12) =
        *(game.g_rom.offset(
            (((0xa6cc18 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa6cc18 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v0 as isize);
}
pub unsafe fn Ridley_Func_78(game: &mut Game) -> uint8 {
    return (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76)
        as ::core::ffi::c_int
        & (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66)
            as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56)
                as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46)
                as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36)
                as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26)
                as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16)
                as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
}
pub unsafe fn Ridley_Func_79(game: &mut Game) {
    Ridley_Func_77(game);
    if Ridley_Func_78(game) != 0 {
        if Ridley_Func_80_DoubleRet(game) != 0 {
            return;
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76)
        as ::core::ffi::c_int
        | (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66)
            as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16)
                as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16) =
            0x8000 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
            -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14) =
            -(1 as ::core::ffi::c_int) as uint16;
        let mut v0: uint16 = 2 as uint16;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int)
            < 0x70 as ::core::ffi::c_int
        {
            v0 = 1 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = v0;
    }
}
pub unsafe fn Ridley_Func_80_DoubleRet(game: &mut Game) -> uint8 {
    let mut current_block: u64;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut retval: bool = false_0 != 0;
    let mut v1: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2)
        as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13)
            as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14)
                as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        current_block = 5875655901067838705;
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(1) == 0 {
        current_block = 15129621311889290514;
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as uint8
        as ::core::ffi::c_int
        >= 0xf0 as ::core::ffi::c_int
    {
        current_block = 5875655901067838705;
    } else if (abs16(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
    ) as ::core::ffi::c_int)
        < 0x80 as ::core::ffi::c_int
    {
        v1 = 0 as uint16;
        current_block = 993568416893248401;
    } else {
        current_block = 15129621311889290514;
    }
    match current_block {
        5875655901067838705 => {
            v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2)
                as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as uint16;
            current_block = 993568416893248401;
        }
        _ => {}
    }
    match current_block {
        993568416893248401 => {
            Ridley_Func_89(v1, game);
            retval = true_0 != 0;
        }
        _ => {}
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2) = 0 as uint16;
    return retval as uint8;
}
pub unsafe fn Ridley_Func_81(game: &mut Game) {
    Ridley_Func_77(game);
    if Ridley_Func_78(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        if Ridley_Func_82_DoubleRet(game) {
            return;
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76)
        as ::core::ffi::c_int
        | (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66)
            as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16)
                as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16) =
            0x8000 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
            -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14) =
            -(1 as ::core::ffi::c_int) as uint16;
        let mut v0: uint16 = 2 as uint16;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int)
            < 0x70 as ::core::ffi::c_int
        {
            v0 = 1 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = v0;
    }
}
pub unsafe fn Ridley_Func_82_DoubleRet(game: &mut Game) -> bool {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2) != 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13)
            as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14)
                as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            Ridley_Func_89(1 as uint16, game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2) =
                0 as uint16;
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
pub unsafe fn Ridley_Func_83(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    Ridley_Func_77(game);
    if Ridley_Func_78(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as uint8
            as ::core::ffi::c_int
            >= 0xf0 as ::core::ffi::c_int
            || (abs16(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int
                    - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
            ) as ::core::ffi::c_int)
                < 0x80 as ::core::ffi::c_int)
        && *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13)
            as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14)
                as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
            16128 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) =
            8 as uint16;
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76)
        as ::core::ffi::c_int
        | (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66)
            as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16)
                as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        == 0
    {
        if (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_C as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) =
                5 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
            -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14) =
            -(1 as ::core::ffi::c_int) as uint16;
        let mut v1: uint16 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(8);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(8) == 0 || {
            v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(8)
                as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as uint16;
            let ref mut fresh14 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(8);
            *fresh14 = v1;
            *fresh14 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        } {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = v1;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16) =
                0x8000 as uint16;
            Ridley_Func_85(game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) =
                8 as uint16;
        }
    }
}
pub unsafe fn Ridley_Func_84(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    Ridley_Func_77(game);
    if Ridley_Func_78(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as uint8
            as ::core::ffi::c_int
            >= 0xf0 as ::core::ffi::c_int
            || (abs16(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int
                    - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
            ) as ::core::ffi::c_int)
                < 0x80 as ::core::ffi::c_int)
        && *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13)
            as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14)
                as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
            16128 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) =
            8 as uint16;
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76)
        as ::core::ffi::c_int
        | (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66)
            as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16)
                as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        == 0
    {
        if (*E).ridley_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) =
                6 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(20) =
                2560 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(30) =
                2560 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(40) =
                2560 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(50) =
                2560 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(60) =
                2560 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(70) =
                2560 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(80) =
                2560 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(21) =
                0x4000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(31) =
                0x4000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(41) =
                0x4000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(51) =
                0x4000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(61) =
                0x4000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(71) =
                0x4000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(81) =
                0x4000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16) =
                0x8000 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
            -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14) =
            -(1 as ::core::ffi::c_int) as uint16;
        let mut v1: uint16 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(8);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(8) != 0 {
            v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(8)
                as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(8) = v1;
            if v1 == 0 {
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16) =
                    0x8000 as uint16;
                Ridley_Func_85(game);
                v1 = 8 as uint16;
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = v1;
    }
}
pub unsafe fn Ridley_Func_85(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(21) =
        (*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(21)
            as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int
            | 0x4000 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(31) =
        (*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(31)
            as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int
            | 0x4000 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(41) =
        (*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(41)
            as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int
            | 0x4000 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(51) =
        (*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(51)
            as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int
            | 0x4000 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(61) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(21)
            as ::core::ffi::c_int
            & 0x4ff as ::core::ffi::c_int
            | 0x4000 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(71) =
        (*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(71)
            as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int
            | 0x4000 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(81) =
        (*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(81)
            as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int
            | 0x4000 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Ridley_Func_86(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(23) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(25)
            as ::core::ffi::c_int
            + (*E).base.y_pos as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(33) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(35)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(23)
                as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(43) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(45)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(35)
                as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(23)
                as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(53) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(55)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(43)
                as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(63) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(65)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(55)
                as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(43)
                as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(73) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(75)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(63)
                as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(83) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(85)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(75)
                as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(63)
                as ::core::ffi::c_int) as uint16;
    v1 = ((*E).ridley_var_10 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
    if (*E).ridley_var_10 as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
        v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(24)
            as ::core::ffi::c_int
            + g_word_A6CF54[(*E).ridley_var_10 as usize] as ::core::ffi::c_int)
            as int16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(22) =
        ((*E).base.x_pos as ::core::ffi::c_int + v1 as ::core::ffi::c_int) as uint16;
    if (*E).ridley_var_10 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32) =
            (*E).base.x_pos;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(42) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(52) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(62) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(72) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(34)
                as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(22)
                    as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(42) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(44)
                as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(34)
                    as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(22)
                    as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(52) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(54)
                as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(42)
                    as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(62) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(64)
                as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(54)
                    as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(42)
                    as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(72) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(74)
                as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(62)
                    as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(84)
                as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(74)
                    as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(62)
                    as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Ridley_Func_87(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(30) != 0 {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(30)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(29)
                as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(30) =
                0 as uint16;
        }
        let mut v0: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset(9) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(29)
                as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(29)
                as ::core::ffi::c_int
            - 6144 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v0 = 6144 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(29) = v0;
    } else if ((2048 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(29)
            as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        let ref mut fresh5 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(29);
        *fresh5 = (*fresh5 as ::core::ffi::c_int - 128 as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(40) != 0 {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(40)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(39)
                as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(40) =
                0 as uint16;
        }
        let mut v1: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset(9) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(39)
                as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(39)
                as ::core::ffi::c_int
            - 6144 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v1 = 6144 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(39) = v1;
    } else if ((2048 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(39)
            as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        let ref mut fresh6 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(39);
        *fresh6 = (*fresh6 as ::core::ffi::c_int - 128 as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(50) != 0 {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(50)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(49)
                as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(50) =
                0 as uint16;
        }
        let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset(9) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(49)
                as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(49)
                as ::core::ffi::c_int
            - 5632 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v2 = 5632 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(49) = v2;
    } else if ((2048 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(49)
            as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        let ref mut fresh7 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(49);
        *fresh7 = (*fresh7 as ::core::ffi::c_int - 128 as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(60) != 0 {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(60)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(59)
                as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(60) =
                0 as uint16;
        }
        let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset(9) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(59)
                as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(59)
                as ::core::ffi::c_int
            - 5632 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v3 = 5632 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(59) = v3;
    } else if ((2048 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(59)
            as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        let ref mut fresh8 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(59);
        *fresh8 = (*fresh8 as ::core::ffi::c_int - 128 as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(70) != 0 {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(70)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(69)
                as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(70) =
                0 as uint16;
        }
        let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset(9) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(69)
                as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(69)
                as ::core::ffi::c_int
            - 4608 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v4 = 4608 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(69) = v4;
    } else if ((2048 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(69)
            as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        let ref mut fresh9 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(69);
        *fresh9 = (*fresh9 as ::core::ffi::c_int - 128 as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(80) != 0 {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(80)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(79)
                as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(80) =
                0 as uint16;
        }
        let mut v5: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset(9) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(79)
                as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(79)
                as ::core::ffi::c_int
            - 1280 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v5 = 1280 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(79) = v5;
    } else if ((1280 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(79)
            as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        let ref mut fresh10 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(79);
        *fresh10 = (*fresh10 as ::core::ffi::c_int - 128 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Ridley_Func_88(mut k: uint16, game: &mut Game) {
    let mut current_block: u64;
    let mut v6: int16 = 0;
    let mut v2: uint16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v1 + 16 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v5: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 17 as ::core::ffi::c_int) as isize);
        if (v5 as ::core::ffi::c_int)
            < *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(15)
                as ::core::ffi::c_int
        {
            let ref mut fresh12 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(
                    ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 17 as ::core::ffi::c_int) as isize,
                );
            *fresh12 = (*fresh12 as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10)
                    as ::core::ffi::c_int) as uint16;
            return;
        }
        if v5 as ::core::ffi::c_int != 0xffff as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v4 + 17 as ::core::ffi::c_int) as isize) =
                -(1 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v4 + 26 as ::core::ffi::c_int) as isize) = 0x8000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v4 + 28 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v4 + 18 as ::core::ffi::c_int) as isize);
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 18 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14)
                as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                v2 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset(10) as ::core::ffi::c_int
                    + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset((v4 + 21 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int) as uint16;
                if (v2 as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(12) as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int
                {
                    current_block = 4838848238792072683;
                } else {
                    current_block = 4216153537599944117;
                }
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v4 + 20 as ::core::ffi::c_int) as isize) = 3072 as uint16;
                v2 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset(10) as ::core::ffi::c_int
                    + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset((v4 + 21 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int) as uint16;
                if (v2 as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(14) as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int
                {
                    if k == 0
                        || *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                            .offset((v4 + 6 as ::core::ffi::c_int) as isize)
                            == 0
                    {
                        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                            .offset((v4 + 21 as ::core::ffi::c_int) as isize) =
                            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                                .offset(14);
                        current_block = 5689001924483802034;
                    } else {
                        current_block = 4838848238792072683;
                    }
                } else {
                    current_block = 4216153537599944117;
                }
            }
            match current_block {
                5689001924483802034 => {}
                4216153537599944117 => {}
                _ => {
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset((v4 + 18 as ::core::ffi::c_int) as isize) = 0x8000 as uint16;
                    v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(12);
                    current_block = 4216153537599944117;
                }
            }
        } else {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13)
                as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                v6 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v4 + 21 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(10) as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as int16;
                if ((v6 as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(11) as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
                {
                    current_block = 9260896848985358503;
                } else {
                    current_block = 2873832966593178012;
                }
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v4 + 20 as ::core::ffi::c_int) as isize) = 3072 as uint16;
                v6 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v4 + 21 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(10) as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as int16;
                if ((v6 as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(13) as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
                {
                    if k == 0
                        || *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                            .offset((v4 + 6 as ::core::ffi::c_int) as isize)
                            == 0
                    {
                        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                            .offset((v4 + 21 as ::core::ffi::c_int) as isize) =
                            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                                .offset(13);
                        current_block = 5689001924483802034;
                    } else {
                        current_block = 9260896848985358503;
                    }
                } else {
                    current_block = 2873832966593178012;
                }
            }
            match current_block {
                5689001924483802034 => {}
                _ => {
                    match current_block {
                        2873832966593178012 => {
                            v2 = (v6 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
                        }
                        _ => {
                            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                                .offset((v4 + 18 as ::core::ffi::c_int) as isize) = 0 as uint16;
                            v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x2000)
                                as *mut uint16)
                                .offset(11);
                        }
                    }
                    current_block = 4216153537599944117;
                }
            }
        }
        match current_block {
            5689001924483802034 => {
                let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v3 + 16 as ::core::ffi::c_int) as isize) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v3 + 17 as ::core::ffi::c_int) as isize) = 0 as uint16;
                let ref mut fresh13 = *((&raw mut game.g_ram as *mut uint8).offset(0x2000)
                    as *mut uint16)
                    .offset((v3 + 18 as ::core::ffi::c_int) as isize);
                *fresh13 =
                    (*fresh13 as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int) as uint16;
                return;
            }
            _ => {
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v4 + 21 as ::core::ffi::c_int) as isize) = v2;
            }
        }
    } else {
        v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v1 + 21 as ::core::ffi::c_int) as isize);
    }
    let mut r18: uint16 = v2 as uint8 as uint16;
    if k != 0 {
        r18 = (*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(
            ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 11 as ::core::ffi::c_int)
                as isize,
        ) as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int
            + r18 as ::core::ffi::c_int) as uint8 as uint16;
    }
    let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v7 + 24 as ::core::ffi::c_int) as isize) = ComputeSinMult(
        *(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v7 + 19 as ::core::ffi::c_int) as isize) as *mut uint16
            as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as uint16,
        r18,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v7 + 25 as ::core::ffi::c_int) as isize) = ComputeCosMult(
        *(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v7 + 19 as ::core::ffi::c_int) as isize) as *mut uint16
            as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as uint16,
        r18,
        game,
    );
}
pub unsafe fn Ridley_Func_89(mut a: uint16, game: &mut Game) {
    let mut v1: uint16 = 0;
    let mut r18: uint16 = 0;
    let mut r20: uint16 = 0;
    let mut v2: int8 = 0;
    v2 = a as int8;
    *(&raw mut v1 as *mut uint8).offset(0) = 0 as uint8;
    *(&raw mut v1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) = v2 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(5) = v1;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_10 as ::core::ffi::c_int != 1 as ::core::ffi::c_int
        && (*E).ridley_var_01 as ::core::ffi::c_int != 0
    {
        let mut pt: Point16U = Point16U { x: 0, y: 0 };
        if Ridley_Func_90(&raw mut pt, game) != 0 {
            r18 = (pt.x as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(22)
                    as ::core::ffi::c_int) as uint16;
            r20 = (pt.y as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(23)
                    as ::core::ffi::c_int) as uint16;
        } else {
            r18 = (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(22)
                    as ::core::ffi::c_int) as uint16;
            r20 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                + 24 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(23)
                    as ::core::ffi::c_int) as uint16;
        }
        r18 = -(CalculateAngleFromXY(r18, r20, game) as ::core::ffi::c_int
            + 0x80 as ::core::ffi::c_int) as uint8 as uint16;
        if (*E).ridley_var_10 != 0 {
            let mut v7: uint16 = r18;
            if r18 as ::core::ffi::c_int >= 0x18 as ::core::ffi::c_int
                && (r18 as ::core::ffi::c_int) < 0xe8 as ::core::ffi::c_int
            {
                v7 = 24 as uint16;
            }
            let mut v8: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000)
                as *mut uint16)
                .offset(5) as ::core::ffi::c_int
                + v7 as ::core::ffi::c_int
                + 0x4000 as ::core::ffi::c_int) as uint16;
            if v8 as ::core::ffi::c_int
                >= *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(21)
                    as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14) =
                    v8;
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) =
                    8 as uint16;
            }
        } else {
            let mut v5: uint16 = r18;
            if r18 as ::core::ffi::c_int >= 0x18 as ::core::ffi::c_int
                && (r18 as ::core::ffi::c_int) < 0xe8 as ::core::ffi::c_int
            {
                v5 = 232 as uint16;
            }
            let mut v6: uint16 = (v5 as ::core::ffi::c_int + 16128 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(5)
                    as ::core::ffi::c_int) as uint16;
            if (v6 as ::core::ffi::c_int)
                < *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(21)
                    as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
                    v6;
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) =
                    8 as uint16;
            }
        }
    }
}
pub unsafe fn Ridley_Func_90(mut out: *mut Point16U, game: &mut Game) -> uint8 {
    let mut rect: Rect16U = Rect16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(83),
        w: 64 as uint16,
        h: 64 as uint16,
    };
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcce) as *mut uint16) == 0 {
        return 0 as uint8;
    }
    let mut v1: uint16 = 0 as uint16;
    loop {
        let mut v2: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *(((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(v2 as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
            || *(((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
                .offset(v2 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int
                == 2 as ::core::ffi::c_int
        {
            let mut v3: uint16 = abs16(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int
                    - rect.x as ::core::ffi::c_int) as uint16,
            );
            let mut v4: bool = (v3 as ::core::ffi::c_int)
                < *((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int;
            let mut v5: uint16 = (v3 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int)
                as uint16;
            if v4 as ::core::ffi::c_int != 0
                || (v5 as ::core::ffi::c_int) < rect.w as ::core::ffi::c_int
            {
                let mut v6: uint16 = abs16(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                        .offset(v2 as isize) as ::core::ffi::c_int
                        - rect.y as ::core::ffi::c_int) as uint16,
                );
                v4 = (v6 as ::core::ffi::c_int)
                    < *((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                        .offset(v2 as isize) as ::core::ffi::c_int;
                let mut v7: uint16 = (v6 as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                        .offset(v2 as isize) as ::core::ffi::c_int)
                    as uint16;
                if v4 as ::core::ffi::c_int != 0
                    || (v7 as ::core::ffi::c_int) < rect.h as ::core::ffi::c_int
                {
                    break;
                }
            }
        }
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if (v1 as ::core::ffi::c_int - 10 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            return 0 as uint8;
        }
    }
    let mut v8: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*out).x =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v8 as isize);
    (*out).y =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v8 as isize);
    return 1 as uint8;
}
pub unsafe fn Ridley_Func_91(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10)
        as ::core::ffi::c_int
        - 8 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        let mut v1: uint16 =
            ((*E).ridley_var_0E as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_0E = v1;
        if v1 as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            return;
        }
        if (*E).ridley_var_0F == 0 {
            QueueSfx3_Max6(0x21 as uint16, game);
        }
    }
    (*E).ridley_var_0E = 0 as uint16;
}
pub unsafe fn Ridley_Func_92(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(11) =
        16368 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(12) =
        16448 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9) = 240 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(15) = 16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(17) = 17 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(27) = 17 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(37) = 17 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(47) = 17 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(57) = 17 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(67) = 17 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(77) = 17 as uint16;
    Ridley_Func_93(
        0x2024 as uint16,
        addr_word_A6D36E as ::core::ffi::c_int as uint16,
        game,
    );
    Ridley_Func_93(
        0x2026 as uint16,
        addr_word_A6D37C as ::core::ffi::c_int as uint16,
        game,
    );
    Ridley_Func_93(
        0x202a as uint16,
        addr_word_A6D38A as ::core::ffi::c_int as uint16,
        game,
    );
    Ridley_Func_93(
        0x202c as uint16,
        addr_word_A6D398 as ::core::ffi::c_int as uint16,
        game,
    );
    Ridley_Func_93(
        0x202e as uint16,
        addr_word_A6D3A6 as ::core::ffi::c_int as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76) = 0 as uint16;
}
pub unsafe fn Ridley_Func_93(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut dst: *mut uint16 =
        (&raw mut game.g_ram as *mut uint8).offset(j as ::core::ffi::c_int as isize) as *mut uint16;
    let mut src: *const uint16 = RomPtr_A6(k as uint16_t, game) as *const uint16;
    let mut v3: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
    loop {
        *dst = *src;
        dst = dst.offset(10);
        src = src.offset(1);
        v3 -= 1;
        if !(v3 != 0) {
            break;
        }
    }
}
pub unsafe fn Ridley_Func_94(game: &mut Game) {
    Ridley_Func_D3DC(0 as uint16, game);
}
pub unsafe fn Ridley_Func_95(game: &mut Game) {
    Ridley_Func_D3DC(0x8000 as uint16, game);
}
pub unsafe fn Ridley_Func_D3DC(mut a: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16) = a;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26) = a;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36) = a;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46) = a;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56) = a;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66) = a;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76) = a;
}
pub unsafe fn Ridley_Func_96(game: &mut Game) {
    let mut v0: int16 = 0;
    Ridley_Func_93(
        0x2026 as uint16,
        addr_word_A6D37C as ::core::ffi::c_int as uint16,
        game,
    );
    v0 = 7 as int16;
    let mut v1: uint16 = 0 as uint16;
    loop {
        let mut v2: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 21 as ::core::ffi::c_int) as isize) = (0x8000 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v2 + 21 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int)
            as uint16;
        let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 18 as ::core::ffi::c_int) as isize);
        *fresh3 = (*fresh3 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
        v1 = (v1 as ::core::ffi::c_int + 20 as ::core::ffi::c_int) as uint16;
        v0 -= 1;
        if !(v0 != 0) {
            break;
        }
    }
}
pub unsafe fn Ridley_Func_97(game: &mut Game) -> uint8 {
    return (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76)
        as ::core::ffi::c_int
        & (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66)
            as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56)
                as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46)
                as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36)
                as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26)
                as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16)
                as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
}
pub unsafe fn Ridley_Func_98(game: &mut Game) {
    let mut enemy_ptr: uint16 = (*Get_Ridley(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .enemy_ptr;
    let mut v1: *const uint8 = RomPtr_A0(enemy_ptr as uint16_t, game);
    Samus_DealDamage(
        SuitDamageDivision(*(v1.offset(6) as *mut uint16), game),
        game,
    );
}
pub unsafe fn Ridley_Func_99(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).base.current_instruction = a;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn Ridley_Func_100(game: &mut Game) {
    let mut v0: int16 = 0;
    v0 = 0 as int16;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut health: uint16 = (*E).base.health;
    if (health as ::core::ffi::c_int) < 0x2328 as ::core::ffi::c_int {
        v0 = 1 as int16;
        if (health as ::core::ffi::c_int) < 0x1518 as ::core::ffi::c_int {
            v0 = 2 as int16;
            if (health as ::core::ffi::c_int) < 0x708 as ::core::ffi::c_int {
                v0 = 3 as int16;
            }
        }
    }
    (*E).ridley_var_12 = v0 as uint16;
    if (v0 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        Ridley_D495(
            (v0 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
            game,
        );
    }
}
pub unsafe fn Ridley_D495(mut r18: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16) == 0 {
        WriteColorsToPalette(
            0x1e2 as uint16,
            0xa6 as uint8,
            (28 as ::core::ffi::c_int * r18 as ::core::ffi::c_int - 7062 as ::core::ffi::c_int)
                as uint16,
            0xe as uint16,
            game,
        );
    }
}
pub unsafe fn Ridley_Func_101(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_01 as ::core::ffi::c_int != 0
        && (*E).ridley_var_0D as ::core::ffi::c_int >= 0x32 as ::core::ffi::c_int
    {
        Ridley_D495(
            (if (*E).ridley_var_0D as ::core::ffi::c_int >= 0x46 as ::core::ffi::c_int {
                2 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as uint16,
            game,
        );
    }
}
pub unsafe fn Ridley_Func_102(game: &mut Game) {
    let mut v0: int16 = 0;
    v0 = 3584 as int16;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).base.flash_timer as ::core::ffi::c_int >= 2 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xe44) as *mut uint16) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            & 2 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        v0 = 0 as int16;
    }
    (*E).ridley_var_0C = v0 as uint16;
}
pub unsafe fn Ridley_Func_103(mut k: uint16, mut j: uint16, game: &mut Game) -> uint8 {
    let mut prod: uint16 = Mult8x8(
        (j as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8,
    );
    return (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(
        (prod as ::core::ffi::c_int + (k as ::core::ffi::c_int >> 4 as ::core::ffi::c_int))
            as isize,
    ) as ::core::ffi::c_int
        & 0xf000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
}
pub unsafe fn Ridley_Func_104_0(
    mut k: uint16,
    mut j: uint16,
    mut r18: uint16,
    mut r20: uint16,
    game: &mut Game,
) {
    Ridley_Func_104(k, j, 0 as uint16, r18, r20, game);
}
pub unsafe fn Ridley_Func_104(
    mut k: uint16,
    mut j: uint16,
    mut a: uint16,
    mut r18: uint16,
    mut r20: uint16,
    game: &mut Game,
) {
    let mut v4: int16 = 0;
    let mut v6: int16 = 0;
    let mut v8: int16 = 0;
    let mut v9: int16 = 0;
    let mut v11: int16 = 0;
    let mut ridley_var_C: int16 = 0;
    let mut v17: int16 = 0;
    let mut R26: uint16 = a;
    let mut r24: uint16 = g_byte_A6D61F[j as usize] as uint16;
    Ridley_Func_105(k, r18, r24, a, game);
    let mut E: *mut Enemy_Ridley = Get_Ridley(k as ::core::ffi::c_int, game);
    v4 = ((*E).base.y_pos as ::core::ffi::c_int - r20 as ::core::ffi::c_int) as int16;
    if v4 != 0 {
        if v4 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            let mut RegWord: uint16 = SnesDivide(v4 as uint16, r24 as uint8);
            if RegWord == 0 {
                RegWord = 1 as uint16;
            }
            let mut r22: uint16 = RegWord;
            ridley_var_C = (*E).ridley_var_C as int16;
            let mut v14: bool = 0 as ::core::ffi::c_int != 0;
            if ridley_var_C as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                let mut v15: uint16 =
                    (ridley_var_C as ::core::ffi::c_int - R26 as ::core::ffi::c_int) as uint16;
                v15 = (v15 as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
                let mut v16: uint16 = r22;
                ridley_var_C = (v15 as ::core::ffi::c_int - v16 as ::core::ffi::c_int) as int16;
            }
            v17 = (ridley_var_C as ::core::ffi::c_int - r22 as ::core::ffi::c_int) as int16;
            if v17 as ::core::ffi::c_int + 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v17 = -(1280 as ::core::ffi::c_int) as int16;
            }
            (*E).ridley_var_C = v17 as uint16;
        } else {
            let mut v5: uint16 = SnesDivide(
                (r20 as ::core::ffi::c_int - (*E).base.y_pos as ::core::ffi::c_int) as uint16,
                r24 as uint8,
            );
            if v5 == 0 {
                v5 = 1 as uint16;
            }
            let mut r22_0: uint16 = v5;
            v6 = (*E).ridley_var_C as int16;
            let mut v7: bool = 0 as ::core::ffi::c_int != 0;
            if (v6 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                v8 = (R26 as ::core::ffi::c_int + v6 as ::core::ffi::c_int) as int16;
                v8 = (v8 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as int16;
                v9 = v8;
                v6 = (r22_0 as ::core::ffi::c_int + v9 as ::core::ffi::c_int) as int16;
            }
            v11 = (r22_0 as ::core::ffi::c_int + v6 as ::core::ffi::c_int) as int16;
            if v11 as ::core::ffi::c_int - 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v11 = 1280 as int16;
            }
            (*E).ridley_var_C = v11 as uint16;
        }
    }
}
pub unsafe fn Ridley_Func_105(
    mut k: uint16,
    mut r18: uint16,
    mut r24: uint16,
    mut r26: uint16,
    game: &mut Game,
) {
    let mut v2: int16 = 0;
    let mut v4: int16 = 0;
    let mut v6: int16 = 0;
    let mut v7: int16 = 0;
    let mut v9: int16 = 0;
    let mut ridley_var_B: int16 = 0;
    let mut v15: int16 = 0;
    let mut E: *mut Enemy_Ridley = Get_Ridley(k as ::core::ffi::c_int, game);
    v2 = ((*E).base.x_pos as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as int16;
    if v2 != 0 {
        if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            let mut RegWord: uint16 = SnesDivide(v2 as uint16, r24 as uint8);
            if RegWord == 0 {
                RegWord = 1 as uint16;
            }
            let mut r22: uint16 = RegWord;
            ridley_var_B = (*E).ridley_var_B as int16;
            let mut v12: bool = 0 as ::core::ffi::c_int != 0;
            if ridley_var_B as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                let mut v13: uint16 =
                    (ridley_var_B as ::core::ffi::c_int - r26 as ::core::ffi::c_int) as uint16;
                v13 = (v13 as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
                let mut v14: uint16 = r22;
                ridley_var_B = (v13 as ::core::ffi::c_int - v14 as ::core::ffi::c_int) as int16;
            }
            v15 = (ridley_var_B as ::core::ffi::c_int - r22 as ::core::ffi::c_int) as int16;
            if v15 as ::core::ffi::c_int + 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v15 = -(1280 as ::core::ffi::c_int) as int16;
            }
            (*E).ridley_var_B = v15 as uint16;
        } else {
            let mut v3: uint16 = SnesDivide(
                (r18 as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
                r24 as uint8,
            );
            if v3 == 0 {
                v3 = 1 as uint16;
            }
            let mut r22_0: uint16 = v3;
            v4 = (*E).ridley_var_B as int16;
            if (v4 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                v6 = (r26 as ::core::ffi::c_int + v4 as ::core::ffi::c_int) as int16;
                v6 = (v6 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as int16;
                v7 = v6;
                v4 = (r22_0 as ::core::ffi::c_int + v7 as ::core::ffi::c_int) as int16;
            }
            v9 = (r22_0 as ::core::ffi::c_int + v4 as ::core::ffi::c_int) as int16;
            if v9 as ::core::ffi::c_int - 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v9 = 1280 as int16;
            }
            (*E).ridley_var_B = v9 as uint16;
        }
    }
}
pub unsafe fn Ridley_Func_106(
    mut k: uint16,
    mut j: uint16,
    mut r18: uint16,
    mut r20: uint16,
    game: &mut Game,
) {
    let mut v3: int16 = 0;
    let mut v5: int16 = 0;
    let mut v6: int16 = 0;
    let mut ridley_var_C: int16 = 0;
    let mut v9: int16 = 0;
    let mut r24: uint16 = g_byte_A6D712[j as usize] as uint16;
    Ridley_Func_107(k, r18, r24, game);
    let mut E: *mut Enemy_Ridley = Get_Ridley(k as ::core::ffi::c_int, game);
    v3 = ((*E).base.y_pos as ::core::ffi::c_int - r20 as ::core::ffi::c_int) as int16;
    if v3 != 0 {
        if v3 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            let mut RegWord: uint16 = SnesDivide(v3 as uint16, r24 as uint8);
            if RegWord == 0 {
                RegWord = 1 as uint16;
            }
            let mut r22: uint16 = RegWord;
            ridley_var_C = (*E).ridley_var_C as int16;
            if ridley_var_C as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                ridley_var_C = (ridley_var_C as ::core::ffi::c_int
                    - r22 as ::core::ffi::c_int
                    - r22 as ::core::ffi::c_int) as int16;
            }
            v9 = (ridley_var_C as ::core::ffi::c_int - r22 as ::core::ffi::c_int) as int16;
            if v9 as ::core::ffi::c_int + 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v9 = -(1280 as ::core::ffi::c_int) as int16;
            }
            (*E).ridley_var_C = v9 as uint16;
        } else {
            let mut v4: uint16 = SnesDivide(
                (r20 as ::core::ffi::c_int - (*E).base.y_pos as ::core::ffi::c_int) as uint16,
                r24 as uint8,
            );
            if v4 == 0 {
                v4 = 1 as uint16;
            }
            let mut r22_0: uint16 = v4;
            v5 = (*E).ridley_var_C as int16;
            if (v5 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                v5 = (v5 as ::core::ffi::c_int
                    + (r22_0 as ::core::ffi::c_int + r22_0 as ::core::ffi::c_int))
                    as int16;
            }
            v6 = (r22_0 as ::core::ffi::c_int + v5 as ::core::ffi::c_int) as int16;
            if v6 as ::core::ffi::c_int - 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v6 = 1280 as int16;
            }
            (*E).ridley_var_C = v6 as uint16;
        }
    }
}
pub unsafe fn Ridley_Func_107(mut k: uint16, mut r18: uint16, mut r24: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(k as ::core::ffi::c_int, game);
    let mut v2: int16 =
        ((*E).base.x_pos as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as int16;
    if v2 == 0 {
        return;
    }
    if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        let mut RegWord: uint16 = SnesDivide(v2 as uint16, r24 as uint8);
        RegWord = (if RegWord as ::core::ffi::c_int != 0 {
            RegWord as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        }) as uint16;
        let mut r22: uint16 = RegWord;
        let mut s: int16 = (*E).ridley_var_B as int16;
        if s as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            s = (s as ::core::ffi::c_int - 2 as ::core::ffi::c_int * RegWord as ::core::ffi::c_int)
                as int16;
        }
        s = (s as ::core::ffi::c_int - RegWord as ::core::ffi::c_int) as int16;
        (*E).ridley_var_B = (if s as ::core::ffi::c_int + 1280 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            -(1280 as ::core::ffi::c_int)
        } else {
            s as ::core::ffi::c_int
        }) as uint16;
    } else {
        let mut v3: uint16 = SnesDivide(
            (r18 as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
            r24 as uint8,
        );
        let mut r22_0: uint16 = (if v3 as ::core::ffi::c_int != 0 {
            v3 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        }) as uint16;
        let mut s_0: int16 = (*E).ridley_var_B as int16;
        if (s_0 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            s_0 = (s_0 as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int * r22_0 as ::core::ffi::c_int) as int16;
        }
        s_0 = (s_0 as ::core::ffi::c_int + r22_0 as ::core::ffi::c_int) as int16;
        (*E).ridley_var_B = (if s_0 as ::core::ffi::c_int - 1280 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            1280 as ::core::ffi::c_int
        } else {
            s_0 as ::core::ffi::c_int
        }) as uint16;
    };
}
pub unsafe fn Ridley_Func_111(mut r18: uint16, mut r20: uint16, mut r22: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if r22 as ::core::ffi::c_int != (*E).ridley_var_0B as ::core::ffi::c_int {
        if (r22 as ::core::ffi::c_int - (*E).ridley_var_0B as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            let mut v2: uint16 =
                ((*E).ridley_var_0B as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
            if v2 as ::core::ffi::c_int - r22 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v2 = r22;
            }
            (*E).ridley_var_0B = v2;
        } else {
            let mut v1: uint16 =
                ((*E).ridley_var_0B as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
            if v1 as ::core::ffi::c_int - r22 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v1 = r22;
            }
            (*E).ridley_var_0B = v1;
        }
    }
    let mut v3: uint16 = 0;
    let mut current_block_13: u64;
    if r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        v3 = ((*E).ridley_var_0A as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
        if v3 as ::core::ffi::c_int - r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        {
            current_block_13 = 872684080230000114;
        } else {
            current_block_13 = 12039483399334584727;
        }
    } else {
        v3 = ((*E).ridley_var_0A as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
        if v3 as ::core::ffi::c_int - r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        {
            current_block_13 = 872684080230000114;
        } else {
            current_block_13 = 12039483399334584727;
        }
    }
    match current_block_13 {
        872684080230000114 => {
            v3 = r20;
        }
        _ => {}
    }
    (*E).ridley_var_0A = v3;
    r18 = *(&raw mut v3 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as uint16;
    (*E).ridley_var_B = Math_MultBySin((*E).ridley_var_0B, r18);
    (*E).ridley_var_C = Math_MultByCos((*E).ridley_var_0B, r18);
}
pub unsafe fn Ridley_Func_112(game: &mut Game) {
    let mut Ridley: *mut Enemy_Ridley = ::core::ptr::null_mut::<Enemy_Ridley>();
    let mut v1: uint16 = 0;
    let mut v2: *mut Enemy_Ridley = ::core::ptr::null_mut::<Enemy_Ridley>();
    let mut ridley_var_23: uint16 = 0;
    let mut ridley_var_21: uint16 = 0;
    Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*Ridley).ridley_var_1F = 0 as uint16;
    v1 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
    v2 = Get_Ridley(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut with_carry: ::core::ffi::c_int = *(&raw mut (*v2).base.x_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int
        + *(&raw mut (*v2).ridley_var_B as *mut uint8).offset(0) as ::core::ffi::c_int;
    *(&raw mut (*v2).base.x_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) = with_carry as uint8;
    ridley_var_23 = ((*v2).base.x_pos as ::core::ffi::c_int
        + ((*v2).ridley_var_B as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8
            as ::core::ffi::c_int
        + (with_carry >> 8 as ::core::ffi::c_int)) as uint16;
    if (ridley_var_23 as ::core::ffi::c_int - (*Ridley).ridley_var_22 as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        if (ridley_var_23 as ::core::ffi::c_int - (*Ridley).ridley_var_23 as ::core::ffi::c_int)
            as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            (*v2).ridley_var_B = 0 as uint16;
            (*Ridley).ridley_var_1F = 2 as uint16;
            ridley_var_23 = (*Ridley).ridley_var_23;
        }
        (*v2).base.x_pos = ridley_var_23;
    } else {
        Ridley_Func_113(v1, game);
        (*v2).base.x_pos = (*Ridley).ridley_var_22;
        (*v2).ridley_var_B = 0 as uint16;
        (*Ridley).ridley_var_1F = 1 as uint16;
    }
    with_carry = *(&raw mut (*v2).base.y_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int
        + *(&raw mut (*v2).ridley_var_C as *mut uint8).offset(0) as ::core::ffi::c_int;
    *(&raw mut (*v2).base.y_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) = with_carry as uint8;
    ridley_var_21 = ((*v2).base.y_pos as ::core::ffi::c_int
        + ((*v2).ridley_var_C as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8
            as ::core::ffi::c_int
        + (with_carry >> 8 as ::core::ffi::c_int)) as uint16;
    if (ridley_var_21 as ::core::ffi::c_int - (*Ridley).ridley_var_20 as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        if (ridley_var_21 as ::core::ffi::c_int - (*Ridley).ridley_var_21 as ::core::ffi::c_int)
            as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            (*v2).ridley_var_C = 0 as uint16;
            (*Ridley).ridley_var_1F = 8 as uint16;
            ridley_var_21 = (*Ridley).ridley_var_21;
        }
        (*v2).base.y_pos = ridley_var_21;
    } else {
        (*v2).base.y_pos = (*Ridley).ridley_var_20;
        (*v2).ridley_var_C = 0 as uint16;
        (*Ridley).ridley_var_1F = 4 as uint16;
    };
}
pub unsafe fn Ridley_Func_113(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as ::core::ffi::c_int
        != 2 as ::core::ffi::c_int
    {
        let mut E: *mut Enemy_Ridley = Get_Ridley(k as ::core::ffi::c_int, game);
        let mut r18: uint16 = abs16((*E).ridley_var_B);
        let mut v2: uint16 = abs16((*E).ridley_var_C);
        if (v2 as ::core::ffi::c_int) < r18 as ::core::ffi::c_int {
            v2 = r18;
        }
        if v2 as ::core::ffi::c_int >= 0x280 as ::core::ffi::c_int {
            let mut v3: uint16 = 0;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16)
                as ::core::ffi::c_int
                == 2 as ::core::ffi::c_int
            {
                v3 = 24 as uint16;
            } else {
                v3 = 33 as uint16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = v3;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 12 as uint16;
        }
    }
}
pub unsafe fn Ridley_Func_114(game: &mut Game) {
    let mut current_block: u64;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut ridley_var_10: uint16 = (*E).ridley_var_10;
    let mut v2: uint16 = 0;
    if ridley_var_10 == 0 {
        if (*((&raw mut (*E).base.enemy_ptr as *mut uint8).offset(1) as *mut int16)
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            return;
        }
        v2 = addr_kRidley_Ilist_E6F0 as ::core::ffi::c_int as uint16;
        current_block = 16875472102381009524;
    } else if ridley_var_10 as ::core::ffi::c_int != 1 as ::core::ffi::c_int
        && (*((&raw mut (*E).base.enemy_ptr as *mut uint8).offset(1) as *mut int16)
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
    {
        v2 = addr_kRidley_Ilist_E706 as ::core::ffi::c_int as uint16;
        current_block = 16875472102381009524;
    } else {
        current_block = 14523784380283086299;
    }
    match current_block {
        16875472102381009524 => {
            (*E).base.current_instruction = v2;
            (*E).base.instruction_timer = 2 as uint16;
            (*E).base.timer = 0 as uint16;
        }
        _ => {}
    };
}
pub unsafe fn Ridley_Func_115(game: &mut Game) {
    let mut v1: int16 = 0;
    Ridley_Func_116(game);
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    v1 = ((*E).ridley_var_09 as ::core::ffi::c_int - (*E).ridley_var_08 as ::core::ffi::c_int)
        as int16;
    (*E).ridley_var_09 = v1 as uint16;
    if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).ridley_var_09 = 32 as uint16;
        let mut v2: uint16 =
            ((*E).ridley_var_07 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        if v2 as ::core::ffi::c_int >= 0xa as ::core::ffi::c_int {
            v2 = 0 as uint16;
        }
        (*E).ridley_var_07 = v2;
    }
}
pub unsafe fn Ridley_Func_116(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut r18: uint16 = abs16((*E).ridley_var_B);
    let mut v1: uint16 =
        (r18 as ::core::ffi::c_int + abs16((*E).ridley_var_C) as ::core::ffi::c_int) as uint16;
    if v1 != 0 {
        let mut v2: uint16 = (v1 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
        if v2 as ::core::ffi::c_int - r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        {
            v2 = r18;
        }
        let mut v5: ::core::ffi::c_int = (4 as ::core::ffi::c_int * v2 as ::core::ffi::c_int
            & 0xf00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int;
        if v5 >= 7 as ::core::ffi::c_int {
            v5 = 7 as ::core::ffi::c_int;
        }
        let mut v6: uint16 = g_word_A6D9ED[v5 as usize];
        if (*E).ridley_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            v6 = (v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        }
        (*E).ridley_var_08 = v6;
    } else {
        (*E).ridley_var_08 = 0 as uint16;
    };
}
pub unsafe fn Ridley_Func_117(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_05 = addr_word_A6DA71 as ::core::ffi::c_int as uint16;
    (*E).ridley_var_06 = 1 as uint16;
}
pub unsafe fn Ridley_Func_118(game: &mut Game) {
    let mut v7: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 =
        ((*E).ridley_var_06 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E).ridley_var_06 = v1;
    if v1 == 0 {
        let mut i: ::core::ffi::c_int = 0;
        let mut v3: uint16 = 0;
        let mut v4: *mut uint16 = ::core::ptr::null_mut::<uint16>();
        i = (*E).ridley_var_05 as ::core::ffi::c_int;
        loop {
            v3 = i as uint16;
            v4 = RomPtr_A6(i as uint16_t, game) as *mut uint16;
            i = *v4 as ::core::ffi::c_int;
            if *v4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                break;
            }
            (*E).ridley_var_05 = i as uint16;
        }
        (*E).ridley_var_06 = i as uint16;
        let mut v6: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
        v7 = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        *((&raw mut (*v7).src.addr as *mut uint8).offset(1) as *mut VoidP) =
            -(20480 as ::core::ffi::c_int) as VoidP;
        *((&raw mut (*v7.offset(1)).src.addr as *mut uint8).offset(1) as *mut VoidP) =
            -(20480 as ::core::ffi::c_int) as VoidP;
        (*v7).src.addr = *v4.offset(1) as VoidP;
        (*v7.offset(1)).src.addr = *v4.offset(2) as VoidP;
        (*v7).vram_dst = 29216 as uint16;
        (*v7.offset(1)).vram_dst = 29472 as uint16;
        (*v7).size = 64 as uint16;
        (*v7.offset(1)).size = 64 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            (v6 as ::core::ffi::c_int + 14 as ::core::ffi::c_int) as uint16;
        (*gVramWriteEntry(v6 as ::core::ffi::c_int + 14 as ::core::ffi::c_int, game)).size =
            0 as uint16;
        (*E).ridley_var_05 = (v3 as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Ridley_Func_119(mut carry: uint8, game: &mut Game) {
    let mut v3: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v1: uint16 = addr_off_A6DAD0 as ::core::ffi::c_int as uint16;
    if carry != 0 {
        v1 = addr_off_A6DAD4 as ::core::ffi::c_int as uint16;
    }
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    v3 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut (*v3).src.addr as *mut uint8).offset(1) as *mut VoidP) =
        -(20480 as ::core::ffi::c_int) as VoidP;
    *((&raw mut (*v3.offset(1)).src.addr as *mut uint8).offset(1) as *mut VoidP) =
        -(20480 as ::core::ffi::c_int) as VoidP;
    let mut v4: *const uint8 = RomPtr_A6(v1 as uint16_t, game);
    (*v3).src.addr = *(v4 as *mut uint16) as VoidP;
    (*v3.offset(1)).src.addr = *(v4.offset(2) as *mut uint16) as VoidP;
    (*v3).vram_dst = 31424 as uint16;
    (*v3.offset(1)).vram_dst = 31680 as uint16;
    (*v3).size = 128 as uint16;
    (*v3.offset(1)).size = 128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v2 as ::core::ffi::c_int + 14 as ::core::ffi::c_int) as uint16;
    (*gVramWriteEntry(v2 as ::core::ffi::c_int + 14 as ::core::ffi::c_int, game)).size =
        0 as uint16;
}
pub unsafe fn Ridley_Func_120(game: &mut Game) {
    let mut ridley_var_10: int16 = 0;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    ridley_var_10 = (*E).ridley_var_10 as int16;
    if ridley_var_10 != 0 {
        if ridley_var_10 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            return;
        }
        ridley_var_10 = 10 as int16;
    }
    let mut v2: uint16 = *(game.g_rom.offset(
        (((0xa6db02 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa6db02 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((*E).ridley_var_07 as ::core::ffi::c_int + ridley_var_10 as ::core::ffi::c_int)
                as isize,
        );
    sub_A6DC13(
        v2,
        (*E).base.x_pos,
        (*E).base.y_pos,
        (*E).ridley_var_0C,
        game,
    );
}
pub unsafe fn sub_A6DB2A(game: &mut Game) {
    if (*gEnemyData(0 as ::core::ffi::c_int, game)).properties as ::core::ffi::c_int
        & kEnemyProps_Invisible as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        sub_A6DBC2(
            *(game.g_rom.offset(
                (((0xa6dcba as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6dcba as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    ((*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(71) as *mut uint16 as *mut uint8)
                        .offset(0) as ::core::ffi::c_int
                        + *(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                            .offset(81) as *mut uint16 as *mut uint8)
                            .offset(0) as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        & 0xf0 as ::core::ffi::c_int) as uint8
                        as ::core::ffi::c_int
                        >> 4 as ::core::ffi::c_int) as isize,
                ),
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82),
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(83),
            game,
        );
        sub_A6DBC2(
            0xdc9e as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(72),
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(73),
            game,
        );
        sub_A6DBC2(
            0xdc9e as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(62),
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(63),
            game,
        );
        sub_A6DBC2(
            0xdc97 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(52),
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(53),
            game,
        );
        sub_A6DBC2(
            0xdc97 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(42),
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(43),
            game,
        );
        sub_A6DBC2(
            0xdc90 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32),
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(33),
            game,
        );
        sub_A6DBC2(
            0xdc90 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(22),
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(23),
            game,
        );
    }
}
pub unsafe fn sub_A6DBC2(mut j: uint16, mut x: uint16, mut y: uint16, game: &mut Game) {
    sub_A6DC13(
        j,
        x,
        y,
        (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_0C,
        game,
    );
}
pub unsafe fn CallBabyMetroidInstr(mut ea: uint32, mut k: uint16, game: &mut Game) -> uint16 {
    match ea {
        10928073 => return BabyMetroid_Instr_2(k, game),
        10928097 => return BabyMetroid_Instr_3(k, game),
        10928114 => return BabyMetroid_Instr_1(k, game),
        10928120 => return BabyMetroid_Goto(k, game),
        _ => return Unreachable(game) as uint16,
    };
}
unsafe fn BabyMetroid_DBCB_DoubleRetEx(mut a: uint16, game: &mut Game) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut st: *mut BabyMetroidExecState = (&raw mut game.g_ram as *mut uint8).offset(a as isize)
        as *mut uint8 as *mut BabyMetroidExecState;
    if (*st).ip as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    let mut v2: uint16 = (*st).ip;
    let mut v3: *const uint16 = RomPtr_A6(v2 as uint16_t, game) as *mut uint16;
    if *v3.offset(0) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        current_block = 14076424727262896545;
    } else {
        if (*st).timer as ::core::ffi::c_int != *v3.offset(0) as ::core::ffi::c_int {
            (*st).timer = (*st).timer.wrapping_add(1);
            return *v3.offset(1) as ::core::ffi::c_int;
        }
        v2 = (v2 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
        current_block = 6937071982253665452;
    }
    loop {
        match current_block {
            14076424727262896545 => {
                v2 = CallBabyMetroidInstr(
                    (*v3.offset(0) as ::core::ffi::c_int | 0xa60000 as ::core::ffi::c_int)
                        as uint32,
                    (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
                    game,
                );
                current_block = 6937071982253665452;
            }
            _ => {
                v3 = RomPtr_A6(v2 as uint16_t, game) as *mut uint16;
                if *v3.offset(0) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
                    break;
                } else {
                    current_block = 14076424727262896545;
                }
            }
        }
    }
    (*st).timer = 1 as uint16;
    (*st).ip = v2;
    return *v3.offset(1) as ::core::ffi::c_int;
}
pub unsafe fn sub_A6DC13(
    mut j: uint16,
    mut r18: uint16,
    mut r20: uint16,
    mut r22: uint16,
    game: &mut Game,
) {
    let mut p: *const uint8 = RomPtr_A6(j as uint16_t, game);
    let mut n: ::core::ffi::c_int = *(p as *mut uint16) as ::core::ffi::c_int;
    p = p.offset(2);
    let mut idx: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) as ::core::ffi::c_int;
    loop {
        let mut v9: int16 = (r20 as ::core::ffi::c_int + *p.offset(2) as int8 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int) as int16;
        if v9 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int
            && v9 as ::core::ffi::c_int - 224 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
        {
            let mut x: uint16 = (r18 as ::core::ffi::c_int
                + *(p as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
            let mut v11: *mut OamEnt = gOamEnt(idx, game);
            (*v11).xcoord = x as uint8;
            (*v11).ycoord = v9 as uint8;
            *(&raw mut (*v11).charnum as *mut uint16) = (r22 as ::core::ffi::c_int
                | *(p.offset(3) as *mut uint16) as ::core::ffi::c_int)
                as uint16;
            let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8).offset(0x570)
                as *mut uint16)
                .offset((idx >> 5 as ::core::ffi::c_int) as isize);
            *fresh4 = (*fresh4 as ::core::ffi::c_int
                | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int)
                    >> 8 as ::core::ffi::c_int
                    | ((*(p as *mut int16) as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        * 2 as ::core::ffi::c_int)
                    << 2 as ::core::ffi::c_int
                        * (idx >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
                as uint16;
            idx = idx + 4 as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int;
        }
        p = p.offset(5);
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = idx as uint16;
}
pub unsafe fn Ridley_Func_121(game: &mut Game) -> uint8 {
    let mut y_pos: int16 = 0;
    let mut v2: int16 = 0;
    let mut x_pos: int16 = 0;
    let mut v4: int16 = 0;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    y_pos = (*E).base.y_pos as int16;
    let mut result: uint8 = 1 as uint8;
    if y_pos as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        v2 = (y_pos as ::core::ffi::c_int + 32 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int) as int16;
        if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            if v2 as ::core::ffi::c_int - 288 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                x_pos = (*E).base.x_pos as int16;
                if x_pos as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                    v4 = (x_pos as ::core::ffi::c_int + 32 as ::core::ffi::c_int
                        - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                            as ::core::ffi::c_int) as int16;
                    if v4 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                        if v4 as ::core::ffi::c_int - 320 as ::core::ffi::c_int
                            & 0x8000 as ::core::ffi::c_int
                            != 0
                        {
                            return 0 as uint8;
                        }
                    }
                }
            }
        }
    }
    return result;
}
pub unsafe fn Ridley_Func_122(mut rect: Rect16U, game: &mut Game) -> uint16 {
    let mut v2: int16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcce) as *mut uint16) == 0 {
        return 0xffff as uint16;
    }
    let mut result: uint16 = 0 as uint16;
    loop {
        let mut v1: ::core::ffi::c_int = result as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        v2 = *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(v1 as isize)
            as int16;
        if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            if (*(&raw mut v2 as *mut uint8).offset(
                (::core::mem::size_of::<int16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)
                - 3 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                let mut v3: uint16 = abs16(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int
                        - rect.x as ::core::ffi::c_int) as uint16,
                );
                let mut v4: bool = (v3 as ::core::ffi::c_int)
                    < *((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int;
                let mut v5: uint16 = (v3 as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int)
                    as uint16;
                if v4 as ::core::ffi::c_int != 0
                    || (v5 as ::core::ffi::c_int) < rect.w as ::core::ffi::c_int
                {
                    let mut v6: uint16 = abs16(
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                            .offset(v1 as isize) as ::core::ffi::c_int
                            - rect.y as ::core::ffi::c_int) as uint16,
                    );
                    v4 = (v6 as ::core::ffi::c_int)
                        < *((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                            .offset(v1 as isize) as ::core::ffi::c_int;
                    let mut v7: uint16 = (v6 as ::core::ffi::c_int
                        - *((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                            .offset(v1 as isize) as ::core::ffi::c_int)
                        as uint16;
                    if v4 as ::core::ffi::c_int != 0
                        || (v7 as ::core::ffi::c_int) < rect.h as ::core::ffi::c_int
                    {
                        break;
                    }
                }
            }
        }
        result = (result as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if (result as ::core::ffi::c_int - 10 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            return 0xffff as uint16;
        }
    }
    let mut v8: ::core::ffi::c_int = result as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v8 as isize) =
        rect.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v8 as isize) =
        rect.y;
    let ref mut fresh2 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16).offset(v8 as isize);
    *fresh2 = (*fresh2 as ::core::ffi::c_int | 0x10 as ::core::ffi::c_int) as uint16;
    return result;
}
pub unsafe fn Ridley_Func_123(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 7 as ::core::ffi::c_int
    {
        v2 = 1 as uint16;
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 2 as ::core::ffi::c_int
    {
        v2 = 8 as uint16;
    } else {
        v2 = 5 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16).offset(v1 as isize) = v2;
}
pub unsafe fn Ridley_Func_124(mut rect: Rect16U, game: &mut Game) -> uint8 {
    let mut v0: uint16 = abs16(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            - rect.x as ::core::ffi::c_int) as uint16,
    );
    let mut v1: bool = (v0 as ::core::ffi::c_int)
        < *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16) as ::core::ffi::c_int;
    let mut v2: uint16 = (v0 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    if v1 as ::core::ffi::c_int != 0 || (v2 as ::core::ffi::c_int) < rect.w as ::core::ffi::c_int {
        let mut v3: uint16 = abs16(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - rect.y as ::core::ffi::c_int) as uint16,
        );
        v1 = (v3 as ::core::ffi::c_int)
            < *((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16)
                as ::core::ffi::c_int;
        let mut v4: uint16 = (v3 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
        if v1 as ::core::ffi::c_int != 0
            || (v4 as ::core::ffi::c_int) < rect.h as ::core::ffi::c_int
        {
            return 1 as uint8;
        }
    }
    return 0 as uint8;
}
pub unsafe fn sub_A6DF59(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn sub_A6DF60(game: &mut Game) {
    Ridley_Func_125(game);
}
pub unsafe fn Ridley_Func_125(game: &mut Game) {
    Ridley_Func_98(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) = 96 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x18aa) as *mut uint16) = 5 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa54) as *mut uint16) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            - (*Get_Ridley(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            ))
            .base
            .x_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16;
}
pub unsafe fn Ridley_Shot(game: &mut Game) {
    let mut v0: int16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as ::core::ffi::c_int
        == 2 as ::core::ffi::c_int
    {
        NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    } else {
        v0 = 13 as int16;
        let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        let mut flash_timer: uint16 = (*E).base.flash_timer;
        if flash_timer != 0 {
            if flash_timer as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
                v0 = 14 as int16;
            }
        }
        (*E).base.flash_timer = v0 as uint16;
        (*E).ridley_var_0D = (*E).ridley_var_0D.wrapping_add(1);
    };
}
pub unsafe fn Ridley_Powerbomb(game: &mut Game) {
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn Ridley_Func_126(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).base.health == 0
        && (*E).ridley_var_01 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        (*E).ridley_var_01 = -(1 as ::core::ffi::c_int) as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_A =
            (0xa6c538 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Ridley_Func_127(game: &mut Game) {
    let mut rect: Rect16U = Rect16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(83),
        w: 14 as uint16,
        h: 14 as uint16,
    };
    if Ridley_Func_124(rect, game) != 0 {
        let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        Samus_DealDamage(SuitDamageDivision((*E).ridley_var_1C, game), game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) = 96 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18aa) as *mut uint16) = 5 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa54) as *mut uint16) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82)
                    as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn Ridley_Func_128(game: &mut Game) {
    let mut E: *mut Enemy_Ridley = Get_Ridley(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut r18: uint16 = -(CalculateAngleFromXY(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(0)
            as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(0)
            as ::core::ffi::c_int
            - (*E).base.y_pos as ::core::ffi::c_int) as uint16,
        game,
    ) as ::core::ffi::c_int) as uint8 as uint16;
    let mut v1: uint16 = (4 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0xc2c) as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16) as isize)
            as ::core::ffi::c_int) as uint16;
    if v1 as ::core::ffi::c_int >= 0x300 as ::core::ffi::c_int {
        v1 = 768 as uint16;
    }
    let mut a: uint16 = v1;
    r18 = Math_MultBySin(v1, r18);
    if ((*E).ridley_var_B as ::core::ffi::c_int ^ r18 as ::core::ffi::c_int)
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).ridley_var_B =
            ((*E).ridley_var_B as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
    }
    r18 = Math_MultByCos(a, r18);
    if ((*E).ridley_var_C as ::core::ffi::c_int ^ r18 as ::core::ffi::c_int)
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).ridley_var_C =
            ((*E).ridley_var_C as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Ridley_Func_129(game: &mut Game) {
    if (*Get_Ridley(0 as ::core::ffi::c_int, game)).base.properties as ::core::ffi::c_int
        & kEnemyProps_Tangible as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut v0: uint16 = Ridley_Func_122(
            Rect16U {
                x: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82),
                y: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(83),
                w: 14 as uint16,
                h: 14 as uint16,
            },
            game,
        );
        if v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            v0 = Ridley_Func_122(
                Rect16U {
                    x: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(72),
                    y: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(73),
                    w: 10 as uint16,
                    h: 10 as uint16,
                },
                game,
            );
            if v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
                return;
            }
        }
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        game.eproj_spawn_pt = Point16U {
            x: *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
                .offset(v1 as isize),
            y: *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                .offset(v1 as isize),
        };
        let mut v2: uint16 = 12 as uint16;
        if *(((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(v1 as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
        {
            QueueSfx1_Max6(0x3d as uint16, game);
            v2 = 6 as uint16;
        }
        SpawnEprojWithRoomGfx(0xe509 as uint16, v2, game);
    }
}
pub unsafe fn Ridley_Instr_5(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_0F = 89 as uint16;
    QueueSfx2_Max6(0x59 as uint16, game);
    return jp;
}
pub unsafe fn Ridley_Instr_6(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_0F = 0 as uint16;
    return jp;
}
pub unsafe fn Ridley_Instr_10(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as ::core::ffi::c_int
        == 2 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) as ::core::ffi::c_int
            - 30 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        return jp.offset(1);
    }
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_00 = 8 as uint16;
    return *jp.offset(0) as uintptr_t as *const uint16;
}
pub unsafe fn Ridley_Instr_4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_1B != 0 {
        return *jp.offset(0) as uintptr_t as *const uint16;
    } else {
        return *jp.offset(1) as uintptr_t as *const uint16;
    };
}
pub unsafe fn Ridley_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_1B != 0 {
        return jp.offset(1);
    } else {
        return *jp.offset(0) as uintptr_t as *const uint16;
    };
}
pub unsafe fn Ridley_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_1D = *jp.offset(0);
    return jp.offset(1);
}
pub unsafe fn Ridley_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_10 != 0 {
        return *jp.offset(0) as uintptr_t as *const uint16;
    } else {
        return jp.offset(1);
    };
}
pub unsafe fn Ridley_Instr_14(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos =
        ((*E).base.x_pos as ::core::ffi::c_int + *jp.offset(0) as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos =
        ((*E).base.y_pos as ::core::ffi::c_int + *jp.offset(1) as ::core::ffi::c_int) as uint16;
    return jp.offset(2);
}
pub unsafe fn Ridley_Instr_9(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_10 = 0 as uint16;
    Ridley_Func_96(game);
    return jp;
}
pub unsafe fn Ridley_Instr_7(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_10 = 1 as uint16;
    return jp;
}
pub unsafe fn Ridley_Instr_8(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_10 = 2 as uint16;
    Ridley_Func_96(game);
    return jp;
}
pub unsafe fn Ridley_Func_131(mut k: uint16, game: &mut Game) {
    Ridley_Func_132(k, 0 as uint16, game);
    Ridley_Func_132(k, 1 as uint16, game);
    Ridley_Func_132(k, 2 as uint16, game);
    Ridley_Func_132(k, 3 as uint16, game);
}
pub unsafe fn Ridley_Func_132(mut k: uint16, mut a: uint16, game: &mut Game) {
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_parameter_1 = a;
    SpawnEprojWithGfx(a, k, addr_stru_869634 as ::core::ffi::c_int as uint16, game);
}
pub unsafe fn Ridley_Instr_11(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v3: uint16 = 0;
    let mut current_block: u64;
    let mut r18: uint16 = 0;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_10 != 0 {
        let mut v4: uint16 = -(CalculateAngleFromXY(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - ((*E).base.x_pos as ::core::ffi::c_int + 25 as ::core::ffi::c_int))
                as uint16,
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - ((*E).base.y_pos as ::core::ffi::c_int - 43 as ::core::ffi::c_int))
                as uint16,
            game,
        ) as ::core::ffi::c_int
            + 0x80 as ::core::ffi::c_int) as uint8 as uint16;
        if (v4 as ::core::ffi::c_int) < 0x50 as ::core::ffi::c_int {
            if v4 as ::core::ffi::c_int >= 0x15 as ::core::ffi::c_int {
                current_block = 14379314559172793947;
            } else {
                current_block = 11006700562992250127;
            }
        } else if (v4 as ::core::ffi::c_int) < 0xc0 as ::core::ffi::c_int {
            v4 = 80 as uint16;
            current_block = 14379314559172793947;
        } else {
            current_block = 11006700562992250127;
        }
        match current_block {
            11006700562992250127 => {
                v4 = 21 as uint16;
            }
            _ => {}
        }
        r18 = v4;
    } else {
        v3 = 0;
        v3 = -(CalculateAngleFromXY(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - ((*E).base.x_pos as ::core::ffi::c_int - 25 as ::core::ffi::c_int))
                as uint16,
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - ((*E).base.y_pos as ::core::ffi::c_int - 43 as ::core::ffi::c_int))
                as uint16,
            game,
        ) as ::core::ffi::c_int
            + 0x80 as ::core::ffi::c_int) as uint8 as uint16;
        if v3 as ::core::ffi::c_int >= 0xb0 as ::core::ffi::c_int {
            if (v3 as ::core::ffi::c_int) < 0xeb as ::core::ffi::c_int {
                current_block = 16663838568255336676;
            } else {
                current_block = 11874048045265102773;
            }
        } else if (v3 as ::core::ffi::c_int) < 0x40 as ::core::ffi::c_int {
            current_block = 11874048045265102773;
        } else {
            v3 = 176 as uint16;
            current_block = 16663838568255336676;
        }
        match current_block {
            11874048045265102773 => {
                v3 = 235 as uint16;
            }
            _ => {}
        }
        r18 = v3;
    }
    (*E).ridley_var_19 = Math_MultBySin(0x500 as uint16, r18);
    (*E).ridley_var_1A = Math_MultByCos(0x500 as uint16, r18);
    return jp;
}
pub unsafe fn Ridley_Instr_12(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return Ridley_E90C(k, jp, 0 as uint16, game);
}
pub unsafe fn Ridley_Instr_13(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return Ridley_E90C(k, jp, 0xe as uint16, game);
}
pub unsafe fn Ridley_E90C(
    mut k: uint16,
    mut jp: *const uint16,
    mut a: uint16,
    game: &mut Game,
) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = a;
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    SpawnEprojWithRoomGfx(0x9642 as uint16, (*E).ridley_var_10, game);
    return jp;
}
pub unsafe fn Ridley_Instr_15(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_A = (0xa6a6af as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).ridley_var_C = -(352 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn Ridley_Instr_16(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_A = (0xa6b2f3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).ridley_var_C = -(352 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn CeresSteam_Init(game: &mut Game) {
    let mut E: *mut Enemy_CeresSteam = Get_CeresSteam(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.vram_tiles_index = 0 as uint16;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_DisableSamusColl as ::core::ffi::c_int) as uint16;
    (*E).base.extra_properties =
        ((*E).base.extra_properties as ::core::ffi::c_int | 4 as ::core::ffi::c_int) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.palette_index = 2560 as uint16;
    (*E).csm_var_D = ((NextRandom(game) as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
        + 1 as ::core::ffi::c_int) as uint16;
    let mut v1: ::core::ffi::c_int = (*E).csm_parameter_1 as ::core::ffi::c_int;
    (*E).base.current_instruction = *(game.g_rom.offset(
        (((0xa6eff5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa6eff5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize);
    (*E).csm_var_A = *(game.g_rom.offset(
        (((0xa6f001 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa6f001 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize);
}
pub unsafe fn CallCeresSteamFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        10940404 => return,
        10940441 => {
            CeresSteam_Func_1(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CeresSteam_Main(game: &mut Game) {
    let mut E: *mut Enemy_CeresSteam = Get_CeresSteam(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.health = 0x7fff as uint16;
    CallCeresSteamFunc(
        ((*E).csm_var_A as ::core::ffi::c_int | 0xa60000 as ::core::ffi::c_int) as uint32,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn CeresSteam_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_CeresSteam = Get_CeresSteam(k as ::core::ffi::c_int, game);
    let mut pt: Point16U = CalcCeresSteamPos_Mode7(
        Point16U {
            x: (*E).base.x_pos,
            y: (*E).base.y_pos,
        },
        game,
    );
    let mut v3: *mut EnemySpawnData = gEnemySpawnData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v3).xpos2 = (pt.x as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int) as uint16;
    (*v3).ypos2 = (pt.y as ::core::ffi::c_int - (*E).base.y_pos as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CeresSteam_Touch(game: &mut Game) {
    (*Get_CeresSteam(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .health = 0x7fff as uint16;
    NormalEnemyTouchAi(game);
}
pub unsafe fn CeresSteam_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_CeresSteam = Get_CeresSteam(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_Tangible as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    return jp;
}
pub unsafe fn CeresSteam_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_CeresSteam = Get_CeresSteam(k as ::core::ffi::c_int, game);
    let fresh0 = (*E).csm_var_D;
    (*E).csm_var_D = (*E).csm_var_D.wrapping_sub(1);
    if fresh0 as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
        return *jp.offset(0) as uintptr_t as *const uint16;
    }
    return CeresSteam_Instr_3(
        k,
        RomPtr_A6(*jp.offset(1) as uint16_t, game) as *mut uint16,
        game,
    );
}
pub unsafe fn CeresSteam_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_CeresSteam = Get_CeresSteam(k as ::core::ffi::c_int, game);
    (*E).base.properties =
        ((*E).base.properties as ::core::ffi::c_int & 0xfaff as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn CeresDoor_Instr_6(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_CeresDoor = Get_CeresDoor(k as ::core::ffi::c_int, game);
    if abs16(
        ((*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int) as uint16,
    ) as ::core::ffi::c_int
        >= 0x30 as ::core::ffi::c_int
        || abs16(
            ((*E).base.y_pos as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int
            >= 0x30 as ::core::ffi::c_int
    {
        return *jp as uintptr_t as *const uint16;
    } else {
        return jp.offset(1);
    };
}
pub unsafe fn CeresDoor_Instr_4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if *((&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        return jp.offset(1);
    } else {
        return *jp as uintptr_t as *const uint16;
    };
}
pub unsafe fn CeresDoor_Instr_8(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) != 0 {
        return jp.offset(1);
    } else {
        return *jp as uintptr_t as *const uint16;
    };
}
pub unsafe fn CeresDoor_Func_6b(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) = 0x8000 as uint16;
    }
}
pub unsafe fn CeresSteam_Instr_4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_CeresSteam = Get_CeresSteam(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn CeresDoor_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_CeresDoor = Get_CeresDoor(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
    return jp;
}
pub unsafe fn CeresDoor_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_CeresDoor(k as ::core::ffi::c_int, game)).cdr_var_B = 1 as uint16;
    return jp;
}
pub unsafe fn CeresSteam_Instr_5(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_CeresSteam = Get_CeresSteam(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn CeresDoor_Instr_5(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_CeresDoor(k as ::core::ffi::c_int, game)).cdr_var_B = 0 as uint16;
    return CeresDoor_Instr_2(k, jp, game);
}
pub unsafe fn CeresDoor_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_CeresDoor = Get_CeresDoor(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    return jp;
}
pub unsafe fn CeresDoor_Instr_7(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx3_Max6(0x2c as uint16, game);
    return jp;
}
pub unsafe fn CeresDoor_Init(game: &mut Game) {
    let mut E: *mut Enemy_CeresDoor = Get_CeresDoor(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.spritemap_pointer = addr_kCeresDoor_Sprmap_FAC7 as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.vram_tiles_index = 0 as uint16;
    (*E).base.palette_index = 1024 as uint16;
    let mut v2: ::core::ffi::c_int = (*E).cdr_parameter_1 as ::core::ffi::c_int;
    (*E).cdr_var_A = *(game.g_rom.offset(
        (((0xa6f72b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa6f72b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v2 as isize);
    (*E).base.current_instruction = *(game.g_rom.offset(
        (((0xa6f52c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa6f52c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v2 as isize);
    (*E).cdr_var_B = 0 as uint16;
    CeresDoor_Func_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let mut v3: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) != 0 {
        v3 = (addr_word_A6F50C as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    } else {
        if (*E).cdr_parameter_1 as ::core::ffi::c_int == 3 as ::core::ffi::c_int {
            WriteColorsToTargetPalette(
                0xa6 as uint8,
                0x142 as uint16,
                (addr_kCeresDoor_Palette as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
                0xf as uint16,
                game,
            );
            return;
        }
        v3 = (addr_kCeresDoor_Palette as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
    (*Get_CeresDoor(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .palette_index = 3584 as uint16;
    WriteColorsToTargetPalette(0xa6 as uint8, 0x1e2 as uint16, v3, 0xf as uint16, game);
}
pub unsafe fn CeresDoor_Func_1(mut k: uint16, game: &mut Game) {
    let mut v2: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    if (*Get_CeresDoor(k as ::core::ffi::c_int, game)).cdr_parameter_1 as ::core::ffi::c_int
        == 2 as ::core::ffi::c_int
    {
        let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
        v2 = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*v2).size = 1024 as uint16;
        *((&raw mut (*v2).src.addr as *mut uint8).offset(1) as *mut VoidP) =
            -(20480 as ::core::ffi::c_int) as VoidP;
        (*v2).src.addr = -(15360 as ::core::ffi::c_int) as VoidP;
        (*v2).vram_dst = 28672 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            (v1 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CallCeresDoor(mut ea: uint32, game: &mut Game) {
    let mut k: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
    match ea {
        10942315 => {
            CeresDoor_Func_2(game);
            return;
        }
        10942320 => {
            CeresDoor_Func_3(game);
            return;
        }
        10942373 => {
            CeresDoor_Func_4(game);
            return;
        }
        10942397 => {
            CeresDoor_Func_5(k, game);
            return;
        }
        10942428 => {
            CeresDoor_Func_6(k, game);
            return;
        }
        10942544 => {
            CeresDoor_Func_7(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CeresDoor_Main(game: &mut Game) {
    let mut E: *mut Enemy_CeresDoor = Get_CeresDoor(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallCeresDoor(
        ((*E).cdr_var_A as ::core::ffi::c_int | 0xa60000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn CeresDoor_Func_2(game: &mut Game) {
    CeresDoor_F773(0x14 as uint16, game);
}
pub unsafe fn CeresDoor_Func_3(game: &mut Game) {
    CeresDoor_F773(0x1d as uint16, game);
}
pub unsafe fn CeresDoor_F773(mut j: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) as ::core::ffi::c_int
        >= 2 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) == 0
    {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 0xfff as ::core::ffi::c_int)
            < 0x80 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 4 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) =
                (j as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 2 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = j;
        }
    }
}
pub unsafe fn CeresDoor_Func_4(game: &mut Game) {
    CeresSteam_Instr_5(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        ::core::ptr::null::<uint16>(),
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*Get_CeresDoor(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .base
        .palette_index = 3584 as uint16;
        CeresDoor_Instr_2(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            ::core::ptr::null::<uint16>(),
            game,
        );
    }
}
pub unsafe fn CeresDoor_Func_5(mut k: uint16, game: &mut Game) {
    CeresDoor_Func_7(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) as ::core::ffi::c_int
        >= 2 as ::core::ffi::c_int
    {
        let mut E: *mut Enemy_CeresDoor = Get_CeresDoor(k as ::core::ffi::c_int, game);
        (*E).cdr_var_A = (0xa6f7dc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).cdr_var_D = 48 as uint16;
        let mut E0: *mut Enemy_CeresDoor = Get_CeresDoor(0 as ::core::ffi::c_int, game);
        (*E0).cdr_var_E = 0 as uint16;
        (*E0).cdr_var_F = 0 as uint16;
    }
}
pub unsafe fn CeresDoor_Func_6(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_CeresDoor = Get_CeresDoor(k as ::core::ffi::c_int, game);
    (*E).cdr_var_D = (*E).cdr_var_D.wrapping_sub(1);
    let mut v3: bool = (*E).cdr_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v3 {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
        (*E).cdr_var_A = (0xa6f850 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        CeresDoor_Func_6b(game);
    } else {
        let mut E0: *mut Enemy_CeresDoor = Get_CeresDoor(0 as ::core::ffi::c_int, game);
        (*E0).cdr_var_E = (*E0).cdr_var_E.wrapping_sub(1);
        v3 = (*E0).cdr_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
        if v3 {
            (*E0).cdr_var_E = 4 as uint16;
            (*E0).cdr_var_F = (*E0).cdr_var_F.wrapping_sub(1);
            v3 = (*E0).cdr_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int;
            if v3 {
                (*E0).cdr_var_F = 3 as uint16;
            }
            let mut v5: ::core::ffi::c_int = (4 as ::core::ffi::c_int
                * (*E0).cdr_var_F as ::core::ffi::c_int)
                as uint16 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            game.eproj_spawn_pt = Point16U {
                x: ((*E).base.x_pos as ::core::ffi::c_int
                    + *(game.g_rom.offset(
                        (((0xa6f840 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa6f840 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset(v5 as isize) as ::core::ffi::c_int) as uint16,
                y: ((*E).base.y_pos as ::core::ffi::c_int
                    + *(game.g_rom.offset(
                        (((0xa6f840 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa6f840 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset((v5 + 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int) as uint16,
            };
            let mut v6: uint16 = 3 as uint16;
            if (NextRandom(game) as ::core::ffi::c_int) < 0x4000 as ::core::ffi::c_int {
                v6 = 12 as uint16;
            }
            SpawnEprojWithRoomGfx(0xe509 as uint16, v6, game);
            QueueSfx2_Max6(0x25 as uint16, game);
        }
    };
}
pub unsafe fn CeresDoor_Func_7(game: &mut Game) {
    CeresDoor_Func_8(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16) == 0 {
        WriteColorsToPalette(
            0x52 as uint16,
            0xa6 as uint8,
            (2 as ::core::ffi::c_int
                * (*((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16)
                    as ::core::ffi::c_int
                    & 0x38 as ::core::ffi::c_int)
                - 0x78f as ::core::ffi::c_int) as uint16,
            6 as uint16,
            game,
        );
    }
}
pub unsafe fn CeresDoor_Func_8(game: &mut Game) {
    QueueMode7Transfers(
        0xa6 as uint8,
        *(game.g_rom.offset(
            (((0xa6f900 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa6f900 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                ((*((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16)
                    as ::core::ffi::c_int
                    & 2 as ::core::ffi::c_int)
                    >> 1 as ::core::ffi::c_int) as isize,
            ),
        game,
    );
}
pub unsafe fn Zebetites_Init(game: &mut Game) {
    let mut E: *mut Enemy_Zebetites = Get_Zebetites(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties =
        ((*E).base.properties as ::core::ffi::c_int | 0xa000 as ::core::ffi::c_int) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.palette_index = 1024 as uint16;
    (*E).base.vram_tiles_index = 128 as uint16;
    (*E).zebet_var_C = 0 as uint16;
    let mut v1: uint16 = (0xa6fc41 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if (*E).zebet_parameter_1 != 0 {
        v1 = (0xa6fc5b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    (*E).zebet_var_A = v1;
    let mut r18: uint16 = 0 as uint16;
    r18 = ((r18 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
        | CheckEventHappened(5 as uint16, game) as ::core::ffi::c_int) as uint16;
    r18 = ((r18 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
        | CheckEventHappened(4 as uint16, game) as ::core::ffi::c_int) as uint16;
    r18 = ((r18 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
        | CheckEventHappened(3 as uint16, game) as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = r18;
    (*E).zebet_var_D = r18;
    if v2 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        let mut v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int;
        (*E).zebet_var_F = *(game.g_rom.offset(
            (((0xa6fc03 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa6fc03 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
        (*E).base.y_height = *(game.g_rom.offset(
            (((0xa6fc0b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa6fc0b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
        (*E).base.current_instruction = *(game.g_rom.offset(
            (((0xa6fc13 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa6fc13 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
        (*E).base.x_pos = *(game.g_rom.offset(
            (((0xa6fc1b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa6fc1b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
        let mut v4: uint16 = 0;
        if (*E).zebet_parameter_1 != 0 {
            v4 = *(game.g_rom.offset(
                (((0xa6fc2b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6fc2b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v3 as isize);
        } else {
            v4 = *(game.g_rom.offset(
                (((0xa6fc23 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa6fc23 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v3 as isize);
        }
        (*E).base.y_pos = v4;
    } else {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn CallZebetitesFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        10943553 => {
            Zebetites_Func_1(k, game);
            return;
        }
        10943579 => {
            Zebetites_Func_2(k, game);
            return;
        }
        10943591 => {
            Zebetites_Func_3(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn Zebetites_Main(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) == 0 {
        (*Get_Zebetites(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .base
        .shake_timer = 0 as uint16;
    }
    let mut E: *mut Enemy_Zebetites = Get_Zebetites(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallZebetitesFunc(
        ((*E).zebet_var_A as ::core::ffi::c_int | 0xa60000 as ::core::ffi::c_int) as uint32,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn Zebetites_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Zebetites = Get_Zebetites(k as ::core::ffi::c_int, game);
    if (*E).zebet_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut new_k: uint16 = Zebetites_Func_9(game);
        (*Get_Zebetites(new_k as ::core::ffi::c_int, game)).zebet_parameter_2 = k;
        (*Get_Zebetites(k as ::core::ffi::c_int, game)).zebet_parameter_2 = new_k;
    }
    (*Get_Zebetites(k as ::core::ffi::c_int, game)).zebet_var_A =
        (0xa6fc5b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Zebetites_Func_2(k, game);
}
pub unsafe fn Zebetites_Func_2(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x795) as *mut uint16) == 0 {
        (*Get_Zebetites(k as ::core::ffi::c_int, game)).zebet_var_A =
            (0xa6fc67 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        Zebetites_Func_3(game);
    }
}
pub unsafe fn Zebetites_Func_3(game: &mut Game) {
    let mut v3: int16 = 0;
    Zebetites_Func_4(game);
    Zebetites_Func_5(game);
    let mut E: *mut Enemy_Zebetites = Get_Zebetites(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut health: uint16 = (*E).base.health;
    if health != 0 {
        v3 = (health as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as int16;
        if v3 as ::core::ffi::c_int - 1000 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        {
            v3 = 1000 as int16;
        }
        (*E).base.health = v3 as uint16;
    } else if (*E).zebet_parameter_1 as ::core::ffi::c_int != 0 || {
        Zebetites_Func_6(game);
        (*E).zebet_var_D as ::core::ffi::c_int - 4 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    } {
        EnemyDeathAnimation(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            0 as uint16,
            game,
        );
    } else {
        EnemyDeathAnimation(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            0 as uint16,
            game,
        );
        Zebetites_Func_8(game);
    };
}
pub unsafe fn Zebetites_Func_6(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_Zebetites = Get_Zebetites(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    v1 = ((*E).zebet_var_D as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as int16;
    (*E).zebet_var_D = v1 as uint16;
    let mut r18: uint16 = v1 as uint16;
    Zebetites_Func_7(
        3 as uint16,
        (r18 as ::core::ffi::c_int >> 0 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint8,
        game,
    );
    Zebetites_Func_7(
        4 as uint16,
        (r18 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint8,
        game,
    );
    Zebetites_Func_7(
        5 as uint16,
        (r18 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint8,
        game,
    );
}
pub unsafe fn Zebetites_Func_7(mut j: uint16, mut a: uint8, game: &mut Game) {
    if a != 0 {
        SetEventHappened(j, game);
    } else {
        ClearEventHappened(j, game);
    };
}
pub unsafe fn Zebetites_Func_8(game: &mut Game) {
    SpawnEnemy(
        0xa6 as uint8,
        addr_stru_A6FCE1 as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn Zebetites_Func_9(game: &mut Game) -> uint16 {
    return SpawnEnemy(
        0xa6 as uint8,
        addr_stru_A6FCF9 as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn Zebetites_Func_5(game: &mut Game) {
    let mut v0: uint16 = 0 as uint16;
    let mut E: *mut Enemy_Zebetites = Get_Zebetites(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut health: uint16 = (*E).base.health;
    if health as ::core::ffi::c_int - 800 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    {
        v0 = 2 as uint16;
        if health as ::core::ffi::c_int - 600 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v0 = 4 as uint16;
            if health as ::core::ffi::c_int - 400 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v0 = 6 as uint16;
                if health as ::core::ffi::c_int - 200 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    v0 = 8 as uint16;
                }
            }
        }
    }
    let mut v3: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v4: uint16 = *(game.g_rom.offset(
        (((0xa6fd4a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa6fd4a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v3 as isize);
    if (*E).zebet_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v4 = *(game.g_rom.offset(
            (((0xa6fd54 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa6fd54 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
    }
    (*E).base.current_instruction = v4;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn Zebetites_Func_4(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16) == 0
        && (*Get_Zebetites(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .zebet_parameter_1
            == 0
    {
        let mut E: *mut Enemy_Zebetites = Get_Zebetites(0 as ::core::ffi::c_int, game);
        let mut v1: uint16 = ((*E).zebet_var_C as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int) as uint16;
        (*E).zebet_var_C = v1;
        WriteColorsToPalette(
            0x158 as uint16,
            0xa6 as uint8,
            (4 as ::core::ffi::c_int * v1 as ::core::ffi::c_int - 0x279 as ::core::ffi::c_int)
                as uint16,
            2 as uint16,
            game,
        );
    }
}
pub unsafe fn Zebetites_Touch(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn Zebetites_Shot(game: &mut Game) {
    QueueSfx3_Max6(9 as uint16, game);
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    let mut E: *mut Enemy_Zebetites = Get_Zebetites(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut G: *mut Enemy_Zebetites =
        Get_Zebetites((*E).zebet_parameter_2 as ::core::ffi::c_int, game);
    (*G).base.health = (*E).base.health;
    (*G).base.flash_timer = (*E).base.flash_timer;
}
