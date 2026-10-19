pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a0::*;
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
use crate::ram_map::{ram_word, set_ram_word};
use crate::rtl::abs16;
use crate::rtl::AddToHiLo;
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
pub use crate::types::Consts_A0;
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
unsafe fn RomPtr_86(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0x860000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtr_A0(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0xa00000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtr_A1(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0xa10000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtr_B4(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0xb40000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomBankBase(bank: uint8, game: &mut Game) -> *const uint8 {
    RomPtr(
        (((bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as u32).wrapping_add(0x8000u32),
        game,
    )
    .offset(-(0x8000 as ::core::ffi::c_int as isize))
}
#[inline]
unsafe fn gVramWriteEntry(a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry
}
#[inline]
unsafe fn get_EnemyDef_A2(a: uint16, game: &mut Game) -> *mut EnemyDef {
    RomPtr((0xa00000u32) | ((a) as u32), game) as *mut EnemyDef
}
#[inline]
unsafe fn get_EprojDef(a: uint16, game: &mut Game) -> *mut EprojDef {
    RomPtr((0x860000u32) | ((a) as u32), game) as *mut EprojDef
}
#[inline]
unsafe fn gRam7800_Default(a: uint16, game: &mut Game) -> *mut Ram7800_Default {
    (&raw mut game.g_ram as *mut uint8)
        .offset((0x7800 as ::core::ffi::c_int + a as ::core::ffi::c_int) as isize) as *mut uint8
        as *mut Ram7800_Default
}
pub const fnEprojInstr_Delete: uint32 = 8814932;
pub const fnEprojInstr_Sleep: uint32 = 8814937;
pub const fnEprojInstr_SetPreInstr_: uint32 = 8814945;
pub const fnEprojInstr_ClearPreInstr: uint32 = 8814954;
pub const fnEprojPreInstr_nullsub_297: uint32 = 8814960;
pub const fnEprojInstr_CallFunc: uint32 = 8814961;
pub const fnEprojInstr_Goto: uint32 = 8815019;
pub const fnEprojInstr_GotoRel: uint32 = 8815024;
pub const fnEprojInstr_DecTimerAndGotoIfNonZero: uint32 = 8815046;
pub const fnEprojInstr_DecTimerAndGotoRelIfNonZero: uint32 = 8815054;
pub const fnEprojInstr_SetTimer: uint32 = 8815061;
pub const fnnullsub_82: uint32 = 8815070;
pub const fnEprojInstr_MoveRandomlyWithinRadius: uint32 = 8815071;
pub const fnEprojInstr_SetProjectileProperties: uint32 = 8815152;
pub const fnEprojInstr_ClearProjectileProperties: uint32 = 8815164;
pub const fnEprojInstr_EnableCollisionWithSamusProj: uint32 = 8815176;
pub const fnEprojInstr_DisableCollisionWithSamusProj: uint32 = 8815186;
pub const fnEprojInstr_DisableCollisionWithSamus: uint32 = 8815196;
pub const fnEprojInstr_EnableCollisionWithSamus: uint32 = 8815206;
pub const fnEprojInstr_SetToNotDieOnContact: uint32 = 8815216;
pub const fnEprojInstr_SetToDieOnContact: uint32 = 8815226;
pub const fnEprojInstr_SetLowPriority: uint32 = 8815236;
pub const fnEprojInstr_SetHighPriority: uint32 = 8815246;
pub const fnEprojInstr_SetXyRadius: uint32 = 8815256;
pub const fnEprojInstr_SetXyRadiusZero: uint32 = 8815265;
pub const fnEprojInstr_CalculateDirectionTowardsSamus: uint32 = 8815269;
pub const fnEprojInstr_WriteColorsToPalette: uint32 = 8815317;
pub const fnEprojInstr_QueueMusic: uint32 = 8815357;
pub const fnEprojInstr_QueueSfx1_Max6: uint32 = 8815369;
pub const fnEprojInstr_QueueSfx2_Max6: uint32 = 8815378;
pub const fnEprojInstr_QueueSfx3_Max6: uint32 = 8815387;
pub const fnEprojInstr_QueueSfx1_Max15: uint32 = 8815396;
pub const fnEprojInstr_QueueSfx2_Max15: uint32 = 8815405;
pub const fnEprojInstr_QueueSfx3_Max15: uint32 = 8815414;
pub const fnEprojInstr_QueueSfx1_Max3: uint32 = 8815423;
pub const fnEprojInstr_QueueSfx2_Max3: uint32 = 8815432;
pub const fnEprojInstr_QueueSfx3_Max3: uint32 = 8815441;
pub const fnEprojInstr_QueueSfx1_Max9: uint32 = 8815450;
pub const fnEprojInstr_QueueSfx2_Max9: uint32 = 8815459;
pub const fnEprojInstr_QueueSfx3_Max9: uint32 = 8815468;
pub const fnEprojInstr_QueueSfx1_Max1: uint32 = 8815477;
pub const fnEprojInstr_QueueSfx2_Max1: uint32 = 8815486;
pub const fnEprojInstr_QueueSfx3_Max1: uint32 = 8815495;
pub const fnEprojPreInstr_nullsub_83: uint32 = 8815867;
pub const fnEproj_Init_0x8aaf: uint32 = 8817209;
pub const fnEproj_PreInit_0x8aaf: uint32 = 8817277;
pub const fnEproj_Init_0x8bc2_SkreeDownRight: uint32 = 8817357;
pub const fnEproj_Init_0x8bd0_SkreeUpRight: uint32 = 8817393;
pub const fnEproj_Init_0x8bde_SkreeDownLeft: uint32 = 8817429;
pub const fnEproj_Init_0x8bec_SkreeUpLeft: uint32 = 8817465;
pub const fnEproj_PreInstr_SkreeParticle: uint32 = 8817501;
pub const fnEprojInstr_SpawnEnemyDropsWithDraygonsEyeDrops: uint32 = 8817768;
pub const fnEprojInstr_SetPreInstrA: uint32 = 8817910;
pub const fnEprojInstr_SetPreInstrB: uint32 = 8817917;
pub const fnEprojInit_DraygonsGunk: uint32 = 8817924;
pub const fnEprojInit_DraygonsWallTurretProjs: uint32 = 8817984;
pub const fnnullsub_84: ::core::ffi::c_int = 0x868d54;
pub const fnEprojInstr_868D99: uint32 = 8818073;
pub const fnEprojPreInstr_8DCA: ::core::ffi::c_int = 0x868dca;
pub const fnEprojPreInstr_DraygonsTurret_8DFF: ::core::ffi::c_int = 0x868dff;
pub const fnEprojPreInstr_DraygonsGunk_8E0F: uint32 = 8818191;
pub const fnEprojInit_CrocomireProjectile: uint32 = 8818723;
pub const fnEprojPreInstr_CrocomireProjectile: uint32 = 8818795;
pub const fnsub_8690B3: ::core::ffi::c_int = 0x8690b3;
pub const fnEprojInit_CrocomireSpikeWallPieces: uint32 = 8818895;
pub const fnEprojPreInstr_CrocomireSpikeWallPieces: uint32 = 8818965;
pub const fnEprojInstr_9270: uint32 = 8819312;
pub const fnEprojInit_CrocomireBridgeCrumbling: uint32 = 8819334;
pub const fnEprojPreInstr_CrocomireBridgeCrumbling: uint32 = 8819386;
pub const fnEprojInit_9634: uint32 = 8819533;
pub const fnEprojPreInstr_9634: uint32 = 8819602;
pub const fnEprojInit_9642_RidleysFireball: uint32 = 8819658;
pub const fnEprojPreInstr_9642_RidleysFireball: uint32 = 8819726;
pub const fnEprojInstr_DisableCollisionsWithSamus: uint32 = 8819829;
pub const fnEprojInit_9660_FireballExplosion: uint32 = 8819839;
pub const fnEprojInit_9688: uint32 = 8819872;
pub const fnEprojInit_9696: uint32 = 8819892;
pub const fnEprojInit_966C: uint32 = 8819912;
pub const fnEprojInit_967A: uint32 = 8819932;
pub const fnnullsub_85: uint32 = 8819980;
pub const fnEprojPreInstr_966C: uint32 = 8819981;
pub const fnEprojPreInstr_9688: uint32 = 8820002;
pub const fnEprojPreInstr_96A4: uint32 = 8820023;
pub const fnEprojPreInstr_96C0: uint32 = 8820032;
pub const fnEprojPreInstr_96CE: uint32 = 8820041;
pub const fnEprojInstr_95BA: uint32 = 8820154;
pub const fnEprojInstr_95ED: uint32 = 8820205;
pub const fnEprojInstr_9620: uint32 = 8820256;
pub const fnEprojInit_9734_CeresFallingDebris: uint32 = 8820444;
pub const fnEprojPreInstr_9734_CeresFallingDebris: uint32 = 8820481;
pub const fnEprojInstr_980E: uint32 = 8820750;
pub const fnEprojInit_PhantoonDestroyableFireballs: uint32 = 8820772;
pub const fnEprojInit_PhantoonStartingFireballs: uint32 = 8821050;
pub const fnEprojPreInstr_PhantoonStartingFireballs: uint32 = 8821121;
pub const fnEprojPreInstr_PhantoonStartingFireballs2: ::core::ffi::c_int = 0x8699bf;
pub const fnEprojPreInstr_PhantoonStartingFireballs3: ::core::ffi::c_int = 0x869a01;
pub const fnnullsub_86: ::core::ffi::c_int = 0x869a44;
pub const fnEprojPreInstr_PhantoonDestroyableFireballs: ::core::ffi::c_int = 0x869a45;
pub const fnEprojPreInstr_PhantoonDestroyableFireballs_2: ::core::ffi::c_int = 0x869a94;
pub const fnEprojPreInstr_PhantoonDestroyableFireballs_3: ::core::ffi::c_int = 0x869ada;
pub const fnEprojPreInstr_PhantoonStartingFireballsB: uint32 = 8821545;
pub const fnEprojPreInstr_PhantoonStartingFireballsB_2: ::core::ffi::c_int = 0x869b41;
pub const fnEprojInit_RocksKraidSpits: uint32 = 8821923;
pub const fnEprojInit_RocksFallingKraidCeiling: uint32 = 8821976;
pub const fnEprojInit_RocksWhenKraidRises: uint32 = 8822028;
pub const fnEprojPreInstr_KraidRocks: uint32 = 8822102;
pub const fnEprojPreInstr_RocksFallingKraidCeiling: uint32 = 8822153;
pub const fnsub_869DA5: uint32 = 8822181;
pub const fnEprojInit_MiniKraidSpit: uint32 = 8822252;
pub const fnEprojPreInit_MiniKraidSpit: uint32 = 8822302;
pub const fnEprojInit_MiniKraidSpikesLeft: uint32 = 8822342;
pub const fnEprojInit_MiniKraidSpikesRight: uint32 = 8822347;
pub const fnEprojPreInstr_MiniKraidSpikes: uint32 = 8822403;
pub const fnEprojInit_WalkingLavaSeahorseFireball: uint32 = 8822450;
pub const fnEprojPreInstr_WalkingLavaSeahorseFireball: uint32 = 8822527;
pub const fnEprojInit_PirateMotherBrainLaser: uint32 = 8822793;
pub const fnEprojInstr_SetPreInstrAndRun: uint32 = 8822864;
pub const fnnullsub_87: ::core::ffi::c_int = 0x86a05b;
pub const fnEprojPreInstr_PirateMotherBrainLaser_MoveLeft: uint32 = 8822876;
pub const fnEprojPreInstr_PirateMotherBrainLaser_MoveRight: uint32 = 8822906;
pub const fnEprojInit_PirateClaw: uint32 = 8822936;
pub const fnEprojPreInstr_PirateClawThrownLeft: uint32 = 8822993;
pub const fnEprojPreInstr_PirateClawThrownRight: uint32 = 8823076;
pub const fnEprojInit_A379: uint32 = 8823457;
pub const fnEprojInit_CeresElevatorPad: uint32 = 8823534;
pub const fnEprojInit_CeresElevatorPlatform: uint32 = 8823579;
pub const fnnullsub_88: uint32 = 8823591;
pub const fnEprojPreInstr_CeresElevatorPad: uint32 = 8823592;
pub const fnEprojPreInstr_CeresElevatorPlatform: uint32 = 8823652;
pub const fnEprojPreInstr_PrePhantomRoom: uint32 = 8823715;
pub const fnEprojInstr_A3BE: uint32 = 8823742;
pub const fnEprojInstr_GotoWithProbability25: uint32 = 8823894;
pub const fnEprojInit_BombTorizoLowHealthDrool: uint32 = 8824275;
pub const fnEprojInit_BombTorizoLowHealthInitialDrool: uint32 = 8824413;
pub const fnEprojInit_A977: uint32 = 8824519;
pub const fnEprojInit_BombTorizoExplosiveSwipe: uint32 = 8824566;
pub const fnEprojInit_BombTorizoStatueBreaking: uint32 = 8824676;
pub const fnEprojInit_BombTorizoLowHealthExplode: uint32 = 8824859;
pub const fnEprojInit_BombTorizoDeathExplosion: uint32 = 8824945;
pub const fnsub_86A887: uint32 = 8824967;
pub const fnEprojPreInstr_A977: uint32 = 8825071;
pub const fnnullsub_352: uint32 = 8825112;
pub const fnnullsub_89: uint32 = 8825113;
pub const fnEprojInit_AB07: uint32 = 8825405;
pub const fnEprojPreInstr_AB07: uint32 = 8825484;
pub const fnEprojInstr_SpawnEnemyDrops: uint32 = 8825738;
pub const fnEprojInit_BombTorizosChozoOrbs: uint32 = 8825835;
pub const fnEprojInit_GoldenTorizosChozoOrbs: uint32 = 8825980;
pub const fnEprojPreInstr_BombTorizosChozoOrbs: uint32 = 8826029;
pub const fnEprojPreInstr_GoldenTorizosChozoOrbs: uint32 = 8826106;
pub const fnEprojInstr_GotoDependingOnXDirection: uint32 = 8826258;
pub const fnEprojInit_TorizoSonicBoom: uint32 = 8826389;
pub const fnEprojPreInstr_TorizoSonicBoom: uint32 = 8826476;
pub const fnEprojInit_WreckedShipChozoSpikeFootsteps: uint32 = 8826620;
pub const fnEprojInstr_ResetXYpos1: uint32 = 8826678;
pub const fnEprojInit_TourianStatueDustClouds: uint32 = 8826691;
pub const fnEprojInit_TourianLandingDustCloudsRightFoot: uint32 = 8826704;
pub const fnEprojInstr_MoveY_Minus4: uint32 = 8826770;
pub const fnEprojInstr_SetVelTowardsSamus1: uint32 = 8827497;
pub const fnEprojInstr_SetVelTowardsSamus2: uint32 = 8827506;
pub const fnEprojInit_TorizoLandingDustCloudLeftFoot: uint32 = 8826829;
pub const fnEprojInit_GoldenTorizoEgg: uint32 = 8826881;
pub const fnEprojPreInstr_GoldenTorizoEgg: uint32 = 8826947;
pub const fnsub_86B0B9: uint32 = 8827065;
pub const fnsub_86B0DD: uint32 = 8827101;
pub const fnsub_86B13E: uint32 = 8827198;
pub const fnEprojInit_GoldenTorizoSuperMissile: uint32 = 8827342;
pub const fnEprojPreInstr_GoldenTorizoSuperMissile: uint32 = 8827405;
pub const fnEprojPreInstr_B237: uint32 = 8827447;
pub const fnEprojInit_GoldenTorizoEyeBeam: uint32 = 8827688;
pub const fnEprojPreInstr_GoldenTorizoEyeBeam: uint32 = 8827786;
pub const fnEprojInstr_GotoIfFunc1: uint32 = 8827832;
pub const fnEprojInstr_ResetXYpos2: uint32 = 8827958;
pub const fnEprojInit_TourianEscapeShaftFakeWallExplode: uint32 = 8828061;
pub const fnnullsub_90: uint32 = 8828080;
pub const fnEprojInit_LavaSeahorseFireball: uint32 = 8828143;
pub const fnsub_86B535: uint32 = 8828213;
pub const fnEprojInit_EyeDoorProjectile: uint32 = 8828461;
pub const fnEprojInit_EyeDoorSweat: uint32 = 8828547;
pub const fnEprojPreInstr_EyeDoorProjectile: uint32 = 8828601;
pub const fnEprojPreInstr_EyeDoorSweat: uint32 = 8828692;
pub const fnEprojInstr_SpawnTourianStatueUnlockingParticle: uint32 = 8828906;
pub const fnEprojInstr_Earthquake: uint32 = 8828917;
pub const fnEprojInstr_SpawnTourianStatueUnlockingParticleTail: uint32 = 8828952;
pub const fnEprojInstr_AddToYpos: uint32 = 8828993;
pub const fnEprojInit_TourianStatueUnlockingParticleWaterSplash: uint32 = 8829050;
pub const fnEprojInit_TourianStatueEyeGlow: uint32 = 8829070;
pub const fnEprojInit_TourianStatueUnlockingParticle: uint32 = 8829109;
pub const fnEprojIni_TourianStatueUnlockingParticleTail: uint32 = 8829160;
pub const fnEprojInit_TourianStatueSoul: uint32 = 8829176;
pub const fnEprojInit_TourianStatueBaseDecoration: uint32 = 8829246;
pub const fnEprojInit_TourianStatueRidley: uint32 = 8829265;
pub const fnEprojInit_TourianStatuePhantoon: uint32 = 8829284;
pub const fnEprojPreInstr_TourianStatueUnlockingParticleWaterSplash: uint32 = 8829303;
pub const fnEprojPreInstr_TourianStatueUnlockingParticle: uint32 = 8829314;
pub const fnEprojPreInstr_TourianStatueSoul: uint32 = 8829437;
pub const fnEprojPreInstr_TourianStatueStuff: uint32 = 8829495;
pub const fnEprojPreInstr_BA42: uint32 = 8829506;
pub const fnsub_86BB30: uint32 = 8829744;
pub const fnnullsub_91: uint32 = 8829775;
pub const fnEprojInit_NuclearWaffleBody: uint32 = 8829842;
pub const fnnullsub_92: uint32 = 8829894;
pub const fnEprojInit_NorfairLavaquakeRocks: uint32 = 8829915;
pub const fnEprojPreInstr_NorfairLavaquakeRocks: uint32 = 8829967;
pub const fnEproj_NorfairLavaquakeRocks_Func1: ::core::ffi::c_int = 0x86bc16;
pub const fnEproj_NorfairLavaquakeRocks_Func2: ::core::ffi::c_int = 0x86bc8f;
pub const fnEprojPreInstr_NorfairLavaquakeRocks_Inner: uint32 = 8830238;
pub const fnEprojPreInstr_NorfairLavaquakeRocks_Inner2: uint32 = 8830250;
pub const fnEprojInit_ShaktoolAttackMiddleBackCircle: uint32 = 8830364;
pub const fnEprojInit_BDA2: uint32 = 8830370;
pub const fnEprojInit_ShaktoolAttackFrontCircle: uint32 = 8830467;
pub const fnEprojPreInstr_BE12: uint32 = 8830482;
pub const fnEprojInit_MotherBrainRoomTurrets: uint32 = 8830543;
pub const fnEprojInit_MotherBrainRoomTurretBullets: uint32 = 8830809;
pub const fnEprojPreInstr_MotherBrainRoomTurrets: uint32 = 8830943;
pub const fnEprojPreInstr_MotherBrainRoomTurretBullets: uint32 = 8831200;
pub const fnEprojInstr_SwitchJump: uint32 = 8831347;
pub const fnEprojInstr_UserPalette0: uint32 = 8831412;
pub const fnEproj_MotherBrainsBlueRingLasers: uint32 = 8831731;
pub const fnEproj_MoveToBlueRingSpawnPosition: uint32 = 8831797;
pub const fnEprojInit_MotherBrainBomb: uint32 = 8832130;
pub const fnEprojPreInstr_MotherBrainBomb: uint32 = 8832200;
pub const fnsub_86C605: uint32 = 8832517;
pub const fnEprojInit_MotherBrainDeathBeemFired: uint32 = 8832644;
pub const fnnullsub_366: uint32 = 8832877;
pub const fnSpawnMotherBrainDeathBeam: uint32 = 8833019;
pub const fnEprojInit_MotherBrainRainbowBeam: uint32 = 8833034;
pub const fnEprojPreInstr_MotherBrainRainbowBeam: uint32 = 8833044;
pub const fnEprojInit_MotherBrainsDrool: uint32 = 8833091;
pub const fnEprojPreInstr_C84D: uint32 = 8833101;
pub const fnEprojInit_MotherBrainsDrool_Falling: uint32 = 8833158;
pub const fnEprojInstr_Add12ToY: uint32 = 8833232;
pub const fnEprojInit_MotherBrainsDeathExplosion: uint32 = 8833269;
pub const fnEprojPreInit_MotherBrainsDeathExplosion_0: uint32 = 8833300;
pub const fnEprojInit_MotherBrainsRainbowBeamExplosion: uint32 = 8833327;
pub const fnEprojPreInstr_MotherBrainsRainbowBeamExplosion: uint32 = 8833356;
pub const fnEprojInit_MotherBrainEscapeDoorParticles: uint32 = 8833377;
pub const fnEprojPreInstr_MotherBrainsExplodedDoorParticles: uint32 = 8833490;
pub const fnEprojInit_MotherBrainPurpleBreathBig: uint32 = 8833642;
pub const fnEprojInit_MotherBrainPurpleBreathSmall: uint32 = 8833667;
pub const fnnullsub_94: uint32 = 8833699;
pub const fnEprojInstr_MotherBrainPurpleBreathIsActive: uint32 = 8833774;
pub const fnEprojInit_TimeBombSetJapaneseText: uint32 = 8833782;
pub const fnEprojPreInstr_TimeBombSetJapaneseText: uint32 = 8833786;
pub const fnEprojInit_MotherBrainTubeFalling: uint32 = 8833993;
pub const fnEprojPreInstr_MotherBrainTubeFalling: uint32 = 8834023;
pub const fnMotherBrainTubeFallingFunc_GenerateExplosion: ::core::ffi::c_int = 0x86cbea;
pub const fnMotherBrainTubeFallingFunc_Falling: ::core::ffi::c_int = 0x86cc08;
pub const fnEprojInit_MotherBrainGlassShatteringShard: uint32 = 8834501;
pub const fnEprojInit_MotherBrainGlassShatteringSparkle: uint32 = 8834669;
pub const fnEprojPreInstr_MotherBrainGlassShatteringShard: uint32 = 8834715;
pub const fnEprojInit_KiHunterAcidSpitLeft: uint32 = 8834960;
pub const fnEprojInit_KiHunterAcidSpitRight: uint32 = 8834982;
pub const fnsub_86CFD5: uint32 = 8835029;
pub const fnsub_86CFE6: uint32 = 8835046;
pub const fnnullsub_95: uint32 = 8835063;
pub const fnsub_86CFF8: ::core::ffi::c_int = 0x86cff8;
pub const fnEprojInit_KagosBugs: uint32 = 8835208;
pub const fnEprojPreInstr_KagosBugs_Func1: uint32 = 8835251;
pub const fnEprojPreInstr_KagosBugs: ::core::ffi::c_int = 0x86d0ca;
pub const fnnullsub_302: ::core::ffi::c_int = 0x86d0eb;
pub const fnEprojPreInstr_D0EC: ::core::ffi::c_int = 0x86d0ec;
pub const fnEprojPreInstr_D128: ::core::ffi::c_int = 0x86d128;
pub const fnEprojInstr_D15C: uint32 = 8835420;
pub const fnEprojInstr_D1B6: uint32 = 8835510;
pub const fnEprojInstr_D1C7: uint32 = 8835527;
pub const fnEprojInstr_D1CE: uint32 = 8835534;
pub const fnEprojPreInstr_KagosBugs_Func2: uint32 = 8835556;
pub const fnEprojInit_MaridiaFloatersSpikes: uint32 = 8835642;
pub const fnEprojPreInstr_MaridiaFloatersSpikes: uint32 = 8835683;
pub const fnEprojInit_WreckedShipRobotLaserDown: uint32 = 8835852;
pub const fnEprojInit_WreckedShipRobotLaserHorizontal: uint32 = 8835886;
pub const fnEprojInit_WreckedShipRobotLaserUp: uint32 = 8835905;
pub const fnEprojPreInstr_WreckedShipRobotLaser: uint32 = 8836031;
pub const fnEprojInstr_AssignNewN00bTubeShardVelocity: uint32 = 8836577;
pub const fnEprojInstr_SetN00bTubeShardX: uint32 = 8836594;
pub const fnEprojInstr_D62A: uint32 = 8836650;
pub const fnEprojInstr_SetXvelRandom: uint32 = 8836762;
pub const fnEprojInit_N00bTubeCrack: uint32 = 8836773;
pub const fnEprojInit_N00bTubeShards: uint32 = 8836809;
pub const fnEprojInit_N00bTubeReleasedAirBubbles: uint32 = 8836980;
pub const fnsub_86D7BF: uint32 = 8837055;
pub const fnsub_86D7DE: uint32 = 8837086;
pub const fnEprojPreInstr_N00bTubeShards: uint32 = 8837117;
pub const fnsub_86D83D: uint32 = 8837181;
pub const fnsub_86D89F: uint32 = 8837279;
pub const fnsub_86D8DF: uint32 = 8837343;
pub const fnsub_86D992: uint32 = 8837522;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes: uint32 = 8837595;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_MoveY1: uint32 = 8837606;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_MoveY2: uint32 = 8837648;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_MoveX1: uint32 = 8837690;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_MoveX2: uint32 = 8837732;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1: uint32 = 8837774;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2: uint32 = 8837779;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1: uint32 = 8837784;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2: uint32 = 8837789;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1: uint32 = 8837794;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2: uint32 = 8837802;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1: uint32 = 8837810;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2: uint32 = 8837818;
pub const fnEprojInit_DBF2: uint32 = 8837912;
pub const fnEprojPreInstr_DBF2: uint32 = 8837979;
pub const fnEprojPreInstr_DBF2_MoveX1: ::core::ffi::c_int = 0x86db62;
pub const fnEprojPreInstr_DBF2_MoveX2: ::core::ffi::c_int = 0x86db8c;
pub const fnEprojPreInstr_DBF2_Func1: uint32 = 8838070;
pub const fnEprojInstr_DC5A: uint32 = 8838234;
pub const fnEprojInstr_SpawnEnemyDrops_0: uint32 = 8838241;
pub const fnEprojInstr_SpawnSporesEproj: uint32 = 8838263;
pub const fnEprojInit_Spores: uint32 = 8838285;
pub const fnEprojInit_SporeSpawnStalk: uint32 = 8838307;
pub const fnEprojInit_SporeSpawners: uint32 = 8838356;
pub const fnEprojPreInstr_Spores: uint32 = 8838382;
pub const fnnullsub_96: uint32 = 8838468;
pub const fnEprojPreInstr_SporeSpawners: uint32 = 8838470;
pub const fnEprojInit_NamiFuneFireball: uint32 = 8838870;
pub const fnEprojPreInstr_NamiFuneFireball: uint32 = 8838969;
pub const fnEprojInit_NamiFuneFireball_MoveX1: ::core::ffi::c_int = 0x86df40;
pub const fnEprojInit_NamiFuneFireball_MoveX2: ::core::ffi::c_int = 0x86df6a;
pub const fnEprojInstr_DFEA: uint32 = 8839146;
pub const fnEprojInit_LavaThrownByLavaman: uint32 = 8839168;
pub const fnsub_86E049: uint32 = 8839241;
pub const fnEproj_LavaThrownByLavaman_MoveX1: ::core::ffi::c_int = 0x86e050;
pub const fnEproj_LavaThrownByLavaman_MoveX2: ::core::ffi::c_int = 0x86e07a;
pub const fnEprojInit_DustCloudOrExplosion: uint32 = 8840296;
pub const fnEprojInit_EyeDoorSmoke: uint32 = 8840358;
pub const fnEprojPreInstr_DustCloudOrExplosion: uint32 = 8840446;
pub const fnEprojPreInstr_nullsub_98: uint32 = 8840456;
pub const fnEprojInstr_SetYVel: uint32 = 8840499;
pub const fnEprojInit_SpawnedShotGate: uint32 = 8840656;
pub const fnEprojInit_ClosedDownwardsShotGate: uint32 = 8840661;
pub const fnEprojInit_ClosedUpwardsShotGate: uint32 = 8840666;
pub const fnEprojPreInstr_nullsub_99: uint32 = 8840708;
pub const fnEprojPreInstr_E605: uint32 = 8840709;
pub const fnEprojInit_SaveStationElectricity: uint32 = 8840877;
pub const fnnullsub_100: uint32 = 8840913;
pub const fnEprojInit_BotwoonsBody: uint32 = 8841777;
pub const fnEprojPreInstr_BotwoonsBody: uint32 = 8841856;
pub const fnEproj_BotwoonsBody_Main: ::core::ffi::c_int = 0x86ea98;
pub const fnEproj_BotwonsBodyFunction_Dying: ::core::ffi::c_int = 0x86eaf4;
pub const fnEproj_BotwonsBodyFunction_Dying2: uint32 = 8841988;
pub const fnEproj_BotwonsBodyFunction_DyingFalling: ::core::ffi::c_int = 0x86eb1f;
pub const fnnullsub_101: ::core::ffi::c_int = 0x86eb93;
pub const fnEprojInit_BotwoonsSpit: uint32 = 8842182;
pub const fnEprojPreInstr_BotwoonsSpit: uint32 = 8842245;
pub const fnEprojInit_YappingMawsBody: uint32 = 8842338;
pub const fnEprojPreInstr_Empty2: uint32 = 8842388;
pub const fnEprojInstr_ECE3: uint32 = 8842467;
pub const fnEprojInstr_ED17: uint32 = 8842519;
pub const fnEprojInstr_QueueSfx2_9: uint32 = 8842891;
pub const fnEprojInstr_QueueSfx2_24: uint32 = 8842903;
pub const fnEprojInstr_QueueSfx2_B: uint32 = 8842915;
pub const fnEprojInstr_EEAF: uint32 = 8842927;
pub const fnEprojInstr_HandleRespawningEnemy: uint32 = 8843024;
pub const fnEprojInit_F337: uint32 = 8843049;
pub const fnEprojInit_EnemyDeathExplosion: uint32 = 8843145;
pub const fnEprojPreInstr_Empty: ::core::ffi::c_int = 0x86efdf;
pub const fnEprojPreInstr_Pickup: ::core::ffi::c_int = 0x86efe0;
pub const fnEprojInit_Sparks: uint32 = 8844177;
pub const fnEprojPreInstr_Sparks: uint32 = 8844272;
#[inline]
unsafe fn gEnemyData(o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemyData {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut EnemyData
}
#[inline]
unsafe fn gEnemySpawnData(o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemySpawnData {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0x7000)
        .offset(o as isize) as *mut EnemySpawnData
}
#[inline]
unsafe fn gExtraEnemyRam7800(o: ::core::ffi::c_int, game: &mut Game) -> *mut ExtraEnemyRam7800 {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0x7800)
        .offset(o as isize) as *mut ExtraEnemyRam7800
}
#[inline]
unsafe fn gExtraEnemyRam8000(o: ::core::ffi::c_int, game: &mut Game) -> *mut ExtraEnemyRam8000 {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0x8000)
        .offset(o as isize) as *mut ExtraEnemyRam8000
}
#[inline]
unsafe fn Get_MotherBrain(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_MotherBrain {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MotherBrain
}

// Shared projectile state stays backed by the original little-endian RAM bytes.
struct EprojState<'a>(&'a [u8]);
impl EprojState<'_> {
    fn enabled(&self) -> bool {
        u16::from_le_bytes([self.0[0x198d], self.0[0x198e]]) & 0x8000 != 0
    }
    fn index(&self) -> u16 {
        u16::from_le_bytes([self.0[0x1991], self.0[0x1992]])
    }
    fn init_param_1(&self) -> u16 {
        u16::from_le_bytes([self.0[0x1993], self.0[0x1994]])
    }
    fn slot_word(&self, base: usize, word_index: usize) -> u16 {
        let address = base + word_index * 2;
        u16::from_le_bytes([self.0[address], self.0[address + 1]])
    }
    fn id(&self, word_index: usize) -> u16 {
        self.slot_word(0x1997, word_index)
    }
    fn preinstruction(&self, word_index: usize) -> u16 {
        self.slot_word(0x1a03, word_index)
    }
    fn instruction(&self, word_index: usize) -> u16 {
        self.slot_word(0x1b47, word_index)
    }
    fn properties(&self, word_index: usize) -> u16 {
        self.slot_word(0x1bd7, word_index)
    }
    fn spritemap(&self, word_index: usize) -> u16 {
        self.slot_word(0x1b6b, word_index)
    }
    fn gfx_low(&self, word_index: usize) -> u8 {
        self.0[0x19bb + word_index * 2]
    }
    fn x_radius(&self, word_index: usize) -> u8 {
        self.0[0x1bb3 + word_index * 2]
    }
    fn y_radius(&self, word_index: usize) -> u8 {
        self.0[0x1bb4 + word_index * 2]
    }
}

struct EprojStateMut<'a>(&'a mut [u8]);
impl EprojStateMut<'_> {
    fn decrement_byte(&mut self, base: usize, byte_index: usize) -> u8 {
        let byte = &mut self.0[base + byte_index];
        *byte = byte.wrapping_sub(1);
        *byte
    }
    fn write(&mut self, base: usize, word_index: usize, value: u16) {
        let address = base + word_index * 2;
        self.0[address..address + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn add_split_words(
        &mut self,
        high_base: usize,
        low_base: usize,
        word_index: usize,
        amount: u32,
    ) {
        let low = u32::from(EprojState(self.0).slot_word(low_base, word_index));
        let high = u32::from(EprojState(self.0).slot_word(high_base, word_index));
        let sum = (low | (high << 16)).wrapping_add(amount);
        self.write(low_base, word_index, sum as u16);
        self.write(high_base, word_index, (sum >> 16) as u16);
    }
    fn add_word(&mut self, base: usize, word_index: usize, delta: u16) -> u16 {
        let value = EprojState(self.0)
            .slot_word(base, word_index)
            .wrapping_add(delta);
        self.write(base, word_index, value);
        value
    }
    fn decrement(&mut self, base: usize, word_index: usize) -> u16 {
        let timer = EprojState(self.0).slot_word(base, word_index);
        self.write(base, word_index, timer.wrapping_sub(1));
        timer
    }
    fn decrement_instruction_timer(&mut self, word_index: usize) -> u16 {
        self.decrement(0x1b8f, word_index)
    }
    fn decrement_timer(&mut self, word_index: usize) -> u16 {
        self.decrement(0x19df, word_index)
    }
    fn set_timer(&mut self, word_index: usize, value: u16) {
        self.write(0x19df, word_index, value);
    }
    fn set_instruction_timer(&mut self, word_index: usize, value: u16) {
        self.write(0x1b8f, word_index, value);
    }
    fn set_spritemap(&mut self, word_index: usize, value: u16) {
        self.write(0x1b6b, word_index, value);
    }
    fn set_properties(&mut self, word_index: usize, value: u16) {
        self.write(0x1bd7, word_index, value);
    }
    fn set_property_bits(&mut self, word_index: usize, mask: u16) {
        let properties = EprojState(self.0).properties(word_index);
        self.set_properties(word_index, properties | mask);
    }
    fn set_preinstruction(&mut self, word_index: usize, value: u16) {
        self.write(0x1a03, word_index, value);
    }
    fn start_instruction(&mut self, word_index: usize, instruction: u16) {
        self.set_instruction(word_index, instruction);
        self.set_instruction_timer(word_index, 1);
    }
    fn set_instruction(&mut self, word_index: usize, value: u16) {
        self.write(0x1b47, word_index, value);
    }
    fn finish_pickup(&mut self, word_index: usize) {
        self.set_instruction(word_index, addr_word_86ECA3 as u16);
        self.set_instruction_timer(word_index, 1);
        self.set_properties(word_index, 0x3000);
        self.set_preinstruction(word_index, fnEprojPreInstr_Empty as u16);
    }
    fn clear_spawn_variables(&mut self, word_index: usize) {
        // E/F, timer, X/Y subpositions, then G, in the original store order.
        for base in [0x1aff, 0x1b23, 0x19df, 0x1a27, 0x1a6f, 0x1bfb] {
            self.write(base, word_index, 0);
        }
    }
    fn set_id(&mut self, word_index: usize, value: u16) {
        self.write(0x1997, word_index, value);
    }
    fn set_index(&mut self, value: u16) {
        self.write(0x1991, 0, value);
    }
}

pub fn EnableEprojs(game: &mut Game) {
    let flags = EprojState(&game.g_ram).slot_word(0x198d, 0) | 0x8000;
    EprojStateMut(&mut game.g_ram).write(0x198d, 0, flags);
}
pub unsafe fn DisableEprojs(game: &mut Game) {
    let ref mut fresh17 = *((&raw mut game.g_ram as *mut uint8).offset(0x198d) as *mut uint16);
    *fresh17 = (*fresh17 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
}
pub fn ClearEprojs(game: &mut Game) {
    for word_index in (0..18).rev() {
        EprojStateMut(&mut game.g_ram).set_id(word_index, 0);
    }
}
unsafe fn SpawnEprojInner(j: uint16, gfx_idx: uint16, game: &mut Game) -> Option<uint16> {
    let mut v3: uint16 = 34 as uint16;
    while EprojState(&game.g_ram).id(usize::from(v3 >> 1)) != 0 {
        v3 = v3.wrapping_sub(2);
        if (v3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            return None;
        }
    }
    let v4 = usize::from(v3 >> 1);
    EprojStateMut(&mut game.g_ram).write(0x19bb, v4, gfx_idx);
    EprojStateMut(&mut game.g_ram).set_id(v4, j);
    let Edef: *mut EprojDef = get_EprojDef(j, game);
    let value = (*Edef).pre_instr_ptr;
    EprojStateMut(&mut game.g_ram).set_preinstruction(v4, value);
    let value = (*Edef).instr_list;
    EprojStateMut(&mut game.g_ram).set_instruction(v4, value);
    let radius = (*Edef).radius;
    EprojStateMut(&mut game.g_ram).write(0x1bb3, v4, radius);
    let value = (*Edef).properties;
    EprojStateMut(&mut game.g_ram).set_properties(v4, value);
    EprojStateMut(&mut game.g_ram).set_instruction_timer(v4, 1);
    EprojStateMut(&mut game.g_ram).set_spritemap(v4, 0x8000);
    EprojStateMut(&mut game.g_ram).clear_spawn_variables(v4);
    CallEprojInit((((*Edef).init_code_ptr) as u32) | (0x860000u32), v3, game);
    Some(v3)
}
pub unsafe fn SpawnEprojWithGfx(a: uint16, k: uint16, j: uint16, game: &mut Game) -> uint16 {
    EprojStateMut(&mut game.g_ram).write(0x1993, 0, a);
    let enemy_index = k;
    let gfx_idx = (game.enemy_state(enemy_index).vram_tiles_index())
        | (game.enemy_state(enemy_index).palette_index());
    SpawnEprojInner(j, gfx_idx, game).unwrap_or(gfx_idx)
}
pub unsafe fn SpawnEprojWithRoomGfx(j: uint16, a: uint16, game: &mut Game) -> uint16 {
    EprojStateMut(&mut game.g_ram).write(0x1993, 0, a);
    SpawnEprojInner(j, 0, game)
        .unwrap_or_else(|| u16::from_le_bytes([game.g_ram[0x1997], game.g_ram[0x1998]]))
}
unsafe fn EprojRunOne(k: uint16, game: &mut Game) {
    CallEprojPreInstr(
        0x860000 | u32::from(EprojState(&game.g_ram).preinstruction(usize::from(k >> 1))),
        k,
        game,
    );
    let v1 = EprojState(&game.g_ram).index();
    let v2 = usize::from(EprojState(&game.g_ram).index() >> 1);
    let fresh20 = EprojStateMut(&mut game.g_ram).decrement_instruction_timer(v2);
    if fresh20 == 1 {
        let base = RomBankBase(0x86, game);
        let mut p = base.add(usize::from(EprojState(&game.g_ram).instruction(v2)));
        while *(p as *const u16) & 0x8000 != 0 {
            p = CallEprojInstr(
                0x860000 | u32::from(*(p as *const u16)),
                v1,
                p.offset(2),
                game,
            );
            if (p as usize) < 0x10000 {
                if p.is_null() {
                    return;
                }
                p = base.add(p as usize);
            }
        }
        let v7 = usize::from(v1 >> 1);
        let value = *(p as *mut uint16);
        EprojStateMut(&mut game.g_ram).set_instruction_timer(v7, value);
        let value = *(p.offset(2) as *mut uint16);
        EprojStateMut(&mut game.g_ram).set_spritemap(v7, value);
        let value = p.offset(4).offset_from(base) as ::core::ffi::c_long as uint16;
        EprojStateMut(&mut game.g_ram).set_instruction(v7, value);
    }
}
pub unsafe fn EprojRunAll(game: &mut Game) {
    if EprojState(&game.g_ram).enabled() {
        let mut index = Some(34u16);
        while let Some(mut slot) = index {
            EprojStateMut(&mut game.g_ram).set_index(slot);
            if EprojState(&game.g_ram).id(usize::from(slot >> 1)) != 0 {
                EprojRunOne(slot, game);
                slot = EprojState(&game.g_ram).index();
            }
            index = slot.checked_sub(2);
        }
    }
}
pub fn EprojInstr_Delete(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    ::core::ptr::null()
}
pub unsafe fn EprojInstr_Sleep(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    let value = (epjp.offset_from(RomBankBase(0x86 as uint8, game)) as ::core::ffi::c_long
        - 2 as ::core::ffi::c_long) as uint16;
    EprojStateMut(&mut game.g_ram).set_instruction(usize::from(k >> 1), value);
    ::core::ptr::null()
}
pub unsafe fn EprojInstr_SetPreInstr_(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let value = *(epjp as *mut uint16);
    EprojStateMut(&mut game.g_ram).set_preinstruction(usize::from(k >> 1), value);
    epjp.offset(2)
}
pub fn EprojInstr_ClearPreInstr(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    EprojStateMut(&mut game.g_ram).set_preinstruction(usize::from(k >> 1), 0x8170);
    epjp
}
pub unsafe fn CallEprojFunc(ea: uint32, k: uint32, game: &mut Game) {
    match ea {
        8833019 => {
            SpawnMotherBrainDeathBeam(k as uint16, game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn EprojInstr_CallFunc(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    CallEprojFunc(Load24(epjp as *mut LongPtr), k as uint32, game);
    epjp.offset(3)
}
pub unsafe fn EprojInstr_Goto(k: uint16, epjp: *const uint8) -> *const uint8 {
    *(epjp as *mut uint16) as uintptr_t as *const uint8
}
pub unsafe fn EprojInstr_GotoRel(k: uint16, epjp: *const uint8) -> *const uint8 {
    epjp.offset(*epjp as int8 as ::core::ffi::c_int as isize)
}
pub unsafe fn EprojInstr_DecTimerAndGotoIfNonZero(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    if EprojStateMut(&mut game.g_ram).decrement_timer(usize::from(k >> 1)) == 1 {
        epjp.offset(2)
    } else {
        EprojInstr_Goto(k, epjp)
    }
}
pub unsafe fn EprojInstr_DecTimerAndGotoRelIfNonZero(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh46 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x19df) as *mut uint16).offset(v2 as isize);
    let fresh47 = *fresh46;
    *fresh46 = (*fresh46).wrapping_sub(1);
    if fresh47 == 1u16 {
        epjp.offset(1)
    } else {
        EprojInstr_GotoRel(k, epjp)
    }
}
pub unsafe fn EprojInstr_SetTimer(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    let value = *(epjp as *mut uint16);
    EprojStateMut(&mut game.g_ram).set_timer(usize::from(k >> 1), value);
    epjp.offset(2)
}
pub unsafe fn EprojInstr_MoveRandomlyWithinRadius(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut Random: int8 = 0;
    let mut v4: int8 = 0;
    let mut v6: int16 = 0;
    let r18: uint16 = NextRandom(game);
    loop {
        Random = NextRandom(game) as int8;
        v4 = ((*epjp as ::core::ffi::c_int & Random as ::core::ffi::c_int)
            - *epjp.offset(1) as ::core::ffi::c_int) as int8;
        if (v4 >= 0) {
            break;
        }
    }
    let ref mut fresh44 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh44 = (*fresh44 as ::core::ffi::c_int
        + if r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            -(v4 as uint8 as ::core::ffi::c_int)
        } else {
            v4 as uint8 as ::core::ffi::c_int
        }) as uint16;
    loop {
        *(&raw mut v6 as *mut uint8).offset(0) = NextRandom(game) as uint8;
        *(&raw mut v6 as *mut uint8).offset(0) =
            ((*epjp.offset(2) as ::core::ffi::c_int & v6 as ::core::ffi::c_int)
                - *epjp.offset(3) as ::core::ffi::c_int) as uint8;
        if ((v6 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) == 0i32) {
            break;
        }
    }
    v6 = v6 as uint8 as int16;
    if (r18 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int) != 0i32 {
        v6 = -(v6 as uint8 as ::core::ffi::c_int) as int16;
    }
    let ref mut fresh45 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh45 = (*fresh45).wrapping_add((v6) as u16);
    epjp.offset(4)
}
pub unsafe fn EprojInstr_SetProjectileProperties(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let slot = usize::from(k >> 1);
    let properties = EprojState(&game.g_ram).slot_word(0x1bd7, slot);
    let mask = *(epjp as *mut uint16);
    EprojStateMut(&mut game.g_ram).set_properties(slot, properties | mask);
    epjp.offset(2)
}
pub unsafe fn EprojInstr_ClearProjectileProperties(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let slot = usize::from(k >> 1);
    let properties = EprojState(&game.g_ram).slot_word(0x1bd7, slot);
    let mask = *(epjp as *mut uint16);
    EprojStateMut(&mut game.g_ram).set_properties(slot, properties & mask);
    epjp.offset(2)
}
pub unsafe fn EprojInstr_EnableCollisionWithSamusProj(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh41 = *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh41 = (*fresh41 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
    epjp
}
pub unsafe fn EprojInstr_DisableCollisionWithSamusProj(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh40 = *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh40 = (*fresh40 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
    epjp
}
pub unsafe fn EprojInstr_DisableCollisionWithSamus(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh39 = *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh39 = (*fresh39 as ::core::ffi::c_int | 0x2000 as ::core::ffi::c_int) as uint16;
    epjp
}
pub unsafe fn EprojInstr_EnableCollisionWithSamus(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh38 = *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh38 = (*fresh38 as ::core::ffi::c_int & !(0x2000 as ::core::ffi::c_int)) as uint16;
    epjp
}
pub unsafe fn EprojInstr_SetToNotDieOnContact(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh37 = *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh37 = (*fresh37 as ::core::ffi::c_int | 0x4000 as ::core::ffi::c_int) as uint16;
    epjp
}
pub unsafe fn EprojInstr_SetToDieOnContact(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh36 = *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh36 = (*fresh36 as ::core::ffi::c_int & !(0x4000 as ::core::ffi::c_int)) as uint16;
    epjp
}
pub unsafe fn EprojInstr_SetLowPriority(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh35 = *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh35 = (*fresh35 as ::core::ffi::c_int | 0x1000 as ::core::ffi::c_int) as uint16;
    epjp
}
pub unsafe fn EprojInstr_SetHighPriority(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh34 = *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh34 = (*fresh34 as ::core::ffi::c_int & !(0x1000 as ::core::ffi::c_int)) as uint16;
    epjp
}
pub unsafe fn EprojInstr_SetXyRadius(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1bb3) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(epjp as *mut uint16);
    epjp.offset(2)
}
pub unsafe fn EprojInstr_SetXyRadiusZero(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1bb3) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    epjp
}
pub unsafe fn EprojInstr_CalculateDirectionTowardsSamus(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let x: uint16 = (ram_word!(game, samus_x_pos)).wrapping_sub(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize),
    );
    let y: uint16 = (ram_word!(game, samus_y_pos)).wrapping_sub(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize),
    );
    let v3: uint16 = (2u16).wrapping_mul(CalculateAngleFromXY(x, y, game));
    let v4: ::core::ffi::c_int =
        ram_word!(game, cinematic_var7) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v4 as isize) = v3;
    let v5: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v4 as isize) =
        kSinCosTable8bit_Sext[(v5 + 64 as ::core::ffi::c_int) as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v4 as isize) =
        kSinCosTable8bit_Sext[v5 as usize] as uint16;
    epjp
}
pub unsafe fn EprojInstr_WriteColorsToPalette(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let mut v3: uint16 = *(epjp.offset(2) as *mut uint16);
    let mut n: ::core::ffi::c_int = *epjp.offset(4) as ::core::ffi::c_int;
    let mut v4: uint16 = *(epjp as *mut uint16);
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *(RomPtr_86(v4, game) as *mut uint16);
        v4 = (v4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        n -= 1;
        if ((n & 0x8000 as ::core::ffi::c_int) != 0i32) {
            break;
        }
    }
    epjp.offset(5)
}
pub unsafe fn EprojInstr_QueueMusic(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueMusic_Delayed8(*epjp as uint16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx1_Max6(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx1_Max6(*epjp as uint16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx2_Max6(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max6(*epjp as uint16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx3_Max6(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx3_Max6(*epjp as uint16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx1_Max15(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx1_Max15(*epjp as uint16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx2_Max15(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max15(*epjp as uint16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx3_Max15(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx3_Max15(*epjp as uint16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx1_Max3(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx1_Max3(*epjp as uint16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx2_Max3(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max3(*epjp as uint16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx3_Max3(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx3_Max3(*epjp as uint16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx1_Max9(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx1_Max9(*epjp as uint16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx2_Max9(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max9(*epjp as uint16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx3_Max9(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx3_Max9(*epjp as uint16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx1_Max1(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx1_Max1(*epjp as uint16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx2_Max1(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max1(*epjp as uint16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx3_Max1(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx3_Max1(*epjp as uint16, game);
    epjp.offset(1)
}
pub enum EprojPriority {
    Low,
    High,
}

pub unsafe fn DrawEprojsWithPriority(priority: EprojPriority, game: &mut Game) {
    let pt = GetValuesForScreenShaking(game);
    let low_priority = matches!(priority, EprojPriority::Low);
    for slot in (0..18).rev() {
        if EprojState(&game.g_ram).id(slot) != 0
            && (EprojState(&game.g_ram).properties(slot) & 0x1000 != 0) == low_priority
        {
            DrawEprojs((slot * 2) as u16, pt, game);
        }
    }
}
pub unsafe fn DrawEprojs(k: uint16, pt: Point16U, game: &mut Game) {
    let v1 = usize::from(k >> 1);
    let v2: uint16 = EprojState(&game.g_ram).spritemap(v1);
    let r26: uint16 = EprojState(&game.g_ram).gfx_low(v1) as uint16;
    let R28: uint16 = EprojState(&game.g_ram).slot_word(0x19bb, v1) & 0xff00;
    let r20: uint16 = (pt.x as ::core::ffi::c_int
        + game.enemy_projectile_x_pos(k) as ::core::ffi::c_int
        - game.layer1_x_pos() as ::core::ffi::c_int) as uint16;
    if (r20 as ::core::ffi::c_int + 128 as ::core::ffi::c_int & 0xfe00 as ::core::ffi::c_int)
        == 0i32
    {
        let v3: uint16 = (pt.y as ::core::ffi::c_int
            + game.enemy_projectile_y_pos(k) as ::core::ffi::c_int
            - game.layer1_y_pos() as ::core::ffi::c_int) as uint16;
        let r18: uint16 = v3;
        if (v3 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int) != 0i32 {
            if (v3 as ::core::ffi::c_int + 128 as ::core::ffi::c_int & 0xfe00 as ::core::ffi::c_int)
                == 0i32
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
    if ram_word!(game, earthquake_timer) != 0u16
        && game.time_is_frozen_flag() == 0
        && ram_word!(game, earthquake_type) as ::core::ffi::c_int - 36 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let v0: ::core::ffi::c_int = ((4u16).wrapping_mul(ram_word!(game, earthquake_type)))
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        if (ram_word!(game, earthquake_timer) as ::core::ffi::c_int & 2 as ::core::ffi::c_int)
            != 0i32
        {
            R36 = -(*(game.g_rom.offset(crate::rtl::lorom_offset(0x86846b)) as *const uint8
                as *mut uint16)
                .offset(v0 as isize) as ::core::ffi::c_int) as uint16;
            R34 = -(*(game.g_rom.offset(crate::rtl::lorom_offset(0x86846b)) as *const uint8
                as *mut uint16)
                .offset((v0 + 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int) as uint16;
        } else {
            R36 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x86846b)) as *const uint8
                as *mut uint16)
                .offset(v0 as isize);
            R34 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x86846b)) as *const uint8
                as *mut uint16)
                .offset((v0 + 1 as ::core::ffi::c_int) as isize);
        }
    } else {
        R34 = 0 as uint16;
        R36 = 0 as uint16;
    }
    Point16U { x: R36, y: R34 }
}
unsafe fn EprojColl_8506(eci: *mut EprojCollInfo, game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(ram_word!(game, cur_block_index) as isize)
        != 0
    {
        let ref mut fresh1 = *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16);
        *fresh1 = (*fresh1 as ::core::ffi::c_int
            + *(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402)
                .offset(ram_word!(game, cur_block_index) as isize) as int8
                as ::core::ffi::c_int) as uint16;
        return 0xff as uint8;
    }
    0 as uint8
}
unsafe fn EprojColl_Unknown8536(eci: *mut EprojCollInfo, game: &mut Game) -> uint8 {
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(game.cur_block_index() as isize)
        != 0
    {
        let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16);
        *fresh0 = (*fresh0 as ::core::ffi::c_int
            + *(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402)
                .offset(ram_word!(game, cur_block_index) as isize) as int8
                as ::core::ffi::c_int
                * ram_word!(game, room_width_in_blocks) as ::core::ffi::c_int)
            as uint16;
        return 0xff as uint8;
    }
    0 as uint8
}
unsafe fn EprojColl_858E(eci: *mut EprojCollInfo, game: &mut Game) -> uint8 {
    let v0: uint16 = ((*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(game.cur_block_index() as isize)) as u16)
        & (0x1fu16);
    if v0 < 5 {
        return EprojColl_85C2(eci, v0, ram_word!(game, cur_block_index), game);
    }
    set_ram_word!(
        game.current_slope_bts = *(&raw mut game.g_ram as *mut uint8)
            .offset(0x16402)
            .offset(game.cur_block_index() as isize) as uint16
    );
    EprojColl_873D()
}
unsafe fn EprojColl_85AD(eci: *mut EprojCollInfo, game: &mut Game) -> uint8 {
    let v0: uint16 = ((*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(game.cur_block_index() as isize)) as u16)
        & (0x1fu16);
    if v0 >= 5 {
        EprojColl_874E(eci, game)
    } else {
        EprojColl_8676(eci, v0, game.cur_block_index(), 0 as uint16, game)
    }
}
static unk_868729: [uint8; 20] = [
    0 as uint8,
    1 as uint8,
    0x82 as uint8,
    0x83 as uint8,
    0 as uint8,
    0x81 as uint8,
    2 as uint8,
    0x83 as uint8,
    0 as uint8,
    1 as uint8,
    2 as uint8,
    0x83 as uint8,
    0 as uint8,
    0x81 as uint8,
    0x82 as uint8,
    0x83 as uint8,
    0x80 as uint8,
    0x81 as uint8,
    0x82 as uint8,
    0x83 as uint8,
];
unsafe fn EprojColl_85C2(eci: *mut EprojCollInfo, a: uint16, k: uint16, game: &mut Game) -> uint8 {
    let mut current_block: u64;
    let temp_collision_DD4: uint16 = (4 as ::core::ffi::c_int * a as ::core::ffi::c_int) as uint16;
    let temp_collision_DD6: uint16 = (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(k as isize) as ::core::ffi::c_int
        >> 6 as ::core::ffi::c_int) as uint16;
    let v2: uint16 = (4 as ::core::ffi::c_int * a as ::core::ffi::c_int
        + (temp_collision_DD6 as ::core::ffi::c_int
            ^ ((*eci).eci_r34 as ::core::ffi::c_int & 8 as ::core::ffi::c_int)
                >> 3 as ::core::ffi::c_int)) as uint16;
    if (*eci).eci_r32 == 0 {
        let v3: ::core::ffi::c_int =
            ram_word!(game, cinematic_var7) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v3 as isize)
            as ::core::ffi::c_int
            - (*eci).eci_r30 as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int)
            != 0i32
            || (if unk_868729[v2 as usize] as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
                -(1 as ::core::ffi::c_int)
            } else {
                0 as ::core::ffi::c_int
            }) >= 0i32
        {
            let v4: uint16 = (v2 as ::core::ffi::c_int ^ 2 as ::core::ffi::c_int) as uint16;
            if ((*eci).eci_r30 as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int)
                == 0i32
                || (if unk_868729[v4 as usize] as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
                    != 0
                {
                    -(1 as ::core::ffi::c_int)
                } else {
                    0 as ::core::ffi::c_int
                }) >= 0i32
            {
                return 0 as uint8;
            }
        }
    } else {
        if (*eci).eci_r26 == 0 {
            if ((*eci).eci_r30 as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(
                    (ram_word!(game, cinematic_var7) as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int)
                == 0i32
            {
                if (if unk_868729[v2 as usize] as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
                    != 0
                {
                    -(1 as ::core::ffi::c_int)
                } else {
                    0 as ::core::ffi::c_int
                }) >= 0i32
                {
                    return 0 as uint8;
                }
                current_block = 12039483399334584727;
            } else {
                current_block = 2204372562120708137;
            }
        } else if (*eci).eci_r26 != (*eci).eci_r32
            || (*((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(
                (ram_word!(game, cinematic_var7) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                    as isize,
            ) as ::core::ffi::c_int
                - (*eci).eci_r30 as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int)
                == 0i32
        {
            current_block = 2204372562120708137;
        } else {
            current_block = 3512920355445576850;
        }
        match current_block {
            12039483399334584727 => {}
            _ => {
                if current_block == 2204372562120708137 {
                    if (if unk_868729[v2 as usize] as ::core::ffi::c_int
                        & 0x80 as ::core::ffi::c_int
                        != 0
                    {
                        -(1 as ::core::ffi::c_int)
                    } else {
                        0 as ::core::ffi::c_int
                    }) < 0i32
                    {
                        current_block = 12039483399334584727;
                    } else {
                        current_block = 3512920355445576850;
                    }
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
                        }) >= 0i32
                        {
                            return 0 as uint8;
                        }
                    }
                }
            }
        }
    }
    let v6: ::core::ffi::c_int =
        ram_word!(game, cinematic_var7) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v7: uint16 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v6 as isize) =
        0 as uint16;
    if ((*eci).eci_r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        v7 = ((*eci).eci_r28 as ::core::ffi::c_int
            + ((*eci).eci_r34 as ::core::ffi::c_int | 7 as ::core::ffi::c_int)
            + 1 as ::core::ffi::c_int) as uint16;
    } else {
        v7 = (((*eci).eci_r34 as ::core::ffi::c_int & 0xfff8 as ::core::ffi::c_int)
            - (*eci).eci_r28 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v6 as isize) = v7;
    1 as uint8
}
unsafe fn EprojColl_8676(
    eci: *mut EprojCollInfo,
    a: uint16,
    k: uint16,
    j: uint16,
    game: &mut Game,
) -> uint8 {
    let mut current_block: u64;
    let v2: uint16 = EprojState(&game.g_ram).index();
    let temp_collision_DD4: uint16 = (4u16).wrapping_mul(a);
    let temp_collision_DD6: uint16 = (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(k as isize) as ::core::ffi::c_int
        >> 6 as ::core::ffi::c_int) as uint16;
    let v3: uint16 = (4 as ::core::ffi::c_int * a as ::core::ffi::c_int
        + (temp_collision_DD6 as ::core::ffi::c_int
            ^ ((*eci).eci_r34 as ::core::ffi::c_int & 8 as ::core::ffi::c_int)
                >> 2 as ::core::ffi::c_int)) as uint16;
    if (*eci).eci_r32 == 0 {
        let v4: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if (game.enemy_projectile_x_pos(v2) as ::core::ffi::c_int
            - (*eci).eci_r28 as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int)
            != 0i32
            || (if unk_868729[v3 as usize] as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
                -(1 as ::core::ffi::c_int)
            } else {
                0 as ::core::ffi::c_int
            }) >= 0i32
        {
            let v5: uint16 = (v3) ^ (1u16);
            if ((*eci).eci_r28 as ::core::ffi::c_int
                + game.enemy_projectile_x_pos(v2) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int)
                == 0i32
                || (if unk_868729[v5 as usize] as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
                    != 0
                {
                    -(1 as ::core::ffi::c_int)
                } else {
                    0 as ::core::ffi::c_int
                }) >= 0i32
            {
                return 0 as uint8;
            }
        }
    } else {
        if (*eci).eci_r26 == 0 {
            if ((*eci).eci_r28 as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(
                    (ram_word!(game, cinematic_var7) as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int)
                == 0i32
            {
                if (if unk_868729[v3 as usize] as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
                    != 0
                {
                    -(1 as ::core::ffi::c_int)
                } else {
                    0 as ::core::ffi::c_int
                }) >= 0i32
                {
                    return 0 as uint8;
                }
                current_block = 5143058163439228106;
            } else {
                current_block = 8777407941634159499;
            }
        } else if (*eci).eci_r26 != (*eci).eci_r32
            || (*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(
                (ram_word!(game, cinematic_var7) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                    as isize,
            ) as ::core::ffi::c_int
                - (*eci).eci_r28 as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int)
                == 0i32
        {
            current_block = 8777407941634159499;
        } else {
            current_block = 12349973810996921269;
        }
        match current_block {
            5143058163439228106 => {}
            _ => {
                if current_block == 8777407941634159499 {
                    if (if unk_868729[v3 as usize] as ::core::ffi::c_int
                        & 0x80 as ::core::ffi::c_int
                        != 0
                    {
                        -(1 as ::core::ffi::c_int)
                    } else {
                        0 as ::core::ffi::c_int
                    }) < 0i32
                    {
                        current_block = 5143058163439228106;
                    } else {
                        current_block = 12349973810996921269;
                    }
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
                        }) >= 0i32
                        {
                            return 0 as uint8;
                        }
                    }
                }
            }
        }
    }
    let v7: ::core::ffi::c_int =
        EprojState(&game.g_ram).index() as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v8: uint16 = 0;
    EprojStateMut(&mut game.g_ram).write(0x1a6f, v7 as usize, 0 as uint16);
    if ((*eci).eci_r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        v8 = ((*eci).eci_r30 as ::core::ffi::c_int
            + ((*eci).eci_r34 as ::core::ffi::c_int | 7 as ::core::ffi::c_int)
            + 1 as ::core::ffi::c_int) as uint16;
    } else {
        v8 = (((*eci).eci_r34 as ::core::ffi::c_int & 0xfff8 as ::core::ffi::c_int)
            - (*eci).eci_r30 as ::core::ffi::c_int) as uint16;
    }
    EprojStateMut(&mut game.g_ram).write(0x1a93, v7 as usize, v8);
    1 as uint8
}
pub fn EprojColl_873D() -> uint8 {
    0 as uint8
}
unsafe fn EprojColl_874E(eci: *mut EprojCollInfo, game: &mut Game) -> uint8 {
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
    let v0: uint16 = EprojState(&game.g_ram).index();
    if ((*eci).eci_r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        let v9: uint16 = ram_word!(game, cur_block_index);
        let mod_0: uint16 = SnesModulus(
            ram_word!(game, cur_block_index),
            ram_word!(game, room_width_in_blocks) as uint8,
        );
        let v10: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        v11 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
            .offset(v10 as isize) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as int16;
        if v11 as ::core::ffi::c_int == mod_0 as ::core::ffi::c_int {
            let temp_collision_DD4: uint16 = ((*eci).eci_r24 as ::core::ffi::c_int
                - (*eci).eci_r30 as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int
                ^ 0xf as ::core::ffi::c_int) as uint16;
            let temp_collision_DD6: uint16 = (16 as ::core::ffi::c_int
                * (*(&raw mut game.g_ram as *mut uint8)
                    .offset(0x16402)
                    .offset(v9 as isize) as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int))
                as uint16;
            v12 = ((*(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402)
                .offset(v9 as isize) as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int) as int16;
            if v12 < 0i16 && {
                if (v12 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int) != 0i32 {
                    v13 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                        .offset(v10 as isize) as ::core::ffi::c_int
                        ^ 0xf as ::core::ffi::c_int) as uint16;
                } else {
                    v13 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                        .offset(v10 as isize);
                };
                v14 = (temp_collision_DD6 as ::core::ffi::c_int
                    + (v13 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int))
                    as uint16;
                v15 = ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x948b2b)) as *const uint8
                    as *mut uint8)
                    .offset(v14 as isize) as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int)
                    - temp_collision_DD4 as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as int16;
                ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x948b2b)) as *const uint8
                    as *mut uint8)
                    .offset(v14 as isize) as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int)
                    - temp_collision_DD4 as ::core::ffi::c_int)
                    == 1i32
                    || v15 < 0i16
            } {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                    .offset(v10 as isize) =
                    ((*eci).eci_r24 as ::core::ffi::c_int - v15 as ::core::ffi::c_int) as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16)
                    .offset(v10 as isize) = 0 as uint16;
                1 as uint8
            } else {
                0 as uint8
            }
        } else {
            0 as uint8
        }
    } else {
        let v1: uint16 = game.cur_block_index();
        let mod_1: uint16 =
            SnesModulus(game.cur_block_index(), game.room_width_in_blocks() as uint8);
        let v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        v3 = (game.enemy_projectile_x_pos(v0) as ::core::ffi::c_int >> 4 as ::core::ffi::c_int)
            as int16;
        if v3 as ::core::ffi::c_int == mod_1 as ::core::ffi::c_int {
            let temp_collision_DD4_0: uint16 = (((*eci).eci_r30 as ::core::ffi::c_int
                + (*eci).eci_r24 as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as u16)
                & (0xfu16);
            let temp_collision_DD6_0: uint16 = (16u16).wrapping_mul(
                (*(&raw mut game.g_ram as *mut uint8)
                    .offset(0x16402)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int) as u16,
            );
            v5 = ((*(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402)
                .offset(v1 as isize) as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int) as int16;
            if v5 >= 0 && {
                if (v5 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int) != 0i32 {
                    v6 = (game.enemy_projectile_x_pos(v0)) ^ (0xfu16);
                } else {
                    v6 = game.enemy_projectile_x_pos(v0);
                };
                v7 = (temp_collision_DD6_0)
                    .wrapping_add((v6 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as u16);
                v8 = (((*(game.g_rom.offset(crate::rtl::lorom_offset(0x948b2b)) as *const uint8
                    as *mut uint8)
                    .offset(v7 as isize) as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int)
                    - temp_collision_DD4_0 as ::core::ffi::c_int) as i16)
                    .wrapping_sub(1i16);
                ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x948b2b)) as *const uint8
                    as *mut uint8)
                    .offset(v7 as isize) as ::core::ffi::c_int
                    & 0x1f as ::core::ffi::c_int)
                    - temp_collision_DD4_0 as ::core::ffi::c_int)
                    == 1i32
                    || v8 < 0
            } {
                {
                    let eproj_position: uint16 = ((*eci).eci_r24).wrapping_add((v8) as u16);
                    EprojStateMut(&mut game.g_ram).write(0x1a93, v2 as usize, eproj_position);
                }
                EprojStateMut(&mut game.g_ram).write(
                    0x1a6f,
                    v2 as usize,
                    -(1 as ::core::ffi::c_int) as uint16,
                );
                1 as uint8
            } else {
                0 as uint8
            }
        } else {
            0 as uint8
        }
    }
}
unsafe fn react_to_enemy_projectile_block(
    eci: *mut EprojCollInfo,
    k: uint16,
    slope: Func_EprojCollInfo_U8,
    game: &mut Game,
) -> uint8 {
    set_ram_word!(
        game.cur_block_index = (k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16
    );
    game.react_to_block(|block_type, game| match block_type {
        0 | 2 | 3 | 4 | 6 | 7 | 14 => 0,
        1 => slope(eci, game),
        5 => EprojColl_8506(eci, game),
        8 | 9 | 10 | 11 | 12 | 15 => 1,
        13 => EprojColl_Unknown8536(eci, game),
        _ => unreachable!(),
    })
}
pub unsafe fn EprojBlockCollisition_CheckHorizontal(
    eci: *mut EprojCollInfo,
    k: uint16,
    game: &mut Game,
) -> uint8 {
    react_to_enemy_projectile_block(eci, k, EprojColl_858E, game)
}
pub unsafe fn EprojBlockCollisition_CheckVertical(
    eci: *mut EprojCollInfo,
    k: uint16,
    game: &mut Game,
) -> uint8 {
    react_to_enemy_projectile_block(eci, k, EprojColl_85AD, game)
}
pub unsafe fn EprojBlockCollisition_Horiz(k: uint16, game: &mut Game) -> uint8 {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let amt: int32 = (EprojState(&game.g_ram).slot_word(0x1ab7, v1 as usize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let R30: uint16 = EprojState(&game.g_ram).y_radius(v1 as usize) as uint16;
    let R28: uint16 = EprojState(&game.g_ram).x_radius(v1 as usize) as uint16;
    let mut r26: uint16 = ((game.enemy_projectile_y_pos(k) as ::core::ffi::c_int
        - R30 as ::core::ffi::c_int) as u16)
        & (0xfff0u16);
    r26 = (R30
        .wrapping_add(game.enemy_projectile_y_pos(k))
        .wrapping_sub(1)
        .wrapping_sub(r26) as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    let R32: uint16 = r26;
    let prod: uint16 = Mult8x8(
        (game.enemy_projectile_y_pos(k).wrapping_sub(R30) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint8,
        game.room_width_in_blocks() as uint8,
    );
    let v3: uint16 = ((amt as uint32).wrapping_add(
        (game.enemy_projectile_x_pos(k) as uint32) << 16 as ::core::ffi::c_int
            | EprojState(&game.g_ram).slot_word(0x1a27, v1 as usize) as uint32,
    ) >> 16 as ::core::ffi::c_int) as uint16;
    let r22: uint16 =
        (amt + EprojState(&game.g_ram).slot_word(0x1a27, v1 as usize) as int32) as uint16;
    let r24: uint16 = v3;
    let mut v4: uint16 = 0;
    if amt < 0i32 {
        v4 = v3.wrapping_sub(R28);
    } else {
        v4 = R28.wrapping_add(v3).wrapping_sub(1);
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
        v5 = (v5).wrapping_add(
            (game.room_width_in_blocks() as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as u16,
        );
        eci.eci_r26 = eci.eci_r26.wrapping_sub(1);
        if (eci.eci_r26 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            let v6: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            EprojStateMut(&mut game.g_ram).write(0x1a27, v6 as usize, r22);
            EprojStateMut(&mut game.g_ram).write(0x1a4b, v6 as usize, r24);
            return 0 as uint8;
        }
    }
    let v8: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    EprojStateMut(&mut game.g_ram).write(0x1a27, v8 as usize, 0 as uint16);
    if amt < 0i32 {
        let v10: uint16 = (eci.eci_r28 as ::core::ffi::c_int
            + (eci.eci_r34 as ::core::ffi::c_int | 0xf as ::core::ffi::c_int)
            + 1 as ::core::ffi::c_int) as uint16;
        if v10 <= game.enemy_projectile_x_pos(k) {
            EprojStateMut(&mut game.g_ram).write(0x1a4b, v8 as usize, v10);
        }
    } else {
        let v9: uint16 = ((eci.eci_r34 as ::core::ffi::c_int & 0xfff0 as ::core::ffi::c_int)
            - eci.eci_r28 as ::core::ffi::c_int) as uint16;
        if v9 >= game.enemy_projectile_x_pos(k) {
            EprojStateMut(&mut game.g_ram).write(0x1a4b, v8 as usize, v9);
        }
    }
    1 as uint8
}
pub unsafe fn EprojBlockCollisition_Vertical(k: uint16, game: &mut Game) -> uint8 {
    let mut v5: int16 = 0;
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let amt: int32 = (EprojState(&game.g_ram).slot_word(0x1adb, v1 as usize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let R30: uint16 = EprojState(&game.g_ram).y_radius(v1 as usize) as uint16;
    let R28: uint16 = EprojState(&game.g_ram).x_radius(v1 as usize) as uint16;
    let mut r26: uint16 = ((game.enemy_projectile_x_pos(k) as ::core::ffi::c_int
        - R28 as ::core::ffi::c_int) as u16)
        & (0xfff0u16);
    r26 = (R28
        .wrapping_add(game.enemy_projectile_x_pos(k))
        .wrapping_sub(1)
        .wrapping_sub(r26) as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    let R32: uint16 = r26;
    let v3: uint16 = ((amt as uint32).wrapping_add(
        (game.enemy_projectile_y_pos(k) as uint32) << 16 as ::core::ffi::c_int
            | EprojState(&game.g_ram).slot_word(0x1a6f, v1 as usize) as uint32,
    ) >> 16 as ::core::ffi::c_int) as uint16;
    let r22: uint16 =
        (amt + EprojState(&game.g_ram).slot_word(0x1a6f, v1 as usize) as int32) as uint16;
    let r24: uint16 = v3;
    let mut v4: uint16 = 0;
    if amt < 0i32 {
        v4 = v3.wrapping_sub(R30);
    } else {
        v4 = R30.wrapping_add(v3).wrapping_sub(1);
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
    let prod: uint16 = Mult8x8(
        (v4 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint8,
        game.room_width_in_blocks() as uint8,
    );
    v5 = (game.enemy_projectile_x_pos(k).wrapping_sub(R28) as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as int16;
    let mut i: ::core::ffi::c_int =
        2 as ::core::ffi::c_int * (prod as ::core::ffi::c_int + v5 as ::core::ffi::c_int);
    while EprojBlockCollisition_CheckVertical(&raw mut eci, i as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        eci.eci_r26 = eci.eci_r26.wrapping_sub(1);
        if (eci.eci_r26 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            let v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            EprojStateMut(&mut game.g_ram).write(0x1a6f, v7 as usize, r22);
            EprojStateMut(&mut game.g_ram).write(0x1a93, v7 as usize, r24);
            return 0 as uint8;
        }
        i += 2 as ::core::ffi::c_int;
    }
    let v9: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    EprojStateMut(&mut game.g_ram).write(0x1a6f, v9 as usize, 0 as uint16);
    if amt < 0i32 {
        let v11: uint16 = (eci.eci_r30 as ::core::ffi::c_int
            + (eci.eci_r34 as ::core::ffi::c_int | 0xf as ::core::ffi::c_int)
            + 1 as ::core::ffi::c_int) as uint16;
        if v11 <= game.enemy_projectile_y_pos(k) {
            EprojStateMut(&mut game.g_ram).write(0x1a93, v9 as usize, v11);
        }
    } else {
        let v10: uint16 = ((eci.eci_r34 as ::core::ffi::c_int & 0xfff0 as ::core::ffi::c_int)
            - eci.eci_r30 as ::core::ffi::c_int) as uint16;
        if v10 >= game.enemy_projectile_y_pos(k) {
            EprojStateMut(&mut game.g_ram).write(0x1a93, v9 as usize, v10);
        }
    }
    1 as uint8
}
unsafe fn Eproj_Init_0x8aaf(j: uint16, game: &mut Game) {
    NextRandom(game);
    let v1: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v2: uint16 = ((*v1).y_pos as ::core::ffi::c_int + 12 as ::core::ffi::c_int) as uint16;
    let v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v3 as isize) = v2;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19df) as *mut uint16).offset(v3 as isize) =
        (v2 as ::core::ffi::c_int + 72 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v3 as isize) =
        (*v1).ai_var_D;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v3 as isize) =
        ((*v1).x_pos as ::core::ffi::c_int
            + (ram_word!(game, random_number) as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
            - 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v3 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x868a75)) as *const uint8 as *mut uint16)
            .offset(
                ((*(((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                    .offset(v3 as isize) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ) as ::core::ffi::c_int
                    & 6 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
}
pub unsafe fn Eproj_PreInit_0x8aaf(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
            as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
        >= *((&raw mut game.g_ram as *mut uint8).offset(0x19df) as *mut uint16).offset(v1 as isize)
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
    }
}
fn init_skree_particle(j: u16, x_offset: i16, x_velocity: i16, y_velocity: i16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let word_index = usize::from(j >> 1);
    let y = game.enemy_state(enemy_index).y_pos();
    EprojStateMut(&mut game.g_ram).write(0x1a93, word_index, y);
    EprojStateMut(&mut game.g_ram).write(0x1adb, word_index, y_velocity as u16);
    let x = game
        .enemy_state(enemy_index)
        .x_pos()
        .wrapping_add_signed(x_offset);
    EprojStateMut(&mut game.g_ram).write(0x1a4b, word_index, x);
    EprojStateMut(&mut game.g_ram).write(0x1ab7, word_index, x_velocity as u16);
}

pub fn Eproj_PreInstr_SkreeParticle(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let split_amount = ((EprojState(&game.g_ram).slot_word(0x1ab7, v1 as usize) as int16
        as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a4b, 0x1a27, v1 as usize, split_amount);
    let split_amount = ((EprojState(&game.g_ram).slot_word(0x1adb, v1 as usize) as int16
        as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a93, 0x1a6f, v1 as usize, split_amount);
    EprojStateMut(&mut game.g_ram).add_word(0x1adb, v1 as usize, 80);
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        EprojStateMut(&mut game.g_ram).set_id(v1 as usize, 0);
    }
}
pub unsafe fn EprojInstr_SpawnEnemyDropsWithDraygonsEyeDrops(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize),
    };
    SpawnEnemyDrops(addr_kEnemyDef_DE7F as uint16, k, 0 as uint16, game);
    epjp
}
pub unsafe fn EprojInstr_SetPreInstrA(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (0x868dff as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    epjp
}
pub unsafe fn EprojInstr_SetPreInstrB(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (0x868dca as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    epjp
}
unsafe fn EprojInit_DraygonsGunk(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        game.eproj_spawn_pt.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        game.eproj_spawn_pt.y;
    *((&raw mut game.g_ram as *mut uint8).offset(0x97dc) as *mut uint16).offset(v1 as isize) =
        ram_word!(game, cinematic_var9);
    let pt: Point32 = ConvertAngleToXy(
        ram_word!(game, cinematic_var9),
        ram_word!(game, cinematic_var8),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        (pt.x >> 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        pt.x as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        (pt.y >> 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
        pt.y as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        1024 as uint16;
}
unsafe fn EprojInit_DraygonsWallTurretProjs(j: uint16, game: &mut Game) {
    Eproj_AngleToSamus(j, game.eproj_spawn_pt.x, game.eproj_spawn_pt.y, game);
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        2560 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v1 as isize) =
        (0x868d54 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojPowerBombCollision(k: uint16, game: &mut Game) {
    let r18: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) != 0
    {
        let r20: uint16 = ((r18 as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int)
            + (ram_word!(game, power_bomb_explosion_radius) as ::core::ffi::c_int
                >> 9 as ::core::ffi::c_int)) as uint16
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16;
        let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if abs16(
            (ram_word!(game, power_bomb_explosion_x_pos) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int) as uint16,
        ) < r18
            && abs16(
                (ram_word!(game, power_bomb_explosion_y_pos) as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int) as uint16,
            ) < r20
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
            set_ram_word!(game.samus_x_speed_divisor = 0 as uint16);
        }
    }
}
pub unsafe fn EprojInstr_868D99(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    EprojPowerBombCollision(k, game);
    let v2: uint16 = (ram_word!(game, samus_x_speed_divisor) as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint16;
    if ram_word!(game, samus_x_speed_divisor) as ::core::ffi::c_int - 5 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh33 = *((&raw mut game.g_ram as *mut uint8).offset(0xa66) as *mut uint16);
        *fresh33 = (*fresh33).wrapping_add(1);
        let v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v3 as isize) =
            v2;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v3 as isize) =
            256 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v3 as isize) =
            (0x868dca as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16).offset(v3 as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16)
                .offset(v3 as isize) as ::core::ffi::c_int
                & 0x5fff as ::core::ffi::c_int
                | 0x2000 as ::core::ffi::c_int) as uint16;
        set_ram_word!(game.samus_invincibility_timer = 0 as uint16);
        set_ram_word!(game.samus_knockback_timer = 0 as uint16);
    }
    epjp
}
pub unsafe fn EprojPreInstr_8DCA(k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = 0;
    EprojPowerBombCollision(k, game);
    if ram_word!(game, samus_contact_damage_index) != 0u16 || {
        v1 = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
            ram_word!(game, samus_x_pos);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
            (ram_word!(game, samus_y_pos) as ::core::ffi::c_int
                + 4 as ::core::ffi::c_int
                    * *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int
                - 12 as ::core::ffi::c_int) as uint16;
        let ref mut fresh143 = *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset(v1 as isize);
        *fresh143 = (*fresh143).wrapping_sub(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize)
            == 0
    } {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        let ref mut fresh144 = *((&raw mut game.g_ram as *mut uint8).offset(0xa66) as *mut uint16);
        *fresh144 = (*fresh144).wrapping_sub(1);
        if (*fresh144 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            set_ram_word!(game.samus_x_speed_divisor = 0 as uint16);
        }
    }
}
pub unsafe fn EprojPreInstr_DraygonsTurret_8DFF(k: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    EprojPowerBombCollision(k, game);
    Eproj_FuncE73E_MoveXY(k, game);
    v1 = Eproj_FuncE722(k, game) as int16;
    if v1 != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojPreInstr_DraygonsGunk_8E0F(k: uint16, game: &mut Game) {
    let mut v3: uint16 = 0;
    EprojPowerBombCollision(k, game);
    Eproj_FuncE73E_MoveXY(k, game);
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: uint16 = abs16(
        (ram_word!(game, samus_x_pos) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16,
    );
    if v2 as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 && {
        v3 = abs16(
            (ram_word!(game, samus_y_pos) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int) as uint16,
        );
        v3 as ::core::ffi::c_int - 20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    } {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
            0x8c38 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    } else if Eproj_FuncE722(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
    }
}
unsafe fn EprojInit_CrocomireProjectile(j: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        -(512 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        1 as uint16;
    let v3: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        ((*v3).x_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        ((*v3).y_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19df) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v2 as isize) =
        2560 as uint16;
}
pub unsafe fn EprojPreInstr_CrocomireProjectile(k: uint16, game: &mut Game) {
    EprojBlockCollisition_Horiz(k, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(0) = 2560 as uint16;
    let ref mut fresh142 = *((&raw mut game.g_ram as *mut uint8).offset(0x19df) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh142 = (*fresh142 as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int) as uint16;
    let x: uint16 = -(64 as ::core::ffi::c_int) as uint16;
    let y: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x869059)) as *const uint8
        as *mut uint16)
        .offset(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .ai_preinstr as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    let v1: ::core::ffi::c_int = CalculateAngleFromXY(x, y, game) as ::core::ffi::c_int;
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) = (4
        as ::core::ffi::c_int
        * kSinCosTable8bit_Sext[(v1 + 64 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int)
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        (4 as ::core::ffi::c_int * kSinCosTable8bit_Sext[v1 as usize] as ::core::ffi::c_int)
            as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v2 as isize) =
        (0x8690b3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn sub_8690B3(k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
unsafe fn EprojInit_CrocomireSpikeWallPieces(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x869105)) as *const uint8 as *mut uint16)
            .offset(
                ((j as ::core::ffi::c_int - 20 as ::core::ffi::c_int) as uint16
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        528 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        -(5 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
        -(30720 as ::core::ffi::c_int) as uint16;
}
static CrocomireSpikeWallPieces_Tab2: [uint16; 18] = [
    0 as uint16,
    0 as uint16,
    0xff0 as uint16,
    0xee0 as uint16,
    0xcc0 as uint16,
    0xaa0 as uint16,
    0x880 as uint16,
    0x660 as uint16,
    0x440 as uint16,
    0x220 as uint16,
    0xff0 as uint16,
    0xee0 as uint16,
    0xcc0 as uint16,
    0xaa0 as uint16,
    0x880 as uint16,
    0x660 as uint16,
    0x440 as uint16,
    0x220 as uint16,
];
static CrocomireSpikeWallPieces_Tab1: [uint16; 18] = [
    0 as uint16,
    0 as uint16,
    0xff00 as uint16,
    0xee00 as uint16,
    0xcc00 as uint16,
    0xaa00 as uint16,
    0x8800 as uint16,
    0x6600 as uint16,
    0x4400 as uint16,
    0x2200 as uint16,
    0xff00 as uint16,
    0xee00 as uint16,
    0xcc00 as uint16,
    0xaa00 as uint16,
    0x8800 as uint16,
    0x6600 as uint16,
    0x4400 as uint16,
    0x2200 as uint16,
];
static CrocomireSpikeWallPieces_Tab3: [int8; 36] = [
    0 as int8, 0 as int8, 0 as int8, 0 as int8, 4 as int8, 0 as int8, 4 as int8, 0 as int8,
    3 as int8, 0 as int8, 3 as int8, 0 as int8, 2 as int8, 0 as int8, 2 as int8, 0 as int8,
    1 as int8, 0 as int8, 1 as int8, 0 as int8, 6 as int8, 0 as int8, 5 as int8, 0 as int8,
    4 as int8, 0 as int8, 3 as int8, 0 as int8, 2 as int8, 0 as int8, 2 as int8, 0 as int8,
    1 as int8, 0 as int8, 1 as int8, 0 as int8,
];
pub unsafe fn EprojPreInstr_CrocomireSpikeWallPieces(k: uint16, game: &mut Game) {
    let mut v3: int8 = 0;
    let mut v6: int8 = 0;
    let mut v7: int8 = 0;
    let mut v8: int8 = 0;
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize);
    if v2 != CrocomireSpikeWallPieces_Tab1[v1 as usize] {
        v2 = (v2 as ::core::ffi::c_int
            + CrocomireSpikeWallPieces_Tab2[v1 as usize] as ::core::ffi::c_int)
            as uint16;
        if v2 >= CrocomireSpikeWallPieces_Tab1[v1 as usize] {
            v2 = CrocomireSpikeWallPieces_Tab1[v1 as usize];
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) = v2;
    v3 = *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1) as int8;
    let mut v4: bool = u8::overflowing_add(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize),
        v3 as uint8,
    )
    .1;
    let ref mut fresh138 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize);
    *fresh138 = (*fresh138 as ::core::ffi::c_int + v3 as ::core::ffi::c_int) as uint8;
    let mut v5: uint8 = (v4 as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize)
            .offset(1) as ::core::ffi::c_int) as uint8;
    if ((v5 as ::core::ffi::c_int - CrocomireSpikeWallPieces_Tab3[k as usize] as ::core::ffi::c_int)
        as int8)
        >= 0i8
    {
        v5 = CrocomireSpikeWallPieces_Tab3[k as usize] as uint8;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1) = v5;
    v6 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as int8;
    v4 = u8::overflowing_add(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize)
            .offset(1),
        v6 as uint8,
    )
    .1;
    let ref mut fresh139 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1);
    *fresh139 = (*fresh139 as ::core::ffi::c_int + v6 as ::core::ffi::c_int) as uint8;
    v7 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1) as int8;
    let v9: bool = v4;
    v4 = u8::overflowing_add(v4 as uint8, v7 as uint8).1;
    v8 = (v9 as ::core::ffi::c_int + v7 as ::core::ffi::c_int) as int8;
    v4 = (v4 as ::core::ffi::c_int
        | u8::overflowing_add(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize),
            v8 as uint8,
        )
        .1 as ::core::ffi::c_int)
        != 0;
    let ref mut fresh140 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize);
    *fresh140 = (*fresh140 as ::core::ffi::c_int + v8 as ::core::ffi::c_int) as uint8;
    let ref mut fresh141 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1);
    *fresh141 = (*fresh141 as ::core::ffi::c_int + v4 as ::core::ffi::c_int) as uint8;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        0x3000 as uint32,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
            as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                .offset(v1 as isize) as uint32,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
        >= 0xa8u16
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
        if (k as ::core::ffi::c_int & 2 as ::core::ffi::c_int) == 0i32 {
            QueueSfx2_Max6(0x29 as uint16, game);
        }
        let v12: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        game.eproj_spawn_pt = Point16U {
            x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v12 as isize),
            y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v12 as isize),
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as uint16,
            0x15 as uint16,
            game,
        );
        QueueSfx2_Max6(0x25 as uint16, game);
    }
}
pub unsafe fn EprojInstr_9270(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize),
    };
    SpawnEnemyDrops(addr_kEnemyDef_DDBF as uint16, k, 0 as uint16, game);
    epjp
}
unsafe fn EprojInit_CrocomireBridgeCrumbling(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        ram_word!(game, cinematic_var8);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        187 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        ((ram_word!(game, random_number) as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int)
            + 64 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        1024 as uint16;
}
pub unsafe fn EprojPreInstr_CrocomireBridgeCrumbling(k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                + 24 as ::core::ffi::c_int
                & 0x3fff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn MoveEprojWithVelocity(k: uint16, game: &mut Game) -> uint16 {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize);
    let carry: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1a27)
        as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1) as ::core::ffi::c_int
        + (v2 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1) = carry as uint8;
    let ref mut fresh7 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize);
    *fresh7 = (*fresh7 as ::core::ffi::c_int
        + ((v2 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
            + (carry >> 8 as ::core::ffi::c_int))) as uint16;
    MoveEprojWithVelocityY(k, game)
}
pub unsafe fn MoveEprojWithVelocityY(k: uint16, game: &mut Game) -> uint16 {
    let v1 = usize::from(k >> 1);
    let v2: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize);
    let (fraction, carry) = (*((&raw mut game.g_ram as *mut uint8).offset(0x1a6f))
        .offset(k as isize)
        .offset(1))
    .overflowing_add(v2 as u8);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f))
        .offset(k as isize)
        .offset(1) = fraction;
    let result = game
        .enemy_projectile_y_pos(k)
        .wrapping_add_signed(((v2 as i16) >> 8) + carry as i16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        result;
    result
}
pub unsafe fn MoveEprojWithVelocityX(k: uint16, game: &mut Game) -> uint16 {
    let v1 = usize::from(k >> 1);
    let v2: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize);
    let (fraction, carry) = (*((&raw mut game.g_ram as *mut uint8).offset(0x1a27))
        .offset(k as isize)
        .offset(1))
    .overflowing_add(v2 as u8);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27))
        .offset(k as isize)
        .offset(1) = fraction;
    let result = game
        .enemy_projectile_x_pos(k)
        .wrapping_add_signed(((v2 as i16) >> 8) + carry as i16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        result;
    result
}
pub unsafe fn SetAreaDependentEprojPropertiesEx(k: uint16, j: uint16, game: &mut Game) {
    let mut v2: uint16 = 0;
    let p: *mut uint16 = RomPtr_86(k, game) as *mut uint16;
    if game.area_index() == 2 {
        v2 = *p.offset(1);
    } else if game.area_index() == 5 {
        v2 = *p.offset(2);
    } else {
        v2 = *p.offset(0);
    }
    EprojStateMut(&mut game.g_ram).set_properties(usize::from(j >> 1), v2);
}
static kEprojInit_9634_Xvel: [int16; 4] = [
    -(0x200 as ::core::ffi::c_int) as int16,
    -(0x1f0 as ::core::ffi::c_int) as int16,
    -(0x1bc as ::core::ffi::c_int) as int16,
    -(0x16a as ::core::ffi::c_int) as int16,
];
static kEprojInit_9634_Yvel: [int16; 4] =
    [0 as int16, 0x88 as int16, 0xfc as int16, 0x16a as int16];
unsafe fn EprojInit_9634(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
            as ::core::ffi::c_int
            - 29 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).y_pos
            as ::core::ffi::c_int
            - 35 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        2560 as uint16;
    let v2: ::core::ffi::c_int =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
            .parameter_1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        kEprojInit_9634_Xvel[v2 as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        kEprojInit_9634_Yvel[v2 as usize] as uint16;
}
pub unsafe fn EprojPreInstr_9634(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize)
        >= 8u16
    {
        MoveEprojWithVelocity(k, game);
        if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16)
                .offset(v1 as isize) = addr_off_869574 as uint16;
            let ref mut fresh136 = *((&raw mut game.g_ram as *mut uint8).offset(0x1aff)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh136 = (*fresh136).wrapping_add(1);
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16)
                .offset(v1 as isize) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16)
                .offset(v1 as isize) = 2560 as uint16;
            QueueSfx2_Max6(0x2b as uint16, game);
        }
    } else {
        let ref mut fresh137 = *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset(v1 as isize);
        *fresh137 = (*fresh137).wrapping_add(1);
    };
}
unsafe fn EprojInit_9642_RidleysFireball(j: uint16, game: &mut Game) {
    let slot = usize::from(j >> 1);
    let parameter = EprojState(&game.g_ram).slot_word(0x1995, 0);
    EprojStateMut(&mut game.g_ram).write(0x1b23, slot, parameter);
    let v2: int16 = (if EprojState(&game.g_ram).init_param_1() != 0u16 {
        25 as ::core::ffi::c_int
    } else {
        -(25 as ::core::ffi::c_int)
    }) as int16;
    let x = (game.enemy_state(0).x_pos()).wrapping_add((v2) as u16);
    EprojStateMut(&mut game.g_ram).write(0x1a4b, slot, x);
    let y = (game.enemy_state(0).y_pos()).wrapping_sub(43u16);
    EprojStateMut(&mut game.g_ram).write(0x1a93, slot, y);
    EprojStateMut(&mut game.g_ram).write(0x19bb, slot, 2560);
    let x_velocity = EprojState(&game.g_ram)
        .slot_word(0x7800 + core::mem::offset_of!(Ram7800_Default, var_19), 0);
    EprojStateMut(&mut game.g_ram).write(0x1ab7, slot, x_velocity);
    let y_velocity = EprojState(&game.g_ram)
        .slot_word(0x7800 + core::mem::offset_of!(Ram7800_Default, var_1A), 0);
    EprojStateMut(&mut game.g_ram).write(0x1adb, slot, y_velocity);
    SetAreaDependentEprojPropertiesEx(addr_kRidleysFireball_Tab0 as uint16, j, game);
}
pub unsafe fn SetAreaDependentEprojProperties(j: uint16, game: &mut Game) {
    SetAreaDependentEprojPropertiesEx(addr_kRidleysFireball_Tab0 as uint16, j, game);
}
fn SetEprojSpawnPoint(k: uint16, game: &mut Game) {
    let x = game.enemy_projectile_x_pos(k);
    let y = game.enemy_projectile_y_pos(k);
    game.eproj_spawn_pt = Point16U { x, y };
}
pub unsafe fn EprojPreInstr_9642_RidleysFireball(k: uint16, game: &mut Game) {
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
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    EprojStateMut(&mut game.g_ram).set_id(v2 as usize, 0);
    if EprojState(&game.g_ram).slot_word(0x1b23, v2 as usize) == 0 {
        SetEprojSpawnPoint(k, game);
        SpawnEprojWithRoomGfx(v1, 3 as uint16, game);
        QueueSfx2_Max6(0x2b as uint16, game);
    }
}
pub unsafe fn EprojInstr_DisableCollisionsWithSamus(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let ref mut fresh32 = *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh32 = (*fresh32 as ::core::ffi::c_int | 0x2000 as ::core::ffi::c_int) as uint16;
    epjp
}
fn EprojInit_9660_FireballExplosion(j: uint16, game: &mut Game) {
    let slot = usize::from(j >> 1);
    EprojStateMut(&mut game.g_ram).write(0x19bb, slot, 2560);
    let x = game.eproj_spawn_pt.x;
    EprojStateMut(&mut game.g_ram).write(0x1a4b, slot, x);
    let y = game.eproj_spawn_pt.y;
    EprojStateMut(&mut game.g_ram).write(0x1a93, slot, y);
    let parameter = EprojState(&game.g_ram).init_param_1();
    EprojStateMut(&mut game.g_ram).write(0x1aff, slot, parameter);
    EprojStateMut(&mut game.g_ram).write(0x1b23, slot, 0);
    EprojStateMut(&mut game.g_ram).write(0x1ab7, slot, 0);
    EprojStateMut(&mut game.g_ram).write(0x1adb, slot, 0);
}
unsafe fn EprojInit_Common(j: uint16, k: uint16, a: uint16, game: &mut Game) {
    let slot = usize::from(j >> 1);
    EprojStateMut(&mut game.g_ram).write(0x1b23, slot, a);
    let x = game.enemy_projectile_x_pos(k);
    EprojStateMut(&mut game.g_ram).write(0x1a4b, slot, x);
    let y = game.enemy_projectile_y_pos(k);
    EprojStateMut(&mut game.g_ram).write(0x1a93, slot, y);
    let e = EprojState(&game.g_ram).slot_word(0x1aff, usize::from(k >> 1));
    EprojStateMut(&mut game.g_ram).write(0x1aff, slot, e);
    EprojStateMut(&mut game.g_ram).write(0x19bb, slot, 2560);
    SetAreaDependentEprojProperties(j, game);
}
unsafe fn EprojInit_FireballChild(
    j: uint16,
    x: uint16,
    y: uint16,
    handler: uint16,
    game: &mut Game,
) {
    let parent = EprojState(&game.g_ram).init_param_1();
    let slot = usize::from(j >> 1);
    EprojStateMut(&mut game.g_ram).write(0x1ab7, slot, x);
    EprojStateMut(&mut game.g_ram).write(0x1adb, slot, y);
    EprojInit_Common(j, parent, handler, game);
}
unsafe fn EprojInit_9688(j: uint16, game: &mut Game) {
    EprojInit_FireballChild(j, 0, (-3584i16) as u16, 0x9688, game);
}
unsafe fn EprojInit_9696(j: uint16, game: &mut Game) {
    EprojInit_FireballChild(j, 0, 3584, 0x9696, game);
}
unsafe fn EprojInit_966C(j: uint16, game: &mut Game) {
    EprojInit_FireballChild(j, 3584, 0, 0x966c, game);
}
unsafe fn EprojInit_967A(j: uint16, game: &mut Game) {
    EprojInit_FireballChild(j, (-3584i16) as u16, 0, 0x967a, game);
}
pub unsafe fn EprojPreInstr_966C(k: uint16, game: &mut Game) {
    MoveEprojWithVelocityX(k, game);
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        let slot = usize::from(k >> 1);
        EprojStateMut(&mut game.g_ram).set_instruction(slot, addr_off_869574 as uint16);
        EprojStateMut(&mut game.g_ram).set_instruction_timer(slot, 1);
    }
}
pub unsafe fn EprojPreInstr_9688(k: uint16, game: &mut Game) {
    MoveEprojWithVelocityY(k, game);
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let slot = usize::from(k >> 1);
        EprojStateMut(&mut game.g_ram).set_instruction(slot, addr_off_869574 as uint16);
        EprojStateMut(&mut game.g_ram).set_instruction_timer(slot, 1);
    }
}
pub unsafe fn EprojPreInstr_96A4(k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojPreInstr_96C0(k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojPreInstr_96CE(k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojInstr_95BA(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    EprojStateMut(&mut game.g_ram).write(0x1995, 0, 0);
    SpawnEprojWithRoomGfx(addr_stru_86966C as uint16, k, game);
    EprojStateMut(&mut game.g_ram).write(0x1995, 0, 0);
    SpawnEprojWithRoomGfx(addr_stru_86967A as uint16, k, game);
    epjp
}
pub unsafe fn EprojInstr_95ED(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    EprojStateMut(&mut game.g_ram).write(0x1995, 0, 0);
    SpawnEprojWithRoomGfx(addr_stru_869688 as uint16, k, game);
    EprojStateMut(&mut game.g_ram).write(0x1995, 0, 0);
    SpawnEprojWithRoomGfx(addr_stru_869696 as uint16, k, game);
    epjp
}
pub unsafe fn EprojInstr_9620(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    let count = EprojStateMut(&mut game.g_ram).decrement_byte(0x1aff, usize::from(k));
    if count as i8 >= 0 {
        let handler = EprojState(&game.g_ram).slot_word(0x1b23, usize::from(k >> 1));
        SpawnEprojWithRoomGfx(handler, k, game);
    }
    epjp
}
fn EprojInit_9734_CeresFallingDebris(j: uint16, game: &mut Game) {
    let slot = usize::from(j >> 1);
    EprojStateMut(&mut game.g_ram).write(0x1aff, slot, 0);
    EprojStateMut(&mut game.g_ram).write(0x1b23, slot, 0);
    EprojStateMut(&mut game.g_ram).write(0x1ab7, slot, 0);
    EprojStateMut(&mut game.g_ram).write(0x19bb, slot, 3584);
    let x = EprojState(&game.g_ram).init_param_1();
    EprojStateMut(&mut game.g_ram).write(0x1a4b, slot, x);
    EprojStateMut(&mut game.g_ram).write(0x1a93, slot, 42);
    EprojStateMut(&mut game.g_ram).write(0x1adb, slot, 16);
}
pub unsafe fn EprojPreInstr_9734_CeresFallingDebris(k: uint16, game: &mut Game) {
    let slot = usize::from(k >> 1);
    EprojStateMut(&mut game.g_ram).add_word(0x1adb, slot, 16);
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        EprojStateMut(&mut game.g_ram).set_id(slot, 0);
        SetEprojSpawnPoint(k, game);
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, 9 as uint16, game);
        QueueSfx2_Max6(0x6d as uint16, game);
    }
}
pub unsafe fn EprojInstr_980E(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize),
    };
    SpawnEnemyDrops(addr_kEnemyDef_E4FF as uint16, k, 0 as uint16, game);
    epjp
}
static byte_8698B4: [uint8; 16] = [
    0x10 as uint8,
    0x20 as uint8,
    0x30 as uint8,
    0x40 as uint8,
    0x50 as uint8,
    0x60 as uint8,
    0x70 as uint8,
    0x80 as uint8,
    0xf0 as uint8,
    0xe0 as uint8,
    0xd0 as uint8,
    0xc0 as uint8,
    0xb0 as uint8,
    0xa0 as uint8,
    0x90 as uint8,
    0x80 as uint8,
];
static byte_8698F7: [uint8; 9] = [
    0x30 as uint8,
    0x44 as uint8,
    0x58 as uint8,
    0x6c as uint8,
    0x80 as uint8,
    0x94 as uint8,
    0xa8 as uint8,
    0xbc as uint8,
    0xd0 as uint8,
];
static byte_869979: [uint8; 8] = [
    0 as uint8,
    0x20 as uint8,
    0x40 as uint8,
    0x60 as uint8,
    0x80 as uint8,
    0xa0 as uint8,
    0xc0 as uint8,
    0xe0 as uint8,
];
unsafe fn EprojInit_PhantoonDestroyableFireballs(j: uint16, game: &mut Game) {
    let mut v1: int8 = 0;
    v1 = ((ram_word!(game, cinematic_var8) as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
        as uint16 as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as int8;
    match v1 as ::core::ffi::c_int {
        0 => {
            let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16)
                .offset(v2 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16)
                .offset(v2 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v2 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v2 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v2 as isize) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                    .x_pos;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v2 as isize) =
                ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                    .y_pos as ::core::ffi::c_int
                    + 32 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16)
                .offset(v2 as isize) = addr_off_8697B4 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16)
                .offset(v2 as isize) = (*((&raw mut game.g_ram as *mut uint8).offset(0x1bd7)
                as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int
                & 0xfff as ::core::ffi::c_int
                | 0x2000 as ::core::ffi::c_int) as uint16;
        }
        2 => {
            let v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16)
                .offset(v3 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16)
                .offset(v3 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v3 as isize) = 0 as uint16;
            let v4: uint16 = ram_word!(game, cinematic_var8) as uint8 as uint16;
            if ((ram_word!(game, cinematic_var8) as uint8 as ::core::ffi::c_int
                - 8 as ::core::ffi::c_int) as int16)
                >= 0i16
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                    .offset(v3 as isize) = -(2 as ::core::ffi::c_int) as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                    .offset(v3 as isize) = 2 as uint16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                .offset(v3 as isize) = byte_8698B4[v4 as usize] as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v3 as isize) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                    .x_pos;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v3 as isize) =
                ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                    .y_pos as ::core::ffi::c_int
                    + 32 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16)
                .offset(v3 as isize) =
                (0x869a45 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        4 => {
            let v5: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16)
                .offset(v5 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16)
                .offset(v5 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v5 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v5 as isize) = ((ram_word!(game, cinematic_var8) as ::core::ffi::c_int
                & 0xf0 as ::core::ffi::c_int)
                >> 1 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v5 as isize) =
                byte_8698F7[(ram_word!(game, cinematic_var8) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as usize] as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v5 as isize) = 40 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16)
                .offset(v5 as isize) =
                (0x869a94 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        6 => {
            let v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16)
                .offset(v7 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16)
                .offset(v7 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v7 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v7 as isize) = 128 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                .offset(v7 as isize) =
                byte_869979[ram_word!(game, cinematic_var8) as uint8 as usize] as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v7 as isize) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                    .x_pos;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v7 as isize) =
                ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                    .y_pos as ::core::ffi::c_int
                    + 16 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16)
                .offset(v7 as isize) =
                (0x869ada as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        _ => {
            Unreachable(game);
        }
    };
}
unsafe fn EprojInit_PhantoonStartingFireballs(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    let v2: uint16 = byte_869979[ram_word!(game, cinematic_var8) as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) = v2;
    let pt: Point16U = Eproj_PhantomFireballs_Func1(v2, 0x30 as uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (pt.x as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .x_pos as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (pt.y as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .y_pos as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojPreInstr_PhantoonStartingFireballs(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh133 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize);
    *fresh133 = (*fresh133 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16).offset(v1 as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0xfff as ::core::ffi::c_int
                | 0x8000 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v1 as isize) =
            (0x8699bf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
            addr_word_86976C as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
            8 as uint16;
        let ref mut fresh134 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
            .offset(v1 as isize);
        *fresh134 = (*fresh134 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn EprojPreInstr_PhantoonStartingFireballs2(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: bool = *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
        .offset(v1 as isize)
        == 1u16;
    let ref mut fresh131 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize);
    *fresh131 = (*fresh131).wrapping_sub(1);
    let v3: bool = (*fresh131 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if v2 || v3 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v1 as isize) =
            (0x869a01 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
            addr_word_869772 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        let ref mut fresh132 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
            .offset(v1 as isize);
        *fresh132 = (*fresh132 as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
            -(768 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
        if (ram_word!(game, nmi_frame_counter_word) as ::core::ffi::c_int & 1 as ::core::ffi::c_int)
            != 0i32
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) = -(128 as ::core::ffi::c_int) as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) = 128 as uint16;
        }
    }
}
static word_869A3E: [uint16; 3] = [0xfd00 as uint16, 0xfe00 as uint16, 0xff00 as uint16];
pub unsafe fn EprojPreInstr_PhantoonStartingFireballs3(k: uint16, game: &mut Game) {
    let current_block: u64;
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh129 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize);
    *fresh129 = (*fresh129 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        let v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
            v2;
        if v2 as ::core::ffi::c_int - 3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
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
    if current_block == 220575731145207134 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
            addr_word_869782 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v1 as isize) =
            (0x869a44 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn EprojPreInstr_PhantoonDestroyableFireballs(k: uint16, game: &mut Game) {
    let mut v4: int16 = 0;
    let mut v5: uint16 = 0;
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh128 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize);
    *fresh128 = (*fresh128 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    let v2: uint16 = (*(((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(0) as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int) as uint8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) = v2;
    let pt: Point16U = Eproj_PhantomFireballs_Func1(
        v2,
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize),
    );
    let mut v3: bool = ((pt.x as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
            as ::core::ffi::c_int) as int16)
        < 0i16;
    v4 = (pt.x as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
            as ::core::ffi::c_int) as int16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (pt.x as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .x_pos as ::core::ffi::c_int) as uint16;
    if v3
        || v4 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        || {
            v3 = ((pt.y as ::core::ffi::c_int
                + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                    .y_pos as ::core::ffi::c_int
                + 16 as ::core::ffi::c_int) as int16)
                < 0i16;
            v5 = (pt.y as ::core::ffi::c_int
                + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                    .y_pos as ::core::ffi::c_int
                + 16 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v1 as isize) = v5;
            v3 as ::core::ffi::c_int != 0
        }
        || v5 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
            addr_off_8697F8 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    }
}
pub unsafe fn EprojPreInstr_PhantoonDestroyableFireballs_2(k: uint16, game: &mut Game) {
    let mut v2: bool = false;
    let mut v3: bool = false;
    let current_block: u64;
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) == 0
    {
        current_block = 8066597768905169940;
    } else {
        v2 = false;
        v3 = false;
        v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
            .offset(v1 as isize)
            == 1u16;
        let ref mut fresh125 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
            .offset(v1 as isize);
        *fresh125 = (*fresh125).wrapping_sub(1);
        v3 = (*fresh125 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
        if v2 || v3 {
            QueueSfx3_Max6(0x1d as uint16, game);
            current_block = 8066597768905169940;
        } else {
            current_block = 2868539653012386629;
        }
    }
    if current_block == 8066597768905169940 {
        let ref mut fresh126 = *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset(v1 as isize);
        *fresh126 = (*fresh126 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
        if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16)
                .offset(v1 as isize) = addr_word_8697AC as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16)
                .offset(v1 as isize) = 1 as uint16;
            let ref mut fresh127 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a93)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh127 = (*fresh127 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16)
                .offset(v1 as isize) =
                (0x869a44 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            QueueSfx3_Max6(0x1d as uint16, game);
        }
    };
}
pub unsafe fn EprojPreInstr_PhantoonDestroyableFireballs_3(k: uint16, game: &mut Game) {
    let mut v4: int16 = 0;
    let mut v5: uint16 = 0;
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh123 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize);
    *fresh123 = (*fresh123 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    let v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int) as uint8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) = v2;
    let pt: Point16U = Eproj_PhantomFireballs_Func1(
        v2,
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize),
    );
    let mut v3: bool = ((pt.x as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
            as ::core::ffi::c_int) as int16)
        < 0i16;
    v4 = (pt.x as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
            as ::core::ffi::c_int) as int16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (pt.x as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .x_pos as ::core::ffi::c_int) as uint16;
    if v3
        || v4 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        || {
            v3 = ((pt.y as ::core::ffi::c_int
                + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                    .y_pos as ::core::ffi::c_int
                + 16 as ::core::ffi::c_int) as int16)
                < 0i16;
            v5 = (pt.y as ::core::ffi::c_int
                + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                    .y_pos as ::core::ffi::c_int
                + 16 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v1 as isize) = v5;
            v3 as ::core::ffi::c_int != 0
        }
        || v5 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
            addr_off_8697F8 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    }
}
pub unsafe fn EprojPreInstr_PhantoonStartingFireballsB(k: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).ai_var_B
        != 0
    {
        let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v1 as isize) =
            (0x869b41 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
            180 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
            48 as uint16;
    }
}
pub unsafe fn EprojPreInstr_PhantoonStartingFireballsB_2(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize);
    if v2 != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
            (v2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    } else if ((ram_word!(game, nmi_frame_counter_word) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int)
        != 0i32)
    {
        let v3: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
            v3;
        if (v3 == 0) {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v1 as isize) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                    .x_pos;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v1 as isize) =
                ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                    .y_pos as ::core::ffi::c_int
                    + 16 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16)
                .offset(v1 as isize) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16)
                .offset(v1 as isize) = addr_off_8697F8 as uint16;
            return;
        }
    }
    let v4: uint8 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        v4 as uint16;
    let pt: Point16U = Eproj_PhantomFireballs_Func1(
        v4 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize),
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (pt.x as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .x_pos as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (pt.y as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
                .y_pos as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Eproj_PhantomFireballs_Func1(j: uint16, a: uint16) -> Point16U {
    let mut v3: int16 = 0;
    let mut v2: uint16 = 0;
    let mut v4: uint16 = 0;
    let r24: uint16 = a;
    let r26: uint16 = j;
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
    let r20: uint16 = v2;
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
    let r22: uint16 = v4;
    Point16U { x: r20, y: r22 }
}
pub unsafe fn Eproj_PhantomFireballs_Func2(k: uint16, r24: uint16) -> uint16 {
    let r18: uint16 = (Mult8x8(
        *((&raw const kSinCosTable8bit_Sext as *const int16).offset(64) as *const int16
            as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize),
        r24 as uint8,
    ) as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as uint16;
    (r18 as ::core::ffi::c_int
        + Mult8x8(
            *((&raw const kSinCosTable8bit_Sext as *const int16).offset(64) as *const int16
                as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize)
                .offset(1),
            r24 as uint8,
        ) as ::core::ffi::c_int) as uint16
}
unsafe fn EprojInit_RocksKraidSpits(j: uint16, game: &mut Game) {
    let enemy_index = ram_word!(game, cur_enemy_index);
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        (game.enemy_state(enemy_index).x_pos()).wrapping_add(16u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        (game.enemy_state(enemy_index).y_pos()).wrapping_sub(96u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        ram_word!(game, cinematic_var8);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        -(1024 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v2 as isize) =
        1536 as uint16;
}
unsafe fn EprojInit_RocksFallingKraidCeiling(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        ram_word!(game, cinematic_var8);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        312 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        ((ram_word!(game, random_number) as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int)
            as u16)
            .wrapping_add(64u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        1536 as uint16;
}
unsafe fn EprojInit_RocksWhenKraidRises(j: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    v1 = ((ram_word!(game, random_number)) as i16) & (0x3fi16);
    if (ram_word!(game, random_number) as ::core::ffi::c_int & 1 as ::core::ffi::c_int) == 0i32 {
        v1 = !(v1 as ::core::ffi::c_int) as int16;
    }
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) = ({
        let enemy_index = ram_word!(game, cur_enemy_index);
        game.enemy_state(enemy_index).x_pos()
    })
    .wrapping_add((v1) as u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        432 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        ram_word!(game, cinematic_var8);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        -(1280 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v2 as isize) =
        1536 as uint16;
    QueueSfx3_Max6(0x1e as uint16, game);
}
pub unsafe fn EprojPreInstr_KraidRocks(k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
    {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    } else {
        let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v2: uint16 = EprojStateMut(&mut game.g_ram).add_word(0x1ab7, v1 as usize, 8);
        if v2 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        {
            v2 = -(256 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
            v2;
        EprojStateMut(&mut game.g_ram).add_word(0x1adb, v1 as usize, 64);
    };
}
pub unsafe fn EprojPreInstr_RocksFallingKraidCeiling(k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                + 24 as ::core::ffi::c_int) as u16)
                & (0x3fffu16);
    };
}
pub unsafe fn sub_869DA5(k: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
}
unsafe fn EprojInit_MiniKraidSpit(j: uint16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let extra_index = ram_word!(game, cur_enemy_index);
    let v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v3 as isize) =
        (game.mini_kraid_spit_state(extra_index).x_offset() as ::core::ffi::c_int
            + game.enemy_state(enemy_index).x_pos() as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v3 as isize) =
        (game.enemy_state(enemy_index).y_pos() as ::core::ffi::c_int - 16 as ::core::ffi::c_int)
            as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v3 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v3 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v3 as isize) =
        game.mini_kraid_spit_state(extra_index).x_velocity();
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v3 as isize) =
        game.mini_kraid_spit_state(extra_index).y_velocity();
}
pub unsafe fn EprojPreInit_MiniKraidSpit(k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
    {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    } else {
        let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        v2 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset(v1 as isize)) as i16)
            .wrapping_add(64i16);
        if v2 >= 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - 960 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            v2 = 1024 as int16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
            v2 as uint16;
    };
}
static kEprojInit_MiniKraidSpikes_Tab0: [int16; 3] = [
    -(2 as ::core::ffi::c_int) as int16,
    12 as int16,
    24 as int16,
];
unsafe fn EprojInit_MiniKraidSpikes(j: uint16, a: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) = a;
    let r18: uint16 = kEprojInit_MiniKraidSpikes_Tab0[((*gExtraEnemyRam7800(
        game.cur_enemy_index() as ::core::ffi::c_int,
        game,
    ))
    .kraid
    .kraid_healths_8ths[0 as ::core::ffi::c_int as usize]
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int) as usize] as uint16;
    let enemy_index = game.cur_enemy_index();
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        game.enemy_state(enemy_index).x_pos();
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        (r18).wrapping_add(game.enemy_state(enemy_index).y_pos());
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
}
unsafe fn EprojInit_MiniKraidSpikesLeft(j: uint16, game: &mut Game) {
    EprojInit_MiniKraidSpikes(j, -(0x200 as ::core::ffi::c_int) as uint16, game);
}
unsafe fn EprojInit_MiniKraidSpikesRight(j: uint16, game: &mut Game) {
    EprojInit_MiniKraidSpikes(j, 0x200 as uint16, game);
}
pub unsafe fn EprojPreInstr_MiniKraidSpikes(k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    }
}
unsafe fn EprojInit_WalkingLavaSeahorseFireball(j: uint16, game: &mut Game) {
    static word_869EF9: [int16; 3] = [
        -(0x100 as ::core::ffi::c_int) as int16,
        0 as int16,
        0x100 as int16,
    ];
    let v1: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        ((*v1).y_pos as ::core::ffi::c_int - 12 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        -(1024 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        ((*v1).x_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
    if ((*v1).ai_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
            1024 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
            ((*v1).x_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        word_869EF9[(ram_word!(game, cinematic_var8) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
}
pub unsafe fn EprojPreInstr_WalkingLavaSeahorseFireball(k: uint16, game: &mut Game) {
    let mut v2: uint16 = 0;
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    } else {
        let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            == 0i32
        {
            v2 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - 64 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) = v2;
            if v2 as ::core::ffi::c_int - 512 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v2 = 512 as uint16;
            }
        } else {
            v2 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + 64 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) = v2;
            if v2 as ::core::ffi::c_int + 512 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v2 = -(512 as ::core::ffi::c_int) as uint16;
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
            v2;
    };
}
unsafe fn EprojInit_PirateMotherBrainLaser(j: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
        (if game.eproj_spawn_r22 != 0u16 {
            addr_word_869F7D as ::core::ffi::c_int
        } else {
            addr_word_869F41 as ::core::ffi::c_int
        }) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v2 as isize) =
        (0x86a05b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        game.eproj_spawn_pt.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        game.eproj_spawn_pt.y;
    let enemy_index = game.cur_enemy_index();
    *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16).offset(v2 as isize) =
        (*(RomPtr_A0(game.enemy_state(enemy_index).enemy_ptr() as uint16_t, game) as *mut uint16)
            .offset(3) as ::core::ffi::c_int
            | 0x1000 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) =
        game.enemy_state(enemy_index).parameter_1();
    QueueSfx2_Max6(0x67 as uint16, game);
}
pub unsafe fn EprojInstr_SetPreInstrAndRun(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(epjp as *mut uint16);
    epjp
}
pub unsafe fn EprojPreInstr_PirateMotherBrainLaser_MoveLeft(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    EprojStateMut(&mut game.g_ram).add_word(0x1a4b, v1 as usize, (-2i16) as u16);
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        let ref mut fresh22 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
            .offset(v1 as isize);
        *fresh22 = (*fresh22 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    }
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        EprojStateMut(&mut game.g_ram).set_id(v1 as usize, 0);
    }
}
pub unsafe fn EprojPreInstr_PirateMotherBrainLaser_MoveRight(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    EprojStateMut(&mut game.g_ram).add_word(0x1a4b, v1 as usize, 2);
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        let ref mut fresh24 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
            .offset(v1 as isize);
        *fresh24 = (*fresh24 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        EprojStateMut(&mut game.g_ram).set_id(v1 as usize, 0);
    }
}
unsafe fn EprojInit_PirateClaw(j: uint16, game: &mut Game) {
    let r: Rect16U = game.eproj_spawn_rect;
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (r.y as ::core::ffi::c_int + r.h as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (r.x as ::core::ffi::c_int + r.w as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
        (if ram_word!(game, cinematic_var8) != 0u16 {
            addr_off_869FE1 as ::core::ffi::c_int
        } else {
            addr_off_869FB9 as ::core::ffi::c_int
        }) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v1 as isize) =
        (0x86a05b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        2048 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
        1 as uint16;
}
pub unsafe fn EprojPreInstr_PirateClawThrownLeft(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) != 0
    {
        let ref mut fresh118 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
            .offset(v1 as isize);
        *fresh118 = (*fresh118 as ::core::ffi::c_int
            - *(((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ) as ::core::ffi::c_int) as uint16;
        let v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - 32 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
            v2;
        if v2 == 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
        }
    } else {
        let ref mut fresh119 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
            .offset(v1 as isize);
        *fresh119 = (*fresh119 as ::core::ffi::c_int
            + *(((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ) as ::core::ffi::c_int) as uint16;
        let ref mut fresh120 = *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset(v1 as isize);
        *fresh120 = (*fresh120 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    }
    let ref mut fresh121 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize);
    *fresh121 = (*fresh121).wrapping_add(1);
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
    }
}
pub unsafe fn EprojPreInstr_PirateClawThrownRight(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) != 0
    {
        let ref mut fresh114 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
            .offset(v1 as isize);
        *fresh114 = (*fresh114 as ::core::ffi::c_int
            + *(((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ) as ::core::ffi::c_int) as uint16;
        let v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - 32 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
            v2;
        if v2 == 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
        }
    } else {
        let ref mut fresh115 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
            .offset(v1 as isize);
        *fresh115 = (*fresh115 as ::core::ffi::c_int
            - *(((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ) as ::core::ffi::c_int) as uint16;
        let ref mut fresh116 = *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset(v1 as isize);
        *fresh116 = (*fresh116 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    }
    let ref mut fresh117 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize);
    *fresh117 = (*fresh117).wrapping_add(1);
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
    }
}
static word_86A2D6: [int16; 6] = [
    64 as int16,
    72 as int16,
    80 as int16,
    -(64 as ::core::ffi::c_int) as int16,
    -(72 as ::core::ffi::c_int) as int16,
    -(80 as ::core::ffi::c_int) as int16,
];
unsafe fn EprojInit_A379(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    let v2: uint16 = ram_word!(game, cinematic_var8);
    let v3: ::core::ffi::c_int =
        ram_word!(game, cinematic_var8) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (word_86A2D6[v3 as usize] as ::core::ffi::c_int
            + ram_word!(game, samus_x_pos) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (ram_word!(game, samus_y_pos) as ::core::ffi::c_int + 80 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86a2e2)) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) = v2;
}
fn EprojInit_CeresElevatorPad(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let eproj_y_pos: uint16 = game.samus_y_pos().wrapping_add(28);
    EprojStateMut(&mut game.g_ram).write(0x1a93, v1 as usize, eproj_y_pos);
    EprojStateMut(&mut game.g_ram).write(0x1aff, v1 as usize, 60 as uint16);
    sub_86A301(j, game);
}
pub fn sub_86A301(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    EprojStateMut(&mut game.g_ram).write(0x1a27, v1 as usize, 0 as uint16);
    EprojStateMut(&mut game.g_ram).write(0x1a6f, v1 as usize, 0 as uint16);
    EprojStateMut(&mut game.g_ram).write(0x1ab7, v1 as usize, 0 as uint16);
    EprojStateMut(&mut game.g_ram).write(0x1adb, v1 as usize, 0 as uint16);
    EprojStateMut(&mut game.g_ram).write(0x19bb, v1 as usize, 0 as uint16);
    let eproj_x_pos: uint16 = game.samus_x_pos();
    EprojStateMut(&mut game.g_ram).write(0x1a4b, v1 as usize, eproj_x_pos);
}
fn EprojInit_CeresElevatorPlatform(j: uint16, game: &mut Game) {
    let eproj_word_index = (j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize;
    EprojStateMut(&mut game.g_ram).write(0x1a93, eproj_word_index, 97 as uint16);
    sub_86A301(j, game);
}
pub unsafe fn EprojPreInstr_CeresElevatorPad(k: uint16, game: &mut Game) {
    let mut v2: bool = false;
    let mut v3: bool = false;
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) == 0
        || {
            v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                .offset(v1 as isize)
                == 1u16;
            v3 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                .offset(v1 as isize)) as i16)
                .wrapping_sub(1i16)
                < 0i16;
            let ref mut fresh112 = *((&raw mut game.g_ram as *mut uint8).offset(0x1aff)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh112 = (*fresh112).wrapping_sub(1);
            v2 as ::core::ffi::c_int != 0
        }
        || v3
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
            game.samus_y_pos().wrapping_add(28);
        let fresh113 = EprojStateMut(&mut game.g_ram).add_word(0xafa, 0, 1);
        if fresh113 as ::core::ffi::c_int - 73 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0
        {
            set_ram_word!(game.samus_y_pos = 72 as uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16)
                .offset(v1 as isize) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16)
                .offset(v1 as isize) = addr_off_86A28B as uint16;
            CallSomeSamusCode(0xe as uint16, game);
        }
    }
}
pub unsafe fn EprojPreInstr_CeresElevatorPlatform(k: uint16, game: &mut Game) {
    if game.samus_y_pos() == 72 {
        let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
            addr_off_86A28B as uint16;
    }
}
pub unsafe fn EprojPreInstr_PrePhantomRoom(j: uint16, game: &mut Game) {
    set_ram_word!(game.bg2_y_scroll = 0 as uint16);
}
pub unsafe fn EprojInstr_A3BE(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize);
    epjp
}
pub unsafe fn EprojInstr_GotoWithProbability25(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    if (NextRandom(game) as ::core::ffi::c_int & 0xc000 as ::core::ffi::c_int) == 0xc000i32 {
        *(epjp as *mut uint16) as uintptr_t as *const uint8
    } else {
        epjp.offset(2)
    }
}
unsafe fn EprojInit_BombTorizoLowHealthDrool(j: uint16, game: &mut Game) {
    let mut parameter_1: int16 = 0;
    let mut v4: uint16 = 0;
    let mut v5: uint16 = 0;
    let mut v8: uint16 = 0;
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86a64d)) as *const uint8 as *mut uint16)
            .offset(
                ((((NextRandom(game) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as u16)
                    & (0xeu16)) as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
    NextRandom(game);
    let v2: *mut EnemyData = gEnemyData(game.cur_enemy_index() as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        ((*v2).y_pos).wrapping_sub(5u16);
    parameter_1 = (*v2).parameter_1 as int16;
    if (parameter_1 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int) != 0i32 {
        v4 = (ram_word!(game, random_number) as ::core::ffi::c_int & 0x1fe as ::core::ffi::c_int)
            as uint16;
    } else {
        if parameter_1 < 0 {
            v5 = 32 as uint16;
        } else {
            v5 = 224 as uint16;
        }
        v4 = (2 as ::core::ffi::c_int
            * (v5 as ::core::ffi::c_int
                + (game.random_number() as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
                - 8 as ::core::ffi::c_int)) as uint16;
    }
    let v6: ::core::ffi::c_int = v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        kSinCosTable8bit_Sext[(v6 + 64 as ::core::ffi::c_int) as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        kSinCosTable8bit_Sext[v6 as usize] as uint16;
    let v7: *mut EnemyData = gEnemyData(game.cur_enemy_index() as ::core::ffi::c_int, game);
    if ((*v7).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        v8 = ((*v7).x_pos).wrapping_add(8u16);
    } else {
        v8 = ((*v7).x_pos as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) = v8;
}
unsafe fn EprojInit_BombTorizoLowHealthInitialDrool(j: uint16, game: &mut Game) {
    let mut v3: int16 = 0;
    let mut parameter_1: int16 = 0;
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    NextRandom(game);
    let v2: *mut EnemyData = gEnemyData(game.cur_enemy_index() as ::core::ffi::c_int, game);
    // Avoid a shared Game reference while the returned enemy pointer remains live.
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (((*v2).y_pos as ::core::ffi::c_int
            + (u16::from_le(
                (&raw const game.g_ram)
                    .cast::<u8>()
                    .add(0x5e5)
                    .cast::<u16>()
                    .read_unaligned(),
            ) as ::core::ffi::c_int
                & 3 as ::core::ffi::c_int)) as u16)
            .wrapping_sub(5u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        ((u16::from_le(
            (&raw const game.g_ram)
                .cast::<u8>()
                .add(0x5e5)
                .cast::<u16>()
                .read_unaligned(),
        ) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int) as u16)
            .wrapping_add(48u16);
    NextRandom(game);
    v3 = ((u16::from_le(
        (&raw const game.g_ram)
            .cast::<u8>()
            .add(0x5e5)
            .cast::<u16>()
            .read_unaligned(),
    )) as i16)
        & (3i16);
    parameter_1 = (*v2).parameter_1 as int16;
    if (parameter_1 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int) != 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
            ((*v2).x_pos as ::core::ffi::c_int + v3 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
    } else {
        if parameter_1 < 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v1 as isize) = (((*v2).x_pos as ::core::ffi::c_int
                + v3 as ::core::ffi::c_int) as u16)
                .wrapping_add(8u16);
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v1 as isize) = (((*v2).x_pos as ::core::ffi::c_int
                + v3 as ::core::ffi::c_int) as u16)
                .wrapping_sub(8u16);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
    };
}
unsafe fn EprojInit_A977(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (16 as ::core::ffi::c_int * ram_word!(game, plm_x_block) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (16 as ::core::ffi::c_int * ram_word!(game, plm_y_block) as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
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
    9 as int16,
    21 as int16,
    20 as int16,
    -(52 as ::core::ffi::c_int) as int16,
    -(27 as ::core::ffi::c_int) as int16,
    -(10 as ::core::ffi::c_int) as int16,
    9 as int16,
    20 as int16,
];
static kEprojInit_BombTorizoStatueBreaking_Xpos: [int16; 16] = [
    8 as int16,
    0x18 as int16,
    -(8 as ::core::ffi::c_int) as int16,
    8 as int16,
    0x18 as int16,
    -(8 as ::core::ffi::c_int) as int16,
    8 as int16,
    0x18 as int16,
    8 as int16,
    -(8 as ::core::ffi::c_int) as int16,
    0x18 as int16,
    8 as int16,
    -(8 as ::core::ffi::c_int) as int16,
    0x18 as int16,
    8 as int16,
    -(8 as ::core::ffi::c_int) as int16,
];
static kEprojInit_BombTorizoStatueBreaking_Ypos: [int16; 8] = [
    -(8 as ::core::ffi::c_int) as int16,
    -(8 as ::core::ffi::c_int) as int16,
    8 as int16,
    8 as int16,
    8 as int16,
    0x18 as int16,
    0x18 as int16,
    0x18 as int16,
];
static kEprojInit_BombTorizoStatueBreaking_Yvel: [int16; 8] = [
    256 as int16,
    256 as int16,
    256 as int16,
    256 as int16,
    256 as int16,
    256 as int16,
    256 as int16,
    256 as int16,
];
static kEprojInit_BombTorizoStatueBreaking_F: [int16; 8] = [
    16 as int16,
    16 as int16,
    16 as int16,
    16 as int16,
    16 as int16,
    16 as int16,
    16 as int16,
    16 as int16,
];
static kEprojInit_BombTorizoLowHealthExplode_X: [int16; 6] = [
    0 as int16,
    12 as int16,
    -(12 as ::core::ffi::c_int) as int16,
    0 as int16,
    16 as int16,
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
unsafe fn EprojInit_BombTorizoExplosiveSwipe(j: uint16, game: &mut Game) {
    let v1: *mut EnemyData = gEnemyData(game.cur_enemy_index() as ::core::ffi::c_int, game);
    if ((*v1).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        let v4: ::core::ffi::c_int =
            ram_word!(game, cinematic_var8) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let v5: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v5 as isize) =
            ((*v1).x_pos)
                .wrapping_sub((kEprojInit_BombTorizoExplosiveSwipe_Tab0[v4 as usize]) as u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v5 as isize) =
            ((*v1).y_pos)
                .wrapping_add((kEprojInit_BombTorizoExplosiveSwipe_Tab1[v4 as usize]) as u16);
    } else {
        let v2: ::core::ffi::c_int =
            ram_word!(game, cinematic_var8) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v3 as isize) =
            ((*v1).x_pos as ::core::ffi::c_int
                + kEprojInit_BombTorizoExplosiveSwipe_Tab0[v2 as usize] as ::core::ffi::c_int)
                as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v3 as isize) =
            ((*v1).y_pos as ::core::ffi::c_int
                + kEprojInit_BombTorizoExplosiveSwipe_Tab1[v2 as usize] as ::core::ffi::c_int)
                as uint16;
    };
}
unsafe fn EprojInit_BombTorizoStatueBreaking(j: uint16, game: &mut Game) {
    let mut v1: int8 = 0;
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    v1 = EprojState(&game.g_ram).init_param_1() as int8;
    let v2: ::core::ffi::c_int =
        EprojState(&game.g_ram).init_param_1() as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v3 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86a7ab)) as *const uint8 as *mut uint16)
            .offset(v2 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v3 as isize) =
        (kEprojInit_BombTorizoStatueBreaking_Xpos[v2 as usize] as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int * ram_word!(game, plm_x_block) as ::core::ffi::c_int)
            as uint16;
    let v4: ::core::ffi::c_int =
        (v1 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v3 as isize) =
        (kEprojInit_BombTorizoStatueBreaking_Ypos[v4 as usize] as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int * ram_word!(game, plm_y_block) as ::core::ffi::c_int)
            as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v3 as isize) =
        kEprojInit_BombTorizoStatueBreaking_Yvel[v4 as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v3 as isize) =
        kEprojInit_BombTorizoStatueBreaking_F[v4 as usize] as uint16;
    EprojStateMut(&mut game.g_ram).set_property_bits(v3 as usize, 0x1000);
}
unsafe fn EprojInit_BombTorizoLowHealthExplode(j: uint16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        game.enemy_state(enemy_index).x_pos();
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        game.enemy_state(enemy_index).y_pos();
    if (game.enemy_state(enemy_index).parameter_1() as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        let ref mut fresh13 = *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16);
        *fresh13 = (*fresh13 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
    EprojStateMut(&mut game.g_ram).add_word(0x1993, 0, 2);
    let v3: ::core::ffi::c_int =
        EprojState(&game.g_ram).init_param_1() as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v4: uint16 = (kEprojInit_BombTorizoLowHealthExplode_X[v3 as usize] as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize)
            as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) = v4;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) = v4;
    let v5: uint16 = (kEprojInit_BombTorizoLowHealthExplode_Y[v3 as usize] as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize)
            as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) = v5;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v2 as isize) = v5;
}
unsafe fn EprojInit_BombTorizoDeathExplosion(j: uint16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let x_pos: uint16 = game.enemy_state(enemy_index).x_pos();
    let v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v3 as isize) =
        x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v3 as isize) =
        x_pos;
    let y_pos: uint16 = game.enemy_state(enemy_index).y_pos();
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v3 as isize) =
        y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v3 as isize) =
        y_pos;
}
pub unsafe fn sub_86A887(v0: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v3: int16 = 0;
    let mut v4: int16 = 0;
    if EprojBlockCollisition_Horiz(v0, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let v6: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v6 as isize) =
            0xa48a as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v6 as isize) =
            1 as uint16;
    } else {
        let v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
            .offset(v1 as isize) as int16;
        if v2 >= 0 {
            v4 = (v2).wrapping_sub(4i16);
            if v4 < 0 {
                v4 = 3 as int16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) = v4 as uint16;
        } else {
            v3 = (v2 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as int16;
            if v3 >= 0i16 {
                v3 = 3 as int16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) = v3 as uint16;
        }
        let carry: uint8 = EprojBlockCollisition_Vertical(v0, game);
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            != 0i32
            || carry == 0
        {
            let v5: uint16 = EprojStateMut(&mut game.g_ram).add_word(0x1adb, v1 as usize, 16);
            if (v5 as ::core::ffi::c_int & 0xf000 as ::core::ffi::c_int) == 4096i32 {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                    .offset(v1 as isize) = 0 as uint16;
            }
        } else {
            let v7: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            EprojStateMut(&mut game.g_ram).add_word(0x1a93, v7 as usize, (-3i16) as u16);
            EprojStateMut(&mut game.g_ram).start_instruction(v7 as usize, 0xa48e as uint16);
        }
    };
}
pub unsafe fn EprojPreInstr_A977(k: uint16, game: &mut Game) {
    let carry: uint8 = EprojBlockCollisition_Vertical(k, game);
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
        || carry == 0
    {
        let v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
            .offset(0))
        .wrapping_add(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v1 as isize),
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
            v2;
        if (v2 as ::core::ffi::c_int & 0xf000 as ::core::ffi::c_int) == 4096i32 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v1 as isize) = 4096 as uint16;
        }
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v1 as isize) =
            0xa918 as uint16;
    };
}
pub unsafe fn sub_86A91A(v0: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    if (ram_word!(game, joypad2_last) as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) != 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
            256 as uint16;
    }
    if (ram_word!(game, joypad2_last) as ::core::ffi::c_int & 0x200 as ::core::ffi::c_int) != 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
            -(256 as ::core::ffi::c_int) as uint16;
    }
    EprojBlockCollisition_Horiz(v0, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    if (ram_word!(game, joypad2_last) as ::core::ffi::c_int & 0x400 as ::core::ffi::c_int) != 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
            256 as uint16;
    }
    if (ram_word!(game, joypad2_last) as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int) != 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
            -(256 as ::core::ffi::c_int) as uint16;
    }
    EprojBlockCollisition_Vertical(v0, game);
}
unsafe fn EprojInit_AB07(j: uint16, game: &mut Game) {
    let mut v1: uint16 = ram_word!(game, vram_write_queue_tail);
    let v2: *mut VramWriteEntry = gVramWriteEntry(
        ram_word!(game, vram_write_queue_tail) as ::core::ffi::c_int,
        game,
    );
    (*v2).size = 64 as uint16;
    (*v2).src.addr = addr_kEprojInit_AB07_Tile0 as ::core::ffi::c_int as VoidP;
    *(&raw mut (*v2).src.bank as *mut uint16) = 134 as uint16;
    (*v2).vram_dst = 28160 as uint16;
    v1 = (v1 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
    let v3: *mut VramWriteEntry = gVramWriteEntry(v1 as ::core::ffi::c_int, game);
    (*v3).size = 64 as uint16;
    (*v3).src.addr = addr_kEprojInit_AB07_Tile1 as ::core::ffi::c_int as VoidP;
    *(&raw mut (*v3).src.bank as *mut uint16) = 134 as uint16;
    (*v3).vram_dst = 28416 as uint16;
    set_ram_word!(
        game.vram_write_queue_tail = (v1 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16
    );
    let v4: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v4 as isize) =
        ram_word!(game, samus_x_pos);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v4 as isize) =
        (ram_word!(game, samus_y_pos) as ::core::ffi::c_int - 36 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojPreInstr_AB07(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = ((8 as ::core::ffi::c_int
        * (ram_word!(game, samus_x_pos) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int)) as int16
        as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        amt as uint32,
    );
    amt = ((8 as ::core::ffi::c_int
        * (ram_word!(game, samus_y_pos) as ::core::ffi::c_int
            - 36 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int)) as int16
        as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        amt as uint32,
    );
}
pub unsafe fn EprojInstr_SpawnEnemyDrops(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize),
    };
    SpawnEnemyDrops(
        (if ram_word!(game, area_index) != 0u16 {
            *(epjp.offset(2) as *mut uint16) as ::core::ffi::c_int
        } else {
            *(epjp as *mut uint16) as ::core::ffi::c_int
        }) as uint16,
        k,
        0 as uint16,
        game,
    );
    epjp.offset(4)
}
unsafe fn Eproj_InitXYVelRandom(j: uint16, k: uint16, pt: Point16U, game: &mut Game) {
    let v2: *const uint16 = RomPtr_86(k, game) as *const uint16;
    let v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v3 as isize) = *v2;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v3 as isize) =
        (*v2.offset(1)).wrapping_add(pt.x);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v3 as isize) =
        (*v2.offset(2) as ::core::ffi::c_int + NextRandom(game) as uint8 as ::core::ffi::c_int
            - 128 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v3 as isize) =
        (*v2.offset(3)).wrapping_add(pt.y);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v3 as isize) =
        (*v2.offset(4) as ::core::ffi::c_int + NextRandom(game) as uint8 as ::core::ffi::c_int
            - 128 as ::core::ffi::c_int) as uint16;
}
unsafe fn EprojInit_BombTorizosChozoOrbs(j: uint16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v2: uint16 = (if (game.enemy_state(enemy_index).parameter_1() as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        addr_kEprojInit_BombTorizosChozoOrbs_init0 as ::core::ffi::c_int
    } else {
        addr_kEprojInit_BombTorizosChozoOrbs_init1 as ::core::ffi::c_int
    }) as uint16;
    Eproj_InitXYVelRandom(
        j,
        v2,
        Point16U {
            x: game.enemy_state(enemy_index).x_pos(),
            y: game.enemy_state(enemy_index).y_pos(),
        },
        game,
    );
}
unsafe fn EprojInit_GoldenTorizosChozoOrbs(j: uint16, game: &mut Game) {
    let E: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v2: uint16 =
        (if ((*E).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
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
pub unsafe fn EprojPreInstr_BombTorizosChozoOrbs(k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) != 0 {
        let v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        EprojStateMut(&mut game.g_ram).start_instruction(v3 as usize, addr_off_86AB25 as uint16);
    } else {
        let carry: uint8 = EprojBlockCollisition_Vertical(k, game);
        let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            != 0i32
            || carry == 0
        {
            let v2: uint16 = EprojStateMut(&mut game.g_ram).add_word(0x1adb, v1 as usize, 18);
            if (v2 as ::core::ffi::c_int & 0xf000 as ::core::ffi::c_int) == 4096i32 {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                    .offset(v1 as isize) = 0 as uint16;
            }
        } else {
            let v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v4 as isize) = ((game.enemy_projectile_y_pos(k) as ::core::ffi::c_int
                & 0xfff0 as ::core::ffi::c_int
                | 8 as ::core::ffi::c_int) as u16)
                .wrapping_sub(2u16);
            EprojStateMut(&mut game.g_ram)
                .start_instruction(v4 as usize, addr_off_86AB41 as uint16);
        }
    };
}
pub unsafe fn EprojPreInstr_GoldenTorizosChozoOrbs(k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = 0;
    let mut v2: int16 = 0;
    let mut v3: uint16 = 0;
    let mut v4: uint16 = 0;
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            -(*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int) as uint16;
    }
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        && {
            v1 = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int)
                == 0i32
        }
        && {
            v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) as int16;
            if v2 >= 0i16 {
                v3 = (v2 as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as uint16;
            } else {
                v3 = (v2 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
            };
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) = v3;
            v4 = -(*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v1 as isize) = v4;
            (v4 as ::core::ffi::c_int & 0xff80 as ::core::ffi::c_int) == 0xff80i32
        }
    {
        let v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v5 as isize) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v5 as isize) as ::core::ffi::c_int
                & 0xfff0 as ::core::ffi::c_int
                | 8 as ::core::ffi::c_int)
                - 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v5 as isize) =
            addr_off_86AB41 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v5 as isize) =
            1 as uint16;
    } else {
        let ref mut fresh110 = *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh110 = (*fresh110 as ::core::ffi::c_int + 24 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn EprojInstr_GotoDependingOnXDirection(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let amt: int32 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
        .offset(v2 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v2 as isize)
            as *mut uint16,
        amt as uint32,
    );
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        *(epjp.offset(2) as *mut uint16) as uintptr_t as *const uint8
    } else {
        *(epjp as *mut uint16) as uintptr_t as *const uint8
    }
}
unsafe fn EprojInit_TorizoSonicBoom(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int =
        if (NextRandom(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32 {
            -(12 as ::core::ffi::c_int)
        } else {
            20 as ::core::ffi::c_int
        };
    let v2: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v3 as isize) =
        ((*v2).y_pos as ::core::ffi::c_int + v1) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v3 as isize) =
        0 as uint16;
    if ((*v2).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v3 as isize) =
            ((*v2).x_pos as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v3 as isize) =
            624 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v3 as isize) =
            addr_off_86ADD2 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v3 as isize) =
            ((*v2).x_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v3 as isize) =
            -(624 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v3 as isize) =
            addr_off_86ADBF as uint16;
    };
}
pub unsafe fn EprojPreInstr_TorizoSonicBoom(k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v4 as isize) =
            addr_off_86ADE5 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v4 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v4 as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v4 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v4 as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v4 as isize);
    } else {
        let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh109 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
            .offset(v1 as isize);
        *fresh109 = (*fresh109 as ::core::ffi::c_int
            + if *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                -(16 as ::core::ffi::c_int)
            } else {
                16 as ::core::ffi::c_int
            }) as uint16;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int)
            == 4096i32
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
        }
    };
}
unsafe fn EprojInit_WreckedShipChozoSpikeFootsteps(j: uint16, game: &mut Game) {
    let v1: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        (ram_word!(game, cinematic_var8) as ::core::ffi::c_int + (*v1).x_pos as ::core::ffi::c_int)
            as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        ((*v1).y_pos as ::core::ffi::c_int + 28 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojInstr_ResetXYpos1(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v2 as isize);
    epjp
}
unsafe fn EprojInit_TourianStatueDustClouds(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
        188 as uint16;
}
unsafe fn EprojInit_TourianLandingDustCloudsRightFoot(j: uint16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        (game.enemy_state(enemy_index).y_pos()).wrapping_add(48u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        (game.enemy_state(enemy_index).x_pos()).wrapping_add(24u16);
}
pub fn EprojInstr_MoveY_Minus4(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    EprojStateMut(&mut game.g_ram).add_word(0x1a93, usize::from(k >> 1), (-4i16) as u16);
    epjp
}
unsafe fn EprojInit_TorizoLandingDustCloudLeftFoot(j: uint16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        (game.enemy_state(enemy_index).y_pos()).wrapping_add(48u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        (game.enemy_state(enemy_index).x_pos()).wrapping_sub(24u16);
}
unsafe fn EprojInit_GoldenTorizoEgg(j: uint16, game: &mut Game) {
    let v1: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v2 as isize) =
        ((0xe2 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) + 64 as ::core::ffi::c_int)
            as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) =
        (*v1).parameter_1;
    let v4: uint16 = (if (*v1).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    {
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
pub unsafe fn EprojPreInstr_GoldenTorizoEgg(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh104 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize);
    *fresh104 = (*fresh104).wrapping_sub(1);
    if (*fresh104 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        let ref mut fresh105 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16)
            .offset(v1 as isize);
        *fresh105 = (*fresh105 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
            (if (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int)
                != 0i32
            {
                256 as ::core::ffi::c_int
            } else {
                -(256 as ::core::ffi::c_int)
            }) as uint16;
    } else {
        if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) =
                -(*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
            let ref mut fresh106 = *((&raw mut game.g_ram as *mut uint8).offset(0x1aff)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh106 = (*fresh106 as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int) as uint16;
        }
        if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
            && (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int)
                == 0i32
        {
            let ref mut fresh107 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh107 = (*fresh107 as ::core::ffi::c_int
                + if *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    32 as ::core::ffi::c_int
                } else {
                    -(32 as ::core::ffi::c_int)
                }) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v1 as isize) =
                -(*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
        }
        let ref mut fresh108 = *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset(v1 as isize);
        *fresh108 = (*fresh108 as ::core::ffi::c_int + 48 as ::core::ffi::c_int) as uint16;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int)
            == 4096i32
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
        }
    };
}
pub unsafe fn sub_86B0B9(k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v3 as isize) =
            0xb0dd as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v3 as isize) =
            0 as uint16;
    } else {
        let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            != 0i32
        {
            v2 = 48 as int16;
        } else {
            v2 = -(48 as ::core::ffi::c_int) as int16;
        }
        let ref mut fresh103 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
            .offset(v1 as isize);
        *fresh103 = (*fresh103 as ::core::ffi::c_int + v2 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn sub_86B0DD(k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
            (if (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int)
                != 0i32
            {
                addr_off_86B1A8 as ::core::ffi::c_int
            } else {
                addr_off_86B190 as ::core::ffi::c_int
            }) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    } else {
        let ref mut fresh102 = *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh102 = (*fresh102 as ::core::ffi::c_int + 48 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn sub_86B13E(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    ((if (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        addr_off_86B166 as ::core::ffi::c_int
    } else {
        addr_off_86B14B as ::core::ffi::c_int
    }) as uintptr_t as *const uint8)
}
static word_86B205: [int16; 2] = [-(0x1e as ::core::ffi::c_int) as int16, 0x1e as int16];
unsafe fn EprojInit_GoldenTorizoSuperMissile(j: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) =
        ram_word!(game, cur_enemy_index);
    let v3: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v4: ::core::ffi::c_int =
        if ((*v3).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        ((*v3).x_pos as ::core::ffi::c_int + word_86B205[v4 as usize] as ::core::ffi::c_int)
            as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        ((*v3).y_pos as ::core::ffi::c_int - 52 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86b209)) as *const uint8 as *mut uint16)
            .offset(v4 as isize);
}
pub unsafe fn EprojPreInstr_GoldenTorizoSuperMissile(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int,
        game,
    );
    let v3: ::core::ffi::c_int =
        if ((*v2).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            32 as ::core::ffi::c_int
        } else {
            -(32 as ::core::ffi::c_int)
        };
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        ((*v2).x_pos as ::core::ffi::c_int + v3) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        ((*v2).y_pos as ::core::ffi::c_int - 52 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojPreInstr_B237(k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = 0;
    let mut carry: uint8 = 0;
    if EprojBlockCollisition_Horiz(k, game) != 0u8 || {
        carry = EprojBlockCollisition_Vertical(k, game);
        v1 = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            == 0i32
            && carry != 0u8
    } {
        let v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v3 as isize) =
            addr_off_86B2EF as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v3 as isize) =
            1 as uint16;
    } else {
        let v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
            v2;
        if (v2 as ::core::ffi::c_int & 0xf000 as ::core::ffi::c_int) == 4096i32 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
        }
    };
}
unsafe fn sub_86B279(k: uint16, a: uint16, game: &mut Game) {
    let v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v3 as isize) =
        (4 as ::core::ffi::c_int
            * kSinCosTable8bit_Sext[(a as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v3 as isize) =
        (4 as ::core::ffi::c_int * kSinCosTable8bit_Sext[a as usize] as ::core::ffi::c_int)
            as uint16;
}
pub unsafe fn EprojInstr_SetVelTowardsSamus1(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    sub_86B279(
        k,
        (CalculateAngleOfSamusFromEproj(k, game) as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int)
            as uint16,
        game,
    );
    epjp
}
pub unsafe fn EprojInstr_SetVelTowardsSamus2(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    sub_86B279(
        k,
        (CalculateAngleOfSamusFromEproj(k, game) as ::core::ffi::c_int | 0x80 as ::core::ffi::c_int)
            as uint16,
        game,
    );
    epjp
}
unsafe fn EprojInit_GoldenTorizoEyeBeam(j: uint16, game: &mut Game) {
    let E: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v2: uint16 =
        (if ((*E).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
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
    if ((*E).parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        v3 = (v3 as ::core::ffi::c_int + 128 as ::core::ffi::c_int) as uint16;
    }
    let v4: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v5: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v5 as isize) = (8
        as ::core::ffi::c_int
        * kSinCosTable8bit_Sext[(v4 + 64 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int)
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v5 as isize) =
        (8 as ::core::ffi::c_int * kSinCosTable8bit_Sext[v4 as usize] as ::core::ffi::c_int)
            as uint16;
}
pub unsafe fn EprojPreInstr_GoldenTorizoEyeBeam(k: uint16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            addr_off_86B3CD as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
        return;
    }
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0xfff0 as ::core::ffi::c_int
                | 8 as ::core::ffi::c_int)
                - 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
            addr_off_86B3E5 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    }
}
pub unsafe fn EprojInstr_GotoIfFunc1(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    if ((*gExtraEnemyRam7800(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int,
        game,
    ))
    .kraid
    .kraid_healths_8ths[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        *(epjp as *mut uint16) as uintptr_t as *const uint8
    } else {
        epjp.offset(2)
    }
}
pub unsafe fn EprojInstr_ResetXYpos2(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v2 as isize);
    epjp
}
unsafe fn EprojInit_TourianEscapeShaftFakeWallExplode(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        272 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        272 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        2184 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
        2184 as uint16;
}
unsafe fn EprojInit_LavaSeahorseFireball(j: uint16, game: &mut Game) {
    let v1: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        ((*v1).y_pos).wrapping_sub(28u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        -(961 as ::core::ffi::c_int) as uint16;
    if ((*v1).ai_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
            ((*v1).x_pos as ::core::ffi::c_int + 12 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
            704 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
            addr_word_86B4CB as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
            ((*v1).x_pos).wrapping_sub(12u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
            -(704 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
            addr_word_86B4BF as uint16;
    };
}
pub unsafe fn sub_86B535(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let split_amount = ((EprojState(&game.g_ram).slot_word(0x1ab7, v1 as usize) as int16
        as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a4b, 0x1a27, v1 as usize, split_amount);
    let split_amount = ((EprojState(&game.g_ram).slot_word(0x1adb, v1 as usize) as int16
        as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a93, 0x1a6f, v1 as usize, split_amount);
    let v6: int16 = EprojState(&game.g_ram).slot_word(0x1adb, v1 as usize) as int16;
    if v6 >= 0i16 {
        EprojStateMut(&mut game.g_ram).write(0x1adb, v1 as usize, (v6 as u16).wrapping_add(32));
        Eproj_DeleteIfYposOutside(k, game);
    } else {
        EprojStateMut(&mut game.g_ram).write(0x1adb, v1 as usize, (v6 as u16).wrapping_add(32));
        if (v6).wrapping_add(32i16) >= 0i16 {
            {
                let instruction: uint16 =
                    (if (*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int)
                        == 0i32
                    {
                        addr_word_86B4E3 as ::core::ffi::c_int
                    } else {
                        addr_word_86B4D7 as ::core::ffi::c_int
                    }) as uint16;
                EprojStateMut(&mut game.g_ram).start_instruction(v1 as usize, instruction);
            }
        }
    };
}
pub unsafe fn Eproj_DeleteIfYposOutside(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize))
        as i16)
        .wrapping_sub((ram_word!(game, layer1_y_pos)) as i16)
        >= 288i16
    {
        EprojStateMut(&mut game.g_ram).set_id(v1 as usize, 0);
    }
}
static word_86B65B: [int16; 20] = [
    -(16 as ::core::ffi::c_int) as int16,
    16 as int16,
    -(96 as ::core::ffi::c_int) as int16,
    -(64 as ::core::ffi::c_int) as int16,
    -(128 as ::core::ffi::c_int) as int16,
    -(32 as ::core::ffi::c_int) as int16,
    -(96 as ::core::ffi::c_int) as int16,
    64 as int16,
    -(128 as ::core::ffi::c_int) as int16,
    32 as int16,
    16 as int16,
    16 as int16,
    96 as int16,
    -(64 as ::core::ffi::c_int) as int16,
    112 as int16,
    -(64 as ::core::ffi::c_int) as int16,
    128 as int16,
    -(64 as ::core::ffi::c_int) as int16,
    144 as int16,
    -(64 as ::core::ffi::c_int) as int16,
];
unsafe fn EprojInit_EyeDoorProjectile(j: uint16, game: &mut Game) {
    let v1: uint16 = ram_word!(game, plm_id);
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(
            (ram_word!(game, plm_id) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
        );
    CalculatePlmBlockCoords(v1, game);
    let v3: ::core::ffi::c_int =
        ram_word!(game, cinematic_var8) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        (word_86B65B[v3 as usize] as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int
                * (2 as ::core::ffi::c_int * ram_word!(game, plm_x_block) as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        (word_86B65B[(v3 + 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int * ram_word!(game, plm_y_block) as ::core::ffi::c_int)
            as uint16;
}
static word_86B6B1: [int16; 4] = [
    -(64 as ::core::ffi::c_int) as int16,
    512 as int16,
    64 as int16,
    512 as int16,
];
unsafe fn EprojInit_EyeDoorSweat(j: uint16, game: &mut Game) {
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (8 as ::core::ffi::c_int
            * (2 as ::core::ffi::c_int
                * (ram_word!(game, plm_x_block) as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int)) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) = (16
        as ::core::ffi::c_int
        * (ram_word!(game, plm_y_block) as ::core::ffi::c_int + 1 as ::core::ffi::c_int))
        as uint16;
    let v2: ::core::ffi::c_int =
        ram_word!(game, cinematic_var8) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        word_86B6B1[v2 as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        word_86B6B1[(v2 + 1 as ::core::ffi::c_int) as usize] as uint16;
}
pub unsafe fn EprojPreInstr_EyeDoorProjectile(k: uint16, game: &mut Game) {
    let v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if !(EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0)
    {
        let v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1aff)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let ref mut fresh100 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
            .offset(v3 as isize);
        *fresh100 = (*fresh100).wrapping_add(
            (kSinCosTable8bit_Sext[(v1 + 64 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
                >> 4 as ::core::ffi::c_int) as u16,
        );
        let ref mut fresh101 = *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset(v3 as isize);
        *fresh101 = (*fresh101 as ::core::ffi::c_int
            + (kSinCosTable8bit_Sext[v1 as usize] as ::core::ffi::c_int >> 4 as ::core::ffi::c_int))
            as uint16;
        let t: ::core::ffi::c_int = PrepareBitAccess(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                .offset(v3 as isize),
            game,
        ) as ::core::ffi::c_int;
        if ((ram_word!(game, bitmask) as ::core::ffi::c_int
            & *(&raw mut game.g_ram as *mut uint8)
                .offset(0xd8b0)
                .offset(t as isize) as ::core::ffi::c_int)
            == 0i32)
        {
            return;
        }
    }
    EprojStateMut(&mut game.g_ram).start_instruction(v3 as usize, addr_off_86B5F3 as uint16);
}
pub unsafe fn EprojPreInstr_EyeDoorSweat(k: uint16, game: &mut Game) {
    EprojBlockCollisition_Horiz(k, game);
    let carry: uint8 = EprojBlockCollisition_Vertical(k, game);
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
        || carry == 0
    {
        EprojStateMut(&mut game.g_ram).add_word(0x1adb, v1 as usize, 12);
    } else {
        EprojStateMut(&mut game.g_ram).add_word(0x1a93, v1 as usize, (-4i16) as u16);
        EprojStateMut(&mut game.g_ram).start_instruction(v1 as usize, addr_off_86B61D as uint16);
    };
}
pub unsafe fn EprojInstr_SpawnTourianStatueUnlockingParticle(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_TourianStatueUnlockingParticle as uint16,
        k,
        game,
    );
    epjp
}
pub unsafe fn EprojInstr_Earthquake(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    set_ram_word!(game.earthquake_type = 1 as uint16);
    let ref mut fresh29 = *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16);
    *fresh29 = (*fresh29 as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int) as uint16;
    epjp
}
pub unsafe fn EprojInstr_SpawnTourianStatueUnlockingParticleTail(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_TourianStatueUnlockingParticleTail as uint16,
        k,
        game,
    );
    epjp
}
pub unsafe fn EprojInstr_AddToYpos(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    let ref mut fresh28 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh28 =
        (*fresh28 as ::core::ffi::c_int + *(epjp as *mut uint16) as ::core::ffi::c_int) as uint16;
    epjp.offset(2)
}
unsafe fn EprojInit_TourianStatueUnlockingParticleWaterSplash(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(
            (ram_word!(game, cinematic_var8) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                as isize,
        );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (ram_word!(game, fx_y_pos) as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
}
static kEprojInit_TourianStatueEyeGlow_X: [uint16; 4] = [
    0x84 as uint16,
    0x7a as uint16,
    0x9e as uint16,
    0x68 as uint16,
];
static kEprojInit_TourianStatueEyeGlow_Y: [uint16; 4] = [
    0x90 as uint16,
    0x51 as uint16,
    0x80 as uint16,
    0x72 as uint16,
];
static kEprojInit_TourianStatueEyeGlow_Colors: [uint16; 16] = [
    0x6bff as uint16,
    0x33b as uint16,
    0x216 as uint16,
    0x173 as uint16,
    0x7f5f as uint16,
    0x7c1f as uint16,
    0x5816 as uint16,
    0x300c as uint16,
    0x7f5a as uint16,
    0x7ec0 as uint16,
    0x6de0 as uint16,
    0x54e0 as uint16,
    0x6bfa as uint16,
    0x3be0 as uint16,
    0x2680 as uint16,
    0x1580 as uint16,
];
unsafe fn EprojInit_TourianStatueEyeGlow(j: uint16, game: &mut Game) {
    let v1: uint16 = ram_word!(game, cinematic_var8);
    let v2: ::core::ffi::c_int =
        ram_word!(game, cinematic_var8) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v3 as isize) =
        kEprojInit_TourianStatueEyeGlow_X[v2 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v3 as isize) =
        kEprojInit_TourianStatueEyeGlow_Y[v2 as usize];
    let mut v4: uint16 = (4 as ::core::ffi::c_int * v1 as ::core::ffi::c_int) as uint16;
    let mut i: ::core::ffi::c_int = 498 as ::core::ffi::c_int;
    while i != 506i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) =
            kEprojInit_TourianStatueEyeGlow_Colors
                [(v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
        v4 = (v4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        i += 2 as ::core::ffi::c_int;
    }
}
unsafe fn EprojInit_TourianStatueUnlockingParticle(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int =
        ram_word!(game, cinematic_var8) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize);
    let v3: uint16 = (2 as ::core::ffi::c_int
        * ((NextRandom(game) as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int)
            - 32 as ::core::ffi::c_int) as uint8 as ::core::ffi::c_int)
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) = v3;
    let v4: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        kSinCosTable8bit_Sext[(v4 + 64 as ::core::ffi::c_int) as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        (4 as ::core::ffi::c_int * kSinCosTable8bit_Sext[v4 as usize] as ::core::ffi::c_int)
            as uint16;
}
pub unsafe fn EprojIni_TourianStatueUnlockingParticleTail(v0: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int =
        ram_word!(game, cinematic_var8) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize);
}
unsafe fn EprojInit_TourianStatueSoul(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int =
        ram_word!(game, cinematic_var8) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        kEprojInit_TourianStatueEyeGlow_X[v1 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        kEprojInit_TourianStatueEyeGlow_Y[v1 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        -(1024 as ::core::ffi::c_int) as uint16;
}
unsafe fn EprojInit_TourianStatueBaseDecoration(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        120 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        120 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
        184 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        184 as uint16;
}
unsafe fn EprojInit_TourianStatueRidley(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        142 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        142 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
        85 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        85 as uint16;
}
unsafe fn EprojInit_TourianStatuePhantoon(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        132 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        132 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
        136 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        136 as uint16;
}
pub unsafe fn EprojPreInstr_TourianStatueUnlockingParticleWaterSplash(k: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (ram_word!(game, fx_y_pos) as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojPreInstr_TourianStatueUnlockingParticle(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        amt as uint32,
    );
    let v7: uint16 = (ram_word!(game, fx_y_pos) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int) as uint16;
    amt = (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
        as int16 as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        amt as uint32,
    );
    if ((v7 as ::core::ffi::c_int
        ^ ram_word!(game, fx_y_pos) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int)
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        SpawnEprojWithRoomGfx(addr_stru_86BA5C as uint16, k, game);
    }
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        == 256i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
            addr_off_86B79F as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    } else {
        let ref mut fresh97 = *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset(v1 as isize);
        *fresh97 = (*fresh97 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn EprojPreInstr_TourianStatueSoul(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let amt: int32 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        amt as uint32,
    );
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        & 0x100 as ::core::ffi::c_int)
        != 0i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
            addr_off_86B79F as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    }
    let ref mut fresh96 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize);
    *fresh96 = (*fresh96 as ::core::ffi::c_int - 128 as ::core::ffi::c_int) as uint16;
}
unsafe fn EprojPreInstr_BA42(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            + ram_word!(game, layer1_y_pos) as ::core::ffi::c_int
            - *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x9e00)
                as *mut ExpandingSquareTransitionHdma)
                .offset(0))
            .field_0 as *mut uint16) as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojPreInstr_TourianStatueStuff(k: uint16, game: &mut Game) {
    if ram_word!(game, tourian_entrance_statue_animstate) == 0 {
        let ref mut fresh95 = *((&raw mut game.g_ram as *mut uint8).offset(0x1e6d) as *mut uint16);
        *fresh95 = (*fresh95 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
    }
    EprojPreInstr_BA42(k, game);
}
pub unsafe fn sub_86BB30(j: uint16, game: &mut Game) {
    let v1: *mut ExtraEnemyRam8000 =
        gExtraEnemyRam8000(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        *((&raw mut (*v1).pad as *mut uint8).offset(34) as *mut uint8 as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        *((&raw mut (*v1).pad as *mut uint8).offset(36) as *mut uint8 as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86bb1e)) as *const uint8 as *mut uint16)
            .offset(ram_word!(game, cinematic_var8) as isize);
}
unsafe fn EprojInit_NuclearWaffleBody(j: uint16, game: &mut Game) {
    let v2: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v3 as isize) =
        (*v2).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v3 as isize) =
        (*v2).x_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v3 as isize) =
        (*v2).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v3 as isize) =
        (*v2).y_subpos;
    let v4: *mut ExtraEnemyRam8000 =
        gExtraEnemyRam8000(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*gExtraEnemyRam7800(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int
            + *((&raw mut (*v4).pad as *mut uint8).offset(20) as *mut uint8 as *mut uint16)
                as ::core::ffi::c_int,
        game,
    ))
    .kraid
    .kraid_next = j;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf380) as *mut uint16).offset(v3 as isize) =
        1 as uint16;
}
unsafe fn EprojInit_NorfairLavaquakeRocks(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
        addr_word_86BBD5 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        (0x86bc16 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        ram_word!(game, cinematic_var8);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        ram_word!(game, cinematic_var9);
    let v2: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (*v2).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize) =
        (*v2).x_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (*v2).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize) =
        (*v2).y_subpos;
}
pub unsafe fn CallNorfairLavaquakeRocksFunc(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        8829974 => {
            Eproj_NorfairLavaquakeRocks_Func1(k, game);
        }
        8830095 => {
            Eproj_NorfairLavaquakeRocks_Func2(k, game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn EprojPreInstr_NorfairLavaquakeRocks_Inner2(k: uint16, game: &mut Game) -> uint16 {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (((*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        - ram_word!(game, layer1_x_pos) as ::core::ffi::c_int) as int16)
        < 0i16
        || ((ram_word!(game, layer1_x_pos) as ::core::ffi::c_int + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16)
            < 0i16
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - ram_word!(game, layer1_y_pos) as ::core::ffi::c_int) as int16)
            < 0i16
        || ((ram_word!(game, layer1_y_pos) as ::core::ffi::c_int + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16)
            < 0i16) as uint16
}
pub unsafe fn EprojPreInstr_NorfairLavaquakeRocks_Inner(k: uint16, game: &mut Game) {
    if EprojPreInstr_NorfairLavaquakeRocks_Inner2(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojPreInstr_NorfairLavaquakeRocks(k: uint16, game: &mut Game) {
    CallNorfairLavaquakeRocksFunc(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            | 0x860000 as ::core::ffi::c_int) as uint32,
        k,
        game,
    );
    EprojPreInstr_NorfairLavaquakeRocks_Inner(k, game);
}
pub unsafe fn Eproj_NorfairLavaquakeRocks_Func1(k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v3: int16 = 0;
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    v2 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        - 2 as ::core::ffi::c_int) as int16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        v2 as uint16;
    if v2 >= 0i16 {
        let mut n: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
        loop {
            v3 = (n + *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as int16;
            if v3 < 0i16 {
                v3 = 0 as int16;
            }
            let t: ::core::ffi::c_int = 8 as ::core::ffi::c_int * v3 as ::core::ffi::c_int
                + 4 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            AddToHiLo(
                ((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                    .offset(v1 as isize) as *mut uint16,
                ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16)
                    .offset(v1 as isize) as *mut uint16,
                *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const uint8
                    as *mut uint32)
                    .offset((t >> 1 as ::core::ffi::c_int) as isize),
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                .offset(v1 as isize) = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7))
                as *const uint8 as *mut uint16)
                .offset((t + 1 as ::core::ffi::c_int) as isize);
            n -= 1;
            if (n == 0) {
                break;
            }
        }
        Eproj_NorfairLavaquakeRocks_Func3(k, game);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
            (0x86bc8f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Eproj_NorfairLavaquakeRocks_Func2(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) = v2;
    if v2 as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
            64 as uint16;
    }
    let mut n: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    loop {
        let t: ::core::ffi::c_int = 8 as ::core::ffi::c_int
            * (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - n
                + 1 as ::core::ffi::c_int)
            >> 1 as ::core::ffi::c_int;
        AddToHiLo(
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const uint8 as *mut uint32)
                .offset((t >> 1 as ::core::ffi::c_int) as isize),
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const uint8 as *mut uint16)
                .offset((t + 1 as ::core::ffi::c_int) as isize);
        n -= 1;
        if (n == 0) {
            break;
        }
    }
    Eproj_NorfairLavaquakeRocks_Func3(k, game);
}
pub unsafe fn Eproj_NorfairLavaquakeRocks_Func3(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize)
            as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
static kEprojInit_ShaktoolAttackMiddleBackCircle_X: [int16; 8] = [
    0 as int16,
    12 as int16,
    16 as int16,
    12 as int16,
    0 as int16,
    -(12 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(12 as ::core::ffi::c_int) as int16,
];
static kEprojInit_ShaktoolAttackMiddleBackCircle_Y: [int16; 8] = [
    -(16 as ::core::ffi::c_int) as int16,
    -(12 as ::core::ffi::c_int) as int16,
    0 as int16,
    12 as int16,
    16 as int16,
    12 as int16,
    0 as int16,
    -(12 as ::core::ffi::c_int) as int16,
];
unsafe fn EprojInit_BDA2(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (*v2).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (*v2).y_pos;
    let v3: uint16 = (2 as ::core::ffi::c_int
        * *(&raw mut (*v2).ai_var_D as *mut uint8).offset(0) as ::core::ffi::c_int)
        as uint16;
    let v4: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        kSinCosTable8bit_Sext[(v4 + 64 as ::core::ffi::c_int) as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        kSinCosTable8bit_Sext[v4 as usize] as uint16;
    let v5: ::core::ffi::c_int = (v3 as ::core::ffi::c_int >> 5 as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let ref mut fresh11 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize);
    *fresh11 = (*fresh11 as ::core::ffi::c_int
        + kEprojInit_ShaktoolAttackMiddleBackCircle_X[v5 as usize] as ::core::ffi::c_int)
        as uint16;
    let ref mut fresh12 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize);
    *fresh12 = (*fresh12 as ::core::ffi::c_int
        + kEprojInit_ShaktoolAttackMiddleBackCircle_Y[v5 as usize] as ::core::ffi::c_int)
        as uint16;
}
unsafe fn EprojInit_ShaktoolAttackMiddleBackCircle(j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        ram_word!(game, cinematic_var8);
    EprojInit_BDA2(j, game);
}
unsafe fn EprojInit_ShaktoolAttackFrontCircle(v1: uint16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(v1, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || EprojBlockCollisition_Vertical(v1, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojPreInstr_BE12(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) != 0
    {
        EprojBlockCollisition_Horiz(k, game);
        EprojBlockCollisition_Vertical(k, game);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
    };
}
static kEprojInit_MotherBrainRoomTurrets_X: [int16; 12] = [
    0x398 as int16,
    0x348 as int16,
    0x328 as int16,
    0x2d8 as int16,
    0x288 as int16,
    0x268 as int16,
    0x218 as int16,
    0x1c8 as int16,
    0x1a8 as int16,
    0x158 as int16,
    0x108 as int16,
    0xe8 as int16,
];
static EprojInit_MotherBrainRoomTurrets_Y: [int16; 12] = [
    0x30 as int16,
    0x40 as int16,
    0x40 as int16,
    0x30 as int16,
    0x40 as int16,
    0x40 as int16,
    0x30 as int16,
    0x40 as int16,
    0x40 as int16,
    0x30 as int16,
    0x40 as int16,
    0x40 as int16,
];
unsafe fn EprojInit_MotherBrainRoomTurrets(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        1024 as uint16;
    let v2: uint16 =
        (2 as ::core::ffi::c_int * ram_word!(game, cinematic_var8) as ::core::ffi::c_int) as uint16;
    let v3: uint16 =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x86bee1)) as *const uint8 as *mut uint16)
            .offset(ram_word!(game, cinematic_var8) as isize) as ::core::ffi::c_int
            | 0x100 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize) = v3;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86beb9)) as *const uint8 as *mut uint16)
            .offset(
                ((2 as ::core::ffi::c_int * v3 as uint8 as ::core::ffi::c_int) as uint16
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
    let v4: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        kEprojInit_MotherBrainRoomTurrets_X[v4 as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        EprojInit_MotherBrainRoomTurrets_Y[v4 as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86bec9)) as *const uint8 as *mut uint16)
            .offset(v4 as isize);
    Eproj_SetXvelRandom(j, game);
    Eproj_SetYvelRandom(j, game);
}
static kEproj_MotherBrainRoomTurretBullets_X: [int16; 8] = [
    -(17 as ::core::ffi::c_int) as int16,
    -(12 as ::core::ffi::c_int) as int16,
    0 as int16,
    12 as int16,
    17 as int16,
    12 as int16,
    0 as int16,
    -(12 as ::core::ffi::c_int) as int16,
];
static kEproj_MotherBrainRoomTurretBullets_Y: [int16; 8] = [
    -(9 as ::core::ffi::c_int) as int16,
    3 as int16,
    7 as int16,
    3 as int16,
    -(9 as ::core::ffi::c_int) as int16,
    -(19 as ::core::ffi::c_int) as int16,
    -(21 as ::core::ffi::c_int) as int16,
    -(19 as ::core::ffi::c_int) as int16,
];
static kEproj_MotherBrainRoomTurretBullets_Xvel: [int16; 8] = [
    -(704 as ::core::ffi::c_int) as int16,
    -(498 as ::core::ffi::c_int) as int16,
    0 as int16,
    498 as int16,
    704 as int16,
    498 as int16,
    0 as int16,
    -(498 as ::core::ffi::c_int) as int16,
];
static kEproj_MotherBrainRoomTurretBullets_Yvel: [int16; 8] = [
    0 as int16,
    498 as int16,
    704 as int16,
    498 as int16,
    0 as int16,
    -(498 as ::core::ffi::c_int) as int16,
    -(704 as ::core::ffi::c_int) as int16,
    -(498 as ::core::ffi::c_int) as int16,
];
unsafe fn EprojInit_MotherBrainRoomTurretBullets(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        1024 as uint16;
    let v2: uint16 = (2 as ::core::ffi::c_int
        * *(((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(
            (ram_word!(game, cinematic_var8) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                as isize,
        ) as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) = v2;
    let v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let pt: Point16U = Point16U {
        x: kEproj_MotherBrainRoomTurretBullets_X[v3 as usize] as uint16,
        y: kEproj_MotherBrainRoomTurretBullets_Y[v3 as usize] as uint16,
    };
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        kEproj_MotherBrainRoomTurretBullets_Xvel[v3 as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        kEproj_MotherBrainRoomTurretBullets_Yvel[v3 as usize] as uint16;
    let v4: ::core::ffi::c_int =
        ram_word!(game, cinematic_var8) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (pt.x as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v4 as isize) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (pt.y as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v4 as isize) as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojPreInstr_MotherBrainRoomTurrets(k: uint16, game: &mut Game) {
    if Eproj_MotherBrainRoomTurretBullets_CheckIfTurretOnScreen(k, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        if (*gRam7800_Default(0 as uint16, game)).var_1D != 0 {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        }
    } else if (*gRam7800_Default(0 as uint16, game)).var_1D != 0 {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        let v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        game.eproj_spawn_pt = Point16U {
            x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v5 as isize),
            y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v5 as isize),
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as uint16,
            0xc as uint16,
            game,
        );
    } else {
        let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh88 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
            .offset(v1 as isize);
        let fresh89 = *fresh88;
        *fresh88 = (*fresh88).wrapping_sub(1);
        let mut v2: bool = fresh89 == 1u16;
        if v2 {
            Eproj_SetXvelRandom(k, game);
            Eproj_MotherBrainRoomTurretBullets_Func2(k, game);
            let v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16)
                .offset(v3 as isize) = *(game.g_rom.offset(crate::rtl::lorom_offset(0x86c040))
                as *const uint8 as *mut uint16)
                .offset(
                    *(((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16)
                        .offset(v3 as isize) as *mut uint16 as *mut uint8)
                        .offset(0) as isize,
                );
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16)
                .offset(v3 as isize) = 1 as uint16;
        }
        let v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh92 = *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset(v4 as isize);
        let fresh93 = *fresh92;
        *fresh92 = (*fresh92).wrapping_sub(1);
        v2 = fresh93 == 1u16;
        if v2 {
            Eproj_SetYvelRandom(k, game);
            SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainRoomTurretBullets as uint16, k, game);
        }
    };
}
pub unsafe fn Eproj_MotherBrainRoomTurretBullets_Func2(v0: uint16, game: &mut Game) {
    let r20: uint8 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16
        as *mut uint8)
        .offset(v0 as ::core::ffi::c_int as isize)
        .offset(1) as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int) as uint8;
    let r18: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    if *RomPtr_86(r18, game).offset((r20 as ::core::ffi::c_int & 7 as ::core::ffi::c_int) as isize)
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) = r20;
    } else {
        let v1: int8 = -(*((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16
            as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize)
            .offset(1) as ::core::ffi::c_int) as int8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize)
            .offset(1) = v1 as uint8;
        let ref mut fresh94 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16
            as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize);
        *fresh94 = (*fresh94 as ::core::ffi::c_int + v1 as ::core::ffi::c_int) as uint8;
    };
}
pub unsafe fn Eproj_SetXvelRandom(v0: uint16, game: &mut Game) {
    let mut Random: uint16 = NextRandom(game) as uint8 as uint16;
    if Random as uint8 as ::core::ffi::c_int - 32 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        Random = 32 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = Random;
}
pub unsafe fn Eproj_SetYvelRandom(v0: uint16, game: &mut Game) {
    let mut Random: uint16 = NextRandom(game) as uint8 as uint16;
    if Random as uint8 as ::core::ffi::c_int - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        Random = 128 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = Random;
}
pub unsafe fn Eproj_MotherBrainRoomTurretBullets_CheckIfTurretOnScreen(
    k: uint16,
    game: &mut Game,
) -> uint8 {
    let mut v2: int16 = 0;
    let mut v3: int16 = 0;
    let mut v4: int16 = 0;
    let mut v5: int16 = 0;
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
        as int16;
    let result: uint8 = 1 as uint8;
    if v2 >= 0i16 {
        v3 = (v2 as ::core::ffi::c_int + 16 as ::core::ffi::c_int
            - ram_word!(game, layer1_y_pos) as ::core::ffi::c_int) as int16;
        if v3 >= 0i16
            && v3 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
        {
            v4 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v1 as isize) as int16;
            if v4 >= 0i16 {
                v5 = (v4 as ::core::ffi::c_int + 4 as ::core::ffi::c_int
                    - ram_word!(game, layer1_x_pos) as ::core::ffi::c_int)
                    as int16;
                if v5 >= 0i16
                    && v5 as ::core::ffi::c_int - 264 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0
                {
                    return 0 as uint8;
                }
            }
        }
    }
    result
}
pub unsafe fn EprojPreInstr_MotherBrainRoomTurretBullets(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh85 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16).offset(v1 as isize);
    *fresh85 = (*fresh85 as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int) as uint16;
    MoveEprojWithVelocity(k, game);
    if Ridley_Func_103(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize),
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize),
        game,
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(
            (ram_word!(game, cinematic_var7) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                as isize,
        ) = 0 as uint16;
    }
}
static kMotherBrainsBomb_Yaccel: [int16; 10] = [
    7 as int16,
    0x10 as int16,
    0x20 as int16,
    0x40 as int16,
    0x70 as int16,
    0xb0 as int16,
    0xf0 as int16,
    0x130 as int16,
    0x170 as int16,
    0 as int16,
];
pub unsafe fn EprojPreInstr_MotherBrainBomb(k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    if MotherBrainBomb_Bomb_CollDetect_DoubleRet(k, game) != 0 {
        return;
    }
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let current_block_19: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) != 0
    {
        let v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let v5: uint16 = kMotherBrainsBomb_Yaccel[(*((&raw mut game.g_ram as *mut uint8)
            .offset(0x1b23) as *mut uint16)
            .offset(v4 as isize)
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as usize] as uint16;
        if v5 == 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v4 as isize) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v4 as isize) = 0 as uint16;
            let fresh81 = &raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x7800)
                as *mut ExtraEnemyRam7800)
                .offset(1))
            .kraid
            .kraid_mouth_flags;
            fresh81.write_unaligned(fresh81.read_unaligned().wrapping_sub(1));
            *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
                .offset(v4 as isize) = 0 as uint16;
            game.eproj_spawn_pt = Point16U {
                x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                    .offset(v4 as isize),
                y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                    .offset(v4 as isize),
            };
            SpawnEprojWithRoomGfx(
                addr_stru_869650 as uint16,
                *(((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16)
                    .offset(v4 as isize) as *mut uint16 as *mut uint8)
                    .offset(0) as uint16,
                game,
            );
            game.eproj_spawn_pt = Point16U {
                x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                    .offset(v4 as isize),
                y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                    .offset(v4 as isize),
            };
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, 3 as uint16, game);
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
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize),
        ) as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int) as int16;
        if v2 < 0i16 {
            v2 = 0 as int16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
            (if *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
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
    if current_block_19 == 15976848397966268834 {
        let v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh82 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
            .offset(v3 as isize);
        *fresh82 = (*fresh82 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn EprojInstr_SwitchJump(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *(epjp.offset(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int as isize,
    ) as *mut uint16) as uintptr_t as *const uint8
}
pub unsafe fn EprojInstr_UserPalette0(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    epjp
}
unsafe fn Eproj_CheckForBombCollisionWithRect(rect: Rect16U, game: &mut Game) -> uint8 {
    if ram_word!(game, bomb_counter) == 0 {
        return 0 as uint8;
    }
    let mut v1: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
    while v1 < 20i32 {
        let v2: ::core::ffi::c_int = v1 >> 1 as ::core::ffi::c_int;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(v2 as isize)
            as ::core::ffi::c_int
            & 0xf00 as ::core::ffi::c_int)
            == 1280i32
            && *((&raw mut game.g_ram as *mut uint8).offset(0xc7c) as *mut uint16)
                .offset(v2 as isize)
                == 0
        {
            let x: uint16 = abs16(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int
                    - rect.x as ::core::ffi::c_int) as uint16,
            );
            if x < *((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
                .offset(v2 as isize)
                || ((x as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
                        .offset(v2 as isize) as ::core::ffi::c_int) as uint16)
                    < rect.w
            {
                let y: uint16 = abs16(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                        .offset(v2 as isize) as ::core::ffi::c_int
                        - rect.y as ::core::ffi::c_int) as uint16,
                );
                if y < *((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                    .offset(v2 as isize)
                    || ((y as ::core::ffi::c_int
                        - *((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                            .offset(v2 as isize) as ::core::ffi::c_int)
                        as uint16)
                        < rect.h
                {
                    return 1 as uint8;
                }
            }
        }
        v1 += 2 as ::core::ffi::c_int;
    }
    0 as uint8
}
unsafe fn Eproj_CheckForEnemyCollisionWithRect(k: uint16, rect: Rect16U, game: &mut Game) -> uint8 {
    let v1: *mut EnemyData = gEnemyData(k as ::core::ffi::c_int, game);
    ((abs16(((*v1).x_pos as ::core::ffi::c_int - rect.x as ::core::ffi::c_int) as uint16)
        as ::core::ffi::c_int
        - (*v1).x_width as ::core::ffi::c_int)
        < rect.w as ::core::ffi::c_int
        && (abs16(((*v1).y_pos as ::core::ffi::c_int - rect.y as ::core::ffi::c_int) as uint16)
            as ::core::ffi::c_int
            - (*v1).y_height as ::core::ffi::c_int)
            < rect.h as ::core::ffi::c_int) as uint8
}
unsafe fn Eproj_CheckForCollisionWithSamus(v0: uint16, game: &mut Game) -> uint8 {
    let rect: Rect16U = Eproj_GetCollDetectRect(v0, game);
    let v1: uint16 = abs16(
        (ram_word!(game, samus_x_pos) as ::core::ffi::c_int - rect.x as ::core::ffi::c_int)
            as uint16,
    );
    let mut v2: bool = v1 < ram_word!(game, samus_x_radius);
    let v3: uint16 = (v1 as ::core::ffi::c_int
        - ram_word!(game, samus_x_radius) as ::core::ffi::c_int) as uint16;
    let result: uint8 = 0 as uint8;
    if v2 || v3 < rect.w {
        let v4: uint16 = abs16(
            (ram_word!(game, samus_y_pos) as ::core::ffi::c_int - rect.y as ::core::ffi::c_int)
                as uint16,
        );
        v2 = v4 < ram_word!(game, samus_y_radius);
        let v5: uint16 = (v4 as ::core::ffi::c_int
            - ram_word!(game, samus_y_radius) as ::core::ffi::c_int)
            as uint16;
        if v2 || v5 < rect.h {
            return 1 as uint8;
        }
    }
    result
}
pub unsafe fn Math_MultBySin(a: uint16, r18: uint16) -> uint16 {
    Math_MultBySinCos(a, r18)
}
pub unsafe fn Math_MultByCos(a: uint16, r18: uint16) -> uint16 {
    Math_MultBySinCos(a, r18.wrapping_add(64))
}
pub unsafe fn Math_MultBySinCos(r38: uint16, a: uint16) -> uint16 {
    let r46: uint16 = kSinCosTable8bit_Sext[((a as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
        + 64 as ::core::ffi::c_int) as usize] as uint16;
    let prod: uint16 =
        ((r38 as uint32).wrapping_mul(abs16(r46) as uint32) >> 8 as ::core::ffi::c_int) as uint16;
    ((if r46 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(prod as ::core::ffi::c_int)
    } else {
        prod as ::core::ffi::c_int
    }) as uint16)
}
pub unsafe fn Eproj_MotherBrainsBlueRingLasers(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        1024 as uint16;
    let r18: uint16 = ram_word!(game, cinematic_var8);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        Math_MultBySin(0x450 as uint16, r18);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        Math_MultByCos(0x450 as uint16, r18);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1)).x_pos
            as ::core::ffi::c_int
            + 10 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1)).y_pos
            as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as uint16;
    sub_86C320(j, game);
}
pub unsafe fn sub_86C320(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1)).x_pos
            as ::core::ffi::c_int
            + 10 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1)).y_pos
            as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Eproj_MoveToBlueRingSpawnPosition(k: uint16, game: &mut Game) {
    let mut v5: int16 = 0;
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) != 0
    {
        let ref mut fresh83 = *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset(v1 as isize);
        *fresh83 = (*fresh83).wrapping_sub(1);
        sub_86C320(k, game);
    } else {
        MoveEprojWithVelocity(k, game);
        let t: uint8 = CheckForCollisionWithShitroid_DoubleRet(k, game);
        if t as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
            return;
        }
        if t != 0 {
            let fresh84 = &raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x7800)
                as *mut ExtraEnemyRam7800)
                .offset(0))
            .kraid
            .field_28;
            fresh84.write_unaligned(fresh84.read_unaligned().wrapping_add(1));
            BlueRingContactExplosion(k, game);
            let v3: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x7800)
                as *mut ExtraEnemyRam7800)
                .offset(1))
            .kraid
            .kraid_healths_8ths[4 as ::core::ffi::c_int as usize];
            (*gExtraEnemyRam7800(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x7800) as *mut ExtraEnemyRam7800)
                    .offset(1))
                .kraid
                .kraid_healths_8ths[4 as ::core::ffi::c_int as usize]
                    as ::core::ffi::c_int,
                game,
            ))
            .kraid
            .kraid_healths_8ths[0 as ::core::ffi::c_int as usize] = 16 as uint16;
            let v4: *mut EnemyData = gEnemyData(v3 as ::core::ffi::c_int, game);
            v5 = ((*v4).health as ::core::ffi::c_int - 80 as ::core::ffi::c_int) as int16;
            if v5 < 0i16 {
                v5 = 0 as int16;
            }
            (*v4).health = v5 as uint16;
        } else if Eproj_CheckForCollisionWithSamus(k, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            BlueRingContactExplosion(k, game);
            Samus_DealDamage(SuitDamageDivision(0x50 as uint16, game), game);
            set_ram_word!(game.samus_invincibility_timer = 96 as uint16);
            set_ram_word!(game.samus_knockback_timer = 5 as uint16);
            set_ram_word!(
                game.knockback_x_dir = (((ram_word!(game, samus_x_pos) as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int) as int16)
                    >= 0i16) as uint16
            );
        } else if CheckForBlueRingCollisionWithRoom(k, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            Eproj_Earthqhake5(k, game);
        }
    };
}
pub unsafe fn CheckForCollisionWithShitroid_DoubleRet(v0: uint16, game: &mut Game) -> uint8 {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x7800) as *mut ExtraEnemyRam7800).offset(1))
        .kraid
        .kraid_healths_8ths[4 as ::core::ffi::c_int as usize]
        == 0
    {
        return 0 as uint8;
    }
    if (*gEnemyData(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x7800) as *mut ExtraEnemyRam7800).offset(1))
            .kraid
            .kraid_healths_8ths[4 as ::core::ffi::c_int as usize] as ::core::ffi::c_int,
        game,
    ))
    .health
        != 0
    {
        let rect: Rect16U = Eproj_GetCollDetectRect(v0, game);
        return Eproj_CheckForEnemyCollisionWithRect(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x7800) as *mut ExtraEnemyRam7800)
                .offset(1))
            .kraid
            .kraid_healths_8ths[4 as ::core::ffi::c_int as usize],
            rect,
            game,
        );
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    0xff as uint8
}
pub unsafe fn CheckForBlueRingCollisionWithRoom(k: uint16, game: &mut Game) -> uint8 {
    let mut v2: int16 = 0;
    let mut v3: int16 = 0;
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let result: uint8 = 1 as uint8;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        - 32 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
            < 0xd8u16
    {
        v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
            .offset(v1 as isize) as int16;
        if v2 >= 0i16 {
            v3 = (v2 as ::core::ffi::c_int - ram_word!(game, layer1_x_pos) as ::core::ffi::c_int)
                as int16;
            if v3 >= 0i16
                && v3 as ::core::ffi::c_int - 248 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
            {
                return 0 as uint8;
            }
        }
    }
    result
}
unsafe fn Eproj_GetCollDetectRect(k: uint16, game: &mut Game) -> Rect16U {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let rect: Rect16U = Rect16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize),
        w: *(((&raw mut game.g_ram as *mut uint8).offset(0x1bb3) as *mut uint16).offset(v1 as isize)
            as *mut uint16 as *mut uint8)
            .offset(0) as uint16,
        h: *(((&raw mut game.g_ram as *mut uint8).offset(0x1bb3) as *mut uint16).offset(v1 as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as uint16,
    };
    rect
}
pub unsafe fn Eproj_Earthqhake5(k: uint16, game: &mut Game) {
    set_ram_word!(game.earthquake_timer = 10 as uint16);
    set_ram_word!(game.earthquake_type = 5 as uint16);
    BlueRingContactExplosion(k, game);
}
pub unsafe fn BlueRingContactExplosion(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, 3 as uint16, game);
    QueueSfx3_Max6(0x13 as uint16, game);
    sub_86C42E(k, game);
}
pub unsafe fn sub_86C42E(k: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
}
unsafe fn EprojInit_MotherBrainBomb(j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16 as *mut uint8)
        .offset(j as ::core::ffi::c_int as isize) = ram_word!(game, cinematic_var8) as uint8;
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        224 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1)).x_pos
            as ::core::ffi::c_int
            + 12 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1)).y_pos
            as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        1024 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        112 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    let fresh10 = &raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x7800)
        as *mut ExtraEnemyRam7800)
        .offset(1))
    .kraid
    .kraid_mouth_flags;
    fresh10.write_unaligned(fresh10.read_unaligned().wrapping_add(1));
}
pub unsafe fn MotherBrainBomb_Bomb_CollDetect_DoubleRet(k: uint16, game: &mut Game) -> uint8 {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let rect: Rect16U = Rect16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize),
        w: *(((&raw mut game.g_ram as *mut uint8).offset(0x1bb3) as *mut uint16).offset(v1 as isize)
            as *mut uint16 as *mut uint8)
            .offset(0) as uint16,
        h: *(((&raw mut game.g_ram as *mut uint8).offset(0x1bb3) as *mut uint16).offset(v1 as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as uint16,
    };
    if Eproj_CheckForBombCollisionWithRect(rect, game) == 0 {
        return 0 as uint8;
    }
    let fresh2 = &raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x7800)
        as *mut ExtraEnemyRam7800)
        .offset(1))
    .kraid
    .kraid_mouth_flags;
    fresh2.write_unaligned(fresh2.read_unaligned().wrapping_sub(1));
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, 9 as uint16, game);
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize),
    };
    SpawnEnemyDrops(addr_kEnemyDef_EC3F as uint16, k, 0 as uint16, game);
    1 as uint8
}
pub unsafe fn MoveMotherBrainBomb(k: uint16, a: uint16, game: &mut Game) -> uint8 {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize);
    *fresh16 = (*fresh16 as ::core::ffi::c_int + a as ::core::ffi::c_int) as uint16;
    MoveEprojWithVelocity(k, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize)
        as ::core::ffi::c_int
        - 240 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
            -(*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize)
        as ::core::ffi::c_int
        - 208 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        return 0 as uint8;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        208 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        (if *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize)
            as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            -(*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int)
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize)
                as ::core::ffi::c_int
        }) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        -(512 as ::core::ffi::c_int) as uint16;
    1 as uint8
}
pub unsafe fn sub_86C605(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_23 = 0 as uint16;
    (*E).mbn_var_25 = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        1024 as uint16;
    let ref mut fresh8 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize);
    *fresh8 = ((*E).base.x_pos as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_24 = *fresh8;
    let x: uint16 = (ram_word!(game, samus_x_pos) as ::core::ffi::c_int
        - (*E).mbn_var_24 as ::core::ffi::c_int) as uint16;
    let ref mut fresh9 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize);
    *fresh9 = ((*E).base.y_pos as ::core::ffi::c_int - 48 as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_26 = *fresh9;
    let y: uint16 = (ram_word!(game, samus_y_pos) as ::core::ffi::c_int
        - (*E).mbn_var_26 as ::core::ffi::c_int) as uint16;
    let r18: ::core::ffi::c_int = -(CalculateAngleFromXY(x, y, game) as ::core::ffi::c_int
        + 0x80 as ::core::ffi::c_int) as uint8
        as ::core::ffi::c_int;
    (*E).mbn_var_29 = r18 as uint16;
    (*E).mbn_var_27 = Math_MultBySin(0xc00 as uint16, r18 as uint16);
    (*E).mbn_var_28 = Math_MultByCos(0xc00 as uint16, r18 as uint16);
}
unsafe fn EprojInit_MotherBrainDeathBeemFired(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (*E).mbn_var_24;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize) =
        (*E).mbn_var_23;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (*E).mbn_var_26;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize) =
        (*E).mbn_var_25;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        (*E).mbn_var_27;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        (*E).mbn_var_28;
    MoveEprojWithVelocity(j, game);
    (*E).mbn_var_24 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize);
    (*E).mbn_var_23 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize);
    (*E).mbn_var_26 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize);
    (*E).mbn_var_25 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize);
    let r18: uint16 = (*(&raw mut (*E).mbn_var_29 as *mut uint8).offset(0) as ::core::ffi::c_int
        + NextRandom(game) as ::core::ffi::c_int) as uint8 as uint16;
    let rv: uint16 = NextRandom(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        Math_MultBySin(
            (rv as ::core::ffi::c_int & 0x700 as ::core::ffi::c_int) as uint16,
            r18,
        );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        Math_MultByCos(
            (ram_word!(game, random_number) as ::core::ffi::c_int & 0x700 as ::core::ffi::c_int)
                as uint16,
            r18,
        );
    MoveEprojWithVelocity(j, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        - 34 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            - 206 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            - 238 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
        game.eproj_spawn_pt = Point16U {
            x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v1 as isize),
            y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v1 as isize),
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as uint16,
            0x1d as uint16,
            game,
        );
        QueueSfx3_Max6(0x13 as uint16, game);
        set_ram_word!(game.earthquake_timer = 10 as uint16);
        set_ram_word!(game.earthquake_type = 5 as uint16);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                & 3 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
    };
}
pub unsafe fn SpawnMotherBrainDeathBeam(x: uint16, game: &mut Game) {
    SpawnEprojWithRoomGfx(
        addr_kEproj_MotherBrainDeathBeamFired as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset((x as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
        game,
    );
}
pub unsafe fn EprojPreInstr_MotherBrainRainbowBeam(k: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1)).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1)).y_pos;
}
unsafe fn EprojInit_MotherBrainRainbowBeam(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    EprojPreInstr_MotherBrainRainbowBeam(j, game);
}
static kEprojInit_MotherBrainsDrool: [int16; 12] = [
    6 as int16,
    0x14 as int16,
    0xe as int16,
    0x12 as int16,
    8 as int16,
    0x17 as int16,
    0xa as int16,
    0x13 as int16,
    0xb as int16,
    0x19 as int16,
    0xc as int16,
    0x12 as int16,
];
pub unsafe fn EprojPreInstr_C84D(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: ::core::ffi::c_int = (4 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1)).x_pos
            as ::core::ffi::c_int
            + kEprojInit_MotherBrainsDrool[v2 as usize] as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1)).y_pos
            as ::core::ffi::c_int
            + kEprojInit_MotherBrainsDrool[(v2 + 1 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
}
unsafe fn EprojInit_MotherBrainsDrool(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        ram_word!(game, cinematic_var8);
    EprojPreInstr_C84D(j, game);
}
unsafe fn EprojInit_MotherBrainsDrool_Falling(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh79 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize);
    *fresh79 = (*fresh79 as ::core::ffi::c_int + 12 as ::core::ffi::c_int) as uint16;
    if MoveEprojWithVelocityY(k, game) >= 0xd7u16 {
        let ref mut fresh80 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
            .offset(v1 as isize);
        *fresh80 = (*fresh80 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
            addr_off_86C8E1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    }
}
pub unsafe fn EprojInstr_Add12ToY(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    let ref mut fresh27 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh27 = (*fresh27 as ::core::ffi::c_int + 12 as ::core::ffi::c_int) as uint16;
    epjp
}
pub unsafe fn EprojPreInit_MotherBrainsDeathExplosion_0(k: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).y_pos
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
}
unsafe fn EprojInit_MotherBrainsDeathExplosion(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86c929)) as *const uint8 as *mut uint16)
            .offset(ram_word!(game, cinematic_var8) as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
        1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        game.eproj_spawn_pt.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        game.eproj_spawn_pt.y;
    EprojPreInit_MotherBrainsDeathExplosion_0(j, game);
}
unsafe fn EprojInit_MotherBrainsRainbowBeamExplosion(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        game.eproj_spawn_pt.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (ram_word!(game, samus_x_pos) as ::core::ffi::c_int
            + game.eproj_spawn_pt.x as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        game.eproj_spawn_pt.y;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (ram_word!(game, samus_y_pos) as ::core::ffi::c_int
            + game.eproj_spawn_pt.y as ::core::ffi::c_int) as uint16;
}
pub unsafe fn EprojPreInstr_MotherBrainsRainbowBeamExplosion(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (ram_word!(game, samus_x_pos) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (ram_word!(game, samus_y_pos) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
}
unsafe fn EprojInit_MotherBrainEscapeDoorParticles(j: uint16, game: &mut Game) {
    static kEprojInit_MotherBrainEscapeDoorParticles_X: [int16; 16] = [
        0 as int16,
        -(0x20 as ::core::ffi::c_int) as int16,
        0 as int16,
        -(0x18 as ::core::ffi::c_int) as int16,
        0 as int16,
        -(0x10 as ::core::ffi::c_int) as int16,
        0 as int16,
        -(8 as ::core::ffi::c_int) as int16,
        0 as int16,
        0 as int16,
        0 as int16,
        8 as int16,
        0 as int16,
        0x10 as int16,
        0 as int16,
        0x18 as int16,
    ];
    static kEprojInit_MotherBrainEscapeDoorParticles_Xvel: [int16; 16] = [
        0x500 as int16,
        -(0x200 as ::core::ffi::c_int) as int16,
        0x500 as int16,
        -(0x100 as ::core::ffi::c_int) as int16,
        0x500 as int16,
        -(0x100 as ::core::ffi::c_int) as int16,
        0x500 as int16,
        -(0x80 as ::core::ffi::c_int) as int16,
        0x500 as int16,
        -(0x80 as ::core::ffi::c_int) as int16,
        0x500 as int16,
        0x80 as int16,
        0x500 as int16,
        -(0x100 as ::core::ffi::c_int) as int16,
        0x500 as int16,
        0x200 as int16,
    ];
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    let v2: ::core::ffi::c_int = 4 as ::core::ffi::c_int
        * ram_word!(game, cinematic_var8) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (kEprojInit_MotherBrainEscapeDoorParticles_X[v2 as usize] as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (kEprojInit_MotherBrainEscapeDoorParticles_X[(v2 + 1 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int
            + 128 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        kEprojInit_MotherBrainEscapeDoorParticles_Xvel[v2 as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        kEprojInit_MotherBrainEscapeDoorParticles_Xvel[(v2 + 1 as ::core::ffi::c_int) as usize]
            as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        32 as uint16;
}
pub unsafe fn EprojPreInstr_MotherBrainsExplodedDoorParticles(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: int16 = (abs16(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize),
    ) as ::core::ffi::c_int
        - 16 as ::core::ffi::c_int) as int16;
    if v2 < 0i16 {
        v2 = 0 as int16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        (if *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            -(v2 as ::core::ffi::c_int)
        } else {
            v2 as ::core::ffi::c_int
        }) as uint16;
    let ref mut fresh76 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize);
    *fresh76 = (*fresh76 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    MoveEprojWithVelocity(k, game);
    let ref mut fresh77 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize);
    *fresh77 = (*fresh77).wrapping_sub(1);
    if (*fresh77 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
        let ref mut fresh78 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
            .offset(v1 as isize);
        *fresh78 = (*fresh78 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
        game.eproj_spawn_pt = Point16U {
            x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v1 as isize),
            y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v1 as isize),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, 9 as uint16, game);
    }
}
unsafe fn EprojInit_MotherBrainPurpleBreathBig(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1)).x_pos
            as ::core::ffi::c_int
            + 6 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1)).y_pos
            as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as uint16;
}
unsafe fn EprojInit_MotherBrainPurpleBreathSmall(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1)).x_pos
            as ::core::ffi::c_int
            + 6 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(1)).y_pos
            as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as uint16;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x7800) as *mut ExtraEnemyRam7800).offset(1))
        .kraid
        .kraid_hurt_frame = 1 as uint16;
}
pub unsafe fn EprojInstr_MotherBrainPurpleBreathIsActive(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    (*((&raw mut game.g_ram as *mut uint8).offset(0x7800) as *mut ExtraEnemyRam7800).offset(1))
        .kraid
        .kraid_hurt_frame = 0 as uint16;
    epjp
}
pub unsafe fn EprojPreInstr_TimeBombSetJapaneseText(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        192 as uint16;
}
unsafe fn EprojInit_TimeBombSetJapaneseText(j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    EprojPreInstr_TimeBombSetJapaneseText(j, game);
}
unsafe fn EprojInit_MotherBrainTubeFalling(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        3584 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        game.eproj_spawn_pt.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        game.eproj_spawn_pt.y;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        (0x86cbea as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CallMotherBrainTubeFallingFunc(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        8834026 => {
            MotherBrainTubeFallingFunc_GenerateExplosion(k, game);
        }
        8834056 => {
            MotherBrainTubeFallingFunc_Falling(k, game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn EprojPreInstr_MotherBrainTubeFalling(k: uint16, game: &mut Game) {
    CallMotherBrainTubeFallingFunc(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            | 0x860000 as ::core::ffi::c_int) as uint32,
        k,
        game,
    );
}
pub unsafe fn MotherBrainTubeFallingFunc_GenerateExplosion(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize),
        y: (*((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int) as uint16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, 9 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        (0x86cc08 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    MotherBrainTubeFallingFunc_Falling(k, game);
}
pub unsafe fn MotherBrainTubeFallingFunc_Falling(k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh75 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize);
    *fresh75 = (*fresh75 as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
    v2 = MoveEprojWithVelocityY(k, game) as int16;
    if v2 as ::core::ffi::c_int - 208 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
        game.eproj_spawn_pt = Point16U {
            x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v1 as isize),
            y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v1 as isize),
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as uint16,
            0xc as uint16,
            game,
        );
    }
}
static kEprojInit_MotherBrainGlassShatteringShard_X: [int16; 3] = [
    8 as int16,
    -(40 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
];
static kEprojInit_MotherBrainGlassShatteringShard_Y: [int16; 3] =
    [32 as int16, 32 as int16, 32 as int16];
unsafe fn EprojInit_MotherBrainGlassShatteringShard(j: uint16, game: &mut Game) {
    let v1: uint16 = (2 as ::core::ffi::c_int * NextRandom(game) as ::core::ffi::c_int
        & 0x1fe as ::core::ffi::c_int) as uint16;
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) = v1;
    let v3: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        kSinCosTable8bit_Sext[(v3 + 64 as ::core::ffi::c_int) as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        (4 as ::core::ffi::c_int * kSinCosTable8bit_Sext[v3 as usize] as ::core::ffi::c_int)
            as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86ce41)) as *const uint8 as *mut uint16)
            .offset(
                ((v1 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
                    as uint16 as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v2 as isize) =
        1600 as uint16;
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    let v4: ::core::ffi::c_int =
        ram_word!(game, cinematic_var8) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        (kEprojInit_MotherBrainGlassShatteringShard_X[v4 as usize] as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int * ram_word!(game, plm_x_block) as ::core::ffi::c_int)
            as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        (kEprojInit_MotherBrainGlassShatteringShard_Y[v4 as usize] as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int * ram_word!(game, plm_y_block) as ::core::ffi::c_int)
            as uint16;
    let ref mut fresh5 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize);
    *fresh5 = (*fresh5 as ::core::ffi::c_int
        + ((NextRandom(game) as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
            - 8 as ::core::ffi::c_int)) as uint16;
    let ref mut fresh6 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize);
    *fresh6 = (*fresh6 as ::core::ffi::c_int
        + ((NextRandom(game) as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
            - 8 as ::core::ffi::c_int)) as uint16;
}
unsafe fn EprojInit_MotherBrainGlassShatteringSparkle(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int =
        ram_word!(game, cinematic_var8) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            + (NextRandom(game) as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
            - 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            + (NextRandom(game) as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
            - 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v2 as isize) =
        1600 as uint16;
}
pub unsafe fn EprojPreInstr_MotherBrainGlassShatteringShard(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        amt as uint32,
    );
    amt = (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
        as int16 as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        amt as uint32,
    );
    let v6: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize);
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        != 0i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
    } else {
        let ref mut fresh74 = *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset(v1 as isize);
        *fresh74 = (*fresh74 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
        if (NextRandom(game) as ::core::ffi::c_int & 0x420 as ::core::ffi::c_int) == 0i32 {
            SpawnEprojWithRoomGfx(
                addr_kEproj_MotherBrainGlassShatteringSparkle as uint16,
                k,
                game,
            );
        }
    };
}
unsafe fn EprojInit_KiHunterAcidSpitCommon(k: uint16, j: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        ((*gEnemyData(k as ::core::ffi::c_int, game)).y_pos as ::core::ffi::c_int
            - 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
}
unsafe fn EprojInit_KiHunterAcidSpitLeft(j: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        -(768 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        ((*gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).x_pos
            as ::core::ffi::c_int
            - 22 as ::core::ffi::c_int) as uint16;
    EprojInit_KiHunterAcidSpitCommon(ram_word!(game, cur_enemy_index), j, game);
}
unsafe fn EprojInit_KiHunterAcidSpitRight(j: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        768 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        ((*gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).x_pos
            as ::core::ffi::c_int
            + 22 as ::core::ffi::c_int) as uint16;
    EprojInit_KiHunterAcidSpitCommon(ram_word!(game, cur_enemy_index), j, game);
}
pub unsafe fn sub_86CFD5(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v1 as isize) =
        (0x86cff8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let ref mut fresh73 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize);
    *fresh73 = (*fresh73 as ::core::ffi::c_int - 19 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn sub_86CFE6(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v1 as isize) =
        (0x86cff8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let ref mut fresh72 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize);
    *fresh72 = (*fresh72 as ::core::ffi::c_int + 19 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn sub_86CFF8(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
            addr_off_86CF56 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    } else if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    } else {
        let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
            v2;
        if v2 as ::core::ffi::c_int - 512 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        {
            v2 = 512 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
            v2;
    };
}
unsafe fn EprojInit_KagosBugs(j: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v2 as isize) =
        game.cur_enemy_index();
    let enemy_index = game.cur_enemy_index();
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        game.enemy_state(enemy_index).x_pos();
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        game.enemy_state(enemy_index).y_pos();
    let v4: uint16 = ((game.random_number() as ::core::ffi::c_int & 7 as ::core::ffi::c_int)
        + 1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1bfb) as *mut uint16).offset(v2 as isize) = v4;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) =
        v4.wrapping_add(4);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v2 as isize) =
        ((0x86d0cai32) as u16);
}
pub unsafe fn EprojPreInstr_KagosBugs_Func1(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: uint16 = EprojState(&game.g_ram).slot_word(0x1aff, v1 as usize);
    if v2 != 0 {
        let v3: uint16 = v2.wrapping_sub(1);
        EprojStateMut(&mut game.g_ram).write(0x1aff, v1 as usize, v3);
        if v3 == 0 {
            QueueSfx2_Max6(0x6c as uint16, game);
        }
    }
}
pub unsafe fn EprojPreInstr_KagosBugs_Func2(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let enemy_index =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize);
    let v3: uint16 =
        abs16((game.enemy_state(enemy_index).x_pos()).wrapping_sub(game.enemy_projectile_x_pos(k)));
    if v3 as ::core::ffi::c_int - 23 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        EprojStateMut(&mut game.g_ram).set_property_bits(v1 as usize, 0x8000);
    }
}
pub unsafe fn EprojPreInstr_KagosBugs(k: uint16, game: &mut Game) {
    EprojPreInstr_KagosBugs_Func1(k, game);
    EprojPreInstr_KagosBugs_Func2(k, game);
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1bfb) as *mut uint16).offset(v1 as isize) != 0
    {
        let fresh71 = EprojStateMut(&mut game.g_ram).add_word(0x1bfb, v1 as usize, u16::MAX);
    } else {
        EprojStateMut(&mut game.g_ram).start_instruction(v1 as usize, addr_word_86D052 as uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v1 as isize) =
            ((0x86d0ebi32) as u16);
    };
}
static word_86D082: uint16 = 0xe0 as uint16;
pub unsafe fn EprojPreInstr_D0EC(k: uint16, game: &mut Game) {
    let current_block: u64;
    EprojPreInstr_KagosBugs_Func1(k, game);
    EprojPreInstr_KagosBugs_Func2(k, game);
    if EprojBlockCollisition_Horiz(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        current_block = 4781832012581146689;
    } else if EprojBlockCollisition_Vertical(k, game) != 0 {
        current_block = 4781832012581146689;
    } else {
        let v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        let v3: bool = ((word_86D082) as i16).wrapping_add((v2) as i16) < 0i16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (word_86D082).wrapping_add(v2);
        if !v3 {
            current_block = 15427931788582360902;
        } else {
            return;
        }
    }
    if current_block == 4781832012581146689 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 256 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        ((0x86d128i32) as u16);
    EprojStateMut(&mut game.g_ram)
        .start_instruction(usize::from(k >> 1), addr_word_86D04A as uint16);
}
pub unsafe fn EprojPreInstr_D128(v0: uint16, game: &mut Game) {
    EprojPreInstr_KagosBugs_Func1(v0, game);
    EprojPreInstr_KagosBugs_Func2(v0, game);
    if EprojBlockCollisition_Horiz(v0, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    } else if EprojBlockCollisition_Vertical(v0, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        let v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v1 as isize) =
            ((0x86d0ebi32) as u16);
        EprojStateMut(&mut game.g_ram).start_instruction(v1 as usize, addr_word_86D03C as uint16);
    } else {
        let ref mut fresh70 = *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh70 = (*fresh70).wrapping_add(word_86D082);
    };
}
static g_word_86D086: uint16 = 0x200 as uint16;
static g_word_86D084: uint16 = 0x30 as uint16;
pub unsafe fn EprojInstr_D15C(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    EprojPreInstr_KagosBugs_Func1(k, game);
    EprojPreInstr_KagosBugs_Func2(k, game);
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut t: uint16 = ((game.random_number() as ::core::ffi::c_int & 0x300 as ::core::ffi::c_int)
        + 2048 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        -(t as ::core::ffi::c_int) as uint16;
    let enemy_index =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v2 as isize);
    let t2: uint16 =
        (game.enemy_state(enemy_index).x_pos()).wrapping_sub(game.enemy_projectile_x_pos(k));
    if ((abs16(t2)) as i16).wrapping_sub((g_word_86D084) as i16) >= 0i16 {
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
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) = v4;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v2 as isize) =
        ((0x86d0eci32) as u16);
    epjp
}
pub unsafe fn EprojInstr_D1B6(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1bfb) as *mut uint16).offset(v1 as isize) =
        ((game.random_number() as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as u16)
            .wrapping_add(1u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v1 as isize) =
        ((0x86d0cai32) as u16);
    epjp
}
pub unsafe fn EprojInstr_D1C7(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    epjp
}
pub unsafe fn EprojInstr_D1CE(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
    };
    SpawnEnemyDrops(addr_kEnemyDef_E7FF as uint16, k, 0 as uint16, game);
    epjp
}
unsafe fn EprojInit_MaridiaFloatersSpikes(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    let v2: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (*v2).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (*v2).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        ram_word!(game, cinematic_var8);
}
static word_86D21A: [int16; 8] = [
    0 as int16,
    32 as int16,
    32 as int16,
    32 as int16,
    0 as int16,
    -(32 as ::core::ffi::c_int) as int16,
    -(32 as ::core::ffi::c_int) as int16,
    -(32 as ::core::ffi::c_int) as int16,
];
static word_86D22A: [int16; 8] = [
    -(32 as ::core::ffi::c_int) as int16,
    -(32 as ::core::ffi::c_int) as int16,
    0 as int16,
    32 as int16,
    32 as int16,
    32 as int16,
    0 as int16,
    -(32 as ::core::ffi::c_int) as int16,
];
pub unsafe fn EprojPreInstr_MaridiaFloatersSpikes(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh68 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize);
    *fresh68 = (*fresh68 as ::core::ffi::c_int
        + word_86D21A[*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset(v1 as isize) as usize] as ::core::ffi::c_int) as uint16;
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || {
            let ref mut fresh69 = *((&raw mut game.g_ram as *mut uint8).offset(0x1adb)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh69 = (*fresh69 as ::core::ffi::c_int
                + word_86D22A[*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                    .offset(v1 as isize) as usize] as ::core::ffi::c_int)
                as uint16;
            EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
                != 0
        }
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
            addr_off_86D218 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    }
}
unsafe fn EprojInit_WreckedShipRobotLaserCommon(k: uint16, j: uint16, game: &mut Game) {
    let v2: *mut EnemyData = gEnemyData(k as ::core::ffi::c_int, game);
    let v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v3 as isize) =
        ((*v2).y_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v3 as isize) =
        ((*v2).x_pos as ::core::ffi::c_int
            + (if *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v3 as isize) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                -(4 as ::core::ffi::c_int)
            } else {
                4 as ::core::ffi::c_int
            })) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v3 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v3 as isize) =
        0 as uint16;
    if (((*v2).x_width as ::core::ffi::c_int + (*v2).x_pos as ::core::ffi::c_int
        - ram_word!(game, layer1_x_pos) as ::core::ffi::c_int) as int16)
        >= 0i16
        && (((*v2).x_pos as ::core::ffi::c_int
            - (*v2).x_width as ::core::ffi::c_int
            - 257 as ::core::ffi::c_int
            - ram_word!(game, layer1_x_pos) as ::core::ffi::c_int) as int16)
            < 0i16
        && (((*v2).y_height as ::core::ffi::c_int + (*v2).y_pos as ::core::ffi::c_int
            - ram_word!(game, layer1_y_pos) as ::core::ffi::c_int) as int16)
            >= 0i16
        && (((*v2).y_pos as ::core::ffi::c_int
            - (*v2).y_height as ::core::ffi::c_int
            - 224 as ::core::ffi::c_int) as int16)
            < 0i16
    {
        QueueSfx2_Max6(0x67 as uint16, game);
    }
}
unsafe fn EprojInit_WreckedShipRobotLaserDown(j: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        (*gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).ai_var_A;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    EprojInit_WreckedShipRobotLaserCommon(ram_word!(game, cur_enemy_index), j, game);
}
unsafe fn EprojInit_WreckedShipRobotLaserHorizontal(j: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        (*gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).ai_var_A;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    EprojInit_WreckedShipRobotLaserCommon(ram_word!(game, cur_enemy_index), j, game);
}
unsafe fn EprojInit_WreckedShipRobotLaserUp(j: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        (*gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).ai_var_A;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        -(128 as ::core::ffi::c_int) as uint16;
    EprojInit_WreckedShipRobotLaserCommon(ram_word!(game, cur_enemy_index), j, game);
}
pub unsafe fn EprojPreInstr_WreckedShipRobotLaser(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    if EprojBlockCollisition_Horiz(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        || EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
    }
}
pub unsafe fn EprojInstr_AssignNewN00bTubeShardVelocity(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    NextRandom(game);
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        *(((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16 as *mut uint8).offset(1)
            as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        192 as uint16;
    epjp
}
pub unsafe fn EprojInstr_SetN00bTubeShardX(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if ram_word!(game, nmi_frame_counter_word) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                .offset(v2 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b6b) as *mut uint16).offset(v2 as isize) =
            *(epjp as *mut uint16);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
            (128 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int
                + 128 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b6b) as *mut uint16).offset(v2 as isize) =
            *(epjp.offset(2) as *mut uint16);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
        (epjp.offset_from(RomBankBase(0x86 as uint8, game)) as ::core::ffi::c_long
            + 4 as ::core::ffi::c_long) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v2 as isize) =
        1 as uint16;
    ::core::ptr::null::<uint8>()
}
pub unsafe fn EprojInstr_D62A(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    if ram_word!(game, nmi_frame_counter_word) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            -(4608 as ::core::ffi::c_int) as uint16;
    }
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b6b) as *mut uint16).offset(v2 as isize) =
        *(epjp as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
        (epjp.offset_from(RomBankBase(0x86 as uint8, game)) as ::core::ffi::c_long
            + 2 as ::core::ffi::c_long) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v2 as isize) =
        1 as uint16;
    ::core::ptr::null::<uint8>()
}
pub unsafe fn EprojInstr_SetXvelRandom(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    NextRandom(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = *(((&raw mut game
        .g_ram
        as *mut uint8)
        .offset(0x5e5)
        as *mut uint16
        as *mut uint8)
        .offset(1)
        as *mut uint16);
    epjp
}
unsafe fn EprojInit_N00bTubeCrack(j: uint16, game: &mut Game) {
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (16 as ::core::ffi::c_int * ram_word!(game, plm_x_block) as ::core::ffi::c_int
            + 96 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (16 as ::core::ffi::c_int * ram_word!(game, plm_y_block) as ::core::ffi::c_int
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
    0 as int16,
    -(8 as ::core::ffi::c_int) as int16,
];
static kEprojInit_N00bTubeShards_Y: [int16; 10] = [
    8 as int16,
    -(12 as ::core::ffi::c_int) as int16,
    -(26 as ::core::ffi::c_int) as int16,
    -(24 as ::core::ffi::c_int) as int16,
    -(32 as ::core::ffi::c_int) as int16,
    28 as int16,
    16 as int16,
    -(8 as ::core::ffi::c_int) as int16,
    -(24 as ::core::ffi::c_int) as int16,
    16 as int16,
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
    0 as int16,
    -(64 as ::core::ffi::c_int) as int16,
];
static kEprojInit_N00bTubeShards_Yvel: [int16; 10] = [
    320 as int16,
    -(256 as ::core::ffi::c_int) as int16,
    -(416 as ::core::ffi::c_int) as int16,
    -(288 as ::core::ffi::c_int) as int16,
    -(288 as ::core::ffi::c_int) as int16,
    448 as int16,
    576 as int16,
    -(96 as ::core::ffi::c_int) as int16,
    -(288 as ::core::ffi::c_int) as int16,
    384 as int16,
];
unsafe fn EprojInit_N00bTubeShards(j: uint16, game: &mut Game) {
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    let v1: ::core::ffi::c_int =
        ram_word!(game, cinematic_var8) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v2 as isize) =
        (kEprojInit_N00bTubeShards_X[v1 as usize] as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int * ram_word!(game, plm_x_block) as ::core::ffi::c_int
            + 96 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        (kEprojInit_N00bTubeShards_Y[v1 as usize] as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int * ram_word!(game, plm_y_block) as ::core::ffi::c_int
            + 48 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86d760)) as *const uint8 as *mut uint16)
            .offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        kEprojInit_N00bTubeShards_Xvel[v1 as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        kEprojInit_N00bTubeShards_Yvel[v1 as usize] as uint16;
}
static kEprojInit_N00bTubeReleasedAirBubbles_X: [uint16; 6] = [
    40 as uint16,
    80 as uint16,
    104 as uint16,
    120 as uint16,
    152 as uint16,
    184 as uint16,
];
static kEprojInit_N00bTubeReleasedAirBubbles_Y: [uint16; 6] = [
    80 as uint16,
    72 as uint16,
    84 as uint16,
    32 as uint16,
    64 as uint16,
    84 as uint16,
];
unsafe fn EprojInit_N00bTubeReleasedAirBubbles(j: uint16, game: &mut Game) {
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    let v1: ::core::ffi::c_int =
        ram_word!(game, cinematic_var8) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v2 as isize) =
        (kEprojInit_N00bTubeReleasedAirBubbles_X[v1 as usize] as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int * ram_word!(game, plm_x_block) as ::core::ffi::c_int)
            as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        (kEprojInit_N00bTubeReleasedAirBubbles_Y[v1 as usize] as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int * ram_word!(game, plm_y_block) as ::core::ffi::c_int)
            as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        -(1280 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn sub_86D7BF(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
        != 0xee00u16
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v1 as isize);
    }
    if ram_word!(game, nmi_frame_counter_word) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
            -(4608 as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                .offset(v1 as isize);
    };
}
pub unsafe fn sub_86D7DE(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let amt: int32 = (192 as int16 as int32) << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        amt as uint32,
    );
}
pub unsafe fn EprojPreInstr_N00bTubeShards(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        amt as uint32,
    );
    amt = (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
        as int16 as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        amt as uint32,
    );
    Eproj_DeleteIfYposOutside(k, game);
}
pub unsafe fn sub_86D83D(k: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = (*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0x17e as ::core::ffi::c_int
        | 0x80 as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int;
    let v6: ::core::ffi::c_int =
        ram_word!(game, cinematic_var7) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = ((kSinCosTable8bit_Sext[(v2 + 64 as ::core::ffi::c_int) as usize]
        as ::core::ffi::c_int
        >> 2 as ::core::ffi::c_int) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v6 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v6 as isize)
            as *mut uint16,
        amt as uint32,
    );
    let ref mut fresh67 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v6 as isize);
    *fresh67 = (*fresh67 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    amt = (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v6 as isize)
        as int16 as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v6 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v6 as isize)
            as *mut uint16,
        amt as uint32,
    );
    Eproj_DeleteIfYposOutside(ram_word!(game, cinematic_var7), game);
}
pub unsafe fn sub_86D89F(k: uint16, game: &mut Game) {
    let v1: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0x17e as ::core::ffi::c_int
        | 0x80 as ::core::ffi::c_int) as uint16;
    let v2: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let amt: int32 = ((kSinCosTable8bit_Sext[(v2 + 64 as ::core::ffi::c_int) as usize]
        as ::core::ffi::c_int
        >> 2 as ::core::ffi::c_int) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let v5: ::core::ffi::c_int =
        ram_word!(game, cinematic_var7) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v5 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v5 as isize)
            as *mut uint16,
        amt as uint32,
    );
    let ref mut fresh66 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v5 as isize);
    *fresh66 = (*fresh66 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    sub_86D8DF(ram_word!(game, cinematic_var7), game);
}
pub unsafe fn sub_86D8DF(k: uint16, game: &mut Game) {
    let v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let amt: int32 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
        .offset(v5 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v5 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v5 as isize)
            as *mut uint16,
        amt as uint32,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v5 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v5 as isize);
}
pub unsafe fn sub_86D992(v0: uint16, game: &mut Game) {
    let v1: uint16 = EprojState(&game.g_ram).init_param_1();
    let v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) =
        EprojState(&game.g_ram).init_param_1();
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86d96a)) as *const uint8 as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    let enemy_index = game.cur_enemy_index();
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        game.enemy_state(enemy_index).x_pos();
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v2 as isize) =
        game.enemy_state(enemy_index).x_subpos();
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        game.enemy_state(enemy_index).y_pos();
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v2 as isize) =
        game.enemy_state(enemy_index).y_subpos();
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        -(512 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        512 as uint16;
    if EprojState(&game.g_ram).init_param_1() as ::core::ffi::c_int - 12 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
            -(384 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
            384 as uint16;
    }
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let split_amount = ((EprojState(&game.g_ram).slot_word(0x1adb, v1 as usize) as int16
        as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a93, 0x1a6f, v1 as usize, split_amount);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize)
            as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let split_amount = ((EprojState(&game.g_ram).slot_word(0x1adb, v1 as usize) as int16
        as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a4b, 0x1a27, v1 as usize, split_amount);
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let split_amount = ((EprojState(&game.g_ram).slot_word(0x1ab7, v1 as usize) as int16
        as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a4b, 0x1a27, v1 as usize, split_amount);
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1(k: uint16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, game);
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2(k: uint16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, game);
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1(k: uint16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, game);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2(k: uint16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, game);
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1(k: uint16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, game);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, game);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2(k: uint16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, game);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, game);
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1(k: uint16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, game);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, game);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2(k: uint16, game: &mut Game) {
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
pub fn EprojPreInstrHelper_SpikeShootingPlantSpikes_Func1(k: uint16, game: &mut Game) {
    if EprojPreInstrHelper_SpikeShootingPlantSpikes_Func2(k, game) != 0 {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    }
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes(k: uint16, game: &mut Game) {
    kEprojPreInstr_SpikeShootingPlantSpikes[(*((&raw mut game.g_ram as *mut uint8).offset(0x1aff)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(k, game);
    EprojPreInstrHelper_SpikeShootingPlantSpikes_Func1(k, game);
}
pub fn EprojPreInstrHelper_SpikeShootingPlantSpikes_Func2(k: uint16, game: &mut Game) -> uint16 {
    ((game
        .enemy_projectile_x_pos(k)
        .wrapping_sub(game.layer1_x_pos()) as int16)
        < 0i16
        || (game
            .layer1_x_pos()
            .wrapping_add(256)
            .wrapping_sub(game.enemy_projectile_x_pos(k)) as int16)
            < 0i16
        || (game
            .enemy_projectile_y_pos(k)
            .wrapping_sub(game.layer1_y_pos()) as int16)
            < 0i16
        || (game
            .layer1_y_pos()
            .wrapping_add(256)
            .wrapping_sub(game.enemy_projectile_y_pos(k)) as int16)
            < 0i16) as uint16
}
unsafe fn EprojInit_DBF2(j: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
        addr_word_86DB0C as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) =
        (0x86db62 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if ram_word!(game, cinematic_var8) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) =
            (0x86db8c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    let v3: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        (*v3).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v2 as isize) =
        (*v3).x_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        ((*v3).y_pos as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v2 as isize) =
        (*v3).y_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        -(256 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        256 as uint16;
}
pub unsafe fn EprojPreInstr_DBF2_Func1(k: uint16, game: &mut Game) {
    if EprojPreInstrHelper_DBF2_Func2(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn EprojPreInstr_DBF2(k: uint16, game: &mut Game) {
    CallEprojPreInstr(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            | 0x860000 as ::core::ffi::c_int) as uint32,
        k,
        game,
    );
    EprojPreInstr_DBF2_Func1(k, game);
}
pub unsafe fn EprojPreInstr_DBF2_MoveX1(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
            as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn EprojPreInstr_DBF2_MoveX2(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize)
            as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn EprojPreInstrHelper_DBF2_Func2(k: uint16, game: &mut Game) -> uint16 {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (((*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        - ram_word!(game, layer1_x_pos) as ::core::ffi::c_int) as int16)
        < 0i16
        || ((ram_word!(game, layer1_x_pos) as ::core::ffi::c_int + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16)
            < 0i16
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - ram_word!(game, layer1_y_pos) as ::core::ffi::c_int) as int16)
            < 0i16
        || ((ram_word!(game, layer1_y_pos) as ::core::ffi::c_int + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16)
            < 0i16) as uint16
}
pub unsafe fn EprojInstr_DC5A(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 12288 as uint16;
    epjp
}
pub unsafe fn EprojInstr_SpawnEnemyDrops_0(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize),
    };
    SpawnEnemyDrops(addr_kEnemyDef_DF7F as uint16, k, 0 as uint16, game);
    epjp
}
pub unsafe fn EprojInstr_SpawnSporesEproj(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_Spores as uint16, 0 as uint16, game);
    epjp
}
unsafe fn EprojInit_Spores(j: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh4 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize);
    *fresh4 = game.eproj_spawn_pt.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v2 as isize) =
        *fresh4;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        game.eproj_spawn_pt.y;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v2 as isize) =
        512 as uint16;
}
static word_86DCB9: [int16; 5] = [
    -(64 as ::core::ffi::c_int) as int16,
    -(56 as ::core::ffi::c_int) as int16,
    -(48 as ::core::ffi::c_int) as int16,
    -(40 as ::core::ffi::c_int) as int16,
    -(32 as ::core::ffi::c_int) as int16,
];
unsafe fn EprojInit_SporeSpawnStalk(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).y_pos
            as ::core::ffi::c_int
            + word_86DCB9[ram_word!(game, cinematic_var8) as usize] as ::core::ffi::c_int)
            as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos;
}
pub unsafe fn sub_86DCC3(v0: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).y_pos
            as ::core::ffi::c_int
            - 96 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos;
}
static kEprojInit_SporeSpawners_X: [int16; 4] =
    [0x20 as int16, 0x60 as int16, 0xa0 as int16, 0xe0 as int16];
unsafe fn EprojInit_SporeSpawners(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        kEprojInit_SporeSpawners_X[ram_word!(game, cinematic_var8) as usize] as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        520 as uint16;
}
pub unsafe fn EprojPreInstr_Spores(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: uint16 = *(((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(0) as uint16;
    let mut r18: uint16 = SignExtend8(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86dd6c)) as *const uint8 as *mut uint8)
            .offset(
                *(((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                    .offset(v1 as isize) as *mut uint16 as *mut uint8)
                    .offset(0) as isize,
            ) as uint16,
    );
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int)
        != 0i32
    {
        r18 = -(r18 as ::core::ffi::c_int) as uint16;
    }
    let ref mut fresh65 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize);
    *fresh65 = (*fresh65 as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
    r18 = SignExtend8(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86dd6c)) as *const uint8 as *mut uint8)
            .offset((v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as uint16,
    );
    let v3: uint16 = (r18 as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
        + r18 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) = v3;
    if v3 as ::core::ffi::c_int - 768 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16).offset(v1 as isize) =
            0 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        (*(((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize)
            as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as uint8 as uint16;
}
pub unsafe fn EprojPreInstr_SporeSpawners(k: uint16, game: &mut Game) {
    if ram_word!(game, kraid_unk9000) == 0 {
        let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize)
            == 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16)
                .offset(v1 as isize) = addr_word_86DC06 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16)
                .offset(v1 as isize) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                .offset(v1 as isize) =
                (NextRandom(game) as ::core::ffi::c_int & 0x1ff as ::core::ffi::c_int) as uint16;
        }
        let ref mut fresh64 = *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
            .offset(v1 as isize);
        *fresh64 = (*fresh64).wrapping_sub(1);
    }
}
unsafe fn EprojInit_NamiFuneFireball(j: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
        addr_word_86DE96 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) =
        (0x86df40 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if ram_word!(game, cinematic_var8) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
            addr_word_86DEA6 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) =
            (0x86df6a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    let v3: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        (*v3).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v2 as isize) =
        (*v3).x_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        (*v3).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v2 as isize) =
        (*v3).y_subpos;
    if ((*v3).parameter_1 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) != 0i32 {
        let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
            .offset(v2 as isize);
        *fresh3 = (*fresh3 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    }
    let v4: ::core::ffi::c_int = (4 as ::core::ffi::c_int
        * *(&raw mut (*v3).parameter_2 as *mut uint8).offset(0) as ::core::ffi::c_int)
        as uint16 as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86deb6)) as *const uint8 as *mut uint16)
            .offset(v4 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86deb6)) as *const uint8 as *mut uint16)
            .offset((v4 + 1 as ::core::ffi::c_int) as isize);
}
unsafe fn EprojInit_NamiFuneFireball_MoveX1(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
            as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
unsafe fn EprojInit_NamiFuneFireball_MoveX2(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize)
            as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn CallNamiFuneFireballFunc(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        8838976 => {
            EprojInit_NamiFuneFireball_MoveX1(k, game);
        }
        8839018 => {
            EprojInit_NamiFuneFireball_MoveX2(k, game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn EprojPreInstr_NamiFuneFireball(v0: uint16, game: &mut Game) {
    CallNamiFuneFireballFunc(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            | 0x860000 as ::core::ffi::c_int) as uint32,
        v0,
        game,
    );
    Eproj_NamiFuneFireball_After(v0, game);
}
pub unsafe fn Eproj_NamiFuneFireball_After(v0: uint16, game: &mut Game) {
    if EprojPreInstrHelper_DBF2_Func2(v0, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn sub_86DFA0(k: uint16, game: &mut Game) -> uint16 {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (((*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        - ram_word!(game, layer1_x_pos) as ::core::ffi::c_int) as int16)
        < 0i16
        || ((ram_word!(game, layer1_x_pos) as ::core::ffi::c_int + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16)
            < 0i16) as uint16
}
pub unsafe fn EprojInstr_DFEA(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize),
    };
    SpawnEnemyDrops(addr_kEnemyDef_E83F as uint16, k, 0 as uint16, game);
    epjp
}
unsafe fn EprojInit_LavaThrownByLavaman(j: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
        addr_word_86DFD8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) =
        (0x86e050 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if ram_word!(game, cinematic_var8) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
            addr_word_86DFDE as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) =
            (0x86e07a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    let v3: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        (*v3).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v2 as isize) =
        (*v3).x_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        ((*v3).y_pos as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v2 as isize) =
        (*v3).y_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        -(768 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        768 as uint16;
}
pub unsafe fn CallLavamanFunc(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        8839248 => {
            Eproj_LavaThrownByLavaman_MoveX1(k, game);
        }
        8839290 => {
            Eproj_LavaThrownByLavaman_MoveX2(k, game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn sub_86E049(v0: uint16, game: &mut Game) {
    CallLavamanFunc(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            | 0x860000 as ::core::ffi::c_int) as uint32,
        v0,
        game,
    );
    sub_86E0A4(v0, game);
}
pub unsafe fn Eproj_LavaThrownByLavaman_MoveX1(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
            as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn Eproj_LavaThrownByLavaman_MoveX2(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize)
            as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn sub_86E0A4(v0: uint16, game: &mut Game) {
    if sub_86E0B0(v0, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn sub_86E0B0(k: uint16, game: &mut Game) -> uint16 {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (((*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        - ram_word!(game, layer1_x_pos) as ::core::ffi::c_int) as int16)
        < 0i16
        || ((ram_word!(game, layer1_x_pos) as ::core::ffi::c_int + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16)
            < 0i16
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - ram_word!(game, layer1_y_pos) as ::core::ffi::c_int) as int16)
            < 0i16
        || ((ram_word!(game, layer1_y_pos) as ::core::ffi::c_int + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16)
            < 0i16) as uint16
}
unsafe fn EprojInit_DustCloudOrExplosion(v0: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86e42c)) as *const uint8 as *mut uint16)
            .offset(EprojState(&game.g_ram).init_param_1() as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        game.eproj_spawn_pt.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        game.eproj_spawn_pt.y;
}
unsafe fn EprojInit_EyeDoorSmoke(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86e42c)) as *const uint8 as *mut uint16)
            .offset(ram_word!(game, cinematic_var8) as uint8 as isize);
    let v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let x: uint16 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x86e47e)) as *const uint8
        as *mut uint16)
        .offset((v2 + 2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
        + (*(game.g_rom.offset(crate::rtl::lorom_offset(0x86e47e)) as *const uint8 as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            & ram_word!(game, random_number) as ::core::ffi::c_int)) as uint16;
    let y: uint16 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x86e47e)) as *const uint8
        as *mut uint16)
        .offset((v2 + 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
        + (*(game.g_rom.offset(crate::rtl::lorom_offset(0x86e47e)) as *const uint8 as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            & ram_word!(game, random_number) as ::core::ffi::c_int >> 8 as ::core::ffi::c_int))
        as uint16;
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (x as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int
                * (2 as ::core::ffi::c_int * ram_word!(game, plm_x_block) as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (y as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int
                * (2 as ::core::ffi::c_int * ram_word!(game, plm_y_block) as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)) as uint16;
    NextRandom(game);
}
pub fn EprojPreInstr_DustCloudOrExplosion(k: uint16, game: &mut Game) {
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    }
}
pub unsafe fn EprojInstr_SetYVel(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(epjp as *mut uint16);
    epjp.offset(2)
}
unsafe fn EprojInit_SpawnedGate_Common(j: uint16, v1: uint16, game: &mut Game) {
    let v2: uint16 = ram_word!(game, plm_id);
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    let v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v3 as isize) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1c87) as *mut uint16)
            .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v3 as isize) =
        (16u16).wrapping_mul(ram_word!(game, plm_x_block));
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v3 as isize) = (v1
        as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int * ram_word!(game, plm_y_block) as ::core::ffi::c_int)
        as uint16;
}
unsafe fn EprojInit_SpawnedShotGate(j: uint16, game: &mut Game) {
    EprojInit_SpawnedGate_Common(j, 0 as uint16, game);
}
unsafe fn EprojInit_ClosedDownwardsShotGate(j: uint16, game: &mut Game) {
    EprojInit_SpawnedGate_Common(j, 0x40 as uint16, game);
}
unsafe fn EprojInit_ClosedUpwardsShotGate(j: uint16, game: &mut Game) {
    EprojInit_SpawnedGate_Common(j, -(0x40 as ::core::ffi::c_int) as uint16, game);
}
pub unsafe fn EprojPreInstr_E605(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x19df) as *mut uint16)
        .offset(v1 as isize))
    .wrapping_add(abs16(
        *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize),
    ));
    if v2 >= 0x1000 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        let fresh60 = EprojStateMut(&mut game.g_ram).add_word(0x1b47, v1 as usize, 1);
        let fresh61 = EprojStateMut(&mut game.g_ram).add_word(0x1b47, v1 as usize, 1);
        v2 = 0 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x19df) as *mut uint16).offset(v1 as isize) = v2;
    let ref mut fresh62 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize);
    *fresh62 = (*fresh62 as ::core::ffi::c_int
        + ((*(((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset(v1 as isize) as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int)) as uint16;
    let ref mut fresh63 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize);
    *fresh63 = (*fresh63).wrapping_add(
        (*(((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as int8) as u16,
    );
}
unsafe fn EprojInit_SaveStationElectricity(j: uint16, game: &mut Game) {
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) = (16
        as ::core::ffi::c_int
        * (ram_word!(game, plm_x_block) as ::core::ffi::c_int + 1 as ::core::ffi::c_int))
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) = (16
        as ::core::ffi::c_int
        * (ram_word!(game, plm_y_block) as ::core::ffi::c_int - 2 as ::core::ffi::c_int))
        as uint16;
}
fn CheckIfEprojIsOffScreen(k: uint16, game: &mut Game) -> uint16 {
    if (game
        .enemy_projectile_x_pos(k)
        .wrapping_sub(game.layer1_x_pos()) as int16)
        >= 0i16
        && ((game.enemy_projectile_x_pos(k) as ::core::ffi::c_int
            - (game.layer1_x_pos() as ::core::ffi::c_int + 256 as ::core::ffi::c_int))
            as int16)
            < 0i16
        && (game
            .enemy_projectile_y_pos(k)
            .wrapping_sub(game.layer1_y_pos()) as int16)
            >= 0i16
        && ((game.enemy_projectile_y_pos(k) as ::core::ffi::c_int
            - (game.layer1_y_pos() as ::core::ffi::c_int + 256 as ::core::ffi::c_int))
            as int16)
            < 0i16
    {
        return 0 as uint16;
    }
    1 as uint16
}
pub unsafe fn Eproj_FuncE722(k: uint16, game: &mut Game) -> uint16 {
    let mut v2: int16 = 0;
    let mut v3: int16 = 0;
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
        as int16;
    let result: uint16 = 1 as uint16;
    if v2 >= 0i16
        && v2 as ::core::ffi::c_int - 512 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    {
        v3 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
            .offset(v1 as isize) as int16;
        if v3 >= 0i16
            && v3 as ::core::ffi::c_int - 512 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
        {
            return 0 as uint16;
        }
    }
    result
}
pub unsafe fn Eproj_FuncE73E_MoveXY(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x97dc) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        + 64 as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int)
        != 0i32
    {
        AddToHiLo(
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            -((*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int) as uint32,
        );
    } else {
        AddToHiLo(
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                    .offset(v1 as isize) as uint32,
        );
    }
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x97dc) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        + 128 as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int)
        != 0i32
    {
        AddToHiLo(
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            -((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int) as uint32,
        );
    } else {
        AddToHiLo(
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize)
                as *mut uint16,
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v1 as isize) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                    .offset(v1 as isize) as uint32,
        );
    };
}
pub unsafe fn Eproj_AngleToSamus(j: uint16, r18: uint16, r20: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) = r18;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) = r20;
    let v2: uint16 = (64 as ::core::ffi::c_int
        - CalculateAngleFromXY(
            (ram_word!(game, samus_x_pos) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int) as uint16,
            (ram_word!(game, samus_y_pos) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int) as uint16,
            game,
        ) as ::core::ffi::c_int) as uint8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x97dc) as *mut uint16).offset(v1 as isize) = v2;
    let pt: Point32 = ConvertAngleToXy(v2, ram_word!(game, cinematic_var8), game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        (pt.x >> 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        pt.x as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        (pt.y >> 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
        pt.y as uint16;
}
unsafe fn EprojInit_BotwoonsBody(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    let v2: uint16 = (if (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData)
        .offset(0))
    .ai_var_A
        != 0u16
    {
        16 as ::core::ffi::c_int
    } else {
        48 as ::core::ffi::c_int
    }) as uint16;
    let v3: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x86e9f1)) as *const uint8
        as *mut uint16)
        .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) = v3;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) = v3;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) = v2;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        (0x86ea98 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let Ex: *mut ExtraEnemyRam7800 = gExtraEnemyRam7800(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).ai_var_A
            as ::core::ffi::c_int,
        game,
    );
    (*Ex).kraid.kraid_next = j;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf380) as *mut uint16).offset(v1 as isize) =
        2 as uint16;
    (*Ex).kraid.kraid_healths_4ths[0 as ::core::ffi::c_int as usize] = 1 as uint16;
}
pub unsafe fn CallBotwoonEprojFunc(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        8841880 => {
            Eproj_BotwoonsBody_Main(k, game);
        }
        8841972 => {
            Eproj_BotwonsBodyFunction_Dying(k, game);
        }
        8841988 => {
            Eproj_BotwonsBodyFunction_Dying2(k, game);
        }
        8842015 => {
            Eproj_BotwonsBodyFunction_DyingFalling(k, game);
        }
        8842131 => (),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn EprojPreInstr_BotwoonsBody(k: uint16, game: &mut Game) {
    if *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x8000) as *mut ExtraEnemyRam8000)
        .offset(0))
    .pad as *mut uint8)
        .offset(32) as *mut uint8 as *mut uint16)
        != 0
    {
        let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            == 0x86ea98 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) =
                (0x86eaf4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
    CallBotwoonEprojFunc(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            | 0x860000 as ::core::ffi::c_int) as uint32,
        k,
        game,
    );
}
pub unsafe fn Eproj_BotwoonsBody_Main(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x86e9f1)) as *const uint8
        as *mut uint16)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    if v2
        != *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize)
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
            v2;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize) =
            v2;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    }
    Eproj_BotwoonsBodyHurtFlashHandling1(k, game);
}
pub unsafe fn Eproj_BotwoonsBodyHurtFlashHandling1(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh58 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize);
    *fresh58 = (*fresh58 as ::core::ffi::c_int | 0xe00 as ::core::ffi::c_int) as uint16;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .flash_timer
        != 0
        && (ram_word!(game, random_enemy_counter) as ::core::ffi::c_int & 2 as ::core::ffi::c_int)
            != 0i32
    {
        let ref mut fresh59 = *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16)
            .offset(v1 as isize);
        *fresh59 = (*fresh59 as ::core::ffi::c_int & 0xf1ff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Eproj_BotwoonsBodyHurtFlashHandling2(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh55 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize);
    *fresh55 = (*fresh55 as ::core::ffi::c_int | 0xe00 as ::core::ffi::c_int) as uint16;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0))
        .flash_timer
        != 0
        && (ram_word!(game, random_enemy_counter) as ::core::ffi::c_int & 2 as ::core::ffi::c_int)
            != 0i32
    {
        let ref mut fresh56 = *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16)
            .offset(v1 as isize);
        *fresh56 = (*fresh56 as ::core::ffi::c_int & 0xf1ff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Eproj_BotwonsBodyFunction_Dying(v0: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize) =
        (4 as ::core::ffi::c_int * v0 as ::core::ffi::c_int + 96 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
        0xeb04 as uint16;
    Eproj_BotwonsBodyFunction_Dying2(v0, game);
}
pub unsafe fn Eproj_BotwonsBodyFunction_Dying2(v0: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh57 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v1 as isize);
    *fresh57 = (*fresh57).wrapping_add(1);
    if *fresh57 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
            (0x86eb1f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    Eproj_BotwoonsBodyHurtFlashHandling1(v0, game);
}
pub unsafe fn Eproj_BotwonsBodyFunction_DyingFalling(v0: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v3: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * ((*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const uint8 as *mut uint32)
            .offset((v3 >> 1 as ::core::ffi::c_int) as isize),
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        - 200 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let v7: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh54 = *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
            .offset(v7 as isize);
        *fresh54 = (*fresh54 as ::core::ffi::c_int + 192 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v7 as isize) =
            0 as uint16;
        Eproj_BotwoonsBodyHurtFlashHandling2(v0, game);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
            200 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
            (0x86eb93 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v1 as isize) =
            addr_word_86E208 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
            2560 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        QueueSmallExplosionSfx(game);
        if v0 == 10u16 {
            *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x8800)
                as *mut ExtraEnemyRam8800)
                .offset(0))
            .pad as *mut uint8)
                .offset(62) as *mut uint8 as *mut uint16) = 1 as uint16;
        }
    };
}
pub unsafe fn QueueSmallExplosionSfx(game: &mut Game) {
    QueueSfx2_Max6(0x24 as uint16, game);
}
unsafe fn EprojInit_BotwoonsSpit(j: uint16, game: &mut Game) {
    let v1: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        (*v1).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        (*v1).y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
        addr_word_86EBAE as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x97dc) as *mut uint16).offset(v2 as isize) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x60b) as uint16;
    let pt: Point32 = ConvertAngleToXy(
        *(&raw mut game.g_ram as *mut uint8).offset(0x60b) as uint16,
        ram_word!(game, cinematic_var8),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        (pt.x >> 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) =
        pt.x as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        (pt.y >> 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v2 as isize) =
        pt.y as uint16;
}
pub unsafe fn EprojPreInstr_BotwoonsSpit(k: uint16, game: &mut Game) {
    Eproj_FuncE73E_MoveXY(k, game);
    sub_86EC0C(k, game);
}
pub unsafe fn sub_86EC0C(k: uint16, game: &mut Game) {
    if sub_86EC18(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn sub_86EC18(k: uint16, game: &mut Game) -> uint16 {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (((*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        - ram_word!(game, layer1_x_pos) as ::core::ffi::c_int) as int16)
        < 0i16
        || ((ram_word!(game, layer1_x_pos) as ::core::ffi::c_int + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16)
            < 0i16
        || ((*((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - ram_word!(game, layer1_y_pos) as ::core::ffi::c_int) as int16)
            < 0i16
        || ((ram_word!(game, layer1_y_pos) as ::core::ffi::c_int + 256 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16)
            < 0i16) as uint16
}
unsafe fn EprojInit_YappingMawsBody(j: uint16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v3 as isize) =
        game.enemy_state(enemy_index).x_pos();
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v3 as isize) =
        game.enemy_state(enemy_index).y_pos();
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v3 as isize) =
        addr_word_86EC5C as uint16;
    if game.enemy_state(enemy_index).parameter_2() == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v3 as isize) =
            addr_word_86EC56 as uint16;
    }
    *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x8800) as *mut ExtraEnemyRam8800)
        .offset(0))
    .pad as *mut uint8)
        .offset(
            (game.cur_enemy_index() as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int
                    * *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x8800)
                        as *mut ExtraEnemyRam8800)
                        .offset(0))
                    .pad as *mut uint8)
                        .offset(
                            (game.cur_enemy_index() as ::core::ffi::c_int + 8 as ::core::ffi::c_int)
                                as isize,
                        ) as *mut uint8 as *mut uint16) as ::core::ffi::c_int) as uint16
                as isize,
        ) as *mut uint8 as *mut uint16) = j;
}
pub unsafe fn EprojInstr_ECE3(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    let x: uint16 = (game.enemy_projectile_x_pos(k) as ::core::ffi::c_int
        + (NextRandom(game) as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int)
        - 32 as ::core::ffi::c_int) as uint16;
    let y: uint16 = ((game.enemy_projectile_y_pos(k) as ::core::ffi::c_int
        + (((game.random_number()) & (0x3f00u16)) as ::core::ffi::c_int >> 8 as ::core::ffi::c_int))
        as u16)
        .wrapping_sub(32u16);
    CreateSpriteAtPos(x, y, *(epjp as *mut uint16), 0 as uint16, game);
    epjp.offset(2)
}
pub unsafe fn EprojInstr_ED17(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    let x: uint16 = (game.enemy_projectile_x_pos(k) as ::core::ffi::c_int
        + (NextRandom(game) as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
        - 16 as ::core::ffi::c_int) as uint16;
    let y: uint16 = ((game.enemy_projectile_y_pos(k) as ::core::ffi::c_int
        + (((game.random_number()) & (0x1f00u16)) as ::core::ffi::c_int >> 8 as ::core::ffi::c_int))
        as u16)
        .wrapping_sub(16u16);
    CreateSpriteAtPos(x, y, *(epjp as *mut uint16), 0 as uint16, game);
    epjp.offset(2)
}
pub unsafe fn EprojInstr_QueueSfx2_9(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max1(9 as uint16, game);
    epjp
}
pub unsafe fn EprojInstr_QueueSfx2_24(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max1(0x24 as uint16, game);
    epjp
}
pub unsafe fn EprojInstr_QueueSfx2_B(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    QueueSfx2_Max1(0xb as uint16, game);
    epjp
}
pub unsafe fn EprojInstr_EEAF(k: uint16, epjp: *const uint8, game: &mut Game) -> *const uint8 {
    let v2: uint16 = RandomDropRoutine(k, game);
    if k != 0
        && v2 as ::core::ffi::c_int - 6 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    {
        let v3: uint16 = (2u16).wrapping_mul(v2);
        let v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v4 as isize) =
            v3;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v4 as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x86ef04)) as *const uint8 as *mut uint16)
                .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v4 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v4 as isize) =
            400 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v4 as isize) =
            ((0x86efe0i32) as u16);
        let ref mut fresh25 = *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16)
            .offset(v4 as isize);
        *fresh25 = (*fresh25) & ((!(0x4000 as ::core::ffi::c_int)) as u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v4 as isize)
            as uintptr_t as *const uint8
    } else {
        let v6: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v6 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16).offset(v6 as isize) =
            12288 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v6 as isize) =
            ((0x86efdfi32) as u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v6 as isize) =
            0xeca3 as uint16;
        0xeca3 as ::core::ffi::c_int as uintptr_t as *const uint8
    }
}
pub unsafe fn EprojInstr_HandleRespawningEnemy(
    k: uint16,
    epjp: *const uint8,
    game: &mut Game,
) -> *const uint8 {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xf410) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) as int16)
        <= -2i16
    {
        RespawnEnemy(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xf410) as *mut uint16)
                .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize))
                & (0x7fffu16),
            game,
        );
    }
    epjp
}
unsafe fn EprojInit_F337(j: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        game.eproj_spawn_pt.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        game.eproj_spawn_pt.y;
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf3c8) as *mut uint16).offset(v2 as isize) =
        game.eproj_spawn_varE24;
    let v3: uint16 = RandomDropRoutine(j, game);
    if v3 != 0
        && v3 as ::core::ffi::c_int - 6 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    {
        let v4: uint16 = (2u16).wrapping_mul(v3);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) =
            v4;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x86ef04)) as *const uint8 as *mut uint16)
                .offset((v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v2 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v2 as isize) =
            400 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xf410) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            -(1 as ::core::ffi::c_int) as uint16;
    } else {
        let v5: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v5 as isize) =
            addr_word_86ECA3 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16).offset(v5 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1bd7) as *mut uint16).offset(v5 as isize) =
            12288 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a03) as *mut uint16).offset(v5 as isize) =
            (0x86efdf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
unsafe fn EprojInit_EnemyDeathExplosion(j: uint16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        game.enemy_state(enemy_index).x_pos();
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        game.enemy_state(enemy_index).y_pos();
    *((&raw mut game.g_ram as *mut uint8).offset(0xf410) as *mut uint16).offset(v2 as isize) =
        game.cur_enemy_index();
    if (game.enemy_state(enemy_index).properties() as ::core::ffi::c_int
        & 0x4000 as ::core::ffi::c_int)
        != 0i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xf410) as *mut uint16).offset(v2 as isize) =
            (game.cur_enemy_index()) | (0x8000u16);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xf3c8) as *mut uint16).offset(v2 as isize) =
        game.enemy_state(enemy_index).enemy_ptr();
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    {
        let instruction: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x86efd5))
            as *const uint8 as *mut uint16)
            .offset(EprojState(&game.g_ram).init_param_1() as isize);
        EprojStateMut(&mut game.g_ram).start_instruction(v2 as usize, instruction);
    }
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
pub unsafe fn EprojPreInstr_Pickup(k: uint16, game: &mut Game) {
    let mut v2: uint16 = 0;
    let mut v3: uint16 = 0;
    let current_block: u64;
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let fresh53 = EprojStateMut(&mut game.g_ram).add_word(0x1b23, v1 as usize, u16::MAX);
    if (fresh53 != 0) {
        if CallSomeSamusCode(0xd as uint16, game) != 0 {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - 384 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v2 = 0;
                v3 = 0;
                v2 = abs16(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int
                        - ram_word!(game, grapple_beam_end_x_pos) as ::core::ffi::c_int)
                        as uint16,
                );
                if v2 as ::core::ffi::c_int - 16 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    v3 = abs16(
                        (*((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                            .offset(v1 as isize) as ::core::ffi::c_int
                            - ram_word!(game, grapple_beam_end_y_pos) as ::core::ffi::c_int)
                            as uint16,
                    );
                    if v3 as ::core::ffi::c_int - 16 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0
                    {
                        off_86F0AD[(*((&raw mut game.g_ram as *mut uint8).offset(0x1aff)
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
                let v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                let varE20: uint16 = EprojState(&game.g_ram).x_radius(v5 as usize) as uint16;
                let varE22: uint16 = EprojState(&game.g_ram).y_radius(v5 as usize) as uint16;
                let v6: uint16 = abs16(
                    game.samus_x_pos()
                        .wrapping_sub(game.enemy_projectile_x_pos(k)),
                );
                let mut v7: bool = v6 < game.samus_x_radius();
                let v8: uint16 = v6.wrapping_sub(game.samus_x_radius());
                if v7 || v8 < varE20 {
                    let v9: uint16 = abs16(
                        game.samus_y_pos()
                            .wrapping_sub(game.enemy_projectile_y_pos(k)),
                    );
                    v7 = v9 < game.samus_y_radius();
                    let v10: uint16 = v9.wrapping_sub(game.samus_y_radius());
                    if v7 || v10 < varE22 {
                        off_86F0AD[(*((&raw mut game.g_ram as *mut uint8).offset(0x1aff)
                            as *mut uint16)
                            .offset(v5 as isize)
                            as ::core::ffi::c_int
                            >> 1 as ::core::ffi::c_int)
                            as usize]
                            .expect("non-null function pointer")(game);
                        let v11: ::core::ffi::c_int =
                            k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                        EprojStateMut(&mut game.g_ram).finish_pickup(v11 as usize);
                    }
                }
                return;
            }
        }
    }
    let v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    EprojStateMut(&mut game.g_ram).finish_pickup(v4 as usize);
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
    1 as uint8, 2 as uint8, 4 as uint8, 6 as uint8, 5 as uint8, 3 as uint8,
];
pub unsafe fn RandomDropRoutine(k: uint16, game: &mut Game) -> uint16 {
    let mut v2: uint16 = 0;
    let mut Random: uint8 = 0;
    let mut v5: uint16 = 0;
    let mut v7: *const uint8 = ::core::ptr::null::<uint8>();
    let mut i: ::core::ffi::c_int = 0;
    let current_block: u64;
    let mut v9: int8 = 0;
    let mut r18: uint8 = 0;
    let mut r22: uint8 = 0;
    let mut r20: uint8 = 0;
    let mut r24: uint16 = 0;
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let varE28: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xf3c8) as *mut uint16).offset(v1 as isize);
    if (varE28 != 0) {
        v2 = 0;
        v2 = (*get_EnemyDef_A2(varE28, game)).item_drop_chances_ptr;
        if (v2 != 0) {
            Random = 0;
            loop {
                Random = NextRandom(game) as uint8;
                if (Random != 0) {
                    break;
                }
            }
            r20 = 255 as uint8;
            r24 = 0 as uint16;
            v5 = 0;
            v5 = 1 as uint16;
            if game
                .samus_reserve_health()
                .wrapping_add(game.samus_health())
                >= 0x1eu16
            {
                if game
                    .samus_reserve_health()
                    .wrapping_add(game.samus_health())
                    < 0x32u16
                {
                    current_block = 4166486009154926805;
                } else {
                    v5 = 0 as uint16;
                    current_block = 5399440093318478209;
                }
            } else {
                current_block = 5399440093318478209;
            }
            if current_block == 5399440093318478209 {
                set_ram_word!(game.health_drop_bias_flag = v5);
            }
            v7 = RomPtr_B4(v2, game);
            if ram_word!(game, health_drop_bias_flag) as uint8 != 0 {
                r18 = (*v7.offset(1)).wrapping_add(*v7);
                r22 = 3 as uint8;
            } else {
                r18 = *v7.offset(3);
                r22 = 8 as uint8;
                if game.samus_health() != game.samus_max_health()
                    || game.samus_reserve_health() != game.samus_max_reserve_health()
                {
                    r18 = ((*v7.offset(1) as ::core::ffi::c_int + *v7 as ::core::ffi::c_int) as u8)
                        .wrapping_add(r18);
                    r22 = (r22) | (3u8);
                }
                if game.samus_missiles() != game.samus_max_missiles() {
                    r18 = (*v7.offset(2)).wrapping_add(r18);
                    r22 = (r22) | (4u8);
                }
                if game.samus_super_missiles() != game.samus_max_super_missiles() {
                    r20 = (r20).wrapping_sub(*v7.offset(4));
                    r22 = (r22) | (0x10u8);
                }
                if game.samus_power_bombs() != game.samus_max_power_bombs() {
                    r20 =
                        (r20 as ::core::ffi::c_int - *v7.offset(5) as ::core::ffi::c_int) as uint8;
                    r22 = (r22 as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int) as uint8;
                }
            }
            i = 0;
            i = 0 as ::core::ffi::c_int;
            while i != 4i32 {
                if r18 == 0 {
                    r22 = (r22 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint8;
                    v2 = v2.wrapping_add(4);
                    i = 4 as ::core::ffi::c_int;
                    break;
                } else {
                    v9 = ((r22) as i8) & (1i8);
                    r22 = (r22 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint8;
                    if v9 != 0 {
                        let v10: *const uint8 = RomPtr_B4(v2, game);
                        let RegWord: uint16 = Mult8x8(r20, *v10);
                        let divved: ::core::ffi::c_int =
                            SnesDivide(RegWord, r18) as ::core::ffi::c_int;
                        let v12: uint16 = r24;
                        let v13: uint16 = (divved + v12 as ::core::ffi::c_int) as uint16;
                        if v13 >= ((Random) as u16) {
                            return byte_86F25E[i as usize] as uint16;
                        }
                        r24 = v13;
                    }
                    v2 = v2.wrapping_add(1);
                    i += 1;
                }
            }
            loop {
                v9 = ((r22) as i8) & (1i8);
                r22 = (r22 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint8;
                if v9 != 0 {
                    let v14: *const uint8 = RomPtr_B4(v2, game);
                    if (r24).wrapping_add((*v14) as u16) >= ((Random) as u16) {
                        return byte_86F25E[i as usize] as uint16;
                    }
                    r24 = (r24 as ::core::ffi::c_int + *v14 as ::core::ffi::c_int) as uint16;
                }
                v2 = v2.wrapping_add(1);
                i += 1;
                if (i == 6i32) {
                    break;
                }
            }
        }
    }
    i = 3 as ::core::ffi::c_int;
    byte_86F25E[i as usize] as uint16
}
pub unsafe fn RespawnEnemy(v0: uint16, game: &mut Game) {
    set_ram_word!(game.cur_enemy_index = v0);
    let v1: *const uint8 = RomPtr_A1(
        (ram_word!(game, room_enemy_population_ptr))
            .wrapping_add((v0 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as u16),
        game,
    );
    let E: *mut EnemyData = gEnemyData(v0 as ::core::ffi::c_int, game);
    (*E).enemy_ptr = *(v1 as *mut uint16);
    (*E).x_pos = *(v1.offset(2) as *mut uint16);
    (*E).y_pos = *(v1.offset(4) as *mut uint16);
    (*E).current_instruction = *(v1.offset(6) as *mut uint16);
    (*E).properties = *(v1.offset(8) as *mut uint16);
    (*E).extra_properties = *(v1.offset(10) as *mut uint16);
    (*E).parameter_1 = *(v1.offset(12) as *mut uint16);
    (*E).parameter_2 = *(v1.offset(14) as *mut uint16);
    let ESD: *mut EnemySpawnData = gEnemySpawnData(v0 as ::core::ffi::c_int, game);
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
    let Edef: *mut EnemyDef = get_EnemyDef_A2((*E).enemy_ptr, game);
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
unsafe fn EprojInit_Sparks(j: uint16, game: &mut Game) {
    static word_86F3D4: [uint16; 14] = [
        0xffff as uint16,
        0xb800 as uint16,
        0xffff as uint16,
        0xc000 as uint16,
        0xffff as uint16,
        0xe000 as uint16,
        0xffff as uint16,
        0xff00 as uint16,
        0 as uint16,
        0x100 as uint16,
        0 as uint16,
        0x2000 as uint16,
        0 as uint16,
        0x4000 as uint16,
    ];
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16).offset(v2 as isize) =
        addr_kEnemyDef_F353 as uint16;
    let v3: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v2 as isize) =
        (*v3).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v2 as isize) =
        (*v3).x_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v2 as isize) =
        ((*v3).y_pos as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v2 as isize) =
        (*v3).y_subpos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    let v4: uint16 = ((NextRandom(game) as ::core::ffi::c_int & 0x1c as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v2 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86f3d4)) as *const uint8 as *mut uint16)
            .offset((v4 as ::core::ffi::c_int + 0 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16).offset(v2 as isize) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86f3d4)) as *const uint8 as *mut uint16)
            .offset((v4 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize);
}
pub unsafe fn EprojPreInstr_Sparks(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        if EprojBlockCollisition_Vertical(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b47) as *mut uint16)
                .offset(v1 as isize) = 0xf363 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b8f) as *mut uint16)
                .offset(v1 as isize) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8).offset(0x1b23)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                * 2 as ::core::ffi::c_int
                + (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    >> 15 as ::core::ffi::c_int)) as uint16;
            let ref mut fresh50 = *((&raw mut game.g_ram as *mut uint8).offset(0x1aff)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh50 = (*fresh50 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16)
                .offset(v1 as isize) = (*((&raw mut game.g_ram as *mut uint8).offset(0x1b23)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                * 2 as ::core::ffi::c_int
                + (*((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    >> 15 as ::core::ffi::c_int)) as uint16;
            let ref mut fresh51 = *((&raw mut game.g_ram as *mut uint8).offset(0x1aff)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh51 = (*fresh51 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) = 0x8000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v1 as isize) = -(1 as ::core::ffi::c_int) as uint16;
            let ref mut fresh52 = *((&raw mut game.g_ram as *mut uint8).offset(0x1a93)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh52 = (*fresh52 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
            return;
        }
        let v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
            .offset(v1 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16).offset(v1 as isize) =
            (v2 as ::core::ffi::c_int + 0x4000 as ::core::ffi::c_int) as uint16;
        let v3: uint16 = (u16::overflowing_add(v2, 0x4000 as uint16).1 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
        if v3 < 4u16 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16)
                .offset(v1 as isize) = v3;
        }
    }
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a6f) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1adb) as *mut uint16).offset(v1 as isize)
            as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x1ab7) as *mut uint16)
                .offset(v1 as isize) as uint32,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x1a27) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1b23) as *mut uint16).offset(v1 as isize)
            as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x1aff) as *mut uint16)
                .offset(v1 as isize) as uint32,
    );
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x5b5) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int)
        == 0i32
    {
        CreateSpriteAtPos(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                .offset(v1 as isize),
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                .offset(v1 as isize),
            48 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16)
                .offset(v1 as isize),
            game,
        );
    }
}
unsafe fn CallEprojInit(ea: uint32, j: uint16, game: &mut Game) {
    match ea {
        8817209 => {
            Eproj_Init_0x8aaf(j, game);
        }
        8817357 => {
            init_skree_particle(j, 6, 320, -769, game);
        }
        8817393 => {
            init_skree_particle(j, 6, 96, -1025, game);
        }
        8817429 => {
            init_skree_particle(j, -6, -320, -769, game);
        }
        8817465 => {
            init_skree_particle(j, -6, -96, -1025, game);
        }
        8817924 => {
            EprojInit_DraygonsGunk(j, game);
        }
        8817984 => {
            EprojInit_DraygonsWallTurretProjs(j, game);
        }
        8818723 => {
            EprojInit_CrocomireProjectile(j, game);
        }
        8818895 => {
            EprojInit_CrocomireSpikeWallPieces(j, game);
        }
        8819334 => {
            EprojInit_CrocomireBridgeCrumbling(j, game);
        }
        8819533 => {
            EprojInit_9634(j, game);
        }
        8819658 => {
            EprojInit_9642_RidleysFireball(j, game);
        }
        8819839 => {
            EprojInit_9660_FireballExplosion(j, game);
        }
        8819872 => {
            EprojInit_9688(j, game);
        }
        8819892 => {
            EprojInit_9696(j, game);
        }
        8819912 => {
            EprojInit_966C(j, game);
        }
        8819932 => {
            EprojInit_967A(j, game);
        }
        8820444 => {
            EprojInit_9734_CeresFallingDebris(j, game);
        }
        8820772 => {
            EprojInit_PhantoonDestroyableFireballs(j, game);
        }
        8821050 => {
            EprojInit_PhantoonStartingFireballs(j, game);
        }
        8821923 => {
            EprojInit_RocksKraidSpits(j, game);
        }
        8821976 => {
            EprojInit_RocksFallingKraidCeiling(j, game);
        }
        8822028 => {
            EprojInit_RocksWhenKraidRises(j, game);
        }
        8822252 => {
            EprojInit_MiniKraidSpit(j, game);
        }
        8822342 => {
            EprojInit_MiniKraidSpikesLeft(j, game);
        }
        8822347 => {
            EprojInit_MiniKraidSpikesRight(j, game);
        }
        8822450 => {
            EprojInit_WalkingLavaSeahorseFireball(j, game);
        }
        8822793 => {
            EprojInit_PirateMotherBrainLaser(j, game);
        }
        8822936 => {
            EprojInit_PirateClaw(j, game);
        }
        8823457 => {
            EprojInit_A379(j, game);
        }
        8823534 => {
            EprojInit_CeresElevatorPad(j, game);
        }
        8823579 => {
            EprojInit_CeresElevatorPlatform(j, game);
        }
        8823715 => {
            EprojPreInstr_PrePhantomRoom(j, game);
        }
        8824275 => {
            EprojInit_BombTorizoLowHealthDrool(j, game);
        }
        8824413 => {
            EprojInit_BombTorizoLowHealthInitialDrool(j, game);
        }
        8824519 => {
            EprojInit_A977(j, game);
        }
        8824566 => {
            EprojInit_BombTorizoExplosiveSwipe(j, game);
        }
        8824676 => {
            EprojInit_BombTorizoStatueBreaking(j, game);
        }
        8824859 => {
            EprojInit_BombTorizoLowHealthExplode(j, game);
        }
        8824945 => {
            EprojInit_BombTorizoDeathExplosion(j, game);
        }
        8825405 => {
            EprojInit_AB07(j, game);
        }
        8825835 => {
            EprojInit_BombTorizosChozoOrbs(j, game);
        }
        8825980 => {
            EprojInit_GoldenTorizosChozoOrbs(j, game);
        }
        8826389 => {
            EprojInit_TorizoSonicBoom(j, game);
        }
        8826620 => {
            EprojInit_WreckedShipChozoSpikeFootsteps(j, game);
        }
        8826691 => {
            EprojInit_TourianStatueDustClouds(j, game);
        }
        8826704 => {
            EprojInit_TourianLandingDustCloudsRightFoot(j, game);
        }
        8826829 => {
            EprojInit_TorizoLandingDustCloudLeftFoot(j, game);
        }
        8826881 => {
            EprojInit_GoldenTorizoEgg(j, game);
        }
        8827342 => {
            EprojInit_GoldenTorizoSuperMissile(j, game);
        }
        8827688 => {
            EprojInit_GoldenTorizoEyeBeam(j, game);
        }
        8828061 => {
            EprojInit_TourianEscapeShaftFakeWallExplode(j, game);
        }
        8828143 => {
            EprojInit_LavaSeahorseFireball(j, game);
        }
        8828461 => {
            EprojInit_EyeDoorProjectile(j, game);
        }
        8828547 => {
            EprojInit_EyeDoorSweat(j, game);
        }
        8829050 => {
            EprojInit_TourianStatueUnlockingParticleWaterSplash(j, game);
        }
        8829070 => {
            EprojInit_TourianStatueEyeGlow(j, game);
        }
        8829109 => {
            EprojInit_TourianStatueUnlockingParticle(j, game);
        }
        8829160 => {
            EprojIni_TourianStatueUnlockingParticleTail(j, game);
        }
        8829176 => {
            EprojInit_TourianStatueSoul(j, game);
        }
        8829246 => {
            EprojInit_TourianStatueBaseDecoration(j, game);
        }
        8829265 => {
            EprojInit_TourianStatueRidley(j, game);
        }
        8829284 => {
            EprojInit_TourianStatuePhantoon(j, game);
        }
        8829744 => {
            sub_86BB30(j, game);
        }
        8829842 => {
            EprojInit_NuclearWaffleBody(j, game);
        }
        8829915 => {
            EprojInit_NorfairLavaquakeRocks(j, game);
        }
        8830364 => {
            EprojInit_ShaktoolAttackMiddleBackCircle(j, game);
        }
        8830370 => {
            EprojInit_BDA2(j, game);
        }
        8830543 => {
            EprojInit_MotherBrainRoomTurrets(j, game);
        }
        8830809 => {
            EprojInit_MotherBrainRoomTurretBullets(j, game);
        }
        8831731 => {
            Eproj_MotherBrainsBlueRingLasers(j, game);
        }
        8832130 => {
            EprojInit_MotherBrainBomb(j, game);
        }
        8832517 => {
            sub_86C605(j, game);
        }
        8832644 => {
            EprojInit_MotherBrainDeathBeemFired(j, game);
        }
        8833034 => {
            EprojInit_MotherBrainRainbowBeam(j, game);
        }
        8833091 => {
            EprojInit_MotherBrainsDrool(j, game);
        }
        8833269 => {
            EprojInit_MotherBrainsDeathExplosion(j, game);
        }
        8833327 => {
            EprojInit_MotherBrainsRainbowBeamExplosion(j, game);
        }
        8833377 => {
            EprojInit_MotherBrainEscapeDoorParticles(j, game);
        }
        8833642 => {
            EprojInit_MotherBrainPurpleBreathBig(j, game);
        }
        8833667 => {
            EprojInit_MotherBrainPurpleBreathSmall(j, game);
        }
        8833782 => {
            EprojInit_TimeBombSetJapaneseText(j, game);
        }
        8833993 => {
            EprojInit_MotherBrainTubeFalling(j, game);
        }
        8834501 => {
            EprojInit_MotherBrainGlassShatteringShard(j, game);
        }
        8834669 => {
            EprojInit_MotherBrainGlassShatteringSparkle(j, game);
        }
        8834960 => {
            EprojInit_KiHunterAcidSpitLeft(j, game);
        }
        8834982 => {
            EprojInit_KiHunterAcidSpitRight(j, game);
        }
        8835208 => {
            EprojInit_KagosBugs(j, game);
        }
        8835642 => {
            EprojInit_MaridiaFloatersSpikes(j, game);
        }
        8835852 => {
            EprojInit_WreckedShipRobotLaserDown(j, game);
        }
        8835886 => {
            EprojInit_WreckedShipRobotLaserHorizontal(j, game);
        }
        8835905 => {
            EprojInit_WreckedShipRobotLaserUp(j, game);
        }
        8836773 => {
            EprojInit_N00bTubeCrack(j, game);
        }
        8836809 => {
            EprojInit_N00bTubeShards(j, game);
        }
        8836980 => {
            EprojInit_N00bTubeReleasedAirBubbles(j, game);
        }
        8837522 => {
            sub_86D992(j, game);
        }
        8837912 => {
            EprojInit_DBF2(j, game);
        }
        8838285 => {
            EprojInit_Spores(j, game);
        }
        8838307 => {
            EprojInit_SporeSpawnStalk(j, game);
        }
        8838356 => {
            EprojInit_SporeSpawners(j, game);
        }
        8838870 => {
            EprojInit_NamiFuneFireball(j, game);
        }
        8839168 => {
            EprojInit_LavaThrownByLavaman(j, game);
        }
        8840296 => {
            EprojInit_DustCloudOrExplosion(j, game);
        }
        8840358 => {
            EprojInit_EyeDoorSmoke(j, game);
        }
        8840656 => {
            EprojInit_SpawnedShotGate(j, game);
        }
        8840661 => {
            EprojInit_ClosedDownwardsShotGate(j, game);
        }
        8840666 => {
            EprojInit_ClosedUpwardsShotGate(j, game);
        }
        8840877 => {
            EprojInit_SaveStationElectricity(j, game);
        }
        8841777 => {
            EprojInit_BotwoonsBody(j, game);
        }
        8842182 => {
            EprojInit_BotwoonsSpit(j, game);
        }
        8842338 => {
            EprojInit_YappingMawsBody(j, game);
        }
        8843049 => {
            EprojInit_F337(j, game);
        }
        8843145 => {
            EprojInit_EnemyDeathExplosion(j, game);
        }
        8844177 => {
            EprojInit_Sparks(j, game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
unsafe fn CallEprojPreInstr(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        8825112 => (),
        8814960 => (),
        8815867 => (),
        8817277 => {
            Eproj_PreInit_0x8aaf(k, game);
        }
        8817501 => {
            Eproj_PreInstr_SkreeParticle(k, game);
        }
        8818004 => (),
        8818122 => {
            EprojPreInstr_8DCA(k, game);
        }
        8818175 => {
            EprojPreInstr_DraygonsTurret_8DFF(k, game);
        }
        8818191 => {
            EprojPreInstr_DraygonsGunk_8E0F(k, game);
        }
        8818795 => {
            EprojPreInstr_CrocomireProjectile(k, game);
        }
        8818867 => {
            sub_8690B3(k, game);
        }
        8818965 => {
            EprojPreInstr_CrocomireSpikeWallPieces(k, game);
        }
        8819386 => {
            EprojPreInstr_CrocomireBridgeCrumbling(k, game);
        }
        8819602 => {
            EprojPreInstr_9634(k, game);
        }
        8819726 => {
            EprojPreInstr_9642_RidleysFireball(k, game);
        }
        8819980 => (),
        8819981 => {
            EprojPreInstr_966C(k, game);
        }
        8820002 => {
            EprojPreInstr_9688(k, game);
        }
        8820023 => {
            EprojPreInstr_96A4(k, game);
        }
        8820032 => {
            EprojPreInstr_96C0(k, game);
        }
        8820041 => {
            EprojPreInstr_96CE(k, game);
        }
        8820481 => {
            EprojPreInstr_9734_CeresFallingDebris(k, game);
        }
        8821121 => {
            EprojPreInstr_PhantoonStartingFireballs(k, game);
        }
        8821183 => {
            EprojPreInstr_PhantoonStartingFireballs2(k, game);
        }
        8821249 => {
            EprojPreInstr_PhantoonStartingFireballs3(k, game);
        }
        8821316 => (),
        8821317 => {
            EprojPreInstr_PhantoonDestroyableFireballs(k, game);
        }
        8821396 => {
            EprojPreInstr_PhantoonDestroyableFireballs_2(k, game);
        }
        8821466 => {
            EprojPreInstr_PhantoonDestroyableFireballs_3(k, game);
        }
        8821545 => {
            EprojPreInstr_PhantoonStartingFireballsB(k, game);
        }
        8821569 => {
            EprojPreInstr_PhantoonStartingFireballsB_2(k, game);
        }
        8822102 => {
            EprojPreInstr_KraidRocks(k, game);
        }
        8822153 => {
            EprojPreInstr_RocksFallingKraidCeiling(k, game);
        }
        8822181 => {
            sub_869DA5(k, game);
        }
        8822302 => {
            EprojPreInit_MiniKraidSpit(k, game);
        }
        8822403 => {
            EprojPreInstr_MiniKraidSpikes(k, game);
        }
        8822527 => {
            EprojPreInstr_WalkingLavaSeahorseFireball(k, game);
        }
        8822875 => (),
        8822876 => {
            EprojPreInstr_PirateMotherBrainLaser_MoveLeft(k, game);
        }
        8822906 => {
            EprojPreInstr_PirateMotherBrainLaser_MoveRight(k, game);
        }
        8822993 => {
            EprojPreInstr_PirateClawThrownLeft(k, game);
        }
        8823076 => {
            EprojPreInstr_PirateClawThrownRight(k, game);
        }
        8823591 => (),
        8823592 => {
            EprojPreInstr_CeresElevatorPad(k, game);
        }
        8823652 => {
            EprojPreInstr_CeresElevatorPlatform(k, game);
        }
        8823715 => {
            EprojPreInstr_PrePhantomRoom(k, game);
        }
        8824967 => {
            sub_86A887(k, game);
        }
        8825071 => {
            EprojPreInstr_A977(k, game);
        }
        8825113 => (),
        8825484 => {
            EprojPreInstr_AB07(k, game);
        }
        8826029 => {
            EprojPreInstr_BombTorizosChozoOrbs(k, game);
        }
        8826106 => {
            EprojPreInstr_GoldenTorizosChozoOrbs(k, game);
        }
        8826476 => {
            EprojPreInstr_TorizoSonicBoom(k, game);
        }
        8826947 => {
            EprojPreInstr_GoldenTorizoEgg(k, game);
        }
        8827065 => {
            sub_86B0B9(k, game);
        }
        8827101 => {
            sub_86B0DD(k, game);
        }
        8827405 => {
            EprojPreInstr_GoldenTorizoSuperMissile(k, game);
        }
        8827447 => {
            EprojPreInstr_B237(k, game);
        }
        8827786 => {
            EprojPreInstr_GoldenTorizoEyeBeam(k, game);
        }
        8828080 => (),
        8828213 => {
            sub_86B535(k, game);
        }
        8828601 => {
            EprojPreInstr_EyeDoorProjectile(k, game);
        }
        8828692 => {
            EprojPreInstr_EyeDoorSweat(k, game);
        }
        8829303 => {
            EprojPreInstr_TourianStatueUnlockingParticleWaterSplash(k, game);
        }
        8829314 => {
            EprojPreInstr_TourianStatueUnlockingParticle(k, game);
        }
        8829437 => {
            EprojPreInstr_TourianStatueSoul(k, game);
        }
        8829495 => {
            EprojPreInstr_TourianStatueStuff(k, game);
        }
        8829506 => {
            EprojPreInstr_BA42(k, game);
        }
        8829775 => (),
        8829894 => (),
        8829967 => {
            EprojPreInstr_NorfairLavaquakeRocks(k, game);
        }
        8830238 => {
            EprojPreInstr_NorfairLavaquakeRocks_Inner(k, game);
        }
        8830250 => {
            EprojPreInstr_NorfairLavaquakeRocks_Inner2(k, game);
        }
        8830467 => {
            EprojInit_ShaktoolAttackFrontCircle(k, game);
        }
        8830482 => {
            EprojPreInstr_BE12(k, game);
        }
        8830943 => {
            EprojPreInstr_MotherBrainRoomTurrets(k, game);
        }
        8831200 => {
            EprojPreInstr_MotherBrainRoomTurretBullets(k, game);
        }
        8831797 => {
            Eproj_MoveToBlueRingSpawnPosition(k, game);
        }
        8832200 => {
            EprojPreInstr_MotherBrainBomb(k, game);
        }
        8833044 => {
            EprojPreInstr_MotherBrainRainbowBeam(k, game);
        }
        8833101 => {
            EprojPreInstr_C84D(k, game);
        }
        8833158 => {
            EprojInit_MotherBrainsDrool_Falling(k, game);
        }
        8833300 => {
            EprojPreInit_MotherBrainsDeathExplosion_0(k, game);
        }
        8833356 => {
            EprojPreInstr_MotherBrainsRainbowBeamExplosion(k, game);
        }
        8833490 => {
            EprojPreInstr_MotherBrainsExplodedDoorParticles(k, game);
        }
        8833699 => (),
        8833786 => {
            EprojPreInstr_TimeBombSetJapaneseText(k, game);
        }
        8834023 => {
            EprojPreInstr_MotherBrainTubeFalling(k, game);
        }
        8834715 => {
            EprojPreInstr_MotherBrainGlassShatteringShard(k, game);
        }
        8835029 => {
            sub_86CFD5(k, game);
        }
        8835046 => {
            sub_86CFE6(k, game);
        }
        8835063 => (),
        8835064 => {
            sub_86CFF8(k, game);
        }
        8835251 => {
            EprojPreInstr_KagosBugs_Func1(k, game);
        }
        8835274 => {
            EprojPreInstr_KagosBugs(k, game);
        }
        8835307 => (),
        8835308 => {
            EprojPreInstr_D0EC(k, game);
        }
        8835368 => {
            EprojPreInstr_D128(k, game);
        }
        8835556 => {
            EprojPreInstr_KagosBugs_Func2(k, game);
        }
        8835683 => {
            EprojPreInstr_MaridiaFloatersSpikes(k, game);
        }
        8836031 => {
            EprojPreInstr_WreckedShipRobotLaser(k, game);
        }
        8837055 => {
            sub_86D7BF(k, game);
        }
        8837086 => {
            sub_86D7DE(k, game);
        }
        8837117 => {
            EprojPreInstr_N00bTubeShards(k, game);
        }
        8837181 => {
            sub_86D83D(k, game);
        }
        8837279 => {
            sub_86D89F(k, game);
        }
        8837343 => {
            sub_86D8DF(k, game);
        }
        8837595 => {
            EprojPreInstr_SpikeShootingPlantSpikes(k, game);
        }
        8837606 => {
            EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, game);
        }
        8837648 => {
            EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, game);
        }
        8837690 => {
            EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, game);
        }
        8837732 => {
            EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, game);
        }
        8837774 => {
            EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1(k, game);
        }
        8837779 => {
            EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2(k, game);
        }
        8837784 => {
            EprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1(k, game);
        }
        8837789 => {
            EprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2(k, game);
        }
        8837794 => {
            EprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1(k, game);
        }
        8837802 => {
            EprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2(k, game);
        }
        8837810 => {
            EprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1(k, game);
        }
        8837818 => {
            EprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2(k, game);
        }
        8837979 => {
            EprojPreInstr_DBF2(k, game);
        }
        8837986 => {
            EprojPreInstr_DBF2_MoveX1(k, game);
        }
        8838028 => {
            EprojPreInstr_DBF2_MoveX2(k, game);
        }
        8838070 => {
            EprojPreInstr_DBF2_Func1(k, game);
        }
        8838382 => {
            EprojPreInstr_Spores(k, game);
        }
        8838468 => (),
        8838470 => {
            EprojPreInstr_SporeSpawners(k, game);
        }
        8838969 => {
            EprojPreInstr_NamiFuneFireball(k, game);
        }
        8839241 => {
            sub_86E049(k, game);
        }
        8840446 => {
            EprojPreInstr_DustCloudOrExplosion(k, game);
        }
        8840456 => (),
        8840708 => (),
        8840709 => {
            EprojPreInstr_E605(k, game);
        }
        8840913 => (),
        8841856 => {
            EprojPreInstr_BotwoonsBody(k, game);
        }
        8842245 => {
            EprojPreInstr_BotwoonsSpit(k, game);
        }
        8842388 => (),
        8843231 => (),
        8843232 => {
            EprojPreInstr_Pickup(k, game);
        }
        8844272 => {
            EprojPreInstr_Sparks(k, game);
        }
        8832877 => (),
        _ => {
            Unreachable(game);
        }
    }
}
unsafe fn CallEprojInstr(ea: uint32, k: uint16, j: *const uint8, game: &mut Game) -> *const uint8 {
    match ea {
        8814932 => EprojInstr_Delete(k, j, game),
        8814937 => EprojInstr_Sleep(k, j, game),
        8814945 => EprojInstr_SetPreInstr_(k, j, game),
        8814954 => EprojInstr_ClearPreInstr(k, j, game),
        8814961 => EprojInstr_CallFunc(k, j, game),
        8815019 => EprojInstr_Goto(k, j),
        8815024 => EprojInstr_GotoRel(k, j),
        8815046 => EprojInstr_DecTimerAndGotoIfNonZero(k, j, game),
        8815054 => EprojInstr_DecTimerAndGotoRelIfNonZero(k, j, game),
        8815061 => EprojInstr_SetTimer(k, j, game),
        8815071 => EprojInstr_MoveRandomlyWithinRadius(k, j, game),
        8815152 => EprojInstr_SetProjectileProperties(k, j, game),
        8815164 => EprojInstr_ClearProjectileProperties(k, j, game),
        8815176 => EprojInstr_EnableCollisionWithSamusProj(k, j, game),
        8815186 => EprojInstr_DisableCollisionWithSamusProj(k, j, game),
        8815196 => EprojInstr_DisableCollisionWithSamus(k, j, game),
        8815206 => EprojInstr_EnableCollisionWithSamus(k, j, game),
        8815216 => EprojInstr_SetToNotDieOnContact(k, j, game),
        8815226 => EprojInstr_SetToDieOnContact(k, j, game),
        8815236 => EprojInstr_SetLowPriority(k, j, game),
        8815246 => EprojInstr_SetHighPriority(k, j, game),
        8815256 => EprojInstr_SetXyRadius(k, j, game),
        8815265 => EprojInstr_SetXyRadiusZero(k, j, game),
        8815269 => EprojInstr_CalculateDirectionTowardsSamus(k, j, game),
        8815317 => EprojInstr_WriteColorsToPalette(k, j, game),
        8815357 => EprojInstr_QueueMusic(k, j, game),
        8815369 => EprojInstr_QueueSfx1_Max6(k, j, game),
        8815378 => EprojInstr_QueueSfx2_Max6(k, j, game),
        8815387 => EprojInstr_QueueSfx3_Max6(k, j, game),
        8815396 => EprojInstr_QueueSfx1_Max15(k, j, game),
        8815405 => EprojInstr_QueueSfx2_Max15(k, j, game),
        8815414 => EprojInstr_QueueSfx3_Max15(k, j, game),
        8815423 => EprojInstr_QueueSfx1_Max3(k, j, game),
        8815432 => EprojInstr_QueueSfx2_Max3(k, j, game),
        8815441 => EprojInstr_QueueSfx3_Max3(k, j, game),
        8815450 => EprojInstr_QueueSfx1_Max9(k, j, game),
        8815459 => EprojInstr_QueueSfx2_Max9(k, j, game),
        8815468 => EprojInstr_QueueSfx3_Max9(k, j, game),
        8815477 => EprojInstr_QueueSfx1_Max1(k, j, game),
        8815486 => EprojInstr_QueueSfx2_Max1(k, j, game),
        8815495 => EprojInstr_QueueSfx3_Max1(k, j, game),
        8817768 => EprojInstr_SpawnEnemyDropsWithDraygonsEyeDrops(k, j, game),
        8818073 => EprojInstr_868D99(k, j, game),
        8819829 => EprojInstr_DisableCollisionsWithSamus(k, j, game),
        8820154 => EprojInstr_95BA(k, j, game),
        8820205 => EprojInstr_95ED(k, j, game),
        8820256 => EprojInstr_9620(k, j, game),
        8820750 => EprojInstr_980E(k, j, game),
        8822864 => EprojInstr_SetPreInstrAndRun(k, j, game),
        8823894 => EprojInstr_GotoWithProbability25(k, j, game),
        8825738 => EprojInstr_SpawnEnemyDrops(k, j, game),
        8826258 => EprojInstr_GotoDependingOnXDirection(k, j, game),
        8826678 => EprojInstr_ResetXYpos1(k, j, game),
        8826770 => EprojInstr_MoveY_Minus4(k, j, game),
        8827497 => EprojInstr_SetVelTowardsSamus1(k, j, game),
        8827506 => EprojInstr_SetVelTowardsSamus2(k, j, game),
        8827832 => EprojInstr_GotoIfFunc1(k, j, game),
        8827958 => EprojInstr_ResetXYpos2(k, j, game),
        8828906 => EprojInstr_SpawnTourianStatueUnlockingParticle(k, j, game),
        8828917 => EprojInstr_Earthquake(k, j, game),
        8828952 => EprojInstr_SpawnTourianStatueUnlockingParticleTail(k, j, game),
        8828993 => EprojInstr_AddToYpos(k, j, game),
        8831347 => EprojInstr_SwitchJump(k, j, game),
        8831412 => EprojInstr_UserPalette0(k, j, game),
        8833232 => EprojInstr_Add12ToY(k, j, game),
        8833774 => EprojInstr_MotherBrainPurpleBreathIsActive(k, j, game),
        8835420 => EprojInstr_D15C(k, j, game),
        8835510 => EprojInstr_D1B6(k, j, game),
        8835527 => EprojInstr_D1C7(k, j, game),
        8835534 => EprojInstr_D1CE(k, j, game),
        8836577 => EprojInstr_AssignNewN00bTubeShardVelocity(k, j, game),
        8836594 => EprojInstr_SetN00bTubeShardX(k, j, game),
        8836650 => EprojInstr_D62A(k, j, game),
        8836762 => EprojInstr_SetXvelRandom(k, j, game),
        8838234 => EprojInstr_DC5A(k, j, game),
        8838241 => EprojInstr_SpawnEnemyDrops_0(k, j, game),
        8838263 => EprojInstr_SpawnSporesEproj(k, j, game),
        8839146 => EprojInstr_DFEA(k, j, game),
        8840499 => EprojInstr_SetYVel(k, j, game),
        8842467 => EprojInstr_ECE3(k, j, game),
        8842519 => EprojInstr_ED17(k, j, game),
        8842891 => EprojInstr_QueueSfx2_9(k, j, game),
        8842903 => EprojInstr_QueueSfx2_24(k, j, game),
        8842915 => EprojInstr_QueueSfx2_B(k, j, game),
        8842927 => EprojInstr_EEAF(k, j, game),
        8843024 => EprojInstr_HandleRespawningEnemy(k, j, game),
        8817910 => EprojInstr_SetPreInstrA(k, j, game),
        8817917 => EprojInstr_SetPreInstrB(k, j, game),
        8822906 => {
            EprojPreInstr_PirateMotherBrainLaser_MoveRight(k, game);
            j
        }
        8822876 => {
            EprojPreInstr_PirateMotherBrainLaser_MoveLeft(k, game);
            j
        }
        8823742 => EprojInstr_A3BE(k, j, game),
        8819312 => EprojInstr_9270(k, j, game),
        8815070 => j,
        8827198 => sub_86B13E(k, j, game),
        _ => {
            Unreachable(game);
            ::core::ptr::null::<uint8>()
        }
    }
}
