use crate::game::sm_80::{
    DecompressToMem, NextRandom, QueueMusic_Delayed8, QueueSfx1_Max6, QueueSfx2_Max15,
    QueueSfx2_Max6, QueueSfx3_Max6, ScreenOff, ScreenOn, SetupDmaTransfer, WaitForNMI_Async,
};
use crate::game::sm_82::AdvancePaletteFade_BgPalette6;
use crate::game::sm_84::SpawnHardcodedPlm;
use crate::game::sm_86::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::sm_88::{sub_88E487, SpawnHdmaObject};
use crate::game::sm_90::DisableMinimapAndMarkBossRoomAsExplored;
use crate::game::sm_a0::{
    CallEnemyPreInstr, CheckIfEnemyTouchesSamus, DetermineDirectionOfSamusFromEnemy,
    EnemyDeathAnimation, EnemyFunc_BBBF, EnemyFunc_C8AD, Enemy_ItemDrop_Kraid,
    Enemy_ItemDrop_Phantoon, Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes, Enemy_SwitchToFrozenAi,
    IsSamusWithinEnemy_X, IsSamusWithinEnemy_Y, NormalEnemyFrozenAI,
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy, NormalEnemyShotAi,
    NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAi,
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy, SwitchEnemyAiToMainAi,
};
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::game_state::Game;
use crate::rtl::abs16;
use crate::rtl::{MakePairU16, RomPtr, SnesDivide, Unreachable, WriteReg};
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
    int32_t, int8, int8_t, size_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t,
    CoroutineRet, Func_Y_V, LongPtr, PairU16, Point16U, SnesRegs, VoidP, VramWriteEntry,
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
pub use crate::types::{
    EnemyBase, EnemyData, EnemyProps, ExtendedSpriteMap, Hitbox, SpawnHdmaObject_Args,
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
pub struct SpriteDrawInstr {
    pub field_0: uint16,
    pub field_2: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct KraidInstrList {
    pub timer: uint16,
    pub tilemap: VoidP,
    pub vuln_mouth_hitbox: VoidP,
    pub invuln_mouth_hitbox: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct KraidSinkTable {
    pub field_0: uint16,
    pub field_2: uint16,
    pub field_4: VoidP,
}
pub use crate::types::{
    Consts_86, EnemyDef, Ram4000, Ram4000_Backups, StartDmaCopy, VramReadQueueEnt,
};
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
pub use crate::types::Consts_A7;
pub const addr_kDachora_Ilist_F4C1: Consts_A7 = 62657;
pub const addr_kDachora_Ilist_F4B9: Consts_A7 = 62649;
pub const addr_kDachora_Ilist_F4B3: Consts_A7 = 62643;
pub const addr_kDachora_Ilist_F48B: Consts_A7 = 62603;
pub const addr_kDachora_Ilist_F45B: Consts_A7 = 62555;
pub const addr_kDachora_Ilist_F407: Consts_A7 = 62471;
pub const addr_kDachora_Ilist_F3FF: Consts_A7 = 62463;
pub const addr_kDachora_Ilist_F3F7: Consts_A7 = 62455;
pub const addr_kDachora_Ilist_F3C9: Consts_A7 = 62409;
pub const addr_kDachora_Ilist_F399: Consts_A7 = 62361;
pub const addr_kDachora_Ilist_F345: Consts_A7 = 62277;
pub const addr_kDachora_Palette: Consts_A7 = 61989;
pub const addr_kEtecoon_Ilist_E8D6: Consts_A7 = 59606;
pub const addr_kEtecoon_Ilist_E8CE: Consts_A7 = 59598;
pub const addr_kEtecoon_Ilist_E8C8: Consts_A7 = 59592;
pub const addr_kEtecoon_Ilist_E8AC: Consts_A7 = 59564;
pub const addr_stru_A7E898: Consts_A7 = 59544;
pub const addr_kEtecoon_Ilist_E894: Consts_A7 = 59540;
pub const addr_stru_A7E880: Consts_A7 = 59520;
pub const addr_kEtecoon_Ilist_E876: Consts_A7 = 59510;
pub const addr_kEtecoon_Ilist_E870: Consts_A7 = 59504;
pub const addr_stru_A7E862: Consts_A7 = 59490;
pub const addr_kEtecoon_Ilist_E854: Consts_A7 = 59476;
pub const addr_kEtecoon_Ilist_E83C: Consts_A7 = 59452;
pub const addr_stru_A7E828: Consts_A7 = 59432;
pub const addr_kEtecoon_Ilist_E81E: Consts_A7 = 59422;
pub const addr_kPhantoonMoveData: Consts_A7 = 58322;
pub const addr_kKraid_Ilist_CCEB: Consts_A7 = 52459;
pub const addr_kKraid_Ilist_CC9D: Consts_A7 = 52381;
pub const addr_kKraid_Ilist_CC91: Consts_A7 = 52369;
pub const addr_kKraid_Ilist_CC81: Consts_A7 = 52353;
pub const addr_kKraid_Ilist_CC53: Consts_A7 = 52307;
pub const addr_kKraid_Ilist_CC4D: Consts_A7 = 52301;
pub const addr_kKraid_Ilist_CC47: Consts_A7 = 52295;
pub const addr_kKraid_Ilist_CC41: Consts_A7 = 52289;
pub const addr_locret_A7B831: Consts_A7 = 47153;
pub const addr_kKraidsRoomBg: Consts_A7 = 42774;
pub const addr_kKraid_Sprmap_A5DF: Consts_A7 = 42463;
pub const addr_kKraidTilemaps_3: Consts_A7 = 41160;
pub const addr_kKraidTilemaps_2: Consts_A7 = 40392;
pub const addr_kKraidTilemaps_1: Consts_A7 = 39624;
pub const addr_kKraidTilemaps_0: Consts_A7 = 38856;
pub const addr_stru_A79764: Consts_A7 = 38756;
pub const addr_stru_A7974A: Consts_A7 = 38730;
pub const addr_stru_A796DA: Consts_A7 = 38618;
pub const addr_kKraid_Sprmap_8C6C: Consts_A7 = 35948;
pub const addr_kKraid_Ilist_8B0A: Consts_A7 = 35594;
pub const addr_kKraid_Ilist_8B04: Consts_A7 = 35588;
pub const addr_kKraid_Ilist_8AFE: Consts_A7 = 35582;
pub const addr_kKraid_Ilist_8AF0: Consts_A7 = 35568;
pub const addr_kKraid_Ilist_8AA4: Consts_A7 = 35492;
pub const addr_kKraid_Ilist_8A41: Consts_A7 = 35393;
pub const addr_kKraid_Ilist_89F3: Consts_A7 = 35315;
pub const addr_kKraid_Ilist_8887: Consts_A7 = 34951;
pub const addr_off_A78885: Consts_A7 = 34949;
pub const addr_kKraid_Ilist_87BD: Consts_A7 = 34749;
pub const addr_off_A787BB: Consts_A7 = 34747;
pub const addr_kKraid_Ilist_86F3: Consts_A7 = 34547;
pub const addr_kKraid_Ilist_86ED: Consts_A7 = 34541;
pub const addr_kKraid_Ilist_86E7: Consts_A7 = 34535;
pub const addr_kSpritemap_Nothing_A7: Consts_A7 = 32845;
pub use crate::types::SpawnHardcodedPlmArgs;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Phantoon {
    pub base: EnemyBase,
    pub phant_var_A: uint16,
    pub phant_var_B: uint16,
    pub phant_var_C: uint16,
    pub phant_var_D: uint16,
    pub phant_var_E: uint16,
    pub phant_var_F: uint16,
    pub phant_parameter_1: uint16,
    pub phant_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Kraid {
    pub base: EnemyBase,
    pub kraid_var_A: uint16,
    pub kraid_var_B: uint16,
    pub kraid_var_C: uint16,
    pub kraid_var_D: uint16,
    pub kraid_var_E: uint16,
    pub kraid_var_F: uint16,
    pub kraid_parameter_1: uint16,
    pub kraid_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub kraid_next: uint16,
    pub field_2: uint16,
    pub field_4: uint16,
    pub kraid_thinking: uint16,
    pub kraid_min_y_pos_eject: uint16,
    pub kraid_mouth_flags: uint16,
    pub kraid_healths_8ths: [uint16; 8],
    pub field_1C: uint16,
    pub kraid_target_x: uint16,
    pub kraid_healths_4ths: [uint16; 4],
    pub field_28: uint16,
    pub kraid_hurt_frame: uint16,
    pub kraid_hurt_frame_timer: uint16,
    pub field_2E: [uint16; 9],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Etecoon {
    pub base: EnemyBase,
    pub etecoon_var_A: uint16,
    pub etecoon_var_B: uint16,
    pub etecoon_var_C: uint16,
    pub etecoon_var_D: uint16,
    pub etecoon_var_E: uint16,
    pub etecoon_var_F: uint16,
    pub etecoon_parameter_1: uint16,
    pub etecoon_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Dachora {
    pub base: EnemyBase,
    pub dachor_var_A: uint16,
    pub dachor_var_B: uint16,
    pub dachor_var_C: uint16,
    pub dachor_var_D: uint16,
    pub dachor_var_E: uint16,
    pub dachor_var_F: uint16,
    pub dachor_parameter_1: uint16,
    pub dachor_parameter_2: uint16,
}
pub const kCoroutineNone: ::core::ffi::c_int = 0;
#[inline]
unsafe fn RomPtr_A7(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0xa70000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn gVramWriteEntry(mut a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    return (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry;
}
#[inline]
unsafe fn get_EnemyDef_A2(mut a: uint16, game: &mut Game) -> *mut EnemyDef {
    return RomPtr(
        (0xa00000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t,
        game,
    ) as *mut EnemyDef;
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
#[inline]
unsafe fn SetHiLo(mut hi: *mut uint16, mut lo: *mut uint16, mut t: uint32) {
    *lo = t as uint16;
    *hi = (t >> 16 as ::core::ffi::c_int) as uint16;
}
pub const fnnullsub_170_A7: ::core::ffi::c_int = 0xa7804c;
pub const fnKraid_GetsBig_BreakCeilingPlatforms: ::core::ffi::c_int = 0xa7ac4d;
pub const fnKraid_GetsBig_SetBG2TilemapPrioBits: ::core::ffi::c_int = 0xa7ad3a;
pub const fnKraid_GetsBig_FinishUpdateBg2Tilemap: ::core::ffi::c_int = 0xa7ad61;
pub const fnKraid_GetsBig_DrawRoomBg: ::core::ffi::c_int = 0xa7ad8e;
pub const fnKraid_GetsBig_FadeInRoomBg: ::core::ffi::c_int = 0xa7ae23;
pub const fnKraid_Mainloop_Thinking: ::core::ffi::c_int = 0xa7aea4;
pub const fnKraid_GetsBig_Thinking: ::core::ffi::c_int = 0xa7aec4;
pub const fnKraid_Shot_MouthIsOpen: ::core::ffi::c_int = 0xa7aee4;
pub const fnKraidInstr_PlayRoarSfx: uint32 = 10989460;
pub const fnKraidInstr_PlayDyingSfx: uint32 = 10989471;
pub const fnKraid_InitEyeGlowing: ::core::ffi::c_int = 0xa7b6bf;
pub const fnKraid_Shot_GlowHisEye: ::core::ffi::c_int = 0xa7b6d7;
pub const fnKraid_Shot_UnglowEye: ::core::ffi::c_int = 0xa7b73d;
pub const fnnullsub_347: uint32 = 10991665;
pub const fnKraidLint_ProduceLint: ::core::ffi::c_int = 0xa7b832;
pub const fnKraidLint_ChargeLint: ::core::ffi::c_int = 0xa7b868;
pub const fnKraidLint_FireLint: ::core::ffi::c_int = 0xa7b89b;
pub const fnKraidFingernail_WaitForLintXpos: ::core::ffi::c_int = 0xa7b907;
pub const fnKraid_AlignEnemyToKraid: ::core::ffi::c_int = 0xa7b923;
pub const fnKraidEnemy_HandleFunctionTimer: ::core::ffi::c_int = 0xa7b92d;
pub const fnKraidEnemy_DecrementEnemyFunctionTimer: ::core::ffi::c_int = 0xa7b93f;
pub const fnKraidFoot_FirstPhase_Thinking: ::core::ffi::c_int = 0xa7b960;
pub const fnKraidEnemy_ProcessInstrEnemyTimer: ::core::ffi::c_int = 0xa7b965;
pub const fnnullsub_234: ::core::ffi::c_int = 0xa7ba2d;
pub const fnKraidsFoot_SecondPhase_Thinking: ::core::ffi::c_int = 0xa7ba2e;
pub const fnKraidsFoot_SecondPhase_WalkingBackwards: ::core::ffi::c_int = 0xa7bb45;
pub const fnKraidsFoot_SecondPhaseSetup_WalkToStartPt: ::core::ffi::c_int = 0xa7bb6e;
pub const fnKraidsFoot_SecondPhase_Init: ::core::ffi::c_int = 0xa7bba4;
pub const fnKraidsFoot_SecondPhase_WalkForward: ::core::ffi::c_int = 0xa7bbae;
pub const fnKraid_Main_AttackWithMouthOpen: ::core::ffi::c_int = 0xa7bbea;
pub const fnKraidsFingernail_Init: ::core::ffi::c_int = 0xa7bd60;
pub const fnKraidsFingernail_Fire: ::core::ffi::c_int = 0xa7be8e;
pub const fnKraidsFoot_PrepareToLungeForward: ::core::ffi::c_int = 0xa7bf2d;
pub const fnKraidsFoot_FirstPhase_LungeForward: ::core::ffi::c_int = 0xa7bf5d;
pub const fnKraidsFoot_FirstPhase_RetreatFromLunge: ::core::ffi::c_int = 0xa7bfab;
pub const fnKraid_GetsBig_ReleaseCamera: ::core::ffi::c_int = 0xa7c0a1;
pub const fnUnpauseHook_Kraid_IsDead: ::core::ffi::c_int = 0xa7c1fb;
pub const fnUnpauseHook_Kraid_IsAlive: ::core::ffi::c_int = 0xa7c24e;
pub const fnKraid_UnpauseHook_IsSinking: ::core::ffi::c_int = 0xa7c2a0;
pub const fnPauseHook_Kraid: ::core::ffi::c_int = 0xa7c325;
pub const fnKraid_Death_Init: ::core::ffi::c_int = 0xa7c360;
pub const fnKraid_Death_Fadeout: ::core::ffi::c_int = 0xa7c3f9;
pub const fnKraid_Death_UpdateBG2TilemapTopHalf: ::core::ffi::c_int = 0xa7c4a4;
pub const fnKraid_Death_UpdateBG2TilemapBottomHalf: ::core::ffi::c_int = 0xa7c4c8;
pub const fnKraid_Death_SinkThroughFloor: ::core::ffi::c_int = 0xa7c537;
pub const fnKraid_CrumbleLeftPlatform_Left: uint32 = 10995345;
pub const fnnullsub_356: uint32 = 10995366;
pub const fnKraid_CrumbleRightPlatform_Middle: uint32 = 10995367;
pub const fnKraid_CrumbleRightPlatform_Left: uint32 = 10995389;
pub const fnKraid_CrumbleLeftPlatform_Right: uint32 = 10995411;
pub const fnKraid_CrumbleLeftPlatform_Middle: uint32 = 10995433;
pub const fnKraid_CrumbleRightPlatform_Right: uint32 = 10995455;
pub const fnKraid_FadeInBg_ClearBg2TilemapTopHalf: ::core::ffi::c_int = 0xa7c715;
pub const fnKraid_FadeInBg_ClearBg2TilemapBottomHalf: ::core::ffi::c_int = 0xa7c751;
pub const fnKraid_FadeInBg_LoadBg3Tiles1of4: ::core::ffi::c_int = 0xa7c777;
pub const fnKraid_FadeInBg_LoadBg3Tiles2of4: ::core::ffi::c_int = 0xa7c7a3;
pub const fnKraid_FadeInBg_LoadBg3Tiles3of4: ::core::ffi::c_int = 0xa7c7c9;
pub const fnKraid_FadeInBg_LoadBg3Tiles4of4: ::core::ffi::c_int = 0xa7c7ef;
pub const fnKraid_FadeInBg_FadeInBp6: ::core::ffi::c_int = 0xa7c815;
pub const fnKraid_FadeInBg_SetEnemyDead_KraidWasAlive: ::core::ffi::c_int = 0xa7c843;
pub const fnKraid_FadeInBg_SetEnemyDead_KraidWasDead: ::core::ffi::c_int = 0xa7c851;
pub const fnKraid_RestrictSamusXtoFirstScreen: ::core::ffi::c_int = 0xa7c865;
pub const fnKraid_RaiseKraidThroughFloor: ::core::ffi::c_int = 0xa7c86b;
pub const fnKraid_Raise_LoadTilemapBottomAndShake: ::core::ffi::c_int = 0xa7c89a;
pub const fnKraid_Raise_SpawnRandomEarthquakeProjs16: ::core::ffi::c_int = 0xa7c8e0;
pub const fnKraid_Raise_SpawnRandomEarthquakeProjs8: ::core::ffi::c_int = 0xa7c902;
pub const fnKraid_Raise_Handler: ::core::ffi::c_int = 0xa7c924;
pub const fnPhantoon_Func_1: uint32 = 10997485;
pub const fnPhantoon_Func_4: uint32 = 10997598;
pub const fnPhantoon_StartTrackingSamusAndInitEyeTimer: uint32 = 10997823;
pub const fnPhantoon_PickPatternForRound2: uint32 = 10997878;
pub const fnnullsub_237: ::core::ffi::c_int = 0xa7d4a8;
pub const fnPhantoon_Spawn8FireballsInCircleAtStart: ::core::ffi::c_int = 0xa7d4a9;
pub const fnPhantoon_WaitBetweenSpawningAndSpinningFireballs: ::core::ffi::c_int = 0xa7d4ee;
pub const fnPhantoon_SpawnFireballsBeforeFight: ::core::ffi::c_int = 0xa7d508;
pub const fnPhantoon_WavyFadeIn: ::core::ffi::c_int = 0xa7d54a;
pub const fnPhantoon_PickPatternForRound1: ::core::ffi::c_int = 0xa7d596;
pub const fnPhantoon_MovePhantoonInFigure8ThenOpenEye: ::core::ffi::c_int = 0xa7d5e7;
pub const fnPhantoon_EyeFollowsSamusUntilTimerRunsOut: ::core::ffi::c_int = 0xa7d60d;
pub const fnPhantoon_BecomesSolidAndBodyVuln: ::core::ffi::c_int = 0xa7d65c;
pub const fnPhantoon_IsSwooping: ::core::ffi::c_int = 0xa7d678;
pub const fnPhantoon_FadeoutWithSwoop: ::core::ffi::c_int = 0xa7d6b9;
pub const fnPhantoon_WaitAfterFadeOut: ::core::ffi::c_int = 0xa7d6d4;
pub const fnPhantoon_MoveLeftOrRightAndPickEyeOpenPatt: ::core::ffi::c_int = 0xa7d6e2;
pub const fnPhantoon_FadeInBeforeFigure8: ::core::ffi::c_int = 0xa7d72d;
pub const fnPhantoon_BecomeSolidAfterRainingFireballs: ::core::ffi::c_int = 0xa7d73f;
pub const fnPhantoon_FadeInDuringFireballRain: ::core::ffi::c_int = 0xa7d767;
pub const fnPhantoon_FollowSamusWithEyeDuringFireballRain: ::core::ffi::c_int = 0xa7d788;
pub const fnPhantoon_FadeOutDuringFireballRain: ::core::ffi::c_int = 0xa7d7d5;
pub const fnPhantoon_SpawnRainingFireballs: ::core::ffi::c_int = 0xa7d7f7;
pub const fnPhantoon_FadeOutBeforeFirstFireballRain: ::core::ffi::c_int = 0xa7d82a;
pub const fnPhantoon_FadeOutBeforeEnrage: ::core::ffi::c_int = 0xa7d85c;
pub const fnPhantoon_MoveEnragedPhantoonToTopCenter: ::core::ffi::c_int = 0xa7d874;
pub const fnPhantoon_FadeInEnragedPhantoon: ::core::ffi::c_int = 0xa7d891;
pub const fnPhantoon_Enraged: ::core::ffi::c_int = 0xa7d8ac;
pub const fnPhantoon_FadeoutAfterEnrage: ::core::ffi::c_int = 0xa7d916;
pub const fnPhantoon_CompleteSwoopAfterFatalShot: ::core::ffi::c_int = 0xa7d92e;
pub const fnPhantoon_DyingPhantoonFadeInOut: ::core::ffi::c_int = 0xa7d948;
pub const fnPhantoon_DyingPhantoonExplosions: ::core::ffi::c_int = 0xa7d98b;
pub const fnPhantoon_WavyDyingPhantoonAndCry: ::core::ffi::c_int = 0xa7da51;
pub const fnPhantoon_DyingFadeOut: ::core::ffi::c_int = 0xa7da86;
pub const fnPhantoon_AlmostDead: ::core::ffi::c_int = 0xa7dad7;
pub const fnPhantoon_Dead: ::core::ffi::c_int = 0xa7db3d;
pub const fnEtecoon_Func_4: ::core::ffi::c_int = 0xa7e9af;
pub const fnEtecoon_Func_5: ::core::ffi::c_int = 0xa7ea00;
pub const fnEtecoon_Func_6: ::core::ffi::c_int = 0xa7ea37;
pub const fnEtecoon_Func_7: ::core::ffi::c_int = 0xa7eab5;
pub const fnEtecoon_Func_8: ::core::ffi::c_int = 0xa7eb02;
pub const fnEtecoon_Func_9: ::core::ffi::c_int = 0xa7eb2c;
pub const fnEtecoon_Func_10: ::core::ffi::c_int = 0xa7eb50;
pub const fnEtecoon_Func_11: ::core::ffi::c_int = 0xa7ebcd;
pub const fnEtecoon_Func_12: ::core::ffi::c_int = 0xa7ec1b;
pub const fnEtecoon_Func_16: ::core::ffi::c_int = 0xa7ec97;
pub const fnEtecoon_Func_17: ::core::ffi::c_int = 0xa7ecbb;
pub const fnEtecoon_Func_18: ::core::ffi::c_int = 0xa7ecdf;
pub const fnEtecoon_Func_19: ::core::ffi::c_int = 0xa7ed09;
pub const fnEtecoon_Func_20: ::core::ffi::c_int = 0xa7ed2a;
pub const fnEtecoon_Func_21: ::core::ffi::c_int = 0xa7ed54;
pub const fnEtecoon_Func_22: ::core::ffi::c_int = 0xa7ed75;
pub const fnEtecoon_Func_23: ::core::ffi::c_int = 0xa7edc7;
pub const fnEtecoon_Func_24: ::core::ffi::c_int = 0xa7ee3e;
pub const fnEtecoon_Func_25: ::core::ffi::c_int = 0xa7ee9a;
pub const fnEtecoon_Func_26: ::core::ffi::c_int = 0xa7eeb8;
pub const fnDachora_Func_2: ::core::ffi::c_int = 0xa7f570;
pub const fnDachora_Func_3: ::core::ffi::c_int = 0xa7f5bc;
pub const fnDachora_Func_4: ::core::ffi::c_int = 0xa7f5ed;
pub const fnDachora_Func_5: ::core::ffi::c_int = 0xa7f65e;
pub const fnDachora_Func_7: ::core::ffi::c_int = 0xa7f78f;
pub const fnDachora_Func_8: ::core::ffi::c_int = 0xa7f806;
pub const fnDachora_Func_11: ::core::ffi::c_int = 0xa7f935;
pub const fnDachora_Func_12: ::core::ffi::c_int = 0xa7f98c;
#[inline]
unsafe fn gEnemyData(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemyData {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut EnemyData;
}
#[inline]
unsafe fn Get_Dachora(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Dachora {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Dachora;
}
#[inline]
unsafe fn Get_Etecoon(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Etecoon {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Etecoon;
}
#[inline]
unsafe fn Get_Kraid(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Kraid {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Kraid;
}
#[inline]
unsafe fn Get_Phantoon(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Phantoon {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Phantoon;
}
static g_word_A7A916: uint16 = 0x120 as uint16;
static g_word_A7A918: uint16 = 0xa0 as uint16;
static g_word_A7A91A: uint16 = 0x40 as uint16;
static g_word_A7A91C: uint16 = 3 as uint16;
static g_word_A7A920: uint16 = 3 as uint16;
static g_word_A7A922: uint16 = 4 as uint16;
static g_word_A7A926: uint16 = 0x8000 as uint16;
static g_word_A7A928: uint16 = 3 as uint16;
static g_word_A7CD73: uint16 = 0x600 as uint16;
static g_word_A7CD75: uint16 = 0 as uint16;
static g_word_A7CD77: uint16 = 0x1000 as uint16;
static g_word_A7CD79: uint16 = 0 as uint16;
static g_word_A7CD7B: uint16 = 2 as uint16;
static g_word_A7CD7D: uint16 = 7 as uint16;
static g_word_A7CD7F: uint16 = 0 as uint16;
static g_word_A7CD81: uint16 = 0x600 as uint16;
static g_word_A7CD83: uint16 = 0 as uint16;
static g_word_A7CD85: uint16 = 0x1000 as uint16;
static g_word_A7CD87: uint16 = 0 as uint16;
static g_word_A7CD89: uint16 = 0xfffe as uint16;
static g_word_A7CD8B: uint16 = 0xfff9 as uint16;
static g_word_A7CD8D: uint16 = 0 as uint16;
static g_word_A7CD9B: uint16 = 0x40 as uint16;
static g_word_A7CD9D: uint16 = 0xc00 as uint16;
static g_word_A7CD9F: uint16 = 0x100 as uint16;
static g_word_A7CDA1: uint16 = 0xf000 as uint16;
static g_word_A7CDA3: uint16 = 8 as uint16;
static g_byte_A7CDA5: [uint8; 8] = [
    6 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
];
static g_word_A7E900: uint16 = 0xfffd as uint16;
static g_word_A7E902: uint16 = 0 as uint16;
static g_word_A7E906: uint16 = 0 as uint16;
static g_word_A7E904: uint16 = 0xfffc as uint16;
static g_word_A7E908: uint16 = 2 as uint16;
static g_word_A7E90A: uint16 = 0 as uint16;
static g_word_A7E90C: uint16 = 0xfffe as uint16;
static g_word_A7E90E: uint16 = 0 as uint16;
static g_word_A7E910: uint16 = 0x40 as uint16;
static g_word_A7F4C9: uint16 = 0x60 as uint16;
static g_word_A7F4CD: uint16 = 0x78 as uint16;
static g_word_A7F4CF: uint16 = 0x3c as uint16;
static g_word_A7F4D1: uint16 = 1 as uint16;
static g_word_A7F4D3: uint16 = 8 as uint16;
static g_word_A7F4D5: uint16 = 8 as uint16;
static g_word_A7F4D7: uint16 = 0 as uint16;
static g_word_A7F4D9: uint16 = 0 as uint16;
static g_word_A7F4DB: uint16 = 0x1000 as uint16;
pub unsafe fn CallEnemyInstrExtFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        10997485 => {
            Phantoon_Func_1(game);
            return;
        }
        10997598 => {
            Phantoon_Func_4(k, game);
            return;
        }
        10997823 => {
            Phantoon_StartTrackingSamusAndInitEyeTimer(game);
            return;
        }
        10997878 => {
            Phantoon_PickPatternForRound2(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn EnemyInstr_Call_A7(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    CallEnemyInstrExtFunc(
        (*jp.offset(0) as ::core::ffi::c_int | 0xa70000 as ::core::ffi::c_int) as uint32,
        k,
        game,
    );
    return jp.offset(1);
}
pub unsafe fn Enemy_GrappleReact_NoInteract_A7(game: &mut Game) {
    SwitchEnemyAiToMainAi(game);
}
pub unsafe fn Enemy_GrappleReact_CancelBeam_A7(game: &mut Game) {
    Enemy_SwitchToFrozenAi(game);
}
pub unsafe fn Enemy_NormalShotAI_A7(game: &mut Game) {
    NormalEnemyShotAi(game);
}
pub unsafe fn Enemy_NormalPowerBombAI_SkipDeathAnim_A7(game: &mut Game) {
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn Enemy_NormalFrozenAI_A7(game: &mut Game) {
    NormalEnemyFrozenAI(game);
}
pub unsafe fn Kraid_Instr_9(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    if (((*E).base.health as ::core::ffi::c_int
        - (*E).kraid_healths_8ths[3 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        let mut E1: *mut Enemy_Kraid = Get_Kraid(0x40 as ::core::ffi::c_int, game);
        if (*E1).base.current_instruction as ::core::ffi::c_int + 0x75bf as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            return addr_kKraid_Ilist_8A41 as ::core::ffi::c_int as uintptr_t as *const uint16;
        }
    }
    return jp;
}
pub unsafe fn Kraid_Touch_ArmFoot(game: &mut Game) {
    NormalEnemyTouchAi(game);
}
pub unsafe fn Kraid_Func_1(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = 4 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16) =
        -(8 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Kraid_Touch(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn Kraid_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
}
pub unsafe fn KraidsArm_Touch(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) == 0 {
        Kraid_Func_1(game);
        (*gEnemyData(0 as ::core::ffi::c_int, game).offset(4)).ai_var_A =
            (0xa7b89b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        Kraid_Touch(game);
    }
}
pub unsafe fn Kraid_Arm_Shot(mut j: uint16, game: &mut Game) {
    Kraid_SpawnExplosionEproj(j, game);
    let ref mut fresh16 = *((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh16 = (*fresh16 as ::core::ffi::c_int | 0x10 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn sub_A7A92A(game: &mut Game) {
    Kraid_CheckIfDead(game);
}
pub unsafe fn Kraid_CheckIfDead(game: &mut Game) -> uint16 {
    return (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16;
}
pub unsafe fn Kraid_SetEnemyPropsToDead(game: &mut Game) {
    let mut E: *mut Enemy_Kraid = Get_Kraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & 0x50ff as ::core::ffi::c_int
        | 0x700 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Kraid_Init(game: &mut Game) {
    let mut v5: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut j: uint16 = 0;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x604) as *mut LongPtr)).bank =
        -(89 as ::core::ffi::c_int) as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x601) as *mut LongPtr)).bank =
        -(89 as ::core::ffi::c_int) as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x604) as *mut LongPtr)).addr =
        (0xa7c24e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x601) as *mut LongPtr)).addr =
        (0xa7c325 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP;
    if Kraid_CheckIfDead(game) != 0 {
        let mut v0: uint16 = 192 as uint16;
        let mut v1: uint16 = 0 as uint16;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                *(game.g_rom.offset(
                    (((0xa786c7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa786c7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v1 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        let mut i: ::core::ffi::c_int = 2046 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
        let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
        (*E).field_4 = 0 as uint16;
        let mut v4: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
        v5 = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*v5).size = 512 as uint16;
        (*v5).src.addr = addr_kKraidsRoomBg as ::core::ffi::c_int as VoidP;
        (*v5).src.bank = -(89 as ::core::ffi::c_int) as uint8;
        (*v5).vram_dst = (((*(&raw mut game.g_ram as *mut uint8).offset(0x5d)
            as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int)
            + 16128 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            (v4 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
        Kraid_SpawnPlmToClearCeiling(game);
        Kraid_ClearSomeSpikes(game);
        (*E).kraid_var_A =
            (0xa7c715 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 67 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x941) as *mut uint16) = 2 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) as *mut uint8
            as *mut uint16) = 1 as uint16;
        let mut E_0: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
        (*E_0).kraid_min_y_pos_eject = 324 as uint16;
        let mut v7: uint16 = 0 as uint16;
        let mut v8: uint16 =
            ((*E_0).base.health as ::core::ffi::c_int >> 3 as ::core::ffi::c_int) as uint16;
        let mut r18: uint16 = v8;
        loop {
            (*Get_Kraid(v7 as ::core::ffi::c_int, game)).kraid_healths_8ths
                [0 as ::core::ffi::c_int as usize] = v8;
            v8 = (v8 as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
            v7 = (v7 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v7 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        Kraid_SetupGfxWithTilePrioClear(0xdfff as uint16, game);
        let mut v10: uint16 =
            ((*E_0).base.health as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16;
        (*E_0).kraid_healths_4ths[0 as ::core::ffi::c_int as usize] = v10;
        let mut v11: uint16 = ((*E_0).kraid_healths_4ths[0 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int
            + v10 as ::core::ffi::c_int) as uint16;
        (*E_0).kraid_healths_4ths[1 as ::core::ffi::c_int as usize] = v11;
        let mut v12: uint16 = ((*E_0).kraid_healths_4ths[0 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int
            + v11 as ::core::ffi::c_int) as uint16;
        (*E_0).kraid_healths_4ths[2 as ::core::ffi::c_int as usize] = v12;
        (*E_0).kraid_healths_4ths[3 as ::core::ffi::c_int as usize] =
            ((*E_0).kraid_healths_4ths[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                + v12 as ::core::ffi::c_int) as uint16;
        (*E_0).field_2E[8 as ::core::ffi::c_int as usize] = 0 as uint16;
        (*E_0).base.x_pos = 176 as uint16;
        (*E_0).base.y_pos = 592 as uint16;
        (*E_0).base.properties = ((*E_0).base.properties as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int)
            as uint16;
        (*E_0).kraid_var_A =
            (0xa7c865 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E_0).kraid_var_F = 300 as uint16;
        (*E_0).kraid_next =
            (0xa7c86b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E_0).kraid_var_C = 64 as uint16;
        DisableMinimapAndMarkBossRoomAsExplored(game);
        j = 62 as uint16;
        while j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(
                ((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 2016 as ::core::ffi::c_int)
                    as isize,
            ) = 824 as uint16;
            j = (j as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 5 as uint16;
        let mut v14: uint16 = 0 as uint16;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(
                ((v14 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 176 as ::core::ffi::c_int)
                    as isize,
            ) = *(game.g_rom.offset(
                (((0xa7aaa6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa7aaa6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v14 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v14 = (v14 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v14 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    };
}
pub unsafe fn Kraid_SetupGfxWithTilePrioClear(mut r18: uint16, game: &mut Game) {
    DecompressToMem(
        0xb9fa38 as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x4000),
        game,
    );
    DecompressToMem(
        0xb9fe3e as uint32,
        (&raw mut game.g_ram as *mut uint8).offset(0x2000),
        game,
    );
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    (*E).kraid_hurt_frame = 0 as uint16;
    (*E).kraid_hurt_frame_timer = 0 as uint16;
    let mut v1: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(
            ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 1024 as ::core::ffi::c_int)
                as isize,
        ) = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            & 0xdfff as ::core::ffi::c_int) as uint16;
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v1 as ::core::ffi::c_int - 1536 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    let mut v2: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(v2 as isize) =
            (r18 as ::core::ffi::c_int
                & (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000))
                    .xray_tilemaps[v2 as usize] as ::core::ffi::c_int) as uint16;
        v2 = v2.wrapping_add(1);
        if !(((v2 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 2048 as ::core::ffi::c_int)
            as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
pub unsafe fn KraidsArm_Init(game: &mut Game) {
    if Kraid_CheckIfDead(game) != 0 {
        Kraid_SetEnemyPropsToDead(game);
    } else {
        let mut v0: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
        (*v0.offset(1)).palette_index = (*v0).palette_index;
        (*v0.offset(1)).ai_var_A =
            (0xa7ba2d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*v0.offset(1)).current_instruction =
            addr_kKraid_Ilist_8AA4 as ::core::ffi::c_int as uint16;
        (*v0.offset(1)).instruction_timer = 1 as uint16;
        (*v0.offset(1)).ai_var_B = 0 as uint16;
    };
}
pub unsafe fn KraidsTopLint_Init(game: &mut Game) {
    if Kraid_CheckIfDead(game) != 0 {
        Kraid_SetEnemyPropsToDead(game);
    } else {
        let mut v0: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
        (*v0.offset(2)).palette_index = (*v0).palette_index;
        (*v0.offset(2)).instruction_timer = 0x7fff as uint16;
        (*v0.offset(2)).current_instruction =
            addr_kKraid_Ilist_8AFE as ::core::ffi::c_int as uint16;
        (*v0.offset(2)).spritemap_pointer = addr_kKraid_Sprmap_A5DF as ::core::ffi::c_int as uint16;
        (*v0.offset(2)).ai_var_A = addr_locret_A7B831 as ::core::ffi::c_int as uint16;
        (*v0.offset(2)).ai_preinstr = 0x7fff as uint16;
        (*v0.offset(2)).ai_var_C = 0 as uint16;
    };
}
pub unsafe fn KraidsMiddleLint_Init(game: &mut Game) {
    if Kraid_CheckIfDead(game) != 0 {
        Kraid_SetEnemyPropsToDead(game);
    } else {
        let mut v0: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
        (*v0.offset(3)).palette_index = (*v0).palette_index;
        (*v0.offset(3)).instruction_timer = 0x7fff as uint16;
        (*v0.offset(3)).current_instruction =
            addr_kKraid_Ilist_8AFE as ::core::ffi::c_int as uint16;
        (*v0.offset(3)).spritemap_pointer = addr_kKraid_Sprmap_A5DF as ::core::ffi::c_int as uint16;
        (*v0.offset(3)).ai_var_A = addr_locret_A7B831 as ::core::ffi::c_int as uint16;
        (*v0.offset(3)).ai_var_C = -(16 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn KraidsBottomLint_Init(game: &mut Game) {
    if Kraid_CheckIfDead(game) != 0 {
        Kraid_SetEnemyPropsToDead(game);
    } else {
        let mut v0: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
        (*v0.offset(4)).palette_index = (*v0).palette_index;
        (*v0.offset(4)).instruction_timer = 0x7fff as uint16;
        (*v0.offset(4)).current_instruction =
            addr_kKraid_Ilist_8AFE as ::core::ffi::c_int as uint16;
        (*v0.offset(4)).spritemap_pointer = addr_kKraid_Sprmap_A5DF as ::core::ffi::c_int as uint16;
        (*v0.offset(4)).ai_var_A = addr_locret_A7B831 as ::core::ffi::c_int as uint16;
        (*v0.offset(4)).ai_var_C = -(16 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn KraidsFoot_Init(game: &mut Game) {
    if Kraid_CheckIfDead(game) != 0 {
        Kraid_SetEnemyPropsToDead(game);
    } else {
        let mut palette_index: uint16 = (*Get_Kraid(0 as ::core::ffi::c_int, game))
            .base
            .palette_index;
        let mut E: *mut Enemy_Kraid = Get_Kraid(0x140 as ::core::ffi::c_int, game);
        (*E).base.palette_index = palette_index;
        (*E).base.current_instruction = addr_kKraid_Ilist_86E7 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).kraid_var_A =
            (0xa7ba2d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).kraid_next = 0 as uint16;
    };
}
pub unsafe fn Kraid_Main(game: &mut Game) {
    Kraid_Shot_Mouth(game);
    Kraid_Palette_Handling(game);
    Kraid_Shot_Body(game);
    Kraid_Enemy_Touch(game);
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = ((*E).base.x_width
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x91d) as *mut uint16) as ::core::ffi::c_int
        - (*E).base.x_pos as ::core::ffi::c_int)
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) as ::core::ffi::c_int
            - (*E).base.y_pos as ::core::ffi::c_int
            + 152 as ::core::ffi::c_int) as uint16;
    CallKraidFunc(
        ((*E).kraid_var_A as ::core::ffi::c_int | 0xa70000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn Kraid_GetsBig_BreakCeilingPlatforms(game: &mut Game) {
    let mut kraid_var_F: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16) as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        Kraid_SpawnRandomQuakeProjs(game);
    }
    let mut v0: uint16 = 1 as uint16;
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    if (*E).base.y_pos as ::core::ffi::c_int & 2 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        v0 = -(1 as ::core::ffi::c_int) as uint16;
    }
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int + v0 as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    if !((*E).base.y_pos as ::core::ffi::c_int & 3 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        || {
            kraid_var_F = (*E).kraid_var_F;
            (kraid_var_F as ::core::ffi::c_int - 18 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
        })
    {
        SpawnEprojWithGfx(
            *(game.g_rom.offset(
                (((0xa7acb3 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa7acb3 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((kraid_var_F as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            addr_kEproj_RocksFallingKraidCeiling as ::core::ffi::c_int as uint16,
            game,
        );
        match (*E).kraid_var_F as ::core::ffi::c_int >> 1 as ::core::ffi::c_int {
            0 => {
                SpawnHardcodedPlm(
                    SpawnHardcodedPlmArgs {
                        field_0: 0x6 as uint8,
                        field_1: 0x12 as uint8,
                        field_2: 0xb7b3 as VoidP,
                    },
                    game,
                );
            }
            1 => {
                SpawnHardcodedPlm(
                    SpawnHardcodedPlmArgs {
                        field_0: 0xd as uint8,
                        field_1: 0x12 as uint8,
                        field_2: 0xb7ab as VoidP,
                    },
                    game,
                );
            }
            2 => {
                SpawnHardcodedPlm(
                    SpawnHardcodedPlmArgs {
                        field_0: 0x2 as uint8,
                        field_1: 0x12 as uint8,
                        field_2: 0xb7a3 as VoidP,
                    },
                    game,
                );
            }
            3 => {
                SpawnHardcodedPlm(
                    SpawnHardcodedPlmArgs {
                        field_0: 0xa as uint8,
                        field_1: 0x12 as uint8,
                        field_2: 0xb7b3 as VoidP,
                    },
                    game,
                );
            }
            4 => {
                SpawnHardcodedPlm(
                    SpawnHardcodedPlmArgs {
                        field_0: 0x5 as uint8,
                        field_1: 0x12 as uint8,
                        field_2: 0xb7ab as VoidP,
                    },
                    game,
                );
            }
            5 => {
                SpawnHardcodedPlm(
                    SpawnHardcodedPlmArgs {
                        field_0: 0xc as uint8,
                        field_1: 0x12 as uint8,
                        field_2: 0xb7b3 as VoidP,
                    },
                    game,
                );
            }
            6 => {
                SpawnHardcodedPlm(
                    SpawnHardcodedPlmArgs {
                        field_0: 0x3 as uint8,
                        field_1: 0x12 as uint8,
                        field_2: 0xb7ab as VoidP,
                    },
                    game,
                );
            }
            7 => {
                SpawnHardcodedPlm(
                    SpawnHardcodedPlmArgs {
                        field_0: 0xb as uint8,
                        field_1: 0x12 as uint8,
                        field_2: 0xb7ab as VoidP,
                    },
                    game,
                );
            }
            8 => {
                SpawnHardcodedPlm(
                    SpawnHardcodedPlmArgs {
                        field_0: 0x4 as uint8,
                        field_1: 0x12 as uint8,
                        field_2: 0xb7b3 as VoidP,
                    },
                    game,
                );
            }
            _ => {}
        }
        (*E).kraid_var_F =
            ((*E).kraid_var_F as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
    if (*E).base.y_pos as ::core::ffi::c_int - 296 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).kraid_var_A =
            (0xa7ad3a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Kraid_GetsBig_SetBG2TilemapPrioBits(game: &mut Game) {
    let mut v0: uint16 = 0 as uint16;
    loop {
        let ref mut fresh11 = *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh11 = (*fresh11 as ::core::ffi::c_int | 0x2000 as ::core::ffi::c_int) as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int - 4096 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    let mut E: *mut Enemy_Kraid = Get_Kraid(0x40 as ::core::ffi::c_int, game);
    (*E).base.properties =
        ((*E).base.properties as ::core::ffi::c_int & !(0x400 as ::core::ffi::c_int)) as uint16;
    (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_var_A =
        (0xa7ad61 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Kraid_UpdateBg2TilemapTopHalf(game);
}
pub unsafe fn Kraid_GetsBig_FinishUpdateBg2Tilemap(game: &mut Game) {
    (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_var_A =
        (0xa7ad8e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut E5: *mut Enemy_Kraid = Get_Kraid(0x140 as ::core::ffi::c_int, game);
    (*E5).base.instruction_timer = 1 as uint16;
    (*E5).base.current_instruction = addr_kKraid_Ilist_86ED as ::core::ffi::c_int as uint16;
    let mut E2: *mut Enemy_Kraid = Get_Kraid(0x80 as ::core::ffi::c_int, game);
    (*E2).base.current_instruction = addr_kKraid_Ilist_8B04 as ::core::ffi::c_int as uint16;
    let mut E3: *mut Enemy_Kraid = Get_Kraid(0xc0 as ::core::ffi::c_int, game);
    (*E3).base.current_instruction = addr_kKraid_Ilist_8B04 as ::core::ffi::c_int as uint16;
    let mut E: *mut Enemy_Kraid = Get_Kraid(0x100 as ::core::ffi::c_int, game);
    (*E).base.current_instruction = addr_kKraid_Ilist_8B04 as ::core::ffi::c_int as uint16;
    (*E2).base.spritemap_pointer = addr_kKraid_Sprmap_8C6C as ::core::ffi::c_int as uint16;
    (*E3).base.spritemap_pointer = addr_kKraid_Sprmap_8C6C as ::core::ffi::c_int as uint16;
    (*E).base.spritemap_pointer = addr_kKraid_Sprmap_8C6C as ::core::ffi::c_int as uint16;
    Kraid_UpdateBG2TilemapBottomHalf(game);
}
pub unsafe fn Kraid_DrawRoomBg(game: &mut Game) {
    let mut v3: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v0: uint16 = 192 as uint16;
    let mut v1: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(
                (((0xa786c7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa786c7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v1 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16) = 0 as uint16;
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    v3 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v3).size = 512 as uint16;
    (*v3).src.addr = addr_kKraidsRoomBg as ::core::ffi::c_int as VoidP;
    (*v3).src.bank = -(89 as ::core::ffi::c_int) as uint8;
    (*v3).vram_dst = (((*(&raw mut game.g_ram as *mut uint8).offset(0x5d) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int)
        + 16128 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v2 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Kraid_GetsBig_DrawRoomBg(game: &mut Game) {
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    (*E).kraid_var_A = (0xa7ae23 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).kraid_var_E = 0 as uint16;
    (*E).kraid_var_F = 0 as uint16;
    Kraid_DrawRoomBg(game);
}
pub unsafe fn Kraid_SetLintYAndRandomThinkTimer(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    (*Get_Kraid(0x80 as ::core::ffi::c_int, game)).base.y_pos =
        ((*E).base.y_pos as ::core::ffi::c_int - 20 as ::core::ffi::c_int) as uint16;
    (*Get_Kraid(0xc0 as ::core::ffi::c_int, game)).base.y_pos =
        ((*E).base.y_pos as ::core::ffi::c_int + 46 as ::core::ffi::c_int) as uint16;
    (*Get_Kraid(0x100 as ::core::ffi::c_int, game)).base.y_pos =
        ((*E).base.y_pos as ::core::ffi::c_int + 112 as ::core::ffi::c_int) as uint16;
    v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int) as int16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v1 = 2 as int16;
    }
    (*E).kraid_thinking = ((v1 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Kraid_GetsBig_Thinking_Setup(game: &mut Game) {
    (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_var_A =
        (0xa7aec4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Kraid_SetLintYAndRandomThinkTimer(game);
}
pub unsafe fn Kraid_Mainloop_Thinking_Setup(game: &mut Game) {
    (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_var_A =
        (0xa7aea4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Kraid_SetLintYAndRandomThinkTimer(game);
}
pub unsafe fn Kraid_Lints_Enable(mut k: uint16, mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Kraid = Get_Kraid(k as ::core::ffi::c_int, game);
    (*E).kraid_var_F = a;
    (*E).kraid_var_A = (0xa7b923 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).kraid_next = (0xa7b832 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).kraid_var_B = 0 as uint16;
}
pub unsafe fn Kraid_GetsBig_FadeInRoomBg(game: &mut Game) {
    if AdvancePaletteFade_BgPalette6(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        Kraid_GetsBig_Thinking_Setup(game);
        Kraid_Lints_Enable(0x80 as uint16, g_word_A7A916, game);
        Kraid_Lints_Enable(0xc0 as uint16, g_word_A7A918, game);
        Kraid_Lints_Enable(0x100 as uint16, g_word_A7A91A, game);
        let mut E6: *mut Enemy_Kraid = Get_Kraid(0x180 as ::core::ffi::c_int, game);
        (*E6).kraid_next =
            (0xa7bd60 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut E7: *mut Enemy_Kraid = Get_Kraid(0x1c0 as ::core::ffi::c_int, game);
        (*E7).kraid_next =
            (0xa7bd60 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E6).kraid_var_A =
            (0xa7b92d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E7).kraid_var_A =
            (0xa7b92d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E6).kraid_var_F = 64 as uint16;
        (*E7).kraid_var_F = 128 as uint16;
        (*Get_Kraid(0x40 as ::core::ffi::c_int, game)).kraid_var_C = 1 as uint16;
        let mut E0: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
        (*E0).kraid_var_B = addr_stru_A796DA as ::core::ffi::c_int as uint16;
        (*E0).kraid_target_x = 288 as uint16;
        let mut E5: *mut Enemy_Kraid = Get_Kraid(0x140 as ::core::ffi::c_int, game);
        (*E5).kraid_var_A =
            (0xa7bb6e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E5).base.instruction_timer = 1 as uint16;
        (*E5).base.current_instruction = addr_kKraid_Ilist_8887 as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn Kraid_Mainloop_Thinking(game: &mut Game) {
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    let mut kraid_thinking: uint16 = (*E).kraid_thinking;
    if kraid_thinking != 0 {
        let mut v2: uint16 =
            (kraid_thinking as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        (*E).kraid_thinking = v2;
        if v2 == 0 {
            (*E).kraid_var_A =
                (0xa7bbea as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).kraid_var_B = addr_stru_A796DA as ::core::ffi::c_int as uint16;
            (*E).kraid_var_C = (*(game.g_rom.offset(
                (((0xa796d2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa796d2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut KraidInstrList))
                .timer;
        }
    }
}
pub unsafe fn Kraid_GetsBig_Thinking(game: &mut Game) {
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    let mut kraid_thinking: uint16 = (*E).kraid_thinking;
    if kraid_thinking != 0 {
        let mut v2: uint16 =
            (kraid_thinking as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        (*E).kraid_thinking = v2;
        if v2 == 0 {
            (*E).kraid_var_A =
                (0xa7aee4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).kraid_var_B = addr_stru_A796DA as ::core::ffi::c_int as uint16;
            (*E).kraid_var_C = (*(game.g_rom.offset(
                (((0xa796d2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa796d2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut KraidInstrList))
                .timer;
        }
    }
}
pub unsafe fn Kraid_Shot_MouthIsOpen(game: &mut Game) {
    let mut v2: uint16 = 0;
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    if Kraid_ProcessKraidInstr(game) as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
        (*E).kraid_var_A =
            (0xa7aea4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).kraid_var_C = 90 as uint16;
        let mut kraid_mouth_flags: uint16 = (*E).kraid_mouth_flags;
        if kraid_mouth_flags as ::core::ffi::c_int & 4 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            && {
                v2 =
                    (kraid_mouth_flags as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as uint16;
                (*E).kraid_mouth_flags = v2;
                v2 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
            }
        {
            (*E).kraid_var_A =
                (0xa7b92d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).kraid_var_F = 64 as uint16;
            (*E).kraid_next =
                (0xa7b6bf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).field_2 = 2 as uint16;
        } else {
            (*E).kraid_mouth_flags = 0 as uint16;
        }
    }
}
pub unsafe fn Kraid_ProcessKraidInstr(game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    let mut result: uint16 = (*E).kraid_var_C;
    if result != 0 {
        let fresh1 = (*E).kraid_var_C;
        (*E).kraid_var_C = (*E).kraid_var_C.wrapping_sub(1);
        if fresh1 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            return Kraid_ExecuteInstr(game);
        }
    }
    return result;
}
pub unsafe fn KraidInstr_PlayRoarSfx(game: &mut Game) {
    QueueSfx2_Max6(0x2d as uint16, game);
}
pub unsafe fn KraidInstr_PlayDyingSfx(game: &mut Game) {
    QueueSfx2_Max15(0x2e as uint16, game);
}
pub unsafe fn CallKraidInstr(mut ea: uint32, game: &mut Game) {
    match ea {
        10989460 => {
            KraidInstr_PlayRoarSfx(game);
            return;
        }
        10989471 => {
            KraidInstr_PlayDyingSfx(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn Kraid_ExecuteInstr(game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    loop {
        let mut kraid_var_B: uint16 = (*E).kraid_var_B;
        let mut v2: *const uint16 = RomPtr_A7(kraid_var_B as uint16_t, game) as *const uint16;
        let mut result: uint16 = *v2;
        if *v2 as ::core::ffi::c_int != 0xffff as ::core::ffi::c_int {
            if ((*v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                CallKraidInstr(
                    (result as ::core::ffi::c_int | 0xa70000 as ::core::ffi::c_int) as uint32,
                    game,
                );
                (*E).kraid_var_B =
                    ((*E).kraid_var_B as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            } else {
                (*E).kraid_var_C = result;
                (*E).kraid_var_B =
                    (kraid_var_B as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
                let mut v4: uint16 = *v2.offset(1);
                let mut v5: uint16 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
                (*gVramWriteEntry(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                        as ::core::ffi::c_int,
                    game,
                ))
                .size = 704 as uint16;
                v5 = (v5 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                (*gVramWriteEntry(v5 as ::core::ffi::c_int, game)).size = v4;
                v5 = (v5 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                let fresh0 = v5;
                v5 = v5.wrapping_add(1);
                *(&raw mut (*(gVramWriteEntry
                    as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
                    fresh0 as ::core::ffi::c_int,
                    game,
                ))
                .size as *mut uint8)
                    .offset(0) = -(89 as ::core::ffi::c_int) as uint8;
                (*gVramWriteEntry(v5 as ::core::ffi::c_int, game)).size =
                    ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
                        & 0xfc as ::core::ffi::c_int)
                        << 8 as ::core::ffi::c_int) as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
                    (v5 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                return 1 as uint16;
            }
        } else {
            return result;
        }
    }
}
pub unsafe fn Kraid_Shot_Mouth(game: &mut Game) {
    let mut v8: uint16 = 0;
    let mut v4: uint16 = 0;
    let mut v5: *const uint8 = ::core::ptr::null::<uint8>();
    let mut r22: uint16 = 0;
    let mut r20: uint16 = 0;
    let mut r18: uint16 = 0;
    let mut current_block: u64;
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    if (*E).kraid_var_A as ::core::ffi::c_int + 0x3ac9 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        return;
    }
    let mut v2: *const uint8 = RomPtr_A7(
        ((*E).kraid_var_B as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16_t,
        game,
    );
    if *(v2.offset(6) as *mut uint16) as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
        v3 = 0 as int16;
    } else {
        v4 = 0;
        v4 = *(v2.offset(6) as *mut uint16);
        (*E).kraid_var_E = 1 as uint16;
        v3 = 0 as int16;
        v5 = ::core::ptr::null::<uint8>();
        v5 = RomPtr_A7(v4 as uint16_t, game);
        r22 = ((*E).base.x_pos as ::core::ffi::c_int + *(v5 as *mut uint16) as ::core::ffi::c_int)
            as uint16;
        r20 = ((*E).base.y_pos as ::core::ffi::c_int
            + *(v5.offset(2) as *mut uint16) as ::core::ffi::c_int) as uint16;
        r18 = ((*E).base.y_pos as ::core::ffi::c_int
            + *(v5.offset(6) as *mut uint16) as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xcce) as *mut uint16) != 0 {
            let mut v6: uint16 = (2 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0xcce) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
            loop {
                let mut v7: ::core::ffi::c_int = 0;
                v7 = v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                if !(*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                    .offset(v7 as isize) as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                        .offset(v7 as isize) as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int
                    - r18 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                    || *((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                        .offset(v7 as isize) as ::core::ffi::c_int
                        + *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                            .offset(v7 as isize) as ::core::ffi::c_int
                        - r20 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0
                    || *((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
                        .offset(v7 as isize) as ::core::ffi::c_int
                        + *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
                            .offset(v7 as isize) as ::core::ffi::c_int
                        - r22 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0)
                {
                    v8 = 0;
                    v8 = *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
                        .offset(v7 as isize);
                    if v8 as ::core::ffi::c_int & 0xf00 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                    {
                        current_block = 10915565897473588658;
                    } else if v8 as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                    {
                        (*E).kraid_mouth_flags = ((*E).kraid_mouth_flags as ::core::ffi::c_int
                            | 1 as ::core::ffi::c_int)
                            as uint16;
                        current_block = 10915565897473588658;
                    } else {
                        current_block = 2822809649030392525;
                    }
                    match current_block {
                        2822809649030392525 => {}
                        _ => {
                            *((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16) =
                                (v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
                            NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
                            let ref mut fresh15 = *((&raw mut game.g_ram as *mut uint8)
                                .offset(0xc04)
                                as *mut uint16)
                                .offset(
                                    (v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                                );
                            *fresh15 = (*fresh15 as ::core::ffi::c_int | 0x10 as ::core::ffi::c_int)
                                as uint16;
                            v3 = 1 as int16;
                            v6 = 0 as uint16;
                        }
                    }
                }
                v6 = (v6 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
                if v6 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    break;
                }
            }
        }
    }
    if v3 != 0 {
        (*E).kraid_hurt_frame = 6 as uint16;
        (*E).kraid_hurt_frame_timer = 2 as uint16;
        let mut kraid_mouth_flags: uint16 = (*E).kraid_mouth_flags;
        if kraid_mouth_flags as ::core::ffi::c_int & 2 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).kraid_mouth_flags =
                (kraid_mouth_flags as ::core::ffi::c_int | 4 as ::core::ffi::c_int) as uint16;
        }
        if (*E).base.health as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            && (*E).kraid_var_A as ::core::ffi::c_int + 0x3ca0 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
        {
            (*E).kraid_var_A =
                (0xa7c360 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).kraid_mouth_flags = 0 as uint16;
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                | kEnemyProps_Tangible as ::core::ffi::c_int)
                as uint16;
            Kraid_SetupGfxWithTilePrioClear(!(0x2000 as ::core::ffi::c_int) as uint16, game);
            let mut v12: uint16 = 0 as uint16;
            loop {
                let mut EL: *mut Enemy_Kraid = Get_Kraid(v12 as ::core::ffi::c_int, game);
                (*EL).base.properties = ((*EL).base.properties as ::core::ffi::c_int
                    | kEnemyProps_Tangible as ::core::ffi::c_int)
                    as uint16;
                v12 = (v12 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
                if !(((v12 as ::core::ffi::c_int - 384 as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int)
                {
                    break;
                }
            }
            if (*E).kraid_var_B as ::core::ffi::c_int + 0x68f2 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                (*E).kraid_var_B =
                    ((*E).kraid_var_B as ::core::ffi::c_int + 60 as ::core::ffi::c_int) as uint16;
            }
        }
    }
}
pub unsafe fn Kraid_SpawnExplosionEproj(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v1 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v1 as isize),
    };
    let mut v2: uint16 = (if *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0x200 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        6 as ::core::ffi::c_int
    } else {
        29 as ::core::ffi::c_int
    }) as uint16;
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        v2,
        game,
    );
    QueueSfx1_Max6(0x3d as uint16, game);
}
pub unsafe fn Kraid_Enemy_Touch(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    if (*E).kraid_var_A as ::core::ffi::c_int + 0x3ca0 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut r18: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
        v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
            as ::core::ffi::c_int
            - (*E).base.y_pos as ::core::ffi::c_int) as int16;
        let mut i: ::core::ffi::c_int = 0;
        i = 0 as ::core::ffi::c_int;
        loop {
            let mut v3: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
            if (v1 as ::core::ffi::c_int
                - *(game.g_rom.offset(
                    (((0xa7b161 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa7b161 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
                || (v1 as ::core::ffi::c_int
                    - *(game.g_rom.offset(
                        (((0xa7b161 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa7b161 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset((v3 + 2 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int
            {
                break;
            }
            i += 4 as ::core::ffi::c_int;
        }
        if (((*E).base.x_pos as ::core::ffi::c_int
            + *(game.g_rom.offset(
                (((0xa7b161 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa7b161 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - 40 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                let ref mut fresh13 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
                *fresh13 = (*fresh13 as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) =
                    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
            }
            let mut kraid_min_y_pos_eject: uint16 =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int
                    - 8 as ::core::ffi::c_int) as uint16;
            if ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - 8 as ::core::ffi::c_int
                - (*E).kraid_min_y_pos_eject as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                kraid_min_y_pos_eject = (*E).kraid_min_y_pos_eject;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) =
                kraid_min_y_pos_eject;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) =
                kraid_min_y_pos_eject;
            Kraid_Func_1(game);
            if *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) == 0 {
                NormalEnemyTouchAi(game);
            }
        }
    }
}
pub unsafe fn Kraid_Shot_Body(game: &mut Game) {
    let mut v11: int16 = 0;
    let mut j: uint16 = 0;
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    if (*E).kraid_var_A as ::core::ffi::c_int + 0x3ac9 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).kraid_var_E = 0 as uint16;
        (*E).kraid_mouth_flags =
            ((*E).kraid_mouth_flags as ::core::ffi::c_int & !(1 as ::core::ffi::c_int)) as uint16;
        let mut R48: uint16 = 0 as uint16;
        let mut v2: *const uint8 = RomPtr_A7(
            ((*E).kraid_var_B as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16_t,
            game,
        );
        let mut v3: *const uint8 = RomPtr_A7(*(v2.offset(4) as *mut uint16), game);
        let mut r22: uint16 = ((*E).base.x_pos as ::core::ffi::c_int
            + *(v3 as *mut uint16) as ::core::ffi::c_int) as uint16;
        let mut r20: uint16 = ((*E).base.y_pos as ::core::ffi::c_int
            + *(v3.offset(2) as *mut uint16) as ::core::ffi::c_int)
            as uint16;
        let mut r18: uint16 = ((*E).base.y_pos as ::core::ffi::c_int
            + *(v3.offset(6) as *mut uint16) as ::core::ffi::c_int)
            as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xcce) as *mut uint16) != 0 {
            let mut i: ::core::ffi::c_int = 2 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0xcce) as *mut uint16)
                    as ::core::ffi::c_int;
            while i >= 0 as ::core::ffi::c_int {
                let mut v5: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
                let mut v10: ::core::ffi::c_int = 0;
                let mut current_block_12: u64;
                if *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                    .offset(v5 as isize) as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                        .offset(v5 as isize) as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int
                    - r18 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    if *((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                        .offset(v5 as isize) as ::core::ffi::c_int
                        + *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                            .offset(v5 as isize) as ::core::ffi::c_int
                        - r20 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0
                        && *((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
                            .offset(v5 as isize) as ::core::ffi::c_int
                            + *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
                                .offset(v5 as isize)
                                as ::core::ffi::c_int
                            - r22 as ::core::ffi::c_int
                            & 0x8000 as ::core::ffi::c_int
                            == 0
                    {
                        current_block_12 = 2894936153070068674;
                    } else {
                        current_block_12 = 11194104282611034094;
                    }
                } else {
                    v10 = 0;
                    v10 = i >> 1 as ::core::ffi::c_int;
                    r18 = (*((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
                        .offset(v10 as isize) as ::core::ffi::c_int
                        + *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
                            .offset(v10 as isize) as ::core::ffi::c_int)
                        as uint16;
                    v11 = (*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                        .offset(v10 as isize) as ::core::ffi::c_int
                        - (*E).base.y_pos as ::core::ffi::c_int) as int16;
                    j = 0 as uint16;
                    loop {
                        let mut v13: ::core::ffi::c_int =
                            j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                        if (v11 as ::core::ffi::c_int
                            - *(game.g_rom.offset(
                                (((0xa7b161 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                    << 15 as ::core::ffi::c_int
                                    | 0xa7b161 as ::core::ffi::c_int
                                        & 0x7fff as ::core::ffi::c_int)
                                    & 0x3fffff as ::core::ffi::c_int)
                                    as isize,
                            ) as *const uint8 as *mut uint16)
                                .offset(v13 as isize)
                                as ::core::ffi::c_int) as int16
                            as ::core::ffi::c_int
                            >= 0 as ::core::ffi::c_int
                            || (v11 as ::core::ffi::c_int
                                - *(game.g_rom.offset(
                                    (((0xa7b161 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                        << 15 as ::core::ffi::c_int
                                        | 0xa7b161 as ::core::ffi::c_int
                                            & 0x7fff as ::core::ffi::c_int)
                                        & 0x3fffff as ::core::ffi::c_int)
                                        as isize,
                                ) as *const uint8
                                    as *mut uint16)
                                    .offset((v13 + 2 as ::core::ffi::c_int) as isize)
                                    as ::core::ffi::c_int) as int16
                                as ::core::ffi::c_int
                                >= 0 as ::core::ffi::c_int
                        {
                            break;
                        }
                        j = (j as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
                    }
                    if (((*E).base.x_pos as ::core::ffi::c_int
                        + *(game.g_rom.offset(
                            (((0xa7b161 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                << 15 as ::core::ffi::c_int
                                | 0xa7b161 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                                & 0x3fffff as ::core::ffi::c_int)
                                as isize,
                        ) as *const uint8 as *mut uint16)
                            .offset(
                                ((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                                    + 1 as ::core::ffi::c_int)
                                    as isize,
                            ) as ::core::ffi::c_int
                        - r18 as ::core::ffi::c_int) as int16
                        as ::core::ffi::c_int)
                        < 0 as ::core::ffi::c_int
                    {
                        current_block_12 = 2894936153070068674;
                    } else {
                        current_block_12 = 11194104282611034094;
                    }
                }
                match current_block_12 {
                    2894936153070068674 => {
                        Kraid_SpawnExplosionEproj(i as uint16, game);
                        let mut v6: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
                        let ref mut fresh14 = *((&raw mut game.g_ram as *mut uint8).offset(0xc04)
                            as *mut uint16)
                            .offset(v6 as isize);
                        *fresh14 =
                            (*fresh14 as ::core::ffi::c_int | 0x10 as ::core::ffi::c_int) as uint16;
                        if *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
                            .offset(v6 as isize) as ::core::ffi::c_int
                            & 0x10 as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_int
                        {
                            (*E).kraid_mouth_flags = ((*E).kraid_mouth_flags as ::core::ffi::c_int
                                | 1 as ::core::ffi::c_int)
                                as uint16;
                        }
                        R48 = R48.wrapping_add(1);
                    }
                    _ => {}
                }
                i -= 2 as ::core::ffi::c_int;
            }
        }
        if R48 != 0 {
            if (*E).kraid_var_A as ::core::ffi::c_int
                == 0xa7aea4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            {
                (*E).kraid_var_A =
                    (0xa7b6bf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                let mut kraid_mouth_flags: uint16 = (*E).kraid_mouth_flags;
                if kraid_mouth_flags as ::core::ffi::c_int & 1 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    (*E).kraid_mouth_flags = (kraid_mouth_flags as ::core::ffi::c_int
                        | 0x302 as ::core::ffi::c_int)
                        as uint16;
                }
            }
        }
    }
}
pub unsafe fn Kraid_Palette_Handling(game: &mut Game) {
    let mut current_block: u64;
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    if (*E).base.health as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).kraid_hurt_frame = (*E).base.health;
    } else {
        if (*E).kraid_hurt_frame != 0 {
            let mut v1: uint16 = ((*E).kraid_hurt_frame_timer as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as uint16;
            (*E).kraid_hurt_frame_timer = v1;
            if v1 == 0 {
                (*E).kraid_hurt_frame_timer = 2 as uint16;
                (*E).kraid_hurt_frame = (*E).kraid_hurt_frame.wrapping_sub(1);
                current_block = 530921790263604621;
            } else {
                current_block = 5720623009719927633;
            }
        } else {
            current_block = 5720623009719927633;
        }
        match current_block {
            530921790263604621 => {}
            _ => {
                return;
            }
        }
    }
    Kraid_HurtFlash_Handling(game);
    Kraid_HealthBasedPaletteHandling(game);
}
pub unsafe fn Kraid_HurtFlash_Handling(game: &mut Game) {
    let mut v0: uint16 = 0 as uint16;
    if (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_hurt_frame as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v0 = 32 as uint16;
    }
    let mut v1: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
            ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 240 as ::core::ffi::c_int)
                as isize,
        ) = *(game.g_rom.offset(
            (((0xa7b513 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7b513 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v1 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
pub unsafe fn Kraid_HealthBasedPaletteHandling(game: &mut Game) {
    let mut v0: uint16 = 0 as uint16;
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    if (*E).kraid_hurt_frame as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut v2: uint16 = 14 as uint16;
        let mut health: uint16 = (*E).base.health;
        while !((health as ::core::ffi::c_int
            - (*Get_Kraid(v2 as ::core::ffi::c_int, game)).kraid_healths_8ths
                [0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int)
        {
            v2 = (v2 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
            if !(v2 != 0) {
                break;
            }
        }
        v0 = (16 as ::core::ffi::c_int * (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int))
            as uint16;
    }
    let mut v4: uint16 = 0 as uint16;
    loop {
        let mut v5: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v6 + 112 as ::core::ffi::c_int) as isize) = *(game.g_rom.offset(
            (((0xa7b3d3 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7b3d3 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset(v5 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v6 + 240 as ::core::ffi::c_int) as isize) = *(game.g_rom.offset(
            (((0xa7b513 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7b513 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset(v5 as isize);
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v4 = (v4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v4 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
pub unsafe fn Kraid_Instr_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return jp;
}
pub unsafe fn Kraid_Instr_DecYpos(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    return jp;
}
pub unsafe fn Kraid_Instr_IncrYpos_Shake(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 10 as uint16;
    return jp;
}
pub unsafe fn Kraid_Instr_PlaySound_0x76(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x76 as uint16, game);
    return jp;
}
pub unsafe fn Kraid_Instr_XposMinus3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos =
        ((*E).base.x_pos as ::core::ffi::c_int - g_word_A7A91C as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn Kraid_Instr_XposMinus3b(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos =
        ((*E).base.x_pos as ::core::ffi::c_int - g_word_A7A91C as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn Kraid_Instr_XposPlus3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos =
        ((*E).base.x_pos as ::core::ffi::c_int + g_word_A7A920 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn Kraid_Instr_MoveHimRight(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v3: uint16 = 0;
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    if (*E).base.x_pos as ::core::ffi::c_int - 320 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        || {
            v3 = ((*E).kraid_target_x as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
            (*E).kraid_target_x = v3;
            (*E).kraid_target_x as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        }
    {
        if Enemy_MoveRight_IgnoreSlopes(
            0 as uint16,
            (g_word_A7A922 as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 7 as uint16;
            let mut x_pos: uint16 = (*Get_Kraid(0 as ::core::ffi::c_int, game)).base.x_pos;
            (*Get_Kraid(0x140 as ::core::ffi::c_int, game)).base.x_pos = x_pos;
        }
    }
    return jp;
}
pub unsafe fn Kraid_InitEyeGlowing(game: &mut Game) {
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    (*E).kraid_var_A = (0xa7aee4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).kraid_var_A = (0xa7b6d7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).kraid_var_B = (addr_stru_A7974A as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
    (*E).kraid_var_C = (*(game.g_rom.offset(
        (((0xa7974a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa7974a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut KraidInstrList)
        .offset(0))
    .timer;
    Kraid_Shot_GlowHisEye(game);
}
pub unsafe fn Kraid_Shot_GlowHisEye(game: &mut Game) {
    let mut v1: int16 = 0;
    Kraid_ProcessKraidInstr(game);
    let mut v0: uint16 = 226 as uint16;
    v1 = 0 as int16;
    loop {
        let mut v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v3: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
            + 1 as ::core::ffi::c_int) as uint16;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(v2 as isize)
            as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
            - 30 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v1 += 1;
            v3 = 31 as uint16;
        }
        let mut v4: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            & 0x3e0 as ::core::ffi::c_int)
            + 32 as ::core::ffi::c_int) as uint16;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(v2 as isize)
            as ::core::ffi::c_int
            & 0x3e0 as ::core::ffi::c_int)
            - 960 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v1 += 1;
            v4 = 992 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(v2 as isize) =
            (v4 as ::core::ffi::c_int
                | v3 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int
                    & 0xfc00 as ::core::ffi::c_int) as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int - 232 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    if (v1 as ::core::ffi::c_int - 6 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_var_A =
            (0xa7b73d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Kraid_Shot_UnglowEye(game: &mut Game) {
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    let mut v0: uint16 = 14 as uint16;
    let mut health: uint16 = (*E).base.health;
    while !((health as ::core::ffi::c_int
        - (*Get_Kraid(v0 as ::core::ffi::c_int, game)).kraid_healths_8ths
            [0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int)
    {
        v0 = (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if !(v0 != 0) {
            break;
        }
    }
    let mut v2: uint16 =
        (16 as ::core::ffi::c_int * (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int)) as uint16;
    let mut v3: uint16 = 226 as uint16;
    let mut r20: uint16 = 0 as uint16;
    let mut r18: uint16 = 0;
    loop {
        let mut v4: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        r18 = (*((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset(v4 as isize) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int) as uint16;
        let mut v5: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *(game.g_rom.offset(
            (((0xa7b3d3 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7b3d3 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v5 + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int
            != r18 as ::core::ffi::c_int
        {
            r20 = r20.wrapping_add(1);
            let ref mut fresh9 = *((&raw mut game.g_ram as *mut uint8).offset(0xc000)
                as *mut uint16)
                .offset(v4 as isize);
            *fresh9 = (*fresh9).wrapping_sub(1);
        }
        r18 = (*((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset(v4 as isize) as ::core::ffi::c_int
            & 0x3e0 as ::core::ffi::c_int) as uint16;
        if *(game.g_rom.offset(
            (((0xa7b3d3 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7b3d3 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v5 + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            & 0x3e0 as ::core::ffi::c_int
            != r18 as ::core::ffi::c_int
        {
            r20 = r20.wrapping_add(1);
            let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8).offset(0xc000)
                as *mut uint16)
                .offset(v4 as isize);
            *fresh10 = (*fresh10 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
        }
        v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v3 as ::core::ffi::c_int - 232 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    if r20 == 0 {
        (*E).kraid_var_A =
            (0xa7aee4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).kraid_var_B = addr_stru_A796DA as ::core::ffi::c_int as uint16;
        (*E).kraid_var_C = (*(game.g_rom.offset(
            (((0xa796d2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa796d2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut KraidInstrList))
            .timer;
    }
}
pub unsafe fn KraidsArm_Main(game: &mut Game) {
    let mut r18: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
        as ::core::ffi::c_int
        + 224 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 =
        ((*E).base.y_pos as ::core::ffi::c_int - 44 as ::core::ffi::c_int) as uint16;
    let mut E1: *mut Enemy_Kraid = Get_Kraid(0x40 as ::core::ffi::c_int, game);
    (*E1).base.y_pos = v1;
    let mut v3: uint16 = v1;
    let mut v4: uint16 =
        ((*E1).base.properties as ::core::ffi::c_int | 0x100 as ::core::ffi::c_int) as uint16;
    if (v3 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
        && ((v3 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
    {
        v4 = ((*E1).base.properties as ::core::ffi::c_int & 0xfeff as ::core::ffi::c_int) as uint16;
    }
    (*E1).base.properties = v4;
    (*E1).base.x_pos = (*E).base.x_pos;
    if *(&raw mut (*E).kraid_mouth_flags as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) != 0
    {
        (*E1).base.instruction_timer = (*E1).base.instruction_timer.wrapping_add(1);
    }
}
pub unsafe fn KraidLintCommon_Main(mut k: uint16, game: &mut Game) {
    Kraid_EnemyTouch_Lint(k, game);
    let mut v1: *mut EnemyData = gEnemyData(k as ::core::ffi::c_int, game);
    CallKraidFunc(
        ((*v1).ai_var_A as ::core::ffi::c_int | 0xa70000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn KraidsTopLint_Main(game: &mut Game) {
    (*gEnemyData(0x80 as ::core::ffi::c_int, game)).instruction_timer = 0x7fff as uint16;
    KraidLintCommon_Main(0x80 as uint16, game);
}
pub unsafe fn KraidsMiddleLint_Main(game: &mut Game) {
    (*gEnemyData(0xc0 as ::core::ffi::c_int, game)).instruction_timer = 0x7fff as uint16;
    KraidLintCommon_Main(0xc0 as uint16, game);
}
pub unsafe fn KraidsBottomLint_Main(game: &mut Game) {
    (*gEnemyData(0x100 as ::core::ffi::c_int, game)).instruction_timer = 0x7fff as uint16;
    KraidLintCommon_Main(0x100 as uint16, game);
}
pub unsafe fn KraidLint_ProduceLint(mut k: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(k as ::core::ffi::c_int, game);
    (*v1).properties =
        ((*v1).properties as ::core::ffi::c_int & 0xfaff as ::core::ffi::c_int) as uint16;
    (*v1).x_pos = ((*v1).ai_var_C as ::core::ffi::c_int
        + (*gEnemyData(0 as ::core::ffi::c_int, game)).x_pos as ::core::ffi::c_int
        - (*v1).ai_var_B as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = ((*v1).ai_var_B as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    (*v1).ai_var_B = v2;
    if v2 as ::core::ffi::c_int - 32 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        (*v1).ai_var_A = (0xa7b868 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*v1).ai_preinstr = 30 as uint16;
    }
}
pub unsafe fn KraidLint_ChargeLint(mut k: uint16, game: &mut Game) {
    let mut E: *mut EnemyData = gEnemyData(k as ::core::ffi::c_int, game);
    let mut v1: int16 = 0 as int16;
    if (*E).ai_preinstr as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        v1 = 3584 as int16;
    }
    (*E).palette_index = v1 as uint16;
    (*E).x_pos = ((*E).ai_var_C as ::core::ffi::c_int
        + (*gEnemyData(0 as ::core::ffi::c_int, game)).x_pos as ::core::ffi::c_int
        - (*E).ai_var_B as ::core::ffi::c_int) as uint16;
    let fresh5 = (*E).ai_preinstr;
    (*E).ai_preinstr = (*E).ai_preinstr.wrapping_sub(1);
    if fresh5 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).ai_var_A = (0xa7b89b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        QueueSfx3_Max6(0x1f as uint16, game);
    }
}
pub unsafe fn KraidLint_FireLint(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Kraid = Get_Kraid(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        -(((g_word_A7A928 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | g_word_A7A926 as ::core::ffi::c_int) as int32) as uint32,
    );
    if (*E).base.x_pos as ::core::ffi::c_int - 56 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
    }
    if (*E).base.x_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
        (*E).kraid_var_A =
            (0xa7b923 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).kraid_var_F = 300 as uint16;
        (*E).kraid_next = (0xa7b832 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).kraid_var_B = 0 as uint16;
    }
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16,
            -(((g_word_A7A928 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | g_word_A7A926 as ::core::ffi::c_int) as int32) as uint32,
        );
        if *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) =
                -(16 as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn KraidFingernail_WaitForLintXpos(mut k: uint16, game: &mut Game) {
    let mut E2: *mut Enemy_Kraid = Get_Kraid(0x80 as ::core::ffi::c_int, game);
    if (*E2).base.x_pos as ::core::ffi::c_int - 256 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        let mut E: *mut Enemy_Kraid = Get_Kraid(k as ::core::ffi::c_int, game);
        (*E).kraid_var_A = (*E).kraid_next;
        (*E).base.properties =
            ((*E).base.properties as ::core::ffi::c_int & 0xfaff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn KraidEnemy_HandleFunctionTimer(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Kraid = Get_Kraid(k as ::core::ffi::c_int, game);
    if (*E).kraid_var_F as ::core::ffi::c_int != 0 && {
        let fresh6 = (*E).kraid_var_F;
        (*E).kraid_var_F = (*E).kraid_var_F.wrapping_sub(1);
        fresh6 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
    } {
        (*E).kraid_var_A = (*E).kraid_next;
    }
}
pub unsafe fn Kraid_AlignEnemyToKraid(mut k: uint16, game: &mut Game) {
    let mut x_pos: uint16 = (*Get_Kraid(0 as ::core::ffi::c_int, game)).base.x_pos;
    let mut E: *mut Enemy_Kraid = Get_Kraid(k as ::core::ffi::c_int, game);
    (*E).base.x_pos =
        (x_pos as ::core::ffi::c_int - (*E).base.x_width as ::core::ffi::c_int) as uint16;
    KraidEnemy_HandleFunctionTimer(k, game);
}
pub unsafe fn KraidEnemy_DecrementEnemyFunctionTimer(game: &mut Game) {
    let mut E: *mut Enemy_Kraid = Get_Kraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).kraid_var_F != 0 {
        let fresh8 = (*E).kraid_var_F;
        (*E).kraid_var_F = (*E).kraid_var_F.wrapping_sub(1);
        if fresh8 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            (*E).kraid_var_A = (*E).kraid_next;
            (*E).base.current_instruction = addr_kKraid_Ilist_8887 as ::core::ffi::c_int as uint16;
            (*E).base.instruction_timer = 1 as uint16;
        }
    }
}
pub unsafe fn KraidFoot_FirstPhase_Thinking(mut k: uint16, game: &mut Game) {
    Kraid_HandleFirstPhase(game);
    KraidEnemy_HandleFunctionTimer(k, game);
}
pub unsafe fn KraidEnemy_ProcessInstrEnemyTimer(mut k: uint16, game: &mut Game) {
    Kraid_ProcessKraidInstr(game);
    KraidEnemy_HandleFunctionTimer(k, game);
}
pub unsafe fn Kraid_EnemyTouch_Lint(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Kraid = Get_Kraid(k as ::core::ffi::c_int, game);
    if (*E).base.properties as ::core::ffi::c_int & kEnemyProps_Tangible as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) == 0
    {
        let mut r18: uint16 = ((*(game.g_rom.offset(
            (((0xa792b7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa792b7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut Hitbox))
            .left as ::core::ffi::c_int
            + (*E).base.x_pos as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16)
                    as ::core::ffi::c_int
                - r18 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                let mut r22: uint16 = ((*(game.g_rom.offset(
                    (((0xa792b7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa792b7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut Hitbox))
                    .top as ::core::ffi::c_int
                    + (*E).base.y_pos as ::core::ffi::c_int
                    + 2 as ::core::ffi::c_int) as uint16;
                if *((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16)
                    as ::core::ffi::c_int
                    + *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                        as ::core::ffi::c_int
                    - r22 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    let mut r24: uint16 = ((*(game.g_rom.offset(
                        (((0xa792b7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa792b7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut Hitbox))
                        .bottom as ::core::ffi::c_int
                        + (*E).base.y_pos as ::core::ffi::c_int
                        - 2 as ::core::ffi::c_int)
                        as uint16;
                    if *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                        as ::core::ffi::c_int
                        - *((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16)
                            as ::core::ffi::c_int
                        - r24 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0
                    {
                        let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xb58)
                            as *mut uint16)
                            as ::core::ffi::c_int
                            + !(*((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16)
                                as ::core::ffi::c_int
                                + 16 as ::core::ffi::c_int))
                            as uint16;
                        if v2 as ::core::ffi::c_int - 16 as ::core::ffi::c_int
                            & 0x8000 as ::core::ffi::c_int
                            == 0
                        {
                            v2 = 16 as uint16;
                        }
                        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = v2;
                        NormalEnemyTouchAi(game);
                        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                            | kEnemyProps_Tangible as ::core::ffi::c_int)
                            as uint16;
                    }
                }
            }
        }
    }
}
pub unsafe fn KraidsFoot_Main(game: &mut Game) {
    let mut v3: int16 = 0;
    let mut v0: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
    (*v0.offset(5)).x_pos = (*v0).x_pos;
    let mut v1: uint16 = ((*v0).y_pos as ::core::ffi::c_int + 100 as ::core::ffi::c_int) as uint16;
    (*v0.offset(5)).y_pos = v1;
    let mut v2: uint16 = v1;
    v3 = (v1 as ::core::ffi::c_int - 224 as ::core::ffi::c_int) as int16;
    let mut v4: uint16 =
        ((*v0.offset(5)).properties as ::core::ffi::c_int & 0xfeff as ::core::ffi::c_int) as uint16;
    if (v2 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        if (v3 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            v4 = (v4 as ::core::ffi::c_int | 0x100 as ::core::ffi::c_int) as uint16;
        }
    } else {
        v4 = (v4 as ::core::ffi::c_int | 0x100 as ::core::ffi::c_int) as uint16;
    }
    (*v0.offset(5)).properties = v4;
    CallKraidFunc(
        ((*v0.offset(5)).ai_var_A as ::core::ffi::c_int | 0xa70000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn Kraid_SetWalkingBackwards(mut j: uint16, mut a: uint16, game: &mut Game) {
    (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_target_x = a;
    let mut E: *mut Enemy_Kraid = Get_Kraid(0x140 as ::core::ffi::c_int, game);
    (*E).kraid_next = j;
    (*E).kraid_var_A = (0xa7bb45 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.current_instruction = addr_kKraid_Ilist_8887 as ::core::ffi::c_int as uint16;
}
pub unsafe fn Kraid_SetWalkingForwards(mut j: uint16, mut a: uint16, game: &mut Game) {
    (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_target_x = a;
    let mut E: *mut Enemy_Kraid = Get_Kraid(0x140 as ::core::ffi::c_int, game);
    (*E).kraid_next = j;
    (*E).kraid_var_A = (0xa7bbae as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.current_instruction = addr_kKraid_Ilist_86F3 as ::core::ffi::c_int as uint16;
}
pub unsafe fn KraidsFoot_SecondPhase_Thinking(game: &mut Game) {
    let mut v4: int16 = 0;
    let mut E5: *mut Enemy_Kraid = Get_Kraid(0x140 as ::core::ffi::c_int, game);
    let mut v1: uint16 =
        ((*E5).kraid_next as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E5).kraid_next = v1;
    if v1 == 0 {
        let mut v2: uint16 = 0 as uint16;
        let mut E0: *mut Enemy_Kraid = ::core::ptr::null_mut::<Enemy_Kraid>();
        loop {
            E0 = Get_Kraid(0 as ::core::ffi::c_int, game);
            if (*E0).base.x_pos as ::core::ffi::c_int
                == *(game.g_rom.offset(
                    (((0xa7ba7d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa7ba7d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
            {
                break;
            }
            v2 = (v2 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
            if !((v2 as ::core::ffi::c_int - 24 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int)
            {
                continue;
            }
            v2 = 4 as uint16;
            break;
        }
        v4 = (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 0x1c as ::core::ffi::c_int) as int16;
        if v4 as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v4 = 16 as int16;
        }
        let mut v5: *const uint16 = RomPtr_A7(
            (*(game.g_rom.offset(
                (((0xa7ba7d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa7ba7d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    ((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 1 as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int
                + v4 as ::core::ffi::c_int) as uint16_t,
            game,
        ) as *const uint16;
        let mut v6: uint16 = *v5.offset(1);
        if (*v5 as ::core::ffi::c_int - (*E0).base.x_pos as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            Kraid_SetWalkingBackwards(v6, *v5, game);
        } else {
            Kraid_SetWalkingForwards(v6, *v5, game);
        }
    }
}
pub unsafe fn KraidsFoot_SecondPhase_WalkingBackwards(game: &mut Game) {
    let mut E0: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    let mut kraid_target_x: uint16 = (*E0).kraid_target_x;
    if kraid_target_x as ::core::ffi::c_int != (*E0).base.x_pos as ::core::ffi::c_int {
        if (kraid_target_x as ::core::ffi::c_int - (*E0).base.x_pos as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            return;
        }
        (*E0).base.x_pos = kraid_target_x;
    }
    let mut E5: *mut Enemy_Kraid = Get_Kraid(0x140 as ::core::ffi::c_int, game);
    if (*E5).base.current_instruction as ::core::ffi::c_int + 0x76c7 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E5).kraid_var_A =
            (0xa7ba2e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E5).base.instruction_timer = 1 as uint16;
        (*E5).base.current_instruction = addr_kKraid_Ilist_86ED as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn KraidsFoot_SecondPhaseSetup_WalkToStartPt(game: &mut Game) {
    let mut E0: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    let mut kraid_target_x: uint16 = (*E0).kraid_target_x;
    if kraid_target_x as ::core::ffi::c_int != (*E0).base.x_pos as ::core::ffi::c_int {
        if (kraid_target_x as ::core::ffi::c_int - (*E0).base.x_pos as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            return;
        }
        (*E0).base.x_pos = kraid_target_x;
    }
    let mut E5: *mut Enemy_Kraid = Get_Kraid(0x140 as ::core::ffi::c_int, game);
    if (*E5).base.current_instruction as ::core::ffi::c_int + 0x76c7 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E5).kraid_var_A =
            (0xa7b92d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E5).kraid_var_F = 180 as uint16;
        (*E5).kraid_next =
            (0xa7bba4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E5).base.instruction_timer = 1 as uint16;
        (*E5).base.current_instruction = addr_kKraid_Ilist_86ED as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn KraidsFoot_SecondPhase_Init(game: &mut Game) {
    Kraid_SetWalkingBackwards(0xb4 as uint16, 0x160 as uint16, game);
}
pub unsafe fn KraidsFoot_SecondPhase_WalkForward(game: &mut Game) {
    let mut E0: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    let mut kraid_target_x: uint16 = (*E0).kraid_target_x;
    if ((kraid_target_x as ::core::ffi::c_int - (*E0).base.x_pos as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        let mut E5: *mut Enemy_Kraid = Get_Kraid(0x140 as ::core::ffi::c_int, game);
        if (*E5).base.current_instruction as ::core::ffi::c_int
            == addr_off_A787BB as ::core::ffi::c_int as uint16 as ::core::ffi::c_int
        {
            (*E5).base.current_instruction = addr_kKraid_Ilist_86F3 as ::core::ffi::c_int as uint16;
            (*E5).base.instruction_timer = 1 as uint16;
        }
    } else {
        (*E0).base.x_pos = kraid_target_x;
        let mut E5_0: *mut Enemy_Kraid = Get_Kraid(0x140 as ::core::ffi::c_int, game);
        if (*E5_0).base.current_instruction as ::core::ffi::c_int
            == addr_off_A787BB as ::core::ffi::c_int as uint16 as ::core::ffi::c_int
        {
            (*E5_0).kraid_var_A =
                (0xa7ba2e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E5_0).base.instruction_timer = 1 as uint16;
            (*E5_0).base.current_instruction =
                addr_kKraid_Ilist_86ED as ::core::ffi::c_int as uint16;
        }
    };
}
pub unsafe fn Kraid_Main_AttackWithMouthOpen(game: &mut Game) {
    let mut v3: uint16 = 0;
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    if Kraid_ProcessKraidInstr(game) as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
        Kraid_Mainloop_Thinking_Setup(game);
        (*E).kraid_var_C = 90 as uint16;
        let mut kraid_mouth_flags: uint16 = (*E).kraid_mouth_flags;
        if kraid_mouth_flags as ::core::ffi::c_int & 4 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            && {
                v3 =
                    (kraid_mouth_flags as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as uint16;
                (*E).kraid_mouth_flags = v3;
                v3 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
            }
        {
            (*E).kraid_var_A =
                (0xa7b92d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).kraid_var_F = 64 as uint16;
            (*E).kraid_next =
                (0xa7b6bf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).field_2 = 2 as uint16;
        } else {
            (*E).kraid_mouth_flags = 0 as uint16;
        }
    } else if *(RomPtr_A7(
        ((*E).kraid_var_B as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16_t,
        game,
    ) as *mut uint16)
        .offset(1) as ::core::ffi::c_int
        == addr_kKraidTilemaps_3 as ::core::ffi::c_int
        && (*E).kraid_var_C as ::core::ffi::c_int & 0xf as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        SpawnEprojWithGfx(
            *(game.g_rom.offset(
                (((0xa7bc65 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa7bc65 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                        as ::core::ffi::c_int
                        & 0xe as ::core::ffi::c_int)
                        >> 1 as ::core::ffi::c_int) as isize,
                ),
            (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                as ::core::ffi::c_int
                & 0xe as ::core::ffi::c_int) as uint16,
            addr_kEproj_RocksKraidSpits as ::core::ffi::c_int as uint16,
            game,
        );
        QueueSfx3_Max6(0x1e as uint16, game);
    }
}
pub unsafe fn KraidsGoodFingernail_Touch(game: &mut Game) {
    let mut v0: uint16 = 0 as uint16;
    printf(b"X undefined\n\0" as *const u8 as *const ::core::ffi::c_char);
    NormalEnemyTouchAi(game);
    EnemyDeathAnimation(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        v0,
        game,
    );
}
pub unsafe fn KraidsBadFingernail_Touch(game: &mut Game) {
    let mut v0: uint16 = 0 as uint16;
    printf(b"X undefined\n\0" as *const u8 as *const ::core::ffi::c_char);
    NormalEnemyTouchAi(game);
    EnemyDeathAnimation(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        v0,
        game,
    );
}
pub unsafe fn KraidFingernailInit(mut k: uint16, game: &mut Game) {
    let mut palette_index: uint16 = (*Get_Kraid(0 as ::core::ffi::c_int, game))
        .base
        .palette_index;
    let mut E: *mut Enemy_Kraid = Get_Kraid(k as ::core::ffi::c_int, game);
    (*E).base.palette_index = palette_index;
    (*E).kraid_var_B = 40 as uint16;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
    (*E).base.instruction_timer = 0x7fff as uint16;
    (*E).base.current_instruction = addr_kKraid_Ilist_8B0A as ::core::ffi::c_int as uint16;
    (*E).base.spritemap_pointer = (*(game.g_rom.offset(
        (((0xa78b0a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa78b0a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SpriteDrawInstr))
        .field_2 as uint16;
    (*E).kraid_next = (0xa7bd60 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).kraid_var_A = (0xa7b92d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).kraid_var_F = 64 as uint16;
}
pub unsafe fn KraidsGoodFingernail_Init(game: &mut Game) {
    KraidFingernailInit(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn KraidsBadFingernail_Init(game: &mut Game) {
    KraidFingernailInit(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn KraidsGoodFingernail_Main(game: &mut Game) {
    let mut v0: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
    if (*v0).health as ::core::ffi::c_int - 1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v1: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
        let fresh12 = &raw mut (*v1.offset(6)).properties;
        fresh12.write_unaligned(
            (fresh12.read_unaligned() as ::core::ffi::c_int
                | (kEnemyProps_Deleted as ::core::ffi::c_int
                    | kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16,
        );
    } else {
        CallKraidFunc(
            ((*v0.offset(6)).ai_var_A as ::core::ffi::c_int | 0xa70000 as ::core::ffi::c_int)
                as uint32,
            game,
        );
    };
}
pub unsafe fn KraidsBadFingernail_Main(game: &mut Game) {
    let mut v0: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
    if (*v0).health as ::core::ffi::c_int - 1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v1: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
        let fresh4 = &raw mut (*v1.offset(7)).properties;
        fresh4.write_unaligned(
            (fresh4.read_unaligned() as ::core::ffi::c_int
                | (kEnemyProps_Deleted as ::core::ffi::c_int
                    | kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16,
        );
    } else {
        CallKraidFunc(
            ((*v0.offset(7)).ai_var_A as ::core::ffi::c_int | 0xa70000 as ::core::ffi::c_int)
                as uint32,
            game,
        );
    };
}
pub unsafe fn KraidsFingernail_Init(game: &mut Game) {
    let mut v7: uint16 = 0;
    let mut current_block: u64;
    let mut v2: uint16 = 0;
    let mut kraid_var_E: uint16 = (*Get_Kraid(0x180 as ::core::ffi::c_int, game)).kraid_var_E;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
        == 384 as ::core::ffi::c_int
    {
        kraid_var_E = (*Get_Kraid(0x1c0 as ::core::ffi::c_int, game)).kraid_var_E;
    }
    if kraid_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v2 = *(game.g_rom.offset(
            (((0xa7be3e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7be3e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                    as ::core::ffi::c_int
                    & 6 as ::core::ffi::c_int)
                    >> 1 as ::core::ffi::c_int) as isize,
            );
    } else {
        v2 = *(game.g_rom.offset(
            (((0xa7be46 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7be46 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                    as ::core::ffi::c_int
                    & 6 as ::core::ffi::c_int)
                    >> 1 as ::core::ffi::c_int) as isize,
            );
    }
    let mut v3: *const uint8 = RomPtr_A7(v2 as uint16_t, game);
    let mut E: *mut Enemy_Kraid = Get_Kraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).kraid_var_B = *(v3 as *mut uint16);
    (*E).kraid_var_C = *(v3.offset(2) as *mut uint16);
    (*E).kraid_var_D = *(v3.offset(4) as *mut uint16);
    (*E).kraid_var_E = *(v3.offset(6) as *mut uint16);
    (*E).kraid_parameter_1 = 1 as uint16;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Tangible as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.current_instruction = addr_kKraid_Ilist_8B0A as ::core::ffi::c_int as uint16;
    (*E).kraid_var_A = (0xa7be8e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        current_block = 1333174206292935201;
    } else {
        v7 = 0;
        v7 = (*Get_Kraid(0x180 as ::core::ffi::c_int, game)).kraid_healths_8ths
            [1 as ::core::ffi::c_int as usize];
        if *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            != 448 as ::core::ffi::c_int
        {
            v7 = (*Get_Kraid(0x1c0 as ::core::ffi::c_int, game)).kraid_healths_8ths
                [1 as ::core::ffi::c_int as usize];
        }
        if v7 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            current_block = 1333174206292935201;
        } else {
            (*E).kraid_healths_8ths[1 as ::core::ffi::c_int as usize] = 1 as uint16;
            (*E).base.x_pos = 50 as uint16;
            (*E).base.y_pos = 240 as uint16;
            (*E).kraid_var_B = 0 as uint16;
            (*E).kraid_var_C = 1 as uint16;
            (*E).kraid_var_D = 0 as uint16;
            (*E).kraid_var_E = 0 as uint16;
            (*E).kraid_var_A =
                (0xa7b907 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).kraid_next =
                (0xa7be8e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                | (kEnemyProps_Tangible as ::core::ffi::c_int
                    | kEnemyProps_Invisible as ::core::ffi::c_int))
                as uint16;
            current_block = 16203760046146113240;
        }
    }
    match current_block {
        1333174206292935201 => {
            (*E).kraid_healths_8ths[1 as ::core::ffi::c_int as usize] = 0 as uint16;
            let mut E0: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
            (*E).base.x_pos = ((*E0).base.x_pos as ::core::ffi::c_int
                - (*E0).base.x_width as ::core::ffi::c_int
                - (*E).base.x_width as ::core::ffi::c_int
                & 0xfff0 as ::core::ffi::c_int) as uint16;
            (*E).base.y_pos = ((*Get_Kraid(0x40 as ::core::ffi::c_int, game)).base.y_pos
                as ::core::ffi::c_int
                + 128 as ::core::ffi::c_int) as uint16;
        }
        _ => {}
    };
}
pub unsafe fn KraidsFingernail_Fire(mut k: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*v1).ai_var_C as uint32) << 16 as ::core::ffi::c_int | (*v1).ai_var_B as uint32)
            as int32,
        game,
    ) != 0
    {
        (*v1).ai_var_B = -((*v1).ai_var_B as ::core::ffi::c_int) as uint16;
        (*v1).ai_var_C = -((*v1).ai_var_C as ::core::ffi::c_int) as uint16;
    } else {
        let mut i: ::core::ffi::c_int = 0;
        let mut v3: *mut EnemyData = ::core::ptr::null_mut::<EnemyData>();
        let mut v4: *mut EnemyData = ::core::ptr::null_mut::<EnemyData>();
        i = 0 as ::core::ffi::c_int;
        loop {
            v3 = gEnemyData(0 as ::core::ffi::c_int, game);
            v4 = gEnemyData(k as ::core::ffi::c_int, game);
            if ((*(game.g_rom.offset(
                (((0xa7bf1d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa7bf1d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                + (*v3).y_pos as ::core::ffi::c_int
                - (*v4).y_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                break;
            }
            i += 4 as ::core::ffi::c_int;
        }
        let mut r18: uint16 = (*(game.g_rom.offset(
            (((0xa7bf1d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7bf1d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            + (*v3).x_pos as ::core::ffi::c_int) as uint16;
        if (*v4).x_width as ::core::ffi::c_int + (*v4).x_pos as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
            && (*v4).ai_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            (*v4).ai_var_B = -((*v4).ai_var_B as ::core::ffi::c_int) as uint16;
            (*v4).ai_var_C = -((*v4).ai_var_C as ::core::ffi::c_int) as uint16;
        }
    }
    let mut v5: *mut EnemyData = gEnemyData(k as ::core::ffi::c_int, game);
    if Enemy_MoveDown(
        k,
        (((*v5).ai_var_E as uint32) << 16 as ::core::ffi::c_int | (*v5).ai_var_D as uint32)
            as int32,
        game,
    ) != 0
    {
        SetHiLo(
            &raw mut (*v5).ai_var_E,
            &raw mut (*v5).ai_var_D,
            -((((*v5).ai_var_E as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*v5).ai_var_D as ::core::ffi::c_int) as int32) as uint32,
        );
    }
}
pub unsafe fn KraidsFoot_PrepareToLungeForward(game: &mut Game) {
    Kraid_HandleFirstPhase(game);
    let mut v0: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
    if (*v0.offset(1)).current_instruction as ::core::ffi::c_int + 0x75c9 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*v0.offset(1)).current_instruction =
            addr_kKraid_Ilist_8AF0 as ::core::ffi::c_int as uint16;
        (*v0.offset(1)).instruction_timer = 1 as uint16;
        (*v0.offset(5)).instruction_timer = 1 as uint16;
        (*v0.offset(5)).current_instruction =
            addr_kKraid_Ilist_87BD as ::core::ffi::c_int as uint16;
        (*v0.offset(5)).ai_var_A =
            (0xa7bf5d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*v0.offset(5)).ai_preinstr = 0 as uint16;
    }
}
pub unsafe fn KraidsFoot_FirstPhase_LungeForward(game: &mut Game) {
    Kraid_HandleFirstPhase(game);
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    if (*E).base.x_pos as ::core::ffi::c_int - 92 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).base.x_pos = 92 as uint16;
    }
    let mut E5: *mut Enemy_Kraid = Get_Kraid(0x140 as ::core::ffi::c_int, game);
    if (*E5).base.current_instruction as ::core::ffi::c_int
        == addr_off_A78885 as ::core::ffi::c_int as uint16 as ::core::ffi::c_int
    {
        if (*E).base.x_pos as ::core::ffi::c_int == 92 as ::core::ffi::c_int {
            (*E5).kraid_next =
                (0xa7bfab as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E5).kraid_var_A =
                (0xa7b93f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E5).kraid_var_F = 1 as uint16;
            (*E5).base.instruction_timer = 1 as uint16;
            (*E5).base.current_instruction = addr_kKraid_Ilist_86ED as ::core::ffi::c_int as uint16;
        } else {
            (*E5).base.instruction_timer = 1 as uint16;
            (*E5).base.current_instruction = addr_kKraid_Ilist_87BD as ::core::ffi::c_int as uint16;
        }
    }
}
pub unsafe fn KraidsFoot_FirstPhase_RetreatFromLunge(game: &mut Game) {
    Kraid_HandleFirstPhase(game);
    let mut E0: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    if (*E0).base.x_pos as ::core::ffi::c_int - 176 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E0).base.x_pos = 176 as uint16;
    }
    let mut E5: *mut Enemy_Kraid = Get_Kraid(0x140 as ::core::ffi::c_int, game);
    if (*E5).base.current_instruction as ::core::ffi::c_int + 0x76c7 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        if (*E0).base.x_pos as ::core::ffi::c_int == 176 as ::core::ffi::c_int {
            let mut E1: *mut Enemy_Kraid = Get_Kraid(0x40 as ::core::ffi::c_int, game);
            (*E1).base.current_instruction = addr_kKraid_Ilist_89F3 as ::core::ffi::c_int as uint16;
            (*E1).base.instruction_timer = 1 as uint16;
            (*E5).base.instruction_timer = 1 as uint16;
            (*E5).base.current_instruction = addr_kKraid_Ilist_86ED as ::core::ffi::c_int as uint16;
            (*E5).kraid_var_A =
                (0xa7b960 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E5).kraid_var_F = 300 as uint16;
            (*E5).kraid_next =
                (0xa7bf2d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            printf(b"X undefined\n\0" as *const u8 as *const ::core::ffi::c_char);
            let mut E: *mut Enemy_Kraid = Get_Kraid(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            );
            (*E).base.current_instruction = addr_kKraid_Ilist_8887 as ::core::ffi::c_int as uint16;
            (*E).base.instruction_timer = 1 as uint16;
        }
    }
}
pub unsafe fn Kraid_HandleFirstPhase(game: &mut Game) {
    let mut E0: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    if (((*E0).base.health as ::core::ffi::c_int
        - (*E0).kraid_healths_8ths[6 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        (*E0).kraid_var_A =
            (0xa7b965 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E0).kraid_var_F = 180 as uint16;
        (*E0).kraid_next =
            (0xa7c0a1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut v2: uint16 =
            *(RomPtr_A7((*E0).kraid_var_B as uint16_t, game) as *mut uint16).offset(1);
        let mut v3: uint16 = 50 as uint16;
        if v2 as ::core::ffi::c_int != addr_kKraidTilemaps_0 as ::core::ffi::c_int {
            v3 = 42 as uint16;
            if v2 as ::core::ffi::c_int != addr_kKraidTilemaps_1 as ::core::ffi::c_int {
                v3 = 34 as uint16;
                if v2 as ::core::ffi::c_int != addr_kKraidTilemaps_2 as ::core::ffi::c_int {
                    v3 = 26 as uint16;
                }
            }
        }
        (*E0).kraid_var_B = (v3 as ::core::ffi::c_int - 26918 as ::core::ffi::c_int) as uint16;
        (*E0).kraid_var_C = *((&raw mut (*(game.g_rom.offset(
            (((0xa796d2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa796d2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut KraidInstrList))
            .timer as *mut uint8)
            .offset(v3 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 4 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 340 as uint16;
        let mut E5: *mut Enemy_Kraid = Get_Kraid(0x140 as ::core::ffi::c_int, game);
        (*E5).base.current_instruction = addr_kKraid_Ilist_86E7 as ::core::ffi::c_int as uint16;
        (*E5).base.instruction_timer = 1 as uint16;
        (*E5).kraid_var_A =
            (0xa7ba2d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut E1: *mut Enemy_Kraid = Get_Kraid(0x40 as ::core::ffi::c_int, game);
        (*E1).base.current_instruction = addr_kKraid_Ilist_89F3 as ::core::ffi::c_int as uint16;
        (*E1).base.instruction_timer = 1 as uint16;
        let mut E2: *mut Enemy_Kraid = Get_Kraid(0x80 as ::core::ffi::c_int, game);
        (*E2).base.properties =
            ((*E2).base.properties as ::core::ffi::c_int | 0x100 as ::core::ffi::c_int) as uint16;
        let mut E3: *mut Enemy_Kraid = Get_Kraid(0xc0 as ::core::ffi::c_int, game);
        (*E3).base.properties =
            ((*E3).base.properties as ::core::ffi::c_int | 0x100 as ::core::ffi::c_int) as uint16;
        let mut E4: *mut Enemy_Kraid = Get_Kraid(0x100 as ::core::ffi::c_int, game);
        (*E4).base.properties =
            ((*E4).base.properties as ::core::ffi::c_int | 0x100 as ::core::ffi::c_int) as uint16;
        (*E1).base.properties =
            ((*E1).base.properties as ::core::ffi::c_int | 0x400 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Kraid_GetsBig_ReleaseCamera(game: &mut Game) {
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    (*E).kraid_var_A = (0xa7ac4d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd20) as *mut uint16) = 514 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) as *mut uint8 as *mut uint16) =
        257 as uint16;
    (*E).kraid_min_y_pos_eject = 164 as uint16;
}
pub unsafe fn Kraid_SpawnPlmToClearCeiling(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x2 as uint8,
            field_1: 0x12 as uint8,
            field_2: 0xb7b7 as VoidP,
        },
        game,
    );
}
pub unsafe fn Kraid_ClearSomeSpikes(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x5 as uint8,
            field_1: 0x1b as uint8,
            field_2: 0xb7bb as VoidP,
        },
        game,
    );
}
pub unsafe fn UnpauseHook_Kraid_IsDead(game: &mut Game) -> CoroutineRet {
    static unk_A7C21E: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0xa7a716 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0xa7a716 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x200 as VoidP,
    };
    static unk_A7C23E: StartDmaCopy = StartDmaCopy {
        chan: 1 as uint8,
        dmap: 1 as uint8,
        bbad: 0x18 as uint8,
        a1: LongPtr {
            addr: (0x9ab200 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
            bank: (0x9ab200 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
        },
        das: 0x800 as VoidP,
    };
    ScreenOff(game);
    WriteReg(VMADDL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(
        VMADDH as ::core::ffi::c_int as uint16,
        (16 as ::core::ffi::c_int
            * (*(&raw mut game.g_ram as *mut uint8).offset(0x5d) as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)
            + 63 as ::core::ffi::c_int) as uint8,
        game,
    );
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    SetupDmaTransfer(&raw const unk_A7C21E as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
    WriteReg(VMADDL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(VMADDH as ::core::ffi::c_int as uint16, 0x40 as uint8, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    SetupDmaTransfer(&raw const unk_A7C23E as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
    Kraid_TransferTopHalfToVram(game);
    return kCoroutineNone as CoroutineRet;
}
static unk_A7C26B: StartDmaCopy = StartDmaCopy {
    chan: 1 as uint8,
    dmap: 1 as uint8,
    bbad: 0x18 as uint8,
    a1: LongPtr {
        addr: (0x7e5000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
        bank: (0x7e5000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
    },
    das: 0x400 as VoidP,
};
static unk_A7C28D: StartDmaCopy = StartDmaCopy {
    chan: 1 as uint8,
    dmap: 1 as uint8,
    bbad: 0x18 as uint8,
    a1: LongPtr {
        addr: (0x7e2000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
        bank: (0x7e2000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
    },
    das: 0x800 as VoidP,
};
pub unsafe fn UnpauseHook_Kraid_IsAlive(game: &mut Game) -> CoroutineRet {
    ScreenOff(game);
    WriteReg(VMADDL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(
        VMADDH as ::core::ffi::c_int as uint16,
        (*(&raw mut game.g_ram as *mut uint8).offset(0x5d) as ::core::ffi::c_int
            + 62 as ::core::ffi::c_int) as uint8,
        game,
    );
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    SetupDmaTransfer(&raw const unk_A7C26B as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
    Kraid_TransferTopHalfToVram(game);
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn Kraid_TransferTopHalfToVram(game: &mut Game) {
    WriteReg(VMADDL as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(
        VMADDH as ::core::ffi::c_int as uint16,
        (*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int) as uint8,
        game,
    );
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    SetupDmaTransfer(&raw const unk_A7C28D as *const ::core::ffi::c_void, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
    ScreenOn(game);
}
static unk_A7C2BD: StartDmaCopy = StartDmaCopy {
    chan: 1 as uint8,
    dmap: 1 as uint8,
    bbad: 0x18 as uint8,
    a1: LongPtr {
        addr: (0x7e5000 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP,
        bank: (0x7e5000 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) as uint8,
    },
    das: 0x400 as VoidP,
};
pub unsafe fn Kraid_UnpauseHook_IsSinking(game: &mut Game) -> CoroutineRet {
    let mut _state_variable_: *mut uint8 = (&raw mut game.g_ram as *mut uint8).offset(0x613);
    's_100: {
        let mut v2: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
        let mut current_block_21: u64;
        match *_state_variable_ as ::core::ffi::c_int {
            0 => {
                ScreenOff(game);
                WriteReg(VMADDL as ::core::ffi::c_int as uint16, 0 as uint8, game);
                WriteReg(
                    VMADDH as ::core::ffi::c_int as uint16,
                    (*(&raw mut game.g_ram as *mut uint8).offset(0x5d) as ::core::ffi::c_int
                        + 62 as ::core::ffi::c_int) as uint8,
                    game,
                );
                WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
                SetupDmaTransfer(&raw const unk_A7C2BD as *const ::core::ffi::c_void, game);
                WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
                if ((*Get_Kraid(0 as ::core::ffi::c_int, game)).base.y_pos as ::core::ffi::c_int
                    - (*(game.g_rom.offset(
                        (((0xa7c5e7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa7c5e7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut KraidSinkTable)
                        .offset(0))
                    .field_0 as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int
                {
                    *((&raw mut game.g_ram as *mut uint8).offset(0x77e) as *mut uint16) =
                        0 as uint16;
                    *((&raw mut game.g_ram as *mut uint8).offset(0x780) as *mut uint16) =
                        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
                    current_block_21 = 7351195479953500246;
                } else {
                    current_block_21 = 12124785117276362961;
                }
            }
            1 => {
                current_block_21 = 16936033848137449375;
            }
            _ => {
                current_block_21 = 12124785117276362961;
            }
        }
        loop {
            match current_block_21 {
                12124785117276362961 => {
                    break 's_100;
                }
                7351195479953500246 => {
                    if !((*(game.g_rom.offset(
                        (((0xa7c5e7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa7c5e7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut KraidSinkTable)
                        .offset(
                            *((&raw mut game.g_ram as *mut uint8).offset(0x77e) as *mut uint16)
                                as isize,
                        ))
                    .field_0 as ::core::ffi::c_int
                        != 0xffff as ::core::ffi::c_int
                        && ((*Get_Kraid(0 as ::core::ffi::c_int, game)).base.y_pos
                            as ::core::ffi::c_int
                            - (*(game.g_rom.offset(
                                (((0xa7c5e7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                    << 15 as ::core::ffi::c_int
                                    | 0xa7c5e7 as ::core::ffi::c_int
                                        & 0x7fff as ::core::ffi::c_int)
                                    & 0x3fffff as ::core::ffi::c_int)
                                    as isize,
                            ) as *const uint8
                                as *mut KraidSinkTable)
                                .offset(
                                    *((&raw mut game.g_ram as *mut uint8).offset(0x77e)
                                        as *mut uint16)
                                        as isize,
                                ))
                            .field_0 as ::core::ffi::c_int) as int16
                            as ::core::ffi::c_int
                            >= 0 as ::core::ffi::c_int)
                    {
                        current_block_21 = 12124785117276362961;
                        continue;
                    }
                    v2 = ::core::ptr::null_mut::<VramWriteEntry>();
                    v2 = gVramWriteEntry(
                        *((&raw mut game.g_ram as *mut uint8).offset(0x780) as *mut uint16)
                            as ::core::ffi::c_int,
                        game,
                    );
                    (*v2).size = 64 as uint16;
                    (*v2).src.addr = 0x2fc0 as VoidP;
                    (*v2).src.bank = 0x7e as uint8;
                    (*v2).vram_dst = ((*(game.g_rom.offset(
                        (((0xa7c5e7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa7c5e7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8
                        as *mut KraidSinkTable)
                        .offset(
                            *((&raw mut game.g_ram as *mut uint8).offset(0x77e) as *mut uint16)
                                as isize,
                        ))
                    .field_2 as ::core::ffi::c_int
                        + ((*(&raw mut game.g_ram as *mut uint8).offset(0x59)
                            as ::core::ffi::c_int
                            & 0xfc as ::core::ffi::c_int)
                            << 8 as ::core::ffi::c_int))
                        as uint16;
                    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
                        (*((&raw mut game.g_ram as *mut uint8).offset(0x780) as *mut uint16)
                            as ::core::ffi::c_int
                            + 7 as ::core::ffi::c_int) as uint16;
                    current_block_21 = 16936033848137449375;
                }
                _ => {
                    let mut _coret_: uint8 = WaitForNMI_Async(game) as uint8;
                    if _coret_ != 0 {
                        *_state_variable_ = 1 as uint8;
                        return _coret_ as CoroutineRet;
                    }
                    let ref mut fresh17 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0x77e) as *mut uint16);
                    *fresh17 = (*fresh17).wrapping_add(1);
                    current_block_21 = 7351195479953500246;
                }
            }
        }
    }
    *_state_variable_ = 0 as uint8;
    return 0 as CoroutineRet;
}
pub unsafe fn PauseHook_Kraid(game: &mut Game) {
    let mut v1: ::core::ffi::c_uint = 0;
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x360) as *mut uint16);
    v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x360) as *mut uint16) as ::core::ffi::c_uint;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .vram_target as *mut uint8)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x360) as *mut uint16)
                as ::core::ffi::c_int as isize,
        ) as *mut uint16) = (((*(&raw mut game.g_ram as *mut uint8).offset(0x5d)
        as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int)
        + 15872 as ::core::ffi::c_int) as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .dma_parameters as *mut uint8)
        .offset(v1 as isize) as *mut uint16) = 129 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .dma_parameters as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize)
        .offset(1) as *mut uint16) = 57 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .src
    .addr as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut VoidP) = 20480 as VoidP;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .src
    .bank)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 126 as uint16;
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x340) as *mut VramReadQueueEnt)
        .offset(0))
    .size as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = 1024 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x360) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 9 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Kraid_Death_Init(game: &mut Game) {
    let mut E0: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    if (*E0).kraid_hurt_frame == 0 {
        let mut v0: uint16 = 192 as uint16;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                0 as uint16;
            v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v0 as ::core::ffi::c_int - 224 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 112 as ::core::ffi::c_int) as isize) =
                *(game.g_rom.offset(
                    (((0xa7b4f3 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa7b4f3 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((i >> 1 as ::core::ffi::c_int) as isize);
            i -= 2 as ::core::ffi::c_int;
        }
        let mut E1: *mut Enemy_Kraid = Get_Kraid(0x40 as ::core::ffi::c_int, game);
        (*E1).base.current_instruction = addr_kKraid_Ilist_8AF0 as ::core::ffi::c_int as uint16;
        (*E1).base.instruction_timer = 1 as uint16;
        (*E0).kraid_var_A =
            (0xa7c3f9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E0).kraid_var_B =
            (addr_stru_A79764 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
        (*E0).kraid_var_C = (*(game.g_rom.offset(
            (((0xa79764 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa79764 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut KraidInstrList)
            .offset(0))
        .timer;
        let mut v4: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
        let mut v7: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
        let mut E6: *mut Enemy_Kraid = Get_Kraid(0x180 as ::core::ffi::c_int, game);
        (*E6).base.properties = ((*E6).base.properties as ::core::ffi::c_int
            & !(0x4000 as ::core::ffi::c_int)) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) = 384 as uint16;
        EnemyDeathAnimation(v4, 0x180 as uint16, game);
        let mut E7: *mut Enemy_Kraid = Get_Kraid(0x1c0 as ::core::ffi::c_int, game);
        (*E7).base.properties = ((*E7).base.properties as ::core::ffi::c_int
            & !(0x4000 as ::core::ffi::c_int)) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) = 448 as uint16;
        EnemyDeathAnimation(v4, 0x1c0 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) = 128 as uint16;
        EnemyDeathAnimation(v4, 0x80 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) = 192 as uint16;
        EnemyDeathAnimation(v4, 0xc0 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) = 256 as uint16;
        EnemyDeathAnimation(v4, 0x100 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) = v7;
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x5 as uint8,
                field_1: 0x1b as uint8,
                field_2: 0xb7bf as VoidP,
            },
            game,
        );
    }
}
pub unsafe fn Kraid_Death_Fadeout(game: &mut Game) {
    let mut v3: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    Kraid_ProcessKraidInstr(game);
    if AdvancePaletteFade_BgPalette6(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
        (*E).kraid_var_A =
            (0xa7c4a4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).kraid_hurt_frame_timer = 1 as uint16;
        (*E).kraid_hurt_frame = 1 as uint16;
        Kraid_Palette_Handling(game);
        let mut v1: uint16 = 0 as uint16;
        loop {
            (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
                [v1 as usize] = 0 as uint16;
            (*((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000)).xray_tilemaps
                [(v1 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize] = 0 as uint16;
            v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v1 as ::core::ffi::c_int * 2 as ::core::ffi::c_int - 512 as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
        v3 = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*v3).size = 512 as uint16;
        (*v3).src.addr = 0x4000 as VoidP;
        (*v3).src.bank = 126 as uint8;
        (*v3).vram_dst = (((*(&raw mut game.g_ram as *mut uint8).offset(0x5d)
            as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int)
            + 16128 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            (v2 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Kraid_Death_UpdateBG2TilemapTopHalf(game: &mut Game) {
    Kraid_ProcessKraidInstr(game);
    (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_var_A =
        (0xa7c4c8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut E2: *mut Enemy_Kraid = Get_Kraid(0x80 as ::core::ffi::c_int, game);
    (*E2).kraid_var_A = (0xa7b923 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E2).kraid_var_F = 0x7fff as uint16;
    let mut E3: *mut Enemy_Kraid = Get_Kraid(0xc0 as ::core::ffi::c_int, game);
    (*E3).kraid_var_A = (0xa7b923 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E3).kraid_var_F = 0x7fff as uint16;
    let mut E4: *mut Enemy_Kraid = Get_Kraid(0x100 as ::core::ffi::c_int, game);
    (*E4).kraid_var_A = (0xa7b923 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E4).kraid_var_F = 0x7fff as uint16;
    Kraid_UpdateBg2TilemapTopHalf(game);
}
pub unsafe fn Kraid_Death_UpdateBG2TilemapBottomHalf(game: &mut Game) {
    Kraid_ProcessKraidInstr(game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0x604) as *mut LongPtr)).bank =
        -(89 as ::core::ffi::c_int) as uint8;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x604) as *mut LongPtr)).addr =
        (0xa7c2a0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP;
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    (*E).kraid_var_A = (0xa7c537 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9000) as *mut uint16) = 43 as uint16;
    (*E).base.properties =
        ((*E).base.properties as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 256 as uint16;
    let mut E1: *mut Enemy_Kraid = Get_Kraid(0x40 as ::core::ffi::c_int, game);
    (*E1).base.current_instruction = addr_kKraid_Ilist_8AA4 as ::core::ffi::c_int as uint16;
    (*E1).base.instruction_timer = 1 as uint16;
    let mut E5: *mut Enemy_Kraid = Get_Kraid(0x140 as ::core::ffi::c_int, game);
    (*E5).base.current_instruction = addr_kKraid_Ilist_86E7 as ::core::ffi::c_int as uint16;
    (*E5).base.instruction_timer = 1 as uint16;
    (*E5).kraid_var_A = (0xa7ba2d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Kraid_UpdateBG2TilemapBottomHalf(game);
}
pub unsafe fn Kraid_PlaySoundEveryHalfSecond(game: &mut Game) {
    let ref mut fresh7 = *((&raw mut game.g_ram as *mut uint8).offset(0x9000) as *mut uint16);
    *fresh7 = (*fresh7).wrapping_sub(1);
    if *fresh7 == 0 {
        QueueSfx3_Max6(0x1e as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x9000) as *mut uint16) = 30 as uint16;
    }
}
pub unsafe fn Kraid_HandleSinking(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v3: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        v1 = (*(game.g_rom.offset(
            (((0xa7c5e7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7c5e7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut KraidSinkTable)
            .offset(i as isize))
        .field_0 as int16;
        if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            break;
        }
        if v1 as ::core::ffi::c_int
            == (*Get_Kraid(0 as ::core::ffi::c_int, game)).base.y_pos as ::core::ffi::c_int
        {
            if (*(game.g_rom.offset(
                (((0xa7c5e7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa7c5e7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut KraidSinkTable)
                .offset(i as isize))
            .field_2 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                let mut v2: uint16 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
                v3 = gVramWriteEntry(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                        as ::core::ffi::c_int,
                    game,
                );
                (*v3).size = 64 as uint16;
                (*v3).src.addr = 12224 as VoidP;
                (*v3).src.bank = 126 as uint8;
                (*v3).vram_dst = ((*(game.g_rom.offset(
                    (((0xa7c5e7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa7c5e7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut KraidSinkTable)
                    .offset(i as isize))
                .field_2 as ::core::ffi::c_int
                    + ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
                        & 0xfc as ::core::ffi::c_int)
                        << 8 as ::core::ffi::c_int)) as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
                    (v2 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
            }
            CallKraidSinkTableFunc(
                ((*(game.g_rom.offset(
                    (((0xa7c5e7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa7c5e7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut KraidSinkTable)
                    .offset(i as isize))
                .field_4 as ::core::ffi::c_int
                    | 0xa70000 as ::core::ffi::c_int) as uint32,
                game,
            );
            return;
        }
        i += 1;
    }
}
pub unsafe fn Kraid_Death_SinkThroughFloor(game: &mut Game) {
    Kraid_ProcessKraidInstr(game);
    Kraid_PlaySoundEveryHalfSecond(game);
    Kraid_HandleSinking(game);
    let mut E0: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    (*E0).base.y_pos = (*E0).base.y_pos.wrapping_add(1);
    if (*E0).base.y_pos as ::core::ffi::c_int - 608 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E0).base.properties = ((*E0).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_Tangible as ::core::ffi::c_int))
            as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x179a) as *mut uint16) = 2 as uint16;
        let mut enemy_ptr: uint16 = (*Get_Kraid(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .base
        .enemy_ptr;
        (*get_EnemyDef_A2(enemy_ptr, game)).shot_ai =
            (0xa7804c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP;
        let mut E1: *mut Enemy_Kraid = Get_Kraid(0x40 as ::core::ffi::c_int, game);
        let mut v3: uint16 = ((*E1).base.properties as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int
            | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
        (*E1).base.properties = v3;
        let mut v4: uint16 = (v3 as ::core::ffi::c_int & 0x51ff as ::core::ffi::c_int
            | 0x600 as ::core::ffi::c_int) as uint16;
        (*Get_Kraid(0x80 as ::core::ffi::c_int, game))
            .base
            .properties = v4;
        (*Get_Kraid(0xc0 as ::core::ffi::c_int, game))
            .base
            .properties = v4;
        (*Get_Kraid(0x100 as ::core::ffi::c_int, game))
            .base
            .properties = v4;
        (*Get_Kraid(0x140 as ::core::ffi::c_int, game))
            .base
            .properties = v4;
        (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_var_A =
            (0xa7c715 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x941) as *mut uint16) = 0 as uint16;
        Enemy_ItemDrop_Kraid(enemy_ptr, game);
        Kraid_DrawRoomBg(game);
    }
}
pub unsafe fn Kraid_CrumbleLeftPlatform_Left(game: &mut Game) {
    SpawnEprojWithGfx(
        0x70 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kEproj_RocksFallingKraidCeiling as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x7 as uint8,
            field_1: 0x12 as uint8,
            field_2: 0xb7a7 as VoidP,
        },
        game,
    );
}
pub unsafe fn Kraid_CrumbleRightPlatform_Middle(game: &mut Game) {
    SpawnEprojWithGfx(
        0xf0 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kEproj_RocksFallingKraidCeiling as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0xf as uint8,
            field_1: 0x12 as uint8,
            field_2: 0xb7a7 as VoidP,
        },
        game,
    );
}
pub unsafe fn Kraid_CrumbleRightPlatform_Left(game: &mut Game) {
    SpawnEprojWithGfx(
        0xe0 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kEproj_RocksFallingKraidCeiling as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0xe as uint8,
            field_1: 0x12 as uint8,
            field_2: 0xb7af as VoidP,
        },
        game,
    );
}
pub unsafe fn Kraid_CrumbleLeftPlatform_Right(game: &mut Game) {
    SpawnEprojWithGfx(
        0x90 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kEproj_RocksFallingKraidCeiling as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x9 as uint8,
            field_1: 0x12 as uint8,
            field_2: 0xb7a7 as VoidP,
        },
        game,
    );
}
pub unsafe fn Kraid_CrumbleLeftPlatform_Middle(game: &mut Game) {
    SpawnEprojWithGfx(
        0x80 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kEproj_RocksFallingKraidCeiling as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x8 as uint8,
            field_1: 0x12 as uint8,
            field_2: 0xb7af as VoidP,
        },
        game,
    );
}
pub unsafe fn Kraid_CrumbleRightPlatform_Right(game: &mut Game) {
    SpawnEprojWithGfx(
        0x100 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kEproj_RocksFallingKraidCeiling as ::core::ffi::c_int as uint16,
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x10 as uint8,
            field_1: 0x12 as uint8,
            field_2: 0xb7af as VoidP,
        },
        game,
    );
}
pub unsafe fn CallKraidSinkTableFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        10995345 => {
            Kraid_CrumbleLeftPlatform_Left(game);
            return;
        }
        10995366 => return,
        10995367 => {
            Kraid_CrumbleRightPlatform_Middle(game);
            return;
        }
        10995389 => {
            Kraid_CrumbleRightPlatform_Left(game);
            return;
        }
        10995411 => {
            Kraid_CrumbleLeftPlatform_Right(game);
            return;
        }
        10995433 => {
            Kraid_CrumbleLeftPlatform_Middle(game);
            return;
        }
        10995455 => {
            Kraid_CrumbleRightPlatform_Right(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn Kraid_FadeInBg_ClearBg2TilemapTopHalf(game: &mut Game) {
    let mut v2: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    *(&raw mut game.g_ram as *mut uint8).offset(0x59) = 72 as uint8;
    let mut i: ::core::ffi::c_int = 2046 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    v2 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v2).size = 1024 as uint16;
    (*v2).src.addr = 0x2000 as VoidP;
    *(&raw mut (*v2).src.bank as *mut uint16) = 126 as uint16;
    (*v2).vram_dst = 18432 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v1 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_var_A =
        (0xa7c751 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Kraid_FadeInBg_ClearBg2TilemapBottomHalf(game: &mut Game) {
    let mut v1: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    v1 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).size = 1024 as uint16;
    (*v1).src.addr = 0x2000 as VoidP;
    *(&raw mut (*v1).src.bank as *mut uint16) = 126 as uint16;
    (*v1).vram_dst = 18944 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_var_A =
        (0xa7c777 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Kraid_FadeInBg_LoadBg3Tiles1of4(game: &mut Game) {
    let mut v1: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    (*((&raw mut game.g_ram as *mut uint8).offset(0x604) as *mut LongPtr)).addr =
        (0xa7c1fb as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as VoidP;
    (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_var_A =
        (0xa7c7a3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    v1 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).size = 1024 as uint16;
    (*v1).src.addr = -(19968 as ::core::ffi::c_int) as VoidP;
    *(&raw mut (*v1).src.bank as *mut uint16) = 154 as uint16;
    (*v1).vram_dst = 0x4000 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Kraid_FadeInBg_LoadBg3Tiles2of4(game: &mut Game) {
    let mut v1: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_var_A =
        (0xa7c7c9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    v1 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).size = 1024 as uint16;
    (*v1).src.addr = -(18944 as ::core::ffi::c_int) as VoidP;
    *(&raw mut (*v1).src.bank as *mut uint16) = 154 as uint16;
    (*v1).vram_dst = 16896 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Kraid_FadeInBg_LoadBg3Tiles3of4(game: &mut Game) {
    let mut v1: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_var_A =
        (0xa7c7ef as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    v1 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).size = 1024 as uint16;
    (*v1).src.addr = -(17920 as ::core::ffi::c_int) as VoidP;
    *(&raw mut (*v1).src.bank as *mut uint16) = 154 as uint16;
    (*v1).vram_dst = 17408 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Kraid_FadeInBg_LoadBg3Tiles4of4(game: &mut Game) {
    let mut v1: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_var_A =
        (0xa7c815 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    v1 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).size = 1024 as uint16;
    (*v1).src.addr = -(16896 as ::core::ffi::c_int) as VoidP;
    *(&raw mut (*v1).src.bank as *mut uint16) = 154 as uint16;
    (*v1).vram_dst = 17920 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Kraid_FadeInBg_FadeInBp6(game: &mut Game) {
    let mut v0: int16 = 0;
    if AdvancePaletteFade_BgPalette6(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        QueueMusic_Delayed8(3 as uint16, game);
        v0 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd828)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
            as *mut uint8 as *mut uint16) as int16;
        if v0 as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_var_A =
                (0xa7c851 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd828).offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize,
            ) as *mut uint8 as *mut uint16) =
                (v0 as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint16;
            (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_var_A =
                (0xa7c843 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn Kraid_FadeInBg_SetEnemyDead_KraidWasAlive(game: &mut Game) {
    if Kraid_CheckIfDead(game) == 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) as ::core::ffi::c_int
            != 0
    {
        Kraid_SetEnemyPropsToDead(game);
    }
}
pub unsafe fn Kraid_FadeInBg_SetEnemyDead_KraidWasDead(game: &mut Game) {
    if Kraid_CheckIfDead(game) == 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) as ::core::ffi::c_int
            != 0
    {
        Kraid_SetEnemyPropsToDead(game);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x8ff) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Kraid_RestrictSamusXtoFirstScreen(mut k: uint16, game: &mut Game) {
    Kraid_RestrictSamusXtoFirstScreen_2(game);
    KraidEnemy_HandleFunctionTimer(k, game);
}
pub unsafe fn Kraid_UpdateBg2TilemapTopHalf(game: &mut Game) {
    let mut v1: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    v1 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).size = 2048 as uint16;
    (*v1).src.addr = 0x2000 as VoidP;
    (*v1).src.bank = 126 as uint8;
    (*v1).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Kraid_UpdateBG2TilemapBottomHalf(game: &mut Game) {
    let mut v1: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    v1 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).size = 2048 as uint16;
    (*v1).src.addr = 10240 as VoidP;
    (*v1).src.bank = 126 as uint8;
    (*v1).vram_dst = (((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int)
        + 2048 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Kraid_RaiseKraidThroughFloor(mut k: uint16, game: &mut Game) {
    Kraid_RestrictSamusXtoFirstScreen_2(game);
    (*Get_Kraid(0 as ::core::ffi::c_int, game)).kraid_var_A =
        (0xa7c89a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Kraid_UpdateBg2TilemapTopHalf(game);
}
pub unsafe fn Kraid_Raise_LoadTilemapBottomAndShake(game: &mut Game) {
    Kraid_RestrictSamusXtoFirstScreen_2(game);
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    (*E).kraid_var_A = (0xa7c8e0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).kraid_var_F = 120 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 496 as uint16;
    QueueMusic_Delayed8(5 as uint16, game);
    Kraid_UpdateBG2TilemapBottomHalf(game);
}
pub unsafe fn Kraid_Raise_SpawnRandomEarthquakeProjs16(game: &mut Game) {
    Kraid_RestrictSamusXtoFirstScreen_2(game);
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 =
        ((*E).kraid_var_F as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E).kraid_var_F = v1;
    if v1 != 0 {
        if v1 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            Kraid_SpawnRandomQuakeProjs(game);
        }
    } else {
        (*E).kraid_var_A =
            (0xa7c902 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).kraid_var_F = 96 as uint16;
    };
}
pub unsafe fn Kraid_Raise_SpawnRandomEarthquakeProjs8(game: &mut Game) {
    Kraid_RestrictSamusXtoFirstScreen_2(game);
    let mut E: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 =
        ((*E).kraid_var_F as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E).kraid_var_F = v1;
    if v1 != 0 {
        if v1 as ::core::ffi::c_int & 7 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            Kraid_SpawnRandomQuakeProjs(game);
        }
    } else {
        (*E).kraid_var_A =
            (0xa7c924 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).kraid_var_F = 288 as uint16;
    };
}
pub unsafe fn Kraid_Raise_Handler(game: &mut Game) {
    Kraid_RestrictSamusXtoFirstScreen_2(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) as ::core::ffi::c_int
        & 5 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        Kraid_SpawnRandomQuakeProjs(game);
    }
    let mut E0: *mut Enemy_Kraid = Get_Kraid(0 as ::core::ffi::c_int, game);
    (*E0).base.x_pos = ((*E0).base.x_pos as ::core::ffi::c_int
        + if (*E0).base.y_pos as ::core::ffi::c_int & 2 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            -(1 as ::core::ffi::c_int)
        } else {
            1 as ::core::ffi::c_int
        }) as uint16;
    AddToHiLo(
        &raw mut (*E0).base.y_pos,
        &raw mut (*E0).base.y_subpos,
        -(0x8000 as ::core::ffi::c_int) as uint32,
    );
    if (*E0).base.y_pos as ::core::ffi::c_int - 457 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E0).base.x_pos = 176 as uint16;
        let mut E5: *mut Enemy_Kraid = Get_Kraid(0x140 as ::core::ffi::c_int, game);
        (*E5).kraid_var_A =
            (0xa7b960 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E5).kraid_var_F = 300 as uint16;
        (*E5).kraid_next =
            (0xa7bf2d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E0).kraid_var_B = addr_stru_A796DA as ::core::ffi::c_int as uint16;
        Kraid_Mainloop_Thinking_Setup(game);
        let mut E1: *mut Enemy_Kraid = Get_Kraid(0x40 as ::core::ffi::c_int, game);
        (*E1).base.current_instruction = addr_kKraid_Ilist_89F3 as ::core::ffi::c_int as uint16;
        (*E1).base.instruction_timer = 1 as uint16;
    }
}
pub unsafe fn Kraid_SpawnRandomQuakeProjs(game: &mut Game) {
    let mut v0: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
        as ::core::ffi::c_int
        & 0x3f as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v0 = !(v0 as ::core::ffi::c_int) as uint16;
    }
    let mut x: uint16 = ((*Get_Kraid(0 as ::core::ffi::c_int, game)).base.x_pos
        as ::core::ffi::c_int
        + v0 as ::core::ffi::c_int) as uint16;
    let mut y: uint16 = (448 as ::core::ffi::c_int
        - ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 0x3f00 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int)) as uint16;
    CreateSpriteAtPos(x, y, 21 as uint16, 0 as uint16, game);
    let mut v1: uint16 = (if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
        as ::core::ffi::c_int
        & 0x10 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        addr_kEproj_RocksWhenKraidRisesRight as ::core::ffi::c_int
    } else {
        addr_kEproj_RocksWhenKraidRisesLeft as ::core::ffi::c_int
    }) as uint16;
    SpawnEprojWithGfx(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
            & 0x3f0 as ::core::ffi::c_int) as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        v1,
        game,
    );
}
pub unsafe fn Kraid_RestrictSamusXtoFirstScreen_2(game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        - 256 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) = 256 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) = 256 as uint16;
    }
}
pub unsafe fn CallKraidFunc(mut ea: uint32, game: &mut Game) {
    let mut k: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
    match ea {
        10988621 => {
            Kraid_GetsBig_BreakCeilingPlatforms(game);
            return;
        }
        10988858 => {
            Kraid_GetsBig_SetBG2TilemapPrioBits(game);
            return;
        }
        10988897 => {
            Kraid_GetsBig_FinishUpdateBg2Tilemap(game);
            return;
        }
        10988942 => {
            Kraid_GetsBig_DrawRoomBg(game);
            return;
        }
        10989091 => {
            Kraid_GetsBig_FadeInRoomBg(game);
            return;
        }
        10989220 => {
            Kraid_Mainloop_Thinking(game);
            return;
        }
        10989252 => {
            Kraid_GetsBig_Thinking(game);
            return;
        }
        10989284 => {
            Kraid_Shot_MouthIsOpen(game);
            return;
        }
        10991295 => {
            Kraid_InitEyeGlowing(game);
            return;
        }
        10991319 => {
            Kraid_Shot_GlowHisEye(game);
            return;
        }
        10991421 => {
            Kraid_Shot_UnglowEye(game);
            return;
        }
        10991666 => {
            KraidLint_ProduceLint(k, game);
            return;
        }
        10991771 => {
            KraidLint_FireLint(k, game);
            return;
        }
        10991879 => {
            KraidFingernail_WaitForLintXpos(k, game);
            return;
        }
        10991907 => {
            Kraid_AlignEnemyToKraid(k, game);
            return;
        }
        10991917 => {
            KraidEnemy_HandleFunctionTimer(k, game);
            return;
        }
        10991935 => {
            KraidEnemy_DecrementEnemyFunctionTimer(game);
            return;
        }
        10991968 => {
            KraidFoot_FirstPhase_Thinking(k, game);
            return;
        }
        10991973 => {
            KraidEnemy_ProcessInstrEnemyTimer(k, game);
            return;
        }
        10992174 => {
            KraidsFoot_SecondPhase_Thinking(game);
            return;
        }
        10992453 => {
            KraidsFoot_SecondPhase_WalkingBackwards(game);
            return;
        }
        10992494 => {
            KraidsFoot_SecondPhaseSetup_WalkToStartPt(game);
            return;
        }
        10992548 => {
            KraidsFoot_SecondPhase_Init(game);
            return;
        }
        10992558 => {
            KraidsFoot_SecondPhase_WalkForward(game);
            return;
        }
        10992618 => {
            Kraid_Main_AttackWithMouthOpen(game);
            return;
        }
        10992992 => {
            KraidsFingernail_Init(game);
            return;
        }
        10993294 => {
            KraidsFingernail_Fire(k, game);
            return;
        }
        10993453 => {
            KraidsFoot_PrepareToLungeForward(game);
            return;
        }
        10993579 => {
            KraidsFoot_FirstPhase_RetreatFromLunge(game);
            return;
        }
        10993825 => {
            Kraid_GetsBig_ReleaseCamera(game);
            return;
        }
        10994528 => {
            Kraid_Death_Init(game);
            return;
        }
        10994681 => {
            Kraid_Death_Fadeout(game);
            return;
        }
        10994852 => {
            Kraid_Death_UpdateBG2TilemapTopHalf(game);
            return;
        }
        10994888 => {
            Kraid_Death_UpdateBG2TilemapBottomHalf(game);
            return;
        }
        10994999 => {
            Kraid_Death_SinkThroughFloor(game);
            return;
        }
        10995477 => {
            Kraid_FadeInBg_ClearBg2TilemapTopHalf(game);
            return;
        }
        10995537 => {
            Kraid_FadeInBg_ClearBg2TilemapBottomHalf(game);
            return;
        }
        10995575 => {
            Kraid_FadeInBg_LoadBg3Tiles1of4(game);
            return;
        }
        10995619 => {
            Kraid_FadeInBg_LoadBg3Tiles2of4(game);
            return;
        }
        10995657 => {
            Kraid_FadeInBg_LoadBg3Tiles3of4(game);
            return;
        }
        10995695 => {
            Kraid_FadeInBg_LoadBg3Tiles4of4(game);
            return;
        }
        10995733 => {
            Kraid_FadeInBg_FadeInBp6(game);
            return;
        }
        10995779 => {
            Kraid_FadeInBg_SetEnemyDead_KraidWasAlive(game);
            return;
        }
        10995793 => {
            Kraid_FadeInBg_SetEnemyDead_KraidWasDead(game);
            return;
        }
        10995813 => {
            Kraid_RestrictSamusXtoFirstScreen(k, game);
            return;
        }
        10995819 => {
            Kraid_RaiseKraidThroughFloor(k, game);
            return;
        }
        10995866 => {
            Kraid_Raise_LoadTilemapBottomAndShake(game);
            return;
        }
        10995936 => {
            Kraid_Raise_SpawnRandomEarthquakeProjs16(game);
            return;
        }
        10995970 => {
            Kraid_Raise_SpawnRandomEarthquakeProjs8(game);
            return;
        }
        10996004 => {
            Kraid_Raise_Handler(game);
            return;
        }
        10991720 => {
            KraidLint_ChargeLint(k, game);
            return;
        }
        10992173 => return,
        10993501 => {
            KraidsFoot_FirstPhase_LungeForward(game);
            return;
        }
        10991665 => return,
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn Phantoon_Init(game: &mut Game) {
    let mut j: int16 = 0;
    let mut k: uint16 = 0;
    let mut i: ::core::ffi::c_int = 4094 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    j = 2046 as int16;
    while j as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9000) as *mut uint16 as *mut uint8)
            .offset(j as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        j = (j as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as int16;
    }
    k = 30 as uint16;
    while k as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(
            ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 112 as ::core::ffi::c_int)
                as isize,
        ) = 0 as uint16;
        k = (k as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x179a) as *mut uint16) = 864 as uint16;
    DisableMinimapAndMarkBossRoomAsExplored(game);
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).phant_var_E = 120 as uint16;
    (*E).phant_var_A = 0 as uint16;
    (*E).phant_var_B = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9032) as *mut uint16) = 0 as uint16;
    let mut E1: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    (*E1).phant_parameter_1 = 0 as uint16;
    (*E1).phant_parameter_2 = 0 as uint16;
    let mut E0: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
    (*E0).base.properties = ((*E0).base.properties as ::core::ffi::c_int
        | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
    static unk_A7CE51: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x1 as uint8,
        field_1: 0x14 as uint8,
        addr: 0xce96 as VoidP,
    };
    SpawnHdmaObject(
        0xa7 as uint8,
        &raw const unk_A7CE51 as *const ::core::ffi::c_void,
        game,
    );
    Phantoon2_Init(game);
}
pub unsafe fn Phantoon2_Init(game: &mut Game) {
    let mut v6: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v6).spritemap_pointer = addr_kSpritemap_Nothing_A7 as ::core::ffi::c_int as uint16;
    (*v6).instruction_timer = 1 as uint16;
    (*v6).timer = 0 as uint16;
    let mut v7: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
    (*v6).palette_index = (*v7).palette_index;
    (*v6).vram_tiles_index = (*v7).vram_tiles_index;
    (*v6).current_instruction = *(game.g_rom.offset(
        (((0xa7ce8e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa7ce8e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((*v6).parameter_2 as isize);
    (*v6).ai_preinstr = (0xa7d4a9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*v7.offset(3)).parameter_1 = 0 as uint16;
    (*v7.offset(3)).ai_var_C = -(1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Phantoon_Main(game: &mut Game) {
    Phantoon_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let mut EK: *mut Enemy_Phantoon = Get_Phantoon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallEnemyPreInstr(
        ((*EK).phant_var_F as ::core::ffi::c_int | 0xa70000 as ::core::ffi::c_int) as uint32,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut E0: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
        let mut E1: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
        let mut E2: *mut Enemy_Phantoon = Get_Phantoon(0x80 as ::core::ffi::c_int, game);
        let mut E3: *mut Enemy_Phantoon = Get_Phantoon(0xc0 as ::core::ffi::c_int, game);
        let mut x_pos: uint16 = (*E0).base.x_pos;
        (*E1).base.x_pos = x_pos;
        (*E2).base.x_pos = x_pos;
        (*E3).base.x_pos = x_pos;
        let mut y_pos: uint16 = (*E0).base.y_pos;
        (*E1).base.y_pos = y_pos;
        (*E2).base.y_pos = y_pos;
        (*E3).base.y_pos = y_pos;
        if (*E1).phant_parameter_1 == 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                    as ::core::ffi::c_int
                    - (*E0).base.x_pos as ::core::ffi::c_int
                    + 40 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                    as ::core::ffi::c_int
                    - (*E0).base.y_pos as ::core::ffi::c_int
                    + 40 as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn Phantoon_Func_1(game: &mut Game) {
    QueueSfx2_Max6(
        *(game.g_rom.offset(
            (((0xa7cded as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7cded as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x9032) as *mut uint16) as isize),
        game,
    );
    let mut v0: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x9032) as *mut uint16)
        as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9032) as *mut uint16) as ::core::ffi::c_int
        - 2 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        v0 = 0 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x9032) as *mut uint16) = v0;
}
pub unsafe fn Phantoon_Func_2(mut k: uint16, game: &mut Game) {
    if k == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
            & 0x4000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        *((&raw mut (*(game.g_rom.offset(
            (((0xa7902d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7902d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut ExtendedSpriteMap)
            .offset(0))
        .ypos as *mut uint8)
            .offset(1) as *mut uint16) =
            (*((&raw mut (*(game.g_rom.offset(
                (((0xa7902d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa7902d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut ExtendedSpriteMap)
                .offset(0))
            .ypos as *mut uint8)
                .offset(1) as *mut uint16) as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn Phantoon_Func_3(mut amt: int32, game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0xc0 as ::core::ffi::c_int, game);
    let mut r18: uint16 = amt as uint16;
    let mut r20: uint16 = (amt >> 16 as ::core::ffi::c_int) as uint16;
    if (*E).phant_var_E != 0 {
        let mut phant_var_D: uint16 = (*E).phant_var_D;
        let mut v4: bool = (phant_var_D as ::core::ffi::c_int) < r18 as ::core::ffi::c_int;
        (*E).phant_var_D =
            (phant_var_D as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
        if v4 {
            (*E).phant_var_D = 0 as uint16;
            return 1 as uint8;
        }
    } else {
        let mut r22: uint16 = ((r20 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint16;
        let mut v1: uint16 =
            (r18 as ::core::ffi::c_int + (*E).phant_var_D as ::core::ffi::c_int) as uint16;
        (*E).phant_var_D = v1;
        v1 = ((v1 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int)
            as uint16;
        if v1 as ::core::ffi::c_int - r22 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        {
            (*E).phant_var_D = r20;
        }
    }
    return 0 as uint8;
}
pub unsafe fn Phantoon_Func_4(mut k: uint16, game: &mut Game) {
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_kEproj_DestroyableFireballs as ::core::ffi::c_int as uint16,
        game,
    );
    QueueSfx3_Max6(0x1d as uint16, game);
}
pub unsafe fn Phantoon_Func_5(mut k: uint16, game: &mut Game) {
    let mut i: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        SpawnEprojWithGfx(
            (i | 0x600 as ::core::ffi::c_int) as uint16,
            k,
            addr_kEproj_DestroyableFireballs as ::core::ffi::c_int as uint16,
            game,
        );
        i -= 1;
    }
    QueueSfx3_Max6(0x28 as uint16, game);
}
pub unsafe fn Phantoon_Func_6(mut k: uint16, mut a: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v3: int16 = 0;
    v2 = *(game.g_rom.offset(
        (((0xa7cfc2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa7cfc2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset(a as isize) as int16;
    let mut n: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
    let mut r20: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
    loop {
        v3 = v2;
        let fresh25 = v2;
        v2 = v2 + 1;
        SpawnEprojWithGfx(
            (r20 | fresh25 as ::core::ffi::c_int | 0x400 as ::core::ffi::c_int) as uint16,
            k,
            addr_kEproj_DestroyableFireballs as ::core::ffi::c_int as uint16,
            game,
        );
        if (v3 as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            v2 = 0 as int16;
        }
        r20 += 16 as ::core::ffi::c_int;
        n -= 1;
        if !(n >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
pub unsafe fn Phantoon_Func_7(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Phantoon =
        Get_Phantoon(k as ::core::ffi::c_int + 192 as ::core::ffi::c_int, game);
    let mut v3: bool = (*E).phant_var_B as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).phant_var_B = (*E).phant_var_B.wrapping_sub(1);
    let mut v4: bool = (*E).phant_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v3 as ::core::ffi::c_int != 0 || v4 as ::core::ffi::c_int != 0 {
        if (*E).phant_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            v3 = (*E).phant_var_C as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
            (*E).phant_var_C = (*E).phant_var_C.wrapping_sub(1);
            let mut v8: bool = (*E).phant_var_C as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int;
            if v3 as ::core::ffi::c_int != 0 || v8 as ::core::ffi::c_int != 0 {
                (*E).phant_var_C = -(1 as ::core::ffi::c_int) as uint16;
                (*E).phant_var_B = *(RomPtr_A7(
                    *(game.g_rom.offset(
                        (((0xa7ccfd as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa7ccfd as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset((*E).phant_var_A as isize) as uint16_t,
                    game,
                ) as *mut uint16)
                    .offset(1);
            } else {
                (*E).phant_var_B = *(RomPtr_A7(
                    (2 as ::core::ffi::c_int * (*E).phant_var_C as ::core::ffi::c_int
                        + *(game.g_rom.offset(
                            (((0xa7ccfd as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                << 15 as ::core::ffi::c_int
                                | 0xa7ccfd as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                                & 0x3fffff as ::core::ffi::c_int)
                                as isize,
                        ) as *const uint8 as *mut uint16)
                            .offset((*E).phant_var_A as isize)
                            as ::core::ffi::c_int) as uint16_t,
                    game,
                ) as *mut uint16)
                    .offset(1);
            }
            (*E).base.instruction_timer = 1 as uint16;
            (*E).base.current_instruction = addr_kKraid_Ilist_CCEB as ::core::ffi::c_int as uint16;
        } else {
            let mut v5: uint16 =
                (NextRandom(game) as ::core::ffi::c_int & 3 as ::core::ffi::c_int) as uint16;
            (*E).phant_var_A = v5;
            let mut v6: uint16 = *(game.g_rom.offset(
                (((0xa7ccfd as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa7ccfd as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v5 as isize);
            let mut v7: uint16 = *(RomPtr_A7(v6 as uint16_t, game) as *mut uint16);
            (*E).phant_var_C = v7;
            (*E).phant_var_B = *(RomPtr_A7(
                (2 as ::core::ffi::c_int * v7 as ::core::ffi::c_int + v6 as ::core::ffi::c_int)
                    as uint16_t,
                game,
            ) as *mut uint16)
                .offset(1);
        }
    }
}
pub unsafe fn Phantoon_StartTrackingSamusAndInitEyeTimer(game: &mut Game) {
    (*Get_Phantoon(0x80 as ::core::ffi::c_int, game)).phant_var_A = 0 as uint16;
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.current_instruction = addr_kKraid_Ilist_CC4D as ::core::ffi::c_int as uint16;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
    (*E).phant_var_E = *(game.g_rom.offset(
        (((0xa7cd41 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa7cd41 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((NextRandom(game) as ::core::ffi::c_int & 7 as ::core::ffi::c_int) as isize);
    (*E).phant_var_F = (0xa7d60d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut E1: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    (*E1).base.instruction_timer = 1 as uint16;
    (*E1).base.current_instruction = addr_kKraid_Ilist_CC9D as ::core::ffi::c_int as uint16;
}
pub unsafe fn Phantoon_PickPatternForRound2(game: &mut Game) {
    let mut v4: int16 = 0;
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
    (*E).phant_var_E = 60 as uint16;
    let mut v1: uint16 = *(game.g_rom.offset(
        (((0xa7cd53 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa7cd53 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((NextRandom(game) as ::core::ffi::c_int & 7 as ::core::ffi::c_int) as isize);
    let mut E1: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    (*E1).phant_var_A = v1;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if (*E1).phant_var_C == 0 {
            v4 = ((*E).phant_var_A as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
            (*E).phant_var_A = v4 as uint16;
            if (v4 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                (*E).phant_var_A = 533 as uint16;
            }
        }
        (*E).phant_var_C = 0 as uint16;
        (*E).phant_var_B = 0 as uint16;
        (*E).phant_var_D = 0 as uint16;
        (*E1).phant_var_C = 1 as uint16;
    } else {
        if (*E1).phant_var_C != 0 {
            let mut v3: uint16 =
                ((*E).phant_var_A as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            (*E).phant_var_A = v3;
            if v3 as ::core::ffi::c_int - 534 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                (*E).phant_var_A = 0 as uint16;
            }
        }
        (*E).phant_var_C = 1 as uint16;
        (*E).phant_var_B = 0 as uint16;
        (*E).phant_var_D = 0 as uint16;
        (*E1).phant_var_C = 0 as uint16;
    }
    if (*E).phant_parameter_2 != 0 {
        (*E1).phant_var_F = 0 as uint16;
        (*E).phant_var_F =
            (0xa7d82a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).phant_var_F =
            (0xa7d5e7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Phantoon_AdjustSpeedAndMoveInFigure8(game: &mut Game) {
    if (*Get_Phantoon(0x40 as ::core::ffi::c_int, game)).phant_var_C != 0 {
        Phantoon_AdjustSpeedRightSideClockwise(game);
        Phantoon_MoveInFigure8_RightSideClockwise(
            addr_kPhantoonMoveData as ::core::ffi::c_int as uint16,
            533 as uint16,
            game,
        );
    } else {
        Phantoon_AdjustSpeedLeftSideClockwise(game);
        Phantoon_MoveInFigure8_LeftSideClockwise(
            addr_kPhantoonMoveData as ::core::ffi::c_int as uint16,
            534 as uint16,
            game,
        );
    };
}
pub unsafe fn Phantoon_AdjustSpeedLeftSideClockwise(game: &mut Game) {
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
    let mut phant_var_D: uint16 = (*E).phant_var_D;
    if phant_var_D != 0 {
        if phant_var_D as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            AddToHiLo(
                &raw mut (*E).phant_var_C,
                &raw mut (*E).phant_var_B,
                (g_word_A7CD79 as uint32) << 16 as ::core::ffi::c_int | g_word_A7CD77 as uint32,
            );
            if ((*E).phant_var_C as ::core::ffi::c_int - g_word_A7CD7D as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                (*E).phant_var_C = g_word_A7CD7D;
                (*E).phant_var_B = 0 as uint16;
                (*E).phant_var_D = (*E).phant_var_D.wrapping_add(1);
            }
        } else {
            AddToHiLo(
                &raw mut (*E).phant_var_C,
                &raw mut (*E).phant_var_B,
                -(((g_word_A7CD79 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                    | g_word_A7CD77 as ::core::ffi::c_int) as int32) as uint32,
            );
            if (*E).phant_var_C as ::core::ffi::c_int == g_word_A7CD7F as ::core::ffi::c_int
                || (((*E).phant_var_C as ::core::ffi::c_int - g_word_A7CD7F as ::core::ffi::c_int)
                    as int16 as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
            {
                (*E).phant_var_C =
                    (g_word_A7CD7F as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
                (*E).phant_var_B = 0 as uint16;
                (*E).phant_var_D = 0 as uint16;
            }
        }
    } else {
        AddToHiLo(
            &raw mut (*E).phant_var_C,
            &raw mut (*E).phant_var_B,
            (g_word_A7CD75 as uint32) << 16 as ::core::ffi::c_int | g_word_A7CD73 as uint32,
        );
        if ((*E).phant_var_C as ::core::ffi::c_int - g_word_A7CD7B as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            (*E).phant_var_C =
                (g_word_A7CD7B as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
            (*E).phant_var_B = 0 as uint16;
            (*E).phant_var_D = (*E).phant_var_D.wrapping_add(1);
        }
    };
}
pub unsafe fn Phantoon_AdjustSpeedRightSideClockwise(game: &mut Game) {
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
    let mut phant_var_D: uint16 = (*E).phant_var_D;
    if phant_var_D != 0 {
        if phant_var_D as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            AddToHiLo(
                &raw mut (*E).phant_var_C,
                &raw mut (*E).phant_var_B,
                -(((g_word_A7CD87 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                    | g_word_A7CD85 as ::core::ffi::c_int) as int32) as uint32,
            );
            if (*E).phant_var_C as ::core::ffi::c_int == g_word_A7CD8B as ::core::ffi::c_int
                || (((*E).phant_var_C as ::core::ffi::c_int - g_word_A7CD8B as ::core::ffi::c_int)
                    as int16 as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
            {
                (*E).phant_var_C =
                    (g_word_A7CD8B as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
                (*E).phant_var_B = 0 as uint16;
                (*E).phant_var_D = (*E).phant_var_D.wrapping_add(1);
            }
        } else {
            AddToHiLo(
                &raw mut (*E).phant_var_C,
                &raw mut (*E).phant_var_B,
                (g_word_A7CD87 as uint32) << 16 as ::core::ffi::c_int | g_word_A7CD85 as uint32,
            );
            if ((*E).phant_var_C as ::core::ffi::c_int - g_word_A7CD8D as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                (*E).phant_var_C = g_word_A7CD8D;
                (*E).phant_var_B = 0 as uint16;
                (*E).phant_var_D = 0 as uint16;
            }
        }
    } else {
        AddToHiLo(
            &raw mut (*E).phant_var_C,
            &raw mut (*E).phant_var_B,
            -(((g_word_A7CD83 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | g_word_A7CD81 as ::core::ffi::c_int) as int32) as uint32,
        );
        if (*E).phant_var_C as ::core::ffi::c_int == g_word_A7CD89 as ::core::ffi::c_int
            || (((*E).phant_var_C as ::core::ffi::c_int - g_word_A7CD89 as ::core::ffi::c_int)
                as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
        {
            (*E).phant_var_C =
                (g_word_A7CD89 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            (*E).phant_var_B = 0 as uint16;
            (*E).phant_var_D = (*E).phant_var_D.wrapping_add(1);
        }
    };
}
pub unsafe fn Phantoon_MoveInFigure8_LeftSideClockwise(
    mut j: uint16,
    mut r20: uint16,
    game: &mut Game,
) {
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
    let mut n: ::core::ffi::c_int = (*E).phant_var_C as ::core::ffi::c_int;
    while n != 0 {
        let mut v2: *const uint8 = RomPtr_A7(
            (2 as ::core::ffi::c_int * (*E).phant_var_A as ::core::ffi::c_int
                + j as ::core::ffi::c_int) as uint16_t,
            game,
        );
        (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
            + *v2.offset(0) as int8 as ::core::ffi::c_int) as uint16;
        (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
            + *v2.offset(1) as int8 as ::core::ffi::c_int) as uint16;
        (*E).phant_var_A = (*E).phant_var_A.wrapping_add(1);
        if (*E).phant_var_A as ::core::ffi::c_int - r20 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).phant_var_A = 0 as uint16;
        }
        n -= 1;
    }
}
pub unsafe fn Phantoon_MoveInFigure8_RightSideClockwise(
    mut j: uint16,
    mut r20: uint16,
    game: &mut Game,
) {
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
    let mut n: ::core::ffi::c_int =
        -((*E).phant_var_C as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int;
    while n != 0 {
        let mut v4: *const uint8 = RomPtr_A7(
            (2 as ::core::ffi::c_int * (*E).phant_var_A as ::core::ffi::c_int
                + j as ::core::ffi::c_int) as uint16_t,
            game,
        );
        (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
            - *v4.offset(0) as int8 as ::core::ffi::c_int) as uint16;
        (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
            - *v4.offset(1) as int8 as ::core::ffi::c_int) as uint16;
        (*E).phant_var_A = (*E).phant_var_A.wrapping_sub(1);
        if (*E).phant_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).phant_var_A = r20;
        }
        n -= 1;
    }
}
pub unsafe fn Phantoon_MoveInSwoopingPattern(mut k: uint16, game: &mut Game) {
    let mut phant_var_E: int16 = 0;
    let mut v4: uint16 = 0;
    let mut v5: uint16 = 0;
    let mut v6: uint16 = 0;
    let mut v13: int16 = 0;
    let mut E2: *mut Enemy_Phantoon = Get_Phantoon(0x80 as ::core::ffi::c_int, game);
    phant_var_E = (*E2).phant_var_E as int16;
    if (phant_var_E as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        v6 = (phant_var_E as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        (*E2).phant_var_E = v6;
        v4 = (v6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int) as uint16;
        if v4 == 0 {
            v4 = 0 as uint16;
            (*E2).phant_var_E = 0 as uint16;
        }
    } else {
        v4 = (phant_var_E as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        (*E2).phant_var_E = v4;
        if v4 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        {
            v5 = (v4 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
            (*E2).phant_var_E = v5;
            v4 = (v5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int) as uint16;
        }
    }
    let mut E0: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
    if ((v4 as ::core::ffi::c_int - (*E0).base.x_pos as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if (*E2).phant_var_C as ::core::ffi::c_int + 2047 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E2).phant_var_C =
                ((*E2).phant_var_C as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
        }
    } else if (*E2).phant_var_C as ::core::ffi::c_int - 2048 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E2).phant_var_C =
            ((*E2).phant_var_C as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    }
    AddToHiLo(
        &raw mut (*E0).base.x_pos,
        &raw mut (*E0).base.x_subpos,
        (((*E2).phant_var_C as int16 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint32,
    );
    if (*E0).base.x_pos as ::core::ffi::c_int + 64 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E0).base.x_pos = -(64 as ::core::ffi::c_int) as uint16;
    } else if (*E0).base.x_pos as ::core::ffi::c_int - 448 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E0).base.x_pos = 448 as uint16;
    }
    if (*Get_Phantoon(k as ::core::ffi::c_int, game)).phant_var_F as ::core::ffi::c_int
        == 0xa7d92e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        v13 = 112 as int16;
    } else {
        v13 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
            as ::core::ffi::c_int
            - 48 as ::core::ffi::c_int) as int16;
    }
    if ((v13 as ::core::ffi::c_int - (*E0).base.y_pos as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if (*E2).phant_var_D as ::core::ffi::c_int + 1535 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E2).phant_var_D =
                ((*E2).phant_var_D as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as uint16;
        }
    } else if (*E2).phant_var_D as ::core::ffi::c_int - 1536 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E2).phant_var_D =
            ((*E2).phant_var_D as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
    }
    AddToHiLo(
        &raw mut (*E0).base.y_pos,
        &raw mut (*E0).base.y_subpos,
        (((*E2).phant_var_D as int16 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint32,
    );
    if (*E0).base.y_pos as ::core::ffi::c_int - 64 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E0).base.y_pos = 64 as uint16;
    } else if (*E0).base.y_pos as ::core::ffi::c_int - 216 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E0).base.y_pos = 216 as uint16;
    }
}
pub unsafe fn Phantoon_BeginSwoopingPattern(mut k: uint16, game: &mut Game) {
    let mut E2: *mut Enemy_Phantoon = Get_Phantoon(0x80 as ::core::ffi::c_int, game);
    (*E2).phant_var_C = 1024 as uint16;
    (*E2).phant_var_D = 1024 as uint16;
    (*E2).phant_var_E = 0 as uint16;
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
    (*E).phant_var_F = (0xa7d678 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).phant_var_E = 360 as uint16;
}
pub unsafe fn Phantoon_ChangeEyeSpriteBasedOnSamusDist(game: &mut Game) {
    let mut v0: uint16 = (2 as ::core::ffi::c_int
        * DetermineDirectionOfSamusFromEnemy(game) as ::core::ffi::c_int)
        as uint16;
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.current_instruction = *(game.g_rom.offset(
        (((0xa7d40d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa7d40d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
}
pub unsafe fn Phantoon_StartDeathSequence(mut k: uint16, game: &mut Game) {
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
    if (*E).phant_var_F as ::core::ffi::c_int
        == 0xa7d678 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        || (*E).phant_var_F as ::core::ffi::c_int
            == 0xa7d6b9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        (*E).phant_var_F =
            (0xa7d92e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).phant_var_F =
            (0xa7d948 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    let mut E1: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    (*E1).phant_var_C = 0 as uint16;
    (*E1).phant_var_F = 0 as uint16;
    Phantoon_ChangeEyeSpriteBasedOnSamusDist(game);
    v3 = 510 as int16;
    let mut v4: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16);
    loop {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9100) as *mut uint16 as *mut uint8)
            .offset(v3 as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = v4;
        v3 = (v3 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as int16;
        if !(v3 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
    let ref mut fresh29 = *((&raw mut game.g_ram as *mut uint8).offset(0x1988) as *mut uint16);
    *fresh29 = (*fresh29 as ::core::ffi::c_int | 0x4000 as ::core::ffi::c_int) as uint16;
    (*Get_Phantoon(0xc0 as ::core::ffi::c_int, game)).phant_parameter_2 = 1 as uint16;
}
pub unsafe fn Phantoon_FadeOut(mut a: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut E: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
        if (*E).phant_var_F == 0 {
            (*E).phant_var_D = a;
            if Phantoon_Func_8(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
                (*E).phant_var_F = 1 as uint16;
            }
        }
    }
}
pub unsafe fn Phantoon_FadeIn(mut a: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut E: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
        if (*E).phant_var_F == 0 {
            (*E).phant_var_D = a;
            if Phantoon_SetColorBasedOnHp(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
            {
                (*E).phant_var_F = 1 as uint16;
            }
        }
    }
}
pub unsafe fn Phantoon_Spawn8FireballsInCircleAtStart(mut k: uint16, game: &mut Game) {
    let mut EK: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*EK).phant_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*EK).phant_var_E = (*EK).phant_var_E.wrapping_sub(1);
    let mut v3: bool = (*EK).phant_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        let mut E0: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
        SpawnEprojWithGfx(
            (*E0).phant_var_A,
            k,
            addr_kEproj_StartingFireballs as ::core::ffi::c_int as uint16,
            game,
        );
        QueueSfx3_Max6(0x1d as uint16, game);
        (*EK).phant_var_E = 30 as uint16;
        let mut v5: uint16 =
            ((*E0).phant_var_A as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E0).phant_var_A = v5;
        if v5 as ::core::ffi::c_int - 8 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            (*E0).phant_var_A = 0 as uint16;
            (*Get_Phantoon(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game))
                .phant_var_B = 0 as uint16;
            (*EK).phant_var_F =
                (0xa7d4ee as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*EK).phant_var_E = 30 as uint16;
            SpawnHardcodedPlm(
                SpawnHardcodedPlmArgs {
                    field_0: 0 as uint8,
                    field_1: 0x6 as uint8,
                    field_2: 0xb781 as VoidP,
                },
                game,
            );
        }
    }
}
pub unsafe fn Phantoon_WaitBetweenSpawningAndSpinningFireballs(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).phant_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).phant_var_E = (*E).phant_var_E.wrapping_sub(1);
    let mut v3: bool = (*E).phant_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*E).phant_var_E = 240 as uint16;
        (*E).phant_var_B = 1 as uint16;
        (*E).phant_var_F =
            (0xa7d508 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Phantoon_SpawnFireballsBeforeFight(mut k: uint16, game: &mut Game) {
    let mut EK: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
    let mut v3: bool = (*EK).phant_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*EK).phant_var_E = (*EK).phant_var_E.wrapping_sub(1);
    let mut v4: bool = (*EK).phant_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v3 as ::core::ffi::c_int != 0 || v4 as ::core::ffi::c_int != 0 {
        (*EK).phant_var_B = 0 as uint16;
        let ref mut fresh30 = *((&raw mut game.g_ram as *mut uint8).offset(0x1988) as *mut uint16);
        *fresh30 = (*fresh30 as ::core::ffi::c_int | 0x4000 as ::core::ffi::c_int) as uint16;
        (*EK).phant_var_F =
            (0xa7d54a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut E3: *mut Enemy_Phantoon = Get_Phantoon(0xc0 as ::core::ffi::c_int, game);
        (*E3).phant_parameter_1 = -(32767 as ::core::ffi::c_int) as uint16;
        (*EK).phant_var_E = 120 as uint16;
        sub_88E487(2 as uint16, g_word_A7CDA3, game);
        (*E3).phant_var_D = g_word_A7CD9D;
        (*Get_Phantoon(0x40 as ::core::ffi::c_int, game)).phant_var_F = 0 as uint16;
        QueueMusic_Delayed8(5 as uint16, game);
    }
}
pub unsafe fn Phantoon_WavyFadeIn(mut k: uint16, game: &mut Game) {
    Phantoon_FadeIn(0xc as uint16, game);
    if Phantoon_Func_3(
        ((g_word_A7CD9D as uint32) << 16 as ::core::ffi::c_int | g_word_A7CD9B as uint32) as int32,
        game,
    ) != 0
    {
        let mut E: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
        (*E).phant_var_F =
            (0xa7d596 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*Get_Phantoon(0xc0 as ::core::ffi::c_int, game)).phant_parameter_1 = 1 as uint16;
        (*E).phant_var_E = 30 as uint16;
    } else {
        let mut E_0: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
        let mut v2: bool = (*E_0).phant_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
        (*E_0).phant_var_E = (*E_0).phant_var_E.wrapping_sub(1);
        let mut v3: bool = (*E_0).phant_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
        if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
            (*Get_Phantoon(0xc0 as ::core::ffi::c_int, game)).phant_var_E = 1 as uint16;
        }
    };
}
pub unsafe fn Phantoon_PickPatternForRound1(mut k: uint16, game: &mut Game) {
    let mut EK: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*EK).phant_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*EK).phant_var_E = (*EK).phant_var_E.wrapping_sub(1);
    let mut v3: bool = (*EK).phant_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        let mut E1: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
        (*E1).phant_parameter_1 = 0 as uint16;
        (*EK).phant_var_F =
            (0xa7d5e7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E1).phant_var_A = *(game.g_rom.offset(
            (((0xa7cd53 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7cd53 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16)
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int) as isize,
            );
        if NextRandom(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let mut E: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
            (*E).phant_var_C = 0 as uint16;
            (*E).phant_var_B = 0 as uint16;
            (*E).phant_var_D = 0 as uint16;
            (*E1).phant_var_C = 1 as uint16;
            (*E).phant_var_A = 533 as uint16;
        } else {
            let mut E_0: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
            (*E_0).phant_var_C = 1 as uint16;
            (*E_0).phant_var_B = 0 as uint16;
            (*E_0).phant_var_D = 0 as uint16;
            (*E1).phant_var_C = 0 as uint16;
            (*E_0).phant_var_A = 0 as uint16;
        }
    }
}
pub unsafe fn Phantoon_MovePhantoonInFigure8ThenOpenEye(mut k: uint16, game: &mut Game) {
    Phantoon_AdjustSpeedAndMoveInFigure8(game);
    Phantoon_Func_7(k, game);
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).phant_var_A as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).phant_var_A = (*E).phant_var_A.wrapping_sub(1);
    let mut v3: bool = (*E).phant_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*Get_Phantoon(k as ::core::ffi::c_int, game)).phant_var_F =
            (0xa7d4a8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_kKraid_Ilist_CC53 as ::core::ffi::c_int as uint16;
        (*Get_Phantoon(0 as ::core::ffi::c_int, game)).phant_parameter_2 = 0 as uint16;
        Phantoon_Func_5(k, game);
    }
}
pub unsafe fn Phantoon_EyeFollowsSamusUntilTimerRunsOut(mut k: uint16, game: &mut Game) {
    let mut EK: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*EK).phant_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*EK).phant_var_E = (*EK).phant_var_E.wrapping_sub(1);
    let mut v3: bool = (*EK).phant_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*Get_Phantoon(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game)).phant_var_B =
            0 as uint16;
        let mut E2: *mut Enemy_Phantoon = Get_Phantoon(0x80 as ::core::ffi::c_int, game);
        if (*E2).phant_var_A == 0 {
            (*EK).phant_var_F =
                (0xa7d4a8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            let mut E0: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
            (*E0).base.instruction_timer = 1 as uint16;
            let mut E1: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
            (*E1).base.instruction_timer = 1 as uint16;
            (*E0).base.current_instruction = addr_kKraid_Ilist_CC41 as ::core::ffi::c_int as uint16;
            (*E1).base.current_instruction = addr_kKraid_Ilist_CC81 as ::core::ffi::c_int as uint16;
            (*E0).base.properties = ((*E0).base.properties as ::core::ffi::c_int
                | kEnemyProps_Tangible as ::core::ffi::c_int)
                as uint16;
            (*E0).phant_parameter_2 = 1 as uint16;
            return;
        }
        (*E2).phant_var_A = 0 as uint16;
        (*EK).phant_var_E = 60 as uint16;
        (*EK).phant_var_F =
            (0xa7d65c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    Phantoon_ChangeEyeSpriteBasedOnSamusDist(game);
}
pub unsafe fn Phantoon_BecomesSolidAndBodyVuln(mut v0: uint16, game: &mut Game) {
    Phantoon_ChangeEyeSpriteBasedOnSamusDist(game);
    let ref mut fresh21 = *((&raw mut game.g_ram as *mut uint8).offset(0x1988) as *mut uint16);
    *fresh21 = (*fresh21 as ::core::ffi::c_int & !(0x4000 as ::core::ffi::c_int)) as uint16;
    Phantoon_BeginSwoopingPattern(v0, game);
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.current_instruction = addr_kKraid_Ilist_CC47 as ::core::ffi::c_int as uint16;
}
pub unsafe fn Phantoon_IsSwooping(mut k: uint16, game: &mut Game) {
    Phantoon_ChangeEyeSpriteBasedOnSamusDist(game);
    Phantoon_MoveInSwoopingPattern(k, game);
    let mut EK: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*EK).phant_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*EK).phant_var_E = (*EK).phant_var_E.wrapping_sub(1);
    let mut v3: bool = (*EK).phant_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*EK).phant_var_F =
            (0xa7d6b9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let ref mut fresh27 = *((&raw mut game.g_ram as *mut uint8).offset(0x1988) as *mut uint16);
        *fresh27 = (*fresh27 as ::core::ffi::c_int | 0x4000 as ::core::ffi::c_int) as uint16;
        let mut E0: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
        (*E0).base.instruction_timer = 1 as uint16;
        let mut E1: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
        (*E1).base.instruction_timer = 1 as uint16;
        (*E0).base.current_instruction = addr_kKraid_Ilist_CC41 as ::core::ffi::c_int as uint16;
        (*E1).base.current_instruction = addr_kKraid_Ilist_CC91 as ::core::ffi::c_int as uint16;
        (*E0).base.properties = ((*E0).base.properties as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
        (*E1).phant_var_F = 0 as uint16;
        (*Get_Phantoon(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game)).phant_var_B =
            0 as uint16;
    }
}
pub unsafe fn Phantoon_FadeoutWithSwoop(mut k: uint16, game: &mut Game) {
    Phantoon_MoveInSwoopingPattern(k, game);
    Phantoon_FadeOut(0xc as uint16, game);
    if (*Get_Phantoon(0x40 as ::core::ffi::c_int, game)).phant_var_F != 0 {
        let mut E: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
        (*E).phant_var_F =
            (0xa7d6d4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).phant_var_E = 120 as uint16;
    }
}
pub unsafe fn Phantoon_WaitAfterFadeOut(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).phant_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).phant_var_E = (*E).phant_var_E.wrapping_sub(1);
    let mut v3: bool = (*E).phant_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*E).phant_var_F =
            (0xa7d6e2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Phantoon_MoveLeftOrRightAndPickEyeOpenPatt(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
    if NextRandom(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        (*E).phant_var_A = 136 as uint16;
        (*E).base.x_pos = 208 as uint16;
    } else {
        (*E).phant_var_A = 399 as uint16;
        (*E).base.x_pos = 48 as uint16;
    }
    (*E).base.y_pos = 96 as uint16;
    let mut E1: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    (*E1).phant_var_C = 0 as uint16;
    (*E).phant_var_C = 1 as uint16;
    (*E).phant_var_B = 0 as uint16;
    (*E).phant_parameter_2 = 0 as uint16;
    Phantoon_PickPatternForRound2(game);
    (*Get_Phantoon(k as ::core::ffi::c_int, game)).phant_var_F =
        (0xa7d72d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E1).phant_var_F = 0 as uint16;
}
pub unsafe fn Phantoon_FadeInBeforeFigure8(mut k: uint16, game: &mut Game) {
    Phantoon_FadeIn(0xc as uint16, game);
    if (*Get_Phantoon(0x40 as ::core::ffi::c_int, game)).phant_var_F != 0 {
        (*Get_Phantoon(0 as ::core::ffi::c_int, game)).phant_var_F =
            (0xa7d5e7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Phantoon_BecomeSolidAfterRainingFireballs(mut k: uint16, game: &mut Game) {
    let mut E1: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    (*E1).phant_var_F = 0 as uint16;
    let ref mut fresh20 = *((&raw mut game.g_ram as *mut uint8).offset(0x1988) as *mut uint16);
    *fresh20 = (*fresh20 as ::core::ffi::c_int & !(0x4000 as ::core::ffi::c_int)) as uint16;
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
    (*E).base.instruction_timer = 1 as uint16;
    (*E1).base.instruction_timer = 1 as uint16;
    (*E).base.current_instruction = addr_kKraid_Ilist_CC47 as ::core::ffi::c_int as uint16;
    (*E1).base.current_instruction = addr_kKraid_Ilist_CC9D as ::core::ffi::c_int as uint16;
    (*Get_Phantoon(k as ::core::ffi::c_int, game)).phant_var_F =
        (0xa7d767 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Phantoon_FadeInDuringFireballRain(mut k: uint16, game: &mut Game) {
    Phantoon_FadeIn(1 as uint16, game);
    if (*Get_Phantoon(0x40 as ::core::ffi::c_int, game)).phant_var_F != 0 {
        let mut E: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_Tangible as ::core::ffi::c_int))
            as uint16;
        (*Get_Phantoon(k as ::core::ffi::c_int, game)).phant_var_F =
            (0xa7d788 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).phant_var_E = 90 as uint16;
    }
}
pub unsafe fn Phantoon_FollowSamusWithEyeDuringFireballRain(mut k: uint16, game: &mut Game) {
    let mut EK: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*EK).phant_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*EK).phant_var_E = (*EK).phant_var_E.wrapping_sub(1);
    let mut v3: bool = (*EK).phant_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*Get_Phantoon(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game)).phant_var_B =
            0 as uint16;
        let mut E2: *mut Enemy_Phantoon = Get_Phantoon(0x80 as ::core::ffi::c_int, game);
        if (*E2).phant_var_A != 0 {
            (*E2).phant_var_A = 0 as uint16;
            (*Get_Phantoon(0 as ::core::ffi::c_int, game)).phant_parameter_2 = 1 as uint16;
            Phantoon_BeginSwoopingPattern(k, game);
        } else {
            (*EK).phant_var_F =
                (0xa7d7d5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            let mut E1: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
            (*E1).phant_var_F = 0 as uint16;
            let mut E0: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
            (*E0).base.instruction_timer = 1 as uint16;
            (*E1).base.instruction_timer = 1 as uint16;
            (*E0).base.current_instruction = addr_kKraid_Ilist_CC41 as ::core::ffi::c_int as uint16;
            (*E1).base.current_instruction = addr_kKraid_Ilist_CC91 as ::core::ffi::c_int as uint16;
            (*E0).base.properties = ((*E0).base.properties as ::core::ffi::c_int
                | kEnemyProps_Tangible as ::core::ffi::c_int)
                as uint16;
            let ref mut fresh26 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1988) as *mut uint16);
            *fresh26 = (*fresh26 as ::core::ffi::c_int | 0x4000 as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn Phantoon_FadeOutDuringFireballRain(mut k: uint16, game: &mut Game) {
    Phantoon_FadeOut(0xc as uint16, game);
    if (*Get_Phantoon(0x40 as ::core::ffi::c_int, game)).phant_var_F != 0 {
        let mut E: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
        (*E).phant_var_F =
            (0xa7d7f7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).phant_var_E = *(game.g_rom.offset(
            (((0xa7cd63 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7cd63 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((NextRandom(game) as ::core::ffi::c_int & 7 as ::core::ffi::c_int) as isize);
    }
}
pub unsafe fn Phantoon_SpawnRainingFireballs(mut k: uint16, game: &mut Game) {
    let mut EK: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*EK).phant_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*EK).phant_var_E = (*EK).phant_var_E.wrapping_sub(1);
    let mut v3: bool = (*EK).phant_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        let mut a: uint16 =
            (NextRandom(game) as ::core::ffi::c_int & 7 as ::core::ffi::c_int) as uint16;
        let mut v4: ::core::ffi::c_int =
            8 as ::core::ffi::c_int * a as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut E0: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
        (*E0).phant_var_A = *(game.g_rom.offset(
            (((0xa7cdad as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7cdad as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v4 as isize);
        (*E0).base.x_pos = *(game.g_rom.offset(
            (((0xa7cdad as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7cdad as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v4 + 1 as ::core::ffi::c_int) as isize);
        (*E0).base.y_pos = *(game.g_rom.offset(
            (((0xa7cdad as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7cdad as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v4 + 2 as ::core::ffi::c_int) as isize);
        (*Get_Phantoon(0x40 as ::core::ffi::c_int, game)).phant_var_C = 0 as uint16;
        (*EK).phant_var_F =
            (0xa7d73f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        Phantoon_Func_6(k, a, game);
    }
}
pub unsafe fn Phantoon_FadeOutBeforeFirstFireballRain(mut k: uint16, game: &mut Game) {
    Phantoon_FadeOut(0xc as uint16, game);
    Phantoon_AdjustSpeedAndMoveInFigure8(game);
    Phantoon_Func_7(k, game);
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).phant_var_A as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).phant_var_A = (*E).phant_var_A.wrapping_sub(1);
    let mut v3: bool = (*E).phant_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*Get_Phantoon(0x80 as ::core::ffi::c_int, game)).phant_var_A = 0 as uint16;
        (*Get_Phantoon(k as ::core::ffi::c_int, game)).phant_var_F =
            (0xa7d73f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        if (*Get_Phantoon(0 as ::core::ffi::c_int, game)).base.x_pos as ::core::ffi::c_int
            - 128 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            Phantoon_Func_6(k, 0 as uint16, game);
        } else {
            Phantoon_Func_6(k, 2 as uint16, game);
        }
    }
}
pub unsafe fn Phantoon_FadeOutBeforeEnrage(mut k: uint16, game: &mut Game) {
    Phantoon_FadeOut(0xc as uint16, game);
    if (*Get_Phantoon(0x40 as ::core::ffi::c_int, game)).phant_var_F != 0 {
        let mut E: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
        (*E).phant_var_F =
            (0xa7d874 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).phant_var_E = 120 as uint16;
    }
}
pub unsafe fn Phantoon_MoveEnragedPhantoonToTopCenter(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).phant_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).phant_var_E = (*E).phant_var_E.wrapping_sub(1);
    let mut v3: bool = (*E).phant_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*E).phant_var_F =
            (0xa7d891 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut E_0: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
        (*E_0).base.x_pos = 128 as uint16;
        (*E_0).base.y_pos = 32 as uint16;
        (*Get_Phantoon(0x40 as ::core::ffi::c_int, game)).phant_var_F = 0 as uint16;
    }
}
pub unsafe fn Phantoon_FadeInEnragedPhantoon(mut k: uint16, game: &mut Game) {
    Phantoon_FadeIn(0xc as uint16, game);
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    if (*E).phant_var_F != 0 {
        (*Get_Phantoon(0 as ::core::ffi::c_int, game)).phant_var_F =
            (0xa7d8ac as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*Get_Phantoon(k as ::core::ffi::c_int, game)).phant_var_E = 4 as uint16;
        (*E).phant_var_F = 0 as uint16;
    }
}
pub unsafe fn Phantoon_Enraged(mut k: uint16, game: &mut Game) {
    let mut v5: int16 = 0;
    let mut v8: int16 = 0;
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).phant_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).phant_var_E = (*E).phant_var_E.wrapping_sub(1);
    let mut v3: bool = (*E).phant_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        if (*Get_Phantoon(0x40 as ::core::ffi::c_int, game)).phant_var_F as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v5 = 15 as int16;
            loop {
                v8 = v5;
                let fresh24 = v5;
                v5 = v5 - 1;
                SpawnEprojWithGfx(
                    (fresh24 as ::core::ffi::c_int | 0x200 as ::core::ffi::c_int) as uint16,
                    k,
                    addr_kEproj_DestroyableFireballs as ::core::ffi::c_int as uint16,
                    game,
                );
                if !((v8 as ::core::ffi::c_int - 9 as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int)
                {
                    break;
                }
            }
        } else {
            let mut i: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
            while i >= 0 as ::core::ffi::c_int {
                SpawnEprojWithGfx(
                    (i | 0x200 as ::core::ffi::c_int) as uint16,
                    k,
                    addr_kEproj_DestroyableFireballs as ::core::ffi::c_int as uint16,
                    game,
                );
                i -= 1;
            }
        }
        QueueSfx3_Max6(0x29 as uint16, game);
        let mut E1: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
        let mut v7: uint16 =
            ((*E1).phant_var_F as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E1).phant_var_F = v7;
        if v7 as ::core::ffi::c_int - 8 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*Get_Phantoon(k as ::core::ffi::c_int, game)).phant_var_E = 128 as uint16;
        } else {
            (*E1).base.instruction_timer = 1 as uint16;
            (*E1).base.current_instruction = addr_kKraid_Ilist_CC91 as ::core::ffi::c_int as uint16;
            (*E1).phant_var_F = 0 as uint16;
            (*Get_Phantoon(k as ::core::ffi::c_int, game)).phant_var_F =
                (0xa7d916 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn Phantoon_FadeoutAfterEnrage(mut k: uint16, game: &mut Game) {
    Phantoon_FadeOut(0xc as uint16, game);
    if (*Get_Phantoon(0x40 as ::core::ffi::c_int, game)).phant_var_F != 0 {
        let mut E: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
        (*E).phant_var_F =
            (0xa7d6d4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).phant_var_E = 120 as uint16;
    }
}
pub unsafe fn Phantoon_CompleteSwoopAfterFatalShot(mut k: uint16, game: &mut Game) {
    Phantoon_ChangeEyeSpriteBasedOnSamusDist(game);
    Phantoon_MoveInSwoopingPattern(k, game);
    let mut x_pos: uint16 = (*Get_Phantoon(0 as ::core::ffi::c_int, game)).base.x_pos;
    if x_pos as ::core::ffi::c_int - 96 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        if x_pos as ::core::ffi::c_int - 160 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0
        {
            (*Get_Phantoon(k as ::core::ffi::c_int, game)).phant_var_F =
                (0xa7d948 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn Phantoon_DyingPhantoonFadeInOut(mut k: uint16, game: &mut Game) {
    let mut E1: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    if (*E1).phant_var_C as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        Phantoon_FadeIn(0xc as uint16, game);
        if (*E1).phant_var_F == 0 {
            return;
        }
    } else {
        Phantoon_FadeOut(0xc as uint16, game);
        if (*E1).phant_var_F == 0 {
            return;
        }
    }
    (*E1).phant_var_F = 0 as uint16;
    let mut v2: uint16 =
        ((*E1).phant_var_C as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    (*E1).phant_var_C = v2;
    if v2 as ::core::ffi::c_int - 10 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        let mut E: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
        (*E).phant_var_F =
            (0xa7d98b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).phant_var_E = 15 as uint16;
        (*Get_Phantoon(0x80 as ::core::ffi::c_int, game)).phant_var_F = 0 as uint16;
        (*Get_Phantoon(0 as ::core::ffi::c_int, game)).phant_var_A = 0 as uint16;
    }
}
pub unsafe fn Phantoon_DyingPhantoonExplosions(mut k: uint16, game: &mut Game) {
    let mut EK: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*EK).phant_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*EK).phant_var_E = (*EK).phant_var_E.wrapping_sub(1);
    let mut v3: bool = (*EK).phant_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        let mut E2: *mut Enemy_Phantoon = Get_Phantoon(0x80 as ::core::ffi::c_int, game);
        let mut v5: uint16 =
            (4 as ::core::ffi::c_int * (*E2).phant_var_F as ::core::ffi::c_int) as uint16;
        let mut E0: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
        game.eproj_spawn_pt = Point16U {
            x: (*(game.g_rom.offset(
                (((0xa7da1d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa7da1d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(v5 as isize) as int8 as ::core::ffi::c_int
                + (*E0).base.x_pos as ::core::ffi::c_int) as uint16,
            y: (*(game.g_rom.offset(
                (((0xa7da1d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa7da1d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset((v5 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
                as int8 as ::core::ffi::c_int
                + (*E0).base.y_pos as ::core::ffi::c_int) as uint16,
        };
        let mut v11: uint16 = *(game.g_rom.offset(
            (((0xa7da1d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7da1d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset((v5 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize)
            as uint16;
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
            v11,
            game,
        );
        if v11 as ::core::ffi::c_int == 29 as ::core::ffi::c_int {
            QueueSfx2_Max6(0x24 as uint16, game);
        } else {
            QueueSfx2_Max6(0x2b as uint16, game);
        }
        (*EK).phant_var_E = *(game.g_rom.offset(
            (((0xa7da1d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7da1d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset((v5 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize)
            as uint16;
        let mut v9: uint16 =
            ((*E2).phant_var_F as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        (*E2).phant_var_F = v9;
        if v9 as ::core::ffi::c_int - 13 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            (*E2).phant_var_F = 5 as uint16;
            let mut v10: uint16 =
                ((*E0).phant_var_A as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            (*E0).phant_var_A = v10;
            if v10 as ::core::ffi::c_int - 3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                (*EK).phant_var_F =
                    (0xa7da51 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            }
        }
    }
}
pub unsafe fn Phantoon_WavyDyingPhantoonAndCry(mut k: uint16, game: &mut Game) {
    let mut Phantoon: *mut Enemy_Phantoon = ::core::ptr::null_mut::<Enemy_Phantoon>();
    sub_88E487(1 as uint16, g_word_A7CDA3, game);
    (*Get_Phantoon(0xc0 as ::core::ffi::c_int, game)).phant_var_D = 0 as uint16;
    (*Get_Phantoon(k as ::core::ffi::c_int, game)).phant_var_F =
        (0xa7da86 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    (*Phantoon).phant_var_C = 2 as uint16;
    let mut v2: uint16 = ((*Get_Phantoon(0 as ::core::ffi::c_int, game))
        .base
        .properties as ::core::ffi::c_int
        & !(kEnemyProps_DisableSamusColl as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int)
        | kEnemyProps_Tangible as ::core::ffi::c_int
        | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
    (*Phantoon).base.properties = v2;
    (*Get_Phantoon(0x80 as ::core::ffi::c_int, game))
        .base
        .properties = v2;
    (*Get_Phantoon(0xc0 as ::core::ffi::c_int, game))
        .base
        .properties = v2;
    QueueSfx2_Max6(0x7e as uint16, game);
}
pub unsafe fn Phantoon_DyingFadeOut(mut k: uint16, game: &mut Game) {
    Phantoon_Func_3(
        ((g_word_A7CDA1 as uint32) << 16 as ::core::ffi::c_int | g_word_A7CD9F as uint32) as int32,
        game,
    );
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    if (*E).phant_var_C as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
        Phantoon_FadeOut(0xc as uint16, game);
        if (*E).phant_var_F != 0 {
            (*Get_Phantoon(k as ::core::ffi::c_int, game)).phant_var_F =
                (0xa7dad7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if *(&raw mut (*E).phant_var_C as *mut uint8).offset(0) as ::core::ffi::c_int
            == 0xf2 as ::core::ffi::c_int
        {
            (*E).phant_var_C = -(1 as ::core::ffi::c_int) as uint16;
            (*E).phant_var_F = 0 as uint16;
        } else {
            let mut v2: uint8 = (*(&raw mut (*E).phant_var_C as *mut uint8).offset(0)
                as ::core::ffi::c_int
                + 16 as ::core::ffi::c_int) as uint8;
            *(&raw mut (*E).phant_var_C as *mut uint8).offset(0) = v2;
            *(&raw mut game.g_ram as *mut uint8).offset(0x57) = v2;
        }
    }
}
pub unsafe fn Phantoon_AlmostDead(mut k: uint16, game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x57) = 0 as uint8;
    let mut E1: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    (*E1).phant_parameter_1 = 0 as uint16;
    let ref mut fresh18 = *((&raw mut game.g_ram as *mut uint8).offset(0x1988) as *mut uint16);
    *fresh18 = (*fresh18 as ::core::ffi::c_int & !(0x4000 as ::core::ffi::c_int)) as uint16;
    (*Get_Phantoon(0xc0 as ::core::ffi::c_int, game)).phant_parameter_1 =
        -(1 as ::core::ffi::c_int) as uint16;
    let mut EK: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
    (*EK).phant_var_F = (0xa7db3d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*EK).phant_var_E = 60 as uint16;
    (*E1).phant_var_F = 0 as uint16;
    let mut E0: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
    (*E0).base.x_pos = 384 as uint16;
    (*E0).base.y_pos = 128 as uint16;
    let mut i: ::core::ffi::c_int = 1022 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let mut v5: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    (*gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .size = 1024 as uint16;
    v5 = (v5 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    (*gVramWriteEntry(v5 as ::core::ffi::c_int, game)).size = 0x2000 as uint16;
    v5 = (v5 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    let fresh19 = v5;
    v5 = v5.wrapping_add(1);
    *(&raw mut (*(gVramWriteEntry
        as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
        fresh19 as ::core::ffi::c_int,
        game,
    ))
    .size as *mut uint8)
        .offset(0) = 126 as uint8;
    (*gVramWriteEntry(v5 as ::core::ffi::c_int, game)).size = 18432 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v5 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Phantoon_Dead(mut k: uint16, game: &mut Game) {
    let mut EK: *mut Enemy_Phantoon = Get_Phantoon(k as ::core::ffi::c_int, game);
    if (*EK).phant_var_E != 0 {
        (*EK).phant_var_E = (*EK).phant_var_E.wrapping_sub(1);
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16)
        as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut E1: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
        (*E1).phant_var_D = 12 as uint16;
        if Phantoon_Func_9(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
            let ref mut fresh22 = *(&raw mut game.g_ram as *mut uint8).offset(0x69);
            *fresh22 = (*fresh22 as ::core::ffi::c_int | 2 as ::core::ffi::c_int) as uint8;
            Enemy_ItemDrop_Phantoon(k, game);
            let mut E0: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
            let mut v4: uint16 = ((*E0).base.properties as ::core::ffi::c_int
                | kEnemyProps_Deleted as ::core::ffi::c_int)
                as uint16;
            (*E0).base.properties = v4;
            (*E1).base.properties = v4;
            (*Get_Phantoon(0x80 as ::core::ffi::c_int, game))
                .base
                .properties = v4;
            (*Get_Phantoon(0xc0 as ::core::ffi::c_int, game))
                .base
                .properties = v4;
            let ref mut fresh23 = *((&raw mut game.g_ram as *mut uint8).offset(0xd828).offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize,
            ) as *mut uint8 as *mut uint16);
            *fresh23 = (*fresh23 as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint16;
            SpawnHardcodedPlm(
                SpawnHardcodedPlmArgs {
                    field_0: 0 as uint8,
                    field_1: 0x6 as uint8,
                    field_2: 0xb78b as VoidP,
                },
                game,
            );
            QueueMusic_Delayed8(3 as uint16, game);
        }
    }
}
pub unsafe fn Phantoon_Func_8(game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    if ((*E).phant_var_D as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >= (*E).phant_var_E as ::core::ffi::c_int
    {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 0x20 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 112 as ::core::ffi::c_int) as isize) =
                Phantoon_Func_10_CalculateNthTransitionColorFromXtoY(
                    (*E).phant_var_E,
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                        ((i >> 1 as ::core::ffi::c_int) + 112 as ::core::ffi::c_int) as isize,
                    ),
                    *(game.g_rom.offset(
                        (((0xa7ca41 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa7ca41 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset((i >> 1 as ::core::ffi::c_int) as isize),
                    game,
                );
            i += 2 as ::core::ffi::c_int;
        }
        (*E).phant_var_E = (*E).phant_var_E.wrapping_add(1);
        return 0 as uint8;
    } else {
        (*E).phant_var_E = 0 as uint16;
        return 1 as uint8;
    };
}
pub unsafe fn Phantoon_SetColorBasedOnHp(game: &mut Game) -> uint8 {
    let mut Entry: PairU16 = PairU16 { k: 0, j: 0 };
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    if ((*E).phant_var_D as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >= (*E).phant_var_E as ::core::ffi::c_int
    {
        let mut v2: uint16 = 0 as uint16;
        let mut v7: uint16 = 0;
        loop {
            v7 = v2;
            Entry = Phantoon_SetColorBasedOnHp_FindEntry(v2, game);
            let mut j: uint16 = Entry.j;
            let mut v5: uint16 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                    ((Entry.k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 112 as ::core::ffi::c_int) as isize,
                );
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                ((v7 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 112 as ::core::ffi::c_int)
                    as isize,
            ) = Phantoon_Func_10_CalculateNthTransitionColorFromXtoY((*E).phant_var_E, v5, j, game);
            v2 = (v7 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v7 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16
                as ::core::ffi::c_int)
                < 0x20 as ::core::ffi::c_int)
            {
                break;
            }
        }
        (*E).phant_var_E = (*E).phant_var_E.wrapping_add(1);
        return 0 as uint8;
    } else {
        (*E).phant_var_E = 0 as uint16;
        return 1 as uint8;
    };
}
pub unsafe fn Phantoon_SetColorBasedOnHp_FindEntry(mut k: uint16, game: &mut Game) -> PairU16 {
    let mut r24: uint16 = k;
    let mut r18: uint16 = 312 as uint16;
    let mut r20: uint16 = 312 as uint16;
    let mut r22: uint16 = 0 as uint16;
    while !((r20 as ::core::ffi::c_int
        - (*Get_Phantoon(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .base
        .health as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int)
    {
        r20 = (r20 as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
        r22 = r22.wrapping_add(1);
        if !(r22 as ::core::ffi::c_int - 7 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0)
        {
            break;
        }
    }
    let mut v1: *const uint16 = RomPtr_A7(
        (r24 as ::core::ffi::c_int
            + *(game.g_rom.offset(
                (((0xa7dc4a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa7dc4a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(r22 as isize) as ::core::ffi::c_int) as uint16_t,
        game,
    ) as *const uint16;
    return MakePairU16(r24, *v1);
}
pub unsafe fn Phantoon_Func_9(game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    if ((*E).phant_var_D as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >= (*E).phant_var_E as ::core::ffi::c_int
    {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 0xe0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize) =
                Phantoon_Func_10_CalculateNthTransitionColorFromXtoY(
                    (*E).phant_var_E,
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset((i >> 1 as ::core::ffi::c_int) as isize),
                    *(game.g_rom.offset(
                        (((0xa7ca61 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa7ca61 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset((i >> 1 as ::core::ffi::c_int) as isize),
                    game,
                );
            i += 2 as ::core::ffi::c_int;
        }
        (*E).phant_var_E = (*E).phant_var_E.wrapping_add(1);
        return 0 as uint8;
    } else {
        (*E).phant_var_E = 0 as uint16;
        return 1 as uint8;
    };
}
pub unsafe fn Phantoon_Func_10_CalculateNthTransitionColorFromXtoY(
    mut a: uint16,
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    return (Phantoon_CalculateNthTransitionColorComponentFromXtoY(
        a,
        (k as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as uint16,
        (j as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as uint16,
        game,
    ) as ::core::ffi::c_int
        + ((Phantoon_CalculateNthTransitionColorComponentFromXtoY(
            a,
            (k as ::core::ffi::c_int >> 5 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
                as uint16,
            (j as ::core::ffi::c_int >> 5 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
                as uint16,
            game,
        ) as ::core::ffi::c_int)
            << 5 as ::core::ffi::c_int)
        + ((Phantoon_CalculateNthTransitionColorComponentFromXtoY(
            a,
            (k as ::core::ffi::c_int >> 10 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
                as uint16,
            (j as ::core::ffi::c_int >> 10 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
                as uint16,
            game,
        ) as ::core::ffi::c_int)
            << 10 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn Phantoon_CalculateNthTransitionColorComponentFromXtoY(
    mut a: uint16,
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    if a == 0 {
        return k;
    }
    let mut v4: uint16 = (a as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_Phantoon = Get_Phantoon(0x40 as ::core::ffi::c_int, game);
    if v4 as ::core::ffi::c_int == (*E).phant_var_D as ::core::ffi::c_int {
        return j;
    }
    let mut r20: uint16 = (v4 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    let mut r18: uint16 = (j as ::core::ffi::c_int - k as ::core::ffi::c_int) as uint16;
    let mut v6: uint8 = abs16(r18) as uint8;
    let mut RegWord: uint16 = SnesDivide(
        ((v6 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16,
        (*(&raw mut (*E).phant_var_D as *mut uint8).offset(0) as ::core::ffi::c_int
            - r20 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint8,
    );
    r18 = (if r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(RegWord as ::core::ffi::c_int)
    } else {
        RegWord as ::core::ffi::c_int
    }) as uint16;
    return ((r18 as ::core::ffi::c_int + u16::swap_bytes(k) as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Phantoon_Hurt(game: &mut Game) {
    let mut Entry: PairU16 = PairU16 { k: 0, j: 0 };
    let mut E0: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
    if !((*E0).base.flash_timer as ::core::ffi::c_int == 8 as ::core::ffi::c_int) {
        if (*E0).base.frame_counter as ::core::ffi::c_int & 2 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if *(&raw mut (*(Get_Phantoon
                as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut Enemy_Phantoon)(
                0x80 as ::core::ffi::c_int,
                game,
            ))
            .phant_parameter_2 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                )
                == 0
            {
                return;
            }
        } else {
            if *(&raw mut (*(Get_Phantoon
                as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut Enemy_Phantoon)(
                0x80 as ::core::ffi::c_int,
                game,
            ))
            .phant_parameter_2 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                )
                == 0
            {
                let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
                while i >= 0 as ::core::ffi::c_int {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                        ((i >> 1 as ::core::ffi::c_int) + 112 as ::core::ffi::c_int) as isize,
                    ) = 0x7fff as uint16;
                    i -= 2 as ::core::ffi::c_int;
                }
                let mut E3_0: *mut Enemy_Phantoon = Get_Phantoon(0x80 as ::core::ffi::c_int, game);
                (*E3_0).phant_parameter_2 = ((*E3_0).phant_parameter_2 as ::core::ffi::c_int
                    | 0x100 as ::core::ffi::c_int)
                    as uint16;
            }
            return;
        }
    }
    let mut v1: uint16 = 30 as uint16;
    loop {
        Entry = Phantoon_SetColorBasedOnHp_FindEntry(v1, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
            ((Entry.k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 112 as ::core::ffi::c_int)
                as isize,
        ) = Entry.j;
        v1 = (Entry.k as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if !((Entry.k as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    let mut E3: *mut Enemy_Phantoon = Get_Phantoon(0x80 as ::core::ffi::c_int, game);
    (*E3).phant_parameter_2 = *(&raw mut (*E3).phant_parameter_2 as *mut uint8).offset(0) as uint16;
}
pub unsafe fn Phantoon_Touch(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn Phantoon_Shot(game: &mut Game) {
    let mut E2_0: *mut Enemy_Phantoon = ::core::ptr::null_mut::<Enemy_Phantoon>();
    let mut v9: uint16 = 0;
    let mut current_block: u64;
    let mut E0: *mut Enemy_Phantoon = Get_Phantoon(0 as ::core::ffi::c_int, game);
    if (*E0).phant_var_F as ::core::ffi::c_int + 0x26b8 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        return;
    }
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
    let mut EK: *mut Enemy_Phantoon = Get_Phantoon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut health: uint16 = (*EK).base.health;
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    if (*EK).base.health == 0 {
        QueueSfx2_Max6(0x73 as uint16, game);
        (*Get_Phantoon(0x80 as ::core::ffi::c_int, game)).phant_parameter_2 = 1 as uint16;
        (*E0).base.properties = ((*E0).base.properties as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
        Phantoon_StartDeathSequence(v1, game);
        return;
    }
    if (*EK).base.ai_handler_bits as ::core::ffi::c_int & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        QueueSfx2_Max6(0x73 as uint16, game);
        let mut phanto_var_F: uint16 = (*EK).phant_var_F;
        if phanto_var_F as ::core::ffi::c_int
            != 0xa7d60d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            && phanto_var_F as ::core::ffi::c_int
                != 0xa7d788 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            if phanto_var_F as ::core::ffi::c_int
                != 0xa7d678 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            {
                current_block = 17320862420850216187;
            } else {
                health = (health as ::core::ffi::c_int - (*EK).base.health as ::core::ffi::c_int)
                    as uint16;
                if health as ::core::ffi::c_int - 300 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                    || *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
                        *((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16)
                            as isize,
                    ) as ::core::ffi::c_int
                        & 0xf00 as ::core::ffi::c_int
                        != 512 as ::core::ffi::c_int
                {
                    let mut E2: *mut Enemy_Phantoon =
                        Get_Phantoon(v1 as ::core::ffi::c_int + 0x80 as ::core::ffi::c_int, game);
                    let mut v7: uint16 = (health as ::core::ffi::c_int
                        + (*E2).phant_var_B as ::core::ffi::c_int)
                        as uint16;
                    (*E2).phant_var_B = v7;
                    if v7 as ::core::ffi::c_int - 300 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0
                    {
                        (*EK).phant_var_E = 1 as uint16;
                    }
                    current_block = 17320862420850216187;
                } else {
                    current_block = 1670498606268217318;
                }
            }
        } else {
            health =
                (health as ::core::ffi::c_int - (*EK).base.health as ::core::ffi::c_int) as uint16;
            if health as ::core::ffi::c_int - 300 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
                && *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16) as isize,
                ) as ::core::ffi::c_int
                    & 0xf00 as ::core::ffi::c_int
                    == 512 as ::core::ffi::c_int
            {
                current_block = 1670498606268217318;
            } else {
                E2_0 = ::core::ptr::null_mut::<Enemy_Phantoon>();
                E2_0 = Get_Phantoon(v1 as ::core::ffi::c_int + 0x80 as ::core::ffi::c_int, game);
                v9 = 0;
                v9 = (health as ::core::ffi::c_int + (*E2_0).phant_var_B as ::core::ffi::c_int)
                    as uint16;
                (*E2_0).phant_var_B = v9;
                if v9 as ::core::ffi::c_int - 300 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    (*EK).phant_var_F =
                        (0xa7d6b9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                    current_block = 4949463109084628481;
                } else {
                    let mut v10: uint16 = (NextRandom(game) as ::core::ffi::c_int
                        & 7 as ::core::ffi::c_int)
                        as uint16;
                    (*Get_Phantoon(0x40 as ::core::ffi::c_int, game)).phant_var_B =
                        *(&raw const g_byte_A7CDA5 as *const uint8)
                            .offset(v10 as ::core::ffi::c_int as isize)
                            as uint16;
                    (*Get_Phantoon(0xc0 as ::core::ffi::c_int, game)).phant_parameter_2 = v10;
                    let mut E2x: *mut Enemy_Phantoon =
                        Get_Phantoon(0x80 as ::core::ffi::c_int, game);
                    (*E2x).phant_parameter_2 = 1 as uint16;
                    if (*E2x).phant_var_A == 0 {
                        (*E2x).phant_var_A = 1 as uint16;
                        if (*EK).phant_var_E as ::core::ffi::c_int - 16 as ::core::ffi::c_int
                            & 0x8000 as ::core::ffi::c_int
                            == 0
                        {
                            (*EK).phant_var_E = 16 as uint16;
                        }
                    }
                    current_block = 6450597802325118133;
                }
            }
        }
        match current_block {
            6450597802325118133 => {}
            _ => {
                match current_block {
                    1670498606268217318 => {
                        (*EK).phant_var_F = (0xa7d85c as ::core::ffi::c_int
                            & 0xffff as ::core::ffi::c_int)
                            as uint16;
                        current_block = 4949463109084628481;
                    }
                    _ => {}
                }
                match current_block {
                    4949463109084628481 => {
                        (*EK).phant_var_E = 0 as uint16;
                        (*Get_Phantoon(0x80 as ::core::ffi::c_int, game)).phant_var_A = 0 as uint16;
                        (*Get_Phantoon(
                            v1 as ::core::ffi::c_int + 0x80 as ::core::ffi::c_int,
                            game,
                        ))
                        .phant_var_B = 0 as uint16;
                        let ref mut fresh28 =
                            *((&raw mut game.g_ram as *mut uint8).offset(0x1988) as *mut uint16);
                        *fresh28 = (*fresh28 as ::core::ffi::c_int | 0x4000 as ::core::ffi::c_int)
                            as uint16;
                        (*E0).base.instruction_timer = 1 as uint16;
                        let mut E1: *mut Enemy_Phantoon =
                            Get_Phantoon(0x40 as ::core::ffi::c_int, game);
                        (*E1).base.instruction_timer = 1 as uint16;
                        (*E0).base.current_instruction =
                            addr_kKraid_Ilist_CC41 as ::core::ffi::c_int as uint16;
                        (*E1).base.current_instruction =
                            addr_kKraid_Ilist_CC91 as ::core::ffi::c_int as uint16;
                        (*E0).base.properties = ((*E0).base.properties as ::core::ffi::c_int
                            | kEnemyProps_Tangible as ::core::ffi::c_int)
                            as uint16;
                        (*E1).phant_var_F = 0 as uint16;
                    }
                    _ => {}
                }
                (*Get_Phantoon(0x80 as ::core::ffi::c_int, game)).phant_parameter_2 = 2 as uint16;
                return;
            }
        }
    }
}
pub unsafe fn Etecoon_Init(game: &mut Game) {
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_DisableSamusColl as ::core::ffi::c_int) as uint16;
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A7 as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction = addr_kEtecoon_Ilist_E8CE as ::core::ffi::c_int as uint16;
    (*E).etecoon_var_F = (0xa7e9af as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).etecoon_var_E = -(1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Etecoon_Main(game: &mut Game) {
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if *(&raw mut (*E).etecoon_parameter_2 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) != 0
    {
        (*E).etecoon_parameter_2 =
            ((*E).etecoon_parameter_2 as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as uint16;
    } else {
        CallEnemyPreInstr(
            ((*E).etecoon_var_F as ::core::ffi::c_int | 0xa70000 as ::core::ffi::c_int) as uint32,
            game,
        );
    };
}
pub unsafe fn Etecoon_Func_1(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) != 0 {
        let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
        (*E).etecoon_parameter_2 = (*(&raw mut (*E).etecoon_parameter_2 as *mut uint8).offset(0)
            as ::core::ffi::c_int
            | 0x8000 as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = ((*E).base.instruction_timer as ::core::ffi::c_int
            + 128 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Etecoon_Func_2(mut k: uint16, game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
    return Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).etecoon_var_C as uint32) << 16 as ::core::ffi::c_int | (*E).etecoon_var_D as uint32)
            as int32,
        game,
    );
}
pub unsafe fn Etecoon_Func_3(mut k: uint16, game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
    let mut amt: int32 = (((*E).etecoon_var_A as uint32) << 16 as ::core::ffi::c_int
        | (*E).etecoon_var_B as uint32) as int32;
    if (*E).etecoon_var_A as ::core::ffi::c_int - 5 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        AddToHiLo(
            &raw mut (*E).etecoon_var_A,
            &raw mut (*E).etecoon_var_B,
            (*((&raw mut game.g_ram as *mut uint8).offset(0xb34) as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0xb32) as *mut uint16) as uint32,
        );
    }
    return Enemy_MoveDown(k, amt, game);
}
pub unsafe fn Etecoon_Func_4(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x797) as *mut uint16) == 0 {
        let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
        if (*E).etecoon_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            let mut v2: bool = (*E).etecoon_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
            (*E).etecoon_var_E = (*E).etecoon_var_E.wrapping_sub(1);
            let mut v3: bool = (*E).etecoon_var_E as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int;
            if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
                (*E).base.current_instruction =
                    addr_kEtecoon_Ilist_E854 as ::core::ffi::c_int as uint16;
                (*E).etecoon_var_F =
                    (0xa7ea00 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                (*E).etecoon_var_E = 11 as uint16;
            }
        } else if IsSamusWithinEnemy_Y(k, 0x80 as uint16, game) != 0 {
            if (*E).etecoon_parameter_2 as ::core::ffi::c_int & 3 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                QueueSfx2_Max15(0x35 as uint16, game);
            }
            (*E).base.instruction_timer = 1 as uint16;
            (*E).base.current_instruction =
                addr_kEtecoon_Ilist_E8D6 as ::core::ffi::c_int as uint16;
            (*E).etecoon_var_E = 256 as uint16;
        }
    }
}
pub unsafe fn Etecoon_Func_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).etecoon_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).etecoon_var_E = (*E).etecoon_var_E.wrapping_sub(1);
    let mut v3: bool = (*E).etecoon_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*E).etecoon_var_A = g_word_A7E900;
        (*E).etecoon_var_B = g_word_A7E902;
        (*E).base.current_instruction = ((*E).base.current_instruction as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).etecoon_var_F =
            (0xa7ea37 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            - 256 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            QueueSfx2_Max6(0x33 as uint16, game);
        }
    }
}
pub unsafe fn Etecoon_Func_6(mut k: uint16, game: &mut Game) {
    if Etecoon_Func_3(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
        if (*E).etecoon_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if IsSamusWithinEnemy_Y(k, 0x40 as uint16, game) as ::core::ffi::c_int != 0
                && IsSamusWithinEnemy_X(k, g_word_A7E910, game) as ::core::ffi::c_int != 0
            {
                let mut v2: uint16 = DetermineDirectionOfSamusFromEnemy(game);
                if v2 as ::core::ffi::c_int - 5 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    (*E).base.current_instruction =
                        addr_kEtecoon_Ilist_E81E as ::core::ffi::c_int as uint16;
                    (*E).etecoon_parameter_1 = 0 as uint16;
                } else {
                    (*E).base.current_instruction =
                        addr_kEtecoon_Ilist_E876 as ::core::ffi::c_int as uint16;
                    (*E).etecoon_parameter_1 = 1 as uint16;
                }
                (*E).etecoon_var_E = 32 as uint16;
                (*E).base.instruction_timer = 1 as uint16;
                (*E).etecoon_var_F =
                    (0xa7eab5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            } else {
                (*E).etecoon_var_E = 11 as uint16;
                (*E).etecoon_var_F =
                    (0xa7ea00 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                (*E).base.instruction_timer = 1 as uint16;
                (*E).base.current_instruction =
                    addr_kEtecoon_Ilist_E854 as ::core::ffi::c_int as uint16;
            }
        } else {
            (*E).etecoon_var_A = 0 as uint16;
            (*E).etecoon_var_B = 0 as uint16;
            (*E).base.instruction_timer = 3 as uint16;
            (*E).base.current_instruction = addr_stru_A7E862 as ::core::ffi::c_int as uint16;
        }
    }
}
pub unsafe fn Etecoon_Func_7(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).etecoon_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).etecoon_var_E = (*E).etecoon_var_E.wrapping_sub(1);
    let mut v3: bool = (*E).etecoon_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*E).base.current_instruction = ((*E).base.current_instruction as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        if (*E).etecoon_parameter_1 != 0 {
            (*E).etecoon_var_C = g_word_A7E908;
            (*E).etecoon_var_D = g_word_A7E90A;
            (*E).etecoon_var_F =
                (0xa7eb2c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            (*E).etecoon_var_C = g_word_A7E90C;
            (*E).etecoon_var_D = g_word_A7E90E;
            (*E).etecoon_var_F =
                (0xa7eb02 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        (*E).etecoon_var_A = g_word_A7E900;
        (*E).etecoon_var_B = g_word_A7E902;
    }
}
pub unsafe fn Etecoon_Func_8(mut k: uint16, game: &mut Game) {
    if Etecoon_Func_2(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
        (*E).etecoon_var_C = g_word_A7E908;
        (*E).etecoon_var_D = g_word_A7E90A;
        (*E).etecoon_var_F =
            (0xa7eb2c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_stru_A7E880 as ::core::ffi::c_int as uint16;
        (*E).etecoon_parameter_1 = 1 as uint16;
    }
}
pub unsafe fn Etecoon_Func_9(mut k: uint16, game: &mut Game) {
    if EnemyFunc_BBBF(
        k,
        ((32 as ::core::ffi::c_int as uint32) << 16 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int as uint16 as uint32) as int32,
        game,
    ) != 0
    {
        let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_stru_A7E898 as ::core::ffi::c_int as uint16;
        (*E).etecoon_var_F =
            (0xa7eb50 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        Etecoon_Func_2(k, game);
    };
}
pub unsafe fn Etecoon_Func_10(mut k: uint16, game: &mut Game) {
    Etecoon_Func_1(k, game);
    if Etecoon_Func_2(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
        if (*E).etecoon_parameter_1 != 0 {
            (*E).base.current_instruction =
                addr_kEtecoon_Ilist_E870 as ::core::ffi::c_int as uint16;
            (*E).etecoon_parameter_1 = 0 as uint16;
        } else {
            (*E).base.current_instruction =
                addr_kEtecoon_Ilist_E8C8 as ::core::ffi::c_int as uint16;
            (*E).etecoon_parameter_1 = 1 as uint16;
        }
        (*E).base.instruction_timer = 1 as uint16;
        (*E).etecoon_var_F =
            (0xa7ebcd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).etecoon_var_E = 8 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            - 256 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            QueueSfx2_Max6(0x32 as uint16, game);
        }
    } else if Etecoon_Func_3(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut E_0: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
        if (*E_0).etecoon_parameter_1 != 0 {
            (*E_0).base.current_instruction =
                addr_kEtecoon_Ilist_E854 as ::core::ffi::c_int as uint16;
        } else {
            (*E_0).base.current_instruction =
                addr_kEtecoon_Ilist_E8AC as ::core::ffi::c_int as uint16;
        }
        (*E_0).base.instruction_timer = 1 as uint16;
        (*E_0).etecoon_var_E = 11 as uint16;
        (*E_0).etecoon_var_F =
            (0xa7ec1b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E_0).etecoon_var_A = g_word_A7E900;
        (*E_0).etecoon_var_B = g_word_A7E902;
    }
}
pub unsafe fn Etecoon_Func_11(mut k: uint16, game: &mut Game) {
    Etecoon_Func_1(k, game);
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).etecoon_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).etecoon_var_E = (*E).etecoon_var_E.wrapping_sub(1);
    let mut v3: bool = (*E).etecoon_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        if (*E).etecoon_parameter_1 != 0 {
            (*E).base.current_instruction =
                addr_kEtecoon_Ilist_E894 as ::core::ffi::c_int as uint16;
            (*E).etecoon_var_C = g_word_A7E908;
            (*E).etecoon_var_D = g_word_A7E90A;
        } else {
            (*E).base.current_instruction =
                addr_kEtecoon_Ilist_E83C as ::core::ffi::c_int as uint16;
            (*E).etecoon_var_C = g_word_A7E90C;
            (*E).etecoon_var_D = g_word_A7E90E;
        }
        (*E).base.instruction_timer = 1 as uint16;
        (*E).etecoon_var_F =
            (0xa7eb50 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).etecoon_var_A = g_word_A7E900;
        (*E).etecoon_var_B = g_word_A7E902;
    }
}
static funcs_A2A79: [Option<Func_Y_V>; 3] = unsafe {
    [
        Some(Etecoon_Func_13 as unsafe fn(uint16, &mut Game) -> ()),
        Some(Etecoon_Func_14 as unsafe fn(uint16, &mut Game) -> ()),
        Some(Etecoon_Func_15 as unsafe fn(uint16, &mut Game) -> ()),
    ]
};
pub unsafe fn Etecoon_Func_12(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).etecoon_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).etecoon_var_E = (*E).etecoon_var_E.wrapping_sub(1);
    let mut v3: bool = (*E).etecoon_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*E).etecoon_var_A = g_word_A7E900;
        (*E).etecoon_var_B = g_word_A7E902;
        funcs_A2A79[*(&raw mut (*E).etecoon_parameter_2 as *mut uint8).offset(0) as usize]
            .expect("non-null function pointer")(k, game);
        (*Get_Etecoon(k as ::core::ffi::c_int, game))
            .base
            .instruction_timer = 1 as uint16;
    }
}
pub unsafe fn Etecoon_Func_13(mut j: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(j as ::core::ffi::c_int, game);
    (*E).etecoon_var_F = (0xa7ec97 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.current_instruction = addr_stru_A7E828 as ::core::ffi::c_int as uint16;
    (*E).etecoon_var_C = g_word_A7E90C;
    (*E).etecoon_var_D = g_word_A7E90E;
}
pub unsafe fn Etecoon_Func_14(mut j: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(j as ::core::ffi::c_int, game);
    (*E).etecoon_var_F = (0xa7ecbb as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.current_instruction = addr_stru_A7E880 as ::core::ffi::c_int as uint16;
    (*E).etecoon_var_C = g_word_A7E908;
    (*E).etecoon_var_D = g_word_A7E90A;
}
pub unsafe fn Etecoon_Func_15(mut j: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(j as ::core::ffi::c_int, game);
    (*E).etecoon_var_F = (0xa7ed75 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.current_instruction =
        ((*E).base.current_instruction as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    (*E).etecoon_var_C = g_word_A7E908;
    (*E).etecoon_var_D = g_word_A7E90A;
}
pub unsafe fn Etecoon_Func_16(mut k: uint16, game: &mut Game) {
    Etecoon_Func_2(k, game);
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
    if (*E).base.x_pos as ::core::ffi::c_int - 537 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).etecoon_var_E = 11 as uint16;
        (*E).etecoon_var_F =
            (0xa7edc7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_kEtecoon_Ilist_E854 as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn Etecoon_Func_17(mut k: uint16, game: &mut Game) {
    Etecoon_Func_2(k, game);
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
    if (*E).base.x_pos as ::core::ffi::c_int - 600 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E).etecoon_var_E = 11 as uint16;
        (*E).etecoon_var_F =
            (0xa7edc7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_kEtecoon_Ilist_E854 as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn Etecoon_Func_18(mut k: uint16, game: &mut Game) {
    Etecoon_Func_2(k, game);
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
    if (*E).base.x_pos as ::core::ffi::c_int - 600 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E).etecoon_var_F =
            (0xa7ed09 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).etecoon_var_A = g_word_A7E904;
        (*E).etecoon_var_B = g_word_A7E906;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_stru_A7E898 as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn Etecoon_Func_19(mut k: uint16, game: &mut Game) {
    Etecoon_Func_2(k, game);
    Etecoon_Func_3(k, game);
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
    if (*E).base.x_pos as ::core::ffi::c_int - 680 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_stru_A7E880 as ::core::ffi::c_int as uint16;
        (*E).etecoon_var_F =
            (0xa7ed2a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Etecoon_Func_20(mut k: uint16, game: &mut Game) {
    Etecoon_Func_2(k, game);
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
    if (*E).base.x_pos as ::core::ffi::c_int - 840 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_stru_A7E898 as ::core::ffi::c_int as uint16;
        (*E).etecoon_var_F =
            (0xa7ed54 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).etecoon_var_A = -(1 as ::core::ffi::c_int) as uint16;
        (*E).etecoon_var_B = g_word_A7E906;
    }
}
pub unsafe fn Etecoon_Func_21(mut k: uint16, game: &mut Game) {
    Etecoon_Func_2(k, game);
    if Etecoon_Func_3(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
        (*E).etecoon_var_E = 11 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_kEtecoon_Ilist_E854 as ::core::ffi::c_int as uint16;
        (*E).etecoon_var_F =
            (0xa7edc7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Etecoon_Func_22(mut k: uint16, game: &mut Game) {
    Etecoon_Func_1(k, game);
    if Etecoon_Func_3(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
        if (*E).etecoon_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*E).etecoon_var_E = 11 as uint16;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).base.current_instruction =
                addr_kEtecoon_Ilist_E854 as ::core::ffi::c_int as uint16;
            if (*E).etecoon_parameter_2 as ::core::ffi::c_int & 2 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
                && (*E).base.x_pos as ::core::ffi::c_int - 832 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
            {
                (*E).etecoon_var_F =
                    (0xa7ee3e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            } else {
                (*E).etecoon_var_F =
                    (0xa7edc7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            }
        } else {
            (*E).etecoon_var_A = 0 as uint16;
            (*E).etecoon_var_B = 0 as uint16;
            (*E).base.instruction_timer = 3 as uint16;
            (*E).base.current_instruction = addr_stru_A7E862 as ::core::ffi::c_int as uint16;
        }
    }
}
pub unsafe fn Etecoon_Func_23(mut k: uint16, game: &mut Game) {
    Etecoon_Func_1(k, game);
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).etecoon_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).etecoon_var_E = (*E).etecoon_var_E.wrapping_sub(1);
    let mut v3: bool = (*E).etecoon_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        let mut v4: uint16 =
            ((*E).etecoon_parameter_1 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as uint16;
        (*E).etecoon_parameter_1 = v4;
        if (v4 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int) - 1024 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            || {
                (*E).etecoon_parameter_1 =
                    *(&raw mut (*E).etecoon_parameter_1 as *mut uint8).offset(0) as uint16;
                (*E).etecoon_parameter_2 as ::core::ffi::c_int & 2 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
            }
        {
            (*E).etecoon_var_F =
                (0xa7ed75 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).base.current_instruction = ((*E).base.current_instruction as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16;
        } else {
            (*E).etecoon_var_F =
                (0xa7ee9a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).base.current_instruction = addr_stru_A7E880 as ::core::ffi::c_int as uint16;
            (*E).etecoon_var_C = g_word_A7E908;
            (*E).etecoon_var_D = g_word_A7E90A;
        }
        (*E).etecoon_var_A = g_word_A7E900;
        (*E).etecoon_var_B = g_word_A7E902;
        (*E).base.instruction_timer = 1 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            - 256 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            QueueSfx2_Max6(0x33 as uint16, game);
        }
    }
}
pub unsafe fn Etecoon_Func_24(mut k: uint16, game: &mut Game) {
    Etecoon_Func_1(k, game);
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).etecoon_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).etecoon_var_E = (*E).etecoon_var_E.wrapping_sub(1);
    let mut v3: bool = (*E).etecoon_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        if IsSamusWithinEnemy_Y(k, 0x40 as uint16, game) as ::core::ffi::c_int != 0
            && IsSamusWithinEnemy_X(k, 0x30 as uint16, game) as ::core::ffi::c_int != 0
        {
            (*E).base.current_instruction = addr_stru_A7E880 as ::core::ffi::c_int as uint16;
            (*E).etecoon_var_F =
                (0xa7ecdf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            (*E).etecoon_var_A = g_word_A7E900;
            (*E).etecoon_var_B = g_word_A7E902;
            (*E).base.current_instruction = ((*E).base.current_instruction as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16;
            (*E).etecoon_var_F =
                (0xa7ed75 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - 256 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                QueueSfx2_Max6(0x33 as uint16, game);
            }
        }
        (*E).base.instruction_timer = 1 as uint16;
    }
}
pub unsafe fn Etecoon_Func_25(mut k: uint16, game: &mut Game) {
    Etecoon_Func_2(k, game);
    let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
    if (*E).base.x_pos as ::core::ffi::c_int - 600 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E).etecoon_var_F =
            (0xa7eeb8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_stru_A7E898 as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn Etecoon_Func_26(mut k: uint16, game: &mut Game) {
    Etecoon_Func_2(k, game);
    if Etecoon_Func_3(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut E: *mut Enemy_Etecoon = Get_Etecoon(k as ::core::ffi::c_int, game);
        (*E).etecoon_var_C = g_word_A7E90C;
        (*E).etecoon_var_D = g_word_A7E90E;
        (*E).etecoon_var_F =
            (0xa7eb02 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).etecoon_var_A = g_word_A7E900;
        (*E).etecoon_var_B = g_word_A7E902;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_stru_A7E828 as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn Dachora_Init(game: &mut Game) {
    let mut dachor_parameter_1: int16 = 0;
    let mut E: *mut Enemy_Dachora = Get_Dachora(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_DisableSamusColl as ::core::ffi::c_int) as uint16;
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A7 as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    dachor_parameter_1 = (*E).dachor_parameter_1 as int16;
    if (dachor_parameter_1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        if dachor_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).base.current_instruction =
                addr_kDachora_Ilist_F4B9 as ::core::ffi::c_int as uint16;
        } else {
            (*E).base.current_instruction =
                addr_kDachora_Ilist_F3F7 as ::core::ffi::c_int as uint16;
        }
        (*E).dachor_var_F =
            (0xa7f98c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        if dachor_parameter_1 != 0 {
            (*E).base.current_instruction =
                addr_kDachora_Ilist_F45B as ::core::ffi::c_int as uint16;
        } else {
            (*E).base.current_instruction =
                addr_kDachora_Ilist_F399 as ::core::ffi::c_int as uint16;
        }
        (*E).dachor_var_F =
            (0xa7f570 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Dachora_Main(game: &mut Game) {
    let mut E: *mut Enemy_Dachora = Get_Dachora(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallEnemyPreInstr(
        ((*E).dachor_var_F as ::core::ffi::c_int | 0xa70000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn Dachora_Func_1(mut j: uint16, mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Dachora = Get_Dachora(k as ::core::ffi::c_int, game);
    let mut v3: uint16 = u16::swap_bytes((*E).base.palette_index);
    memcpy(
        (&raw mut game.g_ram as *mut uint8).offset(
            *(game.g_rom.offset(
                (((0xa7f55f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa7f55f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int as isize,
        ) as *mut ::core::ffi::c_void,
        RomPtr_A7(j as uint16_t, game) as *const ::core::ffi::c_void,
        32 as size_t,
    );
}
pub unsafe fn Dachora_Func_2(mut k: uint16, game: &mut Game) {
    Enemy_MoveDown(
        k,
        ((1 as ::core::ffi::c_int as uint32) << 16 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int as uint16 as uint32) as int32,
        game,
    );
    if IsSamusWithinEnemy_Y(k, 0x40 as uint16, game) as ::core::ffi::c_int != 0
        && IsSamusWithinEnemy_X(k, g_word_A7F4C9, game) as ::core::ffi::c_int != 0
    {
        let mut E: *mut Enemy_Dachora = Get_Dachora(k as ::core::ffi::c_int, game);
        if (*E).dachor_parameter_1 != 0 {
            (*E).base.current_instruction =
                addr_kDachora_Ilist_F48B as ::core::ffi::c_int as uint16;
        } else {
            (*E).base.current_instruction =
                addr_kDachora_Ilist_F3C9 as ::core::ffi::c_int as uint16;
        }
        (*E).base.instruction_timer = 1 as uint16;
        (*E).dachor_var_F =
            (0xa7f5bc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).dachor_var_A = g_word_A7F4CD;
        QueueSfx2_Max15(0x1d as uint16, game);
    }
}
pub unsafe fn Dachora_Func_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Dachora = Get_Dachora(k as ::core::ffi::c_int, game);
    let fresh2 = (*E).dachor_var_A;
    (*E).dachor_var_A = (*E).dachor_var_A.wrapping_sub(1);
    if fresh2 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        if (*E).dachor_parameter_1 != 0 {
            (*E).base.current_instruction =
                addr_kDachora_Ilist_F407 as ::core::ffi::c_int as uint16;
            (*E).dachor_var_F =
                (0xa7f65e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            (*E).base.current_instruction =
                addr_kDachora_Ilist_F345 as ::core::ffi::c_int as uint16;
            (*E).dachor_var_F =
                (0xa7f5ed as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        (*E).base.instruction_timer = 1 as uint16;
        (*E).dachor_var_E = 1 as uint16;
    }
}
pub unsafe fn Dachora_Func_4(mut k: uint16, game: &mut Game) {
    let mut amt: int32 = -Dachora_Func_6(k, game);
    let mut E: *mut Enemy_Dachora = Get_Dachora(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(k, amt, game) as ::core::ffi::c_int != 0 || {
        EnemyFunc_C8AD(k, game);
        (*E).base.x_pos as ::core::ffi::c_int - 96 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    } {
        (*E).base.current_instruction = addr_kDachora_Ilist_F407 as ::core::ffi::c_int as uint16;
        (*E).dachor_var_F =
            (0xa7f65e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).dachor_var_E = 1 as uint16;
        (*E).dachor_parameter_1 = 1 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).dachor_var_A = 0 as uint16;
        (*E).dachor_var_B = 0 as uint16;
        Dachora_Func_1(
            addr_kDachora_Palette as ::core::ffi::c_int as uint16,
            k,
            game,
        );
    }
}
pub unsafe fn Dachora_Func_5(mut k: uint16, game: &mut Game) {
    let mut amt: int32 = Dachora_Func_6(k, game);
    let mut E: *mut Enemy_Dachora = Get_Dachora(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(k, amt, game) != 0 {
        QueueSfx2_Max15(0x71 as uint16, game);
        (*E).base.current_instruction = addr_kDachora_Ilist_F345 as ::core::ffi::c_int as uint16;
        (*E).dachor_var_F =
            (0xa7f5ed as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).dachor_parameter_1 = 0 as uint16;
        (*E).dachor_var_A = 0 as uint16;
        Dachora_Func_1(
            addr_kDachora_Palette as ::core::ffi::c_int as uint16,
            k,
            game,
        );
    } else {
        EnemyFunc_C8AD(k, game);
        if (*E).base.x_pos as ::core::ffi::c_int - 1152 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).base.current_instruction =
                addr_kDachora_Ilist_F4B3 as ::core::ffi::c_int as uint16;
            (*E).dachor_var_F =
                (0xa7f78f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).dachor_var_A = g_word_A7F4CF;
            (*E).base.y_pos =
                ((*E).base.y_pos as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
            QueueSfx2_Max6(0x3d as uint16, game);
        } else {
            return;
        }
    }
    (*E).dachor_var_E = 0 as uint16;
    (*E).dachor_var_B = 0 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
}
pub unsafe fn Dachora_Func_6(mut k: uint16, game: &mut Game) -> int32 {
    let mut E: *mut Enemy_Dachora = Get_Dachora(k as ::core::ffi::c_int, game);
    if ((*E).dachor_var_A as ::core::ffi::c_int - g_word_A7F4D5 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        if (*E).dachor_var_E as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            QueueSfx2_Max6(0x39 as uint16, game);
        }
        let mut v2: uint16 =
            ((*E).dachor_var_E as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        (*E).dachor_var_E = v2;
        if v2 as uint8 == 0 {
            Dachora_Func_1(
                *(game.g_rom.offset(
                    (((0xa7f787 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa7f787 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(
                        *(&raw mut (*E).dachor_var_E as *mut uint8).offset(
                            (::core::mem::size_of::<uint16>() as usize)
                                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                .wrapping_sub(1 as usize) as isize,
                        ) as isize,
                    ),
                k,
                game,
            );
            let mut v3: uint16 =
                ((*E).dachor_var_E as ::core::ffi::c_int + 272 as ::core::ffi::c_int) as uint16;
            (*E).dachor_var_E = v3;
            if v3 as ::core::ffi::c_int - 1040 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                (*E).dachor_var_E = 784 as uint16;
            }
        }
    }
    Enemy_MoveDown(
        k,
        (1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    );
    if ((*E).dachor_var_A as ::core::ffi::c_int - g_word_A7F4D5 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
        && ((*E).dachor_var_B as ::core::ffi::c_int - g_word_A7F4D7 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
    {
        (*E).dachor_var_A = g_word_A7F4D5;
        (*E).dachor_var_B = g_word_A7F4D7;
        return ((g_word_A7F4D5 as uint32) << 16 as ::core::ffi::c_int | g_word_A7F4D7 as uint32)
            as int32;
    }
    AddToHiLo(
        &raw mut (*E).dachor_var_A,
        &raw mut (*E).dachor_var_B,
        (g_word_A7F4D9 as uint32) << 16 as ::core::ffi::c_int | g_word_A7F4DB as uint32,
    );
    if ((*E).dachor_var_A as ::core::ffi::c_int == 4 as ::core::ffi::c_int
        || (*E).dachor_var_A as ::core::ffi::c_int == 8 as ::core::ffi::c_int)
        && (*E).dachor_var_B == 0
    {
        (*E).base.current_instruction = ((*E).base.current_instruction as ::core::ffi::c_int
            + 28 as ::core::ffi::c_int) as uint16;
    }
    return (((*E).dachor_var_A as uint32) << 16 as ::core::ffi::c_int | (*E).dachor_var_B as uint32)
        as int32;
}
pub unsafe fn Dachora_Func_7(mut k: uint16, game: &mut Game) {
    Dachora_Func_10(k, game);
    let mut E: *mut Enemy_Dachora = Get_Dachora(k as ::core::ffi::c_int, game);
    let fresh3 = (*E).dachor_var_A;
    (*E).dachor_var_A = (*E).dachor_var_A.wrapping_sub(1);
    if fresh3 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).base.current_instruction = ((*E).base.current_instruction as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).dachor_var_F =
            (0xa7f806 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut E1: *mut Enemy_Dachora =
            Get_Dachora(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
        (*E1).dachor_var_A = 0 as uint16;
        (*E1).dachor_var_D = 0 as uint16;
        (*Get_Dachora(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game)).dachor_var_D =
            0 as uint16;
        (*Get_Dachora(k as ::core::ffi::c_int + 192 as ::core::ffi::c_int, game)).dachor_var_D =
            0 as uint16;
        (*Get_Dachora(k as ::core::ffi::c_int + 256 as ::core::ffi::c_int, game)).dachor_var_D =
            0 as uint16;
        (*E).dachor_var_C = 0 as uint16;
        (*E).dachor_var_D = 0 as uint16;
        (*E).base.y_pos =
            ((*E).base.y_pos as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
        QueueSfx2_Max6(0x3b as uint16, game);
        if (*E).dachor_parameter_1 != 0 {
            (*Get_Dachora(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game))
                .base
                .current_instruction = addr_kDachora_Ilist_F4B9 as ::core::ffi::c_int as uint16;
            (*Get_Dachora(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game))
                .base
                .current_instruction = addr_kDachora_Ilist_F4B9 as ::core::ffi::c_int as uint16;
            (*Get_Dachora(k as ::core::ffi::c_int + 192 as ::core::ffi::c_int, game))
                .base
                .current_instruction = addr_kDachora_Ilist_F4B9 as ::core::ffi::c_int as uint16;
            (*Get_Dachora(k as ::core::ffi::c_int + 256 as ::core::ffi::c_int, game))
                .base
                .current_instruction = addr_kDachora_Ilist_F4B9 as ::core::ffi::c_int as uint16;
        } else {
            (*Get_Dachora(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game))
                .base
                .current_instruction = addr_kDachora_Ilist_F3F7 as ::core::ffi::c_int as uint16;
            (*Get_Dachora(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game))
                .base
                .current_instruction = addr_kDachora_Ilist_F3F7 as ::core::ffi::c_int as uint16;
            (*Get_Dachora(k as ::core::ffi::c_int + 192 as ::core::ffi::c_int, game))
                .base
                .current_instruction = addr_kDachora_Ilist_F3F7 as ::core::ffi::c_int as uint16;
            (*Get_Dachora(k as ::core::ffi::c_int + 256 as ::core::ffi::c_int, game))
                .base
                .current_instruction = addr_kDachora_Ilist_F3F7 as ::core::ffi::c_int as uint16;
        }
        (*Get_Dachora(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game))
            .base
            .instruction_timer = 1 as uint16;
        (*Get_Dachora(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game))
            .base
            .instruction_timer = 1 as uint16;
        (*Get_Dachora(k as ::core::ffi::c_int + 192 as ::core::ffi::c_int, game))
            .base
            .instruction_timer = 1 as uint16;
        (*Get_Dachora(k as ::core::ffi::c_int + 256 as ::core::ffi::c_int, game))
            .base
            .instruction_timer = 1 as uint16;
    }
}
pub unsafe fn Dachora_Func_8(mut k: uint16, game: &mut Game) {
    Dachora_Func_10(k, game);
    Dachora_Func_9(k, game);
    let mut E: *mut Enemy_Dachora = Get_Dachora(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).dachor_var_C,
        &raw mut (*E).dachor_var_D,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xb34) as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0xb32) as *mut uint16) as uint32,
    );
    AddToHiLo(
        &raw mut (*E).dachor_var_A,
        &raw mut (*E).dachor_var_B,
        ((*E).dachor_var_C as uint32) << 16 as ::core::ffi::c_int | (*E).dachor_var_D as uint32,
    );
    let mut v6: uint16 = (*E).dachor_var_A;
    if v6 as ::core::ffi::c_int - 15 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        v6 = 15 as uint16;
    }
    let mut amt: int32 =
        ((v6 as uint32) << 16 as ::core::ffi::c_int | (*E).dachor_var_B as uint32) as int32;
    if Enemy_MoveDown(k, -amt, game) != 0 {
        if (*E).dachor_parameter_1 != 0 {
            (*E).base.current_instruction =
                addr_kDachora_Ilist_F3FF as ::core::ffi::c_int as uint16;
            (*E).dachor_parameter_1 = 0 as uint16;
        } else {
            (*E).base.current_instruction =
                addr_kDachora_Ilist_F4C1 as ::core::ffi::c_int as uint16;
            (*E).dachor_parameter_1 = 1 as uint16;
        }
        (*E).dachor_var_F =
            (0xa7f935 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).dachor_var_A = 0 as uint16;
        (*E).dachor_var_B = 0 as uint16;
        (*E).dachor_var_E = 0 as uint16;
        Dachora_Func_1(
            addr_kDachora_Palette as ::core::ffi::c_int as uint16,
            k,
            game,
        );
        QueueSfx2_Max6(0x3c as uint16, game);
    }
}
pub unsafe fn Dachora_Func_9(mut k: uint16, game: &mut Game) {
    let mut EK: *mut Enemy_Dachora = Get_Dachora(k as ::core::ffi::c_int, game);
    let mut E1: *mut Enemy_Dachora =
        Get_Dachora(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    let mut dachor_var_A: uint16 = (*E1).dachor_var_A;
    if dachor_var_A != 0 {
        (*E1).dachor_var_A =
            (dachor_var_A as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    } else {
        (*E1).dachor_var_A = g_word_A7F4D1;
        if (*E1).dachor_var_D != 0 {
            let mut E2: *mut Enemy_Dachora =
                Get_Dachora(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
            if (*E2).dachor_var_D != 0 {
                let mut E3: *mut Enemy_Dachora =
                    Get_Dachora(k as ::core::ffi::c_int + 192 as ::core::ffi::c_int, game);
                if (*E3).dachor_var_D != 0 {
                    let mut E4: *mut Enemy_Dachora =
                        Get_Dachora(k as ::core::ffi::c_int + 256 as ::core::ffi::c_int, game);
                    if (*E4).dachor_var_D == 0 {
                        (*E4).base.x_pos = (*EK).base.x_pos;
                        (*E4).base.y_pos = (*EK).base.y_pos;
                        (*E4).dachor_var_D = g_word_A7F4D3;
                    }
                } else {
                    (*E3).base.x_pos = (*EK).base.x_pos;
                    (*E3).base.y_pos = (*EK).base.y_pos;
                    (*E3).dachor_var_D = g_word_A7F4D3;
                }
            } else {
                (*E2).base.x_pos = (*EK).base.x_pos;
                (*E2).base.y_pos = (*EK).base.y_pos;
                (*E2).dachor_var_D = g_word_A7F4D3;
            }
        } else {
            (*E1).base.x_pos = (*EK).base.x_pos;
            (*E1).base.y_pos = (*EK).base.y_pos;
            (*E1).dachor_var_D = g_word_A7F4D3;
        }
    };
}
pub unsafe fn Dachora_Func_10(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Dachora = Get_Dachora(k as ::core::ffi::c_int, game);
    Dachora_Func_1(
        *(game.g_rom.offset(
            (((0xa7f92d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa7f92d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                *(&raw mut (*E).dachor_var_E as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as isize,
            ),
        k,
        game,
    );
    let mut v2: uint16 =
        ((*E).dachor_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as uint16;
    (*E).dachor_var_E = v2;
    if v2 as ::core::ffi::c_int - 1024 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        (*E).dachor_var_E = 0 as uint16;
    }
}
pub unsafe fn Dachora_Func_11(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Dachora = Get_Dachora(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).dachor_var_A,
        &raw mut (*E).dachor_var_B,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xb34) as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0xb32) as *mut uint16) as uint32,
    );
    let mut amt: int32 = (((*E).dachor_var_A as uint32) << 16 as ::core::ffi::c_int
        | (*E).dachor_var_B as uint32) as int32;
    if (amt >> 16 as ::core::ffi::c_int) - 10 as int32 & 0x8000 as int32 == 0 {
        amt = ((10 as ::core::ffi::c_int as int16 as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int) as int32;
    }
    if Enemy_MoveDown(k, amt, game) != 0 {
        if (*E).dachor_parameter_1 != 0 {
            (*E).base.current_instruction =
                addr_kDachora_Ilist_F407 as ::core::ffi::c_int as uint16;
            (*E).dachor_var_F =
                (0xa7f65e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            (*E).base.current_instruction =
                addr_kDachora_Ilist_F345 as ::core::ffi::c_int as uint16;
            (*E).dachor_var_F =
                (0xa7f5ed as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        (*E).base.instruction_timer = 1 as uint16;
        (*E).dachor_var_A = 0 as uint16;
        (*E).dachor_var_B = 0 as uint16;
    }
}
pub unsafe fn Dachora_Func_12(mut k: uint16, game: &mut Game) {
    let mut current_block: u64;
    let mut E: *mut Enemy_Dachora = Get_Dachora(k as ::core::ffi::c_int, game);
    let mut dachor_var_D: uint16 = (*E).dachor_var_D;
    if !(dachor_var_D == 0) {
        (*E).dachor_var_D =
            (dachor_var_D as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        if k as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16)
                as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                current_block = 6459227026555017041;
            } else {
                current_block = 7351195479953500246;
            }
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16)
            as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            current_block = 6459227026555017041;
        } else {
            current_block = 7351195479953500246;
        }
        match current_block {
            6459227026555017041 => {}
            _ => {
                (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                    & !(kEnemyProps_Invisible as ::core::ffi::c_int))
                    as uint16;
                return;
            }
        }
    }
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
}
