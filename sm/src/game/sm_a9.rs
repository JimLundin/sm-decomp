use crate::game::sm_80::{
    CheckEventHappened, NextRandom, QueueMusic_Delayed8, QueueSfx1_Max6, QueueSfx2_Max3,
    QueueSfx2_Max6, QueueSfx3_Max3, QueueSfx3_Max6, SetBossBitForCurArea, SetEventHappened,
};
use crate::game::sm_84::SpawnHardcodedPlm;
use crate::game::sm_86::{
    Math_MultByCos, Math_MultBySin, SpawnEprojWithGfx, SpawnEprojWithRoomGfx,
};
use crate::game::sm_88::{MotherBrainRisingHdmaObject, SpawnMotherBrainRainbowBeamHdma};
use crate::game::sm_89::LoadFxEntry;
use crate::game::sm_8d::SpawnPalfxObject;
use crate::game::sm_90::{CallSomeSamusCode, DisableMinimapAndMarkBossRoomAsExplored};
use crate::game::sm_91::SomeMotherBrainScripts;
use crate::game::sm_94::kSinCosTable8bit_Sext;
use crate::game::sm_a0::{
    CalculateAngleFromXY, CreateDudShot, Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes,
    Enemy_SwitchToFrozenAi, NormalEnemyFrozenAI, NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy,
    NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAiSkipDeathAnim_CurEnemy, SpawnEnemy,
    SuitDamageDivision,
};
use crate::game::sm_a6::{HandleTypewriterText_Ext, Ridley_Func_98, SetupZebesEscapeTypewriter};
use crate::game::sm_ad::{
    EnableEarthquakeAframes, HandleMotherBrainBodyFlickering, MotherBrain_CalcHdma,
    MotherBrain_FadeFromGray_Drained, MotherBrain_FadeFromGray_FakeDeath,
    MotherBrain_FadePalToBlack, MotherBrain_FadeToGray_Drained, MotherBrain_FadeToGray_FakeDeath,
    MotherBrain_FadeToGray_RealDeath, MotherBrain_HealthBasedPaletteHandling,
    MotherBrain_Phase3_TurnLightsBackOn, MotherBrain_SealWall, TurnOffLightsForShitroidDeath,
};
use crate::game_state::Game;
use crate::rtl::{mov24, MemCpy, ReadRegWord, RomPtr, SnesDivide, Unreachable, WriteReg};
extern "C" {
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t,
    CoroutineRet, Func_V, LongPtr, OamEnt, Point16U, Rect16U, SnesRegs, VoidP, VramWriteEntry,
};
pub const MIRR7: SnesRegs = 17279;
pub const UNUSED7: SnesRegs = 17275;
pub const NTRL7: SnesRegs = 17274;
pub const A2A7H: SnesRegs = 17273;
pub const A2A7L: SnesRegs = 17272;
pub const DAS70: SnesRegs = 17271;
pub const DAS7H: SnesRegs = 17270;
pub const DAS7L: SnesRegs = 17269;
pub const A1B7: SnesRegs = 17268;
pub const A1T7H: SnesRegs = 17267;
pub const A1T7L: SnesRegs = 17266;
pub const BBAD7: SnesRegs = 17265;
pub const DMAP7: SnesRegs = 17264;
pub const MIRR6: SnesRegs = 17263;
pub const UNUSED6: SnesRegs = 17259;
pub const NTRL6: SnesRegs = 17258;
pub const A2A6H: SnesRegs = 17257;
pub const A2A6L: SnesRegs = 17256;
pub const DAS60: SnesRegs = 17255;
pub const DAS6H: SnesRegs = 17254;
pub const DAS6L: SnesRegs = 17253;
pub const A1B6: SnesRegs = 17252;
pub const A1T6H: SnesRegs = 17251;
pub const A1T6L: SnesRegs = 17250;
pub const BBAD6: SnesRegs = 17249;
pub const DMAP6: SnesRegs = 17248;
pub const MIRR5: SnesRegs = 17247;
pub const UNUSED5: SnesRegs = 17243;
pub const NTRL5: SnesRegs = 17242;
pub const A2A5H: SnesRegs = 17241;
pub const A2A5L: SnesRegs = 17240;
pub const DAS50: SnesRegs = 17239;
pub const DAS5H: SnesRegs = 17238;
pub const DAS5L: SnesRegs = 17237;
pub const A1B5: SnesRegs = 17236;
pub const A1T5H: SnesRegs = 17235;
pub const A1T5L: SnesRegs = 17234;
pub const BBAD5: SnesRegs = 17233;
pub const DMAP5: SnesRegs = 17232;
pub const MIRR4: SnesRegs = 17231;
pub const UNUSED4: SnesRegs = 17227;
pub const NTRL4: SnesRegs = 17226;
pub const A2A4H: SnesRegs = 17225;
pub const A2A4L: SnesRegs = 17224;
pub const DAS40: SnesRegs = 17223;
pub const DAS4H: SnesRegs = 17222;
pub const DAS4L: SnesRegs = 17221;
pub const A1B4: SnesRegs = 17220;
pub const A1T4H: SnesRegs = 17219;
pub const A1T4L: SnesRegs = 17218;
pub const BBAD4: SnesRegs = 17217;
pub const DMAP4: SnesRegs = 17216;
pub const MIRR3: SnesRegs = 17215;
pub const UNUSED3: SnesRegs = 17211;
pub const NTRL3: SnesRegs = 17210;
pub const A2A3H: SnesRegs = 17209;
pub const A2A3L: SnesRegs = 17208;
pub const DAS30: SnesRegs = 17207;
pub const DAS3H: SnesRegs = 17206;
pub const DAS3L: SnesRegs = 17205;
pub const A1B3: SnesRegs = 17204;
pub const A1T3H: SnesRegs = 17203;
pub const A1T3L: SnesRegs = 17202;
pub const BBAD3: SnesRegs = 17201;
pub const DMAP3: SnesRegs = 17200;
pub const MIRR2: SnesRegs = 17199;
pub const UNUSED2: SnesRegs = 17195;
pub const NTRL2: SnesRegs = 17194;
pub const A2A2H: SnesRegs = 17193;
pub const A2A2L: SnesRegs = 17192;
pub const DAS20: SnesRegs = 17191;
pub const DAS2H: SnesRegs = 17190;
pub const DAS2L: SnesRegs = 17189;
pub const A1B2: SnesRegs = 17188;
pub const A1T2H: SnesRegs = 17187;
pub const A1T2L: SnesRegs = 17186;
pub const BBAD2: SnesRegs = 17185;
pub const DMAP2: SnesRegs = 17184;
pub const MIRR1: SnesRegs = 17183;
pub const UNUSED1: SnesRegs = 17179;
pub const NTRL1: SnesRegs = 17178;
pub const A2A1H: SnesRegs = 17177;
pub const A2A1L: SnesRegs = 17176;
pub const DAS10: SnesRegs = 17175;
pub const DAS1H: SnesRegs = 17174;
pub const DAS1L: SnesRegs = 17173;
pub const A1B1: SnesRegs = 17172;
pub const A1T1H: SnesRegs = 17171;
pub const A1T1L: SnesRegs = 17170;
pub const BBAD1: SnesRegs = 17169;
pub const DMAP1: SnesRegs = 17168;
pub const MIRR0: SnesRegs = 17167;
pub const UNUSED0: SnesRegs = 17163;
pub const NTRL0: SnesRegs = 17162;
pub const A2A0H: SnesRegs = 17161;
pub const A2A0L: SnesRegs = 17160;
pub const DAS00: SnesRegs = 17159;
pub const DAS0H: SnesRegs = 17158;
pub const DAS0L: SnesRegs = 17157;
pub const A1B0: SnesRegs = 17156;
pub const A1T0H: SnesRegs = 17155;
pub const A1T0L: SnesRegs = 17154;
pub const BBAD0: SnesRegs = 17153;
pub const DMAP0: SnesRegs = 17152;
pub const JOY4H: SnesRegs = 16927;
pub const JOY4L: SnesRegs = 16926;
pub const JOY3H: SnesRegs = 16925;
pub const JOY3L: SnesRegs = 16924;
pub const JOY2H: SnesRegs = 16923;
pub const JOY2L: SnesRegs = 16922;
pub const JOY1H: SnesRegs = 16921;
pub const JOY1L: SnesRegs = 16920;
pub const RDMPYH: SnesRegs = 16919;
pub const RDMPYL: SnesRegs = 16918;
pub const RDDIVH: SnesRegs = 16917;
pub const RDDIVL: SnesRegs = 16916;
pub const RDIO: SnesRegs = 16915;
pub const HVBJOY: SnesRegs = 16914;
pub const TIMEUP: SnesRegs = 16913;
pub const RDNMI: SnesRegs = 16912;
pub const MEMSEL: SnesRegs = 16909;
pub const HDMAEN: SnesRegs = 16908;
pub const MDMAEN: SnesRegs = 16907;
pub const VTIMEH: SnesRegs = 16906;
pub const VTIMEL: SnesRegs = 16905;
pub const HTIMEH: SnesRegs = 16904;
pub const HTIMEL: SnesRegs = 16903;
pub const WRDIVB: SnesRegs = 16902;
pub const WRDIVH: SnesRegs = 16901;
pub const WRDIVL: SnesRegs = 16900;
pub const WRMPYB: SnesRegs = 16899;
pub const WRMPYA: SnesRegs = 16898;
pub const WRIO: SnesRegs = 16897;
pub const NMITIMEN: SnesRegs = 16896;
pub const JOYB: SnesRegs = 16407;
pub const JOYA: SnesRegs = 16406;
pub const WMADDH: SnesRegs = 8579;
pub const WMADDM: SnesRegs = 8578;
pub const WMADDL: SnesRegs = 8577;
pub const WMDATA: SnesRegs = 8576;
pub const APUI03: SnesRegs = 8515;
pub const APUI02: SnesRegs = 8514;
pub const APUI01: SnesRegs = 8513;
pub const APUI00: SnesRegs = 8512;
pub const STAT78: SnesRegs = 8511;
pub const STAT77: SnesRegs = 8510;
pub const OPVCT: SnesRegs = 8509;
pub const OPHCT: SnesRegs = 8508;
pub const RDCGRAM: SnesRegs = 8507;
pub const RDVRAMH: SnesRegs = 8506;
pub const RDVRAML: SnesRegs = 8505;
pub const RDOAM: SnesRegs = 8504;
pub const SLHV: SnesRegs = 8503;
pub const MPYH: SnesRegs = 8502;
pub const MPYM: SnesRegs = 8501;
pub const MPYL: SnesRegs = 8500;
pub const SETINI: SnesRegs = 8499;
pub const COLDATA: SnesRegs = 8498;
pub const CGADSUB: SnesRegs = 8497;
pub const CGWSEL: SnesRegs = 8496;
pub const TSW: SnesRegs = 8495;
pub const TMW: SnesRegs = 8494;
pub const TS: SnesRegs = 8493;
pub const TM: SnesRegs = 8492;
pub const WOBJLOG: SnesRegs = 8491;
pub const WBGLOG: SnesRegs = 8490;
pub const WH3: SnesRegs = 8489;
pub const WH2: SnesRegs = 8488;
pub const WH1: SnesRegs = 8487;
pub const WH0: SnesRegs = 8486;
pub const WOBJSEL: SnesRegs = 8485;
pub const W34SEL: SnesRegs = 8484;
pub const W12SEL: SnesRegs = 8483;
pub const CGDATA: SnesRegs = 8482;
pub const CGADD: SnesRegs = 8481;
pub const M7Y: SnesRegs = 8480;
pub const M7X: SnesRegs = 8479;
pub const M7D: SnesRegs = 8478;
pub const M7C: SnesRegs = 8477;
pub const M7B: SnesRegs = 8476;
pub const M7A: SnesRegs = 8475;
pub const M7SEL: SnesRegs = 8474;
pub const VMDATAH: SnesRegs = 8473;
pub const VMDATAL: SnesRegs = 8472;
pub const VMADDH: SnesRegs = 8471;
pub const VMADDL: SnesRegs = 8470;
pub const VMAIN: SnesRegs = 8469;
pub const BG4VOFS: SnesRegs = 8468;
pub const BG4HOFS: SnesRegs = 8467;
pub const BG3VOFS: SnesRegs = 8466;
pub const BG3HOFS: SnesRegs = 8465;
pub const BG2VOFS: SnesRegs = 8464;
pub const BG2HOFS: SnesRegs = 8463;
pub const BG1VOFS: SnesRegs = 8462;
pub const BG1HOFS: SnesRegs = 8461;
pub const BG34NBA: SnesRegs = 8460;
pub const BG12NBA: SnesRegs = 8459;
pub const BG4SC: SnesRegs = 8458;
pub const BG3SC: SnesRegs = 8457;
pub const BG2SC: SnesRegs = 8456;
pub const BG1SC: SnesRegs = 8455;
pub const MOSAIC: SnesRegs = 8454;
pub const BGMODE: SnesRegs = 8453;
pub const OAMDATA: SnesRegs = 8452;
pub const OAMADDH: SnesRegs = 8451;
pub const OAMADDL: SnesRegs = 8450;
pub const OBSEL: SnesRegs = 8449;
pub const INIDISP: SnesRegs = 8448;
pub use crate::types::{EnemyBase, EnemyData, EnemyProps};
pub const kEnemyProps_RespawnIfKilled: EnemyProps = 16384;
pub const kEnemyProps_DisableSamusColl: EnemyProps = 8192;
pub const kEnemyProps_BlockPlasmaBeam: EnemyProps = 4096;
pub const kEnemyProps_ProcessedOffscreen: EnemyProps = 2048;
pub const kEnemyProps_Tangible: EnemyProps = 1024;
pub const kEnemyProps_Deleted: EnemyProps = 512;
pub const kEnemyProps_Invisible: EnemyProps = 256;
pub use crate::types::{Consts_86, ExtraEnemyRam7800, Ram7800_Kraid};
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
pub use crate::types::Consts_8D;
pub const addr_kPalfx_FFD5: Consts_8D = 65493;
pub const addr_kPalfx_FFD1: Consts_8D = 65489;
pub const addr_kPalfx_FFCD: Consts_8D = 65485;
pub const addr_kPalfx_FFC9: Consts_8D = 65481;
pub const addr_stru_8DF75D: Consts_8D = 63325;
pub const addr_stru_8DF759: Consts_8D = 63321;
pub const addr_word_8DEC76: Consts_8D = 60534;
pub const addr_word_8DEB43: Consts_8D = 60227;
pub const addr_off_8DE8B6: Consts_8D = 59574;
pub const addr_off_8DE68A: Consts_8D = 59018;
pub const addr_kPalfxInstrList_E45E: Consts_8D = 58462;
pub const addr_off_8DE420: Consts_8D = 58400;
pub const addr_off_8DE400: Consts_8D = 58368;
pub const addr_off_8DE3E0: Consts_8D = 58336;
pub const addr_kPalfx_E200: Consts_8D = 57856;
pub const addr_stru_8DE1FC: Consts_8D = 57852;
pub const addr_stru_8DE1F8: Consts_8D = 57848;
pub const addr_stru_8DE1F4: Consts_8D = 57844;
pub const addr_stru_8DE1F0: Consts_8D = 57840;
pub const addr_kPalfx_E1E8: Consts_8D = 57832;
pub const addr_kPalfx_E1E4: Consts_8D = 57828;
pub const addr_kPalfx_E1E0: Consts_8D = 57824;
pub const addr_kPalfx_E1DC: Consts_8D = 57820;
pub const addr_kPalfx_E1D8: Consts_8D = 57816;
pub const addr_kPalfx_E1D4: Consts_8D = 57812;
pub const addr_kPalfx_E1D0: Consts_8D = 57808;
pub const addr_kPalfx_E1CC: Consts_8D = 57804;
pub const addr_kPalfx_E1C8: Consts_8D = 57800;
pub const addr_kPalfx_E1C4: Consts_8D = 57796;
pub const addr_stru_8DE1C0: Consts_8D = 57792;
pub const addr_kPalfx_E1BC: Consts_8D = 57788;
pub const addr_kPalfx_E1B8: Consts_8D = 57784;
pub const addr_kPalfx_E1B0: Consts_8D = 57776;
pub const addr_kPalfx_E1AC: Consts_8D = 57772;
pub const addr_kPalfx_E1A8: Consts_8D = 57768;
pub const addr_kPalfx_E1A4: Consts_8D = 57764;
pub const addr_kPalfx_E1A0: Consts_8D = 57760;
pub const addr_kPalfx_E19C: Consts_8D = 57756;
pub const addr_kPalfx_E198: Consts_8D = 57752;
pub const addr_kPalfx_E194: Consts_8D = 57748;
pub const addr_off_8DE192: Consts_8D = 57746;
pub const addr_locret_8DC5E3: Consts_8D = 50659;
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
pub use crate::types::Consts_B7;
pub const addr_kMotherBrain_Misc_TileData: Consts_B7 = 52736;
pub const addr_kDeadMonsters_TileData: Consts_B7 = 49152;
pub const addr_kDeadTorizo_TileData: Consts_B7 = 43008;
pub use crate::types::SpawnHardcodedPlmArgs;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MotherBrainInstrExecState {
    pub ip: uint16,
    pub timer: uint16,
}
pub use crate::types::Enemy_MotherBrain;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Shitroid {
    pub base: EnemyBase,
    pub shitr_var_A: uint16,
    pub shitr_var_B: uint16,
    pub shitr_var_C: uint16,
    pub shitr_var_D: uint16,
    pub shitr_var_E: uint16,
    pub shitr_var_F: uint16,
    pub shitr_parameter_1: uint16,
    pub shitr_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub shitr_var_00: uint16,
    pub shitr_var_01: uint16,
    pub shitr_var_02: uint16,
    pub shitr_var_03: uint16,
    pub shitr_var_04: uint16,
    pub shitr_var_05: uint16,
    pub shitr_var_06: uint16,
    pub shitr_var_07: uint16,
    pub shitr_var_08: uint16,
    pub shitr_var_09: uint16,
    pub shitr_var_0A: uint16,
    pub shitr_var_0B: uint16,
    pub shitr_var_0C: uint16,
    pub shitr_var_0D: uint16,
    pub shitr_var_0E: uint16,
    pub shitr_var_0F: uint16,
    pub shitr_var_10: uint16,
    pub shitr_var_11: uint16,
    pub shitr_var_12: uint16,
    pub shitr_var_13: uint16,
    pub shitr_var_14: uint16,
    pub shitr_var_15: uint16,
    pub shitr_var_16: uint16,
    pub shitr_var_17: uint16,
    pub shitr_var_18: uint16,
    pub shitr_var_19: uint16,
    pub shitr_var_1A: uint16,
    pub shitr_var_1B: uint16,
    pub shitr_var_1C: uint16,
    pub shitr_var_1D: uint16,
    pub shitr_var_1E: uint16,
    pub shitr_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_DeadMonsters {
    pub base: EnemyBase,
    pub dms_var_A: uint16,
    pub dms_var_B: uint16,
    pub dms_var_C: uint16,
    pub dms_var_D: uint16,
    pub dms_var_E: uint16,
    pub dms_var_F: uint16,
    pub dms_parameter_1: uint16,
    pub dms_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub dms_var_00: uint16,
    pub dms_var_01: uint16,
    pub dms_var_02: uint16,
    pub dms_var_03: uint16,
    pub dms_var_04: uint16,
    pub dms_var_05: uint16,
    pub dms_var_06: uint16,
    pub dms_var_07: uint16,
    pub dms_var_08: uint16,
    pub dms_var_09: uint16,
    pub dms_var_0A: uint16,
    pub dms_var_0B: uint16,
    pub dms_var_0C: uint16,
    pub dms_var_0D: uint16,
    pub dms_var_0E: uint16,
    pub dms_var_0F: uint16,
    pub dms_var_10: uint16,
    pub dms_var_11: uint16,
    pub dms_var_12: uint16,
    pub dms_var_13: uint16,
    pub dms_var_14: uint16,
    pub dms_var_15: uint16,
    pub dms_var_16: uint16,
    pub dms_var_17: uint16,
    pub dms_var_18: uint16,
    pub dms_var_19: uint16,
    pub dms_var_1A: uint16,
    pub dms_var_1B: uint16,
    pub dms_var_1C: uint16,
    pub dms_var_1D: uint16,
    pub dms_var_1E: uint16,
    pub dms_var_1F: uint16,
    pub pad3: [uint8; 4032],
    pub dms_var_40: uint16,
    pub dms_var_41: uint16,
    pub dms_var_42: uint16,
    pub dms_var_43: uint16,
    pub dms_var_44: uint16,
    pub dms_var_45: uint16,
    pub dms_var_46: uint16,
    pub dms_var_47: uint16,
    pub dms_var_48: uint16,
    pub dms_var_49: uint16,
    pub dms_var_4A: uint16,
    pub dms_var_4B: uint16,
    pub dms_var_4C: uint16,
    pub dms_var_4D: uint16,
    pub dms_var_4E: uint16,
    pub dms_var_4F: uint16,
    pub dms_var_50: uint16,
    pub dms_var_51: uint16,
    pub dms_var_52: uint16,
    pub dms_var_53: uint16,
    pub dms_var_54: uint16,
    pub dms_var_55: uint16,
    pub dms_var_56: uint16,
    pub dms_var_57: uint16,
    pub dms_var_58: uint16,
    pub dms_var_59: uint16,
    pub dms_var_5A: uint16,
    pub dms_var_5B: uint16,
    pub dms_var_5C: uint16,
    pub dms_var_5D: uint16,
    pub dms_var_5E: uint16,
    pub dms_var_5F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_DeadTorizo {
    pub base: EnemyBase,
    pub dto_var_A: uint16,
    pub dto_var_B: uint16,
    pub dto_var_C: uint16,
    pub dto_var_D: uint16,
    pub dto_var_E: uint16,
    pub dto_var_F: uint16,
    pub dto_parameter_1: uint16,
    pub dto_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub dto_var_00: uint16,
    pub dto_var_01: uint16,
    pub dto_var_02: uint16,
    pub dto_var_03: uint16,
    pub dto_var_04: uint16,
    pub dto_var_05: uint16,
    pub dto_var_06: uint16,
    pub dto_var_07: uint16,
    pub dto_var_08: uint16,
    pub dto_var_09: uint16,
    pub dto_var_0A: uint16,
    pub dto_var_0B: uint16,
    pub dto_var_0C: uint16,
    pub dto_var_0D: uint16,
    pub dto_var_0E: uint16,
    pub dto_var_0F: uint16,
    pub dto_var_10: uint16,
    pub dto_var_11: uint16,
    pub dto_var_12: uint16,
    pub dto_var_13: uint16,
    pub dto_var_14: uint16,
    pub dto_var_15: uint16,
    pub dto_var_16: uint16,
    pub dto_var_17: uint16,
    pub dto_var_18: uint16,
    pub dto_var_19: uint16,
    pub dto_var_1A: uint16,
    pub dto_var_1B: uint16,
    pub dto_var_1C: uint16,
    pub dto_var_1D: uint16,
    pub dto_var_1E: uint16,
    pub dto_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_ShitroidInCutscene {
    pub base: EnemyBase,
    pub sice_var_A: uint16,
    pub sice_var_B: uint16,
    pub sice_var_C: uint16,
    pub sice_var_D: uint16,
    pub sice_var_E: uint16,
    pub sice_var_F: uint16,
    pub sice_parameter_1: uint16,
    pub sice_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub sice_var_00: uint16,
    pub sice_var_01: uint16,
    pub sice_var_02: uint16,
    pub sice_var_03: uint16,
    pub sice_var_04: uint16,
    pub sice_var_05: uint16,
    pub sice_var_06: uint16,
    pub sice_var_07: uint16,
    pub sice_var_08: uint16,
    pub sice_var_09: uint16,
    pub sice_var_0A: uint16,
    pub sice_var_0B: uint16,
    pub sice_var_0C: uint16,
    pub sice_var_0D: uint16,
    pub sice_var_0E: uint16,
    pub sice_var_0F: uint16,
    pub sice_var_10: uint16,
    pub sice_var_11: uint16,
    pub sice_var_12: uint16,
    pub sice_var_13: uint16,
    pub sice_var_14: uint16,
    pub sice_var_15: uint16,
    pub sice_var_16: uint16,
    pub sice_var_17: uint16,
    pub sice_var_18: uint16,
    pub sice_var_19: uint16,
    pub sice_var_1A: uint16,
    pub sice_var_1B: uint16,
    pub sice_var_1C: uint16,
    pub sice_var_1D: uint16,
    pub sice_var_1E: uint16,
    pub sice_var_1F: uint16,
    pub pad2: [uint8; 1984],
    pub sice_var_20: uint16,
    pub sice_var_21: uint16,
    pub sice_var_22: uint16,
    pub sice_var_23: uint16,
    pub sice_var_24: uint16,
    pub sice_var_25: uint16,
    pub sice_var_26: uint16,
    pub sice_var_27: uint16,
    pub sice_var_28: uint16,
    pub sice_var_29: uint16,
    pub sice_var_2A: uint16,
    pub sice_var_2B: uint16,
    pub sice_var_2C: uint16,
    pub sice_var_2D: uint16,
    pub sice_var_2E: uint16,
    pub sice_var_2F: uint16,
    pub sice_var_30: uint16,
    pub sice_var_31: uint16,
    pub sice_var_32: uint16,
    pub sice_var_33: uint16,
    pub sice_var_34: uint16,
    pub sice_var_35: uint16,
    pub sice_var_36: uint16,
    pub sice_var_37: uint16,
    pub sice_var_38: uint16,
    pub sice_var_39: uint16,
    pub sice_var_3A: uint16,
    pub sice_var_3B: uint16,
    pub sice_var_3C: uint16,
    pub sice_var_3D: uint16,
    pub sice_var_3E: uint16,
    pub sice_var_3F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_DeadSidehopper {
    pub base: EnemyBase,
    pub dsr_var_A: uint16,
    pub dsr_var_B: uint16,
    pub dsr_var_C: uint16,
    pub dsr_var_D: uint16,
    pub dsr_var_E: uint16,
    pub dsr_var_F: uint16,
    pub dsr_parameter_1: uint16,
    pub dsr_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub dsr_var_00: uint16,
    pub dsr_var_01: uint16,
    pub dsr_var_02: uint16,
    pub dsr_var_03: uint16,
    pub dsr_var_04: uint16,
    pub dsr_var_05: uint16,
    pub dsr_var_06: uint16,
    pub dsr_var_07: uint16,
    pub dsr_var_08: uint16,
    pub dsr_var_09: uint16,
    pub dsr_var_0A: uint16,
    pub dsr_var_0B: uint16,
    pub dsr_var_0C: uint16,
    pub dsr_var_0D: uint16,
    pub dsr_var_0E: uint16,
    pub dsr_var_0F: uint16,
    pub dsr_var_10: uint16,
    pub dsr_var_11: uint16,
    pub dsr_var_12: uint16,
    pub dsr_var_13: uint16,
    pub dsr_var_14: uint16,
    pub dsr_var_15: uint16,
    pub dsr_var_16: uint16,
    pub dsr_var_17: uint16,
    pub dsr_var_18: uint16,
    pub dsr_var_19: uint16,
    pub dsr_var_1A: uint16,
    pub dsr_var_1B: uint16,
    pub dsr_var_1C: uint16,
    pub dsr_var_1D: uint16,
    pub dsr_var_1E: uint16,
    pub dsr_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_DeadEnemy {
    pub base: EnemyBase,
    pub dey_var_A: uint16,
    pub dey_var_B: uint16,
    pub dey_var_C: uint16,
    pub dey_var_D: uint16,
    pub dey_var_E: uint16,
    pub dey_var_F: uint16,
    pub dey_parameter_1: uint16,
    pub dey_parameter_2: uint16,
    pub pad2: [uint8; 28744],
    pub dey_var_20: uint16,
    pub dey_var_21: uint16,
    pub dey_var_22: uint16,
    pub dey_var_23: uint16,
    pub dey_var_24: uint16,
    pub dey_var_25: uint16,
    pub dey_var_26: uint16,
    pub dey_var_27: uint16,
    pub dey_var_28: uint16,
    pub dey_var_29: uint16,
    pub dey_var_2A: uint16,
    pub dey_var_2B: uint16,
    pub dey_var_2C: uint16,
    pub dey_var_2D: uint16,
    pub dey_var_2E: uint16,
    pub dey_var_2F: uint16,
    pub dey_var_30: uint16,
    pub dey_var_31: uint16,
    pub dey_var_32: uint16,
    pub dey_var_33: uint16,
    pub dey_var_34: uint16,
    pub dey_var_35: uint16,
    pub dey_var_36: uint16,
    pub dey_var_37: uint16,
    pub dey_var_38: uint16,
    pub dey_var_39: uint16,
    pub dey_var_3A: uint16,
    pub dey_var_3B: uint16,
    pub dey_var_3C: uint16,
    pub dey_var_3D: uint16,
    pub dey_var_3E: uint16,
    pub dey_var_3F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MotherBomb {
    pub base: EnemyBase,
    pub mbb_var_A: uint16,
    pub mbb_var_B: uint16,
    pub mbb_var_C: uint16,
    pub mbb_var_D: uint16,
    pub mbb_var_E: uint16,
    pub mbb_var_F: uint16,
    pub mbb_parameter_1: uint16,
    pub mbb_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MotherBrainBody {
    pub base: EnemyBase,
    pub mbby_var_A: uint16,
    pub mbby_var_B: uint16,
    pub mbby_var_C: uint16,
    pub mbby_var_D: uint16,
    pub mbby_var_E: uint16,
    pub mbby_var_F: uint16,
    pub mbby_parameter_1: uint16,
    pub mbby_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MotherBrainsTubesFalling {
    pub base: EnemyBase,
    pub mbtfg_var_A: uint16,
    pub mbtfg_var_B: uint16,
    pub mbtfg_var_C: uint16,
    pub mbtfg_var_D: uint16,
    pub mbtfg_var_E: uint16,
    pub mbtfg_var_F: uint16,
    pub mbtfg_parameter_1: uint16,
    pub mbtfg_parameter_2: uint16,
}
#[inline]
unsafe fn abs16(mut t: uint16) -> uint16 {
    return (if t as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(t as ::core::ffi::c_int)
    } else {
        t as ::core::ffi::c_int
    }) as uint16;
}
pub const kCoroutineNone: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe fn RomPtr_A9(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0xa90000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomPtr_B7(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0xb70000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomPtrWithBank(mut bank: uint8, mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        ((bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int | addr as ::core::ffi::c_int)
            as uint32_t,
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
pub const fnEproj_Init_0x8bde_SkreeDownLeft: ::core::ffi::c_int = 0x868b15 as ::core::ffi::c_int;
pub const fnMotherBrainsBody_UnpauseHook: ::core::ffi::c_int = 0xa98763 as ::core::ffi::c_int;
pub const fnMotherBrainsBrain_SetupBrainAndNeckToDraw: ::core::ffi::c_int =
    0xa987a2 as ::core::ffi::c_int;
pub const fnMotherBrainsBrain_SetupBrainToDraw: ::core::ffi::c_int = 0xa987d0 as ::core::ffi::c_int;
pub const fnMotherBrainsBody_FirstPhase_DoubleRet: ::core::ffi::c_int =
    0xa987e1 as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Descent_0_Pause: uint32 = 11110429 as uint32;
pub const fnMotherBrainBody_FakeDeath_Descent_1: ::core::ffi::c_int =
    0xa98829 as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Descent_2: ::core::ffi::c_int =
    0xa9884d as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Descent_3: ::core::ffi::c_int =
    0xa9886c as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Descent_4: ::core::ffi::c_int =
    0xa98884 as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Descent_5: ::core::ffi::c_int =
    0xa988b2 as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Descent_6: ::core::ffi::c_int =
    0xa988d3 as ::core::ffi::c_int;
pub const fnMotherBrainBody_0_Wait: ::core::ffi::c_int = 0xa98949 as ::core::ffi::c_int;
pub const fnMotherBrainBody_1_ClearBottomLeftTube: ::core::ffi::c_int =
    0xa9896e as ::core::ffi::c_int;
pub const fnMotherBrainBody_2_SpawnTopRightTubeFalling: ::core::ffi::c_int =
    0xa98983 as ::core::ffi::c_int;
pub const fnMotherBrainBody_3_ClearCeilingBlock9: uint32 = 11110816 as uint32;
pub const fnMotherBrainBody_4_SpawnTopLeftTubeFalling: ::core::ffi::c_int =
    0xa989b5 as ::core::ffi::c_int;
pub const fnMotherBrainBody_4_ClearCeilingBlock6: ::core::ffi::c_int =
    0xa989d2 as ::core::ffi::c_int;
pub const fnMotherBrainBody_5_SpawnTubeFallingEnemy1: ::core::ffi::c_int =
    0xa989e7 as ::core::ffi::c_int;
pub const fnMotherBrainBody_6_ClearBottomRightTube: ::core::ffi::c_int =
    0xa989fa as ::core::ffi::c_int;
pub const fnMotherBrainBody_7_SpawnTubeFallingEnemy2: ::core::ffi::c_int =
    0xa98a0f as ::core::ffi::c_int;
pub const fnMotherBrainBody_8_ClearBottomMiddleLeftTube: ::core::ffi::c_int =
    0xa98a22 as ::core::ffi::c_int;
pub const fnMotherBrainBody_9_SpawnTopMiddleLeftFalling: ::core::ffi::c_int =
    0xa98a37 as ::core::ffi::c_int;
pub const fnMotherBrainBody_10_ClearCeilingTubeColumn7: ::core::ffi::c_int =
    0xa98a54 as ::core::ffi::c_int;
pub const fnMotherBrainBody_11_SpawnTopMiddleRightFalling: ::core::ffi::c_int =
    0xa98a69 as ::core::ffi::c_int;
pub const fnMotherBrainBody_12_ClearCeilingTubeColumn8: ::core::ffi::c_int =
    0xa98a86 as ::core::ffi::c_int;
pub const fnMotherBrainBody_13_SpawnTubeFallingEnemy3: ::core::ffi::c_int =
    0xa98a9b as ::core::ffi::c_int;
pub const fnMotherBrainBody_14_ClearBottomMiddleRightTube: ::core::ffi::c_int =
    0xa98aae as ::core::ffi::c_int;
pub const fnMotherBrainBody_15_SpawnTubeFallingEnemy4: ::core::ffi::c_int =
    0xa98ac3 as ::core::ffi::c_int;
pub const fnMotherBrainBody_16_ClearBottomMiddleTubes: ::core::ffi::c_int =
    0xa98ad6 as ::core::ffi::c_int;
pub const fnnullsub_365: uint32 = 11111140 as uint32;
pub const fnMotherBrainsTubesFalling_Main_NonMain: uint32 = 11111304 as uint32;
pub const fnMotherBrainsTubesFalling_WaitToFall: uint32 = 11111371 as uint32;
pub const fnMotherBrainsTubesFalling_Falling: ::core::ffi::c_int = 0xa98bd6 as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Ascent_0_DrawBG1Row23: ::core::ffi::c_int =
    0xa98c87 as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Ascent_1_DrawBG1Row45: ::core::ffi::c_int =
    0xa98c9e as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Ascent_2_DrawBG1Row67: ::core::ffi::c_int =
    0xa98cb5 as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Ascent_3_DrawBG1Row89: ::core::ffi::c_int =
    0xa98ccc as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Ascent_4_DrawBG1RowAB: ::core::ffi::c_int =
    0xa98ce3 as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Ascent_5_DrawBG1RowCD: ::core::ffi::c_int =
    0xa98cfa as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Ascent_6_SetupPhase2Gfx: ::core::ffi::c_int =
    0xa98d11 as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Ascent_7_SetupPhase2Brain: ::core::ffi::c_int =
    0xa98d49 as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Ascent_8_Pause: ::core::ffi::c_int =
    0xa98d79 as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Ascent_9_PrepareRise: ::core::ffi::c_int =
    0xa98d8b as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Ascent_10_LoadLegTiles: ::core::ffi::c_int =
    0xa98db4 as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Ascent_11_ContinuePause: ::core::ffi::c_int =
    0xa98dc3 as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Ascent_12_StartMusic: ::core::ffi::c_int =
    0xa98dec as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Ascent_13_Raise: ::core::ffi::c_int =
    0xa98e4d as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Ascent_14_WaitForUncouching: ::core::ffi::c_int =
    0xa98e95 as ::core::ffi::c_int;
pub const fnMotherBrainBody_FakeDeath_Ascent_15_TransitionFromGrey: ::core::ffi::c_int =
    0xa98eaa as ::core::ffi::c_int;
pub const fnMotherBrainBody_2ndphase_16_ShakeHeadMenacingly: ::core::ffi::c_int =
    0xa98ef5 as ::core::ffi::c_int;
pub const fnMotherBrainBody_2ndphase_17_BringHeadBackUp: ::core::ffi::c_int =
    0xa98f14 as ::core::ffi::c_int;
pub const fnMotherBrainBody_2ndphase_18_FinishStretching: ::core::ffi::c_int =
    0xa98f33 as ::core::ffi::c_int;
pub const fnMotherBrain_Instr_Goto: uint32 = 11115279 as uint32;
pub const fnMotherBrain_Instr_EnableNeckMovementGoto: uint32 = 11115284 as uint32;
pub const fnMotherBrain_Instr_DisableNeckMovement: uint32 = 11115296 as uint32;
pub const fnMotherBrain_Instr_QueueSfx2: uint32 = 11115304 as uint32;
pub const fnMotherBrain_Instr_QueueSfx3: uint32 = 11115314 as uint32;
pub const fnMotherBrain_Instr_SpawnDroolEproj: uint32 = 11115324 as uint32;
pub const fnMotherBrain_Instr_SpawnPurpleBreath: uint32 = 11115373 as uint32;
pub const fnMotherBrain_Instr_SetMainShakeTimer50: uint32 = 11115383 as uint32;
pub const fnMotherBrain_Instr_GotoEitherOr: uint32 = 11115621 as uint32;
pub const fnMotherBrain_Instr_MaybeGoto: uint32 = 11115693 as uint32;
pub const fnMotherBrain_Instr_MaybeGoto2: uint32 = 11115789 as uint32;
pub const fnMotherBrain_Instr_Goto2: uint32 = 11115809 as uint32;
pub const fnMotherBrain_Instr_QueueShitroidAttackSfx: uint32 = 11116023 as uint32;
pub const fnMotherBrain_Instr_SpawnBlueRingEproj: uint32 = 11116073 as uint32;
pub const fnMotherBrain_Instr_AimRingsAtShitroid: uint32 = 11116087 as uint32;
pub const fnMotherBrain_Instr_AimRingsAtSamus: uint32 = 11116123 as uint32;
pub const fnMotherBrain_Instr_IncrShitroidAttackCtr: uint32 = 11116195 as uint32;
pub const fnMotherBrain_Instr_SetShitroidAttackCtr0: uint32 = 11116213 as uint32;
pub const fnMotherBrain_Instr_SpawnBombEproj: uint32 = 11116221 as uint32;
pub const fnMotherBrain_Instr_SpawnLaserEproj: uint32 = 11116358 as uint32;
pub const fnMotherBrain_Instr_SpawnRainbowEproj: uint32 = 11116420 as uint32;
pub const fnMotherBrain_Instr_SetupFxForRainbowBeam: uint32 = 11116430 as uint32;
pub const fnMotherBrain_Phase3_Death_0: ::core::ffi::c_int = 0xa9aee1 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_1: ::core::ffi::c_int = 0xa9af12 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_2: ::core::ffi::c_int = 0xa9af21 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_3: ::core::ffi::c_int = 0xa9af54 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_4: ::core::ffi::c_int = 0xa9af9d as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_5: ::core::ffi::c_int = 0xa9afb6 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_6: ::core::ffi::c_int = 0xa9b013 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_7: ::core::ffi::c_int = 0xa9b115 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_8: ::core::ffi::c_int = 0xa9b12d as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_9: ::core::ffi::c_int = 0xa9b15e as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_10: ::core::ffi::c_int = 0xa9b173 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_11: ::core::ffi::c_int = 0xa9b189 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_12: ::core::ffi::c_int = 0xa9b1b8 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_13: ::core::ffi::c_int = 0xa9b1d5 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_14_20framedelay: ::core::ffi::c_int =
    0xa9b211 as ::core::ffi::c_int;
pub const fnMotherBrain_CorpseRottingFinished: uint32 = 11121187 as uint32;
pub const fnMotherBrain_Phase3_Death_15_LoadEscapeTimerTiles: ::core::ffi::c_int =
    0xa9b258 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_16_StartEscape: ::core::ffi::c_int =
    0xa9b26d as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_17_SpawnTimeBomb: ::core::ffi::c_int =
    0xa9b2d1 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_18_TypesZebesText: ::core::ffi::c_int =
    0xa9b2e3 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_19_EscapeDoorExploding: ::core::ffi::c_int =
    0xa9b2f9 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_20_BlowUpEscapeDoor: ::core::ffi::c_int =
    0xa9b32a as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Death_21_KeepEarthquakeGoing: ::core::ffi::c_int =
    0xa9b33c as ::core::ffi::c_int;
pub const fnMotherBrain_Body_Phase2_Thinking: ::core::ffi::c_int = 0xa9b605 as ::core::ffi::c_int;
pub const fnMotherBrain_Body_Phase2_TryAttack: ::core::ffi::c_int = 0xa9b64b as ::core::ffi::c_int;
pub const fnMotherBrain_FiringBomb_DecideOnWalking: ::core::ffi::c_int =
    0xa9b781 as ::core::ffi::c_int;
pub const fnMotherBrain_FiringBomb_WalkingBackwards: ::core::ffi::c_int =
    0xa9b7ac as ::core::ffi::c_int;
pub const fnMotherBrain_FiringBomb_Crouch: ::core::ffi::c_int = 0xa9b7c6 as ::core::ffi::c_int;
pub const fnMotherBrain_FiringBomb_Fired: ::core::ffi::c_int = 0xa9b7e8 as ::core::ffi::c_int;
pub const fnMotherBrain_FiringBomb_Standup: ::core::ffi::c_int = 0xa9b7f8 as ::core::ffi::c_int;
pub const fnMotherBomb_FiringLaser_PositionHead: ::core::ffi::c_int =
    0xa9b80e as ::core::ffi::c_int;
pub const fnMotherBomb_FiringLaser_PositionHeadSlowlyFire: ::core::ffi::c_int =
    0xa9b839 as ::core::ffi::c_int;
pub const fnMotherBomb_FiringLaser_FinishAttack: ::core::ffi::c_int =
    0xa9b863 as ::core::ffi::c_int;
pub const fnMotherBomb_FiringDeathBeam: ::core::ffi::c_int = 0xa9b87d as ::core::ffi::c_int;
pub const fnMotherBomb_FiringRainbowBeam_0: ::core::ffi::c_int = 0xa9b8eb as ::core::ffi::c_int;
pub const fnMotherBomb_FiringRainbowBeam_1_StartCharge: ::core::ffi::c_int =
    0xa9b91a as ::core::ffi::c_int;
pub const fnMotherBomb_FiringRainbowBeam_2_RetractNeck: ::core::ffi::c_int =
    0xa9b92b as ::core::ffi::c_int;
pub const fnMotherBomb_FiringRainbowBeam_3_Wait: ::core::ffi::c_int =
    0xa9b93f as ::core::ffi::c_int;
pub const fnMotherBomb_FiringRainbowBeam_4_ExtendNeckDown: ::core::ffi::c_int =
    0xa9b951 as ::core::ffi::c_int;
pub const fnMotherBomb_FiringRainbowBeam_5_StartFiring: ::core::ffi::c_int =
    0xa9b975 as ::core::ffi::c_int;
pub const fnMotherBomb_FiringRainbowBeam_6_MoveSamusToWall: ::core::ffi::c_int =
    0xa9b9e5 as ::core::ffi::c_int;
pub const fnMotherBomb_FiringRainbowBeam_7_DelayFrame: ::core::ffi::c_int =
    0xa9ba00 as ::core::ffi::c_int;
pub const fnMotherBomb_FiringRainbowBeam_8_StartDrainSamus: ::core::ffi::c_int =
    0xa9ba27 as ::core::ffi::c_int;
pub const fnMotherBomb_FiringRainbowBeam_9_DrainingSamus: ::core::ffi::c_int =
    0xa9ba3c as ::core::ffi::c_int;
pub const fnMotherBomb_FiringRainbowBeam_10_FinishFiringRainbow: ::core::ffi::c_int =
    0xa9ba5e as ::core::ffi::c_int;
pub const fnMotherBomb_FiringRainbowBeam_11_LetSamusFall: ::core::ffi::c_int =
    0xa9bac4 as ::core::ffi::c_int;
pub const fnMotherBomb_FiringRainbowBeam_12_WaitForSamusHitGround: ::core::ffi::c_int =
    0xa9bad1 as ::core::ffi::c_int;
pub const fnMotherBomb_FiringRainbowBeam_13_LowerHead: ::core::ffi::c_int =
    0xa9badd as ::core::ffi::c_int;
pub const fnMotherBomb_FiringRainbowBeam_14_DecideNextAction: ::core::ffi::c_int =
    0xa9bb06 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2Cut_0: ::core::ffi::c_int = 0xa9bd45 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2Cut_1: ::core::ffi::c_int = 0xa9bd98 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2Cut_2: ::core::ffi::c_int = 0xa9bda9 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2Cut_3: ::core::ffi::c_int = 0xa9bdc1 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2Cut_4: ::core::ffi::c_int = 0xa9bdd2 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2Cut_5: ::core::ffi::c_int = 0xa9bded as ::core::ffi::c_int;
pub const fnnullsub_364: ::core::ffi::c_int = 0xa9be1a as ::core::ffi::c_int;
pub const fnMotherBrain_DrainedByShitroid_0: ::core::ffi::c_int = 0xa9be38 as ::core::ffi::c_int;
pub const fnMotherBrain_DrainedByShitroid_1: ::core::ffi::c_int = 0xa9be5d as ::core::ffi::c_int;
pub const fnMotherBrain_DrainedByShitroid_2: ::core::ffi::c_int = 0xa9be96 as ::core::ffi::c_int;
pub const fnMotherBrain_DrainedByShitroid_3: ::core::ffi::c_int = 0xa9bf0e as ::core::ffi::c_int;
pub const fnMotherBrain_DrainedByShitroid_4: ::core::ffi::c_int = 0xa9bf41 as ::core::ffi::c_int;
pub const fnMotherBrain_DrainedByShitroid_5: ::core::ffi::c_int = 0xa9bf56 as ::core::ffi::c_int;
pub const fnMotherBrain_DrainedByShitroid_6: ::core::ffi::c_int = 0xa9bf7d as ::core::ffi::c_int;
pub const fnMotherBrain_DrainedByShitroid_7: ::core::ffi::c_int = 0xa9bf95 as ::core::ffi::c_int;
pub const fnMotherBrain_PainfulWalkForwards: ::core::ffi::c_int = 0xa9bfd0 as ::core::ffi::c_int;
pub const fnMotherBrain_PainfulWalkingForwards: ::core::ffi::c_int = 0xa9bfe8 as ::core::ffi::c_int;
pub const fnMotherBrain_PainfulWalkBackwards: ::core::ffi::c_int = 0xa9c004 as ::core::ffi::c_int;
pub const fnMotherBrain_PainfulWalkingBackwards: ::core::ffi::c_int =
    0xa9c01c as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2_Revive_0: ::core::ffi::c_int = 0xa9c059 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2_Revive_1: ::core::ffi::c_int = 0xa9c066 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2_Revive_2: ::core::ffi::c_int = 0xa9c082 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2_Revive_3: ::core::ffi::c_int = 0xa9c08f as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2_Revive_4: ::core::ffi::c_int = 0xa9c0ba as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2_Revive_5: ::core::ffi::c_int = 0xa9c0e4 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2_Revive_6: ::core::ffi::c_int = 0xa9c0fb as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2_Revive_7: ::core::ffi::c_int = 0xa9c11e as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2_Revive_8: ::core::ffi::c_int = 0xa9c147 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2_MurderShitroid_1: ::core::ffi::c_int =
    0xa9c15c as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2_MurderShitroid_2: ::core::ffi::c_int =
    0xa9c182 as ::core::ffi::c_int;
pub const fnnullsub_367: uint32 = 11125133 as uint32;
pub const fnMotherBrain_Phase2_PrepareForFinalShitroid: ::core::ffi::c_int =
    0xa9c18e as ::core::ffi::c_int;
pub const fnMotherBrain_Phase2_ExecuteFinalkShitroid: ::core::ffi::c_int =
    0xa9c19a as ::core::ffi::c_int;
pub const fnnullsub_363: ::core::ffi::c_int = 0xa9c1a6 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Recover_MakeDistance: ::core::ffi::c_int =
    0xa9c1cf as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Recover_SetupForFight: ::core::ffi::c_int =
    0xa9c1f0 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Fighting_Main: ::core::ffi::c_int = 0xa9c209 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Fighting_Cooldown: ::core::ffi::c_int =
    0xa9c24e as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Walk_TryToInchForward: ::core::ffi::c_int =
    0xa9c26a as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Walk_RetreatQuickly: ::core::ffi::c_int =
    0xa9c2b3 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Walk_RetreatSlowly: ::core::ffi::c_int =
    0xa9c2d2 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Neck_Normal: ::core::ffi::c_int = 0xa9c330 as ::core::ffi::c_int;
pub const fnnullsub_369: uint32 = 11125587 as uint32;
pub const fnMotherBrain_Phase3_Neck_SetupRecoilRecovery: ::core::ffi::c_int =
    0xa9c354 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Neck_RecoilRecovery: ::core::ffi::c_int =
    0xa9c37b as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Neck_SetupHyperBeamRecoil: ::core::ffi::c_int =
    0xa9c395 as ::core::ffi::c_int;
pub const fnMotherBrain_Phase3_Neck_HyperBeamRecoil: ::core::ffi::c_int =
    0xa9c3cd as ::core::ffi::c_int;
pub const fnShitroidInCutscene_DashOntoScreen: ::core::ffi::c_int = 0xa9c7cc as ::core::ffi::c_int;
pub const fnShitroidInCutscene_CurveTowardsBrain: ::core::ffi::c_int =
    0xa9c7ec as ::core::ffi::c_int;
pub const fnShitroidInCutscene_GetIntoFace: ::core::ffi::c_int = 0xa9c811 as ::core::ffi::c_int;
pub const fnShitroidInCutscene_LatchOntoBrain: ::core::ffi::c_int = 0xa9c851 as ::core::ffi::c_int;
pub const fnShitroidInCutscene_SetMotherBrainToStumbleBack: ::core::ffi::c_int =
    0xa9c879 as ::core::ffi::c_int;
pub const fnShitroidInCutscene_ActivateRainbowBeam: ::core::ffi::c_int =
    0xa9c889 as ::core::ffi::c_int;
pub const fnShitroidInCutscene_BrainTurnsToCorpse: ::core::ffi::c_int =
    0xa9c8e2 as ::core::ffi::c_int;
pub const fnShitroidInCutscene_StopDraining: ::core::ffi::c_int = 0xa9c915 as ::core::ffi::c_int;
pub const fnShitroidInCutscene_LetGoAndSpawnDust: ::core::ffi::c_int =
    0xa9c94b as ::core::ffi::c_int;
pub const fnShitroidInCutscene_MoveUpToCeiling: uint32 = 11127129 as uint32;
pub const fnShitroidInCutscene_MoveToSamus: ::core::ffi::c_int = 0xa9c9c3 as ::core::ffi::c_int;
pub const fnShitroidInCutscene_LatchOntoSamus: ::core::ffi::c_int = 0xa9ca66 as ::core::ffi::c_int;
pub const fnShitroidInCutscene_HealSamusToFullHealth: ::core::ffi::c_int =
    0xa9ca7a as ::core::ffi::c_int;
pub const fnnullsub_368: ::core::ffi::c_int = 0xa9cabc as ::core::ffi::c_int;
pub const fnShitroidInCutscene_IdleUntilToNoHealth: ::core::ffi::c_int =
    0xa9cabd as ::core::ffi::c_int;
pub const fnShitroidInCutscene_ReleaseSamus: ::core::ffi::c_int = 0xa9cb13 as ::core::ffi::c_int;
pub const fnShitroidInCutscene_StareDownMotherBrain: ::core::ffi::c_int =
    0xa9cb2d as ::core::ffi::c_int;
pub const fnShitroidInCutscene_FlyOffScreen: ::core::ffi::c_int = 0xa9cb56 as ::core::ffi::c_int;
pub const fnShitroidInCutscene_MoveToFinalChargeStart: ::core::ffi::c_int =
    0xa9cb7b as ::core::ffi::c_int;
pub const fnShitroidInCutscene_InitiateFinalCharge: ::core::ffi::c_int =
    0xa9cbb3 as ::core::ffi::c_int;
pub const fnShitroidInCutscene_FinalCharge: ::core::ffi::c_int = 0xa9cbd8 as ::core::ffi::c_int;
pub const fnShitroidInCutscene_ShitroidFinalBelow: ::core::ffi::c_int =
    0xa9cc3e as ::core::ffi::c_int;
pub const fnShitroidInCutscene_PlaySamusTheme: ::core::ffi::c_int = 0xa9cc60 as ::core::ffi::c_int;
pub const fnShitroidInCutscene_PrepareSamusHyperbeam: ::core::ffi::c_int =
    0xa9cc7f as ::core::ffi::c_int;
pub const fnnullsub_338: ::core::ffi::c_int = 0xa9cc98 as ::core::ffi::c_int;
pub const fnShitroidInCutscene_DeathSequence: ::core::ffi::c_int = 0xa9cc99 as ::core::ffi::c_int;
pub const fnShitroidInCutscene_UnloadShitroid: ::core::ffi::c_int = 0xa9ccc0 as ::core::ffi::c_int;
pub const fnShitroidInCutscene_LetSamusRainbowMore: ::core::ffi::c_int =
    0xa9ccde as ::core::ffi::c_int;
pub const fnShitroidInCutscene_FinishCutscene: ::core::ffi::c_int = 0xa9ccf0 as ::core::ffi::c_int;
pub const fnSamusRainbowPaletteFunc_ActivateWhenEnemyLow: ::core::ffi::c_int =
    0xa9cd30 as ::core::ffi::c_int;
pub const fnSamusRainbowPaletteFunc_GraduallySlowDown: ::core::ffi::c_int =
    0xa9cd4b as ::core::ffi::c_int;
pub const fnDeadTorizo_WaitForSamusColl: ::core::ffi::c_int = 0xa9d3ad as ::core::ffi::c_int;
pub const fnnullsub_361: ::core::ffi::c_int = 0xa9d3c7 as ::core::ffi::c_int;
pub const fnDeadTorizo_PreRotDelay: ::core::ffi::c_int = 0xa9d3c8 as ::core::ffi::c_int;
pub const fnDeadTorizo_Rotting: ::core::ffi::c_int = 0xa9d3e6 as ::core::ffi::c_int;
pub const fnDeadTorizo_CorpseRottingFinished: uint32 = 11130301 as uint32;
pub const fnDeadSidehopper_Alive_WaitForActivate: ::core::ffi::c_int =
    0xa9d8e2 as ::core::ffi::c_int;
pub const fnDeadSidehopper_Activated: ::core::ffi::c_int = 0xa9d8f1 as ::core::ffi::c_int;
pub const fnnullsub_362: ::core::ffi::c_int = 0xa9d90f as ::core::ffi::c_int;
pub const fnDeadMonsters_Func_1: ::core::ffi::c_int = 0xa9d910 as ::core::ffi::c_int;
pub const fnDeadMonsters_Func_2: ::core::ffi::c_int = 0xa9d91d as ::core::ffi::c_int;
pub const fnDeadMonsters_Func_5: ::core::ffi::c_int = 0xa9da08 as ::core::ffi::c_int;
pub const fnnullsub_268: ::core::ffi::c_int = 0xa9da63 as ::core::ffi::c_int;
pub const fnDeadSidehopper_WaitForSamusColl: ::core::ffi::c_int = 0xa9da64 as ::core::ffi::c_int;
pub const fnDeadZoomer_WaitForSamusColl: ::core::ffi::c_int = 0xa9da69 as ::core::ffi::c_int;
pub const fnDeadSkree_WaitForSamusColl: ::core::ffi::c_int = 0xa9da6e as ::core::ffi::c_int;
pub const fnDeadRipper_WaitForSamusColl: ::core::ffi::c_int = 0xa9da73 as ::core::ffi::c_int;
pub const fnDeadSidehopper_PreRotDelay: ::core::ffi::c_int = 0xa9da8f as ::core::ffi::c_int;
pub const fnDeadZoomer_PreRotDelay: ::core::ffi::c_int = 0xa9da94 as ::core::ffi::c_int;
pub const fnDeadRipper_PreRotDelay: ::core::ffi::c_int = 0xa9da99 as ::core::ffi::c_int;
pub const fnDeadSkree_PreRotDelay: ::core::ffi::c_int = 0xa9da9e as ::core::ffi::c_int;
pub const fnDeadSidehopper_Rotting: ::core::ffi::c_int = 0xa9daba as ::core::ffi::c_int;
pub const fnDeadZoomer_Rotting: ::core::ffi::c_int = 0xa9dad0 as ::core::ffi::c_int;
pub const fnDeadRipper_Rotting: ::core::ffi::c_int = 0xa9dae6 as ::core::ffi::c_int;
pub const fnDeadSkree_Rotting: ::core::ffi::c_int = 0xa9dafc as ::core::ffi::c_int;
pub const fnCorpseRottingRotEntryFinishedHook: uint32 = 11131912 as uint32;
pub const fnTorizo_CorpseRottingInitFunc: uint32 = 11132440 as uint32;
pub const fnSidehopper_CorpseRottingInitFunc_0: uint32 = 11132609 as uint32;
pub const fnSidehopper_CorpseRottingInitFunc_2: uint32 = 11132680 as uint32;
pub const fnZoomer_CorpseRottingInitFunc_0: uint32 = 11132751 as uint32;
pub const fnZoomer_CorpseRottingInitFunc_2: uint32 = 11132780 as uint32;
pub const fnZoomer_CorpseRottingInitFunc_4: uint32 = 11132809 as uint32;
pub const fnRipper_CorpseRottingInitFunc_0: uint32 = 11132838 as uint32;
pub const fnRipper_CorpseRottingInitFunc_2: uint32 = 11132867 as uint32;
pub const fnSkree_CorpseRottingInitFunc_0: uint32 = 11132896 as uint32;
pub const fnSkree_CorpseRottingInitFunc_2: uint32 = 11132953 as uint32;
pub const fnSkree_CorpseRottingInitFunc_4: uint32 = 11133010 as uint32;
pub const fnMotherBrain_CorpseRottingInitFunc: uint32 = 11133067 as uint32;
pub const fnTorizo_CorpseRottingMoveFunc: uint32 = 11133554 as uint32;
pub const fnTorizo_CorpseRottingCopyFunc: uint32 = 11133835 as uint32;
pub const fnSidehopper_CorpseRottingMoveFunc_0: uint32 = 11134056 as uint32;
pub const fnSidehopper_CorpseRottingCopyFunc_0: uint32 = 11134197 as uint32;
pub const fnSidehopper_CorpseRottingMoveFunc_2: uint32 = 11134308 as uint32;
pub const fnSidehopper_CorpseRottingCopyFunc_2: uint32 = 11134454 as uint32;
pub const fnZoomer_CorpseRottingMoveFunc_0: uint32 = 11134570 as uint32;
pub const fnZoomer_CorpseRottingCopyFunc_0: uint32 = 11134649 as uint32;
pub const fnZoomer_CorpseRottingMoveFunc_2: uint32 = 11134710 as uint32;
pub const fnZoomer_CorpseRottingCopyFunc_2: uint32 = 11134789 as uint32;
pub const fnZoomer_CorpseRottingMoveFunc_4: uint32 = 11134850 as uint32;
pub const fnZoomer_CorpseRottingCopyFunc_4: uint32 = 11134929 as uint32;
pub const fnRipper_CorpseRottingMoveFunc_0: uint32 = 11134990 as uint32;
pub const fnRipper_CorpseRottingCopyFunc_0: uint32 = 11135069 as uint32;
pub const fnRipper_CorpseRottingMoveFunc_2: uint32 = 11135130 as uint32;
pub const fnRipper_CorpseRottingCopyFunc_2: uint32 = 11135209 as uint32;
pub const fnSkree_CorpseRottingMoveFunc_0: uint32 = 11135270 as uint32;
pub const fnSkree_CorpseRottingCopyFunc_0: uint32 = 11135323 as uint32;
pub const fnSkree_CorpseRottingMoveFunc_2: uint32 = 11135364 as uint32;
pub const fnSkree_CorpseRottingCopyFunc_2: uint32 = 11135417 as uint32;
pub const fnSkree_CorpseRottingMoveFunc_4: uint32 = 11135458 as uint32;
pub const fnSkree_CorpseRottingCopyFunc_4: uint32 = 11135511 as uint32;
pub const fnMotherBrain_CorpseRottingMoveFunc: uint32 = 11135552 as uint32;
pub const fnMotherBrain_CorpseRottingCopyFunc: uint32 = 11135755 as uint32;
pub const fnShitroid_Func_3: ::core::ffi::c_int = 0xa9efdf as ::core::ffi::c_int;
pub const fnShitroid_Func_4: ::core::ffi::c_int = 0xa9efe6 as ::core::ffi::c_int;
pub const fnShitroid_Func_5: ::core::ffi::c_int = 0xa9f02b as ::core::ffi::c_int;
pub const fnShitroid_Func_6: ::core::ffi::c_int = 0xa9f037 as ::core::ffi::c_int;
pub const fnShitroid_Func_7: ::core::ffi::c_int = 0xa9f049 as ::core::ffi::c_int;
pub const fnShitroid_Func_8: ::core::ffi::c_int = 0xa9f06d as ::core::ffi::c_int;
pub const fnShitroid_Func_9: ::core::ffi::c_int = 0xa9f094 as ::core::ffi::c_int;
pub const fnShitroid_Func_10: ::core::ffi::c_int = 0xa9f0e6 as ::core::ffi::c_int;
pub const fnShitroid_Func_11: ::core::ffi::c_int = 0xa9f125 as ::core::ffi::c_int;
pub const fnShitroid_Func_12: ::core::ffi::c_int = 0xa9f138 as ::core::ffi::c_int;
pub const fnShitroid_Func_13: ::core::ffi::c_int = 0xa9f180 as ::core::ffi::c_int;
pub const fnShitroid_Func_14: ::core::ffi::c_int = 0xa9f1fa as ::core::ffi::c_int;
pub const fnShitroid_Func_15: ::core::ffi::c_int = 0xa9f20e as ::core::ffi::c_int;
pub const fnShitroid_Func_16: ::core::ffi::c_int = 0xa9f21b as ::core::ffi::c_int;
pub const fnShitroid_Func_17: ::core::ffi::c_int = 0xa9f2a2 as ::core::ffi::c_int;
pub const fnShitroid_Func_18: ::core::ffi::c_int = 0xa9f2ae as ::core::ffi::c_int;
pub const fnShitroid_Func_19: ::core::ffi::c_int = 0xa9f2c0 as ::core::ffi::c_int;
pub const fnShitroid_Func_20: ::core::ffi::c_int = 0xa9f2fb as ::core::ffi::c_int;
pub const fnShitroid_Func_21: ::core::ffi::c_int = 0xa9f324 as ::core::ffi::c_int;
pub const fnShitroid_Func_22: ::core::ffi::c_int = 0xa9f360 as ::core::ffi::c_int;
pub const fnShitroid_Func_23: ::core::ffi::c_int = 0xa9f36d as ::core::ffi::c_int;
pub const fnShitroid_Func_24: ::core::ffi::c_int = 0xa9f3a3 as ::core::ffi::c_int;
pub const fnShitroid_Func_25: ::core::ffi::c_int = 0xa9f3be as ::core::ffi::c_int;
pub const fnShitroid_GraduallyAccelerateTowards0x8: uint32 = 11138143 as uint32;
pub const fnShitroid_GraduallyAccelerateTowards0x10: uint32 = 11138150 as uint32;
pub const fnShitroid_HandleCutscenePalette: ::core::ffi::c_int = 0xa9f683 as ::core::ffi::c_int;
pub const fnShitroid_HandleCutscenePalette_LowHealth: ::core::ffi::c_int =
    0xa9f68f as ::core::ffi::c_int;
#[inline]
unsafe fn gEnemyData(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemyData {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut EnemyData;
}
#[inline]
unsafe fn Get_DeadEnemy(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_DeadEnemy {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_DeadEnemy;
}
#[inline]
unsafe fn Get_DeadMonsters(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_DeadMonsters {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_DeadMonsters;
}
#[inline]
unsafe fn Get_DeadSidehopper(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_DeadSidehopper {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_DeadSidehopper;
}
#[inline]
unsafe fn Get_DeadTorizo(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_DeadTorizo {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_DeadTorizo;
}
#[inline]
unsafe fn Get_MotherBomb(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_MotherBomb {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_MotherBomb;
}
#[inline]
unsafe fn Get_MotherBrain(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_MotherBrain {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_MotherBrain;
}
#[inline]
unsafe fn Get_MotherBrainBody(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MotherBrainBody {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_MotherBrainBody;
}
#[inline]
unsafe fn Get_MotherBrainsTubesFalling(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MotherBrainsTubesFalling {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_MotherBrainsTubesFalling;
}
#[inline]
unsafe fn Get_ShitdroidInCutscene(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_ShitroidInCutscene {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_ShitroidInCutscene;
}
#[inline]
unsafe fn Get_Shitroid(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Shitroid {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_Shitroid;
}
#[inline]
unsafe fn Get_ShitroidInCutscene(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_ShitroidInCutscene {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_ShitroidInCutscene;
}
static g_word_A98B5D: [uint16; 5] = [
    0x10 as ::core::ffi::c_int as uint16,
    0x10 as ::core::ffi::c_int as uint16,
    8 as ::core::ffi::c_int as uint16,
    8 as ::core::ffi::c_int as uint16,
    0x10 as ::core::ffi::c_int as uint16,
];
static g_word_A98B67: [uint16; 5] = [
    0x20 as ::core::ffi::c_int as uint16,
    0x20 as ::core::ffi::c_int as uint16,
    0x18 as ::core::ffi::c_int as uint16,
    0x18 as ::core::ffi::c_int as uint16,
    0x20 as ::core::ffi::c_int as uint16,
];
static g_word_A98B71: [uint16; 5] = [
    0xf8 as ::core::ffi::c_int as uint16,
    0xf8 as ::core::ffi::c_int as uint16,
    0xf0 as ::core::ffi::c_int as uint16,
    0xf0 as ::core::ffi::c_int as uint16,
    0xf6 as ::core::ffi::c_int as uint16,
];
static g_word_A98C61: [uint16; 4] = [
    0xfff8 as ::core::ffi::c_int as uint16,
    2 as ::core::ffi::c_int as uint16,
    0xfffc as ::core::ffi::c_int as uint16,
    6 as ::core::ffi::c_int as uint16,
];
static g_byte_A98F7D: [uint8; 2] = [
    9 as ::core::ffi::c_int as uint8,
    0x12 as ::core::ffi::c_int as uint8,
];
static g_word_A98F7F: [uint16; 8] = [
    0x3d as ::core::ffi::c_int as uint16,
    0x54 as ::core::ffi::c_int as uint16,
    0x20 as ::core::ffi::c_int as uint16,
    0x35 as ::core::ffi::c_int as uint16,
    0x5a as ::core::ffi::c_int as uint16,
    0x43 as ::core::ffi::c_int as uint16,
    0x67 as ::core::ffi::c_int as uint16,
    0x29 as ::core::ffi::c_int as uint16,
];
static g_word_A993BB: [int16; 4] = [
    0 as ::core::ffi::c_int as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    1 as ::core::ffi::c_int as int16,
];
static g_word_A993C3: [int16; 4] = [
    0 as ::core::ffi::c_int as int16,
    1 as ::core::ffi::c_int as int16,
    -(1 as ::core::ffi::c_int) as int16,
    1 as ::core::ffi::c_int as int16,
];
static g_word_A99E0F: [uint16; 13] = [
    0x6f as ::core::ffi::c_int as uint16,
    0x6f as ::core::ffi::c_int as uint16,
    0x6f as ::core::ffi::c_int as uint16,
    0x7e as ::core::ffi::c_int as uint16,
    0x6f as ::core::ffi::c_int as uint16,
    0x6f as ::core::ffi::c_int as uint16,
    0x7e as ::core::ffi::c_int as uint16,
    0x6f as ::core::ffi::c_int as uint16,
    0x6f as ::core::ffi::c_int as uint16,
    0x7e as ::core::ffi::c_int as uint16,
    0x7e as ::core::ffi::c_int as uint16,
    0x6f as ::core::ffi::c_int as uint16,
    0x6f as ::core::ffi::c_int as uint16,
];
static g_word_A9B393: [uint16; 8] = [
    8 as ::core::ffi::c_int as uint16,
    0x6c as ::core::ffi::c_int as uint16,
    0x18 as ::core::ffi::c_int as uint16,
    0x80 as ::core::ffi::c_int as uint16,
    9 as ::core::ffi::c_int as uint16,
    0x90 as ::core::ffi::c_int as uint16,
    0x18 as ::core::ffi::c_int as uint16,
    0x74 as ::core::ffi::c_int as uint16,
];
static g_byte_A9B546: [uint8; 8] = [
    0 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
];
static g_byte_A9B5A1: [uint8; 8] = [
    2 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
];
static g_word_A9B72C: [int16; 28] = [
    -(1 as ::core::ffi::c_int) as int16,
    -(17 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(17 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(17 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
];
static g_word_A9BCA6: [int16; 8] = [
    -(8 as ::core::ffi::c_int) as int16,
    6 as ::core::ffi::c_int as int16,
    -(4 as ::core::ffi::c_int) as int16,
    2 as ::core::ffi::c_int as int16,
    3 as ::core::ffi::c_int as int16,
    -(6 as ::core::ffi::c_int) as int16,
    8 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
];
static g_word_A9BCB6: [int16; 8] = [
    -(7 as ::core::ffi::c_int) as int16,
    2 as ::core::ffi::c_int as int16,
    5 as ::core::ffi::c_int as int16,
    -(4 as ::core::ffi::c_int) as int16,
    6 as ::core::ffi::c_int as int16,
    -(2 as ::core::ffi::c_int) as int16,
    -(6 as ::core::ffi::c_int) as int16,
    7 as ::core::ffi::c_int as int16,
];
static g_byte_A9BEEE: [uint8; 16] = [
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    10 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
];
static g_word_A9BEFE: [uint16; 8] = [
    0x500 as ::core::ffi::c_int as uint16,
    0x500 as ::core::ffi::c_int as uint16,
    0x200 as ::core::ffi::c_int as uint16,
    0x200 as ::core::ffi::c_int as uint16,
    0xc0 as ::core::ffi::c_int as uint16,
    0xc0 as ::core::ffi::c_int as uint16,
    0x40 as ::core::ffi::c_int as uint16,
    0x40 as ::core::ffi::c_int as uint16,
];
static g_word_A9C049: [int16; 8] = [
    0x10 as ::core::ffi::c_int as int16,
    0x10 as ::core::ffi::c_int as int16,
    0x20 as ::core::ffi::c_int as int16,
    0x20 as ::core::ffi::c_int as int16,
    0x30 as ::core::ffi::c_int as int16,
    0x30 as ::core::ffi::c_int as int16,
    0x40 as ::core::ffi::c_int as int16,
    0x40 as ::core::ffi::c_int as int16,
];
static g_word_A9C544: uint16 = 1 as uint16;
static g_byte_A9F56A: [int8; 16] = [
    0x10 as ::core::ffi::c_int as int8,
    0xf as ::core::ffi::c_int as int8,
    0xe as ::core::ffi::c_int as int8,
    0xd as ::core::ffi::c_int as int8,
    0xc as ::core::ffi::c_int as int8,
    0xb as ::core::ffi::c_int as int8,
    0xa as ::core::ffi::c_int as int8,
    9 as ::core::ffi::c_int as int8,
    8 as ::core::ffi::c_int as int8,
    7 as ::core::ffi::c_int as int8,
    6 as ::core::ffi::c_int as int8,
    5 as ::core::ffi::c_int as int8,
    4 as ::core::ffi::c_int as int8,
    3 as ::core::ffi::c_int as int8,
    2 as ::core::ffi::c_int as int8,
    1 as ::core::ffi::c_int as int8,
];
pub unsafe fn CallCorpseRottingMove(mut ea: uint32, mut k: uint16, mut j: uint16, game: &mut Game) {
    match ea {
        11133835 => {
            Torizo_CorpseRottingCopyFunc(k, j, game);
            return;
        }
        11134197 => {
            Sidehopper_CorpseRottingCopyFunc_0(k, j, game);
            return;
        }
        11134454 => {
            Sidehopper_CorpseRottingCopyFunc_2(k, j, game);
            return;
        }
        11134649 => {
            Zoomer_CorpseRottingCopyFunc_0(k, j, game);
            return;
        }
        11134789 => {
            Zoomer_CorpseRottingCopyFunc_2(k, j, game);
            return;
        }
        11134929 => {
            Zoomer_CorpseRottingCopyFunc_4(k, j, game);
            return;
        }
        11135069 => {
            Ripper_CorpseRottingCopyFunc_0(k, j, game);
            return;
        }
        11135209 => {
            Ripper_CorpseRottingCopyFunc_2(k, j, game);
            return;
        }
        11135323 => {
            Skree_CorpseRottingCopyFunc_0(k, j, game);
            return;
        }
        11135417 => {
            Skree_CorpseRottingCopyFunc_2(k, j, game);
            return;
        }
        11135511 => {
            Skree_CorpseRottingCopyFunc_4(k, j, game);
            return;
        }
        11135755 => {
            MotherBrain_CorpseRottingCopyFunc(k, j, game);
            return;
        }
        11133554 => {
            Torizo_CorpseRottingMoveFunc(k, j, game);
            return;
        }
        11134056 => {
            Sidehopper_CorpseRottingMoveFunc_0(k, j, game);
            return;
        }
        11134308 => {
            Sidehopper_CorpseRottingMoveFunc_2(k, j, game);
            return;
        }
        11134570 => {
            Zoomer_CorpseRottingMoveFunc_0(k, j, game);
            return;
        }
        11134710 => {
            Zoomer_CorpseRottingMoveFunc_2(k, j, game);
            return;
        }
        11134850 => {
            Zoomer_CorpseRottingMoveFunc_4(k, j, game);
            return;
        }
        11134990 => {
            Ripper_CorpseRottingMoveFunc_0(k, j, game);
            return;
        }
        11135130 => {
            Ripper_CorpseRottingMoveFunc_2(k, j, game);
            return;
        }
        11135270 => {
            Skree_CorpseRottingMoveFunc_0(k, j, game);
            return;
        }
        11135364 => {
            Skree_CorpseRottingMoveFunc_2(k, j, game);
            return;
        }
        11135458 => {
            Skree_CorpseRottingMoveFunc_4(k, j, game);
            return;
        }
        11135552 => {
            MotherBrain_CorpseRottingMoveFunc(k, j, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CallCorpseRottingInit(mut ea: uint32, game: &mut Game) {
    match ea {
        11132440 => {
            Torizo_CorpseRottingInitFunc(game);
            return;
        }
        11132609 => {
            Sidehopper_CorpseRottingInitFunc_0(game);
            return;
        }
        11132680 => {
            Sidehopper_CorpseRottingInitFunc_2(game);
            return;
        }
        11132751 => {
            Zoomer_CorpseRottingInitFunc_0(game);
            return;
        }
        11132780 => {
            Zoomer_CorpseRottingInitFunc_2(game);
            return;
        }
        11132809 => {
            Zoomer_CorpseRottingInitFunc_4(game);
            return;
        }
        11132838 => {
            Ripper_CorpseRottingInitFunc_0(game);
            return;
        }
        11132867 => {
            Ripper_CorpseRottingInitFunc_2(game);
            return;
        }
        11132896 => {
            Skree_CorpseRottingInitFunc_0(game);
            return;
        }
        11132953 => {
            Skree_CorpseRottingInitFunc_2(game);
            return;
        }
        11133010 => {
            Skree_CorpseRottingInitFunc_4(game);
            return;
        }
        11133067 => {
            MotherBrain_CorpseRottingInitFunc(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CallCorpseRottingFinish(mut ea: uint32, game: &mut Game) {
    match ea {
        11121187 => {
            MotherBrain_CorpseRottingFinished(game);
            return;
        }
        11130301 => {
            DeadTorizo_CorpseRottingFinished(game);
            return;
        }
        11131912 => {
            CorpseRottingRotEntryFinishedHook(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CallMotherBrainFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        11110352 => {
            MotherBrainsBrain_SetupBrainToDraw(game);
            return;
        }
        11110369 => {
            Unreachable(game);
            return;
        }
        11110729 => {
            MotherBrainBody_0_Wait(game);
            return;
        }
        11110766 => {
            MotherBrainBody_1_ClearBottomLeftTube(game);
            return;
        }
        11110787 => {
            MotherBrainBody_2_SpawnTopRightTubeFalling(game);
            return;
        }
        11110816 => {
            MotherBrainBody_3_ClearCeilingBlock9(game);
            return;
        }
        11110837 => {
            MotherBrainBody_4_SpawnTopLeftTubeFalling(game);
            return;
        }
        11110866 => {
            MotherBrainBody_4_ClearCeilingBlock6(game);
            return;
        }
        11110887 => {
            MotherBrainBody_5_SpawnTubeFallingEnemy1(game);
            return;
        }
        11110906 => {
            MotherBrainBody_6_ClearBottomRightTube(game);
            return;
        }
        11110927 => {
            MotherBrainBody_7_SpawnTubeFallingEnemy2(game);
            return;
        }
        11110946 => {
            MotherBrainBody_8_ClearBottomMiddleLeftTube(game);
            return;
        }
        11110967 => {
            MotherBrainBody_9_SpawnTopMiddleLeftFalling(game);
            return;
        }
        11110996 => {
            MotherBrainBody_10_ClearCeilingTubeColumn7(game);
            return;
        }
        11111017 => {
            MotherBrainBody_11_SpawnTopMiddleRightFalling(game);
            return;
        }
        11111046 => {
            MotherBrainBody_12_ClearCeilingTubeColumn8(game);
            return;
        }
        11111067 => {
            MotherBrainBody_13_SpawnTubeFallingEnemy3(game);
            return;
        }
        11111086 => {
            MotherBrainBody_14_ClearBottomMiddleRightTube(game);
            return;
        }
        11111107 => {
            MotherBrainBody_15_SpawnTubeFallingEnemy4(game);
            return;
        }
        11111126 => {
            MotherBrainBody_16_ClearBottomMiddleTubes(game);
            return;
        }
        11110429 => {
            MotherBrainBody_FakeDeath_Descent_0_Pause(game);
            return;
        }
        11110441 => {
            MotherBrainBody_FakeDeath_Descent_1(game);
            return;
        }
        11110477 => {
            MotherBrainBody_FakeDeath_Descent_2(game);
            return;
        }
        11110508 => {
            MotherBrainBody_FakeDeath_Descent_3(game);
            return;
        }
        11110532 => {
            MotherBrainBody_FakeDeath_Descent_4(game);
            return;
        }
        11110578 => {
            MotherBrainBody_FakeDeath_Descent_5(game);
            return;
        }
        11110611 => {
            MotherBrainBody_FakeDeath_Descent_6(game);
            return;
        }
        11111559 => {
            MotherBrainBody_FakeDeath_Ascent_0_DrawBG1Row23(game);
            return;
        }
        11111582 => {
            MotherBrainBody_FakeDeath_Ascent_1_DrawBG1Row45(game);
            return;
        }
        11111605 => {
            MotherBrainBody_FakeDeath_Ascent_2_DrawBG1Row67(game);
            return;
        }
        11111628 => {
            MotherBrainBody_FakeDeath_Ascent_3_DrawBG1Row89(game);
            return;
        }
        11111651 => {
            MotherBrainBody_FakeDeath_Ascent_4_DrawBG1RowAB(game);
            return;
        }
        11111674 => {
            MotherBrainBody_FakeDeath_Ascent_5_DrawBG1RowCD(game);
            return;
        }
        11111697 => {
            MotherBrainBody_FakeDeath_Ascent_6_SetupPhase2Gfx(game);
            return;
        }
        11111753 => {
            MotherBrainBody_FakeDeath_Ascent_7_SetupPhase2Brain(game);
            return;
        }
        11111801 => {
            MotherBrainBody_FakeDeath_Ascent_8_Pause(game);
            return;
        }
        11111819 => {
            MotherBrainBody_FakeDeath_Ascent_9_PrepareRise(game);
            return;
        }
        11111860 => {
            MotherBrainBody_FakeDeath_Ascent_10_LoadLegTiles(game);
            return;
        }
        11111875 => {
            MotherBrainBody_FakeDeath_Ascent_11_ContinuePause(game);
            return;
        }
        11111916 => {
            MotherBrainBody_FakeDeath_Ascent_12_StartMusic(game);
            return;
        }
        11112013 => {
            MotherBrainBody_FakeDeath_Ascent_13_Raise(game);
            return;
        }
        11112085 => {
            MotherBrainBody_FakeDeath_Ascent_14_WaitForUncouching(game);
            return;
        }
        11112106 => {
            MotherBrainBody_FakeDeath_Ascent_15_TransitionFromGrey(game);
            return;
        }
        11112181 => {
            MotherBrainBody_2ndphase_16_ShakeHeadMenacingly(game);
            return;
        }
        11112212 => {
            MotherBrainBody_2ndphase_17_BringHeadBackUp(game);
            return;
        }
        11112243 => {
            MotherBrainBody_2ndphase_18_FinishStretching(game);
            return;
        }
        11120353 => {
            MotherBrain_Phase3_Death_0(game);
            return;
        }
        11120402 => {
            MotherBrain_Phase3_Death_1(game);
            return;
        }
        11120417 => {
            MotherBrain_Phase3_Death_2(game);
            return;
        }
        11120468 => {
            MotherBrain_Phase3_Death_3(game);
            return;
        }
        11120541 => {
            MotherBrain_Phase3_Death_4(game);
            return;
        }
        11120566 => {
            MotherBrain_Phase3_Death_5(game);
            return;
        }
        11120659 => {
            MotherBrain_Phase3_Death_6(game);
            return;
        }
        11120917 => {
            MotherBrain_Phase3_Death_7(game);
            return;
        }
        11120941 => {
            MotherBrain_Phase3_Death_8(game);
            return;
        }
        11120990 => {
            MotherBrain_Phase3_Death_9(game);
            return;
        }
        11121011 => {
            MotherBrain_Phase3_Death_10(game);
            return;
        }
        11121033 => {
            MotherBrain_Phase3_Death_11(game);
            return;
        }
        11121080 => {
            MotherBrain_Phase3_Death_12(game);
            return;
        }
        11121109 => {
            MotherBrain_Phase3_Death_13(game);
            return;
        }
        11121169 => {
            MotherBrain_Phase3_Death_14_20framedelay(game);
            return;
        }
        11121240 => {
            MotherBrain_Phase3_Death_15_LoadEscapeTimerTiles(game);
            return;
        }
        11121261 => {
            MotherBrain_Phase3_Death_16_StartEscape(game);
            return;
        }
        11121361 => {
            MotherBrain_Phase3_Death_17_SpawnTimeBomb(game);
            return;
        }
        11121379 => {
            MotherBrain_Phase3_Death_18_TypesZebesText(game);
            return;
        }
        11121401 => {
            MotherBrain_Phase3_Death_19_EscapeDoorExploding(game);
            return;
        }
        11121450 => {
            MotherBrain_Phase3_Death_20_BlowUpEscapeDoor(game);
            return;
        }
        11121468 => {
            MotherBrain_Phase3_Death_21_KeepEarthquakeGoing(game);
            return;
        }
        11122181 => {
            MotherBrain_Body_Phase2_Thinking(game);
            return;
        }
        11122251 => {
            MotherBrain_Body_Phase2_TryAttack(game);
            return;
        }
        11122561 => {
            MotherBrain_FiringBomb_DecideOnWalking(game);
            return;
        }
        11122604 => {
            MotherBrain_FiringBomb_WalkingBackwards(game);
            return;
        }
        11122630 => {
            MotherBrain_FiringBomb_Crouch(game);
            return;
        }
        11122664 => {
            MotherBrain_FiringBomb_Fired(game);
            return;
        }
        11122680 => {
            MotherBrain_FiringBomb_Standup(game);
            return;
        }
        11122702 => {
            MotherBomb_FiringLaser_PositionHead(game);
            return;
        }
        11122745 => {
            MotherBomb_FiringLaser_PositionHeadSlowlyFire(game);
            return;
        }
        11122787 => {
            MotherBomb_FiringLaser_FinishAttack(game);
            return;
        }
        11122813 => {
            MotherBomb_FiringDeathBeam(game);
            return;
        }
        11122923 => {
            MotherBomb_FiringRainbowBeam_0(game);
            return;
        }
        11122970 => {
            MotherBomb_FiringRainbowBeam_1_StartCharge(game);
            return;
        }
        11122987 => {
            MotherBomb_FiringRainbowBeam_2_RetractNeck(game);
            return;
        }
        11123007 => {
            MotherBomb_FiringRainbowBeam_3_Wait(game);
            return;
        }
        11123025 => {
            MotherBomb_FiringRainbowBeam_4_ExtendNeckDown(game);
            return;
        }
        11123061 => {
            MotherBomb_FiringRainbowBeam_5_StartFiring(game);
            return;
        }
        11123173 => {
            MotherBomb_FiringRainbowBeam_6_MoveSamusToWall(game);
            return;
        }
        11123200 => {
            MotherBomb_FiringRainbowBeam_7_DelayFrame(game);
            return;
        }
        11123239 => {
            MotherBomb_FiringRainbowBeam_8_StartDrainSamus(game);
            return;
        }
        11123260 => {
            MotherBomb_FiringRainbowBeam_9_DrainingSamus(game);
            return;
        }
        11123294 => {
            MotherBomb_FiringRainbowBeam_10_FinishFiringRainbow(game);
            return;
        }
        11123396 => {
            MotherBomb_FiringRainbowBeam_11_LetSamusFall(game);
            return;
        }
        11123409 => {
            MotherBomb_FiringRainbowBeam_12_WaitForSamusHitGround(game);
            return;
        }
        11123421 => {
            MotherBomb_FiringRainbowBeam_13_LowerHead(game);
            return;
        }
        11123462 => {
            MotherBomb_FiringRainbowBeam_14_DecideNextAction(game);
            return;
        }
        11124037 => {
            MotherBrain_Phase2Cut_0(game);
            return;
        }
        11124120 => {
            MotherBrain_Phase2Cut_1(game);
            return;
        }
        11124137 => {
            MotherBrain_Phase2Cut_2(game);
            return;
        }
        11124161 => {
            MotherBrain_Phase2Cut_3(game);
            return;
        }
        11124178 => {
            MotherBrain_Phase2Cut_4(game);
            return;
        }
        11124205 => {
            MotherBrain_Phase2Cut_5(game);
            return;
        }
        11124250 => return,
        11124280 => {
            MotherBrain_DrainedByShitroid_0(game);
            return;
        }
        11124317 => {
            MotherBrain_DrainedByShitroid_1(game);
            return;
        }
        11124374 => {
            MotherBrain_DrainedByShitroid_2(game);
            return;
        }
        11124494 => {
            MotherBrain_DrainedByShitroid_3(game);
            return;
        }
        11124545 => {
            MotherBrain_DrainedByShitroid_4(game);
            return;
        }
        11124566 => {
            MotherBrain_DrainedByShitroid_5(game);
            return;
        }
        11124605 => {
            MotherBrain_DrainedByShitroid_6(game);
            return;
        }
        11124629 => {
            MotherBrain_DrainedByShitroid_7(game);
            return;
        }
        11124825 => {
            MotherBrain_Phase2_Revive_0(game);
            return;
        }
        11124838 => {
            MotherBrain_Phase2_Revive_1(game);
            return;
        }
        11124866 => {
            MotherBrain_Phase2_Revive_2(game);
            return;
        }
        11124879 => {
            MotherBrain_Phase2_Revive_3(game);
            return;
        }
        11124922 => {
            MotherBrain_Phase2_Revive_4(game);
            return;
        }
        11124964 => {
            MotherBrain_Phase2_Revive_5(game);
            return;
        }
        11124987 => {
            MotherBrain_Phase2_Revive_6(game);
            return;
        }
        11125022 => {
            MotherBrain_Phase2_Revive_7(game);
            return;
        }
        11125063 => {
            MotherBrain_Phase2_Revive_8(game);
            return;
        }
        11125084 => {
            MotherBrain_Phase2_MurderShitroid_1(game);
            return;
        }
        11125122 => {
            MotherBrain_Phase2_MurderShitroid_2(game);
            return;
        }
        11125134 => {
            MotherBrain_Phase2_PrepareForFinalShitroid(game);
            return;
        }
        11125146 => {
            MotherBrain_Phase2_ExecuteFinalkShitroid(game);
            return;
        }
        11125158 => return,
        11125199 => {
            MotherBrain_Phase3_Recover_MakeDistance(game);
            return;
        }
        11125232 => {
            MotherBrain_Phase3_Recover_SetupForFight(game);
            return;
        }
        11125257 => {
            MotherBrain_Phase3_Fighting_Main(game);
            return;
        }
        11125326 => {
            MotherBrain_Phase3_Fighting_Cooldown(game);
            return;
        }
        11111140 => return,
        11125133 => return,
        11110306 => {
            MotherBrainsBrain_SetupBrainAndNeckToDraw(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn Enemy_GrappleReact_CancelBeam_A9(game: &mut Game) {
    Enemy_SwitchToFrozenAi(game);
}
pub unsafe fn Enemy_NormalFrozenAI_A9(game: &mut Game) {
    NormalEnemyFrozenAI(game);
}
pub unsafe fn MotherBrainsBody_Init(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 4094 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    MotherBrain_SetBodyInstrs(
        addr_kMotherBrain_Ilist_9C13 as ::core::ffi::c_int as uint16,
        game,
    );
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.vram_tiles_index = 0 as uint16;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_BlockPlasmaBeam as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    (*E).base.palette_index = 0 as uint16;
    WriteColorsToTargetPalette(
        0xa9 as uint8,
        0x162 as uint16,
        (addr_kMotherBrainPalette_4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
        0xf as uint16,
        game,
    );
    WriteColorsToTargetPalette(
        0xa9 as uint8,
        0x1e2 as uint16,
        (addr_kMotherBrainPalette_3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
        0xf as uint16,
        game,
    );
    (*E).mbn_var_00 = 0 as uint16;
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).mbn_var_02 = 0 as uint16;
    (*E).mbn_var_04 = 2 as uint16;
    (*E1).mbn_var_A = (0xa987d0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_A = (0xa987e1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    LoadFxEntry(1 as uint16, game);
    let mut v3: uint16 = 0 as uint16;
    let mut v4: uint16 = 0;
    loop {
        v4 = v3;
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainRoomTurrets as ::core::ffi::c_int as uint16,
            v3,
            game,
        );
        v3 = (v4 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        if !(((v4 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int)
            < 0xc as ::core::ffi::c_int)
        {
            break;
        }
    }
}
pub unsafe fn MotherBrainsBrain_Init(game: &mut Game) {
    InitializeEnemyCorpseRotting(
        0x40 as uint16,
        addr_stru_A9DE08 as ::core::ffi::c_int as uint16,
        game,
    );
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).base.health = 3000 as uint16;
    MotherBrain_SetBrainUnusedInstrs(
        addr_kMotherBrain_Ilist_9C13 as ::core::ffi::c_int as uint16,
        game,
    );
    (*E1).base.vram_tiles_index = 0 as uint16;
    (*E1).base.properties =
        ((*E1).base.properties as ::core::ffi::c_int | 0x1100 as ::core::ffi::c_int) as uint16;
    (*E1).base.palette_index = 512 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_0C = 512 as uint16;
    (*E).mbn_var_0D = 512 as uint16;
    MotherBrain_SetBrainInstrs(addr_stru_A99C21 as ::core::ffi::c_int as uint16, game);
    MotherBrain_SetupBrainNormalPal(game);
}
pub unsafe fn MotherBrainsBody_Hurt(game: &mut Game) {
    MotherBrain_Pal_HandleRoomPal(game);
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if (*E).mbn_var_A as ::core::ffi::c_int
        == fnMotherBrainsBody_FirstPhase_DoubleRet as uint16 as ::core::ffi::c_int
    {
        MotherBrainsBody_FirstPhase_DoubleRet(game);
        return;
    }
    CallMotherBrainFunc(
        ((*E).mbn_var_A as ::core::ffi::c_int | 0xa90000 as ::core::ffi::c_int) as uint32,
        game,
    );
    MotherBrain_HandlePalette(game);
    MotherBrain_SamusCollDetect(game);
    MotherBrain_Pal_ProcessInvincibility(game);
    if (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_02 != 0 {
        mov24(
            (&raw mut game.g_ram as *mut uint8).offset(0x604 as ::core::ffi::c_int as isize)
                as *mut LongPtr,
            fnMotherBrainsBody_UnpauseHook as uint32,
        );
    }
}
pub unsafe fn MotherBrainsBody_UnpauseHook(game: &mut Game) -> CoroutineRet {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if (*E).mbn_var_16 != 0 {
        QueueSfx1_Max6(0x40 as uint16, game);
    }
    if (*E).base.extra_properties as ::core::ffi::c_int & 4 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x179a as ::core::ffi::c_int as isize)
            as *mut uint16) = 2048 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe1e as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn MotherBrainsBody_Powerbomb(game: &mut Game) {
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
    MotherBrainsBrain_Hurt(game);
}
pub unsafe fn MotherBrainsBrain_Hurt(game: &mut Game) {
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0x178c as ::core::ffi::c_int as isize)
            as *mut LongPtr,
        0xa98786 as uint32,
    );
    let mut v0: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
    if (*v0.offset(1 as ::core::ffi::c_int as isize)).properties as ::core::ffi::c_int
        & 0x100 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        CallMotherBrainFunc(
            ((*v0.offset(1 as ::core::ffi::c_int as isize)).ai_var_A as ::core::ffi::c_int
                | 0xa90000 as ::core::ffi::c_int) as uint32,
            game,
        );
    }
}
pub unsafe fn MotherBrainsBrain_SetupBrainAndNeckToDraw(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
    {
        MotherBrain_HandleNeck(game);
    }
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).base.x_pos = (*E1).mbn_var_2E;
    (*E1).base.y_pos =
        ((*E1).mbn_var_2F as ::core::ffi::c_int - 21 as ::core::ffi::c_int) as uint16;
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0x178c as ::core::ffi::c_int as isize)
            as *mut LongPtr,
        0xa987c9 as uint32,
    );
}
pub unsafe fn MotherBrain_DrawBrainNeck_EnemyGfxDrawHook(game: &mut Game) {
    MotherBrain_DrawBrain(game);
    MotherBrain_DrawNeck(game);
}
pub unsafe fn MotherBrainsBrain_SetupBrainToDraw(game: &mut Game) {
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0x178c as ::core::ffi::c_int as isize)
            as *mut LongPtr,
        0xa987dd as uint32,
    );
}
pub unsafe fn MotherBrainsBrain_GfxDrawHook(game: &mut Game) {
    MotherBrain_DrawBrain(game);
}
pub unsafe fn MotherBrainsBody_FirstPhase_DoubleRet(game: &mut Game) {
    if CheckEventHappened(2 as uint16, game) != 0 {
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_00 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1840 as ::core::ffi::c_int as isize)
            as *mut uint16);
        if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 236 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if (*E1).base.health == 0 {
                let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
                (*E).mbn_var_1D = 1 as uint16;
                (*E).mbn_var_00 = 1 as uint16;
                DisableMinimapAndMarkBossRoomAsExplored(game);
                QueueMusic_Delayed8(6 as uint16, game);
                MotherBrain_SealWall(game);
            }
        }
    }
    MotherBrain_SamusCollDetect(game);
}
pub unsafe fn MotherBrainBody_FakeDeath_Descent_0_Pause(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0 as ::core::ffi::c_int, game);
    (*E).mbby_var_A = (0xa98829 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbby_var_F = 64 as uint16;
    MotherBrainBody_FakeDeath_Descent_1(game);
}
pub unsafe fn MotherBrainBody_FakeDeath_Descent_1(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0 as ::core::ffi::c_int, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if (*E).mbby_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        CallSomeSamusCode(0 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20 as ::core::ffi::c_int as isize)
            as *mut uint16) = *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20 as ::core::ffi::c_int as isize)
            .offset(0 as ::core::ffi::c_int as isize) as uint16;
        (*E).mbby_var_A = (0xa9884d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbby_var_F = 32 as uint16;
        MotherBrainBody_FakeDeath_Descent_2(game);
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Descent_2(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0 as ::core::ffi::c_int, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if (*E).mbby_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        QueueMusic_Delayed8(0 as uint16, game);
        QueueMusic_Delayed8(0xff21 as uint16, game);
        (*E).mbby_var_A = (0xa9886c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbby_var_F = 12 as uint16;
        MotherBrainBody_FakeDeath_Descent_3(game);
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Descent_3(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0 as ::core::ffi::c_int, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if (*E).mbby_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        CallSomeSamusCode(1 as uint16, game);
        (*E).mbby_var_A = (0xa98884 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbby_var_F = 8 as uint16;
        MotherBrainBody_FakeDeath_Descent_4(game);
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Descent_4(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        MotherBrain_Pal_BeginScreenFlash(game);
        LoadFxEntry(2 as uint16, game);
        (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_E =
            (0xa98949 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_A = (0xa988b2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 0 as uint16;
        (*E).mbn_var_37 = 0 as uint16;
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0xe as uint8,
                field_1: 0x2 as uint8,
                field_2: 0xb6b3 as VoidP,
            },
            game,
        );
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Descent_5(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_F = 8 as uint16;
        let mut v2: uint16 =
            ((*E).mbn_var_37 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_37 = v2;
        if MotherBrain_FadeToGray_FakeDeath(
            (v2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
            game,
        ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            (*E).mbn_var_A =
                (0xa988d3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
    MotherBrainBody_FakeDeath_Descent_6(game);
}
pub unsafe fn MotherBrainBody_FakeDeath_Descent_6(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    CallMotherBrainFunc(
        ((*E).mbby_var_E as ::core::ffi::c_int | 0xa90000 as ::core::ffi::c_int) as uint32,
        game,
    );
    MotherBrain_HandleFakeDeathExplosions(game);
}
pub unsafe fn MotherBrain_HandleFakeDeathExplosions(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    v1 = ((*E).mbn_var_38 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
    if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).mbn_var_38 = 8 as uint16;
        v2 = ((*E).mbn_var_39 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
        if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            v2 = 7 as int16;
        }
        (*E).mbn_var_39 = v2 as uint16;
        let mut v3: ::core::ffi::c_int = (4 as ::core::ffi::c_int * v2 as ::core::ffi::c_int)
            as uint16 as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        game.eproj_spawn_pt = Point16U {
            x: *(game.g_rom.offset(
                (((0xa98929 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa98929 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v3 as isize),
            y: *(game.g_rom.offset(
                (((0xa98929 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa98929 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v3 + 1 as ::core::ffi::c_int) as isize),
        };
        let mut v4: uint16 = (if (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
            < 0x4000 as ::core::ffi::c_int
        {
            12 as ::core::ffi::c_int
        } else {
            3 as ::core::ffi::c_int
        }) as uint16;
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
            v4,
            game,
        );
        QueueSfx2_Max3(0x24 as uint16, game);
    } else {
        (*E).mbn_var_38 = v1 as uint16;
    };
}
pub unsafe fn MotherBrainBody_0_Wait(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v0: uint16 = 0 as uint16;
    v1 = 0 as int16;
    loop {
        if *(((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16)
            == 0
        {
            v1 += 1;
        }
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int - 36 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    if (v1 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        SpawnEnemy(
            0xa9 as uint8,
            addr_stru_A98AE5 as ::core::ffi::c_int as uint16,
            game,
        );
        (*Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game)).mbby_var_E =
            (0xa9896e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrainBody_1_ClearBottomLeftTube(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x5 as uint8,
            field_1: 0x9 as uint8,
            field_2: 0xb6c3 as VoidP,
        },
        game,
    );
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_E = (0xa98983 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbby_var_F = 32 as uint16;
}
pub unsafe fn MotherBrainBody_2_SpawnTopRightTubeFalling(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if (*E).mbby_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        game.eproj_spawn_pt = Point16U {
            x: 152 as uint16,
            y: 47 as uint16,
        };
        SpawnEprojWithRoomGfx(0xcc5b as uint16, 0x2f as uint16, game);
        (*E).mbby_var_E = -(30304 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrainBody_3_ClearCeilingBlock9(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x9 as uint8,
            field_1: 0x2 as uint8,
            field_2: 0xb6b3 as VoidP,
        },
        game,
    );
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_E = (0xa989b5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbby_var_F = 32 as uint16;
}
pub unsafe fn MotherBrainBody_4_SpawnTopLeftTubeFalling(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if (*E).mbby_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        game.eproj_spawn_pt = Point16U {
            x: 104 as uint16,
            y: 47 as uint16,
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainTubeFalling_TopLeft as ::core::ffi::c_int as uint16,
            0x2f as uint16,
            game,
        );
        (*E).mbby_var_E = (0xa989d2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrainBody_4_ClearCeilingBlock6(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x6 as uint8,
            field_1: 0x2 as uint8,
            field_2: 0xb6b3 as VoidP,
        },
        game,
    );
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_E = (0xa989e7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbby_var_F = 32 as uint16;
}
pub unsafe fn MotherBrainBody_5_SpawnTubeFallingEnemy1(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if (*E).mbby_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        SpawnEnemy(
            0xa9 as uint8,
            addr_stru_A98AF5 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).mbby_var_E = (0xa989fa as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrainBody_6_ClearBottomRightTube(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0xa as uint8,
            field_1: 0x9 as uint8,
            field_2: 0xb6c7 as VoidP,
        },
        game,
    );
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_E = (0xa98a0f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbby_var_F = 32 as uint16;
}
pub unsafe fn MotherBrainBody_7_SpawnTubeFallingEnemy2(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if (*E).mbby_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        SpawnEnemy(
            0xa9 as uint8,
            addr_stru_A98B05 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).mbby_var_E = (0xa98a22 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrainBody_8_ClearBottomMiddleLeftTube(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x6 as uint8,
            field_1: 0xa as uint8,
            field_2: 0xb6bb as VoidP,
        },
        game,
    );
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_E = (0xa98a37 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbby_var_F = 32 as uint16;
}
pub unsafe fn MotherBrainBody_9_SpawnTopMiddleLeftFalling(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if (*E).mbby_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        game.eproj_spawn_pt = Point16U {
            x: 120 as uint16,
            y: 59 as uint16,
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainTubeFalling_TopMiddleLeft as ::core::ffi::c_int as uint16,
            0x3b as uint16,
            game,
        );
        (*E).mbby_var_E = (0xa98a54 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrainBody_10_ClearCeilingTubeColumn7(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x7 as uint8,
            field_1: 0x2 as uint8,
            field_2: 0xb6b7 as VoidP,
        },
        game,
    );
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_E = (0xa98a69 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbby_var_F = 32 as uint16;
}
pub unsafe fn MotherBrainBody_11_SpawnTopMiddleRightFalling(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if (*E).mbby_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        game.eproj_spawn_pt = Point16U {
            x: 136 as uint16,
            y: 59 as uint16,
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainTubeFalling_TopMiddleRight as ::core::ffi::c_int as uint16,
            0x3b as uint16,
            game,
        );
        (*E).mbby_var_E = (0xa98a86 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrainBody_12_ClearCeilingTubeColumn8(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x8 as uint8,
            field_1: 0x2 as uint8,
            field_2: 0xb6b7 as VoidP,
        },
        game,
    );
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_E = (0xa98a9b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbby_var_F = 32 as uint16;
}
pub unsafe fn MotherBrainBody_13_SpawnTubeFallingEnemy3(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if (*E).mbby_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        SpawnEnemy(
            0xa9 as uint8,
            (0x868b15 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
            game,
        );
        (*E).mbby_var_E = (0xa98aae as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrainBody_14_ClearBottomMiddleRightTube(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x9 as uint8,
            field_1: 0xa as uint8,
            field_2: 0xb6bb as VoidP,
        },
        game,
    );
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_E = (0xa98ac3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbby_var_F = 2 as uint16;
}
pub unsafe fn MotherBrainBody_15_SpawnTubeFallingEnemy4(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if (*E).mbby_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        SpawnEnemy(
            0xa9 as uint8,
            addr_stru_A98B25 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).mbby_var_E = (0xa98ad6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrainBody_16_ClearBottomMiddleTubes(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x7 as uint8,
            field_1: 0x7 as uint8,
            field_2: 0xb6bf as VoidP,
        },
        game,
    );
    (*Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game)).mbby_var_E =
        addr_locret_A98AE4 as ::core::ffi::c_int as uint16;
}
pub unsafe fn MotherBrainsTubesFalling_Init(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainsTubesFalling = Get_MotherBrainsTubesFalling(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v1: ::core::ffi::c_int =
        (*E).mbtfg_parameter_1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).base.x_width = g_word_A98B5D[v1 as usize];
    (*E).base.y_height = g_word_A98B67[v1 as usize];
    (*E).mbtfg_var_B = g_word_A98B71[v1 as usize];
    (*E).mbtfg_var_D = 0 as uint16;
    (*E).mbtfg_var_E = 0 as uint16;
    (*E).mbtfg_var_C = 0 as uint16;
    (*E).mbtfg_var_A = *(game.g_rom.offset(
        (((0xa98b7b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa98b7b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize);
}
pub unsafe fn CallMotherBrainsTubesFalling(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        11111304 => {
            MotherBrainsTubesFalling_Main_NonMain(k, game);
            return;
        }
        11111371 => {
            MotherBrainsTubesFalling_WaitToFall(k, game);
            return;
        }
        11111382 => {
            MotherBrainsTubesFalling_Falling(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn MotherBrainsTubesFalling_Main(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainsTubesFalling =
        Get_MotherBrainsTubesFalling(k as ::core::ffi::c_int, game);
    CallMotherBrainsTubesFalling(
        ((*E).mbtfg_var_A as ::core::ffi::c_int | 0xa90000 as ::core::ffi::c_int) as uint32,
        k,
        game,
    );
}
pub unsafe fn MotherBrainsTubesFalling_Main_NonMain(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainsTubesFalling =
        Get_MotherBrainsTubesFalling(k as ::core::ffi::c_int, game);
    let mut v2: uint16 =
        ((*E).mbtfg_var_C as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
    (*E).mbtfg_var_C = v2;
    Enemy_IncreaseYpos(k, v2, game);
    if (((*E).base.y_pos as ::core::ffi::c_int - (*E).mbtfg_var_B as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        MotherBrainsTubesFalling_HandleSmoke(k, game);
    } else {
        MotherBrainsTubesFalling_Explode(k, game);
    };
}
pub unsafe fn MotherBrainsTubesFalling_HandleSmoke(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainsTubesFalling =
        Get_MotherBrainsTubesFalling(k as ::core::ffi::c_int, game);
    (*E).mbtfg_var_D = (*E).mbtfg_var_D.wrapping_sub(1);
    if (*E).mbtfg_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        MotherBrainsTubesFalling_SpawnSmoke(k, game);
    }
}
pub unsafe fn MotherBrainsTubesFalling_Explode(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainsTubesFalling =
        Get_MotherBrainsTubesFalling(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
    game.eproj_spawn_pt = Point16U {
        x: (*E).base.x_pos,
        y: (*E).base.y_pos,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        3 as uint16,
        game,
    );
    QueueSfx2_Max3(0x24 as uint16, game);
}
pub unsafe fn MotherBrainsTubesFalling_WaitToFall(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainsTubesFalling =
        Get_MotherBrainsTubesFalling(k as ::core::ffi::c_int, game);
    (*E).mbtfg_parameter_2 = (*E).mbtfg_parameter_2.wrapping_sub(1);
    if (*E).mbtfg_parameter_2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbtfg_var_A =
            (0xa98bd6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBrainsTubesFalling_Falling(k, game);
    }
}
pub unsafe fn MotherBrainsTubesFalling_Falling(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainsTubesFalling =
        Get_MotherBrainsTubesFalling(k as ::core::ffi::c_int, game);
    let mut v2: uint16 =
        ((*E).mbtfg_var_C as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
    (*E).mbtfg_var_C = v2;
    Enemy_IncreaseYpos(k, v2, game);
    let mut ypos: uint16 = (*E).base.y_pos;
    if ypos as ::core::ffi::c_int - 244 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
    }
    let mut E1: *mut Enemy_MotherBrainsTubesFalling =
        Get_MotherBrainsTubesFalling(0x40 as ::core::ffi::c_int, game);
    (*E1).base.y_pos = (ypos as ::core::ffi::c_int - 56 as ::core::ffi::c_int) as uint16;
    if (*E1).base.y_pos as ::core::ffi::c_int - 0xc4 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        MotherBrainsTubesFalling_HandleSmoke(k, game);
    } else {
        MotherBrain_Pal_EndScreenFlash(game);
        EnableEarthquakeAframes(0x19 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = 0 as uint16;
        (*E1).base.y_pos = 196 as uint16;
        let mut E0: *mut Enemy_MotherBrainsTubesFalling =
            Get_MotherBrainsTubesFalling(0 as ::core::ffi::c_int, game);
        (*E0).base.x_pos = 59 as uint16;
        (*E0).base.y_pos = 279 as uint16;
        MotherBrain_SetupNeckForFakeAscent(game);
        (*E0).mbtfg_var_A =
            (0xa98c87 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBrainsTubesFalling_Explode(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            game,
        );
    };
}
pub unsafe fn MotherBrainsTubesFalling_SpawnSmoke(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainsTubesFalling =
        Get_MotherBrainsTubesFalling(k as ::core::ffi::c_int, game);
    (*E).mbtfg_var_D = 8 as uint16;
    let mut v2: uint16 = ((*E).mbtfg_var_E as ::core::ffi::c_int + 1 as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int) as uint16;
    (*E).mbtfg_var_E = v2;
    game.eproj_spawn_pt = Point16U {
        x: ((*E).base.x_pos as ::core::ffi::c_int
            + g_word_A98C61[v2 as usize] as ::core::ffi::c_int) as uint16,
        y: 208 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        9 as uint16,
        game,
    );
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_0_DrawBG1Row23(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x2 as uint8,
            field_1: 0x2 as uint8,
            field_2: 0xb67b as VoidP,
        },
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x2 as uint8,
            field_1: 0x3 as uint8,
            field_2: 0xb67f as VoidP,
        },
        game,
    );
    (*Get_MotherBrainBody(0 as ::core::ffi::c_int, game)).mbby_var_A =
        (0xa98c9e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_1_DrawBG1Row45(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x2 as uint8,
            field_1: 0x4 as uint8,
            field_2: 0xb683 as VoidP,
        },
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x2 as uint8,
            field_1: 0x5 as uint8,
            field_2: 0xb687 as VoidP,
        },
        game,
    );
    (*Get_MotherBrainBody(0 as ::core::ffi::c_int, game)).mbby_var_A =
        (0xa98cb5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_2_DrawBG1Row67(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x2 as uint8,
            field_1: 0x6 as uint8,
            field_2: 0xb68b as VoidP,
        },
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x2 as uint8,
            field_1: 0x7 as uint8,
            field_2: 0xb68f as VoidP,
        },
        game,
    );
    (*Get_MotherBrainBody(0 as ::core::ffi::c_int, game)).mbby_var_A =
        (0xa98ccc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_3_DrawBG1Row89(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x2 as uint8,
            field_1: 0x8 as uint8,
            field_2: 0xb693 as VoidP,
        },
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x2 as uint8,
            field_1: 0x9 as uint8,
            field_2: 0xb697 as VoidP,
        },
        game,
    );
    (*Get_MotherBrainBody(0 as ::core::ffi::c_int, game)).mbby_var_A =
        (0xa98ce3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_4_DrawBG1RowAB(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x2 as uint8,
            field_1: 0xa as uint8,
            field_2: 0xb69b as VoidP,
        },
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x2 as uint8,
            field_1: 0xb as uint8,
            field_2: 0xb69f as VoidP,
        },
        game,
    );
    (*Get_MotherBrainBody(0 as ::core::ffi::c_int, game)).mbby_var_A =
        (0xa98cfa as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_5_DrawBG1RowCD(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x2 as uint8,
            field_1: 0xc as uint8,
            field_2: 0xb6a3 as VoidP,
        },
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x2 as uint8,
            field_1: 0xd as uint8,
            field_2: 0xb6a7 as VoidP,
        },
        game,
    );
    (*Get_MotherBrainBody(0 as ::core::ffi::c_int, game)).mbby_var_A =
        (0xa98d11 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_6_SetupPhase2Gfx(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x91b as ::core::ffi::c_int as isize)
        as *mut uint16) = 257 as uint16;
    let ref mut fresh8 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x59 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh8 = (*fresh8 as ::core::ffi::c_int & 0xfffc as ::core::ffi::c_int) as uint16;
    WriteColorsToPalette(
        0x142 as uint16,
        0xa9 as uint8,
        (addr_kMotherBrainPalette_1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
        0xf as uint16,
        game,
    );
    WriteColorsToPalette(
        0x162 as uint16,
        0xa9 as uint8,
        (addr_kMotherBrainPalette_0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
        0xf as uint16,
        game,
    );
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
        (0xa98d49 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xe1e as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
    (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_02 = 1 as uint16;
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_7_SetupPhase2Brain(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1982 as ::core::ffi::c_int as isize)
        as *mut uint16) = 52 as uint16;
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_A = (0xa987a2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut E0: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0 as ::core::ffi::c_int, game);
    (*E0).base.properties = ((*E0).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
    (*E).base.properties =
        ((*E).base.properties as ::core::ffi::c_int & !(0x400 as ::core::ffi::c_int)) as uint16;
    (*E).base.health = 18000 as uint16;
    (*E0).mbby_var_A = (0xa98d79 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E0).mbby_var_F = 128 as uint16;
    MotherBrainBody_FakeDeath_Ascent_8_Pause(game);
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_8_Pause(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0 as ::core::ffi::c_int, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if (*E).mbby_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbby_var_A = (0xa98d8b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbby_var_F = 32 as uint16;
        MotherBrainBody_FakeDeath_Ascent_9_PrepareRise(game);
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_9_PrepareRise(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_09 = MotherBrainRisingHdmaObject(game);
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).base.properties =
            ((*E1).base.properties as ::core::ffi::c_int | 0x100 as ::core::ffi::c_int) as uint16;
        MotherBrain_SetBrainInstrs(addr_stru_A99C21 as ::core::ffi::c_int as uint16, game);
        (*E).mbn_var_A = (0xa98db4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 256 as uint16;
        MotherBrainBody_FakeDeath_Ascent_10_LoadLegTiles(game);
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_10_LoadLegTiles(game: &mut Game) {
    if ProcessSpriteTilesTransfers(
        0xa9 as uint8,
        addr_stru_A98F8F as ::core::ffi::c_int as uint16,
        game,
    ) != 0
    {
        (*Get_MotherBrainBody(0 as ::core::ffi::c_int, game)).mbby_var_A =
            (0xa98dc3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBrainBody_FakeDeath_Ascent_11_ContinuePause(game);
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_11_ContinuePause(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.x_pos = 59 as uint16;
        (*E).base.y_pos = 279 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
            as *mut uint16) = -(27 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
            as *mut uint16) = -(217 as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_04 = 7 as uint16;
        (*E).mbn_var_A = (0xa98dec as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_12_StartMusic(game: &mut Game) {
    MotherBrain_SetBodyInstrs(
        addr_kMotherBrain_Ilist_9A02 as ::core::ffi::c_int as uint16,
        game,
    );
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).base.instruction_timer = 1 as uint16;
    let mut E0: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E0).base.properties = ((*E0).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    (*E0).base.x_pos = 59 as uint16;
    (*E0).base.y_pos = 279 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
        as *mut uint16) = -(27 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
        as *mut uint16) = -(217 as ::core::ffi::c_int) as uint16;
    QueueMusic_Delayed8(5 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = 2 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) = 256 as uint16;
    (*E1).mbn_var_34 = 80 as uint16;
    (*E1).mbn_var_31 = 1 as uint16;
    (*E1).mbn_var_32 = 8 as uint16;
    (*E1).mbn_var_33 = 6 as uint16;
    (*E0).mbn_var_A = (0xa98e4d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_13_Raise(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 = 0;
    if !(*((&raw mut game.g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || {
            MotherBrain_SpawnDustCloudsForAscent(game);
            let ref mut fresh7 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xb7 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh7 = (*fresh7 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            v1 = ((*E).base.y_pos as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
            (*E).base.y_pos = v1;
            v1 as ::core::ffi::c_int >= 0xbd as ::core::ffi::c_int
        })
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x179a as ::core::ffi::c_int as isize)
            as *mut uint16) = 320 as uint16;
        (*E).base.y_pos = 188 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(((*E).mbn_var_09 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            0 as uint16;
        MotherBrain_SetBodyInstrs(
            addr_kMotherBrain_Ilist_99AA as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).mbn_var_A = (0xa98e95 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBrainBody_FakeDeath_Ascent_14_WaitForUncouching(game);
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_14_WaitForUncouching(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if (*E).mbn_var_02 == 0 {
        (*E).mbn_var_37 = 0 as uint16;
        (*E).mbn_var_A = (0xa98eaa as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 0 as uint16;
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_15_TransitionFromGrey(game: &mut Game) {
    let mut E0: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E0).mbn_var_F = (*E0).mbn_var_F.wrapping_sub(1);
    if (*E0).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E0).mbn_var_F = 4 as uint16;
        let mut v3: uint16 =
            ((*E0).mbn_var_37 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E0).mbn_var_37 = v3;
        if MotherBrain_FadeFromGray_FakeDeath(
            (v3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
            game,
        ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
            (*E1).mbn_var_10 = 1 as uint16;
            (*E0).mbn_var_00 = 2 as uint16;
            (*E1).mbn_var_12 = 1 as uint16;
            (*E1).mbn_var_32 = 6 as uint16;
            (*E1).mbn_var_33 = 6 as uint16;
            (*E1).mbn_var_34 = 1280 as uint16;
            (*E0).mbn_var_A =
                (0xa98ef5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E0).mbn_var_F = 23 as uint16;
        }
    }
}
pub unsafe fn MotherBrainBody_2ndphase_16_ShakeHeadMenacingly(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        MotherBrain_SetBrainInstrs(
            addr_kMotherBrain_Ilist_9B7F as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).mbn_var_A = (0xa98f14 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_34 = 64 as uint16;
        (*E).mbn_var_F = 256 as uint16;
        MotherBrainBody_2ndphase_17_BringHeadBackUp(game);
    }
}
pub unsafe fn MotherBrainBody_2ndphase_17_BringHeadBackUp(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_32 = 2 as uint16;
        (*E1).mbn_var_33 = 4 as uint16;
        (*E).mbn_var_A = (0xa98f33 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 64 as uint16;
        MotherBrainBody_2ndphase_18_FinishStretching(game);
    }
}
pub unsafe fn MotherBrainBody_2ndphase_18_FinishStretching(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_14 = 1 as uint16;
        (*E).mbn_var_A = (0xa9b605 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrain_SpawnDustCloudsForAscent(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_F = 7 as uint16;
    }
    game.eproj_spawn_pt = Point16U {
        x: g_word_A98F7F[(*E).mbn_var_F as usize],
        y: 212 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        g_byte_A98F7D[((*((&raw mut game.g_ram as *mut uint8)
            .offset(0x5e5 as ::core::ffi::c_int as isize) as *mut uint16)
            as ::core::ffi::c_int
            & 0x100 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as usize] as uint16,
        game,
    );
    QueueSfx2_Max3(0x29 as uint16, game);
}
pub unsafe fn MotherBrain_SetupNeckForFakeAscent(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E).mbn_var_24 = 2 as uint16;
    (*E).mbn_var_27 = 10 as uint16;
    (*E).mbn_var_2D = 10 as uint16;
    (*E).mbn_var_2A = 20 as uint16;
    (*E).mbn_var_30 = 20 as uint16;
    (*E).mbn_var_20 = 18432 as uint16;
    (*E).mbn_var_21 = 20480 as uint16;
    (*E).mbn_var_34 = 256 as uint16;
}
pub unsafe fn MotherBrain_HandleNeckLower(game: &mut Game) {
    let mut mbn_var_32: uint16 = (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_32;
    if mbn_var_32 != 0 {
        match mbn_var_32 as ::core::ffi::c_int {
            2 => {
                MotherBrain_HandleNeckLower_2_BobDown(game);
            }
            4 => {
                MotherBrain_HandleNeckLower_4_BobUp(game);
            }
            6 => {
                MotherBrain_HandleNeckLower_6_Lower(game);
            }
            8 => {
                MotherBrain_HandleNeckLower_8_Raise(game);
            }
            _ => {
                Unreachable(game);
                loop {}
            }
        }
    } else {
        MotherBrain_HandleNeckLower_0();
    };
}
pub unsafe fn MotherBrain_HandleNeckLower_0() {}
pub unsafe fn MotherBrain_HandleNeckLower_2_BobDown(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    v1 = ((*E).mbn_var_20 as ::core::ffi::c_int - (*E).mbn_var_34 as ::core::ffi::c_int) as int16;
    if (v1 as uint16 as ::core::ffi::c_int) < 0x2800 as ::core::ffi::c_int {
        (*E).mbn_var_32 = 4 as uint16;
        v1 = 10240 as int16;
    }
    (*E).mbn_var_20 = v1 as uint16;
}
pub unsafe fn MotherBrain_HandleNeckLower_4_BobUp(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if (*E).base.y_pos as ::core::ffi::c_int - 60 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).mbn_var_32 = 2 as uint16;
    } else {
        v1 = ((*E).mbn_var_34 as ::core::ffi::c_int + (*E).mbn_var_20 as ::core::ffi::c_int)
            as int16;
        if v1 as uint16 as ::core::ffi::c_int >= 0x9000 as ::core::ffi::c_int {
            (*E).mbn_var_32 = 2 as uint16;
            v1 = -(28672 as ::core::ffi::c_int) as int16;
        }
        (*E).mbn_var_20 = v1 as uint16;
    };
}
pub unsafe fn MotherBrain_HandleNeckLower_6_Lower(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    v1 = ((*E).mbn_var_20 as ::core::ffi::c_int - (*E).mbn_var_34 as ::core::ffi::c_int) as int16;
    if (v1 as uint16 as ::core::ffi::c_int) < 0x3000 as ::core::ffi::c_int {
        (*E).mbn_var_32 = 0 as uint16;
        v1 = 12288 as int16;
    }
    (*E).mbn_var_20 = v1 as uint16;
}
pub unsafe fn MotherBrain_HandleNeckLower_8_Raise(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    v1 = ((*E).mbn_var_34 as ::core::ffi::c_int + (*E).mbn_var_20 as ::core::ffi::c_int) as int16;
    if v1 as uint16 as ::core::ffi::c_int >= 0x9000 as ::core::ffi::c_int {
        (*E).mbn_var_32 = 0 as uint16;
        v1 = -(28672 as ::core::ffi::c_int) as int16;
    }
    (*E).mbn_var_20 = v1 as uint16;
}
pub unsafe fn MotherBrain_HandleNeckUpper(game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v6: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    let mut mbn_var_33: uint16 = (*E).mbn_var_33;
    if mbn_var_33 != 0 {
        match mbn_var_33 as ::core::ffi::c_int {
            2 => {
                if (((*E).base.y_pos as ::core::ffi::c_int + 4 as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xafa as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
                {
                    v2 = ((*E).mbn_var_21 as ::core::ffi::c_int
                        - (*E).mbn_var_34 as ::core::ffi::c_int) as int16;
                    if (v2 as uint16 as ::core::ffi::c_int) < 0x2000 as ::core::ffi::c_int {
                        (*E).mbn_var_33 = 4 as uint16;
                        v2 = 0x2000 as int16;
                    }
                    (*E).mbn_var_21 = v2 as uint16;
                } else {
                    (*E).mbn_var_32 = 4 as uint16;
                    (*E).mbn_var_33 = 4 as uint16;
                }
            }
            4 => {
                let mut r18: uint16 =
                    ((*E).mbn_var_20 as ::core::ffi::c_int + 2048 as ::core::ffi::c_int) as uint16;
                let mut v4: uint16 = ((*E).mbn_var_34 as ::core::ffi::c_int
                    + (*E).mbn_var_21 as ::core::ffi::c_int)
                    as uint16;
                if v4 as ::core::ffi::c_int >= r18 as ::core::ffi::c_int {
                    (*E).mbn_var_33 = 2 as uint16;
                    v4 = r18;
                }
                (*E).mbn_var_21 = v4;
            }
            6 => {
                v6 = ((*E).mbn_var_21 as ::core::ffi::c_int - (*E).mbn_var_34 as ::core::ffi::c_int)
                    as int16;
                if (v6 as uint16 as ::core::ffi::c_int) < 0x2000 as ::core::ffi::c_int {
                    (*E).mbn_var_33 = 0 as uint16;
                    v6 = 0x2000 as int16;
                }
                (*E).mbn_var_21 = v6 as uint16;
            }
            8 => {
                let mut r18_0: uint16 =
                    ((*E).mbn_var_20 as ::core::ffi::c_int + 2048 as ::core::ffi::c_int) as uint16;
                let mut v8: uint16 = ((*E).mbn_var_34 as ::core::ffi::c_int
                    + (*E).mbn_var_21 as ::core::ffi::c_int)
                    as uint16;
                if v8 as ::core::ffi::c_int >= r18_0 as ::core::ffi::c_int {
                    (*E).mbn_var_33 = 0 as uint16;
                    v8 = r18_0;
                }
                (*E).mbn_var_21 = v8;
            }
            _ => {
                Unreachable(game);
                loop {}
            }
        }
    }
}
pub unsafe fn MotherBrain_HandleNeck(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_0A = ((*E).base.x_pos as ::core::ffi::c_int - 80 as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_0B = ((*E).base.y_pos as ::core::ffi::c_int + 46 as ::core::ffi::c_int) as uint16;
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if (*E1).mbn_var_31 != 0 {
        MotherBrain_HandleNeckLower(game);
        MotherBrain_HandleNeckUpper(game);
    }
    let mut r18: uint16 = *(&raw mut (*E1).mbn_var_20 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as uint16;
    (*E1).mbn_var_22 = ((*E).mbn_var_0A as ::core::ffi::c_int
        + ComputeSinMult((*E1).mbn_var_24, r18, game) as ::core::ffi::c_int
        + 112 as ::core::ffi::c_int) as uint16;
    (*E1).mbn_var_23 = ((*E).mbn_var_0B as ::core::ffi::c_int
        + ComputeCosMult((*E1).mbn_var_24, r18, game) as ::core::ffi::c_int
        - 96 as ::core::ffi::c_int) as uint16;
    (*E1).mbn_var_25 = ((*E).mbn_var_0A as ::core::ffi::c_int
        + ComputeSinMult((*E1).mbn_var_27, r18, game) as ::core::ffi::c_int
        + 112 as ::core::ffi::c_int) as uint16;
    (*E1).mbn_var_26 = ((*E).mbn_var_0B as ::core::ffi::c_int
        + ComputeCosMult((*E1).mbn_var_27, r18, game) as ::core::ffi::c_int
        - 96 as ::core::ffi::c_int) as uint16;
    (*E1).mbn_var_28 = ((*E).mbn_var_0A as ::core::ffi::c_int
        + ComputeSinMult((*E1).mbn_var_2A, r18, game) as ::core::ffi::c_int
        + 112 as ::core::ffi::c_int) as uint16;
    (*E1).mbn_var_29 = ((*E).mbn_var_0B as ::core::ffi::c_int
        + ComputeCosMult((*E1).mbn_var_2A, r18, game) as ::core::ffi::c_int
        - 96 as ::core::ffi::c_int) as uint16;
    r18 = *(&raw mut (*E1).mbn_var_21 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as uint16;
    (*E1).mbn_var_2B = ((*E1).mbn_var_28 as ::core::ffi::c_int
        + ComputeSinMult((*E1).mbn_var_2D, r18, game) as ::core::ffi::c_int)
        as uint16;
    (*E1).mbn_var_2C = ((*E1).mbn_var_29 as ::core::ffi::c_int
        + ComputeCosMult((*E1).mbn_var_2D, r18, game) as ::core::ffi::c_int)
        as uint16;
    (*E1).mbn_var_2E = ((*E1).mbn_var_28 as ::core::ffi::c_int
        + ComputeSinMult((*E1).mbn_var_30, r18, game) as ::core::ffi::c_int)
        as uint16;
    (*E1).mbn_var_2F = ((*E1).mbn_var_29 as ::core::ffi::c_int
        + ComputeCosMult((*E1).mbn_var_30, r18, game) as ::core::ffi::c_int)
        as uint16;
}
pub unsafe fn CallMotherBrainInstr(mut ea: uint32, mut k: uint16, game: &mut Game) -> uint16 {
    match ea {
        11115279 => return MotherBrain_Instr_Goto(k, game),
        11115284 => return MotherBrain_Instr_EnableNeckMovementGoto(k, game),
        11115296 => return MotherBrain_Instr_DisableNeckMovement(k, game),
        11115304 => return MotherBrain_Instr_QueueSfx2(k, game),
        11115314 => return MotherBrain_Instr_QueueSfx3(k, game),
        11115324 => return MotherBrain_Instr_SpawnDroolEproj(k, game),
        11115373 => return MotherBrain_Instr_SpawnPurpleBreath(k, game),
        11115383 => return MotherBrain_Instr_SetMainShakeTimer50(k, game),
        11115621 => return MotherBrain_Instr_GotoEitherOr(k, game),
        11115693 => return MotherBrain_Instr_MaybeGoto(k, game),
        11115789 => return MotherBrain_Instr_MaybeGoto2(k, game),
        11115809 => return MotherBrain_Instr_Goto2(k),
        11116023 => return MotherBrain_Instr_QueueShitroidAttackSfx(k, game),
        11116073 => return MotherBrain_Instr_SpawnBlueRingEproj(k, game),
        11116087 => return MotherBrain_Instr_AimRingsAtShitroid(k, game),
        11116123 => return MotherBrain_Instr_AimRingsAtSamus(k, game),
        11116195 => return MotherBrain_Instr_IncrShitroidAttackCtr(k, game),
        11116213 => return MotherBrain_Instr_SetShitroidAttackCtr0(k, game),
        11116221 => return MotherBrain_Instr_SpawnBombEproj(k, game),
        11116358 => return MotherBrain_Instr_SpawnLaserEproj(k, game),
        11116420 => return MotherBrain_Instr_SpawnRainbowEproj(k, game),
        11116430 => return MotherBrain_Instr_SetupFxForRainbowBeam(k, game),
        _ => return Unreachable(game) as uint16,
    };
}
pub unsafe fn MotherBrain_Func_1_DoubleRet(game: &mut Game) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut v5: uint16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let mut pc: uint16 = (*E).mbn_var_21;
    if pc as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        return -(1 as ::core::ffi::c_int);
    }
    let mut p: *const uint8 = RomPtr_A9(pc as uint16_t, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        return *(p.offset(2 as ::core::ffi::c_int as isize) as *mut uint16) as ::core::ffi::c_int;
    }
    v5 = *(p as *mut uint16);
    if v5 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        current_block = 87389992155768869;
    } else {
        if (v5 as ::core::ffi::c_int - (*E).mbn_var_20 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            (*E).mbn_var_20 = (*E).mbn_var_20.wrapping_add(1);
            return *(p.offset(2 as ::core::ffi::c_int as isize) as *mut uint16)
                as ::core::ffi::c_int;
        }
        pc = (pc as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
        current_block = 1917311967535052937;
    }
    loop {
        match current_block {
            87389992155768869 => {
                pc = CallMotherBrainInstr(
                    (v5 as ::core::ffi::c_int | 0xa90000 as ::core::ffi::c_int) as uint32,
                    (pc as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
                    game,
                );
                current_block = 1917311967535052937;
            }
            _ => {
                p = RomPtr_A9(pc as uint16_t, game);
                v5 = *(p as *mut uint16);
                if v5 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
                    break;
                } else {
                    current_block = 87389992155768869;
                }
            }
        }
    }
    (*E).mbn_var_20 = 1 as uint16;
    (*E).mbn_var_21 = pc;
    return *(p.offset(2 as ::core::ffi::c_int as isize) as *mut uint16) as ::core::ffi::c_int;
}
pub unsafe fn MotherBrain_DrawNeck(game: &mut Game) {
    if (*Get_MotherBrain(0 as ::core::ffi::c_int, game))
        .base
        .properties as ::core::ffi::c_int
        & kEnemyProps_Invisible as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        MotherBrain_DrawNeckSegment((*E).mbn_var_2E, (*E).mbn_var_2F, game);
        MotherBrain_DrawNeckSegment((*E).mbn_var_2B, (*E).mbn_var_2C, game);
        MotherBrain_DrawNeckSegment((*E).mbn_var_28, (*E).mbn_var_29, game);
        MotherBrain_DrawNeckSegment((*E).mbn_var_25, (*E).mbn_var_26, game);
        MotherBrain_DrawNeckSegment((*E).mbn_var_22, (*E).mbn_var_23, game);
    }
}
pub unsafe fn MotherBrain_DrawBrain(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if (*E).mbn_var_14 as ::core::ffi::c_int != 0
        && (*E).mbn_var_15 == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainPurpleBreathSmall as ::core::ffi::c_int as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16),
            game,
        );
    }
    let mut tt: ::core::ffi::c_int = MotherBrain_Func_1_DoubleRet(game);
    if tt < 0 as ::core::ffi::c_int {
        return;
    }
    let mut v2: uint16 = tt as uint16;
    let mut mbn_var_0D: uint16 = (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_0D;
    if (*E).base.flash_timer as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        mbn_var_0D = 0 as uint16;
    }
    let mut r22: uint16 = mbn_var_0D;
    let mut mbn_var_00: uint16 = (*E).mbn_var_00;
    let mut flash_timer: uint16 = 0;
    if mbn_var_00 != 0 {
        flash_timer = (mbn_var_00 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_00 = flash_timer;
    } else {
        flash_timer = (*E).base.flash_timer;
        if flash_timer == 0 {
            flash_timer = (*E).base.shake_timer;
        }
    }
    let mut v6: ::core::ffi::c_int =
        (flash_timer as ::core::ffi::c_int & 6 as ::core::ffi::c_int) >> 1 as ::core::ffi::c_int;
    let mut r18: uint16 = ((*E).base.x_pos as ::core::ffi::c_int
        + g_word_A993BB[v6 as usize] as ::core::ffi::c_int) as uint16;
    if (r18 as ::core::ffi::c_int + 32 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        let mut r20: uint16 = ((*E).base.y_pos as ::core::ffi::c_int
            + g_word_A993C3[v6 as usize] as ::core::ffi::c_int)
            as uint16;
        MotherBrain_AddSpritemapToOam(v2, r18, r20, r22, game);
    }
}
pub unsafe fn MotherBrain_DrawNeckSegment(mut x: uint16, mut y: uint16, game: &mut Game) {
    let mut v0: ::core::ffi::c_int = ((*Get_MotherBrain(0x40 as ::core::ffi::c_int, game))
        .base
        .flash_timer as ::core::ffi::c_int
        & 6 as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int;
    MotherBrain_AddSpritemapToOam(
        addr_kMotherBrain_Sprmap_A694 as ::core::ffi::c_int as uint16,
        (x as ::core::ffi::c_int + g_word_A993BB[v0 as usize] as ::core::ffi::c_int) as uint16,
        (y as ::core::ffi::c_int + g_word_A993C3[v0 as usize] as ::core::ffi::c_int) as uint16,
        (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_0C,
        game,
    );
}
pub unsafe fn MotherBrain_AddSpritemapToOam(
    mut j: uint16,
    mut r18: uint16,
    mut r20: uint16,
    mut r22: uint16,
    game: &mut Game,
) {
    let mut p: *const uint8 = RomPtr_A9(j as uint16_t, game);
    let mut n: ::core::ffi::c_int = *(p as *mut uint16) as ::core::ffi::c_int;
    p = p.offset(2 as ::core::ffi::c_int as isize);
    let mut idx: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x590 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
    loop {
        let mut y: int16 = (r20 as ::core::ffi::c_int
            + *p.offset(2 as ::core::ffi::c_int as isize) as int8 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16;
        if y as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            let mut oam: *mut OamEnt = gOamEnt(idx, game);
            let mut x: uint16 = (r18 as ::core::ffi::c_int
                + *(p as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                as uint16;
            (*oam).xcoord = x as uint8;
            (*oam).ycoord = y as uint8;
            *(&raw mut (*oam).charnum as *mut uint16) = (r22 as ::core::ffi::c_int
                | *(p.offset(3 as ::core::ffi::c_int as isize) as *mut uint16)
                    as ::core::ffi::c_int)
                as uint16;
            let ref mut fresh6 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x570 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((idx >> 5 as ::core::ffi::c_int) as isize);
            *fresh6 = (*fresh6 as ::core::ffi::c_int
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
        p = p.offset(5 as ::core::ffi::c_int as isize);
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize)
        as *mut uint16) = idx as uint16;
}
pub unsafe fn MotherBrain_CalculateRainbowBeamHdma(game: &mut Game) {
    MotherBrain_CalcHdma(game);
}
pub unsafe fn MotherBrain_MoveBodyDownScrollLeft(mut k: uint16, mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int + a as ::core::ffi::c_int) as uint16;
    let ref mut fresh1 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xb7 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh1 = (*fresh1 as ::core::ffi::c_int - a as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
        as *mut uint16) = (k as ::core::ffi::c_int + 34 as ::core::ffi::c_int
        - (*E).base.x_pos as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MotherBrain_MoveBodyDown(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int + a as ::core::ffi::c_int) as uint16;
    let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xb7 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh0 = (*fresh0 as ::core::ffi::c_int - a as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
        as *mut uint16) =
        (34 as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MotherBrain_FootstepEffect(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) = 4 as uint16;
    if (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_00 as ::core::ffi::c_int
        == 3 as ::core::ffi::c_int
    {
        printf(b"Write to rom!\n\0" as *const u8 as *const ::core::ffi::c_char);
    }
}
pub unsafe fn MotherBrain_Instr_MoveBodyUp10Left4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    MotherBrain_MoveBodyDownScrollLeft(4 as uint16, 0xfff6 as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveBodyUp16Left4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    MotherBrain_MoveBodyDownScrollLeft(4 as uint16, 0xfff0 as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveBodyUp12Right2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    MotherBrain_MoveBodyDownScrollLeft(0xfffe as uint16, 0xfff4 as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveDown12Left4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    MotherBrain_MoveBodyDownScrollLeft(4 as uint16, 0xc as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveDown16Right2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    MotherBrain_MoveBodyDownScrollLeft(0xfffe as uint16, 0x10 as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveDown10Right2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    MotherBrain_MoveBodyDownScrollLeft(0xfffe as uint16, 0xa as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveUp2Right1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = (*E).base.x_pos.wrapping_add(1);
    MotherBrain_MoveBodyDown(0xfffe as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveRight2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    MotherBrain_MoveBodyDown(0 as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveUp1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    MotherBrain_MoveBodyDown(1 as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveUp1Right3_Sfx(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    MotherBrain_FootstepEffect(game);
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16;
    MotherBrain_MoveBodyDown(1 as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_Down2Right15(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int + 15 as ::core::ffi::c_int) as uint16;
    MotherBrain_MoveBodyDown(0xfffe as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_Down4Right6(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
    MotherBrain_MoveBodyDown(0xfffc as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_Up4Left2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    MotherBrain_MoveBodyDown(4 as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_Up2Left1_Sfx(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    MotherBrain_FootstepEffect(game);
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = (*E).base.x_pos.wrapping_sub(1);
    MotherBrain_MoveBodyDown(2 as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_Up2Left1_Sfx2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    MotherBrain_FootstepEffect(game);
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = (*E).base.x_pos.wrapping_sub(1);
    MotherBrain_MoveBodyDown(2 as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveLeft2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    MotherBrain_MoveBodyDown(0 as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveDown1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    MotherBrain_MoveBodyDown(0xffff as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveDown1Left3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int - 3 as ::core::ffi::c_int) as uint16;
    MotherBrain_MoveBodyDown(0xffff as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveUp2Left15_Sfx(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    MotherBrain_FootstepEffect(game);
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as uint16;
    MotherBrain_MoveBodyDown(2 as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveUp4Left6(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int - 6 as ::core::ffi::c_int) as uint16;
    MotherBrain_MoveBodyDown(4 as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveDown4Right2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    MotherBrain_MoveBodyDown(0xfffc as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveDown2Right1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = (*E).base.x_pos.wrapping_add(1);
    MotherBrain_MoveBodyDown(0xfffe as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_SetPose_Standing(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_02 = 0 as uint16;
    return jp;
}
pub unsafe fn MotherBrain_Instr_SetPose_Walking(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_02 = 1 as uint16;
    return jp;
}
pub unsafe fn MotherBrain_Instr_SetPose_Crouched(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_02 = 3 as uint16;
    return jp;
}
pub unsafe fn MotherBrain_Instr_SetPose_CrouchedTrans(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_02 = 2 as uint16;
    return jp;
}
pub unsafe fn MotherBrain_Instr_SetPose_DeathBeamMode(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_02 = 4 as uint16;
    return jp;
}
pub unsafe fn MotherBrain_Instr_SetPose_LeaningDown(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_02 = 6 as uint16;
    return jp;
}
pub unsafe fn MotherBrain_Instr_SpawnEprojToOffset(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    game.eproj_spawn_pt = Point16U {
        x: ((*E).base.x_pos as ::core::ffi::c_int
            + *jp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as uint16,
        y: ((*E).base.y_pos as ::core::ffi::c_int
            + *jp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        *jp.offset(2 as ::core::ffi::c_int as isize),
        game,
    );
    return jp.offset(3 as ::core::ffi::c_int as isize);
}
pub unsafe fn MotherBrain_Instr_SpawnDeathBeamEproj(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x63 as uint16, game);
    SpawnEprojWithGfx(
        0 as uint16,
        0x40 as uint16,
        addr_kEproj_MotherBrainDeathBeamCharging as ::core::ffi::c_int as uint16,
        game,
    );
    return jp;
}
pub unsafe fn MotherBrain_Instr_IncrBeamAttackPhase(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_17 = (*E).mbn_var_17.wrapping_add(1);
    return jp;
}
pub unsafe fn MotherBrain_Instr_Goto(mut k: uint16, game: &mut Game) -> uint16 {
    return *(RomPtr_A9(k as uint16_t, game) as *mut uint16);
}
pub unsafe fn MotherBrain_Instr_EnableNeckMovementGoto(mut k: uint16, game: &mut Game) -> uint16 {
    (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_31 = 1 as uint16;
    return *(RomPtr_A9(k as uint16_t, game) as *mut uint16);
}
pub unsafe fn MotherBrain_Instr_DisableNeckMovement(mut k: uint16, game: &mut Game) -> uint16 {
    (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_31 = 0 as uint16;
    return k;
}
pub unsafe fn MotherBrain_Instr_QueueSfx2(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v2: *const uint16 = RomPtr_A9(k as uint16_t, game) as *const uint16;
    QueueSfx2_Max6(*v2, game);
    return (k as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MotherBrain_Instr_QueueSfx3(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v2: *const uint16 = RomPtr_A9(k as uint16_t, game) as *const uint16;
    QueueSfx3_Max6(*v2, game);
    return (k as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MotherBrain_Instr_SpawnDroolEproj(mut k: uint16, game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if (*E).mbn_var_12 != 0 {
        let mut v2: uint16 =
            ((*E).mbn_var_13 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        if (*E).mbn_var_13 as ::core::ffi::c_int - 5 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v2 = 0 as uint16;
        }
        (*E).mbn_var_13 = v2;
        let mut v3: uint16 = (if (*E).mbn_var_34 as ::core::ffi::c_int - 128 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            addr_kEproj_MotherBrainDrool as ::core::ffi::c_int
        } else {
            addr_kEproj_MotherBrainDyingDrool as ::core::ffi::c_int
        }) as uint16;
        SpawnEprojWithRoomGfx(v3, (*E).mbn_var_13, game);
    }
    return k;
}
pub unsafe fn MotherBrain_Instr_SpawnPurpleBreath(mut k: uint16, game: &mut Game) -> uint16 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_MotherBrainPurpleBreathBig as ::core::ffi::c_int as uint16,
        0 as uint16,
        game,
    );
    return k;
}
pub unsafe fn MotherBrain_Instr_SetMainShakeTimer50(mut k: uint16, game: &mut Game) -> uint16 {
    (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_00 = 50 as uint16;
    return k;
}
pub unsafe fn MotherBrain_Instr_GotoEitherOr(mut k: uint16, game: &mut Game) -> uint16 {
    let mut result: uint16 = addr_stru_A99C5F as ::core::ffi::c_int as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0xfff as ::core::ffi::c_int
        >= 0xfe0 as ::core::ffi::c_int
    {
        return addr_stru_A99C47 as ::core::ffi::c_int as uint16;
    }
    return result;
}
pub unsafe fn MotherBrain_Instr_MaybeGoto(mut k: uint16, game: &mut Game) -> uint16 {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int)
        < 0xf000 as ::core::ffi::c_int
    {
        return addr_stru_A99C9F as ::core::ffi::c_int as uint16;
    }
    return k;
}
pub unsafe fn MotherBrain_Instr_MaybeGoto2(mut k: uint16, game: &mut Game) -> uint16 {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0xfff as ::core::ffi::c_int)
        < 0xec0 as ::core::ffi::c_int
    {
        return MotherBrain_Instr_Goto2(k);
    }
    return k;
}
pub unsafe fn MotherBrain_Instr_QueueShitroidAttackSfx(mut k: uint16, game: &mut Game) -> uint16 {
    if (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_13 as ::core::ffi::c_int
        != 11 as ::core::ffi::c_int
    {
        QueueSfx2_Max6(g_word_A99E0F[0 as ::core::ffi::c_int as usize], game);
    }
    return k;
}
pub unsafe fn MotherBrain_Instr_Goto2(mut k: uint16) -> uint16 {
    return addr_stru_A99CD1 as ::core::ffi::c_int as uint16;
}
pub unsafe fn MotherBrain_Instr_SpawnBlueRingEproj(mut k: uint16, game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    SpawnEprojWithRoomGfx(
        addr_kEproj_MotherBrainBlueRingLasers as ::core::ffi::c_int as uint16,
        (*E).mbn_var_1A,
        game,
    );
    return k;
}
pub unsafe fn MotherBrain_Instr_AimRingsAtShitroid(mut k: uint16, game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    let mut E1: *mut Enemy_MotherBrain =
        Get_MotherBrain((*E).mbn_var_0A as ::core::ffi::c_int, game);
    MotherBrain_Instr_AimRings(
        ((*E1).base.x_pos as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int
            - 10 as ::core::ffi::c_int) as uint16,
        ((*E1).base.y_pos as ::core::ffi::c_int
            - (*E).base.y_pos as ::core::ffi::c_int
            - 16 as ::core::ffi::c_int) as uint16,
        game,
    );
    return k;
}
pub unsafe fn MotherBrain_Instr_AimRingsAtSamus(mut k: uint16, game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    MotherBrain_Instr_AimRings(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int
            - 10 as ::core::ffi::c_int) as uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - (*E).base.y_pos as ::core::ffi::c_int
            - 16 as ::core::ffi::c_int) as uint16,
        game,
    );
    return k;
}
pub unsafe fn MotherBrain_Instr_AimRings(mut x: uint16, mut y: uint16, game: &mut Game) {
    let mut current_block: u64;
    let mut v0: uint16 = -(CalculateAngleFromXY(x, y, game) as ::core::ffi::c_int
        + 0x80 as ::core::ffi::c_int) as uint8 as uint16;
    if v0 as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 {
        if (v0 as uint8 as ::core::ffi::c_int) < 0x48 as ::core::ffi::c_int {
            current_block = 8016059118846638429;
        } else {
            current_block = 5817625388393324680;
        }
    } else if v0 as ::core::ffi::c_int + 64 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0
    {
        current_block = 5817625388393324680;
    } else {
        *(&raw mut v0 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) = 16 as uint8;
        current_block = 8016059118846638429;
    }
    match current_block {
        5817625388393324680 => {
            *(&raw mut v0 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) = 72 as uint8;
        }
        _ => {}
    }
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_1A = v0;
}
pub unsafe fn MotherBrain_Instr_IncrShitroidAttackCtr(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    v3 = ((*E).mbn_var_13 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as int16;
    if v3 as uint16 as ::core::ffi::c_int >= 0xc as ::core::ffi::c_int {
        v3 = 12 as int16;
    }
    (*E).mbn_var_13 = v3 as uint16;
    return k;
}
pub unsafe fn MotherBrain_Instr_SetShitroidAttackCtr0(mut k: uint16, game: &mut Game) -> uint16 {
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_13 = 0 as uint16;
    return k;
}
pub unsafe fn MotherBrain_Instr_SpawnBombEproj(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v2: *const uint16 = RomPtr_A9(k as uint16_t, game) as *const uint16;
    SpawnEprojWithRoomGfx(
        addr_kEproj_MotherBrainBomb as ::core::ffi::c_int as uint16,
        *v2,
        game,
    );
    return (k as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MotherBrain_Instr_SpawnLaserEproj(mut k: uint16, game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E).mbn_var_31 = 0 as uint16;
    game.eproj_spawn_r22 = 1 as uint16;
    game.eproj_spawn_pt = Point16U {
        x: ((*E).base.x_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16,
        y: ((*E).base.y_pos as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_stru_86A17B as ::core::ffi::c_int as uint16,
        1 as uint16,
        game,
    );
    return k;
}
pub unsafe fn MotherBrain_Instr_SpawnRainbowEproj(mut k: uint16, game: &mut Game) -> uint16 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_MotherBrainRainbowBeamCharging as ::core::ffi::c_int as uint16,
        0 as uint16,
        game,
    );
    return k;
}
pub unsafe fn MotherBrain_Instr_SetupFxForRainbowBeam(mut k: uint16, game: &mut Game) -> uint16 {
    (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_14 = 0 as uint16;
    MotherBrain_SetupBrainPalForLaser(game);
    QueueSfx2_Max6(0x7f as uint16, game);
    return k;
}
pub unsafe fn MotherBrain_Phase3_Death_0(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).base.properties =
        ((*E1).base.properties as ::core::ffi::c_int | 0x400 as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_04 = 0 as uint16;
    if MotherBrain_MakeWalkBackwards(0x28 as uint16, 6 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*E).mbn_var_A = (0xa9af12 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 128 as uint16;
        MotherBrain_Phase3_Death_1(game);
    }
}
pub unsafe fn MotherBrain_Phase3_Death_1(game: &mut Game) {
    MotherBrain_GenerateSmokyExplosions(game);
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_A = (0xa9af21 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrain_Phase3_Death_2(game: &mut Game) {
    MotherBrain_GenerateSmokyExplosions(game);
    if MotherBrain_MakeWalkForwards(2 as uint16, 0x60 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        MotherBrain_SetBrainInstrs(
            addr_kMotherBrain_Ilist_9C39 as ::core::ffi::c_int as uint16,
            game,
        );
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_32 = 6 as uint16;
        (*E1).mbn_var_33 = 6 as uint16;
        (*E1).mbn_var_34 = 1280 as uint16;
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_A = (0xa9af54 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 32 as uint16;
    }
}
pub unsafe fn MotherBrain_Phase3_Death_3(game: &mut Game) {
    MotherBrain_GenerateSmokyExplosions(game);
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_32 = 0 as uint16;
        (*E1).mbn_var_33 = 0 as uint16;
        (*E1).mbn_var_12 = 0 as uint16;
        (*E1).mbn_var_14 = 0 as uint16;
        (*E1).mbn_var_10 = 0 as uint16;
        (*E1).mbn_var_11 = 0 as uint16;
        let mut i: ::core::ffi::c_int = 28 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 241 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(((i >> 1 as ::core::ffi::c_int) + 145 as ::core::ffi::c_int) as isize);
            i -= 2 as ::core::ffi::c_int;
        }
        MotherBrain_HealthBasedPaletteHandling(game);
        (*E).mbn_var_0D = 3584 as uint16;
        (*E1).mbn_var_E = 0 as uint16;
        (*E1).mbn_var_F = 0 as uint16;
        (*E).mbn_var_A = (0xa9af9d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBrain_Phase3_Death_4(game);
    }
}
pub unsafe fn MotherBrain_Phase3_Death_4(game: &mut Game) {
    MotherBrain_GenerateMixedExplosions(game);
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_37 = 0 as uint16;
        (*E).mbn_var_A = (0xa9afb6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 0 as uint16;
        MotherBrain_Phase3_Death_5(game);
    }
}
pub unsafe fn MotherBrain_Phase3_Death_5(game: &mut Game) {
    HandleMotherBrainBodyFlickering(game);
    MotherBrain_GenerateMixedExplosions(game);
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_F = 16 as uint16;
        let mut v2: uint16 =
            ((*E).mbn_var_37 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_37 = v2;
        if MotherBrain_FadePalToBlack(
            (v2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
            game,
        ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x179a as ::core::ffi::c_int as isize)
                as *mut uint16) = 710 as uint16;
            let mut i: ::core::ffi::c_int = 710 as ::core::ffi::c_int;
            while i >= 0 as ::core::ffi::c_int {
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((i >> 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
                i -= 2 as ::core::ffi::c_int;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xe1e as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & 0xdeff as ::core::ffi::c_int
                | 0x100 as ::core::ffi::c_int) as uint16;
            (*E).base.extra_properties = 0 as uint16;
            (*E).mbn_var_A =
                (0xa9b013 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).mbn_var_F = 16 as uint16;
        }
    }
}
pub unsafe fn MotherBrain_Phase3_Death_6(game: &mut Game) {
    MotherBrain_GenerateMixedExplosions(game);
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_A = (0xa9b115 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrain_GenerateSmokyExplosions(game: &mut Game) {
    MotherBrain_GenerateExplosions(
        0x10 as uint16,
        addr_word_A9B10F as ::core::ffi::c_int as uint16,
        2 as uint16,
        game,
    );
}
pub unsafe fn MotherBrain_GenerateMixedExplosions(game: &mut Game) {
    MotherBrain_GenerateExplosions(
        8 as uint16,
        addr_word_A9B109 as ::core::ffi::c_int as uint16,
        4 as uint16,
        game,
    );
}
pub unsafe fn MotherBrain_GenerateExplosions(
    mut a: uint16,
    mut r22: uint16,
    mut r24: uint16,
    game: &mut Game,
) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E).mbn_var_E = (*E).mbn_var_E.wrapping_sub(1);
    let mut v2: bool = (*E).mbn_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 {
        (*E).mbn_var_E = a;
        (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
        v2 = (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
        if v2 {
            (*E).mbn_var_F = 6 as uint16;
        }
        let mut v3: uint16 =
            (16 as ::core::ffi::c_int * (*E).mbn_var_F as ::core::ffi::c_int) as uint16;
        let mut v4: uint16 = r24;
        let mut v9: uint16 = 0;
        loop {
            v9 = v4;
            let mut v5: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            game.eproj_spawn_pt = Point16U {
                x: *(game.g_rom.offset(
                    (((0xa9b099 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa9b099 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v5 as isize),
                y: *(game.g_rom.offset(
                    (((0xa9b099 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa9b099 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v5 + 1 as ::core::ffi::c_int) as isize),
            };
            let mut v6: *const uint16 = RomPtr_A9(r22 as uint16_t, game) as *const uint16;
            let mut v7: uint16 = *v6;
            let mut Random: uint16 = NextRandom(game);
            if Random as ::core::ffi::c_int >= 0x4000 as ::core::ffi::c_int {
                v7 = *v6.offset(1 as ::core::ffi::c_int as isize);
                if Random as ::core::ffi::c_int >= 0xe000 as ::core::ffi::c_int {
                    v7 = *v6.offset(2 as ::core::ffi::c_int as isize);
                }
            }
            SpawnEprojWithRoomGfx(
                addr_kEproj_MotherBrainDeathExplosion as ::core::ffi::c_int as uint16,
                v7,
                game,
            );
            v3 = (v3 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
            v4 = (v9 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
            if !(v9 as ::core::ffi::c_int != 1 as ::core::ffi::c_int) {
                break;
            }
        }
        QueueSfx3_Max3(0x13 as uint16, game);
    }
}
pub unsafe fn MotherBrain_Phase3_Death_7(game: &mut Game) {
    MotherBrain_SetBrainInstrs(
        addr_kMotherBrain_Ilist_9C29 as ::core::ffi::c_int as uint16,
        game,
    );
    (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_A =
        (0xa987d0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = 0 as uint16;
    (*E).mbn_var_A = (0xa9b12d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    MotherBrain_Phase3_Death_8(game);
}
pub unsafe fn MotherBrain_Phase3_Death_8(game: &mut Game) {
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 =
        ((*E).mbn_var_F as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_F = v1;
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    v3 = ((*E1).base.y_pos as ::core::ffi::c_int
        + *(&raw mut v1 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int) as int16;
    if v3 as uint16 as ::core::ffi::c_int >= 0xc4 as ::core::ffi::c_int {
        EnableEarthquakeAframes(2 as uint16, game);
        (*E).mbn_var_A = (0xa9b15e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 256 as uint16;
        v3 = 196 as int16;
    }
    (*E1).base.y_pos = v3 as uint16;
}
pub unsafe fn MotherBrain_Phase3_Death_9(game: &mut Game) {
    if ProcessSpriteTilesTransfers(
        0xa9 as uint8,
        addr_stru_A99003 as ::core::ffi::c_int as uint16,
        game,
    ) != 0
    {
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_A = (0xa9b173 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 32 as uint16;
    }
}
pub unsafe fn MotherBrain_Phase3_Death_10(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_37 = 0 as uint16;
        (*E).mbn_var_A = (0xa9b189 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 0 as uint16;
    }
}
pub unsafe fn MotherBrain_Phase3_Death_11(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut v2: uint16 =
            ((*E).mbn_var_37 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_37 = v2;
        if MotherBrain_FadeToGray_RealDeath(
            (v2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
            game,
        ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            MotherBrain_SetBrainInstrs(
                addr_kMotherBrain_Ilist_9D25 as ::core::ffi::c_int as uint16,
                game,
            );
            (*E).mbn_var_A =
                (0xa9b1b8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).mbn_var_F = 256 as uint16;
        } else {
            (*E).mbn_var_F = 16 as uint16;
        }
    }
}
pub unsafe fn MotherBrain_Phase3_Death_12(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_A = (0xa9b1d5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).base.properties =
            ((*E1).base.properties as ::core::ffi::c_int | 0x400 as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_04 = 0 as uint16;
    }
}
pub unsafe fn MotherBrain_Phase3_Death_13(game: &mut Game) {
    if ProcessCorpseRotting(0x40 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0
    {
        let mut dms_var_53: uint16 =
            (*Get_DeadMonsters(0x40 as ::core::ffi::c_int, game)).dms_var_53;
        ProcessCorpseRottingVramTransfers(dms_var_53, game);
    } else {
        let mut E1: *mut Enemy_DeadMonsters = Get_DeadMonsters(0x40 as ::core::ffi::c_int, game);
        (*E1).base.properties = ((*E1).base.properties as ::core::ffi::c_int
            & 0xdeff as ::core::ffi::c_int
            | 0x100 as ::core::ffi::c_int) as uint16;
        (*E1).base.extra_properties = 0 as uint16;
        QueueMusic_Delayed8(0 as uint16, game);
        QueueMusic_Delayed8(0xff24 as uint16, game);
        E1 = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
        (*E1).dms_var_A = (0xa9b211 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E1).dms_var_F = 20 as uint16;
        MotherBrain_Phase3_Death_14_20framedelay(game);
    };
}
pub unsafe fn MotherBrain_Phase3_Death_14_20framedelay(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).base.x_pos = 0 as uint16;
        (*E1).base.y_pos = 0 as uint16;
        (*E).mbn_var_A = (0xa9b258 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrain_CorpseRottingFinished(game: &mut Game) {
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    game.eproj_spawn_pt = Point16U {
        x: ((*E1).base.x_pos as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x1f as ::core::ffi::c_int)
            - 16 as ::core::ffi::c_int) as uint16,
        y: ((*E1).base.y_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        0xa as uint16,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe44 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        QueueSfx2_Max3(0x10 as uint16, game);
    }
}
pub unsafe fn MotherBrain_Phase3_Death_15_LoadEscapeTimerTiles(game: &mut Game) {
    if ProcessSpriteTilesTransfers(
        0xa6 as uint8,
        addr_stru_A6C4CB as ::core::ffi::c_int as uint16,
        game,
    ) != 0
    {
        (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
            (0xa9b26d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBrain_Phase3_Death_16_StartEscape(game);
    }
}
pub unsafe fn MotherBrain_Phase3_Death_16_StartEscape(game: &mut Game) {
    if ProcessSpriteTilesTransfers(
        0xa9 as uint8,
        addr_stru_A9902F as ::core::ffi::c_int as uint16,
        game,
    ) != 0
    {
        WriteColorsToPalette(
            0x122 as uint16,
            0xa9 as uint8,
            (addr_kMotherBrainPalette_5 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
            0xe as uint16,
            game,
        );
        QueueMusic_Delayed8(7 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
            as *mut uint16) = 5 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
            as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
        SpawnPalfxObject(addr_kPalfx_FFC9 as ::core::ffi::c_int as uint16, game);
        SpawnPalfxObject(addr_kPalfx_FFCD as ::core::ffi::c_int as uint16, game);
        SpawnPalfxObject(addr_kPalfx_FFD1 as ::core::ffi::c_int as uint16, game);
        SpawnPalfxObject(addr_kPalfx_FFD5 as ::core::ffi::c_int as uint16, game);
        (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_02 = 0 as uint16;
        SetupZebesEscapeTypewriter(game);
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_F = 32 as uint16;
        let mut v1: uint16 =
            (0xa9b2d1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            v1 = (0xa9b2e3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        (*E).mbn_var_A = v1;
    }
}
pub unsafe fn MotherBrain_Phase3_Death_17_SpawnTimeBomb(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_A = (0xa9b2e3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        SpawnEprojWithRoomGfx(
            addr_kEproj_TimeBombSetJapaneseText as ::core::ffi::c_int as uint16,
            (0xa9b2e3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
            game,
        );
    }
    MotherBrain_Phase3_Death_18_TypesZebesText(game);
}
pub unsafe fn MotherBrain_Phase3_Death_18_TypesZebesText(game: &mut Game) {
    if HandleTypewriterText_Ext(0x2610 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_A = (0xa9b2f9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 32 as uint16;
    }
}
pub unsafe fn MotherBrain_Phase3_Death_19_EscapeDoorExploding(game: &mut Game) {
    MotherBrain_GenerateEscapeDoorExploding(game);
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        CallSomeSamusCode(0xf as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x943 as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
        SetBossBitForCurArea(2 as uint16, game);
        SetEventHappened(0xe as uint16, game);
        (*E).mbn_var_A = (0xa9b32a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_E = 0 as uint16;
        (*E1).mbn_var_F = 0 as uint16;
    }
}
pub unsafe fn MotherBrain_Phase3_Death_20_BlowUpEscapeDoor(game: &mut Game) {
    MotherBrain_ExplodeEscapeDoor(game);
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
        (0xa9b33c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0 as uint8,
            field_1: 0x6 as uint8,
            field_2: 0xb677 as VoidP,
        },
        game,
    );
}
pub unsafe fn MotherBrain_Phase3_Death_21_KeepEarthquakeGoing(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
    {
        let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1840 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh10 = (*fresh10).wrapping_sub(1);
    }
}
pub unsafe fn MotherBrain_GenerateEscapeDoorExploding(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    v1 = ((*E).mbn_var_E as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
    (*E).mbn_var_E = v1 as uint16;
    if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).mbn_var_E = 4 as uint16;
        v2 = ((*E).mbn_var_F as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
        (*E).mbn_var_F = v2 as uint16;
        if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            (*E).mbn_var_F = 3 as uint16;
        }
        let mut v3: ::core::ffi::c_int = (4 as ::core::ffi::c_int
            * (*E).mbn_var_F as ::core::ffi::c_int)
            as uint16 as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        game.eproj_spawn_pt = Point16U {
            x: g_word_A9B393[v3 as usize],
            y: g_word_A9B393[(v3 + 1 as ::core::ffi::c_int) as usize],
        };
        let mut v4: uint16 = 3 as uint16;
        if (NextRandom(game) as ::core::ffi::c_int) < 0x4000 as ::core::ffi::c_int {
            v4 = 12 as uint16;
        }
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
            v4,
            game,
        );
        QueueSfx2_Max3(0x24 as uint16, game);
    }
}
pub unsafe fn MotherBrain_ExplodeEscapeDoor(game: &mut Game) {
    let mut v0: uint16 = 0 as uint16;
    let mut v1: uint16 = 0;
    loop {
        v1 = v0;
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainExplodedEscapeDoor as ::core::ffi::c_int as uint16,
            v0,
            game,
        );
        v0 = (v1 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        if !(((v1 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int)
            < 8 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
pub unsafe fn MotherBrain_SamusCollDetect(game: &mut Game) {
    let mut v1: int8 = 0;
    let mut E0: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let mut R26: uint16 = (*E0).mbn_var_04;
    v1 = (R26 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as int8;
    R26 = (R26 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    if v1 == 0
        || MotherBrain_SamusCollDetectPart(
            addr_word_A9B427 as ::core::ffi::c_int as uint16,
            (*E0).base.x_pos,
            (*E0).base.y_pos,
            game,
        ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0
    {
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        v1 = (R26 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as int8;
        R26 = (R26 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        if v1 == 0
            || MotherBrain_SamusCollDetectPart(
                0xb439 as uint16,
                (*E).base.x_pos,
                (*E).base.y_pos,
                game,
            ) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                == 0
        {
            v1 = (R26 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as int8;
            R26 = (R26 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
            if v1 != 0 {
                if MotherBrain_SamusCollDetectPart(
                    0xb44b as uint16,
                    (*E).mbn_var_25,
                    (*E).mbn_var_26,
                    game,
                ) as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                    == 0
                {
                    if MotherBrain_SamusCollDetectPart(
                        0xb44b as uint16,
                        (*E).mbn_var_28,
                        (*E).mbn_var_29,
                        game,
                    ) as ::core::ffi::c_int
                        & 1 as ::core::ffi::c_int
                        == 0
                    {
                        MotherBrain_SamusCollDetectPart(
                            0xb44b as uint16,
                            (*E).mbn_var_2B,
                            (*E).mbn_var_2C,
                            game,
                        );
                    }
                }
            }
        }
    }
}
pub unsafe fn MotherBrain_SamusCollDetectPart(
    mut k: uint16,
    mut r18: uint16,
    mut r20: uint16,
    game: &mut Game,
) -> uint8 {
    let mut v5: int16 = 0;
    let mut p: *const uint8 = RomPtr_A9(k as uint16_t, game);
    let mut n: ::core::ffi::c_int = *(p as *mut uint16) as ::core::ffi::c_int;
    p = p.offset(2 as ::core::ffi::c_int as isize);
    while n != 0 {
        let mut v3: uint16 = 0;
        let mut v4: uint16 = 0;
        let mut R24: uint16 = 0;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - r20 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            R24 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16)
                as ::core::ffi::c_int
                - r20 as ::core::ffi::c_int) as uint16;
            v3 = *(p.offset(6 as ::core::ffi::c_int as isize) as *mut uint16);
        } else {
            R24 = (r20 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
            v3 = *(p.offset(2 as ::core::ffi::c_int as isize) as *mut uint16);
        }
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + abs16(v3) as ::core::ffi::c_int
            - R24 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            if (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - r18 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                R24 = (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - r18 as ::core::ffi::c_int) as uint16;
                v4 = *(p.offset(4 as ::core::ffi::c_int as isize) as *mut uint16);
            } else {
                R24 = (r18 as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xaf6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16;
                v4 = *(p as *mut uint16);
            }
            v5 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafe as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + abs16(v4) as ::core::ffi::c_int
                - R24 as ::core::ffi::c_int) as int16;
            if v5 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                if v5 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    v5 = 4 as int16;
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0xb58 as ::core::ffi::c_int as isize)
                    as *mut uint16) = v5 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5c as ::core::ffi::c_int as isize)
                    as *mut uint16) = 4 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb56 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5a as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x18a8 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 96 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x18aa as ::core::ffi::c_int as isize)
                    as *mut uint16) = 5 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xa54 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 1 as uint16;
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 192 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xb36 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 2 as uint16;
                }
                if (((*Get_MotherBrain(0 as ::core::ffi::c_int, game)).base.x_pos
                    as ::core::ffi::c_int
                    + 24 as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xaf6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
                {
                    MotherBrain_HurtSamus(game);
                }
                return 1 as uint8;
            }
        }
        p = p.offset(8 as ::core::ffi::c_int as isize);
        n -= 1;
    }
    return 0 as uint8;
}
pub unsafe fn MotherBrainsBody_Shot(game: &mut Game) {
    CreateDudShot(game);
}
pub unsafe fn MotherBrainsBrain_Shot(game: &mut Game) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if (*E).mbn_var_00 != 0 {
        MotherBrain_Phase23_ShotReaction(game);
        if (*E).mbn_var_00 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            CreateDudShot(game);
        } else {
            NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
        }
    } else {
        let mut v1: uint16 = (*(((&raw mut game.g_ram as *mut uint8)
            .offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x18a6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int) as uint16;
        if g_byte_A9B546[v1 as usize] != 0 {
            let ref mut fresh11 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x1dc7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(39 as ::core::ffi::c_int as isize);
            *fresh11 = (*fresh11 as ::core::ffi::c_int
                + g_byte_A9B546[v1 as usize] as ::core::ffi::c_int)
                as uint16;
            QueueSfx2_Max6(0x6e as uint16, game);
            v2 = 13 as int16;
            let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
            let mut flash_timer: uint16 = (*E1).base.flash_timer;
            if flash_timer != 0 {
                if flash_timer as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
                    v2 = 14 as int16;
                }
            }
            (*E1).base.flash_timer = v2 as uint16;
            NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
        }
    };
}
pub unsafe fn MotherBrain_Phase23_ShotReaction(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if (*E).mbn_var_00 as ::core::ffi::c_int == 4 as ::core::ffi::c_int
        && MotherBrain_DetermineShotReactionType(game) as ::core::ffi::c_int
            == 2 as ::core::ffi::c_int
    {
        MotherBrain_Phase3_BeamShotReaction(game);
    } else {
        if MotherBrain_DetermineShotReactionType(game) as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
            || {
                v1 = ((*E).mbn_var_07 as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as int16;
                (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int
            }
        {
            v1 = 0 as int16;
        }
        (*E).mbn_var_07 = v1 as uint16;
    };
}
pub unsafe fn MotherBrain_DetermineShotReactionType(game: &mut Game) -> uint16 {
    return g_byte_A9B5A1[(*(((&raw mut game.g_ram as *mut uint8)
        .offset(0xc18 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x18a6 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int) as usize] as uint16;
}
pub unsafe fn MotherBrain_Phase3_BeamShotReaction(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    v1 = ((*E).mbn_var_07 as ::core::ffi::c_int - 266 as ::core::ffi::c_int) as int16;
    if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_18 =
            (0xa9c395 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        v1 = 0 as int16;
        (*E).mbn_var_F = 0 as uint16;
    }
    (*E).mbn_var_07 = v1 as uint16;
}
pub unsafe fn MotherBrainsBrain_Touch(game: &mut Game) {
    let mut v0: int16 = 0;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 3 as ::core::ffi::c_int
    {
        v0 = 13 as int16;
        let mut v1: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
        let mut flash_timer: uint16 = (*v1.offset(1 as ::core::ffi::c_int as isize)).flash_timer;
        if flash_timer != 0 {
            if flash_timer as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
                v0 = 14 as int16;
            }
        }
        (*v1.offset(1 as ::core::ffi::c_int as isize)).flash_timer = v0 as uint16;
    }
}
pub unsafe fn MotherBrain_HurtSamus(game: &mut Game) {
    Ridley_Func_98(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x18a8 as ::core::ffi::c_int as isize)
        as *mut uint16) = 96 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x18aa as ::core::ffi::c_int as isize)
        as *mut uint16) = 5 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa54 as ::core::ffi::c_int as isize)
        as *mut uint16) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - (*Get_MotherBrain(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
            game,
        ))
        .base
        .x_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16;
}
pub unsafe fn MotherBrain_Body_Phase2_Thinking(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if (*E1).base.health == 0 {
        (*E).mbn_var_A = (0xa9b8eb as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBomb_FiringRainbowBeam_0(game);
        return;
    }
    if (*E).mbn_var_02 == 0 {
        let mut current_block_9: u64;
        if (*E1).base.health as ::core::ffi::c_int - 4500 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >= 0x1000 as ::core::ffi::c_int
            {
                current_block_9 = 3601231341105262975;
            } else {
                current_block_9 = 13880174989427944637;
            }
        } else if (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x5e5 as ::core::ffi::c_int as isize) as *mut uint16)
            as ::core::ffi::c_int)
            < 0x2000 as ::core::ffi::c_int
        {
            current_block_9 = 3601231341105262975;
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >= 0xa000 as ::core::ffi::c_int
        {
            current_block_9 = 13880174989427944637;
        } else {
            (*E).mbn_var_A =
                (0xa9b87d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            return;
        }
        match current_block_9 {
            3601231341105262975 => {
                MotherBrain_HandleWalking(game);
                return;
            }
            _ => {
                (*E).mbn_var_A =
                    (0xa9b64b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                return;
            }
        }
    }
}
pub unsafe fn MotherBrain_Body_Phase2_TryAttack(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    v0 = (2 as ::core::ffi::c_int * (*E).mbn_var_18 as ::core::ffi::c_int) as int16;
    if v0 != 0 {
        if v0 as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
            MotherBrain_Phase2_Attack_Cooldown(game);
        } else {
            if v0 as ::core::ffi::c_int != 4 as ::core::ffi::c_int {
                Unreachable(game);
                loop {}
            }
            MotherBrain_Phase2_Attack_End(game);
        }
    } else {
        (*E).mbn_parameter_1 = 64 as uint16;
        (*E).mbn_var_18 = (*E).mbn_var_18.wrapping_add(1);
        if MotherBrain_Phase2_DecideAttackStrategy_DoubleRet(game) != 0 {
            return;
        }
        let mut v2: uint16 = addr_byte_A9B6DC as ::core::ffi::c_int as uint16;
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        if (abs16(
            ((*E1).base.y_pos as ::core::ffi::c_int + 4 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int)
            < 0x20 as ::core::ffi::c_int
        {
            v2 = addr_byte_A9B6DF as ::core::ffi::c_int as uint16;
        }
        let mut v4: uint16 = 0 as uint16;
        let mut v5: *const uint8 = RomPtr_A9(v2 as uint16_t, game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8 as ::core::ffi::c_int
            >= *v5 as ::core::ffi::c_int
        {
            v4 = 2 as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8 as ::core::ffi::c_int
                >= *v5.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            {
                v4 = 4 as uint16;
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x5e5 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint8 as ::core::ffi::c_int
                    >= *v5.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                {
                    v4 = 6 as uint16;
                }
            }
        }
        let mut v6: uint16 = *(game.g_rom.offset(
            (((0xa9b6d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa9b6d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        if v6 as ::core::ffi::c_int == addr_kMotherBrain_Ilist_9ECC as ::core::ffi::c_int {
            if (*E1).mbn_var_05 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                (*E).mbn_var_A =
                    (0xa9b781 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                MotherBrain_FiringBomb_DecideOnWalking(game);
            }
        } else if v6 as ::core::ffi::c_int == addr_kMotherBrain_Ilist_9F34 as ::core::ffi::c_int {
            (*E).mbn_var_A =
                (0xa9b80e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            MotherBomb_FiringLaser_PositionHead(game);
        } else {
            MotherBrain_SetBrainInstrs(v6, game);
        }
    };
}
pub unsafe fn MotherBrain_Phase2_DecideAttackStrategy_DoubleRet(game: &mut Game) -> uint8 {
    if g_word_A9B72C
        [*(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize) as usize]
        != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8 as ::core::ffi::c_int
            - 128 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            return 0 as uint8;
        } else {
            let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
            if (*E1).mbn_var_05 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
                    (0xa9b781 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                MotherBrain_FiringBomb_DecideOnWalking(game);
                return 1 as uint8;
            }
            return 0 as uint8;
        }
    } else {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8 as ::core::ffi::c_int
            - 128 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
                (0xa9b80e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            MotherBomb_FiringLaser_PositionHead(game);
        } else {
            MotherBrain_SetBrainInstrs(
                addr_kMotherBrain_Ilist_9D3D as ::core::ffi::c_int as uint16,
                game,
            );
        }
        return 1 as uint8;
    };
}
pub unsafe fn MotherBrain_Phase2_Attack_Cooldown(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let fresh9 = (*E).mbn_parameter_1;
    (*E).mbn_parameter_1 = (*E).mbn_parameter_1.wrapping_sub(1);
    if fresh9 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).mbn_var_18 = (*E).mbn_var_18.wrapping_add(1);
    }
}
pub unsafe fn MotherBrain_Phase2_Attack_End(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_18 = 0 as uint16;
    (*E).mbn_var_A = (0xa9b605 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MotherBrain_FiringBomb_DecideOnWalking(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >= 0xff80 as ::core::ffi::c_int
    {
        MotherBrain_FiringBomb_DecideOnCrouching(game);
        return;
    }
    let mut v0: uint16 = 64 as uint16;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int)
        < 0x6000 as ::core::ffi::c_int
    {
        v0 = 96 as uint16;
    }
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if (v0 as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
        || {
            (*E).mbn_var_F = v0;
            MotherBrain_MakeWalkBackwards(v0, 6 as uint16, game) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                != 0
        }
    {
        MotherBrain_FiringBomb_DecideOnCrouching(game);
    } else {
        (*E).mbn_var_A = (0xa9b7ac as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn MotherBrain_FiringBomb_WalkingBackwards(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if MotherBrain_MakeWalkBackwards((*E).mbn_var_F, 6 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        MotherBrain_FiringBomb_DecideOnCrouching(game);
    }
}
pub unsafe fn MotherBrain_FiringBomb_DecideOnCrouching(game: &mut Game) {
    if NextRandom(game) as ::core::ffi::c_int >= 0x8000 as ::core::ffi::c_int {
        (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
            (0xa9b7c6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBrain_FiringBomb_Crouch(game);
    } else {
        MotherBrain_B7CB(game);
    };
}
pub unsafe fn MotherBrain_FiringBomb_Crouch(game: &mut Game) {
    if MotherBrain_MakeHerCrouch(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        MotherBrain_B7CB(game);
    }
}
pub unsafe fn MotherBrain_B7CB(game: &mut Game) {
    let mut v0: uint16 = addr_kMotherBrain_Ilist_9ECC as ::core::ffi::c_int as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if (*E).mbn_var_1F != 0 {
        v0 = addr_kMotherBrain_Ilist_9F00 as ::core::ffi::c_int as uint16;
    }
    MotherBrain_SetBrainInstrs(v0, game);
    (*E).mbn_var_A = (0xa9b7e8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_F = 44 as uint16;
}
pub unsafe fn MotherBrain_FiringBomb_Fired(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if MotherBrain_MakeHerStandUp(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
            MotherBrain_FiringBomb_Finish(game);
        } else {
            (*E).mbn_var_A =
                (0xa9b7f8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            MotherBrain_FiringBomb_Standup(game);
        }
    }
}
pub unsafe fn MotherBrain_FiringBomb_Standup(game: &mut Game) {
    if MotherBrain_MakeHerStandUp(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        MotherBrain_FiringBomb_Finish(game);
    }
}
pub unsafe fn MotherBrain_FiringBomb_Finish(game: &mut Game) {
    let mut v0: uint16 = (0xa9b605 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if (*E).mbn_var_1F != 0 {
        v0 = (0xa9c15c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    (*E).mbn_var_A = v0;
}
pub unsafe fn MotherBomb_FiringLaser_PositionHead(game: &mut Game) {
    let mut v0: int16 = 0;
    v0 = 8 as int16;
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if (((*E1).base.y_pos as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        v0 = 6 as int16;
    }
    (*E1).mbn_var_32 = v0 as uint16;
    (*E1).mbn_var_33 = v0 as uint16;
    (*E1).mbn_var_34 = 512 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_A = (0xa9b839 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_F = 4 as uint16;
}
pub unsafe fn MotherBomb_FiringLaser_PositionHeadSlowlyFire(game: &mut Game) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v2 = 256 as int16;
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        if (*E1).mbn_var_34 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v2 = -(256 as ::core::ffi::c_int) as int16;
        }
        (*E1).mbn_var_34 = v2 as uint16;
        MotherBrain_SetBrainInstrs(
            addr_kMotherBrain_Ilist_9F34 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).mbn_var_A = (0xa9b863 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 16 as uint16;
    }
}
pub unsafe fn MotherBomb_FiringLaser_FinishAttack(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_32 = 4 as uint16;
        (*E1).mbn_var_33 = 4 as uint16;
        (*E).mbn_var_A = (0xa9b605 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBrain_Body_Phase2_Thinking(game);
    }
}
static off_A9B887: [Option<Func_V>; 4] = unsafe {
    [
        Some(MotherBomb_FiringDeathBeam_0 as unsafe fn(&mut Game) -> ()),
        Some(MotherBomb_FiringDeathBeam_1 as unsafe fn(&mut Game) -> ()),
        Some(MotherBomb_FiringDeathBeam_2 as unsafe fn(&mut Game) -> ()),
        Some(MotherBomb_FiringDeathBeam_3 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn MotherBomb_FiringDeathBeam(game: &mut Game) {
    let mut v0: ::core::ffi::c_int =
        (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_17 as ::core::ffi::c_int;
    off_A9B887[v0 as usize].expect("non-null function pointer")(game);
}
pub unsafe fn MotherBomb_FiringDeathBeam_0(game: &mut Game) {
    if MotherBrain_MakeWalkBackwards(0x28 as uint16, 8 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_32 = 8 as uint16;
        (*E1).mbn_var_33 = 6 as uint16;
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_17 = (*E).mbn_var_17.wrapping_add(1);
    }
}
pub unsafe fn MotherBomb_FiringDeathBeam_1(game: &mut Game) {
    if (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_05 == 0 {
        MotherBrain_SetBodyInstrs(
            addr_kMotherBrain_Ilist_9A42 as ::core::ffi::c_int as uint16,
            game,
        );
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_17 = (*E).mbn_var_17.wrapping_add(1);
    }
}
pub unsafe fn MotherBomb_FiringDeathBeam_2(game: &mut Game) {}
pub unsafe fn MotherBomb_FiringDeathBeam_3(game: &mut Game) {
    MotherBrain_SetBrainInstrs(
        addr_kMotherBrain_Ilist_9C87 as ::core::ffi::c_int as uint16,
        game,
    );
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).mbn_var_32 = 2 as uint16;
    (*E1).mbn_var_33 = 4 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_17 = 0 as uint16;
    (*E).mbn_var_A = (0xa9b605 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MotherBomb_FiringRainbowBeam_0(game: &mut Game) {
    MotherBrain_SetBrainInstrs(
        addr_kMotherBrain_Ilist_9C87 as ::core::ffi::c_int as uint16,
        game,
    );
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).mbn_var_34 = 64 as uint16;
    (*E1).mbn_var_31 = 1 as uint16;
    (*E1).mbn_var_32 = 2 as uint16;
    (*E1).mbn_var_33 = 4 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_A = (0xa9b91a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_F = 256 as uint16;
}
pub unsafe fn MotherBomb_FiringRainbowBeam_1_StartCharge(game: &mut Game) {
    let mut E: *mut Enemy_MotherBomb = Get_MotherBomb(0 as ::core::ffi::c_int, game);
    (*E).mbb_var_F = (*E).mbb_var_F.wrapping_sub(1);
    if (*E).mbb_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        MotherBrain_SetBrainInstrs(
            addr_kMotherBrain_Ilist_9F6C as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).mbb_var_A = (0xa9b92b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBomb_FiringRainbowBeam_2_RetractNeck(game);
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_2_RetractNeck(game: &mut Game) {
    if MotherBrain_WalkBackwardsSlowlyAndRetractHead(0x28 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        let mut E: *mut Enemy_MotherBomb = Get_MotherBomb(0 as ::core::ffi::c_int, game);
        (*E).mbb_var_A = (0xa9b93f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbb_var_F = 256 as uint16;
        MotherBomb_FiringRainbowBeam_3_Wait(game);
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_3_Wait(game: &mut Game) {
    let mut E: *mut Enemy_MotherBomb = Get_MotherBomb(0 as ::core::ffi::c_int, game);
    (*E).mbb_var_F = (*E).mbb_var_F.wrapping_sub(1);
    if (*E).mbb_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        QueueSfx2_Max6(0x71 as uint16, game);
        (*E).mbb_var_A = (0xa9b951 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBomb_FiringRainbowBeam_4_ExtendNeckDown(game);
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_4_ExtendNeckDown(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
        as *mut uint16) = 8 as uint16;
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).mbn_var_32 = 6 as uint16;
    (*E1).mbn_var_33 = 6 as uint16;
    (*E1).mbn_var_34 = 1280 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_A = (0xa9b975 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_F = 16 as uint16;
    MotherBomb_FiringRainbowBeam_5_StartFiring(game);
}
pub unsafe fn MotherBomb_FiringRainbowBeam_5_StartFiring(game: &mut Game) {
    MotherBrain_AimBeamAndIncrWidth(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcee as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
    {
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
        if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0xcee as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            MotherBrain_BodyRainbowBeamPalAnimIndex0(game);
            MotherBrain_SetBrainInstrs(
                addr_kMotherBrain_Ilist_9C77 as ::core::ffi::c_int as uint16,
                game,
            );
            (*E).mbn_var_33 = 512 as uint16;
            (*E).mbn_var_09 = SpawnMotherBrainRainbowBeamHdma(game);
            let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
            (*E1).mbn_var_34 = 64 as uint16;
            (*E1).mbn_var_31 = 1 as uint16;
            (*E1).mbn_var_32 = 2 as uint16;
            (*E1).mbn_var_33 = 4 as uint16;
            (*E).mbn_parameter_1 = 0 as uint16;
            (*E).mbn_parameter_2 = 0 as uint16;
            let mut v4: uint16 = 5 as uint16;
            if ((*((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 700 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                v4 = 24 as uint16;
            }
            CallSomeSamusCode(v4, game);
            (*E).mbn_var_15 = 6 as uint16;
            (*E).mbn_var_A =
                (0xa9b9e5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_6_MoveSamusToWall(game: &mut Game) {
    MotherBrain_PlayRainbowBeamSfx(game);
    MotherBrain_HandleRainbowBeamPalette(game);
    MotherBrain_AimBeamAndIncrWidth(game);
    MotherBrain_HandleRainbowBeamExplosions(game);
    if MotherBrain_MoveSamusTowardsWallDueToBeam(game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        let mut E: *mut Enemy_MotherBomb = Get_MotherBomb(0 as ::core::ffi::c_int, game);
        (*E).mbb_var_A = (0xa9ba00 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbb_var_F = 0 as uint16;
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_7_DelayFrame(game: &mut Game) {
    MotherBrain_PlayRainbowBeamSfx(game);
    MotherBrain_HandleRainbowBeamPalette(game);
    MotherBrain_AimBeamAndIncrWidth(game);
    MotherBrain_HandleRainbowBeamExplosions(game);
    MotherBrain_MoveSamusTowardsWallDueToBeam(game);
    let mut E: *mut Enemy_MotherBomb = Get_MotherBomb(0 as ::core::ffi::c_int, game);
    (*E).mbb_var_F = (*E).mbb_var_F.wrapping_sub(1);
    if (*E).mbb_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
            as *mut uint16) = 8 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
            as *mut uint16) = 8 as uint16;
        (*E).mbb_var_A = (0xa9ba27 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_8_StartDrainSamus(game: &mut Game) {
    let mut E: *mut Enemy_MotherBomb = Get_MotherBomb(0 as ::core::ffi::c_int, game);
    (*E).mbb_var_A = (0xa9ba3c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbb_var_F = 299 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) = 299 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = 8 as uint16;
    MotherBomb_FiringRainbowBeam_9_DrainingSamus(game);
}
pub unsafe fn MotherBomb_FiringRainbowBeam_9_DrainingSamus(game: &mut Game) {
    MotherBrain_PlayRainbowBeamSfx(game);
    MotherBrain_HandleRainbowBeamPalette(game);
    MotherBrain_AimBeamAndIncrWidth(game);
    MotherBrain_HandleRainbowBeamExplosions(game);
    Samus_DamageDueToRainbowBeam(game);
    Samus_DecrementAmmoDueToRainbowBeam(game);
    MotherBrain_MoveSamusTowardsMiddleOfWall(game);
    let mut E: *mut Enemy_MotherBomb = Get_MotherBomb(0 as ::core::ffi::c_int, game);
    (*E).mbb_var_F = (*E).mbb_var_F.wrapping_sub(1);
    if (*E).mbb_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbb_var_A = (0xa9ba5e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_10_FinishFiringRainbow(game: &mut Game) {
    MotherBrain_PlayRainbowBeamSfx(game);
    MotherBrain_HandleRainbowBeamPalette(game);
    MotherBrain_AimBeam(game);
    MotherBrain_HandleRainbowBeamExplosions(game);
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 =
        ((*E).mbn_var_33 as ::core::ffi::c_int - 384 as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_33 = v1;
    if v1 as ::core::ffi::c_int - 512 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E).mbn_var_33 = 512 as uint16;
        (*E).mbn_parameter_1 = -(256 as ::core::ffi::c_int) as uint16;
        (*E).mbn_parameter_2 = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(((*E).mbn_var_09 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        MotherBrain_SetBrainInstrs(
            addr_kMotherBrain_Ilist_9C87 as ::core::ffi::c_int as uint16,
            game,
        );
        MotherBrain_SetupBrainNormalPal(game);
        MotherBrain_WriteDefaultPalette(game);
        QueueSfx1_Max6(2 as uint16, game);
        (*E).mbn_var_16 = 0 as uint16;
        CallSomeSamusCode(1 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
            as *mut uint16) = 8 as uint16;
        (*E).mbn_var_A = (0xa9bac4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_11_LetSamusFall(game: &mut Game) {
    SomeMotherBrainScripts(0 as uint16, game);
    (*Get_MotherBomb(0 as ::core::ffi::c_int, game)).mbb_var_A =
        (0xa9bad1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    MotherBomb_FiringRainbowBeam_12_WaitForSamusHitGround(game);
}
pub unsafe fn MotherBomb_FiringRainbowBeam_12_WaitForSamusHitGround(game: &mut Game) {
    if MotherBrain_MoveSamusForFallingAfterBeam(game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*Get_MotherBomb(0 as ::core::ffi::c_int, game)).mbb_var_A =
            (0xa9badd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_13_LowerHead(game: &mut Game) {
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).mbn_var_34 = 64 as uint16;
    (*E1).mbn_var_31 = 1 as uint16;
    (*E1).mbn_var_32 = 2 as uint16;
    (*E1).mbn_var_33 = 4 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_A = (0xa9bb06 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_F = 128 as uint16;
}
pub unsafe fn MotherBomb_FiringRainbowBeam_14_DecideNextAction(game: &mut Game) {
    let mut E: *mut Enemy_MotherBomb = Get_MotherBomb(0 as ::core::ffi::c_int, game);
    (*E).mbb_var_F = (*E).mbb_var_F.wrapping_sub(1);
    if (*E).mbb_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 400 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            MotherBrain_MakeWalkForwards(
                0xa as uint16,
                ((*E).base.x_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16,
                game,
            );
            (*E).mbb_var_A =
                (0xa9bd45 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            (*E).mbb_var_A =
                (0xa9b8eb as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn MotherBrain_PlayRainbowBeamSfx(game: &mut Game) {
    let mut mbn_var_15: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    mbn_var_15 = (*E).mbn_var_15 as int16;
    if mbn_var_15 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        (*E).mbn_var_15 = (mbn_var_15 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        QueueSfx1_Max6(0x40 as uint16, game);
        (*E).mbn_var_16 = 1 as uint16;
    }
}
pub unsafe fn MotherBrain_WalkBackwardsSlowlyAndRetractHead(
    mut a: uint16,
    game: &mut Game,
) -> uint8 {
    let mut rv: uint8 = MotherBrain_MakeWalkBackwards(a, 0xa as uint16, game);
    if rv != 0 {
        MotherBrain_RetractHead(game);
    }
    return rv;
}
pub unsafe fn MotherBrain_RetractHead(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E).mbn_var_34 = 80 as uint16;
    (*E).mbn_var_31 = 1 as uint16;
    (*E).mbn_var_32 = 8 as uint16;
    (*E).mbn_var_33 = 6 as uint16;
}
pub unsafe fn MotherBrain_AimBeamAndIncrWidth(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    v1 = ((*E).mbn_var_33 as ::core::ffi::c_int + 384 as ::core::ffi::c_int) as int16;
    if (*E).mbn_var_33 as ::core::ffi::c_int - 2688 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        v1 = 3072 as int16;
    }
    (*E).mbn_var_33 = v1 as uint16;
    MotherBrain_AimBeam(game);
}
pub unsafe fn MotherBrain_AimBeam(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_31 = -(CalculateAngleFromXY(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int
            - 16 as ::core::ffi::c_int) as uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - (*E).base.y_pos as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int) as uint16,
        game,
    ) as ::core::ffi::c_int
        + 0x80 as ::core::ffi::c_int)
        as uint8 as uint16;
}
pub unsafe fn MotherBrain_CalculateRainbowBeamHdma_(game: &mut Game) {
    MotherBrain_CalculateRainbowBeamHdma(game);
}
pub unsafe fn MotherBrain_MoveSamusTowardsWallDueToBeam(game: &mut Game) -> uint8 {
    let mut v0: uint8 = MotherBrain_MoveSamusHorizTowardsWall(0x1000 as uint16, game);
    if v0 == 0 {
        let mut v1: uint16 = Math_MultByCos(
            0x1000 as uint16,
            (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_31,
        );
        MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(v1, game);
    }
    return v0;
}
pub unsafe fn MotherBrain_MoveSamusTowardsMiddleOfWall(game: &mut Game) {
    let mut v0: uint16 = 64 as uint16;
    if ((124 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        v0 = -(64 as ::core::ffi::c_int) as uint16;
    }
    MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(v0, game);
}
pub unsafe fn MotherBrain_MoveSamusForFallingAfterBeam(game: &mut Game) -> uint8 {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    v1 = ((*E).mbn_parameter_1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as int16;
    if v1 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        v1 = 0 as int16;
    }
    (*E).mbn_parameter_1 = v1 as uint16;
    MotherBrain_MoveSamusHorizTowardsWall(v1 as uint16, game);
    let mut v2: uint16 =
        ((*E).mbn_parameter_2 as ::core::ffi::c_int + 24 as ::core::ffi::c_int) as uint16;
    (*E).mbn_parameter_2 = v2;
    return (MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(v2, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int) as uint8;
}
pub unsafe fn MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(
    mut a: uint16,
    game: &mut Game,
) -> uint8 {
    let mut carry: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xafc as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
        + *(&raw mut a as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xafc as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) = carry as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb16 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) = carry as uint8;
    let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + (a as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
        + (carry >> 8 as ::core::ffi::c_int)) as uint16;
    let mut v6: uint16 = 0;
    if v4 as ::core::ffi::c_int - 48 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v6 = 48 as uint16;
    } else {
        if v4 as ::core::ffi::c_int - 192 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        {
            let ref mut fresh2 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh2 = v4;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
                as *mut uint16) = *fresh2;
            return 0 as uint8;
        }
        v6 = 192 as uint16;
    }
    let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh3 = v6;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
        as *mut uint16) = *fresh3;
    *((&raw mut game.g_ram as *mut uint8).offset(0xafc as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb16 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    return 1 as uint8;
}
pub unsafe fn MotherBrain_MoveSamusHorizTowardsWall(mut a: uint16, game: &mut Game) -> uint8 {
    let mut carry: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf8 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
        + *(&raw mut a as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf8 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) = carry as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb12 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) = carry as uint8;
    let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + (a as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
        + (carry >> 8 as ::core::ffi::c_int)) as uint16;
    if v4 as ::core::ffi::c_int - 235 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) = v4;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
            as *mut uint16) = v4;
        return 0 as uint8;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) = 235 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
            as *mut uint16) = 235 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf8 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb12 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        return 1 as uint8;
    };
}
pub unsafe fn MotherBrain_HandleRainbowBeamExplosions(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_parameter_2 = (*E).mbn_parameter_2.wrapping_sub(1);
    if (*E).mbn_parameter_2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_parameter_2 = 8 as uint16;
        (*E).mbn_parameter_1 = (*E).mbn_parameter_1.wrapping_add(1);
        let mut v2: ::core::ffi::c_int =
            (*E).mbn_parameter_1 as ::core::ffi::c_int & 7 as ::core::ffi::c_int;
        game.eproj_spawn_pt = Point16U {
            x: g_word_A9BCA6[v2 as usize] as uint16,
            y: g_word_A9BCB6[v2 as usize] as uint16,
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainRainbowBeamExplosion as ::core::ffi::c_int as uint16,
            0 as uint16,
            game,
        );
        QueueSfx2_Max6(0x24 as uint16, game);
    }
}
pub unsafe fn MotherBrain_BodyRainbowBeamPalAnimIndex0(game: &mut Game) {
    (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_01 = 0 as uint16;
}
pub unsafe fn MotherBrain_WriteDefaultPalette(game: &mut Game) {
    WriteColorsToPalette(
        0x82 as uint16,
        0xa9 as uint8,
        (addr_kMotherBrainsBrain_Palette as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
        0xf as uint16,
        game,
    );
    WriteColorsToPalette(
        0x122 as uint16,
        0xa9 as uint8,
        (addr_kMotherBrainsBrain_Palette as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
        0xf as uint16,
        game,
    );
    WriteColorsToPalette(
        0x162 as uint16,
        0xa9 as uint8,
        (addr_kMotherBrainPalette_0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
        0xf as uint16,
        game,
    );
}
pub unsafe fn MotherBrain_WritePhase2DeathPalette(game: &mut Game) {
    MotherBrain_WritePalette(
        *(game.g_rom.offset(
            (((0xade434 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xade434 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(6 as ::core::ffi::c_int as isize),
        game,
    );
}
pub unsafe fn MotherBrain_HandleRainbowBeamPalette(game: &mut Game) {
    if (*Get_MotherBrain(0 as ::core::ffi::c_int, game))
        .base
        .frame_counter as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut mbn_var_01: uint16 =
            (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_01;
        let mut v1: uint16 = 0;
        loop {
            v1 = mbn_var_01;
            mbn_var_01 = *(game.g_rom.offset(
                (((0xade434 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xade434 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((mbn_var_01 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            if !(mbn_var_01 == 0) {
                break;
            }
        }
        let mut v2: uint16 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_01 = v2;
        MotherBrain_WritePalette(
            *((game.g_rom.offset(
                (((0xade434 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xade434 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16 as *mut uint8)
                .offset(v2 as ::core::ffi::c_int as isize)
                .offset(-(2 as ::core::ffi::c_int as isize)) as *mut uint16),
            game,
        );
    }
}
pub unsafe fn MotherBrain_WritePalette(mut j: uint16, game: &mut Game) {
    WriteColorsToPalette(0x82 as uint16, 0xad as uint8, j, 15 as uint16, game);
    WriteColorsToPalette(0x122 as uint16, 0xad as uint8, j, 15 as uint16, game);
    WriteColorsToPalette(
        0x162 as uint16,
        0xad as uint8,
        (j as ::core::ffi::c_int + 30 as ::core::ffi::c_int) as uint16,
        15 as uint16,
        game,
    );
}
pub unsafe fn MotherBrain_Phase2Cut_0(game: &mut Game) {
    let mut v0: uint16 = SuitDamageDivision(0x50 as uint16, game);
    if (4 as ::core::ffi::c_int * v0 as ::core::ffi::c_int + 20 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
            (0xa9bd98 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0xfff as ::core::ffi::c_int)
        < 0xfa0 as ::core::ffi::c_int
    {
        if (*Get_MotherBrain(0 as ::core::ffi::c_int, game))
            .base
            .frame_counter as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            MotherBrain_MaybeStandupOrLeanDown(game);
        }
    } else {
        let mut v1: uint16 = 0;
        if (SuitDamageDivision(0xa0 as uint16, game) as ::core::ffi::c_int
            + 20 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
            || {
                v1 = addr_kMotherBrain_Ilist_9ECC as ::core::ffi::c_int as uint16;
                (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0xfff as ::core::ffi::c_int)
                    < 0xff0 as ::core::ffi::c_int
            }
        {
            v1 = addr_kMotherBrain_Ilist_9D7F as ::core::ffi::c_int as uint16;
        }
        MotherBrain_SetBrainInstrs(v1, game);
    };
}
pub unsafe fn MotherBrain_Phase2Cut_1(game: &mut Game) {
    if MotherBrain_MakeHerStandUp(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_A = (0xa9bda9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 16 as uint16;
        MotherBrain_Phase2Cut_2(game);
    }
}
pub unsafe fn MotherBrain_Phase2Cut_2(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        MotherBrain_SetBrainInstrs(
            addr_kMotherBrain_Ilist_9B7F as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).mbn_var_A = (0xa9bdc1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 256 as uint16;
    }
}
pub unsafe fn MotherBrain_Phase2Cut_3(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        MotherBrain_SetBrainInstrs(
            addr_kMotherBrain_Ilist_9F6C as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).mbn_var_A = (0xa9bdd2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBrain_Phase2Cut_4(game);
    }
}
pub unsafe fn MotherBrain_Phase2Cut_4(game: &mut Game) {
    if ProcessSpriteTilesTransfers(
        0xa9 as uint8,
        addr_stru_A98FE5 as ::core::ffi::c_int as uint16,
        game,
    ) != 0
    {
        MotherBrain_RetractHead(game);
        MotherBrain_SpawnShitroidInCutscene(game);
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_A = (0xa9bded as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 256 as uint16;
    }
}
pub unsafe fn MotherBrain_Phase2Cut_5(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        MotherBrain_BodyRainbowBeamPalAnimIndex0(game);
        MotherBrain_SetBrainInstrs(
            addr_kMotherBrain_Ilist_9C77 as ::core::ffi::c_int as uint16,
            game,
        );
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_32 = 6 as uint16;
        (*E1).mbn_var_33 = 6 as uint16;
        (*E1).mbn_var_34 = 1280 as uint16;
        QueueSfx2_Max6(0x71 as uint16, game);
        (*E).mbn_var_A = (0xa9be1a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrain_SpawnShitroidInCutscene(game: &mut Game) {
    (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_0A = SpawnEnemy(
        0xa9 as uint8,
        addr_stru_A9BE28 as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn MotherBrain_DrainedByShitroid_0(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_00 = 3 as uint16;
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).mbn_var_32 = 8 as uint16;
    (*E1).mbn_var_33 = 8 as uint16;
    (*E1).mbn_var_34 = 1792 as uint16;
    (*E).mbn_var_A = (0xa9be5d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_F = 48 as uint16;
    MotherBrain_DrainedByShitroid_1(game);
}
pub unsafe fn MotherBrain_DrainedByShitroid_1(game: &mut Game) {
    MotherBrain_HandleRainbowBeamPalette(game);
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_A = (0xa9be96 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_08 =
            (0xa9bfd0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E1).mbn_var_06 = 0 as uint16;
        (*E1).mbn_var_07 = 2 as uint16;
        (*E1).mbn_var_32 = 2 as uint16;
        (*E1).mbn_var_33 = 4 as uint16;
    }
}
pub unsafe fn CallMotherBrainPainfulWalk(mut ea: uint32, game: &mut Game) {
    match ea {
        11124688 => {
            MotherBrain_PainfulWalkForwards(game);
            return;
        }
        11124712 => {
            MotherBrain_PainfulWalkingForwards(game);
            return;
        }
        11124740 => {
            MotherBrain_PainfulWalkBackwards(game);
            return;
        }
        11124764 => {
            MotherBrain_PainfulWalkingBackwards(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn MotherBrain_DrainedByShitroid_2(game: &mut Game) {
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if (*E1).mbn_var_00 == 0 {
        (*E1).mbn_var_00 = 50 as uint16;
    }
    MotherBrain_HandleRainbowBeamPalette(game);
    CallMotherBrainPainfulWalk(
        ((*E1).mbn_var_08 as ::core::ffi::c_int | 0xa90000 as ::core::ffi::c_int) as uint32,
        game,
    );
    let mut v2: uint16 =
        (2 as ::core::ffi::c_int * (*E1).mbn_var_06 as ::core::ffi::c_int) as uint16;
    (*E1).mbn_var_07 = g_byte_A9BEEE[v2 as usize] as uint16;
    (*E1).mbn_var_34 =
        g_word_A9BEFE[(v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
    if (*E1).mbn_var_06 as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_16 = 0 as uint16;
        (*E1).mbn_var_10 = 0 as uint16;
        MotherBrain_WritePhase2DeathPalette(game);
        QueueSfx1_Max6(2 as uint16, game);
        (*E).mbn_var_A = (0xa9bf0e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrain_DrainedByShitroid_3(game: &mut Game) {
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    CallMotherBrainPainfulWalk(
        ((*E1).mbn_var_08 as ::core::ffi::c_int | 0xa90000 as ::core::ffi::c_int) as uint32,
        game,
    );
    if !((*E1).mbn_var_06 as ::core::ffi::c_int - 8 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0)
    {
        (*E1).mbn_var_34 = 64 as uint16;
        (*E1).mbn_var_32 = 8 as uint16;
        (*E1).mbn_var_33 = 8 as uint16;
        MotherBrain_SetBrainInstrs(
            addr_kMotherBrain_Ilist_9C39 as ::core::ffi::c_int as uint16,
            game,
        );
        (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
            (0xa9bf41 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBrain_DrainedByShitroid_4(game);
    }
}
pub unsafe fn MotherBrain_DrainedByShitroid_4(game: &mut Game) {
    if MotherBrain_MakeWalkBackwards(0x28 as uint16, 0 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
            (0xa9bf56 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_33 = 0 as uint16;
        MotherBrain_DrainedByShitroid_5(game);
    }
}
pub unsafe fn MotherBrain_DrainedByShitroid_5(game: &mut Game) {
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if *(&raw mut (*E1).mbn_var_32 as *mut uint32) == 0 {
        (*E1).mbn_var_12 = ((*E1).mbn_var_33 as ::core::ffi::c_int
            | (*E1).mbn_var_32 as ::core::ffi::c_int) as uint16;
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        if (*E).mbn_var_02 == 0 {
            MotherBrain_SetBodyInstrs(
                addr_kMotherBrain_Ilist_9A26 as ::core::ffi::c_int as uint16,
                game,
            );
            (*E).mbn_var_A =
                (0xa9bf7d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).mbn_var_F = 64 as uint16;
        }
    }
}
pub unsafe fn MotherBrain_DrainedByShitroid_6(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_37 = 0 as uint16;
        (*E).mbn_var_A = (0xa9bf95 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 16 as uint16;
        MotherBrain_DrainedByShitroid_7(game);
    }
}
pub unsafe fn MotherBrain_DrainedByShitroid_7(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    let mut v2: bool = (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    let mut v3: uint16 = 0;
    if v2 as ::core::ffi::c_int != 0 && {
        (*E).mbn_var_F = 16 as uint16;
        v3 = ((*E).mbn_var_37 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_37 = v3;
        MotherBrain_FadeToGray_Drained(
            (v3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
            game,
        ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
    } {
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).base.health = -(29536 as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_1F = 1 as uint16;
        (*E1).mbn_var_14 = 0 as uint16;
        (*E).mbn_var_00 = 2 as uint16;
        (*E).mbn_var_A = (0xa9c059 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrain_PainfulWalkForwards(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if MotherBrain_MakeWalkForwards((*E).mbn_var_07, 0x48 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*E).mbn_var_08 = (0xa9bfe8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBrain_SetPainfulWalkingTimer(game);
    }
}
pub unsafe fn MotherBrain_PainfulWalkingForwards(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    let mut v1: uint16 =
        ((*E).mbn_var_09 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_09 = v1;
    if v1 == 0 {
        (*E).mbn_var_06 = (*E).mbn_var_06.wrapping_add(1);
        (*E).mbn_var_08 = (0xa9c004 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrain_PainfulWalkBackwards(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if MotherBrain_MakeWalkBackwards(0x28 as uint16, (*E).mbn_var_07, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*E).mbn_var_08 = (0xa9c01c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBrain_SetPainfulWalkingTimer(game);
    }
}
pub unsafe fn MotherBrain_PainfulWalkingBackwards(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    let mut v1: uint16 =
        ((*E).mbn_var_09 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_09 = v1;
    if v1 == 0 {
        (*E).mbn_var_06 = (*E).mbn_var_06.wrapping_add(1);
        (*E).mbn_var_08 = (0xa9bfd0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrain_SetPainfulWalkingTimer(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E).mbn_var_09 = *((&raw const g_word_A9C049 as *const int16).offset((*E).mbn_var_06 as isize)
        as *const int16 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as uint16;
}
pub unsafe fn MotherBrain_Phase2_Revive_0(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_A = (0xa9c066 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_F = 768 as uint16;
}
pub unsafe fn MotherBrain_Phase2_Revive_1(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_14 = 1 as uint16;
        (*E1).mbn_var_12 = 1 as uint16;
        (*E).mbn_var_A = (0xa9c082 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 224 as uint16;
        MotherBrain_Phase2_Revive_2(game);
    }
}
pub unsafe fn MotherBrain_Phase2_Revive_2(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_A = (0xa9c08f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_37 = 0 as uint16;
    MotherBrain_Phase2_Revive_3(game);
}
pub unsafe fn MotherBrain_Phase2_Revive_3(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    let mut v1: bool = (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    let mut v2: uint16 = 0;
    if v1 as ::core::ffi::c_int != 0 && {
        (*E).mbn_var_F = 16 as uint16;
        v2 = ((*E).mbn_var_37 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_37 = v2;
        MotherBrain_FadeFromGray_Drained(
            (v2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
            game,
        ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
    } {
        (*E).mbn_var_A = (0xa9c0ba as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_10 = 1 as uint16;
        MotherBrain_SetupBrainNormalPal(game);
        MotherBrain_Phase2_Revive_4(game);
    }
}
pub unsafe fn MotherBrain_Phase2_Revive_4(game: &mut Game) {
    if MotherBrain_MakeHerStandUp(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_32 = 6 as uint16;
        (*E1).mbn_var_33 = 6 as uint16;
        (*E1).mbn_var_34 = 1280 as uint16;
        (*E1).mbn_var_31 = 1 as uint16;
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_A = (0xa9c0e4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 16 as uint16;
        MotherBrain_Phase2_Revive_5(game);
    }
}
pub unsafe fn MotherBrain_Phase2_Revive_5(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        MotherBrain_SetBrainInstrs(
            addr_kMotherBrain_Ilist_9BB3 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).mbn_var_A = (0xa9c0fb as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 128 as uint16;
        MotherBrain_Phase2_Revive_6(game);
    }
}
pub unsafe fn MotherBrain_Phase2_Revive_6(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    let mut v1: bool = (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v1 as ::core::ffi::c_int != 0
        && MotherBrain_MakeWalkForwards(4 as uint16, 0x50 as uint16, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
    {
        (*E).mbn_var_A = (0xa9c11e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_1F = 2 as uint16;
        (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_11 = 1 as uint16;
    }
}
pub unsafe fn MotherBrain_Phase2_Revive_7(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_13 = 0 as uint16;
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).mbn_var_31 = 1 as uint16;
    (*E1).mbn_var_32 = 2 as uint16;
    (*E1).mbn_var_33 = 4 as uint16;
    (*E1).mbn_var_34 = 64 as uint16;
    (*E).mbn_var_A = (0xa9c147 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    MotherBrain_Phase2_Revive_8(game);
}
pub unsafe fn MotherBrain_Phase2_Revive_8(game: &mut Game) {
    if MotherBrain_MakeHerStandUp(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
            (0xa9c15c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBrain_MakeWalkForwards(0xa as uint16, 0x50 as uint16, game);
        MotherBrain_Phase2_MurderShitroid_1(game);
    }
}
pub unsafe fn MotherBrain_Phase2_MurderShitroid_1(game: &mut Game) {
    MotherBrain_MaybeStandupOrLeanDown(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut v0: uint16 = addr_kMotherBrain_Ilist_9DBB as ::core::ffi::c_int as uint16;
        if (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_0A != 0 {
            v0 = addr_kMotherBrain_Ilist_9DB1 as ::core::ffi::c_int as uint16;
        }
        MotherBrain_SetBrainInstrs(v0, game);
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_A = (0xa9c182 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_F = 64 as uint16;
    }
}
pub unsafe fn MotherBrain_Phase2_MurderShitroid_2(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_A = (0xa9c15c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrain_Phase2_PrepareForFinalShitroid(game: &mut Game) {
    MotherBrain_MakeHerStandUp(game);
    MotherBrain_MakeWalkBackwards(0x40 as uint16, 4 as uint16, game);
}
pub unsafe fn MotherBrain_Phase2_ExecuteFinalkShitroid(game: &mut Game) {
    MotherBrain_SetBrainInstrs(
        addr_kMotherBrain_Ilist_9DB1 as ::core::ffi::c_int as uint16,
        game,
    );
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
        (0xa9c1a6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MotherBrain_MaybeStandupOrLeanDown(game: &mut Game) {
    let mut mbn_var_02: uint16 = (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_02;
    if mbn_var_02 != 0 {
        if mbn_var_02 as ::core::ffi::c_int == 6 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8 as ::core::ffi::c_int
                >= 0xc0 as ::core::ffi::c_int
        {
            MotherBrain_MakeHerStandUp(game);
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as uint8 as ::core::ffi::c_int
        >= 0xc0 as ::core::ffi::c_int
    {
        MotherBrain_MakeHerLeanDown(game);
    }
}
pub unsafe fn MotherBrain_Phase3_Recover_MakeDistance(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_00 = 4 as uint16;
    (*E).mbn_var_A = (0xa9c1f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_F = 32 as uint16;
    MotherBrain_MakeWalkBackwards(
        ((*E).base.x_pos as ::core::ffi::c_int - 14 as ::core::ffi::c_int) as uint16,
        2 as uint16,
        game,
    );
}
pub unsafe fn MotherBrain_Phase3_Recover_SetupForFight(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_A = (0xa9c209 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_18 =
            (0xa9c330 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E1).mbn_var_1A =
            (0xa9c26a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBrain_Phase3_Fighting_Main(game);
    }
}
pub unsafe fn MotherBrain_Phase3_Fighting_Main(game: &mut Game) {
    if (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game))
        .base
        .health
        != 0
    {
        MotherBrain_Phase3_NeckHandler(game);
        MotherBrain_Phase3_WalkHandler(game);
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        if (*E).mbn_var_02 == 0
            && (*E).mbn_var_06 == 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
        {
            let mut v1: uint16 = addr_kMotherBrain_Ilist_9F00 as ::core::ffi::c_int as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8 as ::core::ffi::c_int
                >= 0x80 as ::core::ffi::c_int
            {
                v1 = addr_kMotherBrain_Ilist_9DBB as ::core::ffi::c_int as uint16;
            }
            MotherBrain_SetBrainInstrs(v1, game);
            (*E).mbn_var_A =
                (0xa9c24e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).mbn_var_F = 64 as uint16;
        }
    } else {
        (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
            (0xa9aee1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn MotherBrain_Phase3_Fighting_Cooldown(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_A = (0xa9c209 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CallMotherBrainWallkFunc1A(mut ea: uint32, game: &mut Game) {
    match ea {
        11125354 => {
            MotherBrain_Phase3_Walk_TryToInchForward(game);
            return;
        }
        11125427 => {
            MotherBrain_Phase3_Walk_RetreatQuickly(game);
            return;
        }
        11125458 => {
            MotherBrain_Phase3_Walk_RetreatSlowly(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn MotherBrain_Phase3_WalkHandler(game: &mut Game) {
    if (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_02 == 0 {
        CallMotherBrainWallkFunc1A(
            ((*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_1A as ::core::ffi::c_int
                | 0xa90000 as ::core::ffi::c_int) as uint32,
            game,
        );
    }
}
pub unsafe fn MotherBrain_Phase3_Walk_TryToInchForward(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let mut mbn_var_07: uint16 = (*E).mbn_var_07;
    if mbn_var_07 != 0 {
        let mut v3: uint16 =
            (mbn_var_07 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_07 = v3;
        if v3 as ::core::ffi::c_int >= 0x100 as ::core::ffi::c_int {
            let mut v4: uint16 =
                ((*E).base.x_pos as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
            (*E1).mbn_var_1B = v4;
            if MotherBrain_MakeWalkForwards(
                ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 2 as ::core::ffi::c_int)
                    + 4 as ::core::ffi::c_int) as uint16,
                (*E1).mbn_var_1B,
                game,
            ) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                != 0
            {
                (*E).mbn_var_07 = 128 as uint16;
            }
        }
    } else {
        let mut v6: uint16 =
            ((*E).base.x_pos as ::core::ffi::c_int - 14 as ::core::ffi::c_int) as uint16;
        let mut E1_0: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1_0).mbn_var_1B = v6;
        (*E1_0).mbn_var_1A =
            (0xa9c2b3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MotherBrain_Phase3_Walk_RetreatQuickly(game);
    };
}
pub unsafe fn MotherBrain_Phase3_Walk_RetreatQuickly(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if MotherBrain_MakeWalkBackwards((*E).mbn_var_1B, 2 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*E).mbn_var_1B = ((*Get_MotherBrain(0 as ::core::ffi::c_int, game)).base.x_pos
            as ::core::ffi::c_int
            - 14 as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_1A = (0xa9c2d2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MotherBrain_Phase3_Walk_RetreatSlowly(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if MotherBrain_MakeWalkBackwards((*E).mbn_var_1B, 4 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        MotherBrain_SetToTryToInchForward(0x40 as uint16, game);
    }
}
pub unsafe fn MotherBrain_SetToTryToInchForward(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_07 = a;
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).mbn_var_1A = (0xa9c26a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E1).mbn_var_1B = ((*E).base.x_pos as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CallMotherBrainNeckFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        11125587 => return,
        11125552 => {
            MotherBrain_Phase3_Neck_Normal(game);
            return;
        }
        11125588 => {
            MotherBrain_Phase3_Neck_SetupRecoilRecovery(game);
            return;
        }
        11125627 => {
            MotherBrain_Phase3_Neck_RecoilRecovery(game);
            return;
        }
        11125653 => {
            MotherBrain_Phase3_Neck_SetupHyperBeamRecoil(game);
            return;
        }
        11125709 => {
            MotherBrain_Phase3_Neck_HyperBeamRecoil(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn MotherBrain_Phase3_NeckHandler(game: &mut Game) {
    CallMotherBrainNeckFunc(
        ((*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_18 as ::core::ffi::c_int
            | 0xa90000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn MotherBrain_Phase3_Neck_Normal(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E).mbn_var_32 = 1 as uint16;
    (*E).mbn_var_34 = 128 as uint16;
    (*E).mbn_var_32 = 2 as uint16;
    (*E).mbn_var_33 = 4 as uint16;
    (*E).mbn_var_18 = addr_locret_A9C353 as ::core::ffi::c_int as uint16;
}
pub unsafe fn MotherBrain_Phase3_Neck_SetupRecoilRecovery(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E).mbn_var_31 = 1 as uint16;
    (*E).mbn_var_34 = 1280 as uint16;
    (*E).mbn_var_32 = 6 as uint16;
    (*E).mbn_var_33 = 6 as uint16;
    (*E).mbn_var_18 = (0xa9c37b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_19 = 16 as uint16;
    MotherBrain_Phase3_Neck_RecoilRecovery(game);
}
pub unsafe fn MotherBrain_Phase3_Neck_RecoilRecovery(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    v1 = ((*E).mbn_var_19 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
    if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        MotherBrain_SetBrainInstrs(
            addr_kMotherBrain_Ilist_9DBB as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).mbn_var_18 = (0xa9c330 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).mbn_var_19 = v1 as uint16;
    };
}
pub unsafe fn MotherBrain_Phase3_Neck_SetupHyperBeamRecoil(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E).mbn_var_31 = 1 as uint16;
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_06 = 1 as uint16;
    MotherBrain_SetBrainInstrs(
        addr_kMotherBrain_Ilist_9BE7 as ::core::ffi::c_int as uint16,
        game,
    );
    (*E).mbn_var_00 = 50 as uint16;
    (*E).mbn_var_34 = 2304 as uint16;
    (*E).mbn_var_32 = 8 as uint16;
    (*E).mbn_var_33 = 8 as uint16;
    (*E).mbn_var_18 = (0xa9c3cd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_19 = 11 as uint16;
    MotherBrain_Phase3_Neck_HyperBeamRecoil(game);
}
pub unsafe fn MotherBrain_Phase3_Neck_HyperBeamRecoil(game: &mut Game) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    v2 = ((*E).mbn_var_19 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
    if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).mbn_var_34 = 128 as uint16;
        (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_06 = 0 as uint16;
        (*E).mbn_var_18 = (0xa9c354 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).mbn_var_19 = v2 as uint16;
    };
}
pub unsafe fn MoveEnemyWithVelocity(game: &mut Game) {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut ai_var_B: uint16 = (*v0).ai_var_B;
    let mut carry: ::core::ffi::c_int = *(&raw mut (*v0).x_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int
        + *(&raw mut ai_var_B as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int;
    *(&raw mut (*v0).x_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) = carry as uint8;
    (*v0).x_pos = ((*v0).x_pos as ::core::ffi::c_int
        + ((ai_var_B as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8
            as ::core::ffi::c_int
            + (carry >> 8 as ::core::ffi::c_int))) as uint16;
    let mut ai_var_C: uint16 = (*v0).ai_var_C;
    carry = *(&raw mut (*v0).y_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int
        + *(&raw mut ai_var_C as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int;
    *(&raw mut (*v0).y_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) = carry as uint8;
    (*v0).y_pos = ((*v0).y_pos as ::core::ffi::c_int
        + ((ai_var_C as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8
            as ::core::ffi::c_int
            + (carry >> 8 as ::core::ffi::c_int))) as uint16;
}
pub unsafe fn MotherBrain_SetBodyInstrs(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.current_instruction = a;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn MotherBrain_SetBrainUnusedInstrs(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E).base.current_instruction = a;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn MotherBrain_SetBrainInstrs(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_21 = a;
    (*E).mbn_var_20 = 1 as uint16;
}
pub unsafe fn Enemy_SetInstrList(mut k: uint16, mut a: uint16, game: &mut Game) {
    let mut v2: *mut EnemyData = gEnemyData(k as ::core::ffi::c_int, game);
    (*v2).current_instruction = a;
    (*v2).instruction_timer = 1 as uint16;
    (*v2).timer = 0 as uint16;
}
pub unsafe fn ComputeSinMult(mut a: uint16, mut r18: uint16, game: &mut Game) -> uint16 {
    return sub_A9C46C(r18, a, game);
}
pub unsafe fn ComputeCosMult(mut a: uint16, mut r18: uint16, game: &mut Game) -> uint16 {
    return sub_A9C46C(
        (r18 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16,
        a,
        game,
    );
}
pub unsafe fn sub_A9C46C(mut a: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v2: uint16 = kSinCosTable8bit_Sext[((a as ::core::ffi::c_int
        & 0xff as ::core::ffi::c_int)
        + 64 as ::core::ffi::c_int) as usize] as uint16;
    WriteReg(M7A as ::core::ffi::c_int as uint16, v2 as uint8, game);
    WriteReg(
        M7A as ::core::ffi::c_int as uint16,
        *(&raw mut v2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ),
        game,
    );
    WriteReg(M7B as ::core::ffi::c_int as uint16, j as uint8, game);
    return ReadRegWord(MPYM as ::core::ffi::c_int as uint16, game);
}
pub unsafe fn Enemy_IncreaseYpos(mut k: uint16, mut a: uint16, game: &mut Game) {
    let mut v2: *mut EnemyData = gEnemyData(k as ::core::ffi::c_int, game);
    let mut carry: ::core::ffi::c_int = *(&raw mut (*v2).y_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int
        + *(&raw mut a as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int;
    *(&raw mut (*v2).y_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) = carry as uint8;
    (*v2).y_pos = ((*v2).y_pos as ::core::ffi::c_int
        + ((a as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
            + (carry >> 8 as ::core::ffi::c_int))) as uint16;
}
pub unsafe fn Samus_DecrementAmmoDueToRainbowBeam(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe44 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9c6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
    {
        let mut v0: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x9c6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - g_word_A9C544 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9c6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - g_word_A9C544 as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == 1 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
            }
            v0 = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa04 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c6 as ::core::ffi::c_int as isize)
            as *mut uint16) = v0;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe44 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9ca as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
    {
        let mut v1: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x9ca as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - g_word_A9C544 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9ca as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - g_word_A9C544 as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == 2 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
            }
            v1 = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa04 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x9ca as ::core::ffi::c_int as isize)
            as *mut uint16) = v1;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9ce as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x9ce as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - g_word_A9C544 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9ce as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - g_word_A9C544 as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == 3 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
            }
            v2 = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa04 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x9ce as ::core::ffi::c_int as isize)
            as *mut uint16) = v2;
    }
}
pub unsafe fn Samus_PlayGainingLosingHealthSfx(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 81 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xe44 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        QueueSfx3_Max3(0x2d as uint16, game);
    }
}
pub unsafe fn Samus_DamageDueToShitroid(game: &mut Game) {
    let mut v0: int16 = 0;
    v0 = -(4 as ::core::ffi::c_int) as int16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        v0 = -(3 as ::core::ffi::c_int) as int16;
    }
    let mut v1: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x9c2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int)
        + v0 as ::core::ffi::c_int) as uint16;
    if v1 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v1 = 1 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
        as *mut uint16) = v1;
}
pub unsafe fn Samus_DamageDueToRainbowBeam(game: &mut Game) {
    let mut v0: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x9c2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int)
        - 2 as ::core::ffi::c_int) as uint16;
    if v0 as ::core::ffi::c_int - 1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v0 = 0 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
        as *mut uint16) = v0;
    Samus_PlayGainingLosingHealthSfx(game);
}
pub unsafe fn Samus_HealDueToShitroid(game: &mut Game) -> uint8 {
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x9c4 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9c2 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh4 = (*fresh4).wrapping_add(1);
        Samus_PlayGainingLosingHealthSfx(game);
        return 1 as uint8;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9c4 as ::core::ffi::c_int as isize)
            as *mut uint16);
        Samus_PlayGainingLosingHealthSfx(game);
        return 0 as uint8;
    };
}
pub unsafe fn ProcessSpriteTilesTransfers(mut db: uint8, mut k: uint16, game: &mut Game) -> uint8 {
    let mut v7: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let mut src: uint16 = (*E).mbn_var_22;
    if src == 0 {
        src = k;
    }
    let mut v4: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v5: *const uint8 = RomPtrWithBank(db, src as uint16_t, game);
    let mut v6: uint16 = *(v5 as *mut uint16);
    if v6 != 0 {
        v7 = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
            game,
        );
        (*v7).size = v6;
        *((&raw mut (*v7).src.addr as *mut uint8).offset(1 as ::core::ffi::c_int as isize)
            as *mut VoidP) = *(v5.offset(3 as ::core::ffi::c_int as isize) as *mut uint16) as VoidP;
        (*v7).src.addr = *(v5.offset(2 as ::core::ffi::c_int as isize) as *mut uint16) as VoidP;
        (*v7).vram_dst = *(v5.offset(5 as ::core::ffi::c_int as isize) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v4 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_22 = (src as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
        v6 = *(RomPtrWithBank(db, (*E).mbn_var_22 as uint16_t, game) as *mut uint16);
        if v6 != 0 {
            return 0 as uint8;
        }
    }
    (*E).mbn_var_22 = v6;
    return 1 as uint8;
}
pub unsafe fn MotherBrain_MakeWalkForwards(mut j: uint16, mut a: uint16, game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if (a as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        if (*E).mbn_var_02 != 0 {
            return 0 as uint8;
        }
        if (*E).base.x_pos as ::core::ffi::c_int - 128 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            MotherBrain_SetBodyInstrs(
                *(game.g_rom.offset(
                    (((0xa9c61e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa9c61e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
                game,
            );
            return 0 as uint8;
        }
    }
    return 1 as uint8;
}
pub unsafe fn MotherBrain_MakeWalkBackwards(
    mut a: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint8 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if ((a as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if (*E).mbn_var_02 != 0 {
            return 0 as uint8;
        }
        if (*E).base.x_pos as ::core::ffi::c_int - 48 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            MotherBrain_SetBodyInstrs(
                *(game.g_rom.offset(
                    (((0xa9c664 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa9c664 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
                game,
            );
            return 0 as uint8;
        }
    }
    return 1 as uint8;
}
pub unsafe fn MotherBrain_MakeHerStandUp(game: &mut Game) -> uint8 {
    let mut mbn_var_02: uint16 = (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_02;
    if mbn_var_02 != 0 {
        let mut v1: uint16 = addr_kMotherBrain_Ilist_99C6 as ::core::ffi::c_int as uint16;
        if mbn_var_02 as ::core::ffi::c_int != 3 as ::core::ffi::c_int {
            if mbn_var_02 as ::core::ffi::c_int != 6 as ::core::ffi::c_int {
                return 0 as uint8;
            }
            v1 = addr_kMotherBrain_Ilist_99E2 as ::core::ffi::c_int as uint16;
        }
        MotherBrain_SetBodyInstrs(v1, game);
        return 0 as uint8;
    }
    return 1 as uint8;
}
pub unsafe fn MotherBrain_MakeHerCrouch(game: &mut Game) -> uint8 {
    let mut mbn_var_02: uint16 = (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_02;
    if mbn_var_02 != 0 {
        if mbn_var_02 as ::core::ffi::c_int == 3 as ::core::ffi::c_int {
            return 1 as uint8;
        }
    } else {
        MotherBrain_SetBodyInstrs(
            addr_kMotherBrain_Ilist_9A0A as ::core::ffi::c_int as uint16,
            game,
        );
    }
    return 0 as uint8;
}
pub unsafe fn MotherBrain_MakeHerLeanDown(game: &mut Game) -> uint8 {
    let mut mbn_var_02: uint16 = (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_02;
    if mbn_var_02 != 0 {
        if mbn_var_02 as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
            return 1 as uint8;
        }
    } else {
        MotherBrain_SetBodyInstrs(
            addr_kMotherBrain_Ilist_99F2 as ::core::ffi::c_int as uint16,
            game,
        );
    }
    return 0 as uint8;
}
pub unsafe fn MotherBrain_HandleWalking(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if (*E).mbn_var_02 == 0 {
        let mut v2: uint16 = 0;
        let mut current_block_15: u64;
        let mut mbn_var_07: uint16 = (*E).mbn_var_07;
        if mbn_var_07 != 0 {
            v2 = 0;
            v2 = (mbn_var_07 as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
            (*E).mbn_var_07 = v2;
            if v2 as ::core::ffi::c_int >= 0x100 as ::core::ffi::c_int {
                current_block_15 = 2842450993777580018;
            } else {
                if (*E).base.x_pos as ::core::ffi::c_int - 48 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    return;
                }
                current_block_15 = 3512920355445576850;
            }
        } else {
            (*E).mbn_var_07 = 1 as uint16;
            if (*E).base.x_pos as ::core::ffi::c_int - 48 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                MotherBrain_SetBodyInstrs(
                    addr_kMotherBrain_Ilist_98C6 as ::core::ffi::c_int as uint16,
                    game,
                );
                return;
            }
            current_block_15 = 3512920355445576850;
        }
        match current_block_15 {
            3512920355445576850 => {
                if (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x5e5 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0xfff as ::core::ffi::c_int)
                    - 4032 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    return;
                }
            }
            _ => {}
        }
        (*E).mbn_var_07 = 128 as uint16;
        if (*E).base.x_pos as ::core::ffi::c_int - 128 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            MotherBrain_SetBodyInstrs(
                addr_kMotherBrain_Ilist_97A4 as ::core::ffi::c_int as uint16,
                game,
            );
        }
        return;
    }
}
pub unsafe fn ShitroidInCutscene_Init(game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_DisableSamusColl as ::core::ffi::c_int
            | kEnemyProps_BlockPlasmaBeam as ::core::ffi::c_int))
        as uint16;
    (*E).base.palette_index = 3584 as uint16;
    (*E).base.current_instruction = addr_kShitroid_Ilist_CFA2 as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).sice_var_04 = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).sice_var_E = 10 as uint16;
    (*E).base.vram_tiles_index = 160 as uint16;
    (*E).base.x_pos = 320 as uint16;
    (*E).base.y_pos = 96 as uint16;
    (*E).sice_var_B = 0 as uint16;
    (*E).sice_var_C = 0 as uint16;
    (*E).sice_var_09 = 0 as uint16;
    (*E).sice_var_A = (0xa9c7cc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).sice_var_F = 248 as uint16;
    (*E).sice_var_0F = (0xa9f683 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    WriteColorsToPalette(
        0x1e2 as uint16,
        0xa9 as uint8,
        (addr_kMotherBrainPalette_2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
        0xf as uint16,
        game,
    );
}
pub unsafe fn CallShitroidCutsceneFuncA(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        11125133 => return,
        11126732 => {
            ShitroidInCutscene_DashOntoScreen(k, game);
            return;
        }
        11126764 => {
            ShitroidInCutscene_CurveTowardsBrain(k, game);
            return;
        }
        11126801 => {
            ShitroidInCutscene_GetIntoFace(k, game);
            return;
        }
        11126865 => {
            ShitroidInCutscene_LatchOntoBrain(k, game);
            return;
        }
        11126905 => {
            ShitroidInCutscene_SetMotherBrainToStumbleBack(k, game);
            return;
        }
        11126921 => {
            ShitroidInCutscene_ActivateRainbowBeam(k, game);
            return;
        }
        11127010 => {
            ShitroidInCutscene_BrainTurnsToCorpse(k, game);
            return;
        }
        11127061 => {
            ShitroidInCutscene_StopDraining(k, game);
            return;
        }
        11127115 => {
            ShitroidInCutscene_LetGoAndSpawnDust(k, game);
            return;
        }
        11127129 => {
            ShitroidInCutscene_MoveUpToCeiling(k, game);
            return;
        }
        11127235 => {
            ShitroidInCutscene_MoveToSamus(k, game);
            return;
        }
        11127398 => {
            ShitroidInCutscene_LatchOntoSamus(k, game);
            return;
        }
        11127418 => {
            ShitroidInCutscene_HealSamusToFullHealth(k, game);
            return;
        }
        11127485 => {
            ShitroidInCutscene_IdleUntilToNoHealth(k, game);
            return;
        }
        11127571 => {
            ShitroidInCutscene_ReleaseSamus(k, game);
            return;
        }
        11127597 => {
            ShitroidInCutscene_StareDownMotherBrain(k, game);
            return;
        }
        11127638 => {
            ShitroidInCutscene_FlyOffScreen(k, game);
            return;
        }
        11127675 => {
            ShitroidInCutscene_MoveToFinalChargeStart(k, game);
            return;
        }
        11127731 => {
            ShitroidInCutscene_InitiateFinalCharge(k, game);
            return;
        }
        11127768 => {
            ShitroidInCutscene_FinalCharge(k, game);
            return;
        }
        11127870 => {
            ShitroidInCutscene_ShitroidFinalBelow(k, game);
            return;
        }
        11127904 => {
            ShitroidInCutscene_PlaySamusTheme(k, game);
            return;
        }
        11127935 => {
            ShitroidInCutscene_PrepareSamusHyperbeam(k, game);
            return;
        }
        11127961 => {
            ShitroidInCutscene_DeathSequence(k, game);
            return;
        }
        11128000 => {
            ShitroidInCutscene_UnloadShitroid(k, game);
            return;
        }
        11128030 => {
            ShitroidInCutscene_LetSamusRainbowMore(k, game);
            return;
        }
        11128048 => {
            ShitroidInCutscene_FinishCutscene(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CallShitroidCutsceneFunc0F(mut ea: uint32, game: &mut Game) {
    match ea {
        11127484 => return,
        11127960 => return,
        11138691 => {
            Shitroid_HandleCutscenePalette(game);
            return;
        }
        11138703 => {
            Shitroid_HandleCutscenePalette_LowHealth(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn ShitroidInCutscene_Main(game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.shake_timer = 0 as uint16;
    CallShitroidCutsceneFuncA(
        ((*E).sice_var_A as ::core::ffi::c_int | 0xa90000 as ::core::ffi::c_int) as uint32,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    MoveEnemyWithVelocity(game);
    ShitroidInCutscene_Flashing(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    ShitroidInCutscene_HandleHealthBasedPalette(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    let mut r18: uint16 = (*Get_ShitdroidInCutscene(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .sice_var_0F;
    CallShitroidCutsceneFunc0F(
        (r18 as ::core::ffi::c_int | 0xa90000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn ShitroidInCutscene_Flashing(mut k: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    v1 = 3584 as int16;
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    let mut sice_var_06: uint16 = (*E).sice_var_06;
    if sice_var_06 != 0 {
        let mut v4: uint16 =
            (sice_var_06 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        (*E).sice_var_06 = v4;
        if v4 as ::core::ffi::c_int & 2 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            v1 = 0 as int16;
        }
    }
    (*E).base.palette_index = v1 as uint16;
}
pub unsafe fn ShitroidInCutscene_HandleCry(game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(0 as ::core::ffi::c_int, game);
    if (*E).sice_var_14 != 0 {
        (*E).sice_var_14 = 0 as uint16;
        QueueSfx2_Max6(0x72 as uint16, game);
    }
}
pub unsafe fn ShitroidInCutscene_DashOntoScreen(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    if (*E).sice_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).sice_var_0A = -(10240 as ::core::ffi::c_int) as uint16;
        (*E).sice_var_0B = 2560 as uint16;
        (*E).sice_var_A = (0xa9c7ec as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sice_var_F = 10 as uint16;
        ShitroidInCutscene_CurveTowardsBrain(k, game);
    }
}
pub unsafe fn ShitroidInCutscene_CurveTowardsBrain(mut k: uint16, game: &mut Game) {
    ShitroidInCutscene_UpdateSpeedAndAngle(
        k,
        -(384 as ::core::ffi::c_int) as uint16,
        -(20480 as ::core::ffi::c_int) as uint16,
        2560 as uint16,
        game,
    );
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitroidInCutscene(k as ::core::ffi::c_int, game);
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    if (*E).sice_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).sice_var_A = (0xa9c811 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sice_var_F = 9 as uint16;
    }
}
pub unsafe fn ShitroidInCutscene_GetIntoFace(mut k: uint16, game: &mut Game) {
    let mut v3: bool = false;
    ShitroidInCutscene_UpdateSpeedAndAngle(
        k,
        -(1536 as ::core::ffi::c_int) as uint16,
        -(32256 as ::core::ffi::c_int) as uint16,
        3584 as uint16,
        game,
    );
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitroidInCutscene(0x40 as ::core::ffi::c_int, game);
    let mut rect: Rect16U = Rect16U {
        x: (*E).base.x_pos,
        y: (*E).base.y_pos,
        w: 4 as uint16,
        h: 4 as uint16,
    };
    if Shitroid_Func_2(k, rect, game) == 0 || {
        E = Get_ShitroidInCutscene(k as ::core::ffi::c_int, game);
        v3 = (((*E).sice_var_F as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int;
        (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
        v3 as ::core::ffi::c_int != 0
    } {
        (*Get_ShitroidInCutscene(k as ::core::ffi::c_int, game)).sice_var_A =
            (0xa9c851 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        SomeMotherBrainScripts(1 as uint16, game);
    }
}
pub unsafe fn ShitroidInCutscene_LatchOntoBrain(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitroidInCutscene(0x40 as ::core::ffi::c_int, game);
    let mut rect: Rect16U = Rect16U {
        x: (*E).base.x_pos,
        y: ((*E).base.y_pos as ::core::ffi::c_int - 24 as ::core::ffi::c_int) as uint16,
        w: 8 as uint16,
        h: 8 as uint16,
    };
    Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0 as uint16, rect.x, rect.y, game);
    if Shitroid_Func_2(k, rect, game) == 0 {
        (*Get_ShitroidInCutscene(k as ::core::ffi::c_int, game)).sice_var_A =
            (0xa9c879 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn ShitroidInCutscene_SetMotherBrainToStumbleBack(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitroidInCutscene(0 as ::core::ffi::c_int, game);
    MotherBrain_MakeWalkBackwards(
        ((*E).base.x_pos as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        2 as uint16,
        game,
    );
    (*Get_ShitroidInCutscene(k as ::core::ffi::c_int, game)).sice_var_A =
        (0xa9c889 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    ShitroidInCutscene_ActivateRainbowBeam(k, game);
}
pub unsafe fn ShitroidInCutscene_ActivateRainbowBeam(mut k: uint16, game: &mut Game) {
    let mut E1: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(0x40 as ::core::ffi::c_int, game);
    if Shitroid_AccelerateTowardsPoint(
        k,
        0x200 as uint16,
        (*E1).base.x_pos,
        ((*E1).base.y_pos as ::core::ffi::c_int - 24 as ::core::ffi::c_int) as uint16,
        game,
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        let mut E: *mut Enemy_ShitroidInCutscene =
            Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
        (*E).sice_var_B = 0 as uint16;
        (*E).sice_var_C = 0 as uint16;
        (*E).base.x_pos = (*E1).base.x_pos;
        (*E).base.y_pos =
            ((*E1).base.y_pos as ::core::ffi::c_int - 24 as ::core::ffi::c_int) as uint16;
        Enemy_SetInstrList(
            k,
            addr_kShitroid_Ilist_CFB8 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).sice_var_A = (0xa9c8e2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sice_var_E = 1 as uint16;
        let mut E0: *mut Enemy_ShitroidInCutscene =
            Get_ShitdroidInCutscene(0 as ::core::ffi::c_int, game);
        (*E0).sice_var_A =
            (0xa9be38 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        QueueSfx1_Max6(0x40 as uint16, game);
        (*E0).sice_var_16 = 1 as uint16;
    }
}
pub unsafe fn ShitroidInCutscene_BrainTurnsToCorpse(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    let mut E1: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(0x40 as ::core::ffi::c_int, game);
    let mut v2: ::core::ffi::c_int = ((*E).base.frame_counter as ::core::ffi::c_int
        & 6 as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int;
    (*E).base.x_pos = ((*E1).base.x_pos as ::core::ffi::c_int
        + g_word_A993BB[v2 as usize] as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E1).base.y_pos as ::core::ffi::c_int
        + g_word_A993C3[v2 as usize] as ::core::ffi::c_int
        - 24 as ::core::ffi::c_int) as uint16;
    if (*Get_ShitdroidInCutscene(0 as ::core::ffi::c_int, game)).sice_var_1F != 0 {
        (*E).sice_var_A = (0xa9c915 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sice_var_F = 64 as uint16;
    }
}
pub unsafe fn ShitroidInCutscene_StopDraining(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitroidInCutscene(k as ::core::ffi::c_int, game);
    let mut E1: *mut Enemy_ShitroidInCutscene =
        Get_ShitroidInCutscene(0x40 as ::core::ffi::c_int, game);
    (*E).base.x_pos = (*E1).base.x_pos;
    (*E).base.y_pos = ((*E1).base.y_pos as ::core::ffi::c_int - 24 as ::core::ffi::c_int) as uint16;
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    if (*E).sice_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        Enemy_SetInstrList(
            k,
            addr_kShitroid_Ilist_CFA2 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).sice_var_E = 10 as uint16;
        (*E).sice_var_A = (0xa9c94b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sice_var_F = 32 as uint16;
        (*E).sice_var_B = 0 as uint16;
        (*E).sice_var_C = 0 as uint16;
    }
}
pub unsafe fn ShitroidInCutscene_LetGoAndSpawnDust(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    if (*E).sice_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        ShitroidInCutscene_SpawnThreeDustClouds(game);
        (*E).sice_var_A = addr_loc_A9C959 as ::core::ffi::c_int as uint16;
    }
    ShitroidInCutscene_MoveUpToCeiling(k, game);
}
pub unsafe fn ShitroidInCutscene_MoveUpToCeiling(mut k: uint16, game: &mut Game) {
    let mut rect: Rect16U = Rect16U {
        x: (*Get_ShitdroidInCutscene(0x40 as ::core::ffi::c_int, game))
            .base
            .x_pos,
        y: 0 as uint16,
        w: 4 as uint16,
        h: 4 as uint16,
    };
    Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0 as uint16, rect.x, rect.y, game);
    if Shitroid_Func_2(k, rect, game) == 0 {
        SomeMotherBrainScripts(4 as uint16, game);
        let mut E: *mut Enemy_ShitroidInCutscene =
            Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
        (*E).sice_var_A = (0xa9c9c3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sice_var_0E = addr_stru_A9CA24 as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn ShitroidInCutscene_SpawnOneDustCloudAt(
    mut xd: uint16,
    mut yd: uint16,
    game: &mut Game,
) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitroidInCutscene(0x40 as ::core::ffi::c_int, game);
    game.eproj_spawn_pt = Point16U {
        x: ((*E).base.x_pos as ::core::ffi::c_int + xd as ::core::ffi::c_int) as uint16,
        y: ((*E).base.y_pos as ::core::ffi::c_int + yd as ::core::ffi::c_int) as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        9 as uint16,
        game,
    );
}
pub unsafe fn ShitroidInCutscene_SpawnThreeDustClouds(game: &mut Game) {
    ShitroidInCutscene_SpawnOneDustCloudAt(
        -(16 as ::core::ffi::c_int) as uint16,
        -(8 as ::core::ffi::c_int) as uint16,
        game,
    );
    ShitroidInCutscene_SpawnOneDustCloudAt(
        0 as uint16,
        -(16 as ::core::ffi::c_int) as uint16,
        game,
    );
    ShitroidInCutscene_SpawnOneDustCloudAt(
        16 as uint16,
        -(8 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn CallShitroidMoveFunc(
    mut ea: uint32,
    mut k: uint16,
    mut j: uint16,
    mut r18: uint16,
    mut r20: uint16,
    game: &mut Game,
) {
    match ea {
        11138143 => {
            Shitroid_GraduallyAccelerateTowards0x8(k, j, r18, r20, game);
            return;
        }
        11138150 => {
            Shitroid_GraduallyAccelerateTowards0x10(k, j, r18, r20, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn ShitroidInCutscene_MoveToSamus(mut k: uint16, game: &mut Game) {
    let mut v4: int16 = 0;
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    (*E).sice_var_04 = 0 as uint16;
    (*E).sice_var_09 = 1 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0xfff as ::core::ffi::c_int
        >= 0xfa0 as ::core::ffi::c_int
    {
        QueueSfx2_Max6(0x52 as uint16, game);
    }
    let mut sice_var_0E: uint16 = (*E).sice_var_0E;
    let mut v3: *const uint16 = RomPtr_A9(sice_var_0E as uint16_t, game) as *const uint16;
    let mut rect: Rect16U = Rect16U {
        x: *v3.offset(0 as ::core::ffi::c_int as isize),
        y: *v3.offset(1 as ::core::ffi::c_int as isize),
        w: 4 as uint16,
        h: 4 as uint16,
    };
    CallShitroidMoveFunc(
        (*v3.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            | 0xa90000 as ::core::ffi::c_int) as uint32,
        k,
        *v3.offset(2 as ::core::ffi::c_int as isize),
        rect.x,
        rect.y,
        game,
    );
    if Shitroid_Func_2(k, rect, game) == 0 {
        v4 = *(RomPtr_A9(sice_var_0E as uint16_t, game) as *mut uint16)
            .offset(4 as ::core::ffi::c_int as isize) as int16;
        if (v4 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            (*E).sice_var_A = v4 as uint16;
        } else {
            (*E).sice_var_0E =
                ((*E).sice_var_0E as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn ShitroidInCutscene_LatchOntoSamus(mut k: uint16, game: &mut Game) {
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        k,
        0 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 20 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn ShitroidInCutscene_HealSamusToFullHealth(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    (*E).sice_var_04 = 0 as uint16;
    ShitroidInCutscene_HandleCry(game);
    let mut v2: ::core::ffi::c_int = ((*E).base.frame_counter as ::core::ffi::c_int
        & 6 as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int;
    (*E).base.x_pos = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + g_word_A993BB[v2 as usize] as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + g_word_A993C3[v2 as usize] as ::core::ffi::c_int
        - 20 as ::core::ffi::c_int) as uint16;
    if Samus_HealDueToShitroid(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9d6 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9d4 as ::core::ffi::c_int as isize)
            as *mut uint16);
        (*E).sice_var_A = (0xa9cabd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sice_var_0F =
            (0xa9cabc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn ShitroidInCutscene_IdleUntilToNoHealth(mut k: uint16, game: &mut Game) {
    ShitroidInCutscene_HandleCry(game);
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    let mut sice_var_06: uint16 = (*E).sice_var_06;
    if sice_var_06 != 0 {
        let mut v3: ::core::ffi::c_int = (sice_var_06 as ::core::ffi::c_int
            & 6 as ::core::ffi::c_int)
            >> 1 as ::core::ffi::c_int;
        (*E).base.x_pos = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int * g_word_A993BB[v3 as usize] as ::core::ffi::c_int)
            as uint16;
        (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int * g_word_A993C3[v3 as usize] as ::core::ffi::c_int
            - 20 as ::core::ffi::c_int) as uint16;
    }
    if (*E).base.health == 0 {
        (*E).base.health = 320 as uint16;
        (*E).sice_var_A = (0xa9cb13 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sice_var_E = 10 as uint16;
        (*E).sice_var_D = 0 as uint16;
        (*E).sice_var_0F =
            (0xa9f68f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sice_var_04 = 1 as uint16;
        (*E).sice_var_09 = 0 as uint16;
    }
}
pub unsafe fn ShitroidInCutscene_ReleaseSamus(mut k: uint16, game: &mut Game) {
    QueueSfx2_Max6(0x72 as uint16, game);
    (*Get_ShitdroidInCutscene(0x40 as ::core::ffi::c_int, game)).sice_var_0B = 1 as uint16;
    (*Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game)).sice_var_A =
        (0xa9cb2d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*Get_ShitdroidInCutscene(0 as ::core::ffi::c_int, game)).sice_var_A =
        (0xa9c18e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    ShitroidInCutscene_StareDownMotherBrain(k, game);
}
pub unsafe fn ShitroidInCutscene_StareDownMotherBrain(mut k: uint16, game: &mut Game) {
    let mut rect: Rect16U = Rect16U {
        x: (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int) as uint16,
        y: 96 as uint16,
        w: 4 as uint16,
        h: 4 as uint16,
    };
    Shitroid_GraduallyAccelerateTowards0x10(k, 0 as uint16, rect.x, rect.y, game);
    if Shitroid_Func_2(k, rect, game) == 0 {
        (*Get_ShitroidInCutscene(k as ::core::ffi::c_int, game)).sice_var_A =
            (0xa9cb56 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn ShitroidInCutscene_FlyOffScreen(mut k: uint16, game: &mut Game) {
    let mut rect: Rect16U = Rect16U {
        x: 272 as uint16,
        y: 64 as uint16,
        w: 4 as uint16,
        h: 4 as uint16,
    };
    Shitroid_GraduallyAccelerateTowards0x10(k, 0 as uint16, rect.x, rect.y, game);
    if Shitroid_Func_2(k, rect, game) == 0 {
        (*Get_ShitroidInCutscene(k as ::core::ffi::c_int, game)).sice_var_A =
            (0xa9cb7b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn ShitroidInCutscene_MoveToFinalChargeStart(mut k: uint16, game: &mut Game) {
    let mut rect: Rect16U = Rect16U {
        x: 305 as uint16,
        y: 160 as uint16,
        w: 4 as uint16,
        h: 4 as uint16,
    };
    Shitroid_GraduallyAccelerateTowards0x10(k, 0 as uint16, rect.x, rect.y, game);
    if Shitroid_Func_2(k, rect, game) == 0 {
        let mut E: *mut Enemy_ShitroidInCutscene =
            Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
        (*E).base.health = 79 as uint16;
        (*Get_ShitdroidInCutscene(0x40 as ::core::ffi::c_int, game)).sice_var_0B = 0 as uint16;
        (*Get_ShitdroidInCutscene(0 as ::core::ffi::c_int, game)).sice_var_A =
            (0xa9c19a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sice_var_A = (0xa9cbb3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn ShitroidInCutscene_InitiateFinalCharge(mut k: uint16, game: &mut Game) {
    let mut rect: Rect16U = Rect16U {
        x: 290 as uint16,
        y: 128 as uint16,
        w: 4 as uint16,
        h: 4 as uint16,
    };
    Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0xa as uint16, rect.x, rect.y, game);
    if Shitroid_Func_2(k, rect, game) == 0 {
        (*Get_ShitroidInCutscene(k as ::core::ffi::c_int, game)).sice_var_A =
            (0xa9cbd8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn ShitroidInCutscene_FinalCharge(mut k: uint16, game: &mut Game) {
    let mut E1: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(0x40 as ::core::ffi::c_int, game);
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        k,
        0xc as uint16,
        (*E1).base.x_pos,
        ((*E1).base.y_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16,
        game,
    );
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    if (*E).base.health == 0 {
        (*E).sice_var_0F =
            (0xa9cc98 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.vram_tiles_index = 4256 as uint16;
        QueueSfx3_Max6(0x19 as uint16, game);
        TurnOffLightsForShitroidDeath(game);
        Enemy_SetInstrList(
            k,
            addr_kShitroid_Ilist_CFCE as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).sice_var_B = 0 as uint16;
        (*E).sice_var_C = 0 as uint16;
        (*Get_ShitdroidInCutscene(0 as ::core::ffi::c_int, game)).sice_var_A =
            addr_locret_A9C18D as ::core::ffi::c_int as uint16;
        QueueMusic_Delayed8(0 as uint16, game);
        (*E).sice_var_A = (0xa9cc3e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sice_var_F = 16 as uint16;
        (*E).sice_var_10 = (*E).base.x_pos;
        (*E).sice_var_11 = (*E).base.y_pos;
        ShitroidInCutscene_ShitroidFinalBelow(k, game);
    }
}
pub unsafe fn ShitroidInCutscene_ShitroidFinalBelow(mut k: uint16, game: &mut Game) {
    ShitroidInCutscene_Shake(k, game);
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    if (*E).sice_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.x_pos = (*E).sice_var_10;
        (*E).base.y_pos = (*E).sice_var_11;
        (*E).sice_var_A = (0xa9cc60 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sice_var_F = 56 as uint16;
        ShitroidInCutscene_PlaySamusTheme(k, game);
    }
}
pub unsafe fn ShitroidInCutscene_PlaySamusTheme(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitroidInCutscene(k as ::core::ffi::c_int, game);
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    if (*E).sice_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        QueueMusic_Delayed8(0xff48 as uint16, game);
        QueueMusic_Delayed8(5 as uint16, game);
        (*E).sice_var_A = (0xa9cc7f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sice_var_F = 12 as uint16;
        ShitroidInCutscene_PrepareSamusHyperbeam(k, game);
    }
}
pub unsafe fn ShitroidInCutscene_PrepareSamusHyperbeam(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    if (*E).sice_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        CallSomeSamusCode(0x19 as uint16, game);
        (*Get_ShitdroidInCutscene(0x40 as ::core::ffi::c_int, game)).sice_var_16 =
            (0xa9cd30 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sice_var_A = (0xa9cc99 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn ShitroidInCutscene_DeathSequence(mut k: uint16, game: &mut Game) {
    HandleSamusRainbowPaletteAnimation(k, game);
    ShitroidInCutscene_AccelerateDownwards(k, game);
    if ShitroidInCutscene_FadeShitroidToBlack(k, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        let mut E: *mut Enemy_ShitroidInCutscene =
            Get_ShitroidInCutscene(k as ::core::ffi::c_int, game);
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
        (*E).sice_var_A = (0xa9ccc0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sice_var_F = 128 as uint16;
    } else {
        ShitroidInCutscene_HandleShitroidDeathExplosions(k, game);
        ShitroidInCutscene_HandleEnemyBlinking(k, game);
    };
}
pub unsafe fn ShitroidInCutscene_UnloadShitroid(mut k: uint16, game: &mut Game) {
    HandleSamusRainbowPaletteAnimation(k, game);
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitroidInCutscene(k as ::core::ffi::c_int, game);
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    let mut v2: bool = (*E).sice_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0
        && ProcessSpriteTilesTransfers(
            0xa9 as uint8,
            addr_stru_A98FC7 as ::core::ffi::c_int as uint16,
            game,
        ) as ::core::ffi::c_int
            != 0
    {
        (*E).sice_var_A = (0xa9ccde as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sice_var_F = 176 as uint16;
        ShitroidInCutscene_LetSamusRainbowMore(k, game);
    }
}
pub unsafe fn ShitroidInCutscene_LetSamusRainbowMore(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    if (*E).sice_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).sice_var_A = (0xa9ccf0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*Get_ShitdroidInCutscene(0 as ::core::ffi::c_int, game)).sice_var_37 = 0 as uint16;
        ShitroidInCutscene_FinishCutscene(k, game);
    }
}
pub unsafe fn ShitroidInCutscene_FinishCutscene(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(0 as ::core::ffi::c_int, game);
    let mut v2: uint16 =
        ((*E).sice_var_37 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    (*E).sice_var_37 = v2;
    if MotherBrain_Phase3_TurnLightsBackOn(
        (v2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        game,
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*E).sice_var_A = (0xa9c1cf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        CallSomeSamusCode(0x17 as uint16, game);
        SomeMotherBrainScripts(3 as uint16, game);
        let fresh12 = &raw mut (*Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game))
            .base
            .properties;
        fresh12.write_unaligned(
            (fresh12.read_unaligned() as ::core::ffi::c_int
                | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16,
        );
        (*Get_ShitdroidInCutscene(0x40 as ::core::ffi::c_int, game)).sice_var_0A = 0 as uint16;
    }
}
pub unsafe fn CallSamusRainbowPaletteFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        11128112 => {
            SamusRainbowPaletteFunc_ActivateWhenEnemyLow(k, game);
            return;
        }
        11128139 => {
            SamusRainbowPaletteFunc_GraduallySlowDown(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn HandleSamusRainbowPaletteAnimation(mut k: uint16, game: &mut Game) {
    let mut r18: uint16 = (*Get_ShitdroidInCutscene(0x40 as ::core::ffi::c_int, game)).sice_var_16;
    CallSamusRainbowPaletteFunc(
        (r18 as ::core::ffi::c_int | 0xa90000 as ::core::ffi::c_int) as uint32,
        k,
        game,
    );
}
pub unsafe fn SamusRainbowPaletteFunc_ActivateWhenEnemyLow(mut k: uint16, game: &mut Game) {
    if ((*Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game))
        .base
        .y_pos as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        CallSomeSamusCode(0x16 as uint16, game);
        (*Get_ShitdroidInCutscene(0x40 as ::core::ffi::c_int, game)).sice_var_16 =
            (0xa9cd4b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn SamusRainbowPaletteFunc_GraduallySlowDown(game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(0x40 as ::core::ffi::c_int, game);
    let mut sice_var_1D: uint16 = (*E).sice_var_1D;
    (*E).sice_var_1D = (sice_var_1D as ::core::ffi::c_int + 768 as ::core::ffi::c_int) as uint16;
    if sice_var_1D as ::core::ffi::c_int >= 0xfd00 as ::core::ffi::c_int {
        let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 9 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v2 = 10 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) = v2;
    }
}
pub unsafe fn ShitroidInCutscene_FadeShitroidToBlack(mut k: uint16, game: &mut Game) -> uint8 {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    if (*E).base.y_pos as ::core::ffi::c_int - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        return 0 as uint8;
    }
    v2 = ((*E).sice_var_0C as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
    if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        (*E).sice_var_0C = v2 as uint16;
        return 0 as uint8;
    }
    (*E).sice_var_0C = 8 as uint16;
    let mut v4: uint16 =
        ((*E).sice_var_0D as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    if (*E).sice_var_0D as ::core::ffi::c_int - 6 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        return 1 as uint8;
    }
    (*E).sice_var_0D = v4;
    WriteColorsToPalette(
        0x1e2 as uint16,
        0xad as uint8,
        *(game.g_rom.offset(
            (((0xade8e2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xade8e2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v4 as isize),
        0xe as uint16,
        game,
    );
    return 0 as uint8;
}
pub unsafe fn ShitroidInCutscene_HandleShitroidDeathExplosions(mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    v2 = ((*E).sice_var_08 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
    if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).sice_var_08 = 4 as uint16;
        let mut v3: uint16 =
            ((*E).sice_var_07 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        if (*E).sice_var_07 as ::core::ffi::c_int - 9 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v3 = 0 as uint16;
        }
        (*E).sice_var_07 = v3;
        let mut v4: ::core::ffi::c_int = (4 as ::core::ffi::c_int * v3 as ::core::ffi::c_int)
            as uint16 as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        game.eproj_spawn_pt = Point16U {
            x: ((*E).base.x_pos as ::core::ffi::c_int
                + *(game.g_rom.offset(
                    (((0xa9cdfc as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa9cdfc as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v4 as isize) as ::core::ffi::c_int) as uint16,
            y: ((*E).base.y_pos as ::core::ffi::c_int
                + *(game.g_rom.offset(
                    (((0xa9cdfc as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa9cdfc as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v4 + 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int) as uint16,
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
            3 as uint16,
            game,
        );
        QueueSfx3_Max3(0x13 as uint16, game);
    } else {
        (*E).sice_var_08 = v2 as uint16;
    };
}
pub unsafe fn ShitroidInCutscene_HandleEnemyBlinking(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitroidInCutscene(k as ::core::ffi::c_int, game);
    if (*E).base.frame_counter as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_Invisible as ::core::ffi::c_int))
            as uint16;
    } else {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn ShitroidInCutscene_AccelerateDownwards(mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitroidInCutscene(k as ::core::ffi::c_int, game);
    v2 = (abs16((*E).sice_var_B) as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as int16;
    if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        v2 = 0 as int16;
    }
    (*E).sice_var_B = (if (*E).sice_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    {
        -(v2 as ::core::ffi::c_int)
    } else {
        v2 as ::core::ffi::c_int
    }) as uint16;
    (*E).sice_var_C = ((*E).sice_var_C as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn ShitroidInCutscene_HandleHealthBasedPalette(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    if (*E).sice_var_09 != 0 {
        let mut health: uint16 = (*E).base.health;
        if health as ::core::ffi::c_int - 2560 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v3: uint16 = 2 as uint16;
            if health as ::core::ffi::c_int - 2240 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v3 = 4 as uint16;
                if health as ::core::ffi::c_int - 1920 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    v3 = 6 as uint16;
                    if health as ::core::ffi::c_int - 1600 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0
                    {
                        v3 = 8 as uint16;
                        if health as ::core::ffi::c_int - 1280 as ::core::ffi::c_int
                            & 0x8000 as ::core::ffi::c_int
                            != 0
                        {
                            v3 = 10 as uint16;
                            if health as ::core::ffi::c_int - 960 as ::core::ffi::c_int
                                & 0x8000 as ::core::ffi::c_int
                                != 0
                            {
                                v3 = 12 as uint16;
                                if health as ::core::ffi::c_int - 640 as ::core::ffi::c_int
                                    & 0x8000 as ::core::ffi::c_int
                                    != 0
                                {
                                    v3 = 14 as uint16;
                                }
                            }
                        }
                    }
                }
            }
            let mut v5: uint16 = v3;
            let mut v4: uint16 = *(game.g_rom.offset(
                (((0xade7e2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xade7e2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            WriteColorsToPalette(0x1e2 as uint16, 0xad as uint8, v4, 4 as uint16, game);
            WriteColorsToPalette(
                0x1f4 as uint16,
                0xad as uint8,
                (v4 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16,
                5 as uint16,
                game,
            );
            WriteColorsToPalette(
                0x1ea as uint16,
                0xad as uint8,
                *(game.g_rom.offset(
                    (((0xade882 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xade882 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
                5 as uint16,
                game,
            );
        }
    }
}
pub unsafe fn ShitroidInCutscene_Shake(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    (*E).sice_var_C = (*E).sice_var_C.wrapping_sub(1);
    let mut v2: ::core::ffi::c_int = ((*E).base.frame_counter as ::core::ffi::c_int
        & 6 as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int;
    (*E).base.x_pos = ((*E).sice_var_10 as ::core::ffi::c_int
        + g_word_A993BB[v2 as usize] as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E).sice_var_11 as ::core::ffi::c_int
        + g_word_A993C3[v2 as usize] as ::core::ffi::c_int) as uint16;
}
pub unsafe fn ShitroidInCutscene_Touch(game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene = Get_ShitroidInCutscene(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).sice_var_A as ::core::ffi::c_int
        == 0xa9ca66 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        if Shitroid_AccelerateTowardsPoint(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            0x10 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16),
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 20 as ::core::ffi::c_int) as uint16,
            game,
        ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            (*E).sice_var_B = 0 as uint16;
            (*E).sice_var_C = 0 as uint16;
            (*E).sice_var_A =
                (0xa9ca7a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn ShitroidInCutscene_UpdateSpeedAndAngle(
    mut k: uint16,
    mut r18: uint16,
    mut r20: uint16,
    mut r22: uint16,
    game: &mut Game,
) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    if r22 as ::core::ffi::c_int != (*E).sice_var_0B as ::core::ffi::c_int {
        if (r22 as ::core::ffi::c_int - (*E).sice_var_0B as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            let mut v3: uint16 =
                ((*E).sice_var_0B as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
            if v3 as ::core::ffi::c_int - r22 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v3 = r22;
            }
            (*E).sice_var_0B = v3;
        } else {
            let mut v2: uint16 =
                ((*E).sice_var_0B as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
            if v2 as ::core::ffi::c_int - r22 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v2 = r22;
            }
            (*E).sice_var_0B = v2;
        }
    }
    let mut v4: uint16 = 0;
    let mut current_block_13: u64;
    if r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        v4 = ((*E).sice_var_0A as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
        if v4 as ::core::ffi::c_int - r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        {
            current_block_13 = 17187448977823727056;
        } else {
            current_block_13 = 12039483399334584727;
        }
    } else {
        v4 = ((*E).sice_var_0A as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
        if v4 as ::core::ffi::c_int - r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        {
            current_block_13 = 17187448977823727056;
        } else {
            current_block_13 = 12039483399334584727;
        }
    }
    match current_block_13 {
        17187448977823727056 => {
            v4 = r20;
        }
        _ => {}
    }
    (*E).sice_var_0A = v4;
    r18 = *(&raw mut v4 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as uint16;
    (*E).sice_var_B = Math_MultBySin((*E).sice_var_0B, r18);
    (*E).sice_var_C = Math_MultByCos((*E).sice_var_0B, r18);
}
pub unsafe fn Shitroid_Instr_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return addr_kShitroid_Ilist_CFA2 as ::core::ffi::c_int as uintptr_t as *const uint16;
}
pub unsafe fn Shitroid_Instr_2(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return addr_kShitroid_Ilist_CFB8 as ::core::ffi::c_int as uintptr_t as *const uint16;
}
pub unsafe fn MotherBrain_Pal_ProcessInvincibility(game: &mut Game) {
    if (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_00 as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        let mut flash_timer: uint16 = (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game))
            .base
            .flash_timer;
        if flash_timer as ::core::ffi::c_int >> 1 as ::core::ffi::c_int != 0 {
            if flash_timer as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
                let mut i: ::core::ffi::c_int = 28 as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    let mut v2: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
                    let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc000 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v2 + 129 as ::core::ffi::c_int) as isize);
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc000 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v2 + 145 as ::core::ffi::c_int) as isize) = v3;
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc000 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v2 + 177 as ::core::ffi::c_int) as isize) = v3;
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xc000 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v2 + 65 as ::core::ffi::c_int) as isize) = v3;
                    i -= 2 as ::core::ffi::c_int;
                }
            }
        }
    }
}
pub unsafe fn MotherBrain_Pal_BeginScreenFlash(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_0E = addr_stru_A9D046 as ::core::ffi::c_int as uint16;
    (*E).mbn_var_0F = 1 as uint16;
}
pub unsafe fn MotherBrain_Pal_EndScreenFlash(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_0E = 0 as uint16;
    (*E).mbn_var_0F = 0 as uint16;
    MotherBrain_Pal_WriteRoomPal(addr_word_A9D082 as ::core::ffi::c_int as uint16, game);
}
pub unsafe fn MotherBrain_Pal_HandleRoomPal(game: &mut Game) {
    let mut j: uint16 = HandleMotherBrainInstructionList(
        (((&raw mut game.g_ram as *mut uint8).offset(0x7800 as ::core::ffi::c_int as isize)
            as *mut ExtraEnemyRam7800 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            + 28 as ::core::ffi::c_long) as uint16,
        game,
    );
    if j != 0 {
        MotherBrain_Pal_WriteRoomPal(j, game);
    }
}
pub unsafe fn MotherBrain_Pal_WriteRoomPal(mut j: uint16, game: &mut Game) {
    WriteColorsToPalette(0x68 as uint16, 0xa9 as uint8, j, 12 as uint16, game);
    WriteColorsToPalette(
        0xa6 as uint16,
        0xa9 as uint8,
        (j as ::core::ffi::c_int + 24 as ::core::ffi::c_int) as uint16,
        12 as uint16,
        game,
    );
    WriteColorsToPalette(
        0xe6 as uint16,
        0xa9 as uint8,
        (j as ::core::ffi::c_int + 24 as ::core::ffi::c_int) as uint16,
        12 as uint16,
        game,
    );
}
pub unsafe fn HandleMotherBrainInstructionList(mut a: uint16, game: &mut Game) -> uint16 {
    let mut current_block: u64;
    let mut st: *mut MotherBrainInstrExecState = (&raw mut game.g_ram as *mut uint8)
        .offset(a as isize) as *mut uint8
        as *mut MotherBrainInstrExecState;
    if (*st).ip as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        return 0 as uint16;
    }
    let mut v2: uint16 = (*st).ip;
    let mut v3: *const uint16 = RomPtr_A9(v2 as uint16_t, game) as *mut uint16;
    if *v3.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        current_block = 2809726207655838168;
    } else {
        if (*st).timer as ::core::ffi::c_int
            != *v3.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        {
            (*st).timer = (*st).timer.wrapping_add(1);
            return *v3.offset(1 as ::core::ffi::c_int as isize);
        }
        v2 = (v2 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
        current_block = 6937071982253665452;
    }
    loop {
        match current_block {
            2809726207655838168 => {
                v2 = CallMotherBrainInstr(
                    (*v3.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        | 0xa90000 as ::core::ffi::c_int) as uint32,
                    (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
                    game,
                );
                current_block = 6937071982253665452;
            }
            _ => {
                v3 = RomPtr_A9(v2 as uint16_t, game) as *mut uint16;
                if *v3.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    break;
                } else {
                    current_block = 2809726207655838168;
                }
            }
        }
    }
    if *v3.offset(0 as ::core::ffi::c_int as isize) == 0 {
        (*st).ip = 0 as uint16;
        (*st).timer = 0 as uint16;
        return 0 as uint16;
    } else {
        (*st).timer = 1 as uint16;
        (*st).ip = v2;
        return *v3.offset(1 as ::core::ffi::c_int as isize);
    };
}
pub unsafe fn MotherBrain_HandlePalette(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if (*E).mbn_var_10 != 0 {
        MotherBrain_HandleBrainPal(game);
    }
    if (*E).mbn_var_11 != 0 {
        MotherBrain_HealthBasedPaletteHandling(game);
    }
}
pub unsafe fn MotherBrain_SetupBrainNormalPal(game: &mut Game) {
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_E = 10 as uint16;
}
pub unsafe fn MotherBrain_SetupBrainPalForLaser(game: &mut Game) {
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_E = 514 as uint16;
}
pub unsafe fn MotherBrain_HandleBrainPal(game: &mut Game) {
    let mut mbn_var_D_high: int8 = 0;
    let mut mbn_var_D: int8 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if (*E).mbn_var_00 as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
        mbn_var_D_high = *(&raw mut (*E).mbn_var_D as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as int8;
        if mbn_var_D_high != 0 {
            *(&raw mut (*E).mbn_var_D as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) = (mbn_var_D_high as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint8;
        } else {
            let mut r18: uint16 = *(game.g_rom.offset(
                (((0xa9d260 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa9d260 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    (*(&raw mut (*E).mbn_var_E as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ) as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                );
            *(&raw mut (*E).mbn_var_D as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) = (*E).mbn_var_E as uint8;
            mbn_var_D = (*E).mbn_var_D as int8;
            if mbn_var_D as ::core::ffi::c_int != 0
                || (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_A
                    as ::core::ffi::c_int
                    == 0xa987a2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            {
                let mut v4: uint8 = (mbn_var_D as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                    & 7 as ::core::ffi::c_int) as uint8;
                *(&raw mut (*E).mbn_var_D as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
                    v4;
                let mut v5: uint16 = 290 as uint16;
                if (*E).mbn_var_0D as ::core::ffi::c_int != 512 as ::core::ffi::c_int {
                    v5 = 482 as uint16;
                }
                WriteColorsToPalette(
                    v5,
                    0xa9 as uint8,
                    (r18 as ::core::ffi::c_int + 8 as ::core::ffi::c_int * v4 as ::core::ffi::c_int)
                        as uint16,
                    3 as uint16,
                    game,
                );
            }
        }
    }
}
pub unsafe fn WriteColorsToPalette(
    mut k: uint16,
    mut db: uint8,
    mut j: uint16,
    mut a: uint16,
    game: &mut Game,
) {
    let mut n: ::core::ffi::c_int = a as ::core::ffi::c_int;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *(RomPtrWithBank(db, j as uint16_t, game) as *mut uint16);
        k = (k as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        j = (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
}
pub unsafe fn WriteColorsToTargetPalette(
    mut db: uint8,
    mut k: uint16,
    mut j: uint16,
    mut a: uint16,
    game: &mut Game,
) {
    let mut n: ::core::ffi::c_int = a as ::core::ffi::c_int;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *(RomPtrWithBank(db, j as uint16_t, game) as *mut uint16);
        k = (k as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        j = (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
}
pub unsafe fn DeadTorizo_Init(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 4094 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let mut E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0 as ::core::ffi::c_int, game);
    (*E).dto_var_A = (0xa9d3ad as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.properties =
        ((*E).base.properties as ::core::ffi::c_int | 0xa000 as ::core::ffi::c_int) as uint16;
    (*E).base.current_instruction = addr_kDeadTorizo_Ilist_D6DC as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.palette_index = 512 as uint16;
    (*E).dto_var_B = 0 as uint16;
    (*E).dto_var_C = 8 as uint16;
    (*E).dto_var_04 = 0 as uint16;
    (*E).dto_var_02 = 15 as uint16;
    (*E).dto_var_03 = 0 as uint16;
    InitializeEnemyCorpseRotting(
        0 as uint16,
        addr_stru_A9DD58 as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn CallDeadTorizoFuncA(mut ea: uint32, game: &mut Game) {
    match ea {
        11129773 => {
            DeadTorizo_WaitForSamusColl(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                game,
            );
            return;
        }
        11129830 => {
            DeadTorizo_Rotting(game);
            return;
        }
        11129800 => {
            DeadTorizo_PreRotDelay(game);
            return;
        }
        11129799 => return,
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn DeadTorizo_Main(game: &mut Game) {
    let mut E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0 as ::core::ffi::c_int, game);
    if (*E).base.properties as ::core::ffi::c_int & kEnemyProps_Tangible as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && DeadTorizo_Func_0(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
        (*E).dto_var_A = (0xa9d3e6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0x178c as ::core::ffi::c_int as isize)
            as *mut LongPtr,
        0xa9d39a as uint32,
    );
    CallDeadTorizoFuncA(
        ((*E).dto_var_A as ::core::ffi::c_int | 0xa90000 as ::core::ffi::c_int) as uint32,
        game,
    );
    DeadTorizo_Func_1(game);
}
pub unsafe fn DeadTorizo_MainGfxHook(game: &mut Game) {
    MotherBrain_AddSpritemapToOam(
        addr_kDeadTorizo_Sprmap_D761 as ::core::ffi::c_int as uint16,
        296 as uint16,
        187 as uint16,
        0 as uint16,
        game,
    );
}
pub unsafe fn DeadTorizo_WaitForSamusColl(mut k: uint16, game: &mut Game) {
    if k as ::core::ffi::c_int
        == *((&raw mut game.g_ram as *mut uint8).offset(0x182c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        || k as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0x182c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        || k as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0x182c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        || k as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0x182c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    {
        (*Get_DeadTorizo(0 as ::core::ffi::c_int, game)).dto_var_A =
            (0xa9d3c8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn DeadTorizo_PreRotDelay(game: &mut Game) {
    let mut E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 =
        ((*E).dto_var_04 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    (*E).dto_var_04 = v1;
    if v1 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
        (*E).dto_var_A = (0xa9d3e6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        DeadTorizo_Rotting(game);
    }
}
pub unsafe fn DeadTorizo_Rotting(game: &mut Game) {
    let mut E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 =
        ((*E).dto_var_03 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    (*E).dto_var_03 = v1;
    if v1 as ::core::ffi::c_int >= 0xf as ::core::ffi::c_int {
        (*E).dto_var_03 = 0 as uint16;
        let mut dto_var_02: uint16 = (*E).dto_var_02;
        if dto_var_02 != 0 {
            DeadTorizo_CopyLineOfSandHeapTileData(dto_var_02, game);
            (*E).dto_var_02 = (*E).dto_var_02.wrapping_sub(1);
        }
    }
    (*E).dto_var_C = (*E).dto_var_C.wrapping_add(1);
    MoveEnemyWithVelocity(game);
    if ProcessCorpseRotting(0 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0
    {
        (*E).dto_var_A = (0xa9d3c7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn DeadTorizo_Powerbomb(game: &mut Game) {
    if (*Get_DeadTorizo(0 as ::core::ffi::c_int, game))
        .base
        .properties as ::core::ffi::c_int
        & kEnemyProps_Tangible as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        DeadTorizo_Shot(game);
    }
}
pub unsafe fn DeadTorizo_Shot(game: &mut Game) {
    let mut E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0 as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
    (*E).dto_var_A = (0xa9d3e6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn DeadTorizo_Func_0(game: &mut Game) -> uint8 {
    let mut v5: int16 = 0;
    let mut v3: uint16 = 0;
    let mut v4: uint16 = 0;
    let mut E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0 as ::core::ffi::c_int, game);
    let mut r18: uint16 = (*E).base.x_pos;
    let mut r20: uint16 = (*E).base.y_pos;
    let mut p: *const uint8 = RomPtr_A9(
        addr_kDeadTorizo_Hitbox_D77C as ::core::ffi::c_int as uint16_t,
        game,
    );
    let mut R22: ::core::ffi::c_int = *(p as *mut uint16) as ::core::ffi::c_int;
    let mut R24: uint16 = 0;
    p = p.offset(2 as ::core::ffi::c_int as isize);
    while R22 != 0 {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - r20 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            R24 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16)
                as ::core::ffi::c_int
                - r20 as ::core::ffi::c_int) as uint16;
            v3 = *(p.offset(6 as ::core::ffi::c_int as isize) as *mut uint16);
        } else {
            R24 = (r20 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
            v3 = *(p.offset(2 as ::core::ffi::c_int as isize) as *mut uint16);
        }
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + abs16(v3) as ::core::ffi::c_int
            - R24 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            if (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - r18 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                R24 = (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - r18 as ::core::ffi::c_int) as uint16;
                v4 = *(p.offset(4 as ::core::ffi::c_int as isize) as *mut uint16);
            } else {
                R24 = (r18 as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xaf6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16;
                v4 = *(p as *mut uint16);
            }
            v5 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafe as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + abs16(v4) as ::core::ffi::c_int
                - R24 as ::core::ffi::c_int) as int16;
            if v5 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                if v5 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    v5 = 4 as int16;
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0xb58 as ::core::ffi::c_int as isize)
                    as *mut uint16) = v5 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5c as ::core::ffi::c_int as isize)
                    as *mut uint16) = 4 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb56 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5a as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                return 1 as uint8;
            }
        }
        p = p.offset(8 as ::core::ffi::c_int as isize);
        R22 -= 1;
    }
    return 0 as uint8;
}
pub unsafe fn DeadTorizo_Func_1(game: &mut Game) {
    let mut v5: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v10: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 =
        ((*E).dto_var_00 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    (*E).dto_var_00 = v1;
    let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    if v1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut v8: uint16 = 0 as uint16;
        let mut v9: uint16 = *(game.g_rom.offset(
            (((0xa9d583 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa9d583 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize);
        loop {
            v10 = gVramWriteEntry(v3 as ::core::ffi::c_int, game);
            (*v10).size = v9;
            let mut v11: ::core::ffi::c_int = v8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut (*v10).src.addr as *mut uint8).offset(1 as ::core::ffi::c_int as isize)
                as *mut VoidP) = *(game.g_rom.offset(
                (((0xa9d583 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa9d583 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v11 + 1 as ::core::ffi::c_int) as isize)
                as VoidP;
            (*v10).src.addr = *(game.g_rom.offset(
                (((0xa9d583 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa9d583 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v11 + 2 as ::core::ffi::c_int) as isize)
                as VoidP;
            (*v10).vram_dst = *(game.g_rom.offset(
                (((0xa9d583 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa9d583 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v11 + 3 as ::core::ffi::c_int) as isize);
            v3 = (v3 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
            v8 = (v8 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
            v9 = *(game.g_rom.offset(
                (((0xa9d583 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa9d583 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            if !(v9 != 0) {
                break;
            }
        }
    } else {
        let mut v2: uint16 = 0 as uint16;
        let mut v4: uint16 = *(game.g_rom.offset(
            (((0xa9d549 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa9d549 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize);
        loop {
            v5 = gVramWriteEntry(v3 as ::core::ffi::c_int, game);
            (*v5).size = v4;
            let mut v6: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut (*v5).src.addr as *mut uint8).offset(1 as ::core::ffi::c_int as isize)
                as *mut VoidP) = *(game.g_rom.offset(
                (((0xa9d549 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa9d549 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v6 + 1 as ::core::ffi::c_int) as isize)
                as VoidP;
            (*v5).src.addr = *(game.g_rom.offset(
                (((0xa9d549 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa9d549 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v6 + 2 as ::core::ffi::c_int) as isize)
                as VoidP;
            (*v5).vram_dst = *(game.g_rom.offset(
                (((0xa9d549 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa9d549 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v6 + 3 as ::core::ffi::c_int) as isize);
            v3 = (v3 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
            v2 = (v2 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
            v4 = *(game.g_rom.offset(
                (((0xa9d549 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa9d549 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            if !(v4 != 0) {
                break;
            }
        }
    }
    (*Get_DeadEnemy(0 as ::core::ffi::c_int, game)).dey_var_22 = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
        as *mut uint16) = v3;
}
pub unsafe fn DeadTorizo_CorpseRottingFinished(game: &mut Game) {
    game.eproj_spawn_pt = Point16U {
        x: ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
            + 272 as ::core::ffi::c_int) as uint16,
        y: 188 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        0xa as uint16,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe44 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        QueueSfx2_Max6(0x10 as uint16, game);
    }
}
pub unsafe fn DeadTorizo_CopyLineOfSandHeapTileData(mut a: uint16, game: &mut Game) {
    let mut v1: uint16 = *(game.g_rom.offset(
        (((0xa9d67c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa9d67c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(a as isize);
    let mut v2: ::core::ffi::c_int = *(game.g_rom.offset(
        (((0xa9d69c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa9d69c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(a as isize) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9500 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset(v2 as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9510 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 8 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9520 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 16 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9530 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 24 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9540 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 32 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9550 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 40 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9560 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 48 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9570 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 56 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9580 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 64 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9590 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 72 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x95a0 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 80 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x95b0 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 88 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x95c0 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 96 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x95d0 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 104 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x95e0 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 112 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x95f0 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 120 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9600 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 128 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9610 as ::core::ffi::c_int as isize)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 136 as ::core::ffi::c_int) as isize);
}
pub unsafe fn DeadSidehopper_Init(game: &mut Game) {
    let mut dsr_parameter_1: uint16 = (*Get_DeadSidehopper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .dsr_parameter_1;
    if dsr_parameter_1 != 0 {
        if dsr_parameter_1 as ::core::ffi::c_int != 2 as ::core::ffi::c_int {
            Unreachable(game);
        }
        DeadSidehopper_Init_1(game);
    } else {
        DeadSidehopper_Init_0(game);
    };
}
pub unsafe fn DeadSidehopper_Init_0(game: &mut Game) {
    let mut E: *mut Enemy_DeadSidehopper = Get_DeadSidehopper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties =
        ((*E).base.properties as ::core::ffi::c_int & 0x77ff as ::core::ffi::c_int
            | kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int) as uint16;
    if (*Get_DeadSidehopper(0 as ::core::ffi::c_int, game))
        .base
        .properties as ::core::ffi::c_int
        & kEnemyProps_Invisible as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
    }
    (*E).dsr_var_08 = 0 as uint16;
    (*E).dsr_var_0A = 96 as uint16;
    (*E).dsr_var_0B = 256 as uint16;
    (*E).base.x_pos = 488 as uint16;
    (*E).base.y_pos = 184 as uint16;
    (*E).dsr_var_A = (0xa9d8e2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.palette_index = 512 as uint16;
    (*E).base.y_height = 21 as uint16;
    Enemy_SetInstrList(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        addr_kDeadMonsters_Ilist_ECE3 as ::core::ffi::c_int as uint16,
        game,
    );
    InitializeEnemyCorpseRotting(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        addr_stru_A9DD68 as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn DeadSidehopper_Init_1(game: &mut Game) {
    let mut E: *mut Enemy_DeadSidehopper = Get_DeadSidehopper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).dsr_var_08 = -(1 as ::core::ffi::c_int) as uint16;
    (*E).dsr_var_A = (0xa9da64 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.palette_index = 3584 as uint16;
    Enemy_SetInstrList(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        addr_kDeadMonsters_Ilist_ECEF as ::core::ffi::c_int as uint16,
        game,
    );
    InitializeEnemyCorpseRotting(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        addr_stru_A9DD78 as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn DeadZoomer_Init(game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).palette_index = 3584 as uint16;
    (*v1).ai_var_A = (0xa9da69 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut v2: ::core::ffi::c_int =
        (*v1).parameter_1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    Enemy_SetInstrList(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        *(game.g_rom.offset(
            (((0xa9d86a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa9d86a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize),
        game,
    );
    InitializeEnemyCorpseRotting(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        *(game.g_rom.offset(
            (((0xa9d870 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa9d870 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize),
        game,
    );
}
pub unsafe fn DeadRipper_Init(game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).palette_index = 3584 as uint16;
    (*v1).ai_var_A = (0xa9da73 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut v2: ::core::ffi::c_int =
        (*v1).parameter_1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    Enemy_SetInstrList(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        *(game.g_rom.offset(
            (((0xa9d897 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa9d897 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize),
        game,
    );
    InitializeEnemyCorpseRotting(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        *(game.g_rom.offset(
            (((0xa9d89b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa9d89b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize),
        game,
    );
}
pub unsafe fn DeadSkree_Init(game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).palette_index = 3584 as uint16;
    (*v1).ai_var_A = (0xa9da6e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut v2: ::core::ffi::c_int =
        (*v1).parameter_1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    Enemy_SetInstrList(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        *(game.g_rom.offset(
            (((0xa9d8c0 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa9d8c0 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize),
        game,
    );
    InitializeEnemyCorpseRotting(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        *(game.g_rom.offset(
            (((0xa9d8c6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa9d8c6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize),
        game,
    );
}
pub unsafe fn DeadSidehopper_Powerbomb(game: &mut Game) {
    if (*Get_DeadSidehopper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .dsr_var_08 as ::core::ffi::c_int
        >= 8 as ::core::ffi::c_int
    {
        DeadSidehopper_Shot(game);
    } else {
        DeadSidehopper_Main(game);
    };
}
pub unsafe fn CallDeadSidehopperFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        11131121 => {
            DeadSidehopper_Activated(k, game);
            return;
        }
        11131106 => {
            DeadSidehopper_Alive_WaitForActivate(k, game);
            return;
        }
        11131492 => {
            DeadSidehopper_WaitForSamusColl(k, game);
            return;
        }
        11131497 => {
            DeadZoomer_WaitForSamusColl(k, game);
            return;
        }
        11131502 => {
            DeadSkree_WaitForSamusColl(k, game);
            return;
        }
        11131507 => {
            DeadRipper_WaitForSamusColl(k, game);
            return;
        }
        11131535 => {
            DeadSidehopper_PreRotDelay(k, game);
            return;
        }
        11131540 => {
            DeadZoomer_PreRotDelay(k, game);
            return;
        }
        11131550 => {
            DeadSkree_PreRotDelay(k, game);
            return;
        }
        11131545 => {
            DeadRipper_PreRotDelay(k, game);
            return;
        }
        11131578 => {
            DeadSidehopper_Rotting(k, game);
            return;
        }
        11131600 => {
            DeadZoomer_Rotting(k, game);
            return;
        }
        11131644 => {
            DeadSkree_Rotting(k, game);
            return;
        }
        11131622 => {
            DeadRipper_Rotting(k, game);
            return;
        }
        11131152 => {
            DeadMonsters_Func_1(k, game);
            return;
        }
        11131165 => {
            DeadMonsters_Func_2(k, game);
            return;
        }
        11131400 => {
            DeadMonsters_Func_5(k, game);
            return;
        }
        11131151 => return,
        11131491 => return,
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn DeadSidehopper_Main(game: &mut Game) {
    let mut E: *mut Enemy_DeadSidehopper = Get_DeadSidehopper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallDeadSidehopperFunc(
        ((*E).dsr_var_A as ::core::ffi::c_int | 0xa90000 as ::core::ffi::c_int) as uint32,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
}
pub unsafe fn DeadSidehopper_Alive_WaitForActivate(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 513 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*Get_DeadSidehopper(k as ::core::ffi::c_int, game)).dsr_var_A =
            (0xa9d8f1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        DeadSidehopper_Activated(k, game);
    }
}
pub unsafe fn DeadSidehopper_Activated(mut k: uint16, game: &mut Game) {
    if DeadMonsters_Func_3(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut E: *mut Enemy_DeadSidehopper = Get_DeadSidehopper(k as ::core::ffi::c_int, game);
        (*E).dsr_var_06 = ((*E).dsr_var_06 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int) as uint16;
        Enemy_SetInstrList(
            k,
            addr_kDeadMonsters_Ilist_ECAC as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).dsr_var_A = (0xa9d90f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn DeadMonsters_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as ::core::ffi::c_int, game);
    (*E).dms_var_A = (0xa9d91d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).dms_var_F = 64 as uint16;
}
pub unsafe fn DeadMonsters_Func_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as ::core::ffi::c_int, game);
    (*E).dms_var_F = (*E).dms_var_F.wrapping_sub(1);
    if (*E).dms_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if (*E).dms_var_08 != 0 {
            (*E).dms_var_A =
                (0xa9da08 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            (*E).dms_var_A =
                (0xa9d8f1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            Enemy_SetInstrList(
                k,
                addr_kDeadMonsters_Ilist_ECE3 as ::core::ffi::c_int as uint16,
                game,
            );
            let mut v3: ::core::ffi::c_int = (*E).dms_var_06 as ::core::ffi::c_int;
            (*E).dms_var_0B = *(game.g_rom.offset(
                (((0xa9d951 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa9d951 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v3 as isize);
            (*E).dms_var_0A = *(game.g_rom.offset(
                (((0xa9d959 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa9d959 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v3 as isize);
        }
    }
}
pub unsafe fn DeadMonsters_Func_3(mut k: uint16, game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as ::core::ffi::c_int, game);
    DeadMonsters_Func_4(k, (*E).dms_var_0A, game);
    let mut v2: uint16 = 32 as uint16;
    if (*E).dms_var_0B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v2 = 128 as uint16;
    }
    let mut v3: uint16 =
        ((*E).dms_var_0B as ::core::ffi::c_int + v2 as ::core::ffi::c_int) as uint16;
    (*E).dms_var_0B = v3;
    if (*E).base.x_pos as ::core::ffi::c_int - 544 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut full: ::core::ffi::c_int = *(&raw mut (*E).base.y_subpos as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            + *(&raw mut v3 as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int;
        *(&raw mut (*E).base.y_subpos as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) = full as uint8;
        let mut v12: uint16 = ((*E).base.y_pos as ::core::ffi::c_int
            + (v3 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
            + (full >> 8 as ::core::ffi::c_int)) as uint16;
        (*E).base.y_pos = v12;
        return (v12 as ::core::ffi::c_int - 184 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
    } else {
        return Enemy_MoveDown(k, (v3 as int16 as int32) << 8 as ::core::ffi::c_int, game);
    };
}
pub unsafe fn DeadMonsters_Func_4(mut k: uint16, mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as ::core::ffi::c_int, game);
    if (*E).base.x_pos as ::core::ffi::c_int - 544 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut full: ::core::ffi::c_int = *(&raw mut (*E).base.x_subpos as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            + *(&raw mut a as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int;
        *(&raw mut (*E).base.x_subpos as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) = full as uint8;
        (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
            + ((a as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
                + (full >> 8 as ::core::ffi::c_int))) as uint16;
    } else {
        Enemy_MoveRight_IgnoreSlopes(k, (a as int16 as int32) << 8 as ::core::ffi::c_int, game);
    };
}
pub unsafe fn DeadMonsters_Func_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as ::core::ffi::c_int, game);
    let mut v2: uint16 =
        ((*E).dms_var_07 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    (*E).dms_var_07 = v2;
    if v2 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
        (*E).dms_var_07 = 0 as uint16;
        WriteColorsToPalette(
            0x122 as uint16,
            0xa9 as uint8,
            (32 as ::core::ffi::c_int
                * ((*E).dms_var_08 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                - 0x1434 as ::core::ffi::c_int) as uint16,
            0xf as uint16,
            game,
        );
        let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut v5: uint16 =
            ((*E).dms_var_08 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E).dms_var_08 = v5;
        if v5 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
            Enemy_SetInstrList(
                v3,
                addr_kDeadMonsters_Ilist_ECE9 as ::core::ffi::c_int as uint16,
                game,
            );
            (*E).dms_var_A =
                (0xa9da64 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                | 0x8000 as ::core::ffi::c_int) as uint16;
            (*E).base.y_height = 12 as uint16;
        }
    }
}
pub unsafe fn DeadSidehopper_WaitForSamusColl(mut k: uint16, game: &mut Game) {
    DeadMonsters_WaitForSamusColl(
        k,
        (0xa9da8f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn DeadZoomer_WaitForSamusColl(mut k: uint16, game: &mut Game) {
    DeadMonsters_WaitForSamusColl(
        k,
        (0xa9da94 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn DeadSkree_WaitForSamusColl(mut k: uint16, game: &mut Game) {
    DeadMonsters_WaitForSamusColl(
        k,
        (0xa9da9e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn DeadRipper_WaitForSamusColl(mut k: uint16, game: &mut Game) {
    DeadMonsters_WaitForSamusColl(
        k,
        (0xa9da99 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn DeadMonsters_WaitForSamusColl(mut k: uint16, mut j: uint16, game: &mut Game) {
    if k as ::core::ffi::c_int
        == *((&raw mut game.g_ram as *mut uint8).offset(0x182c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        || k as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0x182c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        || k as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0x182c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        || k as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0x182c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    {
        (*Get_DeadMonsters(k as ::core::ffi::c_int, game)).dms_var_A = j;
    }
}
pub unsafe fn DeadSidehopper_PreRotDelay(mut k: uint16, game: &mut Game) {
    DeadMonsters_PreRotDelay_Common(
        k,
        (0xa9daba as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn DeadZoomer_PreRotDelay(mut k: uint16, game: &mut Game) {
    DeadMonsters_PreRotDelay_Common(
        k,
        (0xa9dad0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn DeadRipper_PreRotDelay(mut k: uint16, game: &mut Game) {
    DeadMonsters_PreRotDelay_Common(
        k,
        (0xa9dae6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn DeadSkree_PreRotDelay(mut k: uint16, game: &mut Game) {
    DeadMonsters_PreRotDelay_Common(
        k,
        (0xa9dafc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn DeadMonsters_PreRotDelay_Common(mut k: uint16, mut j: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as ::core::ffi::c_int, game);
    (*E).dms_var_B = (*E).dms_var_B.wrapping_add(1);
    if (*E).dms_var_B as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
        (*E).dms_var_A = j;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn DeadSidehopper_Rotting(mut k: uint16, game: &mut Game) {
    let mut v1: uint8 = ProcessCorpseRotting(k, game);
    if v1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        (*Get_DeadMonsters(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
            game,
        ))
        .dms_var_A = (0xa9da64 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    let mut dms_var_53: uint16 = (*Get_DeadMonsters(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .dms_var_53;
    ProcessCorpseRottingVramTransfers(dms_var_53, game);
}
pub unsafe fn DeadZoomer_Rotting(mut k: uint16, game: &mut Game) {
    let mut v1: uint8 = ProcessCorpseRotting(k, game);
    if v1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        (*Get_DeadMonsters(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
            game,
        ))
        .dms_var_A = (0xa9da63 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    let mut dms_var_53: uint16 = (*Get_DeadMonsters(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .dms_var_53;
    ProcessCorpseRottingVramTransfers(dms_var_53, game);
}
pub unsafe fn DeadRipper_Rotting(mut k: uint16, game: &mut Game) {
    let mut v1: uint8 = ProcessCorpseRotting(k, game);
    if v1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        (*Get_DeadMonsters(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
            game,
        ))
        .dms_var_A = (0xa9da63 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    let mut dms_var_53: uint16 = (*Get_DeadMonsters(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .dms_var_53;
    ProcessCorpseRottingVramTransfers(dms_var_53, game);
}
pub unsafe fn DeadSkree_Rotting(mut k: uint16, game: &mut Game) {
    let mut v1: uint8 = ProcessCorpseRotting(k, game);
    if v1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        (*Get_DeadMonsters(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
            game,
        ))
        .dms_var_A = (0xa9da63 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    let mut dms_var_53: uint16 = (*Get_DeadMonsters(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .dms_var_53;
    ProcessCorpseRottingVramTransfers(dms_var_53, game);
}
pub unsafe fn ProcessCorpseRotting(mut k: uint16, game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_DeadMonsters = ::core::ptr::null_mut::<Enemy_DeadMonsters>();
    let mut r18_0: uint16 = 0;
    let mut v11: *mut uint16 = ::core::ptr::null_mut::<uint16>();
    let mut v12: uint16 = 0;
    let mut v5: *mut int16 = ::core::ptr::null_mut::<int16>();
    let mut v7: int16 = 0;
    let mut v8: int16 = 0;
    let mut EK: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as ::core::ffi::c_int, game);
    let mut E0: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    (*E0).dms_var_45 = (*EK).dms_var_57;
    (*E0).dms_var_46 = (*EK).dms_var_58;
    (*E0).dms_var_47 = (*EK).dms_var_59;
    (*E0).dms_var_48 = (*EK).dms_var_5A;
    (*E0).dms_var_49 = (*EK).dms_var_5B;
    (*E0).dms_var_44 = (*EK).dms_var_56;
    (*E0).dms_var_42 = (*EK).dms_var_54;
    (*E0).dms_var_43 = (*EK).dms_var_55;
    let mut dms_var_52: uint16 = (*EK).dms_var_52;
    let mut v4: uint16 = 0 as uint16;
    let mut v15: uint16 = 0;
    let mut v6: *mut uint16 = ::core::ptr::null_mut::<uint16>();
    loop {
        v15 = v4;
        v5 = (&raw mut game.g_ram as *mut uint8).offset(dms_var_52 as ::core::ffi::c_int as isize)
            as *mut int16;
        v6 = v5 as *mut uint16;
        if *v5 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            v7 = *v5.offset(1 as ::core::ffi::c_int as isize);
            if v7 != 0 {
                v8 = (v7 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
                *v5.offset(1 as ::core::ffi::c_int as isize) = v8;
                if (v8 as uint16 as ::core::ffi::c_int) < 4 as ::core::ffi::c_int {
                    E0 = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
                    let mut r18: uint16 = (*E0).dms_var_42;
                    if v15 as ::core::ffi::c_int >= (*E0).dms_var_47 as ::core::ffi::c_int {
                        r18 = (*E0).dms_var_43;
                    }
                    CopyMoveCorpseRottingRotEntry(
                        RomPtr_A9((*E0).dms_var_44 as uint16_t, game) as *const uint16,
                        *v6,
                        r18,
                        game,
                    );
                }
            } else {
                E = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
                r18_0 = (*E).dms_var_43;
                v11 = (&raw mut game.g_ram as *mut uint8)
                    .offset(dms_var_52 as ::core::ffi::c_int as isize)
                    as *mut uint16;
                CopyMoveCorpseRottingRotEntry(
                    RomPtr_A9((*E).dms_var_44 as uint16_t, game) as *const uint16,
                    *v11,
                    r18_0,
                    game,
                );
                v12 = (*v11 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                if (v12 as ::core::ffi::c_int) < (*E).dms_var_46 as ::core::ffi::c_int {
                    *v11 = v12;
                } else {
                    CallCorpseRottingFinish(
                        ((*E).dms_var_49 as ::core::ffi::c_int | 0xa90000 as ::core::ffi::c_int)
                            as uint32,
                        game,
                    );
                    if !((v15 as ::core::ffi::c_int)
                        < (*Get_DeadMonsters(0 as ::core::ffi::c_int, game)).dms_var_46
                            as ::core::ffi::c_int)
                    {
                        break;
                    }
                    *v11 = -(1 as ::core::ffi::c_int) as uint16;
                }
            }
        }
        dms_var_52 = (dms_var_52 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
        v4 = (v15 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        E0 = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
        if (v15 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            - (*E0).dms_var_45 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            return ((v15 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16
                as ::core::ffi::c_int
                >= (*E0).dms_var_45 as ::core::ffi::c_int) as ::core::ffi::c_int
                as uint8;
        }
    }
    return 0 as uint8;
}
pub unsafe fn CopyMoveCorpseRottingRotEntry(
    mut r20: *const uint16,
    mut a: uint16,
    mut r18: uint16,
    game: &mut Game,
) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    (*E).dms_var_41 = a;
    let mut v1: uint16 = ((a as ::core::ffi::c_int & 0xfff8 as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 2 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = (a as ::core::ffi::c_int & 7 as ::core::ffi::c_int) as uint16;
    let mut v6: uint16 = (*r20
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int * v2 as ::core::ffi::c_int) as uint16;
    if v2 as ::core::ffi::c_int >= 6 as ::core::ffi::c_int {
        CallCorpseRottingMove(
            (r18 as ::core::ffi::c_int | 0xa90000 as ::core::ffi::c_int) as uint32,
            ((*E).dms_var_48 as ::core::ffi::c_int + v6 as ::core::ffi::c_int) as uint16,
            v6,
            game,
        );
    } else {
        CallCorpseRottingMove(
            (r18 as ::core::ffi::c_int | 0xa90000 as ::core::ffi::c_int) as uint32,
            v6,
            v6,
            game,
        );
    };
}
pub unsafe fn CorpseRottingRotEntryFinishedHook(game: &mut Game) {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    game.eproj_spawn_pt = Point16U {
        x: ((*v0).x_pos as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x1a as ::core::ffi::c_int)
            - 14 as ::core::ffi::c_int) as uint16,
        y: ((*v0).y_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        0xa as uint16,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe44 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        QueueSfx2_Max3(0x10 as uint16, game);
    }
}
pub unsafe fn InitializeCorpseRottingDataTable(mut table: *mut uint16, mut a: uint16) {
    let mut v2: int16 = 0;
    v2 = (a as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
    let mut r18: uint16 = 0 as uint16;
    loop {
        *table.offset(0 as ::core::ffi::c_int as isize) = v2 as uint16;
        *table.offset(1 as ::core::ffi::c_int as isize) = r18;
        table = table.offset(2 as ::core::ffi::c_int as isize);
        r18 = (r18 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v2 -= 1;
        if !(v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
pub unsafe fn InitializeEnemyCorpseRotting(mut k: uint16, mut j: uint16, game: &mut Game) {
    let mut v2: *const uint8 = RomPtr_A9(j as uint16_t, game);
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as ::core::ffi::c_int, game);
    (*E).dms_var_52 = *(v2 as *mut uint16);
    (*E).dms_var_53 = *(v2.offset(2 as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).dms_var_54 = *(v2.offset(4 as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).dms_var_55 = *(v2.offset(6 as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).dms_var_5B = *(v2.offset(14 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v4: uint16 = *(v2.offset(12 as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).dms_var_56 = v4;
    (*E).dms_var_5A = (*(RomPtr_A9(v4 as uint16_t, game) as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        - 12 as ::core::ffi::c_int) as uint16;
    let mut v5: *mut uint16 = v2 as *mut uint16;
    let mut v6: uint16 = *v5.offset(4 as ::core::ffi::c_int as isize);
    let fresh5 = v6;
    v6 = v6.wrapping_sub(1);
    (*E).dms_var_57 = fresh5;
    (*E).dms_var_58 = v6;
    (*E).dms_var_59 = (v6 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    InitializeCorpseRottingDataTable(
        (&raw mut game.g_ram as *mut uint8).offset(*v5 as ::core::ffi::c_int as isize)
            as *mut uint16,
        *v5.offset(4 as ::core::ffi::c_int as isize),
    );
    CallCorpseRottingInit(
        (*v5.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            | 0xa90000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn ProcessCorpseRottingVramTransfers(mut k: uint16, game: &mut Game) {
    let mut p: *const uint8 = RomPtr_A9(k as uint16_t, game);
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: uint16 = *(p as *mut uint16);
    loop {
        let mut v3: *mut VramWriteEntry = gVramWriteEntry(v1 as ::core::ffi::c_int, game);
        (*v3).size = v2;
        *((&raw mut (*v3).src.addr as *mut uint8).offset(1 as ::core::ffi::c_int as isize)
            as *mut VoidP) = *(p as *mut uint16).offset(1 as ::core::ffi::c_int as isize) as VoidP;
        (*v3).src.addr = *(p as *mut uint16).offset(2 as ::core::ffi::c_int as isize) as VoidP;
        (*v3).vram_dst = *(p as *mut uint16).offset(3 as ::core::ffi::c_int as isize);
        v1 = (v1 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
        p = p.offset(8 as ::core::ffi::c_int as isize);
        v2 = *(p as *mut uint16);
        if !(v2 != 0) {
            break;
        }
    }
    (*Get_DeadEnemy(0 as ::core::ffi::c_int, game)).dey_var_22 = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
        as *mut uint16) = v1;
}
pub unsafe fn DeadZoomer_Powerbomb(game: &mut Game) {
    if (*gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .properties as ::core::ffi::c_int
        & kEnemyProps_Tangible as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        DeadZoomer_Shot(game);
    }
}
pub unsafe fn DeadZoomer_Shot(game: &mut Game) {
    DeadSidehopper_DD34(
        (0xa9dad0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn DeadRipper_Powerbomb(game: &mut Game) {
    if (*gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .properties as ::core::ffi::c_int
        & kEnemyProps_Tangible as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        DeadRipper_Shot(game);
    }
}
pub unsafe fn DeadRipper_Shot(game: &mut Game) {
    DeadSidehopper_DD34(
        (0xa9dae6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn DeadSkree_Powerbomb(game: &mut Game) {
    if (*gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .properties as ::core::ffi::c_int
        & kEnemyProps_Tangible as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        DeadSkree_Shot(game);
    }
}
pub unsafe fn DeadSkree_Shot(game: &mut Game) {
    DeadSidehopper_DD34(
        (0xa9dafc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn DeadSidehopper_Shot(game: &mut Game) {
    let mut E: *mut Enemy_DeadSidehopper = Get_DeadSidehopper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if !((*E).base.properties as ::core::ffi::c_int & kEnemyProps_Tangible as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || ((*E).dsr_var_08 as ::core::ffi::c_int) < 8 as ::core::ffi::c_int)
    {
        DeadSidehopper_DD31(game);
    }
}
pub unsafe fn DeadSidehopper_DD31(game: &mut Game) {
    DeadSidehopper_DD34(
        (0xa9daba as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn DeadSidehopper_DD34(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadSidehopper = Get_DeadSidehopper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).dsr_var_A = a;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn DeadSidehopper_Touch(game: &mut Game) {
    if ((*Get_DeadSidehopper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .dsr_var_08 as ::core::ffi::c_int)
        < 8 as ::core::ffi::c_int
    {
        NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
    } else {
        DeadSidehopper_DD31(game);
    };
}
pub unsafe fn Torizo_CorpseRottingInitFunc(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(
        addr_kDeadTorizo_TileData as ::core::ffi::c_int as uint16_t,
        game,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x60 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(288 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0xc0 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x1a0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(800 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0xc0 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x2c0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1280 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x100 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x400 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1792 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x100 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x540 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2304 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x100 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x680 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2816 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x100 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x7c0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(3328 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x100 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x900 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(3840 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x100 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xa40 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(4352 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x100 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xb60 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(4832 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x120 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xc80 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(5312 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x140 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xdc0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(5824 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x140 as ::core::ffi::c_int,
    );
}
pub unsafe fn Sidehopper_CorpseRottingInitFunc_0(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(
        addr_kDeadMonsters_TileData as ::core::ffi::c_int as uint16_t,
        game,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x40 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(64 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xa0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(512 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0xa0 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x140 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1024 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0xa0 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x1e0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1536 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0xa0 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x280 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2048 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0xa0 as ::core::ffi::c_int,
    );
}
pub unsafe fn Sidehopper_CorpseRottingInitFunc_2(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(
        addr_kDeadMonsters_TileData as ::core::ffi::c_int as uint16_t,
        game,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x320 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(288 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x3c0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(800 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0xa0 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x460 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1312 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0xa0 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x500 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1824 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0xa0 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x5a0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2336 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0xa0 as ::core::ffi::c_int,
    );
}
pub unsafe fn Zoomer_CorpseRottingInitFunc_0(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(
        addr_kDeadMonsters_TileData as ::core::ffi::c_int as uint16_t,
        game,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x940 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2656 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x9a0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(3168 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
}
pub unsafe fn Zoomer_CorpseRottingInitFunc_2(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(
        addr_kDeadMonsters_TileData as ::core::ffi::c_int as uint16_t,
        game,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xa00 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2752 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xa60 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(3264 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
}
pub unsafe fn Zoomer_CorpseRottingInitFunc_4(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(
        addr_kDeadMonsters_TileData as ::core::ffi::c_int as uint16_t,
        game,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xac0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2848 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xb20 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(3360 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
}
pub unsafe fn Ripper_CorpseRottingInitFunc_0(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(
        addr_kDeadMonsters_TileData as ::core::ffi::c_int as uint16_t,
        game,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xb80 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2560 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xbe0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(3072 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
}
pub unsafe fn Ripper_CorpseRottingInitFunc_2(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(
        addr_kDeadMonsters_TileData as ::core::ffi::c_int as uint16_t,
        game,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xc40 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2944 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xca0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(3456 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
}
pub unsafe fn Skree_CorpseRottingInitFunc_0(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(
        addr_kDeadMonsters_TileData as ::core::ffi::c_int as uint16_t,
        game,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x640 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(672 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x680 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1184 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x6c0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1696 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x700 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2208 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
}
pub unsafe fn Skree_CorpseRottingInitFunc_2(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(
        addr_kDeadMonsters_TileData as ::core::ffi::c_int as uint16_t,
        game,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x740 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(224 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x780 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(736 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x7c0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1248 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x800 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1760 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
}
pub unsafe fn Skree_CorpseRottingInitFunc_4(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(
        addr_kDeadMonsters_TileData as ::core::ffi::c_int as uint16_t,
        game,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x840 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(448 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x880 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(960 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x8c0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1472 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x900 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1984 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
}
pub unsafe fn MotherBrain_CorpseRottingInitFunc(game: &mut Game) {
    let mut base: *mut uint8 = (&raw mut game.g_ram as *mut uint8)
        .offset(0x9000 as ::core::ffi::c_int as isize) as *mut uint8;
    let mut src: *const uint8 = RomPtr_B7(
        addr_kMotherBrain_Misc_TileData as ::core::ffi::c_int as uint16_t,
        game,
    );
    MemCpy(
        base as *mut ::core::ffi::c_void,
        src.offset(192 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0xc0 as ::core::ffi::c_int,
    );
    MemCpy(
        base.offset(0xe0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
        src.offset(704 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0xc0 as ::core::ffi::c_int,
    );
    MemCpy(
        base.offset(0x1c0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
        src.offset(1216 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0xc0 as ::core::ffi::c_int,
    );
    MemCpy(
        base.offset(0x2a0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
        src.offset(1728 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0xc0 as ::core::ffi::c_int,
    );
    MemCpy(
        base.offset(0x380 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
        src.offset(2240 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0xe0 as ::core::ffi::c_int,
    );
    MemCpy(
        base.offset(0x460 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
        src.offset(2752 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        0xe0 as ::core::ffi::c_int,
    );
}
pub unsafe fn Torizo_CorpseRottingMoveFunc(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x50 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v4 + 1 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v4 + 9 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v3 + 8 as ::core::ffi::c_int) as isize);
        }
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v5 + 8 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x48 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v6: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 17 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v6 + 16 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 25 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v6 + 24 as ::core::ffi::c_int) as isize);
        }
        let mut v8: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 16 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 24 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v9: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v10: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v10 + 33 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v9 + 32 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v10 + 41 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v9 + 40 as ::core::ffi::c_int) as isize);
        }
        let mut v11: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v11 + 32 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v11 + 40 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v12: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v13: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v13 + 49 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v12 + 48 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v13 + 57 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v12 + 56 as ::core::ffi::c_int) as isize);
    }
    let mut v14: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v14 + 48 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v14 + 56 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v15: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v15 + 65 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v14 + 64 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v15 + 73 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v14 + 72 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v14 + 64 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v14 + 72 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v16: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v16 + 81 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v14 + 80 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v16 + 89 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v14 + 88 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v14 + 80 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v14 + 88 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v17: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v17 + 97 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v14 + 96 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v17 + 105 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v14 + 104 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v14 + 96 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v14 + 104 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v18: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v18 + 113 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v14 + 112 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v18 + 121 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v14 + 120 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v14 + 112 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v14 + 120 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v19: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v19 + 129 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v14 + 128 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v19 + 137 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v14 + 136 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v14 + 128 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v14 + 136 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v20: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v20 + 145 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v14 + 144 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v20 + 153 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v14 + 152 as ::core::ffi::c_int) as isize);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v14 + 144 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v14 + 152 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn Torizo_CorpseRottingCopyFunc(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x50 as ::core::ffi::c_int
        && (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 9 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 8 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x48 as ::core::ffi::c_int
        && (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 17 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 16 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 25 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 24 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int
        && (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 33 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 32 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 41 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 40 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v9: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v10: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v10 + 49 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v9 + 48 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v10 + 57 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v9 + 56 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v11: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v12: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v12 + 65 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v11 + 64 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v12 + 73 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v11 + 72 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v13: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v14: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v14 + 81 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v13 + 80 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v14 + 89 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v13 + 88 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v15: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v16: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v16 + 97 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v15 + 96 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v16 + 105 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v15 + 104 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v17: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v18: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v18 + 113 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v17 + 112 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v18 + 121 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v17 + 120 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v19: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v20: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v20 + 129 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v19 + 128 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v20 + 137 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v19 + 136 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v21: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v22: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v22 + 145 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v21 + 144 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v22 + 153 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v21 + 152 as ::core::ffi::c_int) as isize);
        }
    }
}
pub unsafe fn Sidehopper_CorpseRottingMoveFunc_0(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v4 + 1 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v4 + 9 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v3 + 8 as ::core::ffi::c_int) as isize);
        }
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v5 + 8 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v6: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 17 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v6 + 16 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 25 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v6 + 24 as ::core::ffi::c_int) as isize);
        }
        let mut v8: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 16 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 24 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v9: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v10: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v10 + 33 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v9 + 32 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v10 + 41 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v9 + 40 as ::core::ffi::c_int) as isize);
    }
    let mut v11: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v11 + 32 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v11 + 40 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v12: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v12 + 49 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v11 + 48 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v12 + 57 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v11 + 56 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v11 + 48 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v11 + 56 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v13: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v13 + 65 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v11 + 64 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v13 + 73 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v11 + 72 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v11 + 64 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v11 + 72 as ::core::ffi::c_int) as isize) = 0 as uint16;
}
pub unsafe fn Sidehopper_CorpseRottingCopyFunc_0(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int
        && (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 9 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 8 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int
        && (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 17 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 16 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 25 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 24 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 33 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 32 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 41 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 40 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v9: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v10: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v10 + 49 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v9 + 48 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v10 + 57 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v9 + 56 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v11: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v12: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v12 + 65 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v11 + 64 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v12 + 73 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v11 + 72 as ::core::ffi::c_int) as isize);
    }
}
pub unsafe fn Sidehopper_CorpseRottingMoveFunc_2(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 401 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 400 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 409 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 408 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 400 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 408 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 417 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 416 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 425 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 424 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 416 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 424 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 433 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v5 + 432 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 441 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v5 + 440 as ::core::ffi::c_int) as isize);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v5 + 432 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v5 + 440 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v8 + 449 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v5 + 448 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v8 + 457 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v5 + 456 as ::core::ffi::c_int) as isize);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v5 + 448 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v5 + 456 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v9: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v9 + 465 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v5 + 464 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v9 + 473 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v5 + 472 as ::core::ffi::c_int) as isize);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v5 + 464 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v5 + 472 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn Sidehopper_CorpseRottingCopyFunc_2(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 401 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 400 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 409 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 408 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 417 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 416 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 425 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 424 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int
        && (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 433 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 432 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 441 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 440 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int
        && (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let mut v9: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v10: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v10 + 449 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v9 + 448 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v10 + 457 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v9 + 456 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v11: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v12: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v12 + 465 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v11 + 464 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v12 + 473 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v11 + 472 as ::core::ffi::c_int) as isize);
        }
    }
}
pub unsafe fn Zoomer_CorpseRottingMoveFunc_0(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1185 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1184 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1193 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1192 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1184 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1192 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1201 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1200 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1209 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1208 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1200 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1208 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v7 + 1217 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1216 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v7 + 1225 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1224 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1216 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1224 as ::core::ffi::c_int) as isize) = 0 as uint16;
}
pub unsafe fn Zoomer_CorpseRottingCopyFunc_0(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1185 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1184 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1193 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1192 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1201 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1200 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1209 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1208 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 1217 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 1216 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 1225 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 1224 as ::core::ffi::c_int) as isize);
    }
}
pub unsafe fn Zoomer_CorpseRottingMoveFunc_2(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1281 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1280 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1289 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1288 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1280 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1288 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1297 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1296 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1305 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1304 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1296 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1304 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v7 + 1313 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1312 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v7 + 1321 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1320 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1312 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1320 as ::core::ffi::c_int) as isize) = 0 as uint16;
}
pub unsafe fn Zoomer_CorpseRottingCopyFunc_2(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1281 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1280 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1289 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1288 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1297 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1296 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1305 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1304 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 1313 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 1312 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 1321 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 1320 as ::core::ffi::c_int) as isize);
    }
}
pub unsafe fn Zoomer_CorpseRottingMoveFunc_4(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1377 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1376 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1385 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1384 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1376 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1384 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1393 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1392 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1401 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1400 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1392 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1400 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v7 + 1409 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1408 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v7 + 1417 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1416 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1408 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1416 as ::core::ffi::c_int) as isize) = 0 as uint16;
}
pub unsafe fn Zoomer_CorpseRottingCopyFunc_4(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1377 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1376 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1385 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1384 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1393 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1392 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1401 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1400 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 1409 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 1408 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 1417 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 1416 as ::core::ffi::c_int) as isize);
    }
}
pub unsafe fn Ripper_CorpseRottingMoveFunc_0(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1473 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1472 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1481 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1480 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1472 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1480 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1489 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1488 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1497 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1496 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1488 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1496 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v7 + 1505 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1504 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v7 + 1513 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1512 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1504 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1512 as ::core::ffi::c_int) as isize) = 0 as uint16;
}
pub unsafe fn Ripper_CorpseRottingCopyFunc_0(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1473 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1472 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1481 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1480 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1489 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1488 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1497 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1496 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 1505 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 1504 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 1513 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 1512 as ::core::ffi::c_int) as isize);
    }
}
pub unsafe fn Ripper_CorpseRottingMoveFunc_2(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1569 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1568 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1577 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1576 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1568 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1576 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1585 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1584 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1593 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1592 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1584 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1592 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v7 + 1601 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1600 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v7 + 1609 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1608 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1600 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1608 as ::core::ffi::c_int) as isize) = 0 as uint16;
}
pub unsafe fn Ripper_CorpseRottingCopyFunc_2(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1569 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1568 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1577 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1576 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1585 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1584 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1593 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1592 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 1601 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 1600 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v8 + 1609 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v7 + 1608 as ::core::ffi::c_int) as isize);
    }
}
pub unsafe fn Skree_CorpseRottingMoveFunc_0(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 801 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 800 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 809 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 808 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 800 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 808 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 817 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 816 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 825 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 824 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 816 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 824 as ::core::ffi::c_int) as isize) = 0 as uint16;
}
pub unsafe fn Skree_CorpseRottingCopyFunc_0(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 801 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 800 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 809 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 808 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 817 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 816 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 825 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 824 as ::core::ffi::c_int) as isize);
    }
}
pub unsafe fn Skree_CorpseRottingMoveFunc_2(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 929 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 928 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 937 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 936 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 928 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 936 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 945 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 944 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 953 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 952 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 944 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 952 as ::core::ffi::c_int) as isize) = 0 as uint16;
}
pub unsafe fn Skree_CorpseRottingCopyFunc_2(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 929 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 928 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 937 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 936 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 945 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 944 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 953 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 952 as ::core::ffi::c_int) as isize);
    }
}
pub unsafe fn Skree_CorpseRottingMoveFunc_4(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1057 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1056 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1065 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1064 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1056 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1064 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1073 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1072 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1081 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1080 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1072 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v5 + 1080 as ::core::ffi::c_int) as isize) = 0 as uint16;
}
pub unsafe fn Skree_CorpseRottingCopyFunc_4(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1057 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1056 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1065 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 1064 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1073 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1072 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 1081 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v5 + 1080 as ::core::ffi::c_int) as isize);
    }
}
pub unsafe fn MotherBrain_CorpseRottingMoveFunc(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) = *(((&raw mut game.g_ram
                as *mut uint8)
                .offset(0x9000 as ::core::ffi::c_int as isize)
                as *mut uint16
                as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize)
                as *mut uint16);
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9012 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) = *(((&raw mut game.g_ram
                as *mut uint8)
                .offset(0x900f as ::core::ffi::c_int as isize)
                as *mut uint16
                as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize)
                .offset(1 as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9000 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x900f as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9022 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) = *(((&raw mut game.g_ram
                as *mut uint8)
                .offset(0x9020 as ::core::ffi::c_int as isize)
                as *mut uint16
                as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize)
                as *mut uint16);
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9032 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) = *(((&raw mut game.g_ram
                as *mut uint8)
                .offset(0x9030 as ::core::ffi::c_int as isize)
                as *mut uint16
                as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9020 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9030 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9042 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9040 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9052 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9050 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
    }
    *(((&raw mut game.g_ram as *mut uint8).offset(0x9040 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x9050 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9062 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9060 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9072 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9070 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
    }
    *(((&raw mut game.g_ram as *mut uint8).offset(0x9060 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x9070 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9082 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9080 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9092 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9090 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
    }
    *(((&raw mut game.g_ram as *mut uint8).offset(0x9080 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x9090 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x90a2 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) = *(((&raw mut game.g_ram
                as *mut uint8)
                .offset(0x90a0 as ::core::ffi::c_int as isize)
                as *mut uint16
                as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize)
                as *mut uint16);
            *(((&raw mut game.g_ram as *mut uint8).offset(0x90b2 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) = *(((&raw mut game.g_ram
                as *mut uint8)
                .offset(0x90b0 as ::core::ffi::c_int as isize)
                as *mut uint16
                as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
        *(((&raw mut game.g_ram as *mut uint8).offset(0x90a0 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x90b0 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x90c2 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) = *(((&raw mut game.g_ram
                as *mut uint8)
                .offset(0x90c0 as ::core::ffi::c_int as isize)
                as *mut uint16
                as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize)
                as *mut uint16);
            *(((&raw mut game.g_ram as *mut uint8).offset(0x90d2 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) = *(((&raw mut game.g_ram
                as *mut uint8)
                .offset(0x90d0 as ::core::ffi::c_int as isize)
                as *mut uint16
                as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
        *(((&raw mut game.g_ram as *mut uint8).offset(0x90c0 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x90d0 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn MotherBrain_CorpseRottingCopyFunc(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int
        && (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9000 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9012 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x900f as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize)
                .offset(1 as ::core::ffi::c_int as isize) as *mut uint16);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int
        && (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9022 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9020 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9032 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9030 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9042 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9040 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9052 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9050 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9062 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9060 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9072 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9070 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9082 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9080 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9092 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9090 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int
        && (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x90a2 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x90a0 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x90b2 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x90b0 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x90c2 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) = *(((&raw mut game.g_ram
                as *mut uint8)
                .offset(0x90c0 as ::core::ffi::c_int as isize)
                as *mut uint16
                as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize)
                as *mut uint16);
            *(((&raw mut game.g_ram as *mut uint8).offset(0x90d2 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) = *(((&raw mut game.g_ram
                as *mut uint8)
                .offset(0x90d0 as ::core::ffi::c_int as isize)
                as *mut uint16
                as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
    }
}
pub unsafe fn sub_A9ECD0(mut k: uint16, mut jp: *const uint16, game: &mut Game) -> *const uint16 {
    let mut v2: uint16 = (0xa9d910 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as ::core::ffi::c_int, game);
    if (*E).dms_var_08 != 0 {
        v2 = (0xa9da08 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    (*E).dms_var_A = v2;
    return jp;
}
pub unsafe fn Shitroid_Func_1(mut k: uint16, mut j: uint16, game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(j as ::core::ffi::c_int, game);
    let mut G: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    let mut r18: uint16 = ((*G).base.x_width as ::core::ffi::c_int
        + (*E).base.x_width as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint16;
    let mut v4: uint16 = abs16(
        ((*E).base.x_pos as ::core::ffi::c_int - (*G).base.x_pos as ::core::ffi::c_int) as uint16,
    );
    let mut v5: bool = v4 as ::core::ffi::c_int >= r18 as ::core::ffi::c_int;
    if (v4 as ::core::ffi::c_int) < r18 as ::core::ffi::c_int {
        r18 = ((*G).base.y_height as ::core::ffi::c_int
            + (*E).base.y_height as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint16;
        return (abs16(
            ((*E).base.y_pos as ::core::ffi::c_int - (*G).base.y_pos as ::core::ffi::c_int)
                as uint16,
        ) as ::core::ffi::c_int
            >= r18 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
    }
    return v5 as uint8;
}
pub unsafe fn Shitroid_Func_2(mut k: uint16, mut rect: Rect16U, game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    let mut R26: uint16 = ((*E).base.x_width as ::core::ffi::c_int
        + rect.w as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 =
        abs16((rect.x as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int) as uint16);
    let mut v3: bool = v2 as ::core::ffi::c_int >= R26 as ::core::ffi::c_int;
    if (v2 as ::core::ffi::c_int) < R26 as ::core::ffi::c_int {
        R26 = ((*E).base.y_height as ::core::ffi::c_int
            + rect.h as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint16;
        return (abs16(
            (rect.y as ::core::ffi::c_int - (*E).base.y_pos as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int
            >= R26 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
    }
    return v3 as uint8;
}
pub unsafe fn Shitroid_Init(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 4094 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_DisableSamusColl as ::core::ffi::c_int
            | kEnemyProps_BlockPlasmaBeam as ::core::ffi::c_int))
        as uint16;
    (*E).base.palette_index = 1024 as uint16;
    (*E).base.current_instruction = addr_kShitroid_Ilist_F90E as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    let mut v2: uint16 = (0xa9efe6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | (kEnemyProps_Tangible as ::core::ffi::c_int
                | kEnemyProps_Invisible as ::core::ffi::c_int))
            as uint16;
        v2 = (0xa9efdf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    (*E).shitr_var_A = v2;
    (*E).shitr_var_B = 0 as uint16;
    (*E).shitr_var_C = 0 as uint16;
    (*E).shitr_var_E = 10 as uint16;
    (*E).shitr_parameter_2 = 0 as uint16;
    WriteColorsToTargetPalette(
        0xa9 as uint8,
        0x120 as uint16,
        addr_word_A9F8C6 as ::core::ffi::c_int as uint16,
        0x10 as uint16,
        game,
    );
    WriteColorsToTargetPalette(
        0xa9 as uint8,
        0x140 as uint16,
        addr_kShitroidInCutscene_Palette as ::core::ffi::c_int as uint16,
        0x10 as uint16,
        game,
    );
    WriteColorsToTargetPalette(
        0xa9 as uint8,
        0x1e0 as uint16,
        addr_kDeadSidehopper_Palette_0 as ::core::ffi::c_int as uint16,
        0x10 as uint16,
        game,
    );
}
pub unsafe fn Shitroid_Powerbomb(game: &mut Game) {
    if (*Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .shitr_parameter_2
        != 0
    {
        Shitroid_Func_26(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            game,
        );
    }
    Shitroid_Main(game);
}
pub unsafe fn CallShitroidFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        11136991 => {
            Shitroid_Func_3(k, game);
            return;
        }
        11136998 => {
            Shitroid_Func_4(game);
            return;
        }
        11137067 => {
            Shitroid_Func_5(k, game);
            return;
        }
        11137079 => {
            Shitroid_Func_6(k, game);
            return;
        }
        11137097 => {
            Shitroid_Func_7(k, game);
            return;
        }
        11137133 => {
            Shitroid_Func_8(k, game);
            return;
        }
        11137172 => {
            Shitroid_Func_9(k, game);
            return;
        }
        11137254 => {
            Shitroid_Func_10(k, game);
            return;
        }
        11137317 => {
            Shitroid_Func_11(k, game);
            return;
        }
        11137336 => {
            Shitroid_Func_12(k, game);
            return;
        }
        11137408 => {
            Shitroid_Func_13(k, game);
            return;
        }
        11137530 => {
            Shitroid_Func_14(k, game);
            return;
        }
        11137550 => {
            Shitroid_Func_15(k, game);
            return;
        }
        11137563 => {
            Shitroid_Func_16(k, game);
            return;
        }
        11137698 => {
            Shitroid_Func_17(k, game);
            return;
        }
        11137710 => {
            Shitroid_Func_18(k, game);
            return;
        }
        11137728 => {
            Shitroid_Func_19(k, game);
            return;
        }
        11137787 => {
            Shitroid_Func_20(game);
            return;
        }
        11137828 => {
            Shitroid_Func_21(game);
            return;
        }
        11137888 => {
            Shitroid_Func_22(k, game);
            return;
        }
        11137901 => {
            Shitroid_Func_23(game);
            return;
        }
        11137955 => {
            Shitroid_Func_24(k, game);
            return;
        }
        11137982 => {
            Shitroid_Func_25(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn Shitroid_Main(game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.health = 0x7fff as uint16;
    CallShitroidFunc(
        ((*E).shitr_var_A as ::core::ffi::c_int | 0xa90000 as ::core::ffi::c_int) as uint32,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    MoveEnemyWithVelocity(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc400 as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
    {
        Shitroid_HandleNormalPalette(game);
    }
}
pub unsafe fn Shitroid_Func_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    (*E).shitr_var_B = 0 as uint16;
    (*E).shitr_var_C = 0 as uint16;
}
pub unsafe fn Shitroid_Func_4(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 513 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) = 512 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20 as ::core::ffi::c_int as isize)
            as *mut uint16) = *(&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20 as ::core::ffi::c_int as isize)
            .offset(0 as ::core::ffi::c_int as isize) as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20 as ::core::ffi::c_int as isize)
            .offset(2 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
            *(&raw mut game.g_ram as *mut uint8)
                .offset(0xcd20 as ::core::ffi::c_int as isize)
                .offset(2 as ::core::ffi::c_int as isize) as uint16;
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x30 as uint8,
                field_1: 0x3 as uint8,
                field_2: 0xb767 as VoidP,
            },
            game,
        );
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x1f as uint8,
                field_1: 0x3 as uint8,
                field_2: 0xb767 as VoidP,
            },
            game,
        );
        let mut E: *mut Enemy_Shitroid = Get_Shitroid(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
            game,
        );
        (*E).shitr_var_A =
            (0xa9f02b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).shitr_var_04 = 1 as uint16;
    }
}
pub unsafe fn Shitroid_Func_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    (*E).shitr_var_A = (0xa9f037 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).shitr_var_F = 464 as uint16;
    Shitroid_Func_6(k, game);
}
pub unsafe fn Shitroid_Func_6(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if (*E).shitr_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        QueueMusic_Delayed8(5 as uint16, game);
        (*E).shitr_var_A =
            (0xa9f049 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        Shitroid_Func_7(k, game);
    }
}
pub unsafe fn Shitroid_Func_7(mut k: uint16, game: &mut Game) {
    let mut rect: Rect16U = Rect16U {
        x: 584 as uint16,
        y: 74 as uint16,
        w: 1 as uint16,
        h: 1 as uint16,
    };
    Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0xf as uint16, rect.x, rect.y, game);
    if Shitroid_Func_2(k, rect, game) == 0 {
        (*Get_Shitroid(k as ::core::ffi::c_int, game)).shitr_var_A =
            (0xa9f06d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Shitroid_Func_8(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shitroid =
        Get_Shitroid(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        k,
        0xf as uint16,
        (*E).base.x_pos,
        ((*E).base.y_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16,
        game,
    );
    if Shitroid_Func_1(
        k,
        (k as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16,
        game,
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        (*Get_Shitroid(k as ::core::ffi::c_int, game)).shitr_var_A =
            (0xa9f094 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Shitroid_Func_9(mut k: uint16, game: &mut Game) {
    let mut E1: *mut Enemy_Shitroid =
        Get_Shitroid(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    if Shitroid_AccelerateTowardsPoint(
        k,
        0x200 as uint16,
        (*E1).base.x_pos,
        ((*E1).base.y_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16,
        game,
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
        (*E).shitr_var_B = 0 as uint16;
        (*E).shitr_var_C = 0 as uint16;
        (*E).base.x_pos = (*E1).base.x_pos;
        (*E).base.y_pos =
            ((*E1).base.y_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction = addr_kShitroid_Ilist_F924 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).shitr_var_A =
            (0xa9f0e6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).shitr_var_E = 1 as uint16;
        (*E).shitr_parameter_2 = 0 as uint16;
        (*E).shitr_var_F = 320 as uint16;
    }
}
pub unsafe fn Shitroid_Func_10(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    let mut v2: ::core::ffi::c_int = ((*E).base.frame_counter as ::core::ffi::c_int
        & 6 as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int;
    let mut E1: *mut Enemy_Shitroid =
        Get_Shitroid(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E1).base.x_pos as ::core::ffi::c_int
        + g_word_A993BB[v2 as usize] as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E1).base.y_pos as ::core::ffi::c_int
        + g_word_A993C3[v2 as usize] as ::core::ffi::c_int
        - 32 as ::core::ffi::c_int) as uint16;
    let fresh13 = (*E).shitr_var_F;
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if fresh13 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).shitr_var_A =
            (0xa9f125 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction = addr_kShitroid_Ilist_F906 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).shitr_var_E = 10 as uint16;
    }
}
pub unsafe fn Shitroid_Func_11(mut k: uint16, game: &mut Game) {
    (*Get_Shitroid(0x40 as ::core::ffi::c_int, game)).shitr_var_08 = 1 as uint16;
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    (*E).shitr_var_A = (0xa9f138 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).shitr_var_F = 192 as uint16;
    Shitroid_Func_12(k, game);
}
pub unsafe fn Shitroid_Func_12(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        k,
        0 as uint16,
        (*E).base.x_pos,
        104 as uint16,
        game,
    );
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if (*E).shitr_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).shitr_var_A =
            (0xa9f180 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).shitr_parameter_2 = 1 as uint16;
        let ref mut fresh14 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh14 = (*fresh14 as ::core::ffi::c_int | 0x100 as ::core::ffi::c_int) as uint16;
        let ref mut fresh15 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xcd20 as ::core::ffi::c_int as isize)
            .offset(2 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut uint16);
        *fresh15 = (*fresh15 as ::core::ffi::c_int | 0x100 as ::core::ffi::c_int) as uint16;
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x30 as uint8,
                field_1: 0x3 as uint8,
                field_2: 0xb763 as VoidP,
            },
            game,
        );
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x1f as uint8,
                field_1: 0x3 as uint8,
                field_2: 0xb763 as VoidP,
            },
            game,
        );
    }
}
pub unsafe fn Shitroid_Func_13(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    let mut v2: uint16 = abs16(
        ((*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
    );
    let mut shitro_var_02: uint16 = 0;
    if v2 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
        shitro_var_02 = (*E).shitr_var_02;
        if shitro_var_02 != 0 {
            shitro_var_02 = shitro_var_02.wrapping_sub(1);
            if shitro_var_02 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                shitro_var_02 = 0 as uint16;
            }
        }
    } else {
        shitro_var_02 = ((v2 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int)
            as ::core::ffi::c_int
            + (*E).shitr_var_02 as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as uint16;
    }
    (*E).shitr_var_02 = shitro_var_02;
    if shitro_var_02 as ::core::ffi::c_int >= 0x100 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 512 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        (*Get_Shitroid(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
            game,
        ))
        .shitr_var_A = (0xa9f1fa as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        let mut shitro_var_01: uint16 = (*E).shitr_var_01;
        let mut v5: uint16 = 0;
        if shitro_var_01 != 0 {
            (*E).shitr_var_01 =
                (shitro_var_01 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
            v5 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16);
        } else {
            v5 = 80 as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xfff as ::core::ffi::c_int
                >= 0xfe0 as ::core::ffi::c_int
            {
                (*E).shitr_var_01 = 32 as uint16;
            }
        }
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            0xa as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16),
            v5,
            game,
        );
    };
}
pub unsafe fn Shitroid_Func_14(mut k: uint16, game: &mut Game) {
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        k,
        0xf as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 32 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn Shitroid_Func_15(mut k: uint16, game: &mut Game) {
    CallSomeSamusCode(0x12 as uint16, game);
    (*Get_Shitroid(0 as ::core::ffi::c_int, game)).shitr_var_A =
        (0xa9f21b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Shitroid_Func_16(k, game);
}
pub unsafe fn Shitroid_Func_16(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int)
        < 2 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd2 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        (*E).shitr_var_A =
            (0xa9f2a2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).shitr_var_B = 0 as uint16;
        (*E).shitr_var_C = 0 as uint16;
        Enemy_SetInstrList(
            k,
            addr_kShitroid_Ilist_F906 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).shitr_var_E = 10 as uint16;
        CallSomeSamusCode(0x13 as uint16, game);
        SomeMotherBrainScripts(0 as uint16, game);
        (*E).shitr_var_04 = 0 as uint16;
        QueueMusic_Delayed8(7 as uint16, game);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
            as *mut uint16) = 8 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd2 as ::core::ffi::c_int as isize)
            as *mut uint16) = 5 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
                as *mut uint16) = 2 as uint16;
        }
        let mut v2: ::core::ffi::c_int = ((*E).base.frame_counter as ::core::ffi::c_int
            & 6 as ::core::ffi::c_int)
            >> 1 as ::core::ffi::c_int;
        (*E).base.x_pos = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + g_word_A993BB[v2 as usize] as ::core::ffi::c_int) as uint16;
        (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + g_word_A993C3[v2 as usize] as ::core::ffi::c_int
            - 20 as ::core::ffi::c_int) as uint16;
        Samus_DamageDueToShitroid(game);
    };
}
pub unsafe fn Shitroid_Func_17(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    (*E).shitr_var_A = (0xa9f2ae as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).shitr_var_F = 120 as uint16;
    Shitroid_Func_18(k, game);
}
pub unsafe fn Shitroid_Func_18(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if (*E).shitr_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).shitr_var_A =
            (0xa9f2c0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).shitr_var_F = 192 as uint16;
        Shitroid_Func_19(k, game);
    }
}
pub unsafe fn Shitroid_Func_19(mut k: uint16, game: &mut Game) {
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        0 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16),
        104 as uint16,
        game,
    );
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if (*E).shitr_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        QueueSfx2_Max6(0x7d as uint16, game);
        (*E).shitr_var_A =
            (0xa9f2fb as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).shitr_var_F = 88 as uint16;
        (*E).base.current_instruction = addr_kShitroid_Ilist_F924 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        Shitroid_Func_20(game);
    }
}
pub unsafe fn Shitroid_Func_20(game: &mut Game) {
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        0 as uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 64 as ::core::ffi::c_int) as uint16,
        100 as uint16,
        game,
    );
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if (*E).shitr_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).shitr_var_A =
            (0xa9f324 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).shitr_var_F = 88 as uint16;
        Shitroid_Func_21(game);
    }
}
pub unsafe fn Shitroid_Func_21(game: &mut Game) {
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        0 as uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 96 as ::core::ffi::c_int) as uint16,
        104 as uint16,
        game,
    );
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if (*E).shitr_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).shitr_var_A =
            (0xa9f3a3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).shitr_var_F = 256 as uint16;
        (*E).base.current_instruction = addr_kShitroid_Ilist_F93A as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn Shitroid_Func_22(mut k: uint16, game: &mut Game) {
    QueueSfx2_Max6(0x52 as uint16, game);
    (*Get_Shitroid(k as ::core::ffi::c_int, game)).shitr_var_A =
        (0xa9f36d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Shitroid_Func_23(game);
}
pub unsafe fn Shitroid_Func_23(game: &mut Game) {
    let mut rect: Rect16U = Rect16U {
        x: -(128 as ::core::ffi::c_int) as uint16,
        y: 64 as uint16,
        w: 8 as uint16,
        h: 8 as uint16,
    };
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        0 as uint16,
        rect.x,
        rect.y,
        game,
    );
    if Shitroid_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        rect,
        game,
    ) == 0
    {
        let mut E: *mut Enemy_Shitroid = Get_Shitroid(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
            game,
        );
        (*E).shitr_var_B = 0 as uint16;
        (*E).shitr_var_C = 0 as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_DisableSamusColl as ::core::ffi::c_int
                | kEnemyProps_Invisible as ::core::ffi::c_int))
            as uint16;
        (*E).shitr_var_A =
            (0xa9efdf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Shitroid_Func_24(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if (*E).shitr_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        SomeMotherBrainScripts(2 as uint16, game);
        (*E).shitr_parameter_2 = 1 as uint16;
        (*E).shitr_var_A =
            (0xa9f3be as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        Shitroid_Func_25(k, game);
    } else {
        Shitroid_Func_27(k, game);
    };
}
pub unsafe fn Shitroid_Func_25(mut k: uint16, game: &mut Game) {
    if Shitroid_Func_27(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        Shitroid_F3C4(k, game);
    }
}
pub unsafe fn Shitroid_F3C4(mut k: uint16, game: &mut Game) {
    (*Get_Shitroid(k as ::core::ffi::c_int, game)).shitr_var_A =
        (0xa9f360 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Shitroid_Func_26(mut k: uint16, game: &mut Game) {
    if (*Get_Shitroid(k as ::core::ffi::c_int, game)).shitr_var_A as ::core::ffi::c_int
        == 0xa9f3be as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        Shitroid_F3C4(k, game);
    }
}
pub unsafe fn Shitroid_Func_27(mut k: uint16, game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    let mut v2: uint16 = abs16(
        ((*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
    );
    let mut shitro_var_02: uint16 = 0;
    if v2 as ::core::ffi::c_int >= 2 as ::core::ffi::c_int {
        shitro_var_02 = (*E).shitr_var_02;
        if shitro_var_02 != 0 {
            shitro_var_02 = shitro_var_02.wrapping_sub(1);
            if shitro_var_02 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                shitro_var_02 = 0 as uint16;
            }
        }
    } else {
        shitro_var_02 = ((v2 as ::core::ffi::c_int >= 2 as ::core::ffi::c_int)
            as ::core::ffi::c_int
            + (*E).shitr_var_02 as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as uint16;
    }
    (*E).shitr_var_02 = shitro_var_02;
    let mut shitro_var_01: uint16 = (*E).shitr_var_01;
    let mut v5: uint16 = 0;
    if shitro_var_01 != 0 {
        (*E).shitr_var_01 =
            (shitro_var_01 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        v5 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 18 as ::core::ffi::c_int) as uint16;
    } else {
        v5 = 80 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xfff as ::core::ffi::c_int
            >= 0xfe0 as ::core::ffi::c_int
        {
            (*E).shitr_var_01 = 32 as uint16;
        }
    }
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        8 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16),
        v5,
        game,
    );
    return ((*Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .shitr_var_02 as ::core::ffi::c_int
        >= 0x400 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 128 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0) as ::core::ffi::c_int as uint8;
}
pub unsafe fn Shitroid_GraduallyAccelerateTowardsPt(
    mut k: uint16,
    mut j: uint16,
    mut r18: uint16,
    mut r20: uint16,
    mut r26: uint16,
    game: &mut Game,
) {
    let mut v3: int16 = 0;
    let mut v5: int16 = 0;
    let mut v6: int16 = 0;
    let mut shitr_var_C: int16 = 0;
    let mut v9: int16 = 0;
    let mut r24: uint16 = g_byte_A9F56A[j as usize] as uint16;
    Shitroid_GraduallyAccelerateHoriz(k, r18, r24, r26, game);
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    v3 = ((*E).base.y_pos as ::core::ffi::c_int - r20 as ::core::ffi::c_int) as int16;
    if v3 != 0 {
        if v3 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            let mut RegWord: uint16 = SnesDivide(v3 as uint16, r24 as uint8);
            if RegWord == 0 {
                RegWord = 1 as uint16;
            }
            let mut r22: uint16 = RegWord;
            shitr_var_C = (*E).shitr_var_C as int16;
            if shitr_var_C as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                shitr_var_C = (shitr_var_C as ::core::ffi::c_int
                    - 8 as ::core::ffi::c_int
                    - r22 as ::core::ffi::c_int) as int16;
            }
            v9 = (shitr_var_C as ::core::ffi::c_int - r22 as ::core::ffi::c_int) as int16;
            if v9 as ::core::ffi::c_int + 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v9 = -(1280 as ::core::ffi::c_int) as int16;
            }
            (*E).shitr_var_C = v9 as uint16;
        } else {
            let mut v4: uint16 = SnesDivide(
                (r20 as ::core::ffi::c_int - (*E).base.y_pos as ::core::ffi::c_int) as uint16,
                r24 as uint8,
            );
            if v4 == 0 {
                v4 = 1 as uint16;
            }
            let mut r22_0: uint16 = v4;
            v5 = (*E).shitr_var_C as int16;
            if (v5 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                v5 = (v5 as ::core::ffi::c_int
                    + (r22_0 as ::core::ffi::c_int + 8 as ::core::ffi::c_int))
                    as int16;
            }
            v6 = (r22_0 as ::core::ffi::c_int + v5 as ::core::ffi::c_int) as int16;
            if v6 as ::core::ffi::c_int - 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v6 = 1280 as int16;
            }
            (*E).shitr_var_C = v6 as uint16;
        }
    }
}
pub unsafe fn Shitroid_Func_GraduallyAccelerateTowards0x400(
    mut k: uint16,
    mut j: uint16,
    mut r18: uint16,
    mut r20: uint16,
    game: &mut Game,
) {
    Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 1024 as uint16, game);
}
pub unsafe fn Shitroid_GraduallyAccelerateTowards0x4(
    mut k: uint16,
    mut j: uint16,
    mut r18: uint16,
    mut r20: uint16,
    game: &mut Game,
) {
    Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 4 as uint16, game);
}
pub unsafe fn Shitroid_GraduallyAccelerateTowards0x8(
    mut k: uint16,
    mut j: uint16,
    mut r18: uint16,
    mut r20: uint16,
    game: &mut Game,
) {
    Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 8 as uint16, game);
}
pub unsafe fn Shitroid_GraduallyAccelerateTowards0x10(
    mut k: uint16,
    mut j: uint16,
    mut r18: uint16,
    mut r20: uint16,
    game: &mut Game,
) {
    Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 16 as uint16, game);
}
pub unsafe fn Shitroid_GraduallyAccelerateHoriz(
    mut k: uint16,
    mut r18: uint16,
    mut r24: uint16,
    mut r26: uint16,
    game: &mut Game,
) {
    let mut v2: int16 = 0;
    let mut v4: int16 = 0;
    let mut v5: int8 = 0;
    let mut v7: int16 = 0;
    let mut shitr_var_B: int16 = 0;
    let mut v11: int16 = 0;
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    v2 = ((*E).base.x_pos as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as int16;
    if v2 != 0 {
        if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            let mut RegWord: uint16 = SnesDivide(v2 as uint16, r24 as uint8);
            if RegWord == 0 {
                RegWord = 1 as uint16;
            }
            let mut r22: uint16 = RegWord;
            shitr_var_B = (*E).shitr_var_B as int16;
            if shitr_var_B as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                let mut v13: uint16 = (*E).shitr_var_B;
                v5 = (Shitroid_CheckIfOnScreen(k, game) as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int) as int8;
                let mut v10: uint16 = v13;
                if v5 != 0 {
                    v10 = (v13 as ::core::ffi::c_int - r26 as ::core::ffi::c_int) as uint16;
                }
                shitr_var_B = (v10 as ::core::ffi::c_int
                    - 8 as ::core::ffi::c_int
                    - r22 as ::core::ffi::c_int) as int16;
            }
            v11 = (shitr_var_B as ::core::ffi::c_int - r22 as ::core::ffi::c_int) as int16;
            if v11 as ::core::ffi::c_int + 2048 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v11 = -(2048 as ::core::ffi::c_int) as int16;
            }
            (*E).shitr_var_B = v11 as uint16;
        } else {
            let mut v3: uint16 = SnesDivide(
                (r18 as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
                r24 as uint8,
            );
            if v3 == 0 {
                v3 = 1 as uint16;
            }
            let mut r22_0: uint16 = v3;
            v4 = (*E).shitr_var_B as int16;
            if (v4 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                let mut v12: uint16 = (*E).shitr_var_B;
                v5 = (Shitroid_CheckIfOnScreen(k, game) as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int) as int8;
                let mut v6: uint16 = v12;
                if v5 != 0 {
                    v6 = (r26 as ::core::ffi::c_int + v12 as ::core::ffi::c_int) as uint16;
                }
                v4 = (r22_0 as ::core::ffi::c_int
                    + v6 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int) as int16;
            }
            v7 = (r22_0 as ::core::ffi::c_int + v4 as ::core::ffi::c_int) as int16;
            if v7 as ::core::ffi::c_int - 2048 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v7 = 2048 as int16;
            }
            (*E).shitr_var_B = v7 as uint16;
        }
    }
}
pub unsafe fn Shitroid_CheckIfOnScreen(mut k: uint16, game: &mut Game) -> uint8 {
    let mut y_pos: int16 = 0;
    let mut v3: int16 = 0;
    let mut x_pos: int16 = 0;
    let mut v5: int16 = 0;
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    y_pos = (*E).base.y_pos as int16;
    let mut result: uint8 = 1 as uint8;
    if y_pos as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        v3 = (y_pos as ::core::ffi::c_int + 96 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16;
        if v3 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            if v3 as ::core::ffi::c_int - 416 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                x_pos = (*E).base.x_pos as int16;
                if x_pos as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                    v5 = (x_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int
                        - *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x911 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int)
                        as int16;
                    if v5 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                        if v5 as ::core::ffi::c_int - 288 as ::core::ffi::c_int
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
pub unsafe fn Shitroid_AccelerateTowardsPoint(
    mut k: uint16,
    mut a: uint16,
    mut r18: uint16,
    mut r20: uint16,
    game: &mut Game,
) -> uint8 {
    let mut R28: uint16 = 0 as uint16;
    R28 = (R28 as ::core::ffi::c_int
        + Shitroid_AccelerateTowardsX(k, r18, a, game) as ::core::ffi::c_int) as uint16;
    R28 = (R28 as ::core::ffi::c_int
        + Shitroid_AccelerateTowardsY(k, r20, a, game) as ::core::ffi::c_int) as uint16;
    R28 = (R28 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    return (R28 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint8;
}
pub unsafe fn Shitroid_AccelerateTowardsY(
    mut k: uint16,
    mut r20: uint16,
    mut r22: uint16,
    game: &mut Game,
) -> uint16 {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    let mut result: uint16 = 0 as uint16;
    let mut v2: int16 =
        ((*E).base.y_pos as ::core::ffi::c_int - r20 as ::core::ffi::c_int) as int16;
    if v2 == 0 {
        result = result.wrapping_add(1);
    } else if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        let mut v7: uint16 =
            ((*E).shitr_var_C as ::core::ffi::c_int - r22 as ::core::ffi::c_int) as uint16;
        if v7 as ::core::ffi::c_int + 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        {
            v7 = -(1280 as ::core::ffi::c_int) as uint16;
        }
        (*E).shitr_var_C = v7;
        if ((*E).base.y_pos as ::core::ffi::c_int
            + (v7 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
            - r20 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            <= 0 as ::core::ffi::c_int
        {
            (*E).shitr_var_C = 0 as uint16;
            result = result.wrapping_add(1);
        }
    } else {
        let mut v3: uint16 =
            (r22 as ::core::ffi::c_int + (*E).shitr_var_C as ::core::ffi::c_int) as uint16;
        if v3 as ::core::ffi::c_int - 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        {
            v3 = 1280 as uint16;
        }
        (*E).shitr_var_C = v3;
        if ((*E).base.y_pos as ::core::ffi::c_int
            + (v3 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
            - r20 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            (*E).shitr_var_C = 0 as uint16;
            result = result.wrapping_add(1);
        }
    }
    return result;
}
pub unsafe fn Shitroid_AccelerateTowardsX(
    mut k: uint16,
    mut r18: uint16,
    mut r22: uint16,
    game: &mut Game,
) -> uint16 {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    let mut result: uint16 = 0 as uint16;
    if ((*E).base.x_pos as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        let mut v6: uint16 =
            ((*E).shitr_var_B as ::core::ffi::c_int - r22 as ::core::ffi::c_int) as uint16;
        if v6 as ::core::ffi::c_int + 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        {
            v6 = -(1280 as ::core::ffi::c_int) as uint16;
        }
        (*E).shitr_var_B = v6;
        if ((*E).base.x_pos as ::core::ffi::c_int
            + (v6 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            <= 0 as ::core::ffi::c_int
        {
            (*E).shitr_var_B = 0 as uint16;
            result = result.wrapping_add(1);
        }
    } else {
        let mut v2: uint16 =
            (r22 as ::core::ffi::c_int + (*E).shitr_var_B as ::core::ffi::c_int) as uint16;
        if v2 as ::core::ffi::c_int - 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        {
            v2 = 1280 as uint16;
        }
        (*E).shitr_var_B = v2;
        if ((*E).base.x_pos as ::core::ffi::c_int
            + (v2 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            (*E).shitr_var_B = 0 as uint16;
            result = result.wrapping_add(1);
        }
    }
    return result;
}
pub unsafe fn Shitroid_HandleNormalPalette(game: &mut Game) {
    Shitroid_HandleCutscenePalette_Common(
        addr_word_A9F6D1 as ::core::ffi::c_int as uint16,
        330 as uint16,
        game,
    );
}
pub unsafe fn Shitroid_HandleCutscenePalette(game: &mut Game) {
    Shitroid_HandleCutscenePalette_Common(
        addr_word_A9F6D1 as ::core::ffi::c_int as uint16,
        490 as uint16,
        game,
    );
}
pub unsafe fn Shitroid_HandleCutscenePalette_LowHealth(game: &mut Game) {
    Shitroid_HandleCutscenePalette_Common(
        addr_word_A9F711 as ::core::ffi::c_int as uint16,
        490 as uint16,
        game,
    );
}
pub unsafe fn Shitroid_HandleCutscenePalette_Common(
    mut r22: uint16,
    mut r18: uint16,
    game: &mut Game,
) {
    let mut shitro_var_D_high: int8 = 0;
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    shitro_var_D_high = *(&raw mut (*E).shitr_var_D as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as int8;
    if shitro_var_D_high != 0 {
        *(&raw mut (*E).shitr_var_D as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) = (shitro_var_D_high as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint8;
    } else {
        *(&raw mut (*E).shitr_var_D as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) = (*E).shitr_var_E as uint8;
        let mut v3: uint8 = (*(&raw mut (*E).shitr_var_D as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int) as uint8;
        *(&raw mut (*E).shitr_var_D as *mut uint8).offset(0 as ::core::ffi::c_int as isize) = v3;
        let mut v4: uint16 = Shitroid_HandleCrySoundEffect(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            v3 as uint16,
            game,
        );
        WriteColorsToPalette(
            r18,
            0xa9 as uint8,
            (r22 as ::core::ffi::c_int + 8 as ::core::ffi::c_int * v4 as ::core::ffi::c_int)
                as uint16,
            4 as uint16,
            game,
        );
    };
}
pub unsafe fn Shitroid_HandleCrySoundEffect(
    mut k: uint16,
    mut a: uint16,
    game: &mut Game,
) -> uint16 {
    if a as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
        let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
        if (*E).shitr_var_04 != 0 {
            let mut v3: uint16 =
                ((*E).shitr_var_05 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            (*E).shitr_var_05 = v3;
            if v3 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
                (*E).shitr_var_05 = 0 as uint16;
                let mut v4: uint16 = 114 as uint16;
                if ((*E).shitr_var_E as ::core::ffi::c_int) < 0xa as ::core::ffi::c_int {
                    v4 = 120 as uint16;
                }
                QueueSfx2_Max6(v4, game);
            }
        }
        return 5 as uint16;
    }
    return a;
}
pub unsafe fn Shitroid_Touch(game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).shitr_parameter_2 != 0 {
        Shitroid_Func_26(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            game,
        );
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 3 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 512 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            let mut r18: uint16 = (0x80 as ::core::ffi::c_int
                - CalculateAngleFromXY(
                    (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xaf6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
                    (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xafa as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - (*E).base.y_pos as ::core::ffi::c_int) as uint16,
                    game,
                ) as ::core::ffi::c_int
                + 0x80 as ::core::ffi::c_int) as uint8 as uint16;
            (*E).shitr_var_B = ((*E).shitr_var_B as ::core::ffi::c_int
                + Math_MultBySin(0x40 as uint16, r18) as ::core::ffi::c_int)
                as uint16;
            (*E).shitr_var_C = ((*E).shitr_var_C as ::core::ffi::c_int
                + Math_MultByCos(0x40 as uint16, r18) as ::core::ffi::c_int)
                as uint16;
        } else if (*E).shitr_var_A as ::core::ffi::c_int
            == 0xa9f1fa as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            if Shitroid_AccelerateTowardsPoint(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                0x200 as uint16,
                *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                (*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 32 as ::core::ffi::c_int) as uint16,
                game,
            ) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                != 0
            {
                (*E).base.current_instruction =
                    addr_kShitroid_Ilist_F924 as ::core::ffi::c_int as uint16;
                (*E).base.instruction_timer = 1 as uint16;
                (*E).base.timer = 0 as uint16;
                (*E).shitr_var_E = 1 as uint16;
                (*E).shitr_parameter_2 = 0 as uint16;
                (*E).shitr_var_B = 0 as uint16;
                (*E).shitr_var_C = 0 as uint16;
                (*E).shitr_var_A =
                    (0xa9f20e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            }
        } else if (*E).shitr_var_A as ::core::ffi::c_int
            == 0xa9f180 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            (*E).shitr_var_A =
                (0xa9f1fa as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn Shitroid_Shot(game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).shitr_parameter_2 != 0 {
        Shitroid_Func_26(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            game,
        );
        let mut r18: uint16 = -(CalculateAngleFromXY(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
            (*((&raw mut game.g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                - (*E).base.y_pos as ::core::ffi::c_int) as uint16,
            game,
        ) as ::core::ffi::c_int) as uint8 as uint16;
        let mut v1: uint16 = (8 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x18a6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ) as ::core::ffi::c_int) as uint16;
        if v1 as ::core::ffi::c_int >= 0xf0 as ::core::ffi::c_int {
            v1 = 240 as uint16;
        }
        (*E).shitr_var_B = ((*E).shitr_var_B as ::core::ffi::c_int
            + Math_MultBySin(v1, r18) as ::core::ffi::c_int) as uint16;
        (*E).shitr_var_C = ((*E).shitr_var_C as ::core::ffi::c_int
            + Math_MultByCos(v1, r18) as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Shitroid_Instr_3(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return addr_kShitroid_Ilist_F90E as ::core::ffi::c_int as uintptr_t as *const uint16;
}
pub unsafe fn Shitroid_Instr_4(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return addr_kShitroid_Ilist_F924 as ::core::ffi::c_int as uintptr_t as *const uint16;
}
pub unsafe fn Shitroid_Instr_6(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return addr_kShitroid_Ilist_F93A as ::core::ffi::c_int as uintptr_t as *const uint16;
}
pub unsafe fn Shitroid_Instr_5(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return *jp.offset(0 as ::core::ffi::c_int as isize) as uintptr_t as *const uint16;
    }
    QueueSfx2_Max6(0x52 as uint16, game);
    return jp.offset(1 as ::core::ffi::c_int as isize);
}
