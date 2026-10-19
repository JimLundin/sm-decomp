use crate::game::sm_80::{NextRandom, QueueSfx2_Max6, QueueSfx2_Max9, QueueSfx3_Max6};
use crate::game::sm_86::SpawnEprojWithGfx;
use crate::game::sm_88::SpawnMorphBallEyeBeamHdma;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game::sm_a0::{
    Abs16, CalculateAngleFromXY, CalculateAngleOfSamusFromEnemy, CallEnemyPreInstr,
    CheckIfEnemyIsOnScreen, CheckIfEnemyTouchesSamus, CosineMult8bit, CosineMult8bitFull,
    EnemyDeathAnimation, EnemyFunc_ACA8, EnemyFunc_ADA3, EnemyFunc_B7A1, EnemyFunc_C8AD,
    EnemyGrappleDeath, Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes, Enemy_SwitchToFrozenAi,
    GetSamusEnemyDelta_X, GetSamusEnemyDelta_Y, IsSamusWithinEnemy_X, IsSamusWithinEnemy_Y,
    NormalEnemyFrozenAI, NormalEnemyPowerBombAi, NormalEnemyShotAi,
    NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAi, SamusHurtFromGrapple,
    SamusLatchesOnWithGrappleNoInvinc, SignExtend8, SineMult8bit, SineMult8bitFull,
    SwitchEnemyAiToMainAi,
};
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::game_state::Game;
use crate::rtl::abs16;
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::rtl::{Mult8x8, RomPtr, Unreachable, WriteReg};
extern "C" {
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;

}
pub use crate::types::{
    __int16_t, __int32_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32, int32_t,
    uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t, Func_V, LongPtr, PairU16,
    Point16U, SnesRegs, VoidP, VramWriteEntry,
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
pub use crate::types::{EnemyBase, EnemyData, EnemyProps, EnemySpawnData};
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
pub use crate::types::Consts_A8;
pub const addr_locret_A8F5E3: Consts_A8 = 62947;
pub const addr_kKiHunter_Ilist_EB10: Consts_A8 = 60176;
pub const addr_kKiHunter_Ilist_EAF2: Consts_A8 = 60146;
pub const addr_kKiHunter_Ilist_EADA: Consts_A8 = 60122;
pub const addr_kKiHunter_Ilist_EAC2: Consts_A8 = 60098;
pub const addr_kKiHunter_Ilist_EAA6: Consts_A8 = 60070;
pub const addr_kKiHunter_Ilist_EA8A: Consts_A8 = 60042;
pub const addr_kKiHunter_Ilist_EA7E: Consts_A8 = 60030;
pub const addr_kKiHunter_Ilist_EA5E: Consts_A8 = 59998;
pub const addr_kKiHunter_Ilist_EA4E: Consts_A8 = 59982;
pub const addr_kKiHunter_Ilist_EA32: Consts_A8 = 59954;
pub const addr_kKiHunter_Ilist_EA24: Consts_A8 = 59940;
pub const addr_kKiHunter_Ilist_EA08: Consts_A8 = 59912;
pub const addr_kKiHunter_Ilist_E9FA: Consts_A8 = 59898;
pub const addr_kBlueBrinstarFaceBlock_Ilist_E828: Consts_A8 = 59432;
pub const addr_kBlueBrinstarFaceBlock_Ilist_E81A: Consts_A8 = 59418;
pub const addr_kBlueBrinstarFaceBlock_Ilist_E80C: Consts_A8 = 59404;
pub const addr_kWreckedShipSpark_Ilist_E5E5: Consts_A8 = 58853;
pub const addr_kWreckedShipSpark_Ilist_E5A7: Consts_A8 = 58791;
pub const addr_kWalkingLavaSeahorse_Ilist_DCC1: Consts_A8 = 56513;
pub const addr_stru_A8DCBB: Consts_A8 = 56507;
pub const addr_kWalkingLavaSeahorse_Ilist_DC73: Consts_A8 = 56435;
pub const addr_stru_A8DC59: Consts_A8 = 56409;
pub const addr_kWalkingLavaSeahorse_Ilist_DC57: Consts_A8 = 56407;
pub const addr_kWalkingLavaSeahorse_Ilist_DC51: Consts_A8 = 56401;
pub const addr_stru_A8DC4B: Consts_A8 = 56395;
pub const addr_kWalkingLavaSeahorse_Ilist_DC03: Consts_A8 = 56323;
pub const addr_stru_A8DBE9: Consts_A8 = 56297;
pub const addr_kWalkingLavaSeahorse_Ilist_DBE7: Consts_A8 = 56295;
pub const addr_kMaridiaPuffer_Ilist_D855: Consts_A8 = 55381;
pub const addr_kMaridiaPuffer_Ilist_D841: Consts_A8 = 55361;
pub const addr_loc_A8D02E: Consts_A8 = 53294;
pub const addr_loc_A8CF26: Consts_A8 = 53030;
pub const addr_loc_A8CF18: Consts_A8 = 53016;
pub const addr_kWreckedShipRobot_Ilist_CB65: Consts_A8 = 52069;
pub const addr_kWreckedShipRobot_Ilist_CB1D: Consts_A8 = 51997;
pub const addr_kWreckedShipRobot_Ilist_CB09: Consts_A8 = 51977;
pub const addr_kWreckedShipRobot_Ilist_CAFD: Consts_A8 = 51965;
pub const addr_kWreckedShipRobot_Ilist_CA7D: Consts_A8 = 51837;
pub const addr_kWreckedShipRobot_Ilist_CA01: Consts_A8 = 51713;
pub const addr_kWreckedShipRobot_Ilist_C985: Consts_A8 = 51589;
pub const addr_kWreckedShipRobot_Ilist_C92D: Consts_A8 = 51501;
pub const addr_kWreckedShipRobot_Ilist_C91B: Consts_A8 = 51483;
pub const addr_kWreckedShipRobot_Ilist_C8D1: Consts_A8 = 51409;
pub const addr_kWreckedShipRobot_Ilist_C8BD: Consts_A8 = 51389;
pub const addr_kWreckedShipRobot_Ilist_C8B1: Consts_A8 = 51377;
pub const addr_kWreckedShipRobot_Ilist_C833: Consts_A8 = 51251;
pub const addr_kWreckedShipRobot_Ilist_C7BB: Consts_A8 = 51131;
pub const addr_kWreckedShipRobot_Ilist_C73F: Consts_A8 = 51007;
pub const addr_stru_A8C6E9: Consts_A8 = 50921;
pub const addr_kWreckedShipRobot_Ilist_C6E5: Consts_A8 = 50917;
pub const addr_stru_A8C199: Consts_A8 = 49561;
pub const addr_kMaridiaFloater_Ilist_C191: Consts_A8 = 49553;
pub const addr_kMaridiaFloater_Ilist_C183: Consts_A8 = 49539;
pub const addr_kMaridiaFloater_Ilist_C173: Consts_A8 = 49523;
pub const addr_kMaridiaFloater_Ilist_C163: Consts_A8 = 49507;
pub const addr_kBeetom_Ilist_B728: Consts_A8 = 46888;
pub const addr_kBeetom_Ilist_B708: Consts_A8 = 46856;
pub const addr_kBeetom_Ilist_B6F2: Consts_A8 = 46834;
pub const addr_kBeetom_Ilist_B6CC: Consts_A8 = 46796;
pub const addr_kBeetom_Ilist_B6AC: Consts_A8 = 46764;
pub const addr_kBeetom_Ilist_B696: Consts_A8 = 46742;
pub const addr_kNorfairLavaMan_Ilist_AE0C: Consts_A8 = 44556;
pub const addr_kNorfairLavaMan_Ilist_ADDC: Consts_A8 = 44508;
pub const addr_kNorfairLavaMan_Ilist_ADAC: Consts_A8 = 44460;
pub const addr_kNorfairLavaMan_Ilist_AD7E: Consts_A8 = 44414;
pub const addr_kNorfairLavaMan_Ilist_AD50: Consts_A8 = 44368;
pub const addr_kNorfairLavaMan_Ilist_AD3C: Consts_A8 = 44348;
pub const addr_kNorfairLavaMan_Ilist_AD0C: Consts_A8 = 44300;
pub const addr_kNorfairLavaMan_Ilist_ACDE: Consts_A8 = 44254;
pub const addr_kNorfairLavaMan_Ilist_ACB0: Consts_A8 = 44208;
pub const addr_kNorfairLavaMan_Ilist_AC9C: Consts_A8 = 44188;
pub const addr_kKago_Ilist_AB32: Consts_A8 = 43826;
pub const addr_kKago_Ilist_AB1E: Consts_A8 = 43806;
pub const addr_kYappingMaw_Ilist_A079: Consts_A8 = 41081;
pub const addr_off_A8A061: Consts_A8 = 41057;
pub const addr_kYappingMaw_Ilist_A05B: Consts_A8 = 41051;
pub const addr_kYappingMaw_Ilist_A03D: Consts_A8 = 41021;
pub const addr_off_A8A025: Consts_A8 = 40997;
pub const addr_kYappingMaw_Ilist_A01F: Consts_A8 = 40991;
pub const addr_kYappingMaw_Ilist_9FC7: Consts_A8 = 40903;
pub const addr_kYappingMaw_Ilist_9F6F: Consts_A8 = 40815;
pub const addr_loc_A89E90: Consts_A8 = 40592;
pub const addr_kWreckedShipGhost_Ilist_9A8C: Consts_A8 = 39564;
pub const addr_off_A896DF: Consts_A8 = 38623;
pub const addr_off_A896D7: Consts_A8 = 38615;
pub const addr_kMorphBallEye_Ilist_9026: Consts_A8 = 36902;
pub const addr_kMorphBallEye_Ilist_9014: Consts_A8 = 36884;
pub const addr_stru_A8900E: Consts_A8 = 36878;
pub const addr_kMorphBallEye_Ilist_9002: Consts_A8 = 36866;
pub const addr_stru_A88FFC: Consts_A8 = 36860;
pub const addr_kMorphBallEye_Ilist_8FF0: Consts_A8 = 36848;
pub const addr_kMiniDraygon_Ilist_8775: Consts_A8 = 34677;
pub const addr_kMiniDraygon_Ilist_876F: Consts_A8 = 34671;
pub const addr_kMiniDraygon_Ilist_8727: Consts_A8 = 34599;
pub const addr_kMiniDraygon_Ilist_870B: Consts_A8 = 34571;
pub const addr_kMiniDraygon_Ilist_86C3: Consts_A8 = 34499;
pub const addr_kMiniDraygon_Ilist_86A7: Consts_A8 = 34471;
pub const addr_kSpritemap_Nothing_A8: Consts_A8 = 32845;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Fune {
    pub base: EnemyBase,
    pub fune_var_A: uint16,
    pub fune_var_B: uint16,
    pub fune_var_C: uint16,
    pub fune_var_D: uint16,
    pub fune_var_E: uint16,
    pub fune_var_F: uint16,
    pub fune_parameter_1: uint16,
    pub fune_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub fune_var_00: uint16,
    pub fune_var_01: uint16,
    pub fune_var_02: uint16,
    pub fune_var_03: uint16,
    pub fune_var_04: uint16,
    pub fune_var_05: uint16,
    pub fune_var_06: uint16,
    pub fune_var_07: uint16,
    pub fune_var_08: uint16,
    pub fune_var_09: uint16,
    pub fune_var_0A: uint16,
    pub fune_var_0B: uint16,
    pub fune_var_0C: uint16,
    pub fune_var_0D: uint16,
    pub fune_var_0E: uint16,
    pub fune_var_0F: uint16,
    pub fune_var_10: uint16,
    pub fune_var_11: uint16,
    pub fune_var_12: uint16,
    pub fune_var_13: uint16,
    pub fune_var_14: uint16,
    pub fune_var_15: uint16,
    pub fune_var_16: uint16,
    pub fune_var_17: uint16,
    pub fune_var_18: uint16,
    pub fune_var_19: uint16,
    pub fune_var_1A: uint16,
    pub fune_var_1B: uint16,
    pub fune_var_1C: uint16,
    pub fune_var_1D: uint16,
    pub fune_var_1E: uint16,
    pub fune_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_KiHunter {
    pub base: EnemyBase,
    pub khr_var_A: uint16,
    pub khr_var_B: uint16,
    pub khr_var_C: uint16,
    pub khr_var_D: uint16,
    pub khr_var_E: uint16,
    pub khr_var_F: uint16,
    pub khr_parameter_1: uint16,
    pub khr_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub khr_var_00: uint16,
    pub khr_var_01: uint16,
    pub khr_var_02: uint16,
    pub khr_var_03: uint16,
    pub khr_var_04: uint16,
    pub khr_var_05: uint16,
    pub khr_var_06: uint16,
    pub khr_var_07: uint16,
    pub khr_var_08: uint16,
    pub khr_var_09: uint16,
    pub khr_var_0A: uint16,
    pub khr_var_0B: uint16,
    pub khr_var_0C: uint16,
    pub khr_var_0D: uint16,
    pub khr_var_0E: uint16,
    pub khr_var_0F: uint16,
    pub khr_var_10: uint16,
    pub khr_var_11: uint16,
    pub khr_var_12: uint16,
    pub khr_var_13: uint16,
    pub khr_var_14: uint16,
    pub khr_var_15: uint16,
    pub khr_var_16: uint16,
    pub khr_var_17: uint16,
    pub khr_var_18: uint16,
    pub khr_var_19: uint16,
    pub khr_var_1A: uint16,
    pub khr_var_1B: uint16,
    pub khr_var_1C: uint16,
    pub khr_var_1D: uint16,
    pub khr_var_1E: uint16,
    pub khr_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MiniDraygon {
    pub base: EnemyBase,
    pub mdn_var_A: uint16,
    pub mdn_var_B: uint16,
    pub mdn_var_C: uint16,
    pub mdn_var_D: uint16,
    pub mdn_var_E: uint16,
    pub mdn_var_F: uint16,
    pub mdn_parameter_1: uint16,
    pub mdn_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub mdn_var_00: uint16,
    pub mdn_var_01: uint16,
    pub mdn_var_02: uint16,
    pub mdn_var_03: uint16,
    pub mdn_var_04: uint16,
    pub mdn_var_05: uint16,
    pub mdn_var_06: uint16,
    pub mdn_var_07: uint16,
    pub mdn_var_08: uint16,
    pub mdn_var_09: uint16,
    pub mdn_var_0A: uint16,
    pub mdn_var_0B: uint16,
    pub mdn_var_0C: uint16,
    pub mdn_var_0D: uint16,
    pub mdn_var_0E: uint16,
    pub mdn_var_0F: uint16,
    pub mdn_var_10: uint16,
    pub mdn_var_11: uint16,
    pub mdn_var_12: uint16,
    pub mdn_var_13: uint16,
    pub mdn_var_14: uint16,
    pub mdn_var_15: uint16,
    pub mdn_var_16: uint16,
    pub mdn_var_17: uint16,
    pub mdn_var_18: uint16,
    pub mdn_var_19: uint16,
    pub mdn_var_1A: uint16,
    pub mdn_var_1B: uint16,
    pub mdn_var_1C: uint16,
    pub mdn_var_1D: uint16,
    pub mdn_var_1E: uint16,
    pub mdn_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_NorfairLavaMan {
    pub base: EnemyBase,
    pub nlmn_var_A: uint16,
    pub nlmn_var_B: uint16,
    pub nlmn_var_C: uint16,
    pub nlmn_var_D: uint16,
    pub nlmn_var_E: uint16,
    pub nlmn_var_F: uint16,
    pub nlmn_parameter_1: uint16,
    pub nlmn_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub nlmn_var_00: uint16,
    pub nlmn_var_01: uint16,
    pub nlmn_var_02: uint16,
    pub nlmn_var_03: uint16,
    pub nlmn_var_04: uint16,
    pub nlmn_var_05: uint16,
    pub nlmn_var_06: uint16,
    pub nlmn_var_07: uint16,
    pub nlmn_var_08: uint16,
    pub nlmn_var_09: uint16,
    pub nlmn_var_0A: uint16,
    pub nlmn_var_0B: uint16,
    pub nlmn_var_0C: uint16,
    pub nlmn_var_0D: uint16,
    pub nlmn_var_0E: uint16,
    pub nlmn_var_0F: uint16,
    pub nlmn_var_10: uint16,
    pub nlmn_var_11: uint16,
    pub nlmn_var_12: uint16,
    pub nlmn_var_13: uint16,
    pub nlmn_var_14: uint16,
    pub nlmn_var_15: uint16,
    pub nlmn_var_16: uint16,
    pub nlmn_var_17: uint16,
    pub nlmn_var_18: uint16,
    pub nlmn_var_19: uint16,
    pub nlmn_var_1A: uint16,
    pub nlmn_var_1B: uint16,
    pub nlmn_var_1C: uint16,
    pub nlmn_var_1D: uint16,
    pub nlmn_var_1E: uint16,
    pub nlmn_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_WalkingLavaSeahorse {
    pub base: EnemyBase,
    pub wlse_var_A: uint16,
    pub wlse_var_B: uint16,
    pub wlse_var_C: uint16,
    pub wlse_var_D: uint16,
    pub wlse_var_E: uint16,
    pub wlse_var_F: uint16,
    pub wlse_parameter_1: uint16,
    pub wlse_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub wlse_var_00: uint16,
    pub wlse_var_01: uint16,
    pub wlse_var_02: uint16,
    pub wlse_var_03: uint16,
    pub wlse_var_04: uint16,
    pub wlse_var_05: uint16,
    pub wlse_var_06: uint16,
    pub wlse_var_07: uint16,
    pub wlse_var_08: uint16,
    pub wlse_var_09: uint16,
    pub wlse_var_0A: uint16,
    pub wlse_var_0B: uint16,
    pub wlse_var_0C: uint16,
    pub wlse_var_0D: uint16,
    pub wlse_var_0E: uint16,
    pub wlse_var_0F: uint16,
    pub wlse_var_10: uint16,
    pub wlse_var_11: uint16,
    pub wlse_var_12: uint16,
    pub wlse_var_13: uint16,
    pub wlse_var_14: uint16,
    pub wlse_var_15: uint16,
    pub wlse_var_16: uint16,
    pub wlse_var_17: uint16,
    pub wlse_var_18: uint16,
    pub wlse_var_19: uint16,
    pub wlse_var_1A: uint16,
    pub wlse_var_1B: uint16,
    pub wlse_var_1C: uint16,
    pub wlse_var_1D: uint16,
    pub wlse_var_1E: uint16,
    pub wlse_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_WreckedShipGhost {
    pub base: EnemyBase,
    pub wsgt_var_A: uint16,
    pub wsgt_var_B: uint16,
    pub wsgt_var_C: uint16,
    pub wsgt_var_D: uint16,
    pub wsgt_var_E: uint16,
    pub wsgt_var_F: uint16,
    pub wsgt_parameter_1: uint16,
    pub wsgt_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub wsgt_var_00: uint16,
    pub wsgt_var_01: uint16,
    pub wsgt_var_02: uint16,
    pub wsgt_var_03: uint16,
    pub wsgt_var_04: uint16,
    pub wsgt_var_05: uint16,
    pub wsgt_var_06: uint16,
    pub wsgt_var_07: uint16,
    pub wsgt_var_08: uint16,
    pub wsgt_var_09: uint16,
    pub wsgt_var_0A: uint16,
    pub wsgt_var_0B: uint16,
    pub wsgt_var_0C: uint16,
    pub wsgt_var_0D: uint16,
    pub wsgt_var_0E: uint16,
    pub wsgt_var_0F: uint16,
    pub wsgt_var_10: uint16,
    pub wsgt_var_11: uint16,
    pub wsgt_var_12: uint16,
    pub wsgt_var_13: uint16,
    pub wsgt_var_14: uint16,
    pub wsgt_var_15: uint16,
    pub wsgt_var_16: uint16,
    pub wsgt_var_17: uint16,
    pub wsgt_var_18: uint16,
    pub wsgt_var_19: uint16,
    pub wsgt_var_1A: uint16,
    pub wsgt_var_1B: uint16,
    pub wsgt_var_1C: uint16,
    pub wsgt_var_1D: uint16,
    pub wsgt_var_1E: uint16,
    pub wsgt_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_WreckedShipRobot {
    pub base: EnemyBase,
    pub wsrt_var_A: uint16,
    pub wsrt_var_B: uint16,
    pub wsrt_var_C: uint16,
    pub wsrt_var_D: uint16,
    pub wsrt_var_E: uint16,
    pub wsrt_var_F: uint16,
    pub wsrt_parameter_1: uint16,
    pub wsrt_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_WreckedShipSpark {
    pub base: EnemyBase,
    pub wssk_var_A: uint16,
    pub wssk_var_B: uint16,
    pub wssk_var_C: uint16,
    pub wssk_var_D: uint16,
    pub wssk_var_E: uint16,
    pub wssk_var_F: uint16,
    pub wssk_parameter_1: uint16,
    pub wssk_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_YappingMaw {
    pub base: EnemyBase,
    pub ymw_var_A: uint16,
    pub ymw_var_B: uint16,
    pub ymw_var_C: uint16,
    pub ymw_var_D: uint16,
    pub ymw_var_E: uint16,
    pub ymw_var_F: uint16,
    pub ymw_parameter_1: uint16,
    pub ymw_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub ymw_var_00: uint16,
    pub ymw_var_01: uint16,
    pub ymw_var_02: uint16,
    pub ymw_var_03: uint16,
    pub ymw_var_04: uint16,
    pub ymw_var_05: uint16,
    pub ymw_var_06: uint16,
    pub ymw_var_07: uint16,
    pub ymw_var_08: uint16,
    pub ymw_var_09: uint16,
    pub ymw_var_0A: uint16,
    pub ymw_var_0B: uint16,
    pub ymw_var_0C: uint16,
    pub ymw_var_0D: uint16,
    pub ymw_var_0E: uint16,
    pub ymw_var_0F: uint16,
    pub ymw_var_10: uint16,
    pub ymw_var_11: uint16,
    pub ymw_var_12: uint16,
    pub ymw_var_13: uint16,
    pub ymw_var_14: uint16,
    pub ymw_var_15: uint16,
    pub ymw_var_16: uint16,
    pub ymw_var_17: uint16,
    pub ymw_var_18: uint16,
    pub ymw_var_19: uint16,
    pub ymw_var_1A: uint16,
    pub ymw_var_1B: uint16,
    pub ymw_var_1C: uint16,
    pub ymw_var_1D: uint16,
    pub ymw_var_1E: uint16,
    pub ymw_var_1F: uint16,
    pub pad2: [uint8; 1984],
    pub ymw_var_20: uint16,
    pub ymw_var_21: uint16,
    pub ymw_var_22: uint16,
    pub ymw_var_23: uint16,
    pub ymw_var_24: uint16,
    pub ymw_var_25: uint16,
    pub ymw_var_26: uint16,
    pub ymw_var_27: uint16,
    pub ymw_var_28: uint16,
    pub ymw_var_29: uint16,
    pub ymw_var_2A: uint16,
    pub ymw_var_2B: uint16,
    pub ymw_var_2C: uint16,
    pub ymw_var_2D: uint16,
    pub ymw_var_2E: uint16,
    pub ymw_var_2F: uint16,
    pub ymw_var_30: uint16,
    pub ymw_var_31: uint16,
    pub ymw_var_32: uint16,
    pub ymw_var_33: uint16,
    pub ymw_var_34: uint16,
    pub ymw_var_35: uint16,
    pub ymw_var_36: uint16,
    pub ymw_var_37: uint16,
    pub ymw_var_38: uint16,
    pub ymw_var_39: uint16,
    pub ymw_var_3A: uint16,
    pub ymw_var_3B: uint16,
    pub ymw_var_3C: uint16,
    pub ymw_var_3D: uint16,
    pub ymw_var_3E: uint16,
    pub ymw_var_3F: uint16,
    pub pad3: [uint8; 1984],
    pub ymw_var_40: uint16,
    pub ymw_var_41: uint16,
    pub ymw_var_42: uint16,
    pub ymw_var_43: uint16,
    pub ymw_var_44: uint16,
    pub ymw_var_45: uint16,
    pub ymw_var_46: uint16,
    pub ymw_var_47: uint16,
    pub ymw_var_48: uint16,
    pub ymw_var_49: uint16,
    pub ymw_var_4A: uint16,
    pub ymw_var_4B: uint16,
    pub ymw_var_4C: uint16,
    pub ymw_var_4D: uint16,
    pub ymw_var_4E: uint16,
    pub ymw_var_4F: uint16,
    pub ymw_var_50: uint16,
    pub ymw_var_51: uint16,
    pub ymw_var_52: uint16,
    pub ymw_var_53: uint16,
    pub ymw_var_54: uint16,
    pub ymw_var_55: uint16,
    pub ymw_var_56: uint16,
    pub ymw_var_57: uint16,
    pub ymw_var_58: uint16,
    pub ymw_var_59: uint16,
    pub ymw_var_5A: uint16,
    pub ymw_var_5B: uint16,
    pub ymw_var_5C: uint16,
    pub ymw_var_5D: uint16,
    pub ymw_var_5E: uint16,
    pub ymw_var_5F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Beetom {
    pub base: EnemyBase,
    pub beetom_var_A: uint16,
    pub beetom_var_B: uint16,
    pub beetom_var_C: uint16,
    pub beetom_var_D: uint16,
    pub beetom_var_E: uint16,
    pub beetom_var_F: uint16,
    pub beetom_parameter_1: uint16,
    pub beetom_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub beetom_var_00: uint16,
    pub beetom_var_01: uint16,
    pub beetom_var_02: uint16,
    pub beetom_var_03: uint16,
    pub beetom_var_04: uint16,
    pub beetom_var_05: uint16,
    pub beetom_var_06: uint16,
    pub beetom_var_07: uint16,
    pub beetom_var_08: uint16,
    pub beetom_var_09: uint16,
    pub beetom_var_0A: uint16,
    pub beetom_var_0B: uint16,
    pub beetom_var_0C: uint16,
    pub beetom_var_0D: uint16,
    pub beetom_var_0E: uint16,
    pub beetom_var_0F: uint16,
    pub beetom_var_10: uint16,
    pub beetom_var_11: uint16,
    pub beetom_var_12: uint16,
    pub beetom_var_13: uint16,
    pub beetom_var_14: uint16,
    pub beetom_var_15: uint16,
    pub beetom_var_16: uint16,
    pub beetom_var_17: uint16,
    pub beetom_var_18: uint16,
    pub beetom_var_19: uint16,
    pub beetom_var_1A: uint16,
    pub beetom_var_1B: uint16,
    pub beetom_var_1C: uint16,
    pub beetom_var_1D: uint16,
    pub beetom_var_1E: uint16,
    pub beetom_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Kago {
    pub base: EnemyBase,
    pub kago_var_A: uint16,
    pub kago_var_B: uint16,
    pub kago_var_C: uint16,
    pub kago_var_D: uint16,
    pub kago_var_E: uint16,
    pub kago_var_F: uint16,
    pub kago_parameter_1: uint16,
    pub kago_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub kago_var_00: uint16,
    pub kago_var_01: uint16,
    pub kago_var_02: uint16,
    pub kago_var_03: uint16,
    pub kago_var_04: uint16,
    pub kago_var_05: uint16,
    pub kago_var_06: uint16,
    pub kago_var_07: uint16,
    pub kago_var_08: uint16,
    pub kago_var_09: uint16,
    pub kago_var_0A: uint16,
    pub kago_var_0B: uint16,
    pub kago_var_0C: uint16,
    pub kago_var_0D: uint16,
    pub kago_var_0E: uint16,
    pub kago_var_0F: uint16,
    pub kago_var_10: uint16,
    pub kago_var_11: uint16,
    pub kago_var_12: uint16,
    pub kago_var_13: uint16,
    pub kago_var_14: uint16,
    pub kago_var_15: uint16,
    pub kago_var_16: uint16,
    pub kago_var_17: uint16,
    pub kago_var_18: uint16,
    pub kago_var_19: uint16,
    pub kago_var_1A: uint16,
    pub kago_var_1B: uint16,
    pub kago_var_1C: uint16,
    pub kago_var_1D: uint16,
    pub kago_var_1E: uint16,
    pub kago_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MaridiaFloater {
    pub base: EnemyBase,
    pub mfr_var_A: uint16,
    pub mfr_var_B: uint16,
    pub mfr_var_C: uint16,
    pub mfr_var_D: uint16,
    pub mfr_var_E: uint16,
    pub mfr_var_F: uint16,
    pub mfr_parameter_1: uint16,
    pub mfr_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MaridiaPuffer {
    pub base: EnemyBase,
    pub mpr_var_A: uint16,
    pub mpr_var_B: uint16,
    pub mpr_var_C: uint16,
    pub mpr_var_D: uint16,
    pub mpr_var_E: uint16,
    pub mpr_var_F: uint16,
    pub mpr_parameter_1: uint16,
    pub mpr_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub mpr_var_00: uint16,
    pub mpr_var_01: uint16,
    pub mpr_var_02: uint16,
    pub mpr_var_03: uint16,
    pub mpr_var_04: uint16,
    pub mpr_var_05: uint16,
    pub mpr_var_06: uint16,
    pub mpr_var_07: uint16,
    pub mpr_var_08: uint16,
    pub mpr_var_09: uint16,
    pub mpr_var_0A: uint16,
    pub mpr_var_0B: uint16,
    pub mpr_var_0C: uint16,
    pub mpr_var_0D: uint16,
    pub mpr_var_0E: uint16,
    pub mpr_var_0F: uint16,
    pub mpr_var_10: uint16,
    pub mpr_var_11: uint16,
    pub mpr_var_12: uint16,
    pub mpr_var_13: uint16,
    pub mpr_var_14: uint16,
    pub mpr_var_15: uint16,
    pub mpr_var_16: uint16,
    pub mpr_var_17: uint16,
    pub mpr_var_18: uint16,
    pub mpr_var_19: uint16,
    pub mpr_var_1A: uint16,
    pub mpr_var_1B: uint16,
    pub mpr_var_1C: uint16,
    pub mpr_var_1D: uint16,
    pub mpr_var_1E: uint16,
    pub mpr_var_1F: uint16,
    pub pad3: [uint8; 4032],
    pub mpr_var_40: uint16,
    pub mpr_var_41: uint16,
    pub mpr_var_42: uint16,
    pub mpr_var_43: uint16,
    pub mpr_var_44: uint16,
    pub mpr_var_45: uint16,
    pub mpr_var_46: uint16,
    pub mpr_var_47: uint16,
    pub mpr_var_48: uint16,
    pub mpr_var_49: uint16,
    pub mpr_var_4A: uint16,
    pub mpr_var_4B: uint16,
    pub mpr_var_4C: uint16,
    pub mpr_var_4D: uint16,
    pub mpr_var_4E: uint16,
    pub mpr_var_4F: uint16,
    pub mpr_var_50: uint16,
    pub mpr_var_51: uint16,
    pub mpr_var_52: uint16,
    pub mpr_var_53: uint16,
    pub mpr_var_54: uint16,
    pub mpr_var_55: uint16,
    pub mpr_var_56: uint16,
    pub mpr_var_57: uint16,
    pub mpr_var_58: uint16,
    pub mpr_var_59: uint16,
    pub mpr_var_5A: uint16,
    pub mpr_var_5B: uint16,
    pub mpr_var_5C: uint16,
    pub mpr_var_5D: uint16,
    pub mpr_var_5E: uint16,
    pub mpr_var_5F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MorphBallEye {
    pub base: EnemyBase,
    pub mbee_var_A: uint16,
    pub mbee_var_B: uint16,
    pub mbee_var_C: uint16,
    pub mbee_var_D: uint16,
    pub mbee_var_E: uint16,
    pub mbee_var_F: uint16,
    pub mbee_parameter_1: uint16,
    pub mbee_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_WreckedShipOrbs {
    pub base: EnemyBase,
    pub wsos_var_A: uint16,
    pub wsos_var_B: uint16,
    pub wsos_var_C: uint16,
    pub wsos_var_D: uint16,
    pub wsos_var_E: uint16,
    pub wsos_var_F: uint16,
    pub wsos_parameter_1: uint16,
    pub wsos_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub wsos_var_00: uint16,
    pub wsos_var_01: uint16,
    pub wsos_var_02: uint16,
    pub wsos_var_03: uint16,
    pub wsos_var_04: uint16,
    pub wsos_var_05: uint16,
    pub wsos_var_06: uint16,
    pub wsos_var_07: uint16,
    pub wsos_var_08: uint16,
    pub wsos_var_09: uint16,
    pub wsos_var_0A: uint16,
    pub wsos_var_0B: uint16,
    pub wsos_var_0C: uint16,
    pub wsos_var_0D: uint16,
    pub wsos_var_0E: uint16,
    pub wsos_var_0F: uint16,
    pub wsos_var_10: uint16,
    pub wsos_var_11: uint16,
    pub wsos_var_12: uint16,
    pub wsos_var_13: uint16,
    pub wsos_var_14: uint16,
    pub wsos_var_15: uint16,
    pub wsos_var_16: uint16,
    pub wsos_var_17: uint16,
    pub wsos_var_18: uint16,
    pub wsos_var_19: uint16,
    pub wsos_var_1A: uint16,
    pub wsos_var_1B: uint16,
    pub wsos_var_1C: uint16,
    pub wsos_var_1D: uint16,
    pub wsos_var_1E: uint16,
    pub wsos_var_1F: uint16,
}
#[inline]
unsafe fn RomPtr_A8(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0xa80000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn gVramWriteEntry(mut a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    return (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry;
}
pub const fnnullsub_152: ::core::ffi::c_int = 0x90e90e;
pub const fnSamus_InputHandler_E913: ::core::ffi::c_int = 0x90e913;
pub const fnSamus_InputHandler_E91D: ::core::ffi::c_int = 0x90e91d;
pub const fnnullsub_170_A8: ::core::ffi::c_int = 0xa8804c;
pub const fnMiniDraygon_Func_4: ::core::ffi::c_int = 0xa88922;
pub const fnMiniDraygon_Func_8: ::core::ffi::c_int = 0xa88a34;
pub const fnMiniDraygon_Func_9: ::core::ffi::c_int = 0xa88a3b;
pub const fnMiniDraygon_Func_10: ::core::ffi::c_int = 0xa88a78;
pub const fnMorphBallEye_Func_1: ::core::ffi::c_int = 0xa890f1;
pub const fnMorphBallEye_Func_2: ::core::ffi::c_int = 0xa8912e;
pub const fnMorphBallEye_Func_3: ::core::ffi::c_int = 0xa89160;
pub const fnMorphBallEye_Func_4: ::core::ffi::c_int = 0xa891ce;
pub const fnnullsub_244: ::core::ffi::c_int = 0xa891dc;
pub const fnFune_Func_1: ::core::ffi::c_int = 0xa89737;
pub const fnFune_Func_2: ::core::ffi::c_int = 0xa8975c;
pub const fnnullsub_247: ::core::ffi::c_int = 0xa8978e;
pub const fnnullsub_248: ::core::ffi::c_int = 0xa8978f;
pub const fnWreckedShipGhost_Func_1: ::core::ffi::c_int = 0xa89b42;
pub const fnWreckedShipGhost_Func_2: ::core::ffi::c_int = 0xa89bad;
pub const fnWreckedShipGhost_Func_4: ::core::ffi::c_int = 0xa89c69;
pub const fnWreckedShipGhost_Func_5: ::core::ffi::c_int = 0xa89c8a;
pub const fnWreckedShipGhost_Func_6: ::core::ffi::c_int = 0xa89d13;
pub const fnWreckedShipGhost_Func_7: ::core::ffi::c_int = 0xa89d36;
pub const fnYappingMaw_Func_1: ::core::ffi::c_int = 0xa8a235;
pub const fnYappingMaw_Func_2: ::core::ffi::c_int = 0xa8a28c;
pub const fnYappingMaw_Func_8: ::core::ffi::c_int = 0xa8a445;
pub const fnYappingMaw_Func_11: ::core::ffi::c_int = 0xa8a68a;
pub const fnKago_Func_1: ::core::ffi::c_int = 0xa8ab7b;
pub const fnnullsub_306: ::core::ffi::c_int = 0xa8ab81;
pub const fnNorfairLavaMan_Func_6: ::core::ffi::c_int = 0xa8b0b2;
pub const fnNorfairLavaMan_Func_7: ::core::ffi::c_int = 0xa8b11a;
pub const fnNorfairLavaMan_Func_8: ::core::ffi::c_int = 0xa8b175;
pub const fnsub_A8B193: ::core::ffi::c_int = 0xa8b193;
pub const fnNorfairLavaMan_Func_9: ::core::ffi::c_int = 0xa8b1b8;
pub const fnNorfairLavaMan_Func_10: ::core::ffi::c_int = 0xa8b1dd;
pub const fnNorfairLavaMan_Func_11: ::core::ffi::c_int = 0xa8b204;
pub const fnsub_A8B291: ::core::ffi::c_int = 0xa8b291;
pub const fnNorfairLavaMan_Func_13: ::core::ffi::c_int = 0xa8b295;
pub const fnNorfairLavaMan_Func_15: ::core::ffi::c_int = 0xa8b30d;
pub const fnNorfairLavaMan_Func_16: ::core::ffi::c_int = 0xa8b31f;
pub const fnNorfairLavaMan_Func_17: ::core::ffi::c_int = 0xa8b356;
pub const fnNorfairLavaMan_Func_18: ::core::ffi::c_int = 0xa8b3a7;
pub const fnBeetom_Func_3: ::core::ffi::c_int = 0xa8b814;
pub const fnBeetom_Func_4: ::core::ffi::c_int = 0xa8b82f;
pub const fnBeetom_Func_5: uint32 = 11057231;
pub const fnBeetom_Func_6: ::core::ffi::c_int = 0xa8b85f;
pub const fnBeetom_Func_7: ::core::ffi::c_int = 0xa8b873;
pub const fnBeetom_Func_8: uint32 = 11057287;
pub const fnBeetom_Func_9: uint32 = 11057321;
pub const fnBeetom_Func_10: uint32 = 11057355;
pub const fnBeetom_Func_11: uint32 = 11057389;
pub const fnBeetom_Func_12: ::core::ffi::c_int = 0xa8b90f;
pub const fnBeetom_Func_13: ::core::ffi::c_int = 0xa8b952;
pub const fnBeetom_Func_14: ::core::ffi::c_int = 0xa8b966;
pub const fnBeetom_Func_15: ::core::ffi::c_int = 0xa8b97a;
pub const fnBeetom_Func_16: ::core::ffi::c_int = 0xa8b9a2;
pub const fnBeetom_Func_17: ::core::ffi::c_int = 0xa8b9b2;
pub const fnBeetom_Func_18: ::core::ffi::c_int = 0xa8b9c1;
pub const fnBeetom_Func_19: ::core::ffi::c_int = 0xa8ba24;
pub const fnBeetom_Func_20: ::core::ffi::c_int = 0xa8ba84;
pub const fnBeetom_Func_21: ::core::ffi::c_int = 0xa8bab7;
pub const fnBeetom_Func_24: ::core::ffi::c_int = 0xa8bb55;
pub const fnBeetom_Func_25: ::core::ffi::c_int = 0xa8bb88;
pub const fnBeetom_Func_28: ::core::ffi::c_int = 0xa8bc26;
pub const fnBeetom_Func_29: ::core::ffi::c_int = 0xa8bc5a;
pub const fnBeetom_Func_32: ::core::ffi::c_int = 0xa8bcf8;
pub const fnBeetom_Func_33: ::core::ffi::c_int = 0xa8bd42;
pub const fnBeetom_Func_35: ::core::ffi::c_int = 0xa8bd9d;
pub const fnBeetom_Func_36: ::core::ffi::c_int = 0xa8bdc5;
pub const fnMaridiaFloater_Func_3: ::core::ffi::c_int = 0xa8c283;
pub const fnMaridiaFloater_Func_4: ::core::ffi::c_int = 0xa8c2a6;
pub const fnMaridiaFloater_Func_5: ::core::ffi::c_int = 0xa8c2cf;
pub const fnMaridiaFloater_Func_6: ::core::ffi::c_int = 0xa8c36b;
pub const fnMaridiaFloater_Func_7: ::core::ffi::c_int = 0xa8c3e1;
pub const fnMaridiaFloater_Func_8: ::core::ffi::c_int = 0xa8c469;
pub const fnMaridiaFloater_Func_9: ::core::ffi::c_int = 0xa8c4dc;
pub const fnMaridiaFloater_Func_10: ::core::ffi::c_int = 0xa8c500;
pub const fnMaridiaFloater_Func_11: ::core::ffi::c_int = 0xa8c51d;
pub const fnnullsub_256: ::core::ffi::c_int = 0xa8c568;
pub const fnMaridiaFloater_Func_12: ::core::ffi::c_int = 0xa8c569;
pub const fnMaridiaFloater_Func_13: ::core::ffi::c_int = 0xa8c59f;
pub const fnWreckedShipRobot_Func_1: ::core::ffi::c_int = 0xa8cc67;
pub const fnMaridiaPuffer_Func_1: ::core::ffi::c_int = 0xa8d92b;
pub const fnMaridiaPuffer_Func_2: ::core::ffi::c_int = 0xa8d940;
pub const fnMaridiaPuffer_Func_3: ::core::ffi::c_int = 0xa8d963;
pub const fnMaridiaPuffer_Func_4: ::core::ffi::c_int = 0xa8d97c;
pub const fnWalkingLavaSeahorse_Func_3: ::core::ffi::c_int = 0xa8dd71;
pub const fnWalkingLavaSeahorse_Func_4: ::core::ffi::c_int = 0xa8ddc6;
pub const fnWalkingLavaSeahorse_Func_6: ::core::ffi::c_int = 0xa8de05;
pub const fnWalkingLavaSeahorse_Func_7: ::core::ffi::c_int = 0xa8de4b;
pub const fnnullsub_257: ::core::ffi::c_int = 0xa8decc;
pub const fnWalkingLavaSeahorse_Func_8: ::core::ffi::c_int = 0xa8decd;
pub const fnWalkingLavaSeahorse_Func_9: ::core::ffi::c_int = 0xa8deec;
pub const fnWreckedShipOrbs_Func_3: ::core::ffi::c_int = 0xa8e405;
pub const fnWreckedShipOrbs_Func_4: ::core::ffi::c_int = 0xa8e424;
pub const fnWreckedShipOrbs_Func_5: ::core::ffi::c_int = 0xa8e443;
pub const fnWreckedShipOrbs_Func_6: ::core::ffi::c_int = 0xa8e462;
pub const fnnullsub_259: uint32 = 11069076;
pub const fnWreckedShipSpark_Func_1: ::core::ffi::c_int = 0xa8e695;
pub const fnWreckedShipSpark_Func_2: ::core::ffi::c_int = 0xa8e6b7;
pub const fnWreckedShipSpark_Func_3: uint32 = 11069148;
pub const fnBlueBrinstarFaceBlock_Func_1: ::core::ffi::c_int = 0xa8e86e;
pub const fnKiHunter_Func_1: ::core::ffi::c_int = 0xa8f268;
pub const fnKiHunter_Func_2: ::core::ffi::c_int = 0xa8f3b8;
pub const fnKiHunter_Func_3: ::core::ffi::c_int = 0xa8f4ed;
pub const fnKiHunter_Func_4: ::core::ffi::c_int = 0xa8f55a;
pub const fnKiHunter_Func_5: ::core::ffi::c_int = 0xa8f58b;
pub const fnnullsub_346: uint32 = 11072995;
pub const fnKiHunter_Func_6: ::core::ffi::c_int = 0xa8f5f0;
pub const fnKiHunter_Func_7: ::core::ffi::c_int = 0xa8f68b;
pub const fnKiHunter_Func_8: ::core::ffi::c_int = 0xa8f6b3;
pub const fnKiHunter_Func_9: ::core::ffi::c_int = 0xa8f6f3;
pub const fnKiHunter_Func_10: ::core::ffi::c_int = 0xa8f7cf;
pub const fnKiHunter_Func_11: ::core::ffi::c_int = 0xa8f7db;
pub const fnKiHunter_Func_14: ::core::ffi::c_int = 0xa8f8ad;
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
unsafe fn Get_Beetom(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Beetom {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Beetom;
}
#[inline]
unsafe fn Get_Fune(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Fune {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Fune;
}
#[inline]
unsafe fn Get_Kago(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Kago {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Kago;
}
#[inline]
unsafe fn Get_KiHunter(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_KiHunter {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_KiHunter;
}
#[inline]
unsafe fn Get_MaridiaFloater(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MaridiaFloater {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MaridiaFloater;
}
#[inline]
unsafe fn Get_MaridiaPuffer(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MaridiaPuffer {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MaridiaPuffer;
}
#[inline]
unsafe fn Get_MiniDraygon(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_MiniDraygon {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MiniDraygon;
}
#[inline]
unsafe fn Get_MorphBallEye(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_MorphBallEye {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MorphBallEye;
}
#[inline]
unsafe fn Get_NorfairLavaMan(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_NorfairLavaMan {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_NorfairLavaMan;
}
#[inline]
unsafe fn Get_WalkingLavaSeahorse(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_WalkingLavaSeahorse {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_WalkingLavaSeahorse;
}
#[inline]
unsafe fn Get_WreckedShipGhost(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_WreckedShipGhost {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_WreckedShipGhost;
}
#[inline]
unsafe fn Get_WreckedShipOrbs(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_WreckedShipOrbs {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_WreckedShipOrbs;
}
#[inline]
unsafe fn Get_WreckedShipRobot(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_WreckedShipRobot {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_WreckedShipRobot;
}
#[inline]
unsafe fn Get_WreckedShipSpark(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_WreckedShipSpark {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_WreckedShipSpark;
}
#[inline]
unsafe fn Get_YappingMaw(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_YappingMaw {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_YappingMaw;
}
static g_word_A890CA: [int16; 4] = [
    -(8 as ::core::ffi::c_int) as int16,
    8 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
];
static g_word_A890D2: [int16; 4] = [
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(8 as ::core::ffi::c_int) as int16,
    8 as ::core::ffi::c_int as int16,
];
static g_word_A89050: uint16 = 0x80 as uint16;
static g_word_A89052: uint16 = 0xb0 as uint16;
static g_word_A89054: uint16 = 0x80 as uint16;
static g_word_A89056: uint16 = 0x80 as uint16;
static g_word_A89A9C: uint16 = 0x10 as uint16;
static g_word_A89A9E: uint16 = 0x40 as uint16;
static g_word_A89AA0: uint16 = 0x1800 as uint16;
static g_word_A89AA2: uint16 = 1 as uint16;
static g_word_A89AA4: uint16 = 0x78 as uint16;
static g_word_A89AA6: uint16 = 0x78 as uint16;
static g_word_A89AA8: [uint16; 18] = [
    0xffc0 as ::core::ffi::c_int as uint16,
    0xffc0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0xffc0 as ::core::ffi::c_int as uint16,
    0x40 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0xffc0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x40 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0xffc0 as ::core::ffi::c_int as uint16,
    0x40 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x40 as ::core::ffi::c_int as uint16,
    0x40 as ::core::ffi::c_int as uint16,
    0x40 as ::core::ffi::c_int as uint16,
];
static g_word_A89ACC: [uint16; 17] = [
    1 as ::core::ffi::c_int as uint16,
    8 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    8 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    7 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    7 as ::core::ffi::c_int as uint16,
    2 as ::core::ffi::c_int as uint16,
    6 as ::core::ffi::c_int as uint16,
    2 as ::core::ffi::c_int as uint16,
    6 as ::core::ffi::c_int as uint16,
    3 as ::core::ffi::c_int as uint16,
    5 as ::core::ffi::c_int as uint16,
    3 as ::core::ffi::c_int as uint16,
    5 as ::core::ffi::c_int as uint16,
    0xffff as ::core::ffi::c_int as uint16,
];
static kWreckedShipGhost_Palette: [uint16; 16] = [
    0x3800 as ::core::ffi::c_int as uint16,
    0x57ff as ::core::ffi::c_int as uint16,
    0x42f7 as ::core::ffi::c_int as uint16,
    0x929 as ::core::ffi::c_int as uint16,
    0xa5 as ::core::ffi::c_int as uint16,
    0x4f5a as ::core::ffi::c_int as uint16,
    0x36b5 as ::core::ffi::c_int as uint16,
    0x2610 as ::core::ffi::c_int as uint16,
    0x1dce as ::core::ffi::c_int as uint16,
    0x1df as ::core::ffi::c_int as uint16,
    0x1f as ::core::ffi::c_int as uint16,
    0x18 as ::core::ffi::c_int as uint16,
    0xa as ::core::ffi::c_int as uint16,
    0x6b9 as ::core::ffi::c_int as uint16,
    0xea as ::core::ffi::c_int as uint16,
    0x45 as ::core::ffi::c_int as uint16,
];
static g_word_A89D32: uint16 = 1 as uint16;
static g_word_A89D34: uint16 = 1 as uint16;
static g_word_A8C277: [int16; 3] = [
    -(12 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(20 as ::core::ffi::c_int) as int16,
];
static g_word_A8C27D: [int16; 3] = [
    -(20 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(12 as ::core::ffi::c_int) as int16,
];
static g_word_A8C19F: uint16 = 0x40 as uint16;
static g_word_A8C1A1: [uint16; 12] = [
    0 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    2 as ::core::ffi::c_int as uint16,
    3 as ::core::ffi::c_int as uint16,
    2 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0xffff as ::core::ffi::c_int as uint16,
    0xfffe as ::core::ffi::c_int as uint16,
    0xfffd as ::core::ffi::c_int as uint16,
    0xfffe as ::core::ffi::c_int as uint16,
    0xffff as ::core::ffi::c_int as uint16,
];
static g_word_A8C1B9: uint16 = 0 as uint16;
static g_word_A8C1BB: uint16 = 0 as uint16;
static g_word_A8C1BD: uint16 = 0 as uint16;
static g_word_A8C1BF: uint16 = 0 as uint16;
static g_word_A8C1C1: uint16 = 1 as uint16;
static g_word_A8C1C3: uint16 = 0 as uint16;
static g_word_A8C1C5: uint16 = 0xffff as uint16;
static g_word_A8C1C7: uint16 = 0x8000 as uint16;
static g_word_A8CCC1: [int16; 31] = [
    0x1f as ::core::ffi::c_int as int16,
    0x18 as ::core::ffi::c_int as int16,
    0xf as ::core::ffi::c_int as int16,
    8 as ::core::ffi::c_int as int16,
    0x40 as ::core::ffi::c_int as int16,
    0x18 as ::core::ffi::c_int as int16,
    0xf as ::core::ffi::c_int as int16,
    8 as ::core::ffi::c_int as int16,
    0x1f as ::core::ffi::c_int as int16,
    0x10 as ::core::ffi::c_int as int16,
    0xf as ::core::ffi::c_int as int16,
    8 as ::core::ffi::c_int as int16,
    0x1f as ::core::ffi::c_int as int16,
    0x18 as ::core::ffi::c_int as int16,
    0x10 as ::core::ffi::c_int as int16,
    8 as ::core::ffi::c_int as int16,
    0x1f as ::core::ffi::c_int as int16,
    0x18 as ::core::ffi::c_int as int16,
    0xf as ::core::ffi::c_int as int16,
    0x40 as ::core::ffi::c_int as int16,
    0xf as ::core::ffi::c_int as int16,
    8 as ::core::ffi::c_int as int16,
    0x1f as ::core::ffi::c_int as int16,
    0x18 as ::core::ffi::c_int as int16,
    0x10 as ::core::ffi::c_int as int16,
    0x18 as ::core::ffi::c_int as int16,
    0xf as ::core::ffi::c_int as int16,
    8 as ::core::ffi::c_int as int16,
    0x1f as ::core::ffi::c_int as int16,
    0x10 as ::core::ffi::c_int as int16,
    -(1 as ::core::ffi::c_int) as int16,
];
static g_word_A8D871: [int16; 10] = [
    0xc0 as ::core::ffi::c_int as int16,
    0xe0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    0x20 as ::core::ffi::c_int as int16,
    0x40 as ::core::ffi::c_int as int16,
    0x40 as ::core::ffi::c_int as int16,
    0x60 as ::core::ffi::c_int as int16,
    0x80 as ::core::ffi::c_int as int16,
    0xa0 as ::core::ffi::c_int as int16,
    0xc0 as ::core::ffi::c_int as int16,
];
static g_word_A8D885: [int16; 8] = [
    0x3ff as ::core::ffi::c_int as int16,
    0x4ff as ::core::ffi::c_int as int16,
    0x5ff as ::core::ffi::c_int as int16,
    0x6ff as ::core::ffi::c_int as int16,
    0x7ff as ::core::ffi::c_int as int16,
    0x8ff as ::core::ffi::c_int as int16,
    0x9ff as ::core::ffi::c_int as int16,
    0xaff as ::core::ffi::c_int as int16,
];
static g_word_A8D895: [int16; 26] = [
    3 as ::core::ffi::c_int as int16,
    1 as ::core::ffi::c_int as int16,
    4 as ::core::ffi::c_int as int16,
    1 as ::core::ffi::c_int as int16,
    5 as ::core::ffi::c_int as int16,
    2 as ::core::ffi::c_int as int16,
    6 as ::core::ffi::c_int as int16,
    2 as ::core::ffi::c_int as int16,
    7 as ::core::ffi::c_int as int16,
    2 as ::core::ffi::c_int as int16,
    8 as ::core::ffi::c_int as int16,
    3 as ::core::ffi::c_int as int16,
    9 as ::core::ffi::c_int as int16,
    3 as ::core::ffi::c_int as int16,
    0xa as ::core::ffi::c_int as int16,
    4 as ::core::ffi::c_int as int16,
    0xb as ::core::ffi::c_int as int16,
    4 as ::core::ffi::c_int as int16,
    0xc as ::core::ffi::c_int as int16,
    5 as ::core::ffi::c_int as int16,
    0xd as ::core::ffi::c_int as int16,
    5 as ::core::ffi::c_int as int16,
    0xe as ::core::ffi::c_int as int16,
    6 as ::core::ffi::c_int as int16,
    0xf as ::core::ffi::c_int as int16,
    6 as ::core::ffi::c_int as int16,
];
static g_word_A8DCC7: uint16 = 0x50 as uint16;
static g_word_A8DCCB: uint16 = 0x70 as uint16;
static g_word_A8F180: uint16 = 0x60 as uint16;
static g_word_A8F182: uint16 = 0xe000 as uint16;
static g_word_A8F184: uint16 = 0 as uint16;
static g_byte_A8F186: uint8 = 0x30 as uint8;
pub unsafe fn Enemy_GrappleReact_NoInteract_A8(game: &mut Game) {
    SwitchEnemyAiToMainAi(game);
}
pub unsafe fn Enemy_GrappleReact_KillEnemy_A8(game: &mut Game) {
    EnemyGrappleDeath(game);
}
pub unsafe fn Enemy_GrappleReact_CancelBeam_A8(game: &mut Game) {
    Enemy_SwitchToFrozenAi(game);
}
pub unsafe fn Enemy_GrappleReact_SamusLatchesNoInvinc_A8(game: &mut Game) {
    SamusLatchesOnWithGrappleNoInvinc(game);
}
pub unsafe fn Enemy_GrappleReact_HurtSamus_A8(game: &mut Game) {
    SamusHurtFromGrapple(game);
}
pub unsafe fn Enemy_NormalTouchAI_A8(game: &mut Game) {
    NormalEnemyTouchAi(game);
}
pub unsafe fn Enemy_NormalShotAI_A8(game: &mut Game) {
    NormalEnemyShotAi(game);
}
pub unsafe fn Enemy_NormalPowerBombAI_A8(game: &mut Game) {
    NormalEnemyPowerBombAi(game);
}
pub unsafe fn Enemy_NormalFrozenAI_A8(game: &mut Game) {
    NormalEnemyFrozenAI(game);
}
pub unsafe fn MiniDraygon_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x5e as uint16, game);
    return jp;
}
pub unsafe fn MiniDraygon_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if (*gEnemySpawnData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    )
    .offset(30))
    .cause_of_death as ::core::ffi::c_int
        == addr_kMiniDraygon_Ilist_870B as ::core::ffi::c_int
    {
        (*Get_MiniDraygon(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .mdn_var_F = -(8 as ::core::ffi::c_int) as uint16;
    } else {
        (*Get_MiniDraygon(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .mdn_var_F = 8 as uint16;
    }
    return jp;
}
pub unsafe fn MiniDraygon_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MiniDraygon = Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*gEnemySpawnData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    )
    .offset(30))
    .cause_of_death as ::core::ffi::c_int
        == addr_kMiniDraygon_Ilist_870B as ::core::ffi::c_int
    {
        (*E).mdn_var_F = (*E).mdn_var_F.wrapping_add(1);
    } else {
        (*E).mdn_var_F = (*E).mdn_var_F.wrapping_sub(1);
    }
    return jp;
}
pub unsafe fn MiniDraygon_Instr_4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MiniDraygon = Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mdn_var_0C = 0 as uint16;
    (*E).mdn_var_0B = 0 as uint16;
    (*E).mdn_var_C = (0xa88a34 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn MiniDraygon_Init(game: &mut Game) {
    let mut E: *mut Enemy_MiniDraygon = Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).mdn_parameter_1 != 0 {
        MiniDraygon_Func_2(game);
        (*E).base.layer = 4 as uint16;
    } else {
        MiniDraygon_Func_1(game);
        let mut v2: uint16 = (8 as ::core::ffi::c_int
            * *(&raw mut (*E).mdn_parameter_2 as *mut uint8).offset(0) as ::core::ffi::c_int)
            as uint16;
        let mut v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        (*E).mdn_var_04 = *(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
        (*E).mdn_var_03 = *(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v3 + 1 as ::core::ffi::c_int) as isize);
        (*E).mdn_var_06 = *(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v3 + 2 as ::core::ffi::c_int) as isize);
        (*E).mdn_var_05 = *(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v3 + 3 as ::core::ffi::c_int) as isize);
        (*E).mdn_var_E = (*(&raw mut (*(Get_MiniDraygon
            as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut Enemy_MiniDraygon)(
            v2 as ::core::ffi::c_int,
            game,
        ))
        .mdn_parameter_2 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16;
    }
    (*E).mdn_var_00 = 0 as uint16;
    (*E).mdn_var_01 = 0 as uint16;
    (*E).mdn_var_C = (0xa88922 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MiniDraygon_Func_1(game: &mut Game) {
    (*Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .mdn_var_B = (GetSamusEnemyDelta_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16;
    let mut E: *mut Enemy_MiniDraygon = Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_86A7 as ::core::ffi::c_int as uint16;
    if (*E).mdn_var_B != 0 {
        (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_870B as ::core::ffi::c_int as uint16;
    }
    MiniDraygon_Func_12(game);
}
pub unsafe fn MiniDraygon_Func_2(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xe54)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
        .offset((v0 + 1 as ::core::ffi::c_int) as isize);
    let mut E: *mut Enemy_MiniDraygon = Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mdn_var_B = v1;
    if v1 != 0 {
        (*E).base.x_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v0 + 91 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            + 4 as ::core::ffi::c_int) as uint16;
        (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v0 + 93 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            + 10 as ::core::ffi::c_int) as uint16;
        (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_8727 as ::core::ffi::c_int as uint16;
    } else {
        (*E).base.x_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v0 + 91 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int) as uint16;
        (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v0 + 93 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            + 10 as ::core::ffi::c_int) as uint16;
        (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_86C3 as ::core::ffi::c_int as uint16;
    }
    MiniDraygon_Func_12(game);
}
pub unsafe fn EvirProjectile_Init(game: &mut Game) {
    let mut E: *mut Enemy_MiniDraygon = Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_876F as ::core::ffi::c_int as uint16;
    MiniDraygon_Func_12(game);
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xe54)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).base.palette_index = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v2 + 73 as ::core::ffi::c_int) as isize);
    (*E).base.vram_tiles_index = *((&raw mut game.g_ram as *mut uint8).offset(0xe84)
        as *mut uint16)
        .offset((v2 + 74 as ::core::ffi::c_int) as isize);
    MiniDraygon_Func_3(game);
    (*E).mdn_var_01 = 0 as uint16;
    (*E).mdn_var_0C = 0 as uint16;
    (*E).mdn_var_0B = 0 as uint16;
    (*E).mdn_var_F = 0 as uint16;
    (*E).mdn_var_C = (0xa88a34 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MiniDraygon_Func_3(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xe54)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v0 + 83 as ::core::ffi::c_int) as isize);
    let mut E: *mut Enemy_MiniDraygon = Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mdn_var_B = v1;
    if v1 != 0 {
        (*E).base.x_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v0 + 59 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            + 4 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).base.x_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v0 + 59 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int) as uint16;
    }
    (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v0 + 61 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        + 18 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CallMiniDraygonFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        11045154 => {
            MiniDraygon_Func_4(game);
            return;
        }
        11045428 => {
            MiniDraygon_Func_8(game);
            return;
        }
        11045435 => {
            MiniDraygon_Func_9(game);
            return;
        }
        11045496 => {
            MiniDraygon_Func_10(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn MiniDraygon_Main(game: &mut Game) {
    let mut E: *mut Enemy_MiniDraygon = Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallMiniDraygonFunc(
        ((*E).mdn_var_C as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn MiniDraygon_Func_4(game: &mut Game) {
    if (*Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .mdn_parameter_1
        != 0
    {
        MiniDraygon_Func_6(game);
    } else {
        MiniDraygon_Func_5(game);
    };
}
pub unsafe fn MiniDraygon_Func_5(game: &mut Game) {
    if (*Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 128 as ::core::ffi::c_int,
        game,
    ))
    .mdn_var_0C
        == 0
    {
        MiniDraygon_Func_1(game);
    }
    let mut E: *mut Enemy_MiniDraygon = Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).mdn_var_00 != 0 {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            ((*E).mdn_var_04 as uint32) << 16 as ::core::ffi::c_int | (*E).mdn_var_03 as uint32,
        );
    } else {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            ((*E).mdn_var_06 as uint32) << 16 as ::core::ffi::c_int | (*E).mdn_var_05 as uint32,
        );
    }
    (*E).mdn_var_E = (*E).mdn_var_E.wrapping_sub(1);
    if (*E).mdn_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mdn_var_E = *(&raw mut (*E).mdn_parameter_2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as uint16;
        (*E).mdn_var_00 =
            ((*E).mdn_var_00 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MiniDraygon_Func_6(game: &mut Game) {
    MiniDraygon_Func_2(game);
}
pub unsafe fn EvirProjectile_Main(game: &mut Game) {
    let mut E: *mut Enemy_MiniDraygon = Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).base.frozen_timer == 0 {
        if (*E).mdn_var_0B != 0 {
            (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_876F as ::core::ffi::c_int as uint16;
            MiniDraygon_Func_12(game);
        } else if (*E).mdn_var_0C != 0 {
            (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_8775 as ::core::ffi::c_int as uint16;
            MiniDraygon_Func_12(game);
        } else {
            MiniDraygon_Func_7(game);
        }
    }
    CallMiniDraygonFunc(
        ((*E).mdn_var_C as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn MiniDraygon_Func_7(game: &mut Game) {
    if IsSamusWithinEnemy_X(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            - 128 as ::core::ffi::c_int) as uint16,
        0x80 as uint16,
        game,
    ) != 0
    {
        let mut r22: uint16 = -((CalculateAngleOfSamusFromEnemy(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        ) as ::core::ffi::c_int
            - 64 as ::core::ffi::c_int) as uint8
            as ::core::ffi::c_int) as uint16;
        let mut E: *mut Enemy_MiniDraygon = Get_MiniDraygon(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        SetHiLo(
            &raw mut (*E).mdn_var_07,
            &raw mut (*E).mdn_var_08,
            CosineMult8bitFull(r22, 4 as uint16, game),
        );
        SetHiLo(
            &raw mut (*E).mdn_var_09,
            &raw mut (*E).mdn_var_0A,
            SineMult8bitFull(r22, 4 as uint16, game),
        );
        (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_876F as ::core::ffi::c_int as uint16;
        MiniDraygon_Func_12(game);
        (*E).mdn_var_0B = 1 as uint16;
        (*E).mdn_var_C = (0xa88a3b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MiniDraygon_Func_8(game: &mut Game) {
    MiniDraygon_Func_3(game);
}
pub unsafe fn MiniDraygon_Func_9(game: &mut Game) {
    MiniDraygon_Func_11(game);
    let mut E: *mut Enemy_MiniDraygon = Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).mdn_var_07 as uint32) << 16 as ::core::ffi::c_int | (*E).mdn_var_08 as uint32,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).mdn_var_09 as uint32) << 16 as ::core::ffi::c_int | (*E).mdn_var_0A as uint32,
    );
}
pub unsafe fn MiniDraygon_Func_10(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16).offset(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int)
            + 77 as ::core::ffi::c_int) as isize,
    ) == 0
    {
        let mut E: *mut Enemy_MiniDraygon = Get_MiniDraygon(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        if (*E).mdn_var_0C != 0 {
            MiniDraygon_Func_3(game);
            (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
                + (*E).mdn_var_F as ::core::ffi::c_int) as uint16;
        } else {
            (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_876F as ::core::ffi::c_int as uint16;
            MiniDraygon_Func_12(game);
            (*E).mdn_var_0C = 0 as uint16;
            (*E).mdn_var_0B = 0 as uint16;
            (*E).mdn_var_C =
                (0xa88a34 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn MiniDraygon_Func_11(game: &mut Game) {
    if EnemyFunc_ADA3(0x100 as uint16, game) != 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16).offset(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int)
                + 77 as ::core::ffi::c_int) as isize,
        ) == 0
        {
            let mut E: *mut Enemy_MiniDraygon = Get_MiniDraygon(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            );
            (*E).mdn_var_0B = 0 as uint16;
            (*E).mdn_var_0C = 1 as uint16;
            (*E).mdn_var_C =
                (0xa88a78 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).mdn_var_0C = 1 as uint16;
            (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_8775 as ::core::ffi::c_int as uint16;
            MiniDraygon_Func_12(game);
        }
    }
}
pub unsafe fn MiniDraygon_Func_12(game: &mut Game) {
    let mut E: *mut Enemy_MiniDraygon = Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut mdn_var_02: uint16 = (*E).mdn_var_02;
    if mdn_var_02 as ::core::ffi::c_int != (*E).mdn_var_01 as ::core::ffi::c_int {
        (*E).base.current_instruction = mdn_var_02;
        (*E).mdn_var_01 = mdn_var_02;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn MiniDraygon_Touch(game: &mut Game) {
    Enemy_NormalTouchAI_A8(game);
    MiniDraygon_Func_13(game);
}
pub unsafe fn MiniDraygon_Powerbomb(game: &mut Game) {
    Enemy_NormalPowerBombAI_A8(game);
    MiniDraygon_Func_13(game);
}
pub unsafe fn MiniDraygon_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    MiniDraygon_Func_13(game);
}
pub unsafe fn MiniDraygon_Func_13(game: &mut Game) {
    let mut E: *mut Enemy_MiniDraygon = Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut E1: *mut Enemy_MiniDraygon = Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    let mut E2: *mut Enemy_MiniDraygon = Get_MiniDraygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 128 as ::core::ffi::c_int,
        game,
    );
    if (*E).base.health == 0 {
        (*E1).base.properties =
            ((*E1).base.properties as ::core::ffi::c_int | 0x200 as ::core::ffi::c_int) as uint16;
        (*E2).base.properties =
            ((*E2).base.properties as ::core::ffi::c_int | 0x200 as ::core::ffi::c_int) as uint16;
    }
    let mut frozen_timer: uint16 = (*E).base.frozen_timer;
    if frozen_timer != 0 {
        (*E1).base.frozen_timer = frozen_timer;
        (*E1).base.ai_handler_bits =
            ((*E1).base.ai_handler_bits as ::core::ffi::c_int | 4 as ::core::ffi::c_int) as uint16;
        if (*E2).mdn_var_C as ::core::ffi::c_int
            != 0xa88a3b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            (*E2).base.ai_handler_bits = ((*E2).base.ai_handler_bits as ::core::ffi::c_int
                | 4 as ::core::ffi::c_int) as uint16;
            (*E2).base.frozen_timer = (*E).base.frozen_timer;
        }
    }
}
pub unsafe fn MorphBallEye_Init(game: &mut Game) {
    let mut E: *mut Enemy_MorphBallEye = Get_MorphBallEye(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_DisableSamusColl as ::core::ffi::c_int) as uint16;
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A8 as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    if (*E).mbee_parameter_2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int =
            (*E).mbee_parameter_2 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int;
        (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
            + g_word_A890CA[v1 as usize] as ::core::ffi::c_int) as uint16;
        (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
            + g_word_A890D2[v1 as usize] as ::core::ffi::c_int) as uint16;
        (*E).mbee_var_F = (0xa891dc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction = *(game.g_rom.offset(
            (((0xa890da as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa890da as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v1 as isize);
        let mut i: ::core::ffi::c_int = 510 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9100) as *mut uint16 as *mut uint8)
                .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 255 as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
    } else {
        (*E).mbee_var_F = (0xa890f1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        if (*E).mbee_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).base.current_instruction = addr_stru_A8900E as ::core::ffi::c_int as uint16;
        } else {
            (*E).base.current_instruction = addr_stru_A88FFC as ::core::ffi::c_int as uint16;
        }
    };
}
pub unsafe fn MorphBallEye_Main(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16) as ::core::ffi::c_int
        & 4 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut E: *mut Enemy_MorphBallEye = Get_MorphBallEye(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        CallEnemyPreInstr(
            ((*E).mbee_var_F as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
            game,
        );
    }
}
pub unsafe fn MorphBallEye_Func_1(mut k: uint16, game: &mut Game) {
    if IsSamusWithinEnemy_Y(k, g_word_A89054, game) as ::core::ffi::c_int != 0
        && IsSamusWithinEnemy_X(k, g_word_A89050, game) as ::core::ffi::c_int != 0
    {
        let mut E: *mut Enemy_MorphBallEye = Get_MorphBallEye(k as ::core::ffi::c_int, game);
        (*E).mbee_var_E = 32 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        if (*E).mbee_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).base.current_instruction =
                addr_kMorphBallEye_Ilist_9026 as ::core::ffi::c_int as uint16;
        } else {
            (*E).base.current_instruction =
                addr_kMorphBallEye_Ilist_9014 as ::core::ffi::c_int as uint16;
        }
        (*E).mbee_var_F = (0xa8912e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MorphBallEye_Func_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MorphBallEye = Get_MorphBallEye(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).mbee_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).mbee_var_E = (*E).mbee_var_E.wrapping_sub(1);
    let mut v3: bool = (*E).mbee_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        QueueSfx2_Max6(0x17 as uint16, game);
        SpawnMorphBallEyeBeamHdma(game);
        (*E).mbee_var_F = (0xa89160 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mbee_var_D = CalculateAngleFromXY(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - (*E).base.y_pos as ::core::ffi::c_int) as uint16,
            game,
        );
    }
}
pub unsafe fn MorphBallEye_Func_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MorphBallEye = Get_MorphBallEye(k as ::core::ffi::c_int, game);
    if IsSamusWithinEnemy_Y(k, g_word_A89056, game) as ::core::ffi::c_int != 0
        && IsSamusWithinEnemy_X(k, g_word_A89052, game) as ::core::ffi::c_int != 0
    {
        let mut v3: uint16 = CalculateAngleFromXY(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - (*E).base.y_pos as ::core::ffi::c_int) as uint16,
            game,
        );
        (*E).mbee_var_D = v3;
        (*E).base.current_instruction = (((v3 as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int)
            >> 2 as ::core::ffi::c_int)
            - 28756 as ::core::ffi::c_int) as uint16;
    } else {
        QueueSfx2_Max6(0x71 as uint16, game);
        (*E).mbee_var_C = 0 as uint16;
        (*E).mbee_var_E = 32 as uint16;
        if (*E).mbee_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).base.current_instruction =
                addr_kMorphBallEye_Ilist_9002 as ::core::ffi::c_int as uint16;
        } else {
            (*E).base.current_instruction =
                addr_kMorphBallEye_Ilist_8FF0 as ::core::ffi::c_int as uint16;
        }
        (*E).mbee_var_F = (0xa891ce as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    (*E).base.instruction_timer = 1 as uint16;
}
pub unsafe fn MorphBallEye_Func_4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MorphBallEye = Get_MorphBallEye(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).mbee_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).mbee_var_E = (*E).mbee_var_E.wrapping_sub(1);
    let mut v3: bool = (*E).mbee_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*E).mbee_var_F = (0xa890f1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Fune_Instr_2(mut k: uint16, mut jp: *const uint16, game: &mut Game) -> *const uint16 {
    QueueSfx2_Max9(0x1f as uint16, game);
    return jp;
}
pub unsafe fn Fune_Instr_6(mut k: uint16, mut jp: *const uint16, game: &mut Game) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) =
        *(&raw mut (*(Get_Fune as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut Enemy_Fune)(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .fune_parameter_2 as *mut uint8)
            .offset(0) as uint16;
    SpawnEprojWithGfx(
        0 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_stru_86DFBC as ::core::ffi::c_int as uint16,
        game,
    );
    return jp;
}
pub unsafe fn Fune_Instr_7(mut k: uint16, mut jp: *const uint16, game: &mut Game) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) =
        *(&raw mut (*(Get_Fune as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut Enemy_Fune)(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .fune_parameter_2 as *mut uint8)
            .offset(0) as uint16;
    SpawnEprojWithGfx(
        1 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_stru_86DFBC as ::core::ffi::c_int as uint16,
        game,
    );
    return jp;
}
pub unsafe fn Fune_Instr_1(mut k: uint16, mut jp: *const uint16, game: &mut Game) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) =
        *(&raw mut (*(Get_Fune as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut Enemy_Fune)(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .fune_parameter_2 as *mut uint8)
            .offset(0) as uint16;
    SpawnEprojWithGfx(
        0 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_stru_86DFCA as ::core::ffi::c_int as uint16,
        game,
    );
    return jp;
}
pub unsafe fn Fune_Instr_4(mut k: uint16, mut jp: *const uint16, game: &mut Game) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) =
        *(&raw mut (*(Get_Fune as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut Enemy_Fune)(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .fune_parameter_2 as *mut uint8)
            .offset(0) as uint16;
    SpawnEprojWithGfx(
        1 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_stru_86DFCA as ::core::ffi::c_int as uint16,
        game,
    );
    return jp;
}
pub unsafe fn Fune_Instr_3(mut k: uint16, mut jp: *const uint16, game: &mut Game) -> *const uint16 {
    let mut E: *mut Enemy_Fune = Get_Fune(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).fune_var_A = ((*E).fune_var_A as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    (*E).fune_var_B = (0xa89737 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if (*E).fune_var_D != 0 {
        (*E).fune_var_B = (0xa8975c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    return jp;
}
pub unsafe fn Fune_Instr_5(mut k: uint16, mut jp: *const uint16, game: &mut Game) -> *const uint16 {
    let mut E: *mut Enemy_Fune = Get_Fune(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).fune_var_A = ((*E).fune_var_A as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    (*E).fune_var_B = (0xa89737 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if (*E).fune_var_D != 0 {
        (*E).fune_var_B = (0xa8975c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    return jp;
}
pub unsafe fn Fune_Init(game: &mut Game) {
    let mut E: *mut Enemy_Fune = Get_Fune(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).fune_var_A = addr_off_A896D7 as ::core::ffi::c_int as uint16;
    (*E).fune_var_B = (0xa89737 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 =
        ((*E).fune_parameter_1 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as uint16;
    (*E).fune_var_D = v1;
    if v1 != 0 {
        (*E).fune_var_A = addr_off_A896DF as ::core::ffi::c_int as uint16;
        (*E).fune_var_B = (0xa8975c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    if (*E).fune_parameter_1 as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).fune_var_A =
            ((*E).fune_var_A as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
    Fune_Func_4(game);
    (*E).fune_var_C = *(&raw mut (*E).fune_parameter_2 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as uint16;
    (*E).fune_var_F = *(&raw mut (*E).fune_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as uint16;
    (*E).fune_var_E = 0 as uint16;
}
pub unsafe fn CallFuneFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        11048759 => {
            Fune_Func_1(game);
            return;
        }
        11048796 => {
            Fune_Func_2(game);
            return;
        }
        11048846 => return,
        11048847 => return,
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn Fune_Main(game: &mut Game) {
    let mut E: *mut Enemy_Fune = Get_Fune(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallFuneFunc(
        ((*E).fune_var_B as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn Fune_Func_1(game: &mut Game) {
    let mut E: *mut Enemy_Fune = Get_Fune(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).fune_var_E = (*E).fune_var_E.wrapping_add(1);
    if ((*E).fune_var_E as ::core::ffi::c_int - (*E).fune_var_F as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).fune_var_A =
            ((*E).fune_var_A as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
        Fune_Func_4(game);
        (*E).fune_var_B = (0xa8978e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).fune_var_E = 0 as uint16;
    }
}
pub unsafe fn Fune_Func_2(game: &mut Game) {
    let mut E: *mut Enemy_Fune = Get_Fune(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).fune_var_00 = (*E).fune_var_A;
    if Fune_Func_3(game) != 0 {
        let mut fune_var_00: uint16 = (*E).fune_var_00;
        (*E).fune_var_A = fune_var_00;
        fune_var_00 = (fune_var_00 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
        (*E).fune_var_00 = fune_var_00;
        (*E).fune_var_A = fune_var_00;
        Fune_Func_4(game);
        (*E).fune_var_B = (0xa8978f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    (*E).fune_var_A = (*E).fune_var_00;
}
pub unsafe fn Fune_Func_3(game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_Fune = Get_Fune(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    return IsSamusWithinEnemy_Y(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (*E).fune_var_C,
        game,
    );
}
pub unsafe fn Fune_Func_4(game: &mut Game) {
    let mut E: *mut Enemy_Fune = Get_Fune(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction = *(RomPtr_A8((*E).fune_var_A as uint16_t, game) as *mut uint16);
}
pub unsafe fn WreckedShipGhost_Init(game: &mut Game) {
    let mut E: *mut Enemy_WreckedShipGhost = Get_WreckedShipGhost(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_DisableSamusColl as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction =
        addr_kWreckedShipGhost_Ilist_9A8C as ::core::ffi::c_int as uint16;
    (*E).wsgt_var_A = (0xa89d13 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).wsgt_var_B = (g_word_A89AA4 as ::core::ffi::c_int + 160 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = u16::swap_bytes((*E).base.palette_index);
    let mut v3: uint16 =
        (16 as ::core::ffi::c_int * v2 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as uint16;
    let mut n: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        n -= 1;
        if !(n >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
pub unsafe fn CallWreckedShipGhost(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        11049794 => {
            WreckedShipGhost_Func_1(k, game);
            return;
        }
        11049901 => {
            WreckedShipGhost_Func_2(k, game);
            return;
        }
        11050089 => {
            WreckedShipGhost_Func_4(k, game);
            return;
        }
        11050122 => {
            WreckedShipGhost_Func_5(k, game);
            return;
        }
        11050259 => {
            WreckedShipGhost_Func_6(k, game);
            return;
        }
        11050294 => {
            WreckedShipGhost_Func_7(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn WreckedShipGhost_Main(game: &mut Game) {
    let mut E: *mut Enemy_WreckedShipGhost = Get_WreckedShipGhost(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallWreckedShipGhost(
        ((*E).wsgt_var_A as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn WreckedShipGhost_Func_1(mut k: uint16, game: &mut Game) {
    WreckedShipGhost_Func_3(k, game);
    let mut E: *mut Enemy_WreckedShipGhost = Get_WreckedShipGhost(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v3: uint16 = u16::swap_bytes((*E).base.palette_index);
    let mut v4: uint16 =
        (16 as ::core::ffi::c_int * v3 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as uint16;
    let mut n: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
    let mut v1: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
    loop {
        let mut v5: ::core::ffi::c_int = v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(v5 as isize)
            as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
            - 31 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let ref mut fresh12 = *((&raw mut game.g_ram as *mut uint8).offset(0xc000)
                as *mut uint16)
                .offset(v5 as isize);
            *fresh12 = (*fresh12 as ::core::ffi::c_int + 1057 as ::core::ffi::c_int) as uint16;
            v1 -= 1;
        }
        v4 = (v4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
    if (v1 - 16 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        (*E).wsgt_var_A = (0xa89bad as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut v7: uint16 = u16::swap_bytes((*E).base.palette_index);
        let mut v8: uint16 = (16 as ::core::ffi::c_int * v7 as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int) as uint16;
        let mut v9: uint16 = 0 as uint16;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                .offset((v8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                kWreckedShipGhost_Palette
                    [(v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
            v8 = (v8 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            v9 = (v9 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v9 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    }
}
pub unsafe fn WreckedShipGhost_Func_2(mut k: uint16, game: &mut Game) {
    let mut v4: uint16 = WreckedShipGhost_Func_8(game);
    WreckedShipGhost_Func_3(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let mut E: *mut Enemy_WreckedShipGhost = Get_WreckedShipGhost(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).wsgt_var_B as ::core::ffi::c_int | v4 as ::core::ffi::c_int == 0 {
        (*E).base.properties =
            ((*E).base.properties as ::core::ffi::c_int & 0xfaff as ::core::ffi::c_int) as uint16;
        (*E).wsgt_var_A = (0xa89c8a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).wsgt_var_00 = (*E).base.y_pos;
        (*E).wsgt_var_B = 1 as uint16;
        (*E).wsgt_var_C = 2 as uint16;
        (*E).wsgt_var_01 = 0 as uint16;
        (*E).wsgt_var_02 = g_word_A89AA2;
        (*E).wsgt_var_B = g_word_A89AA6;
        (*E).wsgt_var_05 = 4 as uint16;
        let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
        (*E).wsgt_var_06 = *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
        (*E).wsgt_var_07 = v2;
        (*E).wsgt_var_08 = v2;
        (*E).wsgt_var_09 = 12 as uint16;
        let mut v3: uint16 = *(((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16
            as *mut uint8)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                    as ::core::ffi::c_int as isize,
            ) as *mut uint16);
        (*E).wsgt_var_0A = v3;
        (*E).wsgt_var_0B = v3;
        (*E).wsgt_var_0C = v3;
        (*E).wsgt_var_0D = g_word_A89A9E;
        (*E).wsgt_var_0E = g_word_A89A9C;
    }
}
pub unsafe fn WreckedShipGhost_Func_3(mut k: uint16, game: &mut Game) {
    let mut v3: uint16 = 0;
    let mut wsgt_var_C: uint16 = 0;
    let mut current_block: u64;
    let mut v5: int16 = 0;
    let mut E: *mut Enemy_WreckedShipGhost = Get_WreckedShipGhost(k as ::core::ffi::c_int, game);
    let mut wsgt_var_B: uint16 = (*E).wsgt_var_B;
    if wsgt_var_B == 0 {
        current_block = 6804514963299881235;
    } else {
        v3 = 0;
        v3 = (wsgt_var_B as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        (*E).wsgt_var_B = v3;
        if v3 == 0 {
            wsgt_var_C = 0;
            wsgt_var_C = (*E).wsgt_var_C;
            v5 = g_word_A89ACC
                [(wsgt_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
                as int16;
            if (v5 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                (*E).wsgt_var_B = 0 as uint16;
                (*E).wsgt_var_C = 0 as uint16;
                return;
            }
            (*E).wsgt_var_B = v5 as uint16;
            (*E).wsgt_var_C =
                (wsgt_var_C as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if wsgt_var_C as ::core::ffi::c_int & 2 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            {
                current_block = 6804514963299881235;
            } else {
                current_block = 12800627514080957624;
            }
        } else {
            current_block = 12800627514080957624;
        }
    }
    match current_block {
        6804514963299881235 => {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Invisible as ::core::ffi::c_int))
                as uint16;
        }
        _ => {}
    };
}
pub unsafe fn WreckedShipGhost_Func_4(mut k: uint16, game: &mut Game) {
    if WreckedShipGhost_Func_8(game) == 0 {
        let mut E: *mut Enemy_WreckedShipGhost = Get_WreckedShipGhost(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*E).wsgt_var_A = (0xa89d13 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
        (*E).wsgt_var_B = g_word_A89AA4;
    }
}
pub unsafe fn WreckedShipGhost_Func_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_WreckedShipGhost = Get_WreckedShipGhost(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).wsgt_var_02 as uint32) << 16 as ::core::ffi::c_int | (*E).wsgt_var_01 as uint32,
    );
    AddToHiLo(
        &raw mut (*E).wsgt_var_02,
        &raw mut (*E).wsgt_var_01,
        (if (((*E).base.y_pos as ::core::ffi::c_int - (*E).wsgt_var_00 as ::core::ffi::c_int)
            as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            g_word_A89AA0 as ::core::ffi::c_int
        } else {
            -(g_word_A89AA0 as ::core::ffi::c_int)
        }) as uint32,
    );
    (*E).wsgt_var_B = (*E).wsgt_var_B.wrapping_sub(1);
    if (*E).wsgt_var_B as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        (*E).wsgt_var_A = (0xa89c69 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
        let mut t: ::core::ffi::c_int =
            u16::swap_bytes((*E).base.palette_index) as ::core::ffi::c_int;
        let mut v9: uint16 = (16 as ::core::ffi::c_int * t + 256 as ::core::ffi::c_int) as uint16;
        let mut n: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                .offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                0x7fff as uint16;
            v9 = (v9 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            n -= 1;
            if !(n & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
                break;
            }
        }
    }
}
pub unsafe fn WreckedShipGhost_Func_6(mut k: uint16, game: &mut Game) {
    let mut v3: uint16 = 0;
    let mut E: *mut Enemy_WreckedShipGhost = Get_WreckedShipGhost(k as ::core::ffi::c_int, game);
    let mut wsgt_var_B: uint16 = (*E).wsgt_var_B;
    if wsgt_var_B == 0 || {
        v3 = (wsgt_var_B as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        (*E).wsgt_var_B = v3;
        (*E).wsgt_var_B as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    } {
        (*E).wsgt_var_B = 1 as uint16;
        (*E).wsgt_var_C = 2 as uint16;
        (*E).wsgt_var_A = (0xa89d36 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn WreckedShipGhost_Func_7(mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v4: int16 = 0;
    let mut v7: int16 = 0;
    let mut v9: int16 = 0;
    let mut E: *mut Enemy_WreckedShipGhost = Get_WreckedShipGhost(k as ::core::ffi::c_int, game);
    let mut v3: uint16 = 0;
    let mut current_block_26: u64;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        - (*E).wsgt_var_07 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
        || (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            - (*E).wsgt_var_08 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        || {
            v2 = *(((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16) as int16;
            ((v2 as ::core::ffi::c_int - (*E).wsgt_var_0B as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
        }
        || (v2 as ::core::ffi::c_int - (*E).wsgt_var_0C as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
    {
        (*E).wsgt_var_0D = g_word_A89A9E;
        v4 = 0 as int16;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            - (*E).wsgt_var_06 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            v4 = (if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                == (*E).wsgt_var_06 as ::core::ffi::c_int
            {
                4 as ::core::ffi::c_int
            } else {
                8 as ::core::ffi::c_int
            }) as int16;
        }
        if v4 as ::core::ffi::c_int == (*E).wsgt_var_05 as ::core::ffi::c_int {
            v7 = 0 as int16;
            v9 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - (*E).wsgt_var_0A as ::core::ffi::c_int) as int16;
            if v9 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                v7 = (if v9 as ::core::ffi::c_int != 0 {
                    24 as ::core::ffi::c_int
                } else {
                    12 as ::core::ffi::c_int
                }) as int16;
            }
            if v7 as ::core::ffi::c_int == (*E).wsgt_var_09 as ::core::ffi::c_int {
                let mut v10: uint16 =
                    ((*E).wsgt_var_0E as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
                (*E).wsgt_var_0E = v10;
                if v10 == 0 {
                    current_block_26 = 1955728515491069284;
                } else {
                    current_block_26 = 3437258052017859086;
                }
            } else {
                (*E).wsgt_var_09 = v7 as uint16;
                (*E).wsgt_var_0E = g_word_A89A9C;
                current_block_26 = 3437258052017859086;
            }
        } else {
            (*E).wsgt_var_05 = v4 as uint16;
            (*E).wsgt_var_0E = g_word_A89A9C;
            current_block_26 = 3437258052017859086;
        }
    } else {
        v3 = 0;
        v3 = ((*E).wsgt_var_0D as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        (*E).wsgt_var_0D = v3;
        if v3 == 0 {
            (*E).wsgt_var_05 = 4 as uint16;
            (*E).wsgt_var_09 = 12 as uint16;
            current_block_26 = 1955728515491069284;
        } else {
            current_block_26 = 3437258052017859086;
        }
    }
    match current_block_26 {
        3437258052017859086 => {}
        _ => {
            (*E).wsgt_var_A =
                (0xa89b42 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).wsgt_var_0D = g_word_A89A9E;
            (*E).wsgt_var_0E = g_word_A89A9C;
            let mut v6: ::core::ffi::c_int = ((*E).wsgt_var_09 as ::core::ffi::c_int
                + (*E).wsgt_var_05 as ::core::ffi::c_int)
                as uint16 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            (*E).base.x_pos = (g_word_A89AA8[v6 as usize] as ::core::ffi::c_int
                + *(((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16 as *mut uint8)
                    .offset(k as ::core::ffi::c_int as isize) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
            (*E).base.y_pos = (g_word_A89AA8[(v6 + 1 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int
                + *(((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16 as *mut uint8)
                    .offset(k as ::core::ffi::c_int as isize) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
            return;
        }
    }
    let mut v11: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
    (*E).wsgt_var_06 = *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
    (*E).wsgt_var_07 = (v11 as ::core::ffi::c_int - g_word_A89D32 as ::core::ffi::c_int) as uint16;
    (*E).wsgt_var_08 = (g_word_A89D32 as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    let mut v13: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    (*E).wsgt_var_0A = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    (*E).wsgt_var_0B = (v13 as ::core::ffi::c_int - g_word_A89D34 as ::core::ffi::c_int) as uint16;
    (*E).wsgt_var_0C = (g_word_A89D34 as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
}
pub unsafe fn WreckedShipGhost_Func_8(game: &mut Game) -> uint16 {
    let mut v0: uint16 = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x797) as *mut uint16) == 0 {
        let mut E: *mut Enemy_WreckedShipGhost = Get_WreckedShipGhost(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        let mut v2: uint16 = u16::swap_bytes((*E).base.palette_index);
        let mut v3: uint16 = (16 as ::core::ffi::c_int * v2 as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int) as uint16;
        let mut r20: uint16 = (16 as ::core::ffi::c_int * v2 as ::core::ffi::c_int
            + 288 as ::core::ffi::c_int) as uint16;
        loop {
            let mut v4: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            if *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                .offset(v4 as isize) as ::core::ffi::c_int
                != *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                    .offset(v4 as isize) as ::core::ffi::c_int
            {
                let mut r18: uint16 =
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                        .offset(v4 as isize) as ::core::ffi::c_int
                        & 0x1f as ::core::ffi::c_int) as uint16;
                let mut v5: uint16 =
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset(v4 as isize) as ::core::ffi::c_int
                        & 0x1f as ::core::ffi::c_int) as uint16;
                if v5 as ::core::ffi::c_int != r18 as ::core::ffi::c_int {
                    let mut v6: uint16 = 0;
                    if (v5 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as int16
                        as ::core::ffi::c_int
                        >= 0 as ::core::ffi::c_int
                    {
                        v6 = (v5 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
                    } else {
                        v6 = (v5 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
                    }
                    r18 = v6;
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset(v4 as isize) = (v6 as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                            .offset(v4 as isize) as ::core::ffi::c_int
                            & 0xffe0 as ::core::ffi::c_int)
                        as uint16;
                    v0 = v0.wrapping_add(1);
                }
                r18 = (*((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                    .offset(v4 as isize) as ::core::ffi::c_int
                    & 0x3e0 as ::core::ffi::c_int) as uint16;
                let mut v7: uint16 =
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset(v4 as isize) as ::core::ffi::c_int
                        & 0x3e0 as ::core::ffi::c_int) as uint16;
                if v7 as ::core::ffi::c_int != r18 as ::core::ffi::c_int {
                    let mut v8: uint16 = 0;
                    if (v7 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as int16
                        as ::core::ffi::c_int
                        >= 0 as ::core::ffi::c_int
                    {
                        v8 = (v7 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
                    } else {
                        v8 = (v7 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
                    }
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset(v4 as isize) = (v8 as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                            .offset(v4 as isize) as ::core::ffi::c_int
                            & 0xfc1f as ::core::ffi::c_int)
                        as uint16;
                    v0 = v0.wrapping_add(1);
                }
                r18 = (*((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                    .offset(v4 as isize) as ::core::ffi::c_int
                    & 0x7c00 as ::core::ffi::c_int) as uint16;
                let mut v9: uint16 =
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset(v4 as isize) as ::core::ffi::c_int
                        & 0x7c00 as ::core::ffi::c_int) as uint16;
                if v9 as ::core::ffi::c_int != r18 as ::core::ffi::c_int {
                    let mut v10: uint16 = 0;
                    if (v9 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as int16
                        as ::core::ffi::c_int
                        >= 0 as ::core::ffi::c_int
                    {
                        v10 = (v9 as ::core::ffi::c_int - 1024 as ::core::ffi::c_int) as uint16;
                    } else {
                        v10 = (v9 as ::core::ffi::c_int + 1024 as ::core::ffi::c_int) as uint16;
                    }
                    r18 = v10;
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset(v4 as isize) = (v10 as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                            .offset(v4 as isize) as ::core::ffi::c_int
                            & 0x83ff as ::core::ffi::c_int)
                        as uint16;
                    v0 = v0.wrapping_add(1);
                }
            }
            v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v3 as ::core::ffi::c_int - r20 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    }
    return v0;
}
pub unsafe fn YappingMaw_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).ymw_var_32 = *(game.g_rom.offset(
        (((0xa8a0ab as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8a0ab as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16);
    (*E).ymw_var_33 = *(game.g_rom.offset(
        (((0xa8a0ad as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8a0ad as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16);
    return jp;
}
pub unsafe fn YappingMaw_Instr_4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).ymw_var_32 = *(game.g_rom.offset(
        (((0xa8a0c3 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8a0c3 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16);
    (*E).ymw_var_33 = *(game.g_rom.offset(
        (((0xa8a0c5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8a0c5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16);
    return jp;
}
pub unsafe fn YappingMaw_Instr_5(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).ymw_var_32 = *(game.g_rom.offset(
        (((0xa8a0b3 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8a0b3 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16);
    (*E).ymw_var_33 = *(game.g_rom.offset(
        (((0xa8a0b5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8a0b5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16);
    return jp;
}
pub unsafe fn YappingMaw_Instr_7(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).ymw_var_32 = *(game.g_rom.offset(
        (((0xa8a0bb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8a0bb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16);
    (*E).ymw_var_33 = *(game.g_rom.offset(
        (((0xa8a0bd as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8a0bd as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16);
    return jp;
}
pub unsafe fn YappingMaw_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).ymw_var_32 = *(game.g_rom.offset(
        (((0xa8a0a7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8a0a7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16);
    (*E).ymw_var_33 = *(game.g_rom.offset(
        (((0xa8a0a9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8a0a9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16);
    return jp;
}
pub unsafe fn YappingMaw_Instr_6(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).ymw_var_32 = *(game.g_rom.offset(
        (((0xa8a0b7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8a0b7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16);
    (*E).ymw_var_33 = *(game.g_rom.offset(
        (((0xa8a0b9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8a0b9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16);
    return jp;
}
pub unsafe fn YappingMaw_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if (*Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .ymw_var_36
        == 0
    {
        QueueSfx2_Max6(0x2f as uint16, game);
    }
    return jp;
}
pub unsafe fn YappingMaw_Init(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).ymw_var_0C = (*E).base.x_pos;
    (*E).ymw_var_0D = (*E).base.y_pos;
    (*E).ymw_var_00 = 0 as uint16;
    (*E).ymw_var_01 = 0 as uint16;
    (*E).ymw_var_02 = 0 as uint16;
    (*E).ymw_var_03 = 0 as uint16;
    (*E).ymw_var_04 = 0 as uint16;
    (*E).ymw_var_05 = 0 as uint16;
    (*E).ymw_var_06 = 0 as uint16;
    (*E).ymw_var_07 = 0 as uint16;
    (*E).ymw_var_30 = 0 as uint16;
    (*E).ymw_var_F = (*E).ymw_parameter_1;
    (*E).ymw_var_E = 64 as uint16;
    (*E).base.current_instruction = addr_kYappingMaw_Ilist_9F6F as ::core::ffi::c_int as uint16;
    let mut R36: uint16 = 57 as uint16;
    let mut R34: uint16 = 8 as uint16;
    if (*E).ymw_parameter_2 == 0 {
        (*E).base.current_instruction = addr_kYappingMaw_Ilist_9FC7 as ::core::ffi::c_int as uint16;
        R36 = 56 as uint16;
        R34 = -(8 as ::core::ffi::c_int) as uint16;
    }
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).ymw_var_A = (0xa8a235 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_45 =
        ((*E).base.palette_index as ::core::ffi::c_int & 0xe00 as ::core::ffi::c_int) as uint16;
    v1 = 3 as int16;
    (*E).ymw_var_44 = 3 as uint16;
    loop {
        SpawnEprojWithGfx(
            v1 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            addr_kEproj_YappingMawsBody as ::core::ffi::c_int as uint16,
            game,
        );
        v1 = ((*E).ymw_var_44 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
        (*E).ymw_var_44 = v1 as uint16;
        if !(v1 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
    let mut v3: uint16 = ((*E).base.vram_tiles_index as ::core::ffi::c_int
        | (*E).base.palette_index as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_47 = v3;
    (*E).ymw_var_46 = CreateSpriteAtPos(
        (*E).base.x_pos,
        (R34 as ::core::ffi::c_int + (*E).base.y_pos as ::core::ffi::c_int) as uint16,
        R36,
        v3,
        game,
    );
}
pub unsafe fn CallYappingMawFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        11051573 => {
            YappingMaw_Func_1(game);
            return;
        }
        11051660 => {
            YappingMaw_Func_2(game);
            return;
        }
        11052101 => {
            YappingMaw_Func_8(game);
            return;
        }
        11052682 => {
            YappingMaw_Func_11(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn YappingMaw_Main(game: &mut Game) {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).ymw_var_35 = (*E).ymw_var_35.wrapping_sub(1);
    (*E).ymw_var_36 = CheckIfEnemyIsOnScreen(game);
    CallYappingMawFunc(
        ((*E).ymw_var_A as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
        game,
    );
    YappingMaw_Func_15(game);
    YappingMaw_Func_14(game);
    YappingMaw_Func_13(game);
    YappingMaw_Func_12(game);
}
pub unsafe fn YappingMaw_Func_1(game: &mut Game) {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut pair: PairU16 = EnemyFunc_ACA8(
        Point16U {
            x: (*E).base.x_pos,
            y: (*E).base.y_pos,
        },
        Point16U {
            x: *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16),
            y: *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16),
        },
        game,
    );
    let mut v2: uint16 = Abs16(pair.k);
    (*E).ymw_var_08 = v2;
    if v2 as ::core::ffi::c_int - 32 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E).ymw_var_35 = 48 as uint16;
    } else if ((v2 as ::core::ffi::c_int - (*E).ymw_var_F as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if (*E).ymw_var_08 as ::core::ffi::c_int - 64 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).ymw_var_08 = 64 as uint16;
        }
        (*E).ymw_var_0A = pair.j;
        (*E).ymw_var_A = (0xa8a28c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn YappingMaw_Func_2(game: &mut Game) {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).ymw_var_B = 0 as uint16;
    (*E).ymw_var_C = 0 as uint16;
    (*E).ymw_var_D = 0 as uint16;
    (*E).ymw_var_09 = ((*E).ymw_var_08 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 =
        (64 as ::core::ffi::c_int - (*E).ymw_var_0A as ::core::ffi::c_int) as uint8 as uint16;
    (*E).ymw_var_0B = v1;
    if v1 as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E).ymw_var_2F = 0 as uint16;
    } else {
        (*E).ymw_var_2F = 1 as uint16;
    }
    YappingMaw_Func_3(game);
    let mut v2: uint16 = (2 as ::core::ffi::c_int
        * (((*E).ymw_var_0A as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint8
            as ::core::ffi::c_int
            >> 5 as ::core::ffi::c_int)) as uint16;
    (*E).ymw_var_34 = v2;
    (*E).base.current_instruction = *(game.g_rom.offset(
        (((0xa8a097 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8a097 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).ymw_var_32 = *((game.g_rom.offset(
        (((0xa8a0a7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8a0a7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16 as *mut uint8)
        .offset(
            (2 as ::core::ffi::c_int * v2 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                as isize,
        ) as *mut uint16);
    (*E).ymw_var_33 = *((game.g_rom.offset(
        (((0xa8a0a9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8a0a9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16 as *mut uint8)
        .offset(
            (2 as ::core::ffi::c_int * v2 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                as isize,
        ) as *mut uint16);
    (*E).ymw_var_A = (0xa8a445 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn YappingMaw_Func_3(game: &mut Game) {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).ymw_var_0E = YappingMaw_Func_16(0x80 as uint16, (*E).ymw_var_09, game);
    (*E).ymw_var_0F = YappingMaw_Func_17(
        0x80 as uint16,
        ((*E).ymw_var_09 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn YappingMaw_Func_4(game: &mut Game) {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut varE32: uint16 = (*E).ymw_var_00;
    (*E).ymw_var_10 = YappingMaw_Func_16(0 as uint16, varE32, game);
    (*E).ymw_var_11 = YappingMaw_Func_17(0 as uint16, varE32, game);
    (*E).ymw_var_27 = (YappingMaw_Func_16((*E).ymw_var_0B, varE32, game) as ::core::ffi::c_int
        - (*E).ymw_var_10 as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_28 = (YappingMaw_Func_17((*E).ymw_var_0B, varE32, game) as ::core::ffi::c_int
        - (*E).ymw_var_11 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn YappingMaw_Func_5(game: &mut Game) {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut varE32: uint16 = (*E).ymw_var_02;
    (*E).ymw_var_10 = YappingMaw_Func_16(0 as uint16, varE32, game);
    (*E).ymw_var_11 = YappingMaw_Func_17(0 as uint16, varE32, game);
    (*E).ymw_var_29 = (YappingMaw_Func_16((*E).ymw_var_0B, varE32, game) as ::core::ffi::c_int
        - (*E).ymw_var_10 as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_2A = (YappingMaw_Func_17((*E).ymw_var_0B, varE32, game) as ::core::ffi::c_int
        - (*E).ymw_var_11 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn YappingMaw_Func_6(game: &mut Game) {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut varE32: uint16 = (*E).ymw_var_04;
    (*E).ymw_var_10 = YappingMaw_Func_16(0 as uint16, varE32, game);
    (*E).ymw_var_11 = YappingMaw_Func_17(0 as uint16, varE32, game);
    (*E).ymw_var_2B = (YappingMaw_Func_16((*E).ymw_var_0B, varE32, game) as ::core::ffi::c_int
        - (*E).ymw_var_10 as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_2C = (YappingMaw_Func_17((*E).ymw_var_0B, varE32, game) as ::core::ffi::c_int
        - (*E).ymw_var_11 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn YappingMaw_Func_7(game: &mut Game) {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut varE32: uint16 = (*E).ymw_var_06;
    (*E).ymw_var_10 = YappingMaw_Func_16(0 as uint16, varE32, game);
    (*E).ymw_var_11 = YappingMaw_Func_17(0 as uint16, varE32, game);
    (*E).ymw_var_2D = (YappingMaw_Func_16((*E).ymw_var_0B, varE32, game) as ::core::ffi::c_int
        - (*E).ymw_var_10 as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_2E = (YappingMaw_Func_17((*E).ymw_var_0B, varE32, game) as ::core::ffi::c_int
        - (*E).ymw_var_11 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn YappingMaw_Func_8(game: &mut Game) {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut r20: uint16 =
        ((*E).ymw_var_B as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16;
    if (*E).ymw_var_2F != 0 {
        (*E).ymw_var_21 = (128 as ::core::ffi::c_int + r20 as ::core::ffi::c_int) as uint16;
        (*E).ymw_var_22 = (128 as ::core::ffi::c_int
            + r20 as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
            as uint16;
        (*E).ymw_var_23 = (128 as ::core::ffi::c_int
            + r20 as ::core::ffi::c_int * 3 as ::core::ffi::c_int)
            as uint16;
        (*E).ymw_var_24 = (128 as ::core::ffi::c_int
            + r20 as ::core::ffi::c_int * 4 as ::core::ffi::c_int)
            as uint16;
    } else {
        (*E).ymw_var_21 = (128 as ::core::ffi::c_int - r20 as ::core::ffi::c_int) as uint16;
        (*E).ymw_var_22 = (128 as ::core::ffi::c_int
            - r20 as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
            as uint16;
        (*E).ymw_var_23 = (128 as ::core::ffi::c_int
            - r20 as ::core::ffi::c_int * 3 as ::core::ffi::c_int)
            as uint16;
        (*E).ymw_var_24 = (128 as ::core::ffi::c_int
            - r20 as ::core::ffi::c_int * 4 as ::core::ffi::c_int)
            as uint16;
    }
    let mut varE32: uint16 = (*E).ymw_var_09;
    (*E).ymw_var_00 = (YappingMaw_Func_16((*E).ymw_var_21, varE32, game) as ::core::ffi::c_int
        - (*E).ymw_var_0E as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_02 = (YappingMaw_Func_16((*E).ymw_var_22, varE32, game) as ::core::ffi::c_int
        - (*E).ymw_var_0E as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_04 = (YappingMaw_Func_16((*E).ymw_var_23, varE32, game) as ::core::ffi::c_int
        - (*E).ymw_var_0E as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_06 = (YappingMaw_Func_16((*E).ymw_var_24, varE32, game) as ::core::ffi::c_int
        - (*E).ymw_var_0E as ::core::ffi::c_int) as uint16;
    varE32 = ((*E).ymw_var_09 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_01 = (YappingMaw_Func_17((*E).ymw_var_21, varE32, game) as ::core::ffi::c_int
        - (*E).ymw_var_0F as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_03 = (YappingMaw_Func_17((*E).ymw_var_22, varE32, game) as ::core::ffi::c_int
        - (*E).ymw_var_0F as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_05 = (YappingMaw_Func_17((*E).ymw_var_23, varE32, game) as ::core::ffi::c_int
        - (*E).ymw_var_0F as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_07 = (YappingMaw_Func_17((*E).ymw_var_24, varE32, game) as ::core::ffi::c_int
        - (*E).ymw_var_0F as ::core::ffi::c_int) as uint16;
    YappingMaw_Func_4(game);
    YappingMaw_Func_5(game);
    YappingMaw_Func_6(game);
    YappingMaw_Func_7(game);
    (*E).ymw_var_00 =
        ((*E).ymw_var_00 as ::core::ffi::c_int + (*E).ymw_var_27 as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_01 =
        ((*E).ymw_var_01 as ::core::ffi::c_int + (*E).ymw_var_28 as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_02 =
        ((*E).ymw_var_02 as ::core::ffi::c_int + (*E).ymw_var_29 as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_03 =
        ((*E).ymw_var_03 as ::core::ffi::c_int + (*E).ymw_var_2A as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_04 =
        ((*E).ymw_var_04 as ::core::ffi::c_int + (*E).ymw_var_2B as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_05 =
        ((*E).ymw_var_05 as ::core::ffi::c_int + (*E).ymw_var_2C as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_06 =
        ((*E).ymw_var_06 as ::core::ffi::c_int + (*E).ymw_var_2D as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_07 =
        ((*E).ymw_var_07 as ::core::ffi::c_int + (*E).ymw_var_2E as ::core::ffi::c_int) as uint16;
    (*E).base.x_pos =
        ((*E).ymw_var_06 as ::core::ffi::c_int + (*E).ymw_var_0C as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos =
        ((*E).ymw_var_07 as ::core::ffi::c_int + (*E).ymw_var_0D as ::core::ffi::c_int) as uint16;
    YappingMaw_Func_9(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    if (*E).ymw_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        if (*E).ymw_var_B as ::core::ffi::c_int - 128 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).ymw_var_B = 128 as uint16;
            (*E).ymw_var_C = 0 as uint16;
            (*E).ymw_var_D =
                ((*E).ymw_var_D as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
        }
    } else {
        (*E).ymw_var_A = (0xa8a68a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).ymw_var_35 = 48 as uint16;
        if (*E).ymw_var_34 as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
            (*E).base.current_instruction =
                addr_kYappingMaw_Ilist_A01F as ::core::ffi::c_int as uint16;
        } else if (*E).ymw_var_34 as ::core::ffi::c_int == 12 as ::core::ffi::c_int {
            (*E).base.current_instruction =
                addr_kYappingMaw_Ilist_A03D as ::core::ffi::c_int as uint16;
        } else {
            (*E).base.current_instruction = addr_off_A8A025 as ::core::ffi::c_int as uint16;
        }
        if (*E).ymw_parameter_2 == 0 {
            if (*E).ymw_var_34 as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
                (*E).base.current_instruction =
                    addr_kYappingMaw_Ilist_A05B as ::core::ffi::c_int as uint16;
            } else {
                if (*E).ymw_var_34 as ::core::ffi::c_int != 12 as ::core::ffi::c_int {
                    (*E).base.current_instruction = addr_off_A8A061 as ::core::ffi::c_int as uint16;
                    return;
                }
                (*E).base.current_instruction =
                    addr_kYappingMaw_Ilist_A079 as ::core::ffi::c_int as uint16;
            }
        } else {
            return;
        }
    }
    if (*E).ymw_var_30 != 0 {
        YappingMaw_Func_10(game);
    }
}
pub unsafe fn YappingMaw_Func_9(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(k as ::core::ffi::c_int, game);
    let mut v2: ::core::ffi::c_int =
        (*E).ymw_var_D as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).ymw_var_B,
        &raw mut (*E).ymw_var_C,
        (*(game.g_rom.offset(
            (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(game.g_rom.offset(
                (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v2 as isize) as uint32,
    );
    (*E).ymw_var_D = ((*E).ymw_var_D as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn YappingMaw_Func_10(game: &mut Game) {
    CallSomeSamusCode(3 as uint16, game);
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) =
        ((*E).ymw_var_32 as ::core::ffi::c_int + (*E).base.x_pos as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) =
        ((*E).ymw_var_33 as ::core::ffi::c_int + (*E).base.y_pos as ::core::ffi::c_int) as uint16;
    EnemyFunc_B7A1(game);
}
pub unsafe fn YappingMaw_Func_11(game: &mut Game) {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).ymw_var_30 != 0 {
        YappingMaw_Func_10(game);
    }
    (*E).ymw_var_E = (*E).ymw_var_E.wrapping_sub(1);
    if (*E).ymw_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa60) as *mut uint16) as ::core::ffi::c_int
            != 0x90e91d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa60) as *mut uint16) =
            (0x90e913 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).ymw_var_30 = 0 as uint16;
        (*E).ymw_var_35 = 48 as uint16;
        (*E).ymw_var_E = 64 as uint16;
        (*E).ymw_var_A = (0xa8a235 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn YappingMaw_Func_12(game: &mut Game) {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v1: ::core::ffi::c_int =
        (*E).ymw_var_40 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (*E).ymw_var_0C;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (*E).ymw_var_0D;
}
pub unsafe fn YappingMaw_Func_13(game: &mut Game) {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v1: ::core::ffi::c_int =
        (*E).ymw_var_41 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        ((*E).ymw_var_00 as ::core::ffi::c_int + (*E).ymw_var_0C as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        ((*E).ymw_var_01 as ::core::ffi::c_int + (*E).ymw_var_0D as ::core::ffi::c_int) as uint16;
}
pub unsafe fn YappingMaw_Func_14(game: &mut Game) {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v1: ::core::ffi::c_int =
        (*E).ymw_var_42 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        ((*E).ymw_var_02 as ::core::ffi::c_int + (*E).ymw_var_0C as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        ((*E).ymw_var_03 as ::core::ffi::c_int + (*E).ymw_var_0D as ::core::ffi::c_int) as uint16;
}
pub unsafe fn YappingMaw_Func_15(game: &mut Game) {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v1: ::core::ffi::c_int =
        (*E).ymw_var_43 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        ((*E).ymw_var_04 as ::core::ffi::c_int + (*E).ymw_var_0C as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        ((*E).ymw_var_05 as ::core::ffi::c_int + (*E).ymw_var_0D as ::core::ffi::c_int) as uint16;
}
pub unsafe fn YappingMaw_Func_16(mut a: uint16, mut varE32: uint16, game: &mut Game) -> uint16 {
    return YappingMaw_Func_17(
        (a as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as uint16,
        varE32,
        game,
    );
}
pub unsafe fn YappingMaw_Func_17(mut a: uint16, mut varE32: uint16, game: &mut Game) -> uint16 {
    let mut v1: int16 = 0;
    let mut r20: uint16 = 0 as uint16;
    let mut r22: uint16 = 0 as uint16;
    v1 = *(game.g_rom.offset(
        (((0xa0b1c3 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa0b1c3 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(-(a as ::core::ffi::c_int) as uint8 as isize) as int16;
    if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        v1 = -(v1 as ::core::ffi::c_int) as int16;
        r20 = r20.wrapping_add(1);
    }
    WriteReg(
        WRMPYA as ::core::ffi::c_int as uint16,
        *(&raw mut v1 as *mut uint8).offset(
            (::core::mem::size_of::<int16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ),
        game,
    );
    if varE32 as uint8 != 0 {
        let mut prod: uint16 = Mult8x8(
            *(&raw mut v1 as *mut uint8).offset(
                (::core::mem::size_of::<int16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ),
            varE32 as uint8,
        );
        r22 = (2 as ::core::ffi::c_int
            * ((prod as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int)) as uint16;
        if r20 != 0 {
            r22 = (2 as ::core::ffi::c_int
                * (-(prod as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int)
                | 0xff00 as ::core::ffi::c_int) as uint16;
        }
    }
    return r22;
}
pub unsafe fn YappingMaw_Touch(game: &mut Game) {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).ymw_var_35 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && (*E).ymw_var_30 == 0
    {
        (*E).ymw_var_35 = 0 as uint16;
        (*E).ymw_var_30 = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa60) as *mut uint16) =
            (0x90e90e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn YappingMaw_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).base.health != 0 {
        if (*E).base.frozen_timer != 0 {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xa60) as *mut uint16)
                as ::core::ffi::c_int
                != 0x90e91d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa60) as *mut uint16) =
                    (0x90e913 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            }
            (*E).ymw_var_30 = 0 as uint16;
        }
    } else {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16 as *mut uint8)
            .offset((*E).ymw_var_40 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16 as *mut uint8)
            .offset((*E).ymw_var_41 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16 as *mut uint8)
            .offset((*E).ymw_var_42 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16 as *mut uint8)
            .offset((*E).ymw_var_43 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16)
            .offset(((*E).ymw_var_46 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            0 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa60) as *mut uint16) as ::core::ffi::c_int
            != 0x90e91d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa60) as *mut uint16) =
                (0x90e913 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        (*Get_YappingMaw(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .ymw_var_30 = 0 as uint16;
    };
}
pub unsafe fn YappingMaw_Frozen(game: &mut Game) {
    Enemy_NormalFrozenAI_A8(game);
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    YappingMaw_Func_18((*E).ymw_var_40, game);
    YappingMaw_Func_18((*E).ymw_var_41, game);
    YappingMaw_Func_18((*E).ymw_var_42, game);
    YappingMaw_Func_18((*E).ymw_var_43, game);
    YappingMaw_Func_19(game);
}
pub unsafe fn YappingMaw_Func_18(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        ((*E).ymw_var_45 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0xf1ff as ::core::ffi::c_int) as uint16;
    if (*E).base.frozen_timer != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0xf1ff as ::core::ffi::c_int
                | 0xc00 as ::core::ffi::c_int) as uint16;
        let mut frozen_timer: uint16 = (*E).base.frozen_timer;
        if frozen_timer as ::core::ffi::c_int - 90 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if frozen_timer as ::core::ffi::c_int & 2 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16)
                    .offset(v1 as isize) = ((*E).ymw_var_45 as ::core::ffi::c_int
                    | *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int
                        & 0xf1ff as ::core::ffi::c_int)
                    as uint16;
            }
        }
    }
}
pub unsafe fn YappingMaw_Func_19(game: &mut Game) {
    let mut E: *mut Enemy_YappingMaw = Get_YappingMaw(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut r18: uint16 = (*E).ymw_var_45;
    let mut r20: uint16 = (*E).ymw_var_46;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf078) as *mut uint16)
        .offset((r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (r18 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0xf078) as *mut uint16)
                .offset((r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                & 0xf1ff as ::core::ffi::c_int) as uint16;
    if (*E).base.frozen_timer != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xf078) as *mut uint16)
            .offset((r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0xf078) as *mut uint16)
                .offset((r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                & 0xf1ff as ::core::ffi::c_int
                | 0xc00 as ::core::ffi::c_int) as uint16;
        let mut frozen_timer: uint16 = (*E).base.frozen_timer;
        if frozen_timer as ::core::ffi::c_int - 90 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if frozen_timer as ::core::ffi::c_int & 2 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xf078) as *mut uint16)
                    .offset((r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                    (r18 as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8).offset(0xf078) as *mut uint16)
                            .offset((r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            & 0xf1ff as ::core::ffi::c_int) as uint16;
            }
        }
    }
}
pub unsafe fn Kago_Init(game: &mut Game) {
    let mut E: *mut Enemy_Kago = Get_Kago(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_DisableSamusColl as ::core::ffi::c_int) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).kago_var_E = 0 as uint16;
    (*E).base.current_instruction = addr_kKago_Ilist_AB1E as ::core::ffi::c_int as uint16;
    (*E).kago_var_A = (0xa8ab7b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).kago_var_F = 0 as uint16;
    (*E).kago_var_04 = (*E).kago_parameter_1;
}
pub unsafe fn CallKagoFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        11053947 => {
            Kago_Func_1(k, game);
            return;
        }
        11053953 => return,
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn Kago_Main(game: &mut Game) {
    let mut E: *mut Enemy_Kago = Get_Kago(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallKagoFunc(
        ((*E).kago_var_A as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn Kago_Func_1(mut k: uint16, game: &mut Game) {
    (*Get_Kago(k as ::core::ffi::c_int, game)).kago_var_A =
        (0xa8ab81 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Kago_Shot(game: &mut Game) {
    let mut v2: int16 = 0;
    NormalEnemyShotAi(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 2 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 16 as uint16;
    let mut E: *mut Enemy_Kago = Get_Kago(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).kago_var_B == 0 {
        (*E).kago_var_B = 1 as uint16;
        (*E).base.current_instruction = addr_kKago_Ilist_AB32 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
    }
    v2 = ((*E).kago_var_04 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
    (*E).kago_var_04 = v2 as uint16;
    if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        EnemyDeathAnimation(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            4 as uint16,
            game,
        );
        (*E).kago_var_F = 1 as uint16;
    }
    SpawnEprojWithGfx(
        (*E).base.y_pos,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_loc_A8D02E as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn NorfairLavaMan_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if CheckIfEnemyIsOnScreen(game) == 0 {
        QueueSfx2_Max6(*jp.offset(0), game);
    }
    return jp.offset(1);
}
pub unsafe fn NorfairLavaMan_Instr_8(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    return jp;
}
pub unsafe fn NorfairLavaMan_Instr_14(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    return jp;
}
pub unsafe fn NorfairLavaMan_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nlmn_var_01 = 1 as uint16;
    return jp;
}
pub unsafe fn NorfairLavaMan_Instr_7(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nlmn_var_01 = 0 as uint16;
    return jp;
}
pub unsafe fn NorfairLavaMan_Instr_10(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    let mut E1: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E1).base.y_pos = (*E1).base.y_pos.wrapping_sub(1);
    return jp;
}
pub unsafe fn NorfairLavaMan_Instr_12(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    let mut E1: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E1).base.y_pos = (*E1).base.y_pos.wrapping_add(1);
    return jp;
}
pub unsafe fn NorfairLavaMan_Instr_9(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v4: uint16 =
        ((*E).nlmn_var_03 as ::core::ffi::c_int + 24 as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = v4;
    let mut E1: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E1).base.y_pos = v4;
    (*E1).base.properties =
        ((*E1).base.properties as ::core::ffi::c_int & !(0x100 as ::core::ffi::c_int)) as uint16;
    let mut E2: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 128 as ::core::ffi::c_int,
        game,
    );
    (*E2).base.properties =
        ((*E2).base.properties as ::core::ffi::c_int & !(0x100 as ::core::ffi::c_int)) as uint16;
    return jp;
}
pub unsafe fn NorfairLavaMan_Instr_11(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut nlmn_var_03: uint16 = (*E).nlmn_var_03;
    (*E).base.y_pos = nlmn_var_03;
    (*Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    ))
    .base
    .y_pos = nlmn_var_03;
    return jp;
}
pub unsafe fn NorfairLavaMan_Instr_13(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v4: uint16 =
        ((*E).nlmn_var_03 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = v4;
    let mut E1: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E1).base.y_pos = v4;
    (*E1).base.properties =
        ((*E1).base.properties as ::core::ffi::c_int | 0x100 as ::core::ffi::c_int) as uint16;
    let mut E2: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 128 as ::core::ffi::c_int,
        game,
    );
    (*E2).base.properties =
        ((*E2).base.properties as ::core::ffi::c_int | 0x100 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn NorfairLavaMan_Instr_5(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    SpawnEprojWithGfx(
        (*E).nlmn_var_B,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kEproj_LavaThrownByLavaman as ::core::ffi::c_int as uint16,
        game,
    );
    return jp;
}
pub unsafe fn NorfairLavaMan_Instr_15(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.x_pos = ((*E).nlmn_var_12 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E).nlmn_var_13 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn NorfairLavaMan_Instr_4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.x_pos = ((*E).nlmn_var_12 as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E).nlmn_var_13 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn NorfairLavaMan_Instr_16(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.x_pos = ((*E).nlmn_var_12 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E).nlmn_var_13 as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn NorfairLavaMan_Instr_6(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.x_pos = ((*E).nlmn_var_12 as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E).nlmn_var_13 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn sub_A8AF32(game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.x_pos = (*E).nlmn_var_12;
    (*E).base.y_pos = (*E).nlmn_var_13;
}
pub unsafe fn NorfairLavaMan_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nlmn_var_04 = 256 as uint16;
    return jp;
}
static g_off_A8AF4F: [Option<Func_V>; 3] = unsafe {
    [
        Some(NorfairLavaMan_Func_1 as unsafe fn(&mut Game) -> ()),
        Some(NorfairLavaMan_Func_2 as unsafe fn(&mut Game) -> ()),
        Some(NorfairLavaMan_Func_3 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn NorfairLavaMan_Init(game: &mut Game) {
    let mut v0: uint16 = (2 as ::core::ffi::c_int
        * (*Get_NorfairLavaMan(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .nlmn_parameter_1 as ::core::ffi::c_int) as uint16;
    g_off_A8AF4F[(v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(game);
    NorfairLavaMan_Func_4(game);
    NorfairLavaMan_Func_5(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn NorfairLavaMan_Func_1(game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).nlmn_var_C = 0 as uint16;
    (*E).nlmn_var_00 = 0 as uint16;
    (*E).nlmn_var_02 = 0 as uint16;
    (*E).nlmn_var_03 = (*E).base.y_pos;
    if GetSamusEnemyDelta_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).nlmn_var_00 = 1 as uint16;
    }
    (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AC9C as ::core::ffi::c_int as uint16;
    if (*E).nlmn_var_00 == 0 {
        (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AD3C as ::core::ffi::c_int as uint16;
    }
    NorfairLavaMan_Func_20(game);
    (*E).nlmn_var_F = (0xa8b11a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn NorfairLavaMan_Func_2(game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).nlmn_var_03 = (*E).base.y_pos;
    (*E).nlmn_var_C = 0 as uint16;
    (*E).nlmn_var_0C = 0 as uint16;
    (*E).nlmn_var_0D = 1 as uint16;
    (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_ADDC as ::core::ffi::c_int as uint16;
    NorfairLavaMan_Func_20(game);
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    (*E).nlmn_var_F = (0xa8b1dd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
}
pub unsafe fn NorfairLavaMan_Func_3(game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).nlmn_var_03 = (*E).base.y_pos;
    (*E).nlmn_var_11 = (*E).base.x_pos;
    (*E).nlmn_var_C = 0 as uint16;
    (*E).nlmn_var_04 = 0 as uint16;
    (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AE0C as ::core::ffi::c_int as uint16;
    NorfairLavaMan_Func_20(game);
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    (*E).nlmn_var_F = (0xa8b30d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
}
pub unsafe fn NorfairLavaMan_Func_4(game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v1: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * *(&raw mut (*E).nlmn_parameter_2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).nlmn_var_08 = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize);
    (*E).nlmn_var_07 = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v1 + 1 as ::core::ffi::c_int) as isize);
    (*E).nlmn_var_0A = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v1 + 2 as ::core::ffi::c_int) as isize);
    (*E).nlmn_var_09 = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v1 + 3 as ::core::ffi::c_int) as isize);
}
pub unsafe fn NorfairLavaMan_Func_5(mut k: uint16, game: &mut Game) {
    (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).addr =
        (0xa8b0b2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP;
    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).bank
        as *mut uint16) = 168 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(0) =
        ((((*Get_NorfairLavaMan(k as ::core::ffi::c_int, game))
            .base
            .palette_index as ::core::ffi::c_int
            & 0xe00 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int)
            + 256 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(2) = 8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(1) = 0 as uint16;
}
pub unsafe fn NorfairLavaMan_Func_6(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x797) as *mut uint16) == 0 && {
        let ref mut fresh10 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(2);
        *fresh10 = (*fresh10).wrapping_sub(1);
        *fresh10 == 0
    } {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(2) =
            8 as uint16;
        let ref mut fresh11 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(1);
        *fresh11 = (*fresh11).wrapping_add(1);
        let mut v0: ::core::ffi::c_int = (32 as ::core::ffi::c_int
            * (*((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(1)
                as ::core::ffi::c_int
                & 3 as ::core::ffi::c_int)) as uint16
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let mut r18: uint16 = *(game.g_rom.offset(
            (((0xa8ac1c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa8ac1c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v0 + 9 as ::core::ffi::c_int) as isize);
        let mut r20: uint16 = *(game.g_rom.offset(
            (((0xa8ac1c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa8ac1c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v0 + 10 as ::core::ffi::c_int) as isize);
        let mut r22: uint16 = *(game.g_rom.offset(
            (((0xa8ac1c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa8ac1c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v0 + 11 as ::core::ffi::c_int) as isize);
        let mut r24: uint16 = *(game.g_rom.offset(
            (((0xa8ac1c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa8ac1c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v0 + 12 as ::core::ffi::c_int) as isize);
        let mut v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1794)
            as *mut uint16)
            .offset(0) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v1 + 9 as ::core::ffi::c_int) as isize) = r18;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v1 + 10 as ::core::ffi::c_int) as isize) = r20;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v1 + 11 as ::core::ffi::c_int) as isize) = r22;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v1 + 12 as ::core::ffi::c_int) as isize) = r24;
    }
}
pub unsafe fn NorfairLavaMan_Main(game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).nlmn_var_04 = (*E).nlmn_var_04.wrapping_sub(1);
    CallEnemyPreInstr(
        ((*E).nlmn_var_F as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn NorfairLavaMan_Func_7(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).nlmn_var_00 = 0 as uint16;
    (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AC9C as ::core::ffi::c_int as uint16;
    if GetSamusEnemyDelta_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AD3C as ::core::ffi::c_int as uint16;
        (*E).nlmn_var_00 = 1 as uint16;
    }
    NorfairLavaMan_Func_20(game);
    let mut E2: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 128 as ::core::ffi::c_int,
        game,
    );
    if (*E2).nlmn_var_04 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E2).nlmn_var_04 = 0 as uint16;
        if IsSamusWithinEnemy_X(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            *(&raw mut (*E).nlmn_parameter_2 as *mut uint8).offset(0) as uint16,
            game,
        ) != 0
        {
            (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_ACDE as ::core::ffi::c_int as uint16;
            if (*E).nlmn_var_00 != 0 {
                (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AD7E as ::core::ffi::c_int as uint16;
            }
            NorfairLavaMan_Func_20(game);
            (*E).nlmn_var_F =
                (0xa8b175 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn NorfairLavaMan_Func_8(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).nlmn_var_01 == 0 {
        (*E).nlmn_var_02 = 1 as uint16;
        (*Get_NorfairLavaMan(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
                + 64 as ::core::ffi::c_int,
            game,
        ))
        .nlmn_var_0D = 0 as uint16;
        (*E).nlmn_var_F = (0xa8b193 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn sub_A8B193(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).nlmn_var_02 == 0 {
        (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AD0C as ::core::ffi::c_int as uint16;
        if GetSamusEnemyDelta_X(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        ) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_ADAC as ::core::ffi::c_int as uint16;
        }
        NorfairLavaMan_Func_20(game);
        (*E).nlmn_var_F = (0xa8b1b8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn NorfairLavaMan_Func_9(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).nlmn_var_01 == 0 {
        (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AC9C as ::core::ffi::c_int as uint16;
        if GetSamusEnemyDelta_X(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        ) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AD3C as ::core::ffi::c_int as uint16;
        }
        NorfairLavaMan_Func_20(game);
        (*E).nlmn_var_F = (0xa8b11a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn NorfairLavaMan_Func_10(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).nlmn_var_0D == 0 {
        (*E).nlmn_var_F = (0xa8b204 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).nlmn_var_E = 0 as uint16;
        (*E).nlmn_var_0D = 0 as uint16;
        (*E).nlmn_var_B = 2 as uint16;
        (*E).base.y_pos = (*E).nlmn_var_03;
    }
}
pub unsafe fn NorfairLavaMan_Func_11(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v3: *mut EnemySpawnData = gEnemySpawnData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v: uint32 = ((*v3.offset(31)).ypos2 as ::core::ffi::c_int
        + (*E).base.y_subpos as ::core::ffi::c_int) as uint32;
    (*E).base.y_subpos = v as uint16;
    (*E).base.y_pos = ((*E).base.y_pos as uint32).wrapping_add(
        ((*v3.offset(31)).field_14 as uint32).wrapping_add(v >> 16 as ::core::ffi::c_int),
    ) as uint16 as uint16;
    (*E).nlmn_var_E = ((*E).nlmn_var_E as uint32).wrapping_add(
        ((*v3.offset(31)).field_14 as uint32).wrapping_add(v >> 16 as ::core::ffi::c_int),
    ) as uint16 as uint16;
    NorfairLavaMan_Func_12(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn NorfairLavaMan_Func_12(mut k: uint16, game: &mut Game) {
    let mut current_block: u64;
    let mut v3: ::core::ffi::c_int = 0;
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(k as ::core::ffi::c_int, game);
    if -((*E).nlmn_var_E as ::core::ffi::c_int) - 108 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        printf(
            b"Y undefined %d!\n\0" as *const u8 as *const ::core::ffi::c_char,
            -((*E).nlmn_var_E as int16 as ::core::ffi::c_int),
        );
    }
    let mut nlmn_var_B: uint16 = (*E).nlmn_var_B;
    if -((*E).nlmn_var_E as ::core::ffi::c_int) - 108 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        || {
            v3 = nlmn_var_B as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            (((*E).base.y_pos as ::core::ffi::c_int
                - *(game.g_rom.offset(
                    (((0xa8af79 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa8af79 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
        }
    {
        (*E).nlmn_var_F = (0xa8b291 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).nlmn_var_0C = 1 as uint16;
        if ((-((*E).nlmn_var_E as ::core::ffi::c_int)
            - *(game.g_rom.offset(
                (((0xa8af55 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa8af55 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((nlmn_var_B as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            return;
        }
        current_block = 9609259077933178539;
    } else if (-((*E).nlmn_var_E as ::core::ffi::c_int)
        - *(game.g_rom.offset(
            (((0xa8af55 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa8af55 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        current_block = 9609259077933178539;
    } else {
        current_block = 7746791466490516765;
    }
    match current_block {
        9609259077933178539 => {
            (*E).nlmn_var_B = (*E).nlmn_var_B.wrapping_add(1);
            (*E).nlmn_var_B = (*E).nlmn_var_B.wrapping_add(1);
            (*E).base.y_pos =
                ((*E).base.y_pos as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
            (*E).nlmn_var_D = *(game.g_rom.offset(
                (((0xa8af67 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa8af67 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    ((*E).nlmn_var_B as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                );
            NorfairLavaMan_Func_20(game);
        }
        _ => {}
    };
}
pub unsafe fn sub_A8B291(mut k: uint16) {}
pub unsafe fn NorfairLavaMan_Func_13(game: &mut Game) {
    let mut v2: *mut EnemySpawnData = ::core::ptr::null_mut::<EnemySpawnData>();
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    v2 = gEnemySpawnData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v4: bool = u16::overflowing_add((*v2.offset(31)).field_E, (*E).base.y_subpos).1;
    (*E).base.y_subpos = ((*v2.offset(31)).field_E as ::core::ffi::c_int
        + (*E).base.y_subpos as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        + ((*v2.offset(31)).xpos2 as ::core::ffi::c_int + v4 as ::core::ffi::c_int))
        as uint16;
    (*E).nlmn_var_E = ((*E).nlmn_var_E as ::core::ffi::c_int
        + ((*v2.offset(31)).xpos2 as ::core::ffi::c_int + v4 as ::core::ffi::c_int))
        as uint16;
    NorfairLavaMan_Func_14(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn NorfairLavaMan_Func_14(mut k: uint16, game: &mut Game) {
    let mut nlmn_var_E: int16 = 0;
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(k as ::core::ffi::c_int, game);
    nlmn_var_E = (*E).nlmn_var_E as int16;
    if nlmn_var_E as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        (*E).nlmn_var_0D = 1 as uint16;
        (*gEnemySpawnData(k as ::core::ffi::c_int, game).offset(31)).field_4 = 0 as uint16;
        (*E).nlmn_var_F = (0xa8b1dd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else if ((-(nlmn_var_E as ::core::ffi::c_int)
        - *(game.g_rom.offset(
            (((0xa8af55 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa8af55 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (((*E).nlmn_var_B as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            ) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        (*E).nlmn_var_B =
            ((*E).nlmn_var_B as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        (*E).base.y_pos =
            ((*E).base.y_pos as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
        (*E).nlmn_var_D = *(game.g_rom.offset(
            (((0xa8af67 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa8af67 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(((*E).nlmn_var_B as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        NorfairLavaMan_Func_20(game);
    }
}
pub unsafe fn NorfairLavaMan_Func_15(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16).offset(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int)
            + 87 as ::core::ffi::c_int) as isize,
    ) as ::core::ffi::c_int
        == 0xa8b175 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        (*Get_NorfairLavaMan(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .nlmn_var_F = (0xa8b356 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn NorfairLavaMan_Func_16(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).nlmn_var_01 == 0 {
        (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AE0C as ::core::ffi::c_int as uint16;
        NorfairLavaMan_Func_20(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16).offset(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int)
                + 5 as ::core::ffi::c_int) as isize,
        ) = (0xa8b295 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).nlmn_var_F = (0xa8b3a7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*gEnemySpawnData(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        )
        .offset(31))
        .field_18 = 0 as uint16;
        (*E).base.x_pos = (*E).nlmn_var_12;
        (*E).base.y_pos = (*E).nlmn_var_13;
    }
    NorfairLavaMan_Func_19(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn NorfairLavaMan_Func_17(mut k: uint16, game: &mut Game) {
    if (*gEnemySpawnData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    )
    .offset(31))
    .field_18
        != 0
    {
        let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_ACB0 as ::core::ffi::c_int as uint16;
        (*E).nlmn_var_B = 0 as uint16;
        if GetSamusEnemyDelta_X(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        ) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AD50 as ::core::ffi::c_int as uint16;
            (*E).nlmn_var_B = 1 as uint16;
        }
        NorfairLavaMan_Func_20(game);
        (*E).nlmn_var_F = (0xa8b31f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).nlmn_var_12 = (*E).base.x_pos;
        (*E).nlmn_var_13 = (*E).base.y_pos;
    } else {
        (*Get_NorfairLavaMan(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .base
        .y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16).offset(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int)
                + 93 as ::core::ffi::c_int) as isize,
        ) as ::core::ffi::c_int
            - *(game.g_rom.offset(
                (((0xa8af79 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa8af79 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16).offset(
                        ((*((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                            as ::core::ffi::c_int
                            >> 1 as ::core::ffi::c_int)
                            + 1 as ::core::ffi::c_int) as isize,
                    ) as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int) as uint16;
    }
    NorfairLavaMan_Func_19(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn NorfairLavaMan_Func_18(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xe54)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 87 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
        == 0xa8b11a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        (*Get_NorfairLavaMan(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .nlmn_var_F = (0xa8b30d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*Get_NorfairLavaMan(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .base
        .y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 93 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            - *(game.g_rom.offset(
                (((0xa8af79 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa8af79 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
                        .offset((v1 + 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int) as uint16;
    }
    NorfairLavaMan_Func_19(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn NorfairLavaMan_Func_19(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 61 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        - (*Get_NorfairLavaMan(k as ::core::ffi::c_int, game))
            .base
            .y_pos as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 64 as ::core::ffi::c_int) as isize) = v2;
    if v2 as ::core::ffi::c_int - 8 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 64 as ::core::ffi::c_int) as isize) = 8 as uint16;
    }
}
pub unsafe fn NorfairLavaMan_Func_20(game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut nlmn_var_D: uint16 = (*E).nlmn_var_D;
    if nlmn_var_D as ::core::ffi::c_int != (*E).nlmn_var_C as ::core::ffi::c_int {
        (*E).base.current_instruction = nlmn_var_D;
        (*E).nlmn_var_C = nlmn_var_D;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn NorfairLavaMan_Powerbomb(game: &mut Game) {
    Enemy_NormalPowerBombAI_A8(game);
    NorfairLavaMan_Common(game);
}
pub unsafe fn NorfairLavaMan_Touch(game: &mut Game) {
    Enemy_NormalTouchAI_A8(game);
    NorfairLavaMan_Common(game);
}
pub unsafe fn NorfairLavaMan_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    NorfairLavaMan_Common(game);
}
pub unsafe fn NorfairLavaMan_Common(game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut E1: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    let mut E2: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 128 as ::core::ffi::c_int,
        game,
    );
    if (*E).base.health == 0 {
        (*E1).base.properties =
            ((*E1).base.properties as ::core::ffi::c_int | 0x200 as ::core::ffi::c_int) as uint16;
        (*E2).base.properties =
            ((*E2).base.properties as ::core::ffi::c_int | 0x200 as ::core::ffi::c_int) as uint16;
    }
    let mut frozen_timer: uint16 = (*E).base.frozen_timer;
    if frozen_timer != 0 {
        (*E1).base.frozen_timer = frozen_timer;
        (*E1).base.ai_handler_bits =
            ((*E1).base.ai_handler_bits as ::core::ffi::c_int | 4 as ::core::ffi::c_int) as uint16;
        (*E2).base.frozen_timer = frozen_timer;
        (*E2).base.ai_handler_bits =
            ((*E2).base.ai_handler_bits as ::core::ffi::c_int | 4 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Beetom_Instr_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return jp;
}
pub unsafe fn Beetom_Func_1(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction = (*E).beetom_var_00;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn Beetom_Init(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).beetom_var_D = 0 as uint16;
    (*E).beetom_var_C = 0 as uint16;
    (*E).beetom_var_05 = 0 as uint16;
    (*E).beetom_var_08 = 0 as uint16;
    (*E).beetom_var_A = 0 as uint16;
    (*E).beetom_var_E = 64 as uint16;
    (*E).beetom_var_F = *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) = 23 as uint16;
    (*E).beetom_var_02 = Beetom_Func_2(12288 as uint16, 4 as uint16, game);
    (*E).beetom_var_03 = Beetom_Func_2(0x4000 as uint16, 5 as uint16, game);
    (*E).beetom_var_04 = Beetom_Func_2(12288 as uint16, 3 as uint16, game);
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B6F2 as ::core::ffi::c_int as uint16;
    if GetSamusEnemyDelta_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).beetom_var_00 = addr_kBeetom_Ilist_B696 as ::core::ffi::c_int as uint16;
    }
    Beetom_Func_1(game);
    (*E).beetom_var_C = (0xa8b814 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Beetom_Func_2(mut r22: uint16, mut r24: uint16, game: &mut Game) -> uint16 {
    let mut r18: uint16 = 0 as uint16;
    let mut r20: uint16 = 0 as uint16;
    loop {
        r18 = (r18 as ::core::ffi::c_int + r24 as ::core::ffi::c_int) as uint16;
        r20 = (r20 as ::core::ffi::c_int
            + *((game.g_rom.offset(
                (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16 as *mut uint8)
                .offset((8 as ::core::ffi::c_int * r18 as ::core::ffi::c_int) as isize)
                .offset(1) as *mut uint16) as ::core::ffi::c_int) as uint16;
        if !(r20 as ::core::ffi::c_int - r22 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0)
        {
            break;
        }
    }
    return r18;
}
pub unsafe fn Beetom_Func_3(game: &mut Game) {
    if IsSamusWithinEnemy_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0x60 as uint16,
        game,
    ) != 0
    {
        (*Get_Beetom(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .beetom_var_C = (0xa8b90f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*Get_Beetom(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .beetom_var_C = (0xa8b82f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Beetom_Func_4(game: &mut Game) {
    NextRandom(game);
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).beetom_var_C = *(game.g_rom.offset(
        (((0xa8b74e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8b74e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                as ::core::ffi::c_int
                & 7 as ::core::ffi::c_int) as isize,
        );
    (*E).beetom_var_09 = (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Beetom_Func_5(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).beetom_var_D = 32 as uint16;
    (*E).beetom_var_C = (0xa8b9b2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Beetom_Func_6(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).beetom_var_C = (0xa8b9c1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B696 as ::core::ffi::c_int as uint16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_7(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).beetom_var_C = (0xa8ba24 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B6F2 as ::core::ffi::c_int as uint16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_8(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).beetom_var_B = (*E).beetom_var_02;
    (*E).beetom_var_C = (0xa8ba84 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).beetom_var_05 = 0 as uint16;
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B6AC as ::core::ffi::c_int as uint16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_9(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).beetom_var_B = (*E).beetom_var_02;
    (*E).beetom_var_C = (0xa8bab7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).beetom_var_05 = 0 as uint16;
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B708 as ::core::ffi::c_int as uint16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_10(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).beetom_var_B = (*E).beetom_var_03;
    (*E).beetom_var_C = (0xa8bb55 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).beetom_var_05 = 0 as uint16;
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B6AC as ::core::ffi::c_int as uint16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_11(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).beetom_var_B = (*E).beetom_var_03;
    (*E).beetom_var_C = (0xa8bb88 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).beetom_var_05 = 0 as uint16;
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B708 as ::core::ffi::c_int as uint16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_12(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).beetom_var_B = (*E).beetom_var_04;
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B708 as ::core::ffi::c_int as uint16;
    (*E).beetom_var_C = (0xa8bc5a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).beetom_var_09 = 1 as uint16;
    if GetSamusEnemyDelta_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).beetom_var_00 = addr_kBeetom_Ilist_B6AC as ::core::ffi::c_int as uint16;
        (*E).beetom_var_C =
            (0xa8bc26 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).beetom_var_09 = 0 as uint16;
    }
    Beetom_Func_1(game);
    (*E).beetom_var_05 = 0 as uint16;
}
pub unsafe fn Beetom_Func_13(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B6CC as ::core::ffi::c_int as uint16;
    Beetom_Func_1(game);
    (*E).beetom_var_C = (0xa8bcf8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Beetom_Func_14(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B728 as ::core::ffi::c_int as uint16;
    Beetom_Func_1(game);
    (*E).beetom_var_C = (0xa8bd42 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Beetom_Func_15(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B696 as ::core::ffi::c_int as uint16;
    if (*E).beetom_var_09 != 0 {
        (*E).beetom_var_00 = addr_kBeetom_Ilist_B6F2 as ::core::ffi::c_int as uint16;
    }
    Beetom_Func_1(game);
    (*E).beetom_var_05 = 0 as uint16;
    (*E).beetom_var_C = (0xa8bd9d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Beetom_Func_16(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).beetom_var_B = 0 as uint16;
    (*E).beetom_var_C = (0xa8bdc5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Beetom_Func_17(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).beetom_var_D = (*E).beetom_var_D.wrapping_sub(1);
    if (*E).beetom_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).beetom_var_C =
            (0xa8b814 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Beetom_Func_18(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).beetom_var_D = (*E).beetom_var_D.wrapping_sub(1);
    if (*E).beetom_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).beetom_var_D = 64 as uint16;
        (*E).beetom_var_C =
            (0xa8b814 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).base.x_pos =
            ((*E).base.x_pos as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
        if Enemy_MoveDown(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).base.x_pos =
                ((*E).base.x_pos as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
            if Enemy_MoveRight_IgnoreSlopes(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                -(16384 as int32),
                game,
            ) != 0
            {
                (*E).beetom_var_C =
                    (0xa8b873 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            }
        } else {
            (*E).beetom_var_C =
                (0xa8b873 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
            (*E).base.x_pos =
                ((*E).base.x_pos as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
        }
    };
}
pub unsafe fn Beetom_Func_19(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).beetom_var_D = (*E).beetom_var_D.wrapping_sub(1);
    if (*E).beetom_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).beetom_var_D = 64 as uint16;
        (*E).beetom_var_C =
            (0xa8b814 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).base.x_pos =
            ((*E).base.x_pos as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
        if Enemy_MoveDown(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).base.x_pos =
                ((*E).base.x_pos as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
            if Enemy_MoveRight_IgnoreSlopes(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                0x4000 as int32,
                game,
            ) != 0
            {
                (*E).beetom_var_C =
                    (0xa8b85f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            }
        } else {
            (*E).beetom_var_C =
                (0xa8b85f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
            (*E).base.x_pos =
                ((*E).base.x_pos as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
        }
    };
}
pub unsafe fn Beetom_Func_22(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(k as ::core::ffi::c_int, game);
    let mut v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * (*E).beetom_var_B as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(
            (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint32)
            .offset((v2 + 2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_C =
            (0xa8b97a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).beetom_var_B =
            ((*E).beetom_var_B as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
        if (*E).beetom_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).beetom_var_B = 0 as uint16;
            (*E).beetom_var_05 = 1 as uint16;
        }
    };
}
pub unsafe fn Beetom_Func_23(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(k as ::core::ffi::c_int, game);
    let mut v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * (*E).beetom_var_B as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(
            (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint32)
            .offset((v2 >> 1 as ::core::ffi::c_int) as isize) as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_C =
            (0xa8b814 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).beetom_var_B =
            ((*E).beetom_var_B as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
        if (*E).beetom_var_B as ::core::ffi::c_int - 64 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).beetom_var_B = 64 as uint16;
        }
    };
}
pub unsafe fn Beetom_Func_20(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).beetom_var_05 != 0 {
        Beetom_Func_23(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    } else {
        Beetom_Func_22(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        -(16384 as int32),
        game,
    ) != 0
    {
        (*E).beetom_var_09 =
            ((*E).beetom_var_09 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
        (*E).beetom_var_C =
            (0xa8b97a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Beetom_Func_21(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).beetom_var_05 != 0 {
        Beetom_Func_23(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    } else {
        Beetom_Func_22(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0x4000 as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_09 =
            ((*E).beetom_var_09 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
        (*E).beetom_var_C =
            (0xa8b97a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Beetom_Func_26(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(k as ::core::ffi::c_int, game);
    let mut v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * (*E).beetom_var_B as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(
            (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint32)
            .offset((v2 + 2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_C =
            (0xa8b97a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).beetom_var_B =
            ((*E).beetom_var_B as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as uint16;
        if (*E).beetom_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).beetom_var_B = 0 as uint16;
            (*E).beetom_var_05 = 1 as uint16;
        }
    };
}
pub unsafe fn Beetom_Func_27(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(k as ::core::ffi::c_int, game);
    let mut v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * (*E).beetom_var_B as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(
            (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint32)
            .offset((v2 >> 1 as ::core::ffi::c_int) as isize) as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_C =
            (0xa8b814 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).beetom_var_B =
            ((*E).beetom_var_B as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16;
        if (*E).beetom_var_B as ::core::ffi::c_int - 64 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).beetom_var_B = 64 as uint16;
        }
    };
}
pub unsafe fn Beetom_Func_24(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).beetom_var_05 != 0 {
        Beetom_Func_27(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    } else {
        Beetom_Func_26(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        -(16384 as int32),
        game,
    ) != 0
    {
        (*E).beetom_var_09 =
            ((*E).beetom_var_09 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
        (*E).beetom_var_C =
            (0xa8b97a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Beetom_Func_25(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).beetom_var_05 != 0 {
        Beetom_Func_27(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    } else {
        Beetom_Func_26(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0x4000 as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_09 =
            ((*E).beetom_var_09 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
        (*E).beetom_var_C =
            (0xa8b97a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Beetom_Func_30(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(k as ::core::ffi::c_int, game);
    let mut v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * (*E).beetom_var_B as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(
            (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint32)
            .offset((v2 + 2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_C =
            (0xa8b97a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        let mut v3: int16 =
            ((*E).beetom_var_B as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as int16;
        (*E).beetom_var_B = v3 as uint16;
        if (v3 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            (*E).beetom_var_B = 0 as uint16;
            (*E).beetom_var_05 = 1 as uint16;
        }
    };
}
pub unsafe fn Beetom_Func_31(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(k as ::core::ffi::c_int, game);
    let mut v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * (*E).beetom_var_B as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(
            (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint32)
            .offset((v2 >> 1 as ::core::ffi::c_int) as isize) as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_C =
            (0xa8b814 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        let mut v3: uint16 =
            ((*E).beetom_var_B as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16;
        (*E).beetom_var_B = v3;
        if v3 as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            (*E).beetom_var_B = 64 as uint16;
        }
    };
}
pub unsafe fn Beetom_Func_28(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).beetom_var_05 != 0 {
        Beetom_Func_31(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    } else {
        Beetom_Func_30(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        ((-(3 as ::core::ffi::c_int) as uint32) << 16 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int as uint16 as uint32) as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_09 =
            ((*E).beetom_var_09 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
        (*E).beetom_var_C =
            (0xa8b97a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Beetom_Func_29(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).beetom_var_05 != 0 {
        Beetom_Func_31(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    } else {
        Beetom_Func_30(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        ((3 as ::core::ffi::c_int as uint32) << 16 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int as uint16 as uint32) as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_09 =
            ((*E).beetom_var_09 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
        (*E).beetom_var_C =
            (0xa8b97a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Beetom_Func_34(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int
        != (*E).beetom_var_F as ::core::ffi::c_int
    {
        (*E).beetom_var_F = *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16);
        (*E).beetom_var_E = (*E).beetom_var_E.wrapping_sub(1);
    }
}
pub unsafe fn Beetom_Func_32(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).beetom_var_E != 0 {
        (*E).base.x_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
        (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
            as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int) as uint16;
        Beetom_Func_34(game);
    } else {
        if Enemy_MoveRight_IgnoreSlopes(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            ((16 as ::core::ffi::c_int as uint32) << 16 as ::core::ffi::c_int
                | 0 as ::core::ffi::c_int as uint16 as uint32) as int32,
            game,
        ) != 0
        {
            (*E).beetom_var_09 = 1 as uint16;
            Enemy_MoveRight_IgnoreSlopes(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                ((-(32 as ::core::ffi::c_int) as uint32) << 16 as ::core::ffi::c_int
                    | 0 as ::core::ffi::c_int as uint16 as uint32) as int32,
                game,
            );
        }
        (*E).beetom_var_08 = 0 as uint16;
        (*E).beetom_var_C =
            (0xa8b9a2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Beetom_Func_33(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).beetom_var_E != 0 {
        (*E).base.x_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
        (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
            as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int) as uint16;
        Beetom_Func_34(game);
    } else {
        if Enemy_MoveRight_IgnoreSlopes(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            ((-(16 as ::core::ffi::c_int) as uint32) << 16 as ::core::ffi::c_int
                | 0 as ::core::ffi::c_int as uint16 as uint32) as int32,
            game,
        ) != 0
        {
            (*E).beetom_var_09 = 0 as uint16;
            Enemy_MoveRight_IgnoreSlopes(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                ((32 as ::core::ffi::c_int as uint32) << 16 as ::core::ffi::c_int
                    | 0 as ::core::ffi::c_int as uint16 as uint32) as int32,
                game,
            );
        }
        (*E).beetom_var_08 = 0 as uint16;
        (*E).beetom_var_C =
            (0xa8b9a2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Beetom_Func_35(game: &mut Game) {
    if Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        ((3 as ::core::ffi::c_int as uint32) << 16 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int as uint16 as uint32) as int32,
        game,
    ) != 0
    {
        let mut E: *mut Enemy_Beetom = Get_Beetom(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        if (*E).beetom_var_09 != 0 {
            (*E).beetom_var_C =
                (0xa8b873 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            (*E).beetom_var_C =
                (0xa8b85f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn Beetom_Func_37(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(k as ::core::ffi::c_int, game);
    let mut v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * (*E).beetom_var_B as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(
            (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint32)
            .offset((v2 >> 1 as ::core::ffi::c_int) as isize) as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_C =
            (0xa8b814 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        let mut v3: uint16 =
            ((*E).beetom_var_B as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E).beetom_var_B = v3;
        if v3 as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            (*E).beetom_var_B = 64 as uint16;
        }
    }
    let mut v4: uint16 = (if (*E).beetom_var_09 as ::core::ffi::c_int != 0 {
        -(2 as ::core::ffi::c_int)
    } else {
        2 as ::core::ffi::c_int
    }) as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((v4 as uint32) << 16 as ::core::ffi::c_int | 0 as ::core::ffi::c_int as uint16 as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_09 =
            ((*E).beetom_var_09 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
        (*E).beetom_var_C =
            (0xa8b97a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Beetom_Func_36(game: &mut Game) {
    Beetom_Func_37(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn CallBeetomFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        11057172 => {
            Beetom_Func_3(game);
            return;
        }
        11057199 => {
            Beetom_Func_4(game);
            return;
        }
        11057231 => {
            Beetom_Func_5(game);
            return;
        }
        11057247 => {
            Beetom_Func_6(game);
            return;
        }
        11057267 => {
            Beetom_Func_7(game);
            return;
        }
        11057287 => {
            Beetom_Func_8(game);
            return;
        }
        11057321 => {
            Beetom_Func_9(game);
            return;
        }
        11057355 => {
            Beetom_Func_10(game);
            return;
        }
        11057389 => {
            Beetom_Func_11(game);
            return;
        }
        11057423 => {
            Beetom_Func_12(game);
            return;
        }
        11057490 => {
            Beetom_Func_13(game);
            return;
        }
        11057510 => {
            Beetom_Func_14(game);
            return;
        }
        11057530 => {
            Beetom_Func_15(game);
            return;
        }
        11057570 => {
            Beetom_Func_16(game);
            return;
        }
        11057586 => {
            Beetom_Func_17(game);
            return;
        }
        11057601 => {
            Beetom_Func_18(game);
            return;
        }
        11057700 => {
            Beetom_Func_19(game);
            return;
        }
        11057796 => {
            Beetom_Func_20(game);
            return;
        }
        11057847 => {
            Beetom_Func_21(game);
            return;
        }
        11058005 => {
            Beetom_Func_24(game);
            return;
        }
        11058056 => {
            Beetom_Func_25(game);
            return;
        }
        11058214 => {
            Beetom_Func_28(game);
            return;
        }
        11058266 => {
            Beetom_Func_29(game);
            return;
        }
        11058424 => {
            Beetom_Func_32(game);
            return;
        }
        11058498 => {
            Beetom_Func_33(game);
            return;
        }
        11058589 => {
            Beetom_Func_35(game);
            return;
        }
        11058629 => {
            Beetom_Func_36(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn Beetom_Main(game: &mut Game) {
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallBeetomFunc(
        ((*E).beetom_var_C as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn Beetom_Touch(game: &mut Game) {
    let mut current_block: u64;
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).beetom_var_08 == 0 {
        if (*E).beetom_var_09 != 0 {
            (*E).beetom_var_C =
                (0xa8b966 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            (*E).beetom_var_C =
                (0xa8b952 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        (*E).beetom_var_08 = 1 as uint16;
        (*E).beetom_var_E = 64 as uint16;
        (*E).base.layer = 2 as uint16;
        (*E).beetom_var_06 = (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as uint16;
        (*E).beetom_var_07 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
            as ::core::ffi::c_int
            - (*E).base.y_pos as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa6e) as *mut uint16) != 0 {
        current_block = 12103599785691237014;
    } else {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xe44) as *mut uint16) as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int
            == 7 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16)
                as ::core::ffi::c_int
                - 30 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            QueueSfx3_Max6(0x2d as uint16, game);
        }
        if (*E).base.frame_counter as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int
            == 63 as ::core::ffi::c_int
        {
            current_block = 12103599785691237014;
        } else {
            current_block = 10599921512955367680;
        }
    }
    match current_block {
        12103599785691237014 => {
            Enemy_NormalTouchAI_A8(game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x18aa) as *mut uint16) = 0 as uint16;
        }
        _ => {}
    };
}
pub unsafe fn Beetom_Shot(game: &mut Game) {
    Enemy_NormalShotAI_A8(game);
    let mut E: *mut Enemy_Beetom = Get_Beetom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).base.frozen_timer as ::core::ffi::c_int != 0
        && ((*E).beetom_var_C as ::core::ffi::c_int
            == 0xa8bd42 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            || (*E).beetom_var_C as ::core::ffi::c_int
                == 0xa8bcf8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
    {
        (*E).beetom_var_C =
            (0xa8b97a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    (*E).beetom_var_08 = 0 as uint16;
}
pub unsafe fn MaridiaFloater_Init(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_DisableSamusColl as ::core::ffi::c_int) as uint16;
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A8 as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    if (*E).mfr_parameter_1 != 0 {
        (*E).mfr_var_A = (*E).base.x_pos;
        (*E).mfr_var_B = (*E).base.y_pos;
        (*E).mfr_var_F = (0xa8c568 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction = addr_stru_A8C199 as ::core::ffi::c_int as uint16;
        (*E).mfr_var_D = (*E).mfr_parameter_2;
    } else {
        (*E).mfr_var_E = 60 as uint16;
        (*E).mfr_var_F = (0xa8c283 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction =
            addr_kMaridiaFloater_Ilist_C173 as ::core::ffi::c_int as uint16;
    };
}
pub unsafe fn MaridiaFloater_Main(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallEnemyPreInstr(
        ((*E).mfr_var_F as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn MaridiaFloater_Func_1(mut k: uint16, game: &mut Game) {
    let mut i: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        SpawnEprojWithGfx(
            i as uint16,
            k,
            addr_kEproj_MaridiaFloatersSpikes as ::core::ffi::c_int as uint16,
            game,
        );
        i -= 1;
    }
}
pub unsafe fn MaridiaFloater_Func_2(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 103 as ::core::ffi::c_int) as isize);
    if v2 as ::core::ffi::c_int + 0x3e6f as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        let mut v3: uint16 = ((v2 as ::core::ffi::c_int - 4 as ::core::ffi::c_int
            + 15997 as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16;
        if v3 as ::core::ffi::c_int - 6 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v3 = 0 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 93 as ::core::ffi::c_int) as isize) =
            (g_word_A8C277[(v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int
                + (*Get_MaridiaFloater(k as ::core::ffi::c_int, game))
                    .base
                    .y_pos as ::core::ffi::c_int) as uint16;
    } else {
        let mut v4: uint16 = ((v2 as ::core::ffi::c_int - 4 as ::core::ffi::c_int
            + 15983 as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16;
        if v4 as ::core::ffi::c_int - 6 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v4 = 0 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 93 as ::core::ffi::c_int) as isize) =
            (g_word_A8C27D[(v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int
                + (*Get_MaridiaFloater(k as ::core::ffi::c_int, game))
                    .base
                    .y_pos as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn MaridiaFloater_Func_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).mfr_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).mfr_var_E = (*E).mfr_var_E.wrapping_sub(1);
    let mut v3: bool = (*E).mfr_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        let mut v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v4 + 104 as ::core::ffi::c_int) as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v4 + 103 as ::core::ffi::c_int) as isize) =
            addr_kMaridiaFloater_Ilist_C183 as ::core::ffi::c_int as uint16;
        (*E).mfr_var_F = (0xa8c2a6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mfr_var_E = 10 as uint16;
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).mfr_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).mfr_var_E = (*E).mfr_var_E.wrapping_sub(1);
    let mut v3: bool = (*E).mfr_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*E).mfr_var_F = (0xa8c2cf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mfr_var_A = g_word_A8C1C5;
        (*E).mfr_var_B = g_word_A8C1C7;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction =
            addr_kMaridiaFloater_Ilist_C163 as ::core::ffi::c_int as uint16;
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    if (*E).base.ai_handler_bits as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mfr_var_F = (0xa8c3e1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        MaridiaFloater_Func_7(k, game);
    } else {
        let mut v2: bool = (*E).mfr_var_D as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
        (*E).mfr_var_D = (*E).mfr_var_D.wrapping_sub(1);
        let mut v3: bool = (*E).mfr_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
        if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
            (*E).mfr_var_D = 5 as uint16;
            let mut v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v5: uint16 = (g_word_A8C1A1[(*E).mfr_var_C as usize] as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
                    .offset(v4 as isize) as ::core::ffi::c_int)
                as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v4 + 91 as ::core::ffi::c_int) as isize) = v5;
            (*E).base.x_pos = v5;
            let mut v6: uint16 =
                ((*E).mfr_var_C as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            if (*E).mfr_var_C as ::core::ffi::c_int - 11 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v6 = 0 as uint16;
            }
            (*E).mfr_var_C = v6;
        }
        AddToHiLo(
            &raw mut (*E).mfr_var_A,
            &raw mut (*E).mfr_var_B,
            (g_word_A8C1B9 as uint32) << 16 as ::core::ffi::c_int | g_word_A8C1BB as uint32,
        );
        if Enemy_MoveDown(
            k,
            (((*E).mfr_var_A as uint32) << 16 as ::core::ffi::c_int | (*E).mfr_var_B as uint32)
                as int32,
            game,
        ) as ::core::ffi::c_int
            != 0
            || (*((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16).offset(
                ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                    as isize,
            ) as ::core::ffi::c_int
                - g_word_A8C19F as ::core::ffi::c_int
                - (*E).base.y_pos as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
        {
            let mut mfr_var_C: uint16 = (*E).mfr_var_C;
            if mfr_var_C == 0 || mfr_var_C as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
                (*E).mfr_var_F =
                    (0xa8c500 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                (*E).mfr_var_E = 10 as uint16;
                let mut v10: ::core::ffi::c_int =
                    k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                    .offset((v10 + 104 as ::core::ffi::c_int) as isize) = 1 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                    .offset((v10 + 103 as ::core::ffi::c_int) as isize) =
                    addr_kMaridiaFloater_Ilist_C191 as ::core::ffi::c_int as uint16;
            } else {
                (*E).mfr_var_F =
                    (0xa8c36b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            }
        }
        MaridiaFloater_Func_2(k, game);
    };
}
pub unsafe fn MaridiaFloater_Func_6(mut k: uint16, game: &mut Game) {
    let mut v4: ::core::ffi::c_int = 0;
    let mut current_block: u64;
    let mut E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).mfr_var_D as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).mfr_var_D = (*E).mfr_var_D.wrapping_sub(1);
    let mut v3: bool = (*E).mfr_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if !v2 && !v3 {
        current_block = 14916268686031723178;
    } else {
        (*E).mfr_var_D = 5 as uint16;
        v4 = 0;
        v4 = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v5: uint16 = (g_word_A8C1A1[(*E).mfr_var_C as usize] as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
                .offset(v4 as isize) as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v4 + 91 as ::core::ffi::c_int) as isize) = v5;
        (*E).base.x_pos = v5;
        if (*E).mfr_var_C as ::core::ffi::c_int != 0
            && (*E).mfr_var_C as ::core::ffi::c_int != 6 as ::core::ffi::c_int
        {
            let mut v7: uint16 =
                ((*E).mfr_var_C as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            if v7 as ::core::ffi::c_int - 12 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v7 = 0 as uint16;
            }
            (*E).mfr_var_C = v7;
            current_block = 14916268686031723178;
        } else {
            (*E).mfr_var_F =
                (0xa8c500 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).mfr_var_E = 10 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v4 + 104 as ::core::ffi::c_int) as isize) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v4 + 103 as ::core::ffi::c_int) as isize) =
                addr_kMaridiaFloater_Ilist_C191 as ::core::ffi::c_int as uint16;
            current_block = 14963264723745262966;
        }
    }
    match current_block {
        14916268686031723178 => {
            AddToHiLo(
                &raw mut (*E).mfr_var_A,
                &raw mut (*E).mfr_var_B,
                (g_word_A8C1B9 as uint32) << 16 as ::core::ffi::c_int | g_word_A8C1BB as uint32,
            );
            Enemy_MoveDown(
                k,
                (((*E).mfr_var_A as uint32) << 16 as ::core::ffi::c_int | (*E).mfr_var_B as uint32)
                    as int32,
                game,
            );
        }
        _ => {}
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_7(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    if (*E).base.ai_handler_bits as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut v2: bool = (*E).mfr_var_D as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
        (*E).mfr_var_D = (*E).mfr_var_D.wrapping_sub(1);
        let mut v3: bool = (*E).mfr_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
        if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
            (*E).mfr_var_D = 5 as uint16;
            let mut v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v5: uint16 = (g_word_A8C1A1[(*E).mfr_var_C as usize] as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
                    .offset(v4 as isize) as ::core::ffi::c_int)
                as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v4 + 91 as ::core::ffi::c_int) as isize) = v5;
            (*E).base.x_pos = v5;
            let mut v6: uint16 =
                ((*E).mfr_var_C as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            if (*E).mfr_var_C as ::core::ffi::c_int - 11 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v6 = 0 as uint16;
            }
            (*E).mfr_var_C = v6;
        }
        AddToHiLo(
            &raw mut (*E).mfr_var_A,
            &raw mut (*E).mfr_var_B,
            (g_word_A8C1B9 as uint32) << 16 as ::core::ffi::c_int | g_word_A8C1BB as uint32,
        );
        if Enemy_MoveDown(
            k,
            (((*E).mfr_var_A as uint32) << 16 as ::core::ffi::c_int | (*E).mfr_var_B as uint32)
                as int32,
            game,
        ) as ::core::ffi::c_int
            != 0
            || (*((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16).offset(
                ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                    as isize,
            ) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16).offset(
                    ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 3 as ::core::ffi::c_int)
                        as isize,
                ) as ::core::ffi::c_int
                - (*E).base.y_pos as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
        {
            if (*E).mfr_var_C == 0
                || (*E).mfr_var_C as ::core::ffi::c_int == 6 as ::core::ffi::c_int
            {
                (*E).mfr_var_F =
                    (0xa8c4dc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            } else {
                (*E).mfr_var_F =
                    (0xa8c469 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            }
        }
        MaridiaFloater_Func_2(k, game);
    } else {
        (*E).mfr_var_F = (0xa8c36b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn MaridiaFloater_Func_8(mut k: uint16, game: &mut Game) {
    let mut v4: ::core::ffi::c_int = 0;
    let mut v5: uint16 = 0;
    let mut current_block: u64;
    let mut E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    if (*E).base.ai_handler_bits as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).mfr_var_F = (0xa8c36b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        return;
    }
    let mut v2: bool = (*E).mfr_var_D as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).mfr_var_D = (*E).mfr_var_D.wrapping_sub(1);
    let mut v3: bool = (*E).mfr_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if !v2 && !v3 {
        current_block = 12800627514080957624;
    } else {
        (*E).mfr_var_D = 5 as uint16;
        v4 = 0;
        v4 = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        v5 = 0;
        v5 = (g_word_A8C1A1[(*E).mfr_var_C as usize] as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
                .offset(v4 as isize) as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v4 + 91 as ::core::ffi::c_int) as isize) = v5;
        (*E).base.x_pos = v5;
        if (*E).mfr_var_C as ::core::ffi::c_int != 0
            && (*E).mfr_var_C as ::core::ffi::c_int != 6 as ::core::ffi::c_int
        {
            let mut v7: uint16 =
                ((*E).mfr_var_C as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            if v7 as ::core::ffi::c_int - 12 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v7 = 0 as uint16;
            }
            (*E).mfr_var_C = v7;
            current_block = 12800627514080957624;
        } else {
            (*E).mfr_var_F =
                (0xa8c4dc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            current_block = 4833750592470317582;
        }
    }
    match current_block {
        12800627514080957624 => {
            AddToHiLo(
                &raw mut (*E).mfr_var_A,
                &raw mut (*E).mfr_var_B,
                (g_word_A8C1B9 as uint32) << 16 as ::core::ffi::c_int | g_word_A8C1BB as uint32,
            );
            Enemy_MoveDown(
                k,
                (((*E).mfr_var_A as uint32) << 16 as ::core::ffi::c_int | (*E).mfr_var_B as uint32)
                    as int32,
                game,
            );
        }
        _ => {}
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_9(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    if (*E).base.ai_handler_bits as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).mfr_var_F = (0xa8c500 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mfr_var_E = 10 as uint16;
        let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v2 + 104 as ::core::ffi::c_int) as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v2 + 103 as ::core::ffi::c_int) as isize) =
            addr_kMaridiaFloater_Ilist_C191 as ::core::ffi::c_int as uint16;
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_10(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).mfr_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).mfr_var_E = (*E).mfr_var_E.wrapping_sub(1);
    let mut v3: bool = (*E).mfr_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*E).mfr_var_F = (0xa8c51d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mfr_var_A = g_word_A8C1C1;
        (*E).mfr_var_B = g_word_A8C1C3;
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_11(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).mfr_var_A,
        &raw mut (*E).mfr_var_B,
        (g_word_A8C1BD as uint32) << 16 as ::core::ffi::c_int | g_word_A8C1BF as uint32,
    );
    Enemy_MoveDown(
        k,
        (((*E).mfr_var_A as uint32) << 16 as ::core::ffi::c_int | (*E).mfr_var_B as uint32)
            as int32,
        game,
    );
    let mut v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if ((*E).base.y_pos as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
            .offset((v4 + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
            .offset((v4 + 1 as ::core::ffi::c_int) as isize);
        (*E).mfr_var_F = (0xa8c283 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mfr_var_E = 60 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction =
            addr_kMaridiaFloater_Ilist_C173 as ::core::ffi::c_int as uint16;
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_12(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 103 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
        + 0x3e6f as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        let mut v2: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 103 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int
            + 15983 as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16;
        if v2 != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v1 + 103 as ::core::ffi::c_int) as isize) = *(game.g_rom.offset(
                (((0xa8c599 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa8c599 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8
                as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v1 + 104 as ::core::ffi::c_int) as isize) = 1 as uint16;
        }
    }
    let mut E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    (*E).mfr_var_F = (0xa8c59f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mfr_var_E = 32 as uint16;
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_13(mut k: uint16, game: &mut Game) {
    let mut v5: uint16 = 0;
    let mut E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).mfr_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).mfr_var_E = (*E).mfr_var_E.wrapping_sub(1);
    let mut v3: bool = (*E).mfr_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        let mut v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v4 + 100 as ::core::ffi::c_int) as isize) = 0 as uint16;
        let ref mut fresh2 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v4 + 97 as ::core::ffi::c_int) as isize);
        *fresh2 = (*fresh2 as ::core::ffi::c_int | 0x200 as ::core::ffi::c_int) as uint16;
        MaridiaFloater_Func_1(k, game);
        v5 = 0 as uint16;
        EnemyDeathAnimation(k, v5, game);
    } else {
        MaridiaFloater_Func_2(k, game);
    };
}
pub unsafe fn MaridiaFloater_Touch(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).mfr_parameter_2 == 0 {
        let mut palette_index: uint16 = (*E).base.palette_index;
        NormalEnemyTouchAi(game);
        if (*E).base.health == 0 {
            let ref mut fresh6 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16).offset(
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                        as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int)
                        + 97 as ::core::ffi::c_int) as isize,
                );
            *fresh6 = (*fresh6 as ::core::ffi::c_int | 0x200 as ::core::ffi::c_int) as uint16;
            (*E).base.palette_index = palette_index;
            MaridiaFloater_Func_1(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                game,
            );
            (*E).base.palette_index = 2560 as uint16;
        }
    }
}
pub unsafe fn MaridiaFloater_Shot(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).mfr_parameter_2 == 0 {
        NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
        if (*E).base.ai_handler_bits as ::core::ffi::c_int & 4 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xe54)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v2 + 109 as ::core::ffi::c_int) as isize) = (*E).base.frozen_timer;
            let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84)
                as *mut uint16)
                .offset((v2 + 99 as ::core::ffi::c_int) as isize);
            *fresh4 = (*fresh4 as ::core::ffi::c_int | 4 as ::core::ffi::c_int) as uint16;
        }
        if (*E).base.ai_handler_bits as ::core::ffi::c_int & 2 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let mut v3: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xe54)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v3 + 108 as ::core::ffi::c_int) as isize) = (*E).base.flash_timer;
            let ref mut fresh5 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84)
                as *mut uint16)
                .offset((v3 + 99 as ::core::ffi::c_int) as isize);
            *fresh5 = (*fresh5 as ::core::ffi::c_int | 2 as ::core::ffi::c_int) as uint16;
        }
        if (*E).base.health == 0 {
            (*E).mfr_var_F =
                (0xa8c569 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).mfr_parameter_2 = 1 as uint16;
        }
    }
}
pub unsafe fn MaridiaFloater_Powerbomb(game: &mut Game) {
    Enemy_NormalPowerBombAI_A8(game);
    let mut E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).base.health != 0 {
        let mut v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xe54)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 111 as ::core::ffi::c_int) as isize) = (*E).base.shake_timer;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 110 as ::core::ffi::c_int) as isize) = (*E).base.invincibility_timer;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 108 as ::core::ffi::c_int) as isize) = (*E).base.flash_timer;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 109 as ::core::ffi::c_int) as isize) = (*E).base.frozen_timer;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 99 as ::core::ffi::c_int) as isize) = (*E).base.ai_handler_bits;
    } else {
        let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset(
                ((*((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int)
                    + 97 as ::core::ffi::c_int) as isize,
            );
        *fresh3 = (*fresh3 as ::core::ffi::c_int | 0x200 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn WreckedShipRobot_Init(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).bank =
            -(88 as ::core::ffi::c_int) as uint8;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).addr =
            (0xa8cc67 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP;
        let mut E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*E).base.properties =
            ((*E).base.properties as ::core::ffi::c_int | 0xa000 as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 4 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction =
            addr_kWreckedShipRobot_Ilist_C6E5 as ::core::ffi::c_int as uint16;
        (*E).wsrt_var_A = -(512 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd654) as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd656) as *mut uint16) = 0 as uint16;
        (*E).wsrt_var_B = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd652) as *mut uint16) =
            (*E).base.palette_index;
    } else {
        WreckedShipRobotDeactivated_Init(game);
    };
}
pub unsafe fn WreckedShipRobotDeactivated_Init(game: &mut Game) {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut parameter_1: uint16 = (*v0).parameter_1;
    if parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        || parameter_1 as ::core::ffi::c_int - 4 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        parameter_1 = 0 as uint16;
    }
    (*v0).parameter_1 = parameter_1;
    (*v0).current_instruction = *(game.g_rom.offset(
        (((0xa8cc30 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8cc30 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(parameter_1 as isize);
    (*v0).properties =
        ((*v0).properties as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
    (*v0).instruction_timer = 1 as uint16;
    (*v0).timer = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd654) as *mut uint16) = 0 as uint16;
    (*v0).ai_var_E = 0 as uint16;
    (*v0).ai_preinstr = 1 as uint16;
    let mut v2: ::core::ffi::c_int = 16 as ::core::ffi::c_int
        * ((*((&raw mut game.g_ram as *mut uint8).offset(0xd652) as *mut uint16)
            as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
        .offset((v2 + 137 as ::core::ffi::c_int) as isize) = 10 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
        .offset((v2 + 138 as ::core::ffi::c_int) as isize) = 10 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
        .offset((v2 + 139 as ::core::ffi::c_int) as isize) = 10 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
        .offset((v2 + 140 as ::core::ffi::c_int) as isize) = 10 as uint16;
}
pub unsafe fn WreckedShipRobot_Main(game: &mut Game) {
    let mut E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (((*E).wsrt_var_F as uint32) << 16 as ::core::ffi::c_int | (*E).wsrt_var_E as uint32)
            as int32,
        game,
    ) == 0
    {
        (*E).base.instruction_timer = (*E).base.instruction_timer.wrapping_add(1);
        AddToHiLo(
            &raw mut (*E).wsrt_var_F,
            &raw mut (*E).wsrt_var_E,
            0x8000 as uint32,
        );
    }
}
pub unsafe fn WreckedShipRobot_Func_1(game: &mut Game) {
    let mut i: uint16 = 0;
    let mut v2: int16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x797) as *mut uint16) == 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xd654) as *mut uint16) != 0 {
            let ref mut fresh13 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xd654) as *mut uint16);
            *fresh13 = (*fresh13).wrapping_sub(1);
            if *fresh13 == 0 {
                let mut v1: ::core::ffi::c_int = 0;
                i = *((&raw mut game.g_ram as *mut uint8).offset(0xd656) as *mut uint16);
                loop {
                    v1 = i as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                    v2 = g_word_A8CCC1[v1 as usize];
                    if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                        break;
                    }
                    i = 0 as uint16;
                }
                let mut v3: ::core::ffi::c_int = 16 as ::core::ffi::c_int
                    * ((*((&raw mut game.g_ram as *mut uint8).offset(0xd652) as *mut uint16)
                        as ::core::ffi::c_int
                        & 0xff00 as ::core::ffi::c_int)
                        >> 8 as ::core::ffi::c_int)
                    >> 1 as ::core::ffi::c_int;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                    .offset((v3 + 137 as ::core::ffi::c_int) as isize) = v2 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                    .offset((v3 + 138 as ::core::ffi::c_int) as isize) =
                    g_word_A8CCC1[(v1 + 1 as ::core::ffi::c_int) as usize] as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                    .offset((v3 + 139 as ::core::ffi::c_int) as isize) =
                    g_word_A8CCC1[(v1 + 2 as ::core::ffi::c_int) as usize] as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                    .offset((v3 + 140 as ::core::ffi::c_int) as isize) =
                    g_word_A8CCC1[(v1 + 3 as ::core::ffi::c_int) as usize] as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xd654) as *mut uint16) =
                    g_word_A8CCC1[(v1 + 4 as ::core::ffi::c_int) as usize] as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xd656) as *mut uint16) =
                    (i as ::core::ffi::c_int + 10 as ::core::ffi::c_int) as uint16;
            }
        }
    }
}
pub unsafe fn WreckedShipRobot_Func_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(k as ::core::ffi::c_int, game);
    let mut wsrt_var_B: uint16 = (*E).wsrt_var_B;
    if wsrt_var_B != 0 {
        (*E).wsrt_var_B = (wsrt_var_B as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn WreckedShipRobot_Instr_4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let mut E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).wsrt_var_A = -(512 as ::core::ffi::c_int) as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (-(4 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).wsrt_var_B =
            ((*E).wsrt_var_B as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
        return addr_kWreckedShipRobot_Ilist_C73F as ::core::ffi::c_int as uintptr_t
            as *const uint16;
    } else {
        if CheckIfEnemyTouchesSamus(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        ) != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) =
                -(4 as ::core::ffi::c_int) as uint16;
        }
        (*E).wsrt_var_D = (*E).base.y_pos;
        let mut x_pos: uint16 = (*E).base.x_pos;
        (*E).wsrt_var_C = x_pos;
        (*E).base.x_pos = (x_pos as ::core::ffi::c_int
            - (*E).base.x_width as ::core::ffi::c_int
            - (*E).base.x_width as ::core::ffi::c_int) as uint16;
        if Enemy_MoveDown(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).base.x_pos = (*E).wsrt_var_C;
            (*E).base.y_pos = (*E).wsrt_var_D;
        } else {
            (*E).wsrt_var_B =
                ((*E).wsrt_var_B as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
            (*E).base.x_pos = (*E).wsrt_var_C;
            (*E).base.y_pos = (*E).wsrt_var_D;
            (*E).wsrt_var_A = 512 as uint16;
            return addr_kWreckedShipRobot_Ilist_CB65 as ::core::ffi::c_int as uintptr_t
                as *const uint16;
        }
    }
    return jp;
}
pub unsafe fn WreckedShipRobot_Instr_9(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let mut E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).wsrt_var_A = -(512 as ::core::ffi::c_int) as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (-(4 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).wsrt_var_B =
            ((*E).wsrt_var_B as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
        return addr_kWreckedShipRobot_Ilist_C73F as ::core::ffi::c_int as uintptr_t
            as *const uint16;
    } else if CheckIfEnemyTouchesSamus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) =
            -(4 as ::core::ffi::c_int) as uint16;
    }
    return jp;
}
pub unsafe fn WreckedShipRobot_Instr_6(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let mut E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).wsrt_var_A = -(512 as ::core::ffi::c_int) as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (4 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).wsrt_var_B =
            ((*E).wsrt_var_B as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
        return addr_stru_A8C6E9 as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else {
        if CheckIfEnemyTouchesSamus(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        ) != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = 4 as uint16;
        }
        (*E).wsrt_var_D = (*E).base.y_pos;
        let mut x_pos: uint16 = (*E).base.x_pos;
        (*E).wsrt_var_C = x_pos;
        (*E).base.x_pos = ((*E).base.x_width as ::core::ffi::c_int
            + (*E).base.x_width as ::core::ffi::c_int
            + x_pos as ::core::ffi::c_int) as uint16;
        if Enemy_MoveDown(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).base.x_pos = (*E).wsrt_var_C;
            (*E).base.y_pos = (*E).wsrt_var_D;
        } else {
            (*E).wsrt_var_B =
                ((*E).wsrt_var_B as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
            (*E).base.x_pos = (*E).wsrt_var_C;
            (*E).base.y_pos = (*E).wsrt_var_D;
            (*E).wsrt_var_A = -(512 as ::core::ffi::c_int) as uint16;
            return addr_kWreckedShipRobot_Ilist_C91B as ::core::ffi::c_int as uintptr_t
                as *const uint16;
        }
    }
    return jp;
}
pub unsafe fn WreckedShipRobot_Instr_8(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let mut E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).wsrt_var_A = -(512 as ::core::ffi::c_int) as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (4 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).wsrt_var_B =
            ((*E).wsrt_var_B as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
        return addr_stru_A8C6E9 as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else if CheckIfEnemyTouchesSamus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = 4 as uint16;
    }
    return jp;
}
pub unsafe fn WreckedShipRobot_Instr_7(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return addr_kWreckedShipRobot_Ilist_C92D as ::core::ffi::c_int as uintptr_t as *const uint16;
}
pub unsafe fn WreckedShipRobot_Instr_15(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let mut E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).wsrt_var_A = 512 as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (4 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).wsrt_var_B =
            ((*E).wsrt_var_B as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
        return addr_kWreckedShipRobot_Ilist_C985 as ::core::ffi::c_int as uintptr_t
            as *const uint16;
    } else {
        if CheckIfEnemyTouchesSamus(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        ) != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = 4 as uint16;
        }
        (*E).wsrt_var_D = (*E).base.y_pos;
        let mut x_pos: uint16 = (*E).base.x_pos;
        (*E).wsrt_var_C = x_pos;
        (*E).base.x_pos = ((*E).base.x_width as ::core::ffi::c_int
            + (*E).base.x_width as ::core::ffi::c_int
            + x_pos as ::core::ffi::c_int) as uint16;
        if Enemy_MoveDown(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).base.x_pos = (*E).wsrt_var_C;
            (*E).base.y_pos = (*E).wsrt_var_D;
        } else {
            (*E).wsrt_var_B =
                ((*E).wsrt_var_B as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
            (*E).base.x_pos = (*E).wsrt_var_C;
            (*E).base.y_pos = (*E).wsrt_var_D;
            (*E).wsrt_var_A = -(512 as ::core::ffi::c_int) as uint16;
            return addr_kWreckedShipRobot_Ilist_C91B as ::core::ffi::c_int as uintptr_t
                as *const uint16;
        }
    }
    return jp;
}
pub unsafe fn WreckedShipRobot_Instr_18(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let mut E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).wsrt_var_A = 512 as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (4 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).wsrt_var_B =
            ((*E).wsrt_var_B as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
        return addr_kWreckedShipRobot_Ilist_C985 as ::core::ffi::c_int as uintptr_t
            as *const uint16;
    } else if CheckIfEnemyTouchesSamus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = 4 as uint16;
    }
    return jp;
}
pub unsafe fn WreckedShipRobot_Instr_16(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let mut E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).wsrt_var_A = 512 as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (-(4 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).wsrt_var_B =
            ((*E).wsrt_var_B as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
        return addr_stru_A8C6E9 as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else {
        if CheckIfEnemyTouchesSamus(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        ) != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) =
                -(4 as ::core::ffi::c_int) as uint16;
        }
        (*E).wsrt_var_D = (*E).base.y_pos;
        let mut x_pos: uint16 = (*E).base.x_pos;
        (*E).wsrt_var_C = x_pos;
        (*E).base.x_pos = (x_pos as ::core::ffi::c_int
            - (*E).base.x_width as ::core::ffi::c_int
            - (*E).base.x_width as ::core::ffi::c_int) as uint16;
        if Enemy_MoveDown(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).base.y_pos = (*E).wsrt_var_D;
            (*E).base.x_pos = (*E).wsrt_var_C;
        } else {
            (*E).wsrt_var_B =
                ((*E).wsrt_var_B as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
            (*E).base.y_pos = (*E).wsrt_var_D;
            (*E).base.x_pos = (*E).wsrt_var_C;
            (*E).wsrt_var_A = 512 as uint16;
            return addr_kWreckedShipRobot_Ilist_CB65 as ::core::ffi::c_int as uintptr_t
                as *const uint16;
        }
    }
    return jp;
}
pub unsafe fn WreckedShipRobot_Instr_17(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let mut E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).wsrt_var_A = 512 as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (-(4 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).wsrt_var_B =
            ((*E).wsrt_var_B as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
        return addr_stru_A8C6E9 as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else if CheckIfEnemyTouchesSamus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) =
            -(4 as ::core::ffi::c_int) as uint16;
    }
    return jp;
}
pub unsafe fn WreckedShipRobot_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) as ::core::ffi::c_int
        - (*E).base.x_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
            as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        && ((*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
            as ::core::ffi::c_int
            - (*E).base.y_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
            as ::core::ffi::c_int
            + 224 as ::core::ffi::c_int
            - (*E).base.y_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
    {
        QueueSfx2_Max6(0x68 as uint16, game);
    }
    return jp;
}
pub unsafe fn WreckedShipRobot_Instr_10(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return addr_stru_A8C6E9 as ::core::ffi::c_int as uintptr_t as *const uint16;
}
pub unsafe fn WreckedShipRobot_Instr_14(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return WreckedShipRobot_CommonInstr(
        k,
        jp,
        addr_kWreckedShipRobot_Ilist_CB1D as ::core::ffi::c_int as uint16,
        addr_kEproj_WreckedShipRobotLaserUpRight as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn WreckedShipRobot_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return WreckedShipRobot_CommonInstr(
        k,
        jp,
        addr_kWreckedShipRobot_Ilist_C8D1 as ::core::ffi::c_int as uint16,
        addr_kEproj_WreckedShipRobotLaserUpLeft as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn WreckedShipRobot_CommonInstr(
    mut k: uint16,
    mut jp: *const uint16,
    mut r50: uint16,
    mut r48: uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(k as ::core::ffi::c_int, game);
    if (*E).wsrt_var_B != 0 {
        WreckedShipRobot_Func_2(k, game);
    } else {
        let mut v3: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
            + 16 as ::core::ffi::c_int) as uint16;
        (*E).wsrt_var_B = v3;
        SpawnEprojWithGfx(
            v3,
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            r48,
            game,
        );
        return r50 as uintptr_t as *const uint16;
    }
    return jp;
}
pub unsafe fn WreckedShipRobot_Instr_13(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return WreckedShipRobot_D10C(
        k,
        jp,
        addr_kWreckedShipRobot_Ilist_CB09 as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn WreckedShipRobot_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return WreckedShipRobot_D10C(
        k,
        jp,
        addr_kWreckedShipRobot_Ilist_C8BD as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn WreckedShipRobot_D10C(
    mut k: uint16,
    mut jp: *const uint16,
    mut r50: uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(k as ::core::ffi::c_int, game);
    if (*E).wsrt_var_B != 0 {
        WreckedShipRobot_Func_2(k, game);
    } else {
        let mut v3: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
            + 16 as ::core::ffi::c_int) as uint16;
        (*E).wsrt_var_B = v3;
        SpawnEprojWithGfx(
            v3,
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            addr_kEproj_WreckedShipRobotLaserHorizontal as ::core::ffi::c_int as uint16,
            game,
        );
        return r50 as uintptr_t as *const uint16;
    }
    return jp;
}
pub unsafe fn WreckedShipRobot_Instr_12(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return WreckedShipRobot_D147(
        k,
        jp,
        addr_kEproj_WreckedShipRobotLaserDownRight as ::core::ffi::c_int as uint16,
        addr_kWreckedShipRobot_Ilist_CAFD as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn WreckedShipRobot_Instr_5(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return WreckedShipRobot_D147(
        k,
        jp,
        addr_kEproj_WreckedShipRobotLaserDownLeft as ::core::ffi::c_int as uint16,
        addr_kWreckedShipRobot_Ilist_C8B1 as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn WreckedShipRobot_D147(
    mut k: uint16,
    mut jp: *const uint16,
    mut r48: uint16,
    mut r50: uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(k as ::core::ffi::c_int, game);
    if (*E).wsrt_var_B != 0 {
        WreckedShipRobot_Func_2(k, game);
    } else {
        let mut v3: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
            + 16 as ::core::ffi::c_int) as uint16;
        (*E).wsrt_var_B = v3;
        SpawnEprojWithGfx(
            v3,
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            r48,
            game,
        );
        return r50 as uintptr_t as *const uint16;
    }
    return jp;
}
pub unsafe fn WreckedShipRobot_Instr_11(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    return jp;
}
pub unsafe fn WreckedShipRobotDeactivated_Touch(game: &mut Game) {
    if (((*gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .x_pos as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = 4 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) =
            -(4 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn WreckedShipRobotDeactivated_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
}
pub unsafe fn WreckedShipRobot_Shot(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        NormalEnemyShotAi(game);
        let mut E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        if (*E).base.health != 0 {
            let mut v1: uint16 = 0;
            if (*E).wsrt_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                if (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int
                    - (*E).base.x_pos as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int
                {
                    v1 = addr_kWreckedShipRobot_Ilist_C833 as ::core::ffi::c_int as uint16;
                } else {
                    v1 = addr_kWreckedShipRobot_Ilist_C7BB as ::core::ffi::c_int as uint16;
                }
            } else if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - (*E).base.x_pos as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                v1 = addr_kWreckedShipRobot_Ilist_CA7D as ::core::ffi::c_int as uint16;
            } else {
                v1 = addr_kWreckedShipRobot_Ilist_CA01 as ::core::ffi::c_int as uint16;
            }
            (*E).base.current_instruction = v1;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).wsrt_var_B =
                ((*E).wsrt_var_B as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn MaridiaPuffer_Init(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaPuffer = Get_MaridiaPuffer(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction = addr_kMaridiaPuffer_Ilist_D841 as ::core::ffi::c_int as uint16;
    let mut v1: ::core::ffi::c_int = (4 as ::core::ffi::c_int
        * (*E).mpr_parameter_1 as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = g_word_A8D895[v1 as usize] as uint16;
    (*E).mpr_var_05 = v2;
    (*E).mpr_var_F = v2;
    (*E).mpr_var_06 = g_word_A8D895[(v1 + 1 as ::core::ffi::c_int) as usize] as uint16;
    (*E).mpr_var_E = 16 as uint16;
    (*E).mpr_var_A = (0xa8d92b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mpr_var_00 = g_word_A8D885[(*E).mpr_parameter_2 as usize] as uint16;
}
pub unsafe fn CallMaridiaPuffer(mut ea: uint32, game: &mut Game) {
    match ea {
        11065643 => {
            MaridiaPuffer_Func_1(game);
            return;
        }
        11065664 => {
            MaridiaPuffer_Func_2(game);
            return;
        }
        11065699 => {
            MaridiaPuffer_Func_3(game);
            return;
        }
        11065724 => {
            MaridiaPuffer_Func_4(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn MaridiaPuffer_Main(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaPuffer = Get_MaridiaPuffer(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v1: uint16 =
        ((*E).mpr_var_07 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E).mpr_var_07 = v1;
    if v1 == 0 {
        (*E).mpr_var_07 = 1 as uint16;
        (*E).mpr_var_03 = 0 as uint16;
    }
    CallMaridiaPuffer(
        ((*E).mpr_var_A as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn MaridiaPuffer_Func_1(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaPuffer = Get_MaridiaPuffer(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let fresh7 = (*E).mpr_var_E;
    (*E).mpr_var_E = (*E).mpr_var_E.wrapping_sub(1);
    if fresh7 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).mpr_var_E = 16 as uint16;
        (*E).mpr_var_A = (0xa8d940 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MaridiaPuffer_Func_2(game: &mut Game) {
    let mut v1: uint8 = (CalculateAngleOfSamusFromEnemy(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) as ::core::ffi::c_int
        - 64 as ::core::ffi::c_int) as uint8;
    let mut E: *mut Enemy_MaridiaPuffer = Get_MaridiaPuffer(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mpr_var_01 = v1 as uint16;
    (*E).mpr_var_02 = v1 as uint16;
    (*E).mpr_var_A = (0xa8d963 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mpr_var_C = 24 as uint16;
}
pub unsafe fn MaridiaPuffer_Func_3(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaPuffer = Get_MaridiaPuffer(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (((*E).mpr_var_D as ::core::ffi::c_int - (*E).mpr_var_00 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        MaridiaPuffer_Func_8(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
    MaridiaPuffer_Func_6(game);
    MaridiaPuffer_Func_7(game);
    MaridiaPuffer_Func_5(game);
}
pub unsafe fn MaridiaPuffer_Func_4(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaPuffer = Get_MaridiaPuffer(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).mpr_var_D as int16 as ::core::ffi::c_int > 0 as ::core::ffi::c_int
        && (*E).mpr_var_B as int16 as ::core::ffi::c_int > 0 as ::core::ffi::c_int
    {
        MaridiaPuffer_Func_6(game);
        MaridiaPuffer_Func_7(game);
        MaridiaPuffer_Func_9(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    } else {
        (*E).mpr_var_A = (0xa8d92b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mpr_var_B = 0 as uint16;
        (*E).mpr_var_C = 0 as uint16;
        (*E).mpr_var_D = 0 as uint16;
    };
}
pub unsafe fn MaridiaPuffer_Func_5(game: &mut Game) {
    let mut v1: uint8 = (CalculateAngleOfSamusFromEnemy(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) as ::core::ffi::c_int
        - 64 as ::core::ffi::c_int) as uint8;
    let mut E: *mut Enemy_MaridiaPuffer = Get_MaridiaPuffer(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mpr_var_01 = v1 as uint16;
    let mut v3: uint16 =
        SignExtend8((v1 as ::core::ffi::c_int - (*E).mpr_var_02 as ::core::ffi::c_int) as uint16);
    let mut v4: uint16 = Abs16(v3);
    if v4 as ::core::ffi::c_int - 48 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        (*E).mpr_var_A = (0xa8d97c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mpr_var_C = 24 as uint16;
    }
}
pub unsafe fn MaridiaPuffer_Func_6(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaPuffer = Get_MaridiaPuffer(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut R26: uint16 = 0 as uint16;
    let mut r18: uint16 = *(game.g_rom.offset(
        (((0xa0b1c3 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa0b1c3 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((*E).mpr_var_02 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint8 as isize,
        );
    if r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        R26 = R26.wrapping_add(1);
    }
    let mut t: uint32 = (((Abs16(r18) as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int)
        * (*E).mpr_var_D as ::core::ffi::c_int) as uint32;
    if R26 != 0 {
        t = -(t as int32) as uint32;
    }
    AddToHiLo(&raw mut (*E).base.x_pos, &raw mut (*E).base.x_subpos, t);
}
pub unsafe fn MaridiaPuffer_Func_7(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaPuffer = Get_MaridiaPuffer(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut R26: uint16 = 0 as uint16;
    let mut r18: uint16 = *(game.g_rom.offset(
        (((0xa0b1c3 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa0b1c3 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((*E).mpr_var_02 as uint8 as isize);
    if r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        R26 = R26.wrapping_add(1);
    }
    let mut t: uint32 = (((Abs16(r18) as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int)
        * (*E).mpr_var_D as ::core::ffi::c_int) as uint32;
    if R26 != 0 {
        t = -(t as int32) as uint32;
    }
    AddToHiLo(&raw mut (*E).base.y_pos, &raw mut (*E).base.y_subpos, t);
}
pub unsafe fn MaridiaPuffer_Func_8(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaPuffer = Get_MaridiaPuffer(k as ::core::ffi::c_int, game);
    let fresh8 = (*E).mpr_var_F;
    (*E).mpr_var_F = (*E).mpr_var_F.wrapping_sub(1);
    if fresh8 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).mpr_var_F = (*E).mpr_var_05;
        (*E).mpr_var_B =
            ((*E).mpr_var_B as ::core::ffi::c_int + (*E).mpr_var_C as ::core::ffi::c_int) as uint16;
        (*E).mpr_var_D =
            ((*E).mpr_var_D as ::core::ffi::c_int + (*E).mpr_var_B as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MaridiaPuffer_Func_9(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaPuffer = Get_MaridiaPuffer(k as ::core::ffi::c_int, game);
    let fresh9 = (*E).mpr_var_F;
    (*E).mpr_var_F = (*E).mpr_var_F.wrapping_sub(1);
    if fresh9 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).mpr_var_F = (*E).mpr_var_06;
        (*E).mpr_var_B =
            ((*E).mpr_var_B as ::core::ffi::c_int - (*E).mpr_var_C as ::core::ffi::c_int) as uint16;
        (*E).mpr_var_D =
            ((*E).mpr_var_D as ::core::ffi::c_int - (*E).mpr_var_B as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MaridiaPuffer_Shot(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaPuffer = Get_MaridiaPuffer(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mpr_var_40 = (*E).base.health;
    Enemy_NormalShotAI_A8(game);
    if (*E).base.health as ::core::ffi::c_int == (*E).mpr_var_40 as ::core::ffi::c_int
        && (*E).mpr_var_03 == 0
    {
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction =
            addr_kMaridiaPuffer_Ilist_D855 as ::core::ffi::c_int as uint16;
        (*E).mpr_var_02 = g_word_A8D871[(*((&raw mut game.g_ram as *mut uint8).offset(0xc04)
            as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16) as isize)
            as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int) as usize] as uint16;
        (*E).mpr_var_B = 256 as uint16;
        (*E).mpr_var_D = 1536 as uint16;
        (*E).mpr_var_A = (0xa8d97c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mpr_var_07 = 48 as uint16;
        (*E).mpr_var_03 = 1 as uint16;
    }
}
pub unsafe fn WalkingLavaSeahorse_Init(game: &mut Game) {
    let mut E: *mut Enemy_WalkingLavaSeahorse = Get_WalkingLavaSeahorse(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).wlse_var_04 = 0 as uint16;
    (*E).wlse_var_F = (*E).base.y_pos;
    (*E).wlse_var_02 = (*E).base.x_pos;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_DisableSamusColl as ::core::ffi::c_int) as uint16;
    (*E).base.current_instruction =
        addr_kWalkingLavaSeahorse_Ilist_DBE7 as ::core::ffi::c_int as uint16;
    (*E).wlse_var_A = (0xa8dd71 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    WalkingLavaSeahorse_Func_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    loop {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            ((*E).wlse_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).wlse_var_C as uint32,
        );
        if !(WalkingLavaSeahorse_Func_2(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        ) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    while !(Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (((*E).wlse_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).wlse_var_C as uint32)
            as int32,
        game,
    ) != 0)
    {
        WalkingLavaSeahorse_Func_2(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
    (*E).wlse_var_03 = (*E).base.y_pos;
    (*E).base.y_pos = (*E).wlse_var_F;
}
pub unsafe fn WalkingLavaSeahorse_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_WalkingLavaSeahorse =
        Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    (*E).wlse_var_B = -(12 as ::core::ffi::c_int) as uint16;
    (*E).wlse_var_C = 0 as uint16;
    (*E).wlse_var_D = 0 as uint16;
    (*E).wlse_var_E = 0 as uint16;
    (*E).wlse_var_00 = 0 as uint16;
    (*E).wlse_var_01 = 0x8000 as uint16;
}
pub unsafe fn WalkingLavaSeahorse_Func_2(mut k: uint16, game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_WalkingLavaSeahorse =
        Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).wlse_var_B,
        &raw mut (*E).wlse_var_C,
        ((*E).wlse_var_00 as uint32) << 16 as ::core::ffi::c_int | (*E).wlse_var_01 as uint32,
    );
    return (*E).wlse_var_B;
}
pub unsafe fn CallWalkingLavaSeahorseFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        11067084 => return,
        11066737 => {
            WalkingLavaSeahorse_Func_3(k, game);
            return;
        }
        11066822 => {
            WalkingLavaSeahorse_Func_4(k, game);
            return;
        }
        11066885 => {
            WalkingLavaSeahorse_Func_6(k, game);
            return;
        }
        11066955 => {
            WalkingLavaSeahorse_Func_7(k, game);
            return;
        }
        11067085 => {
            WalkingLavaSeahorse_Func_8(k, game);
            return;
        }
        11067116 => {
            WalkingLavaSeahorse_Func_9(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn WalkingLavaSeahorse_Main(game: &mut Game) {
    let mut E: *mut Enemy_WalkingLavaSeahorse = Get_WalkingLavaSeahorse(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallWalkingLavaSeahorseFunc(
        ((*E).wlse_var_A as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn WalkingLavaSeahorse_Func_3(mut k: uint16, game: &mut Game) {
    let mut v4: uint16 = 0;
    let mut v5: int16 = 0;
    let mut E: *mut Enemy_WalkingLavaSeahorse =
        Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    let mut xxx: uint16 = 0;
    let mut v3: uint16 = abs16(
        ((*E).wlse_var_03 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int) as uint16,
    );
    if v3 as ::core::ffi::c_int - 32 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 && {
        xxx = (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as uint16;
        ((abs16(xxx) as ::core::ffi::c_int - g_word_A8DCC7 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
    } {
        WalkingLavaSeahorse_Func_1(k, game);
        v4 = addr_stru_A8DC4B as ::core::ffi::c_int as uint16;
        v5 = -(2 as ::core::ffi::c_int) as int16;
        if xxx as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v4 = addr_stru_A8DCBB as ::core::ffi::c_int as uint16;
            v5 = 2 as int16;
        }
        (*E).wlse_var_D = v5 as uint16;
        (*E).base.current_instruction = v4;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).wlse_var_A = (0xa8ddc6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        QueueSfx2_Max6(0x5e as uint16, game);
    }
}
pub unsafe fn WalkingLavaSeahorse_Func_4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_WalkingLavaSeahorse =
        Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).wlse_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).wlse_var_C as uint32,
    );
    if WalkingLavaSeahorse_Func_2(k, game) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        sub_A8DDDE(k, game);
    }
}
pub unsafe fn sub_A8DDDE(mut k: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(k as ::core::ffi::c_int, game);
    (*v1).ai_var_A = (0xa8de05 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = addr_kWalkingLavaSeahorse_Ilist_DC51 as ::core::ffi::c_int as uint16;
    if (*v1).ai_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v2 = addr_kWalkingLavaSeahorse_Ilist_DCC1 as ::core::ffi::c_int as uint16;
    }
    (*v1).current_instruction = v2;
    (*v1).instruction_timer = 1 as uint16;
}
pub unsafe fn WalkingLavaSeahorse_Func_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_WalkingLavaSeahorse =
        Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    (*E).wlse_var_B = 0 as uint16;
    (*E).wlse_var_C = 0 as uint16;
    sub_A8DDDE(k, game);
}
pub unsafe fn WalkingLavaSeahorse_Func_6(mut k: uint16, game: &mut Game) {
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_WalkingLavaSeahorse =
        Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    if Enemy_MoveDown(
        k,
        (((*E).wlse_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).wlse_var_C as uint32)
            as int32,
        game,
    ) != 0
    {
        let mut t: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xaf6)
            as *mut uint16) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int;
        let mut v2: uint16 = addr_kWalkingLavaSeahorse_Ilist_DBE7 as ::core::ffi::c_int as uint16;
        v3 = -(2 as ::core::ffi::c_int) as int16;
        if t & 0x8000 as ::core::ffi::c_int == 0 {
            v3 = 2 as int16;
            v2 = addr_kWalkingLavaSeahorse_Ilist_DC57 as ::core::ffi::c_int as uint16;
        }
        (*E).wlse_var_D = v3 as uint16;
        (*E).base.current_instruction = v2;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).wlse_var_A = (0xa8de4b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).wlse_var_04 = 1 as uint16;
    } else {
        WalkingLavaSeahorse_Func_2(k, game);
    };
}
pub unsafe fn WalkingLavaSeahorse_Func_7(mut k: uint16, game: &mut Game) {
    Enemy_MoveDown(
        k,
        (2 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    );
    let mut E: *mut Enemy_WalkingLavaSeahorse =
        Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    if (abs16(
        ((*E).wlse_var_02 as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
    ) as ::core::ffi::c_int
        - g_word_A8DCCB as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).wlse_var_A = (0xa8decd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).wlse_var_B = -(4 as ::core::ffi::c_int) as uint16;
        (*E).wlse_var_C = 0 as uint16;
        let mut v4: uint16 = addr_kWalkingLavaSeahorse_Ilist_DC51 as ::core::ffi::c_int as uint16;
        if (*E).wlse_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            v4 = addr_kWalkingLavaSeahorse_Ilist_DCC1 as ::core::ffi::c_int as uint16;
        }
        (*E).base.current_instruction = v4;
        (*E).base.instruction_timer = 1 as uint16;
    } else {
        if (*E).wlse_var_04 != 0 {
            return;
        }
        let mut t: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xaf6)
            as *mut uint16) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int;
        let mut v3: uint16 = 0;
        if t & 0x8000 as ::core::ffi::c_int == 0 {
            v3 = addr_kWalkingLavaSeahorse_Ilist_DC73 as ::core::ffi::c_int as uint16;
            if (*E).wlse_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                return;
            }
        } else {
            v3 = addr_kWalkingLavaSeahorse_Ilist_DC03 as ::core::ffi::c_int as uint16;
            if (*E).wlse_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                return;
            }
        }
        (*E).base.current_instruction = v3;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).wlse_var_A = (0xa8decc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn WalkingLavaSeahorse_Func_8(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_WalkingLavaSeahorse =
        Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).wlse_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).wlse_var_C as uint32,
    );
    if WalkingLavaSeahorse_Func_2(k, game) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).wlse_var_A = (0xa8deec as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn WalkingLavaSeahorse_Func_9(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_WalkingLavaSeahorse =
        Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).wlse_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).wlse_var_C as uint32,
    );
    if ((*E).base.y_pos as ::core::ffi::c_int - (*E).wlse_var_F as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos = (*E).wlse_var_F;
        (*E).base.x_pos = (*E).wlse_var_02;
        (*E).wlse_var_A = (0xa8dd71 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        WalkingLavaSeahorse_Func_2(k, game);
    };
}
pub unsafe fn WalkingLavaSeahorse_Instr_4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WalkingLavaSeahorse_DF20(0 as uint16, game);
    return jp;
}
pub unsafe fn WalkingLavaSeahorse_DF20(mut a: uint16, game: &mut Game) {
    SpawnEprojWithGfx(
        a,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_loc_A89E90 as ::core::ffi::c_int as uint16,
        game,
    );
    QueueSfx2_Max6(0x3f as uint16, game);
}
pub unsafe fn WalkingLavaSeahorse_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WalkingLavaSeahorse_DF20(2 as uint16, game);
    return jp;
}
pub unsafe fn WalkingLavaSeahorse_Instr_5(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WalkingLavaSeahorse_DF20(4 as uint16, game);
    return jp;
}
pub unsafe fn WalkingLavaSeahorse_Instr_6(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_WalkingLavaSeahorse = Get_WalkingLavaSeahorse(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).wlse_var_A = (0xa8de4b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    v3 = (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int) as int16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v3 = 2 as int16;
    }
    (*E).wlse_var_04 = v3 as uint16;
    if (*E).wlse_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return addr_kWalkingLavaSeahorse_Ilist_DC57 as ::core::ffi::c_int as uintptr_t
            as *const uint16;
    }
    return addr_kWalkingLavaSeahorse_Ilist_DBE7 as ::core::ffi::c_int as uintptr_t
        as *const uint16;
}
pub unsafe fn WalkingLavaSeahorse_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_WalkingLavaSeahorse = Get_WalkingLavaSeahorse(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut wlse_var_04: uint16 = (*E).wlse_var_04;
    if wlse_var_04 != 0 {
        (*E).wlse_var_04 = (wlse_var_04 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    }
    return WalkingLavaSeahorse_Instr_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        jp,
        game,
    );
}
pub unsafe fn WalkingLavaSeahorse_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut wlse_var_D: int16 = 0;
    let mut E: *mut Enemy_WalkingLavaSeahorse = Get_WalkingLavaSeahorse(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        ((*E).wlse_var_D as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        jp = addr_stru_A8DBE9 as ::core::ffi::c_int as uintptr_t as *const uint16;
        wlse_var_D = (*E).wlse_var_D as int16;
        if (wlse_var_D as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            jp = addr_stru_A8DC59 as ::core::ffi::c_int as uintptr_t as *const uint16;
        }
        (*E).wlse_var_D = -(wlse_var_D as ::core::ffi::c_int) as uint16;
    } else {
        EnemyFunc_C8AD(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
    return jp;
}
pub unsafe fn WalkingLavaSeahorse_Func_10(game: &mut Game) {
    NormalEnemyShotAi(game);
}
pub unsafe fn WreckedShipOrbs_Init(game: &mut Game) {
    let mut E: *mut Enemy_WreckedShipOrbs = Get_WreckedShipOrbs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction = *(game.g_rom.offset(
        (((0xa8e380 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8e380 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((*E).wsos_parameter_1 as isize);
    let mut v1: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * (*E).wsos_parameter_2 as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).wsos_var_01 = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize);
    (*E).wsos_var_00 = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v1 + 1 as ::core::ffi::c_int) as isize);
    (*E).wsos_var_03 = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v1 + 2 as ::core::ffi::c_int) as isize);
    (*E).wsos_var_02 = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v1 + 3 as ::core::ffi::c_int) as isize);
}
pub unsafe fn CallWreckedShipOrbsA(mut ea: uint32, game: &mut Game) {
    match ea {
        11068421 => {
            WreckedShipOrbs_Func_3(game);
            return;
        }
        11068452 => {
            WreckedShipOrbs_Func_4(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CallWreckedShipOrbsB(mut ea: uint32, game: &mut Game) {
    match ea {
        11068483 => {
            WreckedShipOrbs_Func_5(game);
            return;
        }
        11068514 => {
            WreckedShipOrbs_Func_6(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn WreckedShipOrbs_Main(game: &mut Game) {
    WreckedShipOrbs_Func_1(game);
    WreckedShipOrbs_Func_2(game);
    let mut E: *mut Enemy_WreckedShipOrbs = Get_WreckedShipOrbs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallWreckedShipOrbsA(
        ((*E).wsos_var_A as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
        game,
    );
    CallWreckedShipOrbsB(
        ((*E).wsos_var_B as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn WreckedShipOrbs_Func_1(game: &mut Game) {
    let mut E: *mut Enemy_WreckedShipOrbs = Get_WreckedShipOrbs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).wsos_var_A = (0xa8e405 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if GetSamusEnemyDelta_Y(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).wsos_var_A = (0xa8e424 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn WreckedShipOrbs_Func_2(game: &mut Game) {
    let mut E: *mut Enemy_WreckedShipOrbs = Get_WreckedShipOrbs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).wsos_var_B = (0xa8e443 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if GetSamusEnemyDelta_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).wsos_var_B = (0xa8e462 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn WreckedShipOrbs_Func_3(game: &mut Game) {
    let mut E: *mut Enemy_WreckedShipOrbs = Get_WreckedShipOrbs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).wsos_var_03 as uint32) << 16 as ::core::ffi::c_int | (*E).wsos_var_02 as uint32,
    );
}
pub unsafe fn WreckedShipOrbs_Func_4(game: &mut Game) {
    let mut E: *mut Enemy_WreckedShipOrbs = Get_WreckedShipOrbs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).wsos_var_01 as uint32) << 16 as ::core::ffi::c_int | (*E).wsos_var_00 as uint32,
    );
}
pub unsafe fn WreckedShipOrbs_Func_5(game: &mut Game) {
    let mut E: *mut Enemy_WreckedShipOrbs = Get_WreckedShipOrbs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).wsos_var_03 as uint32) << 16 as ::core::ffi::c_int | (*E).wsos_var_02 as uint32,
    );
}
pub unsafe fn WreckedShipOrbs_Func_6(game: &mut Game) {
    let mut E: *mut Enemy_WreckedShipOrbs = Get_WreckedShipOrbs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).wsos_var_01 as uint32) << 16 as ::core::ffi::c_int | (*E).wsos_var_00 as uint32,
    );
}
pub unsafe fn WreckedShipSpark_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_WreckedShipSpark = Get_WreckedShipSpark(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn WreckedShipSpark_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_WreckedShipSpark = Get_WreckedShipSpark(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
    return jp;
}
unsafe fn WreckedShipSpark_Func_4(mut k: uint16, mut r18: uint16, game: &mut Game) {
    let mut wssk_var_E: int16 = 0;
    let mut E: *mut Enemy_WreckedShipSpark = Get_WreckedShipSpark(k as ::core::ffi::c_int, game);
    wssk_var_E = (*E).wssk_var_E as int16;
    if (wssk_var_E as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        wssk_var_E = ((NextRandom(game) as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int)
            + 4 as ::core::ffi::c_int) as int16;
    }
    (*E).wssk_var_F = (r18 as ::core::ffi::c_int + wssk_var_E as ::core::ffi::c_int) as uint16;
}
pub unsafe fn WreckedShipSpark_Init(game: &mut Game) {
    let mut E: *mut Enemy_WreckedShipSpark = Get_WreckedShipSpark(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int =
        (*E).wssk_parameter_1 as ::core::ffi::c_int & 3 as ::core::ffi::c_int;
    (*E).wssk_var_B = *(game.g_rom.offset(
        (((0xa8e688 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8e688 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v2 as isize);
    (*E).wssk_var_E = (*E).wssk_parameter_2;
    WreckedShipSpark_Func_4(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0 as uint16,
        game,
    );
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.current_instruction = *(game.g_rom.offset(
        (((0xa8e682 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa8e682 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v2 as isize);
    (*E).base.timer = 0 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | *((&raw mut (*(gVramWriteEntry
                as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
                0 as ::core::ffi::c_int,
                game,
            )
            .offset(6))
            .vram_dst as *mut uint8)
                .offset(1) as *mut uint16) as ::core::ffi::c_int)
            as uint16;
    }
}
pub unsafe fn CallWreckedShipSpark(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        11069077 => {
            WreckedShipSpark_Func_1(k, game);
            return;
        }
        11069111 => {
            WreckedShipSpark_Func_2(k, game);
            return;
        }
        11069148 => {
            WreckedShipSpark_Func_3(k, game);
            return;
        }
        11069076 => return,
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn WreckedShipSpark_Main(game: &mut Game) {
    let mut E: *mut Enemy_WreckedShipSpark = Get_WreckedShipSpark(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallWreckedShipSpark(
        ((*E).wssk_var_B as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn WreckedShipSpark_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_WreckedShipSpark = Get_WreckedShipSpark(k as ::core::ffi::c_int, game);
    if (*E).wssk_var_F as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).wssk_var_B = (0xa8e6b7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction =
            addr_kWreckedShipSpark_Ilist_E5A7 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        WreckedShipSpark_Func_4(k, 0 as uint16, game);
    } else {
        (*E).wssk_var_F = (*E).wssk_var_F.wrapping_sub(1);
    };
}
pub unsafe fn WreckedShipSpark_Func_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_WreckedShipSpark = Get_WreckedShipSpark(k as ::core::ffi::c_int, game);
    if (*E).wssk_var_F as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).wssk_var_B = (0xa8e695 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction =
            addr_kWreckedShipSpark_Ilist_E5E5 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        WreckedShipSpark_Func_4(k, 8 as uint16, game);
    } else {
        (*E).wssk_var_F = (*E).wssk_var_F.wrapping_sub(1);
    };
}
pub unsafe fn WreckedShipSpark_Func_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_WreckedShipSpark = Get_WreckedShipSpark(k as ::core::ffi::c_int, game);
    if (*E).wssk_var_F as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
        SpawnEprojWithGfx(
            0 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            addr_kEproj_Sparks as ::core::ffi::c_int as uint16,
            game,
        );
        WreckedShipSpark_Func_4(v2, 0 as uint16, game);
    } else {
        (*E).wssk_var_F = (*E).wssk_var_F.wrapping_sub(1);
    };
}
pub unsafe fn WreckedShipSpark_Shot(game: &mut Game) {
    let ref mut fresh14 = *((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16) as isize);
    *fresh14 = (*fresh14 as ::core::ffi::c_int & !(0x10 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn BlueBrinstarFaceBlock_Init(game: &mut Game) {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v0).current_instruction =
        addr_kBlueBrinstarFaceBlock_Ilist_E828 as ::core::ffi::c_int as uint16;
    let mut v1: uint16 = (0xa8e86e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16) as ::core::ffi::c_int
        & 4 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v1 = (0xa8804c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).addr = v1 as VoidP;
    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).bank
        as *mut uint16) = 168 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(0) =
        ((16 as ::core::ffi::c_int * (*v0).palette_index as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(2) = 16 as uint16;
    (*v0).parameter_2 = (((*v0).parameter_2 as ::core::ffi::c_int & 1 as ::core::ffi::c_int)
        >> 2 as ::core::ffi::c_int
        | ((*v0).parameter_2 as ::core::ffi::c_int & 1 as ::core::ffi::c_int)
            << 15 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn BlueBrinstarFaceBlock_Func_1(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x797) as *mut uint16) == 0 && {
        let ref mut fresh0 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(2);
        *fresh0 = (*fresh0).wrapping_sub(1);
        *fresh0 == 0
    } {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(2) =
            16 as uint16;
        let mut v0: uint16 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(0);
        let mut v1: uint16 = (8 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(1)
                as ::core::ffi::c_int) as uint16;
        let mut n: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                ((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 137 as ::core::ffi::c_int)
                    as isize,
            ) = *(game.g_rom.offset(
                (((0xa8e7cc as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa8e7cc as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(1) =
            (*(((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(1)
                as *mut uint16 as *mut uint8)
                .offset(0) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                & 7 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn BlueBrinstarFaceBlock_Main(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16) as ::core::ffi::c_int
        & 4 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).addr =
            (0xa8e86e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP;
        let mut v1: *mut EnemyData = gEnemyData(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(0) =
            ((16 as ::core::ffi::c_int * (*v1).palette_index as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint16;
        if (*v1).ai_var_A == 0 {
            let mut SamusEnemyDelta_Y: uint16 = GetSamusEnemyDelta_Y(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                game,
            );
            if ((Abs16(SamusEnemyDelta_Y) as ::core::ffi::c_int
                - (*v1).parameter_1 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                let mut SamusEnemyDelta_X: uint16 = GetSamusEnemyDelta_X(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                    game,
                );
                (*v1).ai_var_B = SamusEnemyDelta_X;
                if ((Abs16(SamusEnemyDelta_X) as ::core::ffi::c_int
                    - (*v1).parameter_1 as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
                    && (*v1).ai_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                        != (*v1).parameter_2 as ::core::ffi::c_int
                {
                    let mut v4: uint16 =
                        addr_kBlueBrinstarFaceBlock_Ilist_E80C as ::core::ffi::c_int as uint16;
                    if (*v1).ai_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        v4 = addr_kBlueBrinstarFaceBlock_Ilist_E81A as ::core::ffi::c_int as uint16;
                    }
                    (*v1).current_instruction = v4;
                    (*v1).instruction_timer = 1 as uint16;
                    (*v1).ai_var_A = 1 as uint16;
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16)
                        .offset(2) = 16 as uint16;
                }
            }
        }
    }
}
pub unsafe fn BlueBrinstarFaceBlock_Shot(game: &mut Game) {
    let ref mut fresh1 = *((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16) as isize);
    *fresh1 = (*fresh1 as ::core::ffi::c_int & !(0x10 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn KiHunter_Init(game: &mut Game) {
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_DisableSamusColl as ::core::ffi::c_int) as uint16;
    (*E).khr_var_14 = 0 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction = addr_kKiHunter_Ilist_E9FA as ::core::ffi::c_int as uint16;
    (*E).khr_var_F = 0 as uint16;
    (*E).khr_var_A = (0xa8f268 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).khr_var_08 = 0 as uint16;
    (*E).khr_var_09 = 1 as uint16;
    (*E).khr_var_06 = 0 as uint16;
    (*E).khr_var_07 = -(1 as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 =
        ((*E).base.y_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
    (*E).khr_var_0A = v1;
    (*E).khr_var_0B = (v1 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    (*E).khr_var_0C = (*E).base.x_pos;
    (*E).khr_var_0D = (*E).base.y_pos;
    if (*E).khr_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).khr_var_14 = 1 as uint16;
        (*E).khr_var_A = (0xa8f55a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).khr_var_08 = 0 as uint16;
        (*E).khr_var_09 = 1 as uint16;
    }
}
pub unsafe fn KiHunterWings_Init(game: &mut Game) {
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_DisableSamusColl as ::core::ffi::c_int) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction = addr_kKiHunter_Ilist_EA4E as ::core::ffi::c_int as uint16;
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xe54)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).base.y_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v2 + 93 as ::core::ffi::c_int) as isize);
    (*E).base.x_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v2 + 91 as ::core::ffi::c_int) as isize);
    (*E).khr_var_A = (0xa8f6f3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.palette_index = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v2 + 105 as ::core::ffi::c_int) as isize);
    (*E).base.vram_tiles_index = *((&raw mut game.g_ram as *mut uint8).offset(0xe84)
        as *mut uint16)
        .offset((v2 + 106 as ::core::ffi::c_int) as isize);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
        .offset((v2 + 6 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CallKiHunterFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        11072104 => {
            KiHunter_Func_1(k, game);
            return;
        }
        11072440 => {
            KiHunter_Func_2(k, game);
            return;
        }
        11072749 => {
            KiHunter_Func_3(k, game);
            return;
        }
        11072858 => {
            KiHunter_Func_4(k, game);
            return;
        }
        11072907 => {
            KiHunter_Func_5(k, game);
            return;
        }
        11073008 => {
            KiHunter_Func_6(k, game);
            return;
        }
        11073163 => {
            KiHunter_Func_7(k, game);
            return;
        }
        11073203 => {
            KiHunter_Func_8(k, game);
            return;
        }
        11073267 => {
            KiHunter_Func_9(k, game);
            return;
        }
        11073487 => {
            KiHunter_Func_10(k, game);
            return;
        }
        11072995 => return,
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn KiHunter_Main(game: &mut Game) {
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallKiHunterFunc(
        ((*E).khr_var_A as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn KiHunterWings_Main(game: &mut Game) {
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallKiHunterFunc(
        ((*E).khr_var_A as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn KiHunter_Func_1(mut k: uint16, game: &mut Game) {
    let mut current_block: u64;
    let mut v4: int16 = 0;
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    if Enemy_MoveDown(
        k,
        (((*E).khr_var_09 as uint32) << 16 as ::core::ffi::c_int | (*E).khr_var_08 as uint32)
            as int32,
        game,
    ) != 0
    {
        v4 = -((*E).khr_var_09 as ::core::ffi::c_int) as int16;
        current_block = 8515828400728868193;
    } else {
        let mut y_pos: uint16 = (*E).base.y_pos;
        if ((y_pos as ::core::ffi::c_int - (*E).khr_var_0A as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            v4 = 1 as int16;
            current_block = 8515828400728868193;
        } else if ((y_pos as ::core::ffi::c_int - (*E).khr_var_0B as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            current_block = 3218563456939568490;
        } else {
            v4 = -(1 as ::core::ffi::c_int) as int16;
            current_block = 8515828400728868193;
        }
    }
    match current_block {
        8515828400728868193 => {
            (*E).khr_var_09 = v4 as uint16;
        }
        _ => {}
    }
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).khr_var_07 as uint32) << 16 as ::core::ffi::c_int | (*E).khr_var_06 as uint32)
            as int32,
        game,
    ) != 0
    {
        let mut v5: uint16 = 0 as uint16;
        let mut v6: bool = -((*E).khr_var_07 as ::core::ffi::c_int) & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
        (*E).khr_var_07 = -((*E).khr_var_07 as ::core::ffi::c_int) as uint16;
        if !v6 {
            v5 = 4 as uint16;
        }
        let mut v7: ::core::ffi::c_int = v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        (*E).base.current_instruction = *(game.g_rom.offset(
            (((0xa8f3b0 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa8f3b0 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v7 as isize);
        let mut E1: *mut Enemy_KiHunter =
            Get_KiHunter(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
        (*E1).base.current_instruction = *(game.g_rom.offset(
            (((0xa8f3b0 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa8f3b0 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v7 + 1 as ::core::ffi::c_int) as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E1).base.instruction_timer = 1 as uint16;
    }
    EnemyFunc_C8AD(k, game);
    let mut r18: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
        as ::core::ffi::c_int
        - (*E).base.x_pos as ::core::ffi::c_int) as uint16;
    let mut r20: uint16 = abs16(r18);
    if ((r20 as ::core::ffi::c_int - g_word_A8F180 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
            - (*E).base.y_pos as ::core::ffi::c_int
            - 32 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        let mut r24: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
            as ::core::ffi::c_int
            - (*E).base.y_pos as ::core::ffi::c_int) as uint16;
        let mut v9: uint16 = 0;
        if r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            (*E).khr_var_00 = -(2 as ::core::ffi::c_int) as uint16;
            (*E).khr_var_02 = 0 as uint16;
            (*E).khr_var_03 = 0 as uint16;
            (*E).khr_var_04 = -(1 as ::core::ffi::c_int) as uint16;
            (*E).khr_var_07 = -(1 as ::core::ffi::c_int) as uint16;
            (*E).khr_var_05 = -(8192 as ::core::ffi::c_int) as uint16;
            (*E).khr_var_F = 255 as uint16;
            (*E).khr_var_0E = 240 as uint16;
            v9 = 0 as uint16;
        } else {
            (*E).khr_var_00 = 2 as uint16;
            (*E).khr_var_02 = 0 as uint16;
            (*E).khr_var_03 = 0 as uint16;
            (*E).khr_var_04 = 0 as uint16;
            (*E).khr_var_05 = 0x2000 as uint16;
            (*E).khr_var_F = 128 as uint16;
            (*E).khr_var_07 = 1 as uint16;
            (*E).khr_var_0E = 144 as uint16;
            v9 = 4 as uint16;
        }
        let mut v10: ::core::ffi::c_int = v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        (*E).base.current_instruction = *(game.g_rom.offset(
            (((0xa8f3b0 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa8f3b0 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v10 as isize);
        let mut E1_0: *mut Enemy_KiHunter =
            Get_KiHunter(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
        (*E1_0).base.current_instruction = *(game.g_rom.offset(
            (((0xa8f3b0 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa8f3b0 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v10 + 1 as ::core::ffi::c_int) as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E1_0).base.instruction_timer = 1 as uint16;
        (*E).khr_var_B =
            (r18 as ::core::ffi::c_int + (*E).base.x_pos as ::core::ffi::c_int) as uint16;
        (*E).khr_var_C = (*E).base.y_pos;
        (*E).khr_var_A = (0xa8f3b8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).khr_var_12 = r24;
        (*E).khr_var_11 = r20;
        (*E).khr_var_10 = 0 as uint16;
    }
}
pub unsafe fn KiHunter_Func_2(mut k: uint16, game: &mut Game) {
    let mut current_block: u64;
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    if (*E).khr_var_04 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if (((*E).khr_var_F as ::core::ffi::c_int - (*E).khr_var_0E as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            current_block = 17315748505249742599;
        } else {
            current_block = 17179679302217393232;
        }
    } else if ((*E).khr_var_F as ::core::ffi::c_int - (*E).khr_var_0E as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        current_block = 17315748505249742599;
    } else {
        current_block = 17179679302217393232;
    }
    match current_block {
        17179679302217393232 => {
            if (*E).khr_var_10 == 0 {
                (*E).khr_var_10 = 1 as uint16;
                let mut v2: uint16 = addr_kKiHunter_Ilist_EA32 as ::core::ffi::c_int as uint16;
                if (*E).khr_var_04 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    v2 = addr_kKiHunter_Ilist_EA08 as ::core::ffi::c_int as uint16;
                }
                (*E).base.current_instruction = v2;
                (*E).base.instruction_timer = 1 as uint16;
            }
        }
        _ => {}
    }
    if (*E).khr_var_04 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        AddToHiLo(
            &raw mut (*E).khr_var_02,
            &raw mut (*E).khr_var_03,
            ((*E).khr_var_04 as uint32) << 16 as ::core::ffi::c_int | (*E).khr_var_05 as uint32,
        );
        if ((*E).khr_var_02 as ::core::ffi::c_int - (*E).khr_var_00 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            (*E).khr_var_02 = (*E).khr_var_00;
        }
        (*E).khr_var_F = ((*E).khr_var_F as ::core::ffi::c_int
            + (*E).khr_var_02 as ::core::ffi::c_int) as uint16;
        if (*E).khr_var_F as ::core::ffi::c_int - 256 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).khr_var_A =
                (0xa8f268 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            return;
        }
    } else {
        AddToHiLo(
            &raw mut (*E).khr_var_02,
            &raw mut (*E).khr_var_03,
            ((*E).khr_var_04 as uint32) << 16 as ::core::ffi::c_int | (*E).khr_var_05 as uint32,
        );
        if (((*E).khr_var_02 as ::core::ffi::c_int - (*E).khr_var_00 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            (*E).khr_var_02 = (*E).khr_var_00;
        }
        (*E).khr_var_F = ((*E).khr_var_F as ::core::ffi::c_int
            + (*E).khr_var_02 as ::core::ffi::c_int) as uint16;
        if (*E).khr_var_F as ::core::ffi::c_int - 128 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            (*E).khr_var_A =
                (0xa8f268 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            return;
        }
    }
    let mut r20: uint16 = ((*E).khr_var_B as ::core::ffi::c_int
        + CosineMult8bit((*E).khr_var_F, (*E).khr_var_11, game) as ::core::ffi::c_int
        - (*E).base.x_pos as ::core::ffi::c_int) as uint16;
    if Enemy_MoveRight_IgnoreSlopes(k, (r20 as int16 as int32) << 16 as ::core::ffi::c_int, game)
        != 0
    {
        if (*E).khr_var_04 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*E).khr_var_06 = 0 as uint16;
            (*E).khr_var_07 = -(1 as ::core::ffi::c_int) as uint16;
        } else {
            (*E).khr_var_06 = 0 as uint16;
            (*E).khr_var_07 = 1 as uint16;
        }
        current_block = 14751541640656221110;
    } else {
        EnemyFunc_C8AD(k, game);
        r20 = ((*E).khr_var_C as ::core::ffi::c_int
            + SineMult8bit((*E).khr_var_F, (*E).khr_var_12, game) as ::core::ffi::c_int
            - (*E).base.y_pos as ::core::ffi::c_int) as uint16;
        if Enemy_MoveDown(k, (r20 as int16 as int32) << 16 as ::core::ffi::c_int, game) != 0 {
            current_block = 14751541640656221110;
        } else {
            current_block = 7828949454673616476;
        }
    }
    match current_block {
        14751541640656221110 => {
            (*E).khr_var_A =
                (0xa8f4ed as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).khr_var_08 = 0 as uint16;
            (*E).khr_var_09 = -(1 as ::core::ffi::c_int) as uint16;
        }
        _ => {}
    };
}
pub unsafe fn KiHunter_Func_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).khr_var_07 as uint32) << 16 as ::core::ffi::c_int | (*E).khr_var_06 as uint32)
            as int32,
        game,
    ) as ::core::ffi::c_int
        != 0
        || {
            EnemyFunc_C8AD(k, game);
            Enemy_MoveDown(
                k,
                (((*E).khr_var_09 as uint32) << 16 as ::core::ffi::c_int
                    | (*E).khr_var_08 as uint32) as int32,
                game,
            ) as ::core::ffi::c_int
                != 0
        }
        || (((*E).base.y_pos as ::core::ffi::c_int - (*E).khr_var_0D as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
    {
        (*E).khr_var_A = (0xa8f268 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn KiHunter_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut result: uint16 = addr_kKiHunter_Ilist_E9FA as ::core::ffi::c_int as uint16;
    let mut r18: uint16 = addr_kKiHunter_Ilist_EA4E as ::core::ffi::c_int as uint16;
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    if (*E).khr_var_07 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        result = addr_kKiHunter_Ilist_EA24 as ::core::ffi::c_int as uint16;
        r18 = addr_kKiHunter_Ilist_EA5E as ::core::ffi::c_int as uint16;
    }
    (*E).base.current_instruction = r18;
    (*E).base.instruction_timer = 1 as uint16;
    let mut E1: *mut Enemy_KiHunter =
        Get_KiHunter(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    if (*E1).khr_var_A as ::core::ffi::c_int
        == 0xa8f6f3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        (*E1).base.current_instruction = r18;
        (*E1).base.instruction_timer = 1 as uint16;
    }
    return result as uintptr_t as *const uint16;
}
pub unsafe fn KiHunter_Func_4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    if Enemy_MoveDown(
        k,
        (((*E).khr_var_09 as uint32) << 16 as ::core::ffi::c_int | (*E).khr_var_08 as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).khr_var_A = (0xa8f58b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        AddToHiLo(
            &raw mut (*E).khr_var_09,
            &raw mut (*E).khr_var_08,
            (g_word_A8F184 as uint32) << 16 as ::core::ffi::c_int | g_word_A8F182 as uint32,
        );
    };
}
pub unsafe fn KiHunter_Func_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    (*E).khr_var_A = addr_locret_A8F5E3 as ::core::ffi::c_int as uint16;
    (*E).khr_var_08 = 0 as uint16;
    (*E).khr_var_09 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int)
        - 8 as ::core::ffi::c_int) as uint16;
    if ((*E).base.x_pos as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).khr_var_06 = 0 as uint16;
        (*E).khr_var_07 = -(2 as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction = addr_kKiHunter_Ilist_EA8A as ::core::ffi::c_int as uint16;
    } else {
        (*E).khr_var_06 = 0 as uint16;
        (*E).khr_var_07 = 2 as uint16;
        (*E).base.current_instruction = addr_kKiHunter_Ilist_EAA6 as ::core::ffi::c_int as uint16;
    }
    (*E).base.instruction_timer = 1 as uint16;
}
pub unsafe fn KiHunter_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_KiHunter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .khr_var_A = (0xa8f5f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn KiHunter_Func_6(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    if Enemy_MoveDown(
        k,
        (((*E).khr_var_09 as uint32) << 16 as ::core::ffi::c_int | (*E).khr_var_08 as uint32)
            as int32,
        game,
    ) != 0
    {
        if (*E).khr_var_09 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).khr_var_09 = 1 as uint16;
        } else {
            (*E).khr_var_08 = 0 as uint16;
            (*E).khr_var_09 = -(4 as ::core::ffi::c_int) as uint16;
            (*E).khr_var_A = addr_locret_A8F5E3 as ::core::ffi::c_int as uint16;
            (*E).khr_var_0F = 12 as uint16;
            let mut v4: uint16 = addr_kKiHunter_Ilist_EAC2 as ::core::ffi::c_int as uint16;
            if (*E).base.current_instruction as ::core::ffi::c_int + 0x155a as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v4 = addr_kKiHunter_Ilist_EADA as ::core::ffi::c_int as uint16;
            }
            (*E).base.current_instruction = v4;
            (*E).base.instruction_timer = 1 as uint16;
        }
    } else if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).khr_var_07 as uint32) << 16 as ::core::ffi::c_int | (*E).khr_var_06 as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).khr_var_07 = -((*E).khr_var_07 as ::core::ffi::c_int) as uint16;
    } else {
        EnemyFunc_C8AD(k, game);
        AddToHiLo(
            &raw mut (*E).khr_var_09,
            &raw mut (*E).khr_var_08,
            (g_word_A8F184 as uint32) << 16 as ::core::ffi::c_int | g_word_A8F182 as uint32,
        );
    };
}
pub unsafe fn KiHunter_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_KiHunter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .khr_var_A = (0xa8f68b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn KiHunter_Func_7(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    let mut v2: uint16 =
        ((*E).khr_var_0F as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E).khr_var_0F = v2;
    if v2 == 0 {
        let mut v3: uint16 =
            (0xa8f58b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut v4: uint16 = abs16(
            ((*E).base.x_pos as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int) as uint16,
        );
        if v4 as ::core::ffi::c_int - 96 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            v3 = (0xa8f6b3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        (*E).khr_var_A = v3;
    }
}
pub unsafe fn KiHunter_Func_8(mut k: uint16, game: &mut Game) {
    let mut v1: uint16 = addr_kKiHunter_Ilist_EAF2 as ::core::ffi::c_int as uint16;
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    if (((*E).base.x_pos as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        v1 = addr_kKiHunter_Ilist_EB10 as ::core::ffi::c_int as uint16;
    }
    (*E).base.current_instruction = v1;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).khr_var_A = addr_locret_A8F5E3 as ::core::ffi::c_int as uint16;
}
pub unsafe fn KiHunter_Instr_4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    sub_A8F6DC(k, addr_loc_A8CF18 as ::core::ffi::c_int as uint16, game);
    return jp;
}
pub unsafe fn KiHunter_Instr_5(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    sub_A8F6DC(k, addr_loc_A8CF26 as ::core::ffi::c_int as uint16, game);
    return jp;
}
pub unsafe fn sub_A8F6DC(mut k: uint16, mut j: uint16, game: &mut Game) {
    QueueSfx2_Max6(0x4c as uint16, game);
    SpawnEprojWithGfx(
        0 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        j,
        game,
    );
    (*Get_KiHunter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .khr_var_0F = 24 as uint16;
}
pub unsafe fn KiHunter_Func_9(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    (*E).base.x_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 91 as ::core::ffi::c_int) as isize);
    (*E).base.y_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 93 as ::core::ffi::c_int) as isize);
}
pub unsafe fn KiHunter_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    let mut old_cur_enemy_index: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
    let mut EK: *mut Enemy_KiHunter = Get_KiHunter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut E1: *mut Enemy_KiHunter = Get_KiHunter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    let mut health: uint16 = (*EK).base.health;
    if health != 0 {
        if health as ::core::ffi::c_int == (*E1).khr_parameter_1 as ::core::ffi::c_int
            || ((health as ::core::ffi::c_int - (*E1).khr_parameter_1 as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
        {
            if (*EK).khr_var_14 == 0 {
                (*EK).khr_var_14 = 1 as uint16;
                (*EK).khr_var_A =
                    (0xa8f55a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                (*EK).khr_var_08 = 0 as uint16;
                (*EK).khr_var_09 = 1 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) =
                    (old_cur_enemy_index as ::core::ffi::c_int + 64 as ::core::ffi::c_int)
                        as uint16;
                if (*E1).khr_var_A as ::core::ffi::c_int
                    != 0xa8f7cf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
                {
                    (*E1).khr_var_07 = (*E1).base.y_pos;
                    (*E1).khr_var_08 = (*E1).base.x_pos;
                    KiHunter_Func_17(game);
                    KiHunter_Func_12(game);
                    KiHunter_Func_13(game);
                    (*E1).khr_var_F = -(8192 as ::core::ffi::c_int) as uint16;
                    (*E1).khr_var_A =
                        (0xa8f7cf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                    (*E1).khr_var_00 =
                        (0xa8f7db as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                    (*E1).khr_var_06 = ((*E1).khr_var_07 as ::core::ffi::c_int
                        - (*E1).khr_var_0B as ::core::ffi::c_int)
                        as uint16;
                    (*E1).khr_var_05 = (*E1).base.x_pos;
                    (*E1).khr_var_B = (*E1).khr_var_0A;
                    (*E1).base.current_instruction =
                        addr_kKiHunter_Ilist_EA7E as ::core::ffi::c_int as uint16;
                    (*E1).base.spritemap_pointer =
                        addr_kSpritemap_Nothing_A8 as ::core::ffi::c_int as uint16;
                    (*E1).base.instruction_timer = 1 as uint16;
                    (*E1).base.timer = 0 as uint16;
                    (*E1).base.properties = ((*E1).base.properties as ::core::ffi::c_int
                        | kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int)
                        as uint16;
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) =
                    old_cur_enemy_index;
            }
        } else {
            (*E1).base.ai_handler_bits = (*EK).base.ai_handler_bits;
            (*E1).base.frozen_timer = (*EK).base.frozen_timer;
            (*E1).base.invincibility_timer = (*EK).base.invincibility_timer;
            (*E1).base.flash_timer = (*EK).base.flash_timer;
        }
    } else {
        (*E1).base.properties = 512 as uint16;
    };
}
pub unsafe fn CallKiHunterBFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        11073499 => {
            KiHunter_Func_11(game);
            return;
        }
        11073709 => {
            KiHunter_Func_14(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn KiHunter_Func_10(mut k: uint16, game: &mut Game) {
    let mut r18: uint16 = (*Get_KiHunter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .khr_var_00;
    CallKiHunterBFunc(
        (r18 as ::core::ffi::c_int | 0xa80000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn KiHunter_Func_11(game: &mut Game) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).khr_var_F = ((*E).khr_var_F as ::core::ffi::c_int
        + *(((game.g_rom.offset(
            (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(2) as *mut uint16 as *mut uint8)
            .offset(
                (8 as ::core::ffi::c_int
                    * *(&raw mut (*E).khr_var_B as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ) as ::core::ffi::c_int) as isize,
            )
            .offset(1) as *mut uint16) as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = ((*E).khr_var_06 as ::core::ffi::c_int
        + SineMult8bit(
            *(&raw mut (*E).khr_var_F as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as uint16,
            g_byte_A8F186 as uint16,
            game,
        ) as ::core::ffi::c_int
        - (*E).khr_var_04 as ::core::ffi::c_int) as uint16;
    (*E).base.x_pos = ((*E).khr_var_05 as ::core::ffi::c_int
        + CosineMult8bit(
            *(&raw mut (*E).khr_var_F as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as uint16,
            g_byte_A8F186 as uint16,
            game,
        ) as ::core::ffi::c_int
        - (*E).khr_var_03 as ::core::ffi::c_int) as uint16;
    if (*E).khr_var_F as ::core::ffi::c_int + 0x4000 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        KiHunter_Func_16(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    } else {
        v2 = ((*E).khr_var_B as ::core::ffi::c_int - 384 as ::core::ffi::c_int) as int16;
        if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            v2 = 256 as int16;
        }
        (*E).khr_var_B = v2 as uint16;
    };
}
pub unsafe fn KiHunter_Func_12(game: &mut Game) {
    let mut v1: uint16 = CosineMult8bit(0xe0 as uint16, g_byte_A8F186 as uint16, game);
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).khr_var_03 = v1;
    (*E).khr_var_04 = SineMult8bit(0xe0 as uint16, g_byte_A8F186 as uint16, game);
}
pub unsafe fn KiHunter_Func_13(game: &mut Game) {
    let mut v1: uint16 = CosineMult8bit(0xa0 as uint16, g_byte_A8F186 as uint16, game);
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).khr_var_01 = v1;
    (*E).khr_var_02 = SineMult8bit(0xa0 as uint16, g_byte_A8F186 as uint16, game);
}
pub unsafe fn KiHunter_Func_14(game: &mut Game) {
    let mut khr_var_07: int16 = 0;
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).khr_var_F = ((*E).khr_var_F as ::core::ffi::c_int
        + *((game.g_rom.offset(
            (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16 as *mut uint8)
            .offset(
                (8 as ::core::ffi::c_int
                    * *(&raw mut (*E).khr_var_B as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ) as ::core::ffi::c_int) as isize,
            )
            .offset(1) as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut r20: uint16 = ((*E).khr_var_06 as ::core::ffi::c_int
        + SineMult8bit(
            *(&raw mut (*E).khr_var_F as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as uint16,
            g_byte_A8F186 as uint16,
            game,
        ) as ::core::ffi::c_int
        - (*E).khr_var_02 as ::core::ffi::c_int
        - (*E).base.y_pos as ::core::ffi::c_int) as uint16;
    if Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (r20 as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
        (*E).base.x_pos = (*E).khr_var_08;
        khr_var_07 = (*E).khr_var_07 as int16;
        (*E).base.y_pos = khr_var_07 as uint16;
    } else {
        (*E).base.x_pos = ((*E).khr_var_05 as ::core::ffi::c_int
            + CosineMult8bit(
                *(&raw mut (*E).khr_var_F as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as uint16,
                g_byte_A8F186 as uint16,
                game,
            ) as ::core::ffi::c_int
            - (*E).khr_var_01 as ::core::ffi::c_int) as uint16;
        if (*E).khr_var_F as ::core::ffi::c_int + 0x4000 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            KiHunter_Func_15(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                game,
            );
            return;
        }
        khr_var_07 = ((*E).khr_var_B as ::core::ffi::c_int - 384 as ::core::ffi::c_int) as int16;
        if (khr_var_07 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            khr_var_07 = 256 as int16;
        }
    }
    (*Get_KiHunter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .khr_var_B = khr_var_07 as uint16;
}
pub unsafe fn KiHunter_Func_15(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    (*E).khr_var_00 = (0xa8f7db as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).khr_var_B = (*E).khr_var_0A;
    (*E).khr_var_F = -(8192 as ::core::ffi::c_int) as uint16;
    (*E).khr_var_05 = (*E).base.x_pos;
    (*E).khr_var_06 = (*E).base.y_pos;
}
pub unsafe fn KiHunter_Func_16(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    (*E).khr_var_00 = (0xa8f8ad as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).khr_var_B = (*E).khr_var_0A;
    (*E).khr_var_F = -(24576 as ::core::ffi::c_int) as uint16;
    (*E).khr_var_05 = (*E).base.x_pos;
    (*E).khr_var_06 = (*E).base.y_pos;
}
pub unsafe fn KiHunter_Func_17(game: &mut Game) {
    let mut v5: uint16 = 0;
    let mut E: *mut Enemy_KiHunter = Get_KiHunter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).khr_var_0A = 0 as uint16;
    (*E).khr_var_B = 0 as uint16;
    loop {
        let mut v3: uint16 =
            ((*E).khr_var_0A as ::core::ffi::c_int + 384 as ::core::ffi::c_int) as uint16;
        (*E).khr_var_0A = v3;
        v3 = (v3 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint16;
        v5 = (*((game.g_rom.offset(
            (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16 as *mut uint8)
            .offset((8 as ::core::ffi::c_int * v3 as ::core::ffi::c_int) as isize)
            .offset(1) as *mut uint16) as ::core::ffi::c_int
            + (*E).khr_var_D as ::core::ffi::c_int) as uint16;
        (*E).khr_var_D = v5;
        if !(v5 as ::core::ffi::c_int - 0x2000 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0)
        {
            break;
        }
    }
}
