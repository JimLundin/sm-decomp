pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_a6::*;
pub use crate::game::constants::bank_a9::*;
pub use crate::game::constants::enemy_properties::*;
use crate::game::enemy_layouts::enemy_layout;
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
use crate::rtl::abs16;
use crate::rtl::{mov24, MemCpy, ReadRegWord, RomPtr, SnesDivide, Unreachable, WriteReg};
pub use crate::snes::registers::*;
extern "C" {
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;

}
pub use crate::types::Consts_8D;
pub use crate::types::Consts_A6;
pub use crate::types::Consts_A9;
pub use crate::types::Consts_B7;
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t,
    CoroutineRet, Func_V, LongPtr, OamEnt, Point16U, Rect16U, SnesRegs, VoidP, VramWriteEntry,
};
pub use crate::types::{Consts_86, ExtraEnemyRam7800, Ram7800_Kraid};
pub use crate::types::{EnemyBase, EnemyData, EnemyProps};
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
enemy_layout! {
    Enemy_Shitroid {
        base: EnemyBase,
        words: [
            shitr_var_A, shitr_var_B, shitr_var_C, shitr_var_D, shitr_var_E, shitr_var_F,
            shitr_parameter_1, shitr_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            shitr_var_00, shitr_var_01, shitr_var_02, shitr_var_03, shitr_var_04,
            shitr_var_05, shitr_var_06, shitr_var_07, shitr_var_08, shitr_var_09,
            shitr_var_0A, shitr_var_0B, shitr_var_0C, shitr_var_0D, shitr_var_0E,
            shitr_var_0F, shitr_var_10, shitr_var_11, shitr_var_12, shitr_var_13,
            shitr_var_14, shitr_var_15, shitr_var_16, shitr_var_17, shitr_var_18,
            shitr_var_19, shitr_var_1A, shitr_var_1B, shitr_var_1C, shitr_var_1D,
            shitr_var_1E, shitr_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_DeadMonsters {
        base: EnemyBase,
        words: [
            dms_var_A, dms_var_B, dms_var_C, dms_var_D, dms_var_E, dms_var_F,
            dms_parameter_1, dms_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            dms_var_00, dms_var_01, dms_var_02, dms_var_03, dms_var_04, dms_var_05,
            dms_var_06, dms_var_07, dms_var_08, dms_var_09, dms_var_0A, dms_var_0B,
            dms_var_0C, dms_var_0D, dms_var_0E, dms_var_0F, dms_var_10, dms_var_11,
            dms_var_12, dms_var_13, dms_var_14, dms_var_15, dms_var_16, dms_var_17,
            dms_var_18, dms_var_19, dms_var_1A, dms_var_1B, dms_var_1C, dms_var_1D,
            dms_var_1E, dms_var_1F,
        ],
        pad3: [uint8; 4032],
        words: [
            dms_var_40, dms_var_41, dms_var_42, dms_var_43, dms_var_44, dms_var_45,
            dms_var_46, dms_var_47, dms_var_48, dms_var_49, dms_var_4A, dms_var_4B,
            dms_var_4C, dms_var_4D, dms_var_4E, dms_var_4F, dms_var_50, dms_var_51,
            dms_var_52, dms_var_53, dms_var_54, dms_var_55, dms_var_56, dms_var_57,
            dms_var_58, dms_var_59, dms_var_5A, dms_var_5B, dms_var_5C, dms_var_5D,
            dms_var_5E, dms_var_5F,
        ],
    }
}

enemy_layout! {
    Enemy_DeadTorizo {
        base: EnemyBase,
        words: [
            dto_var_A, dto_var_B, dto_var_C, dto_var_D, dto_var_E, dto_var_F,
            dto_parameter_1, dto_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            dto_var_00, dto_var_01, dto_var_02, dto_var_03, dto_var_04, dto_var_05,
            dto_var_06, dto_var_07, dto_var_08, dto_var_09, dto_var_0A, dto_var_0B,
            dto_var_0C, dto_var_0D, dto_var_0E, dto_var_0F, dto_var_10, dto_var_11,
            dto_var_12, dto_var_13, dto_var_14, dto_var_15, dto_var_16, dto_var_17,
            dto_var_18, dto_var_19, dto_var_1A, dto_var_1B, dto_var_1C, dto_var_1D,
            dto_var_1E, dto_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_ShitroidInCutscene {
        base: EnemyBase,
        words: [
            sice_var_A, sice_var_B, sice_var_C, sice_var_D, sice_var_E, sice_var_F,
            sice_parameter_1, sice_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            sice_var_00, sice_var_01, sice_var_02, sice_var_03, sice_var_04, sice_var_05,
            sice_var_06, sice_var_07, sice_var_08, sice_var_09, sice_var_0A, sice_var_0B,
            sice_var_0C, sice_var_0D, sice_var_0E, sice_var_0F, sice_var_10, sice_var_11,
            sice_var_12, sice_var_13, sice_var_14, sice_var_15, sice_var_16, sice_var_17,
            sice_var_18, sice_var_19, sice_var_1A, sice_var_1B, sice_var_1C, sice_var_1D,
            sice_var_1E, sice_var_1F,
        ],
        pad2: [uint8; 1984],
        words: [
            sice_var_20, sice_var_21, sice_var_22, sice_var_23, sice_var_24, sice_var_25,
            sice_var_26, sice_var_27, sice_var_28, sice_var_29, sice_var_2A, sice_var_2B,
            sice_var_2C, sice_var_2D, sice_var_2E, sice_var_2F, sice_var_30, sice_var_31,
            sice_var_32, sice_var_33, sice_var_34, sice_var_35, sice_var_36, sice_var_37,
            sice_var_38, sice_var_39, sice_var_3A, sice_var_3B, sice_var_3C, sice_var_3D,
            sice_var_3E, sice_var_3F,
        ],
    }
}

enemy_layout! {
    Enemy_DeadSidehopper {
        base: EnemyBase,
        words: [
            dsr_var_A, dsr_var_B, dsr_var_C, dsr_var_D, dsr_var_E, dsr_var_F,
            dsr_parameter_1, dsr_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            dsr_var_00, dsr_var_01, dsr_var_02, dsr_var_03, dsr_var_04, dsr_var_05,
            dsr_var_06, dsr_var_07, dsr_var_08, dsr_var_09, dsr_var_0A, dsr_var_0B,
            dsr_var_0C, dsr_var_0D, dsr_var_0E, dsr_var_0F, dsr_var_10, dsr_var_11,
            dsr_var_12, dsr_var_13, dsr_var_14, dsr_var_15, dsr_var_16, dsr_var_17,
            dsr_var_18, dsr_var_19, dsr_var_1A, dsr_var_1B, dsr_var_1C, dsr_var_1D,
            dsr_var_1E, dsr_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_DeadEnemy {
        base: EnemyBase,
        words: [
            dey_var_A, dey_var_B, dey_var_C, dey_var_D, dey_var_E, dey_var_F,
            dey_parameter_1, dey_parameter_2,
        ],
        pad2: [uint8; 28744],
        words: [
            dey_var_20, dey_var_21, dey_var_22, dey_var_23, dey_var_24, dey_var_25,
            dey_var_26, dey_var_27, dey_var_28, dey_var_29, dey_var_2A, dey_var_2B,
            dey_var_2C, dey_var_2D, dey_var_2E, dey_var_2F, dey_var_30, dey_var_31,
            dey_var_32, dey_var_33, dey_var_34, dey_var_35, dey_var_36, dey_var_37,
            dey_var_38, dey_var_39, dey_var_3A, dey_var_3B, dey_var_3C, dey_var_3D,
            dey_var_3E, dey_var_3F,
        ],
    }
}

enemy_layout! {
    Enemy_MotherBomb {
        base: EnemyBase,
        words: [
            mbb_var_A, mbb_var_B, mbb_var_C, mbb_var_D, mbb_var_E, mbb_var_F,
            mbb_parameter_1, mbb_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_MotherBrainBody {
        base: EnemyBase,
        words: [
            mbby_var_A, mbby_var_B, mbby_var_C, mbby_var_D, mbby_var_E, mbby_var_F,
            mbby_parameter_1, mbby_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_MotherBrainsTubesFalling {
        base: EnemyBase,
        words: [
            mbtfg_var_A, mbtfg_var_B, mbtfg_var_C, mbtfg_var_D, mbtfg_var_E, mbtfg_var_F,
            mbtfg_parameter_1, mbtfg_parameter_2,
        ],
    }
}

pub const kCoroutineNone: ::core::ffi::c_int = 0;
#[inline]
unsafe fn RomPtr_A9(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(((0xa90000u32) | ((addr) as u32)), game);
}
#[inline]
unsafe fn RomPtr_B7(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(((0xb70000u32) | ((addr) as u32)), game);
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
pub const fnEproj_Init_0x8bde_SkreeDownLeft: ::core::ffi::c_int = 0x868b15;
pub const fnMotherBrainsBody_UnpauseHook: ::core::ffi::c_int = 0xa98763;
pub const fnMotherBrainsBrain_SetupBrainAndNeckToDraw: ::core::ffi::c_int = 0xa987a2;
pub const fnMotherBrainsBrain_SetupBrainToDraw: ::core::ffi::c_int = 0xa987d0;
pub const fnMotherBrainsBody_FirstPhase_DoubleRet: ::core::ffi::c_int = 0xa987e1;
pub const fnMotherBrainBody_FakeDeath_Descent_0_Pause: uint32 = 11110429;
pub const fnMotherBrainBody_FakeDeath_Descent_1: ::core::ffi::c_int = 0xa98829;
pub const fnMotherBrainBody_FakeDeath_Descent_2: ::core::ffi::c_int = 0xa9884d;
pub const fnMotherBrainBody_FakeDeath_Descent_3: ::core::ffi::c_int = 0xa9886c;
pub const fnMotherBrainBody_FakeDeath_Descent_4: ::core::ffi::c_int = 0xa98884;
pub const fnMotherBrainBody_FakeDeath_Descent_5: ::core::ffi::c_int = 0xa988b2;
pub const fnMotherBrainBody_FakeDeath_Descent_6: ::core::ffi::c_int = 0xa988d3;
pub const fnMotherBrainBody_0_Wait: ::core::ffi::c_int = 0xa98949;
pub const fnMotherBrainBody_1_ClearBottomLeftTube: ::core::ffi::c_int = 0xa9896e;
pub const fnMotherBrainBody_2_SpawnTopRightTubeFalling: ::core::ffi::c_int = 0xa98983;
pub const fnMotherBrainBody_3_ClearCeilingBlock9: uint32 = 11110816;
pub const fnMotherBrainBody_4_SpawnTopLeftTubeFalling: ::core::ffi::c_int = 0xa989b5;
pub const fnMotherBrainBody_4_ClearCeilingBlock6: ::core::ffi::c_int = 0xa989d2;
pub const fnMotherBrainBody_5_SpawnTubeFallingEnemy1: ::core::ffi::c_int = 0xa989e7;
pub const fnMotherBrainBody_6_ClearBottomRightTube: ::core::ffi::c_int = 0xa989fa;
pub const fnMotherBrainBody_7_SpawnTubeFallingEnemy2: ::core::ffi::c_int = 0xa98a0f;
pub const fnMotherBrainBody_8_ClearBottomMiddleLeftTube: ::core::ffi::c_int = 0xa98a22;
pub const fnMotherBrainBody_9_SpawnTopMiddleLeftFalling: ::core::ffi::c_int = 0xa98a37;
pub const fnMotherBrainBody_10_ClearCeilingTubeColumn7: ::core::ffi::c_int = 0xa98a54;
pub const fnMotherBrainBody_11_SpawnTopMiddleRightFalling: ::core::ffi::c_int = 0xa98a69;
pub const fnMotherBrainBody_12_ClearCeilingTubeColumn8: ::core::ffi::c_int = 0xa98a86;
pub const fnMotherBrainBody_13_SpawnTubeFallingEnemy3: ::core::ffi::c_int = 0xa98a9b;
pub const fnMotherBrainBody_14_ClearBottomMiddleRightTube: ::core::ffi::c_int = 0xa98aae;
pub const fnMotherBrainBody_15_SpawnTubeFallingEnemy4: ::core::ffi::c_int = 0xa98ac3;
pub const fnMotherBrainBody_16_ClearBottomMiddleTubes: ::core::ffi::c_int = 0xa98ad6;
pub const fnnullsub_365: uint32 = 11111140;
pub const fnMotherBrainsTubesFalling_Main_NonMain: uint32 = 11111304;
pub const fnMotherBrainsTubesFalling_WaitToFall: uint32 = 11111371;
pub const fnMotherBrainsTubesFalling_Falling: ::core::ffi::c_int = 0xa98bd6;
pub const fnMotherBrainBody_FakeDeath_Ascent_0_DrawBG1Row23: ::core::ffi::c_int = 0xa98c87;
pub const fnMotherBrainBody_FakeDeath_Ascent_1_DrawBG1Row45: ::core::ffi::c_int = 0xa98c9e;
pub const fnMotherBrainBody_FakeDeath_Ascent_2_DrawBG1Row67: ::core::ffi::c_int = 0xa98cb5;
pub const fnMotherBrainBody_FakeDeath_Ascent_3_DrawBG1Row89: ::core::ffi::c_int = 0xa98ccc;
pub const fnMotherBrainBody_FakeDeath_Ascent_4_DrawBG1RowAB: ::core::ffi::c_int = 0xa98ce3;
pub const fnMotherBrainBody_FakeDeath_Ascent_5_DrawBG1RowCD: ::core::ffi::c_int = 0xa98cfa;
pub const fnMotherBrainBody_FakeDeath_Ascent_6_SetupPhase2Gfx: ::core::ffi::c_int = 0xa98d11;
pub const fnMotherBrainBody_FakeDeath_Ascent_7_SetupPhase2Brain: ::core::ffi::c_int = 0xa98d49;
pub const fnMotherBrainBody_FakeDeath_Ascent_8_Pause: ::core::ffi::c_int = 0xa98d79;
pub const fnMotherBrainBody_FakeDeath_Ascent_9_PrepareRise: ::core::ffi::c_int = 0xa98d8b;
pub const fnMotherBrainBody_FakeDeath_Ascent_10_LoadLegTiles: ::core::ffi::c_int = 0xa98db4;
pub const fnMotherBrainBody_FakeDeath_Ascent_11_ContinuePause: ::core::ffi::c_int = 0xa98dc3;
pub const fnMotherBrainBody_FakeDeath_Ascent_12_StartMusic: ::core::ffi::c_int = 0xa98dec;
pub const fnMotherBrainBody_FakeDeath_Ascent_13_Raise: ::core::ffi::c_int = 0xa98e4d;
pub const fnMotherBrainBody_FakeDeath_Ascent_14_WaitForUncouching: ::core::ffi::c_int = 0xa98e95;
pub const fnMotherBrainBody_FakeDeath_Ascent_15_TransitionFromGrey: ::core::ffi::c_int = 0xa98eaa;
pub const fnMotherBrainBody_2ndphase_16_ShakeHeadMenacingly: ::core::ffi::c_int = 0xa98ef5;
pub const fnMotherBrainBody_2ndphase_17_BringHeadBackUp: ::core::ffi::c_int = 0xa98f14;
pub const fnMotherBrainBody_2ndphase_18_FinishStretching: ::core::ffi::c_int = 0xa98f33;
pub const fnMotherBrain_Instr_Goto: uint32 = 11115279;
pub const fnMotherBrain_Instr_EnableNeckMovementGoto: uint32 = 11115284;
pub const fnMotherBrain_Instr_DisableNeckMovement: uint32 = 11115296;
pub const fnMotherBrain_Instr_QueueSfx2: uint32 = 11115304;
pub const fnMotherBrain_Instr_QueueSfx3: uint32 = 11115314;
pub const fnMotherBrain_Instr_SpawnDroolEproj: uint32 = 11115324;
pub const fnMotherBrain_Instr_SpawnPurpleBreath: uint32 = 11115373;
pub const fnMotherBrain_Instr_SetMainShakeTimer50: uint32 = 11115383;
pub const fnMotherBrain_Instr_GotoEitherOr: uint32 = 11115621;
pub const fnMotherBrain_Instr_MaybeGoto: uint32 = 11115693;
pub const fnMotherBrain_Instr_MaybeGoto2: uint32 = 11115789;
pub const fnMotherBrain_Instr_Goto2: uint32 = 11115809;
pub const fnMotherBrain_Instr_QueueShitroidAttackSfx: uint32 = 11116023;
pub const fnMotherBrain_Instr_SpawnBlueRingEproj: uint32 = 11116073;
pub const fnMotherBrain_Instr_AimRingsAtShitroid: uint32 = 11116087;
pub const fnMotherBrain_Instr_AimRingsAtSamus: uint32 = 11116123;
pub const fnMotherBrain_Instr_IncrShitroidAttackCtr: uint32 = 11116195;
pub const fnMotherBrain_Instr_SetShitroidAttackCtr0: uint32 = 11116213;
pub const fnMotherBrain_Instr_SpawnBombEproj: uint32 = 11116221;
pub const fnMotherBrain_Instr_SpawnLaserEproj: uint32 = 11116358;
pub const fnMotherBrain_Instr_SpawnRainbowEproj: uint32 = 11116420;
pub const fnMotherBrain_Instr_SetupFxForRainbowBeam: uint32 = 11116430;
pub const fnMotherBrain_Phase3_Death_0: ::core::ffi::c_int = 0xa9aee1;
pub const fnMotherBrain_Phase3_Death_1: ::core::ffi::c_int = 0xa9af12;
pub const fnMotherBrain_Phase3_Death_2: ::core::ffi::c_int = 0xa9af21;
pub const fnMotherBrain_Phase3_Death_3: ::core::ffi::c_int = 0xa9af54;
pub const fnMotherBrain_Phase3_Death_4: ::core::ffi::c_int = 0xa9af9d;
pub const fnMotherBrain_Phase3_Death_5: ::core::ffi::c_int = 0xa9afb6;
pub const fnMotherBrain_Phase3_Death_6: ::core::ffi::c_int = 0xa9b013;
pub const fnMotherBrain_Phase3_Death_7: ::core::ffi::c_int = 0xa9b115;
pub const fnMotherBrain_Phase3_Death_8: ::core::ffi::c_int = 0xa9b12d;
pub const fnMotherBrain_Phase3_Death_9: ::core::ffi::c_int = 0xa9b15e;
pub const fnMotherBrain_Phase3_Death_10: ::core::ffi::c_int = 0xa9b173;
pub const fnMotherBrain_Phase3_Death_11: ::core::ffi::c_int = 0xa9b189;
pub const fnMotherBrain_Phase3_Death_12: ::core::ffi::c_int = 0xa9b1b8;
pub const fnMotherBrain_Phase3_Death_13: ::core::ffi::c_int = 0xa9b1d5;
pub const fnMotherBrain_Phase3_Death_14_20framedelay: ::core::ffi::c_int = 0xa9b211;
pub const fnMotherBrain_CorpseRottingFinished: uint32 = 11121187;
pub const fnMotherBrain_Phase3_Death_15_LoadEscapeTimerTiles: ::core::ffi::c_int = 0xa9b258;
pub const fnMotherBrain_Phase3_Death_16_StartEscape: ::core::ffi::c_int = 0xa9b26d;
pub const fnMotherBrain_Phase3_Death_17_SpawnTimeBomb: ::core::ffi::c_int = 0xa9b2d1;
pub const fnMotherBrain_Phase3_Death_18_TypesZebesText: ::core::ffi::c_int = 0xa9b2e3;
pub const fnMotherBrain_Phase3_Death_19_EscapeDoorExploding: ::core::ffi::c_int = 0xa9b2f9;
pub const fnMotherBrain_Phase3_Death_20_BlowUpEscapeDoor: ::core::ffi::c_int = 0xa9b32a;
pub const fnMotherBrain_Phase3_Death_21_KeepEarthquakeGoing: ::core::ffi::c_int = 0xa9b33c;
pub const fnMotherBrain_Body_Phase2_Thinking: ::core::ffi::c_int = 0xa9b605;
pub const fnMotherBrain_Body_Phase2_TryAttack: ::core::ffi::c_int = 0xa9b64b;
pub const fnMotherBrain_FiringBomb_DecideOnWalking: ::core::ffi::c_int = 0xa9b781;
pub const fnMotherBrain_FiringBomb_WalkingBackwards: ::core::ffi::c_int = 0xa9b7ac;
pub const fnMotherBrain_FiringBomb_Crouch: ::core::ffi::c_int = 0xa9b7c6;
pub const fnMotherBrain_FiringBomb_Fired: ::core::ffi::c_int = 0xa9b7e8;
pub const fnMotherBrain_FiringBomb_Standup: ::core::ffi::c_int = 0xa9b7f8;
pub const fnMotherBomb_FiringLaser_PositionHead: ::core::ffi::c_int = 0xa9b80e;
pub const fnMotherBomb_FiringLaser_PositionHeadSlowlyFire: ::core::ffi::c_int = 0xa9b839;
pub const fnMotherBomb_FiringLaser_FinishAttack: ::core::ffi::c_int = 0xa9b863;
pub const fnMotherBomb_FiringDeathBeam: ::core::ffi::c_int = 0xa9b87d;
pub const fnMotherBomb_FiringRainbowBeam_0: ::core::ffi::c_int = 0xa9b8eb;
pub const fnMotherBomb_FiringRainbowBeam_1_StartCharge: ::core::ffi::c_int = 0xa9b91a;
pub const fnMotherBomb_FiringRainbowBeam_2_RetractNeck: ::core::ffi::c_int = 0xa9b92b;
pub const fnMotherBomb_FiringRainbowBeam_3_Wait: ::core::ffi::c_int = 0xa9b93f;
pub const fnMotherBomb_FiringRainbowBeam_4_ExtendNeckDown: ::core::ffi::c_int = 0xa9b951;
pub const fnMotherBomb_FiringRainbowBeam_5_StartFiring: ::core::ffi::c_int = 0xa9b975;
pub const fnMotherBomb_FiringRainbowBeam_6_MoveSamusToWall: ::core::ffi::c_int = 0xa9b9e5;
pub const fnMotherBomb_FiringRainbowBeam_7_DelayFrame: ::core::ffi::c_int = 0xa9ba00;
pub const fnMotherBomb_FiringRainbowBeam_8_StartDrainSamus: ::core::ffi::c_int = 0xa9ba27;
pub const fnMotherBomb_FiringRainbowBeam_9_DrainingSamus: ::core::ffi::c_int = 0xa9ba3c;
pub const fnMotherBomb_FiringRainbowBeam_10_FinishFiringRainbow: ::core::ffi::c_int = 0xa9ba5e;
pub const fnMotherBomb_FiringRainbowBeam_11_LetSamusFall: ::core::ffi::c_int = 0xa9bac4;
pub const fnMotherBomb_FiringRainbowBeam_12_WaitForSamusHitGround: ::core::ffi::c_int = 0xa9bad1;
pub const fnMotherBomb_FiringRainbowBeam_13_LowerHead: ::core::ffi::c_int = 0xa9badd;
pub const fnMotherBomb_FiringRainbowBeam_14_DecideNextAction: ::core::ffi::c_int = 0xa9bb06;
pub const fnMotherBrain_Phase2Cut_0: ::core::ffi::c_int = 0xa9bd45;
pub const fnMotherBrain_Phase2Cut_1: ::core::ffi::c_int = 0xa9bd98;
pub const fnMotherBrain_Phase2Cut_2: ::core::ffi::c_int = 0xa9bda9;
pub const fnMotherBrain_Phase2Cut_3: ::core::ffi::c_int = 0xa9bdc1;
pub const fnMotherBrain_Phase2Cut_4: ::core::ffi::c_int = 0xa9bdd2;
pub const fnMotherBrain_Phase2Cut_5: ::core::ffi::c_int = 0xa9bded;
pub const fnnullsub_364: ::core::ffi::c_int = 0xa9be1a;
pub const fnMotherBrain_DrainedByShitroid_0: ::core::ffi::c_int = 0xa9be38;
pub const fnMotherBrain_DrainedByShitroid_1: ::core::ffi::c_int = 0xa9be5d;
pub const fnMotherBrain_DrainedByShitroid_2: ::core::ffi::c_int = 0xa9be96;
pub const fnMotherBrain_DrainedByShitroid_3: ::core::ffi::c_int = 0xa9bf0e;
pub const fnMotherBrain_DrainedByShitroid_4: ::core::ffi::c_int = 0xa9bf41;
pub const fnMotherBrain_DrainedByShitroid_5: ::core::ffi::c_int = 0xa9bf56;
pub const fnMotherBrain_DrainedByShitroid_6: ::core::ffi::c_int = 0xa9bf7d;
pub const fnMotherBrain_DrainedByShitroid_7: ::core::ffi::c_int = 0xa9bf95;
pub const fnMotherBrain_PainfulWalkForwards: ::core::ffi::c_int = 0xa9bfd0;
pub const fnMotherBrain_PainfulWalkingForwards: ::core::ffi::c_int = 0xa9bfe8;
pub const fnMotherBrain_PainfulWalkBackwards: ::core::ffi::c_int = 0xa9c004;
pub const fnMotherBrain_PainfulWalkingBackwards: ::core::ffi::c_int = 0xa9c01c;
pub const fnMotherBrain_Phase2_Revive_0: ::core::ffi::c_int = 0xa9c059;
pub const fnMotherBrain_Phase2_Revive_1: ::core::ffi::c_int = 0xa9c066;
pub const fnMotherBrain_Phase2_Revive_2: ::core::ffi::c_int = 0xa9c082;
pub const fnMotherBrain_Phase2_Revive_3: ::core::ffi::c_int = 0xa9c08f;
pub const fnMotherBrain_Phase2_Revive_4: ::core::ffi::c_int = 0xa9c0ba;
pub const fnMotherBrain_Phase2_Revive_5: ::core::ffi::c_int = 0xa9c0e4;
pub const fnMotherBrain_Phase2_Revive_6: ::core::ffi::c_int = 0xa9c0fb;
pub const fnMotherBrain_Phase2_Revive_7: ::core::ffi::c_int = 0xa9c11e;
pub const fnMotherBrain_Phase2_Revive_8: ::core::ffi::c_int = 0xa9c147;
pub const fnMotherBrain_Phase2_MurderShitroid_1: ::core::ffi::c_int = 0xa9c15c;
pub const fnMotherBrain_Phase2_MurderShitroid_2: ::core::ffi::c_int = 0xa9c182;
pub const fnnullsub_367: uint32 = 11125133;
pub const fnMotherBrain_Phase2_PrepareForFinalShitroid: ::core::ffi::c_int = 0xa9c18e;
pub const fnMotherBrain_Phase2_ExecuteFinalkShitroid: ::core::ffi::c_int = 0xa9c19a;
pub const fnnullsub_363: ::core::ffi::c_int = 0xa9c1a6;
pub const fnMotherBrain_Phase3_Recover_MakeDistance: ::core::ffi::c_int = 0xa9c1cf;
pub const fnMotherBrain_Phase3_Recover_SetupForFight: ::core::ffi::c_int = 0xa9c1f0;
pub const fnMotherBrain_Phase3_Fighting_Main: ::core::ffi::c_int = 0xa9c209;
pub const fnMotherBrain_Phase3_Fighting_Cooldown: ::core::ffi::c_int = 0xa9c24e;
pub const fnMotherBrain_Phase3_Walk_TryToInchForward: ::core::ffi::c_int = 0xa9c26a;
pub const fnMotherBrain_Phase3_Walk_RetreatQuickly: ::core::ffi::c_int = 0xa9c2b3;
pub const fnMotherBrain_Phase3_Walk_RetreatSlowly: ::core::ffi::c_int = 0xa9c2d2;
pub const fnMotherBrain_Phase3_Neck_Normal: ::core::ffi::c_int = 0xa9c330;
pub const fnnullsub_369: uint32 = 11125587;
pub const fnMotherBrain_Phase3_Neck_SetupRecoilRecovery: ::core::ffi::c_int = 0xa9c354;
pub const fnMotherBrain_Phase3_Neck_RecoilRecovery: ::core::ffi::c_int = 0xa9c37b;
pub const fnMotherBrain_Phase3_Neck_SetupHyperBeamRecoil: ::core::ffi::c_int = 0xa9c395;
pub const fnMotherBrain_Phase3_Neck_HyperBeamRecoil: ::core::ffi::c_int = 0xa9c3cd;
pub const fnShitroidInCutscene_DashOntoScreen: ::core::ffi::c_int = 0xa9c7cc;
pub const fnShitroidInCutscene_CurveTowardsBrain: ::core::ffi::c_int = 0xa9c7ec;
pub const fnShitroidInCutscene_GetIntoFace: ::core::ffi::c_int = 0xa9c811;
pub const fnShitroidInCutscene_LatchOntoBrain: ::core::ffi::c_int = 0xa9c851;
pub const fnShitroidInCutscene_SetMotherBrainToStumbleBack: ::core::ffi::c_int = 0xa9c879;
pub const fnShitroidInCutscene_ActivateRainbowBeam: ::core::ffi::c_int = 0xa9c889;
pub const fnShitroidInCutscene_BrainTurnsToCorpse: ::core::ffi::c_int = 0xa9c8e2;
pub const fnShitroidInCutscene_StopDraining: ::core::ffi::c_int = 0xa9c915;
pub const fnShitroidInCutscene_LetGoAndSpawnDust: ::core::ffi::c_int = 0xa9c94b;
pub const fnShitroidInCutscene_MoveUpToCeiling: uint32 = 11127129;
pub const fnShitroidInCutscene_MoveToSamus: ::core::ffi::c_int = 0xa9c9c3;
pub const fnShitroidInCutscene_LatchOntoSamus: ::core::ffi::c_int = 0xa9ca66;
pub const fnShitroidInCutscene_HealSamusToFullHealth: ::core::ffi::c_int = 0xa9ca7a;
pub const fnnullsub_368: ::core::ffi::c_int = 0xa9cabc;
pub const fnShitroidInCutscene_IdleUntilToNoHealth: ::core::ffi::c_int = 0xa9cabd;
pub const fnShitroidInCutscene_ReleaseSamus: ::core::ffi::c_int = 0xa9cb13;
pub const fnShitroidInCutscene_StareDownMotherBrain: ::core::ffi::c_int = 0xa9cb2d;
pub const fnShitroidInCutscene_FlyOffScreen: ::core::ffi::c_int = 0xa9cb56;
pub const fnShitroidInCutscene_MoveToFinalChargeStart: ::core::ffi::c_int = 0xa9cb7b;
pub const fnShitroidInCutscene_InitiateFinalCharge: ::core::ffi::c_int = 0xa9cbb3;
pub const fnShitroidInCutscene_FinalCharge: ::core::ffi::c_int = 0xa9cbd8;
pub const fnShitroidInCutscene_ShitroidFinalBelow: ::core::ffi::c_int = 0xa9cc3e;
pub const fnShitroidInCutscene_PlaySamusTheme: ::core::ffi::c_int = 0xa9cc60;
pub const fnShitroidInCutscene_PrepareSamusHyperbeam: ::core::ffi::c_int = 0xa9cc7f;
pub const fnnullsub_338: ::core::ffi::c_int = 0xa9cc98;
pub const fnShitroidInCutscene_DeathSequence: ::core::ffi::c_int = 0xa9cc99;
pub const fnShitroidInCutscene_UnloadShitroid: ::core::ffi::c_int = 0xa9ccc0;
pub const fnShitroidInCutscene_LetSamusRainbowMore: ::core::ffi::c_int = 0xa9ccde;
pub const fnShitroidInCutscene_FinishCutscene: ::core::ffi::c_int = 0xa9ccf0;
pub const fnSamusRainbowPaletteFunc_ActivateWhenEnemyLow: ::core::ffi::c_int = 0xa9cd30;
pub const fnSamusRainbowPaletteFunc_GraduallySlowDown: ::core::ffi::c_int = 0xa9cd4b;
pub const fnDeadTorizo_WaitForSamusColl: ::core::ffi::c_int = 0xa9d3ad;
pub const fnnullsub_361: ::core::ffi::c_int = 0xa9d3c7;
pub const fnDeadTorizo_PreRotDelay: ::core::ffi::c_int = 0xa9d3c8;
pub const fnDeadTorizo_Rotting: ::core::ffi::c_int = 0xa9d3e6;
pub const fnDeadTorizo_CorpseRottingFinished: uint32 = 11130301;
pub const fnDeadSidehopper_Alive_WaitForActivate: ::core::ffi::c_int = 0xa9d8e2;
pub const fnDeadSidehopper_Activated: ::core::ffi::c_int = 0xa9d8f1;
pub const fnnullsub_362: ::core::ffi::c_int = 0xa9d90f;
pub const fnDeadMonsters_Func_1: ::core::ffi::c_int = 0xa9d910;
pub const fnDeadMonsters_Func_2: ::core::ffi::c_int = 0xa9d91d;
pub const fnDeadMonsters_Func_5: ::core::ffi::c_int = 0xa9da08;
pub const fnnullsub_268: ::core::ffi::c_int = 0xa9da63;
pub const fnDeadSidehopper_WaitForSamusColl: ::core::ffi::c_int = 0xa9da64;
pub const fnDeadZoomer_WaitForSamusColl: ::core::ffi::c_int = 0xa9da69;
pub const fnDeadSkree_WaitForSamusColl: ::core::ffi::c_int = 0xa9da6e;
pub const fnDeadRipper_WaitForSamusColl: ::core::ffi::c_int = 0xa9da73;
pub const fnDeadSidehopper_PreRotDelay: ::core::ffi::c_int = 0xa9da8f;
pub const fnDeadZoomer_PreRotDelay: ::core::ffi::c_int = 0xa9da94;
pub const fnDeadRipper_PreRotDelay: ::core::ffi::c_int = 0xa9da99;
pub const fnDeadSkree_PreRotDelay: ::core::ffi::c_int = 0xa9da9e;
pub const fnDeadSidehopper_Rotting: ::core::ffi::c_int = 0xa9daba;
pub const fnDeadZoomer_Rotting: ::core::ffi::c_int = 0xa9dad0;
pub const fnDeadRipper_Rotting: ::core::ffi::c_int = 0xa9dae6;
pub const fnDeadSkree_Rotting: ::core::ffi::c_int = 0xa9dafc;
pub const fnCorpseRottingRotEntryFinishedHook: uint32 = 11131912;
pub const fnTorizo_CorpseRottingInitFunc: uint32 = 11132440;
pub const fnSidehopper_CorpseRottingInitFunc_0: uint32 = 11132609;
pub const fnSidehopper_CorpseRottingInitFunc_2: uint32 = 11132680;
pub const fnZoomer_CorpseRottingInitFunc_0: uint32 = 11132751;
pub const fnZoomer_CorpseRottingInitFunc_2: uint32 = 11132780;
pub const fnZoomer_CorpseRottingInitFunc_4: uint32 = 11132809;
pub const fnRipper_CorpseRottingInitFunc_0: uint32 = 11132838;
pub const fnRipper_CorpseRottingInitFunc_2: uint32 = 11132867;
pub const fnSkree_CorpseRottingInitFunc_0: uint32 = 11132896;
pub const fnSkree_CorpseRottingInitFunc_2: uint32 = 11132953;
pub const fnSkree_CorpseRottingInitFunc_4: uint32 = 11133010;
pub const fnMotherBrain_CorpseRottingInitFunc: uint32 = 11133067;
pub const fnTorizo_CorpseRottingMoveFunc: uint32 = 11133554;
pub const fnTorizo_CorpseRottingCopyFunc: uint32 = 11133835;
pub const fnSidehopper_CorpseRottingMoveFunc_0: uint32 = 11134056;
pub const fnSidehopper_CorpseRottingCopyFunc_0: uint32 = 11134197;
pub const fnSidehopper_CorpseRottingMoveFunc_2: uint32 = 11134308;
pub const fnSidehopper_CorpseRottingCopyFunc_2: uint32 = 11134454;
pub const fnZoomer_CorpseRottingMoveFunc_0: uint32 = 11134570;
pub const fnZoomer_CorpseRottingCopyFunc_0: uint32 = 11134649;
pub const fnZoomer_CorpseRottingMoveFunc_2: uint32 = 11134710;
pub const fnZoomer_CorpseRottingCopyFunc_2: uint32 = 11134789;
pub const fnZoomer_CorpseRottingMoveFunc_4: uint32 = 11134850;
pub const fnZoomer_CorpseRottingCopyFunc_4: uint32 = 11134929;
pub const fnRipper_CorpseRottingMoveFunc_0: uint32 = 11134990;
pub const fnRipper_CorpseRottingCopyFunc_0: uint32 = 11135069;
pub const fnRipper_CorpseRottingMoveFunc_2: uint32 = 11135130;
pub const fnRipper_CorpseRottingCopyFunc_2: uint32 = 11135209;
pub const fnSkree_CorpseRottingMoveFunc_0: uint32 = 11135270;
pub const fnSkree_CorpseRottingCopyFunc_0: uint32 = 11135323;
pub const fnSkree_CorpseRottingMoveFunc_2: uint32 = 11135364;
pub const fnSkree_CorpseRottingCopyFunc_2: uint32 = 11135417;
pub const fnSkree_CorpseRottingMoveFunc_4: uint32 = 11135458;
pub const fnSkree_CorpseRottingCopyFunc_4: uint32 = 11135511;
pub const fnMotherBrain_CorpseRottingMoveFunc: uint32 = 11135552;
pub const fnMotherBrain_CorpseRottingCopyFunc: uint32 = 11135755;
pub const fnShitroid_Func_3: ::core::ffi::c_int = 0xa9efdf;
pub const fnShitroid_Func_4: ::core::ffi::c_int = 0xa9efe6;
pub const fnShitroid_Func_5: ::core::ffi::c_int = 0xa9f02b;
pub const fnShitroid_Func_6: ::core::ffi::c_int = 0xa9f037;
pub const fnShitroid_Func_7: ::core::ffi::c_int = 0xa9f049;
pub const fnShitroid_Func_8: ::core::ffi::c_int = 0xa9f06d;
pub const fnShitroid_Func_9: ::core::ffi::c_int = 0xa9f094;
pub const fnShitroid_Func_10: ::core::ffi::c_int = 0xa9f0e6;
pub const fnShitroid_Func_11: ::core::ffi::c_int = 0xa9f125;
pub const fnShitroid_Func_12: ::core::ffi::c_int = 0xa9f138;
pub const fnShitroid_Func_13: ::core::ffi::c_int = 0xa9f180;
pub const fnShitroid_Func_14: ::core::ffi::c_int = 0xa9f1fa;
pub const fnShitroid_Func_15: ::core::ffi::c_int = 0xa9f20e;
pub const fnShitroid_Func_16: ::core::ffi::c_int = 0xa9f21b;
pub const fnShitroid_Func_17: ::core::ffi::c_int = 0xa9f2a2;
pub const fnShitroid_Func_18: ::core::ffi::c_int = 0xa9f2ae;
pub const fnShitroid_Func_19: ::core::ffi::c_int = 0xa9f2c0;
pub const fnShitroid_Func_20: ::core::ffi::c_int = 0xa9f2fb;
pub const fnShitroid_Func_21: ::core::ffi::c_int = 0xa9f324;
pub const fnShitroid_Func_22: ::core::ffi::c_int = 0xa9f360;
pub const fnShitroid_Func_23: ::core::ffi::c_int = 0xa9f36d;
pub const fnShitroid_Func_24: ::core::ffi::c_int = 0xa9f3a3;
pub const fnShitroid_Func_25: ::core::ffi::c_int = 0xa9f3be;
pub const fnShitroid_GraduallyAccelerateTowards0x8: uint32 = 11138143;
pub const fnShitroid_GraduallyAccelerateTowards0x10: uint32 = 11138150;
pub const fnShitroid_HandleCutscenePalette: ::core::ffi::c_int = 0xa9f683;
pub const fnShitroid_HandleCutscenePalette_LowHealth: ::core::ffi::c_int = 0xa9f68f;
#[inline]
unsafe fn gEnemyData(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemyData {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut EnemyData;
}
#[inline]
unsafe fn Get_DeadEnemy(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_DeadEnemy {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_DeadEnemy;
}
#[inline]
unsafe fn Get_DeadMonsters(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_DeadMonsters {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_DeadMonsters;
}
#[inline]
unsafe fn Get_DeadSidehopper(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_DeadSidehopper {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_DeadSidehopper;
}
#[inline]
unsafe fn Get_DeadTorizo(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_DeadTorizo {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_DeadTorizo;
}
#[inline]
unsafe fn Get_MotherBomb(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_MotherBomb {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MotherBomb;
}
#[inline]
unsafe fn Get_MotherBrain(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_MotherBrain {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MotherBrain;
}
#[inline]
unsafe fn Get_MotherBrainBody(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MotherBrainBody {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MotherBrainBody;
}
#[inline]
unsafe fn Get_MotherBrainsTubesFalling(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MotherBrainsTubesFalling {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MotherBrainsTubesFalling;
}
#[inline]
unsafe fn Get_ShitdroidInCutscene(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_ShitroidInCutscene {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_ShitroidInCutscene;
}
#[inline]
unsafe fn Get_Shitroid(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Shitroid {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Shitroid;
}
#[inline]
unsafe fn Get_ShitroidInCutscene(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_ShitroidInCutscene {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_ShitroidInCutscene;
}
static g_word_A98B5D: [uint16; 5] = [
    0x10 as uint16,
    0x10 as uint16,
    8 as uint16,
    8 as uint16,
    0x10 as uint16,
];
static g_word_A98B67: [uint16; 5] = [
    0x20 as uint16,
    0x20 as uint16,
    0x18 as uint16,
    0x18 as uint16,
    0x20 as uint16,
];
static g_word_A98B71: [uint16; 5] = [
    0xf8 as uint16,
    0xf8 as uint16,
    0xf0 as uint16,
    0xf0 as uint16,
    0xf6 as uint16,
];
static g_word_A98C61: [uint16; 4] = [0xfff8 as uint16, 2 as uint16, 0xfffc as uint16, 6 as uint16];
static g_byte_A98F7D: [uint8; 2] = [9 as uint8, 0x12 as uint8];
static g_word_A98F7F: [uint16; 8] = [
    0x3d as uint16,
    0x54 as uint16,
    0x20 as uint16,
    0x35 as uint16,
    0x5a as uint16,
    0x43 as uint16,
    0x67 as uint16,
    0x29 as uint16,
];
static g_word_A993BB: [int16; 4] = [
    0 as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0 as int16,
    1 as int16,
];
static g_word_A993C3: [int16; 4] = [
    0 as int16,
    1 as int16,
    -(1 as ::core::ffi::c_int) as int16,
    1 as int16,
];
static g_word_A99E0F: [uint16; 13] = [
    0x6f as uint16,
    0x6f as uint16,
    0x6f as uint16,
    0x7e as uint16,
    0x6f as uint16,
    0x6f as uint16,
    0x7e as uint16,
    0x6f as uint16,
    0x6f as uint16,
    0x7e as uint16,
    0x7e as uint16,
    0x6f as uint16,
    0x6f as uint16,
];
static g_word_A9B393: [uint16; 8] = [
    8 as uint16,
    0x6c as uint16,
    0x18 as uint16,
    0x80 as uint16,
    9 as uint16,
    0x90 as uint16,
    0x18 as uint16,
    0x74 as uint16,
];
static g_byte_A9B546: [uint8; 8] = [
    0 as uint8, 1 as uint8, 1 as uint8, 0 as uint8, 0 as uint8, 0 as uint8, 0 as uint8, 0 as uint8,
];
static g_byte_A9B5A1: [uint8; 8] = [
    2 as uint8, 1 as uint8, 1 as uint8, 0 as uint8, 0 as uint8, 0 as uint8, 0 as uint8, 0 as uint8,
];
static g_word_A9B72C: [int16; 28] = [
    -(1 as ::core::ffi::c_int) as int16,
    -(17 as ::core::ffi::c_int) as int16,
    0 as int16,
    0 as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0 as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0 as int16,
    0 as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0 as int16,
    0 as int16,
    0 as int16,
    -(17 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    -(1 as ::core::ffi::c_int) as int16,
    0 as int16,
    0 as int16,
    0 as int16,
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
    6 as int16,
    -(4 as ::core::ffi::c_int) as int16,
    2 as int16,
    3 as int16,
    -(6 as ::core::ffi::c_int) as int16,
    8 as int16,
    0 as int16,
];
static g_word_A9BCB6: [int16; 8] = [
    -(7 as ::core::ffi::c_int) as int16,
    2 as int16,
    5 as int16,
    -(4 as ::core::ffi::c_int) as int16,
    6 as int16,
    -(2 as ::core::ffi::c_int) as int16,
    -(6 as ::core::ffi::c_int) as int16,
    7 as int16,
];
static g_byte_A9BEEE: [uint8; 16] = [
    2 as uint8,
    0 as uint8,
    2 as uint8,
    0 as uint8,
    6 as uint8,
    0 as uint8,
    6 as uint8,
    0 as uint8,
    8 as uint8,
    0 as uint8,
    8 as uint8,
    0 as uint8,
    10 as uint8,
    0 as uint8,
    10 as uint8,
    0 as uint8,
];
static g_word_A9BEFE: [uint16; 8] = [
    0x500 as uint16,
    0x500 as uint16,
    0x200 as uint16,
    0x200 as uint16,
    0xc0 as uint16,
    0xc0 as uint16,
    0x40 as uint16,
    0x40 as uint16,
];
static g_word_A9C049: [int16; 8] = [
    0x10 as int16,
    0x10 as int16,
    0x20 as int16,
    0x20 as int16,
    0x30 as int16,
    0x30 as int16,
    0x40 as int16,
    0x40 as int16,
];
static g_word_A9C544: uint16 = 1 as uint16;
static g_byte_A9F56A: [int8; 16] = [
    0x10 as int8,
    0xf as int8,
    0xe as int8,
    0xd as int8,
    0xc as int8,
    0xb as int8,
    0xa as int8,
    9 as int8,
    8 as int8,
    7 as int8,
    6 as int8,
    5 as int8,
    4 as int8,
    3 as int8,
    2 as int8,
    1 as int8,
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9C13 as uint16, game);
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
    (*E1).mbn_var_A = (((0xa987d0i32) as u16) & (0xffffu16));
    (*E).mbn_var_A = (((0xa987e1i32) as u16) & (0xffffu16));
    LoadFxEntry(1 as uint16, game);
    let mut v3: uint16 = 0 as uint16;
    let mut v4: uint16 = 0;
    loop {
        v4 = v3;
        SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainRoomTurrets as uint16, v3, game);
        v3 = ((v4).wrapping_add((1u16)));
        if !((((v4).wrapping_add((1u16))) as ::core::ffi::c_int) < 0xc as ::core::ffi::c_int) {
            break;
        }
    }
}
pub unsafe fn MotherBrainsBrain_Init(game: &mut Game) {
    InitializeEnemyCorpseRotting(0x40 as uint16, addr_stru_A9DE08 as uint16, game);
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).base.health = 3000 as uint16;
    MotherBrain_SetBrainUnusedInstrs(addr_kMotherBrain_Ilist_9C13 as uint16, game);
    (*E1).base.vram_tiles_index = 0 as uint16;
    (*E1).base.properties = (((*E1).base.properties) | (0x1100u16));
    (*E1).base.palette_index = 512 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_0C = 512 as uint16;
    (*E).mbn_var_0D = 512 as uint16;
    MotherBrain_SetBrainInstrs(addr_stru_A99C21 as uint16, game);
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
    CallMotherBrainFunc(((((*E).mbn_var_A) as u32) | (0xa90000u32)), game);
    MotherBrain_HandlePalette(game);
    MotherBrain_SamusCollDetect(game);
    MotherBrain_Pal_ProcessInvincibility(game);
    if (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_02 != 0 {
        mov24(
            (&raw mut game.g_ram as *mut uint8).offset(0x604) as *mut LongPtr,
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x179a) as *mut uint16) = 2048 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe1e) as *mut uint16) = 1 as uint16;
    }
    return kCoroutineNone as CoroutineRet;
}
pub unsafe fn MotherBrainsBody_Powerbomb(game: &mut Game) {
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
    MotherBrainsBrain_Hurt(game);
}
pub unsafe fn MotherBrainsBrain_Hurt(game: &mut Game) {
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr,
        0xa98786 as uint32,
    );
    let mut v0: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
    if (*v0.offset(1)).properties as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        CallMotherBrainFunc(((((*v0.offset(1)).ai_var_A) as u32) | (0xa90000u32)), game);
    }
}
pub unsafe fn MotherBrainsBrain_SetupBrainAndNeckToDraw(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) == 0 {
        MotherBrain_HandleNeck(game);
    }
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).base.x_pos = (*E1).mbn_var_2E;
    (*E1).base.y_pos = (((*E1).mbn_var_2F).wrapping_sub((21u16)));
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr,
        0xa987c9 as uint32,
    );
}
pub unsafe fn MotherBrain_DrawBrainNeck_EnemyGfxDrawHook(game: &mut Game) {
    MotherBrain_DrawBrain(game);
    MotherBrain_DrawNeck(game);
}
pub unsafe fn MotherBrainsBrain_SetupBrainToDraw(game: &mut Game) {
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr,
        0xa987dd as uint32,
    );
}
pub unsafe fn MotherBrainsBrain_GfxDrawHook(game: &mut Game) {
    MotherBrain_DrawBrain(game);
}
pub unsafe fn MotherBrainsBody_FirstPhase_DoubleRet(game: &mut Game) {
    if CheckEventHappened(2 as uint16, game) != 0 {
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_00 = *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16);
        if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
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
    (*E).mbby_var_A = (((0xa98829i32) as u16) & (0xffffu16));
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
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20) as *mut uint16) =
            *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) as uint16;
        (*E).mbby_var_A = (((0xa9884di32) as u16) & (0xffffu16));
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
        (*E).mbby_var_A = (((0xa9886ci32) as u16) & (0xffffu16));
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
        (*E).mbby_var_A = (((0xa98884i32) as u16) & (0xffffu16));
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
            (((0xa98949i32) as u16) & (0xffffu16));
        (*E).mbn_var_A = (((0xa988b2i32) as u16) & (0xffffu16));
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
        let mut v2: uint16 = (((*E).mbn_var_37).wrapping_add((1u16)));
        (*E).mbn_var_37 = v2;
        if MotherBrain_FadeToGray_FakeDeath(((v2).wrapping_sub((1u16))), game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            (*E).mbn_var_A = (((0xa988d3i32) as u16) & (0xffffu16));
        }
    }
    MotherBrainBody_FakeDeath_Descent_6(game);
}
pub unsafe fn MotherBrainBody_FakeDeath_Descent_6(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    CallMotherBrainFunc(((((*E).mbby_var_E) as u32) | (0xa90000u32)), game);
    MotherBrain_HandleFakeDeathExplosions(game);
}
pub unsafe fn MotherBrain_HandleFakeDeathExplosions(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    v1 = ((((*E).mbn_var_38) as i16).wrapping_sub((1i16)));
    if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).mbn_var_38 = 8 as uint16;
        v2 = ((((*E).mbn_var_39) as i16).wrapping_sub((1i16)));
        if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            v2 = 7 as int16;
        }
        (*E).mbn_var_39 = v2 as uint16;
        let mut v3: ::core::ffi::c_int =
            ((4u16).wrapping_mul(((v2) as u16))) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
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
        let mut v4: uint16 = (if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5)
            as *mut uint16) as ::core::ffi::c_int)
            < 0x4000 as ::core::ffi::c_int
        {
            12 as ::core::ffi::c_int
        } else {
            3 as ::core::ffi::c_int
        }) as uint16;
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, v4, game);
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
        if *(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16)
            == 0
        {
            v1 += 1;
        }
        v0 = ((v0).wrapping_add((2u16)));
        if !(((((v0) as i16).wrapping_sub((36i16))) as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    if ((v1).wrapping_sub((4i16))) as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        SpawnEnemy(0xa9 as uint8, addr_stru_A98AE5 as uint16, game);
        (*Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game)).mbby_var_E =
            (((0xa9896ei32) as u16) & (0xffffu16));
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
    (*E).mbby_var_E = (((0xa98983i32) as u16) & (0xffffu16));
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
    (*E).mbby_var_E = (((0xa989b5i32) as u16) & (0xffffu16));
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
            addr_kEproj_MotherBrainTubeFalling_TopLeft as uint16,
            0x2f as uint16,
            game,
        );
        (*E).mbby_var_E = (((0xa989d2i32) as u16) & (0xffffu16));
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
    (*E).mbby_var_E = (((0xa989e7i32) as u16) & (0xffffu16));
    (*E).mbby_var_F = 32 as uint16;
}
pub unsafe fn MotherBrainBody_5_SpawnTubeFallingEnemy1(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if (*E).mbby_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        SpawnEnemy(0xa9 as uint8, addr_stru_A98AF5 as uint16, game);
        (*E).mbby_var_E = (((0xa989fai32) as u16) & (0xffffu16));
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
    (*E).mbby_var_E = (((0xa98a0fi32) as u16) & (0xffffu16));
    (*E).mbby_var_F = 32 as uint16;
}
pub unsafe fn MotherBrainBody_7_SpawnTubeFallingEnemy2(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if (*E).mbby_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        SpawnEnemy(0xa9 as uint8, addr_stru_A98B05 as uint16, game);
        (*E).mbby_var_E = (((0xa98a22i32) as u16) & (0xffffu16));
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
    (*E).mbby_var_E = (((0xa98a37i32) as u16) & (0xffffu16));
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
            addr_kEproj_MotherBrainTubeFalling_TopMiddleLeft as uint16,
            0x3b as uint16,
            game,
        );
        (*E).mbby_var_E = (((0xa98a54i32) as u16) & (0xffffu16));
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
    (*E).mbby_var_E = (((0xa98a69i32) as u16) & (0xffffu16));
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
            addr_kEproj_MotherBrainTubeFalling_TopMiddleRight as uint16,
            0x3b as uint16,
            game,
        );
        (*E).mbby_var_E = (((0xa98a86i32) as u16) & (0xffffu16));
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
    (*E).mbby_var_E = (((0xa98a9bi32) as u16) & (0xffffu16));
    (*E).mbby_var_F = 32 as uint16;
}
pub unsafe fn MotherBrainBody_13_SpawnTubeFallingEnemy3(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if (*E).mbby_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        SpawnEnemy(0xa9 as uint8, (((0x868b15i32) as u16) & (0xffffu16)), game);
        (*E).mbby_var_E = (((0xa98aaei32) as u16) & (0xffffu16));
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
    (*E).mbby_var_E = (((0xa98ac3i32) as u16) & (0xffffu16));
    (*E).mbby_var_F = 2 as uint16;
}
pub unsafe fn MotherBrainBody_15_SpawnTubeFallingEnemy4(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if (*E).mbby_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        SpawnEnemy(0xa9 as uint8, addr_stru_A98B25 as uint16, game);
        (*E).mbby_var_E = (((0xa98ad6i32) as u16) & (0xffffu16));
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
        addr_locret_A98AE4 as uint16;
}
pub unsafe fn MotherBrainsTubesFalling_Init(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainsTubesFalling = Get_MotherBrainsTubesFalling(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
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
    CallMotherBrainsTubesFalling(((((*E).mbtfg_var_A) as u32) | (0xa90000u32)), k, game);
}
pub unsafe fn MotherBrainsTubesFalling_Main_NonMain(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainsTubesFalling =
        Get_MotherBrainsTubesFalling(k as ::core::ffi::c_int, game);
    let mut v2: uint16 = (((*E).mbtfg_var_C).wrapping_add((6u16)));
    (*E).mbtfg_var_C = v2;
    Enemy_IncreaseYpos(k, v2, game);
    if (((((*E).base.y_pos) as i16).wrapping_sub((((*E).mbtfg_var_B) as i16)))
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
    (*E).base.properties = (((*E).base.properties) | ((kEnemyProps_Deleted) as u16));
    game.eproj_spawn_pt = Point16U {
        x: (*E).base.x_pos,
        y: (*E).base.y_pos,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, 3 as uint16, game);
    QueueSfx2_Max3(0x24 as uint16, game);
}
pub unsafe fn MotherBrainsTubesFalling_WaitToFall(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainsTubesFalling =
        Get_MotherBrainsTubesFalling(k as ::core::ffi::c_int, game);
    (*E).mbtfg_parameter_2 = (*E).mbtfg_parameter_2.wrapping_sub(1);
    if (*E).mbtfg_parameter_2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbtfg_var_A = (((0xa98bd6i32) as u16) & (0xffffu16));
        MotherBrainsTubesFalling_Falling(k, game);
    }
}
pub unsafe fn MotherBrainsTubesFalling_Falling(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainsTubesFalling =
        Get_MotherBrainsTubesFalling(k as ::core::ffi::c_int, game);
    let mut v2: uint16 = (((*E).mbtfg_var_C).wrapping_add((6u16)));
    (*E).mbtfg_var_C = v2;
    Enemy_IncreaseYpos(k, v2, game);
    let mut ypos: uint16 = (*E).base.y_pos;
    if ypos as ::core::ffi::c_int - 244 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        (*E).base.properties = (((*E).base.properties) | ((kEnemyProps_Invisible) as u16));
    }
    let mut E1: *mut Enemy_MotherBrainsTubesFalling =
        Get_MotherBrainsTubesFalling(0x40 as ::core::ffi::c_int, game);
    (*E1).base.y_pos = ((ypos).wrapping_sub((56u16)));
    if (*E1).base.y_pos as ::core::ffi::c_int - 0xc4 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        MotherBrainsTubesFalling_HandleSmoke(k, game);
    } else {
        MotherBrain_Pal_EndScreenFlash(game);
        EnableEarthquakeAframes(0x19 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x18b4) as *mut uint16).offset(0) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18b4) as *mut uint16).offset(1) =
            0 as uint16;
        (*E1).base.y_pos = 196 as uint16;
        let mut E0: *mut Enemy_MotherBrainsTubesFalling =
            Get_MotherBrainsTubesFalling(0 as ::core::ffi::c_int, game);
        (*E0).base.x_pos = 59 as uint16;
        (*E0).base.y_pos = 279 as uint16;
        MotherBrain_SetupNeckForFakeAscent(game);
        (*E0).mbtfg_var_A = (((0xa98c87i32) as u16) & (0xffffu16));
        MotherBrainsTubesFalling_Explode(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
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
        x: (((*E).base.x_pos).wrapping_add((g_word_A98C61[v2 as usize]))),
        y: 208 as uint16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, 9 as uint16, game);
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
        (((0xa98c9ei32) as u16) & (0xffffu16));
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
        (((0xa98cb5i32) as u16) & (0xffffu16));
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
        (((0xa98ccci32) as u16) & (0xffffu16));
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
        (((0xa98ce3i32) as u16) & (0xffffu16));
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
        (((0xa98cfai32) as u16) & (0xffffu16));
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
        (((0xa98d11i32) as u16) & (0xffffu16));
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_6_SetupPhase2Gfx(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x91b) as *mut uint16) = 257 as uint16;
    let ref mut fresh8 = *((&raw mut game.g_ram as *mut uint8).offset(0x59) as *mut uint16);
    *fresh8 = ((*fresh8) & (0xfffcu16));
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
        (((0xa98d49i32) as u16) & (0xffffu16));
    *((&raw mut game.g_ram as *mut uint8).offset(0xe1e) as *mut uint16) = 1 as uint16;
    (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_02 = 1 as uint16;
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_7_SetupPhase2Brain(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1982) as *mut uint16) = 52 as uint16;
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40 as ::core::ffi::c_int, game);
    (*E).mbby_var_A = (((0xa987a2i32) as u16) & (0xffffu16));
    let mut E0: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0 as ::core::ffi::c_int, game);
    (*E0).base.properties = ((*E0).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
    (*E).base.properties =
        ((*E).base.properties as ::core::ffi::c_int & !(0x400 as ::core::ffi::c_int)) as uint16;
    (*E).base.health = 18000 as uint16;
    (*E0).mbby_var_A = (((0xa98d79i32) as u16) & (0xffffu16));
    (*E0).mbby_var_F = 128 as uint16;
    MotherBrainBody_FakeDeath_Ascent_8_Pause(game);
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_8_Pause(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0 as ::core::ffi::c_int, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if (*E).mbby_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbby_var_A = (((0xa98d8bi32) as u16) & (0xffffu16));
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
        (*E1).base.properties = (((*E1).base.properties) | (0x100u16));
        MotherBrain_SetBrainInstrs(addr_stru_A99C21 as uint16, game);
        (*E).mbn_var_A = (((0xa98db4i32) as u16) & (0xffffu16));
        (*E).mbn_var_F = 256 as uint16;
        MotherBrainBody_FakeDeath_Ascent_10_LoadLegTiles(game);
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_10_LoadLegTiles(game: &mut Game) {
    if ProcessSpriteTilesTransfers(0xa9 as uint8, addr_stru_A98F8F as uint16, game) != 0 {
        (*Get_MotherBrainBody(0 as ::core::ffi::c_int, game)).mbby_var_A =
            (((0xa98dc3i32) as u16) & (0xffffu16));
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
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) =
            -(27 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) =
            -(217 as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_04 = 7 as uint16;
        (*E).mbn_var_A = (((0xa98deci32) as u16) & (0xffffu16));
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_12_StartMusic(game: &mut Game) {
    MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9A02 as uint16, game);
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).base.instruction_timer = 1 as uint16;
    let mut E0: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E0).base.properties = ((*E0).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    (*E0).base.x_pos = 59 as uint16;
    (*E0).base.y_pos = 279 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) =
        -(27 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) =
        -(217 as ::core::ffi::c_int) as uint16;
    QueueMusic_Delayed8(5 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 2 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 256 as uint16;
    (*E1).mbn_var_34 = 80 as uint16;
    (*E1).mbn_var_31 = 1 as uint16;
    (*E1).mbn_var_32 = 8 as uint16;
    (*E1).mbn_var_33 = 6 as uint16;
    (*E0).mbn_var_A = (((0xa98e4di32) as u16) & (0xffffu16));
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_13_Raise(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 = 0;
    if !(*((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || {
            MotherBrain_SpawnDustCloudsForAscent(game);
            let ref mut fresh7 = *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16);
            *fresh7 = ((*fresh7).wrapping_add((2u16)));
            v1 = (((*E).base.y_pos).wrapping_sub((2u16)));
            (*E).base.y_pos = v1;
            v1 as ::core::ffi::c_int >= 0xbd as ::core::ffi::c_int
        })
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x179a) as *mut uint16) = 320 as uint16;
        (*E).base.y_pos = 188 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18b4) as *mut uint16)
            .offset(((*E).mbn_var_09 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            0 as uint16;
        MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_99AA as uint16, game);
        (*E).mbn_var_A = (((0xa98e95i32) as u16) & (0xffffu16));
        MotherBrainBody_FakeDeath_Ascent_14_WaitForUncouching(game);
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_14_WaitForUncouching(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if (*E).mbn_var_02 == 0 {
        (*E).mbn_var_37 = 0 as uint16;
        (*E).mbn_var_A = (((0xa98eaai32) as u16) & (0xffffu16));
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
        let mut v3: uint16 = (((*E0).mbn_var_37).wrapping_add((1u16)));
        (*E0).mbn_var_37 = v3;
        if MotherBrain_FadeFromGray_FakeDeath(((v3).wrapping_sub((1u16))), game)
            as ::core::ffi::c_int
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
            (*E0).mbn_var_A = (((0xa98ef5i32) as u16) & (0xffffu16));
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
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9B7F as uint16, game);
        (*E).mbn_var_A = (((0xa98f14i32) as u16) & (0xffffu16));
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
        (*E).mbn_var_A = (((0xa98f33i32) as u16) & (0xffffu16));
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
        (*E).mbn_var_A = (((0xa9b605i32) as u16) & (0xffffu16));
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
        addr_kEproj_DustCloudExplosion as uint16,
        g_byte_A98F7D[(((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16))
            & (0x100u16)) as ::core::ffi::c_int
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
pub fn MotherBrain_HandleNeckLower_0() {}
pub unsafe fn MotherBrain_HandleNeckLower_2_BobDown(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    v1 = ((((*E).mbn_var_20) as i16).wrapping_sub((((*E).mbn_var_34) as i16)));
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
        v1 = ((((*E).mbn_var_34) as i16).wrapping_add((((*E).mbn_var_20) as i16)));
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
    v1 = ((((*E).mbn_var_20) as i16).wrapping_sub((((*E).mbn_var_34) as i16)));
    if (v1 as uint16 as ::core::ffi::c_int) < 0x3000 as ::core::ffi::c_int {
        (*E).mbn_var_32 = 0 as uint16;
        v1 = 12288 as int16;
    }
    (*E).mbn_var_20 = v1 as uint16;
}
pub unsafe fn MotherBrain_HandleNeckLower_8_Raise(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    v1 = ((((*E).mbn_var_34) as i16).wrapping_add((((*E).mbn_var_20) as i16)));
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
                    - *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                        as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
                {
                    v2 = ((((*E).mbn_var_21) as i16).wrapping_sub((((*E).mbn_var_34) as i16)));
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
                let mut r18: uint16 = (((*E).mbn_var_20).wrapping_add((2048u16)));
                let mut v4: uint16 = (((*E).mbn_var_34).wrapping_add(((*E).mbn_var_21)));
                if v4 as ::core::ffi::c_int >= r18 as ::core::ffi::c_int {
                    (*E).mbn_var_33 = 2 as uint16;
                    v4 = r18;
                }
                (*E).mbn_var_21 = v4;
            }
            6 => {
                v6 = ((((*E).mbn_var_21) as i16).wrapping_sub((((*E).mbn_var_34) as i16)));
                if (v6 as uint16 as ::core::ffi::c_int) < 0x2000 as ::core::ffi::c_int {
                    (*E).mbn_var_33 = 0 as uint16;
                    v6 = 0x2000 as int16;
                }
                (*E).mbn_var_21 = v6 as uint16;
            }
            8 => {
                let mut r18_0: uint16 = (((*E).mbn_var_20).wrapping_add((2048u16)));
                let mut v8: uint16 = (((*E).mbn_var_34).wrapping_add(((*E).mbn_var_21)));
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
    (*E).mbn_var_0A = (((*E).base.x_pos).wrapping_sub((80u16)));
    (*E).mbn_var_0B = (((*E).base.y_pos).wrapping_add((46u16)));
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if (*E1).mbn_var_31 != 0 {
        MotherBrain_HandleNeckLower(game);
        MotherBrain_HandleNeckUpper(game);
    }
    let mut r18: uint16 = *(&raw mut (*E1).mbn_var_20 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
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
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) as uint16;
    (*E1).mbn_var_2B =
        (((*E1).mbn_var_28).wrapping_add((ComputeSinMult((*E1).mbn_var_2D, r18, game))));
    (*E1).mbn_var_2C =
        (((*E1).mbn_var_29).wrapping_add((ComputeCosMult((*E1).mbn_var_2D, r18, game))));
    (*E1).mbn_var_2E =
        (((*E1).mbn_var_28).wrapping_add((ComputeSinMult((*E1).mbn_var_30, r18, game))));
    (*E1).mbn_var_2F =
        (((*E1).mbn_var_29).wrapping_add((ComputeCosMult((*E1).mbn_var_30, r18, game))));
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
    let mut p: *const uint8 = RomPtr_A9(pc, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) != 0 {
        return *(p.offset(2) as *mut uint16) as ::core::ffi::c_int;
    }
    v5 = *(p as *mut uint16);
    if v5 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        current_block = 87389992155768869;
    } else {
        if (((v5) as i16).wrapping_sub((((*E).mbn_var_20) as i16))) as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            (*E).mbn_var_20 = (*E).mbn_var_20.wrapping_add(1);
            return *(p.offset(2) as *mut uint16) as ::core::ffi::c_int;
        }
        pc = ((pc).wrapping_add((4u16)));
        current_block = 1917311967535052937;
    }
    loop {
        match current_block {
            87389992155768869 => {
                pc = CallMotherBrainInstr(
                    (((v5) as u32) | (0xa90000u32)),
                    ((pc).wrapping_add((2u16))),
                    game,
                );
                current_block = 1917311967535052937;
            }
            _ => {
                p = RomPtr_A9(pc, game);
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
    return *(p.offset(2) as *mut uint16) as ::core::ffi::c_int;
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
        && *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainPurpleBreathSmall as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16),
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
        flash_timer = ((mbn_var_00).wrapping_sub((1u16)));
        (*E).mbn_var_00 = flash_timer;
    } else {
        flash_timer = (*E).base.flash_timer;
        if flash_timer == 0 {
            flash_timer = (*E).base.shake_timer;
        }
    }
    let mut v6: ::core::ffi::c_int =
        (flash_timer as ::core::ffi::c_int & 6 as ::core::ffi::c_int) >> 1 as ::core::ffi::c_int;
    let mut r18: uint16 = (((*E).base.x_pos).wrapping_add(((g_word_A993BB[v6 as usize]) as u16)));
    if (r18 as ::core::ffi::c_int + 32 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        let mut r20: uint16 =
            (((*E).base.y_pos).wrapping_add(((g_word_A993C3[v6 as usize]) as u16)));
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
        addr_kMotherBrain_Sprmap_A694 as uint16,
        ((x).wrapping_add(((g_word_A993BB[v0 as usize]) as u16))),
        ((y).wrapping_add(((g_word_A993C3[v0 as usize]) as u16))),
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
    let mut p: *const uint8 = RomPtr_A9(j, game);
    let mut n: ::core::ffi::c_int = *(p as *mut uint16) as ::core::ffi::c_int;
    p = p.offset(2);
    let mut idx: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) as ::core::ffi::c_int;
    loop {
        let mut y: int16 = (r20 as ::core::ffi::c_int + *p.offset(2) as int8 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int) as int16;
        if y as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            let mut oam: *mut OamEnt = gOamEnt(idx, game);
            let mut x: uint16 = (r18 as ::core::ffi::c_int
                + *(p as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
            (*oam).xcoord = x as uint8;
            (*oam).ycoord = y as uint8;
            *(&raw mut (*oam).charnum as *mut uint16) = ((r22) | (*(p.offset(3) as *mut uint16)));
            let ref mut fresh6 = *((&raw mut game.g_ram as *mut uint8).offset(0x570)
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
        p = p.offset(5);
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) = idx as uint16;
}
pub unsafe fn MotherBrain_CalculateRainbowBeamHdma(game: &mut Game) {
    MotherBrain_CalcHdma(game);
}
pub unsafe fn MotherBrain_MoveBodyDownScrollLeft(mut k: uint16, mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.y_pos = (((*E).base.y_pos).wrapping_add((a)));
    let ref mut fresh1 = *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16);
    *fresh1 = ((*fresh1).wrapping_sub((a)));
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = (k as ::core::ffi::c_int
        + 34 as ::core::ffi::c_int
        - (*E).base.x_pos as ::core::ffi::c_int)
        as uint16;
}
pub unsafe fn MotherBrain_MoveBodyDown(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.y_pos = (((*E).base.y_pos).wrapping_add((a)));
    let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16);
    *fresh0 = ((*fresh0).wrapping_sub((a)));
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) =
        ((34u16).wrapping_sub(((*E).base.x_pos)));
}
pub unsafe fn MotherBrain_FootstepEffect(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 4 as uint16;
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
    (*E).base.x_pos = (((*E).base.x_pos).wrapping_add((2u16)));
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
    (*E).base.x_pos = (((*E).base.x_pos).wrapping_add((3u16)));
    MotherBrain_MoveBodyDown(1 as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_Down2Right15(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = (((*E).base.x_pos).wrapping_add((15u16)));
    MotherBrain_MoveBodyDown(0xfffe as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_Down4Right6(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = (((*E).base.x_pos).wrapping_add((6u16)));
    MotherBrain_MoveBodyDown(0xfffc as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_Up4Left2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = (((*E).base.x_pos).wrapping_sub((2u16)));
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
    (*E).base.x_pos = (((*E).base.x_pos).wrapping_sub((2u16)));
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
    (*E).base.x_pos = (((*E).base.x_pos).wrapping_sub((3u16)));
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
    (*E).base.x_pos = (((*E).base.x_pos).wrapping_sub((15u16)));
    MotherBrain_MoveBodyDown(2 as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveUp4Left6(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = (((*E).base.x_pos).wrapping_sub((6u16)));
    MotherBrain_MoveBodyDown(4 as uint16, game);
    return jp;
}
pub unsafe fn MotherBrain_Instr_MoveDown4Right2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = (((*E).base.x_pos).wrapping_add((2u16)));
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
        x: (((*E).base.x_pos).wrapping_add((*jp.offset(0)))),
        y: (((*E).base.y_pos).wrapping_add((*jp.offset(1)))),
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as uint16,
        *jp.offset(2),
        game,
    );
    return jp.offset(3);
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
        addr_kEproj_MotherBrainDeathBeamCharging as uint16,
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
    return *(RomPtr_A9(k, game) as *mut uint16);
}
pub unsafe fn MotherBrain_Instr_EnableNeckMovementGoto(mut k: uint16, game: &mut Game) -> uint16 {
    (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_31 = 1 as uint16;
    return *(RomPtr_A9(k, game) as *mut uint16);
}
pub unsafe fn MotherBrain_Instr_DisableNeckMovement(mut k: uint16, game: &mut Game) -> uint16 {
    (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_31 = 0 as uint16;
    return k;
}
pub unsafe fn MotherBrain_Instr_QueueSfx2(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v2: *const uint16 = RomPtr_A9(k, game) as *const uint16;
    QueueSfx2_Max6(*v2, game);
    return ((k).wrapping_add((2u16)));
}
pub unsafe fn MotherBrain_Instr_QueueSfx3(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v2: *const uint16 = RomPtr_A9(k, game) as *const uint16;
    QueueSfx3_Max6(*v2, game);
    return ((k).wrapping_add((2u16)));
}
pub unsafe fn MotherBrain_Instr_SpawnDroolEproj(mut k: uint16, game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if (*E).mbn_var_12 != 0 {
        let mut v2: uint16 = (((*E).mbn_var_13).wrapping_add((1u16)));
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
        addr_kEproj_MotherBrainPurpleBreathBig as uint16,
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
    let mut result: uint16 = addr_stru_A99C5F as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        & 0xfff as ::core::ffi::c_int
        >= 0xfe0 as ::core::ffi::c_int
    {
        return addr_stru_A99C47 as uint16;
    }
    return result;
}
pub unsafe fn MotherBrain_Instr_MaybeGoto(mut k: uint16, game: &mut Game) -> uint16 {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int)
        < 0xf000 as ::core::ffi::c_int
    {
        return addr_stru_A99C9F as uint16;
    }
    return k;
}
pub unsafe fn MotherBrain_Instr_MaybeGoto2(mut k: uint16, game: &mut Game) -> uint16 {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
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
    return addr_stru_A99CD1 as uint16;
}
pub unsafe fn MotherBrain_Instr_SpawnBlueRingEproj(mut k: uint16, game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    SpawnEprojWithRoomGfx(
        addr_kEproj_MotherBrainBlueRingLasers as uint16,
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
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int
            - 10 as ::core::ffi::c_int) as uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
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
        *(&raw mut v0 as *mut uint8).offset(0) = 16 as uint8;
        current_block = 8016059118846638429;
    }
    match current_block {
        5817625388393324680 => {
            *(&raw mut v0 as *mut uint8).offset(0) = 72 as uint8;
        }
        _ => {}
    }
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_1A = v0;
}
pub unsafe fn MotherBrain_Instr_IncrShitroidAttackCtr(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    v3 = ((((*E).mbn_var_13) as i16).wrapping_add((1i16)));
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
    let mut v2: *const uint16 = RomPtr_A9(k, game) as *const uint16;
    SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainBomb as uint16, *v2, game);
    return ((k).wrapping_add((2u16)));
}
pub unsafe fn MotherBrain_Instr_SpawnLaserEproj(mut k: uint16, game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E).mbn_var_31 = 0 as uint16;
    game.eproj_spawn_r22 = 1 as uint16;
    game.eproj_spawn_pt = Point16U {
        x: (((*E).base.x_pos).wrapping_add((16u16))),
        y: (((*E).base.y_pos).wrapping_add((4u16))),
    };
    SpawnEprojWithRoomGfx(addr_stru_86A17B as uint16, 1 as uint16, game);
    return k;
}
pub unsafe fn MotherBrain_Instr_SpawnRainbowEproj(mut k: uint16, game: &mut Game) -> uint16 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_MotherBrainRainbowBeamCharging as uint16,
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
    (*E).base.properties = (((*E).base.properties) | ((kEnemyProps_Tangible) as u16));
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).base.properties = (((*E1).base.properties) | (0x400u16));
    (*E).mbn_var_04 = 0 as uint16;
    if MotherBrain_MakeWalkBackwards(0x28 as uint16, 6 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*E).mbn_var_A = (((0xa9af12i32) as u16) & (0xffffu16));
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
        (*E).mbn_var_A = (((0xa9af21i32) as u16) & (0xffffu16));
    }
}
pub unsafe fn MotherBrain_Phase3_Death_2(game: &mut Game) {
    MotherBrain_GenerateSmokyExplosions(game);
    if MotherBrain_MakeWalkForwards(2 as uint16, 0x60 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C39 as uint16, game);
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_32 = 6 as uint16;
        (*E1).mbn_var_33 = 6 as uint16;
        (*E1).mbn_var_34 = 1280 as uint16;
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_A = (((0xa9af54i32) as u16) & (0xffffu16));
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
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 241 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                    .offset(((i >> 1 as ::core::ffi::c_int) + 145 as ::core::ffi::c_int) as isize);
            i -= 2 as ::core::ffi::c_int;
        }
        MotherBrain_HealthBasedPaletteHandling(game);
        (*E).mbn_var_0D = 3584 as uint16;
        (*E1).mbn_var_E = 0 as uint16;
        (*E1).mbn_var_F = 0 as uint16;
        (*E).mbn_var_A = (((0xa9af9di32) as u16) & (0xffffu16));
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
        (*E).mbn_var_A = (((0xa9afb6i32) as u16) & (0xffffu16));
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
        let mut v2: uint16 = (((*E).mbn_var_37).wrapping_add((1u16)));
        (*E).mbn_var_37 = v2;
        if MotherBrain_FadePalToBlack(((v2).wrapping_sub((1u16))), game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x179a) as *mut uint16) = 710 as uint16;
            let mut i: ::core::ffi::c_int = 710 as ::core::ffi::c_int;
            while i >= 0 as ::core::ffi::c_int {
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((i >> 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
                i -= 2 as ::core::ffi::c_int;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xe1e) as *mut uint16) = 1 as uint16;
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & 0xdeff as ::core::ffi::c_int
                | 0x100 as ::core::ffi::c_int) as uint16;
            (*E).base.extra_properties = 0 as uint16;
            (*E).mbn_var_A = (((0xa9b013i32) as u16) & (0xffffu16));
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
        (*E).mbn_var_A = (((0xa9b115i32) as u16) & (0xffffu16));
    }
}
pub unsafe fn MotherBrain_GenerateSmokyExplosions(game: &mut Game) {
    MotherBrain_GenerateExplosions(
        0x10 as uint16,
        addr_word_A9B10F as uint16,
        2 as uint16,
        game,
    );
}
pub unsafe fn MotherBrain_GenerateMixedExplosions(game: &mut Game) {
    MotherBrain_GenerateExplosions(8 as uint16, addr_word_A9B109 as uint16, 4 as uint16, game);
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
        let mut v3: uint16 = ((16u16).wrapping_mul(((*E).mbn_var_F)));
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
            let mut v6: *const uint16 = RomPtr_A9(r22, game) as *const uint16;
            let mut v7: uint16 = *v6;
            let mut Random: uint16 = NextRandom(game);
            if Random as ::core::ffi::c_int >= 0x4000 as ::core::ffi::c_int {
                v7 = *v6.offset(1);
                if Random as ::core::ffi::c_int >= 0xe000 as ::core::ffi::c_int {
                    v7 = *v6.offset(2);
                }
            }
            SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainDeathExplosion as uint16, v7, game);
            v3 = ((v3).wrapping_add((4u16)));
            v4 = ((v9).wrapping_sub((1u16)));
            if !(v9 as ::core::ffi::c_int != 1 as ::core::ffi::c_int) {
                break;
            }
        }
        QueueSfx3_Max3(0x13 as uint16, game);
    }
}
pub unsafe fn MotherBrain_Phase3_Death_7(game: &mut Game) {
    MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C29 as uint16, game);
    (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_A =
        (((0xa987d0i32) as u16) & (0xffffu16));
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = 0 as uint16;
    (*E).mbn_var_A = (((0xa9b12di32) as u16) & (0xffffu16));
    MotherBrain_Phase3_Death_8(game);
}
pub unsafe fn MotherBrain_Phase3_Death_8(game: &mut Game) {
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 = (((*E).mbn_var_F).wrapping_add((32u16)));
    (*E).mbn_var_F = v1;
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    v3 = ((((*E1).base.y_pos) as i16).wrapping_add(
        ((*(&raw mut v1 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        )) as i16),
    ));
    if v3 as uint16 as ::core::ffi::c_int >= 0xc4 as ::core::ffi::c_int {
        EnableEarthquakeAframes(2 as uint16, game);
        (*E).mbn_var_A = (((0xa9b15ei32) as u16) & (0xffffu16));
        (*E).mbn_var_F = 256 as uint16;
        v3 = 196 as int16;
    }
    (*E1).base.y_pos = v3 as uint16;
}
pub unsafe fn MotherBrain_Phase3_Death_9(game: &mut Game) {
    if ProcessSpriteTilesTransfers(0xa9 as uint8, addr_stru_A99003 as uint16, game) != 0 {
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_A = (((0xa9b173i32) as u16) & (0xffffu16));
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
        (*E).mbn_var_A = (((0xa9b189i32) as u16) & (0xffffu16));
        (*E).mbn_var_F = 0 as uint16;
    }
}
pub unsafe fn MotherBrain_Phase3_Death_11(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut v2: uint16 = (((*E).mbn_var_37).wrapping_add((1u16)));
        (*E).mbn_var_37 = v2;
        if MotherBrain_FadeToGray_RealDeath(((v2).wrapping_sub((1u16))), game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9D25 as uint16, game);
            (*E).mbn_var_A = (((0xa9b1b8i32) as u16) & (0xffffu16));
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
        (*E).mbn_var_A = (((0xa9b1d5i32) as u16) & (0xffffu16));
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).base.properties = (((*E1).base.properties) | (0x400u16));
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
        (*E1).dms_var_A = (((0xa9b211i32) as u16) & (0xffffu16));
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
        (*E).mbn_var_A = (((0xa9b258i32) as u16) & (0xffffu16));
    }
}
pub unsafe fn MotherBrain_CorpseRottingFinished(game: &mut Game) {
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    game.eproj_spawn_pt = Point16U {
        x: ((*E1).base.x_pos as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                as ::core::ffi::c_int
                & 0x1f as ::core::ffi::c_int)
            - 16 as ::core::ffi::c_int) as uint16,
        y: (((*E1).base.y_pos).wrapping_add((16u16))),
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as uint16,
        0xa as uint16,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe44) as *mut uint16) as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        QueueSfx2_Max3(0x10 as uint16, game);
    }
}
pub unsafe fn MotherBrain_Phase3_Death_15_LoadEscapeTimerTiles(game: &mut Game) {
    if ProcessSpriteTilesTransfers(0xa6 as uint8, addr_stru_A6C4CB as uint16, game) != 0 {
        (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
            (((0xa9b26di32) as u16) & (0xffffu16));
        MotherBrain_Phase3_Death_16_StartEscape(game);
    }
}
pub unsafe fn MotherBrain_Phase3_Death_16_StartEscape(game: &mut Game) {
    if ProcessSpriteTilesTransfers(0xa9 as uint8, addr_stru_A9902F as uint16, game) != 0 {
        WriteColorsToPalette(
            0x122 as uint16,
            0xa9 as uint8,
            (addr_kMotherBrainPalette_5 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
            0xe as uint16,
            game,
        );
        QueueMusic_Delayed8(7 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 5 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) =
            -(1 as ::core::ffi::c_int) as uint16;
        SpawnPalfxObject(addr_kPalfx_FFC9 as uint16, game);
        SpawnPalfxObject(addr_kPalfx_FFCD as uint16, game);
        SpawnPalfxObject(addr_kPalfx_FFD1 as uint16, game);
        SpawnPalfxObject(addr_kPalfx_FFD5 as uint16, game);
        (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_02 = 0 as uint16;
        SetupZebesEscapeTypewriter(game);
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_F = 32 as uint16;
        let mut v1: uint16 = (((0xa9b2d1i32) as u16) & (0xffffu16));
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) == 0 {
            v1 = (((0xa9b2e3i32) as u16) & (0xffffu16));
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
        (*E).mbn_var_A = (((0xa9b2e3i32) as u16) & (0xffffu16));
        SpawnEprojWithRoomGfx(
            addr_kEproj_TimeBombSetJapaneseText as uint16,
            (((0xa9b2e3i32) as u16) & (0xffffu16)),
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
        (*E).mbn_var_A = (((0xa9b2f9i32) as u16) & (0xffffu16));
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x943) as *mut uint16) = 2 as uint16;
        SetBossBitForCurArea(2 as uint16, game);
        SetEventHappened(0xe as uint16, game);
        (*E).mbn_var_A = (((0xa9b32ai32) as u16) & (0xffffu16));
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_E = 0 as uint16;
        (*E1).mbn_var_F = 0 as uint16;
    }
}
pub unsafe fn MotherBrain_Phase3_Death_20_BlowUpEscapeDoor(game: &mut Game) {
    MotherBrain_ExplodeEscapeDoor(game);
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
        (((0xa9b33ci32) as u16) & (0xffffu16));
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
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) == 0 {
        let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16);
        *fresh10 = (*fresh10).wrapping_sub(1);
    }
}
pub unsafe fn MotherBrain_GenerateEscapeDoorExploding(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    v1 = ((((*E).mbn_var_E) as i16).wrapping_sub((1i16)));
    (*E).mbn_var_E = v1 as uint16;
    if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).mbn_var_E = 4 as uint16;
        v2 = ((((*E).mbn_var_F) as i16).wrapping_sub((1i16)));
        (*E).mbn_var_F = v2 as uint16;
        if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            (*E).mbn_var_F = 3 as uint16;
        }
        let mut v3: ::core::ffi::c_int = ((4u16).wrapping_mul(((*E).mbn_var_F)))
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        game.eproj_spawn_pt = Point16U {
            x: g_word_A9B393[v3 as usize],
            y: g_word_A9B393[(v3 + 1 as ::core::ffi::c_int) as usize],
        };
        let mut v4: uint16 = 3 as uint16;
        if (NextRandom(game) as ::core::ffi::c_int) < 0x4000 as ::core::ffi::c_int {
            v4 = 12 as uint16;
        }
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, v4, game);
        QueueSfx2_Max3(0x24 as uint16, game);
    }
}
pub unsafe fn MotherBrain_ExplodeEscapeDoor(game: &mut Game) {
    let mut v0: uint16 = 0 as uint16;
    let mut v1: uint16 = 0;
    loop {
        v1 = v0;
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainExplodedEscapeDoor as uint16,
            v0,
            game,
        );
        v0 = ((v1).wrapping_add((1u16)));
        if !((((v1).wrapping_add((1u16))) as ::core::ffi::c_int) < 8 as ::core::ffi::c_int) {
            break;
        }
    }
}
pub unsafe fn MotherBrain_SamusCollDetect(game: &mut Game) {
    let mut v1: int8 = 0;
    let mut E0: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let mut R26: uint16 = (*E0).mbn_var_04;
    v1 = (((R26) as i8) & (1i8));
    R26 = (R26 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    if v1 == 0
        || MotherBrain_SamusCollDetectPart(
            addr_word_A9B427 as uint16,
            (*E0).base.x_pos,
            (*E0).base.y_pos,
            game,
        ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0
    {
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        v1 = (((R26) as i8) & (1i8));
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
            v1 = (((R26) as i8) & (1i8));
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
    let mut p: *const uint8 = RomPtr_A9(k, game);
    let mut n: ::core::ffi::c_int = *(p as *mut uint16) as ::core::ffi::c_int;
    p = p.offset(2);
    while n != 0 {
        let mut v3: uint16 = 0;
        let mut v4: uint16 = 0;
        let mut R24: uint16 = 0;
        if (((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)) as i16)
            .wrapping_sub(((r20) as i16))) as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            R24 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
                .wrapping_sub((r20)));
            v3 = *(p.offset(6) as *mut uint16);
        } else {
            R24 = ((r20).wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)),
            ));
            v3 = *(p.offset(2) as *mut uint16);
        }
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16)
            as ::core::ffi::c_int
            + abs16(v3) as ::core::ffi::c_int
            - R24 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            if (((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
                .wrapping_sub(((r18) as i16))) as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                R24 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
                    .wrapping_sub((r18)));
                v4 = *(p.offset(4) as *mut uint16);
            } else {
                R24 = ((r18).wrapping_sub(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)),
                ));
                v4 = *(p as *mut uint16);
            }
            v5 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16)
                as ::core::ffi::c_int
                + abs16(v4) as ::core::ffi::c_int
                - R24 as ::core::ffi::c_int) as int16;
            if v5 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                if v5 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    v5 = 4 as int16;
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = v5 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16) = 4 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5a) as *mut uint16) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) = 96 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x18aa) as *mut uint16) = 5 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xa54) as *mut uint16) = 1 as uint16;
                if *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int
                    - 192 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xb36) as *mut uint16) =
                        2 as uint16;
                }
                if (((*Get_MotherBrain(0 as ::core::ffi::c_int, game)).base.x_pos
                    as ::core::ffi::c_int
                    + 24 as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                        as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
                {
                    MotherBrain_HurtSamus(game);
                }
                return 1 as uint8;
            }
        }
        p = p.offset(8);
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
        let mut v1: uint16 =
            (((*(((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16) as isize,
            ) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1 as usize) as isize,
                )) as u16)
                & (7u16));
        if g_byte_A9B546[v1 as usize] != 0 {
            let ref mut fresh11 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x1dc7) as *mut uint16).offset(39);
            *fresh11 = ((*fresh11).wrapping_add(((g_byte_A9B546[v1 as usize]) as u16)));
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
                v1 = ((((*E).mbn_var_07) as i16).wrapping_sub((256i16)));
                (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int
            }
        {
            v1 = 0 as int16;
        }
        (*E).mbn_var_07 = v1 as uint16;
    };
}
pub unsafe fn MotherBrain_DetermineShotReactionType(game: &mut Game) -> uint16 {
    return g_byte_A9B5A1[(*(((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16) as isize)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int) as usize] as uint16;
}
pub unsafe fn MotherBrain_Phase3_BeamShotReaction(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    v1 = ((((*E).mbn_var_07) as i16).wrapping_sub((266i16)));
    if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_18 =
            (((0xa9c395i32) as u16) & (0xffffu16));
        v1 = 0 as int16;
        (*E).mbn_var_F = 0 as uint16;
    }
    (*E).mbn_var_07 = v1 as uint16;
}
pub unsafe fn MotherBrainsBrain_Touch(game: &mut Game) {
    let mut v0: int16 = 0;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as ::core::ffi::c_int
        == 3 as ::core::ffi::c_int
    {
        v0 = 13 as int16;
        let mut v1: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
        let mut flash_timer: uint16 = (*v1.offset(1)).flash_timer;
        if flash_timer != 0 {
            if flash_timer as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
                v0 = 14 as int16;
            }
        }
        (*v1.offset(1)).flash_timer = v0 as uint16;
    }
}
pub unsafe fn MotherBrain_HurtSamus(game: &mut Game) {
    Ridley_Func_98(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) = 96 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x18aa) as *mut uint16) = 5 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa54) as *mut uint16) =
        ((((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
            .wrapping_sub(
                (((*Get_MotherBrain(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                        as ::core::ffi::c_int,
                    game,
                ))
                .base
                .x_pos) as i16),
            )) as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MotherBrain_Body_Phase2_Thinking(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if (*E1).base.health == 0 {
        (*E).mbn_var_A = (((0xa9b8ebi32) as u16) & (0xffffu16));
        MotherBomb_FiringRainbowBeam_0(game);
        return;
    }
    if (*E).mbn_var_02 == 0 {
        let mut current_block_9: u64;
        if (*E1).base.health as ::core::ffi::c_int - 4500 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                as ::core::ffi::c_int
                >= 0x1000 as ::core::ffi::c_int
            {
                current_block_9 = 3601231341105262975;
            } else {
                current_block_9 = 13880174989427944637;
            }
        } else if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int)
            < 0x2000 as ::core::ffi::c_int
        {
            current_block_9 = 3601231341105262975;
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            >= 0xa000 as ::core::ffi::c_int
        {
            current_block_9 = 13880174989427944637;
        } else {
            (*E).mbn_var_A = (((0xa9b87di32) as u16) & (0xffffu16));
            return;
        }
        match current_block_9 {
            3601231341105262975 => {
                MotherBrain_HandleWalking(game);
                return;
            }
            _ => {
                (*E).mbn_var_A = (((0xa9b64bi32) as u16) & (0xffffu16));
                return;
            }
        }
    }
}
pub unsafe fn MotherBrain_Body_Phase2_TryAttack(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    v0 = ((2i16).wrapping_mul((((*E).mbn_var_18) as i16)));
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
        let mut v2: uint16 = addr_byte_A9B6DC as uint16;
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        if (abs16(
            ((*E1).base.y_pos as ::core::ffi::c_int + 4 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int) as uint16,
        ) as ::core::ffi::c_int)
            < 0x20 as ::core::ffi::c_int
        {
            v2 = addr_byte_A9B6DF as uint16;
        }
        let mut v4: uint16 = 0 as uint16;
        let mut v5: *const uint8 = RomPtr_A9(v2, game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as uint8
            as ::core::ffi::c_int
            >= *v5 as ::core::ffi::c_int
        {
            v4 = 2 as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as uint8
                as ::core::ffi::c_int
                >= *v5.offset(1) as ::core::ffi::c_int
            {
                v4 = 4 as uint16;
                if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as uint8
                    as ::core::ffi::c_int
                    >= *v5.offset(2) as ::core::ffi::c_int
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
                (*E).mbn_var_A = (((0xa9b781i32) as u16) & (0xffffu16));
                MotherBrain_FiringBomb_DecideOnWalking(game);
            }
        } else if v6 as ::core::ffi::c_int == addr_kMotherBrain_Ilist_9F34 as ::core::ffi::c_int {
            (*E).mbn_var_A = (((0xa9b80ei32) as u16) & (0xffffu16));
            MotherBomb_FiringLaser_PositionHead(game);
        } else {
            MotherBrain_SetBrainInstrs(v6, game);
        }
    };
}
pub unsafe fn MotherBrain_Phase2_DecideAttackStrategy_DoubleRet(game: &mut Game) -> uint8 {
    if g_word_A9B72C[*(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as usize] != 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as uint8
            as ::core::ffi::c_int
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
                    (((0xa9b781i32) as u16) & (0xffffu16));
                MotherBrain_FiringBomb_DecideOnWalking(game);
                return 1 as uint8;
            }
            return 0 as uint8;
        }
    } else {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as uint8
            as ::core::ffi::c_int
            - 128 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
                (((0xa9b80ei32) as u16) & (0xffffu16));
            MotherBomb_FiringLaser_PositionHead(game);
        } else {
            MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9D3D as uint16, game);
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
    (*E).mbn_var_A = (((0xa9b605i32) as u16) & (0xffffu16));
}
pub unsafe fn MotherBrain_FiringBomb_DecideOnWalking(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        >= 0xff80 as ::core::ffi::c_int
    {
        MotherBrain_FiringBomb_DecideOnCrouching(game);
        return;
    }
    let mut v0: uint16 = 64 as uint16;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int)
        < 0x6000 as ::core::ffi::c_int
    {
        v0 = 96 as uint16;
    }
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if (((v0) as i16).wrapping_sub((((*E).base.x_pos) as i16))) as ::core::ffi::c_int
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
        (*E).mbn_var_A = (((0xa9b7aci32) as u16) & (0xffffu16));
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
            (((0xa9b7c6i32) as u16) & (0xffffu16));
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
    let mut v0: uint16 = addr_kMotherBrain_Ilist_9ECC as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if (*E).mbn_var_1F != 0 {
        v0 = addr_kMotherBrain_Ilist_9F00 as uint16;
    }
    MotherBrain_SetBrainInstrs(v0, game);
    (*E).mbn_var_A = (((0xa9b7e8i32) as u16) & (0xffffu16));
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
            (*E).mbn_var_A = (((0xa9b7f8i32) as u16) & (0xffffu16));
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
    let mut v0: uint16 = (((0xa9b605i32) as u16) & (0xffffu16));
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if (*E).mbn_var_1F != 0 {
        v0 = (((0xa9c15ci32) as u16) & (0xffffu16));
    }
    (*E).mbn_var_A = v0;
}
pub unsafe fn MotherBomb_FiringLaser_PositionHead(game: &mut Game) {
    let mut v0: int16 = 0;
    v0 = 8 as int16;
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if (((((*E1).base.y_pos) as i16).wrapping_sub(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)) as i16),
    )) as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        v0 = 6 as int16;
    }
    (*E1).mbn_var_32 = v0 as uint16;
    (*E1).mbn_var_33 = v0 as uint16;
    (*E1).mbn_var_34 = 512 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_A = (((0xa9b839i32) as u16) & (0xffffu16));
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
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9F34 as uint16, game);
        (*E).mbn_var_A = (((0xa9b863i32) as u16) & (0xffffu16));
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
        (*E).mbn_var_A = (((0xa9b605i32) as u16) & (0xffffu16));
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
        MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9A42 as uint16, game);
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_17 = (*E).mbn_var_17.wrapping_add(1);
    }
}
pub fn MotherBomb_FiringDeathBeam_2(game: &mut Game) {}
pub unsafe fn MotherBomb_FiringDeathBeam_3(game: &mut Game) {
    MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C87 as uint16, game);
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).mbn_var_32 = 2 as uint16;
    (*E1).mbn_var_33 = 4 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_17 = 0 as uint16;
    (*E).mbn_var_A = (((0xa9b605i32) as u16) & (0xffffu16));
}
pub unsafe fn MotherBomb_FiringRainbowBeam_0(game: &mut Game) {
    MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C87 as uint16, game);
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).mbn_var_34 = 64 as uint16;
    (*E1).mbn_var_31 = 1 as uint16;
    (*E1).mbn_var_32 = 2 as uint16;
    (*E1).mbn_var_33 = 4 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_A = (((0xa9b91ai32) as u16) & (0xffffu16));
    (*E).mbn_var_F = 256 as uint16;
}
pub unsafe fn MotherBomb_FiringRainbowBeam_1_StartCharge(game: &mut Game) {
    let mut E: *mut Enemy_MotherBomb = Get_MotherBomb(0 as ::core::ffi::c_int, game);
    (*E).mbb_var_F = (*E).mbb_var_F.wrapping_sub(1);
    if (*E).mbb_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9F6C as uint16, game);
        (*E).mbb_var_A = (((0xa9b92bi32) as u16) & (0xffffu16));
        MotherBomb_FiringRainbowBeam_2_RetractNeck(game);
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_2_RetractNeck(game: &mut Game) {
    if MotherBrain_WalkBackwardsSlowlyAndRetractHead(0x28 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        let mut E: *mut Enemy_MotherBomb = Get_MotherBomb(0 as ::core::ffi::c_int, game);
        (*E).mbb_var_A = (((0xa9b93fi32) as u16) & (0xffffu16));
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
        (*E).mbb_var_A = (((0xa9b951i32) as u16) & (0xffffu16));
        MotherBomb_FiringRainbowBeam_4_ExtendNeckDown(game);
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_4_ExtendNeckDown(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xccc) as *mut uint16) = 8 as uint16;
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).mbn_var_32 = 6 as uint16;
    (*E1).mbn_var_33 = 6 as uint16;
    (*E1).mbn_var_34 = 1280 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_A = (((0xa9b975i32) as u16) & (0xffffu16));
    (*E).mbn_var_F = 16 as uint16;
    MotherBomb_FiringRainbowBeam_5_StartFiring(game);
}
pub unsafe fn MotherBomb_FiringRainbowBeam_5_StartFiring(game: &mut Game) {
    MotherBrain_AimBeamAndIncrWidth(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcee) as *mut uint16) == 0 {
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
        if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0xcee) as *mut uint16) == 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xccc) as *mut uint16) = 0 as uint16;
            MotherBrain_BodyRainbowBeamPalAnimIndex0(game);
            MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C77 as uint16, game);
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
            if ((((*((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16)) as i16)
                .wrapping_sub((700i16))) as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                v4 = 24 as uint16;
            }
            CallSomeSamusCode(v4, game);
            (*E).mbn_var_15 = 6 as uint16;
            (*E).mbn_var_A = (((0xa9b9e5i32) as u16) & (0xffffu16));
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
        (*E).mbb_var_A = (((0xa9ba00i32) as u16) & (0xffffu16));
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 8 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 8 as uint16;
        (*E).mbb_var_A = (((0xa9ba27i32) as u16) & (0xffffu16));
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_8_StartDrainSamus(game: &mut Game) {
    let mut E: *mut Enemy_MotherBomb = Get_MotherBomb(0 as ::core::ffi::c_int, game);
    (*E).mbb_var_A = (((0xa9ba3ci32) as u16) & (0xffffu16));
    (*E).mbb_var_F = 299 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 299 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 8 as uint16;
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
        (*E).mbb_var_A = (((0xa9ba5ei32) as u16) & (0xffffu16));
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_10_FinishFiringRainbow(game: &mut Game) {
    MotherBrain_PlayRainbowBeamSfx(game);
    MotherBrain_HandleRainbowBeamPalette(game);
    MotherBrain_AimBeam(game);
    MotherBrain_HandleRainbowBeamExplosions(game);
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 = (((*E).mbn_var_33).wrapping_sub((384u16)));
    (*E).mbn_var_33 = v1;
    if v1 as ::core::ffi::c_int - 512 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E).mbn_var_33 = 512 as uint16;
        (*E).mbn_parameter_1 = -(256 as ::core::ffi::c_int) as uint16;
        (*E).mbn_parameter_2 = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18b4) as *mut uint16)
            .offset(((*E).mbn_var_09 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 0 as uint16;
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C87 as uint16, game);
        MotherBrain_SetupBrainNormalPal(game);
        MotherBrain_WriteDefaultPalette(game);
        QueueSfx1_Max6(2 as uint16, game);
        (*E).mbn_var_16 = 0 as uint16;
        CallSomeSamusCode(1 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xccc) as *mut uint16) = 8 as uint16;
        (*E).mbn_var_A = (((0xa9bac4i32) as u16) & (0xffffu16));
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_11_LetSamusFall(game: &mut Game) {
    SomeMotherBrainScripts(0 as uint16, game);
    (*Get_MotherBomb(0 as ::core::ffi::c_int, game)).mbb_var_A =
        (((0xa9bad1i32) as u16) & (0xffffu16));
    MotherBomb_FiringRainbowBeam_12_WaitForSamusHitGround(game);
}
pub unsafe fn MotherBomb_FiringRainbowBeam_12_WaitForSamusHitGround(game: &mut Game) {
    if MotherBrain_MoveSamusForFallingAfterBeam(game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*Get_MotherBomb(0 as ::core::ffi::c_int, game)).mbb_var_A =
            (((0xa9baddi32) as u16) & (0xffffu16));
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_13_LowerHead(game: &mut Game) {
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).mbn_var_34 = 64 as uint16;
    (*E1).mbn_var_31 = 1 as uint16;
    (*E1).mbn_var_32 = 2 as uint16;
    (*E1).mbn_var_33 = 4 as uint16;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_A = (((0xa9bb06i32) as u16) & (0xffffu16));
    (*E).mbn_var_F = 128 as uint16;
}
pub unsafe fn MotherBomb_FiringRainbowBeam_14_DecideNextAction(game: &mut Game) {
    let mut E: *mut Enemy_MotherBomb = Get_MotherBomb(0 as ::core::ffi::c_int, game);
    (*E).mbb_var_F = (*E).mbb_var_F.wrapping_sub(1);
    if (*E).mbb_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) as ::core::ffi::c_int
            - 400 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            MotherBrain_MakeWalkForwards(
                0xa as uint16,
                (((*E).base.x_pos).wrapping_add((16u16))),
                game,
            );
            (*E).mbb_var_A = (((0xa9bd45i32) as u16) & (0xffffu16));
        } else {
            (*E).mbb_var_A = (((0xa9b8ebi32) as u16) & (0xffffu16));
        }
    }
}
pub unsafe fn MotherBrain_PlayRainbowBeamSfx(game: &mut Game) {
    let mut mbn_var_15: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    mbn_var_15 = (*E).mbn_var_15 as int16;
    if mbn_var_15 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        (*E).mbn_var_15 = (((mbn_var_15) as u16).wrapping_sub((1u16)));
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
    v1 = ((((*E).mbn_var_33) as i16).wrapping_add((384i16)));
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
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int
            - 16 as ::core::ffi::c_int) as uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
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
    if (((124i16).wrapping_sub(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)) as i16),
    )) as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        v0 = -(64 as ::core::ffi::c_int) as uint16;
    }
    MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(v0, game);
}
pub unsafe fn MotherBrain_MoveSamusForFallingAfterBeam(game: &mut Game) -> uint8 {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    v1 = ((((*E).mbn_parameter_1) as i16).wrapping_add((2i16)));
    if v1 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        v1 = 0 as int16;
    }
    (*E).mbn_parameter_1 = v1 as uint16;
    MotherBrain_MoveSamusHorizTowardsWall(v1 as uint16, game);
    let mut v2: uint16 = (((*E).mbn_parameter_2).wrapping_add((24u16)));
    (*E).mbn_parameter_2 = v2;
    return ((MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(v2, game)) & (1u8));
}
pub unsafe fn MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(
    mut a: uint16,
    game: &mut Game,
) -> uint8 {
    let mut carry: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0xafc) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            + *(&raw mut a as *mut uint8).offset(0) as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xafc) as *mut uint16 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) = carry as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb16) as *mut uint16 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) = carry as uint8;
    let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
        as ::core::ffi::c_int
        + (a as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
        + (carry >> 8 as ::core::ffi::c_int)) as uint16;
    let mut v6: uint16 = 0;
    if v4 as ::core::ffi::c_int - 48 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v6 = 48 as uint16;
    } else {
        if v4 as ::core::ffi::c_int - 192 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        {
            let ref mut fresh2 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
            *fresh2 = v4;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) = *fresh2;
            return 0 as uint8;
        }
        v6 = 192 as uint16;
    }
    let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    *fresh3 = v6;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) = *fresh3;
    *((&raw mut game.g_ram as *mut uint8).offset(0xafc) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb16) as *mut uint16) = 0 as uint16;
    return 1 as uint8;
}
pub unsafe fn MotherBrain_MoveSamusHorizTowardsWall(mut a: uint16, game: &mut Game) -> uint8 {
    let mut carry: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf8) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            + *(&raw mut a as *mut uint8).offset(0) as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf8) as *mut uint16 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) = carry as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb12) as *mut uint16 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) = carry as uint8;
    let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
        as ::core::ffi::c_int
        + (a as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
        + (carry >> 8 as ::core::ffi::c_int)) as uint16;
    if v4 as ::core::ffi::c_int - 235 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) = v4;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) = v4;
        return 0 as uint8;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) = 235 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) = 235 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf8) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb12) as *mut uint16) = 0 as uint16;
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
            addr_kEproj_MotherBrainRainbowBeamExplosion as uint16,
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
            .offset(6),
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
        let mut v2: uint16 = ((v1).wrapping_add((2u16)));
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
        ((j).wrapping_add((30u16))),
        15 as uint16,
        game,
    );
}
pub unsafe fn MotherBrain_Phase2Cut_0(game: &mut Game) {
    let mut v0: uint16 = SuitDamageDivision(0x50 as uint16, game);
    if (4 as ::core::ffi::c_int * v0 as ::core::ffi::c_int + 20 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
            (((0xa9bd98i32) as u16) & (0xffffu16));
    } else if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
        as ::core::ffi::c_int
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
            - *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16)
                as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
            || {
                v1 = addr_kMotherBrain_Ilist_9ECC as uint16;
                (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                    as ::core::ffi::c_int
                    & 0xfff as ::core::ffi::c_int)
                    < 0xff0 as ::core::ffi::c_int
            }
        {
            v1 = addr_kMotherBrain_Ilist_9D7F as uint16;
        }
        MotherBrain_SetBrainInstrs(v1, game);
    };
}
pub unsafe fn MotherBrain_Phase2Cut_1(game: &mut Game) {
    if MotherBrain_MakeHerStandUp(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_A = (((0xa9bda9i32) as u16) & (0xffffu16));
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
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9B7F as uint16, game);
        (*E).mbn_var_A = (((0xa9bdc1i32) as u16) & (0xffffu16));
        (*E).mbn_var_F = 256 as uint16;
    }
}
pub unsafe fn MotherBrain_Phase2Cut_3(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9F6C as uint16, game);
        (*E).mbn_var_A = (((0xa9bdd2i32) as u16) & (0xffffu16));
        MotherBrain_Phase2Cut_4(game);
    }
}
pub unsafe fn MotherBrain_Phase2Cut_4(game: &mut Game) {
    if ProcessSpriteTilesTransfers(0xa9 as uint8, addr_stru_A98FE5 as uint16, game) != 0 {
        MotherBrain_RetractHead(game);
        MotherBrain_SpawnShitroidInCutscene(game);
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_A = (((0xa9bdedi32) as u16) & (0xffffu16));
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
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C77 as uint16, game);
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_32 = 6 as uint16;
        (*E1).mbn_var_33 = 6 as uint16;
        (*E1).mbn_var_34 = 1280 as uint16;
        QueueSfx2_Max6(0x71 as uint16, game);
        (*E).mbn_var_A = (((0xa9be1ai32) as u16) & (0xffffu16));
    }
}
pub unsafe fn MotherBrain_SpawnShitroidInCutscene(game: &mut Game) {
    (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_0A =
        SpawnEnemy(0xa9 as uint8, addr_stru_A9BE28 as uint16, game);
}
pub unsafe fn MotherBrain_DrainedByShitroid_0(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_00 = 3 as uint16;
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E1).mbn_var_32 = 8 as uint16;
    (*E1).mbn_var_33 = 8 as uint16;
    (*E1).mbn_var_34 = 1792 as uint16;
    (*E).mbn_var_A = (((0xa9be5di32) as u16) & (0xffffu16));
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
        (*E).mbn_var_A = (((0xa9be96i32) as u16) & (0xffffu16));
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_08 = (((0xa9bfd0i32) as u16) & (0xffffu16));
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
    CallMotherBrainPainfulWalk(((((*E1).mbn_var_08) as u32) | (0xa90000u32)), game);
    let mut v2: uint16 = ((2u16).wrapping_mul(((*E1).mbn_var_06)));
    (*E1).mbn_var_07 = g_byte_A9BEEE[v2 as usize] as uint16;
    (*E1).mbn_var_34 =
        g_word_A9BEFE[(v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
    if (*E1).mbn_var_06 as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_16 = 0 as uint16;
        (*E1).mbn_var_10 = 0 as uint16;
        MotherBrain_WritePhase2DeathPalette(game);
        QueueSfx1_Max6(2 as uint16, game);
        (*E).mbn_var_A = (((0xa9bf0ei32) as u16) & (0xffffu16));
    }
}
pub unsafe fn MotherBrain_DrainedByShitroid_3(game: &mut Game) {
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    CallMotherBrainPainfulWalk(((((*E1).mbn_var_08) as u32) | (0xa90000u32)), game);
    if !((*E1).mbn_var_06 as ::core::ffi::c_int - 8 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0)
    {
        (*E1).mbn_var_34 = 64 as uint16;
        (*E1).mbn_var_32 = 8 as uint16;
        (*E1).mbn_var_33 = 8 as uint16;
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C39 as uint16, game);
        (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
            (((0xa9bf41i32) as u16) & (0xffffu16));
        MotherBrain_DrainedByShitroid_4(game);
    }
}
pub unsafe fn MotherBrain_DrainedByShitroid_4(game: &mut Game) {
    if MotherBrain_MakeWalkBackwards(0x28 as uint16, 0 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
            (((0xa9bf56i32) as u16) & (0xffffu16));
        (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_33 = 0 as uint16;
        MotherBrain_DrainedByShitroid_5(game);
    }
}
pub unsafe fn MotherBrain_DrainedByShitroid_5(game: &mut Game) {
    let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if *(&raw mut (*E1).mbn_var_32 as *mut uint32) == 0 {
        (*E1).mbn_var_12 = (((*E1).mbn_var_33) | ((*E1).mbn_var_32));
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        if (*E).mbn_var_02 == 0 {
            MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9A26 as uint16, game);
            (*E).mbn_var_A = (((0xa9bf7di32) as u16) & (0xffffu16));
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
        (*E).mbn_var_A = (((0xa9bf95i32) as u16) & (0xffffu16));
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
    if v2 && {
        (*E).mbn_var_F = 16 as uint16;
        v3 = (((*E).mbn_var_37).wrapping_add((1u16)));
        (*E).mbn_var_37 = v3;
        MotherBrain_FadeToGray_Drained(((v3).wrapping_sub((1u16))), game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
    } {
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).base.health = -(29536 as ::core::ffi::c_int) as uint16;
        (*E).mbn_var_1F = 1 as uint16;
        (*E1).mbn_var_14 = 0 as uint16;
        (*E).mbn_var_00 = 2 as uint16;
        (*E).mbn_var_A = (((0xa9c059i32) as u16) & (0xffffu16));
    }
}
pub unsafe fn MotherBrain_PainfulWalkForwards(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if MotherBrain_MakeWalkForwards((*E).mbn_var_07, 0x48 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*E).mbn_var_08 = (((0xa9bfe8i32) as u16) & (0xffffu16));
        MotherBrain_SetPainfulWalkingTimer(game);
    }
}
pub unsafe fn MotherBrain_PainfulWalkingForwards(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    let mut v1: uint16 = (((*E).mbn_var_09).wrapping_sub((1u16)));
    (*E).mbn_var_09 = v1;
    if v1 == 0 {
        (*E).mbn_var_06 = (*E).mbn_var_06.wrapping_add(1);
        (*E).mbn_var_08 = (((0xa9c004i32) as u16) & (0xffffu16));
    }
}
pub unsafe fn MotherBrain_PainfulWalkBackwards(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if MotherBrain_MakeWalkBackwards(0x28 as uint16, (*E).mbn_var_07, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*E).mbn_var_08 = (((0xa9c01ci32) as u16) & (0xffffu16));
        MotherBrain_SetPainfulWalkingTimer(game);
    }
}
pub unsafe fn MotherBrain_PainfulWalkingBackwards(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    let mut v1: uint16 = (((*E).mbn_var_09).wrapping_sub((1u16)));
    (*E).mbn_var_09 = v1;
    if v1 == 0 {
        (*E).mbn_var_06 = (*E).mbn_var_06.wrapping_add(1);
        (*E).mbn_var_08 = (((0xa9bfd0i32) as u16) & (0xffffu16));
    }
}
pub unsafe fn MotherBrain_SetPainfulWalkingTimer(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E).mbn_var_09 = *((&raw const g_word_A9C049 as *const int16).offset((*E).mbn_var_06 as isize)
        as *const int16 as *mut uint8)
        .offset(0) as uint16;
}
pub unsafe fn MotherBrain_Phase2_Revive_0(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_A = (((0xa9c066i32) as u16) & (0xffffu16));
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
        (*E).mbn_var_A = (((0xa9c082i32) as u16) & (0xffffu16));
        (*E).mbn_var_F = 224 as uint16;
        MotherBrain_Phase2_Revive_2(game);
    }
}
pub unsafe fn MotherBrain_Phase2_Revive_2(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_A = (((0xa9c08fi32) as u16) & (0xffffu16));
    (*E).mbn_var_37 = 0 as uint16;
    MotherBrain_Phase2_Revive_3(game);
}
pub unsafe fn MotherBrain_Phase2_Revive_3(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    let mut v1: bool = (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    let mut v2: uint16 = 0;
    if v1 && {
        (*E).mbn_var_F = 16 as uint16;
        v2 = (((*E).mbn_var_37).wrapping_add((1u16)));
        (*E).mbn_var_37 = v2;
        MotherBrain_FadeFromGray_Drained(((v2).wrapping_sub((1u16))), game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
    } {
        (*E).mbn_var_A = (((0xa9c0bai32) as u16) & (0xffffu16));
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
        (*E).mbn_var_A = (((0xa9c0e4i32) as u16) & (0xffffu16));
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
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9BB3 as uint16, game);
        (*E).mbn_var_A = (((0xa9c0fbi32) as u16) & (0xffffu16));
        (*E).mbn_var_F = 128 as uint16;
        MotherBrain_Phase2_Revive_6(game);
    }
}
pub unsafe fn MotherBrain_Phase2_Revive_6(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    let mut v1: bool = (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v1
        && MotherBrain_MakeWalkForwards(4 as uint16, 0x50 as uint16, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
    {
        (*E).mbn_var_A = (((0xa9c11ei32) as u16) & (0xffffu16));
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
    (*E).mbn_var_A = (((0xa9c147i32) as u16) & (0xffffu16));
    MotherBrain_Phase2_Revive_8(game);
}
pub unsafe fn MotherBrain_Phase2_Revive_8(game: &mut Game) {
    if MotherBrain_MakeHerStandUp(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
            (((0xa9c15ci32) as u16) & (0xffffu16));
        MotherBrain_MakeWalkForwards(0xa as uint16, 0x50 as uint16, game);
        MotherBrain_Phase2_MurderShitroid_1(game);
    }
}
pub unsafe fn MotherBrain_Phase2_MurderShitroid_1(game: &mut Game) {
    MotherBrain_MaybeStandupOrLeanDown(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut v0: uint16 = addr_kMotherBrain_Ilist_9DBB as uint16;
        if (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_0A != 0 {
            v0 = addr_kMotherBrain_Ilist_9DB1 as uint16;
        }
        MotherBrain_SetBrainInstrs(v0, game);
        let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
        (*E).mbn_var_A = (((0xa9c182i32) as u16) & (0xffffu16));
        (*E).mbn_var_F = 64 as uint16;
    }
}
pub unsafe fn MotherBrain_Phase2_MurderShitroid_2(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_A = (((0xa9c15ci32) as u16) & (0xffffu16));
    }
}
pub unsafe fn MotherBrain_Phase2_PrepareForFinalShitroid(game: &mut Game) {
    MotherBrain_MakeHerStandUp(game);
    MotherBrain_MakeWalkBackwards(0x40 as uint16, 4 as uint16, game);
}
pub unsafe fn MotherBrain_Phase2_ExecuteFinalkShitroid(game: &mut Game) {
    MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9DB1 as uint16, game);
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
        (((0xa9c1a6i32) as u16) & (0xffffu16));
}
pub unsafe fn MotherBrain_MaybeStandupOrLeanDown(game: &mut Game) {
    let mut mbn_var_02: uint16 = (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_02;
    if mbn_var_02 != 0 {
        if mbn_var_02 as ::core::ffi::c_int == 6 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as uint8
                as ::core::ffi::c_int
                >= 0xc0 as ::core::ffi::c_int
        {
            MotherBrain_MakeHerStandUp(game);
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as uint8
        as ::core::ffi::c_int
        >= 0xc0 as ::core::ffi::c_int
    {
        MotherBrain_MakeHerLeanDown(game);
    }
}
pub unsafe fn MotherBrain_Phase3_Recover_MakeDistance(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_00 = 4 as uint16;
    (*E).mbn_var_A = (((0xa9c1f0i32) as u16) & (0xffffu16));
    (*E).mbn_var_F = 32 as uint16;
    MotherBrain_MakeWalkBackwards((((*E).base.x_pos).wrapping_sub((14u16))), 2 as uint16, game);
}
pub unsafe fn MotherBrain_Phase3_Recover_SetupForFight(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_A = (((0xa9c209i32) as u16) & (0xffffu16));
        let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1).mbn_var_18 = (((0xa9c330i32) as u16) & (0xffffu16));
        (*E1).mbn_var_1A = (((0xa9c26ai32) as u16) & (0xffffu16));
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
            && *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
        {
            let mut v1: uint16 = addr_kMotherBrain_Ilist_9F00 as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as uint8
                as ::core::ffi::c_int
                >= 0x80 as ::core::ffi::c_int
            {
                v1 = addr_kMotherBrain_Ilist_9DBB as uint16;
            }
            MotherBrain_SetBrainInstrs(v1, game);
            (*E).mbn_var_A = (((0xa9c24ei32) as u16) & (0xffffu16));
            (*E).mbn_var_F = 64 as uint16;
        }
    } else {
        (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A =
            (((0xa9aee1i32) as u16) & (0xffffu16));
    };
}
pub unsafe fn MotherBrain_Phase3_Fighting_Cooldown(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if (*E).mbn_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).mbn_var_A = (((0xa9c209i32) as u16) & (0xffffu16));
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
            ((((*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_1A) as u32)
                | (0xa90000u32)),
            game,
        );
    }
}
pub unsafe fn MotherBrain_Phase3_Walk_TryToInchForward(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let mut mbn_var_07: uint16 = (*E).mbn_var_07;
    if mbn_var_07 != 0 {
        let mut v3: uint16 = ((mbn_var_07).wrapping_add((32u16)));
        (*E).mbn_var_07 = v3;
        if v3 as ::core::ffi::c_int >= 0x100 as ::core::ffi::c_int {
            let mut v4: uint16 = (((*E).base.x_pos).wrapping_add((1u16)));
            let mut E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
            (*E1).mbn_var_1B = v4;
            if MotherBrain_MakeWalkForwards(
                ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                    as ::core::ffi::c_int
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
        let mut v6: uint16 = (((*E).base.x_pos).wrapping_sub((14u16)));
        let mut E1_0: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
        (*E1_0).mbn_var_1B = v6;
        (*E1_0).mbn_var_1A = (((0xa9c2b3i32) as u16) & (0xffffu16));
        MotherBrain_Phase3_Walk_RetreatQuickly(game);
    };
}
pub unsafe fn MotherBrain_Phase3_Walk_RetreatQuickly(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    if MotherBrain_MakeWalkBackwards((*E).mbn_var_1B, 2 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*E).mbn_var_1B =
            (((*Get_MotherBrain(0 as ::core::ffi::c_int, game)).base.x_pos).wrapping_sub((14u16)));
        (*E).mbn_var_1A = (((0xa9c2d2i32) as u16) & (0xffffu16));
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
    (*E1).mbn_var_1A = (((0xa9c26ai32) as u16) & (0xffffu16));
    (*E1).mbn_var_1B = (((*E).base.x_pos).wrapping_add((1u16)));
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
        ((((*Get_MotherBrain(0x40 as ::core::ffi::c_int, game)).mbn_var_18) as u32)
            | (0xa90000u32)),
        game,
    );
}
pub unsafe fn MotherBrain_Phase3_Neck_Normal(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E).mbn_var_32 = 1 as uint16;
    (*E).mbn_var_34 = 128 as uint16;
    (*E).mbn_var_32 = 2 as uint16;
    (*E).mbn_var_33 = 4 as uint16;
    (*E).mbn_var_18 = addr_locret_A9C353 as uint16;
}
pub unsafe fn MotherBrain_Phase3_Neck_SetupRecoilRecovery(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E).mbn_var_31 = 1 as uint16;
    (*E).mbn_var_34 = 1280 as uint16;
    (*E).mbn_var_32 = 6 as uint16;
    (*E).mbn_var_33 = 6 as uint16;
    (*E).mbn_var_18 = (((0xa9c37bi32) as u16) & (0xffffu16));
    (*E).mbn_var_19 = 16 as uint16;
    MotherBrain_Phase3_Neck_RecoilRecovery(game);
}
pub unsafe fn MotherBrain_Phase3_Neck_RecoilRecovery(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    v1 = ((((*E).mbn_var_19) as i16).wrapping_sub((1i16)));
    if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9DBB as uint16, game);
        (*E).mbn_var_18 = (((0xa9c330i32) as u16) & (0xffffu16));
    } else {
        (*E).mbn_var_19 = v1 as uint16;
    };
}
pub unsafe fn MotherBrain_Phase3_Neck_SetupHyperBeamRecoil(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    (*E).mbn_var_31 = 1 as uint16;
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_06 = 1 as uint16;
    MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9BE7 as uint16, game);
    (*E).mbn_var_00 = 50 as uint16;
    (*E).mbn_var_34 = 2304 as uint16;
    (*E).mbn_var_32 = 8 as uint16;
    (*E).mbn_var_33 = 8 as uint16;
    (*E).mbn_var_18 = (((0xa9c3cdi32) as u16) & (0xffffu16));
    (*E).mbn_var_19 = 11 as uint16;
    MotherBrain_Phase3_Neck_HyperBeamRecoil(game);
}
pub unsafe fn MotherBrain_Phase3_Neck_HyperBeamRecoil(game: &mut Game) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    v2 = ((((*E).mbn_var_19) as i16).wrapping_sub((1i16)));
    if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).mbn_var_34 = 128 as uint16;
        (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_06 = 0 as uint16;
        (*E).mbn_var_18 = (((0xa9c354i32) as u16) & (0xffffu16));
    } else {
        (*E).mbn_var_19 = v2 as uint16;
    };
}
pub unsafe fn MoveEnemyWithVelocity(game: &mut Game) {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut ai_var_B: uint16 = (*v0).ai_var_B;
    let mut carry: ::core::ffi::c_int = *(&raw mut (*v0).x_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int
        + *(&raw mut ai_var_B as *mut uint8).offset(0) as ::core::ffi::c_int;
    *(&raw mut (*v0).x_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) = carry as uint8;
    (*v0).x_pos = ((*v0).x_pos as ::core::ffi::c_int
        + ((ai_var_B as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8
            as ::core::ffi::c_int
            + (carry >> 8 as ::core::ffi::c_int))) as uint16;
    let mut ai_var_C: uint16 = (*v0).ai_var_C;
    carry = *(&raw mut (*v0).y_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int
        + *(&raw mut ai_var_C as *mut uint8).offset(0) as ::core::ffi::c_int;
    *(&raw mut (*v0).y_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
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
    return sub_A9C46C(((r18).wrapping_add((64u16))), a, game);
}
pub unsafe fn sub_A9C46C(mut a: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v2: uint16 = kSinCosTable8bit_Sext[((a as ::core::ffi::c_int
        & 0xff as ::core::ffi::c_int)
        + 64 as ::core::ffi::c_int) as usize] as uint16;
    WriteReg(M7A as uint16, v2 as uint8, game);
    WriteReg(
        M7A as uint16,
        *(&raw mut v2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ),
        game,
    );
    WriteReg(M7B as uint16, j as uint8, game);
    return ReadRegWord(MPYM as uint16, game);
}
pub unsafe fn Enemy_IncreaseYpos(mut k: uint16, mut a: uint16, game: &mut Game) {
    let mut v2: *mut EnemyData = gEnemyData(k as ::core::ffi::c_int, game);
    let mut carry: ::core::ffi::c_int = *(&raw mut (*v2).y_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int
        + *(&raw mut a as *mut uint8).offset(0) as ::core::ffi::c_int;
    *(&raw mut (*v2).y_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) = carry as uint8;
    (*v2).y_pos = ((*v2).y_pos as ::core::ffi::c_int
        + ((a as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
            + (carry >> 8 as ::core::ffi::c_int))) as uint16;
}
pub unsafe fn Samus_DecrementAmmoDueToRainbowBeam(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe44) as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9c6) as *mut uint16) as ::core::ffi::c_int
            != 0
    {
        let mut v0: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x9c6)
            as *mut uint16))
            .wrapping_sub((g_word_A9C544)));
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9c6) as *mut uint16) as ::core::ffi::c_int
            - g_word_A9C544 as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9d2) as *mut uint16)
                as ::core::ffi::c_int
                == 1 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x9d2) as *mut uint16) = 0 as uint16;
            }
            v0 = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa04) as *mut uint16) = 0 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c6) as *mut uint16) = v0;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe44) as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9ca) as *mut uint16) as ::core::ffi::c_int
            != 0
    {
        let mut v1: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x9ca)
            as *mut uint16))
            .wrapping_sub((g_word_A9C544)));
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9ca) as *mut uint16) as ::core::ffi::c_int
            - g_word_A9C544 as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9d2) as *mut uint16)
                as ::core::ffi::c_int
                == 2 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x9d2) as *mut uint16) = 0 as uint16;
            }
            v1 = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa04) as *mut uint16) = 0 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x9ca) as *mut uint16) = v1;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9ce) as *mut uint16) != 0 {
        let mut v2: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x9ce)
            as *mut uint16))
            .wrapping_sub((g_word_A9C544)));
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9ce) as *mut uint16) as ::core::ffi::c_int
            - g_word_A9C544 as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9d2) as *mut uint16)
                as ::core::ffi::c_int
                == 3 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x9d2) as *mut uint16) = 0 as uint16;
            }
            v2 = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa04) as *mut uint16) = 0 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x9ce) as *mut uint16) = v2;
    }
}
pub unsafe fn Samus_PlayGainingLosingHealthSfx(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) as ::core::ffi::c_int
        - 81 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xe44) as *mut uint16) as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        QueueSfx3_Max3(0x2d as uint16, game);
    }
}
pub unsafe fn Samus_DamageDueToShitroid(game: &mut Game) {
    let mut v0: int16 = 0;
    v0 = -(4 as ::core::ffi::c_int) as int16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        v0 = -(3 as ::core::ffi::c_int) as int16;
    }
    let mut v1: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16)
        as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16)
            as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int)
        + v0 as ::core::ffi::c_int) as uint16;
    if v1 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v1 = 1 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) = v1;
}
pub unsafe fn Samus_DamageDueToRainbowBeam(game: &mut Game) {
    let mut v0: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16)
        as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16)
            as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int)
        - 2 as ::core::ffi::c_int) as uint16;
    if v0 as ::core::ffi::c_int - 1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v0 = 0 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) = v0;
    Samus_PlayGainingLosingHealthSfx(game);
}
pub unsafe fn Samus_HealDueToShitroid(game: &mut Game) -> uint8 {
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x9c4) as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16);
        *fresh4 = (*fresh4).wrapping_add(1);
        Samus_PlayGainingLosingHealthSfx(game);
        return 1 as uint8;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x9c4) as *mut uint16);
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
    let mut v4: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    let mut v5: *const uint8 = RomPtrWithBank(db, src, game);
    let mut v6: uint16 = *(v5 as *mut uint16);
    if v6 != 0 {
        v7 = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*v7).size = v6;
        *((&raw mut (*v7).src.addr as *mut uint8).offset(1) as *mut VoidP) =
            *(v5.offset(3) as *mut uint16);
        (*v7).src.addr = *(v5.offset(2) as *mut uint16);
        (*v7).vram_dst = *(v5.offset(5) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            ((v4).wrapping_add((7u16)));
        (*E).mbn_var_22 = ((src).wrapping_add((7u16)));
        v6 = *(RomPtrWithBank(db, (*E).mbn_var_22, game) as *mut uint16);
        if v6 != 0 {
            return 0 as uint8;
        }
    }
    (*E).mbn_var_22 = v6;
    return 1 as uint8;
}
pub unsafe fn MotherBrain_MakeWalkForwards(mut j: uint16, mut a: uint16, game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    if (((a) as i16).wrapping_sub((((*E).base.x_pos) as i16))) as ::core::ffi::c_int
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
    if ((((a) as i16).wrapping_sub((((*E).base.x_pos) as i16))) as ::core::ffi::c_int)
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
        let mut v1: uint16 = addr_kMotherBrain_Ilist_99C6 as uint16;
        if mbn_var_02 as ::core::ffi::c_int != 3 as ::core::ffi::c_int {
            if mbn_var_02 as ::core::ffi::c_int != 6 as ::core::ffi::c_int {
                return 0 as uint8;
            }
            v1 = addr_kMotherBrain_Ilist_99E2 as uint16;
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
        MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9A0A as uint16, game);
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
        MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_99F2 as uint16, game);
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
            v2 = ((mbn_var_07).wrapping_add((6u16)));
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
                MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_98C6 as uint16, game);
                return;
            }
            current_block_15 = 3512920355445576850;
        }
        match current_block_15 {
            3512920355445576850 => {
                if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                    as ::core::ffi::c_int
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
            MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_97A4 as uint16, game);
        }
        return;
    }
}
pub unsafe fn ShitroidInCutscene_Init(game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_DisableSamusColl as ::core::ffi::c_int
            | kEnemyProps_BlockPlasmaBeam as ::core::ffi::c_int))
        as uint16;
    (*E).base.palette_index = 3584 as uint16;
    (*E).base.current_instruction = addr_kShitroid_Ilist_CFA2 as uint16;
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
    (*E).sice_var_A = (((0xa9c7cci32) as u16) & (0xffffu16));
    (*E).sice_var_F = 248 as uint16;
    (*E).sice_var_0F = (((0xa9f683i32) as u16) & (0xffffu16));
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
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.shake_timer = 0 as uint16;
    CallShitroidCutsceneFuncA(
        ((((*E).sice_var_A) as u32) | (0xa90000u32)),
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    MoveEnemyWithVelocity(game);
    ShitroidInCutscene_Flashing(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    ShitroidInCutscene_HandleHealthBasedPalette(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let mut r18: uint16 = (*Get_ShitdroidInCutscene(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .sice_var_0F;
    CallShitroidCutsceneFunc0F((((r18) as u32) | (0xa90000u32)), game);
}
pub unsafe fn ShitroidInCutscene_Flashing(mut k: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    v1 = 3584 as int16;
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    let mut sice_var_06: uint16 = (*E).sice_var_06;
    if sice_var_06 != 0 {
        let mut v4: uint16 = ((sice_var_06).wrapping_sub((1u16)));
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
        (*E).sice_var_A = (((0xa9c7eci32) as u16) & (0xffffu16));
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
        (*E).sice_var_A = (((0xa9c811i32) as u16) & (0xffffu16));
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
        v3 = (((((*E).sice_var_F) as i16).wrapping_sub((1i16))) as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int;
        (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
        v3 as ::core::ffi::c_int != 0
    } {
        (*Get_ShitroidInCutscene(k as ::core::ffi::c_int, game)).sice_var_A =
            (((0xa9c851i32) as u16) & (0xffffu16));
        SomeMotherBrainScripts(1 as uint16, game);
    }
}
pub unsafe fn ShitroidInCutscene_LatchOntoBrain(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitroidInCutscene(0x40 as ::core::ffi::c_int, game);
    let mut rect: Rect16U = Rect16U {
        x: (*E).base.x_pos,
        y: (((*E).base.y_pos).wrapping_sub((24u16))),
        w: 8 as uint16,
        h: 8 as uint16,
    };
    Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0 as uint16, rect.x, rect.y, game);
    if Shitroid_Func_2(k, rect, game) == 0 {
        (*Get_ShitroidInCutscene(k as ::core::ffi::c_int, game)).sice_var_A =
            (((0xa9c879i32) as u16) & (0xffffu16));
    }
}
pub unsafe fn ShitroidInCutscene_SetMotherBrainToStumbleBack(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitroidInCutscene(0 as ::core::ffi::c_int, game);
    MotherBrain_MakeWalkBackwards((((*E).base.x_pos).wrapping_sub((1u16))), 2 as uint16, game);
    (*Get_ShitroidInCutscene(k as ::core::ffi::c_int, game)).sice_var_A =
        (((0xa9c889i32) as u16) & (0xffffu16));
    ShitroidInCutscene_ActivateRainbowBeam(k, game);
}
pub unsafe fn ShitroidInCutscene_ActivateRainbowBeam(mut k: uint16, game: &mut Game) {
    let mut E1: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(0x40 as ::core::ffi::c_int, game);
    if Shitroid_AccelerateTowardsPoint(
        k,
        0x200 as uint16,
        (*E1).base.x_pos,
        (((*E1).base.y_pos).wrapping_sub((24u16))),
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
        (*E).base.y_pos = (((*E1).base.y_pos).wrapping_sub((24u16)));
        Enemy_SetInstrList(k, addr_kShitroid_Ilist_CFB8 as uint16, game);
        (*E).sice_var_A = (((0xa9c8e2i32) as u16) & (0xffffu16));
        (*E).sice_var_E = 1 as uint16;
        let mut E0: *mut Enemy_ShitroidInCutscene =
            Get_ShitdroidInCutscene(0 as ::core::ffi::c_int, game);
        (*E0).sice_var_A = (((0xa9be38i32) as u16) & (0xffffu16));
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
    (*E).base.x_pos = (((*E1).base.x_pos).wrapping_add(((g_word_A993BB[v2 as usize]) as u16)));
    (*E).base.y_pos = ((*E1).base.y_pos as ::core::ffi::c_int
        + g_word_A993C3[v2 as usize] as ::core::ffi::c_int
        - 24 as ::core::ffi::c_int) as uint16;
    if (*Get_ShitdroidInCutscene(0 as ::core::ffi::c_int, game)).sice_var_1F != 0 {
        (*E).sice_var_A = (((0xa9c915i32) as u16) & (0xffffu16));
        (*E).sice_var_F = 64 as uint16;
    }
}
pub unsafe fn ShitroidInCutscene_StopDraining(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitroidInCutscene(k as ::core::ffi::c_int, game);
    let mut E1: *mut Enemy_ShitroidInCutscene =
        Get_ShitroidInCutscene(0x40 as ::core::ffi::c_int, game);
    (*E).base.x_pos = (*E1).base.x_pos;
    (*E).base.y_pos = (((*E1).base.y_pos).wrapping_sub((24u16)));
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    if (*E).sice_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        Enemy_SetInstrList(k, addr_kShitroid_Ilist_CFA2 as uint16, game);
        (*E).sice_var_E = 10 as uint16;
        (*E).sice_var_A = (((0xa9c94bi32) as u16) & (0xffffu16));
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
        (*E).sice_var_A = addr_loc_A9C959 as uint16;
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
        (*E).sice_var_A = (((0xa9c9c3i32) as u16) & (0xffffu16));
        (*E).sice_var_0E = addr_stru_A9CA24 as uint16;
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
        x: (((*E).base.x_pos).wrapping_add((xd))),
        y: (((*E).base.y_pos).wrapping_add((yd))),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, 9 as uint16, game);
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
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        & 0xfff as ::core::ffi::c_int
        >= 0xfa0 as ::core::ffi::c_int
    {
        QueueSfx2_Max6(0x52 as uint16, game);
    }
    let mut sice_var_0E: uint16 = (*E).sice_var_0E;
    let mut v3: *const uint16 = RomPtr_A9(sice_var_0E, game) as *const uint16;
    let mut rect: Rect16U = Rect16U {
        x: *v3.offset(0),
        y: *v3.offset(1),
        w: 4 as uint16,
        h: 4 as uint16,
    };
    CallShitroidMoveFunc(
        (((*v3.offset(3)) as u32) | (0xa90000u32)),
        k,
        *v3.offset(2),
        rect.x,
        rect.y,
        game,
    );
    if Shitroid_Func_2(k, rect, game) == 0 {
        v4 = *(RomPtr_A9(sice_var_0E, game) as *mut uint16).offset(4) as int16;
        if (v4 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            (*E).sice_var_A = v4 as uint16;
        } else {
            (*E).sice_var_0E = (((*E).sice_var_0E).wrapping_add((8u16)));
        }
    }
}
pub unsafe fn ShitroidInCutscene_LatchOntoSamus(mut k: uint16, game: &mut Game) {
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        k,
        0 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16),
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
            .wrapping_sub((20u16))),
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
    (*E).base.x_pos = ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
        .wrapping_add(((g_word_A993BB[v2 as usize]) as u16)));
    (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
        as ::core::ffi::c_int
        + g_word_A993C3[v2 as usize] as ::core::ffi::c_int
        - 20 as ::core::ffi::c_int) as uint16;
    if Samus_HealDueToShitroid(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x9d4) as *mut uint16);
        (*E).sice_var_A = (((0xa9cabdi32) as u16) & (0xffffu16));
        (*E).sice_var_0F = (((0xa9cabci32) as u16) & (0xffffu16));
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
        (*E).base.x_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int * g_word_A993BB[v3 as usize] as ::core::ffi::c_int)
            as uint16;
        (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
            as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int * g_word_A993C3[v3 as usize] as ::core::ffi::c_int
            - 20 as ::core::ffi::c_int) as uint16;
    }
    if (*E).base.health == 0 {
        (*E).base.health = 320 as uint16;
        (*E).sice_var_A = (((0xa9cb13i32) as u16) & (0xffffu16));
        (*E).sice_var_E = 10 as uint16;
        (*E).sice_var_D = 0 as uint16;
        (*E).sice_var_0F = (((0xa9f68fi32) as u16) & (0xffffu16));
        (*E).sice_var_04 = 1 as uint16;
        (*E).sice_var_09 = 0 as uint16;
    }
}
pub unsafe fn ShitroidInCutscene_ReleaseSamus(mut k: uint16, game: &mut Game) {
    QueueSfx2_Max6(0x72 as uint16, game);
    (*Get_ShitdroidInCutscene(0x40 as ::core::ffi::c_int, game)).sice_var_0B = 1 as uint16;
    (*Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game)).sice_var_A =
        (((0xa9cb2di32) as u16) & (0xffffu16));
    (*Get_ShitdroidInCutscene(0 as ::core::ffi::c_int, game)).sice_var_A =
        (((0xa9c18ei32) as u16) & (0xffffu16));
    ShitroidInCutscene_StareDownMotherBrain(k, game);
}
pub unsafe fn ShitroidInCutscene_StareDownMotherBrain(mut k: uint16, game: &mut Game) {
    let mut rect: Rect16U = Rect16U {
        x: ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
            .wrapping_sub((4u16))),
        y: 96 as uint16,
        w: 4 as uint16,
        h: 4 as uint16,
    };
    Shitroid_GraduallyAccelerateTowards0x10(k, 0 as uint16, rect.x, rect.y, game);
    if Shitroid_Func_2(k, rect, game) == 0 {
        (*Get_ShitroidInCutscene(k as ::core::ffi::c_int, game)).sice_var_A =
            (((0xa9cb56i32) as u16) & (0xffffu16));
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
            (((0xa9cb7bi32) as u16) & (0xffffu16));
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
            (((0xa9c19ai32) as u16) & (0xffffu16));
        (*E).sice_var_A = (((0xa9cbb3i32) as u16) & (0xffffu16));
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
            (((0xa9cbd8i32) as u16) & (0xffffu16));
    }
}
pub unsafe fn ShitroidInCutscene_FinalCharge(mut k: uint16, game: &mut Game) {
    let mut E1: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(0x40 as ::core::ffi::c_int, game);
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        k,
        0xc as uint16,
        (*E1).base.x_pos,
        (((*E1).base.y_pos).wrapping_sub((32u16))),
        game,
    );
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game);
    if (*E).base.health == 0 {
        (*E).sice_var_0F = (((0xa9cc98i32) as u16) & (0xffffu16));
        (*E).base.vram_tiles_index = 4256 as uint16;
        QueueSfx3_Max6(0x19 as uint16, game);
        TurnOffLightsForShitroidDeath(game);
        Enemy_SetInstrList(k, addr_kShitroid_Ilist_CFCE as uint16, game);
        (*E).sice_var_B = 0 as uint16;
        (*E).sice_var_C = 0 as uint16;
        (*Get_ShitdroidInCutscene(0 as ::core::ffi::c_int, game)).sice_var_A =
            addr_locret_A9C18D as uint16;
        QueueMusic_Delayed8(0 as uint16, game);
        (*E).sice_var_A = (((0xa9cc3ei32) as u16) & (0xffffu16));
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
        (*E).sice_var_A = (((0xa9cc60i32) as u16) & (0xffffu16));
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
        (*E).sice_var_A = (((0xa9cc7fi32) as u16) & (0xffffu16));
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
            (((0xa9cd30i32) as u16) & (0xffffu16));
        (*E).sice_var_A = (((0xa9cc99i32) as u16) & (0xffffu16));
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
        (*E).base.properties = (((*E).base.properties) | ((kEnemyProps_Invisible) as u16));
        (*E).sice_var_A = (((0xa9ccc0i32) as u16) & (0xffffu16));
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
    if v2
        && ProcessSpriteTilesTransfers(0xa9 as uint8, addr_stru_A98FC7 as uint16, game)
            as ::core::ffi::c_int
            != 0
    {
        (*E).sice_var_A = (((0xa9ccdei32) as u16) & (0xffffu16));
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
        (*E).sice_var_A = (((0xa9ccf0i32) as u16) & (0xffffu16));
        (*Get_ShitdroidInCutscene(0 as ::core::ffi::c_int, game)).sice_var_37 = 0 as uint16;
        ShitroidInCutscene_FinishCutscene(k, game);
    }
}
pub unsafe fn ShitroidInCutscene_FinishCutscene(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(0 as ::core::ffi::c_int, game);
    let mut v2: uint16 = (((*E).sice_var_37).wrapping_add((1u16)));
    (*E).sice_var_37 = v2;
    if MotherBrain_Phase3_TurnLightsBackOn(((v2).wrapping_sub((1u16))), game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*E).sice_var_A = (((0xa9c1cfi32) as u16) & (0xffffu16));
        CallSomeSamusCode(0x17 as uint16, game);
        SomeMotherBrainScripts(3 as uint16, game);
        let fresh12 = &raw mut (*Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game))
            .base
            .properties;
        fresh12.write_unaligned(((fresh12.read_unaligned()) | ((kEnemyProps_Deleted) as u16)));
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
    CallSamusRainbowPaletteFunc((((r18) as u32) | (0xa90000u32)), k, game);
}
pub unsafe fn SamusRainbowPaletteFunc_ActivateWhenEnemyLow(mut k: uint16, game: &mut Game) {
    if ((*Get_ShitdroidInCutscene(k as ::core::ffi::c_int, game))
        .base
        .y_pos as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        CallSomeSamusCode(0x16 as uint16, game);
        (*Get_ShitdroidInCutscene(0x40 as ::core::ffi::c_int, game)).sice_var_16 =
            (((0xa9cd4bi32) as u16) & (0xffffu16));
    }
}
pub unsafe fn SamusRainbowPaletteFunc_GraduallySlowDown(game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(0x40 as ::core::ffi::c_int, game);
    let mut sice_var_1D: uint16 = (*E).sice_var_1D;
    (*E).sice_var_1D = ((sice_var_1D).wrapping_add((768u16)));
    if sice_var_1D as ::core::ffi::c_int >= 0xfd00 as ::core::ffi::c_int {
        let mut v2: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xace)
            as *mut uint16))
            .wrapping_add((1u16)));
        if *((&raw mut game.g_ram as *mut uint8).offset(0xace) as *mut uint16) as ::core::ffi::c_int
            - 9 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v2 = 10 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xace) as *mut uint16) = v2;
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
    v2 = ((((*E).sice_var_0C) as i16).wrapping_sub((1i16)));
    if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        (*E).sice_var_0C = v2 as uint16;
        return 0 as uint8;
    }
    (*E).sice_var_0C = 8 as uint16;
    let mut v4: uint16 = (((*E).sice_var_0D).wrapping_add((1u16)));
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
    v2 = ((((*E).sice_var_08) as i16).wrapping_sub((1i16)));
    if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).sice_var_08 = 4 as uint16;
        let mut v3: uint16 = (((*E).sice_var_07).wrapping_add((1u16)));
        if (*E).sice_var_07 as ::core::ffi::c_int - 9 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v3 = 0 as uint16;
        }
        (*E).sice_var_07 = v3;
        let mut v4: ::core::ffi::c_int =
            ((4u16).wrapping_mul((v3))) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        game.eproj_spawn_pt = Point16U {
            x: (((*E).base.x_pos).wrapping_add(
                (*(game.g_rom.offset(
                    (((0xa9cdfc as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa9cdfc as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v4 as isize)),
            )),
            y: (((*E).base.y_pos).wrapping_add(
                (*(game.g_rom.offset(
                    (((0xa9cdfc as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa9cdfc as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v4 + 1 as ::core::ffi::c_int) as isize)),
            )),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, 3 as uint16, game);
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
        (*E).base.properties = (((*E).base.properties) | ((kEnemyProps_Invisible) as u16));
    };
}
pub unsafe fn ShitroidInCutscene_AccelerateDownwards(mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_ShitroidInCutscene =
        Get_ShitroidInCutscene(k as ::core::ffi::c_int, game);
    v2 = (((abs16((*E).sice_var_B)) as i16).wrapping_sub((32i16)));
    if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        v2 = 0 as int16;
    }
    (*E).sice_var_B = (if (*E).sice_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    {
        -(v2 as ::core::ffi::c_int)
    } else {
        v2 as ::core::ffi::c_int
    }) as uint16;
    (*E).sice_var_C = (((*E).sice_var_C).wrapping_add((2u16)));
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
                ((v4).wrapping_add((8u16))),
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
    (*E).base.x_pos = (((*E).sice_var_10).wrapping_add(((g_word_A993BB[v2 as usize]) as u16)));
    (*E).base.y_pos = (((*E).sice_var_11).wrapping_add(((g_word_A993C3[v2 as usize]) as u16)));
}
pub unsafe fn ShitroidInCutscene_Touch(game: &mut Game) {
    let mut E: *mut Enemy_ShitroidInCutscene = Get_ShitroidInCutscene(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).sice_var_A as ::core::ffi::c_int
        == 0xa9ca66 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        if Shitroid_AccelerateTowardsPoint(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            0x10 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16),
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
                .wrapping_sub((20u16))),
            game,
        ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            (*E).sice_var_B = 0 as uint16;
            (*E).sice_var_C = 0 as uint16;
            (*E).sice_var_A = (((0xa9ca7ai32) as u16) & (0xffffu16));
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
        if (((r22) as i16).wrapping_sub((((*E).sice_var_0B) as i16))) as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            let mut v3: uint16 = (((*E).sice_var_0B).wrapping_add((32u16)));
            if v3 as ::core::ffi::c_int - r22 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v3 = r22;
            }
            (*E).sice_var_0B = v3;
        } else {
            let mut v2: uint16 = (((*E).sice_var_0B).wrapping_sub((32u16)));
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
        v4 = (((*E).sice_var_0A).wrapping_add((r18)));
        if v4 as ::core::ffi::c_int - r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        {
            current_block_13 = 17187448977823727056;
        } else {
            current_block_13 = 12039483399334584727;
        }
    } else {
        v4 = (((*E).sice_var_0A).wrapping_add((r18)));
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
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
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
                    let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xc000)
                        as *mut uint16)
                        .offset((v2 + 129 as ::core::ffi::c_int) as isize);
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset((v2 + 145 as ::core::ffi::c_int) as isize) = v3;
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset((v2 + 177 as ::core::ffi::c_int) as isize) = v3;
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset((v2 + 65 as ::core::ffi::c_int) as isize) = v3;
                    i -= 2 as ::core::ffi::c_int;
                }
            }
        }
    }
}
pub unsafe fn MotherBrain_Pal_BeginScreenFlash(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_0E = addr_stru_A9D046 as uint16;
    (*E).mbn_var_0F = 1 as uint16;
}
pub unsafe fn MotherBrain_Pal_EndScreenFlash(game: &mut Game) {
    let mut E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_0E = 0 as uint16;
    (*E).mbn_var_0F = 0 as uint16;
    MotherBrain_Pal_WriteRoomPal(addr_word_A9D082 as uint16, game);
}
pub unsafe fn MotherBrain_Pal_HandleRoomPal(game: &mut Game) {
    let mut j: uint16 = HandleMotherBrainInstructionList(
        (((&raw mut game.g_ram as *mut uint8).offset(0x7800) as *mut ExtraEnemyRam7800
            as *mut uint8)
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
        ((j).wrapping_add((24u16))),
        12 as uint16,
        game,
    );
    WriteColorsToPalette(
        0xe6 as uint16,
        0xa9 as uint8,
        ((j).wrapping_add((24u16))),
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
    let mut v3: *const uint16 = RomPtr_A9(v2, game) as *mut uint16;
    if *v3.offset(0) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        current_block = 2809726207655838168;
    } else {
        if (*st).timer as ::core::ffi::c_int != *v3.offset(0) as ::core::ffi::c_int {
            (*st).timer = (*st).timer.wrapping_add(1);
            return *v3.offset(1);
        }
        v2 = ((v2).wrapping_add((4u16)));
        current_block = 6937071982253665452;
    }
    loop {
        match current_block {
            2809726207655838168 => {
                v2 = CallMotherBrainInstr(
                    (((*v3.offset(0)) as u32) | (0xa90000u32)),
                    ((v2).wrapping_add((2u16))),
                    game,
                );
                current_block = 6937071982253665452;
            }
            _ => {
                v3 = RomPtr_A9(v2, game) as *mut uint16;
                if *v3.offset(0) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
                    break;
                } else {
                    current_block = 2809726207655838168;
                }
            }
        }
    }
    if *v3.offset(0) == 0 {
        (*st).ip = 0 as uint16;
        (*st).timer = 0 as uint16;
        return 0 as uint16;
    } else {
        (*st).timer = 1 as uint16;
        (*st).ip = v2;
        return *v3.offset(1);
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
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as int8;
        if mbn_var_D_high != 0 {
            *(&raw mut (*E).mbn_var_D as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1 as usize) as isize,
            ) = (((mbn_var_D_high) as u8).wrapping_sub((1u8)));
        } else {
            let mut r18: uint16 = *(game.g_rom.offset(
                (((0xa9d260 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa9d260 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    (*(&raw mut (*E).mbn_var_E as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1 as usize) as isize,
                    ) as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                );
            *(&raw mut (*E).mbn_var_D as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
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
                *(&raw mut (*E).mbn_var_D as *mut uint8).offset(0) = v4;
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
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *(RomPtrWithBank(db, j, game) as *mut uint16);
        k = ((k).wrapping_add((2u16)));
        j = ((j).wrapping_add((2u16)));
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
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            *(RomPtrWithBank(db, j, game) as *mut uint16);
        k = ((k).wrapping_add((2u16)));
        j = ((j).wrapping_add((2u16)));
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
}
pub unsafe fn DeadTorizo_Init(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 4094 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let mut E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0 as ::core::ffi::c_int, game);
    (*E).dto_var_A = (((0xa9d3adi32) as u16) & (0xffffu16));
    (*E).base.properties = (((*E).base.properties) | (0xa000u16));
    (*E).base.current_instruction = addr_kDeadTorizo_Ilist_D6DC as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.palette_index = 512 as uint16;
    (*E).dto_var_B = 0 as uint16;
    (*E).dto_var_C = 8 as uint16;
    (*E).dto_var_04 = 0 as uint16;
    (*E).dto_var_02 = 15 as uint16;
    (*E).dto_var_03 = 0 as uint16;
    InitializeEnemyCorpseRotting(0 as uint16, addr_stru_A9DD58 as uint16, game);
}
pub unsafe fn CallDeadTorizoFuncA(mut ea: uint32, game: &mut Game) {
    match ea {
        11129773 => {
            DeadTorizo_WaitForSamusColl(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
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
        (*E).base.properties = (((*E).base.properties) | ((kEnemyProps_Tangible) as u16));
        (*E).dto_var_A = (((0xa9d3e6i32) as u16) & (0xffffu16));
    }
    mov24(
        (&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr,
        0xa9d39a as uint32,
    );
    CallDeadTorizoFuncA(((((*E).dto_var_A) as u32) | (0xa90000u32)), game);
    DeadTorizo_Func_1(game);
}
pub unsafe fn DeadTorizo_MainGfxHook(game: &mut Game) {
    MotherBrain_AddSpritemapToOam(
        addr_kDeadTorizo_Sprmap_D761 as uint16,
        296 as uint16,
        187 as uint16,
        0 as uint16,
        game,
    );
}
pub unsafe fn DeadTorizo_WaitForSamusColl(mut k: uint16, game: &mut Game) {
    if k as ::core::ffi::c_int
        == *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(0)
            as ::core::ffi::c_int
        || k as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(1)
                as ::core::ffi::c_int
        || k as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(2)
                as ::core::ffi::c_int
        || k as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(3)
                as ::core::ffi::c_int
    {
        (*Get_DeadTorizo(0 as ::core::ffi::c_int, game)).dto_var_A =
            (((0xa9d3c8i32) as u16) & (0xffffu16));
    }
}
pub unsafe fn DeadTorizo_PreRotDelay(game: &mut Game) {
    let mut E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 = (((*E).dto_var_04).wrapping_add((1u16)));
    (*E).dto_var_04 = v1;
    if v1 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
        (*E).base.properties = (((*E).base.properties) | ((kEnemyProps_Tangible) as u16));
        (*E).dto_var_A = (((0xa9d3e6i32) as u16) & (0xffffu16));
        DeadTorizo_Rotting(game);
    }
}
pub unsafe fn DeadTorizo_Rotting(game: &mut Game) {
    let mut E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 = (((*E).dto_var_03).wrapping_add((1u16)));
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
        (*E).dto_var_A = (((0xa9d3c7i32) as u16) & (0xffffu16));
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
    (*E).base.properties = (((*E).base.properties) | ((kEnemyProps_Tangible) as u16));
    (*E).dto_var_A = (((0xa9d3e6i32) as u16) & (0xffffu16));
}
pub unsafe fn DeadTorizo_Func_0(game: &mut Game) -> uint8 {
    let mut v5: int16 = 0;
    let mut v3: uint16 = 0;
    let mut v4: uint16 = 0;
    let mut E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0 as ::core::ffi::c_int, game);
    let mut r18: uint16 = (*E).base.x_pos;
    let mut r20: uint16 = (*E).base.y_pos;
    let mut p: *const uint8 = RomPtr_A9(addr_kDeadTorizo_Hitbox_D77C as uint16_t, game);
    let mut R22: ::core::ffi::c_int = *(p as *mut uint16) as ::core::ffi::c_int;
    let mut R24: uint16 = 0;
    p = p.offset(2);
    while R22 != 0 {
        if (((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)) as i16)
            .wrapping_sub(((r20) as i16))) as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            R24 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
                .wrapping_sub((r20)));
            v3 = *(p.offset(6) as *mut uint16);
        } else {
            R24 = ((r20).wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)),
            ));
            v3 = *(p.offset(2) as *mut uint16);
        }
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16)
            as ::core::ffi::c_int
            + abs16(v3) as ::core::ffi::c_int
            - R24 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            if (((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
                .wrapping_sub(((r18) as i16))) as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                R24 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
                    .wrapping_sub((r18)));
                v4 = *(p.offset(4) as *mut uint16);
            } else {
                R24 = ((r18).wrapping_sub(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)),
                ));
                v4 = *(p as *mut uint16);
            }
            v5 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16)
                as ::core::ffi::c_int
                + abs16(v4) as ::core::ffi::c_int
                - R24 as ::core::ffi::c_int) as int16;
            if v5 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                if v5 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    v5 = 4 as int16;
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = v5 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16) = 4 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5a) as *mut uint16) = 0 as uint16;
                return 1 as uint8;
            }
        }
        p = p.offset(8);
        R22 -= 1;
    }
    return 0 as uint8;
}
pub unsafe fn DeadTorizo_Func_1(game: &mut Game) {
    let mut v5: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v10: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 = (((*E).dto_var_00).wrapping_add((1u16)));
    (*E).dto_var_00 = v1;
    let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    if v1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut v8: uint16 = 0 as uint16;
        let mut v9: uint16 = *(game.g_rom.offset(
            (((0xa9d583 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa9d583 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(0);
        loop {
            v10 = gVramWriteEntry(v3 as ::core::ffi::c_int, game);
            (*v10).size = v9;
            let mut v11: ::core::ffi::c_int = v8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut (*v10).src.addr as *mut uint8).offset(1) as *mut VoidP) =
                *(game.g_rom.offset(
                    (((0xa9d583 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa9d583 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v11 + 1 as ::core::ffi::c_int) as isize);
            (*v10).src.addr = *(game.g_rom.offset(
                (((0xa9d583 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa9d583 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v11 + 2 as ::core::ffi::c_int) as isize);
            (*v10).vram_dst = *(game.g_rom.offset(
                (((0xa9d583 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa9d583 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v11 + 3 as ::core::ffi::c_int) as isize);
            v3 = ((v3).wrapping_add((7u16)));
            v8 = ((v8).wrapping_add((8u16)));
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
            .offset(0);
        loop {
            v5 = gVramWriteEntry(v3 as ::core::ffi::c_int, game);
            (*v5).size = v4;
            let mut v6: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut (*v5).src.addr as *mut uint8).offset(1) as *mut VoidP) =
                *(game.g_rom.offset(
                    (((0xa9d549 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa9d549 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v6 + 1 as ::core::ffi::c_int) as isize);
            (*v5).src.addr = *(game.g_rom.offset(
                (((0xa9d549 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa9d549 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v6 + 2 as ::core::ffi::c_int) as isize);
            (*v5).vram_dst = *(game.g_rom.offset(
                (((0xa9d549 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa9d549 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v6 + 3 as ::core::ffi::c_int) as isize);
            v3 = ((v3).wrapping_add((7u16)));
            v2 = ((v2).wrapping_add((8u16)));
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
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) = v3;
}
pub unsafe fn DeadTorizo_CorpseRottingFinished(game: &mut Game) {
    game.eproj_spawn_pt = Point16U {
        x: ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
            + 272 as ::core::ffi::c_int) as uint16,
        y: 188 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as uint16,
        0xa as uint16,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe44) as *mut uint16) as ::core::ffi::c_int
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
        .offset(0x9500)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset(v2 as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9510)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 8 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9520)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 16 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9530)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 24 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9540)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 32 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9550)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 40 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9560)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 48 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9570)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 56 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9580)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 64 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9590)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 72 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x95a0)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 80 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x95b0)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 88 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x95c0)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 96 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x95d0)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 104 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x95e0)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 112 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x95f0)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 120 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9600)
        .offset(v1 as ::core::ffi::c_int as isize) as *mut uint16) = *(game.g_rom.offset(
        (((0xb7a800 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb7a800 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v2 + 128 as ::core::ffi::c_int) as isize);
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9610)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
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
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
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
        (*E).base.properties = (((*E).base.properties) | ((kEnemyProps_Deleted) as u16));
    }
    (*E).dsr_var_08 = 0 as uint16;
    (*E).dsr_var_0A = 96 as uint16;
    (*E).dsr_var_0B = 256 as uint16;
    (*E).base.x_pos = 488 as uint16;
    (*E).base.y_pos = 184 as uint16;
    (*E).dsr_var_A = (((0xa9d8e2i32) as u16) & (0xffffu16));
    (*E).base.palette_index = 512 as uint16;
    (*E).base.y_height = 21 as uint16;
    Enemy_SetInstrList(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kDeadMonsters_Ilist_ECE3 as uint16,
        game,
    );
    InitializeEnemyCorpseRotting(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_stru_A9DD68 as uint16,
        game,
    );
}
pub unsafe fn DeadSidehopper_Init_1(game: &mut Game) {
    let mut E: *mut Enemy_DeadSidehopper = Get_DeadSidehopper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).dsr_var_08 = -(1 as ::core::ffi::c_int) as uint16;
    (*E).dsr_var_A = (((0xa9da64i32) as u16) & (0xffffu16));
    (*E).base.palette_index = 3584 as uint16;
    Enemy_SetInstrList(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kDeadMonsters_Ilist_ECEF as uint16,
        game,
    );
    InitializeEnemyCorpseRotting(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_stru_A9DD78 as uint16,
        game,
    );
}
pub unsafe fn DeadZoomer_Init(game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).palette_index = 3584 as uint16;
    (*v1).ai_var_A = (((0xa9da69i32) as u16) & (0xffffu16));
    let mut v2: ::core::ffi::c_int =
        (*v1).parameter_1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    Enemy_SetInstrList(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
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
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
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
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).palette_index = 3584 as uint16;
    (*v1).ai_var_A = (((0xa9da73i32) as u16) & (0xffffu16));
    let mut v2: ::core::ffi::c_int =
        (*v1).parameter_1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    Enemy_SetInstrList(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
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
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
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
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).palette_index = 3584 as uint16;
    (*v1).ai_var_A = (((0xa9da6ei32) as u16) & (0xffffu16));
    let mut v2: ::core::ffi::c_int =
        (*v1).parameter_1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    Enemy_SetInstrList(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
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
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
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
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
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
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallDeadSidehopperFunc(
        ((((*E).dsr_var_A) as u32) | (0xa90000u32)),
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn DeadSidehopper_Alive_WaitForActivate(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) as ::core::ffi::c_int
        - 513 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*Get_DeadSidehopper(k as ::core::ffi::c_int, game)).dsr_var_A =
            (((0xa9d8f1i32) as u16) & (0xffffu16));
        DeadSidehopper_Activated(k, game);
    }
}
pub unsafe fn DeadSidehopper_Activated(mut k: uint16, game: &mut Game) {
    if DeadMonsters_Func_3(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut E: *mut Enemy_DeadSidehopper = Get_DeadSidehopper(k as ::core::ffi::c_int, game);
        (*E).dsr_var_06 = ((*E).dsr_var_06 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int) as uint16;
        Enemy_SetInstrList(k, addr_kDeadMonsters_Ilist_ECAC as uint16, game);
        (*E).dsr_var_A = (((0xa9d90fi32) as u16) & (0xffffu16));
    }
}
pub unsafe fn DeadMonsters_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as ::core::ffi::c_int, game);
    (*E).dms_var_A = (((0xa9d91di32) as u16) & (0xffffu16));
    (*E).dms_var_F = 64 as uint16;
}
pub unsafe fn DeadMonsters_Func_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as ::core::ffi::c_int, game);
    (*E).dms_var_F = (*E).dms_var_F.wrapping_sub(1);
    if (*E).dms_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if (*E).dms_var_08 != 0 {
            (*E).dms_var_A = (((0xa9da08i32) as u16) & (0xffffu16));
        } else {
            (*E).dms_var_A = (((0xa9d8f1i32) as u16) & (0xffffu16));
            Enemy_SetInstrList(k, addr_kDeadMonsters_Ilist_ECE3 as uint16, game);
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
    let mut v3: uint16 = (((*E).dms_var_0B).wrapping_add((v2)));
    (*E).dms_var_0B = v3;
    if (*E).base.x_pos as ::core::ffi::c_int - 544 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut full: ::core::ffi::c_int = *(&raw mut (*E).base.y_subpos as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            + *(&raw mut v3 as *mut uint8).offset(0) as ::core::ffi::c_int;
        *(&raw mut (*E).base.y_subpos as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) = full as uint8;
        let mut v12: uint16 = ((*E).base.y_pos as ::core::ffi::c_int
            + (v3 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
            + (full >> 8 as ::core::ffi::c_int)) as uint16;
        (*E).base.y_pos = v12;
        return (v12 as ::core::ffi::c_int - 184 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int) as uint8;
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
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            + *(&raw mut a as *mut uint8).offset(0) as ::core::ffi::c_int;
        *(&raw mut (*E).base.x_subpos as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
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
    let mut v2: uint16 = (((*E).dms_var_07).wrapping_add((1u16)));
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
        let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
        let mut v5: uint16 = (((*E).dms_var_08).wrapping_add((1u16)));
        (*E).dms_var_08 = v5;
        if v5 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
            Enemy_SetInstrList(v3, addr_kDeadMonsters_Ilist_ECE9 as uint16, game);
            (*E).dms_var_A = (((0xa9da64i32) as u16) & (0xffffu16));
            (*E).base.properties = (((*E).base.properties) | (0x8000u16));
            (*E).base.y_height = 12 as uint16;
        }
    }
}
pub unsafe fn DeadSidehopper_WaitForSamusColl(mut k: uint16, game: &mut Game) {
    DeadMonsters_WaitForSamusColl(k, (((0xa9da8fi32) as u16) & (0xffffu16)), game);
}
pub unsafe fn DeadZoomer_WaitForSamusColl(mut k: uint16, game: &mut Game) {
    DeadMonsters_WaitForSamusColl(k, (((0xa9da94i32) as u16) & (0xffffu16)), game);
}
pub unsafe fn DeadSkree_WaitForSamusColl(mut k: uint16, game: &mut Game) {
    DeadMonsters_WaitForSamusColl(k, (((0xa9da9ei32) as u16) & (0xffffu16)), game);
}
pub unsafe fn DeadRipper_WaitForSamusColl(mut k: uint16, game: &mut Game) {
    DeadMonsters_WaitForSamusColl(k, (((0xa9da99i32) as u16) & (0xffffu16)), game);
}
pub unsafe fn DeadMonsters_WaitForSamusColl(mut k: uint16, mut j: uint16, game: &mut Game) {
    if k as ::core::ffi::c_int
        == *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(0)
            as ::core::ffi::c_int
        || k as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(1)
                as ::core::ffi::c_int
        || k as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(2)
                as ::core::ffi::c_int
        || k as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(3)
                as ::core::ffi::c_int
    {
        (*Get_DeadMonsters(k as ::core::ffi::c_int, game)).dms_var_A = j;
    }
}
pub unsafe fn DeadSidehopper_PreRotDelay(mut k: uint16, game: &mut Game) {
    DeadMonsters_PreRotDelay_Common(k, (((0xa9dabai32) as u16) & (0xffffu16)), game);
}
pub unsafe fn DeadZoomer_PreRotDelay(mut k: uint16, game: &mut Game) {
    DeadMonsters_PreRotDelay_Common(k, (((0xa9dad0i32) as u16) & (0xffffu16)), game);
}
pub unsafe fn DeadRipper_PreRotDelay(mut k: uint16, game: &mut Game) {
    DeadMonsters_PreRotDelay_Common(k, (((0xa9dae6i32) as u16) & (0xffffu16)), game);
}
pub unsafe fn DeadSkree_PreRotDelay(mut k: uint16, game: &mut Game) {
    DeadMonsters_PreRotDelay_Common(k, (((0xa9dafci32) as u16) & (0xffffu16)), game);
}
pub unsafe fn DeadMonsters_PreRotDelay_Common(mut k: uint16, mut j: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as ::core::ffi::c_int, game);
    (*E).dms_var_B = (*E).dms_var_B.wrapping_add(1);
    if (*E).dms_var_B as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
        (*E).dms_var_A = j;
        (*E).base.properties = (((*E).base.properties) | ((kEnemyProps_Tangible) as u16));
    }
}
pub unsafe fn DeadSidehopper_Rotting(mut k: uint16, game: &mut Game) {
    let mut v1: uint8 = ProcessCorpseRotting(k, game);
    if v1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        (*Get_DeadMonsters(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .dms_var_A = (((0xa9da64i32) as u16) & (0xffffu16));
    }
    let mut dms_var_53: uint16 = (*Get_DeadMonsters(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .dms_var_53;
    ProcessCorpseRottingVramTransfers(dms_var_53, game);
}
pub unsafe fn DeadZoomer_Rotting(mut k: uint16, game: &mut Game) {
    let mut v1: uint8 = ProcessCorpseRotting(k, game);
    if v1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        (*Get_DeadMonsters(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .dms_var_A = (((0xa9da63i32) as u16) & (0xffffu16));
    }
    let mut dms_var_53: uint16 = (*Get_DeadMonsters(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .dms_var_53;
    ProcessCorpseRottingVramTransfers(dms_var_53, game);
}
pub unsafe fn DeadRipper_Rotting(mut k: uint16, game: &mut Game) {
    let mut v1: uint8 = ProcessCorpseRotting(k, game);
    if v1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        (*Get_DeadMonsters(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .dms_var_A = (((0xa9da63i32) as u16) & (0xffffu16));
    }
    let mut dms_var_53: uint16 = (*Get_DeadMonsters(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .dms_var_53;
    ProcessCorpseRottingVramTransfers(dms_var_53, game);
}
pub unsafe fn DeadSkree_Rotting(mut k: uint16, game: &mut Game) {
    let mut v1: uint8 = ProcessCorpseRotting(k, game);
    if v1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        (*Get_DeadMonsters(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .dms_var_A = (((0xa9da63i32) as u16) & (0xffffu16));
    }
    let mut dms_var_53: uint16 = (*Get_DeadMonsters(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
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
            v7 = *v5.offset(1);
            if v7 != 0 {
                v8 = ((v7).wrapping_sub((1i16)));
                *v5.offset(1) = v8;
                if (v8 as uint16 as ::core::ffi::c_int) < 4 as ::core::ffi::c_int {
                    E0 = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
                    let mut r18: uint16 = (*E0).dms_var_42;
                    if v15 as ::core::ffi::c_int >= (*E0).dms_var_47 as ::core::ffi::c_int {
                        r18 = (*E0).dms_var_43;
                    }
                    CopyMoveCorpseRottingRotEntry(
                        RomPtr_A9((*E0).dms_var_44, game) as *const uint16,
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
                    RomPtr_A9((*E).dms_var_44, game) as *const uint16,
                    *v11,
                    r18_0,
                    game,
                );
                v12 = ((*v11).wrapping_add((2u16)));
                if (v12 as ::core::ffi::c_int) < (*E).dms_var_46 as ::core::ffi::c_int {
                    *v11 = v12;
                } else {
                    CallCorpseRottingFinish(((((*E).dms_var_49) as u32) | (0xa90000u32)), game);
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
        dms_var_52 = ((dms_var_52).wrapping_add((4u16)));
        v4 = ((v15).wrapping_add((1u16)));
        E0 = Get_DeadMonsters(0 as ::core::ffi::c_int, game);
        if (v15 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            - (*E0).dms_var_45 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            return (((v15).wrapping_add((1u16))) as ::core::ffi::c_int
                >= (*E0).dms_var_45 as ::core::ffi::c_int) as uint8;
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
    let mut v1: uint16 =
        (((a) & (0xfff8u16)) as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = ((a) & (7u16));
    let mut v6: uint16 = (*r20
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int * v2 as ::core::ffi::c_int) as uint16;
    if v2 as ::core::ffi::c_int >= 6 as ::core::ffi::c_int {
        CallCorpseRottingMove(
            (((r18) as u32) | (0xa90000u32)),
            (((*E).dms_var_48).wrapping_add((v6))),
            v6,
            game,
        );
    } else {
        CallCorpseRottingMove((((r18) as u32) | (0xa90000u32)), v6, v6, game);
    };
}
pub unsafe fn CorpseRottingRotEntryFinishedHook(game: &mut Game) {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    game.eproj_spawn_pt = Point16U {
        x: ((*v0).x_pos as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                as ::core::ffi::c_int
                & 0x1a as ::core::ffi::c_int)
            - 14 as ::core::ffi::c_int) as uint16,
        y: (((*v0).y_pos).wrapping_add((16u16))),
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as uint16,
        0xa as uint16,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe44) as *mut uint16) as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        QueueSfx2_Max3(0x10 as uint16, game);
    }
}
pub unsafe fn InitializeCorpseRottingDataTable(mut table: *mut uint16, mut a: uint16) {
    let mut v2: int16 = 0;
    v2 = (((a) as i16).wrapping_sub((1i16)));
    let mut r18: uint16 = 0 as uint16;
    loop {
        *table.offset(0) = v2 as uint16;
        *table.offset(1) = r18;
        table = table.offset(2);
        r18 = ((r18).wrapping_add((2u16)));
        v2 -= 1;
        if !(v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
pub unsafe fn InitializeEnemyCorpseRotting(mut k: uint16, mut j: uint16, game: &mut Game) {
    let mut v2: *const uint8 = RomPtr_A9(j, game);
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as ::core::ffi::c_int, game);
    (*E).dms_var_52 = *(v2 as *mut uint16);
    (*E).dms_var_53 = *(v2.offset(2) as *mut uint16);
    (*E).dms_var_54 = *(v2.offset(4) as *mut uint16);
    (*E).dms_var_55 = *(v2.offset(6) as *mut uint16);
    (*E).dms_var_5B = *(v2.offset(14) as *mut uint16);
    let mut v4: uint16 = *(v2.offset(12) as *mut uint16);
    (*E).dms_var_56 = v4;
    (*E).dms_var_5A = ((*(RomPtr_A9(v4, game) as *mut uint16).offset(1)).wrapping_sub((12u16)));
    let mut v5: *mut uint16 = v2 as *mut uint16;
    let mut v6: uint16 = *v5.offset(4);
    let fresh5 = v6;
    v6 = v6.wrapping_sub(1);
    (*E).dms_var_57 = fresh5;
    (*E).dms_var_58 = v6;
    (*E).dms_var_59 = ((v6).wrapping_sub((1u16)));
    InitializeCorpseRottingDataTable(
        (&raw mut game.g_ram as *mut uint8).offset(*v5 as ::core::ffi::c_int as isize)
            as *mut uint16,
        *v5.offset(4),
    );
    CallCorpseRottingInit((((*v5.offset(5)) as u32) | (0xa90000u32)), game);
}
pub unsafe fn ProcessCorpseRottingVramTransfers(mut k: uint16, game: &mut Game) {
    let mut p: *const uint8 = RomPtr_A9(k, game);
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    let mut v2: uint16 = *(p as *mut uint16);
    loop {
        let mut v3: *mut VramWriteEntry = gVramWriteEntry(v1 as ::core::ffi::c_int, game);
        (*v3).size = v2;
        *((&raw mut (*v3).src.addr as *mut uint8).offset(1) as *mut VoidP) =
            *(p as *mut uint16).offset(1);
        (*v3).src.addr = *(p as *mut uint16).offset(2);
        (*v3).vram_dst = *(p as *mut uint16).offset(3);
        v1 = ((v1).wrapping_add((7u16)));
        p = p.offset(8);
        v2 = *(p as *mut uint16);
        if !(v2 != 0) {
            break;
        }
    }
    (*Get_DeadEnemy(0 as ::core::ffi::c_int, game)).dey_var_22 = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) = v1;
}
pub unsafe fn DeadZoomer_Powerbomb(game: &mut Game) {
    if (*gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
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
    DeadSidehopper_DD34((((0xa9dad0i32) as u16) & (0xffffu16)), game);
}
pub unsafe fn DeadRipper_Powerbomb(game: &mut Game) {
    if (*gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
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
    DeadSidehopper_DD34((((0xa9dae6i32) as u16) & (0xffffu16)), game);
}
pub unsafe fn DeadSkree_Powerbomb(game: &mut Game) {
    if (*gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
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
    DeadSidehopper_DD34((((0xa9dafci32) as u16) & (0xffffu16)), game);
}
pub unsafe fn DeadSidehopper_Shot(game: &mut Game) {
    let mut E: *mut Enemy_DeadSidehopper = Get_DeadSidehopper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
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
    DeadSidehopper_DD34((((0xa9dabai32) as u16) & (0xffffu16)), game);
}
pub unsafe fn DeadSidehopper_DD34(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_DeadSidehopper = Get_DeadSidehopper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).dsr_var_A = a;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn DeadSidehopper_Touch(game: &mut Game) {
    if ((*Get_DeadSidehopper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
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
    let mut p: *const uint8 = RomPtr_B7(addr_kDeadTorizo_TileData as uint16_t, game);
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x60 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(288) as *const ::core::ffi::c_void,
        0xc0 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x1a0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(800) as *const ::core::ffi::c_void,
        0xc0 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x2c0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1280) as *const ::core::ffi::c_void,
        0x100 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x400 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1792) as *const ::core::ffi::c_void,
        0x100 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x540 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2304) as *const ::core::ffi::c_void,
        0x100 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x680 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2816) as *const ::core::ffi::c_void,
        0x100 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x7c0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(3328) as *const ::core::ffi::c_void,
        0x100 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x900 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(3840) as *const ::core::ffi::c_void,
        0x100 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xa40 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(4352) as *const ::core::ffi::c_void,
        0x100 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xb60 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(4832) as *const ::core::ffi::c_void,
        0x120 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xc80 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(5312) as *const ::core::ffi::c_void,
        0x140 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xdc0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(5824) as *const ::core::ffi::c_void,
        0x140 as ::core::ffi::c_int,
    );
}
pub unsafe fn Sidehopper_CorpseRottingInitFunc_0(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(addr_kDeadMonsters_TileData as uint16_t, game);
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x40 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(64) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xa0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(512) as *const ::core::ffi::c_void,
        0xa0 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x140 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1024) as *const ::core::ffi::c_void,
        0xa0 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x1e0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1536) as *const ::core::ffi::c_void,
        0xa0 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x280 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2048) as *const ::core::ffi::c_void,
        0xa0 as ::core::ffi::c_int,
    );
}
pub unsafe fn Sidehopper_CorpseRottingInitFunc_2(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(addr_kDeadMonsters_TileData as uint16_t, game);
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x320 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(288) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x3c0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(800) as *const ::core::ffi::c_void,
        0xa0 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x460 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1312) as *const ::core::ffi::c_void,
        0xa0 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x500 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1824) as *const ::core::ffi::c_void,
        0xa0 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x5a0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2336) as *const ::core::ffi::c_void,
        0xa0 as ::core::ffi::c_int,
    );
}
pub unsafe fn Zoomer_CorpseRottingInitFunc_0(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(addr_kDeadMonsters_TileData as uint16_t, game);
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x940 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2656) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x9a0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(3168) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
}
pub unsafe fn Zoomer_CorpseRottingInitFunc_2(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(addr_kDeadMonsters_TileData as uint16_t, game);
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xa00 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2752) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xa60 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(3264) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
}
pub unsafe fn Zoomer_CorpseRottingInitFunc_4(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(addr_kDeadMonsters_TileData as uint16_t, game);
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xac0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2848) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xb20 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(3360) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
}
pub unsafe fn Ripper_CorpseRottingInitFunc_0(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(addr_kDeadMonsters_TileData as uint16_t, game);
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xb80 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2560) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xbe0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(3072) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
}
pub unsafe fn Ripper_CorpseRottingInitFunc_2(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(addr_kDeadMonsters_TileData as uint16_t, game);
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xc40 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2944) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0xca0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(3456) as *const ::core::ffi::c_void,
        0x60 as ::core::ffi::c_int,
    );
}
pub unsafe fn Skree_CorpseRottingInitFunc_0(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(addr_kDeadMonsters_TileData as uint16_t, game);
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x640 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(672) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x680 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1184) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x6c0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1696) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x700 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(2208) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
}
pub unsafe fn Skree_CorpseRottingInitFunc_2(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(addr_kDeadMonsters_TileData as uint16_t, game);
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x740 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(224) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x780 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(736) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x7c0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1248) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x800 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1760) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
}
pub unsafe fn Skree_CorpseRottingInitFunc_4(game: &mut Game) {
    let mut p: *const uint8 = RomPtr_B7(addr_kDeadMonsters_TileData as uint16_t, game);
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x840 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(448) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x880 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(960) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x8c0 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1472) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
    MemCpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset((0x2000 as ::core::ffi::c_int + 0x900 as ::core::ffi::c_int) as isize)
            as *mut uint8 as *mut ::core::ffi::c_void,
        p.offset(1984) as *const ::core::ffi::c_void,
        0x40 as ::core::ffi::c_int,
    );
}
pub unsafe fn MotherBrain_CorpseRottingInitFunc(game: &mut Game) {
    let mut base: *mut uint8 = (&raw mut game.g_ram as *mut uint8).offset(0x9000) as *mut uint8;
    let mut src: *const uint8 = RomPtr_B7(addr_kMotherBrain_Misc_TileData as uint16_t, game);
    MemCpy(
        base as *mut ::core::ffi::c_void,
        src.offset(192) as *const ::core::ffi::c_void,
        0xc0 as ::core::ffi::c_int,
    );
    MemCpy(
        base.offset(0xe0) as *mut ::core::ffi::c_void,
        src.offset(704) as *const ::core::ffi::c_void,
        0xc0 as ::core::ffi::c_int,
    );
    MemCpy(
        base.offset(0x1c0) as *mut ::core::ffi::c_void,
        src.offset(1216) as *const ::core::ffi::c_void,
        0xc0 as ::core::ffi::c_int,
    );
    MemCpy(
        base.offset(0x2a0) as *mut ::core::ffi::c_void,
        src.offset(1728) as *const ::core::ffi::c_void,
        0xc0 as ::core::ffi::c_int,
    );
    MemCpy(
        base.offset(0x380) as *mut ::core::ffi::c_void,
        src.offset(2240) as *const ::core::ffi::c_void,
        0xe0 as ::core::ffi::c_int,
    );
    MemCpy(
        base.offset(0x460) as *mut ::core::ffi::c_void,
        src.offset(2752) as *const ::core::ffi::c_void,
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
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v4 + 1 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset(v3 as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v4 + 9 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v3 + 8 as ::core::ffi::c_int) as isize);
        }
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(v5 as isize) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v5 + 8 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x48 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v6: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v7 + 17 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v6 + 16 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v7 + 25 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v6 + 24 as ::core::ffi::c_int) as isize);
        }
        let mut v8: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 16 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 24 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v9: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v10: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v10 + 33 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v9 + 32 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v10 + 41 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v9 + 40 as ::core::ffi::c_int) as isize);
        }
        let mut v11: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v11 + 32 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v11 + 40 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v12: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v13: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v13 + 49 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v12 + 48 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v13 + 57 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v12 + 56 as ::core::ffi::c_int) as isize);
    }
    let mut v14: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v14 + 48 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v14 + 56 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v15: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v15 + 65 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v14 + 64 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v15 + 73 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v14 + 72 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v14 + 64 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v14 + 72 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v16: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v16 + 81 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v14 + 80 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v16 + 89 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v14 + 88 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v14 + 80 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v14 + 88 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v17: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v17 + 97 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v14 + 96 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v17 + 105 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v14 + 104 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v14 + 96 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v14 + 104 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v18: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v18 + 113 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v14 + 112 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v18 + 121 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v14 + 120 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v14 + 112 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v14 + 120 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v19: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v19 + 129 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v14 + 128 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v19 + 137 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v14 + 136 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v14 + 128 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v14 + 136 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v20: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v20 + 145 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v14 + 144 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v20 + 153 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v14 + 152 as ::core::ffi::c_int) as isize);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v14 + 144 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset(v3 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 9 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 8 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x48 as ::core::ffi::c_int
        && (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 17 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 16 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 25 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 24 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int
        && (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 33 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v7 + 32 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 41 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v7 + 40 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v9: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v10: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v10 + 49 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v9 + 48 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v10 + 57 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v9 + 56 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v11: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v12: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v12 + 65 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v11 + 64 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v12 + 73 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v11 + 72 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v13: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v14: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v14 + 81 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v13 + 80 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v14 + 89 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v13 + 88 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v15: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v16: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v16 + 97 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v15 + 96 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v16 + 105 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v15 + 104 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v17: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v18: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v18 + 113 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v17 + 112 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v18 + 121 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v17 + 120 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v19: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v20: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v20 + 129 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v19 + 128 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v20 + 137 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v19 + 136 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x10 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 94 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v21: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v22: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v22 + 145 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v21 + 144 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v22 + 153 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v4 + 1 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset(v3 as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v4 + 9 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v3 + 8 as ::core::ffi::c_int) as isize);
        }
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(v5 as isize) =
            0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v5 + 8 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v6: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v7 + 17 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v6 + 16 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v7 + 25 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v6 + 24 as ::core::ffi::c_int) as isize);
        }
        let mut v8: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 16 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 24 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v9: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v10: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v10 + 33 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v9 + 32 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v10 + 41 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v9 + 40 as ::core::ffi::c_int) as isize);
    }
    let mut v11: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v11 + 32 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v11 + 40 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v12: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v12 + 49 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v11 + 48 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v12 + 57 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v11 + 56 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v11 + 48 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v11 + 56 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v13: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v13 + 65 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v11 + 64 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v13 + 73 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v11 + 72 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v11 + 64 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset(v3 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 9 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 8 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int
        && (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 17 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 16 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 25 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 24 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 33 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v7 + 32 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 41 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v7 + 40 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v9: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v10: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v10 + 49 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v9 + 48 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v10 + 57 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v9 + 56 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v11: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v12: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v12 + 65 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v11 + 64 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v12 + 73 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 401 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 400 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 409 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 408 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 400 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 408 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 417 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 416 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 425 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 424 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 416 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 424 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v7 + 433 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v5 + 432 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v7 + 441 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v5 + 440 as ::core::ffi::c_int) as isize);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v5 + 432 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v5 + 440 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v8 + 449 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v5 + 448 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v8 + 457 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v5 + 456 as ::core::ffi::c_int) as isize);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v5 + 448 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v5 + 456 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v9: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v9 + 465 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v5 + 464 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v9 + 473 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v5 + 472 as ::core::ffi::c_int) as isize);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v5 + 464 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 401 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 400 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 409 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 408 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 417 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 416 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 425 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 424 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int
        && (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 433 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v7 + 432 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 441 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v7 + 440 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int
        && (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let mut v9: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v10: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v10 + 449 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v9 + 448 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v10 + 457 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v9 + 456 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 38 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let mut v11: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v12: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v12 + 465 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v11 + 464 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v12 + 473 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1185 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1184 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1193 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1192 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1184 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1192 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1201 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1200 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1209 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1208 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1200 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1208 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v7 + 1217 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1216 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v7 + 1225 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1224 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1216 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1185 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1184 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1193 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1192 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1201 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1200 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1209 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1208 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 1217 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v7 + 1216 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 1225 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1281 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1280 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1289 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1288 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1280 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1288 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1297 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1296 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1305 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1304 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1296 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1304 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v7 + 1313 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1312 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v7 + 1321 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1320 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1312 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1281 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1280 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1289 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1288 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1297 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1296 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1305 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1304 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 1313 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v7 + 1312 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 1321 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1377 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1376 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1385 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1384 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1376 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1384 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1393 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1392 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1401 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1400 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1392 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1400 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v7 + 1409 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1408 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v7 + 1417 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1416 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1408 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1377 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1376 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1385 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1384 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1393 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1392 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1401 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1400 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 1409 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v7 + 1408 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 1417 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1473 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1472 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1481 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1480 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1472 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1480 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1489 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1488 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1497 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1496 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1488 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1496 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v7 + 1505 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1504 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v7 + 1513 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1512 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1504 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1473 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1472 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1481 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1480 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1489 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1488 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1497 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1496 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 1505 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v7 + 1504 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 1513 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1569 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1568 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1577 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1576 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1568 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1576 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1585 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1584 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1593 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1592 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1584 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1592 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v7 + 1601 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1600 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v7 + 1609 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1608 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1600 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1569 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1568 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1577 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1576 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1585 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1584 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1593 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1592 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v8: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 1601 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v7 + 1600 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v8 + 1609 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 801 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 800 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 809 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 808 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 800 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 808 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 817 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 816 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 825 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 824 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 816 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 801 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 800 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 809 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 808 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 817 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 816 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 825 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 929 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 928 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 937 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 936 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 928 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 936 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 945 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 944 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 953 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 952 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 944 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 929 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 928 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 937 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 936 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 945 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 944 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 953 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1057 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1056 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1065 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1064 as ::core::ffi::c_int) as isize);
    }
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1056 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1064 as ::core::ffi::c_int) as isize) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1073 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1072 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1081 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1080 as ::core::ffi::c_int) as isize);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v5 + 1072 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1057 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1056 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1065 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 + 1064 as ::core::ffi::c_int) as isize);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v6: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1073 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v5 + 1072 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 1081 as ::core::ffi::c_int) as isize) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
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
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9002) as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
                *(((&raw mut game.g_ram as *mut uint8).offset(0x9000) as *mut uint16 as *mut uint8)
                    .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9012) as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
                *(((&raw mut game.g_ram as *mut uint8).offset(0x900f) as *mut uint16 as *mut uint8)
                    .offset(k as ::core::ffi::c_int as isize)
                    .offset(1) as *mut uint16);
        }
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9000) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x900f) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize)
            .offset(1) as *mut uint16) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9022) as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
                *(((&raw mut game.g_ram as *mut uint8).offset(0x9020) as *mut uint16 as *mut uint8)
                    .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9032) as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
                *(((&raw mut game.g_ram as *mut uint8).offset(0x9030) as *mut uint16 as *mut uint8)
                    .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        }
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9020) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9030) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9042) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9040) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9052) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9050) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
    }
    *(((&raw mut game.g_ram as *mut uint8).offset(0x9040) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x9050) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9062) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9060) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9072) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9070) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
    }
    *(((&raw mut game.g_ram as *mut uint8).offset(0x9060) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x9070) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9082) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9080) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9092) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9090) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
    }
    *(((&raw mut game.g_ram as *mut uint8).offset(0x9080) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x9090) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x90a2) as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
                *(((&raw mut game.g_ram as *mut uint8).offset(0x90a0) as *mut uint16 as *mut uint8)
                    .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
            *(((&raw mut game.g_ram as *mut uint8).offset(0x90b2) as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
                *(((&raw mut game.g_ram as *mut uint8).offset(0x90b0) as *mut uint16 as *mut uint8)
                    .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        }
        *(((&raw mut game.g_ram as *mut uint8).offset(0x90a0) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x90b0) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x90c2) as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
                *(((&raw mut game.g_ram as *mut uint8).offset(0x90c0) as *mut uint16 as *mut uint8)
                    .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
            *(((&raw mut game.g_ram as *mut uint8).offset(0x90d2) as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
                *(((&raw mut game.g_ram as *mut uint8).offset(0x90d0) as *mut uint16 as *mut uint8)
                    .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        }
        *(((&raw mut game.g_ram as *mut uint8).offset(0x90c0) as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x90d0) as *mut uint16 as *mut uint8)
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
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9002) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9000) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9012) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x900f) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize)
                .offset(1) as *mut uint16);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int
        && (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9022) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9020) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9032) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9030) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9042) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9040) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9052) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9050) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9062) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9060) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9072) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9070) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9082) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9080) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9092) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9090) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 8 as ::core::ffi::c_int
        && (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x90a2) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x90a0) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x90b2) as *mut uint16 as *mut uint8)
            .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
            *(((&raw mut game.g_ram as *mut uint8).offset(0x90b0) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
    }
    if (*E).dms_var_41 as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int {
        if (*E).dms_var_41 as ::core::ffi::c_int - 46 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x90c2) as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
                *(((&raw mut game.g_ram as *mut uint8).offset(0x90c0) as *mut uint16 as *mut uint8)
                    .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
            *(((&raw mut game.g_ram as *mut uint8).offset(0x90d2) as *mut uint16 as *mut uint8)
                .offset(j as ::core::ffi::c_int as isize) as *mut uint16) =
                *(((&raw mut game.g_ram as *mut uint8).offset(0x90d0) as *mut uint16 as *mut uint8)
                    .offset(k as ::core::ffi::c_int as isize) as *mut uint16);
        }
    }
}
pub unsafe fn sub_A9ECD0(mut k: uint16, mut jp: *const uint16, game: &mut Game) -> *const uint16 {
    let mut v2: uint16 = (((0xa9d910i32) as u16) & (0xffffu16));
    let mut E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as ::core::ffi::c_int, game);
    if (*E).dms_var_08 != 0 {
        v2 = (((0xa9da08i32) as u16) & (0xffffu16));
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
    let mut v4: uint16 = abs16((((*E).base.x_pos).wrapping_sub(((*G).base.x_pos))));
    let mut v5: bool = v4 as ::core::ffi::c_int >= r18 as ::core::ffi::c_int;
    if (v4 as ::core::ffi::c_int) < r18 as ::core::ffi::c_int {
        r18 = ((*G).base.y_height as ::core::ffi::c_int
            + (*E).base.y_height as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint16;
        return (abs16((((*E).base.y_pos).wrapping_sub(((*G).base.y_pos)))) as ::core::ffi::c_int
            >= r18 as ::core::ffi::c_int) as uint8;
    }
    return v5 as uint8;
}
pub unsafe fn Shitroid_Func_2(mut k: uint16, mut rect: Rect16U, game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    let mut R26: uint16 = ((*E).base.x_width as ::core::ffi::c_int
        + rect.w as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = abs16(((rect.x).wrapping_sub(((*E).base.x_pos))));
    let mut v3: bool = v2 as ::core::ffi::c_int >= R26 as ::core::ffi::c_int;
    if (v2 as ::core::ffi::c_int) < R26 as ::core::ffi::c_int {
        R26 = ((*E).base.y_height as ::core::ffi::c_int
            + rect.h as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint16;
        return (abs16(((rect.y).wrapping_sub(((*E).base.y_pos)))) as ::core::ffi::c_int
            >= R26 as ::core::ffi::c_int) as uint8;
    }
    return v3 as uint8;
}
pub unsafe fn Shitroid_Init(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 4094 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_DisableSamusColl as ::core::ffi::c_int
            | kEnemyProps_BlockPlasmaBeam as ::core::ffi::c_int))
        as uint16;
    (*E).base.palette_index = 1024 as uint16;
    (*E).base.current_instruction = addr_kShitroid_Ilist_F90E as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    let mut v2: uint16 = (((0xa9efe6i32) as u16) & (0xffffu16));
    if *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | (kEnemyProps_Tangible as ::core::ffi::c_int
                | kEnemyProps_Invisible as ::core::ffi::c_int))
            as uint16;
        v2 = (((0xa9efdfi32) as u16) & (0xffffu16));
    }
    (*E).shitr_var_A = v2;
    (*E).shitr_var_B = 0 as uint16;
    (*E).shitr_var_C = 0 as uint16;
    (*E).shitr_var_E = 10 as uint16;
    (*E).shitr_parameter_2 = 0 as uint16;
    WriteColorsToTargetPalette(
        0xa9 as uint8,
        0x120 as uint16,
        addr_word_A9F8C6 as uint16,
        0x10 as uint16,
        game,
    );
    WriteColorsToTargetPalette(
        0xa9 as uint8,
        0x140 as uint16,
        addr_kShitroidInCutscene_Palette as uint16,
        0x10 as uint16,
        game,
    );
    WriteColorsToTargetPalette(
        0xa9 as uint8,
        0x1e0 as uint16,
        addr_kDeadSidehopper_Palette_0 as uint16,
        0x10 as uint16,
        game,
    );
}
pub unsafe fn Shitroid_Powerbomb(game: &mut Game) {
    if (*Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .shitr_parameter_2
        != 0
    {
        Shitroid_Func_26(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
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
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.health = 0x7fff as uint16;
    CallShitroidFunc(
        ((((*E).shitr_var_A) as u32) | (0xa90000u32)),
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    MoveEnemyWithVelocity(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16) == 0 {
        Shitroid_HandleNormalPalette(game);
    }
}
pub unsafe fn Shitroid_Func_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    (*E).shitr_var_B = 0 as uint16;
    (*E).shitr_var_C = 0 as uint16;
}
pub unsafe fn Shitroid_Func_4(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) as ::core::ffi::c_int
        - 513 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) = 512 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20) as *mut uint16) =
            *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(0) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) as *mut uint8
            as *mut uint16) =
            *(&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) as uint16;
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
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*E).shitr_var_A = (((0xa9f02bi32) as u16) & (0xffffu16));
        (*E).shitr_var_04 = 1 as uint16;
    }
}
pub unsafe fn Shitroid_Func_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    (*E).shitr_var_A = (((0xa9f037i32) as u16) & (0xffffu16));
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
        (*E).shitr_var_A = (((0xa9f049i32) as u16) & (0xffffu16));
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
            (((0xa9f06di32) as u16) & (0xffffu16));
    }
}
pub unsafe fn Shitroid_Func_8(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shitroid =
        Get_Shitroid(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        k,
        0xf as uint16,
        (*E).base.x_pos,
        (((*E).base.y_pos).wrapping_sub((32u16))),
        game,
    );
    if Shitroid_Func_1(k, ((k).wrapping_add((64u16))), game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        (*Get_Shitroid(k as ::core::ffi::c_int, game)).shitr_var_A =
            (((0xa9f094i32) as u16) & (0xffffu16));
    }
}
pub unsafe fn Shitroid_Func_9(mut k: uint16, game: &mut Game) {
    let mut E1: *mut Enemy_Shitroid =
        Get_Shitroid(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    if Shitroid_AccelerateTowardsPoint(
        k,
        0x200 as uint16,
        (*E1).base.x_pos,
        (((*E1).base.y_pos).wrapping_sub((32u16))),
        game,
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
        (*E).shitr_var_B = 0 as uint16;
        (*E).shitr_var_C = 0 as uint16;
        (*E).base.x_pos = (*E1).base.x_pos;
        (*E).base.y_pos = (((*E1).base.y_pos).wrapping_sub((32u16)));
        (*E).base.current_instruction = addr_kShitroid_Ilist_F924 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).shitr_var_A = (((0xa9f0e6i32) as u16) & (0xffffu16));
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
    (*E).base.x_pos = (((*E1).base.x_pos).wrapping_add(((g_word_A993BB[v2 as usize]) as u16)));
    (*E).base.y_pos = ((*E1).base.y_pos as ::core::ffi::c_int
        + g_word_A993C3[v2 as usize] as ::core::ffi::c_int
        - 32 as ::core::ffi::c_int) as uint16;
    let fresh13 = (*E).shitr_var_F;
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if fresh13 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).shitr_var_A = (((0xa9f125i32) as u16) & (0xffffu16));
        (*E).base.current_instruction = addr_kShitroid_Ilist_F906 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).shitr_var_E = 10 as uint16;
    }
}
pub unsafe fn Shitroid_Func_11(mut k: uint16, game: &mut Game) {
    (*Get_Shitroid(0x40 as ::core::ffi::c_int, game)).shitr_var_08 = 1 as uint16;
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    (*E).shitr_var_A = (((0xa9f138i32) as u16) & (0xffffu16));
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
        (*E).shitr_var_A = (((0xa9f180i32) as u16) & (0xffffu16));
        (*E).shitr_parameter_2 = 1 as uint16;
        let ref mut fresh14 = *((&raw mut game.g_ram as *mut uint8).offset(0xcd20) as *mut uint16);
        *fresh14 = ((*fresh14) | (0x100u16));
        let ref mut fresh15 = *((&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2)
            as *mut uint8 as *mut uint16);
        *fresh15 = ((*fresh15) | (0x100u16));
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
        (((*E).base.x_pos)
            .wrapping_sub((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)))),
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
        || *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            - 512 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        (*Get_Shitroid(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .shitr_var_A = (((0xa9f1fai32) as u16) & (0xffffu16));
    } else {
        let mut shitro_var_01: uint16 = (*E).shitr_var_01;
        let mut v5: uint16 = 0;
        if shitro_var_01 != 0 {
            (*E).shitr_var_01 = ((shitro_var_01).wrapping_sub((1u16)));
            v5 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
        } else {
            v5 = 80 as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                as ::core::ffi::c_int
                & 0xfff as ::core::ffi::c_int
                >= 0xfe0 as ::core::ffi::c_int
            {
                (*E).shitr_var_01 = 32 as uint16;
            }
        }
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            0xa as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16),
            v5,
            game,
        );
    };
}
pub unsafe fn Shitroid_Func_14(mut k: uint16, game: &mut Game) {
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        k,
        0xf as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16),
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
            .wrapping_sub((32u16))),
        game,
    );
}
pub unsafe fn Shitroid_Func_15(mut k: uint16, game: &mut Game) {
    CallSomeSamusCode(0x12 as uint16, game);
    (*Get_Shitroid(0 as ::core::ffi::c_int, game)).shitr_var_A =
        (((0xa9f21bi32) as u16) & (0xffffu16));
    Shitroid_Func_16(k, game);
}
pub unsafe fn Shitroid_Func_16(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) as ::core::ffi::c_int)
        < 2 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa66) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd2) as *mut uint16) = 0 as uint16;
        (*E).shitr_var_A = (((0xa9f2a2i32) as u16) & (0xffffu16));
        (*E).shitr_var_B = 0 as uint16;
        (*E).shitr_var_C = 0 as uint16;
        Enemy_SetInstrList(k, addr_kShitroid_Ilist_F906 as uint16, game);
        (*E).shitr_var_E = 10 as uint16;
        CallSomeSamusCode(0x13 as uint16, game);
        SomeMotherBrainScripts(0 as uint16, game);
        (*E).shitr_var_04 = 0 as uint16;
        QueueMusic_Delayed8(7 as uint16, game);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xccc) as *mut uint16) = 8 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd2) as *mut uint16) = 5 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa66) as *mut uint16) = 2 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xb2e) as *mut uint16) as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb2e) as *mut uint16) = 2 as uint16;
        }
        let mut v2: ::core::ffi::c_int = ((*E).base.frame_counter as ::core::ffi::c_int
            & 6 as ::core::ffi::c_int)
            >> 1 as ::core::ffi::c_int;
        (*E).base.x_pos = ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
            .wrapping_add(((g_word_A993BB[v2 as usize]) as u16)));
        (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
            as ::core::ffi::c_int
            + g_word_A993C3[v2 as usize] as ::core::ffi::c_int
            - 20 as ::core::ffi::c_int) as uint16;
        Samus_DamageDueToShitroid(game);
    };
}
pub unsafe fn Shitroid_Func_17(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    (*E).shitr_var_A = (((0xa9f2aei32) as u16) & (0xffffu16));
    (*E).shitr_var_F = 120 as uint16;
    Shitroid_Func_18(k, game);
}
pub unsafe fn Shitroid_Func_18(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if (*E).shitr_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).shitr_var_A = (((0xa9f2c0i32) as u16) & (0xffffu16));
        (*E).shitr_var_F = 192 as uint16;
        Shitroid_Func_19(k, game);
    }
}
pub unsafe fn Shitroid_Func_19(mut k: uint16, game: &mut Game) {
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16),
        104 as uint16,
        game,
    );
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if (*E).shitr_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        QueueSfx2_Max6(0x7d as uint16, game);
        (*E).shitr_var_A = (((0xa9f2fbi32) as u16) & (0xffffu16));
        (*E).shitr_var_F = 88 as uint16;
        (*E).base.current_instruction = addr_kShitroid_Ilist_F924 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        Shitroid_Func_20(game);
    }
}
pub unsafe fn Shitroid_Func_20(game: &mut Game) {
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0 as uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
            .wrapping_sub((64u16))),
        100 as uint16,
        game,
    );
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if (*E).shitr_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).shitr_var_A = (((0xa9f324i32) as u16) & (0xffffu16));
        (*E).shitr_var_F = 88 as uint16;
        Shitroid_Func_21(game);
    }
}
pub unsafe fn Shitroid_Func_21(game: &mut Game) {
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0 as uint16,
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
            .wrapping_add((96u16))),
        104 as uint16,
        game,
    );
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if (*E).shitr_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).shitr_var_A = (((0xa9f3a3i32) as u16) & (0xffffu16));
        (*E).shitr_var_F = 256 as uint16;
        (*E).base.current_instruction = addr_kShitroid_Ilist_F93A as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn Shitroid_Func_22(mut k: uint16, game: &mut Game) {
    QueueSfx2_Max6(0x52 as uint16, game);
    (*Get_Shitroid(k as ::core::ffi::c_int, game)).shitr_var_A =
        (((0xa9f36di32) as u16) & (0xffffu16));
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
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0 as uint16,
        rect.x,
        rect.y,
        game,
    );
    if Shitroid_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        rect,
        game,
    ) == 0
    {
        let mut E: *mut Enemy_Shitroid = Get_Shitroid(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*E).shitr_var_B = 0 as uint16;
        (*E).shitr_var_C = 0 as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_DisableSamusColl as ::core::ffi::c_int
                | kEnemyProps_Invisible as ::core::ffi::c_int))
            as uint16;
        (*E).shitr_var_A = (((0xa9efdfi32) as u16) & (0xffffu16));
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
        (*E).shitr_var_A = (((0xa9f3bei32) as u16) & (0xffffu16));
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
        (((0xa9f360i32) as u16) & (0xffffu16));
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
        (((*E).base.x_pos)
            .wrapping_sub((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)))),
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
        (*E).shitr_var_01 = ((shitro_var_01).wrapping_sub((1u16)));
        v5 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
            .wrapping_sub((18u16)));
    } else {
        v5 = 80 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
            & 0xfff as ::core::ffi::c_int
            >= 0xfe0 as ::core::ffi::c_int
        {
            (*E).shitr_var_01 = 32 as uint16;
        }
    }
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        8 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16),
        v5,
        game,
    );
    return ((*Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .shitr_var_02 as ::core::ffi::c_int
        >= 0x400 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            - 128 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0) as uint8;
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
    v3 = ((((*E).base.y_pos) as i16).wrapping_sub(((r20) as i16)));
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
            v9 = ((shitr_var_C).wrapping_sub(((r22) as i16)));
            if v9 as ::core::ffi::c_int + 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v9 = -(1280 as ::core::ffi::c_int) as int16;
            }
            (*E).shitr_var_C = v9 as uint16;
        } else {
            let mut v4: uint16 = SnesDivide(((r20).wrapping_sub(((*E).base.y_pos))), r24 as uint8);
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
            v6 = (((r22_0) as i16).wrapping_add((v5)));
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
    v2 = ((((*E).base.x_pos) as i16).wrapping_sub(((r18) as i16)));
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
                v5 = (((Shitroid_CheckIfOnScreen(k, game)) as i8) & (1i8));
                let mut v10: uint16 = v13;
                if v5 != 0 {
                    v10 = ((v13).wrapping_sub((r26)));
                }
                shitr_var_B = (v10 as ::core::ffi::c_int
                    - 8 as ::core::ffi::c_int
                    - r22 as ::core::ffi::c_int) as int16;
            }
            v11 = ((shitr_var_B).wrapping_sub(((r22) as i16)));
            if v11 as ::core::ffi::c_int + 2048 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v11 = -(2048 as ::core::ffi::c_int) as int16;
            }
            (*E).shitr_var_B = v11 as uint16;
        } else {
            let mut v3: uint16 = SnesDivide(((r18).wrapping_sub(((*E).base.x_pos))), r24 as uint8);
            if v3 == 0 {
                v3 = 1 as uint16;
            }
            let mut r22_0: uint16 = v3;
            v4 = (*E).shitr_var_B as int16;
            if (v4 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                let mut v12: uint16 = (*E).shitr_var_B;
                v5 = (((Shitroid_CheckIfOnScreen(k, game)) as i8) & (1i8));
                let mut v6: uint16 = v12;
                if v5 != 0 {
                    v6 = ((r26).wrapping_add((v12)));
                }
                v4 = (r22_0 as ::core::ffi::c_int
                    + v6 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int) as int16;
            }
            v7 = (((r22_0) as i16).wrapping_add((v4)));
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
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int) as int16;
        if v3 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            if v3 as ::core::ffi::c_int - 416 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                x_pos = (*E).base.x_pos as int16;
                if x_pos as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                    v5 = (x_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int
                        - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                            as ::core::ffi::c_int) as int16;
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
    R28 = ((R28).wrapping_add((Shitroid_AccelerateTowardsX(k, r18, a, game))));
    R28 = ((R28).wrapping_add((Shitroid_AccelerateTowardsY(k, r20, a, game))));
    R28 = (R28 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    return (((R28) as u8) & (1u8));
}
pub unsafe fn Shitroid_AccelerateTowardsY(
    mut k: uint16,
    mut r20: uint16,
    mut r22: uint16,
    game: &mut Game,
) -> uint16 {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(k as ::core::ffi::c_int, game);
    let mut result: uint16 = 0 as uint16;
    let mut v2: int16 = ((((*E).base.y_pos) as i16).wrapping_sub(((r20) as i16)));
    if v2 == 0 {
        result = result.wrapping_add(1);
    } else if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        let mut v7: uint16 = (((*E).shitr_var_C).wrapping_sub((r22)));
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
        let mut v3: uint16 = ((r22).wrapping_add(((*E).shitr_var_C)));
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
    if ((((*E).base.x_pos) as i16).wrapping_sub(((r18) as i16))) as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        let mut v6: uint16 = (((*E).shitr_var_B).wrapping_sub((r22)));
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
        let mut v2: uint16 = ((r22).wrapping_add(((*E).shitr_var_B)));
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
    Shitroid_HandleCutscenePalette_Common(addr_word_A9F6D1 as uint16, 330 as uint16, game);
}
pub unsafe fn Shitroid_HandleCutscenePalette(game: &mut Game) {
    Shitroid_HandleCutscenePalette_Common(addr_word_A9F6D1 as uint16, 490 as uint16, game);
}
pub unsafe fn Shitroid_HandleCutscenePalette_LowHealth(game: &mut Game) {
    Shitroid_HandleCutscenePalette_Common(addr_word_A9F711 as uint16, 490 as uint16, game);
}
pub unsafe fn Shitroid_HandleCutscenePalette_Common(
    mut r22: uint16,
    mut r18: uint16,
    game: &mut Game,
) {
    let mut shitro_var_D_high: int8 = 0;
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    shitro_var_D_high = *(&raw mut (*E).shitr_var_D as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) as int8;
    if shitro_var_D_high != 0 {
        *(&raw mut (*E).shitr_var_D as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) = (((shitro_var_D_high) as u8).wrapping_sub((1u8)));
    } else {
        *(&raw mut (*E).shitr_var_D as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) = (*E).shitr_var_E as uint8;
        let mut v3: uint8 = (*(&raw mut (*E).shitr_var_D as *mut uint8).offset(0)
            as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int) as uint8;
        *(&raw mut (*E).shitr_var_D as *mut uint8).offset(0) = v3;
        let mut v4: uint16 = Shitroid_HandleCrySoundEffect(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
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
            let mut v3: uint16 = (((*E).shitr_var_05).wrapping_add((1u16)));
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
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).shitr_parameter_2 != 0 {
        Shitroid_Func_26(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as ::core::ffi::c_int
            == 3 as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - 512 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            let mut r18: uint16 = (0x80 as ::core::ffi::c_int
                - CalculateAngleFromXY(
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
                        .wrapping_sub(((*E).base.x_pos))),
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
                        .wrapping_sub(((*E).base.y_pos))),
                    game,
                ) as ::core::ffi::c_int
                + 0x80 as ::core::ffi::c_int) as uint8 as uint16;
            (*E).shitr_var_B =
                (((*E).shitr_var_B).wrapping_add((Math_MultBySin(0x40 as uint16, r18))));
            (*E).shitr_var_C =
                (((*E).shitr_var_C).wrapping_add((Math_MultByCos(0x40 as uint16, r18))));
        } else if (*E).shitr_var_A as ::core::ffi::c_int
            == 0xa9f1fa as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            if Shitroid_AccelerateTowardsPoint(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                0x200 as uint16,
                *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16),
                ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
                    .wrapping_sub((32u16))),
                game,
            ) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                != 0
            {
                (*E).base.current_instruction = addr_kShitroid_Ilist_F924 as uint16;
                (*E).base.instruction_timer = 1 as uint16;
                (*E).base.timer = 0 as uint16;
                (*E).shitr_var_E = 1 as uint16;
                (*E).shitr_parameter_2 = 0 as uint16;
                (*E).shitr_var_B = 0 as uint16;
                (*E).shitr_var_C = 0 as uint16;
                (*E).shitr_var_A = (((0xa9f20ei32) as u16) & (0xffffu16));
            }
        } else if (*E).shitr_var_A as ::core::ffi::c_int
            == 0xa9f180 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            (*E).shitr_var_A = (((0xa9f1fai32) as u16) & (0xffffu16));
        }
    }
}
pub unsafe fn Shitroid_Shot(game: &mut Game) {
    let mut E: *mut Enemy_Shitroid = Get_Shitroid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).shitr_parameter_2 != 0 {
        Shitroid_Func_26(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
        let mut r18: uint16 = -(CalculateAngleFromXY(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(0))
                .wrapping_sub(((*E).base.x_pos))),
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(0))
                .wrapping_sub(((*E).base.y_pos))),
            game,
        ) as ::core::ffi::c_int) as uint8 as uint16;
        let mut v1: uint16 = ((8u16).wrapping_mul(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xc2c) as *mut uint16).offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16) as isize,
            )),
        ));
        if v1 as ::core::ffi::c_int >= 0xf0 as ::core::ffi::c_int {
            v1 = 240 as uint16;
        }
        (*E).shitr_var_B = (((*E).shitr_var_B).wrapping_add((Math_MultBySin(v1, r18))));
        (*E).shitr_var_C = (((*E).shitr_var_C).wrapping_add((Math_MultByCos(v1, r18))));
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
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return *jp.offset(0) as uintptr_t as *const uint16;
    }
    QueueSfx2_Max6(0x52 as uint16, game);
    return jp.offset(1);
}
