extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    static mut g_ram: [uint8; 131072];
    static mut g_rom: *const uint8;
    fn RomPtr(addr: uint32_t) -> *const uint8;
    fn QueueMusic_Delayed8(a: uint16);
    fn QueueSfx2_Max6(a: uint16);
    fn QueueSfx3_Max6(a: uint16);
    fn SpawnHardcodedPlm(p: SpawnHardcodedPlmArgs);
    static mut eproj_spawn_pt: Point16U;
    fn ClearEprojs();
    fn SpawnEprojWithGfx(a: uint16, k: uint16, j: uint16);
    fn SpawnEprojWithRoomGfx(j: uint16, a: uint16);
    fn SpawnHdmaObject(db: uint8, p: *const ::core::ffi::c_void) -> uint16;
    fn DisableMinimapAndMarkBossRoomAsExplored();
    fn Enemy_MoveRight_IgnoreSlopes(k: uint16, amt: int32) -> uint8;
    fn Enemy_ItemDrop_Crocomire(k: uint16);
    fn NormalEnemyFrozenAI();
    fn NormalEnemyShotAi();
    fn NormalEnemyTouchAi();
    fn SamusLatchesOnWithGrapple();
    fn CreateSpriteAtPos(
        x_r18: uint16,
        y_r20: uint16,
        ilist_r22: uint16,
        pal_r24: uint16,
    ) -> uint16;
}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t,
    EnemyBase, EnemyData, EnemyProps, Func_V, LongPtr, Point16U, SpawnHdmaObject_Args, VoidP,
    VramWriteEntry,
};
pub const kEnemyProps_RespawnIfKilled: EnemyProps = 16384;
pub const kEnemyProps_DisableSamusColl: EnemyProps = 8192;
pub const kEnemyProps_BlockPlasmaBeam: EnemyProps = 4096;
pub const kEnemyProps_ProcessedOffscreen: EnemyProps = 2048;
pub const kEnemyProps_Tangible: EnemyProps = 1024;
pub const kEnemyProps_Deleted: EnemyProps = 512;
pub const kEnemyProps_Invisible: EnemyProps = 256;
pub use crate::types::{Consts_86, Ram4000, Ram4000_Backups};
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
pub use crate::types::Consts_A4;
pub const addr_kCrocomire_Ilist_E1D2: Consts_A4 = 57810;
pub const addr_kCrocomire_Ilist_E1CC: Consts_A4 = 57804;
pub const addr_kCrocomire_Ilist_E158: Consts_A4 = 57688;
pub const addr_kCrocomire_Ilist_E14A: Consts_A4 = 57674;
pub const addr_kCrocomire_BigSprmap_D600: Consts_A4 = 54784;
pub const addr_kCrocomire_BigSprmap_D51C: Consts_A4 = 54556;
pub const addr_kCrocomire_Ilist_BFB0: Consts_A4 = 49072;
pub const addr_kCrocomire_Ilist_BF98: Consts_A4 = 49048;
pub const addr_kCrocomire_Ilist_BF92: Consts_A4 = 49042;
pub const addr_kCrocomire_Ilist_BF8C: Consts_A4 = 49036;
pub const addr_kCrocomire_Ilist_BF86: Consts_A4 = 49030;
pub const addr_kCrocomire_Ilist_BF7E: Consts_A4 = 49022;
pub const addr_kCrocomire_Ilist_BF78: Consts_A4 = 49016;
pub const addr_kCrocomire_Ilist_BF72: Consts_A4 = 49010;
pub const addr_kCrocomire_Ilist_BF6C: Consts_A4 = 49004;
pub const addr_kCrocomire_Ilist_BF64: Consts_A4 = 48996;
pub const addr_kCrocomire_Ilist_BF62: Consts_A4 = 48994;
pub const addr_kCrocomire_Ilist_BF3C: Consts_A4 = 48956;
pub const addr_kCrocomire_Ilist_BE7E: Consts_A4 = 48766;
pub const addr_kCrocomire_Ilist_BE56: Consts_A4 = 48726;
pub const addr_kCrocomire_Ilist_BDB6: Consts_A4 = 48566;
pub const addr_kCrocomire_Ilist_BDB2: Consts_A4 = 48562;
pub const addr_kCrocomire_Ilist_BDAE: Consts_A4 = 48558;
pub const addr_kCrocomire_Ilist_BDA2: Consts_A4 = 48546;
pub const addr_kCrocomire_Ilist_BD8E: Consts_A4 = 48526;
pub const addr_kCrocomire_Ilist_BD2A: Consts_A4 = 48426;
pub const addr_kCrocomire_Ilist_BCD8: Consts_A4 = 48344;
pub const addr_kCrocomire_Ilist_BC56: Consts_A4 = 48214;
pub const addr_kCrocomire_Ilist_BC34: Consts_A4 = 48180;
pub const addr_kCrocomire_Ilist_BC30: Consts_A4 = 48176;
pub const addr_kCrocomire_Ilist_BBCE: Consts_A4 = 48078;
pub const addr_kCrocomire_Ilist_BBCA: Consts_A4 = 48074;
pub const addr_kCrocomire_Ilist_BB36: Consts_A4 = 47926;
pub const addr_kCrocomire_Ilist_BAEA: Consts_A4 = 47850;
pub const addr_kCrocomire_Ilist_BADE: Consts_A4 = 47838;
pub const addr_kExtendedSpritemap_Nothing_A4: Consts_A4 = 32847;
pub const addr_kSpritemap_Nothing_A4: Consts_A4 = 32845;
pub use crate::types::SpawnHardcodedPlmArgs;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Crocomire {
    pub base: EnemyBase,
    pub crocom_var_A: uint16,
    pub crocom_var_B: uint16,
    pub crocom_var_C: uint16,
    pub crocom_var_D: uint16,
    pub crocom_var_E: uint16,
    pub crocom_var_F: uint16,
    pub crocom_parameter_1: uint16,
    pub crocom_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub crocom_var_00: uint16,
    pub crocom_var_01: uint16,
    pub crocom_var_02: uint16,
    pub crocom_var_03: uint16,
    pub crocom_var_04: uint16,
    pub crocom_var_05: uint16,
    pub crocom_var_06: uint16,
    pub crocom_var_07: uint16,
    pub crocom_var_08: uint16,
    pub crocom_var_09: uint16,
    pub crocom_var_0A: uint16,
    pub crocom_var_0B: uint16,
    pub crocom_var_0C: uint16,
    pub crocom_var_0D: uint16,
    pub crocom_var_0E: uint16,
    pub crocom_var_0F: uint16,
    pub crocom_var_10: uint16,
    pub crocom_var_11: uint16,
    pub crocom_var_12: uint16,
    pub crocom_var_13: uint16,
    pub crocom_var_14: uint16,
    pub crocom_var_15: uint16,
    pub crocom_var_16: uint16,
    pub crocom_var_17: uint16,
    pub crocom_var_18: uint16,
    pub crocom_var_19: uint16,
    pub crocom_var_1A: uint16,
    pub crocom_var_1B: uint16,
    pub crocom_var_1C: uint16,
    pub crocom_var_1D: uint16,
    pub crocom_var_1E: uint16,
    pub crocom_var_1F: uint16,
    pub pad2: [uint8; 1984],
    pub crocom_var_20: uint16,
    pub crocom_var_21: uint16,
    pub crocom_var_22: uint16,
    pub crocom_var_23: uint16,
    pub crocom_var_24: uint16,
    pub crocom_var_25: uint16,
    pub crocom_var_26: uint16,
    pub crocom_var_27: uint16,
    pub crocom_var_28: uint16,
    pub crocom_var_29: uint16,
    pub crocom_var_2A: uint16,
    pub crocom_var_2B: uint16,
    pub crocom_var_2C: uint16,
    pub crocom_var_2D: uint16,
    pub crocom_var_2E: uint16,
    pub crocom_var_2F: uint16,
    pub crocom_var_30: uint16,
    pub crocom_var_31: uint16,
    pub crocom_var_32: uint16,
    pub crocom_var_33: uint16,
    pub crocom_var_34: uint16,
    pub crocom_var_35: uint16,
    pub crocom_var_36: uint16,
    pub crocom_var_37: uint16,
    pub crocom_var_38: uint16,
    pub crocom_var_39: uint16,
    pub crocom_var_3A: uint16,
    pub crocom_var_3B: uint16,
    pub crocom_var_3C: uint16,
    pub crocom_var_3D: uint16,
    pub crocom_var_3E: uint16,
    pub crocom_var_3F: uint16,
}
pub use crate::types::CrocomireFunc;
#[inline]
unsafe extern "C" fn abs16(mut t: uint16) -> uint16 {
    return (if t as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(t as ::core::ffi::c_int)
    } else {
        t as ::core::ffi::c_int
    }) as uint16;
}
#[inline]
unsafe extern "C" fn __CFADD__uint16(mut x: uint16, mut y: uint16) -> bool {
    return x as ::core::ffi::c_int
        > (x as ::core::ffi::c_int + y as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn __CFADD__uint8(mut x: uint8, mut y: uint8) -> bool {
    return x as ::core::ffi::c_int
        > (x as ::core::ffi::c_int + y as ::core::ffi::c_int) as uint8 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn RomPtr_A4(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0xa40000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
#[inline]
unsafe extern "C" fn RomPtrWithBank(mut bank: uint8, mut addr: uint16_t) -> *const uint8 {
    return RomPtr(
        ((bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int | addr as ::core::ffi::c_int)
            as uint32_t,
    );
}
#[inline]
unsafe extern "C" fn RomBankBase(mut bank: uint8) -> *const uint8 {
    return RomPtr(
        (((bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) + 0x8000 as ::core::ffi::c_int)
            as uint32_t,
    )
    .offset(-(0x8000 as ::core::ffi::c_int as isize));
}
#[inline]
unsafe extern "C" fn gVramWriteEntry(mut a: ::core::ffi::c_int) -> *mut VramWriteEntry {
    return (&raw mut g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry;
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
#[inline]
unsafe extern "C" fn gEnemyData(mut o: ::core::ffi::c_int) -> *mut EnemyData {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut EnemyData;
}
#[inline]
unsafe extern "C" fn Get_Crocomire(mut o: ::core::ffi::c_int) -> *mut Enemy_Crocomire {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_Crocomire;
}
static mut g_word_A48692: uint16 = 8 as uint16;
static mut g_word_A48694: uint16 = 8 as uint16;
static mut g_word_A48698: uint16 = 2 as uint16;
static mut g_word_A4869A: uint16 = 1 as uint16;
static mut g_word_A4869C: uint16 = 3 as uint16;
static mut g_word_A4869E: uint16 = 3 as uint16;
static mut g_word_A486A0: uint16 = 8 as uint16;
static mut g_word_A486A2: uint16 = 0x300 as uint16;
static mut g_word_A486A4: uint16 = 0x640 as uint16;
#[no_mangle]
pub unsafe extern "C" fn Enemy_GrappleReact_SamusLatchesOn_A4() {
    SamusLatchesOnWithGrapple();
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_NormalTouchAI_A4() {
    NormalEnemyTouchAi();
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_NormalShotAI_A4() {
    NormalEnemyShotAi();
}
#[no_mangle]
pub unsafe extern "C" fn Enemy_NormalFrozenAI_A4() {
    NormalEnemyFrozenAI();
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Hurt() {
    Crocomire_Func_27(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    Crocomire_Func_31();
}
static mut off_A486B3: [Option<CrocomireFunc>; 21] = unsafe {
    [
        Some(Crocomire_Func_2 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_3 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_4 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_5 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_7 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_8 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_9 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_10 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_11 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_13 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_14 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_15 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_16 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_17 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_18 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_21 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_22 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_23 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_24 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_25 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
        Some(Crocomire_Func_26 as unsafe extern "C" fn(uint16, *const uint16) -> *const uint16),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_1(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut crocom_var_C: uint16 = (*Get_Crocomire(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .crocom_var_C;
    return off_A486B3[(crocom_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(crocom_var_C, jp);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_2(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Crocomire(k as ::core::ffi::c_int))
        .base
        .instruction_timer = 1 as uint16;
    return addr_kCrocomire_Ilist_BADE as ::core::ffi::c_int as uintptr_t as *const uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_3(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Crocomire(0 as ::core::ffi::c_int)).crocom_var_C = 4 as uint16;
    return addr_kCrocomire_Ilist_BBCE as ::core::ffi::c_int as uintptr_t as *const uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_4(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    let mut v1: uint16 = abs16(
        ((*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
    );
    if v1 as ::core::ffi::c_int - 224 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E).crocom_var_B =
            ((*E).crocom_var_B as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
        (*E).crocom_var_C = 18 as uint16;
        return addr_kCrocomire_Ilist_BC56 as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_5(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (*E).crocom_var_B as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && {
            (*E).crocom_var_B = ((*E).crocom_var_B as ::core::ffi::c_int
                & !(0x800 as ::core::ffi::c_int)) as uint16;
            (*E).crocom_var_D as ::core::ffi::c_int != 0
        }
    {
        jp = addr_kCrocomire_Ilist_BC30 as ::core::ffi::c_int as uintptr_t as *const uint16;
        (*E).crocom_var_C = 12 as uint16;
    } else if (((*E).base.x_pos as ::core::ffi::c_int - g_word_A486A2 as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        jp = addr_kCrocomire_Ilist_BE7E as ::core::ffi::c_int as uintptr_t as *const uint16;
        (*E).crocom_var_C = 10 as uint16;
    } else if ((jp as *mut uint8)
        .offset_from(RomBankBase((*gEnemyData(0 as ::core::ffi::c_int)).bank))
        as ::core::ffi::c_long
        - addr_kCrocomire_Ilist_BC34 as ::core::ffi::c_int as ::core::ffi::c_long)
        as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        return addr_kCrocomire_Ilist_BBCE as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_14(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    if (*((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0xfff as ::core::ffi::c_int)
        - 1024 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
        (*E).crocom_var_C = 8 as uint16;
        (*E).crocom_var_F = 0 as uint16;
        return addr_kCrocomire_Ilist_BB36 as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_7(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    let mut crocom_var_B: uint16 = (*E).crocom_var_B;
    if crocom_var_B as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        (*E).crocom_var_B =
            (crocom_var_B as ::core::ffi::c_int & 0xf7ff as ::core::ffi::c_int) as uint16;
        jp = addr_kCrocomire_Ilist_BC30 as ::core::ffi::c_int as uintptr_t as *const uint16;
        (*E).crocom_var_C = 12 as uint16;
    } else {
        let mut crocom_var_F: uint16 = (*E).crocom_var_F;
        if crocom_var_F as ::core::ffi::c_int - 18 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            (*E).crocom_var_F =
                ((*E).crocom_var_F as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            SpawnEprojWithGfx(
                crocom_var_F,
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                addr_stru_868F8F as ::core::ffi::c_int as uint16,
            );
            QueueSfx3_Max6(0x1c as uint16);
        } else {
            jp = addr_kCrocomire_Ilist_BBCA as ::core::ffi::c_int as uintptr_t as *const uint16;
            (*E).crocom_var_C = 6 as uint16;
        }
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_8(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    let mut crocom_var_B: uint16 = (*E).crocom_var_B;
    if crocom_var_B as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        (*E).crocom_var_B =
            (crocom_var_B as ::core::ffi::c_int & 0xf7ff as ::core::ffi::c_int) as uint16;
        jp = addr_kCrocomire_Ilist_BC30 as ::core::ffi::c_int as uintptr_t as *const uint16;
        (*E).crocom_var_C = 12 as uint16;
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_9(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut v2: uint16 = 0;
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    let mut crocom_var_D: uint16 = (*E).crocom_var_D;
    if crocom_var_D as ::core::ffi::c_int != 0 && {
        v2 = (crocom_var_D as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        (*E).crocom_var_D = v2;
        (*E).crocom_var_D as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    } {
        (*E).crocom_var_C = 12 as uint16;
        return addr_kCrocomire_Ilist_BC34 as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else {
        (*E).crocom_var_C = 6 as uint16;
        return addr_kCrocomire_Ilist_BBCE as ::core::ffi::c_int as uintptr_t as *const uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_10(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if ((*E).base.x_pos as ::core::ffi::c_int - g_word_A486A2 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).crocom_var_C = 6 as uint16;
        return addr_kCrocomire_Ilist_BBCE as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_11(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    (*Get_Crocomire(0 as ::core::ffi::c_int)).crocom_var_C = 6 as uint16;
    return addr_kCrocomire_Ilist_BD2A as ::core::ffi::c_int as uintptr_t as *const uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_12(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    (*E).crocom_var_B =
        ((*E).crocom_var_B as ::core::ffi::c_int & !(0x400 as ::core::ffi::c_int)) as uint16;
    return addr_kCrocomire_Ilist_BCD8 as ::core::ffi::c_int as uintptr_t as *const uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_13(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (*E).crocom_var_B as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).crocom_var_B =
            ((*E).crocom_var_B as ::core::ffi::c_int & !(0x800 as ::core::ffi::c_int)) as uint16;
        (*E).crocom_var_C = 20 as uint16;
        return addr_kCrocomire_Ilist_BC30 as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else if ((jp as *mut uint8)
        .offset_from(RomBankBase((*gEnemyData(0 as ::core::ffi::c_int)).bank))
        as ::core::ffi::c_long
        - addr_kCrocomire_Ilist_BDA2 as ::core::ffi::c_int as ::core::ffi::c_long)
        as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        return addr_kCrocomire_Ilist_BD2A as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_14(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (*E).crocom_var_B as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).crocom_var_B =
            ((*E).crocom_var_B as ::core::ffi::c_int & !(0x800 as ::core::ffi::c_int)) as uint16;
        (*E).crocom_var_C = 12 as uint16;
        return addr_kCrocomire_Ilist_BC30 as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else if ((jp as *mut uint8)
        .offset_from(RomBankBase((*gEnemyData(0 as ::core::ffi::c_int)).bank))
        as ::core::ffi::c_long
        - addr_kCrocomire_Ilist_BDA2 as ::core::ffi::c_int as ::core::ffi::c_long)
        as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        return addr_kCrocomire_Ilist_BD2A as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_15(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (*E).crocom_var_B as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).crocom_var_B =
            ((*E).crocom_var_B as ::core::ffi::c_int & !(0x800 as ::core::ffi::c_int)) as uint16;
        (*E).crocom_var_C = 12 as uint16;
        return addr_kCrocomire_Ilist_BC30 as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else if ((jp as *mut uint8)
        .offset_from(RomBankBase((*gEnemyData(0 as ::core::ffi::c_int)).bank))
        as ::core::ffi::c_long
        - addr_kCrocomire_Ilist_BDA2 as ::core::ffi::c_int as ::core::ffi::c_long)
        as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        return addr_kCrocomire_Ilist_BD2A as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_16(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    let mut v1: uint16 =
        ((*E).crocom_var_D as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E).crocom_var_D = v1;
    if v1 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E).crocom_var_D = 0 as uint16;
        (*E).crocom_var_C = 6 as uint16;
        return addr_kCrocomire_Ilist_BBCE as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_17(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    let mut crocom_var_B: uint16 = (*E).crocom_var_B;
    if crocom_var_B as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        (*E).crocom_var_B = (crocom_var_B as ::core::ffi::c_int & 0x1f00 as ::core::ffi::c_int
            | 0xa000 as ::core::ffi::c_int) as uint16;
        (*E).crocom_var_D = 1 as uint16;
        (*E).crocom_var_E = 10 as uint16;
        (*E).crocom_var_C = 12 as uint16;
        QueueSfx2_Max6(0x54 as uint16);
    } else {
        (*Get_Crocomire(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        ))
        .crocom_var_C = 10 as uint16;
        return addr_kCrocomire_Ilist_BD8E as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_18(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut result: *const uint16 = Crocomire_Func_2(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        jp,
    );
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    (*E).crocom_var_B =
        ((*E).crocom_var_B as ::core::ffi::c_int | 0x200 as ::core::ffi::c_int) as uint16;
    (*E).crocom_var_D = 32 as uint16;
    (*E).crocom_var_C = 30 as uint16;
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_19(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut v2: bool = false;
    jp = Crocomire_Func_2(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        jp,
    );
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (*E).crocom_var_D == 0 || {
        v2 = (*E).crocom_var_D as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
        (*E).crocom_var_D = (*E).crocom_var_D.wrapping_sub(1);
        v2 as ::core::ffi::c_int != 0
    } {
        jp = Crocomire_Func_20(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
        (*E).crocom_var_C = 32 as uint16;
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_20(mut k: uint16) -> *const uint16 {
    (*Get_Crocomire(k as ::core::ffi::c_int)).crocom_var_C = 20 as uint16;
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    (*E).crocom_var_B = (*E).crocom_var_B;
    return addr_kCrocomire_Ilist_BAEA as ::core::ffi::c_int as uintptr_t as *const uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_21(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (*E).crocom_var_B as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        jp = Crocomire_Func_2(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            jp,
        );
        (*E).crocom_var_D = 16 as uint16;
        (*E).crocom_var_C = 34 as uint16;
    } else {
        jp = Crocomire_Func_20(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
        (*E).crocom_var_C = 32 as uint16;
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_22(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    k = *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
        as *mut uint16);
    jp = Crocomire_Func_2(k, jp);
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (*E).crocom_var_D == 0 {
        (*E).crocom_var_B =
            ((*E).crocom_var_B as ::core::ffi::c_int | 0x2000 as ::core::ffi::c_int) as uint16;
        jp = Crocomire_Func_3(k, jp);
        (*E).crocom_var_C = 36 as uint16;
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_23(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    k = *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
        as *mut uint16);
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (*E).base.x_pos as ::core::ffi::c_int - 672 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        jp = Crocomire_Func_3(k, jp);
        (*E).crocom_var_C = 36 as uint16;
        (*E).crocom_var_D = 3 as uint16;
    } else {
        if (*E).crocom_var_B as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*E).crocom_var_C = 38 as uint16;
            jp = Crocomire_Func_12(k, jp);
        }
        if (*E).crocom_var_B as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).crocom_var_D = 5 as uint16;
            jp = addr_kCrocomire_Ilist_BCD8 as ::core::ffi::c_int as uintptr_t as *const uint16;
            (*E).crocom_var_F = (*E).crocom_var_C;
            (*E).crocom_var_C = 42 as uint16;
        }
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_24(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut v2: bool = false;
    k = *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
        as *mut uint16);
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (*E).crocom_var_D == 0 || {
        v2 = (*E).crocom_var_D as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
        (*E).crocom_var_D = (*E).crocom_var_D.wrapping_sub(1);
        v2 as ::core::ffi::c_int != 0
    } {
        jp = Crocomire_Func_3(k, jp);
        (*E).crocom_var_C = 40 as uint16;
        (*E).crocom_var_B =
            ((*E).crocom_var_B as ::core::ffi::c_int & !(0x400 as ::core::ffi::c_int)) as uint16;
    } else {
        (*E).crocom_var_C = 36 as uint16;
        (*Get_Crocomire(0x40 as ::core::ffi::c_int)).crocom_var_D = 0 as uint16;
        (*E).crocom_var_B =
            ((*E).crocom_var_B as ::core::ffi::c_int | 0x400 as ::core::ffi::c_int) as uint16;
        return addr_kCrocomire_Ilist_BCD8 as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_25(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    k = *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
        as *mut uint16);
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    let mut crocom_var_B: uint16 = (*E).crocom_var_B;
    if crocom_var_B as ::core::ffi::c_int & 0x2000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*E).crocom_var_B =
            (crocom_var_B as ::core::ffi::c_int & 0xfcff as ::core::ffi::c_int) as uint16;
    }
    jp = Crocomire_Func_3(k, jp);
    (*E).crocom_var_C = 40 as uint16;
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_26(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (*E).crocom_var_D != 0 {
        let mut crocom_var_B: uint16 = (*E).crocom_var_B;
        if crocom_var_B as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).crocom_var_D = (*E).crocom_var_D.wrapping_sub(1);
            QueueSfx2_Max6(0x3b as uint16);
            return addr_kCrocomire_Ilist_BCD8 as ::core::ffi::c_int as uintptr_t as *const uint16;
        } else {
            (*E).crocom_var_B =
                (crocom_var_B as ::core::ffi::c_int & 0xbfff as ::core::ffi::c_int) as uint16;
            (*E).crocom_var_C = 12 as uint16;
        }
    } else {
        (*E).crocom_var_B =
            ((*E).crocom_var_B as ::core::ffi::c_int & !(0x4000 as ::core::ffi::c_int)) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).crocom_var_C = (*E).crocom_var_F;
        return addr_kCrocomire_Ilist_BCD8 as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Init() {
    let mut v7: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    *((&raw mut g_ram as *mut uint8).offset(0x179c as ::core::ffi::c_int as isize)
        as *mut uint16) = 6 as uint16;
    let mut v0: uint16 = 0 as uint16;
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int - 4096 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    if *((&raw mut g_ram as *mut uint8)
        .offset(0xd828 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as *mut uint8 as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xcd20 as ::core::ffi::c_int as isize)
            as *mut uint16) = 257 as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0xcd20 as ::core::ffi::c_int as isize)
            .offset(2 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 257 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x688 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & 0x7bff as ::core::ffi::c_int
            | 0x400 as ::core::ffi::c_int) as uint16;
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
            field_0: 0x20 as uint8,
            field_1: 0x3 as uint8,
            field_2: 0xb753 as VoidP,
        });
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
            field_0: 0x1e as uint8,
            field_1: 0x3 as uint8,
            field_2: 0xb753 as VoidP,
        });
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
            field_0: 0x61 as uint8,
            field_1: 0xb as uint8,
            field_2: 0xb747 as VoidP,
        });
        (*E).crocom_var_A = 84 as uint16;
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E1CC as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.x_pos = 576 as uint16;
        (*E).base.y_pos = 144 as uint16;
        (*E).base.y_height = 28 as uint16;
        (*E).base.x_width = 40 as uint16;
        let mut v6: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16);
        v7 = gVramWriteEntry(
            *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        );
        (*v7).size = 2048 as uint16;
        (*v7).src.addr = 0x2000 as VoidP;
        *(&raw mut (*v7).src.bank as *mut uint16) = 126 as uint16;
        (*v7).vram_dst = ((*(&raw mut g_ram as *mut uint8)
            .offset(0x59 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v6 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    } else {
        DisableMinimapAndMarkBossRoomAsExplored();
        *((&raw mut g_ram as *mut uint8).offset(0x69a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        let mut E_0: *mut Enemy_Crocomire = Get_Crocomire(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        );
        (*E_0).crocom_var_A = 0 as uint16;
        (*E_0).crocom_var_E = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xcd20 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        let mut i: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            let mut v3: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
            *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 160 as ::core::ffi::c_int) as isize) = *(g_rom.offset(
                (((0xa4b8bd as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa4b8bd as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8
                as *mut uint16)
                .offset(v3 as isize);
            *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 + 208 as ::core::ffi::c_int) as isize) = *(g_rom.offset(
                (((0xa4b8dd as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa4b8dd as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8
                as *mut uint16)
                .offset(v3 as isize);
            i -= 2 as ::core::ffi::c_int;
        }
        (*E_0).crocom_var_C = 4 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x179e as ::core::ffi::c_int as isize)
            as *mut uint16) = 16 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x941 as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x179a as ::core::ffi::c_int as isize)
            as *mut uint16) = 1024 as uint16;
        (*E_0).base.current_instruction =
            addr_kCrocomire_Ilist_BADE as ::core::ffi::c_int as uint16;
        (*E_0).base.extra_properties = ((*E_0).base.extra_properties as ::core::ffi::c_int
            | 4 as ::core::ffi::c_int) as uint16;
        (*E_0).base.instruction_timer = 1 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_27(mut k: uint16) {
    *((&raw mut g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize) as *mut uint16) =
        (67 as ::core::ffi::c_int
            - (*Get_Crocomire(k as ::core::ffi::c_int)).base.y_pos as ::core::ffi::c_int)
            as uint16;
    let mut v1: uint16 = 32 as uint16;
    let mut spritemap_pointer: uint16 = 0;
    loop {
        spritemap_pointer = (*Get_Crocomire(0 as ::core::ffi::c_int))
            .base
            .spritemap_pointer;
        if spritemap_pointer as ::core::ffi::c_int
            == *(g_rom.offset(
                (((0xa48b79 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa48b79 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
        {
            break;
        }
        v1 = (v1 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            Crocomire_8BA4();
            return;
        }
    }
    let ref mut fresh8 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb7 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh8 = (*fresh8 as ::core::ffi::c_int
        + *(RomPtr_A4(spritemap_pointer as uint16_t) as *mut uint16)
            .offset(14 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        as uint16;
    Crocomire_8BA4();
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_8BA4() {
    let mut current_block: u64;
    let mut E0: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut E1: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
    );
    (*E1).base.x_pos = ((*E1).crocom_var_A as ::core::ffi::c_int
        + (*E0).base.x_pos as ::core::ffi::c_int) as uint16;
    (*E1).base.y_pos = (*E0).base.y_pos;
    let mut x_pos: uint16 = (*E0).base.x_pos;
    if (x_pos as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        if (*E0).base.x_pos as ::core::ffi::c_int
            - 128 as ::core::ffi::c_int
            - (*((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 256 as ::core::ffi::c_int)
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            current_block = 6937071982253665452;
        } else {
            current_block = 1762290943679506230;
        }
    } else if ((x_pos as ::core::ffi::c_int + 128 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        current_block = 1762290943679506230;
    } else {
        current_block = 6937071982253665452;
    }
    match current_block {
        6937071982253665452 => {
            let mut v6: uint16 = (*((&raw mut g_ram as *mut uint8)
                .offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - (*E0).base.x_pos as ::core::ffi::c_int
                + 51 as ::core::ffi::c_int) as uint16;
            let mut v4: uint16 = abs16(v6);
            if v4 as ::core::ffi::c_int - 284 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v6 = 256 as uint16;
            }
            *((&raw mut g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
                as *mut uint16) = v6;
            return;
        }
        _ => {
            *((&raw mut g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize)
                as *mut uint16) = 256 as uint16;
            return;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn nullsub_305() {}
static mut off_A48C14: [Option<Func_V>; 45] = unsafe {
    [
        Some(Crocomire_Func_28 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_52 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_36 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_92CE as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_54 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_36 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_92CE as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_54 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_57 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_60 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_62 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_929E as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_64 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_65 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_66 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_51 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_8D47 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_929E as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_51 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_8D47 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_929E as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_51 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_59 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_60 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_62 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_63 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_56 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_9506 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_65 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_66 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_49 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_68 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_70 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_71 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_72 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_73 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_88 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_9B65 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_90 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_89 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_9B86 as unsafe extern "C" fn() -> ()),
        Some(nullsub_305 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_91 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_29 as unsafe extern "C" fn() -> ()),
        Some(Crocomire_Func_69 as unsafe extern "C" fn() -> ()),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Main() {
    let mut v0: ::core::ffi::c_int = (*Get_Crocomire(0 as ::core::ffi::c_int)).crocom_var_A
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    off_A48C14[v0 as usize].expect("non-null function pointer")();
    Crocomire_Func_30();
    Crocomire_Func_31();
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_28() {
    Crocomire_Func_37();
    *((&raw mut g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 257 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 1312 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut g_ram as *mut uint8)
            .offset(0xcd20 as ::core::ffi::c_int as isize)
            .offset(4 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 256 as uint16;
    }
    Crocomire_Func_27(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_29() {
    *((&raw mut g_ram as *mut uint8).offset(0xb5 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_30() {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (*E).crocom_var_A == 0
        && (((*E).base.x_pos as ::core::ffi::c_int
            - (*E).base.x_width as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xafe as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
    {
        NormalEnemyTouchAi();
        *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) = ((*E).base.x_pos as ::core::ffi::c_int
            - (*E).base.x_width as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xafe as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xb58 as ::core::ffi::c_int as isize)
            as *mut uint16) = -(4 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb5c as ::core::ffi::c_int as isize)
            as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_31() {
    let mut j: uint16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0x797 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        if (*Get_Crocomire(0 as ::core::ffi::c_int)).base.flash_timer as ::core::ffi::c_int != 0
            && *((&raw mut g_ram as *mut uint8).offset(0xe44 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 2 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
        {
            let mut i: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
            while i >= 0 as ::core::ffi::c_int {
                *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(
                        ((i >> 1 as ::core::ffi::c_int) + 112 as ::core::ffi::c_int) as isize,
                    ) = 0x7fff as uint16;
                i -= 2 as ::core::ffi::c_int;
            }
        } else {
            j = 14 as uint16;
            while j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(
                        ((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                            + 112 as ::core::ffi::c_int) as isize,
                    ) = *(g_rom.offset(
                    (((0xa4b89d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa4b89d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
                j = (j as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_11(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max6(0x74 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_7(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max6(0x25 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_19(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    QueueSfx2_Max6(0x75 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_35() {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    let mut crocom_var_20: uint16 = (*E).crocom_var_20;
    if crocom_var_20 != 0 {
        let mut v2: uint16 =
            (crocom_var_20 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        (*E).crocom_var_20 = v2;
        if v2 == 0 {
            (*E).crocom_var_20 = 32 as uint16;
            QueueSfx3_Max6(0x22 as uint16);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_36() {
    Crocomire_Func_35();
    Crocomire_8BA4();
    Crocomire_8D47();
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_8D47() {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    let mut crocom_var_D: uint16 = (*E).crocom_var_D;
    if crocom_var_D != 0 {
        (*E).crocom_var_D =
            (crocom_var_D as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        (*E).crocom_var_E = 768 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_37() {
    let mut E0: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (*E0).base.x_pos as ::core::ffi::c_int - 1536 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0x9000 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9006 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x900a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    } else if (*E0).base.x_pos as ::core::ffi::c_int - 1552 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x9000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0x9000 as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
            eproj_spawn_pt = Point16U {
                x: 1536 as uint16,
                y: 176 as uint16,
            };
            SpawnEprojWithRoomGfx(
                addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
                0x15 as uint16,
            );
        }
    } else if (*E0).base.x_pos as ::core::ffi::c_int - 1552 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9006 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x900a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    } else if (*E0).base.x_pos as ::core::ffi::c_int - 1568 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0x9002 as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
            SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
                field_0: 0x61 as uint8,
                field_1: 0xb as uint8,
                field_2: 0xb74b as VoidP,
            });
            eproj_spawn_pt = Point16U {
                x: 1568 as uint16,
                y: 176 as uint16,
            };
            SpawnEprojWithRoomGfx(
                addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
                0x15 as uint16,
            );
        }
    } else {
        let mut x_pos: uint16 = (*E0).base.x_pos;
        if x_pos as ::core::ffi::c_int - 1568 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0x9006 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x900a as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        } else if x_pos as ::core::ffi::c_int - 1584 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if *((&raw mut g_ram as *mut uint8).offset(0x9006 as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
            {
                *((&raw mut g_ram as *mut uint8).offset(0x9006 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 1 as uint16;
                SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
                    field_0: 0x62 as uint8,
                    field_1: 0xb as uint8,
                    field_2: 0xb74b as VoidP,
                });
                SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
                    field_0: 0x63 as uint8,
                    field_1: 0xb as uint8,
                    field_2: 0xb74b as VoidP,
                });
                eproj_spawn_pt = Point16U {
                    x: 1584 as uint16,
                    y: 176 as uint16,
                };
                SpawnEprojWithRoomGfx(
                    addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
                    0x15 as uint16,
                );
            }
        } else if (x_pos as ::core::ffi::c_int - g_word_A486A4 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            Crocomire_8EE5();
            (*E0).crocom_var_20 = 1 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x9018 as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
            let mut v3: uint16 = *((&raw mut g_ram as *mut uint8)
                .offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16);
            let mut EK: *mut Enemy_Crocomire = Get_Crocomire(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int,
            );
            (*EK).crocom_var_A =
                ((*EK).crocom_var_A as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            (*Get_Crocomire(0 as ::core::ffi::c_int)).crocom_var_00 = 2 as uint16;
            (*Get_Crocomire(0x40 as ::core::ffi::c_int)).crocom_var_00 = 2 as uint16;
            (*Get_Crocomire(0x80 as ::core::ffi::c_int)).crocom_var_00 = 2 as uint16;
            (*Get_Crocomire(0xc0 as ::core::ffi::c_int)).crocom_var_00 = 2 as uint16;
            (*Get_Crocomire(0x100 as ::core::ffi::c_int)).crocom_var_00 = 2 as uint16;
            (*Get_Crocomire(0x140 as ::core::ffi::c_int)).crocom_var_00 = 2 as uint16;
            (*Get_Crocomire(0 as ::core::ffi::c_int)).crocom_var_01 = 0 as uint16;
            (*Get_Crocomire(0x40 as ::core::ffi::c_int)).crocom_var_01 = 0 as uint16;
            (*Get_Crocomire(0x80 as ::core::ffi::c_int)).crocom_var_01 = 0 as uint16;
            (*Get_Crocomire(0xc0 as ::core::ffi::c_int)).crocom_var_01 = 0 as uint16;
            (*Get_Crocomire(0x100 as ::core::ffi::c_int)).crocom_var_01 = 0 as uint16;
            (*Get_Crocomire(0x140 as ::core::ffi::c_int)).crocom_var_01 = 0 as uint16;
            QueueSfx2_Max6(0x3b as uint16);
            *(((&raw mut g_ram as *mut uint8).offset(0x9015 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
            SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
                field_0: 0x4e as uint8,
                field_1: 0x3 as uint8,
                field_2: 0xb757 as VoidP,
            });
            (*EK).base.current_instruction =
                addr_kCrocomire_Ilist_BFB0 as ::core::ffi::c_int as uint16;
            (*EK).base.instruction_timer = 1 as uint16;
            (*EK).base.properties = ((*EK).base.properties as ::core::ffi::c_int
                | kEnemyProps_Tangible as ::core::ffi::c_int)
                as uint16;
            let mut E1: *mut Enemy_Crocomire = Get_Crocomire(
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + 64 as ::core::ffi::c_int,
            );
            (*E1).base.instruction_timer = 0x7fff as uint16;
            (*E1).base.current_instruction =
                addr_kCrocomire_Ilist_BF62 as ::core::ffi::c_int as uint16;
            let fresh11 = &raw mut (*Get_Crocomire(0x40 as ::core::ffi::c_int)).base.properties;
            fresh11.write_unaligned(
                (fresh11.read_unaligned() as ::core::ffi::c_int | 0x100 as ::core::ffi::c_int)
                    as uint16,
            );
            (*EK).crocom_var_E = 0 as uint16;
            (*EK).crocom_var_F = 0 as uint16;
            (*EK).crocom_var_D = 2048 as uint16;
            (*Get_Crocomire(0 as ::core::ffi::c_int)).base.y_height = 16 as uint16;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_8EE5() {
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
        field_0: 0x61 as uint8,
        field_1: 0xb as uint8,
        field_2: 0xb74f as VoidP,
    });
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
        field_0: 0x62 as uint8,
        field_1: 0xb as uint8,
        field_2: 0xb74f as VoidP,
    });
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
        field_0: 0x63 as uint8,
        field_1: 0xb as uint8,
        field_2: 0xb74f as VoidP,
    });
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
        field_0: 0x64 as uint8,
        field_1: 0xb as uint8,
        field_2: 0xb74f as VoidP,
    });
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
        field_0: 0x65 as uint8,
        field_1: 0xb as uint8,
        field_2: 0xb74f as VoidP,
    });
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
        field_0: 0x66 as uint8,
        field_1: 0xb as uint8,
        field_2: 0xb74f as VoidP,
    });
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
        field_0: 0x67 as uint8,
        field_1: 0xb as uint8,
        field_2: 0xb74f as VoidP,
    });
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
        field_0: 0x68 as uint8,
        field_1: 0xb as uint8,
        field_2: 0xb74f as VoidP,
    });
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
        field_0: 0x69 as uint8,
        field_1: 0xb as uint8,
        field_2: 0xb74f as VoidP,
    });
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
        field_0: 0x6a as uint8,
        field_1: 0xb as uint8,
        field_2: 0xb74f as VoidP,
    });
    eproj_spawn_pt = Point16U {
        x: 1536 as uint16,
        y: 176 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        0x15 as uint16,
    );
    eproj_spawn_pt = Point16U {
        x: 1552 as uint16,
        y: 192 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        0x15 as uint16,
    );
    eproj_spawn_pt = Point16U {
        x: 1568 as uint16,
        y: 176 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        0x15 as uint16,
    );
    eproj_spawn_pt = Point16U {
        x: 1584 as uint16,
        y: 192 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        0x15 as uint16,
    );
    eproj_spawn_pt = Point16U {
        x: 1600 as uint16,
        y: 192 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        0x15 as uint16,
    );
    eproj_spawn_pt = Point16U {
        x: 1616 as uint16,
        y: 192 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        0x15 as uint16,
    );
    eproj_spawn_pt = Point16U {
        x: 1632 as uint16,
        y: 192 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        0x15 as uint16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_2(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    *((&raw mut g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = 4 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) = 5 as uint16;
    QueueSfx2_Max6(0x76 as uint16);
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_4(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    if (*Get_Crocomire(0 as ::core::ffi::c_int)).crocom_var_B as ::core::ffi::c_int
        & 0x800 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        Enemy_MoveRight_IgnoreSlopes(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            (-(4 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int,
        );
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_3(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    Crocomire_Func_43();
    return Crocomire_Instr_4(k, jp);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_15(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    Crocomire_Func_43();
    return Crocomire_Instr_4(k, jp);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_43() {
    let mut v1: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0x5e5 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int) as uint16;
    if (*((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 4096 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        v1 = -(v1 as ::core::ffi::c_int) as uint16;
    }
    Crocomire_Func_87(::core::ptr::null::<uint16>(), v1);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_16(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
        (-(4 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int,
    ) != 0
    {
        (*Get_Crocomire(0 as ::core::ffi::c_int)).crocom_var_C = 14 as uint16;
        return addr_kCrocomire_Ilist_BF3C as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else {
        let mut v2: uint16 = 32 as uint16;
        if *((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 2048 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v2 = -(32 as ::core::ffi::c_int) as uint16;
        }
        return Crocomire_Func_87(
            jp,
            (v2 as ::core::ffi::c_int
                + (*((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int)) as uint16,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_13(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (((*E).base.x_pos as ::core::ffi::c_int
        - (*E).base.x_width as ::core::ffi::c_int
        - 260 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        Enemy_MoveRight_IgnoreSlopes(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
            (4 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
        );
    }
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_18(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    Enemy_MoveRight_IgnoreSlopes(
        k,
        (4 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
    );
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_12(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    Crocomire_Func_43();
    return Crocomire_Instr_13(k, jp);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_17(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    Crocomire_Func_43();
    return Crocomire_Instr_18(k, jp);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_49() {
    Crocomire_Func_50();
    Crocomire_Func_53();
    Crocomire_Func_55();
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (*E).crocom_var_A as ::core::ffi::c_int == 62 as ::core::ffi::c_int {
        QueueMusic_Delayed8(6 as uint16);
        (*E).crocom_var_A = 88 as uint16;
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E1D2 as ::core::ffi::c_int as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0xcd20 as ::core::ffi::c_int as isize)
            .offset(4 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 257 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x5f7 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        let fresh12 = &raw mut (*Get_Crocomire(0x40 as ::core::ffi::c_int)).base.properties;
        fresh12.write_unaligned(
            (fresh12.read_unaligned() as ::core::ffi::c_int | 0x200 as ::core::ffi::c_int)
                as uint16,
        );
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
            field_0: 0x4e as uint8,
            field_1: 0x3 as uint8,
            field_2: 0xb753 as VoidP,
        });
        *((&raw mut g_ram as *mut uint8).offset(0x941 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x688 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_50() {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    let mut y_pos: uint16 = (*E).base.y_pos;
    let mut v2: uint16 = addr_kCrocomire_Ilist_BF7E as ::core::ffi::c_int as uint16;
    if y_pos as ::core::ffi::c_int - 280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v2 = addr_kCrocomire_Ilist_BF86 as ::core::ffi::c_int as uint16;
        if y_pos as ::core::ffi::c_int - 264 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v2 = addr_kCrocomire_Ilist_BF8C as ::core::ffi::c_int as uint16;
            if y_pos as ::core::ffi::c_int - 248 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v2 = addr_kCrocomire_Ilist_BF92 as ::core::ffi::c_int as uint16;
            }
        }
    }
    (*E).base.current_instruction = v2;
    (*E).base.instruction_timer = 1 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_51() {
    Crocomire_Func_53();
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    let mut y_pos: uint16 = (*E).base.y_pos;
    let mut v2: uint16 = addr_kCrocomire_Ilist_BF64 as ::core::ffi::c_int as uint16;
    if y_pos as ::core::ffi::c_int - 280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v2 = addr_kCrocomire_Ilist_BF6C as ::core::ffi::c_int as uint16;
        if y_pos as ::core::ffi::c_int - 264 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v2 = addr_kCrocomire_Ilist_BF72 as ::core::ffi::c_int as uint16;
            if y_pos as ::core::ffi::c_int - 248 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v2 = addr_kCrocomire_Ilist_BF78 as ::core::ffi::c_int as uint16;
            }
        }
    }
    (*E).base.current_instruction = v2;
    (*E).base.instruction_timer = 1 as uint16;
    Crocomire_Func_55();
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_52() {
    let mut v0: uint16 = *(((&raw mut g_ram as *mut uint8)
        .offset(0x9015 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16);
    if *(((&raw mut g_ram as *mut uint8).offset(0x9015 as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16) as ::core::ffi::c_int
        - 22 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v1: uint16 =
            *(((&raw mut g_ram as *mut uint8).offset(0x9015 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(1 as ::core::ffi::c_int as isize) as *mut uint16);
        let ref mut fresh17 =
            *(((&raw mut g_ram as *mut uint8).offset(0x9015 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(1 as ::core::ffi::c_int as isize) as *mut uint16);
        *fresh17 = (*fresh17 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        SpawnEprojWithGfx(
            *(g_rom.offset(
                (((0xa49156 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa49156 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
            v1,
            addr_stru_868F9D as ::core::ffi::c_int as uint16,
        );
    }
    Crocomire_Func_54();
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_53() {
    let ref mut fresh9 = *((&raw mut g_ram as *mut uint8)
        .offset(0x9018 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh9 = (*fresh9).wrapping_sub(1);
    if *fresh9 == 0 {
        *((&raw mut g_ram as *mut uint8).offset(0x9018 as ::core::ffi::c_int as isize)
            as *mut uint16) = 6 as uint16;
        let mut v0: uint16 = (*((&raw mut g_ram as *mut uint8)
            .offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x3f as ::core::ffi::c_int) as uint16;
        if *((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 2 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            v0 = !(v0 as ::core::ffi::c_int) as uint16;
        }
        let mut x: uint16 = ((*Get_Crocomire(0 as ::core::ffi::c_int)).base.x_pos
            as ::core::ffi::c_int
            + v0 as ::core::ffi::c_int) as uint16;
        let mut y: uint16 = (*((&raw mut g_ram as *mut uint8)
            .offset(0x1962 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int
            - ((*((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x1f00 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int)) as uint16;
        CreateSpriteAtPos(x, y, 21 as uint16, 0 as uint16);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_54() {
    Crocomire_Func_35();
    Crocomire_Func_53();
    Crocomire_Func_55();
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_55() {
    let mut crocom_var_D: int8 = 0;
    let mut v7: int8 = 0;
    let mut v9: int8 = 0;
    let mut crocom_var_E: int8 = 0;
    let mut crocom_var_E_high: int8 = 0;
    let mut v12: int8 = 0;
    Crocomire_Func_58();
    let mut E0: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    (*E0).crocom_var_B =
        ((*E0).crocom_var_B as ::core::ffi::c_int & !(0x800 as ::core::ffi::c_int)) as uint16;
    Crocomire_Func_27(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*E).base.y_pos as ::core::ffi::c_int - 280 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: uint16 = (8 as ::core::ffi::c_int
            * (*((&raw mut g_ram as *mut uint8).offset(0xb7 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 288 as ::core::ffi::c_int
                & 0xfff8 as ::core::ffi::c_int)) as uint16;
        let mut m: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
        loop {
            *((&raw mut g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                824 as uint16;
            v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            m -= 1;
            if !(m != 0) {
                break;
            }
        }
        *((&raw mut g_ram as *mut uint8).offset(0xe1e as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        (*E0).base.extra_properties = ((*E0).base.extra_properties as ::core::ffi::c_int
            & !(0x8000 as ::core::ffi::c_int)) as uint16;
        crocom_var_D = (*E).crocom_var_D as int8;
        *(&raw mut (*E).crocom_var_D as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
            (crocom_var_D as ::core::ffi::c_int + 0x80 as ::core::ffi::c_int) as uint8;
        v7 = (__CFADD__uint8(crocom_var_D as uint8, 0x80 as uint8) as ::core::ffi::c_int
            + *(&raw mut (*E).crocom_var_D as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            + 3 as ::core::ffi::c_int) as int8;
        if v7 as ::core::ffi::c_int - 48 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 {
            v7 = 48 as int8;
        }
        *(&raw mut (*E).crocom_var_D as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) = v7 as uint8;
        let mut v8: bool = __CFADD__uint8(
            *(&raw mut (*E).crocom_var_E as *mut uint8).offset(0 as ::core::ffi::c_int as isize),
            v7 as uint8,
        );
        let ref mut fresh13 =
            *(&raw mut (*E).crocom_var_E as *mut uint8).offset(0 as ::core::ffi::c_int as isize);
        *fresh13 = (*fresh13 as ::core::ffi::c_int + v7 as ::core::ffi::c_int) as uint8;
        v9 = (v8 as ::core::ffi::c_int
            + *(&raw mut (*E).crocom_var_E as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int) as int8;
        if v9 as ::core::ffi::c_int - 3 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 {
            v9 = 3 as int8;
        }
        *(&raw mut (*E).crocom_var_E as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) = v9 as uint8;
        crocom_var_E = (*E).crocom_var_E as int8;
        v8 = __CFADD__uint8(
            *(&raw mut (*E).crocom_var_F as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ),
            crocom_var_E as uint8,
        );
        let ref mut fresh14 = *(&raw mut (*E).crocom_var_F as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
        *fresh14 = (*fresh14 as ::core::ffi::c_int + crocom_var_E as ::core::ffi::c_int) as uint8;
        crocom_var_E_high = *(&raw mut (*E).crocom_var_E as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as int8;
        let mut v13: bool = v8;
        v8 = __CFADD__uint8(v8 as uint8, crocom_var_E_high as uint8);
        v12 = (v13 as ::core::ffi::c_int + crocom_var_E_high as ::core::ffi::c_int) as int8;
        v8 = (v8 as ::core::ffi::c_int
            | __CFADD__uint8(
                *(&raw mut (*E).base.y_pos as *mut uint8).offset(0 as ::core::ffi::c_int as isize),
                v12 as uint8,
            ) as ::core::ffi::c_int)
            != 0;
        let ref mut fresh15 =
            *(&raw mut (*E).base.y_pos as *mut uint8).offset(0 as ::core::ffi::c_int as isize);
        *fresh15 = (*fresh15 as ::core::ffi::c_int + v12 as ::core::ffi::c_int) as uint8;
        let ref mut fresh16 = *(&raw mut (*E).base.y_pos as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
        *fresh16 = (*fresh16 as ::core::ffi::c_int + v8 as ::core::ffi::c_int) as uint8;
    } else {
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        (*E0).crocom_var_D = 48 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_56() {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut y_pos: uint16 = (*E).base.y_pos;
    let mut v2: uint16 = addr_kCrocomire_Ilist_BF7E as ::core::ffi::c_int as uint16;
    if y_pos as ::core::ffi::c_int - 280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v2 = addr_kCrocomire_Ilist_BF86 as ::core::ffi::c_int as uint16;
        if y_pos as ::core::ffi::c_int - 264 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v2 = addr_kCrocomire_Ilist_BF8C as ::core::ffi::c_int as uint16;
            if y_pos as ::core::ffi::c_int - 248 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v2 = addr_kCrocomire_Ilist_BF92 as ::core::ffi::c_int as uint16;
            }
        }
    }
    (*E).base.current_instruction = v2;
    (*E).base.instruction_timer = 1 as uint16;
    Crocomire_Func_53();
    Crocomire_92D8();
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_929E() {
    Crocomire_Func_53();
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    let mut y_pos: uint16 = (*E).base.y_pos;
    let mut v2: uint16 = addr_kCrocomire_Ilist_BF64 as ::core::ffi::c_int as uint16;
    if y_pos as ::core::ffi::c_int - 280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v2 = addr_kCrocomire_Ilist_BF6C as ::core::ffi::c_int as uint16;
        if y_pos as ::core::ffi::c_int - 264 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v2 = addr_kCrocomire_Ilist_BF72 as ::core::ffi::c_int as uint16;
            if y_pos as ::core::ffi::c_int - 248 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v2 = addr_kCrocomire_Ilist_BF78 as ::core::ffi::c_int as uint16;
            }
        }
    }
    (*E).base.current_instruction = v2;
    (*E).base.instruction_timer = 1 as uint16;
    Crocomire_92D8();
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_92CE() {
    Crocomire_Func_35();
    Crocomire_Func_53();
    Crocomire_92D8();
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_92D8() {
    let mut v3: int16 = 0;
    let mut v6: int8 = 0;
    Crocomire_Func_58();
    let mut EK: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if (*EK).base.y_pos as ::core::ffi::c_int - 218 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*EK).crocom_var_A = (*EK).crocom_var_A.wrapping_add(1);
        (*EK).crocom_var_A = (*EK).crocom_var_A.wrapping_add(1);
    } else {
        Crocomire_Func_27(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
        let mut E0: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
        v3 = ((*E0).crocom_var_D as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as int16;
        if (*E0).crocom_var_D as ::core::ffi::c_int - 7680 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v3 = 7936 as int16;
        }
        (*E0).crocom_var_D = v3 as uint16;
        let mut crocom_var_E: uint8 = (*E0).crocom_var_E as uint8;
        let mut v5: bool = (crocom_var_E as ::core::ffi::c_int)
            < *(&raw mut (*E0).crocom_var_D as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int;
        *(&raw mut (*E0).crocom_var_E as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
            (crocom_var_E as ::core::ffi::c_int
                - *(&raw mut (*E0).crocom_var_D as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int) as uint8;
        v6 = (*(&raw mut (*E0).crocom_var_E as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            - v5 as ::core::ffi::c_int) as int8;
        if (v6 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            *(&raw mut (*E0).crocom_var_E as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
                -(1 as ::core::ffi::c_int) as uint8;
            v6 = 0 as int8;
        }
        *(&raw mut (*E0).crocom_var_E as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) = v6 as uint8;
        let mut crocom_var_F_high: uint8 = *(&raw mut (*E0).crocom_var_F as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
        v5 = (crocom_var_F_high as ::core::ffi::c_int)
            < *(&raw mut (*E0).crocom_var_E as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int;
        *(&raw mut (*E0).crocom_var_F as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) = (crocom_var_F_high as ::core::ffi::c_int
            - *(&raw mut (*E0).crocom_var_E as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int) as uint8;
        let mut y_pos: uint8 = (*E0).base.y_pos as uint8;
        let mut v9: uint8 = (v5 as ::core::ffi::c_int
            + *(&raw mut (*E0).crocom_var_E as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int) as uint8;
        *(&raw mut (*E0).base.y_pos as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
            (y_pos as ::core::ffi::c_int - v9 as ::core::ffi::c_int) as uint8;
        let ref mut fresh7 = *(&raw mut (*E0).base.y_pos as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
        *fresh7 = (*fresh7 as ::core::ffi::c_int
            - ((y_pos as ::core::ffi::c_int) < v9 as ::core::ffi::c_int) as ::core::ffi::c_int)
            as uint8;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_57() {
    let mut i: ::core::ffi::c_int = 0;
    *(&raw mut g_ram as *mut uint8).offset(0x68c as ::core::ffi::c_int as isize) = 48 as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0x688 as ::core::ffi::c_int as isize) as *mut uint16) =
        48 as uint16;
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).base.current_instruction = addr_kCrocomire_Ilist_BF64 as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xe1e as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    let mut E1: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
    );
    (*E1).base.current_instruction = addr_kCrocomire_Ilist_BF98 as ::core::ffi::c_int as uint16;
    (*E1).base.instruction_timer = 1 as uint16;
    (*E1).base.properties = ((*E1).base.properties as ::core::ffi::c_int
        & 0xd2ff as ::core::ffi::c_int
        | 0x2c00 as ::core::ffi::c_int) as uint16;
    (*E1).base.x_pos = (*E).base.x_pos;
    (*E1).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
    let mut v3: uint16 = 0 as uint16;
    loop {
        let mut v4: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize) = 824 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v4 + 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
        v3 = (v3 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
        if !(((v3 as ::core::ffi::c_int - 1024 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    i = 0 as ::core::ffi::c_int;
    loop {
        let mut v6: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        if *(g_rom.offset(
            (((0xa49c79 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa49c79 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v6 as isize) as ::core::ffi::c_int
            == 0xffff as ::core::ffi::c_int
        {
            break;
        }
        *((&raw mut g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 32 as ::core::ffi::c_int) as isize) = *(g_rom.offset(
            (((0xa49c79 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa49c79 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset(v6 as isize);
        i += 2 as ::core::ffi::c_int;
    }
    Crocomire_93BE((i + 1024 as ::core::ffi::c_int) as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_93BE(mut k: uint16) {
    let mut v3: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    v3 = gVramWriteEntry(
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v3).size = k;
    (*v3).src.addr = 0x2000 as VoidP;
    *(&raw mut (*v3).src.bank as *mut uint16) = 126 as uint16;
    (*v3).vram_dst = ((*(&raw mut g_ram as *mut uint8).offset(0x59 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16) =
        (v2 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_58() {
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb7 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut i: ::core::ffi::c_int = 510 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xcaf0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = v0;
        i -= 2 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_59() {
    let mut i: ::core::ffi::c_int = 0;
    Crocomire_Func_58();
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).base.instruction_timer = 1 as uint16;
    *(&raw mut g_ram as *mut uint8).offset(0x68c as ::core::ffi::c_int as isize) = 48 as uint8;
    *((&raw mut g_ram as *mut uint8).offset(0x688 as ::core::ffi::c_int as isize) as *mut uint16) =
        48 as uint16;
    (*E).base.current_instruction = addr_kCrocomire_Ilist_BF7E as ::core::ffi::c_int as uint16;
    let mut v1: uint16 = 0 as uint16;
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v1 as ::core::ffi::c_int - 2048 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    i = 0 as ::core::ffi::c_int;
    loop {
        let mut v3: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        if *(g_rom.offset(
            (((0xa49e7b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa49e7b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize) as ::core::ffi::c_int
            == 0xffff as ::core::ffi::c_int
        {
            break;
        }
        *((&raw mut g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v3 + 32 as ::core::ffi::c_int) as isize) = *(g_rom.offset(
            (((0xa49e7b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa49e7b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset(v3 as isize);
        i += 2 as ::core::ffi::c_int;
    }
    Crocomire_93BE((i + 1024 as ::core::ffi::c_int) as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_60() {
    let mut i: ::core::ffi::c_int = 0;
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).crocom_var_A =
        ((*E).crocom_var_A as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x692 as ::core::ffi::c_int as isize) as *mut uint16) =
        256 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x690 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x698 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((g_rom.offset(
            (((0xa49bc5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa49bc5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16 as *mut uint8)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0x69a as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int as isize,
            ) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x694 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0x698 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x696 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((g_rom.offset(
            (((0xa49bc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa49bc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16 as *mut uint8)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0x69a as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int as isize,
            ) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x68e as ::core::ffi::c_int as isize) as *mut uint16) =
        *((g_rom.offset(
            (((0xa49bc9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa49bc9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16 as *mut uint8)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0x69a as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int as isize,
            ) as *mut uint16);
    let mut bank: uint8 = *((g_rom.offset(
        (((0xa49bcb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa49bcb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16 as *mut uint8)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0x69a as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int as isize,
        ) as *mut uint16) as uint8;
    let mut v6: uint16 = 0;
    i = *((&raw mut g_ram as *mut uint8).offset(0x69a as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + 8 as ::core::ffi::c_int;
    loop {
        let mut v2: uint16 = *((g_rom.offset(
            (((0xa49bc5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa49bc5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16 as *mut uint8)
            .offset(i as isize) as *mut uint16);
        if v2 as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
            break;
        }
        v6 = i as uint16;
        let mut v3: uint16 = *((g_rom.offset(
            (((0xa49bc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa49bc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16 as *mut uint8)
            .offset(i as isize) as *mut uint16);
        let mut n: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x68e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int;
        loop {
            *((&raw mut g_ram as *mut uint8).offset(v3 as isize) as *mut uint8 as *mut uint16) =
                *(RomPtrWithBank(bank, v2 as uint16_t) as *mut uint16);
            v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            n -= 1;
            if !(n >= 0 as ::core::ffi::c_int) {
                break;
            }
        }
        i = v6 as ::core::ffi::c_int + 4 as ::core::ffi::c_int;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x69a as ::core::ffi::c_int as isize) as *mut uint16) =
        (i + 2 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x68a as ::core::ffi::c_int as isize) as *mut uint16) =
        (i + 2 as ::core::ffi::c_int) as uint16;
    let mut j: ::core::ffi::c_int = 128 as ::core::ffi::c_int;
    while j >= 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8)
            .offset(0x69c as ::core::ffi::c_int as isize)
            .offset(j as uint16 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        j -= 2 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_61() {
    Crocomire_Func_62();
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_62() {
    let mut v2: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x68a as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v1: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    if *((g_rom.offset(
        (((0xa49bc5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa49bc5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16 as *mut uint8)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0x68a as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int as isize,
        ) as *mut uint16) as ::core::ffi::c_int
        == 0xffff as ::core::ffi::c_int
    {
        let mut E: *mut Enemy_Crocomire = Get_Crocomire(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        );
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        *((&raw mut g_ram as *mut uint8).offset(0x68a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    } else {
        v2 = gVramWriteEntry(
            *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        );
        (*v2).size = *((g_rom.offset(
            (((0xa49bc5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa49bc5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16 as *mut uint8)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0x68a as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int as isize,
            ) as *mut uint16);
        (*v2).src.addr = *((g_rom.offset(
            (((0xa49bcb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa49bcb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16)
            as VoidP;
        *(&raw mut (*v2).src.bank as *mut uint16) =
            *((g_rom.offset(
                (((0xa49bc9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa49bc9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16 as *mut uint8)
                .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16);
        (*v2).vram_dst = *((g_rom.offset(
            (((0xa49bc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa49bc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0x68a as ::core::ffi::c_int as isize)
            as *mut uint16) = (v0 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v1 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_64() {
    QueueSfx2_Max6(0x77 as uint16);
    Crocomire_950F();
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_9506() {
    QueueSfx2_Max6(0x2d as uint16);
    Crocomire_950F();
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_950F() {
    static mut unk_A49559: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x42 as uint8,
        field_1: 0x10 as uint8,
        addr: 0x9563 as VoidP,
    };
    let mut v4: *mut Enemy_Crocomire = ::core::ptr::null_mut::<Enemy_Crocomire>();
    let mut v1: uint16 = (2 as ::core::ffi::c_int
        * ((*Get_Crocomire(0x40 as ::core::ffi::c_int)).base.y_pos as ::core::ffi::c_int
            - 72 as ::core::ffi::c_int)) as uint16;
    let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb7 as ::core::ffi::c_int as isize) as *mut uint16);
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0xcaf0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v2;
        v1 = (v1 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if !(v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    v4 = Get_Crocomire(0 as ::core::ffi::c_int);
    (*Get_Crocomire(0x80 as ::core::ffi::c_int)).crocom_var_D = (*v4).base.x_pos;
    *((&raw mut g_ram as *mut uint8).offset(0xcae0 as ::core::ffi::c_int as isize)
        as *mut uint16) = 255 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0xcae0 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
        -(13584 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0xcae0 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 225 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0xcae0 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
        -(13330 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0xcae0 as ::core::ffi::c_int as isize)
        .offset(6 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
    (*v4).crocom_var_1F = SpawnHdmaObject(
        0xa4 as uint8,
        &raw const unk_A49559 as *const ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_63() {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).crocom_var_A =
        ((*E).crocom_var_A as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_65() {
    let mut v4: uint16 = 0;
    let mut current_block: u64;
    let mut i: ::core::ffi::c_int = 0;
    Crocomire_Func_53();
    let mut E0: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    let mut E1: *mut Enemy_Crocomire = Get_Crocomire(0x40 as ::core::ffi::c_int);
    let mut E2: *mut Enemy_Crocomire = Get_Crocomire(0x80 as ::core::ffi::c_int);
    let mut crocom_var_D: uint16 = (*E2).crocom_var_D;
    (*E1).crocom_var_D = (*E1).crocom_var_D.wrapping_sub(1);
    if (*E1).crocom_var_D as ::core::ffi::c_int & 2 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        crocom_var_D = (crocom_var_D as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    }
    (*E0).base.x_pos = crocom_var_D;
    if !(Crocomire_Func_67() == 0) {
        Crocomire_Func_27(
            *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
        v4 = 0;
        v4 = (*((&raw mut g_ram as *mut uint8).offset(0x694 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 3 as ::core::ffi::c_int) as uint16;
        if *((&raw mut g_ram as *mut uint8).offset(0x694 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 19 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if (*((&raw mut g_ram as *mut uint8).offset(0x692 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 20480 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                current_block = 11812396948646013369;
            } else {
                v4 = 16 as uint16;
                current_block = 15652330335145281839;
            }
        } else {
            current_block = 15652330335145281839;
        }
        match current_block {
            11812396948646013369 => {}
            _ => {
                *((&raw mut g_ram as *mut uint8).offset(0x694 as ::core::ffi::c_int as isize)
                    as *mut uint16) = v4;
                let mut v5: uint16 = (*((&raw mut g_ram as *mut uint8)
                    .offset(0x692 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + 384 as ::core::ffi::c_int) as uint16;
                if *((&raw mut g_ram as *mut uint8).offset(0x692 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 20096 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    v5 = 20480 as uint16;
                }
                *((&raw mut g_ram as *mut uint8).offset(0x692 as ::core::ffi::c_int as isize)
                    as *mut uint16) = v5;
                let mut r18: uint16 = 0 as uint16;
                let mut v6: uint16 = (2 as ::core::ffi::c_int
                    * ((*E1).base.y_pos as ::core::ffi::c_int - 72 as ::core::ffi::c_int))
                    as uint16;
                let mut v7: uint16 = *((&raw mut g_ram as *mut uint8)
                    .offset(0x694 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *((&raw mut g_ram as *mut uint8).offset(0x5a6 as ::core::ffi::c_int as isize)
                    as *mut uint16) = *((&raw mut g_ram as *mut uint8)
                    .offset(0x694 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                loop {
                    *((&raw mut g_ram as *mut uint8)
                        .offset(0xcaf0 as ::core::ffi::c_int as isize) as *mut uint16)
                        .offset(
                            (v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                                as isize,
                        ) = (*((&raw mut g_ram as *mut uint8)
                        .offset(0xb7 as ::core::ffi::c_int as isize) as *mut uint16)
                        as ::core::ffi::c_int + v7 as ::core::ffi::c_int
                        - *((&raw mut g_ram as *mut uint8)
                            .offset(0x5a6 as ::core::ffi::c_int as isize) as *mut uint16)
                            as ::core::ffi::c_int) as uint16;
                    let mut v8: bool = __CFADD__uint16(
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0x692 as ::core::ffi::c_int as isize)
                            as *mut uint16),
                        r18,
                    );
                    r18 = (r18 as ::core::ffi::c_int
                        + *((&raw mut g_ram as *mut uint8)
                            .offset(0x692 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int)
                        as uint16;
                    if !v8 {
                        v7 = v7.wrapping_add(1);
                    }
                    let ref mut fresh18 = *((&raw mut g_ram as *mut uint8)
                        .offset(0x5a6 as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    *fresh18 = (*fresh18).wrapping_add(1);
                    v6 = (v6 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                    if !(((v7 as ::core::ffi::c_int
                        - *((&raw mut g_ram as *mut uint8)
                            .offset(0x698 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int)
                        as int16 as ::core::ffi::c_int)
                        < 0 as ::core::ffi::c_int)
                    {
                        break;
                    }
                }
                if ((v6 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
                {
                    let mut v9: uint16 = *((&raw mut g_ram as *mut uint8)
                        .offset(0xb7 as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    loop {
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xcaf0 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(
                                (v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                            ) = v9;
                        v6 = (v6 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                        if !(((v6 as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16
                            as ::core::ffi::c_int)
                            < 0 as ::core::ffi::c_int)
                        {
                            break;
                        }
                    }
                }
                return;
            }
        }
    }
    let mut EK: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*EK).crocom_var_A =
        ((*EK).crocom_var_A as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    i = *((&raw mut g_ram as *mut uint8).offset(0x69a as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
    while *((g_rom.offset(
        (((0xa49bc5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa49bc5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16 as *mut uint8)
        .offset(i as isize) as *mut uint16) as ::core::ffi::c_int
        != 0xffff as ::core::ffi::c_int
    {
        i += 8 as ::core::ffi::c_int;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x69a as ::core::ffi::c_int as isize) as *mut uint16) =
        (i + 2 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x18b4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(((*E0).crocom_var_1F as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_66() {
    let mut v3: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*E).crocom_var_E = 0 as uint16;
    (*E).crocom_var_F = 0 as uint16;
    (*E).crocom_var_D = 2048 as uint16;
    let mut i: ::core::ffi::c_int = 4094 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0x2000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    v3 = gVramWriteEntry(
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v3).size = 2048 as uint16;
    (*v3).src.addr = 0x2000 as VoidP;
    *(&raw mut (*v3).src.bank as *mut uint16) = 126 as uint16;
    (*v3).vram_dst = ((*(&raw mut g_ram as *mut uint8).offset(0x59 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 0xfe as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16) =
        (v2 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    Crocomire_9BB3();
}
static mut kCrocoEraseLineMasks: [uint8; 8] = [
    0x7f as ::core::ffi::c_int as uint8,
    0xbf as ::core::ffi::c_int as uint8,
    0xdf as ::core::ffi::c_int as uint8,
    0xef as ::core::ffi::c_int as uint8,
    0xf7 as ::core::ffi::c_int as uint8,
    0xfb as ::core::ffi::c_int as uint8,
    0xfd as ::core::ffi::c_int as uint8,
    0xfe as ::core::ffi::c_int as uint8,
];
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_67() -> uint16 {
    let mut n: ::core::ffi::c_int = *(&raw mut g_ram as *mut uint8)
        .offset(0x68c as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int;
    loop {
        if *((&raw mut g_ram as *mut uint8).offset(0x690 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            > 48 as ::core::ffi::c_int
        {
            return 1 as uint16;
        }
        let mut rr: ::core::ffi::c_int = *(g_rom.offset(
            (((0xa49697 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa49697 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0x690 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int;
        if ((*(&raw mut g_ram as *mut uint8)
            .offset(0x69c as ::core::ffi::c_int as isize)
            .offset(rr as isize) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x688 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int8 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            break;
        }
        let ref mut fresh0 = *((&raw mut g_ram as *mut uint8)
            .offset(0x690 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh0 = (*fresh0).wrapping_add(1);
        if *fresh0 as ::core::ffi::c_int >= 128 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0x690 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            return 0 as uint16;
        }
    }
    '_c2rust_label: {
        if *((&raw mut g_ram as *mut uint8).offset(0x690 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            <= 48 as ::core::ffi::c_int
        {
        } else {
            __assert_fail(
                b"croco_cur_vline_idx <= 48\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_a4.c\0" as *const u8 as *const ::core::ffi::c_char,
                1215 as ::core::ffi::c_uint,
                b"uint16 Crocomire_Func_67(void)\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    };
    let mut vline_idx: ::core::ffi::c_int = *(g_rom.offset(
        (((0xa49697 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa49697 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint8)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0x690 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as ::core::ffi::c_int;
    let mut mask: uint8 = kCrocoEraseLineMasks[(vline_idx & 7 as ::core::ffi::c_int) as usize];
    loop {
        let mut q: ::core::ffi::c_int = *(&raw mut g_ram as *mut uint8)
            .offset(0x69c as ::core::ffi::c_int as isize)
            .offset(vline_idx as isize)
            as ::core::ffi::c_int;
        let mut j: ::core::ffi::c_int = 2 as ::core::ffi::c_int * (q & 7 as ::core::ffi::c_int)
            + ((q & !(7 as ::core::ffi::c_int)) << 6 as ::core::ffi::c_int)
            + 4 as ::core::ffi::c_int * (vline_idx & !(7 as ::core::ffi::c_int));
        let ref mut fresh1 = (*((&raw mut g_ram as *mut uint8)
            .offset(0x4000 as ::core::ffi::c_int as isize)
            as *mut Ram4000))
            .backups
            .field_0[(j + 0 as ::core::ffi::c_int) as usize];
        *fresh1 = (*fresh1 as ::core::ffi::c_int & mask as ::core::ffi::c_int) as uint8;
        let ref mut fresh2 = (*((&raw mut g_ram as *mut uint8)
            .offset(0x4000 as ::core::ffi::c_int as isize)
            as *mut Ram4000))
            .backups
            .field_0[(j + 1 as ::core::ffi::c_int) as usize];
        *fresh2 = (*fresh2 as ::core::ffi::c_int & mask as ::core::ffi::c_int) as uint8;
        let ref mut fresh3 = (*((&raw mut g_ram as *mut uint8)
            .offset(0x4000 as ::core::ffi::c_int as isize)
            as *mut Ram4000))
            .backups
            .field_0[(j + 16 as ::core::ffi::c_int) as usize];
        *fresh3 = (*fresh3 as ::core::ffi::c_int & mask as ::core::ffi::c_int) as uint8;
        let ref mut fresh4 = (*((&raw mut g_ram as *mut uint8)
            .offset(0x4000 as ::core::ffi::c_int as isize)
            as *mut Ram4000))
            .backups
            .field_0[(j + 17 as ::core::ffi::c_int) as usize];
        *fresh4 = (*fresh4 as ::core::ffi::c_int & mask as ::core::ffi::c_int) as uint8;
        if *(&raw mut g_ram as *mut uint8)
            .offset(0x69c as ::core::ffi::c_int as isize)
            .offset(vline_idx as isize) as ::core::ffi::c_int
            == 48 as ::core::ffi::c_int
        {
            break;
        }
        let ref mut fresh5 = *(&raw mut g_ram as *mut uint8)
            .offset(0x69c as ::core::ffi::c_int as isize)
            .offset(vline_idx as isize);
        *fresh5 = (*fresh5).wrapping_add(1);
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
    let mut v7: uint16 = 0;
    let mut v8: uint16 = 0;
    loop {
        v7 = (*((&raw mut g_ram as *mut uint8).offset(0x68a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0x69a as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        v8 = *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16);
        if *((g_rom.offset(
            (((0xa49bc5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa49bc5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16 as *mut uint8)
            .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16)
            as ::core::ffi::c_int
            != 0xffff as ::core::ffi::c_int
        {
            break;
        }
        *((&raw mut g_ram as *mut uint8).offset(0x68a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    let mut v9: *mut VramWriteEntry = gVramWriteEntry(
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    (*v9).size = *((g_rom.offset(
        (((0xa49bc5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa49bc5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16 as *mut uint8)
        .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16);
    (*v9).src.addr = *((g_rom.offset(
        (((0xa49bcb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa49bcb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16 as *mut uint8)
        .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16) as VoidP;
    *(&raw mut (*v9).src.bank as *mut uint16) = *((g_rom.offset(
        (((0xa49bc9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa49bc9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16
        as *mut uint8)
        .offset(v7 as ::core::ffi::c_int as isize)
        as *mut uint16);
    (*v9).vram_dst = *((g_rom.offset(
        (((0xa49bc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa49bc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16 as *mut uint8)
        .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16) =
        (v8 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    let ref mut fresh6 = *((&raw mut g_ram as *mut uint8)
        .offset(0x68a as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh6 = (*fresh6 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
    return 1 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_68() {
    if *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 640 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        QueueMusic_Delayed8(5 as uint16);
        *((&raw mut g_ram as *mut uint8)
            .offset(0xcd20 as ::core::ffi::c_int as isize)
            .offset(3 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 256 as uint16;
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
            field_0: 0x30 as uint8,
            field_1: 0x3 as uint8,
            field_2: 0xb757 as VoidP,
        });
        *((&raw mut g_ram as *mut uint8).offset(0x941 as ::core::ffi::c_int as isize)
            as *mut uint16) = 6 as uint16;
        let mut E0: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
        (*E0).base.properties = ((*E0).base.properties as ::core::ffi::c_int
            & 0x7bff as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
        let mut E1: *mut Enemy_Crocomire = Get_Crocomire(0x40 as ::core::ffi::c_int);
        (*E1).base.properties =
            ((*E1).base.properties as ::core::ffi::c_int | 0x500 as ::core::ffi::c_int) as uint16;
        (*E0).crocom_var_D = 4 as uint16;
        (*E1).crocom_var_D = 0 as uint16;
        (*Get_Crocomire(0x80 as ::core::ffi::c_int)).crocom_var_D = 10 as uint16;
        (*Get_Crocomire(0xc0 as ::core::ffi::c_int)).crocom_var_D = 1 as uint16;
        (*E0).crocom_var_B = 0 as uint16;
        (*E0).base.y_height = 56 as uint16;
        Crocomire_9BB3();
    }
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_69() {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    let mut v1: uint16 =
        ((*E).base.x_pos as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    (*E).base.x_pos = v1;
    if v1 as ::core::ffi::c_int - 480 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E).base.x_pos = 480 as uint16;
        (*E).base.y_pos = 54 as uint16;
        (*E).crocom_var_A = 62 as uint16;
    } else {
        (*E).base.y_pos = 220 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_70() {
    let mut E0: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    let mut E1: *mut Enemy_Crocomire = Get_Crocomire(0x40 as ::core::ffi::c_int);
    let mut E2: *mut Enemy_Crocomire = Get_Crocomire(0x80 as ::core::ffi::c_int);
    let mut E3: *mut Enemy_Crocomire = Get_Crocomire(0xc0 as ::core::ffi::c_int);
    if *(g_rom.offset(
        (((0xa498ca as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa498ca as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(((*E0).crocom_var_D as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        == 0x8080 as ::core::ffi::c_int
    {
        (*E1).crocom_var_D = -(32640 as ::core::ffi::c_int) as uint16;
        (*E0).crocom_var_D = 128 as uint16;
        let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 176 as ::core::ffi::c_int) as isize) =
                *(g_rom.offset(
                    (((0xa4b91d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa4b91d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((i >> 1 as ::core::ffi::c_int) as isize);
            i -= 2 as ::core::ffi::c_int;
        }
        Crocomire_9BB3();
    } else {
        let mut crocom_var_D: uint16 = (*E0).crocom_var_D;
        let mut v3: uint16 = (*E1).crocom_var_D;
        let mut v4: ::core::ffi::c_int =
            crocom_var_D as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if v3 as ::core::ffi::c_int
            == *(g_rom.offset(
                (((0xa498ca as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa498ca as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v4 as isize) as ::core::ffi::c_int
        {
            if *(g_rom.offset(
                (((0xa498ca as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa498ca as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((crocom_var_D as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                let mut v7: uint16 = (*E2).crocom_var_D;
                if v7 != 0 {
                    (*E2).crocom_var_D =
                        (v7 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
                    (*E0).crocom_var_D =
                        (crocom_var_D as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
                    QueueSfx2_Max6(0x2b as uint16);
                    return;
                }
                let mut v8: uint16 =
                    (crocom_var_D as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                (*E2).crocom_var_D = *(g_rom.offset(
                    (((0xa498ca as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa498ca as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
                crocom_var_D = (v8 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                (*E3).crocom_var_D = *(g_rom.offset(
                    (((0xa498ca as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa498ca as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(
                        (crocom_var_D as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                    );
            }
            (*E0).crocom_var_D =
                (crocom_var_D as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        } else {
            let mut v5: uint16 = 0;
            if (v3 as ::core::ffi::c_int
                - *(g_rom.offset(
                    (((0xa498ca as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa498ca as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v4 as isize) as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                v5 =
                    (v3 as ::core::ffi::c_int - (*E3).crocom_var_D as ::core::ffi::c_int) as uint16;
            } else {
                v5 =
                    ((*E3).crocom_var_D as ::core::ffi::c_int + v3 as ::core::ffi::c_int) as uint16;
            }
            (*E1).crocom_var_D = v5;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_71() {
    let mut v3: int16 = 0;
    let mut v6: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    let mut crocom_var_D: uint16 = (*E).crocom_var_D;
    if crocom_var_D != 0 {
        (*E).crocom_var_D =
            (crocom_var_D as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        v3 = *((&raw mut g_ram as *mut uint8).offset(0x688 as ::core::ffi::c_int as isize)
            as *mut uint16) as int16;
        let mut v4: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0x688 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        if *(g_rom.offset(
            (((0xa499cb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa499cb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v4 as isize) as ::core::ffi::c_int
            != 0xffff as ::core::ffi::c_int
        {
            let mut v5: uint16 = *((&raw mut g_ram as *mut uint8)
                .offset(0x330 as ::core::ffi::c_int as isize)
                as *mut uint16);
            v6 = gVramWriteEntry(
                *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int,
            );
            (*v6).size = 512 as uint16;
            (*v6).src.addr = *(g_rom.offset(
                (((0xa499d9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa499d9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v4 as isize) as VoidP;
            *(&raw mut (*v6).src.bank as *mut uint16) = 173 as uint16;
            (*v6).vram_dst = (*(g_rom.offset(
                (((0xa499cb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa499cb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v4 as isize) as ::core::ffi::c_int
                + ((*(&raw mut g_ram as *mut uint8).offset(0x52 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & 7 as ::core::ffi::c_int)
                    << 13 as ::core::ffi::c_int)) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
                as *mut uint16) = (v5 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x688 as ::core::ffi::c_int as isize)
                as *mut uint16) = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        }
    } else {
        (*E).base.x_pos = 480 as uint16;
        (*E).base.y_pos = 54 as uint16;
        (*Get_Crocomire(0x80 as ::core::ffi::c_int)).crocom_var_D = 80 as uint16;
        (*E).crocom_var_E = 0 as uint16;
        (*E).crocom_var_F = 0 as uint16;
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
            field_0: 0x20 as uint8,
            field_1: 0x3 as uint8,
            field_2: 0xb753 as VoidP,
        });
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
            field_0: 0x1e as uint8,
            field_1: 0x3 as uint8,
            field_2: 0xb757 as VoidP,
        });
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
            field_0: 0x70 as uint8,
            field_1: 0xb as uint8,
            field_2: 0xb747 as VoidP,
        });
        QueueSfx2_Max6(0x29 as uint16);
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E158 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.palette_index = 0 as uint16;
        let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 144 as ::core::ffi::c_int) as isize) =
                *(g_rom.offset(
                    (((0xa4b8fd as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa4b8fd as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((i >> 1 as ::core::ffi::c_int) as isize);
            i -= 2 as ::core::ffi::c_int;
        }
        ClearEprojs();
        let mut v8: uint16 = 8 as uint16;
        let mut n: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
        loop {
            SpawnEprojWithGfx(
                v8,
                *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                addr_kEproj_CrocomireSpikeWallPieces as ::core::ffi::c_int as uint16,
            );
            n -= 1;
            if !(n != 0) {
                break;
            }
        }
        QueueSfx2_Max6(0x30 as uint16);
        Crocomire_9BB3();
    };
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_72() {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (*E).base.x_pos as ::core::ffi::c_int - 224 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut t: uint32 = (((*E).crocom_var_F as uint32) << 16 as ::core::ffi::c_int
            | (*E).crocom_var_E as uint32)
            .wrapping_add(0x8000 as uint32);
        if (t >> 16 as ::core::ffi::c_int).wrapping_sub(2 as uint32) & 0x8000 as uint32 == 0 {
            t = t & 0xffff as uint32
                | ((2 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as uint32;
        }
        SetHiLo(&raw mut (*E).crocom_var_F, &raw mut (*E).crocom_var_E, t);
        AddToHiLo(&raw mut (*E).base.x_pos, &raw mut (*E).base.x_subpos, t);
    }
    let mut E2: *mut Enemy_Crocomire = Get_Crocomire(0x80 as ::core::ffi::c_int);
    if (*E2).crocom_var_D as ::core::ffi::c_int != 0 && {
        let fresh19 = (*E2).crocom_var_D;
        (*E2).crocom_var_D = (*E2).crocom_var_D.wrapping_sub(1);
        fresh19 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
    } {
        (*E).crocom_var_E = 0 as uint16;
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E14A as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        Crocomire_9BB3();
    }
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_73() {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    let mut t: uint32 = (((*E).crocom_var_F as uint32) << 16 as ::core::ffi::c_int
        | (*E).crocom_var_E as uint32)
        .wrapping_add(2048 as uint32);
    if (t >> 16 as ::core::ffi::c_int).wrapping_sub(5 as uint32) & 0x8000 as uint32 == 0 {
        t = t & 0xffff as uint32
            | ((5 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as uint32;
    }
    SetHiLo(&raw mut (*E).crocom_var_F, &raw mut (*E).crocom_var_E, t);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        0xe000 as uint32,
    );
    AddToHiLo(&raw mut (*E).base.x_pos, &raw mut (*E).base.x_subpos, t);
    if (*E).base.x_pos as ::core::ffi::c_int - 576 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        QueueSfx2_Max6(0x25 as uint16);
        (*Get_Crocomire(0x40 as ::core::ffi::c_int))
            .base
            .palette_index = (*E).base.palette_index;
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E158 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        Crocomire_9BB3();
    }
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_8(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return Crocomire_Func_87(jp, 0xffe0 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_6(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return Crocomire_Func_87(jp, 0 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_9(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return Crocomire_Func_87(jp, 0xfff0 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_5(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return Crocomire_Func_87(jp, 0x10 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_20(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return Crocomire_Func_87(jp, 0 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_21(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return Crocomire_Func_87(jp, 8 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_22(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return Crocomire_Func_87(jp, 0x10 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_23(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return Crocomire_Func_87(jp, 0x18 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_24(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return Crocomire_Func_87(jp, 0x20 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_10(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return Crocomire_Func_87(jp, 0x28 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_25(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return Crocomire_Func_87(jp, 0x30 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_26(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return Crocomire_Func_87(jp, 0x38 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Instr_27(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    return Crocomire_Func_87(jp, 0x40 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_87(mut jp: *const uint16, mut a: uint16) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    eproj_spawn_pt = Point16U {
        x: (a as ::core::ffi::c_int
            + (*E).base.x_pos as ::core::ffi::c_int
            + (*((&raw mut g_ram as *mut uint8).offset(0x5e5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 7 as ::core::ffi::c_int)) as uint16,
        y: ((*E).base.y_height as ::core::ffi::c_int + (*E).base.y_pos as ::core::ffi::c_int
            - 16 as ::core::ffi::c_int) as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        0x15 as uint16,
    );
    return jp;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_88() {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (*E).base.current_instruction as ::core::ffi::c_int + 0x1e3a as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut t: uint32 = (((*E).crocom_var_F as uint32) << 16 as ::core::ffi::c_int
            | (*E).crocom_var_E as uint32)
            .wrapping_add(4096 as uint32);
        if (t >> 16 as ::core::ffi::c_int).wrapping_sub(6 as uint32) & 0x8000 as uint32 == 0 {
            t = t & 0xffff as uint32
                | ((6 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as uint32;
        }
        SetHiLo(&raw mut (*E).crocom_var_F, &raw mut (*E).crocom_var_E, t);
    } else {
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E1CC as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.x_pos =
            ((*E).base.x_pos as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
        (*E).base.y_pos =
            ((*E).base.y_pos as ::core::ffi::c_int + 21 as ::core::ffi::c_int) as uint16;
        (*E).base.y_height = 28 as uint16;
        (*E).base.x_width = 40 as uint16;
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
            field_0: 0x30 as uint8,
            field_1: 0x3 as uint8,
            field_2: 0xb753 as VoidP,
        });
        Enemy_ItemDrop_Crocomire(0 as uint16);
        Crocomire_9BB3();
    };
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_9B65() {
    *((&raw mut g_ram as *mut uint8).offset(0xcd20 as ::core::ffi::c_int as isize)
        as *mut uint16) = 257 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0xcd20 as ::core::ffi::c_int as isize)
        .offset(2 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 257 as uint16;
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
        field_0: 0x1e as uint8,
        field_1: 0x3 as uint8,
        field_2: 0xb753 as VoidP,
    });
    Crocomire_9BB3();
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_89() {
    Crocomire_9BB3();
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_90() {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (*E).base.current_instruction as ::core::ffi::c_int + 0x1e3a as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        Crocomire_9BB3();
    }
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_9B86() {
    QueueMusic_Delayed8(6 as uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x941 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    let ref mut fresh10 = *((&raw mut g_ram as *mut uint8)
        .offset(0xd828 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as *mut uint8 as *mut uint16);
    *fresh10 = (*fresh10 as ::core::ffi::c_int | 2 as ::core::ffi::c_int) as uint16;
    QueueMusic_Delayed8(6 as uint16);
    Crocomire_Func_87(::core::ptr::null::<uint16>(), 0xfff0 as uint16);
    Crocomire_Func_87(::core::ptr::null::<uint16>(), 0x10 as uint16);
    Crocomire_9BB3();
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_9BB3() {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_91() {
    Crocomire_9BB3();
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_92() {
    NormalEnemyTouchAi();
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    (*E).crocom_var_B =
        ((*E).crocom_var_B as ::core::ffi::c_int | 0x4000 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb58 as ::core::ffi::c_int as isize) as *mut uint16) =
        -(4 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_93() {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    let mut v1: uint16 =
        ((*E).crocom_var_B as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as uint16;
    if v1 as ::core::ffi::c_int - 15 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v1 = v1.wrapping_add(1);
    }
    (*E).crocom_var_B =
        ((*E).crocom_var_B as ::core::ffi::c_int | v1 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_94() {
    let mut v0: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x18a6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
    eproj_spawn_pt = Point16U {
        x: *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v0 as isize),
        y: *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v0 as isize),
    };
    let mut v1: uint16 = (if *((&raw mut g_ram as *mut uint8)
        .offset(0xc18 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v0 as isize) as ::core::ffi::c_int
        & 0x200 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        6 as ::core::ffi::c_int
    } else {
        29 as ::core::ffi::c_int
    }) as uint16;
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        v1,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Powerbomb() {
    let mut v2: uint16 = 0;
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    if (*E).crocom_var_A == 0 {
        if g_word_A4869E != 0 {
            (*E).crocom_var_D = g_word_A4869E;
            if (*E).crocom_var_C as ::core::ffi::c_int != 24 as ::core::ffi::c_int {
                (*E).crocom_var_B = ((*E).crocom_var_B as ::core::ffi::c_int
                    & 0x3ff0 as ::core::ffi::c_int
                    | 0x8000 as ::core::ffi::c_int) as uint16;
                (*E).crocom_var_E = 10 as uint16;
                (*E).base.flash_timer = ((*E).base.flash_timer as ::core::ffi::c_int
                    + 4 as ::core::ffi::c_int) as uint16;
                (*E).base.ai_handler_bits = ((*E).base.ai_handler_bits as ::core::ffi::c_int
                    | 2 as ::core::ffi::c_int)
                    as uint16;
                (*E).crocom_var_C = 24 as uint16;
                let mut spritemap_pointer: uint16 = (*E).base.spritemap_pointer;
                let mut v3: uint16 = 0;
                let mut n: ::core::ffi::c_int = *(RomPtr_A4(spritemap_pointer as uint16_t)
                    as *mut uint16)
                    as ::core::ffi::c_int;
                loop {
                    v2 = *(RomPtr_A4(spritemap_pointer as uint16_t) as *mut uint16)
                        .offset(3 as ::core::ffi::c_int as isize);
                    v3 = addr_kCrocomire_Ilist_BDAE as ::core::ffi::c_int as uint16;
                    if v2 as ::core::ffi::c_int
                        == addr_kCrocomire_BigSprmap_D600 as ::core::ffi::c_int
                    {
                        break;
                    }
                    v3 = addr_kCrocomire_Ilist_BDB2 as ::core::ffi::c_int as uint16;
                    if v2 as ::core::ffi::c_int
                        == addr_kCrocomire_BigSprmap_D51C as ::core::ffi::c_int
                    {
                        break;
                    }
                    spritemap_pointer = (spritemap_pointer as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int)
                        as uint16;
                    n -= 1;
                    if !(n == 0) {
                        continue;
                    }
                    v3 = addr_kCrocomire_Ilist_BDB6 as ::core::ffi::c_int as uint16;
                    break;
                }
                let mut E0: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
                (*E0).base.current_instruction = v3;
                (*E0).base.instruction_timer = 1 as uint16;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_95() {
    let mut v1: uint16 = 0;
    let mut v2: uint16 = 0;
    let mut current_block: u64;
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int);
    (*E).base.invincibility_timer = 0 as uint16;
    if ((*E).base.x_pos as ::core::ffi::c_int
        - (*E).base.x_width as ::core::ffi::c_int
        - 256 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        current_block = 4956146061682418353;
    } else {
        v1 = 0;
        v2 = 0;
        v1 = *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0x18a6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            );
        if v1 as ::core::ffi::c_int & 0xf00 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            v3 = (v1 as ::core::ffi::c_int & 0xf00 as ::core::ffi::c_int) as int16;
            v2 = g_word_A4869A;
            if v3 as ::core::ffi::c_int != 256 as ::core::ffi::c_int {
                v2 = g_word_A4869C;
                if v3 as ::core::ffi::c_int != 512 as ::core::ffi::c_int {
                    v2 = 0 as uint16;
                }
            }
        } else {
            v2 = g_word_A48698;
            if v1 as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                (*E).base.instruction_timer = g_word_A486A0;
                Crocomire_Func_1();
                return;
            }
        }
        if v2 != 0 {
            (*E).crocom_var_D =
                ((*E).crocom_var_D as ::core::ffi::c_int + v2 as ::core::ffi::c_int) as uint16;
            current_block = 4956146061682418353;
        } else {
            current_block = 7172762164747879670;
        }
    }
    match current_block {
        4956146061682418353 => {
            let mut v4: uint16 =
                ((*E).crocom_var_B as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as uint16;
            if v4 as ::core::ffi::c_int - 15 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v4 = v4.wrapping_add(1);
            }
            if (*E).crocom_var_B as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                let mut v5: uint16 = g_word_A48692;
                if (*E).crocom_var_C as ::core::ffi::c_int == 8 as ::core::ffi::c_int {
                    v5 = g_word_A48694;
                }
                (*E).base.instruction_timer = ((*E).base.instruction_timer as ::core::ffi::c_int
                    + v5 as ::core::ffi::c_int)
                    as uint16;
            }
            (*E).crocom_var_B = (v4 as ::core::ffi::c_int
                | (*E).crocom_var_B as ::core::ffi::c_int & 0xb7f0 as ::core::ffi::c_int
                | 0x800 as ::core::ffi::c_int) as uint16;
            (*E).crocom_var_E = 10 as uint16;
        }
        _ => {}
    }
    (*E).base.flash_timer =
        ((*E).base.flash_timer as ::core::ffi::c_int + 14 as ::core::ffi::c_int) as uint16;
    (*E).base.ai_handler_bits =
        ((*E).base.ai_handler_bits as ::core::ffi::c_int | 2 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Crocomire_Func_1() {
    let mut v0: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x18a6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int;
    eproj_spawn_pt = Point16U {
        x: *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v0 as isize),
        y: *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v0 as isize),
    };
    let mut v1: uint16 = (if *((&raw mut g_ram as *mut uint8)
        .offset(0xc18 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v0 as isize) as ::core::ffi::c_int
        & 0x200 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        6 as ::core::ffi::c_int
    } else {
        29 as ::core::ffi::c_int
    }) as uint16;
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        v1,
    );
}
#[no_mangle]
pub unsafe extern "C" fn CrocomireTongue_Init() {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut g_ram as *mut uint8).offset(0xe54 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    );
    if *((&raw mut g_ram as *mut uint8)
        .offset(0xd828 as ::core::ffi::c_int as isize)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ) as *mut uint8 as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & 0xdcff as ::core::ffi::c_int
            | 0x300 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).base.current_instruction = addr_kCrocomire_Ilist_BE56 as ::core::ffi::c_int as uint16;
        (*E).base.extra_properties = ((*E).base.extra_properties as ::core::ffi::c_int
            | 0x404 as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).crocom_var_A = 23 as uint16;
        (*E).base.palette_index = 3584 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn CrocomireTongue_Main() {}
