use crate::game::sm_80::{
    CheckBossBitForCurArea, CheckEventHappened, QueueMusic_Delayed8, QueueSfx1_Max6,
    QueueSfx2_Max6, QueueSfx3_Max6, SetBossBitForCurArea, SetEventHappened,
};
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_8d::SpawnPalfxObject;
use crate::game::sm_90::CallSomeSamusCode;
use crate::rtl::{g_ram, RomPtr, Unreachable};

pub use crate::types::{
    __int8_t, __uint16_t, __uint32_t, __uint8_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t,
    uint8, uint8_t, Consts_86,
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
#[inline]
unsafe fn RomPtr_87(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0x870000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
pub const fnAnimtilesInstr_Delete: uint32 = 8880306 as uint32;
pub const fnAnimtilesInstr_Goto: uint32 = 8880311 as uint32;
pub const fnAnimtilesInstr_GotoRel: uint32 = 8880316 as uint32;
pub const fnAnimtilesInstr_DecrementTimerAndGoto: uint32 = 8880340 as uint32;
pub const fnAnimtilesInstr_DecrementTimerAndGotoRel: uint32 = 8880348 as uint32;
pub const fnAnimtilesInstr_SetTimer: uint32 = 8880355 as uint32;
pub const fnAnimtilesInstr_QueueMusic: uint32 = 8880368 as uint32;
pub const fnAnimtilesInstr_QueueSfx1: uint32 = 8880380 as uint32;
pub const fnAnimtilesInstr_QueueSfx2: uint32 = 8880392 as uint32;
pub const fnAnimtilesInstr_QueueSfx3: uint32 = 8880404 as uint32;
pub const fnAnimtilesInstr_GotoIfBossBitSet: uint32 = 8880416 as uint32;
pub const fnAnimtilesInstr_SetBossBit: uint32 = 8880435 as uint32;
pub const fnAnimtilesInstr_GotoIfEventHappened: uint32 = 8880447 as uint32;
pub const fnAnimtilesInstr_SetEventHappened: uint32 = 8880464 as uint32;
pub const fnAnimtilesInstr_WaitUntilAreaBossDead_DoubleRet: uint32 = 8880570 as uint32;
pub const fnAnimtilesInstr_GotoIfBossBitSetInArea: uint32 = 8880899 as uint32;
pub const fnAnimtilesInstr_SpawnTourianStatueEyeGlow: uint32 = 8880928 as uint32;
pub const fnAnimtilesInstr_SpawnTourianStatueSoul: uint32 = 8880943 as uint32;
pub const fnAnimtilesInstr_GotoIfTourianStatueBusy: uint32 = 8880958 as uint32;
pub const fnAnimtilesInstr_TourianStatueSetState: uint32 = 8880969 as uint32;
pub const fnAnimtilesInstr_TourianStatueClearState: uint32 = 8880978 as uint32;
pub const fnAnimtilesInstr_Clear3PaletteColors: uint32 = 8880987 as uint32;
pub const fnAnimtilesInstr_SpawnPalfxObj: uint32 = 8881010 as uint32;
pub const fnAnimtilesInstr_Write8PaletteColors: uint32 = 8881023 as uint32;
pub unsafe fn EnableAnimtiles() {
    let ref mut fresh9 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1ef1 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh9 = (*fresh9 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn DisableAnimtiles() {
    let ref mut fresh8 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1ef1 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh8 = (*fresh8 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn ClearAnimtiles() {
    let mut i: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0x1ef5 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn SpawnAnimtiles(mut j: uint16) {
    let mut v0: uint16 = j;
    let mut v1: uint16 = 10 as uint16;
    while *((&raw mut g_ram as *mut uint8).offset(0x1ef5 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        != 0
    {
        v1 = (v1 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return;
        }
    }
    let mut v2: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0x1ef5 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = v0;
    *((&raw mut g_ram as *mut uint8).offset(0x1f01 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    let mut v3: *const uint16 = RomPtr_87(v0 as uint16_t) as *const uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1f0d as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *v3;
    *((&raw mut g_ram as *mut uint8).offset(0x1f25 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1f31 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *v3.offset(1 as ::core::ffi::c_int as isize);
    *((&raw mut g_ram as *mut uint8).offset(0x1f3d as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = *v3.offset(2 as ::core::ffi::c_int as isize);
    *((&raw mut g_ram as *mut uint8).offset(0x1f19 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 1 as uint16;
}
pub unsafe fn AnimtilesHandler() {
    if *((&raw mut g_ram as *mut uint8).offset(0x1ef1 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut i: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0x1ef3 as ::core::ffi::c_int as isize)
                as *mut uint16) = i as uint16;
            if *((&raw mut g_ram as *mut uint8).offset(0x1ef5 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize)
                != 0
            {
                ProcessAnimtilesObject();
                i = *((&raw mut g_ram as *mut uint8).offset(0x1ef3 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int;
            }
            i -= 2 as ::core::ffi::c_int;
        }
    }
}
pub unsafe fn CallAnimtilesInstr(mut ea: uint32, mut j: uint16, mut k: uint16) -> uint16 {
    match ea {
        8880306 => return AnimtilesInstr_Delete(j, k),
        8880311 => return AnimtilesInstr_Goto(j, k),
        8880316 => return AnimtilesInstr_GotoRel(j, k),
        8880340 => return AnimtilesInstr_DecrementTimerAndGoto(j, k),
        8880348 => return AnimtilesInstr_DecrementTimerAndGotoRel(j, k),
        8880355 => return AnimtilesInstr_SetTimer(j, k),
        8880368 => return AnimtilesInstr_QueueMusic(j, k),
        8880380 => return AnimtilesInstr_QueueSfx1(j, k),
        8880392 => return AnimtilesInstr_QueueSfx2(j, k),
        8880404 => return AnimtilesInstr_QueueSfx3(j, k),
        8880416 => return AnimtilesInstr_GotoIfBossBitSet(j, k),
        8880435 => return AnimtilesInstr_SetBossBit(j, k),
        8880447 => return AnimtilesInstr_GotoIfEventHappened(j, k),
        8880464 => return AnimtilesInstr_SetEventHappened(j, k),
        8880570 => return AnimtilesInstr_WaitUntilAreaBossDead_DoubleRet(j, k),
        8880899 => return AnimtilesInstr_GotoIfBossBitSetInArea(j, k),
        8880928 => return AnimtilesInstr_SpawnTourianStatueEyeGlow(j, k),
        8880943 => return AnimtilesInstr_SpawnTourianStatueSoul(j, k),
        8880958 => return AnimtilesInstr_GotoIfTourianStatueBusy(j, k),
        8880969 => return AnimtilesInstr_TourianStatueSetState(j, k),
        8880978 => return AnimtilesInstr_TourianStatueClearState(j, k),
        8880987 => return AnimtilesInstr_Clear3PaletteColors(j, k),
        8881010 => return AnimtilesInstr_SpawnPalfxObj(j, k),
        8881023 => return AnimtilesInstr_Write8PaletteColors(j, k),
        _ => return Unreachable() as uint16,
    };
}
pub unsafe fn ProcessAnimtilesObject() {
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1ef3 as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v1: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0x1ef3 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let ref mut fresh6 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1f19 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    let fresh7 = *fresh6;
    *fresh6 = (*fresh6).wrapping_sub(1);
    if fresh7 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        let mut v3: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0x1f0d as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        let mut v5: uint16 = 0;
        loop {
            let mut v4: *const uint16 = RomPtr_87(v3 as uint16_t) as *const uint16;
            v5 = *v4;
            if *v4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                break;
            }
            *((&raw mut g_ram as *mut uint8).offset(0x1f49 as ::core::ffi::c_int as isize)
                as *mut uint16) = *v4;
            v3 = CallAnimtilesInstr(
                (v5 as ::core::ffi::c_int | 0x870000 as ::core::ffi::c_int) as uint32,
                v0,
                (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
            );
            if v3 == 0 {
                return;
            }
        }
        let mut v6: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0x1f19 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v6 as isize) = v5;
        *((&raw mut g_ram as *mut uint8).offset(0x1f25 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v6 as isize) =
            *(RomPtr_87(v3 as uint16_t) as *mut uint16).offset(1 as ::core::ffi::c_int as isize);
        *((&raw mut g_ram as *mut uint8).offset(0x1f0d as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v6 as isize) = (v3 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn AnimtilesInstr_Delete(mut k: uint16, mut j: uint16) -> uint16 {
    *((&raw mut g_ram as *mut uint8).offset(0x1ef5 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return 0 as uint16;
}
pub unsafe fn AnimtilesInstr_Goto(mut k: uint16, mut j: uint16) -> uint16 {
    return *(RomPtr_87(j as uint16_t) as *mut uint16);
}
pub unsafe fn AnimtilesInstr_GotoRel(mut k: uint16, mut j: uint16) -> uint16 {
    *((&raw mut g_ram as *mut uint8).offset(0x1f49 as ::core::ffi::c_int as isize)
        as *mut uint16) = j;
    return (j as ::core::ffi::c_int + *RomPtr_87(j as uint16_t) as int8 as ::core::ffi::c_int)
        as uint16;
}
pub unsafe fn AnimtilesInstr_DecrementTimerAndGoto(mut k: uint16, mut j: uint16) -> uint16 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh0 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1f01 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize);
    let fresh1 = *fresh0;
    *fresh0 = (*fresh0).wrapping_sub(1);
    if fresh1 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    } else {
        return AnimtilesInstr_Goto(k, j);
    };
}
pub unsafe fn AnimtilesInstr_DecrementTimerAndGotoRel(mut k: uint16, mut j: uint16) -> uint16 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh2 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1f01 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize);
    let fresh3 = *fresh2;
    *fresh2 = (*fresh2).wrapping_sub(1);
    if fresh3 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        return (j as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    } else {
        return AnimtilesInstr_GotoRel(k, j);
    };
}
pub unsafe fn AnimtilesInstr_SetTimer(mut k: uint16, mut j: uint16) -> uint16 {
    *((&raw mut g_ram as *mut uint8).offset(0x1f01 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) = *RomPtr_87(j as uint16_t);
    return (j as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn AnimtilesInstr_QueueMusic(mut k: uint16, mut j: uint16) -> uint16 {
    let mut v2: *const uint8 = RomPtr_87(j as uint16_t);
    QueueMusic_Delayed8(*v2 as uint16);
    return (j as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn AnimtilesInstr_QueueSfx1(mut k: uint16, mut j: uint16) -> uint16 {
    let mut v2: *const uint8 = RomPtr_87(j as uint16_t);
    QueueSfx1_Max6(*v2 as uint16);
    return (j as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn AnimtilesInstr_QueueSfx2(mut k: uint16, mut j: uint16) -> uint16 {
    let mut v2: *const uint8 = RomPtr_87(j as uint16_t);
    QueueSfx2_Max6(*v2 as uint16);
    return (j as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn AnimtilesInstr_QueueSfx3(mut k: uint16, mut j: uint16) -> uint16 {
    let mut v2: *const uint8 = RomPtr_87(j as uint16_t);
    QueueSfx3_Max6(*v2 as uint16);
    return (j as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn AnimtilesInstr_GotoIfBossBitSet(mut k: uint16, mut j: uint16) -> uint16 {
    let mut v2: *const uint8 = RomPtr_87(j as uint16_t);
    let mut v3: uint16 = (j as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    if CheckBossBitForCurArea(*(v2 as *mut uint16) as uint8 as uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        return AnimtilesInstr_Goto(k, v3);
    } else {
        return (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn AnimtilesInstr_SetBossBit(mut k: uint16, mut j: uint16) -> uint16 {
    let mut v2: *const uint8 = RomPtr_87(j as uint16_t);
    SetBossBitForCurArea(*v2 as uint16);
    return (j as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn AnimtilesInstr_GotoIfEventHappened(mut k: uint16, mut j: uint16) -> uint16 {
    let mut v2: *const uint16 = RomPtr_87(j as uint16_t) as *const uint16;
    let mut v3: uint16 = (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    if CheckEventHappened(*v2) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        return AnimtilesInstr_Goto(k, v3);
    } else {
        return (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn AnimtilesInstr_SetEventHappened(mut k: uint16, mut j: uint16) -> uint16 {
    let mut v2: *const uint16 = RomPtr_87(j as uint16_t) as *const uint16;
    SetEventHappened(*v2);
    return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn UNUSED_sub_87815A() {
    CallSomeSamusCode(0 as uint16);
}
pub unsafe fn UNUSED_sub_878162() {
    CallSomeSamusCode(1 as uint16);
}
pub unsafe fn AnimtilesInstr_WaitUntilAreaBossDead_DoubleRet(
    mut k: uint16,
    mut j: uint16,
) -> uint16 {
    if CheckBossBitForCurArea(1 as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        *((&raw mut g_ram as *mut uint8).offset(0x1f19 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
        return 0 as uint16;
    }
    return j;
}
pub unsafe fn AnimtilesInstr_GotoIfBossBitSetInArea(mut k: uint16, mut j: uint16) -> uint16 {
    let mut v2: *const uint8 = RomPtr_87(j as uint16_t);
    let mut v3: uint16 = (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    if *v2 as ::core::ffi::c_int
        & *(&raw mut g_ram as *mut uint8)
            .offset(0xd828 as ::core::ffi::c_int as isize)
            .offset(*v2.offset(1 as ::core::ffi::c_int as isize) as isize)
            as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return AnimtilesInstr_Goto(k, v3);
    } else {
        return (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn AnimtilesInstr_SpawnTourianStatueEyeGlow(mut k: uint16, mut j: uint16) -> uint16 {
    let mut v2: *const uint16 = RomPtr_87(j as uint16_t) as *const uint16;
    SpawnEprojWithRoomGfx(
        addr_kEproj_TourianStatueEyeGlow as ::core::ffi::c_int as uint16,
        *v2,
    );
    return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn AnimtilesInstr_SpawnTourianStatueSoul(mut k: uint16, mut j: uint16) -> uint16 {
    let mut v2: *const uint16 = RomPtr_87(j as uint16_t) as *const uint16;
    SpawnEprojWithRoomGfx(
        addr_kEproj_TourianStatueSoul as ::core::ffi::c_int as uint16,
        *v2,
    );
    return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn AnimtilesInstr_GotoIfTourianStatueBusy(mut k: uint16, mut j: uint16) -> uint16 {
    if *((&raw mut g_ram as *mut uint8).offset(0x1e6f as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    } else {
        return AnimtilesInstr_Goto(k, j);
    };
}
pub unsafe fn AnimtilesInstr_TourianStatueSetState(mut k: uint16, mut j: uint16) -> uint16 {
    let ref mut fresh5 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1e6f as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh5 = (*fresh5 as ::core::ffi::c_int
        | *(RomPtr_87(j as uint16_t) as *mut uint16) as ::core::ffi::c_int) as uint16;
    return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn AnimtilesInstr_TourianStatueClearState(mut k: uint16, mut j: uint16) -> uint16 {
    let ref mut fresh4 = *((&raw mut g_ram as *mut uint8)
        .offset(0x1e6f as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh4 = (*fresh4 as ::core::ffi::c_int
        & !(*(RomPtr_87(j as uint16_t) as *mut uint16) as ::core::ffi::c_int))
        as uint16;
    return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn AnimtilesInstr_Clear3PaletteColors(mut k: uint16, mut j: uint16) -> uint16 {
    let mut v2: ::core::ffi::c_int =
        *(RomPtr_87(j as uint16_t) as *mut uint16) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v2 + 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v2 + 2 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn AnimtilesInstr_SpawnPalfxObj(mut k: uint16, mut j: uint16) -> uint16 {
    let mut v2: *const uint16 = RomPtr_87(j as uint16_t) as *const uint16;
    SpawnPalfxObject(*v2);
    return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn AnimtilesInstr_Write8PaletteColors(mut k: uint16, mut j: uint16) -> uint16 {
    static kAnimtilesInstr_Write8PaletteColors: [uint16; 8] = [
        0x3800 as ::core::ffi::c_int as uint16,
        0x7f58 as ::core::ffi::c_int as uint16,
        0x6ed5 as ::core::ffi::c_int as uint16,
        0x5a71 as ::core::ffi::c_int as uint16,
        0x49ee as ::core::ffi::c_int as uint16,
        0x356a as ::core::ffi::c_int as uint16,
        0x24e7 as ::core::ffi::c_int as uint16,
        0x1083 as ::core::ffi::c_int as uint16,
    ];
    let mut v2: uint16 = *(RomPtr_87(j as uint16_t) as *mut uint16);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 16 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            kAnimtilesInstr_Write8PaletteColors[(i >> 1 as ::core::ffi::c_int) as usize];
        v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        i += 2 as ::core::ffi::c_int;
    }
    return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
