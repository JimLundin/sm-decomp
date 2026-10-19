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
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::abs16;
use crate::rtl::AddToHiLo;
use crate::rtl::{Load24, Mult8x8, RomPtr, SnesDivide, SnesModulus, Unreachable};
use ::core::ffi::{c_long, c_void};
use ::core::ptr::{null, null_mut};

pub use crate::types::{
    EnemyBase, EnemyData, EnemyDef, EnemySpawnData, EprojDef, ExpandingSquareTransitionHdma,
    ExtraEnemyRam7800, ExtraEnemyRam8000, Func_V, Func_Y_V, LongPtr, Point16U, Ram7800_Kraid,
    Rect16U, VoidP, VramWriteEntry,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ExtraEnemyRam8800 {
    pub pad: [u8; 64],
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Ram7800_Default {
    pub var_00: u16,
    pub var_01: u16,
    pub var_02: u16,
    pub var_03: u16,
    pub var_04: u16,
    pub var_05: u16,
    pub var_06: u16,
    pub var_07: u16,
    pub var_08: u16,
    pub var_09: u16,
    pub var_0A: u16,
    pub var_0B: u16,
    pub var_0C: u16,
    pub var_0D: u16,
    pub var_0E: u16,
    pub var_0F: u16,
    pub var_10: u16,
    pub var_11: u16,
    pub var_12: u16,
    pub var_13: u16,
    pub var_14: u16,
    pub var_15: u16,
    pub var_16: u16,
    pub var_17: u16,
    pub var_18: u16,
    pub var_19: u16,
    pub var_1A: u16,
    pub var_1B: u16,
    pub var_1C: u16,
    pub var_1D: u16,
    pub var_1E: u16,
    pub var_1F: u16,
}
pub use crate::types::Consts_86;
pub use crate::types::Consts_A0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct EprojCollInfo {
    pub eci_r20: u16,
    pub eci_r24: u16,
    pub eci_r26: u16,
    pub eci_r28: u16,
    pub eci_r30: u16,
    pub eci_r32: u16,
    pub eci_r34: u16,
}
pub type Func_EprojCollInfo_U8 = unsafe fn(*mut EprojCollInfo, &mut Game) -> u8;
pub use crate::types::{Enemy_MotherBrain, Point32};
pub const NULL: *mut c_void = null_mut::<c_void>();
#[inline]
unsafe fn RomPtr_86(addr: u16, game: &mut Game) -> *const u8 {
    RomPtr((0x860000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtr_A0(addr: u16, game: &mut Game) -> *const u8 {
    RomPtr((0xa00000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtr_A1(addr: u16, game: &mut Game) -> *const u8 {
    RomPtr((0xa10000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtr_B4(addr: u16, game: &mut Game) -> *const u8 {
    RomPtr((0xb40000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomBankBase(bank: u8, game: &mut Game) -> *const u8 {
    RomPtr(
        (((bank as i32) << 16i32) as u32).wrapping_add(0x8000u32),
        game,
    )
    .offset(-(0x8000i32 as isize))
}
#[inline]
unsafe fn gVramWriteEntry(a: i32, game: &mut Game) -> *mut VramWriteEntry {
    (&raw mut game.g_ram as *mut u8).offset((0xd0i32 + a) as isize) as *mut u8
        as *mut VramWriteEntry
}
#[inline]
unsafe fn get_EnemyDef_A2(a: u16, game: &mut Game) -> *mut EnemyDef {
    RomPtr((0xa00000u32) | ((a) as u32), game) as *mut EnemyDef
}
#[inline]
unsafe fn get_EprojDef(a: u16, game: &mut Game) -> *mut EprojDef {
    RomPtr((0x860000u32) | ((a) as u32), game) as *mut EprojDef
}
#[inline]
unsafe fn gRam7800_Default(a: u16, game: &mut Game) -> *mut Ram7800_Default {
    (&raw mut game.g_ram as *mut u8).offset((0x7800i32 + a as i32) as isize) as *mut u8
        as *mut Ram7800_Default
}
pub const fnEprojInstr_Delete: u32 = 8814932;
pub const fnEprojInstr_Sleep: u32 = 8814937;
pub const fnEprojInstr_SetPreInstr_: u32 = 8814945;
pub const fnEprojInstr_ClearPreInstr: u32 = 8814954;
pub const fnEprojPreInstr_nullsub_297: u32 = 8814960;
pub const fnEprojInstr_CallFunc: u32 = 8814961;
pub const fnEprojInstr_Goto: u32 = 8815019;
pub const fnEprojInstr_GotoRel: u32 = 8815024;
pub const fnEprojInstr_DecTimerAndGotoIfNonZero: u32 = 8815046;
pub const fnEprojInstr_DecTimerAndGotoRelIfNonZero: u32 = 8815054;
pub const fnEprojInstr_SetTimer: u32 = 8815061;
pub const fnnullsub_82: u32 = 8815070;
pub const fnEprojInstr_MoveRandomlyWithinRadius: u32 = 8815071;
pub const fnEprojInstr_SetProjectileProperties: u32 = 8815152;
pub const fnEprojInstr_ClearProjectileProperties: u32 = 8815164;
pub const fnEprojInstr_EnableCollisionWithSamusProj: u32 = 8815176;
pub const fnEprojInstr_DisableCollisionWithSamusProj: u32 = 8815186;
pub const fnEprojInstr_DisableCollisionWithSamus: u32 = 8815196;
pub const fnEprojInstr_EnableCollisionWithSamus: u32 = 8815206;
pub const fnEprojInstr_SetToNotDieOnContact: u32 = 8815216;
pub const fnEprojInstr_SetToDieOnContact: u32 = 8815226;
pub const fnEprojInstr_SetLowPriority: u32 = 8815236;
pub const fnEprojInstr_SetHighPriority: u32 = 8815246;
pub const fnEprojInstr_SetXyRadius: u32 = 8815256;
pub const fnEprojInstr_SetXyRadiusZero: u32 = 8815265;
pub const fnEprojInstr_CalculateDirectionTowardsSamus: u32 = 8815269;
pub const fnEprojInstr_WriteColorsToPalette: u32 = 8815317;
pub const fnEprojInstr_QueueMusic: u32 = 8815357;
pub const fnEprojInstr_QueueSfx1_Max6: u32 = 8815369;
pub const fnEprojInstr_QueueSfx2_Max6: u32 = 8815378;
pub const fnEprojInstr_QueueSfx3_Max6: u32 = 8815387;
pub const fnEprojInstr_QueueSfx1_Max15: u32 = 8815396;
pub const fnEprojInstr_QueueSfx2_Max15: u32 = 8815405;
pub const fnEprojInstr_QueueSfx3_Max15: u32 = 8815414;
pub const fnEprojInstr_QueueSfx1_Max3: u32 = 8815423;
pub const fnEprojInstr_QueueSfx2_Max3: u32 = 8815432;
pub const fnEprojInstr_QueueSfx3_Max3: u32 = 8815441;
pub const fnEprojInstr_QueueSfx1_Max9: u32 = 8815450;
pub const fnEprojInstr_QueueSfx2_Max9: u32 = 8815459;
pub const fnEprojInstr_QueueSfx3_Max9: u32 = 8815468;
pub const fnEprojInstr_QueueSfx1_Max1: u32 = 8815477;
pub const fnEprojInstr_QueueSfx2_Max1: u32 = 8815486;
pub const fnEprojInstr_QueueSfx3_Max1: u32 = 8815495;
pub const fnEprojPreInstr_nullsub_83: u32 = 8815867;
pub const fnEproj_Init_0x8aaf: u32 = 8817209;
pub const fnEproj_PreInit_0x8aaf: u32 = 8817277;
pub const fnEproj_Init_0x8bc2_SkreeDownRight: u32 = 8817357;
pub const fnEproj_Init_0x8bd0_SkreeUpRight: u32 = 8817393;
pub const fnEproj_Init_0x8bde_SkreeDownLeft: u32 = 8817429;
pub const fnEproj_Init_0x8bec_SkreeUpLeft: u32 = 8817465;
pub const fnEproj_PreInstr_SkreeParticle: u32 = 8817501;
pub const fnEprojInstr_SpawnEnemyDropsWithDraygonsEyeDrops: u32 = 8817768;
pub const fnEprojInstr_SetPreInstrA: u32 = 8817910;
pub const fnEprojInstr_SetPreInstrB: u32 = 8817917;
pub const fnEprojInit_DraygonsGunk: u32 = 8817924;
pub const fnEprojInit_DraygonsWallTurretProjs: u32 = 8817984;
pub const fnnullsub_84: i32 = 0x868d54;
pub const fnEprojInstr_868D99: u32 = 8818073;
pub const fnEprojPreInstr_8DCA: i32 = 0x868dca;
pub const fnEprojPreInstr_DraygonsTurret_8DFF: i32 = 0x868dff;
pub const fnEprojPreInstr_DraygonsGunk_8E0F: u32 = 8818191;
pub const fnEprojInit_CrocomireProjectile: u32 = 8818723;
pub const fnEprojPreInstr_CrocomireProjectile: u32 = 8818795;
pub const fnsub_8690B3: i32 = 0x8690b3;
pub const fnEprojInit_CrocomireSpikeWallPieces: u32 = 8818895;
pub const fnEprojPreInstr_CrocomireSpikeWallPieces: u32 = 8818965;
pub const fnEprojInstr_9270: u32 = 8819312;
pub const fnEprojInit_CrocomireBridgeCrumbling: u32 = 8819334;
pub const fnEprojPreInstr_CrocomireBridgeCrumbling: u32 = 8819386;
pub const fnEprojInit_9634: u32 = 8819533;
pub const fnEprojPreInstr_9634: u32 = 8819602;
pub const fnEprojInit_9642_RidleysFireball: u32 = 8819658;
pub const fnEprojPreInstr_9642_RidleysFireball: u32 = 8819726;
pub const fnEprojInstr_DisableCollisionsWithSamus: u32 = 8819829;
pub const fnEprojInit_9660_FireballExplosion: u32 = 8819839;
pub const fnEprojInit_9688: u32 = 8819872;
pub const fnEprojInit_9696: u32 = 8819892;
pub const fnEprojInit_966C: u32 = 8819912;
pub const fnEprojInit_967A: u32 = 8819932;
pub const fnnullsub_85: u32 = 8819980;
pub const fnEprojPreInstr_966C: u32 = 8819981;
pub const fnEprojPreInstr_9688: u32 = 8820002;
pub const fnEprojPreInstr_96A4: u32 = 8820023;
pub const fnEprojPreInstr_96C0: u32 = 8820032;
pub const fnEprojPreInstr_96CE: u32 = 8820041;
pub const fnEprojInstr_95BA: u32 = 8820154;
pub const fnEprojInstr_95ED: u32 = 8820205;
pub const fnEprojInstr_9620: u32 = 8820256;
pub const fnEprojInit_9734_CeresFallingDebris: u32 = 8820444;
pub const fnEprojPreInstr_9734_CeresFallingDebris: u32 = 8820481;
pub const fnEprojInstr_980E: u32 = 8820750;
pub const fnEprojInit_PhantoonDestroyableFireballs: u32 = 8820772;
pub const fnEprojInit_PhantoonStartingFireballs: u32 = 8821050;
pub const fnEprojPreInstr_PhantoonStartingFireballs: u32 = 8821121;
pub const fnEprojPreInstr_PhantoonStartingFireballs2: i32 = 0x8699bf;
pub const fnEprojPreInstr_PhantoonStartingFireballs3: i32 = 0x869a01;
pub const fnnullsub_86: i32 = 0x869a44;
pub const fnEprojPreInstr_PhantoonDestroyableFireballs: i32 = 0x869a45;
pub const fnEprojPreInstr_PhantoonDestroyableFireballs_2: i32 = 0x869a94;
pub const fnEprojPreInstr_PhantoonDestroyableFireballs_3: i32 = 0x869ada;
pub const fnEprojPreInstr_PhantoonStartingFireballsB: u32 = 8821545;
pub const fnEprojPreInstr_PhantoonStartingFireballsB_2: i32 = 0x869b41;
pub const fnEprojInit_RocksKraidSpits: u32 = 8821923;
pub const fnEprojInit_RocksFallingKraidCeiling: u32 = 8821976;
pub const fnEprojInit_RocksWhenKraidRises: u32 = 8822028;
pub const fnEprojPreInstr_KraidRocks: u32 = 8822102;
pub const fnEprojPreInstr_RocksFallingKraidCeiling: u32 = 8822153;
pub const fnsub_869DA5: u32 = 8822181;
pub const fnEprojInit_MiniKraidSpit: u32 = 8822252;
pub const fnEprojPreInit_MiniKraidSpit: u32 = 8822302;
pub const fnEprojInit_MiniKraidSpikesLeft: u32 = 8822342;
pub const fnEprojInit_MiniKraidSpikesRight: u32 = 8822347;
pub const fnEprojPreInstr_MiniKraidSpikes: u32 = 8822403;
pub const fnEprojInit_WalkingLavaSeahorseFireball: u32 = 8822450;
pub const fnEprojPreInstr_WalkingLavaSeahorseFireball: u32 = 8822527;
pub const fnEprojInit_PirateMotherBrainLaser: u32 = 8822793;
pub const fnEprojInstr_SetPreInstrAndRun: u32 = 8822864;
pub const fnnullsub_87: i32 = 0x86a05b;
pub const fnEprojPreInstr_PirateMotherBrainLaser_MoveLeft: u32 = 8822876;
pub const fnEprojPreInstr_PirateMotherBrainLaser_MoveRight: u32 = 8822906;
pub const fnEprojInit_PirateClaw: u32 = 8822936;
pub const fnEprojPreInstr_PirateClawThrownLeft: u32 = 8822993;
pub const fnEprojPreInstr_PirateClawThrownRight: u32 = 8823076;
pub const fnEprojInit_A379: u32 = 8823457;
pub const fnEprojInit_CeresElevatorPad: u32 = 8823534;
pub const fnEprojInit_CeresElevatorPlatform: u32 = 8823579;
pub const fnnullsub_88: u32 = 8823591;
pub const fnEprojPreInstr_CeresElevatorPad: u32 = 8823592;
pub const fnEprojPreInstr_CeresElevatorPlatform: u32 = 8823652;
pub const fnEprojPreInstr_PrePhantomRoom: u32 = 8823715;
pub const fnEprojInstr_A3BE: u32 = 8823742;
pub const fnEprojInstr_GotoWithProbability25: u32 = 8823894;
pub const fnEprojInit_BombTorizoLowHealthDrool: u32 = 8824275;
pub const fnEprojInit_BombTorizoLowHealthInitialDrool: u32 = 8824413;
pub const fnEprojInit_A977: u32 = 8824519;
pub const fnEprojInit_BombTorizoExplosiveSwipe: u32 = 8824566;
pub const fnEprojInit_BombTorizoStatueBreaking: u32 = 8824676;
pub const fnEprojInit_BombTorizoLowHealthExplode: u32 = 8824859;
pub const fnEprojInit_BombTorizoDeathExplosion: u32 = 8824945;
pub const fnsub_86A887: u32 = 8824967;
pub const fnEprojPreInstr_A977: u32 = 8825071;
pub const fnnullsub_352: u32 = 8825112;
pub const fnnullsub_89: u32 = 8825113;
pub const fnEprojInit_AB07: u32 = 8825405;
pub const fnEprojPreInstr_AB07: u32 = 8825484;
pub const fnEprojInstr_SpawnEnemyDrops: u32 = 8825738;
pub const fnEprojInit_BombTorizosChozoOrbs: u32 = 8825835;
pub const fnEprojInit_GoldenTorizosChozoOrbs: u32 = 8825980;
pub const fnEprojPreInstr_BombTorizosChozoOrbs: u32 = 8826029;
pub const fnEprojPreInstr_GoldenTorizosChozoOrbs: u32 = 8826106;
pub const fnEprojInstr_GotoDependingOnXDirection: u32 = 8826258;
pub const fnEprojInit_TorizoSonicBoom: u32 = 8826389;
pub const fnEprojPreInstr_TorizoSonicBoom: u32 = 8826476;
pub const fnEprojInit_WreckedShipChozoSpikeFootsteps: u32 = 8826620;
pub const fnEprojInstr_ResetXYpos1: u32 = 8826678;
pub const fnEprojInit_TourianStatueDustClouds: u32 = 8826691;
pub const fnEprojInit_TourianLandingDustCloudsRightFoot: u32 = 8826704;
pub const fnEprojInstr_MoveY_Minus4: u32 = 8826770;
pub const fnEprojInstr_SetVelTowardsSamus1: u32 = 8827497;
pub const fnEprojInstr_SetVelTowardsSamus2: u32 = 8827506;
pub const fnEprojInit_TorizoLandingDustCloudLeftFoot: u32 = 8826829;
pub const fnEprojInit_GoldenTorizoEgg: u32 = 8826881;
pub const fnEprojPreInstr_GoldenTorizoEgg: u32 = 8826947;
pub const fnsub_86B0B9: u32 = 8827065;
pub const fnsub_86B0DD: u32 = 8827101;
pub const fnsub_86B13E: u32 = 8827198;
pub const fnEprojInit_GoldenTorizoSuperMissile: u32 = 8827342;
pub const fnEprojPreInstr_GoldenTorizoSuperMissile: u32 = 8827405;
pub const fnEprojPreInstr_B237: u32 = 8827447;
pub const fnEprojInit_GoldenTorizoEyeBeam: u32 = 8827688;
pub const fnEprojPreInstr_GoldenTorizoEyeBeam: u32 = 8827786;
pub const fnEprojInstr_GotoIfFunc1: u32 = 8827832;
pub const fnEprojInstr_ResetXYpos2: u32 = 8827958;
pub const fnEprojInit_TourianEscapeShaftFakeWallExplode: u32 = 8828061;
pub const fnnullsub_90: u32 = 8828080;
pub const fnEprojInit_LavaSeahorseFireball: u32 = 8828143;
pub const fnsub_86B535: u32 = 8828213;
pub const fnEprojInit_EyeDoorProjectile: u32 = 8828461;
pub const fnEprojInit_EyeDoorSweat: u32 = 8828547;
pub const fnEprojPreInstr_EyeDoorProjectile: u32 = 8828601;
pub const fnEprojPreInstr_EyeDoorSweat: u32 = 8828692;
pub const fnEprojInstr_SpawnTourianStatueUnlockingParticle: u32 = 8828906;
pub const fnEprojInstr_Earthquake: u32 = 8828917;
pub const fnEprojInstr_SpawnTourianStatueUnlockingParticleTail: u32 = 8828952;
pub const fnEprojInstr_AddToYpos: u32 = 8828993;
pub const fnEprojInit_TourianStatueUnlockingParticleWaterSplash: u32 = 8829050;
pub const fnEprojInit_TourianStatueEyeGlow: u32 = 8829070;
pub const fnEprojInit_TourianStatueUnlockingParticle: u32 = 8829109;
pub const fnEprojIni_TourianStatueUnlockingParticleTail: u32 = 8829160;
pub const fnEprojInit_TourianStatueSoul: u32 = 8829176;
pub const fnEprojInit_TourianStatueBaseDecoration: u32 = 8829246;
pub const fnEprojInit_TourianStatueRidley: u32 = 8829265;
pub const fnEprojInit_TourianStatuePhantoon: u32 = 8829284;
pub const fnEprojPreInstr_TourianStatueUnlockingParticleWaterSplash: u32 = 8829303;
pub const fnEprojPreInstr_TourianStatueUnlockingParticle: u32 = 8829314;
pub const fnEprojPreInstr_TourianStatueSoul: u32 = 8829437;
pub const fnEprojPreInstr_TourianStatueStuff: u32 = 8829495;
pub const fnEprojPreInstr_BA42: u32 = 8829506;
pub const fnsub_86BB30: u32 = 8829744;
pub const fnnullsub_91: u32 = 8829775;
pub const fnEprojInit_NuclearWaffleBody: u32 = 8829842;
pub const fnnullsub_92: u32 = 8829894;
pub const fnEprojInit_NorfairLavaquakeRocks: u32 = 8829915;
pub const fnEprojPreInstr_NorfairLavaquakeRocks: u32 = 8829967;
pub const fnEproj_NorfairLavaquakeRocks_Func1: i32 = 0x86bc16;
pub const fnEproj_NorfairLavaquakeRocks_Func2: i32 = 0x86bc8f;
pub const fnEprojPreInstr_NorfairLavaquakeRocks_Inner: u32 = 8830238;
pub const fnEprojPreInstr_NorfairLavaquakeRocks_Inner2: u32 = 8830250;
pub const fnEprojInit_ShaktoolAttackMiddleBackCircle: u32 = 8830364;
pub const fnEprojInit_BDA2: u32 = 8830370;
pub const fnEprojInit_ShaktoolAttackFrontCircle: u32 = 8830467;
pub const fnEprojPreInstr_BE12: u32 = 8830482;
pub const fnEprojInit_MotherBrainRoomTurrets: u32 = 8830543;
pub const fnEprojInit_MotherBrainRoomTurretBullets: u32 = 8830809;
pub const fnEprojPreInstr_MotherBrainRoomTurrets: u32 = 8830943;
pub const fnEprojPreInstr_MotherBrainRoomTurretBullets: u32 = 8831200;
pub const fnEprojInstr_SwitchJump: u32 = 8831347;
pub const fnEprojInstr_UserPalette0: u32 = 8831412;
pub const fnEproj_MotherBrainsBlueRingLasers: u32 = 8831731;
pub const fnEproj_MoveToBlueRingSpawnPosition: u32 = 8831797;
pub const fnEprojInit_MotherBrainBomb: u32 = 8832130;
pub const fnEprojPreInstr_MotherBrainBomb: u32 = 8832200;
pub const fnsub_86C605: u32 = 8832517;
pub const fnEprojInit_MotherBrainDeathBeemFired: u32 = 8832644;
pub const fnnullsub_366: u32 = 8832877;
pub const fnSpawnMotherBrainDeathBeam: u32 = 8833019;
pub const fnEprojInit_MotherBrainRainbowBeam: u32 = 8833034;
pub const fnEprojPreInstr_MotherBrainRainbowBeam: u32 = 8833044;
pub const fnEprojInit_MotherBrainsDrool: u32 = 8833091;
pub const fnEprojPreInstr_C84D: u32 = 8833101;
pub const fnEprojInit_MotherBrainsDrool_Falling: u32 = 8833158;
pub const fnEprojInstr_Add12ToY: u32 = 8833232;
pub const fnEprojInit_MotherBrainsDeathExplosion: u32 = 8833269;
pub const fnEprojPreInit_MotherBrainsDeathExplosion_0: u32 = 8833300;
pub const fnEprojInit_MotherBrainsRainbowBeamExplosion: u32 = 8833327;
pub const fnEprojPreInstr_MotherBrainsRainbowBeamExplosion: u32 = 8833356;
pub const fnEprojInit_MotherBrainEscapeDoorParticles: u32 = 8833377;
pub const fnEprojPreInstr_MotherBrainsExplodedDoorParticles: u32 = 8833490;
pub const fnEprojInit_MotherBrainPurpleBreathBig: u32 = 8833642;
pub const fnEprojInit_MotherBrainPurpleBreathSmall: u32 = 8833667;
pub const fnnullsub_94: u32 = 8833699;
pub const fnEprojInstr_MotherBrainPurpleBreathIsActive: u32 = 8833774;
pub const fnEprojInit_TimeBombSetJapaneseText: u32 = 8833782;
pub const fnEprojPreInstr_TimeBombSetJapaneseText: u32 = 8833786;
pub const fnEprojInit_MotherBrainTubeFalling: u32 = 8833993;
pub const fnEprojPreInstr_MotherBrainTubeFalling: u32 = 8834023;
pub const fnMotherBrainTubeFallingFunc_GenerateExplosion: i32 = 0x86cbea;
pub const fnMotherBrainTubeFallingFunc_Falling: i32 = 0x86cc08;
pub const fnEprojInit_MotherBrainGlassShatteringShard: u32 = 8834501;
pub const fnEprojInit_MotherBrainGlassShatteringSparkle: u32 = 8834669;
pub const fnEprojPreInstr_MotherBrainGlassShatteringShard: u32 = 8834715;
pub const fnEprojInit_KiHunterAcidSpitLeft: u32 = 8834960;
pub const fnEprojInit_KiHunterAcidSpitRight: u32 = 8834982;
pub const fnsub_86CFD5: u32 = 8835029;
pub const fnsub_86CFE6: u32 = 8835046;
pub const fnnullsub_95: u32 = 8835063;
pub const fnsub_86CFF8: i32 = 0x86cff8;
pub const fnEprojInit_KagosBugs: u32 = 8835208;
pub const fnEprojPreInstr_KagosBugs_Func1: u32 = 8835251;
pub const fnEprojPreInstr_KagosBugs: i32 = 0x86d0ca;
pub const fnnullsub_302: i32 = 0x86d0eb;
pub const fnEprojPreInstr_D0EC: i32 = 0x86d0ec;
pub const fnEprojPreInstr_D128: i32 = 0x86d128;
pub const fnEprojInstr_D15C: u32 = 8835420;
pub const fnEprojInstr_D1B6: u32 = 8835510;
pub const fnEprojInstr_D1C7: u32 = 8835527;
pub const fnEprojInstr_D1CE: u32 = 8835534;
pub const fnEprojPreInstr_KagosBugs_Func2: u32 = 8835556;
pub const fnEprojInit_MaridiaFloatersSpikes: u32 = 8835642;
pub const fnEprojPreInstr_MaridiaFloatersSpikes: u32 = 8835683;
pub const fnEprojInit_WreckedShipRobotLaserDown: u32 = 8835852;
pub const fnEprojInit_WreckedShipRobotLaserHorizontal: u32 = 8835886;
pub const fnEprojInit_WreckedShipRobotLaserUp: u32 = 8835905;
pub const fnEprojPreInstr_WreckedShipRobotLaser: u32 = 8836031;
pub const fnEprojInstr_AssignNewN00bTubeShardVelocity: u32 = 8836577;
pub const fnEprojInstr_SetN00bTubeShardX: u32 = 8836594;
pub const fnEprojInstr_D62A: u32 = 8836650;
pub const fnEprojInstr_SetXvelRandom: u32 = 8836762;
pub const fnEprojInit_N00bTubeCrack: u32 = 8836773;
pub const fnEprojInit_N00bTubeShards: u32 = 8836809;
pub const fnEprojInit_N00bTubeReleasedAirBubbles: u32 = 8836980;
pub const fnsub_86D7BF: u32 = 8837055;
pub const fnsub_86D7DE: u32 = 8837086;
pub const fnEprojPreInstr_N00bTubeShards: u32 = 8837117;
pub const fnsub_86D83D: u32 = 8837181;
pub const fnsub_86D89F: u32 = 8837279;
pub const fnsub_86D8DF: u32 = 8837343;
pub const fnsub_86D992: u32 = 8837522;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes: u32 = 8837595;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_MoveY1: u32 = 8837606;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_MoveY2: u32 = 8837648;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_MoveX1: u32 = 8837690;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_MoveX2: u32 = 8837732;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1: u32 = 8837774;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2: u32 = 8837779;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1: u32 = 8837784;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2: u32 = 8837789;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1: u32 = 8837794;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2: u32 = 8837802;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1: u32 = 8837810;
pub const fnEprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2: u32 = 8837818;
pub const fnEprojInit_DBF2: u32 = 8837912;
pub const fnEprojPreInstr_DBF2: u32 = 8837979;
pub const fnEprojPreInstr_DBF2_MoveX1: i32 = 0x86db62;
pub const fnEprojPreInstr_DBF2_MoveX2: i32 = 0x86db8c;
pub const fnEprojPreInstr_DBF2_Func1: u32 = 8838070;
pub const fnEprojInstr_DC5A: u32 = 8838234;
pub const fnEprojInstr_SpawnEnemyDrops_0: u32 = 8838241;
pub const fnEprojInstr_SpawnSporesEproj: u32 = 8838263;
pub const fnEprojInit_Spores: u32 = 8838285;
pub const fnEprojInit_SporeSpawnStalk: u32 = 8838307;
pub const fnEprojInit_SporeSpawners: u32 = 8838356;
pub const fnEprojPreInstr_Spores: u32 = 8838382;
pub const fnnullsub_96: u32 = 8838468;
pub const fnEprojPreInstr_SporeSpawners: u32 = 8838470;
pub const fnEprojInit_NamiFuneFireball: u32 = 8838870;
pub const fnEprojPreInstr_NamiFuneFireball: u32 = 8838969;
pub const fnEprojInit_NamiFuneFireball_MoveX1: i32 = 0x86df40;
pub const fnEprojInit_NamiFuneFireball_MoveX2: i32 = 0x86df6a;
pub const fnEprojInstr_DFEA: u32 = 8839146;
pub const fnEprojInit_LavaThrownByLavaman: u32 = 8839168;
pub const fnsub_86E049: u32 = 8839241;
pub const fnEproj_LavaThrownByLavaman_MoveX1: i32 = 0x86e050;
pub const fnEproj_LavaThrownByLavaman_MoveX2: i32 = 0x86e07a;
pub const fnEprojInit_DustCloudOrExplosion: u32 = 8840296;
pub const fnEprojInit_EyeDoorSmoke: u32 = 8840358;
pub const fnEprojPreInstr_DustCloudOrExplosion: u32 = 8840446;
pub const fnEprojPreInstr_nullsub_98: u32 = 8840456;
pub const fnEprojInstr_SetYVel: u32 = 8840499;
pub const fnEprojInit_SpawnedShotGate: u32 = 8840656;
pub const fnEprojInit_ClosedDownwardsShotGate: u32 = 8840661;
pub const fnEprojInit_ClosedUpwardsShotGate: u32 = 8840666;
pub const fnEprojPreInstr_nullsub_99: u32 = 8840708;
pub const fnEprojPreInstr_E605: u32 = 8840709;
pub const fnEprojInit_SaveStationElectricity: u32 = 8840877;
pub const fnnullsub_100: u32 = 8840913;
pub const fnEprojInit_BotwoonsBody: u32 = 8841777;
pub const fnEprojPreInstr_BotwoonsBody: u32 = 8841856;
pub const fnEproj_BotwoonsBody_Main: i32 = 0x86ea98;
pub const fnEproj_BotwonsBodyFunction_Dying: i32 = 0x86eaf4;
pub const fnEproj_BotwonsBodyFunction_Dying2: u32 = 8841988;
pub const fnEproj_BotwonsBodyFunction_DyingFalling: i32 = 0x86eb1f;
pub const fnnullsub_101: i32 = 0x86eb93;
pub const fnEprojInit_BotwoonsSpit: u32 = 8842182;
pub const fnEprojPreInstr_BotwoonsSpit: u32 = 8842245;
pub const fnEprojInit_YappingMawsBody: u32 = 8842338;
pub const fnEprojPreInstr_Empty2: u32 = 8842388;
pub const fnEprojInstr_ECE3: u32 = 8842467;
pub const fnEprojInstr_ED17: u32 = 8842519;
pub const fnEprojInstr_QueueSfx2_9: u32 = 8842891;
pub const fnEprojInstr_QueueSfx2_24: u32 = 8842903;
pub const fnEprojInstr_QueueSfx2_B: u32 = 8842915;
pub const fnEprojInstr_EEAF: u32 = 8842927;
pub const fnEprojInstr_HandleRespawningEnemy: u32 = 8843024;
pub const fnEprojInit_F337: u32 = 8843049;
pub const fnEprojInit_EnemyDeathExplosion: u32 = 8843145;
pub const fnEprojPreInstr_Empty: i32 = 0x86efdf;
pub const fnEprojPreInstr_Pickup: i32 = 0x86efe0;
pub const fnEprojInit_Sparks: u32 = 8844177;
pub const fnEprojPreInstr_Sparks: u32 = 8844272;
#[inline]
unsafe fn gEnemyData(o: i32, game: &mut Game) -> *mut EnemyData {
    (&raw mut game.g_ram as *mut u8)
        .offset(0xf78)
        .offset(o as isize) as *mut EnemyData
}
#[inline]
unsafe fn gEnemySpawnData(o: i32, game: &mut Game) -> *mut EnemySpawnData {
    (&raw mut game.g_ram as *mut u8)
        .offset(0x7000)
        .offset(o as isize) as *mut EnemySpawnData
}
#[inline]
unsafe fn gExtraEnemyRam7800(o: i32, game: &mut Game) -> *mut ExtraEnemyRam7800 {
    (&raw mut game.g_ram as *mut u8)
        .offset(0x7800)
        .offset(o as isize) as *mut ExtraEnemyRam7800
}
#[inline]
unsafe fn gExtraEnemyRam8000(o: i32, game: &mut Game) -> *mut ExtraEnemyRam8000 {
    (&raw mut game.g_ram as *mut u8)
        .offset(0x8000)
        .offset(o as isize) as *mut ExtraEnemyRam8000
}
#[inline]
unsafe fn Get_MotherBrain(o: i32, game: &mut Game) -> *mut Enemy_MotherBrain {
    (&raw mut game.g_ram as *mut u8)
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
    update_ram_word!(
        game.cinematic_var5,
        |fresh17| (fresh17 as i32 & !(0x8000i32)) as u16
    );
}
pub fn ClearEprojs(game: &mut Game) {
    for word_index in (0..18).rev() {
        EprojStateMut(&mut game.g_ram).set_id(word_index, 0);
    }
}
unsafe fn SpawnEprojInner(j: u16, gfx_idx: u16, game: &mut Game) -> Option<u16> {
    let mut v3: u16 = 34u16;
    while EprojState(&game.g_ram).id(usize::from(v3 >> 1)) != 0 {
        v3 = v3.wrapping_sub(2);
        if (v3 as i32 & 0x8000i32) != 0i32 {
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
pub unsafe fn SpawnEprojWithGfx(a: u16, k: u16, j: u16, game: &mut Game) -> u16 {
    EprojStateMut(&mut game.g_ram).write(0x1993, 0, a);
    let enemy_index = k;
    let gfx_idx = (game.enemy_state(enemy_index).vram_tiles_index())
        | (game.enemy_state(enemy_index).palette_index());
    SpawnEprojInner(j, gfx_idx, game).unwrap_or(gfx_idx)
}
pub unsafe fn SpawnEprojWithRoomGfx(j: u16, a: u16, game: &mut Game) -> u16 {
    EprojStateMut(&mut game.g_ram).write(0x1993, 0, a);
    SpawnEprojInner(j, 0, game)
        .unwrap_or_else(|| u16::from_le_bytes([game.g_ram[0x1997], game.g_ram[0x1998]]))
}
unsafe fn EprojRunOne(k: u16, game: &mut Game) {
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
        let value = *(p as *mut u16);
        EprojStateMut(&mut game.g_ram).set_instruction_timer(v7, value);
        let value = *(p.offset(2) as *mut u16);
        EprojStateMut(&mut game.g_ram).set_spritemap(v7, value);
        let value = p.offset(4).offset_from(base) as c_long as u16;
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
pub fn EprojInstr_Delete(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    null()
}
pub unsafe fn EprojInstr_Sleep(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let value = (epjp.offset_from(RomBankBase(0x86u8, game)) as c_long - 2 as c_long) as u16;
    EprojStateMut(&mut game.g_ram).set_instruction(usize::from(k >> 1), value);
    null()
}
pub unsafe fn EprojInstr_SetPreInstr_(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let value = *(epjp as *mut u16);
    EprojStateMut(&mut game.g_ram).set_preinstruction(usize::from(k >> 1), value);
    epjp.offset(2)
}
pub fn EprojInstr_ClearPreInstr(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    EprojStateMut(&mut game.g_ram).set_preinstruction(usize::from(k >> 1), 0x8170);
    epjp
}
pub unsafe fn CallEprojFunc(ea: u32, k: u32, game: &mut Game) {
    match ea {
        8833019 => {
            SpawnMotherBrainDeathBeam(k as u16, game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn EprojInstr_CallFunc(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    CallEprojFunc(Load24(epjp as *mut LongPtr), k as u32, game);
    epjp.offset(3)
}
pub unsafe fn EprojInstr_Goto(k: u16, epjp: *const u8) -> *const u8 {
    *(epjp as *mut u16) as usize as *const u8
}
pub unsafe fn EprojInstr_GotoRel(k: u16, epjp: *const u8) -> *const u8 {
    epjp.offset(*epjp as i8 as i32 as isize)
}
pub unsafe fn EprojInstr_DecTimerAndGotoIfNonZero(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    if EprojStateMut(&mut game.g_ram).decrement_timer(usize::from(k >> 1)) == 1 {
        epjp.offset(2)
    } else {
        EprojInstr_Goto(k, epjp)
    }
}
pub unsafe fn EprojInstr_DecTimerAndGotoRelIfNonZero(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    let v2: i32 = k as i32 >> 1i32;
    let ref mut fresh46 =
        *((&raw mut game.g_ram as *mut u8).offset(0x19df) as *mut u16).offset(v2 as isize);
    let fresh47 = *fresh46;
    *fresh46 = (*fresh46).wrapping_sub(1);
    if fresh47 == 1u16 {
        epjp.offset(1)
    } else {
        EprojInstr_GotoRel(k, epjp)
    }
}
pub unsafe fn EprojInstr_SetTimer(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let value = *(epjp as *mut u16);
    EprojStateMut(&mut game.g_ram).set_timer(usize::from(k >> 1), value);
    epjp.offset(2)
}
pub unsafe fn EprojInstr_MoveRandomlyWithinRadius(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    let mut Random: i8 = 0;
    let mut v4: i8 = 0;
    let mut v6: i16 = 0;
    let r18: u16 = NextRandom(game);
    loop {
        Random = NextRandom(game) as i8;
        v4 = ((*epjp as i32 & Random as i32) - *epjp.offset(1) as i32) as i8;
        if (v4 >= 0) {
            break;
        }
    }
    update_ram_word!(
        game.eproj_x_pos[(k as i32 >> 1i32)],
        |fresh44| (fresh44 as i32
            + if r18 as i32 & 0x8000i32 != 0 {
                -(v4 as u8 as i32)
            } else {
                v4 as u8 as i32
            }) as u16
    );
    loop {
        *(&raw mut v6 as *mut u8).offset(0) = NextRandom(game) as u8;
        *(&raw mut v6 as *mut u8).offset(0) =
            ((*epjp.offset(2) as i32 & v6 as i32) - *epjp.offset(3) as i32) as u8;
        if ((v6 as i32 & 0x80i32) == 0i32) {
            break;
        }
    }
    v6 = v6 as u8 as i16;
    if (r18 as i32 & 0x4000i32) != 0i32 {
        v6 = -(v6 as u8 as i32) as i16;
    }
    update_ram_word!(
        game.eproj_y_pos[(k as i32 >> 1i32)],
        wrapping_add((v6) as u16)
    );
    epjp.offset(4)
}
pub unsafe fn EprojInstr_SetProjectileProperties(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    let slot = usize::from(k >> 1);
    let properties = EprojState(&game.g_ram).slot_word(0x1bd7, slot);
    let mask = *(epjp as *mut u16);
    EprojStateMut(&mut game.g_ram).set_properties(slot, properties | mask);
    epjp.offset(2)
}
pub unsafe fn EprojInstr_ClearProjectileProperties(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    let slot = usize::from(k >> 1);
    let properties = EprojState(&game.g_ram).slot_word(0x1bd7, slot);
    let mask = *(epjp as *mut u16);
    EprojStateMut(&mut game.g_ram).set_properties(slot, properties & mask);
    epjp.offset(2)
}
pub unsafe fn EprojInstr_EnableCollisionWithSamusProj(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    update_ram_word!(
        game.eproj_properties[(k as i32 >> 1i32)],
        |fresh41| (fresh41 as i32 | 0x8000i32) as u16
    );
    epjp
}
pub unsafe fn EprojInstr_DisableCollisionWithSamusProj(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    update_ram_word!(
        game.eproj_properties[(k as i32 >> 1i32)],
        |fresh40| (fresh40 as i32 & !(0x8000i32)) as u16
    );
    epjp
}
pub unsafe fn EprojInstr_DisableCollisionWithSamus(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    update_ram_word!(
        game.eproj_properties[(k as i32 >> 1i32)],
        |fresh39| (fresh39 as i32 | 0x2000i32) as u16
    );
    epjp
}
pub unsafe fn EprojInstr_EnableCollisionWithSamus(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    update_ram_word!(
        game.eproj_properties[(k as i32 >> 1i32)],
        |fresh38| (fresh38 as i32 & !(0x2000i32)) as u16
    );
    epjp
}
pub unsafe fn EprojInstr_SetToNotDieOnContact(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    update_ram_word!(
        game.eproj_properties[(k as i32 >> 1i32)],
        |fresh37| (fresh37 as i32 | 0x4000i32) as u16
    );
    epjp
}
pub unsafe fn EprojInstr_SetToDieOnContact(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    update_ram_word!(
        game.eproj_properties[(k as i32 >> 1i32)],
        |fresh36| (fresh36 as i32 & !(0x4000i32)) as u16
    );
    epjp
}
pub unsafe fn EprojInstr_SetLowPriority(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    update_ram_word!(
        game.eproj_properties[(k as i32 >> 1i32)],
        |fresh35| (fresh35 as i32 | 0x1000i32) as u16
    );
    epjp
}
pub unsafe fn EprojInstr_SetHighPriority(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    update_ram_word!(
        game.eproj_properties[(k as i32 >> 1i32)],
        |fresh34| (fresh34 as i32 & !(0x1000i32)) as u16
    );
    epjp
}
pub unsafe fn EprojInstr_SetXyRadius(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.eproj_radius[(k as i32 >> 1i32)] = *(epjp as *mut u16));
    epjp.offset(2)
}
pub unsafe fn EprojInstr_SetXyRadiusZero(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.eproj_radius[(k as i32 >> 1i32)] = 0u16);
    epjp
}
pub unsafe fn EprojInstr_CalculateDirectionTowardsSamus(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    let v2: i32 = k as i32 >> 1i32;
    let x: u16 = (ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, eproj_x_pos[v2]));
    let y: u16 = (ram_word!(game, samus_y_pos)).wrapping_sub(ram_word!(game, eproj_y_pos[v2]));
    let v3: u16 = (2u16).wrapping_mul(CalculateAngleFromXY(x, y, game));
    let v4: i32 = ram_word!(game, cinematic_var7) as i32 >> 1i32;
    set_ram_word!(game.eproj_E[v4] = v3);
    let v5: i32 = v3 as i32 >> 1i32;
    set_ram_word!(game.eproj_x_vel[v4] = kSinCosTable8bit_Sext[(v5 + 64i32) as usize] as u16);
    set_ram_word!(game.eproj_y_vel[v4] = kSinCosTable8bit_Sext[v5 as usize] as u16);
    epjp
}
pub unsafe fn EprojInstr_WriteColorsToPalette(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    let mut v3: u16 = *(epjp.offset(2) as *mut u16);
    let mut n: i32 = *epjp.offset(4) as i32;
    let mut v4: u16 = *(epjp as *mut u16);
    loop {
        set_ram_word!(
            game.palette_buffer[(v3 as i32 >> 1i32)] = *(RomPtr_86(v4, game) as *mut u16)
        );
        v4 = (v4 as i32 + 2i32) as u16;
        v3 = (v3 as i32 + 2i32) as u16;
        n -= 1;
        if ((n & 0x8000i32) != 0i32) {
            break;
        }
    }
    epjp.offset(5)
}
pub unsafe fn EprojInstr_QueueMusic(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueMusic_Delayed8(*epjp as u16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx1_Max6(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx1_Max6(*epjp as u16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx2_Max6(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx2_Max6(*epjp as u16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx3_Max6(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx3_Max6(*epjp as u16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx1_Max15(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx1_Max15(*epjp as u16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx2_Max15(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx2_Max15(*epjp as u16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx3_Max15(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx3_Max15(*epjp as u16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx1_Max3(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx1_Max3(*epjp as u16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx2_Max3(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx2_Max3(*epjp as u16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx3_Max3(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx3_Max3(*epjp as u16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx1_Max9(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx1_Max9(*epjp as u16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx2_Max9(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx2_Max9(*epjp as u16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx3_Max9(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx3_Max9(*epjp as u16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx1_Max1(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx1_Max1(*epjp as u16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx2_Max1(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx2_Max1(*epjp as u16, game);
    epjp.offset(1)
}
pub unsafe fn EprojInstr_QueueSfx3_Max1(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx3_Max1(*epjp as u16, game);
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
pub unsafe fn DrawEprojs(k: u16, pt: Point16U, game: &mut Game) {
    let v1 = usize::from(k >> 1);
    let v2: u16 = EprojState(&game.g_ram).spritemap(v1);
    let r26: u16 = EprojState(&game.g_ram).gfx_low(v1) as u16;
    let R28: u16 = EprojState(&game.g_ram).slot_word(0x19bb, v1) & 0xff00;
    let r20: u16 =
        (pt.x as i32 + game.enemy_projectile_x_pos(k) as i32 - game.layer1_x_pos() as i32) as u16;
    if (r20 as i32 + 128i32 & 0xfe00i32) == 0i32 {
        let v3: u16 = (pt.y as i32 + game.enemy_projectile_y_pos(k) as i32
            - game.layer1_y_pos() as i32) as u16;
        let r18: u16 = v3;
        if (v3 as i32 & 0xff00i32) != 0i32 {
            if (v3 as i32 + 128i32 & 0xfe00i32) == 0i32 {
                DrawEprojSpritemapWithBaseTileOffscreen(0x8du8, v2, r20, r18, r26, R28, game);
            }
        } else {
            DrawEprojSpritemapWithBaseTile(0x8du8, v2, r20, r18, r26, R28, game);
        }
    }
}
pub unsafe fn GetValuesForScreenShaking(game: &mut Game) -> Point16U {
    let mut R34: u16 = 0;
    let mut R36: u16 = 0;
    if ram_word!(game, earthquake_timer) != 0u16
        && game.time_is_frozen_flag() == 0
        && ram_word!(game, earthquake_type) as i32 - 36i32 & 0x8000i32 != 0
    {
        let v0: i32 = ((4u16).wrapping_mul(ram_word!(game, earthquake_type))) as i32 >> 1i32;
        if (ram_word!(game, earthquake_timer) as i32 & 2i32) != 0i32 {
            R36 = -(crate::rtl::rom_table_word!(game, 0x86846b, v0 as isize) as i32) as u16;
            R34 =
                -(crate::rtl::rom_table_word!(game, 0x86846b, (v0 + 1i32) as isize) as i32) as u16;
        } else {
            R36 = crate::rtl::rom_table_word!(game, 0x86846b, v0 as isize);
            R34 = crate::rtl::rom_table_word!(game, 0x86846b, (v0 + 1i32) as isize);
        }
    } else {
        R34 = 0u16;
        R36 = 0u16;
    }
    Point16U { x: R36, y: R34 }
}
unsafe fn EprojColl_8506(eci: *mut EprojCollInfo, game: &mut Game) -> u8 {
    if *(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(ram_word!(game, cur_block_index) as isize)
        != 0
    {
        update_ram_word!(game.cur_block_index, |fresh1| (fresh1 as i32
            + *(&raw mut game.g_ram as *mut u8)
                .offset(0x16402)
                .offset(ram_word!(game, cur_block_index) as isize) as i8 as i32)
            as u16);
        return 0xffu8;
    }
    0u8
}
unsafe fn EprojColl_Unknown8536(eci: *mut EprojCollInfo, game: &mut Game) -> u8 {
    if *(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(game.cur_block_index() as isize)
        != 0
    {
        update_ram_word!(game.cur_block_index, |fresh0| (fresh0 as i32
            + *(&raw mut game.g_ram as *mut u8)
                .offset(0x16402)
                .offset(ram_word!(game, cur_block_index) as isize) as i8 as i32
                * ram_word!(game, room_width_in_blocks) as i32)
            as u16);
        return 0xffu8;
    }
    0u8
}
unsafe fn EprojColl_858E(eci: *mut EprojCollInfo, game: &mut Game) -> u8 {
    let v0: u16 = ((*(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(game.cur_block_index() as isize)) as u16)
        & (0x1fu16);
    if v0 < 5 {
        return EprojColl_85C2(eci, v0, ram_word!(game, cur_block_index), game);
    }
    set_ram_word!(
        game.current_slope_bts = *(&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset(game.cur_block_index() as isize) as u16
    );
    EprojColl_873D()
}
unsafe fn EprojColl_85AD(eci: *mut EprojCollInfo, game: &mut Game) -> u8 {
    let v0: u16 = ((*(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(game.cur_block_index() as isize)) as u16)
        & (0x1fu16);
    if v0 >= 5 {
        EprojColl_874E(eci, game)
    } else {
        EprojColl_8676(eci, v0, game.cur_block_index(), 0u16, game)
    }
}
static unk_868729: [u8; 20] = [
    0u8, 1u8, 0x82u8, 0x83u8, 0u8, 0x81u8, 2u8, 0x83u8, 0u8, 1u8, 2u8, 0x83u8, 0u8, 0x81u8, 0x82u8,
    0x83u8, 0x80u8, 0x81u8, 0x82u8, 0x83u8,
];
unsafe fn EprojColl_85C2(eci: *mut EprojCollInfo, a: u16, k: u16, game: &mut Game) -> u8 {
    let mut current_block: u64;
    let temp_collision_DD4: u16 = (4i32 * a as i32) as u16;
    let temp_collision_DD6: u16 = (*(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(k as isize) as i32
        >> 6i32) as u16;
    let v2: u16 = (4i32 * a as i32
        + (temp_collision_DD6 as i32 ^ ((*eci).eci_r34 as i32 & 8i32) >> 3i32))
        as u16;
    if (*eci).eci_r32 == 0 {
        let v3: i32 = ram_word!(game, cinematic_var7) as i32 >> 1i32;
        if (ram_word!(game, eproj_y_pos[v3]) as i32 - (*eci).eci_r30 as i32 & 8i32) != 0i32
            || (if unk_868729[v2 as usize] as i32 & 0x80i32 != 0 {
                -1i32
            } else {
                0i32
            }) >= 0i32
        {
            let v4: u16 = (v2 as i32 ^ 2i32) as u16;
            if ((*eci).eci_r30 as i32 + ram_word!(game, eproj_y_pos[v3]) as i32 - 1i32 & 8i32)
                == 0i32
                || (if unk_868729[v4 as usize] as i32 & 0x80i32 != 0 {
                    -1i32
                } else {
                    0i32
                }) >= 0i32
            {
                return 0u8;
            }
        }
    } else {
        if (*eci).eci_r26 == 0 {
            if ((*eci).eci_r30 as i32
                + *((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16)
                    .offset((ram_word!(game, cinematic_var7) as i32 >> 1i32) as isize)
                    as i32
                - 1i32
                & 8i32)
                == 0i32
            {
                if (if unk_868729[v2 as usize] as i32 & 0x80i32 != 0 {
                    -1i32
                } else {
                    0i32
                }) >= 0i32
                {
                    return 0u8;
                }
                current_block = 12039483399334584727;
            } else {
                current_block = 2204372562120708137;
            }
        } else if (*eci).eci_r26 != (*eci).eci_r32
            || (*((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16)
                .offset((ram_word!(game, cinematic_var7) as i32 >> 1i32) as isize)
                as i32
                - (*eci).eci_r30 as i32
                & 8i32)
                == 0i32
        {
            current_block = 2204372562120708137;
        } else {
            current_block = 3512920355445576850;
        }
        if current_block == 12039483399334584727 {
        } else {
            if current_block == 2204372562120708137 {
                if (if unk_868729[v2 as usize] as i32 & 0x80i32 != 0 {
                    -1i32
                } else {
                    0i32
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
                    if (if unk_868729[(v2 as i32 ^ 2i32) as usize] as i32 & 0x80i32 != 0 {
                        -1i32
                    } else {
                        0i32
                    }) >= 0i32
                    {
                        return 0u8;
                    }
                }
            }
        }
    }
    let v6: i32 = ram_word!(game, cinematic_var7) as i32 >> 1i32;
    let mut v7: u16 = 0;
    set_ram_word!(game.eproj_x_subpos[v6] = 0u16);
    if ((*eci).eci_r20 as i32 & 0x8000i32) != 0i32 {
        v7 = ((*eci).eci_r28 as i32 + ((*eci).eci_r34 as i32 | 7i32) + 1i32) as u16;
    } else {
        v7 = (((*eci).eci_r34 as i32 & 0xfff8i32) - (*eci).eci_r28 as i32) as u16;
    }
    set_ram_word!(game.eproj_x_pos[v6] = v7);
    1u8
}
unsafe fn EprojColl_8676(eci: *mut EprojCollInfo, a: u16, k: u16, j: u16, game: &mut Game) -> u8 {
    let mut current_block: u64;
    let v2: u16 = EprojState(&game.g_ram).index();
    let temp_collision_DD4: u16 = (4u16).wrapping_mul(a);
    let temp_collision_DD6: u16 = (*(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(k as isize) as i32
        >> 6i32) as u16;
    let v3: u16 = (4i32 * a as i32
        + (temp_collision_DD6 as i32 ^ ((*eci).eci_r34 as i32 & 8i32) >> 2i32))
        as u16;
    if (*eci).eci_r32 == 0 {
        let v4: i32 = v2 as i32 >> 1i32;
        if (game.enemy_projectile_x_pos(v2) as i32 - (*eci).eci_r28 as i32 & 8i32) != 0i32
            || (if unk_868729[v3 as usize] as i32 & 0x80i32 != 0 {
                -1i32
            } else {
                0i32
            }) >= 0i32
        {
            let v5: u16 = (v3) ^ (1u16);
            if ((*eci).eci_r28 as i32 + game.enemy_projectile_x_pos(v2) as i32 - 1i32 & 8i32)
                == 0i32
                || (if unk_868729[v5 as usize] as i32 & 0x80i32 != 0 {
                    -1i32
                } else {
                    0i32
                }) >= 0i32
            {
                return 0u8;
            }
        }
    } else {
        if (*eci).eci_r26 == 0 {
            if ((*eci).eci_r28 as i32
                + *((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16)
                    .offset((ram_word!(game, cinematic_var7) as i32 >> 1i32) as isize)
                    as i32
                - 1i32
                & 8i32)
                == 0i32
            {
                if (if unk_868729[v3 as usize] as i32 & 0x80i32 != 0 {
                    -1i32
                } else {
                    0i32
                }) >= 0i32
                {
                    return 0u8;
                }
                current_block = 5143058163439228106;
            } else {
                current_block = 8777407941634159499;
            }
        } else if (*eci).eci_r26 != (*eci).eci_r32
            || (*((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16)
                .offset((ram_word!(game, cinematic_var7) as i32 >> 1i32) as isize)
                as i32
                - (*eci).eci_r28 as i32
                & 8i32)
                == 0i32
        {
            current_block = 8777407941634159499;
        } else {
            current_block = 12349973810996921269;
        }
        if current_block == 5143058163439228106 {
        } else {
            if current_block == 8777407941634159499 {
                if (if unk_868729[v3 as usize] as i32 & 0x80i32 != 0 {
                    -1i32
                } else {
                    0i32
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
                    if (if unk_868729[(v3 as i32 ^ 1i32) as usize] as i32 & 0x80i32 != 0 {
                        -1i32
                    } else {
                        0i32
                    }) >= 0i32
                    {
                        return 0u8;
                    }
                }
            }
        }
    }
    let v7: i32 = EprojState(&game.g_ram).index() as i32 >> 1i32;
    let mut v8: u16 = 0;
    EprojStateMut(&mut game.g_ram).write(0x1a6f, v7 as usize, 0u16);
    if ((*eci).eci_r20 as i32 & 0x8000i32) != 0i32 {
        v8 = ((*eci).eci_r30 as i32 + ((*eci).eci_r34 as i32 | 7i32) + 1i32) as u16;
    } else {
        v8 = (((*eci).eci_r34 as i32 & 0xfff8i32) - (*eci).eci_r30 as i32) as u16;
    }
    EprojStateMut(&mut game.g_ram).write(0x1a93, v7 as usize, v8);
    1u8
}
pub fn EprojColl_873D() -> u8 {
    0u8
}
unsafe fn EprojColl_874E(eci: *mut EprojCollInfo, game: &mut Game) -> u8 {
    let mut v3: i16 = 0;
    let mut v5: i16 = 0;
    let mut v6: u16 = 0;
    let mut v7: u16 = 0;
    let mut v8: i16 = 0;
    let mut v11: i16 = 0;
    let mut v12: i16 = 0;
    let mut v13: u16 = 0;
    let mut v14: u16 = 0;
    let mut v15: i16 = 0;
    let v0: u16 = EprojState(&game.g_ram).index();
    if ((*eci).eci_r20 as i32 & 0x8000i32) != 0i32 {
        let v9: u16 = ram_word!(game, cur_block_index);
        let mod_0: u16 = SnesModulus(
            ram_word!(game, cur_block_index),
            ram_word!(game, room_width_in_blocks) as u8,
        );
        let v10: i32 = v0 as i32 >> 1i32;
        v11 = (ram_word!(game, eproj_x_pos[v10]) as i32 >> 4i32) as i16;
        if v11 as i32 == mod_0 as i32 {
            let temp_collision_DD4: u16 =
                ((*eci).eci_r24 as i32 - (*eci).eci_r30 as i32 & 0xfi32 ^ 0xfi32) as u16;
            let temp_collision_DD6: u16 = (16i32
                * (*(&raw mut game.g_ram as *mut u8)
                    .offset(0x16402)
                    .offset(v9 as isize) as i32
                    & 0x1fi32)) as u16;
            v12 = ((*(&raw mut game.g_ram as *mut u8)
                .offset(0x16402)
                .offset(v9 as isize) as i32)
                << 8i32) as i16;
            if v12 < 0i16 && {
                if (v12 as i32 & 0x4000i32) != 0i32 {
                    v13 = (ram_word!(game, eproj_x_pos[v10]) as i32 ^ 0xfi32) as u16;
                } else {
                    v13 = ram_word!(game, eproj_x_pos[v10]);
                };
                v14 = (temp_collision_DD6 as i32 + (v13 as i32 & 0xfi32)) as u16;
                v15 = ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x948b2b)) as *const u8
                    as *mut u8)
                    .offset(v14 as isize) as i32
                    & 0x1fi32)
                    - temp_collision_DD4 as i32
                    - 1i32) as i16;
                ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x948b2b)) as *const u8 as *mut u8)
                    .offset(v14 as isize) as i32
                    & 0x1fi32)
                    - temp_collision_DD4 as i32)
                    == 1i32
                    || v15 < 0i16
            } {
                set_ram_word!(game.eproj_y_pos[v10] = ((*eci).eci_r24 as i32 - v15 as i32) as u16);
                set_ram_word!(game.eproj_y_subpos[v10] = 0u16);
                1u8
            } else {
                0u8
            }
        } else {
            0u8
        }
    } else {
        let v1: u16 = game.cur_block_index();
        let mod_1: u16 = SnesModulus(game.cur_block_index(), game.room_width_in_blocks() as u8);
        let v2: i32 = v0 as i32 >> 1i32;
        v3 = (game.enemy_projectile_x_pos(v0) as i32 >> 4i32) as i16;
        if v3 as i32 == mod_1 as i32 {
            let temp_collision_DD4_0: u16 =
                (((*eci).eci_r30 as i32 + (*eci).eci_r24 as i32 - 1i32) as u16) & (0xfu16);
            let temp_collision_DD6_0: u16 = (16u16).wrapping_mul(
                (*(&raw mut game.g_ram as *mut u8)
                    .offset(0x16402)
                    .offset(v1 as isize) as i32
                    & 0x1fi32) as u16,
            );
            v5 = ((*(&raw mut game.g_ram as *mut u8)
                .offset(0x16402)
                .offset(v1 as isize) as i32)
                << 8i32) as i16;
            if v5 >= 0 && {
                if (v5 as i32 & 0x4000i32) != 0i32 {
                    v6 = (game.enemy_projectile_x_pos(v0)) ^ (0xfu16);
                } else {
                    v6 = game.enemy_projectile_x_pos(v0);
                };
                v7 = (temp_collision_DD6_0).wrapping_add((v6 as i32 & 0xfi32) as u16);
                v8 = (((*(game.g_rom.offset(crate::rtl::lorom_offset(0x948b2b)) as *const u8
                    as *mut u8)
                    .offset(v7 as isize) as i32
                    & 0x1fi32)
                    - temp_collision_DD4_0 as i32) as i16)
                    .wrapping_sub(1i16);
                ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x948b2b)) as *const u8 as *mut u8)
                    .offset(v7 as isize) as i32
                    & 0x1fi32)
                    - temp_collision_DD4_0 as i32)
                    == 1i32
                    || v8 < 0
            } {
                {
                    let eproj_position: u16 = ((*eci).eci_r24).wrapping_add((v8) as u16);
                    EprojStateMut(&mut game.g_ram).write(0x1a93, v2 as usize, eproj_position);
                }
                EprojStateMut(&mut game.g_ram).write(0x1a6f, v2 as usize, -1i32 as u16);
                1u8
            } else {
                0u8
            }
        } else {
            0u8
        }
    }
}
unsafe fn react_to_enemy_projectile_block(
    eci: *mut EprojCollInfo,
    k: u16,
    slope: Func_EprojCollInfo_U8,
    game: &mut Game,
) -> u8 {
    set_ram_word!(game.cur_block_index = (k as i32 >> 1i32) as u16);
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
    k: u16,
    game: &mut Game,
) -> u8 {
    react_to_enemy_projectile_block(eci, k, EprojColl_858E, game)
}
pub unsafe fn EprojBlockCollisition_CheckVertical(
    eci: *mut EprojCollInfo,
    k: u16,
    game: &mut Game,
) -> u8 {
    react_to_enemy_projectile_block(eci, k, EprojColl_85AD, game)
}
pub unsafe fn EprojBlockCollisition_Horiz(k: u16, game: &mut Game) -> u8 {
    let v1: i32 = k as i32 >> 1i32;
    let amt: i32 = (EprojState(&game.g_ram).slot_word(0x1ab7, v1 as usize) as i16 as i32) << 8i32;
    let R30: u16 = EprojState(&game.g_ram).y_radius(v1 as usize) as u16;
    let R28: u16 = EprojState(&game.g_ram).x_radius(v1 as usize) as u16;
    let mut r26: u16 = ((game.enemy_projectile_y_pos(k) as i32 - R30 as i32) as u16) & (0xfff0u16);
    r26 = (R30
        .wrapping_add(game.enemy_projectile_y_pos(k))
        .wrapping_sub(1)
        .wrapping_sub(r26) as i32
        >> 4i32) as u16;
    let R32: u16 = r26;
    let prod: u16 = Mult8x8(
        (game.enemy_projectile_y_pos(k).wrapping_sub(R30) as i32 >> 4i32) as u8,
        game.room_width_in_blocks() as u8,
    );
    let v3: u16 = ((amt as u32).wrapping_add(
        (game.enemy_projectile_x_pos(k) as u32) << 16i32
            | EprojState(&game.g_ram).slot_word(0x1a27, v1 as usize) as u32,
    ) >> 16i32) as u16;
    let r22: u16 = (amt + EprojState(&game.g_ram).slot_word(0x1a27, v1 as usize) as i32) as u16;
    let r24: u16 = v3;
    let mut v4: u16 = 0;
    if amt < 0i32 {
        v4 = v3.wrapping_sub(R28);
    } else {
        v4 = R28.wrapping_add(v3).wrapping_sub(1);
    }
    let mut eci: EprojCollInfo = EprojCollInfo {
        eci_r20: (amt >> 16i32) as u16,
        eci_r24: r24,
        eci_r26: r26,
        eci_r28: R28,
        eci_r30: R30,
        eci_r32: R32,
        eci_r34: v4,
    };
    let mut v5: u16 = (2i32 * (prod as i32 + (v4 as i32 >> 4i32))) as u16;
    while EprojBlockCollisition_CheckHorizontal(&raw mut eci, v5, game) as i32 & 1i32 == 0 {
        v5 = (v5).wrapping_add((game.room_width_in_blocks() as i32 * 2i32) as u16);
        eci.eci_r26 = eci.eci_r26.wrapping_sub(1);
        if (eci.eci_r26 as i32 & 0x8000i32) != 0i32 {
            let v6: i32 = k as i32 >> 1i32;
            EprojStateMut(&mut game.g_ram).write(0x1a27, v6 as usize, r22);
            EprojStateMut(&mut game.g_ram).write(0x1a4b, v6 as usize, r24);
            return 0u8;
        }
    }
    let v8: i32 = k as i32 >> 1i32;
    EprojStateMut(&mut game.g_ram).write(0x1a27, v8 as usize, 0u16);
    if amt < 0i32 {
        let v10: u16 = (eci.eci_r28 as i32 + (eci.eci_r34 as i32 | 0xfi32) + 1i32) as u16;
        if v10 <= game.enemy_projectile_x_pos(k) {
            EprojStateMut(&mut game.g_ram).write(0x1a4b, v8 as usize, v10);
        }
    } else {
        let v9: u16 = ((eci.eci_r34 as i32 & 0xfff0i32) - eci.eci_r28 as i32) as u16;
        if v9 >= game.enemy_projectile_x_pos(k) {
            EprojStateMut(&mut game.g_ram).write(0x1a4b, v8 as usize, v9);
        }
    }
    1u8
}
pub unsafe fn EprojBlockCollisition_Vertical(k: u16, game: &mut Game) -> u8 {
    let mut v5: i16 = 0;
    let v1: i32 = k as i32 >> 1i32;
    let amt: i32 = (EprojState(&game.g_ram).slot_word(0x1adb, v1 as usize) as i16 as i32) << 8i32;
    let R30: u16 = EprojState(&game.g_ram).y_radius(v1 as usize) as u16;
    let R28: u16 = EprojState(&game.g_ram).x_radius(v1 as usize) as u16;
    let mut r26: u16 = ((game.enemy_projectile_x_pos(k) as i32 - R28 as i32) as u16) & (0xfff0u16);
    r26 = (R28
        .wrapping_add(game.enemy_projectile_x_pos(k))
        .wrapping_sub(1)
        .wrapping_sub(r26) as i32
        >> 4i32) as u16;
    let R32: u16 = r26;
    let v3: u16 = ((amt as u32).wrapping_add(
        (game.enemy_projectile_y_pos(k) as u32) << 16i32
            | EprojState(&game.g_ram).slot_word(0x1a6f, v1 as usize) as u32,
    ) >> 16i32) as u16;
    let r22: u16 = (amt + EprojState(&game.g_ram).slot_word(0x1a6f, v1 as usize) as i32) as u16;
    let r24: u16 = v3;
    let mut v4: u16 = 0;
    if amt < 0i32 {
        v4 = v3.wrapping_sub(R30);
    } else {
        v4 = R30.wrapping_add(v3).wrapping_sub(1);
    }
    let mut eci: EprojCollInfo = EprojCollInfo {
        eci_r20: (amt >> 16i32) as u16,
        eci_r24: r24,
        eci_r26: r26,
        eci_r28: R28,
        eci_r30: R30,
        eci_r32: R32,
        eci_r34: v4,
    };
    let prod: u16 = Mult8x8((v4 as i32 >> 4i32) as u8, game.room_width_in_blocks() as u8);
    v5 = (game.enemy_projectile_x_pos(k).wrapping_sub(R28) as i32 >> 4i32) as i16;
    let mut i: i32 = 2i32 * (prod as i32 + v5 as i32);
    while EprojBlockCollisition_CheckVertical(&raw mut eci, i as u16, game) as i32 & 1i32 == 0 {
        eci.eci_r26 = eci.eci_r26.wrapping_sub(1);
        if (eci.eci_r26 as i32 & 0x8000i32) != 0i32 {
            let v7: i32 = k as i32 >> 1i32;
            EprojStateMut(&mut game.g_ram).write(0x1a6f, v7 as usize, r22);
            EprojStateMut(&mut game.g_ram).write(0x1a93, v7 as usize, r24);
            return 0u8;
        }
        i += 2i32;
    }
    let v9: i32 = k as i32 >> 1i32;
    EprojStateMut(&mut game.g_ram).write(0x1a6f, v9 as usize, 0u16);
    if amt < 0i32 {
        let v11: u16 = (eci.eci_r30 as i32 + (eci.eci_r34 as i32 | 0xfi32) + 1i32) as u16;
        if v11 <= game.enemy_projectile_y_pos(k) {
            EprojStateMut(&mut game.g_ram).write(0x1a93, v9 as usize, v11);
        }
    } else {
        let v10: u16 = ((eci.eci_r34 as i32 & 0xfff0i32) - eci.eci_r30 as i32) as u16;
        if v10 >= game.enemy_projectile_y_pos(k) {
            EprojStateMut(&mut game.g_ram).write(0x1a93, v9 as usize, v10);
        }
    }
    1u8
}
unsafe fn Eproj_Init_0x8aaf(j: u16, game: &mut Game) {
    NextRandom(game);
    let v1: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: u16 = ((*v1).y_pos as i32 + 12i32) as u16;
    let v3: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_y_pos[v3] = v2);
    set_ram_word!(game.eproj_timers[v3] = (v2 as i32 + 72i32) as u16);
    set_ram_word!(game.eproj_y_vel[v3] = (*v1).ai_var_D);
    set_ram_word!(
        game.eproj_x_pos[v3] =
            ((*v1).x_pos as i32 + (ram_word!(game, random_number) as i32 & 0x1fi32) - 16i32) as u16
    );
    set_ram_word!(
        game.eproj_instr_list_ptr[v3] = crate::rtl::rom_table_word!(
            game,
            0x868a75,
            ((*(((&raw mut game.g_ram as *mut u8).offset(0x1adb) as *mut u16).offset(v3 as isize)
                as *mut u16 as *mut u8)
                .offset(1isize,) as i32
                & 6i32) as u16 as i32
                >> 1i32) as isize,
        )
    );
}
pub unsafe fn Eproj_PreInit_0x8aaf(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((ram_word!(game, eproj_y_vel[v1]) as i16 as i32) << 8i32) as u32,
    );
    if ram_word!(game, eproj_y_pos[v1]) >= ram_word!(game, eproj_timers[v1]) {
        set_ram_word!(game.eproj_id[v1] = 0u16);
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

pub fn Eproj_PreInstr_SkreeParticle(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let split_amount =
        ((EprojState(&game.g_ram).slot_word(0x1ab7, v1 as usize) as i16 as i32) << 8i32) as u32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a4b, 0x1a27, v1 as usize, split_amount);
    let split_amount =
        ((EprojState(&game.g_ram).slot_word(0x1adb, v1 as usize) as i16 as i32) << 8i32) as u32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a93, 0x1a6f, v1 as usize, split_amount);
    EprojStateMut(&mut game.g_ram).add_word(0x1adb, v1 as usize, 80);
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        EprojStateMut(&mut game.g_ram).set_id(v1 as usize, 0);
    }
}
pub unsafe fn EprojInstr_SpawnEnemyDropsWithDraygonsEyeDrops(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    let v1: i32 = k as i32 >> 1i32;
    game.eproj_spawn_pt = Point16U {
        x: ram_word!(game, eproj_x_pos[v1]),
        y: ram_word!(game, eproj_y_pos[v1]),
    };
    SpawnEnemyDrops(addr_kEnemyDef_DE7F as u16, k, 0u16, game);
    epjp
}
pub unsafe fn EprojInstr_SetPreInstrA(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.eproj_pre_instr[(k as i32 >> 1i32)] = (0x868dffi32 & 0xffffi32) as u16);
    epjp
}
pub unsafe fn EprojInstr_SetPreInstrB(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.eproj_pre_instr[(k as i32 >> 1i32)] = (0x868dcai32 & 0xffffi32) as u16);
    epjp
}
unsafe fn EprojInit_DraygonsGunk(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v1] = game.eproj_spawn_pt.x);
    set_ram_word!(game.eproj_y_pos[v1] = game.eproj_spawn_pt.y);
    set_ram_word!(game.g_word_7E97DC[v1] = ram_word!(game, cinematic_var9));
    let pt: Point32 = ConvertAngleToXy(
        ram_word!(game, cinematic_var9),
        ram_word!(game, cinematic_var8),
        game,
    );
    set_ram_word!(game.eproj_x_vel[v1] = (pt.x >> 16i32) as u16);
    set_ram_word!(game.eproj_E[v1] = pt.x as u16);
    set_ram_word!(game.eproj_y_vel[v1] = (pt.y >> 16i32) as u16);
    set_ram_word!(game.eproj_F[v1] = pt.y as u16);
    set_ram_word!(game.eproj_gfx_idx[v1] = 1024u16);
}
unsafe fn EprojInit_DraygonsWallTurretProjs(j: u16, game: &mut Game) {
    Eproj_AngleToSamus(j, game.eproj_spawn_pt.x, game.eproj_spawn_pt.y, game);
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_gfx_idx[v1] = 2560u16);
    set_ram_word!(game.eproj_pre_instr[v1] = (0x868d54i32 & 0xffffi32) as u16);
}
pub unsafe fn EprojPowerBombCollision(k: u16, game: &mut Game) {
    let r18: u16 = *((&raw mut game.g_ram as *mut u8).offset(0xcea) as *mut u16 as *mut u8)
        .offset(1isize) as u16;
    if *((&raw mut game.g_ram as *mut u8).offset(0xcea) as *mut u16 as *mut u8).offset(1isize) != 0
    {
        let r20: u16 = ((r18 as i32
            + (*((&raw mut game.g_ram as *mut u8).offset(0xcea) as *mut u16 as *mut u8)
                .offset(1isize) as i32
                & 1i32)
            + (ram_word!(game, power_bomb_explosion_radius) as i32 >> 9i32))
            as u16 as i32
            >> 1i32) as u16;
        let v1: i32 = k as i32 >> 1i32;
        if abs16(
            (ram_word!(game, power_bomb_explosion_x_pos) as i32
                - ram_word!(game, eproj_x_pos[v1]) as i32) as u16,
        ) < r18
            && abs16(
                (ram_word!(game, power_bomb_explosion_y_pos) as i32
                    - ram_word!(game, eproj_y_pos[v1]) as i32) as u16,
            ) < r20
        {
            set_ram_word!(game.eproj_id[v1] = 0u16);
            set_ram_word!(game.samus_x_speed_divisor = 0u16);
        }
    }
}
pub unsafe fn EprojInstr_868D99(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    EprojPowerBombCollision(k, game);
    let v2: u16 = (ram_word!(game, samus_x_speed_divisor) as i32 + 1i32) as u16;
    if ram_word!(game, samus_x_speed_divisor) as i32 - 5i32 & 0x8000i32 != 0 {
        update_ram_word!(game.samus_x_speed_divisor, wrapping_add(1));
        let v3: i32 = k as i32 >> 1i32;
        set_ram_word!(game.eproj_F[v3] = v2);
        set_ram_word!(game.eproj_E[v3] = 256u16);
        set_ram_word!(game.eproj_pre_instr[v3] = (0x868dcai32 & 0xffffi32) as u16);
        set_ram_word!(
            game.eproj_properties[v3] =
                (ram_word!(game, eproj_properties[v3]) as i32 & 0x5fffi32 | 0x2000i32) as u16
        );
        set_ram_word!(game.samus_invincibility_timer = 0u16);
        set_ram_word!(game.samus_knockback_timer = 0u16);
    }
    epjp
}
pub unsafe fn EprojPreInstr_8DCA(k: u16, game: &mut Game) {
    let mut v1: i32 = 0;
    EprojPowerBombCollision(k, game);
    if ram_word!(game, samus_contact_damage_index) != 0u16 || {
        v1 = k as i32 >> 1i32;
        set_ram_word!(game.eproj_x_pos[v1] = ram_word!(game, samus_x_pos));
        set_ram_word!(
            game.eproj_y_pos[v1] = (ram_word!(game, samus_y_pos) as i32
                + 4i32 * ram_word!(game, eproj_F[v1]) as i32
                - 12i32) as u16
        );
        update_ram_word!(game.eproj_E[v1], wrapping_sub(1));
        ram_word!(game, eproj_E[v1]) == 0
    } {
        set_ram_word!(game.eproj_id[(k as i32 >> 1i32)] = 0u16);
        if (update_ram_word!(game.samus_x_speed_divisor, wrapping_sub(1), reread) as i32
            & 0x8000i32)
            != 0i32
        {
            set_ram_word!(game.samus_x_speed_divisor = 0u16);
        }
    }
}
pub unsafe fn EprojPreInstr_DraygonsTurret_8DFF(k: u16, game: &mut Game) {
    let mut v1: i16 = 0;
    EprojPowerBombCollision(k, game);
    Eproj_FuncE73E_MoveXY(k, game);
    v1 = Eproj_FuncE722(k, game) as i16;
    if v1 != 0 {
        set_ram_word!(game.eproj_id[(k as i32 >> 1i32)] = 0u16);
    }
}
pub unsafe fn EprojPreInstr_DraygonsGunk_8E0F(k: u16, game: &mut Game) {
    let mut v3: u16 = 0;
    EprojPowerBombCollision(k, game);
    Eproj_FuncE73E_MoveXY(k, game);
    let v1: i32 = k as i32 >> 1i32;
    let v2: u16 = abs16(
        (ram_word!(game, samus_x_pos) as i32 - ram_word!(game, eproj_x_pos[v1]) as i32) as u16,
    );
    if v2 as i32 - 16i32 & 0x8000i32 != 0 && {
        v3 = abs16(
            (ram_word!(game, samus_y_pos) as i32 - ram_word!(game, eproj_y_pos[v1]) as i32) as u16,
        );
        v3 as i32 - 20i32 & 0x8000i32 != 0
    } {
        set_ram_word!(game.eproj_instr_list_ptr[v1] = 0x8c38u16);
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
    } else if Eproj_FuncE722(k, game) != 0 {
        set_ram_word!(game.eproj_id[v1] = 0u16);
    }
}
unsafe fn EprojInit_CrocomireProjectile(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_vel[v2] = -512i32 as u16);
    set_ram_word!(game.eproj_y_vel[v2] = 1u16);
    let v3: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.eproj_x_pos[v2] = ((*v3).x_pos as i32 - 32i32) as u16);
    set_ram_word!(game.eproj_y_pos[v2] = ((*v3).y_pos as i32 - 16i32) as u16);
    set_ram_word!(game.eproj_timers[v2] = 0u16);
    set_ram_word!(game.eproj_x_subpos[v2] = 0u16);
    set_ram_word!(game.eproj_y_subpos[v2] = 0u16);
    set_ram_word!(game.eproj_gfx_idx[v2] = 2560u16);
}
pub unsafe fn EprojPreInstr_CrocomireProjectile(k: u16, game: &mut Game) {
    EprojBlockCollisition_Horiz(k, game);
    set_ram_word!(game.eproj_gfx_idx[0] = 2560u16);
    update_ram_word!(
        game.eproj_timers[(k as i32 >> 1i32)],
        |fresh142| (fresh142 as i32 + ram_word!(game, eproj_x_vel[(k as i32 >> 1i32)]) as i32)
            as u16
    );
    let x: u16 = -64i32 as u16;
    let y: u16 = crate::rtl::rom_table_word!(
        game,
        0x869059,
        ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).ai_preinstr
            as i32
            >> 1i32) as isize,
    );
    let v1: i32 = CalculateAngleFromXY(x, y, game) as i32;
    let v2: i32 = k as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_vel[v2] = (4i32 * kSinCosTable8bit_Sext[(v1 + 64i32) as usize] as i32) as u16
    );
    set_ram_word!(game.eproj_y_vel[v2] = (4i32 * kSinCosTable8bit_Sext[v1 as usize] as i32) as u16);
    set_ram_word!(game.eproj_pre_instr[v2] = (0x8690b3i32 & 0xffffi32) as u16);
}
pub unsafe fn sub_8690B3(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0
        || EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0
    {
        set_ram_word!(game.eproj_id[(k as i32 >> 1i32)] = 0u16);
    }
}
unsafe fn EprojInit_CrocomireSpikeWallPieces(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_y_pos[v1] = crate::rtl::rom_table_word!(
            game,
            0x869105,
            ((j as i32 - 20i32) as u16 as i32 >> 1i32) as isize,
        )
    );
    set_ram_word!(game.eproj_x_pos[v1] = 528u16);
    set_ram_word!(game.eproj_x_vel[v1] = 0u16);
    set_ram_word!(game.eproj_y_vel[v1] = 0u16);
    set_ram_word!(game.eproj_E[v1] = 0u16);
    set_ram_word!(game.eproj_F[v1] = 0u16);
    set_ram_word!(game.eproj_x_subpos[v1] = 0u16);
    set_ram_word!(game.eproj_y_subpos[v1] = 0u16);
    set_ram_word!(game.eproj_y_vel[v1] = -5i32 as u16);
    set_ram_word!(game.eproj_F[v1] = -30720i32 as u16);
}
static CrocomireSpikeWallPieces_Tab2: [u16; 18] = [
    0u16, 0u16, 0xff0u16, 0xee0u16, 0xcc0u16, 0xaa0u16, 0x880u16, 0x660u16, 0x440u16, 0x220u16,
    0xff0u16, 0xee0u16, 0xcc0u16, 0xaa0u16, 0x880u16, 0x660u16, 0x440u16, 0x220u16,
];
static CrocomireSpikeWallPieces_Tab1: [u16; 18] = [
    0u16, 0u16, 0xff00u16, 0xee00u16, 0xcc00u16, 0xaa00u16, 0x8800u16, 0x6600u16, 0x4400u16,
    0x2200u16, 0xff00u16, 0xee00u16, 0xcc00u16, 0xaa00u16, 0x8800u16, 0x6600u16, 0x4400u16,
    0x2200u16,
];
static CrocomireSpikeWallPieces_Tab3: [i8; 36] = [
    0i8, 0i8, 0i8, 0i8, 4i8, 0i8, 4i8, 0i8, 3i8, 0i8, 3i8, 0i8, 2i8, 0i8, 2i8, 0i8, 1i8, 0i8, 1i8,
    0i8, 6i8, 0i8, 5i8, 0i8, 4i8, 0i8, 3i8, 0i8, 2i8, 0i8, 2i8, 0i8, 1i8, 0i8, 1i8, 0i8,
];
pub unsafe fn EprojPreInstr_CrocomireSpikeWallPieces(k: u16, game: &mut Game) {
    let mut v3: i8 = 0;
    let mut v6: i8 = 0;
    let mut v7: i8 = 0;
    let mut v8: i8 = 0;
    let v1: i32 = k as i32 >> 1i32;
    let mut v2: u16 = ram_word!(game, eproj_E[v1]);
    if v2 != CrocomireSpikeWallPieces_Tab1[v1 as usize] {
        v2 = (v2 as i32 + CrocomireSpikeWallPieces_Tab2[v1 as usize] as i32) as u16;
        if v2 >= CrocomireSpikeWallPieces_Tab1[v1 as usize] {
            v2 = CrocomireSpikeWallPieces_Tab1[v1 as usize];
        }
    }
    set_ram_word!(game.eproj_E[v1] = v2);
    v3 = *((&raw mut game.g_ram as *mut u8).offset(0x1aff) as *mut u16 as *mut u8)
        .offset(k as i32 as isize)
        .offset(1) as i8;
    let mut v4: bool = u8::overflowing_add(
        *((&raw mut game.g_ram as *mut u8).offset(0x1ab7) as *mut u16 as *mut u8)
            .offset(k as i32 as isize),
        v3 as u8,
    )
    .1;
    let ref mut fresh138 = *((&raw mut game.g_ram as *mut u8).offset(0x1ab7) as *mut u16
        as *mut u8)
        .offset(k as i32 as isize);
    *fresh138 = (*fresh138 as i32 + v3 as i32) as u8;
    let mut v5: u8 = (v4 as i32
        + *((&raw mut game.g_ram as *mut u8).offset(0x1ab7) as *mut u16 as *mut u8)
            .offset(k as i32 as isize)
            .offset(1) as i32) as u8;
    if ((v5 as i32 - CrocomireSpikeWallPieces_Tab3[k as usize] as i32) as i8) >= 0i8 {
        v5 = CrocomireSpikeWallPieces_Tab3[k as usize] as u8;
    }
    *((&raw mut game.g_ram as *mut u8).offset(0x1ab7) as *mut u16 as *mut u8)
        .offset(k as i32 as isize)
        .offset(1) = v5;
    v6 = *((&raw mut game.g_ram as *mut u8).offset(0x1ab7) as *mut u16 as *mut u8)
        .offset(k as i32 as isize) as i8;
    v4 = u8::overflowing_add(
        *((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16 as *mut u8)
            .offset(k as i32 as isize)
            .offset(1),
        v6 as u8,
    )
    .1;
    let ref mut fresh139 = *((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16
        as *mut u8)
        .offset(k as i32 as isize)
        .offset(1);
    *fresh139 = (*fresh139 as i32 + v6 as i32) as u8;
    v7 = *((&raw mut game.g_ram as *mut u8).offset(0x1ab7) as *mut u16 as *mut u8)
        .offset(k as i32 as isize)
        .offset(1) as i8;
    let v9: bool = v4;
    v4 = u8::overflowing_add(v4 as u8, v7 as u8).1;
    v8 = (v9 as i32 + v7 as i32) as i8;
    v4 = (v4 as i32
        | u8::overflowing_add(
            *((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16 as *mut u8)
                .offset(k as i32 as isize),
            v8 as u8,
        )
        .1 as i32)
        != 0;
    let ref mut fresh140 = *((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16
        as *mut u8)
        .offset(k as i32 as isize);
    *fresh140 = (*fresh140 as i32 + v8 as i32) as u8;
    let ref mut fresh141 = *((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16
        as *mut u8)
        .offset(k as i32 as isize)
        .offset(1);
    *fresh141 = (*fresh141 as i32 + v4 as i32) as u8;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1adb) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1b23) as *mut u16).offset(v1 as isize)
            as *mut u16,
        0x3000u32,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16).offset(v1 as isize)
            as *mut u16,
        (ram_word!(game, eproj_y_vel[v1]) as u32) << 16i32 | ram_word!(game, eproj_F[v1]) as u32,
    );
    if ram_word!(game, eproj_y_pos[v1]) >= 0xa8u16 {
        set_ram_word!(game.eproj_id[v1] = 0u16);
        if (k as i32 & 2i32) == 0i32 {
            QueueSfx2_Max6(0x29u16, game);
        }
        let v12: i32 = k as i32 >> 1i32;
        game.eproj_spawn_pt = Point16U {
            x: ram_word!(game, eproj_x_pos[v12]),
            y: ram_word!(game, eproj_y_pos[v12]),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15u16, game);
        QueueSfx2_Max6(0x25u16, game);
    }
}
pub unsafe fn EprojInstr_9270(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v1: i32 = k as i32 >> 1i32;
    game.eproj_spawn_pt = Point16U {
        x: ram_word!(game, eproj_x_pos[v1]),
        y: ram_word!(game, eproj_y_pos[v1]),
    };
    SpawnEnemyDrops(addr_kEnemyDef_DDBF as u16, k, 0u16, game);
    epjp
}
unsafe fn EprojInit_CrocomireBridgeCrumbling(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v1] = ram_word!(game, cinematic_var8));
    set_ram_word!(game.eproj_y_pos[v1] = 187u16);
    set_ram_word!(game.eproj_y_subpos[v1] = 0u16);
    set_ram_word!(game.eproj_x_subpos[v1] = 0u16);
    set_ram_word!(game.eproj_x_vel[v1] = 0u16);
    set_ram_word!(
        game.eproj_y_vel[v1] = ((ram_word!(game, random_number) as i32 & 0x3fi32) + 64i32) as u16
    );
    set_ram_word!(game.eproj_gfx_idx[v1] = 1024u16);
}
pub unsafe fn EprojPreInstr_CrocomireBridgeCrumbling(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0 {
        set_ram_word!(game.eproj_id[(k as i32 >> 1i32)] = 0u16);
    } else {
        set_ram_word!(
            game.eproj_y_vel[(k as i32 >> 1i32)] =
                (ram_word!(game, eproj_y_vel[(k as i32 >> 1i32)]) as i32 + 24i32 & 0x3fffi32)
                    as u16
        );
    };
}
pub unsafe fn MoveEprojWithVelocity(k: u16, game: &mut Game) -> u16 {
    let v1: i32 = k as i32 >> 1i32;
    let v2: u16 = ram_word!(game, eproj_x_vel[v1]);
    let carry: i32 = *((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16 as *mut u8)
        .offset(k as i32 as isize)
        .offset(1) as i32
        + (v2 as i32 & 0xffi32);
    *((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16 as *mut u8)
        .offset(k as i32 as isize)
        .offset(1) = carry as u8;
    update_ram_word!(game.eproj_x_pos[v1], |fresh7| (fresh7 as i32
        + ((v2 as i32 >> 8i32) as i8 as i32 + (carry >> 8i32)))
        as u16);
    MoveEprojWithVelocityY(k, game)
}
pub unsafe fn MoveEprojWithVelocityY(k: u16, game: &mut Game) -> u16 {
    let v1 = usize::from(k >> 1);
    let v2: u16 = ram_word!(game, eproj_y_vel[v1]);
    let (fraction, carry) = (*((&raw mut game.g_ram as *mut u8).offset(0x1a6f))
        .offset(k as isize)
        .offset(1))
    .overflowing_add(v2 as u8);
    *((&raw mut game.g_ram as *mut u8).offset(0x1a6f))
        .offset(k as isize)
        .offset(1) = fraction;
    let result = game
        .enemy_projectile_y_pos(k)
        .wrapping_add_signed(((v2 as i16) >> 8) + carry as i16);
    set_ram_word!(game.eproj_y_pos[v1] = result);
    result
}
pub unsafe fn MoveEprojWithVelocityX(k: u16, game: &mut Game) -> u16 {
    let v1 = usize::from(k >> 1);
    let v2: u16 = ram_word!(game, eproj_x_vel[v1]);
    let (fraction, carry) = (*((&raw mut game.g_ram as *mut u8).offset(0x1a27))
        .offset(k as isize)
        .offset(1))
    .overflowing_add(v2 as u8);
    *((&raw mut game.g_ram as *mut u8).offset(0x1a27))
        .offset(k as isize)
        .offset(1) = fraction;
    let result = game
        .enemy_projectile_x_pos(k)
        .wrapping_add_signed(((v2 as i16) >> 8) + carry as i16);
    set_ram_word!(game.eproj_x_pos[v1] = result);
    result
}
pub unsafe fn SetAreaDependentEprojPropertiesEx(k: u16, j: u16, game: &mut Game) {
    let mut v2: u16 = 0;
    let p: *mut u16 = RomPtr_86(k, game) as *mut u16;
    if game.area_index() == 2 {
        v2 = *p.offset(1);
    } else if game.area_index() == 5 {
        v2 = *p.offset(2);
    } else {
        v2 = *p.offset(0);
    }
    EprojStateMut(&mut game.g_ram).set_properties(usize::from(j >> 1), v2);
}
static kEprojInit_9634_Xvel: [i16; 4] = [-0x200i16, -0x1f0i16, -0x1bci16, -0x16ai16];
static kEprojInit_9634_Yvel: [i16; 4] = [0i16, 0x88i16, 0xfci16, 0x16ai16];
unsafe fn EprojInit_9634(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_E[v1] = 0u16);
    set_ram_word!(game.eproj_F[v1] = 0u16);
    set_ram_word!(
        game.eproj_x_pos[v1] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
                as i32
                - 29i32) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v1] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).y_pos
                as i32
                - 35i32) as u16
    );
    set_ram_word!(game.eproj_gfx_idx[v1] = 2560u16);
    let v2: i32 = (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0))
        .parameter_1 as i32;
    set_ram_word!(game.eproj_x_vel[v1] = kEprojInit_9634_Xvel[v2 as usize] as u16);
    set_ram_word!(game.eproj_y_vel[v1] = kEprojInit_9634_Yvel[v2 as usize] as u16);
}
pub unsafe fn EprojPreInstr_9634(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, eproj_E[v1]) >= 8u16 {
        MoveEprojWithVelocity(k, game);
        if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0 {
            set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_off_869574 as u16);
            update_ram_word!(game.eproj_E[v1], wrapping_add(1));
            set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
            set_ram_word!(game.eproj_x_vel[v1] = 0u16);
            set_ram_word!(game.eproj_y_vel[v1] = 0u16);
            set_ram_word!(game.eproj_gfx_idx[v1] = 2560u16);
            QueueSfx2_Max6(0x2bu16, game);
        }
    } else {
        update_ram_word!(game.eproj_E[v1], wrapping_add(1));
    };
}
unsafe fn EprojInit_9642_RidleysFireball(j: u16, game: &mut Game) {
    let slot = usize::from(j >> 1);
    let parameter = EprojState(&game.g_ram).slot_word(0x1995, 0);
    EprojStateMut(&mut game.g_ram).write(0x1b23, slot, parameter);
    let v2: i16 = (if EprojState(&game.g_ram).init_param_1() != 0u16 {
        25i32
    } else {
        -25i32
    }) as i16;
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
    SetAreaDependentEprojPropertiesEx(addr_kRidleysFireball_Tab0 as u16, j, game);
}
pub unsafe fn SetAreaDependentEprojProperties(j: u16, game: &mut Game) {
    SetAreaDependentEprojPropertiesEx(addr_kRidleysFireball_Tab0 as u16, j, game);
}
fn SetEprojSpawnPoint(k: u16, game: &mut Game) {
    let x = game.enemy_projectile_x_pos(k);
    let y = game.enemy_projectile_y_pos(k);
    game.eproj_spawn_pt = Point16U { x, y };
}
pub unsafe fn EprojPreInstr_9642_RidleysFireball(k: u16, game: &mut Game) {
    let mut v1: u16 = 0;
    if EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0 {
        v1 = -27042i32 as u16;
    } else {
        if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 == 0 {
            return;
        }
        v1 = -27056i32 as u16;
    }
    let v2: i32 = k as i32 >> 1i32;
    EprojStateMut(&mut game.g_ram).set_id(v2 as usize, 0);
    if EprojState(&game.g_ram).slot_word(0x1b23, v2 as usize) == 0 {
        SetEprojSpawnPoint(k, game);
        SpawnEprojWithRoomGfx(v1, 3u16, game);
        QueueSfx2_Max6(0x2bu16, game);
    }
}
pub unsafe fn EprojInstr_DisableCollisionsWithSamus(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    update_ram_word!(
        game.eproj_properties[(k as i32 >> 1i32)],
        |fresh32| (fresh32 as i32 | 0x2000i32) as u16
    );
    epjp
}
fn EprojInit_9660_FireballExplosion(j: u16, game: &mut Game) {
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
unsafe fn EprojInit_Common(j: u16, k: u16, a: u16, game: &mut Game) {
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
unsafe fn EprojInit_FireballChild(j: u16, x: u16, y: u16, handler: u16, game: &mut Game) {
    let parent = EprojState(&game.g_ram).init_param_1();
    let slot = usize::from(j >> 1);
    EprojStateMut(&mut game.g_ram).write(0x1ab7, slot, x);
    EprojStateMut(&mut game.g_ram).write(0x1adb, slot, y);
    EprojInit_Common(j, parent, handler, game);
}
unsafe fn EprojInit_9688(j: u16, game: &mut Game) {
    EprojInit_FireballChild(j, 0, (-3584i16) as u16, 0x9688, game);
}
unsafe fn EprojInit_9696(j: u16, game: &mut Game) {
    EprojInit_FireballChild(j, 0, 3584, 0x9696, game);
}
unsafe fn EprojInit_966C(j: u16, game: &mut Game) {
    EprojInit_FireballChild(j, 3584, 0, 0x966c, game);
}
unsafe fn EprojInit_967A(j: u16, game: &mut Game) {
    EprojInit_FireballChild(j, (-3584i16) as u16, 0, 0x967a, game);
}
pub unsafe fn EprojPreInstr_966C(k: u16, game: &mut Game) {
    MoveEprojWithVelocityX(k, game);
    if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0 {
        let slot = usize::from(k >> 1);
        EprojStateMut(&mut game.g_ram).set_instruction(slot, addr_off_869574 as u16);
        EprojStateMut(&mut game.g_ram).set_instruction_timer(slot, 1);
    }
}
pub unsafe fn EprojPreInstr_9688(k: u16, game: &mut Game) {
    MoveEprojWithVelocityY(k, game);
    if EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0 {
        let slot = usize::from(k >> 1);
        EprojStateMut(&mut game.g_ram).set_instruction(slot, addr_off_869574 as u16);
        EprojStateMut(&mut game.g_ram).set_instruction_timer(slot, 1);
    }
}
pub unsafe fn EprojPreInstr_96A4(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0 {
        set_ram_word!(game.eproj_id[(k as i32 >> 1i32)] = 0u16);
    }
}
pub unsafe fn EprojPreInstr_96C0(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0 {
        set_ram_word!(game.eproj_id[(k as i32 >> 1i32)] = 0u16);
    }
}
pub unsafe fn EprojPreInstr_96CE(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0 {
        set_ram_word!(game.eproj_id[(k as i32 >> 1i32)] = 0u16);
    }
}
pub unsafe fn EprojInstr_95BA(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    EprojStateMut(&mut game.g_ram).write(0x1995, 0, 0);
    SpawnEprojWithRoomGfx(addr_stru_86966C as u16, k, game);
    EprojStateMut(&mut game.g_ram).write(0x1995, 0, 0);
    SpawnEprojWithRoomGfx(addr_stru_86967A as u16, k, game);
    epjp
}
pub unsafe fn EprojInstr_95ED(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    EprojStateMut(&mut game.g_ram).write(0x1995, 0, 0);
    SpawnEprojWithRoomGfx(addr_stru_869688 as u16, k, game);
    EprojStateMut(&mut game.g_ram).write(0x1995, 0, 0);
    SpawnEprojWithRoomGfx(addr_stru_869696 as u16, k, game);
    epjp
}
pub unsafe fn EprojInstr_9620(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let count = EprojStateMut(&mut game.g_ram).decrement_byte(0x1aff, usize::from(k));
    if count as i8 >= 0 {
        let handler = EprojState(&game.g_ram).slot_word(0x1b23, usize::from(k >> 1));
        SpawnEprojWithRoomGfx(handler, k, game);
    }
    epjp
}
fn EprojInit_9734_CeresFallingDebris(j: u16, game: &mut Game) {
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
pub unsafe fn EprojPreInstr_9734_CeresFallingDebris(k: u16, game: &mut Game) {
    let slot = usize::from(k >> 1);
    EprojStateMut(&mut game.g_ram).add_word(0x1adb, slot, 16);
    if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0 {
        EprojStateMut(&mut game.g_ram).set_id(slot, 0);
        SetEprojSpawnPoint(k, game);
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9u16, game);
        QueueSfx2_Max6(0x6du16, game);
    }
}
pub unsafe fn EprojInstr_980E(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v2: i32 = k as i32 >> 1i32;
    game.eproj_spawn_pt = Point16U {
        x: ram_word!(game, eproj_x_pos[v2]),
        y: ram_word!(game, eproj_y_pos[v2]),
    };
    SpawnEnemyDrops(addr_kEnemyDef_E4FF as u16, k, 0u16, game);
    epjp
}
static byte_8698B4: [u8; 16] = [
    0x10u8, 0x20u8, 0x30u8, 0x40u8, 0x50u8, 0x60u8, 0x70u8, 0x80u8, 0xf0u8, 0xe0u8, 0xd0u8, 0xc0u8,
    0xb0u8, 0xa0u8, 0x90u8, 0x80u8,
];
static byte_8698F7: [u8; 9] = [
    0x30u8, 0x44u8, 0x58u8, 0x6cu8, 0x80u8, 0x94u8, 0xa8u8, 0xbcu8, 0xd0u8,
];
static byte_869979: [u8; 8] = [0u8, 0x20u8, 0x40u8, 0x60u8, 0x80u8, 0xa0u8, 0xc0u8, 0xe0u8];
unsafe fn EprojInit_PhantoonDestroyableFireballs(j: u16, game: &mut Game) {
    let mut v1: i8 = 0;
    v1 = ((ram_word!(game, cinematic_var8) as i32 & 0xff00i32) as u16 as i32 >> 8i32) as i8;
    match v1 as i32 {
        0 => {
            let v2: i32 = j as i32 >> 1i32;
            set_ram_word!(game.eproj_x_subpos[v2] = 0u16);
            set_ram_word!(game.eproj_y_subpos[v2] = 0u16);
            set_ram_word!(game.eproj_x_vel[v2] = 0u16);
            set_ram_word!(game.eproj_y_vel[v2] = 0u16);
            set_ram_word!(
                game.eproj_x_pos[v2] =
                    (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0))
                        .x_pos
            );
            set_ram_word!(
                game.eproj_y_pos[v2] = ((*((&raw mut game.g_ram as *mut u8).offset(0xf78)
                    as *mut EnemyData)
                    .offset(0))
                .y_pos as i32
                    + 32i32) as u16
            );
            set_ram_word!(game.eproj_instr_list_ptr[v2] = addr_off_8697B4 as u16);
            set_ram_word!(
                game.eproj_properties[v2] =
                    (ram_word!(game, eproj_properties[v2]) as i32 & 0xfffi32 | 0x2000i32) as u16
            );
        }
        2 => {
            let v3: i32 = j as i32 >> 1i32;
            set_ram_word!(game.eproj_x_subpos[v3] = 0u16);
            set_ram_word!(game.eproj_y_subpos[v3] = 0u16);
            set_ram_word!(game.eproj_y_vel[v3] = 0u16);
            let v4: u16 = ram_word!(game, cinematic_var8) as u8 as u16;
            if ((ram_word!(game, cinematic_var8) as u8 as i32 - 8i32) as i16) >= 0i16 {
                set_ram_word!(game.eproj_x_vel[v3] = -2i32 as u16);
            } else {
                set_ram_word!(game.eproj_x_vel[v3] = 2u16);
            }
            set_ram_word!(game.eproj_E[v3] = byte_8698B4[v4 as usize] as u16);
            set_ram_word!(
                game.eproj_x_pos[v3] =
                    (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0))
                        .x_pos
            );
            set_ram_word!(
                game.eproj_y_pos[v3] = ((*((&raw mut game.g_ram as *mut u8).offset(0xf78)
                    as *mut EnemyData)
                    .offset(0))
                .y_pos as i32
                    + 32i32) as u16
            );
            set_ram_word!(game.eproj_pre_instr[v3] = (0x869a45i32 & 0xffffi32) as u16);
        }
        4 => {
            let v5: i32 = j as i32 >> 1i32;
            set_ram_word!(game.eproj_x_subpos[v5] = 0u16);
            set_ram_word!(game.eproj_y_subpos[v5] = 0u16);
            set_ram_word!(game.eproj_y_vel[v5] = 0u16);
            set_ram_word!(
                game.eproj_x_vel[v5] =
                    ((ram_word!(game, cinematic_var8) as i32 & 0xf0i32) >> 1i32) as u16
            );
            set_ram_word!(
                game.eproj_x_pos[v5] =
                    byte_8698F7[(ram_word!(game, cinematic_var8) as i32 & 0xfi32) as usize] as u16
            );
            set_ram_word!(game.eproj_y_pos[v5] = 40u16);
            set_ram_word!(game.eproj_pre_instr[v5] = (0x869a94i32 & 0xffffi32) as u16);
        }
        6 => {
            let v7: i32 = j as i32 >> 1i32;
            set_ram_word!(game.eproj_x_subpos[v7] = 0u16);
            set_ram_word!(game.eproj_y_subpos[v7] = 0u16);
            set_ram_word!(game.eproj_y_vel[v7] = 0u16);
            set_ram_word!(game.eproj_x_vel[v7] = 128u16);
            set_ram_word!(
                game.eproj_E[v7] =
                    byte_869979[ram_word!(game, cinematic_var8) as u8 as usize] as u16
            );
            set_ram_word!(
                game.eproj_x_pos[v7] =
                    (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0))
                        .x_pos
            );
            set_ram_word!(
                game.eproj_y_pos[v7] = ((*((&raw mut game.g_ram as *mut u8).offset(0xf78)
                    as *mut EnemyData)
                    .offset(0))
                .y_pos as i32
                    + 16i32) as u16
            );
            set_ram_word!(game.eproj_pre_instr[v7] = (0x869adai32 & 0xffffi32) as u16);
        }
        _ => {
            Unreachable(game);
        }
    };
}
unsafe fn EprojInit_PhantoonStartingFireballs(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_subpos[v1] = 0u16);
    set_ram_word!(game.eproj_y_subpos[v1] = 0u16);
    set_ram_word!(game.eproj_x_vel[v1] = 0u16);
    set_ram_word!(game.eproj_y_vel[v1] = 0u16);
    let v2: u16 = byte_869979[ram_word!(game, cinematic_var8) as usize] as u16;
    set_ram_word!(game.eproj_E[v1] = v2);
    let pt: Point16U = Eproj_PhantomFireballs_Func1(v2, 0x30u16);
    set_ram_word!(
        game.eproj_x_pos[v1] = (pt.x as i32
            + (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
                as i32) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v1] = (pt.y as i32
            + (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).y_pos
                as i32
            + 16i32) as u16
    );
}
pub unsafe fn EprojPreInstr_PhantoonStartingFireballs(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    update_ram_word!(game.eproj_y_vel[v1], |fresh133| (fresh133 as i32 + 16i32)
        as u16);
    if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0 {
        set_ram_word!(
            game.eproj_properties[v1] =
                (ram_word!(game, eproj_properties[v1]) as i32 & 0xfffi32 | 0x8000i32) as u16
        );
        set_ram_word!(game.eproj_pre_instr[v1] = (0x8699bfi32 & 0xffffi32) as u16);
        set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_word_86976C as u16);
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
        set_ram_word!(game.eproj_E[v1] = 8u16);
        update_ram_word!(game.eproj_y_pos[v1], |fresh134| (fresh134 as i32 + 8i32)
            as u16);
    }
}
pub unsafe fn EprojPreInstr_PhantoonStartingFireballs2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let v2: bool = ram_word!(game, eproj_E[v1]) == 1u16;
    let v3: bool =
        (update_ram_word!(game.eproj_E[v1], wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32;
    if v2 || v3 {
        set_ram_word!(game.eproj_pre_instr[v1] = (0x869a01i32 & 0xffffi32) as u16);
        set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_word_869772 as u16);
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
        update_ram_word!(game.eproj_y_pos[v1], |fresh132| (fresh132 as i32 - 8i32)
            as u16);
        set_ram_word!(game.eproj_y_vel[v1] = -768i32 as u16);
        set_ram_word!(game.eproj_E[v1] = 0u16);
        if (ram_word!(game, nmi_frame_counter_word) as i32 & 1i32) != 0i32 {
            set_ram_word!(game.eproj_x_vel[v1] = -128i32 as u16);
        } else {
            set_ram_word!(game.eproj_x_vel[v1] = 128u16);
        }
    }
}
static word_869A3E: [u16; 3] = [0xfd00u16, 0xfe00u16, 0xff00u16];
pub unsafe fn EprojPreInstr_PhantoonStartingFireballs3(k: u16, game: &mut Game) {
    let current_block: u64;
    let v1: i32 = k as i32 >> 1i32;
    update_ram_word!(game.eproj_y_vel[v1], |fresh129| (fresh129 as i32 + 16i32)
        as u16);
    if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0 {
        let v2: u16 = (ram_word!(game, eproj_E[v1]) as i32 + 1i32) as u16;
        set_ram_word!(game.eproj_E[v1] = v2);
        if v2 as i32 - 3i32 & 0x8000i32 != 0 {
            set_ram_word!(game.eproj_y_vel[v1] = word_869A3E[v2 as usize]);
            return;
        }
        current_block = 220575731145207134;
    } else if EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0 {
        current_block = 220575731145207134;
    } else {
        current_block = 13183875560443969876;
    }
    if current_block == 220575731145207134 {
        set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_word_869782 as u16);
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
        set_ram_word!(game.eproj_pre_instr[v1] = (0x869a44i32 & 0xffffi32) as u16);
    };
}
pub unsafe fn EprojPreInstr_PhantoonDestroyableFireballs(k: u16, game: &mut Game) {
    let mut v4: i16 = 0;
    let mut v5: u16 = 0;
    let v1: i32 = k as i32 >> 1i32;
    update_ram_word!(game.eproj_y_vel[v1], |fresh128| (fresh128 as i32 + 4i32)
        as u16);
    let v2: u16 = (*(((&raw mut game.g_ram as *mut u8).offset(0x1ab7) as *mut u16)
        .offset(v1 as isize) as *mut u16 as *mut u8)
        .offset(0) as i32
        + ram_word!(game, eproj_E[v1]) as i32) as u8 as u16;
    set_ram_word!(game.eproj_E[v1] = v2);
    let pt: Point16U = Eproj_PhantomFireballs_Func1(v2, ram_word!(game, eproj_y_vel[v1]));
    let mut v3: bool = ((pt.x as i32
        + (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
            as i32) as i16)
        < 0i16;
    v4 = (pt.x as i32
        + (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
            as i32) as i16;
    set_ram_word!(
        game.eproj_x_pos[v1] = (pt.x as i32
            + (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
                as i32) as u16
    );
    if v3
        || v4 as i32 - 256i32 & 0x8000i32 == 0
        || {
            v3 = ((pt.y as i32
                + (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0))
                    .y_pos as i32
                + 16i32) as i16)
                < 0i16;
            v5 = (pt.y as i32
                + (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0))
                    .y_pos as i32
                + 16i32) as u16;
            set_ram_word!(game.eproj_y_pos[v1] = v5);
            v3 as i32 != 0
        }
        || v5 as i32 - 256i32 & 0x8000i32 == 0
    {
        set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_off_8697F8 as u16);
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
    }
}
pub unsafe fn EprojPreInstr_PhantoonDestroyableFireballs_2(k: u16, game: &mut Game) {
    let mut v2: bool = false;
    let mut v3: bool = false;
    let current_block: u64;
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, eproj_x_vel[v1]) == 0 {
        current_block = 8066597768905169940;
    } else {
        v2 = false;
        v3 = false;
        v2 = ram_word!(game, eproj_x_vel[v1]) == 1u16;
        v3 = (update_ram_word!(game.eproj_x_vel[v1], wrapping_sub(1), reread) as i32 & 0x8000i32)
            != 0i32;
        if v2 || v3 {
            QueueSfx3_Max6(0x1du16, game);
            current_block = 8066597768905169940;
        } else {
            current_block = 2868539653012386629;
        }
    }
    if current_block == 8066597768905169940 {
        update_ram_word!(game.eproj_y_vel[v1], |fresh126| (fresh126 as i32 + 16i32)
            as u16);
        if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0 {
            set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_word_8697AC as u16);
            set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
            update_ram_word!(game.eproj_y_pos[v1], |fresh127| (fresh127 as i32 + 8i32)
                as u16);
            set_ram_word!(game.eproj_pre_instr[v1] = (0x869a44i32 & 0xffffi32) as u16);
            QueueSfx3_Max6(0x1du16, game);
        }
    };
}
pub unsafe fn EprojPreInstr_PhantoonDestroyableFireballs_3(k: u16, game: &mut Game) {
    let mut v4: i16 = 0;
    let mut v5: u16 = 0;
    let v1: i32 = k as i32 >> 1i32;
    update_ram_word!(game.eproj_y_vel[v1], |fresh123| (fresh123 as i32 + 2i32)
        as u16);
    let v2: u16 = (ram_word!(game, eproj_E[v1]) as i32 + 2i32) as u8 as u16;
    set_ram_word!(game.eproj_E[v1] = v2);
    let pt: Point16U = Eproj_PhantomFireballs_Func1(v2, ram_word!(game, eproj_y_vel[v1]));
    let mut v3: bool = ((pt.x as i32
        + (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
            as i32) as i16)
        < 0i16;
    v4 = (pt.x as i32
        + (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
            as i32) as i16;
    set_ram_word!(
        game.eproj_x_pos[v1] = (pt.x as i32
            + (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
                as i32) as u16
    );
    if v3
        || v4 as i32 - 256i32 & 0x8000i32 == 0
        || {
            v3 = ((pt.y as i32
                + (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0))
                    .y_pos as i32
                + 16i32) as i16)
                < 0i16;
            v5 = (pt.y as i32
                + (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0))
                    .y_pos as i32
                + 16i32) as u16;
            set_ram_word!(game.eproj_y_pos[v1] = v5);
            v3 as i32 != 0
        }
        || v5 as i32 - 256i32 & 0x8000i32 == 0
    {
        set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_off_8697F8 as u16);
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
    }
}
pub unsafe fn EprojPreInstr_PhantoonStartingFireballsB(k: u16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).ai_var_B != 0
    {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.eproj_pre_instr[v1] = (0x869b41i32 & 0xffffi32) as u16);
        set_ram_word!(game.eproj_x_vel[v1] = 180u16);
        set_ram_word!(game.eproj_y_vel[v1] = 48u16);
    }
}
pub unsafe fn EprojPreInstr_PhantoonStartingFireballsB_2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let v2: u16 = ram_word!(game, eproj_x_vel[v1]);
    if v2 != 0 {
        set_ram_word!(game.eproj_x_vel[v1] = (v2 as i32 - 1i32) as u16);
    } else if ((ram_word!(game, nmi_frame_counter_word) as i32 & 1i32) != 0i32) {
        let v3: u16 = (ram_word!(game, eproj_y_vel[v1]) as i32 - 1i32) as u16;
        set_ram_word!(game.eproj_y_vel[v1] = v3);
        if (v3 == 0) {
            set_ram_word!(
                game.eproj_x_pos[v1] =
                    (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0))
                        .x_pos
            );
            set_ram_word!(
                game.eproj_y_pos[v1] = ((*((&raw mut game.g_ram as *mut u8).offset(0xf78)
                    as *mut EnemyData)
                    .offset(0))
                .y_pos as i32
                    + 16i32) as u16
            );
            set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
            set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_off_8697F8 as u16);
            return;
        }
    }
    let v4: u8 = (ram_word!(game, eproj_E[v1]) as i32 + 1i32) as u8;
    set_ram_word!(game.eproj_E[v1] = v4 as u16);
    let pt: Point16U = Eproj_PhantomFireballs_Func1(v4 as u16, ram_word!(game, eproj_y_vel[v1]));
    set_ram_word!(
        game.eproj_x_pos[v1] = (pt.x as i32
            + (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
                as i32) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v1] = (pt.y as i32
            + (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).y_pos
                as i32
            + 16i32) as u16
    );
}
pub unsafe fn Eproj_PhantomFireballs_Func1(j: u16, a: u16) -> Point16U {
    let mut v3: i16 = 0;
    let mut v2: u16 = 0;
    let mut v4: u16 = 0;
    let r24: u16 = a;
    let r26: u16 = j;
    if j as i32 - 128i32 & 0x8000i32 != 0 {
        v2 = Eproj_PhantomFireballs_Func2((2i32 * j as i32) as u16, r24);
    } else {
        v2 = -(Eproj_PhantomFireballs_Func2((2i32 * (j as i32 + 0x80i32) as u8 as i32) as u16, r24)
            as i32) as u16;
    }
    let r20: u16 = v2;
    v3 = (r26 as i32 - 64i32) as u8 as i16;
    if v3 as i32 - 128i32 & 0x8000i32 != 0 {
        v4 = Eproj_PhantomFireballs_Func2((2i32 * v3 as i32) as u16, r24);
    } else {
        v4 = -(Eproj_PhantomFireballs_Func2((2i32 * (v3 as i32 + 0x80i32) as u8 as i32) as u16, r24)
            as i32) as u16;
    }
    let r22: u16 = v4;
    Point16U { x: r20, y: r22 }
}
pub unsafe fn Eproj_PhantomFireballs_Func2(k: u16, r24: u16) -> u16 {
    let r18: u16 = (Mult8x8(
        *((&raw const kSinCosTable8bit_Sext as *const i16).offset(64) as *const i16 as *mut u8)
            .offset(k as i32 as isize),
        r24 as u8,
    ) as i32
        >> 8i32) as u16;
    (r18 as i32
        + Mult8x8(
            *((&raw const kSinCosTable8bit_Sext as *const i16).offset(64) as *const i16 as *mut u8)
                .offset(k as i32 as isize)
                .offset(1),
            r24 as u8,
        ) as i32) as u16
}
unsafe fn EprojInit_RocksKraidSpits(j: u16, game: &mut Game) {
    let enemy_index = ram_word!(game, cur_enemy_index);
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v2] = (game.enemy_state(enemy_index).x_pos()).wrapping_add(16u16)
    );
    set_ram_word!(
        game.eproj_y_pos[v2] = (game.enemy_state(enemy_index).y_pos()).wrapping_sub(96u16)
    );
    set_ram_word!(game.eproj_y_subpos[v2] = 0u16);
    set_ram_word!(game.eproj_x_subpos[v2] = 0u16);
    set_ram_word!(game.eproj_x_vel[v2] = ram_word!(game, cinematic_var8));
    set_ram_word!(game.eproj_y_vel[v2] = -1024i32 as u16);
    set_ram_word!(game.eproj_gfx_idx[v2] = 1536u16);
}
unsafe fn EprojInit_RocksFallingKraidCeiling(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v1] = ram_word!(game, cinematic_var8));
    set_ram_word!(game.eproj_y_pos[v1] = 312u16);
    set_ram_word!(game.eproj_y_subpos[v1] = 0u16);
    set_ram_word!(game.eproj_x_subpos[v1] = 0u16);
    set_ram_word!(game.eproj_x_vel[v1] = 0u16);
    set_ram_word!(
        game.eproj_y_vel[v1] =
            ((ram_word!(game, random_number) as i32 & 0x3fi32) as u16).wrapping_add(64u16)
    );
    set_ram_word!(game.eproj_gfx_idx[v1] = 1536u16);
}
unsafe fn EprojInit_RocksWhenKraidRises(j: u16, game: &mut Game) {
    let mut v1: i16 = 0;
    v1 = ((ram_word!(game, random_number)) as i16) & (0x3fi16);
    if (ram_word!(game, random_number) as i32 & 1i32) == 0i32 {
        v1 = !(v1 as i32) as i16;
    }
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v2] = ({
            let enemy_index = ram_word!(game, cur_enemy_index);
            game.enemy_state(enemy_index).x_pos()
        })
        .wrapping_add((v1) as u16)
    );
    set_ram_word!(game.eproj_y_pos[v2] = 432u16);
    set_ram_word!(game.eproj_y_subpos[v2] = 0u16);
    set_ram_word!(game.eproj_x_subpos[v2] = 0u16);
    set_ram_word!(game.eproj_x_vel[v2] = ram_word!(game, cinematic_var8));
    set_ram_word!(game.eproj_y_vel[v2] = -1280i32 as u16);
    set_ram_word!(game.eproj_gfx_idx[v2] = 1536u16);
    QueueSfx3_Max6(0x1eu16, game);
}
pub unsafe fn EprojPreInstr_KraidRocks(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0
        || EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0
    {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    } else {
        let v1: i32 = k as i32 >> 1i32;
        let mut v2: u16 = EprojStateMut(&mut game.g_ram).add_word(0x1ab7, v1 as usize, 8);
        if v2 as i32 - 256i32 & 0x8000i32 == 0 {
            v2 = -256i32 as u16;
        }
        set_ram_word!(game.eproj_x_vel[v1] = v2);
        EprojStateMut(&mut game.g_ram).add_word(0x1adb, v1 as usize, 64);
    };
}
pub unsafe fn EprojPreInstr_RocksFallingKraidCeiling(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0 {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    } else {
        set_ram_word!(
            game.eproj_y_vel[(k as i32 >> 1i32)] =
                ((ram_word!(game, eproj_y_vel[(k as i32 >> 1i32)]) as i32 + 24i32) as u16)
                    & (0x3fffu16)
        );
    };
}
pub unsafe fn sub_869DA5(k: u16, game: &mut Game) {
    set_ram_word!(game.eproj_gfx_idx[(k as i32 >> 1i32)] = 0u16);
}
unsafe fn EprojInit_MiniKraidSpit(j: u16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let extra_index = ram_word!(game, cur_enemy_index);
    let v3: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v3] = (game.mini_kraid_spit_state(extra_index).x_offset() as i32
            + game.enemy_state(enemy_index).x_pos() as i32) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v3] = (game.enemy_state(enemy_index).y_pos() as i32 - 16i32) as u16
    );
    set_ram_word!(game.eproj_y_subpos[v3] = 0u16);
    set_ram_word!(game.eproj_x_subpos[v3] = 0u16);
    set_ram_word!(game.eproj_x_vel[v3] = game.mini_kraid_spit_state(extra_index).x_velocity());
    set_ram_word!(game.eproj_y_vel[v3] = game.mini_kraid_spit_state(extra_index).y_velocity());
}
pub unsafe fn EprojPreInit_MiniKraidSpit(k: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    if EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0
        || EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0
    {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    } else {
        let v1: i32 = k as i32 >> 1i32;
        v2 = ((ram_word!(game, eproj_y_vel[v1])) as i16).wrapping_add(64i16);
        if v2 >= 0 && ram_word!(game, eproj_y_vel[v1]) as i32 - 960i32 & 0x8000i32 == 0 {
            v2 = 1024i16;
        }
        set_ram_word!(game.eproj_y_vel[v1] = v2 as u16);
    };
}
static kEprojInit_MiniKraidSpikes_Tab0: [i16; 3] = [-2i16, 12i16, 24i16];
unsafe fn EprojInit_MiniKraidSpikes(j: u16, a: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_vel[v2] = a);
    let r18: u16 =
        kEprojInit_MiniKraidSpikes_Tab0[((*gExtraEnemyRam7800(game.cur_enemy_index() as i32, game))
            .kraid
            .kraid_healths_8ths[0i32 as usize] as i32
            >> 1i32) as usize] as u16;
    let enemy_index = game.cur_enemy_index();
    set_ram_word!(game.eproj_x_pos[v2] = game.enemy_state(enemy_index).x_pos());
    set_ram_word!(game.eproj_y_pos[v2] = (r18).wrapping_add(game.enemy_state(enemy_index).y_pos()));
    set_ram_word!(game.eproj_y_subpos[v2] = 0u16);
    set_ram_word!(game.eproj_x_subpos[v2] = 0u16);
    set_ram_word!(game.eproj_y_vel[v2] = 0u16);
}
unsafe fn EprojInit_MiniKraidSpikesLeft(j: u16, game: &mut Game) {
    EprojInit_MiniKraidSpikes(j, -0x200i32 as u16, game);
}
unsafe fn EprojInit_MiniKraidSpikesRight(j: u16, game: &mut Game) {
    EprojInit_MiniKraidSpikes(j, 0x200u16, game);
}
pub unsafe fn EprojPreInstr_MiniKraidSpikes(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0 {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    }
}
unsafe fn EprojInit_WalkingLavaSeahorseFireball(j: u16, game: &mut Game) {
    static word_869EF9: [i16; 3] = [-0x100i16, 0i16, 0x100i16];
    let v1: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_y_pos[v2] = ((*v1).y_pos as i32 - 12i32) as u16);
    set_ram_word!(game.eproj_x_vel[v2] = -1024i32 as u16);
    set_ram_word!(game.eproj_x_pos[v2] = ((*v1).x_pos as i32 - 16i32) as u16);
    if ((*v1).ai_var_D as i32 & 0x8000i32) == 0i32 {
        set_ram_word!(game.eproj_x_vel[v2] = 1024u16);
        set_ram_word!(game.eproj_x_pos[v2] = ((*v1).x_pos as i32 + 16i32) as u16);
    }
    set_ram_word!(
        game.eproj_y_vel[v2] =
            word_869EF9[(ram_word!(game, cinematic_var8) as i32 >> 1i32) as usize] as u16
    );
    set_ram_word!(game.eproj_y_subpos[v2] = 0u16);
    set_ram_word!(game.eproj_x_subpos[v2] = 0u16);
}
pub unsafe fn EprojPreInstr_WalkingLavaSeahorseFireball(k: u16, game: &mut Game) {
    let mut v2: u16 = 0;
    if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0
        || EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0
    {
        set_ram_word!(game.eproj_id[(k as i32 >> 1i32)] = 0u16);
    } else {
        let v1: i32 = k as i32 >> 1i32;
        if (ram_word!(game, eproj_x_vel[v1]) as i32 & 0x8000i32) == 0i32 {
            v2 = (ram_word!(game, eproj_x_vel[v1]) as i32 - 64i32) as u16;
            set_ram_word!(game.eproj_x_vel[v1] = v2);
            if v2 as i32 - 512i32 & 0x8000i32 != 0 {
                v2 = 512u16;
            }
        } else {
            v2 = (ram_word!(game, eproj_x_vel[v1]) as i32 + 64i32) as u16;
            set_ram_word!(game.eproj_x_vel[v1] = v2);
            if v2 as i32 + 512i32 & 0x8000i32 == 0 {
                v2 = -512i32 as u16;
            }
        }
        set_ram_word!(game.eproj_x_vel[v1] = v2);
    };
}
unsafe fn EprojInit_PirateMotherBrainLaser(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_instr_list_ptr[v2] = (if game.eproj_spawn_r22 != 0u16 {
            addr_word_869F7D as i32
        } else {
            addr_word_869F41 as i32
        }) as u16
    );
    set_ram_word!(game.eproj_pre_instr[v2] = (0x86a05bi32 & 0xffffi32) as u16);
    set_ram_word!(game.eproj_x_pos[v2] = game.eproj_spawn_pt.x);
    set_ram_word!(game.eproj_y_pos[v2] = game.eproj_spawn_pt.y);
    let enemy_index = game.cur_enemy_index();
    set_ram_word!(
        game.eproj_properties[v2] =
            (*(RomPtr_A0(game.enemy_state(enemy_index).enemy_ptr() as u16, game) as *mut u16)
                .offset(3) as i32
                | 0x1000i32) as u16
    );
    set_ram_word!(game.eproj_E[v2] = game.enemy_state(enemy_index).parameter_1());
    QueueSfx2_Max6(0x67u16, game);
}
pub unsafe fn EprojInstr_SetPreInstrAndRun(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.eproj_pre_instr[(k as i32 >> 1i32)] = *(epjp as *mut u16));
    epjp
}
pub unsafe fn EprojPreInstr_PirateMotherBrainLaser_MoveLeft(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    EprojStateMut(&mut game.g_ram).add_word(0x1a4b, v1 as usize, (-2i16) as u16);
    if (ram_word!(game, eproj_E[v1]) as i32 & 0x8000i32) == 0i32 {
        update_ram_word!(game.eproj_x_pos[v1], |fresh22| (fresh22 as i32 - 2i32)
            as u16);
    }
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        EprojStateMut(&mut game.g_ram).set_id(v1 as usize, 0);
    }
}
pub unsafe fn EprojPreInstr_PirateMotherBrainLaser_MoveRight(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    EprojStateMut(&mut game.g_ram).add_word(0x1a4b, v1 as usize, 2);
    if (ram_word!(game, eproj_E[v1]) as i32 & 0x8000i32) == 0i32 {
        update_ram_word!(game.eproj_x_pos[v1], |fresh24| (fresh24 as i32 + 2i32)
            as u16);
    }
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        EprojStateMut(&mut game.g_ram).set_id(v1 as usize, 0);
    }
}
unsafe fn EprojInit_PirateClaw(j: u16, game: &mut Game) {
    let r: Rect16U = game.eproj_spawn_rect;
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_y_pos[v1] = (r.y as i32 + r.h as i32) as u16);
    set_ram_word!(game.eproj_x_pos[v1] = (r.x as i32 + r.w as i32) as u16);
    set_ram_word!(
        game.eproj_instr_list_ptr[v1] = (if ram_word!(game, cinematic_var8) != 0u16 {
            addr_off_869FE1 as i32
        } else {
            addr_off_869FB9 as i32
        }) as u16
    );
    set_ram_word!(game.eproj_pre_instr[v1] = (0x86a05bi32 & 0xffffi32) as u16);
    set_ram_word!(game.eproj_E[v1] = 2048u16);
    set_ram_word!(game.eproj_F[v1] = 1u16);
}
pub unsafe fn EprojPreInstr_PirateClawThrownLeft(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, eproj_F[v1]) != 0 {
        update_ram_word!(game.eproj_x_pos[v1], |fresh118| (fresh118 as i32
            - *(((&raw mut game.g_ram as *mut u8).offset(0x1aff) as *mut u16).offset(v1 as isize)
                as *mut u16 as *mut u8)
                .offset(1isize,) as i32)
            as u16);
        let v2: u16 = (ram_word!(game, eproj_E[v1]) as i32 - 32i32) as u16;
        set_ram_word!(game.eproj_E[v1] = v2);
        if v2 == 0 {
            set_ram_word!(game.eproj_F[v1] = 0u16);
        }
    } else {
        update_ram_word!(game.eproj_x_pos[v1], |fresh119| (fresh119 as i32
            + *(((&raw mut game.g_ram as *mut u8).offset(0x1aff) as *mut u16).offset(v1 as isize)
                as *mut u16 as *mut u8)
                .offset(1isize,) as i32)
            as u16);
        update_ram_word!(game.eproj_E[v1], |fresh120| (fresh120 as i32 + 32i32)
            as u16);
    }
    update_ram_word!(game.eproj_y_pos[v1], wrapping_add(1));
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        set_ram_word!(game.eproj_id[v1] = 0u16);
    }
}
pub unsafe fn EprojPreInstr_PirateClawThrownRight(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, eproj_F[v1]) != 0 {
        update_ram_word!(game.eproj_x_pos[v1], |fresh114| (fresh114 as i32
            + *(((&raw mut game.g_ram as *mut u8).offset(0x1aff) as *mut u16).offset(v1 as isize)
                as *mut u16 as *mut u8)
                .offset(1isize,) as i32)
            as u16);
        let v2: u16 = (ram_word!(game, eproj_E[v1]) as i32 - 32i32) as u16;
        set_ram_word!(game.eproj_E[v1] = v2);
        if v2 == 0 {
            set_ram_word!(game.eproj_F[v1] = 0u16);
        }
    } else {
        update_ram_word!(game.eproj_x_pos[v1], |fresh115| (fresh115 as i32
            - *(((&raw mut game.g_ram as *mut u8).offset(0x1aff) as *mut u16).offset(v1 as isize)
                as *mut u16 as *mut u8)
                .offset(1isize,) as i32)
            as u16);
        update_ram_word!(game.eproj_E[v1], |fresh116| (fresh116 as i32 + 32i32)
            as u16);
    }
    update_ram_word!(game.eproj_y_pos[v1], wrapping_add(1));
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        set_ram_word!(game.eproj_id[v1] = 0u16);
    }
}
static word_86A2D6: [i16; 6] = [64i16, 72i16, 80i16, -64i16, -72i16, -80i16];
unsafe fn EprojInit_A379(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_subpos[v1] = 0u16);
    set_ram_word!(game.eproj_y_subpos[v1] = 0u16);
    set_ram_word!(game.eproj_x_vel[v1] = 0u16);
    set_ram_word!(game.eproj_y_vel[v1] = 0u16);
    let v2: u16 = ram_word!(game, cinematic_var8);
    let v3: i32 = ram_word!(game, cinematic_var8) as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v1] =
            (word_86A2D6[v3 as usize] as i32 + ram_word!(game, samus_x_pos) as i32) as u16
    );
    set_ram_word!(game.eproj_y_pos[v1] = (ram_word!(game, samus_y_pos) as i32 + 80i32) as u16);
    set_ram_word!(
        game.eproj_instr_list_ptr[v1] = crate::rtl::rom_table_word!(game, 0x86a2e2, v3 as isize)
    );
    set_ram_word!(game.eproj_E[v1] = v2);
}
fn EprojInit_CeresElevatorPad(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    let eproj_y_pos: u16 = game.samus_y_pos().wrapping_add(28);
    EprojStateMut(&mut game.g_ram).write(0x1a93, v1 as usize, eproj_y_pos);
    EprojStateMut(&mut game.g_ram).write(0x1aff, v1 as usize, 60u16);
    sub_86A301(j, game);
}
pub fn sub_86A301(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    EprojStateMut(&mut game.g_ram).write(0x1a27, v1 as usize, 0u16);
    EprojStateMut(&mut game.g_ram).write(0x1a6f, v1 as usize, 0u16);
    EprojStateMut(&mut game.g_ram).write(0x1ab7, v1 as usize, 0u16);
    EprojStateMut(&mut game.g_ram).write(0x1adb, v1 as usize, 0u16);
    EprojStateMut(&mut game.g_ram).write(0x19bb, v1 as usize, 0u16);
    let eproj_x_pos: u16 = game.samus_x_pos();
    EprojStateMut(&mut game.g_ram).write(0x1a4b, v1 as usize, eproj_x_pos);
}
fn EprojInit_CeresElevatorPlatform(j: u16, game: &mut Game) {
    let eproj_word_index = (j as i32 >> 1i32) as usize;
    EprojStateMut(&mut game.g_ram).write(0x1a93, eproj_word_index, 97u16);
    sub_86A301(j, game);
}
pub unsafe fn EprojPreInstr_CeresElevatorPad(k: u16, game: &mut Game) {
    let mut v2: bool = false;
    let mut v3: bool = false;
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, eproj_E[v1]) == 0
        || {
            v2 = ram_word!(game, eproj_E[v1]) == 1u16;
            v3 = ((ram_word!(game, eproj_E[v1])) as i16).wrapping_sub(1i16) < 0i16;
            update_ram_word!(game.eproj_E[v1], wrapping_sub(1));
            v2 as i32 != 0
        }
        || v3
    {
        set_ram_word!(game.eproj_y_pos[v1] = game.samus_y_pos().wrapping_add(28));
        let fresh113 = EprojStateMut(&mut game.g_ram).add_word(0xafa, 0, 1);
        if fresh113 as i32 - 73i32 & 0x8000i32 == 0 {
            set_ram_word!(game.samus_y_pos = 72u16);
            set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
            set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_off_86A28B as u16);
            CallSomeSamusCode(0xeu16, game);
        }
    }
}
pub unsafe fn EprojPreInstr_CeresElevatorPlatform(k: u16, game: &mut Game) {
    if game.samus_y_pos() == 72 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
        set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_off_86A28B as u16);
    }
}
pub unsafe fn EprojPreInstr_PrePhantomRoom(j: u16, game: &mut Game) {
    set_ram_word!(game.bg2_y_scroll = 0u16);
}
pub unsafe fn EprojInstr_A3BE(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v1: i32 = k as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v1] = ram_word!(game, eproj_E[v1]));
    set_ram_word!(game.eproj_y_pos[v1] = ram_word!(game, eproj_F[v1]));
    epjp
}
pub unsafe fn EprojInstr_GotoWithProbability25(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    if (NextRandom(game) as i32 & 0xc000i32) == 0xc000i32 {
        *(epjp as *mut u16) as usize as *const u8
    } else {
        epjp.offset(2)
    }
}
unsafe fn EprojInit_BombTorizoLowHealthDrool(j: u16, game: &mut Game) {
    let mut parameter_1: i16 = 0;
    let mut v4: u16 = 0;
    let mut v5: u16 = 0;
    let mut v8: u16 = 0;
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_gfx_idx[v1] = 0u16);
    set_ram_word!(
        game.eproj_instr_list_ptr[v1] = crate::rtl::rom_table_word!(
            game,
            0x86a64d,
            ((((NextRandom(game) as i32 >> 1i32) as u16) & (0xeu16)) as i32 >> 1i32) as isize,
        )
    );
    NextRandom(game);
    let v2: *mut EnemyData = gEnemyData(game.cur_enemy_index() as i32, game);
    set_ram_word!(game.eproj_y_pos[v1] = ((*v2).y_pos).wrapping_sub(5u16));
    parameter_1 = (*v2).parameter_1 as i16;
    if (parameter_1 as i32 & 0x4000i32) != 0i32 {
        v4 = (ram_word!(game, random_number) as i32 & 0x1fei32) as u16;
    } else {
        if parameter_1 < 0 {
            v5 = 32u16;
        } else {
            v5 = 224u16;
        }
        v4 = (2i32 * (v5 as i32 + (game.random_number() as i32 & 0xfi32) - 8i32)) as u16;
    }
    let v6: i32 = v4 as i32 >> 1i32;
    set_ram_word!(game.eproj_x_vel[v1] = kSinCosTable8bit_Sext[(v6 + 64i32) as usize] as u16);
    set_ram_word!(game.eproj_y_vel[v1] = kSinCosTable8bit_Sext[v6 as usize] as u16);
    let v7: *mut EnemyData = gEnemyData(game.cur_enemy_index() as i32, game);
    if ((*v7).parameter_1 as i32 & 0x8000i32) != 0i32 {
        v8 = ((*v7).x_pos).wrapping_add(8u16);
    } else {
        v8 = ((*v7).x_pos as i32 - 8i32) as u16;
    }
    set_ram_word!(game.eproj_x_pos[v1] = v8);
}
unsafe fn EprojInit_BombTorizoLowHealthInitialDrool(j: u16, game: &mut Game) {
    let mut v3: i16 = 0;
    let mut parameter_1: i16 = 0;
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_gfx_idx[v1] = 0u16);
    NextRandom(game);
    let v2: *mut EnemyData = gEnemyData(game.cur_enemy_index() as i32, game);
    // Avoid a shared Game reference while the returned enemy pointer remains live.
    set_ram_word!(
        game.eproj_y_pos[v1] = (((*v2).y_pos as i32
            + (u16::from_le(
                (&raw const game.g_ram)
                    .cast::<u8>()
                    .add(0x5e5)
                    .cast::<u16>()
                    .read_unaligned(),
            ) as i32
                & 3i32)) as u16)
            .wrapping_sub(5u16)
    );
    set_ram_word!(
        game.eproj_y_vel[v1] = ((u16::from_le(
            (&raw const game.g_ram)
                .cast::<u8>()
                .add(0x5e5)
                .cast::<u16>()
                .read_unaligned(),
        ) as i32
            & 0x1fi32) as u16)
            .wrapping_add(48u16)
    );
    NextRandom(game);
    v3 = ((u16::from_le(
        (&raw const game.g_ram)
            .cast::<u8>()
            .add(0x5e5)
            .cast::<u16>()
            .read_unaligned(),
    )) as i16)
        & (3i16);
    parameter_1 = (*v2).parameter_1 as i16;
    if (parameter_1 as i32 & 0x4000i32) != 0i32 {
        set_ram_word!(game.eproj_x_pos[v1] = ((*v2).x_pos as i32 + v3 as i32) as u16);
        set_ram_word!(game.eproj_x_vel[v1] = 0u16);
    } else {
        if parameter_1 < 0 {
            set_ram_word!(
                game.eproj_x_pos[v1] = (((*v2).x_pos as i32 + v3 as i32) as u16).wrapping_add(8u16)
            );
        } else {
            set_ram_word!(
                game.eproj_x_pos[v1] = (((*v2).x_pos as i32 + v3 as i32) as u16).wrapping_sub(8u16)
            );
        }
        set_ram_word!(game.eproj_x_vel[v1] = 0u16);
    };
}
unsafe fn EprojInit_A977(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_gfx_idx[v1] = 0u16);
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    set_ram_word!(game.eproj_x_pos[v1] = (16i32 * ram_word!(game, plm_x_block) as i32) as u16);
    set_ram_word!(
        game.eproj_y_pos[v1] = (16i32 * ram_word!(game, plm_y_block) as i32 - 4i32) as u16
    );
    set_ram_word!(game.eproj_x_subpos[v1] = 0u16);
    set_ram_word!(game.eproj_y_subpos[v1] = 0u16);
}
static kEprojInit_BombTorizoExplosiveSwipe_Tab0: [i16; 11] = [
    -30i16, -40i16, -47i16, -31i16, -21i16, -1i16, -28i16, -43i16, -48i16, -31i16, -21i16,
];
static kEprojInit_BombTorizoExplosiveSwipe_Tab1: [i16; 11] = [
    -52i16, -28i16, -11i16, 9i16, 21i16, 20i16, -52i16, -27i16, -10i16, 9i16, 20i16,
];
static kEprojInit_BombTorizoStatueBreaking_Xpos: [i16; 16] = [
    8i16, 0x18i16, -8i16, 8i16, 0x18i16, -8i16, 8i16, 0x18i16, 8i16, -8i16, 0x18i16, 8i16, -8i16,
    0x18i16, 8i16, -8i16,
];
static kEprojInit_BombTorizoStatueBreaking_Ypos: [i16; 8] =
    [-8i16, -8i16, 8i16, 8i16, 8i16, 0x18i16, 0x18i16, 0x18i16];
static kEprojInit_BombTorizoStatueBreaking_Yvel: [i16; 8] = [
    256i16, 256i16, 256i16, 256i16, 256i16, 256i16, 256i16, 256i16,
];
static kEprojInit_BombTorizoStatueBreaking_F: [i16; 8] =
    [16i16, 16i16, 16i16, 16i16, 16i16, 16i16, 16i16, 16i16];
static kEprojInit_BombTorizoLowHealthExplode_X: [i16; 6] =
    [0i16, 12i16, -12i16, 0i16, 16i16, -16i16];
static kEprojInit_BombTorizoLowHealthExplode_Y: [i16; 6] =
    [-8i16, -8i16, -8i16, -20i16, -20i16, -20i16];
unsafe fn EprojInit_BombTorizoExplosiveSwipe(j: u16, game: &mut Game) {
    let v1: *mut EnemyData = gEnemyData(game.cur_enemy_index() as i32, game);
    if ((*v1).parameter_1 as i32 & 0x8000i32) != 0i32 {
        let v4: i32 = ram_word!(game, cinematic_var8) as i32 >> 1i32;
        let v5: i32 = j as i32 >> 1i32;
        set_ram_word!(
            game.eproj_x_pos[v5] = ((*v1).x_pos)
                .wrapping_sub((kEprojInit_BombTorizoExplosiveSwipe_Tab0[v4 as usize]) as u16)
        );
        set_ram_word!(
            game.eproj_y_pos[v5] = ((*v1).y_pos)
                .wrapping_add((kEprojInit_BombTorizoExplosiveSwipe_Tab1[v4 as usize]) as u16)
        );
    } else {
        let v2: i32 = ram_word!(game, cinematic_var8) as i32 >> 1i32;
        let v3: i32 = j as i32 >> 1i32;
        set_ram_word!(
            game.eproj_x_pos[v3] = ((*v1).x_pos as i32
                + kEprojInit_BombTorizoExplosiveSwipe_Tab0[v2 as usize] as i32)
                as u16
        );
        set_ram_word!(
            game.eproj_y_pos[v3] = ((*v1).y_pos as i32
                + kEprojInit_BombTorizoExplosiveSwipe_Tab1[v2 as usize] as i32)
                as u16
        );
    };
}
unsafe fn EprojInit_BombTorizoStatueBreaking(j: u16, game: &mut Game) {
    let mut v1: i8 = 0;
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    v1 = EprojState(&game.g_ram).init_param_1() as i8;
    let v2: i32 = EprojState(&game.g_ram).init_param_1() as i32 >> 1i32;
    let v3: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_instr_list_ptr[v3] = crate::rtl::rom_table_word!(game, 0x86a7ab, v2 as isize)
    );
    set_ram_word!(
        game.eproj_x_pos[v3] = (kEprojInit_BombTorizoStatueBreaking_Xpos[v2 as usize] as i32
            + 16i32 * ram_word!(game, plm_x_block) as i32) as u16
    );
    let v4: i32 = (v1 as i32 & 0xfi32) >> 1i32;
    set_ram_word!(
        game.eproj_y_pos[v3] = (kEprojInit_BombTorizoStatueBreaking_Ypos[v4 as usize] as i32
            + 16i32 * ram_word!(game, plm_y_block) as i32) as u16
    );
    set_ram_word!(
        game.eproj_y_vel[v3] = kEprojInit_BombTorizoStatueBreaking_Yvel[v4 as usize] as u16
    );
    set_ram_word!(game.eproj_F[v3] = kEprojInit_BombTorizoStatueBreaking_F[v4 as usize] as u16);
    EprojStateMut(&mut game.g_ram).set_property_bits(v3 as usize, 0x1000);
}
unsafe fn EprojInit_BombTorizoLowHealthExplode(j: u16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v2] = game.enemy_state(enemy_index).x_pos());
    set_ram_word!(game.eproj_y_pos[v2] = game.enemy_state(enemy_index).y_pos());
    if (game.enemy_state(enemy_index).parameter_1() as i32 & 0x8000i32) == 0i32 {
        update_ram_word!(game.cinematic_var8, |fresh13| (fresh13 as i32 + 2i32)
            as u16);
    }
    EprojStateMut(&mut game.g_ram).add_word(0x1993, 0, 2);
    let v3: i32 = EprojState(&game.g_ram).init_param_1() as i32 >> 1i32;
    let v4: u16 = (kEprojInit_BombTorizoLowHealthExplode_X[v3 as usize] as i32
        + ram_word!(game, eproj_x_pos[v2]) as i32) as u16;
    set_ram_word!(game.eproj_x_pos[v2] = v4);
    set_ram_word!(game.eproj_E[v2] = v4);
    let v5: u16 = (kEprojInit_BombTorizoLowHealthExplode_Y[v3 as usize] as i32
        + ram_word!(game, eproj_y_pos[v2]) as i32) as u16;
    set_ram_word!(game.eproj_y_pos[v2] = v5);
    set_ram_word!(game.eproj_F[v2] = v5);
}
unsafe fn EprojInit_BombTorizoDeathExplosion(j: u16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let x_pos: u16 = game.enemy_state(enemy_index).x_pos();
    let v3: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v3] = x_pos);
    set_ram_word!(game.eproj_E[v3] = x_pos);
    let y_pos: u16 = game.enemy_state(enemy_index).y_pos();
    set_ram_word!(game.eproj_y_pos[v3] = y_pos);
    set_ram_word!(game.eproj_F[v3] = y_pos);
}
pub unsafe fn sub_86A887(v0: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    let mut v3: i16 = 0;
    let mut v4: i16 = 0;
    if EprojBlockCollisition_Horiz(v0, game) as i32 & 1i32 != 0 {
        let v6: i32 = v0 as i32 >> 1i32;
        set_ram_word!(game.eproj_instr_list_ptr[v6] = 0xa48au16);
        set_ram_word!(game.eproj_instr_timers[v6] = 1u16);
    } else {
        let v1: i32 = v0 as i32 >> 1i32;
        v2 = ram_word!(game, eproj_x_vel[v1]) as i16;
        if v2 >= 0 {
            v4 = (v2).wrapping_sub(4i16);
            if v4 < 0 {
                v4 = 3i16;
            }
            set_ram_word!(game.eproj_x_vel[v1] = v4 as u16);
        } else {
            v3 = (v2 as i32 + 4i32) as i16;
            if v3 >= 0i16 {
                v3 = 3i16;
            }
            set_ram_word!(game.eproj_x_vel[v1] = v3 as u16);
        }
        let carry: u8 = EprojBlockCollisition_Vertical(v0, game);
        if (ram_word!(game, eproj_y_vel[v1]) as i32 & 0x8000i32) != 0i32 || carry == 0 {
            let v5: u16 = EprojStateMut(&mut game.g_ram).add_word(0x1adb, v1 as usize, 16);
            if (v5 as i32 & 0xf000i32) == 4096i32 {
                set_ram_word!(game.eproj_id[v1] = 0u16);
            }
        } else {
            let v7: i32 = v0 as i32 >> 1i32;
            EprojStateMut(&mut game.g_ram).add_word(0x1a93, v7 as usize, (-3i16) as u16);
            EprojStateMut(&mut game.g_ram).start_instruction(v7 as usize, 0xa48eu16);
        }
    };
}
pub unsafe fn EprojPreInstr_A977(k: u16, game: &mut Game) {
    let carry: u8 = EprojBlockCollisition_Vertical(k, game);
    let v1: i32 = k as i32 >> 1i32;
    if (ram_word!(game, eproj_y_vel[v1]) as i32 & 0x8000i32) != 0i32 || carry == 0 {
        let v2: u16 = (ram_word!(game, eproj_F[0])).wrapping_add(ram_word!(game, eproj_y_vel[v1]));
        set_ram_word!(game.eproj_y_vel[v1] = v2);
        if (v2 as i32 & 0xf000i32) == 4096i32 {
            set_ram_word!(game.eproj_y_vel[v1] = 4096u16);
        }
    } else {
        set_ram_word!(game.eproj_pre_instr[v1] = 0xa918u16);
    };
}
pub unsafe fn sub_86A91A(v0: u16, game: &mut Game) {
    let v1: i32 = v0 as i32 >> 1i32;
    set_ram_word!(game.eproj_x_vel[v1] = 0u16);
    if (ram_word!(game, joypad2_last) as i32 & 0x100i32) != 0i32 {
        set_ram_word!(game.eproj_x_vel[v1] = 256u16);
    }
    if (ram_word!(game, joypad2_last) as i32 & 0x200i32) != 0i32 {
        set_ram_word!(game.eproj_x_vel[v1] = -256i32 as u16);
    }
    EprojBlockCollisition_Horiz(v0, game);
    set_ram_word!(game.eproj_y_vel[v1] = 0u16);
    if (ram_word!(game, joypad2_last) as i32 & 0x400i32) != 0i32 {
        set_ram_word!(game.eproj_y_vel[v1] = 256u16);
    }
    if (ram_word!(game, joypad2_last) as i32 & 0x800i32) != 0i32 {
        set_ram_word!(game.eproj_y_vel[v1] = -256i32 as u16);
    }
    EprojBlockCollisition_Vertical(v0, game);
}
unsafe fn EprojInit_AB07(j: u16, game: &mut Game) {
    let mut v1: u16 = ram_word!(game, vram_write_queue_tail);
    let v2: *mut VramWriteEntry =
        gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
    (*v2).size = 64u16;
    (*v2).src.addr = addr_kEprojInit_AB07_Tile0 as i32 as VoidP;
    *(&raw mut (*v2).src.bank as *mut u16) = 134u16;
    (*v2).vram_dst = 28160u16;
    v1 = (v1 as i32 + 7i32) as u16;
    let v3: *mut VramWriteEntry = gVramWriteEntry(v1 as i32, game);
    (*v3).size = 64u16;
    (*v3).src.addr = addr_kEprojInit_AB07_Tile1 as i32 as VoidP;
    *(&raw mut (*v3).src.bank as *mut u16) = 134u16;
    (*v3).vram_dst = 28416u16;
    set_ram_word!(game.vram_write_queue_tail = (v1 as i32 + 7i32) as u16);
    let v4: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v4] = ram_word!(game, samus_x_pos));
    set_ram_word!(game.eproj_y_pos[v4] = (ram_word!(game, samus_y_pos) as i32 - 36i32) as u16);
}
pub unsafe fn EprojPreInstr_AB07(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let mut amt: i32 = ((8i32
        * (ram_word!(game, samus_x_pos) as i32 - ram_word!(game, eproj_x_pos[v1]) as i32))
        as i16 as i32)
        << 8i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16).offset(v1 as isize)
            as *mut u16,
        amt as u32,
    );
    amt = ((8i32
        * (ram_word!(game, samus_y_pos) as i32 - 36i32 - ram_word!(game, eproj_y_pos[v1]) as i32))
        as i16 as i32)
        << 8i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16).offset(v1 as isize)
            as *mut u16,
        amt as u32,
    );
}
pub unsafe fn EprojInstr_SpawnEnemyDrops(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v2: i32 = k as i32 >> 1i32;
    game.eproj_spawn_pt = Point16U {
        x: ram_word!(game, eproj_x_pos[v2]),
        y: ram_word!(game, eproj_y_pos[v2]),
    };
    SpawnEnemyDrops(
        (if ram_word!(game, area_index) != 0u16 {
            *(epjp.offset(2) as *mut u16) as i32
        } else {
            *(epjp as *mut u16) as i32
        }) as u16,
        k,
        0u16,
        game,
    );
    epjp.offset(4)
}
unsafe fn Eproj_InitXYVelRandom(j: u16, k: u16, pt: Point16U, game: &mut Game) {
    let v2: *const u16 = RomPtr_86(k, game) as *const u16;
    let v3: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_instr_list_ptr[v3] = *v2);
    set_ram_word!(game.eproj_x_pos[v3] = (*v2.offset(1)).wrapping_add(pt.x));
    set_ram_word!(
        game.eproj_x_vel[v3] =
            (*v2.offset(2) as i32 + NextRandom(game) as u8 as i32 - 128i32) as u16
    );
    set_ram_word!(game.eproj_y_pos[v3] = (*v2.offset(3)).wrapping_add(pt.y));
    set_ram_word!(
        game.eproj_y_vel[v3] =
            (*v2.offset(4) as i32 + NextRandom(game) as u8 as i32 - 128i32) as u16
    );
}
unsafe fn EprojInit_BombTorizosChozoOrbs(j: u16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v2: u16 = (if (game.enemy_state(enemy_index).parameter_1() as i32 & 0x8000i32) != 0i32 {
        addr_kEprojInit_BombTorizosChozoOrbs_init0 as i32
    } else {
        addr_kEprojInit_BombTorizosChozoOrbs_init1 as i32
    }) as u16;
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
unsafe fn EprojInit_GoldenTorizosChozoOrbs(j: u16, game: &mut Game) {
    let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: u16 = (if ((*E).parameter_1 as i32 & 0x8000i32) != 0i32 {
        addr_kEprojInit_GoldenTorizosChozoOrbs_init0 as i32
    } else {
        addr_kEprojInit_GoldenTorizosChozoOrbs_init1 as i32
    }) as u16;
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
pub unsafe fn EprojPreInstr_BombTorizosChozoOrbs(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) != 0 {
        let v3: i32 = k as i32 >> 1i32;
        EprojStateMut(&mut game.g_ram).start_instruction(v3 as usize, addr_off_86AB25 as u16);
    } else {
        let carry: u8 = EprojBlockCollisition_Vertical(k, game);
        let v1: i32 = k as i32 >> 1i32;
        if (ram_word!(game, eproj_y_vel[v1]) as i32 & 0x8000i32) != 0i32 || carry == 0 {
            let v2: u16 = EprojStateMut(&mut game.g_ram).add_word(0x1adb, v1 as usize, 18);
            if (v2 as i32 & 0xf000i32) == 4096i32 {
                set_ram_word!(game.eproj_id[v1] = 0u16);
            }
        } else {
            let v4: i32 = k as i32 >> 1i32;
            set_ram_word!(
                game.eproj_y_pos[v4] = ((game.enemy_projectile_y_pos(k) as i32 & 0xfff0i32 | 8i32)
                    as u16)
                    .wrapping_sub(2u16)
            );
            EprojStateMut(&mut game.g_ram).start_instruction(v4 as usize, addr_off_86AB41 as u16);
        }
    };
}
pub unsafe fn EprojPreInstr_GoldenTorizosChozoOrbs(k: u16, game: &mut Game) {
    let mut v1: i32 = 0;
    let mut v2: i16 = 0;
    let mut v3: u16 = 0;
    let mut v4: u16 = 0;
    if EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0 {
        set_ram_word!(
            game.eproj_x_vel[(k as i32 >> 1i32)] =
                -(ram_word!(game, eproj_x_vel[(k as i32 >> 1i32)]) as i32) as u16
        );
    }
    if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0
        && {
            v1 = k as i32 >> 1i32;
            (ram_word!(game, eproj_y_vel[v1]) as i32 & 0x8000i32) == 0i32
        }
        && {
            v2 = ram_word!(game, eproj_x_vel[v1]) as i16;
            if v2 >= 0i16 {
                v3 = (v2 as i32 - 64i32) as u16;
            } else {
                v3 = (v2 as i32 + 64i32) as u16;
            };
            set_ram_word!(game.eproj_x_vel[v1] = v3);
            v4 = -(ram_word!(game, eproj_y_vel[v1]) as i32 >> 1i32) as u16;
            set_ram_word!(game.eproj_y_vel[v1] = v4);
            (v4 as i32 & 0xff80i32) == 0xff80i32
        }
    {
        let v5: i32 = k as i32 >> 1i32;
        set_ram_word!(
            game.eproj_y_pos[v5] =
                ((ram_word!(game, eproj_y_pos[v5]) as i32 & 0xfff0i32 | 8i32) - 2i32) as u16
        );
        set_ram_word!(game.eproj_instr_list_ptr[v5] = addr_off_86AB41 as u16);
        set_ram_word!(game.eproj_instr_timers[v5] = 1u16);
    } else {
        update_ram_word!(
            game.eproj_y_vel[(k as i32 >> 1i32)],
            |fresh110| (fresh110 as i32 + 24i32) as u16
        );
    };
}
pub unsafe fn EprojInstr_GotoDependingOnXDirection(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    let v2: i32 = k as i32 >> 1i32;
    let amt: i32 = (ram_word!(game, eproj_x_vel[v2]) as i16 as i32) << 8i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16).offset(v2 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16).offset(v2 as isize)
            as *mut u16,
        amt as u32,
    );
    if (ram_word!(game, eproj_x_vel[v2]) as i32 & 0x8000i32) == 0i32 {
        *(epjp.offset(2) as *mut u16) as usize as *const u8
    } else {
        *(epjp as *mut u16) as usize as *const u8
    }
}
unsafe fn EprojInit_TorizoSonicBoom(j: u16, game: &mut Game) {
    let v1: i32 = if (NextRandom(game) as i32 & 1i32) != 0i32 {
        -12i32
    } else {
        20i32
    };
    let v2: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let v3: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_y_pos[v3] = ((*v2).y_pos as i32 + v1) as u16);
    set_ram_word!(game.eproj_y_vel[v3] = 0u16);
    if ((*v2).parameter_1 as i32 & 0x8000i32) != 0i32 {
        set_ram_word!(game.eproj_x_pos[v3] = ((*v2).x_pos as i32 + 32i32) as u16);
        set_ram_word!(game.eproj_x_vel[v3] = 624u16);
        set_ram_word!(game.eproj_instr_list_ptr[v3] = addr_off_86ADD2 as u16);
    } else {
        set_ram_word!(game.eproj_x_pos[v3] = ((*v2).x_pos as i32 - 32i32) as u16);
        set_ram_word!(game.eproj_x_vel[v3] = -624i32 as u16);
        set_ram_word!(game.eproj_instr_list_ptr[v3] = addr_off_86ADBF as u16);
    };
}
pub unsafe fn EprojPreInstr_TorizoSonicBoom(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0 {
        let v4: i32 = k as i32 >> 1i32;
        set_ram_word!(game.eproj_instr_list_ptr[v4] = addr_off_86ADE5 as u16);
        set_ram_word!(game.eproj_instr_timers[v4] = 1u16);
        set_ram_word!(game.eproj_E[v4] = ram_word!(game, eproj_x_pos[v4]));
        set_ram_word!(game.eproj_F[v4] = ram_word!(game, eproj_y_pos[v4]));
    } else {
        let v1: i32 = k as i32 >> 1i32;
        update_ram_word!(game.eproj_x_vel[v1], |fresh109| (fresh109 as i32
            + if ram_word!(game, eproj_x_vel[v1]) as i32 & 0x8000i32 != 0 {
                -16i32
            } else {
                16i32
            }) as u16);
        if (ram_word!(game, eproj_x_vel[v1]) as i32 & 0xf000i32) == 4096i32 {
            set_ram_word!(game.eproj_id[v1] = 0u16);
        }
    };
}
unsafe fn EprojInit_WreckedShipChozoSpikeFootsteps(j: u16, game: &mut Game) {
    let v1: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v2] = (ram_word!(game, cinematic_var8) as i32 + (*v1).x_pos as i32) as u16
    );
    set_ram_word!(game.eproj_y_pos[v2] = ((*v1).y_pos as i32 + 28i32) as u16);
}
pub unsafe fn EprojInstr_ResetXYpos1(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v2: i32 = k as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v2] = ram_word!(game, eproj_E[v2]));
    set_ram_word!(game.eproj_y_pos[v2] = ram_word!(game, eproj_F[v2]));
    epjp
}
unsafe fn EprojInit_TourianStatueDustClouds(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_E[v1] = 128u16);
    set_ram_word!(game.eproj_F[v1] = 188u16);
}
unsafe fn EprojInit_TourianLandingDustCloudsRightFoot(j: u16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_y_pos[v2] = (game.enemy_state(enemy_index).y_pos()).wrapping_add(48u16)
    );
    set_ram_word!(
        game.eproj_x_pos[v2] = (game.enemy_state(enemy_index).x_pos()).wrapping_add(24u16)
    );
}
pub fn EprojInstr_MoveY_Minus4(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    EprojStateMut(&mut game.g_ram).add_word(0x1a93, usize::from(k >> 1), (-4i16) as u16);
    epjp
}
unsafe fn EprojInit_TorizoLandingDustCloudLeftFoot(j: u16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_y_pos[v2] = (game.enemy_state(enemy_index).y_pos()).wrapping_add(48u16)
    );
    set_ram_word!(
        game.eproj_x_pos[v2] = (game.enemy_state(enemy_index).x_pos()).wrapping_sub(24u16)
    );
}
unsafe fn EprojInit_GoldenTorizoEgg(j: u16, game: &mut Game) {
    let v1: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_F[v2] = ((0xe2i32 & 0x1fi32) + 64i32) as u16);
    set_ram_word!(game.eproj_E[v2] = (*v1).parameter_1);
    let v4: u16 = (if (*v1).parameter_1 as i32 & 0x8000i32 != 0 {
        addr_kEprojInit_GoldenTorizoEgg0 as i32
    } else {
        addr_kEprojInit_GoldenTorizoEgg1 as i32
    }) as u16;
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
pub unsafe fn EprojPreInstr_GoldenTorizoEgg(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if (update_ram_word!(game.eproj_F[v1], wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32 {
        update_ram_word!(
            game.eproj_instr_list_ptr[v1],
            |fresh105| (fresh105 as i32 + 2i32) as u16
        );
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
        set_ram_word!(
            game.eproj_x_vel[v1] = (if (ram_word!(game, eproj_E[v1]) as i32 & 0x8000i32) != 0i32 {
                256i32
            } else {
                -256i32
            }) as u16
        );
    } else {
        if EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0 {
            set_ram_word!(game.eproj_x_vel[v1] = -(ram_word!(game, eproj_x_vel[v1]) as i32) as u16);
            update_ram_word!(game.eproj_E[v1], |fresh106| (fresh106 as i32 ^ 0x8000i32)
                as u16);
        }
        if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0
            && (ram_word!(game, eproj_y_vel[v1]) as i32 & 0x8000i32) == 0i32
        {
            update_ram_word!(game.eproj_x_vel[v1], |fresh107| (fresh107 as i32
                + if ram_word!(game, eproj_x_vel[v1]) as i32 & 0x8000i32 != 0 {
                    32i32
                } else {
                    -32i32
                }) as u16);
            set_ram_word!(game.eproj_y_vel[v1] = -(ram_word!(game, eproj_y_vel[v1]) as i32) as u16);
        }
        update_ram_word!(game.eproj_y_vel[v1], |fresh108| (fresh108 as i32 + 48i32)
            as u16);
        if (ram_word!(game, eproj_y_vel[v1]) as i32 & 0xf000i32) == 4096i32 {
            set_ram_word!(game.eproj_id[v1] = 0u16);
        }
    };
}
pub unsafe fn sub_86B0B9(k: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    if EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0 {
        let v3: i32 = k as i32 >> 1i32;
        set_ram_word!(game.eproj_pre_instr[v3] = 0xb0ddu16);
        set_ram_word!(game.eproj_y_vel[v3] = 0u16);
    } else {
        let v1: i32 = k as i32 >> 1i32;
        if (ram_word!(game, eproj_E[v1]) as i32 & 0x8000i32) != 0i32 {
            v2 = 48i16;
        } else {
            v2 = -48i16;
        }
        update_ram_word!(
            game.eproj_x_vel[v1],
            |fresh103| (fresh103 as i32 + v2 as i32) as u16
        );
    };
}
pub unsafe fn sub_86B0DD(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(
            game.eproj_instr_list_ptr[v1] =
                (if (ram_word!(game, eproj_E[v1]) as i32 & 0x8000i32) != 0i32 {
                    addr_off_86B1A8 as i32
                } else {
                    addr_off_86B190 as i32
                }) as u16
        );
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
    } else {
        update_ram_word!(
            game.eproj_y_vel[(k as i32 >> 1i32)],
            |fresh102| (fresh102 as i32 + 48i32) as u16
        );
    };
}
pub unsafe fn sub_86B13E(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    ((if (ram_word!(game, eproj_E[(k as i32 >> 1i32)]) as i32 & 0x8000i32) != 0i32 {
        addr_off_86B166 as i32
    } else {
        addr_off_86B14B as i32
    }) as usize as *const u8)
}
static word_86B205: [i16; 2] = [-0x1ei16, 0x1ei16];
unsafe fn EprojInit_GoldenTorizoSuperMissile(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_E[v2] = ram_word!(game, cur_enemy_index));
    let v3: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let v4: i32 = if ((*v3).parameter_1 as i32 & 0x8000i32) != 0i32 {
        1i32
    } else {
        0i32
    };
    set_ram_word!(
        game.eproj_x_pos[v2] = ((*v3).x_pos as i32 + word_86B205[v4 as usize] as i32) as u16
    );
    set_ram_word!(game.eproj_y_pos[v2] = ((*v3).y_pos as i32 - 52i32) as u16);
    set_ram_word!(
        game.eproj_instr_list_ptr[v2] = crate::rtl::rom_table_word!(game, 0x86b209, v4 as isize)
    );
}
pub unsafe fn EprojPreInstr_GoldenTorizoSuperMissile(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let v2: *mut EnemyData = gEnemyData(ram_word!(game, eproj_E[v1]) as i32, game);
    let v3: i32 = if ((*v2).parameter_1 as i32 & 0x8000i32) != 0i32 {
        32i32
    } else {
        -32i32
    };
    set_ram_word!(game.eproj_x_pos[v1] = ((*v2).x_pos as i32 + v3) as u16);
    set_ram_word!(game.eproj_y_pos[v1] = ((*v2).y_pos as i32 - 52i32) as u16);
}
pub unsafe fn EprojPreInstr_B237(k: u16, game: &mut Game) {
    let mut v1: i32 = 0;
    let mut carry: u8 = 0;
    if EprojBlockCollisition_Horiz(k, game) != 0u8 || {
        carry = EprojBlockCollisition_Vertical(k, game);
        v1 = k as i32 >> 1i32;
        (ram_word!(game, eproj_y_vel[v1]) as i32 & 0x8000i32) == 0i32 && carry != 0u8
    } {
        let v3: i32 = k as i32 >> 1i32;
        set_ram_word!(game.eproj_instr_list_ptr[v3] = addr_off_86B2EF as u16);
        set_ram_word!(game.eproj_instr_timers[v3] = 1u16);
    } else {
        let v2: u16 = (ram_word!(game, eproj_y_vel[v1]) as i32 + 16i32) as u16;
        set_ram_word!(game.eproj_y_vel[v1] = v2);
        if (v2 as i32 & 0xf000i32) == 4096i32 {
            set_ram_word!(game.eproj_id[v1] = 0u16);
        }
    };
}
unsafe fn sub_86B279(k: u16, a: u16, game: &mut Game) {
    let v3: i32 = k as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_vel[v3] =
            (4i32 * kSinCosTable8bit_Sext[(a as i32 + 64i32) as usize] as i32) as u16
    );
    set_ram_word!(game.eproj_y_vel[v3] = (4i32 * kSinCosTable8bit_Sext[a as usize] as i32) as u16);
}
pub unsafe fn EprojInstr_SetVelTowardsSamus1(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    sub_86B279(
        k,
        (CalculateAngleOfSamusFromEproj(k, game) as i32 & 0x7fi32) as u16,
        game,
    );
    epjp
}
pub unsafe fn EprojInstr_SetVelTowardsSamus2(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    sub_86B279(
        k,
        (CalculateAngleOfSamusFromEproj(k, game) as i32 | 0x80i32) as u16,
        game,
    );
    epjp
}
unsafe fn EprojInit_GoldenTorizoEyeBeam(j: u16, game: &mut Game) {
    let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: u16 = (if ((*E).parameter_1 as i32 & 0x8000i32) != 0i32 {
        addr_stru_86B376 as i32
    } else {
        addr_stru_86B380 as i32
    }) as u16;
    Eproj_InitXYVelRandom(
        j,
        v2,
        Point16U {
            x: (*E).x_pos,
            y: (*E).y_pos,
        },
        game,
    );
    let mut v3: u16 = ((NextRandom(game) as i32 & 0x1ei32) - 16i32 + 192i32) as u16;
    if ((*E).parameter_1 as i32 & 0x8000i32) == 0i32 {
        v3 = (v3 as i32 + 128i32) as u16;
    }
    let v4: i32 = v3 as i32 >> 1i32;
    let v5: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_vel[v5] = (8i32 * kSinCosTable8bit_Sext[(v4 + 64i32) as usize] as i32) as u16
    );
    set_ram_word!(game.eproj_y_vel[v5] = (8i32 * kSinCosTable8bit_Sext[v4 as usize] as i32) as u16);
}
pub unsafe fn EprojPreInstr_GoldenTorizoEyeBeam(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0 {
        set_ram_word!(game.eproj_instr_list_ptr[(k as i32 >> 1i32)] = addr_off_86B3CD as u16);
        set_ram_word!(game.eproj_instr_timers[(k as i32 >> 1i32)] = 1u16);
        return;
    }
    if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(
            game.eproj_y_pos[v1] =
                ((ram_word!(game, eproj_y_pos[v1]) as i32 & 0xfff0i32 | 8i32) - 2i32) as u16
        );
        set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_off_86B3E5 as u16);
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
    }
}
pub unsafe fn EprojInstr_GotoIfFunc1(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    if ((*gExtraEnemyRam7800(ram_word!(game, eproj_F[(k as i32 >> 1i32)]) as i32, game))
        .kraid
        .kraid_healths_8ths[0i32 as usize] as i32
        & 0x8000i32)
        == 0i32
    {
        *(epjp as *mut u16) as usize as *const u8
    } else {
        epjp.offset(2)
    }
}
pub unsafe fn EprojInstr_ResetXYpos2(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v2: i32 = k as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v2] = ram_word!(game, eproj_E[v2]));
    set_ram_word!(game.eproj_y_pos[v2] = ram_word!(game, eproj_F[v2]));
    epjp
}
unsafe fn EprojInit_TourianEscapeShaftFakeWallExplode(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v1] = 272u16);
    set_ram_word!(game.eproj_E[v1] = 272u16);
    set_ram_word!(game.eproj_y_pos[v1] = 2184u16);
    set_ram_word!(game.eproj_F[v1] = 2184u16);
}
unsafe fn EprojInit_LavaSeahorseFireball(j: u16, game: &mut Game) {
    let v1: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_y_pos[v2] = ((*v1).y_pos).wrapping_sub(28u16));
    set_ram_word!(game.eproj_y_vel[v2] = -961i32 as u16);
    if ((*v1).ai_var_A as i32 & 0x8000i32) == 0i32 {
        set_ram_word!(game.eproj_x_pos[v2] = ((*v1).x_pos as i32 + 12i32) as u16);
        set_ram_word!(game.eproj_x_vel[v2] = 704u16);
        set_ram_word!(game.eproj_instr_list_ptr[v2] = addr_word_86B4CB as u16);
    } else {
        set_ram_word!(game.eproj_x_pos[v2] = ((*v1).x_pos).wrapping_sub(12u16));
        set_ram_word!(game.eproj_x_vel[v2] = -704i32 as u16);
        set_ram_word!(game.eproj_instr_list_ptr[v2] = addr_word_86B4BF as u16);
    };
}
pub unsafe fn sub_86B535(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let split_amount =
        ((EprojState(&game.g_ram).slot_word(0x1ab7, v1 as usize) as i16 as i32) << 8i32) as u32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a4b, 0x1a27, v1 as usize, split_amount);
    let split_amount =
        ((EprojState(&game.g_ram).slot_word(0x1adb, v1 as usize) as i16 as i32) << 8i32) as u32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a93, 0x1a6f, v1 as usize, split_amount);
    let v6: i16 = EprojState(&game.g_ram).slot_word(0x1adb, v1 as usize) as i16;
    if v6 >= 0i16 {
        EprojStateMut(&mut game.g_ram).write(0x1adb, v1 as usize, (v6 as u16).wrapping_add(32));
        Eproj_DeleteIfYposOutside(k, game);
    } else {
        EprojStateMut(&mut game.g_ram).write(0x1adb, v1 as usize, (v6 as u16).wrapping_add(32));
        if (v6).wrapping_add(32i16) >= 0i16 {
            {
                let instruction: u16 =
                    (if (ram_word!(game, eproj_x_vel[v1]) as i32 & 0x8000i32) == 0i32 {
                        addr_word_86B4E3 as i32
                    } else {
                        addr_word_86B4D7 as i32
                    }) as u16;
                EprojStateMut(&mut game.g_ram).start_instruction(v1 as usize, instruction);
            }
        }
    };
}
pub unsafe fn Eproj_DeleteIfYposOutside(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if ((ram_word!(game, eproj_y_pos[v1])) as i16)
        .wrapping_sub((ram_word!(game, layer1_y_pos)) as i16)
        >= 288i16
    {
        EprojStateMut(&mut game.g_ram).set_id(v1 as usize, 0);
    }
}
static word_86B65B: [i16; 20] = [
    -16i16, 16i16, -96i16, -64i16, -128i16, -32i16, -96i16, 64i16, -128i16, 32i16, 16i16, 16i16,
    96i16, -64i16, 112i16, -64i16, 128i16, -64i16, 144i16, -64i16,
];
unsafe fn EprojInit_EyeDoorProjectile(j: u16, game: &mut Game) {
    let v1: u16 = ram_word!(game, plm_id);
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_F[v2] = *((&raw mut game.g_ram as *mut u8).offset(0x1dc7) as *mut u16)
            .offset((ram_word!(game, plm_id) as i32 >> 1i32) as isize,)
    );
    CalculatePlmBlockCoords(v1, game);
    let v3: i32 = ram_word!(game, cinematic_var8) as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v2] = (word_86B65B[v3 as usize] as i32
            + 8i32 * (2i32 * ram_word!(game, plm_x_block) as i32 + 1i32))
            as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v2] = (word_86B65B[(v3 + 1i32) as usize] as i32
            + 16i32 * ram_word!(game, plm_y_block) as i32) as u16
    );
}
static word_86B6B1: [i16; 4] = [-64i16, 512i16, 64i16, 512i16];
unsafe fn EprojInit_EyeDoorSweat(j: u16, game: &mut Game) {
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v1] =
            (8i32 * (2i32 * (ram_word!(game, plm_x_block) as i32 - 1i32) + 1i32)) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v1] = (16i32 * (ram_word!(game, plm_y_block) as i32 + 1i32)) as u16
    );
    let v2: i32 = ram_word!(game, cinematic_var8) as i32 >> 1i32;
    set_ram_word!(game.eproj_x_vel[v1] = word_86B6B1[v2 as usize] as u16);
    set_ram_word!(game.eproj_y_vel[v1] = word_86B6B1[(v2 + 1i32) as usize] as u16);
}
pub unsafe fn EprojPreInstr_EyeDoorProjectile(k: u16, game: &mut Game) {
    let v3: i32 = k as i32 >> 1i32;
    if !(EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0
        || EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0)
    {
        let v1: i32 = ram_word!(game, eproj_E[(k as i32 >> 1i32)]) as i32 >> 1i32;
        update_ram_word!(
            game.eproj_x_vel[v3],
            wrapping_add((kSinCosTable8bit_Sext[(v1 + 64i32) as usize] as i32 >> 4i32) as u16)
        );
        update_ram_word!(game.eproj_y_vel[v3], |fresh101| (fresh101 as i32
            + (kSinCosTable8bit_Sext[v1 as usize] as i32 >> 4i32))
            as u16);
        let t: i32 = PrepareBitAccess(ram_word!(game, eproj_F[v3]), game) as i32;
        if ((ram_word!(game, bitmask) as i32
            & *(&raw mut game.g_ram as *mut u8)
                .offset(0xd8b0)
                .offset(t as isize) as i32)
            == 0i32)
        {
            return;
        }
    }
    EprojStateMut(&mut game.g_ram).start_instruction(v3 as usize, addr_off_86B5F3 as u16);
}
pub unsafe fn EprojPreInstr_EyeDoorSweat(k: u16, game: &mut Game) {
    EprojBlockCollisition_Horiz(k, game);
    let carry: u8 = EprojBlockCollisition_Vertical(k, game);
    let v1: i32 = k as i32 >> 1i32;
    if (ram_word!(game, eproj_y_vel[v1]) as i32 & 0x8000i32) != 0i32 || carry == 0 {
        EprojStateMut(&mut game.g_ram).add_word(0x1adb, v1 as usize, 12);
    } else {
        EprojStateMut(&mut game.g_ram).add_word(0x1a93, v1 as usize, (-4i16) as u16);
        EprojStateMut(&mut game.g_ram).start_instruction(v1 as usize, addr_off_86B61D as u16);
    };
}
pub unsafe fn EprojInstr_SpawnTourianStatueUnlockingParticle(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueUnlockingParticle as u16, k, game);
    epjp
}
pub unsafe fn EprojInstr_Earthquake(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.earthquake_type = 1u16);
    update_ram_word!(game.earthquake_timer, |fresh29| (fresh29 as i32 | 0x20i32)
        as u16);
    epjp
}
pub unsafe fn EprojInstr_SpawnTourianStatueUnlockingParticleTail(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_TourianStatueUnlockingParticleTail as u16,
        k,
        game,
    );
    epjp
}
pub unsafe fn EprojInstr_AddToYpos(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    update_ram_word!(
        game.eproj_y_pos[(k as i32 >> 1i32)],
        |fresh28| (fresh28 as i32 + *(epjp as *mut u16) as i32) as u16
    );
    epjp.offset(2)
}
unsafe fn EprojInit_TourianStatueUnlockingParticleWaterSplash(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v1] = *((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16)
            .offset((ram_word!(game, cinematic_var8) as i32 >> 1i32) as isize,)
    );
    set_ram_word!(game.eproj_y_pos[v1] = (ram_word!(game, fx_y_pos) as i32 - 4i32) as u16);
}
static kEprojInit_TourianStatueEyeGlow_X: [u16; 4] = [0x84u16, 0x7au16, 0x9eu16, 0x68u16];
static kEprojInit_TourianStatueEyeGlow_Y: [u16; 4] = [0x90u16, 0x51u16, 0x80u16, 0x72u16];
static kEprojInit_TourianStatueEyeGlow_Colors: [u16; 16] = [
    0x6bffu16, 0x33bu16, 0x216u16, 0x173u16, 0x7f5fu16, 0x7c1fu16, 0x5816u16, 0x300cu16, 0x7f5au16,
    0x7ec0u16, 0x6de0u16, 0x54e0u16, 0x6bfau16, 0x3be0u16, 0x2680u16, 0x1580u16,
];
unsafe fn EprojInit_TourianStatueEyeGlow(j: u16, game: &mut Game) {
    let v1: u16 = ram_word!(game, cinematic_var8);
    let v2: i32 = ram_word!(game, cinematic_var8) as i32 >> 1i32;
    let v3: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v3] = kEprojInit_TourianStatueEyeGlow_X[v2 as usize]);
    set_ram_word!(game.eproj_y_pos[v3] = kEprojInit_TourianStatueEyeGlow_Y[v2 as usize]);
    let mut v4: u16 = (4i32 * v1 as i32) as u16;
    let mut i: i32 = 498i32;
    while i != 506i32 {
        set_ram_word!(
            game.palette_buffer[(i >> 1i32)] =
                kEprojInit_TourianStatueEyeGlow_Colors[(v4 as i32 >> 1i32) as usize]
        );
        v4 = (v4 as i32 + 2i32) as u16;
        i += 2i32;
    }
}
unsafe fn EprojInit_TourianStatueUnlockingParticle(j: u16, game: &mut Game) {
    let v1: i32 = ram_word!(game, cinematic_var8) as i32 >> 1i32;
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v2] = ram_word!(game, eproj_x_pos[v1]));
    set_ram_word!(game.eproj_y_pos[v2] = ram_word!(game, eproj_y_pos[v1]));
    let v3: u16 = (2i32 * ((NextRandom(game) as i32 & 0x3fi32) - 32i32) as u8 as i32) as u16;
    set_ram_word!(game.eproj_E[v2] = v3);
    let v4: i32 = v3 as i32 >> 1i32;
    set_ram_word!(game.eproj_x_vel[v2] = kSinCosTable8bit_Sext[(v4 + 64i32) as usize] as u16);
    set_ram_word!(game.eproj_y_vel[v2] = (4i32 * kSinCosTable8bit_Sext[v4 as usize] as i32) as u16);
}
pub unsafe fn EprojIni_TourianStatueUnlockingParticleTail(v0: u16, game: &mut Game) {
    let v1: i32 = ram_word!(game, cinematic_var8) as i32 >> 1i32;
    let v2: i32 = v0 as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v2] = ram_word!(game, eproj_x_pos[v1]));
    set_ram_word!(game.eproj_y_pos[v2] = ram_word!(game, eproj_y_pos[v1]));
}
unsafe fn EprojInit_TourianStatueSoul(j: u16, game: &mut Game) {
    let v1: i32 = ram_word!(game, cinematic_var8) as i32 >> 1i32;
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v2] = kEprojInit_TourianStatueEyeGlow_X[v1 as usize]);
    set_ram_word!(game.eproj_y_pos[v2] = kEprojInit_TourianStatueEyeGlow_Y[v1 as usize]);
    set_ram_word!(game.eproj_y_vel[v2] = -1024i32 as u16);
}
unsafe fn EprojInit_TourianStatueBaseDecoration(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_E[v1] = 120u16);
    set_ram_word!(game.eproj_x_pos[v1] = 120u16);
    set_ram_word!(game.eproj_F[v1] = 184u16);
    set_ram_word!(game.eproj_y_pos[v1] = 184u16);
}
unsafe fn EprojInit_TourianStatueRidley(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_E[v1] = 142u16);
    set_ram_word!(game.eproj_x_pos[v1] = 142u16);
    set_ram_word!(game.eproj_F[v1] = 85u16);
    set_ram_word!(game.eproj_y_pos[v1] = 85u16);
}
unsafe fn EprojInit_TourianStatuePhantoon(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_E[v1] = 132u16);
    set_ram_word!(game.eproj_x_pos[v1] = 132u16);
    set_ram_word!(game.eproj_F[v1] = 136u16);
    set_ram_word!(game.eproj_y_pos[v1] = 136u16);
}
pub unsafe fn EprojPreInstr_TourianStatueUnlockingParticleWaterSplash(k: u16, game: &mut Game) {
    set_ram_word!(
        game.eproj_y_pos[(k as i32 >> 1i32)] = (ram_word!(game, fx_y_pos) as i32 - 4i32) as u16
    );
}
pub unsafe fn EprojPreInstr_TourianStatueUnlockingParticle(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let mut amt: i32 = (ram_word!(game, eproj_x_vel[v1]) as i16 as i32) << 8i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16).offset(v1 as isize)
            as *mut u16,
        amt as u32,
    );
    let v7: u16 =
        (ram_word!(game, fx_y_pos) as i32 - ram_word!(game, eproj_y_pos[v1]) as i32) as u16;
    amt = (ram_word!(game, eproj_y_vel[v1]) as i16 as i32) << 8i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16).offset(v1 as isize)
            as *mut u16,
        amt as u32,
    );
    if ((v7 as i32 ^ ram_word!(game, fx_y_pos) as i32 - ram_word!(game, eproj_y_pos[v1]) as i32)
        & 0x8000i32)
        != 0i32
    {
        SpawnEprojWithRoomGfx(addr_stru_86BA5C as u16, k, game);
    }
    if (ram_word!(game, eproj_y_pos[v1]) as i32 & 0xff00i32) == 256i32 {
        set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_off_86B79F as u16);
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
    } else {
        update_ram_word!(game.eproj_y_vel[v1], |fresh97| (fresh97 as i32 + 16i32)
            as u16);
    };
}
pub unsafe fn EprojPreInstr_TourianStatueSoul(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let amt: i32 = (ram_word!(game, eproj_y_vel[v1]) as i16 as i32) << 8i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16).offset(v1 as isize)
            as *mut u16,
        amt as u32,
    );
    if (ram_word!(game, eproj_y_pos[v1]) as i32 & 0x100i32) != 0i32 {
        set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_off_86B79F as u16);
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
    }
    update_ram_word!(game.eproj_y_vel[v1], |fresh96| (fresh96 as i32 - 128i32)
        as u16);
}
unsafe fn EprojPreInstr_BA42(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v1] = ram_word!(game, eproj_E[v1]));
    set_ram_word!(
        game.eproj_y_pos[v1] = (ram_word!(game, eproj_F[v1]) as i32
            + ram_word!(game, layer1_y_pos) as i32
            - *(&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x9e00)
                as *mut ExpandingSquareTransitionHdma)
                .offset(0))
            .field_0 as *mut u16) as i32) as u16
    );
}
pub unsafe fn EprojPreInstr_TourianStatueStuff(k: u16, game: &mut Game) {
    if ram_word!(game, tourian_entrance_statue_animstate) == 0 {
        update_ram_word!(
            game.tourian_entrance_statue_finished,
            |fresh95| (fresh95 as i32 | 0x8000i32) as u16
        );
    }
    EprojPreInstr_BA42(k, game);
}
pub unsafe fn sub_86BB30(j: u16, game: &mut Game) {
    let v1: *mut ExtraEnemyRam8000 =
        gExtraEnemyRam8000(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v2] = *((&raw mut (*v1).pad as *mut u8).offset(34) as *mut u8 as *mut u16)
    );
    set_ram_word!(
        game.eproj_y_pos[v2] = *((&raw mut (*v1).pad as *mut u8).offset(36) as *mut u8 as *mut u16)
    );
    set_ram_word!(
        game.eproj_instr_list_ptr[v2] =
            crate::rtl::rom_table_word!(game, 0x86bb1e, ram_word!(game, cinematic_var8) as isize)
    );
}
unsafe fn EprojInit_NuclearWaffleBody(j: u16, game: &mut Game) {
    let v2: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let v3: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v3] = (*v2).x_pos);
    set_ram_word!(game.eproj_x_subpos[v3] = (*v2).x_subpos);
    set_ram_word!(game.eproj_y_pos[v3] = (*v2).y_pos);
    set_ram_word!(game.eproj_y_subpos[v3] = (*v2).y_subpos);
    let v4: *mut ExtraEnemyRam8000 =
        gExtraEnemyRam8000(ram_word!(game, cur_enemy_index) as i32, game);
    (*gExtraEnemyRam7800(
        ram_word!(game, cur_enemy_index) as i32
            + *((&raw mut (*v4).pad as *mut u8).offset(20) as *mut u8 as *mut u16) as i32,
        game,
    ))
    .kraid
    .kraid_next = j;
    set_ram_word!(game.eproj_flags[v3] = 1u16);
}
unsafe fn EprojInit_NorfairLavaquakeRocks(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_word_86BBD5 as u16);
    set_ram_word!(game.eproj_E[v1] = (0x86bc16i32 & 0xffffi32) as u16);
    set_ram_word!(game.eproj_y_vel[v1] = ram_word!(game, cinematic_var8));
    set_ram_word!(game.eproj_x_vel[v1] = ram_word!(game, cinematic_var9));
    let v2: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.eproj_x_pos[v1] = (*v2).x_pos);
    set_ram_word!(game.eproj_x_subpos[v1] = (*v2).x_subpos);
    set_ram_word!(game.eproj_y_pos[v1] = (*v2).y_pos);
    set_ram_word!(game.eproj_y_subpos[v1] = (*v2).y_subpos);
}
pub unsafe fn CallNorfairLavaquakeRocksFunc(ea: u32, k: u16, game: &mut Game) {
    match ea {
        8829974 => Eproj_NorfairLavaquakeRocks_Func1(k, game),
        8830095 => Eproj_NorfairLavaquakeRocks_Func2(k, game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn EprojPreInstr_NorfairLavaquakeRocks_Inner2(k: u16, game: &mut Game) -> u16 {
    let v1: i32 = k as i32 >> 1i32;
    (((ram_word!(game, eproj_x_pos[v1]) as i32 - ram_word!(game, layer1_x_pos) as i32) as i16)
        < 0i16
        || ((ram_word!(game, layer1_x_pos) as i32 + 256i32
            - ram_word!(game, eproj_x_pos[v1]) as i32) as i16)
            < 0i16
        || ((ram_word!(game, eproj_y_pos[v1]) as i32 - ram_word!(game, layer1_y_pos) as i32)
            as i16)
            < 0i16
        || ((ram_word!(game, layer1_y_pos) as i32 + 256i32
            - ram_word!(game, eproj_y_pos[v1]) as i32) as i16)
            < 0i16) as u16
}
pub unsafe fn EprojPreInstr_NorfairLavaquakeRocks_Inner(k: u16, game: &mut Game) {
    if EprojPreInstr_NorfairLavaquakeRocks_Inner2(k, game) != 0 {
        set_ram_word!(game.eproj_id[(k as i32 >> 1i32)] = 0u16);
    }
}
pub unsafe fn EprojPreInstr_NorfairLavaquakeRocks(k: u16, game: &mut Game) {
    CallNorfairLavaquakeRocksFunc(
        (ram_word!(game, eproj_E[(k as i32 >> 1i32)]) as i32 | 0x860000i32) as u32,
        k,
        game,
    );
    EprojPreInstr_NorfairLavaquakeRocks_Inner(k, game);
}
pub unsafe fn Eproj_NorfairLavaquakeRocks_Func1(k: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    let mut v3: i16 = 0;
    let v1: i32 = k as i32 >> 1i32;
    v2 = (ram_word!(game, eproj_y_vel[v1]) as i32 - 2i32) as i16;
    set_ram_word!(game.eproj_y_vel[v1] = v2 as u16);
    if v2 >= 0i16 {
        let mut n: i32 = 2i32;
        loop {
            v3 = (n + ram_word!(game, eproj_y_vel[v1]) as i32 - 1i32) as i16;
            if v3 < 0i16 {
                v3 = 0i16;
            }
            let t: i32 = 8i32 * v3 as i32 + 4i32 >> 1i32;
            AddToHiLo(
                ((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v1 as isize)
                    as *mut u16,
                ((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16).offset(v1 as isize)
                    as *mut u16,
                *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const u8 as *mut u32)
                    .offset((t >> 1i32) as isize),
            );
            set_ram_word!(
                game.eproj_F[v1] = crate::rtl::rom_table_word!(game, 0xa0cbc7, (t + 1i32) as isize)
            );
            n -= 1;
            if (n == 0) {
                break;
            }
        }
        Eproj_NorfairLavaquakeRocks_Func3(k, game);
    } else {
        set_ram_word!(game.eproj_y_vel[v1] = 0u16);
        set_ram_word!(game.eproj_E[v1] = (0x86bc8fi32 & 0xffffi32) as u16);
    };
}
pub unsafe fn Eproj_NorfairLavaquakeRocks_Func2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let v2: u16 = (ram_word!(game, eproj_y_vel[v1]) as i32 + 2i32) as u16;
    set_ram_word!(game.eproj_y_vel[v1] = v2);
    if v2 as i32 - 64i32 & 0x8000i32 == 0 {
        set_ram_word!(game.eproj_y_vel[v1] = 64u16);
    }
    let mut n: i32 = 2i32;
    loop {
        let t: i32 = 8i32 * (ram_word!(game, eproj_y_vel[v1]) as i32 - n + 1i32) >> 1i32;
        AddToHiLo(
            ((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v1 as isize)
                as *mut u16,
            ((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16).offset(v1 as isize)
                as *mut u16,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const u8 as *mut u32)
                .offset((t >> 1i32) as isize),
        );
        set_ram_word!(
            game.eproj_F[v1] = crate::rtl::rom_table_word!(game, 0xa0cbc7, (t + 1i32) as isize)
        );
        n -= 1;
        if (n == 0) {
            break;
        }
    }
    Eproj_NorfairLavaquakeRocks_Func3(k, game);
}
pub unsafe fn Eproj_NorfairLavaquakeRocks_Func3(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((ram_word!(game, eproj_x_vel[v1]) as i16 as i32) << 8i32) as u32,
    );
}
static kEprojInit_ShaktoolAttackMiddleBackCircle_X: [i16; 8] =
    [0i16, 12i16, 16i16, 12i16, 0i16, -12i16, -16i16, -12i16];
static kEprojInit_ShaktoolAttackMiddleBackCircle_Y: [i16; 8] =
    [-16i16, -12i16, 0i16, 12i16, 16i16, 12i16, 0i16, -12i16];
unsafe fn EprojInit_BDA2(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    let v2: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.eproj_x_pos[v1] = (*v2).x_pos);
    set_ram_word!(game.eproj_y_pos[v1] = (*v2).y_pos);
    let v3: u16 = (2i32 * *(&raw mut (*v2).ai_var_D as *mut u8).offset(0) as i32) as u16;
    let v4: i32 = v3 as i32 >> 1i32;
    set_ram_word!(game.eproj_x_vel[v1] = kSinCosTable8bit_Sext[(v4 + 64i32) as usize] as u16);
    set_ram_word!(game.eproj_y_vel[v1] = kSinCosTable8bit_Sext[v4 as usize] as u16);
    let v5: i32 = (v3 as i32 >> 5i32) as u16 as i32 >> 1i32;
    update_ram_word!(game.eproj_x_pos[v1], |fresh11| (fresh11 as i32
        + kEprojInit_ShaktoolAttackMiddleBackCircle_X[v5 as usize] as i32)
        as u16);
    update_ram_word!(game.eproj_y_pos[v1], |fresh12| (fresh12 as i32
        + kEprojInit_ShaktoolAttackMiddleBackCircle_Y[v5 as usize] as i32)
        as u16);
}
unsafe fn EprojInit_ShaktoolAttackMiddleBackCircle(j: u16, game: &mut Game) {
    set_ram_word!(game.eproj_E[(j as i32 >> 1i32)] = ram_word!(game, cinematic_var8));
    EprojInit_BDA2(j, game);
}
unsafe fn EprojInit_ShaktoolAttackFrontCircle(v1: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(v1, game) as i32 & 1i32 != 0
        || EprojBlockCollisition_Vertical(v1, game) as i32 & 1i32 != 0
    {
        set_ram_word!(game.eproj_id[(v1 as i32 >> 1i32)] = 0u16);
    }
}
pub unsafe fn EprojPreInstr_BE12(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if *((&raw mut game.g_ram as *mut u8).offset(0x1997) as *mut u16)
        .offset((ram_word!(game, eproj_E[v1]) as i32 >> 1i32) as isize)
        != 0
    {
        EprojBlockCollisition_Horiz(k, game);
        EprojBlockCollisition_Vertical(k, game);
    } else {
        set_ram_word!(game.eproj_id[v1] = 0u16);
    };
}
static kEprojInit_MotherBrainRoomTurrets_X: [i16; 12] = [
    0x398i16, 0x348i16, 0x328i16, 0x2d8i16, 0x288i16, 0x268i16, 0x218i16, 0x1c8i16, 0x1a8i16,
    0x158i16, 0x108i16, 0xe8i16,
];
static EprojInit_MotherBrainRoomTurrets_Y: [i16; 12] = [
    0x30i16, 0x40i16, 0x40i16, 0x30i16, 0x40i16, 0x40i16, 0x30i16, 0x40i16, 0x40i16, 0x30i16,
    0x40i16, 0x40i16,
];
unsafe fn EprojInit_MotherBrainRoomTurrets(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_gfx_idx[v1] = 1024u16);
    let v2: u16 = (2i32 * ram_word!(game, cinematic_var8) as i32) as u16;
    let v3: u16 =
        (crate::rtl::rom_table_word!(game, 0x86bee1, ram_word!(game, cinematic_var8) as isize)
            as i32
            | 0x100i32) as u16;
    set_ram_word!(game.eproj_y_subpos[v1] = v3);
    set_ram_word!(
        game.eproj_instr_list_ptr[v1] = crate::rtl::rom_table_word!(
            game,
            0x86beb9,
            ((2i32 * v3 as u8 as i32) as u16 as i32 >> 1i32) as isize,
        )
    );
    let v4: i32 = v2 as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v1] = kEprojInit_MotherBrainRoomTurrets_X[v4 as usize] as u16);
    set_ram_word!(game.eproj_y_pos[v1] = EprojInit_MotherBrainRoomTurrets_Y[v4 as usize] as u16);
    set_ram_word!(
        game.eproj_x_subpos[v1] = crate::rtl::rom_table_word!(game, 0x86bec9, v4 as isize)
    );
    Eproj_SetXvelRandom(j, game);
    Eproj_SetYvelRandom(j, game);
}
static kEproj_MotherBrainRoomTurretBullets_X: [i16; 8] =
    [-17i16, -12i16, 0i16, 12i16, 17i16, 12i16, 0i16, -12i16];
static kEproj_MotherBrainRoomTurretBullets_Y: [i16; 8] =
    [-9i16, 3i16, 7i16, 3i16, -9i16, -19i16, -21i16, -19i16];
static kEproj_MotherBrainRoomTurretBullets_Xvel: [i16; 8] = [
    -704i16, -498i16, 0i16, 498i16, 704i16, 498i16, 0i16, -498i16,
];
static kEproj_MotherBrainRoomTurretBullets_Yvel: [i16; 8] = [
    0i16, 498i16, 704i16, 498i16, 0i16, -498i16, -704i16, -498i16,
];
unsafe fn EprojInit_MotherBrainRoomTurretBullets(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_F[v1] = 0u16);
    set_ram_word!(game.eproj_gfx_idx[v1] = 1024u16);
    let v2: u16 = (2i32
        * *(((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16)
            .offset((ram_word!(game, cinematic_var8) as i32 >> 1i32) as isize)
            as *mut u16 as *mut u8)
            .offset(0) as i32) as u16;
    set_ram_word!(game.eproj_E[v1] = v2);
    let v3: i32 = v2 as i32 >> 1i32;
    let pt: Point16U = Point16U {
        x: kEproj_MotherBrainRoomTurretBullets_X[v3 as usize] as u16,
        y: kEproj_MotherBrainRoomTurretBullets_Y[v3 as usize] as u16,
    };
    set_ram_word!(
        game.eproj_x_vel[v1] = kEproj_MotherBrainRoomTurretBullets_Xvel[v3 as usize] as u16
    );
    set_ram_word!(
        game.eproj_y_vel[v1] = kEproj_MotherBrainRoomTurretBullets_Yvel[v3 as usize] as u16
    );
    let v4: i32 = ram_word!(game, cinematic_var8) as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v1] = (pt.x as i32 + ram_word!(game, eproj_x_pos[v4]) as i32) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v1] = (pt.y as i32 + ram_word!(game, eproj_y_pos[v4]) as i32) as u16
    );
}
pub unsafe fn EprojPreInstr_MotherBrainRoomTurrets(k: u16, game: &mut Game) {
    if Eproj_MotherBrainRoomTurretBullets_CheckIfTurretOnScreen(k, game) as i32 & 1i32 != 0 {
        if (*gRam7800_Default(0u16, game)).var_1D != 0 {
            *(((&raw mut game.g_ram as *mut u8).offset(0x1997) as *mut u16 as *mut u8)
                .offset(k as i32 as isize) as *mut u16) = 0u16;
        }
    } else if (*gRam7800_Default(0u16, game)).var_1D != 0 {
        *(((&raw mut game.g_ram as *mut u8).offset(0x1997) as *mut u16 as *mut u8)
            .offset(k as i32 as isize) as *mut u16) = 0u16;
        let v5: i32 = k as i32 >> 1i32;
        game.eproj_spawn_pt = Point16U {
            x: ram_word!(game, eproj_x_pos[v5]),
            y: ram_word!(game, eproj_y_pos[v5]),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0xcu16, game);
    } else {
        let v1: i32 = k as i32 >> 1i32;
        let ref mut fresh88 =
            *((&raw mut game.g_ram as *mut u8).offset(0x1ab7) as *mut u16).offset(v1 as isize);
        let fresh89 = *fresh88;
        *fresh88 = (*fresh88).wrapping_sub(1);
        let mut v2: bool = fresh89 == 1u16;
        if v2 {
            Eproj_SetXvelRandom(k, game);
            Eproj_MotherBrainRoomTurretBullets_Func2(k, game);
            let v3: i32 = k as i32 >> 1i32;
            set_ram_word!(
                game.eproj_instr_list_ptr[v3] = *(game
                    .g_rom
                    .offset(crate::rtl::lorom_offset(0x86c040))
                    as *const u8 as *mut u16)
                    .offset(
                        *(((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16)
                            .offset(v3 as isize) as *mut u16 as *mut u8)
                            .offset(0) as isize,
                    )
            );
            set_ram_word!(game.eproj_instr_timers[v3] = 1u16);
        }
        let v4: i32 = k as i32 >> 1i32;
        let ref mut fresh92 =
            *((&raw mut game.g_ram as *mut u8).offset(0x1adb) as *mut u16).offset(v4 as isize);
        let fresh93 = *fresh92;
        *fresh92 = (*fresh92).wrapping_sub(1);
        v2 = fresh93 == 1u16;
        if v2 {
            Eproj_SetYvelRandom(k, game);
            SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainRoomTurretBullets as u16, k, game);
        }
    };
}
pub unsafe fn Eproj_MotherBrainRoomTurretBullets_Func2(v0: u16, game: &mut Game) {
    let r20: u8 = (*((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16 as *mut u8)
        .offset(v0 as i32 as isize)
        .offset(1) as i32
        + *((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16 as *mut u8)
            .offset(v0 as i32 as isize) as i32
        & 7i32) as u8;
    let r18: u16 = ram_word!(game, eproj_x_subpos[(v0 as i32 >> 1i32)]);
    if *RomPtr_86(r18, game).offset((r20 as i32 & 7i32) as isize) != 0 {
        *((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16 as *mut u8)
            .offset(v0 as i32 as isize) = r20;
    } else {
        let v1: i8 = -(*((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16 as *mut u8)
            .offset(v0 as i32 as isize)
            .offset(1) as i32) as i8;
        *((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16 as *mut u8)
            .offset(v0 as i32 as isize)
            .offset(1) = v1 as u8;
        let ref mut fresh94 = *((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16
            as *mut u8)
            .offset(v0 as i32 as isize);
        *fresh94 = (*fresh94 as i32 + v1 as i32) as u8;
    };
}
pub unsafe fn Eproj_SetXvelRandom(v0: u16, game: &mut Game) {
    let mut Random: u16 = NextRandom(game) as u8 as u16;
    if Random as u8 as i32 - 32i32 & 0x8000i32 != 0 {
        Random = 32u16;
    }
    set_ram_word!(game.eproj_x_vel[(v0 as i32 >> 1i32)] = Random);
}
pub unsafe fn Eproj_SetYvelRandom(v0: u16, game: &mut Game) {
    let mut Random: u16 = NextRandom(game) as u8 as u16;
    if Random as u8 as i32 - 128i32 & 0x8000i32 != 0 {
        Random = 128u16;
    }
    set_ram_word!(game.eproj_y_vel[(v0 as i32 >> 1i32)] = Random);
}
pub unsafe fn Eproj_MotherBrainRoomTurretBullets_CheckIfTurretOnScreen(
    k: u16,
    game: &mut Game,
) -> u8 {
    let mut v2: i16 = 0;
    let mut v3: i16 = 0;
    let mut v4: i16 = 0;
    let mut v5: i16 = 0;
    let v1: i32 = k as i32 >> 1i32;
    v2 = ram_word!(game, eproj_y_pos[v1]) as i16;
    let result: u8 = 1u8;
    if v2 >= 0i16 {
        v3 = (v2 as i32 + 16i32 - ram_word!(game, layer1_y_pos) as i32) as i16;
        if v3 >= 0i16 && v3 as i32 - 256i32 & 0x8000i32 != 0 {
            v4 = ram_word!(game, eproj_x_pos[v1]) as i16;
            if v4 >= 0i16 {
                v5 = (v4 as i32 + 4i32 - ram_word!(game, layer1_x_pos) as i32) as i16;
                if v5 >= 0i16 && v5 as i32 - 264i32 & 0x8000i32 != 0 {
                    return 0u8;
                }
            }
        }
    }
    result
}
pub unsafe fn EprojPreInstr_MotherBrainRoomTurretBullets(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    update_ram_word!(
        game.eproj_properties[v1],
        |fresh85| (fresh85 as i32 ^ 0x8000i32) as u16
    );
    MoveEprojWithVelocity(k, game);
    if Ridley_Func_103(
        ram_word!(game, eproj_x_pos[v1]),
        ram_word!(game, eproj_y_pos[v1]),
        game,
    ) as i32
        & 1i32
        != 0
    {
        *((&raw mut game.g_ram as *mut u8).offset(0x1997) as *mut u16)
            .offset((ram_word!(game, cinematic_var7) as i32 >> 1i32) as isize) = 0u16;
    }
}
static kMotherBrainsBomb_Yaccel: [i16; 10] = [
    7i16, 0x10i16, 0x20i16, 0x40i16, 0x70i16, 0xb0i16, 0xf0i16, 0x130i16, 0x170i16, 0i16,
];
pub unsafe fn EprojPreInstr_MotherBrainBomb(k: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    if MotherBrainBomb_Bomb_CollDetect_DoubleRet(k, game) != 0 {
        return;
    }
    let v1: i32 = k as i32 >> 1i32;
    let current_block_19: u64;
    if ram_word!(game, eproj_F[v1]) != 0 {
        let v4: i32 = k as i32 >> 1i32;
        let v5: u16 =
            kMotherBrainsBomb_Yaccel[(ram_word!(game, eproj_F[v4]) as i32 >> 1i32) as usize] as u16;
        if v5 == 0 {
            set_ram_word!(game.eproj_x_vel[v4] = 0u16);
            set_ram_word!(game.eproj_y_vel[v4] = 0u16);
            let fresh81 = &raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x7800)
                as *mut ExtraEnemyRam7800)
                .offset(1))
            .kraid
            .kraid_mouth_flags;
            fresh81.write_unaligned(fresh81.read_unaligned().wrapping_sub(1));
            set_ram_word!(game.eproj_id[v4] = 0u16);
            game.eproj_spawn_pt = Point16U {
                x: ram_word!(game, eproj_x_pos[v4]),
                y: ram_word!(game, eproj_y_pos[v4]),
            };
            SpawnEprojWithRoomGfx(
                addr_stru_869650 as u16,
                *(((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16).offset(v4 as isize)
                    as *mut u16 as *mut u8)
                    .offset(0) as u16,
                game,
            );
            game.eproj_spawn_pt = Point16U {
                x: ram_word!(game, eproj_x_pos[v4]),
                y: ram_word!(game, eproj_y_pos[v4]),
            };
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 3u16, game);
            QueueSfx3_Max6(0x13u16, game);
            return;
        }
        if MoveMotherBrainBomb(k, v5, game) as i32 & 1i32 != 0 {
            current_block_19 = 15976848397966268834;
        } else {
            current_block_19 = 7175849428784450219;
        }
    } else {
        v2 = (abs16(ram_word!(game, eproj_x_vel[v1])) as i32 - 2i32) as i16;
        if v2 < 0i16 {
            v2 = 0i16;
        }
        set_ram_word!(
            game.eproj_x_vel[v1] = (if ram_word!(game, eproj_x_vel[v1]) as i32 & 0x8000i32 != 0 {
                -(v2 as i32)
            } else {
                v2 as i32
            }) as u16
        );
        if MoveMotherBrainBomb(k, 7u16, game) as i32 & 1i32 != 0 {
            current_block_19 = 15976848397966268834;
        } else {
            current_block_19 = 7175849428784450219;
        }
    }
    if current_block_19 == 15976848397966268834 {
        let v3: i32 = k as i32 >> 1i32;
        update_ram_word!(game.eproj_F[v3], |fresh82| (fresh82 as i32 + 2i32) as u16);
    };
}
pub unsafe fn EprojInstr_SwitchJump(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    *(epjp.offset(ram_word!(game, eproj_E[(k as i32 >> 1i32)]) as i32 as isize) as *mut u16)
        as usize as *const u8
}
pub unsafe fn EprojInstr_UserPalette0(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.eproj_gfx_idx[(k as i32 >> 1i32)] = 0u16);
    epjp
}
unsafe fn Eproj_CheckForBombCollisionWithRect(rect: Rect16U, game: &mut Game) -> u8 {
    if ram_word!(game, bomb_counter) == 0 {
        return 0u8;
    }
    let mut v1: i32 = 10i32;
    while v1 < 20i32 {
        let v2: i32 = v1 >> 1i32;
        if (ram_word!(game, projectile_type[v2]) as i32 & 0xf00i32) == 1280i32
            && ram_word!(game, projectile_variables[v2]) == 0
        {
            let x: u16 =
                abs16((ram_word!(game, projectile_x_pos[v2]) as i32 - rect.x as i32) as u16);
            if x < ram_word!(game, projectile_x_radius[v2])
                || ((x as i32 - ram_word!(game, projectile_x_radius[v2]) as i32) as u16) < rect.w
            {
                let y: u16 =
                    abs16((ram_word!(game, projectile_y_pos[v2]) as i32 - rect.y as i32) as u16);
                if y < ram_word!(game, projectile_y_radius[v2])
                    || ((y as i32 - ram_word!(game, projectile_y_radius[v2]) as i32) as u16)
                        < rect.h
                {
                    return 1u8;
                }
            }
        }
        v1 += 2i32;
    }
    0u8
}
unsafe fn Eproj_CheckForEnemyCollisionWithRect(k: u16, rect: Rect16U, game: &mut Game) -> u8 {
    let v1: *mut EnemyData = gEnemyData(k as i32, game);
    ((abs16(((*v1).x_pos as i32 - rect.x as i32) as u16) as i32 - (*v1).x_width as i32)
        < rect.w as i32
        && (abs16(((*v1).y_pos as i32 - rect.y as i32) as u16) as i32 - (*v1).y_height as i32)
            < rect.h as i32) as u8
}
unsafe fn Eproj_CheckForCollisionWithSamus(v0: u16, game: &mut Game) -> u8 {
    let rect: Rect16U = Eproj_GetCollDetectRect(v0, game);
    let v1: u16 = abs16((ram_word!(game, samus_x_pos) as i32 - rect.x as i32) as u16);
    let mut v2: bool = v1 < ram_word!(game, samus_x_radius);
    let v3: u16 = (v1 as i32 - ram_word!(game, samus_x_radius) as i32) as u16;
    let result: u8 = 0u8;
    if v2 || v3 < rect.w {
        let v4: u16 = abs16((ram_word!(game, samus_y_pos) as i32 - rect.y as i32) as u16);
        v2 = v4 < ram_word!(game, samus_y_radius);
        let v5: u16 = (v4 as i32 - ram_word!(game, samus_y_radius) as i32) as u16;
        if v2 || v5 < rect.h {
            return 1u8;
        }
    }
    result
}
pub unsafe fn Math_MultBySin(a: u16, r18: u16) -> u16 {
    Math_MultBySinCos(a, r18)
}
pub unsafe fn Math_MultByCos(a: u16, r18: u16) -> u16 {
    Math_MultBySinCos(a, r18.wrapping_add(64))
}
pub unsafe fn Math_MultBySinCos(r38: u16, a: u16) -> u16 {
    let r46: u16 = kSinCosTable8bit_Sext[((a as i32 & 0xffi32) + 64i32) as usize] as u16;
    let prod: u16 = ((r38 as u32).wrapping_mul(abs16(r46) as u32) >> 8i32) as u16;
    ((if r46 as i32 & 0x8000i32 != 0 {
        -(prod as i32)
    } else {
        prod as i32
    }) as u16)
}
pub unsafe fn Eproj_MotherBrainsBlueRingLasers(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_E[v1] = 8u16);
    set_ram_word!(game.eproj_F[v1] = 0u16);
    set_ram_word!(game.eproj_gfx_idx[v1] = 1024u16);
    let r18: u16 = ram_word!(game, cinematic_var8);
    set_ram_word!(game.eproj_x_vel[v1] = Math_MultBySin(0x450u16, r18));
    set_ram_word!(game.eproj_y_vel[v1] = Math_MultByCos(0x450u16, r18));
    set_ram_word!(
        game.eproj_x_pos[v1] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1)).x_pos
                as i32
                + 10i32) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v1] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1)).y_pos
                as i32
                + 16i32) as u16
    );
    sub_86C320(j, game);
}
pub unsafe fn sub_86C320(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v1] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1)).x_pos
                as i32
                + 10i32) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v1] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1)).y_pos
                as i32
                + 16i32) as u16
    );
}
pub unsafe fn Eproj_MoveToBlueRingSpawnPosition(k: u16, game: &mut Game) {
    let mut v5: i16 = 0;
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, eproj_E[v1]) != 0 {
        update_ram_word!(game.eproj_E[v1], wrapping_sub(1));
        sub_86C320(k, game);
    } else {
        MoveEprojWithVelocity(k, game);
        let t: u8 = CheckForCollisionWithShitroid_DoubleRet(k, game);
        if t as i32 & 0x80i32 != 0 {
            return;
        }
        if t != 0 {
            let fresh84 = &raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x7800)
                as *mut ExtraEnemyRam7800)
                .offset(0))
            .kraid
            .field_28;
            fresh84.write_unaligned(fresh84.read_unaligned().wrapping_add(1));
            BlueRingContactExplosion(k, game);
            let v3: u16 = (*((&raw mut game.g_ram as *mut u8).offset(0x7800)
                as *mut ExtraEnemyRam7800)
                .offset(1))
            .kraid
            .kraid_healths_8ths[4i32 as usize];
            (*gExtraEnemyRam7800(
                (*((&raw mut game.g_ram as *mut u8).offset(0x7800) as *mut ExtraEnemyRam7800)
                    .offset(1))
                .kraid
                .kraid_healths_8ths[4i32 as usize] as i32,
                game,
            ))
            .kraid
            .kraid_healths_8ths[0i32 as usize] = 16u16;
            let v4: *mut EnemyData = gEnemyData(v3 as i32, game);
            v5 = ((*v4).health as i32 - 80i32) as i16;
            if v5 < 0i16 {
                v5 = 0i16;
            }
            (*v4).health = v5 as u16;
        } else if Eproj_CheckForCollisionWithSamus(k, game) as i32 & 1i32 != 0 {
            BlueRingContactExplosion(k, game);
            Samus_DealDamage(SuitDamageDivision(0x50u16, game), game);
            set_ram_word!(game.samus_invincibility_timer = 96u16);
            set_ram_word!(game.samus_knockback_timer = 5u16);
            set_ram_word!(
                game.knockback_x_dir = (((ram_word!(game, samus_x_pos) as i32
                    - ram_word!(game, eproj_x_pos[(k as i32 >> 1i32)]) as i32)
                    as i16)
                    >= 0i16) as u16
            );
        } else if CheckForBlueRingCollisionWithRoom(k, game) as i32 & 1i32 != 0 {
            Eproj_Earthqhake5(k, game);
        }
    };
}
pub unsafe fn CheckForCollisionWithShitroid_DoubleRet(v0: u16, game: &mut Game) -> u8 {
    if (*((&raw mut game.g_ram as *mut u8).offset(0x7800) as *mut ExtraEnemyRam7800).offset(1))
        .kraid
        .kraid_healths_8ths[4i32 as usize]
        == 0
    {
        return 0u8;
    }
    if (*gEnemyData(
        (*((&raw mut game.g_ram as *mut u8).offset(0x7800) as *mut ExtraEnemyRam7800).offset(1))
            .kraid
            .kraid_healths_8ths[4i32 as usize] as i32,
        game,
    ))
    .health
        != 0
    {
        let rect: Rect16U = Eproj_GetCollDetectRect(v0, game);
        return Eproj_CheckForEnemyCollisionWithRect(
            (*((&raw mut game.g_ram as *mut u8).offset(0x7800) as *mut ExtraEnemyRam7800)
                .offset(1))
            .kraid
            .kraid_healths_8ths[4i32 as usize],
            rect,
            game,
        );
    }
    set_ram_word!(game.eproj_id[(v0 as i32 >> 1i32)] = 0u16);
    0xffu8
}
pub unsafe fn CheckForBlueRingCollisionWithRoom(k: u16, game: &mut Game) -> u8 {
    let mut v2: i16 = 0;
    let mut v3: i16 = 0;
    let v1: i32 = k as i32 >> 1i32;
    let result: u8 = 1u8;
    if ram_word!(game, eproj_y_pos[v1]) as i32 - 32i32 & 0x8000i32 == 0
        && ram_word!(game, eproj_y_pos[v1]) < 0xd8u16
    {
        v2 = ram_word!(game, eproj_x_pos[v1]) as i16;
        if v2 >= 0i16 {
            v3 = (v2 as i32 - ram_word!(game, layer1_x_pos) as i32) as i16;
            if v3 >= 0i16 && v3 as i32 - 248i32 & 0x8000i32 != 0 {
                return 0u8;
            }
        }
    }
    result
}
unsafe fn Eproj_GetCollDetectRect(k: u16, game: &mut Game) -> Rect16U {
    let v1: i32 = k as i32 >> 1i32;
    let rect: Rect16U = Rect16U {
        x: ram_word!(game, eproj_x_pos[v1]),
        y: ram_word!(game, eproj_y_pos[v1]),
        w: *(((&raw mut game.g_ram as *mut u8).offset(0x1bb3) as *mut u16).offset(v1 as isize)
            as *mut u16 as *mut u8)
            .offset(0) as u16,
        h: *(((&raw mut game.g_ram as *mut u8).offset(0x1bb3) as *mut u16).offset(v1 as isize)
            as *mut u16 as *mut u8)
            .offset(1isize) as u16,
    };
    rect
}
pub unsafe fn Eproj_Earthqhake5(k: u16, game: &mut Game) {
    set_ram_word!(game.earthquake_timer = 10u16);
    set_ram_word!(game.earthquake_type = 5u16);
    BlueRingContactExplosion(k, game);
}
pub unsafe fn BlueRingContactExplosion(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    set_ram_word!(game.eproj_id[v1] = 0u16);
    game.eproj_spawn_pt = Point16U {
        x: ram_word!(game, eproj_x_pos[v1]),
        y: ram_word!(game, eproj_y_pos[v1]),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 3u16, game);
    QueueSfx3_Max6(0x13u16, game);
    sub_86C42E(k, game);
}
pub unsafe fn sub_86C42E(k: u16, game: &mut Game) {
    set_ram_word!(game.eproj_gfx_idx[(k as i32 >> 1i32)] = 0u16);
}
unsafe fn EprojInit_MotherBrainBomb(j: u16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16 as *mut u8)
        .offset(j as i32 as isize) = ram_word!(game, cinematic_var8) as u8;
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_y_vel[v1] = 256u16);
    set_ram_word!(game.eproj_x_vel[v1] = 224u16);
    set_ram_word!(
        game.eproj_x_pos[v1] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1)).x_pos
                as i32
                + 12i32) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v1] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1)).y_pos
                as i32
                + 16i32) as u16
    );
    set_ram_word!(game.eproj_gfx_idx[v1] = 1024u16);
    set_ram_word!(game.eproj_E[v1] = 112u16);
    set_ram_word!(game.eproj_F[v1] = 0u16);
    let fresh10 = &raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x7800)
        as *mut ExtraEnemyRam7800)
        .offset(1))
    .kraid
    .kraid_mouth_flags;
    fresh10.write_unaligned(fresh10.read_unaligned().wrapping_add(1));
}
pub unsafe fn MotherBrainBomb_Bomb_CollDetect_DoubleRet(k: u16, game: &mut Game) -> u8 {
    let v1: i32 = k as i32 >> 1i32;
    let rect: Rect16U = Rect16U {
        x: ram_word!(game, eproj_x_pos[v1]),
        y: ram_word!(game, eproj_y_pos[v1]),
        w: *(((&raw mut game.g_ram as *mut u8).offset(0x1bb3) as *mut u16).offset(v1 as isize)
            as *mut u16 as *mut u8)
            .offset(0) as u16,
        h: *(((&raw mut game.g_ram as *mut u8).offset(0x1bb3) as *mut u16).offset(v1 as isize)
            as *mut u16 as *mut u8)
            .offset(1isize) as u16,
    };
    if Eproj_CheckForBombCollisionWithRect(rect, game) == 0 {
        return 0u8;
    }
    let fresh2 = &raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x7800)
        as *mut ExtraEnemyRam7800)
        .offset(1))
    .kraid
    .kraid_mouth_flags;
    fresh2.write_unaligned(fresh2.read_unaligned().wrapping_sub(1));
    set_ram_word!(game.eproj_x_vel[v1] = 0u16);
    set_ram_word!(game.eproj_y_vel[v1] = 0u16);
    set_ram_word!(game.eproj_id[v1] = 0u16);
    game.eproj_spawn_pt = Point16U {
        x: ram_word!(game, eproj_x_pos[v1]),
        y: ram_word!(game, eproj_y_pos[v1]),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9u16, game);
    game.eproj_spawn_pt = Point16U {
        x: ram_word!(game, eproj_x_pos[v1]),
        y: ram_word!(game, eproj_y_pos[v1]),
    };
    SpawnEnemyDrops(addr_kEnemyDef_EC3F as u16, k, 0u16, game);
    1u8
}
pub unsafe fn MoveMotherBrainBomb(k: u16, a: u16, game: &mut Game) -> u8 {
    let v2: i32 = k as i32 >> 1i32;
    update_ram_word!(game.eproj_y_vel[v2], |fresh16| (fresh16 as i32 + a as i32)
        as u16);
    MoveEprojWithVelocity(k, game);
    if ram_word!(game, eproj_x_pos[v2]) as i32 - 240i32 & 0x8000i32 == 0 {
        set_ram_word!(game.eproj_x_vel[v2] = -(ram_word!(game, eproj_x_vel[v2]) as i32) as u16);
    }
    if ram_word!(game, eproj_y_pos[v2]) as i32 - 208i32 & 0x8000i32 != 0 {
        return 0u8;
    }
    set_ram_word!(game.eproj_y_pos[v2] = 208u16);
    set_ram_word!(
        game.eproj_x_vel[v2] = (if ram_word!(game, eproj_x_vel[v2]) as i32 & 0x8000i32 != 0 {
            -(ram_word!(game, eproj_E[v2]) as i32)
        } else {
            ram_word!(game, eproj_E[v2]) as i32
        }) as u16
    );
    set_ram_word!(game.eproj_y_vel[v2] = -512i32 as u16);
    1u8
}
pub unsafe fn sub_86C605(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_E[v1] = 0u16);
    set_ram_word!(game.eproj_F[v1] = 0u16);
    set_ram_word!(game.eproj_x_vel[v1] = 0u16);
    set_ram_word!(game.eproj_y_vel[v1] = 0u16);
    set_ram_word!(game.eproj_x_subpos[v1] = 0u16);
    set_ram_word!(game.eproj_y_subpos[v1] = 0u16);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_23 = 0u16;
    (*E).mbn_var_25 = 0u16;
    set_ram_word!(game.eproj_gfx_idx[v1] = 1024u16);
    let ref mut fresh8 =
        *((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16).offset(v1 as isize);
    *fresh8 = ((*E).base.x_pos as i32 + 64i32) as u16;
    (*E).mbn_var_24 = *fresh8;
    let x: u16 = (ram_word!(game, samus_x_pos) as i32 - (*E).mbn_var_24 as i32) as u16;
    let ref mut fresh9 =
        *((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v1 as isize);
    *fresh9 = ((*E).base.y_pos as i32 - 48i32) as u16;
    (*E).mbn_var_26 = *fresh9;
    let y: u16 = (ram_word!(game, samus_y_pos) as i32 - (*E).mbn_var_26 as i32) as u16;
    let r18: i32 = -(CalculateAngleFromXY(x, y, game) as i32 + 0x80i32) as u8 as i32;
    (*E).mbn_var_29 = r18 as u16;
    (*E).mbn_var_27 = Math_MultBySin(0xc00u16, r18 as u16);
    (*E).mbn_var_28 = Math_MultByCos(0xc00u16, r18 as u16);
}
unsafe fn EprojInit_MotherBrainDeathBeemFired(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    set_ram_word!(game.eproj_x_pos[v1] = (*E).mbn_var_24);
    set_ram_word!(game.eproj_x_subpos[v1] = (*E).mbn_var_23);
    set_ram_word!(game.eproj_y_pos[v1] = (*E).mbn_var_26);
    set_ram_word!(game.eproj_y_subpos[v1] = (*E).mbn_var_25);
    set_ram_word!(game.eproj_x_vel[v1] = (*E).mbn_var_27);
    set_ram_word!(game.eproj_y_vel[v1] = (*E).mbn_var_28);
    MoveEprojWithVelocity(j, game);
    (*E).mbn_var_24 = ram_word!(game, eproj_x_pos[v1]);
    (*E).mbn_var_23 = ram_word!(game, eproj_x_subpos[v1]);
    (*E).mbn_var_26 = ram_word!(game, eproj_y_pos[v1]);
    (*E).mbn_var_25 = ram_word!(game, eproj_y_subpos[v1]);
    let r18: u16 = (*(&raw mut (*E).mbn_var_29 as *mut u8).offset(0) as i32
        + NextRandom(game) as i32) as u8 as u16;
    let rv: u16 = NextRandom(game);
    set_ram_word!(game.eproj_x_vel[v1] = Math_MultBySin((rv as i32 & 0x700i32) as u16, r18,));
    set_ram_word!(
        game.eproj_y_vel[v1] = Math_MultByCos(
            (ram_word!(game, random_number) as i32 & 0x700i32) as u16,
            r18,
        )
    );
    MoveEprojWithVelocity(j, game);
    if ram_word!(game, eproj_y_pos[v1]) as i32 - 34i32 & 0x8000i32 != 0
        || ram_word!(game, eproj_y_pos[v1]) as i32 - 206i32 & 0x8000i32 == 0
        || ram_word!(game, eproj_x_pos[v1]) as i32 - 2i32 & 0x8000i32 != 0
        || ram_word!(game, eproj_x_pos[v1]) as i32 - 238i32 & 0x8000i32 == 0
    {
        set_ram_word!(game.eproj_id[v1] = 0u16);
        game.eproj_spawn_pt = Point16U {
            x: ram_word!(game, eproj_x_pos[v1]),
            y: ram_word!(game, eproj_y_pos[v1]),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x1du16, game);
        QueueSfx3_Max6(0x13u16, game);
        set_ram_word!(game.earthquake_timer = 10u16);
        set_ram_word!(game.earthquake_type = 5u16);
    } else {
        set_ram_word!(
            game.eproj_E[v1] = (ram_word!(game, eproj_E[v1]) as i32 + 1i32 & 3i32) as u16
        );
        set_ram_word!(game.eproj_F[v1] = 0u16);
        set_ram_word!(game.eproj_x_vel[v1] = 0u16);
        set_ram_word!(game.eproj_y_vel[v1] = 0u16);
    };
}
pub unsafe fn SpawnMotherBrainDeathBeam(x: u16, game: &mut Game) {
    SpawnEprojWithRoomGfx(
        addr_kEproj_MotherBrainDeathBeamFired as u16,
        ram_word!(game, eproj_E[(x as i32 >> 1i32)]),
        game,
    );
}
pub unsafe fn EprojPreInstr_MotherBrainRainbowBeam(k: u16, game: &mut Game) {
    let v2: i32 = k as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v2] =
            (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1)).x_pos
    );
    set_ram_word!(
        game.eproj_y_pos[v2] =
            (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1)).y_pos
    );
}
unsafe fn EprojInit_MotherBrainRainbowBeam(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_gfx_idx[v1] = 0u16);
    set_ram_word!(game.eproj_x_vel[v1] = 0u16);
    set_ram_word!(game.eproj_y_vel[v1] = 0u16);
    EprojPreInstr_MotherBrainRainbowBeam(j, game);
}
static kEprojInit_MotherBrainsDrool: [i16; 12] = [
    6i16, 0x14i16, 0xei16, 0x12i16, 8i16, 0x17i16, 0xai16, 0x13i16, 0xbi16, 0x19i16, 0xci16,
    0x12i16,
];
pub unsafe fn EprojPreInstr_C84D(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let v2: i32 = (4i32 * ram_word!(game, eproj_E[v1]) as i32) as u16 as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v1] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1)).x_pos
                as i32
                + kEprojInit_MotherBrainsDrool[v2 as usize] as i32) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v1] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1)).y_pos
                as i32
                + kEprojInit_MotherBrainsDrool[(v2 + 1i32) as usize] as i32) as u16
    );
    set_ram_word!(game.eproj_x_vel[v1] = 0u16);
    set_ram_word!(game.eproj_y_vel[v1] = 0u16);
}
unsafe fn EprojInit_MotherBrainsDrool(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_gfx_idx[v1] = 0u16);
    set_ram_word!(game.eproj_E[v1] = ram_word!(game, cinematic_var8));
    EprojPreInstr_C84D(j, game);
}
unsafe fn EprojInit_MotherBrainsDrool_Falling(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    update_ram_word!(game.eproj_y_vel[v1], |fresh79| (fresh79 as i32 + 12i32)
        as u16);
    if MoveEprojWithVelocityY(k, game) >= 0xd7u16 {
        update_ram_word!(game.eproj_y_pos[v1], |fresh80| (fresh80 as i32 - 4i32)
            as u16);
        set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_off_86C8E1 as u16);
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
    }
}
pub unsafe fn EprojInstr_Add12ToY(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    update_ram_word!(
        game.eproj_y_pos[(k as i32 >> 1i32)],
        |fresh27| (fresh27 as i32 + 12i32) as u16
    );
    epjp
}
pub unsafe fn EprojPreInit_MotherBrainsDeathExplosion_0(k: u16, game: &mut Game) {
    let v2: i32 = k as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v2] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
                as i32
                + ram_word!(game, eproj_x_vel[v2]) as i32) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v2] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).y_pos
                as i32
                + ram_word!(game, eproj_y_vel[v2]) as i32) as u16
    );
}
unsafe fn EprojInit_MotherBrainsDeathExplosion(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_instr_list_ptr[v1] =
            crate::rtl::rom_table_word!(game, 0x86c929, ram_word!(game, cinematic_var8) as isize)
    );
    set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
    set_ram_word!(game.eproj_gfx_idx[v1] = 0u16);
    set_ram_word!(game.eproj_x_vel[v1] = game.eproj_spawn_pt.x);
    set_ram_word!(game.eproj_y_vel[v1] = game.eproj_spawn_pt.y);
    EprojPreInit_MotherBrainsDeathExplosion_0(j, game);
}
unsafe fn EprojInit_MotherBrainsRainbowBeamExplosion(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_gfx_idx[v1] = 0u16);
    set_ram_word!(game.eproj_x_vel[v1] = game.eproj_spawn_pt.x);
    set_ram_word!(
        game.eproj_x_pos[v1] =
            (ram_word!(game, samus_x_pos) as i32 + game.eproj_spawn_pt.x as i32) as u16
    );
    set_ram_word!(game.eproj_y_vel[v1] = game.eproj_spawn_pt.y);
    set_ram_word!(
        game.eproj_y_pos[v1] =
            (ram_word!(game, samus_y_pos) as i32 + game.eproj_spawn_pt.y as i32) as u16
    );
}
pub unsafe fn EprojPreInstr_MotherBrainsRainbowBeamExplosion(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v1] =
            (ram_word!(game, samus_x_pos) as i32 + ram_word!(game, eproj_x_vel[v1]) as i32) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v1] =
            (ram_word!(game, samus_y_pos) as i32 + ram_word!(game, eproj_y_vel[v1]) as i32) as u16
    );
}
unsafe fn EprojInit_MotherBrainEscapeDoorParticles(j: u16, game: &mut Game) {
    static kEprojInit_MotherBrainEscapeDoorParticles_X: [i16; 16] = [
        0i16, -0x20i16, 0i16, -0x18i16, 0i16, -0x10i16, 0i16, -8i16, 0i16, 0i16, 0i16, 8i16, 0i16,
        0x10i16, 0i16, 0x18i16,
    ];
    static kEprojInit_MotherBrainEscapeDoorParticles_Xvel: [i16; 16] = [
        0x500i16, -0x200i16, 0x500i16, -0x100i16, 0x500i16, -0x100i16, 0x500i16, -0x80i16,
        0x500i16, -0x80i16, 0x500i16, 0x80i16, 0x500i16, -0x100i16, 0x500i16, 0x200i16,
    ];
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_gfx_idx[v1] = 0u16);
    let v2: i32 = 4i32 * ram_word!(game, cinematic_var8) as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v1] =
            (kEprojInit_MotherBrainEscapeDoorParticles_X[v2 as usize] as i32 + 16i32) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v1] = (kEprojInit_MotherBrainEscapeDoorParticles_X[(v2 + 1i32) as usize]
            as i32
            + 128i32) as u16
    );
    set_ram_word!(
        game.eproj_x_vel[v1] = kEprojInit_MotherBrainEscapeDoorParticles_Xvel[v2 as usize] as u16
    );
    set_ram_word!(
        game.eproj_y_vel[v1] =
            kEprojInit_MotherBrainEscapeDoorParticles_Xvel[(v2 + 1i32) as usize] as u16
    );
    set_ram_word!(game.eproj_E[v1] = 32u16);
}
pub unsafe fn EprojPreInstr_MotherBrainsExplodedDoorParticles(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let mut v2: i16 = (abs16(ram_word!(game, eproj_x_vel[v1])) as i32 - 16i32) as i16;
    if v2 < 0i16 {
        v2 = 0i16;
    }
    set_ram_word!(
        game.eproj_x_vel[v1] = (if ram_word!(game, eproj_x_vel[v1]) as i32 & 0x8000i32 != 0 {
            -(v2 as i32)
        } else {
            v2 as i32
        }) as u16
    );
    update_ram_word!(game.eproj_y_vel[v1], |fresh76| (fresh76 as i32 + 32i32)
        as u16);
    MoveEprojWithVelocity(k, game);
    if (update_ram_word!(game.eproj_E[v1], wrapping_sub(1), reread) as i32 & 0x8000i32) != 0i32 {
        set_ram_word!(game.eproj_id[v1] = 0u16);
        update_ram_word!(game.eproj_y_pos[v1], |fresh78| (fresh78 as i32 - 4i32)
            as u16);
        game.eproj_spawn_pt = Point16U {
            x: ram_word!(game, eproj_x_pos[v1]),
            y: ram_word!(game, eproj_y_pos[v1]),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9u16, game);
    }
}
unsafe fn EprojInit_MotherBrainPurpleBreathBig(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_gfx_idx[v1] = 0u16);
    set_ram_word!(
        game.eproj_x_pos[v1] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1)).x_pos
                as i32
                + 6i32) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v1] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1)).y_pos
                as i32
                + 16i32) as u16
    );
}
unsafe fn EprojInit_MotherBrainPurpleBreathSmall(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_gfx_idx[v1] = 0u16);
    set_ram_word!(
        game.eproj_x_pos[v1] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1)).x_pos
                as i32
                + 6i32) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v1] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(1)).y_pos
                as i32
                + 16i32) as u16
    );
    (*((&raw mut game.g_ram as *mut u8).offset(0x7800) as *mut ExtraEnemyRam7800).offset(1))
        .kraid
        .kraid_hurt_frame = 1u16;
}
pub unsafe fn EprojInstr_MotherBrainPurpleBreathIsActive(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    (*((&raw mut game.g_ram as *mut u8).offset(0x7800) as *mut ExtraEnemyRam7800).offset(1))
        .kraid
        .kraid_hurt_frame = 0u16;
    epjp
}
pub unsafe fn EprojPreInstr_TimeBombSetJapaneseText(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    set_ram_word!(game.eproj_x_vel[v1] = 0u16);
    set_ram_word!(game.eproj_y_vel[v1] = 0u16);
    set_ram_word!(game.eproj_x_pos[v1] = 128u16);
    set_ram_word!(game.eproj_y_pos[v1] = 192u16);
}
unsafe fn EprojInit_TimeBombSetJapaneseText(j: u16, game: &mut Game) {
    set_ram_word!(game.eproj_gfx_idx[(j as i32 >> 1i32)] = 0u16);
    EprojPreInstr_TimeBombSetJapaneseText(j, game);
}
unsafe fn EprojInit_MotherBrainTubeFalling(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_gfx_idx[v1] = 3584u16);
    set_ram_word!(game.eproj_x_vel[v1] = 0u16);
    set_ram_word!(game.eproj_y_vel[v1] = 0u16);
    set_ram_word!(game.eproj_x_pos[v1] = game.eproj_spawn_pt.x);
    set_ram_word!(game.eproj_y_pos[v1] = game.eproj_spawn_pt.y);
    set_ram_word!(game.eproj_E[v1] = (0x86cbeai32 & 0xffffi32) as u16);
}
pub unsafe fn CallMotherBrainTubeFallingFunc(ea: u32, k: u16, game: &mut Game) {
    match ea {
        8834026 => MotherBrainTubeFallingFunc_GenerateExplosion(k, game),
        8834056 => MotherBrainTubeFallingFunc_Falling(k, game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn EprojPreInstr_MotherBrainTubeFalling(k: u16, game: &mut Game) {
    CallMotherBrainTubeFallingFunc(
        (ram_word!(game, eproj_E[(k as i32 >> 1i32)]) as i32 | 0x860000i32) as u32,
        k,
        game,
    );
}
pub unsafe fn MotherBrainTubeFallingFunc_GenerateExplosion(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    game.eproj_spawn_pt = Point16U {
        x: ram_word!(game, eproj_x_pos[v1]),
        y: (ram_word!(game, eproj_y_pos[v1]) as i32 + 8i32) as u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9u16, game);
    set_ram_word!(game.eproj_E[v1] = (0x86cc08i32 & 0xffffi32) as u16);
    MotherBrainTubeFallingFunc_Falling(k, game);
}
pub unsafe fn MotherBrainTubeFallingFunc_Falling(k: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    let v1: i32 = k as i32 >> 1i32;
    update_ram_word!(game.eproj_y_vel[v1], |fresh75| (fresh75 as i32 + 6i32)
        as u16);
    v2 = MoveEprojWithVelocityY(k, game) as i16;
    if v2 as i32 - 208i32 & 0x8000i32 == 0 {
        set_ram_word!(game.eproj_id[v1] = 0u16);
        game.eproj_spawn_pt = Point16U {
            x: ram_word!(game, eproj_x_pos[v1]),
            y: ram_word!(game, eproj_y_pos[v1]),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0xcu16, game);
    }
}
static kEprojInit_MotherBrainGlassShatteringShard_X: [i16; 3] = [8i16, -40i16, -16i16];
static kEprojInit_MotherBrainGlassShatteringShard_Y: [i16; 3] = [32i16, 32i16, 32i16];
unsafe fn EprojInit_MotherBrainGlassShatteringShard(j: u16, game: &mut Game) {
    let v1: u16 = (2i32 * NextRandom(game) as i32 & 0x1fei32) as u16;
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_E[v2] = v1);
    let v3: i32 = v1 as i32 >> 1i32;
    set_ram_word!(game.eproj_x_vel[v2] = kSinCosTable8bit_Sext[(v3 + 64i32) as usize] as u16);
    set_ram_word!(game.eproj_y_vel[v2] = (4i32 * kSinCosTable8bit_Sext[v3 as usize] as i32) as u16);
    set_ram_word!(
        game.eproj_instr_list_ptr[v2] = crate::rtl::rom_table_word!(
            game,
            0x86ce41,
            ((v1 as i32 >> 4i32 & 0x1ei32) as u16 as i32 >> 1i32) as isize,
        )
    );
    set_ram_word!(game.eproj_gfx_idx[v2] = 1600u16);
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    let v4: i32 = ram_word!(game, cinematic_var8) as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v2] = (kEprojInit_MotherBrainGlassShatteringShard_X[v4 as usize] as i32
            + 16i32 * ram_word!(game, plm_x_block) as i32) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v2] = (kEprojInit_MotherBrainGlassShatteringShard_Y[v4 as usize] as i32
            + 16i32 * ram_word!(game, plm_y_block) as i32) as u16
    );
    update_ram_word!(game.eproj_x_pos[v2], |fresh5| (fresh5 as i32
        + ((NextRandom(game) as i32 & 0xfi32) - 8i32))
        as u16);
    update_ram_word!(game.eproj_y_pos[v2], |fresh6| (fresh6 as i32
        + ((NextRandom(game) as i32 & 0xfi32) - 8i32))
        as u16);
}
unsafe fn EprojInit_MotherBrainGlassShatteringSparkle(j: u16, game: &mut Game) {
    let v1: i32 = ram_word!(game, cinematic_var8) as i32 >> 1i32;
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v2] = (ram_word!(game, eproj_x_pos[v1]) as i32
            + (NextRandom(game) as i32 & 0x1fi32)
            - 16i32) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v2] = (ram_word!(game, eproj_y_pos[v1]) as i32
            + (NextRandom(game) as i32 & 0x1fi32)
            - 16i32) as u16
    );
    set_ram_word!(game.eproj_gfx_idx[v2] = 1600u16);
}
pub unsafe fn EprojPreInstr_MotherBrainGlassShatteringShard(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let mut amt: i32 = (ram_word!(game, eproj_x_vel[v1]) as i16 as i32) << 8i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16).offset(v1 as isize)
            as *mut u16,
        amt as u32,
    );
    amt = (ram_word!(game, eproj_y_vel[v1]) as i16 as i32) << 8i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16).offset(v1 as isize)
            as *mut u16,
        amt as u32,
    );
    let v6: u16 = ram_word!(game, eproj_y_subpos[v1]);
    if (ram_word!(game, eproj_y_pos[v1]) as i32 & 0xff00i32) != 0i32 {
        set_ram_word!(game.eproj_id[v1] = 0u16);
    } else {
        update_ram_word!(game.eproj_y_vel[v1], |fresh74| (fresh74 as i32 + 32i32)
            as u16);
        if (NextRandom(game) as i32 & 0x420i32) == 0i32 {
            SpawnEprojWithRoomGfx(
                addr_kEproj_MotherBrainGlassShatteringSparkle as u16,
                k,
                game,
            );
        }
    };
}
unsafe fn EprojInit_KiHunterAcidSpitCommon(k: u16, j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_y_vel[v2] = 0u16);
    set_ram_word!(
        game.eproj_y_pos[v2] = ((*gEnemyData(k as i32, game)).y_pos as i32 - 16i32) as u16
    );
    set_ram_word!(game.eproj_y_subpos[v2] = 0u16);
    set_ram_word!(game.eproj_x_subpos[v2] = 0u16);
}
unsafe fn EprojInit_KiHunterAcidSpitLeft(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_vel[v2] = -768i32 as u16);
    set_ram_word!(
        game.eproj_x_pos[v2] = ((*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).x_pos
            as i32
            - 22i32) as u16
    );
    EprojInit_KiHunterAcidSpitCommon(ram_word!(game, cur_enemy_index), j, game);
}
unsafe fn EprojInit_KiHunterAcidSpitRight(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_vel[v2] = 768u16);
    set_ram_word!(
        game.eproj_x_pos[v2] = ((*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).x_pos
            as i32
            + 22i32) as u16
    );
    EprojInit_KiHunterAcidSpitCommon(ram_word!(game, cur_enemy_index), j, game);
}
pub unsafe fn sub_86CFD5(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    set_ram_word!(game.eproj_pre_instr[v1] = (0x86cff8i32 & 0xffffi32) as u16);
    update_ram_word!(game.eproj_x_pos[v1], |fresh73| (fresh73 as i32 - 19i32)
        as u16);
}
pub unsafe fn sub_86CFE6(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    set_ram_word!(game.eproj_pre_instr[v1] = (0x86cff8i32 & 0xffffi32) as u16);
    update_ram_word!(game.eproj_x_pos[v1], |fresh72| (fresh72 as i32 + 19i32)
        as u16);
}
pub unsafe fn sub_86CFF8(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0 {
        set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_off_86CF56 as u16);
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
    } else if EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0 {
        set_ram_word!(game.eproj_x_vel[(k as i32 >> 1i32)] = 0u16);
    } else {
        let mut v2: u16 = (ram_word!(game, eproj_y_vel[v1]) as i32 + 16i32) as u16;
        set_ram_word!(game.eproj_y_vel[v1] = v2);
        if v2 as i32 - 512i32 & 0x8000i32 == 0 {
            v2 = 512u16;
        }
        set_ram_word!(game.eproj_y_vel[v1] = v2);
    };
}
unsafe fn EprojInit_KagosBugs(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_F[v2] = game.cur_enemy_index());
    let enemy_index = game.cur_enemy_index();
    set_ram_word!(game.eproj_x_pos[v2] = game.enemy_state(enemy_index).x_pos());
    set_ram_word!(game.eproj_y_pos[v2] = game.enemy_state(enemy_index).y_pos());
    let v4: u16 = ((game.random_number() as i32 & 7i32) + 1i32) as u16;
    set_ram_word!(game.eproj_G[v2] = v4);
    set_ram_word!(game.eproj_E[v2] = v4.wrapping_add(4));
    set_ram_word!(game.eproj_pre_instr[v2] = ((0x86d0cai32) as u16));
}
pub unsafe fn EprojPreInstr_KagosBugs_Func1(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let v2: u16 = EprojState(&game.g_ram).slot_word(0x1aff, v1 as usize);
    if v2 != 0 {
        let v3: u16 = v2.wrapping_sub(1);
        EprojStateMut(&mut game.g_ram).write(0x1aff, v1 as usize, v3);
        if v3 == 0 {
            QueueSfx2_Max6(0x6cu16, game);
        }
    }
}
pub unsafe fn EprojPreInstr_KagosBugs_Func2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let enemy_index = ram_word!(game, eproj_F[v1]);
    let v3: u16 =
        abs16((game.enemy_state(enemy_index).x_pos()).wrapping_sub(game.enemy_projectile_x_pos(k)));
    if v3 as i32 - 23i32 & 0x8000i32 == 0 {
        EprojStateMut(&mut game.g_ram).set_property_bits(v1 as usize, 0x8000);
    }
}
pub unsafe fn EprojPreInstr_KagosBugs(k: u16, game: &mut Game) {
    EprojPreInstr_KagosBugs_Func1(k, game);
    EprojPreInstr_KagosBugs_Func2(k, game);
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, eproj_G[v1]) != 0 {
        let fresh71 = EprojStateMut(&mut game.g_ram).add_word(0x1bfb, v1 as usize, u16::MAX);
    } else {
        EprojStateMut(&mut game.g_ram).start_instruction(v1 as usize, addr_word_86D052 as u16);
        set_ram_word!(game.eproj_pre_instr[v1] = ((0x86d0ebi32) as u16));
    };
}
static word_86D082: u16 = 0xe0u16;
pub unsafe fn EprojPreInstr_D0EC(k: u16, game: &mut Game) {
    let current_block: u64;
    EprojPreInstr_KagosBugs_Func1(k, game);
    EprojPreInstr_KagosBugs_Func2(k, game);
    if EprojBlockCollisition_Horiz(k, game) != 0 {
        set_ram_word!(game.eproj_x_vel[(k as i32 >> 1i32)] = 0u16);
        current_block = 4781832012581146689;
    } else if EprojBlockCollisition_Vertical(k, game) != 0 {
        current_block = 4781832012581146689;
    } else {
        let v2: u16 = ram_word!(game, eproj_y_vel[(k as i32 >> 1i32)]);
        let v3: bool = ((word_86D082) as i16).wrapping_add((v2) as i16) < 0i16;
        set_ram_word!(game.eproj_y_vel[(k as i32 >> 1i32)] = (word_86D082).wrapping_add(v2));
        if !v3 {
            current_block = 15427931788582360902;
        } else {
            return;
        }
    }
    if current_block == 4781832012581146689 {
        set_ram_word!(game.eproj_y_vel[(k as i32 >> 1i32)] = 256u16);
    }
    set_ram_word!(game.eproj_pre_instr[(k as i32 >> 1i32)] = ((0x86d128i32) as u16));
    EprojStateMut(&mut game.g_ram).start_instruction(usize::from(k >> 1), addr_word_86D04A as u16);
}
pub unsafe fn EprojPreInstr_D128(v0: u16, game: &mut Game) {
    EprojPreInstr_KagosBugs_Func1(v0, game);
    EprojPreInstr_KagosBugs_Func2(v0, game);
    if EprojBlockCollisition_Horiz(v0, game) as i32 & 1i32 != 0 {
        set_ram_word!(game.eproj_x_vel[(v0 as i32 >> 1i32)] = 0u16);
    } else if EprojBlockCollisition_Vertical(v0, game) as i32 & 1i32 != 0 {
        let v1: i32 = v0 as i32 >> 1i32;
        set_ram_word!(game.eproj_pre_instr[v1] = ((0x86d0ebi32) as u16));
        EprojStateMut(&mut game.g_ram).start_instruction(v1 as usize, addr_word_86D03C as u16);
    } else {
        update_ram_word!(
            game.eproj_y_vel[(v0 as i32 >> 1i32)],
            wrapping_add(word_86D082)
        );
    };
}
static g_word_86D086: u16 = 0x200u16;
static g_word_86D084: u16 = 0x30u16;
pub unsafe fn EprojInstr_D15C(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    EprojPreInstr_KagosBugs_Func1(k, game);
    EprojPreInstr_KagosBugs_Func2(k, game);
    let v2: i32 = k as i32 >> 1i32;
    let mut t: u16 = ((game.random_number() as i32 & 0x300i32) + 2048i32) as u16;
    set_ram_word!(game.eproj_y_vel[v2] = -(t as i32) as u16);
    let enemy_index = ram_word!(game, eproj_F[v2]);
    let t2: u16 =
        (game.enemy_state(enemy_index).x_pos()).wrapping_sub(game.enemy_projectile_x_pos(k));
    if ((abs16(t2)) as i16).wrapping_sub((g_word_86D084) as i16) >= 0i16 {
        t = (if t2 as i32 & 0x8000i32 != 0 {
            -1i32
        } else {
            0i32
        }) as u16;
    }
    let mut v4: u16 = g_word_86D086;
    if t as i32 & 0x100i32 != 0 {
        v4 = -(v4 as i32) as u16;
    }
    set_ram_word!(game.eproj_x_vel[v2] = v4);
    set_ram_word!(game.eproj_pre_instr[v2] = ((0x86d0eci32) as u16));
    epjp
}
pub unsafe fn EprojInstr_D1B6(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v1: i32 = k as i32 >> 1i32;
    set_ram_word!(
        game.eproj_G[v1] = ((game.random_number() as i32 & 0x1fi32) as u16).wrapping_add(1u16)
    );
    set_ram_word!(game.eproj_pre_instr[v1] = ((0x86d0cai32) as u16));
    epjp
}
pub unsafe fn EprojInstr_D1C7(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.eproj_gfx_idx[(k as i32 >> 1i32)] = 0u16);
    epjp
}
pub unsafe fn EprojInstr_D1CE(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    game.eproj_spawn_pt = Point16U {
        x: ram_word!(game, eproj_x_pos[(k as i32 >> 1i32)]),
        y: ram_word!(game, eproj_y_pos[(k as i32 >> 1i32)]),
    };
    SpawnEnemyDrops(addr_kEnemyDef_E7FF as u16, k, 0u16, game);
    epjp
}
unsafe fn EprojInit_MaridiaFloatersSpikes(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_subpos[v1] = 0u16);
    set_ram_word!(game.eproj_y_subpos[v1] = 0u16);
    set_ram_word!(game.eproj_x_vel[v1] = 0u16);
    set_ram_word!(game.eproj_y_vel[v1] = 0u16);
    let v2: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.eproj_x_pos[v1] = (*v2).x_pos);
    set_ram_word!(game.eproj_y_pos[v1] = (*v2).y_pos);
    set_ram_word!(game.eproj_E[v1] = ram_word!(game, cinematic_var8));
}
static word_86D21A: [i16; 8] = [0i16, 32i16, 32i16, 32i16, 0i16, -32i16, -32i16, -32i16];
static word_86D22A: [i16; 8] = [-32i16, -32i16, 0i16, 32i16, 32i16, 32i16, 0i16, -32i16];
pub unsafe fn EprojPreInstr_MaridiaFloatersSpikes(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    update_ram_word!(game.eproj_x_vel[v1], |fresh68| (fresh68 as i32
        + word_86D21A[ram_word!(game, eproj_E[v1]) as usize] as i32)
        as u16);
    if EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0 || {
        update_ram_word!(game.eproj_y_vel[v1], |fresh69| (fresh69 as i32
            + word_86D22A[ram_word!(game, eproj_E[v1]) as usize] as i32)
            as u16);
        EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0
    } {
        set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_off_86D218 as u16);
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
    }
}
unsafe fn EprojInit_WreckedShipRobotLaserCommon(k: u16, j: u16, game: &mut Game) {
    let v2: *mut EnemyData = gEnemyData(k as i32, game);
    let v3: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_y_pos[v3] = ((*v2).y_pos as i32 - 16i32) as u16);
    set_ram_word!(
        game.eproj_x_pos[v3] = ((*v2).x_pos as i32
            + (if ram_word!(game, eproj_x_vel[v3]) as i32 & 0x8000i32 != 0 {
                -4i32
            } else {
                4i32
            })) as u16
    );
    set_ram_word!(game.eproj_y_subpos[v3] = 0u16);
    set_ram_word!(game.eproj_x_subpos[v3] = 0u16);
    if (((*v2).x_width as i32 + (*v2).x_pos as i32 - ram_word!(game, layer1_x_pos) as i32) as i16)
        >= 0i16
        && (((*v2).x_pos as i32
            - (*v2).x_width as i32
            - 257i32
            - ram_word!(game, layer1_x_pos) as i32) as i16)
            < 0i16
        && (((*v2).y_height as i32 + (*v2).y_pos as i32 - ram_word!(game, layer1_y_pos) as i32)
            as i16)
            >= 0i16
        && (((*v2).y_pos as i32 - (*v2).y_height as i32 - 224i32) as i16) < 0i16
    {
        QueueSfx2_Max6(0x67u16, game);
    }
}
unsafe fn EprojInit_WreckedShipRobotLaserDown(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_vel[v2] =
            (*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).ai_var_A
    );
    set_ram_word!(game.eproj_y_vel[v2] = 128u16);
    set_ram_word!(game.eproj_gfx_idx[v2] = 0u16);
    EprojInit_WreckedShipRobotLaserCommon(ram_word!(game, cur_enemy_index), j, game);
}
unsafe fn EprojInit_WreckedShipRobotLaserHorizontal(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_vel[v2] =
            (*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).ai_var_A
    );
    set_ram_word!(game.eproj_y_vel[v2] = 0u16);
    EprojInit_WreckedShipRobotLaserCommon(ram_word!(game, cur_enemy_index), j, game);
}
unsafe fn EprojInit_WreckedShipRobotLaserUp(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_vel[v2] =
            (*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).ai_var_A
    );
    set_ram_word!(game.eproj_y_vel[v2] = -128i32 as u16);
    EprojInit_WreckedShipRobotLaserCommon(ram_word!(game, cur_enemy_index), j, game);
}
pub unsafe fn EprojPreInstr_WreckedShipRobotLaser(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    set_ram_word!(game.eproj_gfx_idx[v1] = 0u16);
    if EprojBlockCollisition_Horiz(k, game) as i32 & 1i32 != 0
        || EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0
    {
        set_ram_word!(game.eproj_id[v1] = 0u16);
    }
}
pub unsafe fn EprojInstr_AssignNewN00bTubeShardVelocity(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    NextRandom(game);
    let v2: i32 = k as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_vel[v2] = *(((&raw mut game.g_ram as *mut u8).offset(0x5e5) as *mut u16
            as *mut u8)
            .offset(1) as *mut u16)
    );
    set_ram_word!(game.eproj_y_vel[v2] = 192u16);
    epjp
}
pub unsafe fn EprojInstr_SetN00bTubeShardX(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v2: i32 = k as i32 >> 1i32;
    if ram_word!(game, nmi_frame_counter_word) as i32 & 1i32 != 0 {
        set_ram_word!(game.eproj_x_pos[v2] = ram_word!(game, eproj_F[v2]));
        set_ram_word!(game.eproj_spritemap_ptr[v2] = *(epjp as *mut u16));
    } else {
        set_ram_word!(
            game.eproj_x_pos[v2] = (128i32 - ram_word!(game, eproj_F[v2]) as i32 + 128i32) as u16
        );
        set_ram_word!(game.eproj_spritemap_ptr[v2] = *(epjp.offset(2) as *mut u16));
    }
    set_ram_word!(
        game.eproj_instr_list_ptr[v2] =
            (epjp.offset_from(RomBankBase(0x86u8, game)) as c_long + 4 as c_long) as u16
    );
    set_ram_word!(game.eproj_instr_timers[v2] = 1u16);
    null::<u8>()
}
pub unsafe fn EprojInstr_D62A(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    if ram_word!(game, nmi_frame_counter_word) as i32 & 1i32 != 0 {
        set_ram_word!(
            game.eproj_x_pos[(k as i32 >> 1i32)] = ram_word!(game, eproj_F[(k as i32 >> 1i32)])
        );
    } else {
        set_ram_word!(game.eproj_x_pos[(k as i32 >> 1i32)] = -4608i32 as u16);
    }
    let v2: i32 = k as i32 >> 1i32;
    set_ram_word!(game.eproj_spritemap_ptr[v2] = *(epjp as *mut u16));
    set_ram_word!(
        game.eproj_instr_list_ptr[v2] =
            (epjp.offset_from(RomBankBase(0x86u8, game)) as c_long + 2 as c_long) as u16
    );
    set_ram_word!(game.eproj_instr_timers[v2] = 1u16);
    null::<u8>()
}
pub unsafe fn EprojInstr_SetXvelRandom(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    NextRandom(game);
    set_ram_word!(
        game.eproj_x_vel[(k as i32 >> 1i32)] = *(((&raw mut game.g_ram as *mut u8).offset(0x5e5)
            as *mut u16 as *mut u8)
            .offset(1) as *mut u16)
    );
    epjp
}
unsafe fn EprojInit_N00bTubeCrack(j: u16, game: &mut Game) {
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v1] = (16i32 * ram_word!(game, plm_x_block) as i32 + 96i32) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v1] = (16i32 * ram_word!(game, plm_y_block) as i32 + 48i32) as u16
    );
}
static kEprojInit_N00bTubeShards_X: [i16; 10] = [
    -56i16, -64i16, -20i16, -40i16, -64i16, -48i16, -24i16, -40i16, 0i16, -8i16,
];
static kEprojInit_N00bTubeShards_Y: [i16; 10] = [
    8i16, -12i16, -26i16, -24i16, -32i16, 28i16, 16i16, -8i16, -24i16, 16i16,
];
static kEprojInit_N00bTubeShards_Xvel: [i16; 10] = [
    -384i16, -384i16, -160i16, -288i16, -288i16, -320i16, -96i16, -352i16, 0i16, -64i16,
];
static kEprojInit_N00bTubeShards_Yvel: [i16; 10] = [
    320i16, -256i16, -416i16, -288i16, -288i16, 448i16, 576i16, -96i16, -288i16, 384i16,
];
unsafe fn EprojInit_N00bTubeShards(j: u16, game: &mut Game) {
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    let v1: i32 = ram_word!(game, cinematic_var8) as i32 >> 1i32;
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_F[v2] = (kEprojInit_N00bTubeShards_X[v1 as usize] as i32
            + 16i32 * ram_word!(game, plm_x_block) as i32
            + 96i32) as u16
    );
    set_ram_word!(game.eproj_E[v2] = 0u16);
    set_ram_word!(
        game.eproj_y_pos[v2] = (kEprojInit_N00bTubeShards_Y[v1 as usize] as i32
            + 16i32 * ram_word!(game, plm_y_block) as i32
            + 48i32) as u16
    );
    set_ram_word!(
        game.eproj_instr_list_ptr[v2] = crate::rtl::rom_table_word!(game, 0x86d760, v1 as isize)
    );
    set_ram_word!(game.eproj_x_vel[v2] = kEprojInit_N00bTubeShards_Xvel[v1 as usize] as u16);
    set_ram_word!(game.eproj_y_vel[v2] = kEprojInit_N00bTubeShards_Yvel[v1 as usize] as u16);
}
static kEprojInit_N00bTubeReleasedAirBubbles_X: [u16; 6] =
    [40u16, 80u16, 104u16, 120u16, 152u16, 184u16];
static kEprojInit_N00bTubeReleasedAirBubbles_Y: [u16; 6] =
    [80u16, 72u16, 84u16, 32u16, 64u16, 84u16];
unsafe fn EprojInit_N00bTubeReleasedAirBubbles(j: u16, game: &mut Game) {
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    let v1: i32 = ram_word!(game, cinematic_var8) as i32 >> 1i32;
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_F[v2] = (kEprojInit_N00bTubeReleasedAirBubbles_X[v1 as usize] as i32
            + 16i32 * ram_word!(game, plm_x_block) as i32) as u16
    );
    set_ram_word!(game.eproj_E[v2] = 0u16);
    set_ram_word!(
        game.eproj_y_pos[v2] = (kEprojInit_N00bTubeReleasedAirBubbles_Y[v1 as usize] as i32
            + 16i32 * ram_word!(game, plm_y_block) as i32) as u16
    );
    set_ram_word!(game.eproj_y_vel[v2] = -1280i32 as u16);
}
pub unsafe fn sub_86D7BF(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, eproj_x_pos[v1]) != 0xee00u16 {
        set_ram_word!(game.eproj_E[v1] = ram_word!(game, eproj_x_pos[v1]));
    }
    if ram_word!(game, nmi_frame_counter_word) as i32 & 1i32 != 0 {
        set_ram_word!(game.eproj_x_pos[v1] = -4608i32 as u16);
    } else {
        set_ram_word!(game.eproj_x_pos[v1] = ram_word!(game, eproj_E[v1]));
    };
}
pub unsafe fn sub_86D7DE(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let amt: i32 = (192i16 as i32) << 8i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16).offset(v1 as isize)
            as *mut u16,
        amt as u32,
    );
}
pub unsafe fn EprojPreInstr_N00bTubeShards(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let mut amt: i32 = (ram_word!(game, eproj_x_vel[v1]) as i16 as i32) << 8i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1b23) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1aff) as *mut u16).offset(v1 as isize)
            as *mut u16,
        amt as u32,
    );
    amt = (ram_word!(game, eproj_y_vel[v1]) as i16 as i32) << 8i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16).offset(v1 as isize)
            as *mut u16,
        amt as u32,
    );
    Eproj_DeleteIfYposOutside(k, game);
}
pub unsafe fn sub_86D83D(k: u16, game: &mut Game) {
    let v2: i32 =
        (ram_word!(game, eproj_x_vel[(k as i32 >> 1i32)]) as i32 & 0x17ei32 | 0x80i32) >> 1i32;
    let v6: i32 = ram_word!(game, cinematic_var7) as i32 >> 1i32;
    let mut amt: i32 =
        ((kSinCosTable8bit_Sext[(v2 + 64i32) as usize] as i32 >> 2i32) as i16 as i32) << 8i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1b23) as *mut u16).offset(v6 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1aff) as *mut u16).offset(v6 as isize)
            as *mut u16,
        amt as u32,
    );
    update_ram_word!(game.eproj_x_vel[v6], |fresh67| (fresh67 as i32 + 2i32)
        as u16);
    amt = (ram_word!(game, eproj_y_vel[v6]) as i16 as i32) << 8i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v6 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16).offset(v6 as isize)
            as *mut u16,
        amt as u32,
    );
    Eproj_DeleteIfYposOutside(ram_word!(game, cinematic_var7), game);
}
pub unsafe fn sub_86D89F(k: u16, game: &mut Game) {
    let v1: u16 =
        (ram_word!(game, eproj_x_vel[(k as i32 >> 1i32)]) as i32 & 0x17ei32 | 0x80i32) as u16;
    let v2: i32 = v1 as i32 >> 1i32;
    let amt: i32 =
        ((kSinCosTable8bit_Sext[(v2 + 64i32) as usize] as i32 >> 2i32) as i16 as i32) << 8i32;
    let v5: i32 = ram_word!(game, cinematic_var7) as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1b23) as *mut u16).offset(v5 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1aff) as *mut u16).offset(v5 as isize)
            as *mut u16,
        amt as u32,
    );
    update_ram_word!(game.eproj_x_vel[v5], |fresh66| (fresh66 as i32 + 4i32)
        as u16);
    sub_86D8DF(ram_word!(game, cinematic_var7), game);
}
pub unsafe fn sub_86D8DF(k: u16, game: &mut Game) {
    let v5: i32 = k as i32 >> 1i32;
    let amt: i32 = (ram_word!(game, eproj_y_vel[v5]) as i16 as i32) << 8i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v5 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16).offset(v5 as isize)
            as *mut u16,
        amt as u32,
    );
    set_ram_word!(game.eproj_x_pos[v5] = ram_word!(game, eproj_F[v5]));
}
pub unsafe fn sub_86D992(v0: u16, game: &mut Game) {
    let v1: u16 = EprojState(&game.g_ram).init_param_1();
    let v2: i32 = v0 as i32 >> 1i32;
    set_ram_word!(game.eproj_E[v2] = EprojState(&game.g_ram).init_param_1());
    set_ram_word!(
        game.eproj_instr_list_ptr[v2] =
            crate::rtl::rom_table_word!(game, 0x86d96a, (v1 as i32 >> 1i32) as isize)
    );
    let enemy_index = game.cur_enemy_index();
    set_ram_word!(game.eproj_x_pos[v2] = game.enemy_state(enemy_index).x_pos());
    set_ram_word!(game.eproj_x_subpos[v2] = game.enemy_state(enemy_index).x_subpos());
    set_ram_word!(game.eproj_y_pos[v2] = game.enemy_state(enemy_index).y_pos());
    set_ram_word!(game.eproj_y_subpos[v2] = game.enemy_state(enemy_index).y_subpos());
    set_ram_word!(game.eproj_y_vel[v2] = -512i32 as u16);
    set_ram_word!(game.eproj_x_vel[v2] = 512u16);
    if EprojState(&game.g_ram).init_param_1() as i32 - 12i32 & 0x8000i32 == 0 {
        set_ram_word!(game.eproj_y_vel[v2] = -384i32 as u16);
        set_ram_word!(game.eproj_x_vel[v2] = 384u16);
    }
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let split_amount =
        ((EprojState(&game.g_ram).slot_word(0x1adb, v1 as usize) as i16 as i32) << 8i32) as u32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a93, 0x1a6f, v1 as usize, split_amount);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((ram_word!(game, eproj_x_vel[v1]) as i16 as i32) << 8i32) as u32,
    );
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let split_amount =
        ((EprojState(&game.g_ram).slot_word(0x1adb, v1 as usize) as i16 as i32) << 8i32) as u32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a4b, 0x1a27, v1 as usize, split_amount);
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let split_amount =
        ((EprojState(&game.g_ram).slot_word(0x1ab7, v1 as usize) as i16 as i32) << 8i32) as u32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a4b, 0x1a27, v1 as usize, split_amount);
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1(k: u16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, game);
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2(k: u16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, game);
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1(k: u16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, game);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2(k: u16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, game);
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1(k: u16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, game);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, game);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2(k: u16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, game);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, game);
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1(k: u16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, game);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, game);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2(k: u16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, game);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, game);
}
static kEprojPreInstr_SpikeShootingPlantSpikes: [Option<Func_Y_V>; 10] = unsafe {
    [
        Some(EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1 as unsafe fn(u16, &mut Game) -> ()),
        Some(EprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1 as unsafe fn(u16, &mut Game) -> ()),
        Some(EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2 as unsafe fn(u16, &mut Game) -> ()),
        Some(EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1 as unsafe fn(u16, &mut Game) -> ()),
        Some(EprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2 as unsafe fn(u16, &mut Game) -> ()),
        Some(EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2 as unsafe fn(u16, &mut Game) -> ()),
        Some(EprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1 as unsafe fn(u16, &mut Game) -> ()),
        Some(EprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1 as unsafe fn(u16, &mut Game) -> ()),
        Some(EprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2 as unsafe fn(u16, &mut Game) -> ()),
        Some(EprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2 as unsafe fn(u16, &mut Game) -> ()),
    ]
};
pub fn EprojPreInstrHelper_SpikeShootingPlantSpikes_Func1(k: u16, game: &mut Game) {
    if EprojPreInstrHelper_SpikeShootingPlantSpikes_Func2(k, game) != 0 {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    }
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes(k: u16, game: &mut Game) {
    kEprojPreInstr_SpikeShootingPlantSpikes
        [(ram_word!(game, eproj_E[(k as i32 >> 1i32)]) as i32 >> 1i32) as usize]
        .expect("non-null function pointer")(k, game);
    EprojPreInstrHelper_SpikeShootingPlantSpikes_Func1(k, game);
}
pub fn EprojPreInstrHelper_SpikeShootingPlantSpikes_Func2(k: u16, game: &mut Game) -> u16 {
    ((game
        .enemy_projectile_x_pos(k)
        .wrapping_sub(game.layer1_x_pos()) as i16)
        < 0i16
        || (game
            .layer1_x_pos()
            .wrapping_add(256)
            .wrapping_sub(game.enemy_projectile_x_pos(k)) as i16)
            < 0i16
        || (game
            .enemy_projectile_y_pos(k)
            .wrapping_sub(game.layer1_y_pos()) as i16)
            < 0i16
        || (game
            .layer1_y_pos()
            .wrapping_add(256)
            .wrapping_sub(game.enemy_projectile_y_pos(k)) as i16)
            < 0i16) as u16
}
unsafe fn EprojInit_DBF2(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_instr_list_ptr[v2] = addr_word_86DB0C as u16);
    set_ram_word!(game.eproj_E[v2] = (0x86db62i32 & 0xffffi32) as u16);
    if ram_word!(game, cinematic_var8) != 0 {
        set_ram_word!(game.eproj_E[v2] = (0x86db8ci32 & 0xffffi32) as u16);
    }
    let v3: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.eproj_x_pos[v2] = (*v3).x_pos);
    set_ram_word!(game.eproj_x_subpos[v2] = (*v3).x_subpos);
    set_ram_word!(game.eproj_y_pos[v2] = ((*v3).y_pos as i32 + 2i32) as u16);
    set_ram_word!(game.eproj_y_subpos[v2] = (*v3).y_subpos);
    set_ram_word!(game.eproj_y_vel[v2] = -256i32 as u16);
    set_ram_word!(game.eproj_x_vel[v2] = 256u16);
}
pub unsafe fn EprojPreInstr_DBF2_Func1(k: u16, game: &mut Game) {
    if EprojPreInstrHelper_DBF2_Func2(k, game) != 0 {
        set_ram_word!(game.eproj_id[(k as i32 >> 1i32)] = 0u16);
    }
}
pub unsafe fn EprojPreInstr_DBF2(k: u16, game: &mut Game) {
    CallEprojPreInstr(
        (ram_word!(game, eproj_E[(k as i32 >> 1i32)]) as i32 | 0x860000i32) as u32,
        k,
        game,
    );
    EprojPreInstr_DBF2_Func1(k, game);
}
pub unsafe fn EprojPreInstr_DBF2_MoveX1(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((ram_word!(game, eproj_y_vel[v1]) as i16 as i32) << 8i32) as u32,
    );
}
pub unsafe fn EprojPreInstr_DBF2_MoveX2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((ram_word!(game, eproj_x_vel[v1]) as i16 as i32) << 8i32) as u32,
    );
}
pub unsafe fn EprojPreInstrHelper_DBF2_Func2(k: u16, game: &mut Game) -> u16 {
    let v1: i32 = k as i32 >> 1i32;
    (((ram_word!(game, eproj_x_pos[v1]) as i32 - ram_word!(game, layer1_x_pos) as i32) as i16)
        < 0i16
        || ((ram_word!(game, layer1_x_pos) as i32 + 256i32
            - ram_word!(game, eproj_x_pos[v1]) as i32) as i16)
            < 0i16
        || ((ram_word!(game, eproj_y_pos[v1]) as i32 - ram_word!(game, layer1_y_pos) as i32)
            as i16)
            < 0i16
        || ((ram_word!(game, layer1_y_pos) as i32 + 256i32
            - ram_word!(game, eproj_y_pos[v1]) as i32) as i16)
            < 0i16) as u16
}
pub unsafe fn EprojInstr_DC5A(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.eproj_properties[(k as i32 >> 1i32)] = 12288u16);
    epjp
}
pub unsafe fn EprojInstr_SpawnEnemyDrops_0(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v2: i32 = k as i32 >> 1i32;
    game.eproj_spawn_pt = Point16U {
        x: ram_word!(game, eproj_x_pos[v2]),
        y: ram_word!(game, eproj_y_pos[v2]),
    };
    SpawnEnemyDrops(addr_kEnemyDef_DF7F as u16, k, 0u16, game);
    epjp
}
pub unsafe fn EprojInstr_SpawnSporesEproj(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v2: i32 = k as i32 >> 1i32;
    game.eproj_spawn_pt = Point16U {
        x: ram_word!(game, eproj_x_pos[v2]),
        y: ram_word!(game, eproj_y_pos[v2]),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_Spores as u16, 0u16, game);
    epjp
}
unsafe fn EprojInit_Spores(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    let ref mut fresh4 =
        *((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16).offset(v2 as isize);
    *fresh4 = game.eproj_spawn_pt.x;
    set_ram_word!(game.eproj_F[v2] = *fresh4);
    set_ram_word!(game.eproj_y_pos[v2] = game.eproj_spawn_pt.y);
    set_ram_word!(game.eproj_gfx_idx[v2] = 512u16);
}
static word_86DCB9: [i16; 5] = [-64i16, -56i16, -48i16, -40i16, -32i16];
unsafe fn EprojInit_SporeSpawnStalk(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_y_pos[v1] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).y_pos
                as i32
                + word_86DCB9[ram_word!(game, cinematic_var8) as usize] as i32) as u16
    );
    set_ram_word!(
        game.eproj_x_pos[v1] =
            (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
    );
}
pub unsafe fn sub_86DCC3(v0: u16, game: &mut Game) {
    let v1: i32 = v0 as i32 >> 1i32;
    set_ram_word!(
        game.eproj_y_pos[v1] =
            ((*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).y_pos
                as i32
                - 96i32) as u16
    );
    set_ram_word!(
        game.eproj_x_pos[v1] =
            (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
    );
}
static kEprojInit_SporeSpawners_X: [i16; 4] = [0x20i16, 0x60i16, 0xa0i16, 0xe0i16];
unsafe fn EprojInit_SporeSpawners(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v1] =
            kEprojInit_SporeSpawners_X[ram_word!(game, cinematic_var8) as usize] as u16
    );
    set_ram_word!(game.eproj_y_pos[v1] = 520u16);
}
pub unsafe fn EprojPreInstr_Spores(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let v2: u16 = *(((&raw mut game.g_ram as *mut u8).offset(0x1aff) as *mut u16)
        .offset(v1 as isize) as *mut u16 as *mut u8)
        .offset(0) as u16;
    let mut r18: u16 = SignExtend8(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86dd6c)) as *const u8 as *mut u8).offset(
            *(((&raw mut game.g_ram as *mut u8).offset(0x1aff) as *mut u16).offset(v1 as isize)
                as *mut u16 as *mut u8)
                .offset(0) as isize,
        ) as u16,
    );
    if (ram_word!(game, eproj_F[v1]) as i32 & 0x80i32) != 0i32 {
        r18 = -(r18 as i32) as u16;
    }
    update_ram_word!(
        game.eproj_x_pos[v1],
        |fresh65| (fresh65 as i32 + r18 as i32) as u16
    );
    r18 = SignExtend8(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0x86dd6c)) as *const u8 as *mut u8)
            .offset((v2 as i32 + 1i32) as isize) as u16,
    );
    let v3: u16 = (r18 as i32 + ram_word!(game, eproj_y_pos[v1]) as i32 + r18 as i32) as u16;
    set_ram_word!(game.eproj_y_pos[v1] = v3);
    if v3 as i32 - 768i32 & 0x8000i32 == 0 {
        set_ram_word!(game.eproj_id[v1] = 0u16);
    }
    set_ram_word!(
        game.eproj_E[v1] = (*(((&raw mut game.g_ram as *mut u8).offset(0x1aff) as *mut u16)
            .offset(v1 as isize) as *mut u16 as *mut u8)
            .offset(0) as i32
            + 2i32) as u8 as u16
    );
}
pub unsafe fn EprojPreInstr_SporeSpawners(k: u16, game: &mut Game) {
    if ram_word!(game, kraid_unk9000) == 0 {
        let v1: i32 = k as i32 >> 1i32;
        if ram_word!(game, eproj_F[v1]) == 0 {
            set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_word_86DC06 as u16);
            set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
            set_ram_word!(game.eproj_F[v1] = (NextRandom(game) as i32 & 0x1ffi32) as u16);
        }
        update_ram_word!(game.eproj_F[v1], wrapping_sub(1));
    }
}
unsafe fn EprojInit_NamiFuneFireball(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_instr_list_ptr[v2] = addr_word_86DE96 as u16);
    set_ram_word!(game.eproj_E[v2] = (0x86df40i32 & 0xffffi32) as u16);
    if ram_word!(game, cinematic_var8) != 0 {
        set_ram_word!(game.eproj_instr_list_ptr[v2] = addr_word_86DEA6 as u16);
        set_ram_word!(game.eproj_E[v2] = (0x86df6ai32 & 0xffffi32) as u16);
    }
    let v3: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.eproj_x_pos[v2] = (*v3).x_pos);
    set_ram_word!(game.eproj_x_subpos[v2] = (*v3).x_subpos);
    set_ram_word!(game.eproj_y_pos[v2] = (*v3).y_pos);
    set_ram_word!(game.eproj_y_subpos[v2] = (*v3).y_subpos);
    if ((*v3).parameter_1 as i32 & 0xfi32) != 0i32 {
        update_ram_word!(game.eproj_y_pos[v2], |fresh3| (fresh3 as i32 + 4i32) as u16);
    }
    let v4: i32 =
        (4i32 * *(&raw mut (*v3).parameter_2 as *mut u8).offset(0) as i32) as u16 as i32 >> 1i32;
    set_ram_word!(game.eproj_y_vel[v2] = crate::rtl::rom_table_word!(game, 0x86deb6, v4 as isize));
    set_ram_word!(
        game.eproj_x_vel[v2] = crate::rtl::rom_table_word!(game, 0x86deb6, (v4 + 1i32) as isize)
    );
}
unsafe fn EprojInit_NamiFuneFireball_MoveX1(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((ram_word!(game, eproj_y_vel[v1]) as i16 as i32) << 8i32) as u32,
    );
}
unsafe fn EprojInit_NamiFuneFireball_MoveX2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((ram_word!(game, eproj_x_vel[v1]) as i16 as i32) << 8i32) as u32,
    );
}
pub unsafe fn CallNamiFuneFireballFunc(ea: u32, k: u16, game: &mut Game) {
    match ea {
        8838976 => EprojInit_NamiFuneFireball_MoveX1(k, game),
        8839018 => EprojInit_NamiFuneFireball_MoveX2(k, game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn EprojPreInstr_NamiFuneFireball(v0: u16, game: &mut Game) {
    CallNamiFuneFireballFunc(
        (ram_word!(game, eproj_E[(v0 as i32 >> 1i32)]) as i32 | 0x860000i32) as u32,
        v0,
        game,
    );
    Eproj_NamiFuneFireball_After(v0, game);
}
pub unsafe fn Eproj_NamiFuneFireball_After(v0: u16, game: &mut Game) {
    if EprojPreInstrHelper_DBF2_Func2(v0, game) != 0 {
        set_ram_word!(game.eproj_id[(v0 as i32 >> 1i32)] = 0u16);
    }
}
pub unsafe fn sub_86DFA0(k: u16, game: &mut Game) -> u16 {
    let v1: i32 = k as i32 >> 1i32;
    (((ram_word!(game, eproj_x_pos[v1]) as i32 - ram_word!(game, layer1_x_pos) as i32) as i16)
        < 0i16
        || ((ram_word!(game, layer1_x_pos) as i32 + 256i32
            - ram_word!(game, eproj_x_pos[v1]) as i32) as i16)
            < 0i16) as u16
}
pub unsafe fn EprojInstr_DFEA(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v1: i32 = k as i32 >> 1i32;
    game.eproj_spawn_pt = Point16U {
        x: ram_word!(game, eproj_x_pos[v1]),
        y: ram_word!(game, eproj_y_pos[v1]),
    };
    SpawnEnemyDrops(addr_kEnemyDef_E83F as u16, k, 0u16, game);
    epjp
}
unsafe fn EprojInit_LavaThrownByLavaman(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_instr_list_ptr[v2] = addr_word_86DFD8 as u16);
    set_ram_word!(game.eproj_E[v2] = (0x86e050i32 & 0xffffi32) as u16);
    if ram_word!(game, cinematic_var8) != 0 {
        set_ram_word!(game.eproj_instr_list_ptr[v2] = addr_word_86DFDE as u16);
        set_ram_word!(game.eproj_E[v2] = (0x86e07ai32 & 0xffffi32) as u16);
    }
    let v3: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.eproj_x_pos[v2] = (*v3).x_pos);
    set_ram_word!(game.eproj_x_subpos[v2] = (*v3).x_subpos);
    set_ram_word!(game.eproj_y_pos[v2] = ((*v3).y_pos as i32 + 2i32) as u16);
    set_ram_word!(game.eproj_y_subpos[v2] = (*v3).y_subpos);
    set_ram_word!(game.eproj_y_vel[v2] = -768i32 as u16);
    set_ram_word!(game.eproj_x_vel[v2] = 768u16);
}
pub unsafe fn CallLavamanFunc(ea: u32, k: u16, game: &mut Game) {
    match ea {
        8839248 => Eproj_LavaThrownByLavaman_MoveX1(k, game),
        8839290 => Eproj_LavaThrownByLavaman_MoveX2(k, game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn sub_86E049(v0: u16, game: &mut Game) {
    CallLavamanFunc(
        (ram_word!(game, eproj_E[(v0 as i32 >> 1i32)]) as i32 | 0x860000i32) as u32,
        v0,
        game,
    );
    sub_86E0A4(v0, game);
}
pub unsafe fn Eproj_LavaThrownByLavaman_MoveX1(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((ram_word!(game, eproj_y_vel[v1]) as i16 as i32) << 8i32) as u32,
    );
}
pub unsafe fn Eproj_LavaThrownByLavaman_MoveX2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((ram_word!(game, eproj_x_vel[v1]) as i16 as i32) << 8i32) as u32,
    );
}
pub unsafe fn sub_86E0A4(v0: u16, game: &mut Game) {
    if sub_86E0B0(v0, game) != 0 {
        set_ram_word!(game.eproj_id[(v0 as i32 >> 1i32)] = 0u16);
    }
}
pub unsafe fn sub_86E0B0(k: u16, game: &mut Game) -> u16 {
    let v1: i32 = k as i32 >> 1i32;
    (((ram_word!(game, eproj_x_pos[v1]) as i32 - ram_word!(game, layer1_x_pos) as i32) as i16)
        < 0i16
        || ((ram_word!(game, layer1_x_pos) as i32 + 256i32
            - ram_word!(game, eproj_x_pos[v1]) as i32) as i16)
            < 0i16
        || ((ram_word!(game, eproj_y_pos[v1]) as i32 - ram_word!(game, layer1_y_pos) as i32)
            as i16)
            < 0i16
        || ((ram_word!(game, layer1_y_pos) as i32 + 256i32
            - ram_word!(game, eproj_y_pos[v1]) as i32) as i16)
            < 0i16) as u16
}
unsafe fn EprojInit_DustCloudOrExplosion(v0: u16, game: &mut Game) {
    let v1: i32 = v0 as i32 >> 1i32;
    set_ram_word!(
        game.eproj_instr_list_ptr[v1] = crate::rtl::rom_table_word!(
            game,
            0x86e42c,
            EprojState(&game.g_ram).init_param_1() as isize
        )
    );
    set_ram_word!(game.eproj_x_pos[v1] = game.eproj_spawn_pt.x);
    set_ram_word!(game.eproj_y_pos[v1] = game.eproj_spawn_pt.y);
}
unsafe fn EprojInit_EyeDoorSmoke(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_instr_list_ptr[v1] = crate::rtl::rom_table_word!(
            game,
            0x86e42c,
            ram_word!(game, cinematic_var8) as u8 as isize
        )
    );
    let v2: i32 = 8i32
        * *((&raw mut game.g_ram as *mut u8).offset(0x1993) as *mut u16 as *mut u8).offset(1isize)
            as i32
        >> 1i32;
    let x: u16 = (crate::rtl::rom_table_word!(game, 0x86e47e, (v2 + 2i32) as isize) as i32
        + (crate::rtl::rom_table_word!(game, 0x86e47e, v2 as isize) as i32
            & ram_word!(game, random_number) as i32)) as u16;
    let y: u16 = (crate::rtl::rom_table_word!(game, 0x86e47e, (v2 + 3i32) as isize) as i32
        + (crate::rtl::rom_table_word!(game, 0x86e47e, (v2 + 1i32) as isize) as i32
            & ram_word!(game, random_number) as i32 >> 8i32)) as u16;
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    set_ram_word!(
        game.eproj_x_pos[v1] =
            (x as i32 + 8i32 * (2i32 * ram_word!(game, plm_x_block) as i32 + 1i32)) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v1] =
            (y as i32 + 8i32 * (2i32 * ram_word!(game, plm_y_block) as i32 + 1i32)) as u16
    );
    NextRandom(game);
}
pub fn EprojPreInstr_DustCloudOrExplosion(k: u16, game: &mut Game) {
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    }
}
pub unsafe fn EprojInstr_SetYVel(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    set_ram_word!(game.eproj_y_vel[(k as i32 >> 1i32)] = *(epjp as *mut u16));
    epjp.offset(2)
}
unsafe fn EprojInit_SpawnedGate_Common(j: u16, v1: u16, game: &mut Game) {
    let v2: u16 = ram_word!(game, plm_id);
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    let v3: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_E[v3] = ram_word!(game, plm_block_indices[(v2 as i32 >> 1i32)]));
    set_ram_word!(game.eproj_x_pos[v3] = (16u16).wrapping_mul(ram_word!(game, plm_x_block)));
    set_ram_word!(
        game.eproj_y_pos[v3] = (v1 as i32 + 16i32 * ram_word!(game, plm_y_block) as i32) as u16
    );
}
unsafe fn EprojInit_SpawnedShotGate(j: u16, game: &mut Game) {
    EprojInit_SpawnedGate_Common(j, 0u16, game);
}
unsafe fn EprojInit_ClosedDownwardsShotGate(j: u16, game: &mut Game) {
    EprojInit_SpawnedGate_Common(j, 0x40u16, game);
}
unsafe fn EprojInit_ClosedUpwardsShotGate(j: u16, game: &mut Game) {
    EprojInit_SpawnedGate_Common(j, -0x40i32 as u16, game);
}
pub unsafe fn EprojPreInstr_E605(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let mut v2: u16 =
        (ram_word!(game, eproj_timers[v1])).wrapping_add(abs16(ram_word!(game, eproj_y_vel[v1])));
    if v2 >= 0x1000 {
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
        let fresh60 = EprojStateMut(&mut game.g_ram).add_word(0x1b47, v1 as usize, 1);
        let fresh61 = EprojStateMut(&mut game.g_ram).add_word(0x1b47, v1 as usize, 1);
        v2 = 0u16;
    }
    set_ram_word!(game.eproj_timers[v1] = v2);
    update_ram_word!(game.eproj_y_subpos[v1], |fresh62| (fresh62 as i32
        + ((*(((&raw mut game.g_ram as *mut u8).offset(0x1adb) as *mut u16).offset(v1 as isize)
            as *mut u16 as *mut u8)
            .offset(0) as i32)
            << 8i32))
        as u16);
    update_ram_word!(
        game.eproj_y_pos[v1],
        wrapping_add(
            (*(((&raw mut game.g_ram as *mut u8).offset(0x1adb) as *mut u16).offset(v1 as isize)
                as *mut u16 as *mut u8)
                .offset(1isize,) as i8) as u16
        )
    );
}
unsafe fn EprojInit_SaveStationElectricity(j: u16, game: &mut Game) {
    CalculatePlmBlockCoords(ram_word!(game, plm_id), game);
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v1] = (16i32 * (ram_word!(game, plm_x_block) as i32 + 1i32)) as u16
    );
    set_ram_word!(
        game.eproj_y_pos[v1] = (16i32 * (ram_word!(game, plm_y_block) as i32 - 2i32)) as u16
    );
}
fn CheckIfEprojIsOffScreen(k: u16, game: &mut Game) -> u16 {
    if (game
        .enemy_projectile_x_pos(k)
        .wrapping_sub(game.layer1_x_pos()) as i16)
        >= 0i16
        && ((game.enemy_projectile_x_pos(k) as i32 - (game.layer1_x_pos() as i32 + 256i32)) as i16)
            < 0i16
        && (game
            .enemy_projectile_y_pos(k)
            .wrapping_sub(game.layer1_y_pos()) as i16)
            >= 0i16
        && ((game.enemy_projectile_y_pos(k) as i32 - (game.layer1_y_pos() as i32 + 256i32)) as i16)
            < 0i16
    {
        return 0u16;
    }
    1u16
}
pub unsafe fn Eproj_FuncE722(k: u16, game: &mut Game) -> u16 {
    let mut v2: i16 = 0;
    let mut v3: i16 = 0;
    let v1: i32 = k as i32 >> 1i32;
    v2 = ram_word!(game, eproj_x_pos[v1]) as i16;
    let result: u16 = 1u16;
    if v2 >= 0i16 && v2 as i32 - 512i32 & 0x8000i32 != 0 {
        v3 = ram_word!(game, eproj_y_pos[v1]) as i16;
        if v3 >= 0i16 && v3 as i32 - 512i32 & 0x8000i32 != 0 {
            return 0u16;
        }
    }
    result
}
pub unsafe fn Eproj_FuncE73E_MoveXY(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if (ram_word!(game, g_word_7E97DC[v1]) as i32 + 64i32 & 0x80i32) != 0i32 {
        AddToHiLo(
            ((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16).offset(v1 as isize)
                as *mut u16,
            ((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16).offset(v1 as isize)
                as *mut u16,
            -((ram_word!(game, eproj_x_vel[v1]) as i32) << 16i32
                | ram_word!(game, eproj_E[v1]) as i32) as u32,
        );
    } else {
        AddToHiLo(
            ((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16).offset(v1 as isize)
                as *mut u16,
            ((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16).offset(v1 as isize)
                as *mut u16,
            (ram_word!(game, eproj_x_vel[v1]) as u32) << 16i32
                | ram_word!(game, eproj_E[v1]) as u32,
        );
    }
    if (ram_word!(game, g_word_7E97DC[v1]) as i32 + 128i32 & 0x80i32) != 0i32 {
        AddToHiLo(
            ((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v1 as isize)
                as *mut u16,
            ((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16).offset(v1 as isize)
                as *mut u16,
            -((ram_word!(game, eproj_y_vel[v1]) as i32) << 16i32
                | ram_word!(game, eproj_F[v1]) as i32) as u32,
        );
    } else {
        AddToHiLo(
            ((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v1 as isize)
                as *mut u16,
            ((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16).offset(v1 as isize)
                as *mut u16,
            (ram_word!(game, eproj_y_vel[v1]) as u32) << 16i32
                | ram_word!(game, eproj_F[v1]) as u32,
        );
    };
}
pub unsafe fn Eproj_AngleToSamus(j: u16, r18: u16, r20: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v1] = r18);
    set_ram_word!(game.eproj_y_pos[v1] = r20);
    let v2: u16 = (64i32
        - CalculateAngleFromXY(
            (ram_word!(game, samus_x_pos) as i32 - ram_word!(game, eproj_x_pos[v1]) as i32) as u16,
            (ram_word!(game, samus_y_pos) as i32 - ram_word!(game, eproj_y_pos[v1]) as i32) as u16,
            game,
        ) as i32) as u8 as u16;
    set_ram_word!(game.g_word_7E97DC[v1] = v2);
    let pt: Point32 = ConvertAngleToXy(v2, ram_word!(game, cinematic_var8), game);
    set_ram_word!(game.eproj_x_vel[v1] = (pt.x >> 16i32) as u16);
    set_ram_word!(game.eproj_E[v1] = pt.x as u16);
    set_ram_word!(game.eproj_y_vel[v1] = (pt.y >> 16i32) as u16);
    set_ram_word!(game.eproj_F[v1] = pt.y as u16);
}
unsafe fn EprojInit_BotwoonsBody(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    set_ram_word!(
        game.eproj_x_pos[v1] =
            (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).x_pos
    );
    set_ram_word!(
        game.eproj_y_pos[v1] =
            (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).y_pos
    );
    set_ram_word!(game.eproj_y_vel[v1] = 0u16);
    let v2: u16 = (if (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData)
        .offset(0))
    .ai_var_A
        != 0u16
    {
        16i32
    } else {
        48i32
    }) as u16;
    let v3: u16 = crate::rtl::rom_table_word!(game, 0x86e9f1, (v2 as i32 >> 1i32) as isize);
    set_ram_word!(game.eproj_instr_list_ptr[v1] = v3);
    set_ram_word!(game.eproj_F[v1] = v3);
    set_ram_word!(game.eproj_E[v1] = v2);
    set_ram_word!(game.eproj_x_vel[v1] = (0x86ea98i32 & 0xffffi32) as u16);
    let Ex: *mut ExtraEnemyRam7800 = gExtraEnemyRam7800(
        (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).ai_var_A
            as i32,
        game,
    );
    (*Ex).kraid.kraid_next = j;
    set_ram_word!(game.eproj_flags[v1] = 2u16);
    (*Ex).kraid.kraid_healths_4ths[0i32 as usize] = 1u16;
}
pub unsafe fn CallBotwoonEprojFunc(ea: u32, k: u16, game: &mut Game) {
    match ea {
        8841880 => Eproj_BotwoonsBody_Main(k, game),
        8841972 => Eproj_BotwonsBodyFunction_Dying(k, game),
        8841988 => Eproj_BotwonsBodyFunction_Dying2(k, game),
        8842015 => Eproj_BotwonsBodyFunction_DyingFalling(k, game),
        8842131 => (),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn EprojPreInstr_BotwoonsBody(k: u16, game: &mut Game) {
    if *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x8000) as *mut ExtraEnemyRam8000)
        .offset(0))
    .pad as *mut u8)
        .offset(32) as *mut u8 as *mut u16)
        != 0
    {
        let v1: i32 = k as i32 >> 1i32;
        if ram_word!(game, eproj_x_vel[v1]) as i32 == 0x86ea98i32 & 0xffffi32 {
            set_ram_word!(game.eproj_x_vel[v1] = (0x86eaf4i32 & 0xffffi32) as u16);
        }
    }
    CallBotwoonEprojFunc(
        (ram_word!(game, eproj_x_vel[(k as i32 >> 1i32)]) as i32 | 0x860000i32) as u32,
        k,
        game,
    );
}
pub unsafe fn Eproj_BotwoonsBody_Main(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let v2: u16 = crate::rtl::rom_table_word!(
        game,
        0x86e9f1,
        (ram_word!(game, eproj_E[v1]) as i32 >> 1i32) as isize
    );
    if v2 != ram_word!(game, eproj_F[v1]) {
        set_ram_word!(game.eproj_instr_list_ptr[v1] = v2);
        set_ram_word!(game.eproj_F[v1] = v2);
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
    }
    Eproj_BotwoonsBodyHurtFlashHandling1(k, game);
}
pub unsafe fn Eproj_BotwoonsBodyHurtFlashHandling1(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    update_ram_word!(
        game.eproj_gfx_idx[v1],
        |fresh58| (fresh58 as i32 | 0xe00i32) as u16
    );
    if (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).flash_timer
        != 0
        && (ram_word!(game, random_enemy_counter) as i32 & 2i32) != 0i32
    {
        update_ram_word!(
            game.eproj_gfx_idx[v1],
            |fresh59| (fresh59 as i32 & 0xf1ffi32) as u16
        );
    }
}
pub unsafe fn Eproj_BotwoonsBodyHurtFlashHandling2(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    update_ram_word!(
        game.eproj_gfx_idx[v1],
        |fresh55| (fresh55 as i32 | 0xe00i32) as u16
    );
    if (*((&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut EnemyData).offset(0)).flash_timer
        != 0
        && (ram_word!(game, random_enemy_counter) as i32 & 2i32) != 0i32
    {
        update_ram_word!(
            game.eproj_gfx_idx[v1],
            |fresh56| (fresh56 as i32 & 0xf1ffi32) as u16
        );
    }
}
pub unsafe fn Eproj_BotwonsBodyFunction_Dying(v0: u16, game: &mut Game) {
    let v1: i32 = v0 as i32 >> 1i32;
    set_ram_word!(game.eproj_E[v1] = (4i32 * v0 as i32 + 96i32) as u16);
    set_ram_word!(game.eproj_x_vel[v1] = 0xeb04u16);
    Eproj_BotwonsBodyFunction_Dying2(v0, game);
}
pub unsafe fn Eproj_BotwonsBodyFunction_Dying2(v0: u16, game: &mut Game) {
    let v1: i32 = v0 as i32 >> 1i32;
    if update_ram_word!(game.eproj_E[v1], wrapping_add(1), reread) as i32 - 256i32 & 0x8000i32 == 0
    {
        set_ram_word!(game.eproj_x_vel[v1] = (0x86eb1fi32 & 0xffffi32) as u16);
    }
    set_ram_word!(game.eproj_instr_timers[v1] = 0u16);
    Eproj_BotwoonsBodyHurtFlashHandling1(v0, game);
}
pub unsafe fn Eproj_BotwonsBodyFunction_DyingFalling(v0: u16, game: &mut Game) {
    let v1: i32 = v0 as i32 >> 1i32;
    let v3: i32 = 8i32 * ((ram_word!(game, eproj_y_vel[v1]) as i32 & 0xff00i32) >> 8i32) >> 1i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16).offset(v1 as isize)
            as *mut u16,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const u8 as *mut u32)
            .offset((v3 >> 1i32) as isize),
    );
    if ram_word!(game, eproj_y_pos[v1]) as i32 - 200i32 & 0x8000i32 != 0 {
        let v7: i32 = v0 as i32 >> 1i32;
        update_ram_word!(game.eproj_y_vel[v7], |fresh54| (fresh54 as i32 + 192i32)
            as u16);
        set_ram_word!(game.eproj_instr_timers[v7] = 0u16);
        Eproj_BotwoonsBodyHurtFlashHandling2(v0, game);
    } else {
        set_ram_word!(game.eproj_y_pos[v1] = 200u16);
        set_ram_word!(game.eproj_x_vel[v1] = (0x86eb93i32 & 0xffffi32) as u16);
        set_ram_word!(game.eproj_instr_list_ptr[v1] = addr_word_86E208 as u16);
        set_ram_word!(game.eproj_gfx_idx[v1] = 2560u16);
        set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
        QueueSmallExplosionSfx(game);
        if v0 == 10u16 {
            *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x8800)
                as *mut ExtraEnemyRam8800)
                .offset(0))
            .pad as *mut u8)
                .offset(62) as *mut u8 as *mut u16) = 1u16;
        }
    };
}
pub unsafe fn QueueSmallExplosionSfx(game: &mut Game) {
    QueueSfx2_Max6(0x24u16, game);
}
unsafe fn EprojInit_BotwoonsSpit(j: u16, game: &mut Game) {
    let v1: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v2] = (*v1).x_pos);
    set_ram_word!(game.eproj_y_pos[v2] = (*v1).y_pos);
    set_ram_word!(game.eproj_instr_list_ptr[v2] = addr_word_86EBAE as u16);
    set_ram_word!(game.g_word_7E97DC[v2] = *(&raw mut game.g_ram as *mut u8).offset(0x60b) as u16);
    let pt: Point32 = ConvertAngleToXy(
        *(&raw mut game.g_ram as *mut u8).offset(0x60b) as u16,
        ram_word!(game, cinematic_var8),
        game,
    );
    set_ram_word!(game.eproj_x_vel[v2] = (pt.x >> 16i32) as u16);
    set_ram_word!(game.eproj_E[v2] = pt.x as u16);
    set_ram_word!(game.eproj_y_vel[v2] = (pt.y >> 16i32) as u16);
    set_ram_word!(game.eproj_F[v2] = pt.y as u16);
}
pub unsafe fn EprojPreInstr_BotwoonsSpit(k: u16, game: &mut Game) {
    Eproj_FuncE73E_MoveXY(k, game);
    sub_86EC0C(k, game);
}
pub unsafe fn sub_86EC0C(k: u16, game: &mut Game) {
    if sub_86EC18(k, game) != 0 {
        set_ram_word!(game.eproj_id[(k as i32 >> 1i32)] = 0u16);
    }
}
pub unsafe fn sub_86EC18(k: u16, game: &mut Game) -> u16 {
    let v1: i32 = k as i32 >> 1i32;
    (((ram_word!(game, eproj_x_pos[v1]) as i32 - ram_word!(game, layer1_x_pos) as i32) as i16)
        < 0i16
        || ((ram_word!(game, layer1_x_pos) as i32 + 256i32
            - ram_word!(game, eproj_x_pos[v1]) as i32) as i16)
            < 0i16
        || ((ram_word!(game, eproj_y_pos[v1]) as i32 - ram_word!(game, layer1_y_pos) as i32)
            as i16)
            < 0i16
        || ((ram_word!(game, layer1_y_pos) as i32 + 256i32
            - ram_word!(game, eproj_y_pos[v1]) as i32) as i16)
            < 0i16) as u16
}
unsafe fn EprojInit_YappingMawsBody(j: u16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v3: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v3] = game.enemy_state(enemy_index).x_pos());
    set_ram_word!(game.eproj_y_pos[v3] = game.enemy_state(enemy_index).y_pos());
    set_ram_word!(game.eproj_instr_list_ptr[v3] = addr_word_86EC5C as u16);
    if game.enemy_state(enemy_index).parameter_2() == 0 {
        set_ram_word!(game.eproj_instr_list_ptr[v3] = addr_word_86EC56 as u16);
    }
    *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x8800) as *mut ExtraEnemyRam8800)
        .offset(0))
    .pad as *mut u8)
        .offset(
            (game.cur_enemy_index() as i32
                + 2i32
                    * *((&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x8800)
                        as *mut ExtraEnemyRam8800)
                        .offset(0))
                    .pad as *mut u8)
                        .offset((game.cur_enemy_index() as i32 + 8i32) as isize)
                        as *mut u8 as *mut u16) as i32) as u16 as isize,
        ) as *mut u8 as *mut u16) = j;
}
pub unsafe fn EprojInstr_ECE3(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let x: u16 = (game.enemy_projectile_x_pos(k) as i32 + (NextRandom(game) as i32 & 0x3fi32)
        - 32i32) as u16;
    let y: u16 = ((game.enemy_projectile_y_pos(k) as i32
        + (((game.random_number()) & (0x3f00u16)) as i32 >> 8i32)) as u16)
        .wrapping_sub(32u16);
    CreateSpriteAtPos(x, y, *(epjp as *mut u16), 0u16, game);
    epjp.offset(2)
}
pub unsafe fn EprojInstr_ED17(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let x: u16 = (game.enemy_projectile_x_pos(k) as i32 + (NextRandom(game) as i32 & 0x1fi32)
        - 16i32) as u16;
    let y: u16 = ((game.enemy_projectile_y_pos(k) as i32
        + (((game.random_number()) & (0x1f00u16)) as i32 >> 8i32)) as u16)
        .wrapping_sub(16u16);
    CreateSpriteAtPos(x, y, *(epjp as *mut u16), 0u16, game);
    epjp.offset(2)
}
pub unsafe fn EprojInstr_QueueSfx2_9(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx2_Max1(9u16, game);
    epjp
}
pub unsafe fn EprojInstr_QueueSfx2_24(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx2_Max1(0x24u16, game);
    epjp
}
pub unsafe fn EprojInstr_QueueSfx2_B(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    QueueSfx2_Max1(0xbu16, game);
    epjp
}
pub unsafe fn EprojInstr_EEAF(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v2: u16 = RandomDropRoutine(k, game);
    if k != 0 && v2 as i32 - 6i32 & 0x8000i32 != 0 {
        let v3: u16 = (2u16).wrapping_mul(v2);
        let v4: i32 = k as i32 >> 1i32;
        set_ram_word!(game.eproj_E[v4] = v3);
        set_ram_word!(
            game.eproj_instr_list_ptr[v4] =
                crate::rtl::rom_table_word!(game, 0x86ef04, (v3 as i32 >> 1i32) as isize)
        );
        set_ram_word!(game.eproj_instr_timers[v4] = 1u16);
        set_ram_word!(game.eproj_F[v4] = 400u16);
        set_ram_word!(game.eproj_pre_instr[v4] = ((0x86efe0i32) as u16));
        update_ram_word!(game.eproj_properties[v4], |fresh25| (fresh25)
            & ((!(0x4000i32)) as u16));
        ram_word!(game, eproj_instr_list_ptr[v4]) as usize as *const u8
    } else {
        let v6: i32 = k as i32 >> 1i32;
        set_ram_word!(game.eproj_instr_timers[v6] = 1u16);
        set_ram_word!(game.eproj_properties[v6] = 12288u16);
        set_ram_word!(game.eproj_pre_instr[v6] = ((0x86efdfi32) as u16));
        set_ram_word!(game.eproj_instr_list_ptr[v6] = 0xeca3u16);
        0xeca3i32 as usize as *const u8
    }
}
pub unsafe fn EprojInstr_HandleRespawningEnemy(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    if (ram_word!(game, eproj_killed_enemy_index[(k as i32 >> 1i32)]) as i16) <= -2i16 {
        RespawnEnemy(
            (ram_word!(game, eproj_killed_enemy_index[(k as i32 >> 1i32)])) & (0x7fffu16),
            game,
        );
    }
    epjp
}
unsafe fn EprojInit_F337(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v2] = game.eproj_spawn_pt.x);
    set_ram_word!(game.eproj_y_pos[v2] = game.eproj_spawn_pt.y);
    set_ram_word!(game.eproj_gfx_idx[v2] = 0u16);
    set_ram_word!(game.eproj_enemy_header_ptr[v2] = game.eproj_spawn_varE24);
    let v3: u16 = RandomDropRoutine(j, game);
    if v3 != 0 && v3 as i32 - 6i32 & 0x8000i32 != 0 {
        let v4: u16 = (2u16).wrapping_mul(v3);
        set_ram_word!(game.eproj_E[v2] = v4);
        set_ram_word!(
            game.eproj_instr_list_ptr[v2] =
                crate::rtl::rom_table_word!(game, 0x86ef04, (v4 as i32 >> 1i32) as isize)
        );
        set_ram_word!(game.eproj_instr_timers[v2] = 1u16);
        set_ram_word!(game.eproj_F[v2] = 400u16);
        set_ram_word!(game.eproj_killed_enemy_index[(j as i32 >> 1i32)] = -1i32 as u16);
    } else {
        let v5: i32 = j as i32 >> 1i32;
        set_ram_word!(game.eproj_instr_list_ptr[v5] = addr_word_86ECA3 as u16);
        set_ram_word!(game.eproj_instr_timers[v5] = 1u16);
        set_ram_word!(game.eproj_properties[v5] = 12288u16);
        set_ram_word!(game.eproj_pre_instr[v5] = (0x86efdfi32 & 0xffffi32) as u16);
    };
}
unsafe fn EprojInit_EnemyDeathExplosion(j: u16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v2] = game.enemy_state(enemy_index).x_pos());
    set_ram_word!(game.eproj_y_pos[v2] = game.enemy_state(enemy_index).y_pos());
    set_ram_word!(game.eproj_killed_enemy_index[v2] = game.cur_enemy_index());
    if (game.enemy_state(enemy_index).properties() as i32 & 0x4000i32) != 0i32 {
        set_ram_word!(game.eproj_killed_enemy_index[v2] = (game.cur_enemy_index()) | (0x8000u16));
    }
    set_ram_word!(game.eproj_enemy_header_ptr[v2] = game.enemy_state(enemy_index).enemy_ptr());
    set_ram_word!(game.eproj_gfx_idx[v2] = 0u16);
    {
        let instruction: u16 = crate::rtl::rom_table_word!(
            game,
            0x86efd5,
            EprojState(&game.g_ram).init_param_1() as isize
        );
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
pub unsafe fn EprojPreInstr_Pickup(k: u16, game: &mut Game) {
    let mut v2: u16 = 0;
    let mut v3: u16 = 0;
    let current_block: u64;
    let v1: i32 = k as i32 >> 1i32;
    let fresh53 = EprojStateMut(&mut game.g_ram).add_word(0x1b23, v1 as usize, u16::MAX);
    if (fresh53 != 0) {
        if CallSomeSamusCode(0xdu16, game) != 0 {
            if ram_word!(game, eproj_F[v1]) as i32 - 384i32 & 0x8000i32 != 0 {
                v2 = 0;
                v3 = 0;
                v2 = abs16(
                    (ram_word!(game, eproj_x_pos[v1]) as i32
                        - ram_word!(game, grapple_beam_end_x_pos) as i32)
                        as u16,
                );
                if v2 as i32 - 16i32 & 0x8000i32 != 0 {
                    v3 = abs16(
                        (ram_word!(game, eproj_y_pos[v1]) as i32
                            - ram_word!(game, grapple_beam_end_y_pos) as i32)
                            as u16,
                    );
                    if v3 as i32 - 16i32 & 0x8000i32 != 0 {
                        off_86F0AD[(ram_word!(game, eproj_E[v1]) as i32 >> 1i32) as usize]
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
        if current_block == 4906268039856690917 {
        } else {
            let v5: i32 = k as i32 >> 1i32;
            let varE20: u16 = EprojState(&game.g_ram).x_radius(v5 as usize) as u16;
            let varE22: u16 = EprojState(&game.g_ram).y_radius(v5 as usize) as u16;
            let v6: u16 = abs16(
                game.samus_x_pos()
                    .wrapping_sub(game.enemy_projectile_x_pos(k)),
            );
            let mut v7: bool = v6 < game.samus_x_radius();
            let v8: u16 = v6.wrapping_sub(game.samus_x_radius());
            if v7 || v8 < varE20 {
                let v9: u16 = abs16(
                    game.samus_y_pos()
                        .wrapping_sub(game.enemy_projectile_y_pos(k)),
                );
                v7 = v9 < game.samus_y_radius();
                let v10: u16 = v9.wrapping_sub(game.samus_y_radius());
                if v7 || v10 < varE22 {
                    off_86F0AD[(ram_word!(game, eproj_E[v5]) as i32 >> 1i32) as usize]
                        .expect("non-null function pointer")(game);
                    let v11: i32 = k as i32 >> 1i32;
                    EprojStateMut(&mut game.g_ram).finish_pickup(v11 as usize);
                }
            }
            return;
        }
    }
    let v4: i32 = k as i32 >> 1i32;
    EprojStateMut(&mut game.g_ram).finish_pickup(v4 as usize);
}
pub unsafe fn Eproj_Pickup_SmallHealth(game: &mut Game) {
    Samus_RestoreHealth(5u16, game);
    QueueSfx2_Max1(1u16, game);
}
pub unsafe fn Eproj_Pickup_BigHealth(game: &mut Game) {
    Samus_RestoreHealth(0x14u16, game);
    QueueSfx2_Max1(2u16, game);
}
pub unsafe fn Eproj_Pickup_PowerBombs(game: &mut Game) {
    Samus_RestorePowerBombs(1u16, game);
    QueueSfx2_Max1(5u16, game);
}
pub unsafe fn Eproj_Pickup_Missiles(game: &mut Game) {
    Samus_RestoreMissiles(2u16, game);
    QueueSfx2_Max1(3u16, game);
}
pub unsafe fn Eproj_Pickup_SuperMissiles(game: &mut Game) {
    Samus_RestoreSuperMissiles(1u16, game);
    QueueSfx2_Max1(4u16, game);
}
static byte_86F25E: [u8; 6] = [1u8, 2u8, 4u8, 6u8, 5u8, 3u8];
pub unsafe fn RandomDropRoutine(k: u16, game: &mut Game) -> u16 {
    let mut v2: u16 = 0;
    let mut Random: u8 = 0;
    let mut v5: u16 = 0;
    let mut v7: *const u8 = null::<u8>();
    let mut i: i32 = 0;
    let current_block: u64;
    let mut v9: i8 = 0;
    let mut r18: u8 = 0;
    let mut r22: u8 = 0;
    let mut r20: u8 = 0;
    let mut r24: u16 = 0;
    let v1: i32 = k as i32 >> 1i32;
    let varE28: u16 = ram_word!(game, eproj_enemy_header_ptr[v1]);
    if (varE28 != 0) {
        v2 = 0;
        v2 = (*get_EnemyDef_A2(varE28, game)).item_drop_chances_ptr;
        if (v2 != 0) {
            Random = 0;
            loop {
                Random = NextRandom(game) as u8;
                if (Random != 0) {
                    break;
                }
            }
            r20 = 255u8;
            r24 = 0u16;
            v5 = 0;
            v5 = 1u16;
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
                    v5 = 0u16;
                    current_block = 5399440093318478209;
                }
            } else {
                current_block = 5399440093318478209;
            }
            if current_block == 5399440093318478209 {
                set_ram_word!(game.health_drop_bias_flag = v5);
            }
            v7 = RomPtr_B4(v2, game);
            if ram_word!(game, health_drop_bias_flag) as u8 != 0 {
                r18 = (*v7.offset(1)).wrapping_add(*v7);
                r22 = 3u8;
            } else {
                r18 = *v7.offset(3);
                r22 = 8u8;
                if game.samus_health() != game.samus_max_health()
                    || game.samus_reserve_health() != game.samus_max_reserve_health()
                {
                    r18 = ((*v7.offset(1) as i32 + *v7 as i32) as u8).wrapping_add(r18);
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
                    r20 = (r20 as i32 - *v7.offset(5) as i32) as u8;
                    r22 = (r22 as i32 | 0x20i32) as u8;
                }
            }
            i = 0;
            i = 0i32;
            while i != 4i32 {
                if r18 == 0 {
                    r22 = (r22 as i32 >> 4i32) as u8;
                    v2 = v2.wrapping_add(4);
                    i = 4i32;
                    break;
                } else {
                    v9 = ((r22) as i8) & (1i8);
                    r22 = (r22 as i32 >> 1i32) as u8;
                    if v9 != 0 {
                        let v10: *const u8 = RomPtr_B4(v2, game);
                        let RegWord: u16 = Mult8x8(r20, *v10);
                        let divved: i32 = SnesDivide(RegWord, r18) as i32;
                        let v12: u16 = r24;
                        let v13: u16 = (divved + v12 as i32) as u16;
                        if v13 >= ((Random) as u16) {
                            return byte_86F25E[i as usize] as u16;
                        }
                        r24 = v13;
                    }
                    v2 = v2.wrapping_add(1);
                    i += 1;
                }
            }
            loop {
                v9 = ((r22) as i8) & (1i8);
                r22 = (r22 as i32 >> 1i32) as u8;
                if v9 != 0 {
                    let v14: *const u8 = RomPtr_B4(v2, game);
                    if (r24).wrapping_add((*v14) as u16) >= ((Random) as u16) {
                        return byte_86F25E[i as usize] as u16;
                    }
                    r24 = (r24 as i32 + *v14 as i32) as u16;
                }
                v2 = v2.wrapping_add(1);
                i += 1;
                if (i == 6i32) {
                    break;
                }
            }
        }
    }
    i = 3i32;
    byte_86F25E[i as usize] as u16
}
pub unsafe fn RespawnEnemy(v0: u16, game: &mut Game) {
    set_ram_word!(game.cur_enemy_index = v0);
    let v1: *const u8 = RomPtr_A1(
        (ram_word!(game, room_enemy_population_ptr)).wrapping_add((v0 as i32 >> 2i32) as u16),
        game,
    );
    let E: *mut EnemyData = gEnemyData(v0 as i32, game);
    (*E).enemy_ptr = *(v1 as *mut u16);
    (*E).x_pos = *(v1.offset(2) as *mut u16);
    (*E).y_pos = *(v1.offset(4) as *mut u16);
    (*E).current_instruction = *(v1.offset(6) as *mut u16);
    (*E).properties = *(v1.offset(8) as *mut u16);
    (*E).extra_properties = *(v1.offset(10) as *mut u16);
    (*E).parameter_1 = *(v1.offset(12) as *mut u16);
    (*E).parameter_2 = *(v1.offset(14) as *mut u16);
    let ESD: *mut EnemySpawnData = gEnemySpawnData(v0 as i32, game);
    (*E).palette_index = (*ESD).palette_index;
    (*E).vram_tiles_index = (*ESD).vram_tiles_index;
    (*E).frozen_timer = 0u16;
    (*E).flash_timer = 0u16;
    (*E).invincibility_timer = 0u16;
    (*E).timer = 0u16;
    (*E).frame_counter = 0u16;
    (*E).ai_var_A = 0u16;
    (*E).ai_var_B = 0u16;
    (*E).ai_var_C = 0u16;
    (*E).ai_var_D = 0u16;
    (*E).ai_var_E = 0u16;
    (*E).ai_preinstr = 0u16;
    (*E).instruction_timer = 1u16;
    let Edef: *mut EnemyDef = get_EnemyDef_A2((*E).enemy_ptr, game);
    (*E).x_width = (*Edef).x_radius;
    (*E).y_height = (*Edef).y_radius;
    (*E).health = (*Edef).health;
    (*E).layer = (*Edef).layer as u16;
    *(&raw mut (*E).bank as *mut u16) = *(&raw mut (*Edef).bank as *mut u16);
    CallEnemyAi(
        (((*Edef).bank as i32) << 16i32 | (*Edef).ai_init as i32) as u32,
        game,
    );
}
unsafe fn EprojInit_Sparks(j: u16, game: &mut Game) {
    static word_86F3D4: [u16; 14] = [
        0xffffu16, 0xb800u16, 0xffffu16, 0xc000u16, 0xffffu16, 0xe000u16, 0xffffu16, 0xff00u16,
        0u16, 0x100u16, 0u16, 0x2000u16, 0u16, 0x4000u16,
    ];
    let v2: i32 = j as i32 >> 1i32;
    set_ram_word!(game.eproj_instr_list_ptr[v2] = addr_kEnemyDef_F353 as u16);
    let v3: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.eproj_x_pos[v2] = (*v3).x_pos);
    set_ram_word!(game.eproj_x_subpos[v2] = (*v3).x_subpos);
    set_ram_word!(game.eproj_y_pos[v2] = ((*v3).y_pos as i32 + 8i32) as u16);
    set_ram_word!(game.eproj_y_subpos[v2] = (*v3).y_subpos);
    set_ram_word!(game.eproj_x_vel[v2] = 0u16);
    set_ram_word!(game.eproj_y_vel[v2] = 0u16);
    let v4: u16 = ((NextRandom(game) as i32 & 0x1ci32) >> 1i32) as u16;
    set_ram_word!(
        game.eproj_F[v2] = crate::rtl::rom_table_word!(game, 0x86f3d4, (v4 as i32 + 0i32) as isize)
    );
    set_ram_word!(
        game.eproj_E[v2] = crate::rtl::rom_table_word!(game, 0x86f3d4, (v4 as i32 + 1i32) as isize)
    );
}
pub unsafe fn EprojPreInstr_Sparks(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if (ram_word!(game, eproj_y_vel[v1]) as i32 & 0x8000i32) == 0i32 {
        if EprojBlockCollisition_Vertical(k, game) as i32 & 1i32 != 0 {
            set_ram_word!(game.eproj_instr_list_ptr[v1] = 0xf363u16);
            set_ram_word!(game.eproj_instr_timers[v1] = 1u16);
            set_ram_word!(
                game.eproj_F[v1] = (ram_word!(game, eproj_F[v1]) as i32 * 2i32
                    + (ram_word!(game, eproj_E[v1]) as i32 >> 15i32))
                    as u16
            );
            update_ram_word!(game.eproj_E[v1], |fresh50| (fresh50 as i32 * 2i32) as u16);
            set_ram_word!(
                game.eproj_F[v1] = (ram_word!(game, eproj_F[v1]) as i32 * 2i32
                    + (ram_word!(game, eproj_E[v1]) as i32 >> 15i32))
                    as u16
            );
            update_ram_word!(game.eproj_E[v1], |fresh51| (fresh51 as i32 * 2i32) as u16);
            set_ram_word!(game.eproj_x_vel[v1] = 0x8000u16);
            set_ram_word!(game.eproj_y_vel[v1] = -1i32 as u16);
            update_ram_word!(game.eproj_y_pos[v1], |fresh52| (fresh52 as i32 - 2i32)
                as u16);
            return;
        }
        let v2: u16 = ram_word!(game, eproj_x_vel[v1]);
        set_ram_word!(game.eproj_x_vel[v1] = (v2 as i32 + 0x4000i32) as u16);
        let v3: u16 = (u16::overflowing_add(v2, 0x4000u16).1 as i32
            + ram_word!(game, eproj_y_vel[v1]) as i32) as u16;
        if v3 < 4u16 {
            set_ram_word!(game.eproj_y_vel[v1] = v3);
        }
    }
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a93) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a6f) as *mut u16).offset(v1 as isize)
            as *mut u16,
        (ram_word!(game, eproj_y_vel[v1]) as u32) << 16i32
            | ram_word!(game, eproj_x_vel[v1]) as u32,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0x1a27) as *mut u16).offset(v1 as isize)
            as *mut u16,
        (ram_word!(game, eproj_F[v1]) as u32) << 16i32 | ram_word!(game, eproj_E[v1]) as u32,
    );
    if (*(&raw mut game.g_ram as *mut u8).offset(0x5b5) as i32 & 3i32) == 0i32 {
        CreateSpriteAtPos(
            ram_word!(game, eproj_x_pos[v1]),
            ram_word!(game, eproj_y_pos[v1]),
            48u16,
            ram_word!(game, eproj_gfx_idx[v1]),
            game,
        );
    }
}
unsafe fn CallEprojInit(ea: u32, j: u16, game: &mut Game) {
    match ea {
        8817209 => Eproj_Init_0x8aaf(j, game),
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
        8817924 => EprojInit_DraygonsGunk(j, game),
        8817984 => EprojInit_DraygonsWallTurretProjs(j, game),
        8818723 => EprojInit_CrocomireProjectile(j, game),
        8818895 => EprojInit_CrocomireSpikeWallPieces(j, game),
        8819334 => EprojInit_CrocomireBridgeCrumbling(j, game),
        8819533 => EprojInit_9634(j, game),
        8819658 => EprojInit_9642_RidleysFireball(j, game),
        8819839 => EprojInit_9660_FireballExplosion(j, game),
        8819872 => EprojInit_9688(j, game),
        8819892 => EprojInit_9696(j, game),
        8819912 => EprojInit_966C(j, game),
        8819932 => EprojInit_967A(j, game),
        8820444 => EprojInit_9734_CeresFallingDebris(j, game),
        8820772 => EprojInit_PhantoonDestroyableFireballs(j, game),
        8821050 => EprojInit_PhantoonStartingFireballs(j, game),
        8821923 => EprojInit_RocksKraidSpits(j, game),
        8821976 => EprojInit_RocksFallingKraidCeiling(j, game),
        8822028 => EprojInit_RocksWhenKraidRises(j, game),
        8822252 => EprojInit_MiniKraidSpit(j, game),
        8822342 => EprojInit_MiniKraidSpikesLeft(j, game),
        8822347 => EprojInit_MiniKraidSpikesRight(j, game),
        8822450 => EprojInit_WalkingLavaSeahorseFireball(j, game),
        8822793 => EprojInit_PirateMotherBrainLaser(j, game),
        8822936 => EprojInit_PirateClaw(j, game),
        8823457 => EprojInit_A379(j, game),
        8823534 => EprojInit_CeresElevatorPad(j, game),
        8823579 => EprojInit_CeresElevatorPlatform(j, game),
        8823715 => EprojPreInstr_PrePhantomRoom(j, game),
        8824275 => EprojInit_BombTorizoLowHealthDrool(j, game),
        8824413 => EprojInit_BombTorizoLowHealthInitialDrool(j, game),
        8824519 => EprojInit_A977(j, game),
        8824566 => EprojInit_BombTorizoExplosiveSwipe(j, game),
        8824676 => EprojInit_BombTorizoStatueBreaking(j, game),
        8824859 => EprojInit_BombTorizoLowHealthExplode(j, game),
        8824945 => EprojInit_BombTorizoDeathExplosion(j, game),
        8825405 => EprojInit_AB07(j, game),
        8825835 => EprojInit_BombTorizosChozoOrbs(j, game),
        8825980 => EprojInit_GoldenTorizosChozoOrbs(j, game),
        8826389 => EprojInit_TorizoSonicBoom(j, game),
        8826620 => EprojInit_WreckedShipChozoSpikeFootsteps(j, game),
        8826691 => EprojInit_TourianStatueDustClouds(j, game),
        8826704 => EprojInit_TourianLandingDustCloudsRightFoot(j, game),
        8826829 => EprojInit_TorizoLandingDustCloudLeftFoot(j, game),
        8826881 => EprojInit_GoldenTorizoEgg(j, game),
        8827342 => EprojInit_GoldenTorizoSuperMissile(j, game),
        8827688 => EprojInit_GoldenTorizoEyeBeam(j, game),
        8828061 => EprojInit_TourianEscapeShaftFakeWallExplode(j, game),
        8828143 => EprojInit_LavaSeahorseFireball(j, game),
        8828461 => EprojInit_EyeDoorProjectile(j, game),
        8828547 => EprojInit_EyeDoorSweat(j, game),
        8829050 => EprojInit_TourianStatueUnlockingParticleWaterSplash(j, game),
        8829070 => EprojInit_TourianStatueEyeGlow(j, game),
        8829109 => EprojInit_TourianStatueUnlockingParticle(j, game),
        8829160 => EprojIni_TourianStatueUnlockingParticleTail(j, game),
        8829176 => EprojInit_TourianStatueSoul(j, game),
        8829246 => EprojInit_TourianStatueBaseDecoration(j, game),
        8829265 => EprojInit_TourianStatueRidley(j, game),
        8829284 => EprojInit_TourianStatuePhantoon(j, game),
        8829744 => sub_86BB30(j, game),
        8829842 => EprojInit_NuclearWaffleBody(j, game),
        8829915 => EprojInit_NorfairLavaquakeRocks(j, game),
        8830364 => EprojInit_ShaktoolAttackMiddleBackCircle(j, game),
        8830370 => EprojInit_BDA2(j, game),
        8830543 => EprojInit_MotherBrainRoomTurrets(j, game),
        8830809 => EprojInit_MotherBrainRoomTurretBullets(j, game),
        8831731 => Eproj_MotherBrainsBlueRingLasers(j, game),
        8832130 => EprojInit_MotherBrainBomb(j, game),
        8832517 => sub_86C605(j, game),
        8832644 => EprojInit_MotherBrainDeathBeemFired(j, game),
        8833034 => EprojInit_MotherBrainRainbowBeam(j, game),
        8833091 => EprojInit_MotherBrainsDrool(j, game),
        8833269 => EprojInit_MotherBrainsDeathExplosion(j, game),
        8833327 => EprojInit_MotherBrainsRainbowBeamExplosion(j, game),
        8833377 => EprojInit_MotherBrainEscapeDoorParticles(j, game),
        8833642 => EprojInit_MotherBrainPurpleBreathBig(j, game),
        8833667 => EprojInit_MotherBrainPurpleBreathSmall(j, game),
        8833782 => EprojInit_TimeBombSetJapaneseText(j, game),
        8833993 => EprojInit_MotherBrainTubeFalling(j, game),
        8834501 => EprojInit_MotherBrainGlassShatteringShard(j, game),
        8834669 => EprojInit_MotherBrainGlassShatteringSparkle(j, game),
        8834960 => EprojInit_KiHunterAcidSpitLeft(j, game),
        8834982 => EprojInit_KiHunterAcidSpitRight(j, game),
        8835208 => EprojInit_KagosBugs(j, game),
        8835642 => EprojInit_MaridiaFloatersSpikes(j, game),
        8835852 => EprojInit_WreckedShipRobotLaserDown(j, game),
        8835886 => EprojInit_WreckedShipRobotLaserHorizontal(j, game),
        8835905 => EprojInit_WreckedShipRobotLaserUp(j, game),
        8836773 => EprojInit_N00bTubeCrack(j, game),
        8836809 => EprojInit_N00bTubeShards(j, game),
        8836980 => EprojInit_N00bTubeReleasedAirBubbles(j, game),
        8837522 => sub_86D992(j, game),
        8837912 => EprojInit_DBF2(j, game),
        8838285 => EprojInit_Spores(j, game),
        8838307 => EprojInit_SporeSpawnStalk(j, game),
        8838356 => EprojInit_SporeSpawners(j, game),
        8838870 => EprojInit_NamiFuneFireball(j, game),
        8839168 => EprojInit_LavaThrownByLavaman(j, game),
        8840296 => EprojInit_DustCloudOrExplosion(j, game),
        8840358 => EprojInit_EyeDoorSmoke(j, game),
        8840656 => EprojInit_SpawnedShotGate(j, game),
        8840661 => EprojInit_ClosedDownwardsShotGate(j, game),
        8840666 => EprojInit_ClosedUpwardsShotGate(j, game),
        8840877 => EprojInit_SaveStationElectricity(j, game),
        8841777 => EprojInit_BotwoonsBody(j, game),
        8842182 => EprojInit_BotwoonsSpit(j, game),
        8842338 => EprojInit_YappingMawsBody(j, game),
        8843049 => EprojInit_F337(j, game),
        8843145 => EprojInit_EnemyDeathExplosion(j, game),
        8844177 => EprojInit_Sparks(j, game),
        _ => {
            Unreachable(game);
        }
    }
}
unsafe fn CallEprojPreInstr(ea: u32, k: u16, game: &mut Game) {
    match ea {
        8825112 => (),
        8814960 => (),
        8815867 => (),
        8817277 => Eproj_PreInit_0x8aaf(k, game),
        8817501 => Eproj_PreInstr_SkreeParticle(k, game),
        8818004 => (),
        8818122 => EprojPreInstr_8DCA(k, game),
        8818175 => EprojPreInstr_DraygonsTurret_8DFF(k, game),
        8818191 => EprojPreInstr_DraygonsGunk_8E0F(k, game),
        8818795 => EprojPreInstr_CrocomireProjectile(k, game),
        8818867 => sub_8690B3(k, game),
        8818965 => EprojPreInstr_CrocomireSpikeWallPieces(k, game),
        8819386 => EprojPreInstr_CrocomireBridgeCrumbling(k, game),
        8819602 => EprojPreInstr_9634(k, game),
        8819726 => EprojPreInstr_9642_RidleysFireball(k, game),
        8819980 => (),
        8819981 => EprojPreInstr_966C(k, game),
        8820002 => EprojPreInstr_9688(k, game),
        8820023 => EprojPreInstr_96A4(k, game),
        8820032 => EprojPreInstr_96C0(k, game),
        8820041 => EprojPreInstr_96CE(k, game),
        8820481 => EprojPreInstr_9734_CeresFallingDebris(k, game),
        8821121 => EprojPreInstr_PhantoonStartingFireballs(k, game),
        8821183 => EprojPreInstr_PhantoonStartingFireballs2(k, game),
        8821249 => EprojPreInstr_PhantoonStartingFireballs3(k, game),
        8821316 => (),
        8821317 => EprojPreInstr_PhantoonDestroyableFireballs(k, game),
        8821396 => EprojPreInstr_PhantoonDestroyableFireballs_2(k, game),
        8821466 => EprojPreInstr_PhantoonDestroyableFireballs_3(k, game),
        8821545 => EprojPreInstr_PhantoonStartingFireballsB(k, game),
        8821569 => EprojPreInstr_PhantoonStartingFireballsB_2(k, game),
        8822102 => EprojPreInstr_KraidRocks(k, game),
        8822153 => EprojPreInstr_RocksFallingKraidCeiling(k, game),
        8822181 => sub_869DA5(k, game),
        8822302 => EprojPreInit_MiniKraidSpit(k, game),
        8822403 => EprojPreInstr_MiniKraidSpikes(k, game),
        8822527 => EprojPreInstr_WalkingLavaSeahorseFireball(k, game),
        8822875 => (),
        8822876 => EprojPreInstr_PirateMotherBrainLaser_MoveLeft(k, game),
        8822906 => EprojPreInstr_PirateMotherBrainLaser_MoveRight(k, game),
        8822993 => EprojPreInstr_PirateClawThrownLeft(k, game),
        8823076 => EprojPreInstr_PirateClawThrownRight(k, game),
        8823591 => (),
        8823592 => EprojPreInstr_CeresElevatorPad(k, game),
        8823652 => EprojPreInstr_CeresElevatorPlatform(k, game),
        8823715 => EprojPreInstr_PrePhantomRoom(k, game),
        8824967 => sub_86A887(k, game),
        8825071 => EprojPreInstr_A977(k, game),
        8825113 => (),
        8825484 => EprojPreInstr_AB07(k, game),
        8826029 => EprojPreInstr_BombTorizosChozoOrbs(k, game),
        8826106 => EprojPreInstr_GoldenTorizosChozoOrbs(k, game),
        8826476 => EprojPreInstr_TorizoSonicBoom(k, game),
        8826947 => EprojPreInstr_GoldenTorizoEgg(k, game),
        8827065 => sub_86B0B9(k, game),
        8827101 => sub_86B0DD(k, game),
        8827405 => EprojPreInstr_GoldenTorizoSuperMissile(k, game),
        8827447 => EprojPreInstr_B237(k, game),
        8827786 => EprojPreInstr_GoldenTorizoEyeBeam(k, game),
        8828080 => (),
        8828213 => sub_86B535(k, game),
        8828601 => EprojPreInstr_EyeDoorProjectile(k, game),
        8828692 => EprojPreInstr_EyeDoorSweat(k, game),
        8829303 => EprojPreInstr_TourianStatueUnlockingParticleWaterSplash(k, game),
        8829314 => EprojPreInstr_TourianStatueUnlockingParticle(k, game),
        8829437 => EprojPreInstr_TourianStatueSoul(k, game),
        8829495 => EprojPreInstr_TourianStatueStuff(k, game),
        8829506 => EprojPreInstr_BA42(k, game),
        8829775 => (),
        8829894 => (),
        8829967 => EprojPreInstr_NorfairLavaquakeRocks(k, game),
        8830238 => EprojPreInstr_NorfairLavaquakeRocks_Inner(k, game),
        8830250 => {
            EprojPreInstr_NorfairLavaquakeRocks_Inner2(k, game);
        }
        8830467 => EprojInit_ShaktoolAttackFrontCircle(k, game),
        8830482 => EprojPreInstr_BE12(k, game),
        8830943 => EprojPreInstr_MotherBrainRoomTurrets(k, game),
        8831200 => EprojPreInstr_MotherBrainRoomTurretBullets(k, game),
        8831797 => Eproj_MoveToBlueRingSpawnPosition(k, game),
        8832200 => EprojPreInstr_MotherBrainBomb(k, game),
        8833044 => EprojPreInstr_MotherBrainRainbowBeam(k, game),
        8833101 => EprojPreInstr_C84D(k, game),
        8833158 => EprojInit_MotherBrainsDrool_Falling(k, game),
        8833300 => EprojPreInit_MotherBrainsDeathExplosion_0(k, game),
        8833356 => EprojPreInstr_MotherBrainsRainbowBeamExplosion(k, game),
        8833490 => EprojPreInstr_MotherBrainsExplodedDoorParticles(k, game),
        8833699 => (),
        8833786 => EprojPreInstr_TimeBombSetJapaneseText(k, game),
        8834023 => EprojPreInstr_MotherBrainTubeFalling(k, game),
        8834715 => EprojPreInstr_MotherBrainGlassShatteringShard(k, game),
        8835029 => sub_86CFD5(k, game),
        8835046 => sub_86CFE6(k, game),
        8835063 => (),
        8835064 => sub_86CFF8(k, game),
        8835251 => EprojPreInstr_KagosBugs_Func1(k, game),
        8835274 => EprojPreInstr_KagosBugs(k, game),
        8835307 => (),
        8835308 => EprojPreInstr_D0EC(k, game),
        8835368 => EprojPreInstr_D128(k, game),
        8835556 => EprojPreInstr_KagosBugs_Func2(k, game),
        8835683 => EprojPreInstr_MaridiaFloatersSpikes(k, game),
        8836031 => EprojPreInstr_WreckedShipRobotLaser(k, game),
        8837055 => sub_86D7BF(k, game),
        8837086 => sub_86D7DE(k, game),
        8837117 => EprojPreInstr_N00bTubeShards(k, game),
        8837181 => sub_86D83D(k, game),
        8837279 => sub_86D89F(k, game),
        8837343 => sub_86D8DF(k, game),
        8837595 => EprojPreInstr_SpikeShootingPlantSpikes(k, game),
        8837606 => EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, game),
        8837648 => EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, game),
        8837690 => EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, game),
        8837732 => EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, game),
        8837774 => EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1(k, game),
        8837779 => EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2(k, game),
        8837784 => EprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1(k, game),
        8837789 => EprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2(k, game),
        8837794 => EprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1(k, game),
        8837802 => EprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2(k, game),
        8837810 => EprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1(k, game),
        8837818 => EprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2(k, game),
        8837979 => EprojPreInstr_DBF2(k, game),
        8837986 => EprojPreInstr_DBF2_MoveX1(k, game),
        8838028 => EprojPreInstr_DBF2_MoveX2(k, game),
        8838070 => EprojPreInstr_DBF2_Func1(k, game),
        8838382 => EprojPreInstr_Spores(k, game),
        8838468 => (),
        8838470 => EprojPreInstr_SporeSpawners(k, game),
        8838969 => EprojPreInstr_NamiFuneFireball(k, game),
        8839241 => sub_86E049(k, game),
        8840446 => EprojPreInstr_DustCloudOrExplosion(k, game),
        8840456 => (),
        8840708 => (),
        8840709 => EprojPreInstr_E605(k, game),
        8840913 => (),
        8841856 => EprojPreInstr_BotwoonsBody(k, game),
        8842245 => EprojPreInstr_BotwoonsSpit(k, game),
        8842388 => (),
        8843231 => (),
        8843232 => EprojPreInstr_Pickup(k, game),
        8844272 => EprojPreInstr_Sparks(k, game),
        8832877 => (),
        _ => {
            Unreachable(game);
        }
    }
}
unsafe fn CallEprojInstr(ea: u32, k: u16, j: *const u8, game: &mut Game) -> *const u8 {
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
            null::<u8>()
        }
    }
}
