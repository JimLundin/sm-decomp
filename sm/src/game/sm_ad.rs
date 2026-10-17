extern "C" {
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    static mut g_ram: [uint8; 131072];
    static mut g_rom: *const uint8;
    fn RomPtr(addr: uint32_t) -> *const uint8;
    fn QueueSfx2_Max6(a: uint16);
    fn SpawnHardcodedPlm(p: SpawnHardcodedPlmArgs);
    static mut eproj_spawn_pt: Point16U;
    fn SpawnEprojWithRoomGfx(j: uint16, a: uint16);
    fn WriteColorsToPalette(k: uint16, db: uint8, j: uint16, a: uint16);
}
pub use crate::types::{
    __uint16_t, __uint32_t, __uint8_t, uint16, uint16_t, uint32_t, uint8, uint8_t, EnemyBase,
    EnemyData, EnemyProps, Func_V, Point16U, VoidP,
};
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
pub use crate::types::Consts_AD;
pub const addr_kTurnOffLightsForShitroidDeath: Consts_AD = 61999;
pub use crate::types::{Enemy_MotherBrain, Func_CalcHdma, Ram8000_Default, SpawnHardcodedPlmArgs};
#[inline]
unsafe extern "C" fn __CFADD__uint16(mut x: uint16, mut y: uint16) -> bool {
    return x as ::core::ffi::c_int
        > (x as ::core::ffi::c_int + y as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn RomPtr_AD(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0xad0000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
pub const fnMotherBrainBody_FakeDeath_Descent_0_Pause: ::core::ffi::c_int =
    0xa9881d as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn gEnemyData(mut o: ::core::ffi::c_int) -> *mut EnemyData {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut EnemyData;
}
#[inline]
unsafe extern "C" fn gRam8000_Default(mut o: ::core::ffi::c_int) -> *mut Ram8000_Default {
    return (&raw mut g_ram as *mut uint8)
        .offset(0x8000 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Ram8000_Default;
}
#[inline]
unsafe extern "C" fn Get_MotherBrain(mut o: ::core::ffi::c_int) -> *mut Enemy_MotherBrain {
    return (&raw mut g_ram as *mut uint8)
        .offset(0xf78 as ::core::ffi::c_int as isize)
        .offset(o as isize) as *mut Enemy_MotherBrain;
}
#[no_mangle]
pub unsafe extern "C" fn nullsub_341() {}
static mut funcs_BE56D: [Option<Func_V>; 16] = unsafe {
    [
        Some(MotherBrain_CalcHdma_Down as unsafe extern "C" fn() -> ()),
        Some(MotherBrain_CalcHdma_BeamAimedRight2 as unsafe extern "C" fn() -> ()),
        None,
        None,
        None,
        Some(MotherBrain_CalcHdma_BeamAimedUp as unsafe extern "C" fn() -> ()),
        Some(MotherBrain_CalcHdma_BeamAimedUp as unsafe extern "C" fn() -> ()),
        None,
        None,
        None,
        Some(MotherBrain_CalcHdma_BeamAimedUp as unsafe extern "C" fn() -> ()),
        Some(nullsub_341 as unsafe extern "C" fn() -> ()),
        Some(MotherBrain_CalcHdma_Down as unsafe extern "C" fn() -> ()),
        None,
        None,
        Some(MotherBrain_CalcHdma_Down as unsafe extern "C" fn() -> ()),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_CalcHdma() {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int);
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int);
    let mut r18: uint16 = (*(&raw mut (*E).mbn_var_33 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_3A = ((*E).mbn_var_31 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_3B = (r18 as ::core::ffi::c_int + (*E).mbn_var_31 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = (*((&raw mut (*E1).base.enemy_ptr as *mut uint8)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 3584 as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_3C = v2;
    (*E).mbn_var_3E = v2;
    let mut v3: uint16 =
        ((*E1).base.y_pos as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_3D = v3;
    (*E).mbn_var_3F = v3;
    r18 = (4 as ::core::ffi::c_int
        * ((*E).mbn_var_3A as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int)) as uint16;
    let mut i: ::core::ffi::c_int = (r18 as ::core::ffi::c_int
        | (*E).mbn_var_3B as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int)
        >> 6 as ::core::ffi::c_int;
    funcs_BE56D[i as usize].expect("non-null function pointer")();
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_CalcHdma_BeamAimedRight2() {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int);
    MotherBrain_CalcHdma_BeamAimedRight((*E).mbn_var_3C, (*E).mbn_var_3E);
    *((&raw mut g_ram as *mut uint8).offset(0x9c00 as ::core::ffi::c_int as isize)
        as *mut uint16) = 16 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
        -(25600 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 16 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
        -(25597 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(6 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 240 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(7 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
        -(25340 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(9 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 244 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(10 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
        -(25108 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(12 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_CalcHdma_BeamAimedRight(mut r22: uint16, mut r24: uint16) {
    *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 255 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) = 255 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int);
    let mut r18: uint16 = *(g_rom.offset(
        (((0x91c9d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91c9d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *(&raw mut (*E).mbn_var_3A as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as isize,
        );
    let mut r20: uint16 = *(g_rom.offset(
        (((0x91c9d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91c9d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *(&raw mut (*E).mbn_var_3B as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as isize,
        );
    let mut mbn_var_3F: uint16 = (*E).mbn_var_3F;
    let mut v2: *mut uint16 = ((&raw mut g_ram as *mut uint8)
        .offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((mbn_var_3F as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as isize)
        .offset(1 as ::core::ffi::c_int as isize);
    *v2.offset(1 as ::core::ffi::c_int as isize) = 255 as uint16;
    *v2.offset(2 as ::core::ffi::c_int as isize) = 255 as uint16;
    let mut v8: *mut uint16 = v2;
    loop {
        let mut v4: uint16 = (r20 as ::core::ffi::c_int + r24 as ::core::ffi::c_int) as uint16;
        if __CFADD__uint16(r20, r24) {
            break;
        }
        r24 = (r24 as ::core::ffi::c_int + r20 as ::core::ffi::c_int) as uint16;
        let fresh11 = v2;
        v2 = v2.offset(-1);
        *fresh11 = (*(&raw mut v4 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            | 0xff00 as ::core::ffi::c_int) as uint16;
        mbn_var_3F = mbn_var_3F.wrapping_sub(1);
        if !(mbn_var_3F as ::core::ffi::c_int != 32 as ::core::ffi::c_int) {
            break;
        }
    }
    loop {
        let fresh12 = v2;
        v2 = v2.offset(-1);
        *fresh12 = 255 as uint16;
        mbn_var_3F = mbn_var_3F.wrapping_sub(1);
        if !(mbn_var_3F as ::core::ffi::c_int != 32 as ::core::ffi::c_int) {
            break;
        }
    }
    let mut v5: *mut uint16 = v8.offset(1 as ::core::ffi::c_int as isize);
    let mut var_3F: uint16 = (*gRam8000_Default(0 as ::core::ffi::c_int)).var_3F;
    loop {
        let mut v7: uint16 = (r18 as ::core::ffi::c_int + r22 as ::core::ffi::c_int) as uint16;
        if __CFADD__uint16(r18, r22) {
            break;
        }
        r22 = (r22 as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
        let fresh13 = v5;
        v5 = v5.offset(1);
        *fresh13 = (*(&raw mut v7 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            | 0xff00 as ::core::ffi::c_int) as uint16;
        var_3F = var_3F.wrapping_add(1);
        if var_3F as ::core::ffi::c_int == 232 as ::core::ffi::c_int {
            return;
        }
    }
    loop {
        let fresh14 = v5;
        v5 = v5.offset(1);
        *fresh14 = 255 as uint16;
        var_3F = var_3F.wrapping_add(1);
        if !(var_3F as ::core::ffi::c_int != 232 as ::core::ffi::c_int) {
            break;
        }
    }
}
static mut off_ADE024: [Option<Func_CalcHdma>; 4] = unsafe {
    [
        Some(MotherBrain_CalcHdma_Up_UpRight as unsafe extern "C" fn(uint16, uint16) -> ()),
        Some(MotherBrain_CalcHdma_Up_Up as unsafe extern "C" fn(uint16, uint16) -> ()),
        None,
        Some(MotherBrain_CalcHdma_Up_UpLeft as unsafe extern "C" fn(uint16, uint16) -> ()),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_CalcHdma_BeamAimedUp() {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int);
    off_ADE024[((*E).mbn_var_3A as ::core::ffi::c_int >> 6 as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        | (*E).mbn_var_3B as ::core::ffi::c_int >> 7 as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")((*E).mbn_var_3C, (*E).mbn_var_3E);
    *((&raw mut g_ram as *mut uint8).offset(0x9c00 as ::core::ffi::c_int as isize)
        as *mut uint16) = 16 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
        -(25344 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 16 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
        -(25342 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 =
        ((*E).mbn_var_3D as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
    printf(b"Carry crap!\n\0" as *const u8 as *const ::core::ffi::c_char);
    if v2 as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset(6 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
            (v2 as ::core::ffi::c_int - 127 as ::core::ffi::c_int | 0x80 as ::core::ffi::c_int)
                as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset(7 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
            -(25340 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset(9 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 255 as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset(10 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
            (2 as ::core::ffi::c_int * (v2 as ::core::ffi::c_int - 127 as ::core::ffi::c_int)
                - 25340 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset(12 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
            127 as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset(13 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
            (2 as ::core::ffi::c_int * (v2 as ::core::ffi::c_int - 127 as ::core::ffi::c_int)
                - 25340 as ::core::ffi::c_int
                + 254 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset(15 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset(6 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
            (v2 as ::core::ffi::c_int | 0x80 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset(7 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
            -(25340 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset(9 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 127 as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset(10 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
            (2 as ::core::ffi::c_int * v2 as ::core::ffi::c_int - 25340 as ::core::ffi::c_int)
                as uint16;
        *((&raw mut g_ram as *mut uint8)
            .offset(0x9c00 as ::core::ffi::c_int as isize)
            .offset(12 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_CalcHdma_Up_UpRight(mut r22: uint16, mut r24: uint16) {
    *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 255 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) = 255 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int);
    let mut r18: uint16 = *(g_rom.offset(
        (((0x91c9d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91c9d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *(&raw mut (*E).mbn_var_3B as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as isize,
        );
    let mut r20: uint16 = *(g_rom.offset(
        (((0x91c9d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91c9d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *(&raw mut (*E).mbn_var_3A as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as isize,
        );
    let mut mbn_var_3D: uint16 = (*E).mbn_var_3D;
    let mut v2: *mut uint16 = ((&raw mut g_ram as *mut uint8)
        .offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((mbn_var_3D as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as isize)
        .offset(1 as ::core::ffi::c_int as isize);
    *v2.offset(1 as ::core::ffi::c_int as isize) = 255 as uint16;
    *v2.offset(2 as ::core::ffi::c_int as isize) = 255 as uint16;
    loop {
        let mut v4: uint16 = (r18 as ::core::ffi::c_int + r22 as ::core::ffi::c_int) as uint16;
        if __CFADD__uint16(r18, r22) {
            v4 = -(1 as ::core::ffi::c_int) as uint16;
        }
        r22 = v4;
        let mut R36: uint16 = (v4 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int
            & 0xff as ::core::ffi::c_int) as uint16;
        let mut v7: uint16 = (r20 as ::core::ffi::c_int + r24 as ::core::ffi::c_int) as uint16;
        if __CFADD__uint16(r20, r24) {
            v7 = -(1 as ::core::ffi::c_int) as uint16;
        }
        r24 = v7;
        let mut v8: uint16 = (R36 as ::core::ffi::c_int
            | v7 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            as uint16;
        if v8 as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
            v8 = 255 as uint16;
        }
        let fresh10 = v2;
        v2 = v2.offset(-1);
        *fresh10 = v8;
        mbn_var_3D = mbn_var_3D.wrapping_sub(1);
        if !(mbn_var_3D as ::core::ffi::c_int != 32 as ::core::ffi::c_int) {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_CalcHdma_Up_Up(mut r22: uint16, mut r24: uint16) {
    *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 255 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) = 255 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int);
    let mut r18: uint16 = *(g_rom.offset(
        (((0x91c9d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91c9d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            -(*(&raw mut (*E).mbn_var_3B as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int) as uint8 as isize,
        );
    let mut r20: uint16 = *(g_rom.offset(
        (((0x91c9d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91c9d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *(&raw mut (*E).mbn_var_3A as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as isize,
        );
    let mut mbn_var_3D: uint16 = (*E).mbn_var_3D;
    let mut v2: *mut uint16 = ((&raw mut g_ram as *mut uint8)
        .offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((mbn_var_3D as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as isize)
        .offset(1 as ::core::ffi::c_int as isize);
    *v2.offset(1 as ::core::ffi::c_int as isize) = 255 as uint16;
    *v2.offset(2 as ::core::ffi::c_int as isize) = 255 as uint16;
    loop {
        let mut v4: uint16 = (r22 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
        if (r22 as ::core::ffi::c_int) < r18 as ::core::ffi::c_int {
            v4 = 0 as uint16;
        }
        r22 = v4;
        let mut r26: uint16 =
            (v4 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8 as uint16;
        let mut v7: uint16 = (r20 as ::core::ffi::c_int + r24 as ::core::ffi::c_int) as uint16;
        if __CFADD__uint16(r20, r24) {
            v7 = -(1 as ::core::ffi::c_int) as uint16;
        }
        r24 = v7;
        let mut v8: uint16 = (r26 as ::core::ffi::c_int
            | v7 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            as uint16;
        if v8 as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
            v8 = 255 as uint16;
        }
        let fresh9 = v2;
        v2 = v2.offset(-1);
        *fresh9 = v8;
        mbn_var_3D = mbn_var_3D.wrapping_sub(1);
        if !(mbn_var_3D as ::core::ffi::c_int != 32 as ::core::ffi::c_int) {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_CalcHdma_Up_UpLeft(mut r22: uint16, mut r24: uint16) {
    *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 255 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) = 255 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int);
    let mut r18: uint16 = *(g_rom.offset(
        (((0x91c9d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91c9d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            -(*(&raw mut (*E).mbn_var_3B as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int) as uint8 as isize,
        );
    let mut r20: uint16 = *(g_rom.offset(
        (((0x91c9d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91c9d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            -(*(&raw mut (*E).mbn_var_3A as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int) as uint8 as isize,
        );
    let mut mbn_var_3D: uint16 = (*E).mbn_var_3D;
    let mut v2: *mut uint16 = ((&raw mut g_ram as *mut uint8)
        .offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((mbn_var_3D as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as isize)
        .offset(1 as ::core::ffi::c_int as isize);
    *v2.offset(1 as ::core::ffi::c_int as isize) = 255 as uint16;
    *v2.offset(2 as ::core::ffi::c_int as isize) = 255 as uint16;
    loop {
        let mut v4: uint16 = (r22 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
        if (r22 as ::core::ffi::c_int) < r18 as ::core::ffi::c_int {
            v4 = 0 as uint16;
        }
        r22 = v4;
        let mut R36: uint16 = (v4 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint16;
        let mut v7: uint16 = (r24 as ::core::ffi::c_int - r20 as ::core::ffi::c_int) as uint16;
        if (r24 as ::core::ffi::c_int) < r20 as ::core::ffi::c_int {
            v7 = 0 as uint16;
        }
        r24 = v7;
        let mut v8: uint16 = (R36 as ::core::ffi::c_int
            | v7 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            as uint16;
        if v8 as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
            v8 = 255 as uint16;
        }
        let fresh8 = v2;
        v2 = v2.offset(-1);
        *fresh8 = v8;
        mbn_var_3D = mbn_var_3D.wrapping_sub(1);
        if !(mbn_var_3D as ::core::ffi::c_int != 32 as ::core::ffi::c_int) {
            break;
        }
    }
}
static mut g_off_ADE20E: [Option<Func_CalcHdma>; 4] = unsafe {
    [
        Some(MotherBrain_CalcHdma_Down_DownRight as unsafe extern "C" fn(uint16, uint16) -> ()),
        None,
        Some(MotherBrain_CalcHdma_Down_Down as unsafe extern "C" fn(uint16, uint16) -> ()),
        Some(MotherBrain_CalcHdma_Down_DownLeft as unsafe extern "C" fn(uint16, uint16) -> ()),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_CalcHdma_Down() {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int);
    g_off_ADE20E[((*E).mbn_var_3A as ::core::ffi::c_int >> 6 as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        | (*E).mbn_var_3B as ::core::ffi::c_int >> 7 as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")((*E).mbn_var_3C, (*E).mbn_var_3E);
    *((&raw mut g_ram as *mut uint8).offset(0x9c00 as ::core::ffi::c_int as isize)
        as *mut uint16) = 16 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
        -(25600 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(3 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 16 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(4 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
        -(25597 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(6 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 240 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(7 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
        -(25340 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(9 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 244 as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(10 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) =
        -(25114 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8)
        .offset(0x9c00 as ::core::ffi::c_int as isize)
        .offset(12 as ::core::ffi::c_int as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_CalcHdma_Down_DownRight(mut r22: uint16, mut r24: uint16) {
    *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 255 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) = 255 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int);
    let mut r18: uint16 = *(g_rom.offset(
        (((0x91c9d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91c9d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *(&raw mut (*E).mbn_var_3A as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as isize,
        );
    let mut r20: uint16 = *(g_rom.offset(
        (((0x91c9d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91c9d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *(&raw mut (*E).mbn_var_3B as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as isize,
        );
    let mut v1: uint16 =
        ((*E).mbn_var_3D as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
    let mut dst: *mut uint16 = ((&raw mut g_ram as *mut uint8)
        .offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize);
    loop {
        let fresh6 = dst;
        dst = dst.offset(1);
        *fresh6 = 255 as uint16;
        v1 = v1.wrapping_sub(1);
        if !(v1 != 0) {
            break;
        }
    }
    let mut mbn_var_3D: uint16 = (*Get_MotherBrain(0 as ::core::ffi::c_int)).mbn_var_3D;
    loop {
        let mut v4: uint16 = (r18 as ::core::ffi::c_int + r22 as ::core::ffi::c_int) as uint16;
        if __CFADD__uint16(r18, r22) {
            v4 = -(1 as ::core::ffi::c_int) as uint16;
        }
        r22 = v4;
        let mut r26: uint16 = (v4 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint16;
        let mut v7: uint16 = (r20 as ::core::ffi::c_int + r24 as ::core::ffi::c_int) as uint16;
        if __CFADD__uint16(r20, r24) {
            v7 = -(1 as ::core::ffi::c_int) as uint16;
        }
        r24 = v7;
        let mut v8: uint16 = (r26 as ::core::ffi::c_int
            | v7 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            as uint16;
        if v8 as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
            v8 = 255 as uint16;
        }
        let fresh7 = dst;
        dst = dst.offset(1);
        *fresh7 = v8;
        mbn_var_3D = mbn_var_3D.wrapping_add(1);
        if !(mbn_var_3D as ::core::ffi::c_int != 232 as ::core::ffi::c_int) {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_CalcHdma_Down_Down(mut r22: uint16, mut r24: uint16) {
    *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 255 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) = 255 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int);
    let mut r18: uint16 = *(g_rom.offset(
        (((0x91c9d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91c9d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            -(*(&raw mut (*E).mbn_var_3A as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int) as uint8 as isize,
        );
    let mut r20: uint16 = *(g_rom.offset(
        (((0x91c9d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91c9d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *(&raw mut (*E).mbn_var_3B as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as isize,
        );
    let mut v1: uint16 =
        ((*E).mbn_var_3D as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
    let mut dst: *mut uint16 = ((&raw mut g_ram as *mut uint8)
        .offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize);
    loop {
        let fresh4 = dst;
        dst = dst.offset(1);
        *fresh4 = 255 as uint16;
        v1 = v1.wrapping_sub(1);
        if !(v1 != 0) {
            break;
        }
    }
    let mut mbn_var_3D: uint16 = (*Get_MotherBrain(0 as ::core::ffi::c_int)).mbn_var_3D;
    loop {
        let mut v4: uint16 = (r22 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
        if (r22 as ::core::ffi::c_int) < r18 as ::core::ffi::c_int {
            v4 = 0 as uint16;
        }
        r22 = v4;
        let mut r26: uint16 = (v4 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint16;
        let mut v7: uint16 = (r20 as ::core::ffi::c_int + r24 as ::core::ffi::c_int) as uint16;
        if __CFADD__uint16(r20, r24) {
            v7 = -(1 as ::core::ffi::c_int) as uint16;
        }
        r24 = v7;
        let mut v8: uint16 = (r26 as ::core::ffi::c_int
            | v7 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            as uint16;
        if v8 as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
            v8 = 255 as uint16;
        }
        let fresh5 = dst;
        dst = dst.offset(1);
        *fresh5 = v8;
        mbn_var_3D = mbn_var_3D.wrapping_add(1);
        if !(mbn_var_3D as ::core::ffi::c_int != 232 as ::core::ffi::c_int) {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_CalcHdma_Down_DownLeft(mut r22: uint16, mut r24: uint16) {
    *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 255 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) = 255 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int);
    let mut r18: uint16 = *(g_rom.offset(
        (((0x91c9d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91c9d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            -(*(&raw mut (*E).mbn_var_3A as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int) as uint8 as isize,
        );
    let mut r20: uint16 = *(g_rom.offset(
        (((0x91c9d4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91c9d4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            -(*(&raw mut (*E).mbn_var_3B as *mut uint8).offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int) as uint8 as isize,
        );
    let mut v1: uint16 =
        ((*E).mbn_var_3D as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
    let mut dst: *mut uint16 = ((&raw mut g_ram as *mut uint8)
        .offset(0x9d00 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize);
    loop {
        let fresh2 = dst;
        dst = dst.offset(1);
        *fresh2 = 255 as uint16;
        v1 = v1.wrapping_sub(1);
        if !(v1 != 0) {
            break;
        }
    }
    let mut mbn_var_3D: uint16 = (*Get_MotherBrain(0 as ::core::ffi::c_int)).mbn_var_3D;
    loop {
        let mut v4: uint16 = (r22 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
        if (r22 as ::core::ffi::c_int) < r18 as ::core::ffi::c_int {
            v4 = 0 as uint16;
        }
        r22 = v4;
        let mut v5: uint16 = (v4 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint16;
        let mut r26: uint16 = v5;
        let mut v7: uint16 = (r24 as ::core::ffi::c_int - r20 as ::core::ffi::c_int) as uint16;
        if (r24 as ::core::ffi::c_int) < r20 as ::core::ffi::c_int {
            v7 = 0 as uint16;
        }
        r24 = v7;
        let mut v8: uint16 = (r26 as ::core::ffi::c_int
            | v7 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            as uint16;
        if v8 == 0 {
            v8 = 255 as uint16;
        }
        let fresh3 = dst;
        dst = dst.offset(1);
        *fresh3 = v8;
        mbn_var_3D = mbn_var_3D.wrapping_add(1);
        if !(mbn_var_3D as ::core::ffi::c_int != 232 as ::core::ffi::c_int) {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_SealWall() {
    eproj_spawn_pt = Point16U {
        x: 248 as uint16,
        y: 72 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        9 as uint16,
    );
    eproj_spawn_pt = Point16U {
        x: 248 as uint16,
        y: 152 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as ::core::ffi::c_int as uint16,
        9 as uint16,
    );
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
        field_0: 0xf as uint8,
        field_1: 0x4 as uint8,
        field_2: 0xb673 as VoidP,
    });
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs {
        field_0: 0xf as uint8,
        field_1: 0x9 as uint8,
        field_2: 0xb673 as VoidP,
    });
    (*Get_MotherBrain(0 as ::core::ffi::c_int)).mbn_var_A =
        (0xa9881d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_HealthBasedPaletteHandling() {
    if (*Get_MotherBrain(0 as ::core::ffi::c_int)).mbn_var_1F as ::core::ffi::c_int
        >= 2 as ::core::ffi::c_int
    {
        let mut v0: uint16 = 0 as uint16;
        let mut health: uint16 = (*Get_MotherBrain(0x40 as ::core::ffi::c_int)).base.health;
        if (health as ::core::ffi::c_int) < 0x2328 as ::core::ffi::c_int {
            v0 = 2 as uint16;
            if (health as ::core::ffi::c_int) < 0x1518 as ::core::ffi::c_int {
                v0 = 4 as uint16;
                if (health as ::core::ffi::c_int) < 0x708 as ::core::ffi::c_int {
                    v0 = 6 as uint16;
                }
            }
        }
        WriteColorsToPalette(
            0x82 as uint16,
            0xad as uint8,
            *(g_rom.offset(
                (((0xade6a2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xade6a2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
            0xf as uint16,
        );
        WriteColorsToPalette(
            0x122 as uint16,
            0xad as uint8,
            *(g_rom.offset(
                (((0xade6a2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xade6a2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
            0xf as uint16,
        );
        WriteColorsToPalette(
            0x162 as uint16,
            0xad as uint8,
            *(g_rom.offset(
                (((0xade742 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xade742 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
            0xf as uint16,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_FadePalToBlack(mut a: uint16) -> uint8 {
    let mut v1: ::core::ffi::c_int = a as ::core::ffi::c_int;
    if *(g_rom.offset(
        (((0xade9e8 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xade9e8 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize)
        == 0
    {
        return 1 as uint8;
    }
    let mut j: uint16 = *(g_rom.offset(
        (((0xade9e8 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xade9e8 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize);
    WriteColorsToPalette(0x82 as uint16, 0xad as uint8, j, 14 as uint16);
    WriteColorsToPalette(0x122 as uint16, 0xad as uint8, j, 14 as uint16);
    WriteColorsToPalette(
        0x162 as uint16,
        0xad as uint8,
        (j as ::core::ffi::c_int + 28 as ::core::ffi::c_int) as uint16,
        14 as uint16,
    );
    return 0 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_FadeToGray_FakeDeath(mut a: uint16) -> uint8 {
    let mut v1: uint16 = *(g_rom.offset(
        (((0xaded8a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xaded8a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(a as isize);
    if v1 != 0 {
        return (MotherBrain_EEF6(v1) as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint8;
    } else {
        return 1 as uint8;
    };
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_FadeFromGray_FakeDeath(mut a: uint16) -> uint8 {
    let mut v1: uint16 = *(g_rom.offset(
        (((0xaded9c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xaded9c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(a as isize);
    if v1 != 0 {
        return (MotherBrain_EEF6(v1) as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint8;
    } else {
        return 1 as uint8;
    };
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_EEF6(mut a: uint16) -> uint8 {
    WriteColorsToPalette(0x122 as uint16, 0xad as uint8, a, 3 as uint16);
    return 0 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_FadeFromGray_Drained(mut a: uint16) -> uint8 {
    let mut v1: ::core::ffi::c_int = a as ::core::ffi::c_int;
    if *(g_rom.offset(
        (((0xaded9c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xaded9c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize)
        == 0
    {
        return 1 as uint8;
    }
    let mut j: uint16 = *(g_rom.offset(
        (((0xaded9c as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xaded9c as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize);
    WriteColorsToPalette(0x82 as uint16, 0xad as uint8, j, 13 as uint16);
    WriteColorsToPalette(0x122 as uint16, 0xad as uint8, j, 13 as uint16);
    WriteColorsToPalette(
        0x168 as uint16,
        0xad as uint8,
        (j as ::core::ffi::c_int + 26 as ::core::ffi::c_int) as uint16,
        5 as uint16,
    );
    *((&raw mut g_ram as *mut uint8).offset(0x17c as ::core::ffi::c_int as isize) as *mut uint8
        as *mut uint16) =
        *(RomPtr_AD((j as ::core::ffi::c_int + 36 as ::core::ffi::c_int) as uint16_t)
            as *mut uint16);
    return 0 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_FadeToGray_Drained(mut a: uint16) -> uint8 {
    let mut v1: ::core::ffi::c_int = a as ::core::ffi::c_int;
    if *(g_rom.offset(
        (((0xadef87 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xadef87 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize)
        == 0
    {
        return 1 as uint8;
    }
    let mut j: uint16 = *(g_rom.offset(
        (((0xadef87 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xadef87 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize);
    WriteColorsToPalette(0x82 as uint16, 0xad as uint8, j, 15 as uint16);
    WriteColorsToPalette(0x122 as uint16, 0xad as uint8, j, 15 as uint16);
    WriteColorsToPalette(
        0x168 as uint16,
        0xad as uint8,
        (j as ::core::ffi::c_int + 30 as ::core::ffi::c_int) as uint16,
        5 as uint16,
    );
    *((&raw mut g_ram as *mut uint8).offset(0x17c as ::core::ffi::c_int as isize) as *mut uint8
        as *mut uint16) =
        *(RomPtr_AD((j as ::core::ffi::c_int + 40 as ::core::ffi::c_int) as uint16_t)
            as *mut uint16);
    return 0 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_FadeToGray_RealDeath(mut a: uint16) -> uint8 {
    let mut v1: ::core::ffi::c_int = a as ::core::ffi::c_int;
    if *(g_rom.offset(
        (((0xadf107 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xadf107 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize)
        == 0
    {
        return 1 as uint8;
    }
    WriteColorsToPalette(
        0x1e2 as uint16,
        0xad as uint8,
        *(g_rom.offset(
            (((0xadf107 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xadf107 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v1 as isize),
        0xf as uint16,
    );
    return 0 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn TurnOffLightsForShitroidDeath() {
    WriteColorsToPalette(
        0x62 as uint16,
        0xad as uint8,
        addr_kTurnOffLightsForShitroidDeath as ::core::ffi::c_int as uint16,
        14 as uint16,
    );
    WriteColorsToPalette(
        0xa2 as uint16,
        0xad as uint8,
        addr_kTurnOffLightsForShitroidDeath as ::core::ffi::c_int as uint16,
        14 as uint16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn MotherBrain_Phase3_TurnLightsBackOn(mut a: uint16) -> uint8 {
    let mut v1: ::core::ffi::c_int = a as ::core::ffi::c_int;
    if *(g_rom.offset(
        (((0xadf273 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xadf273 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize)
        == 0
    {
        return 1 as uint8;
    }
    let mut v3: uint16 = *(g_rom.offset(
        (((0xadf273 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xadf273 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v1 as isize);
    WriteColorsToPalette(0x62 as uint16, 0xad as uint8, v3, 14 as uint16);
    WriteColorsToPalette(
        0xa2 as uint16,
        0xad as uint8,
        (v3 as ::core::ffi::c_int + 28 as ::core::ffi::c_int) as uint16,
        14 as uint16,
    );
    return 0 as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn EnableEarthquakeAframes(mut a: uint16) {
    *((&raw mut g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
        as *mut uint16) = a;
    *((&raw mut g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
        as *mut uint16) = 20 as uint16;
    QueueSfx2_Max6(0x25 as uint16);
}
#[no_mangle]
pub unsafe extern "C" fn HandleMotherBrainBodyFlickering() {
    let mut v0: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int);
    if (*v0).frame_counter as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let ref mut fresh0 = *((&raw mut g_ram as *mut uint8)
            .offset(0x69 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh0 = (*fresh0 as ::core::ffi::c_int | 2 as ::core::ffi::c_int) as uint16;
        (*v0).properties = ((*v0).properties as ::core::ffi::c_int
            & !(kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    } else {
        let ref mut fresh1 = *((&raw mut g_ram as *mut uint8)
            .offset(0x69 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh1 = (*fresh1 as ::core::ffi::c_int & !(2 as ::core::ffi::c_int)) as uint16;
        (*v0).properties = ((*v0).properties as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int) as uint16;
    };
}
