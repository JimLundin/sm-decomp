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
use crate::rtl::{mov24, Mult8x8, RomPtr, Unreachable, WriteReg};
extern "C" {
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;

}
pub use crate::types::{
    __int16_t, __int32_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32, int32_t,
    uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, CoroutineRet, LongPtr, SnesRegs, VoidP,
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
pub use crate::types::EnemyData;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct RoomDefStateSelect_E6E5_Finish {
    pub code_ptr: VoidP,
}
pub use crate::types::{Consts_86, DoorDef, RoomDefRoomstate};
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
pub use crate::types::Consts_87;
pub const addr_kAnimtiles_TourianStatue_Draygon: Consts_87 = 34142;
pub const addr_kAnimtiles_TourianStatue_Kraid: Consts_87 = 34136;
pub const addr_kAnimtiles_TourianStatue_Ridley: Consts_87 = 34130;
pub const addr_kAnimtiles_TourianStatue_Phantoon: Consts_87 = 34124;
pub const addr_kAnimtiles_Spores: Consts_87 = 33533;
pub const addr_kAnimtiles_Rain: Consts_87 = 33511;
pub const addr_kAnimtiles_Acid: Consts_87 = 33481;
pub const addr_kAnimtiles_Lava: Consts_87 = 33451;
pub const addr_kAnimtiles_WreckedShipTradmillLeft: Consts_87 = 33403;
pub const addr_kAnimtiles_WreckedShipTradmillRight: Consts_87 = 33397;
pub use crate::types::SpawnHardcodedPlmArgs;
pub const kCoroutineNone: ::core::ffi::c_int = 0;
#[inline]
unsafe fn RomPtr_8F(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x8f0000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn get_DoorDef(mut a: uint16, game: &mut Game) -> *mut DoorDef {
    return RomPtr(
        (0x830000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t,
        game,
    ) as *mut DoorDef;
}
#[inline]
unsafe fn get_RoomDefRoomstate(mut a: uint16, game: &mut Game) -> *mut RoomDefRoomstate {
    return RomPtr(
        (0x8f0000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t,
        game,
    ) as *mut RoomDefRoomstate;
}
#[inline]
unsafe fn get_RoomDefStateSelect_E6E5_Finish(
    mut a: uint16,
    game: &mut Game,
) -> *mut RoomDefStateSelect_E6E5_Finish {
    return RomPtr(
        (0x8f0000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t,
        game,
    ) as *mut RoomDefStateSelect_E6E5_Finish;
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
pub const fnPauseHook_DraygonRoom: ::core::ffi::c_int = 0x8fc8f6;
pub const fnUnpauseHook_DraygonRoom: ::core::ffi::c_int = 0x8fc8fc;
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
        SpawnHardcodedPlmArgs {
            field_0: 0x3d as uint8,
            field_1: 0xb as uint8,
            field_2: 0xbb30 as VoidP,
        },
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = 24 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn RoomSetup_AutoDestroyWallAfterEscape(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x10 as uint8,
            field_1: 0x87 as uint8,
            field_2: 0xb964 as VoidP,
        },
        game,
    );
}
pub unsafe fn RoomSetup_TurnWallIntoShotblocks(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0xf as uint8,
            field_1: 0xa as uint8,
            field_2: 0xb9ed as VoidP,
        },
        game,
    );
}
pub unsafe fn RoomSetup_ShakeDuringEscape(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = 6 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
    FxTypeFunc_20(game);
}
pub unsafe fn RoomSetup_ScrollingSkyLand(game: &mut Game) {
    FxTypeFunc_20(game);
}
pub unsafe fn RoomSetup_ScrollingSkyOcean(game: &mut Game) {
    RoomSetupAsm_ScrollingSkyOcean(game);
}
pub unsafe fn RoomSetup_RunStatueUnlockAnims(game: &mut Game) {
    SpawnAnimtiles(
        addr_kAnimtiles_TourianStatue_Kraid as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnAnimtiles(
        addr_kAnimtiles_TourianStatue_Phantoon as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnAnimtiles(
        addr_kAnimtiles_TourianStatue_Draygon as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnAnimtiles(
        addr_kAnimtiles_TourianStatue_Ridley as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn DoorCode_StartWreckedShipTreadmillWest(game: &mut Game) {
    SpawnAnimtiles(
        addr_kAnimtiles_WreckedShipTradmillRight as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x4 as uint8,
            field_1: 0x9 as uint8,
            field_2: 0xb64b as VoidP,
        },
        game,
    );
}
pub unsafe fn DoorCode_Scroll6_Green(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(6 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_Scroll0_Blue(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_Scroll13_Blue(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(19 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_Scroll_4Blue_8Green(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(8 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_Scroll_8toB_Red(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(8 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(9 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(10 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(11 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_Scroll_LotsRed(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(5 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(11 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(12 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(13 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(17 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_Scroll_1_4_Green(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_Scroll_2_Blue(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_Scroll_17_Blue(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(23 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_Scroll_4_Blue(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_Scroll_6_Green(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(6 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_Scroll_3_Green(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_0(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(24 as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(28 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_1(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(5 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(6 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_2(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(29 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_3(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_4(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_5(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(11 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_6(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(28 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(29 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_7(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_8(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(32 as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(36 as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(37 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_9(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_10(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_11(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(6 as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(7 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_12(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_13(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_14(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_15(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_16(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_17(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_18(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(15 as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(18 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_19(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(6 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_20(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_21(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_22(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(6 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(7 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(8 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_23(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(6 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_24(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_25(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_26(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(10 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_27(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_28(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_29(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(6 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(7 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(8 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_30(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_31(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(7 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_32(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_33(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_34(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_35(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_36(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_37(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_38(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_39(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_40(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_41(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(24 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_42(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_43(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(14 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn RoomCode_ScrollingSkyLand_(game: &mut Game) {
    RoomCode_ScrollingSkyLand(game);
}
pub unsafe fn RoomCode_ScrollingSkyOcean_(game: &mut Game) {
    RoomMainAsm_ScrollingSkyOcean(game);
}
pub unsafe fn RoomCode_GenRandomExplodes_Nonblank(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        let mut Random: uint16 = NextRandom(game);
        let mut x: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + Random as uint8 as ::core::ffi::c_int) as uint16;
        let mut y: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *(&raw mut Random as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int) as uint16;
        let mut prod: uint16 = Mult8x8(
            (y as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint8,
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8,
        );
        if *((&raw mut game.g_ram as *mut uint8).offset(0x10002 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                (prod as ::core::ffi::c_int + (x as ::core::ffi::c_int >> 4 as ::core::ffi::c_int))
                    as isize,
            ) as ::core::ffi::c_int
            & 0x3ff as ::core::ffi::c_int
            != 255 as ::core::ffi::c_int
        {
            RoomCode_GenRandomExplodes(x, y, game);
        }
    }
}
pub unsafe fn RoomCode_ScrollingSkyLand_Shakes(game: &mut Game) {
    RoomCode_ScrollingSkyLand(game);
    RoomCode_GenRandomExplodes_Nonblank(game);
    let ref mut fresh17 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1840 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh17 = (*fresh17 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn RoomCode_ExplodeShakes(game: &mut Game) {
    RoomCode_GenRandomExplodes_Nonblank(game);
    let ref mut fresh16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1840 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh16 = (*fresh16 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn RoomCode_GenRandomExplodes_4th(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        let mut Random: uint16 = NextRandom(game);
        RoomCode_GenRandomExplodes(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + Random as uint8 as ::core::ffi::c_int) as uint16,
            (*((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *(&raw mut Random as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int) as uint16,
            game,
        );
    }
}
pub unsafe fn RoomCode_GenRandomExplodes(mut x_r18: uint16, mut y_r20: uint16, game: &mut Game) {
    static kRoomCode_GenRandomExplodes_Sprite: [uint8; 8] = [
        3 as ::core::ffi::c_int as uint8,
        3 as ::core::ffi::c_int as uint8,
        9 as ::core::ffi::c_int as uint8,
        12 as ::core::ffi::c_int as uint8,
        12 as ::core::ffi::c_int as uint8,
        18 as ::core::ffi::c_int as uint8,
        18 as ::core::ffi::c_int as uint8,
        21 as ::core::ffi::c_int as uint8,
    ];
    static kRoomCode_GenRandomExplodes_Sfx: [uint8; 8] = [
        0x24 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0x25 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
    ];
    let mut v1: uint16 =
        (NextRandom(game) as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as uint16;
    if (v1 as ::core::ffi::c_int) < 8 as ::core::ffi::c_int {
        let mut v2: uint16 = kRoomCode_GenRandomExplodes_Sfx[v1 as usize] as uint16;
        if v2 != 0 {
            QueueSfx2_Max6(v2, game);
        }
    }
    CreateSpriteAtPos(
        x_r18,
        y_r20,
        kRoomCode_GenRandomExplodes_Sprite
            [(v1 as ::core::ffi::c_int & 7 as ::core::ffi::c_int) as usize] as uint16,
        0 as uint16,
        game,
    );
}
pub unsafe fn RoomCode_ScrollRightDachora(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(11 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 2 as ::core::ffi::c_int
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
            < 0x500 as ::core::ffi::c_int
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
            < 0x380 as ::core::ffi::c_int
    {
        let ref mut fresh15 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh15 = (*fresh15 as ::core::ffi::c_int
            + ((*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >= 0x380 as ::core::ffi::c_int) as ::core::ffi::c_int
                + 3 as ::core::ffi::c_int)) as uint16;
    }
}
pub unsafe fn RoomCode_8FC8C8(game: &mut Game) {
    SpawnEprojWithRoomGfx(
        addr_kEproj_PrePhantomRoom as ::core::ffi::c_int as uint16,
        0 as uint16,
        game,
    );
}
pub unsafe fn RoomCode_SetupShaktoolRoomPlm(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0 as uint8,
            field_1: 0 as uint8,
            field_2: 0xb8eb as VoidP,
        },
        game,
    );
}
pub unsafe fn RoomCode_SetPauseCodeForDraygon(game: &mut Game) {
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0x601 as ::core::ffi::c_int as isize)
            as *mut LongPtr,
        fnPauseHook_DraygonRoom as uint32,
    );
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0x604 as ::core::ffi::c_int as isize)
            as *mut LongPtr,
        fnUnpauseHook_DraygonRoom as uint32,
    );
}
pub unsafe fn PauseHook_DraygonRoom(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa7 as ::core::ffi::c_int as isize)
        as *mut uint16) = 4 as uint16;
}
pub unsafe fn UnpauseHook_DraygonRoom(game: &mut Game) -> CoroutineRet {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 8 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa7 as ::core::ffi::c_int as isize)
            as *mut uint16) = 12 as uint16;
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn RoomCode_SetCollectedMap(game: &mut Game) {
    let mut v0: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd908 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as *mut uint8 as *mut uint16) as ::core::ffi::c_int
        | 1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0xd908 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as *mut uint8 as *mut uint16) = v0;
    *((&raw mut game.g_ram as *mut uint8).offset(0x789 as ::core::ffi::c_int as isize)
        as *mut uint16) = v0;
}
pub unsafe fn RoomCode_SetZebesTimebombEvent(game: &mut Game) {
    SetEventHappened(0xe as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = 18 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn RoomCode_SetLightHorizRoomShake(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = 18 as uint16;
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x7e1 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 18 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e1 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn RoomCode_SetMediumHorizRoomShake(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = 21 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn RoomCode_Escape4_SetMediumHorizRoomShake(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x10 as uint8,
            field_1: 0x10 as uint8,
            field_2: 0xb968 as VoidP,
        },
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = 21 as uint16;
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x7e1 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 21 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e1 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn RoomCode_SetCeresDoorSolid(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0xf as uint8,
            field_1: 0x26 as uint8,
            field_2: 0xba48 as VoidP,
        },
        game,
    );
    FxTypeFunc_2C_Haze(game);
}
pub unsafe fn RoomCode_CeresColorMathHdma(game: &mut Game) {
    FxTypeFunc_2C_Haze(game);
}
pub unsafe fn RoomCode_CeresColorMathHdma_BgBase(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x5d as ::core::ffi::c_int as isize) = 102 as uint8;
    FxTypeFunc_2C_Haze(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x7eb as ::core::ffi::c_int as isize)
        as *mut uint16) = 9 as uint16;
}
pub unsafe fn DoorCode_StartWreckedSkipTreadmill_East(game: &mut Game) {
    SpawnAnimtiles(
        addr_kAnimtiles_WreckedShipTradmillLeft as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x4 as uint8,
            field_1: 0x9 as uint8,
            field_2: 0xb64f as VoidP,
        },
        game,
    );
}
pub unsafe fn DoorCode_SetScroll_44(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_45(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_46(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_47(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(41 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_48(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(40 as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(46 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_49(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(6 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(7 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(8 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(9 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(10 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(11 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_SetupElevatubeFromSouth(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x7e1 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
        -(256 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x7e1 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 2496 as uint16;
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x7e1 as ::core::ffi::c_int as isize)
        .offset(6 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
        -(32 as ::core::ffi::c_int) as uint16;
    CallSomeSamusCode(0 as uint16, game);
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x1 as uint8,
            field_1: 0 as uint8,
            field_2: 0xb8f9 as VoidP,
        },
        game,
    );
}
pub unsafe fn DoorCode_SetupElevatubeFromNorth(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x7e1 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 256 as uint16;
    strcpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset(0x7e1 as ::core::ffi::c_int as isize)
            .offset(2 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_char,
        b"@\0" as *const u8 as *const ::core::ffi::c_char,
    );
    strcpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset(0x7e1 as ::core::ffi::c_int as isize)
            .offset(6 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_char,
        b" \0" as *const u8 as *const ::core::ffi::c_char,
    );
    CallSomeSamusCode(0 as uint16, game);
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x1 as uint8,
            field_1: 0 as uint8,
            field_2: 0xb8f9 as VoidP,
        },
        game,
    );
}
pub unsafe fn RoomCode_Elevatube(game: &mut Game) {
    let mut v0: uint16 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) = 128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf8 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x7e1 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) as *mut uint8
        as *mut uint16) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    v0 = ((amt as uint32).wrapping_add(
        *((&raw mut game.g_ram as *mut uint8).offset(0x7e1 as ::core::ffi::c_int as isize)
            as *mut uint32),
    ) >> 16 as ::core::ffi::c_int) as uint16;
    let ref mut fresh13 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x7e1 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh13 = (*fresh13 as ::core::ffi::c_int + amt as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x7e1 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = v0;
    Samus_MoveDown_NoSolidColl(amt, game);
    if ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0x7e1 as ::core::ffi::c_int as isize)
        .offset(6 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16)
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8)
            .offset(0x7e1 as ::core::ffi::c_int as isize)
            .offset(4 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16)
            as ::core::ffi::c_int
        + 3616 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int)
        < 0x1c41 as ::core::ffi::c_int
    {
        let ref mut fresh14 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x7e1 as ::core::ffi::c_int as isize)
            .offset(4 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut uint16);
        *fresh14 = (*fresh14 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8)
                .offset(0x7e1 as ::core::ffi::c_int as isize)
                .offset(6 as ::core::ffi::c_int as isize) as *mut uint8
                as *mut uint16) as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn DoorCode_ResetElevatubeNorthExit(game: &mut Game) {
    CallSomeSamusCode(1 as uint16, game);
}
pub unsafe fn DoorCode_ResetElevatubeSouthExit(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd20 as ::core::ffi::c_int as isize)
        as *mut uint16) = 514 as uint16;
    CallSomeSamusCode(1 as uint16, game);
}
pub unsafe fn DoorCode_SetScroll_50(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(10 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(11 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn UNUSED_DoorCode_SetScroll_51(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(17 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(16 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn UNUSED_DoorCode_SetScroll_52(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_53(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_54(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_55(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(9 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(10 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_56(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_57(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_58(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(6 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_59(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_60(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_61(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(7 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_SetScroll_62(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 1 as uint8;
}
pub unsafe fn DoorCode_SetScroll_63(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_64(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(0 as ::core::ffi::c_int as isize) = 2 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = 2 as uint8;
}
pub unsafe fn DoorCode_SetScroll_65(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(24 as ::core::ffi::c_int as isize) = 1 as uint8;
    *(&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(25 as ::core::ffi::c_int as isize) = 0 as uint8;
}
pub unsafe fn DoorCode_CeresElevatorShaft(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x56 as ::core::ffi::c_int as isize) = 7 as uint8;
    WriteReg(BGMODE as ::core::ffi::c_int as uint16, 7 as uint8, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x78 as ::core::ffi::c_int as isize)
        as *mut uint16) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e as ::core::ffi::c_int as isize)
        as *mut uint16) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7c as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x80 as ::core::ffi::c_int as isize)
        as *mut uint16) = 128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x82 as ::core::ffi::c_int as isize)
        as *mut uint16) = 1008 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x783 as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
    strcpy(
        (&raw mut game.g_ram as *mut uint8).offset(0x7e1 as ::core::ffi::c_int as isize)
            as *mut ::core::ffi::c_char,
        b"\"\0" as *const u8 as *const ::core::ffi::c_char,
    );
    strcpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset(0x7e1 as ::core::ffi::c_int as isize)
            .offset(2 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut ::core::ffi::c_char,
        b"<\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
pub unsafe fn DoorCode_CeresElevatorShaft_2(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x56 as ::core::ffi::c_int as isize) = 9 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x783 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
}
pub unsafe fn RoomCode_CeresElevatorShaft_(game: &mut Game) {
    RoomCode_CeresElevatorShaft(game);
}
static kRoomCode_SpawnCeresFallingDebris_Tab: [uint16; 16] = [
    0x50 as ::core::ffi::c_int as uint16,
    0x60 as ::core::ffi::c_int as uint16,
    0x70 as ::core::ffi::c_int as uint16,
    0x80 as ::core::ffi::c_int as uint16,
    0x90 as ::core::ffi::c_int as uint16,
    0xa0 as ::core::ffi::c_int as uint16,
    0xb0 as ::core::ffi::c_int as uint16,
    0xc0 as ::core::ffi::c_int as uint16,
    0xd0 as ::core::ffi::c_int as uint16,
    0xe0 as ::core::ffi::c_int as uint16,
    0xf0 as ::core::ffi::c_int as uint16,
    0x110 as ::core::ffi::c_int as uint16,
    0x130 as ::core::ffi::c_int as uint16,
    0x150 as ::core::ffi::c_int as uint16,
    0x170 as ::core::ffi::c_int as uint16,
    0x190 as ::core::ffi::c_int as uint16,
];
pub unsafe fn RoomCode_SpawnCeresFallingDebris(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
        && {
            let ref mut fresh12 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x7e1 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh12 = (*fresh12).wrapping_sub(1);
            (*((&raw mut game.g_ram as *mut uint8).offset(0x7e1 as ::core::ffi::c_int as isize)
                as *mut int16) as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
        }
    {
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0x7e1 as ::core::ffi::c_int as isize)
            .offset(0 as ::core::ffi::c_int as isize) = 8 as uint8;
        *(&raw mut game.g_ram as *mut uint8)
            .offset(0x7e1 as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize) = 0 as uint8;
        let mut v0: uint16 = (if *((&raw mut game.g_ram as *mut uint8)
            .offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            addr_stru_869742 as ::core::ffi::c_int
        } else {
            addr_stru_869734 as ::core::ffi::c_int
        }) as uint16;
        SpawnEprojWithRoomGfx(
            v0,
            kRoomCode_SpawnCeresFallingDebris_Tab[(*((&raw mut game.g_ram as *mut uint8)
                .offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16)
                as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)
                as usize],
            game,
        );
    }
}
pub unsafe fn RoomCode_HandleCeresRidleyGetaway(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        HandleCeresRidleyGetawayCutscene(game);
    }
}
pub unsafe fn RoomCode_ShakeScreenHorizDiag(game: &mut Game) {
    let mut v0: uint16 = 0;
    let mut current_block_3: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x7e1 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        let ref mut fresh11 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x7e1 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh11 = (*fresh11).wrapping_sub(1);
        if *fresh11 == 0 {
            v0 = 18 as uint16;
            current_block_3 = 10175730460759437497;
        } else {
            current_block_3 = 11006700562992250127;
        }
    } else if (NextRandom(game) as ::core::ffi::c_int) < 0x200 as ::core::ffi::c_int {
        strcpy(
            (&raw mut game.g_ram as *mut uint8).offset(0x7e1 as ::core::ffi::c_int as isize)
                as *mut ::core::ffi::c_char,
            b"*\0" as *const u8 as *const ::core::ffi::c_char,
        );
        v0 = 23 as uint16;
        current_block_3 = 10175730460759437497;
    } else {
        current_block_3 = 11006700562992250127;
    }
    match current_block_3 {
        10175730460759437497 => {
            *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
                as *mut uint16) = v0;
        }
        _ => {}
    }
    RoomCode_GenRandomExplodes_4th(game);
}
pub unsafe fn RoomCode_ShakeScreenHorizDiagStrong(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut current_block_3: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x7e1 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x7e1 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh10 = (*fresh10).wrapping_sub(1);
        if *fresh10 == 0 {
            v0 = 21 as int16;
            current_block_3 = 1544504899801662111;
        } else {
            current_block_3 = 11006700562992250127;
        }
    } else if (NextRandom(game) as ::core::ffi::c_int) < 0x180 as ::core::ffi::c_int {
        strcpy(
            (&raw mut game.g_ram as *mut uint8).offset(0x7e1 as ::core::ffi::c_int as isize)
                as *mut ::core::ffi::c_char,
            b"*\0" as *const u8 as *const ::core::ffi::c_char,
        );
        v0 = 26 as int16;
        current_block_3 = 1544504899801662111;
    } else {
        current_block_3 = 11006700562992250127;
    }
    match current_block_3 {
        1544504899801662111 => {
            *((&raw mut game.g_ram as *mut uint8)
                .offset(0x7e1 as ::core::ffi::c_int as isize)
                .offset(2 as ::core::ffi::c_int as isize) as *mut uint8
                as *mut uint16) = v0 as uint16;
        }
        _ => {}
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x7e1 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) as *mut uint8
        as *mut uint16);
    RoomCode_GenRandomExplodes_4th(game);
}
pub unsafe fn CallRoomDefStateSelect(mut ea: uint32, mut k: uint16, game: &mut Game) -> uint16 {
    match ea {
        9430502 => return RoomDefStateSelect_Finish(k, game),
        9430527 => return RoomDefStateSelect_TourianBoss01(k, game),
        9430546 => return RoomDefStateSelect_IsEventSet(k, game),
        9430569 => return RoomDefStateSelect_IsBossDead(k, game),
        9430610 => return RoomDefStateSelect_MorphBallMissiles(k, game),
        9430633 => return RoomDefStateSelect_PowerBombs(k, game),
        _ => return Unreachable(game) as uint16,
    };
}
pub unsafe fn HandleRoomDefStateSelect(mut k: uint16, game: &mut Game) {
    let mut v1: uint16 = (k as ::core::ffi::c_int + 11 as ::core::ffi::c_int) as uint16;
    loop {
        let mut event_pointer: uint16 =
            (*get_RoomDefStateSelect_E6E5_Finish(v1, game)).code_ptr as uint16;
        v1 = CallRoomDefStateSelect(
            (event_pointer as ::core::ffi::c_int | 0x8f0000 as ::core::ffi::c_int) as uint32,
            (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
            game,
        );
        if !(v1 != 0) {
            break;
        }
    }
}
pub unsafe fn RoomDefStateSelect_Finish(mut k: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x7bb as ::core::ffi::c_int as isize)
        as *mut uint16) = k;
    return 0 as uint16;
}
pub unsafe fn RoomDefStateSelect_TourianBoss01(mut k: uint16, game: &mut Game) -> uint16 {
    if CheckBossBitForCurArea(1 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        == 0
    {
        return (k as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
    let mut v1: *const uint16 = RomPtr_8F(k as uint16_t, game) as *const uint16;
    return RoomDefStateSelect_Finish(*v1, game);
}
pub unsafe fn RoomDefStateSelect_IsEventSet(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v1: *const uint8 = RomPtr_8F(k as uint16_t, game);
    if CheckEventHappened(*v1 as uint16, game) != 0 {
        return RoomDefStateSelect_Finish(
            *(v1.offset(1 as ::core::ffi::c_int as isize) as *mut uint16),
            game,
        );
    } else {
        return (k as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn RoomDefStateSelect_IsBossDead(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v1: *const uint8 = RomPtr_8F(k as uint16_t, game);
    if CheckBossBitForCurArea(*v1 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0
    {
        return RoomDefStateSelect_Finish(
            *(v1.offset(1 as ::core::ffi::c_int as isize) as *mut uint16),
            game,
        );
    } else {
        return (k as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn RoomDefStateSelect_MorphBallMissiles(mut k: uint16, game: &mut Game) -> uint16 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a4 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 4 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x9c8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
    {
        return (k as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
    let mut v1: *const uint16 = RomPtr_8F(k as uint16_t, game) as *const uint16;
    return RoomDefStateSelect_Finish(*v1, game);
}
pub unsafe fn RoomDefStateSelect_PowerBombs(mut k: uint16, game: &mut Game) -> uint16 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9d0 as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
    {
        return (k as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
    let mut v1: *const uint16 = RomPtr_8F(k as uint16_t, game) as *const uint16;
    return RoomDefStateSelect_Finish(*v1, game);
}
pub unsafe fn CallRoomSetupCode(mut ea: uint32, game: &mut Game) {
    match ea {
        9408916 => {
            RoomSetup_AfterSavingAnimals(game);
            return;
        }
        9408937 => {
            RoomSetup_AutoDestroyWallAfterEscape(game);
            return;
        }
        9408946 => {
            RoomSetup_TurnWallIntoShotblocks(game);
            return;
        }
        9408957 => {
            RoomSetup_ShakeDuringEscape(game);
            return;
        }
        9408969 => {
            RoomSetup_ScrollingSkyLand(game);
            return;
        }
        9408974 => {
            RoomSetup_ScrollingSkyOcean(game);
            return;
        }
        9408983 => {
            RoomSetup_RunStatueUnlockAnims(game);
            return;
        }
        9423048 => {
            RoomCode_8FC8C8(game);
            return;
        }
        9423059 => {
            RoomCode_SetupShaktoolRoomPlm(game);
            return;
        }
        9423069 => {
            RoomCode_SetPauseCodeForDraygon(game);
            return;
        }
        9423114 => {
            RoomCode_SetCollectedMap(game);
            return;
        }
        9408955 | 9408956 | 9408979 | 9408980 | 9408981 | 9408982 | 9409012 | 9409013 | 9409014
        | 9409015 | 9423047 | 9423056 | 9423057 | 9423058 | 9423068 | 9423134 => return,
        9423135 => {
            RoomCode_SetZebesTimebombEvent(game);
            return;
        }
        9423155 => {
            RoomCode_SetLightHorizRoomShake(game);
            return;
        }
        9423174 => {
            RoomCode_SetMediumHorizRoomShake(game);
            return;
        }
        9423187 => {
            RoomCode_Escape4_SetMediumHorizRoomShake(game);
            return;
        }
        9423214 => {
            RoomCode_SetCeresDoorSolid(game);
            return;
        }
        9423222 => {
            RoomCode_CeresColorMathHdma(game);
            return;
        }
        9423227 => {
            RoomCode_CeresColorMathHdma_BgBase(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn RunRoomSetupCode(game: &mut Game) {
    let mut RoomDefRoomstate: *mut RoomDefRoomstate = ::core::ptr::null_mut::<RoomDefRoomstate>();
    RoomDefRoomstate = get_RoomDefRoomstate(
        *((&raw mut game.g_ram as *mut uint8).offset(0x7bb as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    if (*RoomDefRoomstate).room_setup_code != 0 {
        CallRoomSetupCode(
            ((*RoomDefRoomstate).room_setup_code as ::core::ffi::c_int
                | 0x8f0000 as ::core::ffi::c_int) as uint32,
            game,
        );
    }
}
pub unsafe fn CallDoorDefSetupCode(mut ea: uint32, game: &mut Game) {
    match ea {
        9419121 => {
            DoorCode_StartWreckedShipTreadmillWest(game);
            return;
        }
        9419137 => {
            DoorCode_Scroll6_Green(game);
            return;
        }
        9419148 => {
            DoorCode_Scroll0_Blue(game);
            return;
        }
        9419159 => {
            DoorCode_Scroll13_Blue(game);
            return;
        }
        9419170 => {
            DoorCode_Scroll_4Blue_8Green(game);
            return;
        }
        9419187 => {
            DoorCode_Scroll_8toB_Red(game);
            return;
        }
        9419210 => {
            DoorCode_Scroll_LotsRed(game);
            return;
        }
        9419249 => {
            DoorCode_Scroll_1_4_Green(game);
            return;
        }
        9419264 => {
            DoorCode_Scroll_2_Blue(game);
            return;
        }
        9419275 => {
            DoorCode_Scroll_17_Blue(game);
            return;
        }
        9419286 => {
            DoorCode_Scroll_4_Blue(game);
            return;
        }
        9419297 => {
            DoorCode_Scroll_6_Green(game);
            return;
        }
        9419308 => {
            DoorCode_Scroll_3_Green(game);
            return;
        }
        9420039 => {
            DoorCode_SetScroll_0(game);
            return;
        }
        9420054 => {
            DoorCode_SetScroll_1(game);
            return;
        }
        9420069 => {
            DoorCode_SetScroll_2(game);
            return;
        }
        9420080 => {
            DoorCode_SetScroll_3(game);
            return;
        }
        9420095 => {
            DoorCode_SetScroll_4(game);
            return;
        }
        9420112 => {
            DoorCode_SetScroll_5(game);
            return;
        }
        9420123 => {
            DoorCode_SetScroll_6(game);
            return;
        }
        9420140 => {
            DoorCode_SetScroll_7(game);
            return;
        }
        9420151 => {
            DoorCode_SetScroll_8(game);
            return;
        }
        9420170 => {
            DoorCode_SetScroll_9(game);
            return;
        }
        9420181 => {
            DoorCode_SetScroll_10(game);
            return;
        }
        9420192 => {
            DoorCode_SetScroll_11(game);
            return;
        }
        9420207 => {
            DoorCode_SetScroll_12(game);
            return;
        }
        9420224 => {
            DoorCode_SetScroll_13(game);
            return;
        }
        9420241 => {
            DoorCode_SetScroll_14(game);
            return;
        }
        9420258 => {
            DoorCode_SetScroll_15(game);
            return;
        }
        9420273 => {
            DoorCode_SetScroll_16(game);
            return;
        }
        9420288 => {
            DoorCode_SetScroll_17(game);
            return;
        }
        9420299 => {
            DoorCode_SetScroll_18(game);
            return;
        }
        9420314 => {
            DoorCode_SetScroll_19(game);
            return;
        }
        9420325 => {
            DoorCode_SetScroll_20(game);
            return;
        }
        9420342 => {
            DoorCode_SetScroll_21(game);
            return;
        }
        9420702 => {
            DoorCode_SetScroll_22(game);
            return;
        }
        9420731 => {
            DoorCode_SetScroll_23(game);
            return;
        }
        9420762 => {
            DoorCode_SetScroll_24(game);
            return;
        }
        9420777 => {
            DoorCode_SetScroll_25(game);
            return;
        }
        9420794 => {
            DoorCode_SetScroll_26(game);
            return;
        }
        9420822 => {
            DoorCode_SetScroll_28(game);
            return;
        }
        9420837 => {
            DoorCode_SetScroll_29(game);
            return;
        }
        9420858 => {
            DoorCode_SetScroll_30(game);
            return;
        }
        9420875 => {
            DoorCode_SetScroll_31(game);
            return;
        }
        9420886 => {
            DoorCode_SetScroll_32(game);
            return;
        }
        9420903 => {
            DoorCode_SetScroll_33(game);
            return;
        }
        9420920 => {
            DoorCode_SetScroll_34(game);
            return;
        }
        9420937 => {
            DoorCode_SetScroll_35(game);
            return;
        }
        9420962 => {
            DoorCode_SetScroll_36(game);
            return;
        }
        9420973 => {
            DoorCode_SetScroll_37(game);
            return;
        }
        9420994 => {
            DoorCode_SetScroll_38(game);
            return;
        }
        9421011 => {
            DoorCode_SetScroll_39(game);
            return;
        }
        9421022 => {
            DoorCode_SetScroll_40(game);
            return;
        }
        9421039 => {
            DoorCode_SetScroll_41(game);
            return;
        }
        9421050 => {
            DoorCode_SetScroll_42(game);
            return;
        }
        9421067 => {
            DoorCode_SetScroll_43(game);
            return;
        }
        9429464 => {
            DoorCode_StartWreckedSkipTreadmill_East(game);
            return;
        }
        9429480 => {
            DoorCode_SetScroll_44(game);
            return;
        }
        9429491 => {
            DoorCode_SetScroll_45(game);
            return;
        }
        9429502 => {
            DoorCode_SetScroll_46(game);
            return;
        }
        9429519 => {
            DoorCode_SetScroll_47(game);
            return;
        }
        9429530 => {
            DoorCode_SetScroll_48(game);
            return;
        }
        9429545 => {
            DoorCode_SetScroll_49(game);
            return;
        }
        9429612 => {
            DoorCode_SetupElevatubeFromSouth(game);
            return;
        }
        9429649 => {
            DoorCode_SetupElevatubeFromNorth(game);
            return;
        }
        9429761 => {
            DoorCode_ResetElevatubeNorthExit(game);
            return;
        }
        9429769 => {
            DoorCode_ResetElevatubeSouthExit(game);
            return;
        }
        9429784 => {
            DoorCode_SetScroll_50(game);
            return;
        }
        9429829 => {
            DoorCode_SetScroll_53(game);
            return;
        }
        9429846 => {
            DoorCode_SetScroll_54(game);
            return;
        }
        9429863 => {
            DoorCode_SetScroll_55(game);
            return;
        }
        9429880 => {
            DoorCode_SetScroll_56(game);
            return;
        }
        9429901 => {
            DoorCode_SetScroll_57(game);
            return;
        }
        9429912 => {
            DoorCode_SetScroll_58(game);
            return;
        }
        9429923 => {
            DoorCode_SetScroll_59(game);
            return;
        }
        9429945 => {
            DoorCode_SetScroll_61(game);
            return;
        }
        9429960 => {
            DoorCode_SetScroll_62(game);
            return;
        }
        9429977 => {
            DoorCode_SetScroll_63(game);
            return;
        }
        9430208 => {
            DoorCode_SetScroll_64(game);
            return;
        }
        9430223 => {
            DoorCode_SetScroll_65(game);
            return;
        }
        9430240 => {
            DoorCode_CeresElevatorShaft(game);
            return;
        }
        9430291 => {
            DoorCode_CeresElevatorShaft_2(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn RunDoorSetupCode(game: &mut Game) {
    let mut DD: *mut DoorDef = get_DoorDef(
        *((&raw mut game.g_ram as *mut uint8).offset(0x78d as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    if (*DD).door_setup_code != 0 {
        CallDoorDefSetupCode(
            ((*DD).door_setup_code as ::core::ffi::c_int | 0x8f0000 as ::core::ffi::c_int)
                as uint32,
            game,
        );
    }
}
pub unsafe fn RoomCode_CrocomireShaking(game: &mut Game) {
    let mut ai_var_D: uint16 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .properties as ::core::ffi::c_int
        & 0x200 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .ai_var_A as ::core::ffi::c_int
            == 64 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb3 as ::core::ffi::c_int as isize)
                as *mut uint16) = ((*((&raw mut game.g_ram as *mut uint8)
                .offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(1 as ::core::ffi::c_int as isize))
            .ai_var_D as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x91f as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                as uint16;
        } else if (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xf78 as ::core::ffi::c_int as isize) as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .ai_var_B as ::core::ffi::c_int
            & 0x400 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let fresh5 = &raw mut (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(1 as ::core::ffi::c_int as isize))
            .ai_var_D;
            fresh5.write_unaligned(fresh5.read_unaligned().wrapping_sub(1));
            if fresh5.read_unaligned() as ::core::ffi::c_int + 7 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                ai_var_D = ((*((&raw mut game.g_ram as *mut uint8)
                    .offset(0xf78 as ::core::ffi::c_int as isize)
                    as *mut EnemyData)
                    .offset(1 as ::core::ffi::c_int as isize))
                .ai_var_D as ::core::ffi::c_int
                    - 2 as ::core::ffi::c_int
                        * ((*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xf78 as ::core::ffi::c_int as isize)
                            as *mut EnemyData)
                            .offset(1 as ::core::ffi::c_int as isize))
                        .ai_var_D as ::core::ffi::c_int
                            + 7 as ::core::ffi::c_int)) as uint16;
            } else {
                ai_var_D = (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0xf78 as ::core::ffi::c_int as isize)
                    as *mut EnemyData)
                    .offset(1 as ::core::ffi::c_int as isize))
                .ai_var_D;
            }
            let ref mut fresh6 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xb3 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh6 = (*fresh6 as ::core::ffi::c_int + ai_var_D as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (ai_var_D as ::core::ffi::c_int - 48 as ::core::ffi::c_int) as uint16;
        } else if (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xf78 as ::core::ffi::c_int as isize) as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .ai_var_C as ::core::ffi::c_int
            == 34 as ::core::ffi::c_int
            && (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .ai_var_D as ::core::ffi::c_int
                != 0
        {
            let fresh7 = &raw mut (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .ai_var_D;
            fresh7.write_unaligned(fresh7.read_unaligned().wrapping_sub(1));
            if (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .ai_var_D as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                let ref mut fresh8 = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh8 = (*fresh8 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
            } else {
                let ref mut fresh9 = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh9 = (*fresh9 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
            }
        }
    }
}
static kRoomCode_RidleyRoomShaking_X: [int16; 8] = [
    0 as ::core::ffi::c_int as int16,
    2 as ::core::ffi::c_int as int16,
    2 as ::core::ffi::c_int as int16,
    2 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(2 as ::core::ffi::c_int) as int16,
    -(2 as ::core::ffi::c_int) as int16,
    -(2 as ::core::ffi::c_int) as int16,
];
static kRoomCode_RidleyRoomShaking_Y: [int16; 8] = [
    -(2 as ::core::ffi::c_int) as int16,
    -(2 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    2 as ::core::ffi::c_int as int16,
    2 as ::core::ffi::c_int as int16,
    2 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(2 as ::core::ffi::c_int) as int16,
];
pub unsafe fn RoomCode_RidleyRoomShaking(game: &mut Game) {
    let mut ai_var_A: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(4 as ::core::ffi::c_int as isize))
    .ai_var_A;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(4 as ::core::ffi::c_int as isize))
    .ai_var_A
        != 0
    {
        let fresh0 = &raw mut (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(4 as ::core::ffi::c_int as isize))
        .ai_var_A;
        fresh0.write_unaligned(fresh0.read_unaligned().wrapping_sub(1));
        let mut v1: ::core::ffi::c_int = ai_var_A as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        let ref mut fresh1 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xb1 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh1 = (*fresh1 as ::core::ffi::c_int
            + kRoomCode_RidleyRoomShaking_X[v1 as usize] as ::core::ffi::c_int)
            as uint16;
        let ref mut fresh2 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xb5 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh2 = (*fresh2 as ::core::ffi::c_int
            + kRoomCode_RidleyRoomShaking_X[v1 as usize] as ::core::ffi::c_int)
            as uint16;
        let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xb3 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh3 = (*fresh3 as ::core::ffi::c_int
            + kRoomCode_RidleyRoomShaking_Y[v1 as usize] as ::core::ffi::c_int)
            as uint16;
        let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xb7 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh4 = (*fresh4 as ::core::ffi::c_int
            + kRoomCode_RidleyRoomShaking_Y[v1 as usize] as ::core::ffi::c_int)
            as uint16;
    }
}
pub unsafe fn CallRoomCode(mut ea: uint32, game: &mut Game) {
    match ea {
        9421078 => {
            RoomCode_ScrollingSkyLand_(game);
            return;
        }
        9421083 => {
            RoomCode_ScrollingSkyOcean_(game);
            return;
        }
        9421088 => {
            RoomCode_ScrollingSkyLand_Shakes(game);
            return;
        }
        9421092 => {
            RoomCode_ExplodeShakes(game);
            return;
        }
        9421286 => {
            RoomCode_ScrollRightDachora(game);
            return;
        }
        9429686 => {
            RoomCode_Elevatube(game);
            return;
        }
        9430303 => {
            RoomCode_CeresElevatorShaft_(game);
            return;
        }
        9430308 => return,
        9430309 => {
            RoomCode_SpawnCeresFallingDebris(game);
            return;
        }
        9430385 => {
            RoomCode_HandleCeresRidleyGetaway(game);
            return;
        }
        9430396 => {
            RoomCode_ShakeScreenHorizDiag(game);
            return;
        }
        9430432 => {
            RoomCode_GenRandomExplodes_4th(game);
            return;
        }
        9430436 => {
            RoomCode_ShakeScreenHorizDiagStrong(game);
            return;
        }
        9431245 => {
            RoomCode_CrocomireShaking(game);
            return;
        }
        9431376 => {
            RoomCode_RidleyRoomShaking(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn RunRoomMainCode(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x7df as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        CallRoomCode(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x7df as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                | 0x8f0000 as ::core::ffi::c_int) as uint32,
            game,
        );
    }
}
