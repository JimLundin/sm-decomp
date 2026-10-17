use crate::game::sm_80::{
    NextRandom, PrepareBitAccess, QueueMusic_Delayed8, QueueSfx1_Max1, QueueSfx1_Max15,
    QueueSfx1_Max3, QueueSfx1_Max6, QueueSfx1_Max9, QueueSfx2_Max1, QueueSfx2_Max15,
    QueueSfx2_Max3, QueueSfx2_Max6, QueueSfx2_Max9, QueueSfx3_Max1, QueueSfx3_Max15,
    QueueSfx3_Max3, QueueSfx3_Max6, QueueSfx3_Max9,
};
use crate::game::sm_81::{DrawEprojSpritemapWithBaseTile, DrawEprojSpritemapWithBaseTileOffscreen};
use crate::game::sm_84::CalculatePlmBlockCoords;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game::sm_91::{
    Samus_DealDamage, Samus_RestoreHealth, Samus_RestoreMissiles, Samus_RestorePowerBombs,
    Samus_RestoreSuperMissiles,
};
use crate::game::sm_94::kSinCosTable8bit_Sext;
use crate::game::sm_a0::{
    CalculateAngleFromXY, CalculateAngleOfSamusFromEproj, CallEnemyAi, ConvertAngleToXy,
    SignExtend8, SpawnEnemyDrops, SuitDamageDivision,
};
use crate::game::sm_a6::Ridley_Func_103;
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::game_state::Game;
use crate::rtl::{Load24, Mult8x8, RomPtr, SnesDivide, SnesModulus, Unreachable};

pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t,
    EnemyBase, EnemyData, EnemyDef, EnemySpawnData, EprojDef, ExpandingSquareTransitionHdma,
    ExtraEnemyRam7800, ExtraEnemyRam8000, Func_V, Func_Y_V, LongPtr, Point16U, Ram7800_Kraid,
    Rect16U, VoidP, VramWriteEntry,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ExtraEnemyRam8800 {
    pub pad: [uint8; 64],
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Ram7800_Default {
    pub var_00: uint16,
    pub var_01: uint16,
    pub var_02: uint16,
    pub var_03: uint16,
    pub var_04: uint16,
    pub var_05: uint16,
    pub var_06: uint16,
    pub var_07: uint16,
    pub var_08: uint16,
    pub var_09: uint16,
    pub var_0A: uint16,
    pub var_0B: uint16,
    pub var_0C: uint16,
    pub var_0D: uint16,
    pub var_0E: uint16,
    pub var_0F: uint16,
    pub var_10: uint16,
    pub var_11: uint16,
    pub var_12: uint16,
    pub var_13: uint16,
    pub var_14: uint16,
    pub var_15: uint16,
    pub var_16: uint16,
    pub var_17: uint16,
    pub var_18: uint16,
    pub var_19: uint16,
    pub var_1A: uint16,
    pub var_1B: uint16,
    pub var_1C: uint16,
    pub var_1D: uint16,
    pub var_1E: uint16,
    pub var_1F: uint16,
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct EprojCollInfo {
    pub eci_r20: uint16,
    pub eci_r24: uint16,
    pub eci_r26: uint16,
    pub eci_r28: uint16,
    pub eci_r30: uint16,
    pub eci_r32: uint16,
    pub eci_r34: uint16,
}
pub type Func_EprojCollInfo_U8 = unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8;
pub use crate::types::{Enemy_MotherBrain, Point32};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[inline]
unsafe fn abs16(mut t: uint16) -> uint16 {
    return (if t as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(t as ::core::ffi::c_int)
    } else {
        t as ::core::ffi::c_int
    }) as uint16;
}
#[inline]
unsafe fn __CFADD__uint16(mut x: uint16, mut y: uint16) -> bool {
    return x as ::core::ffi::c_int
        > (x as ::core::ffi::c_int + y as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int;
}
#[inline]
unsafe fn __CFADD__uint8(mut x: uint8, mut y: uint8) -> bool {
    return x as ::core::ffi::c_int
        > (x as ::core::ffi::c_int + y as ::core::ffi::c_int) as uint8 as ::core::ffi::c_int;
}
#[inline]
unsafe fn RomPtr_86(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x860000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomPtr_A0(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0xa00000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomPtr_A1(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0xa10000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomPtr_B4(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0xb40000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomBankBase(mut bank: uint8, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (((bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) + 0x8000 as ::core::ffi::c_int)
            as uint32_t,
        game,
    )
    .offset(-(0x8000 as ::core::ffi::c_int as isize));
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
unsafe fn get_EprojDef(mut a: uint16, game: &mut Game) -> *mut EprojDef {
    return RomPtr(
        (0x860000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t,
        game,
    ) as *mut EprojDef;
}
#[inline]
unsafe fn gRam7800_Default(mut a: uint16, game: &mut Game) -> *mut Ram7800_Default {
    return (&raw mut game.g_ram as *mut uint8)
        .offset((0x7800 as ::core::ffi::c_int + a as ::core::ffi::c_int) as isize)
        as *mut uint8 as *mut Ram7800_Default;
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
pub const fnEprojInstr_Delete: uint32 = 8814932 as uint32;
pub const fnEprojInstr_Sleep: uint32 = 8814937 as uint32;
pub const fnEprojInstr_SetPreInstr_: uint32 = 8814945 as uint32;
pub const fnEprojInstr_ClearPreInstr: uint32 = 8814954 as uint32;
pub const fnEprojPreInstr_nullsub_297: uint32 = 8814960 as uint32;
pub const fnEprojInstr_CallFunc: uint32 = 8814961 as uint32;
pub const fnEprojInstr_Goto: uint32 = 8815019 as uint32;
pub const fnEprojInstr_GotoRel: uint32 = 8815024 as uint32;
pub const fnEprojInstr_DecTimerAndGotoIfNonZero: uint32 = 8815046 as uint32;
pub const fnEprojInstr_DecTimerAndGotoRelIfNonZero: uint32 = 8815054 as uint32;
pub const fnEprojInstr_SetTimer: uint32 = 8815061 as uint32;
pub const fnnullsub_82: uint32 = 8815070 as uint32;
pub const fnEprojInstr_MoveRandomlyWithinRadius: uint32 = 8815071 as uint32;
pub const fnEprojInstr_SetProjectileProperties: uint32 = 8815152 as uint32;
pub const fnEprojInstr_ClearProjectileProperties: uint32 = 8815164 as uint32;
pub const fnEprojInstr_EnableCollisionWithSamusProj: uint32 = 8815176 as uint32;
pub const fnEprojInstr_DisableCollisionWithSamusProj: uint32 = 8815186 as uint32;
pub const fnEprojInstr_DisableCollisionWithSamus: uint32 = 8815196 as uint32;
pub const fnEprojInstr_EnableCollisionWithSamus: uint32 = 8815206 as uint32;
pub const fnEprojInstr_SetToNotDieOnContact: uint32 = 8815216 as uint32;
pub const fnEprojInstr_SetToDieOnContact: uint32 = 8815226 as uint32;
pub const fnEprojInstr_SetLowPriority: uint32 = 8815236 as uint32;
pub const fnEprojInstr_SetHighPriority: uint32 = 8815246 as uint32;
pub const fnEprojInstr_SetXyRadius: uint32 = 8815256 as uint32;
pub const fnEprojInstr_SetXyRadiusZero: uint32 = 8815265 as uint32;
pub const fnEprojInstr_CalculateDirectionTowardsSamus: uint32 = 8815269 as uint32;
pub const fnEprojInstr_WriteColorsToPalette: uint32 = 8815317 as uint32;
pub const fnEprojInstr_QueueMusic: uint32 = 8815357 as uint32;
pub const fnEprojInstr_QueueSfx1_Max6: uint32 = 8815369 as uint32;
pub const fnEprojInstr_QueueSfx2_Max6: uint32 = 8815378 as uint32;
pub const fnEprojInstr_QueueSfx3_Max6: uint32 = 8815387 as uint32;
pub const fnEprojInstr_QueueSfx1_Max15: uint32 = 8815396 as uint32;
pub const fnEprojInstr_QueueSfx2_Max15: uint32 = 8815405 as uint32;
pub const fnEprojInstr_QueueSfx3_Max15: uint32 = 8815414 as uint32;
pub const fnEprojInstr_QueueSfx1_Max3: uint32 = 8815423 as uint32;
pub const fnEprojInstr_QueueSfx2_Max3: uint32 = 8815432 as uint32;
pub const fnEprojInstr_QueueSfx3_Max3: uint32 = 8815441 as uint32;
pub const fnEprojInstr_QueueSfx1_Max9: uint32 = 8815450 as uint32;
pub const fnEprojInstr_QueueSfx2_Max9: uint32 = 8815459 as uint32;
pub const fnEprojInstr_QueueSfx3_Max9: uint32 = 8815468 as uint32;
pub const fnEprojInstr_QueueSfx1_Max1: uint32 = 8815477 as uint32;
pub const fnEprojInstr_QueueSfx2_Max1: uint32 = 8815486 as uint32;
pub const fnEprojInstr_QueueSfx3_Max1: uint32 = 8815495 as uint32;
pub const fnEprojPreInstr_nullsub_83: uint32 = 8815867 as uint32;
pub const fnEproj_Init_0x8aaf: uint32 = 8817209 as uint32;
pub const fnEproj_PreInit_0x8aaf: uint32 = 8817277 as uint32;
pub const fnEproj_Init_0x8bc2_SkreeDownRight: uint32 = 8817357 as uint32;
pub const fnEproj_Init_0x8bd0_SkreeUpRight: uint32 = 8817393 as uint32;
pub const fnEproj_Init_0x8bde_SkreeDownLeft: uint32 = 8817429 as uint32;
pub const fnEproj_Init_0x8bec_SkreeUpLeft: uint32 = 8817465 as uint32;
pub const fnEproj_PreInstr_SkreeParticle: uint32 = 8817501 as uint32;
pub const fnEprojInstr_SpawnEnemyDropsWithDraygonsEyeDrops: uint32 = 8817768 as uint32;
pub const fnEprojInstr_SetPreInstrA: uint32 = 8817910 as uint32;
pub const fnEprojInstr_SetPreInstrB: uint32 = 8817917 as uint32;
pub const fnEprojInit_DraygonsGunk: uint32 = 8817924 as uint32;
pub const fnEprojInit_DraygonsWallTurretProjs: uint32 = 8817984 as uint32;
pub const fnnullsub_84: ::core::ffi::c_int = 0x868d54 as ::core::ffi::c_int;
pub const fnEprojInstr_868D99: uint32 = 8818073 as uint32;
pub const fnEprojPreInstr_8DCA: ::core::ffi::c_int = 0x868dca as ::core::ffi::c_int;
pub const fnEprojPreInstr_DraygonsTurret_8DFF: ::core::ffi::c_int = 0x868dff as ::core::ffi::c_int;
pub const fnEprojPreInstr_DraygonsGunk_8E0F: uint32 = 8818191 as uint32;
pub const fnEprojInit_CrocomireProjectile: uint32 = 8818723 as uint32;
pub const fnEprojPreInstr_CrocomireProjectile: uint32 = 8818795 as uint32;
pub const fnsub_8690B3: ::core::ffi::c_int = 0x8690b3 as ::core::ffi::c_int;
pub const fnEprojInit_CrocomireSpikeWallPieces: uint32 = 8818895 as uint32;
pub const fnEprojPreInstr_CrocomireSpikeWallPieces: uint32 = 8818965 as uint32;
pub const fnEprojInstr_9270: uint32 = 8819312 as uint32;
pub const fnEprojInit_CrocomireBridgeCrumbling: uint32 = 8819334 as uint32;
pub const fnEprojPreInstr_CrocomireBridgeCrumbling: uint32 = 8819386 as uint32;
pub const fnEprojInit_9634: uint32 = 8819533 as uint32;
pub const fnEprojPreInstr_9634: uint32 = 8819602 as uint32;
pub const fnEprojInit_9642_RidleysFireball: uint32 = 8819658 as uint32;
pub const fnEprojPreInstr_9642_RidleysFireball: uint32 = 8819726 as uint32;
pub const fnEprojInstr_DisableCollisionsWithSamus: uint32 = 8819829 as uint32;
pub const fnEprojInit_9660_FireballExplosion: uint32 = 8819839 as uint32;
pub const fnEprojInit_9688: uint32 = 8819872 as uint32;
pub const fnEprojInit_9696: uint32 = 8819892 as uint32;
pub const fnEprojInit_966C: uint32 = 8819912 as uint32;
pub const fnEprojInit_967A: uint32 = 8819932 as uint32;
pub const fnnullsub_85: uint32 = 8819980 as uint32;
pub const fnEprojPreInstr_966C: uint32 = 8819981 as uint32;
pub const fnEprojPreInstr_9688: uint32 = 8820002 as uint32;
pub const fnEprojPreInstr_96A4: uint32 = 8820023 as uint32;
pub const fnEprojPreInstr_96C0: uint32 = 8820032 as uint32;
pub const fnEprojPreInstr_96CE: uint32 = 8820041 as uint32;
pub const fnEprojInstr_95BA: uint32 = 8820154 as uint32;
pub const fnEprojInstr_95ED: uint32 = 8820205 as uint32;
pub const fnEprojInstr_9620: uint32 = 8820256 as uint32;
pub const fnEprojInit_9734_CeresFallingDebris: uint32 = 8820444 as uint32;
pub const fnEprojPreInstr_9734_CeresFallingDebris: uint32 = 8820481 as uint32;
pub const fnEprojInstr_980E: uint32 = 8820750 as uint32;
pub const fnEprojInit_PhantoonDestroyableFireballs: uint32 = 8820772 as uint32;
pub const fnEprojInit_PhantoonStartingFireballs: uint32 = 8821050 as uint32;
pub const fnEprojPreInstr_PhantoonStartingFireballs: uint32 = 8821121 as uint32;
pub const fnEprojPreInstr_PhantoonStartingFireballs2: ::core::ffi::c_int =
    0x8699bf as ::core::ffi::c_int;
pub const fnEprojPreInstr_PhantoonStartingFireballs3: ::core::ffi::c_int =
    0x869a01 as ::core::ffi::c_int;
pub const fnnullsub_86: ::core::ffi::c_int = 0x869a44 as ::core::ffi::c_int;
pub const fnEprojPreInstr_PhantoonDestroyableFireballs: ::core::ffi::c_int =
    0x869a45 as ::core::ffi::c_int;
pub const fnEprojPreInstr_PhantoonDestroyableFireballs_2: ::core::ffi::c_int =
    0x869a94 as ::core::ffi::c_int;
pub const fnEprojPreInstr_PhantoonDestroyableFireballs_3: ::core::ffi::c_int =
    0x869ada as ::core::ffi::c_int;
pub const fnEprojPreInstr_PhantoonStartingFireballsB: uint32 = 8821545 as uint32;
pub const fnEprojPreInstr_PhantoonStartingFireballsB_2: ::core::ffi::c_int =
    0x869b41 as ::core::ffi::c_int;
pub const fnEprojInit_RocksKraidSpits: uint32 = 8821923 as uint32;
pub const fnEprojInit_RocksFallingKraidCeiling: uint32 = 8821976 as uint32;
pub const fnEprojInit_RocksWhenKraidRises: uint32 = 8822028 as uint32;
pub const fnEprojPreInstr_KraidRocks: uint32 = 8822102 as uint32;
pub const fnEprojPreInstr_RocksFallingKraidCeiling: uint32 = 8822153 as uint32;
pub const fnsub_869DA5: uint32 = 8822181 as uint32;
pub const fnEprojInit_MiniKraidSpit: uint32 = 8822252 as uint32;
pub const fnEprojPreInit_MiniKraidSpit: uint32 = 8822302 as uint32;
pub const fnEprojInit_MiniKraidSpikesLeft: uint32 = 8822342 as uint32;
pub const fnEprojInit_MiniKraidSpikesRight: uint32 = 8822347 as uint32;
pub const fnEprojPreInstr_MiniKraidSpikes: uint32 = 8822403 as uint32;
pub const fnEprojInit_WalkingLavaSeahorseFireball: uint32 = 8822450 as uint32;
pub const fnEprojPreInstr_WalkingLavaSeahorseFireball: uint32 = 8822527 as uint32;
pub const fnEprojInit_PirateMotherBrainLaser: uint32 = 8822793 as uint32;
pub const fnEprojInstr_SetPreInstrAndRun: uint32 = 8822864 as uint32;
pub const fnnullsub_87: ::core::ffi::c_int = 0x86a05b as ::core::ffi::c_int;
pub const fnEprojPreInstr_PirateMotherBrainLaser_MoveLeft: uint32 = 8822876 as uint32;
pub const fnEprojPreInstr_PirateMotherBrainLaser_MoveRight: uint32 = 8822906 as uint32;
pub const fnEprojInit_PirateClaw: uint32 = 8822936 as uint32;
pub const fnEprojPreInstr_PirateClawThrownLeft: uint32 = 8822993 as uint32;
pub const fnEprojPreInstr_PirateClawThrownRight: uint32 = 8823076 as uint32;
pub const fnEprojInit_A379: uint32 = 8823457 as uint32;
pub const fnEprojInit_CeresElevatorPad: uint32 = 8823534 as uint32;
pub const fnEprojInit_CeresElevatorPlatform: uint32 = 8823579 as uint32;
pub const fnnullsub_88: uint32 = 8823591 as uint32;
pub const fnEprojPreInstr_CeresElevatorPad: uint32 = 8823592 as uint32;
pub const fnEprojPreInstr_CeresElevatorPlatform: uint32 = 8823652 as uint32;
pub const fnEprojPreInstr_PrePhantomRoom: uint32 = 8823715 as uint32;
pub const fnEprojInstr_A3BE: uint32 = 8823742 as uint32;
pub const fnEprojInstr_GotoWithProbability25: uint32 = 8823894 as uint32;
pub const fnEprojInit_BombTorizoLowHealthDrool: uint32 = 8824275 as uint32;
pub const fnEprojInit_BombTorizoLowHealthInitialDrool: uint32 = 8824413 as uint32;
pub const fnEprojInit_A977: uint32 = 8824519 as uint32;
pub const fnEprojInit_BombTorizoExplosiveSwipe: uint32 = 8824566 as uint32;
pub const fnEprojInit_BombTorizoStatueBreaking: uint32 = 8824676 as uint32;
pub const fnEprojInit_BombTorizoLowHealthExplode: uint32 = 8824859 as uint32;
pub const fnEprojInit_BombTorizoDeathExplosion: uint32 = 8824945 as uint32;
pub const fnsub_86A887: uint32 = 8824967 as uint32;
pub const fnEprojPreInstr_A977: uint32 = 8825071 as uint32;
pub const fnnullsub_352: uint32 = 8825112 as uint32;
pub const fnnullsub_89: uint32 = 8825113 as uint32;
pub const fnEprojInit_AB07: uint32 = 8825405 as uint32;
pub const fnEprojPreInstr_AB07: uint32 = 8825484 as uint32;
pub const fnEprojInstr_SpawnEnemyDrops: uint32 = 8825738 as uint32;
pub const fnEprojInit_BombTorizosChozoOrbs: uint32 = 8825835 as uint32;
pub const fnEprojInit_GoldenTorizosChozoOrbs: uint32 = 8825980 as uint32;
pub const fnEprojPreInstr_BombTorizosChozoOrbs: uint32 = 8826029 as uint32;
pub const fnEprojPreInstr_GoldenTorizosChozoOrbs: uint32 = 8826106 as uint32;
pub const fnEprojInstr_GotoDependingOnXDirection: uint32 = 8826258 as uint32;
pub const fnEprojInit_TorizoSonicBoom: uint32 = 8826389 as uint32;
pub const fnEprojPreInstr_TorizoSonicBoom: uint32 = 8826476 as uint32;
pub const fnEprojInit_WreckedShipChozoSpikeFootsteps: uint32 = 8826620 as uint32;
pub const fnEprojInstr_ResetXYpos1: uint32 = 8826678 as uint32;
pub const fnEprojInit_TourianStatueDustClouds: uint32 = 8826691 as uint32;
pub const fnEprojInit_TourianLandingDustCloudsRightFoot: uint32 = 8826704 as uint32;
pub const fnEprojInstr_MoveY_Minus4: uint32 = 8826770 as uint32;
pub const fnEprojInstr_SetVelTowardsSamus1: uint32 = 8827497 as uint32;
pub const fnEprojInstr_SetVelTowardsSamus2: uint32 = 8827506 as uint32;
pub const fnEprojInit_TorizoLandingDustCloudLeftFoot: uint32 = 8826829 as uint32;
pub const fnEprojInit_GoldenTorizoEgg: uint32 = 8826881 as uint32;
pub const fnEprojPreInstr_GoldenTorizoEgg: uint32 = 8826947 as uint32;
pub const fnsub_86B0B9: uint32 = 8827065 as uint32;
pub const fnsub_86B0DD: uint32 = 8827101 as uint32;
pub const fnsub_86B13E: uint32 = 8827198 as uint32;
pub const fnEprojInit_GoldenTorizoSuperMissile: uint32 = 8827342 as uint32;
pub const fnEprojPreInstr_GoldenTorizoSuperMissile: uint32 = 8827405 as uint32;
pub const fnEprojPreInstr_B237: uint32 = 8827447 as uint32;
pub const fnEprojInit_GoldenTorizoEyeBeam: uint32 = 8827688 as uint32;
pub const fnEprojPreInstr_GoldenTorizoEyeBeam: uint32 = 8827786 as uint32;
pub const fnEprojInstr_GotoIfFunc1: uint32 = 8827832 as uint32;
pub const fnEprojInstr_ResetXYpos2: uint32 = 8827958 as uint32;
pub const fnEprojInit_TourianEscapeShaftFakeWallExplode: uint32 = 8828061 as uint32;
pub const fnnullsub_90: uint32 = 8828080 as uint32;
pub const fnEprojInit_LavaSeahorseFireball: uint32 = 8828143 as uint32;
pub const fnsub_86B535: uint32 = 8828213 as uint32;
pub const fnEprojInit_EyeDoorProjectile: uint32 = 8828461 as uint32;
pub const fnEprojInit_EyeDoorSweat: uint32 = 8828547 as uint32;
pub const fnEprojPreInstr_EyeDoorProjectile: uint32 = 8828601 as uint32;
pub const fnEprojPreInstr_EyeDoorSweat: uint32 = 8828692 as uint32;
pub const fnEprojInstr_SpawnTourianStatueUnlockingParticle: uint32 = 8828906 as uint32;
pub const fnEprojInstr_Earthquake: uint32 = 8828917 as uint32;
pub const fnEprojInstr_SpawnTourianStatueUnlockingParticleTail: uint32 = 8828952 as uint32;
pub const fnEprojInstr_AddToYpos: uint32 = 8828993 as uint32;
pub const fnEprojInit_TourianStatueUnlockingParticleWaterSplash: uint32 = 8829050 as uint32;
pub const fnEprojInit_TourianStatueEyeGlow: uint32 = 8829070 as uint32;
pub const fnEprojInit_TourianStatueUnlockingParticle: uint32 = 8829109 as uint32;
pub const fnEprojIni_TourianStatueUnlockingParticleTail: uint32 = 8829160 as uint32;
pub const fnEprojInit_TourianStatueSoul: uint32 = 8829176 as uint32;
pub const fnEprojInit_TourianStatueBaseDecoration: uint32 = 8829246 as uint32;
pub const fnEprojInit_TourianStatueRidley: uint32 = 8829265 as uint32;
pub const fnEprojInit_TourianStatuePhantoon: uint32 = 8829284 as uint32;
pub const fnEprojPreInstr_TourianStatueUnlockingParticleWaterSplash: uint32 = 8829303 as uint32;
pub const fnEprojPreInstr_TourianStatueUnlockingParticle: uint32 = 8829314 as uint32;
pub const fnEprojPreInstr_TourianStatueSoul: uint32 = 8829437 as uint32;
pub const fnEprojPreInstr_TourianStatueStuff: uint32 = 8829495 as uint32;
pub const fnEprojPreInstr_BA42: uint32 = 8829506 as uint32;
pub const fnsub_86BB30: uint32 = 8829744 as uint32;
pub const fnnullsub_91: uint32 = 8829775 as uint32;
pub const fnEprojInit_NuclearWaffleBody: uint32 = 8829842 as uint32;
pub const fnnullsub_92: uint32 = 8829894 as uint32;
pub const fnEprojInit_NorfairLavaquakeRocks: uint32 = 8829915 as uint32;
pub const fnEprojPreInstr_NorfairLavaquakeRocks: uint32 = 8829967 as uint32;
pub const fnEproj_NorfairLavaquakeRocks_Func1: ::core::ffi::c_int = 0x86bc16 as ::core::ffi::c_int;
pub const fnEproj_NorfairLavaquakeRocks_Func2: ::core::ffi::c_int = 0x86bc8f as ::core::ffi::c_int;
pub const fnEprojPreInstr_NorfairLavaquakeRocks_Inner: uint32 = 8830238 as uint32;
pub const fnEprojPreInstr_NorfairLavaquakeRocks_Inner2: uint32 = 8830250 as uint32;
pub const fnEprojInit_ShaktoolAttackMiddleBackCircle: uint32 = 8830364 as uint32;
pub const fnEprojInit_BDA2: uint32 = 8830370 as uint32;
pub const fnEprojInit_ShaktoolAttackFrontCircle: uint32 = 8830467 as uint32;
pub const fnEprojPreInstr_BE12: uint32 = 8830482 as uint32;
pub const fnEprojInit_MotherBrainRoomTurrets: uint32 = 8830543 as uint32;
pub const fnEprojInit_MotherBrainRoomTurretBullets: uint32 = 8830809 as uint32;
pub const fnEprojPreInstr_MotherBrainRoomTurrets: uint32 = 8830943 as uint32;
pub const fnEprojPreInstr_MotherBrainRoomTurretBullets: uint32 = 8831200 as uint32;
pub const fnEprojInstr_SwitchJump: uint32 = 8831347 as uint32;
pub const fnEprojInstr_UserPalette0: uint32 = 8831412 as uint32;
pub const fnEproj_MotherBrainsBlueRingLasers: uint32 = 8831731 as uint32;
pub const fnEproj_MoveToBlueRingSpawnPosition: uint32 = 8831797 as uint32;
pub const fnEprojInit_MotherBrainBomb: uint32 = 8832130 as uint32;
pub const fnEprojPreInstr_MotherBrainBomb: uint32 = 8832200 as uint32;
pub const fnsub_86C605: uint32 = 8832517 as uint32;
pub const fnEprojInit_MotherBrainDeathBeemFired: uint32 = 8832644 as uint32;
pub const fnnullsub_366: uint32 = 8832877 as uint32;
pub const fnSpawnMotherBrainDeathBeam: uint32 = 8833019 as uint32;
pub const fnEprojInit_MotherBrainRainbowBeam: uint32 = 8833034 as uint32;
pub const fnEprojPreInstr_MotherBrainRainbowBeam: uint32 = 8833044 as uint32;
pub const fnEprojInit_MotherBrainsDrool: uint32 = 8833091 as uint32;
pub const fnEprojPreInstr_C84D: uint32 = 8833101 as uint32;
pub const fnEprojInit_MotherBrainsDrool_Falling: uint32 = 8833158 as uint32;
pub const fnEprojInstr_Add12ToY: uint32 = 8833232 as uint32;
pub const fnEprojInit_MotherBrainsDeathExplosion: uint32 = 8833269 as uint32;
pub const fnEprojPreInit_MotherBrainsDeathExplosion_0: uint32 = 8833300 as uint32;
pub const fnEprojInit_MotherBrainsRainbowBeamExplosion: uint32 = 8833327 as uint32;
pub const fnEprojPreInstr_MotherBrainsRainbowBeamExplosion: uint32 = 8833356 as uint32;
pub const fnEprojInit_MotherBrainEscapeDoorParticles: uint32 = 8833377 as uint32;
pub const fnEprojPreInstr_MotherBrainsExplodedDoorParticles: uint32 = 8833490 as uint32;
pub const fnEprojInit_MotherBrainPurpleBreathBig: uint32 = 8833642 as uint32;
pub const fnEprojInit_MotherBrainPurpleBreathSmall: uint32 = 8833667 as uint32;
pub const fnnullsub_94: uint32 = 8833699 as uint32;
pub const fnEprojInstr_MotherBrainPurpleBreathIsActive: uint32 = 8833774 as uint32;
pub const fnEprojInit_TimeBombSetJapaneseText: uint32 = 8833782 as uint32;
pub const fnEprojPreInstr_TimeBombSetJapaneseText: uint32 = 8833786 as uint32;
pub const fnEprojInit_MotherBrainTubeFalling: uint32 = 8833993 as uint32;
pub const fnEprojPreInstr_MotherBrainTubeFalling: uint32 = 8834023 as uint32;
pub const fnMotherBrainTubeFallingFunc_GenerateExplosion: ::core::ffi::c_int =
    0x86cbea as ::core::ffi::c_int;
pub const fnMotherBrainTubeFallingFunc_Falling: ::core::ffi::c_int = 0x86cc08 as ::core::ffi::c_int;
pub const fnEprojInit_MotherBrainGlassShatteringShard: uint32 = 8834501 as uint32;
pub const fnEprojInit_MotherBrainGlassShatteringSparkle: uint32 = 8834669 as uint32;
pub const fnEprojPreInstr_MotherBrainGlassShatteringShard: uint32 = 8834715 as uint32;
pub const fnEprojInit_KiHunterAcidSpitLeft: uint32 = 8834960 as uint32;
pub const fnEprojInit_KiHunterAcidSpitRight: uint32 = 8834982 as uint32;
pub const fnsub_86CFD5: uint32 = 8835029 as uint32;
pub const fnsub_86CFE6: uint32 = 8835046 as uint32;
pub const fnnullsub_95: uint32 = 8835063 as uint32;
pub const fnsub_86CFF8: ::core::ffi::c_int = 0x86cff8 as ::core::ffi::c_int;
pub const fnEprojInit_KagosBugs: uint32 = 8835208 as uint32;
pub const fnEprojPreInstr_KagosBugs_Func1: uint32 = 8835251 as uint32;
pub const fnEprojPreInstr_KagosBugs: ::core::ffi::c_int = 0x86d0ca as ::core::ffi::c_int;
pub const fnnullsub_302: ::core::ffi::c_int = 0x86d0eb as ::core::ffi::c_int;
pub const fnEprojPreInstr_D0EC: ::core::ffi::c_int = 0x86d0ec as ::core::ffi::c_int;
pub const fnEprojPreInstr_D128: ::core::ffi::c_int = 0x86d128 as ::core::ffi::c_int;
pub const fnEprojInstr_D15C: uint32 = 8835420 as uint32;
pub const fnEprojInstr_D1B6: uint32 = 8835510 as uint32;
pub const fnEprojInstr_D1C7: uint32 = 8835527 as uint32;
pub const fnEprojInstr_D1CE: uint32 = 8835534 as uint32;
pub const fnEprojPreInstr_KagosBugs_Func2: uint32 = 8835556 as uint32;
pub const fnEprojInit_MaridiaFloatersSpikes: uint32 = 8835642 as uint32;
pub const fnEprojPreInstr_MaridiaFloatersSpikes: uint32 = 8835683 as uint32;
pub const fnEprojInit_WreckedShipRobotLaserDown: uint32 = 8835852 as uint32;
pub const fnEprojInit_WreckedShipRobotLaserHorizontal: uint32 = 8835886 as uint32;
pub const fnEprojInit_WreckedShipRobotLaserUp: uint32 = 8835905 as uint32;
pub const fnEprojPreInstr_WreckedShipRobotLaser: uint32 = 8836031 as uint32;
pub const fnEprojInstr_AssignNewN00bTubeShardVelocity: uint32 = 8836577 as uint32;
pub const fnEprojInstr_SetN00bTubeShardX: uint32 = 8836594 as uint32;
pub const fnEprojInstr_D62A: uint32 = 8836650 as uint32;
pub const fnEprojInstr_SetXvelRandom: uint32 = 8836762 as uint32;
pub const fnEprojInit_N00bTubeCrack: uint32 = 8836773 as uint32;
pub const fnEprojInit_N00bTubeShards: uint32 = 8836809 as uint32;
pub const fnEprojInit_N00bTubeReleasedAirBubbles: uint32 = 8836980 as uint32;
pub const fnsub_86D7BF: uint32 = 8837055 as uint32;
pub const fnsub_86D7DE: uint32 = 8837086 as uint32;
pub const fnEprojPreInstr_N00bTubeShards: uint32 = 8837117 as uint32;
pub const fnsub_86D83D: uint32 = 8837181 as uint32;
pub const fnsub_86D89F: uint32 = 8837279 as uint32;
pub const fnsub_86D8DF: uint32 = 8837343 as uint32;
pub const fnsub_86D992: uint32 = 8837522 as uint32;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes: uint32 = 8837595 as uint32;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_MoveY1: uint32 = 8837606 as uint32;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_MoveY2: uint32 = 8837648 as uint32;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_MoveX1: uint32 = 8837690 as uint32;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_MoveX2: uint32 = 8837732 as uint32;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1: uint32 = 8837774 as uint32;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2: uint32 = 8837779 as uint32;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1: uint32 = 8837784 as uint32;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2: uint32 = 8837789 as uint32;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1: uint32 = 8837794 as uint32;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2: uint32 = 8837802 as uint32;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1: uint32 = 8837810 as uint32;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2: uint32 = 8837818 as uint32;
pub const fnEprojInit_DBF2: uint32 = 8837912 as uint32;
pub const fnEprojPreInstr_DBF2: uint32 = 8837979 as uint32;
pub const fnEprojPreInstr_DBF2_MoveX1: ::core::ffi::c_int = 0x86db62 as ::core::ffi::c_int;
pub const fnEprojPreInstr_DBF2_MoveX2: ::core::ffi::c_int = 0x86db8c as ::core::ffi::c_int;
pub const fnEprojPreInstr_DBF2_Func1: uint32 = 8838070 as uint32;
pub const fnEprojInstr_DC5A: uint32 = 8838234 as uint32;
pub const fnEprojInstr_SpawnEnemyDrops_0: uint32 = 8838241 as uint32;
pub const fnEprojInstr_SpawnSporesEproj: uint32 = 8838263 as uint32;
pub const fnEprojInit_Spores: uint32 = 8838285 as uint32;
pub const fnEprojInit_SporeSpawnStalk: uint32 = 8838307 as uint32;
pub const fnEprojInit_SporeSpawners: uint32 = 8838356 as uint32;
pub const fnEprojPreInstr_Spores: uint32 = 8838382 as uint32;
pub const fnnullsub_96: uint32 = 8838468 as uint32;
pub const fnEprojPreInstr_SporeSpawners: uint32 = 8838470 as uint32;
pub const fnEprojInit_NamiFuneFireball: uint32 = 8838870 as uint32;
pub const fnEprojPreInstr_NamiFuneFireball: uint32 = 8838969 as uint32;
pub const fnEprojInit_NamiFuneFireball_MoveX1: ::core::ffi::c_int = 0x86df40 as ::core::ffi::c_int;
pub const fnEprojInit_NamiFuneFireball_MoveX2: ::core::ffi::c_int = 0x86df6a as ::core::ffi::c_int;
pub const fnEprojInstr_DFEA: uint32 = 8839146 as uint32;
pub const fnEprojInit_LavaThrownByLavaman: uint32 = 8839168 as uint32;
pub const fnsub_86E049: uint32 = 8839241 as uint32;
pub const fnEproj_LavaThrownByLavaman_MoveX1: ::core::ffi::c_int = 0x86e050 as ::core::ffi::c_int;
pub const fnEproj_LavaThrownByLavaman_MoveX2: ::core::ffi::c_int = 0x86e07a as ::core::ffi::c_int;
pub const fnEprojInit_DustCloudOrExplosion: uint32 = 8840296 as uint32;
pub const fnEprojInit_EyeDoorSmoke: uint32 = 8840358 as uint32;
pub const fnEprojPreInstr_DustCloudOrExplosion: uint32 = 8840446 as uint32;
pub const fnEprojPreInstr_nullsub_98: uint32 = 8840456 as uint32;
pub const fnEprojInstr_SetYVel: uint32 = 8840499 as uint32;
pub const fnEprojInit_SpawnedShotGate: uint32 = 8840656 as uint32;
pub const fnEprojInit_ClosedDownwardsShotGate: uint32 = 8840661 as uint32;
pub const fnEprojInit_ClosedUpwardsShotGate: uint32 = 8840666 as uint32;
pub const fnEprojPreInstr_nullsub_99: uint32 = 8840708 as uint32;
pub const fnEprojPreInstr_E605: uint32 = 8840709 as uint32;
pub const fnEprojInit_SaveStationElectricity: uint32 = 8840877 as uint32;
pub const fnnullsub_100: uint32 = 8840913 as uint32;
pub const fnEprojInit_BotwoonsBody: uint32 = 8841777 as uint32;
pub const fnEprojPreInstr_BotwoonsBody: uint32 = 8841856 as uint32;
pub const fnEproj_BotwoonsBody_Main: ::core::ffi::c_int = 0x86ea98 as ::core::ffi::c_int;
pub const fnEproj_BotwonsBodyFunction_Dying: ::core::ffi::c_int = 0x86eaf4 as ::core::ffi::c_int;
pub const fnEproj_BotwonsBodyFunction_Dying2: uint32 = 8841988 as uint32;
pub const fnEproj_BotwonsBodyFunction_DyingFalling: ::core::ffi::c_int =
    0x86eb1f as ::core::ffi::c_int;
pub const fnnullsub_101: ::core::ffi::c_int = 0x86eb93 as ::core::ffi::c_int;
pub const fnEprojInit_BotwoonsSpit: uint32 = 8842182 as uint32;
pub const fnEprojPreInstr_BotwoonsSpit: uint32 = 8842245 as uint32;
pub const fnEprojInit_YappingMawsBody: uint32 = 8842338 as uint32;
pub const fnEprojPreInstr_Empty2: uint32 = 8842388 as uint32;
pub const fnEprojInstr_ECE3: uint32 = 8842467 as uint32;
pub const fnEprojInstr_ED17: uint32 = 8842519 as uint32;
pub const fnEprojInstr_QueueSfx2_9: uint32 = 8842891 as uint32;
pub const fnEprojInstr_QueueSfx2_24: uint32 = 8842903 as uint32;
pub const fnEprojInstr_QueueSfx2_B: uint32 = 8842915 as uint32;
pub const fnEprojInstr_EEAF: uint32 = 8842927 as uint32;
pub const fnEprojInstr_HandleRespawningEnemy: uint32 = 8843024 as uint32;
pub const fnEprojInit_F337: uint32 = 8843049 as uint32;
pub const fnEprojInit_EnemyDeathExplosion: uint32 = 8843145 as uint32;
pub const fnEprojPreInstr_Empty: ::core::ffi::c_int = 0x86efdf as ::core::ffi::c_int;
pub const fnEprojPreInstr_Pickup: ::core::ffi::c_int = 0x86efe0 as ::core::ffi::c_int;
pub const fnEprojInit_Sparks: uint32 = 8844177 as uint32;
pub const fnEprojPreInstr_Sparks: uint32 = 8844272 as uint32;
#[inline]
unsafe fn gEnemyData(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemyData {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut EnemyData;
}
#[inline]
unsafe fn gEnemySpawnData(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemySpawnData {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0x7000 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut EnemySpawnData;
}
#[inline]
unsafe fn gExtraEnemyRam7800(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut ExtraEnemyRam7800 {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0x7800 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut ExtraEnemyRam7800;
}
#[inline]
unsafe fn gExtraEnemyRam8000(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut ExtraEnemyRam8000 {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0x8000 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut ExtraEnemyRam8000;
}
#[inline]
unsafe fn Get_MotherBrain(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_MotherBrain {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_MotherBrain;
}

pub unsafe fn EnableEprojs(game: &mut Game) {
    let ref mut fresh18 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x198d as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh18 = (*fresh18 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn DisableEprojs(game: &mut Game) {
    let ref mut fresh17 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x198d as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh17 = (*fresh17 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn ClearEprojs(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 34 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
}
unsafe fn SpawnEprojInner(mut j: uint16, mut gfx_idx: uint16, game: &mut Game) {
    let mut v3: uint16 = 34 as uint16;
    while *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        != 0
    {
        v3 = (v3 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if v3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return;
        }
    }
    let mut v4: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = gfx_idx;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = j;
    let mut Edef: *mut EprojDef = get_EprojDef(j, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = (*Edef).pre_instr_ptr as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = (*Edef).instr_list as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1bb3 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = (*Edef).radius;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = (*Edef).properties;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b6b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = 0x8000 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19df as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1bfb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = 0 as uint16;
    CallEprojInit(
        ((*Edef).init_code_ptr as ::core::ffi::c_int | 0x860000 as ::core::ffi::c_int) as uint32,
        v3,
        game,
    );
}
pub unsafe fn SpawnEprojWithGfx(mut a: uint16, mut k: uint16, mut j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) = a;
    let mut E: *mut EnemyData = gEnemyData(k as ::core::ffi::c_int, game);
    SpawnEprojInner(
        j,
        ((*E).vram_tiles_index as ::core::ffi::c_int | (*E).palette_index as ::core::ffi::c_int)
            as uint16,
        game,
    );
}
pub unsafe fn SpawnEprojWithRoomGfx(mut j: uint16, mut a: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) = a;
    SpawnEprojInner(j, 0 as uint16, game);
}
unsafe fn EprojRunOne(mut k: uint16, game: &mut Game) {
    CallEprojPreInstr(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            | 0x860000 as ::core::ffi::c_int) as uint32,
        k,
        game,
    );
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1991 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1991 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let ref mut fresh19 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1b8f as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize);
    let fresh20 = *fresh19;
    *fresh19 = (*fresh19).wrapping_sub(1);
    if fresh20 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        let mut base: *const uint8 = RomBankBase(0x86 as uint8, game);
        let mut p: *const uint8 = base.offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int as isize,
        );
        while !(*(p as *mut uint16) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int)
        {
            p = CallEprojInstr(
                (*(p as *mut uint16) as ::core::ffi::c_int | 0x860000 as ::core::ffi::c_int)
                    as uint32,
                v1,
                p.offset(2 as ::core::ffi::c_int as isize),
                game,
            );
            if (p as uintptr_t) < 0x10000 as ::core::ffi::c_int as uintptr_t {
                if p.is_null() {
                    return;
                }
                p = base.offset(p as uintptr_t as isize);
            }
        }
        let mut v7: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v7 as isize) = *(p as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b6b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v7 as isize) = *(p.offset(2 as ::core::ffi::c_int as isize) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v7 as isize) = p.offset(4 as ::core::ffi::c_int as isize).offset_from(base)
            as ::core::ffi::c_long as uint16;
    }
}
pub unsafe fn EprojRunAll(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x198d as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut i: ::core::ffi::c_int = 34 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1991 as ::core::ffi::c_int as isize)
                as *mut uint16) = i as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize)
                != 0
            {
                EprojRunOne(i as uint16, game);
                i = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1991 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int;
            }
            i -= 2 as ::core::ffi::c_int;
        }
    }
}
pub unsafe fn EprojInstr_Delete(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return ::core::ptr::null::<uint8>();
}
pub unsafe fn EprojInstr_Sleep(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (epjp.offset_from(RomBankBase(0x86 as uint8, game)) as ::core::ffi::c_long
            - 2 as ::core::ffi::c_long) as uint16;
    return ::core::ptr::null::<uint8>();
}
pub unsafe fn EprojInstr_SetPreInstr_(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(epjp as *mut uint16);
    return epjp.offset(2 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_ClearPreInstr(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0x8170 as uint16;
    return epjp;
}
pub unsafe fn CallEprojFunc(mut ea: uint32, mut k: uint32, game: &mut Game) {
    match ea {
        8833019 => {
            SpawnMotherBrainDeathBeam(k as uint16, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn EprojInstr_CallFunc(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    CallEprojFunc(Load24(epjp as *mut LongPtr), k as uint32, game);
    return epjp.offset(3 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_Goto(mut k: uint16, mut epjp: *const uint8) -> *const uint8 {
    return *(epjp as *mut uint16) as uintptr_t as *const uint8;
}
pub unsafe fn EprojInstr_GotoRel(mut k: uint16, mut epjp: *const uint8) -> *const uint8 {
    return epjp.offset(*epjp as int8 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_DecTimerAndGotoIfNonZero(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh48 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x19df as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize);
    let fresh49 = *fresh48;
    *fresh48 = (*fresh48).wrapping_sub(1);
    if fresh49 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        return epjp.offset(2 as ::core::ffi::c_int as isize);
    } else {
        return EprojInstr_Goto(k, epjp);
    };
}
pub unsafe fn EprojInstr_DecTimerAndGotoRelIfNonZero(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh46 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x19df as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize);
    let fresh47 = *fresh46;
    *fresh46 = (*fresh46).wrapping_sub(1);
    if fresh47 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        return epjp.offset(1 as ::core::ffi::c_int as isize);
    } else {
        return EprojInstr_GotoRel(k, epjp);
    };
}
pub unsafe fn EprojInstr_SetTimer(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19df as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(epjp as *mut uint16);
    return epjp.offset(2 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_MoveRandomlyWithinRadius(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut Random: int8 = 0;
    let mut v4: int8 = 0;
    let mut v6: int16 = 0;
    let mut r18: uint16 = NextRandom(game);
    loop {
        Random = NextRandom(game) as int8;
        v4 = ((*epjp as ::core::ffi::c_int & Random as ::core::ffi::c_int)
            - *epjp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as int8;
        if !((v4 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int) {
            break;
        }
    }
    let ref mut fresh44 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh44 = (*fresh44 as ::core::ffi::c_int
        + if r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            -(v4 as uint8 as ::core::ffi::c_int)
        } else {
            v4 as uint8 as ::core::ffi::c_int
        }) as uint16;
    loop {
        *(&raw mut v6 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
            NextRandom(game) as uint8;
        *(&raw mut v6 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
            ((*epjp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & v6 as ::core::ffi::c_int)
                - *epjp.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as uint8;
        if !(v6 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 as ::core::ffi::c_int) {
            break;
        }
    }
    v6 = v6 as uint8 as int16;
    if r18 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        v6 = -(v6 as uint8 as ::core::ffi::c_int) as int16;
    }
    let ref mut fresh45 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh45 = (*fresh45 as ::core::ffi::c_int + v6 as ::core::ffi::c_int) as uint16;
    return epjp.offset(4 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_SetProjectileProperties(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh43 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1bd7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh43 =
        (*fresh43 as ::core::ffi::c_int | *(epjp as *mut uint16) as ::core::ffi::c_int) as uint16;
    return epjp.offset(2 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_ClearProjectileProperties(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh42 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1bd7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh42 =
        (*fresh42 as ::core::ffi::c_int & *(epjp as *mut uint16) as ::core::ffi::c_int) as uint16;
    return epjp.offset(2 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_EnableCollisionWithSamusProj(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh41 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1bd7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh41 = (*fresh41 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
    return epjp;
}
pub unsafe fn EprojInstr_DisableCollisionWithSamusProj(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh40 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1bd7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh40 = (*fresh40 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
    return epjp;
}
pub unsafe fn EprojInstr_DisableCollisionWithSamus(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh39 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1bd7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh39 = (*fresh39 as ::core::ffi::c_int | 0x2000 as ::core::ffi::c_int) as uint16;
    return epjp;
}
pub unsafe fn EprojInstr_EnableCollisionWithSamus(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh38 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1bd7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh38 = (*fresh38 as ::core::ffi::c_int & !(0x2000 as ::core::ffi::c_int)) as uint16;
    return epjp;
}
pub unsafe fn EprojInstr_SetToNotDieOnContact(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh37 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1bd7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh37 = (*fresh37 as ::core::ffi::c_int | 0x4000 as ::core::ffi::c_int) as uint16;
    return epjp;
}
pub unsafe fn EprojInstr_SetToDieOnContact(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh36 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1bd7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh36 = (*fresh36 as ::core::ffi::c_int & !(0x4000 as ::core::ffi::c_int)) as uint16;
    return epjp;
}
pub unsafe fn EprojInstr_SetLowPriority(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh35 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1bd7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh35 = (*fresh35 as ::core::ffi::c_int | 0x1000 as ::core::ffi::c_int) as uint16;
    return epjp;
}
pub unsafe fn EprojInstr_SetHighPriority(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh34 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1bd7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh34 = (*fresh34 as ::core::ffi::c_int & !(0x1000 as ::core::ffi::c_int)) as uint16;
    return epjp;
}
pub unsafe fn EprojInstr_SetXyRadius(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1bb3 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(epjp as *mut uint16);
    return epjp.offset(2 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_SetXyRadiusZero(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1bb3 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return epjp;
}
pub unsafe fn EprojInstr_CalculateDirectionTowardsSamus(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut x: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
    let mut y: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
    let mut v3: uint16 = (2 as ::core::ffi::c_int
        * CalculateAngleFromXY(x, y, game) as ::core::ffi::c_int)
        as uint16;
    let mut v4: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1991 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = v3;
    let mut v5: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) =
        kSinCosTable8bit_Sext[(v5 + 64 as ::core::ffi::c_int) as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = kSinCosTable8bit_Sext[v5 as usize] as uint16;
    return epjp;
}
pub unsafe fn EprojInstr_WriteColorsToPalette(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v3: uint16 = *(epjp.offset(2 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut n: ::core::ffi::c_int =
        *epjp.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
    let mut v4: uint16 = *(epjp as *mut uint16);
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *(RomPtr_86(v4 as uint16_t, game) as *mut uint16);
        v4 = (v4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        n -= 1;
        if !(n & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
    return epjp.offset(5 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_QueueMusic(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueMusic_Delayed8(*epjp as uint16, game);
    return epjp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_QueueSfx1_Max6(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx1_Max6(*epjp as uint16, game);
    return epjp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_QueueSfx2_Max6(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max6(*epjp as uint16, game);
    return epjp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_QueueSfx3_Max6(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx3_Max6(*epjp as uint16, game);
    return epjp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_QueueSfx1_Max15(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx1_Max15(*epjp as uint16, game);
    return epjp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_QueueSfx2_Max15(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max15(*epjp as uint16, game);
    return epjp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_QueueSfx3_Max15(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx3_Max15(*epjp as uint16, game);
    return epjp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_QueueSfx1_Max3(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx1_Max3(*epjp as uint16, game);
    return epjp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_QueueSfx2_Max3(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max3(*epjp as uint16, game);
    return epjp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_QueueSfx3_Max3(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx3_Max3(*epjp as uint16, game);
    return epjp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_QueueSfx1_Max9(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx1_Max9(*epjp as uint16, game);
    return epjp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_QueueSfx2_Max9(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max9(*epjp as uint16, game);
    return epjp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_QueueSfx3_Max9(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx3_Max9(*epjp as uint16, game);
    return epjp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_QueueSfx1_Max1(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx1_Max1(*epjp as uint16, game);
    return epjp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_QueueSfx2_Max1(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max1(*epjp as uint16, game);
    return epjp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_QueueSfx3_Max1(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx3_Max1(*epjp as uint16, game);
    return epjp.offset(1 as ::core::ffi::c_int as isize);
}
pub unsafe fn DrawLowPriorityEprojs(game: &mut Game) {
    let mut pt: Point16U = GetValuesForScreenShaking(game);
    let mut i: ::core::ffi::c_int = 34 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize)
            != 0
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0x1000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                DrawEprojs(i as uint16, pt, game);
            }
        }
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn DrawHighPriorityEprojs(game: &mut Game) {
    let mut pt: Point16U = GetValuesForScreenShaking(game);
    let mut i: ::core::ffi::c_int = 34 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize)
            != 0
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0x1000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                DrawEprojs(i as uint16, pt, game);
            }
        }
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn DrawEprojs(mut k: uint16, mut pt: Point16U, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1b6b as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    let mut r26: uint16 = *(((&raw mut game.g_ram as *mut uint8)
        .offset(0x19bb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as uint16;
    let mut R28: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x19bb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int) as uint16;
    let mut r20: uint16 = (pt.x as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    if r20 as ::core::ffi::c_int + 128 as ::core::ffi::c_int & 0xfe00 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut v3: uint16 = (pt.y as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        let mut r18: uint16 = v3;
        if v3 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            if v3 as ::core::ffi::c_int + 128 as ::core::ffi::c_int & 0xfe00 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                DrawEprojSpritemapWithBaseTileOffscreen(
                    0x8d as uint8,
                    v2,
                    r20,
                    r18,
                    r26,
                    R28,
                    game,
                );
            }
        } else {
            DrawEprojSpritemapWithBaseTile(0x8d as uint8, v2, r20, r18, r26, R28, game);
        }
    }
}
pub unsafe fn GetValuesForScreenShaking(game: &mut Game) -> Point16U {
    let mut R34: uint16 = 0;
    let mut R36: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 36 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let mut v0: ::core::ffi::c_int = (4 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)
            as uint16 as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 2 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            R36 = -(*(game.g_rom.offset(
                (((0x86846b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x86846b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v0 as isize) as ::core::ffi::c_int) as uint16;
            R34 = -(*(game.g_rom.offset(
                (((0x86846b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x86846b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v0 + 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int) as uint16;
        } else {
            R36 = *(game.g_rom.offset(
                (((0x86846b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x86846b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v0 as isize);
            R34 = *(game.g_rom.offset(
                (((0x86846b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x86846b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v0 + 1 as ::core::ffi::c_int) as isize);
        }
    } else {
        R34 = 0 as uint16;
        R36 = 0 as uint16;
    }
    return Point16U { x: R36, y: R34 };
}
unsafe fn EprojColl_8506(mut eci: *mut EprojCollInfo, game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        )
        != 0
    {
        let ref mut fresh1 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xdc4 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh1 = (*fresh1 as ::core::ffi::c_int
            + *(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402 as ::core::ffi::c_int as isize)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xdc4 as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ) as int8 as ::core::ffi::c_int) as uint16;
        return 0xff as uint8;
    }
    return 0 as uint8;
}
unsafe fn EprojColl_Unknown8536(mut eci: *mut EprojCollInfo, game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        )
        != 0
    {
        let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xdc4 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh0 = (*fresh0 as ::core::ffi::c_int
            + *(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402 as ::core::ffi::c_int as isize)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xdc4 as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ) as int8 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
        return 0xff as uint8;
    }
    return 0 as uint8;
}
unsafe fn EprojColl_ClearCarry(mut eci: *mut EprojCollInfo, game: &mut Game) -> uint8 {
    return 0 as uint8;
}
unsafe fn EprojColl_SetCarry(mut eci: *mut EprojCollInfo, game: &mut Game) -> uint8 {
    return 1 as uint8;
}
unsafe fn EprojColl_858E(mut eci: *mut EprojCollInfo, game: &mut Game) -> uint8 {
    let mut v0: uint16 = (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int) as uint16;
    if (v0 as ::core::ffi::c_int) < 5 as ::core::ffi::c_int {
        return EprojColl_85C2(
            eci,
            v0,
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16),
            game,
        );
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e77 as ::core::ffi::c_int as isize)
        as *mut uint16) = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as uint16;
    return EprojColl_873D();
}
unsafe fn EprojColl_85AD(mut eci: *mut EprojCollInfo, game: &mut Game) -> uint8 {
    let mut v0: uint16 = (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int) as uint16;
    if v0 as ::core::ffi::c_int >= 5 as ::core::ffi::c_int {
        return EprojColl_874E(eci, game);
    } else {
        return EprojColl_8676(
            eci,
            v0,
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16),
            0 as uint16,
            game,
        );
    };
}
static unk_868729: [uint8; 20] = [
    0 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0x82 as ::core::ffi::c_int as uint8,
    0x83 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0x81 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0x83 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0x83 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0x81 as ::core::ffi::c_int as uint8,
    0x82 as ::core::ffi::c_int as uint8,
    0x83 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0x81 as ::core::ffi::c_int as uint8,
    0x82 as ::core::ffi::c_int as uint8,
    0x83 as ::core::ffi::c_int as uint8,
];
unsafe fn EprojColl_85C2(
    mut eci: *mut EprojCollInfo,
    mut a: uint16,
    mut k: uint16,
    game: &mut Game,
) -> uint8 {
    let mut current_block: u64;
    let mut temp_collision_DD4: uint16 =
        (4 as ::core::ffi::c_int * a as ::core::ffi::c_int) as uint16;
    let mut temp_collision_DD6: uint16 = (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(k as isize) as ::core::ffi::c_int
        >> 6 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = (4 as ::core::ffi::c_int * a as ::core::ffi::c_int
        + (temp_collision_DD6 as ::core::ffi::c_int
            ^ ((*eci).eci_r34 as ::core::ffi::c_int & 8 as ::core::ffi::c_int)
                >> 3 as ::core::ffi::c_int)) as uint16;
    if (*eci).eci_r32 == 0 {
        let mut v3: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1991 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) as ::core::ffi::c_int
            - (*eci).eci_r30 as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            || (if unk_868729[v2 as usize] as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
                -(1 as ::core::ffi::c_int)
            } else {
                0 as ::core::ffi::c_int
            }) >= 0 as ::core::ffi::c_int
        {
            let mut v4: uint16 = (v2 as ::core::ffi::c_int ^ 2 as ::core::ffi::c_int) as uint16;
            if (*eci).eci_r30 as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1a93 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                || (if unk_868729[v4 as usize] as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
                    != 0
                {
                    -(1 as ::core::ffi::c_int)
                } else {
                    0 as ::core::ffi::c_int
                }) >= 0 as ::core::ffi::c_int
            {
                return 0 as uint8;
            }
        }
    } else {
        if (*eci).eci_r26 == 0 {
            if (*eci).eci_r30 as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1a93 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(
                        (*((&raw mut game.g_ram as *mut uint8)
                            .offset(0x1991 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            >> 1 as ::core::ffi::c_int) as isize,
                    ) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                if (if unk_868729[v2 as usize] as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
                    != 0
                {
                    -(1 as ::core::ffi::c_int)
                } else {
                    0 as ::core::ffi::c_int
                }) >= 0 as ::core::ffi::c_int
                {
                    return 0 as uint8;
                }
                current_block = 12039483399334584727;
            } else {
                current_block = 2204372562120708137;
            }
        } else if (*eci).eci_r26 as ::core::ffi::c_int != (*eci).eci_r32 as ::core::ffi::c_int
            || *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(
                    (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0x1991 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int
                - (*eci).eci_r30 as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            current_block = 2204372562120708137;
        } else {
            current_block = 3512920355445576850;
        }
        match current_block {
            12039483399334584727 => {}
            _ => {
                match current_block {
                    2204372562120708137 => {
                        if (if unk_868729[v2 as usize] as ::core::ffi::c_int
                            & 0x80 as ::core::ffi::c_int
                            != 0
                        {
                            -(1 as ::core::ffi::c_int)
                        } else {
                            0 as ::core::ffi::c_int
                        }) < 0 as ::core::ffi::c_int
                        {
                            current_block = 12039483399334584727;
                        } else {
                            current_block = 3512920355445576850;
                        }
                    }
                    _ => {}
                }
                match current_block {
                    12039483399334584727 => {}
                    _ => {
                        if (if unk_868729
                            [(v2 as ::core::ffi::c_int ^ 2 as ::core::ffi::c_int) as usize]
                            as ::core::ffi::c_int
                            & 0x80 as ::core::ffi::c_int
                            != 0
                        {
                            -(1 as ::core::ffi::c_int)
                        } else {
                            0 as ::core::ffi::c_int
                        }) >= 0 as ::core::ffi::c_int
                        {
                            return 0 as uint8;
                        }
                    }
                }
            }
        }
    }
    let mut v6: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1991 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v7: uint16 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v6 as isize) = 0 as uint16;
    if (*eci).eci_r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v7 = ((*eci).eci_r28 as ::core::ffi::c_int
            + ((*eci).eci_r34 as ::core::ffi::c_int | 7 as ::core::ffi::c_int)
            + 1 as ::core::ffi::c_int) as uint16;
    } else {
        v7 = (((*eci).eci_r34 as ::core::ffi::c_int & 0xfff8 as ::core::ffi::c_int)
            - (*eci).eci_r28 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v6 as isize) = v7;
    return 1 as uint8;
}
unsafe fn EprojColl_8676(
    mut eci: *mut EprojCollInfo,
    mut a: uint16,
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint8 {
    let mut current_block: u64;
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1991 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut temp_collision_DD4: uint16 =
        (4 as ::core::ffi::c_int * a as ::core::ffi::c_int) as uint16;
    let mut temp_collision_DD6: uint16 = (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402 as ::core::ffi::c_int as isize)
        .offset(k as isize) as ::core::ffi::c_int
        >> 6 as ::core::ffi::c_int) as uint16;
    let mut v3: uint16 = (4 as ::core::ffi::c_int * a as ::core::ffi::c_int
        + (temp_collision_DD6 as ::core::ffi::c_int
            ^ ((*eci).eci_r34 as ::core::ffi::c_int & 8 as ::core::ffi::c_int)
                >> 2 as ::core::ffi::c_int)) as uint16;
    if (*eci).eci_r32 == 0 {
        let mut v4: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize) as ::core::ffi::c_int
            - (*eci).eci_r28 as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            || (if unk_868729[v3 as usize] as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
                -(1 as ::core::ffi::c_int)
            } else {
                0 as ::core::ffi::c_int
            }) >= 0 as ::core::ffi::c_int
        {
            let mut v5: uint16 = (v3 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
            if (*eci).eci_r28 as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1a4b as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v4 as isize) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                || (if unk_868729[v5 as usize] as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
                    != 0
                {
                    -(1 as ::core::ffi::c_int)
                } else {
                    0 as ::core::ffi::c_int
                }) >= 0 as ::core::ffi::c_int
            {
                return 0 as uint8;
            }
        }
    } else {
        if (*eci).eci_r26 == 0 {
            if (*eci).eci_r28 as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1a4b as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(
                        (*((&raw mut game.g_ram as *mut uint8)
                            .offset(0x1991 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            >> 1 as ::core::ffi::c_int) as isize,
                    ) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                if (if unk_868729[v3 as usize] as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
                    != 0
                {
                    -(1 as ::core::ffi::c_int)
                } else {
                    0 as ::core::ffi::c_int
                }) >= 0 as ::core::ffi::c_int
                {
                    return 0 as uint8;
                }
                current_block = 5143058163439228106;
            } else {
                current_block = 8777407941634159499;
            }
        } else if (*eci).eci_r26 as ::core::ffi::c_int != (*eci).eci_r32 as ::core::ffi::c_int
            || *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(
                    (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0x1991 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int
                - (*eci).eci_r28 as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            current_block = 8777407941634159499;
        } else {
            current_block = 12349973810996921269;
        }
        match current_block {
            5143058163439228106 => {}
            _ => {
                match current_block {
                    8777407941634159499 => {
                        if (if unk_868729[v3 as usize] as ::core::ffi::c_int
                            & 0x80 as ::core::ffi::c_int
                            != 0
                        {
                            -(1 as ::core::ffi::c_int)
                        } else {
                            0 as ::core::ffi::c_int
                        }) < 0 as ::core::ffi::c_int
                        {
                            current_block = 5143058163439228106;
                        } else {
                            current_block = 12349973810996921269;
                        }
                    }
                    _ => {}
                }
                match current_block {
                    5143058163439228106 => {}
                    _ => {
                        if (if unk_868729
                            [(v3 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as usize]
                            as ::core::ffi::c_int
                            & 0x80 as ::core::ffi::c_int
                            != 0
                        {
                            -(1 as ::core::ffi::c_int)
                        } else {
                            0 as ::core::ffi::c_int
                        }) >= 0 as ::core::ffi::c_int
                        {
                            return 0 as uint8;
                        }
                    }
                }
            }
        }
    }
    let mut v7: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1991 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v8: uint16 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v7 as isize) = 0 as uint16;
    if (*eci).eci_r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v8 = ((*eci).eci_r30 as ::core::ffi::c_int
            + ((*eci).eci_r34 as ::core::ffi::c_int | 7 as ::core::ffi::c_int)
            + 1 as ::core::ffi::c_int) as uint16;
    } else {
        v8 = (((*eci).eci_r34 as ::core::ffi::c_int & 0xfff8 as ::core::ffi::c_int)
            - (*eci).eci_r30 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v7 as isize) = v8;
    return 1 as uint8;
}
pub unsafe fn EprojColl_873D() -> uint8 {
    return 0 as uint8;
}
unsafe fn EprojColl_874E(mut eci: *mut EprojCollInfo, game: &mut Game) -> uint8 {
    let mut v3: int16 = 0;
    let mut v5: int16 = 0;
    let mut v6: uint16 = 0;
    let mut v7: uint16 = 0;
    let mut v8: int16 = 0;
    let mut v11: int16 = 0;
    let mut v12: int16 = 0;
    let mut v13: uint16 = 0;
    let mut v14: uint16 = 0;
    let mut v15: int16 = 0;
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1991 as ::core::ffi::c_int as isize) as *mut uint16);
    if (*eci).eci_r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut v9: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xdc4 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut mod_0: uint16 = SnesModulus(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16),
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8,
        );
        let mut v10: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        v11 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v10 as isize) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as int16;
        if v11 as ::core::ffi::c_int == mod_0 as ::core::ffi::c_int {
            let mut temp_collision_DD4: uint16 =
                ((*eci).eci_r24 as ::core::ffi::c_int - (*eci).eci_r30 as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int
                    ^ 0xf as ::core::ffi::c_int) as uint16;
            let mut temp_collision_DD6: uint16 = (16 as ::core::ffi::c_int
                * (*(&raw mut game.g_ram as *mut uint8)
                    .offset(0x16402 as ::core::ffi::c_int as isize)
                    .offset(v9 as isize) as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int))
                as uint16;
            v12 = ((*(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402 as ::core::ffi::c_int as isize)
                .offset(v9 as isize) as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int) as int16;
            if (v12 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int && {
                if v12 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    v13 = (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0x1a4b as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v10 as isize) as ::core::ffi::c_int
                        ^ 0xf as ::core::ffi::c_int) as uint16;
                } else {
                    v13 = *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x1a4b as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v10 as isize);
                };
                v14 = (temp_collision_DD6 as ::core::ffi::c_int
                    + (v13 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int))
                    as uint16;
                v15 = ((*(game.g_rom.offset(
                    (((0x948b2b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x948b2b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint8)
                    .offset(v14 as isize) as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int)
                    - temp_collision_DD4 as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as int16;
                (*(game.g_rom.offset(
                    (((0x948b2b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x948b2b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint8)
                    .offset(v14 as isize) as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int)
                    - temp_collision_DD4 as ::core::ffi::c_int
                    == 1 as ::core::ffi::c_int
                    || (v15 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int
            } {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v10 as isize) =
                    ((*eci).eci_r24 as ::core::ffi::c_int - v15 as ::core::ffi::c_int) as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v10 as isize) = 0 as uint16;
                return 1 as uint8;
            } else {
                return 0 as uint8;
            }
        } else {
            return 0 as uint8;
        }
    } else {
        let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xdc4 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut mod_1: uint16 = SnesModulus(
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                as *mut uint16),
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8,
        );
        let mut v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        v3 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as int16;
        if v3 as ::core::ffi::c_int == mod_1 as ::core::ffi::c_int {
            let mut temp_collision_DD4_0: uint16 =
                ((*eci).eci_r30 as ::core::ffi::c_int + (*eci).eci_r24 as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as uint16;
            let mut temp_collision_DD6_0: uint16 = (16 as ::core::ffi::c_int
                * (*(&raw mut game.g_ram as *mut uint8)
                    .offset(0x16402 as ::core::ffi::c_int as isize)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int))
                as uint16;
            v5 = ((*(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402 as ::core::ffi::c_int as isize)
                .offset(v1 as isize) as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int) as int16;
            if v5 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int && {
                if v5 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    v6 = (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0x1a4b as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v2 as isize) as ::core::ffi::c_int
                        ^ 0xf as ::core::ffi::c_int) as uint16;
                } else {
                    v6 = *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x1a4b as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v2 as isize);
                };
                v7 = (temp_collision_DD6_0 as ::core::ffi::c_int
                    + (v6 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int))
                    as uint16;
                v8 = ((*(game.g_rom.offset(
                    (((0x948b2b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x948b2b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint8)
                    .offset(v7 as isize) as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int)
                    - temp_collision_DD4_0 as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as int16;
                (*(game.g_rom.offset(
                    (((0x948b2b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x948b2b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint8)
                    .offset(v7 as isize) as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int)
                    - temp_collision_DD4_0 as ::core::ffi::c_int
                    == 1 as ::core::ffi::c_int
                    || (v8 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int
            } {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v2 as isize) =
                    ((*eci).eci_r24 as ::core::ffi::c_int + v8 as ::core::ffi::c_int) as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v2 as isize) = -(1 as ::core::ffi::c_int) as uint16;
                return 1 as uint8;
            } else {
                return 0 as uint8;
            }
        } else {
            return 0 as uint8;
        }
    };
}
static kEprojBlockCollisition_FuncA: [Option<Func_EprojCollInfo_U8>; 16] = unsafe {
    [
        Some(EprojColl_ClearCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_858E as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_ClearCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_ClearCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_ClearCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_8506 as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_ClearCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_ClearCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_SetCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_SetCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_SetCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_SetCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_SetCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_Unknown8536 as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_ClearCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_SetCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
    ]
};
static kEprojBlockCollisition_FuncB: [Option<Func_EprojCollInfo_U8>; 16] = unsafe {
    [
        Some(EprojColl_ClearCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_85AD as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_ClearCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_ClearCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_ClearCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_8506 as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_ClearCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_ClearCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_SetCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_SetCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_SetCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_SetCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_SetCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_Unknown8536 as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_ClearCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
        Some(EprojColl_SetCarry as unsafe fn(*mut EprojCollInfo, &mut Game) -> uint8),
    ]
};
pub unsafe fn EprojBlockCollisition_CheckHorizontal(
    mut eci: *mut EprojCollInfo,
    mut k: uint16,
    game: &mut Game,
) -> uint8 {
    let mut rv: uint8 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
        as *mut uint16) = (k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    loop {
        rv = kEprojBlockCollisition_FuncA[((*((&raw mut game.g_ram as *mut uint8)
            .offset(0x10002 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int)
            >> 12 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(eci, game);
        if !(rv as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0) {
            break;
        }
    }
    return rv;
}
pub unsafe fn EprojBlockCollisition_CheckVertical(
    mut eci: *mut EprojCollInfo,
    mut k: uint16,
    game: &mut Game,
) -> uint8 {
    let mut rv: uint8 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
        as *mut uint16) = (k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    loop {
        rv = kEprojBlockCollisition_FuncB[((*((&raw mut game.g_ram as *mut uint8)
            .offset(0x10002 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xdc4 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int)
            >> 12 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(eci, game);
        if !(rv as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0) {
            break;
        }
    }
    return rv;
}
pub unsafe fn EprojBlockCollisition_Horiz(mut k: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1ab7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let mut R30: uint16 = *(((&raw mut game.g_ram as *mut uint8)
        .offset(0x1bb3 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as uint16;
    let mut R28: uint16 = *(((&raw mut game.g_ram as *mut uint8)
        .offset(0x1bb3 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as uint16;
    let mut r26: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - R30 as ::core::ffi::c_int
        & 0xfff0 as ::core::ffi::c_int) as uint16;
    r26 = ((R30 as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int
        - r26 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    let mut R32: uint16 = r26;
    let mut prod: uint16 = Mult8x8(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - R30 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    let mut v3: uint16 = ((amt as uint32).wrapping_add(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as uint32,
    ) >> 16 as ::core::ffi::c_int) as uint16;
    let mut r22: uint16 = (amt
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int32) as uint16;
    let mut r24: uint16 = v3;
    let mut v4: uint16 = 0;
    if amt < 0 as int32 {
        v4 = (v3 as ::core::ffi::c_int - R28 as ::core::ffi::c_int) as uint16;
    } else {
        v4 = (R28 as ::core::ffi::c_int + v3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
            as uint16;
    }
    let mut eci: EprojCollInfo = EprojCollInfo {
        eci_r20: (amt >> 16 as ::core::ffi::c_int) as uint16,
        eci_r24: r24,
        eci_r26: r26,
        eci_r28: R28,
        eci_r30: R30,
        eci_r32: R32,
        eci_r34: v4,
    };
    let mut v5: uint16 = (2 as ::core::ffi::c_int
        * (prod as ::core::ffi::c_int + (v4 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int)))
        as uint16;
    while EprojBlockCollisition_CheckHorizontal(&raw mut eci, v5, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        v5 = (v5 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                * 2 as ::core::ffi::c_int) as uint16;
        eci.eci_r26 = eci.eci_r26.wrapping_sub(1);
        if eci.eci_r26 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let mut v6: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v6 as isize) = r22;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v6 as isize) = r24;
            return 0 as uint8;
        }
    }
    let mut v8: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v8 as isize) = 0 as uint16;
    if amt < 0 as int32 {
        let mut v10: uint16 = (eci.eci_r28 as ::core::ffi::c_int
            + (eci.eci_r34 as ::core::ffi::c_int | 0xf as ::core::ffi::c_int)
            + 1 as ::core::ffi::c_int) as uint16;
        if v10 as ::core::ffi::c_int
            <= *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v8 as isize) as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v8 as isize) = v10;
        }
    } else {
        let mut v9: uint16 = ((eci.eci_r34 as ::core::ffi::c_int & 0xfff0 as ::core::ffi::c_int)
            - eci.eci_r28 as ::core::ffi::c_int) as uint16;
        if v9 as ::core::ffi::c_int
            >= *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v8 as isize) as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v8 as isize) = v9;
        }
    }
    return 1 as uint8;
}
pub unsafe fn EprojBlockCollisition_Vertical(mut k: uint16, game: &mut Game) -> uint8 {
    let mut v5: int16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let mut R30: uint16 = *(((&raw mut game.g_ram as *mut uint8)
        .offset(0x1bb3 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as uint16;
    let mut R28: uint16 = *(((&raw mut game.g_ram as *mut uint8)
        .offset(0x1bb3 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as uint16;
    let mut r26: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - R28 as ::core::ffi::c_int
        & 0xfff0 as ::core::ffi::c_int) as uint16;
    r26 = ((R28 as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int
        - r26 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    let mut R32: uint16 = r26;
    let mut v3: uint16 = ((amt as uint32).wrapping_add(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as uint32,
    ) >> 16 as ::core::ffi::c_int) as uint16;
    let mut r22: uint16 = (amt
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int32) as uint16;
    let mut r24: uint16 = v3;
    let mut v4: uint16 = 0;
    if amt < 0 as int32 {
        v4 = (v3 as ::core::ffi::c_int - R30 as ::core::ffi::c_int) as uint16;
    } else {
        v4 = (R30 as ::core::ffi::c_int + v3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
            as uint16;
    }
    let mut eci: EprojCollInfo = EprojCollInfo {
        eci_r20: (amt >> 16 as ::core::ffi::c_int) as uint16,
        eci_r24: r24,
        eci_r26: r26,
        eci_r28: R28,
        eci_r30: R30,
        eci_r32: R32,
        eci_r34: v4,
    };
    let mut prod: uint16 = Mult8x8(
        (v4 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8,
    );
    v5 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - R28 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as int16;
    let mut i: ::core::ffi::c_int =
        2 as ::core::ffi::c_int * (prod as ::core::ffi::c_int + v5 as ::core::ffi::c_int);
    while EprojBlockCollisition_CheckVertical(&raw mut eci, i as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        eci.eci_r26 = eci.eci_r26.wrapping_sub(1);
        if eci.eci_r26 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v7 as isize) = r22;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v7 as isize) = r24;
            return 0 as uint8;
        }
        i += 2 as ::core::ffi::c_int;
    }
    let mut v9: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v9 as isize) = 0 as uint16;
    if amt < 0 as int32 {
        let mut v11: uint16 = (eci.eci_r30 as ::core::ffi::c_int
            + (eci.eci_r34 as ::core::ffi::c_int | 0xf as ::core::ffi::c_int)
            + 1 as ::core::ffi::c_int) as uint16;
        if v11 as ::core::ffi::c_int
            <= *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v9 as isize) as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v9 as isize) = v11;
        }
    } else {
        let mut v10: uint16 = ((eci.eci_r34 as ::core::ffi::c_int & 0xfff0 as ::core::ffi::c_int)
            - eci.eci_r30 as ::core::ffi::c_int) as uint16;
        if v10 as ::core::ffi::c_int
            >= *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v9 as isize) as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v9 as isize) = v10;
        }
    }
    return 1 as uint8;
}
unsafe fn Eproj_Init_0x8aaf(mut j: uint16, game: &mut Game) {
    NextRandom(game);
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: uint16 = ((*v1).y_pos as ::core::ffi::c_int + 12 as ::core::ffi::c_int) as uint16;
    let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = v2;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19df as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (v2 as ::core::ffi::c_int + 72 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (*v1).ai_var_D;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = ((*v1).x_pos as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
        - 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = *(game.g_rom.offset(
        (((0x868a75 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x868a75 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((*(((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int
                & 6 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
}
pub unsafe fn Eproj_PreInit_0x8aaf(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        >= *((&raw mut game.g_ram as *mut uint8).offset(0x19df as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
    }
}
unsafe fn Eproj_Init_0x8bc2_SkreeDownRight(mut j: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v1).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(769 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v1).x_pos as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 320 as uint16;
}
unsafe fn Eproj_Init_0x8bd0_SkreeUpRight(mut j: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v1).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(1025 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v1).x_pos as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 96 as uint16;
}
unsafe fn Eproj_Init_0x8bde_SkreeDownLeft(mut j: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v1).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(769 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v1).x_pos as ::core::ffi::c_int - 6 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(320 as ::core::ffi::c_int) as uint16;
}
unsafe fn Eproj_Init_0x8bec_SkreeUpLeft(mut j: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v1).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(1025 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v1).x_pos as ::core::ffi::c_int - 6 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(96 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Eproj_PreInstr_SkreeParticle(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
    let ref mut fresh145 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh145 = (*fresh145 as ::core::ffi::c_int + 80 as ::core::ffi::c_int) as uint16;
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojInstr_SpawnEnemyDropsWithDraygonsEyeDrops(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
    };
    SpawnEnemyDrops(
        addr_kEnemyDef_DE7F as ::core::ffi::c_int as uint16,
        k,
        0 as uint16,
        game,
    );
    return epjp;
}
pub unsafe fn EprojInstr_SetPreInstrA(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (0x868dff as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return epjp;
}
pub unsafe fn EprojInstr_SetPreInstrB(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (0x868dca as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return epjp;
}
unsafe fn EprojInit_DraygonsGunk(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = game.eproj_spawn_pt.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = game.eproj_spawn_pt.y;
    *((&raw mut game.g_ram as *mut uint8).offset(0x97dc as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1995 as ::core::ffi::c_int as isize)
        as *mut uint16);
    let mut pt: Point32 = ConvertAngleToXy(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1995 as ::core::ffi::c_int as isize)
            as *mut uint16),
        *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (pt.x >> 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = pt.x as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (pt.y >> 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = pt.y as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 1024 as uint16;
}
unsafe fn EprojInit_DraygonsWallTurretProjs(mut j: uint16, game: &mut Game) {
    Eproj_AngleToSamus(j, game.eproj_spawn_pt.x, game.eproj_spawn_pt.y, game);
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 2560 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) =
        (0x868d54 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojPowerBombCollision(mut k: uint16, game: &mut Game) {
    let mut r18: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xcea as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        )
        != 0
    {
        let mut r20: uint16 = ((r18 as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int)
            + (*((&raw mut game.g_ram as *mut uint8).offset(0xcea as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >> 9 as ::core::ffi::c_int)) as uint16
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16;
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if (abs16(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xce2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1a4b as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int)
            < r18 as ::core::ffi::c_int
            && (abs16(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xce4 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x1a93 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int) as uint16,
            ) as ::core::ffi::c_int)
                < r20 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
    }
}
pub unsafe fn EprojInstr_868D99(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    EprojPowerBombCollision(k, game);
    let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xa66 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 5 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh33 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa66 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh33 = (*fresh33).wrapping_add(1);
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = v2;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = 256 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) =
            (0x868dca as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1bd7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) as ::core::ffi::c_int
            & 0x5fff as ::core::ffi::c_int
            | 0x2000 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18a8 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18aa as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    return epjp;
}
pub unsafe fn EprojPreInstr_8DCA(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = 0;
    EprojPowerBombCollision(k, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
        || {
            v1 = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
                .offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 4 as ::core::ffi::c_int
                    * *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x1b23 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int
                - 12 as ::core::ffi::c_int) as uint16;
            let ref mut fresh143 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x1aff as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh143 = (*fresh143).wrapping_sub(1);
            *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize)
                == 0
        }
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        let ref mut fresh144 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xa66 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh144 = (*fresh144).wrapping_sub(1);
        if *fresh144 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
    }
}
pub unsafe fn EprojPreInstr_DraygonsTurret_8DFF(mut k: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    EprojPowerBombCollision(k, game);
    Eproj_FuncE73E_MoveXY(k, game);
    v1 = Eproj_FuncE722(k, game) as int16;
    if v1 != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojPreInstr_DraygonsGunk_8E0F(mut k: uint16, game: &mut Game) {
    let mut v3: uint16 = 0;
    EprojPowerBombCollision(k, game);
    Eproj_FuncE73E_MoveXY(k, game);
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = abs16(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16,
    );
    if v2 as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 && {
        v3 = abs16(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1a93 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int) as uint16,
        );
        v3 as ::core::ffi::c_int - 20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    } {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0x8c38 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    } else if Eproj_FuncE722(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
    }
}
unsafe fn EprojInit_CrocomireProjectile(mut j: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(512 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 1 as uint16;
    let mut v3: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v3).x_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v3).y_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19df as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 2560 as uint16;
}
pub unsafe fn EprojPreInstr_CrocomireProjectile(mut k: uint16, game: &mut Game) {
    EprojBlockCollisition_Horiz(k, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 2560 as uint16;
    let ref mut fresh142 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x19df as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh142 = (*fresh142 as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int) as uint16;
    let mut x: uint16 = -(64 as ::core::ffi::c_int) as uint16;
    let mut y: uint16 = *(game.g_rom.offset(
        (((0x869059 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x869059 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .ai_preinstr as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    let mut v1: ::core::ffi::c_int = CalculateAngleFromXY(x, y, game) as ::core::ffi::c_int;
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (4 as ::core::ffi::c_int
        * kSinCosTable8bit_Sext[(v1 + 64 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int)
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (4 as ::core::ffi::c_int
        * kSinCosTable8bit_Sext[v1 as usize] as ::core::ffi::c_int)
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        (0x8690b3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn sub_8690B3(mut k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
unsafe fn EprojInit_CrocomireSpikeWallPieces(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *(game.g_rom.offset(
        (((0x869105 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x869105 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((j as ::core::ffi::c_int - 20 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 528 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = -(5 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = -(30720 as ::core::ffi::c_int) as uint16;
}
static CrocomireSpikeWallPieces_Tab2: [uint16; 18] = [
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0xff0 as ::core::ffi::c_int as uint16,
    0xee0 as ::core::ffi::c_int as uint16,
    0xcc0 as ::core::ffi::c_int as uint16,
    0xaa0 as ::core::ffi::c_int as uint16,
    0x880 as ::core::ffi::c_int as uint16,
    0x660 as ::core::ffi::c_int as uint16,
    0x440 as ::core::ffi::c_int as uint16,
    0x220 as ::core::ffi::c_int as uint16,
    0xff0 as ::core::ffi::c_int as uint16,
    0xee0 as ::core::ffi::c_int as uint16,
    0xcc0 as ::core::ffi::c_int as uint16,
    0xaa0 as ::core::ffi::c_int as uint16,
    0x880 as ::core::ffi::c_int as uint16,
    0x660 as ::core::ffi::c_int as uint16,
    0x440 as ::core::ffi::c_int as uint16,
    0x220 as ::core::ffi::c_int as uint16,
];
static CrocomireSpikeWallPieces_Tab1: [uint16; 18] = [
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0xff00 as ::core::ffi::c_int as uint16,
    0xee00 as ::core::ffi::c_int as uint16,
    0xcc00 as ::core::ffi::c_int as uint16,
    0xaa00 as ::core::ffi::c_int as uint16,
    0x8800 as ::core::ffi::c_int as uint16,
    0x6600 as ::core::ffi::c_int as uint16,
    0x4400 as ::core::ffi::c_int as uint16,
    0x2200 as ::core::ffi::c_int as uint16,
    0xff00 as ::core::ffi::c_int as uint16,
    0xee00 as ::core::ffi::c_int as uint16,
    0xcc00 as ::core::ffi::c_int as uint16,
    0xaa00 as ::core::ffi::c_int as uint16,
    0x8800 as ::core::ffi::c_int as uint16,
    0x6600 as ::core::ffi::c_int as uint16,
    0x4400 as ::core::ffi::c_int as uint16,
    0x2200 as ::core::ffi::c_int as uint16,
];
static CrocomireSpikeWallPieces_Tab3: [int8; 36] = [
    0 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    4 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    4 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    3 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    3 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    2 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    2 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    1 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    1 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    6 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    5 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    4 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    3 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    2 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    2 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    1 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
    1 as ::core::ffi::c_int as int8,
    0 as ::core::ffi::c_int as int8,
];
pub unsafe fn EprojPreInstr_CrocomireSpikeWallPieces(mut k: uint16, game: &mut Game) {
    let mut v3: int8 = 0;
    let mut v6: int8 = 0;
    let mut v7: int8 = 0;
    let mut v8: int8 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1aff as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    if v2 as ::core::ffi::c_int != CrocomireSpikeWallPieces_Tab1[v1 as usize] as ::core::ffi::c_int
    {
        v2 = (v2 as ::core::ffi::c_int
            + CrocomireSpikeWallPieces_Tab2[v1 as usize] as ::core::ffi::c_int)
            as uint16;
        if v2 as ::core::ffi::c_int
            >= CrocomireSpikeWallPieces_Tab1[v1 as usize] as ::core::ffi::c_int
        {
            v2 = CrocomireSpikeWallPieces_Tab1[v1 as usize];
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = v2;
    v3 = *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) as int8;
    let mut v4: bool = __CFADD__uint8(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize),
        v3 as uint8,
    );
    let ref mut fresh138 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1ab7 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize);
    *fresh138 = (*fresh138 as ::core::ffi::c_int + v3 as ::core::ffi::c_int) as uint8;
    let mut v5: uint8 = (v4 as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        as uint8;
    if (v5 as ::core::ffi::c_int - CrocomireSpikeWallPieces_Tab3[k as usize] as ::core::ffi::c_int)
        as int8 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        v5 = CrocomireSpikeWallPieces_Tab3[k as usize] as uint8;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = v5;
    v6 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as int8;
    v4 = __CFADD__uint8(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize),
        v6 as uint8,
    );
    let ref mut fresh139 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a27 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize);
    *fresh139 = (*fresh139 as ::core::ffi::c_int + v6 as ::core::ffi::c_int) as uint8;
    v7 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) as int8;
    let mut v9: bool = v4;
    v4 = __CFADD__uint8(v4 as uint8, v7 as uint8);
    v8 = (v9 as ::core::ffi::c_int + v7 as ::core::ffi::c_int) as int8;
    v4 = (v4 as ::core::ffi::c_int
        | __CFADD__uint8(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize),
            v8 as uint8,
        ) as ::core::ffi::c_int)
        != 0;
    let ref mut fresh140 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize);
    *fresh140 = (*fresh140 as ::core::ffi::c_int + v8 as ::core::ffi::c_int) as uint8;
    let ref mut fresh141 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize);
    *fresh141 = (*fresh141 as ::core::ffi::c_int + v4 as ::core::ffi::c_int) as uint8;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        0x3000 as uint32,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        >= 0xa8 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        if k as ::core::ffi::c_int & 2 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            QueueSfx2_Max6(0x29 as uint16, game);
        }
        let mut v12: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        game.eproj_spawn_pt = Point16U {
            x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v12 as isize),
            y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v12 as isize),
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
            0x15 as uint16,
            game,
        );
        QueueSfx2_Max6(0x25 as uint16, game);
    }
}
pub unsafe fn EprojInstr_9270(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
    };
    SpawnEnemyDrops(
        addr_kEnemyDef_DDBF as ::core::ffi::c_int as uint16,
        k,
        0 as uint16,
        game,
    );
    return epjp;
}
unsafe fn EprojInit_CrocomireBridgeCrumbling(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 187 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x3f as ::core::ffi::c_int)
        + 64 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 1024 as uint16;
}
pub unsafe fn EprojPreInstr_CrocomireBridgeCrumbling(mut k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                + 24 as ::core::ffi::c_int
                & 0x3fff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn MoveEprojWithVelocity(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1ab7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    let mut carry: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + (v2 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = carry as uint8;
    let ref mut fresh7 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh7 = (*fresh7 as ::core::ffi::c_int
        + ((v2 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
            + (carry >> 8 as ::core::ffi::c_int))) as uint16;
    return MoveEprojWithVelocityY(k, game);
}
pub unsafe fn MoveEprojWithVelocityY(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    let mut carry: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + (v2 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = carry as uint8;
    let mut result: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + (v2 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
        + (carry >> 8 as ::core::ffi::c_int)) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = result;
    return result;
}
pub unsafe fn MoveEprojWithVelocityX(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1ab7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    let mut carry: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + (v2 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) = carry as uint8;
    let mut result: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + (v2 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
        + (carry >> 8 as ::core::ffi::c_int)) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = result;
    return result;
}
pub unsafe fn SetAreaDependentEprojPropertiesEx(mut k: uint16, mut j: uint16, game: &mut Game) {
    let mut v2: uint16 = 0;
    let mut p: *mut uint16 = RomPtr_86(k as uint16_t, game) as *mut uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 2 as ::core::ffi::c_int
    {
        v2 = *p.offset(1 as ::core::ffi::c_int as isize);
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 5 as ::core::ffi::c_int
    {
        v2 = *p.offset(2 as ::core::ffi::c_int as isize);
    } else {
        v2 = *p.offset(0 as ::core::ffi::c_int as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v2;
}
static kEprojInit_9634_Xvel: [int16; 4] = [
    -(0x200 as ::core::ffi::c_int) as int16,
    -(0x1f0 as ::core::ffi::c_int) as int16,
    -(0x1bc as ::core::ffi::c_int) as int16,
    -(0x16a as ::core::ffi::c_int) as int16,
];
static kEprojInit_9634_Yvel: [int16; 4] = [
    0 as ::core::ffi::c_int as int16,
    0x88 as ::core::ffi::c_int as int16,
    0xfc as ::core::ffi::c_int as int16,
    0x16a as ::core::ffi::c_int as int16,
];
unsafe fn EprojInit_9634(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .x_pos as ::core::ffi::c_int
        - 29 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .y_pos as ::core::ffi::c_int
        - 35 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 2560 as uint16;
    let mut v2: ::core::ffi::c_int = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .parameter_1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = kEprojInit_9634_Xvel[v2 as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = kEprojInit_9634_Yvel[v2 as usize] as uint16;
}
pub unsafe fn EprojPreInstr_9634(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        >= 8 as ::core::ffi::c_int
    {
        MoveEprojWithVelocity(k, game);
        if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = addr_off_869574 as ::core::ffi::c_int as uint16;
            let ref mut fresh136 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x1aff as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh136 = (*fresh136).wrapping_add(1);
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 2560 as uint16;
            QueueSfx2_Max6(0x2b as uint16, game);
        }
    } else {
        let ref mut fresh137 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh137 = (*fresh137).wrapping_add(1);
    };
}
unsafe fn EprojInit_9642_RidleysFireball(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1995 as ::core::ffi::c_int as isize)
        as *mut uint16);
    let mut v2: int16 = (if *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0
    {
        25 as ::core::ffi::c_int
    } else {
        -(25 as ::core::ffi::c_int)
    }) as int16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .x_pos as ::core::ffi::c_int
        + v2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .y_pos as ::core::ffi::c_int
        - 43 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 2560 as uint16;
    let mut v4: *mut Ram7800_Default = gRam7800_Default(0 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*v4).var_19;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*v4).var_1A;
    SetAreaDependentEprojPropertiesEx(
        addr_kRidleysFireball_Tab0 as ::core::ffi::c_int as uint16,
        j,
        game,
    );
}
pub unsafe fn SetAreaDependentEprojProperties(mut j: uint16, game: &mut Game) {
    SetAreaDependentEprojPropertiesEx(
        addr_kRidleysFireball_Tab0 as ::core::ffi::c_int as uint16,
        j,
        game,
    );
}
pub unsafe fn EprojPreInstr_9642_RidleysFireball(mut k: uint16, game: &mut Game) {
    let mut v1: uint16 = 0;
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        v1 = -(27042 as ::core::ffi::c_int) as uint16;
    } else {
        if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            == 0
        {
            return;
        }
        v1 = -(27056 as ::core::ffi::c_int) as uint16;
    }
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize)
        == 0
    {
        game.eproj_spawn_pt = Point16U {
            x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize),
            y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize),
        };
        SpawnEprojWithRoomGfx(v1, 3 as uint16, game);
        QueueSfx2_Max6(0x2b as uint16, game);
    }
}
pub unsafe fn EprojInstr_DisableCollisionsWithSamus(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh32 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1bd7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh32 = (*fresh32 as ::core::ffi::c_int | 0x2000 as ::core::ffi::c_int) as uint16;
    return epjp;
}
unsafe fn EprojInit_9660_FireballExplosion(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 2560 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = game.eproj_spawn_pt.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = game.eproj_spawn_pt.y;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
}
unsafe fn EprojInit_Common(mut j: uint16, mut k: uint16, mut a: uint16, game: &mut Game) {
    let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = a;
    let mut v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = 2560 as uint16;
    SetAreaDependentEprojProperties(j, game);
}
unsafe fn EprojInit_9688(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(3584 as ::core::ffi::c_int) as uint16;
    EprojInit_Common(j, v1, 0x9688 as uint16, game);
}
unsafe fn EprojInit_9696(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 3584 as uint16;
    EprojInit_Common(j, v1, 0x9696 as uint16, game);
}
unsafe fn EprojInit_966C(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 3584 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    EprojInit_Common(j, v1, 0x966c as uint16, game);
}
unsafe fn EprojInit_967A(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(3584 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    EprojInit_Common(j, v1, 0x967a as uint16, game);
}
pub unsafe fn EprojPreInstr_966C(mut k: uint16, game: &mut Game) {
    MoveEprojWithVelocityX(k, game);
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_off_869574 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    }
}
pub unsafe fn EprojPreInstr_9688(mut k: uint16, game: &mut Game) {
    MoveEprojWithVelocityY(k, game);
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_off_869574 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    }
}
pub unsafe fn EprojPreInstr_96A4(mut k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojPreInstr_96C0(mut k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojPreInstr_96CE(mut k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojInstr_95BA(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    SpawnEprojWithRoomGfx(addr_stru_86966C as ::core::ffi::c_int as uint16, k, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    SpawnEprojWithRoomGfx(addr_stru_86967A as ::core::ffi::c_int as uint16, k, game);
    return epjp;
}
pub unsafe fn EprojInstr_95ED(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    SpawnEprojWithRoomGfx(addr_stru_869688 as ::core::ffi::c_int as uint16, k, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    SpawnEprojWithRoomGfx(addr_stru_869696 as ::core::ffi::c_int as uint16, k, game);
    return epjp;
}
pub unsafe fn EprojInstr_9620(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh31 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1aff as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize);
    *fresh31 = (*fresh31).wrapping_sub(1);
    if *fresh31 as int8 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        SpawnEprojWithRoomGfx(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
            k,
            game,
        );
    }
    return epjp;
}
unsafe fn EprojInit_9734_CeresFallingDebris(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 3584 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 42 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 16 as uint16;
}
pub unsafe fn EprojPreInstr_9734_CeresFallingDebris(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh135 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh135 = (*fresh135 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        game.eproj_spawn_pt = Point16U {
            x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize),
            y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize),
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
            9 as uint16,
            game,
        );
        QueueSfx2_Max6(0x6d as uint16, game);
    }
}
pub unsafe fn EprojInstr_980E(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize),
    };
    SpawnEnemyDrops(
        addr_kEnemyDef_E4FF as ::core::ffi::c_int as uint16,
        k,
        0 as uint16,
        game,
    );
    return epjp;
}
static byte_8698B4: [uint8; 16] = [
    0x10 as ::core::ffi::c_int as uint8,
    0x20 as ::core::ffi::c_int as uint8,
    0x30 as ::core::ffi::c_int as uint8,
    0x40 as ::core::ffi::c_int as uint8,
    0x50 as ::core::ffi::c_int as uint8,
    0x60 as ::core::ffi::c_int as uint8,
    0x70 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0xf0 as ::core::ffi::c_int as uint8,
    0xe0 as ::core::ffi::c_int as uint8,
    0xd0 as ::core::ffi::c_int as uint8,
    0xc0 as ::core::ffi::c_int as uint8,
    0xb0 as ::core::ffi::c_int as uint8,
    0xa0 as ::core::ffi::c_int as uint8,
    0x90 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
];
static byte_8698F7: [uint8; 9] = [
    0x30 as ::core::ffi::c_int as uint8,
    0x44 as ::core::ffi::c_int as uint8,
    0x58 as ::core::ffi::c_int as uint8,
    0x6c as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0x94 as ::core::ffi::c_int as uint8,
    0xa8 as ::core::ffi::c_int as uint8,
    0xbc as ::core::ffi::c_int as uint8,
    0xd0 as ::core::ffi::c_int as uint8,
];
static byte_869979: [uint8; 8] = [
    0 as ::core::ffi::c_int as uint8,
    0x20 as ::core::ffi::c_int as uint8,
    0x40 as ::core::ffi::c_int as uint8,
    0x60 as ::core::ffi::c_int as uint8,
    0x80 as ::core::ffi::c_int as uint8,
    0xa0 as ::core::ffi::c_int as uint8,
    0xc0 as ::core::ffi::c_int as uint8,
    0xe0 as ::core::ffi::c_int as uint8,
];
unsafe fn EprojInit_PhantoonDestroyableFireballs(mut j: uint16, game: &mut Game) {
    let mut v1: int8 = 0;
    v1 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as int8;
    match v1 as ::core::ffi::c_int {
        0 => {
            let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .x_pos;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                .offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .y_pos as ::core::ffi::c_int
                + 32 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = addr_off_8697B4 as ::core::ffi::c_int as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x1bd7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int
                & 0xfff as ::core::ffi::c_int
                | 0x2000 as ::core::ffi::c_int) as uint16;
        }
        2 => {
            let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) = 0 as uint16;
            let mut v4: uint16 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x1993 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8 as uint16;
            if (*((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8 as ::core::ffi::c_int
                - 8 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) = -(2 as ::core::ffi::c_int) as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) = 2 as uint16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) = byte_8698B4[v4 as usize] as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .x_pos;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                .offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .y_pos as ::core::ffi::c_int
                + 32 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) =
                (0x869a45 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        4 => {
            let mut v5: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v5 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v5 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v5 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v5 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                .offset(0x1993 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xf0 as ::core::ffi::c_int)
                >> 1 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v5 as isize) =
                byte_8698F7[(*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1993 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as usize] as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v5 as isize) = 40 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v5 as isize) =
                (0x869a94 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        6 => {
            let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v7 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v7 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v7 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v7 as isize) = 128 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v7 as isize) = byte_869979[*((&raw mut game.g_ram as *mut uint8)
                .offset(0x1993 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8
                as usize] as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v7 as isize) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .x_pos;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v7 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                .offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .y_pos as ::core::ffi::c_int
                + 16 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v7 as isize) =
                (0x869ada as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        _ => {
            Unreachable(game);
        }
    };
}
unsafe fn EprojInit_PhantoonStartingFireballs(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    let mut v2: uint16 = byte_869979[*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = v2;
    let mut pt: Point16U = Eproj_PhantomFireballs_Func1(v2, 0x30 as uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (pt.x as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .x_pos as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (pt.y as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .y_pos as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojPreInstr_PhantoonStartingFireballs(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh133 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh133 = (*fresh133 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1bd7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0xfff as ::core::ffi::c_int
            | 0x8000 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) =
            (0x8699bf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_word_86976C as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 8 as uint16;
        let ref mut fresh134 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh134 = (*fresh134 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn EprojPreInstr_PhantoonStartingFireballs2(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: bool = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1aff as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int;
    let ref mut fresh131 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1aff as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh131 = (*fresh131).wrapping_sub(1);
    let mut v3: bool =
        *fresh131 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) =
            (0x869a01 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_word_869772 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
        let ref mut fresh132 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh132 = (*fresh132 as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = -(768 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = -(128 as ::core::ffi::c_int) as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 128 as uint16;
        }
    }
}
static word_869A3E: [uint16; 3] = [
    0xfd00 as ::core::ffi::c_int as uint16,
    0xfe00 as ::core::ffi::c_int as uint16,
    0xff00 as ::core::ffi::c_int as uint16,
];
pub unsafe fn EprojPreInstr_PhantoonStartingFireballs3(mut k: uint16, game: &mut Game) {
    let mut current_block: u64;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh129 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh129 = (*fresh129 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v2;
        if v2 as ::core::ffi::c_int - 3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = word_869A3E[v2 as usize];
            return;
        }
        current_block = 220575731145207134;
    } else if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0
    {
        current_block = 220575731145207134;
    } else {
        current_block = 13183875560443969876;
    }
    match current_block {
        220575731145207134 => {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = addr_word_869782 as ::core::ffi::c_int as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) =
                (0x869a44 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        _ => {}
    };
}
pub unsafe fn EprojPreInstr_PhantoonDestroyableFireballs(mut k: uint16, game: &mut Game) {
    let mut v4: int16 = 0;
    let mut v5: uint16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh128 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh128 = (*fresh128 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = (*(((&raw mut game.g_ram as *mut uint8)
        .offset(0x1ab7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as uint8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = v2;
    let mut pt: Point16U = Eproj_PhantomFireballs_Func1(
        v2,
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
    );
    let mut v3: bool = ((pt.x as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .x_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int;
    v4 = (pt.x as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .x_pos as ::core::ffi::c_int) as int16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (pt.x as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .x_pos as ::core::ffi::c_int) as uint16;
    if v3 as ::core::ffi::c_int != 0
        || v4 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        || {
            v3 = ((pt.y as ::core::ffi::c_int
                + (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0xf78 as ::core::ffi::c_int as isize)
                    as *mut EnemyData)
                    .offset(0 as ::core::ffi::c_int as isize))
                .y_pos as ::core::ffi::c_int
                + 16 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int;
            v5 = (pt.y as ::core::ffi::c_int
                + (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0xf78 as ::core::ffi::c_int as isize)
                    as *mut EnemyData)
                    .offset(0 as ::core::ffi::c_int as isize))
                .y_pos as ::core::ffi::c_int
                + 16 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = v5;
            v3 as ::core::ffi::c_int != 0
        }
        || v5 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_off_8697F8 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    }
}
pub unsafe fn EprojPreInstr_PhantoonDestroyableFireballs_2(mut k: uint16, game: &mut Game) {
    let mut v2: bool = false;
    let mut v3: bool = false;
    let mut current_block: u64;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize)
        == 0
    {
        current_block = 8066597768905169940;
    } else {
        v2 = false;
        v3 = false;
        v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int;
        let ref mut fresh125 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh125 = (*fresh125).wrapping_sub(1);
        v3 = *fresh125 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
        if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
            QueueSfx3_Max6(0x1d as uint16, game);
            current_block = 8066597768905169940;
        } else {
            current_block = 2868539653012386629;
        }
    }
    match current_block {
        8066597768905169940 => {
            let ref mut fresh126 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh126 = (*fresh126 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
            if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                != 0
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) = addr_word_8697AC as ::core::ffi::c_int as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) = 1 as uint16;
                let ref mut fresh127 = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1a93 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize);
                *fresh127 = (*fresh127 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) =
                    (0x869a44 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                QueueSfx3_Max6(0x1d as uint16, game);
            }
        }
        _ => {}
    };
}
pub unsafe fn EprojPreInstr_PhantoonDestroyableFireballs_3(mut k: uint16, game: &mut Game) {
    let mut v4: int16 = 0;
    let mut v5: uint16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh123 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh123 = (*fresh123 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1aff as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int) as uint8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = v2;
    let mut pt: Point16U = Eproj_PhantomFireballs_Func1(
        v2,
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
    );
    let mut v3: bool = ((pt.x as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .x_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int;
    v4 = (pt.x as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .x_pos as ::core::ffi::c_int) as int16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (pt.x as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .x_pos as ::core::ffi::c_int) as uint16;
    if v3 as ::core::ffi::c_int != 0
        || v4 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        || {
            v3 = ((pt.y as ::core::ffi::c_int
                + (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0xf78 as ::core::ffi::c_int as isize)
                    as *mut EnemyData)
                    .offset(0 as ::core::ffi::c_int as isize))
                .y_pos as ::core::ffi::c_int
                + 16 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int;
            v5 = (pt.y as ::core::ffi::c_int
                + (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0xf78 as ::core::ffi::c_int as isize)
                    as *mut EnemyData)
                    .offset(0 as ::core::ffi::c_int as isize))
                .y_pos as ::core::ffi::c_int
                + 16 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = v5;
            v3 as ::core::ffi::c_int != 0
        }
        || v5 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_off_8697F8 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    }
}
pub unsafe fn EprojPreInstr_PhantoonStartingFireballsB(mut k: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .ai_var_B
        != 0
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) =
            (0x869b41 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 180 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 48 as uint16;
    }
}
pub unsafe fn EprojPreInstr_PhantoonStartingFireballsB_2(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1ab7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    if v2 != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (v2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    } else if !(*((&raw mut game.g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int)
    {
        let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v3;
        if !(v3 != 0) {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .x_pos;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                .offset(0xf78 as ::core::ffi::c_int as isize)
                as *mut EnemyData)
                .offset(0 as ::core::ffi::c_int as isize))
            .y_pos as ::core::ffi::c_int
                + 16 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = addr_off_8697F8 as ::core::ffi::c_int as uint16;
            return;
        }
    }
    let mut v4: uint8 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1aff as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = v4 as uint16;
    let mut pt: Point16U = Eproj_PhantomFireballs_Func1(
        v4 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (pt.x as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .x_pos as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (pt.y as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .y_pos as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Eproj_PhantomFireballs_Func1(mut j: uint16, mut a: uint16) -> Point16U {
    let mut v3: int16 = 0;
    let mut v2: uint16 = 0;
    let mut v4: uint16 = 0;
    let mut r24: uint16 = a;
    let mut r26: uint16 = j;
    if j as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v2 = Eproj_PhantomFireballs_Func2(
            (2 as ::core::ffi::c_int * j as ::core::ffi::c_int) as uint16,
            r24,
        );
    } else {
        v2 = -(Eproj_PhantomFireballs_Func2(
            (2 as ::core::ffi::c_int
                * (j as ::core::ffi::c_int + 0x80 as ::core::ffi::c_int) as uint8
                    as ::core::ffi::c_int) as uint16,
            r24,
        ) as ::core::ffi::c_int) as uint16;
    }
    let mut r20: uint16 = v2;
    v3 = (r26 as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as uint8 as int16;
    if v3 as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v4 = Eproj_PhantomFireballs_Func2(
            (2 as ::core::ffi::c_int * v3 as ::core::ffi::c_int) as uint16,
            r24,
        );
    } else {
        v4 = -(Eproj_PhantomFireballs_Func2(
            (2 as ::core::ffi::c_int
                * (v3 as ::core::ffi::c_int + 0x80 as ::core::ffi::c_int) as uint8
                    as ::core::ffi::c_int) as uint16,
            r24,
        ) as ::core::ffi::c_int) as uint16;
    }
    let mut r22: uint16 = v4;
    return Point16U { x: r20, y: r22 };
}
pub unsafe fn Eproj_PhantomFireballs_Func2(mut k: uint16, mut r24: uint16) -> uint16 {
    let mut r18: uint16 = (Mult8x8(
        *((&raw const kSinCosTable8bit_Sext as *const int16)
            .offset(64 as ::core::ffi::c_int as isize) as *const int16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize),
        r24 as uint8,
    ) as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as uint16;
    return (r18 as ::core::ffi::c_int
        + Mult8x8(
            *((&raw const kSinCosTable8bit_Sext as *const int16)
                .offset(64 as ::core::ffi::c_int as isize) as *const int16
                as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize)
                .offset(1 as ::core::ffi::c_int as isize),
            r24 as uint8,
        ) as ::core::ffi::c_int) as uint16;
}
unsafe fn EprojInit_RocksKraidSpits(mut j: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v1).x_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v1).y_pos as ::core::ffi::c_int - 96 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(1024 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 1536 as uint16;
}
unsafe fn EprojInit_RocksFallingKraidCeiling(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 312 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x3f as ::core::ffi::c_int)
        + 64 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 1536 as uint16;
}
unsafe fn EprojInit_RocksWhenKraidRises(mut j: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x3f as ::core::ffi::c_int) as int16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v1 = !(v1 as ::core::ffi::c_int) as int16;
    }
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = ((*gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .x_pos as ::core::ffi::c_int
        + v1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 432 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(1280 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 1536 as uint16;
    QueueSfx3_Max6(0x1e as uint16, game);
}
pub unsafe fn EprojPreInstr_KraidRocks(mut k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    } else {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v2;
        if v2 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        {
            v2 = -(256 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v2;
        let ref mut fresh122 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh122 = (*fresh122 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn EprojPreInstr_RocksFallingKraidCeiling(mut k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                + 24 as ::core::ffi::c_int
                & 0x3fff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn sub_869DA5(mut k: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
}
unsafe fn EprojInit_MiniKraidSpit(mut j: uint16, game: &mut Game) {
    let mut v2: *mut ExtraEnemyRam7800 = ::core::ptr::null_mut::<ExtraEnemyRam7800>();
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    v2 = gExtraEnemyRam7800(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) =
        ((*v2).kraid.field_4 as ::core::ffi::c_int + (*v1).x_pos as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) =
        ((*v1).y_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (*v2).kraid.kraid_next;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (*v2).kraid.field_2;
}
pub unsafe fn EprojPreInit_MiniKraidSpit(mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    } else {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        v2 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int) as int16;
        if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - 960 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            v2 = 1024 as int16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v2 as uint16;
    };
}
static kEprojInit_MiniKraidSpikes_Tab0: [int16; 3] = [
    -(2 as ::core::ffi::c_int) as int16,
    12 as ::core::ffi::c_int as int16,
    24 as ::core::ffi::c_int as int16,
];
unsafe fn EprojInit_MiniKraidSpikes(mut j: uint16, mut a: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = a;
    let mut r18: uint16 = kEprojInit_MiniKraidSpikes_Tab0[((*gExtraEnemyRam7800(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .kraid
    .kraid_healths_8ths[0 as ::core::ffi::c_int as usize]
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int)
        as usize] as uint16;
    let mut v3: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        (r18 as ::core::ffi::c_int + (*v3).y_pos as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
}
unsafe fn EprojInit_MiniKraidSpikesLeft(mut j: uint16, game: &mut Game) {
    EprojInit_MiniKraidSpikes(j, -(0x200 as ::core::ffi::c_int) as uint16, game);
}
unsafe fn EprojInit_MiniKraidSpikesRight(mut j: uint16, game: &mut Game) {
    EprojInit_MiniKraidSpikes(j, 0x200 as uint16, game);
}
pub unsafe fn EprojPreInstr_MiniKraidSpikes(mut k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
unsafe fn EprojInit_WalkingLavaSeahorseFireball(mut j: uint16, game: &mut Game) {
    static word_869EF9: [int16; 3] = [
        -(0x100 as ::core::ffi::c_int) as int16,
        0 as ::core::ffi::c_int as int16,
        0x100 as ::core::ffi::c_int as int16,
    ];
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v1).y_pos as ::core::ffi::c_int - 12 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(1024 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v1).x_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
    if (*v1).ai_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = 1024 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) =
            ((*v1).x_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = word_869EF9[(*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int) as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
}
pub unsafe fn EprojPreInstr_WalkingLavaSeahorseFireball(mut k: uint16, game: &mut Game) {
    let mut v2: uint16 = 0;
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    } else {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            v2 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x1ab7 as ::core::ffi::c_int as isize) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - 64 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = v2;
            if v2 as ::core::ffi::c_int - 512 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v2 = 512 as uint16;
            }
        } else {
            v2 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x1ab7 as ::core::ffi::c_int as isize) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + 64 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = v2;
            if v2 as ::core::ffi::c_int + 512 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v2 = -(512 as ::core::ffi::c_int) as uint16;
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v2;
    };
}
unsafe fn EprojInit_PirateMotherBrainLaser(mut j: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (if game.eproj_spawn_r22 as ::core::ffi::c_int != 0 {
        addr_word_869F7D as ::core::ffi::c_int
    } else {
        addr_word_869F41 as ::core::ffi::c_int
    }) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        (0x86a05b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = game.eproj_spawn_pt.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = game.eproj_spawn_pt.y;
    let mut v3: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*(RomPtr_A0((*v3).enemy_ptr as uint16_t, game) as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        | 0x1000 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).parameter_1;
    QueueSfx2_Max6(0x67 as uint16, game);
}
pub unsafe fn EprojInstr_SetPreInstrAndRun(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(epjp as *mut uint16);
    return epjp;
}
pub unsafe fn EprojPreInstr_PirateMotherBrainLaser_MoveLeft(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh21 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh21 = (*fresh21 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let ref mut fresh22 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh22 = (*fresh22 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    }
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojPreInstr_PirateMotherBrainLaser_MoveRight(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh23 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh23 = (*fresh23 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let ref mut fresh24 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh24 = (*fresh24 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
    }
}
unsafe fn EprojInit_PirateClaw(mut j: uint16, game: &mut Game) {
    let mut r: Rect16U = game.eproj_spawn_rect;
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (r.y as ::core::ffi::c_int + r.h as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (r.x as ::core::ffi::c_int + r.w as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (if *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
    {
        addr_off_869FE1 as ::core::ffi::c_int
    } else {
        addr_off_869FB9 as ::core::ffi::c_int
    }) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) =
        (0x86a05b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 2048 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 1 as uint16;
}
pub unsafe fn EprojPreInstr_PirateClawThrownLeft(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize)
        != 0
    {
        let ref mut fresh118 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh118 = (*fresh118 as ::core::ffi::c_int
            - *(((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int) as uint16;
        let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - 32 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v2;
        if v2 == 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
        }
    } else {
        let ref mut fresh119 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh119 = (*fresh119 as ::core::ffi::c_int
            + *(((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int) as uint16;
        let ref mut fresh120 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh120 = (*fresh120 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    }
    let ref mut fresh121 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh121 = (*fresh121).wrapping_add(1);
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojPreInstr_PirateClawThrownRight(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize)
        != 0
    {
        let ref mut fresh114 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh114 = (*fresh114 as ::core::ffi::c_int
            + *(((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int) as uint16;
        let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - 32 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v2;
        if v2 == 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
        }
    } else {
        let ref mut fresh115 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh115 = (*fresh115 as ::core::ffi::c_int
            - *(((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int) as uint16;
        let ref mut fresh116 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh116 = (*fresh116 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    }
    let ref mut fresh117 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh117 = (*fresh117).wrapping_add(1);
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
    }
}
static word_86A2D6: [int16; 6] = [
    64 as ::core::ffi::c_int as int16,
    72 as ::core::ffi::c_int as int16,
    80 as ::core::ffi::c_int as int16,
    -(64 as ::core::ffi::c_int) as int16,
    -(72 as ::core::ffi::c_int) as int16,
    -(80 as ::core::ffi::c_int) as int16,
];
unsafe fn EprojInit_A379(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v3: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (word_86A2D6[v3 as usize] as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + 80 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *(game.g_rom.offset(
        (((0x86a2e2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86a2e2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v3 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = v2;
}
unsafe fn EprojInit_CeresElevatorPad(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + 28 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 60 as uint16;
    sub_86A301(j, game);
}
pub unsafe fn sub_86A301(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16);
}
unsafe fn EprojInit_CeresElevatorPlatform(mut j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 97 as uint16;
    sub_86A301(j, game);
}
pub unsafe fn EprojPreInstr_CeresElevatorPad(mut k: uint16, game: &mut Game) {
    let mut v2: bool = false;
    let mut v3: bool = false;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize)
        == 0
        || {
            v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                == 1 as ::core::ffi::c_int;
            v3 = ((*((&raw mut game.g_ram as *mut uint8)
                .offset(0x1aff as ::core::ffi::c_int as isize) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int;
            let ref mut fresh112 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x1aff as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh112 = (*fresh112).wrapping_sub(1);
            v2 as ::core::ffi::c_int != 0
        }
        || v3 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 28 as ::core::ffi::c_int) as uint16;
        let ref mut fresh113 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh113 = (*fresh113).wrapping_add(1);
        if *fresh113 as ::core::ffi::c_int - 73 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) = 72 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = addr_off_86A28B as ::core::ffi::c_int as uint16;
            CallSomeSamusCode(0xe as uint16, game);
        }
    }
}
pub unsafe fn EprojPreInstr_CeresElevatorPlatform(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 72 as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_off_86A28B as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn EprojPreInstr_PrePhantomRoom(mut j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x923 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
}
pub unsafe fn EprojInstr_A3BE(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize);
    return epjp;
}
pub unsafe fn EprojInstr_GotoWithProbability25(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    if NextRandom(game) as ::core::ffi::c_int & 0xc000 as ::core::ffi::c_int
        == 0xc000 as ::core::ffi::c_int
    {
        return *(epjp as *mut uint16) as uintptr_t as *const uint8;
    } else {
        return epjp.offset(2 as ::core::ffi::c_int as isize);
    };
}
unsafe fn EprojInit_BombTorizoLowHealthDrool(mut j: uint16, game: &mut Game) {
    let mut parameter_1: int16 = 0;
    let mut v4: uint16 = 0;
    let mut v5: uint16 = 0;
    let mut v8: uint16 = 0;
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *(game.g_rom.offset(
        (((0x86a64d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86a64d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((NextRandom(game) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int
                & 0xe as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    NextRandom(game);
    let mut v2: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) =
        ((*v2).y_pos as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as uint16;
    parameter_1 = (*v2).parameter_1 as int16;
    if parameter_1 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        v4 = (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x1fe as ::core::ffi::c_int) as uint16;
    } else {
        if (parameter_1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            v5 = 32 as uint16;
        } else {
            v5 = 224 as uint16;
        }
        v4 = (2 as ::core::ffi::c_int
            * (v5 as ::core::ffi::c_int
                + (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x5e5 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int)
                - 8 as ::core::ffi::c_int)) as uint16;
    }
    let mut v6: ::core::ffi::c_int = v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) =
        kSinCosTable8bit_Sext[(v6 + 64 as ::core::ffi::c_int) as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = kSinCosTable8bit_Sext[v6 as usize] as uint16;
    let mut v7: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*v7).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        v8 = ((*v7).x_pos as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
    } else {
        v8 = ((*v7).x_pos as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = v8;
}
unsafe fn EprojInit_BombTorizoLowHealthInitialDrool(mut j: uint16, game: &mut Game) {
    let mut v3: int16 = 0;
    let mut parameter_1: int16 = 0;
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    NextRandom(game);
    let mut v2: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*v2).y_pos as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int)
        - 5 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int)
        + 48 as ::core::ffi::c_int) as uint16;
    NextRandom(game);
    v3 = (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int) as int16;
    parameter_1 = (*v2).parameter_1 as int16;
    if parameter_1 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) =
            ((*v2).x_pos as ::core::ffi::c_int + v3 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
    } else {
        if (parameter_1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = ((*v2).x_pos as ::core::ffi::c_int
                + v3 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int) as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = ((*v2).x_pos as ::core::ffi::c_int
                + v3 as ::core::ffi::c_int
                - 8 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
    };
}
unsafe fn EprojInit_A977(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    CalculatePlmBlockCoords(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c27 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (16 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1c29 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (16 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1c2b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        - 4 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
}
static kEprojInit_BombTorizoExplosiveSwipe_Tab0: [int16; 11] = [
    -(30 as ::core::ffi::c_int) as int16,
    -(40 as ::core::ffi::c_int) as int16,
    -(47 as ::core::ffi::c_int) as int16,
    -(31 as ::core::ffi::c_int) as int16,
    -(21 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(28 as ::core::ffi::c_int) as int16,
    -(43 as ::core::ffi::c_int) as int16,
    -(48 as ::core::ffi::c_int) as int16,
    -(31 as ::core::ffi::c_int) as int16,
    -(21 as ::core::ffi::c_int) as int16,
];
static kEprojInit_BombTorizoExplosiveSwipe_Tab1: [int16; 11] = [
    -(52 as ::core::ffi::c_int) as int16,
    -(28 as ::core::ffi::c_int) as int16,
    -(11 as ::core::ffi::c_int) as int16,
    9 as ::core::ffi::c_int as int16,
    21 as ::core::ffi::c_int as int16,
    20 as ::core::ffi::c_int as int16,
    -(52 as ::core::ffi::c_int) as int16,
    -(27 as ::core::ffi::c_int) as int16,
    -(10 as ::core::ffi::c_int) as int16,
    9 as ::core::ffi::c_int as int16,
    20 as ::core::ffi::c_int as int16,
];
static kEprojInit_BombTorizoStatueBreaking_Xpos: [int16; 16] = [
    8 as ::core::ffi::c_int as int16,
    0x18 as ::core::ffi::c_int as int16,
    -(8 as ::core::ffi::c_int) as int16,
    8 as ::core::ffi::c_int as int16,
    0x18 as ::core::ffi::c_int as int16,
    -(8 as ::core::ffi::c_int) as int16,
    8 as ::core::ffi::c_int as int16,
    0x18 as ::core::ffi::c_int as int16,
    8 as ::core::ffi::c_int as int16,
    -(8 as ::core::ffi::c_int) as int16,
    0x18 as ::core::ffi::c_int as int16,
    8 as ::core::ffi::c_int as int16,
    -(8 as ::core::ffi::c_int) as int16,
    0x18 as ::core::ffi::c_int as int16,
    8 as ::core::ffi::c_int as int16,
    -(8 as ::core::ffi::c_int) as int16,
];
static kEprojInit_BombTorizoStatueBreaking_Ypos: [int16; 8] = [
    -(8 as ::core::ffi::c_int) as int16,
    -(8 as ::core::ffi::c_int) as int16,
    8 as ::core::ffi::c_int as int16,
    8 as ::core::ffi::c_int as int16,
    8 as ::core::ffi::c_int as int16,
    0x18 as ::core::ffi::c_int as int16,
    0x18 as ::core::ffi::c_int as int16,
    0x18 as ::core::ffi::c_int as int16,
];
static kEprojInit_BombTorizoStatueBreaking_Yvel: [int16; 8] = [
    256 as ::core::ffi::c_int as int16,
    256 as ::core::ffi::c_int as int16,
    256 as ::core::ffi::c_int as int16,
    256 as ::core::ffi::c_int as int16,
    256 as ::core::ffi::c_int as int16,
    256 as ::core::ffi::c_int as int16,
    256 as ::core::ffi::c_int as int16,
    256 as ::core::ffi::c_int as int16,
];
static kEprojInit_BombTorizoStatueBreaking_F: [int16; 8] = [
    16 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
];
static kEprojInit_BombTorizoLowHealthExplode_X: [int16; 6] = [
    0 as ::core::ffi::c_int as int16,
    12 as ::core::ffi::c_int as int16,
    -(12 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    -(16 as ::core::ffi::c_int) as int16,
];
static kEprojInit_BombTorizoLowHealthExplode_Y: [int16; 6] = [
    -(8 as ::core::ffi::c_int) as int16,
    -(8 as ::core::ffi::c_int) as int16,
    -(8 as ::core::ffi::c_int) as int16,
    -(20 as ::core::ffi::c_int) as int16,
    -(20 as ::core::ffi::c_int) as int16,
    -(20 as ::core::ffi::c_int) as int16,
];
unsafe fn EprojInit_BombTorizoExplosiveSwipe(mut j: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*v1).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut v4: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1993 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let mut v5: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) = ((*v1).x_pos as ::core::ffi::c_int
            - kEprojInit_BombTorizoExplosiveSwipe_Tab0[v4 as usize] as ::core::ffi::c_int)
            as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) = ((*v1).y_pos as ::core::ffi::c_int
            + kEprojInit_BombTorizoExplosiveSwipe_Tab1[v4 as usize] as ::core::ffi::c_int)
            as uint16;
    } else {
        let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1993 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = ((*v1).x_pos as ::core::ffi::c_int
            + kEprojInit_BombTorizoExplosiveSwipe_Tab0[v2 as usize] as ::core::ffi::c_int)
            as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = ((*v1).y_pos as ::core::ffi::c_int
            + kEprojInit_BombTorizoExplosiveSwipe_Tab1[v2 as usize] as ::core::ffi::c_int)
            as uint16;
    };
}
unsafe fn EprojInit_BombTorizoStatueBreaking(mut j: uint16, game: &mut Game) {
    let mut v1: int8 = 0;
    CalculatePlmBlockCoords(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c27 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    v1 = *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as int8;
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = *(game.g_rom.offset(
        (((0x86a7ab as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86a7ab as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v2 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (kEprojInit_BombTorizoStatueBreaking_Xpos[v2 as usize]
        as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x1c29 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut v4: ::core::ffi::c_int =
        (v1 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (kEprojInit_BombTorizoStatueBreaking_Ypos[v4 as usize]
        as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x1c2b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = kEprojInit_BombTorizoStatueBreaking_Yvel[v4 as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = kEprojInit_BombTorizoStatueBreaking_F[v4 as usize] as uint16;
    let ref mut fresh15 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1bd7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v3 as isize);
    *fresh15 = (*fresh15 as ::core::ffi::c_int | 0x1000 as ::core::ffi::c_int) as uint16;
}
unsafe fn EprojInit_BombTorizoLowHealthExplode(mut j: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v1).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v1).y_pos;
    if (*v1).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let ref mut fresh13 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1993 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh13 = (*fresh13 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
    let ref mut fresh14 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh14 = (*fresh14 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    let mut v3: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v4: uint16 = (kEprojInit_BombTorizoLowHealthExplode_X[v3 as usize]
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = v4;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = v4;
    let mut v5: uint16 = (kEprojInit_BombTorizoLowHealthExplode_Y[v3 as usize]
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = v5;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = v5;
}
unsafe fn EprojInit_BombTorizoDeathExplosion(mut j: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut x_pos: uint16 = (*v1).x_pos;
    let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = x_pos;
    let mut y_pos: uint16 = (*v1).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = y_pos;
}
pub unsafe fn sub_86A887(mut v0: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v3: int16 = 0;
    let mut v4: int16 = 0;
    if EprojBlockCollisition_Horiz(v0, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut v6: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v6 as isize) = 0xa48a as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v6 as isize) = 1 as uint16;
    } else {
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16;
        if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            v4 = (v2 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as int16;
            if (v4 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                v4 = 3 as int16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = v4 as uint16;
        } else {
            v3 = (v2 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as int16;
            if v3 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                v3 = 3 as int16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = v3 as uint16;
        }
        let mut carry: uint8 = EprojBlockCollisition_Vertical(v0, game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            || carry == 0
        {
            let mut v5: uint16 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + 16 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = v5;
            if v5 as ::core::ffi::c_int & 0xf000 as ::core::ffi::c_int == 4096 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) = 0 as uint16;
            }
        } else {
            let mut v7: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let ref mut fresh111 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v7 as isize);
            *fresh111 = (*fresh111 as ::core::ffi::c_int - 3 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v7 as isize) = 0xa48e as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v7 as isize) = 1 as uint16;
        }
    };
}
pub unsafe fn EprojPreInstr_A977(mut k: uint16, game: &mut Game) {
    let mut carry: uint8 = EprojBlockCollisition_Vertical(k, game);
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || carry == 0
    {
        let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v2;
        if v2 as ::core::ffi::c_int & 0xf000 as ::core::ffi::c_int == 4096 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 4096 as uint16;
        }
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0xa918 as uint16;
    };
}
pub unsafe fn sub_86A91A(mut v0: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8d as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x100 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 256 as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8d as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x200 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = -(256 as ::core::ffi::c_int) as uint16;
    }
    EprojBlockCollisition_Horiz(v0, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8d as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x400 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 256 as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8d as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x800 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = -(256 as ::core::ffi::c_int) as uint16;
    }
    EprojBlockCollisition_Vertical(v0, game);
}
unsafe fn EprojInit_AB07(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: *mut VramWriteEntry = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v2).size = 64 as uint16;
    (*v2).src.addr = addr_kEprojInit_AB07_Tile0 as ::core::ffi::c_int as VoidP;
    *(&raw mut (*v2).src.bank as *mut uint16) = 134 as uint16;
    (*v2).vram_dst = 28160 as uint16;
    v1 = (v1 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    let mut v3: *mut VramWriteEntry = gVramWriteEntry(v1 as ::core::ffi::c_int, game);
    (*v3).size = 64 as uint16;
    (*v3).src.addr = addr_kEprojInit_AB07_Tile1 as ::core::ffi::c_int as VoidP;
    *(&raw mut (*v3).src.bank as *mut uint16) = 134 as uint16;
    (*v3).vram_dst = 28416 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
        as *mut uint16) = (v1 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    let mut v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 36 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojPreInstr_AB07(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = ((8 as ::core::ffi::c_int
        * (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int)) as int16
        as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        amt as uint32,
    );
    amt = (((8 as ::core::ffi::c_int
        * (*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 36 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int)) as int16
        as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as int32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        amt as uint32,
    );
}
pub unsafe fn EprojInstr_SpawnEnemyDrops(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize),
    };
    SpawnEnemyDrops(
        (if *((&raw mut game.g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
        {
            *(epjp.offset(2 as ::core::ffi::c_int as isize) as *mut uint16) as ::core::ffi::c_int
        } else {
            *(epjp as *mut uint16) as ::core::ffi::c_int
        }) as uint16,
        k,
        0 as uint16,
        game,
    );
    return epjp.offset(4 as ::core::ffi::c_int as isize);
}
unsafe fn Eproj_InitXYVelRandom(mut j: uint16, mut k: uint16, mut pt: Point16U, game: &mut Game) {
    let mut v2: *const uint16 = RomPtr_86(k as uint16_t, game) as *const uint16;
    let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = *v2;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (*v2.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        + pt.x as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (*v2.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        + NextRandom(game) as uint8 as ::core::ffi::c_int
        - 128 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (*v2.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        + pt.y as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (*v2.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        + NextRandom(game) as uint8 as ::core::ffi::c_int
        - 128 as ::core::ffi::c_int) as uint16;
}
unsafe fn EprojInit_BombTorizosChozoOrbs(mut j: uint16, game: &mut Game) {
    let mut E: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: uint16 = (if (*E).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        addr_kEprojInit_BombTorizosChozoOrbs_init0 as ::core::ffi::c_int
    } else {
        addr_kEprojInit_BombTorizosChozoOrbs_init1 as ::core::ffi::c_int
    }) as uint16;
    Eproj_InitXYVelRandom(
        j,
        v2,
        Point16U {
            x: (*E).x_pos,
            y: (*E).y_pos,
        },
        game,
    );
}
unsafe fn EprojInit_GoldenTorizosChozoOrbs(mut j: uint16, game: &mut Game) {
    let mut E: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: uint16 = (if (*E).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        addr_kEprojInit_GoldenTorizosChozoOrbs_init0 as ::core::ffi::c_int
    } else {
        addr_kEprojInit_GoldenTorizosChozoOrbs_init1 as ::core::ffi::c_int
    }) as uint16;
    Eproj_InitXYVelRandom(
        j,
        v2,
        Point16U {
            x: (*E).x_pos,
            y: (*E).y_pos,
        },
        game,
    );
}
pub unsafe fn EprojPreInstr_BombTorizosChozoOrbs(mut k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) != 0 {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = addr_off_86AB25 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = 1 as uint16;
    } else {
        let mut carry: uint8 = EprojBlockCollisition_Vertical(k, game);
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            || carry == 0
        {
            let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + 18 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = v2;
            if v2 as ::core::ffi::c_int & 0xf000 as ::core::ffi::c_int == 4096 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) = 0 as uint16;
            }
        } else {
            let mut v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v4 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
                .offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v4 as isize)
                as ::core::ffi::c_int
                & 0xfff0 as ::core::ffi::c_int
                | 8 as ::core::ffi::c_int)
                - 2 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v4 as isize) = addr_off_86AB41 as ::core::ffi::c_int as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v4 as isize) = 1 as uint16;
        }
    };
}
pub unsafe fn EprojPreInstr_GoldenTorizosChozoOrbs(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = 0;
    let mut v2: int16 = 0;
    let mut v3: uint16 = 0;
    let mut v4: uint16 = 0;
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            -(*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int) as uint16;
    }
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        && {
            v1 = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        }
        && {
            v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as int16;
            if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                v3 = (v2 as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as uint16;
            } else {
                v3 = (v2 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
            };
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = v3;
            v4 = -(*((&raw mut game.g_ram as *mut uint8)
                .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = v4;
            v4 as ::core::ffi::c_int & 0xff80 as ::core::ffi::c_int == 0xff80 as ::core::ffi::c_int
        }
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) as ::core::ffi::c_int
            & 0xfff0 as ::core::ffi::c_int
            | 8 as ::core::ffi::c_int)
            - 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) = addr_off_86AB41 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) = 1 as uint16;
    } else {
        let ref mut fresh110 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh110 = (*fresh110 as ::core::ffi::c_int + 24 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn EprojInstr_GotoDependingOnXDirection(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1ab7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as *mut uint16,
        amt as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return *(epjp.offset(2 as ::core::ffi::c_int as isize) as *mut uint16) as uintptr_t
            as *const uint8;
    } else {
        return *(epjp as *mut uint16) as uintptr_t as *const uint8;
    };
}
unsafe fn EprojInit_TorizoSonicBoom(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = if NextRandom(game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        -(12 as ::core::ffi::c_int)
    } else {
        20 as ::core::ffi::c_int
    };
    let mut v2: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = ((*v2).y_pos as ::core::ffi::c_int + v1) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = 0 as uint16;
    if (*v2).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) =
            ((*v2).x_pos as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = 624 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = addr_off_86ADD2 as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) =
            ((*v2).x_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = -(624 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = addr_off_86ADBF as ::core::ffi::c_int as uint16;
    };
}
pub unsafe fn EprojPreInstr_TorizoSonicBoom(mut k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize) = addr_off_86ADE5 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize);
    } else {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh109 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh109 = (*fresh109 as ::core::ffi::c_int
            + if *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                -(16 as ::core::ffi::c_int)
            } else {
                16 as ::core::ffi::c_int
            }) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int
            == 4096 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
        }
    };
}
unsafe fn EprojInit_WreckedShipChozoSpikeFootsteps(mut j: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + (*v1).x_pos as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v1).y_pos as ::core::ffi::c_int + 28 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojInstr_ResetXYpos1(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize);
    return epjp;
}
unsafe fn EprojInit_TourianStatueDustClouds(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 188 as uint16;
}
unsafe fn EprojInit_TourianLandingDustCloudsRightFoot(mut j: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v1).y_pos as ::core::ffi::c_int + 48 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v1).x_pos as ::core::ffi::c_int + 24 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojInstr_MoveY_Minus4(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh30 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh30 = (*fresh30 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    return epjp;
}
unsafe fn EprojInit_TorizoLandingDustCloudLeftFoot(mut j: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v1).y_pos as ::core::ffi::c_int + 48 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v1).x_pos as ::core::ffi::c_int - 24 as ::core::ffi::c_int) as uint16;
}
unsafe fn EprojInit_GoldenTorizoEgg(mut j: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = ((0xe2 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
        + 64 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v1).parameter_1;
    let mut v4: uint16 =
        (if (*v1).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            addr_kEprojInit_GoldenTorizoEgg0 as ::core::ffi::c_int
        } else {
            addr_kEprojInit_GoldenTorizoEgg1 as ::core::ffi::c_int
        }) as uint16;
    Eproj_InitXYVelRandom(
        j,
        v4,
        Point16U {
            x: (*v1).x_pos,
            y: (*v1).y_pos,
        },
        game,
    );
}
pub unsafe fn EprojPreInstr_GoldenTorizoEgg(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh104 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1b23 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh104 = (*fresh104).wrapping_sub(1);
    if *fresh104 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        let ref mut fresh105 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh105 = (*fresh105 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (if *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            256 as ::core::ffi::c_int
        } else {
            -(256 as ::core::ffi::c_int)
        }) as uint16;
    } else {
        if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = -(*((&raw mut game.g_ram as *mut uint8)
                .offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int)
                as uint16;
            let ref mut fresh106 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x1aff as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh106 = (*fresh106 as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int) as uint16;
        }
        if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            let ref mut fresh107 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh107 = (*fresh107 as ::core::ffi::c_int
                + if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1ab7 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    32 as ::core::ffi::c_int
                } else {
                    -(32 as ::core::ffi::c_int)
                }) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = -(*((&raw mut game.g_ram as *mut uint8)
                .offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int)
                as uint16;
        }
        let ref mut fresh108 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh108 = (*fresh108 as ::core::ffi::c_int + 48 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int
            == 4096 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
        }
    };
}
pub unsafe fn sub_86B0B9(mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = 0xb0dd as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = 0 as uint16;
    } else {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v2 = 48 as int16;
        } else {
            v2 = -(48 as ::core::ffi::c_int) as int16;
        }
        let ref mut fresh103 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh103 = (*fresh103 as ::core::ffi::c_int + v2 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn sub_86B0DD(mut k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (if *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            addr_off_86B1A8 as ::core::ffi::c_int
        } else {
            addr_off_86B190 as ::core::ffi::c_int
        }) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    } else {
        let ref mut fresh102 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh102 = (*fresh102 as ::core::ffi::c_int + 48 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn sub_86B13E(mut k: uint16, mut epjp: *const uint8, game: &mut Game) -> *const uint8 {
    return (if *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        addr_off_86B166 as ::core::ffi::c_int
    } else {
        addr_off_86B14B as ::core::ffi::c_int
    }) as uintptr_t as *const uint8;
}
static word_86B205: [int16; 2] = [
    -(0x1e as ::core::ffi::c_int) as int16,
    0x1e as ::core::ffi::c_int as int16,
];
unsafe fn EprojInit_GoldenTorizoSuperMissile(mut j: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xe54 as ::core::ffi::c_int as isize)
        as *mut uint16);
    let mut v3: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v4: ::core::ffi::c_int = if (*v3).parameter_1 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = ((*v3).x_pos as ::core::ffi::c_int
        + word_86B205[v4 as usize] as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v3).y_pos as ::core::ffi::c_int - 52 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *(game.g_rom.offset(
        (((0x86b209 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86b209 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v4 as isize);
}
pub unsafe fn EprojPreInstr_GoldenTorizoSuperMissile(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int,
        game,
    );
    let mut v3: ::core::ffi::c_int = if (*v2).parameter_1 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        32 as ::core::ffi::c_int
    } else {
        -(32 as ::core::ffi::c_int)
    };
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*v2).x_pos as ::core::ffi::c_int + v3) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) =
        ((*v2).y_pos as ::core::ffi::c_int - 52 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojPreInstr_B237(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = 0;
    let mut carry: uint8 = 0;
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int != 0 || {
        carry = EprojBlockCollisition_Vertical(k, game);
        v1 = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
            && carry as ::core::ffi::c_int != 0
    } {
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = addr_off_86B2EF as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = 1 as uint16;
    } else {
        let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v2;
        if v2 as ::core::ffi::c_int & 0xf000 as ::core::ffi::c_int == 4096 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
        }
    };
}
unsafe fn sub_86B279(mut k: uint16, mut a: uint16, game: &mut Game) {
    let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (4 as ::core::ffi::c_int
        * kSinCosTable8bit_Sext[(a as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (4 as ::core::ffi::c_int
        * kSinCosTable8bit_Sext[a as usize] as ::core::ffi::c_int)
        as uint16;
}
pub unsafe fn EprojInstr_SetVelTowardsSamus1(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    sub_86B279(
        k,
        (CalculateAngleOfSamusFromEproj(k, game) as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int)
            as uint16,
        game,
    );
    return epjp;
}
pub unsafe fn EprojInstr_SetVelTowardsSamus2(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    sub_86B279(
        k,
        (CalculateAngleOfSamusFromEproj(k, game) as ::core::ffi::c_int | 0x80 as ::core::ffi::c_int)
            as uint16,
        game,
    );
    return epjp;
}
unsafe fn EprojInit_GoldenTorizoEyeBeam(mut j: uint16, game: &mut Game) {
    let mut E: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: uint16 = (if (*E).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        addr_stru_86B376 as ::core::ffi::c_int
    } else {
        addr_stru_86B380 as ::core::ffi::c_int
    }) as uint16;
    Eproj_InitXYVelRandom(
        j,
        v2,
        Point16U {
            x: (*E).x_pos,
            y: (*E).y_pos,
        },
        game,
    );
    let mut v3: uint16 = ((NextRandom(game) as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
        - 16 as ::core::ffi::c_int
        + 192 as ::core::ffi::c_int) as uint16;
    if (*E).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v3 = (v3 as ::core::ffi::c_int + 128 as ::core::ffi::c_int) as uint16;
    }
    let mut v4: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v5: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v5 as isize) = (8 as ::core::ffi::c_int
        * kSinCosTable8bit_Sext[(v4 + 64 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int)
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v5 as isize) = (8 as ::core::ffi::c_int
        * kSinCosTable8bit_Sext[v4 as usize] as ::core::ffi::c_int)
        as uint16;
}
pub unsafe fn EprojPreInstr_GoldenTorizoEyeBeam(mut k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            addr_off_86B3CD as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
        return;
    }
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0xfff0 as ::core::ffi::c_int
            | 8 as ::core::ffi::c_int)
            - 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_off_86B3E5 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
        return;
    }
}
pub unsafe fn EprojInstr_GotoIfFunc1(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    if (*gExtraEnemyRam7800(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int,
        game,
    ))
    .kraid
    .kraid_healths_8ths[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return *(epjp as *mut uint16) as uintptr_t as *const uint8;
    } else {
        return epjp.offset(2 as ::core::ffi::c_int as isize);
    };
}
pub unsafe fn EprojInstr_ResetXYpos2(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize);
    return epjp;
}
unsafe fn EprojInit_TourianEscapeShaftFakeWallExplode(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 272 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 272 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 2184 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 2184 as uint16;
}
unsafe fn EprojInit_LavaSeahorseFireball(mut j: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v1).y_pos as ::core::ffi::c_int - 28 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(961 as ::core::ffi::c_int) as uint16;
    if (*v1).ai_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) =
            ((*v1).x_pos as ::core::ffi::c_int + 12 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = 704 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = addr_word_86B4CB as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) =
            ((*v1).x_pos as ::core::ffi::c_int - 12 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = -(704 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = addr_word_86B4BF as ::core::ffi::c_int as uint16;
    };
}
pub unsafe fn sub_86B535(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
    let mut v6: int16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as int16;
    if v6 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (v6 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
        Eproj_DeleteIfYposOutside(k, game);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (v6 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
        if (v6 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = (if *((&raw mut game.g_ram as *mut uint8)
                .offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize)
                as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                addr_word_86B4E3 as ::core::ffi::c_int
            } else {
                addr_word_86B4D7 as ::core::ffi::c_int
            }) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 1 as uint16;
        }
    };
}
pub unsafe fn Eproj_DeleteIfYposOutside(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 288 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
    }
}
static word_86B65B: [int16; 20] = [
    -(16 as ::core::ffi::c_int) as int16,
    16 as ::core::ffi::c_int as int16,
    -(96 as ::core::ffi::c_int) as int16,
    -(64 as ::core::ffi::c_int) as int16,
    -(128 as ::core::ffi::c_int) as int16,
    -(32 as ::core::ffi::c_int) as int16,
    -(96 as ::core::ffi::c_int) as int16,
    64 as ::core::ffi::c_int as int16,
    -(128 as ::core::ffi::c_int) as int16,
    32 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    96 as ::core::ffi::c_int as int16,
    -(64 as ::core::ffi::c_int) as int16,
    112 as ::core::ffi::c_int as int16,
    -(64 as ::core::ffi::c_int) as int16,
    128 as ::core::ffi::c_int as int16,
    -(64 as ::core::ffi::c_int) as int16,
    144 as ::core::ffi::c_int as int16,
    -(64 as ::core::ffi::c_int) as int16,
];
unsafe fn EprojInit_EyeDoorProjectile(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1c27 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1dc7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c27 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    CalculatePlmBlockCoords(v1, game);
    let mut v3: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (word_86B65B[v3 as usize] as ::core::ffi::c_int
        + 8 as ::core::ffi::c_int
            * (2 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1c29 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int)) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (word_86B65B[(v3 + 1 as ::core::ffi::c_int) as usize]
        as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x1c2b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
}
static word_86B6B1: [int16; 4] = [
    -(64 as ::core::ffi::c_int) as int16,
    512 as ::core::ffi::c_int as int16,
    64 as ::core::ffi::c_int as int16,
    512 as ::core::ffi::c_int as int16,
];
unsafe fn EprojInit_EyeDoorSweat(mut j: uint16, game: &mut Game) {
    CalculatePlmBlockCoords(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c27 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (8 as ::core::ffi::c_int
        * (2 as ::core::ffi::c_int
            * (*((&raw mut game.g_ram as *mut uint8).offset(0x1c29 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int)
            + 1 as ::core::ffi::c_int)) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (16 as ::core::ffi::c_int
        * (*((&raw mut game.g_ram as *mut uint8).offset(0x1c2b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int)) as uint16;
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = word_86B6B1[v2 as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = word_86B6B1[(v2 + 1 as ::core::ffi::c_int) as usize] as uint16;
}
pub unsafe fn EprojPreInstr_EyeDoorProjectile(mut k: uint16, game: &mut Game) {
    let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if !(EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0)
    {
        let mut v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh100 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize);
        *fresh100 = (*fresh100 as ::core::ffi::c_int
            + (kSinCosTable8bit_Sext[(v1 + 64 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int
                >> 4 as ::core::ffi::c_int)) as uint16;
        let ref mut fresh101 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize);
        *fresh101 = (*fresh101 as ::core::ffi::c_int
            + (kSinCosTable8bit_Sext[v1 as usize] as ::core::ffi::c_int >> 4 as ::core::ffi::c_int))
            as uint16;
        let mut t: ::core::ffi::c_int = PrepareBitAccess(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize),
            game,
        ) as ::core::ffi::c_int;
        if !(*((&raw mut game.g_ram as *mut uint8).offset(0x5e7 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & *(&raw mut game.g_ram as *mut uint8)
                .offset(0xd8b0 as ::core::ffi::c_int as isize)
                .offset(t as isize) as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int)
        {
            return;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = addr_off_86B5F3 as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = 1 as uint16;
}
pub unsafe fn EprojPreInstr_EyeDoorSweat(mut k: uint16, game: &mut Game) {
    EprojBlockCollisition_Horiz(k, game);
    let mut carry: uint8 = EprojBlockCollisition_Vertical(k, game);
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || carry == 0
    {
        let ref mut fresh98 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh98 = (*fresh98 as ::core::ffi::c_int + 12 as ::core::ffi::c_int) as uint16;
    } else {
        let ref mut fresh99 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh99 = (*fresh99 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_off_86B61D as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    };
}
pub unsafe fn EprojInstr_SpawnTourianStatueUnlockingParticle(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_TourianStatueUnlockingParticle as ::core::ffi::c_int as uint16,
        k,
        game,
    );
    return epjp;
}
pub unsafe fn EprojInstr_Earthquake(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = 1 as uint16;
    let ref mut fresh29 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1840 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh29 = (*fresh29 as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int) as uint16;
    return epjp;
}
pub unsafe fn EprojInstr_SpawnTourianStatueUnlockingParticleTail(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_TourianStatueUnlockingParticleTail as ::core::ffi::c_int as uint16,
        k,
        game,
    );
    return epjp;
}
pub unsafe fn EprojInstr_AddToYpos(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh28 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh28 =
        (*fresh28 as ::core::ffi::c_int + *(epjp as *mut uint16) as ::core::ffi::c_int) as uint16;
    return epjp.offset(2 as ::core::ffi::c_int as isize);
}
unsafe fn EprojInit_TourianStatueUnlockingParticleWaterSplash(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x195e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 4 as ::core::ffi::c_int) as uint16;
}
static kEprojInit_TourianStatueEyeGlow_X: [uint16; 4] = [
    0x84 as ::core::ffi::c_int as uint16,
    0x7a as ::core::ffi::c_int as uint16,
    0x9e as ::core::ffi::c_int as uint16,
    0x68 as ::core::ffi::c_int as uint16,
];
static kEprojInit_TourianStatueEyeGlow_Y: [uint16; 4] = [
    0x90 as ::core::ffi::c_int as uint16,
    0x51 as ::core::ffi::c_int as uint16,
    0x80 as ::core::ffi::c_int as uint16,
    0x72 as ::core::ffi::c_int as uint16,
];
static kEprojInit_TourianStatueEyeGlow_Colors: [uint16; 16] = [
    0x6bff as ::core::ffi::c_int as uint16,
    0x33b as ::core::ffi::c_int as uint16,
    0x216 as ::core::ffi::c_int as uint16,
    0x173 as ::core::ffi::c_int as uint16,
    0x7f5f as ::core::ffi::c_int as uint16,
    0x7c1f as ::core::ffi::c_int as uint16,
    0x5816 as ::core::ffi::c_int as uint16,
    0x300c as ::core::ffi::c_int as uint16,
    0x7f5a as ::core::ffi::c_int as uint16,
    0x7ec0 as ::core::ffi::c_int as uint16,
    0x6de0 as ::core::ffi::c_int as uint16,
    0x54e0 as ::core::ffi::c_int as uint16,
    0x6bfa as ::core::ffi::c_int as uint16,
    0x3be0 as ::core::ffi::c_int as uint16,
    0x2680 as ::core::ffi::c_int as uint16,
    0x1580 as ::core::ffi::c_int as uint16,
];
unsafe fn EprojInit_TourianStatueEyeGlow(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = kEprojInit_TourianStatueEyeGlow_X[v2 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = kEprojInit_TourianStatueEyeGlow_Y[v2 as usize];
    let mut v4: uint16 = (4 as ::core::ffi::c_int * v1 as ::core::ffi::c_int) as uint16;
    let mut i: ::core::ffi::c_int = 498 as ::core::ffi::c_int;
    while i != 506 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) =
            kEprojInit_TourianStatueEyeGlow_Colors
                [(v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
        v4 = (v4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        i += 2 as ::core::ffi::c_int;
    }
}
unsafe fn EprojInit_TourianStatueUnlockingParticle(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize);
    let mut v3: uint16 = (2 as ::core::ffi::c_int
        * ((NextRandom(game) as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int)
            - 32 as ::core::ffi::c_int) as uint8 as ::core::ffi::c_int)
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = v3;
    let mut v4: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        kSinCosTable8bit_Sext[(v4 + 64 as ::core::ffi::c_int) as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (4 as ::core::ffi::c_int
        * kSinCosTable8bit_Sext[v4 as usize] as ::core::ffi::c_int)
        as uint16;
}
pub unsafe fn EprojIni_TourianStatueUnlockingParticleTail(mut v0: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize);
}
unsafe fn EprojInit_TourianStatueSoul(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = kEprojInit_TourianStatueEyeGlow_X[v1 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = kEprojInit_TourianStatueEyeGlow_Y[v1 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(1024 as ::core::ffi::c_int) as uint16;
}
unsafe fn EprojInit_TourianStatueBaseDecoration(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 120 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 120 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 184 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 184 as uint16;
}
unsafe fn EprojInit_TourianStatueRidley(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 142 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 142 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 85 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 85 as uint16;
}
unsafe fn EprojInit_TourianStatuePhantoon(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 132 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 132 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 136 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 136 as uint16;
}
pub unsafe fn EprojPreInstr_TourianStatueUnlockingParticleWaterSplash(
    mut k: uint16,
    game: &mut Game,
) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojPreInstr_TourianStatueUnlockingParticle(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1ab7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        amt as uint32,
    );
    let mut v7: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x195e as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
    amt = ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as int16 as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as int32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        amt as uint32,
    );
    if (v7 as ::core::ffi::c_int
        ^ *((&raw mut game.g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int)
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        SpawnEprojWithRoomGfx(addr_stru_86BA5C as ::core::ffi::c_int as uint16, k, game);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int
        == 256 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_off_86B79F as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    } else {
        let ref mut fresh97 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh97 = (*fresh97 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn EprojPreInstr_TourianStatueSoul(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        amt as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0x100 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_off_86B79F as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    }
    let ref mut fresh96 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh96 = (*fresh96 as ::core::ffi::c_int - 128 as ::core::ffi::c_int) as uint16;
}
unsafe fn EprojPreInstr_BA42(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        - *(&raw mut (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x9e00 as ::core::ffi::c_int as isize)
            as *mut ExpandingSquareTransitionHdma)
            .offset(0 as ::core::ffi::c_int as isize))
        .field_0 as *mut uint16) as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojPreInstr_TourianStatueStuff(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1e6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
    {
        let ref mut fresh95 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1e6d as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh95 = (*fresh95 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
    }
    EprojPreInstr_BA42(k, game);
}
pub unsafe fn sub_86BB30(mut j: uint16, game: &mut Game) {
    let mut v1: *mut ExtraEnemyRam8000 = gExtraEnemyRam8000(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *((&raw mut (*v1).pad as *mut uint8)
        .offset(34 as ::core::ffi::c_int as isize) as *mut uint8
        as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *((&raw mut (*v1).pad as *mut uint8)
        .offset(36 as ::core::ffi::c_int as isize) as *mut uint8
        as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *(game.g_rom.offset(
        (((0x86bb1e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86bb1e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        );
}
unsafe fn EprojInit_NuclearWaffleBody(mut j: uint16, game: &mut Game) {
    let mut v2: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (*v2).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (*v2).x_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (*v2).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (*v2).y_subpos;
    let mut v4: *mut ExtraEnemyRam8000 = gExtraEnemyRam8000(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*gExtraEnemyRam7800(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut (*v4).pad as *mut uint8).offset(20 as ::core::ffi::c_int as isize)
                as *mut uint8 as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .kraid
    .kraid_next = j;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf380 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = 1 as uint16;
}
unsafe fn EprojInit_NorfairLavaquakeRocks(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = addr_word_86BBD5 as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) =
        (0x86bc16 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1995 as ::core::ffi::c_int as isize)
        as *mut uint16);
    let mut v2: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*v2).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*v2).x_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*v2).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*v2).y_subpos;
}
pub unsafe fn CallNorfairLavaquakeRocksFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        8829974 => {
            Eproj_NorfairLavaquakeRocks_Func1(k, game);
            return;
        }
        8830095 => {
            Eproj_NorfairLavaquakeRocks_Func2(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn EprojPreInstr_NorfairLavaquakeRocks_Inner2(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    return (((*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16;
}
pub unsafe fn EprojPreInstr_NorfairLavaquakeRocks_Inner(mut k: uint16, game: &mut Game) {
    if EprojPreInstr_NorfairLavaquakeRocks_Inner2(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojPreInstr_NorfairLavaquakeRocks(mut k: uint16, game: &mut Game) {
    CallNorfairLavaquakeRocksFunc(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            | 0x860000 as ::core::ffi::c_int) as uint32,
        k,
        game,
    );
    EprojPreInstr_NorfairLavaquakeRocks_Inner(k, game);
}
pub unsafe fn Eproj_NorfairLavaquakeRocks_Func1(mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v3: int16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    v2 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - 2 as ::core::ffi::c_int) as int16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = v2 as uint16;
    if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        let mut n: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
        loop {
            v3 = (n + *((&raw mut game.g_ram as *mut uint8)
                .offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as int16;
            if (v3 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                v3 = 0 as int16;
            }
            let mut t: ::core::ffi::c_int = 8 as ::core::ffi::c_int * v3 as ::core::ffi::c_int
                + 4 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            AddToHiLo(
                ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as *mut uint16,
                ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as *mut uint16,
                *(game.g_rom.offset(
                    (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint32)
                    .offset((t >> 1 as ::core::ffi::c_int) as isize),
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = *(game.g_rom.offset(
                (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((t + 1 as ::core::ffi::c_int) as isize);
            n -= 1;
            if !(n != 0) {
                break;
            }
        }
        Eproj_NorfairLavaquakeRocks_Func3(k, game);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) =
            (0x86bc8f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Eproj_NorfairLavaquakeRocks_Func2(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = v2;
    if v2 as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 64 as uint16;
    }
    let mut n: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    loop {
        let mut t: ::core::ffi::c_int = 8 as ::core::ffi::c_int
            * (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - n
                + 1 as ::core::ffi::c_int)
            >> 1 as ::core::ffi::c_int;
        AddToHiLo(
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16,
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16,
            *(game.g_rom.offset(
                (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint32)
                .offset((t >> 1 as ::core::ffi::c_int) as isize),
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = *(game.g_rom.offset(
            (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((t + 1 as ::core::ffi::c_int) as isize);
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
    Eproj_NorfairLavaquakeRocks_Func3(k, game);
}
pub unsafe fn Eproj_NorfairLavaquakeRocks_Func3(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
static kEprojInit_ShaktoolAttackMiddleBackCircle_X: [int16; 8] = [
    0 as ::core::ffi::c_int as int16,
    12 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    12 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(12 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(12 as ::core::ffi::c_int) as int16,
];
static kEprojInit_ShaktoolAttackMiddleBackCircle_Y: [int16; 8] = [
    -(16 as ::core::ffi::c_int) as int16,
    -(12 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    12 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    12 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(12 as ::core::ffi::c_int) as int16,
];
unsafe fn EprojInit_BDA2(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*v2).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*v2).y_pos;
    let mut v3: uint16 = (2 as ::core::ffi::c_int
        * *(&raw mut (*v2).ai_var_D as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int) as uint16;
    let mut v4: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) =
        kSinCosTable8bit_Sext[(v4 + 64 as ::core::ffi::c_int) as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = kSinCosTable8bit_Sext[v4 as usize] as uint16;
    let mut v5: ::core::ffi::c_int = (v3 as ::core::ffi::c_int >> 5 as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let ref mut fresh11 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh11 = (*fresh11 as ::core::ffi::c_int
        + kEprojInit_ShaktoolAttackMiddleBackCircle_X[v5 as usize] as ::core::ffi::c_int)
        as uint16;
    let ref mut fresh12 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh12 = (*fresh12 as ::core::ffi::c_int
        + kEprojInit_ShaktoolAttackMiddleBackCircle_Y[v5 as usize] as ::core::ffi::c_int)
        as uint16;
}
unsafe fn EprojInit_ShaktoolAttackMiddleBackCircle(mut j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
            as *mut uint16);
    EprojInit_BDA2(j, game);
}
unsafe fn EprojInit_ShaktoolAttackFrontCircle(mut v1: uint16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(v1, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || EprojBlockCollisition_Vertical(v1, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojPreInstr_BE12(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        )
        != 0
    {
        EprojBlockCollisition_Horiz(k, game);
        EprojBlockCollisition_Vertical(k, game);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
    };
}
static kEprojInit_MotherBrainRoomTurrets_X: [int16; 12] = [
    0x398 as ::core::ffi::c_int as int16,
    0x348 as ::core::ffi::c_int as int16,
    0x328 as ::core::ffi::c_int as int16,
    0x2d8 as ::core::ffi::c_int as int16,
    0x288 as ::core::ffi::c_int as int16,
    0x268 as ::core::ffi::c_int as int16,
    0x218 as ::core::ffi::c_int as int16,
    0x1c8 as ::core::ffi::c_int as int16,
    0x1a8 as ::core::ffi::c_int as int16,
    0x158 as ::core::ffi::c_int as int16,
    0x108 as ::core::ffi::c_int as int16,
    0xe8 as ::core::ffi::c_int as int16,
];
static EprojInit_MotherBrainRoomTurrets_Y: [int16; 12] = [
    0x30 as ::core::ffi::c_int as int16,
    0x40 as ::core::ffi::c_int as int16,
    0x40 as ::core::ffi::c_int as int16,
    0x30 as ::core::ffi::c_int as int16,
    0x40 as ::core::ffi::c_int as int16,
    0x40 as ::core::ffi::c_int as int16,
    0x30 as ::core::ffi::c_int as int16,
    0x40 as ::core::ffi::c_int as int16,
    0x40 as ::core::ffi::c_int as int16,
    0x30 as ::core::ffi::c_int as int16,
    0x40 as ::core::ffi::c_int as int16,
    0x40 as ::core::ffi::c_int as int16,
];
unsafe fn EprojInit_MotherBrainRoomTurrets(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 1024 as uint16;
    let mut v2: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut v3: uint16 = (*(game.g_rom.offset(
        (((0x86bee1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86bee1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as ::core::ffi::c_int
        | 0x100 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = v3;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *(game.g_rom.offset(
        (((0x86beb9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86beb9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((2 as ::core::ffi::c_int * v3 as uint8 as ::core::ffi::c_int) as uint16
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    let mut v4: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = kEprojInit_MotherBrainRoomTurrets_X[v4 as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = EprojInit_MotherBrainRoomTurrets_Y[v4 as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *(game.g_rom.offset(
        (((0x86bec9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86bec9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v4 as isize);
    Eproj_SetXvelRandom(j, game);
    Eproj_SetYvelRandom(j, game);
}
static kEproj_MotherBrainRoomTurretBullets_X: [int16; 8] = [
    -(17 as ::core::ffi::c_int) as int16,
    -(12 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    12 as ::core::ffi::c_int as int16,
    17 as ::core::ffi::c_int as int16,
    12 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(12 as ::core::ffi::c_int) as int16,
];
static kEproj_MotherBrainRoomTurretBullets_Y: [int16; 8] = [
    -(9 as ::core::ffi::c_int) as int16,
    3 as ::core::ffi::c_int as int16,
    7 as ::core::ffi::c_int as int16,
    3 as ::core::ffi::c_int as int16,
    -(9 as ::core::ffi::c_int) as int16,
    -(19 as ::core::ffi::c_int) as int16,
    -(21 as ::core::ffi::c_int) as int16,
    -(19 as ::core::ffi::c_int) as int16,
];
static kEproj_MotherBrainRoomTurretBullets_Xvel: [int16; 8] = [
    -(704 as ::core::ffi::c_int) as int16,
    -(498 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    498 as ::core::ffi::c_int as int16,
    704 as ::core::ffi::c_int as int16,
    498 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(498 as ::core::ffi::c_int) as int16,
];
static kEproj_MotherBrainRoomTurretBullets_Yvel: [int16; 8] = [
    0 as ::core::ffi::c_int as int16,
    498 as ::core::ffi::c_int as int16,
    704 as ::core::ffi::c_int as int16,
    498 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(498 as ::core::ffi::c_int) as int16,
    -(704 as ::core::ffi::c_int) as int16,
    -(498 as ::core::ffi::c_int) as int16,
];
unsafe fn EprojInit_MotherBrainRoomTurretBullets(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 1024 as uint16;
    let mut v2: uint16 = (2 as ::core::ffi::c_int
        * *(((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            ) as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = v2;
    let mut v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut pt: Point16U = Point16U {
        x: kEproj_MotherBrainRoomTurretBullets_X[v3 as usize] as uint16,
        y: kEproj_MotherBrainRoomTurretBullets_Y[v3 as usize] as uint16,
    };
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = kEproj_MotherBrainRoomTurretBullets_Xvel[v3 as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = kEproj_MotherBrainRoomTurretBullets_Yvel[v3 as usize] as uint16;
    let mut v4: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (pt.x as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (pt.y as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize) as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojPreInstr_MotherBrainRoomTurrets(mut k: uint16, game: &mut Game) {
    if Eproj_MotherBrainRoomTurretBullets_CheckIfTurretOnScreen(k, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        if (*gRam7800_Default(0 as uint16, game)).var_1D != 0 {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        }
    } else if (*gRam7800_Default(0 as uint16, game)).var_1D != 0 {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        game.eproj_spawn_pt = Point16U {
            x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v5 as isize),
            y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v5 as isize),
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
            0xc as uint16,
            game,
        );
    } else {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh88 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        let fresh89 = *fresh88;
        *fresh88 = (*fresh88).wrapping_sub(1);
        let mut v2: bool = fresh89 as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
        if v2 {
            Eproj_SetXvelRandom(k, game);
            Eproj_MotherBrainRoomTurretBullets_Func2(k, game);
            let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) = *(game.g_rom.offset(
                (((0x86c040 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x86c040 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    *(((&raw mut game.g_ram as *mut uint8)
                        .offset(0x1a6f as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v3 as isize) as *mut uint16 as *mut uint8)
                        .offset(0 as ::core::ffi::c_int as isize) as isize,
                );
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) = 1 as uint16;
        }
        let mut v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh92 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize);
        let fresh93 = *fresh92;
        *fresh92 = (*fresh92).wrapping_sub(1);
        v2 = fresh93 as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
        if v2 {
            Eproj_SetYvelRandom(k, game);
            SpawnEprojWithRoomGfx(
                addr_kEproj_MotherBrainRoomTurretBullets as ::core::ffi::c_int as uint16,
                k,
                game,
            );
        }
    };
}
pub unsafe fn Eproj_MotherBrainRoomTurretBullets_Func2(mut v0: uint16, game: &mut Game) {
    let mut r20: uint8 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a6f as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int) as uint8;
    let mut r18: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a27 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    if *RomPtr_86(r18 as uint16_t, game)
        .offset((r20 as ::core::ffi::c_int & 7 as ::core::ffi::c_int) as isize)
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) = r20;
    } else {
        let mut v1: int8 = -(*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int) as int8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize) = v1 as uint8;
        let ref mut fresh94 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize);
        *fresh94 = (*fresh94 as ::core::ffi::c_int + v1 as ::core::ffi::c_int) as uint8;
    };
}
pub unsafe fn Eproj_SetXvelRandom(mut v0: uint16, game: &mut Game) {
    let mut Random: uint16 = NextRandom(game) as uint8 as uint16;
    if Random as uint8 as ::core::ffi::c_int - 32 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        Random = 32 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = Random;
}
pub unsafe fn Eproj_SetYvelRandom(mut v0: uint16, game: &mut Game) {
    let mut Random: uint16 = NextRandom(game) as uint8 as uint16;
    if Random as uint8 as ::core::ffi::c_int - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        Random = 128 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = Random;
}
pub unsafe fn Eproj_MotherBrainRoomTurretBullets_CheckIfTurretOnScreen(
    mut k: uint16,
    game: &mut Game,
) -> uint8 {
    let mut v2: int16 = 0;
    let mut v3: int16 = 0;
    let mut v4: int16 = 0;
    let mut v5: int16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as int16;
    let mut result: uint8 = 1 as uint8;
    if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        v3 = (v2 as ::core::ffi::c_int + 16 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16;
        if v3 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            if v3 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v4 = *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1a4b as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as int16;
                if v4 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                    v5 = (v4 as ::core::ffi::c_int + 4 as ::core::ffi::c_int
                        - *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x911 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int)
                        as int16;
                    if v5 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                        if v5 as ::core::ffi::c_int - 264 as ::core::ffi::c_int
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
pub unsafe fn EprojPreInstr_MotherBrainRoomTurretBullets(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh85 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1bd7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh85 = (*fresh85 as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int) as uint16;
    MoveEprojWithVelocity(k, game);
    if Ridley_Func_103(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
        game,
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x1991 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            ) = 0 as uint16;
    }
}
static kMotherBrainsBomb_Yaccel: [int16; 10] = [
    7 as ::core::ffi::c_int as int16,
    0x10 as ::core::ffi::c_int as int16,
    0x20 as ::core::ffi::c_int as int16,
    0x40 as ::core::ffi::c_int as int16,
    0x70 as ::core::ffi::c_int as int16,
    0xb0 as ::core::ffi::c_int as int16,
    0xf0 as ::core::ffi::c_int as int16,
    0x130 as ::core::ffi::c_int as int16,
    0x170 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
];
pub unsafe fn EprojPreInstr_MotherBrainBomb(mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    if MotherBrainBomb_Bomb_CollDetect_DoubleRet(k, game) != 0 {
        return;
    }
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut current_block_19: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize)
        != 0
    {
        let mut v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v5: uint16 =
            kMotherBrainsBomb_Yaccel[(*((&raw mut game.g_ram as *mut uint8)
                .offset(0x1b23 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v4 as isize) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as usize] as uint16;
        if v5 == 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v4 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v4 as isize) = 0 as uint16;
            let fresh81 = &raw mut (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x7800 as ::core::ffi::c_int as isize)
                as *mut ExtraEnemyRam7800)
                .offset(1 as ::core::ffi::c_int as isize))
            .kraid
            .kraid_mouth_flags;
            fresh81.write_unaligned(fresh81.read_unaligned().wrapping_sub(1));
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v4 as isize) = 0 as uint16;
            game.eproj_spawn_pt = Point16U {
                x: *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1a4b as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v4 as isize),
                y: *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1a93 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v4 as isize),
            };
            SpawnEprojWithRoomGfx(
                addr_stru_869650 as ::core::ffi::c_int as uint16,
                *(((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v4 as isize) as *mut uint16 as *mut uint8)
                    .offset(0 as ::core::ffi::c_int as isize) as uint16,
                game,
            );
            game.eproj_spawn_pt = Point16U {
                x: *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1a4b as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v4 as isize),
                y: *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1a93 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v4 as isize),
            };
            SpawnEprojWithRoomGfx(
                addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
                3 as uint16,
                game,
            );
            QueueSfx3_Max6(0x13 as uint16, game);
            return;
        }
        if MoveMotherBrainBomb(k, v5, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
            current_block_19 = 15976848397966268834;
        } else {
            current_block_19 = 7175849428784450219;
        }
    } else {
        v2 = (abs16(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize),
        ) as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int) as int16;
        if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            v2 = 0 as int16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (if *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            -(v2 as ::core::ffi::c_int)
        } else {
            v2 as ::core::ffi::c_int
        }) as uint16;
        if MoveMotherBrainBomb(k, 7 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
        {
            current_block_19 = 15976848397966268834;
        } else {
            current_block_19 = 7175849428784450219;
        }
    }
    match current_block_19 {
        15976848397966268834 => {
            let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let ref mut fresh82 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x1b23 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize);
            *fresh82 = (*fresh82 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        }
        _ => {}
    };
}
pub unsafe fn EprojInstr_SwitchJump(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    return *(epjp.offset(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int as isize,
    ) as *mut uint16) as uintptr_t as *const uint8;
}
pub unsafe fn EprojInstr_UserPalette0(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return epjp;
}
unsafe fn Eproj_CheckForBombCollisionWithRect(mut rect: Rect16U, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcd2 as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
    {
        return 0 as uint8;
    }
    let mut v1: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
    while v1 < 20 as ::core::ffi::c_int {
        let mut v2: ::core::ffi::c_int = v1 >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            & 0xf00 as ::core::ffi::c_int
            == 1280 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize)
                == 0
        {
            let mut x: uint16 = abs16(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int
                    - rect.x as ::core::ffi::c_int) as uint16,
            );
            if (x as ::core::ffi::c_int)
                < *((&raw mut game.g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int
                || ((x as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xbb4 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v2 as isize) as ::core::ffi::c_int) as uint16
                    as ::core::ffi::c_int)
                    < rect.w as ::core::ffi::c_int
            {
                let mut y: uint16 = abs16(
                    (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xb78 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v2 as isize) as ::core::ffi::c_int
                        - rect.y as ::core::ffi::c_int) as uint16,
                );
                if (y as ::core::ffi::c_int)
                    < *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xbc8 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v2 as isize) as ::core::ffi::c_int
                    || ((y as ::core::ffi::c_int
                        - *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xbc8 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v2 as isize) as ::core::ffi::c_int)
                        as uint16 as ::core::ffi::c_int)
                        < rect.h as ::core::ffi::c_int
                {
                    return 1 as uint8;
                }
            }
        }
        v1 += 2 as ::core::ffi::c_int;
    }
    return 0 as uint8;
}
unsafe fn Eproj_CheckForEnemyCollisionWithRect(
    mut k: uint16,
    mut rect: Rect16U,
    game: &mut Game,
) -> uint8 {
    let mut v1: *mut EnemyData = gEnemyData(k as ::core::ffi::c_int, game);
    return ((abs16(((*v1).x_pos as ::core::ffi::c_int - rect.x as ::core::ffi::c_int) as uint16)
        as ::core::ffi::c_int
        - (*v1).x_width as ::core::ffi::c_int)
        < rect.w as ::core::ffi::c_int
        && (abs16(((*v1).y_pos as ::core::ffi::c_int - rect.y as ::core::ffi::c_int) as uint16)
            as ::core::ffi::c_int
            - (*v1).y_height as ::core::ffi::c_int)
            < rect.h as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
}
unsafe fn Eproj_CheckForCollisionWithSamus(mut v0: uint16, game: &mut Game) -> uint8 {
    let mut rect: Rect16U = Eproj_GetCollDetectRect(v0, game);
    let mut v1: uint16 = abs16(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - rect.x as ::core::ffi::c_int) as uint16,
    );
    let mut v2: bool = (v1 as ::core::ffi::c_int)
        < *((&raw mut game.g_ram as *mut uint8).offset(0xafe as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int;
    let mut v3: uint16 = (v1 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xafe as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut result: uint8 = 0 as uint8;
    if v2 as ::core::ffi::c_int != 0 || (v3 as ::core::ffi::c_int) < rect.w as ::core::ffi::c_int {
        let mut v4: uint16 = abs16(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - rect.y as ::core::ffi::c_int) as uint16,
        );
        v2 = (v4 as ::core::ffi::c_int)
            < *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int;
        let mut v5: uint16 = (v4 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        if v2 as ::core::ffi::c_int != 0
            || (v5 as ::core::ffi::c_int) < rect.h as ::core::ffi::c_int
        {
            return 1 as uint8;
        }
    }
    return result;
}
pub unsafe fn Math_MultBySin(mut a: uint16, mut r18: uint16) -> uint16 {
    return Math_MultBySinCos(a, r18);
}
pub unsafe fn Math_MultByCos(mut a: uint16, mut r18: uint16) -> uint16 {
    return Math_MultBySinCos(
        a,
        (r18 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16,
    );
}
pub unsafe fn Math_MultBySinCos(mut r38: uint16, mut a: uint16) -> uint16 {
    let mut r46: uint16 = kSinCosTable8bit_Sext[((a as ::core::ffi::c_int
        & 0xff as ::core::ffi::c_int)
        + 64 as ::core::ffi::c_int) as usize] as uint16;
    let mut prod: uint16 =
        ((r38 as uint32).wrapping_mul(abs16(r46) as uint32) >> 8 as ::core::ffi::c_int) as uint16;
    return (if r46 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(prod as ::core::ffi::c_int)
    } else {
        prod as ::core::ffi::c_int
    }) as uint16;
}
pub unsafe fn Eproj_MotherBrainsBlueRingLasers(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 1024 as uint16;
    let mut r18: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = Math_MultBySin(0x450 as uint16, r18);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = Math_MultByCos(0x450 as uint16, r18);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .x_pos as ::core::ffi::c_int
        + 10 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .y_pos as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int) as uint16;
    sub_86C320(j, game);
}
pub unsafe fn sub_86C320(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .x_pos as ::core::ffi::c_int
        + 10 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .y_pos as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Eproj_MoveToBlueRingSpawnPosition(mut k: uint16, game: &mut Game) {
    let mut v5: int16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize)
        != 0
    {
        let ref mut fresh83 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh83 = (*fresh83).wrapping_sub(1);
        sub_86C320(k, game);
    } else {
        MoveEprojWithVelocity(k, game);
        let mut t: uint8 = CheckForCollisionWithShitroid_DoubleRet(k, game);
        if t as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
            return;
        }
        if t != 0 {
            let fresh84 = &raw mut (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x7800 as ::core::ffi::c_int as isize)
                as *mut ExtraEnemyRam7800)
                .offset(0 as ::core::ffi::c_int as isize))
            .kraid
            .field_28;
            fresh84.write_unaligned(fresh84.read_unaligned().wrapping_add(1));
            BlueRingContactExplosion(k, game);
            let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x7800 as ::core::ffi::c_int as isize)
                as *mut ExtraEnemyRam7800)
                .offset(1 as ::core::ffi::c_int as isize))
            .kraid
            .kraid_healths_8ths[4 as ::core::ffi::c_int as usize];
            (*gExtraEnemyRam7800(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x7800 as ::core::ffi::c_int as isize)
                    as *mut ExtraEnemyRam7800)
                    .offset(1 as ::core::ffi::c_int as isize))
                .kraid
                .kraid_healths_8ths[4 as ::core::ffi::c_int as usize]
                    as ::core::ffi::c_int,
             game))
            .kraid
            .kraid_healths_8ths[0 as ::core::ffi::c_int as usize] = 16 as uint16;
            let mut v4: *mut EnemyData = gEnemyData(v3 as ::core::ffi::c_int, game);
            v5 = ((*v4).health as ::core::ffi::c_int - 80 as ::core::ffi::c_int) as int16;
            if (v5 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                v5 = 0 as int16;
            }
            (*v4).health = v5 as uint16;
        } else if Eproj_CheckForCollisionWithSamus(k, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            BlueRingContactExplosion(k, game);
            Samus_DealDamage(SuitDamageDivision(0x50 as uint16, game), game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x18a8 as ::core::ffi::c_int as isize)
                as *mut uint16) = 96 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x18aa as ::core::ffi::c_int as isize)
                as *mut uint16) = 5 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa54 as ::core::ffi::c_int as isize)
                as *mut uint16) = ((*((&raw mut game.g_ram as *mut uint8)
                .offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1a4b as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                as uint16;
        } else if CheckForBlueRingCollisionWithRoom(k, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            Eproj_Earthqhake5(k, game);
        }
    };
}
pub unsafe fn CheckForCollisionWithShitroid_DoubleRet(mut v0: uint16, game: &mut Game) -> uint8 {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x7800 as ::core::ffi::c_int as isize)
        as *mut ExtraEnemyRam7800)
        .offset(1 as ::core::ffi::c_int as isize))
    .kraid
    .kraid_healths_8ths[4 as ::core::ffi::c_int as usize]
        == 0
    {
        return 0 as uint8;
    }
    if (*gEnemyData(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x7800 as ::core::ffi::c_int as isize)
            as *mut ExtraEnemyRam7800)
            .offset(1 as ::core::ffi::c_int as isize))
        .kraid
        .kraid_healths_8ths[4 as ::core::ffi::c_int as usize] as ::core::ffi::c_int,
        game,
    ))
    .health
        != 0
    {
        let mut rect: Rect16U = Eproj_GetCollDetectRect(v0, game);
        return Eproj_CheckForEnemyCollisionWithRect(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x7800 as ::core::ffi::c_int as isize)
                as *mut ExtraEnemyRam7800)
                .offset(1 as ::core::ffi::c_int as isize))
            .kraid
            .kraid_healths_8ths[4 as ::core::ffi::c_int as usize],
            rect,
            game,
        );
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return 0xff as uint8;
}
pub unsafe fn CheckForBlueRingCollisionWithRoom(mut k: uint16, game: &mut Game) -> uint8 {
    let mut v2: int16 = 0;
    let mut v3: int16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut result: uint8 = 1 as uint8;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - 32 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int)
            < 0xd8 as ::core::ffi::c_int
    {
        v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16;
        if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            v3 = (v2 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int16;
            if v3 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                if v3 as ::core::ffi::c_int - 248 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    return 0 as uint8;
                }
            }
        }
    }
    return result;
}
unsafe fn Eproj_GetCollDetectRect(mut k: uint16, game: &mut Game) -> Rect16U {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut rect: Rect16U = Rect16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
        w: *(((&raw mut game.g_ram as *mut uint8).offset(0x1bb3 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as uint16,
        h: *(((&raw mut game.g_ram as *mut uint8).offset(0x1bb3 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as uint16,
    };
    return rect;
}
pub unsafe fn Eproj_Earthqhake5(mut k: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) = 10 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = 5 as uint16;
    BlueRingContactExplosion(k, game);
}
pub unsafe fn BlueRingContactExplosion(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        3 as uint16,
        game,
    );
    QueueSfx3_Max6(0x13 as uint16, game);
    sub_86C42E(k, game);
}
pub unsafe fn sub_86C42E(mut k: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
}
unsafe fn EprojInit_MotherBrainBomb(mut j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(j as ::core::ffi::c_int as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as uint8;
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 224 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .x_pos as ::core::ffi::c_int
        + 12 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .y_pos as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 1024 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 112 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    let fresh10 = &raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x7800 as ::core::ffi::c_int as isize)
        as *mut ExtraEnemyRam7800)
        .offset(1 as ::core::ffi::c_int as isize))
    .kraid
    .kraid_mouth_flags;
    fresh10.write_unaligned(fresh10.read_unaligned().wrapping_add(1));
}
pub unsafe fn MotherBrainBomb_Bomb_CollDetect_DoubleRet(mut k: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut rect: Rect16U = Rect16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
        w: *(((&raw mut game.g_ram as *mut uint8).offset(0x1bb3 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as uint16,
        h: *(((&raw mut game.g_ram as *mut uint8).offset(0x1bb3 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as uint16,
    };
    if Eproj_CheckForBombCollisionWithRect(rect, game) == 0 {
        return 0 as uint8;
    }
    let fresh2 = &raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x7800 as ::core::ffi::c_int as isize)
        as *mut ExtraEnemyRam7800)
        .offset(1 as ::core::ffi::c_int as isize))
    .kraid
    .kraid_mouth_flags;
    fresh2.write_unaligned(fresh2.read_unaligned().wrapping_sub(1));
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        9 as uint16,
        game,
    );
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
    };
    SpawnEnemyDrops(
        addr_kEnemyDef_EC3F as ::core::ffi::c_int as uint16,
        k,
        0 as uint16,
        game,
    );
    return 1 as uint8;
}
pub unsafe fn MoveMotherBrainBomb(mut k: uint16, mut a: uint16, game: &mut Game) -> uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize);
    *fresh16 = (*fresh16 as ::core::ffi::c_int + a as ::core::ffi::c_int) as uint16;
    MoveEprojWithVelocity(k, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) as ::core::ffi::c_int
        - 240 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) =
            -(*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) as ::core::ffi::c_int
        - 208 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        return 0 as uint8;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 208 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (if *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        -(*((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int)
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
    }) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(512 as ::core::ffi::c_int) as uint16;
    return 1 as uint8;
}
pub unsafe fn sub_86C605(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_23 = 0 as uint16;
    (*E).mbn_var_25 = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 1024 as uint16;
    let ref mut fresh8 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh8 = ((*E).base.x_pos as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_24 = *fresh8;
    let mut x: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - (*E).mbn_var_24 as ::core::ffi::c_int) as uint16;
    let ref mut fresh9 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh9 = ((*E).base.y_pos as ::core::ffi::c_int - 48 as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_26 = *fresh9;
    let mut y: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - (*E).mbn_var_26 as ::core::ffi::c_int) as uint16;
    let mut r18: ::core::ffi::c_int = -(CalculateAngleFromXY(x, y, game) as ::core::ffi::c_int
        + 0x80 as ::core::ffi::c_int) as uint8
        as ::core::ffi::c_int;
    (*E).mbn_var_29 = r18 as uint16;
    (*E).mbn_var_27 = Math_MultBySin(0xc00 as uint16, r18 as uint16);
    (*E).mbn_var_28 = Math_MultByCos(0xc00 as uint16, r18 as uint16);
}
unsafe fn EprojInit_MotherBrainDeathBeemFired(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*E).mbn_var_24;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*E).mbn_var_23;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*E).mbn_var_26;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*E).mbn_var_25;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*E).mbn_var_27;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*E).mbn_var_28;
    MoveEprojWithVelocity(j, game);
    (*E).mbn_var_24 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    (*E).mbn_var_23 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a27 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    (*E).mbn_var_26 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    (*E).mbn_var_25 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a6f as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    let mut r18: uint16 = (*(&raw mut (*E).mbn_var_29 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        + NextRandom(game) as ::core::ffi::c_int) as uint8 as uint16;
    let mut rv: uint16 = NextRandom(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = Math_MultBySin(
        (rv as ::core::ffi::c_int & 0x700 as ::core::ffi::c_int) as uint16,
        r18,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = Math_MultByCos(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x700 as ::core::ffi::c_int) as uint16,
        r18,
    );
    MoveEprojWithVelocity(j, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - 34 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - 206 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - 238 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        game.eproj_spawn_pt = Point16U {
            x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize),
            y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize),
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
            0x1d as uint16,
            game,
        );
        QueueSfx3_Max6(0x13 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
            as *mut uint16) = 10 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
            as *mut uint16) = 5 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
    };
}
pub unsafe fn SpawnMotherBrainDeathBeam(mut x: uint16, game: &mut Game) {
    SpawnEprojWithRoomGfx(
        addr_kEproj_MotherBrainDeathBeamFired as ::core::ffi::c_int as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((x as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
        game,
    );
}
pub unsafe fn EprojPreInstr_MotherBrainRainbowBeam(mut k: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .y_pos;
}
unsafe fn EprojInit_MotherBrainRainbowBeam(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    EprojPreInstr_MotherBrainRainbowBeam(j, game);
}
static kEprojInit_MotherBrainsDrool: [int16; 12] = [
    6 as ::core::ffi::c_int as int16,
    0x14 as ::core::ffi::c_int as int16,
    0xe as ::core::ffi::c_int as int16,
    0x12 as ::core::ffi::c_int as int16,
    8 as ::core::ffi::c_int as int16,
    0x17 as ::core::ffi::c_int as int16,
    0xa as ::core::ffi::c_int as int16,
    0x13 as ::core::ffi::c_int as int16,
    0xb as ::core::ffi::c_int as int16,
    0x19 as ::core::ffi::c_int as int16,
    0xc as ::core::ffi::c_int as int16,
    0x12 as ::core::ffi::c_int as int16,
];
pub unsafe fn EprojPreInstr_C84D(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: ::core::ffi::c_int = (4 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int)
        as uint16 as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(1 as ::core::ffi::c_int as isize))
        .x_pos as ::core::ffi::c_int
            + kEprojInit_MotherBrainsDrool[v2 as usize] as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .y_pos as ::core::ffi::c_int
        + kEprojInit_MotherBrainsDrool[(v2 + 1 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
}
unsafe fn EprojInit_MotherBrainsDrool(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16);
    EprojPreInstr_C84D(j, game);
}
unsafe fn EprojInit_MotherBrainsDrool_Falling(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh79 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh79 = (*fresh79 as ::core::ffi::c_int + 12 as ::core::ffi::c_int) as uint16;
    if MoveEprojWithVelocityY(k, game) as ::core::ffi::c_int >= 0xd7 as ::core::ffi::c_int {
        let ref mut fresh80 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh80 = (*fresh80 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_off_86C8E1 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    }
}
pub unsafe fn EprojInstr_Add12ToY(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh27 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh27 = (*fresh27 as ::core::ffi::c_int + 12 as ::core::ffi::c_int) as uint16;
    return epjp;
}
pub unsafe fn EprojPreInit_MotherBrainsDeathExplosion_0(mut k: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .x_pos as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .y_pos as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
}
unsafe fn EprojInit_MotherBrainsDeathExplosion(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *(game.g_rom.offset(
        (((0x86c929 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86c929 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = game.eproj_spawn_pt.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = game.eproj_spawn_pt.y;
    EprojPreInit_MotherBrainsDeathExplosion_0(j, game);
}
unsafe fn EprojInit_MotherBrainsRainbowBeamExplosion(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = game.eproj_spawn_pt.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + game.eproj_spawn_pt.x as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = game.eproj_spawn_pt.y;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + game.eproj_spawn_pt.y as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojPreInstr_MotherBrainsRainbowBeamExplosion(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
}
unsafe fn EprojInit_MotherBrainEscapeDoorParticles(mut j: uint16, game: &mut Game) {
    static kEprojInit_MotherBrainEscapeDoorParticles_X: [int16; 16] = [
        0 as ::core::ffi::c_int as int16,
        -(0x20 as ::core::ffi::c_int) as int16,
        0 as ::core::ffi::c_int as int16,
        -(0x18 as ::core::ffi::c_int) as int16,
        0 as ::core::ffi::c_int as int16,
        -(0x10 as ::core::ffi::c_int) as int16,
        0 as ::core::ffi::c_int as int16,
        -(8 as ::core::ffi::c_int) as int16,
        0 as ::core::ffi::c_int as int16,
        0 as ::core::ffi::c_int as int16,
        0 as ::core::ffi::c_int as int16,
        8 as ::core::ffi::c_int as int16,
        0 as ::core::ffi::c_int as int16,
        0x10 as ::core::ffi::c_int as int16,
        0 as ::core::ffi::c_int as int16,
        0x18 as ::core::ffi::c_int as int16,
    ];
    static kEprojInit_MotherBrainEscapeDoorParticles_Xvel: [int16; 16] = [
        0x500 as ::core::ffi::c_int as int16,
        -(0x200 as ::core::ffi::c_int) as int16,
        0x500 as ::core::ffi::c_int as int16,
        -(0x100 as ::core::ffi::c_int) as int16,
        0x500 as ::core::ffi::c_int as int16,
        -(0x100 as ::core::ffi::c_int) as int16,
        0x500 as ::core::ffi::c_int as int16,
        -(0x80 as ::core::ffi::c_int) as int16,
        0x500 as ::core::ffi::c_int as int16,
        -(0x80 as ::core::ffi::c_int) as int16,
        0x500 as ::core::ffi::c_int as int16,
        0x80 as ::core::ffi::c_int as int16,
        0x500 as ::core::ffi::c_int as int16,
        -(0x100 as ::core::ffi::c_int) as int16,
        0x500 as ::core::ffi::c_int as int16,
        0x200 as ::core::ffi::c_int as int16,
    ];
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    let mut v2: ::core::ffi::c_int = 4 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (kEprojInit_MotherBrainEscapeDoorParticles_X[v2 as usize]
        as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (kEprojInit_MotherBrainEscapeDoorParticles_X
        [(v2 + 1 as ::core::ffi::c_int) as usize]
        as ::core::ffi::c_int
        + 128 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) =
        kEprojInit_MotherBrainEscapeDoorParticles_Xvel[v2 as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = kEprojInit_MotherBrainEscapeDoorParticles_Xvel
        [(v2 + 1 as ::core::ffi::c_int) as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 32 as uint16;
}
pub unsafe fn EprojPreInstr_MotherBrainsExplodedDoorParticles(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: int16 = (abs16(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
    ) as ::core::ffi::c_int
        - 16 as ::core::ffi::c_int) as int16;
    if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        v2 = 0 as int16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (if *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        -(v2 as ::core::ffi::c_int)
    } else {
        v2 as ::core::ffi::c_int
    }) as uint16;
    let ref mut fresh76 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh76 = (*fresh76 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    MoveEprojWithVelocity(k, game);
    let ref mut fresh77 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1aff as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh77 = (*fresh77).wrapping_sub(1);
    if *fresh77 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        let ref mut fresh78 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh78 = (*fresh78 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
        game.eproj_spawn_pt = Point16U {
            x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize),
            y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize),
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
            9 as uint16,
            game,
        );
    }
}
unsafe fn EprojInit_MotherBrainPurpleBreathBig(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .x_pos as ::core::ffi::c_int
        + 6 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .y_pos as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int) as uint16;
}
unsafe fn EprojInit_MotherBrainPurpleBreathSmall(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .x_pos as ::core::ffi::c_int
        + 6 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(1 as ::core::ffi::c_int as isize))
    .y_pos as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int) as uint16;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x7800 as ::core::ffi::c_int as isize)
        as *mut ExtraEnemyRam7800)
        .offset(1 as ::core::ffi::c_int as isize))
    .kraid
    .kraid_hurt_frame = 1 as uint16;
}
pub unsafe fn EprojInstr_MotherBrainPurpleBreathIsActive(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    (*((&raw mut game.g_ram as *mut uint8).offset(0x7800 as ::core::ffi::c_int as isize)
        as *mut ExtraEnemyRam7800)
        .offset(1 as ::core::ffi::c_int as isize))
    .kraid
    .kraid_hurt_frame = 0 as uint16;
    return epjp;
}
pub unsafe fn EprojPreInstr_TimeBombSetJapaneseText(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 192 as uint16;
}
unsafe fn EprojInit_TimeBombSetJapaneseText(mut j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    EprojPreInstr_TimeBombSetJapaneseText(j, game);
}
unsafe fn EprojInit_MotherBrainTubeFalling(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 3584 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = game.eproj_spawn_pt.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = game.eproj_spawn_pt.y;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) =
        (0x86cbea as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CallMotherBrainTubeFallingFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        8834026 => {
            MotherBrainTubeFallingFunc_GenerateExplosion(k, game);
            return;
        }
        8834056 => {
            MotherBrainTubeFallingFunc_Falling(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn EprojPreInstr_MotherBrainTubeFalling(mut k: uint16, game: &mut Game) {
    CallMotherBrainTubeFallingFunc(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            | 0x860000 as ::core::ffi::c_int) as uint32,
        k,
        game,
    );
}
pub unsafe fn MotherBrainTubeFallingFunc_GenerateExplosion(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
        y: (*((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int) as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        9 as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) =
        (0x86cc08 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    MotherBrainTubeFallingFunc_Falling(k, game);
}
pub unsafe fn MotherBrainTubeFallingFunc_Falling(mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh75 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh75 = (*fresh75 as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
    v2 = MoveEprojWithVelocityY(k, game) as int16;
    if v2 as ::core::ffi::c_int - 208 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        game.eproj_spawn_pt = Point16U {
            x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize),
            y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize),
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
            0xc as uint16,
            game,
        );
    }
}
static kEprojInit_MotherBrainGlassShatteringShard_X: [int16; 3] = [
    8 as ::core::ffi::c_int as int16,
    -(40 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
];
static kEprojInit_MotherBrainGlassShatteringShard_Y: [int16; 3] = [
    32 as ::core::ffi::c_int as int16,
    32 as ::core::ffi::c_int as int16,
    32 as ::core::ffi::c_int as int16,
];
unsafe fn EprojInit_MotherBrainGlassShatteringShard(mut j: uint16, game: &mut Game) {
    let mut v1: uint16 = (2 as ::core::ffi::c_int * NextRandom(game) as ::core::ffi::c_int
        & 0x1fe as ::core::ffi::c_int) as uint16;
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = v1;
    let mut v3: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        kSinCosTable8bit_Sext[(v3 + 64 as ::core::ffi::c_int) as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (4 as ::core::ffi::c_int
        * kSinCosTable8bit_Sext[v3 as usize] as ::core::ffi::c_int)
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *(game.g_rom.offset(
        (((0x86ce41 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86ce41 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((v1 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
                as uint16 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 1600 as uint16;
    CalculatePlmBlockCoords(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c27 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    let mut v4: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (kEprojInit_MotherBrainGlassShatteringShard_X[v4 as usize]
        as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x1c29 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (kEprojInit_MotherBrainGlassShatteringShard_Y[v4 as usize]
        as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x1c2b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
    let ref mut fresh5 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize);
    *fresh5 = (*fresh5 as ::core::ffi::c_int
        + ((NextRandom(game) as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
            - 8 as ::core::ffi::c_int)) as uint16;
    let ref mut fresh6 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize);
    *fresh6 = (*fresh6 as ::core::ffi::c_int
        + ((NextRandom(game) as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
            - 8 as ::core::ffi::c_int)) as uint16;
}
unsafe fn EprojInit_MotherBrainGlassShatteringSparkle(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + (NextRandom(game) as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
        - 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + (NextRandom(game) as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
        - 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 1600 as uint16;
}
pub unsafe fn EprojPreInstr_MotherBrainGlassShatteringShard(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1ab7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        amt as uint32,
    );
    amt = ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as int16 as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as int32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        amt as uint32,
    );
    let mut v6: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a6f as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
    } else {
        let ref mut fresh74 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh74 = (*fresh74 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
        if NextRandom(game) as ::core::ffi::c_int & 0x420 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            SpawnEprojWithRoomGfx(
                addr_kEproj_MotherBrainGlassShatteringSparkle as ::core::ffi::c_int as uint16,
                k,
                game,
            );
        }
    };
}
unsafe fn EprojInit_KiHunterAcidSpitCommon(mut k: uint16, mut j: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = ((*gEnemyData(k as ::core::ffi::c_int, game)).y_pos
        as ::core::ffi::c_int
        - 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
}
unsafe fn EprojInit_KiHunterAcidSpitLeft(mut j: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(768 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = ((*gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .x_pos as ::core::ffi::c_int
        - 22 as ::core::ffi::c_int) as uint16;
    EprojInit_KiHunterAcidSpitCommon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        j,
        game,
    );
}
unsafe fn EprojInit_KiHunterAcidSpitRight(mut j: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 768 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = ((*gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .x_pos as ::core::ffi::c_int
        + 22 as ::core::ffi::c_int) as uint16;
    EprojInit_KiHunterAcidSpitCommon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        j,
        game,
    );
}
pub unsafe fn sub_86CFD5(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) =
        (0x86cff8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let ref mut fresh73 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh73 = (*fresh73 as ::core::ffi::c_int - 19 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn sub_86CFE6(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) =
        (0x86cff8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let ref mut fresh72 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh72 = (*fresh72 as ::core::ffi::c_int + 19 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn sub_86CFF8(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_off_86CF56 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    } else if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    } else {
        let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v2;
        if v2 as ::core::ffi::c_int - 512 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        {
            v2 = 512 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v2;
    };
}
unsafe fn EprojInit_KagosBugs(mut j: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xe54 as ::core::ffi::c_int as isize)
        as *mut uint16);
    let mut v3: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).y_pos;
    let mut v4: uint16 = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0x5e5 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int)
        + 1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1bfb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = v4;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (v4 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        (0x86d0ca as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojPreInstr_KagosBugs_Func1(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1aff as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    if v2 != 0 {
        let mut v3: uint16 = (v2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v3;
        if v3 == 0 {
            QueueSfx2_Max6(0x6c as uint16, game);
        }
    }
}
pub unsafe fn EprojPreInstr_KagosBugs_Func2(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int,
        game,
    );
    let mut v3: uint16 = abs16(
        ((*v2).x_pos as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16,
    );
    if v3 as ::core::ffi::c_int - 23 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        let ref mut fresh26 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1bd7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh26 = (*fresh26 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn EprojPreInstr_KagosBugs(mut k: uint16, game: &mut Game) {
    EprojPreInstr_KagosBugs_Func1(k, game);
    EprojPreInstr_KagosBugs_Func2(k, game);
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1bfb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize)
        != 0
    {
        let ref mut fresh71 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1bfb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh71 = (*fresh71).wrapping_sub(1);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_word_86D052 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) =
            (0x86d0eb as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
static word_86D082: uint16 = 0xe0 as uint16;
pub unsafe fn EprojPreInstr_D0EC(mut k: uint16, game: &mut Game) {
    let mut current_block: u64;
    EprojPreInstr_KagosBugs_Func1(k, game);
    EprojPreInstr_KagosBugs_Func2(k, game);
    if EprojBlockCollisition_Horiz(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        current_block = 4781832012581146689;
    } else if EprojBlockCollisition_Vertical(k, game) != 0 {
        current_block = 4781832012581146689;
    } else {
        let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        let mut v3: bool = ((word_86D082 as ::core::ffi::c_int + v2 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (word_86D082 as ::core::ffi::c_int + v2 as ::core::ffi::c_int) as uint16;
        if !v3 {
            current_block = 15427931788582360902;
        } else {
            return;
        }
    }
    match current_block {
        4781832012581146689 => {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                256 as uint16;
        }
        _ => {}
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (0x86d128 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        addr_word_86D04A as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
}
pub unsafe fn EprojPreInstr_D128(mut v0: uint16, game: &mut Game) {
    EprojPreInstr_KagosBugs_Func1(v0, game);
    EprojPreInstr_KagosBugs_Func2(v0, game);
    if EprojBlockCollisition_Horiz(v0, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    } else if EprojBlockCollisition_Vertical(v0, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) =
            (0x86d0eb as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_word_86D03C as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    } else {
        let ref mut fresh70 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh70 = (*fresh70 as ::core::ffi::c_int + word_86D082 as ::core::ffi::c_int) as uint16;
    };
}
static g_word_86D086: uint16 = 0x200 as uint16;
static g_word_86D084: uint16 = 0x30 as uint16;
pub unsafe fn EprojInstr_D15C(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    EprojPreInstr_KagosBugs_Func1(k, game);
    EprojPreInstr_KagosBugs_Func2(k, game);
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut t: uint16 = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0x5e5 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x300 as ::core::ffi::c_int)
        + 2048 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(t as ::core::ffi::c_int) as uint16;
    let mut v3: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int,
        game,
    );
    let mut t2: uint16 = ((*v3).x_pos as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
    if (abs16(t2) as ::core::ffi::c_int - g_word_86D084 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        t = (if t2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            -(1 as ::core::ffi::c_int)
        } else {
            0 as ::core::ffi::c_int
        }) as uint16;
    }
    let mut v4: uint16 = g_word_86D086;
    if t as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int != 0 {
        v4 = -(v4 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = v4;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        (0x86d0ec as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return epjp;
}
pub unsafe fn EprojInstr_D1B6(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1bfb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int)
        + 1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) =
        (0x86d0ca as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return epjp;
}
pub unsafe fn EprojInstr_D1C7(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return epjp;
}
pub unsafe fn EprojInstr_D1CE(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
    };
    SpawnEnemyDrops(
        addr_kEnemyDef_E7FF as ::core::ffi::c_int as uint16,
        k,
        0 as uint16,
        game,
    );
    return epjp;
}
unsafe fn EprojInit_MaridiaFloatersSpikes(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    let mut v2: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*v2).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*v2).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16);
}
static word_86D21A: [int16; 8] = [
    0 as ::core::ffi::c_int as int16,
    32 as ::core::ffi::c_int as int16,
    32 as ::core::ffi::c_int as int16,
    32 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(32 as ::core::ffi::c_int) as int16,
    -(32 as ::core::ffi::c_int) as int16,
    -(32 as ::core::ffi::c_int) as int16,
];
static word_86D22A: [int16; 8] = [
    -(32 as ::core::ffi::c_int) as int16,
    -(32 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    32 as ::core::ffi::c_int as int16,
    32 as ::core::ffi::c_int as int16,
    32 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(32 as ::core::ffi::c_int) as int16,
];
pub unsafe fn EprojPreInstr_MaridiaFloatersSpikes(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh68 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1ab7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh68 = (*fresh68 as ::core::ffi::c_int
        + word_86D21A[*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1aff as ::core::ffi::c_int as isize) as *mut uint16)
            .offset(v1 as isize) as usize] as ::core::ffi::c_int) as uint16;
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || {
            let ref mut fresh69 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh69 = (*fresh69 as ::core::ffi::c_int
                + word_86D22A[*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1aff as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as usize] as ::core::ffi::c_int)
                as uint16;
            EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
                != 0
        }
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_off_86D218 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    }
}
unsafe fn EprojInit_WreckedShipRobotLaserCommon(mut k: uint16, mut j: uint16, game: &mut Game) {
    let mut v2: *mut EnemyData = gEnemyData(k as ::core::ffi::c_int, game);
    let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) =
        ((*v2).y_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = ((*v2).x_pos as ::core::ffi::c_int
        + (if *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            -(4 as ::core::ffi::c_int)
        } else {
            4 as ::core::ffi::c_int
        })) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = 0 as uint16;
    if ((*v2).x_width as ::core::ffi::c_int + (*v2).x_pos as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
        && (((*v2).x_pos as ::core::ffi::c_int
            - (*v2).x_width as ::core::ffi::c_int
            - 257 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        && ((*v2).y_height as ::core::ffi::c_int + (*v2).y_pos as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        && (((*v2).y_pos as ::core::ffi::c_int
            - (*v2).y_height as ::core::ffi::c_int
            - 224 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
    {
        QueueSfx2_Max6(0x67 as uint16, game);
    }
}
unsafe fn EprojInit_WreckedShipRobotLaserDown(mut j: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .ai_var_A;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    EprojInit_WreckedShipRobotLaserCommon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        j,
        game,
    );
}
unsafe fn EprojInit_WreckedShipRobotLaserHorizontal(mut j: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .ai_var_A;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    EprojInit_WreckedShipRobotLaserCommon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        j,
        game,
    );
}
unsafe fn EprojInit_WreckedShipRobotLaserUp(mut j: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .ai_var_A;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(128 as ::core::ffi::c_int) as uint16;
    EprojInit_WreckedShipRobotLaserCommon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        j,
        game,
    );
}
pub unsafe fn EprojPreInstr_WreckedShipRobotLaser(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojInstr_AssignNewN00bTubeShardVelocity(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    NextRandom(game);
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *(((&raw mut game.g_ram as *mut uint8)
        .offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 192 as uint16;
    return epjp;
}
pub unsafe fn EprojInstr_SetN00bTubeShardX(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b6b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = *(epjp as *mut uint16);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = (128 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int
            + 128 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b6b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = *(epjp.offset(2 as ::core::ffi::c_int as isize) as *mut uint16);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (epjp.offset_from(RomBankBase(0x86 as uint8, game))
        as ::core::ffi::c_long
        + 4 as ::core::ffi::c_long) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 1 as uint16;
    return ::core::ptr::null::<uint8>();
}
pub unsafe fn EprojInstr_D62A(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            -(4608 as ::core::ffi::c_int) as uint16;
    }
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b6b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *(epjp as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (epjp.offset_from(RomBankBase(0x86 as uint8, game))
        as ::core::ffi::c_long
        + 2 as ::core::ffi::c_long) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 1 as uint16;
    return ::core::ptr::null::<uint8>();
}
pub unsafe fn EprojInstr_SetXvelRandom(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    NextRandom(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(1 as ::core::ffi::c_int as isize) as *mut uint16);
    return epjp;
}
unsafe fn EprojInit_N00bTubeCrack(mut j: uint16, game: &mut Game) {
    CalculatePlmBlockCoords(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c27 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (16 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1c29 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        + 96 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (16 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1c2b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        + 48 as ::core::ffi::c_int) as uint16;
}
static kEprojInit_N00bTubeShards_X: [int16; 10] = [
    -(56 as ::core::ffi::c_int) as int16,
    -(64 as ::core::ffi::c_int) as int16,
    -(20 as ::core::ffi::c_int) as int16,
    -(40 as ::core::ffi::c_int) as int16,
    -(64 as ::core::ffi::c_int) as int16,
    -(48 as ::core::ffi::c_int) as int16,
    -(24 as ::core::ffi::c_int) as int16,
    -(40 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    -(8 as ::core::ffi::c_int) as int16,
];
static kEprojInit_N00bTubeShards_Y: [int16; 10] = [
    8 as ::core::ffi::c_int as int16,
    -(12 as ::core::ffi::c_int) as int16,
    -(26 as ::core::ffi::c_int) as int16,
    -(24 as ::core::ffi::c_int) as int16,
    -(32 as ::core::ffi::c_int) as int16,
    28 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    -(8 as ::core::ffi::c_int) as int16,
    -(24 as ::core::ffi::c_int) as int16,
    16 as ::core::ffi::c_int as int16,
];
static kEprojInit_N00bTubeShards_Xvel: [int16; 10] = [
    -(384 as ::core::ffi::c_int) as int16,
    -(384 as ::core::ffi::c_int) as int16,
    -(160 as ::core::ffi::c_int) as int16,
    -(288 as ::core::ffi::c_int) as int16,
    -(288 as ::core::ffi::c_int) as int16,
    -(320 as ::core::ffi::c_int) as int16,
    -(96 as ::core::ffi::c_int) as int16,
    -(352 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    -(64 as ::core::ffi::c_int) as int16,
];
static kEprojInit_N00bTubeShards_Yvel: [int16; 10] = [
    320 as ::core::ffi::c_int as int16,
    -(256 as ::core::ffi::c_int) as int16,
    -(416 as ::core::ffi::c_int) as int16,
    -(288 as ::core::ffi::c_int) as int16,
    -(288 as ::core::ffi::c_int) as int16,
    448 as ::core::ffi::c_int as int16,
    576 as ::core::ffi::c_int as int16,
    -(96 as ::core::ffi::c_int) as int16,
    -(288 as ::core::ffi::c_int) as int16,
    384 as ::core::ffi::c_int as int16,
];
unsafe fn EprojInit_N00bTubeShards(mut j: uint16, game: &mut Game) {
    CalculatePlmBlockCoords(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c27 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    let mut v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (kEprojInit_N00bTubeShards_X[v1 as usize] as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x1c29 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        + 96 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (kEprojInit_N00bTubeShards_Y[v1 as usize] as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x1c2b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        + 48 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *(game.g_rom.offset(
        (((0x86d760 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86d760 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = kEprojInit_N00bTubeShards_Xvel[v1 as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = kEprojInit_N00bTubeShards_Yvel[v1 as usize] as uint16;
}
static kEprojInit_N00bTubeReleasedAirBubbles_X: [uint16; 6] = [
    40 as ::core::ffi::c_int as uint16,
    80 as ::core::ffi::c_int as uint16,
    104 as ::core::ffi::c_int as uint16,
    120 as ::core::ffi::c_int as uint16,
    152 as ::core::ffi::c_int as uint16,
    184 as ::core::ffi::c_int as uint16,
];
static kEprojInit_N00bTubeReleasedAirBubbles_Y: [uint16; 6] = [
    80 as ::core::ffi::c_int as uint16,
    72 as ::core::ffi::c_int as uint16,
    84 as ::core::ffi::c_int as uint16,
    32 as ::core::ffi::c_int as uint16,
    64 as ::core::ffi::c_int as uint16,
    84 as ::core::ffi::c_int as uint16,
];
unsafe fn EprojInit_N00bTubeReleasedAirBubbles(mut j: uint16, game: &mut Game) {
    CalculatePlmBlockCoords(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c27 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    let mut v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (kEprojInit_N00bTubeReleasedAirBubbles_X[v1 as usize]
        as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x1c29 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (kEprojInit_N00bTubeReleasedAirBubbles_Y[v1 as usize]
        as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x1c2b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(1280 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn sub_86D7BF(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        != 0xee00 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = -(4608 as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
    };
}
pub unsafe fn sub_86D7DE(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (192 as ::core::ffi::c_int as int16 as int32) << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        amt as uint32,
    );
}
pub unsafe fn EprojPreInstr_N00bTubeShards(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1ab7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        amt as uint32,
    );
    amt = ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as int16 as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as int32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        amt as uint32,
    );
    Eproj_DeleteIfYposOutside(k, game);
}
pub unsafe fn sub_86D83D(mut k: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0x17e as ::core::ffi::c_int
        | 0x80 as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int;
    let mut v6: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1991 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = ((kSinCosTable8bit_Sext[(v2 + 64 as ::core::ffi::c_int) as usize]
        as ::core::ffi::c_int
        >> 2 as ::core::ffi::c_int) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v6 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v6 as isize) as *mut uint16,
        amt as uint32,
    );
    let ref mut fresh67 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1ab7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v6 as isize);
    *fresh67 = (*fresh67 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    amt = ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v6 as isize) as int16 as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as int32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v6 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v6 as isize) as *mut uint16,
        amt as uint32,
    );
    Eproj_DeleteIfYposOutside(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1991 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
}
pub unsafe fn sub_86D89F(mut k: uint16, game: &mut Game) {
    let mut v1: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1ab7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0x17e as ::core::ffi::c_int
        | 0x80 as ::core::ffi::c_int) as uint16;
    let mut v2: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = ((kSinCosTable8bit_Sext[(v2 + 64 as ::core::ffi::c_int) as usize]
        as ::core::ffi::c_int
        >> 2 as ::core::ffi::c_int) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let mut v5: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1991 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) as *mut uint16,
        amt as uint32,
    );
    let ref mut fresh66 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1ab7 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v5 as isize);
    *fresh66 = (*fresh66 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    sub_86D8DF(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1991 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
}
pub unsafe fn sub_86D8DF(mut k: uint16, game: &mut Game) {
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1adb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v5 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) as *mut uint16,
        amt as uint32,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v5 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v5 as isize);
}
pub unsafe fn sub_86D992(mut v0: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *(game.g_rom.offset(
        (((0x86d96a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86d96a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    let mut v3: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).x_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).y_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(512 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 512 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 12 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = -(384 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = 384 as uint16;
    }
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1(mut k: uint16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, game);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2(mut k: uint16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, game);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1(mut k: uint16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, game);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2(mut k: uint16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, game);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1(mut k: uint16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, game);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, game);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2(mut k: uint16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, game);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, game);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1(mut k: uint16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, game);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, game);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2(mut k: uint16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, game);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, game);
}
static kEprojPreInstr_SpikeShootingPlantSpikes: [Option<Func_Y_V>; 10] = unsafe {
    [
        Some(EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1 as unsafe fn(uint16, &mut Game) -> ()),
        Some(EprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1 as unsafe fn(uint16, &mut Game) -> ()),
        Some(EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2 as unsafe fn(uint16, &mut Game) -> ()),
        Some(EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1 as unsafe fn(uint16, &mut Game) -> ()),
        Some(EprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2 as unsafe fn(uint16, &mut Game) -> ()),
        Some(EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2 as unsafe fn(uint16, &mut Game) -> ()),
        Some(
            EprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1 as unsafe fn(uint16, &mut Game) -> (),
        ),
        Some(
            EprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1 as unsafe fn(uint16, &mut Game) -> (),
        ),
        Some(
            EprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2 as unsafe fn(uint16, &mut Game) -> (),
        ),
        Some(
            EprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2 as unsafe fn(uint16, &mut Game) -> (),
        ),
    ]
};
pub unsafe fn EprojPreInstrHelper_SpikeShootingPlantSpikes_Func1(mut k: uint16, game: &mut Game) {
    if EprojPreInstrHelper_SpikeShootingPlantSpikes_Func2(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes(mut k: uint16, game: &mut Game) {
    kEprojPreInstr_SpikeShootingPlantSpikes[(*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(k, game);
    EprojPreInstrHelper_SpikeShootingPlantSpikes_Func1(k, game);
}
pub unsafe fn EprojPreInstrHelper_SpikeShootingPlantSpikes_Func2(
    mut k: uint16,
    game: &mut Game,
) -> uint16 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    return (((*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16;
}
unsafe fn EprojInit_DBF2(mut j: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = addr_word_86DB0C as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        (0x86db62 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) =
            (0x86db8c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    let mut v3: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).x_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v3).y_pos as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).y_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(256 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 256 as uint16;
}
pub unsafe fn EprojPreInstr_DBF2_Func1(mut k: uint16, game: &mut Game) {
    if EprojPreInstrHelper_DBF2_Func2(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojPreInstr_DBF2(mut k: uint16, game: &mut Game) {
    CallEprojPreInstr(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            | 0x860000 as ::core::ffi::c_int) as uint32,
        k,
        game,
    );
    EprojPreInstr_DBF2_Func1(k, game);
}
pub unsafe fn EprojPreInstr_DBF2_MoveX1(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn EprojPreInstr_DBF2_MoveX2(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn EprojPreInstrHelper_DBF2_Func2(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    return (((*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16;
}
pub unsafe fn EprojInstr_DC5A(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 12288 as uint16;
    return epjp;
}
pub unsafe fn EprojInstr_SpawnEnemyDrops_0(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize),
    };
    SpawnEnemyDrops(
        addr_kEnemyDef_DF7F as ::core::ffi::c_int as uint16,
        k,
        0 as uint16,
        game,
    );
    return epjp;
}
pub unsafe fn EprojInstr_SpawnSporesEproj(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize),
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_Spores as ::core::ffi::c_int as uint16,
        0 as uint16,
        game,
    );
    return epjp;
}
unsafe fn EprojInit_Spores(mut j: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize);
    *fresh4 = game.eproj_spawn_pt.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *fresh4;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = game.eproj_spawn_pt.y;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 512 as uint16;
}
static word_86DCB9: [int16; 5] = [
    -(64 as ::core::ffi::c_int) as int16,
    -(56 as ::core::ffi::c_int) as int16,
    -(48 as ::core::ffi::c_int) as int16,
    -(40 as ::core::ffi::c_int) as int16,
    -(32 as ::core::ffi::c_int) as int16,
];
unsafe fn EprojInit_SporeSpawnStalk(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .y_pos as ::core::ffi::c_int
        + word_86DCB9[*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1993 as ::core::ffi::c_int as isize) as *mut uint16)
            as usize] as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .x_pos;
}
pub unsafe fn sub_86DCC3(mut v0: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = ((*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .y_pos as ::core::ffi::c_int
        - 96 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .x_pos;
}
static kEprojInit_SporeSpawners_X: [int16; 4] = [
    0x20 as ::core::ffi::c_int as int16,
    0x60 as ::core::ffi::c_int as int16,
    0xa0 as ::core::ffi::c_int as int16,
    0xe0 as ::core::ffi::c_int as int16,
];
unsafe fn EprojInit_SporeSpawners(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = kEprojInit_SporeSpawners_X[*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16) as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 520 as uint16;
}
pub unsafe fn EprojPreInstr_Spores(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = *(((&raw mut game.g_ram as *mut uint8)
        .offset(0x1aff as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as uint16;
    let mut r18: uint16 = SignExtend8(
        *(game.g_rom.offset(
            (((0x86dd6c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x86dd6c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset(
                *(((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as *mut uint16 as *mut uint8)
                    .offset(0 as ::core::ffi::c_int as isize) as isize,
            ) as uint16,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        r18 = -(r18 as ::core::ffi::c_int) as uint16;
    }
    let ref mut fresh65 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh65 = (*fresh65 as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
    r18 = SignExtend8(
        *(game.g_rom.offset(
            (((0x86dd6c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x86dd6c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset((v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as uint16,
    );
    let mut v3: uint16 = (r18 as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
        + r18 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = v3;
    if v3 as ::core::ffi::c_int - 768 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) =
        (*(((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as uint8 as uint16;
}
pub unsafe fn EprojPreInstr_SporeSpawners(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize)
            == 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = addr_word_86DC06 as ::core::ffi::c_int as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) =
                (NextRandom(game) as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int) as uint16;
        }
        let ref mut fresh64 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh64 = (*fresh64).wrapping_sub(1);
    }
}
unsafe fn EprojInit_NamiFuneFireball(mut j: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = addr_word_86DE96 as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        (0x86df40 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = addr_word_86DEA6 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) =
            (0x86df6a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    let mut v3: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).x_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).y_subpos;
    if (*v3).parameter_1 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize);
        *fresh3 = (*fresh3 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    }
    let mut v4: ::core::ffi::c_int = (4 as ::core::ffi::c_int
        * *(&raw mut (*v3).parameter_2 as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *(game.g_rom.offset(
        (((0x86deb6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86deb6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v4 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *(game.g_rom.offset(
        (((0x86deb6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86deb6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v4 + 1 as ::core::ffi::c_int) as isize);
}
unsafe fn EprojInit_NamiFuneFireball_MoveX1(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
unsafe fn EprojInit_NamiFuneFireball_MoveX2(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn CallNamiFuneFireballFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        8838976 => {
            EprojInit_NamiFuneFireball_MoveX1(k, game);
            return;
        }
        8839018 => {
            EprojInit_NamiFuneFireball_MoveX2(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn EprojPreInstr_NamiFuneFireball(mut v0: uint16, game: &mut Game) {
    CallNamiFuneFireballFunc(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            | 0x860000 as ::core::ffi::c_int) as uint32,
        v0,
        game,
    );
    Eproj_NamiFuneFireball_After(v0, game);
}
pub unsafe fn Eproj_NamiFuneFireball_After(mut v0: uint16, game: &mut Game) {
    if EprojPreInstrHelper_DBF2_Func2(v0, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn sub_86DFA0(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    return (((*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16;
}
pub unsafe fn EprojInstr_DFEA(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
    };
    SpawnEnemyDrops(
        addr_kEnemyDef_E83F as ::core::ffi::c_int as uint16,
        k,
        0 as uint16,
        game,
    );
    return epjp;
}
unsafe fn EprojInit_LavaThrownByLavaman(mut j: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = addr_word_86DFD8 as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        (0x86e050 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = addr_word_86DFDE as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) =
            (0x86e07a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    let mut v3: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).x_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v3).y_pos as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).y_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = -(768 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 768 as uint16;
}
pub unsafe fn CallLavamanFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        8839248 => {
            Eproj_LavaThrownByLavaman_MoveX1(k, game);
            return;
        }
        8839290 => {
            Eproj_LavaThrownByLavaman_MoveX2(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn sub_86E049(mut v0: uint16, game: &mut Game) {
    CallLavamanFunc(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            | 0x860000 as ::core::ffi::c_int) as uint32,
        v0,
        game,
    );
    sub_86E0A4(v0, game);
}
pub unsafe fn Eproj_LavaThrownByLavaman_MoveX1(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn Eproj_LavaThrownByLavaman_MoveX2(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn sub_86E0A4(mut v0: uint16, game: &mut Game) {
    if sub_86E0B0(v0, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn sub_86E0B0(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    return (((*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16;
}
unsafe fn EprojInit_DustCloudOrExplosion(mut v0: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *(game.g_rom.offset(
        (((0x86e42c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86e42c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = game.eproj_spawn_pt.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = game.eproj_spawn_pt.y;
}
unsafe fn EprojInit_EyeDoorSmoke(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = *(game.g_rom.offset(
        (((0x86e42c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86e42c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8 as isize,
        );
    let mut v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut x: uint16 = (*(game.g_rom.offset(
        (((0x86e47e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86e47e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v2 + 2 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        + (*(game.g_rom.offset(
            (((0x86e47e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x86e47e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)) as uint16;
    let mut y: uint16 = (*(game.g_rom.offset(
        (((0x86e47e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86e47e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v2 + 3 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        + (*(game.g_rom.offset(
            (((0x86e47e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x86e47e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int)) as uint16;
    CalculatePlmBlockCoords(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c27 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (x as ::core::ffi::c_int
        + 8 as ::core::ffi::c_int
            * (2 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1c29 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int)) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (y as ::core::ffi::c_int
        + 8 as ::core::ffi::c_int
            * (2 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1c2b as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int)) as uint16;
    NextRandom(game);
}
pub unsafe fn EprojPreInstr_DustCloudOrExplosion(mut k: uint16, game: &mut Game) {
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojInstr_SetYVel(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(epjp as *mut uint16);
    return epjp.offset(2 as ::core::ffi::c_int as isize);
}
unsafe fn EprojInit_SpawnedGate_Common(mut j: uint16, mut v1: uint16, game: &mut Game) {
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1c27 as ::core::ffi::c_int as isize) as *mut uint16);
    CalculatePlmBlockCoords(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c27 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1c87 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (16 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1c29 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (v1 as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0x1c2b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
}
unsafe fn EprojInit_SpawnedShotGate(mut j: uint16, game: &mut Game) {
    EprojInit_SpawnedGate_Common(j, 0 as uint16, game);
}
unsafe fn EprojInit_ClosedDownwardsShotGate(mut j: uint16, game: &mut Game) {
    EprojInit_SpawnedGate_Common(j, 0x40 as uint16, game);
}
unsafe fn EprojInit_ClosedUpwardsShotGate(mut j: uint16, game: &mut Game) {
    EprojInit_SpawnedGate_Common(j, -(0x40 as ::core::ffi::c_int) as uint16, game);
}
pub unsafe fn EprojPreInstr_E605(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x19df as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + abs16(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize),
        ) as ::core::ffi::c_int) as uint16;
    if v2 as ::core::ffi::c_int >= 0x1000 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
        let ref mut fresh60 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh60 = (*fresh60).wrapping_add(1);
        let ref mut fresh61 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh61 = (*fresh61).wrapping_add(1);
        v2 = 0 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x19df as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = v2;
    let ref mut fresh62 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a6f as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh62 = (*fresh62 as ::core::ffi::c_int
        + ((*(((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int)) as uint16;
    let ref mut fresh63 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh63 = (*fresh63 as ::core::ffi::c_int
        + *(((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as int8 as ::core::ffi::c_int) as uint16;
}
unsafe fn EprojInit_SaveStationElectricity(mut j: uint16, game: &mut Game) {
    CalculatePlmBlockCoords(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c27 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (16 as ::core::ffi::c_int
        * (*((&raw mut game.g_ram as *mut uint8).offset(0x1c29 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int)) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (16 as ::core::ffi::c_int
        * (*((&raw mut game.g_ram as *mut uint8).offset(0x1c2b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int)) as uint16;
}
unsafe fn CheckIfEprojIsOffScreen(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - (*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 256 as ::core::ffi::c_int)) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
            && (*((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
        {
            if ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + 256 as ::core::ffi::c_int)) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                return 0 as uint16;
            }
        }
    }
    return 1 as uint16;
}
pub unsafe fn Eproj_FuncE722(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v2: int16 = 0;
    let mut v3: int16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as int16;
    let mut result: uint16 = 1 as uint16;
    if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        if v2 as ::core::ffi::c_int - 512 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        {
            v3 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as int16;
            if v3 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                if v3 as ::core::ffi::c_int - 512 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    return 0 as uint16;
                }
            }
        }
    }
    return result;
}
pub unsafe fn Eproj_FuncE73E_MoveXY(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x97dc as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + 64 as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        AddToHiLo(
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16,
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16,
            -(((*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1aff as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int) as int32) as uint32,
        );
    } else {
        AddToHiLo(
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16,
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16,
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1aff as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as uint32,
        );
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x97dc as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + 128 as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        AddToHiLo(
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16,
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16,
            -(((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1b23 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int) as int32) as uint32,
        );
    } else {
        AddToHiLo(
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16,
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16,
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1b23 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as uint32,
        );
    };
}
pub unsafe fn Eproj_AngleToSamus(mut j: uint16, mut r18: uint16, mut r20: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = r18;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = r20;
    let mut v2: uint16 = (64 as ::core::ffi::c_int
        - CalculateAngleFromXY(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1a4b as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int) as uint16,
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1a93 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int) as uint16,
            game,
        ) as ::core::ffi::c_int) as uint8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x97dc as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = v2;
    let mut pt: Point32 = ConvertAngleToXy(
        v2,
        *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (pt.x >> 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = pt.x as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (pt.y >> 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = pt.y as uint16;
}
unsafe fn EprojInit_BotwoonsBody(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    let mut v2: uint16 = (if (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .ai_var_A as ::core::ffi::c_int
        != 0
    {
        16 as ::core::ffi::c_int
    } else {
        48 as ::core::ffi::c_int
    }) as uint16;
    let mut v3: uint16 = *(game.g_rom.offset(
        (((0x86e9f1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86e9f1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = v3;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = v3;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = v2;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) =
        (0x86ea98 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut Ex: *mut ExtraEnemyRam7800 = gExtraEnemyRam7800(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
            as *mut EnemyData)
            .offset(0 as ::core::ffi::c_int as isize))
        .ai_var_A as ::core::ffi::c_int,
        game,
    );
    (*Ex).kraid.kraid_next = j;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf380 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 2 as uint16;
    (*Ex).kraid.kraid_healths_4ths[0 as ::core::ffi::c_int as usize] = 1 as uint16;
}
pub unsafe fn CallBotwoonEprojFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        8841880 => {
            Eproj_BotwoonsBody_Main(k, game);
            return;
        }
        8841972 => {
            Eproj_BotwonsBodyFunction_Dying(k, game);
            return;
        }
        8841988 => {
            Eproj_BotwonsBodyFunction_Dying2(k, game);
            return;
        }
        8842015 => {
            Eproj_BotwonsBodyFunction_DyingFalling(k, game);
            return;
        }
        8842131 => return,
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn EprojPreInstr_BotwoonsBody(mut k: uint16, game: &mut Game) {
    if *((&raw mut (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x8000 as ::core::ffi::c_int as isize)
        as *mut ExtraEnemyRam8000)
        .offset(0 as ::core::ffi::c_int as isize))
    .pad as *mut uint8)
        .offset(32 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16)
        != 0
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            == 0x86ea98 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) =
                (0x86eaf4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
    CallBotwoonEprojFunc(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            | 0x860000 as ::core::ffi::c_int) as uint32,
        k,
        game,
    );
}
pub unsafe fn Eproj_BotwoonsBody_Main(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = *(game.g_rom.offset(
        (((0x86e9f1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86e9f1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    if v2 as ::core::ffi::c_int
        != *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v2;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v2;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    }
    Eproj_BotwoonsBodyHurtFlashHandling1(k, game);
}
pub unsafe fn Eproj_BotwoonsBodyHurtFlashHandling1(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh58 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x19bb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh58 = (*fresh58 as ::core::ffi::c_int | 0xe00 as ::core::ffi::c_int) as uint16;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .flash_timer
        != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xe44 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 2 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let ref mut fresh59 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x19bb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh59 = (*fresh59 as ::core::ffi::c_int & 0xf1ff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn Eproj_BotwoonsBodyHurtFlashHandling2(mut j: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh55 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x19bb as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh55 = (*fresh55 as ::core::ffi::c_int | 0xe00 as ::core::ffi::c_int) as uint16;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xf78 as ::core::ffi::c_int as isize)
        as *mut EnemyData)
        .offset(0 as ::core::ffi::c_int as isize))
    .flash_timer
        != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xe44 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 2 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let ref mut fresh56 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x19bb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh56 = (*fresh56 as ::core::ffi::c_int & 0xf1ff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn Eproj_BotwonsBodyFunction_Dying(mut v0: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) =
        (4 as ::core::ffi::c_int * v0 as ::core::ffi::c_int + 96 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0xeb04 as uint16;
    Eproj_BotwonsBodyFunction_Dying2(v0, game);
}
pub unsafe fn Eproj_BotwonsBodyFunction_Dying2(mut v0: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh57 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1aff as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh57 = (*fresh57).wrapping_add(1);
    if *fresh57 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) =
            (0x86eb1f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    Eproj_BotwoonsBodyHurtFlashHandling1(v0, game);
}
pub unsafe fn Eproj_BotwonsBodyFunction_DyingFalling(mut v0: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v3: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        *(game.g_rom.offset(
            (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint32)
            .offset((v3 >> 1 as ::core::ffi::c_int) as isize),
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - 200 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh54 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v7 as isize);
        *fresh54 = (*fresh54 as ::core::ffi::c_int + 192 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v7 as isize) = 0 as uint16;
        Eproj_BotwoonsBodyHurtFlashHandling2(v0, game);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 200 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) =
            (0x86eb93 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = addr_word_86E208 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 2560 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
        QueueSmallExplosionSfx(game);
        if v0 as ::core::ffi::c_int == 10 as ::core::ffi::c_int {
            *((&raw mut (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x8800 as ::core::ffi::c_int as isize)
                as *mut ExtraEnemyRam8800)
                .offset(0 as ::core::ffi::c_int as isize))
            .pad as *mut uint8)
                .offset(62 as ::core::ffi::c_int as isize) as *mut uint8
                as *mut uint16) = 1 as uint16;
        }
    };
}
pub unsafe fn QueueSmallExplosionSfx(game: &mut Game) {
    QueueSfx2_Max6(0x24 as uint16, game);
}
unsafe fn EprojInit_BotwoonsSpit(mut j: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v1).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v1).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = addr_word_86EBAE as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x97dc as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x60b as ::core::ffi::c_int as isize) as uint16;
    let mut pt: Point32 = ConvertAngleToXy(
        *(&raw mut game.g_ram as *mut uint8).offset(0x60b as ::core::ffi::c_int as isize) as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
            as *mut uint16),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (pt.x >> 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = pt.x as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (pt.y >> 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = pt.y as uint16;
}
pub unsafe fn EprojPreInstr_BotwoonsSpit(mut k: uint16, game: &mut Game) {
    Eproj_FuncE73E_MoveXY(k, game);
    sub_86EC0C(k, game);
}
pub unsafe fn sub_86EC0C(mut k: uint16, game: &mut Game) {
    if sub_86EC18(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn sub_86EC18(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    return (((*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16;
}
unsafe fn EprojInit_YappingMawsBody(mut j: uint16, game: &mut Game) {
    let mut v2: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (*v2).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = (*v2).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v3 as isize) = addr_word_86EC5C as ::core::ffi::c_int as uint16;
    if (*v2).parameter_2 == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = addr_word_86EC56 as ::core::ffi::c_int as uint16;
    }
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x8800 as ::core::ffi::c_int as isize)
        as *mut ExtraEnemyRam8800)
        .offset(0 as ::core::ffi::c_int as isize))
    .pad as *mut uint8)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int
                    * *((&raw mut (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0x8800 as ::core::ffi::c_int as isize)
                        as *mut ExtraEnemyRam8800)
                        .offset(0 as ::core::ffi::c_int as isize))
                    .pad as *mut uint8)
                        .offset(
                            (*((&raw mut game.g_ram as *mut uint8)
                                .offset(0xe54 as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int
                                + 8 as ::core::ffi::c_int) as isize,
                        ) as *mut uint8 as *mut uint16) as ::core::ffi::c_int) as uint16
                as isize,
        ) as *mut uint8 as *mut uint16) = j;
}
pub unsafe fn EprojInstr_ECE3(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut x: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize) as ::core::ffi::c_int
        + (NextRandom(game) as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int)
        - 32 as ::core::ffi::c_int) as uint16;
    let mut y: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize) as ::core::ffi::c_int
        + ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x3f00 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int)
        - 32 as ::core::ffi::c_int) as uint16;
    CreateSpriteAtPos(x, y, *(epjp as *mut uint16), 0 as uint16, game);
    return epjp.offset(2 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_ED17(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut x: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a4b as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize) as ::core::ffi::c_int
        + (NextRandom(game) as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
        - 16 as ::core::ffi::c_int) as uint16;
    let mut y: uint16 = (*((&raw mut game.g_ram as *mut uint8)
        .offset(0x1a93 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize) as ::core::ffi::c_int
        + ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x1f00 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int)
        - 16 as ::core::ffi::c_int) as uint16;
    CreateSpriteAtPos(x, y, *(epjp as *mut uint16), 0 as uint16, game);
    return epjp.offset(2 as ::core::ffi::c_int as isize);
}
pub unsafe fn EprojInstr_QueueSfx2_9(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max1(9 as uint16, game);
    return epjp;
}
pub unsafe fn EprojInstr_QueueSfx2_24(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max1(0x24 as uint16, game);
    return epjp;
}
pub unsafe fn EprojInstr_QueueSfx2_B(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max1(0xb as uint16, game);
    return epjp;
}
pub unsafe fn EprojInstr_EEAF(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v2: uint16 = RandomDropRoutine(k, game);
    if k as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        && v2 as ::core::ffi::c_int - 6 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    {
        let mut v3: uint16 = (2 as ::core::ffi::c_int * v2 as ::core::ffi::c_int) as uint16;
        let mut v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize) = v3;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize) = *(game.g_rom.offset(
            (((0x86ef04 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x86ef04 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize) = 400 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize) =
            (0x86efe0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let ref mut fresh25 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1bd7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize);
        *fresh25 = (*fresh25 as ::core::ffi::c_int & !(0x4000 as ::core::ffi::c_int)) as uint16;
        return *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize) as uintptr_t as *const uint8;
    } else {
        let mut v6: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v6 as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v6 as isize) = 12288 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v6 as isize) =
            (0x86efdf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v6 as isize) = 0xeca3 as uint16;
        return 0xeca3 as ::core::ffi::c_int as uintptr_t as *const uint8;
    };
}
pub unsafe fn EprojInstr_HandleRespawningEnemy(
    mut k: uint16,
    mut epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xf410 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) as int16
        as ::core::ffi::c_int
        <= -(2 as ::core::ffi::c_int)
    {
        RespawnEnemy(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xf410 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                & 0x7fff as ::core::ffi::c_int) as uint16,
            game,
        );
    }
    return epjp;
}
unsafe fn EprojInit_F337(mut j: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = game.eproj_spawn_pt.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = game.eproj_spawn_pt.y;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf3c8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = game.eproj_spawn_varE24;
    let mut v3: uint16 = RandomDropRoutine(j, game);
    if v3 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        && v3 as ::core::ffi::c_int - 6 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    {
        let mut v4: uint16 = (2 as ::core::ffi::c_int * v3 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = v4;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = *(game.g_rom.offset(
            (((0x86ef04 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x86ef04 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = 400 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xf410 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            -(1 as ::core::ffi::c_int) as uint16;
    } else {
        let mut v5: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) = addr_word_86ECA3 as ::core::ffi::c_int as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) = 12288 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v5 as isize) =
            (0x86efdf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
unsafe fn EprojInit_EnemyDeathExplosion(mut j: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v1).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v1).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf410 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xe54 as ::core::ffi::c_int as isize)
        as *mut uint16);
    if (*v1).properties as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xf410 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = (*((&raw mut game.g_ram as *mut uint8)
            .offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            | 0x8000 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xf3c8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v1).enemy_ptr;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *(game.g_rom.offset(
        (((0x86efd5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86efd5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1993 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 1 as uint16;
}
static off_86F0AD: [Option<Func_V>; 7] = unsafe {
    [
        None,
        Some(Eproj_Pickup_SmallHealth as unsafe fn(&mut Game) -> ()),
        Some(Eproj_Pickup_BigHealth as unsafe fn(&mut Game) -> ()),
        Some(Eproj_Pickup_PowerBombs as unsafe fn(&mut Game) -> ()),
        Some(Eproj_Pickup_Missiles as unsafe fn(&mut Game) -> ()),
        Some(Eproj_Pickup_SuperMissiles as unsafe fn(&mut Game) -> ()),
        None,
    ]
};
pub unsafe fn EprojPreInstr_Pickup(mut k: uint16, game: &mut Game) {
    let mut v2: uint16 = 0;
    let mut v3: uint16 = 0;
    let mut current_block: u64;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh53 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x1b23 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh53 = (*fresh53).wrapping_sub(1);
    if !(*fresh53 == 0) {
        if CallSomeSamusCode(0xd as uint16, game) != 0 {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - 384 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v2 = 0;
                v3 = 0;
                v2 = abs16(
                    (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0x1a4b as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int
                        - *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xd08 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int) as uint16,
                );
                if v2 as ::core::ffi::c_int - 16 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    v3 = abs16(
                        (*((&raw mut game.g_ram as *mut uint8)
                            .offset(0x1a93 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v1 as isize) as ::core::ffi::c_int
                            - *((&raw mut game.g_ram as *mut uint8)
                                .offset(0xd0c as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int)
                            as uint16,
                    );
                    if v3 as ::core::ffi::c_int - 16 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0
                    {
                        off_86F0AD[(*((&raw mut game.g_ram as *mut uint8)
                            .offset(0x1aff as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v1 as isize)
                            as ::core::ffi::c_int
                            >> 1 as ::core::ffi::c_int)
                            as usize]
                            .expect("non-null function pointer")(game);
                        current_block = 4906268039856690917;
                    } else {
                        current_block = 11050875288958768710;
                    }
                } else {
                    current_block = 11050875288958768710;
                }
            } else {
                current_block = 11050875288958768710;
            }
        } else {
            current_block = 11050875288958768710;
        }
        match current_block {
            4906268039856690917 => {}
            _ => {
                let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                let mut varE20: uint16 =
                    *(((&raw mut game.g_ram as *mut uint8)
                        .offset(0x1bb3 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v5 as isize) as *mut uint16 as *mut uint8)
                        .offset(0 as ::core::ffi::c_int as isize) as uint16;
                let mut varE22: uint16 = *(((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1bb3 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v5 as isize) as *mut uint16
                    as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ) as uint16;
                let mut v6: uint16 = abs16(
                    (*((&raw mut game.g_ram as *mut uint8)
                        .offset(0xaf6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x1a4b as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v5 as isize) as ::core::ffi::c_int)
                        as uint16,
                );
                let mut v7: bool = (v6 as ::core::ffi::c_int)
                    < *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xafe as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int;
                let mut v8: uint16 = (v6 as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xafe as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int)
                    as uint16;
                if v7 as ::core::ffi::c_int != 0
                    || (v8 as ::core::ffi::c_int) < varE20 as ::core::ffi::c_int
                {
                    let mut v9: uint16 = abs16(
                        (*((&raw mut game.g_ram as *mut uint8)
                            .offset(0xafa as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            - *((&raw mut game.g_ram as *mut uint8)
                                .offset(0x1a93 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                .offset(v5 as isize)
                                as ::core::ffi::c_int) as uint16,
                    );
                    v7 = (v9 as ::core::ffi::c_int)
                        < *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xb00 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int;
                    let mut v10: uint16 = (v9 as ::core::ffi::c_int
                        - *((&raw mut game.g_ram as *mut uint8)
                            .offset(0xb00 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int)
                        as uint16;
                    if v7 as ::core::ffi::c_int != 0
                        || (v10 as ::core::ffi::c_int) < varE22 as ::core::ffi::c_int
                    {
                        off_86F0AD[(*((&raw mut game.g_ram as *mut uint8)
                            .offset(0x1aff as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v5 as isize)
                            as ::core::ffi::c_int
                            >> 1 as ::core::ffi::c_int)
                            as usize]
                            .expect("non-null function pointer")(game);
                        let mut v11: ::core::ffi::c_int =
                            k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x1b47 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v11 as isize) =
                            addr_word_86ECA3 as ::core::ffi::c_int as uint16;
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x1b8f as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v11 as isize) = 1 as uint16;
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x1bd7 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v11 as isize) = 12288 as uint16;
                        *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x1a03 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v11 as isize) = (0x86efdf as ::core::ffi::c_int
                            & 0xffff as ::core::ffi::c_int)
                            as uint16;
                    }
                }
                return;
            }
        }
    }
    let mut v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = addr_word_86ECA3 as ::core::ffi::c_int as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) = 12288 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v4 as isize) =
        (0x86efdf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Eproj_Pickup_SmallHealth(game: &mut Game) {
    Samus_RestoreHealth(5 as uint16, game);
    QueueSfx2_Max1(1 as uint16, game);
}
pub unsafe fn Eproj_Pickup_BigHealth(game: &mut Game) {
    Samus_RestoreHealth(0x14 as uint16, game);
    QueueSfx2_Max1(2 as uint16, game);
}
pub unsafe fn Eproj_Pickup_PowerBombs(game: &mut Game) {
    Samus_RestorePowerBombs(1 as uint16, game);
    QueueSfx2_Max1(5 as uint16, game);
}
pub unsafe fn Eproj_Pickup_Missiles(game: &mut Game) {
    Samus_RestoreMissiles(2 as uint16, game);
    QueueSfx2_Max1(3 as uint16, game);
}
pub unsafe fn Eproj_Pickup_SuperMissiles(game: &mut Game) {
    Samus_RestoreSuperMissiles(1 as uint16, game);
    QueueSfx2_Max1(4 as uint16, game);
}
static byte_86F25E: [uint8; 6] = [
    1 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    6 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
];
pub unsafe fn RandomDropRoutine(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v2: uint16 = 0;
    let mut Random: uint8 = 0;
    let mut v5: uint16 = 0;
    let mut v7: *const uint8 = ::core::ptr::null::<uint8>();
    let mut i: ::core::ffi::c_int = 0;
    let mut current_block: u64;
    let mut v9: int8 = 0;
    let mut r18: uint8 = 0;
    let mut r22: uint8 = 0;
    let mut r20: uint8 = 0;
    let mut r24: uint16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut varE28: uint16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xf3c8 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize);
    if !(varE28 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
        v2 = 0;
        v2 = (*get_EnemyDef_A2(varE28, game)).item_drop_chances_ptr as uint16;
        if !(v2 == 0) {
            Random = 0;
            loop {
                Random = NextRandom(game) as uint8;
                if !(Random == 0) {
                    break;
                }
            }
            r20 = 255 as uint8;
            r24 = 0 as uint16;
            v5 = 0;
            v5 = 1 as uint16;
            if (*((&raw mut game.g_ram as *mut uint8).offset(0x9d6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16
                as ::core::ffi::c_int
                >= 0x1e as ::core::ffi::c_int
            {
                if ((*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x9d6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x9c2 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16
                    as ::core::ffi::c_int)
                    < 0x32 as ::core::ffi::c_int
                {
                    current_block = 4166486009154926805;
                } else {
                    v5 = 0 as uint16;
                    current_block = 5399440093318478209;
                }
            } else {
                current_block = 5399440093318478209;
            }
            match current_block {
                5399440093318478209 => {
                    *((&raw mut game.g_ram as *mut uint8)
                        .offset(0xe1a as ::core::ffi::c_int as isize)
                        as *mut uint16) = v5;
                }
                _ => {}
            }
            v7 = RomPtr_B4(v2 as uint16_t, game);
            if *((&raw mut game.g_ram as *mut uint8).offset(0xe1a as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8
                != 0
            {
                r18 = (*v7.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + *v7 as ::core::ffi::c_int) as uint8;
                r22 = 3 as uint8;
            } else {
                r18 = *v7.offset(3 as ::core::ffi::c_int as isize);
                r22 = 8 as uint8;
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x9c2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    != *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x9c4 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                    || *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x9d6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        != *((&raw mut game.g_ram as *mut uint8)
                            .offset(0x9d4 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                {
                    r18 = (*v7.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        + *v7 as ::core::ffi::c_int
                        + r18 as ::core::ffi::c_int) as uint8;
                    r22 = (r22 as ::core::ffi::c_int | 3 as ::core::ffi::c_int) as uint8;
                }
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x9c6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    != *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x9c8 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                {
                    r18 = (*v7.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        + r18 as ::core::ffi::c_int) as uint8;
                    r22 = (r22 as ::core::ffi::c_int | 4 as ::core::ffi::c_int) as uint8;
                }
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x9ca as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    != *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x9cc as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                {
                    r20 = (r20 as ::core::ffi::c_int
                        - *v7.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                        as uint8;
                    r22 = (r22 as ::core::ffi::c_int | 0x10 as ::core::ffi::c_int) as uint8;
                }
                if *((&raw mut game.g_ram as *mut uint8)
                    .offset(0x9ce as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    != *((&raw mut game.g_ram as *mut uint8)
                        .offset(0x9d0 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                {
                    r20 = (r20 as ::core::ffi::c_int
                        - *v7.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                        as uint8;
                    r22 = (r22 as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int) as uint8;
                }
            }
            i = 0;
            i = 0 as ::core::ffi::c_int;
            while i != 4 as ::core::ffi::c_int {
                if r18 == 0 {
                    r22 = (r22 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint8;
                    v2 = (v2 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
                    i = 4 as ::core::ffi::c_int;
                    break;
                } else {
                    v9 = (r22 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as int8;
                    r22 = (r22 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint8;
                    if v9 != 0 {
                        let mut v10: *const uint8 = RomPtr_B4(v2 as uint16_t, game);
                        let mut RegWord: uint16 = Mult8x8(r20, *v10);
                        let mut divved: ::core::ffi::c_int =
                            SnesDivide(RegWord, r18) as ::core::ffi::c_int;
                        let mut v12: uint16 = r24;
                        let mut v13: uint16 = (divved + v12 as ::core::ffi::c_int) as uint16;
                        if v13 as ::core::ffi::c_int >= Random as ::core::ffi::c_int {
                            return byte_86F25E[i as usize] as uint16;
                        }
                        r24 = v13;
                    }
                    v2 = v2.wrapping_add(1);
                    i += 1;
                }
            }
            loop {
                v9 = (r22 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as int8;
                r22 = (r22 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint8;
                if v9 != 0 {
                    let mut v14: *const uint8 = RomPtr_B4(v2 as uint16_t, game);
                    if (r24 as ::core::ffi::c_int + *v14 as ::core::ffi::c_int) as uint16
                        as ::core::ffi::c_int
                        >= Random as ::core::ffi::c_int
                    {
                        return byte_86F25E[i as usize] as uint16;
                    }
                    r24 = (r24 as ::core::ffi::c_int + *v14 as ::core::ffi::c_int) as uint16;
                }
                v2 = v2.wrapping_add(1);
                i += 1;
                if !(i != 6 as ::core::ffi::c_int) {
                    break;
                }
            }
        }
    }
    i = 3 as ::core::ffi::c_int;
    return byte_86F25E[i as usize] as uint16;
}
pub unsafe fn RespawnEnemy(mut v0: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
        as *mut uint16) = v0;
    let mut v1: *const uint8 = RomPtr_A1(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x7cf as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + (v0 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int)) as uint16_t,
        game,
    );
    let mut E: *mut EnemyData = gEnemyData(v0 as ::core::ffi::c_int, game);
    (*E).enemy_ptr = *(v1 as *mut uint16);
    (*E).x_pos = *(v1.offset(2 as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).y_pos = *(v1.offset(4 as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).current_instruction = *(v1.offset(6 as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).properties = *(v1.offset(8 as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).extra_properties = *(v1.offset(10 as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).parameter_1 = *(v1.offset(12 as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).parameter_2 = *(v1.offset(14 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut ESD: *mut EnemySpawnData = gEnemySpawnData(v0 as ::core::ffi::c_int, game);
    (*E).palette_index = (*ESD).palette_index;
    (*E).vram_tiles_index = (*ESD).vram_tiles_index;
    (*E).frozen_timer = 0 as uint16;
    (*E).flash_timer = 0 as uint16;
    (*E).invincibility_timer = 0 as uint16;
    (*E).timer = 0 as uint16;
    (*E).frame_counter = 0 as uint16;
    (*E).ai_var_A = 0 as uint16;
    (*E).ai_var_B = 0 as uint16;
    (*E).ai_var_C = 0 as uint16;
    (*E).ai_var_D = 0 as uint16;
    (*E).ai_var_E = 0 as uint16;
    (*E).ai_preinstr = 0 as uint16;
    (*E).instruction_timer = 1 as uint16;
    let mut Edef: *mut EnemyDef = get_EnemyDef_A2((*E).enemy_ptr, game);
    (*E).x_width = (*Edef).x_radius;
    (*E).y_height = (*Edef).y_radius;
    (*E).health = (*Edef).health;
    (*E).layer = (*Edef).layer as uint16;
    *(&raw mut (*E).bank as *mut uint16) = *(&raw mut (*Edef).bank as *mut uint16);
    CallEnemyAi(
        (((*Edef).bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | (*Edef).ai_init as ::core::ffi::c_int) as uint32,
        game,
    );
}
unsafe fn EprojInit_Sparks(mut j: uint16, game: &mut Game) {
    static word_86F3D4: [uint16; 14] = [
        0xffff as ::core::ffi::c_int as uint16,
        0xb800 as ::core::ffi::c_int as uint16,
        0xffff as ::core::ffi::c_int as uint16,
        0xc000 as ::core::ffi::c_int as uint16,
        0xffff as ::core::ffi::c_int as uint16,
        0xe000 as ::core::ffi::c_int as uint16,
        0xffff as ::core::ffi::c_int as uint16,
        0xff00 as ::core::ffi::c_int as uint16,
        0 as ::core::ffi::c_int as uint16,
        0x100 as ::core::ffi::c_int as uint16,
        0 as ::core::ffi::c_int as uint16,
        0x2000 as ::core::ffi::c_int as uint16,
        0 as ::core::ffi::c_int as uint16,
        0x4000 as ::core::ffi::c_int as uint16,
    ];
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = addr_kEnemyDef_F353 as ::core::ffi::c_int as uint16;
    let mut v3: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).x_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) =
        ((*v3).y_pos as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = (*v3).y_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    let mut v4: uint16 = ((NextRandom(game) as ::core::ffi::c_int & 0x1c as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *(game.g_rom.offset(
        (((0x86f3d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86f3d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v4 as ::core::ffi::c_int + 0 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *(game.g_rom.offset(
        (((0x86f3d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x86f3d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v4 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize);
}
pub unsafe fn EprojPreInstr_Sparks(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 0xf363 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x1b23 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                * 2 as ::core::ffi::c_int
                + (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1aff as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    >> 15 as ::core::ffi::c_int)) as uint16;
            let ref mut fresh50 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x1aff as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh50 = (*fresh50 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8)
                .offset(0x1b23 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                * 2 as ::core::ffi::c_int
                + (*((&raw mut game.g_ram as *mut uint8)
                    .offset(0x1aff as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    >> 15 as ::core::ffi::c_int)) as uint16;
            let ref mut fresh51 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x1aff as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh51 = (*fresh51 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 0x8000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = -(1 as ::core::ffi::c_int) as uint16;
            let ref mut fresh52 = *((&raw mut game.g_ram as *mut uint8)
                .offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh52 = (*fresh52 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
            return;
        }
        let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) =
            (v2 as ::core::ffi::c_int + 0x4000 as ::core::ffi::c_int) as uint16;
        let mut v3: uint16 = (__CFADD__uint16(v2, 0x4000 as uint16) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
        if (v3 as ::core::ffi::c_int) < 4 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = v3;
        }
    }
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as uint32,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1b23 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x1aff as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as uint32,
    );
    if *(&raw mut game.g_ram as *mut uint8).offset(0x5b5 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        CreateSpriteAtPos(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize),
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize),
            48 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x19bb as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize),
            game,
        );
    }
}
unsafe fn CallEprojInit(mut ea: uint32, mut j: uint16, game: &mut Game) {
    match ea {
        8817209 => {
            Eproj_Init_0x8aaf(j, game);
            return;
        }
        8817357 => {
            Eproj_Init_0x8bc2_SkreeDownRight(j, game);
            return;
        }
        8817393 => {
            Eproj_Init_0x8bd0_SkreeUpRight(j, game);
            return;
        }
        8817429 => {
            Eproj_Init_0x8bde_SkreeDownLeft(j, game);
            return;
        }
        8817465 => {
            Eproj_Init_0x8bec_SkreeUpLeft(j, game);
            return;
        }
        8817924 => {
            EprojInit_DraygonsGunk(j, game);
            return;
        }
        8817984 => {
            EprojInit_DraygonsWallTurretProjs(j, game);
            return;
        }
        8818723 => {
            EprojInit_CrocomireProjectile(j, game);
            return;
        }
        8818895 => {
            EprojInit_CrocomireSpikeWallPieces(j, game);
            return;
        }
        8819334 => {
            EprojInit_CrocomireBridgeCrumbling(j, game);
            return;
        }
        8819533 => {
            EprojInit_9634(j, game);
            return;
        }
        8819658 => {
            EprojInit_9642_RidleysFireball(j, game);
            return;
        }
        8819839 => {
            EprojInit_9660_FireballExplosion(j, game);
            return;
        }
        8819872 => {
            EprojInit_9688(j, game);
            return;
        }
        8819892 => {
            EprojInit_9696(j, game);
            return;
        }
        8819912 => {
            EprojInit_966C(j, game);
            return;
        }
        8819932 => {
            EprojInit_967A(j, game);
            return;
        }
        8820444 => {
            EprojInit_9734_CeresFallingDebris(j, game);
            return;
        }
        8820772 => {
            EprojInit_PhantoonDestroyableFireballs(j, game);
            return;
        }
        8821050 => {
            EprojInit_PhantoonStartingFireballs(j, game);
            return;
        }
        8821923 => {
            EprojInit_RocksKraidSpits(j, game);
            return;
        }
        8821976 => {
            EprojInit_RocksFallingKraidCeiling(j, game);
            return;
        }
        8822028 => {
            EprojInit_RocksWhenKraidRises(j, game);
            return;
        }
        8822252 => {
            EprojInit_MiniKraidSpit(j, game);
            return;
        }
        8822342 => {
            EprojInit_MiniKraidSpikesLeft(j, game);
            return;
        }
        8822347 => {
            EprojInit_MiniKraidSpikesRight(j, game);
            return;
        }
        8822450 => {
            EprojInit_WalkingLavaSeahorseFireball(j, game);
            return;
        }
        8822793 => {
            EprojInit_PirateMotherBrainLaser(j, game);
            return;
        }
        8822936 => {
            EprojInit_PirateClaw(j, game);
            return;
        }
        8823457 => {
            EprojInit_A379(j, game);
            return;
        }
        8823534 => {
            EprojInit_CeresElevatorPad(j, game);
            return;
        }
        8823579 => {
            EprojInit_CeresElevatorPlatform(j, game);
            return;
        }
        8823715 => {
            EprojPreInstr_PrePhantomRoom(j, game);
            return;
        }
        8824275 => {
            EprojInit_BombTorizoLowHealthDrool(j, game);
            return;
        }
        8824413 => {
            EprojInit_BombTorizoLowHealthInitialDrool(j, game);
            return;
        }
        8824519 => {
            EprojInit_A977(j, game);
            return;
        }
        8824566 => {
            EprojInit_BombTorizoExplosiveSwipe(j, game);
            return;
        }
        8824676 => {
            EprojInit_BombTorizoStatueBreaking(j, game);
            return;
        }
        8824859 => {
            EprojInit_BombTorizoLowHealthExplode(j, game);
            return;
        }
        8824945 => {
            EprojInit_BombTorizoDeathExplosion(j, game);
            return;
        }
        8825405 => {
            EprojInit_AB07(j, game);
            return;
        }
        8825835 => {
            EprojInit_BombTorizosChozoOrbs(j, game);
            return;
        }
        8825980 => {
            EprojInit_GoldenTorizosChozoOrbs(j, game);
            return;
        }
        8826389 => {
            EprojInit_TorizoSonicBoom(j, game);
            return;
        }
        8826620 => {
            EprojInit_WreckedShipChozoSpikeFootsteps(j, game);
            return;
        }
        8826691 => {
            EprojInit_TourianStatueDustClouds(j, game);
            return;
        }
        8826704 => {
            EprojInit_TourianLandingDustCloudsRightFoot(j, game);
            return;
        }
        8826829 => {
            EprojInit_TorizoLandingDustCloudLeftFoot(j, game);
            return;
        }
        8826881 => {
            EprojInit_GoldenTorizoEgg(j, game);
            return;
        }
        8827342 => {
            EprojInit_GoldenTorizoSuperMissile(j, game);
            return;
        }
        8827688 => {
            EprojInit_GoldenTorizoEyeBeam(j, game);
            return;
        }
        8828061 => {
            EprojInit_TourianEscapeShaftFakeWallExplode(j, game);
            return;
        }
        8828143 => {
            EprojInit_LavaSeahorseFireball(j, game);
            return;
        }
        8828461 => {
            EprojInit_EyeDoorProjectile(j, game);
            return;
        }
        8828547 => {
            EprojInit_EyeDoorSweat(j, game);
            return;
        }
        8829050 => {
            EprojInit_TourianStatueUnlockingParticleWaterSplash(j, game);
            return;
        }
        8829070 => {
            EprojInit_TourianStatueEyeGlow(j, game);
            return;
        }
        8829109 => {
            EprojInit_TourianStatueUnlockingParticle(j, game);
            return;
        }
        8829160 => {
            EprojIni_TourianStatueUnlockingParticleTail(j, game);
            return;
        }
        8829176 => {
            EprojInit_TourianStatueSoul(j, game);
            return;
        }
        8829246 => {
            EprojInit_TourianStatueBaseDecoration(j, game);
            return;
        }
        8829265 => {
            EprojInit_TourianStatueRidley(j, game);
            return;
        }
        8829284 => {
            EprojInit_TourianStatuePhantoon(j, game);
            return;
        }
        8829744 => {
            sub_86BB30(j, game);
            return;
        }
        8829842 => {
            EprojInit_NuclearWaffleBody(j, game);
            return;
        }
        8829915 => {
            EprojInit_NorfairLavaquakeRocks(j, game);
            return;
        }
        8830364 => {
            EprojInit_ShaktoolAttackMiddleBackCircle(j, game);
            return;
        }
        8830370 => {
            EprojInit_BDA2(j, game);
            return;
        }
        8830543 => {
            EprojInit_MotherBrainRoomTurrets(j, game);
            return;
        }
        8830809 => {
            EprojInit_MotherBrainRoomTurretBullets(j, game);
            return;
        }
        8831731 => {
            Eproj_MotherBrainsBlueRingLasers(j, game);
            return;
        }
        8832130 => {
            EprojInit_MotherBrainBomb(j, game);
            return;
        }
        8832517 => {
            sub_86C605(j, game);
            return;
        }
        8832644 => {
            EprojInit_MotherBrainDeathBeemFired(j, game);
            return;
        }
        8833034 => {
            EprojInit_MotherBrainRainbowBeam(j, game);
            return;
        }
        8833091 => {
            EprojInit_MotherBrainsDrool(j, game);
            return;
        }
        8833269 => {
            EprojInit_MotherBrainsDeathExplosion(j, game);
            return;
        }
        8833327 => {
            EprojInit_MotherBrainsRainbowBeamExplosion(j, game);
            return;
        }
        8833377 => {
            EprojInit_MotherBrainEscapeDoorParticles(j, game);
            return;
        }
        8833642 => {
            EprojInit_MotherBrainPurpleBreathBig(j, game);
            return;
        }
        8833667 => {
            EprojInit_MotherBrainPurpleBreathSmall(j, game);
            return;
        }
        8833782 => {
            EprojInit_TimeBombSetJapaneseText(j, game);
            return;
        }
        8833993 => {
            EprojInit_MotherBrainTubeFalling(j, game);
            return;
        }
        8834501 => {
            EprojInit_MotherBrainGlassShatteringShard(j, game);
            return;
        }
        8834669 => {
            EprojInit_MotherBrainGlassShatteringSparkle(j, game);
            return;
        }
        8834960 => {
            EprojInit_KiHunterAcidSpitLeft(j, game);
            return;
        }
        8834982 => {
            EprojInit_KiHunterAcidSpitRight(j, game);
            return;
        }
        8835208 => {
            EprojInit_KagosBugs(j, game);
            return;
        }
        8835642 => {
            EprojInit_MaridiaFloatersSpikes(j, game);
            return;
        }
        8835852 => {
            EprojInit_WreckedShipRobotLaserDown(j, game);
            return;
        }
        8835886 => {
            EprojInit_WreckedShipRobotLaserHorizontal(j, game);
            return;
        }
        8835905 => {
            EprojInit_WreckedShipRobotLaserUp(j, game);
            return;
        }
        8836773 => {
            EprojInit_N00bTubeCrack(j, game);
            return;
        }
        8836809 => {
            EprojInit_N00bTubeShards(j, game);
            return;
        }
        8836980 => {
            EprojInit_N00bTubeReleasedAirBubbles(j, game);
            return;
        }
        8837522 => {
            sub_86D992(j, game);
            return;
        }
        8837912 => {
            EprojInit_DBF2(j, game);
            return;
        }
        8838285 => {
            EprojInit_Spores(j, game);
            return;
        }
        8838307 => {
            EprojInit_SporeSpawnStalk(j, game);
            return;
        }
        8838356 => {
            EprojInit_SporeSpawners(j, game);
            return;
        }
        8838870 => {
            EprojInit_NamiFuneFireball(j, game);
            return;
        }
        8839168 => {
            EprojInit_LavaThrownByLavaman(j, game);
            return;
        }
        8840296 => {
            EprojInit_DustCloudOrExplosion(j, game);
            return;
        }
        8840358 => {
            EprojInit_EyeDoorSmoke(j, game);
            return;
        }
        8840656 => {
            EprojInit_SpawnedShotGate(j, game);
            return;
        }
        8840661 => {
            EprojInit_ClosedDownwardsShotGate(j, game);
            return;
        }
        8840666 => {
            EprojInit_ClosedUpwardsShotGate(j, game);
            return;
        }
        8840877 => {
            EprojInit_SaveStationElectricity(j, game);
            return;
        }
        8841777 => {
            EprojInit_BotwoonsBody(j, game);
            return;
        }
        8842182 => {
            EprojInit_BotwoonsSpit(j, game);
            return;
        }
        8842338 => {
            EprojInit_YappingMawsBody(j, game);
            return;
        }
        8843049 => {
            EprojInit_F337(j, game);
            return;
        }
        8843145 => {
            EprojInit_EnemyDeathExplosion(j, game);
            return;
        }
        8844177 => {
            EprojInit_Sparks(j, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
unsafe fn CallEprojPreInstr(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        8825112 => return,
        8814960 => return,
        8815867 => return,
        8817277 => {
            Eproj_PreInit_0x8aaf(k, game);
            return;
        }
        8817501 => {
            Eproj_PreInstr_SkreeParticle(k, game);
            return;
        }
        8818004 => return,
        8818122 => {
            EprojPreInstr_8DCA(k, game);
            return;
        }
        8818175 => {
            EprojPreInstr_DraygonsTurret_8DFF(k, game);
            return;
        }
        8818191 => {
            EprojPreInstr_DraygonsGunk_8E0F(k, game);
            return;
        }
        8818795 => {
            EprojPreInstr_CrocomireProjectile(k, game);
            return;
        }
        8818867 => {
            sub_8690B3(k, game);
            return;
        }
        8818965 => {
            EprojPreInstr_CrocomireSpikeWallPieces(k, game);
            return;
        }
        8819386 => {
            EprojPreInstr_CrocomireBridgeCrumbling(k, game);
            return;
        }
        8819602 => {
            EprojPreInstr_9634(k, game);
            return;
        }
        8819726 => {
            EprojPreInstr_9642_RidleysFireball(k, game);
            return;
        }
        8819980 => return,
        8819981 => {
            EprojPreInstr_966C(k, game);
            return;
        }
        8820002 => {
            EprojPreInstr_9688(k, game);
            return;
        }
        8820023 => {
            EprojPreInstr_96A4(k, game);
            return;
        }
        8820032 => {
            EprojPreInstr_96C0(k, game);
            return;
        }
        8820041 => {
            EprojPreInstr_96CE(k, game);
            return;
        }
        8820481 => {
            EprojPreInstr_9734_CeresFallingDebris(k, game);
            return;
        }
        8821121 => {
            EprojPreInstr_PhantoonStartingFireballs(k, game);
            return;
        }
        8821183 => {
            EprojPreInstr_PhantoonStartingFireballs2(k, game);
            return;
        }
        8821249 => {
            EprojPreInstr_PhantoonStartingFireballs3(k, game);
            return;
        }
        8821316 => return,
        8821317 => {
            EprojPreInstr_PhantoonDestroyableFireballs(k, game);
            return;
        }
        8821396 => {
            EprojPreInstr_PhantoonDestroyableFireballs_2(k, game);
            return;
        }
        8821466 => {
            EprojPreInstr_PhantoonDestroyableFireballs_3(k, game);
            return;
        }
        8821545 => {
            EprojPreInstr_PhantoonStartingFireballsB(k, game);
            return;
        }
        8821569 => {
            EprojPreInstr_PhantoonStartingFireballsB_2(k, game);
            return;
        }
        8822102 => {
            EprojPreInstr_KraidRocks(k, game);
            return;
        }
        8822153 => {
            EprojPreInstr_RocksFallingKraidCeiling(k, game);
            return;
        }
        8822181 => {
            sub_869DA5(k, game);
            return;
        }
        8822302 => {
            EprojPreInit_MiniKraidSpit(k, game);
            return;
        }
        8822403 => {
            EprojPreInstr_MiniKraidSpikes(k, game);
            return;
        }
        8822527 => {
            EprojPreInstr_WalkingLavaSeahorseFireball(k, game);
            return;
        }
        8822875 => return,
        8822876 => {
            EprojPreInstr_PirateMotherBrainLaser_MoveLeft(k, game);
            return;
        }
        8822906 => {
            EprojPreInstr_PirateMotherBrainLaser_MoveRight(k, game);
            return;
        }
        8822993 => {
            EprojPreInstr_PirateClawThrownLeft(k, game);
            return;
        }
        8823076 => {
            EprojPreInstr_PirateClawThrownRight(k, game);
            return;
        }
        8823591 => return,
        8823592 => {
            EprojPreInstr_CeresElevatorPad(k, game);
            return;
        }
        8823652 => {
            EprojPreInstr_CeresElevatorPlatform(k, game);
            return;
        }
        8823715 => {
            EprojPreInstr_PrePhantomRoom(k, game);
            return;
        }
        8824967 => {
            sub_86A887(k, game);
            return;
        }
        8825071 => {
            EprojPreInstr_A977(k, game);
            return;
        }
        8825113 => return,
        8825484 => {
            EprojPreInstr_AB07(k, game);
            return;
        }
        8826029 => {
            EprojPreInstr_BombTorizosChozoOrbs(k, game);
            return;
        }
        8826106 => {
            EprojPreInstr_GoldenTorizosChozoOrbs(k, game);
            return;
        }
        8826476 => {
            EprojPreInstr_TorizoSonicBoom(k, game);
            return;
        }
        8826947 => {
            EprojPreInstr_GoldenTorizoEgg(k, game);
            return;
        }
        8827065 => {
            sub_86B0B9(k, game);
            return;
        }
        8827101 => {
            sub_86B0DD(k, game);
            return;
        }
        8827405 => {
            EprojPreInstr_GoldenTorizoSuperMissile(k, game);
            return;
        }
        8827447 => {
            EprojPreInstr_B237(k, game);
            return;
        }
        8827786 => {
            EprojPreInstr_GoldenTorizoEyeBeam(k, game);
            return;
        }
        8828080 => return,
        8828213 => {
            sub_86B535(k, game);
            return;
        }
        8828601 => {
            EprojPreInstr_EyeDoorProjectile(k, game);
            return;
        }
        8828692 => {
            EprojPreInstr_EyeDoorSweat(k, game);
            return;
        }
        8829303 => {
            EprojPreInstr_TourianStatueUnlockingParticleWaterSplash(k, game);
            return;
        }
        8829314 => {
            EprojPreInstr_TourianStatueUnlockingParticle(k, game);
            return;
        }
        8829437 => {
            EprojPreInstr_TourianStatueSoul(k, game);
            return;
        }
        8829495 => {
            EprojPreInstr_TourianStatueStuff(k, game);
            return;
        }
        8829506 => {
            EprojPreInstr_BA42(k, game);
            return;
        }
        8829775 => return,
        8829894 => return,
        8829967 => {
            EprojPreInstr_NorfairLavaquakeRocks(k, game);
            return;
        }
        8830238 => {
            EprojPreInstr_NorfairLavaquakeRocks_Inner(k, game);
            return;
        }
        8830250 => {
            EprojPreInstr_NorfairLavaquakeRocks_Inner2(k, game);
            return;
        }
        8830467 => {
            EprojInit_ShaktoolAttackFrontCircle(k, game);
            return;
        }
        8830482 => {
            EprojPreInstr_BE12(k, game);
            return;
        }
        8830943 => {
            EprojPreInstr_MotherBrainRoomTurrets(k, game);
            return;
        }
        8831200 => {
            EprojPreInstr_MotherBrainRoomTurretBullets(k, game);
            return;
        }
        8831797 => {
            Eproj_MoveToBlueRingSpawnPosition(k, game);
            return;
        }
        8832200 => {
            EprojPreInstr_MotherBrainBomb(k, game);
            return;
        }
        8833044 => {
            EprojPreInstr_MotherBrainRainbowBeam(k, game);
            return;
        }
        8833101 => {
            EprojPreInstr_C84D(k, game);
            return;
        }
        8833158 => {
            EprojInit_MotherBrainsDrool_Falling(k, game);
            return;
        }
        8833300 => {
            EprojPreInit_MotherBrainsDeathExplosion_0(k, game);
            return;
        }
        8833356 => {
            EprojPreInstr_MotherBrainsRainbowBeamExplosion(k, game);
            return;
        }
        8833490 => {
            EprojPreInstr_MotherBrainsExplodedDoorParticles(k, game);
            return;
        }
        8833699 => return,
        8833786 => {
            EprojPreInstr_TimeBombSetJapaneseText(k, game);
            return;
        }
        8834023 => {
            EprojPreInstr_MotherBrainTubeFalling(k, game);
            return;
        }
        8834715 => {
            EprojPreInstr_MotherBrainGlassShatteringShard(k, game);
            return;
        }
        8835029 => {
            sub_86CFD5(k, game);
            return;
        }
        8835046 => {
            sub_86CFE6(k, game);
            return;
        }
        8835063 => return,
        8835064 => {
            sub_86CFF8(k, game);
            return;
        }
        8835251 => {
            EprojPreInstr_KagosBugs_Func1(k, game);
            return;
        }
        8835274 => {
            EprojPreInstr_KagosBugs(k, game);
            return;
        }
        8835307 => return,
        8835308 => {
            EprojPreInstr_D0EC(k, game);
            return;
        }
        8835368 => {
            EprojPreInstr_D128(k, game);
            return;
        }
        8835556 => {
            EprojPreInstr_KagosBugs_Func2(k, game);
            return;
        }
        8835683 => {
            EprojPreInstr_MaridiaFloatersSpikes(k, game);
            return;
        }
        8836031 => {
            EprojPreInstr_WreckedShipRobotLaser(k, game);
            return;
        }
        8837055 => {
            sub_86D7BF(k, game);
            return;
        }
        8837086 => {
            sub_86D7DE(k, game);
            return;
        }
        8837117 => {
            EprojPreInstr_N00bTubeShards(k, game);
            return;
        }
        8837181 => {
            sub_86D83D(k, game);
            return;
        }
        8837279 => {
            sub_86D89F(k, game);
            return;
        }
        8837343 => {
            sub_86D8DF(k, game);
            return;
        }
        8837595 => {
            EprojPreInstr_SpikeShootingPlantSpikes(k, game);
            return;
        }
        8837606 => {
            EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, game);
            return;
        }
        8837648 => {
            EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, game);
            return;
        }
        8837690 => {
            EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, game);
            return;
        }
        8837732 => {
            EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, game);
            return;
        }
        8837774 => {
            EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1(k, game);
            return;
        }
        8837779 => {
            EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2(k, game);
            return;
        }
        8837784 => {
            EprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1(k, game);
            return;
        }
        8837789 => {
            EprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2(k, game);
            return;
        }
        8837794 => {
            EprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1(k, game);
            return;
        }
        8837802 => {
            EprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2(k, game);
            return;
        }
        8837810 => {
            EprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1(k, game);
            return;
        }
        8837818 => {
            EprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2(k, game);
            return;
        }
        8837979 => {
            EprojPreInstr_DBF2(k, game);
            return;
        }
        8837986 => {
            EprojPreInstr_DBF2_MoveX1(k, game);
            return;
        }
        8838028 => {
            EprojPreInstr_DBF2_MoveX2(k, game);
            return;
        }
        8838070 => {
            EprojPreInstr_DBF2_Func1(k, game);
            return;
        }
        8838382 => {
            EprojPreInstr_Spores(k, game);
            return;
        }
        8838468 => return,
        8838470 => {
            EprojPreInstr_SporeSpawners(k, game);
            return;
        }
        8838969 => {
            EprojPreInstr_NamiFuneFireball(k, game);
            return;
        }
        8839241 => {
            sub_86E049(k, game);
            return;
        }
        8840446 => {
            EprojPreInstr_DustCloudOrExplosion(k, game);
            return;
        }
        8840456 => return,
        8840708 => return,
        8840709 => {
            EprojPreInstr_E605(k, game);
            return;
        }
        8840913 => return,
        8841856 => {
            EprojPreInstr_BotwoonsBody(k, game);
            return;
        }
        8842245 => {
            EprojPreInstr_BotwoonsSpit(k, game);
            return;
        }
        8842388 => return,
        8843231 => return,
        8843232 => {
            EprojPreInstr_Pickup(k, game);
            return;
        }
        8844272 => {
            EprojPreInstr_Sparks(k, game);
            return;
        }
        8832877 => return,
        _ => {
            Unreachable(game);
        }
    };
}
unsafe fn CallEprojInstr(
    mut ea: uint32,
    mut k: uint16,
    mut j: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    match ea {
        8814932 => return EprojInstr_Delete(k, j, game),
        8814937 => return EprojInstr_Sleep(k, j, game),
        8814945 => return EprojInstr_SetPreInstr_(k, j, game),
        8814954 => return EprojInstr_ClearPreInstr(k, j, game),
        8814961 => return EprojInstr_CallFunc(k, j, game),
        8815019 => return EprojInstr_Goto(k, j),
        8815024 => return EprojInstr_GotoRel(k, j),
        8815046 => return EprojInstr_DecTimerAndGotoIfNonZero(k, j, game),
        8815054 => return EprojInstr_DecTimerAndGotoRelIfNonZero(k, j, game),
        8815061 => return EprojInstr_SetTimer(k, j, game),
        8815071 => return EprojInstr_MoveRandomlyWithinRadius(k, j, game),
        8815152 => return EprojInstr_SetProjectileProperties(k, j, game),
        8815164 => return EprojInstr_ClearProjectileProperties(k, j, game),
        8815176 => return EprojInstr_EnableCollisionWithSamusProj(k, j, game),
        8815186 => return EprojInstr_DisableCollisionWithSamusProj(k, j, game),
        8815196 => return EprojInstr_DisableCollisionWithSamus(k, j, game),
        8815206 => return EprojInstr_EnableCollisionWithSamus(k, j, game),
        8815216 => return EprojInstr_SetToNotDieOnContact(k, j, game),
        8815226 => return EprojInstr_SetToDieOnContact(k, j, game),
        8815236 => return EprojInstr_SetLowPriority(k, j, game),
        8815246 => return EprojInstr_SetHighPriority(k, j, game),
        8815256 => return EprojInstr_SetXyRadius(k, j, game),
        8815265 => return EprojInstr_SetXyRadiusZero(k, j, game),
        8815269 => return EprojInstr_CalculateDirectionTowardsSamus(k, j, game),
        8815317 => return EprojInstr_WriteColorsToPalette(k, j, game),
        8815357 => return EprojInstr_QueueMusic(k, j, game),
        8815369 => return EprojInstr_QueueSfx1_Max6(k, j, game),
        8815378 => return EprojInstr_QueueSfx2_Max6(k, j, game),
        8815387 => return EprojInstr_QueueSfx3_Max6(k, j, game),
        8815396 => return EprojInstr_QueueSfx1_Max15(k, j, game),
        8815405 => return EprojInstr_QueueSfx2_Max15(k, j, game),
        8815414 => return EprojInstr_QueueSfx3_Max15(k, j, game),
        8815423 => return EprojInstr_QueueSfx1_Max3(k, j, game),
        8815432 => return EprojInstr_QueueSfx2_Max3(k, j, game),
        8815441 => return EprojInstr_QueueSfx3_Max3(k, j, game),
        8815450 => return EprojInstr_QueueSfx1_Max9(k, j, game),
        8815459 => return EprojInstr_QueueSfx2_Max9(k, j, game),
        8815468 => return EprojInstr_QueueSfx3_Max9(k, j, game),
        8815477 => return EprojInstr_QueueSfx1_Max1(k, j, game),
        8815486 => return EprojInstr_QueueSfx2_Max1(k, j, game),
        8815495 => return EprojInstr_QueueSfx3_Max1(k, j, game),
        8817768 => return EprojInstr_SpawnEnemyDropsWithDraygonsEyeDrops(k, j, game),
        8818073 => return EprojInstr_868D99(k, j, game),
        8819829 => return EprojInstr_DisableCollisionsWithSamus(k, j, game),
        8820154 => return EprojInstr_95BA(k, j, game),
        8820205 => return EprojInstr_95ED(k, j, game),
        8820256 => return EprojInstr_9620(k, j, game),
        8820750 => return EprojInstr_980E(k, j, game),
        8822864 => return EprojInstr_SetPreInstrAndRun(k, j, game),
        8823894 => return EprojInstr_GotoWithProbability25(k, j, game),
        8825738 => return EprojInstr_SpawnEnemyDrops(k, j, game),
        8826258 => return EprojInstr_GotoDependingOnXDirection(k, j, game),
        8826678 => return EprojInstr_ResetXYpos1(k, j, game),
        8826770 => return EprojInstr_MoveY_Minus4(k, j, game),
        8827497 => return EprojInstr_SetVelTowardsSamus1(k, j, game),
        8827506 => return EprojInstr_SetVelTowardsSamus2(k, j, game),
        8827832 => return EprojInstr_GotoIfFunc1(k, j, game),
        8827958 => return EprojInstr_ResetXYpos2(k, j, game),
        8828906 => return EprojInstr_SpawnTourianStatueUnlockingParticle(k, j, game),
        8828917 => return EprojInstr_Earthquake(k, j, game),
        8828952 => return EprojInstr_SpawnTourianStatueUnlockingParticleTail(k, j, game),
        8828993 => return EprojInstr_AddToYpos(k, j, game),
        8831347 => return EprojInstr_SwitchJump(k, j, game),
        8831412 => return EprojInstr_UserPalette0(k, j, game),
        8833232 => return EprojInstr_Add12ToY(k, j, game),
        8833774 => return EprojInstr_MotherBrainPurpleBreathIsActive(k, j, game),
        8835420 => return EprojInstr_D15C(k, j, game),
        8835510 => return EprojInstr_D1B6(k, j, game),
        8835527 => return EprojInstr_D1C7(k, j, game),
        8835534 => return EprojInstr_D1CE(k, j, game),
        8836577 => return EprojInstr_AssignNewN00bTubeShardVelocity(k, j, game),
        8836594 => return EprojInstr_SetN00bTubeShardX(k, j, game),
        8836650 => return EprojInstr_D62A(k, j, game),
        8836762 => return EprojInstr_SetXvelRandom(k, j, game),
        8838234 => return EprojInstr_DC5A(k, j, game),
        8838241 => return EprojInstr_SpawnEnemyDrops_0(k, j, game),
        8838263 => return EprojInstr_SpawnSporesEproj(k, j, game),
        8839146 => return EprojInstr_DFEA(k, j, game),
        8840499 => return EprojInstr_SetYVel(k, j, game),
        8842467 => return EprojInstr_ECE3(k, j, game),
        8842519 => return EprojInstr_ED17(k, j, game),
        8842891 => return EprojInstr_QueueSfx2_9(k, j, game),
        8842903 => return EprojInstr_QueueSfx2_24(k, j, game),
        8842915 => return EprojInstr_QueueSfx2_B(k, j, game),
        8842927 => return EprojInstr_EEAF(k, j, game),
        8843024 => return EprojInstr_HandleRespawningEnemy(k, j, game),
        8817910 => return EprojInstr_SetPreInstrA(k, j, game),
        8817917 => return EprojInstr_SetPreInstrB(k, j, game),
        8822906 => {
            EprojPreInstr_PirateMotherBrainLaser_MoveRight(k, game);
            return j;
        }
        8822876 => {
            EprojPreInstr_PirateMotherBrainLaser_MoveLeft(k, game);
            return j;
        }
        8823742 => return EprojInstr_A3BE(k, j, game),
        8819312 => return EprojInstr_9270(k, j, game),
        8815070 => return j,
        8827198 => return sub_86B13E(k, j, game),
        _ => {
            Unreachable(game);
            return ::core::ptr::null::<uint8>();
        }
    };
}
